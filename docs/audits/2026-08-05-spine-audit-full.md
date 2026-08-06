# spine-audit — FULL — 2026-08-05

**Base commit: `08afb88`** (worktree `agent-a70c8ac2133f494cc`; `git merge main` = "Already up to
date"; `git log -1 --oneline main` = `08afb88`, matches tip). **Everything below was read at
`08afb88`** unless a line says otherwise — per the skill's *"cite the commit you read it at"* rule
(corrections #67).

**Mode: FULL.** 156 commits since the `2985273` watermark (`git log --oneline 2985273..HEAD | wc -l`)
**AND `docs/spines.md` itself changed in the window** — the watermark file's own
`_full_run_triggers` rule (`"spine-audit": "any edit to docs/spines.md — every prior verdict was
made against a different rule"`) voids every prior verdict. Nothing was inherited; every § 3 row was
re-searched at source.

**Write-set (dispatching brief):** this file + `docs/audits/.sweep-watermarks.json`, same commit.
`docs/spines.md` was **NOT** edited — the brief overrides the skill's default write-set, so every
finding here is REPORTED, none APPLIED. The main session dispositions and folds.

**Delta shape:** 46 files / +7,819 −444 in `crates/`. The two arcs are **GEO-5** (the deposition
clock, S1's stratigraphic-correlation kernel, the E4 rulings) and **BODIES** (B6-a mass, B7 joint
limits, B8, the anim-rate ruling, the longleg swing probe).

---

## 0. The prescribed control — `scripts/standin_locus_check.py`

Run at `08afb88`. **Not clean: 14 markers, 1 with no resolvable locus pointer.**

```
crates/dc-worldgen/src/deeptime/recorder.rs:832
    /// ⚠ **STAND-IN — chapter and epoch DISAGREE after an overprint, on
```

**This is a true positive of the one-directional-pointer class, and the sharpest kind:** the marker
*asserts its own locus exists* — its last sentence reads *"Until then this comment is the marker;
**the stubs entry** names the blast radius"* (`recorder.rs:843`) — and never names it. The entry
**does** exist: `docs/design/stubs.md:1978`, **#52 `an-epoch-that-alteration-cannot-move`**, added
2026-08-04 by the deposition-clock slice, whose own body cites `set_tag_and_chapter` in
`recorder.rs` (`stubs.md:1984`). So the doc→code edge is written and the code→doc edge is not, which
is exactly CLAUDE.md read-first item 5's failure mode committed by the slice that filed the stub.

**PRESCRIPTION (a prescription, not evidence):** write `stubs.md #52` into the marker at
`recorder.rs:843`. One token; no behaviour changes. Under the skill's default write-set this is
apply-tier (a doc-comment correction); under **this** brief's write-set it is reported.

---

## 1. § 3 "Built, and nothing calls it" — every standing row re-verified

**Count, mechanically, at `08afb88`:**

```
awk '/^# 3\. Built, and nothing calls it/,/^\*\*Departed/' docs/spines.md | grep -c '^| '
→ 22
```

22 − 1 header − 1 `✅ DISCHARGED` row (resting-posture bake) = **20 standing**, which is exactly what
the section states. **The `2985273` sweep's caveat about this command reproduced correctly** — first
time in four sweeps the stated number and the mechanical one agree once the documented correction is
applied. No adjustment owed.

**Verdict: 20 of 20 re-confirmed uncalled in production. NO ROW EMPTIED. ONE ROW OWED (§ 2 below).**

Each search is `grep -rn "<pattern>" --include=*.rs crates/` at `08afb88`, defining file excluded
unless noted.

| row | search | result |
|---|---|---|
| `fits_in_pores` / `K_PORE` | `fits_in_pores\|K_PORE` | still zero call sites outside `packing.rs`. The only external hit is the same doc-comment cross-reference at `dc-core/src/materials/geology.rs:731`. **Unchanged since `d8407e1`** |
| `recv`/`area`/`lake` | `\.recv\b\|\.lake\b` | only `tests/deep_config_plumbing.rs:60,62,222,224` and `tests/flux_record.rs:361,418,419`. Still test-only; the A-1 upgrade stands |
| `DeepField::flux` | `\.flux\b` | only `tests/flux_record.rs` (many) — no `src/` reader outside `deeptime/`. Still ON-PURPOSE unconsumed |
| `DeepField::head` | `\.head\b` | only `tests/artifact_tripwires.rs:501-505` + `tests/head_field.rs`. Still uncalled |
| `exhum` / `t_crust` | `\.exhum\|\.t_crust` | only `tests/deep_config_plumbing.rs:63-64,84-85,133,225-226` and `tests/flux_record.rs:415-416`. Still uncalled |
| `bound_eighths` / `is_occupancy_solid` | `bound_eighths\|is_occupancy_solid` | **zero call sites of any kind.** Only `contents.rs`'s own doc-comment cross-references at `:38`, `:250`, `:252`, `:276`, and the two definitions `:286` / `:307`. Citations **unmoved** |
| R/H views | `derive_regolith_at\|derive_bedrock_at\|derived_regolith_m\|derived_structure_stock_m` | only `tests/rh_unification.rs:59,86` and `inventory.rs:2259-2260` (own suite). **Citation drift: `field.rs:978,993` → `:982,997`; `inventory.rs:1573,1585` unmoved** |
| `DeepField::geotherm` | `\.geotherm` | only `tests/artifact_tripwires.rs:481-487`, `tests/geotherm.rs`, `tests/flux_record.rs:417`. Still uncalled |
| `DeepField::chapters` | `\.chapters` | only `tests/deep_config_plumbing.rs:65,87-88,109,126,227`, `tests/artifact_tripwires.rs:518-558`, `tests/providers_common/mod.rs`. Still uncalled |
| `Agent::Dissolution` + `.dissolution` | `Agent::Dissolution` | only `tests/erodibility.rs:403-404`, `examples/entry_species_probe.rs:325`, `lithology.rs`'s own `:150,160,218,1098` and `dc-core/src/materials/mod.rs:323` (a doc reference). Still zero production |
| the S11 water module | `dc_worldgen::water` | unchanged — no `src/` reader outside the module |
| pass-graph `Resource` vocabulary | (`pipeline.rs`) | unchanged, 7 passes |
| `column_summary` / `open_air_below` / `ColumnSummaries` | `column_summary\|open_air_below\|ColumnSummaries` | only `dc-core/src/column.rs`'s own suite, the `lib.rs:37` re-export, and `dc-client/src/bench_storage.rs:19,201,220` (the `--bench-storage` harness). Still no dc-api handler |
| the S2 statistical tier | `ToyWorld\|force_fact\|statistical::` | **zero production callers workspace-wide.** Only `dc-sim/tests/s2_torture.rs`, `dc-sim/tests/s2_measurements.rs`, and the module's own suites. `dc-worldgen` still touches only the sibling `rng` (`examples/appearance_tour_p11.rs:48`). **State 3 (held as a candidate) — unchanged, and this row still does not want a consumer found for it** |
| `CoarseField` — `sample`, `summarize` | `summarize` / `\.sample(` | **`summarize` has ZERO hits anywhere in `crates/` outside `dc-core/src/coarse.rs`** (the ten hits the search returns are all the English word in unrelated comments). `sample` only inside `coarse.rs`'s own tests `:798,815-825`. Both still uncalled; the row stays PARTIAL |
| `Schedule::Seed` / `SeedAndStep` | `Schedule::Seed\|SeedAndStep` | still no production declarer. Only `schedule.rs`'s own suite `:216-285`, `runner.rs:1112` (doc), `mod.rs:297` (a comment stating the absence). **State 3** |
| the RATE authoring path | `CadenceTable\|sub_turned\|integrated_dt\|unmatched` | still zero production authorers. Non-empty tables only at `tests/rate_axis.rs:105,109`, `tests/schedule_axis.rs:109-110`, `runner.rs:1778-1779,1826`, `cadence.rs:314-316`. **`unmatched` still has no caller at all outside `runner.rs:1809` (a test) and `cadence.rs:323-324`.** Citation drift: `cadence.rs` `:206/:218/:251/:120` — `:251` confirmed, `:120` confirmed; `runner.rs:674` doc comment **unmoved**. **State 3** |
| the deep tier's CONTENT DOOR | `run_cells_with_geology\|build_field_cfg_cadence_geology` | still no production caller. Only the two default-path wrappers (`field.rs:654,667`, `mod.rs:204`) passing `&vanilla()`. Citations: `field.rs:661`, `mod.rs:231`. **State 1, unchanged** |
| the GRAIN-GRADE SPLIT feed | `set_grain` / `grain_write_seam` | **`set_grain` still has exactly one caller and it is its own test** (`recorder.rs:1418`). The seam is still passed only by tests/probes. **⚠ Every citation in this row has drifted:** `GRAIN_UNSET` `:536`→**`:561`**, `grain()` `:695`→**`:781`**, `set_grain` `:703`→**`:789`**, the test caller `:1254`→**`:1418`**, the state assertion `:1337`→**`:1506`**, the bit table `:458`→**`:459`**. The `weather_inventory.rs:116` seam and `inventory.rs:1738` feed are **unmoved** |
| the PER-INSTANCE GAIT SEAM | `InstanceDelta\|gait::pose` | still no production caller — only the `bodies.rs` re-export and `gait/tests.rs`. **State 3, unchanged.** *(Its sibling `bake_gait` remains consumed; and B7's `derive_joint_limits`, built in this window, is fully consumed — see § 4.)* |
| `RESTING-POSTURE BAKE` (discharged) | — | still discharged, kept as testimony. Not a standing row |

**§ 3 § *Departed*: no new departures.** Nothing was consumed this window.

---

## 2. NEW-INSTANCE — § 3 owes one row: **the correlation chain**

`deeptime::correlate` — the S1 stratigraphic-correlation kernel (578 lines, shipped 2026-08-04,
journal/0157) — **is built, tested, exported, and has exactly one reader in the entire tree, which
is its own integration test.**

**The search proving the absence:** `grep -rn "correlate" --include=*.rs crates/` at `08afb88`,
excluding `crates/dc-worldgen/src/deeptime/correlate.rs` itself → **34 hits, of which exactly ONE is
a use of the module**: `crates/dc-worldgen/tests/correlation.rs:19`
(`use dc_worldgen::deeptime::correlate::{…}`). Every other hit is the English word *correlated* in an
unrelated comment (`coarse.rs:1038`, `draws.rs:782`, `rng.rs:34`, the probes). **Note the near-miss:
`examples/correlation_m0_probe.rs` does NOT import the module** — it says *"correlated"* in prose
only. So the module has **zero production callers, zero example callers, one test caller.**

- **Where:** `crates/dc-worldgen/src/deeptime/correlate.rs`; exported `deeptime/mod.rs:28`
  (`pub mod correlate;`). Public surface: `Bed` `:129`, `epochs_non_decreasing` `:152`, `Borehole`
  `:159`, `boreholes_of` `:217`, `bilinear_weights` `:236`, `EpochPartition` `:253`,
  `ColumnInterval` `:512`, `ColumnIntervals` `:535`, `unclocked_events` `:576`.
- **It declares its own hole, verbatim** (`correlate.rs:100-103`), under a heading
  *"What this module deliberately does not do"*:
  > **No wiring.** Nothing calls it yet; `column()` and `collapse.rs`'s call path are untouched.
  > **S2** owns the wiring, the `record_for` semantics change, and the `cell_of` /
  > `NearRecordMembership` retirement.

  **And the module cites § 3 by name three lines further down** (`:114`, arguing that building the
  `discontinuity` seam now *"would be machinery nothing calls (spines § 3)"*) — while being, itself,
  machinery nothing calls and unlisted in § 3.

- **The feeding half is in the same state and reaches production writes.** `StrataEvent::epoch_bottom`
  / `epoch_top` (`geology.rs:130`, `:134`), the deposition clock's carried axis, are **written on the
  production path** (the event funnel, `geology.rs:805`, `:825-826`) and their doc comment says
  **"CARRIED, never READ … the consumer is stratigraphic correlation and the voxel-explainability
  WHEN axis, downstream. A test pins the blindness."** Search `grep -rn "epoch_bottom|epoch_top"
  --include=*.rs crates/` → the only *reader* outside constructors and tests is
  **`correlate.rs:187`**, i.e. the unconsumed module. So the pair is a two-link chain joined to each
  other and to nothing else — the `grain_write_seam` ↔ `set_grain` shape, except here the two halves
  *are* joined and the chain as a whole terminates in air.

- **Which state?** I read it as **state 3, HELD AS A CANDIDATE, not state 1** — the absence is
  *ruled*, not owed: S2 owns the wiring and is a sequenced slice, and the module's `discontinuity`
  paragraph shows the author reasoning about exactly this. It is owed a consumer **the day S2 lands**,
  and the row exists so that day does not arrive with the kernel forgotten. *(Which state it is filed
  in is the main session's call; § 4's rule binds the auditor.)*

- **Not in this row, checked and rejected** (recorded so the next sweep does not re-open them):
  - `dc-client/src/swing_probe.rs` (714 lines, new) — declared **`#[cfg(test)] mod swing_probe;`**
    at `main.rs:74`, with three `#[test]`s of its own at `:607`, `:668`, `:695`. It is an
    instrument with a current consumer (the `flow_cost_probe` / `GaitVector::at` category), not the
    `chapters` category. Its `main.rs` doc comment states the reasoning: `dc-client` is a binary
    crate with no lib target so the `examples/… test = true` shape cannot reach `body.rs`.
  - `dc-api/src/bodies/limits.rs` (B7) — **consumed.** `derive_joint_limits` reaches production at
    `dc-client/src/body.rs:455`, `:623`, `dc-client/src/character.rs:520`, and `bodies.rs:624`.
  - `dc-api/src/bodies/mass.rs` (B6-a) — **consumed.** `mass_properties` / `segment_densities` are
    called from `bake.rs:509-510`, which is on the renderer path.
  - `dc-client/src/anim_rate.rs` — **consumed.** `main.rs:263-283` → `app.rs:191,258` →
    `character.rs:170,179` → `body.rs` throughout.

  *So of five pieces of machinery built in this window, four were joined to consumers in the same
  arc. Only the correlation kernel was not — and it is the one whose consumer is a separate,
  sequenced slice.*

**⚠ This is the `DeepField::chapters` failure mode re-run, and the brief predicted it.** `chapters`
sat unlisted through three sweeps while each added rows for its neighbours in the same struct. Here
the module declares its own hole *and cites § 3 while doing so*, and it has been unlisted for one
day and one sweep. **A self-declaring doc comment is not an index** — that is the fourth time this
file has had to say it (`chapters`, `CoarseField`, RATE, the posture bake), and the fifth
now.

---

## 3. STALE — spines.md entries that no longer describe the code (all good news)

**The three biggest live findings the last two sweeps carried forward have all been FIXED at source
in this window, and spines.md still reads them as live.**

### 3.1 A-7's 🔴 worked-forwards banner — **DISCHARGED.** The source fix landed.

`spines.md:2350-2366` carries *"🔴 **THAT LAST SENTENCE DESCRIBES CODE THAT NO LONGER EXISTS** —
found 2026-08-02 at `3cf8778`, **STILL LIVE** 2026-08-03 at `2985273`"*, naming three comments that
falsely asserted an `outcrop_shares(&[])` call. **All three now read correctly at `08afb88`:**

- `erosion/mod.rs:337-340`: *"P11 slice 2 **retired** the provider-seam walk (`outcrop_shares(&[])`)
  that used to answer this; **same value, no seam consulted.**"* — was *"the pass names no
  lithology."*
- `erosion/weathering.rs:419-422`: *"the provider-seam walk (`outcrop_shares(&[])`) that used to
  answer…"* — was *"the same walk asked with an empty section."*
- `erosion/transport.rs:354-359`: *"since P11 slice 2 the named basement constant (`DEEP_BASEMENT`,
  granite; **the provider-seam walk is retired**)"* — was *"Nothing is named here."*

The absence itself still holds: `grep -rn "outcrop_shares(&\[\])" --include=*.rs crates/` at
`08afb88` → **2 hits, both inside the corrected comments**, zero call sites. **PRESCRIPTION:** stamp
the banner `✅ FIXED 2026-08-05 at 08afb88` and keep the paragraph as dated testimony (the file's own
`CoarseField` / `erosion.rs` precedent). *This was live for four days and three sweeps; recording the
discharge is the point of the row.*

### 3.2 The `Litho::of_material` guard-sentence A-2 — **DISCHARGED, and it died rather than relocating again.**

The 2026-08-02 sweep found *"is what fails loudly if a pack adds a fine clastic the bucket has never
heard of"*; the 2026-08-03 sweep found it had **moved** to the test's doc comment. At `08afb88`,
`grep -rn "fails loudly if a pack" --include=*.rs crates/` → **zero hits.** The doc comment now
reads, honestly and scoped: *"`lithology_buckets_agree_with_the_registry` asserts it agrees with that
authority **for every vanilla member**"* (`lithology.rs:353-354`), above a test that still opens
`let set = dc_core::materials::geology::vanilla();` (`:1181`) — **claim and test now agree.**
**PRESCRIPTION:** stamp both the A-2 entry and A-7's `of_material` bullet.

### 3.3 The `DEEP_MAX_WIDTH` A-2 (the expired `<60 s` pregen bound) — **DISCHARGED.**

The 2026-08-03 sweep's finding 9 filed a third live A-2: `field.rs:28` justified `DEEP_MAX_WIDTH`
by a `<60 s` pregen budget the user had renegotiated 20× to 1,200 s. At `08afb88`, `field.rs:29`
reads: *"the bound was 60 s when this was written, **renegotiated 20× to 1,200 s** on…"*. The
justification now names its own expiry. **This closes the A-2 the last sweep reported-not-applied.**

### 3.4 Citation drift — the grain row is the worst affected

Recorded per row in § 1. The concentration is `recorder.rs` (+50 to +170 across every grain-row
citation) and `field.rs` (+4). **`contents.rs`, `cadence.rs`, `runner.rs:674`, `weather_inventory.rs:116`
and `inventory.rs:1573/1738` are mechanically unmoved.** *Two windows running, `coarse.rs`'s
cite-by-symbol row remains the only citation in this file that has never rotted.*

### 3.5 § 6 has fallen behind for the SIXTH time — **fifteen audits unlisted, the worst gap ever recorded**

Counted mechanically (`ls docs/audits/*.md` diffed against the dated slugs in § 6):

```
2026-08-03-b6-body-composition-design          2026-08-04-s0-correlation-measurements
2026-08-03-doc-topology-sweep                  2026-08-04-stand-in-marker-control-scoping
2026-08-03-roadmap-staleness-sweep             2026-08-04-voxel-explainability-audit
2026-08-04-deposition-clock-design             2026-08-04-walk-stop-channel-design
2026-08-04-doc-topology-sweep                  2026-08-05-d2-deposition-trace-verification
2026-08-04-e4-2-convergence-study              2026-08-05-debt-walkthrough-notes
2026-08-04-longleg-swing-probe                 2026-08-05-passes-skill-sweep
2026-08-04-roster-skill-bio-and-parent-findings
```
(+ this file, + the undated `A1-collapse-slice-plan.md`, unlisted since forever.)

Sequence: eleven → five → eight → three → ten → **fifteen**. **Three of the fifteen are again the
corpus sweeps' own artifacts** — the index of audits still does not list the audits that audit the
index — and `2026-08-04-stand-in-marker-control-scoping.md` is the doc that *defines this skill's
own prescribed control*. **PRESCRIPTION:** backfill. And record honestly that six backfills have not
made this a watcher; the section says so itself and the count keeps proving it.

---

## 4. ANTI-SHAPE-IN-THE-WILD

### 4.1 A-7's own 2026-08-05 enumeration is **short by one** — again

`spines.md:2318-2327` files three new A-7 instances from the weathering investigation, closing with:

> `inventory.rs::Cause` is a fourth (four weathering agents, closed), and **`Dissolution` appears in
> both enums** owned by neither.

Verified at `08afb88`: `FlowCause` (`flux.rs:337`, seven regimes + `MOVER_NONE`, exactly filling 3
bits) and `Cause` (`inventory.rs:395`, four agents) are both real and both carry `Dissolution`. **But
there is a THIRD closed enum in the same crate carrying `Dissolution`, of identical shape, and it is
already a § 3 row in this very file:** `lithology::Agent` (`lithology.rs:122-140`) —
`Abrasion` / `Dissolution` / `FrostIce` / `Wave`, a closed engine-owned vocabulary of erosion agents,
each variant's doc comment naming a real-world process family, with a fixed-arity companion struct
`LithoResistance` holding one field per variant (`lithology.rs:218` dispatches on it). A pack cannot
declare an agent, for exactly the reason it cannot declare a mover.

So the count is **four** enums carrying process identities, and **three** carry `Dissolution`
(`FlowCause::Dissolution` flux.rs:352, `Cause::Dissolution` inventory.rs:407, `Agent::Dissolution`
lithology.rs:130) — one transport regime, one weathering cause, one erosion agent, all named
`Dissolution`, all closed, **owned by no one, and no two of them can be made to agree.**

*This does not weaken the section's conclusion; it strengthens it, and it lands on the same shape
A-7 has now gone stale on four separate times — `"the one place the roster is still named"` (short by
one), `draws.rs`'s `"there is no expression anywhere"` (short by four), the ecology-clause `"four
stubs entries"` (short by two), and now `"both enums"` (short by one). **An enumeration stated as
evidence in this file goes stale roughly as often as it is written**, and we still have no
enumeration-completeness check.*

**⚠ Scope note, carried forward not re-derived:** A-7's own paragraph already concludes *"**E6, which
names only `DeepAxis`, is scoped too narrowly**."* Verified: `docs/ARCHITECTURE.md:560` names
`DeepAxis` and `:654` argues the kernel does not require a closed enum; **neither `FlowCause`,
`Cause` nor `Agent` appears anywhere in `ARCHITECTURE.md`** (`grep -n "DeepAxis\|FlowCause" docs/ARCHITECTURE.md`
→ 3 hits, all `DeepAxis`). The ratified indictment is one-quarter as wide as the shape. **This is
USER-OWNED** — widening a ratified indictment is a ratification, not a sweep edit.

### 4.2 The `docs/audits/2026-07-22-seam-inventory.md` row-29 claim about `outcrop_at` — **three separate falsifications, no banner**

The brief's second pointer. **Verified at source, and the answer is: no, not any more, in three
independent ways.** The seam inventory's row 29 (`2026-07-22-seam-inventory.md:209`, repeated at
`:338`) reads:

> `outcrop_at(cell)` · `deeptime/lithology.rs:365 ff.` (`exposed_litho`) — takes `units.last()` ·
> identity `units.last()` — **ARBITRARY once deformation exists** · heir **structural geology** ·
> *"**This is the best-shaped seam in the codebase** — a single accessor with a named heir."*

1. **It is no longer a seam at all.** `Providers::outcrop_at` (`providers/mod.rs:440`) is now a
   *derived accessor*, and its own doc comment says so in the negative:
   > *"the verdict, **derived** as `argmax ∘ outcrop_shares`, never a stored label beside the
   > quantity (S-3) … **Not a slot: it has no `Option<fn>` and no identity of its own**"*
   > (`providers/mod.rs:426-431`).

   The seam — the thing with an `Option<fn>` and an identity default — is now **`outcrop_shares`**
   (`providers/mod.rs:456`), and `providers/mod.rs:270-274` records the collapse explicitly. So the
   *"best-shaped seam"* title, if it still belongs to anything, belongs to a different symbol.
2. **The identity is not `units.last()`.** It has not been since journal/0068 (2026-07-22, the same
   day): `exposed_litho` (`lithology.rs:529`) is `window_walk(units).1` (`:545`) — the argmax of a
   0.9 m near-surface thickness window with a nearest-surface tie-break. *A-7's own worked instance
   two sections up in `spines.md` records this change; the seam inventory does not.*
3. **The file:line is wrong twice over.** `deeptime/lithology.rs:365` at `08afb88` is inside
   `Litho::of_material`'s match arms, not `exposed_litho`; and the function the row is *about* now
   lives in a different file (`providers/mod.rs`).

   *And the heir survives, relocated:* `providers/mod.rs:432-434` still names structural deformation
   — *"when the structural-deformation heir supplies dipped shares the verdict dips with them"* —
   but it now lands on `outcrop_shares`, not on `outcrop_at`.

**PRESCRIPTION (a prescription):** `2026-07-22-seam-inventory.md` is a dated-measurement artifact
under CLAUDE.md read-first item 5 — **immutable body, mutable header.** It owes a top banner naming
the three falsifications above and pointing at `providers/mod.rs:426-431`. It carries none today
(`grep -n "banner\|SUPERSEDED\|⚠" docs/audits/2026-07-22-seam-inventory.md` was not run as a
completeness check — what I verified is that the row's own text at `:209` and `:338` is unqualified).
**Read-first item 5's own backlog clause applies: "the corpus was never swept for missing banners."**
This is one.

*Why this one matters more than an ordinary stale citation: the phrase **"the best-shaped seam in
the codebase"** is a design exemplar. It is the sentence a cold session greps for when it wants to
know what a good seam looks like — and it currently points at a symbol that has been explicitly
demoted out of seam-hood, at a line number in the wrong file.*

---

## 5. Anti-shapes checked and NOT found in the delta

Reported as nulls, with what was searched — a null from an unrun check is not a result.

- **A-2 (`JUSTIFIED-BY`):** `grep -rn "JUSTIFIED-BY" --include=*.rs crates/` → **zero hits**,
  consistent with CLAUDE.md's own note that the convention got 3 uses and 0 in `crates/`. The
  convention is dead; nothing to audit.
- **A-2 (prose justifications) in the delta:** `grep -rn "so that a pack cannot|because none
  survive|since nothing yet emits|no writer exists|nothing calls it yet|has no caller|no production
  caller|nothing consumes"` over `crates/` → 5 hits, **all of them the grain-grade row's honest
  self-declaration** (`recorder.rs:1506`, `release_spectrum_probe.rs:85,319`,
  `member_diversity_probe.rs:421,442`) and all still true. No expired premise found in the new code.
- **`anim_rate.rs` checked for A-1** (a stand-in hardening into a definition) and **it is the
  opposite** — it *retires* a stand-in. `DEFAULT_TARGET_FPS = 12.0` (`anim_rate.rs:60`) carries
  *"a default for a setting, **not a new world-global constant** — the shape `AnimRate` exists to
  retire"*, `AnimRate::new` **refuses rather than clamps** and is deliberately unbounded (`:88-93`,
  *"a bound here would be exactly the invented absolute constant this module retires"*), and
  `MIN_POSES_PER_CYCLE = 2` is derived from the Nyquist edge rather than tuned. **A candidate S-5 /
  S-8 instance if the main session wants one; proposed, not added — § 4's rule binds the auditor.**
- **NOT OPENED, so a null from them is not a result:** `dc-api/src/bodies/limits.rs` +
  `limits/tests.rs` (1,568 lines, read only at the call-graph level), `dc-client/src/body.rs`
  (+1,896 in the window, grepped only), `dc-worldgen/tests/correlation.rs` (551 lines, grepped for
  imports), `examples/correlation_m0_probe.rs` (485 lines, grepped only), the S-1/S-4/S-6/S-7/S-8/S-9
  narrative bodies (citations spot-checked, claims not re-argued), and `docs/spines.md` §§ 1210-1324
  (S-10) and 1907-2017 (A-4).

---

## 6. What I could not resolve

1. **Whether the correlation-chain row is state 1 or state 3.** The module's own text supports state
   3 (S2 owns the wiring, and it is sequenced); state 1 is defensible if S2 is read as a gap the
   slice opened. **The main session decides; § 4 binds the auditor.**
2. **Whether `lithology::Agent` should be filed as a fourth A-7 instance or folded into the existing
   `FlowCause`/`Cause` bullet.** I report the fact; the filing shape is an editorial call on a
   section that is already USER-OWNED at its E6-scoping conclusion.
3. **The 2,559-line size flag on `docs/spines.md`** — raised by the `2985273` sweep as finding 10
   (2,527 lines against the 2,500-line REGISTRY bar) and **still unaddressed; the file has grown 32
   lines since.** Same disposition as last time: the archive candidates are visible (six superseded
   sweep header blocks, the `✅ FIXED` / `✅ APPLIED` entries, § 3's Departed rows) but *which* have
   stopped requiring a live read is a judgement, and a wrong archive deletes the corpus's memory of a
   defect class. **Proposed to the main session, second consecutive sweep.**
4. **`SpeciesLayout`'s third storage variant** (the `2985273` sweep's `proposed_not_applied`) — I did
   not re-open it. It remains proposed and unruled; carrying it forward unchanged rather than
   re-deriving it.
5. **No cargo was run** (brief: read the source, don't compile it). No claim here rests on a build,
   a test result, or a measured number produced by this sweep.

---

## 7. Summary for the integrator

| class | count | items |
|---|---|---|
| **CONFIRMED-STILL-TRUE** | 20 | every standing § 3 row, searches recorded per row (§ 1) |
| **§3-ROW-EMPTIED** | **0** | nothing was consumed this window |
| **NEW-INSTANCE** | 1 | the correlation chain — `deeptime::correlate` + `StrataEvent::epoch_bottom/top` (§ 2) |
| **STALE** | 5 | A-7's 🔴 banner (fixed), the `of_material` A-2 (fixed), the `DEEP_MAX_WIDTH` A-2 (fixed), grain-row citation drift, § 6 fifteen unlisted (§ 3) |
| **ANTI-SHAPE-IN-THE-WILD** | 2 | A-7's enumeration short by one (`lithology::Agent`); the seam-inventory `outcrop_at` claim, unbannered (§ 4) |
| **CONTROL** | 1 | `standin_locus_check.py`: 14 markers, **1 unresolvable** — `recorder.rs:832`, whose locus (`stubs.md #52`) exists and is not named in code (§ 0) |

**Nothing in this file was applied.** Per the dispatching brief, the write-set was this document and
`docs/audits/.sweep-watermarks.json` only. Every prescription above is marked as a prescription and
is a hypothesis about the right fix, not evidence — this audit has shipped a half-wrong prescription
before (the `dc:field/head` `Forced` claim: free, yes; pins it, no).
