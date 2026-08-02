//! The resting-posture bake's acceptance set — the audit's § 7 tests
//! (`docs/audits/2026-08-02-posture-bake-member0-design.md`), all
//! invariants or derivations, never snapshots. The numeric reports print
//! under `--nocapture`; the design pass's predicted table is what the
//! measured run is checked against, and a divergence is a finding about
//! that document (immutable body, mutable header).

use super::super::default_pack::seg;
use super::super::{ModeDef, RoleDef, biped_plan, longleg_plan, stout_plan};
use super::*;

fn baked(plan: &BodyPlan, mode: &str) -> RestingPosture {
    match bake_resting_posture(plan, mode) {
        BakeOutcome::Baked(p) => p,
        other => panic!("expected Baked for `{}`, got {other:?}", plan.name),
    }
}

fn unsupported(plan: &BodyPlan, mode: &str) -> String {
    match bake_resting_posture(plan, mode) {
        BakeOutcome::Unsupported { reason } => reason,
        other => panic!("expected Unsupported for `{}`, got {other:?}", plan.name),
    }
}

fn by<'a>(plan: &'a BodyPlan, name: &str) -> &'a SegmentDef {
    plan.segments.iter().find(|s| s.name == name).unwrap()
}

/// Standing height at the rest pose: the top face of the highest box,
/// root at the derived height (identity angles, so translation-only).
fn stature_m(plan: &BodyPlan, root_height_m: f64) -> f64 {
    plan.segments
        .iter()
        .map(|s| {
            let base = offset_from_root(plan, s);
            root_height_m + base[1] + s.offset_m[1] + s.size_m[1] / 2.0
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// The ratified acceptance, verbatim (ROADMAP first-slice spec), plus
/// the design pass's predicted table: biped hip 0.880 m, stout
/// 0.440 m, longleg 1.020 m — longleg stands TALLER than the biped
/// with straight legs, against journal/0131's measured −56.2° squat
/// under the pinned hip.
#[test]
fn longleg_stands_taller_than_the_biped() {
    let bip = baked(&biped_plan(), "stand");
    let sto = baked(&stout_plan(), "stand");
    let lon = baked(&longleg_plan(), "stand");
    assert!(
        (bip.root_height_m - 0.880).abs() < 1e-9,
        "{}",
        bip.root_height_m
    );
    assert!(
        (sto.root_height_m - 0.440).abs() < 1e-9,
        "{}",
        sto.root_height_m
    );
    assert!(
        (lon.root_height_m - 1.020).abs() < 1e-9,
        "{}",
        lon.root_height_m
    );
    assert!(
        lon.root_height_m > bip.root_height_m,
        "longleg must stand taller, not squat: {} vs {}",
        lon.root_height_m,
        bip.root_height_m
    );
}

/// The ROADMAP hazard, encoded as an assertion. WHY zero is asserted
/// as RIGHT: for a symmetric standing biped whose leg chains hang
/// straight down, the configuration in which gravity exerts zero
/// moment at every joint — the exact minimiser of static joint
/// torque — IS the vertical column, i.e. the identity rotation.
/// "Legs straight, hip at leg reach" is THE RIGHT ANSWER and must not
/// be "fixed" into bent knees out of suspicion of degeneracy: a later
/// session wanting a folded rest (the bird) is looking for the
/// effort-term heir (audit § 2.4), not for this test to change.
#[test]
fn straight_legs_are_the_answer() {
    for plan in [biped_plan(), stout_plan(), longleg_plan()] {
        let p = baked(&plan, "stand");
        assert_eq!(p.chains.len(), 2, "two stance chains on `{}`", plan.name);
        for chain in &p.chains {
            assert_eq!(chain.joints.len(), 2, "two-bone chains on `{}`", plan.name);
            for j in &chain.joints {
                assert_eq!(
                    j.euler, [0.0; 3],
                    "plan `{}` joint `{}`: every stance angle is exactly 0.0",
                    plan.name, j.segment
                );
            }
        }
    }
}

/// The user's no-artificial-gap ruling as an executable invariant
/// (posture-gait § 8: load-bearing segments have no artificial gap
/// between mesh and ground). With the root at the derived height,
/// every declared sole anchor sits at ground level within 1e-9 m —
/// this kills the +0.020 m error class structurally. The baked angles
/// are all zero (asserted above), so translation-only accumulation IS
/// the posed chain.
#[test]
fn no_resting_gap_between_sole_and_ground() {
    for plan in [biped_plan(), stout_plan(), longleg_plan()] {
        let p = baked(&plan, "stand");
        for s in segments_with_role(&plan, "sole") {
            let anchor = s
                .roles
                .iter()
                .find(|r| r.role == "sole")
                .and_then(|r| r.at_m)
                .expect("shipped soles are anchored");
            let base = offset_from_root(&plan, s);
            let y = p.root_height_m + base[1] + anchor[1];
            assert!(
                y.abs() <= 1e-9,
                "plan `{}` sole `{}` rests {y} m off the ground",
                plan.name,
                s.name
            );
        }
    }
}

/// The inversion itself, as a test: the authored `trunk.pivot_m[1]`
/// is a rest origin, not a hip (body-plan-structure § 5.1) — the bake
/// excludes it from the solve inputs, so mutating it changes nothing.
#[test]
fn hip_height_is_an_output() {
    let baseline = baked(&biped_plan(), "stand");
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "trunk")
        .unwrap()
        .pivot_m[1] = 5.0;
    assert_eq!(
        baked(&plan, "stand"),
        baseline,
        "the authored root pivot must not reach the baked output"
    );
}

/// Decision 24's ratios-not-metres, falsifiable: a uniformly ×2.5
/// scaled plan bakes identical angles and an identical
/// `root_height_ratio`; the metres report scales with the body.
#[test]
fn bake_is_scale_invariant() {
    const K: f64 = 2.5;
    let plan = biped_plan();
    let mut scaled = plan.clone();
    for s in &mut scaled.segments {
        for v in &mut s.pivot_m {
            *v *= K;
        }
        for v in &mut s.size_m {
            *v *= K;
        }
        for v in &mut s.offset_m {
            *v *= K;
        }
        for r in &mut s.roles {
            if let Some(at) = &mut r.at_m {
                for v in at {
                    *v *= K;
                }
            }
        }
    }
    let a = baked(&plan, "stand");
    let b = baked(&scaled, "stand");
    assert_eq!(
        a.root_height_ratio, b.root_height_ratio,
        "the ratio is scale-free"
    );
    assert_eq!(a.chains.len(), b.chains.len());
    for (ca, cb) in a.chains.iter().zip(&b.chains) {
        assert_eq!(
            ca.joints, cb.joints,
            "angles are invariant under uniform scaling"
        );
    }
    assert!(
        (b.root_height_m - K * a.root_height_m).abs() < 1e-9,
        "the metres report scales with the plan: {} vs {}",
        b.root_height_m,
        K * a.root_height_m
    );
}

/// B0's mode-less plan is legal and declares nothing about support —
/// a tree. The bake answers with a legal absence, not an error.
#[test]
fn mode_less_plan_is_a_legal_absence() {
    let mut plan = biped_plan();
    plan.modes.clear();
    assert_eq!(
        bake_resting_posture(&plan, "stand"),
        BakeOutcome::NothingDeclared
    );
}

/// A mode not declared BY NAME is refused, naming it.
#[test]
fn undeclared_mode_is_unsupported_by_name() {
    let reason = unsupported(&biped_plan(), "gallop");
    assert!(reason.contains("gallop"), "{reason}");
    assert!(
        reason.contains("stand"),
        "names the declared modes: {reason}"
    );
}

/// An unanchored bearing role — the whole segment bears, a snake's
/// belly / the alligator's sprawl — is a loud refusal naming role and
/// segment (audit § 7 test 7).
#[test]
fn distributed_bearing_is_loudly_unsupported() {
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "trunk")
        .unwrap()
        .roles
        .push(RoleDef {
            role: "belly".into(),
            at_m: None,
        });
    plan.modes.push(ModeDef {
        mode: "sprawl".into(),
        bearing: vec!["sole".into(), "belly".into()],
    });
    let reason = unsupported(&plan, "sprawl");
    assert!(reason.contains("belly"), "{reason}");
    assert!(reason.contains("trunk"), "{reason}");
    assert!(reason.contains("distributed"), "{reason}");
}

/// A snake-ish chain whose belly touches ground at several ANCHORED
/// points along a line is still a distributed bearing — anchors do
/// not make a spine-drag a stance. The refusal names the geometry.
#[test]
fn a_line_of_point_contacts_is_loudly_unsupported() {
    let grey = [0.5, 0.5, 0.5];
    let mut segments = vec![seg(
        "spine0",
        None,
        [0.0, 0.1, 0.0],
        [0.12, 0.1, 0.4],
        [0.0, 0.0, 0.0],
        grey,
    )];
    for i in 1..=4 {
        let mut s = seg(
            &format!("spine{i}"),
            Some(&format!("spine{}", i - 1)),
            [0.0, 0.0, 0.4],
            [0.12, 0.1, 0.4],
            [0.0, 0.0, 0.0],
            grey,
        );
        s.roles.push(RoleDef {
            role: "belly".into(),
            at_m: Some([0.0, -0.05, 0.0]),
        });
        segments.push(s);
    }
    let snake = BodyPlan {
        name: "dc:test/snake".into(),
        doc: String::new(),
        segments,
        modes: vec![ModeDef {
            mode: "slither".into(),
            bearing: vec!["belly".into()],
        }],
        actions: vec![],
    };
    let reason = unsupported(&snake, "slither");
    assert!(reason.contains("line"), "names the geometry: {reason}");
    assert!(reason.contains("belly"), "{reason}");
    assert!(reason.contains("spine1"), "names the segments: {reason}");
}

/// Unequal stance-chain reaches are refused, not solved — the
/// vertical-column answer is exact only for equal chains, and
/// pretending otherwise would silently pose the first quadruped wrong
/// (audit F4; heir: the first quadruped's design pass).
#[test]
fn unequal_stance_chains_are_refused() {
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "leg_l_lower")
        .unwrap()
        .pivot_m[1] = -0.55; // left reach 0.98 m vs right 0.88 m
    let reason = unsupported(&plan, "stand");
    assert!(reason.contains("unequal"), "{reason}");
    assert!(reason.contains("leg_l_lower"), "names the chains: {reason}");
    assert!(reason.contains("quadruped"), "names the heir: {reason}");
}

/// An empty active bearing set (the alligator's `swim []`) and a
/// single point contact are both out of the standing solve's domain.
#[test]
fn empty_or_single_bearing_sets_are_unsupported() {
    let mut plan = biped_plan();
    plan.modes.push(ModeDef {
        mode: "swim".into(),
        bearing: vec![],
    });
    let reason = unsupported(&plan, "swim");
    assert!(reason.contains("swim"), "{reason}");
    assert!(reason.contains("zero"), "{reason}");

    let mut one_leg = biped_plan();
    one_leg
        .segments
        .iter_mut()
        .find(|s| s.name == "leg_r_lower")
        .unwrap()
        .roles
        .clear();
    let reason = unsupported(&one_leg, "stand");
    assert!(reason.contains("at least 2"), "{reason}");
}

/// Two calls, `==` — pure f64 arithmetic over plan data, no maps, no
/// ambient state (audit § 7 test 9).
#[test]
fn bake_is_deterministic() {
    let plan = longleg_plan();
    assert_eq!(
        bake_resting_posture(&plan, "stand"),
        bake_resting_posture(&plan, "stand")
    );
}

/// The balance verdict holds for all three shipped plans, and the
/// report numbers print beside their anthropometric bands. Published
/// anthropometry puts a standing human's CoM near 0.55–0.57 of
/// stature (the biped from cuboids lands ≈ 0.545, just under) and leg
/// length near ~0.53 of stature. These are BAND CHECKS, not
/// calibrations — if a ratio sits outside its band, that is
/// information about cuboid proxies, never a licence to tune
/// (audit § 11 item 8; run with --nocapture to see the report).
#[test]
fn balance_holds_and_is_reported() {
    println!("plan | CoM (x,y,z) m | stature m | CoM/stature | hip/stature | hull x | hull z");
    for plan in [biped_plan(), stout_plan(), longleg_plan()] {
        let p = baked(&plan, "stand");
        assert!(
            p.com_in_support,
            "plan `{}`: CoM {:?} must sit strictly inside the support hull",
            plan.name, p.com_m
        );
        assert!(
            p.com_m[0].abs() < 1e-9 && p.com_m[2].abs() < 1e-9,
            "mirror symmetry is structural (B0): lateral/fore-aft CoM is 0, got {:?}",
            p.com_m
        );
        let stature = stature_m(&plan, p.root_height_m);
        // Hull extents recomputed here for the report — the hull is
        // deliberately not an exported field (A-4).
        let (mut x, mut z) = ([f64::MAX, f64::MIN], [f64::MAX, f64::MIN]);
        for s in segments_with_role(&plan, "sole") {
            let anchor = s
                .roles
                .iter()
                .find(|r| r.role == "sole")
                .unwrap()
                .at_m
                .unwrap();
            let base = offset_from_root(&plan, s);
            let (ax, az) = (base[0] + anchor[0], base[2] + anchor[2]);
            x = [
                x[0].min(ax - s.size_m[0] / 2.0),
                x[1].max(ax + s.size_m[0] / 2.0),
            ];
            z = [
                z[0].min(az - s.size_m[2] / 2.0),
                z[1].max(az + s.size_m[2] / 2.0),
            ];
        }
        println!(
            "{} | ({:.3}, {:.3}, {:.3}) | {stature:.3} | {:.3} | {:.3} | [{:.2}, {:.2}] | [{:.2}, {:.2}]",
            plan.name,
            p.com_m[0],
            p.com_m[1],
            p.com_m[2],
            p.com_m[1] / stature,
            p.root_height_m / stature,
            x[0],
            x[1],
            z[0],
            z[1]
        );
    }
}

/// The measured § 2.2 table, printed (audit § 7 test 10; run with
/// --nocapture). The design pass's predictions are what this run is
/// checked against — a divergence is a finding about THAT document
/// (immutable body, mutable header), recorded by whoever measures.
#[test]
fn report_the_numbers() {
    println!("plan | l1 | l2 | reach | authored hip | derived hip | knee | authored-derived");
    for plan in [biped_plan(), stout_plan(), longleg_plan()] {
        let p = baked(&plan, "stand");
        let lower = by(&plan, "leg_l_lower");
        let l1 = norm3(lower.pivot_m);
        let l2 = norm3(
            lower
                .roles
                .iter()
                .find(|r| r.role == "sole")
                .unwrap()
                .at_m
                .unwrap(),
        );
        let authored = by(&plan, "trunk").pivot_m[1];
        let knee_deg = p.chains[0].joints[0].euler[0].to_degrees();
        println!(
            "{} | {l1:.3} | {l2:.3} | {:.3} | {authored:.3} | {:.3} | {knee_deg:.1} deg | {:+.3}",
            plan.name,
            p.chains[0].reach_m,
            p.root_height_m,
            authored - p.root_height_m
        );
    }
}
