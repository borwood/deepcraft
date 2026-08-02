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
//!
//! ---
//!
//! # The discriminators (journal/0116, added 2026-07-26)
//!
//! journal/0115 left the blocker carrying a **hypothesis it had not measured** — that the
//! solve above 1× is an explicit scheme past its stability limit — and named the
//! discriminators that would settle it. They live **here**, in the same file as
//! [`census`], because an example is its own crate root: a sibling example cannot call
//! this one's census, so a sibling would have had to *copy* it, and a second census beside
//! the first is exactly the anti-shape (`spines.md` A-1) this repo keeps catching itself
//! in. One census, one solve per configuration, three consumers: `main`'s report, the
//! sweep, and `mod gate`.
//!
//! - **D1 — is it a checkerboard?** Spatial autocorrelation and sign-alternation of the
//!   concavity field. Free: it rides the same solve the census already runs.
//! - **D2 — does it respond like a stability limit?** [`refine_epochs`] halves the epoch
//!   length at doubled epoch count and **fixed total simulated time**.
//! - **D3 — is isostasy the mechanism?** Ablate [`DeepConfig::iso_rate`], and correlate
//!   concavity against the local regolith excess `h − h̄` that drives it.
//!
//! Run the sweep: `cargo run --release -p dc-worldgen --example walk_tour_0115 -- --sweep`
//! (it skips the station search — it wants the solves, not the poses).

use std::time::Instant;

use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{
    DeepConfig, DeepOverrides, SEA_LEVEL_M, build_field_with, census, isostasy, litho_of_tag,
    production_config, production_config_with, run_cells, scale_erosion_rates,
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
    let sweep = std::env::args().any(|a| a == "--sweep");
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

    if sweep {
        run_sweep(&pregen, t0);
        return;
    }
    if std::env::args().any(|a| a == "--fields") {
        run_fields(&pregen, t0);
        return;
    }

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
            present[Litho::of_material(u.species).index()] = true;
            if Litho::of_material(u.species) != litho_of_tag(u.tag) {
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
        let c = census(
            &pregen.grid,
            &arm_cfg(&pregen.grid, calibrated),
            wp,
            STATION_A,
        );
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
        print_structure(&c);
    }

    println!("\ntotal probe wall-clock {:?}", t0.elapsed());
}

// ---------------------------------------------------------------- the instrument

/// The walk's station A (world metres), for the regional-density column.
const STATION_A: (f64, f64) = (41870.0, 12433.0);

/// Both censuses over one solve. **Neither is a "below all its neighbours" test**,
/// and that is the whole point — see `mod gate`.
/// The four census PRIMITIVES this struct is built from — `laplacian8`, `pearson`, `acf4`,
/// `flip_rate` — moved to `dc_worldgen::deeptime::census` on 2026-07-29 (journal/0122). This
/// entry recorded the extraction as open and out of scope for the honest reason that a Rust
/// example is its own crate root and cannot call a sibling's census; the fix slice needed the
/// same four functions and the alternative was **copying** them, which is anti-shape A-1 on the
/// exact code path where journal/0115 found three instruments agreeing because they were the
/// same instrument copied three times.
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

    // ---- the landscape, for D2's "does the shape move?" leg ----------------
    /// `max(surf) − min(surf)` over the whole grid — journal/0114's criterion 1,
    /// kept here only so D2 can show it standing still (corrections #61: it is a
    /// global extremal statistic and licenses nothing about arrangement).
    relief: f64,
    /// Mean surface over land cells.
    surf_mean: f64,

    // ---- D1: the SPATIAL STRUCTURE of the concavity field -------------------
    conc_rms: f64,
    /// Concavity autocorrelation at lags 1..4 along +x and +y.
    acf_x: [f64; 4],
    acf_y: [f64; 4],
    /// Fraction of adjacent land pairs whose concavity has **opposite sign**.
    flip_x: f64,
    flip_y: f64,
    /// Autocorrelation of the surface's **first difference** at lags 1..4.
    dacf_x: [f64; 4],
    dacf_y: [f64; 4],

    // ---- D3: the isostatic-feedback correlates ------------------------------
    /// Mean regolith thickness over land.
    h_mean: f64,
    /// RMS of the local regolith excess `h − h̄` over the flexural smooth — the
    /// quantity `isostasy` differences against.
    h_excess_rms: f64,
    /// Pearson correlation of concavity against `h − h̄`.
    r_conc_hexcess: f64,
    /// Pearson correlation of concavity against `h` itself (the control: if the
    /// correlation is with bulk cover rather than with *local excess*, the
    /// smoothed-target story is not what is acting).
    r_conc_h: f64,
    /// Percent of cells sitting on `apply_thickening`'s 1000 m crustal floor.
    crust_floor_pct: f64,

    // ---- WHICH FIELD is oscillating: bedrock top, or the regolith blanket? ---
    /// Concavity rms and lag-1 ACF of the **bedrock top `r`** alone.
    conc_r_rms: f64,
    acf_r: (f64, f64),
    /// Concavity rms and lag-1 ACF of the **regolith thickness `h`** alone.
    conc_h_rms: f64,
    acf_h: (f64, f64),
    /// Percent of creep cell-epochs (over cells that had regolith) in which the
    /// flux limiter bound — `TransportLedger::creep_limited_cell_epochs`. `NaN`
    /// unless the config armed `denudation_ledger`.
    creep_limited_pct: f64,
}

/// The arm's config: the shipped production config with the calibration on or off.
/// Everything else — every flag, every provider, every rate — is production's.
fn arm_cfg(cells: &CellGrid, calibrated: bool) -> DeepConfig {
    production_config_with(
        cells,
        SEED,
        &DeepOverrides {
            calibrated_rates: Some(calibrated),
            ..DeepOverrides::default()
        },
    )
}

/// **D2's operator: refine the time step, hold the physics fixed.**
///
/// Run `k×` as many epochs, each `1/k` as long, so **total simulated time is
/// unchanged**. Everything the config states *per epoch* is divided by `k`;
/// everything it states *in epochs* is multiplied by `k`. If the solve is
/// converged this is a no-op on the landscape; if it is running past a stability
/// limit, the limit is what moves.
///
/// **What is scaled, and why each one is in the list:**
/// - `iterations` — the epoch count itself.
/// - `weathering`, `diffusion`, `k_transport`, `k_bedrock` — metres (or the
///   coefficients of metres) **per iteration**, via the same
///   [`scale_erosion_rates`] the calibration uses, so the four move together and
///   their ratios — which the erodibility coupling reads — never change.
/// - `thickening_scale` — documented "m/iter" on `DeepConfig`.
/// - `eolian_deflation`, `wave_erosion` — metres per epoch.
/// - `sea_level_period`, `remarch_interval` — stated **in iterations**, so they
///   multiply, keeping the paleo-sea-level curve and the orographic re-march on
///   the same *physical* cadence.
/// - `iso_rate`, `eolian_deposit_frac` — **fractions closed per epoch**, i.e.
///   exponential relaxations. The dt-consistent refinement of `1 − (1−f)` over
///   `k` sub-steps is `1 − (1−f)^(1/k)`, which is what is applied: `k` refined
///   steps then close exactly the same fraction as one coarse step did.
///
/// **What is deliberately NOT scaled, stated so the confound is visible:**
/// - `uplift_scale` — it feeds `apply_uplift`, and `Erosion::step` calls that
///   **only when `tectonic_history` is off**. Production has it on, so the plane
///   is inert; scaling it would be theatre.
/// - `chapters` / `advection_plate_widths` — the chapter blend divides by
///   `cfg.iterations` (`TectonicSchedule::blend_into`) and the per-chapter
///   advection divides by `chapters`, so both are already expressed as fractions
///   of the run. They are scale-free by construction.
/// - `frost_weathering_gain`, `erodibility_*`, `m_exp`/`n_exp`, `h_star`,
///   `rough_jitter`, every `*_km` and `*_band_*` — dimensionless multipliers,
///   exponents, lengths, or one-shot initial conditions. None of them is a rate.
/// - `head::HEAD_PERIOD` and `geotherm::GEOTHERM_PERIOD` — **not reachable from
///   `DeepConfig`**, so the two coarse-rate field passes fire `k×` more often in
///   physical time under refinement. Both plant read-quality fields that no
///   in-epoch erosion pass reads (`runner.rs` says so of the geotherm in as many
///   words, and of the head field that its position "decides the head FIELD'S OWN
///   VALUES"), so neither can move the surface. **This is the one un-neutralised
///   term in the experiment and it is named rather than assumed away.**
fn refine_epochs(cfg: &mut DeepConfig, k: u32) {
    if k == 1 {
        return;
    }
    let kf = f64::from(k);
    cfg.iterations *= k;
    scale_erosion_rates(cfg, 1.0 / kf);
    cfg.thickening_scale /= kf;
    cfg.eolian_deflation /= kf;
    cfg.wave_erosion /= kf;
    cfg.sea_level_period *= k;
    cfg.remarch_interval *= k;
    cfg.iso_rate = 1.0 - (1.0 - cfg.iso_rate).powf(1.0 / kf);
    cfg.eolian_deposit_frac = 1.0 - (1.0 - cfg.eolian_deposit_frac).powf(1.0 / kf);
}

fn census(cells: &CellGrid, cfg: &DeepConfig, wp: usize, target: (f64, f64)) -> Census {
    let run = run_cells(cells, cfg, true);
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
        relief: 0.0,
        surf_mean: 0.0,
        conc_rms: 0.0,
        acf_x: [f64::NAN; 4],
        acf_y: [f64::NAN; 4],
        flip_x: 0.0,
        flip_y: 0.0,
        dacf_x: [f64::NAN; 4],
        dacf_y: [f64::NAN; 4],
        h_mean: 0.0,
        h_excess_rms: 0.0,
        r_conc_hexcess: f64::NAN,
        r_conc_h: f64::NAN,
        crust_floor_pct: 0.0,
        conc_r_rms: 0.0,
        acf_r: (f64::NAN, f64::NAN),
        conc_h_rms: 0.0,
        acf_h: (f64::NAN, f64::NAN),
        creep_limited_pct: f64::NAN,
    };
    // D1 needs the concavity **as a field**, not as a sorted list: its whole
    // question is what a cell's value says about its neighbour's. `ok` is the
    // same land+interior mask the census loop applies, so every statistic below
    // is over exactly the population the percentiles are over.
    let n_all = w * w;
    let surf: Vec<f64> = (0..n_all).map(|i| run.grid.surf_at(i)).collect();
    let mut cgrid = vec![f64::NAN; n_all];
    let mut ok = vec![false; n_all];
    let (mut g1, mut g5, mut g20) = (0usize, 0usize, 0usize);
    let mut vol = 0.0f64;
    let mut surf_sum = 0.0f64;
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
            let s = surf[i];
            surf_sum += s;
            let mut sum = 0.0;
            for (dx, dy) in NEIGH8 {
                sum += surf[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize];
            }
            let c = sum / 8.0 - s;
            conc.push(c);
            cgrid[i] = c;
            ok[i] = true;
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
    out.conc_rms = (conc.iter().map(|c| c * c).sum::<f64>() / n).sqrt();
    out.surf_mean = surf_sum / n;
    out.relief = surf.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - surf.iter().copied().fold(f64::INFINITY, f64::min);

    // ---- D1 ---------------------------------------------------------------
    out.acf_x = census::acf4(&cgrid, &ok, w, true);
    out.acf_y = census::acf4(&cgrid, &ok, w, false);
    out.flip_x = census::flip_rate(&cgrid, &ok, w, true);
    out.flip_y = census::flip_rate(&cgrid, &ok, w, false);
    // The surface's first difference, on the same mask. Its lag-1 autocorrelation
    // separates the same three worlds as the concavity's does but through a
    // *different* operator, so agreement between them is not a property of the
    // Laplacian (see `print_structure` for the three reference values).
    let (mut dx, mut dxok) = (vec![f64::NAN; n_all], vec![false; n_all]);
    let (mut dy, mut dyok) = (vec![f64::NAN; n_all], vec![false; n_all]);
    for y in 0..w {
        for x in 0..w {
            let i = y * w + x;
            if x + 1 < w && ok[i] && ok[i + 1] {
                dx[i] = surf[i + 1] - surf[i];
                dxok[i] = true;
            }
            if y + 1 < w && ok[i] && ok[i + w] {
                dy[i] = surf[i + w] - surf[i];
                dyok[i] = true;
            }
        }
    }
    out.dacf_x = census::acf4(&dx, &dxok, w, true);
    out.dacf_y = census::acf4(&dy, &dyok, w, false);

    // ---- D3 ---------------------------------------------------------------
    // `isostasy` computes its Airy target from the flexurally SMOOTHED loads and
    // then differences it against the cell's own UNSMOOTHED surface, so the term
    // that drives a cell's bedrock is proportional to the local excess `h − h̄`.
    // This is the same box smooth at the same radius the run itself used.
    let radius = isostasy::flex_radius_cells(cfg.flex_wavelength_km, run.grid.cell_m);
    let h_bar = isostasy::box_smooth(&run.grid.h, w, radius);
    let mut hx_pairs: Vec<(f64, f64)> = Vec::new();
    let mut h_pairs: Vec<(f64, f64)> = Vec::new();
    let (mut h_sum, mut hx_sq) = (0.0f64, 0.0f64);
    for i in 0..n_all {
        if !ok[i] {
            continue;
        }
        let excess = run.grid.h[i] - h_bar[i];
        h_sum += run.grid.h[i];
        hx_sq += excess * excess;
        hx_pairs.push((cgrid[i], excess));
        h_pairs.push((cgrid[i], run.grid.h[i]));
    }
    out.h_mean = h_sum / n;
    out.h_excess_rms = (hx_sq / n).sqrt();
    out.r_conc_hexcess = census::pearson(&hx_pairs);
    out.r_conc_h = census::pearson(&h_pairs);
    // D3(c): `apply_thickening` floors `t_crust` at 1000 m. A clamped cell beside
    // an unclamped one is a cell-to-cell discontinuity in the isostatic target.
    out.crust_floor_pct = if run.grid.t_crust.is_empty() {
        f64::NAN
    } else {
        100.0
            * run
                .grid
                .t_crust
                .iter()
                .filter(|t| **t <= 1000.0 + 1e-9)
                .count() as f64
            / run.grid.t_crust.len() as f64
    };

    // **Which field is oscillating?** `surf = r + h`, so a checkerboard in the
    // surface is a checkerboard in the bedrock top, in the regolith blanket, or in
    // both. The same Laplacian on the same mask, run over each summand separately,
    // says which — and that is what decides where a fix has to go.
    let lr = census::laplacian8(&run.grid.r, w);
    let lh = census::laplacian8(&run.grid.h, w);
    let rms = |f: &[f64]| -> f64 {
        let (mut s, mut c) = (0.0f64, 0usize);
        for i in 0..n_all {
            if ok[i] {
                s += f[i] * f[i];
                c += 1;
            }
        }
        (s / c.max(1) as f64).sqrt()
    };
    out.conc_r_rms = rms(&lr);
    out.conc_h_rms = rms(&lh);
    out.acf_r = (
        census::acf4(&lr, &ok, w, true)[0],
        census::acf4(&lr, &ok, w, false)[0],
    );
    out.acf_h = (
        census::acf4(&lh, &ok, w, true)[0],
        census::acf4(&lh, &ok, w, false)[0],
    );

    // The creep flux limiter's binding fraction — the counter journal/0114 measured
    // at 89–96 % and did not connect to the roughness. Only armed when the config
    // asks for it (`denudation_ledger`), so the default census stays on production's
    // code path.
    if cfg.denudation_ledger {
        let l = run.erosion.transport_ledger();
        out.creep_limited_pct =
            100.0 * l.creep_limited_cell_epochs as f64 / l.creep_cell_epochs.max(1) as f64;
    }
    out
}

/// D1's report block. **The three reference values in the caption are derived, not
/// remembered**, and they are the whole reason the numbers mean anything:
///
/// Concavity is `mean(8 neighbours) − self`. For a surface that is **white noise**
/// with variance `σ²`, that operator has `Var = (1 + 8/64)σ² = 1.125σ²` and lag-1
/// covariance `(−1/8 − 1/8 + 4/64)σ² = −0.1875σ²` (the two `−1/8` are the centre of
/// each cell meeting the other's neighbourhood; the `+4/64` is their four shared
/// neighbours), so **ACF(1) = −0.1875/1.125 = −1/6 ≈ −0.167**. For a perfect
/// **checkerboard** `±a`, the four diagonal neighbours carry the cell's own sign and
/// the four cardinal ones the opposite, so `mean(8) = 0` and the concavity field *is*
/// the checkerboard negated: **ACF(1) = −1, ACF(2) = +1**. For a **smooth** surface the
/// concavity is a smoothly varying field: **ACF(1) → +1**.
///
/// Sign-alternation follows from the same three: for jointly-Gaussian neighbours with
/// correlation `ρ`, `P(opposite sign) = arccos(ρ)/π` — **0.55** at white noise's
/// `ρ = −1/6`, **1.00** at a checkerboard's `ρ = −1`, and `→ 0` as `ρ → +1`.
///
/// So the discriminator is **not** "is ACF(1) negative" — a perfectly ordinary noisy
/// heightfield is already at −0.167. It is *how far past −1/6 it has gone*.
fn print_structure(c: &Census) {
    println!(
        "               D1 conc rms {:6.2} m | ACF(1..4) x {:+.3} {:+.3} {:+.3} {:+.3}   y {:+.3} {:+.3} {:+.3} {:+.3}",
        c.conc_rms,
        c.acf_x[0],
        c.acf_x[1],
        c.acf_x[2],
        c.acf_x[3],
        c.acf_y[0],
        c.acf_y[1],
        c.acf_y[2],
        c.acf_y[3]
    );
    println!(
        "                  sign-flip rate  x {:.3}  y {:.3}   [smooth→0.00, white noise 0.55, checkerboard 1.00]",
        c.flip_x, c.flip_y
    );
    println!(
        "                  d(surf) ACF(1..4) x {:+.3} {:+.3} {:+.3} {:+.3}   y {:+.3} {:+.3} {:+.3} {:+.3}   [smooth>0, white −0.50, checkerboard −1.00]",
        c.dacf_x[0],
        c.dacf_x[1],
        c.dacf_x[2],
        c.dacf_x[3],
        c.dacf_y[0],
        c.dacf_y[1],
        c.dacf_y[2],
        c.dacf_y[3]
    );
    println!(
        "               D3 h mean {:6.2} m | rms(h−h̄) {:6.2} m | corr(conc, h−h̄) {:+.3} | corr(conc, h) {:+.3} | t_crust at floor {:.2} %",
        c.h_mean, c.h_excess_rms, c.r_conc_hexcess, c.r_conc_h, c.crust_floor_pct
    );
    println!(
        "                  relief {:8.1} m | mean surface {:7.1} m",
        c.relief, c.surf_mean
    );
    // The limiter column is only real when the config armed `denudation_ledger`. Say so
    // rather than printing a bare `NaN` beside six live numbers — CLAUDE.md § Gates, "a
    // printed caption is a published claim the gate cannot check".
    let limiter = if c.creep_limited_pct.is_nan() {
        "n/a (denudation_ledger off — run with `-- --fields`)".to_string()
    } else {
        format!("{:.1} %", c.creep_limited_pct)
    };
    println!(
        "               WHICH FIELD  conc(r) rms {:6.2} m ACF1 x {:+.3} y {:+.3}  |  conc(h) rms {:6.2} m ACF1 x {:+.3} y {:+.3}  |  creep limiter bound {limiter}",
        c.conc_r_rms, c.acf_r.0, c.acf_r.1, c.conc_h_rms, c.acf_h.0, c.acf_h.1
    );
}

/// One row of the sweep tables: the numbers the RETURN spec asks for, per point.
fn sweep_row(label: &str, c: &Census, secs: f64) {
    println!(
        "  {label:22} conc p10 {:+7.1}  p90 {:+7.1}  p99 {:+7.1}  rms {:7.2} | hollows {:5} ({:4.1} %) deepest {:6.1} m | relief {:8.1} m  mean surf {:7.1} m | ACF_x(1) {:+.3} flip_x {:.3} | {secs:5.1} s",
        c.p10,
        c.p90,
        c.p99,
        c.conc_rms,
        c.hollow_1,
        100.0 * c.hollow_1 as f64 / c.land.max(1) as f64,
        c.deepest,
        c.relief,
        c.surf_mean,
        c.acf_x[0],
        c.flip_x
    );
}

/// **The discriminator sweep** (journal/0116). Three ladders on the production
/// world, each with the shipped arm carried alongside as the control.
fn run_sweep(pregen: &Pregen, t0: Instant) {
    let wp = pregen.deep.wp;
    let cells = &pregen.grid;
    let timed = |label: &str, cfg: &DeepConfig| -> Census {
        let t = Instant::now();
        let c = census(cells, cfg, wp, STATION_A);
        let secs = t.elapsed().as_secs_f64();
        sweep_row(label, &c, secs);
        c
    };

    println!("\n=== D1 · IS IT A CHECKERBOARD? (both arms, one solve each) ===");
    for (label, calibrated) in [("shipped", false), ("calibrated", true)] {
        let c = timed(label, &arm_cfg(cells, calibrated));
        print_structure(&c);
    }

    println!("\n=== D3 · THE ISOSTASY ABLATION (calibrated arm; iso_rate 0.5 is production) ===");
    for rate in [0.25f64, 0.0] {
        let mut cfg = arm_cfg(cells, true);
        cfg.iso_rate = rate;
        let c = timed(&format!("calibrated iso={rate:.2}"), &cfg);
        print_structure(&c);
    }
    {
        let mut cfg = arm_cfg(cells, false);
        cfg.iso_rate = 0.0;
        let c = timed("shipped iso=0.00", &cfg);
        print_structure(&c);
    }

    println!(
        "\n=== D2 · EPOCH REFINEMENT AT FIXED TOTAL TIME (calibrated arm; k× epochs, 1/k× rates) ==="
    );
    for k in [2u32, 4] {
        let mut cfg = arm_cfg(cells, true);
        refine_epochs(&mut cfg, k);
        let c = timed(&format!("calibrated k={k} ({} ep)", cfg.iterations), &cfg);
        print_structure(&c);
    }
    // The refinement operator's own falsifier: applied to the SHIPPED arm it must
    // leave the landscape where it is. If the shipped world moves under refinement,
    // the operator is changing the physics and D2's calibrated rows say nothing.
    {
        let mut cfg = arm_cfg(cells, false);
        refine_epochs(&mut cfg, 2);
        let c = timed("shipped k=2 (control)", &cfg);
        print_structure(&c);
    }

    println!("\ntotal sweep wall-clock {:?}", t0.elapsed());
}

/// **The follow-up the sweep earned** (journal/0116). D2 returned a null — refining
/// the step does not converge the oscillation away — and a null is only worth
/// something if you can say *why* the knob it turned was the wrong knob. Two
/// measurements answer that:
///
/// 1. **Which summand of `surf = r + h` carries the checkerboard.** A fix aimed at
///    the bedrock solve is wasted if the oscillation is in the regolith blanket.
/// 2. **Whether the creep flux limiter's binding fraction responds to the step at
///    all.** If it does not, the diffusion operator in this regime is *not a
///    function of `dt`* — it moves the whole cover regardless — and D2's null
///    stops being a mystery and becomes a prediction.
///
/// Runs with `denudation_ledger` armed, which is the only way to read that counter.
/// The concavity columns must reproduce the `--sweep` run's to the digit; if they
/// do not, the ledger is not the read-only instrument it claims to be, and **that**
/// is the finding.
fn run_fields(pregen: &Pregen, t0: Instant) {
    let wp = pregen.deep.wp;
    let cells = &pregen.grid;
    println!("\n=== WHICH FIELD OSCILLATES, AND IS THE LIMITER dt-DEPENDENT? ===");
    println!("(denudation_ledger armed; concavity columns must match the --sweep run)\n");
    for (label, calibrated, k) in [
        ("shipped k=1", false, 1u32),
        ("calibrated k=1", true, 1),
        ("calibrated k=2", true, 2),
        ("calibrated k=4", true, 4),
    ] {
        let mut cfg = arm_cfg(cells, calibrated);
        cfg.denudation_ledger = true;
        refine_epochs(&mut cfg, k);
        let t = Instant::now();
        let c = census(cells, &cfg, wp, STATION_A);
        sweep_row(label, &c, t.elapsed().as_secs_f64());
        print_structure(&c);
    }
    println!("\ntotal fields wall-clock {:?}", t0.elapsed());
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
        let c = census(&p.grid, &arm_cfg(&p.grid, false), p.deep.wp, STATION_A);
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
        let c = census(&p.grid, &arm_cfg(&p.grid, false), p.deep.wp, STATION_A);
        assert!(
            c.p99 < 5.0 && c.p10 > -5.0,
            "grid-scale roughness: concavity p10 {:+.1} m / p99 {:+.1} m exceeds ±5 m \
             (shipped spread is ±0.3 m). Adjacent cells are oscillating — see stubs #29.",
            c.p10,
            c.p99
        );
    }

    /// **The neighbour-relative half of the pair — the guard corrections #61 asks for.**
    ///
    /// The two guards above are **magnitude** claims (a count, a percentile). corrections
    /// #61's whole lesson is that a magnitude criterion cannot license a claim about
    /// *arrangement*, and this defect is an arrangement: journal/0116 measured a concavity
    /// lag-1 autocorrelation of **−0.87 / −0.91** on the calibrated world against
    /// **+0.38 / +0.27** on the shipped one, with the calibrated first-difference ACF at
    /// −0.86. So the pair is completed here rather than left as advice in a doc.
    ///
    /// **The bound is derived, and the derivation is the point** (the arithmetic is in
    /// [`print_structure`]): the concavity of a **white-noise** surface has ACF(1) = −1/6,
    /// and of a **perfect checkerboard** exactly −1. `−0.5` sits 3× past ordinary noise and
    /// 2× short of a pure oscillation, so it cannot be tripped by a rough-but-honest
    /// landscape and cannot miss grid instability. Same for the sign-flip ceiling: white
    /// noise alternates 55 % of the time, a checkerboard 100 %, and `0.85` is between them.
    ///
    /// **Why it is scale-free**, per CLAUDE.md § Gates: an autocorrelation at lag 1 is a
    /// statement about *pairs of adjacent cells*. It has no world-extent term in it — a
    /// bigger world supplies more pairs, not different ones.
    #[test]
    fn the_shipped_solve_has_no_grid_scale_oscillation() {
        let p = small();
        let c = census(&p.grid, &arm_cfg(&p.grid, false), p.deep.wp, STATION_A);
        assert!(
            c.acf_x[0] > -0.5 && c.acf_y[0] > -0.5,
            "concavity lag-1 autocorrelation x {:+.3} / y {:+.3} is past −0.5, heading for a \
             checkerboard's −1 (white noise is −0.167). Adjacent cells are oscillating \
             against each other — see stubs #29 and journal/0116.",
            c.acf_x[0],
            c.acf_y[0]
        );
        assert!(
            c.flip_x < 0.85 && c.flip_y < 0.85,
            "concavity sign-alternation x {:.3} / y {:.3} exceeds 0.85, approaching a \
             checkerboard's 1.00 (white noise is 0.55) — see journal/0116.",
            c.flip_x,
            c.flip_y
        );
    }

    /// **THE OTHER ARM — without this the three guards above are unfalsifiable.**
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
    ///
    /// # ⚠ Where the positive control moved, 2026-07-29 (journal/0122)
    ///
    /// It used to be the **calibrated** arm, because the calibrated arm was broken. The
    /// operator is fixed, so that arm no longer carries the defect — and this test
    /// promptly failed, which is the test working: it said *"I can no longer see the
    /// thing I exist to be able to see."*
    ///
    /// The right answer is **not** to loosen it. The defect is still reachable by
    /// design, as `DeepConfig::creep_substep: false` — the pre-0122 operator, kept as a
    /// fixed point — so the positive control moves there and this test keeps asserting
    /// exactly what it always did: *the same instrument at the same size can still see
    /// a world with the defect in it.* Named
    /// `the_calibrated_solve_still_shows_the_defect_at_this_size` until then;
    /// journal/0116 cites the old name.
    #[test]
    fn the_unbounded_operator_still_shows_the_defect_at_this_size() {
        let p = small();
        let cfg = DeepConfig {
            creep_substep: false,
            ..arm_cfg(&p.grid, true)
        };
        let c = census(&p.grid, &cfg, p.deep.wp, STATION_A);
        assert!(
            c.hollow_1 > 0,
            "the unbounded arm produced NO closed hollows at Extent::Small, so the two \
             shipped-world guards here are vacuous at this size — move them to Medium"
        );
        assert!(
            c.p99 > 5.0,
            "the unbounded calibrated arm's concavity p99 is only {:+.1} m at Extent::Small, below \
             the ±5 m bound the shipped guard asserts — the guard cannot discriminate at \
             this size and must move to Medium",
            c.p99
        );
        assert!(
            c.acf_x[0] < -0.5 || c.acf_y[0] < -0.5,
            "the unbounded calibrated arm's concavity lag-1 autocorrelation is only x {:+.3} / \
             y {:+.3} at Extent::Small — above the −0.5 bound the shipped guard asserts, so \
             `the_shipped_solve_has_no_grid_scale_oscillation` cannot discriminate at this \
             size and must move to Medium",
            c.acf_x[0],
            c.acf_y[0]
        );
        println!(
            "calibrated @ Small: {} hollows (deepest {:.1} m), concavity p10 {:+.1} / \
             p99 {:+.1}, ACF(1) x {:+.3} / y {:+.3}, sign-flip x {:.3} / y {:.3} — the \
             defect is visible at this size, in magnitude AND in structure",
            c.hollow_1, c.deepest, c.p10, c.p99, c.acf_x[0], c.acf_y[0], c.flip_x, c.flip_y
        );
    }
}
