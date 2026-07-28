//! TEMPORARY (journal/0121): the after-measurement for the bootstrap-history
//! removal. Deleted in the same slice; never merged.
//!
//! The 12 chunks below are the exact `ChunkPos`es that carried all 102
//! `Block::Wood` ruin-post voxels on the pre-removal tree (seed
//! 0x0D5EED572026, Extent::Medium) — measured by this file's own previous
//! revision, commit 76e6a4c.

use dc_core::{Block, ChunkPos};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

/// `(cx, cy, cz, wood voxels on the pre-removal tree)`.
const WAS_WOOD: [(i32, i32, i32, usize); 12] = [
    (-2805, -49, -728, 13),
    (-2805, -49, -727, 5),
    (-2804, -49, -727, 6),
    (-785, -5, -2871, 10),
    (-785, -5, -2870, 17),
    (-1840, -30, -1336, 11),
    (-1839, -30, -1336, 9),
    (-1840, -30, -1335, 5),
    (-1839, -30, -1335, 2),
    (235, 20, -2805, 6),
    (234, 20, -2804, 3),
    (235, 20, -2804, 15),
];

#[test]
fn the_ruin_posts_are_gone_from_the_world() {
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&p);
    let mut total = 0usize;
    for (cx, cy, cz, was) in WAS_WOOD {
        let chunk = g.generate_chunk(ChunkPos::new(cx, cy, cz));
        let w = chunk.blocks().iter().filter(|b| **b == Block::Wood).count();
        println!("({cx},{cy},{cz}): was {was}, now {w}");
        total += w;
    }
    println!("WOOD over the 12 formerly-wood-bearing chunks = {total} (was 102)");
    assert_eq!(total, 0);
}

#[test]
fn nor_anywhere_near_where_they_stood() {
    // A 5x5 chunk box, two y slabs, around each of the four former ruin
    // clusters: the posts did not move, they are absent.
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&p);
    let mut total = 0usize;
    let mut chunks = 0usize;
    for (ccx, ccz) in [(-2805i64, -728i64), (-785, -2871), (-1840, -1336), (235, -2805)] {
        for dz in -2..=2i64 {
            for dx in -2..=2i64 {
                let (cx, cz) = (ccx + dx, ccz + dz);
                let cy = g.surface_chunk_y(cx, cz);
                for y in [cy, cy + 1] {
                    let chunk = g.generate_chunk(ChunkPos::new(cx as i32, y, cz as i32));
                    total += chunk.blocks().iter().filter(|b| **b == Block::Wood).count();
                    chunks += 1;
                }
            }
        }
    }
    println!("WOOD over {chunks} chunks around the four former clusters = {total}");
    assert_eq!(total, 0);
}
