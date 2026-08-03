//! The guided-tour map for the erosion-agent roster flip (journal/0047).
//!
//! journal/0034 built the wind + frost + wave roster and measured its four
//! signatures on a seeded *Small* world at 1 km cells over 40 epochs. On
//! 2026-07-21 the user flipped `full_agents` ON in production and asked to judge
//! the (unratified) agent magnitudes **live**, standing in the world, station by
//! station. This example prints the station list: for the world the dc-client
//! actually boots (client `BENCH_SEED` = 1337, `Extent::Medium`, the full
//! production config with the roster on) it finds, for each of the five agent
//! signatures, the strongest exemplar cell and reports the coordinate a player is
//! teleported to, the local surface elevation, the signal magnitude there, and a
//! caveat when the signature has no legible exemplar at production magnitudes.
//!
//! Attribution is by **isolation**, exactly as the `full_agents` test suite does
//! it: five deep-time runs on the same pregen — the production all-on run (the
//! surface the walker actually stands on, and the strata record the dune/loess
//! facies live in), an all-off run (the baseline), and one run per agent with the
//! other two rate knobs zeroed. Deflation is `ΔH` (off→wind-only) in arid cells,
//! frost stripping is `ΔR` (off→frost-only) in the periglacial band, wave retreat
//! is `Δsurf` (off→wave-only) on the coast. Nothing here is on a generation path.
//!
//! `cargo run --release -p dc-worldgen --example tour_map`

use std::time::Instant;

use dc_worldgen::deeptime::climate::air_temp_c;
use dc_worldgen::deeptime::{
    self, DeepConfig, DeepOverrides, DeepRun, Eolian, build_field, build_field_with,
    production_config,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

/// The client's `BENCH_SEED` (dc-client/src/bench.rs) — the world every walk so
/// far has stood in.
const SEED: u64 = 1337;
/// `GenOptions::default().extent` — the client's boot extent.
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;

/// journal/0046's scarp region (world metres) — ground the user already knows;
/// the wave station prefers a coast near here when it is comparable.
const SCARP_XZ_M: (f64, f64) = (-16545.0, 10592.0);

/// A deposit/stripping magnitude below this (metres) is not a landform a walker
/// will read at production magnitudes — the "this station may show nothing" line.
const LEGIBLE_M: f64 = 1.0;

/// Border ring (deep cells) excluded from the eolian/frost station picks. 0034's
/// wind march settles "whatever is still aloft at the downwind land edge" there,
/// so the single strongest loess/deflation/frost cell piles up on the last column
/// — a march artifact, not a landform a walker should stand in. Stations report
/// the strongest INTERIOR exemplar instead; the edge pileup is called out in the
/// journal.
const EDGE_MARGIN: i64 = 4;

/// Is deep cell `idx` clear of the [`EDGE_MARGIN`] border ring?
fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

fn main() {
    println!("=== journal/0047 guided-tour map: the erosion-agent roster ===");
    println!(
        "seed {SEED}, extent {}, production config (full_agents ON), N=2 ({VOXEL_M} m voxels)\n",
        EXTENT.label()
    );

    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    eprintln!(
        "pregen (with internal deep-time run) built in {:?}",
        t0.elapsed()
    );

    let base = production_config(&pregen.grid, SEED);
    assert!(
        base.full_agents,
        "production_config must have full_agents ON post-flip"
    );

    // Cost A/B at the production entry point (`build_field`), roster on vs off,
    // seed-matched same-build via the override channel — the DeepField the world
    // actually keeps (surf + strata + tectonic exports), not the run scratch.
    cost_ab(&pregen);

    // Five runs on the same pregen, byte-identical parallel path. `full` is the
    // production world (what the walker stands on and where the eolian facies are
    // recorded); the others isolate one agent each for clean attribution.
    let full = run("full (production, all agents)", &pregen, base);
    let off = run(
        "off (full_agents off)",
        &pregen,
        cfg(base, |c| c.full_agents = false),
    );
    let wind = run(
        "wind-only",
        &pregen,
        cfg(base, |c| {
            c.frost_weathering_gain = 0.0;
            c.wave_erosion = 0.0;
        }),
    );
    let frost = run(
        "frost-only",
        &pregen,
        cfg(base, |c| {
            c.eolian_deflation = 0.0;
            c.wave_erosion = 0.0;
        }),
    );
    let wave = run(
        "wave-only",
        &pregen,
        cfg(base, |c| {
            c.eolian_deflation = 0.0;
            c.frost_weathering_gain = 0.0;
        }),
    );

    let w = full.grid.w;
    let cell_m = full.grid.cell_m;
    let wp = pregen.deep.wp;
    println!(
        "\ndeep grid {w}x{w} @ {cell_m:.1} m/cell; pregen {wp}x{wp}; max production surf {:.1} m\n",
        (0..w * w)
            .map(|i| full.grid.surf_at(i))
            .fold(f64::MIN, f64::max)
    );

    // Aggregate signatures at production scale (the 0034-comparable numbers).
    aggregates(&off, &wind, &frost, &wave);

    let conv = Conv { w, wp };
    station_dune(&full, &conv);
    station_loess(&full, &off, &conv);
    station_deflation(&off, &wind, &full, &conv);
    station_periglacial(&off, &frost, &full, &conv);
    station_wave(&off, &wave, &full, &conv);
}

/// Ritual wall time + kept-resident bytes at Medium, roster ON (production) vs
/// OFF (`DeepOverrides { full_agents: Some(false) }`), same build, same seed.
fn cost_ab(pregen: &Pregen) {
    let t_on = Instant::now();
    let on = build_field(&pregen.grid, SEED);
    let d_on = t_on.elapsed();
    let t_off = Instant::now();
    let off = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            full_agents: Some(false),
            ..DeepOverrides::default()
        },
    );
    let d_off = t_off.elapsed();
    let mb = |b: usize| b as f64 / (1024.0 * 1024.0);
    println!("--- ritual cost (build_field, Medium, roster OFF vs ON) ---");
    println!(
        "  OFF: {:>7.1} ms, resident {:>6.2} MB",
        d_off.as_secs_f64() * 1e3,
        mb(off.resident_bytes())
    );
    println!(
        "  ON : {:>7.1} ms, resident {:>6.2} MB  (Δ {:+.1} ms, {:+.2} MB)\n",
        d_on.as_secs_f64() * 1e3,
        mb(on.resident_bytes()),
        (d_on.as_secs_f64() - d_off.as_secs_f64()) * 1e3,
        mb(on.resident_bytes()) - mb(off.resident_bytes()),
    );
}

/// A `DeepConfig` with a mutation applied (config is `Copy`).
fn cfg(base: DeepConfig, f: impl FnOnce(&mut DeepConfig)) -> DeepConfig {
    let mut c = base;
    f(&mut c);
    c
}

fn run(label: &str, pregen: &Pregen, cfg: DeepConfig) -> DeepRun {
    let t = Instant::now();
    let r = deeptime::run_with(pregen, &cfg, true);
    eprintln!("  ran {label} in {:?}", t.elapsed());
    r
}

// ---------------------------------------------------------------- coordinates

/// The deep-cell ↔ world-voxel bridge, lifted verbatim from `roughness_probe`
/// (`DeepField::deep_coords`' own centring: integer `wp/2`).
struct Conv {
    w: usize,
    wp: usize,
}

impl Conv {
    /// World voxel centre of deep cell `idx` (inverse of `deep_coords`).
    fn idx_to_voxel(&self, idx: usize) -> (i64, i64) {
        let w = self.w as f64;
        let wp = self.wp as f64;
        let (gx, gy) = ((idx % self.w) as f64, (idx / self.w) as f64);
        let px = (gx + 0.5) / w * wp - 0.5;
        let py = (gy + 0.5) / w * wp - 0.5;
        let half = (self.wp / 2) as f64;
        (
            ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
            ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        )
    }

    /// Full station line: `deep cell (gx,gy) | voxel (vx,vz) | world (mx,mz) m`.
    fn where_line(&self, idx: usize) -> String {
        let (gx, gy) = (idx % self.w, idx / self.w);
        let (vx, vz) = self.idx_to_voxel(idx);
        format!(
            "deep cell ({gx},{gy}) | voxel ({vx}, {vz}) | world ({:.0} m, {:.0} m)",
            vx as f64 * VOXEL_M,
            vz as f64 * VOXEL_M
        )
    }

    fn meters(&self, idx: usize) -> (f64, f64) {
        let (vx, vz) = self.idx_to_voxel(idx);
        (vx as f64 * VOXEL_M, vz as f64 * VOXEL_M)
    }
}

// ---------------------------------------------------------------- aggregates

/// The four 0034 signatures summed over the whole production-Medium world, so the
/// journal can compare against the seeded-Small numbers (deflation −104.3 m,
/// 71.4 m loess + 65.5 m dunes, +668.5 m frost regolith, ~1.3 m coastal).
fn aggregates(off: &DeepRun, wind: &DeepRun, frost: &DeepRun, wave: &DeepRun) {
    let w = off.grid.w;
    let arid = off_arid_threshold(off);

    // Eolian deposits: sum tagged thickness across the wind-only record.
    let (mut loess, mut dune) = (0.0f64, 0.0f64);
    for s in &wind.grid.strata {
        for u in &s.units {
            match u.tag().eolian {
                Eolian::Loess => loess += u.thickness_m(),
                Eolian::Dune => dune += u.thickness_m(),
                Eolian::None => {}
            }
        }
    }
    // Net deflation over arid subaerial cells (off ΣH → wind ΣH).
    let (mut h_off, mut h_on, mut arid_cells) = (0.0f64, 0.0f64, 0usize);
    for i in 0..w * w {
        if off.grid.surf_at(i) > 0.0 && f64::from(off.grid.precip[i]) < arid {
            h_off += off.grid.h[i];
            h_on += wind.grid.h[i];
            arid_cells += 1;
        }
    }
    // Frost: extra bedrock stripped across the periglacial band.
    let width = off_frost_width(off);
    let (mut r_off, mut r_on, mut band_cells) = (0.0f64, 0.0f64, 0usize);
    for gy in 0..w {
        let lat = off.grid.lat_deg(gy);
        for gx in 0..w {
            let i = gy * w + gx;
            let surf = off.grid.surf_at(i);
            if surf > 0.0 && f64::from(air_temp_c(lat, surf)).abs() < width {
                r_off += off.grid.r[i];
                r_on += frost.grid.r[i];
                band_cells += 1;
            }
        }
    }
    // Wave: total lowering across coastal cells.
    let band = off_wave_band(off);
    let (mut s_off, mut s_on, mut coastal) = (0.0f64, 0.0f64, 0usize);
    for i in 0..w * w {
        let free = off.grid.surf_at(i);
        if free > 0.0 && free <= band {
            s_off += free;
            s_on += wave.grid.surf_at(i);
            coastal += 1;
        }
    }
    println!("--- aggregate signatures over the whole production world ---");
    println!("  eolian deposits (wind-only record): dune {dune:.1} m, loess {loess:.1} m");
    println!(
        "  net deflation over {arid_cells} arid cells: ΣH {h_off:.1} → {h_on:.1} m ({:.1} m stripped)",
        h_off - h_on
    );
    println!(
        "  frost band {band_cells} cells: ΣR {r_off:.0} → {r_on:.0} m ({:.1} m extra bedrock stripped)",
        r_off - r_on
    );
    println!(
        "  wave coast {coastal} cells: Σsurf {s_off:.0} → {s_on:.0} m ({:.1} m total lowering)\n",
        s_off - s_on
    );
}

fn off_arid_threshold(_run: &DeepRun) -> f64 {
    DeepConfig::default().eolian_arid_precip
}
fn off_frost_width(_run: &DeepRun) -> f64 {
    DeepConfig::default().frost_band_width_c
}
fn off_wave_band(_run: &DeepRun) -> f64 {
    DeepConfig::default().wave_band_m
}

// ---------------------------------------------------------------- stations

/// Local relief (metres) over a Chebyshev-`radius` window of the production
/// surface — how much shape surrounds a station (a plateau reads ~0).
fn local_relief(full: &DeepRun, idx: usize, radius: i64) -> f64 {
    let w = full.grid.w as i64;
    let (cx, cy) = ((idx % full.grid.w) as i64, (idx / full.grid.w) as i64);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= w || y >= w {
                continue;
            }
            let s = full.grid.surf_at((y * w + x) as usize);
            lo = lo.min(s);
            hi = hi.max(s);
        }
    }
    hi - lo
}

fn station_dune(full: &DeepRun, conv: &Conv) {
    // Max eolian-dune deposition, read straight from the production record.
    let mut best = (0usize, 0.0f64);
    for (i, s) in full.grid.strata.iter().enumerate() {
        if !interior(full.grid.w, i) {
            continue;
        }
        let dune: f64 = s
            .units
            .iter()
            .filter(|u| u.tag().eolian == Eolian::Dune)
            .map(|u| u.thickness_m())
            .sum();
        if dune > best.1 {
            best = (i, dune);
        }
    }
    header("STATION 1 — DUNE FIELD");
    if best.1 < LEGIBLE_M {
        null_note("dune", best.1);
    }
    println!("  {}", conv.where_line(best.0));
    println!("  surface elevation : {:.1} m", full.grid.surf_at(best.0));
    println!("  dune deposit      : {:.2} m of Dune-tagged sand", best.1);
    println!(
        "  local relief (±2) : {:.1} m",
        local_relief(full, best.0, 2)
    );
    println!(
        "  SEE: a coarse wind-blown sand body in the arid interior; knob `eolian_deflation`\n\
         \x20      (source supply) with `eolian_deposit_frac` (how much settles)."
    );
}

fn station_loess(full: &DeepRun, off: &DeepRun, conv: &Conv) {
    // Max loess deposition; then check the desert (arid/dune source) is crossable
    // within 2 km so the walker can stand on the transition.
    let mut best = (0usize, 0.0f64);
    for (i, s) in full.grid.strata.iter().enumerate() {
        if !interior(full.grid.w, i) {
            continue;
        }
        let loess: f64 = s
            .units
            .iter()
            .filter(|u| u.tag().eolian == Eolian::Loess)
            .map(|u| u.thickness_m())
            .sum();
        if loess > best.1 {
            best = (i, loess);
        }
    }
    header("STATION 2 — LOESS MARGIN");
    if best.1 < LEGIBLE_M {
        null_note("loess", best.1);
    }
    println!("  {}", conv.where_line(best.0));
    println!("  surface elevation : {:.1} m", full.grid.surf_at(best.0));
    println!("  loess deposit     : {:.2} m of Loess-tagged silt", best.1);
    // Nearest arid/dune source within a 2 km search.
    let arid = DeepConfig::default().eolian_arid_precip;
    let w = full.grid.w as i64;
    let r = (2000.0 / full.grid.cell_m).ceil() as i64;
    let (cx, cy) = ((best.0 % full.grid.w) as i64, (best.0 / full.grid.w) as i64);
    let mut nearest: Option<f64> = None;
    for dy in -r..=r {
        for dx in -r..=r {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= w || y >= w {
                continue;
            }
            let j = (y * w + x) as usize;
            let is_dune = full.grid.strata[j]
                .units
                .iter()
                .any(|u| u.tag().eolian == Eolian::Dune);
            let is_arid = off.grid.surf_at(j) > 0.0 && f64::from(off.grid.precip[j]) < arid;
            if is_dune || is_arid {
                let d = ((dx * dx + dy * dy) as f64).sqrt() * full.grid.cell_m;
                nearest = Some(nearest.map_or(d, |n| n.min(d)));
            }
        }
    }
    match nearest {
        Some(d) if d <= 2000.0 => println!(
            "  desert source     : arid/dune ground {d:.0} m away — transition is walkable (<2 km)"
        ),
        Some(d) => println!("  desert source     : nearest arid/dune ground {d:.0} m away (>2 km)"),
        None => println!("  desert source     : no arid/dune ground within 2 km of this margin"),
    }
    println!(
        "  SEE: a fine wind-blown silt sheet on the humid downwind margin of the desert;\n\
         \x20      knob `eolian_deposit_frac` (settling fraction at the vegetated margin)."
    );
}

fn station_deflation(off: &DeepRun, wind: &DeepRun, full: &DeepRun, conv: &Conv) {
    // Strongest wind-stripped arid cell: max ΔH (off→wind) over arid subaerial
    // ground.
    let arid = DeepConfig::default().eolian_arid_precip;
    let w = off.grid.w;
    let mut best = (0usize, 0.0f64);
    for i in 0..w * w {
        if interior(w, i) && off.grid.surf_at(i) > 0.0 && f64::from(off.grid.precip[i]) < arid {
            let d = off.grid.h[i] - wind.grid.h[i];
            if d > best.1 {
                best = (i, d);
            }
        }
    }
    header("STATION 3 — DEFLATION BASIN");
    if best.1 < LEGIBLE_M {
        null_note("wind deflation", best.1);
    }
    println!("  {}", conv.where_line(best.0));
    println!("  surface elevation : {:.1} m", full.grid.surf_at(best.0));
    println!(
        "  loose cover lost  : {:.2} m of H deflated (off→wind)",
        best.1
    );
    println!(
        "  SEE: a wind-scoured arid hollow, loose cover blown out to bedrock/lag; knob\n\
         \x20      `eolian_deflation` (base entrainment), gated by `eolian_arid_precip`."
    );
}

fn station_periglacial(off: &DeepRun, frost: &DeepRun, full: &DeepRun, conv: &Conv) {
    // Strongest frost stripping (ΔR off→frost) in the near-0 °C band, preferring a
    // high-relief site so the signature isn't lost in a plateau.
    let width = DeepConfig::default().frost_band_width_c;
    let w = off.grid.w;
    let mut band: Vec<(usize, f64)> = Vec::new();
    for gy in 0..w {
        let lat = off.grid.lat_deg(gy);
        for gx in 0..w {
            let i = gy * w + gx;
            if !interior(w, i) {
                continue;
            }
            let surf = off.grid.surf_at(i);
            if surf > 0.0 && f64::from(air_temp_c(lat, surf)).abs() < width {
                let d = off.grid.r[i] - frost.grid.r[i];
                if d > 0.0 {
                    band.push((i, d));
                }
            }
        }
    }
    header("STATION 4 — PERIGLACIAL BAND");
    if band.is_empty() {
        println!("  NULL: no periglacial (near-0 °C) band cells stripped on this world.");
        return;
    }
    // Raw strongest, and the strongest among high-relief (above-median relief)
    // cells — the integrator picks; the high-relief one is preferred.
    band.sort_by(|a, b| b.1.total_cmp(&a.1));
    let raw = band[0];
    let mut reliefs: Vec<f64> = band
        .iter()
        .map(|(i, _)| local_relief(full, *i, 2))
        .collect();
    reliefs.sort_by(f64::total_cmp);
    let median_relief = reliefs[reliefs.len() / 2];
    let hi_relief = band
        .iter()
        .find(|(i, _)| local_relief(full, *i, 2) >= median_relief)
        .copied()
        .unwrap_or(raw);

    if raw.1 < LEGIBLE_M {
        null_note("frost stripping", raw.1);
    }
    println!("  PREFERRED (high-relief):");
    println!("    {}", conv.where_line(hi_relief.0));
    println!(
        "    surface elevation : {:.1} m",
        full.grid.surf_at(hi_relief.0)
    );
    println!(
        "    extra bedrock lost: {:.2} m of R stripped (off→frost)",
        hi_relief.1
    );
    println!(
        "    local relief (±2) : {:.1} m",
        local_relief(full, hi_relief.0, 2)
    );
    println!("  RAW STRONGEST:");
    println!("    {}", conv.where_line(raw.0));
    println!("    surface elevation : {:.1} m", full.grid.surf_at(raw.0));
    println!("    extra bedrock lost: {:.2} m of R stripped", raw.1);
    println!(
        "    local relief (±2) : {:.1} m",
        local_relief(full, raw.0, 2)
    );
    println!(
        "  SEE: a frost-shattered cold band producing extra regolith/scree on the flanks;\n\
         \x20      knobs `frost_weathering_gain` (peak rate) and `frost_band_width_c` (band)."
    );
}

fn station_wave(off: &DeepRun, wave: &DeepRun, full: &DeepRun, conv: &Conv) {
    // Strongest littoral retreat: max Δsurf (off→wave) on a coast cell (low
    // freeboard, adjacent to open water). Report the global max and the strongest
    // near the 0046 scarp when it is comparable (≥50% of the global max).
    let band = DeepConfig::default().wave_band_m;
    let w = off.grid.w as i64;
    let coast = |i: usize| -> bool {
        let free = off.grid.surf_at(i);
        if !(free > 0.0 && free <= band) {
            return false;
        }
        let (cx, cy) = ((i as i64 % w), (i as i64 / w));
        for (dx, dy) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
            let (x, y) = (cx + dx, cy + dy);
            if x >= 0 && y >= 0 && x < w && y < w && off.grid.surf_at((y * w + x) as usize) <= 0.0 {
                return true;
            }
        }
        false
    };
    let mut best = (0usize, 0.0f64);
    let mut best_scarp = (0usize, 0.0f64);
    for i in 0..(w * w) as usize {
        if !coast(i) {
            continue;
        }
        let d = off.grid.surf_at(i) - wave.grid.surf_at(i);
        if d > best.1 {
            best = (i, d);
        }
        let (mx, mz) = conv.meters(i);
        let near_scarp = (mx - SCARP_XZ_M.0).hypot(mz - SCARP_XZ_M.1) <= 15_000.0;
        if near_scarp && d > best_scarp.1 {
            best_scarp = (i, d);
        }
    }
    header("STATION 5 — WAVE-CUT COAST");
    if best.1 < LEGIBLE_M {
        println!(
            "  NULL WARNING: strongest littoral retreat is only {:.3} m — at production\n\
             \x20   magnitudes this station shows essentially NOTHING a walker can read.\n\
             \x20   Reported anyway; a sub-metre cut over the whole run is a magnitude verdict.",
            best.1
        );
    }
    println!("  GLOBAL STRONGEST:");
    println!("    {}", conv.where_line(best.0));
    println!("    surface elevation : {:.1} m", full.grid.surf_at(best.0));
    println!(
        "    littoral lowering : {:.3} m cut toward sea level (off→wave)",
        best.1
    );
    if best_scarp.1 > 0.0 && best_scarp.1 >= 0.5 * best.1 {
        println!("  NEAR THE 0046 SCARP (comparable, ground you know):");
        println!("    {}", conv.where_line(best_scarp.0));
        println!(
            "    surface elevation : {:.1} m",
            full.grid.surf_at(best_scarp.0)
        );
        println!("    littoral lowering : {:.3} m", best_scarp.1);
    } else if best_scarp.1 > 0.0 {
        println!(
            "  (near the 0046 scarp the strongest cut is only {:.3} m — not comparable; use the global site.)",
            best_scarp.1
        );
    } else {
        println!("  (no wave-cut coast cell within 15 km of the 0046 scarp.)");
    }
    println!(
        "  SEE: a coast notched/planed toward the waterline over the sea-level cycles;\n\
         \x20      knobs `wave_erosion` (cut rate) and `wave_band_m` (freeboard reach)."
    );
}

fn header(title: &str) {
    println!("\n============================================================");
    println!("{title}");
    println!("============================================================");
}

fn null_note(what: &str, mag: f64) {
    println!(
        "  NULL WARNING: strongest {what} is only {mag:.2} m — this station may show\n\
         \x20   little at current magnitudes. Strongest available site reported anyway."
    );
}
