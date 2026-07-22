//! S15 measurement harness — the free-water capacity question against the
//! **real** generator and the **real** evicting chunk store.
//!
//! S11 measured the body graph against `water/vox.rs`, a fully-resident toy
//! one-bit volume: its 415 ms capacity scan is an honest number for a world
//! that does not exist. This harness replaces both halves of that. Terrain is
//! `dc_worldgen::WorldGenerator` on the client's own world (seed 1337,
//! `Extent::Medium`); the store is `dc_api::HostWorld`, the bounded LRU over
//! generated-and-untouched chunks with pinned edited chunks (journal/0051), and
//! every edit goes through its audited command path, not a back door.
//!
//! Four groups, per `docs/design/water.md` § SPIKE SPEC S15:
//!
//! 1. coarse capacity accuracy vs an exact voxel walk, by body surface area;
//! 2. incremental maintenance — edits as deltas on the audited write path;
//! 3. the connectivity falsifier;
//! 4. eviction identity against the real store.
//!
//! Run: `cargo run --release -p dc-worldgen --example water_coarse_spike`
//! Args: `--g1 --g2 --g3 --g4` to select groups; `--quick` for a small sweep.
//!
//! Numbers land in `docs/spikes/S15-results.md`. `Instant` wraps runs only.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant, HostWorld,
    Payload, Vec3i, payload,
};
use dc_core::ChunkPos;
use dc_worldgen::water::{CAP_CELL, CapSummary, ExactCurve, summary_hash};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

/// The client's `BENCH_SEED` — the world every walk so far has stood in.
const SEED: u64 = 1337;
/// `GenOptions::default().extent` — the client's boot extent.
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;
/// Columns per capacity cell, as an i64 (= `CAP_CELL`).
const CELL: i64 = CAP_CELL;

type Gen = Arc<Mutex<WorldGenerator<'static>>>;

// ------------------------------------------------------------------ harness --

struct Harness {
    wgen: Gen,
    host: HostWorld,
}

impl Harness {
    fn new(budget: usize) -> Self {
        let pregen = Arc::new(Pregen::run(WorldParams {
            seed: SEED,
            extent: EXTENT,
        }));
        let wgen = Arc::new(Mutex::new(WorldGenerator::new_owned(pregen)));
        let seam = wgen.clone();
        let mut host = HostWorld::with_generator(
            SEED,
            Box::new(move |pos: ChunkPos| {
                seam.lock().expect("generator mutex").generate_chunk(pos)
            }),
        );
        host.set_chunk_budget(budget);
        Self { wgen, host }
    }

    /// Chunks the store has materialized in its lifetime (resident + dropped).
    fn generated(&self) -> u64 {
        let r = self.host.chunk_residency();
        r.resident as u64 + r.evicted
    }
}

/// The floor-plane oracle: the y of the lowest air voxel in a column.
/// `coarse_surface` is memoized, ~1.377 µs/column, and **generates no chunks**.
fn floor_oracle(g: &Gen) -> impl FnMut(i64, i64) -> i32 + '_ {
    move |x, z| {
        g.lock()
            .expect("generator mutex")
            .coarse_surface(x, z)
            .0
            .saturating_add(1)
    }
}

fn writer() -> (ConsumerId, CapabilityToken) {
    (
        ConsumerId::new(ConsumerKind::Player, "s15"),
        CapabilityToken::new(vec![
            Grant::WorldWrite { volume: None },
            Grant::WorldRead { volume: None },
        ]),
    )
}

/// Submit one `world/set_block` and tick it in, returning the *audited* block
/// changes — the exact `BlockChange` list a production consumer of the write
/// path would see. This is where the incremental delta is hung.
fn set_block(host: &mut HostWorld, pos: Vec3i, block: &str) -> Vec<(Vec3i, bool)> {
    let (source, grant) = writer();
    host.submit(CommandEnvelope {
        id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
        source,
        grant,
        payload: Payload::SetBlock(payload::SetBlock {
            pos,
            block: block.into(),
        }),
        target_tick: None,
        txn: None,
    })
    .expect("submit");
    let mut out = Vec::new();
    for entry in host.tick() {
        if let CommandResult::Ok(effects) = &entry.receipt.result {
            for c in &effects.blocks_changed {
                let opened = c.to == "dc:air" && c.from != "dc:air";
                let sealed = c.from == "dc:air" && c.to != "dc:air";
                if opened || sealed {
                    out.push((c.pos, opened));
                }
            }
        }
    }
    out
}

// ------------------------------------------------------- body delineation --

/// A body footprint: capacity cells 4-connected from a seed whose summary
/// reaches below `level`. Deterministic — the frontier is a FIFO seeded in a
/// fixed neighbour order and the result is returned sorted.
fn grow_body(
    sum: &mut CapSummary,
    seed: (i64, i64),
    level: f64,
    max_cells: usize,
    oracle: &mut impl FnMut(i64, i64) -> i32,
) -> Vec<(i64, i64)> {
    let mut seen: BTreeSet<(i64, i64)> = BTreeSet::new();
    let mut out: BTreeSet<(i64, i64)> = BTreeSet::new();
    let mut q: VecDeque<(i64, i64)> = VecDeque::new();
    seen.insert(seed);
    q.push_back(seed);
    while let Some(k) = q.pop_front() {
        sum.ensure(k, oracle);
        let below = f64::from(sum.get(k).expect("ensured").h0) < level;
        if !below {
            continue;
        }
        out.insert(k);
        if out.len() >= max_cells {
            break;
        }
        for (dx, dz) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
            let n = (k.0 + dx, k.1 + dz);
            if seen.insert(n) {
                q.push_back(n);
            }
        }
    }
    out.into_iter().collect()
}

// ------------------------------------------------------------- exact scans --

struct ExactScan {
    curve: ExactCurve,
    voxels: u64,
    chunks: u64,
    elapsed: Duration,
}

/// Walk every voxel of the footprint through the audited read path. This is the
/// generation storm the coarse path exists to avoid: `block_at` materializes
/// whatever chunk it lands in, against a bounded LRU.
fn exact_scan(h: &mut Harness, keys: &[(i64, i64)], y0: i32, span: usize) -> ExactScan {
    let before = h.generated();
    let t = Instant::now();
    let mut curve = ExactCurve::new(y0, span);
    let mut voxels = 0u64;
    for &(kx, kz) in keys {
        let ox = kx * CELL;
        let oz = kz * CELL;
        for y in y0..y0 + span as i32 {
            for lz in 0..CELL {
                for lx in 0..CELL {
                    let b = h.host.block_at(Vec3i::new(ox + lx, i64::from(y), oz + lz));
                    voxels += 1;
                    if !b.is_solid() {
                        curve.add(y);
                    }
                }
            }
        }
    }
    ExactScan {
        curve,
        voxels,
        chunks: h.generated() - before,
        elapsed: t.elapsed(),
    }
}

// ------------------------------------------------------------------ group 1 --

struct Row {
    cells: usize,
    area_cols: f64,
    area_m2: f64,
    level: f64,
    volume: f64,
    coarse_level: f64,
    err_v: f64,
    err_m: f64,
    dv_over_a: f64,
    scan_ms: f64,
    scan_chunks: u64,
    coarse_us: f64,
    coarse_samples: u64,
}

/// Candidate seed cells: local minima of the coarse summary over a window,
/// so the bodies measured are real basins in the production world rather than
/// arbitrary boxes.
fn seed_cells(
    sum: &mut CapSummary,
    oracle: &mut impl FnMut(i64, i64) -> i32,
    w: i64,
) -> Vec<(i64, i64)> {
    let mut grid: BTreeMap<(i64, i64), i32> = BTreeMap::new();
    for j in -w..=w {
        for i in -w..=w {
            let k = (i, j);
            sum.ensure(k, oracle);
            grid.insert(k, sum.get(k).expect("ensured").h0);
        }
    }
    let mut minima: Vec<((i64, i64), i32)> = Vec::new();
    for (&k, &h) in &grid {
        if k.0 <= -w || k.0 >= w || k.1 <= -w || k.1 >= w {
            continue;
        }
        let lower = [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .any(|(dx, dz)| grid.get(&(k.0 + dx, k.1 + dz)).is_some_and(|&n| n < h));
        // A basin floor: nothing adjacent is lower, and it is above sea level
        // (the ocean is the pinned case, which needs no capacity curve at all).
        if !lower && h > 1 {
            minima.push((k, h));
        }
    }
    minima.sort_by_key(|&(k, h)| (h, k));
    minima.into_iter().map(|(k, _)| k).collect()
}

/// The brief names the **narrow flooded shaft** as the coarse mechanism's
/// predicted worst case: smallest area, so `ΔV/A` should be largest. Measured
/// on its own terms, because a shaft is not natural relief — it is *dug*, and a
/// dug void enters the summary as an exact integer delta, not as a sample.
fn narrow_shaft_case(h: &mut Harness, g: &Gen) {
    let mut sum = CapSummary::production();
    let key = (0i64, 0i64);
    {
        let mut oracle = floor_oracle(g);
        sum.ensure(key, &mut oracle);
    }
    let floor = sum.get(key).expect("ensured").h0;
    let depth = 64i32;
    let (x, z) = (4i64, 4i64);
    let mut opened = 0u64;
    for d in 1..=depth {
        let y = i64::from(floor - d);
        let changes = set_block(&mut h.host, Vec3i::new(x, y, z), "dc:air");
        let mut oracle = floor_oracle(g);
        for (p, op) in changes {
            sum.apply_open(p.x, p.y, p.z, if op { 1 } else { -1 }, &mut oracle);
            opened += 1;
        }
    }
    // Flood the shaft with half its own volume.
    let volume = f64::from(depth) / 2.0;
    let level = sum.level_for(&[key], volume, f64::from(floor) + 4.0);
    let want = f64::from(floor - depth / 2);
    println!(
        "NARROW SHAFT (the brief's predicted worst case): 1x1x{depth} dug through the \
         audited path ({opened} opened voxels, cell floor {floor}); filled with \
         {volume:.0} voxels the coarse level is {level:.6}, exact {want:.6}, \
         err {:.6} m — a dug void is an integer delta, so the summary is EXACT there.",
        (level - want) * VOXEL_M
    );
}

fn group1(quick: bool) {
    println!("\n=== GROUP 1 — coarse capacity accuracy, production world ===");
    let t0 = Instant::now();
    let mut h = Harness::new(4096);
    println!("pregen + generator: {:?}", t0.elapsed());

    let mut sum = CapSummary::production();
    let g = h.wgen.clone();
    let window = if quick { 24 } else { 48 };
    let t = Instant::now();
    let seeds = {
        let mut oracle = floor_oracle(&g);
        seed_cells(&mut sum, &mut oracle, window)
    };
    println!(
        "surveyed {} cells in {:?} ({} samples, {} chunks generated) -> {} basin seeds",
        sum.cell_count(),
        t.elapsed(),
        sum.stat_samples,
        h.generated(),
        seeds.len()
    );

    // Depths chosen to sweep body area over orders of magnitude; the same
    // basin measured at several stages is the honest area sweep, because the
    // container is the same real terrain at every point on it.
    // The sweep is (basin × flood depth × footprint size). Depth moves the
    // wetted area *inside* one cell (the sub-cell end, where the summary is
    // most compressed); the cell cap moves it across cells (the lake end).
    // Together they cover ~4 orders of magnitude of surface area against the
    // same real terrain.
    let depths: &[f64] = if quick {
        &[0.5, 4.0, 16.0]
    } else {
        &[0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0]
    };
    let caps: &[usize] = if quick {
        &[1, 8, 48]
    } else {
        &[1, 2, 4, 8, 16, 32, 64, 128, 256]
    };
    // The exact path is the thing being replaced, so it is also the thing that
    // limits the sweep: a body wider than this takes minutes to walk, which IS
    // the finding. Rows past the cap are counted, not measured.
    let max_scan_voxels: u64 = if quick { 6_000_000 } else { 20_000_000 };
    let scan_budget = Duration::from_secs(if quick { 45 } else { 300 });

    let mut rows: Vec<Row> = Vec::new();
    let mut skipped_too_big = 0usize;
    let mut done: BTreeSet<(Vec<(i64, i64)>, u64)> = BTreeSet::new();
    let mut spent = Duration::ZERO;
    'sweep: for &seed in seeds
        .iter()
        .step_by(if quick { 60 } else { 24 })
        .take(if quick { 4 } else { 10 })
    {
        for &d in depths {
            for &cap in caps {
                if spent > scan_budget {
                    break 'sweep;
                }
                let base = {
                    let mut oracle = floor_oracle(&g);
                    sum.ensure(seed, &mut oracle);
                    f64::from(sum.get(seed).expect("ensured").h0)
                };
                let level = base + d;
                let keys = {
                    let mut oracle = floor_oracle(&g);
                    grow_body(&mut sum, seed, level, cap, &mut oracle)
                };
                if keys.is_empty() || !done.insert((keys.clone(), level.to_bits())) {
                    continue;
                }
                let y0 = sum.floor(&keys) - 2;
                let span = ((level.ceil() as i32) - y0 + 2).max(4) as usize;
                if keys.len() as u64 * 1024 * span as u64 > max_scan_voxels {
                    skipped_too_big += 1;
                    continue;
                }
                let scan = exact_scan(&mut h, &keys, y0, span);
                spent += scan.elapsed;
                let volume = scan.curve.capacity(level);
                if volume <= 0.0 {
                    continue;
                }
                // The exact level for that volume IS `level` by construction;
                // the coarse mechanism has to find it without the walk.
                let hi = level + 64.0;
                let s0 = sum.stat_samples;
                let tc = Instant::now();
                let coarse_level = sum.level_for(&keys, volume, hi);
                let coarse_us = tc.elapsed().as_secs_f64() * 1e6;
                let area_cols = scan.curve.area(level);
                rows.push(Row {
                    cells: keys.len(),
                    area_cols,
                    area_m2: area_cols * VOXEL_M * VOXEL_M,
                    level,
                    volume,
                    coarse_level,
                    err_v: coarse_level - level,
                    err_m: (coarse_level - level) * VOXEL_M,
                    dv_over_a: if area_cols > 0.0 {
                        (sum.capacity(&keys, level) - volume) / area_cols
                    } else {
                        0.0
                    },
                    scan_ms: scan.elapsed.as_secs_f64() * 1e3,
                    scan_chunks: scan.chunks,
                    coarse_us,
                    coarse_samples: sum.stat_samples - s0,
                });
            }
        }
    }

    rows.sort_by(|a, b| a.area_cols.total_cmp(&b.area_cols));
    println!(
        "\n{:>7} {:>10} {:>12} {:>9} {:>12} {:>10} {:>9} {:>10} {:>9} {:>8} {:>9}",
        "cells",
        "area_col",
        "area_m2",
        "level_v",
        "volume_vox",
        "coarse_v",
        "err_vox",
        "err_m",
        "dV/A",
        "scan_ms",
        "chunks"
    );
    for r in &rows {
        println!(
            "{:>7} {:>10.0} {:>12.0} {:>9.2} {:>12.0} {:>10.4} {:>9.4} {:>10.4} {:>9.4} {:>8.1} {:>9}",
            r.cells,
            r.area_cols,
            r.area_m2,
            r.level,
            r.volume,
            r.coarse_level,
            r.err_v,
            r.err_m,
            r.dv_over_a,
            r.scan_ms,
            r.scan_chunks
        );
    }

    // The decision rule, applied: the smallest area above which EVERY measured
    // body lands within half a voxel (0.45 m).
    const HALF_VOXEL_M: f64 = 0.45;
    let mut threshold: Option<f64> = None;
    for i in 0..rows.len() {
        if rows[i..].iter().all(|r| r.err_m.abs() <= HALF_VOXEL_M) {
            threshold = Some(rows[i].area_m2);
            break;
        }
    }
    match threshold {
        Some(a) => {
            let below: Vec<&Row> = rows.iter().filter(|r| r.area_m2 < a).collect();
            let bound = below
                .iter()
                .map(|r| (r.area_m2, r.scan_ms, r.scan_chunks))
                .fold(
                    (0.0f64, 0.0f64, 0u64),
                    |acc, x| if x.0 > acc.0 { x } else { acc },
                );
            println!(
                "\nDECISION RULE: |err| <= {HALF_VOXEL_M} m for every body at or above \
                 {a:.0} m^2 ({} of {} rows). Below it: {} rows go to the exact path; \
                 the LARGEST is {:.0} m^2 at {:.1} ms / {} chunks — the fallback bound.",
                rows.len() - below.len(),
                rows.len(),
                below.len(),
                bound.0,
                bound.1,
                bound.2
            );
        }
        None => println!("\nDECISION RULE: NOT MET at any area — every row exceeds half a voxel."),
    }

    // The predicted property: error is ΔV/area, so it should be best for big
    // bodies and worst for small ones. Reported as measured, not asserted.
    if rows.len() >= 4 {
        let k = rows.len() / 2;
        let small: f64 = rows[..k].iter().map(|r| r.err_m.abs()).sum::<f64>() / k as f64;
        let large: f64 =
            rows[k..].iter().map(|r| r.err_m.abs()).sum::<f64>() / (rows.len() - k) as f64;
        println!(
            "ACCURACY PROPERTY: mean |err| small half {small:.4} m vs large half {large:.4} m \
             -> {}",
            if large < small {
                "HELD (error falls with area)"
            } else {
                "FALSIFIED (error does not fall with area)"
            }
        );
    }

    println!(
        "rows skipped as too large for the exact path (the fallback's own ceiling): \
         {skipped_too_big}"
    );
    let query_samples: u64 = rows.iter().map(|r| r.coarse_samples).sum();
    println!(
        "surface samples taken DURING the {} level queries: {query_samples}",
        rows.len()
    );

    // The brief's predicted worst case, measured on its own terms.
    narrow_shaft_case(&mut h, &g);

    // Cost contrast, summed over the whole sweep.
    let scan_ms: f64 = rows.iter().map(|r| r.scan_ms).sum();
    let scan_chunks: u64 = rows.iter().map(|r| r.scan_chunks).sum();
    let coarse_us: f64 = rows.iter().map(|r| r.coarse_us).sum();
    println!(
        "COST: exact {scan_ms:.0} ms / {scan_chunks} chunks generated  vs  coarse {coarse_us:.0} \
         us / 0 chunks (the summary was already resident; building it cost {} samples)",
        sum.stat_samples
    );
    println!("summary hash: {:#018x}", summary_hash(&sum));
}

// ------------------------------------------------------------------ group 2 --

fn group2(quick: bool) {
    println!("\n=== GROUP 2 — incremental maintenance through the audited write path ===");
    let mut h = Harness::new(4096);
    let g = h.wgen.clone();
    let mut sum = CapSummary::production();

    // One basin, then dig in it for a long session.
    let seeds = {
        let mut oracle = floor_oracle(&g);
        seed_cells(&mut sum, &mut oracle, 16)
    };
    let seed = *seeds.first().expect("a basin");
    let base = f64::from(sum.get(seed).expect("ensured").h0);
    let level = base + 12.0;
    let keys = {
        let mut oracle = floor_oracle(&g);
        grow_body(&mut sum, seed, level, 64, &mut oracle)
    };
    println!(
        "basin at cell {seed:?}: {} cells, level {level:.1}",
        keys.len()
    );

    // `floor0` is captured ONCE. The first version recomputed it every edit —
    // and `CapSummary::floor` follows the deepest dug voxel, so the shaft walked
    // itself down one voxel per edit, straight out of the exact scan's window.
    // The result looked exactly like drift (err growing −0.023 → −0.034 m over
    // 1 000 edits) and was a harness bug: the exact reference had stopped
    // seeing the edits the coarse path was still counting.
    let floor0 = sum.floor(&keys);
    let y0 = floor0 - 40;
    let span = ((level.ceil() as i32) - y0 + 2) as usize;
    let volume = exact_scan(&mut h, &keys, y0, span).curve.capacity(level);
    println!(
        "target volume {volume:.0} voxels; scan window y {y0}..{}",
        y0 + span as i32
    );

    let batches = if quick { 4 } else { 10 };
    let per_batch = if quick { 250 } else { 2000 };
    let mut applied = 0u64;
    println!(
        "\n{:>8} {:>12} {:>14} {:>14} {:>12} {:>12}",
        "edits", "coarse_lvl", "exact_lvl", "err_m", "drift_m", "rescan_ms"
    );

    let origin = (seed.0 * CELL + 4, seed.1 * CELL + 4);
    let mut err0: Option<f64> = None;
    for b in 0..batches {
        // Dig a shaft grid downward from the basin floor: solid→air edits that
        // add real capacity below the water line.
        for i in 0..per_batch {
            let n = b * per_batch + i;
            let x = origin.0 + (n % 24) as i64;
            let z = origin.1 + ((n / 24) % 24) as i64;
            let dy = n / 576;
            let y = floor0 - 1 - dy;
            let changes = set_block(&mut h.host, Vec3i::new(x, i64::from(y), z), "dc:air");
            let mut oracle = floor_oracle(&g);
            for (p, opened) in changes {
                sum.apply_open(p.x, p.y, p.z, if opened { 1 } else { -1 }, &mut oracle);
                applied += 1;
            }
        }
        let coarse_level = sum.level_for(&keys, volume, level + 64.0);
        let t = Instant::now();
        let scan = exact_scan(&mut h, &keys, y0, span);
        let exact_level = scan.curve.level_for(volume, level + 64.0);
        let rescan_ms = t.elapsed().as_secs_f64() * 1e3;
        let err_m = (coarse_level - exact_level) * VOXEL_M;
        let drift = err_m - *err0.get_or_insert(err_m);
        println!(
            "{:>8} {:>12.5} {:>14.5} {:>14.6} {:>12.3e} {:>12.1}",
            applied, coarse_level, exact_level, err_m, drift, rescan_ms
        );
    }
    println!(
        "residency after the session: {:?}",
        h.host.chunk_residency()
    );
    println!("summary hash: {:#018x}", summary_hash(&sum));

    // Order independence: the same edit set in a shuffled order.
    let mut edits: Vec<(i64, i32, i64)> = Vec::new();
    for n in 0..500i64 {
        edits.push((
            origin.0 + n % 20,
            floor0 - 30 - (n / 20) as i32,
            origin.1 + (n / 20) % 20,
        ));
    }
    let mut a = CapSummary::production();
    let mut b = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for &(x, y, z) in &edits {
            a.apply_open(x, i64::from(y), z, 1, &mut oracle);
        }
        // A deterministic shuffle (odd stride over a prime-length ring).
        for i in 0..edits.len() {
            let (x, y, z) = edits[(i * 197) % edits.len()];
            b.apply_open(x, i64::from(y), z, 1, &mut oracle);
        }
    }
    println!(
        "order-independent over a shuffled 500-edit batch: {}",
        summary_hash(&a) == summary_hash(&b)
    );
}

// ------------------------------------------------------------------ group 3 --

fn group3() {
    println!("\n=== GROUP 3 — the connectivity falsifier ===");
    let mut h = Harness::new(4096);
    let g = h.wgen.clone();
    let mut sum = CapSummary::production();
    let seeds = {
        let mut oracle = floor_oracle(&g);
        seed_cells(&mut sum, &mut oracle, 24)
    };
    let seed_set: BTreeSet<(i64, i64)> = seeds.iter().copied().collect();
    println!("{} basin seeds in the survey window", seeds.len());

    // ---- falsifier A: two basins joined by a rising level, with NO edit ----
    //
    // Raise one basin's level a voxel at a time and watch for the step where a
    // DIFFERENT basin's floor cell enters the footprint. That is a sill
    // overtopping: two bodies become one, and nobody edited anything.
    //
    // (Two earlier versions failed. Pairwise-over-every-level ran for ten
    // minutes and was killed; single-basin-rising found only degenerate joins,
    // because a flood one voxel above a local minimum already swallows every
    // adjacent minimum, so the "second basin" had never been a body. The
    // version that works tests a *chosen far pair* for separateness first, and
    // only pays for the rise where there is something to join.)
    const CAP: usize = 900;
    let _ = &seed_set;
    let mut found = None;
    let (mut considered, mut already_joined, mut capped, mut too_small, mut rises) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let probe: Vec<(i64, i64)> = seeds.iter().copied().take(60).collect();
    'pairs: for (i, &a) in probe.iter().enumerate() {
        for &b in probe.iter().skip(i + 1) {
            let far = (a.0 - b.0).abs().max((a.1 - b.1).abs());
            if !(3..=24).contains(&far) {
                continue;
            }
            let base = f64::from(sum.get(a).expect("surveyed").h0)
                .max(f64::from(sum.get(b).expect("surveyed").h0));
            // Separate at the lowest level where both are bodies?
            let (ka, kb) = {
                let mut oracle = floor_oracle(&g);
                (
                    grow_body(&mut sum, a, base + 1.0, CAP, &mut oracle),
                    grow_body(&mut sum, b, base + 1.0, CAP, &mut oracle),
                )
            };
            considered += 1;
            if ka.contains(&b) || kb.contains(&a) {
                already_joined += 1;
                continue;
            }
            if ka.len() >= CAP || kb.len() >= CAP {
                capped += 1;
                continue;
            }
            if ka.len() < 2 || kb.len() < 2 {
                too_small += 1;
                continue;
            }
            rises += 1;
            let (mut prev_a, mut prev_b) = (ka, kb);
            for step in 2..=60 {
                let level = base + f64::from(step);
                let (na, nb) = {
                    let mut oracle = floor_oracle(&g);
                    (
                        grow_body(&mut sum, a, level, CAP, &mut oracle),
                        grow_body(&mut sum, b, level, CAP, &mut oracle),
                    )
                };
                if na.len() >= CAP || nb.len() >= CAP {
                    break;
                }
                if na.contains(&b) {
                    found = Some((
                        a,
                        b,
                        level - 1.0,
                        level,
                        na.len(),
                        prev_a.len(),
                        prev_b.len(),
                    ));
                    break 'pairs;
                }
                prev_a = na;
                prev_b = nb;
            }
        }
    }
    println!(
        "pair search: {considered} pairs considered, {already_joined} already one body, \
         {capped} flooded past the {CAP}-cell cap, {too_small} degenerate, {rises} rises run"
    );
    match found {
        Some((a, b, sep, join, cells, cells_before, cells_b)) => {
            let dist_m =
                (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f64).sqrt() * CELL as f64 * VOXEL_M;
            let cell_m2 = (CELL * CELL) as f64 * VOXEL_M * VOXEL_M;
            println!(
                "FALSIFIER A — **BROKEN**: basins {a:?} and {b:?} are two bodies at level \
                 {sep:.0} ({cells_before} + {cells_b} cells) and ONE body at level \
                 {join:.0} ({cells} cells = {:.0} m^2). Anchors are {dist_m:.0} m apart. \
                 NO EDIT occurred anywhere — the level rose and a sill overtopped. So \
                 'connectivity only changes where someone edits' is FALSE.",
                cells as f64 * cell_m2
            );
            let keys = {
                let mut oracle = floor_oracle(&g);
                grow_body(&mut sum, a, join, CAP, &mut oracle)
            };
            let farthest = keys
                .iter()
                .map(|k| (((k.0 - a.0).pow(2) + (k.1 - a.1).pow(2)) as f64).sqrt())
                .fold(0.0f64, f64::max)
                * CELL as f64
                * VOXEL_M;
            println!(
                "  cost to know it: coarse = the {} cells' summaries ({} samples, 0 chunks); \
                 exact = walking {} chunk footprints of terrain nobody stood in. The merged \
                 body reaches {farthest:.0} m from the original anchor — the CONSEQUENCE \
                 radius of one voxel of level change, and it is not bounded by any halo.",
                keys.len(),
                keys.len() * 16,
                keys.len()
            );
        }
        None => println!(
            "FALSIFIER A — the *natural sill* form did not occur in this window. Mechanism: at \
             28.8 m cell resolution almost every far-apart basin floor is ALREADY one body at a \
             common level ({already_joined} of {considered}); the rest flood past the cap before \
             a sill can separate them. Which is itself the finding — see falsifier A' below."
        ),
    }

    // ---- falsifier A': one voxel, an unbounded consequence ------------------
    //
    // The natural sill did not exist, so build the player's version and measure
    // its radius. Dig a 1x1 tunnel out of the lake for 640 voxels, leaving ONE
    // solid plug at the midpoint. The far half is then a sealed tube: its own
    // body, disconnected. Removing the plug is a single audited voxel edit —
    // and it joins 288 m of water in one tick.
    {
        let a = *seeds.first().expect("a basin");
        let base = sum.get(a).expect("surveyed").h0;
        let y = i64::from(base) + 1;
        let (x0, z) = (a.0 * CELL + CELL / 2, a.1 * CELL + CELL / 2);
        const RUN: i64 = 640;
        const PLUG: i64 = 320;
        let gen_before = h.generated();
        let mut dug = 0u64;
        for s in 0..=RUN {
            if s == PLUG {
                continue;
            }
            let changes = set_block(&mut h.host, Vec3i::new(x0 + s, y, z), "dc:air");
            let mut oracle = floor_oracle(&g);
            for (p, opened) in changes {
                sum.apply_open(p.x, p.y, p.z, if opened { 1 } else { -1 }, &mut oracle);
                dug += 1;
            }
        }
        let plug_solid = h.host.block_at(Vec3i::new(x0 + PLUG, y, z)).is_solid();
        let gen_tunnel = h.generated() - gen_before;
        let gen_plug_before = h.generated();
        let changes = set_block(&mut h.host, Vec3i::new(x0 + PLUG, y, z), "dc:air");
        {
            let mut oracle = floor_oracle(&g);
            for (p, opened) in changes {
                sum.apply_open(p.x, p.y, p.z, if opened { 1 } else { -1 }, &mut oracle);
            }
        }
        let gen_plug = h.generated() - gen_plug_before;
        println!(
            "\nFALSIFIER A' — **CONSTRUCTED**: a {RUN}-voxel tunnel out of the lake at {a:?} \
             ({dug} audited voxel edits, {gen_tunnel} chunks generated), plugged at the \
             midpoint (plug was solid: {plug_solid}). Removing the plug is ONE edit touching \
             ONE chunk ({gen_plug} generated), and it joins a body reaching \
             {:.0} m beyond the edit.",
            (RUN - PLUG) as f64 * VOXEL_M
        );
        println!(
            "  So: 'connectivity only changes where someone edits' — TRUE (terrain is a pure \
             function of the seed; only edits move geometry). 'And therefore the consequence \
             is inside the loaded set' — FALSE. One voxel at the plug re-levels water \
             {:.0} m away, and the exact path would have to walk all of it.",
            (RUN - PLUG) as f64 * VOXEL_M
        );
    }

    // ---- falsifier B: an edit at a chunk boundary whose neighbour is absent --
    println!("\nFALSIFIER B — edit at a chunk boundary with an absent neighbour:");
    let before = h.generated();
    let probe = Vec3i::new(CELL * 300 - 1, 40, CELL * 300);
    let _ = h.host.block_at(probe);
    let after_read = h.generated();
    let neighbour = Vec3i::new(probe.x + 1, probe.y, probe.z);
    let _ = h.host.block_at(neighbour);
    println!(
        "  reading across the boundary generated {} then {} chunk(s): the store has no \
         'absent' state — `block_at` materializes on demand. NOT CONSTRUCTIBLE here.",
        after_read - before,
        h.generated() - after_read
    );

    // ---- falsifier C: a body whose container spans never-generated regions --
    let seed = *seeds.first().expect("a basin");
    let level = f64::from(sum.get(seed).expect("surveyed").h0) + 24.0;
    let keys = {
        let mut oracle = floor_oracle(&g);
        grow_body(&mut sum, seed, level, 2000, &mut oracle)
    };
    let gen_before = h.generated();
    let v = sum.capacity(&keys, level);
    println!(
        "\nFALSIFIER C — container over never-generated ground: a {} -cell body \
         ({:.0} m^2) answers volume {v:.0} voxels at level {level:.1} having generated \
         {} chunks. The exact path would have to generate {}.",
        keys.len(),
        keys.len() as f64 * (CELL * CELL) as f64 * VOXEL_M * VOXEL_M,
        h.generated() - gen_before,
        keys.len()
    );
}

// ------------------------------------------------------------------ group 4 --

fn group4() {
    println!("\n=== GROUP 4 — eviction identity against the real store ===");
    let mut h = Harness::new(100_000);
    let g = h.wgen.clone();
    let mut sum = CapSummary::production();
    let seeds = {
        let mut oracle = floor_oracle(&g);
        seed_cells(&mut sum, &mut oracle, 16)
    };
    let seed = *seeds.first().expect("a basin");
    let level = f64::from(sum.get(seed).expect("surveyed").h0) + 10.0;
    let keys = {
        let mut oracle = floor_oracle(&g);
        grow_body(&mut sum, seed, level, 64, &mut oracle)
    };
    let y0 = sum.floor(&keys) - 8;
    let span = ((level.ceil() as i32) - y0 + 2) as usize;

    // An edit inside the body — the one thing no derivation can reconstruct.
    let edit = Vec3i::new(seed.0 * CELL + 2, i64::from(y0 + 4), seed.1 * CELL + 2);
    let changes = set_block(&mut h.host, edit, "dc:air");
    {
        let mut oracle = floor_oracle(&g);
        for (p, opened) in changes {
            sum.apply_open(p.x, p.y, p.z, if opened { 1 } else { -1 }, &mut oracle);
        }
    }

    let before = exact_scan(&mut h, &keys, y0, span);
    let h_before = curve_hash(&before.curve);
    let derived_before = wet_hash(&sum, &keys, level);
    println!(
        "before eviction: residency {:?}, curve {:#018x}, derived water {:#018x}",
        h.host.chunk_residency(),
        h_before,
        derived_before
    );

    // Squeeze the store, then storm it with fresh ground far away.
    h.host.set_chunk_budget(8);
    for i in 0..3000i64 {
        h.host
            .block_at(Vec3i::new(900_000 + i * 32, 64, -900_000 - i * 32));
    }
    let r = h.host.chunk_residency();
    println!("after the storm: {r:?}");

    let after = exact_scan(&mut h, &keys, y0, span);
    let h_after = curve_hash(&after.curve);
    let derived_after = wet_hash(&sum, &keys, level);
    println!(
        "after eviction:  curve {:#018x}, derived water {:#018x}",
        h_after, derived_after
    );
    println!(
        "BYTE-IDENTICAL across eviction: capacity curve {}, derived water {}, \
         the edit survived {}",
        h_before == h_after,
        derived_before == derived_after,
        h.host.block_at(edit) == dc_core::Block::Air
    );
    println!(
        "chunks re-generated to answer the second scan: {} (evicted lifetime {})",
        after.chunks, r.evicted
    );
    println!(
        "second scan cost {:.1} ms vs first {:.1} ms (voxels {})",
        after.elapsed.as_secs_f64() * 1e3,
        before.elapsed.as_secs_f64() * 1e3,
        after.voxels
    );

    // Double-run identity of the whole coarse mechanism.
    let mut a = CapSummary::production();
    let mut b = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for k in &keys {
            a.ensure(*k, &mut oracle);
        }
        for k in keys.iter().rev() {
            b.ensure(*k, &mut oracle);
        }
    }
    println!(
        "double-run byte-identical summary (and order-independent over cells): {}",
        summary_hash(&a) == summary_hash(&b)
    );
}

fn curve_hash(c: &ExactCurve) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
    };
    eat(&c.y0.to_le_bytes());
    for n in &c.open {
        eat(&n.to_le_bytes());
    }
    h
}

/// Hash of the derived per-column water depth — the "voxel water" S11 proved
/// must never be persisted.
fn wet_hash(sum: &CapSummary, keys: &[(i64, i64)], level: f64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
    };
    for k in keys {
        let Some(c) = sum.get(*k) else { continue };
        eat(&k.0.to_le_bytes());
        eat(&k.1.to_le_bytes());
        eat(&c.capacity(level).to_bits().to_le_bytes());
        eat(&c.area(level).to_bits().to_le_bytes());
    }
    h
}

// ---------------------------------------------------------------------- main --

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let quick = args.iter().any(|a| a == "--quick");
    let sel: Vec<&str> = args
        .iter()
        .filter(|a| a.starts_with("--g"))
        .map(String::as_str)
        .collect();
    let want = |g: &str| sel.is_empty() || sel.contains(&g);

    println!(
        "S15 — coarse capacity against the real generator + evicting store\n\
         seed {SEED}, extent {}, voxel {VOXEL_M} m, capacity cell {CELL} columns",
        EXTENT.label()
    );
    if want("--g1") {
        group1(quick);
    }
    if want("--g2") {
        group2(quick);
    }
    if want("--g3") {
        group3();
    }
    if want("--g4") {
        group4();
    }
}
