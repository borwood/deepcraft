//! **Slot `outcrop_at`** — *which rock is outcropping at this cell?*
//!
//! - Owing system: **structural**.
//! - Granularity: **value-level**, per cell per epoch.
//! - Identity: [`identity_outcrop_at`] — the top of the record.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::outcrop_at`](field@super::Providers::outcrop_at)); this module holds
//! the identity and whatever payload the slot needs. This one needs none: the
//! call site already had a value of exactly the right shape
//! (`Option<&DepUnit>`), which is why it was the cheapest seam to convert.

use crate::deeptime::lithology::{Litho, exposed_litho};
use crate::deeptime::recorder::DepUnit;

/// **Identity for [`Providers::outcrop_at`](field@super::Providers::outcrop_at)**: the
/// top of the record, or [`Litho::Basement`] when the column has been stripped
/// past its whole sedimentary history. This is the pre-seam
/// [`exposed_litho`](crate::deeptime::lithology::exposed_litho), called
/// unchanged, under the slot's name so the identity is a *thing* and not a
/// description.
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
pub fn identity_outcrop_at(units: Option<&DepUnit>) -> Litho {
    exposed_litho(units)
}
