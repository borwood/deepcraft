//! **Is the thin record a hole?** (session 7 diagnostic)
//!
//! Standing at world metres (106 938, 9 953) — 370 m from journal/0049's dune
//! field station — a walker found ONE voxel of surface over contents-free
//! fallback stone, while a neighbouring column 22 m east carried a real record
//! voxel. The user's read: *"this cell has no history, all surrounding ones do,
//! this is a bug."*
//!
//! Blocks cannot answer that. `Block::Stone` is emitted both for *unrecorded
//! basement* and for any material without a block twin, and the block tier
//! collapses every rock to a handful of names. So this probe reads the
//! **record** directly: for a window of deep cells around a point it prints
//! unit count, `Σ thickness`, carried `H`, and surface elevation, as a map —
//! a cell-shaped hole in a smooth field is visible instantly, and a smooth
//! gradient is the null.
//!
//! Nothing here is on a generation path: every readout is a pure derivation of
//! the same pregen the world boots from.
//!
//! `cargo run --release -p dc-worldgen --example record_hole_probe`

use dc_worldgen::deeptime::{DeepField, DeepStrata};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

/// The client's `BENCH_SEED` — the world every walk so far has stood in.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;

/// Where the walker was standing, in world metres.
const SITES: [(&str, f64, f64); 3] = [
    ("walker (thin record)", 106_938.0, 9_953.0),
    ("dune field station", 107_183.0, 9_672.0),
    ("loess margin station", 82_346.0, 24_391.0),
];

/// Half-width of the printed cell window.
const R: i64 = 6;

fn main() {
    println!("=== record-hole probe (is the thin record a cell-shaped hole?) ===");
    println!("seed {SEED}, extent {}\n", EXTENT.label());

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let f = &pregen.deep;
    println!(
        "deep grid {}x{} @ {:.0} m/cell (pregen {}x{})\n",
        f.w, f.w, f.cell_m, f.wp, f.wp
    );

    for (name, mx, mz) in SITES {
        let (vx, vz) = ((mx / VOXEL_M).round() as i64, (mz / VOXEL_M).round() as i64);
        let Some((cx, cy)) = cell_of_voxel(f.w, f.wp, vx, vz) else {
            println!("{name}: OUTSIDE the pregen extent (the wilds) — no record by design\n");
            continue;
        };
        println!(
            "--- {name}: metres ({mx:.0}, {mz:.0}) = voxel ({vx}, {vz}) = deep cell ({cx}, {cy})"
        );
        let i = cy as usize * f.w + cx as usize;
        println!(
            "    surf {:.1} m | H {:.2} m | units {} | Σrecord {:.2} m",
            f.surf[i],
            f.regolith[i],
            f.strata[i].units.len(),
            total(&f.strata[i]),
        );

        // Maps. Σrecord and H over the window: a hole is a zero in a field of
        // non-zeros; a gradient is the null result.
        map(f, cx, cy, "Σrecord (m)", |s, _| total(s));
        map(f, cx, cy, "units", |s, _| s.units.len() as f64);
        map(f, cx, cy, "H (m)", |_, h| h);
        surf_map(f, cx, cy);

        // The centre cell's units, top-down — the actual history.
        let s = &f.strata[i];
        println!(
            "    centre cell record, top-down ({} units):",
            s.units.len()
        );
        for u in s.units.iter().rev().take(12) {
            println!(
                "      {:>8.3} m  {:?}  chapter {}{}",
                u.thickness_m,
                u.tag,
                u.chapter,
                if u.unconformity { "  UNCONFORMITY" } else { "" }
            );
        }
        if s.units.len() > 12 {
            println!("      … {} more below", s.units.len() - 12);
        }
        println!();
    }

    anisotropy(f, "full_agents ON (production)");

    // **The A/B.** `wind` marches each deep-grid ROW as an independent 1-D
    // transport lane (`erosion.rs::wind`: `load` is declared inside the `gy`
    // loop and never crosses rows). If that is what bands `H` north-south, the
    // banding must vanish with the agent roster off.
    println!("\n=== A/B: same seed, full_agents OFF ===");
    let off = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &dc_worldgen::DeepOverrides {
            full_agents: Some(false),
            ..Default::default()
        },
    );
    anisotropy(&off.deep, "full_agents OFF");
}

/// **Directional roughness of the regolith plane.** Wind transports along `x`
/// within a row and never across rows, so the prediction is: neighbouring cells
/// *along* a row (`|ΔH|` in x) stay similar, while neighbouring cells *across*
/// rows (`|ΔH|` in y) jump. An isotropic field scores ~1.0; the ratio is the
/// statistic, and the null is "no direction is special".
fn anisotropy(f: &DeepField, label: &str) {
    let (mut dx_sum, mut dx_n, mut dy_sum, mut dy_n) = (0.0f64, 0usize, 0.0f64, 0usize);
    let mut worst = (0.0f64, 0usize, 0usize);
    for y in 0..f.w {
        for x in 0..f.w {
            let i = y * f.w + x;
            if f.surf[i] <= 0.0 {
                continue; // land only — the sea has no regolith story
            }
            if x + 1 < f.w && f.surf[i + 1] > 0.0 {
                dx_sum += (f.regolith[i] - f.regolith[i + 1]).abs();
                dx_n += 1;
            }
            if y + 1 < f.w && f.surf[i + f.w] > 0.0 {
                let d = (f.regolith[i] - f.regolith[i + f.w]).abs();
                dy_sum += d;
                dy_n += 1;
                if d > worst.0 {
                    worst = (d, x, y);
                }
            }
        }
    }
    let (mx, my) = (dx_sum / dx_n as f64, dy_sum / dy_n as f64);
    println!("--- H directional roughness, {label}");
    println!("    mean |ΔH| ALONG a row (x, the wind's axis) : {mx:.3} m  ({dx_n} pairs)");
    println!("    mean |ΔH| ACROSS rows (y, no transport)    : {my:.3} m  ({dy_n} pairs)");
    println!(
        "    anisotropy y/x = {:.2}×   (1.00 = isotropic, the null)",
        my / mx
    );
    println!(
        "    worst adjacent-row step: {:.2} m at cell ({}, {})",
        worst.0, worst.1, worst.2
    );

    // How often does a walker cross a soil-depth CLIFF? `H` is sampled
    // NEAREST-cell (`DeepField::regolith_at_voxel` rounds), while the surface
    // elevation beside it is BILINEAR (`surface_at_voxel`) — so every deep-cell
    // boundary is a hard 460 m step in soil depth with no gradient across it.
    let vox = |h: f64| (h / VOXEL_M).round().clamp(0.0, 8.0) as i32;
    let (mut land, mut bare, mut cliffs, mut pairs) = (0usize, 0usize, 0usize, 0usize);
    let mut hist = [0usize; 9];
    for y in 0..f.w {
        for x in 0..f.w {
            let i = y * f.w + x;
            if f.surf[i] <= 0.0 {
                continue;
            }
            land += 1;
            let v = vox(f.regolith[i]);
            hist[v as usize] += 1;
            if v == 0 {
                bare += 1;
            }
            for j in [
                (x + 1 < f.w).then_some(i + 1),
                (y + 1 < f.w).then_some(i + f.w),
            ]
            .into_iter()
            .flatten()
            {
                if f.surf[j] <= 0.0 {
                    continue;
                }
                pairs += 1;
                let w = vox(f.regolith[j]);
                if (v == 0 && w >= 4) || (w == 0 && v >= 4) {
                    cliffs += 1;
                }
            }
        }
    }
    println!("    soil voxels expressed, histogram 0..8: {hist:?}  ({land} land cells)",);
    println!(
        "    BARE (0 voxels): {bare} = {:.1}% of land",
        100.0 * bare as f64 / land as f64
    );
    println!(
        "    adjacent-cell CLIFFS (one side 0 voxels, other ≥4): {cliffs} of {pairs} pairs = {:.2}%",
        100.0 * cliffs as f64 / pairs as f64
    );
}

fn total(s: &DeepStrata) -> f64 {
    s.units.iter().map(|u| u.thickness_m).sum()
}

/// Deep-cell index of a world voxel — the inverse `DeepField::deep_coords`
/// performs, replicated here because it is private (round-tripped against
/// `record_at_voxel` by construction: both round the same continuous coord).
fn cell_of_voxel(w: usize, wp: usize, vx: i64, vz: i64) -> Option<(i64, i64)> {
    let half = (wp / 2) as f64;
    let px = vx as f64 / CELL_VOXELS as f64 + half - 0.5;
    let py = vz as f64 / CELL_VOXELS as f64 + half - 0.5;
    let wpf = wp as f64;
    if px < 0.0 || px > wpf - 1.0 || py < 0.0 || py > wpf - 1.0 {
        return None;
    }
    let gx = (px + 0.5) / wpf * w as f64 - 0.5;
    let gy = (py + 0.5) / wpf * w as f64 - 0.5;
    Some((
        (gx.round() as i64).clamp(0, w as i64 - 1),
        (gy.round() as i64).clamp(0, w as i64 - 1),
    ))
}

fn map(f: &DeepField, cx: i64, cy: i64, label: &str, val: impl Fn(&DeepStrata, f64) -> f64) {
    println!("    {label} over ±{R} cells (centre marked *):");
    for dy in -R..=R {
        let mut row = String::from("      ");
        for dx in -R..=R {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= f.w as i64 || y >= f.w as i64 {
                row.push_str("   ---");
                continue;
            }
            let i = y as usize * f.w + x as usize;
            let v = val(&f.strata[i], f.regolith[i]);
            let mark = if dx == 0 && dy == 0 { '*' } else { ' ' };
            row.push_str(&format!("{mark}{v:>5.1}"));
        }
        println!("{row}");
    }
}

fn surf_map(f: &DeepField, cx: i64, cy: i64) {
    println!("    surf (m) over ±{R} cells (negative = below sea level):");
    for dy in -R..=R {
        let mut row = String::from("      ");
        for dx in -R..=R {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= f.w as i64 || y >= f.w as i64 {
                row.push_str("     ---");
                continue;
            }
            let i = y as usize * f.w + x as usize;
            let mark = if dx == 0 && dy == 0 { '*' } else { ' ' };
            row.push_str(&format!("{mark}{:>7.0}", f.surf[i]));
        }
        println!("{row}");
    }
}
