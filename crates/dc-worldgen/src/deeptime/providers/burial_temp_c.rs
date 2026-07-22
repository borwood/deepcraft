//! **Slot `burial_temp_c`** — *what temperature has this buried unit seen?*
//!
//! - Owing system: **structural** (the heir is a geotherm, and the crustal
//!   thermal field is tectonics' to answer — `t_crust` is a tectonics plane).
//!   The *consumer* is diagenesis, which is exactly the distinction the grouping
//!   records: slots are filed under who will **answer**, not who asks.
//! - Granularity: **value-level**, per candidate unit, at run finalize only.
//! - Identity: [`identity_burial_temp_c`] — the **degenerate geotherm**
//!   (0 °C at the surface, 1 °C/m), i.e. the answer *is* the overburden in
//!   metres, so a threshold on temperature is bit-for-bit the pre-seam threshold
//!   on burial depth.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::burial_temp_c`](field@super::Providers::burial_temp_c)). This module
//! holds the payloads, the identity, and the honest statement of what the
//! identity is pretending.
//!
//! ## Why the slot asks for a temperature and not for "is this coal?"
//!
//! The obvious shape was a predicate — `is_coalified(unit) -> bool` — and it is
//! simpler: one call, one answer, no units to argue about. It was rejected, and
//! the argument is `docs/spines.md` S-8, *one quantity, many regimes*.
//!
//! Coalification is not a phenomenon with its own control. It is one threshold
//! on a **thermal maturity ladder** that keeps going: peat → lignite →
//! sub-bituminous → bituminous → anthracite → graphite, and then straight on
//! into metamorphic grade (`stubs.md` § 4, and the `exhum` / `t_crust` planes
//! that `spines.md` § 3 lists as built-and-unconsumed). A predicate slot answers
//! exactly one of those questions and composes with none of the others: landing
//! a geotherm would mean supplying a *coal* provider, and then a *rank*
//! provider, and then a *grade* provider, each re-deriving the same temperature
//! behind its own signature. A temperature slot answers all of them once, and
//! every rung of the ladder becomes a constant compared against it.
//!
//! It costs one thing, and the cost is paid in this module's docstrings: the
//! identity has to answer in °C when what it actually knows is metres. See
//! [`identity_burial_temp_c`]. That mismatch is the same one
//! [`depth_to_water`](super::depth_to_water) carries deliberately — *the
//! question is the slot's name, and the distance between the question and
//! today's answer is the seam's most useful output.*

/// The column whose record is being walked at finalize — everything the
/// coalification pass knows that is a property of the *place* rather than of the
/// unit.
///
/// Split from [`BuriedUnit`] because it is computed once per column and copied
/// into every unit's payload; the provider never sees this type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BurialColumn {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
    /// Air temperature (°C) at this column's present surface, from the sim's one
    /// climate model
    /// ([`climate::air_temp_c`](crate::deeptime::climate::air_temp_c)) — a
    /// latitude gradient minus an altitude lapse.
    pub surface_temp_c: f64,
}

/// One unit of the record, as a geotherm sees it: a slab of known thickness
/// lying under a known thickness of section, in a known column.
///
/// **What it deliberately does not carry.** Not the unit's facies — the slot
/// answers *temperature*, and a temperature that depended on whether the rock
/// was peat would not be a geotherm. Not the unit's age either, though a real
/// coalification model wants time-at-temperature (Lopatin's TTI, vitrinite
/// reflectance) rather than peak temperature alone: the record stores a tectonic
/// chapter, not a duration, so an age field now would be a payload the heir
/// cannot fill — the `wave_energy` mistake journal/0060 named.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuriedUnit {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
    /// Air temperature (°C) at the column's surface — the geotherm's **upper
    /// boundary condition**, which is why it is here rather than left for the
    /// provider to re-derive from `gy` (it could not: providers are plain `fn`s
    /// with no captured state, so a payload field is the only way in).
    pub surface_temp_c: f64,
    /// Metres of section recorded **above** this unit — `Σ` of the thicknesses
    /// of every later unit. This is depth below the *present* surface, not
    /// maximum depth over the unit's history: the record keeps no memory of
    /// section that was deposited and then stripped again, so a unit that was
    /// once deeply buried and later exhumed reads as shallow. Real coal rank is
    /// irreversible and would not. Filed in the journal, not fixed here.
    pub overburden_m: f64,
    /// The unit's own thickness (m), so an heir integrating a gradient across
    /// the slab can use its mid-depth rather than its top.
    pub thickness_m: f64,
}

/// **Identity for [`Providers::burial_temp_c`](field@super::Providers::burial_temp_c)**:
/// return the overburden, in metres, as though it were degrees.
///
/// This is a **degenerate geotherm** — surface datum 0 °C, gradient exactly
/// 1 °C/m — chosen for one reason: it makes `t >= onset` bit-for-bit identical
/// to the pre-seam `overburden_m >= COAL_BURIAL_M`, so a world with no heir
/// generates exactly the world that existed before this seam did. `x >= 8.0` and
/// `f(x) >= f(8.0)` agree for `f` the identity and for no interesting `f` at
/// all; anything more physical here — a real 0.025 °C/m gradient, a non-zero
/// surface datum — puts a rounding step between the two comparisons and turns a
/// *proof* of byte-identity into a hope.
///
/// **The gradient is 40× Earth's, and that is not a claim about this planet.**
/// It is the arithmetic that keeps the identity honest, in the same way
/// [`identity_wet_index`](super::identity_wet_index) answers a dimensionless
/// index to a slot named `depth_to_water`. Units are the thing this seam has not
/// yet earned, and saying so in the type is better than a plausible-looking
/// number that quietly means nothing.
///
/// **Retire this and [`COAL_ONSET_C`](crate::deeptime::COAL_ONSET_C) together.**
/// They are one calibration in two places: the onset constant is a temperature
/// only in the sense that this function is a geotherm. An heir that supplies a
/// real geotherm and leaves the onset at its identity value would promote
/// essentially the whole record to coal — which is why both carry the same
/// warning and why a test pins their agreement.
pub fn identity_burial_temp_c(unit: BuriedUnit) -> f64 {
    unit.overburden_m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(overburden_m: f64, thickness_m: f64) -> BuriedUnit {
        BuriedUnit {
            index: 7,
            gx: 3,
            gy: 4,
            surface_temp_c: 14.25,
            overburden_m,
            thickness_m,
        }
    }

    /// The identity is the overburden, bit for bit — the whole byte-identity
    /// claim of this seam, stated where it is decidable.
    #[test]
    fn the_identity_burial_temp_is_the_overburden_in_metres() {
        for m in [0.0, 0.05, 7.999_999_999, 8.0, 8.000_000_001, 1_234.5] {
            assert_eq!(identity_burial_temp_c(unit(m, 0.4)).to_bits(), m.to_bits());
        }
    }

    /// The identity ignores everything a real geotherm would use — surface
    /// temperature, position, the slab's own thickness. Stated as a test because
    /// "the identity is depth and nothing but depth" is the property the golden
    /// depends on, and it is cheaper to assert here than to re-derive from a
    /// world hash.
    #[test]
    fn the_identity_depends_on_nothing_but_the_overburden() {
        let a = identity_burial_temp_c(unit(12.0, 0.4));
        let b = identity_burial_temp_c(BuriedUnit {
            index: 90_001,
            gx: 11,
            gy: 12,
            surface_temp_c: -31.0,
            overburden_m: 12.0,
            thickness_m: 44.0,
        });
        assert_eq!(a.to_bits(), b.to_bits());
    }

    /// A threshold on the identity temperature **is** the pre-seam threshold on
    /// burial depth, at and around the shipped value — including the boundary,
    /// which is the only place a `>=` can disagree with itself.
    #[test]
    fn thresholding_the_identity_is_thresholding_the_burial_depth() {
        for m in [0.0, 4.0, 7.999_999_999, 8.0, 8.000_000_001, 100.0] {
            assert_eq!(
                identity_burial_temp_c(unit(m, 1.0)) >= crate::deeptime::COAL_ONSET_C,
                m >= crate::deeptime::COAL_BURIAL_M,
                "the identity geotherm must reproduce the burial test at {m} m"
            );
        }
    }
}
