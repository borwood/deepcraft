# Field inventory — WATER AND TRANSPORT

**Base commit:** `95db80a` (`north-star: strike the retired trust model in the two clocks…`);
worktree tip equals `main` at merge time. Read-only audit — no cargo was run, no source was
edited. This doc is the agent's whole write-set.

**Scope covered:** `drainage` (flood / route / accumulate_area), `transport` (+ incision + the
energy plane), `head` (`dc:field/head`), the **flux record**, and the `eolian` + `wave` agents.

**Production grid used for every byte figure below:** `n = 545² = 297,025` cells at 460 m.
Method: `DEEP_MAX_WIDTH = 550` is a *cap* (`deeptime/field.rs:50`, applied at `:279` as
`cell_m = (extent_m / DEEP_MAX_WIDTH).max(DEEP_CELL_M)`), so `Extent::Medium` lands on 545², not
550² — the reconciliation is recorded at `docs/audits/baseline-2026-07-28/S2-physical-process.md:19`.
Where a doc I cite scaled on 550² I say so. Useful multiples: `1n = 0.297 MB`, `4n = 1.19 MB`,
`8n = 2.38 MB`, `32n = 9.50 MB`, `52n = 15.45 MB`, `64n = 19.01 MB` (decimal MB).

---

## 0. The partition — where I think it is wrong

Three notes, offered rather than forced:

1. **`creep` / `creep_kernel` are in `erosion/` but are NOT mine.** They are the hillslope-creep
   pass and the content half of the E4-1 kernel call. I read them only far enough to establish
   that `dc-core::field::FieldKernel` is *their* extraction, not the water group's. Whoever holds
   hillslope/mass-wasting owns them.
2. **`mfd.rs` / `mfd_law.rs` ARE mine** even though the brief did not name them: they are the
   routing law `drainage` applies, and `route()` is unreadable without them.
3. **A field pass nobody in my brief owns: `dc:deep/climate`** (`runner.rs:708-726`), which writes
   `grid.precip`. It is a genuine per-cell field with a `remarch_interval` cadence, and the eolian
   agent's aridity term reads it (`agents.rs:119`). If no group has it, it is unassigned.

Also: `dc:deep/transport` and `dc:deep/drainage` are **two runner passes over one `Erosion`
struct** (`runner.rs:786-807`). They share every scratch plane. I report per-plane and attribute
the writer, because "the pass's own state" is not separable here — that is itself a finding.

---

## 1. The table

Columns 1–6 of the RETURN spec, one row per pass. Column 7 (refinement) is prose in § 3.

| pass | reads (planes) | writes (planes) | elem × bytes/cell | locality | own state vs output | survives to `DeepField`? | numerics today | wants what shape |
|---|---|---|---|---|---|---|---|---|
| **drainage / flood** | `surf` f64 | `filled` f64, `order` u32, `done` bool, `heap` | 8 / 4 / 1 | **NO** — global min-heap frontier | state 4.06 MB vs output 2.38 MB | `filled` no; `lake` (derived from it) yes, tests-only | bespoke `BinaryHeap` priority-flood, inline | one value per cell (`filled`) + a world-level cell permutation (`order`) |
| **drainage / route** | `surf`, `filled`, `area` (**lagged 1 epoch**) | `recv` i32, `mfd_w` f64×8 | 4 / 64 | **YES** — pure per-cell 8-neighbour gather, `par_iter` byte-identical | no extra state vs output 20.20 MB | `recv` yes (**exported, tests/examples only**); `mfd_w` no | inline `powi` normalised partition (`mfd.rs`), no shared kernel | `recv` is a summary and should die; the real output is **per-face**, 8 shares per cell |
| **drainage / accumulate_area** | `mfd_w`, `order`, `recv` | `area` f64, `out_area` f32×8 | 8 / 32 | **NO** — serial scan in reverse `order`; a cell needs every contributor first | no extra state vs output 2.38 + 9.50 MB | `area` yes (**tests/examples only**); `out_area` no | inline DAG scan with a residual-exact split, no shared kernel | **sparse per-face record** — `out_area` already is that, densely |
| **transport** (+ incision, energy) | `order`, `recv`/`mfd_w`, `filled`, `area`, `sus_flow`, `grid.r`/`h`, `window`/`shares` | `grid.r`, `grid.h`, `dh`, `energy`, `qs`, `qs_sp`, `dep_sp`, `tlayout`, `out_load`, `out_face_load`, `ledger`, `split_residue` | `dh`/`energy`/`qs`/`out_load` 8 each; `out_face_load` 32; species CSR sparse | **NO** — same serial DAG scan, and it mutates terrain as it goes | scalar scratch 19.02 MB + CSR (sparse); "output" is the terrain itself | `grid.r`/`h` → `surf` yes (**the world's main artifact**); `energy`, `dh`, `qs`, `out_*` no | **all inline** — capacity, competence ceiling, coarsest-first drawdown, per-species residual split; zero shared kernel | terrain: one value per cell. Load: **a value per species per cell**, already CSR. Ledger: **a world-level table** (~136 B) |
| **head** (`dc:field/head`) | `grid.strata` (whole record), `filled`, `routed`, `ground`, `area` | `grid.head` f64, `grid.head_exchange` f32 | 8 / 4 | **NO** — Gauss–Seidel to a global steady state; boundary values propagate grid-wide | transient solve state 13.36 MB vs persistent output 3.56 MB (**3.75×**) | `head` **yes** (`field.rs:733`); `head_exchange` **no** (gen-time cache) | **hand-rolled Gauss–Seidel**, `relax_cell` at `head.rs:371-406` | one value per cell — plus a per-column derived struct (`ColumnHydro`) it recomputes every march |
| **flux record** | `out_area`, `out_face_load`, `area`, `routed_surface`, `grid.head_exchange` | `FluxAccum.mag`/`load` f32×13, then `FluxRecord` CSR | accum 52 + 52; record 16 B/**entry** | n/a — pure accumulation, no solve | accumulator **30.89 MB** vs the record it produces (~42.6 MB, stale — see § 2.6) | **yes** (`DeepField::flux`), **read by tests/examples only** | none — it is a store, not a solver | **sparse per-face record**, keyed `(cell, chapter, face)` — it already is exactly this |
| **eolian** (`wind`) | `grid.precip`, `grid.bio_resist`, `grid.r`/`h`, `grid.strata`, `climate::zonal_wind` | `grid.h`, `grid.strata`, `ledger.eolian_to_sea_m` | **no plane of its own** | **NO** — a 1-D serial march along each row; a cell's outcome depends on everything upwind | state = one `Vec<f64>` of axis length vs no output plane | its *effect* survives in `surf` + `strata`; nothing named "eolian" is exported | inline advection march, `agents.rs:60-207` | **nothing kept** — it is a redistribution whose trace is the record |
| **wave** | `grid.r`/`h`, `grid.strata`, `sea_level`, `providers.wave_energy` | `grid.r`, `grid.h`, `grid.strata[j]`, `ledger.wave_*` | **no plane of its own** | **YES-ish** — per-cell 8-neighbour argmin scatter; scalar only because it writes a neighbour | same one `Vec<f64>` scratch | effect in `surf` + `strata`; nothing exported | inline, `agents.rs:230-319` | **nothing kept** |

---

## 2. Per pass, the prose

### 2.1 drainage — flood

**Per-cell state.** Reads `surf` (f64, built as `r + h`, `flood.rs:162-174`). Writes `filled`
(f64, `flood.rs:184-186, 212`), `order` (`Vec<u32>`, `:203`), `done` (`Vec<bool>`, `:187, 202`),
and churns `heap: BinaryHeap<Reverse<Item>>` where `Item` is `{f64, u32}` (`flood.rs:24-27`).
At production: `filled` 2.38 MB, `order` 1.19 MB, `done` 0.297 MB, heap peak ≲ `n × 16 B` = 4.75 MB
if every cell were simultaneously queued (an upper bound I derived from the struct, **not** a
measurement — the real peak is **unverified**; nothing instruments it).

**Locality — NO, and it is the strongest no in the group.** `flood()` seeds every sea/border cell
into a min-heap and pops globally in ascending filled elevation (`flood.rs:188-218`). A cell's
`filled` is `max(own surface, popped neighbour + 1 mm)`, so the value at one cell is a function of
the **entire monotone path back to the nearest outlet**. There is no local formula. The module's own
words: *"Serial by nature (a global elevation-ordered frontier); the measured deep-time floor S9b
could not break without abandoning byte-identity"* (`flood.rs:178-180`). `flood_fill_tiled`
(`:96-158`) exists **only** to measure the optimistic parallel ceiling and is explicitly
*"never on the sim path"*.

**State vs output.** Its state (`done` + `order` + heap) is 1.5–3× the field it produces. That
ratio is unusual in this group and it is a direct consequence of the algorithm, not of sloppiness.

**Lifetime.** `filled` does not survive. What survives is `DeepField::lake`, derived from
`filled > surf` — and `head.rs:472-476` deliberately does **not** use `filled` directly, it uses
`filled − routed`, because a `filled` carried onto a later `ground` reads *"a whole world of
fictitious ponds"* (`head.rs:419-424`). That is a real, recorded trap in the export.

**Numerics / kernel.** No shared kernel. The operation is a **priority-flood / monotone fill**
(Barnes 2014, named at `flood.rs:178`). A kernel for it would have to expose: seed predicate,
a total order on cells, an 8- or 4-neighbourhood, a monotone update rule, and — critically —
**the pop order as a returned artifact**, because `order` is what two downstream passes walk.

**Storage.** One value per cell, plus a world-level permutation. The permutation is the interesting
part: it is not a field at all, it is a *schedule*, and it is the thing `accumulate_area` and
`transport` actually consume.

### 2.2 drainage — route

**Per-cell state.** Reads `surf`, `filled`, and `area` **as of the previous epoch**. Writes
`recv: Vec<i32>` (4 B/cell) and, under MFD, `mfd_w: Vec<f64>` sized `n × 8` (`routing.rs:23-25`)
— **64 B/cell, 19.01 MB**, the single largest gen-time plane in the drainage solve.

**Locality — YES.** `route()` is a pure per-cell function of an 8-neighbourhood plus one scalar
(`area[i]`), which is why both branches run under `par_iter_mut` byte-identically
(`routing.rs:106-149`). This is the only phase in the group that is genuinely point-evaluable.

**The one-epoch lag is load-bearing and honest.** The hybrid-`p` law needs `χ = A·S²`, and `A` comes
from `accumulate_area`, which runs *after* this and *from* this phase's weights. The circularity
has no fixed point, so the exponent reads last epoch's `area` (`routing.rs:82-100`). At epoch 0
that is all zeros, so `χ = 0`, `p = p_hill` and the first epoch is maximally dispersive — stated
in code as *"the honest initial condition rather than a guess."*

**Lifetime — and this is the group's clearest summary-vs-authority case.** `recv` survives into
`DeepField::recv`. Under MFD it is **no longer the routing**; it is the argmax share, a projection
kept only because two consumers want one arrow per cell (`routing.rs:250-269`). Its doc says so in
as many words: *"it is a **summary of an authority** in the exact sense ARCHITECTURE.md warns
about."* The two in-sim consumers are real (`biotic.rs:697` and the `depth_to_water` provider's
`DrainagePass`, `providers/depth_to_water.rs:48`), so the *in-sim value* is consumed. The
**exported plane** is not.

> **Absence claim, with its search.** I grepped `\.recv\b|\.area\b|\.lake\b` over
> `crates/**/*.rs` in the worktree (Grep tool, `output_mode=content`). Hits on a **`DeepField`**
> binding: `tests/deep_config_plumbing.rs` (7), `tests/flux_record.rs` (4),
> `tests/head_field.rs` (4), `tests/mfd_routing.rs` (3), `tests/tectonic_history.rs` (6),
> `tests/providers_common/mod.rs` (3), `examples/{flow_cost_probe, mfd_probe, hybrid_p_probe,
> colluvium_probe, facies_probe, walk_tour_0115, tectonic_spike, head_field_probe}` (≈20).
> **Non-test, non-example hits: only `field.rs:678-679` (construction) and `field.rs:1006-1014`
> (residency accounting).** `collapse.rs` was grepped separately for `deep\.(recv|area|lake)` —
> **0 hits**; it touches `pregen.deep` only for `strata`, `surface_at_voxel`,
> `regolith_at_voxel`, `deep_coords`, `registration`. So: the exported `recv`/`area`/`lake` planes
> have **zero production consumers**. This corroborates `docs/dependency-graph.md:210` — *"P3 …
> BUILT AND IDLE. Zero production consumers, on purpose"* — which I did not have to take on faith.

**Numerics / kernel.** Inline. As a kernel operation this is a **per-cell neighbourhood partition**:
gather 8 drops, apply a monotone weight law, normalise, floor, emit. It is the closest thing in my
group to something a generic stencil primitive could serve — but the payload is *per-edge weights*,
not a scalar, so a scalar-convolution kernel would not cover it.

**Storage.** The honest output shape is **8 shares per cell** (`mfd_w`), i.e. a per-face record.
`recv` is the vestigial scalar and should be read as a compatibility shim.

### 2.3 drainage — accumulate_area

**Per-cell state.** Reads `mfd_w`, `order`, `recv`. Writes `area` (f64, 2.38 MB) and, when the flow
record is on, `out_area` (f32 × 8 = 9.50 MB, allocated at `ledger.rs:257-261`).

**Locality — NO.** `accumulate_area` re-seeds every cell to `1.0` and then scans `order` in
reverse, pushing each cell's accumulated area to its receivers (`routing.rs:182-237`). Drainage
area at a cell is the size of its entire upstream catchment: it is a **transitive closure**, not a
neighbourhood query. The correctness argument is worth quoting because it decides what a kernel
would need: the priority-flood pop order is strictly ascending in `(filled, index)`, and every
routed edge goes to a strictly-smaller `filled`, so reversed it is a valid topological order of the
**DAG**, not just of a tree (`routing.rs:164-181`). *"MFD needs no new topological sort — it needs
the observation that the old one was never about the tree."*

**The residual rule.** The last weighted direction takes `a − Σ(earlier)` rather than `w·a`
(`routing.rs:219-234`), so the split is exact by construction. The comment names why: a per-hop ulp
compounds over a thousand hops and *"would leak silently."* Any accumulation kernel that does not
offer this is not a substitute.

**State vs output.** No additional persistent state; it borrows `order` and `mfd_w`. Output
2.38 MB (`area`) + 9.50 MB (`out_area`).

**Lifetime.** `area` survives into `DeepField::area` — exported, **tests/examples only** (same grep
as § 2.2). The *in-sim* value is heavily consumed: `biotic.rs:696,707`, `head.rs:493`
(`STREAM_ANCHOR_AREA`), `transport.rs:254-256` (`A^m`), `mfd_law` (`χ = A·S²`), `flux.rs:854`
(the sink case). The corpus's in-sim-vs-exported distinction holds exactly here.

**Numerics / kernel.** Inline. The operation is **accumulation over a DAG in a supplied topological
order, with an exact-residual share split**. Note it is *the same traversal* as `transport` below.

### 2.4 transport (+ incision, the energy plane)

**Per-cell state.** The largest read/write set in the group. Reads `order`, `recv`/`mfd_w`,
`filled`, `area`, `sus_flow`, `grid.r`/`grid.h`, `window`/`shares`/`tlayout`. Writes `grid.r`,
`grid.h` (the terrain — this is the pass that *is* the landscape), plus `dh` (f64), `energy` (f64),
`qs` (f64), `out_load` (f64), `out_face_load` (f32×8), the CSR species planes `qs_sp`/`dep_sp` and
their `tlayout`, and the world-level `TransportLedger` + `split_residue`. Cites: `transport.rs:534-765`
for the chain, `erosion/mod.rs:204-232` and `:285-312` for the plane declarations, `ledger.rs:257-266`
for the flow-record buffers.

Bytes: `dh` 2.38 + `energy` 2.38 + `qs` 2.38 + `out_load` 2.38 + `out_face_load` 9.50 = **19.02 MB**
of scalar scratch, plus `species_bytes()` (`erosion/mod.rs:646-654`) which is sparse and
world-dependent — I did **not** measure it (that needs a run, and I ran nothing; **unverified**).

**Locality — NO, and doubly so.** It walks the same reversed `order` (`transport.rs:558`, `:627`),
so a cell needs its complete upstream load. But it also **mutates `grid.r`/`grid.h` as it goes**,
so the value at a cell depends on the epoch's traversal state, not merely on a converged field.
`transport` is the phase that makes the whole drainage solve non-point-evaluable even in principle.

**State vs output.** The pass's own scratch (19.02 MB scalar + CSR) exceeds the *incremental* field
it publishes each epoch, because its real output is the mutation of two 2.38 MB terrain planes. The
"output" framing does not fit this pass: it is an in-place operator, not a field producer. That is
worth saying plainly to the field-primitive question — **not every deep pass produces a field.**

**Lifetime.** `grid.r + grid.h` → `DeepField::surf`, the world's primary artifact, read by
`collapse.rs` in production. `energy`, `dh`, `qs`, `out_load`, `out_face_load` are **gen-time only**
— `DeepField` has no `energy` field (I read the full field list, `field.rs:480-613`). The energy
plane's only surviving trace is the `EnergyBand` tag baked into each `DepUnit` by the recorder
(`record::energy_band`, re-exported at `erosion/mod.rs:127`). So: energy is *consumed then
discarded*, and the three-band classification it produced is what persists.

**Numerics / kernel — nothing shared, and the most content-shaped arithmetic in the group.**
`exchange_cell` (`transport.rs:244-441`) is capacity `k·A^m·S^n`, cover-shielded incision
`k_b·A^m·S^n·exp(−h/h*)`, an entrainment split, a coarsest-first capacity drawdown, and a
competence ceiling (`settle_above_competence`, `:471-502`). Every constant is anchored and
documented (`COMPETENCE_SCALE = 420` derived as `settle_energy(sandstone)/ENERGY_LOW_MED`, with
an asserting test at `:816-836`). **None of this belongs in an engine kernel** — it is pack physics.
What *could* be a kernel is only the traversal + the exact split, which it shares with § 2.3.

**Storage.** Three different shapes in one pass, and they are genuinely different:
terrain = one value per cell; suspended load = **a value per species per cell**, already CSR over a
per-epoch-rebuilt layout (`build_transport_rows`, `:776-793`); the transport ledger = **a small
world-level table**, 15 `f64` + 2 `u64` ≈ 136 B (`ledger.rs:30-130`), which is a real and
under-appreciated fourth shape.

### 2.5 head (`dc:field/head`)

**Per-cell state.** Reads `grid.strata` — **the whole record, every column, every march**
(`head.rs:459-463` calls `column_hydro` per cell, which walks `strata.units` twice, `:277-320`).
Also reads `filled`, `routed`, `ground`, `area`. Writes `grid.head` (f64, 2.38 MB) and
`grid.head_exchange` (f32, 1.19 MB) at `head.rs:535-536`.

**Transient solve state, and it dwarfs the output.** `march` allocates per firing:
`t` f64 (2.38), `unconfined` bool (0.297), `pinned` bool (0.297), `h` f64 (2.38), `k_vert` f32
(1.19), `cap` f32 (1.19), `free: Vec<usize>` ≤ 8n (2.38), `exchange: Vec<f32>` (1.19)
= **≈ 13.36 MB** transient against **3.56 MB** persistent — **3.75×**. Method: sum of the `vec!`
allocations at `head.rs:452-457, 506, 524` at `n = 297,025` with `size_of::<usize>() = 8`. Every one
of these is freshly allocated per firing (every 20 epochs, `HEAD_PERIOD` at `:140`), not reused
scratch — the only pass in my group that allocates its whole working set inside the solve.

**Locality — NO.** 24 Gauss–Seidel sweeps with alternating forward/reverse raster order
(`head.rs:508-522`). A reverse sweep carries a boundary value across the whole grid in one pass;
the design note says a purely local scheme would need `O(w²)` iterations to do it (`:142-150`).
The sweep count is **fixed, not convergence-checked** — the residual is returned and reported, never
used to stop, per the S9 determinism rule. So the value at one cell is a function of the boundary
conditions everywhere plus the transmissivity field everywhere.

**Lifetime.** `grid.head` → `DeepField::head` (`field.rs:733`). `head_exchange` deliberately does
**not** survive: `grid.rs:707-711` calls it *"a cache of `vertical_exchange`, never an authority …
gen-time only."* Its in-sim consumer is real and singular: `flow_record_pass` at `runner.rs:538-539`
feeds it to `FluxAccum::add_epoch_vertical`, which is what populates the `Down`/`Up` faces.

> **Absence claim, with its search.** I grepped `\.head\b|head_exchange|\.flux\b|\.lake\b` over
> `**/*.rs` in the worktree. Hits on `DeepField::head`: `tests/artifact_tripwires.rs:501-505`,
> `tests/head_field.rs` (×7), `examples/head_field_probe.rs` (×6), `examples/flow_cost_probe.rs:204`,
> plus `field.rs:733` (construction) and `field.rs:1011` (residency). **Zero production consumers.**
> The one look-alike, `water/sat.rs:366` `self.head(i, y)`, is a *different* head — the pregen
> `H = y + sat` bound-water proxy, i.e. exactly the thing `head.rs:10-13` exists to supersede. Two
> things named "head" in one crate, one of them the retired proxy, is a naming hazard worth flagging.

**Numerics / kernel — the load-bearing answer of this audit.** `relax_cell` (`head.rs:371-406`) is:
a D4 gather, **harmonic face conductance** `2ab/(a+b)` (`:365-368`), a weighted average
`num/den`, and then **an upper obstacle** — `if unconfined[i] { v = v.min(ground[i]) }`. That one
line is the seepage face, and the module says it is *the whole difference* between confined and
unconfined; artesian is *"the absence of a cap"* (`:396-402`).

Now compare `dc_core::field::FieldKernel`. That kernel is a 4-neighbour stencil, flux-form, on a
frozen potential, with a content-declared coefficient field and **a lower obstacle** (the donor
limiter `scale[i] = min(1, state_i / Σrequested)`, `dc-core/src/field.rs:324-340`). Head is the
**same family with the obstacle on the other side and an elliptic rather than parabolic target**.
That is not a coincidence: `dc-core/src/field.rs:54-55` names `ImplicitBE` as E4-2, and an implicit
backward-Euler step *is* a linear solve on this stencil. **If head were written against a kernel,
that kernel is E4-2 plus (a) harmonic inter-cell conductance derived from a per-cell transmissivity,
(b) a Dirichlet pin set, (c) a per-cell one-sided obstacle, (d) a deterministic fixed sweep count
with a reported residual.** Four small generalisations of one existing primitive — **not a second
primitive.**

**Storage.** One value per cell for the field. But the *derived* per-column `ColumnHydro`
(transmissivity, `k_vertical`, `cap_m`, `confined`) is recomputed from scratch every march and
thrown away — 4 values per column that a "derived column summary" shape would name. The module is
proud of not storing it (S-2, `head.rs:29-33`) and I think that is right, but it means the head
pass's real read-set is *the entire stratigraphic record*, the heaviest read in my group by far.

### 2.6 the flux record

**Per-cell state.** Not a solver — a store. The gen-time accumulator `FluxAccum` (`flux.rs:689-707`)
holds two dense `f32` planes of `n × FACE_SLOTS(13)`: `mag` and `load`, **15.45 MB each, 30.89 MB
together**, plus `simul_seen` (one bit per cell, 37 KB; its own doc says 36 KB, scaled on 550²) and
a growing `pending` vector. The finished `FluxRecord` is CSR: `entries × 16 B + (n+1) × 4 B`
(`flux.rs:598-602`; `a_flux_entry_is_sixteen_bytes` asserts the 16 at `:967-970`).

**Production size — stale, and I am flagging it rather than repeating it as current.** The corpus's
measured figure is **42.6 MB** (`docs/spines.md:1848`, describing what `flow_cost_probe`'s
itemisation was off by). I did **not** re-measure it, and it predates P11 slices 1–3, which moved
`GOLDEN_FLUX` three times (`tests/flux_record.rs:68-80`). Treat it as order-of-magnitude only. The
entry counts I *can* cite verbatim are from the small test world: 297,720 → **291,663** across
journal/0124 (`tests/flux_record.rs:50-51`).

**Locality.** Not applicable in the solver sense — but worth stating precisely: an entry is
**written by the pass that moved the thing**, never re-derived. `out_area` from `accumulate_area`,
`out_face_load` from `transport` (`erosion/mod.rs:300-310`: *"two derivations of one quantity is
exactly the drift flow.md § 3 exists to prevent"*). And **`in_faces` is a query over the
neighbour's stored half-face, not a second copy** (`flux.rs:526-550`) — which is why a refinement
built on face data *cannot* disagree across a cell boundary. That is the single most reusable idea
in my group: **the directed half-face as the storage primitive.**

**State vs output.** The accumulator (30.89 MB) is comparable to the record it emits. It is
dense-per-chapter and flushed sparse (`flush_chapter`, `:750-781`) — the density is the price of
not sorting.

**Lifetime.** `DeepField::flux` survives. **Read by tests and examples only** — same grep as § 2.5:
`tests/{flux_record, head_field, mfd_routing}.rs`, `examples/{flux_record_probe, head_field_probe,
flow_cost_probe}.rs`, plus `field.rs:745` (construction) and `field.rs:1026` (residency). The one
non-test hit, `runner.rs:527,538`, is `ctx.flux` — the gen-time accumulator, not the archive.
`field.rs:553-555` already says so: *"Nothing at runtime expresses it yet, deliberately."*

**Storage — it is already the right shape, and it answers one of the brief's questions directly.**
`FaceKey` (13 variants: 8 lateral, 2 vertical, 3 boundary, `flux.rs:153-194`) + directed half-faces
+ CSR-by-cell + chapter keying, with the stratum slot **derived** rather than stored
(`slot_for_chapter`, `:675-677`) so an eroded chapter honestly answers `None` instead of pointing at
a stranger's stratum. Measured sparsity is exposed as a first-class number
(`FluxCensus::face_sparsity`, `:632-638`).

### 2.7 eolian (`wind`)

**Per-cell state.** No plane of its own. Reads `grid.precip`, `grid.bio_resist`, `grid.r`/`h`,
`grid.strata`, and `climate::zonal_wind(lat)`. Writes `grid.h`, `grid.strata` (via
`deposit_moved` / `erode`), and `ledger.eolian_to_sea_m`. Its only scratch is
`row: Vec<f64>` of species-axis length (`agents.rs:57`) — bytes = registry width × 8, tens of bytes.

**Locality — NO, and for a different reason than everything above.** It is a **1-D advection march**:
per row, direction = `sign(zonal_wind)`, with a serial `load` variable carried across the row
(`agents.rs:60-177`), picking up where arid/bare and dropping where humid/vegetated. A cell's ΔH
depends on the entire upwind traverse. Row conservation is enforced by settling whatever is aloft at
the downwind land edge (`:179-206`).

**State vs output.** Both ≈ zero. It publishes no field at all.

**Lifetime.** Nothing named eolian survives as a plane. Its trace is in `surf` and in the `DepTag`s
(`Eolian::Dune` / `Eolian::Loess`) baked into the record — which *are* read in production, through
`litho_of_tag` and the collapse tier.

**Numerics / kernel.** Inline. As an operation it is **directional advection with a carried
reservoir and source/sink terms** — a genuinely different kernel shape from both the DAG scan and
the diffusion stencil. A kernel for it would need: a direction field, a carried scalar, and
pickup/deposition closures.

**Storage.** Nothing kept, and that is right — the record is the archive.

### 2.8 wave

**Per-cell state.** No plane. Reads `grid.r`/`h`, `grid.strata`, `sea_level`, and
`providers.wave_energy`. Writes `grid.r`/`grid.h` at `i`, `grid.h`/`grid.strata` at the chosen
offshore sink `j`, and two ledger counters (`agents.rs:230-319`).

**Locality — the most local pass in my group.** A cell is attacked iff it sits in
`(sea, sea + wave_band_m]` and has a subsea 8-neighbour; the sink is the deepest such neighbour
(`:262-270`). Everything is a function of an 8-neighbourhood plus two scalars. It is scalar only
because it writes a *neighbour's* cell — a scatter, not a dependency. That distinction matters for
a kernel: this is exactly the case a **gather reformulation** (the same move `erosion/mod.rs:60-65`
records for creep) would parallelise.

**Lifetime.** Effect only, in `surf` and the record. Nothing exported.

**Numerics / kernel.** Inline. Operation: **per-cell predicate + argmin-neighbour scatter transfer**
— structurally route's argmax pick, running the other direction.

---

## 3. Refinement relevance — MY ANALYSIS, NOT A FINDING

Marked clearly as my reading. I ran nothing and consulted no ruling on this.

- **`filled` / lake depth — probably yes.** `filled − routed` is standing-water depth, and a
  refinement expressing a lake shoreline or a pond edge near the player needs a water *surface*, not
  an elevation. It is currently thrown away and reconstructed as a bool (`lake`), which loses the
  depth. My reading: the bool is the lossy artifact; the difference is the useful thing.
- **`mfd_w` / `out_area` (per-face discharge) — yes, and this is the strongest case in my group.**
  Sub-cell channel geometry is a statement about *where within the cell* the water concentrates, and
  a per-face partition is the only thing in the deep tier carrying directional information at
  sub-cell granularity. The half-face involution (`FaceKey::opposite`, `flux.rs:283-289`) means two
  neighbouring refinements read the same number and cannot disagree — seamlessness for free. Against
  this, `north-star.md:163` records that *"no magnitude of flux record produces a channel"* without
  a sub-cell geometry model; so my reading is that the flux record is a **necessary input, not a
  sufficient one**, which is consistent with that line rather than contradicting it.
- **`energy` (transport capacity) — plausibly yes, indirectly.** Capacity sets the competence
  ceiling, and the ceiling decides grain size. A refinement arranging material within a cell (a
  gravel bar vs a mud drape) wants grain size. But the energy plane is discarded and only its
  three-band classification survives in the tag — my reading is that three bands is coarse for
  sub-cell arrangement, and this is the one place where the discard actually costs something.
- **`head` — yes, but for a narrow and specific thing.** Head above ground (excluding lakes) is a
  spring; head below ground is a water-table depth. Both are sub-cell *placement* questions — where
  in the column the saturation boundary sits, where on the hillside the seep emerges. Nothing else
  in the deep tier can answer either. `head_field_probe` already ranks cells by `surf − head`
  (`examples/head_field_probe.rs:308-311`), which is exactly a depth-to-water query.
- **`area` — mildly.** Useful as a channel-presence prior, but it is a scalar and MFD has already
  demoted the scalar view; the per-face record supersedes it.
- **`recv` — no.** It is the summary its own doc says it is.
- **eolian / wave — no field to want.** Their refinement-relevant output is already in the record as
  a facies tag, which is the right place for it.

---

## 4. How many distinct solver kernels does this group need?

**Five candidate operations. Two of them share one kernel; one is a generalisation of the kernel
that already exists; two are genuinely new families.**

| # | operation | passes | relation to `dc-core::field::FieldKernel` |
|---|---|---|---|
| **K1** | **priority-flood / monotone fill** over a scalar potential, returning both the filled field **and the pop order** | `flood` | **unrelated.** Ordered global frontier, inherently serial, no stencil arithmetic |
| **K2** | **ordered scan over a DAG** with a caller-supplied topological order, a per-cell body, and an **exact-residual share split** | `accumulate_area`, `transport` (and `build_transport_layout`, the same walk with `u64` ORs instead of `f64` adds) | **unrelated.** No stencil; the primitive is the traversal + the split |
| **K3** | **conductance-stencil relaxation with pins and a one-sided obstacle** — harmonic face conductance, Dirichlet boundary set, fixed deterministic sweep count, reported residual | `head` | **the same family.** This is E4-2 (`ImplicitBE`, named at `dc-core/src/field.rs:54-55`) plus harmonic conductance, Dirichlet pins, and an **upper** obstacle where E4-1 has a **lower** one |
| **K4** | **per-cell neighbourhood partition** — gather N drops, apply a monotone weight law, normalise, floor, emit per-edge weights | `route`; `wave`'s sink pick is its degenerate argmin form | **adjacent.** Same 4/8-neighbour gather shape, but the payload is per-edge weights, not a scalar net |
| **K5** | **directional advection march with a carried reservoir** — a 1-D traverse along a direction field, with pickup and deposition closures | `eolian` | **unrelated** |

So the honest count is **five operations, four kernels** (K2 covers two passes), of which **one
(K3) is a generalisation of the existing primitive rather than a new one**, and **one (K4) is
arguably too thin to be worth a kernel** — it is a stencil gather the pass expresses in ten lines
and that already parallelises byte-identically without help.

**The two answers I would put weight on:**

1. **"Field" is not one category in this group, and the split is not along the axis you might
   expect.** The dividing line is not physical (water vs rock) but **algorithmic**: three of my
   passes are *ordered traversals* (flood, accumulate_area, transport) where the order **is** the
   semantics, and one is a *relaxation* (head) where the order must not be the semantics. A single
   "field solver" abstraction covering both would have to make ordering a first-class declared
   property. `FieldKernel` today makes exactly the opposite choice — it is order-independent by
   construction, and `dc-core/src/field.rs:276-279` says so deliberately (*"substepping is
   deliberately NOT locally adaptive … local adaptivity breaks order-independence"*). That is
   correct for diffusion and it is **structurally unable** to host K1 or K2.
2. **What a pass should be able to declare, from the evidence here.** The runner already makes
   passes declare `reads` / `writes` / `reads_prev` / `schedule` (`runner.rs:786-848`). The three
   things that are *not* declarable today and that cost something in my group are:
   - **lifetime** — `energy`, `dh`, `head_exchange`, `out_*` are gen-time by convention and by doc
     comment, never by declaration, so a plane's mortality is knowledge you have to hold;
   - **shape** — `mfd_w` (per-face), `qs_sp` (per-species CSR), `ledger` (world table) and `area`
     (per-cell) are four different storage shapes with no vocabulary distinguishing them;
   - **derived-vs-authority** — `recv` and `head_exchange` are both explicitly summaries with
     agreement tests (`the_exchange_plane_agrees_with_the_field`, `head.rs:729-756`), but nothing in
     the pass declaration says "this is a summary of that."

---

## 5. What I could not resolve

Stated as unresolved, not glossed.

1. **The flux record's current production residency.** The 42.6 MB in `spines.md:1848` predates
   P11 slices 1–3. Resolving it needs `cargo run --example flow_cost_probe`; I ran no cargo.
2. **`Erosion::species_bytes()` at production scale.** Sparse and world-dependent; needs a run.
3. **The priority-flood heap's real peak occupancy.** I give an upper bound of `n × 16 B`; nothing
   instruments the actual peak, so the true figure appears to be unknown to the corpus.
4. **Whether the `depth_to_water` provider is registered in production.** It takes `area` and `recv`
   through a `DrainagePass` struct (`providers/depth_to_water.rs:45-48`), which is an *in-sim* read,
   but I did not trace whether that provider is on by default. My § 2.2 absence claim is about the
   **exported `DeepField` planes** and is unaffected either way.
5. **Whether `dc:deep/climate` belongs to any group.** Flagged in § 0; not mine to assign.
