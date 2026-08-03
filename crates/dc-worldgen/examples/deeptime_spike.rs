//! S9 deep-time spike measurement harness. Prints the A/B/C numbers table
//! (wall time, memory, read-quality), the C decay-length profiles, and example
//! collapsed columns. Recorded numbers live in docs/spikes/S9-results.md.
//!
//! Run: `cargo run --release -p dc-worldgen --example deeptime_spike`
//! Args (optional): `--b-iters N` (bounded B iteration budget), `--full-b`
//! (attempt the full 27 M-cell 48 m run instead of the extrapolation scales).
//!
//! `Instant` is wrapped strictly *around* runs, never inside the sim logic
//! (the crate rule): the deep-time engine consumes no wall clock.

use std::time::Instant;

use dc_worldgen::deeptime::{self, Aridity, DeepConfig, DeepGrid, DepEnv, EnergyBand, RegionSpec};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn base_cfg(cell_m: f64, iterations: u32, record: bool) -> DeepConfig {
    DeepConfig {
        seed: SEED,
        cell_m,
        iterations,
        remarch_interval: 20,
        record,
        ..DeepConfig::default()
    }
}

/// Heap high-water estimate for the priority flood (up to ~one Item per cell).
fn heap_estimate_bytes(n: usize) -> usize {
    n * 16
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// One row of the numbers table.
struct Row {
    label: String,
    cell_m: f64,
    w: usize,
    n: usize,
    iters: u32,
    record: bool,
    ms: f64,
    grid_bytes: usize,
    scratch_bytes: usize,
    heap_bytes: usize,
    units: usize,
}

impl Row {
    fn ms_per_iter(&self) -> f64 {
        self.ms / f64::from(self.iters.max(1))
    }
    fn peak_mib(&self) -> f64 {
        mib(self.grid_bytes + self.scratch_bytes + self.heap_bytes)
    }
    fn print(&self) {
        println!(
            "  {:<22} {:>7.0}m  {:>5}²={:>9}  it={:<4} rec={:<5}  {:>9.1} ms ({:>7.2} ms/it)  peak {:>8.1} MiB  units {}",
            self.label,
            self.cell_m,
            self.w,
            self.n,
            self.iters,
            self.record,
            self.ms,
            self.ms_per_iter(),
            self.peak_mib(),
            self.units,
        );
    }
}

fn run_row(pregen: &Pregen, label: &str, cell_m: f64, iters: u32, record: bool) -> (Row, DeepGrid) {
    let cfg = base_cfg(cell_m, iters, record);
    let t = Instant::now();
    let run = deeptime::run(pregen, &cfg);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let n = run.grid.w * run.grid.w;
    let row = Row {
        label: label.to_string(),
        cell_m,
        w: run.grid.w,
        n,
        iters,
        record,
        ms,
        grid_bytes: run.grid.resident_bytes(),
        scratch_bytes: run.erosion.scratch_bytes(),
        heap_bytes: heap_estimate_bytes(n),
        units: run.grid.total_units(),
    };
    (row, run.grid)
}

// ---------------------------------------------------------------- read quality

struct ReadQuality {
    land_cols: usize,
    depth_hist: [usize; 6], // 0,1,2,3,4,>=5 units
    mean_units: f64,
    max_units: usize,
    fining_cols: usize,
    unconformity_cols: usize,
    subsea_cols: usize,
    mean_tag_variety: f64,
}

fn read_quality(grid: &DeepGrid) -> ReadQuality {
    let mut land = 0usize;
    let mut hist = [0usize; 6];
    let mut total_units = 0usize;
    let mut max_units = 0usize;
    let mut fining = 0usize;
    let mut unconf = 0usize;
    let mut subsea = 0usize;
    let mut variety_sum = 0usize;
    for (i, s) in grid.strata.iter().enumerate() {
        if grid.surf_at(i) <= deeptime::SEA_LEVEL_M && s.units.is_empty() {
            continue; // bare ocean floor, no record: not a readable column
        }
        land += 1;
        let u = s.units.len();
        hist[u.min(5)] += 1;
        total_units += u;
        max_units = max_units.max(u);
        variety_sum += s.tag_variety();
        if s.unconformities() > 0 {
            unconf += 1;
        }
        if s.units
            .iter()
            .any(|x| matches!(x.tag().env, DepEnv::Subsea))
        {
            subsea += 1;
        }
        // Fining-upward: a conformable pair whose lower unit is higher energy
        // than the unit above it (coarse below, fine above).
        let fines = s.units.windows(2).any(|w| {
            energy_rank(w[0].tag().energy) > energy_rank(w[1].tag().energy) && !w[1].unconformity()
        });
        if fines {
            fining += 1;
        }
    }
    ReadQuality {
        land_cols: land,
        depth_hist: hist,
        mean_units: total_units as f64 / land.max(1) as f64,
        max_units,
        fining_cols: fining,
        unconformity_cols: unconf,
        subsea_cols: subsea,
        mean_tag_variety: variety_sum as f64 / land.max(1) as f64,
    }
}

fn energy_rank(e: EnergyBand) -> u8 {
    match e {
        EnergyBand::Low => 0,
        EnergyBand::Medium => 1,
        EnergyBand::High => 2,
    }
}

fn print_read_quality(rq: &ReadQuality) {
    println!(
        "    readable columns: {}   mean units {:.2}  max {}  mean tag-variety {:.2}",
        rq.land_cols, rq.mean_units, rq.max_units, rq.mean_tag_variety
    );
    println!("    unit-count hist [0 1 2 3 4 5+]: {:?}", rq.depth_hist);
    println!(
        "    fining-upward columns: {} ({:.1}%)   unconformity columns: {} ({:.1}%)   marine-bearing: {} ({:.1}%)",
        rq.fining_cols,
        100.0 * rq.fining_cols as f64 / rq.land_cols.max(1) as f64,
        rq.unconformity_cols,
        100.0 * rq.unconformity_cols as f64 / rq.land_cols.max(1) as f64,
        rq.subsea_cols,
        100.0 * rq.subsea_cols as f64 / rq.land_cols.max(1) as f64,
    );
}

/// Print a handful of scattered collapsed columns as annotated strata stacks
/// (top = surface). Picks cells spread across the grid, preferring rich records.
fn print_example_columns(grid: &DeepGrid, want: usize) {
    let w = grid.w;
    // Scatter probe positions on a coarse lattice, keep the richest records.
    let mut candidates: Vec<(usize, f64, f64)> = Vec::new();
    let step = (w / 12).max(1);
    let mut gy = step / 2;
    while gy < w {
        let mut gx = step / 2;
        while gx < w {
            let i = gy * w + gx;
            let s = &grid.strata[i];
            if !s.units.is_empty() && grid.surf_at(i) > deeptime::SEA_LEVEL_M {
                // Prefer columns that actually accumulated thickness AND carry a
                // varied record — the readable stories, not sub-cm drapes.
                let score = grid.h[i] + s.units.len() as f64 + 2.0 * s.tag_variety() as f64;
                candidates.push((i, score, grid.surf_at(i)));
            }
            gx += step;
        }
        gy += step;
    }
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
    for &(i, _, elev) in candidates.iter().take(want) {
        let gx = i % w;
        let gy = i / w;
        let s = &grid.strata[i];
        println!(
            "    column ({gx:>4},{gy:>4})  surface {elev:>7.1} m  R {:>7.1} m  H {:>5.1} m  {} units:",
            grid.r[i],
            grid.h[i],
            s.units.len()
        );
        // Top (youngest) first.
        for u in s.units.iter().rev() {
            let unc = if u.unconformity() {
                "  <-- unconformity (erosional gap)"
            } else {
                ""
            };
            let env = match u.tag().env {
                DepEnv::Subaerial => "subaerial",
                DepEnv::Subsea => "marine   ",
            };
            let ar = match (u.tag().env, u.tag().aridity) {
                (DepEnv::Subsea, _) => "     ",
                (_, Aridity::Arid) => "arid ",
                (_, Aridity::Humid) => "humid",
            };
            let en = match u.tag().energy {
                EnergyBand::Low => "low ",
                EnergyBand::Medium => "med ",
                EnergyBand::High => "high",
            };
            println!(
                "       {:>6.2} m  [{}]  {} {} energy:{}{}",
                u.thickness_m(),
                u.tag().code(),
                env,
                ar,
                en,
                unc
            );
        }
    }
}

// ------------------------------------------------------------------------ main

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let full_b = args.iter().any(|a| a == "--full-b");
    let b_iters = args
        .iter()
        .position(|a| a == "--b-iters")
        .and_then(|p| args.get(p + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(40);

    println!("== S9 deep-time spike, seed {SEED:#x}, Medium world (~251 km) ==\n");
    let t = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    println!(
        "pregen (17x17 cells): {:.1} ms\n",
        t.elapsed().as_secs_f64() * 1000.0
    );

    // ---- Coarse tiers (architecture A candidates + the resolution sweep) ----
    println!("-- Coarse global tiers (record ON, 200 iterations) --");
    let mut rows: Vec<Row> = Vec::new();
    let coarse = [
        ("A: 1 km", 1000.0),
        ("A: 500 m", 500.0),
        ("A(spec): 460 m", 460.0),
        ("A: 250 m", 250.0),
    ];
    let mut quality_grid: Option<DeepGrid> = None;
    for (label, cell_m) in coarse {
        let (row, grid) = run_row(&pregen, label, cell_m, 200, true);
        row.print();
        let rq = read_quality(&grid);
        print_read_quality(&rq);
        rows.push(row);
        if (cell_m - 500.0).abs() < 1.0 {
            quality_grid = Some(grid);
        }
    }

    // ---- B: landform-resolution global (48 m ~ 27 M cells) ----
    println!("\n-- B: landform-resolution global (record OFF, {b_iters} bounded iterations) --");
    if full_b {
        let (row, _grid) = run_row(&pregen, "B: 48 m FULL", 48.0, b_iters, false);
        row.print();
        println!(
            "    projected 200-iter wall: {:.1} s",
            row.ms_per_iter() * 200.0 / 1000.0
        );
    } else {
        // Extrapolation scales: 1/4 and 1/16 the cell count of the 48 m grid.
        for (label, cell_m) in [
            ("B/16: 192 m", 192.0),
            ("B/4: 96 m", 96.0),
            ("B: 48 m", 48.0),
        ] {
            let (row, _grid) = run_row(&pregen, label, cell_m, b_iters, false);
            row.print();
        }
        // Extrapolate 48 m from the ms/iter ~ O(n log n) trend, projected to
        // 200 iterations — see the results doc for the honest error bars.
    }

    // ---- C: bounded regional refinement + decay length ----
    // Two regimes over the same window and boundary bump: a diffusion-only
    // (pure relaxation) run and the full fluvial run. The halo theorem predicts
    // the relaxation error decays in a few cells; drainage rerouting makes the
    // full run's error global (the advective wall). Threshold 0.1 m.
    println!("\n-- C: regional refinement decay length (fine 96² window @ 250 m, 200 iters) --");
    let region = RegionSpec {
        px0: 5.5,
        py0: 5.5,
        cells: 96,
        cell_m: 250.0,
    };
    let thr = 0.1;
    let bump = 30.0;
    let halo = 3;
    // Pure relaxation: no rivers (k=0), no uplift, but weathering + hillslope
    // diffusion ON so there is regolith to creep — a genuine diffusive relaxer.
    let mut relax_cfg = base_cfg(250.0, 200, false);
    relax_cfg.sea_level_amp = 0.0; // static shoreline: clean signal
    relax_cfg.k_transport = 0.0;
    relax_cfg.k_bedrock = 0.0;
    relax_cfg.uplift_scale = 0.0;
    relax_cfg.weathering = 0.05;
    // Full fluvial: rivers on, so drainage can reroute.
    let mut full_cfg = base_cfg(250.0, 200, false);
    full_cfg.sea_level_amp = 0.0;

    for (label, cfg) in [
        ("relaxation (hillslope only)", relax_cfg),
        ("full fluvial", full_cfg),
    ] {
        let prof = deeptime::measure_decay(&pregen, &cfg, &region, halo, bump);
        let env = prof.envelope_cells(thr);
        let pen = prof.penetration_cells(thr);
        let verdict = if pen <= env + 1 {
            "RELAXES  (bulk == deepest → short halo suffices)"
        } else {
            "ADVECTS  (deep isolated spikes → drainage is global)"
        };
        println!(
            "  {label:<28}: bump {bump:.0} m, halo {halo}  bulk envelope {env} cells ({:.0} m)  deepest penetration {pen} cells ({:.0} m)  {verdict}",
            env as f64 * region.cell_m,
            pen as f64 * region.cell_m,
        );
        print!("    max-err (m) by boundary distance (cells):");
        for r in prof.rings.iter().step_by(2).take(16) {
            print!(" d{}={:.1e}", r.0, r.1);
        }
        println!();
    }

    // ---- Read-quality example columns (from the 500 m tier) ----
    if let Some(grid) = quality_grid.as_ref() {
        println!("\n-- Read-quality: example collapsed columns (500 m tier) --");
        print_example_columns(grid, 5);
    }

    println!("\n== done ==");
}
