# doc-topology sweep — 2026-07-29 (FULL)

- Mode: **FULL** (76 commits since watermark `f652b60`, ≥25 trigger).
- **HEAD at dispatch and at every quotation below: `72fbe86`.** Where a quote is read at a
  different commit it says so. *(SKILL rule: a quotation without a revision is not a
  quotation — corrections #67.)*
- Prior watermark: `f652b60` (the 2026-07-28 100 %-corpus baseline,
  `docs/audits/baseline-2026-07-28/`).
- Read-only sweep. **Findings are reported, not applied** (standing rule 2026-07-29); no
  `corrections.md` entry filed — the one proposed disposition is in § 4.

---

## 0. Scope actually covered, and what was NOT opened

This FULL run is a **spine-following pass** over the named high-churn arc plus the read-first
set — not a second 100 % re-read (the baseline at `f652b60` was that, six days of subjective
time ago and unchanged in most of the corpus).

**Read in full or in the relevant whole section:** `CLAUDE.md` · `docs/dependency-graph.md`
(all 148) · `docs/design/refinement.md` (all 285) · `docs/design/flow.md` (all 679) ·
`docs/ARCHITECTURE.md` §§ 545–745 · `docs/design/material-behavior.md` § Cadence (336–460) ·
`docs/design/north-star.md` (1–60, 92–182, 225–264, 296–345, 415–445) · `docs/spines.md`
(352–400, 530–560, 1474, 1520–1560) · `ROADMAP.md` (1–120, 651–795, 1013–1085, 2425–2470,
2760–2800, 3535–3644) · `journal/corrections.md` #71/#72/#73 in full + the heading index ·
`docs/audits/2026-07-29-member0-coarsefield-design.md` (header + § 1) ·
`docs/audits/2026-07-29-refinement-coupling-priors.md` (header, § 1.1, § residuals) ·
`docs/audits/2026-07-24-palette-quant-generation-diagnosis.md` (banner + § 0) ·
`docs/spikes/S19-flow-record-cost-results.md` (header + the pairing paragraph) ·
`docs/design/pass-declaration-history.md` (1–45) · `docs/design/geology.md` (350–380) ·
`.claude/skills/wrap/SKILL.md` (65–130) · `.claude/skills/doc-topology/SKILL.md` (all) ·
and `crates/dc-worldgen/src/deeptime/flux.rs` module docs (1–175), because the brief named it
as one of four sites.

**NOT opened** (so a null from this sweep over them is not a result): the journal 0122–0127
bodies (only `0127:110` spot-checked) · `ROADMAP.md` §§ 155–650 and 1100–2374 · `ROADMAP.md`
§ Observed outside the two entries in F2 · `ROADMAP-history.md` (spot greps only) · the nine
`baseline-2026-07-28/S*` audits (greps only) · `docs/design/{water,tectonics,earth-processes,
ores,materials,ecology,stubs,ideas,visuals,light,corpus-knowledge-*}.md` ·
`docs/spikes/*` except S19 · `docs/API.md` · `docs/rendering/PIPELINE.md`.

---

## 1. Contradiction pairs, ranked by blast radius

### F1 · 🔴🔴🔴 The ratified refinement design has almost no inbound pointer — and the three surfaces that own the work all still say the surface is undesigned

**The shape:** a design pass *happened*, its output was *user-ratified*, its first member
*shipped and was walked* — and the only live inbound citations of the resulting document are a
**close block** (which gets archived) and one passing mention. Every read-first surface a cold
session actually lands on still describes the tier as an open hole.

| side | site | text |
|---|---|---|
| **stale** | `docs/design/north-star.md:121-124` (read-first item **0**) | *"**What is still open:** the *mechanism* — which kernels are primitives, what the operator authoring shape is, and how the pure-fn contract is enforced across the backends. **The boundary is decided; the surface is not.** ROADMAP § Sequenced 'REFINEMENT PRIMITIVES' still owns that work."* |
| **stale** | `ROADMAP.md:1041` (the entry north-star routes to) | *"The mechanism is not designed; the direction is set."* — and `:1042` *"**FIRST SLICE — a DESIGN PASS, not code**"* |
| **stale** | `docs/ARCHITECTURE.md:617-619` (read-first item 3) | *"**The engine currently has ZERO refinement primitives** — the tier is decided and undesigned, and its first member is unscoped."* |
| **stale** | `docs/ARCHITECTURE.md:621-624` | *"Candidate first members, **not yet ruled**: `dc-core`'s `sample_dithered` (the cake law, **built and called by nothing**) and an octaves/facies-driven design the user recalls ratifying — under investigation 2026-07-29, because the corpus may hold **several contradictory ratified designs for one problem**."* |
| **current** | `docs/design/refinement.md:3` | *"**Status: ~~PROPOSED~~ CAUTIOUSLY RATIFIED — user, 2026-07-29, same day.**"* |
| **current** | `docs/dependency-graph.md:51` (E5 row) | *"**CAUTIOUSLY RATIFIED 2026-07-29 (user)** … **#0 has its design pass** (`docs/audits/2026-07-29-member0-coarsefield-design.md`) **and its FIRST BUILD SLICE SHIPPED 2026-07-29 (journal/0125), WALKED same evening**"* |
| **current** | `docs/spines.md:1474` | *"**NOW CALLED (2026-07-29): `CoarseField<T>` + `from_cells` + `sample_dithered` + `Registration`, from `collapse.rs::surface_class` / `class_window`**"* |
| **current** | `docs/dependency-graph.md:118-119` | *"**They are not rivals — they compose**, and nothing in the corpus said so, which is why two ratified designs read as contradictory for a week."* |

**Which side is current, and the evidence:** the ratified side. It is **user-originated**
(`refinement.md:6-10` carries the ruling verbatim), dated one day later than every stale
clause, and it is confirmed **by shipped code** — `sample_dithered` has a production caller,
so ARCHITECTURE's *"built and called by nothing"* is falsified by the tree, not by an opinion.
The *"several contradictory ratified designs for one problem"* worry is likewise resolved:
dependency-graph § 3 records that octaves and dither **compose**.

**The reciprocity failure, measured.** Pathspec
`grep -rn "refinement.md" --include=*.md .` at `72fbe86` returns **5 hits**: `ROADMAP.md:3562`,
`:3582`, `:3589`, `:3590` — **all four inside the close block** — and `docs/spines.md:377`.
Zero from `north-star.md`, `ARCHITECTURE.md`, `CLAUDE.md`, or the ROADMAP entry that owns the
work. **This is corrections #73's mechanism reproduced at full scale within 24 hours of #73
being filed**: a resolution whose only live home is a close block, which is a handoff that gets
archived.

**Recommendation (labelled a recommendation).** Strike/annotate the four stale clauses and give
each a pointer to `refinement.md`; the ROADMAP entry can stay `🔴` (nothing is *built* as a
runner) but `:1041-1042` are discharged. Whether the entry is re-headed is an
integrator/user call.

---

### F2 · 🔴🔴 `ROADMAP.md` § Observed still asks the 460-vs-28.8 question corrections #73 exists to close — 345 lines from its own answer, in the same file

| side | site | text |
|---|---|---|
| **stale** | `ROADMAP.md:2784-2788` (the palette-quant STATION entry) | *"**One live-probe question left:** are the visible squares the 460 m deep cells or the 28.8 m chunk staircase — settle by reading dominant surface material per chunk across an east-coast patch … **Planning held.**"* |
| **current** | `ROADMAP.md:2440-2450` | *"**The U3 checkerboard's dominant signal is the 28.8 m MEMBER STEPPING, settled 2026-07-24** — and the answer never flowed back into the audit that asked it"* |
| **current** | `journal/corrections.md:3026` (#73) | *"'The 460 m-vs-28.8 m checkerboard question has never been probed' … **falsified the same evening by the user's memory**"* |

**Which side is current:** the settled side — **user-originated** (the user's memory is the
falsifier), with primary evidence in corrections #45.

**Why it survived:** #73's own repairs paragraph (`corrections.md:3056-3058`) says *"the ROADMAP
entry converted from probe-first to settled-with-consequence."* A **new** entry was written at
`:2440`; the **old** probe-first clause at `:2784` was never struck. The station entry is the one
a walker reads before a walk, so this is the live-cost end.

---

### F3 · 🔴 Face-pairing — the four sites verified, plus **two additional sites** the ROADMAP addendum does not enumerate, plus a stale premise inside `flux.rs` itself

*Disposition reserved to the main-session fluvial design pass, per brief. This section only
verifies and extends the inventory.*

**The four sites named in `ROADMAP.md:3556-3564`, all confirmed at `72fbe86`:**

1. `docs/design/flow.md:133-145` — *"**⚠ UNDER-SPECIFIED — the slot-pairing rule (S19,
   2026-07-25)** … Candidate rule, **NOT RATIFIED** … **Resolve before any refinement/expression
   slice.**"*
2. `docs/design/flow.md:804-816` (§ 11.5) — *"**RATIFIED (user):** flow pairs by **what confines
   it**"* — 671 lines below the flag, no cross-stamp in either direction.
3. `crates/dc-worldgen/src/deeptime/flux.rs:93-108` — *"**The slot-pairing rule — stated, because
   flow.md § 2.2 does not** … a rule is needed and **the design document is silent**. **This
   slice pairs lateral faces by CHAPTER**."*
4. `docs/design/refinement.md:299-302` — *"**The face-pairing rule** (`flow.md` § 2 flag):
   under-specified, not ratified"* + `docs/audits/2026-07-29-refinement-coupling-priors.md:263-265`
   — *"the face-pairing rule is UNDER-SPECIFIED / NOT RATIFIED … **Live blocker on the tier's
   first slice.**"*

**ADDITIONAL site A — `docs/dependency-graph.md:51`** (E5 row, last clause):

> *"**Live blocker for #1: flow.md's face-pairing rule, unratified**"*

This is **read-first item 1c**, and the file's stated job (`:3-8`) is answering *"what is
genuinely ready to start today."* It is not on the addendum's list of four. A cold session
following the prescribed start-of-session order reads this before it reads the close block.

**ADDITIONAL site B — `docs/spikes/S19-flow-record-cost-results.md:346-353`**, unstamped:

> *"**§ 2.2's 'a face is shared … agree from both sides by construction' — a flagged gap, not a
> cost.** … Sharing therefore holds for the *cell pair* but needs a stated pairing rule at the
> *slot* level — by depth, presumably, not by index. … **Worth resolving in the design.**"*

S19 is the artifact flow.md § 2.2's flag cites by name (*"the slot-pairing rule (S19,
2026-07-25)"*), and flow.md § 11.5 ratified a rule **the same day** S19 was measured. S19 carries
no banner. *(S19's banner status is already a live baseline escalation — `baseline-2026-07-28/
S8-cold-artifacts-banners.md:292-298`, item B8 — but that escalation is about the cost
itemisation, not about this paragraph.)*

**ADDITIONAL, and it is a claim refuted in its own neighbourhood (SKILL shape 2):** the residual
paragraph that files the bound regime's pairing as *not yet forced* rests on a premise the same
module doc contradicts ~55 lines above it.

| | site (read at `72fbe86`) | text |
|---|---|---|
| the premise | `flux.rs:110-113` | *"**Slice 1 records no bound flux at all**, so the question is not yet forced; **it is a design decision the user owns**, filed with continuation (c)"* |
| its refutation, same file | `flux.rs:45-46` | *"**vertical** (slot ↔ slot in a column) \| `FaceKey::Down`, `FaceKey::Up` \| **populated since continuation (a)** — the head field's exchange"* |
| and again | `flux.rs:49-56` | *"**The vertical faces were slice 1's honest empty, and continuation (a) filled them.** … infiltration where the water table stands below the ground, **artesian rise** where a confined potential stands above it"* |
| the same premise, in the design doc | `docs/design/flow.md:813` | *"Slice 1 implements the first for lateral free-surface flux and **records no bound flux**, so nothing is forced."* |

**Reading, offered not imposed:** the *pairing* conclusion looks safe — vertical faces pair
**within** a column, which `journal/0098:252-256` states explicitly (*"the slot-pairing question
that § 2.2 [owes]"* does not arise for them). What is stale is the **premise as written**: bound
exchange *is* recorded now, in the vertical family, so *"records no bound flux at all"* is false
at HEAD in both sites. **A reader who checks the premise and finds it false has no way to know
the conclusion survives** — that is the whole cost. The design pass owns which sentence changes.

---

### F4 · 🟠 Three values for `runner.rs`'s size — and the "927 code lines" figure a pending **user call** is framed on is one commit out of date

| site | claim |
|---|---|
| `ROADMAP.md:716` | *"`runner.rs` is **1,694 lines** against the 700-line threshold (2.4×) **and this slice added to it**."* |
| `docs/spines.md:546` | *"Honest limit: the file is **still** 2.4× over (**1,654 lines**, of which **927 are code**)"* |
| `docs/design/pass-declaration-history.md:27-32` | *"was **1,694 lines** … landing the file at **1,654** … **still 2.4× over threshold because 927 of its lines are code**"* |
| `ROADMAP.md:3629` | *"Waiting on user calls: … runner.rs module split (**927 code lines**)"* |
| **measured at `72fbe86`** | `wc -l crates/dc-worldgen/src/deeptime/runner.rs` → **1,908** (≈ **2.7×**) |

`git log --oneline -- crates/dc-worldgen/src/deeptime/runner.rs` puts **`de85bac`** (the
`Schedule` slice) **after** `02aa81a` (the archaeology extraction), so the file grew ~254 lines
*after* the 1,654/927 measurement, and the growth was code, not comment.

**Why this is a docs-vs-docs finding and not only staleness:** the three documents agree with
each other but **two different numbers each read as current in a different file** —
`ROADMAP.md:716`'s 1,694 is pre-extraction and its own sentence says *"this slice added to
it"*, while `spines.md:546` says *"still"*. The number that matters is the one at `:3629`,
because it is the input to a decision the user has not yet taken.

---

### F5 · 🟠 `spines.md` § S-6 quotes a field name that `pass-declaration-history.md`'s own banner announces was renamed

| side | site | text |
|---|---|---|
| **stale** | `docs/spines.md:551` | *"`writes {Saprolite}`, **`cadence: Cadence::EVERY_EPOCH`** (`period = 1` until journal/0123)"* |
| **current** | `docs/design/pass-declaration-history.md:14-15` | *"**`DeepPass::cadence` is now `DeepPass::schedule`**, a `Schedule` sum type carrying the `Cadence`."* |
| **current** | code at `72fbe86` | `runner.rs:304` `pub schedule: Schedule,`; `schedule: Schedule::EVERY_EPOCH` at ~20 declaration sites |

Pathspec: `grep -rn "cadence: Cadence|DeepPass::cadence|Cadence::EVERY_EPOCH" --include=*.md .`
→ 3 hits; only `spines.md:551` is stale (the other two are the banner and the body it corrects).
Notable because `spines.md:544` in the *same bullet* asserts *"journal/0123's RATE commentary is
live contract and was left whole. **Line refs below are refreshed to that extraction commit**"*
— the refresh caught the line numbers and missed the rename that landed after it.

---

### F6 · 🟠 `dependency-graph.md` § 4 item 2 contradicts its own E5 row, 90 lines up, in read-first item 1c

| side | site | text |
|---|---|---|
| **stale** | `docs/dependency-graph.md:141-142` | *"2. **E5 — the refinement design pass.** A whole tier with **zero members** that the north star requires for plugin-agnosticism, with its inputs already built and idle."* |
| **current** | `docs/dependency-graph.md:51` | E5 is *"CAUTIOUSLY RATIFIED … #0 has its design pass … and its FIRST BUILD SLICE SHIPPED 2026-07-29 (journal/0125), WALKED same evening"* |

Items **1** and **3** of that same "Ready to start today" list were struck for the same day's
work (`:138`, `:143`); item **2** was not. The file's § 5 Process says a slice that shipped and
left a row stale is *"an unclosed loop"* — this is that, inside the file that says it.

---

### F7 · 🟡 `material-behavior.md` § Cadence: "four orthogonal axes" vs "the one axis of the three", 28 lines apart

| site | text |
|---|---|
| `docs/design/material-behavior.md:340` | *"The scheduler has **four orthogonal axes**, and the runner declares **all four** per pass."* |
| `docs/design/material-behavior.md:368` | *"**RATE — ✅ BUILT 2026-07-29** … The one axis of **the three** that is no longer a declaration."* |

Both were touched in the same editing pass (`:341-343` explicitly reconciles the count: *"Three
until 2026-07-29; SCHEDULE was ratified and built that day"*). And with SCHEDULE built the same
day, **two** of four axes are no longer declarations, not one. Low blast radius; reported because
this is the enumeration-completeness shape the corpus has now caught repeatedly, and it went
stale *inside the paragraph that corrected the count*.

---

### F8 · 🟠 north-star.md's trust-tier one-directional edges are STILL LIVE at `72fbe86` — re-verified rather than inherited

The `doc-topology` SKILL lists this as *"confirmed live"*; per its own rule (*a sweeper
inheriting that list must re-verify each item*), re-read at source:

**Deviation 2 names three sections by name** — `docs/design/north-star.md:437-439`:

> *"So the **capability-tiering** in § 'The core / plugin boundary', **§ Passes ('field passes …
> trusted … first-party')**, and **§ Refinement ('a global solver is a trusted-tier
> capability')** is **EXPLICITLY NOT THE MODEL** and must not shape any design."*

**All three named targets remain unstruck and carry no back-pointer:**

- `north-star.md:230-231` (§ Passes) — *"Native-backend and first-party — because they are
  **trusted** and hot, *not* because they are core."*
- `north-star.md:258-262` (§ Refinement) — *"**Capability, not core, gates the tiers.** … the
  untrusted tier gets material declarations + cellular passes + data; registering a new field
  pass (a global solver) is a **trusted-tier capability**. Same authoring shape, different
  granted powers."*
- `north-star.md:250` (§ Refinement) — *"it runs native because it is **trusted** and hot"*

**Two further sites not on Deviation 2's list and not banner-covered:**

- `north-star.md:11-14` (the blogworthy line, **the third paragraph of read-first item 0**) —
  *"solves — one better than its ancestral space — the untrusted-mod problem with a **sandbox
  tier** behind a single authoring shape."*
- `north-star.md:341-342` — *"**The tier system** means we never need that corner: we never run
  untrusted native."* (§ 301's banner calls the split *"deferred product infrastructure"*, which
  is weaker than Deviation 2's *"EXPLICITLY NOT THE MODEL"* and than `CLAUDE.md`'s *"no
  difference in permission between native and WASM"*.)

`CLAUDE.md` read-first item 0 states the live rule. **The sections a reader lands on do not** —
and a placement argument from trust is exactly what item 0 forbids.

---

### F9 · 🟡 `geology.md`'s explicitly-open **user** question is answered in `refinement.md`, and geology.md carries no pointer — with a possible mis-citation inside the answer

| site | text |
|---|---|
| `docs/design/geology.md:367-372` | *"**The local procedures are a reflection of the deep-sim processes** … *In what sense they must agree is an explicitly open design question* (user: **"i do not know in what sense they must agree, but on a high level that appears clear to me"**) — recorded as the guiding constraint, **to be sharpened slice by slice rather than resolved here**."* |
| `docs/design/refinement.md:71-74` | *"an enhancement must be a *pure function of the recorded history* that *agrees with the deep process*. **In what sense it must agree was left open there; § 4 below is this document's answer.**"* |

Two things, both flagged rather than resolved:

1. **No back-pointer.** `geology.md` still reads as an open user question. Pathspec:
   `grep -n "refinement.md" docs/design/geology.md` → 0 hits.
2. **The cross-reference may name the wrong section.** `refinement.md` § 4 is *"The coupling —
   record families"*; the clause that actually answers *"in what sense must it agree"* is § 5
   **Law 1** (`refinement.md:189-198`, the anti-carve law: *"An operator's expression varies
   **only** with recorded continuous quantities"*).

**Cannot rule, and deliberately not ruling:** whether a ratified assistant-assembled doc may
close a question the user left explicitly open *"to be sharpened slice by slice"* is a user
call. `refinement.md` *was* ratified, so this is not the corrections-#65 unilateral shape — but
it is a user-owned open question closed in a doc `geology.md`'s reader will not reach.

---

### F10 · 🟢 `2026-07-29-refinement-coupling-priors.md`'s mutable header states its consumer's status wrongly

| site | text |
|---|---|
| `docs/audits/2026-07-29-refinement-coupling-priors.md:6` | *"Consumed by [`docs/design/refinement.md`](../design/refinement.md) (**PROPOSED**), whose § 4/§ 5 are the synthesis of what is below."* |
| `docs/design/refinement.md:3` | *"**Status: ~~PROPOSED~~ CAUTIOUSLY RATIFIED — user, 2026-07-29, same day.**"* |

One-line header fix; immutable body untouched. Noted also because that audit's § *residual
contradictions* (`:253-275`, items 1–10) is a **live open list** — item 6 is F3's site, and items
3, 4, 7, 8 are unverified by this sweep.

---

## 2. One-directional pointers (correction / ruling → target with no back-pointer)

| # | the ruling / correction | target | back-pointer? |
|---|---|---|---|
| P1 | `refinement.md:3` CAUTIOUSLY RATIFIED + member #0 shipped | `north-star.md:121-124`, `ARCHITECTURE.md:617-624`, `ROADMAP.md:1041-1042` | **NONE** — see F1. Zero citations of `refinement.md` outside the close block + `spines.md:377` |
| P2 | `corrections.md` #73 → its two stamped targets | `2026-07-24-palette-quant-generation-diagnosis.md:3-24`; `2026-07-29-member0-coarsefield-design.md:59-68` | **HALF.** Both banners carry the *answer* and the *mechanism narrative*; **neither names `#73`.** Pathspec `grep -rn "#73" --include=*.md .` → `CLAUDE.md:91`, `ROADMAP.md:3576`, `:3604`, `journal/0127:110`, and this file. The palette-quant banner routes *sideways* (*"See `docs/audits/2026-07-29-member0-coarsefield-design.md` § 3 + header rulings"*), not to the entry that holds the mechanism |
| P3 | the 2026-07-24 close block that held the resolution, now `ROADMAP-history.md:6009` (*"the palette-quant 460-vs-28.8 was settled — member stepping"*) | `2026-07-24-palette-quant-generation-diagnosis.md` § 7 | **NONE.** This is the **archived end** of exactly the edge `CLAUDE.md` read-first item 5's new close-block clause was written for, and it is unrepaired |
| P4 | `flow.md:804-816` § 11.5 RATIFIED the pairing rule | `flow.md:133-145` (the flag) · `S19-results.md:346-353` · `flux.rs:93-108` · `refinement.md:299-302` · `dependency-graph.md:51` | **NONE in any direction** — 5 sites, 0 edges. *(disposition reserved)* |
| P5 | `north-star.md:437-439` Deviation 2 voids three **named** sibling sections | `north-star.md:230-231`, `:250`, `:258-262` | **NONE** — F8. Still live at `72fbe86` |
| P6 | `refinement.md:71-74` answers geology's open question | `geology.md:367-372` | **NONE** — F9 |
| P7 | `pass-declaration-history.md:14-15` announces `cadence` → `schedule` | `spines.md:551` | **NONE** — F5 |

**Bidirectionality verdict on corrections #73, which the brief asked for specifically:**
the **correction → target** direction is **stamped and substantive** (both targets carry the
answer, its evidence, and the one-directional-pointer diagnosis). The **target → correction**
direction is **not**: no target names `#73`, so a reader who enters at the stale end gets the
answer but cannot walk to the mechanism. And **P3 is fully unrepaired** — the close block that
held the resolution still points nowhere, which is the half CLAUDE.md's new clause names.

---

## 3. `P6` engine/pack placement — asked in the brief, and it now reads ONE way

Verified at `72fbe86`. Pathspec `grep -rn "P6" --include=*.md .`:

- `dependency-graph.md:74` — the row is **struck** (`~~P6~~`) and marked *"**MOVED TO ENGINE
  2026-07-29 → E5 member #0** — this row and § 3's 'P6 is engine' said opposite things four rows
  apart"*, with the reason stated (*per-voxel draw = engine primitive*; *"a pack SELECTS a source
  by id, never supplies one"*).
- `dependency-graph.md:127` — *"P5 is content, **P6 is engine**."* — consistent.
- `2026-07-29-member0-coarsefield-design.md:91` — *"**P6 is engine work.**"* — consistent.
- `:216-217` — records the fix.
- `ROADMAP.md:3609` and `journal/0127:110` — record it as a same-day repair.

**No contradicting site found.** The one residue is cosmetic: the struck P6 row still physically
sits inside § 2 *"DEFAULT PLUGIN PACK — the passes"*. It is unambiguous as written; flagged only
so nobody re-derives the placement from the table it lives in.

*(Unrelated `P6` at `2026-07-22-deeptime-vector-audit.md:19` is a different numbering space —
"distribution-first expression" — and is not a collision worth acting on.)*

---

## 4. Proposed corrections — and the argument that only one is owed

**Per the SKILL's deliverable rule: a contradiction is not automatically a correction.**
F1–F10 are supersession-banner and strike work, not falsified claims: in every case the stale
half *was true when written* and no reasoning was wrong. `ARCHITECTURE.md:624`'s *"the corpus
may hold several contradictory ratified designs for one problem"* is the nearest thing to a
falsified claim, and it is **already recorded** (`dependency-graph.md:118-119`; the lineage
paragraph of #73). **No new `corrections.md` entry is proposed.**

**One PROCESS amendment is proposed, and it is the only thing here that will prevent a
recurrence rather than repair one:**

> **`CLAUDE.md` read-first item 5's close-block clause (added 2026-07-29, corrections #73)
> lives in no ritual.** `CLAUDE.md:88-96` obliges *"a close block recording a RESOLUTION
> ('settled — X') … owes the asking document its banner in the same commit."*
> `.claude/skills/wrap/SKILL.md` **§ 7** (`:69-84`) carries the stamping rule but conditions it
> on *a correction* refuting *a dated-measurement artifact*; **§ 9 Docs closure** (`:95-117`),
> which is where the close block is actually written, says nothing about it.
>
> By the corpus's own measured law (`doc-topology/SKILL.md` result 3: *a convention survives only
> if it is inseparable from something the author must do anyway; it dies if it asks them to
> restate in a second notation*), the obligation is currently a second notation nobody is
> prompted for — the shape that gave `JUSTIFIED-BY` **3 uses**. **F1 and P3 are both instances
> of it, both created or left standing after the clause was written.**
>
> **Recommendation:** one line in `wrap` § 9, beside *"previous block marked superseded"* —
> *did this close block record a resolution to a question some doc is still asking? Stamp that
> doc now.* Labelled a recommendation; the integrator/user owns whether it lands.

---

## 5. Ambiguous — flagged rather than forced

1. **F3's disposition** — reserved to the main-session fluvial design pass by the brief. This
   sweep adds two sites and one stale premise and rules on none of them.
2. **F9** — whether `refinement.md` § 4 or § 5 Law 1 is the intended answer to `geology.md`'s
   open question, and whether a user-owned *"sharpened slice by slice"* question may be closed
   in a ratified design doc at all. **User call.**
3. **`ARCHITECTURE.md:600` vs `dependency-graph.md:47` (E1).** ARCHITECTURE lists engine
   ownership as *"the pass-graph kernel (**validate an authored order**)"*; E1 says the kernel
   *"today **derives** order from `{reads, writes}` (Kahn + tie-break); validating an *authored*
   order is **E7**."* I read ARCHITECTURE's line as a statement of **DECIDED end-state
   ownership**, not of current capability — so not a contradiction. But it is the exact phrasing
   that produced the E1 overclaim the close block records as fixed same-day
   (`ROADMAP.md:3608-3609`), so whether it wants a *"not yet — E7"* marker is a judgement I am
   not making.
4. **The `🔴 REFINEMENT PRIMITIVES` entry's overall status** (F1). The arc is genuinely open —
   there is no refinement *runner*, and `refinement.md:275-278` says so honestly (*"the
   base-reconstruction step ran with **no runner**"*). Whether the entry is re-headed or only has
   `:1041-1042` struck is an integrator/user call.
5. **`material-genesis-notebook.md:19`'s *"Every one of those four now points here"*** — checked
   and **it holds** (`flow.md:230`, `material-behavior.md:646`, `north-star.md:58`,
   `ROADMAP.md:1738`). Recorded because an earlier pass of this sweep read it as broken on a
   truncated grep; a `head`-limited grep is not a pathspec, and the retraction belongs in the
   record.
