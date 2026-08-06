# Field inventory — SOLID EARTH group

**Base commit:** `95db80a` (`north-star: strike the retired trust model in the two clocks…`);
worktree merged main, tip matched at start of work.

**Method:** code reading only. **No cargo was run.** Every number below is derived from a cited
`file:line` plus the production config constants; the derivations are shown so they can be
re-checked without a build. Where I am guessing I say **unverified**.

**Production grid, used for every byte figure below.** `Extent::Medium` ⇒ pregen `wp = 17`
(`pregen/mod.rs:47-49`); `extent_m = 17 × CELL_VOXELS(16384) × 0.9 = 250 675.2 m`
(`field.rs:277-279`, `pregen/mod.rs:41-43`); `cell_m = max(250675.2/550, 460) = 460.0`
(`field.rs:46,50,279-280`); `w = round(250675.2/460) = 545` (`grid.rs:804-805`); **n = w² =
297 025 cells**, so **one f64 plane = 2 376 200 B ≈ 2.27 MiB**, one u8 plane = 297 025 B.
`chapters = 8`, `plate_scale_km = 65.0`, `flex_wavelength_km = 50.0`, `iso_rate = 0.5`,
`orogen_width_km = 25.0`, `advection_plate_widths = 1.0`, `thickening_scale = 80.0`
(`grid.rs:596-606`, reached from `production_config_base`'s `..DeepConfig::default()`,
`field.rs:381`). `tectonic_history: true` in production (`field.rs:321`) — so every plane in
this group is live, not empty.

---

## The table

| pass | reads | writes | per-cell bytes written | locality | own state at prod grid | survives into `DeepField`? | kernel today | storage shape it wants |
|---|---|---|---|---|---|---|---|---|
| `dc:deep/tectonics` | nothing per-cell; the chapter table + cfg | `Forcing` (`Erosion::forcing`, f64) | 8 B/cell | **point-evaluable** — `forcing_at` is a closed form in `(plates, cfg, x, y)` | **21.39 MB** of precomputed chapter planes + **5.6 KB** chapter table | forcing plane: **no**; chapter table: **yes**, read by 0 production sites | none — bespoke inline analytic | **rule + parameters** (a ~5.6 KB table), not planes |
| `dc:deep/forcing` (tectonic arm) | `Forcing` | `CrustThick` (`t_crust`, f64) | 8 B/cell | pure per-cell `t += f·dt` | none of its own | `t_crust` **yes**; the *exported* copy has 0 non-test consumers | none (trivial AXPY) | one value per cell |
| `dc:deep/expose` | `Recorded` (prev) | `litho` u8, `sus_flow` f64, `sus_creep` f64 (+ `masks` u64 on the member path) | 17 B/cell (25 B with masks) | **per-cell**, but each cell reads a *ragged* record window | ~5.05 MB (+2.38 MB masks) | **no** — dies with `Erosion` | none — a table blend, not a solve | **nothing kept**; re-derivable from the record |
| `dc:deep/diffuse` (creep) | `surf = r+h`, `bio_resist`, `sus_creep`, `h` | `h` (f64) | 8 B/cell mutated | **global** — a stencil relaxation; one cell's value needs its neighbourhood, iterated | `netdiff`+`scale`+`surf`+`creep_flux` ≈ 11.9 MB scratch | `regolith` (= `grid.h`) **yes**, and it *is* read by collapse | **`dc_core::field::FieldKernel`** — the one extracted primitive | one value per cell (it *is* the state) |
| `dc:deep/isostasy` | `Diffused`, `CrustThick`, `crust_kind`, `h` | `r` (f64), `exhum` (f64), `t_crust` (f64) | 16 B/cell written + 8 B/cell decremented | **global** — a ±109-cell box mean; one cell needs a 219-cell-wide window | 0 persistent; **9.5 MB allocated and freed per epoch** | `exhum` **yes** (0 production readers), `r` folded into `surf` **yes** (read by collapse) | none — `box_smooth` rolled inline | one value per cell for `r`; `exhum` wants **nothing kept** until its expresser exists |
| `dc:deep/geotherm` | `t_crust`, `crust_kind`, chapter plates | `geotherm` (f64) | 8 B/cell | **point-evaluable given `t_crust`** — closed form; but `t_crust` is a whole-history integral | none of its own | **yes**; in-sim value read by coal rank, exported plane read by 0 production sites | none — a `match` + clamp per cell | **rule + parameters** over `(t_crust, crust_kind, plates)` |

---

## Per pass

### 1. `tectonics` — the analytic boundary forcing and the chapter table

**1. Per-cell state.** Reads no plane. Writes `Erosion::forcing: Vec<f64>` via
`set_tectonic` (`runner.rs:362-366`, field at `erosion/mod.rs:283`), 8 B/cell =
**2 376 200 B**. The per-epoch body is `TectonicSchedule::blend_into` (`mod.rs:405-424`), which
lerps two precomputed chapter planes into `ctx.blended` (another f64 plane, `runner.rs:222`) —
so the epoch touches **two** f64 planes: `blended` (scratch) and `forcing` (the copy erosion
reads).

**2. Locality.** **Single-point evaluable, and unusually purely so.**
`tectonics::forcing_at(plates, cfg, v_ref, x, y)` (`tectonics.rs:246-284`) is a closed-form
expression: a `k = 3` nearest-seed scan (`nearest_k`, `:162-173`), a convergence projection
(`:177-183`), a signed bisector distance (`:188-194`), a classification (`:198-216`) and a
Gaussian bump (`:280-281`). Its own doc states the property — *"No grid term anywhere in the
expression"* (`:244-245`) — and the code bears it out: `x`/`y` enter only as continuous
kilometres. The plane the runner uses is built by scanning `forcing_at` over cell centres in
`TectonicSchedule::new` (`mod.rs:466-479`) — that scan is a **materialisation of a point
function, not a solve.**

**3. State vs output size — this is the measured case.**

- **Chapter table** (`TectonicSchedule::table`, `mod.rs:437`; `chapter_table`,
  `tectonics.rs:152-157`): `K+1 = 9` chapters × `plate_count(250.6752, 65)` plates.
  `plate_count = round((250.6752/65)²) = round(14.873) = 15` (`tectonics.rs:92-95`). `Plate` is
  4 × f64 + `bool` (`tectonics.rs:80-86`) ⇒ 33 B, aligned to 8 ⇒ **40 B**. So
  **9 × 15 × 40 = 5 400 B**, plus 9 inner `Vec` headers (24 B) + 1 outer ⇒ **≈ 5.6 KB**.
  (This confirms the design doc's *"~5 KB"* at `docs/design/tectonics.md:652` — the arithmetic
  lands almost exactly.)
- **Derived forcing planes** (`TectonicSchedule::planes`, `mod.rs:439`, filled at
  `mod.rs:467-479`): **one full f64 plane per chapter**, 9 × 2 376 200 = **21 385 800 B ≈
  20.39 MiB**, held for the whole run.

**Ratio: 3 960 ×.** The pass keeps ~4 000 times more derived plane than authoritative state, and
the derived plane is a pure function of that state plus six config scalars. This is the sharpest
single datum in my group: **tectonics is a `rule + parameters` pass wearing 20 MiB of per-cell
storage**, paid once per chapter to avoid re-evaluating a ~40-flop expression per cell per
epoch. Whether that trade is right is a perf question (200 epochs × 297 025 cells = 59.4 M
evaluations avoided — **unverified** whether that beats the 20 MiB of cache pressure; I did not
measure it).

**4. Lifetime.** The forcing planes and `blended` die with the run — they are not on
`DeepField`. The **chapter table survives**: `DeepField::chapters: Vec<Vec<Plate>>`
(`field.rs:613`), cloned at `field.rs:694`. **Who reads the surviving copy:** I grepped
`\bPlate\b` over `crates/` (all `.rs`) — **28 hits outside `deeptime/tectonics.rs`**. Those
touching `DeepField.chapters` are `tests/artifact_tripwires.rs` (`:76,:155,:164,:651` — the
`GOLDEN_CHAPTERS` fingerprint), `tests/tectonic_history.rs` (`:13,:223,:230`) and
`examples/tectonic_spike.rs` (`:16,:209,:216,:418`). The remainder are the *pregen* `Plate` (a
different, private type in `pregen/tectonics.rs`), the salt-domain enums, and the producing
module. **Production readers of the exported chapter table: zero.** `field.rs:606-612` already
says so in prose (*"today the table is exported and read by nothing"*); the grep agrees.

**5. Numerics / north-star shape.** No shared kernel, and there is nothing to share — it is not
a solve. If it were written against a primitive, that primitive would be a **scatter-with-kernel
point evaluation**: *given a small world-level table of sites with attributes and a per-site
compact kernel, evaluate the superposition at an arbitrary continuous `(x, y)`*. A **different
family** from `FieldKernel` — no stencil, no neighbours, no stability bound, no grid. It shares
nothing with diffusion but the word "field".

**6. Storage shape.** **A rule plus parameters, re-evaluated rather than stored.** The authority
is 5.6 KB; the 20.39 MiB is a cache. What it needs from the engine is a way to declare *"my
output is a pure function of this small table and these scalars, at continuous coordinates"* —
and then let the engine decide whether to materialise it, materialise it coarser, or evaluate it
on demand at refinement resolution.

**7. Refinement relevance (MY READING, not a finding).** High — and it is the only field in my
group that is *architecturally* refinement-ready. Because `forcing_at` has no grid term, a
refinement operator near the player could evaluate it at voxel pitch and get a genuinely finer
answer, not an interpolation. But see the § 8.2 verdict: what that gives you is the *forcing*,
which is several transforms away from anything visible.

---

### 2. `forcing` (tectonic arm) — `t_crust`

**1.** Reads `Erosion::forcing`; writes `grid.t_crust` (`uplift.rs:22-37`), 8 B/cell =
**2 376 200 B**. Seeded by `tectonics::seed_columns` (`tectonics.rs:365-414`, called at
`grid.rs:853`), which also writes `crust_kind: Vec<u8>` (297 025 B).

**2.** The *step* is purely per-cell (`*t = (*t + *f * dt).max(1000.0)`). But **the value** at a
cell is the whole run's history: seed + Σ(forcing × dt) over 200 epochs − Σ(exhumation)
(`uplift.rs:51-77`). Not point-evaluable after the fact; a time integral coupled to erosion.

**3.** No state of its own beyond the plane it writes. `r_snap` (`erosion/mod.rs:284`) is the
exhumation snapshot, another 2 376 200 B.

**4.** `t_crust` survives (`field.rs:614`). The in-sim **value** is read by two passes:
`isostasy` (`uplift.rs:89`) and `geotherm` (`geotherm.rs:163`). The **exported plane**: I
grepped `\.exhum|\.t_crust|\.geotherm` outside `crates/dc-worldgen/src/deeptime/` — hits fall in
`examples/` (`coal_walk_tour` 2, `denudation_probe` 3, `flow_cost_probe` 6, `tectonic_spike` 3,
`walk_tour_0115` 3) and `tests/` (`artifact_tripwires` 3, `deep_config_plumbing` 8,
`flux_record` 3, `geotherm` 6, `head_field` 3, `providers_common` 3, `tectonic_history` 11).
**Zero collapse-tier or client consumers**, for all three planes. `field.rs:557-577` states the
same in-sim-value-vs-exported-plane distinction and states it correctly.

**5–6.** No kernel; none wanted. One value per cell, genuinely — it is a conserved stock.

**7. (my reading)** Low direct relevance; a refinement operator wants the elevation and the
record, not the crustal thickness. It matters as an input to geotherm/metamorphism only.

---

### 3. `expose` — the outcropping-lithology susceptibility planes

**Partition note: I think this one is mis-filed in my group.** `expose` is not a field solve —
it is a **record query cached into planes**. It reads the near-surface window of `grid.strata`
and blends a per-agent rate table. It belongs with whoever owns the strata record and the
`outcrop_shares` provider seam. I audited it anyway.

**1. Per-cell state.** Reads `grid.strata[i].units` (ragged). Writes `Erosion::litho: Vec<u8>`
(297 025 B), `sus_flow: Vec<f64>` and `sus_creep: Vec<f64>` (2 376 200 B each) —
`weathering.rs:156-243`, fields at `erosion/mod.rs:257-259`. On the member-grade path
(`expose_member`, `weathering.rs:253+`) it additionally writes `masks: Vec<u64>` (2 376 200 B,
`erosion/mod.rs:353`) and scatters into the `shares` CSR plane. **≈ 5.05 MB, or ≈ 7.4 MB with
masks.**

**2. Locality.** Per-cell *in the grid*, but **not point-evaluable** — the value at a cell is a
thickness-weighted walk down that cell's recorded unit stack (`lithology::exposed_shares`,
`lithology.rs:629`). It needs the column's whole depositional history, which is itself 200
epochs of every other pass. Cheap and parallel; not analytic.

**3. State vs output.** The pass's own persistent state is a **6-entry class table**, or a
`Vec<f64>` of `axis.len()` entries on the member path, rebuilt per epoch (`SusTable::build`,
`weathering.rs:56-72`). On the order of **48 B–512 B against 5.05 MB** — another
10 000×-class ratio, but here it is honest: the per-cell variation comes from the *record*, not
from the table, so the output genuinely is per-cell.

**4. Lifetime.** **Nothing survives.** These planes live on `Erosion`, dropped when the run
ends; `DeepField` has no `litho`/`sus_*` field (`field.rs:480-614`). I grepped
`sus_flow|sus_creep|\.litho\b` over `crates/` outside `src/deeptime/erosion/` — **1 hit, and it
is a doc comment** (`lithology.rs:810`). Readers inside erosion: `transport.rs:268`
(`sus_flow`); `creep.rs:185,233,456,617`, `creep_kernel.rs:254,311,325`, `ledger.rs:246`
(`sus_creep`).

> **Found: `Erosion::exposed()` has zero call sites.** Defined at `weathering.rs:125` as *"read
> by the measurement probe to attribute landform statistics to rock type"*. I grepped the fixed
> string `exposed()` over `crates/` (all `.rs`): **2 hits, both inside comments**
> (`weathering.rs:205`, `lithology.rs:810`). A grep for `\.exposed(` returns **0**. So the
> accessor's stated consumer does not exist, and `litho` is written every epoch for it. Reporting,
> not filing — `spines.md` § 3 is outside my write-set.

**5. Numerics.** No kernel and none applicable. If it wanted a primitive, the primitive is a
**ragged-column reduction**: *walk the top-`d` metres of a per-cell variable-length record,
accumulate weights per key, blend against a key-keyed table*. That is a **record/gather**
primitive, not a field solver — and `weathering.rs:36-38` already observes that three other
passes (frost, wind, wave) do the same walk, which is a real argument for having it.

**6. Storage shape.** **Nothing kept.** A per-epoch cache of a query over already-persisted
data. It is a plane only because it would otherwise be recomputed several times per epoch.

**7. (my reading)** Refinement wants the *query*, not the plane — "what outcrops here" at
sub-cell resolution is exactly what the `outcrop_shares` seam and its structural-deformation
heir are for. The susceptibility planes are rate-modifiers for a sim that will not be running
near the player.

---

### 4. `diffuse` / hillslope creep — the one converted pass

**1. Per-cell state.** Reads `self.surf` (the frozen potential `r+h`, built at
`creep_kernel.rs:246`), `grid.h` (the conserved state), `grid.bio_resist` (f32) and
`self.sus_creep` (f64) as the coefficient field's two factors (`creep_kernel.rs:252-255`;
`eff_diff` at `:70-78`). Writes `grid.h` via `netdiff`. Scratch: `scale`, `netdiff`, `surf`
(f64, 2 376 200 B each), `creep_flux` = two f64 edge planes (`dc-core/src/field.rs:149-152`,
4 752 400 B), optional `netdiff_acc` only when sub-cycling *and* carrying identity
(`creep.rs:204-211`). **≈ 11.9 MB of scratch.**

**2. Locality.** **Global.** One explicit step is a 4-neighbour gather
(`dc-core/src/field.rs:324-403`), and the epoch's answer is that step iterated
`plan.substeps()` times, over 200 epochs, on a potential every other pass rewrites. No closed
form. This is the one pass in my group that is a field solve in the textbook sense.

**3. State vs output.** Persistent state *is* the state it evolves: `grid.h`, 2 376 200 B. Its
own bookkeeping is three scalars (`creep_substeps` u32, `creep_peak_coeff` f64, `creep_faces`)
plus the scratch above. State ≈ output, which is what a diffusion pass should look like.

**4. Lifetime.** `grid.h` becomes `DeepField::regolith` (`field.rs:723`, *"carried
(journal/0053) rather than summed away"*) and **is read by production code** —
`geology.rs:175` names `DeepField::regolith_at_voxel` as the loose-cover source for the collapse
tier. The only plane in my group with a real production consumer of the *exported* copy.

**5. Numerics.** **Declares against `dc_core::field::FieldKernel`** — `CREEP_KERNEL` at
`creep_kernel.rs:25`, plan at `creep.rs:188/194`, step at `creep_kernel.rs:256-270`. The pass
supplies `CoeffField` (`creep_kernel.rs:31-41`) and owns flux consumption; the kernel owns the
stencil, the monotonicity bound (`MONOTONE_MAX_EDGE_COEFF = 0.125`, `dc-core/src/field.rs:109`)
and the sub-cycle derivation. The ownership rule is stated at `dc-core/src/field.rs:18-30`. This
is the reference shape; nothing to add.

**6. Storage shape.** One value per cell — it *is* the state.

**7. (my reading)** High. Regolith depth near the player is directly visible (soil vs bare rock)
and the collapse tier already samples it. But a refinement operator almost certainly wants to
*re-solve locally* rather than interpolate, which is exactly what a boundary-value formulation
would enable — the audit's E4-2 implicit scheme is the obvious vehicle. **Unverified** whether
local re-solve is tractable at refinement scale.

---

### 5. `isostasy` — Airy compensation of the smoothed load

**1. Per-cell state.** Reads `grid.t_crust`, `grid.h`, `grid.crust_kind`, `grid.r`. Writes
`grid.r` (f64) and, via `track_exhumation` in the same pass body (`runner.rs:429-432`),
`grid.exhum` (f64, `+=`) and `grid.t_crust` (f64, `-=`). `uplift.rs:86-104` and `:51-77`.
**Written: `r` 2 376 200 B; `exhum` 2 376 200 B; `t_crust` decremented in place.**

**2. Locality.** **Global, and by a very wide window.**
`flex_radius_cells(50.0, 460.0) = round(50000/460) = 109` cells (`isostasy.rs:113-115`), so
`box_smooth` (`isostasy.rs:68-108`) averages over a **219 × 219 cell window ≈ 100.7 km across**.
One cell's equilibrium elevation depends on ~48 000 neighbours' crustal thickness. The step
after smoothing is per-cell (`e_eq`, then `r += rate·(e_eq − surf)` at `iso_rate = 0.5`), so the
pass is *smooth-then-relax*, and only the relax is local.

**3. State vs output.** **Zero persistent state**, worth saying plainly: the pass holds nothing
between epochs. It does **allocate ~9.5 MB per epoch and free it**: `box_smooth` is called twice
(`uplift.rs:89-90`) and each call allocates `tmp` (n f64), `out` (n f64) and `pre` buffers
(`isostasy.rs:72,77,89-91`) — 2 × (2 × 2 376 200) ≈ 9.5 MB, × 200 epochs ≈ **1.9 GB of transient
allocation over a run**. Gen time is not a constraint (CLAUDE.md), so this is a note, not a
defect — but it is exactly the shape a kernel with caller-owned scratch removes for free.

**4. Lifetime.** `grid.r` folds into `DeepField::surf` (`field.rs:670-676`) which drives collapse
macro-terrain — **read by production**. `exhum` survives as `DeepField::exhum` (`field.rs:576`);
its own doc (`field.rs:557-577`) says the exported plane has no collapse-tier consumer, and my
grep (same command as pass 2) confirms it: hits only in `examples/` and `tests/`. **Zero
production readers of the exported `exhum`.**

**5. Numerics — and the question you asked directly.**

> **Is isostasy's smoothed-load step the same kernel as creep's diffusion, or do they only
> rhyme?**

**They only rhyme.** Four separable differences:

- **`box_smooth` is a low-pass filter; `FieldKernel::step` is a conservation law.** The kernel's
  whole contract is antisymmetric per-edge fluxes so that *"every unit that leaves one cell
  arrives in exactly one other by construction"* (`dc-core/src/field.rs:32-41`). `box_smooth`
  moves nothing and conserves nothing — it computes a mean and discards the input. No donor, no
  inventory, no obstacle.
- **The stability bound is meaningless for it.** The kernel exists for
  `MONOTONE_MAX_EDGE_COEFF` and the sub-cycle derived from it (`dc-core/src/field.rs:76-102`,
  `:268-298`). A box mean is unconditionally stable at any radius; nothing to sub-cycle, no von
  Neumann analysis to hide.
- **The radius is a physical constant, not a rate.** 50 km is a flexural wavelength
  (`isostasy.rs:110-115`); creep's knob is a diffusivity per epoch. A kernel API taking a `rate`
  and deriving substeps cannot express "average over 100 km".
- **They are different operators.** A box blur of radius `R` is not the `R`-step limit of a
  4-neighbour diffusion — it is a rectangular window, not a Gaussian. Forcing them onto one
  kernel would move the shipped world's bytes for no gain.

*Where they genuinely do coincide:* both are **O(n) separable sweeps with a fixed scan order and
a scalar-only driver by design** (`isostasy.rs:22-28`, `dc-core/src/field.rs:196-199`) — that is
a shared *determinism discipline*, not a shared operator.

**The kernel isostasy would want** is a **separable smoothing primitive**: *given a plane, a
radius in world units and an edge policy, return the windowed mean in O(n) with a fixed scan
order.* Two arguments it is worth extracting despite being 40 lines: (a) it removes the
per-epoch allocation above by taking caller-owned scratch; (b) the **edge policy** is a real
decision the pass currently makes silently — `box_smooth` shrinks the window at the border
(`isostasy.rs:82-85`), defended in-comment as "keeps the mean well-defined", but at a world edge
that is an *opinion*, and by the opinion-vs-absence test it belongs to the pack.

**6. Storage shape.** `r` wants one value per cell (it is terrain). `exhum` wants **nothing kept
today** — a scalar accumulator whose only intended consumer, the metamorphic expresser, does not
exist (`field.rs:557-577`). 2.27 MiB of world residency serving zero readers.

**7. (my reading)** `r` — obviously yes, it is the terrain. `exhum` — plausibly, *if* the
metamorphic expresser lands: an aureole in a cut face near the player is exactly a refinement
question. Today, no.

---

### 6. `geotherm` — the `temperature` condition-field

**1. Per-cell state.** Reads `grid.t_crust` (f64), `grid.crust_kind` (u8) and the chapter's
advected plates via `tectonics::dominant_kind` (`geotherm.rs:161-163`). Writes
`grid.geotherm: Vec<f64>` — 8 B/cell, **2 376 200 B** (`geotherm.rs:152-165`; plane declared at
`grid.rs:693`).

**2. Locality.** **Point-evaluable given `t_crust`.** `gradient_c_per_m` is a `match` on
`BoundaryKind`, a linear thickness deviation and a clamp (`geotherm.rs:70-96`) — a pure function
of three arguments, and its own tests are written as pure-function tests (`geotherm.rs:175-204`)
precisely because of that. `dominant_kind` is the same closed-form nearest-seed query as
`forcing_at` (`tectonics.rs:289-321`). **The catch:** one of the three arguments, `t_crust`, is
a whole-run integral (pass 2). So the *pass* is analytic and the *value* is historical. That
distinction is the kind of per-pass fact the brief was after: "point-evaluable" is a property of
the expression and can hold while the field is still not reconstructible from parameters.

**3. State vs output.** No persistent state at all. Its parameters are a 7-arm match, 3 crust
baselines, one gain and one clamp — call it **~100 B of constants against 2 376 200 B of
output**, ~24 000×. The per-cell variation is entirely inherited from `t_crust`, `crust_kind`
and the plate table.

**4. Lifetime.** Survives as `DeepField::geotherm` (`field.rs:590`). **The in-sim value is
consumed**: coal rank reads it at finalize through `BurialColumn` (`geotherm.rs:107-130` →
`recorder::DeepStrata::promote_coal`). **The exported plane:** grep as in pass 2 —
`tests/geotherm.rs` (6), `tests/providers_common/mod.rs`, `tests/deep_config_plumbing.rs`,
`examples/coal_walk_tour.rs`, `examples/flow_cost_probe.rs`. **Zero production readers**, exactly
as `field.rs:583-589` claims. The corpus's in-sim-vs-exported distinction holds precisely here
and the doc states it correctly.

**5. Numerics.** No kernel; a per-cell closed form. **If it were written against one, the
primitive it wants is not a solver at all — it is a `map` over declared inputs**: *"for each
cell, evaluate this pure function of these named planes and this world-level table."* Geotherm is
what a **field-valued expression** looks like, and the engine's answer to it is probably a
declaration shape, not a solver.

**Worth flagging for the north-star question:** v1 is explicitly linear in depth
(`temperature_c`, `geotherm.rs:103-105`) and the module names a nonlinear/mantle profile as a
followup (`geotherm.rs:31-32`). *That* heir wants a **1-D vertical integration** primitive — per
column, integrate a depth-varying conductivity. A fourth distinct kernel family, already on the
board.

**6. Storage shape.** **Rule plus parameters.** The stored plane is `f(t_crust[i],
crust_kind[i], dominant_kind(plates, x, y))` and nothing else. If `t_crust`, `crust_kind` and the
chapter table are kept, the geotherm plane is 2.27 MiB of pure redundancy — ~10 flops per cell
to recompute. The one thing that makes it non-trivial is **cadence**: the pass fires every 40
epochs (`GEOTHERM_PERIOD`, `geotherm.rs:57`; declared at `runner.rs:925`), so the stored value is
epoch-160's answer, not epoch-200's, while `t_crust` kept evolving. Re-deriving from the final
`t_crust` would give a **different number**. A small but real example of a field whose stored
value is not a function of the final state — and the only argument for keeping it.

**7. (my reading)** Moderate. A player-visible consequence exists (coal rank, metamorphic
aureoles, hot springs) but all of it is mediated by the record. A refinement operator wants the
*gradient at a point*, which the closed form gives at any resolution — provided it can get
`t_crust` there.

---

## `docs/design/tectonics.md` § 8.2 — the analytic bed-attitude claim

**The claim** (`docs/design/tectonics.md:670-685`): a unit's dip is `g(Σ_{c>u} ∇F_c(x))`,
"evaluated from ~5 KB of state at **collapse resolution, not 460 m resolution**", so no dip
vector need ever be stored.

**Verdict: the claim is *architecturally* sound and *currently unsupported* — for two separate
reasons, one small and one structural. Both are absences in what is kept, not errors in the
reasoning.**

**What the code does keep, and it is the hard part:**

- `DepUnit` carries a **chapter stamp** — `CHAPTER_SHIFT = 22` in the packed `bits` word, and it
  is **inside `MERGE_KEY_MASK`** (`recorder.rs:524,545`), so units of different chapters are
  never merged away. The `u` in `Σ_{c>u}` exists per unit. ✔
- The **chapter table is exported** (`DeepField::chapters`, `field.rs:613`), at the size the doc
  predicted (5.6 KB measured above vs "~5 KB" at `:652`). ✔
- `F_c` is a **genuinely grid-free closed form** — `forcing_at` has no grid term
  (`tectonics.rs:244-284`), so it can be differentiated and evaluated at voxel pitch. ✔
- Units store **no deformation**, exactly as § 8.1 requires (`recorder.rs:507-513` — the unit is
  `bits: u32, packed: u32` and nothing else). ✔

**Gap 1 (small, easily closed): the chapter table alone is not sufficient input.**
`forcing_at(plates, cfg, v_ref, x, y)` reads five `DeepConfig` scalars that are **not on
`DeepField` and not on `Pregen`**: `orogen_width_km` and `arc_gap_km` (`tectonics.rs:253`,
`:274-275`), `thickening_scale` (`:283`), and — through `reference_velocity` (`:235-239`) —
`plate_scale_km`, `advection_plate_widths` and `chapters`. I grepped `DeepConfig` over
`src/pipeline.rs`, `src/pregen/mod.rs` and `src/collapse.rs`: **1 hit, and it is a doc comment**
(`collapse.rs:325`). `Pregen` keeps `seed`, `extent`, `grid`, `pipeline`, `deep`
(`pregen/mod.rs:246-257`) — no config. `DeepField` has no `Serialize` derive
(`field.rs:477-480`), so nothing is being rescued from a serialised sidecar either. So today the
collapse tier **cannot evaluate `F_c` at all**, and "from ~5 KB of state" is off by six scalars.
Trivial to fix; but as written the claim is not supported.

**Gap 2 (structural, and the one that matters): `∇F_c` is not the gradient of surface uplift.**
The doc's premise at `:674-676` is *"the per-chapter uplift field is an analytic function of the
chapter table"*. In the code it is not. `forcing_at` returns **crustal thickening rate**, and
`tectonics.rs:9-10` says so in as many words (*"What that forcing drives is not elevation, it is
crustal thickening"*). The path from forcing to surface is:

```
F_c  --(x dt, accumulate)-->  t_crust  --(box_smooth, radius 109 cells)-->  t_bar
     --(Airy equilibrium)-->  e_eq     --(relax at iso_rate 0.5 toward e_eq)-->  r
                                        ... minus 200 epochs of erosion (track_exhumation)
```

(`uplift.rs:22-37`, `:86-104`; `isostasy.rs:59-61`, `:68-115`.)

Two of those steps destroy the property the claim depends on:

1. **The 100.7 km box mean is a low-pass filter that removes exactly the wavelengths `F_c`
   carries.** The orogen half-width is `orogen_width_km = 25.0` (`grid.rs:601`), and belts narrow
   further under fast convergence (`tectonics.rs:272`). A 219-cell box mean flattens a 25 km
   bump. So `∇(surface uplift) ≠ ∇F_c`; it is `∇` of something with the belt-scale structure
   smoothed out and a foreland moat added — which is the *stated design intent* of the isostasy
   module (`isostasy.rs:15-20`), not an accident. A dip re-derived from `∇F_c` would put ~25 km
   wavelength tilt into a landscape whose actual uplift has none.
2. **Erosion is inside the loop.** `track_exhumation` (`uplift.rs:51-77`) decrements `t_crust` by
   every metre of bedrock removed, which feeds back into `t_bar` and therefore into `e_eq`. The
   surface history is not a function of plate kinematics; it is a function of plate kinematics
   *coupled to* the erosion history, which is not in the chapter table and is not analytic.

**What would make the claim true.** Either (a) re-derive from what actually moved the surface
rather than from `F_c` — which means keeping something the whole-grid isostatic solve produced,
i.e. exactly the per-cell state § 8.2 set out to avoid; or (b) accept that the analytic dip is a
**plausible structural dressing consistent with the tectonic story** rather than a reconstruction
of the sim's own history, and say so. (b) may well be right for a game — a continuously varying
dip that never steps at 460 m is a large visual win even if it is not the sim's true attitude.
But that is a design call the doc does not currently make, because it believes the re-derivation
is exact.

**What I could not check.** Whether an earlier design pass already noticed this — I did not sweep
`journal/`. Whether the intended `g(·)` was ever meant to be calibrated against the smoothed
response rather than the raw forcing. And **unverified**: my claim that the box mean "flattens" a
25 km bump is an argument from window width (219 cells ≈ 100.7 km against a 25 km half-width),
not a measurement; I ran no probe.

---

## How many distinct solver kernels does SOLID EARTH need?

**Four families, and only one of them is a solver.** Six passes, and they do not collapse:

| # | kernel family | operation | passes | is it a "solve"? |
|---|---|---|---|---|
| **K1** | **Stencil diffusion** (exists: `dc_core::field::FieldKernel`) | conservative flux-form relaxation with a monotonicity bound and derived sub-cycling | `diffuse` | yes |
| **K2** | **Separable windowed smoothing** | O(n) box/kernel mean at a radius in **world units**, fixed scan order, declared edge policy | `isostasy` | no — a filter |
| **K3** | **Scatter-with-kernel point evaluation** | superposition of compact kernels over a small world-level site table, evaluable at arbitrary continuous `(x,y)` | `tectonics` (forcing + `dominant_kind`) | no — a closed form |
| **K4** | **Per-cell expression over declared inputs** | a pure function of named planes + a world table, at a declared cadence | `geotherm`, `forcing`/`t_crust` (trivially) | no — a map |

**Which passes share one:** *none of mine share a kernel with another of mine.* That is the
finding. `diffuse` is alone on K1; `isostasy` alone on K2; `tectonics` alone on K3. K4 has
several members but K4 is arguably not a primitive at all — it is a **declaration shape**, which
is what the north star means by "authored in a uniform, self-declaring shape".

**And `expose` is a fifth thing** (ragged-column reduction over the record) that I do not think
belongs in a field-solver taxonomy at all — see its section. If it gets a primitive, the
strongest argument is `weathering.rs:36-38`: three other passes already do the same walk.

**So the honest answer to "one primitive or many": for solid earth the field-*solver* primitive
count is ONE — `FieldKernel`, already extracted, with creep as its only current customer.**
Everything else in my group that we have been calling "a field" is a *stored evaluation of a
closed form*, and what those passes need from the engine is not a solver but (i) a way to declare
"my output is a pure function of `<these inputs>`, materialise it as you see fit", (ii) a cadence
declaration, and (iii) an honest statement of whether the stored copy is the final answer or a
stale snapshot — geotherm's 40-epoch cadence being the live counterexample to
"recomputable ⇒ don't store".

## What a pass should be able to declare (from these six)

The declarations that would have caught something real:

1. **`materialised: cached | authoritative`** — tectonics' 20.39 MiB of chapter planes are a
   cache of a 5.6 KB table; nothing in the code says so, and nothing stops a consumer treating a
   plane as authority. The 3 960× ratio is the argument.
2. **`exported_readers: none`** — four exports in my group have zero production readers
   (`exhum`, `t_crust`, `geotherm`, the chapter table). Every one is *documented* as unconsumed
   in prose (`field.rs:557-612`) — the right instinct, the wrong mechanism, since prose cannot
   fail a build, as that very comment observes at `:559`.
3. **`stale_at_finalize: bool`** — geotherm's cadence means its exported plane is epoch-160's
   answer. Invisible today, and the *only* reason its plane cannot simply be dropped.
4. **`edge_policy`** — `box_smooth` shrinks the window at the border on its own authority
   (`isostasy.rs:82-85`). At a world edge that is an opinion, and by the opinion-vs-absence test
   it belongs to the pack.
5. **`grid_free: bool`** — `forcing_at` and `dominant_kind` have no grid term and could be
   evaluated at voxel pitch; `box_smooth` and `FieldKernel` cannot. A refinement operator has no
   way to ask which it is holding.

## Unresolved / not checked

- I measured nothing. Every byte figure is derived from `size_of` reasoning plus the production
  config constants, with the derivation shown. **No cargo was run**, per brief.
- **Partition:** `expose` is, I think, a record-query pass and belongs with the strata/record
  group. `diffuse` sits on the boundary between this group and whoever owns erosion — I audited
  its *kernel* relationship, not its geomorphology.
- I did not look for field passes outside my list, so I make **no claim** about unassigned
  passes. `head` (`dc:deep/head`) is visibly a field pass in the same runner and is not in my
  partition; I left it alone.
- Whether tectonics' 20.39 MiB precompute beats on-demand evaluation is **unverified**.
- Whether the § 8.2 gap has been noticed before in `journal/` is **unchecked**.
