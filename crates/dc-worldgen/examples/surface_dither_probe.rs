//! **The surface-member probe** (journal/0058).
//!
//! Two questions, one world.
//!
//! **(A) Which coordinates did journal/0053 and /0055 actually name?** Those
//! entries label their stations with a bare pair of numbers — `(82 346,
//! 24 391)` for the loess margin. `soil_depth_probe::STATIONS` holds those same
//! numbers as **world metres**, and divides by 0.9 to reach a voxel. Read the
//! label as a *voxel* address instead and you land ~8.6 km away — nineteen deep
//! cells — on a different column with a different record. This probe prints both
//! readings side by side so the ambiguity can never be re-litigated from memory,
//! and checks that `idx_to_voxel` (which names the *scanned* sites) round-trips.
//!
//! **(B) Is the surface member quantized per chunk?** The buried fill re-picks
//! a class's member per voxel column (the 3c-2 boundary dither, journal/0011).
//! Before journal/0058 the surface voxel drew **one member per chunk column**,
//! so a 32×32 footprint — 28.8 m of ground — was a single flat patch of one
//! member's albedo. This probe counts distinct surface members per chunk
//! footprint: `1` everywhere is the defect, `>1` is the dither working.
//!
//! Nothing here is on a generation path.
//!
//! `cargo run --release -p dc-worldgen --example surface_dither_probe`

use std::time::Instant;

use dc_core::materials::geology::vanilla;
use dc_core::{Block, ChunkPos};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{ColumnFill, Plan, WorldGenerator};

/// The client's `BENCH_SEED` (`dc-client/src/bench.rs`), widened exactly the way
/// `Authority::new_worldgen` widens it (`seed as u64`).
const SEED: u64 = 1337;
/// The client's `WORLDGEN_EXTENT` (`dc-client/src/authority.rs`).
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;

fn main() {
    println!("=== surface-member probe (journal/0058) ===");
    println!(
        "seed {SEED}, extent {}, N=2 ({VOXEL_M} m voxels)",
        EXTENT.label()
    );
    println!(
        "this is `Pregen::run(WorldParams{{seed,extent}})`, and `Pregen::run` IS\n\
         `Pregen::run_with(params, &DeepOverrides::default())` — the exact call the\n\
         client's `Authority::new_worldgen` makes at boot.\n"
    );
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let set = vanilla();
    let mut g = WorldGenerator::with_geology(&pregen, set.clone());

    // ---- (A) which coordinates ------------------------------------------
    let m2v = |m: f64| (m / VOXEL_M).round() as i64;
    // **The signature of a units error, and why it looked site-specific.**
    // Reading a metres label as voxels displaces you by `(1/0.9 − 1) = 11.1 %`
    // of the coordinate's own magnitude. At the loess margin (82 346 m) that is
    // 8.6 km — nineteen deep cells, a different record entirely. At the
    // periglacial summit (−4 586 m) it is 600 m — barely one deep cell, and the
    // regolith reads nearly the same. A world that genuinely differed would
    // disagree everywhere at once. A units error disagrees **in proportion to
    // how far from the origin you are standing**, which is exactly what the two
    // live samples show.
    let sites: [(&str, i64, i64); 6] = [
        (
            "loess margin, label read as METRES (82346 m, 24391 m) — soil_depth_probe's reading",
            m2v(82346.0),
            m2v(24391.0),
        ),
        (
            "loess margin, label read as VOXELS (82346, 24391) — the live-client sample",
            82346,
            24391,
        ),
        ("live-client second sample, 180 m east", 82546, 24391),
        (
            "dune field, label read as METRES (107183 m, 9672 m)",
            m2v(107183.0),
            m2v(9672.0),
        ),
        (
            "periglacial summit, label read as METRES (-4586 m, -3206 m)",
            m2v(-4586.0),
            m2v(-3206.0),
        ),
        (
            "periglacial summit, label read as VOXELS (-4586, -3207) — the live-client sample",
            -4586,
            -3207,
        ),
    ];
    for (label, vx, vz) in sites {
        site(&mut g, &pregen, label, vx, vz);
    }

    // ---- (A3) the BLOCK column, the way a live scan reads it -------------
    //
    // `world_scan_region` reads blocks, so a field report counts *blocks*. This
    // reads the generator's own blocks at the same addresses, with no player
    // edits, so the two are directly comparable — and prints the metres beside
    // them so the block count can be checked against `round(H / 0.9)`.
    println!("=== BLOCK COLUMNS, generator-truth, no player edits ===");
    println!(
        "  `expected` = round(H / 0.9) at THAT voxel. `sediment` = contiguous\n\
         \x20 non-igneous blocks below the surface — what a scan counts as the pile.\n"
    );
    for (label, vx, vz) in [
        ("loess margin AS METRES", m2v(82346.0), m2v(24391.0)),
        ("loess margin AS VOXELS (scanned)", 82346, 24391),
        ("periglacial summit AS METRES", m2v(-4586.0), m2v(-3206.0)),
        ("periglacial summit AS VOXELS (scanned)", -4586, -3207),
        ("wave coast AS METRES", m2v(95224.0), m2v(22091.0)),
        ("wave coast AS VOXELS (scanned)", 95224, 22091),
        ("barest land AS METRES", m2v(101663.0), m2v(5073.0)),
        ("barest land AS VOXELS (scanned)", 101663, 5073),
    ] {
        block_column(&mut g, &pregen, label, vx, vz);
    }

    // ---- (A2) does `idx_to_voxel` round-trip? ----------------------------
    //
    // `soil_depth_probe::idx_to_voxel` is the inverse of
    // `DeepField::deep_coords`, and it is what names the coordinates of sites 6
    // and 7 (the world's thickest/thinnest regolith, found by scanning the
    // plane). If it did not round-trip, every coordinate printed beside a scanned
    // site would point somewhere other than the cell whose numbers are printed
    // with it. Ground truth by construction: `regolith_at_voxel` at a cell's own
    // voxel address must read back that cell's own `H`, exactly.
    {
        let f = &pregen.deep;
        let (mut checked, mut bad, mut worst) = (0usize, 0usize, 0.0f64);
        for idx in (0..f.w * f.w).step_by(7) {
            let (vx, vz) = idx_to_voxel(f.w, f.wp, idx);
            let Some(h) = f.regolith_at_voxel(vx, vz) else {
                continue;
            };
            let d = (h - f.regolith[idx]).abs();
            worst = worst.max(d);
            if d > 0.0 {
                bad += 1;
            }
            checked += 1;
        }
        println!(
            "--- idx_to_voxel round-trip: {checked} deep cells sampled, {bad} disagree, \
             worst |ΔH| = {worst:.6} m ---"
        );
        println!(
            "  (0 disagreements = a scanned site's printed voxel address really is that \
             cell's own address)\n"
        );
    }

    // ---- (B) per-chunk diversity of the surface BLOCK --------------------
    // journal/0074: the near surface is the record's top span now, not the
    // deep-record member draw (which moved to the far summary). This measures
    // near-surface block diversity per chunk footprint.
    println!("--- distinct SURFACE BLOCKS per 32x32 chunk footprint ---");
    let mut hist = [0usize; 9];
    let mut chunks = 0usize;
    let (bcx, bcz) = (m2v(82346.0).div_euclid(32), m2v(24391.0).div_euclid(32));
    for dz in -6..=6i64 {
        for dx in -6..=6i64 {
            let col = g.column_record(bcx + dx, bcz + dz);
            let mut seen: Vec<Block> = Vec::new();
            for &b in &col.surface {
                if !seen.contains(&b) {
                    seen.push(b);
                }
            }
            hist[seen.len().min(8)] += 1;
            chunks += 1;
        }
    }
    println!(
        "  over {chunks} chunk columns around the loess margin: {hist:?}  (index = distinct blocks)"
    );
    let one = hist[1] + hist[0];
    println!(
        "  chunks expressing AT MOST ONE surface block: {one} ({:.1}%)\n",
        100.0 * one as f64 / chunks as f64
    );

    // ---- (C) far-field cost ----------------------------------------------
    // Same stride and count as `mixture_cost_probe`, so the µs/sample number is
    // directly comparable to journal/0055's 22.1 → 22.9.
    let stride = 907i64;
    let n = 220i64;
    let mut land = 0usize;
    let t = Instant::now();
    for i in -n / 2..n / 2 {
        for j in -n / 2..n / 2 {
            let (h, _b) = g.coarse_surface(i * stride, j * stride);
            if h >= 0 {
                land += 1;
            }
        }
    }
    println!(
        "--- far-field cost: {} coarse_surface samples ({land} land) at {stride}-voxel stride: \
         {:.1} µs/sample ---",
        n * n,
        1e6 * t.elapsed().as_secs_f64() / (n * n) as f64
    );
}

/// Verbatim copy of `soil_depth_probe::idx_to_voxel` — the inverse of
/// `DeepField::deep_coords`, with journal/0043's integer `wp / 2` centring. Kept
/// as a copy on purpose: the point of the check above is to test *that* mapping
/// as written, not a refactor of it.
fn idx_to_voxel(w: usize, wp: usize, idx: usize) -> (i64, i64) {
    let (wf, wpf) = (w as f64, wp as f64);
    let (gx, gy) = ((idx % w) as f64, (idx / w) as f64);
    let px = (gx + 0.5) / wf * wpf - 0.5;
    let py = (gy + 0.5) / wf * wpf - 0.5;
    let half = (wp / 2) as f64;
    (
        ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
    )
}

/// The generator's own block column at one voxel address, run-length encoded
/// from the surface downward — the same thing `world_scan_region` reports from
/// the live client, with no player edits in it.
fn block_column(g: &mut WorldGenerator<'_>, pregen: &Pregen, label: &str, vx: i64, vz: i64) {
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    let top = g.column_record(cx, cz).heights[lz * 32 + lx];
    let h_m = pregen.deep.regolith_at_voxel(vx, vz);
    // 160 voxels of section: the loess margin's 89 needs well over 64, and a
    // truncated window reads as a shortfall, which is exactly the error this
    // readout exists to rule out.
    const WINDOW: i32 = 160;
    let mut col: Vec<Block> = Vec::new();
    let mut cy_cache: Option<(i32, dc_core::Chunk)> = None;
    for y in (top - (WINDOW - 1)..=top).rev() {
        let cy = y.div_euclid(32);
        if cy_cache.as_ref().is_none_or(|(c, _)| *c != cy) {
            let ch = g.generate_chunk(ChunkPos::new(cx as i32, cy, cz as i32));
            cy_cache = Some((cy, ch));
        }
        let ch = &cy_cache.as_ref().expect("just filled").1;
        col.push(ch.get(lx, y.rem_euclid(32) as usize, lz));
    }
    // Run-length encode from the top down, and count the sediment cap.
    let mut runs: Vec<(Block, usize)> = Vec::new();
    for b in &col {
        match runs.last_mut() {
            Some((rb, n)) if rb == b => *n += 1,
            _ => runs.push((*b, 1)),
        }
    }
    let igneous = |b: Block| matches!(b, Block::Stone | Block::Granite | Block::Basalt);
    let sediment = col.iter().take_while(|b| !igneous(**b)).count();
    let expected = h_m.map_or(-1.0, |h| (h / VOXEL_M).round());
    println!("--- {label} — voxel ({vx}, {vz}) ---");
    let truncated = if sediment as i32 >= WINDOW {
        " (WINDOW TRUNCATED — read as a lower bound)"
    } else {
        ""
    };
    println!(
        "  H = {}, expected round(H/0.9) = {}, SEDIMENT BLOCKS = {sediment}{truncated}",
        h_m.map_or("NONE".to_string(), |h| format!("{h:.3} m")),
        if expected < 0.0 {
            "n/a".to_string()
        } else {
            format!("{expected:.0}")
        }
    );
    let s: Vec<String> = runs
        .iter()
        .take(8)
        .map(|(b, n)| format!("{b:?}x{n}"))
        .collect();
    println!("  from y{top} down: {}\n", s.join(" / "));
}

fn site(g: &mut WorldGenerator<'_>, pregen: &Pregen, label: &str, vx: i64, vz: i64) {
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    let i = lz * 32 + lx;
    let h_m = pregen.deep.regolith_at_voxel(vx, vz);
    let rec = pregen.deep.record_at_voxel(vx, vz);
    let (units, rec_m) = rec.map_or((0usize, 0.0f64), |s| {
        (s.units.len(), s.units.iter().map(|u| u.thickness_m).sum())
    });
    let col = g.column_record(cx, cz);
    let fill = ColumnFill::build(&col.strata, VOXEL_M);
    let mixed = (1..=fill.depth_count() as u32)
        .filter(|d| matches!(fill.plan(*d), Some(Plan::Mixed(_))))
        .count();
    println!("--- SITE {label} ---");
    println!(
        "  voxel ({vx}, {vz}) = world ({:.0} m, {:.0} m) | chunk ({cx}, {cz}) | surface y {}",
        vx as f64 * VOXEL_M,
        vz as f64 * VOXEL_M,
        col.heights[i]
    );
    match h_m {
        Some(h) => println!(
            "  recorded H            : {h:.3} m ({:.2} voxels)",
            h / VOXEL_M
        ),
        None => println!("  recorded H            : NONE (border wilds)"),
    }
    println!("  deep record           : {units} units, {rec_m:.2} m total");
    println!(
        "  EXPRESSED             : {} events, {} voxel spans, {mixed} of them mixed",
        col.strata.events.len(),
        fill.depth_count()
    );
    let top = fill.plan(1);
    println!(
        "  SURFACE               : {:?}, {} of 8 eighths{}\n",
        col.surface[i],
        if top.is_some() { col.surface_eighths[i] } else { 0 },
        match top {
            None => " (fallback — no record)".to_string(),
            Some(Plan::Single(k)) => format!(" of member {}", col.strata.events[*k].member.0),
            Some(Plan::Mixed(_)) => " (mixed top span)".to_string(),
        }
    );
}
