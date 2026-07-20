//! Tectonic-history spike measurement harness (tectonics.md § SPIKE).
//!
//! Prints the eight measurement groups' numbers for docs/spikes/S12-results.md:
//! stability, recorder growth, ritual cost (incl. the U3 200/300/400 tradeoff),
//! landform evidence proxies, forcing wavelength, determinism, relief/hypsometry
//! for the amplitude call, and drainage-export fidelity.
//!
//! Run: `cargo run --release -p dc-worldgen --example tectonic_spike`
//! `Instant` wraps runs strictly *around* the sim — the engine consumes no wall
//! clock (the crate rule). Cost is measured at the production entry
//! (`build_field_cfg`, the byte-identical parallel path — corrections #12).

use std::time::Instant;

use dc_worldgen::deeptime::{
    self, BoundaryKind, DeepConfig, DeepField, DeepGrid, Erosion, Plate, tectonics,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn base(seed: u64, cell_m: f64, iters: u32) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m,
        iterations: iters,
        record: true,
        biotic: true,
        erodibility: true,
        ..DeepConfig::default()
    }
}

fn tec(seed: u64, cell_m: f64, iters: u32) -> DeepConfig {
    DeepConfig {
        tectonic_history: true,
        ..base(seed, cell_m, iters)
    }
}

/// Production cell size for an extent (mirrors `field::production_config`).
fn prod_cell_m(cells: &dc_worldgen::pregen::CellGrid) -> f64 {
    let extent_m = cells.w as f64 * dc_worldgen::pregen::CELL_VOXELS as f64 * 0.9;
    (extent_m / 550.0).max(460.0)
}

// ----- relief / hypsometry -------------------------------------------------

fn relief_stats(field: &DeepField) -> (f64, f64, f64, f64) {
    // (max land elevation, land relief p50, p95, mean land surface)
    let mut land: Vec<f64> = field.surf.iter().copied().filter(|&s| s > 0.0).collect();
    if land.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    land.sort_by(f64::total_cmp);
    let n = land.len();
    let mean = land.iter().sum::<f64>() / n as f64;
    (land[n - 1], land[n / 2], land[(n * 95) / 100], mean)
}

// ----- record growth -------------------------------------------------------

fn record_stats(field: &DeepField) -> (usize, f64, usize, [usize; 6]) {
    let mut record_bearing = 0usize;
    let mut total = 0usize;
    let mut maxu = 0usize;
    let mut hist = [0usize; 6];
    for s in &field.strata {
        if s.units.is_empty() {
            continue;
        }
        record_bearing += 1;
        total += s.units.len();
        maxu = maxu.max(s.units.len());
        hist[s.units.len().min(5)] += 1;
    }
    let mean = total as f64 / record_bearing.max(1) as f64;
    (total, mean, maxu, hist)
}

// ----- landform proxies ----------------------------------------------------

/// Count columns whose record shows two belts of different chapters (a young and
/// an old orogenic pulse) — a superposition proxy (§ SPIKE 4b).
fn superposition_columns(field: &DeepField) -> usize {
    let mut n = 0;
    for s in &field.strata {
        let mut chapters_seen = [false; 32];
        let mut distinct = 0;
        for u in &s.units {
            let c = u.chapter as usize;
            if c < 32 && !chapters_seen[c] {
                chapters_seen[c] = true;
                distinct += 1;
            }
        }
        if distinct >= 3 {
            n += 1;
        }
    }
    n
}

/// Exhumed-core proxy: land cells whose cumulative exhumation exceeds a
/// threshold (deep basement brought to the surface — § SPIKE 4e).
fn exhumed_cells(field: &DeepField, thresh: f64) -> usize {
    field
        .exhum
        .iter()
        .zip(&field.surf)
        .filter(|&(&e, &s)| s > 0.0 && e > thresh)
        .count()
}

/// Foreland-wedge proxy: mean alluvium thickness in the low-elevation moat
/// (0–400 m) adjacent to high belts (>1500 m), relative to the global land mean.
fn foreland_ratio(field: &DeepField) -> f64 {
    let w = field.w;
    let strata_h = |i: usize| field.strata.get(i).map_or(0.0, |s| s.total_m());
    let mut moat_sum = 0.0;
    let mut moat_n = 0usize;
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let s = field.surf[i];
            if s <= 0.0 || s > 400.0 {
                continue;
            }
            // adjacent to a high belt?
            let mut near_belt = false;
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    let (nx, ny) = (gx as i32 + dx, gy as i32 + dy);
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < w {
                        near_belt |= field.surf[ny as usize * w + nx as usize] > 1500.0;
                    }
                }
            }
            if near_belt {
                moat_sum += strata_h(i);
                moat_n += 1;
            }
        }
    }
    let land_h: f64 = (0..w * w)
        .filter(|&i| field.surf[i] > 0.0)
        .map(strata_h)
        .sum();
    let land_n = (0..w * w).filter(|&i| field.surf[i] > 0.0).count();
    let land_mean = land_h / land_n.max(1) as f64;
    if moat_n == 0 || land_mean <= 0.0 {
        0.0
    } else {
        (moat_sum / moat_n as f64) / land_mean
    }
}

/// Water-gap proxy: transverse channels crossing a high axis. A cell is a "gap"
/// if it is a channel (large area) sitting in a local high, i.e. it maintains a
/// river through a ridge. Count cells with area>Q on land above `axis_m` whose
/// receiver climbs no ridge (the river holds its course).
fn water_gap_cells(field: &DeepField, axis_m: f64, q: f64) -> usize {
    let w = field.w;
    let mut n = 0;
    for i in 0..w * w {
        if field.surf[i] > axis_m && field.area[i] > q && field.recv[i] >= 0 {
            // The channel persists across a locally high ridge — a transverse gap.
            n += 1;
        }
    }
    n
}

// ----- cost ----------------------------------------------------------------

fn peak_bytes(field: &DeepField, grid_scratch: usize) -> usize {
    field.resident_bytes() + grid_scratch
}

fn main() {
    println!("== S12 tectonic-history spike, seed {SEED:#x} ==\n");

    let medium = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let large = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Large,
    });
    let mcell = prod_cell_m(&medium.grid);
    let lcell = prod_cell_m(&large.grid);
    println!("Medium production cell {mcell:.0} m; Large production cell {lcell:.0} m\n");

    // -- Group 5: forcing wavelength (analytic, no run) --
    println!("-- Group 5: forcing wavelength (analytic belt vs W) --");
    for w_km in [15.0, 25.0, 40.0] {
        let cfg = DeepConfig {
            tectonic_history: true,
            orogen_width_km: w_km,
            ..DeepConfig::default()
        };
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
        let v_ref = 8.0;
        let peak = tectonics::forcing_at(&plates, &cfg, v_ref, 150.0, 150.0);
        let mut half = f64::INFINITY;
        let mut x = 150.0;
        while x < 200.0 {
            if tectonics::forcing_at(&plates, &cfg, v_ref, x, 150.0) <= peak * 0.5 {
                half = x - 150.0;
                break;
            }
            x += 0.1;
        }
        println!(
            "  W={w_km:>4.0} km:  peak thickening {peak:>6.2} m/iter   gradation-to-peak (half-max) {half:>5.1} km"
        );
    }
    println!("  (the shipped machine smears every belt flank to ~50 km — earth-processes § 1)\n");

    // -- Group 3: ritual cost + U3 tradeoff (production entry, parallel path) --
    println!("-- Group 3: ritual cost @ production entry (build_field_cfg, parallel) --");
    println!("  extent   flag  iters   wall(s)   peak(MiB)   maxElev   units");
    let mut cost_runs: Vec<(&str, bool, u32, f64, f64, f64, usize)> = Vec::new();
    for (label, pregen, cell_m) in [("Medium", &medium, mcell), ("Large", &large, lcell)] {
        for &iters in &[200u32, 300, 400] {
            // off (legacy) baseline only needs 200 for the reference point
            if iters == 200 {
                let cfg = base(SEED, cell_m, iters);
                let t = Instant::now();
                let f = deeptime::build_field_cfg(&pregen.grid, &cfg);
                let wall = t.elapsed().as_secs_f64();
                let (maxe, _, _, _) = relief_stats(&f);
                let (units, _, _, _) = record_stats(&f);
                let pk = mib(peak_bytes(&f, 0));
                println!(
                    "  {label:<7} off   {iters:<5}  {wall:>7.2}  {pk:>9.1}  {maxe:>8.0}  {units}"
                );
                cost_runs.push((label, false, iters, wall, pk, maxe, units));
            }
            let cfg = tec(SEED, cell_m, iters);
            let t = Instant::now();
            let f = deeptime::build_field_cfg(&pregen.grid, &cfg);
            let wall = t.elapsed().as_secs_f64();
            let (maxe, _, _, _) = relief_stats(&f);
            let (units, _, _, _) = record_stats(&f);
            let pk = mib(peak_bytes(&f, 0));
            println!("  {label:<7} ON    {iters:<5}  {wall:>7.2}  {pk:>9.1}  {maxe:>8.0}  {units}");
            cost_runs.push((label, true, iters, wall, pk, maxe, units));
        }
    }

    // Isostasy per-iteration cost, isolated (the only new per-iter serial cost).
    println!("\n-- Group 3: isostasy phase cost, isolated (Medium production grid) --");
    {
        let cfg = tec(SEED, mcell, 1);
        let mut grid: DeepGrid = deeptime::build(&medium, &cfg);
        let mut ero = Erosion::new(&grid);
        let reps = 20;
        let t = Instant::now();
        let mut sink = 0.0;
        for _ in 0..reps {
            sink += ero.isostasy(&mut grid, &cfg);
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(reps);
        println!(
            "  isostasy (smooth load + Airy relax): {ms:.2} ms/iter  (×200 = {:.2} s)  [sink {sink:.0}]",
            ms * 200.0 / 1000.0
        );
    }

    // -- Group 2: recorder growth vs K --
    println!("\n-- Group 2: recorder growth vs chapter count K (Medium, biology on) --");
    println!("  config      units    mean/col   max   hist[0..5+]");
    {
        // off baseline
        let cfg = base(SEED, mcell, 200);
        let f = deeptime::build_field_cfg(&medium.grid, &cfg);
        let (u, m, mx, h) = record_stats(&f);
        println!("  off(K=1)    {u:>7}   {m:>6.2}   {mx:>4}   {h:?}");
    }
    for k in [4u32, 8, 16] {
        let cfg = DeepConfig {
            chapters: k,
            ..tec(SEED, mcell, 200)
        };
        let f = deeptime::build_field_cfg(&medium.grid, &cfg);
        let (u, m, mx, h) = record_stats(&f);
        println!("  K={k:<2}        {u:>7}   {m:>6.2}   {mx:>4}   {h:?}");
    }
    println!(
        "  sizeof(DepUnit) = {} bytes",
        std::mem::size_of::<deeptime::DepUnit>()
    );

    // -- Group 7: relief / hypsometry for the amplitude call --
    println!(
        "\n-- Group 7: relief vs thickening_scale (Medium, 200 iters) — for the amplitude call --"
    );
    println!("  thicken   maxElev   p50Land   p95Land   meanLand");
    for ts in [40.0, 80.0, 160.0] {
        let cfg = DeepConfig {
            thickening_scale: ts,
            ..tec(SEED, mcell, 200)
        };
        let f = deeptime::build_field_cfg(&medium.grid, &cfg);
        let (mx, p50, p95, mean) = relief_stats(&f);
        println!("  {ts:>5.0}     {mx:>7.0}   {p50:>7.0}   {p95:>7.0}   {mean:>7.0}");
    }

    // -- Group 4: landform evidence proxies (high-amplitude Medium run, ramped) --
    // The landform reads are gated on the amplitude call (U7): at the default
    // thickening the belts are only ~950 m, below the belt/moat thresholds. Run
    // the proxies at a high amplitude to show the landforms light up when the
    // user raises it — the § SPIKE 7 data is what that call is made against.
    println!(
        "\n-- Group 4: landform evidence (Medium, 300 iters, thickening_scale=240, ramped) --"
    );
    let rich_cfg = DeepConfig {
        thickening_scale: 240.0,
        ..tec(SEED, mcell, 300)
    };
    let rich = deeptime::build_field_cfg(&medium.grid, &rich_cfg);
    let step_cfg = DeepConfig {
        ramp_chapters: false,
        ..rich_cfg
    };
    let stepped = deeptime::build_field_cfg(&medium.grid, &step_cfg);
    let (rmax, _, _, _) = relief_stats(&rich);
    println!("  max belt elevation this run: {rmax:.0} m");
    let gap_r = water_gap_cells(&rich, 1200.0, 50.0);
    let gap_s = water_gap_cells(&stepped, 1200.0, 50.0);
    println!("  water-gap proxy (large-area channels held above 1200 m axis, area>50):");
    println!(
        "    ramped {gap_r}   stepped(control) {gap_s}   ramped/stepped {:.2}",
        gap_r as f64 / gap_s.max(1) as f64
    );
    println!(
        "  superposition columns (>=3 chapter-tagged units): {}",
        superposition_columns(&rich)
    );
    let ex_max = rich.exhum.iter().cloned().fold(0.0f64, f64::max);
    let ex_mean_land: f64 = {
        let (mut s, mut n) = (0.0, 0usize);
        for i in 0..rich.w * rich.w {
            if rich.surf[i] > 0.0 {
                s += rich.exhum[i];
                n += 1;
            }
        }
        s / n.max(1) as f64
    };
    println!(
        "  exhumation: max {ex_max:.0} m, land-mean {ex_mean_land:.0} m; exhumed-core cells (>500 m): {}",
        exhumed_cells(&rich, 500.0)
    );
    println!(
        "  foreland-wedge ratio (moat sediment / land-mean sediment): {:.2}",
        foreland_ratio(&rich)
    );

    // Rebound persistence: relief after a belt's convergence ends vs a no-isostasy
    // control is left descriptive (§ SPIKE 4f) — see results doc for the cut note.

    // -- Group 8: drainage export fidelity --
    println!("\n-- Group 8: drainage export fidelity --");
    {
        let n = rich.w * rich.w;
        let sink_area: f64 = (0..n)
            .filter(|&i| rich.recv[i] < 0)
            .map(|i| rich.area[i])
            .sum();
        let lakes = rich.lake.iter().filter(|&&b| b).count();
        println!(
            "  recv/area/lake exported: {} cells; Σ sink area {sink_area:.0} vs n {n} (Δ {:.2e})",
            rich.recv.len(),
            (sink_area - n as f64).abs()
        );
        println!("  exported lakes (closed basins at final stand): {lakes}");
        println!(
            "  chapter table entries: {} (each ~{} plates)",
            rich.chapters.len(),
            rich.chapters.first().map_or(0, Vec::len)
        );
    }

    // -- Group 1/6: determinism + boundary-kind census --
    println!("\n-- Group 6: determinism (double-run + census) --");
    {
        let a = deeptime::build_field_cfg(&medium.grid, &tec(SEED, mcell, 60));
        let b = deeptime::build_field_cfg(&medium.grid, &tec(SEED, mcell, 60));
        println!("  double-run surf identical: {}", a.surf == b.surf);
        println!("  double-run strata identical: {}", a.strata == b.strata);
        // Boundary census of chapter 0.
        let extent_km =
            medium.grid.w as f64 * dc_worldgen::pregen::CELL_VOXELS as f64 * 0.9 / 1000.0;
        let cfg = tec(SEED, mcell, 200);
        let table = tectonics::chapter_table(&cfg, extent_km);
        let v_ref = tectonics::reference_velocity(&cfg, extent_km);
        let plates = &table[0];
        let mut counts = [0usize; 7];
        let step = extent_km / 60.0;
        let mut y = 0.0;
        while y < extent_km {
            let mut x = 0.0;
            while x < extent_km {
                let k = tectonics::dominant_kind(plates, &cfg, v_ref, x, y);
                counts[kind_idx(k)] += 1;
                x += step;
            }
            y += step;
        }
        println!(
            "  chapter-0 boundary census (Orogeny,Arc,Trench,Rift,Ridge,Transform,Interior): {counts:?}"
        );
        println!("  plate count @ Medium: {}", plates.len());
    }

    println!("\n== done ==");
}

fn kind_idx(k: BoundaryKind) -> usize {
    match k {
        BoundaryKind::Orogeny => 0,
        BoundaryKind::Arc => 1,
        BoundaryKind::Trench => 2,
        BoundaryKind::Rift => 3,
        BoundaryKind::Ridge => 4,
        BoundaryKind::Transform => 5,
        BoundaryKind::Interior => 6,
    }
}
