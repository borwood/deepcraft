//! **The share-weighted susceptibility blend** (journal/0072, audit site A1) —
//! falsifiers for the S-4 cure that replaced argmax-then-lookup in erosion's four
//! consumption sites.
//!
//! The verdict rule (`exposed_litho` / the `outcrop_at` seam) is unchanged and its
//! own falsifiers live in `providers_outcrop_at.rs`. These assert the *quantity*
//! the verdict is now the argmax of behaves: that the blend weights the table by
//! window share, that a uniform window reproduces the old argmax lookup **bit for
//! bit** (argmax is the degenerate case), and that the rate is continuous where the
//! argmax used to step — the whole point of the change.

use dc_worldgen::deeptime::lithology::{
    Agent, Litho, blend_susceptibility, dominant_litho, exposed_litho, exposed_shares,
    litho_of_tag, susceptibility_table,
};
use dc_worldgen::deeptime::{Aridity, Biofacies, DepEnv, DepTag, DepUnit, EnergyBand, Eolian};

const CONTRAST: f64 = 2.5;
const CAP: f64 = 8.0;

/// Recorder order: `units[0]` is deepest, the last is the surface.
fn unit(tag: DepTag, thickness_m: f64) -> DepUnit {
    // The tag-derived species — what a unit carries whenever nothing
    // transported it (material-aware transport off, and every non-fluvial
    // depositor). These suites are about the *window walk*, not the load.
    DepUnit::new(
        tag,
        thickness_m,
        false,
        0,
        0,
        litho_of_tag(tag).reference_material(),
    )
}

fn mud(thickness_m: f64) -> DepUnit {
    unit(
        DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low),
        thickness_m,
    )
}

fn coarse(thickness_m: f64) -> DepUnit {
    unit(
        DepTag::mineral(DepEnv::Subaerial, Aridity::Arid, EnergyBand::High),
        thickness_m,
    )
}

fn peat(thickness_m: f64) -> DepUnit {
    unit(
        DepTag {
            env: DepEnv::Subaerial,
            aridity: Aridity::Humid,
            energy: EnergyBand::Low,
            biota: Biofacies::Peat,
            eolian: Eolian::None,
        },
        thickness_m,
    )
}

/// **(a) A 55/45 window blends the table 55/45, for every agent.** The whole
/// promise: a window that is 45 % fine over 55 % coarse yields a rate that is the
/// share-weighted mix of the two rocks' rates — not the winner's rate. 0.495 m of
/// fine over 0.405 m of coarse fills the 0.9 m window exactly, so the shares are
/// 0.55 / 0.45.
#[test]
fn a_mixed_window_blends_the_table_by_share() {
    // fine on top (0.495), coarse beneath (0.405) → shares fine 0.55, coarse 0.45.
    let record = [coarse(0.405), mud(0.495)];
    let shares = exposed_shares(&record);
    let sh = shares.shares(); // producer-side value inspection (no CoarseField raw read)
    let fi = Litho::ClasticFine.index();
    let ci = Litho::ClasticCoarse.index();
    // Tolerance derived from the packed record's thickness quantum (P11
    // slice 3): a stated thickness lands within q/2 = 2⁻¹¹ m of itself, so a
    // share over the 0.9 m window can sit up to ~q/0.9 ≈ 1.1e-3 from the
    // nominal 0.55/0.45. Derived, not fitted.
    assert!(
        (sh[fi] - 0.55).abs() < 2e-3 && (sh[ci] - 0.45).abs() < 2e-3,
        "shares fine {} coarse {} — expected 0.55 / 0.45 (± the thickness quantum)",
        sh[fi],
        sh[ci]
    );
    // Everything else is empty (no deficit — the window is full).
    let others: f64 = (0..Litho::COUNT)
        .filter(|&k| k != fi && k != ci)
        .map(|k| sh[k])
        .sum();
    assert!(
        others.abs() < 1e-12,
        "unexpected share mass elsewhere: {others}"
    );

    for agent in Agent::ALL {
        let tab = susceptibility_table(agent, CONTRAST, CAP);
        let blended = blend_susceptibility(&shares, &tab);
        let expected = sh[fi] * tab[fi] + sh[ci] * tab[ci];
        assert!(
            (blended - expected).abs() < 1e-12,
            "{}: blend {blended} != 0.55·fine + 0.45·coarse {expected}",
            agent.name()
        );
        // And it genuinely lies between the two endpoints (it is a mix, not one of
        // them) whenever the two rocks differ on this axis.
        let (lo, hi) = (tab[fi].min(tab[ci]), tab[fi].max(tab[ci]));
        if (hi - lo) > 1e-9 {
            assert!(
                blended > lo && blended < hi,
                "{}: blend {blended} not strictly between {lo} and {hi}",
                agent.name()
            );
        }
    }
}

/// **(b) A uniform window equals the argmax lookup, bit for bit.** A window that is
/// 100 % one lithology must blend to exactly that lithology's table entry — so the
/// blend is a strict generalisation of the old `sus_tab[exposed_litho.index()]`,
/// and turning it on cannot perturb a single-rock cell by even one ULP. Basement
/// (the empty-record deficit) is included: it is the uniform window a stripped
/// column presents.
#[test]
fn a_uniform_window_is_the_argmax_lookup_bit_for_bit() {
    let cases: [(Vec<DepUnit>, Litho); 4] = [
        (vec![mud(1.0)], Litho::ClasticFine),
        (vec![coarse(1.0)], Litho::ClasticCoarse),
        (vec![peat(1.0)], Litho::OrganicPeat),
        (vec![], Litho::Basement), // stripped column → 100 % basement deficit
    ];
    for (record, want) in cases {
        let shares = exposed_shares(&record);
        let k = want.index();
        assert_eq!(
            shares.shares()[k].to_bits(),
            1.0f64.to_bits(),
            "{} share != 1.0",
            want.code()
        );
        assert_eq!(
            exposed_litho(&record),
            want,
            "verdict disagrees for {}",
            want.code()
        );
        for agent in Agent::ALL {
            let tab = susceptibility_table(agent, CONTRAST, CAP);
            assert_eq!(
                blend_susceptibility(&shares, &tab).to_bits(),
                tab[k].to_bits(),
                "{} {}: blend of a uniform window is not the argmax entry, bit for bit",
                agent.name(),
                want.code()
            );
        }
    }
}

/// **(c) The rate is continuous where the argmax used to step.** A fine caprock
/// thinning over coarse crosses the plurality threshold at 0.45 m: below it the
/// window argmaxes to coarse, above it to fine, so the *old* rule stepped the rate
/// by the full `|fine − coarse|` gap across one contour — the coherent S-4 line
/// walk-0071 flagged. The blend instead moves the rate **linearly** with the
/// caprock thickness, so adjacent samples differ only in proportion to the
/// thickness change. This is the cure, asserted directly.
#[test]
fn the_rate_is_continuous_across_the_old_flip() {
    let tab = susceptibility_table(Agent::Abrasion, CONTRAST, CAP);
    let fi = Litho::ClasticFine.index();
    let ci = Litho::ClasticCoarse.index();
    let gap = (tab[fi] - tab[ci]).abs();
    assert!(
        gap > 0.1,
        "fine and coarse must differ for this test to mean anything"
    );

    let rate_at = |f: f64| {
        // fine caprock of thickness f over thick coarse.
        let record = [coarse(3.0), mud(f)];
        blend_susceptibility(&exposed_shares(&record), &tab)
    };

    // The flip the old rule had, still present in the verdict:
    assert_eq!(
        exposed_litho(&[coarse(3.0), mud(0.44)]),
        Litho::ClasticCoarse
    );
    assert_eq!(exposed_litho(&[coarse(3.0), mud(0.46)]), Litho::ClasticFine);

    // Straddling that flip, the blended rate barely moves — it does NOT jump the
    // gap. The step in shares over 0.44→0.46 is 0.02/0.9, so the rate step is at
    // most that fraction of the gap.
    let jump = (rate_at(0.46) - rate_at(0.44)).abs();
    assert!(
        jump < 0.05 * gap,
        "the blended rate stepped {jump} across the old flip (gap {gap}) — not continuous"
    );

    // Over a fine sweep the maximum adjacent step is bounded by the per-step share
    // change × the gap, i.e. the field is Lipschitz in caprock thickness with no
    // discontinuity anywhere.
    let step = 0.01;
    let mut worst = 0.0f64;
    let mut f = 0.0;
    let mut prev = rate_at(f);
    while f < 0.95 {
        f += step;
        let r = rate_at(f);
        worst = worst.max((r - prev).abs());
        prev = r;
    }
    // The derived Lipschitz bound gained a term with P11 slice 3's packed
    // record: a stated caprock thickness stores as the nearest 2^-10 m quantum,
    // so two adjacent sweep samples can differ in STORED thickness by up to
    // `step + quantum` (each endpoint rounds up to half a quantum the other
    // way). Derived, not fitted — the pre-pack bound is the q → 0 limit.
    let q = dc_worldgen::deeptime::DepUnit::THICKNESS_QUANTUM_M;
    let bound = ((step + q) / 0.9) * gap * 1.001;
    assert!(
        worst <= bound,
        "max adjacent rate step {worst} exceeds the continuity bound {bound}"
    );
}

/// The verdict is the argmax of the shares — the binding that keeps `outcrop_at`
/// and `outcrop_shares` two faces of one walk (S-3: the summary derived from the
/// authority). Checked over a spread of synthetic records with no exact-thickness
/// ties, where `dominant_litho`'s index tie-break and `exposed_litho`'s
/// nearest-surface tie-break cannot differ.
#[test]
fn the_verdict_is_the_argmax_of_the_shares() {
    let records: [Vec<DepUnit>; 6] = [
        vec![],
        vec![mud(1.0)],
        vec![coarse(3.0), mud(0.46)],
        vec![coarse(3.0), mud(0.44)],
        vec![mud(1.0), coarse(0.03)],
        vec![peat(0.7), mud(0.15)],
    ];
    for record in records {
        assert_eq!(
            exposed_litho(&record),
            dominant_litho(&exposed_shares(&record)),
            "verdict and argmax-of-shares disagree on {:?}",
            record.iter().map(|u| u.thickness_m()).collect::<Vec<_>>()
        );
    }
}
