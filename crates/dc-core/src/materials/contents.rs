//! The voxel as a container of eighths (docs/design/materials.md).
//!
//! A [`VoxelContents`] records three occupancy roles over a voxel's 8
//! volume-eighths:
//!
//! - **Structure**: a [`StructureShape`] *reserves* capacity (full = 8,
//!   slab = 4, quarter = 2); its filled slots hold structural materials
//!   (possibly heterogeneous — rubble). Unfilled reserved slots are **pores**.
//! - **Pore fill**: fine material packed into the structure's open pores.
//! - **Debris**: loose granular material filling unreserved volume.
//!
//! **Canonical form** (the property interning and palettes rely on): each of
//! the three material lists is an *unordered multiset*, stored sorted by
//! ascending material id, with unused slots normalized to a fixed pad value.
//! Equal contents therefore have equal representation, equal hashes, and
//! equal encodings — deposit order is deliberately destroyed (that is what
//! mixing *is*).
//!
//! **Compact encoding**: 2 header bytes (shape 2 bits, three lengths 4 bits
//! each) followed by one byte per occupied eighth, segments in
//! structure/pore/debris order, each sorted. Decoding validates canonicality,
//! so the encoding is bijective with the value: equal bytes ⇔ equal contents.

use thiserror::Error;

use super::MaterialId;

/// Volume-eighths per voxel.
pub const VOXEL_EIGHTHS: u8 = 8;

/// Occupied eighths at which a voxel counts as **solid** — collidable,
/// stood-on, face-culling (docs/design/visuals.md reserves "solid ≥ 4/8").
///
/// This is the occupancy-tier home of a property that has lived on `Block`
/// (`Block::is_solid`) since S1. The forms contract (materials.md
/// § forms design pass) moves solidity off the block, which is a derived
/// *classification*, onto the contents, which are the source of truth.
/// [`VoxelContents::is_occupancy_solid`] is the one place that decides;
/// consumers migrate to it slice by slice rather than each inventing a
/// threshold.
pub const SOLID_EIGHTHS: u8 = 4;

/// Structure shape occupying a voxel; reserves capacity in eighths. The shape
/// vocabulary will grow (stairs, …) — what matters here is reserved capacity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum StructureShape {
    #[default]
    None = 0,
    Quarter = 1,
    Slab = 2,
    Full = 3,
}

impl StructureShape {
    /// Reserved capacity in eighths.
    #[inline]
    pub const fn capacity(self) -> u8 {
        match self {
            StructureShape::None => 0,
            StructureShape::Quarter => 2,
            StructureShape::Slab => 4,
            StructureShape::Full => 8,
        }
    }

    #[inline]
    const fn from_raw(raw: u8) -> StructureShape {
        match raw & 0b11 {
            0 => StructureShape::None,
            1 => StructureShape::Quarter,
            2 => StructureShape::Slab,
            _ => StructureShape::Full,
        }
    }
}

/// Invalid contents or encodings.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ContentsError {
    #[error("structure fill {fill} exceeds shape capacity {capacity}")]
    StructureOverflow { fill: u8, capacity: u8 },
    #[error("pore fill {fill} exceeds open pores {pores}")]
    PoreOverflow { fill: u8, pores: u8 },
    #[error("debris {debris} exceeds free eighths {free} outside the shape")]
    DebrisOverflow { debris: u8, free: u8 },
    #[error("truncated contents encoding")]
    Truncated,
    #[error("unknown material id {0}")]
    UnknownMaterial(u8),
    #[error("segment not sorted — encoding is not canonical")]
    NotCanonical,
}

/// Pad value for unused slots so derived `Eq`/`Hash` see one canonical
/// representation per value. The lengths are authoritative; pads are never
/// read as content.
const PAD: MaterialId = MaterialId::SAND;

/// Canonical contents of one voxel. Construct via [`VoxelContents::new`] /
/// [`VoxelContents::debris_only`]; both sort the multisets. `Copy` on
/// purpose: 12 bytes, hashed constantly by the interner.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoxelContents {
    shape: StructureShape,
    structure_len: u8,
    pore_len: u8,
    debris_len: u8,
    /// Segments: `[0, structure_len)` structural fill, then pore fill, then
    /// debris; each segment sorted ascending; the rest is [`PAD`].
    slots: [MaterialId; VOXEL_EIGHTHS as usize],
}

impl VoxelContents {
    /// The empty voxel: no structure, no debris. Interned as mixture id 0.
    pub const EMPTY: VoxelContents = VoxelContents {
        shape: StructureShape::None,
        structure_len: 0,
        pore_len: 0,
        debris_len: 0,
        slots: [PAD; VOXEL_EIGHTHS as usize],
    };

    /// Build canonical contents from the three material multisets (any
    /// order; they are sorted here). Capacity invariants:
    ///
    /// - `structure.len() <= shape.capacity()`
    /// - `pore_fill.len() <= shape.capacity() - structure.len()` (open pores)
    /// - `debris.len() <= 8 - shape.capacity()` (unreserved volume)
    ///
    /// Whether a material's grain *fits* a pore is a packing-rule question
    /// for the deposition/packing logic, not a representational invariant.
    pub fn new(
        shape: StructureShape,
        structure: &[MaterialId],
        pore_fill: &[MaterialId],
        debris: &[MaterialId],
    ) -> Result<Self, ContentsError> {
        let capacity = shape.capacity();
        if structure.len() > capacity as usize {
            return Err(ContentsError::StructureOverflow {
                fill: structure.len().min(255) as u8,
                capacity,
            });
        }
        let structure_len = structure.len() as u8;
        let pores = capacity - structure_len;
        if pore_fill.len() > pores as usize {
            return Err(ContentsError::PoreOverflow {
                fill: pore_fill.len().min(255) as u8,
                pores,
            });
        }
        let pore_len = pore_fill.len() as u8;
        let free = VOXEL_EIGHTHS - capacity;
        if debris.len() > free as usize {
            return Err(ContentsError::DebrisOverflow {
                debris: debris.len().min(255) as u8,
                free,
            });
        }
        let debris_len = debris.len() as u8;

        let mut slots = [PAD; VOXEL_EIGHTHS as usize];
        let (a, b) = (structure_len as usize, (structure_len + pore_len) as usize);
        let c = b + debris_len as usize;
        slots[..a].copy_from_slice(structure);
        slots[a..b].copy_from_slice(pore_fill);
        slots[b..c].copy_from_slice(debris);
        slots[..a].sort_unstable();
        slots[a..b].sort_unstable();
        slots[b..c].sort_unstable();
        Ok(Self {
            shape,
            structure_len,
            pore_len,
            debris_len,
            slots,
        })
    }

    /// Loose debris only (no structure). The deposition simulator's staple.
    pub fn debris_only(debris: &[MaterialId]) -> Result<Self, ContentsError> {
        Self::new(StructureShape::None, &[], &[], debris)
    }

    #[inline]
    pub fn shape(&self) -> StructureShape {
        self.shape
    }

    /// Structural fill multiset, sorted ascending.
    #[inline]
    pub fn structure(&self) -> &[MaterialId] {
        &self.slots[..self.structure_len as usize]
    }

    /// Pore-fill multiset, sorted ascending.
    #[inline]
    pub fn pore_fill(&self) -> &[MaterialId] {
        &self.slots[self.structure_len as usize..(self.structure_len + self.pore_len) as usize]
    }

    /// Debris multiset, sorted ascending.
    #[inline]
    pub fn debris(&self) -> &[MaterialId] {
        let start = (self.structure_len + self.pore_len) as usize;
        &self.slots[start..start + self.debris_len as usize]
    }

    /// All occupied eighths (structure, pore fill, debris), in segment order.
    #[inline]
    pub fn filled_slots(&self) -> &[MaterialId] {
        &self.slots[..(self.structure_len + self.pore_len + self.debris_len) as usize]
    }

    /// Occupied eighths: structural fill + pore fill + debris.
    #[inline]
    pub fn solid_eighths(&self) -> u8 {
        self.structure_len + self.pore_len + self.debris_len
    }

    /// Open (unfilled) pores in the structure's reserved capacity.
    #[inline]
    pub fn open_pores(&self) -> u8 {
        self.shape.capacity() - self.structure_len - self.pore_len
    }

    /// Unreserved eighths still free for debris.
    #[inline]
    pub fn free_debris_eighths(&self) -> u8 {
        VOXEL_EIGHTHS - self.shape.capacity() - self.debris_len
    }

    // ----- occupancy primitives -------------------------------------------
    //
    // **One occupancy answer, read by everyone.** Four systems in flight all
    // need to know "how full is this voxel, and with what kind of stuff":
    //
    // - **fluid fill** (water, aquifers, waterlogging): how many eighths can a
    //   fluid still enter → [`Self::free_eighths`], split into
    //   [`Self::open_pores`] (fluid inside a porous rock) and
    //   [`Self::free_debris_eighths`] (fluid in the open volume);
    // - **the loose gravity march** (materials.md § loose-material mechanics):
    //   which eighths fall column-wise → [`Self::loose_eighths`], and whether
    //   nothing structural is holding them → [`Self::is_loose_only`];
    // - **compaction** (boot-on-snow, and its deep-time cousin): loose volume
    //   available to press, and pore capacity to press it into →
    //   [`Self::loose_eighths`] + [`Self::open_pores`], with
    //   [`Self::bound_eighths`] as the part already held;
    // - **the sim/collision tier**: is this voxel solid enough to stand on and
    //   occlude → [`Self::is_occupancy_solid`] against [`SOLID_EIGHTHS`].
    //
    // They are deliberately thin derivations over the three segment lengths so
    // that no consumer re-derives occupancy by walking `filled_slots()`, and no
    // two consumers can disagree about how full a voxel is. Solidity in
    // particular is moving OFF `Block` onto this threshold
    // (docs/design/visuals.md reserves "solid ≥ 4/8"); the primitive lands
    // here first — rewiring the client's collision to read it is its own slice.

    /// Total unoccupied eighths: open pores plus unreserved volume not yet
    /// holding debris. What a fluid could still enter.
    #[inline]
    pub fn free_eighths(&self) -> u8 {
        VOXEL_EIGHTHS - self.solid_eighths()
    }

    /// Every eighth is occupied — the voxel admits nothing more.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.solid_eighths() == VOXEL_EIGHTHS
    }

    /// Loose (granular) eighths: the debris role. What gravity marches, what a
    /// boot compacts, what a shovel takes first. Pore fill is *not* loose — the
    /// structure around it holds it ([`Self::bound_eighths`]).
    #[inline]
    pub fn loose_eighths(&self) -> u8 {
        self.debris_len
    }

    /// Eighths held in place by a structure: structural fill plus whatever is
    /// packed into its pores. The complement of [`Self::loose_eighths`] within
    /// the occupied volume.
    #[inline]
    pub fn bound_eighths(&self) -> u8 {
        self.structure_len + self.pore_len
    }

    /// Does a structure shape reserve any capacity here?
    #[inline]
    pub fn has_structure(&self) -> bool {
        self.shape.capacity() > 0
    }

    /// Loose material with no structure at all: nothing holds it, so it falls
    /// (the gravity march) and it renders as a partial-height layer
    /// (journal/0010's dormant mesher capability). False for the empty voxel.
    #[inline]
    pub fn is_loose_only(&self) -> bool {
        !self.has_structure() && self.debris_len > 0
    }

    /// Is this voxel solid *by occupancy*? The single occupancy-tier answer for
    /// collision, face culling and standing-on, against [`SOLID_EIGHTHS`].
    #[inline]
    pub fn is_occupancy_solid(&self) -> bool {
        self.solid_eighths() >= SOLID_EIGHTHS
    }

    /// Structure density: filled structural slots over reserved capacity.
    /// `None` when there is no structure. Complement is porosity.
    pub fn structure_density(&self) -> Option<f32> {
        match self.shape.capacity() {
            0 => None,
            cap => Some(f32::from(self.structure_len) / f32::from(cap)),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    /// Bytes of the compact encoding: 2 header bytes + 1 per occupied eighth.
    #[inline]
    pub fn encoded_len(&self) -> usize {
        2 + self.solid_eighths() as usize
    }

    /// Append the compact canonical encoding (see module docs).
    pub fn encode_into(&self, out: &mut Vec<u8>) {
        out.push((self.shape as u8) | (self.structure_len << 2));
        out.push(self.pore_len | (self.debris_len << 4));
        for m in self.filled_slots() {
            out.push(m.raw());
        }
    }

    /// Decode one contents value from the front of `bytes`; returns the value
    /// and the remaining bytes. Validates every invariant plus canonicality
    /// (sorted segments), so decode ∘ encode is identity and non-canonical
    /// bytes are rejected rather than aliased.
    pub fn decode(bytes: &[u8]) -> Result<(Self, &[u8]), ContentsError> {
        let (&b0, rest) = bytes.split_first().ok_or(ContentsError::Truncated)?;
        let (&b1, rest) = rest.split_first().ok_or(ContentsError::Truncated)?;
        let shape = StructureShape::from_raw(b0);
        let structure_len = b0 >> 2;
        let pore_len = b1 & 0x0F;
        let debris_len = b1 >> 4;
        let total = structure_len as usize + pore_len as usize + debris_len as usize;
        if rest.len() < total {
            return Err(ContentsError::Truncated);
        }
        let (raw, remaining) = rest.split_at(total);
        let mut mats = [PAD; VOXEL_EIGHTHS as usize];
        if total > VOXEL_EIGHTHS as usize {
            // Header claims more slots than a voxel has; surface as the
            // capacity error the constructor would raise.
            return Err(ContentsError::DebrisOverflow {
                debris: debris_len,
                free: VOXEL_EIGHTHS.saturating_sub(shape.capacity()),
            });
        }
        for (slot, &r) in mats.iter_mut().zip(raw) {
            *slot = MaterialId::from_raw(r).ok_or(ContentsError::UnknownMaterial(r))?;
        }
        let (a, b) = (
            structure_len as usize,
            structure_len as usize + pore_len as usize,
        );
        if !(mats[..a].is_sorted() && mats[a..b].is_sorted() && mats[b..total].is_sorted()) {
            return Err(ContentsError::NotCanonical);
        }
        let value = Self::new(shape, &mats[..a], &mats[a..b], &mats[b..total])?;
        Ok((value, remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        assert!(VoxelContents::EMPTY.is_empty());
        assert_eq!(VoxelContents::EMPTY.solid_eighths(), 0);
        assert_eq!(
            VoxelContents::debris_only(&[]).unwrap(),
            VoxelContents::EMPTY
        );
    }

    #[test]
    fn canonical_form_is_order_independent() {
        let a = VoxelContents::debris_only(&[
            MaterialId::GRAVEL,
            MaterialId::SAND,
            MaterialId::SNOW,
            MaterialId::SAND,
        ])
        .unwrap();
        let b = VoxelContents::debris_only(&[
            MaterialId::SAND,
            MaterialId::SNOW,
            MaterialId::SAND,
            MaterialId::GRAVEL,
        ])
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(
            a.debris(),
            &[
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::GRAVEL,
                MaterialId::SNOW
            ]
        );
        // Hash equality follows from representation equality (derived Hash);
        // spot-check via a HashSet.
        let mut set = std::collections::HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn segments_are_independent_multisets() {
        // The same material in different roles is NOT merged across segments.
        let c = VoxelContents::new(
            StructureShape::Slab,
            &[MaterialId::SCREE, MaterialId::GRAVEL],
            &[MaterialId::SILT],
            &[MaterialId::SILT, MaterialId::SAND],
        )
        .unwrap();
        assert_eq!(c.structure(), &[MaterialId::GRAVEL, MaterialId::SCREE]);
        assert_eq!(c.pore_fill(), &[MaterialId::SILT]);
        assert_eq!(c.debris(), &[MaterialId::SAND, MaterialId::SILT]);
        assert_eq!(c.solid_eighths(), 5);
        assert_eq!(c.open_pores(), 1);
        assert_eq!(c.free_debris_eighths(), 2);
        assert_eq!(c.structure_density(), Some(0.5));
    }

    #[test]
    fn capacity_invariants_are_enforced() {
        let s = MaterialId::SCREE;
        // Structure beyond shape capacity.
        assert_eq!(
            VoxelContents::new(StructureShape::Quarter, &[s, s, s], &[], &[]),
            Err(ContentsError::StructureOverflow {
                fill: 3,
                capacity: 2
            })
        );
        // Pore fill beyond open pores (slab cap 4, 3 filled -> 1 pore).
        assert_eq!(
            VoxelContents::new(
                StructureShape::Slab,
                &[s, s, s],
                &[MaterialId::SILT, MaterialId::SILT],
                &[]
            ),
            Err(ContentsError::PoreOverflow { fill: 2, pores: 1 })
        );
        // Debris beyond unreserved volume (slab reserves 4 -> 4 free).
        let d = [MaterialId::SAND; 5];
        assert_eq!(
            VoxelContents::new(StructureShape::Slab, &[], &[], &d),
            Err(ContentsError::DebrisOverflow { debris: 5, free: 4 })
        );
        // A full shape admits no debris at all.
        assert!(VoxelContents::new(StructureShape::Full, &[s; 8], &[], &[]).is_ok());
        assert_eq!(
            VoxelContents::new(StructureShape::Full, &[s; 8], &[], &[MaterialId::SAND]),
            Err(ContentsError::DebrisOverflow { debris: 1, free: 0 })
        );
    }

    #[test]
    fn encode_decode_roundtrip() {
        let cases = [
            VoxelContents::EMPTY,
            VoxelContents::debris_only(&[MaterialId::SNOW; 8]).unwrap(),
            VoxelContents::new(
                StructureShape::Slab,
                &[MaterialId::SCREE],
                &[MaterialId::CLAY, MaterialId::SILT],
                &[MaterialId::BONE, MaterialId::ASH, MaterialId::POTSHERD],
            )
            .unwrap(),
            VoxelContents::new(
                StructureShape::Quarter,
                &[MaterialId::GRAVEL],
                &[],
                &[MaterialId::LEAF_LITTER, MaterialId::LOAM],
            )
            .unwrap(),
        ];
        for c in cases {
            let mut bytes = vec![0xAA]; // preceding noise survives untouched
            c.encode_into(&mut bytes);
            bytes.push(0xBB); // trailing bytes are returned, not consumed
            assert_eq!(bytes.len(), 2 + c.encoded_len());
            let (back, rest) = VoxelContents::decode(&bytes[1..]).expect("roundtrip");
            assert_eq!(back, c);
            assert_eq!(rest, &[0xBB]);
        }
    }

    #[test]
    fn decode_rejects_bad_input() {
        // Truncated header / body.
        assert_eq!(VoxelContents::decode(&[]), Err(ContentsError::Truncated));
        assert_eq!(VoxelContents::decode(&[0]), Err(ContentsError::Truncated));
        let mut bytes = Vec::new();
        VoxelContents::debris_only(&[MaterialId::SAND, MaterialId::SNOW])
            .unwrap()
            .encode_into(&mut bytes);
        assert_eq!(
            VoxelContents::decode(&bytes[..bytes.len() - 1]),
            Err(ContentsError::Truncated)
        );
        // Unknown material id.
        let mut bad = bytes.clone();
        bad[2] = 200;
        assert_eq!(
            VoxelContents::decode(&bad),
            Err(ContentsError::UnknownMaterial(200))
        );
        // Non-canonical (unsorted) segment: snow before sand.
        let mut bad = bytes.clone();
        bad.swap(2, 3);
        assert_eq!(
            VoxelContents::decode(&bad),
            Err(ContentsError::NotCanonical)
        );
        // Header claiming more eighths than fit.
        let overfull = [0b0000_0000, 0x88]; // shape none, pore 8, debris 8
        assert!(VoxelContents::decode(&overfull).is_err());
    }
}
