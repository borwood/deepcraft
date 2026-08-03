//! **`SubCell` — one deep cell's strata story, prepared to skin voxel columns**
//! (P11 slice 3, the near-path restructure; member-#0 design pass MM-3, built
//! WITH its consumer per journal/0129's withdrawal condition).
//!
//! Under the restructure a chunk-column no longer holds *the* record — it holds
//! **one `SubCell` per deep cell its columns' bilinear stencil touched** (always
//! 4 in a cell interior, up to 9 straddling cell edges), and a per-voxel-column
//! index saying which one skins each column
//! ([`ColumnRec::record_for`](crate::collapse::ColumnRec::record_for)).
//!
//! ## Why the type owns its `ColumnFill`
//!
//! The fill is built from **this** record in the constructor and is not
//! settable from outside, so *slicing record A with fill B is a type error* —
//! the exact guarantee the MM-3 report asked for. Likewise `soil` is the
//! regolith quantization of the **same** cell's `H` (the F2 bundle:
//! record + `H` + ledger move as one, `DeepField::cell_bundle`), so a column
//! cannot wear cell A's soil over cell B's strata.

use crate::fill::ColumnFill;
use crate::geology::StrataRec;

/// One deep cell's collapse-tier product for one chunk: the strata record
/// `run_strata` produced under that cell's own (record, `H`, ledger) bundle,
/// its derived [`ColumnFill`], and the same cell's regolith depth in voxels.
///
/// Fields are private; the accessors are the whole surface. A consumer that
/// needs "the chunk's record" no longer has one — it names a column
/// (`col.record_for(x, z)`) or the chunk centre (`col.centre_record()`).
#[derive(Debug, Clone)]
pub struct SubCell {
    strata: StrataRec,
    fill: ColumnFill,
    /// Regolith band depth (whole voxels) from the SAME cell's `H`.
    soil: u8,
    /// Row-major deep-cell index this SubCell was run for; `None` for the
    /// fallback SubCell (border wilds / no record grid), whose strata came
    /// from an empty deep record.
    cell: Option<u32>,
}

impl SubCell {
    /// Build from a record `run_strata` produced for one deep cell. The fill is
    /// derived HERE, from this strata and nothing else.
    pub(crate) fn new(strata: StrataRec, voxel_m: f64, soil: u8, cell: Option<u32>) -> Self {
        let fill = ColumnFill::build(&strata, voxel_m);
        SubCell {
            strata,
            fill,
            soil,
            cell,
        }
    }

    /// The strata record (collapse tier) this SubCell skins columns with.
    pub fn strata(&self) -> &StrataRec {
        &self.strata
    }

    /// The fill index derived from [`Self::strata`] — the only fill that may
    /// slice it.
    pub fn fill(&self) -> &ColumnFill {
        &self.fill
    }

    /// Regolith band depth, whole voxels, from the same cell's `H` (the F2
    /// bundle). Applies where this SubCell's record is empty (legacy soil band).
    pub fn soil(&self) -> u8 {
        self.soil
    }

    /// The row-major deep-cell index this SubCell was run for, or `None` for
    /// the fallback (wilds / no record grid). This is what lets an instrument
    /// ask "which parent cell did this column realize?" without re-deriving
    /// the dither.
    pub fn cell_index(&self) -> Option<u32> {
        self.cell
    }

    /// Rough heap footprint (bytes) of this SubCell's owned buffers — the
    /// runtime-residency line the design audit's RETURN spec asks for
    /// (`column_cache` is runtime-resident).
    pub fn heap_bytes(&self) -> usize {
        self.strata.events.capacity() * std::mem::size_of::<crate::geology::StrataEvent>()
            + self.fill.heap_bytes()
    }
}
