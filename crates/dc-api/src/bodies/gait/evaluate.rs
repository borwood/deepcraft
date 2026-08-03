//! Evaluating a baked [`GaitVector`] at a dimensionless speed — the other half
//! of "bake PARAMETERS, not frames" (posture-gait § 6 property 1). The bake
//! emits closed forms; everything speed-dependent is computed here, at the
//! call, from f64 coefficients. **There is no table over Fr and there must not
//! become one**: a table quantizes duty and would make a 3 % limp
//! inexpressible, which is the single highest-risk foreclosure the bones name.
//!
//! Cost, bounded rather than measured (runtime is first-class): [`GaitVector::at`]
//! is two `powf`, one `asin`, one `sqrt` and a per-limb pass — evaluated only
//! when speed changes, not per frame. Per-frame sampling is
//! [`GaitVector::limb_pose`]: one lerp per joint, the same shape as the clip
//! sampler it replaces.

use super::super::JointAngle;
use super::{CADENCE_EXPONENT, GaitVector, LimbGait, STRIDE_EXPONENT};

/// Which side of the published walk/run transition a speed sits on. **Not a
/// gait switch** — Fr is continuous and this is a *report* about where on it we
/// are, which is why nothing here branches the derivation on it except the one
/// thing that genuinely cannot be derived (the flight phase's root height).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Regime {
    /// β > 0.5: at least one contact is always down.
    Walk,
    /// β < 0.5: a flight phase exists.
    Run,
}

/// One derived quantity measured against its published band. **Reported, never
/// refused** — the docket's (b)3: a clockwork golem may want to step wrong.
#[derive(Clone, PartialEq, Debug)]
pub struct BandReport {
    /// The quantity.
    pub quantity: String,
    /// Its derived value.
    pub value: f64,
    /// The published band it is measured against, and the citation.
    pub band: String,
    /// What it means, and the heir if the gap is physics we have not built.
    pub note: String,
}

/// The root's vertical story — **the ONE composition** (design § 4.3). There is
/// no "bob" term anywhere: root height is a function of phase, and "bob" is the
/// name of the *observable* oscillation. A quantity gets a name; an observation
/// does not get a field. Consumed by both the IK hip and the render root, which
/// is what makes corrections #80's two-expression drift structurally impossible
/// rather than merely fixed.
#[derive(Clone, PartialEq, Debug)]
pub enum RootHeight {
    /// Derived from stance-chain geometry under the contact constraint.
    Derived {
        /// Peak-to-trough excursion as a fraction of chain reach — the
        /// scale-free number, identical across bodies at equal Fr.
        amplitude_ratio: f64,
        /// …and in metres, re-derived from the plan's own lengths.
        amplitude_m: f64,
        /// Troughs per limb cycle: one per bearing contact, so a biped gets 2
        /// (the measured 0.5 s against a 1.0 s clip, derived rather than typed)
        /// and a quadruped 4. **Derived, not a special case.**
        minima_per_cycle: usize,
    },
    /// A flight phase's apex is ballistic and needs takeoff force, which does
    /// not exist at density ≡ 1. Declined with the number and the heir —
    /// never fabricated (`stubs.md` #39, heir B6).
    Declined {
        /// Why, with the Froude number and the band.
        reason: String,
    },
}

/// One limb's `(phase, amplitude, duty)` triple, at a speed.
#[derive(Clone, PartialEq, Debug)]
pub struct LimbAtSpeed {
    /// The chain's contact-first head — the key into [`GaitVector::limbs`].
    pub contact_segment: String,
    /// Whether this chain bears.
    pub bearing: bool,
    /// Cycle-fraction offset.
    pub phase: f64,
    /// Signed traversal gain.
    pub amplitude: f64,
    /// Fraction of the cycle in stance, after the mean-preserving bias. For a
    /// non-bearing chain there is no stance and this is 0.5 — the symmetric
    /// fore/aft split of its pendulum, **not** a duty factor.
    pub duty: f64,
}

/// A gait vector evaluated at one dimensionless speed.
#[derive(Clone, PartialEq, Debug)]
pub struct GaitAtSpeed {
    /// `Fr = v²/(gL)`.
    pub froude: f64,
    /// The ground speed that Froude number means for this body, m/s.
    pub speed_m_s: f64,
    /// `λ = 2.3·L·Fr^0.3`, metres.
    pub stride_m: f64,
    /// One footfall's advance, `λ/contacts`, metres.
    pub step_m: f64,
    /// `f = √(g/L)·Fr^0.2/2.3`, Hz.
    pub cadence_hz: f64,
    /// `T = 1/f`, seconds.
    pub period_s: f64,
    /// Hip excursion half-angle, radians: `sin θmax = step/(2L)`.
    pub theta_max_rad: f64,
    /// The derived mean duty `β̄(Fr)` before any bias.
    pub mean_duty: f64,
    /// Which side of the published transition.
    pub regime: Regime,
    /// The per-limb triple.
    pub limbs: Vec<LimbAtSpeed>,
    /// The root's vertical story, or the honest refusal to invent one.
    pub root_height: RootHeight,
    /// Every derived term measured against its published band.
    pub bands: Vec<BandReport>,
}

impl GaitVector {
    /// `Fr = v²/(gL)` — the dimensionless speed the whole chain is written in.
    #[must_use]
    pub fn froude(&self, speed_m_s: f64) -> f64 {
        speed_m_s * speed_m_s / (self.gravity_m_s2 * self.governing_reach_m)
    }

    /// The inverse: what ground speed a Froude number means for *this* body.
    #[must_use]
    pub fn speed_for(&self, froude: f64) -> f64 {
        (froude * self.gravity_m_s2 * self.governing_reach_m).sqrt()
    }

    /// `λ(Fr) = k_λ · Fr^0.3` (Alexander 1976).
    #[must_use]
    pub fn stride_m(&self, froude: f64) -> f64 {
        self.stride_coefficient_m * froude.powf(STRIDE_EXPONENT)
    }

    /// `f(Fr) = k_f · Fr^0.2` — a corollary of the stride regression, not an
    /// independent claim.
    #[must_use]
    pub fn cadence_hz(&self, froude: f64) -> f64 {
        self.cadence_coefficient_hz * froude.powf(CADENCE_EXPONENT)
    }

    /// The derived mean duty. Clamped to 1.0 at the slow end, where the law
    /// leaves its published domain (a cycle cannot be more than all stance).
    #[must_use]
    pub fn mean_duty(&self, froude: f64) -> f64 {
        (super::TRANSITION_DUTY
            * (self.knobs.transition_fr / froude).powf(self.knobs.duty_exponent))
        .min(1.0)
    }

    /// Hip excursion half-angle. `sin θmax = (λ/contacts)/(2L)` — the step is
    /// one footfall's advance, so a biped's is `λ/2` and a quadruped's `λ/4`
    /// with no special case. Saturates at π/2 when the stride the regression
    /// wants is longer than the chain can span; that saturation is reported.
    #[must_use]
    pub fn theta_max_rad(&self, froude: f64) -> f64 {
        let s = self.stride_m(froude) / self.contacts as f64;
        (s / (2.0 * self.governing_reach_m)).clamp(-1.0, 1.0).asin()
    }

    /// `contact(θ) = neutral + θ · contact_per_rad`, the third keyframe
    /// reconstituted at a speed. `+θmax` is touchdown (foot forward).
    #[must_use]
    pub fn contact_pose(&self, limb: &LimbGait, froude: f64) -> Vec<JointAngle> {
        let t = self.theta_max_rad(froude);
        add_scaled(&limb.neutral, &limb.contact_per_rad, t)
    }

    /// The signed, phase-driven traversal (**G6**): `+amplitude` at touchdown,
    /// `−amplitude` at liftoff, and the returned blend says how far toward
    /// `clearance` the swing has carried the chain (0 at both contacts, 1 at
    /// mid-swing). A non-bearing chain gets a clean symmetric pendulum — its
    /// `clearance` IS its rest, so the blend is a no-op and is returned as 0.
    #[must_use]
    pub fn traversal(&self, limb: &LimbGait, phase: f64) -> (f64, f64) {
        let window = if limb.bearing {
            limb.compass_window
        } else {
            0.5
        };
        let u = (phase - limb.phase).rem_euclid(1.0);
        if window <= 0.0 || window >= 1.0 {
            return (limb.amplitude * (1.0 - 2.0 * u), 0.0);
        }
        if u < window {
            (limb.amplitude * (1.0 - 2.0 * u / window), 0.0)
        } else {
            let s = (u - window) / (1.0 - window);
            let blend = if limb.bearing {
                1.0 - (2.0 * s - 1.0).abs()
            } else {
                0.0
            };
            (limb.amplitude * (-1.0 + 2.0 * s), blend)
        }
    }

    /// One limb's sampled pose — the three keyframes composed by the traversal.
    /// This is the whole per-frame cost, and it is the same shape as the clip
    /// sampler it replaces: one lerp per joint.
    #[must_use]
    pub fn limb_pose(&self, limb: &LimbGait, froude: f64, phase: f64) -> Vec<JointAngle> {
        let (a, blend) = self.traversal(limb, phase);
        let swung = add_scaled(
            &limb.neutral,
            &limb.contact_per_rad,
            a * self.theta_max_rad(froude),
        );
        if blend <= 0.0 {
            return swung;
        }
        swung
            .iter()
            .zip(&limb.clearance)
            .map(|(s, c)| JointAngle {
                segment: s.segment.clone(),
                euler: [
                    s.euler[0] * (1.0 - blend) + c.euler[0] * blend,
                    s.euler[1] * (1.0 - blend) + c.euler[1] * blend,
                    s.euler[2] * (1.0 - blend) + c.euler[2] * blend,
                ],
            })
            .collect()
    }

    /// **The one vertical composition** — `root_offset(posture, mode, phase)`,
    /// with the speed axis G2 requires. Returns the root height as a fraction
    /// of chain reach, or `None` where it is honestly not derivable (a run's
    /// flight phase).
    ///
    /// During its compass window a stance chain is a rigid link about a pinned
    /// contact, so the attachment joint traces a circle: `h = L·cos θ`, peaking
    /// at midstance and troughing at the contact extremes. The windows tile the
    /// cycle, so `h` is continuous and single-valued and there is no toggle
    /// anywhere: **a body bobs iff its stance contacts alternate, and by exactly
    /// as much as its own geometry says.**
    #[must_use]
    pub fn root_height_ratio_at(&self, froude: f64, phase: f64) -> Option<f64> {
        if froude >= self.knobs.transition_fr {
            return None;
        }
        let theta_max = self.theta_max_rad(froude);
        let mut governing: Option<f64> = None;
        for limb in self.limbs.iter().filter(|l| l.bearing) {
            let u = (phase - limb.phase).rem_euclid(1.0);
            if u >= limb.compass_window {
                continue;
            }
            let theta = theta_max * (1.0 - 2.0 * u / limb.compass_window);
            let h = 1.0 - self.knobs.bob_damping * (1.0 - theta.cos());
            governing = Some(governing.map_or(h, |g: f64| g.min(h)));
        }
        governing
    }

    /// Evaluate every speed-dependent term at one Froude number.
    #[must_use]
    pub fn at(&self, froude: f64) -> GaitAtSpeed {
        let stride_m = self.stride_m(froude);
        let cadence_hz = self.cadence_hz(froude);
        let theta_max_rad = self.theta_max_rad(froude);
        let mean_duty = self.mean_duty(froude);
        let regime = if froude < self.knobs.transition_fr {
            Regime::Walk
        } else {
            Regime::Run
        };
        let limbs = self.limbs_at(mean_duty);
        let mut bands = Vec::new();

        let raw_sin = stride_m / (self.contacts as f64 * 2.0 * self.governing_reach_m);
        if raw_sin > 1.0 {
            bands.push(BandReport {
                quantity: "sin θmax".into(),
                value: raw_sin,
                band: "[0, 1] — a chain cannot span more than twice its own reach".into(),
                note: "the stride regression asks for a step this chain cannot span; the \
                       excursion saturated at π/2 and the derived gait is out of domain here \
                       (heir: B6 — a real leg does this with a flight phase)"
                    .into(),
            });
        }
        bands.push(BandReport {
            quantity: "duty β̄".into(),
            value: mean_duty,
            band: format!(
                "> {} walking, < {} running; transition at Fr ≈ {} (Alexander & Jayes 1983)",
                super::TRANSITION_DUTY,
                super::TRANSITION_DUTY,
                self.knobs.transition_fr
            ),
            note: format!("Fr = {froude:.4} — the literature calls this a {regime:?}"),
        });
        // The rigid-chain model can only express instantaneous double support:
        // with the compass windows tiling the cycle the geometric duty is
        // 1/contacts, flat. The gap to the published β̄ is exactly the
        // stance-knee flexion and foot roll of § 1's FORCE column — reported so
        // nobody reads the derived poses as claiming a double-support phase.
        let geometric_duty = 1.0 / self.contacts as f64;
        bands.push(BandReport {
            quantity: "geometric duty (compass window)".into(),
            value: geometric_duty,
            band: format!("published β̄ = {mean_duty:.4} at this Fr"),
            note: format!(
                "the rigid chain supports for {:.1} % of the cycle against a published \
                 {:.1} %; the difference is stance-knee flexion, pelvic list and ankle roll \
                 — FORCE, which density ≡ 1 does not have (heir: B6). β̄ is emitted for \
                 contact scheduling; the POSES use the geometric window, because a pose set \
                 built on β > 1/contacts would plant two feet at two different heights — \
                 the exact 46 mm contradiction the authored clip carries",
                geometric_duty * 100.0,
                mean_duty * 100.0
            ),
        });

        let root_height = if regime == Regime::Run {
            RootHeight::Declined {
                reason: format!(
                    "Fr = {froude:.4} ≥ the transition {:.2}: duty {mean_duty:.4} < 0.5, so \
                     this gait has a FLIGHT PHASE whose apex is ballistic. Takeoff force does \
                     not exist at density ≡ 1, so the root height is DECLINED rather than \
                     fabricated (stubs.md #39; heir: B6, per-segment materials)",
                    self.knobs.transition_fr
                ),
            }
        } else {
            let amplitude_ratio = self.knobs.bob_damping * (1.0 - theta_max_rad.cos());
            RootHeight::Derived {
                amplitude_ratio,
                amplitude_m: amplitude_ratio * self.governing_reach_m,
                minima_per_cycle: self.limbs.iter().filter(|l| l.bearing).count(),
            }
        };

        GaitAtSpeed {
            froude,
            speed_m_s: self.speed_for(froude),
            stride_m,
            step_m: stride_m / self.contacts as f64,
            cadence_hz,
            period_s: 1.0 / cadence_hz,
            theta_max_rad,
            mean_duty,
            regime,
            limbs,
            root_height,
            bands,
        }
    }

    /// Per-limb duty: a **mean-preserving** bias plus one deterministic
    /// re-centring pass if a clamp fired. Mean preservation is the right
    /// invariant because `β̄` is what Fr pins — cadence and stride are unchanged
    /// by a limp, which is why a limp reads as *timing* and not as a different
    /// speed. Never a per-limb override.
    fn limbs_at(&self, mean_duty: f64) -> Vec<LimbAtSpeed> {
        let bearing: Vec<usize> = (0..self.limbs.len())
            .filter(|i| self.limbs[*i].bearing)
            .collect();
        let mean_bias = if bearing.is_empty() {
            0.0
        } else {
            bearing.iter().map(|i| self.limbs[*i].duty_bias).sum::<f64>() / bearing.len() as f64
        };
        let mut duty: Vec<f64> = self
            .limbs
            .iter()
            .map(|l| {
                if l.bearing {
                    (mean_duty + l.duty_bias - mean_bias).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            })
            .collect();
        if !bearing.is_empty() {
            let got = bearing.iter().map(|i| duty[*i]).sum::<f64>() / bearing.len() as f64;
            let deficit = mean_duty - got;
            if deficit != 0.0 {
                let free: Vec<usize> = bearing
                    .iter()
                    .copied()
                    .filter(|i| duty[*i] > 0.0 && duty[*i] < 1.0)
                    .collect();
                if !free.is_empty() {
                    let share = deficit * bearing.len() as f64 / free.len() as f64;
                    for i in free {
                        duty[i] = (duty[i] + share).clamp(0.0, 1.0);
                    }
                }
            }
        }
        self.limbs
            .iter()
            .zip(duty)
            .map(|(l, d)| LimbAtSpeed {
                contact_segment: l.contact_segment.clone(),
                bearing: l.bearing,
                phase: l.phase,
                amplitude: l.amplitude,
                duty: d,
            })
            .collect()
    }
}

/// `base + k · gradient`, joint by joint. The gradient carries the segment
/// names, so a mismatched pair is a bug the zip would hide — asserted in debug.
fn add_scaled(base: &[JointAngle], gradient: &[JointAngle], k: f64) -> Vec<JointAngle> {
    debug_assert_eq!(base.len(), gradient.len());
    base.iter()
        .zip(gradient)
        .map(|(b, g)| {
            debug_assert_eq!(b.segment, g.segment);
            JointAngle {
                segment: b.segment.clone(),
                euler: [
                    b.euler[0] + k * g.euler[0],
                    b.euler[1] + k * g.euler[1],
                    b.euler[2] + k * g.euler[2],
                ],
            }
        })
        .collect()
}
