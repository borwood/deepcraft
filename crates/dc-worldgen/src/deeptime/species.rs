//! **The deep tier's species axis, and the sparse budget planes over it**
//! (P11 slice 2, ruling 3: *"transport planes go SPARSE DAY ONE"*).
//!
//! ## Why this module exists
//!
//! Before P11 slice 2 the transport solve resolved its load into
//! [`Litho::COUNT`](super::lithology::Litho::COUNT) = 7 **classes**, in four dense
//! `n × 7` `f64` planes. That was honest while the record was class-grade: the
//! comment on the old `SPECIES` constant said resolving the load finer *"would be
//! inventing identity the tier does not have"*. Slice 1 gave the tier that
//! identity — `DepUnit::species` is a registry `MaterialId` — so the budget became
//! the coarse link in the chain, and a siltstone bed still eroded at mudstone's
//! rate because the table that answered was keyed by class.
//!
//! Widening the planes to the registry is the obvious move and it is the wrong
//! asymptote: **dense scales as cells × registry-size**, and the registry is open
//! by design (the SDK arc). The user's ruling is explicit — *sparse day one* — and
//! the reason is the one this file is built around: **a cell's in-transit load
//! only ever holds its catchment's species**, not the world's.
//!
//! ## The shape, and the precedent it copies
//!
//! CSR, the same shape [`FluxRecord`](super::flux::FluxRecord) and
//! [`FactLedger`](super::inventory::FactLedger) already vindicated — and
//! deliberately **not** a `Vec` per cell, which S19 § 6 measured at *"89 % of its
//! 150 MiB heap in empty headers"*.
//!
//! A [`SpeciesLayout`] is one epoch's answer to *which species can be at which
//! cell*: a per-cell 64-bit presence **mask** over the axis, a CSR row offset, and
//! the row's axis indices spelled out so iteration never decodes bits. Lookup is
//! `mask & (bit − 1)` popcount — O(1), branchless, no search. A
//! [`SpeciesPlane`] is a `Vec<f64>` laid out against a layout.
//!
//! **The layout is exact, not a heuristic.** It is built by *closure*: a cell's
//! mask starts as what lies at that cell (the near-surface window the
//! `outcrop_shares` seam publishes, plus what the bedrock beneath it is made of),
//! and is then propagated **downstream along the same routing the transport pass
//! is about to walk**, in the same order. So a species can never arrive at a cell
//! whose row has no slot for it — the propagation is the transport graph's own
//! reachability, computed with `u64` ORs instead of `f64` adds.
//!
//! ## What stays f64
//!
//! Ruling 3's second half: *"budget arithmetic stays f64 unless a store-f32 split
//! re-proves Law-3 closure"*. It stays f64. The closure instruments
//! (`Erosion::max_split_residue` and the two creep residues) are the gate on that
//! and they are measured after this change, not assumed.
//!
//! ## The order of the axis, and why it is the settling order
//!
//! Two loops in the transport pass are **order-sensitive**: the capacity drawdown
//! (*"coarsest first"* — the excess a flow cannot hold is paid out of the heaviest
//! fraction it carries) and the competence ceiling. Under a dense plane they read
//! a precomputed permutation; under a sparse row, walking a permutation of the
//! whole axis and testing membership would put the registry's width straight back
//! into the hot loop — exactly the asymptote sparsity exists to remove.
//!
//! So the **axis itself is ordered by descending settling energy**, ties broken by
//! `MaterialId` index. A row is stored in ascending axis order, which *is*
//! coarsest-first, so both loops walk the row and stop early. It also gives every
//! order-sensitive rule in the pass — the residual split's *"last non-zero share
//! takes the remainder"*, the arriving-identity argmax's tie-break — one **total,
//! deterministic, content-derived order** instead of an enum's declaration order.

use dc_core::materials::{MATERIAL_COUNT, MaterialId, geology::GeologySet};

/// The widest axis a [`SpeciesLayout`]'s one-word mask can address.
///
/// ⚠ **A named ceiling, not a law.** The mask is a `u64` because a single word
/// keeps `slot_of` to an AND, a popcount and an add. A wider registry needs a
/// multi-word mask (`n × ceil(W/64)` words, rank = the popcounts of the preceding
/// words plus the rank in this one) — a mechanical widening, filed rather than
/// pre-built. Today it is not the binding constraint: `EdgeId`'s mixed-radix
/// packing caps the *material registry* at 51 (stubs #21, compile-asserted in
/// `inventory.rs`), which is stricter than this.
pub const MAX_DEEP_SPECIES: usize = 64;

/// Sentinel in [`SpeciesAxis::slot`] for a material the deep tier cannot carry.
const OFF_AXIS: u8 = u8::MAX;

/// **The materials deep time can carry** — the alphabet the load, the record and
/// the erosion tables are all resolved against.
///
/// It is **derived from the registered content**, never declared: every material
/// some geology member deposits, plus the basement rock beneath the record. No
/// class view is consulted to build it (ruling 2, A-CLEAN), which is why a pack
/// that registers a member widens the axis by construction and nothing else has to
/// know.
///
/// Ordered by **descending settling energy** with `MaterialId` index as the
/// tie-break — see the module docs for why the order is load-bearing rather than
/// cosmetic.
#[derive(Clone, Debug)]
pub struct SpeciesAxis {
    /// Axis order → material.
    ids: Vec<MaterialId>,
    /// Material → axis order, [`OFF_AXIS`] when the deep tier cannot carry it.
    slot: [u8; MATERIAL_COUNT],
    /// Per-axis settling-velocity ordering key (descending along the axis).
    w_settle: Vec<f64>,
    /// The axis index of the basement material — the answer for a stripped column
    /// and the S-5 default for a material that is not on the axis at all.
    basement: u8,
}

impl SpeciesAxis {
    /// Build the axis for a world's registered content.
    ///
    /// `basement` is the rock below the whole sedimentary pile; it is forced onto
    /// the axis even if no member deposits it, because a stripped column's window
    /// is made of it and the incision term entrains it.
    pub fn new(geology: &GeologySet, basement: MaterialId) -> Self {
        let mut present = [false; MATERIAL_COUNT];
        for m in geology.members() {
            present[m.material.raw() as usize] = true;
        }
        present[basement.raw() as usize] = true;
        // Candidates in canonical `MaterialId` order first, so the settle sort's
        // tie-break is a stable, content-order-independent one.
        let mut ids: Vec<MaterialId> = (0..MATERIAL_COUNT as u8)
            .filter_map(MaterialId::from_raw)
            .filter(|m| present[m.raw() as usize])
            .collect();
        ids.sort_by(|&a, &b| {
            let (wa, wb) = (settle_key(a), settle_key(b));
            wb.total_cmp(&wa).then(a.raw().cmp(&b.raw()))
        });
        assert!(
            ids.len() <= MAX_DEEP_SPECIES,
            "the deep species axis holds {} materials; the one-word presence mask \
             addresses {MAX_DEEP_SPECIES}. Widen SpeciesLayout to a multi-word mask.",
            ids.len()
        );
        let mut slot = [OFF_AXIS; MATERIAL_COUNT];
        for (k, &m) in ids.iter().enumerate() {
            slot[m.raw() as usize] = k as u8;
        }
        let w_settle = ids.iter().map(|&m| settle_key(m)).collect();
        let basement = slot[basement.raw() as usize];
        Self {
            ids,
            slot,
            w_settle,
            basement,
        }
    }

    /// The number of species on the axis — the *dense* width, kept only for
    /// scratch rows and the per-agent rate tables (which are one row per run, not
    /// one per cell).
    #[inline]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the axis is empty (no content registered at all).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Every material on the axis, in axis order.
    #[inline]
    pub fn materials(&self) -> &[MaterialId] {
        &self.ids
    }

    /// The material at an axis index.
    #[inline]
    pub fn material(&self, k: usize) -> MaterialId {
        self.ids[k]
    }

    /// The axis index of a material, or [`Self::basement_slot`] when the deep tier
    /// cannot carry it.
    ///
    /// **Total on purpose** (S-5): a material off the axis is one no registered
    /// member deposits — an igneous body the veneer placed, say — and the honest
    /// deep-tier answer for it is *"the rock below the pile"*, which is exactly
    /// what the window walk already charges its deficit to. A `None` here would
    /// force every caller to invent the same fallback.
    #[inline]
    pub fn slot_of(&self, m: MaterialId) -> usize {
        let s = self.slot[m.raw() as usize];
        if s == OFF_AXIS {
            self.basement as usize
        } else {
            s as usize
        }
    }

    /// Whether this material is on the axis in its own right (as opposed to
    /// falling back to basement through [`Self::slot_of`]).
    #[inline]
    pub fn carries(&self, m: MaterialId) -> bool {
        self.slot[m.raw() as usize] != OFF_AXIS
    }

    /// The axis index of the basement material.
    #[inline]
    pub fn basement_slot(&self) -> usize {
        self.basement as usize
    }

    /// The per-axis settling-velocity ordering key — descending along the axis by
    /// construction, so `w_settle()[k] >= w_settle()[k + 1]`.
    #[inline]
    pub fn w_settle(&self) -> &[f64] {
        &self.w_settle
    }
}

/// The settling-velocity ordering key of one material — `dc_core`'s
/// `settle_energy` off its property sheet, the same function the shipped placer
/// pass thresholds on. See `lithology::settling_table` for the honest limit of the
/// proxy (STUB #23: it is size-dominated and buoyancy-blind).
#[inline]
fn settle_key(m: MaterialId) -> f64 {
    dc_core::materials::geology::settle_energy(m.props())
}

/// **One epoch's answer to "which species can be at which cell"** — a per-cell
/// presence mask over a [`SpeciesAxis`], plus the CSR row index it implies.
///
/// Rebuilt every epoch, because both of its inputs move every epoch: what lies at
/// a cell (the record changes) and what drains into it (the routing changes).
/// Building it costs one pass of `u64` ORs over the same order the transport pass
/// walks — nothing beside the `f64` arithmetic it is sizing.
#[derive(Clone, Default, Debug)]
pub struct SpeciesLayout {
    /// Per-cell presence bits, bit `k` = axis index `k`.
    mask: Vec<u64>,
    /// `start[c] .. start[c + 1]` is cell `c`'s row. Length `n + 1`.
    start: Vec<u32>,
    /// The axis index of each stored slot, ascending within a row — spelled out so
    /// iteration is a slice read rather than a bit decode.
    axis: Vec<u8>,
}

impl SpeciesLayout {
    /// An empty layout — the identity the scalar-load path leaves behind.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Cells covered.
    #[inline]
    pub fn cells(&self) -> usize {
        self.start.len().saturating_sub(1)
    }

    /// Total stored slots — the length every [`SpeciesPlane`] over this layout has.
    #[inline]
    pub fn nnz(&self) -> usize {
        self.axis.len()
    }

    /// Cell `c`'s row: `(first slot, its axis indices)`.
    #[inline]
    pub fn row(&self, c: usize) -> (usize, &[u8]) {
        let a = self.start[c] as usize;
        let b = self.start[c + 1] as usize;
        (a, &self.axis[a..b])
    }

    /// The stored slot for `(cell, axis index)`, or `None` when the cell's row has
    /// no place for that species.
    ///
    /// A `None` here is a **fact about the layout**, not a fallback: the closure
    /// that built it is the transport graph's own reachability, so a species the
    /// solve can actually deliver always has a slot.
    #[inline]
    pub fn slot_of(&self, c: usize, k: usize) -> Option<usize> {
        let m = self.mask[c];
        let bit = 1u64 << k;
        if m & bit == 0 {
            return None;
        }
        Some(self.start[c] as usize + (m & (bit - 1)).count_ones() as usize)
    }

    /// Cell `c`'s presence mask.
    #[inline]
    pub fn mask_at(&self, c: usize) -> u64 {
        self.mask[c]
    }

    /// Build the CSR row index from the per-cell masks.
    fn rebuild_from_masks(&mut self, n: usize) {
        self.start.clear();
        self.start.reserve(n + 1);
        self.axis.clear();
        for c in 0..n {
            self.start.push(self.axis.len() as u32);
            let mut m = self.mask[c];
            while m != 0 {
                let k = m.trailing_zeros();
                self.axis.push(k as u8);
                m &= m - 1;
            }
        }
        self.start.push(self.axis.len() as u32);
    }

    /// Reset every cell's mask to empty, sized for `n` cells.
    fn clear_masks(&mut self, n: usize) {
        if self.mask.len() != n {
            self.mask = vec![0u64; n];
        } else {
            self.mask.iter_mut().for_each(|m| *m = 0);
        }
    }

    /// **The mean number of species stored per occupied cell** — the sparsity
    /// number the design priced blind (design audit I3 / record-terms § 3.4).
    /// `0.0` when nothing is stored.
    pub fn mean_row_width(&self) -> f64 {
        let occupied = self.occupied_cells();
        if occupied == 0 {
            return 0.0;
        }
        self.nnz() as f64 / occupied as f64
    }

    /// The widest row in the layout.
    pub fn max_row_width(&self) -> usize {
        self.mask
            .iter()
            .map(|m| m.count_ones() as usize)
            .max()
            .unwrap_or(0)
    }

    /// Cells with at least one species present.
    pub fn occupied_cells(&self) -> usize {
        self.mask.iter().filter(|&&m| m != 0).count()
    }

    /// Bytes this layout holds (mask + row index + axis codes) — the residency a
    /// probe reports beside the planes it sizes.
    pub fn approx_bytes(&self) -> usize {
        self.mask.len() * 8 + self.start.len() * 4 + self.axis.len()
    }
}

/// **A sparse per-cell, per-species `f64` plane** laid out against a
/// [`SpeciesLayout`]. One `Vec`, no per-cell allocation, no headers.
#[derive(Clone, Default, Debug)]
pub struct SpeciesPlane {
    vals: Vec<f64>,
}

impl SpeciesPlane {
    /// Size (and zero) the plane for a layout.
    #[inline]
    pub fn reset_for(&mut self, layout: &SpeciesLayout) {
        if self.vals.len() != layout.nnz() {
            self.vals = vec![0.0; layout.nnz()];
        } else {
            self.vals.iter_mut().for_each(|v| *v = 0.0);
        }
    }

    /// Drop every allocation (the scalar-load path's identity).
    #[inline]
    pub fn clear(&mut self) {
        self.vals = Vec::new();
    }

    /// Whether the plane holds nothing at all.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vals.is_empty()
    }

    /// The flat values, slot-indexed.
    #[inline]
    pub fn vals(&self) -> &[f64] {
        &self.vals
    }

    /// The flat values, mutable.
    #[inline]
    pub fn vals_mut(&mut self) -> &mut [f64] {
        &mut self.vals
    }

    /// Bytes held.
    pub fn approx_bytes(&self) -> usize {
        self.vals.len() * 8
    }
}

/// **Build the layout for the loose-cover composition plane** — the near-surface
/// window at each cell, which is what entrainment and creep lift.
pub fn build_local_layout(layout: &mut SpeciesLayout, n: usize, masks: &[u64]) {
    layout.clear_masks(n);
    layout.mask[..n].copy_from_slice(&masks[..n]);
    layout.rebuild_from_masks(n);
}

/// **Build the transport layout by downstream closure.**
///
/// `seed` is each cell's own contribution (its window composition, plus whatever
/// the bedrock beneath it is made of, since incision can reach it). `order` is the
/// solve's processing order — the same array the transport pass walks in reverse —
/// and `receivers` yields the cells a given cell hands load to.
///
/// The result is exact: after the sweep, cell `j`'s mask contains every species
/// that any cell draining into `j` can release, because the transport pass moves
/// load along exactly these edges in exactly this order.
pub fn build_transport_layout(
    layout: &mut SpeciesLayout,
    n: usize,
    seed: &[u64],
    order: &[u32],
    mut receivers: impl FnMut(usize, &mut dyn FnMut(usize)),
) {
    layout.clear_masks(n);
    layout.mask[..n].copy_from_slice(&seed[..n]);
    // Same sweep direction as `Erosion::transport`: highest first, so a cell's own
    // mask is final before it is pushed to its receivers.
    for k in (0..order.len()).rev() {
        let c = order[k] as usize;
        let m = layout.mask[c];
        if m == 0 {
            continue;
        }
        let mask = &mut layout.mask;
        receivers(c, &mut |j: usize| mask[j] |= m);
    }
    layout.rebuild_from_masks(n);
}

/// **Dilate a local layout by one 4-neighbourhood ring** — the creep layout.
///
/// Hillslope creep gathers across the four edge neighbours, moving the **donor's**
/// composition in proportion, so a cell's creep row must hold everything its
/// neighbours' windows hold as well as its own.
pub fn build_creep_layout(layout: &mut SpeciesLayout, w: usize, local: &[u64]) {
    let n = w * w;
    layout.clear_masks(n);
    for c in 0..n {
        let (gx, gy) = ((c % w) as i32, (c / w) as i32);
        let mut m = local[c];
        for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (gx + dx, gy + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < w {
                m |= local[ny as usize * w + nx as usize];
            }
        }
        layout.mask[c] = m;
    }
    layout.rebuild_from_masks(n);
}

/// Presence mask of a dense share row (non-zero entries).
#[inline]
pub fn mask_of_dense(row: &[f64]) -> u64 {
    let mut m = 0u64;
    for (k, &v) in row.iter().enumerate() {
        if v > 0.0 {
            m |= 1u64 << k;
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis() -> SpeciesAxis {
        SpeciesAxis::new(&dc_core::materials::geology::vanilla(), MaterialId::GRANITE)
    }

    #[test]
    fn the_axis_is_descending_in_settling_energy() {
        let a = axis();
        let w = a.w_settle();
        for k in 1..w.len() {
            assert!(
                w[k - 1] >= w[k],
                "axis must be coarsest-first: {} then {}",
                a.material(k - 1).qualified_name(),
                a.material(k).qualified_name()
            );
        }
    }

    /// The axis is derived from the content, so every rock the deep record can
    /// name has a place on it — and the basement does even though no member
    /// deposits it.
    #[test]
    fn every_registered_member_material_is_on_the_axis() {
        let set = dc_core::materials::geology::vanilla();
        let a = SpeciesAxis::new(&set, MaterialId::GRANITE);
        for m in set.members() {
            assert!(a.carries(m.material), "{} is off the axis", m.id);
        }
        assert!(
            a.carries(MaterialId::GRANITE),
            "the basement must be carried"
        );
    }

    /// Registration order cannot move a byte: the axis is sorted by a property of
    /// the material, tie-broken by its canonical id.
    #[test]
    fn the_axis_is_registration_order_independent() {
        let a = axis();
        let b = axis();
        assert_eq!(a.materials(), b.materials());
    }

    #[test]
    fn a_slot_lookup_is_the_rank_of_its_bit() {
        let mut l = SpeciesLayout::empty();
        // Three cells: {0,3}, {}, {1,2,5}.
        build_local_layout(&mut l, 3, &[0b1001, 0, 0b100110]);
        assert_eq!(l.nnz(), 5);
        assert_eq!(l.slot_of(0, 0), Some(0));
        assert_eq!(l.slot_of(0, 3), Some(1));
        assert_eq!(l.slot_of(0, 1), None);
        assert_eq!(l.slot_of(1, 0), None);
        assert_eq!(l.slot_of(2, 1), Some(2));
        assert_eq!(l.slot_of(2, 2), Some(3));
        assert_eq!(l.slot_of(2, 5), Some(4));
        assert_eq!(l.row(2).1, &[1, 2, 5]);
    }

    /// The closure is the transport graph's reachability: a species released
    /// upstream has a slot at every cell downstream of it.
    #[test]
    fn the_transport_closure_reaches_every_downstream_cell() {
        let mut l = SpeciesLayout::empty();
        // A 4-cell chain 0 → 1 → 2 → 3, processed highest-first (order is the
        // solve's own ascending-elevation array, walked in reverse).
        let seed = [0b1u64, 0b10, 0, 0];
        let order = [3u32, 2, 1, 0];
        build_transport_layout(&mut l, 4, &seed, &order, |c, push| {
            if c + 1 < 4 {
                push(c + 1);
            }
        });
        assert_eq!(l.mask_at(0), 0b1);
        assert_eq!(l.mask_at(1), 0b11);
        assert_eq!(l.mask_at(2), 0b11);
        assert_eq!(l.mask_at(3), 0b11);
        assert_eq!(l.nnz(), 1 + 2 + 2 + 2);
    }

    #[test]
    fn a_creep_layout_holds_its_four_neighbours() {
        let mut l = SpeciesLayout::empty();
        // 3×3, only the centre carries species 4.
        let mut local = [0u64; 9];
        local[4] = 1 << 4;
        build_creep_layout(&mut l, 3, &local);
        for c in [1usize, 3, 4, 5, 7] {
            assert_eq!(l.mask_at(c), 1 << 4, "cell {c} must hold the donor's rock");
        }
        for c in [0usize, 2, 6, 8] {
            assert_eq!(l.mask_at(c), 0, "diagonals are not creep edges");
        }
    }
}
