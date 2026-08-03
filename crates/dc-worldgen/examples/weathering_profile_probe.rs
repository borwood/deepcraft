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

use rayon::prelude::*;

use dc_core::materials::geology::{CLASS_CLASTIC_FINE, vanilla, vanilla_members};
use dc_core::{ChunkPos, MaterialId, VoxelContents};
use dc_worldgen::collapse::WorldGenerator;
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{DeepOverrides, Plan};

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

/// One column's product-mass accounting, **decomposed into its two stages** —
/// the question journal/0103 exists to settle (ROADMAP Observed: is the front's
/// voxel-tier mass error sampling noise or a real upward bias?).
///
/// A single "expressed vs owed" percentage cannot answer that, because two
/// entirely different quantizers sit between the record and the voxels and they
/// have different error laws:
///
/// 1. **GEOMETRY.** [`ColumnFill::build`] slices the record into whole voxels
///    and the record's bottom lands mid-voxel. The last voxel is claimed
///    round-to-nearest and its shares are then **renormalized over the covered
///    part** — so a claimed bottom voxel expresses a *full* voxel of a band the
///    record only partly covers. The weathering front is at the very bottom of
///    the record, so this error lands on the front every time, never on anything
///    else. `plan_m` is what the geometry hands the front's bands before a
///    single eighth is drawn.
/// 2. **THE DRAW.** Inside each voxel the eighths are allocated by the addressed
///    stochastic rounding of `crate::fill` and, for the pore rider, by
///    `pore_rider_share`. `expressed_m − plan_m` isolates *that* error alone.
///
/// Splitting them is the whole diagnosis: stage 2 is the estimator journal/0055's
/// doctrine is about, and stage 1 is not an estimator at all.
struct ColumnMass {
    /// `Σ_bands thickness_m · k/8` — the metres the record owes.
    owed_m: f64,
    /// `Σ_bands (voxel-metres the fill plan gives that band) · k/8`.
    plan_m: f64,
    /// Product eighths counted in the voxels, back in metres.
    expressed_m: f64,
    /// `plan_m` restricted to **attributable** voxels — those whose plan holds no
    /// *non-front* event made of the product's own material.
    ///
    /// **This restriction is the whole reason journal/0099's +3.7 % was not a
    /// diagnosis.** The front's product is `CLASS_CLASTIC_FINE` (mudstone, in
    /// this world) and so is much of the sediment pile sitting directly on top of
    /// it: at 247 production columns, **222 of them** have an overlying bed made
    /// of the very same material. A census that counts materials in the finished
    /// voxel then attributes that bed's eighths to the front. Comparing
    /// `expressed_clean_m` against `plan_clean_m` — the same voxels on both sides
    /// — is the apples-to-apples form.
    plan_clean_m: f64,
    /// `expressed_m` restricted to the same attributable voxels.
    expressed_clean_m: f64,
    /// `plan_clean_m` restricted further to **`Mixed`-plan** voxels.
    ///
    /// A `Single` front voxel is not a draw at all: `contents_for_event` reads
    /// the band's recorded eighths straight out and emits exactly that many, so
    /// its error against the plan is identically zero and it only *dilutes* an
    /// aggregate ratio. **Every stochastic decision in the front's expression
    /// lives in the `Mixed` voxels** — `allocate_partial` splitting the voxel
    /// between events, then `pore_rider_share` splitting the host's winnings
    /// between parent and product. So this is where the estimator claim is
    /// actually on trial.
    plan_mixed_m: f64,
    expressed_mixed_m: f64,
    front_voxels: usize,
    mixed_voxels: usize,
    /// Front voxels dropped as unattributable.
    collision_voxels: usize,
}

impl ColumnMass {
    /// Front magnitude in **eighths of a voxel of pure product** — the natural
    /// unit for the floor-effect question (one eighth is the smallest thing the
    /// voxel tier can express, so "how many eighths does this column owe?" is
    /// exactly "how far from the floor is it?").
    fn owed_eighths(&self) -> f64 {
        self.owed_m / VOXEL_M * 8.0
    }
}

/// The per-column measurement, off the **public generation path** only.
fn column_mass(pregen: &Pregen, cell: usize) -> Option<ColumnMass> {
    let set = vanilla();
    let (w, wp) = (pregen.deep.w, pregen.deep.wp);
    let (vx, vz) = idx_to_voxel(w, wp, cell);
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx0, lz0) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    let mut generator = WorldGenerator::new(pregen);
    let rec = generator.column_record(cx, cz);
    // P11 slice 3: this probe audits ONE deep cell's mass against its ledger,
    // so it must read a column that REALIZES that cell. At the cell centre the
    // home cell's bilinear weight is ~1, so the nearest realizing column is
    // essentially always (lx0, lz0) itself; the search is the honest guard.
    let mut pick: Option<(usize, usize)> = None;
    let mut best_d = i64::MAX;
    for z in 0..32usize {
        for x in 0..32usize {
            if rec
                .record_for(x, z)
                .and_then(dc_worldgen::SubCell::cell_index)
                == Some(cell as u32)
            {
                let d = (x as i64 - lx0 as i64).pow(2) + (z as i64 - lz0 as i64).pow(2);
                if d < best_d {
                    best_d = d;
                    pick = Some((x, z));
                }
            }
        }
    }
    let (lx, lz) = pick?;
    let sub = rec.record_for(lx, lz)?;
    let h = i64::from(rec.heights[lz * 32 + lx]);
    let fill = sub.fill();
    let front: Vec<usize> = (0..sub.strata().events.len())
        .filter(|&i| {
            sub.strata().events[i]
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
            sub.strata().events[i]
                .accessory
                .map(|(_, k)| f64::from(sub.strata().events[i].thickness_m) * f64::from(k) / 8.0)
        })
        .sum();
    let product_mat = sub.strata().events[front[0]]
        .accessory
        .map(|(m, _)| set.member(m).material)?;
    // Eighths of product a front band carries, as the expression path reads it.
    let band_k = |i: usize| -> f64 {
        sub.strata().events[i]
            .accessory
            .map_or(0.0, |(_, k)| f64::from(k))
    };
    let mut grids: HashMap<i64, Option<dc_core::ContentsGrid>> = HashMap::new();
    let mut eighths = 0u32;
    let mut clean_eighths = 0u32;
    let mut mixed_eighths = 0u32;
    let mut plan_m = 0.0f64;
    let mut plan_clean_m = 0.0f64;
    let mut plan_mixed_m = 0.0f64;
    let mut front_voxels = 0usize;
    let mut mixed_voxels = 0usize;
    let mut collision_voxels = 0usize;
    for p in 1..=fill.depth_count() {
        let plan = fill.plan(p as u32);
        let in_front = match plan {
            Some(Plan::Single(k)) => front.contains(k),
            Some(Plan::Mixed(ws)) => ws.iter().any(|(k, _)| front.contains(k)),
            None => false,
        };
        if !in_front {
            continue;
        }
        front_voxels += 1;
        // Stage 1 — the geometry. A `Single` front voxel is a whole voxel of one
        // band; a `Mixed` voxel splits by its own fixed-point weights (which sum
        // to a full voxel even where the record only partly covers it — that
        // renormalization IS the geometric error we are trying to see).
        let mut this_plan_m = 0.0f64;
        let mut attributable = true;
        match plan {
            Some(Plan::Single(k)) => this_plan_m += VOXEL_M * band_k(*k) / 8.0,
            Some(Plan::Mixed(ws)) => {
                let tot: f64 = ws.iter().map(|(_, x)| *x as f64).sum();
                for &(k, x) in ws {
                    if front.contains(&k) {
                        this_plan_m += VOXEL_M * (x as f64 / tot) * band_k(k) / 8.0;
                    } else if set.member(sub.strata().events[k].member).material == product_mat {
                        // An overlying bed of the very material the front's
                        // product is made of: the material census cannot tell
                        // its eighths from the front's.
                        attributable = false;
                    }
                }
            }
            None => {}
        }
        plan_m += this_plan_m;
        // Stage 2 — the draw, read off the real contents.
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
        let mut here = 0u32;
        for slots in [c.structure(), c.pore_fill(), c.debris()] {
            here += slots.iter().filter(|m| **m == product_mat).count() as u32;
        }
        eighths += here;
        let is_mixed = matches!(plan, Some(Plan::Mixed(_)));
        if is_mixed {
            mixed_voxels += 1;
        }
        if attributable {
            clean_eighths += here;
            plan_clean_m += this_plan_m;
            if is_mixed {
                mixed_eighths += here;
                plan_mixed_m += this_plan_m;
            }
        } else {
            collision_voxels += 1;
        }
    }
    Some(ColumnMass {
        owed_m: owed,
        plan_m,
        expressed_m: f64::from(eighths) / 8.0 * VOXEL_M,
        plan_clean_m,
        expressed_clean_m: f64::from(clean_eighths) / 8.0 * VOXEL_M,
        plan_mixed_m,
        expressed_mixed_m: f64::from(mixed_eighths) / 8.0 * VOXEL_M,
        front_voxels,
        mixed_voxels,
        collision_voxels,
    })
}

/// Summary statistics of a per-column relative error series.
struct ErrDist {
    n: usize,
    mean: f64,
    median: f64,
    p5: f64,
    p95: f64,
}

fn err_dist(mut v: Vec<f64>) -> ErrDist {
    if v.is_empty() {
        return ErrDist {
            n: 0,
            mean: 0.0,
            median: 0.0,
            p5: 0.0,
            p95: 0.0,
        };
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let pick = |q: f64| v[((n as f64 * q) as usize).min(n - 1)];
    ErrDist {
        n,
        mean: v.iter().sum::<f64>() / n as f64,
        median: pick(0.5),
        p5: pick(0.05),
        p95: pick(0.95),
    }
}

/// The whole population census, as one value — so the example can print it and
/// the gate test can assert on it without either re-deriving the other's numbers
/// (CLAUDE.md § Gates, 2026-07-25: an example that can fail belongs in the gate).
struct MassCensus {
    columns: Vec<ColumnMass>,
    /// Aggregate (mass-weighted) totals over every sampled column.
    sum_owed: f64,
    sum_plan: f64,
    sum_expressed: f64,
    sum_plan_clean: f64,
    sum_expressed_clean: f64,
    sum_plan_mixed: f64,
    sum_expressed_mixed: f64,
    collisions: usize,
}

impl MassCensus {
    fn n(&self) -> usize {
        self.columns.len()
    }
    /// Per-column relative error of the **whole** pipeline, record → voxels.
    fn rel_total(&self) -> Vec<f64> {
        self.columns
            .iter()
            .map(|c| (c.expressed_m - c.owed_m) / c.owed_m)
            .collect()
    }
    /// Per-column relative error of **stage 1 only** (the fill geometry).
    fn rel_geometry(&self) -> Vec<f64> {
        self.columns
            .iter()
            .map(|c| (c.plan_m - c.owed_m) / c.owed_m)
            .collect()
    }
    /// Per-column relative error of **stage 2 only** (the eighth draw), over the
    /// **attributable** voxels — the only comparison in which both sides count
    /// the same thing.
    fn rel_draw(&self) -> Vec<f64> {
        self.columns
            .iter()
            .filter(|c| c.plan_clean_m > 1e-9)
            .map(|c| (c.expressed_clean_m - c.plan_clean_m) / c.plan_clean_m)
            .collect()
    }
}

/// Sample `want` banded columns spread across the whole band distribution and
/// measure each. `banded` must be sorted strongest-first; the stride walk
/// therefore spans thin fronts as well as thick ones, which is where a floor
/// effect would live if there is one.
fn mass_census(pregen: &Pregen, banded: &[(usize, f64)], want: usize) -> MassCensus {
    let stride = (banded.len() / want.max(1)).max(1);
    let picks: Vec<usize> = banded
        .iter()
        .step_by(stride)
        .take(want)
        .map(|&(i, _)| i)
        .collect();
    let columns: Vec<ColumnMass> = picks
        .par_iter()
        .filter_map(|&cell| column_mass(pregen, cell))
        .filter(|c| c.owed_m > 0.0)
        .collect();
    let sum_owed = columns.iter().map(|c| c.owed_m).sum();
    let sum_plan = columns.iter().map(|c| c.plan_m).sum();
    let sum_expressed = columns.iter().map(|c| c.expressed_m).sum();
    let sum_plan_clean = columns.iter().map(|c| c.plan_clean_m).sum();
    let sum_expressed_clean = columns.iter().map(|c| c.expressed_clean_m).sum();
    let sum_plan_mixed = columns.iter().map(|c| c.plan_mixed_m).sum();
    let sum_expressed_mixed = columns.iter().map(|c| c.expressed_mixed_m).sum();
    let collisions = columns.iter().filter(|c| c.collision_voxels > 0).count();
    MassCensus {
        columns,
        sum_owed,
        sum_plan,
        sum_expressed,
        sum_plan_clean,
        sum_expressed_clean,
        sum_plan_mixed,
        sum_expressed_mixed,
        collisions,
    }
}

/// Every banded interior cell of a pregen, strongest band first.
fn banded_cells(pregen: &Pregen) -> Vec<(usize, f64)> {
    let w = pregen.deep.w;
    let band = |i: usize| -> f64 {
        pregen.deep.ledgers.get(i).map_or(0.0, |l| {
            l.weathering_product_m(pregen.deep.strata[i].units.len())
        })
    };
    let mut banded: Vec<(usize, f64)> = (0..w * w)
        .filter(|&i| interior(w, i) && band(i) > 1e-6)
        .map(|i| (i, band(i)))
        .collect();
    banded.sort_by(|a, b| b.1.total_cmp(&a.1));
    banded
}

/// The production world this probe reads.
fn production_world(extent: Extent) -> Pregen {
    Pregen::run_with(
        WorldParams { seed: SEED, extent },
        &DeepOverrides {
            weather_inventory: Some(true),
            ..DeepOverrides::default()
        },
    )
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
    // P11 slice 3: read the column skinned by THIS cell's record (the cell
    // centre realizes its home cell with weight ~1; fall back to the nearest
    // realizing column if the draw went elsewhere).
    let (lx, lz) = {
        let mut pick = (lx, lz);
        if rec
            .record_for(lx, lz)
            .and_then(dc_worldgen::SubCell::cell_index)
            != Some(cell as u32)
        {
            let mut best_d = i64::MAX;
            for z in 0..32usize {
                for x in 0..32usize {
                    if rec
                        .record_for(x, z)
                        .and_then(dc_worldgen::SubCell::cell_index)
                        == Some(cell as u32)
                    {
                        let d = (x as i64 - lx as i64).pow(2) + (z as i64 - lz as i64).pow(2);
                        if d < best_d {
                            best_d = d;
                            pick = (x, z);
                        }
                    }
                }
            }
        }
        pick
    };
    let Some(sub) = rec.record_for(lx, lz) else {
        println!("(cell {cell}: no column of its chunk realizes a record — skipped)");
        return;
    };
    let h = rec.heights[lz * 32 + lx];
    let fill = sub.fill();

    // The front's events are those carrying a **loose** pore-slot rider — the
    // weathering product. (The sparse igneous inclusion rides the same slot but
    // is a mineral, not a product; without this test the 96-voxel basement body
    // reads as "front".)
    let front: Vec<usize> = (0..sub.strata().events.len())
        .filter(|&i| {
            sub.strata().events[i]
                .accessory
                .is_some_and(|(m, _)| set.member(m).class.as_str() == CLASS_CLASTIC_FINE)
        })
        .collect();
    let product_mat = front
        .first()
        .and_then(|&i| sub.strata().events[i].accessory)
        .map(|(m, _)| set.member(m).material);
    let parent_mat = front
        .first()
        .map(|&i| set.member(sub.strata().events[i].member).material);

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
            .map_or(0.0, |&i| f64::from(sub.strata().events[i].thickness_m)),
        front
            .iter()
            .map(|&i| f64::from(sub.strata().events[i].thickness_m))
            .sum::<f64>(),
        front
            .iter()
            .map(|&i| f64::from(sub.strata().events[i].thickness_m))
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
        let grid = grids.entry(cy).or_insert_with(|| {
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
    // Two rows PAST the record on purpose: the bottom contact is the whole
    // question, and "what does the front sit on" is not answerable from inside it.
    let hi = last_plan + 2;
    let mut product_eighths = 0u32;
    let mut parent_eighths = 0u32;
    let mut rows: Vec<(i64, u8)> = Vec::new();
    for p in lo..=hi {
        let vy = h as i64 - (p as i64 - 1);
        let plan = match fill.plan(p as u32) {
            Some(Plan::Single(_)) => "Single",
            Some(Plan::Mixed(_)) => "Mixed",
            None => "basement",
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
    println!(
        "seed {SEED}, extent {}, weather_inventory ON\n",
        EXTENT.label()
    );
    let pregen = production_world(EXTENT);
    let w = pregen.deep.w;
    let banded = banded_cells(&pregen);
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

    // (3) The population mass check. journal/0099 ran this over 21 columns and
    //     read +3.7 % as quantization noise; the integrator's review (ROADMAP
    //     Observed) pointed out that an unbiased estimator's population mean
    //     trends to ZERO, and that a +16 % median is the signature of a floor
    //     effect. 21 columns cannot tell. This is the same check at N in the
    //     hundreds, decomposed into its two quantizers and stratified by front
    //     magnitude — journal/0103.
    println!("\n================ POPULATION — mass conservation ================");
    let census = mass_census(&pregen, &banded, CENSUS_COLUMNS);
    report_census(&census);
}

/// How many columns the example's population census samples.
const CENSUS_COLUMNS: usize = 400;

/// Front-magnitude strata, in eighths-of-a-voxel of owed product. The floor is
/// one eighth — the smallest thing the voxel tier can express — so these
/// buckets are "how many quanta does this column owe?", which is exactly the
/// axis a floor effect would live on.
const STRATA_EDGES: [f64; 5] = [1.0, 2.0, 4.0, 8.0, 24.0];

fn stratum_label(i: usize) -> String {
    match i {
        0 => "      < 1 eighth".into(),
        k if k < STRATA_EDGES.len() => {
            format!(
                "{:>5.0} – {:<4.0} eighths",
                STRATA_EDGES[k - 1],
                STRATA_EDGES[k]
            )
        }
        _ => format!(
            "     ≥ {:<4.0} eighths",
            STRATA_EDGES[STRATA_EDGES.len() - 1]
        ),
    }
}

fn stratum_of(owed_eighths: f64) -> usize {
    STRATA_EDGES.iter().filter(|e| owed_eighths >= **e).count()
}

fn print_dist(label: &str, d: &ErrDist) {
    println!(
        "  {label:<34} N={:<5} mean {:+7.2} %   median {:+7.2} %   p5 {:+7.2} %   p95 {:+7.2} %",
        d.n,
        100.0 * d.mean,
        100.0 * d.median,
        100.0 * d.p5,
        100.0 * d.p95
    );
}

fn report_census(c: &MassCensus) {
    println!(
        "  {} columns sampled across the whole band distribution.\n\
         \x20 aggregate: record owes {:.3} m · fill geometry hands the front {:.3} m ({:+.2} %) · \
         voxels express {:.3} m ({:+.2} %)",
        c.n(),
        c.sum_owed,
        c.sum_plan,
        100.0 * (c.sum_plan - c.sum_owed) / c.sum_owed,
        c.sum_expressed,
        100.0 * (c.sum_expressed - c.sum_owed) / c.sum_owed,
    );
    println!(
        "  ATTRIBUTABLE-ONLY comparison (the honest one): fill geometry {:.3} m, \
         voxels express {:.3} m ({:+.2} %)",
        c.sum_plan_clean,
        c.sum_expressed_clean,
        100.0 * (c.sum_expressed_clean - c.sum_plan_clean) / c.sum_plan_clean,
    );
    println!(
        "  columns holding at least one voxel where an overlying bed shares the \
         product's own material: {} / {} — those voxels are excluded from the \
         attributable comparison",
        c.collisions,
        c.n()
    );
    println!(
        "  MIXED-PLAN voxels only (the only voxels that contain a DRAW at all; a Single \
         front voxel emits its band's recorded eighths exactly): geometry {:.3} m, \
         voxels express {:.3} m ({:+.2} %)",
        c.sum_plan_mixed,
        c.sum_expressed_mixed,
        100.0 * (c.sum_expressed_mixed - c.sum_plan_mixed) / c.sum_plan_mixed.max(1e-9),
    );
    println!("\n  --- per-column relative error, by stage ---");
    print_dist("TOTAL   record -> voxels (naive)", &err_dist(c.rel_total()));
    print_dist(
        "stage 1 record -> fill geometry",
        &err_dist(c.rel_geometry()),
    );
    print_dist("stage 2 draw, attributable only", &err_dist(c.rel_draw()));

    println!("\n  --- stratified by front magnitude (the floor-effect axis) ---");
    println!(
        "  {:<22} {:>6}  {:>10}  {:>10}  {:>10}   {:>10}",
        "owed product", "N", "naive mean", "st-1 mean", "st-2 mean", "st-2 median"
    );
    for s in 0..=STRATA_EDGES.len() {
        let rows: Vec<&ColumnMass> = c
            .columns
            .iter()
            .filter(|m| stratum_of(m.owed_eighths()) == s)
            .collect();
        if rows.is_empty() {
            continue;
        }
        let tot = err_dist(
            rows.iter()
                .map(|m| (m.expressed_m - m.owed_m) / m.owed_m)
                .collect(),
        );
        let geo = err_dist(
            rows.iter()
                .map(|m| (m.plan_m - m.owed_m) / m.owed_m)
                .collect(),
        );
        let draw = err_dist(
            rows.iter()
                .filter(|m| m.plan_clean_m > 1e-9)
                .map(|m| (m.expressed_clean_m - m.plan_clean_m) / m.plan_clean_m)
                .collect(),
        );
        println!(
            "  {:<22} {:>6}  {:>+9.2} %  {:>+9.2} %  {:>+9.2} %   {:>+9.2} %  (st-2 N={})",
            stratum_label(s),
            tot.n,
            100.0 * tot.mean,
            100.0 * geo.mean,
            100.0 * draw.mean,
            100.0 * draw.median,
            draw.n,
        );
    }
    println!(
        "\n  (the record itself owes exactly the ledger — proven by \
         `the_weathering_front_conserves_the_ledger_product_mass`; everything above \
         is the voxel expression's own error.)"
    );
    println!(
        "  front voxels per column: min {} max {}; of {} front voxels, {} are Mixed \
         (carry a draw) and {} are unattributable",
        c.columns.iter().map(|m| m.front_voxels).min().unwrap_or(0),
        c.columns.iter().map(|m| m.front_voxels).max().unwrap_or(0),
        c.columns.iter().map(|m| m.front_voxels).sum::<usize>(),
        c.columns.iter().map(|m| m.mixed_voxels).sum::<usize>(),
        c.columns.iter().map(|m| m.collision_voxels).sum::<usize>(),
    );
}

/// **The gate's view of this instrument** (journal/0103).
///
/// The claim in the corpus is that the weathering front **conserves the ledger's
/// product mass**. At the record tier that is exact and already asserted
/// (`the_weathering_front_conserves_the_ledger_product_mass`). At the voxel tier
/// it was asserted by nobody and measured over 21 columns.
///
/// These tests fix the two halves separately, because they have different error
/// laws and only one of them is an estimator:
///
/// - **The fill geometry must be unbiased.** `ColumnFill` rounds the record's
///   bottom voxel to nearest, and the weathering front is *always* the thing at
///   the record's bottom, so the whole of that error lands on the front. If it
///   ever became a ceiling instead of a rounding, every front in the world would
///   gain mass and nothing else would notice.
/// - **The eighth draw must not lose the thin front.** The floor is one eighth;
///   a band thinner than that cannot express as less without vanishing, so the
///   *relative* error on a small front is unbounded above and bounded below at
///   −100 %. What must hold is that the draw does not systematically *delete*
///   product — the failure journal/0055 was built to prevent.
///
/// Run at [`Extent::Small`]. Both are statements about a single column's
/// arithmetic, and a column does not know how wide the grid is; a bigger extent
/// buys more samples of the same distribution. The **diagnosis numbers**
/// (journal/0103's verdict) are production numbers and stay in the example at
/// [`Extent::Medium`].
#[cfg(test)]
mod gate {
    use super::*;

    use std::sync::OnceLock;

    /// Columns the gate census samples. Enough that the geometry bound below is
    /// a real constraint rather than a coin flip.
    const GATE_COLUMNS: usize = 120;

    /// **Built once for the whole binary.** Three tests share one world and one
    /// census; libtest runs them on separate threads, so without this the gate
    /// would pay for the small world three times over. Sizing the gate is part of
    /// the conversion, not an afterthought (CLAUDE.md § Gates).
    fn gate_census() -> &'static MassCensus {
        static CENSUS: OnceLock<MassCensus> = OnceLock::new();
        CENSUS.get_or_init(|| {
            let pregen = production_world(Extent::Small);
            let banded = banded_cells(&pregen);
            assert!(
                !banded.is_empty(),
                "no weathering band anywhere on the small world — nothing to weigh"
            );
            let c = mass_census(&pregen, &banded, GATE_COLUMNS);
            assert!(
                c.n() >= 20,
                "only {} columns measured; the census is too thin to bound anything",
                c.n()
            );
            c
        })
    }

    /// **Stage 1.** The record → fill-geometry step is a round-to-nearest on the
    /// record's bottom voxel and must wash out in aggregate. It is *not* an
    /// estimator with variance to hide behind: if this drifts, it is a ceiling.
    #[test]
    fn the_fill_geometry_hands_the_front_the_metres_the_record_owes() {
        let c = gate_census();
        let rel = (c.sum_plan - c.sum_owed) / c.sum_owed;
        assert!(
            rel.abs() < 0.02,
            "the fill geometry hands the weathering front {:.3} m against {:.3} m owed \
             ({:+.2} % over {} columns) — the record-bottom rounding has become one-sided, \
             which silently moves mass into (or out of) every front in the world",
            c.sum_plan,
            c.sum_owed,
            100.0 * rel,
            c.n(),
        );
        let d = err_dist(c.rel_geometry());
        assert!(
            d.median.abs() < 0.02,
            "median per-column geometric error {:+.2} % (mean {:+.2} %, p5 {:+.2} %, \
             p95 {:+.2} %) — a centred rounding has a median at zero",
            100.0 * d.median,
            100.0 * d.mean,
            100.0 * d.p5,
            100.0 * d.p95,
        );
    }

    /// **Stage 2.** The eighth draw, over the `Mixed` voxels — the only ones that
    /// contain a draw at all, since a `Single` front voxel emits its band's
    /// recorded eighths exactly — and only where the product can be attributed to
    /// the front rather than to an overlying bed of the same material.
    ///
    /// journal/0103 measured this at **−0.84 %** aggregate on the production
    /// world, so the bounds are set loosely on both sides: this is an unbiased
    /// estimator over a one-eighth quantum, and its *variance* on a small census
    /// is real. What must never happen is a drift to strongly **negative** —
    /// that is journal/0055's deleted thin bed coming back — or to strongly
    /// **positive**, which would mean the rider's share stopped being
    /// proportional to its host's winnings.
    #[test]
    fn the_eighth_draw_does_not_delete_the_thin_front() {
        let c = gate_census();
        assert!(
            c.sum_plan_mixed > 0.0,
            "no attributable Mixed front voxel in the whole census — either every front \
             voxel shares its product's material with an overlying bed, or the front no \
             longer straddles a contact at all; either way this instrument can no longer \
             see the draw it exists to weigh"
        );
        let rel = (c.sum_expressed_mixed - c.sum_plan_mixed) / c.sum_plan_mixed;
        assert!(
            rel > -0.15,
            "over the Mixed (drawn) front voxels the expression is {:.3} m where the \
             geometry gives {:.3} m ({:+.2} %) — product is being DELETED at the voxel \
             tier, which is exactly the biased-flooring failure journal/0055 replaced",
            c.sum_expressed_mixed,
            c.sum_plan_mixed,
            100.0 * rel,
        );
        assert!(
            rel < 0.30,
            "the drawn front voxels express {:+.2} % more product than the geometry gives \
             them — far past anything an eighth of quantization explains; the rider share \
             has stopped being proportional to its host's winnings",
            100.0 * rel,
        );
    }

    /// The shape claim journal/0099 shipped: the front is **graded over many
    /// voxels**, not a slab, and its deepest voxel is never pure product. If
    /// this breaks, the world silently returns to the hard perimeter
    /// journal/0097's walk caught.
    #[test]
    fn the_front_is_graded_over_many_voxels_and_never_pure_product() {
        let c = gate_census();
        let deep: Vec<&ColumnMass> = c
            .columns
            .iter()
            .filter(|m| m.owed_eighths() >= 8.0)
            .collect();
        assert!(
            !deep.is_empty(),
            "no column owes even one voxel of product; the profile claim is untestable here"
        );
        let multi = deep.iter().filter(|m| m.front_voxels >= 3).count();
        assert!(
            multi * 2 > deep.len(),
            "only {} of {} columns owing >= 1 voxel of product spread their front over 3+ \
             voxels — the profile has collapsed back toward a slab",
            multi,
            deep.len(),
        );
    }
}
