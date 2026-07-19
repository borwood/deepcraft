//! Dev tool: print river cells and channel midpoints for a seed, in world
//! voxels and meters — so a walk (MCP session against the running client)
//! can find fluvial features without scanning a 251 km world blind.
//!
//! Usage: cargo run --release -p dc-worldgen --example river_cells [seed]
//! (seed defaults to the client bench seed, 1337.)

use dc_worldgen::{Extent, Pregen, WorldParams};

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337);
    let pregen = Pregen::run(WorldParams {
        seed,
        extent: Extent::Medium,
    });
    println!("seed {seed}, medium: river cells (land, with downstream flow):");
    let mut n = 0;
    for (i, c) in pregen.grid.cells.iter().enumerate() {
        if !(c.river && c.is_land()) {
            continue;
        }
        let Some(to) = c.flow_to else { continue };
        let (agx, agy) = pregen.grid.coords(i);
        let (bgx, bgy) = pregen.grid.coords(to as usize);
        let a = pregen.grid.cell_center_voxel(agx, agy);
        let b = pregen.grid.cell_center_voxel(bgx, bgy);
        let mid = ((a.0 + b.0) / 2, (a.1 + b.1) / 2);
        // Meters at N=2 (0.9 m voxels) for direct use with pose_set.
        println!(
            "  cell ({agx:2},{agy:2}) -> ({bgx:2},{bgy:2})  mid voxel ({:7}, {:7})  mid meters ({:9.1}, {:9.1})",
            mid.0,
            mid.1,
            mid.0 as f64 * 0.9,
            mid.1 as f64 * 0.9,
        );
        n += 1;
    }
    println!("{n} river segments.");
}
