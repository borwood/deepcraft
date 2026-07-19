//! Collider bubbles: on-demand static voxel colliders around dynamic bodies.
//!
//! Strategy (the S6 collider-bubble note, measured in
//! docs/spikes/S6-results.md):
//!
//! - The voxel lattice is tiled into fixed cubes of
//!   [`BubbleConfig::tile_size_voxels`] per edge. Tiles — not per-body radii —
//!   are the unit of collider lifetime, which makes "refresh incrementally as
//!   bodies move" a set difference: as a body moves, tiles enter at its
//!   leading edge and leave at its trailing edge; the interior of its bubble
//!   is untouched. Tiles shared by several bodies' bubbles are built once and
//!   survive until *no* bubble wants them.
//! - A body's bubble is its collider AABB expanded by
//!   [`BubbleConfig::margin_m`] plus one fixed step of its velocity, converted
//!   to the covering tile range. Sleeping bodies keep their tiles: waking up
//!   mid-step (something falls on a sleeper) must find support already there.
//! - Building a tile scans its voxels through the caller's solidity closure,
//!   greedy-merges them into cuboids ([`crate::merge`]), and inserts ONE fixed
//!   compound collider per non-empty tile. Tiles that scan to empty are
//!   remembered (`collider: None`) so they are not rescanned every step —
//!   "known empty" is as valuable as "built".
//! - When a tile leaves every bubble its collider is removed. There is no
//!   other path by which voxel geometry reaches rapier.
//!
//! The tile map is a `BTreeMap` so build/remove order is deterministic —
//! collider insertion order feeds rapier's iteration order, and the
//! determinism guarantee (lib docs) depends on it.

use std::collections::BTreeMap;

use rapier3d::geometry::ColliderHandle;

/// Tuning for the collider bubbles.
#[derive(Clone, Copy, Debug)]
pub struct BubbleConfig {
    /// Edge length of one collider tile, in voxels.
    pub tile_size_voxels: i64,
    /// Extra margin around each dynamic body's AABB, in meters.
    pub margin_m: f64,
}

impl Default for BubbleConfig {
    fn default() -> Self {
        Self {
            // 4^3 = 64 voxels per tile: fine-grained enough that a resting
            // item wants ~4-8 tiles, coarse enough that tile churn while
            // moving is a handful of scans per step.
            tile_size_voxels: 4,
            margin_m: 1.0,
        }
    }
}

/// Tile coordinate on the tile lattice (voxel coordinate >> tile size).
pub(crate) type TileKey = (i64, i64, i64);

pub(crate) struct TileEntry {
    /// `None` when the tile scanned to all-air (kept as a negative cache).
    pub collider: Option<ColliderHandle>,
    /// Number of merged cuboids inside the compound (0 for empty tiles).
    pub cuboids: usize,
}

/// The set of live tiles. Owned by [`crate::PhysicsWorld`], which performs the
/// actual rapier insert/remove (they need `&mut` over several rapier sets).
#[derive(Default)]
pub(crate) struct BubbleSet {
    pub tiles: BTreeMap<TileKey, TileEntry>,
}

/// Floor-divide a voxel coordinate onto the tile lattice.
#[inline]
pub(crate) fn tile_of(v: i64, tile_size: i64) -> i64 {
    v.div_euclid(tile_size)
}
