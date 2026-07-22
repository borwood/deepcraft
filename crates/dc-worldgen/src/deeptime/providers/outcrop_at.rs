//! **Slot `outcrop_at`** — *which rock is outcropping at this cell?*
//!
//! - Owing system: **structural**.
//! - Granularity: **value-level**, per cell per epoch.
//! - Identity: [`identity_outcrop_at`] — the lithology dominating the record's
//!   near-surface window.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::outcrop_at`](field@super::Providers::outcrop_at)); this module holds
//! the identity and whatever payload the slot needs. This one needs none: the
//! call site hands the recorded column's unit slice (`&[DepUnit]`), which the
//! identity reads directly. (It was once the single top unit, `Option<&DepUnit>`;
//! journal/0068 widened it to the slice so the identity can weigh the whole
//! near-surface window instead of trusting the topmost bed — the thickness-
//! dominance rule.)

use crate::deeptime::lithology::{Litho, exposed_litho};
use crate::deeptime::recorder::DepUnit;

/// **Identity for [`Providers::outcrop_at`](field@super::Providers::outcrop_at)**: the
/// lithology dominating the topmost
/// [`OUTCROP_DOMINANCE_WINDOW_M`](crate::deeptime::lithology::OUTCROP_DOMINANCE_WINDOW_M)
/// of the record, or [`Litho::Basement`] when the column has been stripped past
/// its whole sedimentary history. This is
/// [`exposed_litho`](crate::deeptime::lithology::exposed_litho), called under the
/// slot's name so the identity is a *thing* and not a description.
///
/// ## A rule that used to live here, and no longer needs to
///
/// This was briefly load-bearing in a way it should never have been. When slot
/// identity was decided by **comparing `fn` addresses** against
/// `Providers::default()`, this identity had to be a plain, non-`#[inline]`
/// function defined in *this* module: `exposed_litho` is `#[inline]`, rustc may
/// instantiate it once per codegen unit, and each instance has its own address,
/// so two `Providers::default()` values could hold two addresses for the same
/// function and compare unequal. They did (corrections #32). The remedy was a
/// wrapper plus a rule in a docstring — and a rule in a docstring cannot fail a
/// build.
///
/// The `Option<fn>` slot retired the rule instead of enforcing it: `None` is
/// identity, no address is taken anywhere, and an identity may now be
/// `#[inline]`, a re-export, or anything else without consequence. The wrapper
/// survives only because it gives the identity a *name under the slot's own
/// module*, which is what the module layout is for.
pub fn identity_outcrop_at(units: &[DepUnit]) -> Litho {
    exposed_litho(units)
}
