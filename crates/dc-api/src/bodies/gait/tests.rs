//! The gait bake's acceptance set — the design pass's § 10 tests
//! (`docs/audits/2026-08-02-gait-bake-member1-design.md`), invariants and
//! derivations, never snapshots. The numeric reports print under `--nocapture`;
//! the design pass's predicted tables are what the measured run is checked
//! against, and **a divergence is a finding about THAT document** (immutable
//! body, mutable header) recorded by whoever measures.
//!
//! Two gravities appear throughout and the distinction is load-bearing. The
//! design derived its whole table at `g = 9.81` (Earth, where the cited
//! literature was measured); this world's [`crate::CharacterConfig`] runs at
//! **25.0 m/s²**. Both are reported. The *structural* checks — dynamic
//! similarity and the cadence ratio — hold at either, which is the strongest
//! statement available: they are properties of the derivation, not of a number.

use super::super::default_pack::{seg, with_sole};
use super::super::{
    AnimClip, BodyPlan, JointAngle, ModeDef, RoleDef, SegmentDef, biped_plan, longleg_plan,
    retired_biped_walk_clip, stout_plan,
};
use super::*;

/// Earth, and the basis of every cited measurement in the literature these
/// tests check against. Stated as a literal on purpose: it is a **physical
/// fact**, not a world setting, and it must not follow the world if the world
/// changes.
const G_EARTH: f64 = 9.81;
/// **This world's** gravity — READ, never restated (`dc_core::world_constants`).
///
/// It was a hand-typed `25.0` when these tests were written hours earlier, and
/// it went stale the moment gravity became a world constant defaulting to Earth
/// (DECIDED 2026-08-02, `ARCHITECTURE.md`). A test that restates the world's
/// gravity is the same two-authority defect the constant exists to end — and
/// this one sat *inside the tests that measure gravity's effect*, printing
/// `Fr 0.9205` for a world that had already moved to `Fr 2.3457`.
///
/// **The two are equal today and that is the point, not a redundancy**: `G_EARTH`
/// asserts *what the literature was measured at*, `G_WORLD` asserts *what this
/// world runs at*. A world that sets Mars gravity separates them again without
/// touching a citation.
const G_WORLD: f64 = dc_core::DEFAULT_GRAVITY_M_S2;

fn baked(plan: &BodyPlan, g: f64) -> GaitVector {
    match bake_gait(plan, "stand", g, &GaitKnobs::default()) {
        GaitBakeOutcome::Baked(v) => v,
        other => panic!("expected Baked for `{}`, got {other:?}", plan.name),
    }
}

fn refused(plan: &BodyPlan, mode: &str) -> String {
    match bake_gait(plan, mode, G_WORLD, &GaitKnobs::default()) {
        GaitBakeOutcome::Unsupported { reason } => reason,
        other => panic!("expected Unsupported for `{}`, got {other:?}", plan.name),
    }
}

fn by<'a>(plan: &'a BodyPlan, name: &str) -> &'a SegmentDef {
    plan.segments.iter().find(|s| s.name == name).unwrap()
}

fn plans() -> [BodyPlan; 3] {
    [biped_plan(), stout_plan(), longleg_plan()]
}

fn rot_x(v: [f64; 3], a: f64) -> [f64; 3] {
    let (s, c) = a.sin_cos();
    [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
}

/// Forward-kinematic the contact anchor of a two-bone chain from its
/// attachment joint, given a contact-first pose. This is the independent
/// instrument: the bake solves, this walks the tree and checks.
fn anchor_from_attachment(plan: &BodyPlan, joints: &[JointAngle], anchor: [f64; 3]) -> [f64; 3] {
    let distal = by(plan, &joints[0].segment);
    let p = rot_x(anchor, joints[0].euler[0]);
    rot_x(
        [
            p[0] + distal.pivot_m[0],
            p[1] + distal.pivot_m[1],
            p[2] + distal.pivot_m[2],
        ],
        joints[1].euler[0],
    )
}

fn sole_anchor(plan: &BodyPlan, seg_name: &str) -> [f64; 3] {
    by(plan, seg_name)
        .roles
        .iter()
        .find(|r| r.role == "sole")
        .unwrap()
        .at_m
        .unwrap()
}

/// Every f64 in a gait vector, as raw bits — byte identity, not `==`
/// (which cannot tell `0.0` from `-0.0`).
fn bits(v: &GaitVector) -> Vec<u64> {
    let mut out = vec![
        v.gravity_m_s2.to_bits(),
        v.governing_reach_m.to_bits(),
        v.stride_coefficient_m.to_bits(),
        v.cadence_coefficient_hz.to_bits(),
    ];
    for l in &v.limbs {
        out.push(l.phase.to_bits());
        out.push(l.amplitude.to_bits());
        out.push(l.duty_bias.to_bits());
        out.push(l.reach_m.to_bits());
        out.push(l.compass_window.to_bits());
        for set in [&l.neutral, &l.contact_per_rad, &l.clearance] {
            for j in set {
                out.extend(j.euler.iter().map(|e| e.to_bits()));
            }
        }
    }
    out
}

// ------------------------------------------------- the structural checks --

/// **The bones' scaling law, falsifiable.** `f·√L` is constant across the three
/// plans at equal Fr — because `f = √(g/L)·Fr^0.2/2.3` and the `Fr^0.2` term
/// cancels. Asserted to 1e-12, not to the design table's printed places.
#[test]
fn cadence_scales_as_root_g_over_l() {
    for g in [G_EARTH, G_WORLD] {
        for fr in [0.05, 0.25, 0.4] {
            let mut invariant: Option<f64> = None;
            for plan in plans() {
                let v = baked(&plan, g);
                let k = v.cadence_hz(fr) * v.governing_reach_m.sqrt();
                match invariant {
                    None => invariant = Some(k),
                    Some(k0) => assert!(
                        (k - k0).abs() < 1e-12,
                        "plan `{}`: f·√L = {k} against {k0} (g = {g}, Fr = {fr})",
                        plan.name
                    ),
                }
            }
        }
    }
    // …and the ratio the design pass names, which is pure length scale.
    let (b, s, l) = (
        baked(&biped_plan(), G_WORLD),
        baked(&stout_plan(), G_WORLD),
        baked(&longleg_plan(), G_WORLD),
    );
    let fr = 0.25;
    let r_stout = s.cadence_hz(fr) / b.cadence_hz(fr);
    let r_biped = b.cadence_hz(fr) / l.cadence_hz(fr);
    assert!(
        (r_stout - (0.880_f64 / 0.440).sqrt()).abs() < 1e-12,
        "stout/biped cadence ratio {r_stout}"
    );
    assert!(
        (r_biped - (1.020_f64 / 0.880).sqrt()).abs() < 1e-12,
        "biped/longleg cadence ratio {r_biped}"
    );
}

/// **Alexander & Jayes, reproduced by our own arithmetic.** At equal Froude
/// number the excursion, the duty and the bob-to-reach ratio are *identical*
/// across all three bodies — only the metres differ. This is the strongest
/// available evidence that these outputs are ratios wearing metres, exactly as
/// decision 24 requires, and it holds at either gravity.
#[test]
fn dynamic_similarity_holds() {
    for g in [G_EARTH, G_WORLD] {
        for fr in [0.1, 0.25, 0.45] {
            let mut first: Option<(f64, f64, f64)> = None;
            for plan in plans() {
                let v = baked(&plan, g);
                let at = v.at(fr);
                let ratio = match at.root_height {
                    RootHeight::Derived {
                        amplitude_ratio, ..
                    } => amplitude_ratio,
                    RootHeight::Declined { .. } => panic!("Fr {fr} is a walk"),
                };
                let now = (at.theta_max_rad, at.mean_duty, ratio);
                match first {
                    None => first = Some(now),
                    Some(f) => {
                        assert!((now.0 - f.0).abs() < 1e-12, "θmax {now:?} vs {f:?}");
                        assert!((now.1 - f.1).abs() < 1e-12, "β {now:?} vs {f:?}");
                        assert!((now.2 - f.2).abs() < 1e-12, "Δ/L {now:?} vs {f:?}");
                    }
                }
            }
        }
    }
}

/// **A zero-parameter prediction against hand-typed data.** § 3.3's phase rule
/// consumes the sole set as a SET, derives an order from contact geometry, and
/// never names a limb — and it gives `{0.0, 0.5}`, which is exactly what the
/// authored walk clip measures on both bones. Rule 4's counter-phase arms hold
/// too. Nothing was fitted.
#[test]
fn phase_offsets_reproduce_the_authored_walk() {
    let plan = biped_plan();
    let v = baked(&plan, G_WORLD);
    let mut soles: Vec<f64> = v
        .limbs
        .iter()
        .filter(|l| l.bearing)
        .map(|l| l.phase)
        .collect();
    soles.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(soles.len(), 2);
    assert!(soles[0] == 0.0 && soles[1] == 0.5, "{soles:?}");

    // Rule 4: the derived counter-swing chains are the arms, each at the
    // CONTRALATERAL leg's phase — half a cycle from its own side's leg.
    let get = |n: &str| v.limbs.iter().find(|l| l.contact_segment == n).unwrap();
    for (arm, leg) in [
        ("arm_l_lower", "leg_l_lower"),
        ("arm_r_lower", "leg_r_lower"),
    ] {
        let (a, l) = (get(arm), get(leg));
        assert!(!a.bearing, "{arm} must not bear");
        assert!(
            ((a.phase - l.phase).rem_euclid(1.0) - 0.5).abs() < 1e-12,
            "{arm} at {} against {leg} at {}",
            a.phase,
            l.phase
        );
    }
    // The head is a MIDLINE appendage and gets no derived counter-swing:
    // carriage is taste, not geometry.
    assert!(v.limbs.iter().all(|l| l.contact_segment != "head"));

    // …and the authored clip measures the same phase structure: at every key,
    // on both leg bones, right = left half a cycle later.
    // The PARKED fixture (user call #3: retired as content, kept in the tree).
    let walk = retired_biped_walk_clip();
    for bone in ["upper", "lower"] {
        for kf in &walk.keyframes {
            let l = kf
                .rotations
                .iter()
                .find(|r| r.segment == format!("leg_l_{bone}"))
                .unwrap()
                .euler[0];
            let shifted = clip_angle(&walk, &format!("leg_r_{bone}"), kf.t + 0.5);
            assert!(
                (l - shifted).abs() < 1e-12,
                "leg_l_{bone}({}) = {l} but leg_r_{bone}(t+0.5) = {shifted}",
                kf.t
            );
        }
    }
}

/// Sample an authored clip's X rotation for one joint at time `t` (wrapping).
fn clip_angle(clip: &AnimClip, segment: &str, t: f64) -> f64 {
    let t = t.rem_euclid(clip.duration_s);
    let at = |kf: &super::super::Keyframe| {
        kf.rotations
            .iter()
            .find(|r| r.segment == segment)
            .map_or(0.0, |r| r.euler[0])
    };
    let mut prev = &clip.keyframes[0];
    for kf in &clip.keyframes {
        if kf.t >= t {
            if (kf.t - prev.t).abs() < 1e-12 {
                return at(kf);
            }
            let u = (t - prev.t) / (kf.t - prev.t);
            return at(prev) * (1.0 - u) + at(kf) * u;
        }
        prev = kf;
    }
    at(prev)
}

/// Decision 24's ratios-not-metres, falsifiable at two scales: a uniformly
/// scaled plan bakes identical angles, duty and every ratio; the metres scale.
#[test]
fn gait_is_scale_invariant() {
    for k in [0.4_f64, 2.5] {
        let plan = biped_plan();
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
                    for v in at {
                        *v *= k;
                    }
                }
            }
        }
        let (a, b) = (baked(&plan, G_WORLD), baked(&scaled, G_WORLD));
        let fr = 0.25;
        let (pa, pb) = (a.at(fr), b.at(fr));
        assert!((pa.theta_max_rad - pb.theta_max_rad).abs() < 1e-12);
        assert!((pa.mean_duty - pb.mean_duty).abs() < 1e-12);
        assert!((pb.stride_m - k * pa.stride_m).abs() < 1e-9 * k);
        assert!((pb.cadence_hz * k.sqrt() - pa.cadence_hz).abs() < 1e-9);
        for (la, lb) in a.limbs.iter().zip(&b.limbs) {
            assert_eq!(la.phase, lb.phase, "phases are scale-free");
            assert_eq!(la.compass_window, lb.compass_window);
            for (ja, jb) in la.clearance.iter().zip(&lb.clearance) {
                assert!(
                    (ja.euler[0] - jb.euler[0]).abs() < 1e-12,
                    "clearance angles are scale-free: {ja:?} vs {jb:?}"
                );
            }
        }
    }
}

// ------------------------------------------------- the honest refusals --

/// § 4.1's "including none", as it is actually reachable. **The design's test 5
/// asked for `exactly zero root-height variation` from a distributed bearer;
/// that is not implementable, because the resting solve refuses a distributed
/// bearing upstream** — and a loud refusal naming the geometry is the stronger
/// answer, not the weaker one. Recorded as a finding about the design pass.
#[test]
fn distributed_bearing_yields_no_gait_rather_than_a_fabricated_bob() {
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
    let reason = refused(&plan, "sprawl");
    assert!(reason.contains("distributed"), "{reason}");
    assert!(reason.contains("belly"), "{reason}");
    assert!(reason.contains("resting solve refused"), "{reason}");
}

/// A mode-less plan — a tree — is a legal absence, not an error.
#[test]
fn mode_less_plan_is_a_legal_absence() {
    let mut plan = biped_plan();
    plan.modes.clear();
    assert_eq!(
        bake_gait(&plan, "stand", G_WORLD, &GaitKnobs::default()),
        GaitBakeOutcome::NothingDeclared
    );
}

/// A three-bone leg is refused by NAME, with the heir stated — the mid-swing
/// clearance solve is a two-bone analytic IK and does not pretend otherwise.
#[test]
fn a_three_bone_chain_is_refused_loudly() {
    let plan = three_bone_biped();
    let reason = refused(&plan, "stand");
    assert!(
        reason.contains("three") || reason.contains("3 bones"),
        "{reason}"
    );
    assert!(reason.contains("two-bone"), "{reason}");
    assert!(reason.contains("B7"), "names the heir: {reason}");
}

/// The biped with a shank inserted in each leg: same 0.88 m reach, same
/// vertical column, three bones instead of two.
fn three_bone_biped() -> BodyPlan {
    const LIMB: [f32; 3] = [0.35, 0.4, 0.55];
    let mut plan = biped_plan();
    for side in ["l", "r"] {
        let (lower, mid, upper) = (
            format!("leg_{side}_lower"),
            format!("leg_{side}_mid"),
            format!("leg_{side}_upper"),
        );
        let slot = plan
            .segments
            .iter_mut()
            .find(|s| s.name == lower)
            .expect("the shipped biped has both lower legs");
        // 0.45 (hip→mid) + 0.20 (mid→ankle) + 0.23 (ankle→sole) = 0.88 m,
        // the biped's own reach, so member #0 still accepts the plan and the
        // ONLY thing under test is the bone count.
        *slot = with_sole(seg(
            &lower,
            Some(&mid),
            [0.0, -0.2, 0.0],
            [0.16, 0.23, 0.18],
            [0.0, -0.115, 0.0],
            LIMB,
        ));
        plan.segments.push(seg(
            &mid,
            Some(&upper),
            [0.0, -0.45, 0.0],
            [0.16, 0.2, 0.18],
            [0.0, -0.1, 0.0],
            LIMB,
        ));
    }
    plan
}

/// § 3.5's flag, executable — and it fires at BOTH gravities. At the shipped
/// world-global 4.5 m/s every shipped plan sits above the published walk/run
/// transition, so the bake reports out of band and **declines the root
/// height** rather than fabricating a ballistic apex it has no force term for.
#[test]
fn run_regime_is_declined_loudly() {
    for g in [G_EARTH, G_WORLD] {
        for plan in plans() {
            let v = baked(&plan, g);
            let at = v.at(v.froude(4.5));
            assert_eq!(at.regime, Regime::Run, "plan `{}` at g = {g}", plan.name);
            assert!(
                at.mean_duty < 0.5,
                "a run has β < 0.5, got {}",
                at.mean_duty
            );
            let RootHeight::Declined { reason } = &at.root_height else {
                panic!("plan `{}` must decline the run bob at g = {g}", plan.name);
            };
            assert!(reason.contains("FLIGHT PHASE"), "{reason}");
            assert!(reason.contains("B6"), "names the heir: {reason}");
            assert!(reason.contains(&format!("{:.4}", at.froude)), "{reason}");
            assert!(v.root_height_ratio_at(at.froude, 0.3).is_none());
        }
    }
}

// -------------------------------------------------- the derived geometry --

/// **The measured 0.5 s against a 1.0 s clip, DERIVED rather than typed.** The
/// compass windows tile the cycle, so `h(p)` has exactly one trough per bearing
/// contact — two for a biped, at twice the limb cycle — and its extremes are
/// `L` and `L·cos θmax` exactly. Nothing is authored and there is no toggle.
#[test]
fn bob_has_one_trough_per_bearing_contact() {
    const N: usize = 20_000;
    for plan in plans() {
        let v = baked(&plan, G_WORLD);
        let fr = 0.25;
        let h: Vec<f64> = (0..N)
            .map(|i| v.root_height_ratio_at(fr, i as f64 / N as f64).unwrap())
            .collect();
        let troughs = (0..N)
            .filter(|i| {
                let (a, b, c) = (h[(i + N - 1) % N], h[*i], h[(i + 1) % N]);
                b < a && b < c
            })
            .count();
        assert_eq!(
            troughs,
            v.limbs.iter().filter(|l| l.bearing).count(),
            "plan `{}` root height has {troughs} troughs",
            plan.name
        );
        let theta = v.at(fr).theta_max_rad;
        let (lo, hi) = h
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
        assert!(
            (hi - 1.0).abs() < 1e-12,
            "midstance is the full column: {hi}"
        );
        assert!(
            (lo - theta.cos()).abs() < 1e-12,
            "the trough is L·cos θmax: {lo}"
        );
    }
}

/// The mid-swing solve CLOSES: the contact anchor sits directly under the
/// attachment joint at exactly the clearance height. Asserted by independent
/// forward kinematics, not by the angles the solver produced.
#[test]
fn clearance_pose_puts_the_anchor_under_the_attachment() {
    for plan in plans() {
        for flexion in [0.0_f64, 0.35, 0.9] {
            let knobs = GaitKnobs {
                swing_flexion: flexion,
                ..Default::default()
            };
            let GaitBakeOutcome::Baked(v) = bake_gait(&plan, "stand", G_WORLD, &knobs) else {
                panic!("`{}` bakes", plan.name)
            };
            for limb in v.limbs.iter().filter(|l| l.bearing) {
                let anchor = sole_anchor(&plan, &limb.contact_segment);
                let p = anchor_from_attachment(&plan, &limb.clearance, anchor);
                let floor = knobs.foot_clearance_ratio * limb.reach_m;
                // p is the anchor relative to the attachment joint, so −p.y is
                // the distance down to it and `reach − that` is the foot lift.
                let lift = limb.reach_m + p[1];
                assert!(
                    p[0].abs() < 1e-12 && p[2].abs() < 1e-12,
                    "under the hip: {p:?}"
                );
                assert!(
                    lift >= floor - 1e-12,
                    "plan `{}` lifted {lift:.4} m, floor {floor:.4}",
                    plan.name,
                );
                if flexion == 0.0 {
                    assert!(
                        (lift - floor).abs() < 1e-12,
                        "identity swing_flexion is the published clearance exactly, got {lift}"
                    );
                }
                // The knee bends FORWARD and the shank never hyperextends past
                // the thigh — the defect G1 exists to prevent.
                assert!(limb.clearance[1].euler[0] > 0.0, "thigh forward");
                assert!(
                    limb.clearance[0].euler[0] < 0.0,
                    "knee flexed, not inverted"
                );
            }
        }
    }
}

/// **G2, asserted.** `contact` is not a frame frozen at one speed: the stored
/// gradient is speed-invariant and the pose is `neutral + θmax(Fr)·gradient`,
/// exactly. Two different Froude numbers give two different contact poses from
/// one bake.
#[test]
fn contact_is_a_coefficient_not_a_frozen_frame() {
    let plan = biped_plan();
    let v = baked(&plan, G_WORLD);
    let limb = v.limbs.iter().find(|l| l.bearing).unwrap();
    let top = limb.contact_per_rad.len() - 1;
    assert_eq!(limb.contact_per_rad[top].euler, [1.0, 0.0, 0.0]);
    for j in &limb.contact_per_rad[..top] {
        assert_eq!(j.euler, [0.0; 3]);
    }
    let (a, b) = (v.contact_pose(limb, 0.1), v.contact_pose(limb, 0.4));
    assert!(
        a[top].euler[0] < b[top].euler[0],
        "faster = longer excursion"
    );
    assert!(
        (b[top].euler[0] - (limb.neutral[top].euler[0] + v.theta_max_rad(0.4))).abs() < 1e-15,
        "contact = neutral + θmax·gradient, exactly"
    );
    // …and the traversal is SIGNED (G6): +amplitude at touchdown, −amplitude
    // at liftoff, through zero at midstance.
    assert!((v.traversal(limb, limb.phase).0 - 1.0).abs() < 1e-12);
    let mid = limb.phase + limb.compass_window / 2.0;
    assert!(v.traversal(limb, mid).0.abs() < 1e-12);
    let off = limb.phase + limb.compass_window - 1e-9;
    assert!(v.traversal(limb, off).0 < -0.999_999);
}

/// **G1's three keyframes, composed.** The sampled pose IS `contact` at
/// touchdown, `neutral` at midstance, `−contact` at liftoff and `clearance` at
/// mid-swing — the user's between-the-keyframes formulation preserved verbatim,
/// with three stored poses instead of two. And the defect G1 exists to prevent
/// is asserted directly: **the knee never inverts anywhere in the cycle**, which
/// a naive ping-pong between two poses cannot promise.
#[test]
fn the_three_keyframes_compose_the_cycle() {
    let plan = biped_plan();
    let v = baked(&plan, G_WORLD);
    let fr = 0.25;
    let limb = v.limbs.iter().find(|l| l.bearing).unwrap();
    let same = |a: &[JointAngle], b: &[JointAngle], what: &str| {
        for (x, y) in a.iter().zip(b) {
            assert!(
                (x.euler[0] - y.euler[0]).abs() < 1e-12,
                "{what}: {x:?} vs {y:?}"
            );
        }
    };
    let w = limb.compass_window;
    same(
        &v.limb_pose(limb, fr, limb.phase),
        &v.contact_pose(limb, fr),
        "touchdown is +contact",
    );
    same(
        &v.limb_pose(limb, fr, limb.phase + w / 2.0),
        &limb.neutral,
        "midstance is neutral",
    );
    same(
        &v.limb_pose(limb, fr, limb.phase + (1.0 + w) / 2.0),
        &limb.clearance,
        "mid-swing is clearance",
    );
    // Sweep the whole cycle: the knee (the distal joint of a contact-first
    // chain) never crosses into hyperextension.
    for i in 0..2000 {
        let p = i as f64 / 2000.0;
        let posed = v.limb_pose(limb, fr, p);
        assert!(
            posed[0].euler[0] <= 1e-12,
            "knee inverted at phase {p}: {:?}",
            posed[0]
        );
    }
}

// --------------------------------------------------- the instance seam --

/// **S-5, byte for byte.** The identity instance delta reproduces the species
/// gait bit for bit — not `==`, which cannot tell `0.0` from `-0.0`.
#[test]
fn identity_instance_delta_is_byte_identical() {
    for plan in plans() {
        let v = baked(&plan, G_WORLD);
        let posed = pose(&v, &InstanceDelta::default());
        assert_eq!(bits(&v), bits(&posed), "plan `{}`", plan.name);
        assert_eq!(v, posed);
    }
}

/// A 3 % limp is expressible and **mean-preserving**: `β̄` is what Fr pins, so
/// cadence and stride are unchanged and the limp reads as timing, not speed.
/// No quantization anywhere (bones § 6 property 3).
#[test]
fn a_three_percent_limp_is_expressible_and_mean_preserving() {
    let plan = biped_plan();
    let v = baked(&plan, G_WORLD);
    let limbs: Vec<String> = v
        .limbs
        .iter()
        .filter(|l| l.bearing)
        .map(|l| l.contact_segment.clone())
        .collect();
    let limped = pose(
        &v,
        &InstanceDelta {
            duty_bias: vec![(limbs[0].clone(), 0.03)],
            ..Default::default()
        },
    );
    let fr = 0.25;
    let (base, hurt) = (v.at(fr), limped.at(fr));
    assert_eq!(base.cadence_hz, hurt.cadence_hz, "a limp is not a speed");
    assert_eq!(base.stride_m, hurt.stride_m);
    let duties: Vec<f64> = hurt
        .limbs
        .iter()
        .filter(|l| l.bearing)
        .map(|l| l.duty)
        .collect();
    let mean = duties.iter().sum::<f64>() / duties.len() as f64;
    assert!(
        (mean - base.mean_duty).abs() < 1e-15,
        "mean preserved: {mean} vs {}",
        base.mean_duty
    );
    let spread = duties.iter().cloned().fold(f64::MIN, f64::max)
        - duties.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        (spread - 0.03).abs() < 1e-15,
        "3 % survives intact: {spread}"
    );
}

/// Two calls, `==` — pure f64 arithmetic over plan data, no maps, no ambient
/// state, no clock.
#[test]
fn bake_is_deterministic() {
    let plan = longleg_plan();
    assert_eq!(
        bake_gait(&plan, "stand", G_WORLD, &GaitKnobs::default()),
        bake_gait(&plan, "stand", G_WORLD, &GaitKnobs::default())
    );
}

/// A stand-in knob outside its published band is **reported and proceeds** —
/// a clockwork golem may want to step wrong (the docket's (b)3).
#[test]
fn an_out_of_band_knob_reports_and_proceeds() {
    let knobs = GaitKnobs {
        cadence_scale: 3.0,
        bob_damping: 0.2,
        ..Default::default()
    };
    let GaitBakeOutcome::Baked(v) = bake_gait(&biped_plan(), "stand", G_WORLD, &knobs) else {
        panic!("out of band must not refuse")
    };
    assert!(
        v.notes.iter().any(|n| n.contains("cadence_scale")),
        "{:?}",
        v.notes
    );
    assert!(
        v.notes.iter().any(|n| n.contains("bob_damping")),
        "{:?}",
        v.notes
    );
    assert!(
        v.notes.iter().all(|n| n.contains("B6")),
        "each names its heir"
    );
    // …and the knob actually rode: 3× the cadence, and a damped bob.
    let base = baked(&biped_plan(), G_WORLD);
    assert!((v.cadence_hz(0.25) / base.cadence_hz(0.25) - 3.0).abs() < 1e-12);
}

#[cfg(test)]
mod report;
