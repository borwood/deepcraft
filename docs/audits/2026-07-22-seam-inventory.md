# The seam inventory — deepcraft headless crates

Read-only audit, 2026-07-22. Serves `docs/ARCHITECTURE.md` § "A summary is not
an authority" (DECIDED 2026-07-21) and the seam-first / plugin-first policy.

**Scope**: `dc-core`, `dc-worldgen`, `dc-sim`, `dc-api`, `dc-physics`.
**Test applied to every candidate**: *"if this consumer disappeared tomorrow,
would this code still exist in this shape?"* and *"is this value standing in for
an answer some other system will eventually own?"*

**Count**: 34 seams. 9 confirmed from the seed set, 25 found by sweep.

> **⚠ FOUR OF THE FIVE "SOCIAL SIM" SEAMS NO LONGER EXIST (2026-07-28,
> journal/0121).** Seams **30, 31, 34** and the whole of § 4's premise were sites
> inside `pregen/history.rs` / `collapse.rs::ruin_posts`, **removed** on the
> user's 2026-07-26 direction as unratified bootstrap content. Seams **32** and
> **33** still exist as code but are inside dc-sim's S2 statistical tier, which
> now has **zero production consumers** (spines § 3). **This audit is a dated
> snapshot and is not renumbered** — later docs cite "34 seams" and that count is
> this audit's, not today's. Live seams: **31**, not 34. § 4 is now 1 nominal seam
> (33's `base_danger`, unreached) rather than 5.

**Vocabulary used below**
- *granularity* **pass-level** = a whole field/plane; fits the existing
  `pipeline::Resource` vocabulary (`crates/dc-worldgen/src/pipeline.rs:47-79`).
  **value-level** = a per-cell/per-voxel/per-material function; the pass graph
  has **no mechanism for this today** — it declares resources, not providers.
- *identity fallback* — the value that makes the consumer behave **exactly** as
  today when no provider resolves. `TRUE IDENTITY` = a mathematical no-op
  (`×1.0`, `+0.0`, empty plane). `ARBITRARY` = a magic number that happens to be
  today's answer; substituting anything else silently changes worlds. **Every
  ARBITRARY row is a doctrine trap.**
- *byte-identity precedent*: four flags already prove "provider absent ==
  today" with tests —
  `DeepConfig::biotic` / `erodibility` / `full_agents` / `tectonic_history`
  (`crates/dc-worldgen/src/deeptime/grid.rs:236-262`), asserted by
  `crates/dc-worldgen/tests/full_agents.rs:83`
  (`neutral_full_agents_is_byte_identical_to_off`), `:114`, `:166`, and the
  siblings in `tests/biotic.rs`, `tests/erodibility.rs`,
  `tests/tectonic_history.rs`. The *mechanism* is an **empty plane read through
  an identity accessor** — `wmult_at` returns `1.0` on an empty slice
  (`erosion.rs:311-317`), `frost_at` likewise (`erosion.rs:322-326`),
  `eff_diff` returns unmodified diffusion (`erosion.rs:186-192`). That is the
  pattern every value-level seam below should copy.

---

## A. The seed set (all nine confirmed)

### S1 · `surface_material_at(vx, vz)` — the world's surface material rule

| field | value |
|---|---|
| **site** | def `crates/dc-worldgen/src/collapse.rs:681` (`surface_sample`); the surface branch `:693-743`; class kernel `surface_class` `:786-829`; fallback `:744-769`. Call sites: `:857` (`coarse_surface`, far field) and `:1270` (`column`, near ground) — **the shared kernel, deliberately** (ARCHITECTURE § One world-answer surface). |
| **today** | `surface_class` walks the deep record's topmost `voxel_m` (0.9 m) of units *in reverse*, accumulates metres per `deep_class(u.tag)`, returns the **dominant class by metres** with a `c < bc` id tie-break (`:807-813`). If under half a voxel accumulated, falls back to province basement (`Orogeny\|Arc → intrusive`, `Rift → extrusive`, else `None` — `:824-828`). The member is then dithered (`:729-730`) and the block is `block_twin(member.material)`. |
| **owed by** | **ecology** (the surface is the ecology/soil layer's own answer; grass is explicitly suspended — stubs.md § 2) with **materials** owning the mixing rule. |
| **identity fallback** | The near path already has the honest answer: `classify(mixed_contents(...))` of the top-of-column remainder (`collapse.rs:1346-1355`). The *far* path is the one that needs the summary. **True identity is available**: make the provider return the same contents the near path builds and `classify` it. |
| **byte-identity** | Provable — journal/0055's integrator test already asserts far horizon and near ground agree on surface **material**, which is exactly the agreement test this doctrine generalizes (ARCHITECTURE:341-345). |
| **granularity** | value-level (per voxel column). |
| **blast radius** | Every surface voxel in the world, near and far. Currently 91.4 % Mudstone at a 907-voxel stride (stubs.md § 2). |
| **note** | Doc-vs-code: stubs.md calls this "RETIRED WHERE A RECORD EXISTS". The *dominant-by-metres summarization* is still there and is still the rule; what retired was the climate-threshold Grass/Dirt/Stone table. |

### S2 · `material_properties(litho)` — six fixed reference rocks

| field | value |
|---|---|
| **site** | def `crates/dc-worldgen/src/deeptime/lithology.rs:283-292` (`Litho::reference_material`). Callers: `:296` (`Litho::resistance`), transitively `susceptibility_table` `:386-396`, read per-cell by `erosion.rs` abrasion/eolian/wave/frost paths (`:1163`, `:1345`, `:1478`, `sus_at`). Also `examples/erodibility_probe.rs:460`. |
| **today** | `ClasticFine→MUDSTONE`, `ClasticCoarse→SANDSTONE`, `OrganicSoil→CARBONACEOUS_MUDSTONE`, `OrganicPeat→PEAT`, `OrganicCoal→COAL`, `Basement→GRANITE`. Fixed, deliberately **not** sampled from the live registry ("a content pack cannot move terrain by adding a member", module docs). |
| **owed by** | **materials** (a class-level property sheet: the aggregate properties of a class, not one member's). |
| **identity fallback** | The six ids above. **ARBITRARY — and loudly so.** `Litho::OrganicSoil → CARBONACEOUS_MUDSTONE` means *the deep sim believes loose regolith is a lithified rock*; the doctrine table names this one explicitly ("the sim believes loose regolith is sandstone"). Any provider returning class *means* rather than one member's sheet changes every erosion rate. |
| **byte-identity** | Provable only against the six-id table. There is no identity value — this is a lookup, not a multiplier. |
| **granularity** | value-level, but with a tiny fixed domain (6) — cheap to convert. |
| **blast radius** | Every erosion rate in every world with `erodibility` or `full_agents` on (production: both on, `field.rs:135-157`). Terrain shape. |
| **note** | The *fixedness* is a ratified property (pack-addition blast radius, ROADMAP § S10 call 5), so the provider must be a **class-level** sheet, not a registry sample. |

### S3 · `is_granular(material)` — a class-string test standing in for a form axis

| field | value |
|---|---|
| **site** | def `crates/dc-worldgen/src/fill.rs:310-313` (`is_loose`), one caller `fill.rs:284`. Sibling class-string test `crates/dc-worldgen/src/collapse.rs:1467` (`contents_for_event`). |
| **today** | `class == CLASS_CLASTIC_FINE \|\| class == CLASS_CLASTIC_COARSE \|\| class == CLASS_ORE_PLACER` → debris bucket, else structure bucket. `collapse.rs:1467` is the same test minus the placer arm. |
| **owed by** | **materials** (a `form` / `granular` axis on the property sheet; `grain_size_mm` and `cohesion` already exist — `materials/mod.rs:197 ff.`). |
| **identity fallback** | A `form` axis whose value on the vanilla 26 materials reproduces the class test exactly. **True identity is achievable and testable**, but *only if a pack cannot bind a loose material into a rock class* — and `tests/geology.rs` deliberately registers exactly that pack (`SILT` as clastic-fine, `GRAVEL` as clastic-coarse; see `collapse.rs:1595-1600`). So the material-keyed answer and the class-keyed answer **already differ for a registered pack**. |
| **byte-identity** | Provable for vanilla; **NOT** provable for the loose-materials pack. That divergence is the whole finding. `collapse.rs:1597-1601` (`classify_reproduces_the_retired_class_table`) already runs the pack precisely because "a material-keyed table could have diverged from a class-keyed one" — it asserts they *do* agree at the block tier, which is not the same as agreeing at the form tier. |
| **granularity** | value-level (per member). |
| **blast radius** | Whether a mixed voxel is `StructureShape::Full` or `debris_only` — i.e. whether ground is walkable rock or loose fill, and whether journal/0010's partial-height render fires. |

### S4 · `root_cohesion(cell)` + `biotic_weathering(cell)` — roots as rate multipliers

| field | value |
|---|---|
| **site** | produced `crates/dc-worldgen/src/deeptime/biotic.rs:812` (`resist = root.clamp(0.0, 0.9)`) and `:813` (`weather = 1.0 + weather_boost`); `root`/`weather_boost` are cover-weighted sums over `NICHES` (`:806-811`, roster at `:137-270`). Written to planes `grid.bio_resist` / `grid.bio_weather` (`biotic.rs:518-519`; declared `grid.rs:316`, `:320`). Consumers: `erosion.rs:186-192` (`eff_diff`, hillslope creep), `:311-317` (`wmult_at`, bedrock weathering), `:1249`/`:1266` (diffusion passes), `:1396-1400` (eolian vegetation trapping). |
| **today** | Two `f32` planes. `resist ∈ [0, 0.9]` scales hillslope diffusivity down (`1 - resist`) and traps eolian load; `weather ≥ 1.0` scales bedrock→regolith weathering up. |
| **owed by** | **ecology** (root materials with real cohesion) + **materials** (the cohesion axis already exists — `MaterialProps::cohesion`). |
| **identity fallback** | **TRUE IDENTITY, already implemented**: `bio_resist` empty ⇒ `eff_diff` returns unmodified `diffusion` (`erosion.rs:187-191`); `bio_weather` empty ⇒ `wmult_at` returns `1.0` (`:312-316`); `bio_resist` empty ⇒ `veg = 0.0` (`:1396-1400`). And `BioticSim::new` deliberately fills them with `1.0` / `0.0` "so the very first erosion step is byte-identical to a biology-free run" (`biotic.rs:437-442`). |
| **byte-identity** | **Proven** — `tests/biotic.rs`, and the `DeepConfig::biotic` flag. This is the model precedent for every other value-level seam here. |
| **granularity** | pass-level (two planes) with value-level derivation. The planes belong in `Resource`; the per-species niche contract does not. |
| **blast radius** | Hillslope relief, regolith production rate, dune-field extent. |
| **note** | The doctrine's own table: "roots are a number, not a material". The *plane* is honest; the fact that a root is only ever a scalar is the leak. |

### S5 · `depth_to_water(cell)` — the waterlogging proxy

| field | value |
|---|---|
| **site** | def `crates/dc-worldgen/src/deeptime/biotic.rs:645-647`. Consumers, all in `step_cell`: `:667-671` (peat-former suitability gate `((wet-0.30)/0.15)`), `:752` (`drain_factor = (1.0-(wet-0.30)/0.40).clamp(0.10,1.0)`, the decomposition throttle), `:815` (`dryness = 1.0 - wet`, fire fuel curing), `:829` (`peat_site = cover[4] > 0.15 \|\| (wet > 0.42 && net_org > 0.004)`). |
| **today** | `wet = clamp(moist + low_bonus + area_bonus, 0, 1)` where `moist = grid.precip[i]`, `low_bonus = clamp((80-surf)/80, 0, 1) × 0.20`, `area_bonus = min(area[i]/300, 1) × 0.15`. Five magic numbers (80 m, 0.20, 300, 0.15, and the three thresholds 0.30/0.40/0.42). |
| **owed by** | **hydrology** (a water table). Its retirement is already named in `docs/design/water.md`; the S11 spike (`crates/dc-worldgen/src/water/sat.rs`) already **reads the water table as the top of the saturated zone** and stores nothing — the provider shape exists in-tree, unwired. |
| **identity fallback** | **ARBITRARY, and the single most dangerous row in this inventory.** There is no identity value: `wet` is a *level*, not a multiplier, and four separate consumers threshold it at four different points. A provider returning true depth-to-water in metres would need a calibration to reproduce today, and there is none. Substituting anything moves every coal swamp in the world. |
| **byte-identity** | **NOT provable today.** No flag, no empty-plane path, no `wet_at()` accessor. This is the seam most in need of the empty-plane treatment. |
| **granularity** | value-level today; the heir is pass-level (a saturation field / water-table plane, a new `Resource`). |
| **blast radius** | The geography of all organic facies — peat, coal, paleosols, charcoal — hence `deep_class`, hence surface material (S1). Enormous. |

### S6 · `charcoal_expression(unit)` — a stand-in whose justification has expired

| field | value |
|---|---|
| **site** | `crates/dc-worldgen/src/geology.rs:334` — `Biofacies::Charcoal \| Biofacies::Mineral =>` share one arm. Rationale at `:312-318`. Mirror in `lithology.rs:352-360` (`litho_of_tag`), asserted equal by `tests/erodibility.rs::litho_routing_matches_the_collapse_tier`. |
| **today** | A charcoal-tagged unit routes to the mineral clastic class it is a streak within — no charcoal class, no charcoal member. The doc comment's justification: "measured mean ~0.035 m over 158 310 beds, and **none** of them survives the 0.9 m voxel quantization". |
| **owed by** | **ecology** (fire products) + **materials** (an inclusion/debris expression). |
| **identity fallback** | Today's arm. **ARBITRARY** — the fallback IS the deletion of the fire record from expression. |
| **byte-identity** | Trivially provable (add a class, leave it empty). |
| **granularity** | value-level (per tag). |
| **blast radius** | Small today (invisible), large the day a fire bed should read as a black streak. |
| **note** | **The justification is falsified.** The stated cause was whole-voxel quantization; journal/0055 replaced that with addressed stochastic rounding at eighth granularity (`fill.rs:279-303`, `allocate_partial` `:329`) and journal/0026's inclusion path (`contents_for_event` accessory pores, `collapse.rs:1476-1493`) is built and shipping. **A 0.035 m bed in a 0.9 m voxel is ~1/25 of a voxel — still under one eighth (0.1125 m)**, so the number does not by itself resurrect a charcoal *band*; but the doc comment's own named honest answer ("an inclusion (pore/debris partial) — filed, not built") is now buildable and is not built. Report, not proposal. |

### S7 · sampling-mode asymmetry in `DeepField`

| field | value |
|---|---|
| **site** | `crates/dc-worldgen/src/deeptime/field.rs:354-357` (`surface_at_voxel` — **bilinear**) vs `:377-385` (`regolith_at_voxel` — **nearest**) vs `:394-402` (`record_at_voxel` — nearest, "FLAGGED sampling choice"). |
| **today** | Elevation is interpolated across the 460 m deep grid; regolith `H` and the strata record step. |
| **owed by** | Nobody unbuilt — this is a **deliberate, argued** choice: `H ≡ Σ(record unit thicknesses)` exactly (journal/0053), and the veneer budget subtracts one from the other (`geology.rs:503-507`), which only conserves mass if both name the same cell. |
| **identity fallback** | n/a. |
| **byte-identity** | n/a. |
| **granularity** | pass-level. |
| **blast radius** | Interpolating regolith without interpolating the record would "leak or invent loose material at every cell boundary" (`field.rs:370-372`). |
| **verdict** | **NOT a seam — a coupling constraint.** Included because the seed set named it. What it *is* is an undeclared invariant: the mass-conservation identity between two `DeepField` accessors lives in a doc comment and nothing enforces it (corrections #29's family — prose cannot fail a build). Worth a test, not a provider. |

### S8 · `wave_energy(cell)` — a global constant standing in for fetch

| field | value |
|---|---|
| **site** | field `crates/dc-worldgen/src/deeptime/grid.rs:131`; default `:253` (`wave_erosion: 0.05`, `wave_band_m: 30.0`); consumer `crates/dc-worldgen/src/deeptime/erosion.rs:1478-1531` (`Erosion::wave`). Also `examples/tour_map.rs:106,114`; zeroed in `tests/full_agents.rs:92,128,218,277`. |
| **today** | One global metres-per-epoch cut rate. Attacked cells are those with freeboard in `(sea, sea+band]` **and** a subsea 8-neighbour (`:1494-1509`) — an *adjacency* test standing in for "is there open water with fetch". The cut is `rate × susceptibility × taper`, taper `= 1 - free/band`. |
| **owed by** | **hydrology** — the heir is named in ROADMAP:1327: "fetch model (wave energy from S11 body size/shape/depth × the 0037 wind field)". Both inputs exist: `water::BodyGraph` (`crates/dc-worldgen/src/water/body.rs`) and `pregen::climate::zonal_wind` (`pregen/climate.rs:84-96`). |
| **identity fallback** | **TRUE IDENTITY EXISTS AND IS ALREADY WIRED**: `rate <= 0.0 \|\| band <= 0.0` ⇒ early return (`erosion.rs:1479-1481`). A fetch provider returning a constant `0.05` reproduces today exactly. |
| **byte-identity** | **Proven** — `tests/full_agents.rs:83` zeroes `wave_erosion` and asserts byte-identity with the agent off. |
| **granularity** | value-level (per shore cell) with a pass-level input (the body graph is a new `Resource`). |
| **blast radius** | Every coastline. Today every coast erodes equally regardless of whether it faces a pond or an ocean — a lee shore and a windward shore are identical. |
| **note** | The magnitudes are **EXPLICITLY UNRATIFIED** and flagged as such in `field.rs:148-157`: "the LIVE MAGNITUDES TOUR — not this line — ratifies the numbers. Do not tune them here." Same status for the other six agent knobs (`eolian_deflation` 0.02, `eolian_arid_precip` 0.32, `eolian_deposit_frac` 0.25, `frost_weathering_gain` 1.5, `frost_band_width_c` 12.0, `wave_band_m` 30.0). |

### S9 · `fits_in_pores(filler, host)` — a DECIDED rule with zero production callers

| field | value |
|---|---|
| **site** | `crates/dc-core/src/materials/packing.rs:32` (`K_PORE = 0.25`), `:62-73` (`fits_in_pores`). Callers outside the module: **none**. The only other mention repo-wide is a doc comment explaining why geology *doesn't* call it (`crates/dc-core/src/materials/geology.rs:642`). 7 tests in-module, all green. |
| **today** | `filler.grain_size_mm <= 0.25 × min(host structural grain sizes)`; empty host ⇒ `false`. |
| **owed by** | **hydrology** (groundwater infiltration) and **diagenesis** (ore/cement deposition) — the module docs name "overflow packing, groundwater infiltration, future ore deposition" as its consumers, and none of the three exists. |
| **identity fallback** | n/a — nothing calls it, so absence *is* today. |
| **byte-identity** | Trivially true. |
| **granularity** | value-level. |
| **blast radius** | Zero today. |
| **verdict** | **The inverse defect**: not a summary that became an authority, but an **authority with no consumer**. It fails the doctrine's test in the other direction — "if this consumer disappeared tomorrow" is unanswerable because the consumer never appeared. It is the one place in this inventory where the *provider* is built and the *seam* is missing. This is exactly the shape the user's remark describes ("the stubbed apis would be telling us right now what an unbuilt hydro system is supposed to supply") — and here it already does. |

---

## B. Found by sweep

### dc-worldgen — `geology.rs` (the collapse strata passes)

| # | id | site | today | owed by | identity fallback | byte-id | gran. | blast |
|---|---|---|---|---|---|---|---|---|
| 10 | `emplacement_depth(cell)` | `geology.rs:237` `INTRUSIVE_DEPTH_M = 250.0`; `:241` `INTRUSIVE_TOP_VOX = 96`; extrusive `5.0` inline at `:266`; used `:252-273` | one number where a per-column burial story belongs | **metamorphism / paleo-context** | 250.0 / 5.0 / 96 — **ARBITRARY** | trivial (constant provider) | value-level | intrusive-vs-extrusive member choice + basement thickness in every Orogeny/Arc/Rift column. *stubs.md § 5.* |
| 11 | `paleo_temperature(cell, epoch)` | `geology.rs:402` — `let temp_c = ctx.temp_c; // latitude proxy` | a deep unit's at-deposition temperature is **today's** column temperature; the aridity axis correctly reads the recorder's tag (`:400`, `deep_precip`), temperature does not | **paleoclimate** (an epoch-indexed curve) | present-day `temp_c` — **ARBITRARY** (it is a different quantity, not a neutral one) | provable against today's value | value-level | temp-sensitive member fitness inside every deep stratum. *stubs.md § 6.* |
| 12 | `paleo_precipitation(tag)` | `geology.rs:284-291` (`deep_precip`) | `Arid → 0.12`, `Humid → 0.60` — a 2-bucket quantization of a continuous field | **paleoclimate** | 0.12 / 0.60 — **ARBITRARY** | provable | value-level | clastic-fine vs -coarse fitness in every deep unit. Not in stubs.md. |
| 13 | `accessory_presence(cell)` | `geology.rs:211-212` `ACC_PRESENCE = 0.6`, `ACC_EIGHTHS = 1`; gate `:219`; caller `:271`,`:275` | 60 % of igneous columns carry exactly 1/8 of one accessory | **diagenesis / magmatic differentiation** | 0.6, 1 — **ARBITRARY** | trivial (`>= 1.0` closes the gate → no-op) | value-level | olivine speckle density in all basalt/granite. Not in stubs.md. |
| 14 | `lode_endowment(catchment)` | `geology.rs:588-604` (`placer_pass`) | placer presence is a function of `alluvium.energy` and the ore member's settle threshold **only** — every big-enough river carries gold | **ore genesis (R1 lode field)** — heir named ores.md R2-B: `drainage export × lode field` | a uniform endowment of 1.0 would be a **TRUE IDENTITY** if the term were multiplicative; today there is no term at all, so it is a **missing factor**, not a stand-in value | provable (uniform field ⇒ today) | value-level, pass-level input | "read the river, walk upstream" dead-ends. *stubs.md § 11.* |
| 15 | `settle_calibration` | `geology.rs:554` `SETTLE_TO_FLOW = 4.0` | scale from property-derived `sqrt(mm·SG)` to column flow-energy units; self-described "Calibration, not physics" | **materials** (units) | 4.0 — **ARBITRARY** but honestly labelled | provable | value-level | where in a fan the placer peaks. |
| 16 | `veneer_overburden` | `geology.rs:347` `DEEP_VENEER_MARGIN_M = 2.0` | 2 m of assumed overburden added to every deep unit's burial depth | **materials / stratigraphy** | 2.0 — **ARBITRARY** | provable | value-level | depth-axis member fitness (coal rank!) in every deep unit. |
| 17 | `fan_energy(cell)` | `geology.rs:508` `fluvial = (ctx.flow_energy / 12.0).min(3.0)`; `coarse_fraction` `:277-279` `(e/30.0).clamp(0, 0.85)`; `flow_energy` built `collapse.rs:1307-1311` (`width × exp(-dist/24)`) | the entire surviving veneer (the residue term is now 0 m — `geology.rs:495-507`) is this closed form | **hydrology** (real discharge/transport at collapse scale) | 12.0 / 3.0 / 30.0 / 0.85 / 24.0 — **ARBITRARY** | provable | value-level | **every placer in the world** — journal/0055 measured that rounding this term away deleted all of them. |
| 18 | `wilds_regolith(cell)` | `collapse.rs:1446-1454` (`wilds_regolith_voxels`), fallback `:1246`; twin `geology.rs:509` (`None => 1.0 + ctx.precip*2.5`) | precip → 3/2/1 voxels | — | **Genesis, permanently legitimate** (stubs.md § Genesis) — there is no deep-time run outside the grid | n/a | value-level | **verdict: not a seam.** Listed so the sweep is complete. |

### dc-worldgen — `deeptime/`

| # | id | site | today | owed by | identity fallback | byte-id | gran. | blast |
|---|---|---|---|---|---|---|---|---|
| 19 | `coal_rank(unit, depth, T)` | `deeptime/recorder.rs:420-430` (`promote_coal`), threshold `biotic.rs:83` `COAL_MIN_M = 0.4`, called `biotic.rs:540` | a buried peat unit **thicker than 0.4 m** becomes coal. Thickness, not burial depth, not temperature | **diagenesis** | 0.4 m — **ARBITRARY**, and it is the *wrong axis*: rank is a P/T/time function, and `CLASS_ORGANIC_COAL`'s own doc says "**the class's depth axis is the rank axis**" (`materials/geology.rs:56-61`). The record already carries burial depth (`geology.rs:394-401`). | provable (`min_m = ∞` ⇒ no promotion) | value-level | every coal seam in the world; hence `deep_class`, hence surface material. Not in stubs.md. |
| 20 | `energy_band(capacity)` / `aridity(precip)` | `erosion.rs:445-457` (`energy_band`), `:462-473` (`tag_of`), aridity cut `0.32` at `:468` | the record's entire vocabulary is 2 env × 2 aridity × 3 energy = 12 buckets; thresholds `0.005`/`0.02` and `0.32` | **the recorder itself / stratigraphy** | thresholds — **ARBITRARY** | provable | value-level | the whole strata record's facies geography. stubs.md § 2 residual explicitly asks whether "the recorder's energy banding should [grow]". |
| 21 | `sea_level_at(epoch)` | `deeptime/grid.rs:222-229`, knobs `:253-255` (`sea_level_amp: 35.0`, `sea_level_period: 50`) | one deterministic sinusoid | **paleoclimate** (epoch-indexed curves) | `amp = 0` ⇒ static `SEA_LEVEL_M` — **TRUE IDENTITY, already wired** (`:223-225`) | provable | pass-level (a per-epoch scalar; a new `Resource`) | the marine/terrestrial split of the entire deep record. *stubs.md § 9.* |
| 22 | `parent_material_phosphorus(cell)` | `biotic.rs:289` `P_ROCK_INIT = 1.0`, used `:291`, `:453`, `:456` | every cell starts with the same P — granite and basalt pretended equal | **ecology** + **materials** | 1.0 uniform is a **TRUE IDENTITY** for a normalized-to-parent provider | provable | pass-level (a P plane) | retrogression/paleosol geography. *stubs.md § 8.* |
| 23 | the 7-species niche roster | `biotic.rs:137-270` (`NICHES`), `ROSTER = 7` at `:87` | a fixed struct array. Module doc: "A registry-defined organism pack **would** publish these; the spike ships a fixed vanilla roster" | **ecology** (a registry, exactly like `GeologySet`) | the vanilla 7 — **ARBITRARY** but the *shape* is right: `Niche` is already a contract over the same context axes `GeoMemberDef::window` uses | provable | value-level over a registry (pass-level input) | all organic facies. *stubs.md § 7.* Note the asymmetry: geology content is a **registry** (`GeologySet`, packable, canonically ordered); ecology content is a **const array**. |
| 24 | `background_propagule` | `biotic.rs:279-284` `BACKGROUND_PROPAGULE = 0.03`, `PIONEER_PROPAGULE = 0.12`, `ARRIVE_HALF = 0.15`; used `:701-708` | a species pool that exists everywhere at a trickle. **Self-flagged in code**: "FLAGGED design choice (ecology.md is silent on world-genesis colonization)" (`:695-700`) | **ecology** (a regional species pool / biogeography) | 0.03 — **ARBITRARY**; setting it to 0 **deadlocks the model** (the code says so) so there is *no* identity | provable but not neutral | value-level | which niches ever exist at all. |
| 25 | `fire_regime(cell)` | `biotic.rs:340-355` `FIRE_CHAR_FRAC 0.02`, `FIRE_BASE_KILL 0.75`, `FUEL_MIN 0.15`, `IGNITION_SCALE 0.15`, `CHAR_MIN_BAND 0.020`; `fire_kill` `:900-907` (species 5 → ×0.3, species 6 → ×0.2, **hardcoded indices**) | ignition probability and kill fractions as constants; two species identified **by array index** | **ecology** | `IGNITION_SCALE = 0` ⇒ no fires — **TRUE IDENTITY available**, not wired as a flag | provable | value-level | charcoal beds; fire-adapted community geography. The magic indices `5`/`6` are a hard coupling between two modules' notion of the roster. |
| 26 | `flood_regime(cell)` | `biotic.rs:357` `FLOOD_AREA = 120.0`, used `:880-888` with a bare `0.15` probability | drainage area > 120 ⇒ 15 % chance of a cover reset | **hydrology** (real flood recurrence) | 0.15 / 120.0 — **ARBITRARY** | provable | value-level | riparian community geography. |
| 27 | `soil_hiatus(cell)` | `biotic.rs:335-337` `SOIL_HIATUS_MAX = 0.010`, `PEAT_HIATUS_MAX = 0.030`; used `:832-838` | a horizon forms only where mineral deposition is under the cap | **pedology / ecology** | **ARBITRARY** | provable | value-level | where paleosols can exist at all. |
| 28 | `exhum` / `t_crust` — planes nothing reads | `deeptime/field.rs` (~164), summed into `resident_bytes` `:410-411` | populated, documented in-code as "the metamorphic-grade axes the collapse tier reads" — **and nothing reads them** | **metamorphism** | n/a (no consumer) | trivial | pass-level | zero today. *stubs.md § 4.* Same inverse-defect shape as S9: a provider with no seam. |
| 29 | `outcrop_at(cell)` | `deeptime/lithology.rs:365 ff.` (`exposed_litho`) — takes `units.last()` | "which unit outcrops here" == "the last one", because the record is a flat stack | **structural geology** (the dip/fold term) | `units.last()` — **ARBITRARY once deformation exists**, correct today | provable | value-level | *stubs.md § Sibling gap* ("Layer-cake strata"). The module docs already isolate it: "Only the one function changes." **This is the best-shaped seam in the codebase** — a single accessor with a named heir. |

### dc-worldgen — `pregen/`

| # | id | site | today | owed by | identity fallback | byte-id | gran. | blast |
|---|---|---|---|---|---|---|---|---|
| ~~30~~ **VOID 2026-07-28** | `habitability(cell)` | ~~`pregen/history.rs:302-347` (`plan_slots`)~~ **file deleted** | `score = 2.2·precip + comfort + 0.8·river + 0.5·coast − elev/3000`, gated by `temp ∈ (−3, 32)` and `precip ≥ 0.10`; `comfort = 1 − \|T−14\|/30` | **social sim** (dwarf-fortress-class settlement logic) | the closed form — **ARBITRARY** throughout (7 magic weights) | provable | value-level | where every settlement in the world is. Not in stubs.md. |
| ~~31~~ **VOID 2026-07-28** | `sack_probability` / `expand_probability` | ~~`history.rs:36` `EXPAND_PROB = 0.7`; `:219` `p_sack = if contested { 0.55 } else { 0.30 }`; adjacency radius `:352-364` (Chebyshev ≤ 3); founding wave `:127` (`slots.len()/8` clamped 2..20)~~ **file deleted** | polity dynamics as four constants | **social sim** | **ARBITRARY** | provable | value-level | the entire pregen history and every ruin in the world. |
| 32 **unreached since 2026-07-28** | `prior_pressure_weights(region)` | `crates/dc-sim/src/statistical/world.rs:280-287`; consumer `frontier_pressure` `:290-307` | `[0.60−0.50d, remainder, 0.15+0.70d]`. **Self-described**: "In the real system this would come from the statistical tier's cached summaries; here it is a closed-form stand-in" | **the statistical tier's own cached summaries (S2)** | **ARBITRARY** | provable | value-level | edge-region history synthesis. *stubs.md § 10.* |
| 33 **unreached since 2026-07-28** | `base_danger(region)` / `agent_transition` | `dc-sim/statistical/world.rs:214-216` (`base_danger` = a pure hash of the seed, `[0, 0.5]`); `:245-274` (`pressure_transition`, 6 magic coefficients); `:311-330` (`agent_transition`, `p_die = [0.002, 0.012, 0.06]`, behaviour weights per pressure) | the entire agent/pressure model is hash-and-table | **social sim** | **ARBITRARY** throughout | provable | value-level | all pregen history; all live far-sim. The `ToyWorld` name is honest about it. |
| ~~34~~ **VOID 2026-07-28** | `ruin_form(site)` | ~~`crates/dc-worldgen/src/collapse.rs:1381-1402` (`ruin_posts`)~~ **function deleted** | ≤10 wood posts, 2–4 voxels tall, hashed angle 0..τ and radius 6..18 | **social sim + ecology** | the posts — **ARBITRARY** | provable (empty vec) | value-level | the only world-visible form of settlement history. *stubs.md § 1;* carries a loud in-code marker at `:1374-1380`. |

### dc-core — `materials/`

| # | id | site | today | owed by | identity fallback | byte-id | gran. | blast |
|---|---|---|---|---|---|---|---|---|
| 35 | `block_twin(material)` | `crates/dc-core/src/classify.rs:60-92`; called `collapse.rs:732`, `:838`, and every `classify` path | a hardcoded 17-arm material→`Block` table with `_ => Block::Stone` | **renderer / presentation** (the block vocabulary is a *summary tier* for a material world) | `Block::Stone` — **ARBITRARY, and it is the loud one**: the code comment itself lists nine materials that fall through it ("gold dust and olivine … snow, leaf litter, ash, scree, bone, potsherds and knapping debris are loose materials the block vocabulary has not grown a summary for"). Every one of them renders as generic stone. | provable | value-level | every voxel's rendered identity. **This is the S1 defect's structural cousin**: a summary tier the world's real answer must agree with, enforced only by "every member of a vanilla class shares a block twin" — an invariant asserted in one test (`collapse.rs:1597`) and relied on in four places. |
| 36 | `stratification_time(material)` | `crates/dc-core/src/materials/stratify.rs:23` `STRATIFY_HALF_TIME = 1_000`, `:26` `STRATIFY_FULL_TIME = 16_000` | settling rate is **material-independent**: clay and gravel band at the same speed | **materials** (`grain_size_mm` / `density_kg_m3` already exist and `settle_energy` already combines them — `materials/geology.rs:446-448`) | the two constants — **ARBITRARY** | provable | value-level | derived banding views. Zero production callers today (only `dc-core/tests/s8_properties.rs`) — another provider-without-a-seam. |
| 37 | `porosity` / `permeability` of a rock, in the water spike | `crates/dc-worldgen/src/water/sat.rs:19-28` (`RockProps`), doc: "**In production these come from the property sheet** (materials.md: porosity from structure fill, permeability a granular property); the spike carries a small table so it stays standalone" | a caller-supplied `Vec<RockProps>` | **materials** — and `MaterialProps::permeability` **already exists** (`materials/mod.rs:197`), and porosity is already derivable from `VoxelContents` (`contents.rs:312` "Complement is porosity") | n/a (standalone spike; `water/mod.rs:4-6`: "nothing in the production world-generation path calls this module") | trivial | value-level | zero today. The seam is *declared in a doc comment* and unbuilt. |
| 38 | physics surface parameters | `crates/dc-physics/src/world.rs:97-104` — `ITEM_FRICTION 0.7`, `ITEM_RESTITUTION 0.2`, `TERRAIN_FRICTION 0.9`, `TERRAIN_RESTITUTION 0.0`, `PROP_FRICTION 0.9`, `PROP_RESTITUTION 0.05` | friction/restitution by *object category*, never by material | **materials** (a friction axis; none exists on `MaterialProps` today) | the six constants — **ARBITRARY** | provable | value-level | how everything slides. Ice and gravel are the same surface. Not in stubs.md. |

---

## C. Payload — what each unbuilt system is expected to supply

Ordered by seam count. Signatures are proposals; the *call sites* are real.

### 1. ECOLOGY — 9 seams

The largest owner, and the one with the most already-correct scaffolding
(`bio_resist` / `bio_weather` are a working byte-identity precedent).

```rust
// value-level, per deep cell, per epoch
fn root_cohesion(cell) -> f32              // identity 0.0   [S4]  TRUE IDENTITY, wired
fn biotic_weathering(cell) -> f32          // identity 1.0   [S4]  TRUE IDENTITY, wired
fn surface_material(vx, vz) -> Contents    // identity: the near path's own classify()  [S1]
fn charcoal_expression(unit) -> Expression // identity: fold into mineral host  [S6]  ARBITRARY
fn parent_p(cell) -> f32                   // identity 1.0   [22] TRUE IDENTITY
fn fire_ignition(cell, fuel) -> f64        // identity 0.0   [25] TRUE IDENTITY (unwired)
fn horizon_hiatus_cap(cell) -> f64         // identity: 0.010 / 0.030  [27] ARBITRARY
fn background_propagule(species) -> f32    // NO IDENTITY: 0.0 deadlocks the model  [24]
// registry-level
type NicheRegistry                         // identity: the vanilla 7  [23] ARBITRARY
```

**The structural gap**: geology content is a **registry** (`GeologySet` —
packable, canonically ordered by namespaced id, abundance-normalized, with
define-time validation). Ecology content is `const NICHES: [Niche; 7]`
(`biotic.rs:137`). Two systems, same job, one has a plugin surface and one does
not. `Niche` (`biotic.rs:93-135`) is already shaped like `GeoMemberDef` — it is
a tolerance window over the same context axes.

### 2. HYDROLOGY — 6 seams

```rust
fn depth_to_water(cell) -> f64             // NO IDENTITY  [S5]  ← the dangerous one
fn wave_energy(cell) -> f64                // identity 0.05 (rate<=0 ⇒ early return)  [S8] TRUE IDENTITY, wired
fn fits_in_pores(filler, host) -> bool     // BUILT, TESTED, ZERO CALLERS  [S9]
fn fan_energy(vx, vz) -> f64               // identity: width×exp(-d/24)  [17] ARBITRARY
fn flood_recurrence(cell) -> f64           // identity 0.0  [26] TRUE IDENTITY
fn rock_pore_props(material) -> RockProps  // seam declared in a doc comment only  [37]
```

**The asymmetry worth naming**: `docs/design/water.md`'s own model is *already
implemented* in `crates/dc-worldgen/src/water/sat.rs` — a saturation field with
the water table **read** as the top of the saturated zone, plus a body graph
(`water/body.rs`) and an O(1) connectivity index (`water/conn.rs`). Meanwhile
`biotic.rs:645-647` invents a water table out of three magic numbers. Provider
and consumer are both in the same crate and have never met
(`water/mod.rs:4-6`: "nothing in the production world-generation path calls this
module"). **This is the single clearest instance of the user's own point.**

### 3. MATERIALS — 6 seams

```rust
fn class_properties(litho) -> MaterialProps  // identity: the 6-id table  [S2] ARBITRARY, high blast
fn is_granular(material) -> bool             // identity: the 3-class test  [S3] diverges for packs
fn block_summary(material) -> Block          // identity: the 17-arm table + Stone  [35] ARBITRARY
fn settling_time(material) -> (u64, u64)     // identity: 1_000 / 16_000  [36] ARBITRARY
fn friction(material) -> (f32, f32)          // identity: the 6 category constants  [38] ARBITRARY
fn settle_calibration() -> f64               // identity 4.0  [15] honestly labelled
```

Note three of these (`is_granular`, `settling_time`, `friction`) are **axes the
property sheet does not have** — `MaterialProps` carries `grain_size_mm`,
`density_kg_m3`, `cohesion`, `permeability`, `solubility`, damage resistances,
but no `form` and no `friction`.

### 4. SOCIAL SIM — ~~5 seams~~ **1 nominal, 0 reachable (2026-07-28)**

```rust
fn habitability(cell) -> f64                 // identity: the 7-weight score  [30] ARBITRARY
fn expand_probability(polity, epoch) -> f64  // identity 0.7  [31] ARBITRARY
fn sack_probability(site, contested) -> f64  // identity 0.55 / 0.30  [31] ARBITRARY
fn base_danger(region) -> f64                // identity: a seed hash  [33] ARBITRARY
fn ruin_form(site) -> Vec<Structure>         // identity: 10 posts  [34] ARBITRARY
```

All five are already **acknowledged wholesale** by the user's standing
disposition (stubs.md § 1): culture artifacts are placeholder as a class, do not
bandaid, until ecology lands. Listed for completeness, not for conversion.

### 5. PALEOCLIMATE — 3 seams

```rust
fn paleo_temperature(cell, epoch) -> f64     // identity: present-day temp_c  [11] ARBITRARY (wrong axis)
fn paleo_precipitation(tag) -> f64           // identity 0.12 / 0.60  [12] ARBITRARY
fn sea_level_at(epoch) -> f64                // identity: amp=0 ⇒ flat  [21] TRUE IDENTITY, wired
```

### 6. DIAGENESIS — 3 seams

```rust
fn coal_rank(unit, burial_m, temp_c) -> Biofacies  // identity: thickness ≥ 0.4 m  [19] ARBITRARY, WRONG AXIS
fn accessory_presence(cell, depth) -> f64          // identity 0.6  [13] ARBITRARY
fn fits_in_pores(...)                              // shared with hydrology  [S9]
```

`coal_rank` is the sharpest single finding after S5: `CLASS_ORGANIC_COAL`'s own
docstring says "**the class's depth axis is the rank axis** — a pack that wants
lignite/bituminous/anthracite members discriminates them on burial depth, which
is the real control" (`materials/geology.rs:56-61`), while the code promotes on
**thickness** (`recorder.rs:426`). Doc and code disagree about which axis rank
rides on, and the doc is right.

### 7. METAMORPHISM / STRUCTURAL GEOLOGY — 3 seams

```rust
fn outcrop_at(cell) -> Litho                 // identity: units.last()  [29]  ← best-shaped seam in the tree
fn emplacement_depth(cell) -> f64            // identity 250.0 / 5.0  [10] ARBITRARY
fn metamorphic_grade(cell) -> Grade          // exhum/t_crust: planes built, nothing reads  [28]
```

### 8. ORE GENESIS — 1 seam

```rust
fn lode_endowment(catchment) -> f64          // identity: uniform 1.0 (a missing factor, not a wrong value)  [14]
```

### 9. RENDERER / PRESENTATION — 1 seam

```rust
fn block_summary(material) -> Block          // [35], shared with materials
```

---

## D. Content set and world identity — what the code does today

The user has ratified: *the generated world must not change after creation;
plugins that affect generation cannot be added to an existing world; render
packs and non-generating content can.*

**What exists:**

- **World identity is `{seed, extent}` and nothing else.**
  `crates/dc-worldgen/src/pregen/mod.rs:113-117`:
  ```rust
  pub struct WorldParams { pub seed: u64, pub extent: Extent }
  ```
  Not `Serialize`. `Extent` is (`:70-77`).
- **The geology set is a constructor argument, not world state.**
  `WorldGenerator::new` calls `Self::with_geology(pregen, vanilla())`
  (`collapse.rs:294`). `with_geology` / `try_with_geology` /
  `with_geology_owned` / `try_with_geology_owned` (`:301`, `:309`, `:327`,
  `:332`) take a `GeologySet` by value. The generator holds it as a field
  (`:279`) for its lifetime. Nothing records which set was used.
- **There is validation, but it is satisfiability, not identity.**
  `build_checked` (`collapse.rs:346`) enforces the pass graph's `selects`
  contract: a world **refuses to build** if any class a registered pass selects
  from has zero members (`pipeline.rs:104-110`, `geology.md § unfilled slots`).
  This catches "a required class is empty". It does **not** catch "a different
  set than last time".
- **No `GeologySet` hash, digest, or fingerprint exists.** No content-set
  identity value is computed anywhere in the tree.

**What does not exist:**

- **There is no save layer at all.** `crates/dc-api/src/host.rs:172` — "until
  the save layer exists to spill it." `HostWorld::new` (`:254-278`) builds
  everything from `seed` and in-memory `BTreeMap`s; no `std::fs`, no
  serialization of world state, no load path. ROADMAP:1237-1243 has it as a
  named heir ("the decided save/persistence layer"), unbuilt.
- Therefore **there is no load-time validation of anything**, content set
  included, because there is no load.

**The eviction consequence, stated precisely.**
`HostWorld` bounds *generated-and-untouched* chunks as an LRU cache
(`host.rs:154-165`, `:449-478`), evicting them on the stated ground that "a
generated-untouched chunk is a pure function of (seed, generator, pos) and can be
dropped and re-derived byte-identically at any time" (`:168-171`). Edited chunks
are pinned and never evicted (`:500-532`).

That purity claim is true **only for a fixed generator**, and the generator holds
the `GeologySet` by value. So:

- If the content set were to change **mid-session** — via a new
  `WorldGenerator` over a different set — every already-visited,
  never-edited chunk would re-derive **differently** the next time the player
  looked at it, with no signal. Edited chunks would keep the old rock. There is
  no mechanism today that could do this (the generator is constructed once and
  moved into the client authority), but nothing structurally prevents it either:
  `with_generator` (`host.rs:283-287`) takes any closure and there is no identity
  check on it.
- If a save layer landed tomorrow with today's `WorldParams`, a reload with a
  **missing or different plugin** would produce exactly the silent divergence
  the ratification forbids: `{seed, extent}` would match, the world would load,
  edited chunks would replay from the log, and every un-edited chunk would
  re-derive under the new set. Nothing would report it.

**Sharpest statement of the gap**: the content set is load-bearing input to a
function whose output is treated as reproducible, and it is not part of the
identity of that function's world. The class-satisfiability check proves the set
is *usable*; nothing proves it is *the same one*.

*(Related, already recorded: ROADMAP:2552 — "its ids are
generation-order-dependent — a save layer must…". And ideas.md § Namespace
deltas:189 already states the intended rule for the patch mechanism: "Patch set
joins pack set in world identity." The rule is written; the field is not.)*

---

## E. Ranked shortlist — best (blast radius × cheapness) to convert first

Ranked by *blast radius if the provider ever differs* divided by *cost to
introduce the accessor*. All nine of the top ten already have, or can trivially
have, a true identity fallback.

| rank | seam | why first |
|---|---|---|
| 1 | **[29] `outcrop_at(cell)`** (`lithology.rs:365`) | The cheapest seam in the tree with a real heir. One function, one call shape, module docs already say "Only the one function changes." Identity = `units.last()`. Unlocks the layer-cake/dip-fold term (stubs.md § Sibling gap) with zero behaviour change. |
| 2 | **[S8] `wave_energy(cell)`** (`erosion.rs:1478`) | Identity fallback **already wired** (`rate <= 0.0` early return) and already **byte-identity-tested** (`tests/full_agents.rs:83`). Both provider inputs exist in-tree (`water::BodyGraph`, `climate::zonal_wind`). Heir named in ROADMAP:1327. Highest ratio of "coastlines fixed" to "lines changed". |
| 3 | **[S5] `depth_to_water(cell)`** (`biotic.rs:645-647`) | Biggest blast radius in the inventory (all organic facies → `deep_class` → surface material) and the only seed-set seam with **no identity path at all**. Even converting it to a `wet_at(cell)` accessor over an empty plane — the `wmult_at` pattern — buys the byte-identity proof the other four planes already have. Do this before anything reads `wet` a fifth time. |
| 4 | **[19] `coal_rank(unit, depth, T)`** (`recorder.rs:420`) | Doc and code disagree about the axis, the right input (burial depth) is already carried in the record, and `promote_coal` is 10 lines with one caller. A doctrine-perfect case: a thickness threshold standing in for a P/T answer. |
| 5 | **[S3] `is_granular(material)`** (`fill.rs:310`) | Two call sites, a 3-way string compare, and a **demonstrated divergence**: `tests/geology.rs` already registers a pack where the material answer and the class answer differ. The divergence is in-tree and untested at the form tier. |
| 6 | **[S2] `material_properties(litho)`** (`lithology.rs:283`) | 6-entry domain, one caller, drives every erosion rate in production. Cheap to wrap; the hard part is design (class means vs one member's sheet), which is exactly the conversation this inventory should start. |
| 7 | **[35] `block_twin(material)`** (`classify.rs:60`) | The `_ => Block::Stone` arm silently summarizes nine named materials to generic stone. Same structural defect as S1, one tier down, and the invariant that makes S1 safe ("every member of a class shares a block twin") is asserted in exactly one test. |
| 8 | **[S9] `fits_in_pores`** (`packing.rs:62`) | Zero cost — it is already built and tested. What it needs is a *declaration* that hydrology and diagenesis are expected to call it, so the next person who writes an infiltration path finds it instead of writing a second rule. The inverse-defect exemplar. |
| 9 | **[22] `parent_p(cell)`** (`biotic.rs:289`) | True identity (uniform 1.0), one plane, pass-level, slots straight into the `Resource` vocabulary. The cheapest pass-level conversion available. |
| 10 | **[17] `fan_energy(vx, vz)`** (`geology.rs:508` + `collapse.rs:1307`) | Five magic numbers that are now the *entire* surviving veneer (the residue term measures 0 m at every named site) and therefore the sole cause of every placer in the world. High blast, and journal/0055 already proved the sensitivity the hard way. |
| 11 | **[11] `paleo_temperature`** (`geology.rs:402`) | One line, one caller, already in stubs.md § 6, and its sibling axis (`deep_precip`) already reads the record correctly — so the asymmetry is visible in adjacent lines of the same function. |
| 12 | **[28] `exhum` / `t_crust`** (`field.rs:164`) | Zero blast (nothing reads them) but the *comment claims otherwise*, which is corrections #29's exact failure class. Cheapest possible fix: either wire the seam or correct the claim. |

---

## F. Two cross-cutting observations

**1. The pass graph declares resources, not providers.**
`pipeline::Resource` (`pipeline.rs:47-79`) is a closed 10-variant enum of
*fields*: Plates, Elevation, Provenance, Climate, Hydrology, History,
DeepElevation, DeepStrata, Strata, Alluvium. Every pass declares `reads` /
`writes` over it and the graph topo-sorts and rejects cycles, multiple creators,
and ambiguous writer pairs — a genuinely good mechanism. But **26 of the 34 seams
above are value-level**, and the graph cannot see them: a per-cell function has
no `Resource` to name. Six of the pass-level seams (S5's heir, S8's body-graph
input, [21] sea level, [22] the P plane, [23] the niche registry, [28] the
metamorphic planes) would extend the existing vocabulary naturally. The other
twenty need a mechanism that does not exist.

**2. The byte-identity pattern is already solved and under-used.**
Four flags prove "provider absent == today" with tests. The *mechanism* is three
lines: an empty plane plus an identity accessor (`wmult_at`, `frost_at`,
`eff_diff` — `erosion.rs:186`, `:311`, `:322`). Of the 34 seams, exactly **6**
use it (S4×2, S8, [21], and the two erodibility/frost siblings). Another **9**
could adopt it with a true identity value and no design work. The remaining
**19 have ARBITRARY fallbacks** — which is the doctrine's own trap, stated in the
user's terms: a non-identity fallback means "provider absent" and "provider
present but silent" are different worlds, and nothing tells you which one you
are in.
