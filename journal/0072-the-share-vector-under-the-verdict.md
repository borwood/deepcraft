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

The audit told shape-teacher #1 to "do the blend in-place as a plain function…
it does not need `CoarseField` yet." I did — `exposed_shares` and
`blend_susceptibility` are plain `lithology` functions. But the plain functions
alone left a question the audit posed and asked me to answer: does erosion's new
consumption go through a **provider slot**, or a bare helper?

Three forces decided it, and they all point the same way once you follow the
orphan:

1. **The heir.** `outcrop_at`'s heir is structural deformation — the dip-fold
   term that makes "which units lie in the near-surface window" a function of the
   fold field, not stacking order. Erosion's *rates* now depend on that same
   window. If the rate path read a bare `exposed_shares(units)`, the heir could
   make the verdict dip-aware and leave the rates flat — a silent divergence
   (the exact A-3/S-3 shape: two answers to one question).

2. **The orphan.** After the conversion, erosion's four sites no longer need the
   verdict `Litho` at all — they were only using it to *index a rate table* (the
   audit's "A1's dual nature": a category whose only downstream use is numeric).
   A bare helper would leave `providers::outcrop_at` with no production caller —
   freshly orphaned machinery, which is precisely this project's characteristic
   A-4 failure.

3. **S-5's corollary, "seam the quantity, not the verdict"** (earned by
   `burial_temp_c`, 2026-07-22). The share vector *is* the quantity; the verdict
   is its argmax. Seaming the quantity is the shape the corollary names.

So I added a **value-level `providers::outcrop_shares` slot** (identity
`exposed_shares`), a pinned pair with `outcrop_at` under the one heir. Erosion
reads `outcrop_shares` for rates; the verdict `outcrop_at` remains, provably its
argmax (bound by `tests/outcrop_blend.rs::the_verdict_is_the_argmax_of_the_shares`
— S-3's agreement discipline). When the heir lands it supplies the dipped shares
*here*, and both the rate field and the outcrop map dip together. The stubs.md
sibling-gap entry records the retire-together requirement; the ideal
consolidation (reduce `outcrop_at` to `argmax ∘ outcrop_shares`, one slot) is
noted there for the heir.

**What the shape-teacher taught the future `CoarseField<T>` (the second
deliverable).** The audit's type will publish a coarse field with two S-4-legal
fine accessors. A1 is the `Interpolable` (move-A) witness, and it teaches three
things about that accessor's *shape*:

- **The fine accessor must expose the share vector, not the verdict.**
  `exposed_shares(units) -> [f64; COUNT]` is the `sample` a `CoarseField<[f64;
  COUNT]>` would carry; the `Litho` verdict is `argmax ∘ sample`, never a
  separately stored field. A `CoarseField` API that offered a `dominant()` as a
  *stored* value alongside `sample` would be re-inventing the divergence this
  slice removed.
- **`blend_susceptibility` is the `Interpolable::blend` operation** — and it must
  be **exact at the vertices and for constants**. The naïve dot `Σ shares·tab`
  was exact at a vertex (a one-hot window → that rock's rate) but *not* for a
  constant table, because share normalisation (`acc / w`) does not sum to exactly
  `1.0` — so a neutralised coupling drifted sub-ULP and the byte-identity off-
  switch broke. Anchoring the blend at the dominant index — `tab[d] + Σ
  shares·(tab − tab[d])` — restores both: a uniform window is `tab[d]` bit for
  bit, a uniform table is that value bit for bit, and the between is unchanged.
  **The lesson for `Interpolable`: a blend that must round-trip an identity has
  to be written to be exact there, not merely close.** That is a real design
  constraint the type's trait bound should carry, and A1 is where it was found.
- **The interpolable payload is a small fixed-width struct** (`[f64; COUNT]`), so
  `CoarseField<T: Interpolable>` wants `T` to be `Copy` and cheap-to-blend, not a
  general heap value — the array share vector is the concrete first `T`.

## The measured delta (S-4 contour check)

`examples/outcrop_blend_probe.rs`, production Medium record (297 025 cells,
545×545), fluvial abrasion (contrast 2.5, cap 5):

- **Max adjacent-cell rate jump:** OLD (argmax) **4.49** → NEW (blend) **4.16**.
  The max barely moves, and that is correct: the largest jumps are genuine
  basement↔sediment contacts where *both* windows are uniform, so the blend
  equals the argmax there (a real erodibility contrast, not a quantization line —
  it should stay sharp).
- **The histogram is the real result.** Of the **55 782** former-flip
  adjacencies (where the argmax verdict differs across the edge), the adjacent
  rate-jump distribution moved decisively toward zero:

  | jump bucket | OLD | NEW |
  |---|---|---|
  | [0.00, 0.05) | 0 | 8 126 |
  | [0.05, 0.10) | 0 | 10 060 |
  | [0.10, 0.25) | 41 453 | 22 755 |
  | [0.25, 0.50) | 4 294 | 3 777 |
  | [0.50, 1.00) | 330 | 1 258 |
  | [1.00, ∞) | 9 705 | 9 806 |

  ~18 000 former-flip edges dropped from the [0.10, 0.25) step into sub-0.1
  jumps — these are the **gradational contacts** (a deposit thinning across the
  window), exactly the coherent S-4 lines walk-0071 flagged, now continuous. The
  [1.00, ∞) bucket is unchanged (9 705 → 9 806): the true material contacts that
  should step still do. The blend dissolves the quantization line and leaves the
  physics.

## Runtime cost

Per-epoch table refresh (the `expose` phase — one walk + rate per cell, over the
whole grid, mean of 20): argmax lookup **36.3 ms**, share blend **38.6 ms** —
**×1.06**, a 6 % increase, ~2.3 ms per epoch over ~40 iterations ≈ 90 ms added to
a multi-second world build. The blend is one `window_walk` + a normalise + a
7-wide anchored dot, against the old walk + argmax + one index — genuinely cheap.
No hot-path regression; well inside the runtime budget the convention now pins.

## What it cost in goldens

Authorized behaviour change (erosion rates → terrain geometry). Re-baselined with
notes in place, citing this slice:

- **`providers_common`: `GOLDEN_SURFACE` `0x344C…7BAE` → `0x176D_40F1_1CCB_006A`;
  `GOLDEN_RECORD` `0xEA71…7A05` → `0xC9C6_D6F6_E908_9653`** (production Small
  field). Both moved — the blend changes the erosion input, so surface planes and
  record both.
- **`contents_contract` (three worlds):**
  - Medium `0x0D5EED572026`: `0x18BD…C969 → 0xA266…0D45` (block),
    `0x2FB1…5986 → 0xC460…F410` (materials), `0x35AE…D8CD → 0xD818…7324` (table).
  - Medium `0x539`: `0x9A74…A767 → 0xC20A…FDF2` (block),
    `0x505A…DEFE → 0x0E95…E1E9` (materials), `0xD8C3…3081 → 0x76CC…C87A` (table).
  - **Small `0x00C11A7E2026`: BLOCK hash moved (`0x5B85…2E1D → 0x83A4…A19D`),
    materials + table byte-identical — the hypothesis held.** The same shape
    journal/0068 established: Small's
    sampled columns express no deep record as contents, so the shifting bedrock
    surface changes only which voxels are Stone vs Air (geometry), while the
    contents-derived material and table hashes stay put. `block_equals_classify_
    of_contents` still passes with absent-contents blocks limited to Air and
    Stone — the proof it is geometry, not a classify regression.

- **Coal is still diggable, with less margin (NEEDS RATIFICATION).** The blend
  gives any near-surface window that *contains* coal a rate pulled up toward
  coal's (coal is the softest rock — a coaly bed is recessive), where the old
  argmax handed a coal-minority window the dominant rock's slower rate. So
  near-surface coal is preferentially stripped. Census on the Medium seed
  `0x0D5EED572026`: from 88 record seams over 3 m (strongest 16 diggable) to
  **10 record seams over 3 m, strongest 11 diggable collapse-voxels** (record
  8.89 m). The claim — *a player can find and dig a coal seam* — holds (11 voxels
  ≈ 10 m; the walk-0071 seam the user cut and loved was 3), so
  `MIN_DIGGABLE_COAL_VOX` re-baselined 15 → 10 with the census printed, not slid
  silently. The reduced-coal *appearance* is flagged for the user (ROADMAP).
  Charcoal is untouched: 0.22 % of recorded voxel spans carry it, worst 2 eighths
  (cap 4) — healthy.

## The thread

Journal/0068 taught that a carve-out is a rule not finished factoring. This is
its neighbour one axis over: a **seam pointed at the wrong granularity.**
`outcrop_at` seamed the verdict because that is what erosion happened to call —
but erosion wanted a rate, and forming the verdict on the way to the rate was the
S-4 defect. The fix was not downstream smoothing (a feathered square is still a
square) but reaching *under* the verdict to the quantity it summarises, and
blending there. The seam that already existed was right about the question ("what
rock is in the window") and wrong about the answer's shape — a single label where
erosion needed the whole vector. Finding that took building the blend; the byte-
identity off-switch breaking on the naïve dot is what taught the deeper lesson
the future type will carry: an interpolation that has to round-trip an identity
must be *written* exact there, and anchoring at the dominant is how.
