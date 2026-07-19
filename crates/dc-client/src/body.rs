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
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            clock_s: 0.0,
            current: Loco::Idle,
            blend_from: Loco::Idle,
            blend_w: 1.0,
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
}
