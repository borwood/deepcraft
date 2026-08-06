# doc-topology sweep — 2026-08-05 (FULL)

- **Base commit:** `08afb88` (worktree `agent-a17218640bc7db97c`; `git merge main` → *Already up to
  date*, tip == `main`). **Every quotation below was read at `08afb88`** unless a different commit is
  named at the citation (corrections #67: a quotation without a revision is not a quotation).
- **Mode:** FULL — 25 commits since watermark `b4d7514`, over the threshold. The window is the
  2026-08-05 GEO-5 *characterization* session: no code, one new skill (`passes`, 499 lines), three new
  audits, `ROADMAP` +419, `refinement.md` +30, `stubs.md` +66, `corrections.md` +55 (#101),
  `journal/0159`.
- **Write-set:** this file + `docs/audits/.sweep-watermarks.json`, same commit. **Nothing is fixed
  here.** Zero corrections filed — every disposal below is a banner, a strike or a pointer repair. If
  the integrator disagrees, the next free ordinal *appeared* to be **#102** at `08afb88` and **must be
  re-verified at merge** (a parallel session shares the checkout).

---

## 1. Coverage — what was read, and what was not

**Read IN FULL:** `docs/design/north-star.md` (459 lines) · `.claude/skills/passes/SKILL.md` (499) ·
`.claude/skills/roster/SKILL.md` (189) · `docs/design/earth-processes.md` §§ 1–8 + the method block ·
`docs/design/tectonics.md` §§ 8.1–8.3 + the top banner + § 14 + the outline · `docs/audits/
2026-08-05-debt-walkthrough-notes.md` §§ 3.4–3.7, 4 · `ROADMAP.md` § *In flight* head, § Sequenced's
*THE HONEST RECORD* head, and **the whole current GEO-5 close block, checked against itself** ·
`corrections.md` #101 in full · the `refinement.md` delta in full.

**Targeted at source (code, to adjudicate doc pairs):** `crates/dc-worldgen/src/deeptime/field.rs:595-615`
· `flux.rs:330-353` · `inventory.rs:388-412`, `:434`, `:450-480`, `:556-559` · `docs/ARCHITECTURE.md:540-600`,
`:721` · `docs/design/flow.md:730-760` · `docs/design/material-behavior.md:228-246` · `docs/design/bodies.md:260-268`
· `docs/design/stubs.md:1978-2005` · `docs/dependency-graph.md:190-191`.

**NOT OPENED — a null from these is not a result:** `docs/spines.md` (only via citations; +19 in delta,
and it is now over the filesize threshold) · `stubs.md` beyond #52 and greps (+66) · `water.md`,
`ores.md`, `ecology.md`, `geology.md`, `materials.md`, `posture-gait.md`, `visuals.md`,
`octree-substrate.md` (greps only) · `bodies.md` beyond `:229-268` · `ROADMAP` § Observed (~1,750 lines)
and most of § Sequenced · `journal/0158`, `journal/0159` · `ROADMAP-history.md` · `docs/spikes/` · the
three 2026-08-05 audits beyond the sections cited.

**Hole closed this run:** `north-star.md` — **read-first item 0, skipped by the two previous FULL runs**,
which called it *"the corpus's longest unexamined read-first surface"*. It was not clean; **F2 and F9 come
out of it.** **Hole still open:** the BODIES half of shape 1 (`posture-gait.md`, `bodies.md`) is
essentially unrun for a third consecutive sweep.

---

## 2. Findings — ranked by blast radius

### F1 · 🔴 The corpus's answer on structural attitude is a 60-line design in `tectonics.md` § 8, and the three artifacts arguing the question all miss it

The 2026-08-05 walkthrough filed this as *"⚠ THE REAL FORK — and the corpus asserts BOTH
architectures"* and flagged `tectonics.md` as unread. **It is not a symmetric fork.** Three sides:

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A — "the record folds"** | `docs/design/earth-processes.md:247-248` | *"Sim: **deformation operators on the strata record between epochs**; erosion surfaces recorded as events (an unconformity is a first-class entry)."* |
| **B — "re-derived, never stored"** | `crates/dc-worldgen/src/deeptime/field.rs:606-612` | *"Per-unit deformation (dip, provenance, fault traces) is **intended** to re-derive analytically from it at collapse resolution — the ~5 KB that would replace **stored per-cell dip vectors** … today the table is exported and read by nothing."* |
| **C — the design nobody in this thread read** | `docs/design/tectonics.md:665-694` (§ 8.1–8.2) | *"**units never store deformation** — so the 665 k → 71 k overprint optimization survives **by construction**, not by care."* · *"**the collapse tier computes a unit's attitude directly**: at column `x`, a unit deposited in chapter `u` … its dip is `g(Σ_{c>u} ∇F_c(x))` … **The layer cake dies without a single stored dip vector**."* |

**Why A is weaker than it looks.** `earth-processes.md:31` heads the section containing § 7:
*"## The engines (**working notes — sketches, NOT decisions**)"*. Side A is a sketch line. Side C is a
dispatched design pass, and **its own top banner explicitly preserves it** through the `flow.md`
supersession: *"**What survives untouched:** chapters and the chapter table, crustal columns, isostasy
and flexure, punctuation, **§ 8 events** …"* (`tectonics.md:26-29`).

**And C is not merely (B).** It is (B) **plus** the half of (A) that A actually wants: a sparse
world-level `StructEvent { kind, chapter, support, params }` list with `kind ∈ { FaultSlip, Intrusion,
**Truncation**, Punctuation }` (`tectonics.md:655-661`) — *"an unconformity is a first-class entry"*
satisfied without deformation operators mutating the stack. § 8.2 also names the exception A is right
about: *"Fault offsets are the discrete exception (a throw is not a gradient)."*

**Three live artifacts state the question as open without citing § 8:**
- `.claude/skills/passes/SKILL.md:430-432` — *"**Live fork, unopened:** stored per-cell dip vector vs
  re-derived analytically from tectonic history — the code comments say re-derived is the intent"* —
  sourced to **code comments**, not to the design doc that argues it.
- `.claude/skills/passes/SKILL.md:479-480` (§ 7.5 *Known unknowns*) — *"**Where structural attitude
  should live** — stored vs re-derived."*
- `docs/audits/2026-08-05-debt-walkthrough-notes.md:337-338` — *"`docs/design/tectonics.md` **UNREAD**
  by the integrator; it may settle (A)/(B). **Read before ruling.**"* — **honest, and the reason this
  finding is cheap.** `ROADMAP.md:5130-5131` already sequences the read as first-thing #2.

**Judgment: C looks authoritative on the architecture; the *ratification* is a user call.** § 8 is a
design-pass output, not a user ruling, and it sits under an unratified-at-the-numbers doc; the user's
2026-07-20 DECIDED direction (`earth-processes.md:47-60`, *"dip/fold as **recorded** deformation"*) is
compatible with either — *"recorded"* there means *consequence-of-the-record*, not *stored-per-cell*,
which is exactly the ambiguity § 8.2 resolves. **Blast radius:** the biggest open landform gap (U9, the
layer cake) and the user's parting *"nothing's getting folded on the record level"*.
**Proposed disposal (my reading, unratified):** the walkthrough's § 4.4 gains side C by citation; the
`passes` skill's § 7.3.1 and § 7.5 stop calling it *unopened*. **No doc edit here.**

---

### F2 · 🔴 `north-star.md` § *The two clocks* asserts the trust tiering as current, unstruck — and § Deviations 2's enumeration of where to strike it is short by that section

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A — still asserted** | `docs/design/north-star.md:300-303` | *"**Runtime:** sacred. Only sparse / event-driven passes; **first-party native**. — **Untrusted third-party content lives at the gen tier** … **Trusted / signed content can earn the native runtime path.**"* |
| **B — the ratified void** | `docs/design/north-star.md:445-453` (§ Deviations **2**, user, emphatic) | *"**ASSUME MODS CAN DO ANYTHING** … So the **capability-tiering** in § 'The core / plugin boundary', § Passes ('field passes … trusted … first-party'), and § Refinement ('a global solver is a trusted-tier capability') is **EXPLICITLY NOT THE MODEL** and must not shape any design."* |
| **B′ — the same void, in the file every session loads** | `CLAUDE.md` read-first item 0 | *"there is **no difference in permission between native and WASM**, so never justify a core/content placement by trust."* |

**A states a permission difference between native and WASM as present fact.** B′ forbids exactly that
sentence. **And B's own enumeration names three sections and does not name § *The two clocks*** — which
therefore carries **no strike and no banner**, while §§ Passes (`:239-240`), Refinement (`:272-275`) and
the boundary section all received one from the 2026-07-29 sweep (F8). *An incomplete enumeration inside
the ratified statement that voids the thing — the same defect shape as F3, one doc over.*

Adjacent and **less bad, so do not conflate them**: § *The mod SDK boundary + the tiered backend*
(`:334-340`, *"trusted / signed + first-party → NATIVE … untrusted third-party → WASM sandbox"*) and
`:327` (*"native backend, **since it is trusted**"*) are also unstruck, but that section opens with a
status note (`:316-319`) calling the split *"deferred product infrastructure, not a live architectural
fork"*. § *The two clocks* has nothing.

**User-originated on both sides** — the tiering was user-sketched and the void is user-emphatic; the
void is later and explicit, so this is not a tie. **Blast radius: maximal.** Read-first item 0, § *The
two clocks* is the section quoted whenever the runtime-is-sacred doctrine is cited, and CLAUDE.md tells
every session the opposite. **Proposed disposal:** strike `:301-303` and the *"first-party native"* at
`:300` with a pointer to § Deviations 2, and widen § Deviations 2's enumeration to name § *The two
clocks*. **Not adjudicated here.**

---

### F3 · 🟠 `ARCHITECTURE.md`'s ratified indictment names only `DeepAxis`; the completion lives only in a close block, a graph row and a skill

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A — the ratified statement** | `docs/ARCHITECTURE.md:558-565` (§ *The engine is plugin-agnostic*, **DECIDED 2026-07-26, user**) | *"### What this indicts today — `deeptime::runner::DeepAxis` is a **closed enum inside `dc-worldgen` whose variants are the names of our own pipeline stages** … **That is the violation**."* Nothing else is named; the section carries no banner. |
| **B — verified 2026-08-05** | `.claude/skills/passes/SKILL.md:326-344` (D-10) | *"**`DeepAxis` IS NOT THE ONLY CLOSED ENGINE VOCABULARY** … **`FlowCause`** (`flux.rs:337-352`) … **Seven variants + none = 8 values = exactly the 3 bits allocated. It is FULL** … **And there is a third:** `Cause` (`inventory.rs:395-407`) … **`Dissolution` appears in BOTH enums**, owned by neither. **So E6 is scoped too narrowly.**"* |

**Verified at source at `08afb88`:** `flux.rs:337-352` declares `FlowCause { Fluvial=0 … Dissolution=6 }`
— seven variants, and its own doc comment reads *"Regimes differ in four numbers and a field — viscosity,
density, competence, resistance, and the field they follow"* (`flux.rs:330-331`), i.e. a term definition.
`inventory.rs:395-407` declares `Cause { Chemical, Biotic, Frost, Dissolution }`. **D-10's claim holds.**

**This is the one-directional-pointer shape, and the corpus already knows:** the completion is written at
`docs/dependency-graph.md:190` (E6 row, *"`ARCHITECTURE.md`'s ratified indictment (DECIDED 2026-07-26)
names only `DeepAxis` — **the enumeration is incomplete and completing it is owed**"*) and at
`ROADMAP.md:5180-5181` — **which is a close block, and close blocks get archived.** CLAUDE.md read-first
item 5's own clause covers this case by name (*"a close block recording a RESOLUTION … owes the asking
document its banner in the same commit"*). The asking document is unstamped.

**Judgment: not a contradiction — an incomplete enumeration inside a ratified principle, plus a missing
back-pointer.** Both sides are true; the missing sentence is at A. **Blast radius:** read-first item 3;
this is the paragraph anyone quotes to justify plugin-agnosticism, and it currently under-reports the
defect by two enums, one of which is **persisted in every `DepUnit` and in the merge key**.
**Proposed disposal:** a banner at `ARCHITECTURE.md:558` naming `FlowCause`/`Cause` and pointing at the
E6 row. **Doc amendment, no code** — as `ROADMAP.md:5181` already says.

---

### F4 · 🟠 The `passes` skill tells every future session that a range containing four dated user rulings is "explicitly not decisions"

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A** | `.claude/skills/passes/SKILL.md:486-488` (§ 8 Pointers) | *"`earth-processes.md` **:9-29 (method items 1–5 — the *method*, not the engine sketches at `:31-258`, which are explicitly not decisions)**"* |
| **B** | `docs/design/earth-processes.md`, inside `:31-258` | `:47` *"**Direction DECIDED 2026-07-20 (user, session-4 gen review)** — design pass owed before any code; spike-class."* · `:152` *"**RATIFIED 2026-07-20: wind is agent #5**, and the dormant axes go live."* · `:165` *"**Follow-up slice RATIFIED 2026-07-20 (user)** — zonal circulation profile."* · `:190` *"**Erosion-agent roster FLIPPED ON — DECIDED 2026-07-21 (user**; journal/0034)."* |

The section heading at `:31` really does say *"working notes — sketches, NOT decisions"* — **so both
halves are quoting real text, and the doc contradicts itself**: four dated user rulings were appended
*inside* a section whose heading disclaims them, and the skill compressed the heading into a
line-range verdict that swallows all four.

**The cost is already visible in this same session's output.** `docs/audits/2026-08-05-debt-walkthrough-notes.md:275`
opens item 4 with *"### 4.1 📌 **THE DIRECTION IS ALREADY DECIDED** (user, 2026-07-20, `earth-processes.md:44-60`)"*
— **leaning on precisely the block the skill's pointer tells a reader to discount.** One artifact treats
it as binding; the other, written the same day, routes readers past it.

**User-originated: side B** (four user rulings). **Judgment: A is the wrong side**, but the underlying
defect is in `earth-processes.md`'s own structure, not in the skill's honesty. **Blast radius:** a JIT-loaded
skill, and § 9 of that same skill makes `earth-processes.md` method compliance a thing every pass owes.
**Proposed disposal:** narrow A to *"the engine SKETCHES at `:31-258` are not decisions; the dated
`DECIDED`/`RATIFIED` blocks inside that range are"*, and add the same caveat at `earth-processes.md:31`.

---

### F5 · 🟠 The user's zero-relabelling ruling never reached the defect inventory it settles

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A — the ruling** | `docs/audits/2026-08-05-debt-walkthrough-notes.md:211-215` (**USER RULING, 2026-08-05**) | *"**'any share of recorded metres getting relabelled is too much.'** Settles the argmax question **without a measurement**. The identity-loss fraction is demoted from a *decision* input to a **sizing** input — still worth taking … **but not to decide whether**."* |
| **B — the inventory** | `.claude/skills/passes/SKILL.md:323-324` (D-9) | *"**UNMEASURED and owed before any remedy:** what fraction of recorded metres loses its identity. Sum non-winning mass at each deposit against total recorded metres."* |

**Confirmed by history, not inferred:** the skill's last edit is `539d6cd`; the ruling landed at `38e4a13`,
**later the same day**, and no commit after it touches `.claude/skills/passes/SKILL.md`
(`git log -- .claude/skills/passes/SKILL.md` → `539d6cd, 99569ac, 45ac87e, c1505ac, e962656`, all
2026-08-05). The ruling is recorded in the audit (A) and in the close block (`ROADMAP.md:5142`) — **an
audit and a close block, both of which get archived** — and nowhere in the file `ROADMAP.md:417` points
at as *"Full inventory with provenance and confidence markers."*

**User-originated: A.** **Judgment: B is stale by half a day** — *"owed before any remedy"* is a gate the
user removed. Not a falsified claim (the measurement is still worth taking), so **not a correction**; a
banner. **Blast radius:** D-9 is the walkthrough's headline item and the record-schema pass reads it
first. **Proposed disposal:** one line at D-9 — *"⚠ demoted to a SIZING input by the 2026-08-05 user
ruling; it no longer gates a remedy."*

---

### F6 · 🟡 The `passes` skill governs tectonics passes, never sweeps or cites `tectonics.md`, and mis-cites ROADMAP

Three defects in one file, all in the load-bearing § 0 / § 8 scaffolding:

1. **`tectonics.md` (1,011 lines) is absent from both the sweep list and the pointers.** The skill's
   frontmatter description (`:3`) claims scope over *"weathering, transport, deposition, creep, drainage,
   **tectonics**, genesis"*; § 0's grep list (`:22-27`) names
   `{earth-processes,material-behavior,refinement,flow,worldgen,materials,knowledge,pass-declaration-history}.md`
   and § 8's pointers (`:485-499`) name no design doc beyond those. **Search performed:** `grep -n
   "tectonics.md" .claude/skills/passes/SKILL.md` → **0 hits at `08afb88`**. This is the mechanical cause
   of F1: a skill cannot route a reader to a doc it does not list.
2. **`ROADMAP.md:396-399` is the wrong citation, and was wrong when written.** `.claude/skills/passes/SKILL.md:222`
   reads *"`ROADMAP.md:396-399` names this skill as the inventory's single home — deliberately, checked
   2026-08-05."* At `08afb88`, `ROADMAP.md:396-399` says *"**⚠ THE ARC'S WORKING DOC IS
   `docs/audits/2026-08-05-debt-walkthrough-notes.md`** … **Its § 1 table is the durable home of the
   unwalked items**"* — a **different doc named as a durable home**. The line that does name the skill is
   `ROADMAP.md:417`. **Checked at the skill's own last-edit commit** (`git show 539d6cd:ROADMAP.md`):
   the naming line was at **`:411`** there, and `:393-402` already held the weathering-correction block.
   So the citation was ~15 lines off at authoring and is ~21 off now — **not drift, an unverified
   citation**, in the file whose own § 0 banner warns about exactly this class.
3. **D-6's headline is broader than the stub it files under.** `SKILL.md:290-292`: *"**D-6 — the
   alteration clock is coarser than the deposition clock. FILED as `stubs.md` #52.**"* `stubs.md:1978-2000`
   (#52) is about **one store**: a `DepUnit`'s epoch is immutable while `set_tag_and_chapter` rewrites
   chapter in place. Its blast-radius clause (`:1998-2000`) covers *"any future reader that assumes
   `chapter(unit)` is derivable from `epoch(unit)`"* — **not** the fact ledger. Low confidence that this
   is a defect rather than a compression; flagged so the record-schema pass does not read #52 as covering
   more than it does.

**Judgment: all three are pointer hygiene, not architecture.** **Blast radius:** the skill is read-first
for the next geo session (`ROADMAP.md:5094`).

---

### F7 · 🟡 Third store, third resolution — acknowledged pairwise, enumerated nowhere

**Not a contradiction. A missing sentence, reported because the brief asked whether the docs assert
consistency.** Verified at source at `08afb88`:

| store | time key | citation |
|---|---|---|
| deposited **unit** | `epoch` **and** `chapter` (chapter rewritable in place) | `.claude/skills/passes/SKILL.md:127` (*"material, thickness, **epoch, chapter**, mover"*); `stubs.md:1980-1984` |
| weathering **fact** | `chapter` only, though the pass fires every epoch | `inventory.rs:460` (`Fact::InPlace { … chapter: u8 }`), `:556-559` (*"chapter-ordered list … the only thing a chapter-commit writes"*) vs `:434` (*"appended-and-merged **every epoch**"*) |
| flux **face** | `chapter` | `docs/audits/2026-08-05-debt-walkthrough-notes.md:226` (*"`chapter` — **the tectonic chapter — NOT the epoch**"*, verified there at `flux.rs:372-421`) |

**No doc asserts they are consistent, and no doc enumerates all three.** *Searches performed at `08afb88`:*
`grep -rniE "same (clock|time resolution|granularity)|one clock|shared clock|consistent (clock|resolution)"
--include=*.md docs/ ROADMAP.md CLAUDE.md .claude/skills/ journal/corrections.md` → 8 hits, **none of them
a consistency claim about the three stores**; and `grep -rniE "three (stores|resolutions|clocks)"
--include=*.md docs/ ROADMAP.md .claude/skills/` → **0 hits**.

**⚠ One near-miss that must not be mis-paired:** `docs/ARCHITECTURE.md:721` — *"integrated `dt` over a run
equals the world's elapsed time. **Same clock for everyone**"* — is the SCHEDULE/`dt` integration
invariant, a claim about *how much time a pass advances*, **not** about a store's recorded resolution.
Pairing it with the table above would be a false finding.

What the corpus *does* hold, in three separate places: `stubs.md` #52 (unit epoch vs unit chapter) ·
`SKILL.md` D-6 (alteration vs deposition) · `flow.md:661-662` + `:730-760` (per-chapter vs per-epoch flow
facts, and the **WINDOW axis RATIFIED as a declared axis, *"Sequenced, not built"***). **So the ratified
answer already exists and is unbuilt: every store's window should be declared.** The gap is that no
artifact says *"here are the three, and they disagree"* — which is what makes the record-schema pass
likely to fix one and ship two.

---

### F8 · 🟡 CARRIED, still live: the `/roster` skill forbids as a hand table what a user ruling made an authored table

The 2026-08-04 run's **F7**, **re-verified at source at `08afb88` and unapplied**:

| side | citation (@`08afb88`) | statement |
|---|---|---|
| **A** | `.claude/skills/roster/SKILL.md:109-114` (§ 2 step 2) | *"**Derive, never hand-pair.** Any relationship this member has to others (continuum membership, **weathering products**, settle ordering) must be computed from its terms … **A hand list of related materials is a fitted taxonomy and gets rejected at review.**"* |
| **B** | `docs/design/material-behavior.md:230-232` (**U7 RULED 2026-08-02, user**) | *"**RULED 2026-08-02 (user, FS-A / P10's U7): RELEASE SPECTRA ARE AUTHORED EDGE PRODUCTS — 'R2 it is, authored edges.'** An edge … may declare a **product table**"*; and `:244-245` *"The **authored-table** shape is also what keeps **bimodal release expressible**."* |

**USER-OWNED if contested.** The roster skill's own § 0 watermark still reads *"last swept/updated
2026-08-04"* (`:33-35`), so this survived a sweep by never being re-swept. **Second consecutive run.**

### F9 · 🟡 CARRIED, still live: `bodies.md` restates, 30 lines below its own correction, the figures it retires

The 2026-08-04 run's **F8**, re-verified: `docs/design/bodies.md:265` still reads *"biped **6.977 → 7**,
longleg **7.742 → 8**, stout **4.286 → 4**"* against its own banner at `:229-236` (*"the three-decimal
figures were carried into the banner from a conversation; journal/0148's own two-decimal table … is
**the one to quote**"*). One inline parenthetical closes it. **Second consecutive run.**

### F10 · 🟢 `north-star.md`'s "what exists today" table still sells the superseded ordering model as convergence

`docs/design/north-star.md:378` — *"| pass-runner, declared order | `pipeline.rs`: passes declare
reads/writes, **topo-sorted, conflicts rejected** |"* — presented as the in-tree embryo of the
destination, while `:226-230` of the same file strikes exactly that: *"~~The runner topo-sorts by
declared reads/writes and rejects conflicts.~~ **SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER
WORLD**."* The table row is the **shape-6 "summary that outran its source"** at 150 lines' distance,
inside the doc that already carries two banners about its own opening line doing the same thing. Low
cost (a table of embryos, not a directive); listed for completeness.

---

## 3. Stale pointers, not contradictions

- **`ARCHITECTURE.md:558-565` has no back-pointer** to `dependency-graph.md:190` (E6) or to `SKILL.md`
  D-10 — F3's other half. *The stale end holds no link, and the stale end is where a cold session enters.*
- **`.claude/skills/passes/SKILL.md:222` → `ROADMAP.md:396-399`** — wrong at authoring and wronger now
  (F6.2). Correct target: `ROADMAP.md:417`.
- **`.claude/skills/roster/SKILL.md:33-35`** watermark reads 2026-08-04; the 2026-08-05 window shipped the
  refinement honesty ruling and D-9's identity discard, neither swept against it. Not asserted to bear on
  it — **not checked**, and the skill's own § 0 says it must be.
- **`docs/design/tectonics.md` § 8 carries no forward pointer** to the 2026-08-05 walkthrough or to
  `SKILL.md` § 7, which are the artifacts now arguing its question. Reciprocity runs both ways (F1).

## 4. What I could not resolve

- **Whether `tectonics.md` § 8 was ever ratified** as opposed to designed. The doc's status block records
  U8 (the production flip) as user-DECIDED (`:56-65`) but I found no ratification of § 8's data model.
  **Search:** `grep -n "DECIDED\|RATIFIED" docs/design/tectonics.md` in the § 8 range (`:636-718`) → 0 hits.
  **This is what makes F1 a user call rather than a doc repair.**
- **Whether the fact ledger's chapter-only key is intentional or incidental.** `inventory.rs` states the
  chapter-commit shape as the ratified S-2 model (`:1327-1328`) but I found nothing weighing chapter
  against epoch for facts. Same class as D-3's *"CANNOT DETERMINE whether top-of-stack placement is
  ratified or merely unexamined."*
- **Shape 3 (a user design reconciled away) — reported as a NULL with its search, and the null is weak.**
  `ideas.md` was untouched in the delta (`git diff --stat b4d7514..HEAD -- '*.md'`: not listed) and
  `grep -n "dip\|fold\|attitude\|tilt\|deform" docs/design/ideas.md` → 1 hit, unrelated (`:728`, body
  plans). But **the window's user rulings live in an audit and a close block**, not in `ideas.md`, so the
  primary locus for this shape has moved and I did not re-derive where to. F5 and F8 are the two
  instances I did find, both by other routes.
- **Shape 7 (a premise hiding inside a caveat): no candidate found.** Given the run read `north-star.md`
  and the two skills in full and found none, treat this as a soft null — the shape is invisible to a
  pair-based read by construction.

## 5. Nulls worth stating

- **`refinement.md` received the 2026-08-05 honesty ruling** (`git diff b4d7514..HEAD -- docs/design/refinement.md`:
  a 30-line banner at `:30-59`, with the move-C condemnation, the smooth-is-the-null-hypothesis reframe,
  the scope flag, and a pointer to `SKILL.md` §§ 1 and 7). **The design doc was stamped, not only the
  close block.** Clean.
- **`corrections.md` #101 is internally consistent** with `SKILL.md` § 0's banner and with the walkthrough's
  § 4.2 retraction; read in full, no pair found.
- **The GEO-5 close block, checked against itself** (`ROADMAP.md:5090-5193`): no self-contradiction found.
  It records the tectonics read as first-thing #2, marks the passes skill's unswept sections in § *Owed*,
  and its gate accounting names the three post-dating commits. The one thing it does that this sweep
  flags is structural, not internal — **it is where F3 and F5 are recorded, and a close block gets archived.**
