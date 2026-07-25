//! **The FLOW continuation (b) acceptance probe** — MFD routing
//! (`docs/design/flow.md` § 2.6, journal/0109).
//!
//! Slice 1's probe (`flux_record_probe`) proved the record could *hold*
//! divergence. It could not prove the solve ever *produced* any within one epoch,
//! because the record aggregates 25 epochs into a chapter and the distinction is
//! erased before a reader ever sees it. § 2.6 named the consequence precisely:
//!
//! > every divergence currently in the record is **TEMPORAL (avulsion)**, never
//! > **SIMULTANEOUS** (concurrent distributaries).
//!
//! So this probe reports **both counts side by side**, on the shipped world, with
//! MFD off and on, and refuses to let them be confused:
//!
//! | count | key | what it means |
//! |---|---|---|
//! | simultaneous | `(cell, epoch)` and `(cell, chapter)` | ≥2 faces carried flux **in one epoch** — concurrent distributaries |
//! | temporal | `(cell, chapter)` | ≥2 faces carried flux **at some point in the chapter** — avulsion |
//!
//! A receiver tree's simultaneous count is **identically zero, by construction,
//! forever**; if MFD's is zero or trivially small, that is a **null** and the
//! probe says so in those words.
//!
//! Run: `cargo run --release -p dc-worldgen --example mfd_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{DeepConfig, DeepField, build_field_cfg, production_config};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// One measured run of the deep-time ritual at a chosen routing.
struct Measured {
    field: DeepField,
    deep_secs: f64,
}

/// The numbers the acceptance rests on, pulled out of a field so `main` and the
/// gate test read the same instrument (journal/0103).
struct Counts {
    /// `(cell, epoch)` pairs with ≥2 lateral out-faces **within one epoch**.
    simul_cell_epochs: u64,
    /// Distinct `(cell, chapter)` pairs with at least one such epoch.
    simul_cell_chapters: u64,
    /// Most lateral out-faces any cell used in a single epoch.
    simul_max_faces: u32,
    /// `(cell, chapter)` pairs with ≥2 out-faces summed over the chapter — the
    /// **temporal** kind, which the single-receiver solve already produced. This
    /// is `census().divergent`, and it counts **every** face family.
    temporal_cell_chapters: u64,
    /// The same count restricted to **lateral** faces. It exists because the
    /// census's own number stopped being comparable to journal/0096's headline the
    /// day FLOW (a) landed: a cell that routes east *and* infiltrates downward has
    /// two out-faces and is counted as divergent, which is true of the record and
    /// is **not** what "a distributary" means. Quoted separately rather than
    /// silently.
    temporal_lateral_only: u64,
    entries: usize,
    /// Total suspended load carried across lateral faces, and how many faces carry
    /// any — the `L` channel's liveness. A permanently-zero field looks identical
    /// to an unwired one, and MFD rewrote how load is split.
    load_faces: usize,
    load_total: f64,
    record_bytes: usize,
    field_bytes: usize,
}

fn counts(f: &DeepField) -> Counts {
    let c = f.flux.census();
    let mut lateral_only = 0u64;
    for i in 0..f.flux.cells() {
        for k in 0..f.flux.chapters {
            if f.flux
                .out_faces(i, k)
                .filter(|e| e.face.is_lateral())
                .count()
                >= 2
            {
                lateral_only += 1;
            }
        }
    }
    let (load_faces, load_total) = f
        .flux
        .entries()
        .iter()
        .filter(|e| e.load > 0.0)
        .fold((0usize, 0.0f64), |(n, s), e| (n + 1, s + f64::from(e.load)));
    Counts {
        simul_cell_epochs: f.flux.simultaneous.cell_epochs,
        simul_cell_chapters: f.flux.simultaneous.cell_chapters,
        simul_max_faces: f.flux.simultaneous.max_out_faces,
        temporal_cell_chapters: c.divergent as u64,
        temporal_lateral_only: lateral_only,
        entries: c.entries,
        load_faces,
        load_total,
        record_bytes: f.flux.resident_bytes(),
        field_bytes: f.resident_bytes(),
    }
}

fn cfg(cells: &CellGrid, mfd: bool, p: f64) -> DeepConfig {
    DeepConfig {
        mfd,
        mfd_exponent: p,
        ..production_config(cells, SEED)
    }
}

fn measure(cells: &CellGrid, mfd: bool, p: f64) -> Measured {
    let t = Instant::now();
    let field = build_field_cfg(cells, &cfg(cells, mfd, p));
    Measured {
        field,
        deep_secs: t.elapsed().as_secs_f64(),
    }
}

/// Mean elevation and the relief the surface carries — the cheapest honest read
/// on "did the landscape change shape", printed beside the counts so a reader can
/// see the price of the capability without opening the game.
fn relief(f: &DeepField) -> (f64, f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for &z in &f.surf {
        lo = lo.min(z);
        hi = hi.max(z);
        sum += z;
    }
    (sum / f.surf.len() as f64, lo, hi)
}

fn main() {
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t_pregen = t0.elapsed().as_secs_f64();

    let off = measure(&pregen.grid, false, 0.0);
    let on = measure(&pregen.grid, true, DeepConfig::default().mfd_exponent);

    let (co, cn) = (counts(&off.field), counts(&on.field));
    let cells = on.field.flux.cells();
    let chapters = on.field.flux.chapters as usize;

    println!("=== FLOW continuation (b) — MFD routing, production world ===");
    println!(
        "seed {SEED} · Extent::Medium · deep grid {w}×{w} = {cells} cells · {chapters} chapters",
        w = on.field.flux.w
    );
    println!(
        "pregen {t_pregen:.1} s · deep run OFF {:.1} s · deep run ON {:.1} s",
        off.deep_secs, on.deep_secs
    );
    println!(
        "convergence exponent p = {}",
        DeepConfig::default().mfd_exponent
    );

    println!("\n--- ACCEPTANCE: SIMULTANEOUS divergence (within ONE epoch) ---");
    println!("                                        MFD OFF        MFD ON");
    println!(
        "(cell,epoch) pairs with >=2 out-faces : {:>12} {:>13}",
        co.simul_cell_epochs, cn.simul_cell_epochs
    );
    println!(
        "(cell,chapter) pairs, at least once   : {:>12} {:>13}   ({:.3} % of pairs)",
        co.simul_cell_chapters,
        cn.simul_cell_chapters,
        cn.simul_cell_chapters as f64 * 100.0 / (cells * chapters) as f64
    );
    println!(
        "max out-faces in one epoch            : {:>12} {:>13}",
        co.simul_max_faces, cn.simul_max_faces
    );

    println!("\n--- CONTROL: TEMPORAL divergence (avulsion, within a CHAPTER) ---");
    println!(
        "(cell,chapter) pairs, ALL faces       : {:>12} {:>13}",
        co.temporal_cell_chapters, cn.temporal_cell_chapters
    );
    println!(
        "(cell,chapter) pairs, LATERAL only    : {:>12} {:>13}",
        co.temporal_lateral_only, cn.temporal_lateral_only
    );
    println!(
        "  ^ the LATERAL row is what journal/0096 reported as 175,320. The ALL row\n    \
         grew when FLOW (a) landed, because a cell that routes east AND infiltrates\n    \
         downward has two out-faces — true of the record, not what 'a distributary'\n    \
         means. Neither is the same quantity as the SIMULTANEOUS block above, and\n    \
         § 2.6 exists because they must never be quoted as one."
    );

    if co.simul_cell_epochs != 0 {
        println!(
            "\n*** FAIL: single-receiver routing reported {} simultaneous \
             divergences. It cannot — a D8 receiver is one out-edge per cell per \
             epoch. The counter is wrong, not the physics. ***",
            co.simul_cell_epochs
        );
    } else if cn.simul_cell_epochs == 0 {
        println!(
            "\n*** NULL: MFD produced ZERO simultaneous divergence. The partition \
             never split a cell's discharge inside one epoch, so the slice bought \
             nothing the receiver tree did not already have. ***"
        );
    } else {
        println!(
            "\nPASS: {} (cell,epoch) simultaneous divergences, across {} distinct \
             (cell,chapter) pairs. The receiver tree's count of these is 0 by \
             construction, forever — concurrent distributaries are now not merely \
             representable but PRESENT.",
            cn.simul_cell_epochs, cn.simul_cell_chapters
        );
    }

    println!("\n--- COST ---");
    println!(
        "gen time (deep run)     : OFF {:>8.2} s   ON {:>8.2} s   delta {:+.2} s",
        off.deep_secs,
        on.deep_secs,
        on.deep_secs - off.deep_secs
    );
    println!(
        "flow record entries     : OFF {:>12}   ON {:>12}   x{:.2}",
        co.entries,
        cn.entries,
        cn.entries as f64 / co.entries.max(1) as f64
    );
    println!(
        "flow record residency   : OFF {:>9.2} MiB   ON {:>9.2} MiB   {:+.2} MiB",
        mib(co.record_bytes),
        mib(cn.record_bytes),
        mib(cn.record_bytes) - mib(co.record_bytes)
    );
    println!(
        "DeepField residency     : OFF {:>9.2} MiB   ON {:>9.2} MiB   {:+.2} MiB",
        mib(co.field_bytes),
        mib(cn.field_bytes),
        mib(cn.field_bytes) - mib(co.field_bytes)
    );
    println!(
        "load-carrying faces     : OFF {:>12}   ON {:>12}",
        co.load_faces, cn.load_faces
    );
    println!(
        "total load on faces (m) : OFF {:>12.1}   ON {:>12.1}",
        co.load_total, cn.load_total
    );

    let (mo, lo_o, hi_o) = relief(&off.field);
    let (mn, lo_n, hi_n) = relief(&on.field);
    println!("\n--- WHAT MOVED IN THE WORLD ---");
    println!("mean surface  : OFF {mo:>9.2} m   ON {mn:>9.2} m   {:+.2} m", mn - mo);
    println!("min surface   : OFF {lo_o:>9.2} m   ON {lo_n:>9.2} m");
    println!("max surface   : OFF {hi_o:>9.2} m   ON {hi_n:>9.2} m");
    let moved = off
        .field
        .surf
        .iter()
        .zip(&on.field.surf)
        .filter(|(a, b)| (**a - **b).abs() > 1.0)
        .count();
    println!(
        "cells whose elevation moved by >1 m : {moved} ({:.2} %)",
        moved as f64 * 100.0 / off.field.surf.len() as f64
    );
    let mad: f64 = off
        .field
        .surf
        .iter()
        .zip(&on.field.surf)
        .map(|(a, b)| (a - b).abs())
        .sum::<f64>()
        / off.field.surf.len() as f64;
    println!("mean |Δ elevation|                  : {mad:.3} m");
    // The elevation field is the *least* sensitive readout here, so the drainage
    // network is reported beside it: incision magnitudes on this world are small
    // against 600 m of uplift, but WHERE the water goes is the thing that moved.
    let land = |f: &DeepField, i: usize| f.surf[i] > 0.0;
    let mut land_cells = 0usize;
    let mut recv_changed = 0usize;
    let mut area_rel = 0.0f64;
    let mut land_mad = 0.0f64;
    for i in 0..off.field.surf.len() {
        if !land(&off.field, i) {
            continue;
        }
        land_cells += 1;
        land_mad += (off.field.surf[i] - on.field.surf[i]).abs();
        if off.field.recv[i] != on.field.recv[i] {
            recv_changed += 1;
        }
        let (a, b) = (off.field.area[i], on.field.area[i]);
        area_rel += (a - b).abs() / a.max(b).max(1.0);
    }
    let lc = land_cells.max(1) as f64;
    println!("\nsubaerial cells                     : {land_cells}");
    println!("  mean |Δ elevation| on land        : {:.3} m", land_mad / lc);
    println!(
        "  cells whose ARGMAX receiver moved : {recv_changed} ({:.2} % of land)",
        recv_changed as f64 * 100.0 / lc
    );
    println!(
        "  mean relative |Δ drainage area|   : {:.4}",
        area_rel / lc
    );
    let peak_off = off.field.area.iter().cloned().fold(0.0f64, f64::max);
    let peak_on = on.field.area.iter().cloned().fold(0.0f64, f64::max);
    println!("  peak drainage area  OFF {peak_off:.0}  ON {peak_on:.0}");

    // ---- the exponent sweep -------------------------------------------------
    // `p` is the one knob, and its effect should be reported as a curve rather
    // than asserted from a citation. p -> infinity is the OFF row by construction.
    println!("\n--- CONVERGENCE EXPONENT SWEEP (production world) ---");
    println!("     p    simul (cell,epoch)   simul (cell,chapter)   entries   record MiB   deep s");
    for p in [1.0, 2.0, 4.0, 8.0] {
        let m = measure(&pregen.grid, true, p);
        let c = counts(&m.field);
        println!(
            "{p:>6.1} {:>20} {:>22} {:>10} {:>11.2} {:>8.1}",
            c.simul_cell_epochs,
            c.simul_cell_chapters,
            c.entries,
            mib(c.record_bytes),
            m.deep_secs
        );
    }
    println!(
        "{:>6} {:>20} {:>22} {:>10} {:>11.2} {:>8.1}   <- single-receiver D8 (p -> inf)",
        "off", co.simul_cell_epochs, 0, co.entries, mib(co.record_bytes), off.deep_secs
    );

    // ---- the biggest simultaneous junction, so the number is not abstract ---
    // Re-found from the record: the (cell, chapter) with the most out-faces and
    // the largest total magnitude. It is a *temporal-or-simultaneous* junction —
    // the record cannot tell them apart, which is exactly why the counters exist —
    // so it is labelled as an illustration and not as evidence.
    let rec = &on.field.flux;
    let mut best: Option<(usize, u8, f32)> = None;
    for i in 0..cells {
        for k in 0..rec.chapters {
            if rec.out_face_count(i, k) < 3 {
                continue;
            }
            let m: f32 = rec.out_faces(i, k).map(|e| e.magnitude).sum();
            if best.is_none_or(|(_, _, bm)| m > bm) {
                best = Some((i, k, m));
            }
        }
    }
    if let Some((i, k, _)) = best {
        let (gx, gy) = (i % rec.w, i / rec.w);
        println!(
            "\n--- illustration: the largest multi-face junction — cell {i} ({gx}, {gy}) \
             ~ ({:.1} km, {:.1} km) from the grid corner, chapter {k} ---",
            gx as f64 * on.field.cell_m / 1000.0,
            gy as f64 * on.field.cell_m / 1000.0
        );
        for e in rec.out_faces(i, k) {
            println!(
                "  out  {:>4}  magnitude {:>14.1}  load {:>12.6}",
                e.face.label(),
                e.magnitude,
                e.load
            );
        }
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod gate {
    use super::*;

    /// **The acceptance claim, at the smallest extent that can carry it.**
    ///
    /// *Why the invariant is scale-free.* Both halves are **per-cell predicates
    /// counted over whatever cells exist**: "did this cell's discharge leave
    /// through two faces in this epoch". The zero half is a statement about the
    /// *arity of the routing function* — a D8 receiver is one out-edge, at any grid
    /// size, so the count is zero on a 3×3 and on a 545×545 alike. The positive
    /// half needs only that *some* cell somewhere has two comparable downslope
    /// neighbours, which is a property of having topography at all. The
    /// **magnitudes** are what production scale is for, and they live in `main`.
    #[test]
    fn simultaneous_divergence_is_structurally_zero_off_and_present_on() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let off = counts(&build_field_cfg(&pregen.grid, &cfg(&pregen.grid, false, 0.0)));
        let on = counts(&build_field_cfg(&pregen.grid, &cfg(&pregen.grid, true, 4.0)));
        assert_eq!(
            off.simul_cell_epochs, 0,
            "single-receiver routing cannot diverge within an epoch"
        );
        assert_eq!(off.simul_max_faces, 0);
        assert!(
            on.simul_cell_epochs > 0,
            "MFD produced no simultaneous divergence at all — a null, not a pass"
        );
        assert!(on.simul_max_faces >= 2);
    }

    // (The simultaneous ⊆ temporal set-inclusion is asserted in
    // `tests/mfd_routing.rs::temporal_divergence_survives_alongside_the_simultaneous_kind`
    // and is deliberately not duplicated here — each Small deep run is ~4 s of
    // gate, and a second copy of an assertion buys nothing but wall-clock.)

    /// **The exponent is a convergence knob, and the probe reports a monotone
    /// quantity.** A larger `p` concentrates flow, so it must produce *no more*
    /// simultaneous divergence than a smaller one on the same world. This is the
    /// invariant that would catch a partition whose exponent had been wired
    /// backwards — an error no absolute count could see.
    ///
    /// Scale-free: monotonicity in `p` follows from `Sₖ^p` being order-preserving
    /// and ratio-amplifying, which is per-cell arithmetic.
    #[test]
    fn a_larger_exponent_produces_no_more_simultaneous_divergence() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let disperse = counts(&build_field_cfg(&pregen.grid, &cfg(&pregen.grid, true, 1.0)));
        let converge = counts(&build_field_cfg(&pregen.grid, &cfg(&pregen.grid, true, 8.0)));
        assert!(
            converge.simul_cell_epochs <= disperse.simul_cell_epochs,
            "p=8 diverged MORE than p=1 ({} vs {}) — the exponent is inverted",
            converge.simul_cell_epochs,
            disperse.simul_cell_epochs
        );
    }
}
