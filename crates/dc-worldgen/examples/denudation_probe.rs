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
//! ## What was NOT done
//!
//! Nothing was tuned. No calibration constant was touched. The instrumentation is
//! read-only with respect to the physics and sits behind
//! [`DeepConfig::denudation_ledger`], off in production, and the gate asserts the
//! surface plane is **bit-identical** with it on — which is what makes a number
//! taken with the flag on a number about the *shipped* world.
//!
//! Run: `cargo run --release -p dc-worldgen --example denudation_probe`

use dc_worldgen::deeptime::erosion::TransportLedger;
use dc_worldgen::deeptime::{
    DeepConfig, SEA_LEVEL_M, build_cells, production_config, run_cells, sea_level_at,
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
/// [`MYR_PER_EPOCH`] = **2.5 Myr per iteration**.
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
    let cfg = cfg(cells);
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

fn main() {
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
    println!("                                       RATIFIED 2026-07-19: the Phanerozoic register;");
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
         moved by hillslope creep, TOTAL                    {:>12.1}  (of which {:.2} % left)",
        l.sink_border_m,
        l.weathered_m,
        l.diffused_m,
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
         (they differ only by material crossing the shoreline as cells change status;\n   \
         a close match is what licenses reading D1 as THE land system's loss term)",
        d.land_dsurf_m, d.land_budget_m
    );

    println!("\n--- THE DISTRIBUTION across {} land cells ---", d.land_cells);
    println!("  bedrock erosion rate, m/Myr, per cell:");
    for (name, v) in ["min", "p10", "p25", "MEDIAN", "p75", "p90", "p95", "p99", "p99.9"]
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

    println!("\n--- READING IT ---");
    // The caption is DERIVED from the measurement, never written ahead of it
    // (CLAUDE.md: a printed caption is a published claim the gate cannot check).
    // The bands below are literature, cited in journal/0111 — they are the
    // comparison this probe exists to make, and they are not this world's numbers.
    let d1 = d.catchment_averaged;
    let band = if d1 < 1.0 {
        "BELOW every published terrestrial band. The slowest measured landscapes on \
         Earth — bare\n  cratonic bedrock in arid Australia and southern Africa, the \
         Antarctic Dry Valleys — sit\n  around 1 m/Myr and rarely below it. A world under \
         that is not a slow landscape; it is a\n  landscape whose erosional clock has \
         effectively stopped."
    } else if d1 < 10.0 {
        "in the STABLE CRATON / SHIELD band (~1-10 m/Myr). If the world is meant to read \
         as an\n  ancient low-relief interior, this is the honest answer and the \
         Movement 2b null follows\n  from the landscape rather than from a defect."
    } else if d1 < 50.0 {
        "in the PASSIVE-MARGIN / low-relief upland band (~10-50 m/Myr) — Appalachian, \
         SE Australian\n  escarpment country."
    } else if d1 < 200.0 {
        "in the MODERATE-RELIEF MOUNTAIN band (~50-200 m/Myr)."
    } else {
        "in the ACTIVE OROGEN band (>200 m/Myr, reaching 1000s)."
    };
    println!("  D1 = {d1:.4} m/Myr falls {band}");
    println!(
        "\n  Over the full {:.0} Myr the land system exported {:.2} m of average thickness.\n  \
         A real craton strips HUNDREDS of metres to kilometres over a Phanerozoic span.",
        d.myr,
        d1 * d.myr
    );
    println!(
        "\n  Uniformity: max/median = {concentration:.1}x and the top decile carries {:.0} % of the\n  \
         erosion. A uniformly dead surface and a quiet interior with active margins are\n  \
         different diagnoses; this is the number that tells them apart.",
        100.0 * d.top_decile_share
    );
    println!(
        "\n  NOTHING WAS TUNED. No calibration constant was touched, and the instrumentation\n  \
         is asserted bit-identical to production. If the number is terrible it is the\n  \
         world's number, not the probe's."
    );
}

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
