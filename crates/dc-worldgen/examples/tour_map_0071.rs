//! Guided-tour map for the thickness-dominance outcrop walk (journal/0071).
//!
//! journal/0068 replaced the charcoal carve-out with a general rule: a cell's
//! outcrop is the lithology dominating the top `OUTCROP_DOMINANCE_WINDOW_M`
//! (0.9 m) of its record, with the sub-record deficit counting as basement.
//! 39.9 % of Medium cells changed outcrop (measured on the probe seed); the
//! user walks the *client* world to ratify the look. Lean by request: one
//! production run, stations picked from the record's composition — no pre/post
//! sim diff.
//!
//! Stations:
//!   1. armored coast   — coastal cell whose outcrop flipped fine→basement
//!   2. flipped plain   — interior cell, largest basement-deficit flip
//!   3. coal seam       — thickest Coal-facies column (with burial context)
//!   4. charcoal face — most fire beds in one column (regression check vs 0066)
//!
//! Expectation management is printed per station: the lithology change is in
//! the DATA at every flipped station; whether relief reads at walking scale is
//! the journal/0030 amplitude question, and a null eye-verdict is a verdict.
//!
//! `cargo run --release -p dc-worldgen --example tour_map_0071`

use dc_worldgen::deeptime::lithology::{Litho, exposed_litho};
use dc_worldgen::deeptime::recorder::Biofacies;
use dc_worldgen::deeptime::{self, DepUnit, litho_of_tag, production_config};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

/// The client's `BENCH_SEED` (dc-client/src/bench.rs) — the world the walk
/// stands in.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;
/// Border ring (deep cells) excluded from station picks (march artifacts).
const EDGE_MARGIN: i64 = 4;

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

/// The pre-journal/0068 rule: top unit's lithology, basement if empty.
fn old_rule(units: &[DepUnit]) -> Litho {
    match units.last() {
        Some(u) => litho_of_tag(u.tag),
        None => Litho::Basement,
    }
}

/// Total recorded thickness of a column.
fn record_m(units: &[DepUnit]) -> f64 {
    units.iter().map(|u| u.thickness_m).sum()
}

/// The deep-cell ↔ world-voxel bridge (as tour_map.rs, from roughness_probe).
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

fn local_relief(grid: &deeptime::DeepGrid, idx: usize, radius: i64) -> f64 {
    let w = grid.w as i64;
    let (cx, cy) = ((idx % grid.w) as i64, (idx / grid.w) as i64);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= w || y >= w {
                continue;
            }
            let s = grid.surf_at((y * w + x) as usize);
            lo = lo.min(s);
            hi = hi.max(s);
        }
    }
    hi - lo
}

fn header(title: &str) {
    println!("\n============================================================");
    println!("{title}");
    println!("============================================================");
}

fn main() {
    println!("=== journal/0071 tour map: the thickness-dominance outcrop walk ===");
    println!(
        "seed {SEED}, extent {}, production config\n",
        EXTENT.label()
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let cfg = production_config(&pregen.grid, SEED);
    let run = deeptime::run_with(&pregen, &cfg, true);
    let grid = &run.grid;
    let w = grid.w;
    let conv = Conv {
        w,
        wp: pregen.deep.wp,
    };

    // Walk-world flip census first, so the station picks have context.
    let (mut recorded, mut flipped) = (0usize, 0usize);
    for s in &grid.strata {
        if s.units.is_empty() {
            continue;
        }
        recorded += 1;
        if old_rule(&s.units) != exposed_litho(&s.units) {
            flipped += 1;
        }
    }
    println!(
        "walk-world census: {recorded} recorded cells, {flipped} flipped outcrop \
         ({:.1} %)\n",
        flipped as f64 * 100.0 / recorded.max(1) as f64
    );

    // STATION 1 — armored coast: coastal cell flipped →basement, thinnest record
    // (largest deficit share), preferring local relief so there is a cliff line
    // to look along.
    let band = deeptime::DeepConfig::default().wave_band_m;
    let coast = |i: usize| -> bool {
        let free = grid.surf_at(i);
        if !(free > 0.0 && free <= band) {
            return false;
        }
        let (cx, cy) = ((i % w) as i64, (i / w) as i64);
        for (dx, dy) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
            let (x, y) = (cx + dx, cy + dy);
            if x >= 0
                && y >= 0
                && x < w as i64
                && y < w as i64
                && grid.surf_at((y * w as i64 + x) as usize) <= 0.0
            {
                return true;
            }
        }
        false
    };
    let mut best_coast: Option<(usize, f64)> = None; // (idx, score)
    for i in 0..w * w {
        if !interior(w, i) || !coast(i) {
            continue;
        }
        let units = &grid.strata[i].units;
        if units.is_empty()
            || exposed_litho(units) != Litho::Basement
            || old_rule(units) == Litho::Basement
        {
            continue;
        }
        // Score: relief matters more than thinness for a lookable station.
        let score = local_relief(grid, i, 2) - record_m(units);
        if best_coast.is_none() || score > best_coast.unwrap().1 {
            best_coast = Some((i, score));
        }
    }
    header("STATION 1 — ARMORED COAST (flipped →basement)");
    match best_coast {
        Some((i, _)) => {
            let units = &grid.strata[i].units;
            println!("  {}", conv.where_line(i));
            println!("  surface elevation : {:.1} m", grid.surf_at(i));
            println!(
                "  record            : {:.2} m over basement ({} units) — window deficit {:.0} %",
                record_m(units),
                units.len(),
                (1.0 - (record_m(units) / 0.9).min(1.0)) * 100.0
            );
            println!(
                "  outcrop           : old {} → new {}",
                old_rule(units).code(),
                exposed_litho(units).code()
            );
            println!("  local relief (±2) : {:.1} m", local_relief(grid, i, 2));
            println!(
                "  EXPECT: the DATA says this coast now erodes as rock platform, not mud.\n\
                 \x20        Whether the shore FORM reads differently at production amplitude is\n\
                 \x20        the open amplitude question — a null eye-verdict here is a verdict."
            );
        }
        None => println!("  NULL: no interior coastal cell flipped to basement on this world."),
    }

    // STATION 2 — flipped plain: interior, non-coast, flipped →basement, largest
    // deficit, i.e. the purest case of "thin veneer erodes as platform".
    let mut best_plain: Option<(usize, f64)> = None; // (idx, deficit share)
    for i in 0..w * w {
        if !interior(w, i) || coast(i) || grid.surf_at(i) <= 0.0 {
            continue;
        }
        let units = &grid.strata[i].units;
        if units.is_empty()
            || exposed_litho(units) != Litho::Basement
            || old_rule(units) == Litho::Basement
        {
            continue;
        }
        let deficit = 1.0 - (record_m(units) / 0.9).min(1.0);
        if best_plain.is_none() || deficit > best_plain.unwrap().1 {
            best_plain = Some((i, deficit));
        }
    }
    header("STATION 2 — FLIPPED PLAIN (thin veneer over platform)");
    match best_plain {
        Some((i, deficit)) => {
            let units = &grid.strata[i].units;
            println!("  {}", conv.where_line(i));
            println!("  surface elevation : {:.1} m", grid.surf_at(i));
            println!(
                "  record            : {:.2} m ({} units), window deficit {:.0} %",
                record_m(units),
                units.len(),
                deficit * 100.0
            );
            println!(
                "  outcrop           : old {} → new {}",
                old_rule(units).code(),
                exposed_litho(units).code()
            );
            println!("  local relief (±2) : {:.1} m", local_relief(grid, i, 2));
            println!(
                "  EXPECT: likely a NULL for the eye — a plain that stops eroding looks like\n\
                 \x20        a plain. The station exists to stand on the most-changed ground and\n\
                 \x20        say so honestly."
            );
        }
        None => println!("  NULL: no interior plain cell flipped to basement."),
    }

    // STATION 3 — coal seam: thickest total Coal-facies column, with burial.
    let mut best_coal: Option<(usize, f64)> = None;
    for i in 0..w * w {
        if !interior(w, i) {
            continue;
        }
        let coal: f64 = grid.strata[i]
            .units
            .iter()
            .filter(|u| u.tag.biota == Biofacies::Coal)
            .map(|u| u.thickness_m)
            .sum();
        if coal > 0.0 && (best_coal.is_none() || coal > best_coal.unwrap().1) {
            best_coal = Some((i, coal));
        }
    }
    header("STATION 3 — COAL SEAM");
    match best_coal {
        Some((i, coal)) => {
            let units = &grid.strata[i].units;
            // Burial of the topmost coal unit: thickness above it.
            let top_coal = units
                .iter()
                .rposition(|u| u.tag.biota == Biofacies::Coal)
                .expect("column has coal");
            let burial: f64 = units[top_coal + 1..].iter().map(|u| u.thickness_m).sum();
            println!("  {}", conv.where_line(i));
            println!("  surface elevation : {:.1} m", grid.surf_at(i));
            println!(
                "  coal              : {coal:.2} m total, topmost seam under {burial:.2} m of section"
            );
            println!(
                "  outcrop           : old {} → new {}",
                old_rule(units).code(),
                exposed_litho(units).code()
            );
            println!(
                "  EXPECT: dig the face to reach it if buried; the seam exists because THIS\n\
                 \x20        column's peat crossed the burial threshold (journal/0066/0067)."
            );
        }
        None => println!("  NULL: no coal-facies units recorded on this world."),
    }

    // STATION 4 — charcoal face: most fire beds in one column (regression check).
    let mut best_char: Option<(usize, usize, f64)> = None; // (idx, beds, total m)
    for i in 0..w * w {
        if !interior(w, i) || grid.surf_at(i) <= 0.0 {
            continue;
        }
        let (mut beds, mut m) = (0usize, 0.0f64);
        for u in &grid.strata[i].units {
            if u.tag.biota == Biofacies::Charcoal {
                beds += 1;
                m += u.thickness_m;
            }
        }
        if beds > 0 && (best_char.is_none() || beds > best_char.unwrap().1) {
            best_char = Some((i, beds, m));
        }
    }
    header("STATION 4 — CHARCOAL FACE (regression check vs 0066)");
    match best_char {
        Some((i, beds, m)) => {
            let units = &grid.strata[i].units;
            println!("  {}", conv.where_line(i));
            println!("  surface elevation : {:.1} m", grid.surf_at(i));
            println!(
                "  fire history      : {beds} charcoal beds, {m:.3} m total (mean {:.3} m)",
                m / beds as f64
            );
            println!(
                "  outcrop           : old {} → new {} (a fire bed must NEVER be the outcrop)",
                old_rule(units).code(),
                exposed_litho(units).code()
            );
            println!(
                "  EXPECT: black specks in a cut face, not bands, only in fire layers —\n\
                 \x20        IDENTICAL to 0066. Any visible change here is a defect, not a feature."
            );
        }
        None => println!("  NULL: no fire-bearing columns on this world."),
    }

    println!(
        "\n(pose_set {{ surface: true }} for walker-safe teleports; check eye_in_solid\n\
         before trusting any screenshot. Lit pass — this walk asks shape questions.)"
    );
}
