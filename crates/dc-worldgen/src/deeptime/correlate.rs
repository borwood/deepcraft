//! **Stratigraphic correlation — the kernel** (S1 of the record-read redesign,
//! `docs/audits/2026-08-03-stratigraphic-correlation-design.md`).
//!
//! Deep cells are **boreholes ~460 m apart**; a voxel column between them is
//! the geologist's cross-section. This module answers the one question that
//! makes such a section drawable: *given the ≤ 4 (≤ 9 straddling) parent cells
//! a chunk's bilinear stencil touches, which bed of A corresponds to which bed
//! of B* — and, once that correspondence exists, what thickness each correlated
//! interval realizes at a column standing at bilinear weights `w`.
//!
//! The user's anchor sketch (2026-08-03): *"layers created at the same time in
//! two different deep cells should… blend? find a midpoint and smoothly grade
//! their thickness."* This is that, executed.
//!
//! ## The correspondence rule: an epoch-keyed join, not an alignment
//!
//! The design pass shipped with R-C, a *proportional* rule — partition by
//! tectonic chapter, correlate within a chapter by cumulative thickness
//! fraction — because the record carried no finer clock. **Then the deposition
//! clock landed** (journal/0154, O-2b): every [`DepUnit`](crate::deeptime::recorder::DepUnit)
//! carries the raw runner tick it was first deposited on, and the collapse-tier
//! [`StrataEvent`] carries that tick through as `epoch_bottom`. So correlation
//! no longer has to *guess* correspondence inside a chapter — it reads it.
//!
//! The rule is therefore about as simple as a rule gets:
//!
//! > **The shared partition is the sorted union of the epochs the parents
//! > stamp. Interval `k` IS epoch `epochs[k]`. A parent contributes to interval
//! > `k` exactly the beds it stamped at that epoch — and zero if it stamped
//! > none.**
//!
//! Three properties fall straight out of that, and each replaces a piece of
//! machinery the design had to argue for:
//!
//! 1. **No interpolation is needed to build the partition.** Every bed's epoch
//!    *is* a boundary of the union, so no bed is ever split across two
//!    intervals. R-C survives only as the *within*-interval interpolant (the
//!    sub-interval ordering of several beds a parent stamped on one tick) —
//!    which is S2's identity problem, not this module's.
//! 2. **Pinch-out is not a special case.** A parent that stamped nothing at
//!    epoch `e` contributes `0`, so a bed present in A and absent in B tapers
//!    linearly to zero across the span. **B-1 (P-4, ruled 2026-08-04: *"an
//!    empty stack is a parent whose every interval thickness is 0"*) is the
//!    same statement with every epoch empty** — the onlap feather against
//!    basement falls out of ordinary blending and gets no branch anywhere in
//!    this file.
//! 3. **The partition is seam-free.** Because an interval is *keyed by epoch
//!    value*, a parent with weight 0 can only ever add intervals that are
//!    empty for every other parent too. Two adjacent chunks whose stencils
//!    share the parents that matter at their common boundary therefore produce
//!    the *same* non-empty interval sequence there, even though their unions
//!    differ. This is the R-B rejection (multi-way sequence alignment is not
//!    pairwise-composable, so it seams between chunks) made structurally moot.
//!
//! **R-C′, the unconformity-anchor refinement, is NOT built and must not be.**
//! Its input does not exist: the shipped world carries **0** interior
//! unconformity-flagged contacts, structurally — `DeepStrata::stripped` is set
//! only when the record empties, so a flagged unit is always `units[0]` and its
//! predecessor was deleted by the very strip the flag records
//! (`docs/audits/2026-08-04-s0-correlation-measurements.md` § 5.2,
//! `journal/corrections.md` #99).
//!
//! ## Mass: an identity, never a tolerance
//!
//! With bilinear weights `w_j` (`Σ w_j = 1`) and per-interval blending
//! `h(p, k) = Σ_j w_j · t_jk`, the column total is
//!
//! ```text
//! Σ_k h(p,k) = Σ_k Σ_j w_j·t_jk = Σ_j w_j·Σ_k t_jk = Σ_j w_j·H_j
//! ```
//!
//! — blend-of-sums equals sum-of-blends, because interpolation and summation
//! are both linear (design F2). So the finalize invariant `Σ units == H` holds
//! at *every* column by arithmetic, with only f64 rounding between the two
//! sides; [`EpochPartition::mass_bound`] derives that rounding rather than
//! fitting it. This is what dissolves the constraint that forced
//! one-cell-per-column — a *choice* had to take record + `H` from one cell, a
//! *linear blend* does not.
//!
//! ## Shape: lazy per column, materialized once per chunk
//!
//! S0 measured the alternative and it disqualified itself
//! (`2026-08-04-s0-correlation-measurements.md` § 2.3, resolving I-5 to
//! **(ii) lazy per-column evaluation inside the voxel walk**): naively
//! rebuilding per-column fill state costs **+104.8 % of `column()`** — which is
//! **1.62 ms/chunk**, not the ≈14 ms the design's F4 hand-arithmetic assumed —
//! and ≈3 MB/chunk of transient in a runtime-resident cache.
//!
//! So this module materializes **one** [`EpochPartition`] per chunk (the
//! parents × intervals thickness table: mean 4 × 44.9 × 16 B ≈ **2.9 kB**, worst
//! case 9 × 200 × 16 B ≈ **28.8 kB**) and evaluates each of the 1024 columns
//! **lazily**, as an [`Iterator`] that carries a running cumulative depth and
//! allocates nothing ([`EpochPartition::column`]). The voxel walk already
//! visits spans in depth order; this hands it spans in depth order.
//!
//! Sizing facts this is built against (all measured, S0): 2×2 epoch-union
//! partition mean **44.9** intervals / p95 **145** / max **200**; 3×3 mean
//! **61.7** / p95 183; max stack depth **460** units; mean **24.6** units/cell;
//! **7,304,581** units world-wide.
//!
//! ## What this module deliberately does not do
//!
//! - **No wiring.** Nothing calls it yet; `column()` and `collapse.rs`'s call
//!   path are untouched. S2 owns the wiring, the `record_for` semantics change,
//!   and the `cell_of` / `NearRecordMembership` retirement.
//! - **No identity blending.** P-1 ruled M-C — mixture grading where two
//!   materials' regions adjoin on a declared axis, the octaves cut where they
//!   are discrete, with the predicate **derived, never hand-paired**
//!   (`materials.md` § DECIDED 2026-08-04). This module carries each bed's
//!   member through the partition so S2 can ask the question; it answers none
//!   of it. See [`EpochPartition::parent_run`].
//! - **No discontinuity seam.** Design § 1.3: correlation never fails today
//!   (no faults are modelled), and the `discontinuity(a, b) -> Option<_>`
//!   provider with identity default `None` ships when the fault/deformation
//!   pass that would fill it exists. Building an empty seam now would be
//!   machinery nothing calls (spines § 3).

use crate::geology::{StrataEvent, StrataRec};
use crate::subcell::SubCell;
use dc_core::materials::geology::GeoMemberIdx;

/// One correlatable bed, as the kernel sees a borehole's stack.
///
/// This is the collapse-tier [`StrataEvent`] reduced to the three facts
/// correlation needs: **when** it was laid (the deposition clock's raw tick),
/// **how thick** it is, and **what it is** (carried, never blended here — S2's
/// M-C question).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bed {
    /// The raw runner tick this bed was first deposited on
    /// (`StrataEvent::epoch_bottom` / `DepUnit::epoch`). **A raw tick, never a
    /// derived time** — what an epoch means in years is a world/pack call.
    pub epoch: u8,
    /// Recorded thickness, metres. Non-negative.
    pub thickness_m: f64,
    /// The bed's material identity, carried through the partition untouched so
    /// S2's M-C branch can read which parents present which member in a
    /// correlated interval.
    pub member: GeoMemberIdx,
}

/// True when `beds` is ordered bottom-up by non-decreasing epoch — the
/// recorder's invariant (a merged run keeps its bottom epoch, no writer inserts
/// below the top, no writer reorders), **confirmed at world scale**: 0
/// non-monotone units across all 7,304,581
/// (`2026-08-04-s0-correlation-measurements.md` § 5.1, measured in release
/// where `debug_assert!` does not run).
///
/// Exposed as a predicate rather than only a `debug_assert!` so a test can
/// state it in any profile — the gate runs `--release`, where a `should_panic`
/// on a debug assertion proves nothing.
pub fn epochs_non_decreasing(beds: &[Bed]) -> bool {
    beds.windows(2).all(|w| w[0].epoch <= w[1].epoch)
}

/// One parent cell's correlatable stack, plus the thickness this module
/// deliberately did **not** correlate.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Borehole {
    /// Deep-history beds, bottom-up, non-decreasing in epoch.
    pub beds: Vec<Bed>,
    /// **Metres of un-clocked record** — the year-zero veneer passes (clastic
    /// veneer, igneous bodies, placer, weathering front: `StrataEvent::dither
    /// == true`), whose beds are laid *after* the deep run and carry
    /// `epoch_bottom == 0` by charter rather than by measurement.
    ///
    /// Folding them into interval 0 would correlate present-day veneer with the
    /// world's oldest deep beds — a real defect, and the reason they are split
    /// out here rather than filtered away silently. They are the same passes,
    /// in the same order, in every parent (design § 1.4), so they correlate
    /// trivially; **where they sit in the blended column is S2's**, and this
    /// number exists so S2 cannot lose them.
    pub unclocked_m: f64,
}

impl Borehole {
    /// Split one parent's collapse-tier record into correlatable beds and
    /// un-clocked veneer metres.
    pub fn from_strata(rec: &StrataRec) -> Self {
        let mut beds = Vec::with_capacity(rec.events.len());
        let mut unclocked_m = 0.0f64;
        for ev in &rec.events {
            if ev.dither {
                unclocked_m += f64::from(ev.thickness_m);
            } else {
                beds.push(Bed {
                    epoch: ev.epoch_bottom,
                    thickness_m: f64::from(ev.thickness_m),
                    member: ev.member,
                });
            }
        }
        debug_assert!(
            epochs_non_decreasing(&beds),
            "the deposition clock is non-decreasing up-stack (recorder invariant, \
             journal/0154; 0 violations over 7,304,581 units at S0) — correlation \
             bins by epoch and a reordered stack would bin into the wrong interval"
        );
        Borehole { beds, unclocked_m }
    }

    /// Total correlatable recorded thickness, metres — the `H_j` of the mass
    /// identity. Excludes [`Self::unclocked_m`].
    pub fn total_m(&self) -> f64 {
        self.beds.iter().map(|b| b.thickness_m).sum()
    }
}

/// The `&[SubCell]` entry point: the boreholes a chunk's stencil touched.
///
/// [`ColumnRec::records`](crate::collapse::ColumnRec) is exactly this set —
/// one `SubCell` per touched deep cell, each built from its own cell's
/// (record, `H`, ledger) bundle. **Parent order is the caller's**: the weight
/// vector handed to [`EpochPartition::column`] addresses `boreholes[j]`
/// positionally, and it is the caller that knows which stencil corner each
/// `SubCell` came from.
pub fn boreholes_of(records: &[SubCell]) -> Vec<Borehole> {
    records
        .iter()
        .map(|sub| Borehole::from_strata(sub.strata()))
        .collect()
}

/// The bilinear stencil weights `[w00, w10, w01, w11]` at cell-fractional
/// position `(fx, fz)` — the same expression
/// `CoarseField::stencil` uses (`dc-core/src/coarse.rs:483-488`), in the same
/// corner order.
///
/// ⚠ **This is a four-line mirror of a private function, and S2 must not keep
/// it that way.** When the wiring lands, the weights belong to the field's own
/// stencil (one authority, per S-3) rather than being re-derived beside it;
/// this helper exists so the kernel is testable — invariants 2 and 3 are
/// statements *about* these weights — without S1 reaching into another crate's
/// API surface. Flagged rather than committed silently (A-1: re-inventing a
/// mechanism next to the one we already built).
pub fn bilinear_weights(fx: f64, fz: f64) -> [f64; 4] {
    [
        (1.0 - fx) * (1.0 - fz),
        fx * (1.0 - fz),
        (1.0 - fx) * fz,
        fx * fz,
    ]
}

/// The chunk's **shared bed partition**: one ordered list of correlated
/// intervals that every parent's stack maps onto, built once per chunk.
///
/// Interval `k` *is* epoch [`Self::epoch_of`]`(k)`; its age span is
/// `[epochs[k], epochs[k+1])`, the last being open-ended. The table is
/// parents × intervals of `(thickness, run)` — see the module docs for its
/// measured size.
#[derive(Debug, Clone, PartialEq)]
pub struct EpochPartition {
    /// Ascending, deduplicated epochs stamped by at least one parent.
    epochs: Vec<u8>,
    parents: usize,
    /// Row-major `[parent * intervals + k]`: that parent's summed thickness in
    /// interval `k`, metres. Zero where the parent stamped nothing.
    thickness: Vec<f64>,
    /// Row-major `[parent * intervals + k]`: `(start, len)` of the contiguous
    /// run of that parent's beds in interval `k`. Contiguous because epochs are
    /// non-decreasing up-stack.
    runs: Vec<(u32, u32)>,
    /// Per-parent `H_j` — the correlatable recorded total, metres.
    totals: Vec<f64>,
}

impl EpochPartition {
    /// Build the shared partition over the parents' stacks.
    ///
    /// `O(Σ beds + 256 + parents × intervals)`: the epoch union is a 256-bit
    /// presence bitset (epochs are `u8` and `DEEP_ITERATIONS = 200`, so 256
    /// bits covers the axis exactly), and each parent is binned by a single
    /// forward merge-walk against the sorted union. **No pairwise anything** —
    /// a 4-way and a 9-way stencil cost the same union, which is the property
    /// that made R-B's arity problem (align(A,B) and align(B,C) need not agree
    /// with align(A,C)) not arise.
    pub fn build<B: AsRef<[Bed]>>(boreholes: &[B]) -> Self {
        let mut present = [0u64; 4];
        for bh in boreholes {
            for bed in bh.as_ref() {
                present[usize::from(bed.epoch >> 6)] |= 1u64 << (bed.epoch & 63);
            }
        }
        let mut epochs: Vec<u8> = Vec::new();
        for (word, &bits) in present.iter().enumerate() {
            let mut bits = bits;
            while bits != 0 {
                let b = bits.trailing_zeros();
                epochs.push((word * 64 + b as usize) as u8);
                bits &= bits - 1;
            }
        }
        let intervals = epochs.len();
        let parents = boreholes.len();
        let mut thickness = vec![0.0f64; parents * intervals];
        let mut runs = vec![(0u32, 0u32); parents * intervals];
        let mut totals = vec![0.0f64; parents];

        for (j, bh) in boreholes.iter().enumerate() {
            let beds = bh.as_ref();
            debug_assert!(
                epochs_non_decreasing(beds),
                "correlation bins by epoch and relies on the recorder's \
                 non-decreasing-up-stack invariant (journal/0154, S0 § 5.1)"
            );
            let row = j * intervals;
            let mut k = 0usize;
            let mut total = 0.0f64;
            for (i, bed) in beds.iter().enumerate() {
                // Forward merge-walk: both sides are sorted ascending, so the
                // cursor only advances. The fallback below keeps the binning
                // CORRECT (not merely fast) in release, where the debug assert
                // above is compiled out — a mis-ordered stack would otherwise
                // silently bin into the wrong interval.
                while k < intervals && epochs[k] < bed.epoch {
                    k += 1;
                }
                let slot = if k < intervals && epochs[k] == bed.epoch {
                    k
                } else {
                    epochs
                        .binary_search(&bed.epoch)
                        .expect("every stamped epoch is in the union by construction")
                };
                thickness[row + slot] += bed.thickness_m;
                let run = &mut runs[row + slot];
                if run.1 == 0 {
                    *run = (i as u32, 1);
                } else {
                    debug_assert_eq!(
                        run.0 as usize + run.1 as usize,
                        i,
                        "a parent's beds at one epoch are contiguous because the \
                         stack is sorted by epoch — the run is a range, not a set"
                    );
                    run.1 += 1;
                }
                total += bed.thickness_m;
            }
            totals[j] = total;
        }

        EpochPartition {
            epochs,
            parents,
            thickness,
            runs,
            totals,
        }
    }

    /// Number of correlated intervals — the per-column dot-product length.
    /// Measured over the shipped world: 2×2 stencil mean **44.9**, p95 145,
    /// max 200 (S0 § 3.1).
    pub fn intervals(&self) -> usize {
        self.epochs.len()
    }

    /// Number of parent boreholes (≤ 4 in a cell interior, ≤ 9 straddling).
    pub fn parents(&self) -> usize {
        self.parents
    }

    /// The epoch interval `k` is keyed by. Its age span is
    /// `[epoch_of(k), epoch_of(k+1))`.
    pub fn epoch_of(&self, k: usize) -> u8 {
        self.epochs[k]
    }

    /// Parent `j`'s thickness in interval `k`, metres — `0.0` where it stamped
    /// nothing at that epoch. **The pinch-out and B-1's onlap feather are both
    /// this zero**, with no branch anywhere.
    pub fn parent_thickness(&self, j: usize, k: usize) -> f64 {
        self.thickness[j * self.intervals() + k]
    }

    /// The contiguous run `(start, len)` of parent `j`'s beds in interval `k`,
    /// as indices into that parent's own bed slice — `len == 0` where the
    /// parent stamped nothing.
    ///
    /// **This is the hook S2's identity branch (P-1's M-C) reads**: the members
    /// several parents present in one correlated interval are exactly the
    /// members of these runs. Nothing here adjudicates between them.
    pub fn parent_run(&self, j: usize, k: usize) -> (usize, usize) {
        let (s, l) = self.runs[j * self.intervals() + k];
        (s as usize, l as usize)
    }

    /// Parent `j`'s correlatable recorded total `H_j`, metres.
    pub fn parent_total(&self, j: usize) -> f64 {
        self.totals[j]
    }

    /// The column's total recorded thickness `H(p) = Σ_j w_j · H_j` — the
    /// **right-hand side** of the mass identity, and the quantity that replaces
    /// `regolith_at_voxel`'s NEAREST read when S2 wires this up.
    ///
    /// The S-4 live violation (`H` is not interpolable because the record it
    /// must agree with is not) is discharged by construction rather than routed
    /// around: the record became interpolable **bed-wise**, so `H` may finally
    /// go bilinear *in lockstep* with it.
    pub fn blended_total(&self, weights: &[f64]) -> f64 {
        debug_assert_eq!(weights.len(), self.parents);
        weights
            .iter()
            .zip(self.totals.iter())
            .map(|(w, h)| w * h)
            .sum()
    }

    /// **The lazy per-column evaluation** — I-5's ruled mitigation (ii).
    ///
    /// Yields every interval in depth order with its blended thickness and its
    /// running base depth above the record base. Allocates nothing and
    /// materializes no per-column vector: the naive alternative costs
    /// **+104.8 % of `column()`** and ≈3 MB/chunk of transient (S0 § 2.3).
    ///
    /// Zero-thickness intervals are yielded rather than skipped — they are
    /// inert to a voxel walk, and *which* intervals are zero is exactly the
    /// pinch-out signal a consumer may want. Filter at the call site.
    pub fn column<'a>(&'a self, weights: &'a [f64]) -> ColumnIntervals<'a> {
        debug_assert_eq!(weights.len(), self.parents);
        ColumnIntervals {
            part: self,
            weights,
            k: 0,
            base_m: 0.0,
        }
    }

    /// The **derived** f64 bound on `|Σ_k h(p,k) − Σ_j w_j·H_j|`.
    ///
    /// Derivation, not a fit. Both sides are sums of non-negative terms, so
    /// there is no cancellation and the classical `γ_n = n·u/(1−n·u)` bound
    /// applies with `u = 2⁻⁵³`. The longest accumulation chain on the left is
    /// `parents` multiply-adds per interval plus `intervals` outer additions;
    /// on the right it is `parents` multiply-adds. Taking the chain length
    /// `k = intervals + 2·parents` and noting `f64::EPSILON = 2⁻⁵² = 2u ≥
    /// γ_1`, the product `k · EPSILON · total` is ≥ `γ_k · total` for every
    /// `k` this partition can produce (`k ≤ 200 + 18`, so `k·u < 2.5e-14 ≪ 1`).
    /// The 2× slack from `EPSILON = 2u` is stated rather than tuned.
    ///
    /// **ULP count × interval count, derived not fitted** — design § 2.3
    /// invariant 1.
    pub fn mass_bound(&self, weights: &[f64]) -> f64 {
        let total = self.blended_total(weights);
        let chain = (self.intervals() + 2 * self.parents) as f64;
        chain * f64::EPSILON * total.abs()
    }

    /// The **derived** Lipschitz bound on `|H(p) − H(p′)|` for a step of
    /// `(du, dv)` in cell-fractional coordinates.
    ///
    /// `H(u,v) = Σ_j w_j(u,v)·H_j` is bilinear, so
    /// `∂H/∂u = (1−v)(H₁₀−H₀₀) + v(H₁₁−H₀₁)`, whose magnitude is at most
    /// `H_max − H_min` (a convex combination of differences); the same holds
    /// for `∂H/∂v`. The mean value theorem along the segment gives
    /// `|ΔH| ≤ (|du| + |dv|)·(H_max − H_min)`.
    ///
    /// For one voxel of travel `du = voxel_m / DEEP_CELL_M = 0.9/460 ≈
    /// 1.96e-3`, so a 539 m record (the world's deepest, S0 § 1.1) can move at
    /// most ≈1.06 m per voxel step — the continuity class the razor cliff at
    /// the 0149 wall did not have.
    ///
    /// ⚠ **This is a statement about `H`, not about f64.** For a step along one
    /// axis `H` is *exactly linear*, so `|ΔH|` **attains** this bound — the
    /// derivative is constant along the segment. A consumer comparing two
    /// *computed* totals must therefore add the evaluation's own rounding,
    /// [`Self::total_rounding_bound`], twice. Conflating the two into one
    /// number here would hide a mathematical claim inside a numerical fudge.
    ///
    /// *Scale-free: a property of the bilinear weights, not of the world.*
    pub fn lipschitz_bound(&self, du: f64, dv: f64) -> f64 {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &h in &self.totals {
            lo = lo.min(h);
            hi = hi.max(h);
        }
        if !lo.is_finite() || !hi.is_finite() {
            return 0.0;
        }
        (du.abs() + dv.abs()) * (hi - lo)
    }

    /// The **derived** absolute f64 error of one [`Self::blended_total`]
    /// evaluation, independent of the weights: the dot product's longest
    /// accumulation chain is `2·parents` operations, and every term is bounded
    /// in magnitude by `max_j H_j` because the weights are a partition of
    /// unity. Same derivation as [`Self::mass_bound`], with the magnitude
    /// bounded a priori instead of computed — which is what lets it be added
    /// to a bound on the *exact* quantity, such as
    /// [`Self::lipschitz_bound`]'s.
    pub fn total_rounding_bound(&self) -> f64 {
        let hmax = self.totals.iter().fold(0.0f64, |a, h| a.max(h.abs()));
        (2 * self.parents) as f64 * f64::EPSILON * hmax
    }

    /// Rough heap footprint (bytes) of the chunk-level table — the
    /// runtime-residency line the design's RETURN spec asks for.
    pub fn heap_bytes(&self) -> usize {
        self.epochs.capacity()
            + self.thickness.capacity() * std::mem::size_of::<f64>()
            + self.runs.capacity() * std::mem::size_of::<(u32, u32)>()
            + self.totals.capacity() * std::mem::size_of::<f64>()
    }
}

/// One correlated interval as a column realizes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnInterval {
    /// Index into the partition.
    pub index: usize,
    /// The epoch this interval is keyed by.
    pub epoch: u8,
    /// The blended thickness `Σ_j w_j · t_jk`, metres. `0.0` where no
    /// positively-weighted parent stamped this epoch.
    pub thickness_m: f64,
    /// Metres above the record base at which this interval starts — the
    /// running cumulative the voxel walk needs, carried by the iterator so no
    /// prefix-sum vector is ever built.
    pub base_m: f64,
}

impl ColumnInterval {
    /// Metres above the record base at which this interval ends.
    pub fn top_m(&self) -> f64 {
        self.base_m + self.thickness_m
    }
}

/// The lazy per-column interval stream — see [`EpochPartition::column`].
#[derive(Debug, Clone)]
pub struct ColumnIntervals<'a> {
    part: &'a EpochPartition,
    weights: &'a [f64],
    k: usize,
    base_m: f64,
}

impl Iterator for ColumnIntervals<'_> {
    type Item = ColumnInterval;

    fn next(&mut self) -> Option<ColumnInterval> {
        let intervals = self.part.intervals();
        if self.k >= intervals {
            return None;
        }
        let k = self.k;
        let mut t = 0.0f64;
        for (j, &w) in self.weights.iter().enumerate() {
            t += w * self.part.thickness[j * intervals + k];
        }
        let out = ColumnInterval {
            index: k,
            epoch: self.part.epochs[k],
            thickness_m: t,
            base_m: self.base_m,
        };
        self.base_m += t;
        self.k += 1;
        Some(out)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.part.intervals() - self.k;
        (n, Some(n))
    }
}

impl ExactSizeIterator for ColumnIntervals<'_> {}

/// Convenience over [`StrataRec`]: the un-clocked (year-zero veneer) events,
/// for a consumer that wants them without splitting the whole record.
pub fn unclocked_events(rec: &StrataRec) -> impl Iterator<Item = &StrataEvent> {
    rec.events.iter().filter(|e| e.dither)
}
