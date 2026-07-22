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
pub mod classify;
pub mod collision;
pub mod column;
pub mod farfield;
pub mod format;
pub mod lod;
pub mod materials;
pub mod palette;
pub mod raycast;
pub mod scale;
pub mod voxel;

pub use chunk::{CHUNK_SIZE, CHUNK_SIZE_USIZE, CHUNK_VOLUME, Chunk, ChunkPos, local_voxel};
pub use classify::{block_twin, classify, dominant_material};
pub use collision::{
    Aabb, MoveResult, VoxelQuery, aabb_overlaps_solid, column_top_solid_y, move_aabb,
};
pub use column::{ColumnInfo, ColumnSummaries, LodBlockSource, column_summary, open_air_below};
pub use farfield::{
    ColumnSpan, FAR_BOTTOM_UNBOUNDED, compose_column, level_stride, node_column_spans, quantize_top,
};
pub use format::{ChunkContainer, FORMAT_VERSION, FormatError, Sidecar};
pub use lod::{
    DownsampleRule, LodPyramid, MAX_LOD_LEVEL, MajorityNonAir, ancestor_pos, child_positions,
    derive_lod_chunk, parent_pos,
};
pub use materials::contents::{
    ContentsError, SOLID_EIGHTHS, StructureShape, VOXEL_EIGHTHS, VoxelContents,
};
pub use materials::extract::{ExtractionYield, extraction_sequence};
pub use materials::geology::{
    FormationContext, FormationWindow, GeoClass, GeoHabit, GeoMemberDef, GeoMemberIdx,
    GeologyError, GeologySet, GeologySetBuilder,
};
pub use materials::intern::{
    ContentsGrid, MATERIALS_SIDECAR_NAME, MIXTURES_SIDECAR_NAME, MaterialChunk, MaterialChunkError,
    MixtureId, MixtureTable, TableError,
};
pub use materials::lod::{
    DominantClassDebrisAware, MixtureDownsampleRule, derive_material_lod_chunk,
};
pub use materials::stratify::{
    Band, STRATIFY_FULL_TIME, STRATIFY_HALF_TIME, StratifiedView, stratify,
};
pub use materials::{DamageType, MATERIAL_COUNT, MaterialId, MaterialProps};
pub use palette::{PackedIndices, PaletteError, PalettedChunk, bits_for_palette_len};
pub use raycast::{RaycastHit, raycast_voxels};
pub use scale::VoxelScale;
pub use voxel::Block;
