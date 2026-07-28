# S7 results — worldgen: coarse pregen + lazy pyramid

Status: spike complete, 2026-07-18. Code in `crates/dc-worldgen` (`pregen/` =
the finite deep-time pipeline, `collapse.rs` = the lazy pyramid); tests in
`crates/dc-worldgen/tests/s7_{pregen,walk,handoff,measurements}.rs`. Additive
changes to `dc-sim` are listed at the end. Numbers below from the Windows dev
box, release profile, seed `0x0D5E_ED57_2026`.

> **⚠ THE SETTLEMENT-HISTORY HALF OF THIS SPIKE NO LONGER EXISTS (2026-07-28,
> journal/0121).** `pregen/history.rs`, `Pregen.{ledger, overlay, sites,
> n_polities, observe_count}`, `collapse.rs::ruin_posts`, the `dc:pass/history`
> pass and `tests/s7_handoff.rs` were **removed** on the user's direction
> (2026-07-26): unratified early-bootstrap content, no evo/socia/civ model behind
> it even at the design stage. **This document is preserved as the measurement
> record it is** — the numbers below were really measured and are not being revised —
> but every sentence about sites, polities, sacks, ruin posts, the year-zero
> ledger handoff and the `Site`/`Polity` fact kinds describes code that is gone.
> The rest of the spike (topology, the pyramid, the wilds, the lookahead bounds,
> chunk latency) is live and unaffected.

## Topology recommendation: continent-disc in a world-ocean

The design doc left east–west wrap vs continent-disc open. **Recommendation:
continent-disc**, implemented and validated here. Rationale:

1. **The chunk lattice is natively planar and unbounded.** dc-core chunk
   positions are plain `i32` on all axes. An east–west wrap forces modular
   arithmetic through every pyramid level, every neighbor lookup, and a
   special seam column where `x` wraps — cost with no consumer, since nothing
   below the coarse levels ever sees "around the world".
2. **Border wilds work on every compass point.** The decided wilds design
   ("the edge fades into endless mystery") holds uniformly: the pregen grid's
   outer ring is forced abyssal ocean, and beyond the grid the same lazy
   machinery runs on synthesized coarse cells forever — deepening ocean,
   polar ice above ~84° latitude. A wrap topology only gets wilds at the two
   polar caps.
3. **Climate closure does not need a closed surface.** Latitude is a
   prescribed field (compressed 8°→78° south-to-north across the extent, per
   DF practice), winds are zonal by band, and moisture advects from the
   world-ocean — a cylinder would prescribe exactly the same fields.
4. **What the disc gives up** is tectonics on a truly closed surface: plates
   are Voronoi cells on a bounded grid and the ocean ring acts as the
   closure, so every margin is eventually passive and there is no
   antipodal continent. Judged acceptable — the interior still gets real
   convergent/divergent/transform boundaries with provenance, which is what
   terrain reads from.

Walking 10 000 chunks east (288 km) from the world centre crosses continent →
shelf → world-ocean → off-grid abyss with no discontinuity and no special
casing; the wilds boundary registered at chunk 4157 of the ~4352-chunk
half-extent, exactly where the grid ends.

## Per-level resolution spec (N=2 scale, 0.9 m voxels)

| level | edge | voxels | count at Medium | provenance |
|---|---|---|---|---|
| coarse cell | 14.7456 km | 16 384 | 17×17 (finite) | **pregenerated** |
| region | 7.3728 km | 8 192 | lazy, unbounded | collapse rule |
| locale | 460.8 m | 512 | lazy, unbounded | collapse rule |
| chunk-column | 28.8 m footprint | 32 | lazy, unbounded | collapse rule |
| chunk | 28.8 m cube | 32³ | lazy, unbounded (3D) | slice of column |

The coarse cell sits in the design doc's 10–100 km band. Elevation rides a
15-level midpoint-displacement lattice (level 0 = cell corners at 16 384-voxel
spacing … level 14 = per-voxel), which *is* the collapse rule in its purest
form — child = parent average + addressed jitter scaled by
`roughness · 0.55^level`, roughness inherited from tectonic provenance
(orogeny 420 m … ocean floor 25 m). Adjacent columns share lattice ancestors,
so continuity is by construction, not by stitching. Semantic state (rivers,
sites/ruins, civilized-fringe) rides explicit region → locale → column
records, each `f(base, 1-ring neighbor base summary, collapsed parent)`.

## The coarse pregen pipeline

1. **Tectonics** (`pregen/tectonics.rs`): `clamp(cells/20, 3, 24)` Voronoi
   plates with velocity vectors; plate-crossing edges classified by relative
   normal velocity; effects (orogeny ±1400 m·m, arc/trench, rift/ridge,
   transform) spread over 2-ring falloff; provenance recorded per cell; ocean
   ring re-forced after uplift; shelves pinned at −30 m beside land.
2. **Climate** (`pregen/climate.rs`): latitude bands (temp = 31 − 0.52·lat −
   lapse), zonal winds (easterlies <30°, westerlies <60°, polar easterlies),
   per-row moisture march: saturate over ocean, precipitate ∝ uplift, deplete
   downwind. Rain shadow is asserted by test: mean precip directly downwind
   of >900 m terrain < mean directly upwind.
3. **Hydrology** (`pregen/hydrology.rs`): priority-flood depression filling
   seeded from the ocean, steepest-descent flow on *filled* elevations,
   discharge accumulated down the flow forest, river edges above threshold.
4. **History** (`pregen/history.rs`): site slots scored from
   precip/temp/river/coast, up to 240 (site ids must fit the overlay's u8
   `RegionId`); a statistical-tier **overlay world** built over the site
   graph via `ToyWorld::with_graph`; 12 epochs of founding, expansion, and
   conflict, where contested/dangerous sites get their hostile pressure
   **collapsed through `engine::observe`** — the sack of a site in epoch e is
   decided by an S2 collapse whose committed `RegionPressure` fact stays in
   the ledger forever. Fact kinds kept thin: `SiteExists` (founded/abandoned),
   `SitePolity`, `SiteEvent` (Founded/Sacked), `PolityExtent` per epoch, plus
   the observed pressures. A year-zero census observes every living site.

## Rivers reach the sea: proof summary

Priority-flood gives every land cell a filled elevation strictly above some
neighbor on a monotone path to the ocean; steepest-descent routing on filled
elevations therefore cannot cycle and cannot dead-end on land. The test walks
the flow pointer from **every land cell at all three extents** asserting
strict descent, ≤ n steps, and an ocean terminus; zero violations. (No
terminal basins were planned — filling drains everything to the sea; lakes
are marked where filling raised cells. Endorheic basins are a knob for later,
the proof obligation just gains "or terminal basin".)

## Lazy pyramid: bounded lookahead (measured)

`generate_chunk` traces distinct cells consulted per level and asserts them
against position-independent constants every call. Worst case observed across
the 10 000-chunk walk (cold columns) vs the asserted bounds:

| level | observed max | asserted bound |
|---|---|---|
| coarse cells | 36 | 96 |
| regions (recs + bases) | 16 | 40 |
| locales (recs + bases) | 9 | 24 |
| chunk-columns | 1 | 2 |
| lattice points (all levels) | 1 468 | 4 200 |

Constant regardless of extent, position, or distance from origin — including
across the civilized/wilds boundary and 280 km into the wilds.

## Stress + determinism (10 000-chunk walk, Medium)

- **No seams**: max surface-height step between adjacent voxel columns was
  **1 voxel**, both inside chunks and across every chunk/locale/region/cell
  border crossed (tolerance 6). A border seam would read as tens of voxels.
- **Wilds crossing**: pregen data ends at chunk ~4157; generation continues
  to chunk 10 000 (and provably further) on synthesized cells — abyssal
  floor below −50 voxels, no history layer touched.
- **Determinism**: chunks at walk offsets {0, 777, 4999, 9999} regenerate
  **byte-identically** from a fresh pregen + generator with the same seed
  (FNV over block ids); a different seed produces different terrain. Pregen
  itself replays identically (grid, sites, and ledger content hash equal).
- **Depth is unbounded**: a chunk at y = −4000 (−115 km) generates all-stone;
  y = +4000 all-air; both instantly, touching one column record.

## Ledger handoff (the thesis proof)

`tests/s7_handoff.rs`, all on the Medium world:

- Every pregen `RegionPressure` fact is reproduced with probability 1 by a
  live `engine::query` at the fact's tick — the S2 consistency property holds
  across year zero because there is nothing at year zero to cross: one
  overlay world, one ledger, ticks 0..=12 happen to be "history".
- Pregen facts **condition** play-time queries: at t = year zero + 3, the
  conditioned vs unconditioned distributions differ by TV up to the asserted
  threshold (>0.02) for committed regions.
- A **live observe** at t = year zero + 5 appends to the same ledger and is
  honored by subsequent queries with probability 1.
- The sacked site's causal chain is fully in the ledger — `Exists(true)` at
  founding, `Pressure(2)` at the sack epoch (committed by the pregen-era S2
  collapse that *caused* the sack), `Exists(false)` at abandonment strictly
  later — and the lazy layer renders it: ruin posts (Wood) stand in the
  generated chunks at the site's position. Committed history constrains both
  the statistical tier and the terrain.

## Pregen time vs extent (the size-knob labels)

| extent | cells | wall ms | facts | sites | polities | observes | approx bytes |
|---|---|---|---|---|---|---|---|
| small (~74 km) | 5×5 | 1.4 | 41 | 2 | 2 | 18 | 3 008 |
| medium (~251 km) | 17×17 | 16.1 | 137 | 11 | 2 | 86 | 23 408 |
| large (~1017 km) | 69×69 | 250.6 | 1 320 | 130 | 20 | 730 | 353 184 |

Scaling is ~linear in cell count (history observes dominate). The honest
conclusion: **this thin history pass costs milliseconds, so the "generating
world history…" ritual budget is entirely available for a thicker pass** —
the knob's labels should be driven by history richness (epochs × fact kinds ×
site cap), not by terrain. Memory is negligible (the ledger *is* the world
record; fluid state is derived).

## Chunk latency through the full pyramid

| path | mean ms | p95 ms | max ms |
|---|---|---|---|
| cold (new region/locale/column, fresh lattice ancestry) | 0.73 | 0.81 | 7.4 |
| warm (adjacent chunk / cached column) | 0.28 | 0.55 | 0.79 |

The 7.4 ms max is the first-ever chunk (empty caches to level 0). Well under
a frame budget either way; the walk sustained ~580 chunks/s including test
overhead.

## dc-sim changes made (additive, as permitted)

1. `statistical/ledger.rs`: new subjects `Site(u32)`, `Polity(u32)`; new
   aspects `SiteExists`, `SitePolity`, `SiteEvent`, `PolityExtent`; new
   values `Exists(bool)`, `PolityRef(u32)`, `Event(SiteEventKind)`,
   `Extent(u32)`; `SiteEventKind {Founded, Sacked, Abandoned}`. New `Value`
   variants use a 32-bit payload key layout; existing variants' hash shapes
   untouched (committed content hashes unchanged).
2. `statistical/world.rs`: `ToyWorld::with_graph(seed, adjacency,
   agent_home)` — arbitrary region graphs (≤255 regions, validated symmetric
   /irreflexive), agents optional; `num_regions()` / `num_agents()`
   accessors; `ball`/`diameter` use instance size instead of the toy
   constants. `ToyWorld::new` and all S2 behavior unchanged.
3. `statistical/engine.rs`: agent iteration uses `world.num_agents()`;
   worldgen subjects are ledger-only (`unreachable!` as query targets, fall
   through the existing scope filter as facts).

No changes to the collapse algorithm, conditioning, or ledger invariants;
all S2 tests pass unmodified.

## Failure modes and open questions

- **Rivers are cell-to-cell chords.** Straight 14.7 km segments carved with
  width ∝ √discharge; no meanders, no sub-cell course jitter, and the water
  surface interpolates linearly between cell fill levels. Fine for the spike;
  course refinement belongs at the region/locale level (same collapse rule,
  jittered control points) and needs the same 2-ring inclusion argument
  re-proved.
- **History is thin by scope.** 2 polities on Medium (founding count scales
  with site count) and no trade/migration/succession; sacked = abandoned
  forever. The machinery (overlay + observe + facts) is the deliverable; a
  thicker pass raises `observe_count` linearly, which the timing table says
  is affordable at ~0.3 ms/observe.
- **Site cap at 240** keeps site ids inside the overlay's u8 `RegionId`.
  Large worlds want thousands of sites → the statistical tier needs u16
  region ids or a sharded overlay (per-continent?) — known S2 open question
  (ledger scale) now with a concrete number attached.
- **Overlay graph ignores geography beyond adjacency.** Site danger is
  seed-random (`base_danger`), not derived from the map (wilds proximity,
  biome). Wiring terrain into the overlay's transition model is the obvious
  next step and needs no new machinery.
- **Climate is column-sampled.** Temp/precip are bilinear between cell
  centres but sampled once per chunk-column (28.8 m), so biome boundaries are
  column-quantized. Invisible at spike fidelity; per-voxel sampling is a
  4×-cell-lookup cost decision for later.
- **Cache eviction is crude** (clear-on-threshold; lattice keeps levels ≤ 9).
  Correct by purity, but a shipping generator wants LRU and a bound on the
  walk's ~45 MB column-cache high-water mark.
- **Terrain smoothness**: max 1-voxel steps between adjacent columns means
  no cliffs/gorges at voxel scale. The amplitude schedule (0.55 decay) is
  conservative; provenance-driven schedules (cliff bands, karst) are flavor
  work on the same lattice.
- **Savegame-stable worlds** (seed + knob → identical world across releases)
  remain policy, not code: every draw is addressed, but any tuning constant
  change reshapes the world. Version the generator params.
