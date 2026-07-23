//! The voxel atom. **A block IS a material** (materials.md DECIDED 2026-07-22,
//! north-star step 1): the atom is `{ Air, Material(MaterialId) }` — air a peer,
//! every solid voxel *is* the material that fills it. The old parallel geology
//! block tier (`Mudstone`, `Sandstone`, …) is gone; those voxels now carry
//! `Material(MaterialId::MUDSTONE)` and derive their render identity from the
//! material registry directly (the `block_twin` re-translation the collapse
//! deleted, journal/0087).
//!
//! ## The legacy S1 tokens
//!
//! `Stone`/`Dirt`/`Grass`/`Wood` remain as **legacy S1 tokens** — the walking-
//! skeleton terrain (dc-client `TerrainGen`), the legacy soil band / unrecorded
//! basement / ocean floor / border wilds / ruin posts (dc-worldgen `collapse`),
//! and the `dc:*` edit palette still emit them. They have no material twin and
//! are the sole reason the atom is not yet the literal two-variant
//! `{ Air, Material }`; retiring them (re-pointing those paths at real worldgen)
//! is the *next* ratified slice (Crux 2). Until then they ride here as peers.
//!
//! ## One byte
//!
//! [`MaterialId`] carries a niche (its private `MatRepr` backing), so this enum —
//! one payload variant plus five unit tokens — folds into a **single byte**: a
//! 32³ chunk's raw block store drops from 64 KiB to 32 KiB.

use serde::{Deserialize, Serialize};

use crate::materials::MaterialId;

/// The voxel atom: air, a registry material, or a legacy S1 token. One byte
/// (niche-folded through [`MaterialId`]).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum Block {
    #[default]
    Air,
    /// A registry material fills this voxel — its own identity, no block-tier
    /// summary in between. The dominant material of the voxel's contents, as
    /// [`crate::classify::classify`] derives it.
    Material(MaterialId),
    /// Legacy S1 stone (walking-skeleton terrain / unrecorded basement). No
    /// material twin; retired with the legacy-S1 slice.
    Stone,
    /// Legacy S1 dirt (legacy soil band). No material twin.
    Dirt,
    /// Legacy S1 grass surface. No material twin.
    Grass,
    /// Legacy S1 wood (ruin posts). No material twin.
    Wood,
}

impl Block {
    /// Does this block participate in collision and face culling?
    #[inline]
    pub const fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }

    /// A stable total-order numeric key over every `Block` value — the successor
    /// of the old `#[repr(u16)]` discriminant, for the few consumers that need a
    /// deterministic ordinal (the LOD majority tie-break, chunk digests). `Air`
    /// is `0`, the legacy tokens keep their historical ids `1..=4`, and a
    /// material reads as `5 + id` so material order follows registry id.
    #[inline]
    pub const fn ordinal(self) -> u16 {
        match self {
            Block::Air => 0,
            Block::Stone => 1,
            Block::Dirt => 2,
            Block::Grass => 3,
            Block::Wood => 4,
            Block::Material(m) => 5 + m.raw() as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_is_not_solid_everything_else_is() {
        assert!(!Block::Air.is_solid());
        for b in [
            Block::Stone,
            Block::Dirt,
            Block::Grass,
            Block::Wood,
            Block::Material(MaterialId::MUDSTONE),
            Block::Material(MaterialId::SNOW),
        ] {
            assert!(b.is_solid());
        }
    }

    #[test]
    fn block_is_one_byte() {
        // The collapse's storage win: MaterialId's niche folds Air + the four
        // legacy tokens in beside the material payload, so the atom is one byte
        // (a 32³ chunk's raw block store is 32 KiB, was 64 KiB).
        assert_eq!(std::mem::size_of::<Block>(), 1);
    }

    #[test]
    fn ordinal_is_stable_and_orders_materials_by_id() {
        assert_eq!(Block::Air.ordinal(), 0);
        assert_eq!(Block::Stone.ordinal(), 1);
        assert_eq!(Block::Wood.ordinal(), 4);
        assert_eq!(Block::Material(MaterialId::SAND).ordinal(), 5);
        assert!(
            Block::Material(MaterialId::MUDSTONE).ordinal()
                < Block::Material(MaterialId::SANDSTONE).ordinal()
        );
    }
}
