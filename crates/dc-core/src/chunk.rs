//! Cubic chunks: a 3D lattice of 32^3-voxel cubes.
//!
//! Chunk size rationale (32, i.e. 2^5 per edge):
//! - Cubic chunks (see docs/ARCHITECTURE.md § World structure) want a cube, not
//!   a column, so one edge length governs all three axes.
//! - 32^3 = 32768 voxels; at 2 bytes per block id that is 64 KiB of raw block
//!   data per chunk — big enough that per-chunk overhead (hash map entry,
//!   entity, mesh) is amortized, small enough that a single block edit remeshes
//!   only 64 KiB worth of world and chunk gen/mesh stays comfortably under a
//!   frame budget.
//! - Power of two keeps voxel→chunk math to shifts/masks.
//! - 16^3 (Minecraft's section size) doubles chunk-count overhead for our
//!   deep-world streaming radius; 64^3 (256 KiB, 262k voxels) makes single-chunk
//!   remesh latency and gen granularity noticeably worse.
//!
//! Storage is a plain boxed array for S1. Palette compression and LOD pyramids
//! are S3's job — nothing here may preclude them, nothing here implements them.

use serde::{Deserialize, Serialize};

use crate::voxel::Block;

/// log2 of the chunk edge length in voxels.
pub const CHUNK_BITS: u32 = 5;
/// Chunk edge length in voxels.
pub const CHUNK_SIZE: i32 = 1 << CHUNK_BITS;
/// Chunk edge length as usize, for indexing.
pub const CHUNK_SIZE_USIZE: usize = CHUNK_SIZE as usize;
/// Voxels per chunk.
pub const CHUNK_VOLUME: usize = CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;

/// Position of a chunk in the 3D chunk lattice. All three axes are equal
/// citizens: traveling down loads deeper chunks exactly like traveling north
/// loads farther ones. No column assumptions, no bounded world height.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The chunk containing the given world-space voxel coordinate.
    #[inline]
    pub fn from_world_voxel(vx: i64, vy: i64, vz: i64) -> Self {
        Self {
            x: (vx >> CHUNK_BITS) as i32,
            y: (vy >> CHUNK_BITS) as i32,
            z: (vz >> CHUNK_BITS) as i32,
        }
    }

    /// World-space voxel coordinate of this chunk's minimum corner.
    #[inline]
    pub fn min_voxel(self) -> (i64, i64, i64) {
        (
            (self.x as i64) << CHUNK_BITS,
            (self.y as i64) << CHUNK_BITS,
            (self.z as i64) << CHUNK_BITS,
        )
    }
}

/// Local voxel coordinate within a chunk (each component in `0..CHUNK_SIZE`)
/// for a world-space voxel coordinate.
#[inline]
pub fn local_voxel(vx: i64, vy: i64, vz: i64) -> (usize, usize, usize) {
    let mask = (CHUNK_SIZE - 1) as i64;
    (
        (vx & mask) as usize,
        (vy & mask) as usize,
        (vz & mask) as usize,
    )
}

/// A 32^3 cube of voxels. Dense boxed array for S1 (see module docs).
pub struct Chunk {
    blocks: Box<[Block; CHUNK_VOLUME]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// An all-air chunk.
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_VOLUME]
                .into_boxed_slice()
                .try_into()
                .expect("length matches CHUNK_VOLUME"),
        }
    }

    /// Linear index for a local coordinate. Layout is x-fastest, then z, then y
    /// (`x + z*32 + y*32*32`) so horizontal slices are contiguous.
    ///
    /// # Panics
    /// Panics if any coordinate is `>= CHUNK_SIZE`.
    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        assert!(
            x < CHUNK_SIZE_USIZE && y < CHUNK_SIZE_USIZE && z < CHUNK_SIZE_USIZE,
            "local voxel coordinate out of bounds: ({x}, {y}, {z})"
        );
        x + (z << CHUNK_BITS) + (y << (2 * CHUNK_BITS))
    }

    /// # Panics
    /// Panics if any coordinate is `>= CHUNK_SIZE`.
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::index(x, y, z)]
    }

    /// Bounds-checked read; `None` when out of range.
    #[inline]
    pub fn try_get(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        if x < CHUNK_SIZE_USIZE && y < CHUNK_SIZE_USIZE && z < CHUNK_SIZE_USIZE {
            Some(self.blocks[Self::index(x, y, z)])
        } else {
            None
        }
    }

    /// # Panics
    /// Panics if any coordinate is `>= CHUNK_SIZE`.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[Self::index(x, y, z)] = block;
    }

    /// True if every voxel is air (nothing to mesh or collide with).
    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|b| *b == Block::Air)
    }

    /// All blocks in [`Chunk::index`] order (x-fastest, then z, then y).
    /// Read-only bulk access for compression and analysis.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks[..]
    }

    /// Bytes of raw block storage (the measurement S1 reports on).
    pub const fn raw_byte_size() -> usize {
        CHUNK_VOLUME * std::mem::size_of::<Block>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_layout_is_x_fastest() {
        assert_eq!(Chunk::index(0, 0, 0), 0);
        assert_eq!(Chunk::index(1, 0, 0), 1);
        assert_eq!(Chunk::index(0, 0, 1), CHUNK_SIZE_USIZE);
        assert_eq!(Chunk::index(0, 1, 0), CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE);
        assert_eq!(Chunk::index(31, 31, 31), CHUNK_VOLUME - 1);
    }

    #[test]
    fn index_is_a_bijection_over_the_chunk() {
        let mut seen = vec![false; CHUNK_VOLUME];
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let i = Chunk::index(x, y, z);
                    assert!(!seen[i], "index collision at ({x}, {y}, {z})");
                    seen[i] = true;
                }
            }
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn index_panics_out_of_bounds() {
        Chunk::index(32, 0, 0);
    }

    #[test]
    fn try_get_bounds() {
        let c = Chunk::new();
        assert_eq!(c.try_get(31, 31, 31), Some(Block::Air));
        assert_eq!(c.try_get(32, 0, 0), None);
        assert_eq!(c.try_get(0, 32, 0), None);
        assert_eq!(c.try_get(0, 0, 32), None);
    }

    #[test]
    fn set_then_get_roundtrip() {
        let mut c = Chunk::new();
        c.set(3, 4, 5, Block::Stone);
        c.set(0, 0, 0, Block::Grass);
        c.set(31, 31, 31, Block::Wood);
        assert_eq!(c.get(3, 4, 5), Block::Stone);
        assert_eq!(c.get(0, 0, 0), Block::Grass);
        assert_eq!(c.get(31, 31, 31), Block::Wood);
        assert_eq!(c.get(3, 4, 6), Block::Air);
        assert!(!c.is_empty());
    }

    #[test]
    fn chunk_pos_from_world_voxel_handles_negatives() {
        assert_eq!(ChunkPos::from_world_voxel(0, 0, 0), ChunkPos::new(0, 0, 0));
        assert_eq!(
            ChunkPos::from_world_voxel(31, 32, 63),
            ChunkPos::new(0, 1, 1)
        );
        assert_eq!(
            ChunkPos::from_world_voxel(-1, -32, -33),
            ChunkPos::new(-1, -1, -2)
        );
    }

    #[test]
    fn min_voxel_roundtrips_with_local() {
        let pos = ChunkPos::new(-2, 3, -1);
        let (mx, my, mz) = pos.min_voxel();
        assert_eq!((mx, my, mz), (-64, 96, -32));
        // Every world voxel in the chunk maps back to (this chunk, valid local).
        for (wx, wy, wz) in [
            (mx, my, mz),
            (mx + 31, my + 31, mz + 31),
            (mx + 7, my + 30, mz + 1),
        ] {
            assert_eq!(ChunkPos::from_world_voxel(wx, wy, wz), pos);
            let (lx, ly, lz) = local_voxel(wx, wy, wz);
            assert!(lx < CHUNK_SIZE_USIZE && ly < CHUNK_SIZE_USIZE && lz < CHUNK_SIZE_USIZE);
            assert_eq!(
                (mx + lx as i64, my + ly as i64, mz + lz as i64),
                (wx, wy, wz)
            );
        }
    }

    #[test]
    fn raw_byte_size_is_64_kib() {
        assert_eq!(Chunk::raw_byte_size(), 64 * 1024);
    }
}
