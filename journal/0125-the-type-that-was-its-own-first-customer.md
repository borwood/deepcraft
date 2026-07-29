# 0125 — The type that was its own first customer

*2026-07-29. E5 member #0, first build slice: the far site. `docs/dependency-graph.md`
E5 / P4 / P6.*

---

## Extraction is not adoption

On 2026-07-22 this project built `dc_core::coarse::CoarseField<T>` — a boundary type
whose public surface offers exactly two coarse→fine moves and no raw per-cell read, so
that the characteristic defect it was built to forbid (a 460 m cell's verdict painted
across the ground) is a *type error* rather than a discipline. It was ratified in the
strongest language anything in this thread has been ratified in: *"we finish this
today"*, *"this is foundational"*, end state *"solved by construction, approximately
once."*

Then nothing called it for seven days.

`spines.md` § S-4 recorded the state as **"Ratified end-state — EXTRACTED."** That word
is accurate and it is the whole problem: it names the half that shipped, and it reads as
*done* to every sweep that passes it afterwards. Three sweeps did. The largest
built-and-unconsumed mechanism in the palette-quant thread was invisible to all three of
this project's loose-end loci — `spines.md` § 3, `stubs.md`, and the ROADMAP — for a
week, because the status word said the work was over.

This entry is the first call site.

## What "adopt it" turned out to mean

The site is `collapse.rs::surface_class`: the far field's answer to *what rock is the
ground made of here*. Since journal/0073 it has been an inverse-CDF draw over the top
0.9 m of a deep cell's strata record, keyed by a coherent bilinear uniform. Since
journal/0074 it has fed **only** the far field — the near ground's surface voxel comes
from the record's top span through `ColumnFill`.

Written out, that function *was* `ShareVec::draw`, by hand, over a NEAREST cell. The
adoption is a kernel swap, and most of it is deletion:

- `draw_class` and its unbiasedness test retire into `dc_core::coarse::ShareVec::draw`
  and *its* unbiasedness test, which absorbed the retiring test's three share vectors
  verbatim — including the un-normalized metres case, which is the one that proves
  normalisation is the type's job and not the caller's.
- The hand-rolled coherent field becomes `draws.rs::Coherent`, the first production impl
  of `DitherSource` — a trait that had, until today, zero of them. journal/0075's
  *"dc-worldgen wraps its own `draw_f64`"* stopped being a plan.
- The shares become a `CoarseField<ShareVec<6>>`, and `sample_dithered` does the draw.

And the **cake law arrives for free**. That is the point of the exercise and it is worth
being precise about what "free" means here. The user's 2026-07-22 observation (U22) was
that a continuous *source* over *nearest-sampled* shares still guillotines minority
phases at cell perimeters — a cell that is 55 % coarse clastic and 45 % fine renders
both, in shifting proportion, right up to the 460 m line, and then stops dead because
the neighbour's share vector is a different number. `sample_dithered` cures it in a step
the hand-rolled version simply did not have: **before** drawing a class, one uniform
picks *which of the four surrounding cells' shares to draw from*, weighted bilinearly.
Near the frontier the weights are ~½/½, so each cell's minorities surface on both sides
of the line.

Nobody wrote that step in this slice. It was written on 2026-07-22, tested on
2026-07-22, and sat in a file nothing imported.

### And it is not a perimeter treatment — the gate had to tell us that

The sentence I wrote first, and then had to delete, was: *"away from a frontier one
weight is ≈ 1, so it reduces to the containing cell's own shares — which is why cell
interiors are unchanged and only the frontiers moved."* That is a paraphrase of
`coarse.rs`'s own doc comment, and it is **wrong about the magnitude in a way that
matters**.

One weight is ≈ 1 only *at the cell centre*. Averaged over a cell, the weight on the
home cell is

```
E[w_home] = 4·(∫₀^½ (1−x) dx)²  =  4·(3/8)²  =  9/16  =  0.5625
```

so **about 44 % of far voxels take a neighbouring cell's shares — everywhere, not just
near a line.** The membership dither is a cell-wide continuous blend. That is arguably
*more* faithful (real facies boundaries are gradational, and the record's hard cell edges
are an artifact of the sim's resolution rather than a fact about the ground), but it is a
much bigger claim than "the frontiers got softer," and the difference is a design fact
the user should get to weigh rather than inherit.

Nothing in the reasoning caught it. What caught it was a **failing gate**:
`coarse_surface_agrees_with_the_near_column_surface`, the statistical near/far agreement
test, dropped from 0.8225 to **0.7922** and tripped its 0.80 floor. A per-voxel
agreement between "the far draw" and "the near column's own cell" is exactly the
quantity that a cell-wide blend has to lower, and it lowered it by 0.030 — which is the
whole 44 % effect, seen from the side.

**Re-deriving that floor was the interesting half.** A tolerance moved to fit a
measurement is not evidence (CLAUDE.md: *a bound with a derivation is evidence; one
chosen until green is not*), so the new floor is computed from the mechanism before it is
compared to the run. With journal/0112's two measured rates on this same world —
same-cell coincidence 0.8225, independent-cell collision floor ≈ 0.34 — the two brackets
are:

- neighbours statistically **independent** of the home cell (a mis-registered window, a
  broken stencil, a biased draw): `0.5625·0.8225 + 0.4375·0.34` = **0.611**
- neighbours **identical** to it (a uniform world): **0.8225**, unchanged

Measured 0.7922 sits between them and back-solves to a neighbour-cell coincidence of
**0.753** — adjacent 460 m cells agree with each other nearly as often as a cell agrees
with itself. That is what a spatially autocorrelated facies field looks like, and it is
independent evidence that the local window is registered correctly, which is the one
thing MM-6 said would fail silently. **The floor is now 0.70**: 0.09 below the
measurement, 0.09 above the independent-neighbour prediction, and it means something —
*below this line the neighbours contribute no more agreement than randomly chosen cells*,
which is the failure the test exists to catch. It is no longer "yesterday's number minus
a margin."

## Three things the design spec left for the build to decide

The [member-#0 design pass](../docs/audits/2026-07-29-member0-coarsefield-design.md) was
thorough enough that the build had few surprises. It had three.

**1. `DitherSource` has one salt; this crate's entropy has three axes.** dc-core's trait
is `uniform(wx, wz, salt) -> f64`. dc-worldgen addresses a draw by *(domain, tag,
address)*, and a domain is a **type** — `draw_domains!` makes duplicate salts a compile
error precisely by making them types — so no runtime `u64` can select one. The only
honest mapping is: the **domain is fixed when the source is constructed** (which is also
where the seed enters), and the caller's **salt becomes the tag**. One `Coherent` is one
domain's coherent field; a consumer wanting two independent draws at one position passes
two salts, which is exactly what `sample_dithered` does. This inherits `draws.rs`'s
own hole 1 — hand-laid tag space inside a domain — rather than curing it, and says so.
The class draw keeps **tag 0**, so the class uniform at a given voxel is bit-identical
across the adoption; only the shares it indexes, and which cell they came from, changed.

**2. What happens where there is no record.** A deep cell whose top window holds less
than half a voxel of strata is bare rock — the 0.2 % case journal/0053 bought — and the
far field names it after the igneous province instead. Under the old code that decision
was taken about the NEAREST cell. Under a field it could be taken either way, and the
choice matters more than it looks: gating on the nearest cell while dithering the class
mixes a NEAREST decision into an interpolated one, which is the exact shape the design
pass's own Law-3 warning is about (*"`H` and `record_at_voxel` are two terms of a
difference; interpolating one and not the other leaks mass"* — move both or neither).
So a bare cell contributes `ShareVec::zero()`, `sample_dithered` returns `None` there,
and the bare/covered contact interfingers under the same law as every other contact.

**3. Registration is in voxels, and the constant lies.** The design pass flagged this as
MM-6 and it was the single most likely way to ship a wrong world quietly. `DEEP_CELL_M`
is **460.0 metres**; `Registration`'s own doc-test in dc-core models 460 as **voxels**;
a voxel is 0.9 m. Reading one as the other is an 11 % registration error that no test in
this repository would have caught, because every internal instrument would still have
agreed with itself. And `DEEP_CELL_M / 0.9` is *also* wrong in general — the deep grid
width is capped at `DEEP_MAX_WIDTH`, so a Large world's real cell is ~1.8 km, and even
under the cap `w` is a rounded cell count, so the true pitch is `extent / w`. `DeepField`
now hands out its own `Registration`, derived from `deep_coords`, with the units in the
doc comment and an agreement test against `deep_coords` pinning the derivation. *Derive
the pitch from the grid, never from the constant.*

## The acceptance criterion that could not fire

The slice's spec said, flatly: **goldens move — re-capture them with the why recorded.**
It is the right instinct. The draw changed from one-uniform-over-nearest-cell-shares to
membership-dither-then-class-draw, and the canonical order changed too (the CDF used to
be indexed by class *name*, alphabetically; a `ShareVec` slot is `Litho` declaration
order — both are recorder-order-independent, which is the property that ever mattered,
and neither is more correct). Two mechanisms, both of which move which rock a given
voxel shows. Of course the goldens move.

They did not move. **Not one hash in the workspace changed** — and the reason is worth
more than the change.

`contents_contract.rs`'s three golden triples hash **generated chunks**. Chunks come
from `generate_chunk`, which since journal/0074 does not consult `surface_class` at all.
`providers_golden` / `rate_axis` / `creep_operator` hash the deep-time **surface planes
and strata record**, which are upstream of the collapse tier entirely. The far field —
`coarse_surface`, the thing a player sees for every metre of ground beyond the loaded
radius, the thing this slice rewrote — **is hashed by nothing**.

That is a coverage hole, and it had been invisible for the ordinary reason: the far
field has behavioural tests (a split-count floor, a near/far statistical agreement test)
and behavioural tests do not notice that no *fingerprint* exists. A slice that changed
the far field's mechanism twice over produced a byte-identical golden run, and if it had
also broken something, the golden suite would have said so just as confidently.

It is filed in ROADMAP § Observed (*"NO GOLDEN HASHES THE FAR FIELD"*, with the heir
named: hash `coarse_surface` over a fixed strided sample, beside the three chunk hashes)
rather than fixed here — a far-field fingerprint is a new instrument, not a
line of this slice — but the honest reading is that "goldens move" was the *right*
acceptance criterion and the corpus could not answer it. **A green golden is evidence
about what it hashes and nothing else**, which is the same sentence this project already
writes about gates.

## Cost

The far tier runs on the runtime clock, so `coarse_surface`'s throughput is a gate like
any other. Measured A/B **in one binary** — the pre-adoption nearest draw and the adopted
`sample_dithered` behind a test-only switch, so the comparison is not across builds and
not across machine states:

| sweep | before (nearest) | after (`sample_dithered`) | Δ |
|---|---|---|---|
| **stride 16** — far-mesher-shaped, 490 k calls | 3519.5 · 3557.9 ns/call | 3652.7 · 3415.8 ns/call | **−0.1 %** (inside a ±3.4 % run-to-run spread) |
| **stride 1** — dense contiguous, 1.96 M calls | 1626.0 · 1633.2 ns/call | 1458.8 · 1335.3 ns/call | **−14.3 %** |

Two runs of each, interleaved `before / after / before / after`, seed 1337, `Extent::
Medium`, release. **Expected neutral; measured neutral on the strided sweep and
14 % *faster* on the dense one.**

The shape of that result is the interesting part. Naively the new path is strictly more
work: an extra uniform per voxel (two coherent draws instead of one, and each is four
corner hashes plus a bilinear blend). But it also **stops re-scanning the record's unit
list per voxel**. The old code walked a cell's strata from the top on every single call;
the new code walks sixteen cells once and then answers from a 768-byte window until the
sample leaves the deep cell it was cut for — and a deep cell is ~511 voxels across, so a
dense sweep pays that build about once per 511 samples per row, and the extra hashes are
cheaper than the scan they replaced. The strided sweep shows neutral rather than faster
for the obvious reason: at stride 16 the whole call is dominated by the elevation
pyramid (3.5 µs/call against 1.6 µs dense — the memoised lattice is what the stride
defeats), so the class draw is a small fraction of a much bigger number either way.

Which is the honest caveat on the whole measurement: **`coarse_surface` is mostly not
this code path**, and a 14 % move on the dense sweep is a real but bounded win on the
part of it that is.

The window is deliberately the cheap escape, not a resident field (design pass MM-2): a
global `CoarseField` over the production grid's ~300 k cells would be ~14 MiB of
`ShareVec<6>` resident for a summary that costs microseconds to rebuild. **Do not grow a
borrowed or lazy `CoarseField` variant until a second consumer asks for one.** It is 4×4
rather than the 2×2 the bilinear stencil needs, and the extra ring buys two specific
things: no clamp ever fires (so the weights are the weights a global field would have
computed), and a whole chunk's worth of voxels around the base cell stays interior, so a
chunk-ordered sweep rebuilds once per deep cell rather than once per boundary voxel.

## What this did not fix

Naming these matters more than usual, because the entry above reads like a completed
arc and it is one half of one.

- **U3 — the near-field checkerboard — is untouched.** Same root (`record_at_voxel` reads
  NEAREST at 460 m), different site, and the design pass found the old plan does not
  survive contact with the record: the near path needs a separately callable membership
  dither (MM-1) and a declared type for the working sub-cell state (MM-3), which does not
  exist. And it is co-requisite with the octaves source — the U3 squares' *dominant*
  signal was settled on 2026-07-24 as the single-octave member dither, so a perfect
  near-path fix leaves them on screen.
- **The coherent source's majority-amplification bias** (corrections #39) rides
  unchanged, now at *two* salts instead of one. Its named heir was the far field
  `summarize`-ing shares at its own resolution — and on the same day this slice was
  approved, the user ruled `summarize` to the **octree node contract**. So that bias no
  longer has an heir inside the refinement tier at all; a CDF-corrected source is what is
  left.
- **`coarse_surface` still paints one `Block` across a `level_stride` footprint** (1.8 –
  14.4 m). A smaller square, still a square, and it belongs to the same octree contract.
- **The member-dither guillotine** (`geology.rs::dithered_member` under a chunk-centre
  formation context) — U22's named sibling — is a different site with a different cause
  and remains unexamined.

## The shape worth keeping

Three of them.

**A doc comment's qualitative clause hid a factor-of-two.** `coarse.rs` says *"away from
a boundary one weight ≈ 1, so it reduces to the containing cell's own shares."* Every
reader of that sentence — including its author, including the design pass, including this
slice's first draft — takes away "this is a boundary treatment." The integral says the
home cell wins 9/16 of the time. **The sentence is true pointwise and misleading in
aggregate**, and no amount of re-reading it would have produced the number; you have to
integrate it. The general form is uncomfortable: *a doc comment that describes a
mechanism's behaviour at its extremes will be read as describing its behaviour, and
the average is a different claim.* When a mechanism's cost is a distribution, the doc
comment owes an expectation, not a limit — which is corrections #39's standing lesson
("when a cost is a distribution distortion, measure the output fraction") arriving a
second time at a different joint.

**A tolerance is where you find out what you actually believe.** The floor failed; the
cheap move is to lower it to 0.78 and write "the mechanism changed." That number would
have been *correct* and would have asserted nothing, and the next person to change
anything would have hit it again with no way to tell a real drift from another legitimate
move. Deriving 0.70 from `9/16` and two previously measured rates took twenty minutes and
produced something the test can now *mean*: a line under which neighbouring deep cells
are no more informative than random ones. Same rule as the calibration doctrine — *a
bound with a derivation is evidence; one chosen until green is not* — applied to the
place it is easiest to skip.

**A status word that names a half reads as a whole.** "EXTRACTED" cost seven days.
`spines.md` § S-4 now says *"EXTRACTED 2026-07-22; FIRST ADOPTED 2026-07-29 at the FAR
site only"*, which is longer and cannot be misread. The general form: when a status
describes a *stage*, the stage that has not happened has to be in the same sentence, or
the sweep that reads it will stop there.

**A design pass that re-derives beats one that inherits.** The 2026-07-24 adoption plan
was reasonable and would have been wrong twice — it fused three separable kernels into
one member, and it would have taken the near site first, into machinery that does not
exist. The user's ratification qualifier for `refinement.md` (*"first members will
require their own dedicated design attention… revisited in light of this design, not
taken as wherever they landed prior"*) is the thing that caught both, and it caught them
by refusing to let a plan be inherited as a plan.

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — a type built to make a
> defect *inexpressible*, and the seven days between building it and calling it; what
> "adoption" costs when the type is right (mostly deletion) and where it is expensive
> (registration, units, and which decision is allowed to stay NEAREST). Also **lens 1
> (AI-native development)** — the acceptance criterion "goldens move" that could not fire
> because nothing in the corpus hashed the thing that changed; a status word ("EXTRACTED")
> that hid the largest unconsumed mechanism in a thread from three separate sweeps
> designed to find exactly that; and the failing tolerance that turned out to be the only
> instrument in the building that knew the mechanism was cell-wide rather than
> perimeter-wide — *the doc comment said the wrong thing, the reasoning inherited it, and
> a number caught it.*
