//! Hierarchical lazy world generation and deep-time history (spike S7).
//!
//! "Theoretically infinite, globally aware" is resolved by hierarchy with
//! bounded neighborhoods (docs/design/worldgen.md): a finite coarse world is
//! **pregenerated** — tectonics, climate, drainage, and the deep-time erosional
//! history — and everything below the region scale **collapses lazily on
//! approach** under one rule: collapsed(cell) = f(base(cell), 1-ring
//! neighbour base summary, collapsed(parent)). Rivers are planned at
//! region-graph scale before any chunk materializes; no level ever needs
//! unbounded lookahead (instrumented and asserted per chunk).
//!
//! Level pyramid and per-level resolution (N=2 scale, 0.9 m voxels):
//!
//! | level        | edge length        | provenance                        |
//! |--------------|--------------------|-----------------------------------|
//! | coarse cell  | 14.7456 km         | pregenerated (finite grid)        |
//! | region       | 7.3728 km          | lazy                              |
//! | locale       | 460.8 m            | lazy                              |
//! | chunk-column | 28.8 m (footprint) | lazy                              |
//! | chunk        | 28.8 m cube        | lazy (dc-core `Chunk` of blocks)  |
//!
//! **Deep-time history means the EROSIONAL history** ([`deeptime`]): the
//! two-plane tectonics/erosion/weathering sim over the coarse grid, whose
//! eroded surface and per-cell strata record drive the collapse tier.
//! *It used to mean something else too.* A thin settlement / expansion /
//! conflict sim once ran here over 12 pre-player epochs, committing site and
//! polity facts into dc-sim's S2 constraint ledger and expressing them as wood
//! ruin posts; it was **removed 2026-07-28** (journal/0121) as unratified
//! early-bootstrap fabrication with no evo/socia/civ design behind it.
//! There is no settlement, culture or civilization modelling in this crate,
//! and the absence is deliberate rather than pending.
//!
//! The world is bounded in extent (a player knob, [`Extent`]) but its borders
//! are unbounded hostile wilds: beyond the pregen grid the same lazy pyramid
//! runs on synthesized coarse cells forever — abyssal ocean, polar ice.
//!
//! Rules: headless, deterministic (all entropy flows from the world seed).

pub mod collapse;
pub mod deeptime;
pub mod draws;
pub mod far;
pub mod fill;
pub mod geology;
pub mod passgraph;
pub mod pipeline;
pub mod pregen;
pub mod subcell;
pub mod water;

pub use collapse::{
    ChunkStats, ColumnRec, LOOKAHEAD_BOUNDS, LookaheadBounds, PregenSource, WorldGenerator,
};
pub use deeptime::DeepOverrides;
pub use fill::{ColumnFill, Plan};
pub use geology::{AlluviumRec, StrataCtx, StrataEvent, StrataRec};
pub use pipeline::{Pass, PassBody, Phase, Pipeline, PipelineError, PregenCtx, Resource};
pub use pregen::{
    CELL_CHUNKS, CELL_VOXELS, Cell, CellGrid, CellView, Extent, Pregen, Provenance, WorldParams,
};
pub use subcell::SubCell;

pub const CRATE_ROLE: &str = "hierarchical lazy worldgen + deep-time history";
