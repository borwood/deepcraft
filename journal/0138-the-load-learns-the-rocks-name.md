# P11 slice 2 — the load learns the rock's name (FOUNDATION ONLY; the slice is NOT built)

> **⚠ READ THIS FIRST. This entry does not describe a shipped slice.** The brief was
> the whole of P11 build-sequence item 2 — the four budget planes CSR-sparse over
> `MaterialId`, the composition term folded in, the deposition draw retired for
> transported deposits, one golden re-capture. **What landed is the foundation and
> the measurement**: the species axis, the CSR layout machinery with its tests, the
> member-grade window walk, the member-grade susceptibility table, the routing
> accessors an instrument needs, and a gated probe that answers the one number the
> design priced blind. **The solver conversion, the composition term, the draw
> retirement and the golden capture did NOT happen.** The reason is in
> § *What stopped, and why it stopped there*, and it is a resourcing fact, not a
> discovery about the design.

> blogworthy: **AI-native development** — an agent that measured its own remaining
> budget mid-refactor, reverted a half-finished surgery rather than hand over a tree
> that would not compile, and re-spent the remainder on the one measurement the next
> attempt cannot start without. Also **reflexions in a deepsim codebase**: the
> asymptote argument for sparsity was a *claim about a number nobody had*, and
> getting the number turned out to be separable from the change it justifies.

## The wall the slice exists to remove

Slice 1 (journal/0136) put a registry `MaterialId` in `DepUnit::species`. It said so
plainly in its own § *"The scaffolding, named out loud"*:

> The transport arithmetic is still `Litho::COUNT`-wide: four `n × SPECIES` budget
> planes, the susceptibility and settling tables, `WindowShares = ShareVec<7>`. A
> `MaterialId`-grade record has to be bucketed back down before those tables can
> index it. So `Litho::of_material` exists…

That bucket is why **a siltstone bed still erodes at mudstone's rate**. The record
knows which rock it is; every table that decides how fast it wears away is keyed by
the class. The load is now the coarse link in a chain whose two ends are both
member-grade.

Widening the four planes to the registry is the obvious move and it is the wrong
one. The user's ruling was verbatim and it named the reason: *dense scales as cells ×
registry-size*, which is the wrong asymptote for an engine whose material registry is
open by design. **Sparse day one.**

## The claim underneath the ruling, and the fact that nobody had measured it

*"A cell's in-transit load only holds its catchment's species."* That is a claim
about a **number** — how many species a deep cell's load actually carries — and the
corpus had priced four different storage shapes against it without ever measuring it.
The P11 design audit filed it as **I3** and its § 9 item 5 is explicit:

> ⚠ **The per-face species sparsity is unmeasured.** § 3.4's dense-share pricing may
> be a large overestimate.

The record-terms priors (§ 4.4) built a whole cost table — *"parallel CSR over loaded
entries, 4.43 MiB"*, *"widen `FluxEntry`, 19.76 MiB"* — over a **guessed** width of
7, and the P11 audit then re-derived the same table at 14 and 26 without narrowing
the guess. Three documents, one unmeasured quantity, and the ranking they all produce
depends on it.

So the foundation this entry describes was built around getting that number.

## The mechanism: a presence mask, and a closure that is the transport graph

`SpeciesLayout` is one epoch's answer to *which species can be at which cell*. Per
cell it is a `u64` presence mask over the axis; the CSR row offset is the running
popcount, and each row's axis indices are spelled out so iteration is a slice read
rather than a bit decode. A slot lookup is

```rust
Some(self.start[c] as usize + (m & (bit - 1)).count_ones() as usize)
```

— an AND, a popcount and an add, with no search and no per-cell allocation. That last
clause is not incidental: S19 § 6 measured a `Vec`-per-cell shape at *"89 % of its
150 MiB heap in empty headers"*, and the CSR is the shape `flux.rs` and the
`FactLedger` conversion (journal/0102) already vindicated. **The engineering question
was never "is sparse cheaper" — it was "which sparse", and the corpus had already
answered that twice.**

The part worth stopping on is how the layout is *built*, because the naive answer is a
heuristic and this one is exact.

A cell's mask starts as **what lies at that cell**: the materials in its near-surface
window, plus the bedrock beneath it (incision detaches material from below the
record, so basement can enter the load anywhere the flow cuts rock). Then the masks
are **propagated downstream along the solve's own routing, in the solve's own
order** — highest cell first, `mask[receiver] |= mask[cell]`. That sweep is the
transport pass's reachability computed with `u64` ORs instead of `f64` adds, so a
species can never arrive at a cell whose row has no slot for it. The layout is not a
guess with a safety margin; it is the same graph, walked the same way.

Hillslope creep gets its own layout — one 4-neighbour dilation of the window masks,
because creep moves the *donor's* composition across an edge — and the window
composition gets a third, tight one.

## The order of the axis, which is the least obvious decision in the file

Two loops in the transport pass are order-sensitive: the capacity drawdown
(*"coarsest first"* — the excess a flow cannot hold is paid out of the heaviest
fraction it is carrying) and the competence ceiling. Under a dense plane they read a
precomputed permutation of the whole roster. Under a sparse row, walking a
permutation of the whole roster and testing membership would put the registry's width
**straight back into the hot loop** — which is exactly the asymptote sparsity exists
to remove. The saving would have been in memory only, and paid for in time.

So the **axis itself is ordered by descending settling energy**, `MaterialId` index as
the tie-break. A row is stored in ascending axis order, which *is* coarsest-first, so
both loops walk the row and stop early. It also hands every order-sensitive rule in
the pass — the residual split's *"the last non-zero share takes the remainder"*, the
arriving-identity argmax's tie-break — **one total, deterministic, content-derived
order**, instead of the declaration order of an enum that is scheduled for demolition.

And the axis is **derived from the registered content**, never declared: every
material some geology member deposits, plus the basement. No class view is consulted
to build it, which is ruling 2's A-CLEAN read taken literally — a pack that registers
a member widens the axis by construction and nothing else has to know.

## The anchor that had to be named

`susceptibility_table` expresses every rock's rate relative to a reference, and the
reference was `REFERENCE_LITHO` — *fine clastic*. Under member grade there is no
"fine clastic" to be relative to. The design audit's § 6b named this as an obligation
and did not discharge it: *"the anchor member must be **named** and its property sheet
cited. This is a derivation, not a re-tune — but it must be re-stated, or the constant
becomes a number pretending to be a mechanism."*

It is now `REFERENCE_MATERIAL: MaterialId = MaterialId::MUDSTONE`, which is
`Litho::ClasticFine.reference_material()` spelled out. **The value does not move.**
That is the whole content of the change and it is worth the line: the same number,
sourced from a rock instead of from a bucket, is the difference between a calibration
and a coincidence.

(The sibling obligation — `COMPETENCE_SCALE = 420`'s anchor, *"coarse clastic's
`settle_energy` at the Low/Medium boundary"*, which is `dc:sandstone`'s sheet at
≈0.84 — is **not** discharged here, because the constant lives in the solver this
work did not convert. It is named in the report as owed.)

## What stopped, and why it stopped there

The conversion of `erosion.rs` was started and **reverted**. That file is 4,679 lines
and the species axis reaches roughly forty sites in it — `exchange_cell`,
`settle_above_competence`, the two transport chains, the MFD per-species residual
split, `diffuse_species_cell`, `audit_creep_species`, `arriving_species`, `expose`,
the four plane allocations — before the ripple reaches `lithology.rs`'s
`WindowShares`, the `outcrop_shares` provider signature, and roughly ten test and
probe files that construct one.

Halfway through that, the honest arithmetic on the agent's remaining context said it
would run out **during the compile-fix cycles**, leaving a tree that does not build. A
half-converted solver is worth strictly less than nothing: it cannot be gated, it
cannot be measured, and the next session pays to read it before it can pay to finish
it. So it was reverted to `HEAD`, and the remaining budget went to the pieces that
stand alone and to the measurement.

**The lesson generalises and is worth keeping.** The instinct in a large refactor is
that partial progress is progress. For a *compiled* artifact under a gate it is not —
the unit of value is "compiles and passes", and anything short of it is a liability
the next reader inherits. The right move when the budget will not cover the whole
surgery is to find the **separable** part, which here was the one that answers a
question rather than the one that changes a behaviour.

## The measurement, and why it was separable at all

The sparsity number does not require the converted solver. It requires the final
record (which already names `MaterialId`s — slice 1 shipped that), the solve's own
routing (which `Erosion` holds at the end of a run), and the closure. So
`examples/species_sparsity_probe.rs` builds a production world, walks every cell's
window at member grade, seeds the masks, runs the closure over
`Erosion::processing_order()` / `Erosion::out_edges()`, and reports the row-width
distribution of all three layouts.

Two accessors were added to `Erosion` for it rather than re-deriving D8 inside the
probe. That is deliberate: a probe with its own routing rule is a second authority
that can silently disagree with the one the world was built by, and the corpus has
enough of those.

**The honest bounds on the number, stated in the probe's own module docs so they
travel with it:** it is **one epoch — the last one** — not a run average; and it
measures the **layout** (what can be present), not the non-zeros (what is). The second
is the right thing to size a CSR against, because the row has to exist before the
solve can put anything in it. The first is a real limitation: an early epoch has a
thinner record, so more of its window is basement deficit and its rows are *narrower*.
The measurement is therefore a late-run, mature-record reading — the regime the cost
question is about, and not a claim about the mean.

## What the transport row width IS

It is the per-face species count, and the argument is short enough to state rather
than assert. The MFD split offers **every species in the load to every weighted
face** — that is the per-species residual rule journal/0110 had to state twice, and it
is why no face fractionates the multiset. So a face's composition is the cell's load
composition restricted to what is non-zero, and the cell's row width is an exact upper
bound on it. The number the composition term is priced against and the number the
budget planes are priced against are **the same number**, which is not obvious until
the split rule is read.

## Remaining, in the order the next attempt should take it

1. The solver conversion (`erosion.rs`, `lithology.rs`'s `WindowShares`, the
   `outcrop_shares` seam, the tests and probes that construct one). Design and site
   list are in the slice report; nothing about it is unresolved, it is only large.
   **`erosion.rs` at 4,679 lines is 6.7× the file-size threshold and the hook says so
   on every edit — the extraction is a user-owned proposal, and it would make this
   conversion materially cheaper.**
2. The composition term into `FluxAccum` / `FluxRecord`. The **scratch**, not the
   record, is the unpriced part — a dense per-chapter `n × FACE_SLOTS × axis` scratch
   is ~85–140 MiB of gen memory, which is not affordable. The design that survives is
   a compacted face index (`n × FACE_SLOTS` of `i32`, `-1` = none) into a growing
   per-chapter composition vector, so the scratch scales with *loaded* faces rather
   than with all of them.
3. The draw retirement for transported deposits, with the tie rule documented in code,
   and `as_deposited`'s transformation edges kept as the genuine-degeneracy remainder
   that still draws.
4. The single golden capture, after (3), covering the families the ROADMAP arc lists
   as expected-red.

---

## Merge postscript (integrator, 2026-08-02)

The builder never got a cargo invocation — the build slot was contended by the
parallel bodies session for its entire run — so the branch merged with integrator
verification instead: nine `MaterialId as usize/u8` casts fixed via `.raw()` (the
newtype has no primitive cast), two `WorldParams::default()` spreads removed (the
struct has two fields, both set), rustfmt applied. Crate-scoped clippy `-D warnings`
green from this worktree's path; the foundation's own tests green **by name** (6
species-axis unit tests + the creep-absence pair, 4 probe gates, `0 failed` both).

**The number the design priced blind, measured (seed 1337, Medium, final epoch):**
the deep species axis is **14 materials**; per-face species count **p = 3.719 mean,
max 8** — the load is 3.8× sparser than its axis. The four budget planes at that
sparsity: class-grade dense (shipped) 63.45 MiB · member-grade dense 126.90 MiB (the
wrong asymptote, confirmed) · **member-grade CSR 44.98 MiB — 0.71× the shipped
cost**. Ruling 3's "sparse day one" is now measurement-backed: the member-grade
upgrade is a residency *reduction*.

Journal ordinal 0138 and stub #36 assigned at merge per the slug-only protocol
(bodies session live; tails checked at assignment). The solver conversion — the
actual slice 2 — follows the `erosion.rs` concern-split (user, 2026-08-02: "we
split it now").
