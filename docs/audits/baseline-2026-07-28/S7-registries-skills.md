# S7 — registries & skills (baseline doc-topology sweep, 2026-07-28)

> ## 📋 DISPOSITION — applied 2026-07-29.
>
> | # | state |
> |---|---|
> | **F1** `corrections.md` #40 stands unstruck against #67 | **✅ APPLIED** — the #56-shaped reciprocal banner added to #40. **Addition only; no body edit.** |
> | **F2** the dispatch rule pins a model name | **✅ APPLIED** — the Fable binding replaced with *"whatever the main session is running"*, citing CLAUDE.md's identical fix to the commit trailer. The user-ratified rule and the budget constraint are untouched. *(This slice's own falsifier: the sweep ran on Opus 5.)* |
> | **F3** doc-topology's "three confirmed live" instances, two repaired | **✅ ALREADY FIXED before this pass** — the skill carries a *"⚠ COUNTS IN THIS FILE WENT STALE ON 2026-07-28"* block correcting exactly this, and deliberately leaves the original standing. No action taken. |
> | **F4** `session-workflow` files the staleness sweep as the open gap | **✅ ALREADY FIXED before this pass** — that bullet now reads *"~~Make the staleness sweep RECURRING~~ (superseded above)"*, and the same ⚠ block in doc-topology corrects the "four interventions" / "two sweeps" counts. No action taken. |
> | **F5** `spine-audit` gives one anti-shape number to two shapes | **✅ APPLIED** — the first label corrected from `A-3` to **`S-3` (a spine, violated)**, with the consequence named: this skill files into `spines.md`, so the mislabel routed S-3 violations into the A-3 heading. *The highest value-per-line fix in the slice, as the finding said.* |
> | **F6** CLAUDE.md § Gates vs the staged gate | **⚠ REFERRED — out of scope.** The fix belongs in `CLAUDE.md`, which this agent may not edit. **This one is worth raising**: an agent running § Gates verbatim gets the failure mode `session-workflow` was written about — a killed run with **no verdict**, which is neither a red nor a green. |
> | **F7** `corrections #36` asserts "there is no geotherm" as current state | **✅ APPLIED** — reciprocal banner naming the retirement of `stubs.md` § 14, `geotherm.rs`, `COAL_ONSET_C = 22.0`, and #51's gradients. **The measurements stand untouched**; what is marked is the standing current-state claim. |
> | **F8** neither filing checklist carries the banner obligation | **✅ APPLIED to both** — `wrap/SKILL.md` § 7 and `doc-topology/SKILL.md`'s filing step now carry *stamp the target, same commit, immutable body / mutable header*, each stating **why it rides on the act rather than sitting on a later list** (the `JUSTIFIED-BY` failure mode). |
> | **F9** #23's mechanism half falsified by #24, no back-pointer | **✅ APPLIED** — pointer added; the entry's conclusion and its honest self-flagging are preserved and credited. |
> | **M1** "67 entries" vs 68 numbered bodies | **✅ APPLIED** — noted inline at the measurement banner; the `/67` denominator left as measured. |
> | **M2** *"Exit codes lie about GPU crashes"* as a lead phrase | **✅ NO ACTION** — verified the retraction is already inline on the same bullet and both bodies agree. Negligible radius, as the finding says. |
> | **M3** `tour-map`'s conditional "gate the probe **if**" | **✅ APPLIED** — modality raised to match CLAUDE.md § Gates' imperative, with the reason (`cargo test` builds examples and never runs them). |
> | **M4** reciprocity **successes** | **✅ NO ACTION — and worth preserving.** A null on this axis would be suspicious; the finding records four working reciprocal pairs. |

**Watermark commit:** `f652b60`. **Every `file:line` below was read at `f652b60`** in the worktree
`.claude/worktrees/agent-a8385ea45d85f32af`, except where a citation is explicitly marked
*(main checkout, read 2026-07-28)* — those name files that do not exist at `f652b60` and the
distinction is load-bearing for finding **F4** (corrections #67: a quotation without a revision is
not a quotation).

**Read in full (2,114 lines):** `journal/corrections.md` (2,681 — read entire, in three passes) ·
`.claude/skills/session-workflow/SKILL.md` (967) · `doc-topology/SKILL.md` (180) · `wrap/SKILL.md`
(138) · `spine-audit/SKILL.md` (90) · `tour-map/SKILL.md` (58). Cross-checked against `CLAUDE.md`
(worktree copy, 440 lines at `f652b60` — **not** the main-checkout copy, which is older and
modified). Spot-verified against `docs/spines.md` (§ headings only) and `docs/design/stubs.md`
(§ 14 only) — named where used.

**Not opened:** `ROADMAP.md`, `north-star.md`, `material-behavior.md`, the journal entries, the
other audit docs. Findings below that touch them are stated only where a citation inside my slice
makes the claim checkable without them.

---

## 1. Findings — contradiction pairs, ranked by blast radius

### F1 · `corrections.md` #40 stands unstruck against #67, which falsifies it — inside the one file whose purpose is recording falsified claims 🔴

| side | citation | statement |
|---|---|---|
| A | `journal/corrections.md:1327` (entry #40, `:1312-1344`) | *"A comment that already states 'consumed by nothing' is not a justification outliving its premise; **the A-2 was never live**."* |
| B | `journal/corrections.md:2630-2653` (entry #67) | *"**Falsified: the audit did not paraphrase. It quoted the comment exactly as it stood the day before.** … So the A-2 **was live** … and the commit that ended it calls the prior text *lying*, which is the same verdict the audit reached."* |

**Better supported: B, decisively.** #67 is git-verified (`git show 11d4385 -- .../field.rs`, both
revisions quoted, the commit message quoted) and was produced by an adversarial re-coding
dispatched specifically to disagree. #40 is an inference from a single reading of the *current*
text.

**Why this is the top finding rather than a curiosity.** #40 carries **no `#67` token anywhere in
its body**, so a reader who enters at #40 — which is exactly where a cold reader enters, since #40
is the entry a grep for `exhum`/`t_crust`/`A-2` reaches — gets the retracted verdict with nothing
to warn them, *and* gets #40's standing lesson (*"re-read the comment before believing the
sweep"*) whose missing half is #67's whole point. This is the identical defect the project
repaired **nine days later on the same page**: `corrections.md:1984-1995` is the reciprocal strike
added to #56 on 2026-07-28 because *"#60 named this entry and this entry never named #60 … for two
days `corrections.md` — the file whose whole purpose is recording falsified claims — carried an
unstruck falsified claim 220 lines from its own withdrawal."* **#40 is 1,300 lines from its own
falsification and has not had that repair applied.**

Rule-5 note: this is **not** a request to rewrite testimony. #56's repair is the ratified shape —
an *addition*, body untouched. #40's falsified half is a **verdict about history** plus a
**policy** (its standing lesson), both of which #67 explicitly amends; `stubs.md` § 4 and
`spines.md` A-2 were already amended per #67's own consequences list (`:2667-2672`) and **only the
correction itself was left**.

*Recommendation (labelled as one, not applied):* apply the #56-shaped reciprocal banner to #40.
No body edit.

---

### F2 · the dispatch rule pins a model name; `CLAUDE.md` recorded 2026-07-26 that pinning a model name is a lie the moment the roster changes 🔴

| side | citation | statement |
|---|---|---|
| A | `.claude/skills/session-workflow/SKILL.md:86-88` | *"**Fable is the MAIN SESSION's model only; passing no model override inherits Fable** and burns the usage budget."* (also `:87` *"a stock deep-research workflow inherits the main session's Fable into every fan-out agent"*) |
| B | `CLAUDE.md:435-440` | *"Commits end with `Co-Authored-By: Claude <model>`, **naming the model that actually did the work** … *This line used to hardcode `Fable 5` and went stale the first time a session ran on another model* … **The trailer is provenance: pinning one name makes it a lie the moment the roster changes.**"* |

**Better supported: B**, and it is not close — B is a dated correction *of exactly this defect* in
a sibling file, and A is falsified by the session reading it: **this sweep runs on Opus 5, so
"passing no model override inherits Fable" is false as written.** The load-bearing half of A (the
user-ratified *"every agent gets `model: opus` by default"*, `:80-84`, and the usage-budget
constraint at `:89-95`) is **untouched** — what is stale is the mechanism it is justified by. An
agent that reasons "no override ⇒ Fable ⇒ budget burn" on a session whose main model is not Fable
is reasoning from a false premise about its own environment.

**Blast radius: every dispatch in every session** — this is the rule an integrator consults before
launching agents, and B's own lesson is that a pinned model name is the thing that goes stale
first. Provenance: the *rule* is user-ratified (2026-07-19, data); the *Fable binding* is a
fact-of-the-day carried inside it, not a user constraint.

---

### F3 · `doc-topology`'s own opening asserts three "confirmed live" one-directional pointers; two of them were repaired before this watermark 🔴

| side | citation | statement |
|---|---|---|
| A | `.claude/skills/doc-topology/SKILL.md:105-113` | *"Three confirmed live instances, all in read-first surfaces: … **`corrections #12` declares itself 'the pointer' for `S10-results.md`'s ~2×-wrong cost table and `S10` never names it** · and **`corrections.md` #56 asserted, unstruck, exactly what #60 withdraws, with no `#60` token anywhere in the file.**"* |
| B1 | `journal/corrections.md:1984-1995` | #56 **is** struck — a 🔴 WITHDRAWN block naming #60 explicitly, dated *"Struck here 2026-07-28"*. The file now holds the `#60` token at `:1984` and `:1993`. |
| B2 | `journal/corrections.md:300-302` + `CLAUDE.md:54-70` | *"`S10-results.md` **now carries one**"* (a top-of-file banner), under the DECIDED 2026-07-28 **IMMUTABLE BODY, MUTABLE HEADER** rule. |

**Better supported: B1/B2** — both are directly checkable in my slice and both are dated the same
day as A. Only the `north-star.md` instance (first in A's list, outside my slice) is unverified
here; I did not open it and do not claim it.

**Why the blast radius is high despite being "just" a stale example.** This block is the
*calibration* an agent reads before running this sweep. A sweeper who inherits *"three confirmed
live instances"* will either re-report two repaired defects as findings — which is the exact error
`corrections #67` exists to name, a **stale read masquerading as a finding** — or will conclude the
reciprocity repair is not working. It is working; it ran on both of them.

*Note the shape:* this is not staleness-by-decay. A was written on 2026-07-28 and the repairs
landed on 2026-07-28. It is the corpus's characteristic failure — **two halves of one day's work,
in two files, neither pointing at the other** — inside the skill written to catch it.

---

### F4 · `session-workflow` still files the recurring staleness sweep as the open gap; it shipped as a skill 🟠

| side | citation | statement |
|---|---|---|
| A | `.claude/skills/session-workflow/SKILL.md:947-951` | *"**Make the staleness sweep RECURRING, like `spine-audit`.** … Only a mechanism that *forces* a re-read finds those. **Today's sweep had to be requested — that is the gap.** Run it after any batch of merges that ships a new arc…"* |
| B | `.claude/skills/staleness-sweep/SKILL.md:8-11` *(main checkout, read 2026-07-28; **absent at `f652b60`** — `.claude/skills/` in this worktree holds only doc-topology, session-workflow, spine-audit, tour-map, wrap)* | *"Existed as a proven procedure and two worked examples since 2026-07-24; **packaged as a skill 2026-07-28** so it has the same standing as its two siblings."* Its frontmatter: *"Run FIRST THING in a session, incrementally from the recorded watermark. Delegate to a background agent."* |

**Better supported: B** — it is the artifact, and its own header states the shipping date. **The
revision caveat is not a hedge, it is the finding's shape:** at my watermark A is *true*, and it
became false during the same day this sweep runs. I report it because the brief asks for
already-shipped-things-presented-as-proposals and because A sits **three lines below** the
project's own worked example of this defect (`:934-939`, the `ROADMAP-history` bullet, annotated
*"read as a future proposal until 2026-07-28 — four days after it landed — because this skill is
organised as **dated strata** that nothing ever revisits"*). **The bullet immediately after the one
that was caught is the same defect, and the annotation did not reach it.**

**Two dependent claims go stale with it**, both inside my slice:
- `.claude/skills/doc-topology/SKILL.md:36-39` — *"What is already shipped, so you do not propose
  it again"* lists **four** interventions; the staleness-sweep skill is a fifth and is not there.
- `.claude/skills/doc-topology/SKILL.md:45-49` — *"**The two existing sweeps** both compare the
  corpus to something outside it"* / *"nothing checks the docs against each other"*, and
  `spine-audit/SKILL.md:8-11`'s two-half loop. There are now **three** sweeps and the new skill
  states the triad as a table (`staleness-sweep/SKILL.md:13-23`, main checkout).

---

### F5 · `spine-audit` gives one anti-shape number to two different shapes, and the first assignment names a SPINE 🟠

| side | citation | statement |
|---|---|---|
| A | `.claude/skills/spine-audit/SKILL.md:43-45` | *"Does it hit an anti-shape — a stand-in hardening into a definition (A-1), **a summary standing beside an authority rather than deriving from it (A-3)**, **a check that cannot fail for the reason it claims (A-3)**?"* |
| B1 | `docs/spines.md:180` | **`## S-3. A summary is derived from the authority, never beside it`** — a **spine**, not an anti-shape. |
| B2 | `docs/spines.md:910` | **`## A-3. A test green for a reason unrelated to what it asserts`** — which is A's *second* item. |
| B3 | `journal/corrections.md:1485` and `.claude/skills/session-workflow/SKILL.md:552` | both gloss A-3 as *"green for a reason unrelated to the claim"*, matching B2. |

**Better supported: B.** A's second label is right; **the first is wrong twice** — wrong number,
and wrong *class* (it calls S-3 an anti-shape). The consequence is concrete and one-directional:
`spine-audit` is the skill that files findings **into `spines.md`**, so an auditor following check
3 files an S-3 violation under the A-3 heading, where the genuine A-3 instances live. `spines.md`
A-3 already carries a geotherm-fixture instance (`:937-941`); a mis-filed summary-vs-authority row
lands beside it and both become harder to read.

*This is the highest-value finding per line of text in the slice* — it is a two-character defect in
a read-first-adjacent skill with a mechanical, checkable fix, and neither `spine-audit` nor
`spines.md` points at the other for it.

---

### F6 · `CLAUDE.md` § Gates specifies a gate that `session-workflow` says cannot be run 🟠

| side | citation | statement |
|---|---|---|
| A | `CLAUDE.md:134-140` | *"## Gates (all must pass before merge)"* followed by one three-command block (`fmt --all --check`, `clippy --workspace --all-targets --release`, `test --workspace --release`), with no staging note anywhere in the section's ~60 lines of caveats. |
| B | `.claude/skills/session-workflow/SKILL.md:758-776` | *"**The gate has outgrown one tool call — run it as STAGES** … The full gate now exceeds a single 10-minute invocation and was **killed mid-`test`** … Two causes, **both permanent** … **Run it as separate invocations**, each with its own verdict."* |

**Better supported: B** — it is dated, mechanism-bearing (`clean -p` across dc-worldgen + dc-api +
dc-client forces a Bevy rebuild; dc-worldgen's suite alone is past ~870 s), and states the causes
are permanent. A is not *wrong* about what must pass; it is **incomplete about how**, and it is the
half that every agent brief copies, because briefs quote CLAUDE.md § Gates.

**Blast radius:** an agent that runs A verbatim gets the failure mode B was written about — a
killed run with **no verdict at all**, which B warns *"must never be recorded as [a red gate], but
it is equally not a green."* That is a false-signal class this corpus has three corrections about
(#21, #27, #37).

*Not resolved here:* whether CLAUDE.md should carry the staging or point at the skill is a
structure call, and `ROADMAP-history.md:2447` mentions a `./scripts/gate.ps1` I did not open.

---

### F7 · `corrections #36` asserts, as current state, that the project has no geotherm — it shipped, and #36's own heir is marked retired 🟡

| side | citation | statement |
|---|---|---|
| A | `journal/corrections.md:1192-1199` (entry #36, 2026-07-22) | *"There is **no geotherm** in the project — the only temperature anywhere in the sim is surface air temperature — so this is burial depth, not a P/T path, and it cannot express coal **rank**. … `COAL_BURIAL_M = 8.0` … is listed in `docs/design/stubs.md` § 14 **with a geotherm as its heir**."* |
| B1 | `docs/design/stubs.md:367-376` | *"### 14. coal-rank-is-burial-depth-with-no-geotherm — **RETIRED 2026-07-24 (journal/0093, the geotherm field pass)**. *Retired:* the geotherm (the first § 5 field pass, `deeptime/geotherm.rs`) supplies a real [temperature]…"* |
| B2 | `journal/corrections.md:1710-1714` (entry #51) | measures it: *"coal cells mean gradient **41.9 °C/km** vs peat-only **31.3** … rift/arc ≥ 40 °C/km → **13.4 % coal**"*, and #51's subject is *"the geotherm's coal recalibration (`COAL_ONSET_C 8 → 22 °C`)"*. |

**Better supported: B.** This is flagged under rule 5 as a **current-state claim**, not as
testimony: #36's *measurements* (12,892 → 2,216 coal-bearing cells, the 13-under-50 m
distribution) are a dated record and stand untouched. What is false today is the standing
assertion *"there is no geotherm in the project"* and the forward-looking *"with a geotherm as its
heir"* — the heir landed two days later, `stubs.md` recorded it, and `corrections.md` did not.
Same one-directional shape as F1, one tier down: **`stubs.md` § 14 names its retirement; #36 does
not name `stubs.md` § 14's retirement.**

Blast radius is moderate rather than high because #51 (which any coal/geotherm grep also reaches)
makes the geotherm's existence obvious. But #36 is the entry a reader lands on for *coal rank*,
and it currently says rank is inexpressible for a reason that no longer holds.

---

### F8 · the two checklists an author actually consults when filing a correction do not carry the banner half of the 2026-07-28 decision 🟡

| side | citation | statement |
|---|---|---|
| A1 | `.claude/skills/wrap/SKILL.md:66-68` | *"## 7. Corrections for every falsified claim — Including — especially — the assistant's own, and any of the project's own shipped numbers that turned out to be phase, luck, or a stale premise."* (no banner obligation) |
| A2 | `.claude/skills/doc-topology/SKILL.md:174-177` | *"Then file the falsified half in `journal/corrections.md` **only where a claim is actually falsified**…"* (no banner obligation) |
| B | `CLAUDE.md:54-70` (DECIDED 2026-07-28, user) | *"a results doc **must carry, at its top, a pointer to anything that later refuted, superseded or re-scoped it**" … "**The obligation lands on the WRITER OF THE CORRECTION**, in the same commit, while both documents are already open. **This is the one property that makes it likely to stick**…"* |

**Better supported: B** — it is a user DECIDED entry, dated after both A's. This is an
**unpropagated decision** rather than a flat contradiction, and I rank it accordingly. But it is
worth reporting because B's *own stated survival condition* is that the obligation be discharged
"while both documents are already open" — i.e. **at the moment the checklist is consulted**, and
neither checklist mentions it. B even names why conventions die here (`JUSTIFIED-BY`, 3 uses).
`CLAUDE.md:68-70` states the backlog openly: *"the corpus was never swept for missing banners."*

---

### F9 · `corrections #23`'s mechanism half was falsified by #24, which names it; #23 carries no pointer back 🟡

| side | citation | statement |
|---|---|---|
| A | `journal/corrections.md:685-695` (#23) | *"**Mechanism — HYPOTHESIS, not yet measured.** `DeepField::surface_at_voxel` is a *bilinear* sample of a **460 m** deep grid … **Needs measurement before it is believed**."* |
| B | `journal/corrections.md:706-745` (#24) | *"**Falsified by measurement** (S13) … the level-5 lattice reproduces the raw deep grid's relief to within **0.4 %** … Nothing is smoothed away below 460 m because the source holds nothing below 460 m."* B names A (*"carried into corrections #23 as half the mechanism"*, `:708`). |

**Better supported: B.** Ranked low because **#23 flagged its own hypothesis honestly** — the
entry did the right thing and the falsification arrived — and because #23's *conclusion*
(`thickening_scale` is a lift-the-continent knob) is untouched and still live. It is listed for
completeness of the one-directional-edge census below, not as a defect of judgement.

---

### Minor / noted, not ranked

- **M1** `.claude/skills/doc-topology/SKILL.md:14,16` and `:20` speak of *"all **67** `corrections.md` entries"*. The file holds **68 numbered bodies**: `#63b` at `journal/corrections.md:2423` is a deliberate sub-number (*"recorded here rather than as a separate number, because it never entered the corpus"*). Not a defect; flagged because any independent recount lands on 68 and will read as a discrepancy against a measured percentage base.
- **M2** `.claude/skills/session-workflow/SKILL.md:278` keeps *"**Exit codes lie about GPU crashes**"* as the bullet's lead phrase with the retraction inline; `CLAUDE.md:238-243` states the warning is *retired* for dc-client. Both bodies agree; only the lead phrase reads as the live claim. Same shape as F4, negligible radius.
- **M3** `.claude/skills/tour-map/SKILL.md:51` — *"**Gate the probe if it carries assertions**"* is conditional where `CLAUDE.md:150-152` is imperative (*"**an example that can fail belongs in the gate**"*, with the mechanism `[[example]] test = true`). Same direction, weaker modality; no reader is misled.
- **M4** Reciprocity **successes** worth recording, since a null on this axis is suspicious: the push rule is stated in both `session-workflow:130-139` and `wrap:92-98` **with each naming the other**; `spine-audit:67-78` and `doc-topology:133-146` both carry the corrections-#67 commit-citation rule with matching text and both name #67; `tour-map` and `CLAUDE.md` § Agent walks agree on the coal-null precedent (#51) in both directions.

---

## 2. Skill bullets presenting an already-shipped thing as a future proposal

**One**, plus **two** of the mirror shape (a repaired defect still presented as live):

| # | citation | shape |
|---|---|---|
| 1 | `session-workflow/SKILL.md:947-951` | future proposal → **shipped** as `.claude/skills/staleness-sweep/` (F4) |
| 2 | `doc-topology/SKILL.md:109-111` (corrections #12 / `S10-results.md` edge) | live defect → **repaired** (F3) |
| 3 | `doc-topology/SKILL.md:111-113` (corrections #56 / #60 edge) | live defect → **repaired** (F3) |

Plus one already self-annotated by the corpus and therefore **not counted**:
`session-workflow/SKILL.md:934-939` (the `ROADMAP-history` archive), which carries its own
2026-07-28 correction and is the worked example the count above should be read against.

---

## 3. Corrections whose lesson I believe is no longer live

**Two firm, one partial.** Everything else that expired **carries its own annotation**, which is
the honest headline of this section — see the note under the table.

| entry | what is no longer live | evidence |
|---|---|---|
| **#40** (`:1312-1344`) | the verdict *"the A-2 was never live"* and the sufficiency of its standing lesson | `#67` `:2630-2662`, git-verified against `11d4385`; `stubs.md` § 4 and `spines.md` A-2 already amended per `#67:2667-2672` |
| **#36** (`:1192-1199`) | the current-state claim *"there is no geotherm in the project"* and *"with a geotherm as its heir"* | `stubs.md:367` (§ 14 **RETIRED 2026-07-24**, `deeptime/geotherm.rs`); `#51:1710-1714` measures gradients |
| **#23** (`:685-695`) *partial* | the 460 m-bilinear **mechanism** only; the entry's conclusion stands | `#24:706-745` |

**The annotated ones — the reason the count is this low, and the strongest evidence that the
repair habit works.** `#12:287-305` (policy half-superseded, banner rule, dated) · `#32:1079-1091`
(remedy retired in place, *"Do not reintroduce address comparison"*) · `#35:1153-1161` (interval
corrected, git-verified) · `#49:1603-1613` (`~~not applied~~ — **APPLIED**`) · `#54:1875-1886`
(struck at source, with the two sites deliberately *not* struck named and justified) ·
`#56:1984-1995` (🔴 WITHDRAWN, names #60) · `#63:2368-2380` (⚠ NARROWED, names the pathspec it
should have stated). Seven entries whose expiry is recorded *in the entry*. **Two are not** — and
one of those, #40, is the entry a correction was filed against.

---

## 4. Claim inventory — `journal/corrections.md`, all 68 numbered bodies

Format: `#N` (line) — what it falsified → **lesson status**.

1. (`:6`) `depth_bias` fixes coplanar z-fight → **live** (draw order ≠ depth written; the radial-push rationale survives as #11's premise).
2. (`:17`) spawn transform is set before rendering → **live**.
3. (`:27`) walk-3's floating shards / horizon seam → **live**; the founding "verify your own instrument" entry, cited by #10/#13/#19.
4. (`:49`) `skip_serializing_if` is safe on wire types → **live**, in CLAUDE.md § Conventions.
5. (`:55`) `surface_height_m` under-reports ~7 m ("missing octave") → **live** (analytic field ≠ its own voxelization).
6. (`:76`) chunk-line families are cell-stepped climate → **live** (it was the wrong *cell*; measure which quantization the artifact rides).
7. (`:108`) `env!(CARGO_MANIFEST_DIR)` locates the workspace → **live**; standing rule.
8. (`:123`) B is "minutes, paid once"; erosion is bounded → **live** (cost and boundedness are measurements, per-process).
9. (`:139`) a parallel priority-flood pulls B to ~2 min → **live**; reopen condition explicitly stated and unmet.
10. (`:160`) walk-12 "surface:true buried the player" → **live**; the units entry (metres vs voxels), cited by #28.
11. (`:199`) same-level far chunks shift near-identically → **live**; watertightness must be proven past the transform.
12. (`:253`) the ritual costs 25 s with biology → **live, policy half superseded in place** (`:287-305`): dated-record upheld, *"this entry is the pointer"* replaced by IMMUTABLE BODY / MUTABLE HEADER. **The worked example of the correct repair shape.**
13. (`:307`) a fabricated user quote on darkness → **live**; do not quote words the user did not say; attribute the register.
14. (`:351`) the body graph needs rich inlet/outlet structure → **live**; ask what the substrate already encodes.
15. (`:381`) a closed domain with a drain measures a water-table halo → **live**; boundary conditions, and audit the harness when it agrees with you.
16. (`:421`) temper abrasion resistance by cohesion → **live**; measure which *direction* a modifier moves the contrast.
17. (`:448`) coupling incision produces differential erosion → **live**; find the rate-limiting phase.
18. (`:483`) the lit diff is the sun moving; fullbright is honest → **live**; the instrument-choice entry, in CLAUDE.md.
19. (`:527`) integrator wrote an unverified mechanism into CLAUDE.md → **live**; *an agent's mechanism is a hypothesis*, in session-workflow `:222`.
20. (`:574`) rivers share the one-shot disease → **live**; brief agents to contradict their own scope.
21. (`:598`) the impossible red (two wrong mechanisms) → **live**; mechanism confirmed, recorded in session-workflow `:145-164`.
22. (`:626`) "deserts land where a player expects them" → **live**; the gameplay-impact trace working as designed.
23. (`:654`) the model is fine, the amplitude is the bottleneck → **conclusion live; mechanism half falsified by #24, no back-pointer** (F9).
24. (`:706`) the 460 m bilinear sample low-passes detail → **live**; names the real cause (`AMP_DECAY^L_DEEP`) and pre-rejects a proposal by number.
25. (`:746`) "the world is flat" from the summit → **live**; sample the distribution, not the extremum.
26. (`:798`) the deflation basin holds `H ≈ 0` → **live**; an extremum of a *rate* is not an extremum of a *state*.
27. (`:826`) a green gate means your code passed → **live**; the impossible GREEN, in CLAUDE.md § Gates and session-workflow `:290`.
28. (`:865`) the shipped world ≠ the probed world → **live**; a bare number pair is not an address (cited by #67 as the same defect one layer down).
29. (`:949`) partial-height will light up for free → **live**; prose cannot fail a build.
30. (`:975`) the flooded shaft is the coarse-capacity worst case → **live**; ask which error term a case exercises.
31. (`:1004`) connectivity only changes inside the loaded set → **live**; and it names the future condition (absent chunk ⇒ UNKNOWN) precisely.
32. (`:1040`) comparing `fn` addresses fails only harmlessly → **live, remedy retired in place** (`:1079-1091`): the falsification stands, the docstring rule was replaced by `Option<fn>`.
33. (`:1093`) `chunk_budget_for` scales with horizon → **live**; do not ask an agent to report a quantity that cannot vary.
34. (`:1113`) RAM march is flat at 0.00 MB/jump → **live**; a window shorter than the period cannot tell flat from oscillating.
35. (`:1132`) a charcoal band cannot exist, so charcoal is dead content → **live, interval corrected in place** (`:1153-1161`, eleven days → one day, git-verified). A decision *not to build* has a shelf life and no watcher.
36. (`:1174`) `promote_coal` promotes on the wrong axis → **lesson live; the current-state claim *"there is no geotherm"* is FALSE** (F7).
37. (`:1201`) `clean -p` + a gate proves the gate saw your code → **live**; the cross-worktree serve, in CLAUDE.md.
38. (`:1245`) the Small world has no deep-time record → **live**; a control that never moves is evidence only about the axes exercised.
39. (`:1272`) the coherent bilinear source biases toward 50/50 → **live**; it sharpens, not flattens. Carries an unresolved `NEEDS RATIFICATION` on the cost's sign.
40. (`:1312`) the seam audit misquoted the comment, so the A-2 was never live → **FALSIFIED by #67; no back-pointer** (F1).
41. (`:1346`) raising the erosion budget raises relief → **live**; a differential probe says nothing about the absolute response.
42. (`:1390`) render-first is a cleanly-separable wedge → **live**; brief-as-hypothesis caught it at work time.
43. (`:1413`) synchronous chunk gen is the vertical-drop killer → **live**; the instrument overturned the suspect its own slice was filed under.
44. (`:1432`) `fully_resolved` is live in the renderer → **live**; verify a claim about live code against the code.
45. (`:1451`) the member dither is chunk-anchored → **live**; read the noise function before prescribing its replacement.
46. (`:1470`) S18 expresses a saprolite band in the world → **live**; accept by OUTCOME at production scale (session-workflow `:541`).
47. (`:1493`) S18 is the first real weathering behavior → **live**; place the seam where the full thing will live; a one-shot of a continuous process is a category error.
48. (`:1518`) the palette-quant station is ~110 km east → **live**; a prose landmark is not a pose (CLAUDE.md § Agent walks).
49. (`:1548`) the contents record reads empty over solid ground → **live, fix applied in place** (`:1603-1613`); a chunk-level summary worn as a voxel-level authority.
50. (`:1615`) the front over-expresses by +3.7 % → **live**; a voxel-tier conservation audit cannot work by counting materials; 21 columns is not a population.
51. (`:1680`) the coal recalibration leaves a diggable seam → **live**; a helper named `production_field` must BE production. The tour-map null precedent.
52. (`:1732`) reusing fill bits would correlate into a visible pattern → **live**; a justification naming a consequence is testable; chase it even when the named harm is nil.
53. (`:1782`) nine test sites fail under f32 fact storage → **live**; a verdict belongs in the same cell as its condition; over-prediction is still a false number.
54. (`:1825`) MFD needs the head field → **live, struck at source** (`:1875-1886`) with the two append-only sites deliberately not struck and named, and a ROADMAP residual flagged to the integrator. **The best-executed propagation in the file.**
55. (`:1888`) the fluvial pass moves this world's sediment → **live**; measure how much authority a mechanism has before believing it matters.
56. (`:1936`) erosion rates are calibrated to the Phanerozoic → **live, cross-check paragraph struck in place** (`:1984-1995`, names #60). The closed-system / measure-against-the-literature entry.
57. (`:2061`) the one lithology a deposit cannot be is basement → **live**; a rule can be wrong and unreachable at once, and unreachable is a property of current magnitudes.
58. (`:2121`) `p → ∞` is D8 *exactly* → **live**; steepest slope ≠ steepest drop; a claim about shape smuggled in as one about identity.
59. (`:2167`) the competence ceiling is anchored, not a knob → **live**; a derivation is only an anchor if its source cannot move.
60. (`:2215`) two instruments agree to 2.4 %, so it is cross-checked → **live**; a cross-check is evidence only if it survives the regime it licenses.
61. (`:2257`) the calibration preserves the landscape's shape → **live**; an aggregate criterion cannot license a claim about local structure.
62. (`:2306`) 148/530 pits measures the clamp defect → **live**; a "more extreme than all neighbours" predicate saturates as the defect generalises.
63. (`:2352`) the solve is past its stability limit; `myr_per_epoch` is the discriminator → **live, narrowed in place** (`:2368-2380`): the *knob* absence stands, *"does not exist anywhere in the tree"* is false of the string. **An absence needs its pathspec.**
63b. (`:2423`) isostasy is a positive feedback → **live**; deliberately un-numbered because it never entered the corpus; the near-miss is the artefact.
64. (`:2445`) converting *two* engine draws re-rolls the history layer → **live**; a justification assembled by symmetry, never traced.
65. (`:2507`) phase ORDER falls out of the declared reads/writes → **live**; a derivation whose inputs were built to produce its output is not a derivation. Carries the CLAUDE.md rule and the `ARCHITECTURE.md` supersession pointer (`:2557`).
66. (`:2562`) the goldens will move and that is correct → **live**; a pre-authorised golden move cannot be distinguished from an unexplained one.
67. (`:2630`) #40's verdict → **live, and it is this slice's mandate**: a quotation needs a revision; **a correction can be wrong and nothing was checking.**

**One-directional-edge census within `corrections.md`** (the shape `doc-topology:103-113`
prescribes checking): entries that **name another entry which does not name them back** — #24→#23
(F9), #67→#40 (F1). Entries whose target **was** repaired reciprocally: #60→#56 ✔, #63's narrowing
✔, #35's interval ✔, #12→CLAUDE.md item 5 ✔. **Score: 4 reciprocal, 2 one-directional**, against
the corpus-wide 8-of-15 quoted at `doc-topology:111`. The habit is working; F1 is the expensive
miss.

---

## 5. Claim inventory — the skills

Every rule asserted, by `file:line`. Marked ⚠ where a finding above touches it.

### `session-workflow/SKILL.md` (967)
`:12` four hats · `:26` never fold your own proposals until ratified · `:29` ratified-unscheduled →
`ideas.md` · `:33` ratified → dated DECIDED in the owning design doc · `:35` user-owned choices:
options + recommendation, do not preempt · `:38` resolve the ROADMAP Observed line in the SAME
commit · `:45` sweep the corpus BEFORE opening a design pass, priors section first · `:58` one
milestone = one worktree agent · `:61` parallelize by write-set, serialize the machine · `:71`
resource rules mandatory in every brief · `:76` what a good brief contains · **⚠ `:80-99`** model
economy; opus by default; state fan-out size before launching (**F2**) · `:100` a quiet "waiting"
agent may have lost its wake-up · **`:107`** pass `isolation: "worktree"` explicitly · **`:116`**
when you dispatch an adversarial check, name the files it may not read; ask for its numbers first;
ask for the ambiguous population · `:130` push to `origin/main`, standing authorisation ·
`:141` merge `--no-ff` then gate on merged main; grep case-sensitively · `:143` an impossible red
is a stale artifact until proven otherwise (+ 4 sub-disciplines) · `:165` remove worktree and
branch · `:167` fold findings into the docs you own · `:170` architecture and appearance are
different ratification axes · `:179` defer = write it now · `:185` replacements are briefed as
replacements · `:193` triage the NEEDS RATIFICATION list, never relay it whole · `:203` keep
`things-that-will-happen.md` fed · `:210` every gen slice gets a gameplay-impact + fidelity trace ·
`:222` an agent's mechanism is a hypothesis; higher bar for CLAUDE.md and this skill · `:237`
journal entries are narrative · `:239` corrections.md gets every falsified claim · `:243` ROADMAP
rides the journal commit · `:247` walks catch what metrics miss · `:251` the user's read is data
senior to yours · `:253` never smooth over a red gate · `:255` rhythm · `:263` the live guided-tour
ratification · `:273` the build-mutex lockfile · **⚠ `:278`** exit codes lie about GPU crashes
(retired for dc-client; **M2**) · `:289` verify a gate by test NAME · `:299` resume a parked agent,
do not re-brief · `:306` verify the load-bearing claim, not the whole report · `:313` a worktree is
frozen at its branch commit; commit design docs before dispatch · `:324` write the brief's premise
as a hypothesis · `:330` seam-first (9 numbered practices, `:357-399`) · `:401` shape compliance at
plan/work/review time; carve-outs pass through the user; update in the same commit · `:439`
measurement agents write to their worktree early; an empty worktree is not evidence of nothing ·
`:462` clean the crates a SIBLING built · `:490` hold the build lock around the cargo invocation ·
`:505` design passes get journal entries · `:519` sequence the slice AND what it is a slice OF ·
`:541` accept by OUTCOME; place the seam where the full thing lives · `:581` the dispatch-brief
template + 3 hard RETURN checks (`:594-610`) · `:612` "I cannot attribute this" is a first-class
answer · `:621` journal numbers assigned at dispatch · `:628` the integrator gate protocol (5
steps) · `:637` an additive-looking conflict still needs a brace check · `:664` don't
over-calibrate a placeholder · `:670` quote the MEASURE, never the threshold; quote the absolute
beside every ratio · `:692` mark what is assistant-originated · `:716` commit WIP early; agents
die; read the lock before nudging · `:742` assign STUB numbers at dispatch · **⚠ `:758`** run the
gate as STAGES (**F6**) · `:778` Claude drives the whole walk loop · `:797` the world output is a
scratch sheet · `:813` byte-identity: regression detector, not target · `:845` an audit's
prescription is a hypothesis too · `:874` an empty `git` result from inside a worktree is not
evidence of absence · `:890` two brief-template clauses (re-derive an audit's fix; reserve every
numbered artifact) · `:907` the ROADMAP outgrew reading (+ `:909` the docs-ops stop-block, **⚠
`:934` shipped-and-annotated**, **⚠ `:947` shipped-and-NOT-annotated — F4**) · `:953` `clean -p`
must name the whole changed dependency chain.

### `doc-topology/SKILL.md` (180)
`:8-43` the docs-ops stop-block: 4 measured results, the 4 shipped interventions (**⚠ `:36`,
F4**), the one open proposal (§ 5 reciprocity, assistant-originated, unratified) and
`stubs.md:22`'s bar on designing the general mechanism first · **⚠ `:45-49`** the two existing
sweeps (**F4**) · `:60` why grep cannot do this; the unit of work is a *pair* · `:66-113` six
shapes in value order, incl. `:95` the prescribed paraphrase-diff **cannot fire** and `:103`
check RECIPROCITY instead (**⚠ `:105-113`, F3**) · `:117` read whole sections · `:119` prioritise
by blast radius · `:121` file:line on BOTH sides · `:124` do not resolve what you find · `:127`
provenance decides weight · `:130` re-read both sides at source before publishing · `:133` record
the commit beside every quotation (#67); an absence needs its pathspec (#63) · `:147` a null is a
result but a null here is suspicious — say what you did not open · `:150-168` four scoping spines,
incl. `:160` always include the ROADMAP close block · `:170` deliverable = ranked pair table, then
file only actual falsifications · `:179` delegate; hold the corrections number at dispatch.

### `wrap/SKILL.md` (138)
`:12` rescue anything outside the repo · `:24` excluded ≠ sequenced · `:33` deviations,
carve-outs, conditional ratifications (A-7 never ratifiable) · `:45` in-flight agents; an empty
worktree is not evidence of nothing · `:52` unverified constants are gated now or named loudly ·
`:59` numbering collisions, and fix inbound references carefully · **⚠ `:66`** corrections for
every falsified claim (**F8**) · `:71` spines + defer = write it now · `:79` docs closure; previous
close block marked superseded · `:86` repo/machine hygiene, **and PUSHED** (`:92`) · `:100` the
gate actually passed, by name and count, cleaning the crates a sibling built · `:106` memory only
on material shifts · `:111` offer fingerprints and fold in the greenlit ones · `:130` the close
report: short, honest over tidy; **if a check finds nothing, say so**.

### `spine-audit/SKILL.md` (90)
`:8` `spines.md` is only worth having if it is true; this skill keeps it true, session-workflow §
Shape compliance keeps it applied · `:12` run after a batch of merges, a few times a day · `:13`
delegate, opus, read-only, writes exactly one file · `:17` why: the characteristic failure is
building a mechanism and losing it (A-4) · `:27` check 1, instances still resolve · `:32` check 2,
§ 3 accuracy (removal is the success condition) · **⚠ `:40`** check 3, changed files introduce an
unlisted instance or anti-shape (**F5**) · `:47` check 4, do cited justifications still hold (A-2)
· `:54` check 5, is the doc drifting into decoration · `:59` read-only, no "while I was in there"
· `:62` do not run cargo · `:64` cite `file:line` and distinguish code from doc · `:67` **and cite
the commit you read it at** (#67); an absence needs its pathspec (#63) · `:79` do not add a spine
on your own authority · `:83` report what you removed and why · `:85` the return spec.

### `tour-map/SKILL.md` (58)
`:8` a walk with nothing to look at costs the user's session · `:17` when: any owed appearance
walk, before launching · `:22` dispatch read-only on `src/` · `:25-40` six required outputs
(station + ready pose in radians · what makes it strongest, measured · a contrast station or an
honest "none" · the distribution · inter-station distance · **a null is a result, and prove the
instrument on a world where the signature exists**) · `:44` use the world the player actually
boots (#51) · `:46` pick the instrument that can see the question · `:49` do not tune anything ·
**⚠ `:51`** gate the probe if it carries assertions (**M3**) · `:55` hand off to the walk loop;
brief a null and cancel.

---

## 6. What I did not do

I did not edit `corrections.md`, any `SKILL.md`, or `CLAUDE.md`, and I resolved nothing. Every
recommendation above is labelled as one. No cargo, no builds, no source edits. Commits used
explicit pathspecs.
