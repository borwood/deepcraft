# S2 — Physical process design set (doc-topology baseline sweep)

> ## 📋 DISPOSITION — applied 2026-07-29. **12 of 12 applied; 0 escalated.** This was the
> cleanest slice of the nine.
>
> | # | state |
> |---|---|
> | **F1** `geology.md` 4th topo-sort site | **✅ APPLIED** — struck with pointer; **corrections #65's site list amended** to name `geology.md`, `material-behavior.md` § 5 RATE and `flow.md` § 11.1 (it enumerated three sites and there were five). |
> | **F2** `tectonics.md` has no banner | **✅ APPLIED** — `⚠ READ flow.md FIRST` banner at the head, naming what § 7.3 lost (`recv`'s kind, the corridor mechanism, "the collapse carving source") and what survives. **§ 7.3's body is not rewritten**, per the recommendation. |
> | **F3** `geology.md` "no geotherm in this project" | **✅ APPLIED** — dated banner on the SHIP UPDATE block; all three false claims named; the measurement untouched. |
> | **F4** Phanerozoic register unflagged in `tectonics.md` § 0 | **✅ APPLIED** — mirrored **by pointer**, in the § 0 prior *and* the top banner (the prior sits under "do not re-derive", the highest-risk framing). |
> | **F5** `water.md` consequence 1 has no tier qualifier | **✅ APPLIED** — tier qualifier added beneath the consequence (present tier holds; deep tier falsified three ways). **The user-ratified DECIDED above it is untouched.** Open question 5 marked answered by journal/0098. |
> | **F6** `52 MiB` live denominator | **✅ APPLIED** — both sites restated against the measured **162.57 MiB** (S19), with the S9 provenance kept. The `:48` S9 measurement itself is untouched. |
> | **F7** 1,245 vs 1,255 | **✅ APPLIED** — journal/0113's reconciling parenthetical carried into `flow.md` beside the table. |
> | **F8** `light.md` poses a DECIDED question as open | **✅ APPLIED — and this was the brief's flagged live item.** § 5's structural note and § 10 item 3 both narrowed: saturation + temperature are one machinery (condition-fields, DECIDED 2026-07-24); **light specifically is still open**, and the narrowing names *why* (max-plus, not diffusive; derived-never-stored). Verified at `material-behavior.md` § 14 and `flow.md` § 7 at source. |
> | **F9** `water.md` paleo-channels | **✅ APPLIED** — back-pointer at `:398` to its own refutation and to `flow.md` § 2, stating that the conclusion became true again for a different reason. |
> | **F10** 0.85 vs 0.8× rebound | **✅ APPLIED** — `earth-processes.md` now cites `tectonics.md`'s derivation and states the 34 % swing. |
> | **F11** `mfd_exponent = 4.0` "shipped, on by default" | **✅ APPLIED** — grep-arriver pointer to § 2.6.2. |
> | **F12** `550²` vs `297,025` — *"I cannot attribute this"* | **✅ RESOLVED AT SOURCE** — read `crates/dc-worldgen/src/deeptime/field.rs`: `DEEP_MAX_WIDTH = 550` is a **cap** (`cell_m = (extent_m / DEEP_MAX_WIDTH).max(DEEP_CELL_M)`), so Medium lands on 545² = 297,025. Both numbers correct; reconciling clause added. |

**Watermark commit:** `f652b60`. **Every `file:line` below was read at `f652b60`**, in the
worktree `.claude/worktrees/agent-a06bb62264eaface5`. Code citations are at the same commit.

**Slice, read IN FULL:** `docs/design/tectonics.md` (953) · `docs/design/flow.md` (817) ·
`docs/design/water.md` (813) · `docs/design/earth-processes.md` (411) ·
`docs/design/geology.md` (418) · `docs/design/light.md` (243). **Total 3,655 lines.**

**Doctrine read for consistency:** `CLAUDE.md` (worktree copy), `docs/design/north-star.md`,
`.claude/skills/doc-topology/SKILL.md`.

**Docs NOT opened** (so a null from me is not a null for them): `material-behavior.md`,
`materials.md`, `ecology.md`, `worldgen.md`, `ARCHITECTURE.md`, `spines.md`, `ROADMAP.md`,
`docs/spikes/S12-results.md`, `S9*`, `S10*`, `S11*`, `S19*`. Where I cite one of those it is a
**targeted grep at `f652b60`**, stated as such, not a read.

**Nothing here is resolved.** Per the skill: report the pair, say which side is better-supported,
stop. Recommendations are labelled as recommendations.

---

## 1. Findings — 12 pairs, ranked by blast radius

### F1 — `geology.md` is a FOURTH, unstruck site of corrections #65's falsified claim (reads/writes GENERATE pass order)

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — still asserted as current** | `docs/design/geology.md:29-32` | *"**Processes are plugins too.** A creation vector … registers as a pass declaring: phase …, what it READS, what it WRITES. **Declared reads/writes let the pipeline topo-sort passes and detect cycles** — the geology→soil→ecology→history coupling-order problem **becomes a graph problem instead of a hand-maintained list.**"* |
| **B — falsified, user-decided** | `journal/corrections.md:2507` (heading of #65) | *"The deep-time phase ORDER falls out of the declared reads/writes, and this replaces a hand-declared canonical order" … **falsified 2026-07-26**"* — and it names its sites: `material-behavior.md` § 5, `journal/0090`, `north-star.md` § Passes. |
| **B (doctrine)** | `docs/design/north-star.md:206-210` | *"~~The runner topo-sorts by declared reads/writes and rejects conflicts.~~ **SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD**; `{reads, writes}` became the **validator**, not the generator."* |
| **B (read-first)** | `CLAUDE.md:12-13` (worktree) | *"**pass ORDER is authored per world** (ARCHITECTURE.md, DECIDED 2026-07-26)."* |

**Provenance:** side B is **user-originated** (DECIDED 2026-07-26, user; recorded because an
assistant reconciliation had superseded a user sketch — the correction that produced the *"a
user-originated design may not be superseded by an implementation slice"* rule). Side A is
assistant-authored design-backbone prose from 2026-07-18. **Not a tie.**

**Why this is #1 by blast radius.** corrections #65 enumerated **three** sites and each was
struck. This is a **fourth**, in a design doc, unstruck, phrased differently enough that a grep
for the struck sentence never finds it — *"instead of a hand-maintained list"* is precisely the
half that was falsified. This is the exact claim that "cost an architecture" (CLAUDE.md § Read
first 1b), and the corpus currently still teaches it in one place.

**Recommendation (labelled: recommendation).** Strike-with-pointer at `geology.md:29-32`, and add
`geology.md` to corrections #65's site list in the same edit — the reciprocity obligation lands on
the writer of the correction (CLAUDE.md read-first item 5).

---

### F2 — `tectonics.md` carries NO staleness banner, and its § 7.3 still specifies the retired receiver tree AND a "collapse carving source"

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — asserted as a live deliverable** | `docs/design/tectonics.md:551-557` | *"**`DeepField` exports drainage instead of dropping it.** … Add: final-chapter **`recv` (D8 receiver)**, `area`/discharge, and the lake mask … Consumers: (a) the **3e-2 frozen macro drainage topology** (decision 1 … the **river-conditioning corridor mechanism** inherits it) — **replacing the stale pregen chord network … as the collapse carving source, one authority**"* |
| **B — RATIFIED supersession** | `docs/design/flow.md:109-121` (§ 2.1 *Why the receiver tree dies*) | *"`flow_to: Option<u32>` is **one out-edge per cell**. A tree can represent convergence. It **structurally cannot represent divergence**. … Not 'poorly modeled' — **unrepresentable**."* |
| **B** | `docs/design/flow.md:335-339` | *"**`recv` changed kind.** It is now the **argmax share** … it can no longer be read as 'where the water went', and it is now a summary in ARCHITECTURE.md's sense. **Continuation (e) should retire it**, not merely supersede it."* |
| **B** | `docs/design/flow.md:25-27` | *"**There is no separate 'carving.'** … **Any mechanism that draws a channel instead of spending a budget is a second model of the carve and is forbidden by this document**."* |
| **B** | `docs/design/flow.md:558-566` (§ 6 *What this retires*) | retires `RiverSeg`/`carve_rivers`, `Cell::{flow_to, river, discharge}`, and *"`earth-processes.md` § 3e-2, **expression half**"*. |

**The structural half of the finding — a missing back-pointer, not just a stale sentence.**
Grepped at `f652b60`: `docs/design/tectonics.md` contains **zero** occurrences of `flow.md`,
`SUPERSEDED`, `journal/0111`, or `denudation` (pathspec: `grep -n "flow\.md\|flux record\|SUPERSEDED\|journal/0111\|denudation" docs/design/tectonics.md` → no output).
Meanwhile **both its siblings carry the banner**: `water.md:3-13` (*"⚠ READ `flow.md` FIRST"*) and
`earth-processes.md:292-302` (*"⚠ SUPERSEDED IN PART — 2026-07-25, see `flow.md`"*). `tectonics.md`
is the **largest doc in the corpus** (953 lines, per `corpus-knowledge-notebook.md:66`) and the one
whose § 7 owns drainage export — and it is the only one of the three a cold reader would open
without ever learning that flow.md exists.

The code already knows: `crates/dc-worldgen/src/deeptime/field.rs:507` documents the flux record as
*"the representation that supersedes [`Self::recv`]"*, and `erosion.rs:3004` names *"(e), the
retirement slice"*. **The doc is the last holder of the retired shape.**

**Better-supported side: B**, decisively — flow.md is RATIFIED 2026-07-25 (user, *"standing
blessed"*, `flow.md:3`) and the retirement is reflected in shipped code. **Both are
user-ratified**, so this is a supersession, not a conflict of authority: tectonics.md's banner
(RATIFIED 2026-07-20) predates flow.md by five days.

**Recommendation (labelled).** A `> ⚠ READ flow.md FIRST` banner at the top of `tectonics.md`
naming what survives (chapters, columns, isostasy, the chapter table, punctuation, § 8 events) and
what its § 7.3 lost (the `recv` export shape, the corridor mechanism, "the collapse carving
source"). **Do not rewrite § 7.3's body** — its § 7.1 diagnosis of the pregen-chord staleness is
still true and still cited.

---

### F3 — `geology.md` states "there is **no geotherm** in this project", coal promoting on overburden, and `stubs.md` #14 live — all three superseded 2026-07-24

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — asserted as current** | `docs/design/geology.md:160-164` (SHIP UPDATE 2026-07-22) | *"`promote_coal` had been promoting peat on seam **thickness** … **It now promotes on each unit's own overburden.** The ladder still cannot be built — **there is no geotherm in this project**, and 13 of 35 382 peat-derived units lie under 50 m of section — so **`stubs.md` § 14 carries the threshold with a geotherm as its heir.**"* |
| **B — shipped** | `docs/design/stubs.md:367` (grepped) | *"### 14. coal-rank-is-burial-depth-with-no-geotherm — **RETIRED 2026-07-24 (journal/0093, the geotherm field pass)**"*; `stubs.md:369`: *"the geotherm (the first §5 field pass, `deeptime/geotherm.rs`) supplies a real …"* |
| **B — code** | `crates/dc-worldgen/src/deeptime/biotic.rs:182` | `pub const COAL_ONSET_C: f64 = 22.0;` — and `biotic.rs:127`: *"**now a real °C, no longer this depth wearing degrees**"*. `recorder.rs:513`: `pub fn promote_coal(&mut self, col: BurialColumn, onset_c: f64)`. |
| **B — design** | `docs/design/material-behavior.md:1025` (grepped) | *"**The geotherm — the FIRST field pass** (DECIDED) … **writes** the `temperature` field as a per-cell **geothermal gradient**"* |

**Three separate falsehoods in one paragraph:** (i) the geotherm exists (`deeptime/geotherm.rs`);
(ii) coalification is **temperature**-gated at 22 °C, not overburden-gated; (iii) `stubs.md` #14 is
**retired**, so the pointer sends a reader to a retirement notice for a heir that already arrived.

**Reciprocity:** grepped `stubs.md` for `geology.md` — the #14 retirement carries **no** back-pointer
to the doc whose contract it retired. This is the *"a one-directional pointer is not a pointer"*
shape, in a paragraph the geology doc itself calls a precedent worth keeping.

**Note the irony, which is load-bearing:** the *same paragraph* (`geology.md:155`) coins the
precedent *"a resolution argument is a claim about the quantizer, and quantizers change"* — and
then the paragraph beside it became the corpus's own example of a claim outliving its premise.

**Better-supported side: B** (shipped code + a retired stub + a DECIDED design entry).
**Recommendation (labelled):** a dated banner on the SHIP UPDATE block — the block is testimony
about 2026-07-22 and per CLAUDE.md read-first item 5 the *body* is what stands and the *banner* is
what moves. Do not delete the measurement.

---

### F4 — The Phanerozoic register is asserted as live calibration in `tectonics.md` § 0 with no flag; `earth-processes.md` flags the identical register as **not honoured by ~1000×**

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — asserted, unflagged, in a "do not re-derive" priors section** | `docs/design/tectonics.md:58-60` | *"**Calibration (RATIFIED 2026-07-19)**: the Phanerozoic register — the recorded span is ~500 Myr, so at 200 iterations **one iteration ≈ 2.5 Myr**. Ages must be labeled before persistence/knowledge commits them as facts."* (restated `:181-182`, `:231-232`) |
| **B — flagged at the source entry** | `docs/design/earth-processes.md:344-361` | *"**FLAG 2026-07-26 — the register is stipulated and the engine's rates do not honour it** (journal/0111, corrections #56, stubs #24). At 200 epochs this register means **2.5 Myr per iteration**, and **nobody had ever divided the engine's metres-per-iteration constants by it.** … catchment-averaged denudation **0.0110 m/Myr** — **9× slower than the slowest surface ever measured on Earth** … stripping **5.48 m** … where a real craton strips **5–10 km**."* |
| **B (doctrine)** | `CLAUDE.md` § Conventions, *A CLOSED SYSTEM CANNOT DETECT ITS OWN SCALE ERROR* | the same event, elevated to project law. |

**Both statements are true of different things** — and *that sentence is exactly what is missing*.
The register **is** ratified; the engine **does not** honour it. `earth-processes.md` says so at the
site of the ratification. `tectonics.md` § 0 reproduces the ratification **as a prior explicitly
labelled "swept before writing — do not re-derive"** (`tectonics.md:38`), which is the highest-risk
possible framing: a reader is instructed *not* to check it.

**Downstream in the same doc, unflagged:** `tectonics.md:444-447` derives *"total rock processed per
metre of peak lowering ≈ **6.7 m**"* and `:936` makes *"relief persists for hundreds of Myr"*
falsifiable via a spike criterion — both reason from an iteration↔Myr mapping the engine does not
implement. `tectonics.md:232` also binds the chapter age-labelling (`chapter c spans [500 − c·(500/K), …]`)
to the same register.

**Better-supported side: B** — it is measured **against the literature** (Portenga & Bierman 2011;
McMurdo/Atacama band), which is the one class of error CLAUDE.md says an internal audit is
structurally blind to. **Recommendation (labelled):** the FLAG at `earth-processes.md:344` should
be mirrored, by pointer not by copy, into `tectonics.md:58-60`.

---

### F5 — `water.md`'s DECIDED consequence "the water table is READ, not modelled … never has to be traced from a source" vs the shipped, stored, globally-solved `dc:field/head`

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — under a DECIDED heading, unstruck** | `docs/design/water.md:156-161` | *"**The water table is READ, not modelled.** … **Nothing stores 'the water table'**; it is a query over the saturation field. This is why S9 could classify it as a *bounded relaxation* (haloable, C-refinable) rather than an advective field: **it equilibrates locally and never has to be traced from a source.**"* (restated in the priors sweep, `water.md:378-384`) |
| **B — built, and recorded in the doc that supersedes water.md** | `docs/design/flow.md:202-204` | *"Solved as steady `∇·(T ∇h) = 0` by **Gauss–Seidel with alternating forward/reverse raster sweeps** — deterministic, and **information crosses the whole grid in one sweep** instead of diffusing a cell at a time."* |
| **B** | `docs/design/flow.md:194-200` | *"**confined** cell … free, and **UNCAPPED** — the artesian degree of freedom … **its head is whatever the material transmits to it from elsewhere.**"* |
| **B** | `docs/design/flow.md:172-176` | *"Regional aquifers and karst conduits **genuinely cross surface drainage divides** … `3e-2` decision 1's 'never crosses a drainage divide' is hereby **qualified: it binds FREE/surface refinement only.**"* |
| **B — stored** | `docs/spines.md:1083` (grepped) | *"**`DeepField::head` — the exported `head` condition-field** (`dc:field/head`, metres of hydraulic potential per cell), added 2026-07-25 by journal/0098"* |

**The honest scope.** water.md's claim is written without a tier qualifier and therefore reads
across both. At the **present tier** (`sat.rs`, S11's 4–11 cell halo) it survives intact. At the
**deep tier** it is falsified three ways: the potential **is stored** (a `DeepField` plane), it
**is** traced from sources (Dirichlet at sea stand / lakes / anchored streams), and confined flow
is explicitly **non-local**. *That distinction is exactly the sentence neither doc contains.*

**Reciprocity gap:** `water.md`'s own banner (`water.md:3-13`) enumerates what survives —
*"the DECIDED two-regime model …, persist bodies / derive voxels, the measured S11 results,
corrections #14, and the § Session-capture karst analysis"* — and consequence 1 is on **neither**
list. A reader is left with an unqualified DECIDED. Same for `water.md:601-602`, open question 5
(*"Where does the deep-time water field live relative to the A/C tiers — given S9 says it relaxes
and is therefore haloable?"*), still carried as **unanswered** after the head field shipped.

**Provenance:** the *"one quantity, two regimes"* DECIDED at `water.md:136-140` is **user-ratified**
(*"one quantity, two regimes — that's quite right"*) and is **not** what this finding touches.
The **consequences** at `:156-161` are assistant-derived elaboration under that heading — a
hypothesis, not data. flow.md § 2.4 is user-ratified; § 2.4.1's boundary conditions are explicitly
**implementation-originated** (`flow.md:182-184`: *"This section was under-written and **the build
had to supply it**"*). So: assistant hypothesis vs shipped implementation, under a user-ratified
umbrella on both sides.

**Better-supported side: B**, but flag honestly that A is only *partly* falsified.
**Recommendation (labelled):** add the tier qualifier to `water.md:156` and mark open question 5
answered by journal/0098.

---

### F6 — `52 MiB` used as the live `DeepField` denominator in two cost arguments; the measured baseline is `162.57 MiB`

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A** | `docs/design/tectonics.md:387` | *"Total **≈ 5.1 MB** against the **52 MiB** run — noise."* |
| **A** | `docs/design/tectonics.md:748-749` | *"Memory: +~10 MB working, +~8 MB resident (`exhum` + drainage export + masks) — **noise against 52 MiB**."* |
| **B — measured** | `docs/design/flow.md:616` | *"Baseline `DeepField` residency **T = 162.57 MiB**, of which the record is 90.8 %."* (S19, `docs/spikes/S19-flow-record-cost-results.md`) |
| **origin of A** | `docs/design/tectonics.md:48` | *"**S9 (measured)**: A tier 460 m / 297 k cells / **14.1 s / 52 MiB**"* |

Shape 4 + shape 6 together: `52 MiB` was a correct 2026-07-19 measurement (S9) that has been
carried forward as a **live denominator** in two "this is noise" arguments. It is off by **3.1×**.
The arguments' *conclusions* both survive (5.1 MB and 8 MB are noise against 162 MiB too) — so this
is not a wrong decision, it is a **wrong number a future decision will inherit**. `flow.md:788-789`
independently quotes *"a 149 MiB field that is already 13 MiB below where the session started"*,
i.e. 162 → 149, which is internally consistent with B and further from A.

**Better-supported side: B** (dated spike measurement, and the more recent world).
**Recommendation (labelled):** restate as *"≈5.1 MB against the measured 162.57 MiB baseline
(S19)"* — quote the measure, name the world it came from.

---

### F7 — One number, two values, **inside `flow.md`**: D8 peak catchment `1,245` vs `1,255`

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A** | `docs/design/flow.md:344-345` | *"journal/0109 measured what that costs in the same breath as it shipped: the **peak catchment collapsed 1,245 → 84 cells**."* |
| **B** | `docs/design/flow.md:430` | table row *"peak catchment (cells) \| **1,255** \| **84** \| **298**"* (D8 column) |

85 lines apart, same quantity, same column of the same table's baseline, no reconciling sentence
anywhere in `flow.md`. **The reconciliation exists and is stranded in the journal:**
`journal/0113-water-that-stays-in-its-banks.md:172-173` — *"(D8's peak reads 1,255 here against
journal/0109's 1,245 — the same quantity on a world that has since gained …)"*.

**Neither side is wrong**; what is missing is the sentence saying so — the skill's own note that
*"a contradiction is not automatically a correction; sometimes both statements are true of
different things."* Low absolute cost, but it is the textbook instance of the shape and it is in a
RATIFIED doc's headline acceptance table.

**Recommendation (labelled):** carry journal/0113's parenthetical into `flow.md` beside the table.

---

### F8 — `light.md` still poses "does one relaxation machinery serve light and bound water?" as open; condition-fields were DECIDED 2026-07-24 and `flow.md` reports it answered

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — open** | `docs/design/light.md:147-152` | *"**Structural note**: bound-water saturation (S11), sim light, and plausibly heat are all *the same computational shape* … **Check whether one machinery serves all three before building two of them.**"* |
| **A — still owed to a spike** | `docs/design/light.md:236-238` | *"3. **Whether one relaxation machinery serves light and bound water** (§ 5) — or an honest statement of why not."* |
| **B — reported answered** | `docs/design/flow.md:585-586` | *"**Condition-fields over field-ids (§ 14)** — head, saturation, temperature are one shape; **the geotherm proved it.**"* |
| **B — DECIDED** | `docs/design/material-behavior.md:1001` (grepped) | *"## 14. Condition-fields, formation predicates, and the geotherm (**DECIDED 2026-07-24**)"*; `:1010`: *"**`temperature`** (new — the geotherm), `pressure` (derived). **Extensible** — every field …"* |

light.md is a 2026-07-20 design pass with no spike run; its § 10 measurement list is genuinely
still owed. But **item 3 of that list has been answered by construction** for two of the three
members (saturation + temperature ship as condition-fields), and light.md carries no pointer.
A spike dispatched off this list would re-derive a DECIDED result.

**Better-supported side: B**. **Recommendation (labelled):** narrow light.md § 10 item 3 to *"does
the condition-field shape extend to **light**"*, citing material-behavior.md § 14.

---

### F9 — Inside `water.md`: paleo-channels "exist in the record" vs "computed and discarded", 70 lines apart

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A** | `docs/design/water.md:398-403` | *"**Erosional caves may be the cheapest first family** — deep time already computes drainage every epoch, so **paleo-channels exist in the record**; an abandoned conduit is a former channel the water table later dropped below. **Possibly derivable from data already held**"* |
| **B** | `docs/design/water.md:469-474` | *"**The deep sim's per-epoch drainage is computed and discarded** (`DeepField` keeps only `surf` + `strata`). Paleo-channels are the erosional-cave feedstock ('possibly derivable from data already held', § sweep) — **needs a recorder axis** (channels, and the table per chapter)."* |

The corrections #65 shape in miniature: a claim and its own refutation in one file, and **B cites A
by name while A carries no back-pointer** — one-directional again. B is right (`tectonics.md:514-515`
independently confirms the record kept only `surf + strata`). Both are now overtaken a third time
by the shipped flux record (`flow.md:107-149`), which *is* the recorder axis B asked for — so A's
"derivable from data already held" has become true again for a different reason. **A reader
arriving at `water.md:398` today gets the right conclusion from the wrong premise.**

**Recommendation (labelled):** back-pointer at `:398` to `:469` and to `flow.md` § 2.

---

### F10 — Isostatic rebound fraction: `≈0.85` (derived) vs `~0.8×` (asserted)

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A — derived** | `docs/design/tectonics.md:444-446` | *"**Erosion-unloading rebound**: erode a belt, the (smoothed) root rebounds ~ρc/ρm ≈ **0.85** of the removed thickness; net surface lowering per metre eroded ≈ 0.15 m, so total rock processed per metre of peak lowering ≈ **6.7 m**"* (with ρc = 2800 at `:380`, ρm = 3300 at `:429` → 0.848) |
| **B — asserted** | `docs/design/earth-processes.md:85-86` | *"crust floats, so erosion unloads and the root rebounds (**~0.8×**), which is how relief persists for hundreds of Myr"* |

Same physical constant, two values, in the doc that **owns the scope item** (B) and the doc that
**is the design pass for it** (A). The 6.7× exhumation multiplier that A derives is sensitive to
this: at 0.80 it is 5.0×, at 0.85 it is 6.7× — a **34 % swing** in the headline number of the
exhumation argument. Minor today (both are design, not shipped constants I verified), but it is
the same defect class as F4's, one order of magnitude smaller.

**Better-supported side: A** — it carries its derivation and its densities. Per CLAUDE.md,
*"a bound with a derivation is evidence."* **Recommendation (labelled):** make B cite A.

---

### F11 — `flow.md` § 2.6.1's "Shipped, on by default … `mfd_exponent = 4.0`" vs § 2.6.2's hybrid law

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A** | `docs/design/flow.md:286-287` | *"**Shipped, on by default** (`DeepConfig::mfd`, **`mfd_exponent = 4.0`**)."* |
| **B** | `docs/design/flow.md:361-362` | *"Shipped: **`p_hill = 1`, `p_chan = 16`**, `chi_lo = 3×10⁻²`, `chi_hi = 1.2×10⁻¹`."* |
| **B** | `docs/design/flow.md:411-412` | *"**Uniform `p` is a control, not a shipping mode.**"* |
| **B — code** | `crates/dc-worldgen/src/deeptime/erosion.rs:362-363` | `p_hill: 1.0,` / `p_chan: 16.0,` (the shipped hybrid law) |

§ 2.6.1 is a dated `BUILT 2026-07-25` block and § 2.6.2 immediately follows it, so a linear reader
is fine. A **grep-arriving** reader (searching `mfd_exponent`) lands on A and reads *"shipped, on by
default"* about a value § 2.6.2 demotes to a control. **Low blast radius, listed for completeness.**

---

### F12 — Cell count: `DEEP_MAX_WIDTH = 550` / "~300 k cells" vs the measured `297,025`

| side | citation (at `f652b60`) | statement |
|---|---|---|
| **A** | `docs/design/tectonics.md:54-56` | *"`field.rs` caps the deep grid at **`DEEP_MAX_WIDTH = 550`** (**~300 k cells** at every extent …)"* — 550² = 302,500 |
| **B** | `docs/design/flow.md:610` | *"Geometry: **297,025 cells** @ 460 m"* — 545² = 297,025 |

A 1.8 % gap, and every per-cell budget in `tectonics.md` § 5.1 / § 11 is scaled on `550²`. Almost
certainly a border/interior convention, not an error — **I did not read `field.rs`'s constant, so I
cannot attribute it**, and I am recording it rather than reconciling it.

---

## 2. Claim inventory

Load-bearing claims asserted as **CURRENT**, one line each, `file:line` at `f652b60`. Numbers carry
units and stated source. `⚠` marks a claim implicated in a finding above.

### `docs/design/tectonics.md` — RATIFIED 2026-07-20 (user, U1–U8); no staleness banner ⚠F2

- `:3-15` — architecture + U1–U8 RATIFIED 2026-07-20 (user); U3 amended by user: ritual ceiling is **not ~28 s**, *"5 min if that's what it takes."*
- `:17-26` — U8 DECIDED 2026-07-21 (user): `tectonic_history: true` in `production_config`; worlds made before are **not reproducible** under it. `full_agents` stays OFF *(note: `earth-processes.md:186-188` says `full_agents` was flipped ON 2026-07-21 — sequential, not contradictory)*.
- `:48` — **S9 measured**: A tier **460 m / 297 k cells / 14.1 s / 52 MiB**; C refinement **~3.6 s/region**; hillslope decay **21 cells**; drainage the **sole advective** process (corrections #8). ⚠F6
- `:54-57` — **`DEEP_MAX_WIDTH = 550`**, ~300 k cells at every extent, Large ~**1.85 km** cells, **200 iterations**, **~13.8–14.4 s** production parallel path (corrections #12, journal/0029/0030). ⚠F12
- `:58-60` — Phanerozoic register RATIFIED 2026-07-19: ~**500 Myr** span, **1 iteration ≈ 2.5 Myr**. ⚠F4
- `:97-101` — `n_plates = clamp(w²/20, 3, 24)`: Small 3 / Medium 14 / Large 24 → plate diameter **~43 / ~67 / ~208 km**.
- `:105-106` — painted effects: **±1400 m·m** orogeny over a **2-ring** falloff on **14.7 km** pregen cells, bilineared to **460 m**.
- `:181-194` — chapter table: K=4 (50 iters, 125 Myr) / **K=8 recommended** (25 iters, **62.5 Myr**) / K=16 (~31 Myr). ⚠F4
- `:198-207` — Earth plates **2–10 cm/yr ≈ 20–100 km/Myr**; over 500 Myr = **10,000–50,000 km** vs a **251 km** world. Design parameter: total advection **~0.5–1.5 plate diameters** (≈**30–100 km** at Medium).
- `:227-232` — `DepUnit.chapter: u8` joins the merge key; ≤ **K−1 = 7** extra breaks/column at K=8; S9 mean **1.42** units; `DepUnit` currently tag 4 B + f64 8 B + bool 1 B → **16 B**.
- `:269-276` — determinism: new salt family **`0x5B00_*`** (pregen `0x5700_*`, deep-time `0x5900_*`).
- `:307-315` — **`W` orogen half-width**, continent–continent default **≈ 25 km** scaled `1/√v_conv`; trench offset **D_t ≈ 10 km**, arc offset **D_a ≈ 40–60 km** (real arc–trench gaps **100–250 km**).
- `:316-319` — junctions sum over the **k = 3** nearest seeds.
- `:378-387` — new planes at 550²: `t_crust` f64 **2.42 MB** (continental seed **~35,000 m**, oceanic **~7,000 m**), `crust_kind` u8 **0.30 MB** (ρc ∈ **{2800, 2950, 2870}**), `exhum` f64 **2.42 MB**; total **≈ 5.1 MB** against **52 MiB**. ⚠F6
- `:398-399` — sediment density **ρs ≈ 2400**.
- `:420-436` — Airy on the **smoothed** load; **Λ_flex ≈ 50 km** (~110 cells @460 m, ~27 @1.85 km); **ρm = 3300**; `C_ref` calibrated so 35 km continental ≈ **+400 m**, 7 km oceanic ≈ **−4000 m**; `λ_iso` a rate knob.
- `:444-446` — rebound **≈ 0.85** of removed thickness; net lowering **0.15 m/m**; **≈ 6.7 m** rock per metre of peak lowering. ⚠F10 ⚠F4
- `:465-469` — full biharmonic rejected: forebulge is **~10 m-class**, below the read floor.
- `:511-521` — **§ 7.1 correction**: deep-time drainage is **not** one-shot (re-runs every iteration); the *exported* river network is pregen `Cell.river`/`discharge` at **14.7 km** chords on the **pre-erosion** surface. **Two authorities, one stale.**
- `:551-557` — export `recv` / `area` / lake mask; consumers = 3e-2 frozen macro topology + corridor mechanism + water-table pinning lattice + body graph. ⚠F2
- `:558-569` — per-chapter channel masks **~300 k × K bits ≈ 0.3 MB** at K=8.
- `:592-604` — chapter table **~n_plates × 5 f64 ≈ 560 B/chapter**; *"the entire tectonic history of a world is **~5 KB**"*; `StructEvent` list world-level, **dozens** of entries.
- `:606-611` — the **665 k → 71 k** record overprint optimization survives *by construction*.
- `:690-716` — plate-scale knob: recommendation **B, `plate_scale_km` default 65**; the clamp at 24 makes Large **~3× sparser** in provinces/km than Medium; `Cell.plate` is already `u16`.
- `:727-749` — cost: chapter repaint **~ms**, columns **+0.2–0.5 s**, isostasy **+0.5–2 s**, total **≈ +1–3 s → ~15–17 s**; fork: 300 iters ≈ **~21 s**, 400 ≈ **~28 s**; memory **+~10 MB working / +~8 MB resident** — *"noise against 52 MiB"*. ⚠F6
- `:756-771` — flag `DeepConfig::tectonic_history`, default false, byte-identical when off; items 1–4 **do not flip separately**.
- `:862-869` — U1 `plate_scale_km` 65 · U2 K=8 · U3 accept up to ~2× (≈28 s) · U4 W≈25 km, arc gap ≈50 km · U5 ~1 plate width/run · U6 ≤1 impact, ~1–2 other rolls/world · U7 amplitude after forcing · U8 flip after spike.

### `docs/design/flow.md` — RATIFIED 2026-07-25 (user, *"standing blessed"*)

- `:3-7` — supersedes the **river half of `water.md`**; retires **`earth-processes.md` § 3e-2's expression half** (its *constraint* half survives, qualified).
- `:30-41` — regimes differ in four numbers + one field: **viscosity, density, competence, resistance**; field per regime (fluvial→head, eolian→wind, glacial→ice surface, gravity→slope, turbidity→density, hydrothermal→thermal+head, karst→head).
- `:53-56` — **free = fluid in open space, bound = fluid in pore space**; free↔bound is an **edge on the form-transition graph (S-8)**.
- `:73-74` — bound is a bounded relaxation (**S11 halo 4–11 cells**); free is **connectivity, not local at any radius**.
- `:88-94` — **correlate by CHAPTER, never by slot index** (surfaces are diachronous); base is superposition, **facts are cross-cutting**.
- `:98-101` — **the atom**: at (cell, stratum-slot), flux crossed faces **F** with magnitude **M**, in form **P**, carrying load **L** of fluid **f**, under cause **C**, at chapter **K**.
- `:111-120` — the receiver tree **structurally cannot represent divergence**; it is computed at **~14.7 km** on **pre-erosion** topography. ⚠F2
- `:133-145` — **⚠ UNDER-SPECIFIED (S19)**: the slot-pairing rule; pairing by slot index is **silently wrong**; candidate rule **NOT RATIFIED**; the choice is uniformly a **2×** on cost. *Resolve before any refinement/expression slice.*
- `:153-163` — the face set is **lateral (per slot) + vertical (slot↔slot) + boundary (atmosphere / ocean)**.
- `:169-180` — the driving field is **potential/head**; the free-regime divide constraint does **not** bind the bound regime (Great Artesian Basin, Ogallala, karst piracy); 3e-2 decision 1 **qualified to FREE/surface only**. ⚠F5
- `:190-200` — head boundary conditions: Dirichlet at sea stand / at free-water elevation (lake or stream ≥ `STREAM_ANCHOR_AREA`); unconfined elsewhere **capped at its own ground** (seepage face); **confined = UNCAPPED** — *"artesian is not a special case in the code — it is the ABSENCE of a cap."*
- `:202-204` — steady `∇·(T ∇h) = 0`, **Gauss–Seidel, alternating forward/reverse raster sweeps**; information crosses the whole grid in one sweep. ⚠F5
- `:210-214` — **248 of an initial 308** "artesian" columns were **lakes**; a further **232** were priority-flood dust at **~2.5 × 10⁻⁵ m** — `PONDED_MIN_M` floor required.
- `:216-221` — **honest limit: no recharge term (`R = 0`)**, so artesian excesses come out in **metres**, not the hundreds a real GAB gives — **stubs #19**, heir = a real water-balance climate.
- `:226-228` — fluid carries a **material id**; a glacier is a solid of viscosity **~10¹³**.
- `:241-243` — slice-1 measurement: **175,320 divergent (cell, chapter) pairs, 7.378 %**.
- `:255-257` — pass `period` = **1**; aggregation window = one tectonic **chapter = 25 epochs**.
- `:266-269` — **every divergence in the record is TEMPORAL (avulsion), never SIMULTANEOUS.**
- `:278-283` — **corrections #54**: MFD does **not** need `dc:field/head`; the free regime's potential is the priority-flood `filled` surface.
- `:286-287` — MFD shipped on by default, `mfd_exponent = 4.0`. ⚠F11
- `:291-302` — **Holmgren (1994) + Quinn contour width**: `wₖ ∝ Sₖᵖ · Lₖ`, `Sₖ = Δh/dₖ` on true path length (`1`/`√2`), `Lₖ` ∈ (`1`/`1/√2`); **corrections #58**: `p → ∞` is single-receiver **steepest-slope**, not D8 exactly.
- `:305-307` — the partitioned field is `filled = z_bed + depth`.
- `:315-318` — **`MFD_MIN_WEIGHT = 1 %`**; steepest share ≥ **1/8** before the floor.
- `:319-334` — priority-flood pop order is a topo order of the **DAG**; weights sum to `1 ± 1 ulp` with the last direction taking the residue; never-incise clamp becomes *below the **lowest** receiver*; energy slope = **`Σ wₖ Sₖ`**.
- `:335-339` — **`recv` is now the argmax share**, a summary; continuation **(e)** should retire it. ⚠F2
- `:343-345` — journal/0109: peak catchment **1,245 → 84 cells**. ⚠F7
- `:356-362` — **`χ = A · S²`** (Montgomery & Dietrich 1988/1992); ramp `p_hill`→`p_chan` log-linear; **hard switch to single-receiver at `chi_hi`**. Shipped: **`p_hill = 1`, `p_chan = 16`, `chi_lo = 3×10⁻²`, `chi_hi = 1.2×10⁻¹`**. ⚠F11
- `:367-375` — a 90 %-of-steepest rival keeps **19 %** at `p = 16`; suppression below the floor needs **`p > 44`**; a 95 % rival needs **`p > 90`**. **`p = 16` everywhere lifts peak catchment only 84 → 145.**
- `:392-400` — `p` reads the **previous epoch's** drainage plane (one-epoch lag, same shape as the S10 biotic coupling); epoch 0 is maximally dispersive.
- `:401-412` — slopes normalised by `S_max` **on the hybrid path only**; keeps **three cross-commit fixed points byte-reachable**.
- `:413-417` — the ramp is **rounded to an integer** (`powi` not `powf`; ~**half a billion** directions per production run).
- `:419-424` — at 460 m this is a **sub-grid parameterisation**, not "is this cell a channel"; `chi_lo`/`chi_hi` calibrated against this world's own χ distribution — **stub #26**.
- `:426-444` — measured (seed 1337, Medium): peak catchment **1,255 / 84 / 298**; p99 land catchment **127.0 / 70.0 / 164.4**; top-1 % share **0.0849 / 0.0248 / 0.0613**; land cells >100 **530 / 0 / 2,270**; simultaneous divergence `(cell, epoch)` **0 / 7,548,535 / 7,228,964`; **95.8 %** of § 2.6.1's divergence survives. ⚠F7
- `:446-451` — journal/0111: this world exports **0.02 %** of its denudation by rivers, at **0.0110 m/Myr**. Hybrid `p` **cannot move denudation, the facies gradient, or the appearance of the world**.
- `:489-494` — the four tiers; **refinement operators must be a pure fn of (record, shared face data, position)**.
- `:496-499` — **the record is the only seam.**
- `:541-550` — four named limits: oscillatory/tidal nets to ≈0 (needs a **gross-energy** term); **episodic catastrophes** average away inside a chapter; sub-cell lateral channel migration is refinement-tier; **evaporites** precipitate because the carrier left, a distinct trigger.
- `:556-566` — retires `pregen/hydrology.rs`, `RiverSeg`/`carve_rivers`/`BANK`/`RIVER_REACH`, `Cell::{flow_to, river, discharge}`, and 3e-2's expression half. ⚠F2
- `:609-628` — **S19 measured**: **297,025 cells @ 460 m, K = 8, 200 epochs**; land **14.9 %** of cells but **76.8 %** of slots; units/cell **bimodal** (land mean **96.0**, marine **5.1**, overall **18.64**, median **5**, max **478**); baseline **T = 162.57 MiB**, record **90.8 %**; flow facts **causally triangular** → **57.7 %** of `slots × K`, a free **1.73×**; dense layouts **1.9×–5.5× T**; sparse crosses **1× T at 4.30 %** face-sparsity, **4× at 17.94 %**; CSR index floor **0.056× T**; today's `FactLedger` is **98.8 % empty inner Vecs**, **89 %** of heap empty headers, **+156.91 MiB (1.96× T)**. ⚠F6
- `:667-704` — **§ 10 justifications that WILL expire**: paleo-elevation pairing trivial only because strata are layer-cake; paleo-elevation a running sum only because nothing compacts; **pairing is IDENTITY, never CONNECTIVITY**.
- `:713-741` — **11.1 RATIFIED (user)**: the aggregation **WINDOW** is a declared axis. **🔴 the ORDER half SUPERSEDED 2026-07-26 (user)** — correctly struck in place. Cadence model grows to **ORDER × RATE × WINDOW**.
- `:743-750` — **11.2 RATIFIED**: chapter-vs-epoch is a **shipped default**, not an engine property; default to the **cheap end**.
- `:752-758` — **11.3 standing contract**: the record must **self-describe its completeness**.
- `:760-792` — **11.4**: **79.38 % of slice-1 entries (31.37 MiB of 40.66 MiB)** are ocean-sink faces; dropping leaves **9.29 MiB**. **DEFAULT: KEEP — RATIFIED (user)**; the drop is gated on 11.3.
- `:794-817` — **11.5 RATIFIED**: pairing by **what confines it** (material horizon → chapter; potential surface → elevation/head). **A THIRD mode — the conduit/void mode — is named and NOT covered**; ASSIGNED to continuation **(c)**, which therefore ships **three** obligations.

### `docs/design/water.md` — OPEN FIELD NOTEBOOK 2026-07-20; banner ⚠ READ flow.md FIRST

- `:3-13` — banner: the **river/drainage/channel-expression half is superseded**; survivors enumerated; **S14 superseded as posed**.
- `:136-153` — **DECIDED 2026-07-20 (user)**: water is **ONE conserved quantity in two regimes** (bound = pores + empty eighths; free = open space). *"one quantity, two regimes — that's quite right."*
- `:156-161` — consequence 1: the water table is **READ, not modelled**; *"nothing stores 'the water table'"*; equilibrates locally, **never traced from a source**. ⚠F5
- `:162-169` — consequence 2: **vadose vs phreatic falls out for free**; no cave-morphology system needed.
- `:170-180` — consequences 3–5: aquifer/aquitard are **material facts**; S10's waterlogging proxy has a **defined retirement**; conservation is the invariant (cf. `Δ(ΣR+ΣH) == uplift + biotic`).
- `:231-254` — **user stance**: perfect volume conservation is **the ideal, not the requirement**; *"it's going to be lossy… neither is reality (the water cycle is half gaseous)"*; **every perf hack sanctioned**; **the compromise is the LAST step, not the first.**
- `:271-293` — **S11 measured**: bound-water halo **4–11 cells** (**0–6** for the integer table); links measured **0–1** in every scenario (corrections #14); a finite sea breached drops **13.18 m**, a pinned sea costs **415 ms → 0.0 ms**; bodies are **~20 bytes**, **1 body through 1,624 edits**, ceiling **217** = component count; byte-identical reload from **39 bytes**.
- `:301-327` — **DECIDED 2026-07-20 (user)**: **Pinned default ON and toggleable**; the ~12-cell halo is a **KNOB, not a constant** — *"where there's a cell range, there's a knob"* (general doctrine); calls 3–4 reclassified as engineering.
- `:329-335` — recorded assistant error: calls 3 and 4 should never have been put to the user; **the integrator's job is to triage, not relay.**
- `:398-403` — erosional caves may be cheapest; paleo-channels **exist in the record**. ⚠F9
- `:461-468` — **two drainage opinions exist**; the settlement-siting consequence is **VOID as of 2026-07-28** (journal/0121, history pass deleted).
- `:469-474` — deep drainage is **computed and discarded**; needs a recorder axis. ⚠F9
- `:493-497` — water-table placement: **likely BOTH** a coarse paleo table per chapter and a present-tier relaxation.
- `:527-586` — **S14 SPIKE SPEC, NOT dispatched, NOT ratified**; decision rule fixed in advance; **superseded as posed** by the banner.
- `:588-605` — six carried open questions, incl. **#5 where the deep-time water field lives** (answered in fact by journal/0098's head field). ⚠F5
- `:611-614` — **S11's numbers do not transfer** to a lazily generated world: *"**415 ms is an honest number for a world that does not exist.**"*
- `:679-686` — the one-octree leaning **RESOLVED same day** → `octree-substrate.md` (DECIDED 2026-07-22).
- `:736-739` — the user's one-line refutation of the two-index split: **"where is the duke?"**
- `:766-804` — **S15 measured**: coarse path ships above **26 m²** of water surface; **781 of 791** bodies within half a voxel, up to **212,000 m²**; **coarse 109 ms / 0 chunks** vs **exact 64,104 ms / 6,444 chunks**; largest exact body **23 m²** at **0.2–3.2 ms**; drift **1.9 × 10⁻⁵ m over 20,000 edits**; **FALSIFIED** — the flooded shaft is not the worst case (error **0.000000 m**, corrections #30) and connectivity's second clause is false (**one plug voxel** joins a body **288 m** away; **972 of 1,215** basin floors already one body, corrections #31); eviction identity re-proved across **3,120 evictions** at an 8-chunk budget.
- `:806-813` — **the open design question**: capacity below a coarse cell's floor plane comes **only from edits** — complete today, **wrong the moment caves exist.**

### `docs/design/earth-processes.md` — doctrine DECIDED 2026-07-19; method RATIFIED 2026-07-19

- `:3-7` — **doctrine (user)**: ask what reality looks like **without thinking of our code**, then build the highest-fidelity process sim working backward from reality.
- `:9-29` — method rules 1–5, incl. **2 mechanism fidelity over resolution fidelity** and **5 the scream rule** (*"if i see a square boundary I'll scream"*) — contact-softening **IN SCOPE for 3e-2**, not optional.
- `:47-72` — **Direction DECIDED 2026-07-20 (user)**: uplift(t) + analytic boundary forcing; painting on **14.7 km** cells truncates forcing at **~15 km wavelength**; plate compression **~14 plates over 251 km ⇒ ~60–70 km plates** should be a **stated, user-owned knob**.
- `:74-104` — scope expanded (user, *"all of these land — escalate the foundational"*): crustal columns, **isostasy + flexure (rebound ~0.8×)** ⚠F10, drainage export per chapter (with corrections #20 inline), recorder event-entries, punctuation hooks.
- `:129-144` — **erodibility coupling BUILT 2026-07-20 (journal/0029)**, off by default; **resistance is agent-specific, never a single scalar** — *"Limestone makes cliffs AND caves; one number cannot hold both."* Dissolution / frost-ice / wave axes **populated and dormant**.
- `:148-159` — **RATIFIED 2026-07-20**: wind is **agent #5**; frost and wave axes go live; at 460 m the deliverable is dune-*field* / loess / periglacial **regions**.
- `:161-177` — follow-up slice RATIFIED: **zonal circulation profile** replacing the three-way `wind_dx` sign bit; the 30° desert belt is the **Hadley descending limb**, not rain shadow.
- `:186-201` — **DECIDED 2026-07-21 (user)**: `full_agents` **ON** in `production_config`; **the seven agent magnitudes are UNRATIFIED** — `eolian_deflation` **0.02**, `eolian_arid_precip` **0.32**, `eolian_deposit_frac` **0.25**, `frost_weathering_gain` **1.5**, `frost_band_width_c` **12.0**, `wave_erosion` **0.05**, `wave_band_m` **30.0**.
- `:241-242` — cost philosophy RATIFIED: **"coarsen the cause, never delete it and fake the appearance."**
- `:282-290` — **S9 VERDICT measured 2026-07-19**: A (460 m) **14.1 s / 52 MiB**; B (48 m, **27.3 M cells**) **~63 min / ~3 GiB** projected (corrections #8 — *"minutes" was wrong*); C **≈3.6 s/region**; decay **21 cells** hillslope, fluvial spikes to **26**; everything but drainage relaxes at **16–24 cells**. **USER-RATIFIED: A always-on + C refinement.**
- `:292-302` — **⚠ SUPERSEDED IN PART 2026-07-25, see flow.md** — expression half of § 3e-2 retired; (a) the divide rule binds **FREE only**; (b) the driving field is **potential/head**. Survivors: drainage is advective + decided-once-coarse; descent-along-potential is a hard constraint.
- `:304-341` — **3e-2 decisions 1–5 RATIFIED 2026-07-19**: drainage decided ONCE at the coarse tier; width cap is **permanent architecture, not apology**; stitching at the finer lattice with a **16–24-cell** halo; approach trigger **~3.6 s** async; **Phanerozoic register ~500 Myr default**, *"procedural hacks for the boring billion"*.
- `:344-361` — **⚠ FLAG 2026-07-26**: the register is stipulated and **the engine's rates do not honour it** — **0.0110 m/Myr**, **9×** slower than the slowest measured Earth surface (McMurdo/Atacama **0.1–1 m/Myr**), **493×** below the global ¹⁰Be outcrop median (**5.4**, Portenga & Bierman 2011), **5.48 m** stripped vs a real craton's **5–10 km**; denudation is **2.7 %** of rock uplift. **The shape is right; only the rate is wrong.** Target band **1–10 m/Myr**; instrument = `examples/denudation_probe.rs`. ⚠F4
- `:362-376` — **S9b CONFIRMED (corrections #9)**: parallelism does not flip B — the flood is **98.5 %** of the step at B scale and has **no byte-identical parallel form**; whole-step **1.2×**, saturating at **8 threads**; realistic parallel B **15–70 min** vs a 2-min target. Reopens only on **~32-core** hardware with a deterministic parallel flood.
- `:378-390` — **payoff layer DECIDED 2026-07-20 (user)**: **ore is the reward for reading the world correctly.**
- `:398-411` — **Magnitude ratification DECIDED 2026-07-21 (user, journal/0049)**: wind + frost magnitudes **RATIFIED as-built**; **WAVE NOT ratified** — station 5 confirmed the null (**0.68 m** total at the world's most-attacked coast), user directed *"more dramatic by default"*, target class **~10–40×**. **Heir: wave energy as a fact about the water body** (fetch from S11's body graph × the 0037 wind field).

### `docs/design/geology.md` — backbone agreed 2026-07-18; content half under discussion

- `:10-12` — **Processes bind to classes, not instances.** Vanilla geology is just the first geology pack.
- `:13-18` — **classes are contracts**, parameter schemas not labels; validated at define time by the dc-api registry.
- `:19-26` — selection = **fitness × abundance × seed**, canonical order by namespaced id, abundance normalized within class. Seed-stability consequence **SUPERSEDED 2026-07-22 (user)**: generation-affecting content **cannot be added to an existing world at all** — correctly struck in place.
- `:27-32` — processes are plugins; **declared reads/writes topo-sort the pipeline "instead of a hand-maintained list."** ⚠F1
- `:33-38` — **context vocabulary**: depth, T/P history, climate-at-deposition, tectonic province, fault proximity, host class. **OPEN**: plugin-published axes; leaning **core-axes-only for v1**.
- `:43-53` — **v1 DECIDED 2026-07-18**: clastic sediment + igneous (intrusive/extrusive) + one ore vector (placer). Superseded on the ore axis by § Ore (2026-07-20).
- `:64-78` — **TAKEN 2026-07-20 (journal/0029, user "sequence erodibility first")**: erosion resistance is **agent-specific, never a single scalar**; `LithoResistance` carries one resistance per agent (abrasion / dissolution / frost-ice / wave); **only abrasion is wired**.
- `:81-97` — **SHIP UPDATE 2026-07-19 (3e-1)**: the deep-time A tier is **always-on in `Pregen::run`**; the shim is dead for deep strata; year-zero climate remains legitimate **only for the active surficial veneer**.
- `:99-122` — **Formation context DECIDED 2026-07-19**: must become **epoch-indexed**; igneous fitness is **province/depth-driven, never surface weather**.
- `:124-138` — **SHIP UPDATE 2026-07-20 (journal/0026)**: three organic classes added; **charcoal got no member** (mean bed **~3.5 cm**, none of **158,310** survives the 0.9 m voxel); **coal rank got no ladder** (lignite→anthracite discriminated at **~1–2 km** burial, our record tops out near **100 m**).
- `:140-157` — **SHIP UPDATE 2026-07-22 (journal/0063)**: charcoal **now has a class and a member** — the measurement held (**102,113 beds, mean 0.0289 m, max 0.0400 m**, none reaching one eighth alone), the *inference* expired. **Measured expression: 0.394 %** of recorded voxel spans carry a charcoal eighth. Precedent: **a resolution argument is a claim about the quantizer, and quantizers change.**
- `:158-164` — coal rank still has no ladder; *"it **now promotes on each unit's own overburden**"*; *"**there is no geotherm in this project**"*; **13 of 35,382** peat-derived units lie under 50 m; `stubs.md` § 14 named as the holder. ⚠F3
- `:168-204` — **Roster DECIDED 2026-07-19**: rich vanilla mineral roster; **inclusions are pore partials**; unfilled slots enforced at **two layers** (define-time fallback member + world-build-time refusal). Refined: the refuse-to-build stance is the **interim**, eventual shape is a **loud named skip**.
- `:206-249` — **Ore DECIDED 2026-07-20 (user)**: v1 roster = coal (shipped) · banded iron · bog iron · redbed copper · orogenic gold → placers · evaporites. **Grade IS the eighths count.** Endowment = **soft guarantee at Medium+**. Naming owned by the future culture/language layer.
- `:251-271` — **Expression of the ledger DECIDED 2026-07-21 (user)**: runtime generation is **local refinement over the coarse deep-time ledger**; **everything in the ledger must be expressed**; inexpressed permitted only when the expresser is unbuilt, and then **loudly temporary**.
- `:273-281` — placement keyed on present-day slope/elevation/temperature is a **mock, not a model**; three named holdouts.
- `:283-293` — **Holdout (c) closed 2026-07-21 (journal/0053)**: **`Σ(recorded unit thicknesses) ≡ H` exactly** — the record *is* the loose column decomposed into beds. Veneer 8-voxel cap = `stubs.md` § 12.
- `:295-309` — **Form follows provenance**; **per-voxel provenance is a first-class output** (user) — with an integrator note that the literal reading is **NOT yet ratified**.
- `:311-320` — **Genesis addendum DECIDED 2026-07-21 (user)**: a stub is a candidate to be subsumed; **stubs are permanently legitimate only in genesis**.
- `:322-348` — **Enhancement doctrine DECIDED 2026-07-21 (user)**: procedural tricks can be honest given (1) a **deterministic relationship with the larger field of facts** and (2) **local process and deep process must agree** — *"in what sense they must agree is an explicitly open design question."*
- `:350-418` — **DECIDED 2026-07-22 (user)**: the deep sim's material interface. `Litho::reference_material` maps **six** classes to six fixed rocks, discarding **~30** registered materials — a **substance** error, and because the record mirrors the **loose** `H` plane, simultaneously a **form** error. **Its justification (pack-safety) expired the same day** under the frozen-content-set decision — *"fourth instance in one session of a conclusion outliving its premise."* Sequence: **(1) seam now, (2) measure the class-aggregate but do not ship it, (3) ship `f(substance, form)` ONCE.** Aggregate-then-form rejected by the user: each change is a **terrain-shape flip**, paying that cost **twice for one conceptual change**.

### `docs/design/light.md` — DESIGN PASS 2026-07-20; no spike run

- `:9-13` — thesis: sim light is *"the simulation level of our light, **not crisp dynamic shadows etc, just what the voxels know**"* (user). **Not** the renderer's lighting, not derived from it, does not feed it.
- `:18-27` — **S3 wrote the skylight query contract**: consults **only** resident chunks + cached column summaries; bounded to `[y_min, y_max)`; **unknown volume is non-occluding ("optimistic sky") and poisons `fully_resolved`**; column summaries build at **~20 µs**. **S3 OQ 6 defers `re-light-on-load` to this work.**
- `:28-33` — S4: LabPBR packs **emission in channel A** (0–254 meaningful, **255 reserved**); the uniform block carries a **time-of-day slot at a fixed 0.35**.
- `:44-45` — **corrections #13**: an earlier *"user wants no darkness"* record was a misattribution from confusing the **fullbright diagnostic** with the **game's lighting**.
- `:49-81` — **§ 2 DECIDED 2026-07-20 (user)**, eight items: (1) **derived, never stored** — reach = emission ÷ attenuation, dodges **~32 KB/chunk**, and **has no un-propagation path at all**; (2) **sky and block light are separate channels**; (3) **emission and opacity are material properties**, no light-source-specific code; (4) **reach is per-source** — torch quality is a material question; (5) **no light field in deep time**; (6) darkness gates spawning, **light is equipment**; (7) **caves get their own ecology** from biome-as-diagnosis, no cave-specific system; (8) monochrome for now, **band-shaped**.
- `:84-106` — **§ 3 DECIDED 2026-07-20 (user)**: skylight is **directional**; a **table of heavenly bodies** (path, colour, intensity, rise/set); **sun and moon are rows, not special cases**; **day/night is EMERGENT from paths** — *the paths are the ephemeris*; `direction = path(body, world_time, observer_latitude)`; **BUILD NOW: only the path capability** (seasons and latitude explicitly not built); `lat_deg` already exists on pregen cells; world time is a **deterministic tick counter**.
- `:110-133` — **§ 4 the user's decomposition**: **DIRECT is a QUERY, not a field** — therefore **needs no direction quantisation at all**, and is what makes sun-into-the-cave-mouth affordable; **BOUNCE is the FIELD** — direction-free, downward-diffuse, propagates like MC skylight; cloud cover is a **hook, not a feature**. **Inherited gotcha: decide deliberately what full daylight on open ground means.**
- `:136-152` — **§ 5**: light is a **max-plus relaxation**, monotone with a unique fixpoint ⇒ **order-independent by construction** (*"the requirement, not a test outcome"*); attenuation reads **partial fill**, not a binary solid/air test; structural note — saturation, light and heat may be **one machinery** (S11 halo **4–11**). ⚠F8
- `:154-176` — **§ 6 OPEN, recommendation recorded**: colour **rides the source, not the propagated field**; model emission and sensitivity as **band vectors with exactly one band in v1**; cost ≈ a one-element array. *"This is the light-model twin of the erodibility/limestone trap."*
- `:180-198` — **§ 7 consumers**: plant growth wants a **budget over time** (bulk per-chunk pass); spawning/stealth/agent perception want **level at a point**. **Perception parity is a design principle**; **block light is unblocked by a dev-light block** (user).
- `:202-207` — **§ 8**: light attenuates with depth in water **automatically** ⇒ the **photic zone** for free.
- `:211-229` — **§ 9 open questions 1–7**, incl. re-light-on-load policy, per-voxel vs coarser resolution, level quantisation (**knob doctrine**), what full daylight means, coarse/LOD light, halo + invalidation shape, and **#7 DECIDED**: renderer and sim light are separate and neither is derived from the other.
- `:232-244` — **§ 10 what the spike must measure**, five items incl. **order-independence as a proof, not a benchmark** and **#3 whether one relaxation machinery serves light and bound water**. ⚠F8

---

## 3. What I could NOT attribute

Stated as unknowns rather than reconciled, per the skill.

1. **Which of `tectonics.md`'s § 5 / § 6 numbers are as-DESIGNED vs as-SHIPPED.** `Λ_flex ≈ 50 km`,
   `λ_iso`, `W ≈ 25 km`, `D_a ≈ 40–60 km`, ρc/ρm/ρs, `t_crust` seeds — the doc presents them as
   design recommendations, and `tectonic_history` was flipped ON 2026-07-21 (`tectonics.md:17-19`).
   `docs/spikes/S12-results.md:235` carries a section titled *"Deviations from tectonics.md forced
   by implementation reality"* (grepped, **not read** — outside my slice). **I cannot attribute
   which constants survived the spike.** The claim inventory records them as tectonics.md asserts
   them, not as shipped facts. *A stage-2 synthesiser should treat every § 5/§ 6 number as
   unverified against code.*
2. **F12's cell-count gap** (`550²` vs `297,025 = 545²`). I did not read `field.rs`'s
   `DEEP_MAX_WIDTH` or its border convention. **I cannot attribute this** and did not reconcile it.
3. **Whether `water.md` consequence 1 was ever intended to bind the deep tier.** The sentence
   carries no tier qualifier, and I found no third artifact adjudicating it. F5 is reported as a
   pair, not a verdict.
4. **The `full_agents` sequencing.** `tectonics.md:24` (2026-07-21) says *"`full_agents` stays OFF
   (not ratified)"*; `earth-processes.md:186-188` (also 2026-07-21) says it was **flipped ON**. Both
   are dated the same day and I could not order them from the docs alone. **Recorded as
   sequential-not-contradictory, with low confidence** — I did not open journal/0034, 0044 or 0047.
5. **`tectonics.md:97` says `clamp(w²/20, …)`; `:690` says `clamp(cells²/20, …)`** for the same
   formula. One of `w`/`cells` is the wrong symbol. **I did not read `pregen/tectonics.rs`** and
   cannot say which. Too small to rank; recorded so it is not lost.

**A null is a result, but a null here would be suspicious** — this slice was chosen because
tectonics/flow/water overlap. It produced 12 pairs, and the top four are all
**supersession-without-a-back-pointer**, which is the corpus's measured dominant failure
(`SKILL.md:104-113`: 8 of 15 correction→file edges one-directional). **Three of the four have no
banner at all on the stale side.**

---

## 4. Cross-cutting observation (one, offered as data for stage 2, not as a proposal)

Of the 12 findings, **7 (F1, F2, F3, F4, F5, F8, F9) are one-directional-pointer failures**: the
newer, correct artifact exists and names its target, and the target holds no link back. Only
**F7, F10, F11, F12** are pure number divergences, and **F6** is a summary that outran its source.

The single highest-leverage observation in this slice: **`docs/design/tectonics.md` is the largest
doc in the corpus, it shipped, its siblings both carry supersession banners, and it carries none.**
Whatever stage 2 concludes, that one file is where the slice's blast radius concentrates.
