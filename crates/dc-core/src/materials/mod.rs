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
//! - [`geology`]: content classes as contracts + deterministic member
//!   selection (the geology backbone's typed model);
//! - [`stratify`]: derived (never ticked) stratification;
//! - [`lod`]: the mixed-voxel LOD downsample rule.
//!
//! Storage feasibility measurements and the free-form vs curated-recipe
//! verdict live in docs/spikes/S8-results.md.

pub mod contents;
pub mod extract;
pub mod geology;
pub mod intern;
pub mod lod;
pub mod stratify;

use serde::{Deserialize, Serialize};

/// Number of materials in the prototype registry (12 S8 debris materials +
/// the 5-entry v1 geology set; the table widens behind `MaterialId`, the
/// type does not).
pub const MATERIAL_COUNT: usize = 17;

/// Identifier of a granular material in the registry. `u8`-sized: a material
/// id appears up to 8 times per voxel, so entry compactness matters more than
/// ceiling here; the eventual data-driven registry can widen it behind this
/// type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
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
    // --- v1 geology set (docs/design/geology.md, DECIDED 2026-07-18) ---
    /// Clastic sediment, fine (lithified mud/silt).
    pub const MUDSTONE: MaterialId = MaterialId(12);
    /// Clastic sediment, coarse (lithified sand).
    pub const SANDSTONE: MaterialId = MaterialId(13);
    /// Igneous intrusive (coarse-crystalline basement).
    pub const GRANITE: MaterialId = MaterialId(14);
    /// Igneous extrusive (fine-crystalline surface flows).
    pub const BASALT: MaterialId = MaterialId(15);
    /// Placer ore mineral: a dense grain that sorts with the coarse fraction
    /// despite its small size — the placer mechanism in one property sheet.
    pub const GOLD_DUST: MaterialId = MaterialId(16);

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
    /// Linear-RGB base albedo in `[0, 1]³` — the interim vertex-color source
    /// for mixture dithering (docs/design/visuals.md § Mixture rendering road,
    /// DECIDED 2026-07-19). Pack data: materials with a block twin match the
    /// client block palette; the loose granular set carries placeholder colors
    /// shared with `tools/gen_placeholder_textures.py`. The later splat pipeline
    /// replaces this color *source* without disturbing any data plumbing.
    pub albedo: [f32; 3],
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
        albedo: [0.80, 0.72, 0.52],
        density_kg_m3: 1600.0,
        grain_size_mm: 0.5,
        cohesion: 0.05,
        extraction_resistance: [1.0, 6.0, 4.0, 5.0, 0.5],
        permeability: 0.55,
        insulation: 0.25,
    },
    MaterialProps {
        name: "gravel",
        albedo: [0.50, 0.48, 0.45],
        density_kg_m3: 1800.0,
        grain_size_mm: 20.0,
        cohesion: 0.02,
        extraction_resistance: [1.6, 7.0, 3.0, 6.0, 20.0],
        permeability: 0.8,
        insulation: 0.15,
    },
    MaterialProps {
        name: "snow",
        albedo: [0.92, 0.94, 0.98],
        density_kg_m3: 300.0,
        grain_size_mm: 1.0,
        cohesion: 0.3,
        extraction_resistance: [0.4, 5.0, 1.5, 2.0, 1.0],
        permeability: 0.35,
        insulation: 0.85,
    },
    MaterialProps {
        name: "leaf-litter",
        albedo: [0.40, 0.30, 0.14],
        density_kg_m3: 150.0,
        grain_size_mm: 25.0,
        cohesion: 0.15,
        extraction_resistance: [0.2, 1.0, 1.2, 0.8, 25.0],
        permeability: 0.6,
        insulation: 0.7,
    },
    MaterialProps {
        name: "clay",
        albedo: [0.62, 0.48, 0.38],
        density_kg_m3: 1750.0,
        grain_size_mm: 0.002,
        cohesion: 0.9,
        extraction_resistance: [2.6, 4.5, 3.5, 2.2, 0.002],
        permeability: 0.05,
        insulation: 0.4,
    },
    MaterialProps {
        name: "silt",
        albedo: [0.58, 0.50, 0.38],
        density_kg_m3: 1500.0,
        grain_size_mm: 0.02,
        cohesion: 0.5,
        extraction_resistance: [1.4, 4.0, 3.2, 2.0, 0.02],
        permeability: 0.15,
        insulation: 0.35,
    },
    MaterialProps {
        name: "potsherd",
        albedo: [0.60, 0.34, 0.24],
        density_kg_m3: 1900.0,
        grain_size_mm: 40.0,
        cohesion: 0.0,
        extraction_resistance: [2.0, 5.5, 1.0, 4.0, 40.0],
        permeability: 0.85,
        insulation: 0.2,
    },
    MaterialProps {
        name: "knapping-debris",
        albedo: [0.42, 0.42, 0.46],
        density_kg_m3: 2300.0,
        grain_size_mm: 15.0,
        cohesion: 0.0,
        extraction_resistance: [1.8, 6.5, 2.5, 5.5, 15.0],
        permeability: 0.8,
        insulation: 0.1,
    },
    MaterialProps {
        name: "ash",
        albedo: [0.32, 0.31, 0.30],
        density_kg_m3: 700.0,
        grain_size_mm: 0.05,
        cohesion: 0.1,
        extraction_resistance: [0.6, 3.0, 2.8, 1.6, 0.05],
        permeability: 0.3,
        insulation: 0.6,
    },
    MaterialProps {
        name: "loam",
        albedo: [0.36, 0.26, 0.17],
        density_kg_m3: 1300.0,
        grain_size_mm: 0.1,
        cohesion: 0.35,
        extraction_resistance: [0.9, 3.5, 3.0, 1.8, 0.1],
        permeability: 0.4,
        insulation: 0.45,
    },
    MaterialProps {
        name: "scree",
        albedo: [0.48, 0.46, 0.44],
        density_kg_m3: 2000.0,
        grain_size_mm: 100.0,
        cohesion: 0.05,
        extraction_resistance: [2.4, 8.0, 3.8, 7.0, 100.0],
        permeability: 0.9,
        insulation: 0.1,
    },
    MaterialProps {
        name: "bone",
        albedo: [0.86, 0.82, 0.70],
        density_kg_m3: 1100.0,
        grain_size_mm: 60.0,
        cohesion: 0.0,
        extraction_resistance: [1.2, 2.0, 1.1, 1.5, 60.0],
        permeability: 0.75,
        insulation: 0.3,
    },
    MaterialProps {
        name: "mudstone",
        albedo: [0.46, 0.26, 0.20],
        density_kg_m3: 2400.0,
        grain_size_mm: 0.004,
        cohesion: 0.95,
        extraction_resistance: [3.2, 5.0, 4.2, 2.8, 0.004],
        permeability: 0.02,
        insulation: 0.4,
    },
    MaterialProps {
        name: "sandstone",
        albedo: [0.76, 0.66, 0.44],
        density_kg_m3: 2350.0,
        grain_size_mm: 0.3,
        cohesion: 0.85,
        extraction_resistance: [3.6, 6.5, 4.6, 5.2, 0.3],
        permeability: 0.35,
        insulation: 0.3,
    },
    MaterialProps {
        name: "granite",
        albedo: [0.66, 0.56, 0.58],
        density_kg_m3: 2700.0,
        grain_size_mm: 3.0,
        cohesion: 1.0,
        extraction_resistance: [6.0, 9.0, 5.5, 8.0, 3.0],
        permeability: 0.02,
        insulation: 0.2,
    },
    MaterialProps {
        name: "basalt",
        albedo: [0.14, 0.14, 0.16],
        density_kg_m3: 2900.0,
        grain_size_mm: 0.05,
        cohesion: 1.0,
        extraction_resistance: [5.5, 9.5, 5.0, 8.5, 0.05],
        permeability: 0.05,
        insulation: 0.2,
    },
    MaterialProps {
        name: "gold-dust",
        albedo: [0.80, 0.66, 0.28],
        density_kg_m3: 16000.0,
        grain_size_mm: 0.8,
        cohesion: 0.02,
        extraction_resistance: [1.1, 6.0, 4.4, 5.0, 0.8],
        permeability: 0.5,
        insulation: 0.15,
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
            for c in p.albedo {
                assert!((0.0..=1.0).contains(&c), "{} albedo {c}", p.name);
            }
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
