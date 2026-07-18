//! Mixture interning + chunk-level material storage (spike S8).
//!
//! Survival strategy for the 8-eighths model (docs/design/materials.md
//! § Storage): distinct canonical [`VoxelContents`] states are **interned** in
//! a region-level [`MixtureTable`]; each chunk stores one [`MixtureId`] per
//! voxel through the *same palette discipline as the block grid* — a
//! [`MaterialChunk`] is a palette of distinct interned ids (first-appearance
//! order) plus bit-packed indices, reusing [`PackedIndices`] from S3's
//! palette layer.
//!
//! **Sidecar schema `"materials/slots-v0"`** (postcard encoding):
//!
//! ```text
//! [palette: seq of MixtureId (u32 varint)]  -- distinct, first-appearance order
//! [indices: PackedIndices]                  -- bits, len = 32^3, packed words
//! ```
//!
//! Attached to the format-v1 [`ChunkContainer`] as a named sidecar; readers
//! that don't know the name keep working (S3's rules). A chunk with no
//! material contents attaches **no sidecar at all** — debris-free terrain
//! pays zero bytes.
//!
//! The mixture table itself is region-level, one per region file (the region
//! grouping is S3 open question #7; until it exists the table is measured and
//! stored standalone as `"materials/mixtures-v0"`):
//!
//! ```text
//! [count: u32 LEB128 varint]
//! count x [VoxelContents compact encoding]  -- entry i is mixture id i;
//!                                              entry 0 is always EMPTY
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::contents::{ContentsError, VoxelContents};
use crate::chunk::{CHUNK_VOLUME, Chunk};
use crate::format::{ChunkContainer, Sidecar};
use crate::palette::{PackedIndices, PaletteError, bits_for_palette_len};

/// Name of the per-chunk material sidecar section in the format-v1 container.
pub const MATERIALS_SIDECAR_NAME: &str = "materials/slots-v0";

/// Name of the region-level mixture table section.
pub const MIXTURES_SIDECAR_NAME: &str = "materials/mixtures-v0";

/// Interned index of a canonical [`VoxelContents`] in a region's
/// [`MixtureTable`]. Id 0 is always the empty voxel.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize,
)]
pub struct MixtureId(u32);

impl MixtureId {
    /// The empty voxel's id, in every table.
    pub const EMPTY: MixtureId = MixtureId(0);

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Errors decoding a serialized mixture table.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum TableError {
    #[error("truncated mixture table")]
    Truncated,
    #[error("mixture table varint too wide")]
    BadVarint,
    #[error("mixture table entry 0 must be the empty contents")]
    FirstNotEmpty,
    #[error("duplicate mixture table entry at id {0}")]
    Duplicate(u32),
    #[error("invalid mixture table entry at id {id}: {source}")]
    Entry {
        id: u32,
        #[source]
        source: ContentsError,
    },
    #[error("trailing bytes after mixture table")]
    TrailingBytes,
}

/// Region-level table of distinct canonical mixture states. Ids are assigned
/// by first-intern order, so a deterministic fill order produces a
/// deterministic table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MixtureTable {
    entries: Vec<VoxelContents>,
    lookup: HashMap<VoxelContents, MixtureId>,
}

impl MixtureTable {
    /// A table containing only the empty mixture (id 0).
    pub fn new() -> Self {
        let mut t = Self {
            entries: Vec::new(),
            lookup: HashMap::new(),
        };
        t.intern(VoxelContents::EMPTY);
        t
    }

    /// The id for `contents`, interning it if new. Canonical form makes this
    /// well-defined: any ordering of the same multisets is the same entry.
    pub fn intern(&mut self, contents: VoxelContents) -> MixtureId {
        if let Some(&id) = self.lookup.get(&contents) {
            return id;
        }
        let id = MixtureId(self.entries.len() as u32);
        self.entries.push(contents);
        self.lookup.insert(contents, id);
        id
    }

    /// The contents for an interned id.
    #[inline]
    pub fn get(&self, id: MixtureId) -> Option<&VoxelContents> {
        self.entries.get(id.0 as usize)
    }

    /// Number of distinct interned states (including empty).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize (see module docs). Deterministic given intern order.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_varint(self.entries.len() as u32, &mut out);
        for e in &self.entries {
            e.encode_into(&mut out);
        }
        out
    }

    /// Deserialize + validate: entry 0 empty, all entries canonical and
    /// distinct, no trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, TableError> {
        let (count, mut rest) = read_varint(bytes)?;
        let mut table = Self {
            entries: Vec::with_capacity(count as usize),
            lookup: HashMap::with_capacity(count as usize),
        };
        for id in 0..count {
            let (contents, r) =
                VoxelContents::decode(rest).map_err(|source| TableError::Entry { id, source })?;
            rest = r;
            if id == 0 && !contents.is_empty() {
                return Err(TableError::FirstNotEmpty);
            }
            if table.lookup.contains_key(&contents) {
                return Err(TableError::Duplicate(id));
            }
            table.lookup.insert(contents, MixtureId(id));
            table.entries.push(contents);
        }
        if !rest.is_empty() {
            return Err(TableError::TrailingBytes);
        }
        if table.entries.is_empty() {
            // An empty table cannot even represent "no contents".
            return Err(TableError::FirstNotEmpty);
        }
        Ok(table)
    }

    /// Wrap the encoded table as its named sidecar section.
    pub fn to_sidecar(&self) -> Sidecar {
        Sidecar {
            name: MIXTURES_SIDECAR_NAME.to_string(),
            data: self.encode(),
        }
    }
}

fn write_varint(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn read_varint(bytes: &[u8]) -> Result<(u32, &[u8]), TableError> {
    let mut v: u32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if i >= 5 {
            return Err(TableError::BadVarint);
        }
        v |= u32::from(b & 0x7F) << (7 * i);
        if b & 0x80 == 0 {
            return Ok((v, &bytes[i + 1..]));
        }
    }
    Err(TableError::Truncated)
}

/// Errors decoding a material chunk sidecar.
#[derive(Error, Debug)]
pub enum MaterialChunkError {
    #[error("malformed material sidecar: {0}")]
    Malformed(#[from] postcard::Error),
    #[error("mixture palette must have at least one entry")]
    EmptyPalette,
    #[error("mixture palette has duplicate entry {0:?}")]
    DuplicateEntry(MixtureId),
    #[error("bit width {found} does not match palette of {palette_len} entries (want {want})")]
    WrongBitWidth {
        found: u8,
        want: u8,
        palette_len: usize,
    },
    #[error("index array length {found} does not match chunk volume {want}")]
    WrongLength { found: usize, want: usize },
    #[error("voxel {voxel} has palette index {index} out of range (palette len {palette_len})")]
    IndexOutOfRange {
        voxel: usize,
        index: usize,
        palette_len: usize,
    },
    #[error("packed indices invalid: {0}")]
    Packed(#[from] PaletteError),
    #[error("palette references mixture id {0} beyond table of {1} entries")]
    UnknownMixture(u32, usize),
}

/// Per-chunk material storage: one interned [`MixtureId`] per voxel, palette
/// compressed exactly like the block grid (first-appearance palette,
/// straddle-free bit-packed indices, canonical padding). A palette entry *is*
/// an interned mixture id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialChunk {
    palette: Vec<MixtureId>,
    indices: PackedIndices,
}

impl Default for MaterialChunk {
    fn default() -> Self {
        Self::empty()
    }
}

impl MaterialChunk {
    /// A chunk in which every voxel is the empty mixture.
    pub fn empty() -> Self {
        Self {
            palette: vec![MixtureId::EMPTY],
            indices: PackedIndices::new(0, CHUNK_VOLUME),
        }
    }

    /// Compress a dense id array (one id per voxel, [`Chunk::index`] order).
    /// Palette order is first appearance, so equal contents produce equal
    /// bytes.
    ///
    /// # Panics
    /// Panics if `ids.len() != CHUNK_VOLUME`.
    pub fn from_dense(ids: &[MixtureId]) -> Self {
        assert_eq!(ids.len(), CHUNK_VOLUME, "dense id array must cover the chunk");
        let mut palette: Vec<MixtureId> = Vec::new();
        let mut lookup: HashMap<MixtureId, usize> = HashMap::new();
        for &id in ids {
            if let std::collections::hash_map::Entry::Vacant(e) = lookup.entry(id) {
                e.insert(palette.len());
                palette.push(id);
            }
        }
        let bits = bits_for_palette_len(palette.len());
        let mut indices = PackedIndices::new(bits, CHUNK_VOLUME);
        if bits > 0 {
            for (i, id) in ids.iter().enumerate() {
                indices.set(i, lookup[id]);
            }
        }
        Self { palette, indices }
    }

    /// Read one voxel's mixture id without decompressing.
    ///
    /// # Panics
    /// Panics if any coordinate is `>= CHUNK_SIZE`.
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> MixtureId {
        self.palette[self.indices.get(Chunk::index(x, y, z))]
    }

    /// Distinct mixture ids appearing in the chunk, first-appearance order.
    pub fn palette(&self) -> &[MixtureId] {
        &self.palette
    }

    /// Bits per voxel index in the packed array.
    pub fn index_bits(&self) -> u8 {
        self.indices.bits()
    }

    /// True when every voxel is the empty mixture — the chunk that should
    /// carry no sidecar at all.
    pub fn is_all_empty(&self) -> bool {
        self.palette == [MixtureId::EMPTY]
    }

    /// Serialize the sidecar payload (postcard; see module docs).
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("in-memory encode cannot fail")
    }

    /// Deserialize + structurally validate a sidecar payload.
    pub fn decode(bytes: &[u8]) -> Result<Self, MaterialChunkError> {
        let chunk: MaterialChunk = postcard::from_bytes(bytes)?;
        chunk.validate()?;
        Ok(chunk)
    }

    /// Structural validation mirroring [`crate::palette::PalettedChunk`]:
    /// non-empty duplicate-free palette, minimal bit width, canonical packing,
    /// indices in range.
    pub fn validate(&self) -> Result<(), MaterialChunkError> {
        if self.palette.is_empty() {
            return Err(MaterialChunkError::EmptyPalette);
        }
        for (i, id) in self.palette.iter().enumerate() {
            if self.palette[..i].contains(id) {
                return Err(MaterialChunkError::DuplicateEntry(*id));
            }
        }
        let want_bits = bits_for_palette_len(self.palette.len());
        if self.indices.bits() != want_bits {
            return Err(MaterialChunkError::WrongBitWidth {
                found: self.indices.bits(),
                want: want_bits,
                palette_len: self.palette.len(),
            });
        }
        if self.indices.len() != CHUNK_VOLUME {
            return Err(MaterialChunkError::WrongLength {
                found: self.indices.len(),
                want: CHUNK_VOLUME,
            });
        }
        self.indices.validate()?;
        for i in 0..CHUNK_VOLUME {
            let index = self.indices.get(i);
            if index >= self.palette.len() {
                return Err(MaterialChunkError::IndexOutOfRange {
                    voxel: i,
                    index,
                    palette_len: self.palette.len(),
                });
            }
        }
        Ok(())
    }

    /// Check every palette entry resolves in the region table.
    pub fn validate_against(&self, table: &MixtureTable) -> Result<(), MaterialChunkError> {
        for id in &self.palette {
            if table.get(*id).is_none() {
                return Err(MaterialChunkError::UnknownMixture(id.raw(), table.len()));
            }
        }
        Ok(())
    }

    /// The chunk as a named sidecar section — or `None` when all-empty, so
    /// debris-free terrain attaches nothing and pays nothing.
    pub fn to_sidecar(&self) -> Option<Sidecar> {
        if self.is_all_empty() {
            None
        } else {
            Some(Sidecar {
                name: MATERIALS_SIDECAR_NAME.to_string(),
                data: self.encode(),
            })
        }
    }

    /// Read the material sidecar from a format-v1 container. `Ok(None)` when
    /// the container carries none (a debris-free chunk).
    pub fn from_container(
        container: &ChunkContainer,
    ) -> Result<Option<Self>, MaterialChunkError> {
        match container.sidecar(MATERIALS_SIDECAR_NAME) {
            None => Ok(None),
            Some(bytes) => Self::decode(bytes).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialId;
    use crate::materials::contents::StructureShape;
    use crate::palette::PalettedChunk;
    use crate::voxel::Block;

    fn mix(materials: &[MaterialId]) -> VoxelContents {
        VoxelContents::debris_only(materials).unwrap()
    }

    #[test]
    fn interning_is_deduplicating_and_deterministic() {
        let mut table = MixtureTable::new();
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(MixtureId::EMPTY), Some(&VoxelContents::EMPTY));

        let a = table.intern(mix(&[MaterialId::SAND, MaterialId::SNOW]));
        let b = table.intern(mix(&[MaterialId::SNOW, MaterialId::SAND]));
        assert_eq!(a, b, "canonical form makes order irrelevant");
        assert_eq!(table.len(), 2);

        let c = table.intern(mix(&[MaterialId::SNOW]));
        assert_ne!(a, c);
        assert_eq!(table.intern(VoxelContents::EMPTY), MixtureId::EMPTY);
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn table_roundtrip() {
        let mut table = MixtureTable::new();
        table.intern(mix(&[MaterialId::SAND, MaterialId::SAND, MaterialId::GRAVEL]));
        table.intern(
            VoxelContents::new(
                StructureShape::Slab,
                &[MaterialId::SCREE, MaterialId::SCREE],
                &[MaterialId::SILT],
                &[MaterialId::ASH],
            )
            .unwrap(),
        );
        let bytes = table.encode();
        let back = MixtureTable::decode(&bytes).expect("roundtrip");
        assert_eq!(back.len(), table.len());
        for i in 0..table.len() as u32 {
            assert_eq!(back.get(MixtureId(i)), table.get(MixtureId(i)));
        }
        assert_eq!(back.encode(), bytes, "re-encode is byte-identical");
    }

    #[test]
    fn table_decode_rejects_corruption() {
        let mut table = MixtureTable::new();
        table.intern(mix(&[MaterialId::SAND]));
        let bytes = table.encode();
        // Truncated.
        assert!(MixtureTable::decode(&bytes[..bytes.len() - 1]).is_err());
        // Trailing bytes.
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(MixtureTable::decode(&bad), Err(TableError::TrailingBytes));
        // First entry not empty: encode a table whose entry 0 has debris.
        let mut manual = Vec::new();
        write_varint(1, &mut manual);
        mix(&[MaterialId::SAND]).encode_into(&mut manual);
        assert_eq!(
            MixtureTable::decode(&manual),
            Err(TableError::FirstNotEmpty)
        );
        // Duplicate entries.
        let mut manual = Vec::new();
        write_varint(3, &mut manual);
        VoxelContents::EMPTY.encode_into(&mut manual);
        mix(&[MaterialId::SAND]).encode_into(&mut manual);
        mix(&[MaterialId::SAND]).encode_into(&mut manual);
        assert_eq!(MixtureTable::decode(&manual), Err(TableError::Duplicate(2)));
        // Zero-entry table.
        let mut manual = Vec::new();
        write_varint(0, &mut manual);
        assert_eq!(MixtureTable::decode(&manual), Err(TableError::FirstNotEmpty));
    }

    #[test]
    fn varint_roundtrip() {
        for v in [0u32, 1, 127, 128, 300, 16_383, 16_384, u32::MAX] {
            let mut out = Vec::new();
            write_varint(v, &mut out);
            let (back, rest) = read_varint(&out).unwrap();
            assert_eq!(back, v);
            assert!(rest.is_empty());
        }
        assert_eq!(read_varint(&[0x80]), Err(TableError::Truncated));
        assert_eq!(
            read_varint(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]),
            Err(TableError::BadVarint)
        );
    }

    #[test]
    fn material_chunk_palette_compression() {
        let mut table = MixtureTable::new();
        let snowy = table.intern(mix(&[MaterialId::SNOW; 3]));
        let sandy = table.intern(mix(&[MaterialId::SAND; 8]));

        let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
        // A layer of full sand at y=0, snow dusting at y=1.
        for z in 0..32 {
            for x in 0..32 {
                dense[Chunk::index(x, 0, z)] = sandy;
                dense[Chunk::index(x, 1, z)] = snowy;
            }
        }
        let chunk = MaterialChunk::from_dense(&dense);
        assert_eq!(chunk.palette().len(), 3);
        assert_eq!(chunk.index_bits(), 2);
        assert!(!chunk.is_all_empty());
        assert_eq!(chunk.get(5, 0, 5), sandy);
        assert_eq!(chunk.get(5, 1, 5), snowy);
        assert_eq!(chunk.get(5, 2, 5), MixtureId::EMPTY);
        chunk.validate().expect("fresh compression validates");
        chunk.validate_against(&table).expect("ids resolve");

        // Unknown id detection.
        let orphan_ids = vec![MixtureId(99); CHUNK_VOLUME];
        let orphan = MaterialChunk::from_dense(&orphan_ids);
        assert!(matches!(
            orphan.validate_against(&table),
            Err(MaterialChunkError::UnknownMixture(99, _))
        ));
    }

    #[test]
    fn sidecar_roundtrip_through_container() {
        let mut table = MixtureTable::new();
        let drift = table.intern(mix(&[MaterialId::SNOW, MaterialId::SNOW, MaterialId::SAND]));
        let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
        dense[Chunk::index(1, 2, 3)] = drift;
        let materials = MaterialChunk::from_dense(&dense);

        let mut container = ChunkContainer::new(PalettedChunk::uniform(Block::Stone));
        container
            .sidecars
            .push(materials.to_sidecar().expect("non-empty chunk has a sidecar"));
        let bytes = container.encode();

        let back = ChunkContainer::decode(&bytes).expect("container roundtrip");
        let restored = MaterialChunk::from_container(&back)
            .expect("valid sidecar")
            .expect("sidecar present");
        assert_eq!(restored, materials);
        assert_eq!(restored.get(1, 2, 3), drift);
    }

    #[test]
    fn debris_free_chunk_attaches_no_sidecar_and_pays_nothing() {
        let empty = MaterialChunk::empty();
        assert!(empty.is_all_empty());
        assert_eq!(empty.to_sidecar(), None);

        let bare = ChunkContainer::new(PalettedChunk::uniform(Block::Stone));
        let mut with_materials = bare.clone();
        if let Some(sidecar) = empty.to_sidecar() {
            with_materials.sidecars.push(sidecar);
        }
        assert_eq!(
            with_materials.encode(),
            bare.encode(),
            "an all-empty material chunk must not change the container bytes"
        );
        assert!(
            MaterialChunk::from_container(&bare)
                .expect("no sidecar is not an error")
                .is_none()
        );
    }

    #[test]
    fn decode_rejects_corrupt_material_chunk() {
        let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
        dense[0] = MixtureId(1);
        let chunk = MaterialChunk::from_dense(&dense);
        let bytes = chunk.encode();
        assert!(MaterialChunk::decode(&bytes).is_ok());
        // Truncation.
        assert!(MaterialChunk::decode(&bytes[..bytes.len() / 2]).is_err());
        // Duplicate palette entries.
        let bad = MaterialChunk {
            palette: vec![MixtureId(1), MixtureId(1)],
            indices: PackedIndices::new(1, CHUNK_VOLUME),
        };
        assert!(matches!(
            bad.validate(),
            Err(MaterialChunkError::DuplicateEntry(MixtureId(1)))
        ));
        // Wrong bit width.
        let bad = MaterialChunk {
            palette: vec![MixtureId(0), MixtureId(1)],
            indices: PackedIndices::new(2, CHUNK_VOLUME),
        };
        assert!(matches!(
            bad.validate(),
            Err(MaterialChunkError::WrongBitWidth { .. })
        ));
        // Empty palette.
        let bad = MaterialChunk {
            palette: vec![],
            indices: PackedIndices::new(0, CHUNK_VOLUME),
        };
        assert!(matches!(bad.validate(), Err(MaterialChunkError::EmptyPalette)));
    }
}
