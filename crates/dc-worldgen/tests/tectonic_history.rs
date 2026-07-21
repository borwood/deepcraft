//! Tectonic-history spike (tectonics.md § SPIKE) — the falsifiers.
//!
//! Non-negotiable groups: **byte-identity off** (the flag and every tectonic
//! knob may not perturb the legacy path), **determinism on** (double-run and
//! scalar↔parallel byte-identity), **clamp stability** (nothing non-finite, no
//! runaway/stall across chapter repaints), **forcing wavelength** (the 50 km
//! smear dies — the analytic belt half-width tracks `orogen_width_km`), and the
//! two mass ledgers (the R+H stock with isostasy declared as its external input,
//! and the thickness stock `Δ(Σt_crust) == thickening − exhumation`). Measured
//! numbers live in docs/spikes/S12-results.md; these assert the invariants.

use dc_worldgen::deeptime::tectonics;
use dc_worldgen::deeptime::{self, DeepConfig, Plate};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn small_world(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

/// A base tectonic-history config on a coarse cell (fast, and past nothing that
/// needs fine resolution for the invariants).
fn tec_cfg(seed: u64) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m: 1000.0,
        iterations: 40,
        remarch_interval: 20,
        record: true,
        tectonic_history: true,
        ..DeepConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Group 6 — byte-identity OFF and determinism ON.

/// The flag off, and **every tectonic knob varied**, must reproduce the plain
/// legacy run to the bit. The strongest off-path proof: the knobs exist on the
/// struct but touch nothing while `tectonic_history` is false.
#[test]
fn flag_off_is_byte_identical_regardless_of_tectonic_knobs() {
    let pregen = small_world(SEED);
    let base = DeepConfig {
        seed: SEED,
        cell_m: 1000.0,
        iterations: 40,
        ..DeepConfig::default()
    };
    let weird = DeepConfig {
        plate_scale_km: 30.0,
        chapters: 16,
        ramp_chapters: false,
        orogen_width_km: 10.0,
        arc_gap_km: 90.0,
        advection_plate_widths: 2.5,
        thickening_scale: 200.0,
        flex_wavelength_km: 90.0,
        iso_rate: 0.9,
        ..base // tectonic_history stays false
    };
    let a = deeptime::run(&pregen, &base);
    let b = deeptime::run(&pregen, &weird);
    assert_eq!(
        a.grid.r, b.grid.r,
        "bedrock must ignore tectonic knobs when off"
    );
    assert_eq!(
        a.grid.h, b.grid.h,
        "alluvium must ignore tectonic knobs when off"
    );
    assert_eq!(
        a.grid.strata, b.grid.strata,
        "record must be byte-identical when off"
    );
}

/// Off path leaves every tectonic-only plane empty and stamps chapter 0.
#[test]
fn flag_off_allocates_no_tectonic_state_and_stamps_chapter_zero() {
    let pregen = small_world(SEED);
    let run = deeptime::run(
        &pregen,
        &DeepConfig {
            seed: SEED,
            cell_m: 1000.0,
            iterations: 30,
            ..DeepConfig::default()
        },
    );
    assert!(run.grid.t_crust.is_empty());
    assert!(run.grid.crust_kind.is_empty());
    assert!(run.grid.exhum.is_empty());
    assert!(run.chapters.is_empty());
    assert_eq!(run.thickening_total, 0.0);
    for s in &run.grid.strata {
        for u in &s.units {
            assert_eq!(u.chapter, 0, "off path must stamp chapter 0");
        }
    }
}

#[test]
fn tectonic_runs_are_byte_identical_on_a_repeated_seed() {
    let pregen = small_world(SEED);
    let cfg = tec_cfg(SEED);
    let a = deeptime::run(&pregen, &cfg);
    let b = deeptime::run(&pregen, &cfg);
    assert_eq!(a.grid.r, b.grid.r);
    assert_eq!(a.grid.h, b.grid.h);
    assert_eq!(a.grid.t_crust, b.grid.t_crust);
    assert_eq!(a.grid.exhum, b.grid.exhum);
    assert_eq!(a.grid.strata, b.grid.strata);
}

#[test]
fn tectonic_parallel_equals_scalar_byte_identical() {
    // Finer cell to force past the rayon fork floor: the per-cell thickening,
    // exhumation and record phases really fork; the isostasy smoothing is scalar
    // in both drivers by construction.
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        cell_m: 120.0,
        iterations: 20,
        ..tec_cfg(SEED)
    };
    let scalar = deeptime::run_with(&pregen, &cfg, false);
    let parallel = deeptime::run_with(&pregen, &cfg, true);
    assert_eq!(scalar.grid.r, parallel.grid.r, "bedrock parallel != scalar");
    assert_eq!(
        scalar.grid.h, parallel.grid.h,
        "alluvium parallel != scalar"
    );
    assert_eq!(
        scalar.grid.t_crust, parallel.grid.t_crust,
        "t_crust parallel != scalar"
    );
    assert_eq!(
        scalar.grid.strata, parallel.grid.strata,
        "record parallel != scalar"
    );
    assert_eq!(
        scalar.uplift_total, parallel.uplift_total,
        "ledger parallel != scalar"
    );
}

#[test]
fn tectonic_history_changes_the_world() {
    let pregen = small_world(SEED);
    let off = deeptime::run(
        &pregen,
        &DeepConfig {
            seed: SEED,
            cell_m: 1000.0,
            iterations: 40,
            ..DeepConfig::default()
        },
    );
    let on = deeptime::run(&pregen, &tec_cfg(SEED));
    assert_ne!(off.grid.r, on.grid.r, "tectonic history changed nothing");
}

// ---------------------------------------------------------------------------
// Group 1 — clamp / numerical stability under repainting.

/// The whole stack on (tectonic history + erodibility + biology + full agents),
/// high erodibility contrast, across chapter repaints: nothing non-finite, and
/// the world neither stalls (no material moved) nor runs away (finite relief).
#[test]
fn stack_is_finite_and_stable_across_chapter_repaints() {
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        erodibility: true,
        erodibility_contrast: 4.0,
        biotic: true,
        full_agents: true,
        chapters: 8,
        ..tec_cfg(SEED)
    };
    let run = deeptime::run(&pregen, &cfg);
    for i in 0..run.grid.w * run.grid.w {
        assert!(run.grid.r[i].is_finite(), "non-finite bedrock at {i}");
        assert!(
            run.grid.h[i].is_finite() && run.grid.h[i] >= -1e-6,
            "bad alluvium at {i}"
        );
        assert!(
            run.grid.t_crust[i].is_finite() && run.grid.t_crust[i] > 0.0,
            "bad crust at {i}"
        );
        assert!(
            run.grid.exhum[i].is_finite() && run.grid.exhum[i] >= 0.0,
            "bad exhum at {i}"
        );
    }
    let moved: f64 = run.grid.h.iter().sum();
    assert!(moved > 0.0, "no material moved — the engine stalled");
}

// ---------------------------------------------------------------------------
// Group 5 — forcing wavelength: the 50 km smear dies.

/// A fresh continent–continent belt's analytic forcing has a half-width set by
/// `orogen_width_km`, NOT the 14.7 km pregen cell / 50 km smear. Sample the
/// forcing across the bisector of two head-on continental plates and measure the
/// distance from the peak to the half-maximum: it must be a small multiple of
/// `orogen_width_km` and far below the 50 km artifact.
#[test]
fn belt_forcing_half_width_tracks_orogen_width_not_the_50km_smear() {
    let cfg = DeepConfig {
        tectonic_history: true,
        orogen_width_km: 25.0,
        ..DeepConfig::default()
    };
    // Two continental plates, 100 km apart, closing head-on; bisector at x=150.
    let s = 4.0;
    let plates = vec![
        Plate {
            x: 100.0,
            y: 150.0,
            vx: s,
            vy: 0.0,
            continental: true,
        },
        Plate {
            x: 200.0,
            y: 150.0,
            vx: -s,
            vy: 0.0,
            continental: true,
        },
    ];
    let v_ref = 8.0; // vc_norm ≈ 2s/v_ref = 1
    // Sample forcing across the belt.
    let peak = tectonics::forcing_at(&plates, &cfg, v_ref, 150.0, 150.0);
    assert!(peak > 0.0, "no belt built at the boundary (peak {peak})");
    // Walk outward from the axis to the half-max crossing.
    let mut half_width = f64::INFINITY;
    let mut x = 150.0;
    while x < 200.0 {
        let f = tectonics::forcing_at(&plates, &cfg, v_ref, x, 150.0);
        if f <= peak * 0.5 {
            half_width = x - 150.0;
            break;
        }
        x += 0.25;
    }
    assert!(
        half_width < 40.0,
        "belt half-width {half_width} km did not collapse below the 50 km smear"
    );
    assert!(
        half_width > 5.0,
        "belt half-width {half_width} km is implausibly sharp for W=25"
    );
}

// ---------------------------------------------------------------------------
// Group — the two mass ledgers.

/// The R+H stock: `Δ(ΣR+ΣH) == uplift_total + biotic_total`, where `uplift_total`
/// is the isostatic bedrock injection (declared as an external input, the
/// `biotic_total` pattern). Exact up to fp slack.
#[test]
fn r_plus_h_ledger_balances_with_isostasy_as_declared_input() {
    let pregen = small_world(SEED);
    let run = deeptime::run(&pregen, &tec_cfg(SEED));
    let after = deeptime::total_mass(&run.grid);
    let residual = after - run.mass_before - run.uplift_total - run.biotic_total;
    let tol = (after.abs() * 1e-9).max(1.0);
    assert!(
        residual.abs() < tol,
        "R+H ledger leaked: residual {residual} (tol {tol}, after {after}, inject {})",
        run.uplift_total
    );
}

/// The thickness stock: `Δ(Σt_crust) == thickening_total − exhum_total` (§ 3.4).
/// The conserved stock migrates to thickness — thickening adds, exhumation
/// removes, and nothing else touches `t_crust`.
#[test]
fn thickness_ledger_balances() {
    let pregen = small_world(SEED);
    let cfg = tec_cfg(SEED);
    // Initial Σt_crust from a fresh build (the run consumes the grid).
    let grid0 = deeptime::build(&pregen, &cfg);
    let t_before: f64 = grid0.t_crust.iter().sum();
    let run = deeptime::run(&pregen, &cfg);
    let t_after: f64 = run.grid.t_crust.iter().sum();
    let residual = (t_after - t_before) - (run.thickening_total - run.exhum_total);
    let tol = (t_after.abs() * 1e-9).max(1.0);
    assert!(
        residual.abs() < tol,
        "thickness ledger leaked: residual {residual} (tol {tol}, Δt {}, thicken {}, exhum {})",
        t_after - t_before,
        run.thickening_total,
        run.exhum_total
    );
}

// ---------------------------------------------------------------------------
// Recorder — sizeof unchanged, and merge-break bounded.

#[test]
fn dep_unit_sizeof_is_unchanged_by_the_chapter_stamp() {
    // The stamp is expected to fit DepUnit's existing 8-byte-aligned padding.
    assert_eq!(std::mem::size_of::<deeptime::DepUnit>(), 16);
}

/// The recorder invariant `sum(units) == H` survives the chapter-stamp merge key.
#[test]
fn recorder_total_equals_alluvium_with_tectonic_history() {
    let pregen = small_world(SEED);
    let run = deeptime::run(&pregen, &tec_cfg(SEED));
    let mut worst = 0.0f64;
    for (i, s) in run.grid.strata.iter().enumerate() {
        worst = worst.max((s.total_m() - run.grid.h[i]).abs());
    }
    assert!(worst < 1e-6, "sum(units)==H violated, worst {worst}");
}

// ---------------------------------------------------------------------------
// Group 8 — drainage export fidelity.

#[test]
fn drainage_export_is_populated_and_accounts_for_every_cell() {
    let pregen = small_world(SEED);
    let cfg = tec_cfg(SEED);
    let field = deeptime::build_field_cfg(&pregen.grid, &cfg);
    let n = field.w * field.w;
    assert_eq!(field.recv.len(), n, "recv not exported");
    assert_eq!(field.area.len(), n, "area not exported");
    assert_eq!(field.lake.len(), n, "lake mask not exported");
    assert!(!field.chapters.is_empty(), "chapter table not exported");
    // Every cell's unit area drains to a sink: Σ area over sinks == n exactly.
    let sink_area: f64 = (0..n)
        .filter(|&i| field.recv[i] < 0)
        .map(|i| field.area[i])
        .sum();
    assert!(
        (sink_area - n as f64).abs() < 1e-6,
        "drainage does not account for every cell: sink area {sink_area} vs n {n}"
    );
}

#[test]
fn drainage_export_is_empty_off_the_flag() {
    let pregen = small_world(SEED);
    // Since the U8 flip (journal/0044) production runs tectonics ON, take the
    // production config and force the flag back OFF — the off path must still
    // export no drainage / chapters.
    let mut cfg = deeptime::production_config(&pregen.grid, SEED);
    cfg.tectonic_history = false;
    let field = deeptime::build_field_cfg(&pregen.grid, &cfg);
    assert!(field.recv.is_empty());
    assert!(field.area.is_empty());
    assert!(field.chapters.is_empty());
}
