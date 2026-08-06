# Field inventory — CLIMATE AND SURFACE

**Base commit:** `95db80a` (worktree tip == `main` at merge; `git merge main` reported
*Already up to date*).
**Method:** source reading only. **No cargo was run.** Every number below is either
(a) read off a struct definition and multiplied by the production cell count, or
(b) quoted from a dated measurement doc with its citation. Nothing here was measured
by this pass.

**Production grid used for every byte figure:** `w = 545`, `n = 297,025` cells,
`cell_m = 460`, `iterations = 200` — the shipped Medium world. `w`/`n` from
`docs/spikes/S10-results.md:141` and `crates/dc-worldgen/src/deeptime/inventory.rs:841`
(*"a production grid of 297,025 cells"*); the constants from
`crates/dc-worldgen/src/deeptime/field.rs:46,50,52`
(`DEEP_CELL_M = 460.0`, `DEEP_MAX_WIDTH = 550`, `DEEP_ITERATIONS = 200`).
So an `f32` plane = 1,188,100 B = **1.13 MiB**; an `f64` plane = 2,376,200 B = **2.27 MiB**.

**What the shipped world actually runs.** Read from `field.rs::production_config_base`
(`field.rs:276-381`), *not* `DeepConfig::default()`. That function sets `seed`, `cell_m`,
`iterations`, `record: true`, `biotic: true` (`:292`), `erodibility: true` (`:303`),
`tectonic_history: true` (`:321`), `full_agents: true` (`:342`),
`calibrated_rates: false` (`:378`), then `..DeepConfig::default()` (`:379`).
`remarch_interval` and `weather_inventory` are **not** overridden, so they fall through
to the defaults `20` and `false` (`grid.rs:570,607`).

| pass | in the shipped world? | why |
|---|---|---|
| `dc:deep/climate` | **YES**, every 20 epochs (10 firings of 200) | unconditional in `declared_passes` (`runner.rs:708`), `Schedule::every(cfg.remarch_interval)` |
| `dc:deep/frost` | **YES**, every epoch | gated on `full_agents`, which production flips ON (`runner.rs:774`, `field.rs:342`) |
| `dc:deep/biotic` | **YES**, every epoch | gated on `biotic`, ON (`runner.rs:971`, `field.rs:292`) |
| `dc:deep/weather` (height tier) | **YES**, every epoch | unconditional (`runner.rs:876`) |
| `dc:deep/weather_inventory` | **NO** | gated on `weather_inventory`, which production leaves at the `false` default (`runner.rs:996`, `grid.rs:607`). Reachable only via `DeepOverrides::weather_inventory` — the single caller is `dc-client/src/main.rs:159` (`--weather-inventory`). |

---

## The table

| | climate | frost | biotic | weather (height) | weather_inventory |
|---|---|---|---|---|---|
| **writes** | `grid.precip` `Vec<f32>` | `Erosion::frost` `Vec<f64>` | `grid.bio_weather`, `grid.bio_resist` `Vec<f32>`; `grid.h`; `grid.strata` | `grid.r`, `grid.h`, `Erosion::dh` (all `Vec<f64>`) | `DeepStepCtx::weather_ledgers: Vec<FactLedger>` |
| **reads** | `grid.r`+`grid.h` (via `surf_at`), row latitude, `sea_level` | `grid.r`+`grid.h`, latitude, `grid.strata` window | `grid.precip`, `grid.r`/`h`, `ero.area/recv/filled`, own `cells`, `parent_p`, `wet` | `grid.r`/`h`, `bio_weather`, `sus_flow`, `frost` | `grid.r`/`h`, `grid.bio_weather`, `Erosion::frost()`, own ledger |
| **element / bytes-per-cell (output)** | f32 / 4 | f64 / 8 | 2×f32 / 8 | — (mutates shared terrain) | `Fact` 8 B resident, `Fact<FracM>` 16 B accumulating |
| **output size @ n** | 1.13 MiB | 2.27 MiB | 2.27 MiB (planes) | 0 own | **9.57 MiB** resident (measured, S20) |
| **own persistent state @ n** | 0 (+1.13 MiB transient scratch) | 0 | **≈57.8 MiB** (204 B/cell, measured) | 0 | **≥13.60 MiB** of headers + facts, gen-time |
| **survives into `DeepField`?** | **no** | **no** | **no** (planes); yes indirectly as strata units | yes, folded into `surf`/`regolith` | **yes** — `DeepField::ledgers` |
| **calls a shared kernel?** | no | no | no | no | no |
| **kernel it would want** | 1-D directional prefix march + a 3-tap separable blur | pointwise map | pointwise map + a 3×3 max-gather | pointwise map | pointwise map (over a per-cell inventory) |

---

## 1 · Per-cell state — exactly which planes

### climate
Writes exactly one plane: `DeepGrid::precip: Vec<f32>` (`grid.rs:663`), assigned at
`climate.rs:95`. Reads `grid.surf_at(i)` = `r[i] + h[i]` (`climate.rs:65`,
`grid.rs:750`), the row latitude (`climate.rs:53`, `grid.rs:756`), and the scalar
`sea_level` handed in by the runner (`climate.rs:49`, `runner.rs:357`). It allocates a
scratch `raw: Vec<f32>` of `n` each march (`climate.rs:51`) and frees it on return.
4 B/cell output, 4 B/cell transient.

`climate::air_temp_c` (`climate.rs:39`) is part of the same module and is **not a
plane at all** — a free function `(lat_deg, surf) -> f32` with no storage anywhere.
Its callers evaluate it inline: `erosion/weathering.rs:477` (frost),
`erosion/agents.rs:93,161,191,307` (wind/wave), `erosion/record.rs:288` (deposition
tag), `biotic.rs:746,810,846`.

### frost
Writes exactly one plane: `Erosion::frost: Vec<f64>` (`erosion/mod.rs:265`), sized and
filled in `Erosion::periglacial` (`erosion/weathering.rs:463-465, 491-501`). Cleared to
empty when `full_agents` is off (`:454`) — emptiness *is* the identity `1.0`
(`frost_at`, `:117`). Reads `grid.r`/`grid.h`, `grid.lat_deg`, and each cell's
near-surface strata window through `SusTable::blend` (`:488`). 8 B/cell.

### biotic
Writes `grid.bio_weather: Vec<f32>` and `grid.bio_resist: Vec<f32>` (`grid.rs:671,675`;
allocated `biotic.rs:602-603`, written `biotic.rs:740-741`), **and** mutates
`grid.h` (organic deposit, `biotic.rs:752`) and `grid.strata`. Its own carried state is
`BioticSim` (`biotic.rs:532-577`): `cells: Vec<CellBiota>` (72 B/cell, measured —
`S10-results.md:170`), `prev_cover: Vec<[f32; ROSTER]>` (`ROSTER = 7` →
28 B/cell, `biotic.rs:187,539`), `out: Vec<Outcome>` (~104 B/cell, measured —
`S10-results.md:174`), `parent_p: Vec<f32>` (4 B/cell), `wet: Vec<f32>` (**empty under
the identity provider** — `biotic.rs:568-573`).

### weather (height tier)
Owns **no plane of its own**. It mutates `grid.r`, `grid.h` and the shared `Erosion::dh`
accumulator in place (`erosion/weathering.rs:559-598`, via
`weather_behavior::weather_one_cell`, `weather_behavior.rs:307-330`). Reads
`grid.bio_weather` (`:548`), `Erosion::sus_flow` (`:549`, written by `expose`),
`Erosion::frost` (`:550`). This is the pass whose entire output *is* someone else's
plane.

### weather_inventory
Writes `DeepStepCtx::weather_ledgers: Vec<FactLedger>` (`runner.rs:268`), one growable
per-cell ledger, appended by `weather_bedrock_epoch` (`weather_inventory.rs:290-330`).
The resident form is `LedgerField` (`inventory.rs:862-874`): a flat `Vec<Fact>` +
`Vec<SlotRun>` + `Vec<u32>` CSR offsets. `Fact` is **8 B** resident / 16 B accumulating
(`inventory.rs:429-431`); `SlotRun` is 8 B (`inventory.rs:619-624`). It reads
`grid.r`/`grid.h`, `grid.bio_weather`, `Erosion::frost()` and `cfg.weathering`/`h_star`
(`weather_inventory.rs:344-372`). It **never writes `r`/`h`** — the two-authorities
split, stated at `weather_inventory.rs:340-343` and `runner.rs:485-489`.

---

## 2 · Locality — where the line actually falls

### climate — the case to reason carefully
**The precipitation march is neither pointwise nor global. It is a per-row 1-D
recurrence.**

`march` (`climate.rs:49-98`) does, per row `gy`: pick a march direction from
`wind_dir(lat)` (`climate.rs:57-60`), then walk the row in that direction carrying a
scalar `moisture` and a scalar `prev_elev` (`:61-79`). Each land cell calls
`rainout(moisture, uplift, lat)` (`pregen/climate.rs:159-166`), which returns
`(precip, moisture_out)` — so `moisture` is **loop-carried along x**. Then a fixed
3-tap box blur across y (`climate.rs:83-96`).

Precisely: `precip(x, y)` depends on the surface elevation of every cell **upwind of
`x` on rows `y−1, y, y+1`**, back to the last cell that was at or below sea level
(where `moisture` is hard-reset to `1.0`, `climate.rs:68`). That reset is what bounds
it: the dependency does not reach the whole grid, it reaches back to the coast on that
row. Cost to evaluate one cell alone is O(w) in the worst case, not O(w²) and not O(1).

**And it carries no epoch history at all.** `march` overwrites `precip` completely from
the current `(surface, sea_level)` — there is no accumulator, no previous-precip term.
The runner's own comment makes the same argument to justify deleting the pre-loop seed
(`runner.rs:701-707`: *"a pure function of `(surface, sea level)`"*), and
`DeepStepCtx::dt`'s doc classifies it as a **relaxation toward an equilibrium set by
the current state**, not a rate (`runner.rs:238-242`) — which is why it ignores `dt`.

So the honest line is: **climate is memoryless in time and 1-D-nonlocal in space.** The
path dependence one worries about is entirely in its *input*: `surf_at` is the product
of 200 epochs of uplift/incision/creep/isostasy. Climate does not integrate history; it
reads a landscape that did. Latitude and lapse are pointwise; the moisture depletion is
the recurrence.

`air_temp_c` is **strictly pointwise**: `30 − 0.55·|lat| − 6.5·(surf/1000)`
(`climate.rs:39-44`). Zero neighbours, zero history, given the surface.

### frost
**Pointwise, with one caveat.** Per cell: `air_temp_c(lat, r+h)`, a triangular band
about 0 °C (`erosion/weathering.rs:474-486`), times a susceptibility blended from that
cell's own strata window (`:488`). No neighbour is touched; the doc comment claims
*"purely per-cell → byte-identical parallel"* (`:450-451`) and the code supports it
(`par_iter_mut().enumerate()`, `:491-495`). The caveat is that the strata window is a
column read of unbounded depth, not a scalar — the "small set of parameters" is small
in *area*, not in *bytes*.

### biotic
**Bounded-neighbour (radius 1), and loop-carried in time.** Processes 1, 3–6 are
per-cell; process 2 (dispersal) takes a `max` over the 8 neighbours of a **frozen**
previous-epoch cover snapshot (`biotic.rs:996-1017`, snapshot at `:679-681`). The
freeze is what makes the step order-independent and the parallel driver bit-identical.
Time is the real nonlocality: `CellBiota` carries `tsd`, `p_rock`, `soil`, `prev_surf`,
`prev_h` (`biotic.rs:471-495`), and the Walker & Syers depletion curve the module is
built around is *by construction* the integral of the whole run. You cannot evaluate
biotic state at one cell at epoch 200 without having run all 200 epochs there and in a
radius that grows with epoch count.

### weather (height)
**Strictly pointwise**, given four per-cell multipliers. `weather_one_cell`
(`weather_behavior.rs:307-330`) touches only `r[i]`, `h[i]`, `dh[i]` and five scalars.
The upstream multipliers are the nonlocal ones: `sus_flow` is a column blend, `frost` a
column blend, `bio_weather` a neighbour-coupled history.

### weather_inventory
**Pointwise per cell, over a per-cell column inventory.** `weather_epoch`
(`weather_inventory.rs:344-372`) is a plain `for` over cells with an independent
subaerial gate; inside, `weather_bedrock_epoch` rebuilds the working inventory from
`base + facts` and appends. It is accumulating in time (that is the whole point —
`journal/0094` made it a per-epoch process) but never reads a neighbour.

---

## 3 · State vs output size, in bytes at the production grid

| pass | own persistent state | field produced | ratio |
|---|---|---|---|
| climate | **0 B** carried across epochs; 1.13 MiB scratch inside one call (`climate.rs:51`) | 1.13 MiB (`precip`) | the output *is* the state |
| frost | **0 B** — recomputed from scratch every epoch (`erosion/weathering.rs:463-465`) | 2.27 MiB | 0 : 1 |
| biotic | **≈57.8 MiB** (204 B/cell × 297,025; 204 = 72 `CellBiota` + 28 prev-cover + ~104 `Outcome`, `S10-results.md:173-176`) + 1.13 MiB `parent_p` | 2.27 MiB planes; ~6.1 MiB of extra strata units survive (`S10-results.md:178`) | **≈25 : 1** |
| weather (height) | 0 B | 0 B of its own | n/a |
| weather_inventory | ≥13.60 MiB of per-cell `FactLedger` headers alone (48 B × 297,025, `inventory.rs:589-590`) + 16 B per accumulating fact | **9.57 MiB** resident (`S20-fact-ledger-residency-results.md:24,127`: 1,033,189 facts × 8 B + 72,006 rows × 8 B + 297,026 × 4 B) | >1.4 : 1 |

Two things stand out and neither is a defect claim, just a fact:

- **Biotic is the only pass in this group whose state dwarfs its published field by an
  order of magnitude.** A community vector, five nutrient pools and a successional clock
  produce two `f32` multipliers. That 25:1 is the honest price of a stateful ecology.
- **Climate and frost have literally no state.** They are recomputes. A storage question
  about them is a caching question, not a state question.

---

## 4 · Lifetime — and who reads the survivor

**`DeepField` does not carry `precip`, `bio_weather`, `bio_resist` or `frost`.** I read
the full struct definition (`field.rs:480-614`); its fields are `w, wp, cell_m, surf,
regolith, strata, ledgers, recv, area, lake, flux, exhum, t_crust, geotherm, head,
chapters`. None of the four planes is among them, and the distillation
(`build_field_cfg_cadence_geology`, `field.rs:665-748`) never copies them — the grid is
dropped at `:715-733`.

- **climate → `precip`: DIES with the grid.** In-sim it is heavily consumed:
  I grepped `grid\.precip` over `crates/` (19 hits total) and `grid\.precip|\.precip\[`
  over `crates/dc-worldgen/src/` (12 hits, **zero outside `deeptime/`**): `biotic.rs`
  693/750/886, `climate.rs` 48/95, `erosion/agents.rs` 94/113/192/311 (the eolian
  aridity gate), `erosion/record.rs:216` (the facies aridity tag), plus 2 comment-only
  hits (`head.rs:104`, `runner.rs:704`). Outside `src/` the only readers are examples
  (`biotic_spike.rs:422`, `appearance_tour_p11.rs:1296`).
  **⚠ The collapse tier has a *different* precipitation.** `collapse.rs::climate_at`
  (`collapse.rs:1335-1366`) bilinearly samples `v.precip` off the **pregen `CellGrid`**
  — the coarse plane `pregen/climate.rs::apply` wrote (`pregen/climate.rs:168-199`) —
  not the deep-time marched plane. So the world the player walks is textured by a
  precipitation field that never saw 200 epochs of orogeny. The two share `rainout` /
  `subsidence` / `zonal_wind` by re-export (`climate.rs:29`), so they are the same
  *model* at two resolutions on two topographies, which is the design intent stated at
  `climate.rs:13-19` — I am recording the consequence, not calling it a defect.

- **frost: DIES with the `Erosion` scratch.** Readers of the plane: `frost_at` inside
  the height-tier weather kernel (`erosion/weathering.rs:539,550`) and
  `runner.rs:496` handing `ctx.erosion.frost()` to the inventory pass. I grepped
  `self\.frost|erosion\.frost|ero\.frost|\.frost\(\)` over `crates/` — 12 hits, of which
  5 are the plane's own lifecycle in `weathering.rs`, 1 is `scratch_bytes` accounting
  (`erosion/mod.rs:625`), 1 is an unrelated `Agent::FrostIce` match arm
  (`lithology.rs:219`), and 2 are doc comments. **Zero test-only readers, zero
  survivors.**

- **biotic modifiers: DIE with the grid; the biotic *record* survives.** I grepped
  `bio_weather` (22 hits) and `bio_resist` (22 hits) over `crates/`. Live production
  readers of `bio_weather`: `erosion/weathering.rs:538,548` and
  `weather_inventory.rs:356`. Live production readers of `bio_resist`:
  `erosion/agents.rs:119-122`, `erosion/creep.rs:184,455,616`,
  `erosion/creep_kernel.rs:253,324`, `erosion/ledger.rs:246`. Test-only: 3 hits in
  `tests/biotic.rs:57,61,117`. Example-only: 1 comment in
  `examples/creep_operator_probe.rs:52`. What survives is the organic/charcoal/paleosol
  **units in `strata`** — 6.1 MiB (`S10-results.md:178`) — and those are read by the
  collapse tier.

- **weather (height): its output survives as `surf` and `regolith`.** Its whole effect
  is the `R -= q; H += q` transfer, and `regolith` is a first-class `DeepField` plane
  read by `collapse.rs::column` and `geology.rs::clastic_pass` (stated at
  `field.rs:489-503`). This is the one pass in the group whose product is unambiguously
  consumed by production.

- **weather_inventory: survives as `DeepField::ledgers` — and has real production
  consumers, which are currently reached with an empty record.** I grepped
  `ledgers|weathering_product_m|ledger_at_voxel` over `crates/dc-worldgen/src/` and
  `crates/dc-core/src/`: the non-plumbing production readers are
  `collapse.rs:1625` (`bundle.weathering_product_m()`) and `collapse.rs:2916`. Six
  probe/example readers exist (`flow_cost_probe`, `perdepth_weathering_cost_probe`,
  `pore_decorrelation_probe`, `release_spectrum_probe`, `s18_weathering_tour`,
  `s20_ledger_residency_probe`). **Because the flag is off in production, those two
  collapse call sites always read an empty `LedgerField` and get `0.0`.** This is the
  clean instance of the corpus's exported-plane-vs-in-sim-value distinction, inverted:
  here the *exported* plane has consumers and the *pass* does not run.

---

## 5 · Numerics — and what a kernel would have to do

**None of the five passes calls a shared solver kernel.** I grepped
`dc_core::field|FieldKernel` over `crates/` — 20 hits, and every non-`dc-core`
one is in `deeptime/erosion/creep*.rs` or `erosion/mod.rs`/`ledger.rs` doc comments.
`FieldKernel` (`dc-core/src/field.rs`) has exactly **one** consumer: hillslope creep
(`erosion/creep_kernel.rs:17,25,258`). It is a doubly-limited explicit diffusion gather
with a monotonicity bound; nothing in this group diffuses.

Per pass, what a kernel would have to do:

- **climate — a genuinely new primitive: a *directional 1-D prefix march*, plus a
  separable blur.** The operation is: for each row, in a per-row direction, carry a
  small state vector (here one scalar, `moisture`) through a pack-supplied
  `step(state, cell_inputs) -> (output, state')`. That is a scan/recurrence, not a
  relaxation and not a stencil — `FieldKernel`'s gather cannot express it, because the
  gather is symmetric and simultaneous and this is sequential and directed. The blur at
  `climate.rs:83-96` is a *second*, unrelated operation: a separable box filter, which
  is trivially a stencil and could share a kernel with any other smoother. **Two
  operations in one 98-line file.** I would not merge them.
- **frost — NO KERNEL. It is a pointwise map**, and that is the whole answer:
  `f(lat, surf, column_window) -> multiplier`, evaluated independently per cell. Wrapping
  it in a solver would add a driver and buy nothing. What it *does* want from the engine
  is (a) a parallel per-cell driver with a bit-identity promise — which it currently
  rolls itself with `par_iter_mut` (`erosion/weathering.rs:491`) exactly as four other
  passes do — and (b) a column-window read primitive (`SusTable::blend`).
- **biotic — NO FIELD KERNEL, but it is not pointwise either.** Its one nonlocal step is
  a **3×3 max-gather over a frozen plane** (`biotic.rs:996-1017`). If anything were
  extracted it would be that: *"gather a reduction over a bounded neighbourhood of a
  frozen snapshot"*, parameterised by the reduction (`max` here; a sum or a weighted
  kernel elsewhere). Everything else — Liebig `min`, competition normalisation, nutrient
  cycling — is per-cell arithmetic that belongs to the pack, not the engine.
- **weather (height) — NO KERNEL, and it already has the right shape.**
  `weather_one_cell` is a pure per-cell function behind a `WeatherCtx` read capability
  and a pass-owned `WeatherApply` write handle (`weather_behavior.rs:88-330`). This is
  the north-star behavior shape de-risked (S16) and it holds: the pass declares its axes
  as data (`WeatherAxis`, `weather_behavior.rs:74-89`), the behavior is pure, the apply
  is owned. The strain the module itself documents (`weather_behavior.rs:26-38`) is not
  numerical — it is that the height tier has **no single outcropping material**, so
  `weatherability` degenerates to a per-cell blend.
- **weather_inventory — NO FIELD KERNEL.** Pointwise per cell. What it needs from the
  engine is not a solver but *storage*: an append-and-merge fact log with a stable key
  (`FactLedger::append_merged`, the CSR compaction at
  `inventory.rs:876-910`, the single f64→f32 narrowing at persist). Its "numerics" are
  `Σ_a driver_a × susceptibility_a × cover_taper` — three multiplies and an
  `exp` (`weather_inventory.rs:150-206`).

### The explicit answer: how many kernels does this group need?

**One new field kernel, shared by nobody in this group; three passes need none; one
wants a gather primitive that is not a solver.**

1. **A directional prefix-march ("scan") primitive — climate alone.** Nothing else in
   the group, and nothing in `FieldKernel`, can express a loop-carried directed
   traversal. It is the one place where the answer to *"can this be a shared solver
   call"* is yes-but-it-does-not-exist-yet. Whether it is worth building for a single
   pass firing 10 times per world is a real question I cannot answer from source alone.
2. **A separable blur / small-stencil smoother — climate's second half.** Plausibly
   shared corpus-wide (any coarse plane that wants de-streaking), but nothing in *this*
   group is a second customer.
3. **frost, weather, weather_inventory need no kernel.** They are pointwise maps. The
   right engine offering for them is a *driver* (parallel per-cell with a bit-identity
   guarantee) and a *ctx* (what a per-cell behavior may read), not a solver. The engine
   already has the ctx shape — `weather_behavior::WeatherCtx` — and every pass rolls its
   own driver.
4. **biotic wants a bounded-neighbourhood gather over a frozen plane** — one step of it,
   not the pass. Different primitive from `FieldKernel::step`: no potential, no flux, no
   conservation, no stability bound; just *reduce over a radius on a snapshot*.

The load-bearing observation: **four of my five passes are pointwise or
bounded-gather, and the shared kernel that exists is a conservative diffusion solver
none of them can use.** "Fields" is not one category here. There are at least three
shapes — *relaxation/diffusion* (creep, head, geotherm — other groups), *directed scan*
(climate), and *pointwise map over a per-cell inventory* (frost, weather,
weather_inventory) — and the third is the largest population.

---

## 6 · Storage shape each output wants

- **climate → `precip`: a rule plus parameters, re-evaluated.** It is already
  recomputed from scratch every 20 epochs and thrown away at the end. Nothing keeps it.
  Given the topography and `sea_level`, it is reproducible exactly — the runner argues
  this itself when it deleted the pre-loop seed (`runner.rs:701-707`). The only reason
  the plane exists is that five in-epoch consumers want random access to it within the
  epoch, and re-marching per query would be O(w) each. **One value per cell as a
  20-epoch cache of a rule** is the honest description.
- **climate → `air_temp_c`: nothing kept, and that is already true.** It is a free
  function evaluated at 9 call sites. This is the cleanest example in the group of a
  field that is a *rule*, and the corpus already treats it as one.
- **frost: one value per cell, kept only for the epoch.** Same shape as precip but
  finer-cadence. Note it is stored as `f64` for a quantity in `[1.0, 1.0 + gain]` — an
  `f32` would halve it to 1.13 MiB, but it multiplies into `f64` weathering arithmetic
  (`erosion/weathering.rs:539,566-576`) so the narrowing would move the goldens. Flagging
  the width, not proposing the change.
- **biotic modifiers: one value per cell, epoch-lived** (they are consumed one epoch
  later by design, `runner.rs:165-168`). The biotic *state* (`CellBiota`) is a different
  question and wants exactly what it has: a per-cell struct, transient.
- **weather (height): nothing of its own.** Its product is `H`, and `H` is already a
  first-class kept plane. This pass is the argument that "what a pass publishes" and
  "what a pass stores" are different questions — its published output is a *delta on
  someone else's authority*.
- **weather_inventory: a value per recorded layer in a column, sparse.** This is the one
  output in the group that genuinely wants the per-layer shape, and it already has it:
  facts keyed by slot, CSR-indexed by cell, and 75.8 % of cells carry none
  (`field.rs:519`). The measured record is 1,033,189 facts over 72,006 slot-rows across
  297,025 cells (`S20-...:127`) — i.e. the sparse shape is load-bearing, not decorative.

---

## 7 · Refinement relevance — **my reading, marked as such**

**This section is my analysis, not a corpus claim.**

The brief names vegetation cover, weathering intensity and frost as plausible reads for
a landform operator near the player. Checking rather than assuming:

- **All three are structurally unavailable to refinement today.** Vegetation cover
  (`CellBiota::cover`), the biotic multipliers, and the frost plane are all dropped with
  the grid (§ 4). A refinement operator holds a `DeepField`; none of them is in it. So
  the question is not *"would an operator want this"* but *"would we pay to keep it"*.
- **Frost is the cheapest to make available and the least worth keeping.** It is a pure
  function of `(lat, surf, column window)` at the *final* state — an operator can
  re-evaluate `air_temp_c` and the same triangular band from the `DeepField` planes it
  already holds. **Keeping the frost plane would be storing a summary of a rule** — the
  ARCHITECTURE.md § *a summary is not an authority* shape. My reading: do not keep it,
  keep the rule. (Caveat: the *final-epoch* frost is not the *run-integrated* frost, and
  a periglacial landform is about the integral. If an operator wants "how much
  freeze–thaw did this place see", that is a recorded fact and belongs in the ledger,
  not in a resurrected plane.)
- **Vegetation cover is the one I would actually argue for**, and not as a plane. A
  landform operator asking "is this slope bare or forested" is asking a question the
  strata record already half-answers via `Biofacies` on the surviving units
  (`recorder.rs:386`). The final-epoch cover vector is 28 B/cell = 7.9 MiB if kept
  whole; the useful reduction (total cover, or the dominant species) is 1–4 B/cell.
  Unverified whether any drafted operator wants it — `docs/design/refinement.md` § 6–7
  mentions *"vegetation roughness"* once, at `:277`, as an example decorator, and names
  no field.
- **Weathering intensity is the one that is already right.** `DeepField::ledgers` is
  exactly "how much weathering happened here, by which agent, in which chapter",
  per-layer and sparse — and `docs/design/refinement.md:322` explicitly names *"the
  column inventory (the channel incises the regolith the weathering passes actually
  piled)"* as a channel-operator input. **The gap is that the pass producing it does not
  run in the shipped world.** That is the sharpest finding in this audit: refinement's
  named weathering input exists, is designed, has production consumers wired, and is
  empty because a flag defaults false.
- **Precipitation:** an operator near the player would be served by the *pregen* precip
  the collapse tier already samples (`collapse.rs:1335-1366`), not by the deep plane.
  I see no reason to keep the deep one.

---

## Unresolved / things I could not check

1. **`dc:deep/biotic` declares a read it does not perform.** All three biotic read-sets
   include `Frosted` (`runner.rs:595-597`), and `DeepAxis::Frosted`'s doc says
   *"`weather` + `biotic` read"* (`runner.rs:124-126`). I grepped `frost` over
   `crates/dc-worldgen/src/deeptime/biotic.rs`: **2 hits, both doc-comment prose**
   (`:565`, `:842`); there is no code read of `Erosion::frost` anywhere in that file, and
   `biotic_pass` (`runner.rs:472-477`) passes `&ctx.erosion` but `BioticSim::step`
   (`biotic.rs:671-`) never calls `.frost()`. Worse, `BIO_READS_TEC` and `BIO_READS_LEG`
   declare `Frosted` on rosters where the `frost` pass **is not present at all**
   (`runner.rs:774`) — while the runner's own comment at `:598-603` explains that
   `Frosted` appears "only in the agents slice" for the *inventory* pass' read-sets. The
   biotic sets did not get the same treatment. This over-declares a dependency and
   over-constrains the topo-sort. **I did not verify whether it changes the emitted
   order**, because I ran no cargo. Flagging, not fixing — it is outside my write-set.
2. **`sea_level` is an undeclared input.** `climate::march` takes it (`climate.rs:49`)
   and it varies per epoch (`grid.rs::sea_level_at`, `:539`), but it is not a `DeepAxis`
   — it arrives through `DeepStepCtx`. Several passes gate on it. Whether world-level
   scalars *should* be declarable is a real question for "what a pass can declare about
   what it publishes"; I have no ruling to cite either way.
3. **Partition check.** My five passes are all genuinely field/surface passes and I found
   nothing in my list that belongs elsewhere. One boundary note: **`dc:deep/expose`**
   (`erosion/weathering.rs:156-425`) lives in the same file as the height-tier weathering
   and writes two per-cell susceptibility planes (`sus_flow`, `sus_creep`, plus `litho`
   and the CSR `shares`) — it is a field pass by any definition, it is the single largest
   per-cell computation in `weathering.rs`, and it was not assigned to me. If no other
   group has it, someone should.
4. **The biotic byte figures are quoted from `S10-results.md`, which carries a
   ⚠ SUPERSEDED-IN-PART banner** (`S10-results.md:3-31`). The banner falsifies the
   **cost/time** table only (25.19 s scalar vs 13.79 s production); the *memory*
   itemisation at `:168-180` is not touched by it. I am relying on that scoping.
5. **I did not verify the `f32`/`f64` plane byte figures against a running probe.** They
   are `size_of` × 297,025, computed here.
