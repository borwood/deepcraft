//! **Release spectra — the edge product-table PRIMITIVE** (FS-A, 2026-08-02).
//!
//! **The ruling this module builds under** (U7, user, 2026-08-02, *"R2 it is —
//! authored edges"*; recorded in `docs/design/material-behavior.md` §3): release
//! spectra are **PACK-AUTHORED EDGE PRODUCTS**, not engine-derived. An edge
//! (first customer: `structure → loose`, weathering) may declare a **product
//! table**: shares over what the transition emits. The primitive's product
//! vocabulary is **GENERAL**, by the user's own directive — *"an edge can define
//! a transition to a loose grade of the original material, or to a totally
//! different material (mods may want this)"* — so the engine-side shape admits
//! any [`MaterialId`] product. Provenance-keeping (products = loose grades of
//! the source, per U1) is the **default pack's** authoring choice, declared in
//! the sibling module [`super::release_vanilla`], not an engine constraint.
//! **Generality lives here; provenance lives in the vanilla authoring.**
//!
//! **Why this lives on the material definition in `dc-core`**: `materials.md`
//! DECIDED 2026-07-22 — transformations are *"declared on material definitions …
//! the ONE authority both the runtime simulation and deep-time's bulk arithmetic
//! consult."* The deep tier's weathering pass and any future present-tier
//! process read the same table; neither carries a parallel rule (S-3 applied to
//! processes; A-7 is the consumer's side of the same coin — a pass consults the
//! declaration instead of naming rocks).
//!
//! **What this module deliberately does NOT do**: store grain. The grain state's
//! one home is the packed `DepUnit` (P11 slice 3, U5: funded from reclaimed
//! padding, `grain()`/`set_grain()`/`GRAIN_UNSET`); inventing storage here would
//! be the A-1 anti-shape. Until that record merges, a consumer computes the
//! split and hands each product's grade to a clearly-marked seam
//! (`dc-worldgen::deeptime::weather_inventory::grain_write_seam`).

use super::MaterialId;
use super::form::InvForm;

/// **The five-rung grain-grade ladder** — U4 (user, 2026-08-02, *"I agree: 5"*):
/// five φ classes, the ladder's natural rungs, **grain = 3 bits** in the packed
/// record.
///
/// The rung boundaries are the Wentworth/Udden partition (definitional —
/// Wentworth 1922; the P10 audit's L1 anchor), collapsed to five classes:
///
/// | grade | diameter | φ = −log₂(d/mm) | Wentworth span |
/// |---|---|---|---|
/// | `Scree` | > 64 mm | φ < −6 | cobble + boulder |
/// | `Gravel` | 2–64 mm | −6 ≤ φ < −1 | granule + pebble |
/// | `Sand` | 0.0625–2 mm | −1 ≤ φ < 4 | sand |
/// | `Silt` | 3.9 µm–62.5 µm | 4 ≤ φ < 8 | silt |
/// | `Clay` | < 3.9 µm | φ ≥ 8 | clay |
///
/// **⚠ A grade is a STATE, never an identity** (U1). The registry's standalone
/// `SAND`/`GRAVEL`/`CLAY`/`SILT`/`SCREE` materials are the **scaffolding-era
/// encoding** of the same idea (the P10 audit's C4 two-authorities finding,
/// settled by U1 in favour of `(source-id, Loose)` + grain state) — a release
/// product says *"granite, loose, at sand grade"*, never *"the SAND material"*.
/// Vanilla provenance-keeping is asserted by
/// `release_vanilla::tests::vanilla_products_keep_provenance`.
///
/// **⚠ This enum is a vocabulary, not storage.** The record's grain byte lands
/// with P11 slice 3's packed `DepUnit`; nothing here persists a grade.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GrainGrade {
    /// > 64 mm — cobble/boulder grade (talus blocks, corestones).
    Scree,
    /// 2–64 mm — granule/pebble grade.
    Gravel,
    /// 0.0625–2 mm — sand grade.
    Sand,
    /// 3.9–62.5 µm — silt grade.
    Silt,
    /// < 3.9 µm — clay grade.
    Clay,
}

/// The number of [`GrainGrade`] rungs — U4's five.
pub const GRAIN_GRADE_COUNT: usize = 5;

impl GrainGrade {
    /// Every grade, coarse → fine (descending grain size, ascending φ).
    pub const ALL: [GrainGrade; GRAIN_GRADE_COUNT] = [
        GrainGrade::Scree,
        GrainGrade::Gravel,
        GrainGrade::Sand,
        GrainGrade::Silt,
        GrainGrade::Clay,
    ];

    /// This grade's position in the ladder, `0..GRAIN_GRADE_COUNT`, coarse → fine.
    #[inline]
    pub const fn raw(self) -> u8 {
        match self {
            GrainGrade::Scree => 0,
            GrainGrade::Gravel => 1,
            GrainGrade::Sand => 2,
            GrainGrade::Silt => 3,
            GrainGrade::Clay => 4,
        }
    }

    /// The grade at position `raw`, or `None` when out of the ladder.
    #[inline]
    pub const fn from_raw(raw: u8) -> Option<GrainGrade> {
        match raw {
            0 => Some(GrainGrade::Scree),
            1 => Some(GrainGrade::Gravel),
            2 => Some(GrainGrade::Sand),
            3 => Some(GrainGrade::Silt),
            4 => Some(GrainGrade::Clay),
            _ => None,
        }
    }

    /// Short name for probe / provenance output.
    pub const fn name(self) -> &'static str {
        match self {
            GrainGrade::Scree => "scree",
            GrainGrade::Gravel => "gravel",
            GrainGrade::Sand => "sand",
            GrainGrade::Silt => "silt",
            GrainGrade::Clay => "clay",
        }
    }
}

/// **One product of a release spectrum**: `share_permille`/1000 of the
/// transition's quantity emits as `material` at `grade`.
///
/// `material` may be any registered id — generality in the primitive (U7).
/// A product that cannot exist cannot be written: the [`MaterialId`] and
/// [`GrainGrade`] types are closed, so *"every product MaterialId exists"* and
/// *"grades are the 5-rung ladder"* are discharged by the compiler, which is the
/// north star's "compiler-validated shape". What the compiler cannot check —
/// shares summing to one, non-degenerate tables — is
/// [`validate_edge_products`]'s job, run over the whole registry by the gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReleaseProduct {
    /// What the transition emits. Vanilla keeps provenance: always the declaring
    /// material itself (U1). Mods may name any registered id.
    pub material: MaterialId,
    /// The grain grade of this product — the state the packed record will store
    /// (P11 slice 3); until then, the quantity a consumer hands the grain seam.
    pub grade: GrainGrade,
    /// This product's share of the transition, in **per-mille** (integer, so a
    /// table's shares can sum to [`SHARE_DENOMINATOR`] **exactly** — the
    /// Law-3-shaped closure [`validate_edge_products`] asserts; f64 shares
    /// could only ever sum to ≈1).
    pub share_permille: u16,
}

/// The denominator shares are expressed over: a valid table's shares sum to
/// exactly this. Integer per-mille rather than f32 fractions so "shares sum to
/// 1 **exactly**" is checkable equality, not a tolerance.
pub const SHARE_DENOMINATOR: u32 = 1000;

/// **A release spectrum for one declared form edge** on a material definition:
/// when `from → to` runs on this material, it emits `products` instead of the
/// source's own identity at an unresolved grade.
///
/// A material with **no** entry for an edge keeps the identity default (S-5):
/// the transition emits the source material itself, grade unresolved — exactly
/// what every transition did before FS-A.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EdgeProducts {
    /// The source form of the keyed edge.
    pub from: InvForm,
    /// The destination form of the keyed edge. Products occupy volume, so this
    /// is never [`InvForm::Void`] (validated).
    pub to: InvForm,
    /// The spectrum: what the transition emits, shares summing to
    /// [`SHARE_DENOMINATOR`] exactly. **The table shape deliberately permits
    /// multi-modal spectra** (several products of one material at different
    /// grades) — U3 (bimodality) is unforced by ruling, so the shape must not
    /// forbid it.
    pub products: &'static [ReleaseProduct],
}

/// Why a release table failed validation. Carried as data so the gate can say
/// *which* table is bad and *how*, and so a future pack loader can surface the
/// same diagnostics for authored content.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReleaseError {
    /// A table with no products declares nothing; the honest encoding of "no
    /// spectrum" is no entry at all (the S-5 identity default).
    EmptyProducts,
    /// `from == to` — the null edge, which the transition graph itself refuses
    /// (`is_declared_edge`); a product table on it could never be consulted.
    NullEdge,
    /// A product table on an edge INTO `Void`: products occupy volume, and the
    /// complement holds none — dissolution has no products by construction.
    ProductsIntoVoid,
    /// A zero share is a product that never emits — dead weight in an authored
    /// table, refused so tables stay canonical.
    ZeroShare,
    /// Shares must sum to [`SHARE_DENOMINATOR`] exactly (mass closure at the
    /// declaration — Law 3's shape at authoring time). Carries the bad sum.
    SharesDoNotSumToOne(u32),
    /// Two products with the same `(material, grade)` are one product written
    /// twice; refused so a table has one row per bucket.
    DuplicateProduct,
    /// Two tables on one material key the same `(from, to)` edge — two
    /// authorities for one transition.
    DuplicateEdge,
}

/// Validate one edge's product table. See [`ReleaseError`] for each check's
/// rationale; the checks a type cannot express are exactly the ones here.
pub fn validate_edge_products(e: &EdgeProducts) -> Result<(), ReleaseError> {
    if e.from == e.to {
        return Err(ReleaseError::NullEdge);
    }
    if e.to == InvForm::Void {
        return Err(ReleaseError::ProductsIntoVoid);
    }
    if e.products.is_empty() {
        return Err(ReleaseError::EmptyProducts);
    }
    let mut sum: u32 = 0;
    for (i, p) in e.products.iter().enumerate() {
        if p.share_permille == 0 {
            return Err(ReleaseError::ZeroShare);
        }
        sum += u32::from(p.share_permille);
        if e.products[..i]
            .iter()
            .any(|q| q.material == p.material && q.grade == p.grade)
        {
            return Err(ReleaseError::DuplicateProduct);
        }
    }
    if sum != SHARE_DENOMINATOR {
        return Err(ReleaseError::SharesDoNotSumToOne(sum));
    }
    Ok(())
}

/// Validate every material's release declarations — the whole-registry sweep the
/// gate runs (`release_vanilla::tests::the_vanilla_release_registry_validates`),
/// and the same sweep a pack loader owes authored content at registration.
pub fn validate_release_registry() -> Result<(), Vec<(MaterialId, ReleaseError)>> {
    let mut bad = Vec::new();
    for m in MaterialId::all() {
        let tables = m.release_declarations();
        for (i, e) in tables.iter().enumerate() {
            if let Err(err) = validate_edge_products(e) {
                bad.push((m, err));
            }
            if tables[..i].iter().any(|q| q.from == e.from && q.to == e.to) {
                bad.push((m, ReleaseError::DuplicateEdge));
            }
        }
    }
    if bad.is_empty() { Ok(()) } else { Err(bad) }
}

/// **Split `qty` over a table's products so the itemisation equals the total
/// EXACTLY** (Law 3's shape at emission): every product takes
/// `qty × share/1000` except the **last**, which takes `qty − Σ(earlier)` — so
/// the per-product quantities re-sum to `qty` to the bit (in the same
/// summation order), and no rounding crumb is ever minted or dropped.
///
/// This is the ONE split arithmetic: the weathering consumer emits with it and
/// the probe itemises the recorded band with it, so the two cannot disagree
/// (S-3 — the report is derived from the same authority the pass consulted).
pub fn split_quantities(products: &'static [ReleaseProduct], qty: f64) -> SplitQuantities {
    SplitQuantities {
        products,
        qty,
        assigned: 0.0,
        next: 0,
    }
}

/// Iterator over `(product, quantity)` — see [`split_quantities`].
pub struct SplitQuantities {
    products: &'static [ReleaseProduct],
    qty: f64,
    assigned: f64,
    next: usize,
}

impl Iterator for SplitQuantities {
    type Item = (&'static ReleaseProduct, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.products.get(self.next)?;
        let q = if self.next + 1 == self.products.len() {
            self.qty - self.assigned
        } else {
            self.qty * f64::from(p.share_permille) / f64::from(SHARE_DENOMINATOR)
        };
        self.assigned += q;
        self.next += 1;
        Some((p, q))
    }
}

impl MaterialId {
    /// Every release table declared on this material, in declaration order.
    /// Index-aligned with the registry like [`super::MATERIAL_QUALIFIED_NAMES`]
    /// — the declarations ARE part of the material definition; the vanilla
    /// tables live in [`super::release_vanilla`] so the authored content reads
    /// as one block (a pack data file in const form).
    #[inline]
    pub fn release_declarations(self) -> &'static [EdgeProducts] {
        super::release_vanilla::RELEASE_DECLARATIONS[self.raw() as usize]
    }

    /// The release spectrum this material declares for the `from → to` edge, or
    /// `None` — the identity default: the transition emits the source itself,
    /// grade unresolved, exactly as it did before FS-A (S-5).
    pub fn release_products(self, from: InvForm, to: InvForm) -> Option<&'static [ReleaseProduct]> {
        self.release_declarations()
            .iter()
            .find(|e| e.from == from && e.to == to)
            .map(|e| e.products)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The validation catches a bad table** (the FS-A brief's named gate
    /// test). Each refusal in [`ReleaseError`] is exercised with a minimal
    /// offending table — so a future pack loader inherits checks that are known
    /// to fire, not checks that have only ever seen good data. Scale-free:
    /// arithmetic over const data, no world.
    #[test]
    fn validation_catches_a_bad_table() {
        // Shares that do not close to 1.
        static SHORT: [ReleaseProduct; 2] = [
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Sand,
                share_permille: 500,
            },
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Clay,
                share_permille: 499,
            },
        ];
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Structure,
                to: InvForm::Loose,
                products: &SHORT,
            }),
            Err(ReleaseError::SharesDoNotSumToOne(999))
        );
        // The null edge.
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Loose,
                to: InvForm::Loose,
                products: &SHORT,
            }),
            Err(ReleaseError::NullEdge)
        );
        // Products into the complement.
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Structure,
                to: InvForm::Void,
                products: &SHORT,
            }),
            Err(ReleaseError::ProductsIntoVoid)
        );
        // An empty table.
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Structure,
                to: InvForm::Loose,
                products: &[],
            }),
            Err(ReleaseError::EmptyProducts)
        );
        // A zero share.
        static ZERO: [ReleaseProduct; 2] = [
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Sand,
                share_permille: 1000,
            },
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Clay,
                share_permille: 0,
            },
        ];
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Structure,
                to: InvForm::Loose,
                products: &ZERO,
            }),
            Err(ReleaseError::ZeroShare)
        );
        // A duplicated (material, grade) bucket.
        static DUP: [ReleaseProduct; 2] = [
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Sand,
                share_permille: 500,
            },
            ReleaseProduct {
                material: MaterialId::GRANITE,
                grade: GrainGrade::Sand,
                share_permille: 500,
            },
        ];
        assert_eq!(
            validate_edge_products(&EdgeProducts {
                from: InvForm::Structure,
                to: InvForm::Loose,
                products: &DUP,
            }),
            Err(ReleaseError::DuplicateProduct)
        );
    }

    /// The grade ladder is exactly U4's five rungs, coarse → fine, and raw()
    /// round-trips — the packed record's 3-bit encoding depends on it.
    #[test]
    fn grain_grades_are_the_five_rung_ladder() {
        assert_eq!(GrainGrade::ALL.len(), GRAIN_GRADE_COUNT);
        for (i, g) in GrainGrade::ALL.iter().enumerate() {
            assert_eq!(g.raw() as usize, i);
            assert_eq!(GrainGrade::from_raw(g.raw()), Some(*g));
        }
        assert_eq!(GrainGrade::from_raw(GRAIN_GRADE_COUNT as u8), None);
    }

    /// **The split is remainder-exact**: for any table and any quantity, the
    /// per-product quantities re-sum (in split order) to the input to the bit.
    /// This is the itemisation==total invariant at its source; the weathering
    /// consumer and the probe both ride it.
    #[test]
    fn split_quantities_is_remainder_exact() {
        let products = MaterialId::GRANITE
            .release_products(InvForm::Structure, InvForm::Loose)
            .expect("granite declares a weathering spectrum");
        for qty in [0.017, 1.0, 3.719, 50.0, 6.094e-3] {
            let mut sum = 0.0f64;
            let mut n = 0;
            for (_, q) in split_quantities(products, qty) {
                sum += q;
                n += 1;
            }
            assert_eq!(n, products.len());
            // Exact by construction: the last product takes qty - (partial sum),
            // and re-summing in the same order reproduces qty bit-for-bit
            // (fl(S + (qty - S)) == qty whenever qty - S is exact, which
            // Sterbenz guarantees for S in [qty/2, 2*qty] — true whenever the
            // last product's share is <= 500 permille, as in every vanilla
            // table). Asserted, not assumed:
            assert!(
                sum == qty,
                "split of {qty} re-sums to {sum} (diff {:.3e})",
                (sum - qty).abs()
            );
        }
    }
}
