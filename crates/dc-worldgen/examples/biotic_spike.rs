//! S10 biotic-layer spike measurement harness. Prints the cost table (biotic
//! layer ON vs OFF over a resolution/extent curve), the four read-quality target
//! signals with per-signal counts, and example annotated columns at named world
//! voxel coordinates. Recorded numbers live in docs/spikes/S10-results.md.
//!
//! Run: `cargo run --release -p dc-worldgen --example biotic_spike`
//! Args: `--iters N` (default 200), `--quick` (1 km only — a fast smoke run).
//!
//! `Instant` is wrapped strictly *around* runs, never inside the sim logic (the
//! crate rule): the deep-time engine consumes no wall clock.

use std::time::Instant;

use dc_worldgen::deeptime::{
    self, Biofacies, COAL_MIN_M, DeepConfig, DeepGrid, DeepRun, DepEnv, species_name,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn cfg_at(cell_m: f64, iterations: u32, biotic: bool) -> DeepConfig {
    DeepConfig {
        seed: SEED,
        cell_m,
        iterations,
        remarch_interval: 20,
        record: true,
        biotic,
        ..DeepConfig::default()
    }
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Heap high-water estimate for the priority flood (up to ~one Item per cell).
fn heap_estimate_bytes(n: usize) -> usize {
    n * 16
}

fn peak_bytes(run: &DeepRun) -> usize {
    let n = run.grid.w * run.grid.w;
    run.grid.resident_bytes()
        + run.erosion.scratch_bytes()
        + heap_estimate_bytes(n)
        + run.biota.as_ref().map_or(0, |b| b.scratch_bytes())
}

/// Deep-grid cell → world voxel coordinate of its centre (the inverse of
/// `DeepField::deep_coords`), so columns can be named in the coordinates a
/// walker/MCP session would use.
fn cell_to_voxel(gx: usize, gy: usize, w: usize, wp: usize) -> (i64, i64) {
    let half = (wp / 2) as f64;
    let to_v = |g: usize| {
        let p = (g as f64 + 0.5) / w as f64 * wp as f64;
        ((p - half) * CELL_VOXELS as f64).round() as i64
    };
    (to_v(gx), to_v(gy))
}

/// Cheap order-sensitive fingerprint of the whole record (determinism probe the
/// results doc can quote; the byte-identity falsifiers live in tests/).
fn fingerprint(grid: &DeepGrid) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x1000_0000_01b3);
    };
    for s in &grid.strata {
        mix(s.units.len() as u64);
        for u in &s.units {
            mix(u.thickness_m().to_bits());
            mix(u.tag().biota as u64);
            mix(u.tag().energy as u64);
            mix(u.tag().env as u64);
            mix(u.tag().aridity as u64);
            mix(u64::from(u.unconformity()));
        }
    }
    h
}

// ------------------------------------------------------------- read quality --

#[derive(Default)]
struct Signals {
    readable: usize,
    coal_cols: usize,
    coal_seams: usize,
    coal_max_m: f64,
    coal_total_m: f64,
    paleosol_cols: usize,
    paleosols: usize,
    charcoal_cols: usize,
    charcoal_bands: usize,
    retro_cols: usize,
    retro_bands: usize,
    peat_cols: usize,
    organic_cols: usize,
    units: usize,
}

fn scan(grid: &DeepGrid) -> Signals {
    let mut s = Signals::default();
    for (i, rec) in grid.strata.iter().enumerate() {
        if grid.surf_at(i) <= deeptime::SEA_LEVEL_M && rec.units.is_empty() {
            continue;
        }
        s.readable += 1;
        s.units += rec.units.len();
        let coal = rec.coal_seams(COAL_MIN_M);
        if coal > 0 {
            s.coal_cols += 1;
            s.coal_seams += coal;
            for u in &rec.units {
                if u.tag().biota == Biofacies::Coal {
                    s.coal_max_m = s.coal_max_m.max(u.thickness_m());
                    s.coal_total_m += u.thickness_m();
                }
            }
        }
        let paleo = rec.paleosols();
        if paleo > 0 {
            s.paleosol_cols += 1;
            s.paleosols += paleo;
        }
        let char_n = rec.charcoal_bands();
        if char_n > 0 {
            s.charcoal_cols += 1;
            s.charcoal_bands += char_n;
        }
        let retro = rec.retro_surfaces();
        if retro > 0 {
            s.retro_cols += 1;
            s.retro_bands += retro;
        }
        if rec.units.iter().any(|u| u.tag().biota == Biofacies::Peat) {
            s.peat_cols += 1;
        }
        if rec.units.iter().any(|u| u.tag().biota.is_organic()) {
            s.organic_cols += 1;
        }
    }
    s
}

fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

fn print_signals(s: &Signals) {
    println!(
        "    readable columns {}   total units {}   mean units/col {:.2}",
        s.readable,
        s.units,
        s.units as f64 / s.readable.max(1) as f64
    );
    println!(
        "    COAL      : {} cols ({:.2}%)  {} seams  max {:.2} m  total {:.0} m",
        s.coal_cols,
        pct(s.coal_cols, s.readable),
        s.coal_seams,
        s.coal_max_m,
        s.coal_total_m
    );
    println!(
        "    PALEOSOL  : {} cols ({:.2}%)  {} buried soil horizons",
        s.paleosol_cols,
        pct(s.paleosol_cols, s.readable),
        s.paleosols
    );
    println!(
        "    CHARCOAL  : {} cols ({:.2}%)  {} fire bands",
        s.charcoal_cols,
        pct(s.charcoal_cols, s.readable),
        s.charcoal_bands
    );
    println!(
        "    RETROGRESS: {} cols ({:.2}%)  {} sclerophyll horizons",
        s.retro_cols,
        pct(s.retro_cols, s.readable),
        s.retro_bands
    );
    println!(
        "    (peat-bearing {} cols, any-organic {} cols)",
        s.peat_cols, s.organic_cols
    );
}

/// Print one annotated column, top (youngest) first, naming its world voxel.
fn print_column(grid: &DeepGrid, wp: usize, i: usize, why: &str) {
    let w = grid.w;
    let (gx, gy) = (i % w, i / w);
    let (vx, vz) = cell_to_voxel(gx, gy, w, wp);
    let rec = &grid.strata[i];
    println!(
        "\n    [{why}] deep cell ({gx},{gy}) = world voxel ({vx}, {vz})  surface {:.1} m  H {:.2} m  {} units",
        grid.surf_at(i),
        grid.h[i],
        rec.units.len()
    );
    let last = rec.units.len().saturating_sub(1);
    for (k, u) in rec.units.iter().enumerate().rev() {
        let unc = if u.unconformity() {
            "  <-- unconformity"
        } else {
            ""
        };
        let buried = if k != last && u.tag().biota.is_organic() {
            "  <-- PALEOSOL (buried soil)"
        } else {
            ""
        };
        let env = match u.tag().env {
            DepEnv::Subaerial => "subaerial",
            DepEnv::Subsea => "marine   ",
        };
        let bio = match u.tag().biota {
            Biofacies::Mineral => "",
            Biofacies::Soil => "  organic soil horizon",
            Biofacies::Peat => "  PEAT (waterlogged organic)",
            Biofacies::Coal => "  COAL SEAM (buried compacted peat)",
            Biofacies::Charcoal => "  CHARCOAL band (fire event)",
            Biofacies::Retro => "  RETROGRESSIVE horizon (P-starved sclerophyll)",
        };
        println!(
            "       {:>7.3} m  [{:<9}]  {}{}{}{}",
            u.thickness_m(),
            u.tag().code(),
            env,
            bio,
            buried,
            unc
        );
    }
}

/// Find the best example column for a signal, by a scoring closure.
fn best<F: Fn(&DeepGrid, usize) -> Option<f64>>(grid: &DeepGrid, score: F) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for i in 0..grid.strata.len() {
        if let Some(v) = score(grid, i)
            && best.is_none_or(|(_, b)| v > b)
        {
            best = Some((i, v));
        }
    }
    best.map(|(i, _)| i)
}

fn print_community(run: &DeepRun, grid: &DeepGrid, i: usize) {
    let Some(b) = run.biota.as_ref() else { return };
    let Some(c) = b.cell(i) else { return };
    let mut parts: Vec<String> = Vec::new();
    for (s, &cov) in c.cover.iter().enumerate() {
        if cov > 0.01 {
            parts.push(format!("{} {:.0}%", species_name(s), cov * 100.0));
        }
    }
    let _ = grid;
    println!(
        "       community: [{}]  soil {:.2} m  N {:.3}  P {:.4} (rock-P left {:.3})  age {} epochs",
        parts.join(", "),
        c.soil,
        c.n,
        c.p_avail,
        c.p_rock,
        c.tsd
    );
}

// ------------------------------------------------------------------- main ----

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.iter().any(|a| a == "--quick");
    let iters = args
        .iter()
        .position(|a| a == "--iters")
        .and_then(|p| args.get(p + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(200);

    println!("== S10 biotic-layer spike, seed {SEED:#x}, {iters} iterations ==\n");
    println!(
        "sizes: DepUnit {} B (persistent, per recorded unit)   CellBiota {} B (transient, per cell)",
        std::mem::size_of::<dc_worldgen::deeptime::DepUnit>(),
        std::mem::size_of::<dc_worldgen::deeptime::CellBiota>(),
    );

    // ---- cost curve: ON vs OFF over the resolution/extent curve -------------
    println!("-- Cost: biotic layer OFF vs ON (Medium world ~251 km unless noted) --");
    println!(
        "  {:<20} {:>7} {:>6} {:>10} {:>11} {:>11} {:>8} {:>10} {:>10}",
        "case", "cell", "w", "cells", "OFF wall", "ON wall", "ratio", "OFF peak", "ON peak"
    );

    let medium = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let small = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });

    let cells_list: Vec<f64> = if quick {
        vec![1000.0]
    } else {
        vec![1000.0, 700.0, 500.0, 460.0]
    };

    let mut headline: Option<(DeepRun, usize)> = None;
    for cell_m in cells_list {
        let t = Instant::now();
        let off = deeptime::run(&medium, &cfg_at(cell_m, iters, false));
        let off_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let on = deeptime::run(&medium, &cfg_at(cell_m, iters, true));
        let on_ms = t.elapsed().as_secs_f64() * 1000.0;
        let n = on.grid.w * on.grid.w;
        println!(
            "  {:<20} {:>6.0}m {:>6} {:>10} {:>9.2} s {:>9.2} s {:>7.2}x {:>8.1}M {:>8.1}M",
            "Medium",
            cell_m,
            on.grid.w,
            n,
            off_ms / 1000.0,
            on_ms / 1000.0,
            on_ms / off_ms,
            mib(peak_bytes(&off)),
            mib(peak_bytes(&on)),
        );
        println!(
            "      units OFF {} / ON {}   biotic mass input {:.0} m·cells   fingerprint {:#018x}",
            off.grid.total_units(),
            on.grid.total_units(),
            on.biotic_total,
            fingerprint(&on.grid)
        );
        if (cell_m - 460.0).abs() < 1.0 || (quick && headline.is_none()) {
            headline = Some((on, medium.grid.w as usize));
        }
    }

    // Extent leg of the curve (Small at the same 460 m cell).
    {
        let t = Instant::now();
        let off = deeptime::run(&small, &cfg_at(460.0, iters, false));
        let off_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let on = deeptime::run(&small, &cfg_at(460.0, iters, true));
        let on_ms = t.elapsed().as_secs_f64() * 1000.0;
        let n = on.grid.w * on.grid.w;
        println!(
            "  {:<20} {:>6.0}m {:>6} {:>10} {:>9.2} s {:>9.2} s {:>7.2}x {:>8.1}M {:>8.1}M",
            "Small (~74 km)",
            460.0,
            on.grid.w,
            n,
            off_ms / 1000.0,
            on_ms / 1000.0,
            on_ms / off_ms,
            mib(peak_bytes(&off)),
            mib(peak_bytes(&on)),
        );
    }

    // ---- read quality on the headline (A-tier) run --------------------------
    let Some((run, wp)) = headline else {
        println!("\n(no headline run)");
        return;
    };
    let grid = &run.grid;
    println!("\n-- Read-quality: the four target signals (A tier, biotic ON) --");
    let s = scan(grid);
    print_signals(&s);

    // Community diagnostics: where did the vegetation actually get to?
    if let Some(b) = run.biota.as_ref() {
        let mut cover_sum = [0.0f64; deeptime::ROSTER];
        let mut land = 0usize;
        let mut soil_sum = 0.0f64;
        let mut p_sum = 0.0f64;
        let mut n_sum = 0.0f64;
        let mut total_cover_max = 0.0f32;
        for (i, c) in b.cells().iter().enumerate() {
            if grid.surf_at(i) <= deeptime::SEA_LEVEL_M {
                continue;
            }
            land += 1;
            let tc: f32 = c.cover.iter().sum();
            total_cover_max = total_cover_max.max(tc);
            for (s, &v) in c.cover.iter().enumerate() {
                cover_sum[s] += f64::from(v);
            }
            soil_sum += f64::from(c.soil);
            p_sum += f64::from(c.p_avail);
            n_sum += f64::from(c.n);
        }
        let l = land.max(1) as f64;
        println!("\n-- Community diagnostics (land cells {land}) --");
        let parts: Vec<String> = (0..deeptime::ROSTER)
            .map(|s| format!("{} {:.3}", species_name(s), cover_sum[s] / l))
            .collect();
        println!("    mean cover by species: {}", parts.join(", "));
        println!(
            "    mean soil {:.3} m   mean N {:.4}   mean available P {:.5}   max total cover {:.2}",
            soil_sum / l,
            n_sum / l,
            p_sum / l,
            total_cover_max
        );
        // Percentiles of the fields the niches are calibrated against — so the
        // tolerance ranges can be set from the distribution that actually
        // exists, not an imagined uniform 0..1.
        let mut precips: Vec<f32> = Vec::new();
        let mut pavail: Vec<f32> = Vec::new();
        for (i, c) in b.cells().iter().enumerate() {
            if grid.surf_at(i) > deeptime::SEA_LEVEL_M {
                precips.push(grid.precip[i]);
                pavail.push(c.p_avail);
            }
        }
        precips.sort_by(f32::total_cmp);
        pavail.sort_by(f32::total_cmp);
        let q = |v: &[f32], p: f64| v[((v.len() - 1) as f64 * p) as usize];
        println!(
            "    land precip percentiles: p05 {:.3}  p25 {:.3}  p50 {:.3}  p75 {:.3}  p95 {:.3}  max {:.3}",
            q(&precips, 0.05),
            q(&precips, 0.25),
            q(&precips, 0.50),
            q(&precips, 0.75),
            q(&precips, 0.95),
            q(&precips, 1.0)
        );
        println!(
            "    available-P percentiles: p05 {:.4}  p25 {:.4}  p50 {:.4}  p75 {:.4}  p95 {:.4}  max {:.4}",
            q(&pavail, 0.05),
            q(&pavail, 0.25),
            q(&pavail, 0.50),
            q(&pavail, 0.75),
            q(&pavail, 0.95),
            q(&pavail, 1.0)
        );
    }

    println!("\n-- Example columns (world voxel coordinates, seed {SEED:#x}) --");

    // Thickest coal seam.
    if let Some(i) = best(grid, |g, i| {
        let m: f64 = g.strata[i]
            .units
            .iter()
            .filter(|u| u.tag().biota == Biofacies::Coal)
            .map(|u| u.thickness_m())
            .sum();
        (m > 0.0).then_some(m)
    }) {
        print_column(grid, wp, i, "COAL SEAM");
        print_community(&run, grid, i);
    } else {
        println!("\n    [COAL SEAM] ABSENT — no unit reached the coal promotion threshold");
    }

    // Richest paleosol stack (most buried soil horizons).
    if let Some(i) = best(grid, |g, i| {
        let p = g.strata[i].paleosols();
        (p > 0).then_some(p as f64)
    }) {
        print_column(grid, wp, i, "PALEOSOLS");
        print_community(&run, grid, i);
    } else {
        println!("\n    [PALEOSOLS] ABSENT");
    }

    // Most charcoal bands.
    if let Some(i) = best(grid, |g, i| {
        let c = g.strata[i].charcoal_bands();
        (c > 0).then_some(c as f64)
    }) {
        print_column(grid, wp, i, "CHARCOAL");
        print_community(&run, grid, i);
    } else {
        println!("\n    [CHARCOAL] ABSENT");
    }

    // Most retrogressive horizons.
    if let Some(i) = best(grid, |g, i| {
        let r = g.strata[i].retro_surfaces();
        (r > 0).then_some(r as f64)
    }) {
        print_column(grid, wp, i, "RETROGRESSION");
        print_community(&run, grid, i);
    } else {
        println!("\n    [RETROGRESSION] ABSENT");
    }

    println!("\n== done ==");
}
