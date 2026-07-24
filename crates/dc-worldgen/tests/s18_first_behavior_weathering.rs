//! **S18 → Movement 3 — subaerial weathering as an in-loop, per-epoch,
//! ACCUMULATING process on the working inventory (journal/0094), consuming the S17
//! keystone end-to-end.**
//!
//! The load-bearing proofs, over a *real* production `DeepField`:
//!
//! 1. **Identity floor.** With `weather_inventory` off (the default) the field
//!    carries **no ledgers** and the record/surface are untouched — byte-identical.
//! 2. **Purely additive (the two-authorities split).** Turning the flag on does
//!    **not** perturb the erosion sim: the `strata` record and `surf` are identical;
//!    only the `ledgers` sidecar appears. The `dc:deep/weather_inventory` pass reads
//!    the terrain but writes only the ledger — it never touches `R`/`H`.
//! 3. **One fact per agent per chapter, cause-carrying, ACCUMULATED.** The in-loop
//!    pass fires every epoch; same-chapter firings coalesce to one fact per summed
//!    agent (chemical/biotic/frost) while the band keeps growing, so a cell weathered
//!    across several tectonic chapters carries several agent-facts.
//! 4. **The fold has a REAL product.** `base + facts` composes to a loose weathering
//!    product that — accumulated over the run — is a multi-decimetre-to-metre band,
//!    not S18's sub-voxel one-shot. The ≥1-voxel proof is at production scale below.

use dc_worldgen::deeptime::{
    BEDROCK_SEAM_MATERIAL, Cause, DeepConfig, InvForm, WEATHERING_AGENTS, build_field,
    build_field_cfg, production_config,
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
fn a_weathered_cell_carries_one_fact_per_agent_per_chapter_and_accumulates() {
    let pregen = small_pregen();
    let cfg_on = DeepConfig {
        weather_inventory: true,
        ..production_config(&pregen.grid, SEED)
    };
    let field = build_field_cfg(&pregen.grid, &cfg_on);

    // Find subaerial cells whose bedrock seam weathered (positive product). Its
    // bedrock ledger slot is the LAST slot (index == units.len()).
    let mut weathered = 0usize;
    let mut max_band = 0.0f64;
    let mut checked = false;
    for (i, ledger) in field.ledgers.iter().enumerate() {
        let unit_count = field.strata[i].units.len();
        let product = ledger.weathering_product_m(unit_count);
        if product <= 0.0 {
            continue;
        }
        weathered += 1;
        max_band = max_band.max(product);
        let bedrock = ledger.facts_for(unit_count);
        // Every fact is the same Structure→Loose edge on the basement seam.
        for f in bedrock {
            assert_eq!(f.from(), (BEDROCK_SEAM_MATERIAL, InvForm::Structure));
            assert_eq!(f.to(), (BEDROCK_SEAM_MATERIAL, InvForm::Loose));
            assert!(WEATHERING_AGENTS.contains(&f.cause()));
        }
        // **One fact per (chapter, cause)** — the in-loop commit coalesces
        // same-chapter firings, so "one fact per agent per firing" becomes one fact
        // per agent per chapter after coalescing. No duplicate (chapter, cause).
        let mut keys: Vec<(u8, Cause)> = bedrock.iter().map(|f| (f.chapter(), f.cause())).collect();
        let n = keys.len();
        keys.sort_by(|a, b| (a.0, a.1.name()).cmp(&(b.0, b.1.name())));
        keys.dedup();
        assert_eq!(
            keys.len(),
            n,
            "cell {i}: no duplicate (chapter, cause) fact"
        );
        // Σ agent shares == the composed loose product (the fold read-back).
        let sum_shares: f64 = bedrock.iter().map(|f| f.fraction_m()).sum();
        assert!((sum_shares - product).abs() < 1e-9);
        checked = true;
    }
    assert!(weathered > 0, "some subaerial cell must weather (got 0)");
    assert!(checked);
    // The ACCUMULATED process makes a real band — orders above S18's sub-voxel 0.04 m
    // one-shot. (The ≥1-voxel proof is the production-scale test below.)
    assert!(
        max_band > 0.1,
        "accumulation over the loop yields a real (multi-decimetre) band: {max_band:.3} m"
    );
}

#[test]
fn production_scale_saprolite_band_reaches_at_least_one_voxel() {
    // **A-3 GUARD (the S18 lesson made structural).** Prove the ≥1-voxel band on a
    // PRODUCTION-SCALE world — the tour's seed/extent — not a hand-fed magnitude.
    // This is the load-bearing acceptance of Movement 3: the in-loop accumulated
    // process, at production scale, probed in-slice.
    const TOUR_SEED: u64 = 1337;
    const VOXEL_M: f64 = 0.9;
    let pregen = Pregen::run(WorldParams {
        seed: TOUR_SEED,
        extent: Extent::Medium,
    });
    let cfg = DeepConfig {
        weather_inventory: true,
        ..production_config(&pregen.grid, TOUR_SEED)
    };
    let field = build_field_cfg(&pregen.grid, &cfg);

    let (mut max_band, mut argmax) = (0.0f64, 0usize);
    for (i, ledger) in field.ledgers.iter().enumerate() {
        let b = ledger.weathering_product_m(field.strata[i].units.len());
        if b > max_band {
            max_band = b;
            argmax = i;
        }
    }
    let voxels = max_band / VOXEL_M;
    let eighths = (voxels * 8.0).round() as i64;
    println!(
        "production-scale saprolite band: max {max_band:.3} m = {voxels:.2} voxels \
         ({eighths} eighths) @ {VOXEL_M} m/voxel, deep cell {argmax}"
    );
    assert!(
        max_band >= VOXEL_M,
        "band must be >= 1 voxel ({VOXEL_M} m); got {max_band:.3} m ({voxels:.2} vox)"
    );
}
