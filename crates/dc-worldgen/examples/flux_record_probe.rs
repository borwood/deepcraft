//! **The FLOW slice 1 acceptance probe** (`docs/design/flow.md`, journal/0096).
//!
//! Builds the **production world** — seed 1337, `Extent::Medium`, production
//! flags — and reports the two numbers the slice exists to produce, plus the cost
//! of producing them:
//!
//! 1. **DIVERGENCE count** — `(cell, chapter)` pairs whose flux left through two
//!    or more faces. A receiver tree's count of this is *identically zero, by
//!    construction, forever*; it is the number that proves the primitive changed.
//!    Zero here means the slice failed, and the probe says so in those words.
//! 2. **CONVERGENCE count** — the half a tree could already do, which must still
//!    be there.
//! 3. **Measured resident cost** — total bytes and the per-cell / per-chapter
//!    breakdown, plus the **face sparsity** the S19 cost model was missing (gen
//!    time is free; residency is not — flow.md § 9.1).
//!
//! Run: `cargo run --release -p dc-worldgen --example flux_record_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{DeepField, FaceKey, build_field};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Build the deep field this probe reads. Shared by `main` (which prints the
/// full report at production scale) and the gate test below (which asserts the
/// acceptance claims at a small scale) — journal/0103: `cargo test` builds
/// examples but never runs them, so an instrument's claim only reaches the gate
/// if a `#[test]` shares the instrument's code.
fn field(extent: Extent) -> DeepField {
    let pregen = Pregen::run(WorldParams { seed: SEED, extent });
    build_field(&pregen.grid, SEED)
}

fn main() {
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t_pregen = t0.elapsed();
    let t1 = Instant::now();
    let f: DeepField = build_field(&pregen.grid, SEED);
    let t_deep = t1.elapsed();

    let rec = &f.flux;
    let c = rec.census();
    let cells = c.cells;
    let chapters = c.chapters as usize;

    println!("=== FLOW slice 1 — face-flux record, production world ===");
    println!(
        "seed {SEED} · Extent::Medium · deep grid {w}×{w} = {cells} cells · {chapters} chapters",
        w = rec.w
    );
    println!(
        "pregen {:.1} s · deep-time run {:.1} s",
        t_pregen.as_secs_f64(),
        t_deep.as_secs_f64()
    );

    // ---- 1/2. the load-bearing counts -------------------------------------
    println!("\n--- ACCEPTANCE ---");
    println!(
        "DIVERGENCE  (cell,chapter) with >=2 out-faces : {:>12}   ({:.3} % of pairs)",
        c.divergent,
        c.divergent as f64 * 100.0 / (cells * chapters) as f64
    );
    println!(
        "CONVERGENCE (cell,chapter) with >=2 in-faces  : {:>12}   ({:.3} % of pairs)",
        c.convergent,
        c.convergent as f64 * 100.0 / (cells * chapters) as f64
    );
    println!(
        "max out-faces on one cell/chapter             : {:>12}",
        c.max_out_faces
    );
    println!(
        "max in-faces  on one cell/chapter             : {:>12}",
        c.max_in_faces
    );
    if c.divergent == 0 {
        println!(
            "\n*** FAIL: divergence count is ZERO. Every cell has at most one \
             out-face per chapter — which is exactly what the receiver tree \
             already did. The slice has NOT achieved its purpose. ***"
        );
    } else {
        println!(
            "\nPASS: the record holds {} divergent (cell,chapter) junctions. A \
             receiver tree's count of these is 0 by construction.",
            c.divergent
        );
    }

    // ---- how the faces are used -------------------------------------------
    println!("\n--- FACES USED ({} entries) ---", c.entries);
    println!(
        "lateral  (cell<->cell)      : {:>12}  ({:.2} %)",
        c.lateral,
        c.lateral as f64 * 100.0 / c.entries.max(1) as f64
    );
    // **This caption was stale and the gate could not see it** (journal/0103).
    // Slice 1 recorded these structurally and left them honestly zero; the named
    // heir *landed* — `dc:field/head` (journal/0098), ON by default — so the
    // probe was printing a non-zero count beside the word EMPTY. Nothing failed,
    // because `cargo test` never runs an example. The remaining heir (the
    // free/bound edge) is named as still-outstanding, which it is.
    println!(
        "vertical (slot<->slot)      : {:>12}  <- filled by the head field \
         (dc:field/head, journal/0098); zero only under --no-head-field. \
         Remaining heir: the free/bound edge.",
        c.vertical
    );
    println!("boundary  ocean (sea stand)  : {:>12}", c.boundary_ocean);
    println!("boundary  base level (border): {:>12}", c.boundary_base);
    println!(
        "boundary  atmosphere         : {:>12}  <- evaporative sink at closed-basin \
         termini only; the precipitation SOURCE is exactly derivable and is not stored (S-2)",
        c.boundary_atmosphere
    );

    // ---- is the LOAD channel actually carrying anything? -------------------
    // A field that is always zero is indistinguishable from a field that is not
    // wired up, so say which one it is with a number.
    let with_load = rec.entries().iter().filter(|e| e.load > 0.0).count();
    let total_load: f64 = rec.entries().iter().map(|e| f64::from(e.load)).sum();
    let max_load = rec.entries().iter().map(|e| e.load).fold(0.0f32, f32::max);
    println!("\n--- LOAD (the atom's L, bulk; composition is Movement 2b's seam) ---");
    println!(
        "entries carrying load        : {:>12}  ({:.2} % of all entries, {:.2} % of lateral)",
        with_load,
        with_load as f64 * 100.0 / c.entries.max(1) as f64,
        with_load as f64 * 100.0 / c.lateral.max(1) as f64
    );
    println!("total load across all faces  : {total_load:>16.3} m");
    println!("largest single-face load     : {max_load:>16.3} m");

    // ---- the residency lever, sized but NOT taken --------------------------
    // 81 %-ish of entries are marine sinks. A subsea cell that nothing drains
    // into records exactly the seeded `area = 1.0` per epoch — the constant the
    // solve puts there, which S-2 says derivation can predict. Sizing it here so
    // the call is the user's, with a number, rather than mine, silently.
    let epochs_per_chapter = 200.0 / f64::from(c.chapters.max(1));
    let self_source_only = rec
        .entries()
        .iter()
        .filter(|e| {
            e.face == FaceKey::Ocean
                && e.load == 0.0
                && f64::from(e.magnitude) <= epochs_per_chapter + 1e-6
        })
        .count();
    println!("\n--- RESIDENCY LEVER (sized, NOT taken — the user's call) ---");
    println!(
        "ocean-sink entries carrying ONLY the cell's own seeded 1.0/epoch source \
         (i.e. nothing drained through them): {self_source_only} = {:.2} % of entries, \
         {:.2} MiB of the record",
        self_source_only as f64 * 100.0 / c.entries.max(1) as f64,
        mib(self_source_only * 16)
    );
    println!(
        "dropping them would leave {:.2} MiB and lose no information a consumer \
         cannot derive from the absence — but it is a change to what the record \
         MEANS, so it is not taken here.",
        mib(rec.resident_bytes() - self_source_only * 16)
    );

    // ---- 3. the measured cost ---------------------------------------------
    let flux_bytes = rec.resident_bytes();
    let field_bytes = f.resident_bytes();
    let base_bytes = field_bytes - flux_bytes;
    println!("\n--- MEASURED RESIDENT COST ---");
    println!(
        "flow record total            : {:>12} B  ({:.2} MiB)",
        flux_bytes,
        mib(flux_bytes)
    );
    println!(
        "  entries {} x 16 B          : {:>12} B  ({:.2} MiB)",
        c.entries,
        c.entries * 16,
        mib(c.entries * 16)
    );
    println!(
        "  CSR index ({} + 1) x 4 B   : {:>12} B  ({:.2} MiB)",
        cells,
        (cells + 1) * 4,
        mib((cells + 1) * 4)
    );
    println!(
        "per cell                     : {:>12.2} B   ({:.3} entries/cell)",
        flux_bytes as f64 / cells as f64,
        c.entries as f64 / cells as f64
    );
    println!(
        "per cell per chapter         : {:>12.2} B   ({:.3} entries/cell/chapter)",
        flux_bytes as f64 / (cells * chapters) as f64,
        c.entries_per_cell_chapter()
    );
    println!(
        "FACE SPARSITY (entries / cells x chapters x {FACE}) : {:.4} %   [lateral-only \
         rectangle: {:.4} %]",
        c.face_sparsity() * 100.0,
        c.lateral_sparsity() * 100.0,
        FACE = dc_worldgen::deeptime::FACE_SLOTS
    );
    println!(
        "DeepField without the record : {:>12} B  ({:.2} MiB)",
        base_bytes,
        mib(base_bytes)
    );
    println!(
        "DeepField with    the record : {:>12} B  ({:.2} MiB)   = {:.2}x",
        field_bytes,
        mib(field_bytes),
        field_bytes as f64 / base_bytes.max(1) as f64
    );

    // ---- a readable junction, so the numbers are not abstract --------------
    let last = rec.chapters - 1;
    let mut best: Option<(usize, u8, f32)> = None;
    for i in 0..cells {
        for k in 0..rec.chapters {
            if rec.out_face_count(i, k) < 2 {
                continue;
            }
            let m: f32 = rec.out_faces(i, k).map(|e| e.magnitude).sum();
            if best.is_none_or(|(_, _, bm)| m > bm) {
                best = Some((i, k, m));
            }
        }
    }
    if let Some((i, k, _)) = best {
        println!("\n--- the biggest divergence in the world: cell {i}, chapter {k} ---");
        let (gx, gy) = (i % rec.w, i / rec.w);
        println!(
            "deep cell ({gx}, {gy})  ~ ({:.1} km, {:.1} km) from the grid corner",
            gx as f64 * f.cell_m / 1000.0,
            gy as f64 * f.cell_m / 1000.0
        );
        for e in rec.out_faces(i, k) {
            println!(
                "  out  {:>4}  magnitude {:>14.1}  load {:>12.6}",
                e.face.label(),
                e.magnitude,
                e.load
            );
        }
        for (from, e) in rec.in_faces(i, k) {
            println!(
                "  in   {:>4}  from cell {from:>8}  magnitude {:>14.1}",
                e.face.opposite().unwrap_or(FaceKey::N).label(),
                e.magnitude
            );
        }
        println!(
            "  this cell's chapter-{k} stratum slot: {:?}",
            dc_worldgen::deeptime::slot_for_chapter(&f.strata[i], k)
        );
        // And the same cell in the final chapter — the history a tree discarded.
        println!("  same cell, final chapter {last}:");
        for e in rec.out_faces(i, last) {
            println!(
                "  out  {:>4}  magnitude {:>14.1}  load {:>12.6}",
                e.face.label(),
                e.magnitude,
                e.load
            );
        }
    }
}

/// **The gate's view of this instrument** (journal/0103).
///
/// The probe prints `*** FAIL ***` when divergence is zero and then exits 0, so
/// the only reader that could ever act on it was a human scrolling a log. The
/// same claim, as a test.
///
/// Run at [`Extent::Small`]. "Does the record hold a junction a receiver tree
/// cannot represent?" is a question about the **primitive**, not about grid
/// width: a tree's divergence count is identically zero at every scale, so any
/// non-zero count falsifies "this is still a tree". The *production* magnitudes
/// (how many junctions, how many MiB) are the example's job and stay at
/// [`Extent::Medium`].
#[cfg(test)]
mod gate {
    use super::*;

    #[test]
    fn the_flux_record_holds_junctions_a_receiver_tree_cannot() {
        let f = field(Extent::Small);
        let c = f.flux.census();
        assert!(
            c.entries > 0,
            "the flux record is EMPTY ({} cells x {} chapters) — nothing was recorded at all",
            c.cells,
            c.chapters
        );
        assert!(
            c.divergent > 0,
            "divergence count is ZERO over {} cells x {} chapters: every cell has at most \
             one out-face per chapter, which is exactly what the receiver tree already did. \
             max_out_faces = {}",
            c.cells,
            c.chapters,
            c.max_out_faces
        );
        assert!(
            c.convergent > 0,
            "convergence count is ZERO — the half a tree could already do has been lost"
        );
        assert!(
            c.max_out_faces >= 2,
            "max_out_faces = {} — a divergent junction must show at least two out-faces",
            c.max_out_faces
        );
    }

    /// The record must stay **sparse**: the whole residency argument (flow.md
    /// § 9.1) is that a `cells x chapters x FACE_SLOTS` rectangle is never
    /// allocated. A regression that started recording dense faces would still
    /// pass every correctness test and quietly multiply the field's heap.
    #[test]
    fn the_flux_record_stays_sparse_against_the_dense_rectangle() {
        let f = field(Extent::Small);
        let c = f.flux.census();
        let sparsity = c.face_sparsity();
        assert!(
            sparsity > 0.0 && sparsity < 0.25,
            "face sparsity {:.4} % is outside the sparse regime the cost model assumes \
             ({} entries over {} cells x {} chapters x {} faces)",
            sparsity * 100.0,
            c.entries,
            c.cells,
            c.chapters,
            dc_worldgen::deeptime::FACE_SLOTS,
        );
    }
}
