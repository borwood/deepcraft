//! The client-side animation player (docs/design/bodies.md steps 1–2).
//!
//! This is the render half of the determinism firewall: it turns a registry
//! [`AnimClip`] into a posed skeleton, entirely on the client, driven by the
//! render clock. **Nothing here is ever read back into simulation.** The sim
//! sees the swept-AABB mover (`dc_api::character`) and parametric posture; the
//! only sim state this module *reads* is a character's velocity (to pick a
//! locomotion state), which is a legal one-way read — animation is cosmetic.
//!
//! Three ratified aesthetics shape the sampler, built in from day one:
//!
//! - **Stepped ~12 fps.** The animation clock is quantized to [`ANIM_FPS`]
//!   frames before sampling, so a pose only changes twelve times a second — the
//!   stop-motion look (bodies.md § stepped animation), and cheap.
//! - **Quantized rotations.** Sampled Euler angles snap to [`ROT_QUANTUM_RAD`]
//!   increments, so joints click between discrete orientations rather than
//!   sweeping smoothly.
//! - **Stepped crossfade blend.** Locomotion transitions (idle ↔ walk) blend
//!   over a short window, and the blend weight itself is quantized to a few
//!   steps ([`BLEND_STEPS`]) so the transition reads as stop-motion too.
//!
//! The module is pure (no bevy, no glam) so it is trivially testable: a fixed
//! `(dt, speed)` history and a fixed set of clips produce an identical pose
//! stream every run (see the determinism tests). The bevy/glam glue that spawns
//! the segment hierarchy and writes joint transforms lives in `character.rs`.

use std::collections::HashMap;

use dc_api::bodies::AnimClip;

/// Stepped-animation frame rate: the pose updates this many times per second.
pub const ANIM_FPS: f64 = 12.0;
/// Rotation quantum: sampled Euler angles snap to multiples of this (32 steps
/// per revolution — 11.25°).
pub const ROT_QUANTUM_RAD: f64 = std::f64::consts::TAU / 32.0;
/// Crossfade window for a locomotion transition, seconds (short and stepped).
pub const BLEND_WINDOW_S: f64 = 0.18;
/// The blend weight is quantized to this many steps across the window.
pub const BLEND_STEPS: f64 = 4.0;
/// Root-bob quantum, meters: the vertical bob snaps to this grid (stepped like
/// the rotations, and robust to float rounding at the loop-wrap boundary).
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
/// direction. Short, and the *rendered* yaw is quantized, so turns read stepped.
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

/// Snap an angle to the [`ROT_QUANTUM_RAD`] grid.
fn quantize_angle(a: f64) -> f64 {
    (a / ROT_QUANTUM_RAD).round() * ROT_QUANTUM_RAD
}

/// Snap an angle to the stepped-rotation grid (public wrapper for the renderer,
/// which quantizes IK output so the closed-form solver's smooth angles never
/// leak past the 12 fps stop-motion look).
pub fn stepped_angle(a: f64) -> f64 {
    quantize_angle(a)
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
/// linearly interpolate between the bracketing keyframes at that stepped time,
/// then quantize the resulting angles. The two quantizations together are the
/// stop-motion look.
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
        let e = lerp3(rot_in(a, name), rot_in(b, name), f);
        pose.joints.insert(
            name.to_string(),
            [
                quantize_angle(e[0]),
                quantize_angle(e[1]),
                quantize_angle(e[2]),
            ],
        );
    }
    pose.root_bob_m = quantize_bob(a.root_bob_m + (b.root_bob_m - a.root_bob_m) * f);
    pose
}

/// Blend two poses by weight `w` (0 = all `a`, 1 = all `b`). Angles are lerped
/// then re-quantized; the root bob is lerped. `w` is expected pre-stepped.
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
        let e = lerp3(za, zb, w);
        pose.joints.insert(
            name.to_string(),
            [
                quantize_angle(e[0]),
                quantize_angle(e[1]),
                quantize_angle(e[2]),
            ],
        );
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
    /// The stored yaw is smooth; callers quantize it for display.
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
/// All angles are stepped (quantized) for the stop-motion look.
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
        trunk_yaw: quantize_angle(trunk_yaw),
        neck_yaw: quantize_angle(neck_yaw),
        neck_pitch: quantize_angle(neck_pitch),
    }
}

/// A two-bone IK solution: the upper and lower joint rotations (radians, about
/// X — the sagittal plane), with the rest pose pointing straight down (−Y).
/// `upper_x` is applied to the hip joint (relative to the trunk), `lower_x` to
/// the knee joint (relative to the upper). Angles are NOT quantized here — the
/// renderer snaps them so smooth solver output never leaks past the 12 fps grid.
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

    #[test]
    fn angles_are_quantized() {
        let (_, walk) = clips();
        let p = sample_clip(&walk, 0.13);
        for e in p.joints.values() {
            for &a in e {
                let steps = a / ROT_QUANTUM_RAD;
                assert!(
                    (steps - steps.round()).abs() < 1e-9,
                    "angle {a} is not on the quantization grid"
                );
            }
        }
    }

    #[test]
    fn looping_wraps_deterministically() {
        let (_, walk) = clips();
        // One full loop apart samples the same phase.
        let a = sample_clip(&walk, 0.4);
        let b = sample_clip(&walk, 0.4 + walk.duration_s);
        assert_eq!(a, b, "looped phase repeats");
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
    /// hip, `lower_x` at the knee (relative to the upper).
    fn fk_foot(l1: f64, l2: f64, upper_x: f64, lower_x: f64) -> (f64, f64) {
        let ky = -l1 * upper_x.cos();
        let kz = -l1 * upper_x.sin();
        let total = upper_x + lower_x;
        (ky - l2 * total.cos(), kz - l2 * total.sin())
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

    #[test]
    fn orientation_is_stepped_and_splits_look_from_trunk() {
        // A look within the cervical range: the neck turns, the trunk holds,
        // and every output angle lands on the quantization grid.
        let o = resolve_orientation(0.0, 0.3, -0.2);
        assert!(o.trunk_yaw.abs() < 1e-9, "trunk holds within the clamp");
        for a in [o.trunk_yaw, o.neck_yaw, o.neck_pitch] {
            let steps = a / ROT_QUANTUM_RAD;
            assert!((steps - steps.round()).abs() < 1e-9, "angle {a} off grid");
        }

        // A look far to the side drags the trunk; the neck bends only its max,
        // yet the head still ends up aimed at the look (trunk + neck ≈ look).
        let look = 2.0;
        let o = resolve_orientation(0.0, look, 0.0);
        assert!(
            o.neck_yaw.abs() <= NECK_YAW_CLAMP_RAD + ROT_QUANTUM_RAD,
            "neck clamped to the cervical range, got {}",
            o.neck_yaw
        );
        assert!(
            o.trunk_yaw.abs() > 1e-3,
            "trunk turned to make up the excess"
        );
        assert!(
            wrap_pi(o.trunk_yaw + o.neck_yaw - look).abs() < ROT_QUANTUM_RAD * 1.5,
            "head aims at the look"
        );

        // Pitch clamps to the cervical range.
        let o = resolve_orientation(0.0, 0.0, -1.4);
        assert!(
            o.neck_pitch >= -NECK_PITCH_CLAMP_RAD - 1e-9,
            "pitch clamped"
        );
    }
}
