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
//! Nothing here is on a generation path. The dominant class is a deep cell's own
//! plurality: the top `VOXEL_M` of its record, grouped by
//! `geology::deep_class_of_species`, argmax.
//!
//! **⚠ That is no longer how `surface_class` reads it** (member #0, journal/0125).
//! This caption used to say "read the same way the collapse's `surface_class`
//! reads it", and until 2026-07-29 that was exact — both took the NEAREST deep
//! cell's top-`VOXEL_M` shares. `surface_class` now samples a
//! `CoarseField<ShareVec<6>>` through `sample_dithered`, which **dithers which of
//! the four surrounding cells** supplies the shares before drawing a class. So:
//!
//! - the per-cell tint this binary prints is still the right thing to score a
//!   checkerboard by — it is the cell's own composition, which is what a tile of
//!   ground reads as at a distance — but it is now the cell's **centre** answer;
//! - the ~460 m frontier between two tints is **no longer a step**. Expect the
//!   scored windows to be there and the *lines between them* to be interfingered
//!   rather than straight. A frame that still shows a razor-straight 460 m edge
//!   after this is a regression, not the signature.
//!
//! **⚠ AND THE NEAR-FIELD HALF IS NOW HALF-FIXED TOO** (journal/0129). This caption
//! said the near mechanism was *"untouched … still exactly as described"*, and after
//! the octaves slice that is true of **one** of U3's two signals and false of the
//! other. Split them, because they are different scales and different fixes:
//!
//! - **the 28.8 m MEMBER stepping — FIXED.** The within-class member dither read a
//!   *single* bilinear octave at chunk wavelength, so every member patch was
//!   chunk-sized and every contact kinked on the 28.8 m grid (corrections #45; the
//!   dominant U3 signal, settled 2026-07-24). It now reads `draws::Octaves`.
//!   Measured at the reference pose by [`report_u3_member_stepping`] below, which is
//!   why this binary grew a second half.
//! - **the ~460 m RECORD tile — NOT FIXED.** A chunk's whole strata composition is
//!   still ONE point sample of the deep record at the chunk centre, NEAREST at the
//!   deep-cell grid — the paragraph above, still exact. The near-path restructure
//!   that fixes it (per-column membership dither over the touched cells) is
//!   sequenced, not shipped; its blockers MM-1 and MM-3 are discharged.
//!
//! So the tile scoring above is still the right instrument for the 460 m signature,
//! and it is now the *only* signature it scores.
//!
//! `cargo run --release -p dc-worldgen --example palette_quant_tour`

use dc_core::coarse::DitherSource;
use dc_core::materials::geology::{CLASS_IGNEOUS_EXTRUSIVE, CLASS_IGNEOUS_INTRUSIVE};
use dc_sim::statistical::rng::Draws;
use dc_worldgen::deeptime::{DeepField, DeepStrata};
use dc_worldgen::draws::{Coherent, Octaves};
use dc_worldgen::geology::{StrataEvent, deep_class_of_species, dithered_member_with};
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
        let c = deep_class_of_species(Litho::of_material(u.species));
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

    report_u3_member_stepping(&mut wgen);

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

// ─────────── U3's member-stepping signature, measured A/B in one binary ──────────
//
// journal/0129. The station finder above answers *where to stand*; this answers
// *what changed under your feet*, at the U3 reference pose, with the load-bearing
// number rather than an adjective.

/// U3's reference pose, feet in world **metres** (recorded 2026-07-24, corrections
/// #48's rule — a prose landmark is not a pose).
const U3_FEET_M: (f64, f64, f64) = (71_291.7, 372.1, -2_420.9);

/// Half-edge of the measurement window, voxels. 128 → a 256-voxel (230 m) square,
/// eight chunks across, wide enough that a 28.8 m signature has eight periods to
/// show itself in and small enough to stay inside one deep cell's record.
const WIN_VOX: i64 = 128;

/// Lags the member-field autocorrelation is reported at, voxels.
const LAGS: [i64; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// **The square-edge signature: how much of a selection field's curvature sits on
/// the 28.8 m chunk lattice.**
///
/// A bilinear corner field is *linear along x inside a cell*, so its second
/// difference along x is zero except where the three-point window straddles a cell
/// line. A field whose only cell is the chunk therefore puts **all** of its
/// curvature — every kink in every member contact it draws — on the 28.8 m grid,
/// and that is what the eye reads as squares (corrections #45: world-anchored and
/// C0-continuous, and still a grid, because *single-octave*).
///
/// Returns `(mean |Δ²u| where the window crosses a 32-line, mean |Δ²u| elsewhere)`.
/// "Crosses" is phases 0 **and 31**, not phase 0 alone — `x + 1` is in the next
/// cell.
fn lattice_curvature(src: &dyn DitherSource, cx: i64, cz: i64, tag: u64) -> (f64, f64) {
    let (mut on, mut on_n, mut off, mut off_n) = (0.0f64, 0u64, 0.0f64, 0u64);
    for z in (cz - WIN_VOX)..(cz + WIN_VOX) {
        for x in (cx - WIN_VOX)..(cx + WIN_VOX) {
            let d2 = (src.uniform(x - 1, z, tag) - 2.0 * src.uniform(x, z, tag)
                + src.uniform(x + 1, z, tag))
            .abs();
            if (x - 1).div_euclid(32) != (x + 1).div_euclid(32) {
                on += d2;
                on_n += 1;
            } else {
                off += d2;
                off_n += 1;
            }
        }
    }
    (on / on_n as f64, off / off_n as f64)
}

/// The member field of one recorded event over the window, under one source.
fn member_field(
    set: &dc_core::materials::geology::GeologySet,
    event: &StrataEvent,
    src: &dyn DitherSource,
    cx: i64,
    cz: i64,
) -> Vec<u16> {
    let n = (WIN_VOX * 2) as usize;
    let mut out = Vec::with_capacity(n * n);
    for z in 0..n as i64 {
        for x in 0..n as i64 {
            let m = dithered_member_with(set, event, src, cx - WIN_VOX + x, cz - WIN_VOX + z);
            out.push(m.0 as u16);
        }
    }
    out
}

/// `P(m(p) == m(p + lag·x̂))` over the window — the member field's spatial
/// autocorrelation. A field with **one** characteristic length saturates at that
/// length and is flat past it; the lag where it stops falling *is* the patch size
/// the eye reads.
fn agreement_at_lag(field: &[u16], lag: i64) -> f64 {
    let n = (WIN_VOX * 2) as usize;
    let (mut same, mut total) = (0u64, 0u64);
    for z in 0..n {
        for x in 0..(n - lag as usize) {
            total += 1;
            if field[z * n + x] == field[z * n + x + lag as usize] {
                same += 1;
            }
        }
    }
    same as f64 / total as f64
}

/// Fraction of aligned 32×32 chunk footprints in the window that hold **exactly
/// one** member — "a chunk of ground is one rock", the literal shape of the U3
/// complaint. Note this is not monotone in quality on its own: a field with
/// coarse power also produces uniform chunks, honestly, because the ground there
/// *is* uniform over more than a chunk. Read it with the autocorrelation, which
/// says whether the uniformity has a 28.8 m *scale*.
fn single_member_chunk_fraction(field: &[u16]) -> f64 {
    let n = (WIN_VOX * 2) as usize;
    let chunks = n / 32;
    let (mut uniform, mut total) = (0u64, 0u64);
    for cz in 0..chunks {
        for cx in 0..chunks {
            total += 1;
            let first = field[cz * 32 * n + cx * 32];
            let mut all_same = true;
            for z in 0..32 {
                for x in 0..32 {
                    if field[(cz * 32 + z) * n + cx * 32 + x] != first {
                        all_same = false;
                    }
                }
            }
            if all_same {
                uniform += 1;
            }
        }
    }
    uniform as f64 / total as f64
}

/// Distinct members and the largest member's share — the unbiasedness the octaves
/// source adds, read at the site rather than in a fixture.
fn member_mix(field: &[u16]) -> (usize, f64) {
    let mut counts: Vec<(u16, usize)> = Vec::new();
    for &m in field {
        match counts.iter_mut().find(|(k, _)| *k == m) {
            Some((_, n)) => *n += 1,
            None => counts.push((m, 1)),
        }
    }
    let top = counts.iter().map(|(_, n)| *n).max().unwrap_or(0);
    (counts.len(), top as f64 / field.len() as f64)
}

/// Print the whole U3 before/after comparison. `main` calls it after the station
/// search; it is not in the gate (it builds the production world), but the
/// *metric* is — see [`gate::the_stepping_metric_separates_one_octave_from_many`].
fn report_u3_member_stepping(g: &mut WorldGenerator<'_>) {
    let (fx, _fy, fz) = U3_FEET_M;
    let (vx, vz) = ((fx / VOXEL_M).round() as i64, (fz / VOXEL_M).round() as i64);
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    println!("\n=== U3 member stepping at the reference pose (journal/0129) ===");
    println!(
        "feet (world m) {:.1}, {:.1}, {:.1}  →  voxel ({vx}, {vz}), chunk ({cx}, {cz})",
        U3_FEET_M.0, U3_FEET_M.1, U3_FEET_M.2
    );
    let col = g.column_record(cx, cz);
    let set = dc_core::materials::geology::vanilla();

    // ── The ceiling on what a WITHIN-CLASS member dither can express at all ──
    //
    // This census is the first thing to read, and nobody had ever printed it. The
    // member dither re-picks *within* an event's content class, so its expressive
    // range is bounded by that class's member count — a single-member class is a
    // constant field no matter what source drives it.
    println!("\n-- vanilla member census (the dither's ceiling) --");
    let mut multi = 0usize;
    let mut total_classes = 0usize;
    for class in dc_core::materials::geology::v1_classes() {
        let n = set.class(class).map_or(0, |c| c.members().len());
        total_classes += 1;
        if n > 1 {
            multi += 1;
        }
        println!("   {n} member(s)  {class}");
    }
    println!(
        "   → {multi} of {total_classes} classes can express ANY within-class variation; \
         the rest are constant fields under any dither."
    );

    // ── What the SURFACE at this chunk is actually made of, and whether the
    //    member dither can move it ──
    let fill = &col.strata;
    let mut single_movable = 0usize;
    let mut single_frozen = 0usize;
    let mut mixed = 0usize;
    {
        use dc_worldgen::{ColumnFill, Plan};
        let cf = ColumnFill::build(fill, VOXEL_M);
        match cf.plan(1) {
            None => println!("\nthe surface voxel has no record top span here (fallback column)"),
            Some(Plan::Mixed(_)) => mixed = 1024,
            Some(Plan::Single(k)) => {
                let e = fill.events[*k];
                let c = set.member(e.member).class.as_str();
                if set.class(c).map_or(0, |cl| cl.members().len()) > 1 {
                    single_movable = 1024;
                } else {
                    single_frozen = 1024;
                }
            }
        }
    }
    println!(
        "\nsurface top span at chunk ({cx},{cz}): mixed {mixed} / single-movable \
         {single_movable} / single-frozen {single_frozen} columns (of 1024)"
    );
    if mixed == 1024 {
        println!(
            "   ⚠ MIXED top span: `mixed_at` uses the UNDITHERED `event.member` by design, so \
             the member dither does not touch this chunk's surface material AT ALL — the \
             per-voxel eighth allocation is what varies it."
        );
    }

    // ── Pick an event whose class can actually express, so the A/B has content ──
    let Some(event) = col
        .strata
        .events
        .iter()
        .rev()
        .find(|e| {
            let c = set.member(e.member).class.as_str();
            set.class(c).map_or(0, |cl| cl.members().len()) > 1
        })
        .copied()
    else {
        println!(
            "\nNULL AT THE REFERENCE POSE: not one of this chunk-column's {} recorded events \
             belongs to a multi-member class, so the within-class member dither is INERT here \
             under any source. That is a result about U3, not a failed measurement — see \
             journal/0129.",
            col.strata.events.len()
        );
        return;
    };
    let class = set.member(event.member).class.as_str();
    let n_members = set.class(class).map_or(0, |c| c.members().len());
    println!(
        "\nmeasuring on the topmost event whose class can express: member {} \
         (class {class}, {n_members} members), salt {:#x}, tag {}",
        set.member(event.member).id,
        event.sel_salt,
        event.sel_tag
    );
    let before = Coherent::new(Draws::from_recorded_salt(SEED, event.sel_salt), 32);
    let after = Octaves::member(Draws::from_recorded_salt(SEED, event.sel_salt));

    let (b_on, b_off) = lattice_curvature(&before, vx, vz, event.sel_tag);
    let (a_on, a_off) = lattice_curvature(&after, vx, vz, event.sel_tag);
    println!(
        "\n-- the square-edge signature: |Δ²u| on the 28.8 m lattice vs off it --\n\
         \x20 BEFORE (single octave, stride 32): on {b_on:.3e}  off {b_off:.3e}  ratio {:.3e}\n\
         \x20 AFTER  (octaves, prime ladder):    on {a_on:.3e}  off {a_off:.3e}  ratio {:.3}",
        b_on / b_off.max(f64::MIN_POSITIVE),
        a_on / a_off
    );

    let fb = member_field(&set, &event, &before, vx, vz);
    let fa = member_field(&set, &event, &after, vx, vz);
    println!("\n-- member-field autocorrelation P(m(p) == m(p+lag)) --");
    println!("  lag(vox)   metres    BEFORE    AFTER");
    for lag in LAGS {
        println!(
            "  {lag:>7}   {:>6.1}    {:.4}    {:.4}",
            lag as f64 * VOXEL_M,
            agreement_at_lag(&fb, lag),
            agreement_at_lag(&fa, lag)
        );
    }
    let (b_dist, b_top) = member_mix(&fb);
    let (a_dist, a_top) = member_mix(&fa);
    println!(
        "\n-- what the window actually shows --\n\
         \x20 BEFORE: {b_dist} distinct members, largest share {:.3}, single-member chunks {:.3}\n\
         \x20 AFTER : {a_dist} distinct members, largest share {:.3}, single-member chunks {:.3}",
        b_top,
        single_member_chunk_fraction(&fb),
        a_top,
        single_member_chunk_fraction(&fa)
    );
    // ── The OTHER 28.8 m mechanism, measured because the census above says the
    //    member dither cannot be the whole story at this pose ──
    //
    // A chunk's whole `StrataRec` is produced by ONE `run_strata` per chunk, from
    // context sampled at the chunk CENTRE: climate, mean elevation, flow energy,
    // provenance. So the *record itself* — which classes surface and in what
    // proportion — is chunk-quantized, independently of any dither and independently
    // of the 460 m deep-cell grid. A class change is a large tint change (tan vs
    // grey vs dark); a within-class member change is a small one. This counts how
    // often adjacent chunks disagree about their surface class mix.
    println!("\n-- the record itself is per-CHUNK: do neighbours disagree? --");
    let mut sigs: Vec<(i64, i64, String)> = Vec::new();
    for dz in 0..8i64 {
        for dx in 0..8i64 {
            let c = g.column_record(cx + dx, cz + dz);
            let mut top: Vec<(&str, f64)> = Vec::new();
            let mut acc = 0.0f64;
            for e in c.strata.events.iter().rev() {
                if acc >= VOXEL_M {
                    break;
                }
                let take = f64::from(e.thickness_m).min(VOXEL_M - acc);
                if take <= 0.0 {
                    continue;
                }
                acc += take;
                let cl = set.member(e.member).class.as_str();
                match top.iter_mut().find(|(k, _)| *k == cl) {
                    Some((_, m)) => *m += take,
                    None => top.push((cl, take)),
                }
            }
            top.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
            let sig = top
                .iter()
                .map(|(k, m)| format!("{}:{:.2}", glyph(k), m))
                .collect::<Vec<_>>()
                .join(",");
            sigs.push((cx + dx, cz + dz, sig));
        }
    }
    let mut differ = 0usize;
    let mut pairs = 0usize;
    let mut dominant_differ = 0usize;
    for i in 0..8usize {
        for j in 0..7usize {
            let a = &sigs[i * 8 + j];
            let b = &sigs[i * 8 + j + 1];
            pairs += 1;
            if a.2 != b.2 {
                differ += 1;
            }
            if a.2.chars().next() != b.2.chars().next() {
                dominant_differ += 1;
            }
        }
    }
    println!(
        "   over an 8×8 block of chunks (230 m), {differ}/{pairs} horizontally adjacent \
         chunk pairs have a DIFFERENT surface-class mix, and {dominant_differ}/{pairs} \
         disagree on the DOMINANT class."
    );
    println!("   first row of signatures (glyph:metres in the top 0.9 m):");
    for s in sigs.iter().take(8) {
        println!("     chunk ({},{})  {}", s.0, s.1, s.2);
    }
    println!(
        "   ⚠ Whatever these numbers are, they are NOT moved by this slice and NOT moved by \
         the 460 m record-membership restructure either: the fix for a chunk-quantized \
         RECORD is per-column formation context, a third mechanism at this site."
    );

    println!(
        "\nRead it this way: the BEFORE ratio is the mechanism (a bilinear field has NO \n\
         curvature except on its own grid, so the number is a float-residue division), and \n\
         the AFTER ratio near 1 says the 28.8 m lattice is no longer special. The \n\
         autocorrelation says whether the field still has ONE characteristic length: if \n\
         BEFORE goes flat by lag 32-64 and AFTER keeps falling, the patch size has stopped \n\
         being the chunk. **Neither number is a verdict on appearance — the user's eye is \n\
         (the walk this feeds).**"
    );
}

/// **The gate's view of this instrument** (journal/0103).
///
/// This tour is a **station finder**: its output is a place to stand and a camera
/// to point, and the only judge of whether it found the right patch is the eye
/// that looks at the frame. There is no world invariant it can assert, and the
/// world-scale search takes tens of seconds — so **the search itself is
/// deliberately NOT in the gate**, and that is recorded here rather than left as
/// an omission (stubs.md doctrine: an unlisted loose end is the defect).
///
/// What *is* gated is the part that can be silently wrong without anyone
/// noticing: the **ranking function**. `window_score` decides which patch the
/// walk is sent to, and corrections #48 is the standing reminder of what a
/// mis-aimed reference costs — a 39 km error produced four null frames and a
/// confident wrong conclusion. These tests are fixtures over pure functions and
/// build no world at all, so they cost the gate nothing measurable.
#[cfg(test)]
mod gate {
    use super::*;

    const SAND: &str = "dc:stratum/clastic-coarse";
    const MUD: &str = "dc:stratum/clastic-fine";
    const COAL: &str = "dc:stratum/organic-coal";
    const GRAN: &str = "dc:stratum/igneous-intrusive";

    /// The whole point of the score: a patch of several *distinct* classes must
    /// outrank a uniform one. If this inverts, the tour sends the walk to the
    /// blandest ground it can find and still prints a confident pose.
    #[test]
    fn a_multi_class_window_outranks_a_uniform_one() {
        let uniform = window_score(&[Some(GRAN); 9]);
        let mixed = window_score(&[
            Some(SAND),
            Some(MUD),
            Some(COAL),
            Some(GRAN),
            Some(SAND),
            Some(MUD),
            Some(COAL),
            Some(GRAN),
            Some(SAND),
        ]);
        assert_eq!(uniform.n_distinct, 1);
        assert_eq!(mixed.n_distinct, 4);
        assert!(
            mixed.score > uniform.score,
            "a 4-class window scored {:.3} against a uniform window's {:.3} — the \
             checkerboard finder now prefers uniform ground",
            mixed.score,
            uniform.score
        );
    }

    /// Coverage weighting is what stops a window that is mostly ocean or wilds
    /// from winning on two loud cells. Same classes, fewer present cells, lower
    /// score — strictly.
    #[test]
    fn a_mostly_empty_window_cannot_win_on_a_couple_of_loud_cells() {
        let full = window_score(&[
            Some(SAND),
            Some(MUD),
            Some(COAL),
            Some(SAND),
            Some(MUD),
            Some(COAL),
            Some(SAND),
            Some(MUD),
            Some(COAL),
        ]);
        let sparse = window_score(&[
            Some(SAND),
            Some(MUD),
            Some(COAL),
            None,
            None,
            None,
            None,
            None,
            None,
        ]);
        assert_eq!(full.n_distinct, sparse.n_distinct);
        assert!(sparse.coverage < full.coverage);
        assert!(
            sparse.score < full.score,
            "a 1/3-covered window scored {:.3} against a fully covered one's {:.3}",
            sparse.score,
            full.score
        );
    }

    /// Balance breaks ties: eight sandstone cells and one mudstone is not the
    /// reference frame's checkerboard, even though it holds two classes.
    #[test]
    fn balance_breaks_the_tie_between_two_class_windows() {
        let lopsided = window_score(&[
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(SAND),
            Some(MUD),
        ]);
        let even = window_score(&[
            Some(SAND),
            Some(MUD),
            Some(SAND),
            Some(MUD),
            Some(SAND),
            Some(MUD),
            Some(SAND),
            Some(MUD),
            Some(SAND),
        ]);
        assert_eq!(lopsided.n_distinct, even.n_distinct);
        assert!(
            even.hnorm > lopsided.hnorm && even.score > lopsided.score,
            "balanced {:.3} (H {:.3}) did not outrank lopsided {:.3} (H {:.3})",
            even.score,
            even.hnorm,
            lopsided.score,
            lopsided.hnorm
        );
    }

    /// **The stepping metric must separate one octave from many** (journal/0129).
    ///
    /// [`lattice_curvature`] is what the U3 report's headline number comes from, so
    /// if it stops discriminating, the report keeps printing confident numbers about
    /// nothing. Two synthetic sources, no world:
    ///
    /// - a **stride-32 single octave** must show essentially all its curvature on
    ///   the 32-voxel lattice (the ratio is a float-residue division), because a
    ///   bilinear field is linear inside its cell;
    /// - the **octaves ladder** must show no preference for that lattice, because
    ///   no rung is 32 or a divisor of it.
    ///
    /// **Scale-free:** a second difference over three adjacent voxels is a local
    /// arithmetic property of a pure function of position. No world, no extent, and
    /// no amount of surrounding terrain enters it — which is also why this costs the
    /// gate nothing measurable while the world-scale report above stays out of it.
    #[test]
    fn the_stepping_metric_separates_one_octave_from_many() {
        let d = Draws::from_recorded_salt(1337, 0x5700_000E);
        let (b_on, b_off) = lattice_curvature(&Coherent::new(d, 32), 0, 0, 3);
        assert!(
            b_off < 1e-15 && b_on / b_off > 1e9,
            "a stride-32 field must concentrate its curvature on the 32-lattice \
             (on {b_on:.3e}, off {b_off:.3e}) — that concentration IS the signature this \
             probe reports"
        );
        let (a_on, a_off) = lattice_curvature(&Octaves::member(d), 0, 0, 3);
        let ratio = a_on / a_off;
        assert!(
            (0.5..2.0).contains(&ratio),
            "the octaves ladder still prefers the 32-lattice (on {a_on:.3e}, off {a_off:.3e}, \
             ratio {ratio:.2}) — a stride became commensurate with 32"
        );
    }

    /// A glyph is what a human reads the window layout from; an unmapped class
    /// prints `?` and silently erases the very distinction the tour exists to
    /// show. Every class the finder can actually produce must have one.
    #[test]
    fn every_reachable_class_has_a_glyph() {
        for c in [
            SAND,
            MUD,
            COAL,
            GRAN,
            "dc:stratum/igneous-extrusive",
            "dc:stratum/organic-peat",
            "dc:stratum/organic-soil",
            "dc:stratum/organic-charcoal",
            "dc:stratum/ore-placer",
            "dc:stratum/accessory-mafic",
        ] {
            assert_ne!(glyph(c), '?', "class {c} has no glyph and prints as '?'");
        }
    }
}
