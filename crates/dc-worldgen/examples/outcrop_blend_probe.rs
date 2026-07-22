//! **The S-4 contour check for the share-weighted susceptibility blend**
//! (journal/0072, audit site A1).
//!
//! Over the production Medium deep-time record it builds two per-cell abrasion
//! rate fields for the fluvial agent:
//!
//! - **old (argmax)** — `sus_tab[exposed_litho.index()]`, the pre-blend rule: one
//!   lithology's rate, whichever dominates the window;
//! - **new (blend)** — `blend_susceptibility(exposed_shares, sus_tab)`, the
//!   share-weighted rate.
//!
//! It then reports, over 4-neighbour adjacencies:
//!
//! - the **max adjacent-cell rate jump** under each rule (the coherent S-4 step the
//!   old rule could print, vs what the blend leaves);
//! - a **histogram of the adjacent rate jump at former flip boundaries** — the
//!   cells where the argmax verdict differs from a neighbour, i.e. exactly where the
//!   old rule stepped — under both rules, so the collapse of the step is visible;
//! - the **per-epoch table-refresh cost**, argmax vs blend, timed over the whole
//!   grid (the hot path this slice touches).
//!
//! Run: `cargo run --release -p dc-worldgen --example outcrop_blend_probe`

use std::time::Instant;

use dc_worldgen::deeptime::lithology::{
    Agent, Litho, blend_susceptibility, dominant_litho, exposed_litho, exposed_shares,
    susceptibility_table,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;
// Production abrasion knobs (`DeepConfig::default`): the fluvial contrast and the
// stability cap the flow susceptibility table is built with.
const CONTRAST: f64 = 2.5;
const CAP: f64 = 5.0;

fn main() {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let deep = &pregen.deep;
    let w = deep.w;
    let n = deep.strata.len();
    let tab = susceptibility_table(Agent::Abrasion, CONTRAST, CAP);

    // Per-cell rate fields and the argmax verdict (for locating former flips).
    let mut old_rate = vec![0.0f64; n];
    let mut new_rate = vec![0.0f64; n];
    let mut verdict = vec![Litho::Basement; n];
    for (i, s) in deep.strata.iter().enumerate() {
        let units = s.units.as_slice();
        let shares = exposed_shares(units);
        let l = exposed_litho(units);
        verdict[i] = l;
        old_rate[i] = tab[l.index()];
        new_rate[i] = blend_susceptibility(&shares, &tab);
        // Sanity: the blend's argmax is the verdict on this record.
        debug_assert_eq!(dominant_litho(&shares), l);
    }

    // Adjacency sweep (east + south pairs — each interior edge once).
    let mut max_old_jump = 0.0f64;
    let mut max_new_jump = 0.0f64;
    // Histogram buckets for the adjacent jump at former flip boundaries.
    let edges: [f64; 6] = [0.0, 0.05, 0.1, 0.25, 0.5, 1.0];
    let mut hist_old = [0usize; 7];
    let mut hist_new = [0usize; 7];
    let mut flip_edges = 0usize;
    let bucket = |v: f64| -> usize {
        for (k, &e) in edges.iter().enumerate() {
            if v < e {
                return k;
            }
        }
        edges.len()
    };
    let mut consider = |i: usize, j: usize, max_old: &mut f64, max_new: &mut f64| {
        let dj_old = (old_rate[i] - old_rate[j]).abs();
        let dj_new = (new_rate[i] - new_rate[j]).abs();
        *max_old = max_old.max(dj_old);
        *max_new = max_new.max(dj_new);
        if verdict[i] != verdict[j] {
            flip_edges += 1;
            hist_old[bucket(dj_old)] += 1;
            hist_new[bucket(dj_new)] += 1;
        }
    };
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            if gx + 1 < w {
                consider(i, i + 1, &mut max_old_jump, &mut max_new_jump);
            }
            if gy + 1 < w {
                consider(i, i + w, &mut max_old_jump, &mut max_new_jump);
            }
        }
    }

    println!("production Medium record: {n} cells ({w}×{w})");
    println!("max adjacent-cell rate jump (fluvial abrasion, contrast {CONTRAST}, cap {CAP}):");
    println!("  OLD (argmax): {max_old_jump:.4}");
    println!("  NEW (blend):  {max_new_jump:.4}");
    println!(
        "former flip boundaries (adjacent cells whose argmax verdict differs): {flip_edges} edges"
    );
    println!("adjacent rate-jump histogram at those boundaries (jump in [lo,hi)):");
    let label = |k: usize| -> String {
        let lo = if k == 0 { 0.0 } else { edges[k - 1] };
        if k < edges.len() {
            format!("[{lo:.2},{:.2})", edges[k])
        } else {
            format!("[{lo:.2},inf)")
        }
    };
    println!("{:>14}{:>12}{:>12}", "bucket", "OLD", "NEW");
    for k in 0..=edges.len() {
        println!("{:>14}{:>12}{:>12}", label(k), hist_old[k], hist_new[k]);
    }

    // ---- per-epoch table-refresh cost: argmax lookup vs share blend ----
    // Reproduce exactly what `expose` does per cell, over the whole grid, K times.
    const K: u32 = 20;
    let mut sink = 0.0f64;
    let t0 = Instant::now();
    for _ in 0..K {
        for s in &deep.strata {
            let l = exposed_litho(s.units.as_slice());
            sink += tab[l.index()];
        }
    }
    let argmax_ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(K);
    let t1 = Instant::now();
    for _ in 0..K {
        for s in &deep.strata {
            let shares = exposed_shares(s.units.as_slice());
            sink += blend_susceptibility(&shares, &tab);
        }
    }
    let blend_ms = t1.elapsed().as_secs_f64() * 1e3 / f64::from(K);
    println!(
        "\nper-epoch table refresh over {n} cells (mean of {K}):\n  \
         argmax lookup: {argmax_ms:.3} ms\n  share blend:   {blend_ms:.3} ms  \
         (×{:.2})",
        blend_ms / argmax_ms.max(1e-9)
    );
    // Keep the optimiser honest.
    if sink == f64::INFINITY {
        println!("(unreachable) {sink}");
    }
}
