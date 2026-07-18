//! Per-column summaries: lazy heightmap / sky-exposure cache (S3).
//!
//! Cubic chunks mean "under open sky" cannot be answered inside one chunk and
//! nothing may assume a bounded world height. These summaries answer "what's
//! the surface here / is there open air below y" from whatever data is
//! already resident — loaded full-res chunks and LOD pyramid levels — without
//! ever loading a full-res chunk.
//!
//! **Skylight query contract** (the S3 hard problem, stated once, here):
//!
//! - A sky/light query consults ONLY (a) chunks already resident at some LOD
//!   level and (b) cached column summaries. It never loads, generates, or
//!   touches "the column above".
//! - Every query is explicitly bounded to a caller-supplied `[y_min, y_max)`
//!   scan range in level-0 voxel coordinates; there is no unbounded upward or
//!   downward walk.
//! - Volumes with no resident data at any level are treated as
//!   **non-occluding** (optimistic sky) and the answer is flagged
//!   `fully_resolved = false`. Consumers needing certainty react to the flag
//!   (schedule LOD derivation, widen the scan, or accept the coarse answer) —
//!   they do not get to demand chunk loads from inside a light query.
//! - Answers carry the `resolution` (LOD level) they were derived at: a
//!   surface reported from level L is exact to `2^L` voxels.

use std::collections::HashMap;

use crate::lod::LodPyramid;
use crate::voxel::Block;

/// Source of "finest resident data covering a level-0 voxel" queries.
/// `&mut` because lazy LOD derivation may materialize cached levels.
pub trait LodBlockSource {
    /// `(level, block)` read at the finest resident level, or `None` when no
    /// data covers the voxel at any level.
    fn best_block(&mut self, vx: i64, vy: i64, vz: i64) -> Option<(u8, Block)>;
}

impl LodBlockSource for LodPyramid {
    fn best_block(&mut self, vx: i64, vy: i64, vz: i64) -> Option<(u8, Block)> {
        LodPyramid::best_block(self, vx, vy, vz)
    }
}

/// Summary of one (x, z) voxel column over a bounded y range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnInfo {
    /// Level-0 y of the top voxel of the highest solid cell found
    /// (the heightmap value). `None` when the scanned range holds no known
    /// solid.
    pub top_solid_y: Option<i64>,
    /// LOD level of the data that produced `top_solid_y`: the surface is
    /// exact to `2^resolution` voxels. 0 when no solid was found.
    pub resolution: u8,
    /// Lowest level-0 y of any solid cell seen in the range.
    pub min_solid_y: Option<i64>,
    /// Highest level-0 y of any solid cell seen in the range
    /// (equals `top_solid_y` by construction).
    pub max_solid_y: Option<i64>,
    /// True when everything above the surface (or the whole range, if no
    /// solid) was *known* air — no unknown gap that might hide an occluder.
    pub sky_exposed: bool,
    /// False when any part of the scanned range had no resident data at any
    /// LOD level (such gaps are treated as air; see the contract above).
    pub fully_resolved: bool,
}

/// Scan one column downward through resident data. `y_min..y_max` bounds the
/// scan in level-0 voxel coordinates; cells are consumed at whatever LOD
/// resolution answers, so coarse regions cost one probe per `2^L` voxels.
pub fn column_summary(
    src: &mut dyn LodBlockSource,
    x: i64,
    z: i64,
    y_min: i64,
    y_max: i64,
) -> ColumnInfo {
    let mut info = ColumnInfo {
        top_solid_y: None,
        resolution: 0,
        min_solid_y: None,
        max_solid_y: None,
        sky_exposed: true,
        fully_resolved: true,
    };
    let mut gap_above_surface = false;
    let mut y = y_max - 1;
    while y >= y_min {
        match src.best_block(x, y, z) {
            None => {
                info.fully_resolved = false;
                if info.top_solid_y.is_none() {
                    gap_above_surface = true;
                }
                // No data at any level means the whole level-0 chunk cube
                // containing this voxel is unknown; skip below it.
                y = ((y >> crate::chunk::CHUNK_BITS) << crate::chunk::CHUNK_BITS) - 1;
            }
            Some((level, block)) => {
                let base = (y >> level) << level;
                // Clamp coarse cells poking above the scan range.
                let top = (base + (1i64 << level) - 1).min(y_max - 1);
                if block.is_solid() {
                    if info.top_solid_y.is_none() {
                        info.top_solid_y = Some(top);
                        info.resolution = level;
                        info.sky_exposed = !gap_above_surface;
                    }
                    info.max_solid_y = Some(info.max_solid_y.map_or(top, |m: i64| m.max(top)));
                    info.min_solid_y = Some(info.min_solid_y.map_or(base, |m: i64| m.min(base)));
                }
                y = base - 1;
            }
        }
    }
    if info.top_solid_y.is_none() {
        info.sky_exposed = !gap_above_surface && info.fully_resolved;
    }
    info
}

/// Topmost *known-air* cell strictly below `y_start` (down to `y_min`):
/// `Some(y)` gives the top level-0 voxel of that air cell. Unknown volumes do
/// not count as air. This is the "is there open air below me" probe the
/// adaptive load volume uses to decide whether to stream downward.
pub fn open_air_below(
    src: &mut dyn LodBlockSource,
    x: i64,
    z: i64,
    y_start: i64,
    y_min: i64,
) -> Option<i64> {
    let mut y = y_start - 1;
    while y >= y_min {
        match src.best_block(x, y, z) {
            None => {
                y = ((y >> crate::chunk::CHUNK_BITS) << crate::chunk::CHUNK_BITS) - 1;
            }
            Some((level, block)) => {
                let base = (y >> level) << level;
                if !block.is_solid() {
                    return Some(y);
                }
                y = base - 1;
            }
        }
    }
    None
}

/// Lazy per-column cache over a fixed scan range. Built on demand from
/// resident chunks + LOD data; invalidated wholesale when the world changes
/// (fine-grained invalidation is post-spike work).
pub struct ColumnSummaries {
    y_min: i64,
    y_max: i64,
    cache: HashMap<(i64, i64), ColumnInfo>,
}

impl ColumnSummaries {
    pub fn new(y_min: i64, y_max: i64) -> Self {
        assert!(y_min < y_max, "empty scan range");
        Self {
            y_min,
            y_max,
            cache: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// The summary for column (x, z), computing and caching it on first use.
    pub fn get_or_build(&mut self, src: &mut dyn LodBlockSource, x: i64, z: i64) -> ColumnInfo {
        *self
            .cache
            .entry((x, z))
            .or_insert_with(|| column_summary(src, x, z, self.y_min, self.y_max))
    }

    /// Drop all cached summaries (world changed).
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{CHUNK_SIZE_USIZE, Chunk, ChunkPos};
    use crate::palette::PalettedChunk;
    use crate::voxel::Block;

    /// Ground-at-y=10 chunk (world y 0..32): solid below and including 10.
    fn ground_chunk(surface_local_y: usize) -> PalettedChunk {
        let mut dense = Chunk::new();
        for y in 0..=surface_local_y {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let b = if y == surface_local_y {
                        Block::Grass
                    } else {
                        Block::Stone
                    };
                    dense.set(x, y, z, b);
                }
            }
        }
        PalettedChunk::from_dense(&dense)
    }

    #[test]
    fn full_res_summary_finds_surface() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), ground_chunk(10));
        pyramid.insert_chunk(
            ChunkPos::new(0, -1, 0),
            PalettedChunk::uniform(Block::Stone),
        );
        pyramid.insert_chunk(ChunkPos::new(0, 1, 0), PalettedChunk::uniform(Block::Air));

        let info = column_summary(&mut pyramid, 5, 5, -32, 64);
        assert_eq!(info.top_solid_y, Some(10));
        assert_eq!(info.resolution, 0);
        assert_eq!(info.max_solid_y, Some(10));
        assert_eq!(info.min_solid_y, Some(-32));
        assert!(info.sky_exposed);
        assert!(info.fully_resolved);
    }

    #[test]
    fn coarse_only_summary_answers_without_full_res() {
        // ONLY level-1 data resident: surface must still be answerable, at
        // 2-voxel resolution, without any level-0 chunk existing.
        let mut pyramid = LodPyramid::default();
        let mut dense = Chunk::new();
        for y in 0..=5 {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    dense.set(x, y, z, Block::Stone);
                }
            }
        }
        // L1 chunk at (0,0,0): covers level-0 voxels (0..64)^3; solid rows
        // y_l1 in 0..=5 -> level-0 y in 0..12.
        pyramid.insert_lod_chunk(1, ChunkPos::new(0, 0, 0), PalettedChunk::from_dense(&dense));

        let info = column_summary(&mut pyramid, 5, 5, 0, 64);
        assert_eq!(
            info.top_solid_y,
            Some(11),
            "top of the highest solid 2-voxel cell"
        );
        assert_eq!(info.resolution, 1);
        assert_eq!(info.min_solid_y, Some(0));
        assert!(info.sky_exposed);
        assert!(info.fully_resolved);
    }

    #[test]
    fn overhang_open_air_beneath_is_visible_from_lod_only() {
        // An overhang encoded purely in level-2 data (no full-res anywhere):
        // solid slab at L2 rows 20..=21, open air 8..=19, ground 0..=7.
        let mut pyramid = LodPyramid::default();
        let mut dense = Chunk::new();
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                for y in 0..=7 {
                    dense.set(x, y, z, Block::Stone);
                }
                for y in 20..=21 {
                    dense.set(x, y, z, Block::Stone);
                }
            }
        }
        pyramid.insert_lod_chunk(2, ChunkPos::new(0, 0, 0), PalettedChunk::from_dense(&dense));

        // Level-0 view: slab occupies y 80..88, air 32..80, ground 0..32.
        let info = column_summary(&mut pyramid, 10, 10, 0, 128);
        assert_eq!(info.top_solid_y, Some(87), "overhang top");
        assert_eq!(info.resolution, 2);
        assert_eq!(info.min_solid_y, Some(0));
        assert!(info.fully_resolved);

        // The load-volume probe: open air *below* the overhang surface must be
        // reported, using LOD data only.
        let air = open_air_below(&mut pyramid, 10, 10, 80, 0);
        assert_eq!(air, Some(79), "air pocket beneath the overhang");
        // And below the ground there is no known air.
        assert_eq!(open_air_below(&mut pyramid, 10, 10, 32 - 32, -64), None);
    }

    #[test]
    fn unknown_gap_flags_and_optimistic_sky() {
        let mut pyramid = LodPyramid::default();
        // Ground chunk at y-chunk 0, NOTHING above it resident.
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), ground_chunk(10));

        let info = column_summary(&mut pyramid, 3, 3, 0, 96);
        assert_eq!(info.top_solid_y, Some(10));
        assert!(!info.fully_resolved, "y 32..96 was unknown");
        assert!(
            !info.sky_exposed,
            "unknown volume above the surface: sky not certain"
        );

        // With a known-air chunk above, the sky becomes certain.
        pyramid.insert_chunk(ChunkPos::new(0, 1, 0), PalettedChunk::uniform(Block::Air));
        pyramid.insert_chunk(ChunkPos::new(0, 2, 0), PalettedChunk::uniform(Block::Air));
        let info = column_summary(&mut pyramid, 3, 3, 0, 96);
        assert!(info.fully_resolved);
        assert!(info.sky_exposed);
    }

    #[test]
    fn no_solid_column_reports_open() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), PalettedChunk::uniform(Block::Air));
        let info = column_summary(&mut pyramid, 1, 1, 0, 32);
        assert_eq!(info.top_solid_y, None);
        assert_eq!(info.min_solid_y, None);
        assert!(info.sky_exposed);
        assert!(info.fully_resolved);
    }

    #[test]
    fn summaries_cache_is_lazy_and_stable() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), ground_chunk(7));
        let mut summaries = ColumnSummaries::new(0, 32);
        assert!(summaries.is_empty());
        let a = summaries.get_or_build(&mut pyramid, 4, 9);
        assert_eq!(summaries.len(), 1);
        let b = summaries.get_or_build(&mut pyramid, 4, 9);
        assert_eq!(a, b, "cached answer identical");
        assert_eq!(a.top_solid_y, Some(7));

        // Determinism across a fresh pyramid + cache.
        let mut pyramid2 = LodPyramid::default();
        pyramid2.insert_chunk(ChunkPos::new(0, 0, 0), ground_chunk(7));
        let mut summaries2 = ColumnSummaries::new(0, 32);
        assert_eq!(summaries2.get_or_build(&mut pyramid2, 4, 9), a);
    }
}
