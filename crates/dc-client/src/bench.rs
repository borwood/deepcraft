//! `--bench-scales`: headless measurement pass for S1.
//!
//! Generates and meshes the same world region **in meters** at player-height =
//! 2, 3, and 4 voxels (same seed) and reports the numbers the spike exists to
//! collect. No window, no GPU — meshing produces CPU-side vertex arrays only.

use std::collections::HashMap;
use std::time::Instant;

use dc_core::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, ChunkPos, VoxelScale, local_voxel};

use crate::PLAYER_HEIGHT_M;
use crate::meshing::mesh_chunk;
use crate::worldgen::TerrainGen;

/// Fixed world seed shared by the bench and (by default) the interactive app.
pub const BENCH_SEED: i32 = 1337;

/// Measured region in meters: x/z in [-128, 128), y in [-96, 32).
/// 256 m x 128 m x 256 m, placed to include the surface (~-6..+22 m) and the
/// chasm floor (~-80 m) so the numbers reflect real terrain, not empty sky.
pub const REGION_MIN_M: [f64; 3] = [-128.0, -96.0, -128.0];
pub const REGION_MAX_M: [f64; 3] = [128.0, 32.0, 128.0];

struct Row {
    player_voxels: u32,
    voxel_size_m: f64,
    chunk_count: usize,
    voxel_count: u64,
    raw_bytes: u64,
    gen_time_s: f64,
    mesh_time_s: f64,
    triangles: u64,
}

pub fn run() {
    println!(
        "S1 scale bench: region {:.0}x{:.0}x{:.0} m, seed {BENCH_SEED}, chunk edge {CHUNK_SIZE} voxels",
        REGION_MAX_M[0] - REGION_MIN_M[0],
        REGION_MAX_M[1] - REGION_MIN_M[1],
        REGION_MAX_M[2] - REGION_MIN_M[2],
    );
    println!();

    let generator = TerrainGen::new(BENCH_SEED);
    let rows: Vec<Row> = [2u32, 3, 4]
        .into_iter()
        .map(|n| bench_scale(&generator, n))
        .collect();

    println!(
        "| Player height | Voxel size | Chunks | Voxels | Raw chunk memory | Gen time | Mesh time | Triangles |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for r in &rows {
        println!(
            "| {} voxels | {:.3} m | {} | {} | {} ({} B) | {:.2} s | {:.2} s | {} |",
            r.player_voxels,
            r.voxel_size_m,
            group_thousands(r.chunk_count as u64),
            group_thousands(r.voxel_count),
            format_bytes(r.raw_bytes),
            group_thousands(r.raw_bytes),
            r.gen_time_s,
            r.mesh_time_s,
            group_thousands(r.triangles),
        );
    }
}

fn bench_scale(generator: &TerrainGen, player_voxels: u32) -> Row {
    let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, player_voxels);
    let chunk_m = scale.voxels_to_meters(f64::from(CHUNK_SIZE));

    // All chunks intersecting the region. Chunk grids don't align with meter
    // boundaries, so coverage overshoots slightly (differently per scale);
    // the table reports what was actually generated.
    let lo = REGION_MIN_M.map(|m| (m / chunk_m).floor() as i32);
    let hi = REGION_MAX_M.map(|m| ((m / chunk_m).ceil() as i32) - 1);

    let mut chunks: HashMap<ChunkPos, Chunk> = HashMap::new();
    let gen_start = Instant::now();
    for cy in lo[1]..=hi[1] {
        for cz in lo[2]..=hi[2] {
            for cx in lo[0]..=hi[0] {
                let pos = ChunkPos::new(cx, cy, cz);
                chunks.insert(pos, generator.generate_chunk(scale, pos));
            }
        }
    }
    let gen_time_s = gen_start.elapsed().as_secs_f64();

    let neighbor_solid = |x: i64, y: i64, z: i64| -> bool {
        let pos = ChunkPos::from_world_voxel(x, y, z);
        match chunks.get(&pos) {
            Some(chunk) => {
                let (lx, ly, lz) = local_voxel(x, y, z);
                chunk.get(lx, ly, lz).is_solid()
            }
            // Outside the region: cull as if solid so the bench doesn't count
            // a fake "wall" of boundary faces that no real world would have.
            None => true,
        }
    };

    let voxel_size_m = scale.voxel_size_m();
    let mesh_start = Instant::now();
    let mut triangles = 0u64;
    for (pos, chunk) in &chunks {
        let mesh = mesh_chunk(chunk, *pos, voxel_size_m as f32, &neighbor_solid, None);
        triangles += mesh.triangle_count() as u64;
    }
    let mesh_time_s = mesh_start.elapsed().as_secs_f64();

    Row {
        player_voxels,
        voxel_size_m,
        chunk_count: chunks.len(),
        voxel_count: (chunks.len() * CHUNK_VOLUME) as u64,
        raw_bytes: (chunks.len() * Chunk::raw_byte_size()) as u64,
        gen_time_s,
        mesh_time_s,
        triangles,
    }
}

pub fn format_bytes(b: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    format!("{:.1} MiB", b as f64 / MIB)
}

pub fn group_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
