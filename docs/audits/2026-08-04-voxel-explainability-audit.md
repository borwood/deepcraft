# Voxel explainability audit — "can we ask every voxel WHY?"

**Desk audit, 2026-08-04. Read at commit `10ebbc8`** (worktree `agent-af54d62950eb82ef9`,
merged with `main` at that tip before reading). **No cargo was run**; every quantity quoted
is a pre-existing measurement, cited with its own source and date.

**The question** (user sketch, seeded 2026-07-19, live at `docs/design/knowledge.md:25-28`,
requirement 3 *"Computable anomaly"* — *"Every natural feature carries process provenance;
… 'What explains this?' is a real query; anomaly = no valid provenance match"*):

> "we should be able to ask every voxel WHY / HOW it has its current state, which is
> answered from the record. physical processes that are modeled leave history in the
> record."

The user's own framing: *"I don't know how true this currently is."* This audit measures it.

---

## 0. The verdict in one paragraph

**The sketch is roughly one-quarter true, and the true quarter is narrower than it looks.**
The deep-time recorder does carry a genuine, dense, per-bed provenance record — identity,
epoch, mover, depositional environment, biotic facies, aeolian facies, and an
erosional-gap flag, 30 bits per bed at `crates/dc-worldgen/src/deeptime/recorder.rs:446-460`
— and it is **retained in RAM for the life of the process**
(`crates/dc-worldgen/src/deeptime/field.rs:504-505`, `:716-727`). But three independent
walls stand between that record and "ask a voxel why":

1. **The expression path is a one-way funnel that drops almost all of it.** The
   collapse-tier `StrataEvent` (`crates/dc-worldgen/src/geology.rs:83-120`) keeps
   `member` + `thickness_m` + a read-out context, and carries **none** of the deep record's
   `chapter` / `mover` / `unconformity` / `biota` / `eolian` / `energy` / `env` axes; the
   per-voxel `VoxelContents` below it (`crates/dc-core/src/materials/contents.rs:104-112`)
   is 12 bytes of material slots with no provenance field at all.
2. **There is no query.** Zero of the 23 registry entries name a provenance verb; `dc-api`
   does not depend on `dc-worldgen` at all (`crates/dc-api/Cargo.toml:9-20`), so
   `DepUnit` / `StrataRec` / `Cause` / `FlowCause` are *structurally inexpressible* in any
   query result. See § 4 for the exact pathspecs.
3. **The record covers only the loose cover, and only thinly.** Mean `H` on the shipped world
   is **3.91 m** per land cell (`docs/audits/2026-08-02-p2-measurement-runs.md:80`,
   2026-08-02) against a mean land elevation of **517.4 m** (same line); the recorder logs
   every metre of it (`Σ(units) ≡ H` cell-for-cell, journal/0053:54-61) and **~1.15 m**
   survives as whole-voxel expressed units (journal/0053, quoted at
   `crates/dc-worldgen/src/geology.rs:841-842`). Everything below that is
   `has_contents: false` bedrock with **no record of any kind** — and generation is
   **unbounded downward**: no world-floor constant exists anywhere in
   `crates/dc-worldgen/src/**`, so the literal fraction of solid volume with no record
   tends to **100 % in the limit** (true, and useless — see § 5's honest framing).

And the largest single hole is not in the record at all: **all sub-460 m relief in the
shipped world is midpoint-displacement noise** (`docs/design/refinement.md:277-280`;
the draw at `crates/dc-worldgen/src/collapse.rs:1284-1290`). Whether a given voxel is rock
or air, below the deep grid's 460 m pitch, is answered by an addressed hash — honest
"statistics, deterministically", never by a modelled process.

---

## 1. Method and scope

Traced the expression path **backwards** from the voxel: `VoxelContents` ← `ColumnFill` ←
`StrataRec`/`StrataEvent` ← `run_strata` ← `SubCell` ← `DepUnit`/`DeepStrata`, plus the
non-record branches (fallback block, legacy soil, wilds, subaqueous, unrecorded basement).
For each class of voxel the shipped world contains, the table in § 3 states what the record
can answer about **what** (identity), **when** (time), **who** (cause / mover), **why here**
(spatial provenance), what is answered by **noise** (position-addressed draws), and what is
answered by **nothing**.

Two questions are held strictly apart throughout, because the gap between them is the
headline finding:

- **What the DATA could answer** — what is physically stored, somewhere, at some tier.
- **What any CODE currently asks** — what a consumer, test, probe or API actually reads.

CANNOT-DETERMINE is used where the code is genuinely ambiguous, and said so.

---

## 2. The expression path, traced backwards

### 2.1 What the deep record holds (the rich end)

`DepUnit` — `crates/dc-worldgen/src/deeptime/recorder.rs:490-493`, bitfield table at
`:446-460`. **8 bytes**, 30 of 32 bits used:

| bits | axis | answers | source |
|---|---|---|---|
| 0 | `env` (Subaerial/Subsea) | why here (marine vs. land) | `recorder.rs:247-252` |
| 1 | `aridity` | the paleoclimate at deposition | `:258-261` |
| 2–3 | `energy` (Low/Med/High) | the transporting flow's capacity | `:263-271` |
| 4–6 | `biota` (Mineral/Soil/Peat/Coal/Charcoal/Retro) | the living community, incl. paleosols and fire events | `:273-334` |
| 7–8 | `eolian` (None/Loess/Dune) | wind as a distinct transport agent | `:336-375` |
| 9 | `unconformity` | **that** time is missing here (§ 2.5) | `:650-656` |
| 10–15 | `species: MaterialId` | **what it is** — the rock, chosen by fitness *at deposition* | `:666-676` |
| 16–18 | `mover` (`FlowCause` 0..=6, or `MOVER_NONE`) | **who put it here** — fluvial/eolian/glacial/gravity/marine/hydrothermal/dissolution, or "made where it lies" | `:678-691`, `flux.rs:337-352` |
| 19–21 | `grain` | **UNSET on every unit today** (`GRAIN_UNSET = 7`, `recorder.rs:532-536`); FS-A's reserved write surface | `:693-700` |
| 22–29 | `chapter` (u8) | **when** — the tectonic chapter, 0-based; always `0` when tectonic history is off | `:658-664` |

Plus, per cell: `DeepStrata { units, stripped, strips, carry }` —
`crates/dc-worldgen/src/deeptime/recorder.rs:779-799` — ordered bottom-up, so **stratigraphic
order is itself a `when`**, and `strips` counts erosional events over the run (`:786-788`).

This is a real answer to *why*. Three of the five questions are directly stored (**what**,
**when** at chapter granularity, **who**), and two more axes (`env`, `energy`) are honest
*why-here* evidence.

**Ratified reading of the record's honesty** — the module header is explicit that units are
tagged by what was **measured** at deposition, never by interpretation
(`recorder.rs:1-19`). That is what makes it a record rather than a label.

### 2.2 What P11 proved actually survives (the walk)

`journal/0143-the-walk-where-the-rock-came-from-somewhere.md:53-64`, station 2, 2026-08-02:
a sandstone drape whose identity is an **import** — *"the record says sandstone because that
is what the river carried"* — with the counterfactual instrument
(`DeepConfig::identity_audit`, bit-inert, off in production) showing that the site's own
present-day climate would have drawn **conglomerate**.

**The measured size of that effect: 209,670.4 m — 41.31 % of transported metres, and
31.90 % of the ENTIRE record — is rock the site itself would have named differently**
(`docs/audits/2026-08-02-appearance-tour-p11.md:146`; journal/0141:179). Note the unit:
**metres of record**, not sites, not columns. User verdict, live: *"quite interesting and
also unremarkable, for reasons you implied."* Promoted to
`docs/design/things-that-will-happen.md:144-148`.

**What that proves, precisely:** *identity-by-transport survives the whole path to the voxel.*
The `species` bits reach the player's eyes. **It proves nothing about the other seven axes** —
see § 2.3, which is where they die.

### 2.3 Where the "why" is dropped — `deposit_deep_history`

`crates/dc-worldgen/src/geology.rs:723-803`. Per recorded unit it reads `u.species()` and
emits a `StrataEvent`:

```
ctx.strata.events.push(StrataEvent {
    member, thickness_m, temp_c, precip, depth_m,
    sel_salt, sel_tag, ore: None, accessory: None, dither: false,
});                                   // geology.rs:783-798
```

`StrataEvent` — `crates/dc-worldgen/src/geology.rs:84-120` — has **no field for**
`chapter`, `mover`, `unconformity`, `biota`, `eolian`, `energy` or `env`. Only two deep axes
survive, and both survive *reduced to a number*: `aridity` → `precip` via `deep_precip(u.tag())`
(`geology.rs:744`) and `chapter` → `temp_c` through the `paleo_temperature` provider seam
(`geology.rs:755-760`, tested at `:1126-1163`). The chapter *index itself* is consumed and
discarded at that line — it is used to look up a temperature and never stored.

Worse for **when**: adjacent units of the same member are **run-length coalesced**
(`geology.rs:776-782`), so two beds from different chapters, laid by different movers, merge
into one `StrataEvent` if they happen to resolve to the same registry member. Unit boundaries
— the stratigraphy — are partly destroyed here.

**This is the single highest-leverage fact in this audit.** The data exists one tier up and
is dropped by a `push` that has nowhere to put it.

### 2.4 Below `StrataEvent`: `ColumnFill` and `VoxelContents`

- `ColumnFill::build` (`crates/dc-worldgen/src/fill.rs:117-186`) slices the record into
  per-voxel `Plan::Single(event_idx)` / `Plan::Mixed(shares)`. It keeps an **index back into
  `rec.events`** — so at this tier a voxel still knows *which stratum* it belongs to. That
  back-pointer is real and is the natural hook for a future query.
- `ColumnFill::plan(depth)` returns **`None` below the record** — *"unrecorded basement"*
  (`fill.rs:188-194`).
- The near path then interns `VoxelContents` (`collapse.rs:655-690`); below the record it hits
  `None => continue` (`collapse.rs:671`) — **no contents are interned at all**. That `continue`
  is the mechanism behind `has_contents: false`.
- `dc_core::VoxelContents` = `{ shape, structure_len, pore_len, debris_len, slots: [MaterialId; 8] }`
  — `crates/dc-core/src/materials/contents.rs:104-112`. 12 bytes, `Copy`, interned. **There is
  no room for and no field carrying history.** This is the last stop before the player.

### 2.5 Erosion records ABSENCE, not what was removed

`DeepStrata::erode` (`recorder.rs:1005-1035`) pops quanta off the top and, when the record
empties, sets `stripped` and increments `strips`. The next deposit is flagged
`unconformity` (`recorder.rs:650-656`, `:826-830`). So the record honestly says *"time is
missing at this contact"* and **cannot say what was there**: the popped units are gone, and a
partial strip (`top.quanta -= q`) leaves no trace at all.

**State this as the record's honest lossiness, not a defect.** It is exactly how the real
rock record behaves — an unconformity is a readable gap, and what eroded is inferred from
elsewhere, never read off the outcrop. But it does bound the answer: *"why is this contact
here"* is answerable; *"what used to be above it"* is **NOTHING**, permanently, and the
unconformity flag does not even survive into `StrataEvent` (§ 2.3).

### 2.6 The veneer — designed-in year-zero, not a defect

The three collapse-phase passes (igneous emplacement, clastic veneer, placer) and the
weathering front select their members at **expression**, from a formation context sampled
**once per chunk at the chunk centre** — `crates/dc-worldgen/src/geology.rs:33-41`,
`StrataEvent::dither = true`. This is **ratified**: the veneer is still forming *now*, under
*this* climate, so year-zero climate is the correct formation context for it
(geology.md § formation context; restated in the P11 banner at
`crates/dc-worldgen/src/geology.rs:100-110`).

Stub #31 (`docs/design/stubs.md:1426-1462`) records the precise residue: **resolved for the
deep record's identity, live for the veneer's**, at a **28.8 m** step, with the heir
re-pointed on 2026-08-02 to *a per-column `climate_at` slice of its own* (banner 2,
`stubs.md:1444-1454`).

So the veneer's *why* is: **"the present climate here, sampled at the chunk centre."** That is
a real answer and a designed one. It is a *weaker* answer than the deep record's, because it
is a **re-derivation from current conditions**, not a history: a veneer voxel cannot tell you
*when* it arrived or *who* moved it, and it never could, by design.

### 2.7 The noise-answered parts

Every draw below is **ADDRESSED** — a pure function of `(seed, domain salt, position…)` —
so each is re-derivable and can honestly answer *"statistics, deterministically, and here is
the address"*. Domain registration is the `draw_domains!` table at
`crates/dc-worldgen/src/draws.rs:79-210`, which makes a duplicate salt a compile error
(`draws.rs:20-23`).

| draw | domain registered? | addressed by | what it decides |
|---|---|---|---|
| **move C — midpoint jitter** | ✅ `Elev = 0x5700_0004` (`draws.rs:87`) | `(level, i, j)` — `collapse.rs:1284-1286` | **all sub-460 m relief in the shipped world** (`docs/design/refinement.md:277-280`) — i.e. whether a voxel is rock or air |
| member dither (near, buried + surface) | ✅ `GeoDeep = …000E` / `GeoSelect = …000A` (`draws.rs:98-109`) | voxel column position, via `Octaves` (`draws.rs:419-470`) | which member of a class fills a column — **off for deep-record beds** (`dither: false`, `geology.rs:797`) |
| far surface class | ✅ `GeoClass = …0010` (`draws.rs:124`) | `(vx, vz)` via `Coherent`, stride 32 (`collapse.rs:1086-1092`) | the far tier's surface class |
| near record membership | ✅ `NearRecordMembership = …0012` (`draws.rs:151`) | voxel column, `Octaves` (`draws.rs:141-153`) | **which deep cell's record skins this column** |
| eighth allocation | ✅ `GeoFill = …000F` (`draws.rs:115`) | **world voxel position** (`draws.rs:110-114`) | which materials win a mixed voxel's leftover eighths |
| pore rider rounding | ✅ `GeoPore = …0011` (`draws.rs:139`) | voxel + **event index** (`draws.rs:128-138`) | a weathering-front rider's whole eighths |

**Two registered honest holes, both already documented and neither a world-correctness bug:**
tag space *inside* a domain is still hand-laid (`draws.rs:39-44`, hole 1), and
`deeptime/tectonics.rs` never entered the domain conversion at all — three salts under a third
numbering prefix with seven live `draw_f64` sites (`draws.rs:66-75`, found by the 2026-07-29
spine-audit).

**The scale of the noise share is what matters here.** Move C is not a garnish: it is the
entire vertical structure of the world between 460 m and 0.9 m. A voxel's *existence* — solid
or air — is a noise answer for everything finer than the deep grid's pitch
(`DEEP_CELL_M = 460.0`, `crates/dc-worldgen/src/deeptime/field.rs:46`). Refinement.md says so
in its own words: *"All sub-460 m relief in the shipped world comes from move C, which § 3's
kernel list does not name"* (`docs/design/refinement.md:279-280`) — a coarse→fine move with
**a named heir and no owner yet**.

### 2.8 The flux record (P3) — built, and nothing asks it

`FluxRecord` — `crates/dc-worldgen/src/deeptime/flux.rs:422-441`: per-cell, per-chapter
directed flux on 3D faces, each entry carrying a `FlowCause` (`flux.rs:418-419`) and a
`FluidId` (`flux.rs:354-370`). Residency ~40 MiB on the production world
(`docs/spines.md:2134`).

**Production consumers: zero, on purpose.** `docs/spines.md:2134` — *"no production consumer,
ON PURPOSE … only `tests/flux_record.rs`, `tests/head_field.rs` and
`examples/flux_record_probe.rs`. Expressing it now would mean a second fake river beside the
honest absence of one."* Re-confirmed uncalled through the 2026-08-03 spine-audit sweep
(`docs/spines.md:2101`). Its sibling `DeepField::head` is in the same state
(`docs/spines.md:2135`).

**What it could answer that nothing asks:** *by which face, in which chapter, driven by which
mover, did material enter or leave this cell* — i.e. the routing history that would let a
voxel say "the river that brought me came from **that** direction, in chapter 3, and it had
already avulsed twice." It is the only artifact in the tree that holds a **directional,
time-indexed** cause. Its named heirs are the channel operator and refinement-as-a-boundary-
value-problem (`docs/spines.md:2134`, `docs/design/refinement.md:290-300`).

### 2.9 The far tier and the cold tier

- **Far tier**: the surface class is drawn from `CoarseField<ShareVec<DEEP_CLASSES>>` —
  per-deep-cell **metre shares of the top 0.9 m**, six slots (`collapse.rs:1798-1821`,
  `deep_class_slot` at `:1745-1764`). It answers **what, coarsely, at the surface only** —
  and it coarsens `MaterialId` back to a `Litho` class to do it. The doc comment marks it
  explicitly as **rejected-interim**: *"a fixed-6 share vector whose blend semantics the user
  already rejected by eye (2026-07-29) and which rides as interim"* (`collapse.rs:1746-1753`).
  It answers **nothing** about when, who, or why-here.
- **Cold tier**: **CANNOT-DETERMINE from this desk pass.** No tier by that name was located in
  `crates/dc-worldgen/src/` under the pathspec `crates/**/*.rs` with pattern `(?i)cold.?tier`.
  What exists beyond the near/far split is the **border wilds** — beyond the pregen grid, no
  deep-time run at all (`geology.rs:170-176`, `collapse.rs:846-849`) — treated as its own
  class in § 3.

### 2.10 How much of the world is recorded — the measured numbers, and their caveat

**⚠ The number everyone quotes is depth-truncated, and neither of its two source documents
says so in its summary line.** Recorded in this audit so it stops being misquoted.

| quantity | value | source | date |
|---|---|---|---|
| solid voxels examined (169 columns × top 65 voxels) | 10,985 | `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md:310` | 2026-07-25 |
| **recorded mixture (solid)** | **6,072 = 55.3 %** | same `:313`; re-measured through the real query path, `journal/0101:216-226` | 2026-07-25 |
| **UNRECORDED (solid, `has_contents:false`)** | **4,913 = 44.7 %** *(after the #49 fix; 38.3 % + 6.4 % phantom before it)* | `journal/0101:216-226` | 2026-07-25 |
| **⚠ the depth window that produced both** | **`h-64 ..= h` — the top 65 voxels ≈ 58.5 m only** | `crates/dc-worldgen/examples/contents_air_over_solid_probe.rs:177`; `crates/dc-client/examples/identify_census.rs:52` | — |
| mean regolith `H`, shipped | 3.91 m (was 4.57 m pre-journal/0122) | `docs/audits/2026-08-02-p2-measurement-runs.md:80` | 2026-08-02 |
| mean surface elevation, land | 517.4 m (relief 1286.5 m; 44,253 land cells) | same line | 2026-08-02 |
| mean recorded column, all deep cells | ~1.36 m (~25 units/cell, mean unit 0.053 m) | `docs/audits/2026-08-03-stratigraphic-correlation-design.md:84-88` | 2026-08-03 |
| deep grid | 545² = **297,025** cells @ 460 m = 250.7 km; **8 chapters** | same `:85` | 2026-08-03 |
| land share of the deep grid | 44,253 / 297,025 ≈ **14.9 %** | derived from the two rows above | — |
| land cells expressing **zero** voxels ("bare") | 91 / 44,265 = **0.2 %** | journal/0053:41 | 2026-07-22 |
| land columns with **no record at the surface top span** | 22 / 4,792 = **0.5 %** — **likely superseded**, see below | `docs/audits/2026-08-02-appearance-tour-p11.md:44` | 2026-08-02 |
| intrusive basement top (a *veneer* pass, orogeny/arc only) | 96 voxels = **86.4 m** | `crates/dc-worldgen/src/geology.rs:332` | — |

**Three integrity notes on those numbers, all found by this audit's desk sweep:**

- **The 55.3 / 44.7 split is the top 58.5 m of a ~517 m mean column.** Quoting it as a world
  figure is a category error. Note also *why* it reads 55.3 % rather than the ~0.8 % that
  `3.91 m ÷ 517.4 m` would suggest: the top-65 window is dominated by the **veneer** and the
  86.4 m intrusive basement band, which are expressed records with *no history* (§ 3). **A
  high "recorded" share is not a high "explainable" share.**
- **Two live docs disagree on the total unit count** — journal/0136:280 measures
  **10,951,030 units / 404,898.8 m**; `docs/audits/2026-08-03-stratigraphic-correlation-design.md:85-86`
  states **~7.4 M units** over the same 297,025 cells, a 48 % gap. Metre totals roughly agree.
  Neither document points at the other. **Flagged, not resolved here** — outside this audit's
  question, but it is a spike-banner-shaped debt.
- **The 0.5 % "no record" column share predates the mechanism that would move it most.**
  `journal/0149-there-are-no-deep-cells.md:36-44` (2026-08-03) reports bench-cut columns that
  are `has_contents: false` **floor-to-rim** beside fully-recorded neighbours, because the
  slice-3 membership dither transplants record *extent* along with identity
  (`ROADMAP.md:2970-2977`). Nobody has re-measured since, and slice 3 is itself retired-as-plan
  (`journal/0149:87-93`). **Treat 0.5 % as stale.**

---

## 3. The table: voxel class × what the record can answer

Classes are ordered roughly by descending explainability. **noise-share** names what the
class's state owes to an addressed draw; **NOTHING** names what no artifact anywhere holds.

| voxel class | WHAT (identity) | WHEN (time) | WHO (cause/mover) | WHY HERE (spatial) | NOISE-answered | NOTHING |
|---|---|---|---|---|---|---|
| **Deep-recorded bed** (buried, near tier, inside the pregen grid) | ✅ **recorded** `MaterialId`, fitness run at deposition — `recorder.rs:666-676`; survives to the voxel, proven by the 31.90 %-of-record counterfactual, journal/0143:53-64 + `docs/audits/2026-08-02-appearance-tour-p11.md:146` | ⚠ **in the deep record only**: `chapter` u8 + stratigraphic order (`recorder.rs:658-664`). **Dropped at expression** — no `chapter` field on `StrataEvent` (`geology.rs:84-120`); consumed at `geology.rs:755-760` to fetch a temperature and discarded | ⚠ **in the deep record only**: `mover` = `FlowCause`\|`MOVER_NONE` (`recorder.rs:678-691`). **Dropped at expression** — same line | ⚠ `env`/`aridity`/`energy`/`eolian`/`biota` in the record (`recorder.rs:379-391`); only `aridity`→`precip` survives (`geology.rs:744`) | which *member of the class*: **none** for deep beds (`dither: false`, `geology.rs:797`). The **eighth split** inside a mixed voxel: `GeoFill` (`draws.rs:110-115`). Which **deep cell** skins the column: `NearRecordMembership` (`draws.rs:141-153`) | The **elevation** the bed sits at, below 460 m: move C. **Grain size** — `GRAIN_UNSET` on every unit today (`recorder.rs:532-536`). Sub-cell *lateral* position within the 460 m footprint. Whether two coalesced beds were once distinct (`geology.rs:776-782`) |
| **Surface voxel over a record** (near tier) | ✅ the record's top span through `ColumnFill` (`collapse.rs:846-849` notes the surface branch reads the record, not `surface_class`) | ⚠ as above — chapter of the top bed, in the deep record only | ⚠ as above | ⚠ as above | member dither at the surface: `GeoSelect`/`Octaves` — and journal/0143:18-30 measured it **currently invisible at every surface of the shipped world** (0 of 4,792 land columns have a `Single` top span) | as above |
| **Veneer voxel** (clastic veneer, placer, igneous emplacement, weathering front) | ✅ member selected at expression from the **year-zero** formation context — ratified (`geology.rs:33-41`; stubs.md:1435-1442) | ❌ **NOTHING** — "now", by construction. No epoch is stored and none is meaningful | ⚠ **partial**: the *pass* is implied by the class, and the placer's sorting energy is in `AlluviumRec` (`geology.rs:196-201`); no `FlowCause`, no agent id | ⚠ the chunk-centre climate sample + `flow_energy` + `Provenance` (`geology.rs:141-186`) — but frozen at a **28.8 m** step (stub #31, `stubs.md:1437-1441`) | `GeoSelect` member draw, `GeoThick` thickness draw (`geology.rs:366-368`), `GeoOre`, `GeoAccessory`, `GeoPore` | **when** and **who**, entirely. And the 28.8 m step in the fitness *thresholds* (stub #31 residue) |
| **Unrecorded basement** (below the record — the bulk of the solid world) | ❌ **NOTHING** at contents grade: `plan()` → `None` → `continue`, no `VoxelContents` interned (`fill.rs:188-194`, `collapse.rs:671`). Only a `Block::Stone` (`collapse.rs:533`). `DEEP_BASEMENT = MaterialId::GRANITE` (`lithology.rs:728`) is the *deep sim's* basement constant, not an expressed identity | ❌ NOTHING | ❌ NOTHING | ❌ NOTHING beyond the pregen cell's `Provenance` — which is an **input**, never returned (agent sweep § 3; `pregen/mod.rs:97-118`, `:141`) | its **elevation**: move C | **Everything.** This is `has_contents: false` over solid (CLAUDE.md § Agent walks; `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`) |
| **Igneous basement band** (orogeny/arc/rift provinces only) | ✅ a member, but selected at **year-zero** from `Provenance` (`geology.rs:337-379`) — 96 voxels ≈ **86.4 m** thick (`INTRUSIVE_TOP_VOX = 96`, `geology.rs:332`) | ❌ NOTHING — no emplacement epoch | ⚠ the *province* (`Provenance::Orogeny\|Arc\|Rift`), which is a setting, not a mover | ✅ the pregen cell's tectonic province — the strongest spatial answer in the veneer | `GeoSelect` tag 0/1, `GeoThick` tag 1, `GeoAccessory` | when, who; and below the band, § "unrecorded basement" |
| **Legacy soil band** (no deposition record — ocean floor, wilds) | ⚠ `Block::Dirt` only, depth = the cell's `H` in whole voxels (`collapse.rs:511-517`, `subcell.rs:66-70`). **No contents interned** | ❌ NOTHING | ❌ NOTHING | ⚠ only that `H > 0` at the parent cell | its thickness follows `H` (a sim quantity), its **elevation** is move C | identity beyond "dirt"; everything else |
| **Ocean floor / subaqueous** | ❌ fallback block only: `Dirt` above −35 m, `Stone` below (`collapse.rs:864-873`); `clastic_pass` returns early below sea level (`geology.rs:819-821`), `igneous_pass` likewise — *"sea floor keeps its unrecorded basement in v1"* (`geology.rs:339`) | ❌ NOTHING | ❌ NOTHING | ⚠ a depth threshold, nothing more | elevation: move C | **Everything.** An explicit v1 hole, marked in code |
| **Border wilds** (beyond the pregen grid) | ❌ fallback block; `regolith_m: None`, `deep_units: &[]` (`geology.rs:170-176`) — the fallback `SubCell` has `cell: None` (`subcell.rs:36-39`) | ❌ NOTHING — no deep-time run out there (stubs.md § Genesis) | ❌ NOTHING | ⚠ the analytic lattice value only (`collapse.rs:1300-1310`) | elevation: move C, **unmodified** — the deep-surface override is skipped where the sample is absent (`collapse.rs:1300-1310`) | **Everything** |
| **Transplant-hole column** (a column the membership dither pointed at a record-less deep cell — `has_contents: false` floor-to-rim beside a fully recorded neighbour, `journal/0149:36-44`, `ROADMAP.md:2970-2977`) | ❌ NOTHING — the *dither* chose a source cell that had no record; the column inherits the absence | ❌ NOTHING | ❌ NOTHING | ⚠ **the answer is a draw**: `NearRecordMembership` picked this cell (`draws.rs:141-153`) — the honest *why-here* is "the membership dither addressed it here", which is not a physical cause | **the whole class exists because of a noise decision** | everything — and, distinctively, the neighbouring column *does* have a record, so the absence is not a property of the place |
| **Far-tier surface sample** (what a distant horizon shows) | ⚠ a **class**, drawn from top-0.9 m metre shares, coarsened `MaterialId`→`Litho`, 6 slots (`collapse.rs:1798-1821`, `:1745-1764`) — **rejected-interim** (`collapse.rs:1746-1753`) | ❌ NOTHING | ❌ NOTHING | ⚠ the deep cell (bilinearly dithered — the "cake law", `collapse.rs:1040-1050`) | `GeoClass` membership + class draw, coherent source stride 32 (`collapse.rs:1086-1092`) | everything below the top 0.9 m; when; who |
| **Air** | ✅ honestly `has_contents: true` with an empty composition (CLAUDE.md § Agent walks) | n/a | n/a | ⚠ "the surface is below me" — and that surface is move C below 460 m | its **boundary**: move C | why the surface is *at that height* at sub-460 m scale |

**Reading the table:** the only row where **WHO** and **WHEN** are stored at all is the first
two, and in both the answer lives **one tier above the voxel** and is destroyed on the way
down. No row has a **WHO** or **WHEN** answer that any code can currently reach from a voxel.

---

## 4. The query side — there is no query

**`knowledge.md:27` wants the query explicitly: *"'What explains this?' is a real query."*
It does not exist.** The search that establishes this, with exact pathspecs so the absence is
citable:

1. `crates/` — `grep -rniE '"dc:[a-z]+/(explain|why|provenance|history|origin|lineage|cause|audit|attribut)'` → **zero matches**. No registry id names a provenance verb.
2. `crates/dc-mcp-dev/src/` — `(explain|provenance|lineage|history|why)` → **zero matches** in source.
3. `crates/**/*.rs` — `grep -rniE "fn +(explain|why|provenance|history|origin|lineage|audit|attribut)[a-z_]*\s*\("` → 9 hits, **none query-shaped** (console line-editing history; `Bound::provenance()` for joint limits at `crates/dc-api/src/bodies/limits.rs:116-123`).
4. `grep -rn "StrataRec\|DepUnit\|SubCell" --include=*.rs crates/` → **zero hits in `crates/dc-api/`, `crates/dc-core/`, `crates/dc-client/src/`, `crates/dc-mcp-dev/`**.

The registry is a single closed macro table — `crates/dc-api/src/schema.rs:412-818`, macro at
`:354-410` — and the MCP tool list is generated from it (`crates/dc-mcp-dev/src/lib.rs:48`),
so **the absence is structural, not inferred**. All 23 entries: 8 queries
(`get_block`, `get_contents`, `scan_region`, `entity/query`, `events/poll`, `character/pose`,
`sense_raycast`, `sense_surroundings`) and 15 commands. None is provenance-shaped.

**`world_get_contents` returns exactly:** `block`, `classified`, `has_contents`, `contents`
— `crates/dc-api/src/payload.rs:442-461`; `ContentsView` = `shape`, `solid_eighths`,
`free_eighths`, `open_pores`, `free_debris_eighths`, `structure: Vec<MaterialCount>`,
`pore_fill`, `debris` (`payload.rs:359-376`). **No provenance, no time, no cause, no epoch.**
The client's look-at HUD prints the same fields and nothing more
(`crates/dc-client/src/inspector.rs:140-161`).

**And the data IS there, resident, at runtime.** This is the finding that turns "no query"
from a data problem into a *plumbing* problem:

- `DeepField.strata: Vec<DeepStrata>` — `crates/dc-worldgen/src/deeptime/field.rs:504-505`;
  the finalize comment states the record is *"append-only during the compile and read-only
  forever after"* (`field.rs:716-727`), `shrink_to_fit` rather than freed, ~58 MiB on the
  production world (`recorder.rs:441-442`).
- The client holds it: `Arc<Pregen>` in `SurfaceAuthority::Worldgen` —
  `crates/dc-client/src/authority.rs:209-233`.
- The collapse-tier product is retained too: `ColumnRec.records: Vec<SubCell>`
  (`collapse.rs:212`) with per-column index `cell_of` (`:219`) and `record_for(lx, lz)`
  (`:241`), in the runtime-resident `column_cache` (`collapse.rs:336`). And `SubCell` knows
  **which deep cell it realized** — `cell_index()`, `subcell.rs:73-76`, written for exactly
  this kind of instrument.
- **The wall:** `crates/dc-api/Cargo.toml:9-20` lists `dc-core, glam, postcard, serde,
  serde_json, thiserror`, with the comment *"dc-sim / dc-worldgen … are NOT deps here"*. So
  `DepUnit` / `StrataRec` / `Cause` / `FlowCause` are **inexpressible in any query result** as
  the crate graph stands.
- The only cross-crate reader of `column_record` reads `col.heights[…]` and nothing else
  (`crates/dc-client/src/authority.rs:459-460`, `:498-499`, `farmesh.rs:2200`, `:2367`).

**One more designed-but-unread provenance surface**: `Cause`
(`crates/dc-worldgen/src/deeptime/inventory.rs:395-407`) — `Chemical/Biotic/Frost/Dissolution`
on a weathering `Fact` — whose doc comment already forward-notes an `Actor(ActorId)` variant
for *"player P deposited it"* (`:380-393`), and whose `name()` is documented as *"for probe /
provenance output"* (`:410`). The provenance ledger is **designed** and has **no read
surface**.

---

## 5. Verdict, and the three biggest holes ranked

### The verdict

**"We can ask every voxel WHY" is currently FALSE, in three independent ways, and the sketch's
second sentence — *"physical processes that are modeled leave history in the record"* — is
the half that is TRUE.** Modelled deep-time processes do leave a genuine, measured,
non-interpretive history: identity, epoch, mover, environment, biota, wind, and erosional
gaps, 8 bytes a bed, retained for the life of the process. What is false is everything about
*asking*: no query exists, the answer is destroyed on its way to the voxel, and most of the
world's solid volume was never in the record to begin with.

A fair single number, stated with its own caveats: of the five questions
(what / when / who / why-here / — ), **a deep-recorded voxel can answer 1 of 5 at the voxel
(what), 4 of 5 one tier up in RAM (what/when/who/why-here), and 0 of 5 through any API.** For
every other voxel class it is 0 or 1 of 5 at every tier.

### The three biggest holes, ranked

**1. THE FUNNEL — `deposit_deep_history` drops seven of the record's eight axes, and
`StrataEvent` has no fields to catch them.** `crates/dc-worldgen/src/geology.rs:783-798`;
`StrataEvent` at `:84-120`. Ranked first because it is the *cheapest* hole to close and the
one that blocks all the others: the data is already computed, already correct, already
ratified as measured-not-interpreted, and it dies in a struct literal. Until it is caught, no
query — however well plumbed — can answer *when* or *who* about a voxel, because the near
tier no longer knows. Aggravated by the run-length coalescing at `:776-782`, which merges
beds from different chapters and different movers whenever they resolve to the same member —
so the loss is not merely of annotation but of **stratigraphic structure**.

**2. THE ABSENT QUERY, and the crate wall behind it.** `knowledge.md:27` asks for the query
by name; zero of the 23 registry entries provide it (`schema.rs:412-818`), and `dc-api`
cannot name the types that would answer it (`crates/dc-api/Cargo.toml:9-20`). Ranked second
because the *data* survives in RAM (`field.rs:504-505`, `collapse.rs:212-241`) and
`SubCell::cell_index()` was built for exactly this (`subcell.rs:73-76`) — this is a plumbing
and layering decision, not a modelling one. Note it is genuinely a **design** question, not a
chore: the honest options are a `dc-api` query that speaks a *serialized provenance view*
(not worldgen types), or a dev-only MCP surface, and choosing wrongly imports `dc-worldgen`
into the wire layer.

**3. THE UNRECORDED BULK, and the noise beneath it.** Two facts that compound:
*(a)* the record covers the **loose cover only** — mean `H` **3.91 m** per land cell against a
mean land column of **517.4 m** (`docs/audits/2026-08-02-p2-measurement-runs.md:80`), of which
**~1.15 m** expresses as whole-voxel units (journal/0053, quoted at `geology.rs:841-842`) —
and generation is **unbounded downward** (no world-floor constant in
`crates/dc-worldgen/src/**`), so the overwhelming majority of solid world volume is
`has_contents: false` bedrock with **no record of any kind** (`fill.rs:188-194`,
`collapse.rs:671`). The 55.3 % "recorded" figure people quote is the **top 65 voxels only**
(§ 2.10), and most of what it counts is veneer and the 86.4 m intrusive band — expressed
records with no history; and
*(b)* **all sub-460 m relief is move C**, an addressed midpoint-displacement draw
(`refinement.md:277-280`, `collapse.rs:1284-1290`), so whether a voxel *exists as rock at all*
is a noise answer below the deep grid's 460 m pitch (`field.rs:46`). Ranked third only because
closing it is genuinely expensive and partly *correct*: move C is honest addressed statistics
with a named heir and no owner (`refinement.md:279-280`), and the bedrock hole is what the
refinement tier and the members-into-deep-history arc exist to attack. **But it bounds the
sketch's ceiling**: no amount of query plumbing makes a basement voxel explainable, because
nothing ever recorded it.

### The gap between DATA and CODE, stated plainly

| | data holds it | any code reads it |
|---|---|---|
| bed identity (`species`) | ✅ | ✅ — expressed, walked, verdicted (journal/0143) |
| bed epoch (`chapter`) | ✅ `recorder.rs:658-664` | ⚠ **one** reader: the `paleo_temperature` provider key (`geology.rs:755-760`), which discards it |
| mover (`FlowCause`) | ✅ `recorder.rs:678-691` | ❌ **nothing** reads `DepUnit::mover` outside tests/probes |
| unconformity | ✅ `recorder.rs:650-656` | ⚠ recorder-internal merge logic + `unconformities()` count (`:1046-1048`); no expression consumer |
| biota / eolian / energy / env | ✅ `recorder.rs:379-391` | ⚠ `aridity`→`precip` only (`geology.rs:744`); `biota` reaches the far tier as a *class* (`collapse.rs:1745-1764`) |
| grain | ❌ **`GRAIN_UNSET` on every unit** (`recorder.rs:532-536`) | n/a — FS-A's reserved write surface |
| face-flux history (`FluxRecord`) | ✅ ~40 MiB, per-chapter, per-face, with `FlowCause` (`flux.rs:422-441`) | ❌ **zero production consumers, on purpose** (`docs/spines.md:2134`) |
| head field | ✅ (`docs/spines.md:2135`) | ❌ exported plane uncalled; in-sim value consumed |
| weathering `Cause` | ✅ `inventory.rs:395-407` | ❌ *"for probe / provenance output"* — no read surface |

**Nine rows. One is fully consumed.** That asymmetry *is* the audit's finding: this project
has built substantially more provenance than it has ever asked a question of.

### What this audit does NOT establish

- **The exact fraction of solid world volume that answers "no record"** — **CANNOT-DETERMINE**,
  and for a structural reason, not a missing-probe one: **there is no world floor.**
  `generate_chunk` (`collapse.rs:460-533`) is unbounded downward and every voxel below the
  record is unrecorded `Block::Stone`, so the literal fraction tends to 100 % and the question
  has no denominator until a vertical extent is ratified. What *could* be measured, and is
  not: a per-column census of recorded vs. unrecorded voxel spans **to a stated floor** (both
  existing probes stop at `h-64` — a one-constant change:
  `contents_air_over_solid_probe.rs:177`, `identify_census.rs:52`), and a distribution rather
  than a mean of expressed record depth, separating deep-time `H` from geology-pass events.
  Note the two available numbers cannot be reconciled without it: `3.91 m ÷ 517.4 m ≈ 0.8 %`
  against the top-65 window's 55.3 % — the gap is the veneer plus the 86.4 m intrusive band,
  and **no doc attempts the reconciliation**. **Stated as a gap rather than estimated**, per
  the measure-against-the-literature doctrine's sibling rule.
- **A re-measurement of the "no record at all" column share after slice 3.** The only figure
  (0.5 %, `docs/audits/2026-08-02-appearance-tour-p11.md:44`) predates
  `journal/0149:36-44`'s floor-to-rim no-record columns.
- **The 10.95 M vs ~7.4 M recorded-unit-count discrepancy** between journal/0136:280 and
  `docs/audits/2026-08-03-stratigraphic-correlation-design.md:85-86`. Flagged in § 2.10;
  resolving it is outside this audit's question and owed to whoever holds those two docs.
- **A "cold tier"** — no artifact of that name was found under `crates/**/*.rs`; the
  third regime beyond near/far is the **border wilds**, audited as its own class in § 3.
- Whether any of these holes *should* be closed, or in what order. This audit measures; it
  does not sequence.
