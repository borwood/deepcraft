//! **S17 keystone — deep-cell working inventory + transformation-fact ledger.**
//! Falsifiers and the memory measurement for
//! `docs/spikes/S17-deep-cell-inventory-results.md`.
//!
//! The load-bearing proofs, both over a *real* production `DeepField` (built by
//! unmodified deep-time code on merged main):
//!
//! 1. **Identity default is byte-free.** Build the working inventory from
//!    `base + empty-ledger`, run **no** behavior, commit → **zero facts** for every
//!    cell, and the strata record's `units` (the depositional base) are never
//!    written. The record is the pre-spike authority; nothing here rewrites it.
//! 2. **Non-identity agreement.** A synthetic behavior applies a known edge on a
//!    real cell → commit appends the expected fact → re-derive `base + facts`
//!    returns the changed composition and the provenance read returns the fact.
//!
//! (`build_field` / `run_cells` / `collapse.rs` are untouched, so the collapsed
//! world stays byte-identical to merged main regardless.)

use dc_worldgen::deeptime::{
    self, Cause, DeepField, FactLedger, Granularity, InvForm, InvSpan, UnitProvenance,
    build_identity, build_working, commit_chapter, compose_unit, derive_base,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn small_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    deeptime::build_field(&pregen.grid, SEED)
}

#[test]
fn identity_default_appends_no_facts_over_a_whole_field() {
    let field = small_field();
    assert!(!field.strata.is_empty());
    let mut checked = 0usize;
    for strata in &field.strata {
        let mut ledger = FactLedger::empty_with_bedrock(strata);
        let mut inv = build_working(strata, &ledger);
        commit_chapter(&mut inv, &mut ledger);
        assert!(
            ledger.is_empty(),
            "cell {checked}: identity default must append no facts"
        );
        for (ui, u) in strata.units.iter().enumerate() {
            assert_eq!(compose_unit(u, ledger.facts_for(ui)), derive_base(u));
        }
        checked += 1;
    }
    assert_eq!(checked, field.strata.len());
    assert!(checked > 1000, "expected a few thousand cells on Small");
}

#[test]
fn non_identity_agreement_on_a_real_cell() {
    // Find a real cell with a recorded unit, apply a known material-change edge to
    // its bottom span, commit, and assert the fact re-derives + reads back.
    let field = small_field();
    let strata = field
        .strata
        .iter()
        .find(|s| s.units.first().is_some_and(|u| u.thickness_m() > 0.1))
        .expect("some cell has a unit thick enough to transform");

    let mut ledger = FactLedger::empty_with_bedrock(strata);
    let base_mat = strata.units[0].species();
    let sink = dc_core::MaterialId::CLAY;

    let mut inv = build_working(strata, &ledger);
    let avail = inv
        .ctx_for(5, Cause::Chemical)
        .fraction(0, base_mat, InvForm::Loose);
    let moved = inv.ctx_for(5, Cause::Chemical).apply_edge(
        0,
        (base_mat, InvForm::Loose),
        (sink, InvForm::Loose),
        avail * 0.5,
    );
    assert!(moved > 0.0);

    commit_chapter(&mut inv, &mut ledger);
    assert_eq!(ledger.total_facts(), 1, "one edge → one fact");
    let f = ledger.facts_for(0)[0];
    assert_eq!(f.chapter(), 5);
    assert_eq!(f.cause(), Cause::Chemical);
    assert_eq!(f.from(), (base_mat, InvForm::Loose));
    assert_eq!(f.to(), (sink, InvForm::Loose));
    assert!((f.fraction_m() - moved).abs() < 1e-12);

    let prov = UnitProvenance::of(strata, &ledger, 0).unwrap();
    let comp = prov.compose();
    let sink_qty: f64 = comp
        .iter()
        .filter(|p| p.material == sink)
        .map(|p| p.quantity_m)
        .sum();
    assert!((sink_qty - moved).abs() < 1e-12);
    assert_eq!(prov.facts().len(), 1);
}

#[test]
fn memory_measurement_per_stratum_vs_per_voxel() {
    let field = small_field();
    let cells = field.strata.len();
    let voxel_m = 0.9;

    let mut total_units = 0usize;
    let mut total_h = 0.0f64;
    let mut max_units = 0usize;
    let mut max_h = 0.0f64;
    let mut per_stratum_bytes = 0usize;
    let mut per_voxel_bytes = 0usize;
    let mut per_voxel_spans = 0usize;
    for s in &field.strata {
        total_units += s.units.len();
        total_h += s.total_m();
        max_units = max_units.max(s.units.len());
        max_h = max_h.max(s.total_m());
        let ps = build_identity(s, Granularity::PerStratum);
        let pv = build_identity(s, Granularity::PerVoxel { voxel_m });
        per_stratum_bytes += ps.footprint_bytes();
        per_voxel_bytes += pv.footprint_bytes();
        per_voxel_spans += pv.spans.len();
    }

    // Illustrative THICK column (H=20 m, 4 units) — where the granularities diverge.
    let thick = {
        use dc_worldgen::deeptime::{Aridity, DeepStrata, DepEnv, DepTag, EnergyBand};
        let mut s = DeepStrata::default();
        let t = |env, energy| DepTag::mineral(env, Aridity::Humid, energy);
        s.deposit(t(DepEnv::Subsea, EnergyBand::Low), 8.0, 0);
        s.deposit(t(DepEnv::Subaerial, EnergyBand::High), 5.0, 0);
        s.deposit(t(DepEnv::Subaerial, EnergyBand::Low), 4.0, 0);
        s.deposit(t(DepEnv::Subaerial, EnergyBand::Medium), 3.0, 0);
        s
    };
    let thick_ps = build_identity(&thick, Granularity::PerStratum);
    let thick_pv = build_identity(&thick, Granularity::PerVoxel { voxel_m });

    let ps_per_cell = per_stratum_bytes as f64 / cells as f64;
    let pv_per_cell = per_voxel_bytes as f64 / cells as f64;
    let prod_cap_cells = (deeptime::DEEP_MAX_WIDTH * deeptime::DEEP_MAX_WIDTH) as f64;
    let medium_cells = 491.0 * 491.0;
    let mb = |b: f64| b / (1024.0 * 1024.0);

    println!("=== S17 deep-cell inventory memory measurement (Small, seed {SEED:#x}) ===");
    println!("cells                     : {cells}");
    println!("total record units        : {total_units}");
    println!(
        "mean units/cell           : {:.2}   (max {max_units})",
        total_units as f64 / cells as f64
    );
    println!(
        "mean H (m)                : {:.3}   (max {max_h:.2})",
        total_h / cells as f64
    );
    println!(
        "sizeof InvSpan            : {} B",
        std::mem::size_of::<InvSpan>()
    );
    println!("--- footprint on THIS field ---");
    println!(
        "per-stratum               : {:.3} MiB ({:.1} B/cell)",
        mb(per_stratum_bytes as f64),
        ps_per_cell
    );
    println!(
        "per-voxel (0.9 m)         : {:.3} MiB ({:.1} B/cell, {} spans)",
        mb(per_voxel_bytes as f64),
        pv_per_cell,
        per_voxel_spans
    );
    println!(
        "current DeepField resident: {:.3} MiB",
        mb(field.resident_bytes() as f64)
    );
    println!("--- extrapolated (per-stratum) ---");
    println!(
        "Medium (~241k cells)      : {:.2} MiB",
        mb(ps_per_cell * medium_cells)
    );
    println!(
        "cap ({}² cells)           : {:.2} MiB",
        deeptime::DEEP_MAX_WIDTH,
        mb(ps_per_cell * prod_cap_cells)
    );
    println!("--- illustrative THICK column (H=20 m, 4 units) ---");
    println!(
        "per-stratum spans/bytes   : {} / {} B",
        thick_ps.spans.len(),
        thick_ps.footprint_bytes()
    );
    println!(
        "per-voxel  spans/bytes    : {} / {} B ({:.2}× heavier)",
        thick_pv.spans.len(),
        thick_pv.footprint_bytes(),
        thick_pv.footprint_bytes() as f64 / thick_ps.footprint_bytes() as f64
    );

    assert!(per_voxel_bytes >= per_stratum_bytes);
    assert!(per_voxel_spans >= total_units);
    assert!(thick_pv.footprint_bytes() > thick_ps.footprint_bytes());
    assert!(thick_pv.spans.len() > thick_ps.spans.len());
}
