//! The client-side animation player (docs/design/bodies.md steps 1–2).
//!
//! This is the render half of the determinism firewall: it turns a registry
//! [`AnimClip`] into a posed skeleton, entirely on the client, driven by the
//! render clock. **Nothing here is ever read back into simulation.** The sim
//! sees the swept-AABB mover (`dc_api::character`) and parametric posture; the
//! only sim state this module *reads* is a character's velocity (to pick a
//! locomotion state), which is a legal one-way read — animation is cosmetic.
//!
//! Two stepping devices shape the sampler:
//!
//! - **Stepped ~12 fps.** The animation clock is quantized to [`ANIM_FPS`]
//!   frames before sampling, so a pose only changes twelve times a second — the
//!   stop-motion look (bodies.md § stepped animation), and cheap.
//! - **Stepped crossfade blend.** Locomotion transitions (idle ↔ walk) blend
//!   over a short window, and the blend weight itself is quantized to a few
//!   steps ([`BLEND_STEPS`]) so the transition reads as stop-motion too.
//!
//! **There used to be a third: sampled Euler angles snapped to a `TAU/32`
//! (11.25°) rotation grid. It was REMOVED 2026-08-01 by user ruling** —
//! bodies.md § stepped animation carries the banner. It was assistant-originated
//! (a guard against IK instability that 1,056 measured samples show does not
//! exist, journal/0131) and it cost sub-decimetre foot placement outright: one
//! quantum of hip rotation moves the biped's ankle 172 mm, against corrections
//! that need 1.30° / 0.98° / 0.33°. Joint angles are now exact. The 12 fps step
//! is untouched and rides pending the user's taste call.
//!
//! The module is pure (no bevy, no glam) so it is trivially testable: a fixed
//! `(dt, speed)` history and a fixed set of clips produce an identical pose
//! stream every run (see the determinism tests). The bevy/glam glue that spawns
//! the segment hierarchy and writes joint transforms lives in `character.rs`.

use std::collections::HashMap;

use dc_api::bodies::{AnimClip, BodyPlan};

/// Stepped-animation frame rate: the pose updates this many times per second.
pub const ANIM_FPS: f64 = 12.0;
/// Crossfade window for a locomotion transition, seconds (short and stepped).
pub const BLEND_WINDOW_S: f64 = 0.18;
/// The blend weight is quantized to this many steps across the window.
pub const BLEND_STEPS: f64 = 4.0;
/// Root-bob quantum, meters: the vertical bob snaps to this grid. Kept when the
/// rotation quantizer went (2026-08-01): it is a *positional* snap on a single
/// authored scalar, and its second job — robustness to float rounding at the
/// loop-wrap boundary — is not aesthetic.
pub const BOB_QUANTUM_M: f64 = 0.005;
/// Horizontal speed (m/s) above which a body is "walking".
pub const WALK_SPEED_THRESHOLD_M_S: f64 = 0.35;
/// Cervical yaw range: the head may turn this far (radians, ~75°) relative to
/// the trunk before the trunk itself turns to make up the difference. Closes
/// the walk-8 orientation gap: the trunk faces travel, the head faces the look.
pub const NECK_YAW_CLAMP_RAD: f64 = 75.0 * std::f64::consts::PI / 180.0;
/// Cervical pitch range (radians, ~45°); negative looks down (character.rs
/// convention). Beyond this the neck simply clamps — no trunk pitch in v0.
pub const NECK_PITCH_CLAMP_RAD: f64 = 45.0 * std::f64::consts::PI / 180.0;
/// Trunk turn window (seconds): how quickly the trunk yaw chases the travel
/// direction. Short, and the pose is sampled on the 12 fps grid, so turns still
/// read stepped in time even though the yaw itself is now exact.
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

/// The trunk yaw (bevy convention, 0 = −Z) that faces a horizontal velocity.
/// Zero-length velocity has no facing; the caller keeps the last trunk yaw.
pub fn yaw_from_velocity(vel_x: f64, vel_z: f64) -> Option<f64> {
    if vel_x * vel_x + vel_z * vel_z <= 1e-12 {
        None
    } else {
        // view_dir at yaw θ is (−sinθ, ·, −cosθ); to face (vx, vz) horizontally
        // we need θ = atan2(−vx, −vz).
        Some((-vel_x).atan2(-vel_z))
    }
}

/// A sampled skeletal pose: per-joint XYZ Euler rotation (radians) plus a
/// vertical root bob (meters). Joints absent from the map are identity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    pub joints: HashMap<String, [f64; 3]>,
    pub root_bob_m: f64,
}

/// Quantize an animation time to the [`ANIM_FPS`] grid, then wrap (looping
/// clips) or clamp (one-shots) into `[0, duration]`.
fn quantize_time(t: f64, duration_s: f64, loops: bool) -> f64 {
    let stepped = (t * ANIM_FPS).floor() / ANIM_FPS;
    if loops {
        // rem_euclid keeps a negative or huge clock in range deterministically.
        stepped.rem_euclid(duration_s)
    } else {
        stepped.clamp(0.0, duration_s)
    }
}

fn quantize_blend(w: f64) -> f64 {
    ((w.clamp(0.0, 1.0) * BLEND_STEPS).round() / BLEND_STEPS).clamp(0.0, 1.0)
}

fn quantize_bob(b: f64) -> f64 {
    (b / BOB_QUANTUM_M).round() * BOB_QUANTUM_M
}

fn lerp3(a: [f64; 3], b: [f64; 3], f: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
}

/// Sample a clip at animation time `t`: quantize the time to the frame grid,
/// then linearly interpolate between the bracketing keyframes at that stepped
/// time. **The time step is the whole stop-motion look** — the angles it yields
/// are exact (the rotation quantizer was removed 2026-08-01).
pub fn sample_clip(clip: &AnimClip, t: f64) -> Pose {
    let mut pose = Pose::default();
    if clip.keyframes.is_empty() {
        return pose;
    }
    let st = quantize_time(t, clip.duration_s, clip.loops);
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
    pose.root_bob_m = quantize_bob(a.root_bob_m + (b.root_bob_m - a.root_bob_m) * f);
    pose
}

/// Blend two poses by weight `w` (0 = all `a`, 1 = all `b`). Angles and the root
/// bob are lerped. `w` is expected pre-stepped.
pub fn blend(a: &Pose, b: &Pose, w: f64) -> Pose {
    let mut pose = Pose {
        joints: HashMap::new(),
        root_bob_m: quantize_bob(a.root_bob_m + (b.root_bob_m - a.root_bob_m) * w),
    };
    let mut names: Vec<&str> = Vec::new();
    for p in [a, b] {
        for k in p.joints.keys() {
            if !names.contains(&k.as_str()) {
                names.push(k);
            }
        }
    }
    for name in names {
        let za = a.joints.get(name).copied().unwrap_or([0.0, 0.0, 0.0]);
        let zb = b.joints.get(name).copied().unwrap_or([0.0, 0.0, 0.0]);
        pose.joints.insert(name.to_string(), lerp3(za, zb, w));
    }
    pose
}

/// The two locomotion states v0 blends between (driven by body velocity).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loco {
    Idle,
    Walk,
}

/// Per-body animation state: the animation clock plus the crossfade machine.
/// Advanced from the render clock (`dt`) and the body's horizontal speed.
#[derive(Clone, Debug)]
pub struct AnimState {
    /// Animation clock, seconds (drives clip phase; quantized at sample time).
    pub clock_s: f64,
    /// The locomotion state currently blended *toward*.
    pub current: Loco,
    /// The state being blended *from* during a transition.
    pub blend_from: Loco,
    /// Blend progress 0..1 from `blend_from` to `current`.
    pub blend_w: f64,
    /// The smoothed trunk yaw (radians, bevy convention) — the body's facing,
    /// derived from its horizontal velocity (a legal one-way read of sim state).
    /// Persists while stationary so a stopped body keeps facing where it walked.
    pub trunk_yaw: f64,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            clock_s: 0.0,
            current: Loco::Idle,
            blend_from: Loco::Idle,
            blend_w: 1.0,
            trunk_yaw: 0.0,
        }
    }
}

impl AnimState {
    /// Advance one render frame: tick the clock, pick the locomotion state from
    /// `speed_m_s`, and progress (or start) a crossfade.
    pub fn advance(&mut self, dt: f64, speed_m_s: f64) {
        let dt = dt.max(0.0);
        self.clock_s += dt;
        let desired = if speed_m_s > WALK_SPEED_THRESHOLD_M_S {
            Loco::Walk
        } else {
            Loco::Idle
        };
        if desired != self.current {
            self.blend_from = self.current;
            self.current = desired;
            self.blend_w = 0.0;
        }
        if BLEND_WINDOW_S > 0.0 {
            self.blend_w = (self.blend_w + dt / BLEND_WINDOW_S).min(1.0);
        } else {
            self.blend_w = 1.0;
        }
    }

    /// The stepped crossfade weight (quantized).
    pub fn stepped_blend(&self) -> f64 {
        quantize_blend(self.blend_w)
    }

    /// Steer the trunk toward the travel direction. When the body is moving
    /// (horizontal speed above the walk threshold) the trunk yaw chases the
    /// velocity heading over [`TRUNK_TURN_WINDOW_S`]; when stationary it holds.
    /// The stored yaw is smooth, and so is the rendered one.
    pub fn steer(&mut self, dt: f64, vel_x: f64, vel_z: f64) {
        let dt = dt.max(0.0);
        let speed = (vel_x * vel_x + vel_z * vel_z).sqrt();
        if speed <= WALK_SPEED_THRESHOLD_M_S {
            return; // keep the last facing
        }
        let Some(target) = yaw_from_velocity(vel_x, vel_z) else {
            return;
        };
        let delta = wrap_pi(target - self.trunk_yaw);
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
/// All angles are exact radians; only the 12 fps time step remains.
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

/// Split a look direction off a travel-facing trunk (bodies.md step 3, the
/// walk-8 fix). The head follows the look within the cervical clamp; a look
/// beyond the clamp drags the trunk around so the neck only ever bends its
/// maximum. `base_trunk_yaw` is the velocity-followed facing ([`AnimState`]).
pub fn resolve_orientation(base_trunk_yaw: f64, look_yaw: f64, look_pitch: f64) -> Orientation {
    let delta = wrap_pi(look_yaw - base_trunk_yaw);
    let (trunk_yaw, neck_yaw) = if delta.abs() > NECK_YAW_CLAMP_RAD {
        let clamped = NECK_YAW_CLAMP_RAD * delta.signum();
        // Trunk absorbs the excess so the neck bends exactly its limit.
        (wrap_pi(look_yaw - clamped), clamped)
    } else {
        (base_trunk_yaw, delta)
    };
    let neck_pitch = look_pitch.clamp(-NECK_PITCH_CLAMP_RAD, NECK_PITCH_CLAMP_RAD);
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
}

/// Derive each leg's two-bone rig from a body plan: hip = parent(upper).pivot +
/// upper.pivot; `l1` = |lower.pivot| (hip→knee); `l2` = |lower.offset.y| +
/// lower.size.y / 2 (knee→sole). Generic over any `leg_*_upper` / `leg_*_lower`
/// pair, so a plan is not required to be a biped to get foot placement — it is
/// required only to *name* its legs that way (a naming coupling, noted honestly:
/// see the second-plan report).
pub fn leg_rigs(plan: &BodyPlan) -> Vec<LegRig> {
    let seg_by = |name: &str| plan.segments.iter().find(|s| s.name == name);
    let mut legs = Vec::new();
    for upper in plan
        .segments
        .iter()
        .filter(|s| s.name.starts_with("leg_") && s.name.ends_with("_upper"))
    {
        let lower_name = upper.name.replace("_upper", "_lower");
        let Some(lower) = seg_by(&lower_name) else {
            continue;
        };
        let hip_local = match upper.parent.as_deref().and_then(seg_by) {
            Some(parent) => [
                parent.pivot_m[0] + upper.pivot_m[0],
                parent.pivot_m[1] + upper.pivot_m[1],
                parent.pivot_m[2] + upper.pivot_m[2],
            ],
            None => upper.pivot_m,
        };
        let l1 =
            (lower.pivot_m[0].powi(2) + lower.pivot_m[1].powi(2) + lower.pivot_m[2].powi(2)).sqrt();
        let l2 = lower.offset_m[1].abs() + lower.size_m[1] / 2.0;
        legs.push(LegRig {
            upper: upper.name.clone(),
            lower: lower_name,
            hip_local,
            l1,
            l2,
        });
    }
    legs
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LegIk {
    pub upper_x: f64,
    pub lower_x: f64,
    /// False when the target was beyond `l1 + l2`: the leg is left reaching
    /// straight at it (fully extended) and the caller lets the foot float
    /// rather than stretch the bones.
    pub reachable: bool,
}

/// Closed-form two-bone IK in the sagittal (Y–Z) plane. `target` is the desired
/// foot position **relative to the hip joint** (meters; x lateral, y up, z
/// forward = −z), `l1`/`l2` the upper/lower bone lengths. The knee pole points
/// forward (−Z), so knees bend like knees. The reach is clamped into the
/// solvable annulus `[|l1−l2|, l1+l2]`, kept a hair off the singular ends so the
/// solution is always finite (never NaN) and the knee never locks dead straight.
pub fn solve_leg_ik(l1: f64, l2: f64, target: [f64; 3]) -> LegIk {
    let ty = target[1];
    let tz = target[2];
    let d_full = (target[0] * target[0] + ty * ty + tz * tz).sqrt();
    let max = l1 + l2;
    let min = (l1 - l2).abs();
    let reachable = d_full <= max + 1e-9;
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
    let (ka, kb) = (knee(1.0), knee(-1.0));
    let (ky, kz) = if ka.1 <= kb.1 { ka } else { kb };
    // Absolute X-rotation of a planar point p: rotX(θ)·(0,−1,0) = (−cosθ, −sinθ)
    // in (y, z), so θ = atan2(−p.z, −p.y).
    let upper_x = (-kz).atan2(-ky);
    // The (clamped) target, and the lower bone from knee to it.
    let (fy, fz) = (uy * d, uz * d);
    let total = (-(fz - kz)).atan2(-(fy - ky));
    let lower_x = wrap_pi(total - upper_x);
    LegIk {
        upper_x,
        lower_x,
        reachable,
    }
}

/// The pose to render for this state, given the plan's idle/walk clips. Samples
/// the target clip; when mid-transition, blends it against the from-clip at the
/// stepped weight.
pub fn pose_for(state: &AnimState, idle: &AnimClip, walk: &AnimClip) -> Pose {
    let clip = |l: Loco| match l {
        Loco::Idle => idle,
        Loco::Walk => walk,
    };
    let to = sample_clip(clip(state.current), state.clock_s);
    if state.blend_from == state.current || state.blend_w >= 1.0 {
        to
    } else {
        let from = sample_clip(clip(state.blend_from), state.clock_s);
        blend(&from, &to, state.stepped_blend())
    }
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
    pub root_bob_m: f64,
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
/// Every figure is in **metres**, measured over the clip's *stepped* frames (the
/// 12 fps grid the renderer actually samples), for every leg the plan declares.
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
    /// authored — *before* the crouch drop. See [`RetargetReport::hip_eff_m`].
    pub hip_m: f64,
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
    pub knee_bend_max_deg: f64,
    /// Worst residual over samples the IK both attempted and could reach — how far
    /// off the ground the foot still is after the glue ran, in metres. Until
    /// 2026-08-01 this measured the **rotation quantum's** cost (bounded by
    /// `reach × 11.25°`, ~172 mm on the biped); with the quantizer gone it measures
    /// the **solver's own** residual, bounded by its annulus-clamp epsilon.
    pub reachable_residual_max_m: f64,
}

#[cfg(test)]
impl RetargetReport {
    /// The corrections the renderer attempted, reached or not.
    pub fn inside_window(&self) -> usize {
        self.corrected + self.clamped_beyond_reach
    }

    /// The hip height the solver actually works against: the authored hip minus the
    /// crouch drop. **This, not `hip_m`, is what `reach_m` has to beat** — which is
    /// why the stock biped's IK engages while crouching and never while standing.
    pub fn hip_eff_m(&self) -> f64 {
        self.hip_m - self.root_drop_m
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
    let mut r = RetargetReport {
        plan: plan.name.clone(),
        clip: clip.name.clone(),
        ground: ground.label(),
        root_drop_m,
        hip_m: first.hip_local[1],
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
        reachable_residual_max_m: 0.0,
    };
    let frames = ((clip.duration_s * ANIM_FPS).ceil() as usize).max(1);
    for i in 0..frames {
        let t_s = i as f64 / ANIM_FPS;
        let pose = sample_clip(clip, t_s);
        let mut row = FrameRow {
            index: i,
            t_s,
            root_bob_m: pose.root_bob_m,
            legs: Vec::new(),
        };
        for leg in &legs {
            let cu = pose.joints.get(&leg.upper).map_or(0.0, |e| e[0]);
            let cl = pose.joints.get(&leg.lower).map_or(0.0, |e| e[0]);
            let (fy, fz) = fk_foot_local(leg.l1, leg.l2, cu, cl);
            // The hip rides at hip_local.y plus the clip's bob, minus the crouch
            // sink; the root is where the collider bottom is, and terrain never
            // moves it (character.rs applies exactly these three terms).
            let hip_y = leg.hip_local[1] + pose.root_bob_m - root_drop_m;
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
                let ik = solve_leg_ik(leg.l1, leg.l2, [0.0, fy + adjust, fz]);
                let (sy, _) = fk_foot_local(leg.l1, leg.l2, ik.upper_x, ik.lower_x);
                let sole = hip_y + sy;
                let verdict = if ik.reachable {
                    r.reachable_residual_max_m = r.reachable_residual_max_m.max((sole - g).abs());
                    Placement::Corrected
                } else {
                    Placement::ClampedBeyondReach
                };
                (verdict, sole, ik.upper_x, ik.lower_x)
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
    use dc_api::bodies::biped_clips;

    fn clips() -> (AnimClip, AnimClip) {
        let cs = biped_clips();
        let idle = cs
            .iter()
            .find(|c| c.name == "dc:anim/biped_idle")
            .unwrap()
            .clone();
        let walk = cs
            .iter()
            .find(|c| c.name == "dc:anim/biped_walk")
            .unwrap()
            .clone();
        (idle, walk)
    }

    #[test]
    fn time_is_stepped_to_twelve_fps() {
        // Two times inside the same 1/12 s frame sample identically; the next
        // frame differs.
        let (_, walk) = clips();
        let a = sample_clip(&walk, 0.30);
        let b = sample_clip(&walk, 0.30 + 1.0 / (ANIM_FPS * 4.0)); // same frame
        assert_eq!(a, b, "within one frame the pose is held");
        let c = sample_clip(&walk, 0.30 + 1.0 / ANIM_FPS); // next frame
        assert_ne!(a.joints, c.joints, "the next frame moves");
    }

    /// One full loop apart samples the same phase — to f64 precision, not bit
    /// for bit.
    ///
    /// **This test used to assert bit-identity, and it passed only because the
    /// rotation quantizer was rounding f64 noise away** (found when the quantizer
    /// was removed 2026-08-01 — the assertion had been guarding nothing about
    /// looping and quietly guarding `rem_euclid`'s last two ULPs). `quantize_time`
    /// floors on a grid anchored at absolute `t = 0` and *then* wraps, so
    /// `(1.4 * 12).floor() / 12 - 1.0` and `(0.4 * 12).floor() / 12` are the same
    /// real number and differ in the final bit.
    ///
    /// The bound is derived, not fitted: the wrap error is a few ULPs of a value
    /// ~1.4 (≤ 1e-15 s), the walk clip's keyframe spans are 0.25 s so the
    /// interpolation factor moves by ≤ 4e-15, and no joint traverses more than
    /// ~1.1 rad across a span — ≤ 5e-15 rad of angle. 1e-12 leaves two decades of
    /// margin and is still 6e-11 degrees, i.e. below any conceivable display.
    /// `root_bob_m` stays *exactly* equal because [`BOB_QUANTUM_M`] survives, which
    /// is the float-robustness job its doc comment claims.
    #[test]
    fn looping_wraps_to_the_same_phase() {
        let (_, walk) = clips();
        let a = sample_clip(&walk, 0.4);
        let b = sample_clip(&walk, 0.4 + walk.duration_s);
        assert_eq!(
            a.root_bob_m, b.root_bob_m,
            "the quantized bob repeats exactly"
        );
        assert_eq!(
            a.joints.keys().collect::<std::collections::BTreeSet<_>>(),
            b.joints.keys().collect::<std::collections::BTreeSet<_>>(),
            "the looped pose names the same joints"
        );
        for (name, ea) in &a.joints {
            let eb = b.joints[name];
            for i in 0..3 {
                assert!(
                    (ea[i] - eb[i]).abs() < 1e-12,
                    "{name}[{i}]: looped phase differs by {} rad, beyond f64 wrap noise",
                    ea[i] - eb[i]
                );
            }
        }
    }

    #[test]
    fn sampler_is_deterministic_for_fixed_time() {
        let (_, walk) = clips();
        for t in [0.0, 0.05, 0.333, 0.5, 0.917, 1.4, 7.25] {
            assert_eq!(sample_clip(&walk, t), sample_clip(&walk, t));
        }
    }

    #[test]
    fn advance_history_replays_identically() {
        // A fixed (dt, speed) history yields an identical pose stream twice —
        // the whole player, not just the sampler, is deterministic.
        let (idle, walk) = clips();
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
                s.advance(dt, spd);
                poses.push(pose_for(&s, &idle, &walk));
            }
            poses
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn walk_starts_and_stops_by_speed() {
        let mut s = AnimState::default();
        assert_eq!(s.current, Loco::Idle);
        s.advance(0.1, 5.0);
        assert_eq!(s.current, Loco::Walk, "speed above threshold walks");
        // Blend runs and completes over the window.
        assert!(s.blend_w < 1.0 || BLEND_WINDOW_S <= 0.1);
        for _ in 0..10 {
            s.advance(0.05, 5.0);
        }
        assert!((s.stepped_blend() - 1.0).abs() < 1e-9, "blend completes");
        s.advance(0.1, 0.0);
        assert_eq!(s.current, Loco::Idle, "speed below threshold idles");
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

    #[test]
    fn ik_reconstructs_reachable_targets_with_a_forward_knee() {
        let (l1, l2) = (0.45, 0.43);
        for target in [
            [0.0, -0.8, 0.0],
            [0.0, -0.6, -0.3],
            [0.0, -0.7, 0.15],
            [0.05, -0.75, -0.1],
        ] {
            let ik = solve_leg_ik(l1, l2, target);
            assert!(ik.reachable, "{target:?} is within reach");
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
        // Beyond reach: flagged, still finite, near full extension.
        let ik = solve_leg_ik(l1, l2, [0.0, -2.0, 0.0]);
        assert!(!ik.reachable);
        assert!(ik.upper_x.is_finite() && ik.lower_x.is_finite());
        let (fy, _) = fk_foot(l1, l2, ik.upper_x, ik.lower_x);
        assert!(
            (fy.abs() - (l1 + l2)).abs() < 0.01,
            "clamped to near full extension, got {fy}"
        );
        // Degenerate: target at the hip. No NaN.
        let ik = solve_leg_ik(l1, l2, [0.0, 0.0, 0.0]);
        assert!(ik.upper_x.is_finite() && ik.lower_x.is_finite());
    }

    #[test]
    fn ik_is_deterministic_for_fixed_inputs() {
        let t = [0.03, -0.72, -0.15];
        assert_eq!(solve_leg_ik(0.45, 0.43, t), solve_leg_ik(0.45, 0.43, t));
    }

    #[test]
    fn steer_faces_travel_holds_when_stopped_and_replays() {
        let hist = [
            (0.016, 1.0, 0.0),
            (0.02, 1.0, 0.0),
            (0.02, 0.0, -1.0),
            (0.05, 0.0, 0.0),
            (0.016, -1.0, 0.0),
        ];
        let run = || {
            let mut s = AnimState::default();
            let mut ys = Vec::new();
            for (dt, vx, vz) in hist {
                s.steer(dt, vx, vz);
                ys.push(s.trunk_yaw);
            }
            ys
        };
        assert_eq!(run(), run(), "fixed steer history replays identically");

        // Steering toward +X travel settles the trunk facing +X (yaw −π/2).
        let mut s = AnimState::default();
        for _ in 0..60 {
            s.steer(0.05, 1.0, 0.0);
        }
        assert!(
            wrap_pi(s.trunk_yaw - (-std::f64::consts::FRAC_PI_2)).abs() < 1e-3,
            "trunk faces travel, got {}",
            s.trunk_yaw
        );
        // Stationary: the facing is held, not reset.
        let held = s.trunk_yaw;
        s.steer(0.1, 0.0, 0.0);
        assert_eq!(s.trunk_yaw, held, "a stopped body keeps its facing");
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
            "\n  -- {} / {} / {} / {} -- hip {:.3} - drop {:.3} = effective {:.3} \
             | reach {:.3} (slack {:+.3})",
            r.plan.trim_start_matches("dc:body/"),
            r.clip.trim_start_matches("dc:anim/biped_"),
            r.ground,
            if r.root_drop_m > 0.0 {
                "crouch"
            } else {
                "stand"
            },
            r.hip_m,
            r.root_drop_m,
            r.hip_eff_m(),
            r.reach_m,
            r.reach_m - r.hip_eff_m(),
        );
        let mut head = format!("     {:>2} {:>6} {:>7}", "f", "t(s)", "bob");
        for l in &r.frames[0].legs {
            head.push_str(&format!(
                " | {:<7} {:>7} {:>7} {:>6} {:>6} {:>4}",
                l.leg, "gnd", "sole", "gap", "hip", "knee"
            ));
        }
        println!("{head} (deg; v = verdict)");
        for row in &r.frames {
            let mut line = format!(
                "     {:>2} {:>6.3} {:>+7.3}",
                row.index, row.t_s, row.root_bob_m
            );
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
             | knee|max| {:.1} deg | residual<= {:.9} m | clip sole [{:+.3},{:+.3}] \
             rendered [{:+.3},{:+.3}]",
            r.samples,
            r.already_seated,
            r.corrected,
            r.clamped_beyond_reach,
            r.refused,
            r.knee_bend_max_deg,
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
    ///    solver's **own annulus-clamp epsilon** — still a *derived* bound, but a
    ///    different one since 2026-08-01: the rotation quantizer is gone, so the
    ///    bound fell from `reach × 11.25°` (~172 mm on the biped) to ~1 µm. This
    ///    is the assertion that would catch its silent reintroduction;
    /// 5. **the sign law**, against the **effective** hip (`hip − crouch drop`): a
    ///    plan whose `reach ≤ hip_eff` can never correct a single frame on flat
    ///    ground (the sole target is outside the annulus before animation runs —
    ///    journal/0130's refutation, restated as the derivation rather than as
    ///    today's count); a plan whose `reach > hip_eff` **and** whose window let the
    ///    solver run must solve and must bend a knee. Both gates are load-bearing:
    ///    `longleg` crouching passes the annulus and is refused by the window;
    /// 6. **the one-voxel-step law**: the correction window is half a voxel, so a
    ///    step of one whole voxel — the smallest relief real terrain can have at
    ///    *any* scale N — can never fit inside it. Every raised-foot sample is
    ///    refused, and that 2:1 ratio is by construction, independent of N.
    ///
    /// Scale-free: every quantity is a per-frame, per-leg arithmetic on the plan's
    /// own lengths, so one clip loop at 12 fps exercises the whole invariant.
    #[test]
    fn foot_placement_and_retargeting_are_measured() {
        let clips = biped_clips();
        let plans = [
            dc_api::bodies::biped_plan(),
            dc_api::bodies::stout_plan(),
            dc_api::bodies::longleg_plan(),
        ];
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
                    // (4) the solver's own clamp bounds the achieved residual.
                    // Derived from `solve_leg_ik`'s constants, not from today's
                    // numbers: a reachable target has planar distance at most
                    // `l1 + l2 + 1e-9` (the reachability tolerance) and the solver
                    // clamps `d` to `l1 + l2 - 1e-6` (the annulus epsilon, which
                    // keeps the knee off dead-straight), so the reconstruction can
                    // miss by those two and f64 slop, and by nothing else.
                    let bound = 1e-6 + 1e-9 + 1e-12;
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
                        // (5) the sign law, against the EFFECTIVE hip (the crouch
                        // sink moves it). Derived, not snapshotted: a statement about
                        // `reach` vs `hip - drop`, so it stays true (or vacuous) if
                        // anybody re-authors a plan or the drop.
                        if r.reach_m <= r.hip_eff_m() {
                            assert_eq!(
                                r.corrected,
                                0,
                                "{} / {}: reach {:.3} <= effective hip {:.3}, so a sole \
                                 at ground level is OUTSIDE the annulus and the IK \
                                 cannot reach (journal/0130); got {} corrected",
                                r.plan,
                                r.clip,
                                r.reach_m,
                                r.hip_eff_m(),
                                r.corrected
                            );
                        } else if r.inside_window() > 0 {
                            // The annulus admits it AND the window let the renderer
                            // try. BOTH gates are needed — `longleg` crouching passes
                            // the annulus and is refused by the window, which is the
                            // sharpest statement that nothing reconciles the two.
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
                            reach.push((r.plan.clone(), r.reach_m, r.hip_m));
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
            "\n  reach ratio stout/biped = {ratio:.3} (hips {:.3} / {:.3})",
            stout.2, biped.2
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
            "  hip - reach: biped {:+.3} m, stout {:+.3} m, longleg {:+.3} m \
             (positive = the ground is UNREACHABLE before animation runs)",
            biped.2 - biped.1,
            stout.2 - stout.1,
            long.2 - long.1,
        );
        println!(
            "  one-voxel step {vs:.3} m / IK window {half_voxel:.3} m = {:.1}x — \
             scale-FREE, so no real terrain step ever fits the window at any N",
            vs / half_voxel
        );
        // The FOURTH absolute-metres constant, and bodies.md's units banner names
        // only three (hip height, the clips' root bob, the IK window).
        println!(
            "  CROUCH_ROOT_DROP_M {CROUCH_ROOT_DROP_M:.3} m = {:.1}% of the biped's \
             hip height, {:.1}% of the stout's, {:.1}% of the longleg's — a fourth \
             absolute length bodies.md's units banner does not name",
            100.0 * CROUCH_ROOT_DROP_M / biped.2,
            100.0 * CROUCH_ROOT_DROP_M / stout.2,
            100.0 * CROUCH_ROOT_DROP_M / long.2,
        );
        for (name, bob) in [("walk", 0.04_f64), ("jump", 0.12_f64)] {
            println!(
                "  authored {name} root bob {bob:.3} m = {:.1}% of the stout's hip \
                 height, {:.1}% of the biped's",
                100.0 * bob / stout.2,
                100.0 * bob / biped.2
            );
        }
        println!();
        assert!(
            (0.45..=0.55).contains(&ratio),
            "the plan-derived rig must track the plan's proportions, got {ratio:.3}"
        );
    }

    #[test]
    fn orientation_splits_look_from_trunk_exactly() {
        // A look within the cervical range: the neck turns, the trunk holds, and
        // the neck takes the WHOLE delta. Before 2026-08-01 the outputs were
        // snapped to an 11.25° grid and this could only be asserted loosely; the
        // split is now exact, which is the tightening the removal buys.
        let o = resolve_orientation(0.0, 0.3, -0.2);
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
        let o = resolve_orientation(0.0, look, 0.0);
        assert!(
            o.neck_yaw.abs() <= NECK_YAW_CLAMP_RAD + 1e-12,
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
        let o = resolve_orientation(0.0, 0.0, -1.4);
        assert!(
            o.neck_pitch >= -NECK_PITCH_CLAMP_RAD - 1e-9,
            "pitch clamped"
        );
    }
}
