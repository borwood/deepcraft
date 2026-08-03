//! **Movement 2a — R/H unification: the derived-vs-scalar byte-identity proof.**
//!
//! material-behavior.md §13.6 (ratified): the deep-cell working material inventory
//! is the **authority** for surface material; the scalar `R`/`H` planes are its
//! **materialized views**. This suite proves the derivation reproduces the planes
//! over a *real `DeepField` built under the full production config* — the
//! load-bearing acceptance instrument, not a unit fixture.
//!
//! The honest byte-identical landing is **scratch-first reconcile**: the erosion
//! loop runs untouched on the scalar planes (so the goldens do not move — see
//! `providers_golden.rs`), and the persistent per-cell surface-`Loose` inventory
//! IS the strata record (the deposition pass reconciles each epoch's net ΔH into
//! it as `void→Loose`/`Loose→void` facts at the epoch boundary). These tests
//! materialize `H`/`R` from that inventory and assert they equal the stored planes
//! within the recorder residual — the proof the planes are derived views.
//!
//! The **positional/cave rule** (`H` = Loose above the topmost Structure, cave fill
//! excluded) is a unit test in `deeptime::inventory`
//! (`buried_loose_below_a_structure_is_excluded_from_surface_h`).

use dc_worldgen::deeptime::{DeepField, build_field};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The **golden fixture** world (matches `providers_golden.rs`: same seed,
/// `Extent::Small`, `build_field` = full production *config*).
///
/// **NOT the world `dc-client` boots** — that is seed `1337` at `Extent::Medium`
/// (`tests/geotherm.rs`). See **corrections #51** for what a "production"-named
/// helper on a non-shipped world cost us. The claim proved below — that a derived
/// view equals its stored plane — is genuinely seed-independent, so a fixed cheap
/// world is the right fixture; the misleading `production_*` name was retired for
/// `golden_*` on 2026-07-26.
const SEED: u64 = 0x0B0A_57EE_0059;

fn golden_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    build_field(&pregen.grid, SEED)
}

/// Tolerance (metres) for derived-vs-scalar agreement. Two derived terms, both
/// bounded: (a) f64 round-off between the incrementally-mutated `grid.h` and
/// the record's `Σ thickness` (the pre-pack residual, < 1e-6); (b) since P11
/// slice 3's packed record, the **sub-quantum carry** — the record is
/// quantized at 2⁻¹⁰ m and the residue rides per cell with `|carry| ≤ q/2`
/// (`recorder.rs`), so the derived view may sit up to half a quantum from the
/// unquantized plane. The bound is their sum, derived rather than fitted.
const TOL_M: f64 = 1e-6 + dc_worldgen::deeptime::DepUnit::THICKNESS_QUANTUM_M / 2.0;

#[test]
fn derived_regolith_agrees_with_the_scalar_h_plane_over_the_golden_field() {
    let f = golden_field();
    assert!(!f.regolith.is_empty(), "golden field carries the H plane");
    let mut max_res = 0.0f64;
    let mut worst = 0usize;
    for (i, &scalar_h) in f.regolith.iter().enumerate() {
        let d = (f.derive_regolith_at(i) - scalar_h).abs();
        if d > max_res {
            max_res = d;
            worst = i;
        }
    }
    println!(
        "H: max |derived - scalar| = {max_res:e} m over {} cells (worst cell {worst})",
        f.regolith.len()
    );
    assert!(
        max_res < TOL_M,
        "derived surface regolith H diverged from the scalar plane by {max_res:e} m \
         (> {TOL_M:e}) at cell {worst} — the inventory no longer reproduces H"
    );
}

#[test]
fn derived_bedrock_agrees_with_the_scalar_r_plane_over_the_golden_field() {
    let f = golden_field();
    // The scalar `R` plane is `grid.r` = bedrock-top elevation. The field keeps
    // `surf = r + h` and `regolith = h`, so the pre-slice scalar R is exactly
    // `surf - regolith` per cell. The derived R is `surf - H_derived`.
    let mut max_res = 0.0f64;
    let mut worst = 0usize;
    for (i, (&s, &h)) in f.surf.iter().zip(f.regolith.iter()).enumerate() {
        let scalar_r = s - h;
        let d = (f.derive_bedrock_at(i) - scalar_r).abs();
        if d > max_res {
            max_res = d;
            worst = i;
        }
    }
    println!(
        "R: max |derived - scalar| = {max_res:e} m over {} cells",
        f.surf.len()
    );
    assert!(
        max_res < TOL_M,
        "derived bedrock datum R diverged from the scalar plane by {max_res:e} m \
         (> {TOL_M:e}) at cell {worst}"
    );
}
