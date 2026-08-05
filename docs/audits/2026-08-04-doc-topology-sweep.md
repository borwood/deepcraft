# doc-topology sweep — 2026-08-04 (FULL)

**Base commit:** `b4d7514` (`git merge main` = *Already up to date*; HEAD == `main` ==
merge-base, verified before any read). **Read at `978cb64`** — this audit's own skeleton
commit on top of `b4d7514`; **no source or doc file differs between them**.

**Watermark delta:** `2985273..b4d7514` — 105 commits, 45 `.md` files touched.
**Mode: FULL.** Watermark advanced to `b4d7514` in this same commit.

**NOT APPLIED.** This sweep is read-only except this file and
`docs/audits/.sweep-watermarks.json`. Every finding is a **pair**; which side gives is the
integrator's or the user's call, and recommendations below are labelled as readings.

**No cargo was run** (a workspace gate held the slot). This sweep is docs-vs-docs only.

**Ordinal held:** if the integrator disposes any finding below as a new `corrections.md`
entry, the next free ordinal **appeared to be `#101` at `b4d7514`** (tail is `## 100.`,
`journal/corrections.md:4006`; `#99b` is a deliberate sub-number). **Re-verify at merge** —
a parallel BODIES session shares this checkout and six ordinal collisions have happened.

---

## 1. Coverage — stated honestly

**Read IN FULL:**
`docs/design/materials.md` (650) · `.claude/skills/roster/SKILL.md` (189) ·
`docs/dependency-graph.md` (372) · `journal/0154` · `journal/0157` ·
`docs/spines.md` § A-7 body (`:2279-2380`) + § 3 preamble (`:2020-2110`) ·
`docs/design/geology.md` § Backbone (`:1-60`) ·
`ROADMAP.md` § In flight (`:20-60`), the E4/S2 § Sequenced entries (`:478-500`), and **the
entire current close block checked against itself** (`:4865-4970`) ·
`docs/audits/2026-08-04-deposition-clock-design.md` header ·
`docs/audits/2026-08-03-stratigraphic-correlation-design.md` header (P-1…P-5 banners) ·
`docs/audits/2026-08-03-e4-implicit-kernel-design.md` header + arc anchor ·
`docs/audits/2026-08-04-e4-2-convergence-study.md` header ·
the full delta diffs of `docs/design/bodies.md`, `docs/design/ideas.md`,
`docs/design/refinement.md`, `journal/corrections.md`.

**Read in targeted section (whole sections, not grep lines):**
`docs/design/flow.md` §§ 2/6/9/10.3 windows · `docs/design/material-behavior.md`
§§ 3 (edges/U7) and 5 (RATE/stability) · `docs/design/stubs.md` #30 and the clock stub
(`:1285-1345`, `:1980-2005`) · `docs/design/ores.md` § recorded-context-axes (`:155-180`) ·
`docs/design/refinement.md` §§ 4-5 + the fluvial member.

**GREPPED ONLY** (nouns: continuum / hand-pair / taxonomy / term space / axes / chapter /
epoch / stability / byte-identity / membership dither): `tectonics.md`, `water.md`,
`earth-processes.md`, `ecology.md`, `posture-gait.md`, `visuals.md`, `light.md`,
`octree-substrate.md`, `worldgen.md`, `things-that-will-happen.md`,
`material-genesis-notebook.md`, `corpus-knowledge-{notebook,evidence}.md`, `ARCHITECTURE.md`,
`API.md`, `docs/spikes/*`, the pre-2026-08 audits.

**NOT OPENED — a null from these is not a result:**
- `docs/design/north-star.md` (459) — **skipped a SECOND consecutive run.** Untouched in the
  delta, but it is read-first item 0 and the prior run also skipped it on an artifact bound.
  This is now the corpus's longest-standing unexamined read-first surface.
- `docs/design/stubs.md` beyond the two entries above (2,053 lines, **+332 in the delta**) —
  its ordinal space and the five new B7 stubs went unchecked.
- `docs/spines.md` beyond A-7 and § 3's preamble (2,540 lines, **+350 in the delta**).
- `docs/design/posture-gait.md` (421) — untouched in the delta, but B7/B8/quantization
  rulings landed in `bodies.md` next door; **the bodies-side shape-1 check is NOT run.**
- `docs/design/ores.md` beyond § recorded-context-axes; `docs/design/tectonics.md` (1,011);
  `docs/design/water.md` (837); `docs/design/ecology.md` (391).
- The bodies-thread audits (`2026-08-02-joint-limits-b7-design.md`,
  `2026-08-03-b6-body-composition-design.md`, `2026-08-04-longleg-swing-probe.md`,
  `2026-08-04-stand-in-marker-control-scoping.md`) — header-grepped only.
- `ROADMAP.md` § Observed and most of § Sequenced (staleness's axis).

**Did this run close the design-docs shape-1 hole the prior run named? PARTLY.**
`materials.md`, `geology.md`, `flow.md`, `material-behavior.md`, `refinement.md`,
`bodies.md`, `ideas.md` were read against the delta's rulings — that is the geo half, and it
produced F1, F4, F6, F7, F9. **Still unclosed: `north-star.md`, `posture-gait.md`,
`stubs.md`, `spines.md`, `ores.md`, `tectonics.md`, `water.md`, `ecology.md`.** The bodies
half of the hole is essentially untouched.

**Shape 3 (a user design reconciled away):** run against `ideas.md`'s delta only — the
2026-08-03 engine-generality sketch (`ideas.md:757-805`) is recorded verbatim, marked
*recorded ambition*, and nothing in the corpus narrows it. **Clean, but the shape was not run
across `ideas.md`'s older 750 lines.**

**Shape 7 (a premise hiding inside a caveat):** one candidate found and reported as F7; the
shape was run deliberately against the E4-2 fork and the P-1 predicate blocker and found
nothing hidden there.

---

## 2. Findings, ranked by cost-if-unfixed

### F1 · 🔴 The ratified roster ruling names day-one data its own measurement says cannot answer the question

| side | citation (read at `978cb64`) | statement |
|---|---|---|
| **A — the ruling** | `docs/design/materials.md:617-620` (§ DECIDED 2026-08-04, **user-ratified**) | *"**Relations are DERIVED from declared axes, never hand-paired.** Continuum iff two regions adjoin on a declared axis (**day-one data: the FS-A release spectra + the property sheet**; the `settle_energy` derive-don't-table precedent)."* |
| **B — the measurement, same day** | `journal/0157:103-110` (S1) and `.claude/skills/roster/SKILL.md:164-174` (§ 2b ⚠) | *"P-1's M-C rule needs regions adjoining on a declared axis, and **no material declares a region**: `MaterialProps` is point-valued, and **FS-A's spectra describe what a rock sheds, not where it sits**. Any predicate buildable today would need a threshold on a scalar distance — the fitted taxonomy `materials.md` forbids by name."* |

**The pair:** ruling 3 asserts the derivation has its inputs today; the S1 measurement,
landed hours later, says those exact two inputs are the wrong kind of data for it. Both are
true of different things — the spectra *are* term data, they are just not **region** data —
and **the sentence saying so exists nowhere.**

**Why this is ranked first — it is a three-site one-directional pointer, and two of the three
sites are the ones a cold session enters through.** The blocker is stamped in the
`/roster` skill (`§ 2b`) and in `ROADMAP`'s close block (`:4919-4920`, *"PARENT MATERIALS
gained a live blocker"*). It is **NOT** stamped at:
- `docs/design/materials.md:617-620` — the ratified authority every other site cites; and
- `docs/audits/2026-08-03-stratigraphic-correlation-design.md:73-78` (the **P-1 RULED**
  banner), which repeats the premise verbatim: *"computed from the FS-A release spectra +
  property sheet, never a hand pair-list."*

A reader who arrives at either of those — which is the normal path, since the ruling doc is
where `/roster` § 3 and the P-1 banner both point — is told the predicate is buildable.

**Proposed disposal (my reading, unratified):** **banner, not correction.** Nothing is
falsified — ruling 3 stands as a *rule*; what is wrong is the parenthetical's implication of
sufficiency. A one-line stamp on `materials.md` § DECIDED 2026-08-04 ruling 3 and on the
P-1 banner, pointing to `journal/0157` § *What S2 inherits* item 1 and the parent-materials
design pass. **Not user-owned** — the ruling is untouched; only its input inventory is.

---

### F2 · 🔴 `ROADMAP.md` § In flight still runs a diagnosis its own close block records as finished and its hypothesis as wrong

| side | citation | statement |
|---|---|---|
| **A** | `ROADMAP.md:31-43`, § **In flight** | *"**⚠ E4-2 IS NOT GREEN — DIAGNOSIS IN FLIGHT 2026-08-04.** … **both new structural fixtures FAIL** — the implicit step **flipped the grid-scale mode** … **Hypothesis under test** (integrator's, unproven): the limiter-as-projection guarantees mass and h ≥ 0 but **not no-overshoot** … meaning accuracy sub-cycling is *required for monotonicity* … The diagnosis agent repairs fmt/clippy, bisects the true monotonicity bound…"* |
| **B** | `ROADMAP.md:4939-4941`, the **2026-08-04 GEO-4 close block**, § *Falsified* | *"and, unfiled because it was never recorded as a claim: my swap-mechanism hypothesis for E4-2's monotonicity failure was **wrong** (the limiter chain is monotone; **Picard is the culprit**)."* |
| **B′** | `docs/audits/2026-08-04-e4-2-convergence-study.md:14-20` (header) | *"the limiter chain IS monotone to a=1e8; **Picard relinearization breaks monotonicity at a=2.0**"*; the binding bound is **0.85**, the arm is **2.2× slower than explicit**, and *"the decision this document forces"* is a fork. |

**One artifact, two sections, opposite states — and the stale one is § In flight,** which is
the first thing every cold session and every dispatched agent reads. It also **names a
diagnosis agent as running**; § *Machine state at close* two hundred lines down says
*"No agents running."*

**Blast radius:** maximal. A session picking up work from § In flight will re-dispatch a
completed diagnosis, or reason from the refuted swap-mechanism hypothesis.

**Proposed disposal (my reading):** strike-and-replace the § In flight entry with the fork
(`ROADMAP:4910`, *"re-price K2-alt vs K3 on the measured numbers, or park"*), keeping the
refuted hypothesis visible as struck text per house style. **Not user-owned.**
*Cross-sweep note: the board's disposition is the **staleness sweep's** territory; reported
here because the contradiction is between two sections of one artifact, which this sweep's
scope rule requires it to check.*

---

### F3 · 🟠 `ROADMAP` § Sequenced calls the `dc-core` venue a live open ratification; it was ruled, emphatically, in the delta

| side | citation | statement |
|---|---|---|
| **A** | `ROADMAP.md:486-488`, § Sequenced *"E4-2 + E4-3"* | *"**Live ⚠ NEEDS RATIFICATION: the `dc-core` venue + its rayon rider** (audit U-3…)"* |
| **B** | `docs/audits/2026-08-03-e4-implicit-kernel-design.md:3-8` | *"**✅ U-3 RULED 2026-08-04 (user, emphatic) — the venue is `dc-core::field`** … 'The engine owns primitives. The engine owns primitives. The engine owns primitives.' The rayon rider rides with it … ratified as part of the venue."* |
| **B′** | `docs/dependency-graph.md:168` (E4 row) | *"**✅ VENUE RATIFIED 2026-08-04 (U-3 …; rayon rider included; U-5 closed as executed)**"* |

**Provenance is decisive:** side B is the user's own words at the record. Side A is a board
entry nobody re-read when the ruling landed. *This is the same shape as F2 in the same
artifact — the E4 entries were not swept when the picks closed.*

**Proposed disposal:** strike the NEEDS RATIFICATION clause at `ROADMAP:486-488` and point
to the audit header. **Not user-owned** (the user already ruled).

---

### F4 · 🟠 `flow.md`'s correlation rule says *use the chapter stamp*; the delta made the correlation key the **epoch**

| side | citation | statement |
|---|---|---|
| **A** | `docs/design/flow.md:88-90` (§ 2, *"two rules that keep this honest"*) | *"**Correlate by CHAPTER, never by slot index.** Depth is time *within a column*. Across columns, surfaces are **diachronous** … `DepUnit` carries a chapter stamp; **use it**."* |
| **B** | `docs/design/stubs.md:2000` (the clock stub) | *"**The correlation partition uses epoch, not chapter**, so it is unaffected."* |
| **B′** | `journal/0157:38-44` (S1, shipped) | *"Every `DepUnit` now carries the raw runner tick … the shared partition is … the **sorted union of the epochs the parents stamp**, and interval *k* simply **is** epoch *k*."* |
| **B″** | `journal/0154:28-31` | *"**Chapter is a pure function of epoch** … The record was spending 8 bits on a 3-bit quantity that is itself a **lossy projection** of the clock the correlation design was trying to reconstruct."* |

**Both halves of flow.md's rule are still right in their own frame** — depth is not time
across columns, and surfaces are diachronous. What is now false is the **prescription**:
chapter is the coarser shadow, and the shipped correlation kernel keys on epoch. flow.md
carries no banner, and it is the live design doc for refinement member #1 (the fluvial
operator) — the next consumer of exactly this rule.

**Related, and checked NULL** (see § 3): `flow.md:264` (aggregation window = one chapter =
25 epochs) and `flow.md:478` (*"units never merge across a chapter boundary"*) are **still
true** — the epoch lives in the packed unit's second word, which `key_bits()` structurally
never reads (`journal/0154:36-43`). Do not sweep them with the same stroke.

**Proposed disposal (my reading):** a banner on `flow.md` § 2 — *the rule survives, the key
moved to `epoch`; chapter remains the aggregation-window granularity* — plus the reciprocal
pointer from `stubs.md:2000`. **Not user-owned.**

---

### F5 · 🟠 The E4 design pass carries no pointer to the study that found its ruled answer unadoptable

| side | citation | statement |
|---|---|---|
| **A** | `docs/audits/2026-08-03-e4-implicit-kernel-design.md:50` (header, last line) | *"**ALL FIVE E4 PICKS ARE NOW CLOSED** … **E4-2 is dispatchable.**"* — and no banner anywhere in the file names the study. |
| **B** | `docs/audits/2026-08-04-e4-2-convergence-study.md:3-4, 14-20` | Names its parent in line 3 (*"executes `2026-08-03-e4-implicit-kernel-design.md` § 5, under that document's five ruled picks"*) and then: *"at the binding `A_ACC = 0.85` the implicit arm measured **2.2× SLOWER than the explicit scheme it was ruled to replace** (U-1, K2) … the choice is re-price K2-alt vs K3 … **or park E4-2**."* |

**This is exactly the corpus's named one-directional-pointer defect** (CLAUDE.md read-first
item 5): the child names the parent; the parent is silent. A reader who opens the design pass
— which is what both `ROADMAP:488` and `dependency-graph.md:168` point at for U-1…U-5 — is
told the picks are closed and the build is dispatchable. The obligation, per the ratified
rule, lands on the writer of the refutation, in the same commit; that commit is already past.

**Proposed disposal:** a mutable-header banner on `2026-08-03-e4-implicit-kernel-design.md`
naming the study and the owed fork. **Not user-owned** (the *fork* is user-owned; the
back-pointer is clerical).

---

### F6 · 🟡 `material-behavior.md` still books a discharged stub as owed

| side | citation | statement |
|---|---|---|
| **A** | `docs/design/material-behavior.md:414-416` (§ 5, RATE) | *"**It does NOT own stability substepping** (user, 2026-07-29). A field solver takes `dt` from RATE and sub-divides it *internally* … only the kernel knows its own stability constant. **`stubs.md` § 30's remaining half.**"* |
| **B** | `docs/design/stubs.md:1295-1305` (entry 30, banner) | *"**🟢 FULLY DISCHARGED 2026-08-03 (E4-1…).** The DERIVED half landed at the heir the user named: the sub-cycle derivation and the bound live in the S-10 kernel itself now … an out-of-bound step is **inexpressible from a pass**."* |
| **B′** | `docs/dependency-graph.md:187` (P1 row) | *"the sub-cycle is the kernel's since 2026-08-03 (E4-1) … `stubs.md` § 30 fully discharged."* |

**The first sentence of side A is still exactly right** (it is the user's 2026-07-29 ruling
and it is what E4-1 built). Only the trailing pointer is stale — a shape-6 summary that
outran its source. Low cost, cheap fix, but `material-behavior.md` is a ratified read-second
doc and the sentence reads as *still owed*.

**Proposed disposal:** replace the trailing clause with *"discharged by E4-1 — `stubs.md`
§ 30"*. **Not user-owned.**

---

### F7 · 🟡 The `/roster` skill forbids as a *hand table* the thing a user ruling made an *authored table*

| side | citation | statement |
|---|---|---|
| **A** | `.claude/skills/roster/SKILL.md:115-118` (§ 2 step 2) | *"**Derive, never hand-pair.** Any relationship this member has to others (continuum membership, **weathering products**, settle ordering) must be computed from its terms … **A hand list of related materials is a fitted taxonomy and gets rejected at review.**"* |
| **B** | `docs/design/material-behavior.md:231-245` (§ 3, **U7 RULED 2026-08-02, user**) | *"**PRODUCTS — 'R2 it is, authored edges.'**"* — release spectra are pack-**authored** edge products, eight literature-cited vanilla tables; *"the **authored-table** shape is also what keeps bimodal release expressible (granite → …)"*. |

**Weathering products *are* the release spectra.** Side B is a **user ruling**; side A is an
assistant-drafted procedure clause. And `materials.md:617-620` — the ratified philosophy the
skill compresses — forbids only a hand list of ***continuum pairs***; it treats the spectra
as *data* the derivation reads. **So § 2 step 2's enumeration widened a user ruling by two
words**, and it does so in the file that gates every future roster change.

**This is the shape CLAUDE.md § Conventions names by name** — *a user-originated design may
not be superseded by an implementation slice* — in its mildest form: a procedural doc, not a
slice, and a widening rather than a replacement. Reported as a pair, **not** adjudicated.
⚠ **CONTESTS `material-behavior.md` § 3's U7.**

**Proposed disposal (my reading, unratified):** narrow step 2's enumeration to what
`materials.md` ruling 3 actually says — *continuum/adjacency relations are derived; edge
PRODUCTS are authored content per U7* — **but the skill is user-directed and its dispositions
were ratified, so if the user intended the wider reading this is a user call.**
**USER-OWNED if contested.**

---

### F8 · 🟡 `bodies.md` restates, 30 lines below its own correction, the figures that correction retires

| side | citation | statement |
|---|---|---|
| **A — the banner** | `docs/design/bodies.md:229-236` | *"biped … **6.970** (banner: 6.977) … longleg **7.729** (banner: 7.742) … stout **4.291** (banner: 4.286)"* + *"The three-decimal figures were **carried into the banner from a conversation**; journal/0148's own two-decimal table … is correct and is **the one to quote**. **ROADMAP § Sequenced repeats the banner's three-decimal version and is owed the same stamp** (integrator)."* |
| **B — the body** | `docs/design/bodies.md:265` (the `▶▶` DECIDED block) | *"the three shipped bodies barely move: biped **6.977 → 7**, longleg **7.742 → 8**, stout **4.286 → 4**."* |

The correction **named ROADMAP and stamped it** (`ROADMAP.md:394-397` now carries the
corrected figures with the history) — and **did not name the paragraph 30 lines below
itself**. Classic corrections-#65 geometry, compressed into one screen.

**Nothing the ruling rests on moves** (all three `N` are unchanged, as the banner says), so
the cost is a reader quoting the wrong decimals from the section that is *below* the
correction. **Immutable-body/mutable-header arguably covers it** — the `▶▶` block is a dated
ruling record — but the header does not point at its own body's instance.

**Proposed disposal:** one inline *(see banner above — 6.970 / 7.729 / 4.291)* at `:265`.
**Not user-owned.**

---

### F9 · 🟢 `refinement.md`'s smoothness stamp still describes the P-1 pick as pending

| side | citation | statement |
|---|---|---|
| **A** | `docs/design/refinement.md:30-35` | *"…whose § 3.3 surfaces a live tension between this principle and 2026-07-19's dress-every-contact ruling — **the user-owned P-1 pick, pending at stamp time**."* |
| **B** | `docs/audits/2026-08-03-stratigraphic-correlation-design.md:73-78` | *"**✅ P-1 RULED 2026-08-04 (user) — M-C, WITH THE PREDICATE DERIVED** … **§ 3.3's tension resolves rather than picks a side**: smoothness governs continua, dress-every-contact governs the discrete cut's texture."* |

*"Pending at stamp time"* is honest and time-stamped, so this is the mildest possible
instance — but the tension it flags is **resolved**, and `refinement.md` is where a
refinement-member author looks. **Low cost. Proposed disposal:** append the resolution to
the same paragraph. **Not user-owned.**

---

## 3. Nulls worth reporting — suspicion pairs checked and found clean

1. **`dependency-graph.md` § 0a's WITHDRAWN byte-identity/additivity bar vs the rest of the
   corpus.** Searched `grep -rn "byte-identical|byte identity|additivity|good enough to
   ship" --include=*.md docs/ ROADMAP.md CLAUDE.md .claude/skills/` at `978cb64`. **Only
   two sites state the bar, and both state it as WITHDRAWN** —
   `dependency-graph.md:69-91` and `CLAUDE.md:398-404`. **No third doc asserts it as a live
   bar.** The withdrawal propagated correctly, same day. Clean.
2. **The P11 slice-3 membership dither, superseded-as-plan.** Traced across every `.md`
   mention. `docs/audits/2026-08-02-p11-slice3-design.md:4` carries its own supersession
   banner; `docs/audits/2026-08-03-stratigraphic-correlation-design.md:86-92` carries the
   formal P-5 ratification with the retirement list; `dependency-graph.md:197` (P11 row) and
   `ROADMAP.md:478-482` both state it as retired-as-plan with the S2 execution list.
   **No doc presents it as the live plan.** Clean — and this is the best-propagated
   supersession in the delta.
3. **`flow.md`'s aggregation-window claims.** `:264` (window = one chapter = 25 epochs) and
   `:478` (*"units never merge across a chapter boundary"*) survive the deposition clock
   intact, because the raw epoch lives in the packed unit's **second word** and `key_bits()`
   masks the **first** only (`journal/0154:36-43`). Only the *correlation* prescription moved
   (F4). Clean — and worth recording, because a careless F4 fix would take these with it.
4. **`geology.md` § Backbone vs the roster philosophy.** *"Processes bind to classes, not
   instances"* (`:9-10`) and *"Classes are contracts — parameter schemas"* (`:12-18`) sit
   cleanly under `materials.md:640-641` (*"the class roster remains a SELECTION contract …
   never a relation source"*) and `/roster` § 2 step 3. The intrusive/extrusive class split
   is a **formation window**, not a claimed material relation. Clean.
5. **`ideas.md`'s 2026-08-03 engine-generality user sketch** (`:757-805`). Recorded verbatim,
   explicitly marked *recorded ambition* under the CLAUDE.md doctrine, assistant notes fenced
   and labelled, structures/communities/agents tail correctly parked behind the P9 gate.
   **No doc narrows or reconciles it.** Clean (shape 3, on the delta only).
6. **The delta's new `corrections.md` entries stamp their targets.** #97 (marker control),
   #98 (pits bar), #99 (the gap claim — three named sites, header text corrected in place,
   testimony left alone), #99b, #100 (the stop channel — *"banners on all three sites"*).
   The one **self-declared** miss is #100's *"Not fixed: `target_tick`'s A-4 row in
   `spines.md` § 3 — handed up, not written"*, which is **spine-audit's** territory. This
   sweep found no *undeclared* one-directional correction edge in the delta — a genuine
   improvement over the prior two runs.
7. **`ARCHITECTURE.md` § Schedule / epoch-0 rulings vs `material-behavior.md` § 5 RATE.**
   Grep-level check only (nouns: epoch, sub-turn, `Seed`); no disagreement surfaced. **Weak
   null — grepped, not read.**

---

## 4. What I would run next, with another hour

1. **`north-star.md`, read in full, against the delta's rulings** — it has now been skipped
   two consecutive FULL runs on artifact-count bounds, it is read-first item 0, and the prior
   run's own § 6 example of a summary outrunning its source is *inside that file*. The
   specific pairs to look for: § Materials (the `Providers` code-shape disambiguation) vs the
   roster term-space ruling; § Passes' *"trusted … first-party"* vs § Deviations 1.
2. **`posture-gait.md` (421 lines) against `bodies.md`'s delta** — B7 (joint limits), B8
   (proportion variation), the per-cycle quantization ruling and the B6-a mass integral all
   landed in `bodies.md`, and `posture-gait.md` is the *ratified* companion that was not
   touched. This is the bodies half of the shape-1 hole and it is entirely unrun.
3. **`stubs.md` in full (2,053 lines, +332 in the delta)** — its ordinal space, the five new
   B7 stubs (#45-#49), #51-#54, and whether each new entry's *heir* still names a live locus.
   The delta added more to `stubs.md` than to any design doc.
4. **The shape-7 pass over every `until X lands` / gating clause in `ores.md`,
   `earth-processes.md` and `water.md`** — corrections #70 and #98 were both this shape and
   both were found by looking at clauses that hold a **user-owned** fork shut. `ores.md`
   § 8's conceptual revisit is listed as *"owed, genuinely unscheduled"*
   (`dependency-graph.md:347-348`), which is exactly the *"blocked item looks handled"*
   signature.
5. **`spines.md` § 3 and § A-7 against P11 slice 3 + the clock slice** — A-7's
   `Litho::reference_material` instance and the `of_material` stand-in both have demolition
   orders pointing at P11 slices 2 and 4, and slice 4 has not landed; verify the citations did
   not drift again (they were refreshed twice in three days).
