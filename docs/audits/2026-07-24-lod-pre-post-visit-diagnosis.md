# Far-field LOD "haunted" behaviour — pre/post-visit diagnosis

**Date:** 2026-07-24
**Mode:** DIAGNOSIS ONLY (read-only on code; no fix, no plan). Every claim below
carries `file:line`. A final section separates **PROVEN** (static from the code)
from **INFERRED / needs-live-probe** (runtime-streaming behaviour a static read
cannot pin).

Commit base: `main`. All paths under `crates/`.

---

## 0. TL;DR

Under the worldgen authority **all four far-LOD rings run through ONE code
path** (`stream_far_surface` → `tile_column_stacks` → `build_far_tile_mesh`).
There is no per-band subsystem split. The "haunting" is produced by **one
distance gate inside that single path**:

> `REDUCTION_STANDOFF_M = 176.0` (`dc-client/src/farmesh.rs:594`).
> Per far column, if the column is **within 176 m (3-D) of the viewer** it is
> rendered by **cold-synthesize only** (the reduced/warm data is *not even
> consulted*); **beyond 176 m** it is composed with **warm-reduce** data where a
> node's subtree is fully resident. (`farmesh.rs:792-799`.)

- **Obs #1 (first-gen dither/blobs)** = the cold-synthesize block is a
  **member/class dither drawn from a coherent bilinear field, point-sampled at
  the coarse stride** — adjacent coarse cells land on different members/classes
  → blobs of different texture.
- **Obs #2 (post-visit becomes faithful, not identical)** = visiting streams
  **real per-voxel `ContentsGrid`** into the `FarPyramid`; warm-reduce classifies
  the **dominant *subsurface* material** of resident voxels, which tracks the
  actual strata. The two derivations are only held to a **statistical** agreement
  (spines **S-9** consistency-law violation), so they differ.
- **Obs #3 (band inversion)** = the 176 m cut falls **inside the L1 ring
  (112–256 m)**. L1's inner sub-band is < 176 m → cold dither; L2/L3/L4 are all
  ≥ 256 m → warm-faithful (where warm data is resident). Finest band cold,
  coarser bands faithful — exactly the reported inversion.

The user's read — *"multiple periods of work contradicting each other"* — is
**confirmed**: three layered eras collide (FF2a coarse-surface synthesis,
journal/0022-23; FF2b volumetric span reduce + mixture-LOD, journal/0070/0055;
the member/class dither, journal/0058/0073/0074), joined by the block↔material
collapse (journal/0087). See § 6.

---

## 1. The band / ring scheme, and which path feeds each band

### 1.1 Two mutually-exclusive far pipelines, selected by authority

| Pipeline | System | Active when | Feeds |
|---|---|---|---|
| **Worldgen far surface** (FF2a→FF2b) | `stream_far_surface` (`farmesh.rs:1183`) | `authority.far_field_is_worldgen()` — the **boot / default** authority | all four rings the user flies |
| **Legacy S1 volumetric far mesh** | `stream_far_chunks` (`farmesh.rs:367`) | S1 authority only (keys 3/4) | inactive in the user's scenario; torn down under worldgen (`farmesh.rs:384-393`) — it is the "phantom old world" path (`farmesh.rs:88-100`) |

The S1 path (`FarChunkMap`, `FarFieldTerrain`) is **not** in the user's picture:
it despawns itself the moment the worldgen authority is active
(`farmesh.rs:384-393`). Everything below is the **worldgen far-surface** path.

### 1.2 The rings (worldgen path)

Ring geometry is `HorizonConfig` (`farmesh.rs:158-192`). Defaults, pinned by
`default_ring_edges_match_the_shipped_constants` (`farmesh.rs:1562-1576`):

- `FULL_DETAIL_RADIUS_M = 128` (`farmesh.rs:110`); near field streams a 128 m
  sphere (`streaming.rs:41`), unloads at `UNLOAD_RADIUS_M = 160` (`streaming.rs:44`).
- `ring_edges = [112, 256, 512, 1024, 1200]` (`farmesh.rs:190`, `:1567`).
  Level L covers centre-distance `[ring_edges[L-1], ring_edges[L])`
  (`level_for_distance`, `farmesh.rs:291-296`).

| Band | Level | Centre-distance ring | Coarse voxel (base N=2) | Tile edge |
|---|---|---|---|---|
| near full-detail | — | 0 – 112 m (`near_cover_r_m`, `farmesh.rs:222`) | real chunks | — |
| finest far | **L1** | 112 – 256 m | 0.9 m | ~57.6 m |
| | **L2** | 256 – 512 m | 1.8 m | ~115 m |
| | **L3** | 512 – 1024 m | 3.6 m | ~230 m |
| coarsest far | **L4** | 1024 – 1200 m (stretched by `--horizon`) | 14.4 m (`farmesh.rs:1607`) | ~461 m |

Each ring is a 2-D annulus of 32×32-column tiles
(`wanted_far_tiles`, `farmesh.rs:686`; `far_tile_in_ring`, `farmesh.rs:678`).

### 1.3 The single derivation, per tile

Every tile of every level is derived identically (`stream_far_surface` loops
`for level in 1..=4` and calls the same `spawn_tile`, `farmesh.rs:1254`, `:1405`,
`:1339`). The derivation is `tile_column_stacks` (`farmesh.rs:762`) which, per
column, chooses between the two paths **by distance** (see § 2). The per-band
difference the user sees is **not** a different subsystem per band — it is where
the 176 m distance cut lands relative to each ring's inner edge.

---

## 2. Per-state path table

The two derivations of a far column (the S-9 "same base, two producers"):

- **cold-synthesize** — `sample(wx,wz) = generator.coarse_surface(wx,wz)`
  (`farmesh.rs:1358`), wrapped as a single unbounded span carrying that surface
  block (`farmesh.rs:783-787`). Node form of the same call:
  `synthesize_far_node` (`dc-worldgen/src/far.rs:53-77`) — "one span carrying the
  *surface* block all the way down" (`far.rs:18-25`).
- **warm-reduce** — reduced spans from the `FarPyramid`, block =
  `classify(dominant reduced contents)` (`dc-core/src/farfield.rs:137-156` →
  `classify.rs:97`), fed by real streamed voxels (`streaming.rs:153`).

The switch (`tile_column_stacks`, `farmesh.rs:792-799`):

```
beyond_standoff = |viewer→column|² ≥ REDUCTION_STANDOFF_M²      (176 m)
stacks = if beyond_standoff { compose_column(synth, known(wx,wz)) }
         else               { vec![synth] }        // cold only; known() not called
```

`known(wx,wz)` reads a **frame-thread snapshot** of the `FarPyramid`'s
fully-inserted reduced node grids (`farmesh.rs:1314-1322`, `:1362-1378`), which
resolves to `FarPyramid::known_node_grids` (`farpyramid.rs:174`). It returns
**only nodes whose full-res subtree is completely inserted**
(`subtree_fully_inserted`, `farpyramid.rs:191`) — the A-5 guard; a partial
subtree returns empty → "synthesize", never "air".

| State | Column < 176 m of viewer | Column ≥ 176 m, node subtree resident | Column ≥ 176 m, node NOT resident |
|---|---|---|---|
| **Never-visited** | cold-synth (dither) | (no node ever resident) → cold-synth | cold-synth (dither) |
| **Visited-then-departed** (warm nodes still in `FarPyramid`) | **cold-synth (dither)** — reduction not consulted | **warm-reduce (faithful)** | cold-synth |
| **Approaching a visited region** | **cold-synth (dither)** ← finest band re-reverts | **warm-reduce (faithful)** ← coarser bands | cold-synth |

The final "not resident" column includes both (a) regions never walked densely
enough to fully insert a node, and (b) regions whose L0 chunks were evicted from
the `FarPyramid` by the distance budget (`enforce_budget`, `farpyramid.rs:129`;
`FAR_PYRAMID_L0_BUDGET = 8192`, `farpyramid.rs:44`; invoked each frame at
`streaming.rs:233`).

Note the near full-detail sphere (< 112 m) is **culled**, not drawn, by
`near_covers` (`farmesh.rs:602-608`, `tile_column_stacks:791`) — so the cold
dither the user sees "just before the real chunks" is the L1 annulus in the
112–176 m shell, not buried geometry.

---

## 3. Observation #1 — the first-gen dither / "blobs of different texture"

**Cause: the cold-synthesize surface block is a coherent-field dither,
point-sampled at the coarse stride.**

`coarse_surface` (`collapse.rs:1011-1019`) returns `(h, block)` from
`surface_sample`. The block is chosen by the **member dither**
(`collapse.rs:837-858`):

- A **content class** is drawn (`surface_class`, `collapse.rs:873+`) from a
  bilinear corner-hash field `interp_select_draw`
  (`geology.rs:198-213`) at salt `SALT_GEO_SELECT`, read at the voxel's own
  fractional position `(fx,fz)` within its 32-voxel chunk cell
  (`collapse.rs:842-846`).
- The **member within** that class is then dithered from the same field at a
  distinct salt (`collapse.rs:845-846`), and the block is that member's own
  material (`collapse.rs:847-849`): `Block::Material(member.material)` since the
  block↔material collapse (journal/0087).

Why this reads as **blobs** in the far field: the field is *continuous and
coherent over a chunk* (32 base voxels), but `select` maps its continuous value
to a **discrete** member/class by cumulative metre-shares — creating
sub-chunk patches with hard boundaries. The far tile point-samples this field
**once per coarse voxel** at stride `level_stride(level)` (`farmesh.rs:779-781`,
`:1359`) — 2 base voxels at L1 up to 16 at L4. Adjacent coarse cells therefore
land in *different* discretisation patches → a patchwork of different member
albedos / different classes. The surface-class docs name this exactly: "The far
field point-samples this class through `coarse_surface` at a wide stride"
(`collapse.rs:896-899`), and across a deep-cell boundary two cells "each vote a
different plurality winner" (`collapse.rs:886-891`). That patchwork **is** the
"weird dithering, blobs of different texture areas."

It is not a bug in the dither — the dither is deliberately coherent to *avoid*
speckle (`collapse.rs:893-899`). The blobiness is that the cold path shows **only
the dithered surface skin**, with no subsurface context to make the patches read
as real strata.

---

## 4. Observation #2 — post-visit becomes faithful (but not identical)

**Cause: visiting creates warm resident voxel data whose reduction classifies
the dominant SUBSURFACE material — a different producer than the cold surface
dither.**

What visiting creates: `stream_chunks` clones each near chunk from the authority
and calls `far_pyramid.insert_l0(pos, &chunk, contents)` (`streaming.rs:151-154`)
with the **real per-voxel `ContentsGrid`** (`authority.chunk_contents`,
`streaming.rs:142-145`). `insert_l0` (`farpyramid.rs:91-115`):

- inserts the block chunk into the `MajorityNonAir` block pyramid
  (`farpyramid.rs:92`), and
- interns every non-empty voxel's contents into a render-local `MixtureTable`
  and stores a level-0 `MaterialChunk` (`farpyramid.rs:94-108`).

Higher levels are lazily reduced by **`DominantClassDebrisAware`**
(`ensure_material`, `farpyramid.rs:223-249`; rule in
`dc-core/src/materials/lod.rs:53-96`), and the span's block is
`classify` of the reduced contents' **dominant material**
(`farfield.rs:137-156`; `dominant_material` prefers structure → debris → pore,
`classify.rs:65-89`).

Why it is **more faithful**: the reduction sees the **actual deposited strata of
resident voxels**, subsurface included. The ROADMAP Observed entry states the
mechanism precisely: "the user watched mudstone coarse boxes gain **granite**
patches on the second gen — **the sub-surface layer falls into the sample on
warm-reduce but not on cold-synthesize**" (`ROADMAP.md:2540-2547`). The
`DominantClassDebrisAware` cell vote picks a subsurface material where it
dominates the 2×2×2 cell — so warm boxes are "textured in approximate places of
the actual materials."

Why it is **not identical** to the cold answer (the S-9 violation): the two
producers are different functions of the same base:

- cold top block = the surface **class/member dither** draw
  (`collapse.rs:845-849`);
- warm top block = `classify` of the **majority subsurface material** over the
  reduced cell (`farfield.rs:149`).

They are only held to a **statistical** agreement — `far.rs:165`'s node-vs-reduce
test checks **surface tops / occupancy only**, not material identity (see its
tolerances, `far.rs:144-163`), and `coarse_surface_agrees_with_the_near_column_surface`
(`collapse.rs:1938-2018`) compares at **content-class** granularity and *expects*
near and far to "dither different members within a class"
(`collapse.rs:1942-1948`). The consistency law that this breaks is named in
spines **S-9**: "two derivations of the same base **disagree**, exactly what the
consistency law forbids" (`docs/spines.md:300-304`). So post-visit is *more*
faithful (real strata) but *cannot* equal the cold answer — the two eras encode
material identity differently.

---

## 5. Observation #3 — THE BAND INVERSION (the crux)

**Cause: `REDUCTION_STANDOFF_M = 176 m` is a per-column cold/warm gate, and 176 m
lands INSIDE the L1 ring (112–256 m). The finest band's inner shell is forced
back to cold-synth; every coarser band is entirely beyond 176 m and stays warm.**

The gate, again (`farmesh.rs:792-799`): a column within 176 m of the viewer is
`vec![synth]` — **cold only, `known()` not called**. Its stated rationale
(`farmesh.rs:584-594`): inside the near field's draw radius a reduced coarse top
can legitimately sit up to one coarse voxel *above* the true surface
(`MajorityNonAir` rounds to nearest while synthesis floors), so it could poke
through the near ground; within the standoff only floor-quantized synthesis is
allowed. `reduction_standoff_keeps_near_columns_synthesized`
(`farmesh.rs:1893-1929`) proves the gate: "no reduction consulted inside the
standoff."

Now place 176 m against the ring edges (`[112, 256, 512, 1024, 1200]`):

- **L1 ring = 112–256 m.** Its inner shell **112 → 176 m is inside the
  standoff → cold-synth dither**; its outer shell 176 → 256 m is warm. So the
  **finest visible band shows the first-generation dither** on the side nearest
  the player.
- **L2 (256–512), L3 (512–1024), L4 (1024–1200) are entirely ≥ 256 m > 176 m →
  every column is beyond the standoff → warm-reduce** wherever the node subtree
  is resident (§ 2). So **the 2nd band and everything beyond stay faithful.**

That is the reported inversion, and it is **static / deterministic** given the
constants — not an eviction race, not a per-band subsystem difference.

**Adjudicating the brief's three hypotheses:**

1. *"finest band re-synthesizes cold on approach while coarser bands hold a warm
   cache"* — **CONFIRMED as the mechanism**, but the trigger is the **176 m
   standoff**, not a cache-lifetime difference. The warm `FarPyramid` nodes for
   the coarser bands are the same store the finest band *also* has access to; the
   finest band is denied them by the distance gate, not by eviction.
2. *"different bands owned by different subsystems (FF2a vs FF2b vs mixture-LOD)
   with different cache lifetimes"* — **REJECTED for the worldgen path.** All
   four rings run one path (`stream_far_surface`); FF2a is folded into the FF2b
   span form (journal/0070; `farmesh.rs:575-582`). The FF2a-vs-FF2b-vs-mixture
   layering is a *within-column* composition (`compose_column`, `farfield.rs:170`),
   not a per-ring ownership split.
3. *"eviction/streaming ordering regenerates the near band from cold data"* —
   **NOT the primary cause.** Eviction (`enforce_budget`) governs whether the
   *coarser* bands have warm data **at all**; it does not explain why the finest
   band is cold while coarser bands are warm. The standoff explains that directly.

**Why approaching (not just visiting) re-exposes it:** as the viewer moves, tiles
in the cull band re-derive. `stream_far_surface` marks a tile **stale** when its
`cull_chunk` no longer matches the viewer's near-chunk (`farmesh.rs:1271-1285`),
where "in the cull band" is `tile_in_cull_band` =
`dist < max(near_cover_r, REDUCTION_STANDOFF) + tile_m` (`farmesh.rs:1164-1174`).
So tiles crossing the 176 m boundary are rebuilt and their columns re-evaluate
the standoff: a column that was warm (viewer far) turns **cold** as the viewer
closes inside 176 m. The finest band literally re-synthesizes cold on approach
while the tile is refreshed — the "haunted" transition the user watched.

---

## 6. The colliding "periods of work"

The single far column is assembled from **four different eras' encodings of
material identity**, and they meet in `compose_column` / `tile_column_stacks`:

| Era | What it contributes | Where |
|---|---|---|
| **FF2a coarse-surface synthesis** (journal/0022-23) | the top-sheet synth span: surface height floor-quantized + surface **block** = member/class dither | `far.rs`, `coarse_surface` `collapse.rs:1011`, `quantize_top` `farfield.rs:78` |
| **Member/class dither** (journal/0058, 0073; **only feeds far** since 0074) | the cold block is a *dithered member of a class*, coherent field, coarse-sampled → blobs | `collapse.rs:837-858`, `surface_class` `collapse.rs:873+` |
| **FF2b volumetric reduce + mixture-LOD** (journal/0070, 0055) | the warm span: `MajorityNonAir` block pyramid + `DominantClassDebrisAware` material pyramid → `classify(dominant subsurface)` | `farpyramid.rs`, `materials/lod.rs`, `farfield.rs:137` |
| **block↔material collapse** (journal/0087) | makes both blocks `Block::Material(id)` so the disagreement is now *visible as different textures* rather than hidden behind a coarse block tier | `classify.rs:20-29`, `collapse.rs:830-836` |

**Where the assumptions collide:**

1. **Cold vs warm pick a different winning material for the same box** —
   surface-dither (cold) vs dominant-subsurface (warm). This is the S-9
   consistency-law violation named in `docs/spines.md:300-304` and
   `ROADMAP.md:2538-2553`. The agreement tests were written to only guarantee
   **tops/occupancy** (`far.rs:165`) and **class-granularity** surface agreement
   (`collapse.rs:1938`) — neither guarantees material identity, so the divergence
   passes all gates.
2. **The 176 m standoff assumes cold-synth is the *safe* fallback** (it floors,
   so it can't poke through the near ground — `farmesh.rs:584-594`). That safety
   assumption was made for **geometry** (top height), but since journal/0087 the
   *block/material* also flips at the standoff boundary — so a geometry-motivated
   gate now produces a **visible material discontinuity** (the band inversion).
   The standoff and the block↔material collapse were designed in different periods
   and never reconciled.
3. **`REDUCTION_STANDOFF_M` (176) and the L1 ring inner edge (112) were set
   independently.** Nothing aligns the standoff to a ring boundary, so it cuts the
   L1 band in half — an unintended interaction between the streaming-radius
   constant (`streaming.rs:44`, 160 + slack) and the ring ladder
   (`farmesh.rs:121`).

---

## 7. PROVEN vs INFERRED / needs-live-probe

### PROVEN (static, from the code)

- The worldgen far field is one path over four rings; the S1 path is inactive
  under the worldgen authority. `farmesh.rs:384-393`, `:1183-1205`.
- The cold/warm switch is the per-column `REDUCTION_STANDOFF_M = 176 m` gate;
  inside it the reduced data is **not consulted**. `farmesh.rs:594`, `:792-799`;
  test `farmesh.rs:1893-1929`.
- 176 m falls inside the L1 ring (112–256) and below all of L2+ (≥256), so the
  finest band's inner shell is cold and coarser bands are warm — the inversion.
  Ring edges `farmesh.rs:190`, `:1567`; `level_for_distance` `farmesh.rs:291`.
- Cold block = coarse-surface member/class dither, coarse-stride point-sampled
  (blobs). `collapse.rs:837-858`, `:873-899`; `geology.rs:198-213`;
  `farmesh.rs:779-787`.
- Warm block = `classify` of `DominantClassDebrisAware` reduction of real
  resident voxels (subsurface included). `streaming.rs:151-154`;
  `farpyramid.rs:91-108`, `:223-249`; `farfield.rs:137-156`; `classify.rs:65-102`.
- The two derivations disagree on material identity and only pass tops/occupancy
  and class-granularity agreement tests — the S-9 violation. `far.rs:144-165`;
  `collapse.rs:1938-2018`; `docs/spines.md:300-304`; `ROADMAP.md:2538-2553`.
- Warm data is available only where `subtree_fully_inserted` (A-5 guard);
  otherwise the column falls back to cold. `farpyramid.rs:174-199`.
- Tiles re-derive (and re-evaluate the standoff) on viewer near-chunk crossings
  in the cull band. `farmesh.rs:1164-1174`, `:1271-1285`.

### INFERRED / needs-live-probe

- **Whether a departed region's coarser bands are still warm on re-approach.**
  This depends on whether its L0 chunks survived `enforce_budget`
  (`FAR_PYRAMID_L0_BUDGET = 8192`, distance-ranked eviction —
  `farpyramid.rs:129-160`, `streaming.rs:233`). If evicted, the coarser bands
  fall back to cold too, and the inversion would *not* appear. The user reports
  the coarser bands *are* faithful, so the region's nodes were **still resident**
  at observation time — consistent with the code but not provable statically. A
  live probe should log `FarPyramid::l0_len()` and, for the observed node
  columns, whether `subtree_fully_inserted` holds, at the moment of observation.
- **Which LOD levels actually achieved fully-inserted nodes for the visited
  region.** An L1 node needs 8 resident L0 chunks; L2 needs 64; L3 needs 512.
  A fly-through at speed may fully insert only shallow levels. So "coarser bands
  faithful" may hold for L2 but degrade at L3/L4 depending on how densely the
  128 m near sphere (`streaming.rs:41`) actually covered those nodes. Probe:
  count `subtree_fully_inserted` per level over the region.
- **The exact visual width of the cold L1 shell.** 112→176 m is the nominal
  cold shell, but per-column 3-D distance plus `near_covers` culling (< 112 m)
  and tile granularity make the visible cold annulus's precise extent
  pose-dependent. Probe: capture the L1 tiles' `culled`/`beyond_standoff` masks
  for a fixed pose.
- **Timing/latency of the cold re-synthesis on approach** (does it blink, and how
  many frames it lags). This is the async task-drain lag (`drain_far_meshes`,
  `farmesh.rs:1458`), not statically determinable. Probe: watch tile rebuild
  cadence vs viewer speed.

*(Per A-5: none of the runtime-dependent items above are asserted as fact; they
are the questions a live probe must answer to complete the picture. The three
observations' core mechanisms are in the PROVEN set.)*
