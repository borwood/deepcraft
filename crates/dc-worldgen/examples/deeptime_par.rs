//! S9b measurement harness: per-phase profile, scalar↔parallel per-phase
//! speedups, rayon thread scaling, and (optionally) the full-B 27.3 M-cell wall
//! time. Recorded numbers live in `docs/spikes/S9b-results.md`.
//!
//! Run (scales + thread scaling):
//!   cargo run --release -p dc-worldgen --example deeptime_par
//! Full B (27.3 M cells, parallel, N bounded iters, projected to 200):
//!   cargo run --release -p dc-worldgen --example deeptime_par -- --full-b --iters 4
//!
//! `Instant` wraps phase calls strictly from the harness — the engine consumes
//! no wall clock. The phases are driven individually here (the profiling path);
//! `Erosion::step` runs the same sequence in production.

use std::time::Instant;

use dc_worldgen::deeptime::{self, DeepConfig, DeepGrid, Erosion};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;
const PHASES: [&str; 9] = [
    "uplift",
    "surface",
    "flood",
    "route",
    "accumulate",
    "transport",
    "weather",
    "diffuse",
    "record",
];

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

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Per-phase millisecond totals over the run (index matches [`PHASES`]).
struct Profile {
    ms: [f64; 9],
    ms_total: f64,
    n: usize,
    w: usize,
    iters: u32,
    grid_bytes: usize,
    scratch_bytes: usize,
}

impl Profile {
    fn per_iter(&self) -> [f64; 9] {
        let mut o = [0.0; 9];
        for (k, o) in o.iter_mut().enumerate() {
            *o = self.ms[k] / f64::from(self.iters.max(1));
        }
        o
    }
    fn total_per_iter(&self) -> f64 {
        self.ms_total / f64::from(self.iters.max(1))
    }
}

/// Drive the erosion phases individually, timing each, so we can attribute cost.
/// Mirrors `Erosion::step`'s phase order exactly.
fn profile(pregen: &Pregen, cfg: &DeepConfig, parallel: bool) -> Profile {
    let mut grid = deeptime::build(pregen, cfg);
    let mut ero = Erosion::new(&grid);
    ero.set_parallel(parallel);
    deeptime::climate::march(&mut grid, deeptime::sea_level_at(cfg, 0));
    let mut ms = [0.0f64; 9];
    let t_all = Instant::now();
    for it in 0..cfg.iterations {
        let sl = deeptime::sea_level_at(cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            deeptime::climate::march(&mut grid, sl);
        }
        ero.set_sea_level(sl);
        macro_rules! timed {
            ($k:expr, $call:expr) => {{
                let t = Instant::now();
                $call;
                ms[$k] += t.elapsed().as_secs_f64() * 1000.0;
            }};
        }
        timed!(0, {
            ero.apply_uplift(&mut grid, 1.0);
        });
        timed!(1, ero.build_surface(&grid));
        timed!(2, ero.flood());
        timed!(3, ero.route());
        timed!(4, ero.accumulate_area());
        timed!(5, ero.transport(&mut grid, cfg));
        timed!(6, ero.weather(&mut grid, cfg));
        timed!(7, ero.diffuse(&mut grid, cfg, 1.0));
        if cfg.record {
            timed!(8, ero.record(&mut grid));
        }
    }
    let ms_total = t_all.elapsed().as_secs_f64() * 1000.0;
    let n = grid.w * grid.w;
    Profile {
        ms,
        ms_total,
        n,
        w: grid.w,
        iters: cfg.iterations,
        grid_bytes: grid.resident_bytes(),
        scratch_bytes: ero.scratch_bytes(),
    }
}

fn print_phase_table(scalar: &Profile, par: &Profile, label: &str) {
    let sp = scalar.per_iter();
    let pp = par.per_iter();
    println!(
        "\n== {label}: {}² = {} cells, {} iters ==",
        scalar.w, scalar.n, scalar.iters
    );
    println!(
        "  grid {:.1} MiB + scratch {:.1} MiB",
        mib(scalar.grid_bytes),
        mib(scalar.scratch_bytes)
    );
    println!(
        "  {:<12} {:>12} {:>12} {:>9} {:>7}",
        "phase", "scalar ms/it", "par ms/it", "speedup", "scal%"
    );
    let stot: f64 = sp.iter().sum();
    for k in 0..9 {
        if sp[k] == 0.0 && pp[k] == 0.0 {
            continue;
        }
        let speedup = if pp[k] > 0.0 { sp[k] / pp[k] } else { 0.0 };
        println!(
            "  {:<12} {:>12.3} {:>12.3} {:>8.2}x {:>6.1}%",
            PHASES[k],
            sp[k],
            pp[k],
            speedup,
            100.0 * sp[k] / stot.max(1e-9),
        );
    }
    let stot_it = scalar.total_per_iter();
    let ptot_it = par.total_per_iter();
    println!(
        "  {:<12} {:>12.3} {:>12.3} {:>8.2}x  (whole-step, incl. loop overhead)",
        "TOTAL",
        stot_it,
        ptot_it,
        if ptot_it > 0.0 {
            stot_it / ptot_it
        } else {
            0.0
        },
    );
    // Amdahl: serial floor = flood + accumulate + transport (the scalar phases).
    let serial: f64 = sp[2] + sp[4] + sp[5];
    println!(
        "  serial floor (flood+accumulate+transport): {:.3} ms/it = {:.1}% of scalar step  → Amdahl max speedup {:.1}x",
        serial,
        100.0 * serial / stot.max(1e-9),
        stot / serial.max(1e-9),
    );
}

/// Whole-step wall time (ms/iter) at a given rayon thread count, parallel path.
fn thread_scaling(pregen: &Pregen, cfg: &DeepConfig, threads: usize) -> f64 {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("build rayon pool");
    pool.install(|| {
        let p = profile(pregen, cfg, true);
        p.total_per_iter()
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let full_b = args.iter().any(|a| a == "--full-b");
    let iters = args
        .iter()
        .position(|a| a == "--iters")
        .and_then(|p| args.get(p + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(4);

    let cores = std::thread::available_parallelism().map_or(0, |n| n.get());
    println!("== S9b parallel deep-time, seed {SEED:#x} ==");
    println!("available_parallelism (logical CPUs): {cores}");

    let t = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    println!(
        "pregen (17x17): {:.1} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );

    if full_b {
        // The whole point: does B fit patience once parallel?
        println!("\n-- FULL B: 48 m, ~27.3 M cells, parallel, {iters} bounded iters --");
        let cfg = base_cfg(48.0, iters, false);
        let t = Instant::now();
        let par = profile(&pregen, &cfg, true);
        let wall = t.elapsed().as_secs_f64();
        println!(
            "  {}² = {} cells; parallel {:.2} s for {} iters → {:.1} ms/iter",
            par.w,
            par.n,
            wall,
            iters,
            par.total_per_iter(),
        );
        println!(
            "  projected 200-iter parallel wall: {:.1} s ({:.2} min)   grid {:.1} MiB + scratch {:.1} MiB",
            par.total_per_iter() * 200.0 / 1000.0,
            par.total_per_iter() * 200.0 / 60000.0,
            mib(par.grid_bytes),
            mib(par.scratch_bytes),
        );
        let pp = par.per_iter();
        let serial = pp[2] + pp[4] + pp[5];
        println!(
            "  of which serial floor (flood+accum+transport): {:.1} ms/it → {:.1} s over 200 iters",
            serial,
            serial * 200.0 / 1000.0
        );
        return;
    }

    // ---- Phase profiles at 1.7 M (192 m) and 6.8 M (96 m) ----
    let iters_prof = 6;
    for (label, cell_m) in [("1.7 M (192 m)", 192.0), ("6.8 M (96 m)", 96.0)] {
        let cfg = base_cfg(cell_m, iters_prof, false);
        let scalar = profile(&pregen, &cfg, false);
        let par = profile(&pregen, &cfg, true);
        print_phase_table(&scalar, &par, label);
    }

    // ---- Thread scaling (whole parallel step, ms/iter) at 6.8 M ----
    println!("\n== thread scaling (6.8 M cells / 96 m, whole parallel step ms/iter) ==");
    let cfg = base_cfg(96.0, 6, false);
    let scalar_ref = profile(&pregen, &cfg, false).total_per_iter();
    println!("  scalar (1 thread reference): {scalar_ref:.2} ms/it");
    for threads in [1usize, 2, 4, 8, 12, 16] {
        let ms = thread_scaling(&pregen, &cfg, threads);
        println!(
            "  {threads:>2} threads: {ms:>8.2} ms/it   speedup vs scalar {:.2}x",
            scalar_ref / ms.max(1e-9)
        );
    }

    // ---- Flood-parallelism probe: the crux the S9 flip condition named ----
    // The priority-flood is the step's dominant serial cost. Measure the
    // optimistic tiled-flood ceiling (open seams, no reconciliation) and the
    // correctness debt (divergence vs the serial fill).
    println!("\n== flood parallelism probe (optimistic open-seam tiled flood) ==");
    for (label, cell_m) in [("1.7 M (192 m)", 192.0), ("6.8 M (96 m)", 96.0)] {
        let cfg = base_cfg(cell_m, 1, false);
        let grid = deeptime::build(&pregen, &cfg);
        let surf: Vec<f64> = (0..grid.w * grid.w).map(|i| grid.surf_at(i)).collect();
        let sea = deeptime::sea_level_at(&cfg, 0);
        let t = Instant::now();
        let serial = deeptime::flood_fill_serial(grid.w, &surf, sea);
        let serial_ms = t.elapsed().as_secs_f64() * 1000.0;
        println!("  {label}: serial flood {serial_ms:.1} ms");
        for strips in [2usize, 4, 6, 8, 12] {
            let t = Instant::now();
            let tiled = deeptime::flood_fill_tiled(grid.w, &surf, sea, strips);
            let tiled_ms = t.elapsed().as_secs_f64() * 1000.0;
            let (mut maxd, mut ndiff) = (0.0f64, 0usize);
            for i in 0..tiled.len() {
                let d = (tiled[i] - serial[i]).abs();
                if d > 0.01 {
                    ndiff += 1;
                }
                maxd = maxd.max(d);
            }
            println!(
                "    {strips:>2} strips: {tiled_ms:>7.1} ms  speedup {:>4.2}x   divergence max {:>6.2} m, {} cells (>1 cm) = {:.2}%",
                serial_ms / tiled_ms.max(1e-9),
                maxd,
                ndiff,
                100.0 * ndiff as f64 / tiled.len() as f64,
            );
        }
    }

    // ---- Recorder memory: struct-header overhead (the 648 MiB S9 flagged) ----
    println!("\n== recorder memory (per-cell DeepStrata struct overhead) ==");
    let strata_struct = std::mem::size_of::<deeptime::DeepStrata>();
    for (label, n) in [
        ("A 460 m (297 k)", 297_000usize),
        ("B 48 m (27.3 M)", 27_300_000usize),
    ] {
        println!(
            "  {label}: {} B/cell × {} cells = {:.0} MiB of empty struct array (before any unit)",
            strata_struct,
            n,
            mib(strata_struct * n)
        );
    }
    // Measured resident with record on vs off at a coarse scale where record is
    // affordable, to show the header + unit cost concretely.
    let rec_on = deeptime::run(&pregen, &base_cfg(500.0, 200, true)).grid;
    let rec_off = deeptime::build(&pregen, &base_cfg(500.0, 200, false));
    print_mem("500 m record ON ", &rec_on);
    print_mem("500 m record OFF", &rec_off);

    println!("\n== done ==");
}

fn print_mem(label: &str, grid: &DeepGrid) {
    println!(
        "  {label}: resident {:.1} MiB, {} strata units",
        mib(grid.resident_bytes()),
        grid.total_units()
    );
}
