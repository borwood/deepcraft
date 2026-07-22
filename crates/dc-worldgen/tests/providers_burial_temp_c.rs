//! **Slot `burial_temp_c`** — falsifiers for the coalification seam
//! (journal/0067): the provider reaches the production run in **both**
//! directions, and the one rule that is structural rather than thresholded
//! survives an infinitely hot crust.

mod providers_common;
use providers_common::{SEED, production_pregen};

use dc_worldgen::deeptime::{self, Biofacies, BuriedUnit, DeepField, Providers};
use dc_worldgen::pregen::Pregen;

/// A geotherm so cold nothing ever reaches onset.
fn frozen(_: BuriedUnit) -> f64 {
    f64::NEG_INFINITY
}

/// A geotherm so hot every buried unit is past onset the moment it is buried.
fn molten(_: BuriedUnit) -> f64 {
    f64::INFINITY
}

fn field_with(pregen: &Pregen, provider: Option<fn(BuriedUnit) -> f64>) -> DeepField {
    let mut cfg = deeptime::production_config(&pregen.grid, SEED);
    cfg.providers = Providers {
        burial_temp_c: provider,
        ..Providers::default()
    };
    deeptime::build_field_cfg(&pregen.grid, &cfg)
}

fn count(f: &DeepField, biota: Biofacies) -> usize {
    f.strata
        .iter()
        .flat_map(|s| s.units.iter())
        .filter(|u| u.tag.biota == biota)
        .count()
}

/// **The seam is a seam.** The identity world has coal in it; a world whose
/// geotherm never reaches onset has none at all, and the peat it did not promote
/// is still there — promotion is a retagging, so the peat + coal total is
/// conserved across the swap.
///
/// This is the falsifier the golden cannot be: `providers_golden.rs` proves the
/// *absent* provider changes nothing, which is equally consistent with a slot
/// that is never consulted. This one proves it is consulted.
#[test]
fn a_frozen_geotherm_leaves_the_record_with_no_coal() {
    let pregen = production_pregen();
    let identity = field_with(&pregen, None);
    let cold = field_with(&pregen, Some(frozen));

    let (coal_id, peat_id) = (
        count(&identity, Biofacies::Coal),
        count(&identity, Biofacies::Peat),
    );
    let (coal_cold, peat_cold) = (count(&cold, Biofacies::Coal), count(&cold, Biofacies::Peat));

    assert!(
        coal_id > 0,
        "the identity world has no coal to begin with — this falsifier is blind"
    );
    assert_eq!(
        coal_cold, 0,
        "a geotherm that never reaches onset still produced {coal_cold} coal units"
    );
    assert_eq!(
        peat_cold,
        peat_id + coal_id,
        "promotion must be a retagging: every unit the identity called coal is \
         peat here, and no unit appeared or vanished"
    );
}

/// The other direction, and the rule that is **not** a threshold. Under an
/// infinitely hot crust every buried peat is coal — but the topmost unit of each
/// column is the living surface and is never promoted, at any temperature. That
/// guard is written out in `promote_coal` rather than implied by a positive
/// threshold, and this is the only configuration in which the difference shows.
#[test]
fn a_molten_geotherm_promotes_everything_except_the_living_surface() {
    let pregen = production_pregen();
    let identity = field_with(&pregen, None);
    let hot = field_with(&pregen, Some(molten));

    assert!(
        count(&hot, Biofacies::Coal) > count(&identity, Biofacies::Coal),
        "an infinite geotherm did not promote more peat than the 8 m identity"
    );

    let surviving_peat: Vec<usize> = hot
        .strata
        .iter()
        .enumerate()
        .filter(|(_, s)| s.units.iter().any(|u| u.tag.biota == Biofacies::Peat))
        .map(|(i, _)| i)
        .collect();
    for i in surviving_peat {
        let units = &hot.strata[i].units;
        let last = units.len() - 1;
        for (k, u) in units.iter().enumerate() {
            assert!(
                u.tag.biota != Biofacies::Peat || k == last,
                "cell {i} unit {k} of {} stayed peat under an infinite geotherm \
                 without being the living surface",
                units.len()
            );
        }
    }
}
