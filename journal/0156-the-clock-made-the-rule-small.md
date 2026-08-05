# The clock made the rule small

*2026-08-04 · the geo session · S1 of the correlation arc — the kernel, unwired*

> blogworthy: **lens 2 (procgen against priors)** — a design pass reasoned carefully to a
> proportional correlation rule, and a small storage change one day later made the rule
> *smaller* rather than better; the road not taken (sequence alignment) is the one a human
> geologist actually walks, and why it fails for a machine reading four boreholes at once.
> Also **lens 1 (AI-native development)** — a build-slot deadlock turned into a Python
> mirror of the kernel that caught a genuine numerical defect before the compiler ever
> ran, and the fix is a worked example of *derive the bound, don't tune it*.

## The rule we were going to need

The stratigraphic-correlation design pass had to work for its answer. With no time in the
record finer than eight tectonic chapters, it reached for **R-C**: partition each borehole
by chapter, then correlate *within* a chapter by cumulative thickness fraction — the bed at
40 % of A's chapter 5 corresponds to whatever sits at 40 % of B's. That is the standard
construction when internal surfaces are undated, and the pass named its weakness honestly:
it correlates by *position, not identity*, so a distinctive storm bed at 40 % in one
section pairs with a stranger in the next.

It also priced the alternative — bed-to-bed sequence alignment, what a geologist does with
two measured sections — and rejected it on **correctness**, not cost: multi-way alignment
is not pairwise-composable, so `align(A,B)` and `align(B,C)` need not agree with
`align(A,C)`, and a chunk's partition becomes order-dependent. That reintroduces seams
*between chunks*, the exact artifact class this arc exists to kill.

Then the deposition clock landed, and the question dissolved.

## The join

Every `DepUnit` now carries the raw runner tick it was first deposited on, and the
collapse-tier event carries it through. So the shared partition is not something you
*construct* — it is the **sorted union of the epochs the parents stamp**, and interval *k*
simply *is* epoch *k*. A bed's own epoch is always a boundary of the union, so no bed is
ever split and none can land in a stranger's interval. A proportional guess became a keyed
join. R-C did not die; it shrank to the interpolant *inside* one interval, which is where
it was always defensible.

The satisfying part is what stopped needing code:

- **Pinch-out is not a special case.** A parent that stamped nothing at epoch *e*
  contributes `0.0` through the ordinary dot product. P-4's ruling — *"an empty stack is a
  parent whose every interval thickness is 0"* — means the onlap feather against basement,
  the fix for the razor-vertical wall at the 0149 station that opened this whole arc, has
  **no branch anywhere in the file**. It falls out of blending with zero.
- **Seam-freeness is structural, not careful.** Because an interval is keyed by *epoch
  value*, a parent with zero weight can only contribute intervals that are empty for
  everyone else — so two adjacent chunks whose epoch unions genuinely differ still produce
  bit-identical structure at their shared boundary. The test asserts exactly that, and the
  equality is bitwise.
- **Mass is an identity**, not a tolerance: blend-of-sums equals sum-of-blends.

R-C′ was deliberately not built — interior unconformity flags are structurally zero
(corrections #99). No discontinuity provider shipped either: an empty seam nobody calls is
`spines.md` § 3 material, and this arc has enough of those already.

## The shape, which S0 had already decided

Measured, `column()` is **1.62 ms/chunk** — not the ≈14 ms the design's hand-arithmetic
assumed — and naively rebuilding per-column fill state costs **+104.8 %** of that plus
~3 MB/chunk of transient in a runtime-resident cache. So the kernel materializes exactly
**one partition table per chunk** (~2.9 kB mean) and evaluates each of 1024 columns as an
**iterator carrying a running cumulative depth, allocating nothing**. The voxel walk
already visits spans in depth order; this hands it spans in depth order.

## The Python mirror, and the bound that was attained

The build slot was held for most of an hour by a live game walk. Rather than idle, the
slice author wrote a **Python mirror of the kernel** — same IEEE-754 doubles, same
accumulation order — and ran the invariants through it before the compiler was available.

It found a real defect. Along one axis the blended total is *exactly linear* in the
weights, so the Lipschitz bound derived for invariant 2 is not merely respected but
**attained** — and an eight-ULP tolerance failed **thirty times out of eighteen hundred**.
The fix was not to widen the slack until it passed. It was to notice that comparing two
*computed* totals against a bound on the *exact* difference needs the evaluation's own
rounding at both ends — derived as a second quantity — and to compare the *realized* step
rather than the nominal one, since `fx0 + du` rounds. **A bound with a derivation is
evidence; one chosen until green is not** — and here the derivation is what found the bug.

## An impossible red, for once

The first full-suite run came back red with an unresolved import in a file the slice never
touched, naming a symbol that demonstrably exists. Three `libdc_core` rlibs were sitting in
the shared target directory, written by three concurrent worktrees, and rustdoc had been
handed a sibling's mid-refactor copy. Every actual test binary had passed; only the
separate doc-test stage was poisoned. It is corrections #27's hazard running in the **red**
direction — an impossible red rather than a false green — and the discipline is the same
either way: when a result looks impossible, suspect your own pipeline before the artifacts,
and re-run when the machine is quiet. It came back clean.

## What S2 inherits

Three things, all named at the site rather than left for someone to find:

1. **The continuum predicate is OWED, not half-built** — and the reason is the interesting
   part. P-1's M-C rule needs *regions adjoining on a declared axis*, and **no material
   declares a region**: `MaterialProps` is point-valued, and FS-A's spectra describe what a
   rock *sheds*, not where it *sits*. Any predicate buildable today would need a threshold
   on a scalar distance — the fitted taxonomy `materials.md` forbids by name. This belongs
   to the parent-materials / term-schema design pass, not to S2 as an implementation
   detail. The constructive pointer: `GrainGrade`'s Wentworth partition already declares
   real interval bounds; that is the shape the property sheet needs, per material.
2. **An A-1 risk, marked** — `bilinear_weights` mirrors four lines of `CoarseField::stencil`
   (private, one crate over) so that invariants *about* those weights are testable without
   widening another crate's API. S2 repays it by sourcing weights from the field's own
   stencil (`stubs.md` #54).
3. **The veneer is split out, never dropped.** Year-zero veneer events carry
   `epoch_bottom == 0` by charter rather than by measurement; folding them into interval 0
   would correlate present-day veneer with the world's oldest beds. `Borehole::unclocked_m`
   is the number that stops S2 losing that mass — where the veneer *sits* in the blended
   column is S2's call.
