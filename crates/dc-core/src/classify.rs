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
//! `Block` stays the **coarse render / storage / far-field summary** in its
//! existing vocabulary. It deliberately does **not** grow form-aware variants
//! (no `LooseSandstone`): loose clastic and structural clastic classify to the
//! same class block, so today they look identical in a cut face. That
//! consequence is ratified and accepted; form-dependent texture variants are a
//! later *visuals* decision, not a block-vocabulary one.
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

/// The coarse block a material reads as. **A material property, not a geology
/// class property** — which is the whole point: two members of one content
/// class (mudstone and siltstone; sandstone and conglomerate) summarize to the
/// same block because their *materials* do, not because a `GeologySet` said so.
/// That keeps [`classify`] pure over contents and independent of the registry,
/// while reproducing exactly what the class→block table used to answer for
/// every member the vanilla (and the roster-proof) sets register.
///
/// Materials with no block twin in today's vocabulary fall back to
/// [`Block::Stone`] — the same coarse fallback the class table used for unknown
/// classes. Growing the vocabulary is a content decision; growing *this table*
/// is how a new material becomes legible at the block tier.
pub fn block_twin(material: MaterialId) -> Block {
    match material.raw() {
        // --- fine clastic ---
        r if r == MaterialId::MUDSTONE.raw() => Block::Mudstone,
        r if r == MaterialId::SILTSTONE.raw() => Block::Mudstone,
        // Loose fines summarize to the same coarse band as their lithified
        // form: a silt drape and a siltstone bed are the same rock at the
        // block tier (the forms contract — form is not block identity).
        r if r == MaterialId::SILT.raw() => Block::Mudstone,
        r if r == MaterialId::CLAY.raw() => Block::Mudstone,
        // --- coarse clastic ---
        r if r == MaterialId::SANDSTONE.raw() => Block::Sandstone,
        r if r == MaterialId::CONGLOMERATE.raw() => Block::Sandstone,
        r if r == MaterialId::SAND.raw() => Block::Sandstone,
        r if r == MaterialId::GRAVEL.raw() => Block::Sandstone,
        // --- igneous ---
        r if r == MaterialId::GRANITE.raw() => Block::Granite,
        r if r == MaterialId::DIORITE.raw() => Block::Granite,
        r if r == MaterialId::BASALT.raw() => Block::Basalt,
        r if r == MaterialId::ANDESITE.raw() => Block::Basalt,
        // --- organic ---
        r if r == MaterialId::COAL.raw() => Block::Coal,
        r if r == MaterialId::PEAT.raw() => Block::Peat,
        r if r == MaterialId::CARBONACEOUS_MUDSTONE.raw() => Block::CarbonaceousMudstone,
        // Charcoal summarizes to the same coarse black-carbon band as coal. It
        // gets a twin rather than falling through to `Block::Stone` for one
        // specific reason: charcoal is an *inclusion* — measured, it wins at
        // most one eighth of a voxel — and the one place a single eighth can
        // still decide a block is the top-of-column partial fill, where the
        // surface voxel may hold only one eighth in the first place. Without a
        // twin, a burned horizon at the surface would occasionally read as grey
        // stone. Growing the `Block` vocabulary is a separate content decision;
        // sharing coal's band is the honest summary in today's one.
        r if r == MaterialId::CHARCOAL.raw() => Block::Coal,
        // --- soil ---
        r if r == MaterialId::LOAM.raw() => Block::Dirt,
        // No block twin yet: gold dust and olivine are accessory grains that
        // never dominate a voxel; snow, leaf litter, ash, scree, bone,
        // potsherds and knapping debris are loose materials the block
        // vocabulary has not grown a summary for.
        _ => Block::Stone,
    }
}

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
        Some(m) => block_twin(m),
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
        assert_eq!(classify(&c), Block::Granite);
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
            assert_eq!(classify(&c), Block::Sandstone, "{k} ore eighths");
        }
        // A loose sand blanket is the same block as the sandstone it is the
        // loose form of — form is not block identity (the ratified consequence).
        let loose = VoxelContents::debris_only(&[MaterialId::SANDSTONE; 3]).unwrap();
        assert_eq!(classify(&loose), Block::Sandstone);
    }

    #[test]
    fn pore_fill_decides_only_when_alone() {
        let c = VoxelContents::new(StructureShape::Full, &[], &[MaterialId::CLAY; 4], &[]).unwrap();
        assert_eq!(classify(&c), Block::Mudstone);
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
        assert_eq!(classify(&a), Block::Sandstone);
    }

    #[test]
    fn every_material_classifies_to_something() {
        // Totality: no material may panic or fall out of the table.
        for m in MaterialId::all() {
            let c = VoxelContents::debris_only(&[m]).unwrap();
            let b = classify(&c);
            assert_eq!(b, block_twin(m));
            assert!(
                b.is_solid(),
                "a voxel with contents must not classify to Air"
            );
        }
    }
}
