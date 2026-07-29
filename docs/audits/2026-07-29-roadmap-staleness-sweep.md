# ROADMAP staleness sweep — 2026-07-29 (FULL)

**What was swept, and against what.** Every entry of `ROADMAP.md` — § In flight (`:20-356`),
§ Sequenced (`:357-2373`), § Observed (`:2374-3535`) and the close block (`:3538-3645`) —
against everything landed in **`f652b60..72fbe86`** (76 commits: journals 0122–0127,
corrections #71–73, the baseline-sweep application, the first watermarked `spine-audit`, archive
pass 2, and the E3/Schedule/member-#0/tripwire arc). **151 entries checked** (130 top-level
bullets + 21 non-bulleted lead blocks).

**Mode: FULL, and the trigger is verified rather than assumed.** The watermark is one active day
old, but two independent full-run triggers fired: (1) **the board was restructured** — archive
pass 2 (`0764115`, 59 classified ops, 1,347 lines out), the close-block archive (`6eff1c8`, 409
lines), and Observed **129 → 85** entries (`610d2b5`); (2) **a recalibration landed and it IS
enabled by default** (§ 1 below).

**No ROADMAP or design-doc edits made — the integrator applies.** This document and the
`staleness` key in `docs/audits/.sweep-watermarks.json` are the only writes.

---

## 1. The recalibration verdict — TWO recalibrations, opposite dispositions

The brief passed the hook's claim as a hypothesis to test. It is **half right, and the half that
is right is not the half that matters most.**

### 1a. `creep_substep` — **ENABLED BY DEFAULT. The hook's claim is CORRECT.**

| evidence | where |
|---|---|
| `creep_substep: true` in the shipped config | `crates/dc-worldgen/src/deeptime/grid.rs:563` |
| the doc block says so in its own words: *"it takes **`n = 2`**, `DeepConfig::creep_substep` defaults **on**, and **its goldens moved with the fix**"* | `crates/dc-worldgen/src/deeptime/erosion.rs:3150-3160` |
| standing check that the shipped world is past the bound | `crates/dc-worldgen/tests/creep_operator.rs::the_shipped_world_has_cells_past_the_bound` |
| the pre-0122 operator is the *opt-out* (`creep_substep: false`), not the default | `tests/creep_operator.rs:34,48`; `examples/walk_tour_0115.rs:1110` |

**But its cohort-voiding reach is narrow, and that is a measurement, not a judgement.** On the
shipped arm the fix costs **two** sub-steps: mean regolith **4.57 → 3.91 m**, relief **−0.9 m**,
mean surface **−0.2 m**, closed hollows **0 → 0** (`ROADMAP.md:819-829`, journal/0122). So it
voids no Observed appearance report and no residency figure. **What it does void is the
`multiplier`-behaviour cohort** — every claim derived from sweeping the erosion knobs on the
*old* operator. Exactly one board entry still asserts one (finding **F1**).

### 1b. `calibrated_rates` — **STILL OFF BY DEFAULT. corrections #68's receipt still holds.**

`calibrated_rates: false` at `crates/dc-worldgen/src/deeptime/field.rs:374` and
`grid.rs:599`; `dc-client` only sets it behind an explicit `--calibrated-rates`
(`crates/dc-client/src/main.rs:153`); `examples/walk_tour_0115.rs:150-151` still asserts
*"production must still ship `calibrated_rates` OFF"*. journal/0122 reconfirms it in prose:
*"`calibrated_rates` still ships **false**."*

**So the 100-sub-step arm — the one where the fix changes everything (conc(h) rms 62.42 → 2.66 m,
818 hollows → 12) — is an arm nobody ships**, and observations of the shipped world are not its
artifacts. The skill's own qualifier survives its second field test.

---

## 2. Findings

Eleven findings. `file:line` on both sides throughout. The dominant shape is **not decay**: it is
the same *closure-in-the-wrong-place* the S6 baseline named (`ROADMAP.md:2494-2502`) — in **six of
eleven** cases the newer fact was written into `spines.md`, `dependency-graph.md` or the
watermark file, and the ROADMAP body that asked the question was never stamped.

### F1 — **CONTRADICTED (evidence base voided by the enabled-by-default recalibration).** "The band is NOT reachable at any multiplier"

- **Entry:** `ROADMAP.md:876-879` — CALIBRATE THE DEEP-TIME CLOCK, header: *"**The band is NOT
  reachable at any multiplier; read stubs #27 before assuming a bigger number fixes it.**"*
- **What stales it:** that claim is journal/0114's, measured on the **pre-`creep_substep`
  operator**, which is the thing journal/0122 fixed and shipped **on by default**. journal/0122
  measured the ladder again and **it inverted**: cover now *thins* with the multiplier
  (3.91 → 4.40 → 3.02 → 2.18 → 1.65 → 1.40 m) where 0114 had it *thicken*
  (4.6 → … → 782 m) — `ROADMAP.md:855-857`,
  `journal/0122-…:236-247`. journal/0122 states the consequence plainly: *"Transport now outruns
  supply… the joint-calibration argument… now has a different balance point. **That is a
  derivation to redo against a published band, not a number to pick.**"*
- **And its companion claim is flatly falsified.** 0114's *"no safe multiplier above 1×"* →
  journal/0122: deepest hollow at 5× is **2.9 m**, nothing anywhere deeper than 10 m until 45×.
  *"There is a safe multiplier underneath. **There are five of them.**"* (`journal/0122:229-234`).
  The merge commit `8987944` says it in its own message.
- **Disposition:** the header's blanket claim must come down. The honest replacement is *"not
  re-measured under the fixed operator; 0114's ladder is superseded (journal/0122) and the
  denudation band has not been re-swept"* — **not** a claim in either direction. Its sibling
  entry 25 lines earlier (`:850-873`) already carries the inversion; the two disagree in the same
  section. **This is the one genuine recalibration-cohort casualty on the board.**

### F2 — **STALE.** CoarseField "BUILT, AND NOTHING CALLS IT — the whole API"

- **Entry:** `ROADMAP.md:984-988` — *"Verified workspace-wide at `ae4bb29`: `sample_dithered`,
  `sample`, `summarize`, `DitherSource` (trait, **zero production impls**) and `CoarseField<T>`
  itself all have **zero callers outside `dc-core`**. Only `ShareVec<N>` landed."*
- **What stales it:** journal/0125 (`0dcdadb`/`0841816`). **Three of the five are now called.**
  `docs/spines.md:1474` carries the corrected status verbatim — *"**NOW CALLED (2026-07-29):
  `CoarseField<T>` + `from_cells` + `sample_dithered` + `Registration`, from
  `collapse.rs::surface_class` / `class_window`**… **`DitherSource` has its first production
  impl**, `dc-worldgen/src/draws.rs::Coherent`"* — and marks the row **PARTIAL** because only
  `sample` and `summarize` remain uncalled. Verified at source: `CoarseField`/`sample_dithered`/
  `DitherSource` now appear in `crates/dc-worldgen/src/collapse.rs`,
  `crates/dc-worldgen/src/draws.rs`, `…/deeptime/field.rs`, `…/deeptime/lithology.rs`.
- **Disposition:** rewrite the bullet to the two-of-five residual `spines.md:1474` already
  states. *The correction was written into `spines.md` and never back into ROADMAP — one
  directional pointer, in the entry that exists because the mechanism was invisible to all three
  loose-end loci.*

### F3 — **STALE.** "AND IT IS INVISIBLE TO ALL THREE LOOSE-END LOCI" + its owed action

- **Entry:** `ROADMAP.md:989-994` — *"It is in **neither** `spines.md` § 3 … **nor** `stubs.md`
  **nor** — until now — ROADMAP"*, closing with *"**Add the § 3 row in the same commit as the
  first adoption slice.**"*
- **What stales it:** the § 3 row **exists** — `docs/spines.md:1474`, added 2026-07-29 — and the
  adoption slice it was owed against has shipped (journal/0125). The instruction is discharged.
- **Disposition:** strike the instruction; keep the *mechanism* paragraph (*"extraction is not
  adoption"*), which `spines.md:1474` now quotes as § 3's second self-indictment.

### F4 — **STALE (verified at source).** `collapse.rs:949` "still defers to the CoarseField extraction (audit Part 2)"

- **Entry:** `ROADMAP.md:987` — *"`collapse.rs:949` still defers to *'the `CoarseField<T>`
  extraction (audit Part 2)'* as though it were pending."*
- **What stales it:** `grep -rn "audit Part 2" crates/` → **no matches anywhere in the tree.**
  The comment is gone (journal/0125 rewrote `surface_class`).
- **Disposition:** strike. **CANNOT-DETERMINE flag:** I did not establish whether the deferral was
  *resolved* or merely *deleted* — only that the text no longer exists. The integrator should not
  read "gone" as "answered".

### F5 — **SUBSUMED / the arc lives only in a close block.** The refinement design pass shipped and its own entry does not say so

- **Entry:** `ROADMAP.md:1042-1047` — REFINEMENT PRIMITIVES, *"**FIRST SLICE — a DESIGN PASS, not
  code**… It must answer: what is the primitive set · what the operator contract is in SDK terms
  … · and how a **mod** authors a visible channel end-to-end."* Still written as owed.
- **What stales it:** that design pass **ran and was ratified in this window**.
  `docs/design/refinement.md:1-9` — *"**Status: ~~PROPOSED~~ CAUTIOUSLY RATIFIED — user,
  2026-07-29**"* — and its own § head answers the entry's question by name: *"**What this document
  proposes is the surface**: the authoring shape, the coupling, and the first members. That is
  exactly the slot north-star § refinement left open."* Commits `3ee8b0e` (drafted) and `7ee2714`
  (ratified). `docs/dependency-graph.md` **E5** records the whole state including the built,
  walked member #0.
- **The mechanism, and it is the skill's § *What to look for* #6 exactly:** `grep -n
  "refinement\.md" ROADMAP.md` returns **four hits, all inside the close block**
  (`:3562, :3582, :3589, :3590`). **The § Sequenced entry whose deliverable it is never names the
  document.** A close block gets archived; a Sequenced entry is what the next cold session reads.
- **Worse: the entry WAS edited in this window** — the invocation-granularity clause was amended
  2026-07-29 (`:1051-1073`, `ee4fc02`) — so an author had the entry open and stamped one
  sub-clause while leaving its headline deliverable reading as unstarted.
- **Also stale inside the same entry:** `:1018` *"the **CONTRACT** half may still be live"* — the
  contract is what `refinement.md` is; and `:1029-1030` *"In code it is `collapse.rs` — **2,415
  lines**"* → **2,645 lines** today (`wc -l`).

### F6 — **SUBSUMED, and it is corrections #73's shape repeating.** The cake observation was answered; the asking entry carries no banner

- **Entry:** `ROADMAP.md:3395-3409` — *"**The cake observation: minority phases guillotine at cell
  perimeters under the coherent draw**"* (user, 2026-07-22), with *"Cure named, assigned to the
  CoarseField extraction: near boundaries, seeded membership dither of the SOURCE CELL (bilinearly
  weighted), then draw within that cell's shares."*
- **What stales it:** that cure **shipped and was walked**. `journal/0127` is *literally titled*
  **"the cake observation, answered; and the cake, swirled"**. `spines.md:1474`: *"**U22 (the cake
  law, far field) — ✅ DISCHARGED 2026-07-29**"* with the world-scale test named
  (`collapse.rs::far_class_dither_interfingers_across_a_real_deep_cell_frontier`). The user
  confirmed by eye — `ROADMAP.md:2382-2383`, *"welp, there are certainly no cell lines anymore."*
- **The asking entry has no ✅, no banner, no pointer to 0125/0127** — and the answer sits **1,013
  lines above it** in the same section. This is **read-first item 5's close-block clause and
  corrections #73's mechanism, recurring in the same window in which #73 was written**: the
  resolution went into a journal, a close block, `spines.md` and `dependency-graph.md`, and the
  document that asked was not stamped.
- **Second half of the entry is still owed and should survive the banner:** the sibling
  member-dither guillotine (`:3402`) is still unexamined (`ROADMAP.md:960` agrees).
- **One sub-clause is separately wrong:** `:3407-3409` still states *"the **toward-50/50** bias of
  interpolated-uniform noise"* unqualified. corrections **#39** falsified the sign — the source
  pushes splits **away** from 50/50 (`ROADMAP.md:202-205`, `:2465-2470`). Predates my window, so
  flagged rather than claimed as staleness.

### F7 — **STALE on all three clauses.** "NOT COVERED BY THIS BASELINE: spine-audit's question"

- **Entry:** `ROADMAP.md:424-427` — *"All nine slices were docs-vs-docs. **Nothing checked
  `spines.md` against the CODE**, which is why its watermark is deliberately `null`. **That is the
  next sweep**, and it is the one with a live finding already waiting for it (S-6 above)."*
- **What stales it:** (i) the watermark is **not null** —
  `docs/audits/.sweep-watermarks.json:9` reads `"spine-audit": "96ab14b"`; (ii) the sweep **ran** —
  `0e0b210` / `ec9858e` (*"spine-audit: the first watermarked pass"*); (iii) **the S-6 finding was
  applied** — `e568c5b` *"S-6 taught as author-and-validate, and ~90 citations walked back to the
  code"*, merged `b78b4dc`, plus `f10dc03` *"apply spine-audit findings: draws.rs overclaim
  corrected."*
- **And the resolution was recorded in the watermark file itself** —
  `.sweep-watermarks.json:15`: *"**RESOLVED 2026-07-29**: spine-audit ran its own full pass at
  `96ab14b`… the null is no longer outstanding."* **The JSON was stamped; the ROADMAP entry that
  asked was not.** Same one-directional shape as F2 and F6.

### F8 — **STALE / CONTRADICTED.** "NEXT SWEEP'S SPINE: ROADMAP § Observed (~1,970 lines), the largest unswept surface"

- **Entry:** `ROADMAP.md:496-497` — *"**NEXT SWEEP'S SPINE: `ROADMAP.md` § Observed (~1,970
  lines)** — the largest unswept surface in the corpus, and **the section the archive structurally
  could not reduce**."*
- **What stales it:** both halves are now false. (i) It **was** swept — `d69cdb3`
  *"audit(S6): complete read of ROADMAP § Observed — 129 entries classified"*, artifact
  `docs/audits/baseline-2026-07-28/S6-roadmap-observed.md`, which the section itself now cites at
  `ROADMAP.md:2490`. (ii) The archive **did** reduce it — `610d2b5` *"Observed: 129 → 85"*; the
  section measures **1,162 lines** today (`awk 'NR>=2374 && NR<=3535'`), not ~1,970. The
  "structurally could not reduce" claim is refuted by the mechanism that reduced it: archive **by
  status, not age** (`ROADMAP.md:2452-2459`).
- **Disposition:** strike both clauses; the next sweep's spine is a different question now.

### F9 — **STALE (number and disposition).** The `runner.rs` extraction candidate

- **Entry:** `ROADMAP.md:716-719` — *"**⚠ EXTRACTION CANDIDATE, NOT TAKEN (user call).**
  `runner.rs` is **1,694 lines** against the 700-line threshold (2.4×)… The cold half is the
  journal/0090/0104/0107 declaration-history commentary… *Not split mid-slice.*"*
- **What stales it:** the extraction **was taken** — `02aa81a` / `91b21a0`, *"the declaration
  archaeology moved out, the contract stayed put"*, producing
  `docs/design/pass-declaration-history.md` (183 lines). And it produced a **result the entry does
  not carry**: the commit records *"1,694 → 1,654; comment 686 → 646; **code 927 unchanged**…
  comment extraction **CANNOT** fix this file's size, because 927 of its lines are code. **The
  remedy left on the table is an ordinary module split, and it is not sequenced.**"*
- **And the number has moved the other way since:** `wc -l crates/dc-worldgen/src/deeptime/runner.rs`
  → **1,908 lines** (the Schedule slice, `de85bac`, added to it after the extraction). So the entry
  understates the file by 214 lines and still frames the *comment* extraction as the open move when
  it is done and measured insufficient.
- **Disposition:** the live item is the **module split** (the close block carries it at `:3629` as
  *"runner.rs module split (927 code lines)"* awaiting the user) — which is another close-block-only
  residence. Fold it into this entry.

### F10 — **UNBLOCKED / decision condition met.** The banner policy's own falsifier has run twice and held

- **Entry:** `ROADMAP.md:446-449` — *"**⚠ The policy's real test is the NEXT correction written**,
  not this backlog… *and we would know within a week, which is the cheapest possible falsifier.*
  **Watch that before investing in the sweep.**"* (written 2026-07-28)
- **What stales it:** two corrections were written in the window and **the same-commit obligation
  held both times.** **#72** (`journal/corrections.md:2954`, journal/0122) stamped its targets —
  verified at source: `journal/0116-…:5-14` now opens with *"**⚠ ITS D2 NULL WAS READ WRONG —
  corrections #72**… Every measurement in this entry stands… **One inference does not**"*, and the
  merge message enumerates banners on #63, journal/0116, stubs #29/#27, the ROADMAP entry and the
  0116 sweep log. **#73** (`:3026`, `fa5a7e8`) — close block `:3607`, *"Both targets stamped."*
- **Disposition:** the gate this entry set has data and nobody recorded the verdict. Record
  *"falsifier run twice, obligation held (#72, #73)"* — which is the input the entry itself said
  the sweep-investment decision needs. **Note F6 and F7 are counter-evidence at the ROADMAP
  boundary specifically:** the obligation holds for `corrections.md` → journal/spike targets, and
  fails for resolution → *the ROADMAP entry that asked*. That is a sharper finding than either
  half alone.

### F11 — **STALE (residence).** Two live items exist only inside the close block

- **`tectonics.rs` salt conversion (draws.rs residue 3)** — `ROADMAP.md:3637` only. Created by
  `f10dc03` (spine-audit application) in this window. `grep -n "salt conversion\|residue 3"
  ROADMAP.md` → **that one line.** The natural home is the *"Finish the draw-domain conversion"*
  entry at `:1103-1138`, which enumerates (a)/(b)/(c) and does not include it — so the enumeration
  is short, which is the exact pattern `:405-412` is about.
- **The tour instrument needs a LAND FILTER** — `ROADMAP.md:3636` only (*"station 1 scored a
  below-sea-level basin; finding recorded in that audit"*, `docs/audits/2026-07-29-far-frontier-tourmap.md`).
  No Observed or Sequenced home. A defective measurement instrument with no board entry.
- **Both are archived the moment the close block is rewritten.** Same class the 2026-07-25 run
  rescued (Movement 2b) and the 2026-07-29 archive pass re-filed (octree follow-ons, `:359-363`).

---

## 3. Checked and confirmed STILL OPEN — do not re-close these

Verified at source this pass so the next reader does not repeat the work:

| entry | verification |
|---|---|
| `flow_cost_probe` never converted (`:419-423`, `:2641-2653`) | `grep -c "#\[test\]" crates/dc-worldgen/examples/flow_cost_probe.rs` → **0**; `grep -n 'name = "flow_cost_probe"' crates/dc-worldgen/Cargo.toml` → **no match**. Still the one probe the gate cannot see fail. |
| `refine.rs:155` draw-domain conversion owed (`:1130`) | `crates/dc-worldgen/src/deeptime/refine.rs:155` still calls `draw_f64(&[cfg.seed, super::grid::SALT_DT_ROUGH, …])` raw. |
| `journal/0059` is a LOST entry (`:416-417`) | no `journal/0059-*.md` exists (`ls journal/` — 0058 then 0060). Still lost. |
| journals **0110/0111/0112** have no archive entry (`:417-418`) | `ROADMAP-history.md` has dated Shipped entries for 0108/0109/0113/0114 (`:203, :143, :96, :52`) and **none** for 0110/0111/0112 — they appear only inside an archived close block's "Shipped today" list (`:5631, :5634, :5645`). **0111, the scale recalibration, still has no archive entry.** |
| `DeepField::chapters` built and called by nothing (`:2610-2615`) | `spines.md:1467` re-confirms at `96ab14b`: no production caller, only tests. Now **tripwired** by `GOLDEN_CHAPTERS` — *"a golden is a test reader, not a consumer."* |
| `DeepField::strata` CSR collapse (`:2655-2675`) · `derive_regolith_at` empty ledger (`:2708-2723`) · `BENCH_SEED` / no `--seed` (`:3143-3147`) · `TerrainGen` seal (`:3247-3250`) · charcoal (`:3504-3534`) | all five already carry ✅-verified-2026-07-29 stamps from the S6 application (`9d08492`). Spot-re-checked; the stamps are accurate. |

**User field reports: none marked resolved by this sweep on reasoning.** The one that moved did
so on **evidence the world changed** — the cake observation (F6), discharged by a shipped slice
*and* a live user verdict by eye. The 15 open field reports the close block carries (`:3638`) stay
open.

---

## 4. Flagged as CANNOT-DETERMINE

1. **F4's residual** — the `audit Part 2` deferral text is gone from the tree; whether the
   question it deferred was *answered* or the comment merely *deleted* is not establishable from
   the diff alone. Do not read absence as resolution.
2. **The "8 spike/audit files need supersession banners" count** (`:413-415`) — I could not
   re-derive it. Seven files under `docs/spikes/` carry supersession-shaped text
   (`S2, S9, S10, S12, S17, S18, S20`), which neither confirms nor refutes "8 need one"; the
   baseline's list of *which* eight is in `docs/audits/baseline-2026-07-28/S8-*.md` and was not
   re-walked here. **Unchanged as far as I can tell; not verified.**
3. **No cargo was run** (docs-only agent, per brief). Every code claim above is a source read or
   a `grep`/`wc`, never a build or a test result. In particular I did **not** verify that the
   goldens `spines.md:1474` names actually pass.

---

## 5. What this sweep says about the board, in one paragraph

**The board's bodies are now the stale party relative to `spines.md` and
`docs/dependency-graph.md`, both of which were rewritten in this window and are accurate.** Six
of eleven findings are the same defect: a resolution written into the index, the graph, the
journal, the watermark file or the close block, and **never back into the ROADMAP entry that
posed the question**. That is not decay — corrections #73 was written *in this window* to name
exactly this shape, read-first item 5 was extended to close blocks *in this window* to fix it,
and it recurred **six times** anyway, twice inside entries an author had open for another edit.
The obligation demonstrably holds where the target is a journal or a spike (F10) and
demonstrably fails where the target is a ROADMAP body. If one mechanism is worth building out of
this sweep, it is the one that makes *"stamp the asking entry"* inseparable from *"record the
answer"* — the adoption law's own criterion (`:587-596`).
