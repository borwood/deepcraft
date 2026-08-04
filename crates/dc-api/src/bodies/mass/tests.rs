//! The mass integral's acceptance set (B6-a) — invariants and derivations,
//! never snapshots. The one snapshot-shaped thing this slice owes is the
//! **byte-identity of an existing output**, and it is asserted as an identity
//! between two live computations (the bake's CoM against the integral's), not
//! against a recorded magnitude that a colleague's improvement would move.
//!
//! Reports print under `--nocapture`; the design pass
//! (`docs/audits/2026-08-03-b6-body-composition-design.md` § 10, P1–P12) is
//! what the measured run is checked against, and **a divergence is a finding
//! about that document** (immutable body, mutable header).

use super::super::{
    BakeOutcome, BodyPlan, RestingPosture, SegmentDef, bake_resting_posture, biped_plan,
    longleg_plan, stout_plan,
};
use super::*;

/// Earth gravity. The pendulum rows exist to be compared with the *literature*
/// (Mochon & McMahon's ballistic walking), which is measured on Earth; this
/// world's `DEFAULT_GRAVITY_M_S2` is 25.0 and would make the comparison
/// meaningless. Same convention, and the same reason, as
/// `bodies/gait/tests.rs`'s `G_EARTH`.
const G_EARTH: f64 = 9.81;

fn plans() -> [BodyPlan; 3] {
    [biped_plan(), stout_plan(), longleg_plan()]
}

fn baked(plan: &BodyPlan) -> RestingPosture {
    match bake_resting_posture(plan, "stand") {
        BakeOutcome::Baked(p) => p,
        other => panic!("expected Baked for `{}`, got {other:?}", plan.name),
    }
}

fn index_of(plan: &BodyPlan, name: &str) -> usize {
    plan.segments
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("plan `{}` has no segment `{name}`", plan.name))
}

fn by<'a>(plan: &'a BodyPlan, name: &str) -> &'a SegmentDef {
    &plan.segments[index_of(plan, name)]
}

/// Relative difference, with an absolute fallback at zero. For quantities
/// whose own magnitude IS their scale — a mass, a frequency.
fn rel(a: f64, b: f64) -> f64 {
    let scale = a.abs().max(b.abs());
    if scale == 0.0 {
        0.0
    } else {
        (a - b).abs() / scale
    }
}

/// Difference of two **lengths**, measured against the body's own scale.
///
/// ⚠ A CoM coordinate is a position, and the meaningful scale for *"did it
/// move"* is the BODY — never the coordinate's own magnitude. On a
/// mirror-symmetric plan the lateral and fore-aft axes are **exact
/// cancellation residuals**: measured at this build, the biped's `com_m[0]` is
/// **−8.2e-19 m** and the stout's **+9.9e-19 m**, and their SIGN flips freely
/// under a reordering or a uniform rescaling of the densities, because which
/// of `+0.33·V` and `−0.33·V` lands in the accumulator first decides the last
/// ulp of a sum whose true value is zero.
///
/// A plain relative comparison there returns **1.0** and asks whether a
/// rounding residual kept its sign, which is not a fact about the integral —
/// it is the A-3 shape inverted (a RED for a reason unrelated to the claim).
/// The bake's own acceptance already reads these axes the same way
/// (`bake/tests.rs`: `com_m[0].abs() < 1e-9`, mirror symmetry is structural).
///
/// The floor is **derived, not chosen**: it is the body's own root height, the
/// length the CoM is a position within.
fn moved(a: f64, b: f64, body_scale_m: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(body_scale_m)
}

/// Free-swing angular frequency `ω = √(g M d / I)` of the left leg subtree
/// about the hip, sagittal (X) axis, built from the per-segment masses the
/// integral returns.
///
/// **TEST-LOCAL ON PURPOSE.** `subtree_inertia` was NOT shipped in this slice:
/// the design pass § 4 makes it conditional on the same slice deriving
/// `cadence_scale`, which this one does not, and a public function whose only
/// caller is a test is A-4 with a nicer signature. This is the acceptance
/// instrument for the ρ-cancellation property — its consumer is the assertion
/// three functions down, and it exists nowhere else.
///
/// Parallel-axis over two boxes: `I = Σ [ mᵢ(yᵢ² + zᵢ²)/12 + mᵢ dᵢ² ]`, with
/// `dᵢ` measured from the hip pivot down to each box centre.
fn leg_omega(plan: &BodyPlan, mp: &MassProperties, g: f64) -> f64 {
    let (mut m_sum, mut md, mut i_hip) = (0.0_f64, 0.0_f64, 0.0_f64);
    for name in ["leg_l_upper", "leg_l_lower"] {
        let s = by(plan, name);
        let m = mp.segment_mass_kg[index_of(plan, name)];
        // Distance from the hip joint down to the box centre. The thigh
        // pivots AT the hip; the shank's own pivot is a thigh-length below it.
        let d = if name == "leg_l_upper" {
            -s.offset_m[1]
        } else {
            -(s.pivot_m[1] + s.offset_m[1])
        };
        m_sum += m;
        md += m * d;
        i_hip += m * (s.size_m[1] * s.size_m[1] + s.size_m[2] * s.size_m[2]) / 12.0 + m * d * d;
    }
    let d = md / m_sum;
    (g * m_sum * d / i_hip).sqrt()
}

// ---------------------------------------------------------------- S-5 ------

/// The seam's identity is exactly 1.0, one per segment. Asserted as bits:
/// `v * 1.0 == v` is what makes the whole conversion byte-identical, and it
/// holds only for a density that is *exactly* one.
#[test]
fn the_density_seam_is_the_identity_until_b6c() {
    for plan in plans() {
        let d = segment_densities(&plan);
        assert_eq!(d.len(), plan.segments.len(), "plan `{}`", plan.name);
        assert!(
            d.iter().all(|&r| r.to_bits() == 1.0_f64.to_bits()),
            "plan `{}`: the identity density is exactly 1.0, got {d:?}",
            plan.name
        );
    }
}

/// At the identity density, mass IS the volume proxy — per segment and in the
/// whole-body total, **bit for bit**. This is `stubs.md` #40 stated as an
/// equation rather than as a comment, and it is the arithmetic half of the
/// S-5 acceptance.
#[test]
fn at_the_identity_density_mass_is_the_volume_proxy_bit_for_bit() {
    for plan in plans() {
        let p = baked(&plan);
        let mp = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);
        assert_eq!(
            mp.mass_kg.to_bits(),
            mp.volume_m3.to_bits(),
            "plan `{}`: {} kg vs {} m³",
            plan.name,
            mp.mass_kg,
            mp.volume_m3
        );
        assert_eq!(
            mp.bulk_density_kg_m3().to_bits(),
            1.0_f64.to_bits(),
            "plan `{}`: the identity bulk density is exactly 1.0",
            plan.name
        );
        for (s, m) in plan.segments.iter().zip(&mp.segment_mass_kg) {
            let v = s.size_m[0] * s.size_m[1] * s.size_m[2];
            assert_eq!(
                m.to_bits(),
                v.to_bits(),
                "plan `{}` segment `{}`",
                plan.name,
                s.name
            );
        }
    }
}

// ---------------------------------------------------------------- S-3 ------

/// **The re-housing's acceptance.** `bake_resting_posture`'s centre of mass
/// equals `mass_properties(..).com_m` **bit for bit**, for every shipped plan.
///
/// S-3: the bake is a *consumer that re-derives*, never a copy that can
/// diverge — the exact failure `root_bob_m` cost us twice (corrections
/// #80, #93). The day this test fails is the day two authorities exist.
#[test]
fn the_com_loop_has_one_authority() {
    for plan in plans() {
        let p = baked(&plan);
        let mp = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);
        for (axis, (a, b)) in p.com_m.iter().zip(mp.com_m).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "plan `{}` axis {axis}: bake {a:e} vs integral {b:e}",
                plan.name
            );
        }
    }
}

// ------------------------------------------------------- the invariance ----

/// **A UNIFORM density factors out of every animation-side output** — the
/// property that makes this an S-5 conversion and that makes *heterogeneity*,
/// not magnitude, the whole future payoff of B6.
///
/// ρ ∈ {1, 600, 1010, 16000} — the design pass's sweep plus the brief's 600
/// (a hollow-boned flier): centre of mass, root-height ratio and a limb's
/// free-swing frequency all agree to 1e-12 relative.
///
/// *Exact byte-identity holds only at ρ = 1.0, where `v * 1.0 == v`; above it
/// the algebra cancels but the floating-point rounding does not, and a test
/// asserting byte-identity there would be asserting an untruth.*
#[test]
fn uniform_density_is_animation_invariant() {
    for plan in plans() {
        let p = baked(&plan);
        let reference = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);
        let w0 = leg_omega(&plan, &reference, G_EARTH);
        for rho in [600.0_f64, 1010.0, 16000.0] {
            let d = vec![rho; plan.segments.len()];
            let mp = mass_properties(&plan, &d, p.root_height_m);
            for (axis, (a, b)) in reference.com_m.iter().zip(mp.com_m).enumerate() {
                assert!(
                    moved(*a, b, p.root_height_m) <= 1e-12,
                    "plan `{}` axis {axis} at ρ = {rho}: CoM {b:.17} vs {a:.17}",
                    plan.name
                );
            }
            let w = leg_omega(&plan, &mp, G_EARTH);
            assert!(
                rel(w0, w) <= 1e-12,
                "plan `{}` at ρ = {rho}: ω {w:.17} vs {w0:.17} — ρ must cancel out of \
                 √(gMd/I) completely",
                plan.name
            );
            // Volume is geometry and cannot move at all.
            assert_eq!(
                mp.volume_m3.to_bits(),
                reference.volume_m3.to_bits(),
                "plan `{}` at ρ = {rho}: volume is geometry",
                plan.name
            );
        }
        // `root_height_ratio` never sees a density in the first place — it is
        // derived before the integral is called. Stated so the claim "every
        // animation-side output is ρ-invariant" is checked and not assumed.
        assert_eq!(p.root_height_ratio.to_bits(), 1.0_f64.to_bits());
    }
}

/// Mass is EXTENSIVE and the centre of mass is INTENSIVE: scale every density
/// by k and the mass scales by k while the CoM does not move. Separates the
/// two so a future heterogeneity bug cannot hide behind a uniform scaling.
#[test]
fn mass_scales_linearly_and_com_does_not() {
    for plan in plans() {
        let p = baked(&plan);
        let base = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);
        for k in [2.0_f64, 917.5, 1e-6] {
            let d: Vec<f64> = segment_densities(&plan).iter().map(|r| r * k).collect();
            let mp = mass_properties(&plan, &d, p.root_height_m);
            assert!(
                rel(mp.mass_kg, base.mass_kg * k) <= 1e-12,
                "plan `{}` at k = {k}: mass {} vs {}",
                plan.name,
                mp.mass_kg,
                base.mass_kg * k
            );
            for (axis, (a, b)) in base.com_m.iter().zip(mp.com_m).enumerate() {
                assert!(
                    moved(*a, b, p.root_height_m) <= 1e-12,
                    "plan `{}` axis {axis} at k = {k}: CoM moved ({a:.17} → {b:.17})",
                    plan.name
                );
            }
        }
    }
}

/// **The integral is ORDER-INVARIANT.** Permute the (segment, density) pairs
/// and the mass and centre of mass are unchanged: the answer depends on the
/// *pairing*, never on the sequence.
///
/// This pins as a property what the design pass § 3.3 argues in prose — *"a
/// sum is order-free"* — which is the premise its ≤ 2.6 % bound on radial
/// ordering rests on. The day an ordering *does* matter to the integral, this
/// is the test that fails and says so.
///
/// Asserted to 1e-12 relative rather than bit for bit, because f64 addition is
/// not associative: reordering the summands legitimately changes the last
/// ulps, and asserting otherwise would be asserting an untruth.
#[test]
fn the_integral_is_order_invariant() {
    for plan in plans() {
        let p = baked(&plan);
        let d = segment_densities(&plan);
        let base = mass_properties(&plan, &d, p.root_height_m);

        // A heterogeneous, position-dependent density so the permutation is
        // actually carrying information — with a uniform density every
        // permutation is trivially the same multiset.
        let hetero: Vec<f64> = (0..plan.segments.len())
            .map(|i| 500.0 + 137.0 * (i as f64))
            .collect();
        let hetero_base = mass_properties(&plan, &hetero, p.root_height_m);

        let mut permuted = plan.clone();
        permuted.segments.reverse();
        let n = plan.segments.len();
        let perm_uniform: Vec<f64> = (0..n).rev().map(|i| d[i]).collect();
        let perm_hetero: Vec<f64> = (0..n).rev().map(|i| hetero[i]).collect();

        for (label, a, b) in [
            (
                "uniform",
                &base,
                mass_properties(&permuted, &perm_uniform, p.root_height_m),
            ),
            (
                "heterogeneous",
                &hetero_base,
                mass_properties(&permuted, &perm_hetero, p.root_height_m),
            ),
        ] {
            assert!(
                rel(a.mass_kg, b.mass_kg) <= 1e-12,
                "plan `{}` ({label}): mass depends on segment order",
                plan.name
            );
            assert!(
                rel(a.volume_m3, b.volume_m3) <= 1e-12,
                "plan `{}` ({label}): volume depends on segment order",
                plan.name
            );
            for (axis, (x, y)) in a.com_m.iter().zip(b.com_m).enumerate() {
                assert!(
                    moved(*x, y, p.root_height_m) <= 1e-12,
                    "plan `{}` ({label}) axis {axis}: CoM depends on segment order \
                     ({x:.17} vs {y:.17})",
                    plan.name
                );
            }
        }
    }
}

/// A density mismatch is a loud panic, never a silent fill. An index-aligned
/// slice that is the wrong length is a programming error, and defaulting the
/// missing entries to 1.0 would be a wrong answer wearing an identity
/// default's clothes (A-3: a green test measuring nothing).
#[test]
#[should_panic(expected = "index-aligned with segments")]
fn a_density_slice_of_the_wrong_length_is_refused() {
    let plan = biped_plan();
    let _ = mass_properties(&plan, &[1.0, 1.0], 0.88);
}

// ------------------------------------------------------------- report ------

/// The design pass § 10's predictions, measured (P1, P3, P4, P5, P6, P8).
/// Printed under `--nocapture`; **a divergence is a finding about that
/// document**, recorded by whoever measures.
///
/// Asserts only what is ours to assert — the two arithmetic identities (the
/// share table sums to 1, and heterogeneity moves the leg's ω in the direction
/// a heavier distal segment demands). The published comparisons print and
/// assert nothing, because a published band is not ours to assert.
#[test]
fn report_the_mass_integral() {
    let plan = biped_plan();
    let p = baked(&plan);
    let mp = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);

    println!("\nB6-a MASS INTEGRAL — measured against design pass § 10 (g = {G_EARTH})");
    println!(
        "  P1 whole-body volume  {:.9} m³ (predicted 0.168388000)",
        mp.volume_m3
    );
    println!("     share table:");
    let mut share_sum = 0.0;
    for (s, m) in plan.segments.iter().zip(&mp.segment_mass_kg) {
        let share = m / mp.volume_m3;
        share_sum += share;
        println!("       {:<12} {:.5} m³  {:6.2} %", s.name, m, 100.0 * share);
    }
    assert!(
        (share_sum - 1.0).abs() < 1e-12,
        "the share table must sum to 1, got {share_sum}"
    );

    println!(
        "  P3 mass at a uniform 1010 kg/m³  {:.3} kg (predicted 170.072) — NOT a check; \
         the biped is 2.43× a human by volume",
        mass_properties(&plan, &vec![1010.0; plan.segments.len()], p.root_height_m).mass_kg
    );

    // P4: the leg about the hip, uniform density.
    let (mut m_sum, mut md, mut i_hip) = (0.0_f64, 0.0_f64, 0.0_f64);
    for name in ["leg_l_upper", "leg_l_lower"] {
        let s = by(&plan, name);
        let m = mp.segment_mass_kg[index_of(&plan, name)];
        let d = if name == "leg_l_upper" {
            -s.offset_m[1]
        } else {
            -(s.pivot_m[1] + s.offset_m[1])
        };
        m_sum += m;
        md += m * d;
        i_hip += m * (s.size_m[1] * s.size_m[1] + s.size_m[2] * s.size_m[2]) / 12.0 + m * d * d;
    }
    let w_uniform = leg_omega(&plan, &mp, G_EARTH);
    println!(
        "  P4 leg/hip, uniform ρ: M/ρ {m_sum:.6} (0.028584) · d {:.6} (0.415630) · \
         I/ρ {i_hip:.9} (0.006848268) · ω {w_uniform:.6} rad/s (4.125332) · \
         T {:.6} s (1.523074)",
        md / m_sum,
        std::f64::consts::TAU / w_uniform
    );

    // P5: the same leg, heterogeneous.
    for (label, thigh, shank, predicted) in [
        ("Dempster-like 1050/1090", 1050.0, 1090.0, 4.114713),
        ("armoured shank 1060/1900", 1060.0, 1900.0, 3.985832),
    ] {
        let mut d = vec![1.0_f64; plan.segments.len()];
        for (name, rho) in [("leg_l_upper", thigh), ("leg_l_lower", shank)] {
            d[index_of(&plan, name)] = rho;
        }
        let w = leg_omega(&plan, &mass_properties(&plan, &d, p.root_height_m), G_EARTH);
        println!(
            "  P5 {label}: ω {w:.6} rad/s (predicted {predicted:.6}), \
             {:+.3} % vs uniform",
            100.0 * (w - w_uniform) / w_uniform
        );
        assert!(
            w < w_uniform,
            "a distally heavier leg must swing SLOWER, got {w} against {w_uniform}"
        );
    }

    // P6: whole-body CoM, uniform and with lungs in the trunk.
    println!(
        "  P6 CoM y, ground frame {:.6} m (predicted 0.968187) — root-relative {:+.6} m \
         (predicted +0.088187)",
        mp.com_m[1],
        mp.com_m[1] - p.root_height_m
    );
    let mut lungs = vec![1100.0_f64; plan.segments.len()];
    lungs[index_of(&plan, "trunk")] = 800.0;
    let lungs_mp = mass_properties(&plan, &lungs, p.root_height_m);
    println!(
        "     lungs-in-trunk 800 / rest 1100: {:.3} kg (predicted 164.227), root-relative \
         CoM {:+.6} m (predicted +0.067496), Δ {:+.1} mm",
        lungs_mp.mass_kg,
        lungs_mp.com_m[1] - p.root_height_m,
        1000.0 * (lungs_mp.com_m[1] - mp.com_m[1])
    );

    // P8: the bound on what a radial order could ever be worth to the swing.
    let z_term: f64 = ["leg_l_upper", "leg_l_lower"]
        .iter()
        .map(|name| {
            let s = by(&plan, name);
            mp.segment_mass_kg[index_of(&plan, name)] * s.size_m[2] * s.size_m[2] / 12.0
        })
        .sum();
    println!(
        "  P8 radially-movable z-term {z_term:.9} (predicted 0.000087437) = {:.3} % of I \
         (1.277) ⇒ I ∈ [{:+.3}, {:+.3}] %, ω ∈ [{:+.3}, {:+.3}] %",
        100.0 * z_term / i_hip,
        -100.0 * z_term / i_hip,
        200.0 * z_term / i_hip,
        100.0 * ((1.0 + 2.0 * z_term / i_hip).powf(-0.5) - 1.0),
        100.0 * ((1.0 - z_term / i_hip).powf(-0.5) - 1.0),
    );

    println!("\n  the same three plans, whole-body:");
    for plan in plans() {
        let p = baked(&plan);
        let mp = mass_properties(&plan, &segment_densities(&plan), p.root_height_m);
        println!(
            "    {:<16} V {:.6} m³ · CoM ({:.4}, {:.6}, {:.4}) · at 1010 kg/m³ {:.2} kg",
            plan.name,
            mp.volume_m3,
            mp.com_m[0],
            mp.com_m[1],
            mp.com_m[2],
            mp.volume_m3 * 1010.0
        );
    }
}
