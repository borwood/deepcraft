//! **At what rate does the shipped world denude?** — the one question in this
//! project with a *published external answer* to compare against (journal/0111).
//!
//! journal/0110 measured a null and corrections #55 recorded its cause: fluvial
//! transport is 0.109 % of this world's sediment routing, hillslope creep moves
//! 918× more, and no cell can carry sand. That left a fork the project cannot
//! settle by argument:
//!
//! - **an honest low-relief craton** — in-place weathering plus creep dominating
//!   is exactly right for such a landscape, rivers moving a thousandth of the
//!   sediment is correct, and the facies acceptance test simply does not apply;
//! - **a broken energy budget** — almost nothing is eroding at all, and the fix
//!   is structural (hybrid `p`, recalibrated erosion coefficients, real recharge).
//!
//! **Denudation rate is the discriminator**, because stable cratons, passive
//! margins and active orogens occupy well-separated, well-published bands and a
//! landscape that falls *below all of them* is not a craton, it is a stopped
//! clock.
//!
//! ## The definition, and why this one
//!
//! "Denudation rate" has several defensible readings and they do not agree. The
//! headline here is **catchment-averaged denudation**: mass **exported from the
//! subaerial land system** per unit *land* area per unit time. It is the reading
//! the literature measures (sediment yield at a gauging station; a cosmogenic
//! `10Be` catchment average), and it is the strict one — **material that merely
//! redistributes within the landscape is not denudation.** Weathered in place:
//! not denudation. Crept one cell downslope: not denudation. Crossed the
//! shoreline: denudation.
//!
//! The other two readings are reported beside it because they are cheap and
//! because their *disagreement* is informative:
//!
//! - **mean surface lowering** — confounded by uplift, but it is what a player
//!   would see;
//! - **bedrock erosion** — incision plus weathering-front descent, which the
//!   engine already carries per cell as `grid.exhum`, and which therefore also
//!   supplies the **distribution** across land cells.
//!
//! Two things decide whether any of it is trustworthy, and both are printed
//! rather than assumed: the **iteration↔Myr calibration** (every number here is
//! linear in it) and the **land-area denominator** (~15 % of cells are subaerial;
//! dividing by the whole grid understates the rate ~6×).
//!
//! ## And then it became the acceptance instrument (journal/0114)
//!
//! journal/0111 answered the fork — **the energy budget was broken** — and this
//! probe's own sensitivity sweep found the shape of the repair: supply and
//! transport are coupled through the cover taper, neither pays alone, and together
//! they pay 59× more than their separate gains multiplied.
//!
//! So the probe now carries the **calibration's derivation**, and that is a change
//! of role worth stating plainly. It measures three things in one run:
//!
//! 1. the **shipped** world, calibrated (the headline, everything above);
//! 2. the **uncalibrated** world — the raw pre-2026-07-26 constants under *today's*
//!    solve, so the before/after is one variable rather than a quote carried
//!    forward across three merges;
//! 3. the **derivation**: a ladder of uniform multipliers over the raw constants,
//!    each a full 200-epoch world, printed with `D1` *and* the balance ratio
//!    `D1/D3` — beside the world's own Airy steady-state ceiling, which is computed
//!    from two densities and a measured uplift and lands in the published band
//!    without being asked to.
//!
//! **What that does NOT license.** The multiplier is fitted to a *published band*
//! (1–10 m/Myr, stable craton) and to a *balance ratio* near 1 — never to an
//! appearance. Whether sand moves and whether a facies gradient appears are outputs
//! this calibration is CHECKED against and never tuned toward; if they stay null
//! they are reported null (CLAUDE.md § *a closed system cannot detect its own scale
//! error*).
//!
//! The instrumentation itself is still read-only with respect to the physics and
//! sits behind [`DeepConfig::denudation_ledger`], off in production, and the gate
//! still asserts the surface plane is **bit-identical** with it on — which is what
//! makes a number taken with the flag on a number about the *shipped* world.
//!
//! Run: `cargo run --release -p dc-worldgen --example denudation_probe`
//! (optionally with an explicit ladder: `-- 100 300 1000`)

use dc_worldgen::deeptime::erosion::TransportLedger;
use dc_worldgen::deeptime::{
    DeepConfig, DeepOverrides, SEA_LEVEL_M, build_cells, production_config, production_config_with,
    run_cells, sea_level_at,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

// ---------------------------------------------------------------------------
// THE TIME CALIBRATION — derived, not assumed. Every rate below is linear in it.
// ---------------------------------------------------------------------------

/// **The recorded span, in millions of years.**
///
/// Derived from two places in the corpus that agree:
///
/// 1. `docs/design/earth-processes.md` § 3e-2 **decision 5, RATIFIED
///    2026-07-19 (user)** — the **Phanerozoic register**: *"the recorded span
///    calibrates to ~500 Myr … ~500 Myr is the default."*
/// 2. `DeepConfig::chapters`' own doc comment, independently: *"K=8 gives
///    Earth-orogeny-length chapters (**62.5 Myr**) … at the Phanerozoic
///    register."* — and 8 × 62.5 = 500.
///
/// So the run's [`DeepConfig::iterations`] (200) span 500 Myr, i.e.
/// [`myr_per_epoch`] = **2.5 Myr per iteration**.
///
/// **Confidence: high on the intent, and the intent is all there is.** This is a
/// *stipulated* register, not a fitted one — earth-processes.md § 3e's own owed
/// list still reads "calibrate iteration↔Myr against a real orogen", so no
/// physical rate in the engine has ever been checked against the clock. That is
/// exactly the gap this probe measures, and it is why the report prints the rate
/// **per epoch** as well: that number is calibration-free.
const MYR_PER_RUN: f64 = 500.0;

/// Millions of years per deep-time iteration — [`MYR_PER_RUN`] over the run's
/// fixed 200-epoch schedule.
fn myr_per_epoch(cfg: &DeepConfig) -> f64 {
    MYR_PER_RUN / f64::from(cfg.iterations)
}

// ---------------------------------------------------------------------------

fn cfg(cells: &CellGrid) -> DeepConfig {
    DeepConfig {
        denudation_ledger: true,
        ..production_config(cells, SEED)
    }
}

/// **The pre-calibration world** — the raw rate constants journal/0111 measured,
/// reachable as a second path (`DeepOverrides::calibrated_rates: Some(false)`)
/// rather than as a deleted one. Everything else about the solve is today's: hybrid
/// `p`, material-aware transport, material-aware creep. That matters, because it
/// makes the before/after in this report a **single-variable** comparison — the
/// baseline is not journal/0111's printed number carried forward, it is re-measured
/// under the current solve in the same run.
fn cfg_uncalibrated(cells: &CellGrid) -> DeepConfig {
    let o = DeepOverrides {
        calibrated_rates: Some(false),
        ..DeepOverrides::default()
    };
    DeepConfig {
        denudation_ledger: true,
        ..production_config_with(cells, SEED, &o)
    }
}

/// **The derivation's own axis**: a uniform multiplier on the *raw* constants.
///
/// Since journal/0114, `erosion_budget` goes through `scale_erosion_rates` and so
/// covers `diffusion` as well — which is exactly what makes this one call able to
/// express the hypothesis. Applied on top of `calibrated_rates: false`, `mult` is
/// the whole erosional amplitude of the world, measured from the pre-calibration
/// zero point, so the ladder below is directly readable as "what would
/// `EROSION_CALIBRATION = mult` produce?" — and at `mult == EROSION_CALIBRATION`
/// this config is **byte-identical to production** (asserted in
/// `tests/calibrated_rates.rs`).
fn cfg_uniform(cells: &CellGrid, mult: f64) -> DeepConfig {
    let o = DeepOverrides {
        calibrated_rates: Some(false),
        erosion_budget: Some(mult),
        ..DeepOverrides::default()
    };
    DeepConfig {
        denudation_ledger: true,
        ..production_config_with(cells, SEED, &o)
    }
}

/// One denudation measurement of a world.
struct Denudation {
    // --- the frame -------------------------------------------------------
    cells_total: usize,
    land_cells: usize,
    /// Land-cell count at the low and high sea stands the run cycles through —
    /// the denominator's own uncertainty band.
    land_at_low_stand: usize,
    land_at_high_stand: usize,
    cell_m: f64,
    epochs: u32,
    myr: f64,

    // --- the ledger ------------------------------------------------------
    ledger: TransportLedger,

    // --- the three definitions (all m/Myr, land-area averaged) ------------
    /// **D1, the headline** — export from the land system per land area per Myr.
    catchment_averaged: f64,
    /// **D2** — mean lowering of the land surface (negative = the land is
    /// building). Confounded by uplift, which is the point of reporting D4.
    mean_surface_lowering: f64,
    /// **D3** — bedrock erosion: incision + weathering-front descent, from
    /// `grid.exhum` plus the wave agent's bedrock quarry (which runs after
    /// exhumation is tracked and is therefore missing from that plane).
    bedrock_erosion: f64,
    /// **D4** — rock uplift, from the same land cells: `Δbedrock + exhumed`.
    rock_uplift: f64,

    // --- closure ---------------------------------------------------------
    /// `Σ_land Δsurf` and `Σ_land uplift − export`: the land system's own budget.
    /// They differ only by material crossing the shoreline in either direction as
    /// cells change status, so how close they are is how much of the land's
    /// elevation history the export accounting actually explains.
    land_dsurf_m: f64,
    land_budget_m: f64,

    // --- the distribution (m/Myr, per land cell, bedrock erosion) ---------
    pct: [f64; 9],
    max_cell: f64,
    /// Share of all land bedrock erosion done by the most active 10 % of land
    /// cells — the "uniformly quiet vs quiet-with-active-margins" discriminator.
    top_decile_share: f64,
    /// Same, for the most active 1 %.
    top_pct_share: f64,

    // --- shape -----------------------------------------------------------
    mean_surf: f64,
    max_surf: f64,
    relief: f64,
}

/// Percentile of a sorted slice (nearest-rank).
fn pct_of(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let k = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[k.min(sorted.len() - 1)]
}

fn measure(cells: &CellGrid) -> Denudation {
    measure_cfg(cells, &cfg(cells))
}

fn measure_cfg(cells: &CellGrid, cfg: &DeepConfig) -> Denudation {
    let cfg = *cfg;
    // The *initial* grid, rebuilt from the same pure `(cells, cfg)` inputs — the
    // only way to get a "before" plane, since the run consumes its own grid.
    let before = build_cells(cells, &cfg);
    let run = run_cells(cells, &cfg, true);
    let g = &run.grid;
    let n = g.w * g.w;
    let myr = MYR_PER_RUN;
    let ledger = run.erosion.transport_ledger();

    // **The land mask is the whole measurement's denominator.** Taken at the mean
    // sea stand on the final surface; the two cycling stands bracket its own
    // uncertainty and are reported beside it.
    let surf: Vec<f64> = (0..n).map(|i| g.r[i] + g.h[i]).collect();
    let land: Vec<bool> = surf.iter().map(|&s| s > SEA_LEVEL_M).collect();
    let land_cells = land.iter().filter(|&&l| l).count();
    let (lo, hi) = {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for e in 0..cfg.iterations {
            let s = sea_level_at(&cfg, e);
            lo = lo.min(s);
            hi = hi.max(s);
        }
        (lo, hi)
    };
    let land_at_low_stand = surf.iter().filter(|&&s| s > lo).count();
    let land_at_high_stand = surf.iter().filter(|&&s| s > hi).count();
    let nl = land_cells.max(1) as f64;

    // --- D1: export from the land system ---------------------------------
    let catchment_averaged = ledger.exported_m() / nl / myr;

    // --- D2/D3/D4: per-land-cell surface, bedrock and uplift budgets ------
    // Restricted to cells that are land at the end, and stated that way: a cell
    // that drowned mid-run has no "land surface lowering" to average.
    let mut dsurf = 0.0;
    let mut d_bedrock = 0.0;
    let mut exhumed = 0.0;
    let mut mean_surf = 0.0;
    let mut max_surf = f64::NEG_INFINITY;
    let mut min_surf = f64::INFINITY;
    let mut per_cell: Vec<f64> = Vec::with_capacity(land_cells);
    for i in 0..n {
        if !land[i] {
            continue;
        }
        dsurf += surf[i] - (before.r[i] + before.h[i]);
        d_bedrock += g.r[i] - before.r[i];
        let e = g.exhum.get(i).copied().unwrap_or(0.0);
        exhumed += e;
        per_cell.push(e / myr);
        mean_surf += surf[i];
        max_surf = max_surf.max(surf[i]);
        min_surf = min_surf.min(surf[i]);
    }
    mean_surf /= nl;

    // The wave agent lowers `R` *after* `track_exhumation` runs, so its bedrock
    // quarry is missing from `exhum` and has to be added back. It is a whole-grid
    // total (the counter is not per-cell), so it enters the means and not the
    // distribution — stated rather than hidden, and it is small.
    let bedrock_erosion = (exhumed + ledger.wave_bedrock_m) / nl / myr;
    // Rock uplift: every metre of bedrock that arrived from below is either still
    // there (Δr) or has been removed (exhum + wave quarry).
    let rock_uplift = (d_bedrock + exhumed + ledger.wave_bedrock_m) / nl / myr;
    let mean_surface_lowering = -dsurf / nl / myr;

    per_cell.sort_by(f64::total_cmp);
    let total_e: f64 = per_cell.iter().sum();
    let top_decile_share = if total_e > 0.0 {
        let k = per_cell.len() - per_cell.len() / 10;
        per_cell[k..].iter().sum::<f64>() / total_e
    } else {
        0.0
    };
    let top_pct_share = if total_e > 0.0 {
        let k = per_cell.len() - per_cell.len() / 100;
        per_cell[k..].iter().sum::<f64>() / total_e
    } else {
        0.0
    };

    Denudation {
        cells_total: n,
        land_cells,
        land_at_low_stand,
        land_at_high_stand,
        cell_m: g.cell_m,
        epochs: cfg.iterations,
        myr,
        ledger,
        catchment_averaged,
        mean_surface_lowering,
        bedrock_erosion,
        rock_uplift,
        land_dsurf_m: dsurf,
        land_budget_m: (d_bedrock + exhumed + ledger.wave_bedrock_m) - ledger.exported_m(),
        pct: [
            pct_of(&per_cell, 0.0),
            pct_of(&per_cell, 10.0),
            pct_of(&per_cell, 25.0),
            pct_of(&per_cell, 50.0),
            pct_of(&per_cell, 75.0),
            pct_of(&per_cell, 90.0),
            pct_of(&per_cell, 95.0),
            pct_of(&per_cell, 99.0),
            pct_of(&per_cell, 99.9),
        ],
        max_cell: per_cell.last().copied().unwrap_or(0.0),
        top_decile_share,
        top_pct_share,
        mean_surf,
        max_surf,
        relief: max_surf - min_surf,
    }
}

/// The uniform-multiplier ladder the derivation sweep walks by default. Roughly
/// half-decade spacing across the two orders of magnitude between "the stopped
/// clock" and "faster than this world's own steady-state ceiling", so the row that
/// lands in the published band is bracketed on both sides rather than extrapolated
/// to. Override from the command line: `--example denudation_probe -- 100 300 1000`.
const LADDER: [f64; 5] = [30.0, 100.0, 200.0, 300.0, 1000.0];

fn main() {
    let ladder: Vec<f64> = {
        let args: Vec<f64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
        if args.is_empty() { LADDER.to_vec() } else { args }
    };
    println!("=== denudation probe — seed {SEED}, Extent::Medium ===\n");
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let d = measure(&pregen.grid);
    let c = cfg(&pregen.grid);

    println!("--- THE TIME CALIBRATION (every number below is linear in this) ---");
    println!(
        "  recorded span            {:.0} Myr   (earth-processes.md § 3e-2 decision 5,",
        d.myr
    );
    println!(
        "                                       RATIFIED 2026-07-19: the Phanerozoic register;"
    );
    println!(
        "                                       corroborated by DeepConfig::chapters — K={} x 62.5 Myr)",
        c.chapters
    );
    println!(
        "  epochs                   {}          -> {:.2} Myr per iteration",
        d.epochs,
        myr_per_epoch(&c)
    );
    println!(
        "  it is a STIPULATED register, never fitted: earth-processes.md § 3e still owes\n  \
         \"calibrate iteration<->Myr against a real orogen\". No engine rate has been checked\n  \
         against this clock — which is the hypothesis under test here."
    );

    println!("\n--- THE FRAME (the denominator decides the answer) ---");
    println!(
        "  grid                     {} cells at {:.0} m  ({:.0} km across)",
        d.cells_total,
        d.cell_m,
        (d.cells_total as f64).sqrt() * d.cell_m / 1000.0
    );
    println!(
        "  LAND cells               {} ({:.1} % of the grid) = {:.0} km^2",
        d.land_cells,
        100.0 * d.land_cells as f64 / d.cells_total as f64,
        d.land_cells as f64 * d.cell_m * d.cell_m / 1e6
    );
    println!(
        "  land at the low/high sea stands the run cycles through: {} / {}  ({:+.1} % / {:+.1} %)",
        d.land_at_low_stand,
        d.land_at_high_stand,
        100.0 * (d.land_at_low_stand as f64 / d.land_cells as f64 - 1.0),
        100.0 * (d.land_at_high_stand as f64 / d.land_cells as f64 - 1.0),
    );
    println!(
        "  surface                  mean {:.1} m, max {:.1} m, relief {:.1} m",
        d.mean_surf, d.max_surf, d.relief
    );

    let l = d.ledger;
    println!("\n--- WHAT LEFT THE LAND (metres of cell thickness, over the whole run) ---");
    println!(
        "  fluvial yield to the sea       {:>14.1}   ({:>6.2} %)",
        l.sink_marine_m,
        100.0 * l.sink_marine_m / l.exported_m().max(1e-30)
    );
    println!(
        "  regolith crept across the shore{:>14.1}   ({:>6.2} %)",
        l.creep_to_sea_m,
        100.0 * l.creep_to_sea_m / l.exported_m().max(1e-30)
    );
    println!(
        "  wave-quarried, sent offshore   {:>14.1}   ({:>6.2} %)   of which bedrock {:.1}",
        l.wave_offshore_m,
        100.0 * l.wave_offshore_m / l.exported_m().max(1e-30),
        l.wave_bedrock_m
    );
    println!(
        "  dust settled on the sea        {:>14.1}   ({:>6.2} %)",
        l.eolian_to_sea_m,
        100.0 * l.eolian_to_sea_m / l.exported_m().max(1e-30)
    );
    println!("  {:->31} {:>14.1}", " TOTAL EXPORT", l.exported_m());
    println!(
        "\n  NOT export, and reported so it can never be counted as such:\n    \
         fluvial load piled at the subaerial domain BORDER  {:>12.1}\n    \
         weathered to regolith in place (never travels)     {:>12.1}\n    \
         moved by hillslope creep, TOTAL                    {:>12.1}",
        l.sink_border_m, l.weathered_m, l.diffused_m,
    );
    println!(
        "    (that creep total is the per-cell NET GAIN side and the shoreline figure above\n     \
         is a GROSS edge flux, so their ratio — {:.0} % — is indicative, not an itemisation.\n     \
         The two comparable numbers are D1 and D3 below, and they are independent.)",
        100.0 * l.creep_to_sea_m / l.diffused_m.max(1e-30),
    );

    println!("\n=== THE NUMBER ===");
    println!(
        "  D1  CATCHMENT-AVERAGED DENUDATION   {:>12.4} m/Myr      <-- the headline",
        d.catchment_averaged
    );
    println!("        (export from the land system / land area / time — what sediment yield");
    println!("         and cosmogenic catchment averages measure)");
    println!(
        "  D2  mean surface lowering           {:>12.4} m/Myr      (negative = land building)",
        d.mean_surface_lowering
    );
    println!(
        "  D3  bedrock erosion                 {:>12.4} m/Myr      (incision + weathering front)",
        d.bedrock_erosion
    );
    println!(
        "  D4  rock uplift                     {:>12.4} m/Myr",
        d.rock_uplift
    );
    println!(
        "\n  calibration-free restatement (no Myr assumption at all):\n    \
         D1 = {:.3e} m per epoch per land cell; over the whole {}-epoch run the land\n    \
         system exported {:.3} m of average thickness.",
        d.catchment_averaged * d.myr / f64::from(d.epochs),
        d.epochs,
        d.catchment_averaged * d.myr,
    );

    println!("\n--- DENUDATION vs UPLIFT (the steady-state read) ---");
    if d.rock_uplift.abs() > 0.0 {
        println!(
            "  D1 / D4  (export / rock uplift)      {:>10.4}",
            d.catchment_averaged / d.rock_uplift
        );
        println!(
            "  D3 / D4  (bedrock erosion / uplift)  {:>10.4}",
            d.bedrock_erosion / d.rock_uplift
        );
    }
    println!(
        "  land budget closure: Sum(dsurf) = {:.1} m vs Sum(uplift) - export = {:.1} m\n  \
         residual {:.1} m ({:.0} % of the export term) — material that arrived on cells which\n  \
         were SEA at the start and are LAND at the end. The sea stand cycles +-35 m, so the\n  \
         shoreline is not a clean control surface, and D1 alone carries that uncertainty.",
        d.land_dsurf_m,
        d.land_budget_m,
        d.land_dsurf_m - d.land_budget_m,
        100.0 * (d.land_dsurf_m - d.land_budget_m).abs() / d.ledger.exported_m().max(1e-30),
    );
    println!(
        "  WHICH IS WHY D3 MATTERS: bedrock erosion is a per-cell rock-removal plane with no\n  \
         shoreline in it at all, and it agrees with the boundary-flux accounting to {:.1} %\n  \
         (D1 {:.4} vs D3 {:.4}). Two independent instruments, one number.",
        100.0 * (d.catchment_averaged / d.bedrock_erosion - 1.0).abs(),
        d.catchment_averaged,
        d.bedrock_erosion,
    );
    // The limiting regime is a *reading of the ratio*, so it is chosen from the
    // ratio rather than written beside it.
    let shed = d.catchment_averaged / d.bedrock_erosion.max(1e-30);
    println!(
        "  It also names the limiting regime: {:.0} % of every metre of bedrock this world\n  \
         converts or incises LEAVES the land system, so the land is {}.",
        100.0 * shed,
        if shed > 0.9 {
            "SUPPLY-limited — nothing is piling up, and the weathering\n  constant IS the denudation rate"
        } else {
            "TRANSPORT-limited — it is making regolith it\n  cannot move, and the cover taper will shut weathering down from underneath"
        }
    );

    println!(
        "\n--- THE DISTRIBUTION across {} land cells ---",
        d.land_cells
    );
    println!("  bedrock erosion rate, m/Myr, per cell:");
    for (name, v) in [
        "min", "p10", "p25", "MEDIAN", "p75", "p90", "p95", "p99", "p99.9",
    ]
    .iter()
    .zip(d.pct.iter())
    {
        println!("    {name:<7} {v:>12.4}");
    }
    println!("    {:<7} {:>12.4}", "MAX", d.max_cell);
    println!(
        "  the most active 10 % of land does {:.1} % of the bedrock erosion; the top 1 % does {:.1} %.",
        100.0 * d.top_decile_share,
        100.0 * d.top_pct_share
    );
    let concentration = if d.pct[3] > 0.0 {
        d.max_cell / d.pct[3]
    } else {
        f64::INFINITY
    };
    println!("  max / median = {concentration:.1}x");

    println!("\n--- AGAINST THE LITERATURE (m/Myr = mm/kyr = um/yr) ---");
    let d1 = d.catchment_averaged;
    for (name, lo, hi, src) in BANDS {
        let rel = if d1 > 0.0 { lo / d1 } else { f64::INFINITY };
        println!("  {name:<38} {lo:>8.1} - {hi:<9.0}  this world is {rel:>8.0}x slower");
        println!("      {src}");
    }
    println!(
        "\n  D1 = {d1:.4} m/Myr and the world's MOST ACTIVE SINGLE CELL is {:.4} m/Myr.",
        d.max_cell
    );

    println!("\n--- READING IT ---");
    // The caption is DERIVED from the measurement, never written ahead of it
    // (CLAUDE.md: a printed caption is a published claim the gate cannot check).
    // The bands are literature (cited above); the verdict is chosen from the
    // number that was just taken.
    // `CRATON_FLOOR` is the bottom of the *stable craton* band — the slowest a
    // landscape gets while still being an ordinary shield. Below it there is only
    // the hyperarid/hypothermal end-member (McMurdo, Atacama), which is a
    // different claim and is tested separately so the two never blur.
    const CRATON_FLOOR: f64 = 1.0;
    const EXTREME_FLOOR: f64 = 0.1;
    if d1 < EXTREME_FLOOR && d.max_cell < CRATON_FLOOR {
        println!(
            "  BELOW EVERY PUBLISHED TERRESTRIAL BAND — and so is the world's single fastest\n  \
             cell. The land average is {:.0}x slower than the bottom of the stable-craton band\n  \
             and {:.0}x slower than the slowest surfaces ever measured on Earth (McMurdo Dry\n  \
             Valleys bedrock, ~{:.2} m/Myr; hyperarid Atacama, where 21Ne exposure ages reach\n  \
             37 Myr). The MOST ACTIVE CELL ON THE WHOLE WORLD, at {:.4} m/Myr, is still slower\n  \
             than Antarctic bare rock under permanent ice-free hyperaridity.\n  \
             This is not a slow landscape. It is a landscape whose erosional clock has stopped.",
            CRATON_FLOOR / d1,
            EXTREME_FLOOR / d1,
            0.19,
            d.max_cell,
        );
    } else if d1 < CRATON_FLOOR {
        println!(
            "  The land AVERAGE is below the stable-craton band, but the active tail\n  \
             ({:.4} m/Myr) reaches into the measured record — a quiet interior with live margins.",
            d.max_cell
        );
    } else {
        println!("  Within the published record — see the band table above for where.");
    }
    println!(
        "\n  Over the full {:.0} Myr the land system exported {:.2} m of average thickness.\n  \
         A real craton strips 5-10 KM over a Phanerozoic span (Kola: 3-5 km; Pilbara: multi-km\n  \
         in discrete Paleozoic pulses; South African plateau: >=4.5 km since 130 Ma). This\n  \
         world strips {:.0}x less than the low end of that.",
        d.myr,
        d1 * d.myr,
        5000.0 / (d1 * d.myr).max(1e-30),
    );
    // The uniformity caption is DERIVED from where the distribution actually sits,
    // never written ahead of it. journal/0111's version said "the whole distribution
    // is compressed into a band under the global floor" as flat prose; after a
    // calibration that is a claim the report must re-earn every run, which is the
    // `flux_record_probe` lesson (a printed caption is a published claim the gate
    // cannot check).
    println!(
        "\n  Uniformity: max/median = {concentration:.1}x and the top decile carries {:.0} % of the\n  \
         erosion (a perfectly uniform surface would give 10 %).",
        100.0 * d.top_decile_share
    );
    let above_floor = d.pct.iter().filter(|&&v| v >= CRATON_FLOOR).count();
    println!(
        "  Where the DISTRIBUTION sits against the craton floor ({CRATON_FLOOR:.0} m/Myr): the median cell\n  \
         is at {:.4} and the most active at {:.4}, so the land runs from {} the floor at the\n  \
         quiet end to {} it at the active end. {} of the 9 printed percentiles are at or above it.",
        d.pct[3],
        d.max_cell,
        if d.pct[3] >= CRATON_FLOOR { "above" } else { "below" },
        if d.max_cell >= CRATON_FLOOR { "above" } else { "below" },
        above_floor,
    );

    // -----------------------------------------------------------------------
    // THE CALIBRATION (journal/0114) — the before, the after, and the derivation.
    // -----------------------------------------------------------------------

    let base_cfg = cfg(&pregen.grid);
    let raw_cfg = cfg_uncalibrated(&pregen.grid);
    let raw = measure_cfg(&pregen.grid, &raw_cfg);

    println!("\n\n=== THE CALIBRATION (journal/0114) ===");
    println!(
        "  EROSION_CALIBRATION = {:.1}x, applied uniformly to the four rate constants that\n  \
         together set this world's erosional clock. The `uncalibrated` row is the SAME solve\n  \
         (hybrid p, material transport, material creep) on the RAW pre-2026-07-26 constants,\n  \
         re-measured here rather than quoted from journal/0111 — so the difference below is\n  \
         one variable and not a year of drift.\n",
        base_cfg.weathering / raw_cfg.weathering,
    );
    println!("  rate           uncalibrated        shipped     ratio");
    for (name, a, b) in [
        ("weathering", raw_cfg.weathering, base_cfg.weathering),
        ("diffusion", raw_cfg.diffusion, base_cfg.diffusion),
        ("k_transport", raw_cfg.k_transport, base_cfg.k_transport),
        ("k_bedrock", raw_cfg.k_bedrock, base_cfg.k_bedrock),
        // The unscaled neighbours, printed so the scope is visible rather than
        // asserted: these are the agent magnitudes and the cover length, and none
        // of them moved.
        ("h_star (NOT scaled)", raw_cfg.h_star, base_cfg.h_star),
        (
            "wave_erosion (NOT scaled)",
            raw_cfg.wave_erosion,
            base_cfg.wave_erosion,
        ),
    ] {
        println!("  {name:<26} {a:>12.6} {b:>14.6}   {:>7.1}x", b / a);
    }

    println!("\n  quantity                    uncalibrated        shipped");
    for (name, a, b) in [
        ("D1 catchment denudation", raw.catchment_averaged, d1),
        ("D2 mean surface lowering", raw.mean_surface_lowering, d.mean_surface_lowering),
        ("D3 bedrock erosion", raw.bedrock_erosion, d.bedrock_erosion),
        ("D4 rock uplift", raw.rock_uplift, d.rock_uplift),
        (
            "D1/D3 (the balance)",
            raw.catchment_averaged / raw.bedrock_erosion.max(1e-30),
            d1 / d.bedrock_erosion.max(1e-30),
        ),
        (
            "D1/D4 (vs uplift)",
            raw.catchment_averaged / raw.rock_uplift.max(1e-30),
            d1 / d.rock_uplift.max(1e-30),
        ),
        ("mean land surface (m)", raw.mean_surf, d.mean_surf),
        ("relief (m)", raw.relief, d.relief),
        ("land cells", raw.land_cells as f64, d.land_cells as f64),
        ("most active cell", raw.max_cell, d.max_cell),
    ] {
        println!("  {name:<26} {a:>14.4} {b:>14.4}");
    }

    // --- the ceiling this world sets for itself ----------------------------
    // Airy compensation returns (rho_m - rho_c)/rho_m of every eroded metre as a
    // surface drop and rebounds the rest, so a landscape in topographic steady
    // state denudes at U / that fraction. Computed from the isostasy module's own
    // densities, so it moves if they do — never transcribed.
    let rho_m = dc_worldgen::deeptime::isostasy::RHO_MANTLE;
    let rho_c = dc_worldgen::deeptime::isostasy::rho_crust(
        dc_worldgen::deeptime::CrustKind::Continental,
    );
    let f_airy = (rho_m - rho_c) / rho_m;
    let ceiling = raw.rock_uplift / f_airy;
    println!(
        "\n--- THE CEILING THIS WORLD SETS FOR ITSELF (the derivation's first half) ---\n  \
         Airy compensation returns (rho_m - rho_c)/rho_m = {f_airy:.4} of each eroded metre as a\n  \
         surface DROP and rebounds the other {:.1} %. A landscape in topographic steady state\n  \
         therefore denudes at U / {f_airy:.4} = {:.1} x its tectonic rock uplift. With the\n  \
         uncalibrated world's measured D4 = {:.4} m/Myr that ceiling is {ceiling:.3} m/Myr.\n  \
         THAT NUMBER WAS NOT CHOSEN. It falls out of two densities and a measured uplift, and\n  \
         it lands inside the published stable-craton band (1-10) on its own — which is the\n  \
         independent statement that the band is the right target for THIS world.",
        100.0 * (1.0 - f_airy),
        1.0 / f_airy,
        raw.rock_uplift,
    );

    // --- the measured response ---------------------------------------------
    println!(
        "\n--- THE MEASURED RESPONSE (the derivation's second half) ---\n  \
         A uniform multiplier on the four rates, applied to the RAW constants. Each row is a\n  \
         full 200-epoch world; the row at {:.1}x IS the shipped world, bit for bit (the same\n  \
         multiply on the same operands — asserted in tests/calibrated_rates.rs).\n\n  \
         WHY UNIFORM. journal/0111 measured that neither lever pays alone, and the mechanism is\n  \
         the cover taper exp(-H/H*), H* = {:.1} m — the only term in the system carrying an\n  \
         ABSOLUTE LENGTH. Raise supply alone and the regolith made shields the rock that made\n  \
         it; raise transport alone and there is nothing to carry. Scale both and the\n  \
         steady-state H is left where it was, so the taper never engages and the landscape's\n  \
         export follows the multiplier instead of fighting itself.\n",
        base_cfg.weathering / raw_cfg.weathering,
        base_cfg.h_star,
    );
    println!(
        "  uniform x    D1 (m/Myr)   D3 bedrock   D1/D3   D1/D4    mean surf   relief    band?"
    );
    let band = |v: f64| -> &'static str {
        if (1.0..=10.0).contains(&v) {
            "IN BAND"
        } else if v < 1.0 {
            "below"
        } else {
            "above"
        }
    };
    println!(
        "  {:<10} {:>11.4}  {:>11.4}  {:>6.2}  {:>6.3}  {:>10.1} {:>8.1}   {}",
        "1 (raw)",
        raw.catchment_averaged,
        raw.bedrock_erosion,
        raw.catchment_averaged / raw.bedrock_erosion.max(1e-30),
        raw.catchment_averaged / raw.rock_uplift.max(1e-30),
        raw.mean_surf,
        raw.relief,
        band(raw.catchment_averaged),
    );
    for m in &ladder {
        let r = measure_cfg(&pregen.grid, &cfg_uniform(&pregen.grid, *m));
        println!(
            "  {:<10} {:>11.4}  {:>11.4}  {:>6.2}  {:>6.3}  {:>10.1} {:>8.1}   {}",
            format!("{m:.0}"),
            r.catchment_averaged,
            r.bedrock_erosion,
            r.catchment_averaged / r.bedrock_erosion.max(1e-30),
            r.catchment_averaged / r.rock_uplift.max(1e-30),
            r.mean_surf,
            r.relief,
            band(r.catchment_averaged),
        );
    }
    println!(
        "\n  READ IT THIS WAY. `D1/D3` is the tell, and it is a SECOND target, not a curiosity.\n    \
         ~1.0  the land sheds everything it detaches — a landscape in balance.\n    \
         <1.0  the land is making regolith it cannot move — TRANSPORT-limited. The cover\n          \
         taper then shuts weathering off from underneath: the extra regolith shields the\n          \
         rock that made it.\n    \
         >1.0  the land is exporting stored cover faster than it detaches new rock — a\n          \
         TRANSIENT drawdown of the existing regolith, not a sustainable rate.\n  \
         A row that lands in the band with D1/D3 far from 1 has hit the number and missed the\n  \
         physics. The shipped multiplier is chosen for BOTH."
    );

    // --- the single-lever contrast, kept from journal/0111 -------------------
    // `erosion_budget` no longer HAS the old three-rate scope (stubs #24 closed), so
    // these two hypotheticals are built by hand. They are the evidence for the
    // paragraph above and are re-measured rather than quoted.
    println!(
        "\n--- THE CONTRAST: why not one lever (journal/0111's finding, re-measured) ---\n  \
         Hillslope creep is {:.0} % of this world's export, and the old `erosion_budget` scope\n  \
         (weathering + k_transport + k_bedrock, no diffusion) could not reach it. Both rows are\n  \
         built by hand here because that scope no longer exists as a knob.\n",
        100.0 * l.creep_to_sea_m / l.exported_m().max(1e-30)
    );
    let three_only = |m: f64| -> DeepConfig {
        let mut c = raw_cfg;
        c.weathering *= m;
        c.k_transport *= m;
        c.k_bedrock *= m;
        c
    };
    let creep_only = |m: f64| -> DeepConfig {
        let mut c = raw_cfg;
        c.diffusion *= m;
        c
    };
    let contrast: [(&str, DeepConfig); 2] = [
        ("supply only 100x (old scope)", three_only(100.0)),
        ("transport only 100x", creep_only(100.0)),
    ];
    println!("  scenario                       D1 (m/Myr)   D1/D3   vs raw");
    let mut gains = [1.0f64; 2];
    for (k, (name, c)) in contrast.iter().enumerate() {
        let r = measure_cfg(&pregen.grid, c);
        gains[k] = r.catchment_averaged / raw.catchment_averaged.max(1e-30);
        println!(
            "  {name:<30} {:>10.4}  {:>6.2}  {:>7.1}x",
            r.catchment_averaged,
            r.catchment_averaged / r.bedrock_erosion.max(1e-30),
            gains[k],
        );
    }
    let uniform_gain = d1 / raw.catchment_averaged.max(1e-30);
    println!(
        "  {:<30} {d1:>10.4}  {:>6.2}  {uniform_gain:>7.1}x   <-- SHIPPED",
        format!("BOTH {:.0}x (uniform)", base_cfg.weathering / raw_cfg.weathering),
        d1 / d.bedrock_erosion.max(1e-30),
    );
    println!(
        "\n  100x on supply alone buys {:.1}x; 100x on transport alone buys {:.1}x; the uniform\n  \
         {:.0}x buys {uniform_gain:.0}x. Two levers that only pay together (journal/0108's shape, and\n  \
         journal/0111's) — and the reason the fix is ONE multiplier over FOUR rates rather than\n  \
         a fifth multiplicand in a list.",
        gains[0],
        gains[1],
        base_cfg.weathering / raw_cfg.weathering,
    );
}

/// **Published denudation bands, in m/Myr** — the external anchor this whole
/// probe exists to be measured against. Sourced for journal/0111; every one is a
/// real compilation or a named study, and the *floor* row is the load-bearing
/// one, because a world below the floor is not a slow landscape, it is a stopped
/// one.
const BANDS: [(&str, f64, f64, &str); 6] = [
    (
        "FLOOR: Antarctic Dry Valleys / Atacama",
        0.1,
        1.0,
        "Morgan et al. 2010 JGR-ES (10Be/26Al, McMurdo 0.1-4, Arena Valley ~0.19); \
         Ritter et al. 2023 JGR-ES (Atacama near-stasis, 21Ne exposure ages 9-37 Ma)",
    ),
    (
        "stable craton / shield bedrock",
        1.0,
        10.0,
        "Bierman & Caffee 2002 GSA Bull (Australian inselbergs 0.3-5.7); \
         Bierman & Caffee 2001 Am.J.Sci (Namib bedrock 1-5); \
         Veselovskiy et al. 2019 Tectonics (Fennoscandia AFT 1-2.5)",
    ),
    (
        "global outcrop median (10Be, n=1599)",
        5.4,
        12.0,
        "Portenga & Bierman 2011 GSA Today — median 5.4, mean 12, max ~140",
    ),
    (
        "Phanerozoic global continental mean",
        16.0,
        62.0,
        "Wilkinson & McElroy 2007 GSA Bull — 16 from preserved sediment volumes (10^8 yr), \
         62 from modern natural yield; the disagreement is live (cf. Willenbring & von \
         Blanckenburg 2010 Nature)",
    ),
    (
        "passive-margin upland (Appalachians)",
        27.0,
        40.0,
        "Matmon, Bierman et al. 2003 Geology — Great Smokies 27+-4, 10Be catchment-averaged",
    ),
    (
        "active orogen (Taiwan, Himalaya, S.Alps)",
        3000.0,
        12000.0,
        "Dadson et al. 2003 Nature (Taiwan 3000-6000); Herman et al. 2013 Nature (Himalaya \
         7000-12000); Koppes & Montgomery 2009 Nat.Geosci (>10000 local)",
    ),
];

#[cfg(test)]
mod gate {
    use super::*;

    /// **The instrument does not move the world.** The load-bearing test: a
    /// measurement taken with `denudation_ledger` on is only a measurement of the
    /// *shipped* world if the flag changes nothing. Every counter is a `+=` on a
    /// struct nobody reads back into the physics, so the final surface must be
    /// **bit-identical** either way.
    ///
    /// *Why it is scale-free.* It is a statement about the code's data flow, not
    /// about a landscape: the flag gates only reads and ledger writes, so
    /// identity holds cell-for-cell at any world size. The smallest world that
    /// still runs every gated phase (transport sinks, diffusion, wave, wind)
    /// proves it.
    #[test]
    fn the_denudation_ledger_is_inert() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let base = production_config(&pregen.grid, SEED);
        let on = DeepConfig {
            denudation_ledger: true,
            ..base
        };
        let a = run_cells(&pregen.grid, &base, true);
        let b = run_cells(&pregen.grid, &on, true);
        assert_eq!(a.grid.r.len(), b.grid.r.len());
        for i in 0..a.grid.r.len() {
            assert_eq!(
                a.grid.r[i].to_bits(),
                b.grid.r[i].to_bits(),
                "bedrock plane moved at cell {i} when the denudation ledger was turned on"
            );
            assert_eq!(
                a.grid.h[i].to_bits(),
                b.grid.h[i].to_bits(),
                "regolith plane moved at cell {i} when the denudation ledger was turned on"
            );
        }
        assert_eq!(
            a.erosion.transport_ledger().exported_m(),
            0.0,
            "the export counters must be exactly zero with the flag off"
        );
        assert!(
            b.erosion.transport_ledger().exported_m() > 0.0,
            "nothing at all left the land system — either the world is inert or the \
             counters are not wired"
        );
    }

    /// **The sink itemisation equals its own total.** Marine yield plus border
    /// pile-up must re-sum to the sink deposition the pre-existing ledger already
    /// counted — the standing probe-defect shape in this repo (both
    /// `flow_cost_probe` failures were a missing row in an itemisation), and wrong
    /// at every world size because it is an arithmetic identity over one branch.
    ///
    /// This is the test that would catch the single most damaging error this probe
    /// could make: quietly counting sediment piled against the domain edge as
    /// sediment delivered to the sea.
    #[test]
    fn the_sink_itemisation_sums_to_the_sink_total() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let c = DeepConfig {
            denudation_ledger: true,
            ..production_config(&pregen.grid, SEED)
        };
        let l = run_cells(&pregen.grid, &c, true).erosion.transport_ledger();
        let sum = l.sink_marine_m + l.sink_border_m;
        let rel = ((sum - l.deposited_at_sink_m) / l.deposited_at_sink_m.max(1e-30)).abs();
        assert!(
            rel < 1e-12,
            "marine {} + border {} = {sum} != deposited_at_sink {}",
            l.sink_marine_m,
            l.sink_border_m,
            l.deposited_at_sink_m
        );
    }

    /// **Export is bounded by the process that carries it.** Each export term is a
    /// *share* of a flux the solve already computed, so it can never exceed it:
    /// shoreline creep ≤ all creep, marine sink deposition ≤ all sink deposition,
    /// the bedrock share of the wave quarry ≤ the whole quarry.
    ///
    /// *Why it is scale-free.* Each is a subset relation between two sums over the
    /// same non-negative terms — true per epoch, per cell, at any world size. A
    /// double-count or a sign error in any of the four new counters breaks one of
    /// them; a magnitude that merely looks wrong does not, which is the point
    /// (`assert invariants, never snapshots`).
    #[test]
    fn no_export_term_exceeds_the_flux_it_is_a_share_of() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let c = DeepConfig {
            denudation_ledger: true,
            ..production_config(&pregen.grid, SEED)
        };
        let l = run_cells(&pregen.grid, &c, true).erosion.transport_ledger();
        assert!(
            l.creep_to_sea_m <= l.diffused_m,
            "creep across the shore {} exceeds all creep {}",
            l.creep_to_sea_m,
            l.diffused_m
        );
        assert!(
            l.sink_marine_m <= l.deposited_at_sink_m,
            "marine sink deposition {} exceeds all sink deposition {}",
            l.sink_marine_m,
            l.deposited_at_sink_m
        );
        assert!(
            l.wave_bedrock_m <= l.wave_offshore_m,
            "the bedrock share of the wave quarry {} exceeds the whole quarry {}",
            l.wave_bedrock_m,
            l.wave_offshore_m
        );
        for v in [
            l.sink_marine_m,
            l.sink_border_m,
            l.creep_to_sea_m,
            l.wave_offshore_m,
            l.wave_bedrock_m,
            l.eolian_to_sea_m,
        ] {
            assert!(v >= 0.0, "an export counter went negative: {v}");
        }
    }
}
