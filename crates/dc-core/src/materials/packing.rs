//! Pore packability — the single shared infiltration rule
//! (docs/design/materials.md § "Pore packability — DECIDED 2026-07-20").
//!
//! A fine material may be packed into a host structure's open pores only if
//! its grains are small enough to *enter* the pore throats rather than bridge
//! across them. This module owns that one rule as [`fits_in_pores`], consulted
//! by every **transport-time** depositing process (overflow packing,
//! groundwater infiltration, future ore deposition). [`super::contents::VoxelContents`]
//! deliberately does **not** call it — it stays pure volume accounting, so the
//! grain-fits-throat question lives here, once, and never leaks into the
//! representation invariants.
//!
//! **Exemption — genesis, not infiltration.** Formation-context emplacement
//! (worldgen-authored magmatic/diagenetic inclusion — olivine grown inside
//! basalt) bypasses this rule entirely: those crystals grew in place, so the
//! pore is representational, not a passage a grain had to fit through. The
//! geology emplacement path (`super::geology`) therefore never consults this
//! helper; see the note at the accessory member there.
//!
//! ## Expected consumers — the socket this rule waits in (S9)
//!
//! This module is DECIDED, built, and covered by seven tests, and **nothing in
//! production calls it** (`docs/audits/2026-07-22-seam-inventory.md` § S9;
//! `docs/spines.md` § 3, built-but-unconsumed). That is the *inverse* of the
//! "summary becomes an authority" defect: here the authority exists and the
//! seam is missing. The declaration below is what fixes it — so the next author
//! of an infiltration or cementation path **finds this rule and calls it**
//! instead of writing a second, divergent grain-fits-throat test (the user's own
//! point: "the stubbed APIs would be telling us right now what an unbuilt hydro
//! system is supposed to supply"). The two systems expected to call
//! [`fits_in_pores`] are:
//!
//! - **hydrology — groundwater infiltration.** A saturation/water-table pass
//!   (S11, `dc-worldgen/src/water/`, not yet on the production path) deposits
//!   fines transported by percolating water only where they can enter the host's
//!   pore throats. This is the transport-time gate on infiltration fill.
//! - **diagenesis — ore / cement deposition.** Pore-filling cement and
//!   precipitated ore ride the same rule: a mineral phase can occlude a rock's
//!   pores only if its grain (or nucleation habit) clears the throat. Distinct
//!   from *genesis* emplacement above, which is exempt because the crystal grew
//!   in place rather than infiltrating.
//!
//! No behaviour changes here; this is the missing declaration, not a new caller.

use super::MaterialId;

/// Pore-throat ratio: a filler grain packs into a host's pores only when it is
/// no larger than `K_PORE ×` the (finest) host grain.
///
/// Derivation (docs/design/materials.md § Pore packability): the interstitial
/// voids between packed grains admit an intruding grain of ~0.22–0.41 D
/// (D = host grain diameter), the range spanning loose-to-dense sphere
/// packings; the geotechnical **filter criterion** — the rule that stops fines
/// from piping through a coarse filter — independently lands at D/4–D/5.
/// `0.25` (= D/4) sits at the conservative overlap of both ranges: the throat
/// a grain must clear to infiltrate rather than bridge. One constant, no
/// per-material pore spectrum (that refinement is a listed open question).
pub const K_PORE: f32 = 0.25;

/// Can `filler` be packed into the pores of a host whose structural components
/// are `host_structure`?
///
/// True iff `filler_grain ≤ K_PORE × min(host grains)`. For heterogeneous
/// (rubble/breccia) fill the **finest** structural grain sets the throat: the
/// smallest grains choke the passages, so the minimum is the binding
/// constraint (docs/design/materials.md § Pore packability). The slice is the
/// host's structural multiset — exactly what
/// [`VoxelContents::structure`](super::contents::VoxelContents::structure)
/// returns — so a caller passes the structure straight through without
/// coupling this rule to the contents type.
///
/// # Edge cases (decided, docs/design/materials.md)
/// - **Empty `host_structure`** → `false`, never a panic. With no structural
///   grains present there is no throat to define, so there is nothing to fit
///   into; a conservative "no" is the honest answer and spares every caller a
///   guard. (This is deliberately independent of whether a shape *reserves*
///   pore capacity: reserved-but-ungrained pores have no throat, and an
///   undefined throat admits nothing.)
/// - **Zero grain sizes** cannot occur — the registry guarantees every
///   `grain_size_mm > 0` (asserted by `materials::tests::properties_are_sane`).
///   The `≤` comparison stays well-defined regardless: a hypothetical
///   zero-grain host yields a zero throat that admits no positive-grain filler
///   (`false`), the correct conservative fall-through, so no zero is special-cased.
#[inline]
pub fn fits_in_pores(filler: MaterialId, host_structure: &[MaterialId]) -> bool {
    // Finest host grain sets the throat (min over the structural multiset).
    let Some(min_host_grain) = host_structure
        .iter()
        .map(|m| m.props().grain_size_mm)
        .reduce(f32::min)
    else {
        // No host grains → no defined throat → nothing fits.
        return false;
    };
    filler.props().grain_size_mm <= K_PORE * min_host_grain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::DamageType;
    use crate::materials::contents::{StructureShape, VoxelContents};
    use crate::materials::extract::extraction_sequence;

    #[test]
    fn fines_pack_into_coarse_host() {
        // The canonical case: silt/clay infiltrate sandstone pores
        // (throat = 0.25 × 0.3 mm = 0.075 mm) — wattle-and-daub emergent from
        // grain sizes, not a recipe.
        let host = [MaterialId::SANDSTONE];
        assert!(fits_in_pores(MaterialId::SILT, &host)); // 0.02 mm
        assert!(fits_in_pores(MaterialId::CLAY, &host)); // 0.002 mm
    }

    #[test]
    fn sand_does_not_pack_into_sandstone() {
        // The negative that gives the rule teeth: sand (0.5 mm) is far too
        // coarse for the sandstone throat (0.075 mm) — it bridges, not packs.
        assert!(!fits_in_pores(MaterialId::SAND, &[MaterialId::SANDSTONE]));
    }

    #[test]
    fn heterogeneous_host_uses_the_finest_grain() {
        // Gravel (20 mm) alone would admit sand (throat 5 mm); adding a
        // sandstone grain (0.3 mm) to the fill drops the throat to 0.075 mm and
        // sand no longer fits. The MIN, not the max, governs.
        assert!(fits_in_pores(MaterialId::SAND, &[MaterialId::GRAVEL]));
        assert!(!fits_in_pores(
            MaterialId::SAND,
            &[MaterialId::GRAVEL, MaterialId::SANDSTONE]
        ));
        // Order independence: min is order-agnostic.
        assert!(!fits_in_pores(
            MaterialId::SAND,
            &[MaterialId::SANDSTONE, MaterialId::GRAVEL]
        ));
    }

    #[test]
    fn empty_host_admits_nothing() {
        // No structural grains define no throat → false, and crucially no panic
        // on the empty slice (the decided edge case).
        assert!(!fits_in_pores(MaterialId::CLAY, &[]));
        assert!(!fits_in_pores(MaterialId::SAND, &[]));
    }

    #[test]
    fn boundary_is_inclusive() {
        // Andesite grain 0.08 mm → throat exactly 0.25 × 0.08 = 0.02 mm, which
        // is silt's grain size: equality packs (`≤`, not `<`).
        assert!(fits_in_pores(MaterialId::SILT, &[MaterialId::ANDESITE]));
        // Coarser than the throat fails; sand (0.5 mm) certainly does.
        assert!(!fits_in_pores(MaterialId::SAND, &[MaterialId::ANDESITE]));
    }

    #[test]
    fn packing_and_sieving_are_the_same_axis() {
        // Composability (spec § Composability): sieve resistance ≡ grain size,
        // so what PACKS IN is exactly what SIEVES OUT first. Pack the two fines
        // that fit a sandstone host into its pores, then sieve: they emerge
        // fines-first (clay before silt), the same grain-size ordering that let
        // them in.
        let host = [MaterialId::SANDSTONE];
        let fillers = [MaterialId::SILT, MaterialId::CLAY];
        for f in fillers {
            assert!(fits_in_pores(f, &host), "precondition: {f:?} must pack");
        }
        // A slab (cap 4): 1 structural sandstone grain, 2 fines in the pores.
        let voxel = VoxelContents::new(StructureShape::Slab, &host, &fillers, &[])
            .expect("packed voxel is representable");
        let order: Vec<MaterialId> = extraction_sequence(&voxel, DamageType::Sieve)
            .iter()
            .map(|y| y.material)
            .collect();
        assert_eq!(
            order,
            vec![MaterialId::CLAY, MaterialId::SILT],
            "packed fines sieve out finest-first — the same grain-size axis"
        );
    }

    #[test]
    fn accessory_genesis_is_exempt_by_construction() {
        // Olivine (1.5 mm) in a basalt host (throat 0.25 × 0.05 = 0.0125 mm)
        // could NEVER pass the mechanical infiltration rule — it is orders too
        // coarse. That it exists in basalt at all proves the emplacement is
        // GENESIS (the crystal grew in place), not infiltration, which is
        // exactly why worldgen accessory emplacement is EXEMPT and never calls
        // this helper (docs/design/materials.md § Pore packability; see the
        // note at the accessory member in geology.rs).
        assert!(!fits_in_pores(MaterialId::OLIVINE, &[MaterialId::BASALT]));
    }
}
