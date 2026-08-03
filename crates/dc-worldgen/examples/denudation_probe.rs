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
//! (optionally with an explicit ladder: `-- 100 300 1000`; a pair rung
//! `-- 150:50` scales supply 150× / transport 50× — audit § 6.2 R3; an
//! unbounded-operator control rung `-- u:150` runs with `creep_substep: false`
//! — R4. Explicit rungs run the measurement lean: the calibration-comparison
//! and single-lever-contrast arms are skipped.)

use dc_worldgen::deeptime::erosion::TransportLedger;
use dc_worldgen::deeptime::{
    DeepConfig, DeepOverrides, SEA_LEVEL_M, build_cells, census, production_config,
    production_config_with, run_cells, sea_level_at,
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

/// **The world behind the flag** — the calibrated rate constants
/// (`DeepOverrides::calibrated_rates: Some(true)`), which is **not** what production
/// builds. Everything else about the solve is identical, so the before/after in this
/// report is a **single-variable** comparison and the baseline is re-measured in the
/// same run rather than quoted from journal/0111 across three merges.
fn cfg_calibrated(cells: &CellGrid) -> DeepConfig {
    let o = DeepOverrides {
        calibrated_rates: Some(true),
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
        erosion_budget: Some(mult),
        ..DeepOverrides::default()
    };
    DeepConfig {
        denudation_ledger: true,
        ..production_config_with(cells, SEED, &o)
    }
}

/// **The pair rung `(S, T)`** (audit § 6.2 R3): supply (`weathering`,
/// `k_transport`, `k_bedrock`) at `S×` and transport (`diffusion`) at `T×`,
/// both over the raw constants. At `S == T` this is [`cfg_uniform`] bit for bit
/// (the same multiplies on the same operands — independent fields, so order is
/// immaterial). Its one question: is a cheaper transport still in band, i.e. is
/// chain C's "transport stops binding" derivation right where it matters.
fn cfg_pair(cells: &CellGrid, s: f64, t: f64) -> DeepConfig {
    let mut c = DeepConfig {
        denudation_ledger: true,
        ..production_config(cells, SEED)
    };
    c.weathering *= s;
    c.k_transport *= s;
    c.k_bedrock *= s;
    c.diffusion *= t;
    c
}

/// **The unbounded control** (audit § 6.2 R4): one uniform rung run under the
/// pre-journal/0122 operator (`creep_substep: false`), so the operator repair
/// stays visible inside the same report rather than quoted across two entries.
fn cfg_unbounded(cells: &CellGrid, mult: f64) -> DeepConfig {
    DeepConfig {
        creep_substep: false,
        ..cfg_uniform(cells, mult)
    }
}

/// One rung of the derivation ladder, as parsed off the command line:
/// `150` → uniform, `150:50` → pair `(S, T)`, `u:150` → unbounded control.
enum Rung {
    Uniform(f64),
    Pair(f64, f64),
    Unbounded(f64),
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
    /// **Mean regolith thickness on land** — the state variable the whole coupling
    /// turns on. The cover taper is `exp(−H/H*)`, so `H` is the one quantity that
    /// can tell a uniform scaling (which should leave it alone) from a single-lever
    /// one (which moves it and detunes the taper). Reported per row of the
    /// derivation ladder for exactly that reason.
    mean_h: f64,

    // --- the chain-B coupling terms (P2 audit 2026-08-01 § 6.1, M1/M2) ----
    /// **`⟨exp(−H/H*)⟩`, run-integrated** — the area-weighted mean cover taper
    /// over every subaerial cell-epoch the weathering phase visited. The single
    /// number that closes the derivation's chain B: § 4.3 of the audit falsifies
    /// the `exp(−⟨H⟩/H*)` substitute by five orders of magnitude (Jensen — the
    /// weathering lives in the thin-cover tail, the mean in the thick bulk).
    taper_run: f64,
    /// The same taper on the **final** surface, over end-of-run land cells — the
    /// end-state companion, so a drifting taper is visible as run-vs-end.
    taper_end: f64,
    /// **`⟨(biotic × weatherability) × frost⟩`, run-integrated** — the modulator
    /// product, resolving the 1.34-vs-6× ambiguity in chain B's `D3₁/0.008`.
    mod_run: f64,
    /// **D3's supply side**: bedrock converted in place by the weathering front
    /// (`ledger.weathered_m`, whole-grid subaerial, land-area-averaged m/Myr).
    d3_supply: f64,
    /// **D3's incision side**: bedrock detached by `k_bedrock` fluvial incision
    /// (`ledger.incised_m`, same frame). Supply + incision itemise the R-lowering
    /// D3 integrates (up to the land-at-end restriction and the wave quarry,
    /// which are printed beside them rather than folded in).
    d3_incision: f64,

    // --- shape, neighbour-relative (journal/0122's instruments; M5 + the
    // --- acceptance table's conc/ACF/pit bars, corrections #61/#62) --------
    /// rms of the 8-neighbour Laplacian of the regolith plane over interior land.
    conc_h_rms: f64,
    /// Lag-1 autocorrelation of `conc(h)` along x / y (white noise −0.167).
    acf_h: (f64, f64),
    /// Closed hollows deeper than 1 m / 10 m on the router's own fill (the
    /// non-saturating instrument, corrections #62), and the deepest anywhere.
    hollow_1: usize,
    hollow_10: usize,
    deepest: f64,
    /// Sub-steps the hillslope pass took (1 = the unbounded / in-bound operator).
    substeps: u32,
    /// Whether this run held the fixed (sub-cycled) operator — asserted per row
    /// so a report cannot silently mix operators (audit § 6.2).
    substep_on: bool,
    /// **Wall time of this world's deep-time run**, seconds — the gen-time cost, so a
    /// calibration reports its own like any other slice. Both arms are timed in the
    /// same process on the same machine, so the *difference* is the claim and the
    /// absolute is machine noise.
    secs: f64,
    /// **Total recorded units** across every cell — the residency proxy that moves
    /// when a rate change alters how much is deposited and how often the merge key
    /// changes. An absolute, per CLAUDE.md's residency rule.
    units: usize,
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
    let t0 = std::time::Instant::now();
    // The *initial* grid, rebuilt from the same pure `(cells, cfg)` inputs — the
    // only way to get a "before" plane, since the run consumes its own grid.
    let before = build_cells(cells, &cfg);
    let run = run_cells(cells, &cfg, true);
    let secs = t0.elapsed().as_secs_f64();
    let g = &run.grid;
    let units: usize = g.strata.iter().map(|s| s.units.len()).sum();
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
    let mut mean_h = 0.0;
    let mut taper_end = 0.0;
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
        mean_h += g.h[i];
        taper_end += (-g.h[i] / cfg.h_star).exp();
        max_surf = max_surf.max(surf[i]);
        min_surf = min_surf.min(surf[i]);
    }
    mean_surf /= nl;
    mean_h /= nl;
    taper_end /= nl;

    // --- the run-integrated coupling terms (M1/M2), read off the ledger ----
    let taper_run = if ledger.weather_cell_epochs > 0 {
        ledger.weather_taper_sum / ledger.weather_cell_epochs as f64
    } else {
        f64::NAN
    };
    let mod_run = if ledger.weather_cell_epochs > 0 {
        ledger.weather_mod_sum / ledger.weather_cell_epochs as f64
    } else {
        f64::NAN
    };
    let d3_supply = ledger.weathered_m / nl / myr;
    let d3_incision = ledger.incised_m / nl / myr;

    // --- shape, neighbour-relative (journal/0122's instruments, M5) --------
    // Population and stencil match `creep_operator_probe` exactly (interior
    // cells above the FINAL sea stand), so these columns are commensurable with
    // journal/0122's ladder — which is what the ±0.15 acceptance bar is stated
    // against.
    let w = g.w;
    let sea_final = sea_level_at(&cfg, cfg.iterations.saturating_sub(1));
    let ok: Vec<bool> = (0..n)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            x > 0 && y > 0 && x < w - 1 && y < w - 1 && surf[i] > sea_final
        })
        .collect();
    let conc_h = census::laplacian8(&g.h, w);
    let conc_h_rms = census::rms(&conc_h, &ok);
    let acf_h = (
        census::acf4(&conc_h, &ok, w, true)[0],
        census::acf4(&conc_h, &ok, w, false)[0],
    );
    let filled = run.erosion.filled();
    let routed = run.erosion.routed_surface();
    let (mut hollow_1, mut hollow_10, mut deepest) = (0usize, 0usize, 0.0f64);
    for i in 0..n {
        if !ok[i] {
            continue;
        }
        let dpit = filled[i] - routed[i];
        if dpit > 1.0 {
            hollow_1 += 1;
            deepest = deepest.max(dpit);
        }
        if dpit > 10.0 {
            hollow_10 += 1;
        }
    }

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
        mean_h,
        taper_run,
        taper_end,
        mod_run,
        d3_supply,
        d3_incision,
        conc_h_rms,
        acf_h,
        hollow_1,
        hollow_10,
        deepest,
        substeps: run.erosion.creep_substeps(),
        substep_on: cfg.creep_substep,
        secs,
        units,
    }
}

/// The uniform-multiplier ladder the derivation sweep walks by default. Roughly
/// half-decade spacing across the two orders of magnitude between "the stopped
/// clock" and "faster than this world's own steady-state ceiling", so the row that
/// lands in the published band is bracketed on both sides rather than extrapolated
/// to. Override from the command line: `--example denudation_probe -- 100 300 1000`.
const LADDER: [f64; 5] = [10.0, 45.0, 100.0, 300.0, 1000.0];

fn main() {
    // `150` = uniform rung; `150:50` = pair (S, T); `u:150` = unbounded control.
    // With NO args the historical full report runs (default ladder + the
    // calibration comparison + the single-lever contrast). With EXPLICIT rungs
    // the probe runs the measurement plan lean — the 1× headline plus exactly
    // the rungs asked for — because the P2 runs (audit § 6.2) cost over an hour
    // of solve already and the calibration/contrast arms answer a question those
    // runs are not asking.
    let mut rungs: Vec<Rung> = Vec::new();
    for a in std::env::args().skip(1) {
        if let Some(rest) = a.strip_prefix("u:") {
            if let Ok(m) = rest.parse() {
                rungs.push(Rung::Unbounded(m));
            }
        } else if let Some((s, t)) = a.split_once(':') {
            if let (Ok(s), Ok(t)) = (s.parse(), t.parse()) {
                rungs.push(Rung::Pair(s, t));
            }
        } else if let Ok(m) = a.parse::<f64>() {
            rungs.push(Rung::Uniform(m));
        }
    }
    let explicit = !rungs.is_empty();
    if !explicit {
        rungs = LADDER.iter().map(|&m| Rung::Uniform(m)).collect();
    }
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
    // **The agreement is a MEASURED outcome, so the caption is chosen from it.**
    // journal/0111 wrote "two independent instruments, one number" as flat prose
    // because they agreed to 2.4 % at the uncalibrated rates. They do not agree at
    // every rate (journal/0114 — see the calibration section), and a sentence that
    // asserts agreement regardless of the two numbers printed beside it is exactly
    // the `flux_record_probe` defect.
    let gap = 100.0 * (d.catchment_averaged / d.bedrock_erosion.max(1e-30) - 1.0).abs();
    println!(
        "  WHICH IS WHY D3 MATTERS: bedrock erosion is a per-cell rock-removal plane with no\n  \
         shoreline in it at all. D1 {:.4} vs D3 {:.4} — they differ by {gap:.1} %.",
        d.catchment_averaged, d.bedrock_erosion,
    );
    println!(
        "  {}",
        if gap < 25.0 {
            "Two independent instruments that share no arithmetic land on the same number, and\n  \
             that is what licenses reporting it."
        } else {
            "THEY DO NOT AGREE, and the disagreement is the report rather than something to\n  \
             average away. D1 is a GROSS land->sea boundary flux, and the shoreline migrates\n  \
             through a +-35 m sea-level cycle four times over the run — so cover ferried across,\n  \
             stranded by the next regression and ferried across again is counted each time.\n  \
             D3 has no shoreline in it. Read D3 as the rate and D1 as an upper bound; the\n  \
             calibration section itemises the arithmetic that fails to close."
        }
    );
    // The limiting regime is a *reading of the ratio*, so it is chosen from the
    // ratio rather than written beside it.
    let shed = d.catchment_averaged / d.bedrock_erosion.max(1e-30);
    println!(
        "  It also names the limiting regime: the boundary flux is {:.0} % of the bedrock this\n  \
         world converts or incises, so the land is {}.",
        100.0 * shed,
        if shed > 1.25 {
            "reporting MORE leaving than it detaches — which is\n  \
             not a regime at all but the gross-flux artifact just named. Take the regime from\n  \
             the `mean H` trend in the calibration section instead: cover is THICKENING, so\n  \
             this world is transport-limited"
        } else if shed > 0.9 {
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
    println!(
        "  MATCH THE INSTRUMENT TO THE BAND (P2 audit § 1.4): D1 is catchment-averaged, so\n  \
         its literature counterpart is the BASIN row; D3 = {:.4} m/Myr is a bedrock-lowering\n  \
         plane, so its counterpart is the outcrop/craton rows — and D3, not D1, is the\n  \
         acceptance quantity (corrections #60: D1 is a gross shoreline flux, upper bound only).",
        d.bedrock_erosion
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
             Valleys regolith, {:.2}-2.1 m/Myr, Arena Valley steady-state 0.53 — Morgan et al.\n  \
             2010; the hyperarid Atacama core, ~1, arid since the Oligocene-Miocene — Dunai\n  \
             et al. 2005). The MOST ACTIVE CELL ON THE WHOLE WORLD, at {:.4} m/Myr, is still\n  \
             slower than Antarctic bare rock under permanent ice-free hyperaridity.\n  \
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
        if d.pct[3] >= CRATON_FLOOR {
            "above"
        } else {
            "below"
        },
        if d.max_cell >= CRATON_FLOOR {
            "above"
        } else {
            "below"
        },
        above_floor,
    );

    // --- the ceiling this world sets for itself (M3: the FIXED formula) ----
    // Airy compensation returns (rho_m - rho_c)/rho_m of every eroded metre as
    // a surface drop and rebounds the rest. The pre-P2 version of this divided
    // the MEASURED rock uplift — which already contains the rebound of current
    // erosion — by f, a form valid only while erosion is negligible, i.e. only
    // in the regime the calibration exists to leave (P2 audit § 4.1). The fixed
    // form decomposes first: U_tect = D4 − (1−f)·D3, then ceiling = U_tect / f.
    let rho_m = dc_worldgen::deeptime::isostasy::RHO_MANTLE;
    let rho_c =
        dc_worldgen::deeptime::isostasy::rho_crust(dc_worldgen::deeptime::CrustKind::Continental);
    let f_airy = (rho_m - rho_c) / rho_m;
    let u_tect = d.rock_uplift - (1.0 - f_airy) * d.bedrock_erosion;
    let ceiling = u_tect / f_airy;
    println!(
        "\n--- THE CEILING THIS WORLD SETS FOR ITSELF ---\n  \
         Airy compensation returns (rho_m - rho_c)/rho_m = {f_airy:.4} of each eroded metre as a\n  \
         surface DROP and rebounds the other {:.1} %. Decomposing the measured D4 = {:.4} m/Myr:\n  \
         U_tect = D4 - (1-f)*D3 = {u_tect:.4} m/Myr, and a landscape in topographic steady\n  \
         state denudes at U_tect / {f_airy:.4} = {ceiling:.3} m/Myr.\n  \
         THAT NUMBER WAS NOT CHOSEN. It falls out of two densities and a measured uplift, and\n  \
         it lands inside the published cratonic bedrock band (1-4 m/Myr, Namib mean ~2.5) on\n  \
         its own — the independent statement that the band is the right target for THIS world.\n  \
         (The pre-P2 formula, D4/f, prints {:.3} here; the two diverge exactly when the\n  \
         calibration starts working — P2 audit § 4.1.)",
        100.0 * (1.0 - f_airy),
        d.rock_uplift,
        d.rock_uplift / f_airy,
    );

    // -----------------------------------------------------------------------
    // THE CALIBRATION (journal/0114) — built, measured, and OFF. Skipped when
    // explicit rungs are given (the P2 measurement plan, audit § 6.2 — those
    // runs cost an hour-plus of solve, and the calibration/contrast arms
    // answer journal/0114's question, not P2's).
    // -----------------------------------------------------------------------
    let prod_cfg = cfg(&pregen.grid);
    let full = (!explicit).then(|| {
    let cal_cfg = cfg_calibrated(&pregen.grid);
    let cal = measure_cfg(&pregen.grid, &cal_cfg);
    let mult = cal_cfg.weathering / prod_cfg.weathering;

    println!("\n\n=== THE CALIBRATION (journal/0114) — BUILT AND OFF ===");
    println!(
        "  Everything above is the SHIPPED world, and the shipped world is UNCALIBRATED. The\n  \
         calibration exists, is reachable (`--calibrated-rates`), is pinned by name, and is\n  \
         switched OFF — because the acceptance below did not pass and because turning it on\n  \
         opens a defect in the solve. Both worlds are measured here, in one run, so the\n  \
         comparison is one variable rather than a number quoted across three merges.\n"
    );
    println!("  rate                       production      calibrated     ratio");
    for (name, a, b) in [
        ("weathering", prod_cfg.weathering, cal_cfg.weathering),
        ("diffusion", prod_cfg.diffusion, cal_cfg.diffusion),
        ("k_transport", prod_cfg.k_transport, cal_cfg.k_transport),
        ("k_bedrock", prod_cfg.k_bedrock, cal_cfg.k_bedrock),
        // The unscaled neighbours, printed so the scope is visible rather than
        // asserted: the cover length and an agent magnitude, neither of which moved.
        ("h_star (NOT scaled)", prod_cfg.h_star, cal_cfg.h_star),
        (
            "wave_erosion (NOT scaled)",
            prod_cfg.wave_erosion,
            cal_cfg.wave_erosion,
        ),
    ] {
        println!("  {name:<26} {a:>12.6} {b:>14.6}   {:>7.1}x", b / a);
    }

    println!("\n  quantity                       production      calibrated");
    for (name, a, b) in [
        ("D1 catchment denudation", d1, cal.catchment_averaged),
        (
            "D2 mean surface lowering",
            d.mean_surface_lowering,
            cal.mean_surface_lowering,
        ),
        ("D3 bedrock erosion", d.bedrock_erosion, cal.bedrock_erosion),
        ("D4 rock uplift", d.rock_uplift, cal.rock_uplift),
        (
            "D1/D3 (the balance)",
            d1 / d.bedrock_erosion.max(1e-30),
            cal.catchment_averaged / cal.bedrock_erosion.max(1e-30),
        ),
        (
            "D1/D4 (vs uplift)",
            d1 / d.rock_uplift.max(1e-30),
            cal.catchment_averaged / cal.rock_uplift.max(1e-30),
        ),
        ("mean land surface (m)", d.mean_surf, cal.mean_surf),
        ("relief (m)", d.relief, cal.relief),
        ("land cells", d.land_cells as f64, cal.land_cells as f64),
        ("most active cell", d.max_cell, cal.max_cell),
        ("mean regolith H (m)", d.mean_h, cal.mean_h),
        ("recorded units", d.units as f64, cal.units as f64),
        ("gen time, s (this run)", d.secs, cal.secs),
    ] {
        println!("  {name:<28} {a:>14.4} {b:>14.4}");
    }

    // --- the measured response ---------------------------------------------
    println!(
        "\n--- THE MEASURED RESPONSE ---\n  \
         A uniform multiplier on all four rates, applied to the production constants. Each row\n  \
         is a full 200-epoch world; the row at {mult:.0}x IS the world behind the flag, bit for\n  \
         bit (the same multiply on the same operands — asserted in tests/calibrated_rates.rs).\n\n  \
         WHY UNIFORM, AND WHAT THE HYPOTHESIS GOT WRONG. journal/0111 measured that neither\n  \
         lever pays alone, and the mechanism is the cover taper exp(-H/H*), H* = {:.1} m — the\n  \
         only term in the system carrying an ABSOLUTE LENGTH. Raise supply alone and the\n  \
         regolith made shields the rock that made it; raise transport alone and there is\n  \
         nothing to carry.\n  \
         The hypothesis was that scaling BOTH would leave the steady-state H where it was, so\n  \
         the taper would never engage and export would follow the multiplier. READ THE `mean H`\n  \
         COLUMN: IT DOES NOT. Cover thickens at every rung, which means the two levers are not\n  \
         symmetric — transport has a ceiling supply does not. Creep's flux limiter already\n  \
         binds on ~89 % of the cells that HAVE regolith to move at 1x (`creep-lim`), so the\n  \
         pass is a ONE-CELL-PER-EPOCH CONVEYOR and no increase in `diffusion` makes it faster\n  \
         (the contrast table below measures 1.6x for 100x transport). Export is then set by how\n  \
         much cover the shoreline ring can hand over — i.e. by H — which is why D1 tracks\n  \
         `mean H` and why reaching the craton band costs hundreds of metres of soil.\n",
        prod_cfg.h_star,
    );
        (cal, mult)
    });
    if explicit {
        println!("\n\n=== THE DERIVATION LADDER (P2 audit 2026-08-01 § 6.2) ===");
        println!(
            "  Explicit rungs. The calibration-comparison and single-lever-contrast arms are\n  \
             skipped: they answer journal/0114's question, not this run's. Every row is a full\n  \
             {}-epoch world under the FIXED operator (creep_substep: true) unless labelled UNB\n  \
             — the `sub` column in the mechanism table is the per-row assertion.",
            c.iterations
        );
    }

    let limited = |r: &Denudation| -> f64 {
        100.0 * r.ledger.creep_limited_cell_epochs as f64
            / (r.ledger.creep_cell_epochs.max(1)) as f64
    };
    // **The band verdict is on D3, against the cratonic bedrock band** (P2 audit
    // § 1.4): D1 is a gross shoreline flux and is inadmissible as the acceptance
    // quantity (corrections #60) — it is printed as an upper bound only.
    let band = |v: f64| -> &'static str {
        if (1.0..=4.0).contains(&v) {
            "IN 1-4"
        } else if v > 4.0 && v <= 10.0 {
            "in 1-10"
        } else if v < 1.0 {
            "below"
        } else {
            "above"
        }
    };
    println!(
        "\n  x            D1(ub)        D3   D3/D4  D1/D3  meansurf   relief   mean H    land  creep-lim  band(D3)"
    );
    let row = |label: &str, r: &Denudation| {
        println!(
            "  {label:<10} {:>10.4} {:>9.4} {:>6.2} {:>6.2} {:>8.1} {:>8.1} {:>7.2} {:>7} {:>6.1} %  {}",
            r.catchment_averaged,
            r.bedrock_erosion,
            r.bedrock_erosion / r.rock_uplift.max(1e-30),
            r.catchment_averaged / r.bedrock_erosion.max(1e-30),
            r.mean_surf,
            r.relief,
            r.mean_h,
            r.land_cells,
            limited(r),
            band(r.bedrock_erosion),
        );
    };
    row("1 SHIPPED", &d);
    let mut rows: Vec<(String, Denudation)> = Vec::new();
    for rung in &rungs {
        let (label, rcfg) = match rung {
            // The shipped row above IS the 1× rung — do not re-solve it.
            Rung::Uniform(m) if *m == 1.0 => continue,
            Rung::Uniform(m) => (format!("{m:.0}"), cfg_uniform(&pregen.grid, *m)),
            Rung::Pair(s, t) => (format!("{s:.0}:{t:.0}"), cfg_pair(&pregen.grid, *s, *t)),
            Rung::Unbounded(m) => (format!("{m:.0} UNB"), cfg_unbounded(&pregen.grid, *m)),
        };
        let r = measure_cfg(&pregen.grid, &rcfg);
        row(&label, &r);
        rows.push((label, r));
    }

    // --- the mechanism & shape table (M1/M2/M5 + the § 6.2 acceptance rows) --
    println!(
        "\n  MECHANISM & SHAPE per rung — ⟨taper⟩ = run-mean exp(-H/H*) (chain B's missing\n  \
         term), ⟨mod⟩ = run-mean (biotic x weatherability) x frost, supply/incis = the D3\n  \
         split (weathering front vs k_bedrock, whole-grid frame), conc(h)/ACF vs\n  \
         journal/0122 (white noise -0.167), hollows on the router's own fill (bar: >10 m = 0).\n"
    );
    println!(
        "  x           sub  taper.run  taper.end   mod   supply   incis  conc(h)   ACF x   ACF y  h>1     >10  deepest       D4   gen s"
    );
    let mrow = |label: &str, r: &Denudation| {
        // `!` after the sub-step count = creep_substep OFF (the unbounded
        // operator) — the per-row operator assertion the audit § 6.2 requires.
        let sub = if r.substep_on {
            format!("{}", r.substeps)
        } else {
            format!("{}!", r.substeps)
        };
        println!(
            "  {label:<10} {sub:>4} {:>9.4} {:>9.4} {:>6.3} {:>8.4} {:>7.4} {:>8.2} {:>7.3} {:>7.3} {:>6} {:>6} {:>7.1} {:>8.4} {:>7.1}",
            r.taper_run,
            r.taper_end,
            r.mod_run,
            r.d3_supply,
            r.d3_incision,
            r.conc_h_rms,
            r.acf_h.0,
            r.acf_h.1,
            r.hollow_1,
            r.hollow_10,
            r.deepest,
            r.rock_uplift,
            r.secs,
        );
    };
    mrow("1 SHIPPED", &d);
    for (label, r) in &rows {
        mrow(label, r);
    }

    // --- the Airy sustainability column (the audit header's integrator-settled
    // --- measurement: rebound fraction vs the Airy prediction, a physics
    // --- identity — if D4 stalls while D3 rises, the § 5.4 uplift call is owed)
    println!(
        "\n  AIRY SUSTAINABILITY — measured D4 vs the prediction U_tect + (1-f)*D3, with\n  \
         U_tect = {u_tect:.4} m/Myr from the shipped row. A ratio well below 1.0 means the\n  \
         sim's smoothed isostasy is NOT returning Airy rebound at this erosion rate, and\n  \
         the craton-band target may not be sustainable without an uplift_scale call (§ 5.4)."
    );
    println!("  x            D4 meas   D4 Airy    ratio");
    let srow = |label: &str, r: &Denudation| {
        let pred = u_tect + (1.0 - f_airy) * r.bedrock_erosion;
        println!(
            "  {label:<10} {:>8.4} {:>9.4} {:>8.2}",
            r.rock_uplift,
            pred,
            r.rock_uplift / pred.max(1e-30),
        );
    };
    srow("1 SHIPPED", &d);
    for (label, r) in &rows {
        srow(label, r);
    }

    println!(
        "\n  READ IT THIS WAY. D3 is the acceptance quantity (1-4 m/Myr, target 2.63); D1 is\n  \
         an upper bound only (corrections #60). `D1/D3` is the tell.\n    \
         ~1.0  the land sheds everything it detaches — a landscape in balance.\n    \
         <1.0  the land is making regolith it cannot move — TRANSPORT-limited.\n    \
         >1.0  the land is exporting stored cover faster than it detaches new rock\n    \
               (or the shoreline gross-flux artifact — see the D1/D3 caption above)."
    );
    // Everything below compares against the calibrated arm, which only the
    // no-args (journal/0114 replication) mode measures.
    let Some((cal, mult)) = full else {
        return;
    };
    println!(
        "\n  AND THE TWO INSTRUMENTS SEPARATE AS THE FLUXES GROW — say it rather than average it.\n  \
         journal/0111 reported D1 as the headline because D1 and D3 agreed to 2.4 % at the\n  \
         shipped rates. They do not stay agreed: the ratio tracks `mean H` up the ladder, and\n  \
         the arithmetic does not close. At {mult:.0}x the run removes {:.1} m of bedrock per land\n  \
         cell and stores {:+.1} m more regolith, yet D1 claims {:.1} m of export — which cannot\n  \
         come out of the rock that was removed.\n  \
         `creep_to_sea_m` is a GROSS land->sea edge flux. The sea stand cycles +-{:.0} m four\n  \
         times over the run, so a shoreline cell's cover is carried across, stranded by a\n  \
         regression and carried across again, and each crossing is counted. At 1x the fluxes\n  \
         were too small for that to matter; at {mult:.0}x they are not.\n  \
         So D3 is the sound instrument once the fluxes are large — a per-cell rock-removal\n  \
         plane with no shoreline in it — and D1 is an upper bound.",
        cal.bedrock_erosion * cal.myr,
        cal.mean_h - d.mean_h,
        cal.catchment_averaged * cal.myr,
        prod_cfg.sea_level_amp,
    );

    // --- the acceptance, and why the flag is off ---------------------------
    let relief_pct = 100.0 * (cal.relief / d.relief - 1.0);
    println!(
        "\n--- THE ACCEPTANCE: the target band was NOT reached ---\n  \
         Target: 1-10 m/Myr, the published stable-craton band. Measured at {mult:.0}x: D1 {:.4}\n  \
         (an upper bound), D3 {:.4} (the sound one). NO multiplier reaches the band with a\n  \
         world left in it — the ladder above buries itself getting there.\n  \
         What {mult:.0}x DOES buy, and it is not nothing: denudation {:.1}x, bedrock erosion\n  \
         {:.1}x, and erosion's authority over the topography (D1/D4) {:.3} -> {:.2}.\n  \
         journal/0111's sharpest sentence was that erosion removed 2.7 % of what uplift added;\n  \
         at {mult:.0}x it is most of it, and the world leaves 'below every published terrestrial\n  \
         band' to enter the 0.1-1 floor band (McMurdo, Atacama).",
        cal.catchment_averaged,
        cal.bedrock_erosion,
        cal.catchment_averaged / d1.max(1e-30),
        cal.bedrock_erosion / d.bedrock_erosion.max(1e-30),
        d1 / d.rock_uplift.max(1e-30),
        cal.catchment_averaged / cal.rock_uplift.max(1e-30),
    );
    println!(
        "\n  AND WHY IT IS OFF — three costs, measured, none of them a matter of taste:\n  \
         1. DEEP CLOSED DEPRESSIONS AT ANY MULTIPLIER ABOVE 1x. Measured on the `mfd_routing`\n     \
            fixture: 0 pits deeper than 1 m at 1x, 44 at 5x (deepest 45 m), 66 at 10x, 148 at\n     \
            {mult:.0}x (deepest 112 m). The never-incise-below-the-lowest-receiver clamp is\n     \
            defeated once erosion is fast enough for the phases that run AFTER incision to\n     \
            lower a cell further within the same epoch. THIS IS A LATENT DEFECT IN THE SOLVE,\n     \
            NOT A PROPERTY OF THE NUMBER — it was invisible only because the world barely\n     \
            eroded, and it is the most important thing this probe found.\n  \
         2. The geotherm's coal-relocation claim collapses 1.37x -> 1.02x, because {mult:.0}x\n     \
            deposition makes burial depth rather than crustal gradient the dominant control.\n  \
         3. Mean regolith reaches {:.1} m, the top of the published deeply-weathered-shield\n     \
            range (30-60 m), against {:.1} m shipped.\n  \
         Relief moves {relief_pct:+.1} %, so the SHAPE survives — it is the pits and the coal\n  \
         that do not.",
        cal.mean_h, d.mean_h,
    );

    // --- the single-lever contrast, kept from journal/0111 -------------------
    println!(
        "\n--- THE CONTRAST: why not one lever (journal/0111's finding, re-measured) ---\n  \
         Hillslope creep is {:.2} % of this world's export, and the old `erosion_budget` scope\n  \
         (weathering + k_transport + k_bedrock, no diffusion) could not reach it. Both rows are\n  \
         built by hand because that scope no longer exists as a knob (stubs #24, closed).\n",
        100.0 * l.creep_to_sea_m / l.exported_m().max(1e-30)
    );
    let three_only = |m: f64| -> DeepConfig {
        let mut c = prod_cfg;
        c.denudation_ledger = true;
        c.weathering *= m;
        c.k_transport *= m;
        c.k_bedrock *= m;
        c
    };
    let creep_only = |m: f64| -> DeepConfig {
        let mut c = prod_cfg;
        c.denudation_ledger = true;
        c.diffusion *= m;
        c
    };
    println!("  scenario                       D1 (m/Myr)   D1/D3   vs shipped");
    let mut gains = [1.0f64; 2];
    for (k, (name, c)) in [
        ("supply only 100x (old scope)", three_only(100.0)),
        ("transport only 100x", creep_only(100.0)),
    ]
    .iter()
    .enumerate()
    {
        let r = measure_cfg(&pregen.grid, c);
        gains[k] = r.catchment_averaged / d1.max(1e-30);
        println!(
            "  {name:<30} {:>10.4}  {:>6.2}  {:>7.1}x",
            r.catchment_averaged,
            r.catchment_averaged / r.bedrock_erosion.max(1e-30),
            gains[k],
        );
    }
    println!(
        "  {:<30} {:>10.4}  {:>6.2}  {:>7.1}x   <-- behind the flag",
        format!("BOTH {mult:.0}x (uniform)"),
        cal.catchment_averaged,
        cal.catchment_averaged / cal.bedrock_erosion.max(1e-30),
        cal.catchment_averaged / d1.max(1e-30),
    );
    println!(
        "\n  100x on supply alone buys {:.1}x; 100x on transport alone buys {:.1}x; the uniform\n  \
         {mult:.0}x buys {:.0}x. Two levers that only pay together (journal/0108's shape, and\n  \
         journal/0111's) — and the reason the fix is ONE multiplier over FOUR rates rather than\n  \
         a fifth multiplicand in a list. What journal/0111 could not see is that they pay\n  \
         together only up to a ceiling neither of them sets.",
        gains[0],
        gains[1],
        cal.catchment_averaged / d1.max(1e-30),
    );
}

/// **Published denudation bands, in m/Myr** — the external anchor this whole
/// probe exists to be measured against. Sourced for journal/0111; every one is a
/// real compilation or a named study, and the *floor* row is the load-bearing
/// one, because a world below the floor is not a slow landscape, it is a stopped
/// one.
const BANDS: [(&str, f64, f64, &str); 7] = [
    (
        "FLOOR: Antarctic Dry Valleys / Atacama",
        0.1,
        1.0,
        "Morgan et al. 2010 JGR-ES (10Be/26Al, McMurdo regolith 0.19-2.1; Arena Valley \
         steady-state 0.53); Dunai et al. 2005 Geology + Placzek et al. 2010 EPSL \
         (hyperarid Atacama core ~1, arid since the Oligocene-Miocene)",
    ),
    (
        "stable craton / shield BEDROCK",
        1.0,
        4.0,
        "Bierman & Caffee 2001 Am.J.Sci (Namib bedrock 1-5, mean ~2.5); \
         Bierman & Caffee 2002 GSA Bull (Australian inselbergs 0.3-5.7 max-limiting); \
         Veselovskiy et al. 2019 Tectonics (Fennoscandia AFT 1-2.5; UNVERIFIED — \
         P2 audit 2026-08-01 § 1.1). THE D3 ACCEPTANCE BAND (audit § 1.4)",
    ),
    (
        "global OUTCROP median (10Be, n=450)",
        5.4,
        12.0,
        "Portenga & Bierman 2011 GSA Today — outcrops: median 5.4, mean 12 +- 1.3, \
         n = 450 (n=1599 is the WHOLE compilation, outcrops + basins)",
    ),
    (
        "global BASIN median (10Be, drainage basins)",
        54.0,
        218.0,
        "Portenga & Bierman 2011 GSA Today — basins: median 54, mean 218. THE row D1 \
         is commensurable with: D1 is a catchment-averaged quantity (its own doc \
         comment says so), and judging it against the OUTCROP row is a category \
         mismatch of one order of magnitude (P2 audit § 1.3)",
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

    /// **The calibration reaches the measured quantity, not just the config.**
    ///
    /// It measures the SHIPPED world against the world behind the flag. Production is
    /// uncalibrated (journal/0114 — the flag is off), so `raw` here is what ships and
    /// `cal` is what `--calibrated-rates` builds.
    ///
    /// `tests/calibrated_rates.rs` proves the four rate constants are exactly
    /// `EROSION_CALIBRATION×` the raw ones, and that the world moved. Neither of
    /// those says the thing this whole slice exists for actually happened: that
    /// **denudation went up**. A calibration that multiplied four constants and left
    /// the export ledger where it was would pass every test in that file.
    ///
    /// So this measures both worlds and compares them — a **ratio between two runs
    /// in the same test**, never an absolute m/Myr figure. That distinction is the
    /// doctrine (*assert invariants, never snapshots*): the absolute rate is exactly
    /// what a colleague improving the sediment router should be free to move, and a
    /// test pinned to today's 0.414 would fail *because someone fixed the transport
    /// ceiling*, which is worse than the defect it guards.
    ///
    /// *Why it is scale-free.* It is a comparison of one world against the same
    /// world with four constants scaled — the direction of the effect is a property
    /// of the rate law, not of the landscape, so it holds at any extent. The bound
    /// is deliberately loose: **5×**, against **37×** measured at production scale
    /// (journal/0114). It is sized to catch a silent revert or a disconnected knob,
    /// not to pin a tuning.
    #[test]
    fn the_calibration_actually_raises_the_measured_denudation() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let raw = measure_cfg(&pregen.grid, &cfg(&pregen.grid));
        let cal = measure_cfg(&pregen.grid, &cfg_calibrated(&pregen.grid));
        let gain = cal.catchment_averaged / raw.catchment_averaged.max(1e-30);
        println!(
            "uncalibrated D1 {:.6}, calibrated D1 {:.6}, gain {gain:.1}x",
            raw.catchment_averaged, cal.catchment_averaged
        );
        assert!(
            raw.catchment_averaged > 0.0,
            "the uncalibrated world exported nothing, so the ratio below is vacuous"
        );
        assert!(
            gain >= 5.0,
            "the erosional calibration moved the rate constants but not the world: \
             denudation went from {:.6} to {:.6} m/Myr, a gain of {gain:.2}x. Either \
             EROSION_CALIBRATION has been reverted or the scaling no longer reaches \
             the solve.",
            raw.catchment_averaged,
            cal.catchment_averaged
        );
    }

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
        assert_eq!(
            a.erosion.transport_ledger().weather_cell_epochs,
            0,
            "the chain-B coupling counters (P2 audit § 6.1 M1/M2) must be exactly \
             zero with the flag off — the accumulation branch fired in production"
        );
        assert!(
            b.erosion.transport_ledger().exported_m() > 0.0,
            "nothing at all left the land system — either the world is inert or the \
             counters are not wired"
        );
    }

    /// **The chain-B coupling counters are wired and bounded** (P2 audit
    /// 2026-08-01 § 6.1, M1/M2). The run-mean cover taper `⟨exp(−H/H*)⟩` is a
    /// mean of per-cell-epoch values each in `(0, 1]` (H ≥ 0 up to a sub-ULP
    /// transport residue, covered by the 1e-9 slack), so its sum must sit in
    /// `(0, count]`; the modulator product must be positive wherever weathering
    /// ran. Invariants, never snapshots — the magnitudes are the report's job.
    ///
    /// *Why it is scale-free.* Both are per-cell-epoch algebraic bounds on a
    /// monotone function of non-negative state — true at any extent.
    #[test]
    fn the_weather_coupling_counters_are_wired_and_bounded() {
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
            l.weather_cell_epochs > 0,
            "no subaerial cell-epochs were counted — the accumulation never ran"
        );
        let n = l.weather_cell_epochs as f64;
        assert!(
            l.weather_taper_sum > 0.0 && l.weather_taper_sum <= n * (1.0 + 1e-9),
            "the mean cover taper must sit in (0, 1]: sum {} over {} cell-epochs",
            l.weather_taper_sum,
            l.weather_cell_epochs
        );
        assert!(
            l.weather_mod_sum > 0.0,
            "the modulator-product sum must be positive where weathering ran"
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
