//! **The stratigraphic-correlation kernel's invariants** (S1 of the record-read
//! redesign; `docs/audits/2026-08-03-stratigraphic-correlation-design.md`
//! § 2.3, which drafted these five as the gate's spec).
//!
//! The kernel is consumerless for now — nothing calls it, `column()` is
//! untouched, no golden moves — so **these tests are the slice's entire proof**.
//! Every one asserts an *invariant*, never a snapshot: an arithmetic identity, a
//! derived bound, a monotonicity, a structural equality. A MiB figure or a
//! thickness constant would fail the day a colleague improved the sim, which is
//! worse than the defect it guards (CLAUDE.md § Gates).
//!
//! **Each test says why it is scale-free in its own doc comment**, and none of
//! them builds a world: correlation is per-column arithmetic over a parent
//! stencil, so the smallest input that exercises the predicate exercises it at
//! every world size. Added gate wall-clock is therefore ~0 (reported in the
//! slice's RETURN spec).

use dc_core::materials::geology::GeoMemberIdx;
use dc_worldgen::deeptime::correlate::{
    Bed, Borehole, EpochPartition, bilinear_weights, epochs_non_decreasing,
};
use dc_worldgen::geology::{StrataEvent, StrataRec};

const M0: GeoMemberIdx = GeoMemberIdx(0);
const M1: GeoMemberIdx = GeoMemberIdx(1);

fn bed(epoch: u8, thickness_m: f64, member: GeoMemberIdx) -> Bed {
    Bed {
        epoch,
        thickness_m,
        member,
    }
}

/// Four parents with deliberately *disagreeing* epoch sets, unequal stack
/// depths, and thicknesses spanning five orders of magnitude — the shape the
/// shipped world produces (median stack 14, p95 104, max 460; H from 1 mm to
/// 539 m, S0 § 1.1).
fn four_disagreeing_parents() -> Vec<Vec<Bed>> {
    vec![
        // A deep basin: many epochs, one of them (9) uniquely its own.
        vec![
            bed(0, 12.5, M0),
            bed(3, 0.125, M1),
            bed(9, 4.0, M0),
            bed(17, 63.0, M1),
            bed(17, 0.001, M0),
            bed(40, 200.5, M1),
        ],
        // A condensed section: same clock, far fewer stamps.
        vec![bed(0, 0.5, M1), bed(17, 1.25, M0)],
        // An upland with a late start.
        vec![bed(17, 0.0625, M0), bed(40, 8.0, M1), bed(199, 0.75, M0)],
        // A cell that only ever recorded one tick.
        vec![bed(3, 33.0, M1)],
    ]
}

fn totals(parents: &[Vec<Bed>]) -> Vec<f64> {
    parents
        .iter()
        .map(|p| p.iter().map(|b| b.thickness_m).sum())
        .collect()
}

// ───────────────────────── the correspondence rule ─────────────────────────

/// The shared partition is the **sorted union of the epochs the parents stamp**,
/// and nothing else — the whole correspondence rule in one assert. Under the
/// deposition clock a bed's epoch *is* a boundary of the union, so no bed is
/// ever split and no bed lands in a stranger's interval (the R-A failure mode:
/// one deposit-erode cycle shifting every correlation above it by one).
///
/// *Scale-free: a set identity over the epoch axis, independent of stack depth,
/// thickness, world extent or parent count.*
#[test]
fn the_partition_is_the_union_of_the_parents_stamped_epochs() {
    let parents = four_disagreeing_parents();
    let part = EpochPartition::build(&parents[..]);

    let mut expect: Vec<u8> = parents
        .iter()
        .flat_map(|p| p.iter().map(|b| b.epoch))
        .collect();
    expect.sort_unstable();
    expect.dedup();

    assert_eq!(part.intervals(), expect.len());
    for (k, &e) in expect.iter().enumerate() {
        assert_eq!(part.epoch_of(k), e, "interval {k} is keyed by its epoch");
    }
    // And every parent's beds are binned under their own stamp, with the
    // per-(parent, interval) run contiguous and summing to the stored thickness.
    for (j, beds) in parents.iter().enumerate() {
        for k in 0..part.intervals() {
            let (start, len) = part.parent_run(j, k);
            let run = &beds[start..start + len];
            assert!(
                run.iter().all(|b| b.epoch == part.epoch_of(k)),
                "parent {j}'s run in interval {k} holds only that epoch's beds"
            );
            let sum: f64 = run.iter().map(|b| b.thickness_m).sum();
            assert_eq!(sum, part.parent_thickness(j, k));
        }
    }
}

/// **The debug assert the design owed** (§ 10.4: *"a cheap debug assert in S1
/// (chapter non-decreasing up-stack) converts the argument into a fact"* —
/// under the clock it is the epoch that must be monotone). Stated as a
/// predicate rather than only a `debug_assert!` because the gate runs
/// `--release`, where a debug assertion does not execute at all.
///
/// *Scale-free: an ordering property of a single stack.*
#[test]
fn the_deposition_clock_is_non_decreasing_up_stack_where_correlation_consumes_it() {
    for beds in four_disagreeing_parents() {
        assert!(epochs_non_decreasing(&beds));
    }
    assert!(epochs_non_decreasing(&[]));
    assert!(!epochs_non_decreasing(&[bed(7, 1.0, M0), bed(3, 1.0, M0)]));
}

// ─────────────────────────── § 2.3 invariant 1 ───────────────────────────

/// **Per column, `Σ_k h(p,k) == Σ_j w_j·H_j`** — the mass identity that
/// dissolves the one-cell-per-column constraint (design F2). Blend-of-sums
/// equals sum-of-blends exactly in exact arithmetic; in f64 the two sides
/// differ only by rounding, and the bound is *derived*
/// (`EpochPartition::mass_bound`: chain length × `f64::EPSILON` × magnitude),
/// never chosen until green.
///
/// Swept over a 13×13 grid of stencil positions including all four corners and
/// both edges, so the clamped and interior weight regimes are both covered.
///
/// *Scale-free: per-column arithmetic. The identity is linearity of summation;
/// the bound scales with the interval count and the magnitude, both of which
/// appear in it — so it holds at any stack depth and any world size.*
#[test]
fn invariant_1_column_interval_sum_equals_the_blended_parent_total() {
    let parents = four_disagreeing_parents();
    let part = EpochPartition::build(&parents[..]);
    let h = totals(&parents);

    for i in 0..=12 {
        for j in 0..=12 {
            let (fx, fz) = (f64::from(i) / 12.0, f64::from(j) / 12.0);
            let w = bilinear_weights(fx, fz);
            assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-15);

            let lhs: f64 = part.column(&w).map(|iv| iv.thickness_m).sum();
            let rhs = part.blended_total(&w);
            let expect: f64 = w.iter().zip(h.iter()).map(|(a, b)| a * b).sum();
            let bound = part.mass_bound(&w);

            assert!(
                (lhs - rhs).abs() <= bound,
                "at ({fx}, {fz}): Σ intervals {lhs} vs Σ w·H {rhs}, \
                 |Δ| = {} > derived bound {bound}",
                (lhs - rhs).abs()
            );
            assert!((rhs - expect).abs() <= bound);
            // The bound is not vacuous: it is parts-per-quadrillion of the
            // total, not a licence.
            assert!(bound <= 1e-12 * rhs.max(1.0));
        }
    }
}

/// The iterator's running base is a genuine prefix sum: interval `k` starts
/// where interval `k−1` ended, and the last interval's top is the column total.
/// This is what lets the voxel walk consume spans in depth order **without any
/// per-column vector** — I-5's ruled mitigation (ii), against a naive
/// materialization measured at +104.8 % of `column()` (S0 § 2.3).
///
/// *Scale-free: a telescoping property of the accumulation.*
#[test]
fn the_lazy_column_stream_is_a_prefix_sum_over_the_partition() {
    let parents = four_disagreeing_parents();
    let part = EpochPartition::build(&parents[..]);
    let w = bilinear_weights(0.31, 0.77);

    let mut prev_top = 0.0f64;
    let mut count = 0usize;
    let mut last = None;
    for iv in part.column(&w) {
        assert_eq!(
            iv.base_m,
            prev_top,
            "interval {} starts where {} ended",
            iv.index,
            iv.index.saturating_sub(1)
        );
        assert!(iv.thickness_m >= 0.0);
        prev_top = iv.top_m();
        count += 1;
        last = Some(iv);
    }
    assert_eq!(count, part.intervals());
    assert_eq!(part.column(&w).len(), part.intervals());
    let total: f64 = part.column(&w).map(|iv| iv.thickness_m).sum();
    assert_eq!(last.expect("non-empty partition").top_m(), total);
}

// ─────────────────────────── § 2.3 invariant 2 ───────────────────────────

/// **Continuity.** Adjacent columns differ in blended total by at most
/// `(du + dv)·(H_max − H_min)` — the derived Lipschitz constant of the bilinear
/// weights (`EpochPartition::lipschitz_bound`, derivation in its doc). One
/// voxel of travel is `du = 0.9/460 ≈ 1.96e-3` of a cell, so even the world's
/// deepest record (539 m, S0 § 1.1) can move ≈1.06 m per voxel step. **This is
/// the property the 0149 wall did not have**: the razor-vertical record edge
/// was a step of the full column height across one voxel.
///
/// The parents here are deliberately maximally discordant (a 280 m basin beside
/// a 33 m cell beside an 8.8 m upland beside a 1.75 m condensed section) so the
/// bound is being tested where it is tightest — and along one axis `H` is
/// *exactly linear*, so the bound is **attained**, not merely respected. That is
/// why the tolerance adds `2 × total_rounding_bound()`: comparing two *computed*
/// totals against a bound on the *exact* difference needs the evaluation's own
/// f64 error at both ends, and it is derived (`2·parents` ops × `EPSILON` ×
/// `max H_j`), not slack chosen until green. **A naive 8-ULP tolerance failed
/// this test 30 times** — a real defect the derivation caught.
///
/// The step compared is the *realized* one (`fx1 - fx0`), never the nominal
/// `du`: `fx0 + du` rounds, and a bound stated for a step you did not take is
/// not a bound.
///
/// *Scale-free: a Lipschitz property of the bilinear weight functions. It
/// depends on the weight geometry and the parents' spread, both of which are in
/// the bound — not on world size, stack depth, or the number of intervals.*
#[test]
fn invariant_2_adjacent_columns_obey_the_derived_lipschitz_bound() {
    let parents = four_disagreeing_parents();
    let part = EpochPartition::build(&parents[..]);

    // One voxel of travel across a 460 m deep cell at the 0.9 m voxel pitch.
    let voxel_step = 0.9 / 460.0;
    let noise = 2.0 * part.total_rounding_bound();
    assert!(part.lipschitz_bound(voxel_step, 0.0) > 0.0);
    assert!(part.lipschitz_bound(voxel_step, voxel_step) >= part.lipschitz_bound(voxel_step, 0.0));

    let steps = 200;
    for j in 0..=8 {
        let fz = f64::from(j) / 8.0;
        for i in 0..steps {
            let fx0 = f64::from(i) / f64::from(steps);
            let fx1 = fx0 + voxel_step;
            if fx1 > 1.0 {
                break;
            }
            let du = fx1 - fx0;
            let bound = part.lipschitz_bound(du, 0.0) + noise;
            let a = part.blended_total(&bilinear_weights(fx0, fz));
            let b = part.blended_total(&bilinear_weights(fx1, fz));
            assert!(
                (a - b).abs() <= bound,
                "one-voxel step at ({fx0}, {fz}) moved H by {} > {bound}",
                (a - b).abs()
            );
            // And the diagonal step obeys the (du + dv) form.
            let fz1 = (fz + voxel_step).min(1.0);
            let bound_diag = part.lipschitz_bound(du, fz1 - fz) + noise;
            let c = part.blended_total(&bilinear_weights(fx1, fz1));
            assert!((a - c).abs() <= bound_diag);
        }
    }
}

// ─────────────────────────── § 2.3 invariant 3 ───────────────────────────

/// **Pinch-out monotonicity.** A bed present only in parent A realizes
/// thickness monotone in `w_A` along a transect — it tapers linearly to zero
/// rather than ending at a wall. Epoch 9 is stamped by parent 0 alone, so its
/// correlated interval is exactly `w₀ · 4.0 m`, and walking away from parent
/// 0's corner walks it to nothing.
///
/// This is the truncation/pinch-out wedge, and it costs no special case: the
/// absent parents contribute `0.0` through the ordinary dot product.
///
/// *Scale-free: a per-bed predicate. The interval's thickness is a linear
/// function of one weight, at any world size and any stack depth.*
#[test]
fn invariant_3_a_bed_only_one_parent_holds_tapers_monotonically_to_zero() {
    let parents = four_disagreeing_parents();
    let part = EpochPartition::build(&parents[..]);
    let k = (0..part.intervals())
        .find(|&k| part.epoch_of(k) == 9)
        .expect("epoch 9 is in the union");
    assert_eq!(part.parent_thickness(0, k), 4.0);
    for j in 1..part.parents() {
        assert_eq!(
            part.parent_thickness(j, k),
            0.0,
            "only parent 0 holds epoch 9"
        );
    }

    let fz = 0.25;
    let steps = 64;
    let mut prev = f64::INFINITY;
    for i in 0..=steps {
        let fx = f64::from(i) / f64::from(steps);
        let w = bilinear_weights(fx, fz);
        let iv = part
            .column(&w)
            .nth(k)
            .expect("the interval is in every column's stream");
        assert_eq!(iv.epoch, 9);
        assert_eq!(
            iv.thickness_m,
            w[0] * 4.0,
            "the pinch-out is w_A · t_A, exactly"
        );
        assert!(iv.thickness_m <= prev, "thickness is monotone in w_A");
        prev = iv.thickness_m;
    }
    // At parent 0's zero-weight edge the bed is gone — a feather, not a wall.
    let gone = part
        .column(&bilinear_weights(1.0, fz))
        .nth(k)
        .expect("still an interval, now empty");
    assert_eq!(gone.thickness_m, 0.0);
}

// ─────────────────────────── § 2.3 invariant 4 ───────────────────────────

/// **The partition is SEAM-FREE.** Two adjacent chunks whose stencils share the
/// parents that matter at their common boundary produce *bit-identical*
/// interval structure there — same epochs, same thicknesses, same depths —
/// even though their unions differ (the left chunk's union carries epochs only
/// its off-boundary parents stamped, and the right chunk's carries different
/// ones).
///
/// It holds because an interval is **keyed by epoch value**, so a
/// zero-weighted parent can only ever contribute intervals that are empty for
/// every other parent too. This is precisely what a sequence-alignment rule
/// (R-B) could not give: `align(A,B)` and `align(B,C)` need not agree with
/// `align(A,C)`, so a chunk's partition would be order-dependent and the
/// artifact class this arc exists to kill would reappear at every chunk border.
///
/// *Scale-free: a structural equality over the correspondence rule. It has no
/// tolerance and no magnitude in it.*
#[test]
fn invariant_4_adjacent_chunks_sharing_parents_are_seam_free_at_their_boundary() {
    // Six cells in a 3-wide, 2-tall arrangement:  A B C  /  D E F.
    // The left chunk's stencil is [A, B, D, E]; the right chunk's is
    // [B, C, E, F]. Their shared boundary is the B/E edge.
    let a = vec![bed(1, 5.0, M0), bed(6, 2.0, M1)];
    let b = vec![bed(1, 3.0, M1), bed(4, 9.5, M0), bed(6, 0.25, M0)];
    let c = vec![bed(2, 40.0, M0), bed(4, 0.5, M1)];
    let d = vec![bed(3, 1.5, M0)];
    let e = vec![bed(1, 0.75, M0), bed(4, 12.0, M1), bed(88, 7.0, M0)];
    let f = vec![bed(88, 0.125, M1), bed(199, 3.0, M0)];

    let left = EpochPartition::build(&[a, b.clone(), d, e.clone()][..]);
    let right = EpochPartition::build(&[b, c, e, f][..]);
    // The unions genuinely differ — the test would be vacuous otherwise.
    assert_ne!(left.intervals(), right.intervals());

    for i in 0..=16 {
        let fz = f64::from(i) / 16.0;
        // The left chunk's last column: fx = 1 puts all weight on B and E
        // (corners 10 and 11). The right chunk's first column: fx = 0 puts the
        // same weights on B and E (corners 00 and 01).
        let wl = bilinear_weights(1.0, fz);
        let wr = bilinear_weights(0.0, fz);
        assert_eq!((wl[1], wl[3]), (wr[0], wr[2]));

        let l: Vec<_> = left
            .column(&wl)
            .filter(|iv| iv.thickness_m > 0.0)
            .map(|iv| (iv.epoch, iv.thickness_m, iv.base_m))
            .collect();
        let r: Vec<_> = right
            .column(&wr)
            .filter(|iv| iv.thickness_m > 0.0)
            .map(|iv| (iv.epoch, iv.thickness_m, iv.base_m))
            .collect();
        assert_eq!(
            l, r,
            "the shared boundary column at fz = {fz} disagrees across the chunk seam"
        );
        assert!(!l.is_empty());
    }
}

// ─────────────────────────── § 2.3 invariant 5 ───────────────────────────

/// **B-1, the zero-partner blend** (P-4, ruled 2026-08-04, user: *"an empty
/// stack is a parent whose every chap thickness is 0"*). Where a neighbouring
/// parent has no record at all, the record **feathers out against basement as
/// an onlap wedge** — and the test's real claim is that this is *not a special
/// case*: the empty parent goes through the same dot product as every other
/// one, and there is no branch for it anywhere in the kernel.
///
/// The exhibit that opened this arc is the 0149 wall, where recorded strata met
/// `has_contents: false` basement floor-to-rim, razor-vertical, because record
/// EXTENT was transplanted rather than blended. Here the same configuration
/// produces a ≤ 460 m linear wedge.
///
/// *Scale-free: `0 · w = 0` and the taper is linear in the weight, at any world
/// size, any stack depth, and any number of empty partners.*
#[test]
fn invariant_5_an_empty_parent_feathers_the_record_to_zero_with_no_special_case() {
    let recorded: Vec<Vec<Bed>> = vec![
        vec![bed(2, 6.0, M0), bed(11, 1.5, M1)],
        vec![bed(2, 4.0, M1), bed(30, 0.5, M0)],
        vec![bed(11, 9.0, M0)],
        Vec::new(), // the recordless neighbour — an empty stack
    ];
    let part = EpochPartition::build(&recorded[..]);
    let h = totals(&recorded);
    assert_eq!(h[3], 0.0);
    assert_eq!(part.parent_total(3), 0.0);
    for k in 0..part.intervals() {
        assert_eq!(part.parent_thickness(3, k), 0.0);
        assert_eq!(part.parent_run(3, k), (0, 0));
    }
    // The empty parent contributed no epochs of its own: the partition is the
    // union of the recorded three.
    assert_eq!(part.intervals(), 3);

    // At the recordless corner the column is empty — exactly zero, and every
    // interval individually zero. A feather, reaching zero.
    let corner = bilinear_weights(1.0, 1.0);
    assert_eq!(part.blended_total(&corner), 0.0);
    for iv in part.column(&corner) {
        assert_eq!(iv.thickness_m, 0.0);
        assert_eq!(iv.base_m, 0.0);
    }

    // And the taper toward it is monotone, linear, and mass-true throughout —
    // ordinary blending, no branch.
    let steps = 64;
    let mut prev = f64::NEG_INFINITY;
    for i in (0..=steps).rev() {
        let fx = f64::from(i) / f64::from(steps);
        let w = bilinear_weights(fx, fx);
        let total: f64 = part.column(&w).map(|iv| iv.thickness_m).sum();
        assert!(
            total >= prev - 2.0 * part.total_rounding_bound(),
            "the wedge is monotone"
        );
        prev = total;
        assert!((total - part.blended_total(&w)).abs() <= part.mass_bound(&w));
    }
    // Midway between three recorded parents and one empty one, the column is
    // the plain average of the three that have a record.
    let mid = bilinear_weights(0.5, 0.5);
    let expect = 0.25 * (h[0] + h[1] + h[2]);
    assert!((part.blended_total(&mid) - expect).abs() <= part.mass_bound(&mid));

    // The degenerate limit: EVERY parent empty. Still no branch, still no panic.
    let none = EpochPartition::build(&[Vec::<Bed>::new(), Vec::new(), Vec::new(), Vec::new()][..]);
    assert_eq!(none.intervals(), 0);
    assert_eq!(none.blended_total(&mid), 0.0);
    assert_eq!(none.column(&mid).count(), 0);
    assert_eq!(none.mass_bound(&mid), 0.0);
}

// ──────────────────────── the `&[SubCell]`-shaped input ────────────────────

fn event(member: GeoMemberIdx, thickness_m: f32, epoch: u8, dither: bool) -> StrataEvent {
    StrataEvent {
        member,
        thickness_m,
        temp_c: 12.0,
        precip: 900.0,
        depth_m: 0.0,
        sel_salt: 0,
        sel_tag: 0,
        ore: None,
        accessory: None,
        dither,
        epoch_bottom: epoch,
        epoch_top: epoch,
    }
}

/// A borehole is read off the collapse-tier record a `SubCell` holds
/// (`SubCell::strata()`), and the **year-zero veneer is split out, never
/// dropped**. The veneer passes carry `epoch_bottom == 0` by charter rather
/// than by measurement, so folding them into interval 0 would correlate
/// present-day veneer with the world's oldest deep beds. `unclocked_m` is the
/// number that keeps S2 from losing that mass.
///
/// *Scale-free: a per-event classification, one predicate per event.*
#[test]
fn a_borehole_splits_the_year_zero_veneer_out_of_the_correlated_stack() {
    let rec = StrataRec {
        events: vec![
            event(M0, 2.5, 4, false),
            event(M1, 0.75, 4, false),
            event(M0, 8.0, 19, false),
            // the veneer: laid after the deep run, not on the deep clock
            event(M1, 1.25, 0, true),
            event(M0, 0.5, 0, true),
        ],
    };
    let bh = Borehole::from_strata(&rec);
    assert_eq!(bh.beds.len(), 3);
    assert_eq!(bh.unclocked_m, 1.75);
    assert_eq!(bh.total_m(), 11.25);
    assert!(epochs_non_decreasing(&bh.beds));
    assert_eq!(bh.beds[1].member, M1);

    // Two epochs stamped, one of them by two beds — the within-interval
    // structure R-C is demoted to, carried but not adjudicated here.
    let part = EpochPartition::build(&[bh.beds.clone()][..]);
    assert_eq!(part.intervals(), 2);
    assert_eq!(part.parent_thickness(0, 0), 3.25);
    assert_eq!(part.parent_run(0, 0), (0, 2));
    assert_eq!(part.parent_run(0, 1), (2, 1));

    let empty = Borehole::from_strata(&StrataRec::default());
    assert!(empty.beds.is_empty());
    assert_eq!(empty.unclocked_m, 0.0);
    assert_eq!(empty.total_m(), 0.0);
}

/// A 9-parent straddling stencil is the same cost and the same shape as a
/// 4-parent interior one — **arity-free**, which is the property that made
/// multi-way sequence alignment unnecessary. Asserted as the mass identity
/// holding under an arbitrary 9-weight partition of unity.
///
/// *Scale-free: the dot product's length is a parameter, not an assumption.*
#[test]
fn the_rule_is_arity_free_across_a_nine_parent_straddling_stencil() {
    let parents: Vec<Vec<Bed>> = (0..9)
        .map(|j| {
            (0..=j)
                .map(|i| bed((i * 7 + j) as u8, 0.5 + f64::from(i) * 1.25, M0))
                .collect::<Vec<_>>()
        })
        .map(|mut v: Vec<Bed>| {
            v.sort_by_key(|b| b.epoch);
            v
        })
        .collect();
    let part = EpochPartition::build(&parents[..]);
    assert_eq!(part.parents(), 9);
    assert!(part.intervals() >= 9);

    let raw = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0, 5.0];
    let s: f64 = raw.iter().sum();
    let w: Vec<f64> = raw.iter().map(|x| x / s).collect();
    let lhs: f64 = part.column(&w).map(|iv| iv.thickness_m).sum();
    assert!((lhs - part.blended_total(&w)).abs() <= part.mass_bound(&w));
    assert!(part.heap_bytes() > 0);
}
