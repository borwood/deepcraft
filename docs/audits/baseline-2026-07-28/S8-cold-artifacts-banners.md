# S8 — cold artifacts: the supersession-banner backlog + contradictions

> ## 📋 DISPOSITION — applied 2026-07-29. **All seven drafted banners are now ON their files.**
>
> | # | file | state |
> |---|---|---|
> | **B1** | `docs/spikes/S12-results.md` | **✅ APPLIED, with one deliberate departure from the draft.** The draft's closing paragraph said `ores.md` § 6 R1's *"until the erosion-supply calibration lands"* condition **has landed** and *"probe 3 is now runnable."* **That is wrong as of `journal/corrections.md` #70** (user, 2026-07-28), which rejected the exposure premise outright — *the fork is not blocked and probe 3 measures the wrong thing.* The banner as applied says so instead, and adds that the calibration ships **OFF** behind `calibrated_rates`. |
> | **B2** | `docs/spikes/S18-first-behavior-weathering-plan.md` | **✅ APPLIED** as drafted. |
> | **B3** | `docs/spikes/S9-results.md` | **✅ APPLIED** as drafted. |
> | **B4** | `docs/audits/A1-collapse-slice-plan.md` | **✅ APPLIED** as drafted. |
> | **B5** | `docs/audits/2026-07-23-block-consumer-inventory.md` | **✅ APPLIED** as drafted. |
> | **B6** | `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md` | **✅ APPLIED** as drafted. |
> | **B7** | `docs/audits/2026-07-25-roadmap-staleness-sweep.md` | **✅ APPLIED** as drafted. |
> | **B8** | `docs/spikes/S19-flow-record-cost-results.md` | **⚠ ESCALATED** — the slice flags rather than asserts it and says settling it means running `flow_cost_probe` once. **A measurement, not an application.** |
>
> | contradiction | state |
> |---|---|
> | **C1** a user-owned ratification gated on a met condition | **❌ FINDING SUPERSEDED — and in the opposite direction.** C1 reasons that the *blocker landed*, so probe 3 should be run before § 6 R1 goes back to the user. On **2026-07-28** the user rejected the whole caveat (`corrections.md` **#70**): ore does not need to be surface-**exposed**; the fork was never blocked; **probe 3 measures the wrong quantity.** `ores.md` gained a 🔴 top-of-file banner the same day. **What this pass did instead:** struck the four caveat sites that banner named and left unstruck. *C1's instinct — that a stale assistant-side caveat was suppressing a user decision — was exactly right; only its remedy is dead.* |
> | **C2** `spines.md` publishes a retracted denominator | **✅ APPLIED at the read-first sites** — `spines.md` § S-5 and § 6 now read **31 live of 34 inventoried**, citing the seam inventory's own 2026-07-28 retraction. **The `5 converted` numerator was NOT re-verified against `crates/`** (the slice did not verify it either) and is explicitly carried forward as this file's number. **The corpus-knowledge sites were deliberately left alone**: they quote the string *as evidence that an obligation ledger has a denominator*, and that argument is unaffected by its value — rewriting a dated evidence reading is the wrong instrument. |
> | **C3** two spikes are both S16 | **⚠ ESCALATED** — *"User/integrator call; I am not resolving it."* |
> | **C4** `material-behavior.md` cites S18 as a live instance | **⚠ ESCALATED** — the slice calls it *"a proposal, not an edit"* because it touches a ratified design doc. **Its reciprocal half is closed regardless: B2 is on S18.** |

*Baseline `doc-topology` sweep, slice 8 of 9. **All quotations read at commit `f652b60`.***
*Read-only pass over `docs/spikes/*.md` (21 files), `docs/audits/*.md` (15 files at that
directory's root — the `baseline-2026-07-28/` subdirectory is this fan-out's own output and
was **not** read), and `ROADMAP-history.md`. 37 artifacts, ~14,150 lines.*

**This document resolves nothing and edits nothing.** Every banner below is *proposed text*
for the integrator to apply. **No spike, audit or history file was written to.**

**Governing policy:** CLAUDE.md read-first item 5 — **IMMUTABLE BODY, MUTABLE HEADER**
(DECIDED 2026-07-28, user). *The testimony is frozen; the banner is not.* The defect this
sweep hunts is a **missing pointer**, never a wrong number: a dated measurement stays true
about its day.

---

## 0. Method, and what I refused to flag

I greped `journal/corrections.md` (67 entries) for every spike id and every audit filename,
then read each hit **in full on both sides**, then read the head of all 37 artifacts, then
followed the surviving candidates into the live design docs.

**I did not propose a banner for a file merely because the world moved on.** Twelve files
came up in the grep and were dismissed on inspection, for three distinct reasons that are
worth recording so the next sweep does not re-walk them:

| file | why NO banner |
|---|---|
| `S11-results.md` | corrections **#14** (`journal/corrections.md:351`) and **#15** (`:381`) both name S11 — but S11 **already records both**, in its own body: § *"The boundary condition, and a wrong turn worth recording"* (`S11-results.md:56-80`) and § *"Falsified along the way"* (`:509`). **This is the model.** A spike that publishes its own falsifications needs no header. |
| `S13-results.md` | corrections **#24** (`:706`) and **#25** (`:746`) cite S13 as the **falsifier**, not the falsified. Its § 5 (`S13-results.md:286`) is itself titled *"hypothesis FALSIFIED"*. |
| `S15-results.md` | corrections **#30** cites `S15 § group 1` / `§ group 3` as evidence (`:1002`, `:1038`). S15 is the instrument. |
| `S9b-results.md` | the falsifier of S9's flip condition, and it names its target at `S9b-results.md:3`. Its verdict (*"parallelism does NOT flip the verdict"*) still stands, cited live and reciprocally at `docs/design/earth-processes.md:362-363`. |
| `S1`, `S3`, `S4`, `S5`, `S6`, `S8`, `S17` | nothing in the corpus refutes, supersedes or re-scopes them. `S4`'s fixed-sun constant is the *falsifier* in corrections #19 (`:545`). `S17`'s pleas were ratified into `material-behavior.md:96-97,134`. |
| `2026-07-22-hydrology-priors.md`, `2026-07-22-entry-species-probe.md`, `2026-07-22-threshold-quantization-audit.md`, `2026-07-23-perf-baseline-vertical-drop.md`, `2026-07-24-lod-pre-post-visit-diagnosis.md`, `2026-07-24-palette-quant-generation-diagnosis.md` | no correction names them. **Specifically checked and cleared:** corrections **#48** (`:1518`) falsified the palette-quant station's *"~110 km east of spawn"* — I greped for `110 km` across `docs/audits/` and **that distance appears in `ROADMAP.md:3409` only, already struck there**. The audit never carried it. *(Search stated per the sweep's absence-needs-a-pathspec rule: `grep -rn "110 km\|71 km\|km east" docs/audits/`.)* |
| `2026-07-24-roadmap-staleness-sweep.md`, `2026-07-26-doc-topology-sweep.md`, `2026-07-28-corrections-recoding.md` | self-dating sweeps whose genre is explicitly a snapshot; nothing later contradicts their findings. (The 2026-07-25 sweep is the exception — see row 7 below — because a *correction names it by path*.) |
| `ROADMAP-history.md` | **needs no banner and must not get one.** It is an archive of Shipped entries, each already keyed to its journal number; it is a container, not a claim. Its entries carry their own corrections inline (e.g. `:1954` carries #12). |

---

## 1. JOB 1 — the banner table

**Totals: 37 files examined · 5 already carry a banner · 8 need one · 24 need none.**

Two of the eight are **plan documents that outlived their execution** — a distinct class
from a refuted measurement, and arguably the cheaper half of the backlog, but they mislead
in the more dangerous direction: an agent reads a to-do list of already-done work.

### 1.1 Already carrying a banner (5) — no action

| file | banner | pointed at |
|---|---|---|
| `docs/spikes/S10-results.md:3-31` | ⚠ SUPERSEDED IN PART | corrections #12 |
| `docs/spikes/S2-results.md:3-30` | ⚠ NO PRODUCTION CALLER — kept deliberately | journal/0121, user ruling |
| `docs/spikes/S7-results.md:9-15` | ⚠ THE SETTLEMENT-HISTORY HALF NO LONGER EXISTS | journal/0121 |
| `docs/spikes/S20-fact-ledger-residency-results.md:16-22` | inline DECIDED + re-measured block | journal/0108 |
| `docs/audits/2026-07-22-seam-inventory.md:14-20` | ⚠ FOUR OF THE FIVE "SOCIAL SIM" SEAMS NO LONGER EXIST | journal/0121 |

### 1.2 Needs a banner (8), ranked by cost

---

#### **B1 — `docs/spikes/S12-results.md`** · ⚠ the most expensive missing banner

| | |
|---|---|
| **Refuted / re-scoped by** | `journal/corrections.md:1936` (**#56**, *"the deep-time engine's erosion rates are calibrated to the Phanerozoic register"* — falsified 2026-07-26 by journal/0111, the ~1000× scale error), and the calibration itself, `journal/0114` (cited at `journal/corrections.md:2167`, `:2215`, `:2257`) |
| **Carries a banner** | **N** |
| **Why it is the top item** | S12's *self-declared* **"load-bearing finding"** (`docs/spikes/S12-results.md:205`) is *"exhumation … **max 11 m, land-mean 4 m**"*, and `:218` says in its own words what the fix would be: *"exhuming their roots and filling their forelands **needs the denudation rate … turned up too**."* **That is precisely what journal/0111→0114 did.** S12 was right, was acted upon, and holds no pointer to the action. Meanwhile the metre-scale number is still gating a **user-owned ratification** — see contradiction **C1** below. |

> **PROPOSED BANNER**
>
> ```
> > ## ⚠ THE ERODING-RATE PREMISE UNDER § GROUP 7 AND THE EXHUMATION FINDING HAS BEEN RECALIBRATED
> >
> > **Read the relief table (§ Group 7) and the exhumation finding (§ Group 4, "max 11 m,
> > land-mean 4 m") as measurements of the PRE-CALIBRATION engine.** On 2026-07-26 the deep
> > sim was measured against the published literature for the first time
> > (`journal/0111`, `journal/corrections.md` #56) and found to be denuding **~10³ times too
> > slowly** — 5.48 m over a Phanerozoic span where a real craton strips 5–10 km. The rate
> > constants were recalibrated in `journal/0114`.
> >
> > **This document anticipated it.** § *"Exhumation is real but metre-scale"* states that
> > exhuming orogenic roots "needs the denudation rate … turned up too". That has now
> > happened; the magnitudes below have not been re-measured under the new constants.
> >
> > **What still stands, untouched:** every determinism, byte-identity, clamp and
> > mass-balance result (Groups 1, 2, 5, 6, 8), the *shape* of the amplitude response
> > (monotonic in `thickening_scale`), and the isostasy-caps-elevation mechanism.
> >
> > **What must NOT be quoted as current:** the absolute exhumation magnitude and the
> > Group 7 elevation columns. In particular `docs/design/ores.md` § 6 R1 / § 8.1 still
> > conditions the lode-gold call on *"until the erosion-supply calibration lands"* — it
> > has landed (with an open creep-limiter blocker, `journal/0116`); probe 3 is now
> > runnable and its answer is unknown.
> >
> > *Banner added under the immutable body / mutable header policy (CLAUDE.md read-first
> > item 5). Nothing below is edited.*
> ```

---

#### **B2 — `docs/spikes/S18-first-behavior-weathering-plan.md`** · both of its headline claims falsified

| | |
|---|---|
| **Refuted by** | `journal/corrections.md:1470` (**#46**, *"S18 weathering expresses a saprolite band in the walkable world"*) and `journal/corrections.md:1493` (**#47**, *"S18 is the first real weathering BEHAVIOR"*) |
| **Carries a banner** | **N** — the file is 22 lines and contains no marker of any kind |
| **Why it matters** | The plan's stage **E** reads *"Collapse folds `base + facts` — a basal weathering-front band expresses"* (`:16`) and its title claims *"first real cellular behavior"* (`:1`). #46 measured the band at **0.04 m against 0.9 m voxels — zero eighths everywhere**; #47 ruled the whole slice a **category error** (a one-shot snapshot of a continuous process). Both halves of a two-claim document are dead, and the document says so nowhere. Its heir is tracked at `docs/design/stubs.md:506,518`. |

> **PROPOSED BANNER**
>
> ```
> > ## ⚠ BOTH OF THIS PLAN'S HEADLINE CLAIMS WERE FALSIFIED AFTER IT SHIPPED
> >
> > The slice below shipped (`journal/0089`). Two of its claims did not survive:
> >
> > - **Stage E — "a basal weathering-front band expresses" — FALSE.**
> >   `journal/corrections.md` #46: at production scale the strongest band anywhere is
> >   **0.04 m** against **0.9 m** voxels, which quantizes to **zero eighths everywhere**.
> >   The collapsed world is effectively byte-identical. There is nothing to walk to.
> > - **The title — "first real cellular behavior" — FALSE.**
> >   `journal/corrections.md` #47: weathering is *continuous*; this runs `weather_column`
> >   **once**, after the run, over the finished record. A snapshot of a continuous process
> >   is a **category error, not a simplification**. What shipped is
> >   keystone-consumption **plumbing** with a one-shot behavior stub.
> >
> > **What stands:** stages A–D and F are real plumbing and are consumed today —
> > `Fact::cause`, the applied-edge log, the bedrock `Structure` seam, the `FactLedger`
> > sidecar, the empty-ledger identity floor.
> >
> > **The heir** is weathering run as a per-epoch pass riding the `H` process (the R/H
> > unification) — `docs/design/stubs.md` #17.
> >
> > *Banner added under the immutable body / mutable header policy (CLAUDE.md read-first
> > item 5). Nothing below is edited.*
> ```

---

#### **B3 — `docs/spikes/S9-results.md`** · the textbook one-directional edge

| | |
|---|---|
| **Refuted by** | `journal/corrections.md:139-158` (**#9**), and `docs/spikes/S9b-results.md:3-9` which quotes S9's recommendation verbatim as the thing it is testing |
| **Carries a banner** | **N** |
| **The shape** | Identical to the `#12 → S10` case that motivated the whole policy. **S9b names S9. S9 names nothing.** #9 falsifies S9's stated flip condition (*"a parallel priority-flood could pull B to ~2 minutes"*) — real parallel B is **15–70 min** — and separately corrects a measurement: *"S9's recorder empty-header estimate was 648 MiB; measured 833 MiB"* (`journal/corrections.md:151`). The verdict S9 reached (**A+C**) is **unchanged**; only its escape hatch and one estimate died. |

> **PROPOSED BANNER**
>
> ```
> > ## ⚠ THE FLIP CONDITION IN § Recommendation WAS TESTED AND FAILED — see S9b
> >
> > **The verdict stands: A always-on + C refinement.** What did not survive is the
> > *"UNLESS the erosion engine parallelizes"* escape hatch below.
> >
> > **Falsified by [`S9b-results.md`](S9b-results.md) and
> > [`journal/corrections.md` #9](../../journal/corrections.md).** The priority-flood is
> > 65–72 % of the step (98.5 % at B scale) and has **no byte-identical parallel form**;
> > whole-step speedup is 1.18–1.26×, so realistic parallel B is **15–70 min**, not the
> > ~2 min this document projected. The question reopens only on ~32-core hardware with a
> > deterministic parallel flood — `examples/deeptime_par.rs --full-b` re-measures it.
> >
> > **One measurement below is also corrected:** the recorder empty-header estimate of
> > **648 MiB** measured at **833 MiB**.
> >
> > **Everything else stands**, including the A/B/C cost and read-quality comparison that
> > the spike existed to make. See also the erosion-rate recalibration banner class —
> > `journal/0111` / corrections #56 — which re-scopes the *magnitudes* this tier produces
> > without touching its architecture verdict.
> >
> > *Banner added under the immutable body / mutable header policy (CLAUDE.md read-first
> > item 5). Nothing below is edited.*
> ```

---

#### **B4 — `docs/audits/A1-collapse-slice-plan.md`** · a plan that outlived its execution

| | |
|---|---|
| **Superseded by** | `docs/spines.md:213` (*"compliance (2026-07-23, journal/0087 — the block↔material collapse, A1)"*), `ROADMAP-history.md:949`, `:2695` |
| **Carries a banner** | **N** |
| **Why** | 17 lines of imperative future tense — *"1. Redefine `Block = { Air, Material(MaterialId) }` … 3. Migrate render path … Do NOT: build deep-cell inventory"* — for work that **shipped 2026-07-23**. It even names a branch (`worktree-agent-a63c85f041dedde50`) and a base commit that no longer signify. The file reads as live instructions to an agent that opens it. Low blast radius, near-zero cost to fix. |

> **PROPOSED BANNER**
>
> ```
> > ## ✅ EXECUTED 2026-07-23 — journal/0087. This is a historical plan, not a to-do list.
> >
> > Every step below shipped: `Block` is `{ Air, Material(MaterialId) }`, `block_twin` and
> > the `block_layer` geology arms are gone, solidity checks ride the occupancy primitives.
> > See `docs/spines.md` § compliance (2026-07-23) and `ROADMAP-history.md`. The branch and
> > base commit named above are dead references. **Preserved as the record of what was
> > planned and in what order; do not act on it.**
> ```

---

#### **B5 — `docs/audits/2026-07-23-block-consumer-inventory.md`** · a pre-collapse census still stated in the present tense

| | |
|---|---|
| **Superseded by** | the same collapse — `journal/0087`, `docs/spines.md:213` |
| **Carries a banner** | **N** |
| **Why** | Its § B header reads *"Consumers, categorized (135 sites / 31 files in the 0052 audit; **146 raw `is_solid|==Air` hits / 34 files today**)"* (`:21-22`) — *today* being 2026-07-23, before the collapse it was written to ground. Every count and every `file:line` in it is a pre-collapse address. It is genuinely useful testimony about the tail's shape and should not be deleted; it just must not be read as a current census. |

> **PROPOSED BANNER**
>
> ```
> > ## ⚠ PRE-COLLAPSE CENSUS — the collapse it grounds SHIPPED 2026-07-23 (journal/0087)
> >
> > Every count and `file:line` below is addressed to the tree **before** the
> > block↔material collapse. `block_twin` is gone, `classify` returns the material, and the
> > render path's re-translation arms were deleted. **The counts (135 sites / 31 files;
> > 146 raw hits / 34 files) are this audit's, not today's** — re-grep before quoting one.
> > The categorization (solidity-shaped vs identity-shaped) is what has aged well and is
> > why the document is kept.
> ```

---

#### **B6 — `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`** · a diagnosed defect, since fixed

| | |
|---|---|
| **Superseded by** | `journal/corrections.md:1548` (**#49**), `journal/0101`, and **CLAUDE.md § Agent walks** which now states the fixed behaviour: *"`has_contents` is now a PER-VOXEL fact and is trustworthy (fixed 2026-07-25, journal/0101; it used to be answered per-CHUNK — corrections #49)"* |
| **Carries a banner** | **N** |
| **Why** | The audit's § 8.1 is a *"ready to paste"* draft correction and § 8.2 is *"ROADMAP Observed — replacement wording"*. Both were consumed. A reader landing here — and this is a 506-line diagnosis of an **instrument**, exactly the kind of document a walking session opens mid-walk — cannot tell from the file that the instrument was repaired the same day. The mechanism narrative below is still the best account of the query path and is worth keeping. |

> **PROPOSED BANNER**
>
> ```
> > ## ✅ FIXED THE SAME DAY — journal/0101, corrections #49. The instrument described below
> > ## no longer behaves this way.
> >
> > `has_contents` is now a **per-voxel** fact and is trustworthy. It used to be answered
> > per-CHUNK, which is the defect this document diagnoses. `identify(pos)` is untiered and
> > `UNRECORDED` is first-class. **CLAUDE.md § Agent walks carries the current reading rule
> > — use that, not § 1 below, when interpreting a live probe.**
> >
> > § 8's recommended corrections were all filed (corrections #49; ROADMAP Observed).
> > **Kept for its § 2 end-to-end mechanism trace**, which is still the clearest account of
> > how contents, solidity and `dc:air` relate in the query path.
> ```

---

#### **B7 — `docs/audits/2026-07-25-roadmap-staleness-sweep.md`** · a correction names it by path; it does not know

| | |
|---|---|
| **Named by** | `journal/corrections.md:1883` (inside **#55**): *"`docs/audits/2026-07-25-roadmap-staleness-sweep.md` is likewise a dated snapshot."* |
| **Carries a banner** | **N** (it has no top-of-file dating caveat; the `**Read-only audit, 2026-07-25**` line at `:3` is a provenance stamp, not a staleness marker) |
| **Why** | A full-path, one-directional correction→file edge — one of the 8-of-15 the policy was written to close. Cheapest possible fix, and it is the exact shape the measured baseline counts. |

> **PROPOSED BANNER**
>
> ```
> > ## ⚠ A DATED SNAPSHOT — named as such by `journal/corrections.md` #55 (2026-07-26).
> >
> > This sweep's verdicts describe the board **as of 2026-07-25**, before the deep-time
> > scale finding (`journal/0111`, corrections #56), the erosion recalibration (0114) and
> > the bootstrap-content removal (0121). **Its "STALE" and "UNAFFECTED" columns are not
> > current.** Later staleness sweeps supersede it item-by-item; this one is kept so a
> > reader can see what was true on the day and what the next sweep did not have to
> > re-walk (§ *Checked and found genuinely current*).
> ```

---

#### **B8 — `docs/spikes/S19-flow-record-cost-results.md`** · ⚠ CANDIDATE — integrator should verify before applying

| | |
|---|---|
| **Possibly re-scoped by** | `journal/corrections.md:1825` (**#54**, *"MFD needs the head field"* — falsified the same day by journal/0109, retiring `dc:field/head`), `journal/corrections.md:1888` (**#55**, *"the fluvial transport pass is what moves this world's sediment"* — falsified by journal/0110's own probe), and **CLAUDE.md § Gates**, which records that this spike's own instrument, `flow_cost_probe`, **broke twice after the spike ran** — once *"off by the whole 42.6 MB flux record"* and once for a missing `head` row |
| **Carries a banner** | **N** |
| **Confidence** | **Lower than B1–B7 and I am flagging rather than asserting.** S19's headline itemisation is asserted equal to `resident_bytes()` at run time (`S19-results.md:20-24`), so its *arithmetic* was self-checking on the day. What I could not settle inside this slice is whether the **layouts it prices** (slot pairing, the flux record's shape) still describe the shipped flow record after #54 retired the head field and #55 falsified the pass's importance. `docs/design/flow.md:609-611` quotes S19 as the live cost answer to its § 9 Q1. **Recommend the integrator run `flow_cost_probe` once and either stamp a banner or record that the numbers held.** |

---

## 2. JOB 2 — contradictions

**4 findings.** All four are the *same underlying failure* as the banner backlog: a cold
artifact's number is quoted as current in a live doc, and the cold artifact holds no link
back. Ranked by blast radius.

---

### **C1 — a user-owned ratification is still gated on a condition that has been met**

| side | citation @ `f652b60` | statement |
|---|---|---|
| **live, asserted as pending** | `docs/design/ores.md:143-147` | *"**A measured warning on the flagship** — ROADMAP/S12 (2026-07-20): **exhumation comes out metre-scale at shipped erosion rates**, so exhumed-core signals are illegible at any amplitude **until the erosion-supply calibration (Sequenced) lands**."* |
| **same doc, same condition, inside a ratification fork** | `docs/design/ores.md:383-386` | *"Caveat: S12 measured metre-scale exhumation, so **until the erosion-supply calibration lands** the gate may barely discriminate (§ 8.1) — probe 3 measures this first."* |
| **and again in the roster confirmation** | `docs/design/ores.md:485-486` | *"this pass's § 8 reports two collisions discovered since (**the S12 exhumation finding against lode gold**; the Phanerozoic register against BIF)"* |
| **the cold source** | `docs/spikes/S12-results.md:205`, `:210`, `:218` | *"exhumation \| **max 11 m, land-mean 4 m**; cells > 500 m: **0** \| **the load-bearing finding**"* … *"exhuming their roots and filling their forelands **needs the denudation rate (or the iteration count) turned up too**"* |
| **the event nobody propagated** | `journal/corrections.md:1936-1975` (#56); calibration `journal/0114`, cited at `journal/corrections.md:2167`, `:2215`, `:2257` | the engine was **~10³ too slow**; *"real cratons strip **5–10 km** … this world strips **5.48 m**"*. **The erosion-supply calibration landed 2026-07-26.** |

**Blast radius: HIGH.** `ores.md` § 6 is a **NEEDS RATIFICATION** section — the exact
artifact a user reads to make a call. It offers *"Option A, conditioned on probe 3"* against
a blocker described as unlanded. The blocker landed. Whether the gate now discriminates is
**unknown and cheaply measurable** (probe 3 is already specified at `ores.md:363-365`).
Neither `ores.md` nor `S12-results.md` knows about `journal/0114`.

**Provenance:** the roster is **user-ratified** (2026-07-20, `ores.md:483`); the caveat
blocking it is **assistant-originated measurement**. Not a tie — but note which way it cuts
here: the stale caveat is *suppressing* a user decision, not overriding one.

**Recommendation (labelled as one):** apply banner **B1** to S12, and file a ROADMAP/Observed
item to run probe 3 under the calibrated constants before § 6 R1 is put to the user again.
**Do not edit `ores.md`'s § 6 wording** — an unrun probe is not an answer, and the
creep-limiter blocker (`journal/0116`) means the calibrated regime may not be final.
**⚠ This one is close to `CLAUDE.md`'s "a user-originated design may not be superseded by an
implementation slice" — a user-ratified roster is being held hostage by a stale
assistant-side caveat. Surfacing it in main session is the right disposal, not a doc edit.**

---

### **C2 — `spines.md` publishes a denominator the seam inventory has since retracted**

| side | citation @ `f652b60` | statement |
|---|---|---|
| **read-first, asserted as current** | `docs/spines.md:348` | *"**34 seams inventoried; 5 converted** — `outcrop_at`, `wave_energy`, `parent_p`, `depth_to_water`, and `burial_temp_c`"* |
| **read-first, same claim, § 6** | `docs/spines.md:1258-1262` | *"`2026-07-22-seam-inventory.md` — all **34 seams** … **19 of 34 are arbitrary, not identities** … **26 of 34 are invisible to the pass graph**"* |
| **the cold source, which retracted it** | `docs/audits/2026-07-22-seam-inventory.md:18-20` | *"**This audit is a dated snapshot and is not renumbered** — later docs cite "34 seams" and that count is this audit's, not today's. **Live seams: 31, not 34.**"* |

**Blast radius: HIGH, and unusually so.** `spines.md` is **read-first item 0b** — every
session and every agent loads it. Worse, this exact string has been promoted into a
docs-ops *argument*: `docs/design/corpus-knowledge-notebook.md:387` calls
*"34 seams inventoried; **5 converted**"* **the only place in the corpus where an obligation
ledger has a denominator**, and `:501` and `:554` build a proposed intervention on the
property that separates S-5 from six failing conventions. `ROADMAP.md:951` and
`docs/design/corpus-knowledge-evidence.md:603` repeat it. **A wrong denominator is now
load-bearing on a design decision about how the corpus records obligations** — which is a
tidy demonstration of the very failure the notebook is about.

**This is the reciprocity shape exactly:** the seam inventory grew a banner on 2026-07-28
that *predicts this misuse by name* (*"later docs cite '34 seams'"*) — and the docs it
predicts still cite it, because a banner on the cold end cannot reach them.

**Recommendation:** `spines.md:348` and `:1258` should read **31 live / 34 inventoried**,
with the 5-converted numerator re-verified against the tree (I did **not** verify the
numerator — search stated: I read `spines.md` and the inventory's banner only, and ran no
grep over `crates/`). The notebook's *argument* survives intact either way — its point is
that a denominator **exists**, not what it is — but the quoted string should be corrected in
all five sites in one commit, since the whole hazard is that it travels.

---

### **C3 — two different spikes are both called S16, and one of them is described as unwritten**

| side | citation @ `f652b60` | statement |
|---|---|---|
| **live design doc** | `docs/design/water.md:756` | *"**Consequence for S16 (unwritten):** draft it against an **octree node**, not [a per-deep-cell summary]"* |
| **live design doc** | `docs/design/octree-substrate.md:36-38` | *"S15 proved a 64× hypsometric compression answers levels coarsely (`S15-results.md`); **S16 is unwritten** and drafts against an octree **node**"* |
| **live design doc, third site** | `docs/design/octree-substrate.md:118-119` | *"(requirements only — hydrology stays paused; **S16 drafts against this stratum**)"* |
| **the artifact that owns the id** | `docs/spikes/S16-weathering-behavior-shape-results.md:1-6` | *"# S16 — Weathering behavior-shape spike results · **Date: 2026-07-23**"* — complete, shipped, `ROADMAP-history.md:910` |
| **and it is cited as S16 elsewhere, in the other sense** | `docs/design/material-behavior.md:282`, `:318`, `:335`, `:410`, `:537`; `ROADMAP.md:57` | *"(S16's `WeatheringPass` is the first instance.)"* etc. |

**Blast radius: MEDIUM.** Shape 4 — *one number, several values* — in its purest form: **one
id, two referents.** A reader who follows `water.md:756` to `docs/spikes/S16*` finds a
**completed** spike about weathering and must either conclude the water question was
answered or work out that the id was reused. Neither doc flags the collision. The water-side
S16 appears never to have been dispatched; the weathering spike took the next free number on
2026-07-23 and nothing reconciled them.

**Recommendation:** the water-side reference should be renamed to something id-free (*"the
water-summary spike (unwritten)"*) rather than renumbered — spike ids are cited ~1,130 times
corpus-wide (`docs/design/corpus-knowledge-evidence.md:280`) and renumbering a live one is
the more expensive move. **User/integrator call**; I am not resolving it.

---

### **C4 — `material-behavior.md` cites S18 as a live instance of a shape whose instance was falsified**

| side | citation @ `f652b60` | statement |
|---|---|---|
| **live design doc** | `docs/design/material-behavior.md:579` | *"`wood` owns `combust→`; `limestone` owns `dissolve→`. ***S18's weathering is the first live instance*** (bedrock owns `weather→`, gated by agents)."* |
| **live design doc, the four-way test table** | `docs/design/material-behavior.md:638` | *"\| **1** \| "that material, transformed" — driver exists \| transform edge, input-owned \| **bedrock→saprolite (S18)** \|"* |
| **the falsification** | `journal/corrections.md:1493-1516` (**#47**) | *"S18 runs `weather_column` **once**, after the run, over the finished record … **a snapshot of a continuous process, a category error, not a simplification.** What shipped is keystone-consumption **plumbing** with a one-shot behavior stub, **not weathering-as-a-process**."* |
| **and the outcome half** | `journal/corrections.md:1470-1490` (**#46**) | the saprolite band *"reaches **zero eighths everywhere**"*; *"a column scan at the strongest cell reads stone/dirt/air, **no saprolite voxel**"* |

**Blast radius: MEDIUM.** This is the weakest of the four and I want to be precise about
what is and is not contradicted. The **edge-ownership claim is true**: `weather→` really is
declared on bedrock, and that is what § *The organizing principle* is arguing. What is false
is the word **"live"** and the table's implication that `bedrock→saprolite` is a *driver
exists* case in the world — the driver runs once, out of the loop, and the product does not
express. A reader picking the exemplar out of the four-way test table takes away that this
category is demonstrated in shipped, walkable content. It is not.

**Recommendation:** the honest repair is a clause, not a deletion — *"S18's weathering is the
first declared instance (the edge is real; the pass that drives it is a one-shot stub —
corrections #46/#47, heir at `stubs.md` #17)"*. **Integrator's call**; it touches a ratified
design doc, so it is a proposal, not an edit. Applying banner **B2** to S18 closes the
reciprocal half regardless.

---

## 3. Coverage — what I read and what I did not

**Read in full:** `docs/spikes/S18-*` (22 ll), `docs/spikes/movement2a-*` (40 ll),
`docs/audits/A1-collapse-slice-plan.md` (17 ll), the `.claude/skills/doc-topology/SKILL.md`,
and every `journal/corrections.md` entry that names a spike or audit path (#9, #12, #15, #19,
#46, #47, #53, #55, #56, #67 — read whole).

**Read at targeted depth** (head + every section a grep hit landed in): the other 18 spikes,
the other 13 audits.

**Read only as grep context:** `ROADMAP-history.md` (2,835 ll). I confirmed it needs no
banner as a container and spot-checked that its S10/S12 entries carry their corrections
inline (`:1954`), but I did not read it linearly. **A later slice should confirm no Shipped
entry in it asserts a pre-calibration erosion magnitude without a pointer** — that is the
one gap I am consciously leaving.

**Not opened at all:** `docs/audits/baseline-2026-07-28/` (this fan-out's own output, per
brief); `crates/**` (this sweep is docs-vs-docs; every code claim above is quoted from a doc,
never verified against source — and **B8** is flagged precisely because settling it requires
running a probe).

**Two things I checked for and did not find**, stated with their pathspec so the next sweep
does not repeat them:
- `grep -rn "110 km\|71 km\|km east" docs/audits/` → **no hits.** Corrections #48's wrong
  station distance never entered an audit file.
- `grep -rn "nine Class B\|all nine\|nine test tolerance" docs/` → the only live carrier is
  `docs/spikes/S20-fact-ledger-residency-results.md:476`, and **S20 already carries its own
  header block** (`:16-22`) recording that 2c shipped. Corrections #53's *"nine sites"*
  over-prediction is therefore **not** a missing-banner case, though the integrator may
  judge that S20's existing block should name #53 explicitly — it currently does not.
  *(That last one is the closest thing to a ninth banner in this slice; I am reporting it as
  a note rather than counting it, because the file is not silent.)*
