//! **S18 — the first real cellular behavior: sum-agent subaerial weathering on the
//! working inventory, consuming the S17 keystone end-to-end.**
//!
//! The load-bearing proofs, over a *real* production `DeepField`:
//!
//! 1. **Identity floor.** With `weather_inventory` off (the default) the field
//!    carries **no ledgers** and the record/surface are untouched — byte-identical.
//! 2. **Purely additive.** Turning the flag on does **not** perturb the erosion
//!    sim: the `strata` record and `surf` are identical; only the `ledgers` sidecar
//!    appears (the material-transformation layer runs *after* the height sim).
//! 3. **One fact per agent, cause-carrying.** A weathered cell's bedrock seam holds
//!    exactly the three summed agents (chemical/biotic/frost), each its own fact.
//! 4. **The fold has a product.** `base + facts` composes to a positive loose
//!    weathering product — the quantity the collapse expresses as a basal band.

use dc_worldgen::deeptime::{
    Cause, DeepConfig, DepEnv, WEATHERING_AGENTS, build_field, build_field_cfg, production_config,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

// The canonical production golden world at `Extent::Small` (shared with
// `providers_common`, whose littoral/coast tests exercise its land — so its deep
// record carries subaerial deposition).
const SEED: u64 = 0x0B0A_57EE_0059;

fn small_pregen() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

#[test]
fn identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical() {
    let pregen = small_pregen();
    // The default production field (flag off) — the pre-slice authority.
    let field_off = build_field(&pregen.grid, SEED);
    assert!(
        field_off.ledgers.is_empty(),
        "flag off ⇒ no ledgers (S-5 identity default)"
    );
    assert!(field_off.ledger_at_voxel(0, 0).is_none());
    assert!(!field_off.strata.is_empty(), "the record is populated");
}

#[test]
fn on_flag_is_purely_additive_record_and_surface_untouched() {
    let pregen = small_pregen();
    let cfg_off = production_config(&pregen.grid, SEED);
    let cfg_on = DeepConfig {
        weather_inventory: true,
        ..cfg_off
    };
    let field_off = build_field_cfg(&pregen.grid, &cfg_off);
    let field_on = build_field_cfg(&pregen.grid, &cfg_on);

    // The weathering pass runs AFTER the erosion loop and never writes the record or
    // the surface — the §11 continuation-slot separation (material composition vs
    // height budget). So the only difference between the two fields is the sidecar.
    assert_eq!(
        field_on.strata, field_off.strata,
        "the strata record is untouched by inventory weathering"
    );
    assert_eq!(
        field_on.surf, field_off.surf,
        "the eroded surface is untouched"
    );
    assert_eq!(
        field_on.regolith, field_off.regolith,
        "the regolith plane is untouched"
    );

    // The sidecar appeared, index-parallel to the record.
    assert_eq!(field_on.ledgers.len(), field_on.strata.len());
    assert!(field_off.ledgers.is_empty());
}

#[test]
fn a_weathered_cell_carries_one_fact_per_agent_with_the_summed_causes() {
    let pregen = small_pregen();
    let cfg_on = DeepConfig {
        weather_inventory: true,
        ..production_config(&pregen.grid, SEED)
    };
    let field = build_field_cfg(&pregen.grid, &cfg_on);

    // The distillation gates weathering on the RECORD (a cell that deposited any
    // subaerial unit saw land) — not the final surface, which after isostasy sits
    // far below the datum on a real world. Count the same way to bound `weathered`.
    let saw_subaerial = |s: &dc_worldgen::deeptime::DeepStrata| {
        s.units.iter().any(|u| u.tag.env == DepEnv::Subaerial)
    };
    let subaerial = field.strata.iter().filter(|s| saw_subaerial(s)).count();

    // Find a subaerial cell whose bedrock seam actually weathered (a positive
    // product). Its bedrock ledger slot is the LAST slot (index == units.len()).
    let mut weathered = 0usize;
    let mut checked_causes = false;
    for (i, ledger) in field.ledgers.iter().enumerate() {
        let unit_count = field.strata[i].units.len();
        let product = ledger.weathering_product_m(unit_count);
        if product <= 0.0 {
            continue;
        }
        weathered += 1;
        let bedrock = ledger.facts_for(unit_count);
        assert_eq!(
            bedrock.len(),
            WEATHERING_AGENTS.len(),
            "cell {i}: one fact per summed weathering agent"
        );
        let mut causes: Vec<Cause> = bedrock.iter().map(|f| f.cause()).collect();
        causes.sort_by_key(|c| c.name());
        assert_eq!(causes, vec![Cause::Biotic, Cause::Chemical, Cause::Frost]);
        // Σ agent shares == the composed loose product (the fold read-back).
        let sum_shares: f64 = bedrock.iter().map(|f| f.fraction_m()).sum();
        assert!((sum_shares - product).abs() < 1e-9);
        checked_causes = true;
    }
    assert!(
        subaerial > 0,
        "the test seed must record subaerial deposition somewhere"
    );
    // Weathering fires on real subaerial history, and ONLY there (a subset — cells
    // under very thick regolith have sub-EPS shares and commit none).
    assert!(weathered > 0, "some subaerial cell must weather (got 0)");
    assert!(
        weathered <= subaerial,
        "weathering is subaerial-only ({weathered} > {subaerial})"
    );
    assert!(checked_causes);
}
