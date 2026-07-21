//! **The distribution-first cost probe** (journal/0055) — the slice's main risk
//! measurement.
//!
//! Distribution-first expression deliberately makes neighbouring voxels differ:
//! that is what stochastic rounding *is*. So the honest question is not "did the
//! world get better" but "what did the mixture table cost", measured against
//! S8's numbers for real deposition processes (0.14–0.47 B/m³ sidecar, table
//! entries 7.3–9.4 B each, and the combinatorial cap `C(k+8,8) − 1` on distinct
//! states for a locale mixing `k` materials).
//!
//! Deliberately written against the **public generator API only**, so the same
//! file compiles and runs against the pre-slice tree and produces comparable
//! numbers.
//!
//! `cargo run --release -p dc-worldgen --example mixture_cost_probe`

use std::time::Instant;

use dc_core::{ChunkPos, materials::geology::vanilla};
use dc_worldgen::WorldGenerator;
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;
/// 32³ voxels at 0.9 m.
const CHUNK_M3: f64 = 32.0 * 32.0 * 32.0 * VOXEL_M * VOXEL_M * VOXEL_M;

fn main() {
    println!("=== mixture-table cost probe (journal/0055) ===");
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    println!(
        "ritual (Pregen::run, seed {SEED}, {}): {:.2} s",
        EXTENT.label(),
        t0.elapsed().as_secs_f64()
    );
    println!(
        "DeepField resident: {:.2} MB",
        pregen.deep.resident_bytes() as f64 / (1024.0 * 1024.0)
    );

    let mut g = WorldGenerator::with_geology(&pregen, vanilla());

    // A deterministic spread of buried chunks across the map: for each column,
    // the chunk under the surface chunk (where the record lives).
    let mut positions = Vec::new();
    for k in 0..64i64 {
        let (cx, cz) = (k * 37 - 400, (k % 11) * 23 - 90);
        let cy = g.surface_chunk_y(cx, cz);
        positions.push(ChunkPos::new(cx as i32, cy, cz as i32));
        positions.push(ChunkPos::new(cx as i32, cy - 1, cz as i32));
    }

    let t1 = Instant::now();
    let mut sidecar_bytes = 0usize;
    let mut chunks_with_data = 0usize;
    let mut per_chunk: Vec<usize> = Vec::new();
    let mut filled_voxels = 0usize;
    let mut all_materials = std::collections::HashSet::new();
    for pos in &positions {
        let (_, mat) = g.generate_chunk_with_materials(*pos);
        let enc = mat.encode();
        if !mat.is_all_empty() {
            chunks_with_data += 1;
            sidecar_bytes += enc.len();
            // Distinct mixture ids present in this chunk.
            let grid = g.chunk_contents(*pos);
            if let Some(grid) = grid {
                let mut seen = std::collections::HashSet::new();
                for z in 0..32usize {
                    for y in 0..32usize {
                        for x in 0..32usize {
                            let c = grid.get(x, y, z);
                            if !c.is_empty() {
                                seen.insert(c);
                                filled_voxels += 1;
                                for m in c.filled_slots() {
                                    all_materials.insert(*m);
                                }
                            }
                        }
                    }
                }
                per_chunk.push(seen.len());
            }
        }
    }
    let gen_s = t1.elapsed().as_secs_f64();
    per_chunk.sort_unstable();

    let table = g.mixture_table().encode();
    let entries = g.mixture_table().len();
    let m3 = chunks_with_data as f64 * CHUNK_M3;

    println!("\n--- over {} sampled chunks ---", positions.len());
    println!(
        "  chunks carrying material data : {chunks_with_data} ({:.1} ms/chunk generated)",
        1000.0 * gen_s / positions.len() as f64
    );
    println!("  distinct interned mixtures    : {entries}");
    println!(
        "  mixture table bytes           : {} ({:.1} B/entry)",
        table.len(),
        table.len() as f64 / entries.max(1) as f64
    );
    println!(
        "  distinct mixtures per chunk   : min {} / med {} / max {}",
        per_chunk.first().copied().unwrap_or(0),
        per_chunk.get(per_chunk.len() / 2).copied().unwrap_or(0),
        per_chunk.last().copied().unwrap_or(0)
    );
    println!(
        "  sidecar                       : {sidecar_bytes} B over {m3:.0} m³ = {:.3} B/m³",
        sidecar_bytes as f64 / m3
    );
    println!(
        "  sidecar + table               : {:.3} B/m³",
        (sidecar_bytes + table.len()) as f64 / m3
    );
    // Disentangles "more bytes per voxel" from "more voxels carry a record":
    // distribution-first expression does BOTH (the record now fills the column
    // it always described), and only the first is a cost of the dither.
    println!(
        "  contents-bearing voxels       : {filled_voxels} ({:.3} B/voxel of sidecar)",
        sidecar_bytes as f64 / filled_voxels.max(1) as f64
    );
    let k = all_materials.len() as u64;
    let cap = (1..=8u64).fold(1u64, |a, i| a * (k + i) / i) - 1;
    println!(
        "  distinct materials in sample  : {k} → S8 combinatorial cap C(k+8,8)−1 = {cap}; \
         observed {entries} ({:.4}% of the cap)",
        100.0 * entries as f64 / cap as f64
    );
    // ---- what the world is skinned with -----------------------------------
    //
    // A wide, cheap stride over `coarse_surface` — the SAME kernel the ground
    // uses, which is the point (materials.md § one world answer). Run this file
    // against the pre-slice tree for the "before" column.
    let mut skin: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let stride = 907i64; // coprime-ish with every grid in sight
    let n = 220i64;
    let t2 = Instant::now();
    for i in -n / 2..n / 2 {
        for j in -n / 2..n / 2 {
            let (h, b) = g.coarse_surface(i * stride, j * stride);
            if h < 0 {
                continue; // below the datum: ocean floor, not a walkable skin
            }
            *skin.entry(format!("{b:?}")).or_default() += 1;
        }
    }
    let total: usize = skin.values().sum();
    println!(
        "\n--- what the world is skinned with ({total} land samples at {stride}-voxel stride, \
         {:.1} µs/sample) ---",
        1e6 * t2.elapsed().as_secs_f64() / (n * n) as f64
    );
    let mut rows: Vec<_> = skin.into_iter().collect();
    rows.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (b, c) in rows {
        println!(
            "  {b:<24} {c:>7}  {:>5.1}%",
            100.0 * c as f64 / total as f64
        );
    }

    println!(
        "\nS8 baselines: real deposit chunks 0.14–0.47 B/m³; raw dense material grid \
         10.97 B/m³; base block grid 0.144 B/m³ region average."
    );
}
