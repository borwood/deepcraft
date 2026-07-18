//! Hierarchical lazy world generation and deep-time history (spike S7).
//!
//! "Theoretically infinite, globally aware" is resolved by hierarchy with
//! bounded neighborhoods (docs/design/worldgen.md): a finite coarse world is
//! **pregenerated** — tectonics, climate, drainage, and a thin forward-run
//! settlement history whose outcomes are committed facts in the S2 constraint
//! ledger — and everything below the region scale **collapses lazily on
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
//! Deep-time history is dc-sim's statistical tier run over pre-player epochs:
//! the history pass collapses events through `engine::observe` and the live
//! sim keeps querying the same overlay world and ledger after year zero —
//! worldgen history and live far-simulation are one system, not two.
//!
//! The world is bounded in extent (a player knob, [`Extent`]) but its borders
//! are unbounded hostile wilds: beyond the pregen grid the same lazy pyramid
//! runs on synthesized coarse cells forever — abyssal ocean, polar ice, no
//! history layer.
//!
//! Rules: headless, deterministic (all entropy flows from the world seed).

pub mod collapse;
pub mod pregen;

pub use collapse::{ChunkStats, LOOKAHEAD_BOUNDS, LookaheadBounds, WorldGenerator};
pub use pregen::{
    CELL_CHUNKS, CELL_VOXELS, Cell, CellGrid, CellView, Extent, Pregen, Provenance, SiteSummary,
    WorldParams, YEAR_ZERO_TICK,
};

pub const CRATE_ROLE: &str = "hierarchical lazy worldgen + deep-time history";
