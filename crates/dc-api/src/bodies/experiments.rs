//! THE BODY EXPERIMENTS — instruments with NO standing as content
//! (*existence is not standing*, CLAUDE.md). `dc:body/stout` measures clip
//! retargeting across proportions; `dc:body/longleg` is the control that let
//! the foot IK engage at all. Their doc comments carry dated measurement
//! testimony (journal/0130, journal/0131) — read it there, not here.
//! Split out of `bodies.rs` by concern 2026-08-01. Deleting the experiments
//! is this file plus one call site in dc-client `authority.rs`.

use super::default_pack::{LIMB, SKIN, TORSO, mirrored, seg, with_role, with_sole};
use super::{ActionDef, BodyPlan, ModeDef, biped_plan, segments_with_role};
use crate::payload::{DefineBodyPlan, Payload};

/// **The second plan** (`dc:body/stout`) — the instrument that tests bodies.md's
/// retargeting claim, authored in exactly the same shape as [`biped_plan`] and
/// deliberately NOT tasteful.
///
/// Same eleven joint names as the biped, and it binds the biped's three
/// *unmodified* clips (`dc:anim/biped_{idle,walk,jump}`), so [`validate_plan`]
/// accepts it and one clip set genuinely has to serve both bodies. Only the
/// geometry differs, and it differs as hard as the joint topology allows:
///
/// | measure                  | `dc:body/biped` | `dc:body/stout` | ratio |
/// |--------------------------|-----------------|-----------------|-------|
/// | leg (hip→sole)           | 0.88 m          | 0.44 m          | 0.50× |
/// | arm (shoulder→fingertip) | 0.58 m          | 0.93 m          | 1.60× |
/// | hip height               | 0.90 m          | 0.46 m          | 0.51× |
/// | trunk width × depth      | 0.50 × 0.28 m   | 0.78 × 0.50 m   | 1.56× / 1.79× |
/// | head edge                | 0.28 m          | 0.44 m          | 1.57× |
/// | standing height          | 1.80 m          | 1.60 m          | 0.89× |
///
/// The arms reach to 0.08 m above the ground when hanging: a knuckle-dragger.
/// That is the point — it puts the biped's authored swing angles, which are
/// *scale-free*, next to the clips' authored root bob and the renderer's
/// foot-IK correction window, which are **absolute metres**. Anything that
/// survives 0.5× legs and 1.6× arms is genuinely retargeting; anything that
/// does not is a proportion assumption we had never had a second body to find.
///
/// **Tints are identical to the biped's on purpose** — a side-by-side frame then
/// carries only the shape difference, with no colour cue to read as art. The
/// collider is unchanged either way: [`crate::CharacterConfig`] is world-global,
/// so the sim sees the same 1.8 m box under both plans (see the report's honest
/// list of what this experiment does NOT cover).
pub fn stout_plan() -> BodyPlan {
    // Same authoring shape as [`biped_plan`]: left limbs authored, right limbs
    // mirrored, roles on what has a consumer.
    let arm_l_upper = seg(
        "arm_l_upper",
        Some("trunk"),
        [0.47, 0.55, 0.0],
        [0.16, 0.48, 0.16],
        [0.0, -0.24, 0.0],
        LIMB,
    );
    let arm_l_lower = seg(
        "arm_l_lower",
        Some("arm_l_upper"),
        [0.0, -0.48, 0.0],
        [0.14, 0.45, 0.14],
        [0.0, -0.225, 0.0],
        SKIN,
    );
    let leg_l_upper = seg(
        "leg_l_upper",
        Some("trunk"),
        [0.2, 0.0, 0.0],
        [0.26, 0.225, 0.28],
        [0.0, -0.1125, 0.0],
        LIMB,
    );
    let leg_l_lower = with_sole(seg(
        "leg_l_lower",
        Some("leg_l_upper"),
        [0.0, -0.225, 0.0],
        [0.24, 0.215, 0.26],
        [0.0, -0.1075, 0.0],
        LIMB,
    ));
    let arm_r_upper = mirrored(&arm_l_upper, "arm_r_upper", Some("trunk"));
    let arm_r_lower = mirrored(&arm_l_lower, "arm_r_lower", Some("arm_r_upper"));
    let leg_r_upper = mirrored(&leg_l_upper, "leg_r_upper", Some("trunk"));
    let leg_r_lower = mirrored(&leg_l_lower, "leg_r_lower", Some("leg_r_upper"));
    let segments = vec![
        // Trunk root: hips at 0.46 m — half the biped's, matching the halved
        // legs — on a trunk 1.56× wider and 1.79× deeper.
        seg(
            "trunk",
            None,
            [0.0, 0.46, 0.0],
            [0.78, 0.62, 0.5],
            [0.0, 0.31, 0.0],
            TORSO,
        ),
        // Neck: short and thick, so the head sits almost on the shoulders.
        with_role(
            seg(
                "neck",
                Some("trunk"),
                [0.0, 0.62, 0.0],
                [0.22, 0.08, 0.22],
                [0.0, 0.04, 0.0],
                SKIN,
            ),
            "look",
        ),
        with_role(
            seg(
                "head",
                Some("neck"),
                [0.0, 0.08, 0.0],
                [0.44, 0.44, 0.44],
                [0.0, 0.22, 0.0],
                SKIN,
            ),
            "face",
        ),
        // Arms: 1.6× the biped's, shouldered out on the wide trunk.
        arm_l_upper,
        arm_l_lower,
        arm_r_upper,
        arm_r_lower,
        // Legs: half the biped's bone lengths, splayed wider under the trunk.
        leg_l_upper,
        leg_l_lower,
        leg_r_upper,
        leg_r_lower,
    ];
    let modes = vec![ModeDef {
        mode: "stand".into(),
        bearing: vec!["sole".into()],
    }];
    // The biped's clips, unmodified — that is the whole experiment.
    let actions = vec![
        ActionDef {
            action: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        ActionDef {
            action: "walk".into(),
            clip: "dc:anim/biped_walk".into(),
        },
        ActionDef {
            action: "jump".into(),
            clip: "dc:anim/biped_jump".into(),
        },
    ];
    BodyPlan {
        name: "dc:body/stout".into(),
        doc: "A squat, long-armed humanoid: half-length legs, 1.6x arms, a wide \
              trunk and a big head. Same eleven joints as dc:body/biped and it \
              binds the SAME clips — it exists to measure how far one clip set \
              retargets across proportions (bodies.md § IK), not to look good."
            .into(),
        segments,
        modes,
        actions,
    }
}

/// **The third plan** (`dc:body/longleg`) — the control that lets the
/// foot-placement IK *engage at all*.
///
/// journal/0130 measured that the solver has **never once solved**: 176/176
/// sampled frames were beyond the leg's reach and clamped to full extension. The
/// cause is geometric and predates every clip. [`biped_plan`]'s hip pivot sits at
/// **0.900 m** while its legs reach **0.880 m** (0.45 + 0.43), so the sole target
/// at ground level is *outside* the solver's annulus `[|l1−l2|, l1+l2]` before any
/// animation runs. [`stout_plan`] inherits the same sign of error (hip 0.46, reach
/// 0.44). The rendered result is a **hover**, not a bob.
///
/// This plan is [`biped_plan`] with **exactly two numbers changed** — the two leg
/// bone lengths — so the ONLY difference is the one under test:
///
/// | measure            | `dc:body/biped` | `dc:body/longleg` |
/// |--------------------|-----------------|-------------------|
/// | hip height         | 0.900 m         | 0.900 m (same)    |
/// | upper bone `l1`    | 0.450 m         | 0.520 m           |
/// | lower bone `l2`    | 0.430 m         | 0.500 m           |
/// | reach `l1 + l2`    | 0.880 m         | 1.020 m           |
/// | hip − reach        | **+0.020 m**    | **−0.120 m**      |
/// | trunk/arms/head    | —               | identical         |
///
/// **Why 0.120 m of slack, and not less or more.** The number is derived, not
/// picked to look right. The IK target for a planted sole is `(0, −hip_y, fz)`
/// where `fz` is however far forward the *clip* swings that foot, so the reach the
/// solver actually needs is `hypot(hip_y, fz)` — and `fz` itself scales with the
/// bone lengths, so the demand grows as the legs do. Sweeping the authored clip
/// set (`idle` 24 frames, `walk` 12, `jump` 8, both legs, at the renderer's 12 fps
/// grid) gives a fixed point just above **0.102 m** of slack; below it the walk's
/// stride extremes saturate the clamp again, and there is no slack at all that
/// covers every frame without pushing the *resting* knee past a squat:
///
/// | slack | resting knee bend | samples still beyond reach (of 88) |
/// |-------|-------------------|-----------------------------------|
/// | −0.020 (today's biped) | 0.0° (clamped straight) | **88** — every one |
/// | +0.030 | 29.2° | 32 |
/// | +0.050 | 37.3° | 14 |
/// | **+0.120 (this plan)** | **56.2°** | **2** |
/// | +0.180 | 67.1° | 0 |
///
/// 0.120 m is the smallest slack that plants **every** `idle` and `jump` frame and
/// all but two of `walk`'s, while keeping the resting bend under 60°. That the two
/// columns trade against each other at all is the finding: the free variable
/// nobody is using is the root's *vertical* travel (bodies.md § IK, the open user
/// call), and this plan is the instrument that made the trade visible, not a
/// proposal to resolve it.
///
/// **No standing as content** (*existence is not standing*): it rides
/// [`experiment_body_pack`], binds the biped's unmodified clips, and its rest pose
/// is deliberately wrong — un-IK'd *at the authored 0.900 m hip*, the soles sit
/// 0.120 m *below* the floor. The IK is what lifted them, which is precisely what
/// it was here to demonstrate. **(2026-08-02, posture-bake consumer slice: the
/// renderer no longer pins the authored hip — the derived root stands longleg at
/// 1.020 m with straight legs and soles exactly at the floor. The paragraph above
/// is dated testimony about the pinned-hip world this plan was built to probe.)**
pub fn longleg_plan() -> BodyPlan {
    let mut plan = biped_plan();
    plan.name = "dc:body/longleg".into();
    plan.doc = "The default biped with ONLY its two leg bone lengths changed \
                (0.45/0.43 -> 0.52/0.50, reach 0.880 -> 1.020 m against an \
                unchanged 0.900 m hip). It exists so a sole on the ground is \
                inside the IK's annulus and the knee must bend to reach it — the \
                control journal/0130 lacked. Not content."
        .into();
    // l1 = |lower.pivot| (hip→knee); l2 = |sole anchor| (knee→sole). Both
    // bones grow; the boxes grow with them so the rendered limb is the bone.
    // The legs are found by DECLARATION now (B0): the sole-bearing segments
    // are the lower bones and their parents the upper — the `starts_with
    // ("leg_")` prefix mutation this loop used to run is retired with the
    // naming coupling it leaned on.
    const L1: f64 = 0.52;
    const L2: f64 = 0.50;
    let lowers: Vec<String> = segments_with_role(&plan, "sole")
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let uppers: Vec<String> = plan
        .segments
        .iter()
        .filter(|s| lowers.contains(&s.name))
        .filter_map(|s| s.parent.clone())
        .collect();
    for s in &mut plan.segments {
        if uppers.contains(&s.name) {
            s.size_m[1] = L1;
            s.offset_m[1] = -L1 / 2.0;
        } else if lowers.contains(&s.name) {
            s.pivot_m[1] = -L1;
            s.size_m[1] = L2;
            s.offset_m[1] = -L2 / 2.0;
            // The sole anchor is derived from these fields — re-derive it so
            // the declaration cannot go stale against the bone it sits on.
            for r in &mut s.roles {
                if r.role == "sole" {
                    r.at_m = Some([
                        s.offset_m[0],
                        s.offset_m[1] - s.size_m[1] / 2.0,
                        s.offset_m[2],
                    ]);
                }
            }
        }
    }
    plan
}

/// **The body experiments' pack — instruments, with NO standing as content.**
///
/// - [`stout_plan`] tests bodies.md's claim that two-bone IK retargets one clip
///   set across proportions.
/// - [`longleg_plan`] tests whether the foot-placement IK solves *at all* when the
///   ground is inside the leg's reach — the control journal/0130 lacked.
///
/// Neither is a creature anybody designed, ratified, or wants in the world. They
/// ride their own batch rather than [`default_body_pack`] precisely so that the
/// **default pack** stays the default content. *Existence is not standing*
/// (CLAUDE.md): an unratified body sitting inside the default pack would, in three
/// months, be something a session found in the tree and assumed belonged there.
/// Deleting the experiments is one call site in dc-client `authority.rs` plus this
/// function and the two plan functions.
///
/// It emits **only plans**: the clips they bind are the default pack's,
/// unmodified, which is the entire point — so this batch must be submitted *after*
/// [`default_body_pack`], or the verb→slot contract rejects it for binding
/// unregistered clips.
pub fn experiment_body_pack() -> Vec<Payload> {
    vec![
        Payload::DefineBodyPlan(DefineBodyPlan(stout_plan())),
        Payload::DefineBodyPlan(DefineBodyPlan(longleg_plan())),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
        move |name| clips.iter().find(|c| c.name == name).cloned()
    }

    /// The second plan validates against the **unmodified** biped clips — the
    /// precondition of the whole retargeting experiment. If this ever fails,
    /// somebody edited a clip or a joint name and the instrument is gone.
    #[test]
    fn stout_is_well_formed_and_binds_the_biped_clips() {
        let clips = biped_clips();
        let stout = stout_plan();
        let biped = biped_plan();
        validate_plan(&stout, clip_lookup(&clips)).expect("stout plan is valid");
        // Same joint set as the biped, so one clip set serves both.
        let mut a: Vec<&str> = biped.segments.iter().map(|s| &*s.name).collect();
        let mut b: Vec<&str> = stout.segments.iter().map(|s| &*s.name).collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "stout must share the biped's joint names");
        // Same clip bindings, verbatim — no stout-specific clips exist.
        assert_eq!(
            stout.actions, biped.actions,
            "stout binds the biped's clips unmodified"
        );
    }

    /// The proportions are the experiment's independent variable, so pin the
    /// *ratios* (not the absolute numbers, which are free to be re-authored):
    /// the second plan must differ a LOT or it measures nothing.
    #[test]
    fn stout_proportions_differ_materially() {
        // Hip→sole: the upper bone (the lower segment's pivot offset) plus the
        // lower bone (its box's centre offset plus half its length).
        let leg = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "leg_l_lower").unwrap();
            lower.pivot_m[1].abs() + lower.offset_m[1].abs() + lower.size_m[1] / 2.0
        };
        let arm = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "arm_l_lower").unwrap();
            lower.pivot_m[1].abs() + lower.offset_m[1].abs() + lower.size_m[1] / 2.0
        };
        let (bip, sto) = (biped_plan(), stout_plan());
        let leg_ratio = leg(&sto) / leg(&bip);
        let arm_ratio = arm(&sto) / arm(&bip);
        assert!(
            leg_ratio <= 0.6,
            "stout legs must be at most 0.6x the biped's, got {leg_ratio:.3}"
        );
        assert!(
            arm_ratio >= 1.4,
            "stout arms must be at least 1.4x the biped's, got {arm_ratio:.3}"
        );
        // Hips ride at the top of the legs in both plans (the geometry is
        // self-consistent: a plan whose hip height did not match its leg length
        // would float or sink for reasons that are not about retargeting).
        for p in [&bip, &sto] {
            let hip = p
                .segments
                .iter()
                .find(|s| s.name == "trunk")
                .unwrap()
                .pivot_m[1];
            let sole_gap = hip - leg(p);
            assert!(
                (0.0..=0.05).contains(&sole_gap),
                "plan `{}` hip {hip} vs leg {} leaves a {sole_gap} m rest gap",
                p.name,
                leg(p)
            );
        }
    }

    /// **The third plan's defining property, asserted as a property and not as a
    /// pair of magnitudes**: `dc:body/longleg` is the biped with a leg reach
    /// *greater* than its hip height, so a sole at ground level lands **inside**
    /// the IK annulus `[|l1−l2|, l1+l2]` instead of outside it. Every other plan
    /// has the opposite sign, which is why the solver had never solved
    /// (journal/0130).
    ///
    /// Also pins the isolation: **only the leg bones differ from the biped.** If a
    /// later re-authoring drifts the trunk, the arms or the head, the control stops
    /// being a control and this fails loudly rather than quietly measuring two
    /// variables at once.
    #[test]
    fn longleg_reaches_the_ground_and_changes_nothing_else() {
        let clips = biped_clips();
        let long = longleg_plan();
        let biped = biped_plan();
        validate_plan(&long, clip_lookup(&clips)).expect("longleg plan is valid");
        assert_eq!(
            long.actions, biped.actions,
            "longleg binds the default clips unmodified"
        );

        // hip→sole reach and hip height, from the same fields the renderer's
        // `leg_rigs` reads.
        let rig = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "leg_l_lower").unwrap();
            let l1 = lower.pivot_m[1].abs();
            let l2 = lower.offset_m[1].abs() + lower.size_m[1] / 2.0;
            let hip = p
                .segments
                .iter()
                .find(|s| s.name == "trunk")
                .unwrap()
                .pivot_m[1];
            (hip, l1 + l2)
        };
        let (hip_b, reach_b) = rig(&biped);
        let (hip_l, reach_l) = rig(&long);
        assert!(
            reach_b < hip_b,
            "the biped's ground target is unreachable by construction \
             (reach {reach_b} < hip {hip_b}) — the premise of the experiment"
        );
        assert!(
            reach_l > hip_l,
            "longleg must be able to REACH the ground: reach {reach_l} <= hip {hip_l}"
        );
        assert!(
            (hip_l - hip_b).abs() < 1e-12,
            "the hip must not move — the slack has to come from the bones alone"
        );
        // Slack big enough that the knee is unmistakably bent, not a hair off
        // straight: the bend at rest is 2·acos(hip / reach) for near-equal bones.
        let bend = 2.0 * (hip_l / reach_l).acos();
        assert!(
            bend > 45.0f64.to_radians(),
            "the resting knee must be visibly bent, got {:.1}°",
            bend.to_degrees()
        );

        // Nothing but the legs differs from the biped.
        for a in &biped.segments {
            let b = long
                .segments
                .iter()
                .find(|s| s.name == a.name)
                .unwrap_or_else(|| panic!("longleg keeps the biped's joint `{}`", a.name));
            if a.name.starts_with("leg_") {
                continue;
            }
            assert_eq!(a, b, "segment `{}` must be untouched", a.name);
        }
        assert_eq!(
            biped.segments.len(),
            long.segments.len(),
            "same eleven joints"
        );
    }

    /// The declared sole anchors sit on the bottom FACE of their bone's box —
    /// including after `longleg_plan`'s mutation, which must re-derive them.
    #[test]
    fn sole_anchors_sit_on_the_bottom_face() {
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let soles = segments_with_role(&plan, "sole");
            assert_eq!(soles.len(), 2, "plan `{}` declares two soles", plan.name);
            for s in soles {
                let at = s.roles.iter().find(|r| r.role == "sole").unwrap().at_m;
                let expect = [
                    s.offset_m[0],
                    s.offset_m[1] - s.size_m[1] / 2.0,
                    s.offset_m[2],
                ];
                assert_eq!(
                    at,
                    Some(expect),
                    "plan `{}` segment `{}`: sole anchor off the bottom face",
                    plan.name,
                    s.name
                );
            }
        }
    }
}
