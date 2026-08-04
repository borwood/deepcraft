//! The client-side animation player (docs/design/bodies.md steps 1–2).
//!
//! This is the render half of the determinism firewall: it turns a **derived
//! gait** (and, for non-locomotion, a registry [`AnimClip`]) into a posed
//! skeleton, entirely on the client, driven by the render clock. **Nothing here
//! is ever read back into simulation.** The sim sees the swept-AABB mover
//! (`dc_api::character`), parametric posture, and — since 2026-08-02 — the
//! **target trunk facing**; the only sim state this module *reads* is a
//! character's speed and that target, both legal one-way reads.
//!
//! # Locomotion is DERIVED, not played (2026-08-02, gait member #1 slice two)
//!
//! `sample_clip(walk, t)` is gone. [`pose_for`] samples
//! [`dc_api::bodies::GaitVector`] — cadence, stride, duty, hip excursion and
//! root height all closed-form functions of the **Froude number** `Fr =
//! v²/(gL)` of the body's own leg length. Three consequences worth stating
//! because each retires something that used to be here:
//!
//! - **There is no gait SWITCH.** `WALK_SPEED_THRESHOLD_M_S = 0.35` and the
//!   two-state `enum Loco { Idle, Walk }` are gone (user call #1, `stubs.md`
//!   #42 — *"when we have controller support an analog stick can actually grade
//!   intent up the ladder"*). Fr is continuous and **idle is its degenerate
//!   limit**: at v → 0 the stride → 0, so θmax → 0, so every keyframe collapses
//!   onto the derived resting pose and the phase clock stops. Nothing had to be
//!   special-cased to make a stopped body stand still. **Never re-introduce a
//!   discrete gait switch** — that is the ruling's explicit prohibition.
//! - **The phase clock advances with DISTANCE, not wall time.** `phase +=
//!   cadence(Fr)·dt`, and `cadence = v/λ`, so `dphase = ds/λ` exactly: one full
//!   limb cycle per stride travelled. The old clip incremented `clock_s += dt`
//!   with no speed term at all, which is why the feet supplied 1.84 m/s of a
//!   4.5 m/s walk and **2.66 m/s was skate**.
//! - **There is no bob TERM.** Root height is a function of phase
//!   ([`root_offset_m`]) read by both the render root and the IK hip — one
//!   composition, so corrections #80's two-expression drift is structurally
//!   impossible rather than merely fixed. `Keyframe.root_bob_m` left the schema
//!   (user call #2) and `Pose` has no vertical field.
//!
//! Non-locomotion clips (idle's breath, the jump one-shot) still ride, as an
//! **additive layer over the gait's base on non-bearing segments** — design § 6
//! rule 4, which is how a sword swing rides a walk. A clip's contribution to a
//! **bearing** chain is refused: authoring a leg over a locomotion gait is a
//! gait-type choice, not a clip.
//!
//! # The one stepping device left
//!
//! **Stepped, per CYCLE.** A cycle gets `N = round(duration × target_fps)` held
//! poses, `N ≥ 2`, and the target is a **client setting** — see
//! [`crate::anim_rate`], which carries the ruling. `ANIM_FPS = 12.0` was retired
//! 2026-08-04 (user): it was a world-global absolute constant aliasing a cadence
//! that is now derived per body, and the stout — shortest legs, fastest cadence,
//! 4.29 samples per cycle — hitched visibly for it.
//!
//! Three consequences live in this file:
//!
//! - the **gait** is sampled at a phase held on its own cycle's grid, so the
//!   same `N` phases come round every stride whatever the body's leg turnover;
//! - a **clip** is sampled at a phase held on its own duration's grid — wrapped
//!   first, then quantized, which is what makes cycle *k* sample cycle 0's
//!   phases bit for bit;
//! - a **bone owned by several anims** is sampled at the blended time step
//!   ([`crate::anim_rate::blended_step_s`]), weighted by ownership. A bone with
//!   one owner keeps that owner's whole `N`.
//!
//! **There used to be two more.** Sampled Euler angles snapped to a `TAU/32`
//! (11.25°) rotation grid — REMOVED 2026-08-01 by user ruling (bodies.md
//! § stepped animation carries the banner): assistant-originated, guarding an
//! IK instability that 1,056 measured samples show does not exist
//! (journal/0131), and it cost sub-decimetre foot placement outright. And the
//! **stepped crossfade** between the two `Loco` states, which went with the
//! states themselves — a continuous ladder has nothing to cross-fade between.
//!
//! The module is pure (no bevy, no glam) so it is trivially testable: a fixed
//! `(dt, speed)` history and a fixed plan produce an identical pose stream
//! every run (see the determinism tests). The bevy/glam glue that spawns the
//! segment hierarchy and writes joint transforms lives in `character.rs`.

use std::collections::HashMap;

use dc_api::Posture;
use dc_api::bodies::{AnimClip, Axis, BodyPlan, GaitVector};

use crate::anim_rate::{AnimRate, CycleGrid, PhaseGrid, StepBlend};

// `ANIM_FPS = 12.0` LEFT THIS FILE 2026-08-04 (user ruling; `bodies.md`
// § Stepped animation, the `▶▶ DECIDED 2026-08-04` banner). It was a
// world-global absolute constant quantizing per SECOND against a cadence that
// is now derived per BODY — A-1's fourth retirement in this arc, and the one
// the stout's hitch made visible. Its heir is [`crate::anim_rate::AnimRate`], a
// **client setting** whose default (12.0) reproduces the biped's look; the rate
// a given cycle actually renders at is derived per cycle, not declared.
// `NECK_YAW_CLAMP_RAD` (75°) and `NECK_PITCH_CLAMP_RAD` (45°) LEFT THIS FILE
// 2026-08-03 (B7 § 5.4). They were **anatomical joint limits, hard-coded,
// world-global, and blind to the plan** — the stout's 0.08 m neck got the
// biped's numbers — which is A-1 in the family of absolute constants this arc
// has been retiring. They are now DECLARED on the `look` joint of each plan
// (`dc_api::bodies::default_pack`'s `with_cervical_range`), read here through
// [`Cervical::of`]. The migration is byte-identical: the same two numbers,
// a different home, and now a body *can* say its own.
/// Trunk turn window (seconds): how quickly the trunk yaw chases the travel
/// direction. Short, and the pose is sampled on the client's stop-motion grid,
/// so turns still read stepped in time even though the yaw itself is now exact.
pub const TRUNK_TURN_WINDOW_S: f64 = 0.22;
/// How far the root sinks (meters) when the body is crouching — the cosmetic
/// half of the parametric-crouch firewall split (the sim shrinks the collider;
/// this lowers the spine and the feet-IK bends the knees to keep contact).
pub const CROUCH_ROOT_DROP_M: f64 = 0.45;

/// Wrap an angle into `(-π, π]` deterministically.
fn wrap_pi(a: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    let x = (a + PI).rem_euclid(TAU) - PI;
    // rem_euclid maps exactly to [-π, π); nudge the -π endpoint to +π so the
    // range is symmetric and 0 stays 0.
    if x <= -PI { x + TAU } else { x }
}

// `yaw_from_velocity` lived here until 2026-08-02 and is now
// `dc_api::character::yaw_from_travel` — ONE authority for the convention,
// because the SIM derives the trunk's target facing (user call #5) and the
// client only chases it. Two copies of a facing convention in two crates is
// the two-derivations-of-one-quantity failure `bodies.md` § THE SIM OWNS THE
// TARGET forbids by name, and it is where the walk-8 strafe lived.

/// A sampled skeletal pose: per-joint XYZ Euler rotation (radians). Joints
/// absent from the map are identity.
///
/// **There is no vertical field, deliberately.** A pose is joints; root height
/// is [`root_offset_m`]'s, read once per body per frame by both the render root
/// and the IK hip. The retired `root_bob_m` here was the *second* expression of
/// one composition and the two drifted by exactly its own value (corrections
/// #80) — deleting the term is what makes the drift impossible, where a tidier
/// version of the falsified fix (add the bob to the hip too) would only have
/// asked two expressions to agree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    pub joints: HashMap<String, [f64; 3]>,
}

/// Where in its own cycle a clip is at animation time `t`: a fraction in
/// `[0, 1]`, **wrapped** for a looping clip and **clamped** for a one-shot.
///
/// **A one-off is just a cycle that does not repeat** (user, 2026-08-04): the
/// only difference between the two is what happens past the end, and there is no
/// cycle-vs-one-off branch anywhere else in this file.
///
/// **The wrap happens HERE, before any quantization** — the retired
/// `quantize_time` did it the other way round, flooring onto a grid anchored at
/// absolute `t = 0` and wrapping afterwards, so a looping clip whose duration was
/// not a whole number of frames sampled a different set of sub-frame phases every
/// cycle (ROADMAP § Observed; latent because every shipped looping clip happened
/// to be frame-aligned at 12 fps). Quantizing a **phase** cannot express that
/// defect, so the ordering fix is not a separate change here — it is what
/// per-cycle quantization *is*.
///
/// A clip with no usable duration sits at phase 0 (its first keyframe); the
/// define door refuses such a clip (`dc_api::bodies::validate_clip`), so this is
/// reachable only from a hand-built fixture.
fn clip_phase(clip: &AnimClip, t: f64) -> f64 {
    if !(clip.duration_s.is_finite() && clip.duration_s > 0.0) || !t.is_finite() {
        return 0.0;
    }
    let u = t / clip.duration_s;
    if clip.loops {
        // rem_euclid keeps a negative or huge clock in range deterministically.
        let p = u.rem_euclid(1.0);
        if p >= 1.0 { 0.0 } else { p }
    } else {
        u.clamp(0.0, 1.0)
    }
}

fn lerp3(a: [f64; 3], b: [f64; 3], f: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
}

/// Sample a NON-LOCOMOTION clip at animation time `t`, on the clip's **own**
/// per-cycle grid: `N = round(duration × target_fps)` held poses per cycle,
/// `N ≥ 2`. **The time step is the whole stop-motion look** — the angles it
/// yields are exact (the rotation quantizer was removed 2026-08-01).
///
/// Locomotion does not come through here any more: it is derived
/// ([`pose_for`]). What is left is the idle breath, the jump one-shot, and
/// whatever a pack authors that is genuinely *not* gait — transitions, emotes,
/// upper-body action (design § 6).
///
/// **`#[cfg(test)]`, and that is a fact about the composition rather than about
/// this function.** This is the whole-clip entry point: every bone of the clip
/// on the clip's own rate. Production has no such caller — [`pose_for`] samples
/// **per bone group**, because a bone the gait co-owns rides a blended step —
/// so the live consumers are `retarget_report` and the acceptance tests. Left
/// visible (rather than deleted or inlined) because it is the honest statement
/// of one clip's own grid, and it is what the tests assert that grid against.
#[cfg(test)]
pub fn sample_clip(clip: &AnimClip, t: f64, rate: AnimRate) -> Pose {
    let phase = clip_phase(clip, t);
    let stepped = rate
        .cycle(clip.duration_s)
        .map_or(phase, |c| c.phases().quantize(phase));
    sample_clip_at_phase(clip, stepped)
}

/// Interpolate a clip at an **already-quantized** cycle fraction — the shared
/// half of [`sample_clip`], reached directly by [`pose_for`] when a bone's grid
/// is not the clip's own.
fn sample_clip_at_phase(clip: &AnimClip, phase: f64) -> Pose {
    let mut pose = Pose::default();
    if clip.keyframes.is_empty() {
        return pose;
    }
    let st = if clip.duration_s.is_finite() && clip.duration_s > 0.0 {
        phase * clip.duration_s
    } else {
        0.0
    };
    // Find the bracketing keyframes [a, b] with a.t <= st <= b.t.
    let kfs = &clip.keyframes;
    let (a, b, f) = if st <= kfs[0].t {
        (&kfs[0], &kfs[0], 0.0)
    } else if st >= kfs[kfs.len() - 1].t {
        let last = &kfs[kfs.len() - 1];
        (last, last, 0.0)
    } else {
        let mut i = 0;
        while i + 1 < kfs.len() && kfs[i + 1].t <= st {
            i += 1;
        }
        let (a, b) = (&kfs[i], &kfs[i + 1]);
        let span = b.t - a.t;
        let f = if span > 1e-12 { (st - a.t) / span } else { 0.0 };
        (a, b, f)
    };
    // Union of joints named in either bracketing keyframe.
    let mut names: Vec<&str> = Vec::new();
    for kf in [a, b] {
        for r in &kf.rotations {
            if !names.contains(&r.segment.as_str()) {
                names.push(&r.segment);
            }
        }
    }
    let rot_in = |kf: &dc_api::bodies::Keyframe, name: &str| -> [f64; 3] {
        kf.rotations
            .iter()
            .find(|r| r.segment == name)
            .map(|r| r.euler)
            .unwrap_or([0.0, 0.0, 0.0])
    };
    for name in names {
        pose.joints
            .insert(name.to_string(), lerp3(rot_in(a, name), rot_in(b, name), f));
    }
    pose
}

/// Per-body animation state: the **gait phase** and the trunk's approach toward
/// the sim's target facing.
///
/// The state a locomotion crossfade needed is gone with the crossfade: there
/// are no discrete states to blend between, so there is no `blend_w`, no
/// `blend_from`, and no `Loco`. What survives is a clock (for non-locomotion
/// clips and the speed latch), a phase, and a facing.
///
/// **The stepped PHASE is no longer state** (2026-08-04). It used to be latched
/// here beside the Froude number, because both were held on one world-global
/// time grid. Under per-cycle quantization the phase's grid is the *cycle's* and
/// the bone's, so `stepped_phase` is a **derived accessor**
/// ([`AnimState::stepped_phase`]) over the live phase — one authority (S-3),
/// and nothing to keep in sync.
#[derive(Clone, Debug)]
pub struct AnimState {
    /// Wall clock since spawn, seconds — drives NON-locomotion clips and the
    /// stop-motion latch. Locomotion does not read it: gait phase advances with
    /// distance travelled, not with time.
    pub clock_s: f64,
    /// Gait phase in `[0, 1)`: where in the limb cycle this body is.
    /// `phase += cadence(Fr)·dt`, and cadence is `v/λ`, so one cycle passes per
    /// stride **travelled**. A stopped body's phase does not advance at all,
    /// because its cadence is zero — which is also why idle needs no state.
    ///
    /// **This is the UNQUANTIZED target** and it is what a sim-visible consumer
    /// would have to read (`bodies.md` § THE SIM OWNS THE TARGET). It is a pure
    /// function of the `(dt, speed)` history and **does not depend on the
    /// client's target fps at all** — pinned by
    /// `the_target_fps_never_reaches_a_sim_visible_quantity`.
    pub phase: f64,
    /// The dimensionless speed the gait is being evaluated at. Unquantized, and
    /// likewise independent of the target fps.
    pub froude: f64,
    /// The Froude number actually sampled, held on the client's base grid
    /// `1/target_fps`.
    ///
    /// **Why speed is held on a TIME grid while phase is held on a CYCLE grid,
    /// and it is not an inconsistency.** Phase is a cycle-domain quantity — "N
    /// poses per stride" is exactly a statement about it, and it can be
    /// quantized as a pure function of the live value. Speed is a time-domain
    /// signal with no cycle of its own: holding it is a sample-and-hold, which
    /// needs memory, and the cycle grid cannot serve as its clock (a body at
    /// rest has a frozen phase, so a phase-triggered latch would never notice it
    /// starting to move). It is held at `1/target_fps` because every derived
    /// `Δt` in the world is within a rounding of that — which is the same
    /// observation that makes per-bone rates cluster.
    pub stepped_froude: f64,
    /// The frame index the latch last fired on. `NaN` until the first advance,
    /// so the first frame always latches (`NaN != x` for every `x`).
    latched_frame: f64,
    /// The **rendered** trunk yaw (radians, bevy convention) — the *approach*
    /// half of the facing split (user call #5, 2026-08-02; `bodies.md` § THE SIM
    /// OWNS THE TARGET). It chases `CharacterState.facing_yaw` over
    /// [`TRUNK_TURN_WINDOW_S`]. The target is the sim's and is replay-safe; this
    /// smoothing is cosmetic and free to tune.
    pub trunk_yaw: f64,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            clock_s: 0.0,
            phase: 0.0,
            froude: 0.0,
            stepped_froude: 0.0,
            latched_frame: f64::NAN,
            trunk_yaw: 0.0,
        }
    }
}

impl AnimState {
    /// A fresh state already facing `trunk_yaw` — used at spawn so the trunk
    /// does not swing to the sim's target from an arbitrary zero on the first
    /// steps. A constructor rather than a struct literal because the latch
    /// index is private: there is exactly one way to build this, and it starts
    /// un-latched.
    #[must_use]
    pub fn facing(trunk_yaw: f64) -> Self {
        Self {
            trunk_yaw,
            ..Self::default()
        }
    }

    /// Advance one render frame: tick the clock, grade the gait continuously
    /// over Froude, advance the phase by the **derived cadence**, and hold the
    /// sampled speed on the client's base grid.
    ///
    /// **No branch on speed.** `Fr = v²/(gL)` and `cadence = k_f·Fr^0.2`, so a
    /// standing body advances its phase by exactly zero and samples its resting
    /// pose — idle is the ladder's degenerate limit, not a state. A plan whose
    /// gait declined (a tree, an out-of-domain geometry) passes `None` and gets
    /// the same still resting pose, which is the identity fallback.
    ///
    /// **`rate` reaches only the hold.** `clock_s`, `phase` and `froude` are
    /// computed before it is consulted and are bit-identical at any target — the
    /// firewall property, asserted by
    /// `the_target_fps_never_reaches_a_sim_visible_quantity`.
    pub fn advance(&mut self, dt: f64, gait: Option<&GaitVector>, speed_m_s: f64, rate: AnimRate) {
        let dt = dt.max(0.0);
        self.clock_s += dt;
        let froude = gait.map_or(0.0, |g| g.froude(speed_m_s.max(0.0)));
        let cadence_hz = gait.map_or(0.0, |g| g.cadence_hz(froude));
        self.froude = froude;
        // `dphase = f·dt = (v/λ)·dt = ds/λ`: the phase clock is a DISTANCE
        // clock wearing a rate. The retired clip incremented `clock_s += dt`
        // with no speed term, which is precisely where the skate came from.
        if cadence_hz.is_finite() {
            self.phase = (self.phase + cadence_hz * dt).rem_euclid(1.0);
        }
        let frame = (self.clock_s / rate.step_s()).floor();
        if frame != self.latched_frame {
            self.latched_frame = frame;
            self.stepped_froude = self.froude;
        }
    }

    /// This body's **gait cycle** at the speed it is being rendered at — one
    /// stride, in seconds — and the `N` poses the client's target divides it
    /// into.
    ///
    /// `None` when there is no cycle to divide: no gait, or a cadence of zero (a
    /// standing body, whose stride takes forever). See
    /// [`AnimRate::cycle`] for why the long-cycle limit is a dissolve rather than
    /// a cliff.
    #[must_use]
    pub fn gait_cycle(&self, gait: Option<&GaitVector>, rate: AnimRate) -> Option<CycleGrid> {
        let cadence_hz = gait?.cadence_hz(self.stepped_froude);
        rate.cycle(1.0 / cadence_hz)
    }

    /// The gait phase actually sampled: the live phase held at the start of its
    /// bucket on **this body's own stride grid**.
    ///
    /// This is the number the whole slice is about. A biped at 4.5 m/s strides
    /// in 0.581 s and gets `N = 7` — the same seven phases every stride, instead
    /// of 6.977 phases that drift through it.
    ///
    /// Bones shared with a clip do not use this — they ride a blended step
    /// ([`pose_for`]). The bearing chains do, and so does the root's vertical,
    /// which is why `character.rs` reads it directly.
    #[must_use]
    pub fn stepped_phase(&self, gait: Option<&GaitVector>, rate: AnimRate) -> f64 {
        self.gait_cycle(gait, rate)
            .map_or(self.phase, |c| c.phases().quantize(self.phase))
    }

    /// Steer the trunk toward the **sim's target facing** over
    /// [`TRUNK_TURN_WINDOW_S`] — the approach half of the split. The target is
    /// held by the sim when the body is stationary, so this needs no speed test
    /// of its own: a stopped body simply chases a target that is not moving.
    /// *That is the threshold this used to carry, and its removal is the whole
    /// point of the split.*
    pub fn steer(&mut self, dt: f64, target_yaw: f64) {
        let dt = dt.max(0.0);
        if !target_yaw.is_finite() {
            return;
        }
        let delta = wrap_pi(target_yaw - self.trunk_yaw);
        let f = if TRUNK_TURN_WINDOW_S > 0.0 {
            (dt / TRUNK_TURN_WINDOW_S).min(1.0)
        } else {
            1.0
        };
        self.trunk_yaw = wrap_pi(self.trunk_yaw + delta * f);
    }
}

/// The resolved facing of a body: how far its trunk turns and how far its
/// head/neck turns, after splitting the look off the travel-facing trunk.
/// All angles are exact radians; only the per-cycle time step remains.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Orientation {
    /// Trunk (root) yaw about Y, radians.
    pub trunk_yaw: f64,
    /// Neck yaw about Y relative to the trunk, radians (clamped to the cervical
    /// range).
    pub neck_yaw: f64,
    /// Neck pitch about X relative to the trunk, radians (negative looks down).
    pub neck_pitch: f64,
}

/// A plan's **declared** cervical range, resolved once per plan (B7 § 5.4) —
/// the replacement for the two world-global constants that used to live at the
/// top of this file.
///
/// `None` on an end means the plan's `look` joint leaves that end unbounded (or
/// declares no such DOF at all), and the look is then **not clamped on that
/// axis** — the identity behaviour for a body nobody has given a neck range.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Cervical {
    /// Largest |yaw| the neck may take relative to the trunk, radians.
    pub yaw_max: Option<f64>,
    /// Pitch range, radians; negative looks down (character.rs convention).
    pub pitch_min: Option<f64>,
    pub pitch_max: Option<f64>,
}

impl Cervical {
    /// Read a plan's cervical range off its unique `look` joint's declared
    /// limits. Unique-or-loud resolves the joint (B0); a plan with no `look`
    /// role, or an ambiguous one, gets an unclamped neck — the same
    /// feature-off a missing name has always meant.
    pub fn of(plan: &BodyPlan) -> Self {
        use dc_api::bodies::{Axis, derive_joint_limits, unique_role_segment};
        let Ok(Some(seg)) = unique_role_segment(plan, "look") else {
            return Cervical::default();
        };
        let limits = derive_joint_limits(plan);
        let Some(joint) = limits.joint(&seg.name) else {
            return Cervical::default();
        };
        // The YAW bound is a magnitude: the split is symmetric by construction
        // (the trunk absorbs the excess on whichever side), so a plan that
        // declared an asymmetric yaw would need the split itself to change.
        // Take the tighter end rather than inventing a side.
        let yaw = joint
            .dof(Axis::Y)
            .and_then(|d| match (d.min.value(), d.max.value()) {
                (Some(lo), Some(hi)) => Some(lo.abs().min(hi.abs())),
                (Some(lo), None) => Some(lo.abs()),
                (None, Some(hi)) => Some(hi.abs()),
                (None, None) => None,
            });
        let pitch = joint.dof(Axis::X);
        Cervical {
            yaw_max: yaw,
            pitch_min: pitch.and_then(|d| d.min.value()),
            pitch_max: pitch.and_then(|d| d.max.value()),
        }
    }
}

/// Split a look direction off a travel-facing trunk (bodies.md step 3, the
/// walk-8 fix). The head follows the look within the **declared** cervical range
/// (B7 § 5.4); a look beyond it drags the trunk around so the neck only ever
/// bends its maximum. `base_trunk_yaw` is the trunk's *approached* facing
/// ([`AnimState::trunk_yaw`], chasing the sim's target).
///
/// The YAW clamp is a **redistribution**, not a truncation — the trunk absorbs
/// the excess, so the gaze target survives (`bodies.md` § THE SIM OWNS THE
/// TARGET). The PITCH clamp genuinely truncates ("no trunk pitch in v0"): a body
/// can look somewhere its renderer cannot show it looking. That predates B7 and
/// is exactly what B7 exists to absorb — it is now a declared limit rather than
/// an anonymous constant, and the overflow is the honest-float story one level
/// up.
pub fn resolve_orientation(
    base_trunk_yaw: f64,
    look_yaw: f64,
    look_pitch: f64,
    cervical: Cervical,
) -> Orientation {
    let delta = wrap_pi(look_yaw - base_trunk_yaw);
    let (trunk_yaw, neck_yaw) = match cervical.yaw_max {
        Some(m) if delta.abs() > m => {
            let clamped = m * delta.signum();
            // Trunk absorbs the excess so the neck bends exactly its limit.
            (wrap_pi(look_yaw - clamped), clamped)
        }
        _ => (base_trunk_yaw, delta),
    };
    let neck_pitch = look_pitch
        .max(cervical.pitch_min.unwrap_or(f64::NEG_INFINITY))
        .min(cervical.pitch_max.unwrap_or(f64::INFINITY));
    Orientation {
        trunk_yaw,
        neck_yaw,
        neck_pitch,
    }
}

/// A leg's two-bone rig, **derived from a plan** (never hard-coded): the hip
/// joint position in body-local meters and the two bone lengths, for the
/// foot-placement IK. This is the plan-generic half of the retargeting glue —
/// every proportion the solver knows about comes from here, so a plan with half
/// the biped's leg length simply produces half the bone lengths.
///
/// Pure data + pure arithmetic, so it lives in this module rather than the bevy
/// glue: [`leg_rigs`] is exactly what the second-plan measurement needs.
#[derive(Clone, PartialEq, Debug)]
pub struct LegRig {
    pub upper: String,
    pub lower: String,
    /// Hip joint offset from the body root (the feet), meters.
    pub hip_local: [f64; 3],
    /// Upper bone length (hip→knee), meters.
    pub l1: f64,
    /// Lower bone length (knee→sole), meters.
    pub l2: f64,
    /// **The joint limits, resolved from the plan** (B7). Carried on the rig
    /// rather than passed beside it so [`solve_leg_ik`] takes them as a
    /// REQUIRED argument with **no unlimited overload** — there is then no way
    /// to produce an IK pose that ignores the joint (audit § 2.4's E4 test,
    /// answered in the one place inexpressibility is genuinely available: the
    /// solver is code we own, not data flowing through a door).
    pub limits: LegLimits,
}

/// A leg's two joints as the solver needs them: the sagittal (X) range of each,
/// and the inner radius of the reachable **sector** that the knee's range cuts
/// out of the old annulus.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LegLimits {
    /// Is X a declared DOF of the knee? `false` ⇒ the solver refuses rather
    /// than producing a silently wrong pose for a joint that does not hinge
    /// sagittally.
    pub knee_sagittal: bool,
    pub knee_min: Option<f64>,
    pub knee_max: Option<f64>,
    pub hip_sagittal: bool,
    pub hip_min: Option<f64>,
    pub hip_max: Option<f64>,
    /// Distance from hip to foot at **full flexion**,
    /// `sqrt(l1² + l2² + 2·l1·l2·cos f_max)` — the honest inner bound of the
    /// reachable set, replacing the *numerical* `|l1 − l2|`. Where the knee's
    /// range is unbounded this IS `|l1 − l2|`, so an undeclared, underivable
    /// knee reproduces today's annulus exactly (S-5).
    pub d_min: f64,
}

/// Resolve one leg's limits out of a plan's derived+declared joint limits.
fn leg_limits(
    limits: &dc_api::bodies::JointLimits,
    upper: &str,
    lower: &str,
    l1: f64,
    l2: f64,
) -> LegLimits {
    use dc_api::bodies::Axis;
    let dof = |seg: &str| limits.joint(seg).and_then(|j| j.dof(Axis::X)).cloned();
    let (knee, hip) = (dof(lower), dof(upper));
    let f_max = knee
        .as_ref()
        .and_then(|d| d.max_magnitude())
        .unwrap_or(std::f64::consts::PI)
        .min(std::f64::consts::PI);
    let d_min = (l1 * l1 + l2 * l2 + 2.0 * l1 * l2 * f_max.cos())
        .max(0.0)
        .sqrt();
    LegLimits {
        knee_sagittal: knee.is_some(),
        knee_min: knee.as_ref().and_then(|d| d.min.value()),
        knee_max: knee.as_ref().and_then(|d| d.max.value()),
        hip_sagittal: hip.is_some(),
        hip_min: hip.as_ref().and_then(|d| d.min.value()),
        hip_max: hip.as_ref().and_then(|d| d.max.value()),
        d_min,
    }
}

/// Derive each leg's two-bone rig from a body plan's **declared soles** (B0):
/// a stance chain is found by walking parents from each `sole`-bearing segment
/// to the first branch point, and a two-bone rig is built when that chain is
/// exactly two segments — `[lower, upper]`. `hip` = the pivots accumulated
/// from the root down to the upper bone; `l1` = |lower.pivot| (hip→knee);
/// `l2` = the distance from the knee pivot to the declared sole anchor
/// (knee→sole).
///
/// The chain walk itself is the SHARED dc-api derivation
/// ([`dc_api::bodies::stance_chains`]) — hoisted 2026-08-02 so the resting
/// bake and this rig builder derive chains from one function instead of two
/// copies of one truth (posture-bake audit F6; the byte-identical rigs are
/// pinned by the tests below).
///
/// This retires the `starts_with("leg_")` / `_upper`→`_lower` naming coupling
/// that used to live here (anti-shape A-7 — a content identity inside a
/// process; flagged in its own doc comment since the second-plan report). A
/// plan now gets foot placement by *declaring* where it meets the ground, in
/// any names it likes. A sole chain that is not two bones gets no rig — the
/// solver is two-bone; a longer chain is a feature the first such body will
/// have to argue for, not a silent partial answer.
pub fn leg_rigs(plan: &BodyPlan) -> Vec<LegRig> {
    let seg_by = |name: &str| plan.segments.iter().find(|s| s.name == name);
    // B7: the plan's joint limits, derived once here and carried on each rig.
    // Returned, not stored (S-3) — this local is the memoization, and the rig
    // holds resolved numbers rather than a second copy of the plan.
    let limits = dc_api::bodies::derive_joint_limits(plan);
    let mut legs = Vec::new();
    for chain in dc_api::bodies::stance_chains(plan, "sole") {
        // The chain is DERIVED (shared walk, dc-api), the contact is
        // DECLARED: `[sole segment, .., topmost bone]`, parents walked to
        // the first branch point (≥2 children) or the root.
        let &[lower, upper] = &chain[..] else {
            continue; // not a two-bone chain; no rig (see doc comment)
        };
        // Hip = every pivot from the root down to (and including) the upper
        // bone. For the shipped plans this is trunk.pivot + upper.pivot,
        // byte-identical to the retired one-level formula.
        let mut hip_local = upper.pivot_m;
        let mut a = upper;
        while let Some(p) = a.parent.as_deref().and_then(seg_by) {
            hip_local[0] += p.pivot_m[0];
            hip_local[1] += p.pivot_m[1];
            hip_local[2] += p.pivot_m[2];
            a = p;
        }
        let l1 =
            (lower.pivot_m[0].powi(2) + lower.pivot_m[1].powi(2) + lower.pivot_m[2].powi(2)).sqrt();
        let anchor = lower
            .roles
            .iter()
            .find(|r| r.role == "sole")
            .and_then(|r| r.at_m)
            .unwrap_or([0.0, -(lower.offset_m[1].abs() + lower.size_m[1] / 2.0), 0.0]);
        // Knee→sole. The straight-down fast path is exact where the anchor
        // sits directly under the pivot (every shipped plan) — byte-identical
        // to the retired `|offset.y| + size.y/2`; the general norm serves an
        // offset anchor without pretending the shipped numbers moved.
        let l2 = if anchor[0] == 0.0 && anchor[2] == 0.0 {
            anchor[1].abs()
        } else {
            (anchor[0].powi(2) + anchor[1].powi(2) + anchor[2].powi(2)).sqrt()
        };
        legs.push(LegRig {
            upper: upper.name.clone(),
            lower: lower.name.clone(),
            hip_local,
            l1,
            l2,
            limits: leg_limits(&limits, &upper.name, &lower.name, l1, l2),
        });
    }
    legs
}

/// The derived resting-root DELTA the renderer applies on top of the authored
/// root pivot, metres (posture-bake consumer slice, 2026-08-02; audit
/// `docs/audits/2026-08-02-posture-bake-member0-design.md` § 5): the standing
/// bake's `root_height_m` minus the plan's authored root `pivot_m[1]`. The
/// pelvis stops being pinned at the authored hip — the feet pin it, and the
/// root lands at chain reach (biped 0.880 m, stout 0.440 m, longleg 1.020 m
/// against authored 0.900/0.460/0.900).
///
/// `Err(reason)` when the bake declines — a mode-less plan (a tree, a legal
/// absence) or an out-of-domain geometry (the bake's own loud refusal). The
/// caller then renders with the authored pivot unchanged (the IDENTITY
/// FALLBACK: a plan that cannot bake keeps rendering as it always did) and
/// warns once, naming why.
///
/// `"stand"` is the slice-one posture placeholder (the bake's own module doc:
/// the posture axis becomes registry content by namespaced id —
/// `dc:posture/stand` — at the consumer migration). The delta is applied at
/// the CONSUMER — [`leg_rigs`]' `hip_local` stays authored geometry, and
/// `character.rs` adds this delta to both the rendered root and the IK hip.
pub fn derived_root_delta_m(plan: &BodyPlan) -> Result<f64, String> {
    use dc_api::bodies::{BakeOutcome, bake_resting_posture};
    let Some(root) = plan.segments.iter().find(|s| s.parent.is_none()) else {
        return Err(format!("plan `{}` has no root segment", plan.name));
    };
    match bake_resting_posture(plan, "stand") {
        BakeOutcome::Baked(p) => Ok(p.root_height_m - root.pivot_m[1]),
        BakeOutcome::NothingDeclared => Err(format!(
            "plan `{}` declares no locomotion modes (a tree) — no standing posture to derive",
            plan.name
        )),
        BakeOutcome::Unsupported { reason } => Err(reason),
    }
}

/// The DERIVED gait for a plan's standing mode, baked once per plan
/// (`dc_api::bodies::bake_gait`) — the consumer edge of gait member #1.
///
/// **Gravity is the world's and is passed in, never assumed.** The design
/// derived its whole predicted table at Earth's 9.81 m/s²; a gait baked against
/// the wrong `g` would be dimensionally coherent and wrong for the world the
/// body walks in — the closed-system scale error, one tier down. So the caller
/// reads it from `CharacterConfig` and states it.
///
/// `Err(reason)` on any decline — a mode-less plan (a tree, a legal absence),
/// an out-of-domain geometry, a chain the two-bone clearance solve cannot serve.
/// The caller then renders the identity fallback (no locomotion, the resting
/// pose, clips only) and warns once naming why: **a body that cannot bake a
/// gait stands still rather than moving wrongly.**
pub fn derived_gait(plan: &BodyPlan, gravity_m_s2: f64) -> Result<GaitVector, String> {
    use dc_api::bodies::{GaitBakeOutcome, GaitKnobs, bake_gait};
    match bake_gait(plan, "stand", gravity_m_s2, &GaitKnobs::default()) {
        GaitBakeOutcome::Baked(v) => Ok(v),
        GaitBakeOutcome::NothingDeclared => Err(format!(
            "plan `{}` declares no locomotion modes (a tree) — no gait to derive",
            plan.name
        )),
        GaitBakeOutcome::Unsupported { reason } => Err(reason),
    }
}

/// Foot position of a two-bone leg in its sagittal (y, z) plane, from a clip's
/// hip/knee X-rotations — the inverse of [`solve_leg_ik`]'s reconstruction, used
/// to find where the *animation* puts the foot before the IK re-seats it.
pub fn fk_foot_local(l1: f64, l2: f64, upper_x: f64, lower_x: f64) -> (f64, f64) {
    let ky = -l1 * upper_x.cos();
    let kz = -l1 * upper_x.sin();
    let total = upper_x + lower_x;
    (ky - l2 * total.cos(), kz - l2 * total.sin())
}

/// A two-bone IK solution: the upper and lower joint rotations (radians, about
/// X — the sagittal plane), with the rest pose pointing straight down (−Y).
/// `upper_x` is applied to the hip joint (relative to the trunk), `lower_x` to
/// the knee joint (relative to the upper). The renderer applies these **exactly**
/// — the 11.25° snap that used to round them off was removed 2026-08-01.
#[derive(Clone, PartialEq, Debug)]
pub struct LegIk {
    pub upper_x: f64,
    pub lower_x: f64,
    /// Why the solve did or did not land inside the leg's reachable set.
    /// **Replaced `reachable: bool` 2026-08-03 (B7 § 5.3):** "too far" and "too
    /// folded" are opposite failures that want opposite caller behaviour, and
    /// one flag could not carry the second.
    pub reach: Reach,
}

/// The outcome of a two-bone solve against the leg's declared limits.
#[derive(Clone, PartialEq, Debug)]
pub enum Reach {
    Ok,
    /// The target is beyond `l1 + l2`. The leg is left reaching straight at it
    /// (fully extended) and the caller lets the foot float — pre-B7 behaviour,
    /// unchanged: this is the annulus's OUTER edge, not a joint limit.
    BeyondExtension,
    /// The target is inside `d_min` — closer to the hip than the knee's own
    /// range allows the foot to come. **The pose is refused, never clamped**:
    /// solving freely and then clamping the knee into range moves the foot off
    /// its target with no signal, which is a working call, a passing test and a
    /// silently wrong output (A-3).
    BeyondFlexion {
        joint: String,
    },
    /// The solve landed outside a joint's declared range on an axis the solver
    /// does not solve into (the hip — restricting the *upper* joint turns the
    /// reachable set from an annulus sector into a lune, which the closed form
    /// does not cover), or the joint does not declare the axis the solver
    /// assumes. Checked and reported, never solved into.
    JointBlocked {
        joint: String,
        axis: Axis,
    },
}

/// Closed-form two-bone IK in the sagittal (Y–Z) plane, **within the rig's
/// declared limit set**. `target` is the desired foot position **relative to the
/// hip joint** (meters; x lateral, y up, z forward = −z).
///
/// **The reachable set is an annulus SECTOR, not an annulus** (B7 § 5.3): the
/// inner bound is `rig.limits.d_min`, the distance at full flexion, not the
/// *numerical* `|l1 − l2|` that only ever kept `acos` in domain. On the shipped
/// plans that moves the inner radius by 9.6× (biped) to 23.7× (stout) — and it
/// retires by construction the degenerate 180° stout crouch journal/0131
/// measured, which today folds the knee flat to chase a target *below the
/// ground* and leaves a ~10 mm residual nobody is told about.
///
/// **The knee pole is a READ, not a heuristic.** With a one-sided knee range the
/// pole is *determined* by the sign of the range: exactly one of the two
/// candidates lands inside it.
///
/// ⚠ STAND-IN — `stubs.md` (B7-a, `the-fold-sense-is-declared-because-our-bodies-have-no-front`).
/// When BOTH candidates are admissible — a symmetric derived range, which is
/// every shipped plan, because every box is z-centred and the bodies have no
/// front — the tie is broken by the pre-B7 forward-pole convention (*"the knee
/// pole points forward (−Z), so knees bend like knees"*). That is an
/// undeclared anatomical assumption surviving as a **tie-break**, and it is what
/// keeps the identity default byte-identical; the heir is a declaration, and it
/// disappears the moment a plan states which way its knee folds.
pub fn solve_leg_ik(rig: &LegRig, target: [f64; 3]) -> LegIk {
    let (l1, l2) = (rig.l1, rig.l2);
    let ty = target[1];
    let tz = target[2];
    let d_full = (target[0] * target[0] + ty * ty + tz * tz).sqrt();
    let max = l1 + l2;
    let min = (l1 - l2).abs();
    let eps = 1e-6;
    // Solve within the sagittal plane on the planar (y, z) distance.
    let d_planar = (ty * ty + tz * tz).sqrt();
    let d = d_planar.clamp(min + eps, max - eps);
    // Unit direction to the target (fall back to straight down at the hip).
    let (uy, uz) = if d_planar > 1e-9 {
        (ty / d_planar, tz / d_planar)
    } else {
        (-1.0, 0.0)
    };
    // Hip offset angle from the hip→target line (law of cosines).
    let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let sin_a = (1.0 - cos_a * cos_a).max(0.0).sqrt();
    // Two knee candidates (pole ±); pick the more forward one (smaller z) so the
    // knee always bends toward −Z.
    let knee = |pole: f64| {
        let ky = l1 * (cos_a * uy + pole * sin_a * (-uz));
        let kz = l1 * (cos_a * uz + pole * sin_a * uy);
        (ky, kz)
    };
    // Absolute X-rotation of a planar point p: rotX(θ)·(0,−1,0) = (−cosθ, −sinθ)
    // in (y, z), so θ = atan2(−p.z, −p.y).
    let (fy, fz) = (uy * d, uz * d);
    let solve = |(ky, kz): (f64, f64)| {
        let upper_x = (-kz).atan2(-ky);
        let total = (-(fz - kz)).atan2(-(fy - ky));
        (upper_x, wrap_pi(total - upper_x))
    };
    let (ka, kb) = (knee(1.0), knee(-1.0));
    let (sa, sb) = (solve(ka), solve(kb));
    // THE POLE, as declared data: keep the candidate whose knee angle is inside
    // the declared range. Both admissible (a symmetric derived range — every
    // shipped plan) falls back to the forward-pole tie-break; see the doc
    // comment's STAND-IN marker.
    let admits = |lower_x: f64| {
        rig.limits.knee_min.is_none_or(|lo| lower_x >= lo - 1e-9)
            && rig.limits.knee_max.is_none_or(|hi| lower_x <= hi + 1e-9)
    };
    let (upper_x, lower_x) = match (admits(sa.1), admits(sb.1)) {
        (true, false) => sa,
        (false, true) => sb,
        _ => {
            if ka.1 <= kb.1 {
                sa
            } else {
                sb
            }
        }
    };
    let reach = if !rig.limits.knee_sagittal {
        Reach::JointBlocked {
            joint: rig.lower.clone(),
            axis: Axis::X,
        }
    } else if d_planar < rig.limits.d_min - 1e-9 {
        // Inside the sector's inner radius: the knee cannot fold far enough.
        Reach::BeyondFlexion {
            joint: rig.lower.clone(),
        }
    } else if d_full > max + 1e-9 {
        Reach::BeyondExtension
    } else if rig.limits.hip_sagittal
        && (rig.limits.hip_min.is_some_and(|lo| upper_x < lo - 1e-9)
            || rig.limits.hip_max.is_some_and(|hi| upper_x > hi + 1e-9))
    {
        // The hip is CHECKED, not solved into: restricting the upper joint
        // turns the reachable set from a sector into a lune, which this closed
        // form does not cover. Reporting it beats pretending to serve it.
        Reach::JointBlocked {
            joint: rig.upper.clone(),
            axis: Axis::X,
        }
    } else {
        Reach::Ok
    };
    LegIk {
        upper_x,
        lower_x,
        reach,
    }
}

/// The pose to render for this state: **the derived gait, plus non-locomotion
/// clips as an additive layer** (design § 6's composition rule).
///
/// 1. **The gait OWNS** every segment on a bearing chain, and the non-bearing
///    limbs it derives a counter-swing for. It is sampled at the latched
///    `(phase, Fr)`, which is what makes the pose stepped.
/// 2. **A clip is ADDITIVE on non-bearing segments** — this is how a sword
///    swing rides a walk, and it is already the shape `character.rs` uses for
///    the look joint (`base + orient`). The idle breath lands on the arms this
///    way, over their derived counter-swing, with no blend weight anywhere: at
///    v → 0 the counter-swing amplitude is zero of its own accord.
/// 3. **A clip's contribution to a BEARING chain is REFUSED.** You cannot
///    author a leg over a locomotion gait; that request is a gait-type choice
///    or an override, and refusing it here is cheaper than blending two
///    authorities at runtime. (Refused silently at the consumer today; the
///    define-time refusal wants the § 7 binding vocabulary and is not this
///    slice's.)
///
/// `gait: None` is the identity fallback — a plan the bake declined renders its
/// clips over an unposed skeleton, exactly as a clip-only body always did.
///
/// # 4. **The rate is PER BONE** (user, 2026-08-04)
///
/// Each anim has its own cycle and therefore its own `N`; a bone several anims
/// own is sampled at the **blended time step**, weighted by the proportion of
/// them that own it. *"A weighted blend is a weighted blend, a single owner is a
/// single owner."* There is no precedence rule because this is interpolation,
/// not arbitration — the two-blended-cycles problem dissolves.
///
/// Concretely, on the shipped biped: the legs are a bearing chain the gait owns
/// outright, so they sit on the gait's whole `N`. The arms are owned by the gait
/// (counter-swing) **and** the idle clip, so they sit on the mean of the two
/// time steps — a bucket count a few hundredths off an integer, because both
/// steps derive from the same target and differ only by rounding. The body is
/// therefore not one exact snapshot, and the user is explicitly not worried
/// about that reading as buggy; this is why.
pub fn pose_for(
    state: &AnimState,
    gait: Option<&GaitVector>,
    clips: &[&AnimClip],
    rate: AnimRate,
) -> Pose {
    // The gait's own cycle, and the fallback for an anim with no usable one:
    // the client's base step, which is what "we do not know this cycle's
    // duration" honestly means.
    let gait_cycle = state.gait_cycle(gait, rate);
    let gait_step = gait_cycle.map_or(rate.step_s(), CycleGrid::step_s);

    // --- who owns what -----------------------------------------------------
    // Ownership is a property of the ANIM, not of the current keyframe bracket:
    // a clip that names a joint anywhere owns it for the whole cycle, or a
    // bone's rate would flicker as keyframes came and went.
    //
    // **Held as short name LISTS, not as a map, and that is a perf call with a
    // receipt.** This runs per body per frame. A `HashMap<&str, StepBlend>` over
    // ~20 bones cost ~60 sip-hashes and measured **+1.6 µs per body per frame**
    // against a 3.3 µs baseline; a clip names two or three segments and a limb
    // has two or three joints, so a linear scan of a three-element slice beats
    // hashing outright at these sizes. `Vec::contains` on `&str` is a length
    // check and a memcmp.
    let mut bearing: Vec<&str> = Vec::new();
    let mut gait_names: Vec<&str> = Vec::new();
    if let Some(g) = gait {
        for limb in &g.limbs {
            for ja in &limb.neutral {
                gait_names.push(ja.segment.as_str());
                if limb.bearing {
                    bearing.push(ja.segment.as_str());
                }
            }
        }
    }
    // Per clip: its grid, its time step, and the segments it owns — bearing
    // chains excluded, because rule 3's refusal is refused ownership too: a
    // contribution that will not be applied may not drag the bone's rate either.
    let mut clip_rates: Vec<(Option<CycleGrid>, f64, Vec<&str>)> =
        Vec::with_capacity(clips.len());
    for clip in clips {
        let cycle = rate.cycle(clip.duration_s);
        let step = cycle.map_or(rate.step_s(), CycleGrid::step_s);
        let mut named: Vec<&str> = Vec::new();
        for kf in &clip.keyframes {
            for r in &kf.rotations {
                let name = r.segment.as_str();
                if !bearing.contains(&name) && !named.contains(&name) {
                    named.push(name);
                }
            }
        }
        clip_rates.push((cycle, step, named));
    }
    // One bone's blended rate. Every layer contributes with unit weight (the
    // composition carries no blend weights anywhere), so a bone's `k` owners
    // hold `1/k` each and the blend is a mean; the weighted form is the seam for
    // the day layers carry real weights.
    let blended = |name: &str| -> Option<f64> {
        let mut blend = StepBlend::default();
        if gait_names.contains(&name) {
            blend.add(gait_step, 1.0);
        }
        for (_, step, named) in &clip_rates {
            if named.contains(&name) {
                blend.add(*step, 1.0);
            }
        }
        blend.step_s()
    };

    // --- sample each anim, per bone-rate group -----------------------------
    let mut pose = Pose::default();
    // Two scratch buffers, allocated once per call and reused per anim: this
    // bone's rate beside its name, and the distinct rates among them.
    let mut rates: Vec<(&str, f64)> = Vec::new();
    let mut groups: Vec<f64> = Vec::new();
    if let Some(g) = gait {
        for limb in &g.limbs {
            group_rates(
                limb.neutral.iter().map(|j| j.segment.as_str()),
                &blended,
                &mut rates,
                &mut groups,
            );
            for &step in &groups {
                let phase = phase_on(gait_cycle, step, state.phase);
                for ja in g.limb_pose(limb, state.stepped_froude, phase) {
                    if rate_of(&rates, ja.segment.as_str()) == Some(step) {
                        pose.joints.insert(ja.segment, ja.euler);
                    }
                }
            }
        }
    }
    for (clip, (cycle, _, named)) in clips.iter().zip(&clip_rates) {
        let live = clip_phase(clip, state.clock_s);
        group_rates(named.iter().copied(), &blended, &mut rates, &mut groups);
        for &step in &groups {
            let layer = sample_clip_at_phase(clip, phase_on(*cycle, step, live));
            for (name, euler) in layer.joints {
                // Rule 3: the gait owns a bearing chain outright, so a clip's
                // contribution to one is dropped here as it always was.
                if rate_of(&rates, name.as_str()) != Some(step) {
                    continue;
                }
                let base = pose.joints.entry(name).or_insert([0.0, 0.0, 0.0]);
                for i in 0..3 {
                    base[i] += euler[i];
                }
            }
        }
    }
    pose
}

/// One bone's rate out of a small name→rate list.
fn rate_of(rates: &[(&str, f64)], name: &str) -> Option<f64> {
    rates.iter().find(|(n, _)| *n == name).map(|&(_, s)| s)
}

/// Blend every named bone's rate, and collect the **distinct** ones — the groups
/// one anim has to be sampled at.
///
/// Usually **one group**: on the shipped bodies every bone of a limb has the
/// same owner set, so the per-bone rate costs no extra sampling at all. Compared
/// by bits, which is exact rather than approximate because equal owner sets
/// accumulate in the same order and so agree to the last bit.
fn group_rates<'a>(
    names: impl Iterator<Item = &'a str>,
    blended: &impl Fn(&str) -> Option<f64>,
    rates: &mut Vec<(&'a str, f64)>,
    groups: &mut Vec<f64>,
) {
    rates.clear();
    groups.clear();
    for name in names {
        let Some(s) = blended(name) else { continue };
        rates.push((name, s));
        if !groups.iter().any(|x| x.to_bits() == s.to_bits()) {
            groups.push(s);
        }
    }
}

/// One anim's phase, held on the grid a bone with time step `step_s` sees.
///
/// `cycle: None` — an anim whose duration is not known — returns the live phase.
/// That is the **limit**, not a special case: as a cycle lengthens its `N` grows
/// and its phase grid vanishes continuously, so a body slowing to a stop
/// dissolves its quantization rather than snapping out of it.
fn phase_on(cycle: Option<CycleGrid>, step_s: f64, phase: f64) -> f64 {
    cycle.map_or(phase, |c| {
        PhaseGrid::quantize(c.phases_at_step(step_s), phase)
    })
}

/// **THE ONE VERTICAL COMPOSITION** — `root_offset(posture, mode, phase)`,
/// design § 4.3. Metres above the plan's authored root pivot, and the *whole*
/// vertical story: read once per body per frame and consumed by **both** the
/// render root and the IK hip.
///
/// Three terms that used to be able to disagree collapse into one call:
///
/// - `root_delta_m` — member #0's static resting delta (the derived standing
///   root minus the authored pivot), the phase-INVARIANT part;
/// - the stance-chain drop, the phase-VARYING part: during its compass window a
///   stance chain is a rigid link about a pinned contact, so the attachment
///   joint traces `h = L·cos θ`, peaking at midstance and troughing at the
///   contact extremes. **Nothing is authored and there is no toggle** — a body
///   bobs iff its stance contacts alternate, and by exactly as much as its own
///   geometry says. A distributed-bearing body (a snake's belly) gets exactly
///   zero, and needs no flag to say so;
/// - the crouch sink, which is a **posture** and will stop being a term at all
///   once crouch is its own bake key (`CROUCH_ROOT_DROP_M` is scheduled
///   demolition, corrections #81).
///
/// **A declined root height holds the neutral height**, it does not fabricate a
/// flight arc: above the walk/run transition the apex is ballistic and needs
/// takeoff force, which does not exist at density ≡ 1 (`stubs.md` #39, heir
/// B6). The bake reports the decline with the Froude number; the renderer's
/// honest response is to stop bobbing, not to invent a parabola.
///
/// ⚠ **And that decline is the ONE step in an otherwise continuous ladder, by
/// construction: crossing the walk/run transition drops the root excursion to
/// zero in one frame** (≈ 0.10 m for the shipped biped, at v ≈ 2.08 m/s). It is
/// not a gait switch — nothing branches, no state exists, every *joint* stays
/// continuous — it is the visible edge of the model's domain, exactly where
/// **G5 / `stubs.md` #39** says the derivation stops. **Smoothing it would mean
/// interpolating toward a value the bake refused to compute**, which is the
/// fabrication the ruling forbids by name. It goes when B6 gives the flight
/// phase a real answer, not before. *Reported here because a walk will see it
/// and should recognise it rather than file it as a bug.*
///
/// **Why this rather than a tidier version of the falsified fix** (corrections
/// #80): that one *added* the bob to the hip, preserving two expressions and
/// asking them to agree. This deletes the additive term, so there is no second
/// place to forget it. `the_render_root_and_the_ik_hip_read_one_root_offset`
/// is the test that keeps it that way.
#[must_use]
pub fn root_offset_m(
    gait: Option<&GaitVector>,
    root_delta_m: f64,
    froude: f64,
    phase: f64,
    posture: Posture,
) -> f64 {
    let stance_drop_m = gait
        .and_then(|g| {
            g.root_height_ratio_at(froude, phase)
                .map(|ratio| g.governing_reach_m * (ratio - 1.0))
        })
        .unwrap_or(0.0);
    let crouch_drop_m = match posture {
        Posture::Crouching => CROUCH_ROOT_DROP_M,
        Posture::Standing => 0.0,
    };
    root_delta_m + stance_drop_m - crouch_drop_m
}

// ------------------------------------------- the retargeting measurement --

/// The ground the headless probe stands a body on.
///
/// **Flat ground is the degenerate case** and journal/0130 only ever measured it:
/// both feet want the same answer, so a flat-ground null says nothing about *foot
/// placement* — only about the hip/reach arithmetic that is identical for both
/// legs. The step cases are the real test.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GroundCase {
    /// Both soles target y = 0.
    Flat,
    /// The leg whose hip is on the **+X** side stands on a step `rise_m` above the
    /// other; the root does not move (the renderer never moves it for terrain —
    /// only the leg IK adapts). The two feet now want **different** answers.
    Step { rise_m: f64 },
}

#[cfg(test)]
impl GroundCase {
    fn ground_m(self, leg: &LegRig) -> f64 {
        match self {
            GroundCase::Flat => 0.0,
            GroundCase::Step { rise_m } => {
                if leg.hip_local[0] > 0.0 {
                    rise_m
                } else {
                    0.0
                }
            }
        }
    }

    fn label(self) -> String {
        match self {
            GroundCase::Flat => "flat".to_string(),
            GroundCase::Step { rise_m } => format!("step+{rise_m:.3}"),
        }
    }
}

/// What the renderer did with one foot on one frame — the per-sample verdict the
/// summary counts are an itemisation *of*.
///
/// Two levels, and both close on their own total:
/// `Seated + (Corrected + ClampedBeyondReach) + Refused == samples`.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    /// The clip already put the sole within 1 mm of the ground; the renderer
    /// leaves it alone (`character.rs`'s `adjust.abs() > 1e-3` guard).
    Seated,
    /// Inside the half-voxel window **and** the sole target was inside the leg's
    /// annulus: the IK ran and the knee bent.
    Corrected,
    /// Inside the window, but seating the sole was **beyond `l1 + l2`**:
    /// `solve_leg_ik` clamps to full extension, the knee stays straight, and the
    /// foot hovers. This is the state journal/0130 found in 176/176 samples.
    ClampedBeyondReach,
    /// Outside the half-voxel window: the renderer does not correct at all and the
    /// foot floats honestly.
    Refused,
}

#[cfg(test)]
impl Placement {
    fn code(self) -> &'static str {
        match self {
            Placement::Seated => "seat",
            Placement::Corrected => "CORR",
            Placement::ClampedBeyondReach => "clmp",
            Placement::Refused => "ref!",
        }
    }
}

/// One foot on one frame. **The time series is the deliverable** (corrections
/// #76: a still frame is structurally blind to a temporal artifact, and a
/// min/max *range* is an amplitude, not an error bar) — so the probe records
/// every sample rather than only its envelope.
#[cfg(test)]
#[derive(Clone, PartialEq, Debug)]
pub struct LegSample {
    pub leg: String,
    /// Ground height under this foot, metres.
    pub ground_m: f64,
    /// Sole height the **clip alone** puts the foot at, before any IK.
    pub clip_sole_m: f64,
    /// Sole height **as rendered** (IK where the renderer applies it).
    pub rendered_sole_m: f64,
    /// Signed distance from the ground under this foot, as rendered. This is the
    /// number that carries the hover.
    pub gap_m: f64,
    /// The **hip** joint's rendered X rotation, degrees.
    pub hip_deg: f64,
    /// The **knee** joint's rendered X rotation, degrees; `0` is a dead-straight
    /// leg, negative bends the knee forward. *The answer to "does the knee bend".*
    pub knee_deg: f64,
    pub verdict: Placement,
}

/// One stepped animation frame, all legs.
#[cfg(test)]
#[derive(Clone, PartialEq, Debug)]
pub struct FrameRow {
    pub index: usize,
    pub t_s: f64,
    pub legs: Vec<LegSample>,
}

/// What the retargeting glue actually achieves for one (plan, clip, ground,
/// posture) case — **the instrument for bodies.md's IK claim** ("one clip serves
/// every mutation of a plan … across differing proportions") and, since
/// journal/0130, for the prior question of whether the foot IK ever runs to
/// completion at all.
///
/// **Foot placement has TWO independent gates and the probe reports both.** A
/// correction happens only if the ground is inside the **half-voxel window**
/// (`refused` counts the failures) *and* the sole target is inside the leg's
/// **annulus** (`clamped_beyond_reach` counts those). Nothing reconciles the two,
/// and no plan yet passes both in every posture.
///
/// Every figure is in **metres**, measured over the clip's *stepped* frames (its
/// own per-cycle grid at the default target, which is what the renderer samples),
/// for every leg the plan declares.
///
/// `#[cfg(test)]`: this is an instrument, not shipped renderer code, and it lives
/// beside the production math it re-derives so the two cannot drift. It runs in
/// the ordinary workspace gate (`foot_placement_and_retargeting_are_measured`) —
/// the `examples/ … test = true` dance in CLAUDE.md § Gates exists for probes that
/// need a printed `main`; this one prints from the test under `--nocapture`.
#[cfg(test)]
#[derive(Clone, PartialEq, Debug)]
pub struct RetargetReport {
    pub plan: String,
    pub clip: String,
    /// Which ground the body stood on.
    pub ground: String,
    /// How far the cosmetic root was sunk, metres — `0` standing,
    /// [`CROUCH_ROOT_DROP_M`] crouching. **A fourth absolute-metres constant** in a
    /// pipeline whose premise is that proportions vary (bodies.md § IK names three
    /// and misses this one): 0.45 m is 50% of the biped's hip height and **97.8% of
    /// the stout's**.
    pub root_drop_m: f64,
    /// Hip height above the feet, metres (the plan's trunk pivot + hip offset), as
    /// authored — *before* the bake delta and the crouch drop. See
    /// [`RetargetReport::hip_derived_m`] and [`RetargetReport::hip_eff_m`].
    pub hip_m: f64,
    /// The posture bake's root delta ([`derived_root_delta_m`]), metres — the
    /// same number production's `build_plan_assets` caches and the pose loop
    /// applies (2026-08-02, the consumer slice). `0.0` mirrors production's
    /// identity fallback for a plan the bake declines.
    pub root_delta_m: f64,
    /// Total leg reach `l1 + l2`, metres — the IK's outer annulus.
    pub reach_m: f64,
    /// **The whole time series**, frame by frame, both feet. Not a range.
    pub frames: Vec<FrameRow>,
    /// Frames × legs measured.
    pub samples: usize,
    /// Sole height the **clip alone** puts the foot at: the raw retarget, before
    /// any foot IK. Negative = through the floor, positive = floating.
    pub clip_sole_min_m: f64,
    pub clip_sole_max_m: f64,
    /// Sole height **as rendered**: foot IK applied where the renderer applies it
    /// (inside the half-voxel window).
    pub rendered_sole_min_m: f64,
    pub rendered_sole_max_m: f64,
    /// Samples the clip already put within 1 mm of the ground.
    pub already_seated: usize,
    /// Samples the renderer **corrected** and the solver **reached**: the IK ran to
    /// completion and the knee bent. Zero here across every plan was journal/0130's
    /// refutation.
    pub corrected: usize,
    /// Samples inside the window where the target was beyond `l1 + l2`: clamp
    /// saturation, knee straight, foot hovering.
    pub clamped_beyond_reach: usize,
    /// Samples the renderer refused to correct because the ground was further than
    /// half a voxel from the clip's foot — the foot floats honestly.
    pub refused: usize,
    /// Worst |knee angle| over the whole series, degrees — *did any knee bend?*
    /// Includes REFUSED and SEATED samples, whose rendered knee is the clip's
    /// own authored angle rather than anything the solver produced.
    pub knee_bend_max_deg: f64,
    /// Worst |knee angle| over the samples the **solver actually ran on**
    /// (corrected or clamped), degrees. *This, not the field above, is the
    /// number that answers "did the IK bend a knee".*
    ///
    /// Split out 2026-08-03, and the gate is what forced it: with the clip's
    /// root bob gone the jump clip's hip sits lower, some samples fall outside
    /// the half-voxel window and are honestly refused — and a refused sample
    /// renders the **clip's** knee (3.8° on jump), which was being counted
    /// against a bound derived from the *solver's* annulus clamp (~0.25°). The
    /// assertion was measuring one thing and claiming another; it passed before
    /// only because the bob happened to keep every sample inside the window.
    pub solver_knee_bend_max_deg: f64,
    /// Worst residual over samples the IK both attempted and could reach — how far
    /// off the ground the foot still is after the glue ran, in metres. Until
    /// 2026-08-01 this measured the **rotation quantum's** cost (bounded by
    /// `reach × 11.25°`, ~172 mm on the biped); with the quantizer gone it measures
    /// the **solver's own** residual, bounded by its annulus-clamp epsilon.
    pub reachable_residual_max_m: f64,
    /// Samples the solver **refused on a JOINT LIMIT** (B7): the target was
    /// inside `d_min`, or a joint declared no sagittal DOF. Counted separately
    /// from `refused` (outside the half-voxel window) because they are
    /// different refusals — one is the renderer declining to correct, the other
    /// is the body declining to bend.
    pub refused_by_limits: usize,
}

#[cfg(test)]
impl RetargetReport {
    /// The corrections the renderer attempted, reached or not.
    pub fn inside_window(&self) -> usize {
        self.corrected + self.clamped_beyond_reach
    }

    /// The hip the renderer STANDS the body at (2026-08-02, the consumer
    /// slice): authored hip + the posture bake's delta. Equal to `reach_m`
    /// exactly for every plan the bake serves — the derived root IS the
    /// chain reach, which puts a standing sole ON the IK annulus boundary
    /// (posture-bake audit § 2.2).
    pub fn hip_derived_m(&self) -> f64 {
        self.hip_m + self.root_delta_m
    }

    /// The hip height the solver actually works against: the DERIVED hip
    /// minus the crouch drop. **This, not `hip_m`, is what `reach_m` has to
    /// beat.** Under the pinned hip this decided standing-vs-crouching
    /// engagement (journal/0130); under the derived hip, standing sits at
    /// zero slack exactly and only the crouch drop opens slack.
    pub fn hip_eff_m(&self) -> f64 {
        self.hip_m + self.root_delta_m - self.root_drop_m
    }
}

/// Measure one plan against one clip on one ground, in one posture.
/// `voxel_size_m` is the active scale's voxel edge — the foot-IK correction window
/// is **half a voxel**, an absolute length, which is precisely the kind of constant
/// a second plan exists to interrogate. `root_drop_m` is the cosmetic crouch sink
/// (`0` standing, [`CROUCH_ROOT_DROP_M`] crouching), which is the *other* absolute
/// length that decides reachability.
///
/// This deliberately **re-implements** the renderer's foot-placement decision
/// (`character.rs`: correct only inside the half-voxel window), because the real
/// one needs a bevy world
/// and a voxel query. The duplication is the honest cost of measuring a render path
/// headlessly, and it is the one thing in this file that can silently disagree with
/// production — noted as a loose end.
///
/// **One known divergence, stated rather than hidden**: production reads ground
/// through `character.rs::ground_top_m`, which scans a half-voxel band around the
/// foot and returns the *top face of the highest solid voxel it finds* — so it can
/// return a surface further away than the band, and it returns `None` (no
/// correction) where the band holds no solid. This probe is handed the ground
/// height directly. The two agree on the **decision** in every case measured here
/// (correct iff the ground is within half a voxel of the clip's foot); they would
/// diverge on an overhang.
///
/// **The derived hip is mirrored, not re-invented**: the probe composes its hip
/// from the SAME [`root_offset_m`] production's pose loop composes, with the
/// same identity fallback — so the instrument keeps measuring what production
/// does. Corrections #80's temporal half, which used to ride here as a
/// bob-tracking hover in the gap column, is **gone**: there is one expression
/// now and the probe reads it too.
///
/// **This measures CLIPS, not the gait** — the non-locomotion layer and the
/// parked walk fixture, standing still (Fr = 0, phase 0). Whether the *derived*
/// gait seats its feet is a different question, asked by
/// `three_bodies_walk_at_their_own_cadence` and ultimately by a walk.
#[cfg(test)]
pub fn retarget_report(
    plan: &BodyPlan,
    clip: &AnimClip,
    voxel_size_m: f64,
    ground: GroundCase,
    root_drop_m: f64,
) -> Option<RetargetReport> {
    let legs = leg_rigs(plan);
    let first = legs.first()?;
    let half_voxel = voxel_size_m * 0.5;
    // Production's derived-hip read, mirrored (identity fallback included).
    let root_delta_m = derived_root_delta_m(plan).unwrap_or(0.0);
    // THE ONE VERTICAL COMPOSITION, read exactly as production reads it. The
    // probe stands the body still, so there is no gait term; `root_drop_m` is
    // the crouch case, expressed as the posture it actually is.
    let posture = if root_drop_m > 0.0 {
        Posture::Crouching
    } else {
        Posture::Standing
    };
    let root_offset = root_offset_m(None, root_delta_m, 0.0, 0.0, posture);
    let mut r = RetargetReport {
        plan: plan.name.clone(),
        clip: clip.name.clone(),
        ground: ground.label(),
        root_drop_m,
        hip_m: first.hip_local[1],
        root_delta_m,
        reach_m: first.l1 + first.l2,
        frames: Vec::new(),
        samples: 0,
        clip_sole_min_m: f64::INFINITY,
        clip_sole_max_m: f64::NEG_INFINITY,
        rendered_sole_min_m: f64::INFINITY,
        rendered_sole_max_m: f64::NEG_INFINITY,
        already_seated: 0,
        corrected: 0,
        clamped_beyond_reach: 0,
        refused: 0,
        knee_bend_max_deg: 0.0,
        solver_knee_bend_max_deg: 0.0,
        reachable_residual_max_m: 0.0,
        refused_by_limits: 0,
    };
    // The clip's OWN grid at the default target — exactly the frames the
    // renderer holds, rather than a second opinion about how many there are.
    let rate = AnimRate::default();
    let grid = rate.cycle(clip.duration_s);
    let frames = grid.map_or(1, |g| g.poses() as usize);
    let step_s = grid.map_or(rate.step_s(), CycleGrid::step_s);
    for i in 0..frames {
        let t_s = i as f64 * step_s;
        let pose = sample_clip(clip, t_s, rate);
        let mut row = FrameRow {
            index: i,
            t_s,
            legs: Vec::new(),
        };
        for leg in &legs {
            let cu = pose.joints.get(&leg.upper).map_or(0.0, |e| e[0]);
            let cl = pose.joints.get(&leg.lower).map_or(0.0, |e| e[0]);
            let (fy, fz) = fk_foot_local(leg.l1, leg.l2, cu, cl);
            // The hip rides at hip_local.y plus THE ONE ROOT OFFSET — the
            // same call production makes, with no gait (this probe stands the
            // body still) and the crouch expressed as the posture it is. The
            // root is where the collider bottom is; terrain never moves it.
            let hip_y = leg.hip_local[1] + root_offset;
            let clip_sole = hip_y + fy;
            let g = ground.ground_m(leg);
            r.samples += 1;
            r.clip_sole_min_m = r.clip_sole_min_m.min(clip_sole);
            r.clip_sole_max_m = r.clip_sole_max_m.max(clip_sole);

            // What the renderer does (character.rs): correct only inside the
            // half-voxel window, and apply the solved angles exactly.
            let adjust = g - clip_sole;
            let (verdict, rendered_sole, hip_x, knee_x) = if adjust.abs() <= 1e-3 {
                (Placement::Seated, clip_sole, cu, cl)
            } else if adjust.abs() <= half_voxel {
                let ik = solve_leg_ik(leg, [0.0, fy + adjust, fz]);
                let (sy, _) = fk_foot_local(leg.l1, leg.l2, ik.upper_x, ik.lower_x);
                let sole = hip_y + sy;
                // B7: a solve that leaves the LIMIT SET is refused, and the
                // clip pose stands — the foot floats honestly rather than
                // lying about contact. `BeyondExtension` is not a limit (it
                // is the annulus's outer edge) and keeps its pre-B7 handling.
                match &ik.reach {
                    Reach::Ok => {
                        r.reachable_residual_max_m =
                            r.reachable_residual_max_m.max((sole - g).abs());
                        (Placement::Corrected, sole, ik.upper_x, ik.lower_x)
                    }
                    Reach::BeyondExtension => {
                        (Placement::ClampedBeyondReach, sole, ik.upper_x, ik.lower_x)
                    }
                    Reach::BeyondFlexion { .. } | Reach::JointBlocked { .. } => {
                        r.refused_by_limits += 1;
                        (Placement::Refused, clip_sole, cu, cl)
                    }
                }
            } else {
                (Placement::Refused, clip_sole, cu, cl)
            };
            match verdict {
                Placement::Seated => r.already_seated += 1,
                Placement::Corrected => r.corrected += 1,
                Placement::ClampedBeyondReach => r.clamped_beyond_reach += 1,
                Placement::Refused => r.refused += 1,
            }
            let knee_deg = knee_x.to_degrees();
            r.knee_bend_max_deg = r.knee_bend_max_deg.max(knee_deg.abs());
            if matches!(
                verdict,
                Placement::Corrected | Placement::ClampedBeyondReach
            ) {
                r.solver_knee_bend_max_deg = r.solver_knee_bend_max_deg.max(knee_deg.abs());
            }
            r.rendered_sole_min_m = r.rendered_sole_min_m.min(rendered_sole);
            r.rendered_sole_max_m = r.rendered_sole_max_m.max(rendered_sole);
            row.legs.push(LegSample {
                leg: leg.upper.trim_end_matches("_upper").to_string(),
                ground_m: g,
                clip_sole_m: clip_sole,
                rendered_sole_m: rendered_sole,
                gap_m: rendered_sole - g,
                hip_deg: hip_x.to_degrees(),
                knee_deg,
                verdict,
            });
        }
        r.frames.push(row);
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim_rate::{DEFAULT_TARGET_FPS, MIN_POSES_PER_CYCLE, blended_step_s};
    use dc_api::bodies::{
        biped_clips, biped_plan, longleg_plan, retired_biped_walk_clip, stout_plan,
    };

    /// **This world's** gravity — READ, never restated (`dc_core`'s world
    /// constant, DECIDED 2026-08-02). The gait's whole Froude chain closes
    /// against it, and a test that hand-types a `g` is the two-authority defect
    /// the constant exists to end.
    const G: f64 = dc_core::DEFAULT_GRAVITY_M_S2;

    fn idle_clip() -> AnimClip {
        biped_clips()
            .into_iter()
            .find(|c| c.name == "dc:anim/biped_idle")
            .expect("the default pack ships an idle clip")
    }

    fn gait_of(plan: &BodyPlan) -> GaitVector {
        derived_gait(plan, G).expect("every shipped plan bakes a gait")
    }

    /// A clip is held on ITS OWN cycle's grid, and the hold is real: two times
    /// inside one held pose sample identically, the next one moves.
    ///
    /// The step is `duration / N`, not `1/fps` — the same number here only
    /// because the shipped idle happens to be frame-aligned (2.0 s, N = 24).
    /// Written against the grid rather than against the target so it stays a
    /// statement about *this clip's* cycle.
    #[test]
    fn a_clip_is_held_on_its_own_cycle_grid() {
        let rate = AnimRate::default();
        let idle = idle_clip();
        let step = rate.cycle(idle.duration_s).expect("a shipped clip").step_s();
        let a = sample_clip(&idle, 0.30, rate);
        let b = sample_clip(&idle, 0.30 + step / 4.0, rate); // the same held pose
        assert_eq!(a, b, "within one held pose the clip does not move");
        let c = sample_clip(&idle, 0.30 + step, rate); // the next one
        assert_ne!(a.joints, c.joints, "the next held pose moves");
    }

    /// **The target is a SETTING, and it is the only thing that decides how many
    /// poses a cycle gets** — A-1's retirement stated as behaviour rather than
    /// as the absence of a constant.
    #[test]
    fn the_target_is_a_setting_and_it_moves_n() {
        let idle = idle_clip(); // 2.0 s, looping
        let mut counts = Vec::new();
        for (fps, expect_n) in [(6.0, 12_u32), (12.0, 24), (30.0, 60)] {
            let rate = AnimRate::new(fps).expect("a legal target");
            let grid = rate.cycle(idle.duration_s).expect("a shipped clip");
            assert_eq!(grid.poses(), expect_n, "at {fps} fps");
            // …and the sampler really uses it. The idle's arm splay is a
            // triangle wave symmetric about mid-cycle, so the *distinct* count
            // is about N/2 — bounded by N, never equal to it, and that is the
            // clip's own shape rather than anything about the grid.
            let seen: std::collections::BTreeSet<u64> = (0..2_000)
                .map(|k| {
                    let t = idle.duration_s * f64::from(k) / 2_000.0;
                    sample_clip(&idle, t, rate).joints["arm_l_upper"][2].to_bits()
                })
                .collect();
            assert!(
                seen.len() >= 2 && seen.len() <= expect_n as usize,
                "at {fps} fps a 2 s cycle showed {} distinct arm angles against \
                 its own N = {expect_n}",
                seen.len()
            );
            counts.push(seen.len());
        }
        assert!(
            counts[0] < counts[1] && counts[1] < counts[2],
            "a higher target must hold fewer poses for longer, got {counts:?}"
        );
    }

    /// One full loop apart samples the same phase — **bit for bit, and the
    /// tolerance is gone.**
    ///
    /// **The history matters more than the assertion.** This test asserted
    /// bit-identity for a year and passed only because the rotation quantizer was
    /// rounding f64 noise away; when that quantizer was removed (2026-08-01) the
    /// identity turned out to be **false by construction** — `quantize_time`
    /// floored onto a grid anchored at absolute `t = 0` and wrapped *afterwards*,
    /// so `(2.4·12).floor()/12 − 2.0` and `(0.4·12).floor()/12` were one real
    /// number in different bits. It was retargeted to a derived 1e-12 tolerance,
    /// which was locally right and **fixed the test to match the
    /// implementation** (anti-shape A-3; ROADMAP § Observed named the one-line
    /// alternative — *wrap first, then quantize* — and deferred it).
    ///
    /// Per-cycle quantization **is** that alternative, not as a fix but as its
    /// definition: [`clip_phase`] wraps into the cycle and the grid divides the
    /// cycle, so there is no absolute-time grid left to anchor anything to. The
    /// assertion is therefore exact again, and it is now asserting what its name
    /// always claimed.
    ///
    /// *It used to also assert the quantized bob repeated exactly. There is no
    /// bob: `Keyframe.root_bob_m` and `BOB_QUANTUM_M` both left with user call
    /// #2, and root height is a phase function now.*
    #[test]
    fn looping_wraps_to_the_same_phase() {
        let rate = AnimRate::default();
        let idle = idle_clip();
        for k in 1..=8 {
            let a = sample_clip(&idle, 0.4, rate);
            let b = sample_clip(&idle, 0.4 + idle.duration_s * f64::from(k), rate);
            assert_eq!(a, b, "cycle {k} did not sample cycle 0's phase exactly");
        }
        // And on a duration that is NOT a whole number of frames — the case the
        // old ordering broke immediately and no shipped clip happened to hit
        // (ROADMAP § Observed: "a pack author writing a 0.7 s looping clip
        // triggers it immediately").
        let mut odd = idle.clone();
        odd.duration_s = 0.7;
        odd.keyframes[1].t = 0.35;
        odd.keyframes[2].t = 0.7;
        for k in 1..=8 {
            let a = sample_clip(&odd, 0.31, rate);
            let b = sample_clip(&odd, 0.31 + odd.duration_s * f64::from(k), rate);
            assert_eq!(
                a, b,
                "a clip whose duration is not a whole number of frames drifted \
                 through its own loop at cycle {k}"
            );
        }
    }

    #[test]
    fn sampler_is_deterministic_for_fixed_time() {
        let idle = idle_clip();
        for t in [0.0, 0.05, 0.333, 0.5, 0.917, 1.4, 7.25] {
            let rate = AnimRate::default();
            assert_eq!(sample_clip(&idle, t, rate), sample_clip(&idle, t, rate));
        }
    }

    #[test]
    fn advance_history_replays_identically() {
        // A fixed (dt, speed) history yields an identical pose stream twice —
        // the whole player, not just the sampler, is deterministic. The gait is
        // a pure function of (plan, g) and the pose a pure function of
        // (phase, Fr), so this is a statement about the whole derived path.
        let idle = idle_clip();
        let gait = gait_of(&biped_plan());
        let history = [
            (0.016, 0.0),
            (0.016, 0.0),
            (0.02, 2.0),
            (0.02, 3.0),
            (0.033, 3.0),
            (0.016, 0.1),
            (0.05, 0.0),
            (0.016, 4.0),
        ];
        let run = || {
            let mut s = AnimState::default();
            let mut poses = Vec::new();
            for (dt, spd) in history {
                s.advance(dt, Some(&gait), spd, AnimRate::default());
                poses.push(pose_for(&s, Some(&gait), &[&idle], AnimRate::default()));
            }
            poses
        };
        assert_eq!(run(), run());
    }

    /// **The binary switch is gone and nothing may replace it** (user call #1,
    /// `stubs.md` #42 — *"never re-introduce a discrete gait switch"*).
    ///
    /// The old test asserted a *state transition* at `WALK_SPEED_THRESHOLD_M_S`.
    /// The property that replaces it is the one the prohibition actually
    /// protects: **the pose is continuous in speed**. A discrete switch — a
    /// threshold, a crossfade, a gait-name lookup — necessarily puts a step in
    /// this sweep, and a step is what this bound catches.
    ///
    /// Scale-free, and derived rather than fitted: over a fine sweep the
    /// steepest term is `θmax(Fr)`, and `dθ/dv` is bounded on `[0, 4.5]` by the
    /// stride regression's own exponents, so a per-step joint move of more than
    /// a few times the sweep's own resolution can only be a discontinuity. Idle
    /// is checked as the ladder's LIMIT, not as a state: at v = 0 the pose is
    /// exactly the derived resting pose.
    #[test]
    fn the_gait_ladder_is_continuous_and_idle_is_its_limit() {
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let gait = gait_of(&plan);
            let phase = 0.37; // an arbitrary mid-cycle instant, held fixed
            let at_rest = pose_at(&gait, 0.0, phase);
            for (name, e) in &at_rest {
                for (i, a) in e.iter().enumerate() {
                    assert!(
                        a.abs() < 1e-12,
                        "{}: at v = 0 the pose must BE the derived rest, but {name}[{i}] \
                         = {a} rad — idle is the ladder's limit, not a state",
                        plan.name,
                    );
                }
            }
            let steps = 900;
            let v_max = 4.5; // the shipped top speed
            let mut prev = at_rest;
            let mut worst = 0.0_f64;
            for k in 1..=steps {
                let v = v_max * k as f64 / steps as f64;
                let now = pose_at(&gait, gait.froude(v), phase);
                for (name, e) in &now {
                    let p = prev.get(name).copied().unwrap_or([0.0; 3]);
                    for i in 0..3 {
                        worst = worst.max((e[i] - p[i]).abs());
                    }
                }
                prev = now;
            }
            // One sweep step is 5 mm/s of speed; the whole excursion across the
            // sweep is under ~1.6 rad, so a smooth ladder moves any joint by
            // ~1e-3 rad per step. 0.05 rad is 30x that and still 1/30 of any
            // switch's jump (the retired Loco crossfade moved the hip by 1.1 rad
            // across four blend steps).
            assert!(
                worst < 0.05,
                "{}: a {worst:.4} rad jump between adjacent speeds — that is a \
                 DISCRETE GAIT SWITCH, which user call #1 forbids by name",
                plan.name
            );
        }
    }

    /// Sample every gait-driven joint at one `(Fr, phase)`, as the renderer
    /// does. A test helper only — production goes through [`pose_for`], and
    /// this calls the same [`GaitVector::limb_pose`] it does.
    fn pose_at(gait: &GaitVector, froude: f64, phase: f64) -> HashMap<String, [f64; 3]> {
        let mut out = HashMap::new();
        for limb in &gait.limbs {
            for ja in gait.limb_pose(limb, froude, phase) {
                out.insert(ja.segment, ja.euler);
            }
        }
        out
    }

    /// **THE BEHAVIOURAL ACCEPTANCE, at the CONSUMER.** Slice one asserted the
    /// bake's arithmetic; this asserts that what the *renderer samples* carries
    /// it — three bodies of different proportions walking with derived cadence,
    /// stride and duty.
    ///
    /// Two invariants, both ratios, neither a snapshot:
    ///
    /// 1. **Cadence scales as `√(g/L)`.** `f·√L` is constant across the three
    ///    plans at equal Froude — the `Fr^0.2` term cancels — so a big animal
    ///    takes slow steps as *arithmetic*, not as an authored clip duration.
    ///    The retired clip was 1.000 s for every body regardless of its legs.
    /// 2. **Dynamic similarity.** At equal Fr the duty, the hip excursion and
    ///    the root-height excursion **as a fraction of leg length** are
    ///    identical across all three plans; only the metres differ (Alexander &
    ///    Jayes 1983, reproduced by our own arithmetic).
    #[test]
    fn three_bodies_walk_at_their_own_cadence() {
        let plans = [biped_plan(), stout_plan(), longleg_plan()];
        let fr = 0.25; // a comfortable walk, below the published transition
        let mut rows = Vec::new();
        println!("\nthe RENDERED gait at Fr = {fr} (g = {G} m/s²) — what pose_for samples");
        for plan in &plans {
            let gait = gait_of(plan);
            let at = gait.at(fr);
            // The root excursion as the RENDERER composes it: the peak-to-trough
            // of `root_offset_m` over one cycle, which is the only vertical
            // authority there is.
            let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
            for k in 0..720 {
                let phase = k as f64 / 720.0;
                let y = root_offset_m(Some(&gait), 0.0, fr, phase, Posture::Standing);
                lo = lo.min(y);
                hi = hi.max(y);
            }
            let excursion_m = hi - lo;
            println!(
                "  {:<16} L {:.3} m | v {:.4} m/s | cadence {:.4} Hz | period {:.4} s | \
                 duty {:.4} | θmax {:.4} rad | root excursion {:.4} m = {:.3} % of L",
                plan.name.trim_start_matches("dc:body/"),
                gait.governing_reach_m,
                at.speed_m_s,
                at.cadence_hz,
                at.period_s,
                at.mean_duty,
                at.theta_max_rad,
                excursion_m,
                100.0 * excursion_m / gait.governing_reach_m,
            );
            rows.push((
                plan.name.clone(),
                gait.governing_reach_m,
                at.cadence_hz,
                at.mean_duty,
                at.theta_max_rad,
                excursion_m / gait.governing_reach_m,
            ));
        }
        // (1) f·√L is invariant — i.e. f_i/f_j = √(L_j/L_i), exactly.
        for w in rows.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let ka = a.2 * a.1.sqrt();
            let kb = b.2 * b.1.sqrt();
            assert!(
                (ka - kb).abs() < 1e-12 * ka.abs().max(1.0),
                "{} and {}: f·√L = {ka} vs {kb} — cadence is not scaling as √(g/L)",
                a.0,
                b.0
            );
            let predicted = (b.1 / a.1).sqrt();
            let measured = a.2 / b.2;
            assert!(
                (predicted - measured).abs() < 1e-12,
                "{} / {}: cadence ratio {measured} against √(L/L) = {predicted}",
                a.0,
                b.0
            );
        }
        // (2) dynamic similarity: duty, excursion and bob-as-a-fraction are the
        // SAME number on every body at equal Fr.
        for w in rows.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            for (what, x, y) in [
                ("duty", a.3, b.3),
                ("θmax", a.4, b.4),
                ("root excursion / L", a.5, b.5),
            ] {
                assert!(
                    (x - y).abs() < 1e-12,
                    "{} vs {}: {what} differs ({x} vs {y}) — dynamic similarity broken",
                    a.0,
                    b.0
                );
            }
        }
        // …and the metres genuinely do differ, or the check above is vacuous.
        assert!(
            rows[0].1 != rows[1].1 && rows[1].1 != rows[2].1,
            "the three plans must have different leg lengths for this to mean anything"
        );
    }

    /// **Runtime is sacred** (CLAUDE.md § Conventions), and this slice moved a
    /// per-frame path, so it owes a measured cost. Both paths are timed here —
    /// the derived gait that ships, and `sample_clip` on the parked walk fixture
    /// it replaced — and the numbers print under `--nocapture`.
    ///
    /// **The assertion is a bound with a derivation, not a snapshot** (the
    /// timing of a number a colleague will improve tomorrow is exactly what
    /// § Gates forbids pinning): at 60 fps a thousand bodies must fit in a
    /// frame, so 16 µs per body per frame is already catastrophic and 20 µs is
    /// a ceiling nothing sane approaches. Measured cost sits several times
    /// below it, so this cannot flake on a loaded machine and still catches a
    /// pathology — an allocation in the inner loop, a table build per frame.
    ///
    /// **⚠ PER-CYCLE QUANTIZATION COST IT 22 % (2026-08-04, measured both sides
    /// on one machine in one sitting): 3312 ns before, 4036 ns after.** The
    /// per-bone rate is a real derivation — ownership resolved and blended per
    /// bone, then one sample per distinct rate — and it is not free. Two
    /// intermediate shapes were measured and rejected on this number: a
    /// `Vec` of owners per bone (**6445 ns**) and a `HashMap<&str, StepBlend>`
    /// (**4945 ns**); the shipped path holds ownership as short name lists and
    /// hashes nothing. **The named heir for the rest is hoisting the ownership
    /// tables into `character.rs`'s per-plan `BodyAssets`** — ownership is
    /// constant per (plan, clip set) and only `gait_step` moves with speed, so
    /// the whole pre-pass is per-plan work being redone per body per frame.
    /// Not done here: it changes `pose_for`'s signature, and this slice is an
    /// appearance change the user has yet to rule on.
    #[test]
    fn per_frame_pose_cost_is_measured() {
        use std::time::Instant;
        let gait = gait_of(&biped_plan());
        let idle = idle_clip();
        let walk = retired_biped_walk_clip();
        let rate = AnimRate::default();
        let mut state = AnimState::default();
        state.advance(rate.step_s(), Some(&gait), 1.47, rate);

        let n = 20_000;
        // Warm both paths so neither pays first-touch costs in its timed run.
        for _ in 0..2_000 {
            std::hint::black_box(pose_for(&state, Some(&gait), &[&idle], rate));
            std::hint::black_box(sample_clip(&walk, 0.3, rate));
        }
        let t0 = Instant::now();
        for i in 0..n {
            let mut s = state.clone();
            s.advance(1.0 / 60.0, Some(&gait), 1.47, rate);
            std::hint::black_box(pose_for(&s, Some(&gait), &[&idle], rate));
            std::hint::black_box(root_offset_m(
                Some(&gait),
                0.0,
                s.stepped_froude,
                s.stepped_phase(Some(&gait), rate),
                Posture::Standing,
            ));
            std::hint::black_box(i);
        }
        let derived_ns = t0.elapsed().as_nanos() as f64 / f64::from(n);
        let t1 = Instant::now();
        for i in 0..n {
            std::hint::black_box(sample_clip(&walk, f64::from(i) / 60.0, rate));
            std::hint::black_box(sample_clip(&idle, f64::from(i) / 60.0, rate));
        }
        let clip_ns = t1.elapsed().as_nanos() as f64 / f64::from(n);
        println!(
            "\nper-frame pose cost, one body: DERIVED gait + one clip layer + the \
             root offset = {derived_ns:.0} ns; the retired two-clip sample it \
             replaces = {clip_ns:.0} ns ({:.2}x)",
            derived_ns / clip_ns.max(1e-9)
        );
        assert!(
            derived_ns < 20_000.0,
            "a per-body per-frame pose costing {derived_ns:.0} ns cannot serve a \
             thousand bodies at 60 fps — something pathological is in the inner loop"
        );
    }

    /// **The durable value of this slice: the bob is structurally
    /// single-authored.** Corrections #80 was two *expressions* of one vertical
    /// composition — `character.rs` built the IK hip as
    /// `feet.y + hip_local[1] + root_delta − crouch` while the render root
    /// wrote `t.y += root_delta; t.y += (root_bob − crouch)` — and they drifted
    /// by exactly the bob.
    ///
    /// A value test cannot catch that class: two expressions agree until
    /// somebody edits one. So this reads the pose loop's **source** and asserts
    /// the shape:
    ///
    /// 1. there is exactly ONE call to [`root_offset_m`] in the whole file;
    /// 2. inside the per-frame pose loop, `root_delta_m` appears exactly once —
    ///    inside that call — and `CROUCH_ROOT_DROP_M` not at all (the crouch is
    ///    passed as the *posture* it is, so there is no second place to sink a
    ///    root);
    /// 3. both consumers read the same binding by name.
    ///
    /// It fails the moment anyone composes a second vertical anywhere in that
    /// loop, which is precisely the defect. *A source-text assertion is unusual
    /// and deliberate: the property being protected is textual — that two
    /// expressions do not exist — and no value check can express it.*
    #[test]
    fn the_render_root_and_the_ik_hip_read_one_root_offset() {
        let src = include_str!("character.rs");
        assert_eq!(
            src.matches("root_offset_m(").count(),
            1,
            "there must be exactly ONE vertical composition site in character.rs"
        );
        // The per-frame pose block, bounded by its own two anchors: from the
        // per-body match arm to the spawn arm that follows it. Deliberately
        // narrower than `sync_characters`, whose *setup* legitimately reports
        // the resting delta in a log line — the rule is about the per-frame
        // composition, not about never naming the delta.
        let loop_start = src
            .find("match bodies.get_mut(character.name.as_str())")
            .expect("the per-body pose block starts at the match");
        let loop_end = src[loop_start..]
            .find("None => to_spawn.push(")
            .map(|i| loop_start + i)
            .expect("the pose block ends at the spawn arm");
        let pose_loop = &src[loop_start..loop_end];
        assert_eq!(
            pose_loop.matches("root_delta_m").count(),
            1,
            "the resting delta may only be read INSIDE the one composition"
        );
        assert_eq!(
            pose_loop.matches("CROUCH_ROOT_DROP_M").count(),
            0,
            "the crouch sink is a POSTURE passed to the one composition, never a \
             term the pose loop adds for itself"
        );
        // Both consumers, by name, reading the same binding.
        assert!(
            pose_loop.contains("let hip_y = feet.y + leg.hip_local[1] + root_offset;"),
            "the IK hip must read the one root offset"
        );
        assert!(
            pose_loop.contains("t.y += root_offset as f32;"),
            "the render root must read the one root offset"
        );
    }

    /// The one composition's identity and its honest refusal, as values.
    ///
    /// - **At rest it is exactly the posture bake's delta** — adding a gait
    ///   changed nothing for a standing body, which is the S-5 identity the
    ///   consumer slice owes (`derived_hip_reaches_the_render_path` pins the
    ///   delta itself).
    /// - **Its excursion is downward only.** Midstance is the peak (`h = L`,
    ///   the resting height) and the contact extremes are the troughs, so a
    ///   walking body dips below its standing height and never rises above it.
    ///   That is the compass geometry, and it is why the derived gait cannot
    ///   lift a body off its own legs.
    /// - **A run DECLINES rather than fabricating a flight arc** (`stubs.md`
    ///   #39, heir B6): above the transition the offset holds still.
    #[test]
    fn root_offset_is_the_one_vertical_and_declines_a_run() {
        let gait = gait_of(&biped_plan());
        let rest = root_offset_m(Some(&gait), 0.25, 0.0, 0.0, Posture::Standing);
        assert!(
            (rest - 0.25).abs() < 1e-12,
            "at v = 0 the offset must be the resting delta alone, got {rest}"
        );
        // Walking: strictly at or below the resting height, and genuinely moving.
        let fr = 0.25;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for k in 0..720 {
            let y = root_offset_m(Some(&gait), 0.0, fr, k as f64 / 720.0, Posture::Standing);
            assert!(
                y <= 1e-12,
                "the root may dip below the standing height, never rise above it \
                 (got {y} m at phase {})",
                k as f64 / 720.0
            );
            lo = lo.min(y);
            hi = hi.max(y);
        }
        assert!(
            hi - lo > 1e-3,
            "a walking biped's root must actually move, got {:.6} m",
            hi - lo
        );
        // A run: the flight apex is ballistic and is DECLINED, so the offset is
        // flat rather than invented.
        let run_fr = gait.froude(4.5);
        assert!(run_fr > 0.5, "4.5 m/s is a run for this body (Fr {run_fr})");
        let a = root_offset_m(Some(&gait), 0.0, run_fr, 0.10, Posture::Standing);
        let b = root_offset_m(Some(&gait), 0.0, run_fr, 0.60, Posture::Standing);
        assert_eq!(
            a, b,
            "a declined root height must HOLD, not fabricate a flight arc"
        );
        // The identity fallback: no gait at all is the pre-gait behaviour.
        assert_eq!(
            root_offset_m(None, 0.25, 0.9, 0.4, Posture::Standing),
            0.25,
            "a plan with no gait renders exactly as it did before this slice"
        );
        assert_eq!(
            root_offset_m(None, 0.25, 0.9, 0.4, Posture::Crouching),
            0.25 - CROUCH_ROOT_DROP_M
        );
    }

    /// The gait owns bearing chains outright; a clip layers additively on
    /// everything else (design § 6 rules 1–4). Asserted as a *composition*
    /// property, not a snapshot: the arms carry gait + clip, the legs carry the
    /// gait alone even when a clip asks for them.
    #[test]
    fn a_clip_layers_on_non_bearing_segments_and_is_refused_on_a_leg() {
        let plan = biped_plan();
        let gait = gait_of(&plan);
        let mut clip = idle_clip();
        // Ask the clip for a LEG as well as its arms.
        clip.keyframes[0].rotations.push(dc_api::bodies::JointRot {
            segment: "leg_l_upper".into(),
            euler: [1.0, 0.0, 0.0],
        });
        let rate = AnimRate::default();
        let mut state = AnimState::default();
        state.advance(rate.step_s(), Some(&gait), 1.5, rate);
        let bare = pose_for(&state, Some(&gait), &[], rate);
        let layered = pose_for(&state, Some(&gait), &[&clip], rate);
        let arm = "arm_l_upper";
        assert!(
            (layered.joints[arm][2] - bare.joints[arm][2]).abs() > 1e-9,
            "the clip must ADD to a non-bearing segment over its counter-swing"
        );
        assert_eq!(
            layered.joints["leg_l_upper"], bare.joints["leg_l_upper"],
            "a clip may not contribute to a BEARING chain — that request is a \
             gait-type choice, not an animation"
        );
    }

    // --- per-cycle quantization (2026-08-04) -------------------------------

    /// The three shipped plans at the shipped top speed, as `(name, cadence,
    /// cycle, N)` — the table the ruling was taken on, **re-derived from the
    /// shipped bake rather than restated.**
    fn cadence_table(rate: AnimRate) -> Vec<(String, f64, f64, u32)> {
        [biped_plan(), stout_plan(), longleg_plan()]
            .into_iter()
            .map(|plan| {
                let gait = gait_of(&plan);
                let cadence = gait.cadence_hz(gait.froude(4.5));
                let cycle = 1.0 / cadence;
                let n = rate.cycle(cycle).expect("a walking body has a cycle");
                (plan.name.clone(), cadence, cycle, n.poses())
            })
            .collect()
    }

    /// **`N` is a whole number, and never below 2, for every shipped plan across
    /// the whole speed range** — the slice's arithmetic acceptance.
    ///
    /// The floor is the user's ruling and this sweep is where it bites: the stout
    /// is the smallest, fastest-cadence body we ship, and at high speed its cycle
    /// is short enough that a lower target would round it under 2.
    ///
    /// Scale-free: `N` is a per-cycle arithmetic on one duration, so a sweep of
    /// speeds and targets exercises the whole claim without a world.
    #[test]
    fn every_shipped_plan_gets_a_whole_number_of_poses_per_cycle() {
        println!("\nper-cycle quantization at the {DEFAULT_TARGET_FPS} fps default, v = 4.5 m/s:");
        for (name, cadence, cycle, n) in cadence_table(AnimRate::default()) {
            println!(
                "  {name:22} cadence {cadence:.3} Hz  cycle {cycle:.4} s  \
                 fractional {:.3}  ->  N = {n}",
                cycle * DEFAULT_TARGET_FPS
            );
        }
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let gait = gait_of(&plan);
            for fps in [1.0, 6.0, 12.0, 24.0, 60.0] {
                let rate = AnimRate::new(fps).expect("a legal target");
                for k in 1..=450 {
                    let v = 4.5 * f64::from(k) / 450.0;
                    let cadence = gait.cadence_hz(gait.froude(v));
                    let Some(grid) = rate.cycle(1.0 / cadence) else {
                        continue; // a cycle too long to divide — the dissolve limit
                    };
                    assert!(
                        grid.poses() >= MIN_POSES_PER_CYCLE,
                        "{} at {v:.2} m/s, {fps} fps: N = {} is below the floor",
                        plan.name,
                        grid.poses()
                    );
                    // The formula, restated as the property: N is the rounded
                    // product OR the floor, never anything else.
                    let want = (1.0 / cadence * fps).round().max(f64::from(MIN_POSES_PER_CYCLE));
                    assert!(
                        (f64::from(grid.poses()) - want).abs() < 1e-9,
                        "{} at {v:.2} m/s, {fps} fps: N = {} against round(T·fps) = {want}",
                        plan.name,
                        grid.poses()
                    );
                }
            }
        }
    }

    /// **A cycle faster than the floor clamps rather than degenerating** — the
    /// Nyquist edge, with the stout as the real body closest to it.
    #[test]
    fn a_cycle_faster_than_the_floor_clamps() {
        let gait = gait_of(&stout_plan());
        let cadence = gait.cadence_hz(gait.froude(4.5));
        // At a 2 fps target the stout's 0.357 s cycle wants round(0.71) = 1, and
        // below 1.4 fps it wants 0. Both are held at 2.
        for fps in [0.5, 1.0, 1.4, 2.0, 2.8] {
            let rate = AnimRate::new(fps).expect("a legal target");
            let grid = rate.cycle(1.0 / cadence).expect("a cycle");
            assert_eq!(
                grid.poses(),
                MIN_POSES_PER_CYCLE,
                "at {fps} fps the stout's cycle must clamp, not degenerate"
            );
            // A clamped grid is still a grid: two distinct held phases, not one.
            let phases: std::collections::BTreeSet<u64> = (0..1_000)
                .map(|k| grid.phases().quantize(f64::from(k) / 1_000.0).to_bits())
                .collect();
            assert_eq!(phases.len(), 2);
        }
    }

    /// **THE BEHAVIOURAL ACCEPTANCE: a body lands on the same `N` phases every
    /// stride, and the retired grid did not.**
    ///
    /// `N` being an integer is arithmetic; *this* is the thing the user saw. The
    /// old scheme held the pose on a world-global `1/12 s` grid, so the phases it
    /// sampled walked around the cycle — 6.977 per cycle for the biped, 4.286 for
    /// the stout — and the stout's beat was visible as a hitch. Under a per-cycle
    /// grid the sampled phases are exactly `{0/N, 1/N, …, (N−1)/N}`, in every
    /// stride, at any speed.
    ///
    /// The retired scheme is reproduced here (test-side only, like
    /// `legacy_solve_leg_ik`) so the contrast is measured rather than asserted.
    #[test]
    fn a_body_lands_on_the_same_n_phases_every_stride() {
        let rate = AnimRate::default();
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let gait = gait_of(&plan);
            let speed = 4.5;
            let cadence = gait.cadence_hz(gait.froude(speed));
            let n = rate.cycle(1.0 / cadence).expect("a walking body").poses();

            let mut state = AnimState::default();
            let mut sampled: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
            let mut legacy: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
            let dt = 1.0 / 60.0;
            let frames = (20.0 / (cadence * dt)) as usize; // ~20 strides
            for _ in 0..frames {
                state.advance(dt, Some(&gait), speed, rate);
                sampled.insert(state.stepped_phase(Some(&gait), rate).to_bits());
                // The RETIRED scheme: hold the phase on a world-global 1/12 s
                // grid, anchored at absolute t = 0.
                let t_held = (state.clock_s * DEFAULT_TARGET_FPS).floor() / DEFAULT_TARGET_FPS;
                legacy.insert(((cadence * t_held).rem_euclid(1.0) * 1e9).round().to_bits());
            }
            let want: std::collections::BTreeSet<u64> = (0..n)
                .map(|k| (f64::from(k) / f64::from(n)).to_bits())
                .collect();
            assert_eq!(
                sampled, want,
                "{}: {} strides sampled {} distinct phases, not its own N = {n}",
                plan.name,
                20,
                sampled.len()
            );
            println!(
                "  {:22} N = {n} phases per stride, held; the retired 1/12 s grid \
                 wandered over {} distinct phases in 20 strides",
                plan.name,
                legacy.len()
            );
            assert!(
                legacy.len() > sampled.len(),
                "{}: the retired grid must be the one that wanders — if it does \
                 not, this test is measuring nothing",
                plan.name
            );
        }
    }

    /// **The per-bone rate, as behaviour.** A bearing chain has one owner (the
    /// gait refuses clip contributions on it), so it sits on the gait's whole
    /// `N`. An arm is owned by the gait's counter-swing **and** the idle clip, so
    /// it sits on the mean of the two time steps — a different, fractional grid.
    ///
    /// Asserted as a *count of held poses per stride*, which is the quantity the
    /// ruling is about, rather than as an angle nobody can read.
    #[test]
    fn a_shared_bone_rides_a_blended_step_and_an_owned_bone_does_not() {
        let rate = AnimRate::default();
        let plan = biped_plan();
        let gait = gait_of(&plan);
        let idle = idle_clip();
        let speed = 4.5;
        let cadence = gait.cadence_hz(gait.froude(speed));
        let n = rate.cycle(1.0 / cadence).expect("a walking body").poses();

        let mut state = AnimState::default();
        let dt = 1.0 / 240.0; // finer than any grid in play, so nothing is missed
        let frames = (6.0 / (cadence * dt)) as usize; // six strides
        let mut leg: Vec<u64> = Vec::new();
        let mut arm: Vec<u64> = Vec::new();
        for _ in 0..frames {
            state.advance(dt, Some(&gait), speed, rate);
            let pose = pose_for(&state, Some(&gait), &[&idle], rate);
            leg.push(pose.joints["leg_l_upper"][0].to_bits());
            arm.push(pose.joints["arm_l_upper"][2].to_bits());
        }
        let holds = |v: &[u64]| v.windows(2).filter(|w| w[0] != w[1]).count();
        // Six strides on a whole N: the leg changes value exactly 6·N times
        // (every bucket boundary), because the gait alone owns it.
        let leg_changes = holds(&leg);
        assert!(
            leg_changes >= 6 * n as usize - 1 && leg_changes <= 6 * n as usize + 1,
            "the leg changed {leg_changes} times over six strides, not the 6·N = {} \
             a single-owner bearing chain must",
            6 * n as usize
        );
        // The arm is shared, so its grid is the blend and its change count is
        // NOT the gait's N — the idle clip's own step (2.0 s / 24) pulls it.
        let arm_changes = holds(&arm);
        let gait_step = rate.cycle(1.0 / cadence).expect("a cycle").step_s();
        let clip_step = rate.cycle(idle.duration_s).expect("the idle").step_s();
        let blended = blended_step_s(&[(gait_step, 1.0), (clip_step, 1.0)]).expect("two owners");
        println!(
            "  per-bone: leg Δt {gait_step:.5} s (N = {n}, sole owner) · \
             arm Δt {blended:.5} s (gait + idle) · clip Δt {clip_step:.5} s"
        );
        assert!(
            (blended - gait_step).abs() > 1e-9,
            "this test is vacuous unless the two owners actually differ"
        );
        assert!(
            arm_changes != leg_changes,
            "a shared bone must not ride its co-owner's whole N ({arm_changes} \
             changes against the leg's {leg_changes})"
        );
        // …and the blend is a compromise, not a takeover: the arm's rate sits
        // between the two owners', because that is what a mean is.
        let lo = gait_step.min(clip_step);
        let hi = gait_step.max(clip_step);
        assert!(lo <= blended && blended <= hi);
    }

    /// **THE FIREWALL GUARD, and the answer to the slice's open call.**
    ///
    /// Quantization must never feed back into anything **sim-visible**: if B5's
    /// damage resolution ever reads a pose it must read the sim's *unquantized*
    /// target, or two clients at different targets would resolve hits
    /// differently (`bodies.md` § Stepped animation, OPEN; § THE SIM OWNS THE
    /// TARGET).
    ///
    /// **No such path exists today**, and this pins the absence in both
    /// directions:
    ///
    /// 1. **Upstream** — the state a sim-visible consumer would read (`phase`,
    ///    `froude`, `clock_s`, `trunk_yaw`) is **bit-identical at any target**.
    ///    The rate reaches the *hold* and nothing else, so no future consumer of
    ///    the unquantized target can inherit a client's setting by accident.
    /// 2. **Downstream** — `dc_api::CharacterState` is a body's entire
    ///    sim-visible surface, and it carries no pose, phase or frame. **When
    ///    that stops being true this test fails**, which is the moment the guard
    ///    has to become a real one rather than a pin.
    #[test]
    fn the_target_fps_never_reaches_a_sim_visible_quantity() {
        let gait = gait_of(&biped_plan());
        let history = [
            (0.016, 0.0),
            (0.016, 0.0),
            (0.02, 2.0),
            (0.02, 3.0),
            (0.033, 3.0),
            (0.016, 0.1),
            (0.05, 0.0),
            (0.016, 4.5),
            (0.016, 4.5),
        ];
        let run = |fps: f64| {
            let rate = AnimRate::new(fps).expect("a legal target");
            let mut s = AnimState::default();
            let mut targets = Vec::new();
            for (dt, spd) in history {
                s.advance(dt, Some(&gait), spd, rate);
                s.steer(dt, 0.7);
                targets.push((
                    s.clock_s.to_bits(),
                    s.phase.to_bits(),
                    s.froude.to_bits(),
                    s.trunk_yaw.to_bits(),
                ));
            }
            targets
        };
        let reference = run(12.0);
        for fps in [1.0, 6.0, 12.0, 30.0, 144.0, 1000.0] {
            assert_eq!(
                run(fps),
                reference,
                "at {fps} fps the UNQUANTIZED target moved — a client setting \
                 has reached a quantity the sim would resolve against"
            );
        }
        // …and the quantized view really does differ, or the check above is
        // vacuous: the whole point is that only the *approach* moves.
        let held = |fps: f64| {
            let rate = AnimRate::new(fps).expect("a legal target");
            let mut s = AnimState::default();
            for (dt, spd) in history {
                s.advance(dt, Some(&gait), spd, rate);
            }
            s.stepped_phase(Some(&gait), rate)
        };
        assert!(
            (held(6.0) - held(60.0)).abs() > 1e-9,
            "two targets must render differently, or nothing is being quantized"
        );

        // The downstream pin. A body's sim-visible surface is CharacterState;
        // when a pose, a phase or a frame index appears on it, the sim has a
        // pose consumer and this slice's guard is owed for real. Read off the
        // WIRE field names, because that is the surface a replay and a second
        // client both see.
        let state = dc_api::CharacterState::new("pin", dc_api::Vec3f::new(0.0, 0.0, 0.0));
        let sim_visible = serde_json::to_value(&state).expect("CharacterState serializes");
        let fields: Vec<&str> = sim_visible
            .as_object()
            .expect("a struct serializes to an object")
            .keys()
            .map(String::as_str)
            .collect();
        for forbidden in ["pose", "phase", "frame", "stepped", "fps"] {
            assert!(
                !fields.iter().any(|f| f.contains(forbidden)),
                "`{forbidden}` appeared on CharacterState — the sim can now see a \
                 pose. Make it read the UNQUANTIZED target (AnimState::phase), \
                 never AnimState::stepped_phase, and turn this pin into a guard."
            );
        }
    }

    // --- step 3: trunk/look split, two-bone leg IK ------------------------

    /// Forward kinematics of a two-bone leg in the sagittal (y, z) plane, for
    /// checking the solver: rest pose points straight down, `upper_x` at the
    /// hip, `lower_x` at the knee (relative to the upper). This used to be a
    /// second copy of the arithmetic; it now delegates to the production
    /// [`fk_foot_local`] so the check and the renderer cannot disagree.
    fn fk_foot(l1: f64, l2: f64, upper_x: f64, lower_x: f64) -> (f64, f64) {
        fk_foot_local(l1, l2, upper_x, lower_x)
    }

    /// A rig for the classic biped bone lengths with **no declared limits and
    /// no derivable ones** — `d_min` collapses to `|l1 − l2|` and the solver is
    /// exactly the pre-B7 closed form. The tests that predate B7 use this, so
    /// what they assert is still what they asserted.
    fn bare_rig(l1: f64, l2: f64) -> LegRig {
        LegRig {
            upper: "leg_l_upper".into(),
            lower: "leg_l_lower".into(),
            hip_local: [0.0, 0.9, 0.0],
            l1,
            l2,
            limits: LegLimits {
                knee_sagittal: true,
                knee_min: None,
                knee_max: None,
                hip_sagittal: true,
                hip_min: None,
                hip_max: None,
                d_min: (l1 - l2).abs(),
            },
        }
    }

    /// The pre-B7 solver, verbatim, as the byte-identity reference for
    /// `nothing_declared_leaves_the_solver_bit_identical`. A retired
    /// implementation kept **test-side only** so an identity claim has
    /// something to be identical TO — it is not a second authority and nothing
    /// but that one test may call it.
    fn legacy_solve_leg_ik(l1: f64, l2: f64, target: [f64; 3]) -> (f64, f64) {
        let (ty, tz) = (target[1], target[2]);
        let max = l1 + l2;
        let min = (l1 - l2).abs();
        let eps = 1e-6;
        let d_planar = (ty * ty + tz * tz).sqrt();
        let d = d_planar.clamp(min + eps, max - eps);
        let (uy, uz) = if d_planar > 1e-9 {
            (ty / d_planar, tz / d_planar)
        } else {
            (-1.0, 0.0)
        };
        let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
        let sin_a = (1.0 - cos_a * cos_a).max(0.0).sqrt();
        let knee = |pole: f64| {
            let ky = l1 * (cos_a * uy + pole * sin_a * (-uz));
            let kz = l1 * (cos_a * uz + pole * sin_a * uy);
            (ky, kz)
        };
        let (ka, kb) = (knee(1.0), knee(-1.0));
        let (ky, kz) = if ka.1 <= kb.1 { ka } else { kb };
        let upper_x = (-kz).atan2(-ky);
        let (fy, fz) = (uy * d, uz * d);
        let total = (-(fz - kz)).atan2(-(fy - ky));
        (upper_x, wrap_pi(total - upper_x))
    }

    #[test]
    fn ik_reconstructs_reachable_targets_with_a_forward_knee() {
        let (l1, l2) = (0.45, 0.43);
        let rig = bare_rig(l1, l2);
        for target in [
            [0.0, -0.8, 0.0],
            [0.0, -0.6, -0.3],
            [0.0, -0.7, 0.15],
            [0.05, -0.75, -0.1],
        ] {
            let ik = solve_leg_ik(&rig, target);
            assert_eq!(ik.reach, Reach::Ok, "{target:?} is within reach");
            let (fy, fz) = fk_foot(l1, l2, ik.upper_x, ik.lower_x);
            assert!(
                (fy - target[1]).abs() < 1e-6 && (fz - target[2]).abs() < 1e-6,
                "fk ({fy}, {fz}) != target {target:?}"
            );
            // Knee bends forward: its z is at or forward of straight-down.
            let kz = -l1 * ik.upper_x.sin();
            assert!(kz <= 1e-9, "knee not forward (z={kz})");
        }
    }

    #[test]
    fn ik_clamps_reach_and_never_nans() {
        let (l1, l2) = (0.45, 0.43);
        let rig = bare_rig(l1, l2);
        // Beyond reach: flagged, still finite, near full extension.
        let ik = solve_leg_ik(&rig, [0.0, -2.0, 0.0]);
        assert_eq!(ik.reach, Reach::BeyondExtension);
        assert!(ik.upper_x.is_finite() && ik.lower_x.is_finite());
        let (fy, _) = fk_foot(l1, l2, ik.upper_x, ik.lower_x);
        assert!(
            (fy.abs() - (l1 + l2)).abs() < 0.01,
            "clamped to near full extension, got {fy}"
        );
        // Degenerate: target at the hip. No NaN.
        let ik = solve_leg_ik(&rig, [0.0, 0.0, 0.0]);
        assert!(ik.upper_x.is_finite() && ik.lower_x.is_finite());
    }

    // ------------------------------------------------------- B7, tests 1b/10/11 --

    /// **The S-5 acceptance, IK half** (audit § 8 test 1). Across the measured
    /// sweep, the B7 solver is **bit-identical** to the pre-B7 closed form for
    /// every target the pre-B7 solver could honestly serve — that is, every
    /// target OUTSIDE the derived `d_min`. Inside it the two disagree by
    /// construction, and the test enumerates that set rather than hiding it:
    /// it is exactly the degeneracy § 5.3 retires.
    #[test]
    fn nothing_declared_leaves_the_solver_bit_identical() {
        let mut compared = 0_usize;
        let mut refused = Vec::new();
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            for rig in leg_rigs(&plan) {
                let reach = rig.l1 + rig.l2;
                // A sweep over the sagittal plane the renderer actually asks
                // about: distances from a hair off the hip out past full reach.
                for i in 0..=40 {
                    for j in -6..=6 {
                        let d = reach * f64::from(i) / 40.0;
                        let a = f64::from(j) * 0.2;
                        let t = [0.0, -d * a.cos(), -d * a.sin()];
                        let ik = solve_leg_ik(&rig, t);
                        let (uy, ly) = legacy_solve_leg_ik(rig.l1, rig.l2, t);
                        let d_planar = (t[1] * t[1] + t[2] * t[2]).sqrt();
                        if d_planar < rig.limits.d_min - 1e-9 {
                            refused.push((plan.name.clone(), d_planar, rig.limits.d_min));
                            continue;
                        }
                        assert_eq!(
                            (ik.upper_x.to_bits(), ik.lower_x.to_bits()),
                            (uy.to_bits(), ly.to_bits()),
                            "plan `{}` target {t:?}: B7 must be BIT-identical outside d_min",
                            plan.name
                        );
                        compared += 1;
                    }
                }
            }
        }
        assert!(compared > 2000, "the sweep must be a sweep, got {compared}");
        assert!(
            !refused.is_empty(),
            "the inner-sector set must be non-empty, or this test proves nothing \
             about the mechanism being armed (A-3)"
        );
        println!(
            "\nB7 IK identity: {compared} targets bit-identical; {} inside d_min \
             (refused, not clamped)",
            refused.len()
        );
    }

    /// **Audit § 8 test 10.** The reachable set is the annulus **sector**: the
    /// measured `d_min` per plan, and the stout's degenerate crouch honestly
    /// refused instead of folding a knee to 180° to chase a target below the
    /// ground (journal/0131).
    #[test]
    fn the_ik_reachable_set_is_the_sector_not_the_annulus() {
        println!(
            "\n§ 5.3 d_min MEASURED   {:>8} {:>8} {:>12} {:>12} {:>8}",
            "l1", "l2", "|l1-l2|", "d_min", "ratio"
        );
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let rig = leg_rigs(&plan).remove(0);
            let annulus = (rig.l1 - rig.l2).abs();
            println!(
                "{:<22} {:>8.3} {:>8.3} {:>12.5} {:>12.5} {:>8.1}x",
                plan.name,
                rig.l1,
                rig.l2,
                annulus,
                rig.limits.d_min,
                rig.limits.d_min / annulus
            );
            assert!(
                rig.limits.d_min > annulus,
                "plan `{}`: the sector's inner bound must be HONEST, not numerical",
                plan.name
            );
            // The inner bound is the geometry, not a number: at full flexion the
            // foot is exactly d_min from the hip, by the law of cosines.
            let f_max = rig.limits.knee_min.expect("a derived fold magnitude").abs();
            let (fy, fz) = fk_foot_local(rig.l1, rig.l2, 0.0, -f_max);
            assert!(
                ((fy * fy + fz * fz).sqrt() - rig.limits.d_min).abs() < 1e-9,
                "d_min must BE the fully-folded foot distance"
            );
        }
        // journal/0131's degenerate stout crouch: the sole target sits BELOW
        // the ground (hip 0.440 − 0.45 drop = −0.010 m), so the planar distance
        // is 0.010 against a 0.2366 m inner bound. Today: clamped, knee folded
        // flat, ~10 mm residual, nobody told. Under B7: refused, and it says so.
        let stout = leg_rigs(&stout_plan()).remove(0);
        let ik = solve_leg_ik(&stout, [0.0, -0.010, 0.0]);
        assert_eq!(
            ik.reach,
            Reach::BeyondFlexion {
                joint: stout.lower.clone()
            },
            "the stout's crouch target is far inside d_min = {:.5}",
            stout.limits.d_min
        );
    }

    /// **Audit § 8 test 11.** The knee pole is a READ of declared data: flip the
    /// declared sign and the solved knee flips with it, with no code path
    /// selecting a side. (With nothing declared the range is symmetric, both
    /// poles are admissible, and the pre-B7 forward convention survives as the
    /// tie-break — the STAND-IN on `solve_leg_ik`, heir `stubs.md` B7-a.)
    #[test]
    fn the_knee_pole_comes_from_the_declaration() {
        let base = leg_rigs(&longleg_plan()).remove(0);
        let target = [0.0, -0.80, 0.0]; // well inside the annulus: knee must bend
        let mut back = base.clone();
        back.limits.knee_max = Some(0.0); // folds one way (a knee)
        back.limits.knee_min = base.limits.knee_min;
        let mut fore = base.clone();
        fore.limits.knee_min = Some(0.0); // folds the other (an elbow)
        fore.limits.knee_max = base.limits.knee_min.map(f64::abs);

        let a = solve_leg_ik(&back, target);
        let b = solve_leg_ik(&fore, target);
        assert_eq!(a.reach, Reach::Ok);
        assert_eq!(b.reach, Reach::Ok);
        assert!(
            a.lower_x < -1e-3 && b.lower_x > 1e-3,
            "the declared sign must decide the fold: {} vs {}",
            a.lower_x,
            b.lower_x
        );
        assert!(
            (a.lower_x + b.lower_x).abs() < 1e-9,
            "the two poles are mirror images: {} vs {}",
            a.lower_x,
            b.lower_x
        );
        // Both solutions still put the foot on the target — the pole chooses
        // WHICH WAY, never WHERE.
        for ik in [&a, &b] {
            let (fy, fz) = fk_foot_local(base.l1, base.l2, ik.upper_x, ik.lower_x);
            assert!((fy - target[1]).abs() < 1e-6 && (fz - target[2]).abs() < 1e-6);
        }
    }

    /// The cervical migration (§ 5.4) is byte-identical: the two constants that
    /// left `body.rs` are the two numbers the plans now declare, and
    /// `resolve_orientation` reads them rather than restating them.
    #[test]
    fn the_cervical_range_is_declared_and_unchanged() {
        let yaw = 75.0_f64.to_radians();
        let pitch = 45.0_f64.to_radians();
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let c = Cervical::of(&plan);
            assert_eq!(
                (c.yaw_max, c.pitch_min, c.pitch_max),
                (Some(yaw), Some(-pitch), Some(pitch)),
                "plan `{}` must declare the migrated cervical range exactly",
                plan.name
            );
        }
        // A plan with no `look` role gets an unclamped neck — the same
        // feature-off a missing name has always meant.
        let mut faceless = biped_plan();
        for s in &mut faceless.segments {
            s.roles.retain(|r| r.role != "look");
            s.dofs = None;
        }
        let c = Cervical::of(&faceless);
        assert_eq!(c, Cervical::default());
        let o = resolve_orientation(0.0, 3.0, -1.4, c);
        assert!(
            (o.neck_yaw - 3.0).abs() < 1e-12 && (o.neck_pitch + 1.4).abs() < 1e-12,
            "an undeclared neck is unclamped, got {o:?}"
        );
    }

    #[test]
    fn ik_is_deterministic_for_fixed_inputs() {
        let t = [0.03, -0.72, -0.15];
        let rig = leg_rigs(&biped_plan()).remove(0);
        assert_eq!(solve_leg_ik(&rig, t), solve_leg_ik(&rig, t));
    }

    /// The **approach** half of the facing split (user call #5). It chases a
    /// target and nothing else: the speed test it used to carry moved to the
    /// sim, which simply stops moving the target when the body stops. That
    /// removal is the point — one derivation of the facing, not one per side.
    #[test]
    fn the_trunk_chases_the_sims_target_and_holds_when_it_stops_moving() {
        // A fixed target history replays identically.
        // The SIM's convention, read from the sim — not restated here.
        let facing_x = dc_api::character::yaw_from_travel(1.0, 0.0).unwrap();
        let facing_z = dc_api::character::yaw_from_travel(0.0, -1.0).unwrap();
        let hist = [
            (0.016, facing_x),
            (0.02, facing_x),
            (0.02, facing_z),
            (0.05, facing_z),
            (0.016, -facing_x),
        ];
        let run = || {
            let mut s = AnimState::default();
            let mut ys = Vec::new();
            for (dt, target) in hist {
                s.steer(dt, target);
                ys.push(s.trunk_yaw);
            }
            ys
        };
        assert_eq!(run(), run(), "fixed steer history replays identically");

        // Chasing the +X target settles the trunk facing +X (yaw −π/2).
        let mut s = AnimState::default();
        for _ in 0..60 {
            s.steer(0.05, facing_x);
        }
        assert!(
            wrap_pi(s.trunk_yaw - (-std::f64::consts::FRAC_PI_2)).abs() < 1e-3,
            "trunk reaches the sim's target, got {}",
            s.trunk_yaw
        );
        // The sim holds its target when the body stops; the trunk converges on
        // it and stays. Asserted as CONVERGENCE, not equality: the approach is
        // exponential, so more frames against an unchanged target can only
        // close the residual further — never open it, and never drift off.
        // *(This assertion said "unchanged" and the gate caught it: an
        // asymptotic chase never lands exactly, and demanding that it does is
        // asserting a snapshot of the easing curve rather than the property.)*
        let residual_before = wrap_pi(s.trunk_yaw - facing_x).abs();
        for _ in 0..5 {
            s.steer(0.1, facing_x);
        }
        let residual_after = wrap_pi(s.trunk_yaw - facing_x).abs();
        assert!(
            residual_after <= residual_before && residual_after < 1e-3,
            "an unchanged target must close the facing, not move it: \
             {residual_before} -> {residual_after} rad"
        );
        // It approaches over TRUNK_TURN_WINDOW_S rather than snapping — the
        // half that is genuinely the client's and genuinely free to tune.
        let mut s = AnimState::default();
        s.steer(TRUNK_TURN_WINDOW_S / 4.0, facing_x);
        assert!(
            s.trunk_yaw.abs() > 1e-9 && wrap_pi(s.trunk_yaw - facing_x).abs() > 1e-3,
            "a partial frame must move partway, got {}",
            s.trunk_yaw
        );
    }

    // --- the body experiments: does one clip set retarget, and does the foot
    // --- IK ever actually solve? -----------------------------------------

    /// The N=2 boot scale's voxel edge (`PLAYER_HEIGHT_M / 2`) — the scale the
    /// game actually launches at, and therefore the scale whose **half-voxel**
    /// foot-IK window is the live one.
    fn boot_voxel_size_m() -> f64 {
        crate::PLAYER_HEIGHT_M / 2.0
    }

    /// Print one report's **per-frame time series** — the deliverable, not a
    /// min/max range.
    ///
    /// corrections #76: *a still frame is structurally blind to a temporal
    /// artifact*, and a range like `[+0.020, +0.035]` is an **amplitude**, not an
    /// error bar. A reader has to be able to see the oscillation, or its absence,
    /// in the numbers — so every frame gets a line and both feet are on it.
    fn print_series(r: &RetargetReport) {
        println!(
            "\n  -- {} / {} / {} / {} -- hip authored {:.3} {:+.3} bake = {:.3}, \
             - drop {:.3} = effective {:.3} | reach {:.3} (slack {:+.3})",
            r.plan.trim_start_matches("dc:body/"),
            r.clip.trim_start_matches("dc:anim/biped_"),
            r.ground,
            if r.root_drop_m > 0.0 {
                "crouch"
            } else {
                "stand"
            },
            r.hip_m,
            r.root_delta_m,
            r.hip_derived_m(),
            r.root_drop_m,
            r.hip_eff_m(),
            r.reach_m,
            r.reach_m - r.hip_eff_m(),
        );
        let mut head = format!("     {:>2} {:>6}", "f", "t(s)");
        for l in &r.frames[0].legs {
            head.push_str(&format!(
                " | {:<7} {:>7} {:>7} {:>6} {:>6} {:>4}",
                l.leg, "gnd", "sole", "gap", "hip", "knee"
            ));
        }
        println!("{head} (deg; v = verdict)");
        for row in &r.frames {
            let mut line = format!("     {:>2} {:>6.3}", row.index, row.t_s);
            for l in &row.legs {
                line.push_str(&format!(
                    " | {:>+7.3} {:>+7.3} {:>+7.3} {:>+6.1} {:>+6.1} {:>4}",
                    l.ground_m,
                    l.rendered_sole_m,
                    l.gap_m,
                    l.hip_deg,
                    l.knee_deg,
                    l.verdict.code()
                ));
            }
            println!("{line}");
        }
        println!(
            "     = {} samples: seated {} + corrected {} + clamped {} + refused {} \
             | knee|max| {:.1} deg (solver {:.1}) | residual<= {:.9} m | clip sole \
             [{:+.3},{:+.3}] rendered [{:+.3},{:+.3}]",
            r.samples,
            r.already_seated,
            r.corrected,
            r.clamped_beyond_reach,
            r.refused,
            r.knee_bend_max_deg,
            r.solver_knee_bend_max_deg,
            r.reachable_residual_max_m,
            r.clip_sole_min_m,
            r.clip_sole_max_m,
            r.rendered_sole_min_m,
            r.rendered_sole_max_m,
        );
    }

    /// **The measurement.** Prints the full per-frame series for every
    /// (plan × ground × posture × clip)
    /// (`cargo test -p dc-client --release foot_placement -- --nocapture`) and
    /// asserts only what must hold **by derivation**:
    ///
    /// 1. the itemisation closes on its own total, at both levels
    ///    (`seated + corrected + clamped + refused == samples`, and
    ///    `corrected + clamped == inside-window`), and the series holds every
    ///    sample the summary counted;
    /// 2. the glue never produces a non-finite pose for any plan;
    /// 3. the rig the renderer derives really does track the plan's proportions
    ///    (stout reach ≈ half the biped's) — otherwise the experiment is not
    ///    measuring what it claims;
    /// 4. where the IK both ran and could reach, the residual is bounded by the
    ///    solver's **own annulus clamps** — a *derived* bound that has moved twice:
    ///    2026-08-01 the rotation quantizer went (from `reach × 11.25°` ~172 mm to
    ///    the outer clamp's ~1 µm), and 2026-08-02 the derived hip let the crouch
    ///    drop sink a hip to the ground, engaging the INNER clamp (`|l1 − l2| +
    ///    1e-6` — ~10 mm on the stout). Still the assertion that would catch a
    ///    quantizer reintroduction (~172 mm ≫ 21 mm);
    /// 5. **the sign law**, against the **effective** hip
    ///    (`hip + bake delta − crouch drop`), in THREE regimes since the derived
    ///    hip landed (2026-08-02): at **zero slack** (standing — the derived hip
    ///    equals reach exactly) every attempted target is pushed past the annulus
    ///    by the clip's non-negative bob or stride, so nothing corrects, knees
    ///    stay straight, and idle's rest frames SIT on the ground; at **negative
    ///    slack** (the pinned-hip world's standing regime, journal/0130 — now only
    ///    the identity fallback) nothing can correct; at **positive slack**
    ///    (crouching) where the window let the solver run, it must solve and must
    ///    bend a knee. Both gates stay load-bearing: under the pinned hip,
    ///    `longleg` crouching passed the annulus and was refused by the window;
    /// 6. **the one-voxel-step law**: the correction window is half a voxel, so a
    ///    step of one whole voxel — the smallest relief real terrain can have at
    ///    *any* scale N — can never fit inside it. Every raised-foot sample is
    ///    refused, and that 2:1 ratio is by construction, independent of N.
    ///
    /// Scale-free: every quantity is a per-frame, per-leg arithmetic on the plan's
    /// own lengths, so one clip loop at the default target exercises the whole
    /// invariant.
    #[test]
    fn foot_placement_and_retargeting_are_measured() {
        // The shipped clips plus the PARKED walk fixture. The fixture is here
        // because this probe's history is written against it and its rows are
        // the comparison a reader of journal/0130–0132 expects — not because
        // the derived gait is being checked against an animator (user call #3
        // is explicit that it is parked, not enshrined).
        let mut clips = biped_clips();
        clips.push(retired_biped_walk_clip());
        let plans = [biped_plan(), stout_plan(), longleg_plan()];
        let vs = boot_voxel_size_m();
        let half_voxel = vs * 0.5;
        // Four cases. Flat/standing is the degenerate one journal/0130 measured
        // (both feet want the same answer). Flat/CROUCHING is the same ground with
        // the root sunk `CROUCH_ROOT_DROP_M`, which moves the hip *toward* the
        // ground and is therefore the one posture where the stock biped's target is
        // reachable. The synthetic 0.30 m step is inside the window and asks whether
        // foot placement engages on uneven ground; the one-voxel step is the
        // smallest offset real terrain can produce.
        let cases = [
            (GroundCase::Flat, 0.0),
            (GroundCase::Flat, CROUCH_ROOT_DROP_M),
            (GroundCase::Step { rise_m: 0.30 }, 0.0),
            (GroundCase::Step { rise_m: vs }, 0.0),
        ];
        println!(
            "\nfoot-placement + retargeting report — N=2 (voxel {vs:.3} m, IK window \
             +/-{half_voxel:.3} m). Cases: flat standing; flat CROUCHING (root sunk \
             {CROUCH_ROOT_DROP_M:.3} m); a SYNTHETIC +0.300 m step (inside the \
             window, unrealizable at N=2); one whole voxel +{vs:.3} m (the smallest \
             REAL step)."
        );
        let mut reach = Vec::new();
        for plan in &plans {
            // The rig's bone lengths, for the assertion-(4) inner-annulus bound
            // (same source as the report's own reach_m).
            let legs_l1_l2 = {
                let rig = leg_rigs(plan);
                (rig[0].l1, rig[0].l2)
            };
            for (ground, drop) in cases {
                for clip in &clips {
                    let r = retarget_report(plan, clip, vs, ground, drop)
                        .expect("every plan here has legs");
                    print_series(&r);

                    // (1) the itemisation closes on its own total, both levels.
                    assert_eq!(
                        r.already_seated + r.corrected + r.clamped_beyond_reach + r.refused,
                        r.samples,
                        "{} / {} / {}: the placement itemisation must equal its own total",
                        r.plan,
                        r.clip,
                        r.ground
                    );
                    assert_eq!(
                        r.corrected + r.clamped_beyond_reach,
                        r.inside_window(),
                        "{} / {} / {}: the inside-window split must close",
                        r.plan,
                        r.clip,
                        r.ground
                    );
                    assert_eq!(
                        r.samples,
                        r.frames.iter().map(|f| f.legs.len()).sum::<usize>(),
                        "{} / {} / {}: the series must hold every sample counted",
                        r.plan,
                        r.clip,
                        r.ground
                    );
                    // (2) finite everywhere, in the summary AND in every row.
                    for v in [
                        r.clip_sole_min_m,
                        r.clip_sole_max_m,
                        r.rendered_sole_min_m,
                        r.rendered_sole_max_m,
                        r.reachable_residual_max_m,
                        r.knee_bend_max_deg,
                    ] {
                        assert!(v.is_finite(), "{} / {}: non-finite {v}", r.plan, r.clip);
                    }
                    for row in &r.frames {
                        for l in &row.legs {
                            assert!(
                                l.rendered_sole_m.is_finite()
                                    && l.hip_deg.is_finite()
                                    && l.knee_deg.is_finite(),
                                "{} / {} frame {}: non-finite pose {l:?}",
                                r.plan,
                                r.clip,
                                row.index
                            );
                        }
                    }
                    // (4) the solver's own clamp bounds the achieved residual —
                    // BOTH edges of the annulus since the derived hip landed
                    // (2026-08-02). Derived from `solve_leg_ik`'s constants, not
                    // from today's numbers. OUTER edge: a reachable target has
                    // planar distance at most `l1 + l2 + 1e-9` (the reachability
                    // tolerance) and the solver clamps `d` to `l1 + l2 - 1e-6`
                    // (the annulus epsilon), so the reconstruction misses by at
                    // most those two. INNER edge (newly engaged: the crouch drop
                    // is an absolute 0.45 m against a DERIVED stout hip of
                    // 0.440 m, so a crouching stout's hip sits at or below the
                    // ground and idle's sole targets land inside — or exactly
                    // at — the inner annulus `|l1 − l2|`, including the
                    // degenerate hip-at-ground case that falls back to straight
                    // down): the clamp pushes `d` out to `|l1 − l2| + 1e-6`, so
                    // the foot can miss by up to that whole distance (measured
                    // on the stout: 0.010001 m = its |l1 − l2| + the epsilon,
                    // exactly the derivation). The widened bound still catches a
                    // rotation-quantizer reintroduction — one 11.25° quantum
                    // moves the biped's ankle ~172 mm >> 21 mm — which is this
                    // assertion's stated job.
                    let inner_m = (legs_l1_l2.0 - legs_l1_l2.1).abs();
                    let bound = inner_m + 1e-6 + 1e-9 + 1e-12;
                    assert!(
                        r.reachable_residual_max_m <= bound,
                        "{} / {}: residual {:.9} m exceeds the solver's annulus-clamp \
                         epsilon ({bound:.9} m) — the IK is no longer exact, or a \
                         rotation quantizer came back",
                        r.plan,
                        r.clip,
                        r.reachable_residual_max_m
                    );

                    if ground == GroundCase::Flat {
                        // (5) the sign law, against the EFFECTIVE hip (the bake
                        // delta and the crouch sink both move it). Derived, not
                        // snapshotted: a statement about `reach` vs
                        // `hip + delta - drop`, so it stays true (or vacuous) if
                        // anybody re-authors a plan or the drop. THREE regimes
                        // since the derived hip landed (2026-08-02):
                        let slack = r.reach_m - r.hip_eff_m();
                        if slack.abs() <= 1e-9 {
                            // The derived-hip rest: hip_eff == reach EXACTLY
                            // (delta = bake reach − authored pivot is a
                            // Sterbenz-exact f64 subtraction, and adding it back
                            // to the same authored pivot reproduces the reach
                            // bit for bit — `derived_hip_reaches_the_render_path`
                            // pins the exact equality). A standing sole sits ON
                            // the annulus boundary (posture-bake audit § 2.2):
                            // the only target the solver could reach is the rest
                            // pose itself (bob = 0, stride = 0), and that sample
                            // is already seated within 1 mm so the IK never runs
                            // on it. Every sample the window DOES hand the
                            // solver carries a positive clip bob or a stride
                            // offset, putting hypot(hip_eff + bob, fz) beyond
                            // reach + 1e-9 (the authored clips' bobs are all
                            // >= 0) — so nothing is corrected, and the knee
                            // stays straight: the clamped solve's bend is
                            // bounded by sqrt(2·reach·1e-6 / (l1·l2)) ≈ 0.25°
                            // for the shipped plans; 1° is that with 4x
                            // headroom.
                            assert_eq!(
                                r.corrected, 0,
                                "{} / {}: at zero slack (derived hip == reach) every \
                                 attempted target is pushed past the annulus by the \
                                 clip's non-negative bob or stride; got {} corrected",
                                r.plan, r.clip, r.corrected
                            );
                            // The SOLVER's bend, not the clip's: a refused
                            // sample renders the clip's own authored knee and
                            // says nothing about the IK. See the field's doc —
                            // the gate caught this conflation the day the root
                            // bob left the schema.
                            assert!(
                                r.solver_knee_bend_max_deg < 1.0,
                                "{} / {}: a standing body at the derived hip keeps \
                                 straight knees wherever the SOLVER ran (clamp bend \
                                 ~0.25° max), got {:.2} deg",
                                r.plan,
                                r.clip,
                                r.solver_knee_bend_max_deg
                            );
                            if r.clip == "dc:anim/biped_idle" {
                                // The planting the bake buys: idle's zero-bob,
                                // zero-rotation frames put the sole at EXACTLY
                                // ground level — the no-resting-gap invariant
                                // (bake test 2) reaching the render path. Seated,
                                // not corrected: contact by geometry, not by IK.
                                assert!(
                                    r.already_seated > 0,
                                    "{} / {}: the rest pose must SIT on the ground \
                                     (seated {} of {})",
                                    r.plan,
                                    r.clip,
                                    r.already_seated,
                                    r.samples
                                );
                            }
                        } else if slack < 0.0 {
                            // Strictly outside the annulus — the pinned-hip
                            // world's standing regime (journal/0130), now only
                            // reachable via the identity fallback or a
                            // re-authored plan; kept as the derivation it was.
                            assert_eq!(
                                r.corrected,
                                0,
                                "{} / {}: reach {:.3} < effective hip {:.3}, so a sole \
                                 at ground level is OUTSIDE the annulus and the IK \
                                 cannot reach (journal/0130); got {} corrected",
                                r.plan,
                                r.clip,
                                r.reach_m,
                                r.hip_eff_m(),
                                r.corrected
                            );
                        } else if r.inside_window() > 0 {
                            // Positive slack (today: only the crouch drop opens
                            // it) AND the window let the renderer try. BOTH gates
                            // are needed — under the pinned hip, `longleg`
                            // crouching passed the annulus and was refused by the
                            // window; under the derived hip every crouch case
                            // passes both, which is itself a change this slice
                            // measures.
                            assert!(
                                r.corrected > 0,
                                "{} / {}: reach {:.3} > effective hip {:.3} and {} \
                                 samples reached the solver, so it must solve",
                                r.plan,
                                r.clip,
                                r.reach_m,
                                r.hip_eff_m(),
                                r.inside_window()
                            );
                            assert!(
                                r.knee_bend_max_deg > 20.0,
                                "{} / {}: a reachable plan must BEND A KNEE, got {:.1} deg",
                                r.plan,
                                r.clip,
                                r.knee_bend_max_deg
                            );
                        }
                        if r.clip == "dc:anim/biped_walk" && r.root_drop_m == 0.0 {
                            reach.push((r.plan.clone(), r.reach_m, r.hip_m, r.hip_derived_m()));
                        }
                    }

                    // (6) the one-voxel-step law: window = voxel/2 < voxel = step,
                    // at every N. Nothing on the raised foot can be corrected.
                    if ground == (GroundCase::Step { rise_m: vs }) {
                        for row in &r.frames {
                            for l in row.legs.iter().filter(|l| l.ground_m > 0.0) {
                                assert!(
                                    (l.ground_m - l.clip_sole_m).abs() > half_voxel,
                                    "{} / {} frame {}: a one-voxel step must sit outside \
                                     a half-voxel window by construction",
                                    r.plan,
                                    r.clip,
                                    row.index
                                );
                                assert_eq!(
                                    l.verdict,
                                    Placement::Refused,
                                    "{} / {} frame {}: the raised foot must be refused",
                                    r.plan,
                                    r.clip,
                                    row.index
                                );
                            }
                        }
                    }
                }
            }
        }
        // (3) the rig tracks the plan: stout legs are half the biped's.
        let biped = reach.iter().find(|(p, ..)| p == "dc:body/biped").unwrap();
        let stout = reach.iter().find(|(p, ..)| p == "dc:body/stout").unwrap();
        let long = reach.iter().find(|(p, ..)| p == "dc:body/longleg").unwrap();
        let ratio = stout.1 / biped.1;
        println!(
            "\n  reach ratio stout/biped = {ratio:.3} (authored hips {:.3} / {:.3}, \
             derived {:.3} / {:.3})",
            stout.2, biped.2, stout.3, biped.3
        );
        // The absolute-length constants, expressed as a fraction of each body —
        // this is where the retargeting story stops being scale-free. The IK
        // window comes from the VOXEL SCALE and the root bob from the CLIP; the
        // leg lengths come from the plan. Nothing reconciles them.
        println!(
            "  IK window +/-{half_voxel:.3} m = {:.2}x the stout leg, {:.2}x the biped \
             leg, {:.2}x the longleg leg",
            half_voxel / stout.1,
            half_voxel / biped.1,
            half_voxel / long.1,
        );
        println!(
            "  AUTHORED hip - reach: biped {:+.3} m, stout {:+.3} m, longleg {:+.3} m \
             — the gap class the posture bake deletes (journal/0130's hover)",
            biped.2 - biped.1,
            stout.2 - stout.1,
            long.2 - long.1,
        );
        println!(
            "  DERIVED hip - reach: biped {:+.3} m, stout {:+.3} m, longleg {:+.3} m \
             — zero by construction (root height IS chain reach), so a standing \
             sole sits ON the annulus boundary and plants by geometry, not by IK",
            biped.3 - biped.1,
            stout.3 - stout.1,
            long.3 - long.1,
        );
        println!(
            "  one-voxel step {vs:.3} m / IK window {half_voxel:.3} m = {:.1}x — \
             scale-FREE, so no real terrain step ever fits the window at any N",
            vs / half_voxel
        );
        // The FOURTH absolute-metres constant, and bodies.md's units banner names
        // only three (hip height, the clips' root bob, the IK window). Ratios
        // against the DERIVED hips — the ones the renderer now stands bodies at.
        println!(
            "  CROUCH_ROOT_DROP_M {CROUCH_ROOT_DROP_M:.3} m = {:.1}% of the biped's \
             derived hip, {:.1}% of the stout's ({}), {:.1}% of the longleg's — a \
             fourth absolute length bodies.md's units banner does not name",
            100.0 * CROUCH_ROOT_DROP_M / biped.3,
            100.0 * CROUCH_ROOT_DROP_M / stout.3,
            if CROUCH_ROOT_DROP_M >= stout.3 {
                "hip sinks TO OR BELOW the ground crouching — inner-annulus \
                 clamp engages, see assertion (4)"
            } else {
                "above ground crouching"
            },
            100.0 * CROUCH_ROOT_DROP_M / long.3,
        );
        // The authored root bobs used to be itemised here as a fraction of
        // each body — 0.040 m walk, 0.120 m jump, i.e. 9.1 % of the stout's hip
        // against a *derived* 7.5 %. There is nothing left to print: the field
        // left the schema 2026-08-02 (user call #2) and root height is now a
        // function of phase that scales with the body by construction. **That
        // is one of the four absolute-metre constants this arc set out to
        // retire, gone.**
        println!();
        assert!(
            (0.45..=0.55).contains(&ratio),
            "the plan-derived rig must track the plan's proportions, got {ratio:.3}"
        );
    }

    /// **The consumer edge itself** (posture-bake audit § 5, the continuation
    /// slot): the render path stands each body at the DERIVED hip — the
    /// authored root pivot plus [`derived_root_delta_m`]'s bake delta, the
    /// same sum `character.rs` applies to the root translation and the IK
    /// hip — and the three shipped plans land at **0.880 / 0.440 / 1.020 m**
    /// against authored 0.900 / 0.460 / 0.900 (the predicted table of the
    /// design pass, measured in journal/0137, now asserted at the consumer).
    ///
    /// The `assert_eq!` on floats is deliberate and derived, not hopeful:
    /// `delta = bake root height − authored pivot` is an EXACT f64
    /// subtraction (Sterbenz — each pair is within a factor of two), so
    /// adding it back to the same authored pivot reproduces the bake's root
    /// height bit for bit, and that equals the rig's `l1 + l2` because both
    /// sum the identical pair of plan lengths. This exact equality is what
    /// the sign law's zero-slack branch stands on.
    #[test]
    fn derived_hip_reaches_the_render_path() {
        for (plan, expect) in [
            (biped_plan(), 0.880),
            (stout_plan(), 0.440),
            (longleg_plan(), 1.020),
        ] {
            let delta = derived_root_delta_m(&plan)
                .expect("every shipped plan declares `stand [sole]` and bakes");
            let rig = &leg_rigs(&plan)[0];
            let hip = rig.hip_local[1] + delta;
            assert!(
                (hip - expect).abs() < 1e-9,
                "plan `{}`: derived render hip {hip} m, expected {expect} m",
                plan.name
            );
            assert_eq!(
                hip,
                rig.l1 + rig.l2,
                "plan `{}`: the derived hip IS the chain reach (exact — see doc)",
                plan.name
            );
        }

        // The IDENTITY FALLBACK, by reason. A mode-less plan (a tree) is a
        // legal absence: the helper declines naming it, and the renderer
        // keeps the authored pivot (delta absent = 0) — today's behaviour
        // exactly, with one warn at asset build.
        let mut tree = biped_plan();
        tree.modes.clear();
        let why = derived_root_delta_m(&tree).unwrap_err();
        assert!(why.contains("no locomotion modes"), "{why}");

        // And a plan whose declared modes don't include `stand` surfaces the
        // bake's own loud refusal, naming the mode asked for.
        let mut hoverer = biped_plan();
        hoverer.modes[0].mode = "hover".into();
        let why = derived_root_delta_m(&hoverer).unwrap_err();
        assert!(why.contains("stand"), "{why}");
        assert!(why.contains("hover"), "{why}");
    }

    #[test]
    fn orientation_splits_look_from_trunk_exactly() {
        // A look within the cervical range: the neck turns, the trunk holds, and
        // the neck takes the WHOLE delta. Before 2026-08-01 the outputs were
        // snapped to an 11.25° grid and this could only be asserted loosely; the
        // split is now exact, which is the tightening the removal buys.
        // The range is now READ from the plan (B7 § 5.4) instead of being two
        // constants in this file — same numbers, declared home.
        let c = Cervical::of(&biped_plan());
        let o = resolve_orientation(0.0, 0.3, -0.2, c);
        assert!(o.trunk_yaw.abs() < 1e-12, "trunk holds within the clamp");
        assert!(
            (o.neck_yaw - 0.3).abs() < 1e-12,
            "the neck takes the whole delta, got {}",
            o.neck_yaw
        );
        assert!(
            (o.neck_pitch - -0.2).abs() < 1e-12,
            "pitch passes through unclamped, got {}",
            o.neck_pitch
        );

        // A look far to the side drags the trunk; the neck bends only its max,
        // yet the head still ends up aimed exactly at the look (trunk + neck).
        let look = 2.0;
        let o = resolve_orientation(0.0, look, 0.0, c);
        assert!(
            o.neck_yaw.abs() <= c.yaw_max.expect("a declared cervical yaw") + 1e-12,
            "neck clamped to the cervical range, got {}",
            o.neck_yaw
        );
        assert!(
            o.trunk_yaw.abs() > 1e-3,
            "trunk turned to make up the excess"
        );
        assert!(
            wrap_pi(o.trunk_yaw + o.neck_yaw - look).abs() < 1e-12,
            "head aims exactly at the look, off by {}",
            wrap_pi(o.trunk_yaw + o.neck_yaw - look)
        );

        // Pitch clamps to the cervical range.
        let o = resolve_orientation(0.0, 0.0, -1.4, c);
        assert!(
            o.neck_pitch >= c.pitch_min.expect("a declared cervical pitch") - 1e-9,
            "pitch clamped"
        );
    }
}
