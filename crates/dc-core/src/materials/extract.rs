//! Typed-damage extraction ordering (docs/design/materials.md § Extraction).
//!
//! Sustained application of one [`DamageType`] to a voxel yields its
//! materials in **ascending extraction resistance** under that type, ties
//! broken by ascending material id (deterministic). Debris-type damage (dig,
//! cut, sieve) only reaches loose material — debris and pore fill; structural
//! fill is yielded only by structure-targeting damage (smash, chop).
//!
//! Because the registry pins sieve resistance to grain size, sieving is
//! automatically a grain-size separator (fines first) — same rule, no special
//! case.
//!
//! Prototype scope: extraction here is lossless (tool-dependent destruction —
//! "a careless pick pulverizes the potsherds" — is game design layered on
//! this ordering later).

use std::collections::BTreeMap;

use super::contents::VoxelContents;
use super::{DamageType, MaterialId};

/// One step of an extraction sequence: `eighths` eighths of `material` come
/// out before anything with higher resistance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractionYield {
    pub material: MaterialId,
    pub eighths: u8,
}

/// The order in which sustained `damage` empties `contents`.
///
/// - Eligible pools: debris + pore fill always; structural fill only when
///   [`DamageType::targets_structure`].
/// - A material appearing in several eligible pools yields once, with the
///   pooled total (mixing across pools loses nothing the multiset had).
/// - Ordering: ascending `resistance(damage)`, ties by ascending material id.
pub fn extraction_sequence(contents: &VoxelContents, damage: DamageType) -> Vec<ExtractionYield> {
    let mut pool: BTreeMap<MaterialId, u8> = BTreeMap::new();
    let mut add = |materials: &[MaterialId]| {
        for &m in materials {
            *pool.entry(m).or_insert(0) += 1;
        }
    };
    add(contents.debris());
    add(contents.pore_fill());
    if damage.targets_structure() {
        add(contents.structure());
    }

    let mut order: Vec<(MaterialId, u8)> = pool.into_iter().collect();
    order.sort_by(|(ma, _), (mb, _)| {
        ma.props()
            .resistance(damage)
            .total_cmp(&mb.props().resistance(damage))
            .then(ma.cmp(mb))
    });
    order
        .into_iter()
        .map(|(material, eighths)| ExtractionYield { material, eighths })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::contents::StructureShape;

    #[test]
    fn dig_yields_ascending_dig_resistance() {
        // leaf (0.2) then sand (1.0) then gravel (1.6): the design's example.
        let c = VoxelContents::debris_only(&[
            MaterialId::GRAVEL,
            MaterialId::SAND,
            MaterialId::LEAF_LITTER,
            MaterialId::SAND,
            MaterialId::GRAVEL,
        ])
        .unwrap();
        let seq = extraction_sequence(&c, DamageType::Dig);
        assert_eq!(
            seq,
            vec![
                ExtractionYield {
                    material: MaterialId::LEAF_LITTER,
                    eighths: 1
                },
                ExtractionYield {
                    material: MaterialId::SAND,
                    eighths: 2
                },
                ExtractionYield {
                    material: MaterialId::GRAVEL,
                    eighths: 2
                },
            ]
        );
    }

    #[test]
    fn sieve_orders_by_grain_size_fines_first() {
        let c = VoxelContents::debris_only(&[
            MaterialId::GRAVEL,
            MaterialId::CLAY,
            MaterialId::SAND,
            MaterialId::ASH,
        ])
        .unwrap();
        let seq = extraction_sequence(&c, DamageType::Sieve);
        let order: Vec<MaterialId> = seq.iter().map(|y| y.material).collect();
        assert_eq!(
            order,
            vec![
                MaterialId::CLAY,   // 0.002 mm
                MaterialId::ASH,    // 0.05 mm
                MaterialId::SAND,   // 0.5 mm
                MaterialId::GRAVEL, // 20 mm
            ]
        );
    }

    #[test]
    fn debris_damage_never_reaches_structure() {
        // Scree structure, silt in the pores, sand debris.
        let c = VoxelContents::new(
            StructureShape::Slab,
            &[MaterialId::SCREE, MaterialId::SCREE],
            &[MaterialId::SILT],
            &[MaterialId::SAND, MaterialId::SAND],
        )
        .unwrap();
        for damage in [DamageType::Dig, DamageType::Cut, DamageType::Sieve] {
            let seq = extraction_sequence(&c, damage);
            assert!(
                seq.iter().all(|y| y.material != MaterialId::SCREE),
                "{damage:?} must not yield structural fill"
            );
            let total: u32 = seq.iter().map(|y| u32::from(y.eighths)).sum();
            assert_eq!(total, 3, "{damage:?} yields exactly the loose eighths");
        }
        // Smash reaches everything.
        let seq = extraction_sequence(&c, DamageType::Smash);
        let total: u32 = seq.iter().map(|y| u32::from(y.eighths)).sum();
        assert_eq!(total, 5);
        assert!(seq.iter().any(|y| y.material == MaterialId::SCREE));
    }

    #[test]
    fn same_material_across_pools_is_pooled() {
        // Silt both packed in pores and lying as debris: one yield entry.
        let c = VoxelContents::new(
            StructureShape::Quarter,
            &[MaterialId::GRAVEL],
            &[MaterialId::SILT],
            &[MaterialId::SILT, MaterialId::SILT],
        )
        .unwrap();
        let seq = extraction_sequence(&c, DamageType::Dig);
        let silt: Vec<_> = seq
            .iter()
            .filter(|y| y.material == MaterialId::SILT)
            .collect();
        assert_eq!(silt.len(), 1);
        assert_eq!(silt[0].eighths, 3);
    }

    #[test]
    fn empty_contents_yield_nothing() {
        for damage in DamageType::ALL {
            assert!(extraction_sequence(&VoxelContents::EMPTY, damage).is_empty());
        }
    }
}
