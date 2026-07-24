//! Palette-quant checkerboard exemplar finder (journal/0088 reference recovery).
//!
//! journal/0088 captured a top-down frame of the east coast that read as a grid of
//! squares, each a distinctly different overall material tint (tan / grey /
//! red-brown / dark-speckled) — the "palette-quant" chunk-seam checkerboard. The
//! camera pose was never recorded; only "east coast, ~110 km east of spawn,
//! top-down." This binary re-establishes and records that reference.
//!
//! Mechanism (`docs/audits/2026-07-24-palette-quant-generation-diagnosis.md`): a
//! chunk's whole strata composition is ONE point-sample of the deep record at the
//! chunk centre, NEAREST at the ~460 m deep-cell grid. So the tint of a square is
//! the DOMINANT SURFACE CLASS of the deep cell it lands in, uniform within a cell
//! and stepping only across the ~460 m cell frontier. The strongest checkerboard
//! is therefore a patch where several ADJACENT deep cells carry the most DISTINCT
//! (and balanced) dominant classes.
//!
//! This rebuilds the PRODUCTION world the dc-client boots — seed 1337,
//! `Extent::Medium`, default `DeepOverrides` (the exact `Pregen::run_with` in
//! `dc-client::authority::new_worldgen`) — and, over an east-coast search box,
//! scores WIN×WIN windows of deep cells by distinct + balanced dominant classes.
//! It prints the argmax window centre in WORLD METRES, the class layout, and a
//! top-down camera pose, plus runner-up windows.
//!
//! Nothing here is on a generation path. The dominant class is read the same way
//! the collapse's `surface_class` reads it — the top `VOXEL_M` of
//! `DeepField::record_at_voxel` grouped by `geology::deep_class` — but taking the
//! plurality (the tint) rather than the probabilistic per-chunk draw.
//!
//! `cargo run --release -p dc-worldgen --example palette_quant_tour`

use dc_core::materials::geology::{CLASS_IGNEOUS_EXTRUSIVE, CLASS_IGNEOUS_INTRUSIVE};
use dc_worldgen::deeptime::{DeepField, DeepStrata};
use dc_worldgen::geology::deep_class;
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, Provenance, WorldParams};
use dc_worldgen::{DeepOverrides, WorldGenerator};

// ---- production world identity (mirrors dc-client::authority) ----
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
/// N=2 player scale ⇒ 0.9 m voxels. The top-of-record window `surface_class`
/// classifies is exactly one voxel deep.
const VOXEL_M: f64 = 0.9;

// ---- search box, WORLD METRES (east coast; +X = east, +Z = south) ----
// The immediate shoreline (~x 108–109 km) is uniform grey stone + water; the
// multi-class checkerboard sits inland of it, so the box is deliberately wide and
// reaches west of the coast. Widened past the brief's [104,116]×[-4,16] km on
// purpose — the score is what localises the patch, not the box.
const X_MIN_M: f64 = 92_000.0;
const X_MAX_M: f64 = 118_000.0;
const Z_MIN_M: f64 = -12_000.0;
const Z_MAX_M: f64 = 22_000.0;

/// Window edge, in deep cells. 3×3 ≈ 1.4 km — a top-down frame holding a small
/// grid of ~460 m tiles. Rewarded for 3–5 distinct, balanced dominant classes.
const WIN: usize = 3;

/// Bevy's default perspective vertical FOV (π/4). Used to size the camera
/// altitude that frames the whole window looking straight down.
const FOV_V: f64 = std::f64::consts::FRAC_PI_4;

/// World voxel centre of deep cell `idx` (inverse of `DeepField::deep_coords`,
/// lifted from `s18_weathering_tour.rs`).
struct Conv {
    w: usize,
    wp: usize,
}
impl Conv {
    fn idx_to_voxel(&self, gx: usize, gy: usize) -> (i64, i64) {
        let (w, wp) = (self.w as f64, self.wp as f64);
        let (gx, gy) = (gx as f64, gy as f64);
        let px = (gx + 0.5) / w * wp - 0.5;
        let py = (gy + 0.5) / w * wp - 0.5;
        let half = (self.wp / 2) as f64;
        (
            ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
            ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        )
    }
    fn idx_to_world_m(&self, gx: usize, gy: usize) -> (f64, f64) {
        let (vx, vz) = self.idx_to_voxel(gx, gy);
        (vx as f64 * VOXEL_M, vz as f64 * VOXEL_M)
    }
}

/// The dominant (plurality) surface class of a deep cell: group the top `VOXEL_M`
/// of the record by `deep_class`, return the class with the largest share. `None`
/// when less than half a voxel of record accumulates (the bare-rock / basement
/// case `surface_class` hands to the igneous province).
fn dominant_sedimentary(strata: &DeepStrata) -> Option<&'static str> {
    let mut acc = 0.0f64;
    let mut by_class: Vec<(&'static str, f64)> = Vec::new();
    for u in strata.units.iter().rev() {
        if acc >= VOXEL_M {
            break;
        }
        let take = u.thickness_m.min(VOXEL_M - acc);
        if take <= 0.0 {
            continue;
        }
        acc += take;
        let c = deep_class(u.tag);
        match by_class.iter_mut().find(|(k, _)| *k == c) {
            Some((_, m)) => *m += take,
            None => by_class.push((c, take)),
        }
    }
    if acc < VOXEL_M / 2.0 {
        return None;
    }
    by_class
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(c, _)| c)
}

/// The full surface class of a deep cell, matching `collapse::surface_class`'s
/// fallback: sedimentary plurality, else the igneous class the province would
/// emplace (so bare-rock cells still carry their grey/dark basement tint).
fn cell_class(field: &DeepField, pregen: &Pregen, idx: usize, conv: &Conv) -> Option<&'static str> {
    if let Some(c) = dominant_sedimentary(&field.strata[idx]) {
        return Some(c);
    }
    let (vx, vz) = conv.idx_to_voxel(idx % conv.w, idx / conv.w);
    let (gx, gy) = pregen.grid.cell_of_voxel(vx, vz);
    let provenance = match (i32::try_from(gx), i32::try_from(gy)) {
        (Ok(x), Ok(y)) => pregen.grid.get(x, y).map(|c| c.provenance)?,
        _ => return None,
    };
    match provenance {
        Provenance::Orogeny | Provenance::Arc => Some(CLASS_IGNEOUS_INTRUSIVE),
        Provenance::Rift => Some(CLASS_IGNEOUS_EXTRUSIVE),
        _ => None,
    }
}

/// One-letter glyph per class so a window's layout prints as a legible grid.
fn glyph(class: &str) -> char {
    // Class ids are hyphenated & namespaced, e.g. `dc:stratum/clastic-coarse`.
    match class {
        c if c.contains("clastic-coarse") => 'S', // sandstone / tan
        c if c.contains("clastic-fine") => 'm',   // mudstone / grey-brown
        c if c.contains("igneous-intrusive") => 'G', // granite / grey
        c if c.contains("igneous-extrusive") => 'B', // basalt / dark
        c if c.contains("accessory-mafic") => 'M',
        c if c.contains("organic-charcoal") => 'H',
        c if c.contains("organic-coal") => 'C', // coal / dark-speckled
        c if c.contains("organic-peat") => 'P',
        c if c.contains("organic-soil") => 'o',
        c if c.contains("ore-placer") => 'r',
        _ => '?',
    }
}

/// Distinct + balance score over a window's cells. Coverage-weighted so a window
/// half in the wilds/water (many `None`) cannot win on a couple of loud cells.
struct WinScore {
    score: f64,
    n_distinct: usize,
    hnorm: f64,
    coverage: f64,
    counts: Vec<(&'static str, usize)>,
}
fn window_score(cells: &[Option<&'static str>]) -> WinScore {
    let total_slots = cells.len();
    let present: Vec<&'static str> = cells.iter().filter_map(|c| *c).collect();
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for &c in &present {
        match counts.iter_mut().find(|(k, _)| *k == c) {
            Some((_, n)) => *n += 1,
            None => counts.push((c, 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let n_distinct = counts.len();
    let present_n = present.len() as f64;
    let coverage = present_n / total_slots as f64;
    let hnorm = if n_distinct > 1 {
        let h: f64 = counts
            .iter()
            .map(|(_, n)| {
                let p = *n as f64 / present_n;
                -p * p.ln()
            })
            .sum();
        h / (n_distinct as f64).ln()
    } else {
        0.0
    };
    // Distinct is the primary axis but rewarded most in the 3–5 band (the original
    // tan/grey/red/dark); balance (entropy) and coverage break ties and punish a
    // lone-loud-cell window.
    let distinct_reward = match n_distinct {
        0 | 1 => n_distinct as f64 * 0.25,
        2 => 1.4,
        3 => 3.0,
        4 => 3.8,
        5 => 4.2,
        _ => 4.0, // 6+ distinct is busier than the reference, mild demotion
    };
    let score = coverage * (distinct_reward + 1.5 * hnorm);
    WinScore {
        score,
        n_distinct,
        hnorm,
        coverage,
        counts,
    }
}

fn main() {
    println!("=== palette-quant checkerboard exemplar finder (journal/0088) ===");
    println!(
        "seed {SEED}, extent {}, default deep overrides, N=2 ({VOXEL_M} m voxels)",
        EXTENT.label()
    );

    let pregen = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides::default(),
    );
    let field = &pregen.deep;
    let (w, wp) = (field.w, field.wp);
    let conv = Conv { w, wp };
    let extent_m = wp as f64 * CELL_VOXELS as f64 * VOXEL_M;
    let half_m = extent_m / 2.0;
    println!(
        "deep grid {w}x{w} @ {:.1} m/cell; pregen {wp}x{wp}; civilized extent ±{:.0} m ({:.0} km)\n",
        field.cell_m,
        half_m,
        half_m / 1000.0
    );
    println!(
        "search box (world m): x [{:.0},{:.0}], z [{:.0},{:.0}]  (WIN={WIN} cells ≈ {:.0} m)\n",
        X_MIN_M,
        X_MAX_M,
        Z_MIN_M,
        Z_MAX_M,
        WIN as f64 * field.cell_m
    );

    // Dominant class of every deep cell (cheap; w is a few hundred at most).
    let class: Vec<Option<&'static str>> = (0..w * w)
        .map(|i| cell_class(field, &pregen, i, &conv))
        .collect();

    // Score every WIN×WIN window whose CENTRE cell falls inside the search box.
    struct Cand {
        gx: usize,
        gy: usize, // top-left cell of the window
        cx_m: f64,
        cz_m: f64, // window centre, world metres
        s: WinScore,
        grid: Vec<Option<&'static str>>,
    }
    let mut cands: Vec<Cand> = Vec::new();
    let mut box_edge_hit = false;
    for gy in 0..w.saturating_sub(WIN - 1) {
        for gx in 0..w.saturating_sub(WIN - 1) {
            let (ccx, ccy) = (gx + WIN / 2, gy + WIN / 2);
            let (cx_m, cz_m) = conv.idx_to_world_m(ccx, ccy);
            if !(X_MIN_M..=X_MAX_M).contains(&cx_m) || !(Z_MIN_M..=Z_MAX_M).contains(&cz_m) {
                continue;
            }
            let mut grid = Vec::with_capacity(WIN * WIN);
            for dy in 0..WIN {
                for dx in 0..WIN {
                    grid.push(class[(gy + dy) * w + (gx + dx)]);
                }
            }
            let s = window_score(&grid);
            // Note whether the strongest cands ride the box edge (would justify
            // widening): flagged after sort.
            let near_edge = cx_m < X_MIN_M + field.cell_m
                || cx_m > X_MAX_M - field.cell_m
                || cz_m < Z_MIN_M + field.cell_m
                || cz_m > Z_MAX_M - field.cell_m;
            if near_edge && s.n_distinct >= 4 {
                box_edge_hit = true;
            }
            cands.push(Cand {
                gx,
                gy,
                cx_m,
                cz_m,
                s,
                grid,
            });
        }
    }

    if cands.is_empty() {
        println!("NULL: the search box holds no deep cells — outside the civilized extent?");
        return;
    }
    cands.sort_by(|a, b| b.s.score.total_cmp(&a.s.score));

    // Spatially de-duplicated top picks (≥ WIN cells apart) so the runners-up are
    // genuinely different patches, not neighbours of the winner.
    let mut picked: Vec<&Cand> = Vec::new();
    for c in &cands {
        if picked.len() >= 4 {
            break;
        }
        let far = picked.iter().all(|p| {
            let dx = c.gx as i64 - p.gx as i64;
            let dy = c.gy as i64 - p.gy as i64;
            dx.abs().max(dy.abs()) >= WIN as i64
        });
        if far {
            picked.push(c);
        }
    }

    // Elevations for the picks come from the production near-field generator (the
    // true voxel surface, river carving + relief), not the coarse deep bilinear.
    let mut wgen = WorldGenerator::new(&pregen);

    let alt = WIN as f64 * field.cell_m / (2.0 * (FOV_V / 2.0).tan());
    println!("--- ARGMAX and alternates (top-down, pitch -1.55, yaw 0) ---");
    println!(
        "camera altitude to frame a {WIN}x{WIN} tile window at π/4 FOV: {alt:.0} m above ground\n"
    );

    for (rank, c) in picked.iter().enumerate() {
        let tag = if rank == 0 { "ARGMAX" } else { "ALT" };
        let (ccx, ccy) = (c.gx + WIN / 2, c.gy + WIN / 2);
        let (vx, vz) = conv.idx_to_voxel(ccx, ccy);
        let surf = wgen.surface_elev_m(vx, vz);
        let deep_surf = field
            .surface_at_voxel(vx, vz)
            .unwrap_or(field.surf[ccy * w + ccx]);
        println!(
            "{tag} #{}  score {:.3}  | {} distinct, balance {:.2}, coverage {:.0}%",
            rank + 1,
            c.s.score,
            c.s.n_distinct,
            c.s.hnorm,
            c.s.coverage * 100.0
        );
        println!(
            "  window centre (world m): x {:.0}, z {:.0}",
            c.cx_m, c.cz_m
        );
        println!(
            "  near-field surface elev (worldgen frame): {surf:.1} m  (deep bilinear {deep_surf:.1} m)"
        );
        print!("  dominant classes: ");
        for (cl, n) in &c.s.counts {
            print!("{cl}×{n}  ");
        }
        println!();
        println!("  tile layout ({WIN}x{WIN}, '.' = wilds/water/basement-less):");
        for row in 0..WIN {
            print!("      ");
            for col in 0..WIN {
                let g = c.grid[row * WIN + col].map(glyph).unwrap_or('.');
                print!("{g} ");
            }
            println!();
        }
        // Camera pose. surf is the worldgen frame; the live client frame may carry
        // a constant vertical offset (the 0088 session read east-coast ground near
        // -440 m). So the robust recipe is: teleport (x,z,surface:true) to read the
        // true ground Y G, then set feet.y = G + altitude.
        println!(
            "  POSE (worldgen frame): feet ({:.0}, {:.0}, {:.0}), pitch -1.55, yaw 0",
            c.cx_m,
            surf + alt,
            c.cz_m
        );
        println!(
            "        close-in +90 m variant (one boundary, chunk detail): feet ({:.0}, {:.0}, {:.0})",
            c.cx_m,
            surf + 90.0,
            c.cz_m
        );
        println!(
            "        LIVE recipe: pose_set{{x:{:.0}, z:{:.0}, surface:true}} → read ground Y G → pose_set{{x:{:.0}, y:G+{alt:.0}, z:{:.0}, pitch:-1.55, yaw:0}}\n",
            c.cx_m, c.cz_m, c.cx_m, c.cz_m
        );
    }

    println!("--- glyph legend ---");
    println!("  S clastic_coarse (tan) | m clastic_fine (grey-brown) | G igneous_intrusive (grey)");
    println!("  B igneous_extrusive (dark) | M accessory_mafic | C organic_coal (dark-speckled)");
    println!(
        "  P peat | H charcoal | o organic_soil | r ore_placer | . none (wilds/water/basement-less)\n"
    );

    if box_edge_hit {
        println!(
            "NOTE: a strong (≥4-distinct) window rides the search-box edge — widen the box and re-run to be sure the true argmax is not just outside it."
        );
    } else {
        println!("NOTE: the strongest windows sit inside the box interior; no widening indicated.");
    }
    // Honest global check: is there ANY strong multi-class window near the east
    // coast at all, or is the whole coast uniform?
    let best = &cands[0];
    if best.s.n_distinct < 3 {
        println!(
            "CAVEAT / possible NULL: the best east-coast window holds only {} distinct dominant class(es) — the coast may simply be near-uniform (no vivid tan/grey/red/dark checkerboard). Verify live before recording.",
            best.s.n_distinct
        );
    }
}
