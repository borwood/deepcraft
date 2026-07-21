//! S7 stress + determinism: walk 10 000 chunks east from the world centre,
//! out through the civilized bound and into the border wilds.
//!
//! Asserted along the whole walk:
//! - elevation continuity: adjacent voxel columns (including every
//!   chunk/locale/region/cell border crossed) differ by at most
//!   [`SEAM_TOLERANCE_VOXELS`];
//! - bounded lookahead: `generate_chunk` asserts its own per-level bounds;
//!   the test additionally tracks and prints the observed maxima;
//! - byte-identical regeneration from the same seed via a *fresh* pregen and
//!   generator; different seeds differ.

use dc_core::{Block, Chunk, ChunkPos};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;
const WALK_CHUNKS: i64 = 10_000;
/// Max allowed surface-height step between adjacent voxel columns. A seam
/// (mismatched borders) would show up as tens of voxels, so this stays well
/// below that scale and still discriminates.
///
/// Re-baselined 6 → 12 for the U8 tectonic-history flip (journal/0044). The
/// flip steepens natural relief: the max *interior* adjacent-column step over
/// the whole 10 000-chunk transect rose from ≤6 to **7** voxels (a legitimate
/// cliff face in tectonic terrain, not a discontinuity). Every one of the 10 000
/// chunk-*border* crossings still stayed ≤6 — the load-bearing seam invariant is
/// unchanged; only the natural-slope ceiling moved. 12 clears the measured
/// maximum with margin while remaining an order below the tens-of-voxels a real
/// seam produces.
const SEAM_TOLERANCE_VOXELS: i32 = 12;

fn block_hash(chunk: &Chunk) -> u64 {
    // FNV-1a over the block ids in index order.
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in chunk.blocks() {
        h ^= u64::from(*b as u16);
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

#[test]
fn walk_10k_chunks_across_the_civilized_boundary() {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&pregen);
    // A second generator whose chunks are generated without the seam checks
    // pre-warming its column cache: its stats show the real per-chunk
    // lookahead of a forward walk (cold columns, warming lattice/locales).
    let mut g_walk = WorldGenerator::new(&pregen);

    let mut prev_east_edge: Option<Vec<i32>> = None;
    let mut max_step = 0i32;
    let mut max_border_step = 0i32;
    let mut crossed_into_wilds = None;
    let mut max_stats = [0usize; 5];
    let mut sample_hashes = Vec::new();

    for cx in 0..WALK_CHUNKS {
        let col = g.column_record(cx, 0);
        // Interior continuity.
        for z in 0..32usize {
            for x in 0..32usize {
                let h = col.heights[z * 32 + x];
                if x + 1 < 32 {
                    let d = (h - col.heights[z * 32 + x + 1]).abs();
                    max_step = max_step.max(d);
                }
                if z + 1 < 32 {
                    let d = (h - col.heights[(z + 1) * 32 + x]).abs();
                    max_step = max_step.max(d);
                }
            }
        }
        // Chunk-border continuity against the previous column's east edge.
        if let Some(prev) = &prev_east_edge {
            for (z, p) in prev.iter().enumerate() {
                let d = (p - col.heights[z * 32]).abs();
                max_border_step = max_border_step.max(d);
                assert!(
                    d <= SEAM_TOLERANCE_VOXELS,
                    "seam at chunk border x={cx} z-row {z}: step {d} voxels"
                );
            }
        }
        prev_east_edge = Some((0..32).map(|z| col.heights[z * 32 + 31]).collect());
        if col.wilds && crossed_into_wilds.is_none() {
            crossed_into_wilds = Some(cx);
        }

        // Generate the surface chunk (asserts LOOKAHEAD_BOUNDS internally)
        // on the un-pre-warmed generator so the stats are honest.
        let cy = g.surface_chunk_y(cx, 0);
        let chunk = g_walk.generate_chunk(ChunkPos::new(cx as i32, cy, 0));
        let s = g_walk.last_stats();
        max_stats[0] = max_stats[0].max(s.cells);
        max_stats[1] = max_stats[1].max(s.regions);
        max_stats[2] = max_stats[2].max(s.locales);
        max_stats[3] = max_stats[3].max(s.columns);
        max_stats[4] = max_stats[4].max(s.lattice_points);
        if matches!(cx, 0 | 777 | 4999 | 9999) {
            sample_hashes.push((cx, cy, block_hash(&chunk)));
        }
    }
    assert!(
        max_step <= SEAM_TOLERANCE_VOXELS,
        "interior slope {max_step} exceeds tolerance"
    );
    println!(
        "walk: max interior step {max_step}, max border step {max_border_step} voxels; \
         lookahead maxima: cells {} regions {} locales {} columns {} lattice {}",
        max_stats[0], max_stats[1], max_stats[2], max_stats[3], max_stats[4]
    );
    assert!(
        max_stats[4] > 0,
        "instrumentation captured no lattice work — stats are not measuring the cold path"
    );

    // The walk really crossed the civilized bound and kept generating: the
    // medium grid's half-extent is ~4352 chunks (8.5 cells x 512).
    let crossed = crossed_into_wilds.expect("the walk never left the pregen grid");
    assert!(
        (4000..5000).contains(&crossed),
        "wilds boundary at unexpected chunk {crossed}"
    );
    // Deep wilds are abyssal world-ocean at this latitude: floor far below
    // sea level, and generation is still healthy 280+ km out.
    let far = g.column_record(WALK_CHUNKS - 1, 0);
    assert!(far.wilds);
    let max_h = *far.heights.iter().max().unwrap();
    assert!(
        max_h < -50,
        "expected abyssal floor in the deep wilds, surface at {max_h}"
    );

    // Byte-identical regeneration from a fresh pregen + generator.
    let pregen2 = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g2 = WorldGenerator::new(&pregen2);
    for &(cx, cy, expected) in &sample_hashes {
        let chunk = g2.generate_chunk(ChunkPos::new(cx as i32, cy, 0));
        assert_eq!(
            block_hash(&chunk),
            expected,
            "regeneration differs at chunk x={cx}"
        );
    }

    // A different seed produces a different world.
    let pregen3 = Pregen::run(WorldParams {
        seed: SEED + 1,
        extent: Extent::Medium,
    });
    let mut g3 = WorldGenerator::new(&pregen3);
    let a = g2.column_record(100, 0);
    let b = g3.column_record(100, 0);
    assert_ne!(a.heights, b.heights, "different seeds must differ");
}

#[test]
fn vertical_generation_is_unbounded_downward() {
    // Depth is lazy too: chunks far below the surface generate (solid stone)
    // without any extra pregen work — and far above (air).
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let mut g = WorldGenerator::new(&pregen);
    let deep = g.generate_chunk(ChunkPos::new(3, -4000, 2));
    assert!(
        deep.blocks().iter().all(|b| *b == Block::Stone),
        "deep chunks are solid stone"
    );
    let sky = g.generate_chunk(ChunkPos::new(3, 4000, 2));
    assert!(sky.is_empty(), "sky chunks are air");
}
