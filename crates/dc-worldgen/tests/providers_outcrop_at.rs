//! **Slot `outcrop_at`** — falsifiers for the outcrop seam (journal/0060) and
//! its thickness-dominance identity (journal/0068).

use dc_worldgen::deeptime::providers::identity_outcrop_at;
use dc_worldgen::deeptime::{
    Aridity, Biofacies, DepEnv, DepTag, DepUnit, EnergyBand, Eolian, Litho, litho_of_tag,
};

/// The recorder logs units bottom-up: `units[0]` is deepest, the last is the
/// surface. So a "lamina atop mud" record is `[mud, lamina]`.
fn unit(tag: DepTag, thickness_m: f64) -> DepUnit {
    DepUnit {
        tag,
        thickness_m,
        unconformity: false,
        chapter: 0,
    }
}

/// A marine mud unit → `Litho::ClasticFine`.
fn mud(thickness_m: f64) -> DepUnit {
    unit(
        DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low),
        thickness_m,
    )
}

/// A subaerial high-energy sand unit → `Litho::ClasticCoarse`.
fn coarse(thickness_m: f64) -> DepUnit {
    unit(
        DepTag::mineral(DepEnv::Subaerial, Aridity::Arid, EnergyBand::High),
        thickness_m,
    )
}

/// A charcoal fire bed → `Litho::OrganicCharcoal`.
fn charcoal(thickness_m: f64) -> DepUnit {
    unit(
        DepTag {
            env: DepEnv::Subaerial,
            aridity: Aridity::Humid,
            energy: EnergyBand::Low,
            biota: Biofacies::Charcoal,
            eolian: Eolian::None,
        },
        thickness_m,
    )
}

/// The identity `outcrop_at`: an empty record is `Basement`, and a unit that on
/// its own fills the dominance window carries the outcrop (a flat, single-litho
/// record still reads as its top).
#[test]
fn the_identity_outcrop_is_the_dominant_lithology() {
    assert_eq!(identity_outcrop_at(&[]), Litho::Basement);
    let tag = DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low);
    // A 1.0 m unit more than fills the 0.9 m window, so it is unambiguously the
    // outcrop — the pre-journal/0068 "top of the record" answer, recovered as the
    // degenerate case of the dominance rule.
    assert_eq!(identity_outcrop_at(&[unit(tag, 1.0)]), litho_of_tag(tag));
}

/// **(a) A thin lamina cannot define the cell.** A 3 cm bed of any facies sitting
/// on thick mud loses the window to the mud — this is the whole point of the rule
/// that replaced the name-keyed charcoal carve-out. Using a charcoal lamina makes
/// the doubled claim: charcoal is in the record honestly, and it still does not
/// outcrop.
#[test]
fn a_thin_lamina_does_not_define_the_outcrop() {
    let record = [mud(1.0), charcoal(0.03)];
    let out = identity_outcrop_at(&record);
    assert_eq!(
        out,
        Litho::ClasticFine,
        "the thick mud outcrops, not the lamina"
    );
    assert_ne!(
        out,
        Litho::OrganicCharcoal,
        "a 3 cm fire bed never outcrops a cell"
    );
}

/// **(b) Dominance is integrated, not a per-unit floor.** Forty 2 cm beds of
/// coarse — each far below any thin-bed threshold — stack to 0.8 m and *do*
/// dominate a 0.9 m window over the mud beneath. A naive "skip units under X"
/// rule would drop every one of them and read mud; the dominance rule reads
/// coarse. This is the test that distinguishes the two.
#[test]
fn many_thin_units_of_one_litho_do_dominate() {
    let mut record = vec![mud(1.0)];
    for _ in 0..40 {
        record.push(coarse(0.02));
    }
    assert_eq!(
        identity_outcrop_at(&record),
        Litho::ClasticCoarse,
        "0.8 m of stacked thin coarse beds must out-mass the 0.1 m of mud in the window"
    );
}

/// **(c) A short record's deficit is basement.** A single thin unit over nothing
/// leaves most of the window empty, and that emptiness is the rock below the pile
/// — so basement wins even though a real unit is present.
#[test]
fn a_lone_thin_unit_yields_to_the_basement_deficit() {
    assert_eq!(identity_outcrop_at(&[coarse(0.03)]), Litho::Basement);
}

/// **(d) An empty record is basement.** The column has been stripped past its
/// whole sedimentary history (or was never recorded).
#[test]
fn an_empty_record_is_basement() {
    assert_eq!(identity_outcrop_at(&[]), Litho::Basement);
}
