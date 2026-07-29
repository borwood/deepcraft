# spine-audit — FULL pass, 2026-07-29 (evening)

- **Watermark at dispatch:** `96ab14b` · **HEAD read at:** `72fbe86` · **new watermark:** `72fbe86`
- **Mode: FULL.** The reference side moved — `docs/spines.md` itself changed in ten commits
  (`0e0b210 e568c5b f10dc03 9e0de05 a9146dd 02aa81a de85bac 0dcdadb 2d6ab26 0ee4c54`), which the
  watermark file's own `_full_run_triggers.spine-audit` names as the void-everything trigger.
- **No cargo** was run. Docs-vs-source reading only, plus `git log`/`git show`/`grep`.
- **Written:** `docs/spines.md`, this file, `docs/audits/.sweep-watermarks.json`. Nothing else.
- **Every claim below is stated as-of `72fbe86`** unless a different commit is named.

**The first `spine-audit` artifact under `docs/audits/`.** Prior runs wrote only the header block
in `spines.md` — which is why `.sweep-watermarks.json`'s baseline note records *"spine-audit had
left no artifacts at all"*, and why its findings could only ever be read from the doc they edited.

---

## 1. Per-spine verdicts

Verdicts are **HOLDS** (instance real, claim true, citation resolves), **HOLDS / REFS DRIFTED**
(claim true, `file:line` stale — refreshed in `spines.md`), or **AMENDED** (the entry's substance
needed a change).

| spine | verdict | evidence |
|---|---|---|
| **S-1** bounded derivation + synthesized frontier | HOLDS | `dc-client/src/farmesh.rs` present; the S2-collapse bullet's *"the running instance is not currently running"* still true — the toy tier's only callers are `dc-sim/tests/s2_{torture,measurements}.rs` (search `grep -rn "ToyWorld\|force_fact\|statistical::engine" --include=*.rs crates/` excluding `dc-sim/src/statistical/`) |
| **S-2** committed facts vs fluid state | HOLDS | CSR refs verified: `FluxRecord::cell_start` `flux.rs:423` (doc `:421`, inside the cited `:415-424`); `FactLedger::rows` `inventory.rs:663`, `SlotRun` `:674` — both inside the cited ranges |
| **S-2 storage corollary** (the house layout) | HOLDS | both table instances resolve; the A-4 check-before-designing rule was in fact exercised again this batch (the tripwire suite reused `providers_common`'s FNV rather than writing a hasher) |
| **S-3** summary derived from the authority | HOLDS | `head.rs:210-212` S-2 citation + `permeability_of` `:214-216` verified; `dc-api/src/identify.rs` present |
| **S-4** coarse cause, fine expression | HOLDS / REFS DRIFTED + **AMENDED** | the cell-integral note is real: `coarse.rs:482-489` carries `E[w_home] = 4·(3/8)² = 9/16` beside the prose, exactly as the entry claims. **Live-violation ref drifted**: `regolith_at_voxel` is `field.rs:806` (doc from `:788`), cited `:735-744` → refreshed to `:788-806` |
| **S-5** seams with identity defaults | HOLDS | no seam added or converted this batch; the `31 LIVE of 34` correction and its *"the 5-converted numerator was NOT re-verified"* caveat stand unchanged and are still honest |
| **S-6** declared relations, never incidental order | **AMENDED (2 substantive)** + REFS DRIFTED WHOLESALE | see § 1a |
| **S-7** distribution-first quantization | HOLDS | `fill.rs` instances unchanged |
| **S-8** one quantity, many regimes | HOLDS | `EdgeId::declared` / `is_declared_edge` still the only constructor path; `split_by_shares` still the one budget |
| **S-9** derivable base + sparse facts + fallback | HOLDS / REFS DRIFTED | `collapse.rs`'s `weathering_product_m` read moved `:1417-1420` → **`:1467`**; the M3 residual's `derive_regolith_at` moved `:818-824` → **`:900-905`** and the Movement-2a authority block `:796-810` → **`:876-892`**. **The residual itself is STILL LIVE and unchanged**: `field.rs:904` still builds the view from `FactLedger::empty_with_bedrock`, so the derived `H` view still cannot see the ledger's facts |
| **S-10** frozen-snapshot doubly-limited gather | HOLDS / REFS DRIFTED | `water/sat.rs:9-14` header claim verbatim ✓, `lateral_c: 0.25` at `:82` ✓ (both unmoved). `Erosion::diffuse` moved `:3148` → **`:3175`**. `let rate = cfg.diffusion * dt;` `:3197`, `let diff_sub = rate / f64::from(n_sub);` `:3216` — the entry says *"four lines above"*; it is now nineteen, because the sub-cycle gained the `creep_substep` branch and its comment. Substance unaffected: the order and the ownership are as the ruling requires |

### 1a. S-6 in detail — the only spine whose substance changed

**Amendment 1 — the SCHEDULE axis was never listed** (journal/0124, `ARCHITECTURE.md` § Schedule,
user-ratified). A pass's schedule is a sum type on its declaration —
`Seed · Step(Cadence) · SeedAndStep(Cadence)`, `deeptime/schedule.rs:82-88` — because *a
`seeded: bool` cannot express run-once-only*. It belongs to this spine for what it **deleted**: the
runner used to seed three passes pre-loop in `mod.rs` and skip them at epoch 0, so *membership of
the setup epoch was an artifact of which file called what*. Nothing seeds today and the roster
asserts it by name (`runner.rs:1698`, `tests/schedule_axis.rs:131`). Only § 3's `Schedule::Seed` row
mentioned the axis; S-6's narrative did not. **Added as an instance.**

**Amendment 2 — the RATE bullet named the wrong function.** It read *"`declared_passes` is the one
place the table is applied"*. It is not: `deep_passes_with` (`runner.rs:640-651`) applies the table;
`declared_passes` (`runner.rs:655`) is the one place a pass is *declared*, and its own comment says
so — *"split out so `deep_passes_with` has exactly one place to apply the table and cannot miss a
pass"*. The property the sentence wanted (a new pass cannot be added past the table) is real and
survives. **Corrected.**

**Refs drifted wholesale, eight hours after being refreshed.** `e568c5b` verified every non-§3
citation at `ec9858e` that morning; `de85bac`/`ea1028d` then moved `runner.rs` again.

| cited | actual at `72fbe86` |
|---|---|
| revision tokens `runner.rs:66-95`, enum `:97` | `:89-90`, enum `:111` |
| `weather_inventory` declaration `:929-936` | `:963-971` |
| its reasoning `:560-567` / `Exposed` note `:569` | `:566-571` / `:573` |
| `Exposed` axis `:107-109` | `:123` |
| `flow_record` body `:479-495` / decl `:789-802` | `:483-495` / `:818-833` |
| `head`'s `ground` built `:524-530` / consumed `head.rs:456-485` | `:529-537` / `head.rs:462-495` |
| `passgraph.rs:1-20`, `:88`, `:230-231`; `pipeline.rs:25` | **all unmoved** — `passgraph.rs`/`pipeline.rs` untouched this batch |
| `weather_inventory.rs:283`, `biotic.rs:727`, `grid.rs:718-722` | **all unmoved** |

All refreshed in `spines.md`, **except** the two frozen weather-inventory defect entries, whose
in-body refs stay as dated 2026-07-24 testimony by the entry's own instruction — their *live*
parenthetical (the "as of" pointer) was refreshed.

---

## 2. Per-anti-shape verdicts

| anti-shape | verdict |
|---|---|
| **A-1** stand-in becomes the definition | HOLDS. `Litho::reference_material` still the hydraulic model's floor (`head.rs:214-216`); no new instance in the batch |
| **A-2** justification outlives its premise | **TWO ENTRIES STAMPED FIXED, ONE NEW SUB-FINDING** — see § 4.1 and § 4.2 |
| **A-3** test green for an unrelated reason | **INSTANCE ADDED** (the `chapters.len()` hash + its blindness falsifier + the compiler-enforced enumeration) — see § 4.4 |
| **A-4** built machinery with no consumer | **ONE NEW § 3 ROW** (§ 3.1). No second-mechanism-beside-the-first found in the batch; the tripwire suite reused the existing fixture and hasher, and `deep_passes_with` is a single application point rather than a second one |
| **A-5** locality of cause vs effect | HOLDS, no new instance |
| **A-6** measuring what cannot change a decision | HOLDS, no new instance |
| **A-7** naming a content identity inside a process | HOLDS. `Litho::as_deposited` remains the one named-roster site; nothing in `cadence.rs`/`schedule.rs`/`coarse.rs`/`collapse.rs`'s new code names a material |

---

## 3. § 3 "Built, and nothing calls it" — rows added / emptied / confirmed

**Rows emptied: NONE.** All fourteen prior rows re-confirmed uncalled in production at `72fbe86`.
**Rows added: ONE.**

### 3.1 ADDED — the RATE AUTHORING path

`CadenceTable::{with, unmatched}` · `Cadence::sub_turned` · `Schedule::integrated_dt`.

Built 2026-07-29 by journal/0123 (RATE) + journal/0124 (SCHEDULE). The **reading** half is fully
consumed. The **authoring** half has zero production callers:

- every production entry point hands an empty table — `field.rs:608`, `mod.rs:175`, `runner.rs:619`
- search `grep -rn "CadenceTable\|sub_turned\|unmatched\|integrated_dt\|with_cadence" --include=*.rs crates/`
  at `72fbe86`. Every non-empty construction is a test: `tests/rate_axis.rs:105,109`,
  `tests/schedule_axis.rs:109-113`, `runner.rs:1728-1733,1776`, `cadence.rs:314-316`
- `unmatched` (`cadence.rs:251`) has **no caller at all** but `runner.rs:1759` (a test) and its own
  suite; `runner.rs:633` is a **doc comment naming its future consumer** — the § 3 signature verbatim
- `Schedule::integrated_dt` (`schedule.rs:169`): only `tests/schedule_axis.rs:79,115` and
  `schedule.rs`'s own suite. The runner does not compute `dt` through it
- `with_cadence` **is** consumed (`runner.rs:645`) and is excluded from the row

**Intended consumer, named in-code with unusual precision** — `mod.rs:186-190`: *"This is the seam,
not the format… the manifest's cadence section constructs the `CadenceTable` handed here. Inventing
that format now would be building the general mechanism ahead of its caller, so **the parameter
exists and the loader does not**."* That is `dependency-graph.md` **E7**.

**State 3 (HELD AS A CANDIDATE), not state 1.** The absence is ruled, not owed.

**Why the row matters more than the machinery.** journal/0124 added a `Schedule::Seed` row *the day
it was built*, and cited that discipline approvingly. journal/0123 — one commit earlier, the same
arc, the same author-team — added none for its own unconsumed half. And `mod.rs:186-190` declares
the hole in plain language, which is the third time in five days § 3 has had to make the same point:
**a self-declaring comment is not an index** (`chapters`, then `CoarseField`, now this).

### 3.2 CONFIRMED uncalled — the fourteen, with the search run

| row | search | result |
|---|---|---|
| `fits_in_pores` / `K_PORE` | `grep -rn "fits_in_pores\|K_PORE" --include=*.rs crates/` | only `packing.rs` (defn + 7 tests) and a doc cross-ref at `geology.rs:672`. Zero call sites |
| `recv`/`area`/`lake` exports | `grep -rn "\.recv\b\|\.area\b\|\.lake\b" --include=*.rs crates/` | production readers are all `erosion.recv()`/`.area()` (the *in-sim* value, `biotic.rs:689-700`). Export readers are all tests/examples; **the set GREW by the tripwire suite** (`artifact_tripwires.rs:554,573`) and `tests/deep_config_plumbing.rs`, `tests/head_field.rs:245`, `tests/mfd_routing.rs:401,477`, `examples/mfd_probe.rs:333` — all test readers, so the row stands per the header's *a golden is a test reader, not a consumer* |
| `DeepField::flux` | `grep -rn "\.flux\b\|FluxRecord" --include=*.rs crates/` | unchanged: `tests/flux_record.rs`, `tests/head_field.rs`, `examples/flux_record_probe.rs`. `GOLDEN_FLUX` = `flux_record.rs:67` (prior value recorded at `:65`) |
| `DeepField::head` | as row | unchanged; `GOLDEN_HEAD` = `artifact_tripwires.rs:271` |
| `exhum`/`t_crust` | as row | unchanged; `surface_fingerprint` coverage re-confirmed by `artifact_tripwires.rs:33` naming it explicitly |
| `bound_eighths` / `is_occupancy_solid` | `grep -rn "bound_eighths\|is_occupancy_solid" --include=*.rs crates/` | **zero call sites of any kind.** `:286`/`:307` unmoved. Hits outside those two are doc cross-refs at `:38`, `:250`, `:252` — **and `:276`, which the morning sweep's enumeration missed.** Conclusion unaffected. The row's *called-by* claims for the siblings re-verified at the exact cited lines: `payload.rs:367-368`, `meshing.rs:281` |
| Movement 2a `R`/`H` views | `grep -rn "derive_regolith_at\|derive_bedrock_at\|derived_regolith_m\|derived_structure_stock_m"` | only `tests/rh_unification.rs:56,83`. `field.rs` refs refreshed `:818,833` → `:900,915`; `inventory.rs:1622,1634,1645,1665` **unmoved** |
| `DeepField::geotherm` | `grep -rn "geotherm" --include=*.rs crates/` | unchanged; decl `:555` unmoved, clone site `:634` → **`:648`**. `GOLDEN_GEOTHERM` = `artifact_tripwires.rs:253` |
| `DeepField::chapters` | `grep -rn "chapters" --include=*.rs crates/` | unchanged. Decl `:581` and the *"exported and read by nothing"* doc `:574-580` **both unmoved and re-read verbatim**. `providers_common/mod.rs:469` `h.usize(f.chapters.len())` ✓ exactly as cited. `GOLDEN_CHAPTERS` = `artifact_tripwires.rs:295` |
| `Agent::Dissolution` + `LithoResistance.dissolution` | `grep -rni "dissolution" --include=*.rs crates/` | zero production call sites outside `lithology.rs`. `examples/entry_species_probe.rs:325` + `tests/erodibility.rs` only; `weather_inventory.rs:104,125` handles the *separate* `inventory::Cause::Dissolution` at `0.0`, still excluded from `WEATHERING_AGENTS` (`:61-63`) |
| the S11 water module | `grep -rn "water::" --include=*.rs crates/` | unchanged: `tests/water.rs`, `tests/water_coarse.rs`, `examples/water_spike.rs`, `examples/water_coarse_spike.rs`. `lib.rs:48` `pub mod water` ✓ exactly as cited |
| pass-graph `Resource` vocabulary | `grep -n 'id: "dc:pass/' pipeline.rs` | **exactly 7** (`:330,338,346,354,362,376,396`) — the row's *"7 passes (was 8)"* count re-verified, not carried |
| `column_summary` / `open_air_below` / `ColumnSummaries` | `grep -rn "column_summary\|open_air_below\|ColumnSummaries"` | only `dc-client/src/bench_storage.rs:201,220` and `column.rs`'s own tests. No dc-api handler ✓ |
| the S2 statistical tier | `grep -rn "ToyWorld\|force_fact\|statistical::{engine,ledger,world}"` excluding `dc-sim/src/statistical/` | only `dc-sim/tests/s2_torture.rs` + `s2_measurements.rs`. Row is state 3 by the user's 2026-07-28 ruling; unchanged |
| `CoarseField<T>` (PARTIAL) | `grep -rn "CoarseField\|DitherSource\|sample_dithered\|summarize\|\.sample\("` | adoption confirmed: `collapse.rs:39` imports the type, `:961-964` calls `sample_dithered` through `draws::Coherent` (`:54`), `:1006` `class_window`, `:1608,1663` the window struct, world-scale test `:2433`. **`sample` and `summarize` still uncalled**: every `.sample(` hit is `coarse.rs`'s own tests (`:750,767-777`); every `summarize` hit is prose or `coarse.rs:891-923`'s test. Refs `:479`/`:515` → **`:498`/`:534`** (see § 4.3) |

---

## 4. Findings — proposed corrections and reports

**Applied to `docs/spines.md` (this sweep's one writable doc).** Everything else is **reported**.
**I filed no `corrections.md` entry** — the integrator holds the ordinals. Where a finding looks
correction-worthy I say so and give the mechanism.

### 4.1 ✅ `draws.rs`'s completeness overclaim was FIXED — and the fix is SHORT BY ONE SALT 🔴

The `96ab14b` sweep's A-2 finding was applied by `f10dc03`. Verified: `draws.rs:23-28` now records
the sentence *"was **false when written**"* and names the corrections-#64 mechanism; a new residue 3
(`:66-75`) carries `SALT_TEC_POS`/`_VEL`/`_CRUST` with prefix, call sites and the conversion owed.
That is the right shape for an A-2 fix — retired claim kept beside the correction.

**But the finding named FOUR surviving salts and the correction names THREE.**
Search `grep -rn "SALT_" --include=*.rs crates/dc-worldgen/src` at `72fbe86` returns exactly:

- `SALT_DT_ROUGH` — `grid.rs:34`, read at `refine.rs:155` + the agreement test `draws.rs:256`
  → **accounted for**, residue 2
- `SALT_TEC_POS`/`_VEL`/`_CRUST` — `tectonics.rs:51-53`, 7 sites at `:115-121`, `:409`
  → **accounted for**, residue 3
- **`SALT_DT_PERTURB` — `refine.rs:29` (`0x5900_0002`), live call site `refine.rs:180`
  → NAMED NOWHERE IN `draws.rs`.**

> **Proposed correction (integrator, `corrections.md`):** the applied fix to `draws.rs`'s
> completeness claim is itself incomplete, **by the same mechanism it was written to retire** —
> residue 2 enumerates `SALT_DT_ROUGH` (the salt the *earlier* slice touched) and generalises to
> `deeptime/`'s salts; `SALT_DT_PERTURB` sits nine lines from `SALT_DT_ROUGH`'s only reader and was
> never named. *An enumeration written from what a change touched, twice, in the paragraph
> retiring that habit.*
>
> **Proposed fix (source, not applied here):** either add `SALT_DT_PERTURB` to residue 2 (one
> sentence, matching residue 3's form), or convert it — `refine.rs:180` is a single call site and
> `Draws::bits` is byte-identical to `draw_f64(&[seed, SALT, addr…])`, so conversion moves no
> world. **Prefer the conversion**: `refine.rs` then holds zero hand-rolled salts and the file's
> `draw_f64` import goes, which is a structural end-state rather than a longer prose list.

### 4.2 ✅ `Erosion::diffuse`'s A-2 was FIXED — with one residue ⚠

`erosion.rs:3150-3160` now opens *"**The shipped configuration is NOT such a world** — and the
sentence that used to end this paragraph said it was (anti-shape A-2, caught by the 2026-07-29
spine-audit)"*, states the 2.1×, the `n = 2`, the default-on flag, *"its goldens moved with the
fix"*, names the reachable `n = 1` fixed point, **and installs a standing check** —
`tests/creep_operator.rs:119::the_shipped_world_has_cells_past_the_bound`. That last part is the
difference between a corrected comment and a fixed A-2: the premise now has a test, not a reader.

> **Reported (source, not applied):** `erosion.rs:4557-4559` — the doc comment on
> `inside_the_bound_the_driver_is_the_raw_step` — still says *"The claim that **the shipped world is
> untouched** rests entirely on `x / 1.0 == x`"*. That is the retired claim, surviving 1,400 lines
> below its own retraction, on the test that proves the **predicate** rather than the world.
> Suggested: *"the claim that a world **inside the bound** is untouched"*. Not correction-worthy on
> its own (nothing downstream reads it), but it is the same sentence in the same file.

### 4.3 🔴 A citation-accuracy claim falsified by its own commit

`spines.md`'s `CoarseField` § 3 row read: *"`:479` `sample_dithered`, `:515` `summarize` … verified
at those exact lines 2026-07-29 at `96ab14b`; **the adoption did not move them**."*

That clause was written **in `0dcdadb`, the adoption commit**. `git show 96ab14b:…/coarse.rs` gives
`sample_dithered` at **479** and `summarize` at **515**; at `72fbe86` they are **498** and **534**.
`0dcdadb` inserted ~19 lines of doc above `sample_dithered` — the `E[w_home] = 4·(3/8)² = 9/16`
cell-integral derivation now at `:482-489`, which is *the very paragraph the same slice added
because the type's old prose had misled its first adopter*. `:353`, `:446` and `:296` are genuinely
unmoved.

**Corrected in place in `spines.md`, and recorded rather than silently repaired.** A false claim
specifically about citation accuracy, inside the index whose subject is citation accuracy, in the
same diff, is the strongest available argument for the skill's own *"cite the commit you read it
at"* rule (corrections #67): the stamped commit is exactly what made the error findable.

> **Optionally correction-worthy** (integrator's call): the general form is *a citation-freshness
> claim must be verified against the commit being written, not the commit being cited* — a `git
> diff --stat` on the file you are citing, in the commit you are citing it from. Nothing downstream
> propagated the wrong lines, so the cost was one sweep's time.

### 4.4 A-3 instance added: a hash green about the wrong thing, and its structural medicine

`surface_fingerprint` hashed `f.chapters.len()` (`providers_common/mod.rs:469`) and nothing inside
the table. A real number about the wrong thing reads as coverage: the chapter count is a config
constant, so the byte pinned the config — plates could have been reseeded, re-advected or all
flipped oceanic without a twitch. Purest A-3: not lenient, **about something else**. Closed by
`GOLDEN_CHAPTERS` **plus a negative control**,
`artifact_tripwires.rs:501::the_chapter_length_byte_is_blind_to_what_the_chapter_table_says`.

And the generalisation, which is why it earned a spines entry rather than only a § 3 note: the same
file's *"every shipped artifact has a tripwire"* is a **completeness claim** — the class that
produced § 4.1's defect twice. It is **not prose**:
`artifact_tripwires.rs:544::every_deepfield_member_is_classified` exhaustively destructures
`DeepField`, so adding a member **stops the suite compiling** until it is classified with its
evidence. Same move as `EdgeId::declared`, `CoarseField`'s `compile_fail` doctest and
`draw_domains!`'s duplicate-salt `const` assertion. **A completeness claim about a set the compiler
can see should never be prose** — added to A-3, and it is the standing answer to § 4.1.

### 4.5 § 6's audit index fell five behind, one day after the backfill that fixed eleven

The `2026-07-29` audits (`roadmap-classification`, `refinement-coupling-priors`,
`member0-coarsefield-design`, `far-frontier-tourmap`) were unlisted. Listed now, one line each,
plus this file. **Reported as a process finding, not just a fix:** the backfill *was* the remedy for
being eleven behind, and it went stale within a day. § 5's dead `JUSTIFIED-BY` marker is the
standing evidence that a convention asking an author to remember something dies. The candidate
mechanism is the one § 4.4 just praised — an enumeration something *checks* (a sweep comparing the
list to `ls docs/audits/`), not a list a human appends to. **Escalated to main session; not
designed here.**

### 4.6 Live, unchanged, and worth re-stating because it survived a batch that touched its files

**S-9's Movement-3 residual is still open.** `field.rs:904` builds the derived `H` view from
`FactLedger::empty_with_bedrock` — an **empty** ledger — so the "one authority" view structurally
cannot see the facts `dc:deep/weather_inventory` commits. Two derivations of one quantity that
cannot agree, which is S-9's own consistency law. `field.rs` was edited by this batch and this was
not touched. Owner: ROADMAP Movement 2 / R-H unification.

---

## 5. Citations verified vs drifted

Counted over every `file:line` citation in `docs/spines.md` that this sweep actually resolved
against the tree at `72fbe86` (I did not attempt all ~90; the ones below are the ones checked).

- **Checked: 61.**
- **Verified unmoved: 44** — including all of `passgraph.rs` (`:1-20`, `:88`, `:230-231`),
  `pipeline.rs:25`, `contents.rs:286/307`, `inventory.rs:1622/1634/1645/1665` + `:663/:674`,
  `flux.rs:415-424`, `lib.rs:48`, `payload.rs:367-368`, `meshing.rs:281`,
  `providers_common/mod.rs:469`, `field.rs:555/564/574-580/581`, `grid.rs:341/563/580/720`,
  `weather_inventory.rs:283/286`, `biotic.rs:727`, `head.rs:210-216`, `water/sat.rs:9-14/:82`,
  `coarse.rs:353/446/296`, `collapse.rs:947/1006/2433`, `lithology.rs` refs.
- **Drifted: 17** — 13 in `runner.rs` (S-6, listed in § 1a), 4 elsewhere: `field.rs` ×3
  (`:818,833`, `:634`, `:735-744`, `:796-810`, `:818-824` — counted as three sites),
  `coarse.rs` ×2 (`:479`, `:515`), `collapse.rs:1417-1420`, `erosion.rs:3148`, `head.rs:456-485`.
  **All 17 refreshed in `spines.md`.**
- **Deliberately left as dated testimony: 5** — the two frozen weather-inventory defect entries'
  in-body refs, and the three `pregen/history.rs` refs to a deleted file (already annotated).

**Drift concentration is not uniform and that is actionable.** `runner.rs` accounts for 13 of 17,
and it has now been refreshed **three times in eight days**, going stale within one merge each
time. `spines.md` already records the honest remedy for the file itself (an ordinary module split;
it is still 2.4× the 700-line threshold at 1,654 lines). **Until that lands, a `runner.rs` line
number in any doc should be read as decoration.** Recommend citing `runner.rs` by **symbol** — the
pass id string, the const name, the fn name — which is stable and greppable, and is what every
refresh actually used to find the new line.

---

## 6. Unverifiable / not attempted — flagged

- **Nothing was compiled or run.** Every "still uncalled" verdict is a grep over `crates/`, not a
  dead-code analysis: a caller reached through a trait object, a macro expansion or a generic
  instantiation would not appear. Each row's search string is recorded so the negative can be
  re-run and contested.
- **The `5 converted` numerator in S-5** is still carried forward from the seam inventory and was
  **not** re-verified against `crates/` — same caveat the entry already carries.
- **The ~29 `spines.md` citations I did not resolve** are mostly in `docs/`-to-`docs/` positions
  (spike results, `stubs.md` section numbers, ROADMAP items). They are not code claims and this
  sweep's remit is docs-vs-code; `doc-topology` owns them.
- **`GOLDEN_*` values were read, never recomputed.** The tripwire constants
  (`artifact_tripwires.rs:253,271,295,363`, `flux_record.rs:67`) are quoted as present, not as
  correct — verifying a golden requires a build, which this sweep may not do.
- **`erosion.rs` was in flight in an earlier session.** Read at `72fbe86` on `main`; if a sibling
  branch holds newer content, § 4.2's residue may already be fixed there.
