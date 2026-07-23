//! Palette-compressed chunk storage (S3).
//!
//! A [`PalettedChunk`] stores the set of distinct block states appearing in a
//! chunk (the *palette*, ordered by first appearance so compression is
//! deterministic) plus one palette index per voxel, bit-packed at the minimal
//! width for the palette size:
//!
//! - 1 palette entry (uniform chunk — all-air, all-stone, ...): **0 bits per
//!   voxel**. The chunk is the palette entry; no index array is stored at all.
//! - P entries: `ceil(log2(P))` bits per voxel.
//!
//! Packing is **straddle-free** (Minecraft 1.16-style): each `u64` word holds
//! `floor(64 / bits)` indices and an index never crosses a word boundary, so
//! reads are one shift+mask. Unused padding bits are required to be zero,
//! which makes the encoding canonical: equal chunk contents produce equal
//! bytes.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::chunk::{CHUNK_SIZE_USIZE, CHUNK_VOLUME, Chunk};
use crate::voxel::Block;

/// Widest supported palette index. A chunk has at most [`CHUNK_VOLUME`]
/// distinct states, so 15 bits (32768 values) is the ceiling.
pub const MAX_PALETTE_BITS: u8 = 15;

/// Minimal bits per index for a palette of `len` entries: 0 for a uniform
/// chunk, `ceil(log2(len))` otherwise.
#[inline]
pub fn bits_for_palette_len(len: usize) -> u8 {
    if len <= 1 {
        0
    } else {
        (usize::BITS - (len - 1).leading_zeros()) as u8
    }
}

/// Validation failures for palette data arriving from disk/network.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PaletteError {
    #[error("palette must have at least one entry")]
    EmptyPalette,
    #[error("palette has duplicate entry {0:?}")]
    DuplicateEntry(Block),
    #[error("bit width {found} does not match palette of {palette_len} entries (want {want})")]
    WrongBitWidth {
        found: u8,
        want: u8,
        palette_len: usize,
    },
    #[error("index array length {found} does not match chunk volume {want}")]
    WrongLength { found: usize, want: usize },
    #[error("word array length {found} does not match expected {want}")]
    WrongWordCount { found: usize, want: usize },
    #[error("voxel {voxel} has palette index {index} out of range (palette len {palette_len})")]
    IndexOutOfRange {
        voxel: usize,
        index: usize,
        palette_len: usize,
    },
    #[error("non-zero padding bits in final word (encoding must be canonical)")]
    NonZeroPadding,
}

/// A fixed-length array of small unsigned integers, bit-packed straddle-free
/// into `u64` words. `bits == 0` means every value is 0 and no words are
/// stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackedIndices {
    bits: u8,
    len: usize,
    words: Vec<u64>,
}

impl PackedIndices {
    /// All-zero array of `len` values at `bits` width.
    ///
    /// # Panics
    /// Panics if `bits > MAX_PALETTE_BITS`.
    pub fn new(bits: u8, len: usize) -> Self {
        assert!(bits <= MAX_PALETTE_BITS, "bit width {bits} too wide");
        let words = if bits == 0 {
            Vec::new()
        } else {
            vec![0u64; len.div_ceil(64 / bits as usize)]
        };
        Self { bits, len, words }
    }

    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Heap bytes used by the packed words.
    pub fn heap_bytes(&self) -> usize {
        self.words.len() * std::mem::size_of::<u64>()
    }

    /// # Panics
    /// Panics if `i >= len`.
    #[inline]
    pub fn get(&self, i: usize) -> usize {
        assert!(i < self.len, "index {i} out of range {}", self.len);
        if self.bits == 0 {
            return 0;
        }
        let per_word = 64 / self.bits as usize;
        let word = self.words[i / per_word];
        let shift = (i % per_word) * self.bits as usize;
        ((word >> shift) & ((1u64 << self.bits) - 1)) as usize
    }

    /// # Panics
    /// Panics if `i >= len` or `v` does not fit in `bits`.
    #[inline]
    pub fn set(&mut self, i: usize, v: usize) {
        assert!(i < self.len, "index {i} out of range {}", self.len);
        let mask = if self.bits == 0 {
            0
        } else {
            (1u64 << self.bits) - 1
        };
        assert!(
            v as u64 <= mask,
            "value {v} does not fit in {} bits",
            self.bits
        );
        if self.bits == 0 {
            return; // only 0 is storable and it is implicit
        }
        let per_word = 64 / self.bits as usize;
        let word = &mut self.words[i / per_word];
        let shift = (i % per_word) * self.bits as usize;
        *word = (*word & !(mask << shift)) | ((v as u64) << shift);
    }

    /// Structural validation for untrusted data: word count matches `len` and
    /// `bits`, padding bits are zero.
    pub fn validate(&self) -> Result<(), PaletteError> {
        if self.bits == 0 {
            if !self.words.is_empty() {
                return Err(PaletteError::WrongWordCount {
                    found: self.words.len(),
                    want: 0,
                });
            }
            return Ok(());
        }
        let per_word = 64 / self.bits as usize;
        let want = self.len.div_ceil(per_word);
        if self.words.len() != want {
            return Err(PaletteError::WrongWordCount {
                found: self.words.len(),
                want,
            });
        }
        // Padding: bits in the last word beyond the final index, plus the
        // dead bits at the top of every word when bits does not divide 64.
        let mask = (1u64 << self.bits) - 1;
        for (w, word) in self.words.iter().enumerate() {
            let first = w * per_word;
            let used = self.len.saturating_sub(first).min(per_word);
            let mut live = 0u64;
            for slot in 0..used {
                live |= mask << (slot * self.bits as usize);
            }
            if word & !live != 0 {
                return Err(PaletteError::NonZeroPadding);
            }
        }
        Ok(())
    }
}

/// A 32^3 chunk stored as palette + bit-packed indices. See module docs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PalettedChunk {
    palette: Vec<Block>,
    indices: PackedIndices,
}

impl PalettedChunk {
    /// A chunk in which every voxel is `block` (1-entry palette, 0 bits).
    pub fn uniform(block: Block) -> Self {
        Self {
            palette: vec![block],
            indices: PackedIndices::new(0, CHUNK_VOLUME),
        }
    }

    /// Compress a dense chunk. Palette order is first appearance in index
    /// order, so identical contents always produce identical output.
    pub fn from_dense(chunk: &Chunk) -> Self {
        let blocks = chunk.blocks();
        let mut palette: Vec<Block> = Vec::new();
        let mut lookup: HashMap<Block, usize> = HashMap::new();
        for &b in blocks {
            if let std::collections::hash_map::Entry::Vacant(e) = lookup.entry(b) {
                e.insert(palette.len());
                palette.push(b);
            }
        }
        let bits = bits_for_palette_len(palette.len());
        let mut indices = PackedIndices::new(bits, CHUNK_VOLUME);
        if bits > 0 {
            for (i, b) in blocks.iter().enumerate() {
                indices.set(i, lookup[b]);
            }
        }
        Self { palette, indices }
    }

    /// Decompress to the dense form.
    pub fn to_dense(&self) -> Chunk {
        let mut chunk = Chunk::new();
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let block = self.palette[self.indices.get(Chunk::index(x, y, z))];
                    if block != Block::Air {
                        chunk.set(x, y, z, block);
                    }
                }
            }
        }
        chunk
    }

    /// Read one voxel without decompressing.
    ///
    /// # Panics
    /// Panics if any coordinate is `>= CHUNK_SIZE`.
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.palette[self.indices.get(Chunk::index(x, y, z))]
    }

    pub fn palette(&self) -> &[Block] {
        &self.palette
    }

    /// `Some(block)` when the chunk is a single-entry palette (uniform).
    pub fn is_uniform(&self) -> Option<Block> {
        if self.palette.len() == 1 {
            Some(self.palette[0])
        } else {
            None
        }
    }

    /// Bits per voxel index in the packed array.
    pub fn index_bits(&self) -> u8 {
        self.indices.bits()
    }

    /// Approximate in-memory heap footprint (palette + packed words).
    pub fn heap_bytes(&self) -> usize {
        self.palette.len() * std::mem::size_of::<Block>() + self.indices.heap_bytes()
    }

    /// Structural validation for untrusted (deserialized) data.
    pub fn validate(&self) -> Result<(), PaletteError> {
        if self.palette.is_empty() {
            return Err(PaletteError::EmptyPalette);
        }
        for (i, b) in self.palette.iter().enumerate() {
            if self.palette[..i].contains(b) {
                return Err(PaletteError::DuplicateEntry(*b));
            }
        }
        let want_bits = bits_for_palette_len(self.palette.len());
        if self.indices.bits() != want_bits {
            return Err(PaletteError::WrongBitWidth {
                found: self.indices.bits(),
                want: want_bits,
                palette_len: self.palette.len(),
            });
        }
        if self.indices.len() != CHUNK_VOLUME {
            return Err(PaletteError::WrongLength {
                found: self.indices.len(),
                want: CHUNK_VOLUME,
            });
        }
        self.indices.validate()?;
        for i in 0..CHUNK_VOLUME {
            let index = self.indices.get(i);
            if index >= self.palette.len() {
                return Err(PaletteError::IndexOutOfRange {
                    voxel: i,
                    index,
                    palette_len: self.palette.len(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkered_chunk() -> Chunk {
        let mut c = Chunk::new();
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let b = match (x + y * 3 + z * 7) % 5 {
                        0 => Block::Air,
                        1 => Block::Stone,
                        2 => Block::Dirt,
                        3 => Block::Grass,
                        _ => Block::Wood,
                    };
                    c.set(x, y, z, b);
                }
            }
        }
        c
    }

    #[test]
    fn bits_for_palette_len_edges() {
        assert_eq!(bits_for_palette_len(0), 0);
        assert_eq!(bits_for_palette_len(1), 0);
        assert_eq!(bits_for_palette_len(2), 1);
        assert_eq!(bits_for_palette_len(3), 2);
        assert_eq!(bits_for_palette_len(4), 2);
        assert_eq!(bits_for_palette_len(5), 3);
        assert_eq!(bits_for_palette_len(256), 8);
        assert_eq!(bits_for_palette_len(257), 9);
        assert_eq!(bits_for_palette_len(32768), 15);
    }

    #[test]
    fn dense_palette_dense_identity() {
        let dense = checkered_chunk();
        let paletted = PalettedChunk::from_dense(&dense);
        assert_eq!(paletted.palette().len(), 5);
        assert_eq!(paletted.index_bits(), 3);
        let back = paletted.to_dense();
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    assert_eq!(dense.get(x, y, z), back.get(x, y, z), "at ({x},{y},{z})");
                }
            }
        }
    }

    #[test]
    fn get_matches_dense_without_decompressing() {
        let dense = checkered_chunk();
        let paletted = PalettedChunk::from_dense(&dense);
        for (x, y, z) in [(0, 0, 0), (31, 31, 31), (5, 17, 23), (16, 8, 4)] {
            assert_eq!(paletted.get(x, y, z), dense.get(x, y, z));
        }
    }

    #[test]
    fn uniform_chunk_collapses_to_one_entry_zero_bits() {
        // All-air via from_dense.
        let air = PalettedChunk::from_dense(&Chunk::new());
        assert_eq!(air.is_uniform(), Some(Block::Air));
        assert_eq!(air.index_bits(), 0);
        assert_eq!(air.heap_bytes(), std::mem::size_of::<Block>());

        // All-stone via the constructor; identical to compressing a full chunk.
        let mut full = Chunk::new();
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    full.set(x, y, z, Block::Stone);
                }
            }
        }
        assert_eq!(
            PalettedChunk::from_dense(&full),
            PalettedChunk::uniform(Block::Stone)
        );
    }

    #[test]
    fn two_entry_palette_uses_one_bit() {
        let mut c = Chunk::new();
        c.set(0, 0, 0, Block::Stone);
        let p = PalettedChunk::from_dense(&c);
        assert_eq!(p.palette().len(), 2);
        assert_eq!(p.index_bits(), 1);
        // 32768 indices at 1 bit, 64 per word = 512 words = 4096 bytes, plus the
        // 2-entry palette at size_of::<Block>() each (one byte since the collapse).
        assert_eq!(p.heap_bytes(), 512 * 8 + 2 * std::mem::size_of::<Block>());
        assert_eq!(p.to_dense().get(0, 0, 0), Block::Stone);
        assert_eq!(p.to_dense().get(1, 0, 0), Block::Air);
    }

    #[test]
    fn packed_indices_over_256_entries() {
        // A >256-entry palette needs 9 bits; PalettedChunk can't exceed the
        // Block enum's variants, so the packing layer is exercised directly.
        let bits = bits_for_palette_len(300);
        assert_eq!(bits, 9);
        let len = 1000;
        let mut p = PackedIndices::new(bits, len);
        for i in 0..len {
            p.set(i, (i * 7) % 300);
        }
        for i in 0..len {
            assert_eq!(p.get(i), (i * 7) % 300, "at {i}");
        }
        // Straddle-free: 7 indices per word, ceil(1000/7) = 143 words.
        assert_eq!(p.heap_bytes(), 143 * 8);
        p.validate().expect("canonical packing validates");
    }

    #[test]
    fn packed_indices_zero_bits() {
        let p = PackedIndices::new(0, CHUNK_VOLUME);
        assert_eq!(p.heap_bytes(), 0);
        assert_eq!(p.get(0), 0);
        assert_eq!(p.get(CHUNK_VOLUME - 1), 0);
        p.validate().expect("zero-bit array validates");
    }

    #[test]
    fn compression_is_deterministic() {
        let dense = checkered_chunk();
        let a = PalettedChunk::from_dense(&dense);
        let b = PalettedChunk::from_dense(&dense);
        assert_eq!(a, b);
        // Palette order is first-appearance: voxel 0 has (0+0+0)%5 = Air first.
        assert_eq!(a.palette()[0], Block::Air);
    }

    #[test]
    fn validate_rejects_corruption() {
        let good = PalettedChunk::from_dense(&checkered_chunk());
        good.validate().expect("fresh compression validates");

        // Out-of-range index: force palette shorter than the indices claim.
        let mut bad = good.clone();
        bad.palette.truncate(4);
        assert!(matches!(
            bad.validate(),
            Err(PaletteError::WrongBitWidth { .. })
        ));

        // Duplicate palette entry.
        let mut bad = good.clone();
        bad.palette[1] = bad.palette[0];
        assert_eq!(
            bad.validate(),
            Err(PaletteError::DuplicateEntry(Block::Air))
        );

        // Non-canonical padding.
        let mut bad = good.clone();
        *bad.indices.words.last_mut().unwrap() |= 1u64 << 63;
        assert!(matches!(
            bad.validate(),
            Err(PaletteError::NonZeroPadding) | Err(PaletteError::IndexOutOfRange { .. })
        ));

        // Empty palette.
        let bad = PalettedChunk {
            palette: vec![],
            indices: PackedIndices::new(0, CHUNK_VOLUME),
        };
        assert_eq!(bad.validate(), Err(PaletteError::EmptyPalette));
    }
}
