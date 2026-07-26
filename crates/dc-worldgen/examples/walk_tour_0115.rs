//! **The tour map for the walk owed since journal/0112** — colluvium, the
//! alluvial contrast, and the pits.
//!
//! Three slices shipped without anyone standing in front of them (journal/0112
//! material-aware creep, 0113 hybrid `p`, 0114 the calibration that ships OFF).
//! The ROADMAP close block names the stations; this probe finds them on the world
//! the client actually boots — `BENCH_SEED` 1337, `Extent::Medium`, the shipped
//! production config — and prints ready-to-paste poses.
//!
//! **Two arms, because one station lives in a world that does not ship.**
//! Stations B and C are on the **shipped** world (`calibrated_rates: false`).
//! Station A is the pit field, which by construction exists only when the
//! erosional amplitude leaves 1× — so it is measured on the **calibrated** arm,
//! reachable live as `dc-client --calibrated-rates`.
//!
//! **This probe tunes nothing.** It ranks cells and converts coordinates.
//!
//! Run: `cargo run --release -p dc-worldgen --example walk_tour_0115`

use std::time::Instant;

use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{
    DeepOverrides, SEA_LEVEL_M, build_field_with, litho_of_tag, production_config,
    production_config_with, run_cells,
};
use dc_worldgen::pregen::{CELL_VOXELS, CellGrid, Extent, Pregen, WorldParams};

/// The client's `BENCH_SEED` — the world every walk so far has stood in.
const SEED: u64 = 1337;
/// `GenOptions::default().extent` — the client's boot extent.
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;
/// Player eye height above feet, metres — for the standoff arithmetic only.
const EDGE_MARGIN: i64 = 4;

/// The deep-cell ↔ world-voxel bridge, lifted verbatim from `tour_map` (which
/// lifted it from `roughness_probe`): `DeepField::deep_coords`' own centring.
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

    fn meters(&self, idx: usize) -> (f64, f64) {
        let (vx, vz) = self.idx_to_voxel(idx);
        (vx as f64 * VOXEL_M, vz as f64 * VOXEL_M)
    }

    fn where_line(&self, idx: usize) -> String {
        let (gx, gy) = (idx % self.w, idx / self.w);
        let (vx, vz) = self.idx_to_voxel(idx);
        let (mx, mz) = self.meters(idx);
        format!("deep cell ({gx},{gy}) | voxel ({vx}, {vz}) | world ({mx:.0} m, {mz:.0} m)")
    }
}

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

/// A station, ready to paste into `client_player_pose_set`.
fn pose_line(conv: &Conv, idx: usize, surf_m: f64, standoff_m: f64, note: &str) {
    let (mx, mz) = conv.meters(idx);
    println!("      {}", conv.where_line(idx));
    println!("      surface {surf_m:.1} m");
    println!(
        "      POSE  feet ({mx:.0}, {:.0}, {mz:.0})  surface:true   \
         (then back off ~{standoff_m:.0} m — {note})",
        surf_m + 2.0
    );
}

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

fn main() {
    println!("=== walk tour: colluvium · alluvial contrast · the pits ===");
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

    let base = production_config(&pregen.grid, SEED);
    assert!(
        !base.calibrated_rates,
        "production must still ship calibrated_rates OFF"
    );
    assert!(
        base.material_creep,
        "the colluvium stations assume material-aware creep is on in production"
    );

    let wp = pregen.deep.wp;

    // ------------------------------------------------------------- the shipped arm
    let t = Instant::now();
    let shipped = build_field_with(&pregen.grid, SEED, &DeepOverrides::default());
    eprintln!("shipped world built in {:?}", t.elapsed());
    let conv = Conv { w: shipped.w, wp };

    // Rank-based deciles of drainage area over LAND cells — the same instrument
    // `colluvium_probe` uses, for the same reason (catchment area is heavy-tailed,
    // so equal-width bins put 99 % of the world in bin 0).
    let mut areas: Vec<f64> = shipped
        .area
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, _)| shipped.surf[*i] > SEA_LEVEL_M)
        .map(|(_, a)| a)
        .collect();
    areas.sort_by(f64::total_cmp);
    let decile_of = |a: f64| -> usize {
        if areas.is_empty() {
            return 0;
        }
        let rank = areas.partition_point(|&x| x < a);
        ((rank * 10) / areas.len()).min(9)
    };

    // The signature: thickness whose recorded SPECIES disagrees with what the
    // depositing environment would have implied. That is provenance — material
    // that was carried here from somewhere that is not here.
    //
    // Ranked separately on the hillslope end (deciles 0–2, where colluvium
    // belongs) and the trunk end (decile 9, where alluvium belongs), because the
    // walk's whole question is whether those two read as different rocks.
    let mut hill: Vec<(f64, usize, usize)> = Vec::new(); // (travelled_m, idx, distinct species)
    let mut trunk: Vec<(f64, usize, usize)> = Vec::new();
    let (mut cells_with_travel, mut total_travel, mut total_record) = (0usize, 0.0f64, 0.0f64);

    for (i, rec) in shipped.strata.iter().enumerate() {
        if rec.units.is_empty() || !interior(shipped.w, i) || shipped.surf[i] <= SEA_LEVEL_M {
            continue;
        }
        let mut travelled = 0.0;
        let mut present = [false; Litho::COUNT];
        for u in &rec.units {
            total_record += u.thickness_m;
            present[u.species.index()] = true;
            if u.species != litho_of_tag(u.tag) {
                travelled += u.thickness_m;
            }
        }
        total_travel += travelled;
        if travelled <= 0.0 {
            continue;
        }
        cells_with_travel += 1;
        let distinct = present.iter().filter(|p| **p).count();
        let dec = decile_of(shipped.area[i]);
        if dec <= 2 {
            hill.push((travelled, i, distinct));
        } else if dec == 9 {
            trunk.push((travelled, i, distinct));
        }
    }
    hill.sort_by(|a, b| b.0.total_cmp(&a.0));
    trunk.sort_by(|a, b| b.0.total_cmp(&a.0));

    println!("--- DISTRIBUTION (shipped world) ---");
    println!(
        "  land cells with travelled material: {cells_with_travel}\n  \
         travelled thickness {total_travel:.0} m of {total_record:.0} m archive \
         ({:.3} %)",
        100.0 * total_travel / total_record.max(1.0)
    );
    println!(
        "  hillslope candidates (decile 0–2): {}   trunk candidates (decile 9): {}\n",
        hill.len(),
        trunk.len()
    );

    println!("--- STATION B: the colluvium road cut (hillslope, shipped world) ---");
    if hill.is_empty() {
        println!("  NULL — no hillslope cell carries travelled material. Do not launch for this.");
    } else {
        for (rank, &(t_m, idx, distinct)) in hill.iter().take(3).enumerate() {
            println!(
                "  #{}  travelled {t_m:.2} m, {distinct} distinct species in column",
                rank + 1
            );
            pose_line(
                &conv,
                idx,
                shipped.surf[idx],
                60.0,
                "a road-cut bench frames at ~60 m",
            );
        }
    }

    println!("\n--- STATION C: the alluvial contrast (trunk, shipped world) ---");
    if trunk.is_empty() {
        println!("  NULL — no trunk cell carries travelled material.");
    } else {
        for (rank, &(t_m, idx, distinct)) in trunk.iter().take(3).enumerate() {
            println!(
                "  #{}  travelled {t_m:.2} m, {distinct} distinct species in column",
                rank + 1
            );
            pose_line(&conv, idx, shipped.surf[idx], 60.0, "same framing as B");
        }
        if let (Some(&(_, b, _)), Some(&(_, c, _))) = (hill.first(), trunk.first()) {
            let (bx, bz) = conv.meters(b);
            let (cx, cz) = conv.meters(c);
            println!(
                "  B↔C separation: {:.1} km",
                ((bx - cx).powi(2) + (bz - cz).powi(2)).sqrt() / 1000.0
            );
        }
    }

    // ---------------------------------------------------------- the calibrated arm
    println!(
        "\n--- STATION A: the pit field (CALIBRATED world — launch with --calibrated-rates) ---"
    );
    let t = Instant::now();
    let cal = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            calibrated_rates: Some(true),
            ..DeepOverrides::default()
        },
    );
    eprintln!("calibrated world built in {:?}", t.elapsed());
    let convc = Conv { w: cal.w, wp };

    let w = cal.w;
    let mut pits: Vec<(f64, usize, bool)> = Vec::new();
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let lowest = NEIGH8
                .into_iter()
                .map(|(dx, dy)| cal.surf[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize])
                .fold(f64::INFINITY, f64::min);
            if cal.surf[i] < lowest - 1.0 {
                pits.push((
                    lowest - cal.surf[i],
                    i,
                    cal.lake.get(i).copied().unwrap_or(false),
                ));
            }
        }
    }
    pits.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!(
        "  interior pits deeper than 1 m on PRODUCTION-Medium calibrated: {} \
         (deepest {:.2} m, {} of them recorded lakes)",
        pits.len(),
        pits.first().map(|p| p.0).unwrap_or(0.0),
        pits.iter().filter(|p| p.2).count()
    );
    if pits.is_empty() {
        println!(
            "  NULL on Medium — the 148-pit measurement was on the `mfd_routing` \
             SMALL fixture. Brief this honestly; do not launch for pits."
        );
    } else {
        for (rank, &(depth, idx, lake)) in pits.iter().take(3).enumerate() {
            println!(
                "  #{}  {depth:.1} m below every neighbour{}",
                rank + 1,
                if lake { " (recorded lake)" } else { "" }
            );
            pose_line(
                &convc,
                idx,
                cal.surf[idx],
                150.0,
                "stand on the RIM and look in; a 460 m cell needs altitude",
            );
        }
    }

    // The control the walk needs: the SAME census on the shipped world, so a
    // non-zero count above is attributable to the calibration and not to the
    // instrument. (CLAUDE.md: a zero from an unproven census is not evidence —
    // and the mirror holds, a non-zero from an uncontrolled one is not either.)
    let mut shipped_pits = 0usize;
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let lowest = NEIGH8
                .into_iter()
                .map(|(dx, dy)| {
                    shipped.surf[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize]
                })
                .fold(f64::INFINITY, f64::min);
            if shipped.surf[i] < lowest - 1.0 {
                shipped_pits += 1;
            }
        }
    }
    println!("  CONTROL — same census on the shipped world: {shipped_pits} pits");

    // ---------------------------------------------------------- the real censuses
    //
    // Both of the walk's load-bearing measurements, from ONE solve per arm. `main`
    // prints them and `mod gate` asserts on them, so the report and the guard cannot
    // drift apart (journal/0103).
    println!(
        "
--- THE CENSUSES (shipped is the control) ---"
    );
    for (label, calibrated) in [("shipped", false), ("calibrated", true)] {
        let c = census(&pregen.grid, calibrated, wp, STATION_A);
        println!(
            "  {label:11} land {:5} | hollows >1 m {:5} ({:4.1} %)  >10 m {:5}  >50 m {:4}               deepest {:6.1} m  fill {:5.1} km3",
            c.land,
            c.hollow_1,
            100.0 * c.hollow_1 as f64 / c.land as f64,
            c.hollow_10,
            c.hollow_50,
            c.deepest,
            c.fill_km3
        );
        println!(
            "               concavity mean {:+.2} m | p10 {:+.1} p50 {:+.1} p90 {:+.1}              p99 {:+.1} | >1 m {:.1} % >5 m {:.1} % >20 m {:.1} %",
            c.conc_mean,
            c.p10,
            c.p50,
            c.p90,
            c.p99,
            c.conc_gt1_pct,
            c.conc_gt5_pct,
            c.conc_gt20_pct
        );
        println!(
            "               within 10 km of station A: {} closed hollows of {} land cells              ({:.1} %)",
            c.near_hollow,
            c.near_land,
            100.0 * c.near_hollow as f64 / c.near_land.max(1) as f64
        );
    }

    println!("\ntotal probe wall-clock {:?}", t0.elapsed());
}

// ---------------------------------------------------------------- the instrument

/// The walk's station A (world metres), for the regional-density column.
const STATION_A: (f64, f64) = (41870.0, 12433.0);

/// Both censuses over one solve. **Neither is a "below all its neighbours" test**,
/// and that is the whole point — see `mod gate`.
struct Census {
    land: usize,
    /// Closed hollows by the router's own depression fill (`filled − routed`), which
    /// measures the hollow at a cell **regardless of what its neighbours do**.
    hollow_1: usize,
    hollow_10: usize,
    hollow_50: usize,
    deepest: f64,
    fill_km3: f64,
    /// Concavity = `mean(8 neighbours) − self`, the discrete Laplacian of the
    /// surface. Positive ⇒ the cell sits below its surroundings.
    conc_mean: f64,
    p10: f64,
    p50: f64,
    p90: f64,
    p99: f64,
    conc_gt1_pct: f64,
    conc_gt5_pct: f64,
    conc_gt20_pct: f64,
    near_land: usize,
    near_hollow: usize,
}

fn census(cells: &CellGrid, calibrated: bool, wp: usize, target: (f64, f64)) -> Census {
    let cfg = production_config_with(
        cells,
        SEED,
        &DeepOverrides {
            calibrated_rates: Some(calibrated),
            ..DeepOverrides::default()
        },
    );
    let run = run_cells(cells, &cfg, true);
    let (w, cell_a) = (run.grid.w, run.grid.cell_m * run.grid.cell_m);
    let filled = run.erosion.filled();
    let routed = run.erosion.routed_surface();
    let cv = Conv { w, wp };

    let mut conc: Vec<f64> = Vec::new();
    let mut out = Census {
        land: 0,
        hollow_1: 0,
        hollow_10: 0,
        hollow_50: 0,
        deepest: 0.0,
        fill_km3: 0.0,
        conc_mean: 0.0,
        p10: 0.0,
        p50: 0.0,
        p90: 0.0,
        p99: 0.0,
        conc_gt1_pct: 0.0,
        conc_gt5_pct: 0.0,
        conc_gt20_pct: 0.0,
        near_land: 0,
        near_hollow: 0,
    };
    let (mut g1, mut g5, mut g20) = (0usize, 0usize, 0usize);
    let mut vol = 0.0f64;
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if routed[i] <= SEA_LEVEL_M {
                continue;
            }
            out.land += 1;
            let d = filled[i] - routed[i];
            if d > 1.0 {
                out.hollow_1 += 1;
                vol += d * cell_a;
                out.deepest = out.deepest.max(d);
            }
            if d > 10.0 {
                out.hollow_10 += 1;
            }
            if d > 50.0 {
                out.hollow_50 += 1;
            }
            let s = run.grid.surf_at(i);
            let mut sum = 0.0;
            for (dx, dy) in NEIGH8 {
                sum += run
                    .grid
                    .surf_at((y as i32 + dy) as usize * w + (x as i32 + dx) as usize);
            }
            let c = sum / 8.0 - s;
            conc.push(c);
            if c > 1.0 {
                g1 += 1;
            }
            if c > 5.0 {
                g5 += 1;
            }
            if c > 20.0 {
                g20 += 1;
            }
            let (mx, mz) = cv.meters(i);
            if ((mx - target.0).powi(2) + (mz - target.1).powi(2)).sqrt() < 10_000.0 {
                out.near_land += 1;
                if d > 1.0 {
                    out.near_hollow += 1;
                }
            }
        }
    }
    conc.sort_by(f64::total_cmp);
    let n = conc.len().max(1) as f64;
    let q = |p: f64| conc[((conc.len() as f64 - 1.0) * p).max(0.0) as usize];
    out.fill_km3 = vol / 1e9;
    out.conc_mean = conc.iter().sum::<f64>() / n;
    out.p10 = q(0.10);
    out.p50 = q(0.50);
    out.p90 = q(0.90);
    out.p99 = q(0.99);
    out.conc_gt1_pct = 100.0 * g1 as f64 / n;
    out.conc_gt5_pct = 100.0 * g5 as f64 / n;
    out.conc_gt20_pct = 100.0 * g20 as f64 / n;
    out
}

/// **Only the CENSUS is gated — the station search deliberately is not** (the
/// precedent `coal_walk_tour` and `palette_quant_tour` set: a station finder's output
/// is a *recommendation judged by the eye*, not a claim).
///
/// **Why the invariant is scale-free**, per CLAUDE.md § Gates "size the test, not the
/// report": both quantities are **per-cell arithmetic over a 3×3 neighbourhood** — a
/// depression fill at a cell, and the Laplacian of the surface at a cell. Neither
/// references world extent, cell count or run length. A solve that leaves a cell below
/// its own outlet, or 50 m off its neighbours' mean, does so at any size; only the
/// *magnitudes* in the printed report are about production scale.
///
/// **What this guards, and why the existing guard could not** (corrections #62):
/// `mfd_routing::no_interior_cell_is_cut_below_all_of_its_neighbours` counts cells below
/// **all eight** neighbours — a winner-take-all predicate that **saturates**, because as
/// the defect generalises the neighbours sink too and stop qualifying each other. It is
/// maximised by *isolated* pits and blind to a pockmarked world. Both assertions here are
/// **absolute per cell**, so neither can be defeated by its neighbours also being broken.
#[cfg(test)]
mod gate {
    use super::*;

    fn small() -> Pregen {
        Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        })
    }

    /// **The shipped solve produces drainable terrain.** Zero closed hollows deeper than
    /// a metre — asserted as an absolute count, not a ranking.
    #[test]
    fn the_shipped_solve_leaves_no_closed_hollows() {
        let p = small();
        let c = census(&p.grid, false, p.deep.wp, STATION_A);
        assert!(c.land > 100, "fixture found no land ({} cells)", c.land);
        assert_eq!(
            c.hollow_1, 0,
            "the shipped solve left {} closed hollows deeper than 1 m (deepest {:.1} m) — \
             terrain that does not drain",
            c.hollow_1, c.deepest
        );
    }

    /// **And it is smooth at the grid scale.** The bound is derived, not fitted: on the
    /// shipped world the *entire* concavity distribution fits inside ±0.3 m
    /// (journal/0115), while the calibrated world's p99 is +106 m. A 5 m ceiling sits
    /// ~17× above the observed spread and ~20× below the defect, so it cannot be tripped
    /// by ordinary terrain and cannot miss grid instability.
    ///
    /// It asserts a **bound**, never a snapshot: if someone improves the solve and the
    /// spread shrinks, this still passes.
    #[test]
    fn the_shipped_solve_is_smooth_at_the_grid_scale() {
        let p = small();
        let c = census(&p.grid, false, p.deep.wp, STATION_A);
        assert!(
            c.p99 < 5.0 && c.p10 > -5.0,
            "grid-scale roughness: concavity p10 {:+.1} m / p99 {:+.1} m exceeds ±5 m \
             (shipped spread is ±0.3 m). Adjacent cells are oscillating — see stubs #29.",
            c.p10,
            c.p99
        );
    }

    /// **THE OTHER ARM — without this the two guards above are unfalsifiable.**
    ///
    /// They pass on the shipped world. That is only evidence if the *same instrument at
    /// the same size* can still **see** the defect — otherwise a green pair proves the
    /// fixture is small, not that the solve is sound (anti-shape A-3: green for a reason
    /// unrelated to the claim, which is exactly how the clamp guard sat green through
    /// journal/0114 and how the below-all-neighbours census undercounted by 2.6× in
    /// journal/0115).
    ///
    /// It is also the **verification of this module's scale-free claim**, which was
    /// asserted in prose before it was measured. `Extent::Small` is a fraction of
    /// production and the defect must still be plainly visible there. If this test ever
    /// fails, the honest conclusion is *"the invariant is NOT scale-free and the guards
    /// above must move to Medium"* — **not** that the calibration got better.
    #[test]
    fn the_calibrated_solve_still_shows_the_defect_at_this_size() {
        let p = small();
        let c = census(&p.grid, true, p.deep.wp, STATION_A);
        assert!(
            c.hollow_1 > 0,
            "the calibrated arm produced NO closed hollows at Extent::Small, so the two \
             shipped-world guards here are vacuous at this size — move them to Medium"
        );
        assert!(
            c.p99 > 5.0,
            "the calibrated arm's concavity p99 is only {:+.1} m at Extent::Small, below \
             the ±5 m bound the shipped guard asserts — the guard cannot discriminate at \
             this size and must move to Medium",
            c.p99
        );
        println!(
            "calibrated @ Small: {} hollows (deepest {:.1} m), concavity p10 {:+.1} / \
             p99 {:+.1} — the defect is visible at this size",
            c.hollow_1, c.deepest, c.p10, c.p99
        );
    }
}
