//! **The closed set of occupancy modes — the form vocabulary**
//! (`docs/design/material-behavior.md` §2, "Forms — the closed set of occupancy
//! modes (the machine)").
//!
//! **Moved here from `dc-worldgen::deeptime::inventory` by FS-A (2026-08-02),
//! and the move is forced, not cosmetic.** Release spectra are **edge-keyed
//! product tables declared on the material definition** (U7 ruling, R2:
//! `material-behavior.md` §3), and `materials.md` DECIDED 2026-07-22 makes such
//! declarations *"the ONE authority both the runtime simulation and deep-time's
//! bulk arithmetic consult"*. The runtime sim lives above `dc-core` and cannot
//! see `dc-worldgen`, so the edge key's vocabulary — the forms — must live here
//! beside [`MaterialId`](super::MaterialId). The deep tier's `inventory` module
//! re-exports these types, so every existing `deeptime::inventory::InvForm`
//! path still resolves; nothing about the type changed but its address.

/// The **form** a material-portion occupies volume in (material-behavior.md §2).
///
/// `Structure`/`Loose`/`PoreFill`/`Fluid` are the storable roles. `Void` is **not
/// a storable role** — it is the unoccupied complement (§2) — but it *is* a legal
/// **edge endpoint** (§3: edges to/from void change occupancy), so a fact may
/// name it as a source or destination (dissolution is `… → Void`). Invariant: a
/// stored portion never has `form == Void`; the inventory's builders and ctx
/// never create one.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InvForm {
    /// Coherent, load-bearing framework (the `R`/structural stock).
    Structure,
    /// Granular, unreserved volume that obeys gravity (the `H`/regolith stock).
    Loose,
    /// Material held inside another's reserved-but-unfilled pore capacity.
    PoreFill,
    /// Liquid in pores + open space. **Accommodated, never stored by the identity
    /// default** — the water model derives it (§10, S-2).
    Fluid,
    /// The unoccupied complement — **edge endpoint only, never a stored portion**.
    Void,
}

/// The number of [`InvForm`] variants — material-behavior.md §3's *"5 forms → 20
/// directed edges"*. It is the **radix** an `EdgeId` packs a form in, so it is a
/// constant of the encoding and not just a count.
pub const FORM_COUNT: u8 = 5;

impl InvForm {
    /// This form's position in the closed set, `0..FORM_COUNT`.
    #[inline]
    pub const fn raw(self) -> u8 {
        match self {
            InvForm::Structure => 0,
            InvForm::Loose => 1,
            InvForm::PoreFill => 2,
            InvForm::Fluid => 3,
            InvForm::Void => 4,
        }
    }

    /// The form at position `raw`, or `None` when out of the closed set.
    #[inline]
    pub const fn from_raw(raw: u8) -> Option<InvForm> {
        match raw {
            0 => Some(InvForm::Structure),
            1 => Some(InvForm::Loose),
            2 => Some(InvForm::PoreFill),
            3 => Some(InvForm::Fluid),
            4 => Some(InvForm::Void),
            _ => None,
        }
    }

    /// Short name for the edge dictionary and for provenance output.
    pub const fn name(self) -> &'static str {
        match self {
            InvForm::Structure => "structure",
            InvForm::Loose => "loose",
            InvForm::PoreFill => "pore-fill",
            InvForm::Fluid => "fluid",
            InvForm::Void => "void",
        }
    }
}
