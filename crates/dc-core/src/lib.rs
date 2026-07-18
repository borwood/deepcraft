//! Voxel data model, chunk/LOD storage, registries, and shared math.
//!
//! Rules for this crate:
//! - Headless: no rendering, no OS, no I/O beyond (de)serialization of its own types.
//! - Chunk storage is LOD-aware from v1: every chunk can yield or store downsampled
//!   representations. Distant-terrain rendering and the statistical sim tier both
//!   consume these; retrofitting them later is the failure mode we are avoiding
//!   (see docs/ARCHITECTURE.md § Rendering). S1 shipped the dense form;
//!   S3 adds palette compression ([`palette`]), the versioned serialization
//!   container ([`format`]), the octree LOD pyramid ([`lod`]), and per-column
//!   summaries ([`column`]). See docs/spikes/S3-results.md for the format spec.
//! - Voxel scale (player height in voxels) is an open question resolved by spike S1.
//!   Nothing in this crate hard-codes a voxel:meter ratio — everything goes
//!   through [`VoxelScale`].

pub mod chunk;
pub mod collision;
pub mod format;
pub mod palette;
pub mod scale;
pub mod voxel;

pub use chunk::{CHUNK_SIZE, CHUNK_SIZE_USIZE, CHUNK_VOLUME, Chunk, ChunkPos, local_voxel};
pub use collision::{Aabb, MoveResult, VoxelQuery, move_aabb};
pub use format::{ChunkContainer, FORMAT_VERSION, FormatError, Sidecar};
pub use palette::{PackedIndices, PaletteError, PalettedChunk, bits_for_palette_len};
pub use scale::VoxelScale;
pub use voxel::Block;
