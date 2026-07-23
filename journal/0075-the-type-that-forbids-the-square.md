# 0075 — the type that forbids the square

*2026-07-22. The by-construction end-state of the threshold-quantization
migration (spines § S-4 "Ratified end-state"): `dc_core::coarse::CoarseField<T>`
and its two traits, **extracted** in dc-core from what the two shape-teachers
paid for (journal/0072 move A, journal/0073 move B). One site migrated behind it
byte-identically; the pinned pair consolidated to one slot; the far-field and
octree convergences stated, not built.*

> blogworthy (lens 1 AI-native × lens 3 deepsim-reflexions): **a type extracted
> from the lessons its teachers bled for.** The migration refused to design the
> boundary type first and hunt for callers — the named anti-pattern (spines
> A-4). It converted the two cheapest hot seams, let each teach one move, and
> only then froze the vocabulary. Every method on `CoarseField<T>` has a scar
> behind it: the anchored blend exists because A1's naïve dot broke the
> byte-identity off-switch sub-ULP; `summarize` exists because B1's white noise
> doubled the far mesh; the boundary membership dither exists because the user
> watched minority phases guillotine at cell perimeters and named it the
> cake-slice. The type is the fossil record of its own derivation — and, this
> being the point of writing it down, it *measured its teachers* and found one of
> their claims backwards.

## What the square is, and why a discipline could not kill it

The engine's characteristic S-4 defect: a coarse producer (the 460 m deep-time
sim) hands a value to a fine consumer (per-voxel collapse, the far mesher), and
the consumer performs a **raw per-cell read** and then either thresholds it or
paints it across the fine span. A 460 m square, or an analytic line, reaches the
eye. The user's sharpening: *smoothing a verdict preserves the shape of the cell
that voted it* — a feathered square is still a square. The two shape-teachers
cured two instances by hand (A1 blends the cause and thresholds late; B1 dithers
membership), but a per-site helper family is a discipline you must re-remember.
The ratified end-state is **structural**: a boundary type whose only fine-scale
accessors are the two legal moves, so a downstream square is a *type error* —
the S-6 / `Option<fn>` pattern, inexpressible rather than discouraged.

## The type, and the two moves

`CoarseField<T>` in dc-core (headless — the octree node will name it too, so it
lives where both the deep field and the node can). Its public fine reads:

- **Move A — `sample(pos) -> Option<T>`** for `T: Interpolable`. Bilinearly
  blends the four surrounding cells; a downstream threshold then fires on the
  blended value and follows `T`'s contour, not the grid.
- **Move B — `sample_dithered(pos, src, salt_m, salt_c) -> Option<usize>`** for
  a `ShareVec<N>` field. Dithers *which cell wins the contact*, then draws a
  category from that cell's shares.
- **`summarize(region) -> ShareVec<N>`** — the coarse read (below).
- **No `at_cell`, no `Index`.** The one escape hatch is `_cell_honest`, named to
  be greppable and documented sim-internal-only (residue jurisdiction). A
  `compile_fail` doc-test proves `f.at_cell(..)` / `f[..]` do not type-check —
  the structural proof the amendment asked for.

The verdict is never stored. `ShareVec::argmax` derives it; storing the winner
is *the plurality bug frozen into a type* (B1's phrasing), so `CoarseField` has
no `dominant()` field beside `sample`.

## The three laws, each a teacher's scar

**1. `Interpolable::blend` must be exact at identities, not close (A1).** A blend
that round-trips an identity must be *written* exact there. Two identities are
load-bearing because off-switches rely on them being bit-exact: a one-hot weight
→ that sample (argmax is the limiting case of the blend), and *uniform samples →
that value even when the weights carry normalisation drift*. The naïve dot
`Σ w·s` fails the second sub-ULP; the anchor construction `s[d] + Σ w·(s − s[d])`
(d = argmax w) makes every difference zero when the samples are equal. Encoded as
**generic trait-level tests** (`blend_is_exact_at_one_hot_for_*`,
`blend_is_exact_at_uniform_samples_for_*`, `the_naive_dot_would_drift_here_but_
the_anchor_does_not`) so no future implementor can ship the dot.

**2. The source axis is separate from the draw (B1).** Unbiasedness is a property
of the inverse-CDF *draw* (`ShareVec::draw`, the same shape as `fill::allocate`);
the *source* of the uniform — white noise vs a coherent interpolated field — is a
**separate axis**, because a per-position white-noise draw is only correct for a
consumer reading at the field's own resolution. A coarse consumer point-sampling
it aliases (B1 measured: the far mesh doubled). So the type exposes `DitherSource`
(the caller supplies the addressed uniform — dc-core holds no RNG, entropy stays
caller-owned, one draw machinery, no fork) and offers **`summarize`** for coarse
consumers to dither at their own resolution — near/far agreement then statistical
by design, the octree contract.

**3. The cake law (user).** A continuous *source* over *nearest-sampled* shares
still guillotines minority phases at cell perimeters — the shares are still
cell-quantized, only the expression got smooth. So `sample_dithered` does the
**residue-(b) paired pattern at the surface**: near a boundary it performs a
seeded, bilinearly-weighted **membership dither of the source cell**, then draws
within that cell's shares. Minority phases from each cell interfinger across the
perimeter instead of dying on the grid line — falsified by
`cake_law_a_minority_phase_interfingers_across_a_cell_boundary` (both cells'
minorities appear on both sides, the fraction shifting monotonically across).

## The A1 migration — byte-identical, by opposite acceptance gate

The teachers moved goldens on purpose; this slice must move *none*. `WindowShares`
becomes `ShareVec<{ Litho::COUNT }>` (a `[f64; 7]` cannot implement a dc-core
trait — the orphan rule is exactly why the array becomes the newtype, which *is*
the quantity moving behind the type). `exposed_shares` constructs it;
`blend_susceptibility` delegates to `ShareVec::blend_table` → `f64::blend`;
`dominant_litho` delegates to `ShareVec::argmax`. Each delegation preserves the
float operations in the same index order, so the bits do not move.

State fingerprints, by hash: **UNCHANGED.**
`providers_golden::the_production_world_still_hashes_to_the_pre_slice_goldens`
passed (the production world still hashes to the *pre-slice* `GOLDEN_SURFACE` /
`GOLDEN_RECORD`), and `contents_contract`'s three worlds passed — so every block,
material, and table hash held. The acceptance gate was the opposite of the
teachers' (they re-baselined; this moved nothing), and it is met. The corroborating
tell: the probe's max adjacent rate-jump is `4.4904 → 4.1582`, **bit-identical to
journal/0072's `4.49 → 4.16`** — the blend delegated one call deeper without
perturbing a single result.

## The pinned pair collapsed to one slot

A1 (journal/0072) left `outcrop_at` (verdict) and `outcrop_shares` (quantity) as
a **pinned pair** — two provider slots, one heir, retire-together. The extraction
made the pair redundant: its own law is *categorical answers are the argmax OF the
sample, never a stored field*. So `outcrop_at` stops being an `Option<fn>` slot
with its own identity and becomes a **derived accessor**,
`Providers::outcrop_at(units) = dominant_litho(outcrop_shares(units))` =
`argmax ∘ outcrop_shares`. One slot to supply, one heir socket; when structural
deformation lands and supplies dipped shares, the verdict dips with them
*automatically*, because it is their argmax. Slots: **6 → 5**
(`OutcropAt` removed). Byte-identical because no generation consumer reads the
verdict — erosion reads shares and takes its own `dominant_litho`.

## The finding: the coherent source's bias is backwards (LOUD PLEA)

Carve-out 1 and journal/0073 characterise the coherent (interpolated-uniform)
source's cost as a *"toward-50/50 bias"* that *"flattens mixes"*. Building the
draw in dc-core and measuring it
(`white_noise_is_unbiased_but_coherent_amplifies_the_majority`) shows the
aggregate bias is the **opposite sign**. A bilinear-interpolated `u` is
middle-heavy, so its CDF `F(s) > s` for the class straddling the cumulative-½
point; in a 2-class cell the **majority is amplified and the minority
under-represented** — the rendered mix is pushed *away* from 50/50, not toward
it. The coherent source therefore *sharpens* minority phases (a 0.6 share renders
≈ 0.67), compounding the cake observation rather than easing it. The minorities
still appear — better than the plurality's zero, so journal/0073's cream specks
are real — but at less than their true areal share.

This does not reverse the ratified decision to ship the coherent source (the
far-mesh argument stands); it corrects the *characterisation* of its cost and
re-weights the heirs: the far-`summarize` register and/or a CDF-corrected source
are the honest fixes, and the boundary membership dither eases the *perimeter*
guillotine but not the *within-cell* under-representation. Flagged in spines § 4,
ROADMAP, and here as **NEEDS RATIFICATION**.

## What was NOT done (owners, not this slice)

- collapse.rs / far.rs are **not** rewired to the type — a parallel agent owns
  collapse.rs, and adoption is follow-on for those owners. `sample_dithered` and
  `summarize` ship as **type capabilities with their own falsifiers**, not live
  consumers.
- The octree node payload adopts the sampling vocabulary in a follow-on
  (octree-substrate.md § 3, doc-comment + one-line note — stated, not built).
- Audit residue A2 (rides A1's table) / A3 / A4 / B3 held for their owners.

## Tests (by name and count)

- **dc-core** (`coarse.rs`): **13 unit tests + 1 `compile_fail` doc-test**, all
  green. The laws: `blend_is_exact_at_one_hot_for_f64` /
  `..._for_sharevec`, `blend_is_exact_at_uniform_samples_for_f64` /
  `..._for_sharevec`, `the_naive_dot_would_drift_here_but_the_anchor_does_not`,
  `argmax_is_the_limiting_case_of_blend_table` (law 1);
  `sample_is_exact_over_a_uniform_field`, `sample_interpolates_between_cells`
  (move A); `the_draw_is_unbiased_over_the_uniform`,
  `cake_law_a_minority_phase_interfingers_across_a_cell_boundary`,
  `summarize_then_coarse_dither_matches_fine_dither_within_tolerance`,
  `white_noise_is_unbiased_but_coherent_amplifies_the_majority` (laws 2/3);
  `cell_honest_is_the_only_raw_read_and_it_is_named`; and the structural proof
  `coarse::_raw_read_is_inexpressible (compile fail) ... ok`.
- **dc-worldgen**: `outcrop_blend.rs` (4, the A1 falsifiers), `providers_outcrop_at.rs`
  (5, the consolidated derived-verdict falsifiers), `providers_set.rs` (3, the
  5-slot report), `full_agents.rs` (10, one provider fn re-typed to `WindowShares`),
  and the byte-identity goldens `providers_golden.rs` (2) + `contents_contract.rs`
  (3) — all green. Full `test --workspace --release` green (dc-core + dc-worldgen
  `Compiling` lines cite this worktree; clean-first defeated a cross-worktree false
  green that short-circuited the first two gate runs — corrections #34, live).

## Perf (A1's 38.6 ms baseline)

`outcrop_blend_probe`, production Medium (297,025 cells, mean of 20): argmax
lookup **34.49 ms**, share blend **38.99 ms** — versus A1's 38.6 ms blend
baseline, a **+1 %** move inside run-to-run variance (the argmax reference also
moved, 36.3 → 34.49 ms, same-machine noise). No hot-path regression: the blend
delegates `blend_susceptibility → ShareVec::blend_table → f64::blend`, all
`#[inline]`, and the max adjacent rate-jump reproduced A1's exactly.

## The thread

A1 taught that a seam can be pointed at the wrong *granularity* (the verdict
where erosion wanted the quantity). B1 taught that the *source* of a draw is a
separate axis from its unbiasedness. The user taught that a smooth source over a
quantized cause still cuts minorities at the perimeter. The type is those three
lessons made structural — and writing the draw down in one honest place is what
surfaced the fourth: the coherent source we already shipped does not flatten
mixes toward 50/50, it sharpens them away from it. The most valuable output of an
extraction is the teacher it corrects.
