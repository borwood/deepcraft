# 0129 — The half of U3 that was a source, and the half that is a refactor

*2026-07-29. E5 member #0, continuation slot (b): the octaves source adopted at the
near path's member dither, plus MM-1. `docs/dependency-graph.md` E5 / P4 / P6.
The consumer half of journal/0128's pair — and **it is not the whole of slice (b)**;
§ *What did not land* says exactly what remains and why.*

---

## U3 was always two signals, and only one of them is a refactor

The near-field checkerboard has been on the board since 2026-07-24. Its two
mechanisms were both identified that same day, and they are at wildly different
scales:

1. **the 28.8 m member stepping** — the within-class member dither read a *single*
   bilinear octave at chunk wavelength, so every member patch was chunk-sized;
2. **the ~460 m record tile** — a chunk's whole strata composition is ONE point
   sample of the deep record at the chunk centre, NEAREST at the deep-cell grid.

The corpus settled (1) as the **dominant** signal on 2026-07-24 — and then
promptly lost the answer, which is corrections #73's whole story: it lived only in
a close block, the audit that asked was never stamped, a design agent re-derived
the question five days later, a main session propagated the re-derivation, and the
user's memory caught it. This slice fixes (1) and does not fix (2).

The asymmetry between them is the thing worth writing down. **(1) is a one-line
change of source; (2) is a refactor of the near path's shape.** The fix for (1) is
to replace one function's uniform with a better one. The fix for (2) is to make a
chunk-column hold *several* reconstructions and index them per voxel column, which
turns out to touch far more of the tree than anyone had counted.

## The one-line change, and the two lines around it

`dithered_member` used to read `interp_select_draw(seed, salt, tag, cx, cz, fx, fz)`
— a bilinear field over **chunk corners**, sampled at the column's fractional
position. It now reads `selection_field(seed, salt)`, which is
`draws::Octaves::member(...)`, addressed by the **absolute voxel**.

Two adjacent things had to move with it, and both are more interesting than the
swap.

**The chunk/offset address was arithmetic the caller was doing for the field.**
`interp_select_draw` took `(cx, cz, fx, fz)` because the field was *defined* over
chunk corners. A `DitherSource` is defined over world voxels, and
`Coherent::new(draws, 32).uniform(cx·32 + x, cz·32 + z, tag)` is **bit-for-bit** the
old call (pinned by a test — that equality is what lets the A/B run in one binary).
So the split address was never load-bearing; it was a leftover that invited every
caller to reconstruct it, and `surface_voxel_contents` was already
*un*-reconstructing a `(vx, vz)` it had in hand. Four call sites got shorter.

**The record's own pick had to move to the same field, or the two would stop
agreeing.** `StrataCtx::draw` samples the selection field once per chunk to choose
the event's *representative* member; `dithered_member` re-picks per column. The
doc comment on the first one says the shared field is *"what keeps the record's
representative member in agreement with the dithered footprint at its centre"* —
and that agreement is real: `mixed_at` deliberately uses the **undithered**
`e.member` for mixed voxels (re-selecting for up to eight candidates per voxel
would cost more than the rest of the fill, and it could flip a cross-class tie).
If the record picked from one field and expression from another, those two
opinions would become independent rather than merely divergent-away-from-centre.

So `ctx.draw` moved too — and in moving it got *more* exact. The old form asked
for the geometric chunk centre (`fx = fz = 0.5`); the per-column form asks for
`(x + 0.5)/32`, which is 0.515 at `x = 16`. Close, never equal. Both now name the
same **voxel address**, `cx·32 + 16`, which is the chunk-centre convention the rest
of the collapse already uses. The agreement at that column went from approximate to
exact, which nobody had noticed was approximate.

## The number that replaced the adjective

Measured at U3's reference pose (`palette_quant_tour`, extended rather than
duplicated; feet `71291.7, 372.1, −2420.9`, seed 1337, `Extent::Medium`), both
arms in **one binary** — the "before" arm is a stride-32 `Coherent`, which is
bit-for-bit the retired chunk-addressed field.

**The square-edge signature** — mean `|Δ²u|` where a three-voxel window straddles
a 28.8 m chunk line, against everywhere else:

| | on the lattice | off it | ratio |
|---|---|---|---|
| BEFORE (single octave) | 8.553e-3 | 7.119e-17 | **1.201e14** |
| AFTER (octaves) | 5.323e-4 | 4.975e-4 | **1.07** |

**The member field's autocorrelation**, `P(m(p) == m(p+lag))` over a 230 m window:

| lag | 0.9 m | 3.6 m | 14.4 m | **28.8 m** | 57.6 m | 115 m |
|---|---|---|---|---|---|---|
| BEFORE | 0.9831 | 0.9333 | 0.7688 | **0.6124** | 0.6180 | 0.5289 |
| AFTER | 0.9938 | 0.9754 | 0.9106 | **0.8445** | 0.8073 | 0.7396 |

Read the BEFORE row's middle: it falls to 0.6124 at 28.8 m and then **rises** to
0.6180 at 57.6 m. That plateau *is* the chunk — the field has one characteristic
length, it equals the chunk footprint, and past it the corner draws are
independent (the 2-member collision floor for that window's 0.662/0.338 mix is
0.553, and lag 115 m measures 0.5289). The AFTER row decays monotonically and is
still falling at 115 m: no characteristic length at 28.8 m, which is the whole
claim.

**And the mix moved, which is corrections #39 arriving at this joint.** The
majority member's share of the window fell **0.662 → 0.575**. The old source
amplified it: measured in the fixture, `Coherent` renders a recorded 0.6 as
**0.6829** while `Octaves` renders it as **0.6053**. A source biased toward the
majority was suppressing exactly the minority members this dither exists to
surface — the bias was pointing *against* the mechanism's purpose, not merely
sideways.

**One number went the "wrong" way and the caption predicted it.** The fraction of
aligned chunk footprints holding exactly one member rose **0.281 → 0.656**. That
is not a regression; it is what having power at 460 m *means* — the ground is now
genuinely uniform over more than a chunk in places, so a chunk often sits inside
one patch. Read alone it looks like more squares; read with the autocorrelation it
says the patches stopped *being* chunk-sized. The probe's own caption says so,
written before the number was known, because a printed caption is a published
claim.

## ⚠ And the corpus claim it was built on does not survive the site

The brief for this pair said, on the corpus's authority, that U3's dominant signal
is the 28.8 m member stepping — settled 2026-07-24, corrections #45 — and told me
to verify it against the site before relying on it. **It does not verify.** Three
measurements, in order of how much they hurt:

1. **The vanilla set caps the member dither at a coin flip, in 4 of 10 classes.**
   Nobody had printed this census. `clastic-fine`, `clastic-coarse`,
   `igneous-intrusive` and `igneous-extrusive` have **exactly two** members each;
   the other six — including every organic class, the ore and the accessory — have
   **one**, and a single-member class is a constant field under any source. So the
   loudest thing a member dither can ever draw is mudstone-vs-siltstone or
   granite-vs-diorite: **within-class pairs, chosen to be lithologically adjacent,
   hence of similar albedo.** The U3 observation was *"each square a distinctly
   different overall material tint — tan / grey / red-brown / dark-speckled"*.
   Those are **classes**, and no member dither can produce them.
2. **At the reference pose the dither is inert for the surface anyway.** The
   surface voxel's top span is `Mixed` for all 1024 columns of that chunk, and
   `mixed_at` uses the **undithered** `event.member` by deliberate design (the
   comment explaining why has been there since journal/0055). The topmost *event*
   is `carbonaceous-mudstone`, whose class `organic-soil` has one member. The first
   event down whose class can express anything at all is a buried sandstone — which
   is what the table above had to be measured on.
3. **My own replacement hypothesis died too, in the same run.** If not the member
   dither, the obvious candidate for a 28.8 m tint lattice is that a chunk's whole
   `StrataRec` comes from **one `run_strata` per chunk** over context sampled at the
   chunk *centre* — climate, mean elevation, flow energy, provenance — so the record
   itself could step at chunk lines independently of any dither and independently of
   the 460 m grid. Measured over an 8×8 block of chunks at the pose: **0 of 56
   horizontally adjacent chunk pairs differ in their surface-class mix**, and 0
   disagree on the dominant class. Every chunk reads `o:0.80, S:0.10`. Refuted at
   this site.

So: **the 28.8 m stepping defect was real and is now measurably gone (1.2e14 →
1.07), and whether it was ever U3's dominant signal is unsettled — the recorded
reference pose cannot settle it, because today nothing at that pose steps at 28.8
m at all.** The honest fourth possibility is the one corrections #48 exists to
warn about: **the pose is from 2026-07-24 and the world is not.** journal/0111's
1000× denudation recalibration and journal/0112's material creep both landed
after it and both rewrote what is at the surface there. A stale reference pose
producing null frames is exactly the failure that cost four frames and a confident
wrong conclusion once already.

**What this means for the walk this feeds:** do **not** judge this fix at the U3
pose. The station has to be a chunk whose surface top span is `Single` in a
two-member class — a criterion a tour map can search for in one pass and nothing
has ever searched for. That is cheap, and it is owed before any game time is spent.

## The hoist that paid for the ladder

The octaves source costs six times the hashes of the single octave it replaced —
24 corner draws against 4 — and `dithered_member` is on the **chunk-load path**,
which runtime is sacred forbids growing. So the slice includes the perf half.

`dithered_member` is a pure function of `(event, voxel column)`. The `y` loop
cannot change its answer. It was nevertheless being called **per voxel**: up to
32 768 times in `generate_chunk` and again in `material_ids`, for every buried
voxel whose span resolves to a single event. Hoisting it to once per
`(column, event)` — a small `Vec<Option<GeoMemberIdx>>` cleared per column — is
~32× fewer calls for a handful of distinct events per column.

The arithmetic, which is the honest form of this measurement because the two arms
cannot both be production: before, one call per buried single-plan **voxel**, in
each of two paths — up to 2 × 32 768 per chunk, at 4 corner hashes each. After, one
call per `(column, distinct event touched)` — at most 2 × 1024 × (events a 32-voxel
column crosses, a handful), at 24 corner hashes each. The hash count therefore
*falls* for any column crossing fewer than eight events, which is every column in
the shipped world.

Measured on the chunk-generating suite, same machine, same day:
**`contents_contract` 103.72 s before → 102.43 s after.** That suite is
**240 `generate_chunk_with_materials` calls** (80 sampled chunk positions × 3
worlds) on top of three `Pregen::run`s, so roughly two thirds of its wall clock is
chunk generation — sensitive enough to see a real regression in a per-voxel call.
**−1.2 %, i.e. no measurable cost.** Honest caveat: the "before" ran inside a full
workspace test where sibling suites competed for cores and the "after" ran
standalone, so the comparison bounds the change at a few per cent rather than at
one. It does not resolve a 1 % effect and does not need to; a 6× per-call cost
without the hoist would have been visible.

That is the shape the design bones already asked for, arriving at a different
joint: **the expensive thing runs at the granularity its answer changes at, and
only a cheap index is per-voxel.** The restructure that did not land says the same
sentence about `run_strata`.

## MM-1: the membership dither was welded shut

`CoarseField::sample_dithered` does two things — dither *which* of the four
surrounding cells supplies the shares (the cake law), then draw a class from them.
The near path needs only the first half, over a payload that is not a share vector
at all (*which deep cell's record skins this column*), and until now step 1 was
unreachable. That is the design pass's MM-1, and it is discharged:
`sample_source_cell(pos, src, salt) -> Option<&T>`, with `sample_dithered` now
**written in terms of it** — so there is one membership dither in the tree rather
than two, and the new call has a production caller from the hour it landed rather
than waiting seven days for one.

Two things came with it that were owed and missing.

**Its law had never been tested.** The claim the whole cake law rests on is *the
probability a position reads cell k is cell k's bilinear weight there*. That is the
conservation statement any Law-3 argument about redistribution needs, and there was
no test of it — `cake_law_a_minority_phase_interfingers_across_a_cell_boundary`
tests the *consequence* (minorities cross the line), not the *rate*. Now: the home
cell wins **9/16 of its own cell's positions** — the exact integral journal/0125 had
to do to discover that this is a cell-wide blend rather than a perimeter treatment —
and at a cell corner all four cells answer a quarter of the time each while the
five cells outside the stencil answer **never**. Both bounds are 4σ binomials on
derived means, not fitted tolerances.

**The source has to be white for that test to mean anything**, and saying why is
the part I would want a future reader to have. A coherent source is correlated in
space, so a *finite window* of it does not realise its own expectation — measure
the draw's unbiasedness through a coherent source and you are measuring the
source's autocorrelation. Unbiasedness of the **draw** belongs in dc-core with
white noise; unbiasedness of a **source's marginal** belongs to that source
(`Octaves` has one; `Coherent` deliberately fails it — corrections #39). Splitting
the two is why neither test lies.

**And the call carries a named tension.** `sample_source_cell` returns a *cell's
payload*, which is one refactor from the raw per-cell read the whole type exists to
forbid. The distinguishing property is that **the caller cannot choose the cell** —
it names a position and receives whichever cell the seeded draw picked. The
`compile_fail` proof cannot see that difference, so this is a doctrine boundary
rather than a structural one, and the warning now lives at the call rather than in
an audit that predicted it.

## What did NOT land, and the number that decided it

**The near-path record restructure — U3's 460 m half — is not in this slice.** The
design pass scoped its blast radius as *"`column()`, `ColumnRec`, `material_ids`,
`mixed_at`, `surface_voxel_contents`, `generate_chunk*` — every golden moves."*
Measured before starting: **13 files and ~40 call sites**, because
`ColumnRec.strata` is a `pub` field of a `pub` struct and **eight example probes
and five test files read it directly** as *the* record of a chunk — a thing that
stops existing when a chunk-column holds up to nine of them. The design pass
enumerated the *generation path* and generalised to the tree, which is the
corrections #64 mechanism at a smaller scale.

That is not an argument against doing it; it is the reason it is a slice of its own
rather than the tail of this one. Taking it here would have meant a half-refactored
tree at the end of a session, and the dominant signal — the one the user is
looking at — would have shipped inside it instead of on its own.

Two things about it are now *cheaper* than they were this morning, and one is
newly known:

- **MM-1 is discharged** (above), with a caller.
- **MM-3 — the working sub-cell state's declared type — is designed and in the
  report, not in the tree.** It was written, and then deliberately withdrawn: a
  declared type with no consumer is `CoarseField` on 2026-07-22, which is the
  precise failure this whole arc exists to teach. It ships **with** its
  restructure or not at all. Its judgment calls (does the type own the height
  field? does `ColumnFill` travel inside it? — yes, and that is what makes
  slicing record A with fill B a type error) are for the ratification conversation.
- **The boundary-chunk cost question is moot for this slice and belongs to that
  one.** The design pass predicted "up to 9× `run_strata` on boundary chunks,
  unmeasured". It is unmeasured *because the record is still one per chunk*. What
  the restructure will actually pay is worse than "boundary chunks", and the
  arithmetic is worth recording before someone measures the wrong thing: the
  bilinear stencil is **always** 2×2, so a chunk anywhere in a cell's interior
  touches **four** cells, not one — and with 1024 columns drawing, even a weight of
  0.001 is realised somewhere in the chunk. **Typically 4, up to 9, and ~1 only
  within half a metre of a cell-centre line.** The design pass's *"≤ 9, typically
  1"* reads the stencil as "cells the chunk overlaps"; it is the same *"one weight
  ≈ 1 away from a boundary"* misreading that cost journal/0125 a failing gate, in
  its third outfit.

Also not fixed, and neither is a consequence of this slice:

- **`dithered_member`'s formation context is still the chunk's.** The `u` is
  multi-scale now, but `event.temp_c` / `precip` / `depth_m` were recorded once per
  chunk at the chunk centre, so the *fitness landscape* the draw indexes into still
  steps at 28.8 m even though the draw does not. That is U22's named sibling — the
  member-dither guillotine, unexamined since 2026-07-22. A step in the thresholds
  is a much weaker signal than a step in the draw (class fitnesses vary slowly with
  climate; the draw is what the eye reads as a patch), but it is not zero.
- **The FAR summary's member dither is still the single octave**, deliberately. It
  is the same defect at the other tier, and the far register's blend semantics were
  **rejected by the user** the same evening (*"the whole cake is swirled now"*);
  changing the appearance of a register that is being re-asked would answer a
  question nobody asked. Annotated at the call site and listed in § Observed rather
  than left as an oversight.
- **The far field's "swirled cake"** is a different mechanism with a different heir
  (a far register derived from refinement-operator budgets), and nothing here
  touches it.

## Goldens

Two of `contents_contract`'s three triples moved; the third is byte-identical.

| world | blocks | materials | table |
|---|---|---|---|
| medium `0x0D5EED572026` | `0xBA49…C724` → `0x7F53…FD72` | `0x9BB3…9519` → `0xB1A8…FF92` | `0x97C6…F29C` → `0xDC14…9099` |
| medium `0x539` (1337) | `0x1CF9…1CB8` → `0x6594…4D07` | `0x54F0…92A0` → `0x2687…BD0E` | `0xB89B…485D` → `0x10F4…B531` |
| small `0xC11A7E2026` | **unmoved** | **unmoved** | **unmoved** |

The mechanism, recorded in the test file beside the constants rather than only
here: the within-class member dither changed source, and `StrataCtx::draw` — the
record's own representative pick — moved to the same field at the same voxel
address. Both move which *member* of a class a voxel shows; neither can move its
*class*, which is why `coarse_surface_agrees_with_the_near_column_surface` (a
class-granularity comparison) does not budge.

**The Small row holding is a prediction of this file's own header coming true**:
its sampled chunks carry no strata record, so there is no member to dither — the
same structural reason it held for MFD, movement 2b, hybrid `p` and creep. A row
that keeps not moving for a stated reason is worth more than a row that moves.

No other golden in the workspace moved: `GOLDEN_FAR_SURFACE` reads the far tier
(deliberately unconverted, above), and the deep-time fingerprints
(`providers_golden`, `rate_axis`, `creep_operator`, `GOLDEN_GEOTHERM`,
`GOLDEN_HEAD`, `GOLDEN_CHAPTERS`) sit upstream of the collapse tier entirely.

## The shape worth keeping

**A defect with two signals needs two verdicts, and the cheap one is not always the
important one.** U3's dominant signal was a *source*, and swapping a source is
hours; its subordinate signal is a *shape*, and changing a shape is a week. The
corpus knew which was dominant on 2026-07-24 and lost the note, so five days of
planning treated them as one item called "the near path". **When one symptom has
two mechanisms, the estimate that matters is per-mechanism, and the record of which
dominates has to be stamped on the document that asks — not left in a handoff.**

**A blast radius measured from the call graph is a lower bound.** The design pass
listed six generation-path functions; the real count was 13 files, and the extra
eight were *probes and tests reading a `pub` field*. Consumers of a type's shape
are not visible in a trace of the code that produces it. **Grep the field, not the
function.**

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — one visible defect,
> two mechanisms at scales 16× apart, one of them a one-line source swap and the
> other a week-long refactor of who owns a record; and the discipline of shipping
> the first alone rather than letting it ride inside the second. Also **lens 1
> (AI-native development)** — a designed type deliberately *withdrawn* from a
> commit because its consumer slipped, since a declared type with no caller is the
> exact failure the same arc spent seven days paying for; and a design pass's blast
> radius that was a lower bound because it traced functions where the coupling was
> a public field.
