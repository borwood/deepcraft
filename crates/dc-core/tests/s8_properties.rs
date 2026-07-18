//! S8 property tests: randomized (seeded, deterministic — no ambient
//! entropy) sweeps over the materials model. "Property-based" in the
//! hand-rolled style: a splitmix64 generator drives hundreds of cases per
//! property; failures print the case seed.

use dc_core::materials::contents::{StructureShape, VoxelContents};
use dc_core::materials::extract::extraction_sequence;
use dc_core::materials::intern::{MaterialChunk, MixtureId, MixtureTable};
use dc_core::materials::stratify::{STRATIFY_FULL_TIME, stratify};
use dc_core::materials::{DamageType, MATERIAL_COUNT, MaterialId};
use dc_core::{CHUNK_VOLUME, Chunk, ChunkContainer, PalettedChunk};

const CASES: u64 = 500;

/// splitmix64 stream (same construction as dc-sim's addressed draws).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn material(&mut self) -> MaterialId {
        MaterialId::from_raw(self.below(MATERIAL_COUNT) as u8).expect("in range")
    }

    fn materials(&mut self, n: usize) -> Vec<MaterialId> {
        (0..n).map(|_| self.material()).collect()
    }
}

/// A random valid VoxelContents: any shape, any legal fill levels.
fn arbitrary_contents(rng: &mut Rng) -> VoxelContents {
    let shape = match rng.below(4) {
        0 => StructureShape::None,
        1 => StructureShape::Quarter,
        2 => StructureShape::Slab,
        _ => StructureShape::Full,
    };
    let cap = shape.capacity() as usize;
    let structure_n = rng.below(cap + 1);
    let pore_n = rng.below(cap - structure_n + 1);
    let debris_n = rng.below(8 - cap + 1);
    let structure = rng.materials(structure_n);
    let pore = rng.materials(pore_n);
    let debris = rng.materials(debris_n);
    VoxelContents::new(shape, &structure, &pore, &debris).expect("legal fill levels")
}

/// Fisher-Yates with the test rng.
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, rng.below(i + 1));
    }
}

#[test]
fn canonical_form_is_permutation_invariant() {
    let mut rng = Rng::new(0x51);
    for case in 0..CASES {
        let shape = match rng.below(4) {
            0 => StructureShape::None,
            1 => StructureShape::Quarter,
            2 => StructureShape::Slab,
            _ => StructureShape::Full,
        };
        let cap = shape.capacity() as usize;
        let structure_n = rng.below(cap + 1);
        let mut structure = rng.materials(structure_n);
        let pore_n = rng.below(cap - structure.len() + 1);
        let mut pore = rng.materials(pore_n);
        let debris_n = rng.below(8 - cap + 1);
        let mut debris = rng.materials(debris_n);
        let a = VoxelContents::new(shape, &structure, &pore, &debris).unwrap();
        shuffle(&mut rng, &mut structure);
        shuffle(&mut rng, &mut pore);
        shuffle(&mut rng, &mut debris);
        let b = VoxelContents::new(shape, &structure, &pore, &debris).unwrap();
        assert_eq!(a, b, "case {case}: canonical form must ignore input order");
    }
}

#[test]
fn encoding_roundtrips_and_is_canonical() {
    let mut rng = Rng::new(0x52);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        let mut bytes = Vec::new();
        c.encode_into(&mut bytes);
        assert_eq!(bytes.len(), c.encoded_len(), "case {case}");
        let (back, rest) = VoxelContents::decode(&bytes).expect("roundtrip");
        assert!(rest.is_empty(), "case {case}");
        assert_eq!(back, c, "case {case}");
        // Equal values, equal bytes (encoding is a function of the value).
        let mut again = Vec::new();
        back.encode_into(&mut again);
        assert_eq!(again, bytes, "case {case}");
    }
}

#[test]
fn extraction_conserves_mass() {
    let mut rng = Rng::new(0x53);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        for damage in DamageType::ALL {
            let seq = extraction_sequence(&c, damage);
            let yielded: u32 = seq.iter().map(|y| u32::from(y.eighths)).sum();
            let eligible = u32::from(c.debris().len() as u8)
                + u32::from(c.pore_fill().len() as u8)
                + if damage.targets_structure() {
                    c.structure().len() as u32
                } else {
                    0
                };
            assert_eq!(
                yielded, eligible,
                "case {case}, {damage:?}: extraction must conserve mass over eligible pools"
            );
        }
    }
}

#[test]
fn extraction_order_ascends_resistance_with_id_tiebreak() {
    let mut rng = Rng::new(0x54);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        for damage in DamageType::ALL {
            let seq = extraction_sequence(&c, damage);
            for pair in seq.windows(2) {
                let (a, b) = (pair[0].material, pair[1].material);
                let (ra, rb) = (
                    a.props().resistance(damage),
                    b.props().resistance(damage),
                );
                assert!(
                    ra < rb || (ra == rb && a < b),
                    "case {case}, {damage:?}: {} (r={ra}) must precede {} (r={rb})",
                    a.props().name,
                    b.props().name
                );
            }
            // Each material appears at most once (pooled).
            let mut seen = std::collections::HashSet::new();
            for y in &seq {
                assert!(seen.insert(y.material), "case {case}: duplicate yield");
                assert!(y.eighths > 0, "case {case}: zero-mass yield");
            }
        }
    }
}

#[test]
fn sieve_separates_by_grain_size() {
    let mut rng = Rng::new(0x55);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        let seq = extraction_sequence(&c, DamageType::Sieve);
        for pair in seq.windows(2) {
            assert!(
                pair[0].material.props().grain_size_mm <= pair[1].material.props().grain_size_mm,
                "case {case}: sieve must yield fines before coarse"
            );
        }
    }
}

#[test]
fn debris_damage_never_yields_structure_slots() {
    let mut rng = Rng::new(0x56);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        // Total per material across the loose pools only.
        let mut loose: std::collections::HashMap<MaterialId, u32> = std::collections::HashMap::new();
        for &m in c.debris().iter().chain(c.pore_fill()) {
            *loose.entry(m).or_insert(0) += 1;
        }
        for damage in [DamageType::Dig, DamageType::Cut, DamageType::Sieve] {
            for y in extraction_sequence(&c, damage) {
                assert_eq!(
                    u32::from(y.eighths),
                    loose.get(&y.material).copied().unwrap_or(0),
                    "case {case}, {damage:?}: yield must equal the loose count \
                     (structure slots excluded) for {}",
                    y.material.props().name
                );
            }
        }
    }
}

#[test]
fn stratification_is_deterministic_monotone_and_conserving() {
    let mut rng = Rng::new(0x57);
    let times = [0u64, 50, 200, 900, 1_000, 3_000, 9_000, 15_999, 16_000, u64::MAX];
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        let n = c.debris().len() as u32;
        let mut prev_settled = 0u32;
        let mut prev_expansion: Vec<MaterialId> = Vec::new();
        for &t in &times {
            let view = stratify(&c, t);
            assert_eq!(view, stratify(&c, t), "case {case}: deterministic");
            assert_eq!(view.total_eighths(), n, "case {case}: mass conserved");
            let settled: u32 = view.settled.iter().map(|b| u32::from(b.eighths)).sum();
            assert!(settled >= prev_settled, "case {case}: monotone at t={t}");
            let expansion: Vec<MaterialId> = view
                .settled
                .iter()
                .flat_map(|b| std::iter::repeat_n(b.material, usize::from(b.eighths)))
                .collect();
            assert!(
                expansion.starts_with(&prev_expansion),
                "case {case}: settled bands must refine, not rearrange (t={t})"
            );
            // Bands are density-sorted, densest first.
            for pair in view.settled.windows(2) {
                assert!(
                    pair[0].material.props().density_kg_m3
                        >= pair[1].material.props().density_kg_m3,
                    "case {case}: settled bands must descend in density"
                );
            }
            prev_settled = settled;
            prev_expansion = expansion;
        }
        // Fully banded at/after the saturation time.
        let full = stratify(&c, STRATIFY_FULL_TIME);
        assert!(full.mixed.is_empty(), "case {case}: saturates to full bands");
    }
}

#[test]
fn interning_identifies_exactly_the_equal_contents() {
    let mut rng = Rng::new(0x58);
    let mut table = MixtureTable::new();
    let mut by_value: std::collections::HashMap<VoxelContents, MixtureId> =
        std::collections::HashMap::new();
    by_value.insert(VoxelContents::EMPTY, MixtureId::EMPTY);
    for case in 0..CASES {
        let c = arbitrary_contents(&mut rng);
        let id = table.intern(c);
        match by_value.get(&c) {
            Some(&expected) => assert_eq!(id, expected, "case {case}: stable id"),
            None => {
                by_value.insert(c, id);
            }
        }
        assert_eq!(table.get(id), Some(&c), "case {case}: id resolves back");
    }
    assert_eq!(table.len(), by_value.len());
    // Serialization preserves ids exactly.
    let back = MixtureTable::decode(&table.encode()).expect("table roundtrip");
    for (c, id) in &by_value {
        assert_eq!(back.get(*id), Some(c));
    }
}

#[test]
fn material_sidecar_roundtrips_through_the_container() {
    let mut rng = Rng::new(0x59);
    let mut table = MixtureTable::new();
    // A random sparse chunk: ~1000 voxels of random mixtures.
    let mut ids = vec![MixtureId::EMPTY; CHUNK_VOLUME];
    for _ in 0..1000 {
        let at = rng.below(CHUNK_VOLUME);
        ids[at] = table.intern(arbitrary_contents(&mut rng));
    }
    let materials = MaterialChunk::from_dense(&ids);
    materials.validate().expect("fresh chunk validates");
    materials.validate_against(&table).expect("ids resolve");

    // Ride the format-v1 container next to a plain block payload.
    let mut base = Chunk::new();
    base.set(0, 0, 0, dc_core::Block::Stone);
    let mut container = ChunkContainer::new(PalettedChunk::from_dense(&base));
    let baseline_bytes = container.encode().len();
    if let Some(sidecar) = materials.to_sidecar() {
        container.sidecars.push(sidecar);
    }
    let bytes = container.encode();
    assert!(bytes.len() > baseline_bytes, "sidecar occupies real bytes");

    let decoded = ChunkContainer::decode(&bytes).expect("container decodes");
    let restored = MaterialChunk::from_container(&decoded)
        .expect("sidecar decodes")
        .expect("sidecar present");
    assert_eq!(restored, materials);
    // Spot-check voxel-level agreement through the whole pipeline.
    for _ in 0..64 {
        let at = rng.below(CHUNK_VOLUME);
        let (x, y, z) = (at % 32, at / 1024, (at / 32) % 32);
        assert_eq!(restored.get(x, y, z), ids[Chunk::index(x, y, z)]);
    }
    // And the base voxel payload is untouched by the materials sidecar.
    assert_eq!(decoded.voxels.get(0, 0, 0), dc_core::Block::Stone);
}
