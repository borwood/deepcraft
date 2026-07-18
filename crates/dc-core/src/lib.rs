//! Voxel data model, chunk/LOD storage, registries, and shared math.
//!
//! Rules for this crate:
//! - Headless: no rendering, no OS, no I/O beyond (de)serialization of its own types.
//! - Chunk storage is LOD-aware from v1: every chunk can yield or store downsampled
//!   representations. Distant-terrain rendering and the statistical sim tier both
//!   consume these; retrofitting them later is the failure mode we are avoiding
//!   (see docs/ARCHITECTURE.md § Rendering).
//! - Voxel scale (player height in voxels) is an open question resolved by spike S1.
//!   Nothing in this crate may hard-code a voxel:meter ratio.

/// Placeholder until S1/S3 land the real chunk format.
pub const CRATE_ROLE: &str = "voxel data model";
