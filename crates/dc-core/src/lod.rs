//! Octree-style LOD pyramid over cubic chunks (S3).
//!
//! Levels form a 3D lattice of 32^3 chunks at every level: the level-L+1 chunk
//! at position P summarizes the 2x2x2 level-L chunks at positions
//! `2P + {0,1}^3`. One level-L+1 voxel covers a 2x2x2 cell of level-L voxels,
//! and — because the cell grid is aligned — each output voxel's cell lies
//! entirely inside exactly one child chunk.
//!
//! **Downsample rule (pluggable via [`DownsampleRule`]).** The default,
//! [`MajorityNonAir`]:
//!
//! - the output voxel is non-air iff >= 4 of the 8 child voxels are non-air
//!   (ties go to solid, so thin walls/floors survive rather than dissolve);
//! - the representative block is the most frequent non-air child block, ties
//!   broken by the *higher* block id (later-registered blocks are the more
//!   specific/surface-y ones, so e.g. a grass/dirt tie reads as grass).
//!
//! **Incremental derivation.** The pyramid tracks dirt: inserting or removing
//! a chunk marks only its ancestor positions dirty, and [`LodPyramid::get_or_derive`]
//! re-derives lazily along that one path — changing one level-0 chunk
//! re-derives at most one chunk per level.
//!
//! **Coverage vs knowledge.** Two distinct notions, deliberately:
//!
//! - A level-L *position* participates in derivation iff any chunk was
//!   inserted somewhere in its subtree (at level 0 or directly at a higher
//!   level via [`LodPyramid::insert_lod_chunk`], e.g. LOD data loaded from
//!   disk without its full-res children). During derivation a missing child
//!   reads as all-air.
//! - A level-0 *voxel* is **known** ([`LodPyramid::is_known`]) iff some
//!   inserted chunk's cube covers it. Derived chunks summarize inserted data
//!   but never extend knowledge: a coarse chunk derived from one loaded child
//!   does not make the seven unloaded siblings read as "known air". This is
//!   the boundary the column-summary/skylight contract relies on.

use std::collections::{HashMap, HashSet};

use crate::chunk::{CHUNK_SIZE_USIZE, Chunk, ChunkPos};
use crate::palette::PalettedChunk;
use crate::voxel::Block;

/// Highest LOD level the default pyramid derives (level 0 = full resolution).
/// Level 4 chunks cover 512^3 level-0 voxels (~460 m cubes at the N=2 scale).
pub const MAX_LOD_LEVEL: u8 = 4;

/// Reduces a 2x2x2 cell of child voxels to one parent voxel.
///
/// Implementations must be pure functions of the cell contents so derivation
/// stays deterministic and cacheable.
pub trait DownsampleRule {
    fn reduce(&self, cell: &[Block; 8]) -> Block;
}

/// Default rule: majority-non-air with priority ordering. See module docs.
#[derive(Clone, Copy, Debug, Default)]
pub struct MajorityNonAir;

impl DownsampleRule for MajorityNonAir {
    fn reduce(&self, cell: &[Block; 8]) -> Block {
        let solid = cell.iter().filter(|b| b.is_solid()).count();
        if solid * 2 < cell.len() {
            return Block::Air;
        }
        // Most frequent non-air block; ties broken by higher block id.
        let mut best = Block::Air;
        let mut best_count = 0usize;
        for &candidate in cell {
            if !candidate.is_solid() {
                continue;
            }
            let count = cell.iter().filter(|b| **b == candidate).count();
            if count > best_count || (count == best_count && candidate as u16 > best as u16) {
                best = candidate;
                best_count = count;
            }
        }
        best
    }
}

/// Position of the level-L+1 chunk containing a level-L chunk position.
/// Arithmetic shift keeps negative coordinates correct (floor division).
#[inline]
pub fn parent_pos(pos: ChunkPos) -> ChunkPos {
    ChunkPos::new(pos.x >> 1, pos.y >> 1, pos.z >> 1)
}

/// Ancestor of a level-L position, `levels_up` levels higher.
#[inline]
pub fn ancestor_pos(pos: ChunkPos, levels_up: u8) -> ChunkPos {
    ChunkPos::new(pos.x >> levels_up, pos.y >> levels_up, pos.z >> levels_up)
}

/// The 8 child positions of a parent position, indexed `dx | dz<<1 | dy<<2`.
#[inline]
pub fn child_positions(parent: ChunkPos) -> [ChunkPos; 8] {
    let mut out = [ChunkPos::new(0, 0, 0); 8];
    for dy in 0i32..2 {
        for dz in 0i32..2 {
            for dx in 0i32..2 {
                out[(dx | (dz << 1) | (dy << 2)) as usize] =
                    ChunkPos::new(parent.x * 2 + dx, parent.y * 2 + dy, parent.z * 2 + dz);
            }
        }
    }
    out
}

/// Derive one parent chunk from its (up to 8) children. Missing children read
/// as all-air. `children` is indexed by [`child_positions`] order.
pub fn derive_lod_chunk(
    children: &[Option<&PalettedChunk>; 8],
    rule: &dyn DownsampleRule,
) -> PalettedChunk {
    // Fast path: all children uniform (or missing = uniform air). Each output
    // octant then has a single cell value repeated, so the result is at most
    // 8 constant octants — and usually fully uniform (deep stone, open sky).
    let uniforms: [Option<Block>; 8] = std::array::from_fn(|i| match children[i] {
        None => Some(Block::Air),
        Some(c) => c.is_uniform(),
    });
    if uniforms.iter().all(Option::is_some) {
        let octant_blocks: [Block; 8] = std::array::from_fn(|i| {
            let b = uniforms[i].expect("checked above");
            rule.reduce(&[b; 8])
        });
        if octant_blocks.iter().all(|b| *b == octant_blocks[0]) {
            return PalettedChunk::uniform(octant_blocks[0]);
        }
        let mut dense = Chunk::new();
        let half = CHUNK_SIZE_USIZE / 2;
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let octant = usize::from(x >= half)
                        | (usize::from(z >= half) << 1)
                        | (usize::from(y >= half) << 2);
                    let b = octant_blocks[octant];
                    if b != Block::Air {
                        dense.set(x, y, z, b);
                    }
                }
            }
        }
        return PalettedChunk::from_dense(&dense);
    }

    let mut dense = Chunk::new();
    let half = CHUNK_SIZE_USIZE / 2;
    for y in 0..CHUNK_SIZE_USIZE {
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                // The 2x2x2 source cell in combined child space.
                let (cx, cy, cz) = (x * 2, y * 2, z * 2);
                let octant = usize::from(x >= half)
                    | (usize::from(z >= half) << 1)
                    | (usize::from(y >= half) << 2);
                let block = match children[octant] {
                    None => Block::Air,
                    Some(child) => {
                        let (bx, by, bz) = (
                            cx % CHUNK_SIZE_USIZE,
                            cy % CHUNK_SIZE_USIZE,
                            cz % CHUNK_SIZE_USIZE,
                        );
                        let mut cell = [Block::Air; 8];
                        for (i, slot) in cell.iter_mut().enumerate() {
                            *slot =
                                child.get(bx + (i & 1), by + ((i >> 2) & 1), bz + ((i >> 1) & 1));
                        }
                        rule.reduce(&cell)
                    }
                };
                if block != Block::Air {
                    dense.set(x, y, z, block);
                }
            }
        }
    }
    PalettedChunk::from_dense(&dense)
}

/// A multi-level chunk store with lazy, incremental LOD derivation.
pub struct LodPyramid {
    max_level: u8,
    rule: Box<dyn DownsampleRule + Send + Sync>,
    /// `levels[l]` holds level-l chunks (level 0 = inserted full-res data).
    levels: Vec<HashMap<ChunkPos, PalettedChunk>>,
    /// `inserted[l]`: positions holding externally *inserted* (not derived)
    /// data. A level-0 voxel is "known" iff some inserted chunk's cube covers
    /// it — derived chunks summarize but never extend knowledge.
    inserted: Vec<HashSet<ChunkPos>>,
    /// `coverage[l]` counts inserted chunks per level-l position's subtree.
    coverage: Vec<HashMap<ChunkPos, u32>>,
    /// Positions (levels >= 1) whose stored derivation is stale.
    dirty: Vec<HashSet<ChunkPos>>,
    derive_count: u64,
}

impl Default for LodPyramid {
    fn default() -> Self {
        Self::new(MAX_LOD_LEVEL, Box::new(MajorityNonAir))
    }
}

impl LodPyramid {
    pub fn new(max_level: u8, rule: Box<dyn DownsampleRule + Send + Sync>) -> Self {
        let n = usize::from(max_level) + 1;
        Self {
            max_level,
            rule,
            levels: vec![HashMap::new(); n],
            inserted: vec![HashSet::new(); n],
            coverage: vec![HashMap::new(); n],
            dirty: vec![HashSet::new(); n],
            derive_count: 0,
        }
    }

    pub fn max_level(&self) -> u8 {
        self.max_level
    }

    /// Total chunks derived since construction (for incrementality tests and
    /// the derive-cost bench).
    pub fn derive_count(&self) -> u64 {
        self.derive_count
    }

    /// Number of chunks stored at a level (derived or inserted).
    pub fn stored_at(&self, level: u8) -> usize {
        self.levels[usize::from(level)].len()
    }

    /// Known positions at a level (positions with any inserted data in their
    /// subtree), in arbitrary order.
    pub fn known_positions(&self, level: u8) -> Vec<ChunkPos> {
        self.coverage[usize::from(level)].keys().copied().collect()
    }

    /// Insert (or replace) a full-resolution chunk. Ancestors become dirty.
    pub fn insert_chunk(&mut self, pos: ChunkPos, chunk: PalettedChunk) {
        self.insert_lod_chunk(0, pos, chunk);
    }

    /// Insert externally produced data directly at a level (e.g. LOD chunks
    /// loaded from disk without their full-res children). Levels above become
    /// dirty; levels below are untouched.
    ///
    /// # Panics
    /// Panics if `level > max_level`.
    pub fn insert_lod_chunk(&mut self, level: u8, pos: ChunkPos, chunk: PalettedChunk) {
        assert!(
            level <= self.max_level,
            "level {level} beyond max {}",
            self.max_level
        );
        let l = usize::from(level);
        self.levels[l].insert(pos, chunk);
        let newly_inserted = self.inserted[l].insert(pos);
        if newly_inserted {
            for up in l..self.levels.len() {
                let apos = ancestor_pos(pos, (up - l) as u8);
                *self.coverage[up].entry(apos).or_insert(0) += 1;
            }
        }
        for up in (l + 1)..self.levels.len() {
            self.dirty[up].insert(ancestor_pos(pos, (up - l) as u8));
        }
    }

    /// Remove a full-resolution chunk. Ancestors become dirty and coverage
    /// shrinks; a fully vacated subtree stops being known.
    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        if !self.inserted[0].remove(&pos) {
            return;
        }
        self.levels[0].remove(&pos);
        for up in 0..self.levels.len() {
            let apos = ancestor_pos(pos, up as u8);
            if let Some(count) = self.coverage[up].get_mut(&apos) {
                *count -= 1;
                if *count == 0 {
                    self.coverage[up].remove(&apos);
                    self.levels[up].remove(&apos);
                    self.dirty[up].remove(&apos);
                    continue;
                }
            }
            if up >= 1 {
                self.dirty[up].insert(apos);
            }
        }
    }

    /// The chunk at (level, pos), deriving it (and any stale descendants on
    /// its path) if needed. `None` when nothing is known there.
    pub fn get_or_derive(&mut self, level: u8, pos: ChunkPos) -> Option<&PalettedChunk> {
        assert!(
            level <= self.max_level,
            "level {level} beyond max {}",
            self.max_level
        );
        if !self.coverage[usize::from(level)].contains_key(&pos) {
            return None;
        }
        self.ensure_derived(level, pos);
        self.levels[usize::from(level)].get(&pos)
    }

    fn ensure_derived(&mut self, level: u8, pos: ChunkPos) {
        let l = usize::from(level);
        if level == 0 || (!self.dirty[l].contains(&pos) && self.levels[l].contains_key(&pos)) {
            return;
        }
        let children = child_positions(pos);
        for child in children {
            if self.coverage[l - 1].contains_key(&child) {
                self.ensure_derived(level - 1, child);
            }
        }
        let derived = {
            let refs: [Option<&PalettedChunk>; 8] =
                std::array::from_fn(|i| self.levels[l - 1].get(&children[i]));
            derive_lod_chunk(&refs, self.rule.as_ref())
        };
        self.levels[l].insert(pos, derived);
        self.dirty[l].remove(&pos);
        self.derive_count += 1;
    }

    /// Inserted chunks in a level-L position's subtree (0 = unknown position).
    pub fn coverage_count(&self, level: u8, pos: ChunkPos) -> u32 {
        self.coverage[usize::from(level)]
            .get(&pos)
            .copied()
            .unwrap_or(0)
    }

    /// Whether every level-0 chunk of this position's subtree was inserted —
    /// the A-5 guard for consumers that must not read a derived chunk's
    /// missing-children air as knowledge (docs/design/octree-substrate.md § 3:
    /// "ungenerated is not empty"). A derived chunk over a *partial* subtree is
    /// a legitimate summary of what was inserted, but its silence about the
    /// missing siblings is not "air"; FF2b's far composition only substitutes
    /// reduced data where this holds, synthesizing everywhere else.
    ///
    /// Exact when insertions are full-resolution ([`Self::insert_chunk`]);
    /// data inserted directly at a higher level ([`Self::insert_lod_chunk`])
    /// counts as one, so mixed-level insertion under-reports coverage
    /// (conservative: never claims fullness falsely).
    pub fn subtree_fully_inserted(&self, level: u8, pos: ChunkPos) -> bool {
        let full = 1u64 << (3 * u32::from(level));
        u64::from(self.coverage_count(level, pos)) >= full
    }

    /// Is this level-0 voxel covered by some *inserted* chunk's cube (at any
    /// level)? Derived chunks summarize inserted data but never extend
    /// knowledge, so a coarse chunk derived from a partially-loaded subtree
    /// does not make its unloaded children "known".
    pub fn is_known(&self, vx: i64, vy: i64, vz: i64) -> bool {
        for level in 0..=self.max_level {
            let shift = crate::chunk::CHUNK_BITS + u32::from(level);
            let pos = ChunkPos::new(
                (vx >> shift) as i32,
                (vy >> shift) as i32,
                (vz >> shift) as i32,
            );
            if self.inserted[usize::from(level)].contains(&pos) {
                return true;
            }
        }
        false
    }

    /// Finest known data covering a level-0 voxel: `(level, block)`, where the
    /// block is read at that level's resolution. `None` when no inserted
    /// chunk's cube covers the voxel (see [`LodPyramid::is_known`]).
    pub fn best_block(&mut self, vx: i64, vy: i64, vz: i64) -> Option<(u8, Block)> {
        if !self.is_known(vx, vy, vz) {
            return None;
        }
        for level in 0..=self.max_level {
            let (cx, cy, cz) = (vx >> level, vy >> level, vz >> level);
            let pos = ChunkPos::from_world_voxel(cx, cy, cz);
            if self.coverage[usize::from(level)].contains_key(&pos)
                && let Some(chunk) = self.get_or_derive(level, pos)
            {
                let (lx, ly, lz) = crate::chunk::local_voxel(cx, cy, cz);
                return Some((level, chunk.get(lx, ly, lz)));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_chunk(block: Block) -> PalettedChunk {
        PalettedChunk::uniform(block)
    }

    #[test]
    fn majority_rule_thresholds() {
        let rule = MajorityNonAir;
        // 3 of 8 solid -> air.
        let mut cell = [Block::Air; 8];
        cell[0] = Block::Stone;
        cell[1] = Block::Stone;
        cell[2] = Block::Stone;
        assert_eq!(rule.reduce(&cell), Block::Air);
        // 4 of 8 solid -> solid (ties go to non-air).
        cell[3] = Block::Stone;
        assert_eq!(rule.reduce(&cell), Block::Stone);
        // Most frequent non-air wins.
        let cell = [
            Block::Stone,
            Block::Stone,
            Block::Stone,
            Block::Grass,
            Block::Grass,
            Block::Air,
            Block::Air,
            Block::Stone,
        ];
        assert_eq!(rule.reduce(&cell), Block::Stone);
        // Frequency tie between blocks -> higher id (grass over dirt).
        let cell = [
            Block::Dirt,
            Block::Dirt,
            Block::Grass,
            Block::Grass,
            Block::Air,
            Block::Air,
            Block::Air,
            Block::Air,
        ];
        assert_eq!(rule.reduce(&cell), Block::Grass);
    }

    #[test]
    fn derive_places_children_in_correct_octants() {
        // One full-stone child at octant (dx=1, dy=0, dz=0).
        let stone = full_chunk(Block::Stone);
        let mut children: [Option<&PalettedChunk>; 8] = [None; 8];
        children[1] = Some(&stone); // dx | dz<<1 | dy<<2 with dx=1
        let parent = derive_lod_chunk(&children, &MajorityNonAir);
        // Occupies x in [16,32), y in [0,16), z in [0,16).
        assert_eq!(parent.get(16, 0, 0), Block::Stone);
        assert_eq!(parent.get(31, 15, 15), Block::Stone);
        assert_eq!(parent.get(15, 0, 0), Block::Air);
        assert_eq!(parent.get(16, 16, 0), Block::Air);
        assert_eq!(parent.get(16, 0, 16), Block::Air);
    }

    #[test]
    fn derive_hand_built_fixture() {
        // Child 0: a 2-voxel-thick solid floor at y=0..2 -> every parent cell
        // covering y=0 has 8/8 solid; parent floor is 1 voxel at y=0.
        let mut dense = Chunk::new();
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                dense.set(x, 0, z, Block::Stone);
                dense.set(x, 1, z, Block::Stone);
            }
        }
        // Plus a 1-voxel pillar at (4,2..4,4): each parent cell sees only
        // 2 of 8 solid -> vanishes under the majority rule.
        dense.set(4, 2, 4, Block::Wood);
        dense.set(4, 3, 4, Block::Wood);
        let child = PalettedChunk::from_dense(&dense);
        let mut children: [Option<&PalettedChunk>; 8] = [None; 8];
        children[0] = Some(&child);
        let parent = derive_lod_chunk(&children, &MajorityNonAir);
        for z in 0..16 {
            for x in 0..16 {
                assert_eq!(parent.get(x, 0, z), Block::Stone, "floor at ({x},0,{z})");
                assert_eq!(
                    parent.get(x, 1, z),
                    Block::Air,
                    "above floor at ({x},1,{z})"
                );
            }
        }
        assert_eq!(parent.get(2, 1, 2), Block::Air, "thin pillar dissolves");
    }

    #[test]
    fn grass_surface_survives_via_priority() {
        // 1-voxel grass on dirt: the cell covering the surface has 4 grass +
        // 4 dirt... build: y=0..3 dirt, y=3 grass (so cells y0: dirt x8 -> dirt;
        // cell y1: 2 layers: y=2 dirt, y=3 grass -> 4 dirt + 4 grass tie -> grass.
        let mut dense = Chunk::new();
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                for y in 0..3 {
                    dense.set(x, y, z, Block::Dirt);
                }
                dense.set(x, 3, z, Block::Grass);
            }
        }
        let child = PalettedChunk::from_dense(&dense);
        let mut children: [Option<&PalettedChunk>; 8] = [None; 8];
        children[0] = Some(&child);
        let parent = derive_lod_chunk(&children, &MajorityNonAir);
        assert_eq!(parent.get(0, 0, 0), Block::Dirt);
        assert_eq!(parent.get(0, 1, 0), Block::Grass, "surface stays grass");
        assert_eq!(parent.get(0, 2, 0), Block::Air);
    }

    #[test]
    fn pluggable_rule_changes_output() {
        struct AnyNonAir;
        impl DownsampleRule for AnyNonAir {
            fn reduce(&self, cell: &[Block; 8]) -> Block {
                cell.iter()
                    .copied()
                    .find(|b| b.is_solid())
                    .unwrap_or(Block::Air)
            }
        }
        let mut dense = Chunk::new();
        dense.set(0, 0, 0, Block::Wood); // 1 of 8 in its cell
        let child = PalettedChunk::from_dense(&dense);
        let mut children: [Option<&PalettedChunk>; 8] = [None; 8];
        children[0] = Some(&child);
        assert_eq!(
            derive_lod_chunk(&children, &MajorityNonAir).get(0, 0, 0),
            Block::Air
        );
        assert_eq!(
            derive_lod_chunk(&children, &AnyNonAir).get(0, 0, 0),
            Block::Wood
        );
    }

    #[test]
    fn levels_zero_through_four_derive() {
        let mut pyramid = LodPyramid::default();
        // A full-stone chunk at level 0, origin.
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), full_chunk(Block::Stone));
        for level in 1..=MAX_LOD_LEVEL {
            let chunk = pyramid
                .get_or_derive(level, ChunkPos::new(0, 0, 0))
                .unwrap_or_else(|| panic!("level {level} must derive"))
                .clone();
            let extent = CHUNK_SIZE_USIZE >> level; // solid cube shrinks by 2 each level
            assert_eq!(chunk.get(0, 0, 0), Block::Stone, "level {level}");
            assert_eq!(chunk.get(extent - 1, extent - 1, extent - 1), Block::Stone);
            assert_eq!(chunk.get(extent, 0, 0), Block::Air);
        }
        // Nothing is known away from the data.
        assert!(pyramid.get_or_derive(1, ChunkPos::new(5, 5, 5)).is_none());
    }

    #[test]
    fn derivation_is_incremental() {
        let mut pyramid = LodPyramid::default();
        for pos in child_positions(ChunkPos::new(0, 0, 0)) {
            pyramid.insert_chunk(pos, full_chunk(Block::Stone));
        }
        // Also a second parent's worth of children at (2..4)^3.
        for pos in child_positions(ChunkPos::new(1, 1, 1)) {
            pyramid.insert_chunk(pos, full_chunk(Block::Dirt));
        }
        pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0));
        pyramid.get_or_derive(1, ChunkPos::new(1, 1, 1));
        pyramid.get_or_derive(2, ChunkPos::new(0, 0, 0));
        let base = pyramid.derive_count();
        assert_eq!(base, 3);

        // Re-query: fully cached, zero derivations.
        pyramid.get_or_derive(2, ChunkPos::new(0, 0, 0));
        pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0));
        assert_eq!(pyramid.derive_count(), base);

        // Change ONE level-0 chunk: exactly one re-derive per affected level.
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), full_chunk(Block::Wood));
        pyramid.get_or_derive(2, ChunkPos::new(0, 0, 0));
        assert_eq!(
            pyramid.derive_count(),
            base + 2,
            "one L1 chunk + one L2 chunk re-derive; the (1,1,1) subtree is untouched"
        );
        // And the re-derived data reflects the edit.
        let l1 = pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0)).unwrap();
        assert_eq!(l1.get(0, 0, 0), Block::Wood);
        assert_eq!(l1.get(15, 15, 15), Block::Wood);
        assert_eq!(l1.get(16, 0, 0), Block::Stone, "sibling data unchanged");
    }

    #[test]
    fn derivation_is_deterministic() {
        let build = || {
            let mut pyramid = LodPyramid::default();
            let mut dense = Chunk::new();
            for i in 0..CHUNK_SIZE_USIZE {
                dense.set(i, i % 7, (i * 3) % 32, Block::Stone);
                for z in 0..CHUNK_SIZE_USIZE {
                    for x in 0..CHUNK_SIZE_USIZE {
                        dense.set(x, 0, z, Block::Grass);
                    }
                }
            }
            pyramid.insert_chunk(ChunkPos::new(0, 0, 0), PalettedChunk::from_dense(&dense));
            pyramid.insert_chunk(ChunkPos::new(1, 0, 1), full_chunk(Block::Stone));
            pyramid
                .get_or_derive(2, ChunkPos::new(0, 0, 0))
                .unwrap()
                .clone()
        };
        assert_eq!(build(), build());
    }

    #[test]
    fn remove_chunk_vacates_and_rederives() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), full_chunk(Block::Stone));
        pyramid.insert_chunk(ChunkPos::new(1, 0, 0), full_chunk(Block::Stone));
        let l1 = pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0)).unwrap();
        assert_eq!(l1.get(0, 0, 0), Block::Stone);
        assert_eq!(l1.get(16, 0, 0), Block::Stone);

        pyramid.remove_chunk(ChunkPos::new(1, 0, 0));
        let l1 = pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0)).unwrap();
        assert_eq!(l1.get(0, 0, 0), Block::Stone);
        assert_eq!(l1.get(16, 0, 0), Block::Air, "removed subtree reads as air");

        pyramid.remove_chunk(ChunkPos::new(0, 0, 0));
        assert!(
            pyramid.get_or_derive(1, ChunkPos::new(0, 0, 0)).is_none(),
            "fully vacated subtree is unknown again"
        );
    }

    #[test]
    fn direct_lod_insert_supports_higher_levels_without_children() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_lod_chunk(2, ChunkPos::new(3, -1, 0), full_chunk(Block::Stone));
        assert!(pyramid.get_or_derive(0, ChunkPos::new(12, -4, 0)).is_none());
        assert!(
            pyramid.get_or_derive(2, ChunkPos::new(3, -1, 0)).is_some(),
            "directly inserted LOD data is served"
        );
        // And it feeds derivation of the level above.
        let l3 = pyramid
            .get_or_derive(3, ChunkPos::new(1, -1, 0))
            .expect("derives from L2");
        assert_eq!(l3.is_uniform(), None);
    }

    #[test]
    fn subtree_fully_inserted_requires_every_child() {
        let mut pyramid = LodPyramid::default();
        let parent = ChunkPos::new(0, 0, 0);
        let children = child_positions(parent);
        for (i, pos) in children.iter().enumerate() {
            assert!(
                !pyramid.subtree_fully_inserted(1, parent),
                "partial subtree ({i} of 8) must not claim fullness (A-5)"
            );
            pyramid.insert_chunk(*pos, full_chunk(Block::Stone));
        }
        assert!(pyramid.subtree_fully_inserted(1, parent));
        assert_eq!(pyramid.coverage_count(1, parent), 8);
        // Level 2 above it holds 8 of 64: known, not full.
        assert!(pyramid.coverage_count(2, ChunkPos::new(0, 0, 0)) == 8);
        assert!(!pyramid.subtree_fully_inserted(2, ChunkPos::new(0, 0, 0)));
        // Removal reopens the guard.
        pyramid.remove_chunk(children[3]);
        assert!(!pyramid.subtree_fully_inserted(1, parent));
        // A direct higher-level insert counts as one — conservative, never full
        // (documented under-report for mixed-level insertion).
        let mut direct = LodPyramid::default();
        direct.insert_lod_chunk(1, parent, full_chunk(Block::Stone));
        assert_eq!(direct.coverage_count(1, parent), 1);
        assert!(!direct.subtree_fully_inserted(1, parent));
    }

    #[test]
    fn best_block_prefers_finest_available() {
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(0, 0, 0), full_chunk(Block::Stone));
        pyramid.insert_lod_chunk(1, ChunkPos::new(4, 0, 0), full_chunk(Block::Dirt));
        // Full-res where level 0 exists.
        assert_eq!(pyramid.best_block(5, 5, 5), Some((0, Block::Stone)));
        // Coarse where only L1 exists: level-0 voxel (256..320, 0..64, 0..64).
        assert_eq!(pyramid.best_block(300, 10, 10), Some((1, Block::Dirt)));
        // Unknown elsewhere.
        assert_eq!(pyramid.best_block(-1000, 0, 0), None);
    }

    #[test]
    fn negative_positions_parent_correctly() {
        assert_eq!(
            parent_pos(ChunkPos::new(-1, -2, 1)),
            ChunkPos::new(-1, -1, 0)
        );
        assert_eq!(
            ancestor_pos(ChunkPos::new(-33, 33, -1), 2),
            ChunkPos::new(-9, 8, -1)
        );
        let mut pyramid = LodPyramid::default();
        pyramid.insert_chunk(ChunkPos::new(-1, -1, -1), full_chunk(Block::Stone));
        let l1 = pyramid
            .get_or_derive(1, ChunkPos::new(-1, -1, -1))
            .expect("known");
        // The (-1,-1,-1) level-0 chunk is the (1,1,1) octant of its parent.
        assert_eq!(l1.get(31, 31, 31), Block::Stone);
        assert_eq!(l1.get(0, 0, 0), Block::Air);
    }
}
