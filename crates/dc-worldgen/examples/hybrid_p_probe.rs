//! **FLOW continuation (b') — hybrid `p`**: the acceptance probe for a
//! *spatially varying* MFD convergence exponent (`docs/design/flow.md` § 2.6.2,
//! journal/0113).
//!
//! journal/0109 shipped MFD with **one** exponent for the whole world and
//! measured what that costs: the peak catchment collapsed **1,245 → 84 cells**.
//! Uniform MFD leaks water sideways at *every* cell, so a trunk river never
//! accumulates — which is physically wrong in a specific way: it applies
//! **hillslope sheet-flow behaviour inside channels**. Real water spreads where it
//! is unchannelised and stays in its banks once it is not.
//!
//! **The acceptance here is FLOW STRUCTURE, and deliberately nothing else.**
//! journal/0111 measured this world denuding at 0.0110 m/Myr — 9× slower than the
//! slowest landscape ever measured on Earth — with **0.02 %** of its export
//! leaving by rivers. A routing change concentrates that 0.02 %; it cannot move
//! denudation, cannot move the facies gradient, and will not make the world look
//! different. Measuring any of those would be measuring somebody else's slice
//! (the joint supply+transport calibration). So this probe reports:
//!
//! 1. **peak catchment** — D8 vs uniform `p` vs hybrid `p`;
//! 2. **the catchment distribution** — percentiles and a concentration ratio;
//! 3. **simultaneous divergence**, which journal/0109 bought (0 → 7,548,646
//!    within-epoch divergent `(cell, epoch)` pairs) and which concentrating flow
//!    **must not destroy** — deltas are made of it;
//! 4. **mass conservation**, per species;
//! 5. the **`χ = A·S²` distribution** the thresholds were calibrated against, and
//!    the **representational floor**'s measured cost (`stubs.md` § 22, which until
//!    now said "argued, **not measured**").
//!
//! Run: `cargo run --release -p dc-worldgen --example hybrid_p_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{
    self, DeepConfig, DeepField, build_field_cfg, production_config, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

/// D8 neighbour offsets — the solve's `NEIGH8` order, re-stated because the
/// probe re-derives `χ` for its histogram and nothing else exports the table.
const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const DIST: [f64; 8] = [
    std::f64::consts::SQRT_2,
    1.0,
    std::f64::consts::SQRT_2,
    1.0,
    1.0,
    std::f64::consts::SQRT_2,
    1.0,
    std::f64::consts::SQRT_2,
];

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// The three routings this probe compares. `D8` is the pre-MFD solve, `Uniform`
/// is journal/0109's shipped one, `Hybrid` is this slice.
fn cfg_d8(cells: &CellGrid) -> DeepConfig {
    DeepConfig {
        mfd: false,
        ..production_config(cells, SEED)
    }
}

fn cfg_uniform(cells: &CellGrid, p: f64) -> DeepConfig {
    DeepConfig {
        mfd: true,
        mfd_exponent: p,
        mfd_exponent_channel: p,
        ..production_config(cells, SEED)
    }
}

fn cfg_hybrid(cells: &CellGrid) -> DeepConfig {
    DeepConfig {
        mfd: true,
        ..production_config(cells, SEED)
    }
}

/// **The flow-structure readout of one routing.** Everything `main` prints and
/// everything the gate asserts comes through here, so the report and the test
/// cannot disagree (journal/0103).
struct Structure {
    /// Largest accumulated drainage area on the world, in cells.
    peak_catchment: f64,
    /// Land-cell catchment percentiles: p50, p90, p99, p99.9.
    pct: [f64; 4],
    /// **Concentration ratio** — the share of all land drainage area held by the
    /// top 1 % of land cells. A dispersive solve spreads area evenly and this
    /// falls; a concentrating one raises it. Scale-free (a ratio of sums).
    top1_share: f64,
    /// Peak catchment over the mean land catchment — the other end of the same
    /// question, and the one that reads as "how big is the biggest river".
    peak_over_mean: f64,
    /// Land cells whose catchment exceeds 100 cells — "is there a trunk network
    /// at all", counted rather than eyeballed off a percentile.
    trunk_cells: usize,
    /// `(cell, epoch)` pairs that sent discharge through ≥2 lateral faces **in one
    /// epoch** — concurrent distributaries. Identically zero for D8, forever.
    simul_cell_epochs: u64,
    simul_cell_chapters: u64,
    simul_max_faces: u32,
    /// The temporal (avulsion) kind, all face families — the control.
    temporal_cell_chapters: u64,
    entries: usize,
    record_bytes: usize,
    field_bytes: usize,
    land_cells: usize,
    deep_secs: f64,
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

fn structure(f: &DeepField, deep_secs: f64) -> Structure {
    let mut land: Vec<f64> = Vec::new();
    for (z, a) in f.surf.iter().zip(&f.area) {
        if *z > 0.0 {
            land.push(*a);
        }
    }
    land.sort_by(|a, b| a.partial_cmp(b).expect("finite areas"));
    let total: f64 = land.iter().sum();
    let n = land.len();
    let top1 = (n as f64 * 0.01).ceil() as usize;
    let top1_sum: f64 = land[n.saturating_sub(top1)..].iter().sum();
    let mean = if n == 0 { 1.0 } else { total / n as f64 };
    let c = f.flux.census();
    Structure {
        peak_catchment: f.area.iter().cloned().fold(0.0f64, f64::max),
        pct: [
            percentile(&land, 0.50),
            percentile(&land, 0.90),
            percentile(&land, 0.99),
            percentile(&land, 0.999),
        ],
        top1_share: if total > 0.0 { top1_sum / total } else { 0.0 },
        peak_over_mean: land.last().copied().unwrap_or(0.0) / mean.max(1e-12),
        trunk_cells: land.iter().filter(|&&a| a > 100.0).count(),
        simul_cell_epochs: f.flux.simultaneous.cell_epochs,
        simul_cell_chapters: f.flux.simultaneous.cell_chapters,
        simul_max_faces: f.flux.simultaneous.max_out_faces,
        temporal_cell_chapters: c.divergent as u64,
        entries: c.entries,
        record_bytes: f.flux.resident_bytes(),
        field_bytes: f.resident_bytes(),
        land_cells: n,
        deep_secs,
    }
}

fn measure(cells: &CellGrid, cfg: &DeepConfig) -> Structure {
    let t = Instant::now();
    let f = build_field_cfg(cells, cfg);
    structure(&f, t.elapsed().as_secs_f64())
}

// ---------------------------------------------------------------------------

fn main() {
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t_pregen = t0.elapsed().as_secs_f64();

    let d8 = measure(&pregen.grid, &cfg_d8(&pregen.grid));
    let uni = measure(&pregen.grid, &cfg_uniform(&pregen.grid, 4.0));
    let hy = measure(&pregen.grid, &cfg_hybrid(&pregen.grid));
    let dc = DeepConfig::default();

    println!("=== FLOW continuation (b') — HYBRID p, production world ===");
    println!(
        "seed {SEED} · Extent::Medium · {} land cells of {} · pregen {t_pregen:.1} s",
        hy.land_cells,
        pregen.grid.w * pregen.grid.w
    );
    println!(
        "law: p = {} on unchannelised ground, {} in channels, ramped log-linearly \n\
         on chi = A*S^2 between {:.0e} and {:.0e}   (Montgomery & Dietrich channel \
         initiation)",
        dc.mfd_exponent, dc.mfd_exponent_channel, dc.mfd_chi_lo, dc.mfd_chi_hi
    );

    println!("\n--- ACCEPTANCE 1: PEAK CATCHMENT (cells) ---");
    println!("                              D8   uniform p=4    hybrid p");
    println!(
        "peak catchment        {:>10.0} {:>13.0} {:>11.0}",
        d8.peak_catchment, uni.peak_catchment, hy.peak_catchment
    );
    println!(
        "  recovery of the uniform-p collapse: {:.1} % of the D8 peak \
         (uniform was {:.1} %)",
        hy.peak_catchment * 100.0 / d8.peak_catchment.max(1.0),
        uni.peak_catchment * 100.0 / d8.peak_catchment.max(1.0)
    );

    println!("\n--- ACCEPTANCE 2: THE CATCHMENT DISTRIBUTION (land cells) ---");
    println!("                              D8   uniform p=4    hybrid p");
    for (k, label) in ["p50", "p90", "p99", "p99.9"].into_iter().enumerate() {
        println!(
            "{label:<8} catchment    {:>10.1} {:>13.1} {:>11.1}",
            d8.pct[k], uni.pct[k], hy.pct[k]
        );
    }
    println!(
        "top-1 % share of area {:>10.4} {:>13.4} {:>11.4}",
        d8.top1_share, uni.top1_share, hy.top1_share
    );
    println!(
        "peak / mean catchment {:>10.1} {:>13.1} {:>11.1}",
        d8.peak_over_mean, uni.peak_over_mean, hy.peak_over_mean
    );
    println!(
        "land cells with A>100 {:>10} {:>13} {:>11}",
        d8.trunk_cells, uni.trunk_cells, hy.trunk_cells
    );

    println!("\n--- ACCEPTANCE 3: SIMULTANEOUS DIVERGENCE MUST SURVIVE ---");
    println!("                              D8   uniform p=4    hybrid p");
    println!(
        "(cell,epoch) pairs    {:>10} {:>13} {:>11}",
        d8.simul_cell_epochs, uni.simul_cell_epochs, hy.simul_cell_epochs
    );
    println!(
        "(cell,chapter) pairs  {:>10} {:>13} {:>11}",
        d8.simul_cell_chapters, uni.simul_cell_chapters, hy.simul_cell_chapters
    );
    println!(
        "max faces in an epoch {:>10} {:>13} {:>11}",
        d8.simul_max_faces, uni.simul_max_faces, hy.simul_max_faces
    );
    println!(
        "temporal (avulsion)   {:>10} {:>13} {:>11}",
        d8.temporal_cell_chapters, uni.temporal_cell_chapters, hy.temporal_cell_chapters
    );
    if hy.simul_cell_epochs == 0 {
        println!(
            "\n*** FAIL: hybrid p destroyed simultaneous divergence entirely. \
             Concentrating flow has thrown away the slice that preceded this one. ***"
        );
    } else {
        println!(
            "\nhybrid keeps {:.1} % of uniform-p's simultaneous divergence while \
             recovering {:.1}x its peak catchment.",
            hy.simul_cell_epochs as f64 * 100.0 / uni.simul_cell_epochs.max(1) as f64,
            hy.peak_catchment / uni.peak_catchment.max(1.0)
        );
    }

    // ---- ACCEPTANCE 4: mass, and the per-species budget --------------------
    let t = Instant::now();
    let run = run_cells(&pregen.grid, &cfg_hybrid(&pregen.grid), false);
    let mass_secs = t.elapsed().as_secs_f64();
    let residual = deeptime::total_mass(&run.grid) - run.mass_before - run.uplift_total - run.biotic_total;
    println!("\n--- ACCEPTANCE 4: MASS (hybrid p, scalar run, {mass_secs:.1} s) ---");
    println!(
        "d(sum R + sum H) - uplift - biotic = {residual:+.6e} m   \
         (total mass {:.3e} m)",
        deeptime::total_mass(&run.grid)
    );
    println!(
        "worst PER-SPECIES split residue    = {:.3e}  (relative; 0 = exact)",
        run.erosion.max_species_split_residue()
    );

    // ---- the chi distribution the thresholds were calibrated against -------
    // Re-derived from the live solve's own `filled` potential and its final
    // drainage area — the same two quantities `partition_cell` reads. It is a
    // DIAGNOSTIC of the final epoch, not the value any epoch actually used
    // (the solve reads a one-epoch-lagged area), and it is labelled as such.
    let w = run.grid.w;
    let filled = run.erosion.filled().to_vec();
    let area = run.erosion.area().to_vec();
    let surf: Vec<f64> = run.grid.r.iter().zip(&run.grid.h).map(|(r, h)| r + h).collect();
    let mut chis: Vec<f64> = Vec::new();
    let mut smax_all: Vec<f64> = Vec::new();
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if surf[i] <= 0.0 {
                continue;
            }
            let mut s_max = 0.0f64;
            for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
                let j = (y as i32 + dy) as usize * w + (x as i32 + dx) as usize;
                let drop = filled[i] - filled[j];
                if drop > 0.0 {
                    s_max = s_max.max(drop / DIST[d]);
                }
            }
            if s_max > 0.0 {
                chis.push(area[i] * s_max * s_max);
                smax_all.push(s_max);
            }
        }
    }
    chis.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    smax_all.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    println!("\n--- THE CALIBRATION: chi = A * S^2 over {} draining land cells ---", chis.len());
    println!("(final-epoch diagnostic; the solve reads a one-epoch-lagged area)");
    print!("chi   ");
    for q in [0.01, 0.10, 0.50, 0.75, 0.90, 0.99, 0.999] {
        print!("  p{:<5.1}={:>10.3e}", q * 100.0, percentile(&chis, q));
    }
    println!("  max={:.3e}", chis.last().copied().unwrap_or(0.0));
    print!("S_max ");
    for q in [0.01, 0.10, 0.50, 0.75, 0.90, 0.99, 0.999] {
        print!("  p{:<5.1}={:>10.3e}", q * 100.0, percentile(&smax_all, q));
    }
    println!("  max={:.3e}", smax_all.last().copied().unwrap_or(0.0));
    let below = chis.iter().filter(|&&c| c <= dc.mfd_chi_lo).count();
    let above = chis.iter().filter(|&&c| c >= dc.mfd_chi_hi).count();
    println!(
        "share of land at p_hill (chi<=lo) {:>6.2} %   in the ramp {:>6.2} %   \
         at p_chan (chi>=hi) {:>6.2} %",
        below as f64 * 100.0 / chis.len() as f64,
        (chis.len() - below - above) as f64 * 100.0 / chis.len() as f64,
        above as f64 * 100.0 / chis.len() as f64
    );

    // ---- stubs #22: what the representational floor actually costs ---------
    println!("\n--- STUB #22: the representational floor, MEASURED ---");
    let open = measure(
        &pregen.grid,
        &DeepConfig {
            mfd_min_weight: 0.0,
            ..cfg_hybrid(&pregen.grid)
        },
    );
    println!("                     floor 1 %      floor OFF        delta");
    println!(
        "peak catchment  {:>13.0} {:>14.0} {:>12.1} %",
        hy.peak_catchment,
        open.peak_catchment,
        (hy.peak_catchment - open.peak_catchment) * 100.0 / open.peak_catchment.max(1.0)
    );
    println!(
        "simul (c,epoch) {:>13} {:>14} {:>12.1} %",
        hy.simul_cell_epochs,
        open.simul_cell_epochs,
        (hy.simul_cell_epochs as f64 - open.simul_cell_epochs as f64) * 100.0
            / open.simul_cell_epochs.max(1) as f64
    );
    println!(
        "record entries  {:>13} {:>14} {:>12.1} %",
        hy.entries,
        open.entries,
        (hy.entries as f64 - open.entries as f64) * 100.0 / open.entries.max(1) as f64
    );
    println!(
        "record MiB      {:>13.2} {:>14.2} {:>12.1} %",
        mib(hy.record_bytes),
        mib(open.record_bytes),
        (mib(hy.record_bytes) - mib(open.record_bytes)) * 100.0 / mib(open.record_bytes).max(1e-9)
    );
    println!(
        "\nThe floor is a RECORD-affordability constant applied inside the SOLVE \
         (stubs.md § 22).\nThe rows above are what it buys and what it costs the \
         physics, on the shipped world,\nwhich is the measurement that entry was \
         owed — it previously said 'argued, NOT measured'."
    );

    // ---- cost --------------------------------------------------------------
    println!("\n--- COST ---");
    println!(
        "deep run   D8 {:>7.2} s   uniform {:>7.2} s   hybrid {:>7.2} s   \
         (hybrid - uniform {:+.2} s)",
        d8.deep_secs,
        uni.deep_secs,
        hy.deep_secs,
        hy.deep_secs - uni.deep_secs
    );
    println!(
        "record     D8 {:>7} entries / {:>6.2} MiB   uniform {:>7} / {:>6.2} MiB   \
         hybrid {:>7} / {:>6.2} MiB",
        d8.entries,
        mib(d8.record_bytes),
        uni.entries,
        mib(uni.record_bytes),
        hy.entries,
        mib(hy.record_bytes)
    );
    println!(
        "DeepField  D8 {:>7.2} MiB   uniform {:>7.2} MiB   hybrid {:>7.2} MiB",
        mib(d8.field_bytes),
        mib(uni.field_bytes),
        mib(hy.field_bytes)
    );
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod gate {
    use super::*;

    fn small() -> Pregen {
        Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        })
    }

    /// **The acceptance, at the smallest extent that carries it.**
    ///
    /// *Why the invariants are scale-free.* Both are **orderings between two
    /// solves on the same terrain**, not magnitudes. Concentration is a property
    /// of the partition arithmetic — a larger exponent on the channelised subset
    /// moves share onto the steepest line at *every* cell it applies to, so the
    /// accumulated area at the outlet of any chain can only rise; that is true of
    /// a three-cell chain and of a thousand-cell one. Survival of simultaneous
    /// divergence is a **per-cell existence predicate** ("some cell somewhere
    /// still split its discharge within one epoch"), which needs only that
    /// unchannelised ground exists. **No figure is pinned** — a colleague
    /// calibrating this world should be free to move every number in `main`.
    #[test]
    fn hybrid_p_concentrates_flow_without_destroying_simultaneous_divergence() {
        let pregen = small();
        let uni = measure(&pregen.grid, &cfg_uniform(&pregen.grid, 4.0));
        let hy = measure(&pregen.grid, &cfg_hybrid(&pregen.grid));

        assert!(
            hy.peak_catchment > uni.peak_catchment,
            "hybrid p did not concentrate: peak catchment {} vs uniform's {}",
            hy.peak_catchment,
            uni.peak_catchment
        );
        assert!(
            hy.top1_share > uni.top1_share,
            "hybrid p spread drainage more evenly than uniform p ({} vs {}) — the \
             channel exponent is not reaching the channels",
            hy.top1_share,
            uni.top1_share
        );
        assert!(
            hy.simul_cell_epochs > 0,
            "hybrid p destroyed simultaneous divergence — deltas are unrepresentable \
             again and the slice before this one has been thrown away"
        );
        assert!(hy.simul_max_faces >= 2);
    }

    /// **Determinism under the lagged read.** The exponent sees the *previous*
    /// epoch's drainage area, which is exactly the kind of loop-carried coupling
    /// that can introduce an order dependence — and the routing phase is data-
    /// parallel, so a read that were not genuinely read-only would show up here
    /// and nowhere else. (`MfdParams::uniform`'s exact recovery of journal/0109's
    /// solve is asserted in `erosion::mfd_tests` on a hand-built patch, where it
    /// costs no deep run.)
    ///
    /// Scale-free: an equality between two runs of the same code on the same
    /// world.
    #[test]
    fn the_hybrid_solve_is_deterministic_under_its_lagged_area_read() {
        let pregen = small();
        let h1 = measure(&pregen.grid, &cfg_hybrid(&pregen.grid));
        let h2 = measure(&pregen.grid, &cfg_hybrid(&pregen.grid));
        assert_eq!(
            h1.peak_catchment, h2.peak_catchment,
            "the hybrid solve is not deterministic — the lagged-area read has \
             introduced a dependency on something other than (cells, cfg)"
        );
        assert_eq!(h1.simul_cell_epochs, h2.simul_cell_epochs);
        assert_eq!(h1.entries, h2.entries);
    }
}
