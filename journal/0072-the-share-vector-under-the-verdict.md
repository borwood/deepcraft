# 0072 — The share vector under the verdict

*2026-07-22. Shape-teacher #1 of the threshold-quantization migration (audit
`docs/audits/2026-07-22-threshold-quantization-audit.md`, site A1). Erosion's
per-agent susceptibility stops being argmax-then-lookup and becomes a
share-weighted blend over the dominance window, so the coherent S-4 rate
boundary the walk-0071 flag named dissolves by construction. STUB — measurements
land with the gate-green commit.*

> blogworthy (lens: deepsim-reflexions): **a seam can be pointed at the wrong
> thing.** `outcrop_at` seamed the *verdict* — one `Litho` per cell — because
> that is what erosion happened to read. But erosion never wanted the verdict;
> it wanted a rate, and it was deriving the rate by collapsing a rich quantity
> (how much of each rock fills the near-surface window) down to its argmax and
> then looking that one label up in a table. The collapse *was* the S-4 defect.
> The cure is not to smooth the label downstream — a feathered square is still a
> square — it is to never form the label on the rate path at all: blend the
> table by the window's share vector, of which argmax is the limiting case.

## What was wrong (the walk-0071 flag, S-4 sharpened)

journal/0068 replaced a name-keyed charcoal carve-out with a thickness rule:
`exposed_litho` returns the lithology *dominating* the topmost
`OUTCROP_DOMINANCE_WINDOW_M` (0.9 m) of the record. Good rule — but walk-0071
caught its S-4 edge: along a deposit that thins gradually across country, the
window's plurality winner flips from (say) fine to basement at one contour, and
adjacent 460 m cells get *discontinuously different* erosion rates. Unlike the
old top-unit rule's noise-like flips, this boundary is **spatially coherent** —
it follows a thickness contour — which is exactly the analytic line S-4 forbids
reaching the eye (through erosion → terrain shape).

The user's S-4 sharpening (spines § S-4, 2026-07-22): *smoothing a verdict
preserves the shape of the cell that voted it.* The legal cure for an
interpolable cause is **A — threshold late, at fine scale, on the interpolated
cause.** Here the cause is the per-`Litho` thickness *shares* in the window, and
they are interpolable; the consequence is a scalar rate. So the cure is a
**share-weighted susceptibility blend**: a cell whose window is 55 % basement /
45 % fine gets a 55/45-weighted rate per agent, and the rate field then follows
the thickness contours continuously — no step, no line.

## The change

The window walk that `exposed_litho` already does now also publishes its raw
material: `exposed_shares(units) -> [f64; Litho::COUNT]`, the fraction of the
0.9 m window each lithology fills (deficit below a short record → `Basement`;
the vector sums to 1.0 by construction, since the accumulator always totals
exactly the window). Erosion's four consumption sites
(`expose`/`periglacial`/`wind`/`wave`) stop mapping one `Litho` to one table
entry and instead `blend_susceptibility(&shares, &sus_tab)` — the dot product of
the share vector with the per-`Litho` agent table. Argmax is the limiting case:
a window that is 100 % one lithology blends to exactly that lithology's rate,
bit-for-bit.

The blend is a pure function of the record — no entropy, no dither (this is
move A, not move B). The share vector is computed once per cell per epoch, into
the same cached tables the litho/sus_flow/sus_creep refresh already builds
around `erosion.rs` `expose` — never inside an inner loop.

`exposed_litho` (and the `outcrop_at` provider seam it is the identity of) is
**kept, unchanged in behaviour**: it still answers identity questions with a
single `Litho`, as the argmax over the same share vector. What changed is
erosion's *consumption*, which moved from the verdict to the quantity.

## The design call: quantity seam vs direct helper — and what it taught

<!-- TODO: fill after implementation settles. Capture:
 - the choice made (outcrop_shares slot / direct helper) and WHY
 - S-5 corollary "seam the quantity, not the verdict"
 - the heir (structural deformation) supplies the SAME quantity once beds dip;
   pinned-pair vs single-socket implication
 - THE SHAPE-TEACHER LESSON for the future CoarseField<T> API:
   * the fine accessor exposes the share vector (Interpolable), NOT the verdict
   * the verdict is argmax ∘ sample, never a parallel stored slot
   * blend_susceptibility is the Interpolable::blend witness
-->

## The measured delta (S-4 contour check)

<!-- TODO: max adjacent-cell rate jump before/after over production Medium;
     histogram of rate deltas at former flip boundaries. examples/outcrop_blend_probe.rs -->

## Runtime cost

<!-- TODO: per-epoch table-refresh time, argmax vs blend, Medium. -->

## What it cost in goldens

<!-- TODO: GOLDEN_SURFACE/RECORD old→new; contents_contract 3 entries incl. Small;
     coal-dig census. -->

## The thread

<!-- TODO -->
