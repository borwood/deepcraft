---
name: passes
description: Add, modify, split, retire or generalise a PASS in the deepsim default pack — the cellular/field passes that model physical processes in deep time (weathering, transport, deposition, creep, drainage, tectonics, genesis) and write the world's history. Carries the two pass shapes, the four scheduler axes, the two-stores rule (deposition appends a UNIT, transformation appends a FACT), the A-7 altitude test, the refinement firewall, and the live defect inventory. Use whenever anyone proposes a new deep-time pass, changes what an existing pass records, questions whether a pass is at the right altitude, or reaches for a rate constant. NOT for the pass runner, the field-solver kernels, or the record's storage — that is dc-core/engine primitive work; this skill is pack process content only.
---

# passes — the deepsim pack's physical processes

**Scope.** The **default pack's deep-time passes** — the content that models physical
process and writes history. **Not** the pass runner, the ordering/validation machinery,
the field-solver kernels, `DeepAxis`, or the record's storage layout: those are engine
primitives (`dependency-graph.md` § 1, and the scope line was checked against it by the
2026-08-05 sweep — it passes). If your change is to the *capability* rather than the
*process*, say so loudly and go to dc-core.

**Sibling:** [`roster`](../roster/SKILL.md) governs the *materials* passes act on. A
transformation names a material edge, so a change needing both wants both skills open.

## 0. SWEEP BEFORE USE — this skill is self-updating, by design

A stale procedure doc is worse than none. Before relying on anything below:

1. Grep the corpus for rulings newer than the watermark: `ARCHITECTURE.md` DECIDED entries ·
   `docs/design/{earth-processes,material-behavior,refinement,flow,worldgen,materials,
   knowledge,pass-declaration-history}.md` · `docs/spines.md` **A-7, S-2, S-6, § 3** ·
   ROADMAP § Sequenced/Observed — **especially `ROADMAP.md:2376` (WEATHERING IS ONE PROCESS)
   and `:383` (THE HONEST RECORD)** · `docs/dependency-graph.md` §§ 1–2 · `docs/audits/
   2026-08-0{3,4}-*` · journals · `corrections.md` · `stubs.md`.
2. If anything bears on this skill, **update it in the same session and check with the user.**
3. **Watermark: last swept 2026-08-05 at base `dd575b65`** —
   `docs/audits/2026-08-05-passes-skill-sweep.md`, **16 findings**; F1–F5 folded in below.
   D-2 verified independently: `docs/audits/2026-08-05-d2-deposition-trace-verification.md`.
   **NOT swept:** `docs/spikes/`, `docs/API.md`, `ROADMAP-history.md`, the deep-time source
   tree beyond `record.rs`/`geology.rs`/`flux.rs`/`inventory.rs`.

> **⚠ READ THIS BEFORE TRUSTING THE HISTORY OF ANY CLAIM HERE.** The first draft (2026-08-05)
> was written from a code investigation and was **right about the code and wrong about the
> corpus in five places** — it asserted novelty for things already ruled, and twice told the
> reader a thing was unrecorded when it was live on the board. **A claim in this file that
> something is new, missing or unwritten is the class most likely to be false.** Check before
> repeating one.

## 1. The philosophy

**A pass models a physical process. It runs where and when the process runs, it takes its
rate from the literature, and it leaves a fact for everything it does.**

- **PROCESS, NOT SNAPSHOT** (corrections #46/#47). A continuous process run once, post-hoc,
  over frozen state is a **category error, not a simplification**. A seam-first slice is a
  *smaller but real run of the process*, still in the loop. Weathering was caught doing this
  and rebuilt. **Its sibling error is modelling a STAGE as a mechanism** — *"Nature has no
  saprolite **process**"* (`ROADMAP.md:2401-2404`).
- **MEASURE AGAINST THE LITERATURE** (journal/0111). A closed sim cannot detect its own scale
  error — ours denuded ~1000× too slowly with every internal check green.
- **CONSERVATION.** A pass redistributes or transforms; over-spending a budget goes negative
  and fails loudly.
- **NO FACTLESS STATES — USER RULING, 2026-08-05:**
  > *"Refinement does not write a single fact about the world. It uses them as inputs… it is
  > expressing artifacts from deeptime and **NEVER adding new facts/altering stores, creating
  > factless states (by construction should be impossible)**."*

  **This is not the rule's first statement.** `knowledge.md` req 3 (seeded 2026-07-19,
  ratified): *"**Computable anomaly.** Every natural feature carries process provenance…
  **anomaly = no valid provenance match.**"* And four ratified priors already constrain
  refinement: the record is the only seam (`refinement.md:74-76`); **refinement is a pure fn
  of (record, face data, position), which already forbids the write path BY TYPE** and is most
  of the "by construction" the ruling asks for (`:70-73`); Law 3 conservation (`:218-223`);
  Law 1 anti-carve (`:198-207`). **Recorded prior violations:** `carve_rivers`
  (`refinement.md:63-66`) and the weathering profile's unmeasured shape (`stubs.md` #20).

**The corollary that decides most arguments:** refinement may invent **arrangement**, never
**substance**. Inventing *where* is a geologist drawing a cross-section between two cores.
Inventing *what* is the "enhance" trope, and no conservation law makes it honest.

## 2. The two stores — and which one your pass writes

**RATIFIED, not new — `material-behavior.md:76-85` § *Commit semantics*, DECIDED 2026-07-24
(user):** *"In-place transformation → **append a FACT** to the existing unit… Depositional
arrival → **append a new UNIT**… **Transport is a removal fact here + a new unit at the
receiver.**"* See also `material-behavior.md:58-66`, `:86-89`, `spines.md` § S-2.

| | **the record** (`DeepStrata`, `DepUnit`) | **the fact ledger** (`FactLedger`) |
|---|---|---|
| models | **arrival** | **transformation in place** |
| a write | appends a unit at the **top**, or merges into the top unit on a key match | appends a fact; **creates no unit, moves nothing** |
| carries | material, thickness, epoch, chapter, **mover** (3 bits) | declared **edge** `(material, form) → (material, form)`, **cause**, amount, chapter |
| ordering | **strictly top-append; no writer inserts below the top, none reorders** — enforced, and load-bearing for the deposition clock the correlation join depends on | unordered within a slot |

**THE OPERATIONAL TEST — this, and § 5's height-tier diagnosis, are what is actually new
here.** *Did your process MOVE mass between places?* Then it may mint a unit. *Did it change
what mass IS, in place?* Then it owes a **fact**, not a unit. **USER FORMULATION, 2026-08-05:**
> *"Weathering, without transport, **cannot create new layers**, except perhaps **downward**…
> Transport could create a new layer by moving and depositing material somewhere — at
> mass-conserving rate it is removing material from elsewhere."*

**A fact structurally cannot lie about its edge** — endpoints are an id into a declared
transition graph. Protect that property in any new pass. Caveat: the two-authorities split is
an implementation choice for one slice, **not yet an invariant** (`stubs.md:571-575`).

## 3. What a pass owes

0. **Know which SHAPE you are writing** (`north-star.md` § Passes, ratified 2026-07-23;
   `material-behavior.md:309-322`; **both are content**). **Cellular** — per-cell
   select-and-transform, *"select materials matching P in context C, apply T at rate R"*;
   **runs EDGES**, output is changed material state. **Field** — declares reads/writes over
   fields, body is computation over **core solver primitives** (E4); **computes fields, never
   runs edges, never touches the form inventory.** *For the cell world there is no third.*
   And **`ctx` is a capability, not a god-object** — a pass touches only what it declared.
1. **Declare `{reads, writes}` — and know what they are FOR, because it is DECIDED and it has
   changed.** Today's runner *derives* order by topological sort (E1), but **ORDER IS
   AUTHORED, PER WORLD — DECIDED 2026-07-26 (user)** (`ARCHITECTURE.md` § *Author-and-validate*),
   and under it `{reads, writes}` **stop being the ordering input and become the validator.**
   *"Order is derived from the declarations"* is **falsified — corrections #65**, which names
   three sites; **do not write a fourth.** E7 is unblocked and sequenced.
   - Read today's mechanism as `spines.md:774-782` does: *"the best available compliance under
     derivation — **not** the shape to copy into new design."*
   - **So the revision-token pairing is a derive-and-reject ARTIFACT scheduled for deletion**
     (`ARCHITECTURE.md:579-580`). Use it today; never design around it, and **never add a
     `DeepAxis` variant to buy an ordering**: *"encoding it in engine-owned resource names is
     precisely how `DeepAxis` came to exist. The mechanism and the violation are the same
     choice"* (`:584-586`).
   - Where a plane carries several revisions per epoch, declaring the one you consume pins
     **one side only** — pin the other with an anti-dependency on the **next** revision, never
     the last (`pass-declaration-history.md:158-167`, `spines.md` § S-6).
   - ⚠ **The vocabulary is a closed engine enum (`DeepAxis`, 18 variants), NOT retired.** E6.
     **And it is not the only one — see D-10.**
2. **Declare all FOUR scheduler axes** — ORDER · RATE · WINDOW · SCHEDULE
   (`material-behavior.md:373-379`).
   - **SCHEDULE** (`ARCHITECTURE.md` § *Schedule*, DECIDED 2026-07-29, built journal/0124):
     `Seed` (pre-loop once, integrates **zero time**, `dt = 0.0`) · `Step(Cadence)` ·
     `SeedAndStep`. **Epoch 0 fires for everyone; there is no skip rule.** The discriminator:
     *a seed is an initial condition **only if something OBSERVES it before the pass itself
     first steps**.* All three pre-loop incumbents audited to `Step`, so **`Seed`/`SeedAndStep`
     have no production declarer** — reach for one and you owe the argument.
   - **WINDOW** (`flow.md:730-760`): how many epochs sum into one record entry. **A window
     that decides an acceptance number must be declared, not assumed.**
   - **RATE:** cadence is authored data; take your phase length from the engine. A rate
     assuming `dt = 1.0` is a bug waiting for a cadence change — several remain (stream
     transport, bedrock weathering, wind, wave), each needing a **modelling** call: weathering
     is an exponential approach (`1 − exp(−k·dt)`), not `k·dt`.
   - **The invariant, a gate test not a narration:** every pass's integrated `dt` over a run
     equals the world's elapsed time.
3. **Write to the right store** (§ 2).
4. **Ask the ALTITUDE question — and it is anti-shape A-7, DECIDED 2026-07-22 (user), not a
   new formulation** (`spines.md:2279-2299`): *"Naming directly will never, in any world, be
   correct… **This is not a ratifiable carve-out… it is a defect.**"* **So a pass that names a
   content identity, acts on one privileged slot, or hardcodes a target is already wrong — the
   question is not whether, but what.** A-7's resolution: *"ask **what property of that content
   you are actually reaching for. That property is the feature the process lacks.** Name it,
   and the special case dissolves for every future member of the same family."* The user's
   restatement: *"The forces that cause weathering do not magically only impact basement rock"*
   (2026-08-05) — and on the board since 2026-07-25 (`ROADMAP.md:2409-2411`).
5. **Cite the rate's literature.** The soil-production taper is the in-tree example done right.
6. **Ship behind an identity default if it moves the world** — off ⇒ absent ⇒ byte-identical.
   **But know the cost:** a default-off pass means every guard, golden and probe around it runs
   on a world nobody ships (corrections #51). If a slice "moves nothing," check the thing it
   moves **exists in the default build** before believing the byte-identity.
7. **The flip is the USER's**, walk-gated — and a walk verdict is not a model verdict (§ 4).
8. **Say what changed where it is asked:** the design doc, ROADMAP, `dependency-graph.md`, and
   this skill's § 0.
9. **Answer the standing question** (`earth-processes.md:416-420`): ***"What does this look
   like in reality, and how did it get there?"* — if the pass cannot answer in those terms it
   is not ready to be declared.** Upstream, method items 1–5 (`earth-processes.md:9-29`,
   RATIFIED 2026-07-19): **field notebook first, before any code** · mechanism fidelity over
   resolution fidelity · **the record is the world** (present-day fields drive only present-day
   processes) · validation is a geologist's-eye walk · the unsimulated remainder is dressed,
   never left as a square edge.

## 4. A walk verdict is not a model verdict — USER RULING, 2026-08-05

> *"It doesn't matter if I like the look, **looks lie all the time here**. I care about the
> model first."*

Recorded practice walk-gates appearance and has the user ratify looks from screenshots. It did
not say what a *positive* verdict entitles us to. **It entitles us to nothing about the model.
A good look on a dishonest model is worse than a bad one, because it makes the dishonest model
harder to retire.** The weathering front is the worked case.

## 5. The live defect inventory

`ROADMAP.md:396-399` names this skill as the inventory's single home — deliberately, checked
2026-08-05. Confidence is marked per item.

**D-0 — the general-process destination. ⚠ IT WAS WRITTEN DOWN.** `ROADMAP.md:2376`
§ Sequenced, **USER'S STRONG LEANING, 2026-07-25**, in the user's own words — *"WEATHERING IS
ONE PROCESS — SAPROLITE IS A STATE ALONG IT, NOT A SLICE"*, requisites **MET** the same day,
**gate OPEN**, a design question not a prerequisite one. **A whole session (2026-08-05)
re-derived it from scratch because nobody grepped for weathering before opening the thread.
Read that entry before touching this.** What 2026-08-05 adds is only the link to the *flip
decision*, which may not be minuted anywhere.

**D-1 — THREE weathering authorities, not two** (`ROADMAP.md:2395-2400`).
- **height tier** `erosion/weathering.rs` (bring-up 2026-07-19, **default ON**): `R -= q;
  H += q`, rate `base × (biotic × weatherability × frost) × taper`, weatherability blended
  from the cell's **real material window at member grade**, taper = the literature soil-
  production function. **Weathers whatever is at the surface — already the general case.**
  It is the **rate limiter for all denudation**.
- **inventory tier** `weather_inventory.rs` (2026-07-24, **default OFF**): a proper per-epoch
  in-loop pass writing declared, cause-carrying facts — onto **one hardcoded granite sentinel**
  (`stubs.md` #16). Deliberately **not numerically consistent** with the height tier.
- **`geology.rs::emplace_weathering_front`** — *"not declarative — ordinary collapse code that
  invents the vertical distribution."* **Only one of the three is authored in the shape
  everything converges on.**

  The sharpest *pair*: **the one that acts in the right place records in the wrong shape; the
  one that records in the right shape acts in the wrong place.**

**D-2 — the height tier records its product through the ARRIVAL channel** (traced twice, once
blind — `2026-08-05-d2-deposition-trace-verification.md`). Write set is exactly `r -= q;
h += q; dh += q`; `dh` is zeroed once, at transport, **before** weather; order
`transport → weather → diffuse → record`, **asserted by a live test** (`runner.rs:1222`). Mass
reaches the strata record, reaches **neither** the fact ledger nor the flux record, and **no
conversion fact is written on this tier**. The unit lands at the **top** of the stack, where a
weathering front advances **downward** (D-3).
- ⚠ **CORRECTED 2026-08-05.** The draft claimed *"nothing durable says made-here vs
  brought-here."* **False.** Every unit carries 3 mover bits, `MOVER_NONE = 7` = **made in
  place**, packed in the bitfield and **in the merge key**. `stubs.md` **#25 is DISCHARGED
  2026-08-02/03** — and the doc comment at `record.rs:124-128` still describes it as unbuilt,
  which is what produced the wrong claim. **A-2, filed; the pointer ran one way.**
- **What is actually lost:** the **proportion** (`carried` is computed and discarded, so we
  keep *which* dominated and lose *by how much*); and **bedrock parentage entirely**, for a
  structural reason — **`R` is basement everywhere and carries no per-cell material.** When a
  mover outvotes the incumbent, the weathered metres take the transported rock's name and
  mover.

**D-3 — the geometry is inverted.** Confirmed. **⚠ CANNOT DETERMINE whether top-of-stack
placement is ratified or merely unexamined** — the code is unambiguous, the intent is stated
nowhere either trace found. Settled by a ruling or by the user.

**D-4 — record and expression disagree about the product** (documented in-tree): *"The
inventory says **granite**-loose while the collapse expresses a **clastic-fine** product: that
disagreement is stub #16's."* The record's fact is **form-only**; the collapse expresses parent
rock with fine-clastic product in its pores, graded 1/8 → 7/8. The **parent** is honestly
inherited; only the **product** is the stand-in — and "clastic fine" is a **class**, the hand-
assigned category the roster philosophy rules against. The imposed profile is **`stubs.md` #20**
and the shape carries a **7/8 cap** (`ROADMAP.md:2408-2410`); **modelling the cause DELETES #20
rather than tuning it.**

**D-5a — the refinement WRITE path.** Narrower than the draft said: **purity forbids it by
type** (`refinement.md:70-73`). Still unverified for the planes. **D-5b — the refinement READ
path** is **measured 2026-08-04: "roughly one-quarter true."**
⚠ **AND THE RECONCILIATION IS OPEN AND USER-OWNED:** a strict reading of *no factless states*
collides with method item 5's **dress-every-contact** ruling (2026-07-19) and outlaws **all
sub-460 m relief in the shipped world**, which comes from move C — a stochastic synthesizer
tracing to no fact. **Both rulings are the user's. Do not resolve this in a slice.**

**D-6 — the alteration clock is coarser than the deposition clock. FILED as `stubs.md` #52**
(2026-08-04, journal/0154). Sharper statement: **chapter is rewritten in place, so chapter and
epoch disagree after overprint**; the correlation partition uses **epoch**.

**D-7 — the flux archive is bulk-only, but the SIM IS NOT.** The face record is *"written,
never read"* — a pure archive driving nothing. **Transport is species-resolved**: fluvial knows
per species what it set down, creep knows what came down the slope, both handed to the recorder
(summed, never ranked — else the answer depends on call order). Composition survives movement
**inside** the sim; what cannot see it is **refinement**, which reads the archive. Heir: the
fluvial record-terms slice.

**D-8 — no readable unconformities** (measured, corrections #99): the flag is set only on a
**full** strip, so a flagged unit is always bottom-most and its predecessor is deleted. Zero
interior flagged contacts world-wide. Defect or unbuilt: **open**.

**D-9 — 🔴 COMPOSITION IS DISCARDED AT RECORDING** (verified twice, once blind). `record_cell`
deposits the **full net thickness** under a **single** argmax `species` (`record.rs:74`, `:86`).
0.6 m of sandstone by river + 0.4 m of mudstone by creep records **1.0 m of sandstone**. Mass
survives (sub-quantum residue rides `DeepStrata::carry`); identity does not. `DepUnit` has one
6-bit species field; **no path yields >1 unit or a multi-material unit** from one cell's epoch
gain. Grain is `GRAIN_UNSET` everywhere by design.
- **D-7's twin, and together the shape of the whole problem** — twice the sim computes
  composition and stores a summary of it (faces: per-species load → one bulk figure; units: the
  arriving mixture → one winner). *A summary is not an authority*, at the deepest layer in the
  stack, with the unusual property that **the authority is not unavailable — it is in scope at
  the write.**
- ⚠ **The argmax WAS a decision** (`stubs.md:899-903`, `:914-922`; members-into-history `:310`
  costed **Option B, a share vector**). The **discard** is what is unreasoned. **corrections
  #57 is the worked case where the argmax produced a wrong world — it is D-9's best argument.**
- **Consequences:** correlating between boreholes blends names that may each already be a
  flattened mixture, so margin mixtures partly *reconstruct composition the sim discarded*; and
  "should the record carry distributions" is **not a storage upgrade, it is stopping a
  discard** — the tracking is built, only the cost of a wider unit is open.
- **UNMEASURED and owed before any remedy:** what fraction of recorded metres loses its
  identity. Sum non-winning mass at each deposit against total recorded metres.

**D-10 — 🔴 `DeepAxis` IS NOT THE ONLY CLOSED ENGINE VOCABULARY, AND THE MOVER ONE IS FULL**
(user's question, 2026-08-05; verified at source, **new — not yet in any audit**).
- **`FlowCause`** (`flux.rs:337-352`) is a closed engine enum of **transport regimes** —
  Fluvial, Eolian, Glacial, Gravity, Marine, Hydrothermal, Dissolution — plus `MOVER_NONE = 7`.
  **Seven variants + none = 8 values = exactly the 3 bits allocated. It is FULL.** A pack
  cannot add a mover (bioturbation, debris flow, saltation-vs-suspension) without a storage
  change across ~7.3 M units, and the field is **in the merge key**.
- **Its own doc comment gives the game away:** *"Regimes differ in four numbers and a field —
  viscosity, density, competence, resistance, and the field they follow."* **That is a term
  definition.** A mover is a pack-declarable thing described by measurable terms, hardcoded as
  an enum variant — the roster philosophy's exact shape, denied.
- **Worse than `DeepAxis` in one way:** `DeepAxis` is compile-time wiring; this is **persisted
  in the record**, so opening it is a data migration, not a refactor. **Better in one way:** the
  list is drawn from real regimes and says it is reserving deliberately.
- **And there is a third:** `Cause` (`inventory.rs:395-407`) — Chemical, Biotic, Frost,
  Dissolution — a closed enum of **weathering agents**. **`Dissolution` appears in BOTH enums**,
  owned by neither.
- **So E6 is scoped too narrowly.** It names `DeepAxis`; the defect is a **family**. Any
  weathering split (§ 6) mints new causes and runs straight into it.

## 6. What generalising weathering appears to mean

**Assistant analysis, unratified — the session's reading, not a plan.** Given D-1, the work is
not "make the composition pass touch more layers": **the height tier emits transformation facts
instead of height deltas, and `R`/`H` become views of those facts.**

- **It is OWNED AND HALF-BUILT, not unowned** — `derive_regolith_at` derives `H` from an
  **EMPTY** ledger (`stubs.md` #17's residual; `field.rs:822`/`:904`; `ROADMAP.md:4156-4171`).
- **Cost is PRICED, not unknown:** gen time **25.7 s → 46.4 s** (affordable); residency
  `LedgerField` **17.45 MiB → 973 MiB, 55.8×**, resident. S20 costed four options including
  paged facts. **The residency is the blocker, not the clock.**
- **The split question is RULED, not open** (`material-behavior.md:349-371`): **separate
  passes, decided by coupling timescale vs cadence**, with a moddability tiebreaker. It still
  runs into **D-10**.
- **Requisite, and it is load-bearing** (`ROADMAP.md:2417-2421`): **no vertical fluid flux
  means a depth-resolved rate has nothing honest to read**, and emergent gradation from a
  fabricated depth term is **A-1 with a disguise**. R1/R2/R3 **MET**, with the recharge-free
  caveat (`stubs.md` #19) surviving into the arc.
- **Blocked with the margins question on the same thing:** a fact needs a declared **edge**, an
  edge needs materials that declare where they sit and which way they move (`roster` § 2b). The
  FS-A release spectra are the "which way" half and already exist, literature-cited.

## 7. Pointers

- `material-behavior.md` §§ 3–4, **:76-85 (commit semantics)**, **:309-322 (the two shapes)**,
  **:349-371 (the split rule)**, **:373-379 (the four axes)**, § 11 · `earth-processes.md`
  **:9-29 (method items 1–5 — the *method*, not the engine sketches at `:31-258`, which are
  explicitly not decisions)** and **:416-420 (the standing question)** · `refinement.md` §§ 2, 5
  · `knowledge.md` req 3 · `pass-declaration-history.md` · `flow.md:730-760`.
- `spines.md` **A-7**, **S-2**, **S-6**, **§ 3** · `ARCHITECTURE.md` §§ *Author-and-validate*,
  *Schedule* · `dependency-graph.md` §§ 1–2 · `ROADMAP.md:2376`, `:383`.
- `crates/dc-worldgen/src/deeptime/`: `runner.rs` · `erosion/weathering.rs` ·
  `weather_inventory.rs` + `inventory.rs` · `recorder.rs` · `erosion/record.rs` · `flux.rs` ·
  `crates/dc-worldgen/src/geology.rs::emplace_weathering_front`.
- `stubs.md` #16, #17 (+ residual), #19, #20, #25 (discharged), #52 · corrections **#46/#47**,
  **#51**, **#57**, **#65**, **#99**.
- Audits: `2026-08-05-passes-skill-sweep.md` · `2026-08-05-d2-deposition-trace-verification.md`
  · `2026-08-04-voxel-explainability-audit.md` · `2026-08-04-deposition-clock-design.md` ·
  `2026-08-03-stratigraphic-correlation-design.md` · journals **0152**, **0154**, **0157**.
