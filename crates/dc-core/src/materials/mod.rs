//! Material volume model (spike S8).
//!
//! Design: docs/design/materials.md. A voxel is 8 volume-eighths; each eighth
//! holds one granular material (or nothing). This module provides:
//!
//! - material identity + property sheet and a small hardcoded registry
//!   (data-driven *shape*, prototype values — the real registry arrives with
//!   content packs);
//! - [`contents::VoxelContents`]: the structure / pore-fill / debris eighths
//!   model in canonical (multiset-sorted) form;
//! - [`intern::MixtureTable`] + [`intern::MaterialChunk`]: region-level
//!   interning of distinct mixture states, chunk-level palette-compressed
//!   references, serialized as the `"materials/slots-v0"` format-v1 sidecar;
//! - [`extract`]: typed-damage extraction ordering;
//! - [`stratify`]: derived (never ticked) stratification;
//! - [`lod`]: the mixed-voxel LOD downsample rule.
//!
//! Storage feasibility measurements and the free-form vs curated-recipe
//! verdict live in docs/spikes/S8-results.md.

pub mod contents;
pub mod extract;
pub mod intern;
pub mod lod;
pub mod stratify;

use serde::{Deserialize, Serialize};

/// Number of materials in the prototype registry.
pub const MATERIAL_COUNT: usize = 12;

/// Identifier of a granular material in the registry. `u8`-sized: a material
/// id appears up to 8 times per voxel, so entry compactness matters more than
/// ceiling here; the eventual data-driven registry can widen it behind this
/// type.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize,
)]
pub struct MaterialId(u8);

impl MaterialId {
    pub const SAND: MaterialId = MaterialId(0);
    pub const GRAVEL: MaterialId = MaterialId(1);
    pub const SNOW: MaterialId = MaterialId(2);
    pub const LEAF_LITTER: MaterialId = MaterialId(3);
    pub const CLAY: MaterialId = MaterialId(4);
    pub const SILT: MaterialId = MaterialId(5);
    pub const POTSHERD: MaterialId = MaterialId(6);
    pub const KNAPPING_DEBRIS: MaterialId = MaterialId(7);
    pub const ASH: MaterialId = MaterialId(8);
    pub const LOAM: MaterialId = MaterialId(9);
    pub const SCREE: MaterialId = MaterialId(10);
    pub const BONE: MaterialId = MaterialId(11);

    /// A registry-valid id from its raw value; `None` when out of range.
    #[inline]
    pub const fn from_raw(raw: u8) -> Option<MaterialId> {
        if (raw as usize) < MATERIAL_COUNT {
            Some(MaterialId(raw))
        } else {
            None
        }
    }

    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// This material's property sheet.
    #[inline]
    pub fn props(self) -> &'static MaterialProps {
        &REGISTRY[self.0 as usize]
    }

    /// Every registered material, in id order.
    pub fn all() -> impl Iterator<Item = MaterialId> {
        (0..MATERIAL_COUNT as u8).map(MaterialId)
    }
}

/// Typed block damage. Sustained application of one type yields materials in
/// ascending extraction resistance under that type (see [`extract`]).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum DamageType {
    Dig,
    Chop,
    Smash,
    Cut,
    Sieve,
}

impl DamageType {
    pub const ALL: [DamageType; 5] = [
        DamageType::Dig,
        DamageType::Chop,
        DamageType::Smash,
        DamageType::Cut,
        DamageType::Sieve,
    ];

    /// Does this damage type act on *structure* slots? Debris-type damage
    /// (dig, cut, sieve) only ever yields loose material — debris and pore
    /// fill; structural fill requires smash or chop.
    #[inline]
    pub const fn targets_structure(self) -> bool {
        matches!(self, DamageType::Smash | DamageType::Chop)
    }

    #[inline]
    const fn index(self) -> usize {
        match self {
            DamageType::Dig => 0,
            DamageType::Chop => 1,
            DamageType::Smash => 2,
            DamageType::Cut => 3,
            DamageType::Sieve => 4,
        }
    }
}

/// Property sheet for a granular material (docs/design/materials.md
/// § Granular material property sheet). Prototype values; the shape of the
/// data is the deliverable, the numbers are placeholders for tuning.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialProps {
    pub name: &'static str,
    /// Bulk density in kg/m^3 — stratification sort key and weight.
    pub density_kg_m3: f32,
    /// Characteristic grain size in mm — what fits into which pores; sieving.
    pub grain_size_mm: f32,
    /// Cohesion in `[0, 1]` — angle of repose, slumping.
    pub cohesion: f32,
    /// Per-damage-type extraction resistance, indexed by [`DamageType::index`].
    /// Lower = yielded sooner under sustained damage of that type.
    ///
    /// Registry invariant (tested): sieve resistance equals grain size, so
    /// sieving always separates fines-first by construction.
    pub extraction_resistance: [f32; 5],
    /// Permeability contribution in `[0, 1]` when packed into pores.
    pub permeability: f32,
    /// Insulation contribution in `[0, 1]` when packed into pores.
    pub insulation: f32,
}

impl MaterialProps {
    /// Extraction resistance under one damage type.
    #[inline]
    pub fn resistance(&self, damage: DamageType) -> f32 {
        self.extraction_resistance[damage.index()]
    }
}

/// resistances are `[dig, chop, smash, cut, sieve]`; sieve = grain size.
const REGISTRY: [MaterialProps; MATERIAL_COUNT] = [
    MaterialProps {
        name: "sand",
        density_kg_m3: 1600.0,
        grain_size_mm: 0.5,
        cohesion: 0.05,
        extraction_resistance: [1.0, 6.0, 4.0, 5.0, 0.5],
        permeability: 0.55,
        insulation: 0.25,
    },
    MaterialProps {
        name: "gravel",
        density_kg_m3: 1800.0,
        grain_size_mm: 20.0,
        cohesion: 0.02,
        extraction_resistance: [1.6, 7.0, 3.0, 6.0, 20.0],
        permeability: 0.8,
        insulation: 0.15,
    },
    MaterialProps {
        name: "snow",
        density_kg_m3: 300.0,
        grain_size_mm: 1.0,
        cohesion: 0.3,
        extraction_resistance: [0.4, 5.0, 1.5, 2.0, 1.0],
        permeability: 0.35,
        insulation: 0.85,
    },
    MaterialProps {
        name: "leaf-litter",
        density_kg_m3: 150.0,
        grain_size_mm: 25.0,
        cohesion: 0.15,
        extraction_resistance: [0.2, 1.0, 1.2, 0.8, 25.0],
        permeability: 0.6,
        insulation: 0.7,
    },
    MaterialProps {
        name: "clay",
        density_kg_m3: 1750.0,
        grain_size_mm: 0.002,
        cohesion: 0.9,
        extraction_resistance: [2.6, 4.5, 3.5, 2.2, 0.002],
        permeability: 0.05,
        insulation: 0.4,
    },
    MaterialProps {
        name: "silt",
        density_kg_m3: 1500.0,
        grain_size_mm: 0.02,
        cohesion: 0.5,
        extraction_resistance: [1.4, 4.0, 3.2, 2.0, 0.02],
        permeability: 0.15,
        insulation: 0.35,
    },
    MaterialProps {
        name: "potsherd",
        density_kg_m3: 1900.0,
        grain_size_mm: 40.0,
        cohesion: 0.0,
        extraction_resistance: [2.0, 5.5, 1.0, 4.0, 40.0],
        permeability: 0.85,
        insulation: 0.2,
    },
    MaterialProps {
        name: "knapping-debris",
        density_kg_m3: 2300.0,
        grain_size_mm: 15.0,
        cohesion: 0.0,
        extraction_resistance: [1.8, 6.5, 2.5, 5.5, 15.0],
        permeability: 0.8,
        insulation: 0.1,
    },
    MaterialProps {
        name: "ash",
        density_kg_m3: 700.0,
        grain_size_mm: 0.05,
        cohesion: 0.1,
        extraction_resistance: [0.6, 3.0, 2.8, 1.6, 0.05],
        permeability: 0.3,
        insulation: 0.6,
    },
    MaterialProps {
        name: "loam",
        density_kg_m3: 1300.0,
        grain_size_mm: 0.1,
        cohesion: 0.35,
        extraction_resistance: [0.9, 3.5, 3.0, 1.8, 0.1],
        permeability: 0.4,
        insulation: 0.45,
    },
    MaterialProps {
        name: "scree",
        density_kg_m3: 2000.0,
        grain_size_mm: 100.0,
        cohesion: 0.05,
        extraction_resistance: [2.4, 8.0, 3.8, 7.0, 100.0],
        permeability: 0.9,
        insulation: 0.1,
    },
    MaterialProps {
        name: "bone",
        density_kg_m3: 1100.0,
        grain_size_mm: 60.0,
        cohesion: 0.0,
        extraction_resistance: [1.2, 2.0, 1.1, 1.5, 60.0],
        permeability: 0.75,
        insulation: 0.3,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_roundtrip_and_names_are_distinct() {
        let mut names = std::collections::HashSet::new();
        for (i, m) in MaterialId::all().enumerate() {
            assert_eq!(m.raw() as usize, i);
            assert_eq!(MaterialId::from_raw(m.raw()), Some(m));
            assert!(names.insert(m.props().name), "duplicate name");
        }
        assert_eq!(MaterialId::all().count(), MATERIAL_COUNT);
        assert_eq!(MaterialId::from_raw(MATERIAL_COUNT as u8), None);
    }

    #[test]
    fn sieve_resistance_equals_grain_size() {
        // The registry invariant that makes "sieve separates by grain size"
        // and "yield ascends by resistance" the same statement.
        for m in MaterialId::all() {
            let p = m.props();
            assert_eq!(
                p.resistance(DamageType::Sieve),
                p.grain_size_mm,
                "{}",
                p.name
            );
        }
    }

    #[test]
    fn properties_are_sane() {
        for m in MaterialId::all() {
            let p = m.props();
            assert!(p.density_kg_m3 > 0.0);
            assert!(p.grain_size_mm > 0.0);
            assert!((0.0..=1.0).contains(&p.cohesion), "{}", p.name);
            assert!((0.0..=1.0).contains(&p.permeability), "{}", p.name);
            assert!((0.0..=1.0).contains(&p.insulation), "{}", p.name);
            for r in p.extraction_resistance {
                assert!(r > 0.0 && r.is_finite());
            }
        }
    }

    #[test]
    fn debris_damage_types_do_not_target_structure() {
        assert!(!DamageType::Dig.targets_structure());
        assert!(!DamageType::Cut.targets_structure());
        assert!(!DamageType::Sieve.targets_structure());
        assert!(DamageType::Smash.targets_structure());
        assert!(DamageType::Chop.targets_structure());
    }
}
