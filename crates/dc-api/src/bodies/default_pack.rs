//! The DEFAULT PACK'S authored body content — the biped plan, its clip set,
//! and the pack batch — plus the shared authoring helpers (mirror, roles).
//! Split out of `bodies.rs` by concern 2026-08-01: this is the *first content
//! pack*, compiled in for determinism, not engine API. When packs-on-disk
//! arrive, this file is the thing that migrates.

use super::{
    ActionDef, AnimClip, Axis, BodyPlan, DofDef, JointRot, Keyframe, ModeDef, RoleDef, SegmentDef,
};
use crate::payload::{DefineAnimClip, DefineBodyPlan, Payload};

pub(super) const TORSO: [f32; 3] = [0.9, 0.42, 0.12]; // signal-orange (companion legacy)
pub(super) const SKIN: [f32; 3] = [0.95, 0.85, 0.7];
pub(super) const LIMB: [f32; 3] = [0.35, 0.4, 0.55];

pub(super) fn seg(
    name: &str,
    parent: Option<&str>,
    pivot: [f64; 3],
    size: [f64; 3],
    offset: [f64; 3],
    tint: [f32; 3],
) -> SegmentDef {
    SegmentDef {
        name: name.into(),
        parent: parent.map(Into::into),
        pivot_m: pivot,
        size_m: size,
        offset_m: offset,
        tint,
        roles: Vec::new(),
        // B7's identity default: nothing declared, everything derived (S-5).
        // The one plan segment that declares anything is the neck — see
        // [`with_cervical_range`].
        dofs: None,
    }
}

/// Cervical yaw (Y) and pitch (X) ranges, radians — **the ONE declaration in
/// the shipped pack**, and it is a MIGRATION rather than a new authoring act.
///
/// These two numbers were `NECK_YAW_CLAMP_RAD` (75°) and `NECK_PITCH_CLAMP_RAD`
/// (45°) in `dc-client/src/body.rs` until 2026-08-03: world-global absolute
/// constants, blind to the plan, applied to every body including one with an
/// 0.08 m neck. They are **correct anatomy in the wrong place** (A-1, in the
/// family this arc has been retiring), and B7's declaration vocabulary is their
/// right home — so every plan that wants a cervical range now says so itself,
/// and the numbers are byte-identical to the constants they replace.
///
/// Note what is deliberately NOT declared: **roll (Z)**. The look split has
/// never produced a neck roll, and leaving it off the DOF list makes that a
/// checked fact rather than a coincidence of the renderer writing 0.
pub(super) const NECK_YAW_RAD: f64 = 75.0 * std::f64::consts::PI / 180.0;
/// See [`NECK_YAW_RAD`].
pub(super) const NECK_PITCH_RAD: f64 = 45.0 * std::f64::consts::PI / 180.0;

/// Declare the cervical DOFs on a `look` segment ([`NECK_YAW_RAD`]).
pub(super) fn with_cervical_range(mut s: SegmentDef) -> SegmentDef {
    s.dofs = Some(vec![
        DofDef {
            axis: Axis::X,
            min_rad: Some(-NECK_PITCH_RAD),
            max_rad: Some(NECK_PITCH_RAD),
        },
        DofDef {
            axis: Axis::Y,
            min_rad: Some(-NECK_YAW_RAD),
            max_rad: Some(NECK_YAW_RAD),
        },
    ]);
    s
}

/// Negate a lateral coordinate for a mirror without minting `-0.0`: the
/// authored right side had literal `0.0`s where the left does, and `-0.0` is a
/// different bit pattern — the mirror must reproduce the hand-typed plan
/// byte for byte (the seam-first acceptance test).
pub(super) fn mirror_coord(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { -x }
}

/// A left-side segment mirrored across the X=0 plane. The `_r_` rows of the
/// authored plans are EXACTLY the `_l_` rows with lateral coordinates negated
/// (verified against the previously hand-typed literals — body-plan-structure
/// design pass § 5.4), so the mirror is now structural instead of a
/// coincidence of hand-typing — the defect that opened this arc, fixed at the
/// **producer**: `BodyPlan` stays flat, and the gait bake never needs to know
/// two limbs are partners (phase assignment derives from contact geometry).
pub(super) fn mirrored(src: &SegmentDef, name: &str, parent: Option<&str>) -> SegmentDef {
    let mut s = src.clone();
    s.name = name.into();
    s.parent = parent.map(Into::into);
    s.pivot_m[0] = mirror_coord(s.pivot_m[0]);
    s.offset_m[0] = mirror_coord(s.offset_m[0]);
    for r in &mut s.roles {
        if let Some(at) = &mut r.at_m {
            at[0] = mirror_coord(at[0]);
        }
    }
    s
}

/// Declare `sole` on a lower-leg segment, anchored at the **bottom face** of
/// its box — derived from the segment's own geometry rather than typed beside
/// it, so a re-authored bone cannot leave a stale anchor behind
/// ([`longleg_plan`] mutates exactly these fields and re-derives).
pub(super) fn with_sole(mut s: SegmentDef) -> SegmentDef {
    s.roles.push(RoleDef {
        role: "sole".into(),
        at_m: Some([
            s.offset_m[0],
            s.offset_m[1] - s.size_m[1] / 2.0,
            s.offset_m[2],
        ]),
    });
    s
}

/// Declare an unanchored role on a segment (the whole segment carries it).
pub(super) fn with_role(mut s: SegmentDef, role: &str) -> SegmentDef {
    s.roles.push(RoleDef {
        role: role.into(),
        at_m: None,
    });
    s
}

/// The default biped body plan (`dc:body/biped`): trunk root, neck, head, two
/// upper/lower arms, two upper/lower legs — a modest step above the two-cuboid
/// companion (bodies.md staircase step 1: fidelity, not a rig opera). The
/// companion, and characters generally, are rendered as instances of this plan.
///
/// Geometry: feet at the root origin, +Y up, −Z forward (bevy). Pivots stack
/// child-from-parent so a limb rotates at its joint.
pub fn biped_plan() -> BodyPlan {
    // The left limbs are authored; the right limbs are MIRRORED (B0 — the
    // symmetry is structural now, not a coincidence of hand-typing). Roles:
    // soles on the lower legs (bottom face), `look` on the neck, `face` on
    // the head — only what has a consumer today.
    let arm_l_upper = seg(
        "arm_l_upper",
        Some("trunk"),
        [0.33, 0.45, 0.0],
        [0.13, 0.3, 0.13],
        [0.0, -0.15, 0.0],
        LIMB,
    );
    let arm_l_lower = seg(
        "arm_l_lower",
        Some("arm_l_upper"),
        [0.0, -0.3, 0.0],
        [0.11, 0.28, 0.11],
        [0.0, -0.14, 0.0],
        SKIN,
    );
    let leg_l_upper = seg(
        "leg_l_upper",
        Some("trunk"),
        [0.14, 0.0, 0.0],
        [0.18, 0.45, 0.2],
        [0.0, -0.225, 0.0],
        LIMB,
    );
    let leg_l_lower = with_sole(seg(
        "leg_l_lower",
        Some("leg_l_upper"),
        [0.0, -0.45, 0.0],
        [0.16, 0.43, 0.18],
        [0.0, -0.215, 0.0],
        LIMB,
    ));
    let arm_r_upper = mirrored(&arm_l_upper, "arm_r_upper", Some("trunk"));
    let arm_r_lower = mirrored(&arm_l_lower, "arm_r_lower", Some("arm_r_upper"));
    let leg_r_upper = mirrored(&leg_l_upper, "leg_r_upper", Some("trunk"));
    let leg_r_lower = mirrored(&leg_l_lower, "leg_r_lower", Some("leg_r_upper"));
    let segments = vec![
        // Trunk root: pivot at the hips (~0.9 m); torso box rises from there.
        seg(
            "trunk",
            None,
            [0.0, 0.9, 0.0],
            [0.5, 0.5, 0.28],
            [0.0, 0.25, 0.0],
            TORSO,
        ),
        with_cervical_range(with_role(
            seg(
                "neck",
                Some("trunk"),
                [0.0, 0.5, 0.0],
                [0.14, 0.12, 0.14],
                [0.0, 0.06, 0.0],
                SKIN,
            ),
            "look",
        )),
        with_role(
            seg(
                "head",
                Some("neck"),
                [0.0, 0.12, 0.0],
                [0.28, 0.28, 0.28],
                [0.0, 0.14, 0.0],
                SKIN,
            ),
            "face",
        ),
        // Arms: shoulder pivots on the trunk, boxes hang down.
        arm_l_upper,
        arm_l_lower,
        arm_r_upper,
        arm_r_lower,
        // Legs: hip pivots on the trunk, boxes reach to the feet.
        leg_l_upper,
        leg_l_lower,
        leg_r_upper,
        leg_r_lower,
    ];
    let modes = vec![ModeDef {
        mode: "stand".into(),
        bearing: vec!["sole".into()],
    }];
    let actions = vec![
        // No `walk` binding: locomotion is DERIVED (the gait bake), not
        // authored — `dc:anim/biped_walk` was retired as shipped content
        // 2026-08-02 (user call #3). See `retired_biped_walk_clip`.
        ActionDef {
            action: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        ActionDef {
            action: "jump".into(),
            clip: "dc:anim/biped_jump".into(),
        },
    ];
    BodyPlan {
        name: "dc:body/biped".into(),
        doc: "The default humanoid: jointed limbs, a neck, several segments — \
              above Minecraft, still cuboid (bodies.md)."
            .into(),
        segments,
        modes,
        actions,
    }
}

fn kf(t: f64, rots: &[(&str, [f64; 3])]) -> Keyframe {
    Keyframe {
        t,
        rotations: rots
            .iter()
            .map(|(s, e)| JointRot {
                segment: (*s).into(),
                euler: *e,
            })
            .collect(),
    }
}

/// The default biped's authored clips: `idle` and a minimal one-shot `jump`
/// (the documented fallback pose — bodies.md step 2). Angles are XYZ Euler
/// radians.
///
/// **`walk` is NOT here.** Locomotion is derived — [`super::bake_gait`] — and
/// `dc:anim/biped_walk` was retired as shipped content 2026-08-02 (user call
/// #3). It survives as [`retired_biped_walk_clip`], a parked fixture, and is
/// not registered by [`default_body_pack`] nor bound by any plan.
pub fn biped_clips() -> Vec<AnimClip> {
    // Idle: a slow breath — a faint arm splay, 2 s loop. Its root translation
    // went with `Keyframe.root_bob_m` (user call #2): a resting animal's breath
    // is chest and shoulder motion, not the whole body sliding up 15 mm.
    let idle = AnimClip {
        name: "dc:anim/biped_idle".into(),
        doc: "Standing rest: a slow breath in the shoulders.".into(),
        duration_s: 2.0,
        loops: true,
        keyframes: vec![
            kf(
                0.0,
                &[
                    ("arm_l_upper", [0.06, 0.0, 0.05]),
                    ("arm_r_upper", [0.06, 0.0, -0.05]),
                ],
            ),
            kf(
                1.0,
                &[
                    ("arm_l_upper", [-0.02, 0.0, 0.08]),
                    ("arm_r_upper", [-0.02, 0.0, -0.08]),
                ],
            ),
            kf(
                2.0,
                &[
                    ("arm_l_upper", [0.06, 0.0, 0.05]),
                    ("arm_r_upper", [0.06, 0.0, -0.05]),
                ],
            ),
        ],
    };
    // Jump: the minimal fallback — a crouch load then an extended reach, 0.6 s
    // one-shot. Real jump blending is later; the slot must exist and play.
    let jump = AnimClip {
        name: "dc:anim/biped_jump".into(),
        doc: "Minimal one-shot: crouch load into an extended reach (fallback).".into(),
        duration_s: 0.6,
        loops: false,
        keyframes: vec![
            kf(
                0.0,
                &[
                    ("leg_l_upper", [0.5, 0.0, 0.0]),
                    ("leg_l_lower", [-0.9, 0.0, 0.0]),
                    ("leg_r_upper", [0.5, 0.0, 0.0]),
                    ("leg_r_lower", [-0.9, 0.0, 0.0]),
                    ("arm_l_upper", [0.3, 0.0, 0.1]),
                    ("arm_r_upper", [0.3, 0.0, -0.1]),
                ],
            ),
            kf(
                0.3,
                &[
                    ("leg_l_upper", [-0.1, 0.0, 0.0]),
                    ("leg_l_lower", [0.1, 0.0, 0.0]),
                    ("leg_r_upper", [-0.1, 0.0, 0.0]),
                    ("leg_r_lower", [0.1, 0.0, 0.0]),
                    ("arm_l_upper", [-2.4, 0.0, 0.1]),
                    ("arm_r_upper", [-2.4, 0.0, -0.1]),
                ],
            ),
            kf(
                0.6,
                &[
                    ("leg_l_upper", [0.1, 0.0, 0.0]),
                    ("leg_l_lower", [-0.2, 0.0, 0.0]),
                    ("leg_r_upper", [0.1, 0.0, 0.0]),
                    ("leg_r_lower", [-0.2, 0.0, 0.0]),
                    ("arm_l_upper", [-0.4, 0.0, 0.1]),
                    ("arm_r_upper", [-0.4, 0.0, -0.1]),
                ],
            ),
        ],
    };
    vec![idle, jump]
}

/// **The retired `dc:anim/biped_walk` — a PARKED TEST FIXTURE, not content.**
///
/// RETIRED as shipped content 2026-08-02 by user call #3 (the gait-bake design
/// pass's header): *"Sure, A. Mostly because it's not worth doing anything
/// with."* It is **parked, not enshrined** — the user's ground was that
/// removing it is not worth the effort, NOT that the clip has evidentiary
/// standing. So: no comparison harness, no *"the derivation agrees with the
/// animator to within X"* acceptance test, no ceremony. It is a hand-typed
/// locomotion clip that the derived gait replaces, and it is kept only because
/// a couple of tests already decompose it (design § 3.5's Table C) and deleting
/// it would cost more than leaving it.
///
/// It is **not** in [`biped_clips`], **not** in [`default_body_pack`], and
/// **not** bound by any plan's actions. Its authored `root_bob_m` went with the
/// schema field (user call #2) — the clip's own leg geometry demanded 0.154 m
/// while it authored 0.040 m, which is the hover, and the derived `root_offset`
/// now owns that number.
pub fn retired_biped_walk_clip() -> AnimClip {
    // Contralateral swing, a knee bend on the trailing leg. 1 s loop, four
    // poses (two strides). Left-forward at t=0. Its step bob left with the
    // schema field; what remains is the joint testimony.
    AnimClip {
        name: "dc:anim/biped_walk".into(),
        doc: "RETIRED (2026-08-02, user call #3): contralateral limb swing, \
          1 s loop. A parked fixture, not shipped content."
            .into(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![
            kf(
                0.0,
                &[
                    ("leg_l_upper", [0.6, 0.0, 0.0]),
                    ("leg_l_lower", [-0.15, 0.0, 0.0]),
                    ("leg_r_upper", [-0.5, 0.0, 0.0]),
                    ("leg_r_lower", [0.5, 0.0, 0.0]),
                    ("arm_l_upper", [-0.5, 0.0, 0.05]),
                    ("arm_r_upper", [0.5, 0.0, -0.05]),
                ],
            ),
            kf(
                0.25,
                &[
                    ("leg_l_upper", [0.05, 0.0, 0.0]),
                    ("leg_l_lower", [-0.05, 0.0, 0.0]),
                    ("leg_r_upper", [-0.05, 0.0, 0.0]),
                    ("leg_r_lower", [0.2, 0.0, 0.0]),
                    ("arm_l_upper", [0.0, 0.0, 0.05]),
                    ("arm_r_upper", [0.0, 0.0, -0.05]),
                ],
            ),
            kf(
                0.5,
                &[
                    ("leg_l_upper", [-0.5, 0.0, 0.0]),
                    ("leg_l_lower", [0.5, 0.0, 0.0]),
                    ("leg_r_upper", [0.6, 0.0, 0.0]),
                    ("leg_r_lower", [-0.15, 0.0, 0.0]),
                    ("arm_l_upper", [0.5, 0.0, 0.05]),
                    ("arm_r_upper", [-0.5, 0.0, -0.05]),
                ],
            ),
            kf(
                0.75,
                &[
                    ("leg_l_upper", [-0.05, 0.0, 0.0]),
                    ("leg_l_lower", [0.2, 0.0, 0.0]),
                    ("leg_r_upper", [0.05, 0.0, 0.0]),
                    ("leg_r_lower", [-0.05, 0.0, 0.0]),
                    ("arm_l_upper", [0.0, 0.0, 0.05]),
                    ("arm_r_upper", [0.0, 0.0, -0.05]),
                ],
            ),
            kf(
                1.0,
                &[
                    ("leg_l_upper", [0.6, 0.0, 0.0]),
                    ("leg_l_lower", [-0.15, 0.0, 0.0]),
                    ("leg_r_upper", [-0.5, 0.0, 0.0]),
                    ("leg_r_lower", [0.5, 0.0, 0.0]),
                    ("arm_l_upper", [-0.5, 0.0, 0.05]),
                    ("arm_r_upper", [0.5, 0.0, -0.05]),
                ],
            ),
        ],
    }
}

/// The **default pack's** body content as a recorded command batch — clips first,
/// then the plans that bind them (the define order the contract requires). Content
/// packs are just registry command batches; this is the bodies pack, and **the
/// running game loads its bodies through it** (dc-client `authority.rs` submits it
/// at world construction under a `registry.define(dc)` grant). Generated from the
/// authored source ([`biped_plan`]/[`biped_clips`]) so pack and typed model cannot
/// drift (proven by `tests/bodies.rs`'s `default_pack_defines_through_the_door`
/// and `registry_content_equals_the_authored_source`).
///
/// The experiment plans are deliberately **NOT** in here — see
/// [`experiment_body_pack`].
pub fn default_body_pack() -> Vec<Payload> {
    let mut out = Vec::new();
    for clip in biped_clips() {
        out.push(Payload::DefineAnimClip(DefineAnimClip(clip)));
    }
    out.push(Payload::DefineBodyPlan(DefineBodyPlan(biped_plan())));
    out
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
        move |name| clips.iter().find(|c| c.name == name).cloned()
    }

    #[test]
    fn default_biped_is_well_formed() {
        let clips = biped_clips();
        for c in &clips {
            validate_clip(c).unwrap_or_else(|e| panic!("clip {} invalid: {e}", c.name));
        }
        validate_plan(&biped_plan(), clip_lookup(&clips)).expect("biped plan is valid");
    }

    /// The mirror helper reproduces the previously hand-typed right side byte
    /// for byte — the seam-first acceptance test for making the symmetry
    /// structural. Pivots pinned to the old literals; zeros must stay +0.0.
    #[test]
    fn mirror_reproduces_the_authored_right_side() {
        let plan = biped_plan();
        let by = |n: &str| plan.segments.iter().find(|s| s.name == n).unwrap();
        assert_eq!(by("arm_r_upper").pivot_m, [-0.33, 0.45, 0.0]);
        assert_eq!(by("leg_r_upper").pivot_m, [-0.14, 0.0, 0.0]);
        for (l, r) in [
            ("arm_l_upper", "arm_r_upper"),
            ("arm_l_lower", "arm_r_lower"),
            ("leg_l_upper", "leg_r_upper"),
            ("leg_l_lower", "leg_r_lower"),
        ] {
            let (l, r) = (by(l), by(r));
            assert_eq!(l.size_m, r.size_m);
            assert_eq!(l.tint, r.tint);
            assert_eq!(l.pivot_m[0], -r.pivot_m[0]);
            assert_eq!(l.pivot_m[1], r.pivot_m[1]);
            assert_eq!(l.pivot_m[2], r.pivot_m[2]);
            // No -0.0 minted where the left has 0.0 (byte-identity, not just ==).
            for v in [r.pivot_m, r.offset_m] {
                for x in v {
                    if x == 0.0 {
                        assert!(x.is_sign_positive(), "mirror minted a -0.0 in {v:?}");
                    }
                }
            }
        }
    }
}
