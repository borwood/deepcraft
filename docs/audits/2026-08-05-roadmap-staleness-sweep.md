# ROADMAP staleness sweep — 2026-08-05 (FULL)

**Swept:** `ROADMAP.md` § Sequenced (L381–3337) and § Observed (L3338–5089) against everything
landed since watermark `2985273` — **156 commits**, `2985273..08afb88`, spanning the
2026-08-03 GEO-3/BODIES sessions through the 2026-08-05 GEO-5 close.

**Base:** `08afb88` (`git log -1 --oneline main` == this worktree's tip after `git merge main`:
*"wrap: record the closing gate GREEN (1043/0/95 at 961a505) and what post-dates it"*).

**Mode: FULL**, for two of the skill's three triggers: the watermark is 156 commits and ~2 active
days stale, and archive pass 3 restructured the board (`36e5e14`, § Observed 94 → 91).

**No edits were made outside this file and `docs/audits/.sweep-watermarks.json`.** The sweeper
reports; the integrator dispositions. Nothing here touches `ROADMAP.md`, `spines.md`, `CLAUDE.md`,
`journal/`, `stubs.md` or `corrections.md`.

**No cargo was run.** Every code claim below is a read or a `grep`, cited.

---

## 0. The recalibration check, run FIRST — and it voids nothing

The skill's caution: a recalibration only voids a cohort if it is **enabled in the config the
observations were made under**. Verified at source, not taken from the brief:

`crates/dc-worldgen/src/deeptime/field.rs::production_config_base` (`:276`–`:380`) builds the
shipped world's `DeepConfig` and **explicitly** sets:

| flag | line | value |
|---|---|---|
| `tectonic_history` | `field.rs:321` | **`true`** |
| `full_agents` | `field.rs:342` | `true` |
| `calibrated_rates` | `field.rs:378` | **`false`** |
| *everything else* | `field.rs:379` | `..DeepConfig::default()` |

and `DeepConfig::default()` sets `weather_inventory: false` (`crates/dc-worldgen/src/deeptime/grid.rs:607`).

**Consequences for this sweep:**

- **journal/0111's denudation calibration is still OFF in production**, so it voids no cohort here
  — the 2026-07-28 baseline finding stands unchanged.
- **`weather_inventory` is off**, so § Observed's `GOLDEN_LEDGER` entry (`ROADMAP.md:3883-3888`,
  *"`DeepField::ledgers` is empty in every shipped world"*) is **still true**. Not stale.
- **The delta's only real recalibration is a MEASUREMENT one**, not a flag: the record's total unit
  count **10,951,030 → 7,304,581** (S0 § g, `8022c26`). Its cohort is walked in **F11** below.
- ⚠ **`tectonic_history: true` is itself new information in this delta** (`961a505`, the GEO-5
  retraction; `ROADMAP.md:5155` records the `--tectonics` switch as **vestigial**). It falsifies a
  live Sequenced bullet's premise — **F9**.

---

## 1. Findings

Eleven. Each carries the entry's `file:line`, the evidence's `file:line` or command, and a
disposition class.

---

### F1 · SHIPPED — per-cycle quantization shipped 2026-08-04; its Sequenced entry was never stamped

- **Entry:** `ROADMAP.md:607-648`, *"PER-CYCLE QUANTIZATION — the 12 fps call, taken (bodies
  thread; DECIDED 2026-08-04, user)"*. Reads forward throughout: *"**FIRST SLICE.** Client-side
  only… derive `N` per cycle from the target; clamp at 2… delete the fixed constant."* No shipped
  stamp anywhere in the entry.
- **Evidence it shipped:** `9e4a068` *"merge per-cycle quantization: N = round(duration x fps),
  clamped >= 2"* (adds `crates/dc-client/src/anim_rate.rs`, 443 lines; rewrites `body.rs`);
  `0c9f861` *"quantization bookkeeping: gate 1029/0/94, journal 0155, stub #53, two stamps"*.
  The BODIES close block states it plainly at `ROADMAP.md:5394`: *"per-cycle quantization (0155,
  **walked and verdicted**)"*.
- **Why it was missed, precisely:** `0c9f861` **did** touch `ROADMAP.md` (24 lines) — but only to
  correct three cadence decimals (6.977 → 6.970 etc.), per its own commit body. The bookkeeping
  commit had the file open and stamped a number instead of a status.
- **Disposition: SHIPPED.** The entry's CONTINUATION SLOT (per-body cap as pack data, then B3) is
  the part that survives.

---

### F2 · CLOSED-BY-DECISION — the Observed 12 fps entry still says "NOT YET A RULING"

- **Entry:** `ROADMAP.md:3621-3636`, *"🟠 A FIXED 12 fps ALIASES A DERIVED CADENCE"*. At
  `ROADMAP.md:3630`: *"**The user's leaning is FORFEIT and it is NOT YET A RULING** — 'probably we
  just forfeit it… i just don't care enough'"*.
- **Evidence:** `8cfce92` *"DECIDED 2026-08-04: per-cycle, per-bone, client-owned quantization;
  slice sequenced"*; the user's ratified terms are transcribed at `ROADMAP.md:5403-5405`
  (*"there can't be fewer than 2 frames per cycle"*, target fps a **client** setting, *"a one-off
  is just a cycle that doesn't repeat"*). Then F1's slice shipped it.
- **This is the "Observed line never resolved in the same commit as the decision" miss** — the
  skill's item 2, and the most common one.
- **⚠ Sharp irony worth relaying, because it is diagnostic:** the *sibling* entry three bullets
  down (`ROADMAP.md:3697-3704`) **was** stamped resolved on 2026-08-04 by the very same slice, with
  the words *"Recorded because a side-effect resolution is the easy kind to leave rotting on the
  board: nobody who fixed it was looking at this entry."* The **direct** parent, twelve lines
  above it in the same section, was left rotting.
- **Disposition: CLOSED-BY-DECISION.** The diagnosis body (Nyquist, 6.97/7.73/4.29, A-1's fourth
  instance) is worth keeping; the *"not yet a ruling"* clause is false.

---

### F3 · SHIPPED — B7 joint rotation limits, slice one merged 2026-08-03

- **Entry:** `ROADMAP.md:716-719`, *"**B7 — joint rotation limits** (bodies thread; DECIDED
  2026-08-02, user). Design pass done… **its five stubs land WITH the build, deliberately not
  before.** Wants the pre-B3 segment-identity window."* Written entirely as pending.
- **Evidence:** `b8b1d14` *"merge B7 slice one: joint rotation limits (declared DOFs + L3
  derivation + validator/bake/IK enforcement)"*; `cb27555` *"B7 slice one: joint rotation limits,
  gates green"*; journal **0151** (`ROADMAP.md:5393`: *"B7 joint limits (0151)"*).
- **Why it was missed:** the bookkeeping commit `088b9dd` (*"B7 slice one bookkeeping: gate
  verified, ordinals assigned, graph reconciled"*) touched **`docs/dependency-graph.md` and
  `docs/design/stubs.md` only** — `git show --stat 088b9dd` lists no `ROADMAP.md`. The graph was
  reconciled and the board was not.
- **Disposition: SHIPPED (slice one).** Whether B7 has further slices is the integrator's call —
  the entry as written cannot tell a cold reader that any of it is built.
- **⚠ Provenance, stated so this is not read as a fresh find:** this entry **is** the 2026-08-03
  sweep's F1 remedy — `29cf4b5` *"apply staleness F1-F13: board stamped against what shipped, four
  arcs promoted"* created `ROADMAP.md:649` *"FOUR LIVE ARCS PROMOTED FROM CLOSE BLOCKS"*, of which
  B7 is one. **It went stale inside 24 hours of being written.** That is a fact about the
  stamping mechanism, not about this entry.

---

### F4 · SHIPPED — the stand-in marker control: survey done, mechanism built, **wired**

- **Entry:** `ROADMAP.md:796-820`, *"THE STAND-IN MARKER CONTROL — **owed 2026-08-03**… survey
  first, sweep second"*, ending *"**Step 1 is a SURVEY, not a sweep, and this is the load-bearing
  part.** Scope is genuinely unknown… **Answer that before designing the check.**"* Entirely
  unstamped.
- **Evidence — step 1 (survey) is done:** `26bf6e0` *"survey: the stand-in marker control is
  feasible on STAND-IN, a null on `heir`"*; deliverable
  `docs/audits/2026-08-04-stand-in-marker-control-scoping.md` (355 lines) — **213 `heir` sites
  classified individually across 261 `.rs` files, not sampled**; `TODO/FIXME/XXX/HACK`: **zero**.
  This is exactly the density number the entry says must exist first.
- **Evidence — step 2 (the mechanism) is built AND wired:** `scripts/standin_locus_check.py`
  exists at HEAD (`ls scripts/standin_locus_check.py` → present, 138 lines added by `ba7a1b0`);
  `.claude/skills/spine-audit/SKILL.md:116` runs `python scripts/standin_locus_check.py`. It is a
  **mechanism inside an act already performed**, which is the shape the entry demanded.
- **Evidence it ran:** `de1124c` *"close the marker loop: control wired, 6 of 7 live defects
  repaired"* — first run found 7; six repaired (`gait.rs:132/155/158/161/167`, `evaluate.rs:221`);
  `recorder.rs:832` handed up unrepaired (corroborated at `ROADMAP.md:5445`).
- **Disposition: SHIPPED.** The residue is one unresolved marker (`recorder.rs:832`), which the
  BODIES close block already carries.

---

### F5 · CLOSED-BY-DECISION — a `NEEDS RATIFICATION` flag whose ruling landed 2026-08-04

- **Entry:** `ROADMAP.md:710-711`, inside *"E4-2 + E4-3 — the implicit field kernel and its
  adoption"*: *"**Live ⚠ NEEDS RATIFICATION: the `dc-core` venue + its rayon rider** (audit U-3…)"*.
- **Evidence:** `22965a8` *"U-3 ruled: dc-core::field venue ratified (rayon rider included), U-5
  closed as executed"* (2026-08-04). Its `--stat` shows it touched
  `docs/audits/2026-08-03-e4-implicit-kernel-design.md` and `docs/dependency-graph.md` — **not
  `ROADMAP.md`.** The GEO-4 close block records the ruling at `ROADMAP.md:5258`: *"U-3 'The engine
  owns primitives'"*.
- **Disposition: CLOSED-BY-DECISION.** This is the skill's item 5 exactly: a `NEEDS RATIFICATION`
  marker sitting live after the ruling. Same failure mechanism as F3 — the graph was updated, the
  board was not.
- **⚠ THIS IS A RE-FILE, NOT A NEW FIND, AND THAT IS THE POINT.** The **doc-topology** sweep of
  2026-08-04 already caught it — `docs/audits/.sweep-watermarks.json`, `_doc_topology_last_run`
  → `findings`: *"ROADMAP:486-488 still calls the dc-core venue a live NEEDS RATIFICATION after
  U-3 was ruled emphatically"* (its F2/F3, filed **NOT APPLIED**). The line number differs (486 →
  711) only because the board grew between the two runs. **Two independent sweeps, two days
  running, on the same eleven words.** Like F3, this entry was created by the 2026-08-03 sweep's
  F1 remedy (`29cf4b5`) and was falsified the next day.

---

### F6 · CLOSED-BY-DECISION (one clause only) — "User picks pending in its § 9" — all five are ruled

- **Entry:** `ROADMAP.md:3494-3498`, inside the 2026-08-03 slice-3 interfingering field report:
  *"**User picks pending in its § 9: P-1 identity treatment… · P-2 the inverted acceptance
  criterion · P-3 R-C vs R-C′ · P-4 the onlap feather… · P-5 formal supersession ratification.**"*
- **Evidence all five closed, 2026-08-04:** P-1 `b8fe15e` (*"DECIDED: materials are labels over a
  term space; P-1 ruled M-C-derived"*) · P-2 `0ac5b7f` · P-3 `c306546`/`d9d7e79` (paused, then
  **resolved by reshaping** via the deposition clock) · P-4 `39096ae` · P-5 `0442e7c`. The
  Sequenced arc entry already says so at `ROADMAP.md:695`: **"ALL PICKS CLOSED."**
- **⚠ RULE 3 GUARD — the field report itself STAYS OPEN.** The user's verdict *"as predicted, i do
  not like this, visually"* (`ROADMAP.md:3440-3443`) is **data** and I found no evidence the world
  changed for it: the fix is S2, which has not landed (`ROADMAP.md:700`, *"**NEXT: S2**"*), and S2
  is additionally held by the lateral-quantization ruling (`ROADMAP.md:3405`, *"S2 must not wire
  the winner-take-all cut before this is settled"*). **Only the picks clause is stale.**
- **Disposition: CLOSED-BY-DECISION (the § 9 clause) / STILL-LIVE (the field report).**

---

### F7 · FIXED — the GEO-ARC spine-audit entry says "Nothing applied"; F1 and F4 were applied the same day

- **Entry:** `ROADMAP.md:3590-3616`, *"🟠 GEO-ARC FINDINGS FROM THE 2026-08-02 SPINE-AUDIT"*, whose
  header states **"Nothing applied; the arc adjudicates."**
- **Evidence for F1 (three stale `outcrop_shares` comments):** fixed 2026-08-03 by `0d2678c`
  *"fix five stale source comments the sweeps falsified (comment-only)"*. The diff replaces the
  exact cited prose: `erosion/mod.rs:339` now reads *"Since P11 slice 2 this is the **named
  basement constant**… the provider-seam walk… is retired"*; `erosion/transport.rs:351-356`'s
  *"Nothing is named here"* is gone; `erosion/weathering.rs:421` no longer says *"the same walk
  asked with an empty section."*
- **Evidence for F4** (`Litho::of_material` guard claim left verbatim on the test at
  `lithology.rs:1174-1176`): the same commit — *"lithology.rs's guard-test doc comment narrowed to
  the vanilla set it actually walks."*
- **Evidence F8/F10 (citation drift) were handled:** `274ff40` *"merge spine-audit FULL sweep
  2026-08-03 (spines.md refreshed: 1 row added, 19 confirmed, new A-2, citations)"*;
  `docs/spines.md:53` now records *"A-4's `split_by_shares` renamed to `species::split_row_into`"*.
- **STILL-LIVE within the same entry:** **F2** (A-7's enumeration short by three —
  `REFERENCE_MATERIAL`, `DEEP_BASEMENT`, `ANCHOR_MATERIAL`; **USER-OWNED**, a ratified ask against
  a read-first anti-shape). The GEO-5 owed list independently confirms it is open:
  `ROADMAP.md:5180`, *"`ARCHITECTURE.md`'s indictment enumeration is incomplete (E6 row)"*.
- **Disposition: FIXED (F1, F4, and the F8/F10 citations) / STILL-LIVE (F2).** The entry's
  blanket *"Nothing applied"* header is the stale part.

---

### F8 · CONTRADICTED (instrument, small but exactly the named class) — a published absence-proof no longer returns zero

- **Claim:** `ROADMAP.md:3595-3596` and `docs/spines.md:2362` both publish the same search as proof:
  ``grep -rn "outcrop_shares(&\[\])" --include=*.rs crates/`` → **"zero"**.
- **What it returns at `08afb88`:** **two hits** —
  `crates/dc-worldgen/src/deeptime/erosion/mod.rs:339` and
  `crates/dc-worldgen/src/deeptime/erosion/weathering.rs:421`. Both are **doc/line comments written
  by the fix in F7**, naming the retired call in order to say it is retired. **There are no live
  call sites**, so the substance is intact.
- **Why it is worth a line rather than a shrug:** corrections **#101** (2026-08-05) makes an
  absence-claim a claim about **its search**. A search that no longer reproduces its published
  result is the failure mode that rule exists to catch, and it was introduced *by the remediation
  of the finding that published it* — the fix and the proof were written a day apart by different
  hands. A pattern anchored to a call site (e.g. excluding comment lines, or matching
  `= outcrop_shares(&[])`) restores it.
- **Disposition: CONTRADICTED (the instrument, not the conclusion).**

---

### F9 · CONTRADICTED — the Tectonics SPIKE bullet's premise is falsified by this session's own retraction

- **Entry:** `ROADMAP.md:2927-2941`, *"**Tectonics SPIKE**"*, whose action clause at
  `ROADMAP.md:2935` reads: *"implement `DeepConfig::tectonic_history` **behind the flag** and
  produce the eight measurement groups…"*, closing *"**Dispatch after the eolian agent merges** —
  write-sets collide in `deeptime/erosion.rs`."*
- **Evidence the premise is dead:** `tectonic_history` is implemented **and ON in production** —
  `crates/dc-worldgen/src/deeptime/field.rs:321`, `tectonic_history: true` inside
  `production_config_base`. This was **established inside this delta** by `961a505` *"walkthrough
  notes: item 3 ruled… and a retraction — tectonic_history is ON in production"*, and the GEO-5
  close block records the consequence at `ROADMAP.md:5156`: **"the `--tectonics` switch is
  vestigial."** The stated dispatch blocker (the eolian agent) merged long before the watermark —
  `crates/dc-worldgen/src/deeptime/erosion/` is a split module tree now, not the single
  `erosion.rs` the collision note names.
- **What survives:** the **eight measurement groups** were never produced, and the GEO-5 session
  closed on the user's live observation ***"Tectonics still doesn't do what I expect it to do"***
  (`ROADMAP.md:5165`), explicitly undiagnosed. That is plausibly the exact ask this bullet was
  written for, and neither points at the other.
- **Disposition: CONTRADICTED (implementation half + blocker) / STILL-LIVE (the measurements).**
  Flagged for the integrator rather than adjudicated: whether the measurement groups are still the
  right instrument for the user's 08-05 observation is a design call, not a sweep call.

---

### F10 · THREE ARCS LIVING ONLY IN A CLOSE BLOCK — the exact shape staleness F1 spent a day repairing on 2026-08-03

All three live **only** in the 2026-08-04 BODIES close block (`ROADMAP.md:5386+`) — a live block
today, an archived handoff tomorrow. Each absence-claim names its search, per corrections #101.

| # | thread | its only home | my search, over `ROADMAP.md` |
|---|---|---|---|
| a | **The `body.rs` extraction** — 3,653 lines (**5.2×** the size threshold), three-module cut (sample / rig / probe) proposed by the hoist agent, **never taken**; *"a file move wanting its own commit and gate"* | `ROADMAP.md:5433` | `grep -n "3,653\|body.rs extraction\|three-module cut"` → **only `:5433`** |
| b | **The dev-surface gating concept** — owed a design pass; *"No gamemode/server-rule/world-setting/player-permission concept exists; its only trace is a comment at `authority.rs:360`"* | `ROADMAP.md:5435` | `grep -n "dev-surface gating\|gamemode"` → **only `:5435`** |
| c | **The build-slot hook did not hold** — two cargo processes coexisted for most of the ownership hoist's run, observed independently by the agent and the integrator, producing a **false RED** in that agent's gate. The block labels it verbatim: ***"Live defect, unfiled as work."*** | `ROADMAP.md:5443` | `grep -n "false RED\|false red"` → **only `:5443`** |

**(c) is not covered by the two hook entries already in § Observed** — I read both:
`ROADMAP.md:3420` is *"the hook silently **discards non-cargo work** bundled with a denied call"*
and `ROADMAP.md:3431` is *"the hook **denies non-cargo commands that merely mention cargo in
text**"*. Neither is *"the hook let two builds coexist"* — which is the **failure the mutex exists
to prevent**, and the one that manufactures false REDs.

**Disposition: STILL-LIVE, unfiled.** Note the sequenced entry *"THE BUILD SLOT MUST COVER PROBE
**RUNS**, NOT ONLY BUILDS"* (`ROADMAP.md:763`) is a **different** gap (a running probe binary is
neither `cargo` nor `rustc`) and does not subsume (c), though the two plausibly share a slice.

---

### F11 · The recalibration cohort — one artifact still quotes the superseded unit count, unbannered

- **The recalibration:** the record's total unit count settled at **7,304,581**, superseding
  **10,951,030** (S0 § g, `8022c26`; `16394fb` *"the count settled at 7,304,581, all targets
  stamped"*). Its § Observed entry (`ROADMAP.md:3373-3382`) is correctly stamped **✅ RESOLVED**
  and lists what was stamped: *"All artifacts stamped (0136, 0141, the correlation + clock audits,
  the fluvial priors)."*
- **The miss, found by walking the cohort rather than the entry:**
  `grep -n "10,951,030\|10.95\|7,304,581" ROADMAP.md ROADMAP-history.md` returns a hit **not on
  that list** — **`ROADMAP.md:930`**, inside *MEMBERS INTO DEEP HISTORY*:

  > *"**the merge-key split factor is 2.4053× — 10,951,030 units against 4,552,847 — i.e. +97.6
  > MiB of RESIDENT record (167.10 vs 69.47 MiB at 16 B/unit).**"*

  The corroborating residency figures at `ROADMAP.md:940-942` hang off the same measurement.
- **This is NOT a request to rewrite it.** It is a dated slice-2 measurement and read-first item 5
  freezes the body. What it lacks is the **banner**: the correction's author owed it a pointer, and
  `ROADMAP.md:930` is precisely the *stale end* a cold session enters from — a live Sequenced entry
  marked **TOP PRIORITY**, 2,400 lines from the resolution.
- **Cohort bound, stated honestly:** I searched `ROADMAP.md` and `ROADMAP-history.md` only, for the
  literal figures `10,951,030` / `10.95` / `7,304,581`. **I did not search `journal/`, `docs/` or
  `crates/`** — the resolution entry asserts those are stamped and I did not verify it.
- **Disposition: STILL-LIVE (a missing banner, not a wrong number).**

---

## 2. Checked and found NOT stale — recorded so the next sweep does not re-walk them

- **`GOLDEN_LEDGER` is "armed, not owed"** (`ROADMAP.md:3883-3888`) — verified: `weather_inventory`
  falls through to `false` (`grid.rs:607`), so `DeepField::ledgers` is genuinely empty. True.
- **P2's pits bar** — already stamped `~~the pits-bar conflict~~ **DISSOLVED 2026-08-04
  (corrections #98…)**` in the **live** entry at `ROADMAP.md:1707`, not only in the close block.
  Correctly propagated; the counter-example to F3/F5.
- **The walk-loop stop-channel entry** (`ROADMAP.md:3638-3660`) — correctly re-titled *"NO **FAST**
  STOP CHANNEL"* and carries corrections #100's ⚠ banner inline. Correctly propagated.
- **The unit-count Observed entry itself** (`ROADMAP.md:3373`) — stamped resolved. (Its cohort is
  F11.)
- **The layer-cake entry** (`ROADMAP.md:4595-4612`, user, 2026-07-20) — **STILL-LIVE and
  reinforced**, not stale: the GEO-5 user observation *"Nothing's getting folded on the record
  level for sure, least of all in a way refinement knows how to represent"* (`ROADMAP.md:5169`) is
  the same finding sixteen days later, and neither points at the other. Not filed as a defect
  because both are open user data; flagged only because a pointer between them is cheap and this
  corpus has measured what one-directional pointers cost.
- **The lateral-quantization entry** (`ROADMAP.md:3383-3406`) and **E4-2's fork**
  (`ROADMAP.md:3410-3418`) — both explicitly carried forward by the GEO-5 close block
  (`ROADMAP.md:5182`). Open by intent.

## 3. Unresolved — stated as unresolved

- **Whether F3's "B7" entry covers more than the shipped slice one.** `b8b1d14` says *"slice one"*;
  the entry does not enumerate slices. **CANNOT-DETERMINE without the design pass
  (`docs/audits/2026-08-02-joint-limits-b7-design.md`, unread by me).** The integrator owns it.
- **Whether F9's eight tectonics measurement groups are still the right instrument** for the user's
  2026-08-05 observation. A design call; a sweep may not adjudicate it.
- **F11's cohort outside the two ROADMAP files.** Not searched; see the bound stated there.
- **`docs/audits/2026-08-05-debt-walkthrough-notes.md` was read for context only**, per the brief's
  standing warning about its ⬦-marked claims. **No finding above rests on it.** Its 14-item
  inventory (items 5–14 unwalked) is carried by § Sequenced *THE HONEST RECORD*
  (`ROADMAP.md:383`), which correctly names the notes as the arc's working doc — that pointer is
  live and bidirectional, and is not a finding.

---

*Sweeper: background agent, worktree `agent-a1693dd9b7599a524`, model Opus 5. Read-only except
this file and `docs/audits/.sweep-watermarks.json`. Watermark advanced to `08afb88` in the same
commit as this document.*
