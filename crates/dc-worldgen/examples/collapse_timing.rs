//! Per-chunk collapse + material timing on a Medium world — the runtime-perf
//! convention's measurement for the surface-branch removal (journal/0074).
//!
//! Times [`WorldGenerator::generate_chunk_with_materials`] (block collapse +
//! material interning, the hot path a client pays on chunk load) over a warm
//! generator, reporting µs/chunk. Run on this branch for the *after* number and
//! against the pre-slice `collapse.rs` for the *before*:
//!
//! ```text
//! cargo run --release -p dc-worldgen --example collapse_timing
//! ```
//!
//! Deterministic in seed; prints a checksum so a divergent build is visible.

use std::time::Instant;

use dc_core::ChunkPos;
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};
use dc_worldgen::WorldGenerator;

fn main() {
    let seed = 0x0D5E_ED57_2026u64;
    let pregen = Pregen::run(WorldParams {
        seed,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&pregen);

    // A block of chunk columns near the origin, three chunk-ys deep so the
    // surface chunk and the two below it are all built (surface + buried +
    // basement paths all exercised). Warm the caches first (uncounted), then
    // time a fresh sweep with the column cache cleared between columns so every
    // column pays a real collapse — the client's cold-load cost.
    let radius = 12i64; // 25x25 columns
    let ys = [0i32, -1, -2];

    // Warm-up pass (JIT of caches, page-ins) — not counted.
    for cz in -2..=2i64 {
        for cx in -2..=2i64 {
            for &cy in &ys {
                let _ = g.generate_chunk_with_materials(ChunkPos::new(cx as i32, cy, cz as i32));
            }
        }
    }

    let mut checksum = 0u64;
    let mut chunks = 0u64;
    let t = Instant::now();
    for cz in -radius..=radius {
        for cx in -radius..=radius {
            for &cy in &ys {
                let (chunk, mat) =
                    g.generate_chunk_with_materials(ChunkPos::new(cx as i32, cy, cz as i32));
                // Fold a cheap invariant of the output into the checksum so the
                // optimizer cannot elide the work and a divergent build shows.
                checksum = checksum
                    .wrapping_add(chunk.blocks().iter().map(|b| *b as u64).sum::<u64>())
                    .wrapping_add(mat.encode().len() as u64);
                chunks += 1;
            }
        }
    }
    let dt = t.elapsed();
    println!(
        "collapse+materials: {chunks} chunks in {dt:?} = {:.1} µs/chunk (Medium, seed {seed:#x}, checksum {checksum})",
        dt.as_secs_f64() * 1e6 / chunks as f64
    );
}
