---
name: passes
description: Add, modify, split, retire or generalise a PASS in the deepsim default pack — the cellular/field passes that model physical processes in deep time (weathering, transport, deposition, creep, drainage, tectonics, genesis) and write the world's history. Carries the process-not-snapshot rule, the two-stores rule (deposition writes the record, transformation writes the fact ledger), the altitude test, the refinement firewall, and the live defect inventory. Use whenever anyone proposes a new deep-time pass, changes what an existing pass records, questions whether a pass is at the right altitude, or reaches for a rate constant. NOT for the pass runner, the field-solver kernels, or the record's storage — that is dc-core/engine primitive work; this skill is pack process content only.
---

# passes — the deepsim pack's physical processes

**⚠ STATUS: DRAFT, written 2026-08-05 at the user's direction to get down what one
session established.** It has **not** been swept against the corpus, and several of its
findings are single-trace results from one session that nobody has independently
verified. Every claim below carries its provenance and confidence. **Do not treat an
unverified trace as a fact**; § 5 marks them.

**Scope.** This skill governs the **default pack's deep-time passes** — the content that
models physical process and writes history. It does **not** govern the **pass runner**,
the ordering/validation machinery, the field-solver kernels, `DeepAxis`, or the record's
storage layout: those are engine primitives (`dependency-graph.md` § 1). If your change
is to the *capability* rather than the *process*, you are in engine territory and this
skill only tells you to say so loudly.

**Sibling skill:** [`roster`](../roster/SKILL.md) governs the *materials* passes act on.
The two are tightly coupled — a transformation names a material edge — and a change that
needs both wants both skills open.

## 0. SWEEP BEFORE USE — this skill is self-updating, by design

Same clause as `roster`, and for the same reason: a stale procedure doc is worse than
none (the corpus's one-directional-pointer lesson). Before relying on anything below:

1. Grep the corpus for rulings newer than this file's watermark on:
   `ARCHITECTURE.md` DECIDED entries · `docs/design/{earth-processes,material-behavior,
   refinement,flow,worldgen}.md` · ROADMAP § Sequenced/Observed (weathering, P2, E4, E6/E7)
   · `dependency-graph.md` §§ 1–2 · the newest journals and design audits touching deep time.
2. If anything bears on this skill, **update this file in the same session and check with
   the user.**
3. Record the watermark here when you update: **last swept/updated — NEVER. Written
   2026-08-05 from one session's investigation; the § 0 sweep is OWED and is the first
   thing the next reader should run.**

## 1. The philosophy

**A pass models a physical process. It runs where and when the process runs, it takes
its rate from the literature, and it leaves a fact for everything it does.**

Four rules, each with a receipt in the corpus:

- **PROCESS, NOT SNAPSHOT** (`session-workflow`, 2026-07-24, corrections #46/#47). A
  continuous process run once, post-hoc, over frozen final state is a **category error,
  not a simplification** — a snapshot of a movie. A seam-first slice of a process is a
  *smaller but real run of the process* (fewer cells, coarser grid, still in the loop),
  never a one-shot demo. Weathering was caught doing exactly this and rebuilt.
- **MEASURE AGAINST THE LITERATURE** (CLAUDE.md, journal/0111). A closed sim cannot
  detect its own scale error — ours was denuding ~1000× too slowly with every internal
  check green. Any quantity with a published counterpart gets checked against the
  published band **at least once**. A constant tuned until an output looks right is a
  number pretending to be a mechanism.
- **CONSERVATION.** A pass redistributes or transforms; it does not create or destroy.
  Where a budget exists, over-spending goes negative and fails loudly.
- **NO FACTLESS STATES — USER RULING, 2026-08-05, verbatim:**
  > *"Refinement does not write a single fact about the world. It uses them as inputs…
  > If there is refinement expressing material removed by water/wind, deposited, some
  > future crack opened in the earth or cavern or mountain or whatever, it is expressing
  > artifacts from deeptime and **NEVER adding new facts/altering stores, creating
  > factless states (by construction should be impossible)**."*

  This sharpens two existing rules (*the record is the only seam*; *refinement
  redistributes, never creates*) with the part neither stated: **every visible thing
  traces to a recorded fact.** A cavern that exists because an operator felt like a
  cavern is illegal even if mass balances. The *by construction* clause asks for a
  structural guarantee, not a convention — refinement must have no write path to the
  stores. **⚠ Believed true today and NOT VERIFIED** (§ 5, D-5).

**The corollary that decides most arguments:** refinement may invent **arrangement**,
never **substance**. The sim says a cell holds 12 m of this and 3 m of that; refinement
decides where within the cell it sits and must return those totals exactly. Inventing
*where* is a geologist drawing a cross-section between two cores. Inventing *what* is
the "enhance" trope from a spy show, and no conservation law makes it honest.

## 2. The two stores — and which one your pass writes

**This is the rule most likely to be got wrong, because only one store existed when the
oldest passes were written** (the height planes: 2026-07-19; the fact ledger: 2026-07-24).

| | **the record** (`DeepStrata`, `DepUnit`) | **the fact ledger** (`FactLedger`) |
|---|---|---|
| models | **arrival** — material was deposited here | **transformation** — material became something else |
| shape | a bottom-up stack of units, each with material, thickness, epoch, chapter | facts attached to an existing unit (or the bedrock sentinel) |
| a write | appends a unit at the **top**, or merges into the top unit if the key matches | appends a fact; **creates no unit and moves nothing** |
| carries | material identity, thickness, when, mover | declared **edge** `(material, form) → (material, form)`, **cause**, amount, chapter |
| ordering | **strictly top-append; no writer inserts below the top and no writer reorders** — enforced, and load-bearing for the deposition clock's monotonicity, which the stratigraphic-correlation join depends on | unordered within a slot; carries its own chapter stamp |

**THE RULE: deposition writes the record; transformation writes the ledger.** A pass that
transforms material in place and mints a *unit* is writing to the wrong organ — it will
put its product at the **top** of the stack when the process happens somewhere else, and
nothing downstream can tell its product from an arrival.

**USER FORMULATION, 2026-08-05** (the cleanest statement of the constraint we have):
> *"Weathering, without transport, **cannot create new layers**, except perhaps
> **downward** — and only because we have a compromise point at the bottom of the record
> where there is no earlier history… Transport could create a new layer by moving and
> depositing material somewhere — at mass-conserving rate it is removing material from
> elsewhere."*

Read it as the test: **did your process MOVE mass between places?** Then it may mint a
unit. **Did it change what mass IS, in place?** Then it owes a fact, not a unit.

**A transformation fact structurally cannot lie about its edge.** The endpoints are held
as an id into a declared transition graph, so a pass cannot record a transformation the
material system does not declare. That is the property worth protecting in any new pass.

## 3. What a pass owes — the procedure

1. **Declare `{reads, writes}`.** The runner derives order by topological sort; you never
   write the order. Where one plane carries several revisions per epoch, declaring the
   revision you consume pins **one side only** — pin the other with an anti-dependency on
   the **next** revision of that plane, never the last (`session-workflow`, the audit that
   was half wrong about this).
   ⚠ **The vocabulary is a closed engine enum (`DeepAxis`, 18 variants) and is NOT
   retired** — verified 2026-08-05. Retiring it (packs declare their own resource ids) is
   E6, unblocked and sequenced. **A pass split that needs new axes is therefore upstream-
   blocked or must extend an engine enum**, which is the thing that enum is scheduled to
   stop being.
2. **Run in the loop.** Every epoch, on that epoch's live state. See § 1.
3. **Take `dt`.** Cadence is authored data; the pass takes its phase length from the
   engine. A rate that assumes `dt = 1.0` is a bug waiting for a cadence change — and
   several remain (stream transport, bedrock weathering, wind and wave). Each needs a
   **modelling** call, not a mechanical one: weathering is an exponential approach
   (`1 − exp(−k·dt)`), not `k·dt`.
4. **Ask the ALTITUDE question** *(added 2026-08-05 from the weathering investigation;
   assistant-formulated, user-originated as a critique)*: **is this pass formulated as
   the general process, or as one special case of it?** The user's test, in their words:
   *"The forces that cause weathering do not magically only impact basement rock."* A
   pass that acts on one privileged slot, one hardcoded material, or one class of target
   will be re-implemented once per target later. **The general version is usually
   simpler than the special-cased one.**
5. **Write to the right store** (§ 2).
6. **Cite the rate's literature** (§ 1). The soil-production taper (exponential decline
   of bedrock conversion under thickening cover) is the in-tree example done right.
7. **Ship behind an identity default if it moves the world.** Off ⇒ absent ⇒
   byte-identical. **But know what that costs:** a default-off pass means every guard,
   golden and probe around it runs on a world nobody ships (corrections #51's shape). If
   a slice "moves nothing," check whether the thing it moves **exists in the default
   build** before believing the byte-identity.
8. **The flip is the USER's**, walk-gated — and note a walk verdict is *not* a
   ratification of the model (§ 4).
9. **Say what changed where it is asked:** the design doc if the process model moved,
   ROADMAP if it unblocks/blocks anything, `dependency-graph.md` if it moves a state or
   reveals an edge, and this skill (§ 0) if the ruling landscape moved.

## 4. A walk verdict is not a model verdict — USER RULING, 2026-08-05

> *"It doesn't matter if I like the look, **looks lie all the time here**. I care about
> the model first."*

Recorded practice says appearance changes are walk-gated and the user ratifies looks from
screenshots. It did **not** say what a *positive* verdict entitles us to. It entitles us
to nothing about the model. **A good look on a dishonest model is worse than a bad one,
because it makes the dishonest model harder to retire.** The weathering front is the
worked case: correctly-shaped profile, real climate driver — and a hardcoded product
class expressed at the wrong altitude, any of which the look would have carried past us.

## 5. The live defect inventory — weathering, and what it exposes

**All of § 5 is from one session (2026-08-05) and is UNVERIFIED unless marked.** It is
recorded because nothing on the board describes it, not because it is settled.

**D-0 — the reason the weathering flip was deferred, RECOVERED BY THE USER 2026-08-05
(user-originated, lost from the record):** *it wants to be a general process.* The user
declined the flip in the past for this reason; the reason was never written down and was
re-derived from scratch at the cost of most of one session. **Written here so it is not
lost a second time.**

**D-1 — there are TWO weathering systems and they are openly unreconciled** (documented
in-tree; `stubs.md` #17's residual says in as many words: *"do not read the discharge as
'the two authorities are reconciled'"*).
- **The height tier** (`erosion/weathering.rs`, from the 2026-07-19 bring-up spike,
  **default ON**): converts bedrock to regolith as mass (`R -= q; H += q`), rate =
  `base × (biotic × weatherability × frost) × taper`, weatherability blended from the
  cell's **real material window at member grade**, taper = the literature soil-production
  function. **It weathers whatever is at the surface — it is already the general case.**
  It is also the **rate limiter for all denudation** (diffusion is flux-limited by the
  regolith available), so it is not decoration.
- **The composition tier** (`weather_inventory`, 2026-07-24, **default OFF**): a proper
  per-epoch in-loop pass writing declared, cause-carrying transformation facts — **onto a
  single sentinel slot representing a hardcoded 50 m granite basement** (`stubs.md` #16).

  **The one that acts in the right place records in the wrong shape. The one that records
  in the right shape acts in the wrong place.**

**D-2 — the height tier records its product as DEPOSITION** (traced end to end
2026-08-05; phase order `transport → weather → diffuse → record` confirmed; **not
independently verified**). Weathering adds its `q` to `dh`, the epoch's net height change,
which the recorder turns into a unit. **Nuance that softens an earlier overstatement:**
the recorder is *not* blind — it computes `dh − what the movers carried` as the
**incumbent** ("bedrock weathered to regolith in place… never rode any mover"), and if the
incumbent wins the argmax the identity comes from the environment rather than a mover. But
that knowledge exists for one comparison and is then discarded: **no durable fact says
"made here, not brought here."** Where arrivals outweigh it, the weathered material is
absorbed into a unit labelled with a mover's species. Related: `stubs.md` #25 (the record
carries no mover axis; `colluvium_probe` measures the signature the label is missing).

**D-3 — the geometry is inverted.** A weathering front advances *downward*; the newest
product forms at the **bottom** of the loose pile. The record is strictly top-append
(§ 2), so the product lands at the **top**. The two weathering systems disagree about
direction as well as about substance: the composition tier's band correctly *grows
downward at the basement contact*; the height tier's product goes up.

**D-4 — the record and the expression disagree about what the product IS** (documented
in-tree, and the code says so):
> *"The inventory says **granite**-loose while the collapse expresses a **clastic-fine**
> product: that disagreement is stub #16's, and the same heir closes both."*

The record's fact is a **form-only** change (granite solid → granite loose, no material
change). The collapse expresses parent rock with a **fine-clastic** product in its pores,
graded 1/8 → 7/8 with height. That is why the world shows mudstone and siltstone. The
**parent** is honestly inherited from the recorded basement; only the **product** is the
stand-in. And "clastic fine" is a **class** — the hand-assigned category the ratified
roster philosophy rules against, so this stand-in waits on the same term-space work the
`roster` skill's § 2b names.

**D-5 — the refinement firewall is believed, not proven.** § 1's *by construction* clause
needs verifying: does any refinement-side path hold a write handle to the record, the
ledger, or the planes? Expected answer: no. **Nobody has checked.**

**D-6 — the alteration clock is 25× coarser than the deposition clock.** A unit carries
its exact epoch; a fact carries only its chapter (8 chapters, ~25 epochs each). So layers
can be correlated between cells by a clean keyed join on time and **alterations cannot**.
Relevant the moment anything writes facts against real units.

**D-7 — the flux archive is bulk-only, but the SIM IS NOT.** Corrects a natural
misreading: the face record carries a single load figure and no composition — but it is
*"written, never read"*, a pure archive that drives nothing. **Transport itself is
species-resolved**: the fluvial pass knows per species exactly what it set down, creep
knows per species what came down the slope, and both hand per-species rows to the
recorder (summed, never ranked — asking which agent "wins" would make the answer depend
on call order). So composition survives movement **inside** the sim; what cannot see it
is **refinement**, which reads the archive. Heir: the fluvial record-terms slice.

**D-8 — no readable unconformities** (measured, corrections #99): the unconformity flag is
set only on a **full** strip, so a flagged unit is always the bottom-most and its
predecessor is deleted. Zero interior flagged contacts world-wide. Whether that is a
defect or merely unbuilt is **open**.

## 6. What generalising weathering appears to mean

**Assistant analysis, 2026-08-05, unratified — recorded as the session's reading, not as
a plan.** Given D-1, the work is not "make the composition pass touch more layers." It is:
**the height tier emits transformation facts instead of height deltas, and `R`/`H` become
views of those facts** — which is already the corpus's stated end state
(`material-behavior.md` § 11's continuation slot) and has simply never been owned. The two
systems converge rather than one growing to cover the other.

**Open, and named by the user:** the multiple causes bundled into one process (chemical,
biotic, frost, dissolution) *may* want splitting into separate passes — which runs into
`DeepAxis` (§ 3.1). **Cost unpriced.**

**Blocked on the same thing the margins question is blocked on:** a transformation fact
needs a declared **edge**, and an edge needs materials that declare where they sit and
which way they move. See `roster` § 2b. The FS-A release spectra are the "which way it
moves" half and already exist, literature-cited.

## 7. Pointers

- `docs/design/material-behavior.md` §§ 3–4, 11 (the transformation model, the
  two-authorities continuation slot) · `earth-processes.md` (method items) ·
  `refinement.md` §§ 2, 5 (the priors and the three laws — Law 1 anti-carve, Law 2 no
  verdict painting, Law 3 conservation).
- `crates/dc-worldgen/src/deeptime/`: `runner.rs` (`DeepPass`, `DeepAxis`),
  `erosion/weathering.rs` (the height tier), `weather_inventory.rs` + `inventory.rs`
  (the fact tier, `Fact`, `Cause`, `EdgeId`), `recorder.rs` (the record, the top-append
  invariant), `erosion/record.rs` (`arriving_material`, the incumbent), `flux.rs` (the
  archive).
- `crates/dc-worldgen/src/geology.rs::emplace_weathering_front` — where the profile is
  imposed and where D-4's substitution is written down.
- `docs/design/stubs.md` #16 (flat granite basement), #17 (+ its residual), #20, #25.
- `docs/dependency-graph.md` §§ 1–2 (E3 RATE, E4 kernels, E6 `DeepAxis`, P1–P11).
- The 2026-08-05 geo-session conversation — the investigation this skill compresses.
