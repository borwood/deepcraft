//! **Slot `outcrop_at`** — *which rock is outcropping at this cell?*
//!
//! - Owing system: **structural**.
//! - Granularity: **value-level**, per cell per epoch.
//! - Identity: [`identity_outcrop_at`] — the top of the record.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::outcrop_at`](super::Providers::outcrop_at)); this module holds
//! the identity and whatever payload the slot needs. This one needs none: the
//! call site already had a value of exactly the right shape
//! (`Option<&DepUnit>`), which is why it was the cheapest seam to convert.

use crate::deeptime::lithology::{Litho, exposed_litho};
use crate::deeptime::recorder::DepUnit;

/// **Identity for [`Providers::outcrop_at`](super::Providers::outcrop_at)**: the
/// top of the record, or [`Litho::Basement`] when the column has been stripped
/// past its whole sedimentary history. This is the pre-seam
/// [`exposed_litho`](crate::deeptime::lithology::exposed_litho), called
/// unchanged, under the slot's name so the identity is a *thing* and not a
/// description.
///
/// ## Why this is a wrapper and not a `pub use`
///
/// It *was* a re-export, and that made [`Providers::is_identity`](super::Providers::is_identity)
/// report the **default set as non-identity** — which the refactor that split
/// this module surfaced and the flatter code had been hiding.
///
/// `exposed_litho` is `#[inline]`, so rustc may instantiate it in **several
/// codegen units**, and each instance has its own address. Slot identity is
/// decided by comparing `fn` addresses (there is no other way to compare
/// function pointers), so two `Providers::default()` values built in two
/// codegen units could hold two different addresses *for the same function* and
/// compare unequal. The other three identities are plain, non-`#[inline]`
/// crate-local functions, which are codegened once and therefore have exactly
/// one address each; this wrapper gives `outcrop_at` the same property.
///
/// **The rule this establishes: a slot's identity must be a plain, non-inline
/// function defined in this module.** Never a `pub use` of someone else's
/// function, whose inlining attributes are not ours to control.
///
/// It costs nothing. A provider is *always* called through a pointer, so it is
/// never inlined at the call site anyway; the wrapper's own body inlines
/// `exposed_litho` and the result is the same code behind the same indirection.
pub fn identity_outcrop_at(units: Option<&DepUnit>) -> Litho {
    exposed_litho(units)
}
