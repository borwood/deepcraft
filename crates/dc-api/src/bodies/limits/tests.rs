//! B7's acceptance set (joint-limits audit § 8), engine half. The IK members
//! (tests 10 and 11) live with the solver in dc-client.
//!
//! Invariants and derivations, never snapshots — except § 3.2's numeric table,
//! which the design pass asks for explicitly and which is a *derivation over
//! pinned plan literals*: if a number here moves, either the law moved or the
//! plan did, and both are things a reader must be told about.

use super::*;
use crate::bodies::{
    ActionDef, AnimClip, JointRot, Keyframe, ModeDef, RoleDef, biped_clips, biped_plan,
    longleg_plan, retired_biped_walk_clip, stout_plan, validate_plan,
};

fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
    move |name| clips.iter().find(|c| c.name == name).cloned()
}

fn plans() -> [BodyPlan; 3] {
    [biped_plan(), stout_plan(), longleg_plan()]
}

/// Strip every declaration so the DERIVATION alone is under test. The shipped
/// pack has exactly one declaration — the neck's cervical range, § 5.4's
/// byte-identical migration of two client constants — and § 3.2 / § 3.3 are
/// tables about geometry, not about what the pack chose to say.
fn derived_only(mut plan: BodyPlan) -> BodyPlan {
    for s in &mut plan.segments {
        s.dofs = None;
    }
    plan
}

fn bound_of(plan: &BodyPlan, segment: &str, axis: Axis, max: bool) -> Bound {
    let limits = derive_joint_limits(plan);
    let joint = limits
        .joint(segment)
        .unwrap_or_else(|| panic!("plan `{}` has joint `{segment}`", plan.name));
    let dof = joint
        .dof(axis)
        .unwrap_or_else(|| panic!("`{segment}` has a {} DOF", axis.name()));
    if max {
        dof.max.clone()
    } else {
        dof.min.clone()
    }
}

/// A test fixture: the biped, with the DOF declaration the default pack is
/// **not** allowed to carry yet (audit § 5.1 — a declared one-sided knee rejects
/// two of the three shipped clips, and the default pack would fail to define and
/// the game would not start). The declaration lands for real with the gait slice
/// that retires `dc:anim/biped_walk`. **Fixture only, never the shipped pack.**
fn biped_with_declared_knees() -> BodyPlan {
    let mut plan = biped_plan();
    for s in &mut plan.segments {
        if s.name.starts_with("leg_") && s.name.ends_with("_lower") {
            s.dofs = Some(vec![DofDef {
                axis: Axis::X,
                // min derived (the fold magnitude the geometry supplies),
                // max DECLARED: a knee does not go past straight. The whole
                // declaration is ONE number (audit § 4.2).
                min_rad: None,
                max_rad: Some(0.0),
            }]);
        }
    }
    plan
}

/// The same fixture bound to exactly one action, so a rejection names the clip
/// under test rather than whichever action the loop reaches first.
fn fixture_binding(action: &str, clip: &str) -> BodyPlan {
    let mut plan = biped_with_declared_knees();
    plan.actions = vec![ActionDef {
        action: action.into(),
        clip: clip.into(),
    }];
    plan
}

// --------------------------------------------------------------- test 1 (a) --

/// **The S-5 acceptance, engine half.** With `dofs` declared nowhere but the
/// neck (whose declaration is § 5.4's byte-identical migration of two client
/// constants), every shipped plan and every shipped clip validates exactly as
/// before, the resting bake is unchanged, and the derivation asserts nothing:
/// every bound is `Derived` or `Undetermined`, never `Declared`.
///
/// The IK half — bit-identical solver output across the measured sweep — is
/// `nothing_declared_leaves_the_solver_bit_identical` in dc-client.
#[test]
fn nothing_declared_is_byte_identical() {
    let clips = biped_clips();
    for plan in plans() {
        validate_plan(&plan, clip_lookup(&clips))
            .unwrap_or_else(|e| panic!("plan `{}` must still validate: {e}", plan.name));
        let limits = derive_joint_limits(&plan);
        // **The one band report the shipped pack produces, and it is a
        // finding, not a defect.** `dc:body/stout` declares the migrated
        // cervical pitch of ±45° (§ 5.4, byte-identical to the retired
        // `NECK_PITCH_CLAMP_RAD`) against a derived self-contact bound of
        // ±36.03° — its neck is an 0.08 m slab on a 0.62 m trunk, and it
        // genuinely cannot pitch that far. That is EXACTLY the A-1 defect
        // § 5.4 named ("the stout's short thick neck gets the biped's
        // numbers"), now visible instead of invisible. Reported, never
        // refused — so the render is unchanged and the migration stays
        // byte-identical.
        let expect_reports = if plan.name == "dc:body/stout" { 2 } else { 0 };
        assert_eq!(
            limits.reports.len(),
            expect_reports,
            "plan `{}` band reports: {:?}",
            plan.name,
            limits.reports
        );
        for r in &limits.reports {
            assert!(
                r.contains("neck") && r.contains("about X") && r.contains("not refused"),
                "{r}"
            );
        }
        for j in &limits.joints {
            if j.segment == "neck" {
                continue; // § 5.4's migration, the one declaration in the pack
            }
            for d in &j.dofs {
                for (end, b) in [("min", &d.min), ("max", &d.max)] {
                    assert!(
                        !matches!(b, Bound::Declared(_)),
                        "plan `{}` joint `{}` {end} about {} is DECLARED; slice one \
                         declares nothing outside the neck migration",
                        plan.name,
                        j.segment,
                        d.axis.name()
                    );
                }
            }
        }
        // The resting bake still bakes, and its own new check is vacuous.
        assert!(
            matches!(
                crate::bodies::bake_resting_posture(&plan, "stand"),
                crate::bodies::BakeOutcome::Baked(_)
            ),
            "plan `{}` must still bake a resting posture",
            plan.name
        );
    }
}

// ----------------------------------------------------------------- test 2 --

/// **§ 3.2's table, measured.** Every row is a derivation from `default_pack.rs`
/// / `experiments.rs` literals; a divergence is a finding about the design
/// document (immutable body, mutable header), and the ones this build found are
/// stamped on its header.
///
/// Scale-free: the assertion is in radians on a plan whose lengths are pinned by
/// `mirror_reproduces_the_authored_right_side` and
/// `longleg_reaches_the_ground_and_changes_nothing_else`.
#[test]
fn derived_limits_match_the_predicted_table() {
    // (plan, joint, axis, measured rad or None for Undetermined, the audit's
    // § 3.2 prediction for the report).
    #[allow(clippy::type_complexity)]
    let rows: &[(&str, &str, Axis, Option<f64>, Option<f64>)] = &[
        // knees — the audit's magnitudes confirmed to ~1e-4 rad
        (
            "biped",
            "leg_l_lower",
            Axis::X,
            Some(2.705_630_055_648_907),
            Some(2.705_41),
        ),
        (
            "stout",
            "leg_l_lower",
            Axis::X,
            Some(2.006_725_103_173_108_6),
            Some(2.006_76),
        ),
        (
            "longleg",
            "leg_l_lower",
            Axis::X,
            Some(2.765_369_302_485_903),
            Some(2.765_46),
        ),
        // hips — likewise, and the stout/biped SPREAD is the finding
        (
            "biped",
            "leg_l_upper",
            Axis::X,
            Some(2.614_347_249_741_193_5),
            Some(2.614_31),
        ),
        (
            "stout",
            "leg_l_upper",
            Axis::X,
            Some(1.352_267_952_406_582_8),
            Some(1.353_04),
        ),
        ("longleg", "leg_l_upper", Axis::X, None, None),
        // elbows
        (
            "biped",
            "arm_l_lower",
            Axis::X,
            Some(2.717_826_195_059_022_3),
            Some(2.717_87),
        ),
        (
            "longleg",
            "arm_l_lower",
            Axis::X,
            Some(2.717_826_195_059_022_3),
            Some(2.717_87),
        ),
        (
            "stout",
            "arm_l_lower",
            Axis::X,
            Some(2.810_692_493_662_150_7),
            Some(2.810_76),
        ),
        // shoulders — geometry constrains them not at all
        ("biped", "arm_l_upper", Axis::X, None, None),
        ("stout", "arm_l_upper", Axis::X, None, None),
        ("longleg", "arm_l_upper", Axis::X, None, None),
        // ⚠ THE NECK ROWS DIVERGE, and the module doc says why: the audit
        // derived them from the HEAD's distal corner with the neck's own box
        // omitted, while deriving the hips from the thigh's — the rotating
        // segment's own box. Include it, as the hip rows do, and the neck's
        // top corner reaches the trunk first.
        (
            "biped",
            "neck",
            Axis::X,
            Some(1.042_721_878_536_622_5),
            Some(2.468_25),
        ),
        (
            "stout",
            "neck",
            Axis::X,
            Some(0.628_796_286_415_433_2),
            Some(2.282_91),
        ),
        ("biped", "neck", Axis::Y, None, None),
        (
            "biped",
            "neck",
            Axis::Z,
            Some(1.042_721_878_536_622_5),
            Some(2.174_08),
        ),
        // the knee's other two axes
        ("biped", "leg_l_lower", Axis::Y, None, None),
        (
            "biped",
            "leg_l_lower",
            Axis::Z,
            Some(2.750_397_280_311_131_6),
            Some(2.750_46),
        ),
        // ⚠ `head` DIVERGES: the audit read it as *"no child chain to
        // collide"*, but a segment's own box is part of its rotating subtree,
        // and the head's top corners reach the trunk at 126.87°.
        ("biped", "head", Axis::X, Some(2.214_297_435_588_181), None),
        // `trunk` is the ROOT: welded, which is stronger than undetermined.
        ("biped", "trunk", Axis::X, None, None),
    ];
    let by_name = |n: &str| {
        derived_only(match n {
            "biped" => biped_plan(),
            "stout" => stout_plan(),
            "longleg" => longleg_plan(),
            other => panic!("unknown plan {other}"),
        })
    };
    println!(
        "\n§ 3.2 MEASURED vs PREDICTED  (derived symmetric bound, +max end)\n\
         {:<9} {:<13} {:<4} {:>13} {:>11} {:>13} {:>11}",
        "plan", "joint", "axis", "measured rad", "measured °", "audit rad", "audit °"
    );
    let mut bad: Vec<String> = Vec::new();
    for (pn, seg, axis, expect, doc) in rows {
        let plan = by_name(pn);
        // `trunk` is the welded root: it has no DOFs at all, which is a
        // stronger statement than an undetermined bound.
        let limits = derive_joint_limits(&plan);
        let joint = limits.joint(seg).expect("joint present");
        let measured = joint.dof(*axis).map(|d| d.max.clone());
        let (m_rad, note) = match &measured {
            None => (None, "WELDED (root)".to_string()),
            Some(Bound::Undetermined { .. }) => (None, "UNDETERMINED".to_string()),
            Some(b) => (b.value(), String::new()),
        };
        println!(
            "{pn:<9} {seg:<13} {:<4} {:>13} {:>11} {:>13} {:>11}  {note}",
            axis.name(),
            m_rad.map_or("—".into(), |v| format!("{v:.7}")),
            m_rad.map_or("—".into(), |v| format!("{:.3}", v.to_degrees())),
            doc.map_or("—".into(), |v: f64| format!("{v:.7}")),
            doc.map_or("—".into(), |v: f64| format!("{:.3}", v.to_degrees())),
        );
        match (expect, m_rad) {
            (None, None) => {}
            (Some(e), Some(m)) if (m - e).abs() <= 1e-7 => {}
            (e, m) => bad.push(format!(
                "{pn}/{seg}/{}: pinned {e:?}, measured {m:?}",
                axis.name()
            )),
        }
        // Every derived range is symmetric on these plans: every box in every
        // shipped plan is centred in z, so the bodies have no front and the law
        // returns the same magnitude both ways (§ 3.4 — the sign is the one bit
        // geometry cannot supply).
        if let Some(d) = joint.dof(*axis)
            && let (Some(lo), Some(hi)) = (d.min.value(), d.max.value())
            && (lo + hi).abs() > 1e-12
        {
            bad.push(format!(
                "{pn}/{seg}/{}: a shipped plan's derived range must be symmetric \
                 (every box is z-centred — the bodies have no front), got [{lo:.9}, {hi:.9}]",
                axis.name()
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// ----------------------------------------------------------------- test 3 --

/// Angles are dimensionless. Scale a plan uniformly and every derived bound must
/// be *identical* — if a scale changes one, the law has a length in it that
/// should not be there.
#[test]
fn derived_limits_are_scale_invariant() {
    for k in [0.4_f64, 2.5] {
        for plan in plans().map(derived_only) {
            let base = derive_joint_limits(&plan);
            let mut scaled = plan.clone();
            for s in &mut scaled.segments {
                for v in s
                    .pivot_m
                    .iter_mut()
                    .chain(&mut s.size_m)
                    .chain(&mut s.offset_m)
                {
                    *v *= k;
                }
                for r in &mut s.roles {
                    if let Some(at) = &mut r.at_m {
                        for v in at.iter_mut() {
                            *v *= k;
                        }
                    }
                }
            }
            let got = derive_joint_limits(&scaled);
            for (a, b) in base.joints.iter().zip(&got.joints) {
                for (da, db) in a.dofs.iter().zip(&b.dofs) {
                    for (x, y) in [(&da.min, &db.min), (&da.max, &db.max)] {
                        match (x.value(), y.value()) {
                            (None, None) => {}
                            (Some(u), Some(v)) => assert!(
                                (u - v).abs() <= 1e-12,
                                "plan `{}` joint `{}` {} moved under a {k}x scale: {u} -> {v}",
                                plan.name,
                                a.segment,
                                da.axis.name()
                            ),
                            (u, v) => panic!(
                                "plan `{}` joint `{}` {} changed KIND under a {k}x scale: \
                                 {u:?} -> {v:?}",
                                plan.name,
                                a.segment,
                                da.axis.name()
                            ),
                        }
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------- test 4 --

/// **The property that makes L3 a derivation rather than a constant** — and the
/// one L2 (which returns exactly 90° for every joint on every body) fails.
/// Asserted as an ORDERING, never as magnitudes: a squat body with a deep trunk
/// and short thighs genuinely cannot fold or swing as far, and nobody typed it.
#[test]
fn stout_folds_less_than_the_biped() {
    let (biped, stout) = (derived_only(biped_plan()), derived_only(stout_plan()));
    let knee_b = bound_of(&biped, "leg_l_lower", Axis::X, true)
        .value()
        .expect("the biped knee is bounded");
    let knee_s = bound_of(&stout, "leg_l_lower", Axis::X, true)
        .value()
        .expect("the stout knee is bounded");
    assert!(
        knee_s < knee_b,
        "stout knee {knee_s:.4} must fold less than the biped's {knee_b:.4}"
    );
    let hip_b = bound_of(&biped, "leg_l_upper", Axis::X, true)
        .value()
        .expect("the biped hip is bounded");
    let hip_s = bound_of(&stout, "leg_l_upper", Axis::X, true)
        .value()
        .expect("the stout hip is bounded");
    assert!(
        hip_s < hip_b,
        "stout hip {hip_s:.4} must swing less than the biped's {hip_b:.4}"
    );
}

// ----------------------------------------------------------------- test 5 --

/// A legal absence, never a silent ∞: the shoulder, the longleg hip and the neck
/// yaw come back `Undetermined` with a reason that names the geometry.
#[test]
fn an_unbounded_joint_is_loudly_undetermined() {
    let cases: [(BodyPlan, &str, Axis, &str); 4] = [
        (
            derived_only(biped_plan()),
            "arm_l_upper",
            Axis::X,
            "X extent is invariant",
        ),
        (
            derived_only(longleg_plan()),
            "leg_l_upper",
            Axis::X,
            "clears this segment",
        ),
        (
            derived_only(biped_plan()),
            "neck",
            Axis::Y,
            "Y extent is invariant",
        ),
        // The stout's shoulders are drawn FLUSH against its trunk, so the
        // distal extent never *enters*: a graze in the authored geometry, said
        // out loud rather than read as a weld (see `first_entry`).
        (
            derived_only(stout_plan()),
            "arm_l_upper",
            Axis::X,
            "already intersects this segment at REST",
        ),
    ];
    for (plan, seg, axis, needle) in cases {
        let b = bound_of(&plan, seg, axis, true);
        let Bound::Undetermined { reason } = &b else {
            panic!(
                "plan `{}` joint `{seg}` about {} must be Undetermined, got {b:?}",
                plan.name,
                axis.name()
            );
        };
        assert!(
            reason.contains(needle) && reason.contains(seg),
            "the reason must name the geometry, got: {reason}"
        );
        assert!(b.value().is_none(), "an undetermined bound has no value");
    }
}

// ----------------------------------------------------------------- test 6 --

/// **The A-3 guard.** Test 1 proves the mechanism is OFF by default; this proves
/// it is ARMED. On a fixture plan (never the shipped pack — audit § 5.1), a
/// declared one-sided knee rejects `dc:anim/biped_walk`, which hyperextends both
/// knees by 28.65° on every cycle and has since bring-up.
#[test]
fn a_hyperextending_clip_is_rejected() {
    let mut clips = biped_clips();
    clips.push(retired_biped_walk_clip());
    let err = validate_plan(
        &fixture_binding("walk", "dc:anim/biped_walk"),
        clip_lookup(&clips),
    )
    .expect_err("a knee that folds one way must reject a clip that hyperextends it");
    for needle in [
        "leg_r_lower",        // the joint
        "dc:anim/biped_walk", // the clip
        "keyframe 0",         // the keyframe
        "+0.500000",          // the value
        "DECLARED",           // which end was declared
        "derived",            // and which was derived
    ] {
        assert!(err.contains(needle), "the error must name {needle}: {err}");
    }
    println!("\n§ 4.3's worked example, as built:\n{err}\n");

    // § 5.1's second table, confirmed: `biped_jump`'s single +0.10 keyframe is
    // caught too (its one-line content fix rides the gait slice), while `idle`
    // — which keys no leg at all — survives.
    let err = validate_plan(
        &fixture_binding("jump", "dc:anim/biped_jump"),
        clip_lookup(&clips),
    )
    .expect_err("jump's +0.10 knee is a hyperextension too");
    assert!(
        err.contains("+0.100000") && err.contains("leg_l_lower"),
        "{err}"
    );
    validate_plan(
        &fixture_binding("idle", "dc:anim/biped_idle"),
        clip_lookup(&clips),
    )
    .expect("idle keys no leg and must survive a declared knee");
}

// ----------------------------------------------------------------- test 7 --

/// DOF membership, distinct from range: a clip keying Y on a one-DOF hinge fails,
/// and the error names the DOF set rather than muttering "0.12 exceeds [0, 0]".
#[test]
fn an_undeclared_axis_is_rejected() {
    let mut clips = biped_clips();
    clips
        .iter_mut()
        .find(|c| c.name == "dc:anim/biped_idle")
        .unwrap()
        .keyframes[0]
        .rotations
        .push(JointRot {
            segment: "leg_l_lower".into(),
            euler: [0.0, 0.12, 0.0],
        });
    let err = validate_plan(
        &fixture_binding("idle", "dc:anim/biped_idle"),
        clip_lookup(&clips),
    )
    .expect_err("a hinge has no twist");
    assert!(
        err.contains("leg_l_lower") && err.contains("about Y") && err.contains("declares"),
        "the error must name the joint and its DOF set: {err}"
    );
}

// ----------------------------------------------------------------- test 8 --

/// `Some([])` is a WELD: zero DOFs, and any non-zero rotation is a define-time
/// error. This is also the ROOT's derived default (audit § 4.4, ruled
/// provisionally) — body orientation belongs to the facing system.
#[test]
fn a_welded_segment_refuses_any_rotation() {
    // (a) an explicit weld on an ordinary joint
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "arm_l_upper")
        .unwrap()
        .dofs = Some(Vec::new());
    let err =
        validate_plan(&plan, clip_lookup(&biped_clips())).expect_err("a welded joint cannot swing");
    assert!(
        err.contains("arm_l_upper") && err.contains("WELDED"),
        "the error must say the joint is welded: {err}"
    );
    // (b) the root's derived weld, with no declaration at all
    let limits = derive_joint_limits(&biped_plan());
    let root = limits.joint("trunk").expect("the trunk is the root");
    assert!(
        root.dofs.is_empty(),
        "the root's derived default is a weld, got {:?}",
        root.dofs
    );
    let mut clips = biped_clips();
    clips
        .iter_mut()
        .find(|c| c.name == "dc:anim/biped_idle")
        .unwrap()
        .keyframes[0]
        .rotations
        .push(JointRot {
            segment: "trunk".into(),
            euler: [0.0, 0.3, 0.0],
        });
    let err = validate_plan(&biped_plan(), clip_lookup(&clips))
        .expect_err("a clip may not key the root: trunk facing owns body orientation");
    assert!(err.contains("trunk") && err.contains("WELDED"), "{err}");
}

// ----------------------------------------------------------------- test 9 --

/// Composition is **per end of per axis**: declaring only `max` leaves `min` at
/// the derived value *exactly*; declaring both ignores the derivation; and a
/// declared bound outside the derived one is **reported and accepted**, never
/// refused (audit § 4.3 check 4 — a clockwork golem may want to bend wrong).
#[test]
fn declared_overrides_derived_per_end() {
    let derived_min = bound_of(&derived_only(biped_plan()), "leg_l_lower", Axis::X, false)
        .value()
        .expect("the biped knee's fold magnitude is derivable");

    // (a) one end declared, the other inherited exactly.
    let one_ended = biped_with_declared_knees();
    let lim = derive_joint_limits(&one_ended);
    let dof = lim
        .joint("leg_l_lower")
        .unwrap()
        .dof(Axis::X)
        .expect("declared X DOF");
    assert_eq!(dof.max, Bound::Declared(0.0));
    assert_eq!(
        dof.min,
        Bound::Derived(derived_min),
        "the undeclared end must be the derived value EXACTLY — an author who had \
         to restate it would be minting a copy that can drift (S-3)"
    );
    assert!(lim.reports.is_empty(), "0.0 is inside the derived range");

    // (b) both ends declared: the derivation is ignored, and a bound OUTSIDE it
    //     is reported rather than refused.
    let mut wide = biped_plan();
    wide.segments
        .iter_mut()
        .find(|s| s.name == "leg_l_lower")
        .unwrap()
        .dofs = Some(vec![DofDef {
        axis: Axis::X,
        // Past the 155.02° at which this shank's own corner would be inside
        // the thigh: physically impossible for the body as drawn, and legal.
        min_rad: Some(-3.4),
        max_rad: Some(0.2),
    }]);
    let lim = derive_joint_limits(&wide);
    let dof = lim.joint("leg_l_lower").unwrap().dof(Axis::X).unwrap();
    assert_eq!(dof.min, Bound::Declared(-3.4));
    assert_eq!(lim.reports.len(), 1, "one band report: {:?}", lim.reports);
    let r = &lim.reports[0];
    assert!(
        r.contains("-3.400000") && r.contains("leg_l_lower") && r.contains("not refused"),
        "the report names both numbers: {r}"
    );
    validate_plan(&wide, clip_lookup(&biped_clips()))
        .expect("a band violation is REPORTED, never refused");
}

// ---------------------------------------------------------------- test 12 --

/// The resting pose is inside every limit, on every shipped plan — vacuous today
/// by construction (`min <= 0 <= max` is a validator invariant and the resting
/// angles are all exactly zero), and that is the point: the guard sits *before*
/// its trigger, member #0's effort seam.
#[test]
fn the_resting_pose_is_inside_every_limit() {
    for plan in plans() {
        let limits = derive_joint_limits(&plan);
        for j in &limits.joints {
            for d in &j.dofs {
                assert!(
                    d.contains(0.0),
                    "plan `{}` joint `{}` {} excludes its own rest",
                    plan.name,
                    j.segment,
                    d.axis.name()
                );
            }
        }
        let crate::bodies::BakeOutcome::Baked(p) =
            crate::bodies::bake_resting_posture(&plan, "stand")
        else {
            panic!("plan `{}` must bake", plan.name);
        };
        for chain in &p.chains {
            for ja in &chain.joints {
                limits
                    .check(&ja.segment, ja.euler)
                    .unwrap_or_else(|e| panic!("plan `{}`: {e}", plan.name));
            }
        }
    }
    // And the bake REFUSES when the check fails: a knee declared to a range
    // that excludes the resting column is out of the standing solve's domain.
    // (Reached here by declaring past the validator, which the bake does not
    // re-run — the two checks guard different doors.)
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "leg_l_lower")
        .unwrap()
        .dofs = Some(vec![DofDef {
        axis: Axis::X,
        min_rad: Some(-2.0),
        max_rad: Some(-0.5),
    }]);
    let out = crate::bodies::bake_resting_posture(&plan, "stand");
    let crate::bodies::BakeOutcome::Unsupported { reason } = &out else {
        panic!("a rest outside a declared range must be REFUSED, got {out:?}");
    };
    assert!(
        reason.contains("leg_l_lower") && reason.contains("outside its range"),
        "{reason}"
    );
}

// ---------------------------------------------------------------- test 13 --

/// § 3.3's measure-against-the-literature check, **reported and never asserted**.
///
/// ⚠ The published figures are the design pass's, carried forward verbatim with
/// its own disclaimer: *"from the assistant's knowledge and NOT network-verified"*
/// (AAOS/AMA goniometric norms as reported in Norkin & White, *Measurement of
/// Joint Motion*; passive knee flexion per Kapandji, *The Physiology of the
/// Joints*). This build had no network either and adds no citation of its own.
/// They are printed beside the derivation because **a closed system cannot detect
/// its own scale error**, and the pattern is the finding: the derivation is good
/// where the end-range is BONY and useless where it is LIGAMENTOUS.
#[test]
fn report_the_literature_comparison() {
    let biped = derived_only(biped_plan());
    let rows: &[(&str, &str, Axis, &str)] = &[
        (
            "knee flexion",
            "leg_l_lower",
            Axis::X,
            "135-150 active; ~160 passive",
        ),
        (
            "knee extension",
            "leg_l_lower",
            Axis::X,
            "0 (genu recurvatum past ~10)",
        ),
        ("elbow flexion", "arm_l_lower", Axis::X, "145-150"),
        ("hip flexion", "leg_l_upper", Axis::X, "120 (knee flexed)"),
        (
            "hip extension",
            "leg_l_upper",
            Axis::X,
            "20-30 (iliofemoral ligament)",
        ),
        ("cervical rotation", "neck", Axis::Y, "60-80"),
        ("cervical flex/ext", "neck", Axis::X, "45-50 / 45-70"),
        ("cervical lateral", "neck", Axis::Z, "45"),
        ("shoulder flexion", "arm_l_upper", Axis::X, "180"),
    ];
    println!(
        "\n§ 3.3 DERIVED vs PUBLISHED (degrees; published NOT network-verified)\n\
         {:<20} {:>12}  published",
        "joint", "derived"
    );
    let limits = derive_joint_limits(&biped);
    for (label, seg, axis, published) in rows {
        let d = limits.joint(seg).and_then(|j| j.dof(*axis));
        let v = d.and_then(|d| d.max.value());
        println!(
            "{label:<20} {:>12}  {published}",
            v.map_or("undetermined".into(), |v: f64| format!(
                "{:.2}",
                v.to_degrees()
            ))
        );
    }
}

// ------------------------------------------------ the derivation's own edges --

/// L3's degeneracy escape, asserted rather than trusted: **every derived bound
/// on every shipped plan is strictly positive.** The naive law (full box-vs-box)
/// returns 0° for every joint because a parent and child box abut at the pivot;
/// testing the distal extent only is what makes the derivation exist at all, and
/// a regression there would silently weld the whole skeleton.
#[test]
fn no_derived_bound_is_degenerate() {
    for plan in plans().map(derived_only) {
        for j in &derive_joint_limits(&plan).joints {
            for d in &j.dofs {
                for (end, b) in [("min", &d.min), ("max", &d.max)] {
                    if let Bound::Derived(v) = b {
                        assert!(
                            v.abs() > 1e-6,
                            "plan `{}` joint `{}` {end} about {} derived {v} — the L1 \
                             abutting-box degeneracy is back",
                            plan.name,
                            j.segment,
                            d.axis.name()
                        );
                    }
                }
            }
        }
    }
}

/// A plan whose segments declare roles the derivation reads (a sole anchor) must
/// still derive — and the anchor participates as a tested point without ever
/// binding before the box corner it sits on. A guard on the "declared contact
/// anchor" half of the law.
#[test]
fn a_declared_anchor_never_binds_before_its_own_box() {
    let plan = derived_only(biped_plan());
    let with_anchor = bound_of(&plan, "leg_l_lower", Axis::X, true)
        .value()
        .unwrap();
    let mut stripped = plan.clone();
    for s in &mut stripped.segments {
        s.roles.retain(|r: &RoleDef| r.at_m.is_none());
    }
    let without = bound_of(&stripped, "leg_l_lower", Axis::X, true)
        .value()
        .unwrap();
    assert!(
        (with_anchor - without).abs() <= 1e-12,
        "the sole anchor sits on the shank's bottom face, inside the corner that \
         binds — it must not move the bound ({with_anchor} vs {without})"
    );
}

/// A mode-less plan (a tree) still derives limits: the derivation is a function
/// of GEOMETRY, and has nothing to do with locomotion.
#[test]
fn a_plan_with_no_modes_still_derives() {
    let mut plan = biped_plan();
    plan.modes = Vec::<ModeDef>::new();
    plan.actions.clear();
    validate_plan(&plan, clip_lookup(&[])).expect("a tree defines");
    let limits = derive_joint_limits(&plan);
    assert_eq!(limits.joints.len(), plan.segments.len());
}

/// A keyframe that OMITS a joint is identity there, and identity is inside every
/// range by the validator's own `min <= 0 <= max` invariant — so partial
/// keyframes cannot be rejected. Guards the one way check 3 could have become a
/// false positive.
#[test]
fn an_omitted_joint_is_never_out_of_range() {
    // A one-ended declaration that admits every shipped angle, so the only
    // thing this test can fail on is the empty keyframe.
    let mut plan = biped_plan();
    for s in &mut plan.segments {
        if s.name.starts_with("leg_") && s.name.ends_with("_lower") {
            s.dofs = Some(vec![DofDef {
                axis: Axis::X,
                min_rad: Some(-3.4),
                max_rad: None,
            }]);
        }
    }
    let mut clips = biped_clips();
    // A keyframe naming nothing at all.
    clips
        .iter_mut()
        .find(|c| c.name == "dc:anim/biped_idle")
        .unwrap()
        .keyframes
        .push(Keyframe {
            t: 1.5,
            rotations: Vec::new(),
        });
    clips
        .iter_mut()
        .find(|c| c.name == "dc:anim/biped_idle")
        .unwrap()
        .keyframes
        .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    validate_plan(&plan, clip_lookup(&clips)).expect("an omitted joint is identity, and legal");
}
