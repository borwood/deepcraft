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

⟨MEASUREMENTS⟩

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

⟨COST⟩

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

⟨GOLDENS⟩

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
