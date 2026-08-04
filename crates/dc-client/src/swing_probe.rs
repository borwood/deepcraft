//! **Where does the longleg's swing knee jump?** — the diagnostic probe for the
//! 2026-08-04 walk observation (report:
//! `docs/audits/2026-08-04-longleg-swing-probe.md`).
//!
//! > *"the longer legged one has a little weirdness in the forward swing of
//! > walking leg which i dont see in the stout … what it looks like is the leg
//! > doing a **mini bend and correction back to straight and back to bent**
//! > during the swing forward. very hard to see in realtime."*
//!
//! *"Very hard to see in realtime"* is why this is a probe rather than another
//! walk: the artifact is sampled here at **thousands of phases per stride**, far
//! above any pose grid, and the discontinuity is found **numerically**.
//!
//! # What it drives
//!
//! The **production** pose path end to end, with two mirrored pieces:
//!
//! | production | here |
//! |---|---|
//! | [`crate::body::pose_for`], [`crate::body::root_offset_m`] | called |
//! | [`crate::body::fk_foot_local`], [`crate::body::solve_leg_ik`] | called |
//! | [`crate::body::leg_rigs`], [`crate::body::derived_gait`] | called |
//! | `character.rs::ground_top_m` (needs a bevy `VoxelQuery`) | **mirrored** by [`ground_top_level`] |
//! | `character.rs`'s window/limit decision | **mirrored** by [`Decision::of`] |
//!
//! The two mirrors are the honest cost of measuring a render path headlessly —
//! the same cost `body::retarget_report` already pays and names. They are
//! written for the **level-ground** case only, where the voxel scan has a closed
//! form; a step or an overhang would need the real query.
//!
//! # The two traces, and why both
//!
//! - **continuous** — [`dc_api::bodies::GaitVector::limb_pose`] at the raw
//!   phase. This is the *geometry*: if it is C0-continuous, nothing in the gait
//!   or the solver is jumping.
//! - **rendered** — [`crate::body::pose_for`] at the same raw phase, which
//!   quantizes onto the body's own `N`-pose stride grid (journal/0155). This is
//!   what the eye sees, staircase included.
//!
//! Separating them is the whole discrimination between candidate 3
//! (quantization) and candidates 1/2/4 (a real discontinuity): a staircase whose
//! step count tracks `N` is quantization; a jump that survives refinement of the
//! sampling grid **and** shows in the continuous trace is geometry.
//!
//! # Reading the jump statistic
//!
//! [`Trace::max_step`] is the largest change between **adjacent samples**. For a
//! C0-continuous signal that is `|f′| · Δphase`, so **doubling the sample count
//! halves it**; a genuine jump is invariant under refinement. That ratio — not
//! any absolute tolerance — is the scale-free invariant the gate asserts.

use std::collections::HashMap;

use dc_api::Posture;
use dc_api::bodies::{AnimClip, BodyPlan, JointAngle};

use crate::anim_rate::AnimRate;
use crate::body::{
    AnimState, LegRig, Reach, derived_gait, derived_root_delta_m, fk_foot_local, leg_rigs,
    pose_for, root_offset_m, solve_leg_ik,
};

/// The walk's conditions: `speed: 0.3` of `CharacterConfig::walk_speed_m_s`.
pub const WALK_SPEED_M_S: f64 = 4.5;

/// The active client scale at boot (`app.rs`'s `boot_voxels = 2`, player 1.8 m).
pub const VOXEL_M: f64 = 1.8 / 2.0;

/// What the renderer did with this foot on this sample — the mirror of
/// `character.rs`'s two gates, kept as one enum so a *state change* is a value
/// change and not three booleans nobody compares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decision {
    /// `|adjust| <= 1e-3` — the gait already seated the sole; clip pose stands.
    Seated,
    /// No solid in the scanned band, or `|adjust| > half_voxel` — outside the
    /// window; the foot floats honestly.
    OutsideWindow,
    /// Inside the window, solver refused on a joint limit; clip pose stands.
    RefusedByLimits(&'static str),
    /// Inside the window, the solve was applied. Carries the reach verdict:
    /// `Ok` and `BeyondExtension` are **both applied** and are different states.
    Applied(&'static str),
}

impl Decision {
    /// The label that goes in the trace table.
    pub fn code(self) -> &'static str {
        match self {
            Decision::Seated => "seat",
            Decision::OutsideWindow => "outw",
            Decision::RefusedByLimits(r) => r,
            Decision::Applied(r) => r,
        }
    }

    /// The renderer's decision for one foot, mirroring `character.rs:309-337`.
    fn of(leg: &LegRig, fy: f64, fz: f64, adjust: Option<f64>, half_voxel: f64) -> Self {
        let Some(adjust) = adjust else {
            return Decision::OutsideWindow;
        };
        if adjust.abs() <= 1e-3 {
            return Decision::Seated;
        }
        if adjust.abs() > half_voxel {
            return Decision::OutsideWindow;
        }
        let ik = solve_leg_ik(leg, [0.0, fy + adjust, fz]);
        if !(ik.upper_x.is_finite() && ik.lower_x.is_finite()) {
            return Decision::RefusedByLimits("nan!");
        }
        match ik.reach {
            Reach::BeyondFlexion { .. } => Decision::RefusedByLimits("flex"),
            Reach::JointBlocked { .. } => Decision::RefusedByLimits("blok"),
            Reach::Ok => Decision::Applied("ik:OK"),
            Reach::BeyondExtension => Decision::Applied("ik:ex"),
        }
    }
}

/// One foot at one phase. **The time series is the deliverable** (corrections
/// #76): every quantity the candidates could change state on is recorded per
/// sample, so *"what changed at the phase the knee jumped"* is a lookup and not
/// a second run.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sample {
    /// Live (unquantized) cycle phase in `[0, 1)`.
    pub phase: f64,
    /// This limb's own cycle position `u = (phase − limb.phase) mod 1`;
    /// `u < compass_window` is **stance**, above it is **swing**.
    pub u: f64,
    /// Is this foot swinging at this phase?
    pub swing: bool,
    /// Hip (upper) X rotation as rendered, degrees.
    pub hip_deg: f64,
    /// Knee (lower) X rotation as rendered, degrees. *The answer to "does the
    /// knee wobble".*
    pub knee_deg: f64,
    /// The gait's own knee angle before any IK, degrees.
    pub gait_knee_deg: f64,
    /// Sole world height as rendered, metres (ground top face = 0).
    pub foot_y_m: f64,
    /// Sole world height the gait alone puts the foot at, metres.
    pub gait_foot_y_m: f64,
    /// Sole forward offset from the hip, metres (−z is forward).
    pub foot_z_m: f64,
    /// The IK target's planar hip→foot distance, metres.
    pub d_planar_m: f64,
    /// `ground − gait_foot_y`, metres — the correction the renderer was asked
    /// for. `NaN` where the scan found no ground at all.
    pub adjust_m: f64,
    pub decision: Decision,
}

/// One (plan, leg, speed, target-fps, sampling density) trace over one stride.
#[derive(Clone, PartialEq, Debug)]
pub struct Trace {
    pub plan: String,
    pub leg: String,
    pub speed_m_s: f64,
    pub target_fps: f64,
    pub quantized: bool,
    pub samples: usize,
    /// `N` — poses the body's own stride cycle is divided into at this target.
    pub poses_per_cycle: f64,
    pub cycle_s: f64,
    pub froude: f64,
    pub l1_m: f64,
    pub l2_m: f64,
    pub reach_m: f64,
    pub d_min_m: f64,
    /// The limb's stance fraction — `u < compass_window` is stance.
    pub compass_window: f64,
    /// Highest the gait alone ever lifts this sole, metres.
    pub gait_lift_max_m: f64,
    /// Lowest the gait alone ever drives this sole, metres (negative = through
    /// the floor, which is what the foot IK exists to answer).
    pub gait_lift_min_m: f64,
    pub series: Vec<Sample>,
}

impl Trace {
    /// The largest knee-angle change between adjacent samples, and the phase it
    /// happens at (the phase of the *later* sample). Wraps the cycle.
    #[must_use]
    pub fn max_step(&self) -> (f64, f64) {
        self.worst(|a, b| (b.knee_deg - a.knee_deg).abs())
    }

    /// The same, on the rendered sole's world height (metres).
    #[must_use]
    pub fn max_foot_step(&self) -> (f64, f64) {
        self.worst(|a, b| (b.foot_y_m - a.foot_y_m).abs())
    }

    fn worst(&self, f: impl Fn(&Sample, &Sample) -> f64) -> (f64, f64) {
        let n = self.series.len();
        let mut out = (0.0_f64, 0.0_f64);
        for i in 0..n {
            let (a, b) = (&self.series[i], &self.series[(i + 1) % n]);
            let d = f(a, b);
            if d > out.0 {
                out = (d, b.phase);
            }
        }
        out
    }

    /// Every phase at which the renderer's [`Decision`] changes, with the pair
    /// it changed between and the knee jump across it.
    #[must_use]
    pub fn decision_changes(&self) -> Vec<(f64, String, f64)> {
        let n = self.series.len();
        let mut out = Vec::new();
        for i in 0..n {
            let (a, b) = (&self.series[i], &self.series[(i + 1) % n]);
            if a.decision != b.decision {
                out.push((
                    b.phase,
                    format!("{} → {}", a.decision.code(), b.decision.code()),
                    (b.knee_deg - a.knee_deg).abs(),
                ));
            }
        }
        out
    }

    /// How close the IK target ever came to the sector's inner bound —
    /// `min(d_planar) / d_min`. Below 1 would be a `BeyondFlexion` refusal, so
    /// this number alone either implicates or clears candidate 2.
    #[must_use]
    pub fn closest_to_d_min(&self) -> f64 {
        self.series
            .iter()
            .filter(|s| s.d_planar_m.is_finite())
            .map(|s| s.d_planar_m / self.d_min_m)
            .fold(f64::INFINITY, f64::min)
    }

    /// How close the IK target ever came to the annulus's **outer** edge —
    /// `max(d_planar) / (l1 + l2)`. Above 1 is `BeyondExtension`.
    #[must_use]
    pub fn furthest_of_reach(&self) -> f64 {
        self.series
            .iter()
            .filter(|s| s.d_planar_m.is_finite())
            .map(|s| s.d_planar_m / self.reach_m)
            .fold(0.0, f64::max)
    }
}

/// The top face of the highest solid voxel a foot column finds, on **level
/// ground whose top face is `y = 0`** — the closed form of
/// `character.rs::ground_top_m` for the one case this probe measures.
///
/// Solid is every voxel index `≤ −1`. The scan starts at `voxel_at(near +
/// half)` and walks down to `voxel_at(near − half) − 1`, returning the first
/// solid's **top face** — so a foot buried deep gets a surface *below* the real
/// one, and a foot high in the air gets `None`. Both behaviours are
/// production's and both matter here.
///
/// Indices are carried as `f64` rather than `i64` (production's type) purely to
/// avoid a cast; `floor` is exact and the magnitudes here are single digits, so
/// the two agree bit for bit.
#[must_use]
pub fn ground_top_level(voxel_m: f64, near_y_m: f64, half_voxel_m: f64) -> Option<f64> {
    let vy_hi = ((near_y_m + half_voxel_m) / voxel_m).floor();
    let vy_lo = ((near_y_m - half_voxel_m) / voxel_m).floor() - 1.0;
    let mut vy = vy_hi;
    while vy >= vy_lo {
        if vy <= -1.0 {
            return Some((vy + 1.0) * voxel_m);
        }
        vy -= 1.0;
    }
    None
}

/// A `Vec<JointAngle>` as a name→euler lookup.
fn angles(list: &[JointAngle]) -> HashMap<&str, [f64; 3]> {
    list.iter()
        .map(|j| (j.segment.as_str(), j.euler))
        .collect()
}

/// Trace one plan's bearing legs over one full stride on level ground.
///
/// `quantized: false` samples the gait continuously (`limb_pose` at the raw
/// phase) — the underlying geometry. `quantized: true` goes through
/// [`pose_for`] and [`AnimState::stepped_phase`], which is what the renderer
/// actually holds.
///
/// `None` for a plan that cannot bake a gait (a tree) — the identity fallback,
/// which has no swing to measure.
#[must_use]
pub fn trace_stride(
    plan: &BodyPlan,
    clips: &[&AnimClip],
    speed_m_s: f64,
    target_fps: f64,
    samples: usize,
    quantized: bool,
) -> Option<Vec<Trace>> {
    let gait = derived_gait(plan, dc_core::DEFAULT_GRAVITY_M_S2).ok()?;
    let rate = AnimRate::new(target_fps)?;
    let legs = leg_rigs(plan);
    let root_delta_m = derived_root_delta_m(plan).unwrap_or(0.0);
    let froude = gait.froude(speed_m_s);
    let cycle_s = 1.0 / gait.cadence_hz(froude);
    let half_voxel = VOXEL_M * 0.5;

    // Production's own state, driven by hand: `phase` is swept, `froude` is
    // held (a level walk at constant speed latches one value).
    let mut state = AnimState::facing(0.0);
    state.froude = froude;
    state.stepped_froude = froude;
    // `CycleGrid::poses` is cfg(test)-gated on the grid itself; derive the same
    // integer from the step it exposes.
    let poses_per_cycle = rate
        .cycle(cycle_s)
        .map_or(f64::NAN, |c| (cycle_s / c.step_s()).round());

    #[expect(
        clippy::cast_precision_loss,
        reason = "sample counts are thousands; the quotient is the probe's own phase grid"
    )]
    let phase_of = |i: usize| i as f64 / samples as f64;

    let mut out = Vec::new();
    for leg in &legs {
        // The limb this rig belongs to, matched on the joint names the gait
        // poses — never on a name convention (B0).
        let Some(limb) = gait
            .limbs
            .iter()
            .find(|l| l.bearing && l.neutral.iter().any(|j| j.segment == leg.upper))
        else {
            continue;
        };
        let mut series = Vec::with_capacity(samples);
        let mut gait_lift_max_m = f64::NEG_INFINITY;
        let mut gait_lift_min_m = f64::INFINITY;
        for i in 0..samples {
            let phase = phase_of(i);
            state.phase = phase;

            // --- the gait's own pose, before any IK -----------------------
            let (cu, cl, sampled_phase) = if quantized {
                let pose = pose_for(&state, Some(&gait), clips, rate);
                (
                    pose.joints.get(&leg.upper).map_or(0.0, |e| e[0]),
                    pose.joints.get(&leg.lower).map_or(0.0, |e| e[0]),
                    state.stepped_phase(Some(&gait), rate),
                )
            } else {
                let a = gait.limb_pose(limb, froude, phase);
                let m = angles(&a);
                (
                    m.get(leg.upper.as_str()).map_or(0.0, |e| e[0]),
                    m.get(leg.lower.as_str()).map_or(0.0, |e| e[0]),
                    phase,
                )
            };
            let (fy, fz) = fk_foot_local(leg.l1, leg.l2, cu, cl);
            // THE ONE VERTICAL COMPOSITION, read as production reads it.
            let root_offset = root_offset_m(
                Some(&gait),
                root_delta_m,
                froude,
                sampled_phase,
                Posture::Standing,
            );
            // The feet (the collider bottom) sit on the ground: y = 0.
            let hip_y = leg.hip_local[1] + root_offset;
            let gait_foot_y = hip_y + fy;
            gait_lift_max_m = gait_lift_max_m.max(gait_foot_y);
            gait_lift_min_m = gait_lift_min_m.min(gait_foot_y);

            // --- the renderer's foot placement ----------------------------
            let adjust = ground_top_level(VOXEL_M, gait_foot_y, half_voxel).map(|g| g - gait_foot_y);
            let decision = Decision::of(leg, fy, fz, adjust, half_voxel);
            let applied = matches!(decision, Decision::Applied(_));
            let ty = fy + if applied { adjust.unwrap_or(0.0) } else { 0.0 };
            let (hip_x, knee_x) = if applied {
                let ik = solve_leg_ik(leg, [0.0, fy + adjust.unwrap_or(0.0), fz]);
                (ik.upper_x, ik.lower_x)
            } else {
                (cu, cl)
            };
            let (ry, _) = fk_foot_local(leg.l1, leg.l2, hip_x, knee_x);
            let u = (sampled_phase - limb.phase).rem_euclid(1.0);
            series.push(Sample {
                phase,
                u,
                swing: u >= limb.compass_window,
                hip_deg: hip_x.to_degrees(),
                knee_deg: knee_x.to_degrees(),
                gait_knee_deg: cl.to_degrees(),
                foot_y_m: hip_y + ry,
                gait_foot_y_m: gait_foot_y,
                foot_z_m: fz,
                d_planar_m: (ty * ty + fz * fz).sqrt(),
                adjust_m: adjust.unwrap_or(f64::NAN),
                decision,
            });
        }
        out.push(Trace {
            plan: plan.name.clone(),
            leg: leg.upper.clone(),
            speed_m_s,
            target_fps,
            quantized,
            samples,
            poses_per_cycle,
            cycle_s,
            froude,
            l1_m: leg.l1,
            l2_m: leg.l2,
            reach_m: leg.l1 + leg.l2,
            d_min_m: leg.limits.d_min,
            compass_window: limb.compass_window,
            gait_lift_max_m,
            gait_lift_min_m,
            series,
        });
    }
    Some(out)
}

#[cfg(test)]
mod gate {
    use super::*;
    use dc_api::bodies::{biped_clips, biped_plan, longleg_plan, stout_plan};

    /// The walk's own speed: `0.3 × walk_speed_m_s`.
    const V: f64 = 0.3 * WALK_SPEED_M_S;

    fn idle() -> AnimClip {
        biped_clips()
            .into_iter()
            .find(|c| c.name == "dc:anim/biped_idle")
            .expect("the default pack ships an idle clip")
    }

    /// The per-trace block: the leg's constants, the two jump statistics, every
    /// decision change, and a window of the raw series around the worst jump.
    fn report(t: &Trace) {
        println!(
            "\n=== {} / {} — {}\n    v={:.3} m/s  fps={:.0}  samples={}  N={:.0}  cycle={:.4} s  Fr={:.5}",
            t.plan,
            t.leg,
            if t.quantized {
                "RENDERED (per-cycle quantized)"
            } else {
                "continuous geometry"
            },
            t.speed_m_s,
            t.target_fps,
            t.samples,
            t.poses_per_cycle,
            t.cycle_s,
            t.froude
        );
        println!(
            "    l1={:.5} l2={:.5} reach={:.5} d_min={:.5} stance_window={:.4}",
            t.l1_m, t.l2_m, t.reach_m, t.d_min_m, t.compass_window
        );
        println!(
            "    gait sole over the stride: [{:+.5}, {:+.5}] m   (lift above ground / drive below it)",
            t.gait_lift_min_m, t.gait_lift_max_m
        );
        let (dk, pk) = t.max_step();
        let (df, pf) = t.max_foot_step();
        println!("    MAX adjacent knee step {dk:.4}° at phase {pk:.4}");
        println!("    MAX adjacent sole step {df:.5} m at phase {pf:.4}");
        println!(
            "    d_planar / d_min  min = {:.3}   d_planar / reach  max = {:.5}",
            t.closest_to_d_min(),
            t.furthest_of_reach()
        );
        let changes = t.decision_changes();
        if changes.is_empty() {
            println!("    decision: CONSTANT over the whole stride");
        }
        for (phase, change, jump) in &changes {
            println!("    decision change at phase {phase:.4}: {change}  (knee jump {jump:.4}°)");
        }
        // The raw series around the worst knee step — every recorded column, so
        // the cause is a lookup rather than a second run.
        let n = t.series.len();
        let centre = t
            .series
            .iter()
            .position(|s| s.phase >= pk - 1e-12)
            .unwrap_or(0);
        println!(
            "      {:>7} {:>7} {:>5} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}  {}",
            "phase",
            "u",
            "sw",
            "hip°",
            "knee°",
            "gaitknee",
            "sole_m",
            "gaitsole",
            "foot_z",
            "d_plan",
            "adjust",
            "decision"
        );
        for k in 0..9 {
            let i = (centre + n + k - 4) % n;
            let s = &t.series[i];
            println!(
                "      {:>7.4} {:>7.4} {:>5} {:>9.3} {:>9.3} {:>9.3} {:>+9.5} {:>+9.5} {:>+9.5} {:>9.5} {:>+9.5}  {}",
                s.phase,
                s.u,
                if s.swing { "SW" } else { "st" },
                s.hip_deg,
                s.knee_deg,
                s.gait_knee_deg,
                s.foot_y_m,
                s.gait_foot_y_m,
                s.foot_z_m,
                s.d_planar_m,
                s.adjust_m,
                s.decision.code()
            );
        }
    }

    /// **THE REPORT.** Prints the whole comparative measurement under
    /// `--nocapture`; asserts nothing a magnitude could move.
    #[test]
    fn the_longleg_swing_is_measured() {
        let clip = idle();
        let clips: Vec<&AnimClip> = vec![&clip];
        println!(
            "\n################ THE WALK'S CONDITIONS: v = {V:.3} m/s, level stone, voxel {VOXEL_M} m ################"
        );
        for plan in [longleg_plan(), stout_plan(), biped_plan()] {
            for fps in [60.0, 12.0] {
                for quantized in [false, true] {
                    let Some(traces) = trace_stride(&plan, &clips, V, fps, 2000, quantized) else {
                        println!("{}: no gait", plan.name);
                        continue;
                    };
                    for t in &traces {
                        report(t);
                    }
                }
            }
        }
        println!("\n################ LONGLEG SPEED SWEEP (continuous, 60 fps) ################");
        let plan = longleg_plan();
        for frac in [0.15_f64, 0.3, 0.5] {
            let Some(traces) =
                trace_stride(&plan, &clips, frac * WALK_SPEED_M_S, 60.0, 2000, false)
            else {
                continue;
            };
            for t in &traces {
                println!("\n>>> speed fraction {frac}");
                report(t);
            }
        }
    }

    /// **The invariant, and it is scale-free.** A C0-continuous signal's largest
    /// adjacent-sample change is `|f′| · Δphase`, so refining the phase grid by
    /// 4 divides it by ≈ 4; a genuine jump is invariant under refinement. The
    /// ratio therefore tells the two apart **with no tolerance on the angle
    /// itself** — which is what keeps this from being a snapshot test that fails
    /// because somebody improved the gait.
    ///
    /// Asserted on the CONTINUOUS trace only: the rendered one is quantized by
    /// design (journal/0155) and its staircase is not a defect.
    ///
    /// **Scale-free** because it is a statement about a per-phase function of
    /// one body's own geometry — no world is built at all, and the property
    /// holds at any sample count, any speed and any plan.
    #[test]
    fn the_continuous_swing_geometry_refines() {
        let clip = idle();
        let clips: Vec<&AnimClip> = vec![&clip];
        for plan in [longleg_plan(), stout_plan(), biped_plan()] {
            let coarse = trace_stride(&plan, &clips, V, 60.0, 1_000, false).expect("a gait");
            let fine = trace_stride(&plan, &clips, V, 60.0, 4_000, false).expect("a gait");
            for (c, f) in coarse.iter().zip(&fine) {
                let (dc, pc) = c.max_step();
                let (df, pf) = f.max_step();
                assert!(
                    df * 2.5 < dc,
                    "{} / {}: the continuous knee track did NOT refine — {dc:.4}° at 1 000 \
                     samples (phase {pc:.4}), {df:.4}° at 4 000 (phase {pf:.4}). A step that \
                     survives refinement is a DISCONTINUITY in the swing geometry, which is \
                     exactly what this probe exists to find.",
                    c.plan,
                    c.leg
                );
            }
        }
    }

    /// The renderer's foot-placement decision must be a **function of phase**:
    /// the same phase decides the same way however finely the cycle is walked.
    /// A failure means the decision depends on something this probe is not
    /// recording, and every number above would be suspect.
    #[test]
    fn the_placement_decision_is_a_function_of_phase() {
        let clip = idle();
        let clips: Vec<&AnimClip> = vec![&clip];
        for plan in [longleg_plan(), stout_plan(), biped_plan()] {
            let a = trace_stride(&plan, &clips, V, 60.0, 500, false).expect("a gait");
            let b = trace_stride(&plan, &clips, V, 60.0, 1_000, false).expect("a gait");
            for (ta, tb) in a.iter().zip(&b) {
                for (i, sa) in ta.series.iter().enumerate() {
                    let sb = &tb.series[i * 2];
                    assert_eq!(
                        sa.decision, sb.decision,
                        "{} / {}: phase {:.5} decided {:?} at 500 samples and {:?} at 1 000",
                        ta.plan, ta.leg, sa.phase, sa.decision, sb.decision
                    );
                }
            }
        }
    }
}
