# spine-audit — FULL sweep, 2026-08-02, at `3cf8778`

**Mode:** FULL. Trigger per `.sweep-watermarks.json` `_full_run_triggers`:
*"any edit to `docs/spines.md` — every prior verdict was made against a different rule."*
`docs/spines.md` moved in the window (`5959d33`, `4613114`, `6904443`, `c546d19`, `7aa7020`,
`0f0a75c`), so every prior verdict was against a superseded rule.

**Watermark read at:** `d8407e1` (previous run's, set by `9edda08`).
**Commit every citation below was read at: `3cf8778`.** Line numbers in this file are
as-of that commit and nothing else.

**Delta covered:** `git log d8407e1..HEAD` = **61 commits**.

**Write-set:** this file + `docs/audits/.sweep-watermarks.json`, in one commit. Nothing
applied to `docs/spines.md`, `ROADMAP.md`, `corrections.md`, `stubs.md` or
`dependency-graph.md` — the dispatching brief scoped this run report-only, overriding the
skill's own apply tier. **The integrator adjudicates and applies.**

**No cargo was run.** Reading pass only; every absence below states the search that
produced it.

---

## 0. Coverage — what was opened, and what was NOT

Opened in full or in the relevant part:

- `docs/spines.md` — § 3 (rows + header), § 2 A-1/A-2 opening/A-4/A-7, § 4, § 5, § 6,
  § S-2 storage corollary, § S-3, § S-4, § S-5, § S-10's `erosion/` citations.
- `crates/dc-worldgen/src/deeptime/species.rs` (new, 660 lines) — in full.
- `crates/dc-worldgen/src/deeptime/erosion/` — `mod.rs` (struct + accessors),
  `transport.rs`, `weathering.rs`, `creep.rs`, `creep_kernel.rs`, `record.rs`,
  `uplift.rs` (targeted).
- `crates/dc-worldgen/src/draws.rs`, `deeptime/refine.rs` (full diff), `deeptime/lithology.rs`
  (full diff + the guard tests).
- `crates/dc-api/src/character.rs` + `crates/dc-client/src/body.rs` (the look and
  posture-bake slices).
- `journal/corrections.md` #88–#94; `docs/audits/2026-08-01-members-into-history-design.md`
  § 6b.

**NOT opened — a null from these is not a result:**

- `crates/dc-worldgen/examples/appearance_tour_p11.rs` (1,673 lines, new) — read only by
  grep for `Litho::of_material`. **Not audited for anti-shapes.** It is the largest single
  new artifact in the delta and it is a measurement instrument, which is the category that
  produced both `flow_cost_probe` regressions.
- `crates/dc-worldgen/examples/species_sparsity_probe.rs` (323 lines, new) — grep only.
- The full bodies-arc client diff (`dc-client/src/body.rs` +337, `character.rs` +72)
  beyond the posture-bake and look paths.
- `crates/dc-worldgen/tests/*` beyond the greps recorded per finding; the golden
  re-capture (`9c3cb49`) was **not** audited for what moved and why.
- `docs/spines.md` § S-6 (declared relations), § S-7, § S-8, § S-9, § S-10's body,
  A-3, A-5, A-6 — **read for citations only, not re-argued.** Their cited files
  (`runner.rs`, `field.rs`, `inventory.rs`, `collapse.rs`) are **absent from the delta's
  `crates/` diffstat**, so their line refs are unmoved; that is a mechanical statement,
  not a re-verification of their claims.

---

## 1. Findings

### F1 — 🔴 `outcrop_shares` LOST A SECOND CUSTOMER, AND THE COMMENTS STILL CLAIM IT DID NOT

**Implicates: A-2 (live), S-5, A-7's own worked instance.** **Disposition: report to main
session. Behaviour change — outside a sweep's write-set.**

At `d8407e1` the composition of the material **below the record** was asked of the provider
seam with an empty section (`erosion.rs:2208-2209` at `d8407e1`):

```rust
// What lies below the record, asked of the seam with an empty section.
self.bedrock_sp.copy_from_slice(cfg.providers.outcrop_shares(&[]).shares());
```

At `3cf8778` it is a constant, written twice:

- `crates/dc-worldgen/src/deeptime/erosion/weathering.rs:419-422`
- `crates/dc-worldgen/src/deeptime/erosion/transport.rs:174-175`

```rust
// What lies below the record: the same walk asked with an empty section,
// which is entirely whatever is beneath the pile. One entry, share `1.0`.
self.bedrock_axis = vec![axis.basement_slot() as u8];
self.bedrock_sp = vec![1.0];
```

**The walk is not asked.** `basement_slot()` resolves to
`lithology::DEEP_BASEMENT = MaterialId::GRANITE` (`lithology.rs:748`), threaded in at
`transport.rs:145` (`SpeciesAxis::new(geology, DEEP_BASEMENT)`). The value is arithmetically
identical to what `exposed_member_shares(axis, &[], out)` would produce
(`lithology.rs:665-687`: an empty section leaves `remaining == OUTCROP_DOMINANCE_WINDOW_M`,
charged to `basement_slot`, then divided) — **so no world is wrong.** The *claims* are.

Three sentences are now false in the tree:

1. `erosion/weathering.rs:419-420` — *"the same walk asked with an empty section."* No walk
   is called.
2. `erosion/transport.rs:354-359` — *"the composition seam answers that question too — an
   empty section is whatever lies beneath the pile (`bedrock_sp`). **Nothing is named
   here**"*. Something is named: `dc:granite`, one file over.
3. `erosion/mod.rs:316-326` (the field doc) — *"So the pass names no lithology; it asks the
   same question every other consumer asks and takes the answer. (Refreshed each epoch in
   `Self::expose`, because the window's heir — **structural deformation** — may one day
   answer it differently per cell…)"*. The heir path is severed for this quantity: a
   pack-supplied `outcrop_shares` override — which `tests/full_agents.rs:367-375` installs —
   no longer reaches the bedrock composition at all.

**Why this is the headline.** `spines.md` § A-7 records this exact site as the corpus's
**worked instance of the diagnostic working forwards** (`spines.md:2066-2072`):

> *what incision detaches* → the first draft wrote `Litho::Basement`. Applying the
> diagnostic … the composition seam already answers exactly that question when handed an
> **empty section** … So the pass asks `outcrop_shares(&[])` and the constant disappears.

That paragraph is now a description of code that no longer exists. **Search proving the
absence:** `grep -rn "outcrop_shares(&\[\])" --include=*.rs crates/` at `3cf8778` → **zero
hits.**

**And it is `corrections.md` #90 landing twice in one slice.** #90 (filed *by* P11 slice 2)
says: *"when a slice changes the grade, type or units of a quantity, the seams that publish
it are part of the diff. A seam that survives a re-grade unchanged is either genuinely
grade-agnostic (say so) or has quietly lost its customer."* The slice caught the **rate**
customer and filed the correction; it did not catch the **bedrock-composition** customer,
which left in the same diff and is still gone at HEAD.

**Recommended disposition (integrator's call, not mine):** either restore the seam call
(`providers.outcrop_shares(axis, &[], &mut dense)` then mask/compact — byte-identical by the
arithmetic above, so free), or keep the constant and **rewrite all three comments to say what
the code does**, add an agreement test that the constant equals the empty-section walk, and
file the A-7 instance. The first option is the one A-7's own worked instance prescribes.

---

### F2 — 🔴 A-7's ENUMERATION IS SHORT BY THREE, AND THE THREE ARE NAMED `MaterialId`s

**Implicates: A-7, and A-2's "completeness claim" family.** **Disposition: report — the
naming is a ratified design ask that collides with a read-first anti-shape declared
NOT ratifiable. This is a loud plea, not an edit.**

`spines.md:2074-2092` says, corrected only hours before the delta opened:

> ~~The one place the roster is still named is~~ **⚠ TWO places, corrected 2026-08-02 at
> `d8407e1`** … `Litho::as_deposited` … `Litho::of_material`

Both listed instances are `Litho` roster matches inside the lithology **adapter**. The delta
added **three constants that name a material directly**, which is the form A-7's DECIDED
quotes verbatim (*"if you feel a need for naming a material directly: that's a want for a
feature that makes the process more robust. Naming directly will never, in any world, be
correct."*):

| constant | at `3cf8778` | value | reached from |
|---|---|---|---|
| `lithology::REFERENCE_MATERIAL` | `lithology.rs:731` | `MaterialId::MUDSTONE` | `member_susceptibility_table` (`:704`) — every erosion rate |
| `lithology::DEEP_BASEMENT` | `lithology.rs:748` | `MaterialId::GRANITE` | `Erosion::set_species_axis` (`transport.rs:145`) — the whole species axis + F1's bedrock composition |
| `erosion::transport::ANCHOR_MATERIAL` | `transport.rs:86-87` | `MaterialId::SANDSTONE` | `COMPETENCE_SCALE`'s derivation (`transport.rs:65-74`) |

**The honest counter-argument, stated because the file's § 4 rule binds me:** two of the
three were **asked for by a ratified design pass**. `docs/audits/2026-08-01-members-into-history-design.md`
§ 6b (`:697-706`) says *"Under member grade the anchor member must be **named** and its
property sheet cited. This is a derivation, not a re-tune — but it must be re-stated, or the
constant becomes a number pretending to be a mechanism."* Both carry agreement tests against
the class answer they replace (`the_named_basement_is_the_class_reference`, and
`the_competence_anchor_is_where_the_facies_rule_puts_it`), and both are documented as
*"a derivation restated, not a re-tune."* That is real diligence and it is not the shape A-7
was written against.

**But A-7 says explicitly: *"This is not a ratifiable carve-out … there is no world in which
it is correct. It is a defect."*** So a ratified design pass and a read-first anti-shape are
in direct conflict, and neither an auditor nor a slice may resolve that. **Applying A-7's own
diagnostic — which capability is the process missing? — gives a concrete answer in all three
cases: the `GeologySet` does not declare its own basement or its own reference sheet.** The
seam already exists for one of them: `SpeciesAxis::new(geology, basement)` takes basement as a
**parameter** (`species.rs:139`) and `set_species_axis` hands it a constant. `DEEP_BASEMENT`
is the weakest of the three — § 6b never asked for it, and it is the one that displaced a
seam (F1).

**Recommended disposition:** main-session item. Either (a) A-7 gains a stated carve-out for
*calibration anchors with a derivation + an agreement test* — which requires the user, since
A-7 currently forbids exactly that — or (b) a `GeologySet::reference_material()` /
`::basement()` declaration is sequenced and the three constants become its identity defaults
(S-5). Either way, **A-7's enumeration is wrong at HEAD and should say so**, since this is
the third consecutive sweep to find this file's completeness sentence stale.

---

### F3 — 🔴 § 3 ROW EMPTY-ABLE: the resting-posture bake has a production caller

**Implicates: § 3 (the good exit), and CLAUDE.md's same-commit rule.** **Disposition: empty
the row.**

The row at `spines.md:1908` was added at `5959d33` (the `d8407e1` sweep) and reads
*"**no production caller** … The consumers are `bake/tests.rs` (437 lines) and their printed
report."* Its named intended consumer is *"the renderer's `build_plan_assets` … replacing the
pinned `trunk.pivot_m[1]`-as-hip read."*

**That consumer landed five commits later, in `f4e7f75` ("posture bake consumer slice: the
renderer reads the derived hip"), and the row was not emptied.**

- `crates/dc-client/src/body.rs:429-442` — `pub fn derived_root_delta_m(plan) -> Result<f64, String>`
  calls `bake_resting_posture(plan, "stand")` at `:434` and matches all three `BakeOutcome`
  arms.
- Consumed in production at `crates/dc-client/src/character.rs:426`
  (`let (derived_root_m, root_delta_m) = match derived_root_delta_m(&plan) { … }`), plus
  `body.rs:793` (the probe path).

**Search:** `grep -rn "bake_resting_posture|RestingPosture|BakeOutcome|ChainPose|JointAngle"
--include=*.rs crates/` at `3cf8778`. The row's own claim that the only non-`bake.rs` hits are
the re-export and *"`dc-client/src/body.rs:349`, a doc comment"* is false at HEAD:
`:349` is still a doc comment, but `:430`, `:434`, `:435`, `:436`, `:440` are the call.

**Row exit reason:** state 1, CONSUMED, `f4e7f75`, 2026-08-02. The row's neighbouring note —
that `stance_chains` shipped consumed at `dc-client/src/body.rs:364` — **re-verified unmoved
at `3cf8778`** (`for chain in dc_api::bodies::stance_chains(plan, "sole")`).

**Count consequence:** § 3 goes **19 → 18**. Re-run the header's own command after the edit:
`awk '/^# 3\. Built, and nothing calls it/,/^\*\*Departed/' docs/spines.md | grep -c '^| '`
minus 1. It returns **19** at `3cf8778` today, which matches the header — the header's count
is correct as written and becomes wrong the moment the row is removed.

---

### F4 — A-2 STILL LIVE: the `Litho::of_material` guard claim was fixed in one place and left in the other

**Implicates: A-2.** **Disposition: report (source-file doc comment; brief is report-only).**

The `d8407e1` sweep's finding #2 (`spines.md:34-41`) was that `Litho::of_material`'s doc
claimed the agreement test *"is what fails loudly if a pack adds a fine clastic the bucket
has never heard of"* while the test iterates `geology::vanilla()` and structurally cannot see
a pack's members.

**Half applied.** `of_material`'s own doc now reads honestly
(`lithology.rs:353-355`): *"asserts it agrees with that authority **for every vanilla
member**."* Good.

**The identical false sentence survives verbatim on the test**, `lithology.rs:1174-1176`:

> *"It is the test that makes the interim bucket legitimate rather than an S-3 parallel rule
> — and it is what fails loudly if a pack adds a fine clastic the bucket has never heard of."*

with the body at `lithology.rs:1179`: `let set = dc_core::materials::geology::vanilla();`.
Its sibling `every_depositional_class_round_trips_through_the_bucket` (`:1259-1260`) is the
same shape.

**This is the corpus's own recurring failure mode landing on the correction itself** — the
same "fixed at the pointer, live at the target" mechanism CLAUDE.md read-first item 5 names.
The A-2 entry in `spines.md` should be stamped *partially applied*, not closed.

---

### F5 — § 3: NO ROW TO ADD. Two candidates considered and rejected, with reasons

**Implicates: § 3, A-4.** **Disposition: no change; record the reasoning so the next sweep
does not re-derive it.**

The delta's two plausible sources were checked:

**(a) `deeptime/species.rs` — the P11 sparse machinery. NOT a row: consumed in production
the day it was built.** `SpeciesAxis` / `SpeciesLayout` / `SpeciesPlane` /
`build_transport_layout` / `build_creep_layout` / `build_local_layout` / `split_row_into` /
`csr_rows_mut` / `mask_of_dense` all have production call sites in
`erosion/{transport,weathering,creep_kernel,record}.rs` — search
`grep -rn "SpeciesLayout|SpeciesPlane|SpeciesAxis|build_transport_layout|build_creep_layout|build_local_layout|split_row_into|csr_rows_mut|mask_of_dense|MAX_DEEP_SPECIES" --include=*.rs crates/`
at `3cf8778`. **One member is uncalled outside its own tests:** `SpeciesAxis::carries`
(`species.rs:222`), whose only readers are `species.rs:592,595`. One accessor is not a row.

**(b) `DeepConfig::identity_audit` — the draw-retirement instrument. NOT a row.** It is
`false` in production (`grid.rs:605`) and read by
`examples/{member_diversity_probe,appearance_tour_p11}.rs`. That is an instrument with a
*current* consumer, not machinery awaiting one — the `flux_record_probe` category, not the
`chapters` category. Filed here rather than as a row so the next sweep does not re-open it.

---

### F6 — § 3: THE OTHER 18 ROWS RE-CONFIRMED UNCALLED, with one `where`-column drift

**Disposition: refresh one citation; leave the rest.**

Searches at `3cf8778`, each `grep -rn --include=*.rs crates/` with the defining file
excluded:

| row | verdict | search / evidence |
|---|---|---|
| `fits_in_pores` / `K_PORE` | uncalled | only `packing.rs`, `dc-core/src/lib.rs` re-export, `geology.rs:731` doc xref |
| `recv`/`area`/`lake` | uncalled | exported-plane readers still tests/examples only |
| `DeepField::flux` | uncalled | ditto |
| `DeepField::head` | uncalled | ditto |
| `exhum`/`t_crust` | uncalled (exported plane) | in-sim read confirmed at `erosion/uplift.rs:89` — **row cites `:86`, off by 3 after the split** |
| `bound_eighths`/`is_occupancy_solid` | uncalled | `contents.rs` only |
| R/H derived views | uncalled | `rh_unification.rs` + `inventory.rs`'s own suite; `field.rs`/`inventory.rs` absent from delta diffstat ⇒ refs unmoved |
| `DeepField::geotherm` | uncalled | unchanged file set |
| `DeepField::chapters` | uncalled | **`where` ref `tests/providers_common/mod.rs:545` → actually `:600`** (+55, moved by the delta's fingerprint work) |
| `Agent::Dissolution` | uncalled | probes + `tests/erodibility.rs` only |
| S11 water module | uncalled | unchanged |
| `Resource` vocabulary | 7 passes | unchanged |
| `column_summary` | uncalled | `bench_storage.rs` only |
| S2 statistical tier | uncalled (state 3) | `dc-sim` suites only |
| `CoarseField` (partial) | `sample` + `summarize` still uncalled | `grep -rn "\.summarize\(\|CoarseField::" --include=*.rs crates/` → `coarse.rs` defn + its own tests only |
| `Schedule::Seed`/`SeedAndStep` | no production declarer | `schedule.rs` + two test rosters |
| RATE authoring path | no production authorer | `field.rs:608`, `mod.rs:178`, `runner.rs:660` all **verified unmoved at `3cf8778`**; `mod.rs:189-193`, `runner.rs:674`, `runner.rs:686` likewise |
| the deep tier's CONTENT DOOR | **still uncalled** | `grep -rn "run_cells_with_geology\|build_field_cfg_cadence_geology" --include=*.rs crates/` → only the two vanilla wrappers (`mod.rs:202`, `field.rs:622,635`). P11 slice 2 threads `set_species_axis(geology)` (`mod.rs:259`) but the geology reaching it is still `vanilla()` on every production path |

**Note on the content door:** slice 2 made it *matter more* — the species axis is now derived
from the content set, so a custom pack silently gets a vanilla-derived transport axis as well
as a vanilla-laid record. The row's severity rose; its status did not change.

---

### F7 — S-2's storage corollary has a THIRD variant in the tree and its table has two

**Implicates: S-2's storage corollary, A-4 (positively).** **Disposition: propose an
instance/variant row; the "is it a new variant or an instance of the dense row-pointer"
call is the integrator's.**

The corollary's table (`spines.md:350-351`) lists two variants — dense row-pointer
(`FluxRecord`) and keyed rows (`FactLedger`). `SpeciesLayout` (`species.rs:256-371`) is a
**third shape**: a dense row-pointer over cells (`start: Vec<u32>`, length `n+1`) **plus a
per-cell `u64` presence mask**, with lookup by popcount rank —
`slot_of` (`species.rs:300-307`) is `start[c] + (mask & (bit-1)).count_ones()`, and the
row's axis codes are spelled out (`axis: Vec<u8>`) so iteration never decodes bits.

**And it is a clean A-4 discharge-by-porting, which the file's A-4 list should record as a
positive.** The module doc did the check A-4 prescribes, in writing (`species.rs:22-26`):

> CSR, the same shape `FluxRecord` and `FactLedger` already vindicated — and deliberately
> **not** a `Vec` per cell, which S19 § 6 measured at *"89 % of its 150 MiB heap in empty
> headers"*.

That is the fourth author reading the in-tree precedent instead of designing a fourth
layout — the outcome the row exists to produce. The corollary's closing warning
(*"A fourth author writing a fourth layout is the failure this row exists to prevent"*)
should be stamped: the fourth author came, and found it.

**Measured sparsity is available and unrecorded in `spines.md`:** `species_sparsity_probe.rs`
exists (`SpeciesLayout::{mean_row_width, max_row_width, occupied_cells, approx_bytes}` are
its readers). I did not run it and report no number.

---

### F8 — A-4's `split_by_shares` entry names a function that no longer exists

**Implicates: A-4.** **Disposition: stamp the entry; substance survives.**

`spines.md:1767` cites the discharge-by-extraction as
*"one `split_by_shares(total, shares) -> [f64; SPECIES]`"*. At `3cf8778` that function is
gone from production: `erosion/transport.rs:125-132` carries the tombstone
(*"`split_by_shares` LIVED HERE AND IS NOW `species::split_row_into`"*), and the successor is
`species::split_row_into(total, src_axis, src_vals, dst_axis, dst) -> f64`
(`species.rs:515-539`).

**The entry's substance held through the conversion, which is worth recording rather than
just correcting.** All three movers still route through one function —
`transport.rs:334`, `transport.rs:370`, `creep_kernel.rs:252` — and the two named anti-leak
tests survive on it (`creep_kernel.rs:541`, `:603`), now as thin wrappers
(`creep_kernel.rs:529-533`) rather than re-implementations. The residual rule is restated
unchanged in `species.rs:505-510`. **The only defect is the name and the signature in the
citation.**

---

### F9 — § 6 has fallen behind for the FOURTH time — three audits unlisted

**Implicates: A-4 committed inside the index for A-4.** **Disposition: backfill, and treat
the mechanism proposal as owed.**

`ls docs/audits/` at `3cf8778` against § 6's entries (`spines.md:2114-2207`). Unlisted:

- `2026-08-02-appearance-tour-p11.md`
- `2026-08-02-doc-topology-sweep.md`
- `2026-08-02-roadmap-staleness-sweep.md`

Plus **this file**, which will make four. Eleven → five → eight → three. § 6's own
2026-08-02 paragraph (`spines.md:2180-2185`) says the remedy is *"an enumeration a sweep with
a directory listing checks, not a list a human appends to"* — and then hand-appended, exactly
as its predecessor did. **This is now four consecutive sweeps writing the same finding and
four consecutive hand-fixes.** The proposal is unchanged and belongs to the main session:
either a check in the gate (a test that reads the directory) or a `wrap` step; a fifth
paragraph saying so is not the fix.

---

### F10 — Citation drift into `erosion/`, introduced by the split and then re-drifted by slice 2b

**Implicates: nothing structural.** **Disposition: refresh where the ref is a live
pointer.** The retarget at `ed0300a` was done against the pure-move tree; slice 2b then moved
those files again.

| `spines.md` | claims | at `3cf8778` |
|---|---|---|
| `:1091` (S-10) | `erosion/creep.rs:140` = `diffuse` | `erosion/creep.rs:149` |
| `:1092` (S-10) | `erosion/creep_kernel.rs:62` | ✅ `diffuse_net_cell` at `:62` |
| `:1092` | `…:129` | ✅ `diffuse_scale_cell` at `:129` |
| `:1092` | `…:191` | `diffuse_species_cell` at `:210` |
| `:1092` | `…:259` | `diffuse_outflux_faces` at `:274` |
| `:1889` (§ 3 `exhum` row) | `erosion/uplift.rs:86` | the `t_crust` read is `:89` |
| `:1525-1526` (A-2) | `erosion/creep.rs:115-125`, `:140`, `:570-574` | the corrected paragraph now spans `:124-134`; `diffuse` at `:149`; `:570` is inside the parallel-agreement test |
| `:1893` (§ 3 `chapters` row) | `tests/providers_common/mod.rs:545` | `:600` |

Everything else re-checked resolves: `runner.rs`, `field.rs`, `inventory.rs`, `collapse.rs`,
`cadence.rs`, `schedule.rs`, `contents.rs`, `packing.rs`, `column.rs`, `coarse.rs` are all
**absent from the delta's `crates/` diffstat**, so their refs are unmoved by construction.

---

### F11 — The salt fix is correct and complete; `draws.rs`'s residue enumeration is stale-but-harmless

**Implicates: A-2 (weak).** **Disposition: optional stamp.**

The `d8407e1` sweep's headline (the `DeepMember` ↔ `SALT_DT_PERTURB` collision on
`0x5900_0002`) was applied in the delta and applied well:

- `DeepMember` moved to `0x5900_0003` with a banner (`draws.rs:173-183`);
- `DeepTimePerturb = 0x5900_0002` registered at the value it already had (`draws.rs:184-192`);
- `refine.rs:29`'s hand-rolled `const SALT_DT_PERTURB` **deleted**; call site converted to
  `Draws::of::<DeepTimePerturb>(seed)` (`refine.rs:172,185`);
- byte-identity asserted (`the_registered_perturbation_domain_is_the_hand_rolled_salt`) and
  the collision made unrepeatable
  (`the_member_draw_and_the_perturbation_no_longer_share_a_stream`);
- `every_domain_is_listed_and_distinct` bumped 16 → 17.

**Residual:** `draws.rs:53` still says *"Two of the three local `const SALT_*` are gone with
their call sites."* There were **four** (`SALT_BIO_FIRE`, `SALT_BIO_FLOOD`, `SALT_DT_ROUGH`,
`SALT_DT_PERTURB`) — the enumeration was short by one when written, which is what the
`72fbe86` sweep's finding 4 said. It is now three-of-four gone and the sentence is merely
incomplete rather than false. **Search:** `grep -rn "SALT_[A-Z_]+" --include=*.rs
crates/dc-worldgen/src` at `3cf8778` → surviving consts are `grid.rs:34` (`SALT_DT_ROUGH`,
one live reader at `refine.rs:152`) and `tectonics.rs:51-53` (residue 3, unchanged and
honestly named).

*Cosmetic, noted once:* the new assert message in `draws.rs`'s byte-identity test carries a
wrapped string literal with a run of interior whitespace (*"moved the perturbation stream
at&nbsp;…&nbsp;seed {seed}"*). Harmless; mentioned only because it prints on failure.

---

### F12 — Corrections #90's rule is a spine-level statement and lives only in `corrections.md`

**Implicates: S-5.** **Disposition: user/integrator call — a rule change to a spine, which
§ 4 forbids an auditor from making.**

`corrections.md` #90 (`:3678-3681`) offers, explicitly *"not asserted"*:

> when a slice changes the *grade, type or units* of a quantity, the seams that publish it
> are part of the diff. A seam that survives a re-grade unchanged is either genuinely
> grade-agnostic (say so) or has quietly lost its customer.

S-5's rule block (`spines.md:602-605`) has the forward direction (*"the fallback must be an
identity"*, *"pass-level seams need an agreement test"*) and nothing about the converse.
**F1 above is that exact rule catching a second live instance one commit after it was
written** — which is the strongest available argument for promoting it. I am not promoting
it: the file's § 4 rule binds the auditor.

---

## 2. Categories actively checked that returned NULL

- **Deviations never made loud (§ 4).** Read § 4 in full. No new § 4 entry is owed by the
  delta's *ratified* work: the look-ownership ruling (`c11ee10`) records its DECIDED with the
  user's verbatim in `bodies.md`; the U1/U5 rulings are recorded; the golden re-capture is
  covered by the scratch-pad doctrine (§ Conventions) and by the geo session's GATE STATE
  block (corrections #92). **The one candidate is F2**, and it is a collision between a
  ratified design ask and a non-ratifiable anti-shape, which is a plea rather than a missing
  carve-out.
- **A-3 (a test green for a reason unrelated to what it asserts).** Checked the delta's new
  assertions in `species.rs` (5 tests), `draws.rs` (2), `creep_kernel.rs::creep_tests` (5),
  `dc-api/src/character.rs` (1) and `dc-api/tests/character_semantics.rs`. All assert on
  their stated mechanism, and two are notably well-shaped:
  `the_transport_closure_reaches_every_downstream_cell` asserts reachability on a synthetic
  chain (scale-free, correctly sized small), and
  `the_creep_split_is_linear_in_the_quantity_so_nothing_is_sorted` asserts an invariant
  (scale invariance) rather than a snapshot. **The `vanilla()`-only guards in `lithology.rs`
  are A-3's fixture form and are already filed — see F4.**
- **A-1 (a stand-in becoming the definition).** `Litho::reference_material`'s blast-radius
  entry was re-checked: the delta did not widen it. Its remaining production readers are
  `head.rs:253,281` (the basement fallback), `recorder.rs:224,568` (the S-5 deposit
  fallback), `lithology.rs:393`. The `d8407e1` sweep's "half discharged" stamp still reads
  true. **⚠ But see F2:** the delta added a *second* spelling of the same idea
  (`DEEP_BASEMENT`, `REFERENCE_MATERIAL`) beside `reference_material` rather than through it,
  which is worth the integrator's eye when A-1's entry is next touched.
- **A-5, A-6.** No delta instance found; both were read, neither is a shape the delta
  exercises.
- **Spines resting on a premise falsified by corrections #93 or #94.** Searched
  `spines.md` for `bob|look-at|gaze|sim-visible|quantizer|posture|body|bodies` (case-
  insensitive). **No spine or anti-shape row rests on either falsified claim.** The only
  bodies-arc rows are S-5's body-plan instance (`:578-586`), A-4's `vanilla_body_pack`
  discharge (`:1695-1707`), A-7's two B0 retirements (`:2094-2105`) and § 3's posture-bake
  row (F3) — none of them cites the temporal-bob framing or the look-ownership layer table.
  **This is a genuine null from an opened file.**
- **§ 5's `JUSTIFIED-BY` convention.** Not routed through it, per A-2's own pointer
  (`spines.md:1237-1243`: 3 corpus-wide, 0 in `crates/`). Confirmed unchanged in the delta:
  `grep -rn "JUSTIFIED-BY" --include=*.rs crates/` at `3cf8778` → **zero hits.** § 5 remains
  decorative and remains flagged as such.

---

## 3. Summary for the integrator

**Apply to `docs/spines.md`:**

1. **F3** — empty the resting-posture-bake § 3 row (CONSUMED, `f4e7f75`, `dc-client/src/body.rs:434`
   → `character.rs:426`). Count 19 → 18; re-run the header's command, do not re-type a number.
2. **F9** — backfill § 6 with the three unlisted audits **plus this file**.
3. **F10 / F6** — refresh eight `file:line` citations into `erosion/` and
   `tests/providers_common/mod.rs`.
4. **F8** — stamp A-4's extraction entry with `species::split_row_into`; note the substance
   survived the conversion.
5. **F7** — add `SpeciesLayout` to S-2's storage-corollary table and to A-4's
   discharge-by-porting list; stamp the *"a fourth author…"* warning as having been met.
6. **F4** — re-open the A-2 `Litho::of_material` instance as **partially applied**, pointing
   at `lithology.rs:1174-1176`.
7. **F11** — optional: stamp `draws.rs`'s "two of the three" residue sentence.

**Report to main session, do not apply:**

- **F1** — the `outcrop_shares` bedrock customer, three false comments, and A-7's own worked
  instance describing dead code. Behaviour question.
- **F2** — three named `MaterialId` constants vs A-7's non-ratifiable rule. **User-owned.**
- **F12** — corrections #90's converse as an S-5 rule. **User-owned** (§ 4 binds the
  auditor).

**§ 3 movement:** one row **emptied** (F3), **none added** (F5). 19 → 18.
