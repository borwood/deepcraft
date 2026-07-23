//! The guided-tour map for the **erosion-budget amplitude** walk (journal/0076).
//!
//! journal/0076 shipped `--erosion-budget <mult>` so the standing amplitude call
//! from journal/0029 could finally be *seen*. This example does two jobs on the
//! world the dc-client actually boots (`BENCH_SEED` = 1337, `Extent::Medium`,
//! production config):
//!
//!  1. **The response curve** — sweep the budget 1× / 3× / 10× / 30× and report
//!     how much the surface actually moves. This is the faithfulness check *and*
//!     the finding: if the landscape is uplift-dominated, even 30× may barely
//!     dent it (journal/0076's own diagnosis: the whole landscape removes only a
//!     few metres against hundreds of metres of uplift), and the honest answer to
//!     "which budget do we ship" changes shape.
//!  2. **The stations** — the cells where the budget bit hardest, with world
//!     coordinates and the surface at each budget, so a LIVE co-walk is
//!     station-driven.
//!
//! **Faithfulness is proven, not assumed.** Every budget goes through the exact
//! launch path — `build_field_with(grid, seed, DeepOverrides{ erosion_budget })`,
//! which is what `--erosion-budget N` boots — and the 1× field is asserted
//! byte-identical to the shipped `build_field`. So a flat response curve is a real
//! saturation result, never an instrument artifact (the journal/0030 trap).
//!
//! `cargo run --release -p dc-worldgen --example amplitude_tour`

use std::time::Instant;

use dc_worldgen::deeptime::{
    Agent, DeepField, DeepOverrides, Litho, build_field, build_field_with, exposed_litho,
    susceptibility_table,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;
/// The budget sweep. 1× is the shipped world (faithfulness anchor); the walk
/// drives whichever pair the curve says is legible.
const MULTS: [f64; 4] = [1.0, 3.0, 10.0, 30.0];
/// Border ring (deep cells) excluded from station picks — the erosion march
/// piles artifacts on the last column (tour_map.rs § EDGE_MARGIN).
const EDGE_MARGIN: i64 = 4;

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

fn field_for(pregen: &Pregen, mult: f64) -> DeepField {
    build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            erosion_budget: Some(mult),
            ..DeepOverrides::default()
        },
    )
}

fn main() {
    println!("=== journal/0076 amplitude-tour map: erosion-budget sweep ===");
    println!(
        "seed {SEED}, extent {}, production config, N=2 ({VOXEL_M} m voxels)\n",
        EXTENT.label()
    );

    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    eprintln!("pregen built in {:?}", t0.elapsed());

    // Faithfulness anchor: the 1× launch path must reproduce the shipped world.
    let shipped = build_field(&pregen.grid, SEED);
    let fields: Vec<DeepField> = MULTS
        .iter()
        .map(|&m| {
            let t = Instant::now();
            let f = field_for(&pregen, m);
            eprintln!("  built budget {m}× field in {:?}", t.elapsed());
            f
        })
        .collect();
    assert_eq!(
        shipped.surf, fields[0].surf,
        "FAITHFULNESS BROKEN: 1x budget did not reproduce the shipped surface"
    );
    println!("faithfulness: 1× budget == shipped build_field (surface byte-identical) ✓");

    let w = fields[0].w;
    let conv = Conv {
        w,
        wp: pregen.deep.wp,
    };
    println!(
        "deep grid {w}x{w} @ {:.1} m/cell; pregen {}x{}\n",
        fields[0].cell_m, conv.wp, conv.wp
    );

    response_curve(&fields);
    by_litho(&fields);
    stations(&fields, &conv);
}

// ---------------------------------------------------------------- surface reads

fn surf(f: &DeepField, i: usize) -> f64 {
    f.surf[i]
}

fn exposure(f: &DeepField) -> Vec<Litho> {
    (0..f.w * f.w)
        .map(|i| exposed_litho(f.strata.get(i).map_or(&[][..], |s| s.units.as_slice())))
        .collect()
}

fn relief(f: &DeepField) -> f64 {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for i in 0..f.w * f.w {
        let s = surf(f, i);
        if s > 0.0 {
            lo = lo.min(s);
            hi = hi.max(s);
        }
    }
    hi - lo
}

fn local_relief(f: &DeepField, idx: usize, radius: i64) -> f64 {
    let w = f.w as i64;
    let (cx, cy) = ((idx % f.w) as i64, (idx / f.w) as i64);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= w || y >= w {
                continue;
            }
            let s = surf(f, (y * w + x) as usize);
            lo = lo.min(s);
            hi = hi.max(s);
        }
    }
    hi - lo
}

// ---------------------------------------------------------------- response curve

/// The headline: how much does the surface actually move as the budget climbs?
/// Everything is measured against the 1× (shipped) field, so the reader sees the
/// response shape — linear, saturating, or flat.
fn response_curve(fields: &[DeepField]) {
    let base = &fields[0];
    let w = base.w;
    println!("--- response curve (all vs 1× shipped) ---");
    println!(
        "  {:>5}  {:>9}  {:>12}  {:>12}  {:>10}",
        "mult", "relief m", "mean|Δ| vs1×", "max|Δ| vs1×", "cells>1m"
    );
    for (k, f) in fields.iter().enumerate() {
        let (mut sum, mut n, mut mx, mut over) = (0.0f64, 0usize, 0.0f64, 0usize);
        for i in 0..w * w {
            if surf(base, i) > 0.0 && surf(f, i) > 0.0 {
                let d = (surf(f, i) - surf(base, i)).abs();
                sum += d;
                mx = mx.max(d);
                if d > 1.0 {
                    over += 1;
                }
                n += 1;
            }
        }
        println!(
            "  {:>4}×  {:>9.0}  {:>12.2}  {:>12.1}  {:>10}",
            MULTS[k],
            relief(f),
            sum / n.max(1) as f64,
            mx,
            over
        );
    }
    println!();
}

/// Mean land elevation per outcropping lithology across the sweep — the
/// differential-erosion signature. Widening separation between hard (fine/coarse)
/// and soft (peat/soil) as the budget climbs is the "hard stands proud" story.
fn by_litho(fields: &[DeepField]) {
    let top = fields.last().unwrap();
    let ex = exposure(top);
    println!("--- mean land elevation by outcropping lithology (per budget) ---");
    print!("    {:<9}", "litho");
    for m in MULTS {
        print!("  {:>8}", format!("{m}×"));
    }
    println!();
    for l in Litho::ALL {
        let cells: Vec<usize> = (0..top.w * top.w)
            .filter(|&i| ex[i] == l && surf(top, i) > 0.0)
            .collect();
        if cells.is_empty() {
            println!("    {:<9}  —", l.code());
            continue;
        }
        print!("    {:<9}", l.code());
        for f in fields {
            let n = cells.len() as f64;
            let e = cells.iter().map(|&i| surf(f, i)).sum::<f64>() / n;
            print!("  {e:>8.0}");
        }
        println!("   ({} cells)", cells.len());
    }
    println!();
}

// ---------------------------------------------------------------- coordinates

struct Conv {
    w: usize,
    wp: usize,
}

impl Conv {
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

    fn where_line(&self, idx: usize) -> String {
        let (gx, gy) = (idx % self.w, idx / self.w);
        let (vx, vz) = self.idx_to_voxel(idx);
        format!(
            "deep cell ({gx},{gy}) | voxel ({vx}, {vz}) | world ({:.0} m, {:.0} m)",
            vx as f64 * VOXEL_M,
            vz as f64 * VOXEL_M
        )
    }
}

// ---------------------------------------------------------------- stations

/// Stations are the cells where the budget bit hardest (max |Δsurf| 1×→top), so a
/// walker stands where there is something to judge. For each, the surface at every
/// budget and the outcropping rock — plus the massif crest as the landform-scale
/// negative exhibit.
fn stations(fields: &[DeepField], conv: &Conv) {
    let base = &fields[0];
    let top = fields.last().unwrap();
    let w = top.w;
    let ex = exposure(top);
    let surf_line = |i: usize| -> String {
        fields
            .iter()
            .enumerate()
            .map(|(k, f)| format!("{}× {:>7.1}m", MULTS[k] as i64, surf(f, i)))
            .collect::<Vec<_>>()
            .join("  ")
    };

    // The single cell that moved the most from 1× to the top budget.
    let mover = (0..w * w)
        .filter(|&i| interior(w, i) && surf(base, i) > 0.0 && surf(top, i) > 0.0)
        .max_by(|&i, &j| {
            (surf(base, i) - surf(top, i))
                .abs()
                .total_cmp(&(surf(base, j) - surf(top, j)).abs())
        });
    header("STATION A — WHERE THE BUDGET BIT HARDEST");
    if let Some(a) = mover {
        println!("  stand at: {}", conv.where_line(a));
        println!("    outcrop [{}]  {}", ex[a].code(), surf_line(a));
        println!(
            "    1×→{}× moved {:.1} m",
            MULTS.last().unwrap(),
            surf(base, a) - surf(top, a)
        );
    } else {
        println!("  NULL: no land moved.");
    }

    // The sharpest hard-over-soft contact at the top budget, ranked by proudness
    // GAIN from 1× to top.
    let tab = susceptibility_table(Agent::Abrasion, 2.5, 5.0);
    let mut proud: Option<(f64, usize, usize)> = None;
    for gy in 1..w - 1 {
        for gx in 1..w - 1 {
            let i = gy * w + gx;
            let j = gy * w + gx + 1;
            if !interior(w, i) || surf(top, i) <= 0.0 || surf(top, j) <= 0.0 || ex[i] == ex[j] {
                continue;
            }
            let (a, b) = if tab[ex[i].index()] < tab[ex[j].index()] {
                (i, j)
            } else {
                (j, i)
            };
            let gain = (surf(top, a) - surf(top, b)) - (surf(base, a) - surf(base, b));
            if surf(top, a) > surf(top, b) && proud.is_none_or(|(g, _, _)| gain > g) {
                proud = Some((gain, a, b));
            }
        }
    }
    header("STATION B — HARD BED STANDING PROUD (the differential signature)");
    if let Some((gain, a, b)) = proud {
        println!("  stand at HARD cell: {}", conv.where_line(a));
        println!("    hard [{}]  {}", ex[a].code(), surf_line(a));
        println!("    soft [{}]  {}", ex[b].code(), surf_line(b));
        println!(
            "    step (hard−soft): {:+.1} m (1×) → {:+.1} m ({}×);  hard bed gains {gain:.1} m",
            surf(base, a) - surf(base, b),
            surf(top, a) - surf(top, b),
            MULTS.last().unwrap()
        );
    } else {
        println!("  NULL: no hard-over-soft contact.");
    }

    // The highest-relief massif: does amplitude add landform-scale shape?
    let massif = (0..w * w)
        .filter(|&i| interior(w, i) && surf(top, i) > 0.0)
        .max_by(|&i, &j| local_relief(top, i, 3).total_cmp(&local_relief(top, j, 3)));
    header("STATION C — MASSIF CREST (landform-scale negative exhibit)");
    if let Some(a) = massif {
        println!("  stand at: {}", conv.where_line(a));
        println!("    outcrop [{}]  {}", ex[a].code(), surf_line(a));
        print!("    local relief (±3):");
        for f in fields {
            print!("  {:.0}m", local_relief(f, a, 3));
        }
        println!();
        println!(
            "  SEE: cranked erosion re-terraces flanks but cannot ADD a range where the deep field holds a plateau (S13/0041)."
        );
    } else {
        println!("  NULL: no land.");
    }
}

fn header(title: &str) {
    println!("\n============================================================");
    println!("{title}");
    println!("============================================================");
}
