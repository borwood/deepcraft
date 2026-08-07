# Deep-time pass I/O and north-star compliance — the unified table

**Base commit:** `0c4809c` (`field and pass shape: consolidation notes from the 2026-08-06
readings`). Worktree ran `git merge main` (*Already up to date*) and `git log -1 --oneline main`
matched the tip before any work.

**Method.** Source reading only. **No cargo was run**, no build slot taken. Every claim below
carries a `file:line` or the command that produced it. This is a **unification** of three merged
readings plus gap-filling; where I re-verified a prior audit's claim at source I say so, and
where a prior audit was wrong or two disagreed I say that too (§ 6).

**Inputs unified.**
`docs/audits/2026-08-06-field-inventory-water.md` ·
`docs/audits/2026-08-06-field-inventory-solid-earth.md` ·
`docs/audits/2026-08-06-field-inventory-climate-surface.md` ·
`docs/audits/2026-08-06-field-and-pass-shape-notes.md`.

**Write-set: this file only.** Nothing else was edited; no ordinal was taken in any numbered
inventory. Findings are reported, not filed.

---

## 0. What the shipped world runs — read from the production config, not `default()`

`DeepConfig::default()` is **not** what the game runs. `field.rs::production_config_base`
(`crates/dc-worldgen/src/deeptime/field.rs:276-381`) is: it sets `record: true` (`:284`),
`biotic: true` (`:292`), `erodibility: true` (`:303`), `tectonic_history: true` (`:321`),
`full_agents: true` (`:342`), `calibrated_rates: false` (`:378`), then `..DeepConfig::default()`
(`:379`) — which supplies `weather_inventory: false` (`grid.rs:607`), `flow_record: true`
(`grid.rs:608`), `head_field: true` (`grid.rs:609`), `remarch_interval: 20` (`grid.rs:570`).

**So the shipped roster is 17 passes**, and it is exactly the 17 of the test constant
`PRODUCTION_ORDER` (`runner.rs:1185-1220`) — note that constant is built from
`all_on()` = `DeepConfig::default()` + four flag flips (`runner.rs:1173-1183`), which coincides
with production only because production flips the same four. *It is a test's model of production,
not production's own statement of itself.* I checked the two configurations agree field by field
on the six flags that gate pass presence; they do.

**The 18th registered pass, `dc:deep/weather_inventory`, does not run in the shipped world.**
It is gated at `runner.rs:996` on `cfg.weather_inventory`, which falls through to `false`. The
only way to turn it on is `DeepOverrides::weather_inventory` (`field.rs:243`, applied `:443-444`),
and the single caller is the client's `--weather-inventory` flag (`dc-client/src/main.rs:159`).

---

## 1. Coverage — and the pass no prior audit reached

**18 registered deep-time passes**, enumerated from `declared_passes` (`runner.rs:696-1015`):
`climate` · `tectonics` · `forcing` · `expose` · `frost` · `drainage` · `transport` · `head` ·
`flow_record` · `weather` · `diffuse` · `isostasy` · `geotherm` · `deposition` · `eolian` ·
`wave` · `biotic` · `weather_inventory`.

> ### ⚠ LOUD FINDING — `dc:deep/deposition` was covered by NO prior audit.
>
> **Absence claim with its search.** I ran `grep -rn "deposition" docs/audits/2026-08-06-field-inventory-*.md`
> over the three merged readings: **4 hits, and not one of them is a pass entry.** Three are the
> word used generically (`erosion/record.rs:288` cited as a *reader* of `air_temp_c` in the
> climate audit `:64`; "depositional history" in solid-earth `:155`; "pickup and deposition
> closures" for eolian in water `:333, :402`). The solid-earth group took tectonics/forcing/
> expose/diffuse/isostasy/geotherm; water took drainage/transport/head/flux/eolian/wave; climate
> took climate/frost/biotic/weather/weather_inventory. **The strata recorder — the pass that
> creates `DeepAxis::Recorded`, writes every `DepUnit` in the world, and is the sole in-loop
> author of the record the collapse tier actually expresses — fell in the partition gap between
> three groups and was audited by none of them.**
>
> It is not a marginal pass. It is the one whose output *is* the shipped artifact: `DeepField::strata`
> (`field.rs:505`), read in production by the collapse tier. That the three-way partition lost it
> is itself the finding — the partition was drawn by *physical domain* (water / solid earth /
> climate) and deposition is the pass that belongs to all three.

Two lesser coverage gaps, both noted rather than loud:
- **`forcing`'s legacy arm** (`forcing_legacy_pass`, `runner.rs:370-373`) was covered by nobody;
  solid-earth audited the tectonic arm only. It does not run in the shipped world
  (`tectonic_history: true`), so this is a completeness note.
- **`expose`** was audited by solid-earth but its own author flagged it as mis-filed
  (solid-earth `:141-144`), and the climate agent independently flagged it as unassigned
  (climate `:431-437`). Both were right; it is covered, by an agent who said it should not be.

---

## 2. The compliance test being applied

The user's ruling, 2026-08-06, quoted:

> **"Field passes adjust a field (a world vector space); cell passes read the field when marching
> over cells, generally."**
>
> **"I can't imagine a field pass touching materials at all except reading the record — never
> changing them."**

**The working invariant.** A **field pass** may read fields, write fields, and **read** the
record; it may not change materials or write the record. A pass that **changes cell contents is a
cell pass**, which reads fields while marching cells.

**One interpretive call has to be made before the rule can be applied, and it decides half the
verdicts, so I state it rather than smuggle it.** In this tree the terrain planes `grid.r`
(bedrock) and `grid.h` (regolith) are **per-cell f64 scalars — fields, not materials**. They carry
no identity: the material a column is made of lives in `grid.strata` (the record) and in the fact
ledger. **So a pass that moves metres of `R` and `H` around is adjusting a field, and is a field
pass under the rule as written** — even though in ordinary speech "it erodes rock" sounds like a
material operation.

That reading is not mine alone; the code already argues it. `weather_inventory`'s own doc calls
the split explicitly: it *"READS the terrain (regolith cover shielding) but WRITES ONLY the ledger
sidecar — it never touches `R`/`H`"* (`runner.rs:485-489`), naming `R`/`H` as the *other*
authority from the material record. The runner says the same of the geotherm and the head field
— *"plants a field and runs no edges"* (`runner.rs:456-458`, `:546-548`).

**If the user meant the opposite — that moving rock mass is a material act — then `transport`,
`weather`, `diffuse` and `isostasy` flip from compliant field passes to violators, and the split
becomes a nearly useless partition (almost every pass would be a cell pass).** I flag this as the
one place where my verdicts are contingent on an unratified reading, and I recommend it be ruled
on explicitly rather than inherited from this document.

---

## 3. The table

Columns are fixed and identical for every row. `n = 297,025` cells at 460 m on the shipped Medium
world (derivation: `Extent::Medium` ⇒ pregen `wp = 17`; `extent_m = 17 × 16384 × 0.9 = 250,675.2 m`,
`field.rs:277-279`; `cell_m = max(250675.2/550, 460) = 460.0`, `field.rs:46,50`; `w = round(250675.2/460) = 545`,
`grid.rs:804-805`). One f64 plane = 2,376,200 B ≈ 2.27 MiB; one f32 plane ≈ 1.13 MiB.
**Every byte figure in this table is `size_of` × n arithmetic, not a measurement** — the three
source audits derived them the same way and none ran cargo either.

Legend for STORAGE SHAPE: **plane** = per-cell dense array · **face** = per-face record ·
**CSR** = sparse per-species/per-slot compressed rows · **layer** = per-layer-in-column ·
**table** = small world-level table · **none** = nothing kept.

| pass id | INPUTS (read) | OUTPUTS (written) | STORAGE SHAPE | HOW THE BODY COMPUTES | LIFETIME | CONSUMERS | SHIPPED? | NORTH-STAR VERDICT |
|---|---|---|---|---|---|---|---|---|
| **`dc:deep/climate`** | `grid.r`+`grid.h` via `surf_at` (f64 planes, 8 B/c, `climate.rs:65`); row latitude (`grid.rs:756`); scalar `sea_level` (undeclared, `runner.rs:357`) | `grid.precip` f32, 4 B/c, 1.13 MiB (`grid.rs:663`, written `climate.rs:95`) | plane (+1.13 MiB transient `raw`, `climate.rs:51`) | **ordered traversal** — per-row 1-D directed prefix march carrying scalar `moisture` (`climate.rs:49-79`), **plus** a separate 3-tap separable blur across y (`:83-96`). Two operations in one file. Memoryless in time | **dies with the loop** — `DeepField` has no `precip` (full struct read, `field.rs:480-613`) | in-sim: `biotic.rs:693,750,886`, `agents.rs:94,113,192,311`, `record.rs:216`. exported: none (dies) | **YES**, every 20 epochs (10 firings) | **FIELD PASS, compliant.** Writes one plane, reads terrain, touches no material and no record |
| **`dc:deep/tectonics`** | nothing per-cell; the chapter table `Vec<Vec<Plate>>` (≈5.6 KB, `mod.rs:437`) + 6 `DeepConfig` scalars | `Erosion::forcing` f64, 8 B/c (`set_tectonic`, `runner.rs:365`); `ctx.blended` f64 scratch (`runner.rs:222`); `ctx.thickening_total` scalar | plane (cache) + **table** (authority) | **closed-form/analytic** — `forcing_at` is a k=3 nearest-seed scan + Gaussian bump with **no grid term** (`tectonics.rs:244-284`); the plane is a *materialisation of a point function* scanned at `mod.rs:466-479` | forcing planes **die**; **chapter table survives** as `DeepField::chapters` (`field.rs:613`) | in-sim: `forcing` reads the plane. exported table: **zero production readers** (see § 6.4) | **YES**, every epoch | **FIELD PASS, compliant** — and the clearest *rule + parameters* case in the tree: 20.39 MiB of derived plane over 5.6 KB of authority, **3,960×** |
| **`dc:deep/forcing`** (tectonic arm) | `Erosion::forcing` (f64 plane) | `grid.t_crust` f64, 8 B/c (`uplift.rs:22-37`) | plane | **pointwise** — `*t = (*t + *f*dt).max(1000.0)`. Trivial AXPY. But the *value* is a 200-epoch integral coupled to erosion | **survives** as `DeepField::t_crust` (`field.rs:577`) | in-sim: `isostasy` (`uplift.rs:89`), `geotherm` (`geotherm.rs:163`). exported plane: **zero production readers** | **YES**, every epoch | **FIELD PASS, compliant** |
| **`dc:deep/forcing`** (legacy arm) | `Erosion::uplift` plane | `grid.r` f64 (`apply_uplift`, `runner.rs:372`); `ctx.uplift_total` | plane (terrain) | **pointwise** rate add | terrain folds into `DeepField::surf` | — | **NO** (`tectonic_history: true` selects the other arm) | **FIELD PASS, compliant** — contingent on § 2's reading that `R` is a field |
| **`dc:deep/expose`** | `grid.strata` — a **ragged near-surface record window per column** (`weathering.rs:197`); the `SusTable` blend | `Erosion::litho` u8 (297 KB), `sus_flow` f64, `sus_creep` f64 (2.38 MB ea), + `masks` u64 and CSR `shares` on the member path (`erosion/mod.rs:257-259,353`) — **≈5.05 MB, ≈7.4 MB with masks** | plane ×3 (+CSR) | **other — ragged-column reduction.** Per-cell in the grid, but each cell walks its own variable-length unit stack (`lithology::exposed_shares`, `lithology.rs:629`). Not a solve and not point-evaluable | **dies with `Erosion`** — no `litho`/`sus_*` on `DeepField` | in-sim: `transport.rs:268`, `creep.rs:185,233,456,617`, `creep_kernel.rs:254,311,325`, `ledger.rs:246`. exported: none | **YES**, every epoch | **FIELD PASS, compliant — and the exemplar of the user's clause.** It *reads the record and never changes it*, then writes planes. This is literally "a field pass touching materials only by reading the record" |
| **`dc:deep/frost`** | `grid.r`+`grid.h`, `grid.lat_deg`, and each cell's **strata window** via `SusTable::blend` (`weathering.rs:488`) | `Erosion::frost` f64, 8 B/c, 2.27 MiB (`erosion/mod.rs:265`, filled `weathering.rs:491-501`) | plane | **pointwise** — `air_temp_c` + a triangular band about 0 °C × a column-blended susceptibility; `par_iter_mut`, byte-identical parallel (`weathering.rs:450-451,491-495`) | **dies with `Erosion`** | in-sim: `frost_at` in the height weather kernel (`weathering.rs:539,550`); `runner.rs:496` hands it to `weather_inventory`. exported: none | **YES** (`full_agents`), every epoch | **FIELD PASS, compliant.** Same shape as `expose`: reads the record, writes a plane |
| **`dc:deep/drainage`** | `grid.r`+`grid.h` → `surf`; previous epoch's `area` (lagged, `routing.rs:82-100`) | `filled` f64 (2.38 MB), `order` u32 (1.19 MB), `done` bool (297 KB), `recv` i32 (1.19 MB), `mfd_w` f64×8 (**19.01 MB**, `routing.rs:23-25`), `area` f64 (2.38 MB), `out_area` f32×8 (9.50 MB) | plane ×4 + **face** (`mfd_w`, `out_area`) + a world-level **permutation** (`order`) | **three shapes in one pass body.** flood = **ordered traversal** (global min-heap priority-flood, `flood.rs:188-218`); route = **order-independent stencil** (pure 8-neighbour gather, `par_iter` byte-identical, `routing.rs:106-149`); accumulate = **ordered traversal** (reverse-`order` DAG scan with an exact-residual split, `routing.rs:182-237`) | `filled`/`order`/`mfd_w`/`out_area` **die**; `recv`/`area`/`lake` **survive** (`field.rs:534-536`) | in-sim: heavy (`transport`, `head.rs:493`, `biotic.rs:696,707`, `flux.rs:854`, `depth_to_water`). exported `recv`/`area`/`lake`: **zero production readers** (§ 6.4) | **YES**, every epoch | **FIELD PASS, compliant** — but see § 5.1: it is **three operations wearing one pass id**, and one of its outputs (`recv`) is an undeclarable derived summary |
| **`dc:deep/transport`** | `order`, `recv`/`mfd_w`, `filled`, `area`, `sus_flow`, `grid.r`/`h`, `window`/`shares`/`tlayout` | `grid.r`, `grid.h` (in place), `dh`, `energy`, `qs`, `out_load` (f64, 2.38 MB ea), `out_face_load` f32×8 (9.50 MB), CSR `qs_sp`/`dep_sp`, `TransportLedger` (≈136 B) — **19.02 MB scalar scratch + CSR** | plane ×5 + **face** + **CSR** (per-species load) + **table** (ledger) | **ordered traversal** — same reversed `order` DAG scan as `accumulate_area` (`transport.rs:558,627`), and it **mutates terrain as it goes**, so the value at a cell depends on traversal state | terrain → `DeepField::surf` (the world's primary artifact); `energy`/`dh`/`qs`/`out_*` **die** | in-sim: `deposition` reads `energy`+`dh`; `flow_record` reads `out_face_load`. exported: terrain only, read by `collapse.rs` | **YES**, every epoch | **FIELD PASS, compliant** *under § 2's reading*. It never writes `grid.strata` (verified: `grep "\.strata\b|deposit_moved|\.erode\(" ` over `deeptime/` returns **0 hits in `transport.rs`**). ⚠ Contentious: its CSR `qs_sp` is *suspended load keyed by `MaterialId`* — a field whose axis is material identity |
| **`dc:deep/head`** | `grid.strata` — **the whole record, every column, every march** (`head.rs:459-463`); `filled`, `routed`, `area`, and a rebuilt `ground = R+H` (`runner.rs:573-575`) | `grid.head` f64 (2.38 MB), `grid.head_exchange` f32 (1.19 MB) (`head.rs:535-536`) | plane ×2 | **order-independent target via ordered execution** — 24 Gauss–Seidel sweeps, alternating forward/reverse raster (`head.rs:508-522`), harmonic face conductance + Dirichlet pins + a **one-sided upper obstacle** (`relax_cell`, `:371-406`). Sweep count **fixed, not convergence-checked** (S9 determinism). Transient state 13.36 MB vs 3.56 MB output — **3.75×** | `head` **survives** (`field.rs:605`); `head_exchange` deliberately **dies** (gen-time cache, `grid.rs:707-711`) | in-sim: `flow_record` consumes `head_exchange` (`runner.rs:538-539`). exported `head`: **zero production readers** (§ 6.4) | **YES** (`head_field: true`), every 20 epochs (`HEAD_PERIOD = 20`, `head.rs:140`) | **FIELD PASS, compliant — the textbook instance.** Reads the entire material record, changes none of it, writes two planes |
| **`dc:deep/flow_record`** | `out_area`, `out_face_load`, `area`, `routed_surface`, `grid.head_exchange` (`runner.rs:527-539`) | `FluxAccum.mag`/`load` f32×13 (**15.45 MB each, 30.89 MB**, `flux.rs:689-707`); flushed to `FluxRecord` CSR at 16 B/entry (`flux.rs:598-602`) | **face** (directed half-faces, 13 `FaceKey` variants, `flux.rs:153-194`) + **CSR** by cell, chapter-keyed, stratum slot **derived** not stored (`slot_for_chapter`, `:675-677`) | **other — accumulation, not a solve.** An entry is written by the pass that moved the thing, never re-derived (`erosion/mod.rs:300-310`); `in_faces` is a query over the neighbour's stored half-face (`flux.rs:526-550`) | **survives** as `DeepField::flux` (`field.rs:556`) | **tests and examples only** — `tests/{flux_record,head_field,mfd_routing}.rs`, `examples/{flux_record_probe,head_field_probe,flow_cost_probe}.rs`; `field.rs:553-555` says so itself | **YES** (`flow_record: true`), every epoch | **FIELD PASS, compliant.** Its record is a record *of water on faces*, not of materials — it writes no `DepUnit` and no `Fact`. Storage shape is already the right one |
| **`dc:deep/weather`** (height tier) | `grid.r`/`h`, `grid.bio_weather` (`weathering.rs:548`), `Erosion::sus_flow` (`:549`), `Erosion::frost` (`:550`) | `grid.r`, `grid.h`, `Erosion::dh` — **all in place, no plane of its own** (`weathering.rs:559-598` via `weather_behavior::weather_one_cell`, `weather_behavior.rs:307-330`) | none of its own — a **delta on someone else's authority** | **pointwise** — a pure per-cell function behind a `WeatherCtx` read capability and a pass-owned `WeatherApply` write handle. **This is the north-star behavior shape already de-risked (S16) and holding** | product survives inside `surf`/`regolith` | `regolith` read in production by `collapse.rs::column` and `geology.rs:175` | **YES**, every epoch | **FIELD PASS, compliant** *under § 2's reading* — ⚠ **and this is the most contentious row in the table.** See § 5.2: it converts bedrock to regolith, which is a *material* transformation expressed purely as two f64s, and the corpus is already arguing about its altitude |
| **`dc:deep/diffuse`** (hillslope creep) | `surf = r+h` (frozen potential), `grid.h`, `grid.bio_resist` f32, `Erosion::sus_creep` | `grid.h` via `netdiff`; scratch `scale`/`netdiff`/`surf`/`creep_flux` ≈ **11.9 MB** | plane (it *is* the state) | **order-independent stencil** — the one pass declaring against `dc_core::field::FieldKernel` (`CREEP_KERNEL`, `creep_kernel.rs:25`; step `:256-270`). Kernel owns the stencil, the monotonicity bound (`MONOTONE_MAX_EDGE_COEFF = 0.125`) and sub-cycle derivation; the pass supplies the `CoeffField` | `grid.h` → `DeepField::regolith` (`field.rs:723`) | **the only plane in the solid-earth group with a real production consumer of the exported copy** — `geology.rs:175` | **YES**, every epoch | **FIELD PASS, compliant, and the reference shape.** Takes `dt` from the ctx and sub-divides internally to its own stability bound |
| **`dc:deep/isostasy`** | `grid.t_crust`, `grid.h`, `grid.crust_kind`, `grid.r` | `grid.r` f64, `grid.exhum` f64 (`+=`), `grid.t_crust` (`-=`) — `uplift.rs:51-77, 86-104` | plane ×3 | **neighbourhood reduce then pointwise** — a **±109-cell box mean** (219×219 ≈ 100.7 km, `isostasy.rs:113-115`), then a per-cell relax at `iso_rate = 0.5`. Zero persistent state; **~9.5 MB allocated and freed per epoch ≈ 1.9 GB per run** | `r` → `surf` (production); `exhum` **survives** (`field.rs:576`) | in-sim: `r` is terrain. exported `exhum`: **zero production readers** — its intended consumer (a metamorphic expresser) does not exist | **YES**, every epoch | **FIELD PASS, compliant.** ⚠ Its `box_smooth` **shrinks the window at the world border on its own authority** (`isostasy.rs:82-85`) — an **edge policy**, which is an opinion and by the opinion-vs-absence test belongs to the pack |
| **`dc:deep/geotherm`** | `grid.t_crust`, `grid.crust_kind`, the chapter's advected plates via `dominant_kind` (`geotherm.rs:161-163`) | `grid.geotherm` f64, 8 B/c, 2.27 MiB (`geotherm.rs:152-165`) | plane | **closed-form/analytic** — a `match` on `BoundaryKind`, a linear thickness deviation and a clamp (`geotherm.rs:70-96`). ~100 B of constants against 2.27 MiB of output | **survives** as `DeepField::geotherm` (`field.rs:587`) | in-sim: coal rank at finalize via `BurialColumn` (`geotherm.rs:107-130` → `promote_coal`). exported: **zero production readers** | **YES** (tectonic path), every **40** epochs (`GEOTHERM_PERIOD = 40`, `geotherm.rs:57`) | **FIELD PASS, compliant** — *"plants a field and runs no edges"* (`runner.rs:456-458`). Its stored value is **epoch-160's answer, not epoch-200's**, which is the only argument against dropping the plane |
| **`dc:deep/deposition`** ⚠ *no prior audit* | `Erosion::dh` (f64), `energy` (f64), `grid.precip` (f32), `grid.r`+`grid.h`, `sea_level`, `k_transport`, the CSR arriving-species rows `dep_sp`/`creep_sp` + `tlayout`/`clayout`, the `MemberCtx` (`GeologySet` + seeded draw stream + chapter) | **`grid.strata` — the record.** `deposit_moved(tag, dh, chapter, epoch, species, mover)` or `erode(-dh)` per cell (`record.rs:86-88`, driven `:378-384`). Optionally the `id_class`/`id_m` audit plane (off in production) | **layer** — a `DepUnit` appended per depositing cell per epoch into a per-column ordered log | **pointwise** over cells (`par_iter_mut` on disjoint `DeepStrata`, byte-identical), with a **lazy** identity closure so the member draw runs only for the depositing minority (`record.rs:68-88`) | **survives** — `DeepField::strata` (`field.rs:505`), the shipped artifact | **the collapse tier expresses it in production**; it is what the player digs | **YES**, every epoch | **CELL PASS, compliant.** It is the one pass whose entire output is a change to cell contents, it writes **no** field (`r`/`h`/`precip`/`energy` are all taken as `&` borrows, `record.rs:217`), and it reads fields while marching cells. **Exactly the user's cell-pass shape** |
| **`dc:deep/eolian`** | `grid.precip`, `grid.bio_resist`, `grid.r`/`h`, `grid.strata`, `climate::zonal_wind(lat)` | `grid.h` (**field**), `grid.strata` (**record** — `deposit_moved` at `agents.rs:99,167,197`, `erode` at `:135`), `ledger.eolian_to_sea_m`. Scratch: one `Vec<f64>` of axis length | none of its own; effect in **plane** + **layer** | **ordered traversal** — a 1-D advection march per row along `sign(zonal_wind)` carrying a scalar `load` (`agents.rs:60-177`), settled at the downwind land edge (`:179-206`) | effect survives in `surf` and in `DepTag`s (`Eolian::Dune`/`Loess`) | tags read in production via `litho_of_tag` and the collapse tier | **YES** (`full_agents`), every epoch | ⚠ **VIOLATES THE SPLIT — does both.** It writes a field (`grid.h`) *and* writes the record. Proposed split in § 4.1 |
| **`dc:deep/wave`** | `grid.r`/`h`, `grid.strata`, `sea_level`, `providers.wave_energy` | `grid.r`, `grid.h` at `i` (**field**); `grid.h` and `grid.strata` at the offshore sink `j` (**record** — `erode` `:301`, `deposit_moved` `:316`); two ledger counters | none of its own; effect in **plane** + **layer** | **neighbourhood reduce + scatter** — per-cell predicate (in the `(sea, sea+wave_band_m]` band with a subsea 8-neighbour), sink = deepest such neighbour (`agents.rs:262-270`); scalar only because it **writes a neighbour** | effect in `surf` + record | tags read in production | **YES** (`full_agents`), every epoch | ⚠ **VIOLATES THE SPLIT — does both.** Same shape as eolian. Proposed split in § 4.2 |
| **`dc:deep/biotic`** | `grid.precip`, `grid.r`/`h`, `ero.area`/`recv`/`filled`, own `cells`, `parent_p`, `wet` | `grid.bio_weather` + `grid.bio_resist` f32 (**fields**, `biotic.rs:740-741`); `grid.h` (**field**, `:752`); `grid.strata` (**record** — `overprint_top` `:763`, `deposit_as` `:776`); `ctx.biotic_total` | plane ×2 + **layer**; own state **≈57.8 MiB** (204 B/cell) | **bounded-neighbourhood reduce + pointwise, loop-carried in time** — processes 1,3–6 per-cell; process 2 is a **3×3 max over a frozen previous-epoch snapshot** (`biotic.rs:996-1017`, snapshot `:679-681`). Walker & Syers depletion is by construction the integral of the whole run | modifier planes **die**; the organic/charcoal/paleosol **units survive** (~6.1 MiB) | units read by the collapse tier; modifiers read in-sim by `weather`, `diffuse`, `eolian`, `creep`, `weather_inventory` | **YES**, every epoch | ⚠ **VIOLATES THE SPLIT MOST BROADLY — writes two condition fields, mutates terrain, AND writes the record.** Proposed split in § 4.3. Also carries the sharpest A-7 in the tree (§ 5.3) |
| **`dc:deep/weather_inventory`** | `grid.r`/`h`, `grid.bio_weather`, `Erosion::frost()`, `cfg.weathering`/`h_star`, own ledger (`weather_inventory.rs:344-372`) | `DeepStepCtx::weather_ledgers: Vec<FactLedger>` (`runner.rs:268`) — cause-carrying `Fact`s appended at the bedrock seam. **9.57 MiB resident** (measured, S20), ≥13.60 MiB of gen-time headers | **layer** — `Fact` 8 B resident / 16 B accumulating, `SlotRun` 8 B, CSR-offset by cell (`inventory.rs:862-874`); 75.8 % of cells carry none | **pointwise per cell over a per-cell column inventory** — `Σ_a driver_a × susceptibility_a × cover_taper`, three multiplies and an `exp` (`weather_inventory.rs:150-206`). Accumulating in time, never reads a neighbour | **survives** as `DeepField::ledgers` (`field.rs:526`) | **production consumers exist and always read empty**: `collapse.rs:1625` (`weathering_product_m()`) and `collapse.rs:2916` — because the pass does not run | **NO** — `weather_inventory: false` (`grid.rs:607`); reachable only via `--weather-inventory` (`dc-client/src/main.rs:159`) | **CELL PASS, compliant — and the cleanest one in the tree.** It writes only material facts, never `R`/`H`, and says so at the site (`weather_inventory.rs:340-343`, `runner.rs:485-489`). ⚠ It is also the **only cell pass besides deposition**, and it is off |

---

## 4. The passes that VIOLATE the field/cell split, with the split I would propose

Three, and they are the same three: **`eolian`, `wave`, `biotic`.** All three write the record
*and* write a field in one body. Nothing else in the roster does both — verified by
`Grep "strata\[|\.strata\b|deposit_moved|\.erode\("` over
`crates/dc-worldgen/src/deeptime/`: the only **write** sites outside tests are
`record.rs:86-88` (deposition), `agents.rs:99,135,167,197,301,316` (eolian + wave) and
`biotic.rs:763,776` (biotic). `head.rs`, `weathering.rs`, `transport.rs` and `weather_inventory.rs`
appear only as **readers** (or, in `head.rs:630,693,737`, as test fixtures building a grid).

### 4.1 `dc:deep/eolian` — split into a field pass and a cell pass

**What it does today.** One 1-D advection march per row picks regolith up where the ground is arid
and bare, carries it downwind in a scalar `load`, and drops it. The pickup writes `grid.h` *and*
calls `strata.erode(pickup)`; the drop writes `grid.h` *and* calls `strata.deposit_moved(...)`
with an `Eolian::Dune`/`Loess` tag.

**The split.** `dc:deep/eolian_transport` — a **field pass** that runs the row march and publishes
a per-cell net `Δh_eolian` plane (and, honestly, a per-face or per-row *provenance* the way
`transport` already publishes `out_face_load`). Then `dc:deep/eolian_record` — a **cell pass**
that marches cells and turns that plane into `erode`/`deposit_moved` calls under the eolian tag.

**Why the split is real and not clerical.** The recorder already proves the shape works:
`deposition` is exactly "a cell pass that turns `Erosion::dh` into record operations." Eolian is
the same computation with its own `dh`, fused into its own march for no stated reason. The fusion
costs something concrete — an eolian metre deposited today carries **no arriving species identity**
(`agents.rs:99` passes a member from the site draw, not from what the wind carried), which is
precisely the gap `record.rs:263-268` names when it lists "wind, wave, the biotic layer" as movers
that *"carry no identity yet."* A separated transport half would have somewhere to put one.

**Cost of the split, stated honestly:** it materialises one more f64 plane (2.38 MB gen-time) and
it **will move the goldens** unless the split is exactly mass-preserving in the same traversal
order — which is achievable but is not free.

### 4.2 `dc:deep/wave` — split the same way, and it is the easier of the two

**What it does today.** Per cell in the littoral band: cut `R`/`H` at `i`, pick the deepest subsea
8-neighbour `j`, and deposit the cut at `j` — writing `grid.h[j]` and `grid.strata[j]`.

**The split.** `dc:deep/wave_cut` — a **field pass** computing the per-cell cut and the chosen sink
index, publishing a `Δh_wave` plane (a scatter, which the water audit already observes could be
reformulated as a gather, `agents.rs` cf. `erosion/mod.rs:60-65`). `dc:deep/wave_record` — a
**cell pass** applying it to the record with the `Marine` cause.

**Why it is easier than eolian:** wave's transfer is already a pure function of an 8-neighbourhood
plus two scalars, so the field half is a clean local operator. The only structural work is turning
the neighbour-scatter into a gather so the field half is order-independent.

### 4.3 `dc:deep/biotic` — the broadest violation, and it should split three ways

**What it does today.** It is the only pass in the roster writing *all three* kinds of thing: two
condition fields (`bio_weather`, `bio_resist`), the terrain (`grid.h` for the organic deposit), and
the record (`overprint_top`, `deposit_as`).

**The split.**
1. **`dc:deep/biota`** — a stateful **field pass**. It carries `BioticSim` (the 57.8 MiB of
   community state, the nutrient pools, the successional clock), runs the six processes, and
   publishes exactly what other passes consume: the two `f32` modifier planes, plus a *cover* plane
   if anything is to read it. It reads fields and reads the record; it writes only planes.
2. **`dc:deep/biotic_deposit`** — a **cell pass** that turns the published organic/charcoal
   production into `overprint_top`/`deposit_as` calls and the matching `grid.h` change.
3. (Optional, and I would argue for it) the `grid.h` write belongs with (2), because organic matter
   accumulating on the ground is a deposit, not a field adjustment — which is exactly why it is a
   cell-pass concern.

**Why this one matters most.** Biotic's 57.8 MiB of state produces two `f32` multipliers — a 25:1
ratio the climate audit measured. That ratio is the honest price of a stateful ecology *if the pass
is a field pass*; it is an odd shape for something that also writes the record. And the split
resolves a live declaration defect: biotic declares `Frosted` in all three read-sets
(`runner.rs:595-597`) and never reads the frost plane (§ 6.1), which is the kind of error that
happens when a pass's declaration has to cover three unrelated jobs at once.

### 4.4 The contentious near-violations — flagged loudly, not ruled

- **`dc:deep/weather` (height tier).** Under § 2's reading it is a compliant field pass. But it
  performs the *bedrock → regolith* transformation, which is a material change expressed as
  `R -= q; H += q`, and its own module says the strain out loud: the height tier has **no single
  outcropping material**, so `weatherability` degenerates to a per-cell blend
  (`weather_behavior.rs:26-38`). The corpus is already arguing about this pass's altitude. **If the
  user's rule is meant to reach "converting one material to another", this is a violator, and the
  split is: keep the field half (the mass transfer) and let `weather_inventory` — which already
  exists, already emits the cause-carrying facts, and is off — be the cell half.** That is not a
  hypothetical split; it is the pass pair that already exists in the tree with one of them disabled.
- **`dc:deep/transport`.** Compliant as written, but its CSR `qs_sp`/`dep_sp` are **suspended load
  keyed by `MaterialId`** — a field whose axis is material identity. Whether "a per-species mass
  field" counts as "materials" under the rule is a genuine question. My reading: it is a field
  (material *in transit* is not cell contents), and the code agrees — the identity only becomes a
  material when `deposition` records it.
- **`dc:deep/drainage`.** Compliant, but it is **three operations under one pass id** (a global
  priority-flood, an order-independent per-cell partition, and an ordered DAG scan). If passes are
  standardised by computation shape, this pass cannot honestly declare one.

---

## 5. A-7 (content identity / privileged slot / hardcoded target) and the pack/engine question

**A-7 is a defect, not a ratifiable carve-out** (`spines.md`). Per row:

| pass | A-7? | what, at source |
|---|---|---|
| climate | **no** identity; but the whole rainout/lapse model is pack physics living in `dc-worldgen` | `climate.rs:39-44`, `pregen/climate.rs:159-166` |
| tectonics | **no** | — |
| forcing | **no** | — |
| expose | **YES (vocabulary)** — blends over the closed 7-variant `Litho` enum (`lithology.rs:262-290`, `Litho::COUNT = 7`) with a named `Basement` arm. A closed lithology vocabulary in an engine crate is a pack opinion hardcoded | `lithology.rs:262-290`, `weathering.rs:56-72` |
| frost | **YES (vocabulary)** — same `SusTable`/`Litho` blend | `weathering.rs:488` |
| drainage | **no** | — |
| transport | **YES (privileged slot)** — `ANCHOR_MATERIAL = MaterialId::SANDSTONE` (`transport.rs:87`), with `COMPETENCE_SCALE = 420` derived as `settle_energy(sandstone)/ENERGY_LOW_MED`. The doc is candid: *"under member grade there is no 'coarse clastic' to anchor on, so the anchor has to be a rock"* (`:82-86`) — **a named rock is load-bearing in an engine-side threshold** | `transport.rs:78-95` |
| head | **no** — derives transmissivity/confinement from the record's units generically | `head.rs:277-320` |
| flow_record | **no** | — |
| weather | **no** identity, but see § 4.4 | — |
| diffuse | **no** — the coefficient field comes from `sus_creep`, which inherits `expose`'s `Litho` vocabulary | — |
| isostasy | **no** identity. **YES (undeclared opinion)** — the border edge policy (`isostasy.rs:82-85`) | — |
| geotherm | **borderline** — three hardcoded crust baselines and a 7-arm `BoundaryKind` match (`geotherm.rs:70-96`). A closed tectonic-setting vocabulary, not a material identity | — |
| **deposition** | **YES, and it is the densest site** — `litho_of_tag` maps the closed `DepTag` space onto the closed `Litho` space; `lithology::deposited_transform` hardcodes the transformation edges (basement→coarse clastic, peat/coal/charcoal→carbonaceous mud) named in prose at `record.rs:283-287`; the tie rule is *"axis order = descending settling energy, ties by `MaterialId` index"* (`record.rs:118-124`) | `record.rs:263-330`, `lithology.rs` |
| eolian | **YES** — `Eolian::Dune`/`Eolian::Loess` are named `DepTag` variants; the deposit's member comes from a site draw, not from what was carried | `agents.rs:99,167,197` |
| wave | **YES** — `FlowCause::Marine` privileged as the wave agent's cause | `agents.rs:316` |
| **biotic** | **YES — the sharpest in the tree.** `ROSTER = 7` with a `const NICHES: [Niche; 7]` table of **named species** — `"lichen"` and six more, each with hand-tuned temperature/moisture/soil/N/P/shade constants (`biotic.rs:187,237-250`). A fixed seven-species roster compiled into the worldgen crate is a content pack wearing engine clothes | `biotic.rs:187,237+` |
| weather_inventory | **YES** — `BEDROCK_SEAM_MATERIAL = MaterialId::GRANITE` (`inventory.rs:1245`), *"one flat granite basement everywhere"*, **explicitly labelled STUB #16 with its heir named**. This is the honest case: annotated in code, listed in `stubs.md`, not silent | `inventory.rs:1240-1245` |

**The pack-vs-engine answer, stated once.** *Every* pass in this roster is content by the north
star (*"every pass is content, including tectonics and erosion"*), so the interesting question is
not "does this pass belong to the pack" but **"what does it declare over, and does that vocabulary
belong to the pack?"** Three closed vocabularies are declared engine-side and all three are pack
opinions:

1. **`DeepAxis`, 18 variants** (`runner.rs:110-202`). Half of them name *pack processes*, not
   engine resources: `Frosted`, `BioMod`, `BioRecorded`, `Saprolite`, `Geotherm`, `Forcing`. A
   third-party pack adding a karst pass cannot publish an axis; a pack removing biology leaves
   `BioMod` in the engine's enum. This is the open-vocabulary work (E6) and it is **upstream of
   every other finding in this document** — a pass cannot declare its storage shape, its lifetime,
   or its derived-vs-authority status because it cannot declare *anything* the engine has not
   pre-named.
2. **`Litho`, 7 variants** (`lithology.rs:262-290`) — the lithology vocabulary `expose`, `frost`,
   `transport` and `deposition` all blend over.
3. **`Resource`, 9 variants** (`pipeline.rs:47-70`) — the pregen tier's own closed vocabulary, and
   a *different* one (see § 7).

Plus two closed content tables: `NICHES` (7 species) and `MAX_DEEP_SPECIES = 64`
(`species.rs:76`).

---

## 6. Where a prior audit was wrong, or two audits disagreed

**6.1 — `dc:deep/biotic` declares a read it does not perform. VERIFIED, and the climate audit was
right.** I re-ran its search: `grep -n "frost" crates/dc-worldgen/src/deeptime/biotic.rs` returns
**2 hits, both doc-comment prose** (`:565`, `:842`). `BIO_READS_AGENTS`/`_TEC`/`_LEG` all include
`Frosted` (`runner.rs:595-597`). Worse, `BIO_READS_TEC` and `BIO_READS_LEG` declare `Frosted` on
rosters where the `frost` pass **is not registered at all** (`runner.rs:774` gates it on
`full_agents`) — while the runner's own comment at `:598-603` explains that `Frosted` appears "only
in the agents slice" for the *inventory* pass's read-sets. **The biotic sets did not get the same
treatment.** Whether the emitted order moves is **unverified** — I ran no cargo.

**6.2 — the two audits disagreed on `GEOTHERM_PERIOD`, and solid-earth was right.** Solid-earth
says *"the pass fires every 40 epochs"* (`:352`); the shape-notes consolidation is silent; the
water audit's parallel claim about `head` says 20. **Both are correct and they are different
constants**: `GEOTHERM_PERIOD = 40` (`geotherm.rs:57`), `HEAD_PERIOD = 20` (`head.rs:140`). I
re-read both at source. No contradiction — recording it because a reader skimming the two audits
would see "20" and "40" for "the coarse-rate field passes" and reasonably assume one is stale.

**6.3 — `Erosion::exposed()` has zero call sites. VERIFIED.** I re-ran the search:
`grep -rn "\.exposed(" crates/ --include=*.rs` returns **0**. Solid-earth's claim holds. Its doc
(`weathering.rs:125`) names a measurement probe that does not exist, and `litho` is written every
epoch for it.

**6.4 — the "zero production readers" claims. VERIFIED by re-reading `DeepField` rather than by
re-grepping.** I read the full public field list
(`grep -n "^    pub [a-z_]*:" crates/dc-worldgen/src/deeptime/field.rs`): `w, wp, cell_m, surf,
regolith, strata, ledgers, recv, area, lake, flux, exhum, t_crust, geotherm, head, chapters`.
That confirms the *negative* half directly — `precip`, `bio_weather`, `bio_resist`, `frost`,
`filled`, `energy`, `dh`, `mfd_w`, `litho`, `sus_*`, `head_exchange` are **not on the struct at
all**, so they cannot have any consumer, production or otherwise. For the exported planes that
*do* survive, I did not re-run the water and solid-earth agents' greps; **I am relaying their
absence claims with their searches intact rather than re-deriving them**, and both stated their
commands. Net: of 13 exported data members, **`surf`, `regolith`, `strata` and `ledgers` have
production consumers** (and `ledgers` always reads empty); `recv`, `area`, `lake`, `flux`, `exhum`,
`t_crust`, `geotherm`, `head`, `chapters` — **nine** — do not.

**6.5 — a framing error in the consolidation notes, corrected here.** The shape-notes doc
(`§ 2, :69-72`) says *"none of the surprising passes is a mislabelled cellular pass… Height-tier
weathering is the genuinely mixed one."* **That is not the full set.** Under the user's rule the
genuinely mixed passes are **`eolian`, `wave` and `biotic`** — all three write the record *and* a
field — and height weathering is mixed only under a *different* and more aggressive reading of
"materials" (§ 4.4). The consolidation was written before the ruling and was reasoning about
"touches material" informally; applying the ruling as stated moves the answer.

**6.6 — no prior audit was found to be wrong on a number.** I re-derived the production grid
(`n = 545² = 297,025`) independently from `field.rs:277-279`, `field.rs:46,50` and
`grid.rs:804-805` and got the same figure all three audits used. The `DEEP_MAX_WIDTH = 550` cap
trap (it is a cap, so Medium lands on 545 not 550) is real and all three handled it.

---

## 7. The pregen pipeline — a *different* closed vocabulary

The pregen tier (`crates/dc-worldgen/src/pipeline.rs`) runs a DAG **once** to build world state
from nothing, while the deep-time runner runs a **loop** over persistent state. They share the
graph math through `crate::passgraph` — *"one implementation, not a second runner beside it"*
(`runner.rs:12-14`) — but they declare over **two unrelated closed enums**: `Resource`
(**9 variants**, `pipeline.rs:47-70`) and `DeepAxis` (**18 variants**, `runner.rs:110-202`).

| pass id | reads | writes | note |
|---|---|---|---|
| `dc:pass/tectonics` | — | `Plates`, `Elevation`, `Provenance` | `pipeline.rs:330-333` |
| `dc:pass/climate` | `Elevation` | `Climate` | `:338-341` — **the pregen climate, and it is the one the collapse tier samples** (§ below) |
| `dc:pass/hydrology` | `Elevation`, `Climate` | `Hydrology` | `:346-349` |
| `dc:pass/deep-time` | `Elevation`, `Provenance`, `Climate` | `DeepElevation`, `DeepStrata` | `:354-357` — **the entire 18-pass deep-time roster is ONE node here** |
| `dc:pass/igneous-emplacement` | `Provenance`, `Elevation`, `Climate` | `Strata` | `:362-365` |
| `dc:pass/clastic-deposition` | `Climate`, `Hydrology`, `Elevation`, `Strata`, `DeepStrata` | `Strata`, `Alluvium` | `:376-379` |
| `dc:pass/placer` | `Alluvium`, `Hydrology`, `Strata` | `Strata` | `:396-399` |

**The observation worth carrying forward:** the deep-time tier is a *single resource* at the pregen
altitude, and the two vocabularies overlap semantically without sharing a token — `Resource::Climate`
and `DeepAxis::Climate` are two different precipitations, and the collapse tier samples the
**pregen** one (`collapse.rs:1335-1366`), not the deep-time marched plane. So the world the player
walks is textured by a precipitation field that never saw 200 epochs of orogeny. The climate audit
flagged this and declined to call it a defect; I agree with that restraint and repeat the flag.
Two closed vocabularies for one concept is exactly what an open vocabulary would dissolve.

---

## 8. What I could not resolve

1. **Whether § 2's reading of `R`/`H` as fields is the user's intent.** Half the verdicts hinge on
   it. I argued it from the code's own two-authorities language, but it is not ratified.
2. **Whether biotic's over-declared `Frosted` changes the emitted topo order.** Needs a cargo run;
   I ran none. **Unverified.**
3. **The flux record's current production residency.** The corpus's 42.6 MB (`spines.md:1848`)
   predates P11 slices 1–3. Needs `cargo run --example flow_cost_probe`. **Unverified.**
4. **Whether tectonics' 20.39 MiB precompute beats on-demand evaluation** (59.4 M evaluations
   avoided). Solid-earth flagged it; nobody has measured it. **Unverified.**
5. **The priority-flood heap's real peak occupancy.** The water audit's `n × 16 B` is an upper
   bound derived from the struct; nothing instruments the actual peak.
6. **Whether `deposition` should be split at all.** I classified it as a compliant cell pass, but
   it is doing two arguably separable things — *measuring the facies tag* (a field query) and
   *committing the unit* (a record write). I did not propose a split because the cost decision
   (`record.rs:68-76`: the lazy identity closure exists precisely so the member draw runs only for
   depositing cells) is a deliberate, documented fusion. Flagging that I chose not to.
7. **Whether `dc:deep/drainage`'s three-operations-one-id shape is a violation of anything.** It
   violates no stated rule today. It will violate any standardisation keyed on computation shape.
