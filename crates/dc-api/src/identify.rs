//! **`identify(pos)` — the honest identity surface.** One position-addressed
//! answer to *"what is this voxel made of"*, which is allowed to say **"I have
//! no record"** (journal/0101, corrections #49).
//!
//! ## Why this exists
//!
//! `dc:world/get_contents` used to answer a **per-voxel** question from a
//! **per-chunk** presence test. `HostWorld::contents_at` returns `Some(grid)`
//! whenever *any* voxel in the 32³ chunk carries a record, and inside that grid
//! an **unrecorded** voxel resolves to a perfectly ordinary
//! [`VoxelContents::EMPTY`]. So the unrecorded basement under a thin record
//! reported `has_contents: true` (contradicting its own docs) and
//! `classified: dc:air` (the one operation `dc_core::classify`'s module docs
//! forbid) — while sitting on correctly-solid `dc:stone`. Measured: **6.4 % of
//! near-surface solid voxels, globally**. `Option::None`, the only channel that
//! could have carried *"no record here"*, had already been spent on a
//! whole-chunk condition inherited from the mesher's needs.
//!
//! That is *"a summary is not an authority"* **inverted**: a **chunk-level**
//! summary (`has_contents`) worn as a **voxel-level** authority.
//!
//! ## The shape
//!
//! - **UNTIERED** (DECIDED 2026-07-25, user). *"What is at world position X"*
//!   is a **world** question with one true answer, to which viewer distance is
//!   irrelevant. The earlier Near/Mid/Far design conflated it with *"what is
//!   the renderer showing"* — a **render** question, and the only one with any
//!   business reading a LOD ladder. Gen is pure-of-position, so the honest
//!   answer is always derivable: cost may change the *policy or the latency*,
//!   never the **answer**.
//! - **The payload is uniformly a mixture.** A single dominant material is just
//!   a one-component mixture; there is no "winner" case and no far tier to
//!   special-case, so the day the far field goes speckled nothing migrates.
//! - **[`Identity::Unrecorded`] is a first-class answer**, distinct from both
//!   *"air"* and *"recorded"* — and it is distinct **in the type**, not by
//!   convention. An empty mixture is a *positive* statement ("there is nothing
//!   in this voxel"); no record is the absence of any statement at all.
//!
//! ## How the two are told apart (the enabler)
//!
//! The stored `Block` already disambiguates, so nothing in worldgen had to
//! change. Unrecorded basement is `Block::Stone` **by construction**
//! (`dc-worldgen/src/collapse.rs`), and **no voxel below a column's height can
//! ever be `Block::Air`** — the only `Air` branch is `vy > h`. Therefore, when
//! the composition record is empty or absent:
//!
//! | stored block | verdict |
//! |---|---|
//! | `Block::Air` | genuinely empty — [`Identity::Mixture`] of [`VoxelContents::EMPTY`] |
//! | anything else | [`Identity::Unrecorded`] |
//!
//! A **non-empty** record always wins outright, whatever the block says: that
//! divergence (`block: dc:air` over recorded granite) is the legible signal
//! that the voxel was *edited*, and it stays legible.
//!
//! ## What this surface does NOT do yet
//!
//! It is **edit-blind for composition**: the contents source is a pure function
//! of position (worldgen re-derivation), so an edit writes a `Block` and the
//! mixture underneath it does not move. `identify` reports the honest *stored*
//! block beside the honest *derived* mixture and lets the divergence show; it
//! cannot yet report the mixture an edit produced. Closing that is the arc's
//! **runtime edit-fact overlay** (ROADMAP continuation slot), not this slice.

use dc_core::{Block, VoxelContents};

/// What the world knows about one voxel's composition.
///
/// Three values live in two variants, and the distinction the defect destroyed
/// is the one between the first two:
///
/// 1. `Unrecorded` — **no composition record backs this voxel.** The stored
///    block is all that is known. This is normal and not a defect: the
///    unrecorded basement below the deep-time record, the legacy soil band,
///    ocean floor, the border wilds, ruin posts, and the whole S1 terrain
///    authority all live here.
/// 2. `Mixture(EMPTY)` — **a record, and it says nothing is here.** Open air.
/// 3. `Mixture(c)` — the honest full composition.
#[derive(Clone, PartialEq, Debug)]
pub enum Identity {
    /// The world has no composition record for this voxel. Not "empty", not
    /// "air": the absence of a statement, not a statement of absence.
    Unrecorded,
    /// The honest full mixture at this voxel — up to 8 partials across
    /// structure / pore-fill / debris, never one arbitrary component.
    ///
    /// [`VoxelContents::EMPTY`] here is a *positive* answer ("nothing is in
    /// this voxel"), and is a **different value** from [`Identity::Unrecorded`].
    Mixture(VoxelContents),
}

impl Identity {
    /// Decide an identity from a voxel's stored block and whatever the
    /// composition source had for it — the single place the enabler above is
    /// applied, so every consumer inherits the same verdict.
    ///
    /// `record` is `None` when the source had nothing for the voxel's *chunk*
    /// (or no source is installed at all), and `Some(EMPTY)` when the chunk had
    /// a grid but this voxel was never written — the two are the same fact at
    /// voxel resolution, and both are treated as "no record".
    pub fn resolve(block: Block, record: Option<VoxelContents>) -> Self {
        match record {
            Some(c) if !c.is_empty() => Identity::Mixture(c),
            // No record — or a record that records nothing, which at voxel
            // resolution is the same thing. Only the block can tell genuine
            // emptiness from an absent record.
            _ if block == Block::Air => Identity::Mixture(VoxelContents::EMPTY),
            _ => Identity::Unrecorded,
        }
    }

    /// True when no composition record backs this voxel.
    pub fn is_unrecorded(&self) -> bool {
        matches!(self, Identity::Unrecorded)
    }

    /// The mixture, or `None` when unrecorded. **`Some(EMPTY)` and `None` are
    /// different answers** — do not `unwrap_or_default()` them together.
    pub fn mixture(&self) -> Option<&VoxelContents> {
        match self {
            Identity::Unrecorded => None,
            Identity::Mixture(c) => Some(c),
        }
    }

    /// The block this identity classifies to, or `None` when unrecorded —
    /// `classify` is a *derived* answer and must never be applied to a voxel
    /// that was never given a record (`dc_core::classify` module docs, the
    /// absent-contents rule). Callers echo the stored block instead.
    pub fn classified(&self) -> Option<Block> {
        self.mixture().map(dc_core::classify)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::{MaterialId, StructureShape};

    fn granite() -> VoxelContents {
        VoxelContents::new(StructureShape::Full, &[MaterialId::GRANITE; 8], &[], &[]).unwrap()
    }

    #[test]
    fn empty_record_under_a_solid_block_is_unrecorded_not_air() {
        // The corrections #49 voxel: unrecorded basement sharing a chunk with a
        // recorded voxel, so the grid exists and hands back an ordinary EMPTY.
        let id = Identity::resolve(Block::Stone, Some(VoxelContents::EMPTY));
        assert_eq!(id, Identity::Unrecorded);
        assert!(id.mixture().is_none());
        assert_eq!(
            id.classified(),
            None,
            "classify must not run on a non-record"
        );
    }

    #[test]
    fn absent_grid_under_a_solid_block_is_unrecorded() {
        assert_eq!(Identity::resolve(Block::Stone, None), Identity::Unrecorded);
    }

    #[test]
    fn air_is_a_recorded_empty_mixture_not_unrecorded() {
        // Sky must still be air: an empty mixture is a positive statement.
        for record in [None, Some(VoxelContents::EMPTY)] {
            let id = Identity::resolve(Block::Air, record);
            assert_eq!(id, Identity::Mixture(VoxelContents::EMPTY));
            assert!(!id.is_unrecorded());
            assert_eq!(id.classified(), Some(Block::Air));
        }
    }

    #[test]
    fn an_unrecorded_answer_is_not_an_empty_mixture() {
        // The distinction is in the type, not by convention.
        assert_ne!(
            Identity::Unrecorded,
            Identity::Mixture(VoxelContents::EMPTY)
        );
    }

    #[test]
    fn a_nonempty_record_wins_over_the_block_so_edits_stay_legible() {
        // Break granite to air: the stored block is authoritative, the record
        // still says granite, and the divergence is the edit signal.
        let id = Identity::resolve(Block::Air, Some(granite()));
        assert_eq!(id.classified(), Some(Block::Material(MaterialId::GRANITE)));
    }
}
