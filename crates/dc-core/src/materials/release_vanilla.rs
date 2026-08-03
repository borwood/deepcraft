//! **The DEFAULT PACK's release spectra** — vanilla content for the edge
//! product-table primitive in [`super::release`] (FS-A, 2026-08-02).
//!
//! **Authored AGAINST THE LITERATURE** (CLAUDE.md § "a closed system cannot
//! detect its own scale error": measure-against-the-literature applies at
//! authoring time — that is the point of the R2 ruling). Each table's doc
//! comment carries its band and its source, with per-number confidence stated
//! the way the P10 audit states its anchors (`docs/audits/2026-08-01-p10-grain-
//! axis-design.md` § 4.1).
//!
//! **PROVENANCE-KEEPING PER U1**: every product below names the SOURCE material
//! at a grade. NEVER the standalone `SAND`/`GRAVEL`/`SILT`/`CLAY`/`SCREE`
//! identities — those are the scaffolding-era encoding (P10 audit C4), and the
//! guard is [`tests::vanilla_products_keep_provenance`]. This is the default
//! pack's content choice; the primitive next door stays general (U7).
//!
//! Today's authored set is the deep roster's **source rocks** — the materials
//! that stand as `Structure` and weather (`structure → loose`). Loose clastics,
//! organics and accessories keep the identity default: peat does not shed a
//! mineral grain distribution, and a loose material has no structure to
//! weather. Their tables arrive with the passes that need them (e.g. an
//! abrasion edge's comminution products — the P10 arc's later slices).

use super::release::{EdgeProducts, GrainGrade, ReleaseProduct};
use super::{MATERIAL_COUNT, MaterialId};
use crate::materials::form::InvForm;

/// **Granite → grus** (`structure → loose`): the P10 audit's L9 anchor, now
/// with retrieved numbers.
///
/// Literature band (retrieved 2026-08-02): grus is **sand+gravel dominated —
/// 75–100 wt% combined, silt+clay < 25%** (grus weathering-mantle literature;
/// Migoń & Thomas 2002, *Grus weathering mantles — problems of interpretation*,
/// Catena). The Enchanted Rock pluton study (*Grain size of granite and derived
/// grus*, Sedimentary Geology, 1985) measures a **gravel-dominant** grus
/// (primary modes 2.8–23 mm, mostly granite rock fragments) — confidence
/// MEDIUM: two sources, one quantitative. The **clay > silt** split of the
/// fines is the textbook feldspar→kaolinite alteration path (the bimodality of
/// grus is textbook — L9), confidence MEDIUM-HIGH qualitatively, LOW as a
/// number.
///
/// Authored placement inside the band: coarse (gravel+sand) **800‰**, fines
/// **200‰**, with the silt trough (silt 50 < clay 150) expressing the
/// coarse-vs-fines **bimodality** the literature calls first-order (L4's
/// grain-size gap; U3 kept expressible). No scree rung: corestone-scale blocks
/// are a hillslope/talus phenomenon, not a per-cell weathering product.
const GRANITE_GRUS: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::GRANITE,
        grade: GrainGrade::Gravel,
        share_permille: 350,
    },
    ReleaseProduct {
        material: MaterialId::GRANITE,
        grade: GrainGrade::Sand,
        share_permille: 450,
    },
    ReleaseProduct {
        material: MaterialId::GRANITE,
        grade: GrainGrade::Silt,
        share_permille: 50,
    },
    ReleaseProduct {
        material: MaterialId::GRANITE,
        grade: GrainGrade::Clay,
        share_permille: 150,
    },
];

/// **Diorite → grus** (`structure → loose`): same mechanism as granite (coarse
/// crystalline framework disaggregates grain-by-grain), shifted one notch finer
/// — the registry's diorite grain (2 mm) sits below granite's (3 mm), and
/// intermediate plagioclase alters faster than K-feldspar, so slightly more of
/// the framework leaves as fines. Confidence LOW as numbers (positioned by
/// analogy inside granite's band, not independently measured — stated per the
/// audit's honesty discipline).
const DIORITE_GRUS: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::DIORITE,
        grade: GrainGrade::Gravel,
        share_permille: 300,
    },
    ReleaseProduct {
        material: MaterialId::DIORITE,
        grade: GrainGrade::Sand,
        share_permille: 480,
    },
    ReleaseProduct {
        material: MaterialId::DIORITE,
        grade: GrainGrade::Silt,
        share_permille: 60,
    },
    ReleaseProduct {
        material: MaterialId::DIORITE,
        grade: GrainGrade::Clay,
        share_permille: 160,
    },
];

/// **Basalt → clay-rich saprolite** (`structure → loose`): a fine-crystalline
/// rock has no sand-grade framework grains to shed; its ferromagnesian minerals
/// and glass alter to smectite/nontronite clays.
///
/// Literature (retrieved 2026-08-02): Columbia River Basalt saprolite —
/// secondary **nontronite clay** formation with early Fe/Mg depletion
/// (*Geochemistry and mineralogy of a saprolite developed on Columbia River
/// Basalt*, American Mineralogist 2017); basalt weathering to clay minerals in
/// the humid tropics (Leyte, Philippines study). Basalt-derived residual soils
/// are commonly reported at ~40–70% clay fraction — confidence MEDIUM for
/// clay-dominance, LOW for the exact split. Authored: clay **600‰** (in the
/// band), silt 250‰, sand 100‰, gravel 50‰ (corestone chips — spheroidal
/// weathering shells, textbook for basalt).
const BASALT_SAPROLITE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::BASALT,
        grade: GrainGrade::Gravel,
        share_permille: 50,
    },
    ReleaseProduct {
        material: MaterialId::BASALT,
        grade: GrainGrade::Sand,
        share_permille: 100,
    },
    ReleaseProduct {
        material: MaterialId::BASALT,
        grade: GrainGrade::Silt,
        share_permille: 250,
    },
    ReleaseProduct {
        material: MaterialId::BASALT,
        grade: GrainGrade::Clay,
        share_permille: 600,
    },
];

/// **Andesite → saprolite** (`structure → loose`): intermediate between the
/// basalt and diorite cases — finer than diorite (0.08 mm groundmass), less
/// glassy/mafic than basalt, so clay-rich but less extremely so. Confidence LOW
/// as numbers (interpolated between the two anchored neighbours; stated).
const ANDESITE_SAPROLITE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::ANDESITE,
        grade: GrainGrade::Gravel,
        share_permille: 50,
    },
    ReleaseProduct {
        material: MaterialId::ANDESITE,
        grade: GrainGrade::Sand,
        share_permille: 150,
    },
    ReleaseProduct {
        material: MaterialId::ANDESITE,
        grade: GrainGrade::Silt,
        share_permille: 300,
    },
    ReleaseProduct {
        material: MaterialId::ANDESITE,
        grade: GrainGrade::Clay,
        share_permille: 500,
    },
];

/// **Sandstone → sand** (`structure → loose`): disaggregation returns the
/// framework grains at their depositional grade — the definitional case for
/// provenance-keeping release.
///
/// Literature: an arenite is ≥85% framework grains by definition (matrix < 15%
/// separates arenite from wacke — Pettijohn, Potter & Siever, *Sand and
/// Sandstone*; Dott's classification). Confidence HIGH for the shape
/// (definitional), MEDIUM for the split of the non-framework remainder into
/// silt-grade cement fragments vs clay-grade matrix. Authored: sand **800‰**,
/// silt 120‰, clay 80‰.
const SANDSTONE_RELEASE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::SANDSTONE,
        grade: GrainGrade::Sand,
        share_permille: 800,
    },
    ReleaseProduct {
        material: MaterialId::SANDSTONE,
        grade: GrainGrade::Silt,
        share_permille: 120,
    },
    ReleaseProduct {
        material: MaterialId::SANDSTONE,
        grade: GrainGrade::Clay,
        share_permille: 80,
    },
];

/// **Siltstone → silt** (`structure → loose`): as sandstone, one grade finer —
/// the framework is silt (registry grain 0.02 mm sits in the 3.9–62.5 µm rung),
/// with a clay-matrix remainder. Confidence: shape definitional, split
/// authored.
const SILTSTONE_RELEASE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::SILTSTONE,
        grade: GrainGrade::Sand,
        share_permille: 50,
    },
    ReleaseProduct {
        material: MaterialId::SILTSTONE,
        grade: GrainGrade::Silt,
        share_permille: 750,
    },
    ReleaseProduct {
        material: MaterialId::SILTSTONE,
        grade: GrainGrade::Clay,
        share_permille: 200,
    },
];

/// **Mudstone → mud** (`structure → loose`): a mudstone is lithified
/// silt-and-clay mix, clay-dominant (mudstone vs siltstone is drawn at the
/// clay:silt ratio — Folk's textural classification; the registry's mudstone
/// grain is 0.004 mm, clay-grade). Confidence HIGH for clay-dominance
/// (definitional), authored split clay **650‰** / silt 300‰ / sand 50‰.
const MUDSTONE_RELEASE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::MUDSTONE,
        grade: GrainGrade::Sand,
        share_permille: 50,
    },
    ReleaseProduct {
        material: MaterialId::MUDSTONE,
        grade: GrainGrade::Silt,
        share_permille: 300,
    },
    ReleaseProduct {
        material: MaterialId::MUDSTONE,
        grade: GrainGrade::Clay,
        share_permille: 650,
    },
];

/// **Conglomerate → clasts + matrix** (`structure → loose`): cement fails
/// first, releasing the pebble framework whole (gravel grade — registry grain
/// 8 mm) into its sandy matrix. Clast-supported conglomerates run roughly
/// 60–80% clasts by volume (sedimentology textbook range — Boggs, *Principles
/// of Sedimentology and Stratigraphy*; confidence MEDIUM). Authored: gravel
/// **550‰**, sand 350‰, silt 40‰, clay 60‰ — a second deliberately **bimodal**
/// table (clast mode + sand-matrix mode, fines trough between).
const CONGLOMERATE_RELEASE: &[ReleaseProduct] = &[
    ReleaseProduct {
        material: MaterialId::CONGLOMERATE,
        grade: GrainGrade::Gravel,
        share_permille: 550,
    },
    ReleaseProduct {
        material: MaterialId::CONGLOMERATE,
        grade: GrainGrade::Sand,
        share_permille: 350,
    },
    ReleaseProduct {
        material: MaterialId::CONGLOMERATE,
        grade: GrainGrade::Silt,
        share_permille: 40,
    },
    ReleaseProduct {
        material: MaterialId::CONGLOMERATE,
        grade: GrainGrade::Clay,
        share_permille: 60,
    },
];

/// The `structure → loose` weathering edge wrapper for each authored spectrum —
/// the primitive's first customer.
macro_rules! weathering_edge {
    ($products:expr) => {
        &[EdgeProducts {
            from: InvForm::Structure,
            to: InvForm::Loose,
            products: $products,
        }]
    };
}

/// **The release declarations, index-aligned with the registry** (the
/// `MATERIAL_QUALIFIED_NAMES` shape: a sibling const table that is part of the
/// material definition). `&[]` = no spectrum declared = the S-5 identity
/// default on every edge. The array length is checked against
/// [`MATERIAL_COUNT`] by the compiler, so adding a material forces a row here —
/// a checked extension, like every closed set in the tree.
#[rustfmt::skip]
pub(super) static RELEASE_DECLARATIONS: [&[EdgeProducts]; MATERIAL_COUNT] = [
    &[],                                    // sand (loose — nothing to release)
    &[],                                    // gravel (loose)
    &[],                                    // snow
    &[],                                    // leaf-litter
    &[],                                    // clay (loose)
    &[],                                    // silt (loose)
    &[],                                    // potsherd
    &[],                                    // knapping-debris
    &[],                                    // ash
    &[],                                    // loam
    &[],                                    // scree (loose)
    &[],                                    // bone
    weathering_edge!(MUDSTONE_RELEASE),     // mudstone
    weathering_edge!(SANDSTONE_RELEASE),    // sandstone
    weathering_edge!(GRANITE_GRUS),         // granite
    weathering_edge!(BASALT_SAPROLITE),     // basalt
    &[],                                    // gold-dust
    weathering_edge!(SILTSTONE_RELEASE),    // siltstone
    weathering_edge!(CONGLOMERATE_RELEASE), // conglomerate
    weathering_edge!(DIORITE_GRUS),         // diorite
    weathering_edge!(ANDESITE_SAPROLITE),   // andesite
    &[],                                    // olivine (accessory — pore-scale, no bulk seam)
    &[],                                    // peat (organic — no mineral spectrum)
    &[],                                    // coal
    &[],                                    // carbonaceous-mudstone (organic-rich; heir with the organics rework)
    &[],                                    // charcoal
];

#[cfg(test)]
mod tests {
    use super::super::release::validate_release_registry;
    use super::*;

    /// **The gate's registry sweep**: every authored table passes the
    /// declaration-time mass closure and canonical-form checks. Scale-free — a
    /// property of the const data, no world.
    #[test]
    fn the_vanilla_release_registry_validates() {
        if let Err(bad) = validate_release_registry() {
            panic!("invalid release declarations: {bad:?}");
        }
    }

    /// **U1 — vanilla keeps provenance**: every authored product names the
    /// declaring material itself at a grade. This is also the C4 guard: no
    /// spectrum consumes the standalone `SAND`/`GRAVEL`/`CLAY`/`SILT`/`SCREE`
    /// ladder identities (scaffolding-era encoding) as products. The engine
    /// primitive stays general; this pins the DEFAULT PACK's authoring choice.
    #[test]
    fn vanilla_products_keep_provenance() {
        for m in MaterialId::all() {
            for e in m.release_declarations() {
                for pr in e.products {
                    assert_eq!(
                        pr.material,
                        m,
                        "vanilla release on {} names {} — U1 forbids non-provenance products \
                         in the default pack",
                        m.props().name,
                        pr.material.props().name
                    );
                }
            }
        }
    }

    /// Every source rock that stands as structure in the deep roster declares a
    /// weathering spectrum, and only source rocks do — loose clastics, organics
    /// and accessories keep the identity default (each `&[]` row above is a
    /// choice, and this pins which ones).
    #[test]
    fn the_deep_source_rocks_and_only_they_declare_weathering_spectra() {
        let sources = [
            MaterialId::MUDSTONE,
            MaterialId::SANDSTONE,
            MaterialId::GRANITE,
            MaterialId::BASALT,
            MaterialId::SILTSTONE,
            MaterialId::CONGLOMERATE,
            MaterialId::DIORITE,
            MaterialId::ANDESITE,
        ];
        for m in MaterialId::all() {
            let declared = m
                .release_products(InvForm::Structure, InvForm::Loose)
                .is_some();
            assert_eq!(
                declared,
                sources.contains(&m),
                "{}: weathering spectrum declared = {declared}",
                m.props().name
            );
        }
    }

    /// The bimodal tables really are bimodal — a coarse mode and a clay mode
    /// with a silt trough between (the U3-stays-expressible claim, held by
    /// data, not just by the table shape).
    #[test]
    fn granite_and_conglomerate_release_bimodally() {
        for m in [MaterialId::GRANITE, MaterialId::CONGLOMERATE] {
            let products = m
                .release_products(InvForm::Structure, InvForm::Loose)
                .expect("declared");
            let share = |g: GrainGrade| -> u16 {
                products
                    .iter()
                    .find(|p| p.grade == g)
                    .map_or(0, |p| p.share_permille)
            };
            let coarse = share(GrainGrade::Gravel).max(share(GrainGrade::Sand));
            assert!(
                share(GrainGrade::Silt) < coarse
                    && share(GrainGrade::Silt) < share(GrainGrade::Clay),
                "{}: the silt trough between the coarse and clay modes is the bimodal \
                 signature (L4's grain-size gap)",
                m.props().name
            );
        }
    }
}
