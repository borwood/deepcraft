//! **Slot `paleo_temperature`** — *what temperature did this cell see when this
//! unit was deposited?*
//!
//! - Owing system: **paleoclimate** (the heir is an epoch-indexed temperature
//!   curve; the *consumer* is the clastic strata pass, which selects each deep
//!   unit's member under its at-deposition formation context).
//! - Granularity: **value-level, per recorded deep unit** — because the heir's
//!   answer varies per unit even though the identity's does not. See below.
//! - Identity: [`identity_paleo_temperature`] — the column's **present-day**
//!   temperature, handed straight back.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::paleo_temperature`](field@super::Providers::paleo_temperature)).
//! This module holds the payload, the identity, and the honest statement of
//! what the identity is pretending.
//!
//! ## Why value-level per unit, when the identity is constant per column
//!
//! [`deposit_deep_history`](crate::geology) walks a column's recorded deep
//! units bottom-up and selects a member for each under the context it was
//! *deposited* under. The **aridity** axis of that context already reads the
//! recorder's own tag ([`deep_precip`](crate::geology)) — it is genuinely
//! at-deposition. Temperature, in the pre-seam code, did not: it read the
//! column's *present* climate (`ctx.temp_c`) for every unit. So on adjacent
//! lines of one function, one axis is the record's answer and the other is
//! today's — the asymmetry that made this seam visible (seam inventory #11).
//!
//! The identity value is therefore constant across a column's units, and the
//! naïve granularity rule — *never call a provider inside a loop to answer
//! something that does not change inside the loop* — would say "materialize it
//! once per column". That rule is about the **identity**, but granularity
//! follows the **heir** (journal/0060, journal/0061). A paleo-temperature curve
//! answers *per epoch*, and each deep unit carries its own
//! [`chapter`](crate::deeptime::recorder::DepUnit::chapter) — the tectonic epoch
//! it was deposited in. The heir's answer therefore varies per unit; collapsing
//! it to one value per column would erase exactly the epoch axis the curve
//! exists to express. So the slot is value-level per unit, the same shape
//! [`burial_temp_c`](super::burial_temp_c) took for the same reason.
//!
//! The cost is a single always-`None` branch per deep unit on the collapse
//! (per-chunk-load) path — no allocation, no plane — and the world it produces
//! is bit-for-bit the pre-seam world, because the identity returns the very
//! `ctx.temp_c` the old line read.

/// One recorded deep unit, as a paleo-temperature curve would see it: *where*
/// the column is, *when* the unit was deposited, and the present-day baseline
/// the identity hands back.
///
/// **What it deliberately does not carry.** Not the unit's facies or thickness —
/// the slot answers a *temperature of a place at a time*, and a temperature that
/// depended on what rock happened to record it would not be a climate. The
/// fields are exactly the query key of an epoch-indexed climate field plus the
/// identity's return; nothing is here that the heir cannot fill (the
/// `wave_energy` payload mistake journal/0060 named).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaleoUnit {
    /// Chunk-column coordinate — the spatial key a paleoclimate field indexes.
    pub cx: i64,
    /// Chunk-column coordinate — the spatial key a paleoclimate field indexes.
    pub cz: i64,
    /// The tectonic **chapter** (0-based) this unit was deposited in — the
    /// *when* a paleo-temperature curve indexes (tectonics.md § 3.3; chapter `c`
    /// spans a known Myr band before present). Always `0` when
    /// [`DeepConfig::tectonic_history`](crate::deeptime::grid::DeepConfig::tectonic_history)
    /// is off, which is harmless here: the identity ignores it, so byte-identity
    /// holds either way.
    pub chapter: u8,
    /// The column's **present-day** temperature (°C), column-quantized — what the
    /// identity returns unchanged, and the latitude baseline a curve perturbs by
    /// epoch. This is the upper-boundary datum the same way
    /// [`BuriedUnit::surface_temp_c`](super::burial_temp_c::BuriedUnit::surface_temp_c)
    /// is for the geotherm: a provider is a plain `fn` with no captured state, so
    /// a field is the only way the present climate reaches it.
    pub present_temp_c: f64,
}

/// **Identity for
/// [`Providers::paleo_temperature`](field@super::Providers::paleo_temperature)**:
/// the column's present-day temperature, handed straight back.
///
/// This is the pre-seam behaviour verbatim — `deposit_deep_history` read
/// `ctx.temp_c` (the column's *present* climate) as every deep unit's
/// at-deposition temperature. It is **not a true identity** in the neutral
/// sense: it is the *wrong quantity* (today's temperature standing in for the
/// epoch's), not a mathematical no-op like `parent_p`'s `1.0`. That is exactly
/// why it earns a seam — the constant it holds cannot masquerade as the rule
/// once the socket names an heir. `deep_precip`, the sibling aridity axis, reads
/// the record correctly, which is why the asymmetry is visible in adjacent
/// lines of one function (seam inventory #11, `stubs.md` § 6).
pub fn identity_paleo_temperature(unit: PaleoUnit) -> f64 {
    unit.present_temp_c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(present_temp_c: f64, chapter: u8) -> PaleoUnit {
        PaleoUnit {
            cx: 12,
            cz: -7,
            chapter,
            present_temp_c,
        }
    }

    /// The identity is the present-day temperature, bit for bit — the whole
    /// byte-identity claim of this seam, stated where it is decidable.
    #[test]
    fn the_identity_paleo_temp_is_the_present_temperature() {
        for t in [-31.0, 0.0, 14.25, 27.5, 40.0] {
            assert_eq!(
                identity_paleo_temperature(unit(t, 3)).to_bits(),
                t.to_bits()
            );
        }
    }

    /// The identity ignores everything a real curve would use — position and
    /// epoch. Stated as a test because "the identity is present temperature and
    /// nothing but" is the property the golden depends on.
    #[test]
    fn the_identity_depends_on_nothing_but_the_present_temperature() {
        let a = identity_paleo_temperature(unit(14.25, 0));
        let b = identity_paleo_temperature(PaleoUnit {
            cx: -9001,
            cz: 4242,
            chapter: 12,
            present_temp_c: 14.25,
        });
        assert_eq!(a.to_bits(), b.to_bits());
    }
}
