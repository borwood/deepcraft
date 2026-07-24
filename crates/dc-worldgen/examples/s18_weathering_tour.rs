//! Guided-tour map for the weathering walk (journal/0089 S18; **journal/0094 made it
//! a per-epoch accumulating process**).
//!
//! Builds the world the dc-client boots (`BENCH_SEED` = 1337, `Extent::Medium`) with
//! `weather_inventory` ON (the `--weather-inventory` launch flag) — so the
//! `dc:deep/weather_inventory` pass runs **inside the deep-time loop, every epoch,
//! accumulating** — then finds the deep cells whose committed `Structure→Loose`
//! weathering facts produced the thickest basal **saprolite band**
//! (`FactLedger::weathering_product_m`), now ≥1 voxel. For each station it prints
//! the world coordinate a player teleports to, the surface elevation, the band thickness,
//! and the regolith cover `H` there (thin cover = high `cover_taper` = why the band is
//! strong). Nothing here is on a generation path.
//!
//! `cargo run --release -p dc-worldgen --example s18_weathering_tour`

use dc_worldgen::deeptime::{DeepOverrides, build_field_with};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;
/// journal/0046's scarp — ground the user already knows; reported as a bonus station.
const SCARP_XZ_M: (f64, f64) = (-16545.0, 10592.0);
/// Border ring excluded from station picks (march edge artifacts, per tour_map.rs).
const EDGE_MARGIN: i64 = 4;

struct Conv {
    w: usize,
    wp: usize,
}
impl Conv {
    /// World voxel centre of deep cell `idx` (inverse of `DeepField::deep_coords`).
    fn idx_to_voxel(&self, idx: usize) -> (i64, i64) {
        let (w, wp) = (self.w as f64, self.wp as f64);
        let (gx, gy) = ((idx % self.w) as f64, (idx / self.w) as f64);
        let px = (gx + 0.5) / w * wp - 0.5;
        let py = (gy + 0.5) / w * wp - 0.5;
        let half = (self.wp / 2) as f64;
        (
            ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
            ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        )
    }
    fn line(&self, idx: usize) -> String {
        let (gx, gy) = (idx % self.w, idx / self.w);
        let (vx, vz) = self.idx_to_voxel(idx);
        format!(
            "deep cell ({gx},{gy}) | voxel ({vx}, {vz}) | world ({:.0} m, {:.0} m)",
            vx as f64 * VOXEL_M,
            vz as f64 * VOXEL_M
        )
    }
}

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

fn main() {
    println!("=== S18 weathering-band tour map (journal/0089) ===");
    println!(
        "seed {SEED}, extent {}, weather_inventory ON, N=2 ({VOXEL_M} m voxels)\n",
        EXTENT.label()
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let field = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            weather_inventory: Some(true),
            ..DeepOverrides::default()
        },
    );
    let (w, wp) = (field.w, field.wp);
    let conv = Conv { w, wp };

    // Per-cell band thickness (metres) = the collapse's basal saprolite band.
    let band = |i: usize| -> f64 {
        field
            .ledgers
            .get(i)
            .map_or(0.0, |l| l.weathering_product_m(field.strata[i].units.len()))
    };

    // Distribution over banded interior cells.
    let mut banded: Vec<(usize, f64)> = (0..w * w)
        .filter(|&i| interior(w, i) && band(i) > 1e-6)
        .map(|i| (i, band(i)))
        .collect();
    banded.sort_by(|a, b| b.1.total_cmp(&a.1));
    let total_cells = w * w;
    let n_band = banded.len();
    let max = banded.first().map_or(0.0, |x| x.1);
    let mean = if n_band > 0 {
        banded.iter().map(|x| x.1).sum::<f64>() / n_band as f64
    } else {
        0.0
    };
    println!("--- band statistics (interior) ---");
    println!(
        "  deep grid {w}x{w} @ {:.1} m/cell; pregen {wp}x{wp}",
        field.cell_m
    );
    println!(
        "  banded cells: {n_band} / {total_cells}  ({:.1}%)",
        100.0 * n_band as f64 / total_cells as f64
    );
    let max_vox = max / VOXEL_M;
    let max_eighths = (max_vox * 8.0).round() as i64;
    println!("  band thickness: max {max:.2} m, mean {mean:.2} m over banded cells");
    println!(
        "  max band in voxels: {max_vox:.2} voxels ({max_eighths} eighths) @ {VOXEL_M} m/voxel  \
         [>= 1 voxel: {}]\n",
        if max >= VOXEL_M { "YES" } else { "NO" }
    );

    if n_band == 0 {
        println!("NULL: no weathering band anywhere — the flag produced nothing to walk.");
        return;
    }

    // Top stations: strongest bands, spatially de-duplicated (>= 6 deep cells apart)
    // so we do not teleport to five adjacent cells of the same hill.
    println!(
        "--- STATIONS (teleport here, --fullbright, then dig down to the basement contact) ---"
    );
    let mut picked: Vec<usize> = Vec::new();
    for &(i, thick) in &banded {
        if picked.len() >= 5 {
            break;
        }
        let (gx, gy) = ((i % w) as i64, (i / w) as i64);
        let far = picked.iter().all(|&j| {
            let (jx, jy) = ((j % w) as i64, (j / w) as i64);
            (gx - jx).abs().max((gy - jy).abs()) >= 6
        });
        if !far {
            continue;
        }
        picked.push(i);
        println!(
            "\nSTATION {} — band {:.2} m ({:.2} voxels)",
            picked.len(),
            thick,
            thick / VOXEL_M
        );
        println!("  {}", conv.line(i));
        println!("  surface elevation : {:.1} m", field.surf[i]);
        println!(
            "  regolith cover H  : {:.2} m  (thin ⇒ high cover_taper ⇒ strong band)",
            field.regolith.get(i).copied().unwrap_or(0.0)
        );
    }

    // Bonus: the band nearest the known 0046 scarp (ground the user recognises).
    let scarp_i = (0..w * w)
        .filter(|&i| interior(w, i))
        .min_by(|&a, &b| {
            let d = |i: usize| {
                let (vx, vz) = conv.idx_to_voxel(i);
                (vx as f64 * VOXEL_M - SCARP_XZ_M.0).hypot(vz as f64 * VOXEL_M - SCARP_XZ_M.1)
            };
            d(a).total_cmp(&d(b))
        })
        .unwrap();
    println!("\nBONUS — nearest the journal/0046 scarp (ground you know):");
    println!("  {}", conv.line(scarp_i));
    println!("  surface elevation : {:.1} m", field.surf[scarp_i]);
    println!("  band thickness    : {:.2} m", band(scarp_i));
}
