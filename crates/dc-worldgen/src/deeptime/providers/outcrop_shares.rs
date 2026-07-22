//! **Slot `outcrop_shares`** — *how much of each rock fills the near-surface
//! window at this cell?*
//!
//! - Owing system: **structural**.
//! - Granularity: **value-level**, per cell per epoch.
//! - Identity: [`identity_outcrop_shares`] — the per-[`Litho`] thickness shares of
//!   the record's near-surface [`OUTCROP_DOMINANCE_WINDOW_M`](crate::deeptime::lithology::OUTCROP_DOMINANCE_WINDOW_M).
//!
//! ## Why this slot exists next to `outcrop_at`, and is not a duplicate
//!
//! `outcrop_at` seams the **verdict** — the single [`Litho`] outcropping at a cell.
//! For a long time that was all any consumer wanted. But erosion never wanted the
//! label; it wanted a *rate*, and it was forming the rate by taking the window's
//! argmax and looking that one label up in a susceptibility table. That collapse is
//! an S-4 defect (journal/0072): a deposit that thins gradually across country
//! flips the plurality winner at one contour, stepping the erosion rate
//! discontinuously along a spatially coherent line — exactly what S-4 forbids
//! reaching the eye.
//!
//! The cure is to seam the **quantity** the verdict summarises — the share vector —
//! and let erosion blend the table by it (S-5's *"seam the quantity, not the
//! verdict"* corollary, earned by `burial_temp_c`). The verdict is then, by
//! construction, the argmax of this quantity: `outcrop_at` and `outcrop_shares` are
//! two faces of one [`window_walk`](crate::deeptime::lithology::exposed_litho), not
//! two mechanisms. They share a single heir — **structural deformation** — and are
//! a **pinned pair**: once beds dip, the heir supplies the dipped shares here, and
//! the verdict slot's answer must remain their argmax (retire together, or the
//! world's rate field and its outcrop map disagree about where a bed is). The
//! shape-teacher lesson for the future `CoarseField<T>` (audit part 2): the coarse
//! field's fine accessor exposes this **interpolable share vector**, and the
//! verdict is `argmax ∘ sample` — never a separately stored label.

use crate::deeptime::lithology::{Litho, exposed_shares};
use crate::deeptime::recorder::DepUnit;

/// **Identity for [`Providers::outcrop_shares`](field@super::Providers::outcrop_shares)**:
/// the per-[`Litho`] shares of the record's near-surface window
/// ([`exposed_shares`], summing to `1.0`, deficit below a short record charged to
/// [`Litho::Basement`]). Called under the slot's name so the identity is a *thing*
/// and not a description; a `None` slot routes here.
pub fn identity_outcrop_shares(units: &[DepUnit]) -> [f64; Litho::COUNT] {
    exposed_shares(units)
}
