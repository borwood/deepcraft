//! **S17 — deep-cell working-inventory spike.** The falsifiers and the memory
//! measurement for `docs/spikes/S17-deep-cell-inventory-results.md`.
//!
//! The load-bearing proof: over a *real* production `DeepField` (built by
//! unmodified deep-time code at this branch point, commit 606f17a), the
//! identity-default working inventory committed straight back reproduces every
//! cell's strata record **byte-identically**. The record is the pre-spike golden
//! by construction — nothing in this spike touches `run_cells` / `build_field` /
//! the collapse path — so this is the "seam is free when empty" proof on live
//! data, not a self-captured (circular) golden.

use dc_worldgen::deeptime::{
    self, DeepField, Granularity, InvSpan, WorkingInventory, build_identity, commit_chapter,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn small_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    // Production config: all flags on (tectonic history, full agents, biotic), so
    // the record carries realistic multi-tag columns.
    deeptime::build_field(&pregen.grid, SEED)
}

#[test]
fn identity_default_commit_is_byte_identical_over_a_whole_field() {
    let field = small_field();
    assert!(
        !field.strata.is_empty(),
        "the record grid must be populated (production config)"
    );
    let mut checked = 0usize;
    for original in &field.strata {
        let inv = build_identity(original, Granularity::PerStratum);
        let mut committed = original.clone();
        commit_chapter(&inv, &mut committed);
        assert_eq!(
            &committed, original,
            "cell {checked}: identity-default commit must reproduce the record byte-identically"
        );
        checked += 1;
    }
    assert_eq!(checked, field.strata.len());
    assert!(checked > 1000, "expected a few thousand cells on Small");
}

#[test]
fn memory_measurement_per_stratum_vs_per_voxel() {
    let field = small_field();
    let cells = field.strata.len();
    let voxel_m = 0.9; // one collapse voxel

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

    // Illustrative THICK column (H = 20 m, 4 tag units) — where per-voxel and
    // per-stratum diverge. On this Small world mean H is tiny (thin cover), so
    // every unit is < one voxel and the two granularities coincide there; the
    // divergence only appears where a column is many voxels deep.
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

    let span_sz = std::mem::size_of::<InvSpan>();
    let inv_sz = std::mem::size_of::<WorkingInventory>();
    let field_resident = field.resident_bytes();

    // Production-scale extrapolation: bytes-per-cell × the capped production grid
    // (DEEP_MAX_WIDTH² cells) and × the Medium production grid (~241k cells).
    let ps_per_cell = per_stratum_bytes as f64 / cells as f64;
    let pv_per_cell = per_voxel_bytes as f64 / cells as f64;
    let prod_cap_cells = (deeptime::DEEP_MAX_WIDTH * deeptime::DEEP_MAX_WIDTH) as f64;
    let medium_cells = 491.0 * 491.0; // ~226 km / 460 m

    let mb = |b: f64| b / (1024.0 * 1024.0);
    println!("=== S17 deep-cell inventory memory measurement (Small, seed {SEED:#x}) ===");
    println!("cells                     : {cells}");
    println!("total record units        : {total_units}");
    println!(
        "mean units/cell           : {:.2}",
        total_units as f64 / cells as f64
    );
    println!("mean H (m)                : {:.3}", total_h / cells as f64);
    println!("max units/cell            : {max_units}");
    println!("max H (m)                 : {max_h:.2}");
    println!("sizeof InvSpan            : {span_sz} B");
    println!("sizeof WorkingInventory   : {inv_sz} B");
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
        "per-voxel / per-stratum   : {:.2}×",
        per_voxel_bytes as f64 / per_stratum_bytes as f64
    );
    println!(
        "current DeepField resident: {:.3} MiB",
        mb(field_resident as f64)
    );
    println!("--- extrapolated to production grids ---");
    println!(
        "per-stratum @ Medium (~241k): {:.2} MiB",
        mb(ps_per_cell * medium_cells)
    );
    println!(
        "per-voxel   @ Medium (~241k): {:.2} MiB",
        mb(pv_per_cell * medium_cells)
    );
    println!(
        "per-stratum @ cap ({}²)     : {:.2} MiB",
        deeptime::DEEP_MAX_WIDTH,
        mb(ps_per_cell * prod_cap_cells)
    );
    println!(
        "per-voxel   @ cap ({}²)     : {:.2} MiB",
        deeptime::DEEP_MAX_WIDTH,
        mb(pv_per_cell * prod_cap_cells)
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

    // Falsifiers. On the field, per-voxel is never cheaper than per-stratum
    // (equal here because the cover is thin — mean H << voxel_m). In a thick
    // column per-voxel is strictly heavier, which is the divergence that decides
    // the recommendation.
    assert!(per_voxel_bytes >= per_stratum_bytes);
    assert!(per_voxel_spans >= total_units);
    assert!(thick_pv.footprint_bytes() > thick_ps.footprint_bytes());
    assert!(thick_pv.spans.len() > thick_ps.spans.len());
}
