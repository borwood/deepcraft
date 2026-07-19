//! S9 deep-time spike invariants: determinism (byte-identical planes AND
//! records under a repeated seed), explicit mass conservation down the receiver
//! chain, and the recorder's `sum(units) == H` finalize invariant. The
//! measured numbers live in docs/spikes/S9-results.md; these are the falsifiers.

use dc_worldgen::deeptime::{self, DeepConfig, Erosion};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn test_cfg(seed: u64) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m: 1000.0,
        iterations: 40,
        remarch_interval: 20,
        record: true,
        ..DeepConfig::default()
    }
}

fn small_world(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

#[test]
fn same_seed_is_byte_identical_planes_and_records() {
    let pregen = small_world(SEED);
    let cfg = test_cfg(SEED);
    let a = deeptime::run(&pregen, &cfg);
    let b = deeptime::run(&pregen, &cfg);
    assert_eq!(a.grid.r, b.grid.r, "bedrock plane must be byte-identical");
    assert_eq!(a.grid.h, b.grid.h, "alluvium plane must be byte-identical");
    assert_eq!(
        a.grid.strata, b.grid.strata,
        "strata records must be byte-identical"
    );
}

#[test]
fn a_different_seed_diverges() {
    let pregen_a = small_world(SEED);
    let pregen_b = small_world(SEED ^ 0xABCD);
    let a = deeptime::run(&pregen_a, &test_cfg(SEED));
    let b = deeptime::run(&pregen_b, &test_cfg(SEED ^ 0xABCD));
    assert_ne!(
        a.grid.r, b.grid.r,
        "a different world must erode differently"
    );
}

#[test]
fn mass_is_conserved_up_to_uplift() {
    let pregen = small_world(SEED);
    let cfg = test_cfg(SEED);
    let mut grid = deeptime::build(&pregen, &cfg);
    let mut ero = Erosion::new(&grid);
    let before = deeptime::total_mass(&grid);
    deeptime::climate::march(&mut grid, deeptime::sea_level_at(&cfg, 0));
    let mut uplift_total = 0.0;
    for it in 0..cfg.iterations {
        let sl = deeptime::sea_level_at(&cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            deeptime::climate::march(&mut grid, sl);
        }
        uplift_total += ero.step(&mut grid, &cfg, sl);
    }
    let after = deeptime::total_mass(&grid);
    let residual = after - before - uplift_total;
    // The only external input is uplift; erosion, deposition, weathering, and
    // diffusion all conserve. Residual is pure floating-point slack over the
    // ~5.5 k-cell × 40-iter ledger (mass magnitude ~1e6 metre-cells).
    assert!(
        residual.abs() < 1.0,
        "mass leaked: residual {residual} (before {before}, after {after}, uplift {uplift_total})"
    );
}

#[test]
fn recorder_total_equals_alluvium_everywhere() {
    let pregen = small_world(SEED);
    let cfg = test_cfg(SEED);
    let run = deeptime::run(&pregen, &cfg);
    let mut worst = 0.0f64;
    for (i, s) in run.grid.strata.iter().enumerate() {
        let diff = (s.total_m() - run.grid.h[i]).abs();
        worst = worst.max(diff);
    }
    assert!(
        worst < 1e-6,
        "recorder invariant sum(units)==H violated, worst {worst}"
    );
}

#[test]
fn erosion_actually_moves_material() {
    // Sanity: over the run, some cells deposit alluvium and some records carry
    // multiple tagged units — the recorder is catching a real history, not a
    // dead field.
    let pregen = small_world(SEED);
    let cfg = test_cfg(SEED);
    let run = deeptime::run(&pregen, &cfg);
    let deposited: f64 = run.grid.h.iter().sum();
    assert!(deposited > 0.0, "no alluvium was deposited anywhere");
    let multi_unit = run
        .grid
        .strata
        .iter()
        .filter(|s| s.units.len() >= 2)
        .count();
    assert!(
        multi_unit > 0,
        "no column recorded a multi-unit history — the recorder saw nothing"
    );
}
