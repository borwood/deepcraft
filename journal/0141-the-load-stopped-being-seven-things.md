# The load stopped being seven things

*P11 slice 2 — the conversion. The four budget planes go CSR-sparse over
`MaterialId`, the erosion rate stops being a fact about a class, and the
deposition draw is retired for anything a mover carried.*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — the slice is a
> worked example of *"a seam's success condition is that it disappears"* and of
> its opposite failure, a seam the production path quietly stops reading.
> Also **lens 2 (procgen against the priors)**: the choice not to widen a dense
> plane is the whole slice, and the reason is an asymptote, not a megabyte count.

---

## What was wrong, stated as narrowly as it deserves

Slice 1 gave the deep record an identity: `DepUnit::species` became a registry
`MaterialId` and the member was chosen by fitness at deposition, under the
formation context of the epoch that laid the bed. That was the whole of slice 1
and it worked.

It also made the *rest* of the tier the coarse link. Four planes carried the
solve's material arithmetic —

| plane | what it is |
|---|---|
| `shares` | the composition of the loose cover at each cell |
| `qs_sp` | the suspended load in flight |
| `dep_sp` | what the flow set down |
| `creep_sp` | what hillslope creep delivered |

— and every one of them was `n × Litho::COUNT`: **seven classes**, dense. So a
siltstone bed eroded at mudstone's rate, because the table that answered was
keyed by class. The record knew which rock it was; nothing downstream could ask.

The obvious move is to widen the planes to the registry. It is the wrong
asymptote, and the user ruled it out before it could be written: *"transport
planes go **SPARSE DAY ONE**"* (ruling 3, 2026-08-01). Dense scales as
`cells × registry`, and the registry is open by design — a plugin-first engine
whose material count is a *pack's* choice cannot put that count in a per-cell
array. Sparse scales as `cells × local presence`, and the claim underneath that
is a claim about the world: **a cell's in-transit load only ever holds its
catchment's species, never the world's.**

Nobody had measured it. The foundation slice did (journal/0138): on the shipped
world, `p = 3.719` species per cell mean, 8 max, on a 14-material axis.

## The three layouts, and why the layout is exact rather than a guess

A CSR plane needs a row index, and the interesting question is where the rows
come from. There are three, and each answers a different question:

- **window** — what lies *at* a cell (the near-surface `OUTCROP_DOMINANCE_WINDOW_M`);
- **transport** — the window closed **downstream along the solve's own routing**;
- **creep** — the window dilated by one 4-neighbourhood ring, because creep moves
  the *donor's* composition across an edge.

The transport layout is the one that could have been a heuristic and is not. It
is built by asking the transport pass for its own processing order and its own
out-edges — `Erosion::processing_order` and `Erosion::out_edges`, which exist for
exactly this reason — and propagating presence bits along them in the same
direction the load is about to move. So the row index *is* the transport graph's
reachability, computed with `u64` ORs where the pass will compute `f64` adds. A
species cannot arrive at a cell whose row has no slot for it, not because we
sized generously, but because the two walks are the same walk.

That is also why `slot_of(j, k).expect(…)` is honest rather than optimistic. The
`expect` is a statement about the construction, and if it ever fires it means the
closure and the chain have diverged — which is the only interesting failure.

## The thing that nearly went wrong: the order

Two loops in the transport pass are order-sensitive. The capacity drawdown is
*coarsest first* — the excess a flow cannot hold is paid out of the heaviest
fraction it is carrying, which is why a bar is gravel and sand rather than an
average of everything in the water. The competence ceiling is the same shape from
the other side.

Under a dense plane those loops read a precomputed permutation, `ws_order`. Under
a sparse row, walking a permutation of the whole axis and testing membership
would put the registry's width straight back into the hot loop — the exact
asymptote sparsity exists to remove.

The foundation's answer is to move the ordering into the **axis** rather than into
a loop: the axis is sorted by descending settling energy, ties broken by
`MaterialId`, and a row is stored ascending in axis order. So walking a row front
to back *is* coarsest-first, over exactly the species that cell holds, and the
competence sweep can `break` the moment it meets one under the ceiling.

The unlooked-for benefit is that it gives every order-sensitive rule in the slice
**one** total order: the residual split's *"last non-zero share takes the
remainder"*, the arriving-identity argmax's tie-break, and the two drawdown loops
all break ties the same content-derived way instead of by an enum's declaration
order. When we wrote the tie rule into the arriving-identity argmax's doc comment
it did not need a new sentence; it needed a pointer.

## The seam that nearly got left behind

Erosion does not walk the record for its rates. It asks the `outcrop_shares`
provider seam — a named socket whose heir is structural deformation, so that when
beds dip, the *dipped* shares arrive and the rate field dips with them.

The seam answered per `Litho`. The rate table went per material. Both of those
sentences can be true at once only if the rate path stops reading the seam — and
for about an hour, that is what the conversion did: `expose` called
`lithology::exposed_member_shares` directly.

Nothing failed. The tests were green, the world eroded, and the seam still had
consumers. What had happened is that the socket the layer-cake term is supposed to
plug into had quietly become a socket for a *different, smaller* question: erosion
would have kept blending its rates from a walk the heir could never replace.

The tell was a test. `full_agents.rs::waves_cut_down_the_coastline` drives a soft
and a resistant coast **through** the seam, because the shipped world's coasts are
basement-heavy and the mechanism question is not a question about this world's
composition. Under the bypass, that test's override became inert — the run would
still have gone green, because the assertion is differential and both arms would
have moved together. A test that stops testing is not a red.

So the seam was re-signatured to member grade:

```rust
pub outcrop_shares: Option<fn(&SpeciesAxis, &[DepUnit], &mut [f64])>,
```

and the class-grade window walk stayed where it was, for the consumers that
genuinely want a class — the far tier, the outcrop verdict, and the degenerate
no-content door. **A seam's success condition is that it disappears** (providers.rs
says so in as many words); its failure condition is that it *survives with the
wrong customer*, and that is much harder to see.

## The draw is retired, for the deposits that never needed it

Ruling 6, the day before this slice: *"MaterialId — not membership of a group — is
the basic unit of deeptime: identity is a conserved quantity flowing through the
mass arithmetic."*

The deposition-time fitness draw existed to **fill a class**. Erosion released a
metre of *fine clastic*; transport carried a metre of *fine clastic*; deposition
had to decide which fine clastic, and the only witness available was the climate
at the site. That is a real answer to a real hole, and it is also a hole that
member-grade transport closes: the mover now carries `dc:siltstone`, not
`clastic-fine`, and there is nothing left to pick.

So the record's identity rule became:

1. sum what the two identity-carrying movers delivered (fluvial `dep_sp`,
   hillslope `creep_sp`), positive contributions only;
2. compare the biggest arrival against the **un-carried remainder** — the metres
   that were made here or brought by a mover with no identity yet;
3. if an arrival wins, **that is the rock**, no draw;
4. if the remainder wins, or if deposition genuinely transforms the rock, draw.

Step 4's second clause is the part worth defending. Basement a river quarried
lands as coarse clastic detritus — a gravel, not a granite — and detrital
peat/coal/charcoal land as carbonaceous mud. Those are the `as_deposited` edges
journal/0112 found the hard way, when creep carried thin charcoal beds downslope
and won a cell's argmax with them. On a transformation edge the *parent* does not
determine which member of the destination is produced; the conditions at the site
do. So fitness runs, on the destination class, and ruling 6's *"transformation
edges under declared conditions, drawn coherently"* is what that clause is.

### Measuring a retirement is harder than performing one

"The code path is taken" proves nothing. The claim that matters is: **of the
metres whose identity came from the arriving composition, how many would the
deposition site's own climate have named differently?** Those are the metres whose
rock is a fact about their *source*.

Answering it means evaluating exactly the draw the slice exists to stop
evaluating, per depositing cell per epoch — which is why it is an instrument
(`DeepConfig::identity_audit`), off in production, asserted bit-inert, and read by
`member_diversity_probe`. If that number came back near zero the ruling would have
bought correctness of principle with no expression, and saying so would have been
the honest report.

**MEASURED (seed 1337, Medium, 404,202 m of record):** **77.21 %** of the record's
metres take their identity from the arriving composition and never reach a draw.
Of those, **41.31 %** — 209,670 m, **31.90 % of the whole record** — would have
been named a *different rock* by the deposition site's own climate. So it is not a
correctness-of-principle change: nearly a third of the world's recorded rock now
says something about where it came from that the place it landed would have
denied.

The number also explains the residency. Member-grade transport genuinely
diversifies what gets written — a mover delivers siltstone where the draw would
have said mudstone — so `deposit_as`'s merge key splits more often: the split
factor went **1.2775× → 1.9064×** (7,622,541 units against 3,998,428 under the
pre-P11 class-only key), and `Pregen::approx_resident_bytes` with it
(357,154,365 → 403,151,293, **+43.87 MiB**). The record got bigger because it got
more honest, which is the trade this arc has been making since slice 1.

## The salt that was two decisions

The 2026-08-02 spine-audit found `refine.rs`'s hand-rolled
`SALT_DT_PERTURB = 0x5900_0002` sitting on `draws.rs`'s `DeepMember` domain. Two
live decisions, one stream.

The `draw_domains!` macro exists precisely to make this a compile error, and it
could not see this one, **because only one of the two was in the list**. That is
the same shape as the defect the macro was cut for (`pore_rider_share` slicing bits
out of a neighbouring draw with a comment claiming they were disjoint): a claim
about hash bands that no gate can check.

The fix names both halves. `DeepTimePerturb` is registered at the value it already
had, so the refinement experiment's decay profile — a dated measurement — is
unchanged to the bit, asserted rather than argued
(`the_registered_perturbation_domain_is_the_hand_rolled_salt`). `DeepMember` took
the next unused value, because it is production identity and the golden re-capture
was going to move it anyway.

## What moved, and which movements are semantics

Every deep-time golden. Three separate mechanisms, and the distinction matters
because corrections #89 is live here:

1. **semantics** — the erosion rate is now a function of the rock. Mudstone and
   siltstone have different `smash` sheets, so a window that used to blend one
   rate blends two;
2. **semantics** — transported deposits take their identity from the load rather
   than from a draw, which changes what gets recorded and therefore what the next
   epoch's window holds;
3. **float accumulation** — the CSR row visits the same species in the same order
   as the dense plane did, but the *rows differ per cell*, so a sum that used to
   run over seven slots (five of them zero) now runs over three. `a + 0 + b` and
   `a + b` are the same number; `(a + b) + c` and `a + (b + c)` are not, and the
   drawdown loop's early `break` changes where the partial sums land.

Under the scratch-pad doctrine that is a re-capture with the why recorded, not a
ratification. The `_off_is_the_pre_slice_world` arms moved too, and they had to:
those constants pin *"the material tier off is the world before the tier existed"*,
and the world before the tier existed is now a world whose rates are member-grade.

## The one real red

`exchange_cell` read the transport row unconditionally. On the scalar-load solve
every CSR structure is deliberately empty — that emptiness is what makes the
off-path byte-identical — so the row read is an out-of-bounds on any harness that
drives `Erosion` directly without turning the material tier on. Three tests caught
it (`mass_is_conserved_up_to_uplift`, `mass_is_conserved_with_coupling_on`,
`the_differential_erosion_feedback_neither_runs_away_nor_stalls`), and all three
are tests that exist for a *different* reason. The lesson is the ordinary one: the
identity path and the live path have to be guarded at every site that reads a
structure the identity path does not build, and "the flag is off" is not a guard
unless it is written down.

---

## The costs, measured

**The four budget planes, on the real solve** (not on a model of it — the probe
reads `Erosion::species_layouts` now):

| | MiB |
|---|---|
| class-grade, dense, what shipped | 63.45 |
| **member-grade, dense** — the wrong asymptote | 126.90 |
| **member-grade, CSR-sparse** — ruling 3 | **44.67** |

`p = 3.721` mean species per cell, max 8, on a 14-material axis: the foundation's
blind prediction was 3.719 / 8, and its 44.98 MiB estimate lands within 0.7 % of
the 44.67 MiB the solve actually pays. **Sparse member grade is 0.70× the cost of
the class grade it replaces**, while carrying twice the identity.

**Gen time is where the bill landed.** Medium pregen, three uncontended samples:
**50.26 / 50.23 / 50.55 s** against main `5914e708`'s 41.8–42.2 s — **+19.4 %**.
The first measurement of this slice was **60.3 s**, and the extra 10 s was one
loop: `expose`'s window walk had gone sequential in the conversion, on a phase
that runs `n` times an epoch and had been data-parallel since S9b. Restoring the
parallel driver (per-cell, disjoint writes, byte-identical by construction, and
the goldens confirm it) recovered all of it. `pregen_time_vs_extent`'s ratified
60 s budget was **not moved**; it is passing at 50.3 s with ~16 % headroom, where
it had ~30 % before.

**And the pregen budget is where this slice has to stop and ask.** The
`pregen_time_vs_extent` assertion is a ratified 60 s, and it passes uncontended
(50.3 s) and **fails inside the full workspace gate at 60.68 s**, where its
sibling test builds its own Medium world on the same machine. That is the same
shape ROADMAP recorded for slice 1 (62.7 s contended against 47.5 s uncontended),
and it was resolved there by a **design change** — the `(cell, chapter)` draw
address — rather than by moving the number. Moving a gate to admit one's own work
is not a fix, so it is not moved here either. What is left on the table, unpriced:
`expose` still walks each cell's window **twice** an epoch (once to derive the
presence mask, once to scatter the shares into the rows that mask sized), and a
transient dense scratch would buy the second walk back for ~33 MiB of gen-time
memory. That is a real option and it is a user call, because it is spending the
asymptote ruling 3 was cut to protect — even transiently, even in scratch.

**Law 3 closes tighter than the conversion could have loosened it**, measured
after the CSR change rather than assumed:

| instrument | measured | bound |
|---|---|---|
| `max_species_split_residue` | 2.175e-16 | < 1e-12 |
| `max_creep_itemisation_residue` | 1.352e-15 | traffic-relative |
| `max_creep_conservation_residue` | 9.908e-16 | 0 to round-off |

All three are **at or within a few ULP of exact**, which is the point: the
residual rule survived the shape change because it moved with the arithmetic
rather than beside it. `split_row_into` is one function and entrainment, incision
and creep all route through it, so no caller can invent its own budget — the same
guarantee `split_by_shares` gave, over a different index.

---

*Merge postscript, stamped 2026-08-03 (doc-topology F1 — this entry was the one
arc journal of three without one).* The § 11 claim above that the ratified 60 s
pregen budget *"was **not moved**… moving a gate to admit one's own work is not a
fix, so it is not moved here either"* was **overruled by the user at this very
slice's merge**: the budget was renegotiated 60 s → **1,200 s**
(`tests/s7_measurements.rs` — *"Prior value, kept for audit: 60_000.0"*; the
user's words at the record: *"as long as it doesn't take 20min"*). The claim was
honest when written; the author's principle — that the author must not move the
gate — is exactly why the move belonged to the user, who then made it.

*Count note, 2026-08-04 (S0):* this entry's 7,622,541 units (slice 2b) is the missing
middle figure between 0136's 10.95 M and today's **7,304,581** — it dates the big drop
to slice 2's identity rederivation, not slice 3's carry. See
`2026-08-04-s0-correlation-measurements.md` § g.
