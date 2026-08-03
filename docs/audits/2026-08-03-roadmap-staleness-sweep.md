# ROADMAP staleness sweep — 2026-08-03 (FULL)

*Read-only research pass. **No `ROADMAP.md`, design-doc, `stubs.md`, `corrections.md` or
`dependency-graph.md` edits were made — the integrator applies.** No cargo was run by this
agent. This file and its watermark key in `docs/audits/.sweep-watermarks.json` are the only
things written.*

**What was swept.** The live board — `ROADMAP.md` § In flight (`:20-355`) · § Sequenced
(`:357-2864`) · § Observed (`:2865-4506`) · the two live close blocks (GEO-3 `:4507-4597`,
BODIES `:4600-4690`) — plus `docs/dependency-graph.md`, against everything landed in
`git log d8407e1..HEAD` = **143 commits**: journals **0138–0150**, corrections **#88–#97**,
the **P11** arc (slices 2 and 3), **FS-A**, the **P2 measurement runs**, the **E4** design
pass + **E4-1** extraction, the **gravity** flip, the **gait bake** (headless + consumer +
walk), the **look-ownership** slice, the **erosion.rs** concern split, and the
**stratigraphic-correlation** design pass.

- **Watermark at start:** `d8407e1` (2026-08-02 sweep).
- **Swept to / every quotation below read at:** **`2985273`**.
- **Mode: FULL.** Both triggers in `.sweep-watermarks.json` § `_full_run_triggers` fired —
  143 commits (past the ~5-active-day threshold) **and** the board was restructured (a close
  block superseded, two new close blocks written, § Observed grew a new head).

**Prior-sweep carry-over, checked first so nothing is re-reported as new.** Of the
2026-08-02 sweep's eight findings: **F1 APPLIED** (`c546d19` — corrections #85/#86 → #88/#89,
stub #34 → #35), **F3 APPLIED** (`dependency-graph.md:79` now reads *"stubs #35 (renumbered
from a wrong #34→#33 pointer)"*), **F6 APPLIED** (`ROADMAP.md:1585-1586` now reads
*"~~build it~~ ✅ SHIPPED 2026-07-28"*), **F7 APPLIED** (zero live `journal/pending-p11-slice1`
references outside the two audits that reported them), **F8 APPLIED**
(`ROADMAP.md:435-437`). **F4 and F5 are recorded as deliberately owed** at
`ROADMAP.md:2949-2952`. **F2 was NOT applied and is not in that owed list — it is re-filed
below as F4, larger.**

---

## 0. THE RECALIBRATION SCAN — run before any entry was walked, per the skill

Five candidates in the delta could change what an older observation MEANS. Each is graded by
the skill's ⚠⚠ qualifier: **is it enabled in the config the observations were made under?**

| # | candidate | enabled by default? | verdict |
|---|---|---|---|
| **R1** | **gravity 25.0 → 9.81 m/s²** | **YES — no flag** | **VOIDS a narrow cohort.** See below. |
| **R2** | P11 slice 3 (per-column record membership + packed `DepUnit`) | YES | Appearance-class, already walked and rejected in-window. |
| **R3** | FS-A release spectra | **NO** | Voids nothing. |
| **R4** | E4-1 kernel extraction | YES, but byte-identical | Voids nothing by construction. |
| **R5** | P2 measurement runs | docs-only | Voids nothing about the world; recalibrates a *published bracket*. |

### R1 — gravity is the real one, and its blast radius is smaller than it looks

**It is on.** `crates/dc-core/src/world_constants.rs:85` — `pub const DEFAULT_GRAVITY_M_S2:
f64 = 9.81`, read (not restated) by `dc-client/src/player.rs:31`,
`dc-api/src/character.rs:68`, `dc-physics/src/world.rs:46`, `dc-client/src/body.rs:1072`.
There is **no flag**: the shipped client and every character have run at Earth gravity since
`bd0c82c` (2026-08-02 23:11). The board states the consequence itself at
`ROADMAP.md:423-427` — *"everything is **≈1.6× floatier in TIME** — longer arcs, slower
falls"* — and its jump-apex claim is arithmetically correct (`player.rs:125` launches at
`√(2gh)`, so the apex is `h` at any `g`).

**Cohort test, entry by entry — and the answer is that almost nothing moves.** Every live
body/motion observation on the board turns out to rest on a **kinematic or geometric**
quantity, which gravity does not touch:

- `ROADMAP.md:3185-3258` (journal/0130–0131, body plans): hip heights, a −56.2° knee, foot
  planting, `CROUCH_ROOT_DROP_M`, the clip-authored `root_bob_m` peak at +0.125 m. **All
  static geometry or authored clip data. Not voided.**
- `ROADMAP.md:3040-3055` (the 12 fps aliasing entry): its 6.97 / 7.73 / **4.29**
  frames-per-cycle numbers come from journal/0148, **2026-08-03 — after the flip**. Not
  voided.
- `ROADMAP.md:3087-3102` (the strafe), `:3143-3171` (`quantize_time`): orientation and frame
  arithmetic. Not voided.

**Exactly ONE live entry is in the voided cohort**, and it is a **user field report**, so it
stays open — see **CANNOT-DETERMINE #1**.

**Two things worth the integrator's attention, both cutting the other way:**

1. **The measuring instrument is gravity-independent, so the 2026-07-23 perf baseline audit
   is NOT voided.** `--perf-drop` does not fall — it *teleports*:
   `crates/dc-client/src/perf.rs:265-270`, `player.pos_m.y -= DROP_STEP_M` every
   `DROP_STEP_INTERVAL_S`, described at `docs/audits/2026-07-23-perf-baseline-vertical-drop.md:4-6`
   as *"the player teleport-steps −30 m every 0.5s"*. **Do not stamp that audit.**
2. **The recalibration's textbook damage already happened once, in-window, and a gate caught
   it.** `0e344ca`'s message: *"THE GAIT TESTS CARRIED A STALE `G_WORLD = 25.0` … printing
   Fr 0.9205 for a world that had already moved to Fr 2.3457"*, and *"A PRINTED CAPTION WENT
   FALSE within hours."* That is the recalibration trap in miniature — **numbers written
   hours before a default-on constant moved** — and it is the strongest available evidence
   that the cohort question was worth asking.

### R2 — P11 slice 3 is default-on and moved every golden family, but its appearance debt is already booked

`ROADMAP.md:593` records *"all-family golden move in that merge"*. This is **not** the
journal/0111 shape (no rate, gradient or yield was recalibrated), so **no literature-relative
claim moves**. It moves the *appearance* cohort — and unusually, the walk that would normally
be owed **already happened inside the window and returned a REJECT**
(`ROADMAP.md:2867-2886`, user live view 2026-08-03). So the appearance debt it created is
booked at the top of § Observed rather than latent. The prior sweep's *"two stacked
appearance changes"* framing (`docs/audits/2026-08-02-roadmap-staleness-sweep.md:54-56`) is
therefore **superseded, not merely aged**: the octaves + slice-1 stack was verdicted POSITIVE
at the 2026-08-02 walk (`ROADMAP.md:2959-2961`) and slice 3's change was verdicted NEGATIVE
at the 2026-08-03 live view.

### R3 — FS-A is off, exactly as `calibrated_rates` was

The grain writer was **withheld**: `crates/dc-worldgen/examples/release_spectrum_probe.rs:85`
— *"Every unit's grain is still `GRAIN_UNSET` (no writer exists)"*; the seam is marked at
`weather_inventory::grain_write_seam`; the board records the withholding and the user's
ratification of it at `ROADMAP.md:645-651` and `:4519-4522`. **Voids nothing.**

### R4 / R5 — and the third consecutive `calibrated_rates` receipt

`crates/dc-worldgen/src/deeptime/field.rs:374` still reads `calibrated_rates: false` inside
`production_config_base`; `EROSION_CALIBRATION: f64 = 45.0` at `field.rs:158`, and
`git log -S "EROSION_CALIBRATION: f64" -- crates` still returns **one** commit (`cdd1036`,
pre-watermark). **corrections #68's receipt holds a fourth time.** E4-1 is byte-identical
(*"zero goldens moved"*, `ROADMAP.md:4526-4527`).

**But R5 does recalibrate a published number, and the discipline held on one side only.**
The P2 measurement runs moved the multiplier bracket from `M ∈ [60, 240]` to `M ≈ 375–380`
— *"the bracket … is LOW by ~1.6×"*. The **derivation audit is properly banner-stamped**
(`docs/audits/2026-08-01-p2-calibration-derivation.md:20-30`) and the **graph row is stamped**
(`dependency-graph.md:70`). The **ROADMAP entry that asked for the work is not** — finding
**F4**.

---

## 1. FINDINGS, ranked by value

### F1 — 🔴 THREE things the user DECIDED or GREENLIT in this window have **no entry on the live board**. They exist only in `dependency-graph.md` and in a close block.

This is the run's headline and it is the skill's finding-type **6** (*an arc living only in a
close block*) fused with type **2** (*already DECIDED, board line never written*). A close
block is a **handoff that gets archived** — CLAUDE.md read-first item 5 says so in the
clause it added on 2026-07-29. The dependency graph is *"what blocks what"*, not the
sequence. **Neither is § Sequenced**, which is what a cold session reads for *what to do*.

| arc | the ruling / the shipped work | where it lives today | grep for it in § Sequenced |
|---|---|---|---|
| **E4 — field-solver primitives** | user: *"e4 yes. queue right away"* (`ROADMAP.md:4564`); design pass `docs/audits/2026-08-03-e4-implicit-kernel-design.md`; **E4-1 SHIPPED** (`62b0449`, journal/0150, stubs #30 discharged); **E4-2 / E4-3 open**; **`NEEDS RATIFICATION` outstanding on the `dc-core::field` venue** | `dependency-graph.md:50` + `ROADMAP.md:4525-4526`, `:4549-4552` | **zero** — the only `E4` tokens in the body are `:1135` (a passing mention inside the RATE entry) |
| **B7 — joint rotation limits** | **DECIDED 2026-08-02 (user)**; design pass `docs/audits/2026-08-02-joint-limits-b7-design.md`, *"all seven calls closed"*; five stubs land with the build; wants the pre-B3 window | `dependency-graph.md:105` + `ROADMAP.md:4662-4664` | **zero** |
| **B8 — individual proportion variation** | **DECIDED 2026-08-03 (user)** (`e913f1d`), *"SIZE FIRST … range=engine, distribution=pack, pack owns fairness"* | `dependency-graph.md:106` + `ROADMAP.md:4631-4632` | **zero** |

**Adjacent, and graded lower only because it *is* on the board:** the
**stratigraphic-correlation** arc — a user sketch, a user principle, a completed design pass
and **five pending user picks** — lives as a sub-bullet of an § Observed *field report*
(`ROADMAP.md:2901-2927`) and as close-block item 1 (`:4542-4548`). It is the next thing the
project builds; it has no § Sequenced entry. Per the board's own header (`:5-6`), *"only
diagnosed work gets Sequenced"* — this is diagnosed.

**Why this is ranked first.** E4 alone carries a live `NEEDS RATIFICATION` (`:4550`), a
shipped extraction, two unbuilt slices and a sequencing dependency on the P2 flip. B7 carries
seven closed design calls and five undelivered stubs. **All of that is one archive pass away
from being invisible**, and the GEO-3 close block will be superseded by the next wrap.

---

### F2 — 🔴 The gait bake (member #1) SHIPPED — headless, consumer, and a walked user verdict — and the arc's CONTINUATION SLOT still reads as an un-started docket.

**The entry:** `ROADMAP.md:715-720`, § Sequenced, *POSTURE AND GAIT ARE DERIVED* —
> *"(a) the **gait bake** (duty and cadence against the published Froude band) — **docket
> EXPANDED and user-ratified 2026-08-02**: `posture-gait.md` § 7 member 1's banner now
> carries the honest input inventory…"*

No shipped stamp anywhere in the item.

**The evidence, three landings and a user verdict:**
- **Headless bake** — journal/**0144** (`journal/0144-the-gait-had-to-ask-the-world-for-gravity.md`), merged `1ee38b4`.
- **Renderer consumer** — journal/**0147**, merges `40d1dc2` / `8d635df` (*"the renderer samples the derived gait"*).
- **Walked with the user** — journal/**0148**, `8421fff` (*"walk 0148: derived gait verdicted"*).
- **The verdicts, in the close block** (`ROADMAP.md:4607-4616`): *"they genuinely look pretty
  good… the cadence difference is very clear, i like it"*; *"the bob is gone… soles are
  planted"*; and **`Keyframe.root_bob_m` left the schema** — verified in code, the symbol
  survives only as retirement commentary (`crates/dc-api/src/bodies.rs:229`,
  `dc-client/src/body.rs:108`, `:1115`).
- The gate that confirmed it: `0e344ca`, *"the design pass's predicted table is CONFIRMED —
  measured vs predicted agrees to ~1e-5 or better on every row (stride, period, cadence,
  duty, theta_max, bob)"* — **member #0's falsifiable-acceptance bar met a second time.**

Same paragraph, same defect: `ROADMAP.md:739-741` still says stubs **#34** (clip role
binding) is *"decided at the gait-bake design pass"* in the future tense; the pass shipped
2026-08-02 and resolved it as *"one `BindTarget` across all four binders"*
(`ROADMAP.md:4753`).

---

### F3 — 🔴 The CoarseField arc still carries **"⚠ NOT SHIPPED — the near-path RECORD restructure"**. P11 slice 3 shipped exactly that, including the withdrawn MM-3 type.

**The entry:** `ROADMAP.md:1426-1433`, § Sequenced —
> *"**⚠ NOT SHIPPED — the near-path RECORD restructure (U3's 460 m half).** Blast radius
> **MEASURED at 13 files / ~40 sites** … **MM-3's type was written and deliberately
> WITHDRAWN** … it ships with its restructure or not at all (the proposal is in the slice
> report, **awaiting ratification**)."*

**The answering work, merged `cd1058a` 2026-08-03, and the same document already records it
430 lines earlier** (`ROADMAP.md:546-558`): *"✅ SHIPPED 2026-08-03 (journal/0145) …
Per-voxel-column record membership (`SubCell` + `cell_bundle`, the F2 mass-coupled trio) +
packed `DepUnit` L-8 … **The ~460 m near tile is dead in code**"*. In the tree:
`crates/dc-worldgen/src/subcell.rs` (MM-3's type, with a production caller),
`crates/dc-worldgen/src/collapse.rs:1611-1646` (*"One `run_strata` per touched cell"*),
`crates/dc-worldgen/src/deeptime/field.rs:915` (`cell_bundle`). The blast radius was
**re-measured at 14 files / ~71 sites** (`ROADMAP.md:561-562`), so the entry's *13 files /
~40 sites* is also stale.

**The dependency graph already knows, in exactly the terms this entry set as its
condition.** `docs/dependency-graph.md:72` (row P4): *"Its **~460 m record half SHIPPED
2026-08-02 (P11 slice 3a)** … **MM-3's `SubCell` shipped WITH its consumer** exactly per
journal/0129's withdrawal condition … blast radius re-measured 14 files / ~71 sites — the
count had grown since 0129's 13/~40."* So the answer is written, once, in the graph — and
the § Sequenced entry that stated the condition was never stamped. Same shape as F4 and F11.

**Two dependent clauses in the same entry that move with it:**
- `:1366-1374` — *"THE NEAR PATH IS NOT SUFFICIENT ON ITS OWN … Residue: a confirm re-shoot
  at U3's reference pose rides the next appearance walk."* The re-shoot rode the 2026-08-02
  walk and the user's verdict is recorded at `:2959-2961`.
- `:1442-1450` — *"(a) `dithered_member`'s **formation context is still the chunk's**"*.
  **HALF TRUE at HEAD and I checked it rather than assuming:** `collapse.rs:1628-1634` still
  passes `temp_c`, `precip`, `elev_m`, `flow_energy` computed **once per chunk**, so the
  *climate* half survives slice 3 intact; the *record / regolith / weathering* half is now
  per touched cell (`:1620-1631`). **Do not strike this bullet — narrow it.**

---

### F4 — 🔴 REPEAT, and the debt doubled. P2's ROADMAP entry is now **two** landings behind, plus a third that dissolves one of its blockers.

**Second consecutive sweep for this finding.** The prior run filed it as F2
(`docs/audits/2026-08-02-roadmap-staleness-sweep.md:137-183`); it was not applied and is not
in the owed list at `ROADMAP.md:2944-2956`.

**The asker, unchanged and still reading as un-started —** `ROADMAP.md:1248-1272`,
*"🔴 RE-PICK `EROSION_CALIBRATION` AGAINST THE FIXED OPERATOR"*, whose § *How to pick it*
(`:1256-1261`) still says only *"Against the **published literature** … The target is the
craton denudation band journal/0111 established."*

**What has since landed and is recorded nowhere in that entry:**

| landed | where it is recorded | what the entry still says |
|---|---|---|
| **The literature derivation** (2026-08-01): target derived twice, 2.63 ↔ 2.653, and the **user RULED the referent** (*"the PRESENT-STATE band governs"*) | `docs/audits/2026-08-01-p2-calibration-derivation.md:5-18`; `dependency-graph.md:70` | *"the target is the craton denudation band"* — as an open instruction |
| **The measurement runs** (2026-08-02): D3(M) ≈ 0.0070·M, **M ≈ 375–380**, bracket low 1.6×, `uplift_scale` NOT triggered, § 4.5's pair-space collapse REFUTED, **three named flip blockers** | `docs/audits/2026-08-02-p2-measurement-runs.md`; audit banner `:20-30`; `dependency-graph.md:70` | nothing |
| **E4's finding** that the gen-time blocker is pure explicit-stability CFL tax, i.e. *"mostly dissolved by E4"* | `ROADMAP.md:4555`; `dependency-graph.md:50` | nothing |

**And one of the entry's own clauses is now known to CONFLICT.** `:1268-1271` says
*"**Acceptance** pairs an aggregate with a neighbour-relative measure … and re-asserts the
three pit guards."* The measurement runs found *"every in-band rung violates hollows > 10 m
= 0; 8,878 @400"* (`dependency-graph.md:70`) — i.e. **the acceptance criterion as written
cannot be met at any in-band multiplier**, and that conflict is one of the three user calls
the flip is blocked on. The entry presents it as a settled acceptance recipe.

*This is the ROADMAP-body variant of the one-directional pointer that
`ROADMAP.md:838-843` already names as **having no mechanism yet**. Two sweeps have now
found the same entry.*

---

### F5 — 🟠 The tour map ANSWERED the entry's *"Owed before any game time"* clause with a proven NULL, and said so in its own text. The entry was never stamped.

**The entry:** `ROADMAP.md:3367-3372` —
> *"**Owed before any game time: a tour map that finds a chunk whose surface top span is
> `Single` in a two-member class.** Nothing has ever searched for that, and it is the only
> station where this fix is visible."*

**The answer, 2026-08-02** — `docs/audits/2026-08-02-appearance-tour-p11.md:47-56`, verbatim:
> *"**This settles the standing ROADMAP § Observed question** … That chunk **does not exist
> on the shipped world.** journal/0129's observation at the U3 reference pose … **was not an
> unlucky pose. It is the universal case, at 99.1 %.**"*

With a positive control (`:57-60`): the identical classifier finds **3,526 of 4,792 land
columns (73.6 %)** carrying a movable `Single` span *below* the surface — so the null is a
measurement, not a blind instrument.

**The result is on the board — as a *different* entry** (`ROADMAP.md:2996-3007`) 370 lines
above the question it answers. **The audit points at the entry; the entry does not point
back.** Textbook read-first item 5.

**And it is not one unstamped site but three.** The same *owed tour map* clause is restated,
still open, in the dependency graph — the document CLAUDE.md read-first item 1c sends every
cold session to:
- `docs/dependency-graph.md:72` (row **P4**), closing verbatim: *"**Owed: a tour map that
  finds a `Single` top span in a two-member class**."*
- `docs/dependency-graph.md:51` (row **E5**): *"'member stepping dominates U3' is unsettled
  and needs a ***new station***, not the recorded pose."*

The station was searched for, world-wide, and **proven not to exist**. Three asking sites,
zero banners.

⚠ **Scope, so the integrator does not over-apply this:** it discharges the *"owed tour map"*
clause and confirms the *inertness* half. It does **not** touch the `⚠ CONTESTS A POSSIBLY
USER-ORIGINATED CLAIM` banner at `:3337-3345`, which forbids an implementation-side
resolution of whether member stepping was ever U3's *dominant* signal.

---

### F6 — 🟠 `SALT_DT_PERTURB` was converted — byte-identically, with the proof test — by P11 slice 2. The entry still says **QUEUED**. And its stated success condition is false at HEAD.

**The entry:** `ROADMAP.md:4358-4365` —
> *"Preferred fix: **convert, don't document** — one call site, byte-identical, and
> `refine.rs` then holds **zero hand-rolled salts**. **QUEUED behind** the in-flight member-#0
> build pair's merge."*

**Done 2026-08-02.** `crates/dc-worldgen/src/draws.rs:205` — `DeepTimePerturb = 0x5900_0002`;
`draws.rs:187-191` records that it *"was spelled by hand as `SALT_DT_PERTURB`"*;
`draws.rs:691-707` is the byte-identity proof (`assert_eq!(<DeepTimePerturb as
Domain>::SALT, HAND_ROLLED)`); the call site is converted at `refine.rs:172`
(`Draws::of::<DeepTimePerturb>(seed)`) with the history at `:178-179`. Shipped in the same
slice that closed the salt **collision** the 2026-08-02 spine-audit found
(`ROADMAP.md:536-538`).

**⚠ But the entry's success condition did not come true, and nobody has said so.**
`refine.rs:152` still reads `super::grid::SALT_DT_ROUGH`, declared at
`crates/dc-worldgen/src/deeptime/grid.rs:34` as a bare `pub(crate) const … = 0x5900_0001`.
**`refine.rs` does not hold zero hand-rolled salts** — it holds one, spelled in another
file, which is exactly the enumeration failure the entry was written to retire. The sibling
entry at `:4354-4356` (`tectonics.rs` salt conversion) is **still genuinely owed**, though
its address moved: the file is now `crates/dc-worldgen/src/deeptime/tectonics.rs:51-53`
(`SALT_TEC_POS` / `_VEL` / `_CRUST`).

---

### F7 — 🟠 The gravity flip's `⚠ UNGATED` flag is stale. The gate ran twice.

**The entry:** `ROADMAP.md:394-399` — *"✅ LANDED 2026-08-02 (`bd0c82c`, renamed `26bf42f`)
— **⚠ UNGATED, the gate is the first thing owed.** … **Gate owed on dc-core / dc-api /
dc-client / dc-physics** — blocked at the time by the sibling session's 69-minute P2 sweep."*

**It was gated the same day.** `0e344ca` — *"gate the gait slice + the gravity flip: FULL
WORKSPACE GREEN … **962 passed / 0 failed / 93 suites, exit 0** … Both previously-UNGATED
commits (the gravity flip, the harvested gait slice) are now **verified rather than merely
committed**"* — and the flip is inside the wrap-trio green recorded at `ROADMAP.md:4535-4539`
(**984 / 0 / 94 suites**, after `cargo clean -p` of all six touched crates). The dependent
Observed entry that blamed the `[UNGATED]` commits is already stamped resolved
(`ROADMAP.md:2929-2932`); only the Sequenced entry that owns the debt still carries it.

---

### F8 — 🟠 The `"vanilla"` strike debt is **accelerating**, and this window put the struck word into a source **filename**.

**The entry:** `ROADMAP.md:3260-3268` — user ruling, with the census *"residual **220**:
crates **145** … docs **37** … ROADMAP+history **8**"*. The prior sweep measured crates
162 → 177 and the growth is recorded as owed at `:2950-2952`.

**Measured at HEAD vs the watermark, `git grep -ioc vanilla <rev> -- <path>`, same method
both sides:**

| | `72fbe86` (2 sweeps ago) | `d8407e1` | **`2985273`** |
|---|---:|---:|---:|
| `crates/` | 162 | 177 | **215** (+38) |
| `docs/` | — | 56 | **89** (+33) |
| `ROADMAP.md` + history | — | — | **20** |

**The escalation, and it is qualitatively worse than a count:** FS-A created
**`crates/dc-core/src/materials/release_vanilla.rs`** — the struck word is now a **file
path**, which every future doc, `mod` declaration and citation will reproduce verbatim. The
prior sweep flagged the same shape one tier down (the stub *slug*
`the-deep-tiers-content-set-is-hard-wired-to-vanilla`); this is the same act at the level the
compiler enforces. Heaviest files at HEAD: `dc-core/src/materials/geology.rs` (21),
`dc-api/src/classes.rs` (21), `dc-worldgen/tests/geology.rs` (16),
`dc-worldgen/src/pipeline.rs` (15), `dc-core/src/materials/release.rs` (10).

The entry's own trigger (*"once the worldgen agents land"*) fired two sweeps ago. Its census
numbers are now wrong by **+48 %** (crates) and **+140 %** (docs).

---

### F9 — 🟠 The `journal/pending-*` dangling-pointer shape **recurred within one day** of the last sweep's F7 being applied.

F7's three `pending-p11-slice1` references were repaired. The **same convention produced
three new dangling pointers in the same window**, one of them in the live board:

| site | reads | resolves to |
|---|---|---|
| **`ROADMAP.md:580`** | *"✅ BUILT 2026-08-02/03 (slice-3 worktree, **journal/pending-p11-slice3**; audit header carries the banner)"* | `journal/0145-the-column-stopped-reading-its-neighbours-mail.md` |
| `docs/audits/2026-08-02-p11-slice3-design.md:48` | *"BUILT 2026-08-02/03 (the slice-3 worktree; **journal/pending-p11-slice3**)"* | same |
| `docs/audits/2026-08-01-members-into-history-design.md:73` | *"SLICE 2 (THE CONVERSION) LANDED 2026-08-02 — **journal/pending-p11-slice2b**"* | `journal/0141-the-load-stopped-being-seven-things.md` |

The convention itself is documented and endorsed at
`.claude/skills/session-workflow/SKILL.md:1149` (*"proven that day"*). **What has no
mechanism is the resolution step at merge** — and this is now the second consecutive sweep to
find it, with the count going 3 → 3. Cross-ref: the Observed parent at `ROADMAP.md:3439`,
*"THE GATE CANNOT SEE BROKEN DOC LINKS"*.

---

### F10 — 🟡 Two bullets of the body-plans Observed entry describe machinery that no longer exists.

`ROADMAP.md:3185-3258` is a dated measurement record (journal/0130–0131) and its *testimony*
is rightly immutable. Two of its bullets, however, are written as **live claims about the
tree**, and both are false at HEAD:

- **`:3235-3251` — *"🔴 THE SECOND WALL … THE STOP-MOTION IDENTITY AND PLANTED FEET ARE IN
  STRUCTURAL CONFLICT. `ROT_QUANTUM_RAD = TAU/32 = 11.25°` (`dc-client/src/body.rs:35`)"***,
  closing *"This is a **fork nothing in the corpus has posed**."* **`grep -rn
  ROT_QUANTUM_RAD crates/` returns ZERO at `2985273`** — the rotation quantizer was removed
  (journal/0133; the removal is referenced by name at `ROADMAP.md:3053`), and
  `dc-client/src/body.rs:35` now documents the retirement instead. The fork was not just
  posed; it was resolved.
- **`:3194-3198` — *"⚠ THE HOVER IS NOT FIXED"***, whose named cause is *"the clips'
  `root_bob_m` surviving `ROT_QUANTUM_RAD`"*. Both halves are gone:
  `Keyframe.root_bob_m` **left the schema** (`crates/dc-api/src/bodies.rs:229`), and the
  close block records the user's verdict at the 0148 walk — *"the bob is gone… soles are
  planted"*, *"corrections #80's two-authority hover is now **unrepresentable rather than
  fixed**"* (`ROADMAP.md:4609-4615`).

*Graded 🟡 only because the entry's surviving purpose (the A-1 four-constants finding, the
2:1 window proof, the two-gates finding) is untouched. The fix is two banners, not a
strike.*

---

### F11 — 🟡 The RATE entry's S-10 clause reads as an open engine hole. `stubs.md` § 30 is stamped **FULLY DISCHARGED** — the pointer runs one way, in the unusual direction.

`ROADMAP.md:1119-1141` — *"Stability substepping now belongs to the **S-10 field-solver
primitive** (`spines.md` § S-10; **`stubs.md` § 30's heir re-pointed there**)"* … *"**WHY IT
NOW LEADS.** … **Every future field pass that diffuses anything has the same trap and no
defence.** That is an engine-shaped hole."*

**The hole was filled 2026-08-03.** `docs/design/stubs.md:1297-1306`:
> *"**🟢 FULLY DISCHARGED 2026-08-03 (E4-1, byte-identical extraction; ⚠ NEEDS RATIFICATION
> on venue, audit U-3).** … `dc_core::field::FieldKernel::plan` derives `n = ceil(max_cell
> coeff / MONOTONE_MAX_EDGE_COEFF)` **inside the engine** … an out-of-bound step is
> **inexpressible from a pass** … **No future pass author writes the von Neumann analysis —
> which was the whole of this entry's blast radius.**"*

**Note which way this one points.** The obligation `stubs.md` owed — stamp the target — was
**honoured**; what failed is the *ROADMAP body entry that cited the stub*. Same asymmetry the
board already names at `ROADMAP.md:838-843`: *"the obligation holds where the target is a
spike or journal, and **FAILS where the target is a ROADMAP body entry**."* Third
independent instance in two sweeps (this, F4, F5).

---

### F12 — 🟡 Three live citations point at `collapse.rs:1409`, an address whose code slice 3 deleted — and the corrected address filed one day earlier is stale too.

`ROADMAP.md:1485` (*"U3 / near field at `collapse.rs:1409` — one point-sample per chunk
shared across all 1024 columns"*), `:2268`, `:3420`. At HEAD the near record read is
`DeepField::cell_bundle` per touched cell — `crates/dc-worldgen/src/collapse.rs:1611-1646`,
`:1623`; the *one point-sample per chunk* claim is what slice 3 exists to have killed
(`ROADMAP.md:556-558`, *"the ~460 m near tile is dead in code"*).

The 2026-08-02 walk entry already re-addressed this once, to `collapse.rs:1483-1487`
(`ROADMAP.md:2971-2972`) — **and that correction is itself now one slice out of date.** This
is corrections #67's shape (*addresses rot; the finding does not*) firing twice on one line
inside 24 hours; the load-bearing note for the integrator is **cite the mechanism, not the
line**.

---

### F13 — 🟡 The 2026-08-02 walk field report's open half was answered by the user's own live view the next day.

`ROADMAP.md:2962-2981` — *"**(b) OPEN** — 'there are larger regional borders still… a
straight line where mudstone is in the mix on one side, and not on the other'"*, closing
*"**This observation RESOLVES into slice 3's scope and its acceptance criterion** — the
mudstone-mix border at a cell edge must stop being a straight line."*

**Slice 3 shipped, and the user looked again.** `ROADMAP.md:2870-2874` (live view,
2026-08-03): *"sharp differences between regions, **they just don't follow the cardinal
directions**"*. **The straight-line criterion is met; the verdict is REJECT on other
grounds** (the full-depth transplant, `:2875-2879`).

**This is a user field report, so I am not marking it resolved — I am naming the evidence
that the world changed** (a second user observation, not reasoning) and leaving the
disposition to the integrator. The successor entry already sits 90 lines above it.

---

## 2. CANNOT-DETERMINE — flagged, not reconciled

1. **`ROADMAP.md:3774-3782`, *"Perf: throughput ceiling at terminal velocity"* (USER FIELD
   REPORT, 2026-07-23) — the one entry R1 puts in the voided cohort, and it STAYS OPEN.**
   It was observed by *falling*, and the fall integrator has since changed by **2.55×**
   (`player.rs:122`, `GRAVITY_M_S2` now 9.81). There is **no terminal-velocity clamp**
   anywhere in `player.rs` — I grepped — so descent speed is `√(2gh)` and the same drop now
   peaks at **0.626×** the old rate over **1.596×** the time. The claim *"at max fall speed
   the streaming pipeline still saturates"* is a statement about a demand that has moved,
   against a supply that has not. **I cannot say whether it still chokes**, and per hard rule
   3 nothing but a measurement may say so. **The cheap falsifier:** re-run the drop and
   record which gravity it ran at. *(And note the sibling: the `--perf-drop` instrument is a
   teleport and is unaffected, so any re-measure must be a live fall, not the probe.)*
2. **Whether the export decomposition at `ROADMAP.md:1309-1311`** (96.0 % creep · 3.7 % wave
   · 0.3 % eolian · 0.02 % fluvial) survives the fixed creep operator (journal/0122) and the
   member-grade record (P11). It was measured on the pre-fix operator. Nothing in this window
   re-measured it; no cargo was permitted this run.
3. **corrections #39's `NEEDS RATIFICATION` age.** `ROADMAP.md:3410` says *"#39 has sat
   `NEEDS RATIFICATION` for 7 days"*; written 2026-07-29, that is now **13 days** (and the
   two In-flight flags at `:188` / `:202` are the same age). Third consecutive sweep
   reporting the ageing note as the stale part. Nothing in this window ruled on it.
4. **Whether the *"tour-map instrument needs a LAND FILTER"* entry (`ROADMAP.md:4374-4378`)
   is discharged.** `examples/appearance_tour_p11.rs` **has** one
   (`docs/audits/2026-08-02-appearance-tour-p11.md:12-14`, *"near-field `surface_elev_m > 2
   m`"*), but the entry names the instrument generally and I did not read `tour_map.rs` /
   `tour_map_0071.rs`. Partial at best.
5. **The `"4 of 10 classes"` member census** (`ROADMAP.md:3350-3357`, `:445-446`, restated at
   `dependency-graph.md:51` and `:72`) — carried forward unresolved from the prior sweep. Not
   re-run; no cargo. If a member was registered since journal/0129, the number moves.
6. **Golden and gate state** — taken on the close block's word (`ROADMAP.md:4535-4539`,
   *984 / 0*). Not verified by execution, by brief.

---

## 3. What this sweep did NOT cover

*A null over these is not a result.*

- **`ROADMAP-history.md`** — not opened. The sweep's question is about *live* entries.
- **The superseded 2026-08-02 BODIES close block** (`ROADMAP.md:4693-4776`) — read for
  cross-references only. It is marked SUPERSEDED and belongs in `ROADMAP-history.md` per the
  archive-by-status rule; that is an **archive candidate, not a staleness finding**.
- **`journal/corrections.md` #1–#87** — only **#88–#97** (the delta) were read in full.
- **§ Observed entries older than ~2026-07-25** were read by heading and opened selectively;
  the 2026-07-29 FULL sweep and the 2026-07-28 baseline covered that ground exhaustively.
  Budget went to the delta's blast radius.
- **`docs/design/*.md` bodies** — consulted only where a finding pointed into them
  (`stubs.md` §§ 30/34, `posture-gait.md` by citation). This sweep is docs-vs-**what
  happened**; docs-vs-each-other is `doc-topology`'s, which is **also overdue-FULL**
  (`ROADMAP.md:4509-4511`, `:4557`).
- **No cargo** — every code claim above is a read or a grep at `2985273`, cited by
  `file:line`.

---

## 4. Meta — one shape, stated once, because four findings share it

**Six of the thirteen findings — F2, F3, F4, F5, F7 and F11 — are one failure: the answer was
written somewhere authoritative, and the ROADMAP body entry that asked the question was not
stamped.** The corpus's same-commit banner obligation (CLAUDE.md read-first item 5) is
**working** where the target is a spike, an audit or a stub — `stubs.md` § 30, the P2
derivation header and the appearance tour map all carry correct, dated banners pointing
forward, and in F3's case the dependency graph even restates the discharged condition in the
entry's own words. It fails, every time, where the target is a **ROADMAP body entry**. The
board itself diagnosed this on 2026-07-29 (`ROADMAP.md:838-843`) and closed with *"the
ROADMAP-body variant **has no mechanism yet**"*. Two sweeps later it still has none, and this
run found five fresh instances plus one repeat (F4, second consecutive).

**A second, adjacent shape (F1):** the dependency graph is absorbing rulings that never reach
§ Sequenced. That is not a defect *of* the graph — the graph is doing its job, and doing it
well (rows E4/B7/B8/P10/P11 are current and detailed). It is that **the graph plus a close
block now feels like enough**, and a close block gets archived.

Recorded as observations for the integrator, **not filed as proposals** — the sweeper does not
adjudicate what it finds, and both remedies are process changes, which are the user's.
