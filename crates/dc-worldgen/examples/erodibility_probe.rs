//! Erodibility-coupling measurement harness (journal/0029).
//!
//! Landform evidence, not adjectives: relief and slope statistics with the
//! coupling ON vs OFF on the same seed, elevation attributed to the lithology
//! actually outcropping at each cell, the basement/shield prediction, a printed
//! cross-section, and the cost delta on the **shipped** path (the parallel
//! driver `build_field` uses — corrections #12's warning).
//!
//! Corrections #15's corollary is honoured: every differencing measurement is
//! reported alongside the **control's residual drift** (an OFF↔OFF double run),
//! so a reader can see the noise floor the signal sits above. And a
//! *uniformly-weakened* control isolates differential erosion from "coupling
//! just erodes less overall".
//!
//! Run: `cargo run --release -p dc-worldgen --example erodibility_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{
    self, Agent, DeepConfig, DeepGrid, Litho, exposed_litho, susceptibility_table,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

/// Land-cell landform statistics.
struct Stats {
    land: usize,
    mean_elev: f64,
    relief: f64,
    mean_slope: f64,
    slope_sd: f64,
    p95_slope: f64,
    p99_slope: f64,
    steep_frac: f64,
}

fn slopes(grid: &DeepGrid) -> Vec<(usize, f64, f64)> {
    // (index, surface, max |Δsurf| / cell_m over the 4-neighbourhood)
    let w = grid.w;
    let mut out = Vec::with_capacity(w * w);
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let s = grid.surf_at(i);
            if s <= 0.0 {
                continue;
            }
            let mut worst = 0.0f64;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (gx as i32 + dx, gy as i32 + dy);
                if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= w {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                worst = worst.max((s - grid.surf_at(j)).abs());
            }
            out.push((i, s, worst / grid.cell_m));
        }
    }
    out
}

fn stats(grid: &DeepGrid) -> Stats {
    let sl = slopes(grid);
    let land = sl.len();
    if land == 0 {
        return Stats {
            land: 0,
            mean_elev: 0.0,
            relief: 0.0,
            mean_slope: 0.0,
            slope_sd: 0.0,
            p95_slope: 0.0,
            p99_slope: 0.0,
            steep_frac: 0.0,
        };
    }
    let n = land as f64;
    let mean_elev = sl.iter().map(|t| t.1).sum::<f64>() / n;
    let hi = sl.iter().map(|t| t.1).fold(f64::MIN, f64::max);
    let lo = sl.iter().map(|t| t.1).fold(f64::MAX, f64::min);
    let mean_slope = sl.iter().map(|t| t.2).sum::<f64>() / n;
    let var = sl.iter().map(|t| (t.2 - mean_slope).powi(2)).sum::<f64>() / n;
    let mut sorted: Vec<f64> = sl.iter().map(|t| t.2).collect();
    sorted.sort_by(f64::total_cmp);
    let pick = |q: f64| sorted[((n - 1.0) * q).round() as usize];
    let steep = sl.iter().filter(|t| t.2 > 2.0 * mean_slope).count() as f64;
    Stats {
        land,
        mean_elev,
        relief: hi - lo,
        mean_slope,
        slope_sd: var.sqrt(),
        p95_slope: pick(0.95),
        p99_slope: pick(0.99),
        steep_frac: steep / n,
    }
}

fn print_stats(label: &str, s: &Stats) {
    println!(
        "{label:<26} land {:>7}  mean_elev {:>8.1} m  relief {:>8.1} m  \
         slope mean {:.5} sd {:.5} p95 {:.5} p99 {:.5}  steep {:.1}%",
        s.land,
        s.mean_elev,
        s.relief,
        s.mean_slope,
        s.slope_sd,
        s.p95_slope,
        s.p99_slope,
        s.steep_frac * 100.0
    );
}

/// The lithology outcropping at each cell, computed from the record directly so
/// it can be read from an uncoupled run too (an OFF run has no exposure plane,
/// but its record still says what rock is at the top).
fn exposure(grid: &DeepGrid) -> Vec<Litho> {
    (0..grid.w * grid.w)
        .map(|i| exposed_litho(grid.strata.get(i).map_or(&[][..], |s| s.units.as_slice())))
        .collect()
}

/// Mean land-surface elevation and slope per outcropping lithology — the
/// signature of differential erosion. With coupling ON the resistant
/// lithologies should stand higher than the weak ones; with it OFF the
/// separation is whatever depositional accident produced, and should be much
/// smaller.
fn by_litho(grid: &DeepGrid, label: &str) {
    let ex = exposure(grid);
    let sl = slopes(grid);
    println!("  {label}");
    for l in Litho::ALL {
        let rows: Vec<&(usize, f64, f64)> = sl.iter().filter(|t| ex[t.0] == l).collect();
        if rows.is_empty() {
            println!("    {:<9} —", l.code());
            continue;
        }
        let n = rows.len() as f64;
        let e = rows.iter().map(|t| t.1).sum::<f64>() / n;
        let s = rows.iter().map(|t| t.2).sum::<f64>() / n;
        println!(
            "    {:<9} cells {:>7} ({:>5.2}%)  mean_elev {:>8.1} m  mean_slope {:.5}",
            l.code(),
            rows.len(),
            n / sl.len() as f64 * 100.0,
            e,
            s
        );
    }
}

/// The single most legible transect: the land row whose outcrop pattern has the
/// most lithologic alternation, printed cell by cell with ON and OFF surfaces.
fn cross_section(on: &DeepGrid, off: &DeepGrid) {
    let w = on.w;
    let ex_on = exposure(on);
    let mut best = (0usize, 0usize);
    for gy in 0..w {
        let mut changes = 0;
        let mut last: Option<Litho> = None;
        for gx in 0..w {
            let i = gy * w + gx;
            if on.surf_at(i) <= 0.0 {
                continue;
            }
            let l = ex_on[i];
            if last.is_some_and(|p| p != l) {
                changes += 1;
            }
            last = Some(l);
        }
        if changes > best.1 {
            best = (gy, changes);
        }
    }
    let gy = best.0;
    println!(
        "\n  CROSS-SECTION — deep-grid row {gy} ({} lithologic changes along it)",
        best.1
    );
    println!(
        "    {:>4}  {:<9} {:>10} {:>10} {:>9}",
        "x", "outcrop", "ON (m)", "OFF (m)", "Δ (m)"
    );
    let mut shown = 0;
    for gx in 0..w {
        let i = gy * w + gx;
        if on.surf_at(i) <= 0.0 {
            continue;
        }
        if shown >= 48 {
            println!("    … (truncated)");
            break;
        }
        println!(
            "    {gx:>4}  {:<9} {:>10.1} {:>10.1} {:>+9.1}",
            ex_on[i].code(),
            on.surf_at(i),
            off.surf_at(i),
            on.surf_at(i) - off.surf_at(i)
        );
        shown += 1;
    }
}

/// The clearest single pair: adjacent land cells outcropping different
/// lithologies, ranked by how much more the harder one stands proud with
/// coupling ON than it did with coupling OFF.
fn best_contrast_pair(on: &DeepGrid, off: &DeepGrid) {
    let w = on.w;
    let ex = exposure(on);
    let tab = susceptibility_table(Agent::Abrasion, 2.5, 5.0);
    let mut best: Option<(f64, usize, usize)> = None;
    // The same search restricted to contacts where the hard bed ends up
    // *standing above* its soft neighbour — the caprock/bench landform in its
    // most literal form.
    let mut proud: Option<(f64, usize, usize)> = None;
    for gy in 1..w - 1 {
        for gx in 1..w - 1 {
            let i = gy * w + gx;
            let j = gy * w + gx + 1;
            if on.surf_at(i) <= 0.0 || on.surf_at(j) <= 0.0 || ex[i] == ex[j] {
                continue;
            }
            // Orient so `a` is the more resistant (lower susceptibility) cell.
            let (a, b) = if tab[ex[i].index()] < tab[ex[j].index()] {
                (i, j)
            } else {
                (j, i)
            };
            let gain = (on.surf_at(a) - on.surf_at(b)) - (off.surf_at(a) - off.surf_at(b));
            if best.is_none_or(|(g, _, _)| gain > g) {
                best = Some((gain, a, b));
            }
            if on.surf_at(a) > on.surf_at(b) && proud.is_none_or(|(g, _, _)| gain > g) {
                proud = Some((gain, a, b));
            }
        }
    }
    let report = |title: &str, sel: Option<(f64, usize, usize)>| {
        if let Some((gain, a, b)) = sel {
            println!(
                "\n  {title} — the resistant cell gained {gain:.1} m relative to its neighbour\n    \
                 hard  cell {a:>7} {:<9} ON {:>8.1} m  OFF {:>8.1} m  ({:+.1} m)\n    \
                 soft  cell {b:>7} {:<9} ON {:>8.1} m  OFF {:>8.1} m  ({:+.1} m)\n    \
                 step across the contact (hard − soft):  ON {:>+8.1} m   OFF {:>+8.1} m",
                ex[a].code(),
                on.surf_at(a),
                off.surf_at(a),
                on.surf_at(a) - off.surf_at(a),
                ex[b].code(),
                on.surf_at(b),
                off.surf_at(b),
                on.surf_at(b) - off.surf_at(b),
                on.surf_at(a) - on.surf_at(b),
                off.surf_at(a) - off.surf_at(b),
            );
        }
    };
    report("SHARPEST CONTACT", best);
    report("HARD BED STANDING PROUD", proud);
}

fn drift(a: &DeepGrid, b: &DeepGrid) -> f64 {
    a.r.iter()
        .zip(&b.r)
        .zip(a.h.iter().zip(&b.h))
        .map(|((r1, r2), (h1, h2))| ((r1 + h1) - (r2 + h2)).abs())
        .fold(0.0, f64::max)
}

fn mean_abs_diff(a: &DeepGrid, b: &DeepGrid) -> f64 {
    let n = a.r.len() as f64;
    a.r.iter()
        .zip(&b.r)
        .zip(a.h.iter().zip(&b.h))
        .map(|((r1, r2), (h1, h2))| ((r1 + h1) - (r2 + h2)).abs())
        .sum::<f64>()
        / n
}

/// **How much lowering does erosion actually do?** Compare the final surface
/// against the counterfactual surface a cell would have if nothing eroded at all
/// (initial bedrock + all of its uplift). This is the budget any differential
/// erosion has to express itself within — and if it is small, no resistance
/// model of any strength can carve anything.
fn erosion_budget(pregen: &Pregen, cfg: &DeepConfig) {
    let initial = deeptime::build(pregen, cfg);
    let run = deeptime::run_with(pregen, cfg, true);
    let n = initial.r.len();
    let mut lowering: Vec<f64> = Vec::new();
    for i in 0..n {
        if run.grid.surf_at(i) <= 0.0 {
            continue;
        }
        let no_erosion = initial.r[i] + initial.uplift[i] * f64::from(cfg.iterations);
        lowering.push(no_erosion - run.grid.surf_at(i));
    }
    lowering.sort_by(f64::total_cmp);
    let ln = lowering.len() as f64;
    let pick = |q: f64| lowering[((ln - 1.0) * q).round() as usize];
    let mean_uplift = initial.uplift.iter().sum::<f64>() / n as f64 * f64::from(cfg.iterations);
    let mean_h = run.grid.h.iter().sum::<f64>() / n as f64;
    println!(
        "  net lowering vs a no-erosion counterfactual, land cells:\n    \
         p10 {:>8.2} m   p50 {:>8.2} m   p90 {:>8.2} m   max {:>9.2} m   mean {:>8.2} m",
        pick(0.10),
        pick(0.50),
        pick(0.90),
        pick(1.0),
        lowering.iter().sum::<f64>() / ln
    );
    println!(
        "    mean uplift delivered {mean_uplift:.1} m   mean cover H {mean_h:.2} m   \
         weathering ceiling {:.1} m ({} × {} m/epoch)",
        cfg.weathering * f64::from(cfg.iterations),
        cfg.iterations,
        cfg.weathering
    );
    println!(
        "    → erosion removes {:.2}% of what uplift delivers",
        lowering.iter().sum::<f64>() / ln / mean_uplift * 100.0
    );
}

/// One ON/OFF/uniform-control experiment at a given physics calibration.
fn experiment(
    label: &str,
    pregen: &Pregen,
    base: &DeepConfig,
    tab: &[f64; Litho::COUNT],
    ctab: &[f64; Litho::COUNT],
) {
    println!("\n================ {label} ================");
    let cfg_off = DeepConfig {
        erodibility: false,
        ..*base
    };
    let cfg_on = DeepConfig {
        erodibility: true,
        ..*base
    };
    println!("-- erosion budget at this calibration --");
    erosion_budget(pregen, &cfg_off);

    let off = deeptime::run_with(pregen, &cfg_off, true);
    let on = deeptime::run_with(pregen, &cfg_on, true);
    let off2 = deeptime::run_with(pregen, &cfg_off, true);
    println!(
        "\n  CONTROL residual drift (OFF vs OFF): max {:.3e} m, mean {:.3e} m  [noise floor]",
        drift(&off.grid, &off2.grid),
        mean_abs_diff(&off.grid, &off2.grid)
    );
    println!(
        "  TREATMENT surface change  (ON vs OFF): max {:.1} m, mean {:.2} m",
        drift(&on.grid, &off.grid),
        mean_abs_diff(&on.grid, &off.grid)
    );

    let ex_on = exposure(&on.grid);
    let land: Vec<usize> = (0..on.grid.w * on.grid.w)
        .filter(|&i| on.grid.surf_at(i) > 0.0)
        .collect();
    let mean_mult =
        land.iter().map(|&i| tab[ex_on[i].index()]).sum::<f64>() / land.len().max(1) as f64;
    let mean_cmult =
        land.iter().map(|&i| ctab[ex_on[i].index()]).sum::<f64>() / land.len().max(1) as f64;
    let uniform = deeptime::run_with(
        pregen,
        &DeepConfig {
            erodibility: false,
            k_transport: base.k_transport * mean_mult,
            k_bedrock: base.k_bedrock * mean_mult,
            weathering: base.weathering * mean_mult,
            diffusion: base.diffusion * mean_cmult,
            ..*base
        },
        true,
    );
    println!(
        "  (uniform control scales k/weathering by {mean_mult:.4}×, diffusion by {mean_cmult:.4}×)\n"
    );

    let s_off = stats(&off.grid);
    let s_on = stats(&on.grid);
    let s_uni = stats(&uniform.grid);
    print_stats("coupling OFF", &s_off);
    print_stats("coupling ON", &s_on);
    print_stats("uniformly-weakened OFF", &s_uni);
    println!(
        "\n  ON vs OFF     : relief {:+.1} m  mean_slope {:+.1}%  slope_sd {:+.1}%  \
         p99 {:+.1}%  steep {:+.2} pp",
        s_on.relief - s_off.relief,
        (s_on.mean_slope / s_off.mean_slope - 1.0) * 100.0,
        (s_on.slope_sd / s_off.slope_sd - 1.0) * 100.0,
        (s_on.p99_slope / s_off.p99_slope - 1.0) * 100.0,
        (s_on.steep_frac - s_off.steep_frac) * 100.0
    );
    println!(
        "  ON vs UNIFORM : relief {:+.1} m  mean_slope {:+.1}%  slope_sd {:+.1}%  \
         p99 {:+.1}%  steep {:+.2} pp   [isolates DIFFERENTIAL erosion]",
        s_on.relief - s_uni.relief,
        (s_on.mean_slope / s_uni.mean_slope - 1.0) * 100.0,
        (s_on.slope_sd / s_uni.slope_sd - 1.0) * 100.0,
        (s_on.p99_slope / s_uni.p99_slope - 1.0) * 100.0,
        (s_on.steep_frac - s_uni.steep_frac) * 100.0
    );

    println!("\n-- elevation by outcropping lithology --");
    by_litho(&off.grid, "coupling OFF");
    by_litho(&on.grid, "coupling ON");

    println!("\n-- the basement prediction: stripped columns should stand resistant --");
    for (l, g) in [("OFF", &off.grid), ("ON", &on.grid)] {
        let ex = exposure(g);
        let sl = slopes(g);
        let sel = |want: bool| -> (usize, f64, f64) {
            let v: Vec<&(usize, f64, f64)> = sl
                .iter()
                .filter(|t| (ex[t.0] == Litho::Basement) == want)
                .collect();
            if v.is_empty() {
                return (0, 0.0, 0.0);
            }
            let n = v.len() as f64;
            (
                v.len(),
                v.iter().map(|t| t.1).sum::<f64>() / n,
                v.iter().map(|t| t.2).sum::<f64>() / n,
            )
        };
        let (bn, be, bs) = sel(true);
        let (cn, ce, cs) = sel(false);
        println!(
            "  {l:<4} basement {bn:>6} cells mean {be:>8.1} m slope {bs:.5}   \
             covered {cn:>7} cells mean {ce:>8.1} m slope {cs:.5}   proud {:+.1} m",
            be - ce
        );
    }

    cross_section(&on.grid, &off.grid);
    best_contrast_pair(&on.grid, &off.grid);
}

fn main() {
    println!("=== erodibility coupling probe — seed {SEED:#x} ===\n");

    // ---- the susceptibility table itself ---------------------------------
    println!("-- agent-specific resistances (per reference member) --");
    println!(
        "  {:<9} {:<22} {:>9} {:>12} {:>10} {:>8}",
        "litho", "reference member", "abrasion", "dissolution", "frost/ice", "wave"
    );
    for l in Litho::ALL {
        let r = l.resistance();
        let d = if r.dissolution.is_finite() {
            format!("{:.2}", r.dissolution)
        } else {
            "immune".into()
        };
        println!(
            "  {:<9} {:<22} {:>9.3} {:>12} {:>10.3} {:>8.3}",
            l.code(),
            l.reference_material().props().name,
            r.abrasion,
            d,
            r.frost_ice,
            r.wave
        );
    }
    let cfg_default = DeepConfig::default();
    let tab = susceptibility_table(
        Agent::Abrasion,
        cfg_default.erodibility_contrast,
        cfg_default.erodibility_max,
    );
    let ctab = susceptibility_table(
        Agent::Abrasion,
        cfg_default.erodibility_diffusion_contrast,
        cfg_default.erodibility_max,
    );
    println!(
        "\n  abrasion rate multipliers (contrast {} fluvial / {} creep, clamp ±{}×):",
        cfg_default.erodibility_contrast,
        cfg_default.erodibility_diffusion_contrast,
        cfg_default.erodibility_max
    );
    for l in Litho::ALL {
        println!(
            "    {:<9} fluvial {:>6.3}×   creep {:>6.3}×",
            l.code(),
            tab[l.index()],
            ctab[l.index()]
        );
    }

    // ---- the landform experiments -----------------------------------------
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let shipped = DeepConfig {
        seed: SEED,
        cell_m: 460.0,
        iterations: 200,
        record: true,
        biotic: true,
        ..DeepConfig::default()
    };
    experiment(
        "A — the SHIPPED calibration (Medium, 460 m, 200 iterations, biology ON)",
        &pregen,
        &shipped,
        &tab,
        &ctab,
    );

    // The headroom experiment. If experiment A shows little landform change,
    // there are two possible reasons and they demand opposite responses: the
    // resistance model is wrong, or the erosion budget it modulates is too small
    // to express anything. Raising the whole erosion budget 10× — every rate
    // together, so nothing about the *relative* rates changes — separates them.
    // A model that produces landform contrast the moment erosion is allowed to
    // actually cut is a correct model waiting on an amplitude decision, not a
    // broken one.
    let headroom = DeepConfig {
        weathering: shipped.weathering * 10.0,
        k_transport: shipped.k_transport * 10.0,
        k_bedrock: shipped.k_bedrock * 10.0,
        ..shipped
    };
    experiment(
        "B — HEADROOM: the same coupling with a 10x erosion budget",
        &pregen,
        &headroom,
        &tab,
        &ctab,
    );

    // ---- cost on the SHIPPED path ----------------------------------------
    // corrections #12: measure the production driver, not a spike harness.
    // `build_field` = `run_cells(cells, production_config(..), parallel = true)`
    // plus a distil; timing `run_cells` on the parallel path with the production
    // config is the ritual's deep-time cost.
    println!("\n-- cost on the shipped path (parallel driver, production config) --");
    let cells: &CellGrid = &pregen.grid;
    let prod_off = DeepConfig {
        erodibility: false,
        ..deeptime::production_config(cells, SEED)
    };
    let prod_on = DeepConfig {
        erodibility: true,
        ..prod_off
    };
    for (label, cfg) in [("erodibility OFF", prod_off), ("erodibility ON", prod_on)] {
        let t = Instant::now();
        let run = deeptime::run_cells(cells, &cfg, true);
        let dt = t.elapsed();
        println!(
            "  {label:<16} {:>7.2} s   ({} cells @ {:.0} m, {} iterations, {} units recorded)",
            dt.as_secs_f64(),
            run.grid.w * run.grid.w,
            run.grid.cell_m,
            cfg.iterations,
            run.grid.total_units()
        );
    }
}
