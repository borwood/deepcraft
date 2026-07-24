//! **Contents are the single source of truth; the block is a pure derived
//! classification.** (docs/design/materials.md § forms design pass, ratified
//! 2026-07-21.)
//!
//! Before this module the generator held two opinions about every voxel: one
//! path computed a [`Block`], a parallel path computed [`VoxelContents`], and
//! journal/0010 installed a *trust gate* — the block decided whether the
//! contents were believed — to keep the two from contradicting each other at
//! the mesher. Under partials-first emission that gate inverts: the contents
//! know the honest fraction and the block does not, so a block that overrules
//! contents is a bug factory. The contract that replaces it:
//!
//! > For every voxel that carries contents, `block == classify(contents)`.
//!
//! [`classify`] is the *only* derivation. It is pure, total, and
//! order-independent — it takes canonical [`VoxelContents`], never an
//! order-dependent `MixtureId` (the journal/0010 landmine rules still hold
//! absolutely: the intern table's ids never leave the generator).
//!
//! ## What the block is, and is not
//!
//! Since the block↔material collapse (journal/0087) `classify` returns the
//! dominant material's **own identity** — `Block::Material(id)` — not a coarse
//! block-tier summary. There is no `block_twin` re-translation any more: the
//! material IS the block (materials.md DECIDED 2026-07-22). It deliberately does
//! **not** grow form-aware variants (no `LooseSandstone`): loose clastic and
//! structural clastic classify to the same *material*, so today they look
//! identical in a cut face. That consequence is ratified and accepted;
//! form-dependent texture variants are a later *visuals* decision.
//!
//! ## The absent-contents rule
//!
//! A voxel whose contents are [`VoxelContents::EMPTY`] is **unclassified, not
//! classified as Air**. `classify` answers [`Block::Air`] there because a voxel
//! with nothing in it is air *as far as contents go* — but the generator does
//! not apply `classify` to voxels it never gave a contents record: the surface
//! veneer (a stub, docs/design/stubs.md § 2), the legacy soil band and the
//! unrecorded basement below the deep-time record, ocean floor, the border
//! wilds, and ruin posts all keep their legacy blocks. So the enforced
//! invariant is scoped:
//!
//! > for every voxel with **non-empty** contents, `block == classify(contents)`
//!
//! and the exception shrinks on its own as those stubs acquire real records.

use crate::materials::MaterialId;
use crate::materials::contents::VoxelContents;
use crate::voxel::Block;

/// The material that defines this voxel's block-tier identity, or `None` for
/// empty contents.
///
/// Which multiset speaks, in order: **structural fill**, else **debris**, else
/// **pore fill**. Structure is what a voxel *is* — its load-bearing identity —
/// so a granite with olivine in its pores is granite. With no structure the
/// loose fill speaks, so a sand blanket is sand. Pore fill decides only in the
/// degenerate case of a shape with no structural fill at all (a hollow shell
/// packed with mud), where it is the only thing present.
///
/// Within the chosen multiset the **most abundant** material wins; ties break
/// to the lowest material id. Both the segment sort (canonical form) and the
/// tie-break make this order-independent by construction: equal contents give
/// equal answers, and the deposit order that built them is unrecoverable and
/// irrelevant.
pub fn dominant_material(contents: &VoxelContents) -> Option<MaterialId> {
    let segment = if !contents.structure().is_empty() {
        contents.structure()
    } else if !contents.debris().is_empty() {
        contents.debris()
    } else {
        contents.pore_fill()
    };
    // The segment is sorted ascending, so equal materials are adjacent: one
    // pass over runs, strictly-greater to keep the lowest id on a tie.
    let mut best: Option<(MaterialId, usize)> = None;
    let mut i = 0;
    while i < segment.len() {
        let m = segment[i];
        let mut n = 1;
        while i + n < segment.len() && segment[i + n] == m {
            n += 1;
        }
        if best.is_none_or(|(_, bn)| n > bn) {
            best = Some((m, n));
        }
        i += n;
    }
    best.map(|(m, _)| m)
}

/// The block a voxel's contents classify to — the single derivation of the
/// block tier from the source of truth (module docs).
///
/// Empty contents answer [`Block::Air`]; see the module's absent-contents rule
/// for why that is not the same as "a voxel with no record is air".
#[inline]
pub fn classify(contents: &VoxelContents) -> Block {
    match dominant_material(contents) {
        None => Block::Air,
        Some(m) => Block::Material(m),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::contents::StructureShape;

    #[test]
    fn empty_classifies_to_air() {
        assert_eq!(classify(&VoxelContents::EMPTY), Block::Air);
        assert_eq!(dominant_material(&VoxelContents::EMPTY), None);
    }

    #[test]
    fn structure_outranks_pore_fill() {
        // Granite with an olivine pore inclusion is granite, not olivine —
        // the igneous-accessory shape the strata passes emit.
        let c = VoxelContents::new(
            StructureShape::Full,
            &[MaterialId::GRANITE; 7],
            &[MaterialId::OLIVINE],
            &[],
        )
        .unwrap();
        assert_eq!(classify(&c), Block::Material(MaterialId::GRANITE));
    }

    #[test]
    fn debris_speaks_when_there_is_no_structure() {
        // Placer-enriched clastic: gold rides at most 3/8, so the host wins.
        for k in 0..=3usize {
            let mut d = [MaterialId::SANDSTONE; 8];
            for slot in d.iter_mut().take(k) {
                *slot = MaterialId::GOLD_DUST;
            }
            let c = VoxelContents::debris_only(&d).unwrap();
            assert_eq!(
                classify(&c),
                Block::Material(MaterialId::SANDSTONE),
                "{k} ore eighths"
            );
        }
        // A loose sand blanket classifies to sand — its own material identity,
        // no longer summarized to a coarse-clastic block (form is not identity).
        let loose = VoxelContents::debris_only(&[MaterialId::SANDSTONE; 3]).unwrap();
        assert_eq!(classify(&loose), Block::Material(MaterialId::SANDSTONE));
    }

    #[test]
    fn pore_fill_decides_only_when_alone() {
        let c = VoxelContents::new(StructureShape::Full, &[], &[MaterialId::CLAY; 4], &[]).unwrap();
        assert_eq!(classify(&c), Block::Material(MaterialId::CLAY));
    }

    #[test]
    fn classification_is_order_independent() {
        let a = VoxelContents::debris_only(&[
            MaterialId::GOLD_DUST,
            MaterialId::SANDSTONE,
            MaterialId::SANDSTONE,
            MaterialId::GOLD_DUST,
        ])
        .unwrap();
        let b = VoxelContents::debris_only(&[
            MaterialId::SANDSTONE,
            MaterialId::GOLD_DUST,
            MaterialId::GOLD_DUST,
            MaterialId::SANDSTONE,
        ])
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(classify(&a), classify(&b));
        // A 2–2 tie breaks to the lowest material id: sandstone (13) beats
        // gold dust (16). The point is that it is stable and independent of
        // insertion order, not which of the two happens to win.
        assert_eq!(dominant_material(&a), Some(MaterialId::SANDSTONE));
        assert_eq!(classify(&a), Block::Material(MaterialId::SANDSTONE));
    }

    #[test]
    fn every_material_classifies_to_its_own_identity() {
        // Totality: every material classifies to exactly itself — the whole
        // point of the collapse. No `_ => Stone` fallback, no block twin: the
        // nine formerly face-less materials (snow, leaf-litter, potsherd,
        // knapping-debris, ash, scree, bone, gold-dust, olivine) now wear their
        // own identity like every other.
        for m in MaterialId::all() {
            let c = VoxelContents::debris_only(&[m]).unwrap();
            let b = classify(&c);
            assert_eq!(b, Block::Material(m));
            assert!(
                b.is_solid(),
                "a voxel with contents must not classify to Air"
            );
        }
    }
}
