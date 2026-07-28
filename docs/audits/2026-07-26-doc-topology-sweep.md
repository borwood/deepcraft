# doc-topology sweep — the 2026-07-26 merge batch

*Run 2026-07-27 against `main` @ `283f3da`. **First ever run** of
`.claude/skills/doc-topology/SKILL.md`, which shipped the day before.*

**Scope (bounded deliberately).** The nouns the 2026-07-26 batch touched — pass
declarations / scheduling, the erosion blocker, fixture-vs-production naming, the
bootstrap-content removal — plus the read-first set for blast radius. Coverage is
itemised at the end, including **what was not opened**.

**This document resolves nothing.** Which side of a pair wins is frequently a user call
— that is why it survived. Every recommendation below is labelled as a recommendation.
Three sub-sweeps ran (the scheduling/read-first spine by hand; fixture-vs-production and
bootstrap-content in parallel); **every citation reproduced below was re-read at source
before it was published here**, and one parallel finding was downgraded on that re-read
(see #6a).

---

## Ranked findings

Ranked by **blast radius**, not by age or by confidence.

---

### 1. 🔴🔴 `CLAUDE.md` tells every future session that no ecology design exists; `ecology.md` is a user-RATIFIED design and the named heir of four live stubs. **Both sides user-originated. I cannot adjudicate this.**

| | |
|---|---|
| **A** | `CLAUDE.md:321-323` (user, 2026-07-26) — *"**We have no evo / ecology / socia / civ modelling even at the design stage.** Anything in the tree that looks like one is early-bootstrap fabrication awaiting **wholesale replacement**. The project is working on **earth processes**…"* |
| **B** | `docs/design/ecology.md:1-8` (user, 2026-07-19) — *"# Ecology, organisms, and evolution / Status: substrate **RATIFIED 2026-07-19** (user: **"this reads absolutely right"**); **the evolution architecture below is the USER'S DESIGN**, recorded 2026-07-19 as the frame to build toward (not v1, but **v1 must not foreclose it**)."* |

**Both are user-originated, seven days apart, and the provenance tie-break cannot decide
this one.** That is exactly the case the skill says to report and not resolve.

**Why it is ranked first: what depends on B.** If A is read literally, these all come
unmoored, and every one of them names ecology as the **heir of record**:

- `docs/design/stubs.md:177-179` — *"**Heir:** ecology-pass vegetation members with proliferation patterns… **Blast:** the geography of all organic facies."*
- `docs/design/stubs.md:94-95` — *"**Heir for the vegetation half:** unchanged — the ecology system."*
- `docs/design/stubs.md:99-104` — *"the Grass branch goes away with ecology rather than being reimplemented by it."*
- `docs/ARCHITECTURE.md:210` — *"some are placeholders today (**the surface veneer awaiting ecology** is the live example)"*
- `ROADMAP.md:91-92` — *"**heir chain: ecology → entry-species record → burned-axis on organics**"*
- `ROADMAP.md:2355-2362` (Sequenced) — *"**Evolution recorded as the user's teleology design**… post-v1, but **v1 must not foreclose it**"* — a live constraint on v1 design.

**A reading that would dissolve it, offered as a reading and nothing more:** A was
written in a *ruins/polities* context, and its very next clause is *"The project is
working on **earth processes**"* — while `ecology.md` is explicitly an earth-processes
doc (§ 0 *"The reality-first account (per earth-processes.md doctrine)"*, Shelford,
Liebig, CLORPT). On that reading A condemns the **socia/civ** half and swept "evo /
ecology" in rhetorically. **The corpus does not say this anywhere**, and A's literal
words are unambiguous.

**Recommendation: put this in front of the user before anything else in this document.**
`CLAUDE.md` is read by every session and every agent brief; until the scope of A is
settled, an agent is entitled to treat a user-ratified design doc as baggage awaiting
removal. **One sentence from the user closes it.**

*Same family, lower stakes, listed so the question is asked once:*
`docs/design/things-that-will-happen.md:126-128` (read-first item 2) — *"You loot an iron
sword from a ruin and the world knows who forged it: **a smith in a kingdom that rose,
warred, and fell across the deep past**"* — a product-vision line against A. And inside
`CLAUDE.md` itself, `:86-88` names the blog's fourth lens as *"full gamut
geo·paleo·archae·eco·anthro·**socia**"*, 234 lines from A. Neither is strictly a
modelling claim; together they are a mixed signal about scope in the one file nobody can
skip.

---

### 2. 🔴 `CLAUDE.md` read-first item 0 restates the exact framing `north-star.md` retires — and `north-star.md`'s own opening sentence still carries it

| | |
|---|---|
| **A** | `CLAUDE.md:9` — *"a native engine whose **core is cell storage + a pass-runner + native field-solvers + a stable API**"* |
| **A′** | `docs/design/north-star.md:20-21` — *"A **native** engine whose core is only **cell storage + a pass-runner + a small set of native field-solvers + a stable API surface**"* |
| **A″** | `ROADMAP.md:24-25` — *"whose core is only cell storage + a pass-runner + **native field-solvers** + a stable API surface"* |
| **B** | `docs/design/north-star.md:183-186` — *"Core is only: **cell storage + cell/space API · the pass-runner · the event ledger · the field-solver primitives** … **Every pass is content** — including tectonics and erosion. … **(This retires the earlier "native field-solvers are core" framing — that sentence conflated compute shape with trust tier.)**"* |

**Provenance.** B is a **2026-07-23 ratified refinement** which states its own precedence
(`north-star.md:174-176`: *"Ratified in design conversation 2026-07-23… where the two
disagree, **this governs**"*). A is the doc's pre-refinement summary, copied forward into
two read-first surfaces. **The doc names its own line 21 as "the earlier framing" at
`:61` and again at `:185`, and never edits it.**

**The difference is not cosmetic.** *"Native field-solvers are core"* puts tectonics and
erosion **in the engine**; *"field-solver **primitives** are core, every pass is
content"* puts them **in the default pack**. That is precisely the axis the 2026-07-26
plugin-agnostic decision turns on (`ARCHITECTURE.md:541-548`: *"**Content:** passes,
their order, their cadence, the **fields themselves**"*). An agent reasoning from item 0
places the boundary one tier too high — and item 0 is the paragraph a brief quotes when
it states how a slice converges toward the north star.

**Recommendation.** One word at three sites — `field-solver **primitives**` — starting
with `north-star.md:20-21`, since the other two are copies of it. **Whether to edit in
place or annotate** is the user's, given the doc's convention of striking rather than
deleting.

---

### 3. 🔴 `north-star.md` § Passes still asserts derived order — the doc corrections #65 names by name

| | |
|---|---|
| **A** | `docs/design/north-star.md:156` — *"**The runner topo-sorts by declared reads/writes and rejects conflicts.**"* |
| **A′** | `docs/design/north-star.md:54-56` — *"the **pass-runner** — runs passes in declared order at declared cadence within declared epochs; topo-validated (**rejects cycles, conflicting writers, missing producers**)"* |
| **A″** | `docs/design/material-behavior.md:276-277` — § 5's **opening sentence**: *"A pass declares itself (`{reads, writes}`, cadence, epoch) and runs through the pass-runner (**topo-sorted**, cycles rejected)."* |
| **B** | `docs/ARCHITECTURE.md:516-518` — *"**Author-and-validate (DECIDED).** Order is **data on the world** — chosen per world, alongside seed and epoch count. `{reads, writes}` **stop being the ordering input** and become the **validator**"* |
| **B′** | `docs/ARCHITECTURE.md:520-522` — *"given an authored one it is just the sequence, and **the rejection that forces revision tokens disappears**."* |

**Provenance.** B is **user-DECIDED 2026-07-26** (`ARCHITECTURE.md:488`). A is
assistant-authored 2026-07-23.

**This is not a near-miss — it is the named site.** `journal/corrections.md:2450` titles
#65 with its three sites: *"(`material-behavior.md` § 5…; `journal/0090`; **`north-star.md`
§ Passes** — falsified 2026-07-26…)"*. Two of the three were struck the same day.
**`north-star.md` § Passes was not touched.** And `material-behavior.md`'s ORDER bullet
was struck properly at `:341-363` while its **§ 5 preamble, 65 lines above the strike,**
still says `topo-sorted` — so the section contradicts itself on its own first line.

`north-star.md:54` is the interesting half: *"runs passes in **declared order**"* is
already the authored-order shape, and *"rejects … conflicting writers"* in the same
clause is the rejection `ARCHITECTURE.md:522` says disappears. **One sentence carries
both models.**

**Recommendation.** A supersession note on § Passes in the shape
`material-behavior.md:345-355` already uses (strike + 🔴 SUPERSEDED + why the falsity is
instructive); same for `material-behavior.md:276-277`. **Not edited here** — the wording
of a supersession on a read-first doc is the user's.

---

### 4. 🔴 `ideas.md` § Pass cadence still carries the reconciliation that superseded the user's ORDER half — the doc `material-behavior.md` claims to have *restored*

| | |
|---|---|
| **A** | `docs/design/ideas.md:530-536` — *"**RECONCILED 2026-07-24 → `material-behavior.md` §5**… **topo-sorted ORDER (derived from a pass's reads/writes)** × fractional-phase **RATE**… **This sketch is the RATE axis — it *composes with* topo-sort, is not replaced by it**"* |
| **B** | `docs/design/material-behavior.md:361-363` — *"**This restores the user's 2026-07-23 sketch** (`ideas.md` § *Pass cadence*), whose ORDER half — *"a canonical start order (tectonics → hydro → weathering)"* — **was reconciled away rather than contested.**"* |

**Provenance.** The sketch at `ideas.md:547-549` is **user-originated**. The
reconciliation banner at `:530-536` is assistant-authored and is the thing corrections
#65 exists about.

**This is the finding the skill was built for.** `ideas.md` is the repository of user
sketches, and the CLAUDE.md rule added the same day — *"**Grep for the sketch before
writing the reconciliation**"* — makes it the **first** place a future reconciliation
looks. It currently tells that reader, in bold, that ORDER is derived by topo-sort and
that the sketch was only ever about RATE. **The restoration was recorded in the doc the
user does not read and not in the doc that holds their design** — the identical failure
geometry to the one being corrected, sign flipped.

**Recommendation.** The banner wants the same 🔴 SUPERSEDED treatment
`material-behavior.md:345` got. **Flagged, not edited** — and note that editing a
user-sketch doc to record what happened to a user's design is precisely the unilateral
move #65 forbids. **This one goes through the user.**

---

### 5. 🔴 The coal calibration's entire evidence base is labelled *"the production world"* in four live documents; it was measured on the **warm reference** world, which is not the world anyone ships — and the shipped world has **no coal at all**

| | |
|---|---|
| **A** | `docs/design/stubs.md:348-351` (stub #14, live) — *"Measured consequence **(production Medium)**: coal-bearing deep cells **12 892 → 2 216**, units **19 008 → 3 888**, recorded coal **22 459 → 3 773 m**…"* |
| **A′** | `docs/design/stubs.md:360-362` — *"of **35 382 peat-derived units in the production world** exactly **13** lie under 50 m of section… so an honest Earth threshold yields a world with no coal at all"* — this is the sentence that justifies `COAL_BURIAL_M = 8.0` |
| **B** | `crates/dc-worldgen/examples/coal_charcoal_probe.rs:23,44-45` — `const SEED: u64 = 0x0D5E_ED57_2026;` … `seed: SEED, extent: Extent::Medium` |
| **B′** | `ROADMAP-history.md:37-40` — *"`0x0D5EED572026` is the **warm reference** world… Both are worlds **nobody ships**"* |
| **B″** | `journal/corrections.md:1657-1660` (#51) — *"**Falsified — the shipped world has NO COAL AT ALL.** On the world `dc-client` actually boots (`BENCH_SEED = 1337`… `Extent::Medium`): **0 coal units across all 297,025 deep cells**"* |

**All sides assistant-originated — but a user ratification is sitting on top of it.**
`docs/design/stubs.md:378-379` records *"the calibration did not change and is not under
review (user, 2026-07-22)"* directly beneath the mislabelled census. The user ratified a
number whose stated evidence names the wrong world.

**Repeated at three further sites**, all quoting the same probe run:
`journal/corrections.md:1164-1165`; `ROADMAP-history.md:1228-1229` and `:1234-1235`;
`journal/0066-the-wrong-axis-and-the-expired-excuse.md:43-44` (the origin). And
`docs/design/geology.md:163` carries the *"13 of 35 382"* figure with **no world label at
all**.

**`corrections.md` contradicts itself on this, 500 lines apart:** `:1164-1165` says
*"the production Medium world"*, while `:1670-1672` says *"A third seed
(`0x0D5EED572026`) appears elsewhere in the coal corpus, so the coal evidence is spread
across **three worlds, none of them the one the player walks**."* The "elsewhere" is an
entry in the same file.

**Why this survived a sweep that was looking for it.** journal/0106 and corrections #51
ran an explicit sibling audit for exactly this defect — and it swept **test helpers
only** (`journal/0106:171-178`). The numbers in the **design docs** were never checked.

**Recommendation.** Relabel — do not re-measure yet. **I cannot state the correct
values**: these figures also predate the geotherm (`COAL_ONSET_C` 8→22, journal/0093), so
they are stale on a second axis, and confirming that needs a cargo run this sweep was
scoped out of. **The world label is wrong with certainty; the numbers are unknown.** That
distinction should survive into whatever fixes it.

---

### 6. 🔴 Both read-first surfaces undercount `spines.md` — `S-1…S-8` / `A-1…A-6` against an actual `S-1…S-9` / `A-1…A-7`

| | |
|---|---|
| **A** | `CLAUDE.md:17-18` — *"the recurring **shapes** (**S-1…S-8**), the **anti-shapes** (**A-1…A-6**), and the index of machinery that exists and nothing calls"* |
| **A′** | `.claude/skills/session-workflow/SKILL.md:369-370` — *"`docs/spines.md` names the recurring shapes (**S-1…S-8**), the anti-shapes (**A-1…A-6**)…"* |
| **B** | `docs/spines.md:568` — `## S-9. Derivable base + sparse committed facts + fallback query — up to observation-collapse` |
| **B′** | `docs/spines.md:1128` — `## A-7. Naming a content identity inside a process` |

**Nobody suspected this one.** It is a stale range in the two documents that tell every
future session what `spines.md` contains.

**Not harmless bookkeeping — both missing entries are live right now:**
- **S-9** is load-bearing in the destination doc: `north-star.md:215` — *"Provenance and
  observation-collapse ("where is the duke?") are the same overlay shape — **spines
  S-9**"* — and `material-behavior.md:71-72` routes the ratified commit semantics through
  it (*"must **commit as appended facts (S-9)**"*).
- **A-7** — *naming a content identity inside a process* — **is** the live pass-purity
  thread. `north-star.md:42-45` records that the content layer's predicate is *"already
  material-agnostic — **and the running engine violates it**, measured"*, and
  `material-genesis-notebook.md:80-83` proposes the audit. **An agent briefed with
  "anti-shapes A-1…A-6" has been told the anti-shape that names its own finding does not
  exist.**

**Recommendation.** Mechanical at both sites. The interesting question is the process
one: **a range in prose goes stale silently every time `spines.md` grows a section**, and
this is the second read-first item in this sweep to drift by copying (see #2). Consider
dropping the count.

---

### 7. 🔴 The ROADMAP's *"First things next session"* briefs the blocker the way the ROADMAP's own Sequenced entry forbids

| | |
|---|---|
| **A** | `ROADMAP.md:4433-4437` — *"1. **stubs #29 — the REGOLITH-CHECKERBOARD blocker**… It gates every erosional number. **Brief the fix against the flux limiter / donor-cell partition in `erosion.rs::diffuse`** — *not* the clamp, *not* the time step, and **not `iso_rate`**."* |
| **A′** | `ROADMAP.md:4438-4441` — *"2. **stubs #27's heirs**… A non-capped creep operator is now plausibly **one slice for both**."* |
| **B** | `ROADMAP.md:604-607` — *"**🔑 FIRST SLICE — RATE, WITH THE CREEP LIMITER AS ITS ACCEPTANCE TEST. Do not build this as a patch beside the architecture.** journal/0116's prescription is *"a hillslope operator whose transfer stays a function of `rate × dt`"* — **that is the RATE axis**, so the blocker's fix is the axis's first real consumer"* |
| **B′** | `docs/ARCHITECTURE.md:562-568` — *"**That is the RATE axis.** So the fix is not a patch beside the architecture; it is the architecture's first real consumer"* (DECIDED, user) |

**Provenance.** B/B′ descend from a **user decision of 2026-07-26**. A is the close
block, written at the wrap **before** that decision landed (`a6bc66a` precedes
`9e50cad`) and updated only in its blockers section.

**Maximal blast radius for a stale line: this is the literal "first things next session"
list.** Both statements agree on the *register* (the flux limiter) and disagree on the
*shape of the slice*. A slice briefed from A goes green on the world and lands a patch
beside an architecture the user just ratified. **The close block also never mentions
either user decision of 2026-07-26** — the pass architecture or the bootstrap-content
removal — while claiming to *"supersede every earlier block"*.

---

### 8. 🟠 `stubs.md` § 27 still asserts the pit number `stubs.md` § 29 withdraws, 166 lines away in the same file

| | |
|---|---|
| **A** | `docs/design/stubs.md:788-790` (§ 27, live, no fixture attribution) — *"**The incision clamp leaves deep closed depressions at any multiplier above 1×** (§ 29) — 0 pits at 1×, **44 at 5×**, **148 at 45×**, deepest 112 m. **That is the blocker on the flag.**"* |
| **B** | `docs/design/stubs.md:954-957` (§ 29's opening) — *"The original sized this defect at **148 pits** using a *below-all-eight-neighbours* census… **The number was a severe undercount and the diagnosis is probably not the dominant mechanism.**"* |
| **B′** | `journal/corrections.md:2263` (#62) — the claim *"148 pits (530 on production-Medium) measures the severity of the incision-clamp defect"* is **falsified** |
| **B″** | `journal/0115…:25-27` — *"the pit count in `stubs.md` was measured on `mfd_routing`'s **small fixture**. On production-Medium the same census returned **530**"* |

§ 29's *archived original* attributes the number honestly (`:1088` *"Measured, on
`tests/mfd_routing.rs`'s **fixture**"*). **§ 27's cross-reference is the copy that
survived the correction unmarked** — no fixture label, and it still calls the clamp *"the
blocker on the flag"*, which `ROADMAP.md:656` and §29 both retired. A reader arriving via
§ 27 gets the pre-correction story with none of its warnings.

**Recommendation.** § 27's bullet needs the pointer § 29 has. This is the cheapest
high-value fix in the document.

---

### 9. 🟠 The bootstrap-content removal was DECIDED by the user and recorded in three files; `spines.md` and `stubs.md` still schedule the same content for an heir

| | |
|---|---|
| **A** | `ROADMAP.md:624-633` (DECIDED 2026-07-26, user) — *"**REMOVE THE BOOTSTRAP HISTORY CONTENT — polities, sites, ruins, the history pass**… **WHY IT IS A REMOVAL AND NOT A MIGRATION.** There is no design, no model, and no plugin-pack intent behind any of it… **Keeping it means keeping goldens that protect content nobody voted for.**"* |
| **B** | `docs/spines.md:1048` (§ 3 "Built, and nothing calls it", row **added 2026-07-26**) — the `Pregen.ledger/.overlay/.n_polities/.observe_count` row, whose *intended consumer* column reads *"**the social sim / civilization history** (stubs.md § 1 — `ruin_posts` is the loud stand-in). Until then, **a slice that cites "the history layer" as a blast radius is citing this row**"* |
| **B′** | `docs/design/stubs.md:30-40` (§ 1, live) — *"**Heir (user, 2026-07-21):** the social sim + ecology — dwarf-fortress-class civilization history… Culture-related artifacts are placeholder as a class: **do not bandaid, do not think about, until ecology is done and the social sim gets its design pass**"* |

**Provenance:** A is user-originated 2026-07-26; B is assistant-originated the same day;
B′ is **user-originated 2026-07-21** — so B′ against A is user-vs-user, resolved by
recency but **not recorded anywhere**.

**Why it matters structurally, not just verbally.** Both B and B′ are entries in the
**read-first loose-ends lookup** (CLAUDE.md item 6). Their exit contracts are *"record
what consumed it"* (`spines.md:1030-1031`: *"Leaving this list is a **good** event"*) and
*"subsumed by its heir"* (`stubs.md:4-8`). **Neither vocabulary has a word for "deleted,
and nothing replaces it"** — which is A's disposition. An agent reaching the lookup is
told to leave ruin-posts alone and wait.

**Same shape, other sites, ranked below because they are design docs rather than
read-first:** `docs/design/water.md:457-464` plans *"**pipeline surgery**"* to resequence
the history pass *"to site settlements against the final eroded world's drainage"* — and
`ROADMAP.md:434-436` names that water.md section as a **live** item — while
`ROADMAP.md:1073-1076` (user, 2026-07-25, emphatic) says *"socia/civ/eco/bio consumers
(**settlement siting et al.**) are **stubs and baggage to be replaced** and **must not
constrain the flow design at all**"* and A scopes `pregen/history.rs` for deletion. Also
`ROADMAP.md:887-908` — the draw-domain (a) entry is still sequenced as a user-owned
appearance slice, un-annotated, though `ROADMAP.md:644-646` records that A **discharges
it**; the cross-reference exists in one direction only.

---

### 10. 🟠 The ROADMAP's close block reports as owed what the same file reports as shipped — four instances

All four sit in `ROADMAP.md` § *NEXT SESSION* (`:4342`), whose header claims it
*"supersedes every earlier block"*. They read as live obligations.

| # | close block says | the corpus says |
|---|---|---|
| **10a** | `:4425-4426` — *"**ROADMAP is ~6,850 lines**… **The archive-by-status scheme is still undesigned.**"* | `:569-573` — *"**SHIPPED (b): `ROADMAP-history.md`** — archived **by STATUS, not age**… the live board went **7,210 → 4,410**"*, and the file exists |
| **10b** | `:4429-4430` — *"**Commit trailer mismatch:** CLAUDE.md § Conventions says `Claude Fable 5`… **User's call which is canonical.**"* | `CLAUDE.md:318-323` — *"**naming the model that actually did the work** — e.g. `Claude Opus 5`, `Claude Fable 5`. *This line used to hardcode `Fable 5`…*"* — the call was taken |
| **10c** | `:4420-4422` — *"**An appearance walk is owed** — creep's interbedded colluvium… Tour-map first; stations: hillslope road cut, scarp-foot apron…"* | `:4442-4443`, **24 lines below in the same block** — *"3. ~~The appearance walk (creep)~~ — **DONE 2026-07-26, journal/0115.**"* (and `journal/assets/0115-station-b-colluvium-cut.png` exists) |
| **10d** | `:4349` — *"### Shipped (journals **0108–0114**; corrections **#53–#60**…)"* | the same block's blockers cite `journal/0116` and `corrections #63` at `:4393-4394`; the day shipped through **0119** and **#65** |

**10a is the one that bites:** an agent reading *"still undesigned"* may design it a
second time — this project's named characteristic failure (`CLAUDE.md` item 0b:
*"re-inventing a mechanism next to the one it already built"*). 10b invites a settled
user question to be re-raised.

**Note for the wrap ritual:** three of these four are *the wrap's own § Owed not
reconciled against the wrap's own § First things* — a within-artifact check the ritual
does not currently make.

---

### 11. 🟠 `journal/0114` says the `distribution_fill` bound "has been re-derived"; 17 lines later it says four falsifiers needed no change — and the tree agrees with the second

| | |
|---|---|
| **A** | `journal/0114…:392-395` — *"That test's bound was a bedding-thickness snapshot and failed here; **it has been re-derived** to the size of the defect it actually names (independent rounding gives *exactly zero* mixed spans), rather than lowered until green"* |
| **B** | `journal/0114…:409-411`, same entry — *"**Because the flag ships off, four of those five needed no change at all** — the shipped world is untouched, so **every existing falsifier keeps its original bound and its original strength**."* — with `distribution_fill` listed as row 1 of those four at `:403` |
| **ground truth** | `crates/dc-worldgen/tests/distribution_fill.rs:71-75` — `assert!(mixed > 100, "only {mixed} mixed spans — the sieve is still rounding each unit independently")`, under the **original** doc comment at `:43-45`. The re-derivation landed in `b3dd699` and was reverted by `91317c6` (*"every existing falsifier keeps its original bound"*), which is B. |

**The corrections #65 shape at close range** — a claim and its own refutation in one
entry, 17 lines apart rather than 400.

**Blast radius: moderate but sharp.** `journal/0114` is one of two entries the close
block names as *read first* (`ROADMAP.md:4344`), and A is written as a **worked example
of the tolerance doctrine** — so it is likely to be cited as precedent for a
re-derivation that is not in the tree. Anyone grepping the pattern finds `mixed > 100`.

**Not filed as a correction.** The entry's load-bearing claim (B) is true and matches the
code; A narrates a step undone by a later, better decision **in the same slice**. What is
missing is a sentence saying so — a pair to reconcile, not a claim to falsify.

---

### 12. 🟠 Two read-first surfaces state the trusted/untrusted tiered backend as live design; `north-star.md` § Deviations says it is deferred and, in its capability half, **explicitly not the model**

| | |
|---|---|
| **A** | `CLAUDE.md:11-13` — *"**safely moddable by untrusted third parties (native `abi_stable` for trusted, WASM sandbox for untrusted, one authoring shape)**"* |
| **A′** | `ROADMAP.md:29-32` — *"**untrusted-third-party safe** via a tiered backend… native `abi_stable` for trusted/first-party (incl. runtime), WASM sandbox for untrusted (gen-tier…)"* |
| **B** | `docs/design/north-star.md:353-357` — *"**The trusted/untrusted safety split + the ABI/WASM spike are DEFERRED, not gating** (user)… **the whole trusted/untrusted discussion is paused until we are anywhere near having modders.**"* |
| **B′** | `docs/design/north-star.md:365-373` — *"**NO capability tiering — a mod can author ANYTHING, including a field pass** (user, emphatic)… *"Do not bake in any limitations on a trust model I DO NOT RATIFY"*… is **EXPLICITLY NOT THE MODEL and must not shape any design.**"* |

**Provenance is decisive: B and B′ are user-originated and emphatic; A and A′ are
assistant summaries.** Not a tie.

**Two halves, and I want to avoid over-claiming.** The **capability tiering** (*what a
mod may author*) is flatly retired by B′. The **backend** mapping is not retired — it is
**deferred and paused**, and `north-star.md:237-240` calls it *"deferred product
infrastructure, not a live architectural fork"*. A and A′ present both as current
properties of the destination, with no deferral marker — while the live arc's framing is
*"assume mods can do anything."*

---

### 13. 🟡 The ROADMAP still lists the ABI/WASM spike as de-risk item (3) that *"locks the SDK shape"*; `north-star.md` says it is not gating

| | |
|---|---|
| **A** | `ROADMAP.md:42-43` — *"**De-risk before committing the arc**… (3) the ABI/WASM boundary research spike… — **locks the SDK shape.**"* |
| **B** | `docs/design/north-star.md:324-329` — *"**AMENDED — NOT GATING (user, 2026-07-24…): this spike no longer blocks the arc.**… **the *shape* does not require it first.**"* |

`north-star.md:321-323` carries the same item and **was amended in place**; the ROADMAP
copy was not. Mitigating: A says *"sequenced, not urgent"*. Listed because *"locks the
SDK shape"* is the precise claim B withdraws, and the live arc is an SDK-shape arc.

---

### 14. 🟡 `stubs.md` #29's retained original names weathering as a phase that can lower a cell; `journal/0119` shows it cannot — and names an unlisted fifth

| | |
|---|---|
| **A** | `docs/design/stubs.md:1081-1083` — *"**four later phases in the same epoch can lower a cell past it** — **weathering**, hillslope creep, wave attack and eolian deflation all run after incision."* |
| **A′** | `journal/corrections.md:2009-2011` (#56) — the same four-phase claim, unannotated |
| **B** | `journal/0119…:21-32` — *"**it doesn't**. `WeatherApply::apply` is `*r -= q; *h += q`, **mass-neutral**, and `surf = r + h` is **unchanged to the bit**… **Weathering cannot lower a cell.**"* — and *"a fifth post-incision phase nobody had listed at all — **isostasy**"* |

**My brief suspected #29's two blocks contradict rather than supersede. They do not** —
the signposting at `:954` (*"⚠ READ THIS BLOCK BEFORE THE ORIGINAL ENTRY BELOW"*),
`:956-957` and `:1077-1078` is correct and is the best in the file. What the preamble
does not say is that **one of the four named mechanisms is impossible, not merely
non-dominant**, and that the roster is incomplete in the other direction. `:1077-1078`
explicitly invites the heir to treat the original mechanism as *"plausible and probably
contributes"* — a heir taking that at face value hunts a weathering→clamp interaction
that cannot exist.

---

### 15. 🟡 `material-behavior.md` § 5's RATE bullet and `flow.md` still describe ORDER as topo-sort

| | |
|---|---|
| **A** | `docs/design/material-behavior.md:367-369` — *"the temporal-resolution knob **topo-sort alone does not give**… it is the RATE axis, ***composed with* topo-sort**, never replaced by it."* |
| **A′** | `docs/design/flow.md:459` — *"§ 5 gives the scheduler **two axes — ORDER (topo-sort)** and **RATE**"* |
| **A″** | `docs/design/flow.md:715-716`, under a bold **RATIFIED** heading — *"**RATIFIED.** `material-behavior.md` § 5 gives the scheduler **ORDER (topo-sort)** and **RATE**"* |
| **B** | `docs/design/material-behavior.md:345-346`, **immediately above A** — *"**🔴 SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD.**"* |

**Confirms half a suspicion and refutes the other half.** The RATE bullet does still say
*"composed with topo-sort"*. The **WINDOW** bullet does **not** — it is clean. `flow.md`'s
two are lower-stakes (they describe what § 5 said, as setup for adding WINDOW, and
`:729-730` records the outcome correctly) — but `:715` gives an incorrect scheduler model
a **RATIFIED** stamp it never received.

---

### 16. 🟡 `session-workflow` teaches revision-token declaration practice; the ratified direction retires the revision chain

| | |
|---|---|
| **A** | `.claude/skills/session-workflow/SKILL.md:831-834` — *"**Where one plane carries several revisions per epoch, declaring the revision you consume pins ONE SIDE ONLY. Pin the other side with an anti-dependency on the NEXT revision of that plane — never the last.**"* |
| **B** | `ROADMAP.md:611-613` — *"after RATE — **(b)** authored order + the validator (**and with it the retirement of the revision chain**); **(c)** the open resource vocabulary, `DeepAxis` **deleted**"* |

**Labelled honestly: this may not be a contradiction.** A is true of the code today and
stays true until (b)/(c) land, and it is framed as a lesson about *trusting an audit's
prescribed fix*, which is scheme-independent. **What is missing is the sentence saying
the scheme it teaches against is scheduled for deletion.**

Same class, deliberately **not** flagged: `docs/spines.md:363-512` (S-6) describes the
topo-sort instances accurately **as current code**, which is that document's stated
remit. It is this sweep's clearest example of *both statements being true of different
things*.

---

### 17. 🟡 The ROADMAP split left 16 pointers to a section no longer in the file, and 9 pointing the other way

- `ROADMAP.md` holds **16** live references of the form *"see § Shipped"* — `:147, :769,
  :926, :1030, :1245, :1303, :1473, :1835, :1882, :1965, :2193, :2490, :2612, :2728,
  :3264, :3338` — while `ROADMAP.md:8` now reads `## Shipped → ROADMAP-history.md`.
- The reverse also happened: `ROADMAP-history.md:51` — *"(**Sequenced above**…)"* —
  `:2144` — *"(**Observed below**)"* — plus `:495, :541, :568, :753, :1420, :1474, :1665`.
- One orphan predates the split and is now doubly wrong: `ROADMAP.md:511` —
  *"*(Superseded by the session-close block **above**…)*"* — sits at line 511 while the
  only close block is at `:4342`, **below** it, and the six superseded blocks it may have
  meant were archived out (`ROADMAP.md:569-570`).

**Confirms my brief's suspicion #3, and it is larger than suspected.** Mitigating:
`ROADMAP.md:8` and `ROADMAP-history.md:4` both explain the split. **This is a
navigational defect, not a factual contradiction** — ranked here for exactly that reason,
but worth a mechanical pass **because** the archive's stated purpose (`:15-18`) is to
stop the board being grepped instead of read, and dangling pointers are what force the
grep.

---

### 18. ⚪ Two user statements six days apart on whether to design the plugin mechanism now — **I cannot adjudicate; user call**

| | |
|---|---|
| **A** | `docs/design/ideas.md:468-470` (user, 2026-07-20) — *"So: **do not design a plugin-pass mechanism yet.** Keep writing passes; **the API is the residue they leave behind.**… in that order, **later**."* |
| **B** | `docs/ARCHITECTURE.md:490-492` (user, 2026-07-26) — *"**THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP, EMPHATICALLY.**"* + `ROADMAP.md:582-618` sequencing it as the live arc |

A reading that dissolves it: B constrains *how passes are written now* (no pack-specific
names in the engine); A defers *a loader/SDK/versioning surface*.
`north-star.md:246-253` (user, 2026-07-23) sits between and leans toward B. **Reported
because A carries no supersession marker of any kind** — same document as #4. If the
reconciling reading is intended, the sentence saying so does not exist.

---

### 19. ⚪ Justifications whose premise is now contested, recorded without a recommendation

Two ratified decisions whose *stated reasons* invoke systems the 2026-07-26 doctrine says
do not exist. **Both have independent second arguments in the same section, so neither
decision falls** — this is about the recorded reasoning, not the call:

- `docs/ARCHITECTURE.md:352-368` — *"**The content set is frozen at world creation** —
  DECIDED 2026-07-22 (user)… **Why:** **the world's history is going to be a
  dwarf-fortress-class social simulation**"* — against `CLAUDE.md:321`. The independent
  re-derivation argument is at `:387-412`.
- `docs/design/worldgen.md:12-17` — *"the **Dwarf-Fortress-quality histories** we want…
  only come from actually running the process forward. **That requires a finite coarse
  world.**"* — the first-named reason for boundedness. The independent closure argument
  (*"poles, tropics, winds, currents"*) is in the same paragraph.

Also: `docs/ARCHITECTURE.md:414-420` promotes into the read-first tier a seam count
allotting *"**social sim 5**"* of 34 seams — an API surface shaped by a consumer
`ROADMAP.md:1073-1076` says *"must not constrain… at all"*.

---

## What I did NOT find — nulls, reported because a null is a result

- **A finding I published from a sub-sweep and then withdrew on re-reading.**
  `journal/0116…:135` labels a row *"calibrated, `iso_rate` 0.50 **(production)**"*, and
  it was reported to me as a false production label (the calibration ships **off**;
  `journal/0114:227` — *"`production_config` sets `calibrated_rates: false`"*). Reading
  the whole table, the ladder varies `iso_rate` 0.50 / 0.25 / 0.00 and *(production)*
  marks **which `iso_rate` ships** — a defensible reading. **Downgraded to a terminology
  hazard, not a false statement.** It is real evidence for one thing, though: the corpus
  uses *"production"* for the seed+extent in `stubs.md:961` and for the default **config**
  in `ROADMAP-history.md:49`, in live documents, and has never decided which. **That
  undecided term is the mechanism behind #5.**
- **`stubs.md` #29's two blocks do not contradict each other** (see #14).
- **`material-behavior.md` § 5's WINDOW bullet is clean.** Suspected; refuted.
- **`ROADMAP.md`'s blocker entry (`:652-700`) is fully reconciled** with journal/0116 and
  corrections #61/#62/#63. The best-maintained long entry I read.
- **The `530` collision is a coincidence.** `flow.md:433` and `journal/0113:189` use
  **530** for *land cells with catchment > 100*; `corrections.md:2266` uses **530** for
  the production-Medium pit census. Different quantities, both labelled. Checked because
  it looked like finding-shape.
- **`docs/spines.md` S-6 is not a finding** — see #16.
- **`material-genesis-notebook.md:103-106`'s *"it dissolves the topo-sort problem
  entirely"*** concerns a *different* topo-sort problem (read-set unions when materials
  are added). Not flagged.
- **`docs/design/octree-substrate.md`'s social/statistical mentions are the GOOD shape** —
  `ROADMAP.md:196-198` and `water.md:747-750` record the discipline explicitly
  (*"recorded as **"this pattern is intended to extend there"** rather than designed for
  in advance"*). Noted so a future sweep does not re-flag it.

---

## Coverage — read, skimmed, and not opened

**Read in full:** `.claude/skills/doc-topology/SKILL.md`; `CLAUDE.md`;
`docs/design/north-star.md` (379 ll.); `docs/ARCHITECTURE.md` (484 ll.); `journal/0119`;
`journal/corrections.md` #61–#65; `material-behavior.md` §§ 5–6; `stubs.md` §§ 1, 14, 27,
29 (both blocks); `ROADMAP.md` §§ In-flight head, Sequenced head, pass-architecture entry,
bootstrap-removal entry, blocker entry, whole close block; `ideas.md` §§ Pass cadence,
Extensibility; `ecology.md` head + section index; `worldgen.md:1-80`;
`crates/dc-worldgen/tests/distribution_fill.rs:40-80`;
`crates/dc-worldgen/examples/coal_charcoal_probe.rs` head.

**Read in part (whole sections around hits):** `spines.md` (`:1-60, :360-500`, full § 3
table, full heading index); `session-workflow/SKILL.md` (`:800-850` + greps);
`flow.md` (`:440-490, :700-740`); `material-genesis-notebook.md` (`:80-130`);
`journal/0114` (`:370-424`); `water.md` (`:450-470, :690-760`); `visuals.md:470-500`;
`materials.md:335-350`; `geology.md:155-170`; `things-that-will-happen.md` (two blocks);
`docs/audits/2026-07-22-seam-inventory.md` (§ 4); `S17-deep-cell-inventory-results.md`
(`:50-90, :335-350`); `docs/audits/2026-07-22-entry-species-probe.md:1-70`;
`ROADMAP-history.md` (head + cross-reference greps).

**Grep-swept across the whole corpus:** topo-sort · `DeepAxis` · `reads_prev` ·
`AmbiguousWriters` · the pit census · the four-phases claim · S- and A- ranges ·
`0x0D5EED572026` / `0x0B0A57EE0059` / `1337` / `Extent::*` · `fn production_*` /
`fn golden_*` · `const SEED` in every example and integration test · polity / site /
ruin / socia / civ / ecology / evolution.

**NOT OPENED AT ALL — declared so the gap is legible:** `docs/API.md`;
`docs/rendering/PIPELINE.md`; `docs/design/{light,ores,bodies,tectonics,knowledge,
earth-processes,orogeny-recon,voxy-dh-recon}.md`; most of `docs/spikes/` (S2, S9–S16,
S19, S20 bodies); most prior `docs/audits/`; every journal entry except 0114 and 0119
(0066/0090/0104/0107/0110/0115/0116 touched at grep level only);
`.claude/skills/{spine-audit,wrap,tour-map}` beyond greps; **`ROADMAP.md` § Observed
(~1,970 ll.)** and the bulk of § Sequenced; **`ROADMAP-history.md` bodies (2,297 ll.)**.

**The largest unswept surface is `ROADMAP.md` § Observed.** It is the section the archive
explicitly could not reduce (`ROADMAP.md:576-579`: *"the remaining volume lever is
`Observed` (1,969 lines), which is **not archivable by status** — an Observed entry is
live by definition"*), and it is where a superseded observation is most likely to still
assert itself. **A future run should take Observed as its whole spine.**

---

## Verdict on the SKILL — first run

**The method works, and it found things nothing else would have.** Five of the top eight
findings (#1, #2, #5, #6, #10) were **not** on the suspicion list I was handed, and #1
and #5 are the two most expensive things in the document. The rules that did the work:
**"read whole sections, do not grep-and-conclude"** — #2 is invisible to grep because
both halves use the same words in the same order — and **"file:line on both sides"**,
which killed three leads that looked like findings and were not (the `530` collision,
spines S-6, and the `journal/0116` *(production)* label I withdrew after re-reading its
table).

**Three concrete changes to `SKILL.md`:**

1. **Add a sixth shape: *a summary that outran its source*.** Findings #2, #6, #12 and
   #13 are one mechanism — **a paraphrase of doc X living in doc Y, where X was later
   amended in place and Y was not.** It is structurally different from
   "superseded-but-still-asserted", because **the summary was never wrong when written
   and no decision was ever reversed** — which is exactly why no reviewer catches it. The
   procedure is finite: *for every read-first doc, take each paraphrase of another doc and
   diff it against the paragraph it paraphrases.* That would have caught four of my top
   eight mechanically.
2. **Name the close block as a mandatory target.** `ROADMAP.md`'s close block is
   simultaneously the highest-traffic artifact in the corpus (*"first things next
   session"*) and the one nothing audits — written at a wrap, stale the moment work lands
   after it. Findings #7 and #10 are five instances in one artifact. § Scope says
   *"`ROADMAP.md`'s live board"*; it should say *"the live board **and the current close
   block, checked against itself**."*
3. **Bound the scope by artifact count, not by noun.** *"Every noun the batch touched"* is
   not finite — this batch touched twelve journals and forty documents, and I made a
   judgement call about where to stop. A repeatable version: *the read-first set (6 files,
   always) + the merged batch's journal entries + every doc those entries cite by name.*
   Countable **in advance**, which is what makes a sweep repeatable rather than heroic.

**One thing I would remove:** the instruction under shape 3 to *"grep `ideas.md` and every
user sketch block"* sits oddly beside the top-level rule *"grep cannot do this."* Shape 3
is the one shape where grep **is** the right instrument — user sketches are marked and
enumerable. Say so, rather than leaving two rules in tension.

**One thing I would add to § Rules:** *when a sub-sweep hands you a pair, re-read both
sides at source before publishing it.* I did, and it changed one finding's verdict. The
skill already says an agent's mechanism is a hypothesis; it should say so about the
sweep's own helpers.

**Cost:** zero cargo invocations, no build lock taken, read-only except this file. About
3,400 lines read in full, ~2,000 in part, three parallel readers. **That is affordable
after every arc**, which was the design goal.

---

## Numbered artifacts

- **`journal/0120` — deliberately NOT written; the number is unspent.** This sweep found
  bookkeeping drift, four un-propagated supersessions and one genuine user-vs-user
  conflict. That is a *result*, not a *narrative*, and the story it would tell — *the
  corpus outran its own summaries* — is told better by `journal/0119`. **Recorded here so
  the gap reads as a decision, not as a lost entry.** If #1 (the ecology doctrine) goes to
  the user and produces a ruling, **that** is the entry, and it should take 0120.
- **`corrections.md` #66 — deliberately UNSPENT.** Three candidates were declined.
  #11 (`journal/0114`'s re-derivation) is a pair to reconcile — its load-bearing half is
  true and matches the code. #14 (weathering in the four-phase list) is a partial already
  recorded in `journal/0119`. **#5 (the coal census world label) is the one real
  candidate** and I am *not* taking it, for a stated reason: the honest correction must
  say what the numbers are on the right world, and answering that needs a cargo run this
  sweep was scoped out of. **Filing "the label is wrong, the value is unknown" would put a
  half-measurement in the file the project reads to avoid re-deriving.** Per the skill: *a
  contradiction is not automatically a correction.*
