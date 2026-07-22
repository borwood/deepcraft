# Deep-time information-vector audit — feedstock for the form-from-provenance design pass

Read-only audit, 2026-07-21, repo at `main` (2434f37). No builds run, no repo files touched.
Every code claim carries `file:line`. Doc-vs-code disagreements are called out in **CODE≠DOC** blocks.

---

## A. Priors — what the corpus already decided

### A1. RATIFIED / DECIDED (user-owned — do not re-litigate)

| # | Decision | Where |
|---|---|---|
| P1 | **Fractions come only from the ledger.** Partial occupancy expresses *recorded* quantity and *recorded* variance — never cosmetic noise. | materials.md § "The forms design pass" ratification 1 (`docs/design/materials.md:203-212`) |
| P2 | **Sand is a FORM of an existing clastic material, not a new identity.** Within-identity grain gradation is not modelled. Accepted cost: loose and structural clastic are textured identically. | materials.md:214-222 |
| P3 | **The soil model.** Anything with roots carries a **root material in STRUCTURE form** — porous, holding soil in its pores. Anti-erosion for free. Soil placements come **from simulation, in both forms**: structural dirt layers (most structure slots filled), loose layers above, packing downward into porous rock below. | materials.md:223-231 |
| P4 | **Grass is suspended.** Do not express it. "It'll be a mostly brown world for a bit" — pre-ratified. | materials.md:232-245 |
| P5 | **The fill contract.** Contents are authoritative; `classify(contents) -> Block` is a pure derivation; `block == classify(contents)` is the regression invariant. | ARCHITECTURE.md § "The fill contract"; materials.md:246-283 |
| P6 | **Distribution-first expression** — metres survive to the voxel boundary, quantization happens **once** at contents construction, via **addressed stochastic rounding** (dithered eviction). Floyd–Steinberg forbidden (order-dependent). | materials.md:357-435; built in `crates/dc-worldgen/src/fill.rs` |
| P7 | **Where a record exists, the record decides what the world is skinned with.** Surface voxel takes its block from `classify(contents)`; veneer block rule deleted. | materials.md:437-481 |
| P8 | **Pore packability rule.** `filler_grain ≤ K_PORE × min(host grain)`, `K_PORE ≈ 0.25`, one shared helper `fits_in_pores`, consulted by every *transport-time* depositing process. **Exemption: formation-context/genesis emplacement bypasses it** (olivine in basalt grew in place). | materials.md:167-184; built at `crates/dc-core/src/materials/packing.rs:33,60` |
| P9 | **Resistance is agent-specific, never one scalar.** Limestone makes cliffs (mechanically strong) AND caves (chemically soluble); one number forecloses karst. | earth-processes.md:129-144; `deeptime/lithology.rs:8-44` |
| P10 | **Erosion-agent roster ON in production** (wind+frost+wave). Magnitudes for wind/frost RATIFIED as-built; **wave magnitudes NOT ratified and retune STRUCK** — "that whole mechanism changes after water machinery… would be a bandaid". Heir: wave energy as fetch from S11 body graph × the 0037 wind field. | earth-processes.md:186-201, 367-380; ROADMAP.md:1323-1329 |
| P11 | **Water is ONE conserved quantity in two regimes** (bound = pores + empty eighths of loose partials; free = open space). **The water table is READ, not modelled.** Vadose/phreatic falls out for free. **Aquifer/aquitard are material facts** — and the **loose-vs-packed soil permeability contrast is what makes an aquitard**. S10's waterlogging proxy has a *defined retirement*. | water.md:124-174 |
| P12 | **S11 calls ratified**: `Pinned` reservoirs default ON but toggleable; the ~12-cell halo is a **knob, not a constant** ("where there's a cell range, there's a knob"). | water.md:289-315 |
| P13 | **Fracture = per-damage-type outcome weights**, not one brittleness scalar. **Inventory = mass/volume with encumbrance behind a realism knob.** | materials.md:143-153 |
| P14 | **Doctrine**: "coarsen the cause, never delete it and fake the appearance"; "the record is the world"; **no simulation-resolution edge may reach the eye as a square or analytic boundary**. | earth-processes.md:9-29, 240-242 |
| P15 | **No-bandaid**: interim mechanisms ride as-built; ratification flags only for user-owned forks. | user memory `no-bandaid-tuning`; ROADMAP:1323 (wave) is the live instance |
| P16 | **Phanerozoic register** — recorded span calibrates to ~500 Myr; chapter `c` spans a known Myr band. Pre-record basement age is procedural flavour ("procedural hacks for the boring billion"). | earth-processes.md:320-330; `recorder.rs:226-236` |

### A2. Measured (don't re-guess)

- **S9**: A tier 460 m = 14.1 s / 52 MiB; B (48 m) = ~63 min / ~3 GiB. Decay length **21 cells** hillslope, 26 fluvial. **Drainage/flow-area is the sole ADVECT**; everything else relaxes at 16–24 cells. (`docs/spikes/S9-results.md`; earth-processes.md:282-290)
- **S9b**: parallelism does not flip B. Priority-flood is 98.5 % of the step at B scale and has no byte-identical parallel form. (corrections #9)
- **S8**: free-form mixtures are storage-safe. k-material locale caps at `C(k+8,8)−1` states; 2 materials → 45, 6 → 3003, 4-material adversarial gradient → exactly 495. Real deposit chunks 0.14–0.38 B/m³. (materials.md:76-97)
- **S11**: bound-water halo **4–11 cells** bounded-budget, **0–6** for the player-visible integer water table; clean geometric decay, no advective tail. A **sharp aquitard makes it MORE local, not less**. Body graph grows with *components*, not edits: 217 bodies / 4 400 bytes after 1 624 edits and 508 084 dug voxels. (`docs/spikes/S11-results.md:22-27,100-139`)
- **journal/0053**: mean `H` = 4.75 m per subaerial cell; only ~1.15 m survived as whole-voxel strata pre-0055 (~75 % of the sediment pile deleted). Charcoal beds: mean 0.035 m over 158 310 beds, **0 survive 0.9 m quantization**.
- **journal/0047**: the eolian record costs **+92.76 MB at Medium** (`DeepField` 52.8 → 145.5 MB). Undiagnosed. (ROADMAP:1868-1875)
- **corrections #16**: tempering abrasion resistance by cohesion *destroys* rock contrast (mudstone is more cohesive than sandstone). Cohesion is right for wave/eolian, wrong for abrasion.
- **corrections #17**: coupling bedrock incision alone changes nothing — **bedrock→regolith weathering is the rate-limiting hillslope term** and is what must be coupled.
- **corrections #23**: `thickening_scale` buys no sub-km relief — it lifts the continent, it does not make mountains.
- **corrections #29**: partial-height did *not* light up for free; the assumption that made culling safe lived in a doc comment nobody re-read.

### A3. Standing gaps the corpus already names (relevant to this pass)

- **ROADMAP:1271-1291 — "Consume the ledger terms the runtime throws away", piece (c): *derive material FORM (loose / pore-partial / whole / inclusion) from provenance rather than leaving it implicit*.** This audit's question is already a Sequenced roadmap item, and it is named the priority piece.
- **ROADMAP:1728-1740 — "Organics render as solid boxes because form is keyed on CLASS."** First concrete visible symptom of the same gap.
- **ROADMAP:1832-1855 — there is no soil substance in the world.** `MaterialId::LOAM` is registered in **no** geology class member; `dc:stratum/organic-soil` has exactly one member, `dc:geo/carbonaceous-mudstone`, a lithified rock. **Fix order stated: substance first, then form-from-provenance.**
- **earth-processes.md:96-100 — the recorder needs a second entry species** (events that *transform prior units*: dikes, plutons, fault offsets, tilting). "Data-model decision; cheaper before than after." **Diagenesis is exactly such an event.**
- **earth-processes.md:342-345 — S9 owed work: "widen recorder tags (agent axis + grain continuum)".** Unbuilt.
- **ideas.md:274-291 — "Soil is loose, but packable into structural"**: overburden pressure + duration convert loose soil to packed structural; **most sub-surface soil generates already packed**; "plausibly the same machinery as the deep-time strata recorder"; the reverse (structural → loose on disturbance) is the natural pair. NOT decided, user-raised.
- **ideas.md:393-411 — "Rock is not monolithic"**: jitter at the partials scale, dependent on a material property, particularly for soft materials. Scoped by P1 to *property-dependent expression of real record variance*.
- **materials.md:72-74** — "Compaction closes the deep-time loop: debris under overburden, over ledger time, migrates into a structure slot as sedimentary stone. Worldgen strata, gameplay middens, and geology are one process at different tick rates." **This is the user's named vector, already written down in the design draft since 2026-07-18. It is not new — it is unbuilt.**

---

## B. The vector inventory

The production run loop is `deeptime/mod.rs:140-159`: per iteration `sea_level_at` → (periodic `climate::march`) → (tectonic blend) → `erosion.step` → `biotic.step`; then once at finalize `biotic.finalize` → `promote_coal`.

`Erosion::step` phase order is `erosion.rs:660-707`.

Production flags (`deeptime/field.rs:90-159`, `production_config`): `record: true`, `biotic: true`, `erodibility: true`, `tectonic_history: true`, `full_agents: true`. **Everything below is ON in production** except `apply_uplift` (legacy path, superseded by thickening+isostasy) and `Agent::Dissolution` (no call sites at all).

Legend for **record?**: does the vector write anything a player can eventually dig — i.e. a `DepUnit`/`DepTag` axis — or only elevation/mass planes?

### B1. Forcing / tectonic

| # | Vector | file:line | reads | writes | record? | material-aware? | form-aware? | cannot express |
|---|---|---|---|---|---|---|---|---|
| 1 | `apply_thickening` | `erosion.rs:716` | `forcing` plane (analytic, from chapter table) | `grid.t_crust` (floored ≥1000 m) | no | **no** | no | crust composition beyond `CrustKind` density; nothing about what the crust is made of |
| 2 | `apply_uplift` (legacy) | `erosion.rs:802` | `grid.uplift` (constant, from `provenance_uplift`) | `grid.r` | no | no | no | OFF in production (`tectonic_history: true` takes branch 1) |
| 3 | `track_exhumation` | `erosion.rs:745` | `r_snap` vs `grid.r` | `grid.exhum`, `grid.t_crust` | no | no | no | **exports the metamorphic-grade axis and nothing reads it** (`field.rs:224-232`) |
| 4 | `isostasy` (Airy, flexure-smoothed) | `erosion.rs:780` | smoothed `t_crust`, smoothed `h`, `crust_kind` | `grid.r` | no | density only via `CrustKind` | no | sediment *density* is not a function of what the sediment is or how compacted it is — `h_bar` is a thickness, not a mass |
| 5 | `TectonicSchedule::blend_into` | `mod.rs:223-239` | chapter table, `ramp_chapters` | `blended` forcing; **returns the chapter index the recorder stamps** | **yes** (`DepUnit::chapter`) | no | no | the chapter is a `u8` time index only; no strain, no dip, no deformation of prior units |

### B2. Climate / boundary

| # | Vector | file:line | reads | writes | record? | material | form | cannot express |
|---|---|---|---|---|---|---|---|---|
| 6 | `climate::march` | called `mod.rs:116,143`; `deeptime/climate.rs` | current surface, latitude, sea level | `grid.precip` (f32 0..1) | **indirectly** — `precip` decides `Aridity` in `tag_of` | no | no | no humidity/soil-moisture state; nothing about infiltration vs runoff — the only moisture in the sim is a normalized precipitation scalar |
| 7 | `sea_level_at` | `grid.rs:226` | `cfg.sea_level_amp` (35 m), `sea_level_period` (50 it) | the step's `sea_level` | **indirectly** — decides `DepEnv::Subsea/Subaerial` and the wave band | no | no | a pure sinusoid; no ice-volume, no glacial regime change |

### B3. Erosion / transport — the mass movers

| # | Vector | file:line | reads | writes | record? | material-aware? | form-aware? | cannot express |
|---|---|---|---|---|---|---|---|---|
| 8 | `expose` (lithology resolution) | `erosion.rs:845` | **top unit of `grid.strata[i]`** → `Litho` | scratch planes `litho`, `sus_flow`, `sus_creep` | no (scratch) | **yes** — `Litho` (6 classes) → `reference_material()` → `LithoResistance.abrasion` | **NO** | see CODE≠DOC #1 below. Empty record ⇒ `Litho::Basement` (`lithology.rs:376-381`) — that fallback is where shields come from |
| 9 | `periglacial` (frost) | `erosion.rs:923` | latitude, surface, top unit → `Agent::FrostIce` axis | `frost` multiplier plane | no directly (folds into `weather`) | **yes** — `frost_ice = smash × (1 − 0.5·permeability)` (`lithology.rs:335`) | no | freeze–thaw is gated by *material permeability* but the sim has **no water**; permeability is a property-sheet constant, not a saturation state |
| 10 | `flood` / `route` / `accumulate_area` | `erosion.rs:989,1033,1055` | `surf` | `filled`, `recv`, `area` | no (but `recv`/`area`/`lake` are **exported** at `field.rs:214-223`) | no | no | drainage is topology-only; no channel geometry, no bedload calibre |
| 11 | **`transport`** — fluvial entrainment + incision + deposition | `erosion.rs:1072-1149` | `area`, `filled` slope, `sus_flow`, `grid.h`, `grid.r`, `cfg.h_star` | `grid.h` (entrain/deposit), `grid.r` (incise), `self.dh`, `self.energy` | **yes, via `dh` → `record`** | **partially** — one axis: `sus = LithoResistance.abrasion` of the *outcropping* unit, applied identically to entrainment and incision | **NO** | **cannot express that loose cover entrains differently from lithified rock** — a bed's entrainability is read off its *lithified reference material*, so `H` (which is by definition unconsolidated) resists like sandstone. No cohesion term, no grain-size selectivity, no sorting during transport (the only "sorting" is the 3-band `energy_band` tag) |
| 12 | **`weather`** — bedrock→regolith | `erosion.rs:1159-1202` (kernel `:291`) | `grid.r+h > sea`, `cfg.weathering`, `cfg.h_star`, `bio_weather`, `sus_flow`, `frost` | `grid.r -= wth`, `grid.h += wth`, `self.dh += wth` | **yes, via `dh`** | **partially** — product `wmult × litho_sus × frost` (`erosion.rs:1180`) | **NO** | **this is the only place in the sim where solid becomes loose, and it changes only mass, never form.** The produced regolith enters the record tagged by whatever *depositional* flow happened to be passing (see CODE≠DOC #2). No saprolite/residuum facies, no chemical vs mechanical split (`lithology.rs:283-289` explicitly anticipates the sum-over-agents shape but there is one term) |
| 13 | `diffuse` — hillslope creep | `erosion.rs:1236-1294` (kernels `:206,234`) | frozen `surf`, `grid.h`, `bio_resist` (root cohesion), `sus_creep` | `grid.h`, `self.dh` | **yes, via `dh`** | **partially** — `eff_diff = diffusion × (1−bio_resist) × sus_creep`, weaker contrast knob (`grid.rs:71-77`) | **NO** | **no angle of repose, no material cohesion.** Creep rate is set by rock competence and root cohesion; the *cohesion of the loose material itself* — the property that decides whether a pile stands — is never read. This is the exact term P3's root-lattice model wants to differentiate loose from matrix-held soil |
| 14 | **`record`** — the strata recorder | `erosion.rs:1301-1318`; `record_cell:479`; `tag_of:462` | `r+h` vs sea, `precip`, `energy` (this step's stream capacity), chapter, `dh` | `grid.strata[i]` — `deposit(tag, dh, chapter)` or `erode(-dh)` | **THIS IS THE RECORD** | no — tags by *measured environment*, not material (`recorder.rs:1-19`) | **NO** | see § C/D. The tag space is `{env×aridity×energy×biota×eolian}` + `chapter` + `thickness_m` + `unconformity` bool. **There is no form axis, no porosity axis, no grain axis, no burial-history axis** |
| 15 | **`wind`** — eolian deflation + loess/dune | `erosion.rs:1351-1455` | `climate::zonal_wind(lat)` sign+magnitude, `precip` vs `eolian_arid_precip`, `bio_resist` (veg), `grid.h`, top unit → `Agent::Eolian` axis | `grid.h` (row-conservative), **self-records**: `strata[i].erode(pickup)` / `deposit(Eolian::Dune|Loess tag)` | **yes, directly** — the only vector that writes a *distinct facies marker* | **yes** — `eolian = cohesion.max(0.05)`, deliberately **no** grain competence (`lithology.rs:341`) | **NO** | it deflates by cohesion of the *lithified reference rock* of the top unit. A loose dune and a cemented sandstone with identical `Litho` deflate identically. Also: 1-D march dumps residual load at the downwind land edge (ROADMAP:1876) |
| 16 | **`wave`** — littoral attack | `erosion.rs:1477-1533` | freeboard band vs `wave_band_m`, adjacency to subsea, top unit → `Agent::Wave` axis | `grid.h` then `grid.r` at the shore cell; `grid.h[j]` offshore; **self-records** erode + a `Subsea/Humid/Low` deposit | **yes** | **yes** — `wave = smash × cohesion.max(0.05)` (`lithology.rs:338`) | **NO** | wave *energy* is a global constant (`wave_erosion` 0.05), not a fact about the water body. Ratified heir: fetch from S11 body graph × wind field (ROADMAP:1327). Cannot express a shore platform's own debris apron as a distinct form |

### B4. Biotic — the six processes (`biotic.rs`, `step_cell:570`)

All six are inside one per-cell function; they are separable read/write phases, listed as the doc describes them.

| # | Process | file:line | reads | writes | record? | material | form | cannot express |
|---|---|---|---|---|---|---|---|---|
| 17 | 1. Suitability | `biotic.rs:649-673` | `air_temp_c`, `precip`, `cell.soil`, `p_avail`, `n`, `wet` proxy | `suit[]` | no | no | no | soil *texture* — suitability sees a depth scalar, never what the soil is made of |
| 18 | 2. Dispersal | `biotic.rs:675-708` | frozen 8-neighbour `prev_cover` + background propagule | `pp[]` | no | no | no | — |
| 19 | 3. Competition | `biotic.rs:710-746` | `suit`, `pp`, incumbency/shade | `cell.cover[]` | no | no | no | — |
| 20 | 4. Nutrient cycling | `biotic.rs:748-796` | cover, temp, `wet`, `p_rock`/`p_avail`/`n`/`cations`; **rejuvenation from `d_surf` and `mineral_dep`** (`:629-635`) | nutrient pools, `net_org` | no | no — P release is `P_WEATHER_RATE × p_rock`, indifferent to parent lithology | no | **the phosphorus a rock yields does not depend on the rock.** Fresh-P is `strip × P_FRESH` regardless of whether the stripped material was granite or peat |
| 21 | 5. Niche construction | `biotic.rs:802-843` | cover × per-niche `root`/`weather`/`flammable` | **`grid.bio_weather`, `grid.bio_resist`** (next epoch's erosion modifiers), `cell.soil`, `org_tag` | **yes, via `org_tag.biota`** | no | **NO — this is where P3's root lattice would live and does not.** `resist` is a scalar 0..0.9 multiplied into diffusivity | root cohesion is expressed as a *rate multiplier*, never as a **material in structure form occupying slots**. The ratified soil model needs the latter |
| 22 | 6. Disturbance (fire, flood) | `biotic.rs:845-890` | `eff_fuel`, `dryness`, addressed draw; `area[i] > FLOOD_AREA` | cover kill, nutrient pulse, `charcoal` band, `tsd` reset | **yes** (`Biofacies::Charcoal`) | no | no | charcoal beds average 0.035 m and **0 of 158 310 survive quantization** — the honest form is an inclusion (pore/debris partial) and that form does not exist |
| 23 | biotic apply / deposit | `biotic.rs:515-533` | `Outcome` | `grid.h += org_deposit`, **`strata[i].overprint_top(...)`**, `grid.h += charcoal`, `strata[i].deposit(char_tag,…)` | **yes** | no | no | `overprint_top` (`recorder.rs:299-329`) is the **one existing operator that transforms a prior unit in place** rather than stacking — the template for a diagenesis operator |
| 24 | **`promote_coal`** — burial diagenesis | `recorder.rs:420-430`, called `biotic.rs:538-542` | unit is **not the topmost** (⇒ buried) AND `thickness_m ≥ COAL_MIN_M` (0.4) AND `biota == Peat` | `u.tag.biota = Coal` — **tag only, thickness preserved** | **yes** | no | no | **THE ONLY DIAGENESIS VECTOR IN THE WORLD.** Burial is a boolean ("something is on top of me"), not a depth; there is no pressure, no duration, no temperature, no bulk-property test. Runs once at finalize |

### B5. What is not there at all

- **`Agent::Dissolution`** — enum variant, `LithoResistance.dissolution` axis, `susceptibility_table` support (`lithology.rs:92-94,154-156,328-332`) — **zero call sites in the sim.** And `lithology.rs:454-465` tests that *nothing in the roster is soluble*, honestly: there is no carbonate.
- **`fits_in_pores` / `K_PORE`** (`packing.rs:60`) — **no production callers.** Grep over `*.rs` finds the definition and its own test module only. The ratified P8 rule is built and unwired.
- **`VoxelContents::is_occupancy_solid` / `free_eighths` / `loose_eighths` / `bound_eighths`** (`contents.rs:261-280`) — added by journal/0052 for four named future consumers (fluid fill, loose gravity march, compaction, collision). **The client's collision still reads `Block::is_solid`** (materials.md:326-328).
- **`DeepField.exhum` / `t_crust` / `chapters`** (`field.rs:224-240`) — populated in every production world, "currently consumed by nothing".
- **`crates/dc-worldgen/src/water/`** — standalone, not on the production path (§ E).

---

## C. The form axis — what exists today

### C1. What decides loose vs structural: two class-string tests, and that is all

```rust
// crates/dc-worldgen/src/fill.rs:310
fn is_loose(set: &GeologySet, m: GeoMemberIdx) -> bool {
    let class = set.member(m).class.as_str();
    class == CLASS_CLASTIC_FINE || class == CLASS_CLASTIC_COARSE || class == CLASS_ORE_PLACER
}
```

```rust
// crates/dc-worldgen/src/collapse.rs:1467
if class == CLASS_CLASTIC_FINE || class == CLASS_CLASTIC_COARSE { /* debris_only */ }
```

Consequences, precisely:

- **Form is a function of the content class string, nothing else.** Not of the depositional tag, not of burial, not of the material's own properties, not of anything the deep sim recorded beyond `deep_class(tag)`.
- `contents_for_event` (`collapse.rs:1464-1493`) emits exactly three shapes:
  - clastic/placer → `debris_only([host_mat; 8])` — **8/8 loose, zero pores, zero structure**;
  - igneous with an accessory → `StructureShape::Full` with `8−k` structure + `k` pore fill;
  - everything else → `StructureShape::Full`, `[host_mat; 8]` — **8/8 structure, zero porosity**.
  So **every non-contact voxel in the world is either 100 % loose or 100 % dense structure.** `structure_density` is 1.0 and `open_pores` is 0 essentially world-wide. There is no porous rock in the world except where an igneous accessory happens to ride.
- `mixed_contents` (`fill.rs:279-303`) is the only richer emitter: at a contact, structure-dominant → `smallest_shape(k)` with the loose grains as **pore fill**; loose-dominant → all-debris. It also carries **partial fills** (`fill.rs:274-278`): a sub-8 loose partial gets `StructureShape::None`, which is exactly `is_loose_only`.
- **Organics (soil/peat/coal) fall into the structure bucket** and render as solid cubes (ROADMAP:1728-1740). Real soil and peat are loose. This is the visible symptom of the whole gap.

### C2. What `VoxelContents` can represent (the representation is ahead of the generator)

`crates/dc-core/src/materials/contents.rs`:

- `StructureShape` reserves capacity: None 0 / Quarter 2 / Slab 4 / Full 8 (`:55-65`).
- Three multisets: `structure` (filled reserved slots), `pore_fill` (fines packed into reserved-but-unfilled slots), `debris` (loose, in unreserved volume) (`:192-209`).
- Derived: `solid_eighths`, `open_pores`, `free_debris_eighths`, `free_eighths`, `is_full`, `loose_eighths` (= debris only; **pore fill is NOT loose — the structure holds it**), `bound_eighths`, `has_structure`, `is_loose_only`, `is_occupancy_solid` vs `SOLID_EIGHTHS = 4` (`:217-280`, `:41`).
- Canonical, order-destroying by construction — deposit order is deliberately lost (`:12-17`).

**So loose / bound / free / pores / partial density are all fully representable today.** The missing thing is not the container. It is (a) a *provenance rule* that decides which role a material goes in, and (b) *state in the record* for that rule to read.

### C3. `packing.rs` / `K_PORE` — built, decided, unwired

`K_PORE = 0.25` (`packing.rs:33`); `fits_in_pores(filler, host_structure)` uses the **min** host grain (finest grains choke the throat) and returns `false` for an empty host (`:60-71`). Composability invariant: sieve resistance ≡ grain size, so what-packs-in and what-sieves-out-first are the same axis (`:132-155`). **Genesis is exempt** — the igneous accessory path never calls it (`:157-167`).

No production depositing process calls it, because no production depositing process packs anything into pores except the genesis-exempt accessory path.

### C4. What P2/P3 require that does not exist

**P2 (sand is a form of clastic-coarse)** — *de facto already true, by accident.* `dc:stratum/clastic-coarse` has one member `dc:geo/sandstone` whose material is `MaterialId::SANDSTONE` (`dc-core/src/materials/geology.rs:485-498`), and `contents_for_event` emits it into the **debris** role. So a "sand" voxel today is literally eight loose eighths of `SANDSTONE`. That is exactly the ratified shape — reached by a class-string test rather than by a form axis, and with the ratified accepted cost (identical texture) live. **The gap is that the same material can never be emitted in the *other* form from provenance:** a cemented sandstone bed and a loose sand blanket are the same class, so they are the same form, always.

**P3 (root-lattice soil model)** requires, in order of missingness:
1. **A root material** — does not exist in any registry.
2. **A soil material** — `MaterialId::LOAM` exists but is registered in **no** geology class member (ROADMAP:1832-1845). `dc:stratum/organic-soil` has one member, a lithified rock.
3. **Porous structure emission** — nothing emits `structure_len < shape.capacity()` except the accessory path.
4. **A form decision per horizon** — "loose above, packed downward" needs a depth/pressure axis at expression time. `deposit_deep_history` (`geology.rs:394`) *does* compute `depth_above` per unit, so present burial depth IS available at collapse. Nothing uses it for form.
5. **The anti-erosion payoff** — "loose soil with no matrix falls with gravity; the root lattice is what holds a slope" needs the deep sim's `diffuse` to read matrix presence. Today `diffuse` reads `bio_resist`, a rate scalar. Same physical claim, wrong representation: it is a multiplier on creep, not a structure that can be absent.

---

## D. The missing vectors

For each: what it would read, and whether that state exists.

### D1. **Compaction / cementation → lithification** (the user's named vector) — ABSENT

Loose material under sustained pressure becoming a structure containing the former particles.

- **Precedents that make this cheap.** Two operators already exist and are exactly the right shape:
  - `DeepStrata::promote_coal` (`recorder.rs:420`) — a finalize-time pass that **re-tags a buried unit while preserving thickness**. Burial test is `k != last`. Threshold is a thickness. This is diagenesis, already shipping, for exactly one material pair.
  - `DeepStrata::overprint_top` (`recorder.rs:299`) — **transforms a prior unit in place** (adds mass + retags + merges down), with two refusals that keep the record honest (never retag a `Charcoal` bed; never merge across an unconformity).
- **State it would need to read, and whether it exists:**

| input | exists? | where |
|---|---|---|
| present burial depth of a unit | **yes, derivable** | `geology.rs:394` computes `depth_above` at collapse; in-sim it is `Σ thickness` of overlying units, one pass over `DeepStrata::units` |
| unit age / duration under load | **coarse yes** | `DepUnit::chapter: u8` (`recorder.rs:236`) + the Phanerozoic register gives a Myr band per chapter. Resolution: `iterations/chapters` = 200/8 = 25 iterations ≈ 62.5 Myr |
| **maximum** burial ever attained | **NO** | a column uplifted and stripped has been deeper than it is now. `grid.exhum` holds *cumulative bedrock* exhumation per **cell**, not per unit, and only counts `R` lowering (`erosion.rs:745`). Nothing per-unit |
| temperature at depth (geothermal) | **NO** | `climate::air_temp_c` is surface air only. No geotherm anywhere |
| bulk properties of the mix | **yes, at collapse only** | `MaterialProps { density_kg_m3, grain_size_mm, cohesion, permeability, insulation, solubility }` (`dc-core/src/materials/mod.rs:175-219`). **Not available in the deep sim** — the deep tier knows only `Litho` (6 classes) and its fixed `reference_material` |
| a place to write the answer | **NO** | `DepTag` has 5 axes, none of them form; `DepUnit` has thickness/unconformity/chapter |
| water/cement supply | **NO** | § E |

- **What it would unlock immediately:** the mudstone/sandstone monoculture. Today `deep_class` routes every non-organic unit to a lithified clastic rock regardless of whether it was ever buried. A lithification rule would make the *top* of every column loose sediment and the *bottom* rock — which is what the user's soil model, the aquitard contrast (P11), and the "digging feels right" note (ideas.md:283) all want.

### D2. **De-lithification / unpacking on exhumation and disturbance** — ABSENT

Nothing ever moves material from structural back to loose in the record. `weather` moves mass `R → H` (`erosion.rs:301-305`) but that is a plane transfer, not a form change on a recorded unit; `DeepStrata::erode` (`recorder.rs:334`) only pops thickness. ideas.md:288 names it: "The reverse (structural → loose on disturbance) is the natural pair."

Needs: a form axis on the unit (D1's write target), plus the exhumation depth already tracked per cell.

### D3. **Weathering does not change form, and its product is mis-tagged** — PARTIALLY ABSENT

`weather_cell` (`erosion.rs:291-306`) converts bedrock to regolith. The produced metres land in `self.dh`, which `record` (`erosion.rs:1301`) then tags with `tag_of(surf, precip, energy, sea)` — **the *depositional* environment of whatever flow was passing that iteration**. So in-situ weathering product is recorded as a fluvial/marine deposit.

There is no `DepEnv::Residual` / saprolite facies. A weathering profile — the thing that makes soil a *profile* rather than a deposit — cannot be represented. This is a fidelity gap independent of form, and it compounds it: form-from-provenance wants to say "this is in-place weathered rock, therefore a porous, partly-disaggregated structure", and the record cannot distinguish it from river mud.

Also: `weather` is documented (`erosion.rs:283-289`) as a *sum over agents* with one term today. The dissolution term would slot in here — but see D4.

### D4. **Chemical dissolution / karst** — DESIGNED, AXIS BUILT, ZERO CALL SITES

`Agent::Dissolution` and `LithoResistance.dissolution` (`lithology.rs:92,154`) exist and are correct (immune = exactly `0.0` susceptibility, not the clamp floor — `lithology.rs:204-214`). Nothing calls them. And there is genuinely nothing soluble in the roster (`lithology.rs:454-465`).

Needs: (a) a carbonate material + `solubility > 0`, (b) circulating water — the phreatic zone, i.e. § E. water.md:151-157 already decides the payoff shape: vadose/phreatic falls out of the water table for free, no cave-morphology system needed.

### D5. **Porosity as recorded state** — ABSENT

`VoxelContents` can hold pores. `MaterialProps.permeability` exists. `LithoResistance.frost_ice` already *reads* permeability (`lithology.rs:335`). But **no recorded unit carries a porosity or a density**, so "how compacted is this bed" is unanswerable, which is why every non-contact voxel is 8/8 dense. Porosity is the natural *output* of D1 and the natural *input* to hydrology (P11: "aquifer and aquitard are material facts").

### D6. **Grain size / sorting continuum** — ABSENT (explicitly owed)

The only grain proxy in the record is `EnergyBand` (3 levels, thresholds `erosion.rs:449-457`). earth-processes.md:342-345 lists "widen recorder tags (agent axis + grain continuum)" as owed S9 work. P2 accepted *within-identity* gradation not being modelled, so this is about *between-unit* sorting: a fining-upward stack, a well-sorted dune vs a poorly-sorted till.

Note P8's composability invariant (grain size ≡ sieve resistance ≡ pore admission) means a grain axis in the record would immediately feed `fits_in_pores` — the three questions are one axis by construction.

### D7. **Recorder event-entries: transformations of prior units** — ABSENT, and this is the enabling data-model decision

earth-processes.md:96-100: "the strata record is a deposition-ordered stack and cannot express anything that *modifies previous entries*: dikes, plutons, fault offsets, tilting. The record needs a second entry species before the igneous engine lands its first dike. Data-model decision; cheaper before than after."

**Diagenesis is a transformation of prior units.** So is lithification, so is de-lithification, so is mineral replacement. `promote_coal` and `overprint_top` are the two places the codebase has already quietly needed this and solved it ad hoc (mutate the unit in place). Building D1 without settling D7 means a third ad hoc mutator.

### D8. **Bioturbation / mixing as a recorded process** — ABSENT

`geology.rs:475-479` argues in a comment that amalgamating thin beds into a surficial mantle "is not a fudge, it is the physical process: bioturbation, creep and soil mixing homogenize thin beds, which is why real soil is not laminated." That reasoning is sound and lives in a veneer comment rather than in the sim. materials.md:58-70 already decides the general shape: **stratification degree is a pure function of (contents, time since disturbance, agitation history) — never simulated per-voxel**, and worldgen deposits "arrive fully stratified". A mixing vector would be the deep-time end of that function.

### D9. **Angle of repose / cohesion-driven loose-material behaviour at the deep tier** — ABSENT

`diffuse` never reads `cohesion`. materials.md:94-97 lists "angle-of-repose settling" as owed from S8. P3's anti-erosion payoff depends on it: matrix-less loose soil should fall.

### D10. **Geothermal gradient** — ABSENT

No temperature-at-depth anywhere. `air_temp_c` is surface-only (`climate.rs`, used at `erosion.rs:942` and `biotic.rs:551`). Needed by any P/T-path metamorphism (earth-processes.md § 5) and by the "cooked" half of diagenesis. `exhum`/`t_crust` are exported for exactly this and read by nothing.

### D11. **Per-unit provenance query** — DESIGN PASS OWED (ROADMAP:1288-1291)

"read a voxel's ledger: started as X, heat/pressure did Y, moved because of Z." Flagged as needing a design pass first; the integrator's framing (that it constrains every stage to carry reasoning forward instead of collapsing to a final value) is **PROPOSED, not ratified**. This is the general case of which form-from-provenance is the first instance.

---

## E. Where hydrology plugs in

### E1. What S11 built, exactly

`crates/dc-worldgen/src/water/` — four modules, **additive and standalone; nothing in the production path calls it** (`water/mod.rs:1-20`; S11-results.md:3-8).

- **`sat.rs`** — bound water. One scalar per voxel, `sat ∈ [0,1]` = filled fraction of that voxel's **pore volume**; water = `sat × porosity`. Three gather sub-steps per iteration: infiltration into the topmost non-void voxel per column; gravity percolation limited by `min(k_here, k_below)` and receiver free pore space; permeability-limited lateral redistribution on head `H = y + sat`. Mass-exact and order-independent by construction. **The water table is read, never stored** (`sat.rs:1-18`).
  - `RockProps { porosity, perm }` with an explicit note: *"in production these come from the property sheet (porosity from structure fill, permeability a granular property); the spike carries a small table so it stays standalone"* (`sat.rs:20-29`).
- **`conn.rs`** — derived two-level connectivity index; never persisted.
- **`body.rs`** — persisted body graph (~20 bytes/body), `Pinned` vs `Finite` reservoirs.
- **`vox.rs`** — a **toy** one-bit solid/air volume with its own `is_solid` (`vox.rs:60`), deliberately not the production world. materials.md:279-282 already names this as the future asker that wants **free capacity in eighths**, not a bool.

### E2. What would have to change for a deep-sim vector to read saturation / water table

The deep tier is 460 m cells × 200 iterations; S11 is a per-voxel present-tier relaxation. They cannot be the same object. Three honest couplings, cheapest first:

1. **Read the drainage export that already exists.** `DeepField.recv/area/lake` (`field.rs:214-223`) is populated in every production world and consumed by nothing. water.md § consequence 1 + corrections #15 both say the water table is **pinned by the drainage network** — and S11's whole Q1 measurement was only meaningful once that pinning lattice was added (S11-results.md:59-69). So a deep-tier "depth to water table" proxy is `f(elevation above the local drainage line, cell permeability)` and needs **no new sim**. This is the cheapest real hydrology coupling available and it is already exported.
2. **A per-cell saturation/wetness plane in `DeepGrid`**, one `f32`, relaxed at deep-cell scale — the same shape as `precip`/`bio_weather`/`bio_resist`. It would need a *permeability per cell*, which needs the outcropping unit's material properties — which is `Litho::reference_material().props().permeability`, available today via `lithology.rs:296`.
3. **Retire the waterlogging proxy** (P11 consequence 4, already decided). `biotic.rs:645-647` computes `wet = moist + low_bonus(elevation) + area_bonus(drainage area)` — a three-term guess whose named heir is "the water table is at or near the surface here, read from the field". Note it *already* reads drainage area, so it is half-way to coupling 1.

### E3. Which B-vectors become material/form-differentiated once water exists

| vector | what water buys it |
|---|---|
| 12 `weather` | the chemical term. `lithology.rs:283-289` explicitly says in-place weathering is a **sum over agents** with one term today and that the dissolution term lands "without anything here being rewritten". Needs saturation to gate circulating meteoric water |
| — `Agent::Dissolution` (D4) | goes from dormant to live. Vadose/phreatic and therefore cave morphology fall out for free (water.md:151-157) |
| 9 `periglacial` | freeze–thaw currently scales by *permeability*, a constant. With saturation it becomes "water **actually present** in the pores × temperature crossing zero" — the real mechanism |
| 16 `wave` | the ratified heir: wave energy as fetch from the S11 body graph × the 0037 wind field (ROADMAP:1327). This is the reason the wave retune was STRUCK |
| 13 `diffuse` | pore pressure / saturation is the dominant control on real hillslope failure. Also where D9's cohesion belongs |
| 11 `transport` | distinguishing a losing (arid, infiltrating) from a gaining reach — the difference between a wadi and a river |
| D1 **cementation** | **this is the direct one.** Cementation is precipitation from circulating pore fluid. "Loose particles under pressure become a structure containing the particles" is compaction; the *cement* comes from water. Without it, D1 can only model mechanical compaction (grain rearrangement + porosity loss), which is honest but is the weaker half |
| 21 niche construction / P3 soil | the aquitard contrast (P11 consequence 3) IS the loose-vs-packed soil contrast. Form-from-provenance and hydrology are the same axis read from two ends: **form decides permeability, permeability decides where water sits, where water sits decides form** |

**The load-bearing observation for sequencing:** P11 consequence 3 says the loose-vs-packed permeability contrast is what makes an aquitard an aquitard, and S11 measured that a **sharp aquitard makes bound water MORE local, not less** (halo 4 cells at contrast 10). So form-from-provenance is not merely compatible with the hydrology design — it supplies hydrology's cheapest case and hydrology supplies form's missing driver. They should be designed knowing about each other, which is what the user said.

---

## CODE≠DOC — disagreements worth shouting about

**#1 — The deep sim types every recorded unit as a lithified rock, but every recorded unit is by construction loose regolith.**

`erosion.rs:836-841` states it plainly: *"in the two-plane model the strata record mirrors `H`, not `R` — `R` is basement everywhere."* And `H` is the **alluvium/regolith** plane (`grid.rs:304`), the loose mobile cover.

Yet `expose` (`erosion.rs:845`) resolves the top unit's tag to a `Litho` whose `reference_material()` is `MUDSTONE` / `SANDSTONE` / `CARBONACEOUS_MUDSTONE` / `PEAT` / `COAL` / `GRANITE` (`lithology.rs:283-292`) — lithified rocks — and takes its resistances from their property sheets. So the sim asks "how hard is this loose river sand to erode?" and answers with sandstone's smash resistance.

`lithology.rs:71-73` acknowledges it and defers: *"the split between loose and lithified material is a distinction `Litho` can grow a variant for; nothing here assumes a lithology is rock."* Nothing has grown that variant. **This is the single largest material-fidelity defect the audit found, and it sits upstream of everything:** it is why loose and lithified erode identically, why wind deflates a cemented bed like a dune, and why the collapse tier can only express rock.

**#2 — In-place weathering product is recorded as a transported deposit.**

`weather_cell` adds to `self.dh` (`erosion.rs:304`), and `record` tags all of `dh` by `tag_of(surf, precip, energy, sea)` (`erosion.rs:1309`) — where `energy` is *this iteration's stream transport capacity*. So regolith produced in place by frost shattering on a summit is recorded as, e.g., `Subaerial/Arid/Low` — indistinguishable from distal floodplain fines. earth-processes.md § 3 wants weathering "feeding the loose-material budget" with signatures like "rounded vs angular outcrops"; the record cannot carry the distinction.

**#3 — The record's own doc says it tags "the process, not the mineral", and the collapse tier then resolves it to exactly one mineral per tag.**

`recorder.rs:8-12`: *"at deep-time cell resolution the readable story is the **process**… not the mineral. The material-tier member selection still runs at collapse time under the context this record hands it."* But `deep_class` (`geology.rs:326-342`) is a total function from tag to class, and `dc:stratum/clastic-coarse` has exactly one registered member. So the promised context-driven member selection is, in the shipped content set, a lookup table. The world skins **91.4 % Mudstone** (ROADMAP:1813-1831). The doc is right about the architecture; the content set makes it a no-op.

**#4 — materials.md has described the compaction loop since 2026-07-18 as if it existed.**

materials.md:72-74: *"Compaction closes the deep-time loop: debris under overburden, over ledger time, migrates into a structure slot as sedimentary stone. Worldgen strata, gameplay middens, and geology are one process at different tick rates."* Written in the present tense in the design draft. Nothing implements it. earth-processes.md § 5 repeats it ("per-column burial-depth history from the epoch stack → P/T path → class transform"). This is the user's named vector, and it has been on paper for three days without a single line of code or a single field of state to hold it.

**#5 — P8's pore-packability rule is DECIDED, built, tested, and called by nothing.**

`packing.rs` documents itself as *"consulted by every transport-time depositing process (overflow packing, groundwater infiltration, future ore deposition)"*. There are no such processes. Not a defect — the rule was built ahead of its consumers, correctly — but the integrator should know it is a ready-made input, not work to be done.

**#6 — `DeepField.exhum`/`t_crust` are documented as "the metamorphic-grade axes the collapse tier WILL read" and are read by nothing** (`field.rs:224-232`, honestly self-flagged). Same for `chapters` (`field.rs:233-240`).

---

## F. Sequencing recommendation (RECOMMENDATIONS ONLY — user ratifies)

The ordering constraint that dominates everything: **ROADMAP:1846-1855 already states the fix order — substance first, then form from provenance.** Nothing below reorders that.

### F0 (prerequisite, already ROADMAP'd) — the soil substance gap
Register a soil material (`LOAM` exists and is orphaned) and a root material into geology classes. **Reason:** every form rule for soil operates on materials that do not exist. Zero design risk; content-set work. This is P3's precondition and it is already named.

### F1 — Settle the record's entry-species question BEFORE writing a form axis
Decide (data-model, integrator-owned per the process note in water.md:317-323) whether `DepUnit` grows fields or whether the record grows a second entry species for **transformations of prior units** (earth-processes.md:96-100). **Reason:** diagenesis, de-lithification, mineral replacement, dikes, and fault offsets are all the same species. Three ad hoc in-place mutators already exist (`promote_coal`, `overprint_top`, and the erosion `erode`/`deposit` merge logic). "Cheaper before than after" is the doc's own words, and the D1 vector is the fourth caller.

### F2 — Give `Litho` a loose/lithified distinction (CODE≠DOC #1)
The smallest change with the largest blast radius. `lithology.rs:71-73` already names it as a one-variant growth, and `Agent` is deliberately non-exhaustive-matched so every site must answer. **Reason:** until the deep sim can say "this bed is unconsolidated", no agent can behave differently on it, and *every* downstream form rule is guessing. Note this is a **terrain-shape-changing flip** of the same class as erodibility/biotic/tectonic/full-agents — it needs the flag discipline (`DeepOverrides`) and a user appearance call, not a silent merge.

### F3 — The form axis on the record, written by a diagenesis vector
With F1's shape settled and F2's distinction available: add form to the recorded unit and a finalize-or-periodic pass that promotes loose → packed → lithified from (burial depth, duration, bulk properties). **Model it on `promote_coal`** — same file, same shape, same thickness-preserving discipline, same finalize hook. Inputs available today: burial depth (derivable from `Σ` overlying thickness), duration (chapter, ±62.5 Myr), bulk properties (needs F2 to know what the bed actually is). Inputs NOT available: max-burial, geotherm, cement supply — **so scope the first pass to mechanical compaction only**, and state that cementation waits on water. Doing the water-free half first is honest, not a bandaid: mechanical compaction is a real, separate process.

### F4 — Wire form into expression: retire the two class-string tests
Replace `fill.rs:310` and `collapse.rs:1467` with a read of the recorded form. **Reason:** this is where the user sees it — organics stop being cubes, the top of every column becomes genuinely loose, subsurface soil generates already packed (ideas.md:281), and P3's structural-dirt-with-loose-above becomes emittable. Also the natural moment to emit `structure_len < capacity` (porous rock) for the first time, which is what makes `fits_in_pores` (F0's other orphan) reachable.

### F5 — Hydrology couples in, cheapest first
(a) Consume the already-exported `recv`/`area`/`lake` as the water-table pinning lattice — no new sim, and corrections #15 says an unpinned field measures nothing. (b) Derive per-cell permeability from the now-form-aware outcropping unit. (c) Retire S10's waterlogging proxy (P11 consequence 4, already decided). (d) *Then* cementation, dissolution/karst, and the wave-fetch heir become writable — all three are blocked on the same thing and none of them should be attempted before it.

### F6 — Deferred, explicitly
- **Grain continuum in the record** (D6) — owed since S9, but it multiplies the tag space and the eolian record already costs +92.76 MB undiagnosed (ROADMAP:1868). Measure the record-size bill before adding a merge-key axis. Note that adding *form* (F3) is itself a merge-key axis and carries the same risk — **budget for it in the same slice**.
- **Weathering facies / residuum tag** (D3) — a real fidelity gap, but it is a *tag* fix, cheaper after F1 settles the entry species.
- **Geotherm / P–T metamorphism** (D10) — wants `exhum`/`t_crust`, which are the separate ROADMAP'd "tectonic expression at the collapse tier" family.
- **Per-voxel provenance query** (D11) — explicitly needs its own design pass and its framing is PROPOSED, not ratified.

### The one dependency claim worth defending
**F2 before F3.** It is tempting to add a form field to `DepUnit` first and derive the lithology distinction from it. That inverts the causality: form is what the deep sim's *agents* need in order to behave differently, and agents read `Litho`, not `DepUnit` directly (`erosion.rs:1404,1511` and `lithology.rs:376`). If form only lands on the record, the sim keeps eroding loose sand like sandstone and the record's form field is decoration — the exact failure mode P1 was ratified to prevent ("fractions come only from the ledger", never cosmetic).
