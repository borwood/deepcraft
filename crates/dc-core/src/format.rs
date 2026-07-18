//! Chunk serialization container, format v1 (S3).
//!
//! Wire layout (postcard encoding, varints throughout):
//!
//! ```text
//! [version: u16]                 -- decoded FIRST, before anything else
//! [voxels:  PalettedChunk]       -- palette + bit-packed indices (see palette.rs)
//! [sidecars: seq of Sidecar]     -- named opaque byte sections
//! ```
//!
//! **Version field.** `decode` reads the leading version varint and refuses
//! anything other than [`FORMAT_VERSION`] before touching the payload, so a
//! future v2 can change everything after the version byte without confusing a
//! v1 reader (it errors cleanly) and a v2 reader can dispatch per version.
//!
//! **Sidecar sections — the growth hook.** A [`Sidecar`] is `(name, bytes)`;
//! the container carries any number of them and assigns no meaning to either
//! field. This is how format v1 grows without a breaking rewrite: a richer
//! per-voxel material model (mixed granular materials, up to 8 material slots
//! per voxel) is under design and will ship as sidecar sections (e.g.
//! `"materials/slots-v0"`) alongside the v1 voxel payload rather than as
//! format v2. Rules:
//!
//! - Readers MUST ignore sidecars whose name they don't recognize (asserted by
//!   test): unknown sections are carried, not errors.
//! - Rewriters MUST preserve unrecognized sidecars byte-for-byte (they ride
//!   along in the struct, so encode-after-decode does this automatically).
//! - Names are namespaced by convention (`"area/thing-vN"`); a sidecar's own
//!   payload versioning lives in its name.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::palette::{PaletteError, PalettedChunk};

/// Current chunk container format version.
pub const FORMAT_VERSION: u16 = 1;

/// A named opaque byte section riding alongside the voxel payload. See module
/// docs for the growth rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sidecar {
    pub name: String,
    pub data: Vec<u8>,
}

/// Errors from decoding a chunk container.
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("unsupported chunk format version {0} (this build reads version {FORMAT_VERSION})")]
    UnsupportedVersion(u16),
    #[error("malformed chunk container: {0}")]
    Malformed(#[from] postcard::Error),
    #[error("invalid voxel payload: {0}")]
    InvalidPayload(#[from] PaletteError),
}

/// The format-v1 chunk container: versioned voxel payload plus optional named
/// sidecar sections.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChunkContainer {
    pub voxels: PalettedChunk,
    pub sidecars: Vec<Sidecar>,
}

impl ChunkContainer {
    pub fn new(voxels: PalettedChunk) -> Self {
        Self {
            voxels,
            sidecars: Vec::new(),
        }
    }

    /// Serialize as format v1 (version varint, then payload).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = postcard::to_allocvec(&FORMAT_VERSION).expect("varint encode cannot fail");
        out.extend(postcard::to_allocvec(self).expect("in-memory encode cannot fail"));
        out
    }

    /// Deserialize, checking the version field first and structurally
    /// validating the voxel payload. Unknown sidecar sections are preserved,
    /// never an error.
    pub fn decode(bytes: &[u8]) -> Result<Self, FormatError> {
        let (version, rest) = postcard::take_from_bytes::<u16>(bytes)?;
        if version != FORMAT_VERSION {
            return Err(FormatError::UnsupportedVersion(version));
        }
        let container: ChunkContainer = postcard::from_bytes(rest)?;
        container.voxels.validate()?;
        Ok(container)
    }

    /// The named sidecar's bytes, if present.
    pub fn sidecar(&self, name: &str) -> Option<&[u8]> {
        self.sidecars
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.data.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{CHUNK_SIZE_USIZE, Chunk};
    use crate::voxel::Block;

    fn sample_chunk() -> Chunk {
        let mut c = Chunk::new();
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                c.set(x, 0, z, Block::Stone);
                c.set(x, 1, z, Block::Dirt);
                c.set(x, 2, z, Block::Grass);
            }
        }
        c
    }

    #[test]
    fn roundtrip_preserves_voxels_and_version() {
        let container = ChunkContainer::new(PalettedChunk::from_dense(&sample_chunk()));
        let bytes = container.encode();
        // Version 1 encodes as a single 0x01 varint byte at the front.
        assert_eq!(bytes[0], 1);
        let back = ChunkContainer::decode(&bytes).expect("decode own encoding");
        assert_eq!(back, container);
        assert_eq!(back.voxels.to_dense().get(5, 2, 5), Block::Grass);
    }

    #[test]
    fn roundtrip_is_deterministic() {
        let container = ChunkContainer::new(PalettedChunk::from_dense(&sample_chunk()));
        assert_eq!(container.encode(), container.encode());
        let reencoded = ChunkContainer::decode(&container.encode())
            .unwrap()
            .encode();
        assert_eq!(reencoded, container.encode());
    }

    #[test]
    fn unknown_sidecar_sections_are_tolerated_and_preserved() {
        // A future writer attaches a section this build knows nothing about.
        let mut container = ChunkContainer::new(PalettedChunk::uniform(Block::Stone));
        container.sidecars.push(Sidecar {
            name: "materials/slots-v0".to_string(),
            data: vec![0xDE, 0xAD, 0xBE, 0xEF, 42],
        });
        container.sidecars.push(Sidecar {
            name: "totally/unknown-v9".to_string(),
            data: vec![7; 100],
        });
        let bytes = container.encode();

        // Reading must not fail...
        let back = ChunkContainer::decode(&bytes).expect("unknown sidecars must not fail decode");
        assert_eq!(back.voxels.is_uniform(), Some(Block::Stone));
        // ...the unknown sections survive byte-for-byte...
        assert_eq!(
            back.sidecar("materials/slots-v0"),
            Some(&[0xDE, 0xAD, 0xBE, 0xEF, 42][..])
        );
        assert_eq!(back.sidecar("totally/unknown-v9"), Some(&[7u8; 100][..]));
        assert_eq!(back.sidecar("never/written"), None);
        // ...and a rewrite carries them forward unchanged.
        assert_eq!(back.encode(), bytes);
    }

    #[test]
    fn future_version_is_refused_cleanly() {
        let container = ChunkContainer::new(PalettedChunk::uniform(Block::Air));
        let mut bytes = container.encode();
        bytes[0] = 2; // pretend a v2 writer produced this
        match ChunkContainer::decode(&bytes) {
            Err(FormatError::UnsupportedVersion(2)) => {}
            other => panic!("expected UnsupportedVersion(2), got {other:?}"),
        }
    }

    #[test]
    fn corrupt_payload_is_an_error_not_a_panic() {
        let container = ChunkContainer::new(PalettedChunk::from_dense(&sample_chunk()));
        let bytes = container.encode();
        // Truncated.
        assert!(ChunkContainer::decode(&bytes[..bytes.len() / 2]).is_err());
        // Empty.
        assert!(ChunkContainer::decode(&[]).is_err());
    }
}
