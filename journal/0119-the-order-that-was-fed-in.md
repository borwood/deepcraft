# 0119 — The order that was fed in

*2026-07-26 · a design pass on the pass architecture — opened by a question about
weathering, closed by the user finding their own design in the corpus*

> blogworthy: **lens 1 (AI-native development)** and **lens 3 (reflexions in a deepsim
> codebase)**. A user asks why weathering lowers height. It doesn't. Four hours later the
> project has retired the mechanism by which its scheduler decides what runs when, and
> filed a correction against a claim whose refutation had been sitting two sentences below
> it, in the journal entry that celebrated it, for three days. The interesting part is not
> that the claim was wrong. It is that **the argument for it was answering a position
> nobody held**, and that the design it displaced was the user's own — reconciled inside an
> implementation slice, never contested, in a document the user does not read.

## It started as a question about weathering

Mid-walk, parked deliberately: *"wondering why weathering lowers height. what operations
does weathering do that are not transform in place? are they something that should happen
prior to river pass?"*

The answer to the first is **it doesn't**. `WeatherApply::apply` is `*r -= q; *h += q`,
mass-neutral, and `surf = r + h` is unchanged to the bit. The answer to the second is
**none** — exactly one form change, `Structural → Loose`, and the pass *declines* any other
transform a behaviour returns, because it did not declare it. That is one of the cleanest
things in the deep-time code.

Which meant the stub the whole session had been re-scoping was wrong on one of its four
named mechanisms. `stubs.md` #29 said *"four phases run after incision and can lower a cell
past its clamped floor: weathering, hillslope creep, wave attack and eolian deflation."*
Weathering cannot lower a cell. It was aimed at the wrong variable — the clamp is written
on `r`, and weathering does push `r` down, but never the surface, which is what the pit
census measures.

Reading the phase order to check that turned up a fifth post-incision phase nobody had
listed at all — **isostasy** — and a shape that looked like a positive feedback: an
equilibrium target computed from a *smoothed* load, differenced against the *local*
surface. That went to the discriminator agent as D3 with an ablation attached. It was
wrong, and wrong in the most useful direction: isostasy is the only grid-scale **damper**
in the solve, and turning it off puts 6,215 closed hollows into a world that has zero.
Three hypotheses died that day and two of them were the assistant's.

## The question underneath

The third question — *should weathering run before the river pass?* — was the one that
opened everything, and the user closed it before the assistant could answer:

> *"the agent may tell you order is a topo sort, i'm not sure you can actually reorder
> them."*

Correct. `passgraph::schedule` — Kahn, id-lexicographic tie-break. You cannot permute
`step()`. But the reason the sort is forced turned out to be the finding. From
`runner.rs`'s own doc comment:

> *"each terrain-transforming stage exposes its output as a distinct **revision** token the
> next stage consumes — `Forced → Incised → Weathered → Diffused → Compensated → Windblown
> → Settled`. That is the honest declaration of a fixed-order relaxation pipeline, and it is
> what **forces the topo-sort to reproduce the old loop's phase order**."*

So the graph admits exactly one order, **by construction**. Ask it *why weathering runs
after incision* and it answers *because `Weathered` consumes `Incised`* — and that token
exists **because weathering ran after incision**. The declaration is a faithful record of
the loop and a circular justification of it.

## "My mental model of how passes work is clearly wrong"

The user asked for a steelman both ways, and for an honest account of the SDK surface. The
account is short, because there is less machinery than the vocabulary suggests: there is no
revision-token *mechanism*. A revision token is a resource axis, declared and read exactly
like `Climate` or `Routed`. It is a **naming convention inside a closed enum**, and that
enum lives in `dc-worldgen`.

The case for it is real. A dependency graph genuinely cannot express *N modifiers of one
resource, in this order* — declare a single shared `Terrain` and the kernel rejects the
whole schedule with `AmbiguousWriters`, which is a good rejection. Naming the intermediate
states is also what made journal/0107's sidecar bug *stateable*: `dc:field/head` read `R+H`
and could not say **when**, and `Incised` was added precisely so it could.

The case against turned out to be decisive, and the user found its sharpest form:

> *"Our passes are PLUGINS. Our passes should be viewed through the lens of THIRD PARTY
> MODS. We are our own first modders. **THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP,
> EMPHATICALLY.**"*

`DeepAxis` is a closed enum, inside the engine, **whose variants are the names of the
default pack's pipeline stages.** A third party wanting to run between weathering and creep
must name `dc:terrain/weathered` — a token it cannot define, whose meaning is *whatever our
weather pass happens to do*, and which is an internal schedule coordinate exposed as ABI.

## The user's own design was already in the corpus

Then: *"I have lost track of the reasoning that a much earlier claude gave rejecting my
early intention for the runner."*

It is in `ideas.md` § **Pass cadence — the fractional-phase scheduler (user sketch,
2026-07-23)**, intact: per-pass phase length, short-phase passes running many times between
long ones, *"a phase is handed the cell state at its start… and duration is a scalar on its
transformations"*, epochs declared with pass members plus a chapter count **or a terminating
condition**. And explicitly: *"**A canonical start order** (tectonics → hydro → weathering)
= the order each pass takes its first turn."*

It was **RECONCILED 2026-07-24** into `material-behavior.md` §5, and the reconciliation cut
it in half. **RATE survived** — ratified, promoted, cross-referenced, and *never built*
(`dt` pinned to `1.0`, nothing scales by it). **ORDER was replaced**, in one clause:
*"derived from `{reads, writes}` by topo-sort… **This replaces a hand-declared 'canonical
order'**."*

There is no argument for that clause anywhere. What exists is journal/0090's blogworthy
note — *"the moment a hand-ordered loop is replaced by a graph that derives the same order
is the moment the order becomes inspectable and checkable."* That is a claim about
**checkability**, and it argues against **unchecked** order, which nobody proposed.
*Author-and-validate* is equally checkable.

**And the same paragraph refutes itself two sentences later:**

> *"a genuinely linear relaxation pipeline does **not** fall out of a dataflow graph for
> free: the terrain is read, transformed, read again — and **you have to name each revision
> as a distinct resource for the topo-sort to reproduce a fixed sequence**."*

The order does not fall out of the declarations. **It is fed into them.** The seven revision
tokens *are* the hand-declared canonical order, re-encoded so the graph appears to compute
it. Filed as corrections #65, with the generalisable form:

> **A derivation whose inputs were constructed to produce the desired output is not a
> derivation. Ask what the mechanism would produce if you had not known the answer in
> advance** — here, nothing: without the revision chain the schedule is rejected outright.

## The process failure is the larger half

The user: *"I think I failed to fully argue it."*

They didn't. The sketch was filed as *"carried forward to compare against the actual
deep-sim loop and discuss next session."* The comparison happened **inside an implementation
slice**, and the ORDER half died in a design doc the user does not read. It was never
contested. It was **reconciled**.

The ratification protocol has always forbidden *recording an unratified assistant proposal*.
It had nothing to say about the mirror — **an assistant reconciliation quietly superseding a
ratified user design** — and that gap cost three days and produced `DeepAxis`. The new rule
is in CLAUDE.md: *a user-originated design may not be superseded by an implementation slice;
a reconciliation that contradicts one is a loud plea to main session, exactly like a spine
deviation.*

Worth being precise about which way this cuts. The reconciliation was **defensible
engineering**, and whoever wrote it believed it. The defect is not that it was wrong. It is
that it was **unilateral and invisible**.

## And the unbuilt half is the blocker

The discriminators landed in the middle of this. The defect that has held the erosional arc
is **not** the incision clamp, **not** the timestep, **not** isostasy: it is
`erosion.rs::diffuse_scale_cell`, which caps a cell's export at its **entire regolith
inventory** with no `dt` in the expression at all. A donor-cell scheme that moves everything
downslope has a period-2 mode by construction — A gives all its cover to B, B is now higher
and gives it back — and that is independent of step size, which is exactly why refining the
step did nothing (the limiter's binding fraction moved 96.0 % → 94.7 % across a 4×
refinement: measurably deaf).

The heir's prescription, in the agent's own words: *"a hillslope operator whose transfer
stays a function of **`rate × dt`** when cover is thick."*

`rate × dt` **is the RATE axis** — the surviving half of the user's sketch, ratified two days
earlier and never built. So the fix is not a patch beside the architecture. It is the
architecture's first real consumer, and the world becoming grid-stable is its acceptance
test.

## What this entry is really about

Both halves of a user's design were right. One was replaced by an argument that refutes
itself in its own journal entry; the other was adopted and left unbuilt, and its absence is
the bug we spent two days diagnosing.

The corpus held every piece of this the whole time — the sketch, the reconciliation, the
refutation, and the unbuilt axis. Nothing was hidden. What failed was that **no reader ever
had all four in view at once**, because they live in four documents and one of them is seven
thousand lines. The user named it at the close of the same session: *"our docs ops past
critical mass, causing information loop to fail to close."*

That is the honest diagnosis, and it is not solved by anyone reading harder.
