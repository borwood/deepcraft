//! **The hillslope-transport operator, before and after** (journal/0122) — the
//! measurement behind the fix to `stubs.md` #29, the top blocker.
//!
//! journal/0116 named the register (*the flux limiter / donor-cell partition in
//! `erosion.rs::diffuse`*) and left one thing explicitly unmeasured:
//!
//! > *a donor-cell scheme that moves everything downslope has a **period-2 mode by
//! > construction** — A gives all its cover to B, B is now higher and gives it
//! > back — damped only by isostasy downstream.*
//!
//! **That hypothesis is isolated in `erosion.rs::hillslope_operator_tests`**, on a
//! bare grid with the creep pass and nothing else, where the flip-flop is exact
//! and can be read off by hand. This probe is the other half: does the isolated
//! mode explain the *world*, and does removing it fix the world.
//!
//! # What it reports
//!
//! Four arms — {shipped, calibrated} × {unbounded, sub-cycled} — each a full
//! production solve, with **both** an aggregate statistic and a neighbour-relative
//! one, because `corrections.md` **#61** is that an aggregate cannot license a
//! claim about arrangement:
//!
//! - aggregate: relief, mean surface, mean regolith, closed-hollow counts;
//! - neighbour-relative: `conc(h)` rms, its lag-1 autocorrelation on **both**
//!   axes, its sign-flip rate, and `conc(h)/h̄` — the normalisation journal/0116
//!   showed was the only one under which the defect was monotone.
//!
//! And the **temporal** discriminator, which is what makes the period-2 reading a
//! measurement of this world rather than of a fixture: the same world solved to
//! `N` and to `N + 1` epochs. A frozen spatial pattern correlates `+1` across one
//! more epoch; a period-2 mode correlates **−1**.
//!
//! `cargo run --release --example creep_operator_probe [-- --temporal]`

use std::time::Instant;

use dc_worldgen::deeptime::{
    CREEP_MAX_EDGE_COEFF, DeepConfig, DeepOverrides, census, production_config_with, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

/// One arm's measurements.
struct Row {
    label: String,
    substeps: u32,
    /// Peak effective creep diffusivity re-derived from the **post-run** planes —
    /// what "how far past the bound is this world" means as a property of the world.
    d_max: f64,
    /// Peak the last `diffuse` actually divided down. Differs from `d_max` by a few
    /// per cent because the biotic pass rewrites `bio_resist` *after* erosion, so the
    /// post-run planes are one epoch ahead. The gate compares against **this** one;
    /// comparing against `d_max` is comparing across that lag.
    peak_used: f64,
    land: usize,
    // --- aggregate (corrections #61: necessary, never sufficient) ------------
    relief: f64,
    surf_mean: f64,
    h_mean: f64,
    hollow_1: usize,
    hollow_10: usize,
    deepest: f64,
    // --- neighbour-relative --------------------------------------------------
    conc_surf_rms: f64,
    conc_h_rms: f64,
    conc_h_norm: f64,
    acf_h: (f64, f64),
    flip_h: (f64, f64),
    acf_surf: (f64, f64),
    // --- the operator's own counters -----------------------------------------
    limiter_pct: f64,
    secs: f64,
}

fn arm_cfg(cells: &CellGrid, calibrated: bool, substep: bool) -> DeepConfig {
    DeepConfig {
        creep_substep: substep,
        // The limiter-binding counter rides on the denudation ledger, and it is
        // read-only (journal/0116 confirmed the census reproduces to every printed
        // digit with it armed).
        denudation_ledger: true,
        ..production_config_with(
            cells,
            SEED,
            &DeepOverrides {
                calibrated_rates: Some(calibrated),
                ..DeepOverrides::default()
            },
        )
    }
}

fn measure(label: &str, cells: &CellGrid, cfg: &DeepConfig) -> (Row, Vec<f64>, Vec<bool>) {
    let t0 = Instant::now();
    let run = run_cells(cells, cfg, true);
    let secs = t0.elapsed().as_secs_f64();

    let (w, cell_a) = (run.grid.w, run.grid.cell_m * run.grid.cell_m);
    let n = w * w;
    let sea = dc_worldgen::deeptime::sea_level_at(cfg, cfg.iterations.saturating_sub(1));
    let surf: Vec<f64> = (0..n).map(|i| run.grid.surf_at(i)).collect();
    let filled = run.erosion.filled();
    let routed = run.erosion.routed_surface();

    // The population every statistic below is taken over: interior land.
    let ok: Vec<bool> = (0..n)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            x > 0 && y > 0 && x < w - 1 && y < w - 1 && surf[i] > sea
        })
        .collect();
    let land = ok.iter().filter(|v| **v).count();

    let conc_surf = census::laplacian8(&surf, w);
    let conc_h = census::laplacian8(&run.grid.h, w);

    let mut relief = (f64::NEG_INFINITY, f64::INFINITY);
    let mut surf_sum = 0.0;
    let mut h_sum = 0.0;
    let (mut hollow_1, mut hollow_10) = (0usize, 0usize);
    let mut deepest = 0.0f64;
    for i in 0..n {
        relief.0 = relief.0.max(surf[i]);
        relief.1 = relief.1.min(surf[i]);
        if !ok[i] {
            continue;
        }
        surf_sum += surf[i];
        h_sum += run.grid.h[i];
        // The NON-saturating hollow instrument (corrections #62): the router's own
        // depression fill measures the hollow at a cell regardless of what its
        // neighbours are doing, so it cannot fall toward zero as damage spreads.
        let d = filled[i] - routed[i];
        if d > 1.0 {
            hollow_1 += 1;
            deepest = deepest.max(d);
        }
        if d > 10.0 {
            hollow_10 += 1;
        }
    }
    let _ = cell_a;
    let h_mean = h_sum / land as f64;
    let conc_h_rms = census::rms(&conc_h, &ok);
    let ax = census::acf4(&conc_h, &ok, w, true);
    let ay = census::acf4(&conc_h, &ok, w, false);
    let sx = census::acf4(&conc_surf, &ok, w, true);
    let sy = census::acf4(&conc_surf, &ok, w, false);
    let led = run.erosion.transport_ledger();
    let limiter_pct = if led.creep_cell_epochs > 0 {
        100.0 * led.creep_limited_cell_epochs as f64 / led.creep_cell_epochs as f64
    } else {
        f64::NAN
    };

    let row = Row {
        label: label.to_string(),
        substeps: run.erosion.creep_substeps(),
        d_max: run.erosion.max_eff_creep(&run.grid, cfg.diffusion),
        peak_used: run.erosion.creep_peak_coeff(),
        land,
        relief: relief.0 - relief.1,
        surf_mean: surf_sum / land as f64,
        h_mean,
        hollow_1,
        hollow_10,
        deepest,
        conc_surf_rms: census::rms(&conc_surf, &ok),
        conc_h_rms,
        conc_h_norm: conc_h_rms / h_mean,
        acf_h: (ax[0], ay[0]),
        flip_h: (
            census::flip_rate(&conc_h, &ok, w, true),
            census::flip_rate(&conc_h, &ok, w, false),
        ),
        acf_surf: (sx[0], sy[0]),
        limiter_pct,
        secs,
    };
    (row, conc_h, ok)
}

fn print_header() {
    println!(
        "\n{:<26} {:>4} {:>7} {:>9} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8} {:>7} {:>8} {:>7}",
        "arm",
        "sub",
        "d_max",
        "conc(h)",
        "conc/h̄",
        "ACF x",
        "ACF y",
        "flip x",
        "surf rms",
        "hollow>1",
        ">10",
        "mean h",
        "gen s"
    );
}

fn print_row(r: &Row) {
    println!(
        "{:<26} {:>4} {:>7.3} {:>9.2} {:>9.3} {:>8.3} {:>8.3} {:>8.3} {:>8.2} {:>8} {:>7} {:>8.2} {:>7.1}",
        r.label,
        r.substeps,
        r.d_max,
        r.conc_h_rms,
        r.conc_h_norm,
        r.acf_h.0,
        r.acf_h.1,
        r.flip_h.0,
        r.conc_surf_rms,
        r.hollow_1,
        r.hollow_10,
        r.h_mean,
        r.secs
    );
}

fn print_landscape(r: &Row) {
    println!(
        "  {:<24} land {:>7}  relief {:>9.1} m  mean surf {:>8.1} m  deepest hollow {:>7.1} m  \
         limiter bound {:>5.1} %  surf ACF {:+.3}/{:+.3}  peak coeff used {:>7.3}",
        r.label,
        r.land,
        r.relief,
        r.surf_mean,
        r.deepest,
        r.limiter_pct,
        r.acf_surf.0,
        r.acf_surf.1,
        r.peak_used
    );
}

fn main() {
    let temporal = std::env::args().any(|a| a == "--temporal");
    println!(
        "creep operator probe — seed {SEED}, Extent::Medium, monotonicity bound \
         CREEP_MAX_EDGE_COEFF = {CREEP_MAX_EDGE_COEFF}"
    );
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let cells = &pregen.grid;

    print_header();
    let mut rows = Vec::new();
    for (calibrated, substep, label) in [
        (false, false, "shipped · unbounded"),
        (false, true, "shipped · sub-cycled"),
        (true, false, "calibrated · unbounded"),
        (true, true, "calibrated · sub-cycled"),
    ] {
        let cfg = arm_cfg(cells, calibrated, substep);
        let (row, _, _) = measure(label, cells, &cfg);
        print_row(&row);
        rows.push(row);
    }
    println!();
    for r in &rows {
        print_landscape(r);
    }

    println!(
        "\nReference values for ACF(1) of the concavity field, in closed form \
         (journal/0116 § D1):\n  smooth → +1   white noise → −0.167   perfect \
         checkerboard → −1.\nThe discriminator is not the sign; it is how far past \
         −1/6 the value has gone."
    );

    if temporal {
        run_temporal(cells);
    } else {
        println!(
            "\n(--temporal runs the period-2 discriminator on the world: two more \
             solves, N and N+1 epochs.)"
        );
    }
    if std::env::args().any(|a| a == "--ladder") {
        run_ladder(cells);
    } else {
        println!("(--ladder walks the erosion multiplier under the fixed operator.)");
    }
}

/// **The multiplier ladder under the fixed operator** — the input the calibration
/// re-derivation needs, and the thing this slice is *not* allowed to decide.
///
/// `EROSION_CALIBRATION = 45` was chosen (journal/0114) against a world whose
/// hillslope pass was a **one-cell-per-epoch conveyor**: the flux limiter capped
/// every cell's export at its own inventory, so raising `diffusion` bought almost
/// nothing (100× on transport alone bought 1.6×, stubs.md #27). Remove the cap and
/// the same multiplier is no longer the same amount of erosion. **A constant
/// fitted against a broken operator does not survive fixing the operator**, and
/// the only honest thing a fix slice can do about it is measure the ladder and
/// hand it back.
fn run_ladder(cells: &CellGrid) {
    println!("\n--- the erosion multiplier under the sub-cycled operator ---");
    println!(
        "{:>6} {:>4} {:>9} {:>9} {:>9} {:>9} {:>9} {:>8} {:>7}",
        "mult", "sub", "mean h", "conc(h)", "ACF x", "hollow>1", ">10", "deepest", "gen s"
    );
    for mult in [1.0, 3.0, 5.0, 10.0, 20.0, 45.0] {
        let cfg = DeepConfig {
            creep_substep: true,
            denudation_ledger: true,
            ..production_config_with(
                cells,
                SEED,
                &DeepOverrides {
                    calibrated_rates: Some(false),
                    erosion_budget: Some(mult),
                    ..DeepOverrides::default()
                },
            )
        };
        let (r, _, _) = measure("ladder", cells, &cfg);
        println!(
            "{mult:>6.0} {:>4} {:>9.2} {:>9.2} {:>9.3} {:>9} {:>9} {:>8.1} {:>7.1}",
            r.substeps,
            r.h_mean,
            r.conc_h_rms,
            r.acf_h.0,
            r.hollow_1,
            r.hollow_10,
            r.deepest,
            r.secs
        );
    }
}

/// **The period-2 discriminator, on the world rather than on a fixture.**
///
/// Solve the same world to `N` and to `N + 1` epochs and correlate the two
/// `conc(h)` fields cell by cell. The two candidate readings of a large negative
/// spatial autocorrelation make **opposite** predictions here, which is what makes
/// it a discriminator rather than a restatement:
///
/// - a **frozen** grid-scale pattern (say the solve carving a fixed stencil into
///   every cell) is the same pattern one epoch later ⇒ correlation near **+1**;
/// - a **period-2 mode** is its own negation one epoch later ⇒ **−1**.
///
/// Run on both arms, because journal/0116's most load-bearing lesson was that four
/// of its five facts came from the row expected to be a formality.
fn run_temporal(cells: &CellGrid) {
    println!("\n--- the temporal discriminator: conc(h) at N vs N+1 epochs ---");
    for (calibrated, substep, label) in [
        (false, false, "shipped · unbounded"),
        (true, false, "calibrated · unbounded"),
        (true, true, "calibrated · sub-cycled"),
    ] {
        let base = arm_cfg(cells, calibrated, substep);
        let mut plus = base;
        plus.iterations = base.iterations + 1;
        let (_, a, ok_a) = measure(label, cells, &base);
        let (_, b, ok_b) = measure(label, cells, &plus);
        let pairs: Vec<(f64, f64)> = a
            .iter()
            .zip(b.iter())
            .enumerate()
            .filter(|(i, _)| ok_a[*i] && ok_b[*i])
            .map(|(_, (x, y))| (*x, *y))
            .collect();
        let r = census::pearson(&pairs);
        println!(
            "  {label:<26} corr(conc_h[N], conc_h[N+1]) = {r:+.4}   ({} cells)",
            pairs.len()
        );
    }
    println!(
        "  +1 ⇒ a frozen spatial pattern.  −1 ⇒ the field is its own negation one \
         epoch later: a period-2 mode."
    );
}

/// **The gate.** Run at the smallest extent that still exercises the invariant,
/// per the probe doctrine in CLAUDE.md § Gates.
#[cfg(test)]
mod gate {
    use super::*;

    fn small() -> Pregen {
        Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        })
    }

    /// **The guard the blocker was missing**, and the one journal/0116 could only
    /// write for the *shipped* arm because the calibrated arm was broken:
    /// the calibrated world's regolith must not carry a grid-scale oscillation.
    ///
    /// The bound is taken from the closed-form references, not from a run: −0.5
    /// sits 3× past white noise's −1/6 and 2× short of a perfect checkerboard's
    /// −1, and the sign-flip bar of 0.85 sits between white noise's 0.55 and a
    /// checkerboard's 1.00. **A bound with a derivation is evidence; one chosen
    /// until green is not.**
    ///
    /// Scale-free: `ACF(1)` of an eight-neighbour Laplacian is a statement about
    /// adjacent samples, and its three reference values are properties of the
    /// stencil rather than of the grid. It was **−0.847 / −0.797** at this extent
    /// before the fix, so the guard is not vacuous.
    #[test]
    fn the_calibrated_solve_has_no_grid_scale_oscillation() {
        let pregen = small();
        let cfg = arm_cfg(&pregen.grid, true, true);
        let (r, _, _) = measure("gate", &pregen.grid, &cfg);
        assert!(
            r.acf_h.0 > -0.5 && r.acf_h.1 > -0.5,
            "the calibrated regolith is oscillating at the grid scale: conc(h) \
             ACF(1) = {:+.3} / {:+.3} against a white-noise reference of −0.167 and \
             a checkerboard's −1 (rms {:.2} m, conc/h̄ {:.3})",
            r.acf_h.0,
            r.acf_h.1,
            r.conc_h_rms,
            r.conc_h_norm
        );
        assert!(
            r.flip_h.0 < 0.85 && r.flip_h.1 < 0.85,
            "concavity sign-alternation {:.3} / {:.3} is at checkerboard levels",
            r.flip_h.0,
            r.flip_h.1
        );
    }

    /// **And the operator is inert where it was never needed.** The sub-cycle
    /// count is a pure function of the rate the config states and the per-cell
    /// susceptibility planes; asserting it is *reported* rather than assumed is
    /// what keeps the gen-time cost in the diff (`n×` the pass) rather than in a
    /// surprise.
    #[test]
    fn the_sub_cycle_count_follows_from_the_rate() {
        let pregen = small();
        for calibrated in [false, true] {
            let cfg = arm_cfg(&pregen.grid, calibrated, true);
            let (r, _, _) = measure("gate", &pregen.grid, &cfg);
            let expect = (r.peak_used / CREEP_MAX_EDGE_COEFF).ceil().max(1.0) as u32;
            assert_eq!(
                r.substeps, expect,
                "calibrated={calibrated}: {} sub-steps against a peak effective \
                 diffusivity of {:.4}",
                r.substeps, r.d_max
            );
            assert!(
                r.peak_used / f64::from(r.substeps) <= CREEP_MAX_EDGE_COEFF,
                "the sub-step coefficient {:.4} is still past the bound",
                r.peak_used / f64::from(r.substeps)
            );
        }
    }
}
