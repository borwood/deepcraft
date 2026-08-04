//! The gait bake — member #1 of the posture-gait tier
//! (`docs/design/posture-gait.md` § 7; spec: the greenlit design pass
//! `docs/audits/2026-08-02-gait-bake-member1-design.md`, whose header carries
//! the five user rulings and the build greenlight).
//!
//! A **pure function of the body definition plus the world's gravity** — no
//! world query, no RNG, no clock — so it is callable from ANY minting clock
//! (pack build, deeptime worldgen, define time), exactly as
//! [`super::bake_resting_posture`] is and for the same reason (corrections #86:
//! venue is the caller's; dc-worldgen can call dc-api). The result is
//! **returned, not stored**: a sibling of [`BodyPlan`], never a field of it
//! (S-3 — the plan stays the authority).
//!
//! # What it emits, and why it is COEFFICIENTS and not frames
//!
//! Speed is deliberately *not* an axis of the ratified bake key
//! `(species, posture-id, mode, yaw)`, and **every gait term is a function of
//! speed** (design finding **G2**). So the bake emits the *closed form*:
//! `k_λ = 2.3·L` and `k_f = √(g/L)/2.3` with their published exponents, the
//! duty law's two anchors, the clearance ratio — and [`GaitVector::at`]
//! evaluates them at a dimensionless speed (Froude number). A pre-computed
//! table over Fr is the obvious "make it fast" pass and is exactly the
//! foreclosure posture-gait § 6 property 1 names as highest-risk: it would
//! quantize duty and make a 3 % limp inexpressible.
//!
//! The same reasoning governs the three keyframe poses (**G1**, user call #4:
//! three, not two). `neutral` and `clearance` are genuinely speed-invariant and
//! are stored as poses. `contact` is **not** — its magnitude is the hip
//! excursion θmax(Fr) — so it is stored as [`LimbGait::contact_per_rad`], the
//! pose **gradient** (radians of joint rotation per radian of hip excursion),
//! and [`GaitVector::contact_pose`] multiplies. For a chain that is a rigid
//! rotation about its attachment joint — every chain member #0 accepts — the
//! gradient form is *exact*, not a linearisation, and it costs one multiply.
//! **The design's § 2 table lists `contact` as a stored pose in radians while
//! its § 9 demands coefficients over Fr; those two rows are in tension and this
//! is the resolution.**
//!
//! # The bake key's other axes
//!
//! `posture` is carried as this function's fixed meaning (`dc:posture/stand`),
//! the explicitly-marked placeholder, exactly as member #0 carries it; `yaw` is
//! 0 (straight-ahead travel) — a strafe or turn gait is a different yaw and is
//! not slice one's. `mode` names the **bearing set** (B0: support is a
//! per-segment capability activated per mode), not a gait name; which gait
//! *pattern* runs over that set is [`GaitKnobs`]' business.
//!
//! # Where the honest limit is, stated before anything is derived
//!
//! The input set is segment geometry plus volume as the mass proxy at density
//! ≡ 1. There is no strength, no muscle, no actuation cost. Everything derived
//! here is length scale and gravity; everything that needs FORCE is either a
//! band report or a stand-in knob with **B6** (per-segment materials) named as
//! heir. The bake never fabricates a force answer:
//!
//! - **the run is DECLINED** — β < 0.5 has a flight phase whose root height is
//!   ballistic (`stubs.md` #39, heir B6). Reported with the number, never
//!   fabricated;
//! - **mass ≡ volume** (`stubs.md` #40, heir B6) — inherited from member #0
//!   through the resting solve;
//! - **the speed the Froude number needs is the world's, not the body's**
//!   (`stubs.md` #41, heir B4) — which is why `gravity_m_s2` and the evaluation
//!   speed are arguments and not baked constants;
//! - **the binary `Loco` switch** that discarded the analog intent (`stubs.md`
//!   #42) is **retired** — the consumer slice deleted it 2026-08-03
//!   (journal/0147). Fr is continuous and idle is its degenerate limit;
//!   `dc-client`'s `the_gait_ladder_is_continuous_and_idle_is_its_limit` is
//!   what keeps a discrete gait switch from coming back.
//!
//! ⚠ **SEAM — B7 (joint rotation limits).** No plan, solver or validator in
//! this tree prevents a knee inverting: `SegmentDef` carries no rotation range
//! and dc-client's IK clamps only to keep `acos` in domain. When B7 lands, the
//! enforcement point is [`bake_gait`]'s pose emission — every emitted keyframe
//! is checked against the declared limits and a violation **refuses loudly**.
//! That seam is named here and deliberately NOT built (it is its own arc item).
//!
//! ⚠ **Report, never refuse, when a value leaves a published band** — a
//! clockwork golem may want to step wrong. Refusals here are reserved for
//! geometry the solve has no answer for at all.

use self::geometry::{
    Contact, band_notes, clearance_pose, compass_windows, counter_swing_limbs, knobs_are_finite,
    phase_offsets, sagittal_gradient,
};
use super::bake::offset_from_root;
use super::{
    BakeOutcome, BodyPlan, JointAngle, bake_resting_posture, segments_with_role, stance_chain,
};

mod evaluate;
mod geometry;
#[cfg(test)]
mod tests;

pub use evaluate::{BandReport, GaitAtSpeed, LimbAtSpeed, Regime, RootHeight};

// ------------------------------------------------------- published anchors --

/// Alexander (1976), *Estimates of the speeds of dinosaurs*, Nature
/// 261:129–130 — the trackway regression `λ = 2.3·L·Fr^0.3`. The coefficient.
pub const STRIDE_COEFFICIENT: f64 = 2.3;
/// …and its exponent.
pub const STRIDE_EXPONENT: f64 = 0.3;
/// Cadence `f = 1/T = v/λ = √(g/L)·Fr^0.2 / 2.3` — the exponent falls out of
/// the stride regression composed with `Fr = v²/(gL)`; it is `0.5 − 0.3`, not
/// an independent fit. **The bones' `f ∝ √(g/L)` is a corollary, not an
/// assertion.**
pub const CADENCE_EXPONENT: f64 = 0.5 - STRIDE_EXPONENT;
/// Alexander & Jayes (1983), *A dynamic similarity hypothesis for the gaits of
/// quadrupedal mammals*, J. Zool. 201:135–152 — the walk/run transition sits at
/// Fr ≈ 0.5, where duty crosses 0.5.
pub const TRANSITION_FR: f64 = 0.5;
/// Duty at the transition, by construction.
pub const TRANSITION_DUTY: f64 = 0.5;
/// Winter, *Biomechanics and Motor Control of Human Movement* — normal human
/// walking sits near β ≈ 0.60 at Fr ≈ 0.25. The second anchor of the duty law.
pub const WALK_DUTY_ANCHOR: f64 = 0.60;
/// …at this Froude number.
pub const WALK_DUTY_ANCHOR_FR: f64 = 0.25;
/// Winter (1992) — minimum toe clearance in level human walking ≈ 1.3 cm,
/// ≈ 1.5 % of leg length. Expressed as a **ratio of the chain's reach** so it
/// survives scaling.
pub const CLEARANCE_RATIO: f64 = 0.015;

/// The duty law's exponent, **derived from the two published anchors** rather
/// than typed: `β(Fr) = 0.5·(0.5/Fr)^e` with `β(0.25) = 0.60` forces
/// `e = ln(0.60/0.50) / ln(0.5/0.25)`. A constant with a derivation is
/// evidence; one chosen until an output looked right is not (CLAUDE.md
/// § Gates). *The design pass quotes 0.263; the exact value is 0.26303441…,
/// and using the exact one makes `β(0.25)` land on 0.600000000 rather than
/// 0.5999847.*
///
/// ⚠ **STAND-IN — `stubs.md` #44 (S2), heir B6.** The two *endpoints* are
/// published; the curve BETWEEN them is a regression, not a mechanism. B6 plus
/// a cost-of-transport minimisation derives `β(Fr)` per body and this exponent
/// is deleted, not re-tuned.
#[must_use]
pub fn duty_exponent() -> f64 {
    (WALK_DUTY_ANCHOR / TRANSITION_DUTY).ln() / (TRANSITION_FR / WALK_DUTY_ANCHOR_FR).ln()
}

// ------------------------------------------------------------------ knobs --

/// The authored knobs over the derivation. **Every one has an identity default
/// such that no-knob is byte-identical to the pure derivation** (S-5), and the
/// kind-3 stand-ins report — never refuse — when they push an output past a
/// published band.
///
/// The tell that separates the two groups: a **taste** knob has no true value
/// (a giraffe paces where a horse trots); a **stand-in** knob has a true value
/// we cannot currently compute. Every stand-in is a dimensionless multiplier or
/// exponent on a term that *survives* B6, so B6's arrival derives the
/// multiplier rather than colliding with it.
#[derive(Clone, PartialEq, Debug)]
pub struct GaitKnobs {
    /// **STAND-IN — `stubs.md` #44 (S1), heir B6.** Quicker-stepping than its size predicts =
    /// muscle power against limb inertia. Band `[0.8, 1.25]`.
    pub cadence_scale: f64,
    /// **STAND-IN — `stubs.md` #44 (S2), heir B6.** The duty curve's interior exponent; see
    /// [`duty_exponent`].
    pub duty_exponent: f64,
    /// **STAND-IN — `stubs.md` #44 (S3). ⚠ HEIR CORRECTED: NOT B6** (`stubs.md` #50,
    /// 2026-08-04 — *density is not stiffness*; a mass integral cannot yield an
    /// elastic modulus). Real heir: a mechanical-property axis on the material
    /// sheet plus a compliant joint model, neither designed. The rigid
    /// ("compass") chain over-predicts
    /// real vertical excursion — measured human walking ≈ 4.6 cm against a
    /// compass ≈ 6.6 cm (Saunders, Inman & Eberhart 1953), the gap being
    /// stance-knee flexion, pelvic list and ankle rocker: all joint stiffness,
    /// all FORCE. Identity 1.0 is the honest upper bound. Band `[0.5, 1.0]`.
    pub bob_damping: f64,
    /// **STAND-IN — `stubs.md` #44 (S4), heir B6.** Swing-leg energetics. Interpolates the
    /// mid-swing foot lift from the geometric clearance (0.0 — the compass,
    /// near-straight swing) to the highest the chain can tuck with the anchor
    /// under its attachment (1.0). Band `[0, 1]`.
    pub swing_flexion: f64,
    /// **TASTE (kind 1).** Above the geometric minimum, clearance is style.
    /// Identity is the published [`CLEARANCE_RATIO`].
    pub foot_clearance_ratio: f64,
    /// **TASTE (kind 2).** *Where* a species changes gait is biology; the
    /// published [`TRANSITION_FR`] is the band, not the answer.
    pub transition_fr: f64,
    /// **TASTE (kind 1).** Length scale fixes when feet land and says almost
    /// nothing about carriage between landings. Identity 1.0 = the derived
    /// counter-swing (the same angular excursion as the contralateral bearing
    /// chain).
    pub arm_swing_amplitude: f64,
}

impl Default for GaitKnobs {
    fn default() -> Self {
        Self {
            cadence_scale: 1.0,
            duty_exponent: duty_exponent(),
            bob_damping: 1.0,
            swing_flexion: 0.0,
            foot_clearance_ratio: CLEARANCE_RATIO,
            transition_fr: TRANSITION_FR,
            arm_swing_amplitude: 1.0,
        }
    }
}

/// The per-INSTANCE overlay (posture-gait § 6 property 2 — the S-9 shape: a
/// derivable species base, a sparse instance overlay, unforeclosed). The seam
/// is the function signature [`pose`]; the identity default reproduces the
/// species gait **bit for bit**, which is what makes adding an instance axis a
/// non-event.
///
/// A limp is a **duty bias plus deterministic renormalisation**, never a
/// per-limb override: `β̄` is what Fr pins, so a limp must leave the mean alone
/// — which is exactly why a limp reads as *timing* and not as a different
/// speed. f64 throughout; a 3 % limp is `0.03` and must stay expressible.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct InstanceDelta {
    /// Per-limb duty bias, keyed by contact-segment name. Mean-preserving.
    pub duty_bias: Vec<(String, f64)>,
    /// Per-limb amplitude gain, keyed by contact-segment name. Multiplicative;
    /// absent = 1.0.
    pub amplitude_gain: Vec<(String, f64)>,
}

// ----------------------------------------------------------------- output --

/// One limb's gait: the `(phase, amplitude, duty)` triple's speed-invariant
/// half, plus its three derived keyframe poses.
#[derive(Clone, PartialEq, Debug)]
pub struct LimbGait {
    /// The chain's contact-first head — the segment that touches down (a
    /// bearing chain) or the distal segment (a counter-swinging chain). The
    /// key an [`InstanceDelta`] addresses.
    pub contact_segment: String,
    /// The bearing role this chain was activated by, or `""` for a
    /// counter-swinging non-bearing chain.
    pub role: String,
    /// Whether this chain bears in the baked mode.
    pub bearing: bool,
    /// Cycle-fraction offset in `[0, 1)`, derived from contact geometry
    /// (never from names — B0's rule that laterality and pairing are DERIVED).
    pub phase: f64,
    /// Dimensionless gain on the signed, phase-driven traversal (**G6**: the
    /// bones' multiplicative-toward-`extreme` amplitude cannot express a signed
    /// excursion). 1.0 is the pure derivation.
    pub amplitude: f64,
    /// Mean-preserving duty bias; 0.0 is the pure derivation.
    pub duty_bias: f64,
    /// Chain reach in metres — a report number re-derived from the plan's own
    /// lengths, never a baked authority (decision 24).
    pub reach_m: f64,
    /// The fraction of the cycle over which this chain is the governing root
    /// height constraint (bearing chains only; 0.0 otherwise). Derived as the
    /// phase gap to the next contact, so the windows **tile the cycle exactly**
    /// — which is what makes `h(phase)` continuous and single-valued.
    pub compass_window: f64,
    /// Midstance / the zero-torque column — **member #0's resting pose**, read
    /// not re-derived (design § 3.2, confirmed: midstance IS the configuration
    /// in which the stance chain is a vertical column under its attachment).
    pub neutral: Vec<JointAngle>,
    /// The pose GRADIENT toward touchdown: radians of joint rotation per
    /// radian of hip excursion. `contact(θ) = neutral + θ · contact_per_rad`;
    /// see the module doc for why this is stored instead of a frozen pose.
    /// Positive X rotation carries the distal end forward (−Z), so `+θmax` is
    /// **touchdown** (foot forward) and `−θmax` is liftoff.
    pub contact_per_rad: Vec<JointAngle>,
    /// Mid-swing. For a bearing chain: the two-bone solve that puts the contact
    /// anchor directly under the attachment joint at the clearance height. For
    /// a non-bearing chain there is no ground constraint and mid-swing IS
    /// `neutral` — the arm hangs straight as it passes the hip, which is what
    /// the authored clip does.
    pub clearance: Vec<JointAngle>,
}

/// A baked gait: closed-form coefficients over dimensionless speed, plus the
/// per-limb vector. Returned, not stored.
#[derive(Clone, PartialEq, Debug)]
pub struct GaitVector {
    /// The plan this was baked from.
    pub plan: String,
    /// The bearing mode (the bake key's `mode` axis).
    pub mode: String,
    /// Gravity the Froude chain was closed against, m/s². **Not the body's** —
    /// see `stubs.md` #41.
    pub gravity_m_s2: f64,
    /// The governing stance chain's reach **L**, metres — a report number.
    pub governing_reach_m: f64,
    /// `k_λ = 2.3·L`, metres: `λ(Fr) = k_λ · Fr^0.3`.
    pub stride_coefficient_m: f64,
    /// `k_f = √(g/L)/2.3 · cadence_scale`, Hz: `f(Fr) = k_f · Fr^0.2`.
    pub cadence_coefficient_hz: f64,
    /// Number of bearing contacts in the cycle. One footfall per contact per
    /// cycle, so the **step** is `λ/contacts` — for a biped that is the
    /// familiar `λ/2`, and for a quadruped it is `λ/4` with no special case.
    pub contacts: usize,
    /// Per-limb gait, bearing chains first in the mode's declared order, then
    /// counter-swinging non-bearing chains.
    pub limbs: Vec<LimbGait>,
    /// The knobs this bake was closed with, echoed so a report is reproducible.
    pub knobs: GaitKnobs,
    /// Speed-invariant reports: a stand-in knob outside its published band, a
    /// chain declined for counter-swing, a girdle partition worth knowing
    /// about. **Never a refusal** — refusals are [`GaitBakeOutcome`]'s.
    pub notes: Vec<String>,
}

/// The gait bake's answer, mirroring [`BakeOutcome`]'s three shapes.
#[derive(Clone, PartialEq, Debug)]
pub enum GaitBakeOutcome {
    /// The gait for the named bearing mode.
    Baked(GaitVector),
    /// The plan declares zero modes — a tree. A legal absence, not an error.
    NothingDeclared,
    /// Out of the derivation's domain, with the offending geometry named and
    /// the heir where one is filed — never a silent nonsense gait.
    Unsupported {
        /// What took the plan out of domain, and where its heir is filed.
        reason: String,
    },
}

// ------------------------------------------------------------------- bake --

/// Bake the gait of `plan` for the declared bearing `mode` under `gravity_m_s2`
/// — pure, deterministic, no world involvement.
///
/// **Gravity is an argument, and that is a finding, not a convenience.** The
/// design pass derived its whole predicted table at `g = 9.81`; this world's
/// [`crate::CharacterConfig::gravity_m_s2`] is **25.0**. A gait baked against
/// Earth gravity would be dimensionally coherent and wrong for the world the
/// body walks in — the closed-system scale error, one tier down. The bake
/// therefore refuses to own `g` and makes the caller state it.
///
/// Refuses (never fabricates) when: the resting solve refuses (its reason is
/// propagated); gravity is not finite and positive; a bearing chain is not
/// two-boned (the mid-swing clearance solve is a two-bone analytic IK);
/// the clearance target is inside the chain's inner reach limit.
#[must_use]
pub fn bake_gait(
    plan: &BodyPlan,
    mode: &str,
    gravity_m_s2: f64,
    knobs: &GaitKnobs,
) -> GaitBakeOutcome {
    if !(gravity_m_s2.is_finite() && gravity_m_s2 > 0.0) {
        return GaitBakeOutcome::Unsupported {
            reason: format!(
                "gait bake: gravity must be finite and positive, got {gravity_m_s2} m/s² \
                 (the world owns g — see stubs.md #41)"
            ),
        };
    }
    if !knobs_are_finite(knobs) {
        return GaitBakeOutcome::Unsupported {
            reason: format!(
                "gait bake: plan `{}` was given a non-finite knob",
                plan.name
            ),
        };
    }

    // `neutral` IS member #0's resting pose. Read, never re-derived — and
    // where member #0 is honestly wrong (the bird's tendon-held folded rest),
    // the gait's `neutral` is wrong in exactly the same way, behind exactly the
    // same effort seam. One degradation, not two.
    let resting = match bake_resting_posture(plan, mode) {
        BakeOutcome::Baked(p) => p,
        BakeOutcome::NothingDeclared => return GaitBakeOutcome::NothingDeclared,
        BakeOutcome::Unsupported { reason } => {
            return GaitBakeOutcome::Unsupported {
                reason: format!("gait bake: the resting solve refused — {reason}"),
            };
        }
    };

    // Re-resolve the contacts in member #0's exact iteration order, so
    // `resting.chains[i]` and `contacts[i]` are the same chain.
    let mode_def = plan
        .modes
        .iter()
        .find(|m| m.mode == mode)
        .expect("the resting solve accepted this mode by name");
    let mut contacts: Vec<Contact> = Vec::new();
    for role in &mode_def.bearing {
        for seg in segments_with_role(plan, role) {
            for r in seg.roles.iter().filter(|r| r.role == *role) {
                if let Some(anchor) = r.at_m {
                    let base = offset_from_root(plan, seg);
                    contacts.push(Contact {
                        role: role.clone(),
                        seg,
                        anchor,
                        ground: [
                            base[0] + anchor[0],
                            base[1] + anchor[1],
                            base[2] + anchor[2],
                        ],
                    });
                }
            }
        }
    }
    debug_assert_eq!(contacts.len(), resting.chains.len());

    let reach = resting.chains[0].reach_m; // equal across chains (member #0 refuses otherwise)
    let n = contacts.len();
    let mut notes = band_notes(knobs);

    // § 3.3: girdles by fore-aft anchor z, order within a girdle by (x, z, y,
    // plan order). Every key is DERIVED geometry; no name is read.
    let phases = match phase_offsets(plan, &contacts, &mut notes) {
        Ok(p) => p,
        Err(reason) => return GaitBakeOutcome::Unsupported { reason },
    };

    // The compass windows: the phase gap to the next contact, wrapping. They
    // tile the cycle exactly by construction, which is what makes h(phase)
    // single-valued — see `evaluate::root_height_ratio_at`.
    let windows = compass_windows(&phases);

    let mut limbs: Vec<LimbGait> = Vec::with_capacity(n);
    for (i, c) in contacts.iter().enumerate() {
        let chain = stance_chain(plan, c.seg);
        let clearance = match clearance_pose(&chain, c.anchor, reach, knobs) {
            Ok(p) => p,
            Err(reason) => {
                return GaitBakeOutcome::Unsupported {
                    reason: format!("plan `{}` mode `{mode}`: {reason}", plan.name),
                };
            }
        };
        limbs.push(LimbGait {
            contact_segment: c.seg.name.clone(),
            role: c.role.clone(),
            bearing: true,
            phase: phases[i],
            amplitude: 1.0,
            duty_bias: 0.0,
            reach_m: resting.chains[i].reach_m,
            compass_window: windows[i],
            neutral: resting.chains[i].joints.clone(),
            contact_per_rad: sagittal_gradient(&chain),
            clearance,
        });
    }

    // G3: the bake needs a resting answer for NON-bearing chains too — a
    // swinging arm has no stance and member #0's solve has no place for one.
    // It turned out NOT to be a change to member #0's function: the derivation
    // is different (a leaf chain's distal face, not a declared contact anchor),
    // so it lives here and member #0 is untouched.
    let swinging = counter_swing_limbs(plan, &contacts, &limbs, knobs, &mut notes);
    limbs.extend(swinging);

    GaitBakeOutcome::Baked(GaitVector {
        plan: plan.name.clone(),
        mode: mode.to_string(),
        gravity_m_s2,
        governing_reach_m: reach,
        stride_coefficient_m: STRIDE_COEFFICIENT * reach,
        cadence_coefficient_hz: (gravity_m_s2 / reach).sqrt() / STRIDE_COEFFICIENT
            * knobs.cadence_scale,
        contacts: n,
        limbs,
        knobs: knobs.clone(),
        notes,
    })
}

/// The per-instance seam, `pose(species_gait, instance_delta)` (posture-gait
/// § 6 property 2). In a bake-PARAMETERS world "posing an instance" is deriving
/// its gait vector; the frame-level sample is [`GaitVector::at`] plus
/// [`GaitVector::traversal`]. **The identity default returns the species gait
/// bit for bit** — asserted, not asserted-about.
#[must_use]
pub fn pose(species_gait: &GaitVector, delta: &InstanceDelta) -> GaitVector {
    let mut out = species_gait.clone();
    for limb in &mut out.limbs {
        if let Some((_, b)) = delta
            .duty_bias
            .iter()
            .find(|(k, _)| *k == limb.contact_segment)
        {
            limb.duty_bias += *b;
        }
        if let Some((_, g)) = delta
            .amplitude_gain
            .iter()
            .find(|(k, _)| *k == limb.contact_segment)
        {
            limb.amplitude *= *g;
        }
    }
    out
}
