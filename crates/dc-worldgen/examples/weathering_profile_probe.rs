//! **The weathering-front PROFILE probe** (journal/0099, the follow-up
//! journal/0097's walk opened).
//!
//! Movement 3's saprolite band was correctly *magnituded* and wrongly *shaped*:
//! the fold read the scalar `FactLedger::weathering_product_m` and emplaced one
//! stratum of one class at that thickness, which the record→voxel path expressed
//! as `Single` — 8/8 product, pure, against contents-free basement. A hard
//! perimeter, which is not what a weathering front is.
//!
//! This probe reads the **production** world (the tour's seed/extent, the very
//! world `--weather-inventory` boots) at the cell with the thickest band and
//! prints the front **voxel by voxel, top to bottom**: the fill plan, the
//! structure / pore-fill / debris eighths, and the parent-vs-product split. It
//! is the acceptance instrument for three claims a unit test on a fixture cannot
//! make (A-3):
//!
//! 1. the gradient is **real and multi-voxel**, not the one-voxel
//!    `mixed_voxel_contents` boundary-quantization artifact (journal/0055) that
//!    appears wherever *any* two units meet — so the per-voxel **plan kind** is
//!    printed beside every row, and a `Single` row's composition is the record's
//!    own, not a straddle;
//! 2. the profile's **depth scales with the model's own magnitude** — a second,
//!    weaker station is probed for exactly that contrast (A-1: driven by the
//!    model, not decoration);
//! 3. **mass is conserved**: the product eighths summed down the column return
//!    the ledger's metres.
//!
//! `cargo run --release -p dc-worldgen --example weathering_profile_probe`

use std::collections::HashMap;

use dc_core::materials::geology::{CLASS_CLASTIC_FINE, vanilla, vanilla_members};
use dc_core::{ChunkPos, MaterialId, VoxelContents};
use dc_worldgen::collapse::WorldGenerator;
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{ColumnFill, DeepOverrides, Plan};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;
/// Border ring excluded from station picks (march edge artifacts).
const EDGE_MARGIN: i64 = 4;

/// World voxel centre of deep cell `idx` (inverse of `DeepField::deep_coords`,
/// lifted from `examples/s18_weathering_tour.rs`).
fn idx_to_voxel(w: usize, wp: usize, idx: usize) -> (i64, i64) {
    let (wf, wpf) = (w as f64, wp as f64);
    let (gx, gy) = ((idx % w) as f64, (idx / w) as f64);
    let px = (gx + 0.5) / wf * wpf - 0.5;
    let py = (gy + 0.5) / wf * wpf - 0.5;
    let half = (wp / 2) as f64;
    (
        ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
    )
}

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

/// Material id → member id, so the dump names rock rather than `M14`.
fn material_names() -> HashMap<MaterialId, String> {
    vanilla_members()
        .into_iter()
        .map(|m| (m.material, m.id))
        .collect()
}

fn tally(names: &HashMap<MaterialId, String>, slots: &[MaterialId]) -> String {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for m in slots {
        let name = names
            .get(m)
            .cloned()
            .unwrap_or_else(|| format!("{m:?}"))
            .replace("dc:geo/", "");
        match counts.iter_mut().find(|(n, _)| *n == name) {
            Some((_, c)) => *c += 1,
            None => counts.push((name, 1)),
        }
    }
    if counts.is_empty() {
        return "-".into();
    }
    counts
        .iter()
        .map(|(n, c)| format!("{n} {c}/8"))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Product metres a column actually expresses in its voxels, and the ledger
/// metres it owes — the population form of the mass check. Per voxel the
/// expression is an *unbiased estimator* of the record (the eighths are drawn,
/// journal/0055), so one column carries quantization noise; over many columns
/// the two totals must agree.
fn expressed_vs_owed(pregen: &Pregen, cell: usize) -> Option<(f64, f64)> {
    let set = vanilla();
    let (w, wp) = (pregen.deep.w, pregen.deep.wp);
    let (vx, vz) = idx_to_voxel(w, wp, cell);
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    let mut generator = WorldGenerator::new(pregen);
    let rec = generator.column_record(cx, cz);
    let h = i64::from(rec.heights[lz * 32 + lx]);
    let fill = ColumnFill::build(&rec.strata, VOXEL_M);
    let front: Vec<usize> = (0..rec.strata.events.len())
        .filter(|&i| {
            rec.strata.events[i]
                .accessory
                .is_some_and(|(m, _)| set.member(m).class.as_str() == CLASS_CLASTIC_FINE)
        })
        .collect();
    if front.is_empty() {
        return None;
    }
    let owed: f64 = front
        .iter()
        .filter_map(|&i| {
            rec.strata.events[i]
                .accessory
                .map(|(_, k)| f64::from(rec.strata.events[i].thickness_m) * f64::from(k) / 8.0)
        })
        .sum();
    let product_mat = rec.strata.events[front[0]]
        .accessory
        .map(|(m, _)| set.member(m).material)?;
    let mut grids: HashMap<i64, Option<dc_core::ContentsGrid>> = HashMap::new();
    let mut eighths = 0u32;
    for p in 1..=fill.depth_count() {
        let in_front = match fill.plan(p as u32) {
            Some(Plan::Single(k)) => front.contains(k),
            Some(Plan::Mixed(ws)) => ws.iter().any(|(k, _)| front.contains(k)),
            None => false,
        };
        if !in_front {
            continue;
        }
        let vy = h - (p as i64 - 1);
        let cy = vy.div_euclid(32);
        let grid = grids.entry(cy).or_insert_with(|| {
            generator.chunk_contents(ChunkPos {
                x: cx as i32,
                y: cy as i32,
                z: cz as i32,
            })
        });
        let c = grid.as_ref().map_or(VoxelContents::EMPTY, |g| {
            g.get(lx, vy.rem_euclid(32) as usize, lz)
        });
        for slots in [c.structure(), c.pore_fill(), c.debris()] {
            eighths += slots.iter().filter(|m| **m == product_mat).count() as u32;
        }
    }
    Some((f64::from(eighths) / 8.0 * VOXEL_M, owed))
}

/// One probed column: the front, voxel by voxel.
fn probe(pregen: &Pregen, label: &str, cell: usize, band_m: f64) {
    let names = material_names();
    let set = vanilla();
    let (w, wp) = (pregen.deep.w, pregen.deep.wp);
    let (vx, vz) = idx_to_voxel(w, wp, cell);
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);

    let mut generator = WorldGenerator::new(pregen);
    let rec = generator.column_record(cx, cz);
    let h = rec.heights[lz * 32 + lx];
    let fill = ColumnFill::build(&rec.strata, VOXEL_M);

    // The front's events are those carrying a **loose** pore-slot rider — the
    // weathering product. (The sparse igneous inclusion rides the same slot but
    // is a mineral, not a product; without this test the 96-voxel basement body
    // reads as "front".)
    let front: Vec<usize> = (0..rec.strata.events.len())
        .filter(|&i| {
            rec.strata.events[i]
                .accessory
                .is_some_and(|(m, _)| set.member(m).class.as_str() == CLASS_CLASTIC_FINE)
        })
        .collect();
    let product_mat = front
        .first()
        .and_then(|&i| rec.strata.events[i].accessory)
        .map(|(m, _)| set.member(m).material);
    let parent_mat = front.first().map(|&i| set.member(rec.strata.events[i].member).material);

    println!("\n================ {label} ================");
    println!(
        "deep cell {cell} ({},{}) | voxel ({vx}, {vz}) | world ({:.0} m, {:.0} m)",
        cell % w,
        cell / w,
        vx as f64 * VOXEL_M,
        vz as f64 * VOXEL_M
    );
    println!(
        "ledger weathering_product_m = {band_m:.3} m ({:.2} voxels of PURE product — what the \
         pre-0099 slab expressed)",
        band_m / VOXEL_M
    );
    println!(
        "surface voxel y = {h}; record depth {} voxels; {} front bands, {:.3} m each \
         (front {:.2} m = {:.1} voxels)",
        fill.depth_count(),
        front.len(),
        front
            .first()
            .map_or(0.0, |&i| f64::from(rec.strata.events[i].thickness_m)),
        front
            .iter()
            .map(|&i| f64::from(rec.strata.events[i].thickness_m))
            .sum::<f64>(),
        front
            .iter()
            .map(|&i| f64::from(rec.strata.events[i].thickness_m))
            .sum::<f64>()
            / VOXEL_M,
    );
    if let (Some(p), Some(q)) = (parent_mat, product_mat) {
        println!(
            "parent (retained structure) = {}; product (pore fill / debris) = {}",
            names.get(&p).cloned().unwrap_or_default(),
            names.get(&q).cloned().unwrap_or_default()
        );
    }

    // Per-voxel contents, straight off the public generation path.
    let mut grids: HashMap<i64, Option<dc_core::ContentsGrid>> = HashMap::new();
    let contents_at = |generator: &mut WorldGenerator,
                       grids: &mut HashMap<i64, Option<dc_core::ContentsGrid>>,
                       vy: i64|
     -> VoxelContents {
        let cy = vy.div_euclid(32);
        let grid = grids
            .entry(cy)
            .or_insert_with(|| {
                generator.chunk_contents(ChunkPos {
                    x: cx as i32,
                    y: cy as i32,
                    z: cz as i32,
                })
            });
        grid.as_ref().map_or(VoxelContents::EMPTY, |g| {
            g.get(lx, vy.rem_euclid(32) as usize, lz)
        })
    };

    println!(
        "\n  {:>6}  {:>7}  {:<26}  {:<26}  {:<26}",
        "voxel", "plan", "structure", "pore_fill", "debris"
    );
    // Which voxels the front owns, read off the fill plans themselves — a plan
    // is "front" when any event it draws from is a front band. (A `Mixed` plan at
    // the very top of the front is the boundary-quantization voxel; it is labeled
    // `Mixed` in the dump precisely so it cannot be mistaken for the gradient.)
    let is_front_plan = |p: usize| match fill.plan(p as u32) {
        Some(Plan::Single(k)) => front.contains(k),
        Some(Plan::Mixed(ws)) => ws.iter().any(|(k, _)| front.contains(k)),
        None => false,
    };
    let first_plan = (1..=fill.depth_count())
        .find(|&p| is_front_plan(p))
        .unwrap_or(1);
    let last_plan = (1..=fill.depth_count())
        .rev()
        .find(|&p| is_front_plan(p))
        .unwrap_or(fill.depth_count());
    // Print from a few voxels above the front's top down to a few below its base.
    let lo = first_plan.saturating_sub(3).max(1);
    let hi = (last_plan + 3).min(fill.depth_count());
    let mut product_eighths = 0u32;
    let mut parent_eighths = 0u32;
    let mut rows: Vec<(i64, u8)> = Vec::new();
    for p in lo..=hi {
        let vy = h as i64 - (p as i64 - 1);
        let plan = match fill.plan(p as u32) {
            Some(Plan::Single(_)) => "Single",
            Some(Plan::Mixed(_)) => "Mixed",
            None => "-",
        };
        let c = contents_at(&mut generator, &mut grids, vy);
        let is_front = p >= first_plan && p <= last_plan;
        let mut prod = 0u8;
        for slots in [c.structure(), c.pore_fill(), c.debris()] {
            for m in slots {
                if Some(*m) == product_mat {
                    prod += 1;
                    if is_front {
                        product_eighths += 1;
                    }
                } else if Some(*m) == parent_mat && is_front {
                    parent_eighths += 1;
                }
            }
        }
        if is_front {
            rows.push((vy, prod));
        }
        println!(
            "  {vy:>6}  {plan:>7}  {:<26}  {:<26}  {:<26}{}",
            tally(&names, c.structure()),
            tally(&names, c.pore_fill()),
            tally(&names, c.debris()),
            if is_front { "  <- front" } else { "" }
        );
    }

    // Mass: the product eighths summed down the front, back in metres.
    let expressed_m = f64::from(product_eighths) / 8.0 * VOXEL_M;
    println!(
        "\n  product expressed over the front: {product_eighths} eighths = {expressed_m:.3} m \
         (ledger {band_m:.3} m, {:+.1} %)",
        100.0 * (expressed_m - band_m) / band_m
    );
    println!("  parent structure retained inside the front: {parent_eighths} eighths");
    // The gradient, as a one-line shape (top → bottom).
    let shape: Vec<String> = rows.iter().map(|(_, k)| format!("{k}")).collect();
    println!("  product eighths, top → bottom: [{}]", shape.join(", "));
    let strictly_multi_voxel = rows.len() > 1 && rows.iter().any(|(_, k)| *k > 0 && *k < 8);
    println!(
        "  front spans {} voxels; graded (some voxel neither 0/8 nor 8/8): {}",
        rows.len(),
        strictly_multi_voxel
    );
    if let Some((_, deepest)) = rows.last() {
        println!(
            "  DEEPEST front voxel: {deepest}/8 product, {}/8 parent — the bottom contact",
            8 - deepest
        );
    }
}

fn main() {
    println!("=== weathering-front PROFILE probe (journal/0099) ===");
    println!("seed {SEED}, extent {}, weather_inventory ON\n", EXTENT.label());
    let pregen = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides {
            weather_inventory: Some(true),
            ..DeepOverrides::default()
        },
    );
    let w = pregen.deep.w;
    let band = |i: usize| -> f64 {
        pregen
            .deep
            .ledgers
            .get(i)
            .map_or(0.0, |l| l.weathering_product_m(pregen.deep.strata[i].units.len()))
    };
    let mut banded: Vec<(usize, f64)> = (0..w * w)
        .filter(|&i| interior(w, i) && band(i) > 1e-6)
        .map(|i| (i, band(i)))
        .collect();
    banded.sort_by(|a, b| b.1.total_cmp(&a.1));
    if banded.is_empty() {
        println!("NULL: no weathering band anywhere.");
        return;
    }
    println!(
        "banded cells: {} / {}; max {:.2} m, median {:.2} m",
        banded.len(),
        w * w,
        banded[0].1,
        banded[banded.len() / 2].1
    );

    // (1) The argmax cell — the acceptance profile.
    probe(&pregen, "STATION A — argmax band", banded[0].0, banded[0].1);
    // (2) A weaker cell — the same shape at a different magnitude, which is what
    //     tells a model-driven profile from a decorative constant gradient.
    let mid = banded[banded.len() / 2];
    probe(&pregen, "STATION B — median band", mid.0, mid.1);

    // (3) The population mass check: one column's eighths are a draw, so the
    //     claim that the profile *conserves* the ledger's product is a claim
    //     about the estimator, not about one voxel column.
    println!("\n================ POPULATION — mass conservation ================");
    let stride = banded.len() / 32;
    let (mut sum_expressed, mut sum_owed, mut n) = (0.0f64, 0.0f64, 0usize);
    let mut worst_abs = 0.0f64;
    for &(cell, _) in banded.iter().step_by(stride.max(1)).take(32) {
        let Some((expressed, owed)) = expressed_vs_owed(&pregen, cell) else {
            continue;
        };
        sum_expressed += expressed;
        sum_owed += owed;
        n += 1;
        if (expressed - owed).abs() > worst_abs.abs() {
            worst_abs = expressed - owed;
        }
    }
    println!(
        "  {n} columns across the band distribution: expressed {sum_expressed:.3} m, \
         record owes {sum_owed:.3} m ({:+.2} %); worst single column {worst_abs:+.3} m \
         (the eighth quantum is {:.3} m, so a sub-voxel front is all quantum)",
        100.0 * (sum_expressed - sum_owed) / sum_owed,
        VOXEL_M / 8.0
    );
    println!(
        "  (the record itself owes exactly the ledger — proven by \
         `the_weathering_front_conserves_the_ledger_product_mass`; this is the \
         voxel expression's own bias.)"
    );
}
