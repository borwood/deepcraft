//! The client-side far pyramid: the reduce-upward half of the octree node
//! contract (docs/design/octree-substrate.md § 3), FF2b-minimal.
//!
//! Every chunk the near streamer generates is fed here (blocks into the S3
//! [`LodPyramid`], contents interned into a render-local material store), so
//! far tiles over *played* regions can substitute real reduced geometry —
//! chasms, overhangs, whatever the authority actually holds — for the
//! synthesized top sheet. This is a **fluid derived cache** (S-2): everything
//! in it re-derives from the authority, eviction can never change an answer,
//! and it is render-only (never sim state, receipts, or replay — the
//! MixtureTable interned here is local to this resource and its ids never
//! escape: spans leave as plain [`ColumnSpan`]s).
//!
//! **A-5 guard:** reduced data is served only for nodes whose full-res
//! subtree is completely inserted ([`LodPyramid::subtree_fully_inserted`]) —
//! a derived chunk over a partial subtree reads missing children as air,
//! which must never reach the far mesh as "nothing there". Partial subtrees
//! synthesize instead (the composition in `farmesh`).
//!
//! Where the runtime pyramid lives is the contract's open question 1: client
//! side today, host-owned once persistence lands. This resource is that
//! "today" — deliberately nothing but plain dc-core machinery plus bookkeeping.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use bevy::prelude::Resource;
use dc_core::farfield::{ColumnSpan, node_column_spans};
use dc_core::lod::{LodPyramid, MAX_LOD_LEVEL, ancestor_pos, child_positions};
use dc_core::materials::lod::DominantClassDebrisAware;
use dc_core::{
    CHUNK_VOLUME, Chunk, ChunkPos, ContentsGrid, MaterialChunk, MixtureId, MixtureTable,
    PalettedChunk, derive_material_lod_chunk,
};

/// Cap on full-resolution chunks retained for far reduction (the S-1 knob:
/// where there's a range there's a knob). At ~2–8 KiB per palette-compressed
/// chunk this bounds the pyramid's L0 tier to a few tens of MiB; beyond it the
/// farthest chunks from the viewer are dropped and their far columns fall back
/// to synthesis — a fidelity trade, never a correctness one (S-2: everything
/// here re-derives). Eviction policy tuning is octree-substrate.md open
/// question 2; this is the conservative first number, measured in the 0070
/// tripwire.
pub const FAR_PYRAMID_L0_BUDGET: usize = 8192;

/// The fully-inserted nodes over one plan column: `(node_y, span grid)` pairs,
/// grids as [`node_column_spans`] produced them (shared, derived once).
pub type NodeGrids = Vec<(i32, Arc<Vec<Vec<ColumnSpan>>>)>;

/// The far-field reduction store: block pyramid + material store + derived
/// span-grid cache.
#[derive(Resource)]
pub struct FarPyramid {
    /// The S3 block pyramid (existing machinery, `MajorityNonAir`).
    blocks: LodPyramid,
    /// Render-local mixture intern table for the material pyramid
    /// (`MixtureDownsampleRule` path). Ids never leave this resource.
    table: MixtureTable,
    /// Material chunks: level-0 inserted from streamed [`ContentsGrid`]s,
    /// higher levels lazily derived via [`derive_material_lod_chunk`].
    /// `None` = genuinely empty (a debris-free chunk), distinct from absent.
    mats: HashMap<(u8, ChunkPos), Option<MaterialChunk>>,
    /// Derived per-node span grids ([`node_column_spans`] output), invalidated
    /// along the ancestor path on insert/evict exactly like the pyramid's own
    /// dirt.
    span_cache: HashMap<(u8, ChunkPos), Arc<Vec<Vec<ColumnSpan>>>>,
    /// Level-0 chunk Ys by plan column, for the far tile builder's "which
    /// nodes exist over this column" query.
    l0_columns: HashMap<(i32, i32), BTreeSet<i32>>,
    /// Insertion bookkeeping for the eviction budget.
    l0_present: BTreeSet<(i32, i32, i32)>,
}

impl Default for FarPyramid {
    fn default() -> Self {
        Self {
            blocks: LodPyramid::default(),
            table: MixtureTable::new(),
            mats: HashMap::new(),
            span_cache: HashMap::new(),
            l0_columns: HashMap::new(),
            l0_present: BTreeSet::new(),
        }
    }
}

impl FarPyramid {
    /// Feed one generated chunk (with its render contents, when the authority
    /// has any) into the reduction side. Ancestor derivations go stale via the
    /// pyramid's own dirt plus our mirrored cache invalidation.
    pub fn insert_l0(&mut self, pos: ChunkPos, chunk: &Chunk, contents: Option<&ContentsGrid>) {
        self.blocks
            .insert_chunk(pos, PalettedChunk::from_dense(chunk));
        let mat = contents.map(|grid| {
            let mut ids = vec![MixtureId::EMPTY; CHUNK_VOLUME];
            for y in 0..32usize {
                for z in 0..32usize {
                    for x in 0..32usize {
                        let c = grid.get(x, y, z);
                        if !c.is_empty() {
                            ids[Chunk::index(x, y, z)] = self.table.intern(c);
                        }
                    }
                }
            }
            MaterialChunk::from_dense(&ids)
        });
        self.mats.insert((0, pos), mat);
        self.invalidate_ancestors(pos);
        self.l0_columns
            .entry((pos.x, pos.z))
            .or_default()
            .insert(pos.y);
        self.l0_present.insert((pos.x, pos.y, pos.z));
    }

    /// Number of retained full-resolution chunks (the budget's subject; test
    /// and measurement surface — production only writes through
    /// [`Self::enforce_budget`]).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn l0_len(&self) -> usize {
        self.l0_present.len()
    }

    /// Enforce an L0 budget (production passes [`FAR_PYRAMID_L0_BUDGET`]) by
    /// dropping the L0 chunks farthest (in chunk-lattice distance) from the
    /// viewer's chunk. Far columns over dropped subtrees fall back to
    /// synthesis on their next rebuild.
    pub fn enforce_budget(&mut self, viewer_chunk: ChunkPos, budget: usize) {
        let over = self.l0_present.len().saturating_sub(budget);
        if over == 0 {
            return;
        }
        let mut by_dist: Vec<(i64, (i32, i32, i32))> = self
            .l0_present
            .iter()
            .map(|&(x, y, z)| {
                let (dx, dy, dz) = (
                    i64::from(x - viewer_chunk.x),
                    i64::from(y - viewer_chunk.y),
                    i64::from(z - viewer_chunk.z),
                );
                (dx * dx + dy * dy + dz * dz, (x, y, z))
            })
            .collect();
        by_dist.sort_unstable_by_key(|(d, _)| std::cmp::Reverse(*d));
        for &(_, (x, y, z)) in by_dist.iter().take(over) {
            let pos = ChunkPos::new(x, y, z);
            self.blocks.remove_chunk(pos);
            self.mats.remove(&(0, pos));
            self.invalidate_ancestors(pos);
            if let Some(set) = self.l0_columns.get_mut(&(x, z)) {
                set.remove(&y);
                if set.is_empty() {
                    self.l0_columns.remove(&(x, z));
                }
            }
            self.l0_present.remove(&(x, y, z));
        }
    }

    fn invalidate_ancestors(&mut self, l0: ChunkPos) {
        for level in 1..=MAX_LOD_LEVEL {
            let apos = ancestor_pos(l0, level);
            self.mats.remove(&(level, apos));
            self.span_cache.remove(&(level, apos));
        }
    }

    /// The fully-inserted nodes over a level-L plan column `(nx, nz)`, each
    /// with its derived span grid — what the far tile builder lays over the
    /// synthesized backdrop. Partial subtrees are absent by construction (the
    /// A-5 guard); an empty answer means "synthesize", never "air".
    pub fn known_node_grids(&mut self, level: u8, nx: i32, nz: i32) -> NodeGrids {
        if level == 0 || level > MAX_LOD_LEVEL {
            return Vec::new();
        }
        // Candidate node Ys from the L0 columns under this node column.
        let span = 1i32 << level;
        let mut nys: BTreeSet<i32> = BTreeSet::new();
        for x in (nx * span)..((nx + 1) * span) {
            for z in (nz * span)..((nz + 1) * span) {
                if let Some(ys) = self.l0_columns.get(&(x, z)) {
                    nys.extend(ys.iter().map(|y| y >> level));
                }
            }
        }
        let mut out = Vec::new();
        for ny in nys {
            let pos = ChunkPos::new(nx, ny, nz);
            if !self.blocks.subtree_fully_inserted(level, pos) {
                continue;
            }
            if let Some(grid) = self.node_grid(level, pos) {
                out.push((ny, grid));
            }
        }
        out
    }

    /// The derived span grid of one node (cached; None if the block pyramid
    /// cannot serve the node).
    fn node_grid(&mut self, level: u8, pos: ChunkPos) -> Option<Arc<Vec<Vec<ColumnSpan>>>> {
        if let Some(grid) = self.span_cache.get(&(level, pos)) {
            return Some(grid.clone());
        }
        self.ensure_material(level, pos);
        let blocks = self.blocks.get_or_derive(level, pos)?.clone();
        let mat = self.mats.get(&(level, pos)).and_then(|m| m.as_ref());
        let grid = Arc::new(node_column_spans(
            level,
            pos,
            &blocks,
            mat.map(|m| (m, &self.table)),
        ));
        self.span_cache.insert((level, pos), grid.clone());
        Some(grid)
    }

    /// Lazily derive the material chunk for a node from its children — the
    /// `MixtureDownsampleRule` pyramid, mirroring the block pyramid's octant
    /// geometry (dc-core proves they compose voxel-for-voxel).
    fn ensure_material(&mut self, level: u8, pos: ChunkPos) {
        if level == 0 || self.mats.contains_key(&(level, pos)) {
            return;
        }
        let children = child_positions(pos);
        for child in children {
            self.ensure_material(level - 1, child);
        }
        let all_empty = children
            .iter()
            .all(|c| matches!(self.mats.get(&(level - 1, *c)), None | Some(None)));
        let derived = if all_empty {
            None
        } else {
            let refs: [Option<&MaterialChunk>; 8] = std::array::from_fn(|i| {
                self.mats
                    .get(&(level - 1, children[i]))
                    .and_then(|m| m.as_ref())
            });
            Some(derive_material_lod_chunk(
                &refs,
                &mut self.table,
                &DominantClassDebrisAware,
            ))
        };
        self.mats.insert((level, pos), derived);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::Block;
    use dc_core::farfield::FAR_BOTTOM_UNBOUNDED;

    /// A flat slab world: chunks at y=0 solid up to local row 8 (world row 8).
    fn slab_chunk() -> Chunk {
        let mut c = Chunk::new();
        for z in 0..32 {
            for x in 0..32 {
                for y in 0..8 {
                    c.set(x, y, z, Block::Stone);
                }
            }
        }
        c
    }

    #[test]
    fn partial_subtrees_synthesize_rather_than_reading_air() {
        let mut p = FarPyramid::default();
        // 7 of the 8 children of L1 node (0,0,0): the A-5 case. The slab
        // lives in the lower children; the upper ones are genuinely air.
        let children = child_positions(ChunkPos::new(0, 0, 0));
        let child_chunk = |pos: ChunkPos| {
            if pos.y == 0 {
                slab_chunk()
            } else {
                Chunk::new()
            }
        };
        for pos in children.iter().take(7) {
            p.insert_l0(*pos, &child_chunk(*pos), None);
        }
        assert!(
            p.known_node_grids(1, 0, 0).is_empty(),
            "a partial subtree must not be served as knowledge"
        );
        // The eighth completes it; the node now answers with reduced spans.
        p.insert_l0(children[7], &child_chunk(children[7]), None);
        let grids = p.known_node_grids(1, 0, 0);
        assert_eq!(grids.len(), 1);
        let (ny, grid) = &grids[0];
        assert_eq!(*ny, 0);
        // Slab to world row 8 = L1 rows < 4 -> top face plane at base voxel 8.
        let col = &grid[0];
        assert_eq!(
            col,
            &vec![ColumnSpan {
                top: 8,
                bottom: 0,
                block: Block::Stone,
            }]
        );
        assert_ne!(
            col[0].bottom, FAR_BOTTOM_UNBOUNDED,
            "reduced spans are bounded"
        );
    }

    #[test]
    fn eviction_falls_back_to_synthesis_and_keeps_the_nearest() {
        let mut p = FarPyramid::default();
        // Two full L1 subtrees: one at the origin, one far away at x=100.
        for pos in child_positions(ChunkPos::new(0, 0, 0)) {
            p.insert_l0(pos, &slab_chunk(), None);
        }
        for pos in child_positions(ChunkPos::new(100, 0, 0)) {
            p.insert_l0(pos, &slab_chunk(), None);
        }
        assert_eq!(p.l0_len(), 16);
        assert_eq!(p.known_node_grids(1, 0, 0).len(), 1);
        assert_eq!(p.known_node_grids(1, 100, 0).len(), 1);
        // Budget 8 with the viewer at the origin: the distant subtree drops,
        // its node column answers empty again (synthesize, never air), the
        // near one survives intact.
        p.enforce_budget(ChunkPos::new(0, 0, 0), 8);
        assert_eq!(p.l0_len(), 8);
        assert!(p.known_node_grids(1, 100, 0).is_empty());
        assert_eq!(p.known_node_grids(1, 0, 0).len(), 1);
    }
}
