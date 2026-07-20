# 0023 — the voxel-language far field

*2026-07-19 · FF2a (background agent; worktree branch for the main session to
integrate). Seed 1337, N=2 worldgen authority. Replaces the smooth-TIN far
field of journal/0022. Numbers from the Windows dev box (RTX 3070, Vulkan),
release profile.*

## What changed, and why it was time

journal/0022 built the first horizon: a 2-D annulus of coarse-summary tiles,
meshed as a **smooth interpolated heightfield** (33×33 shared-vertex sheet,
smooth normals). It worked — it was a real skyline where before there was
sky — but it spoke the wrong language. Up close the world is voxels; on the
horizon it dissolved into a smooth TIN. visuals.md § *distance speaks the voxel
language* had already decided the far field should read as coarse voxels, and
the user promoted FF2a to do it. Two walk-filed defects rode along:

- **The buried far sheet.** The walk-17 hole fix (journal/0022) lapped each
  ring one tile *inward* to close sky holes. For L1 that slid the far sheet a
  full tile **under** the near field — from ~54 m out, sunk half a coarse voxel
  so the opaque near terrain hid it. But it was still *there*, inside solid
  ground: a user digging in that band hit a phantom floor.
- **Pixel cracks between tiles.** Actual gaps, distinct from the cosmetic
  one-sided-normal stitch lines — the classic smooth-heightfield T-junction
  where a finer ring's edge has vertices the coarser ring lacks.

The prediction going in (ROADMAP Observed): voxelizing plausibly cures the
crack class *inherently* — stepped columns are solid-sided prisms; adjacent
columns share face planes; cracks are a smooth-TIN disease. FF2a is the test of
that prediction, and it held.

## Step 0: does the engine's multidraw path actually engage for us?

The DECIDED substrate (voxy-dh-recon addendum) is Bevy 0.19's engine GPU-driven
path on the ONE shared terrain material — no bespoke cmdgen, no vertex pulling.
But whether Bevy's `gpu_preprocessing` multidraw *engages* for our **custom**
`Material` with **custom vertex attributes** (`Uint32x4` splat layers +
`Float32x4` weights) was unverified. Before building anything I measured it.

A render-app diagnostic (`DC_GPU_PROBE`, env-gated, zero cost when unset) reads
`GpuPreprocessingSupport::max_supported_mode` and, after the phase buffers are
collected, the `IndirectParametersBuffers` batch / batch-set counts for the
`Opaque3d` phase our terrain draws into. Live, on the dev box:

```
GPU preprocessing is fully supported on this device.
DC_GPU_PROBE: mode=Culling | Opaque3d indexed batches=74 sets=1
```

**Verdict: it engages, fully.** `mode=Culling` is the mode that enables GPU-built
indirect multidraw (below it, `PreprocessingOnly`, draws are direct). And
`batches=74 sets=1` is the strong result: all 74 terrain draws — near chunks
*and* far tiles, sharing one `TerrainMaterial` — merged into a **single
multidraw set** (`sets < batches` proves cross-mesh merging). Custom vertex
attributes do not disqualify batching; they only decide which `MeshAllocator`
slab a mesh packs into (by vertex `array_stride`), and ours all share one layout.
Backend is Vulkan, where `multi_draw_indirect_count` is available (Bevy 0.19
excludes DX12 for the `_count` variant — a fallback to plain `multi_draw_indirect`,
still GPU-built). Nothing bespoke needed; the DECIDED architecture is real on
this hardware. (Non-`Culling` fallbacks — GL, feature-poor devices — degrade to
GPU-preprocessed direct draws or CPU uniforms automatically; same `Mesh`, slower
submission, no code change.)

> blogworthy: "the batch count that ratified an architecture." One env var and a
> two-resource read turned a paper decision ("ride the engine's multidraw") into
> a measured fact — 74 draws, one multidraw set — before a single tile was
> re-meshed. Measure the substrate engages before you build on it.

## The floor-quantize that retired the sink

A far column samples `coarse_surface(vx, vz)` (unchanged from 0022 — the near
ground's own kernel at a coarse stride) and gets a surface height `h` in base
voxels. FF2a quantizes it to the level's coarse-voxel lattice by **flooring**:
`top = h.div_euclid(2^L) * 2^L`.

Flooring — not rounding — is the load-bearing choice. It makes the stepped top
**always ≤ the true surface**, hence ≤ the near-field surface at a coinciding
column. So the opaque near terrain wins the overlap band *by construction*, and
journal/0022's half-voxel downward "sink" — the fudge that used to keep the
smooth sheet from poking through — is simply deleted. At a stride-aligned column
the far top equals `floor(near_height / stride) * stride` exactly: quantized-exact
near/far parity, no bias hack. The only error is the sub-coarse relief dropped
below the step, bounded by one coarse voxel (proven in
`far_columns_quantize_the_near_ground_below_it`). The anti-z-fight radial depth
push (corrections #1) survives — overlapping LOD rings still present coplanar
faces where their quantized tops coincide, and only a real depth offset resolves
coplanar z-fight.

## The crack class died on its own

Each tile meshes as stepped prisms: greedy-merged top faces at the quantized
height, plus vertical side faces between neighbour columns of differing height.
The T-junction cure is a property of *how the side face is chosen*: a wall is
emitted **only by the taller column**, spanning down to the shorter neighbour's
top. Two adjacent columns evaluate the same shared step from opposite sides; only
one (the taller) sees "neighbour lower than me" and emits. So there is exactly
one wall per step — never a double wall, never a gap.

This extends across tile boundaries for free. A tile samples a one-cell
neighbour ring (an (N+2)² grid) beyond its 32×32 interior; a same-level
neighbour tile samples that identical shared world column, so the two tiles agree
on the boundary step to the voxel and only the taller side draws it. **Same-level
seams are watertight with no skirt.** The walk confirmed it: the grazing-angle
fullbright shot (`0023-fb-grazing-horizon`) shows far tiles receding as one
continuous stepped surface with no pixel cracks between them — the crack class is
**cured inherently**, exactly as predicted.

Only two edge kinds still get a modest downward skirt (2 coarse voxels): a
**ring-to-ring** boundary (a coarser neighbour tile tessellates the span with a
different stride — a genuine T-junction risk) and the **near/far coverage
boundary** (below). "Skirt only what remains" — and what remained was small.

> blogworthy: "the seam that meshed itself shut." The smooth heightfield needed
> stitching because two tessellations of one surface never agree per-pixel.
> Voxel columns don't tessellate a surface — they *are* the surface — so two
> tiles that sample the same boundary column produce the same prism face, and the
> crack has nowhere to open. Correctness fell out of speaking the right language.

## The buried sheet, replaced by coverage

The fix for "digging hits the LOD" is to stop putting far geometry under the near
field at all. FF2a culls a far column when the near volumetric field covers it:
`near_covers(column, viewer)` is true when the column's 3-D distance to the
viewer is under `FULL_DETAIL_RADIUS − FAR_OVERLAP` (112 m). Inside that, the near
field draws a full column and the far column is **omitted from the mesh**; a tile
entirely inside meshes to *nothing*. Redundancy across the seam is now the thin
occluded overlap band [112 m, 128 m] where the near field's ragged edge lives —
not a buried lap reaching to 54 m.

The cull is **altitude-aware** because it uses 3-D distance: fly to a 1 km vantage
and the vertical term grows, the covered disc shrinks to nothing, and the whole
horizon renders — so the high-vantage skyline is never eaten by a cull meant for
the ground. Because the cull is viewer-relative, seam tiles are refreshed in
place (no blink) when the viewer crosses into a new near-chunk; distant tiles
never have a cullable column and never rebuild.

The dig test is decisive: `world_fill` a pit straight down at the player's feet
(0 m out — far fully culled) and look in (`0023-fb-dig-no-phantom-floor`). The
pit shows only genuine near geology — grass rim, mixed-material granite walls, a
real granite floor at the bottom of the fill. **No phantom sheet at the surface
level, nothing hovering.** The headless proof is stronger still: a fully-covered
tile meshes to empty (`near_coverage_culls_far_columns_instead_of_burying_them`)
— there is no buried geometry to expose, by construction, not by occlusion.

## A pure mesher (so async and edit-tracking stay drop-in)

Mid-build the coordinator flagged three no-preclusion constraints from the open
LOD-store thread. They shaped the structure:

1. **The tile mesher is a pure function** of plain data — the extended
   `ColumnSpan` grid, a near-coverage mask, and four ring-edge flags. The impure
   derivation (sampling the authority, computing coverage from the viewer) lives
   in the streamer and passes the mesher plain inputs. No ECS/world read happens
   inside the mesher. Async far-meshing (a filed follow-on) becomes a drop-in:
   derive on the main thread, mesh on a task.
2. **Summaries stay per-tile re-derivable.** A tile's spans derive in isolation
   from `coarse_surface`; nothing caches summary immutability, so edit-driven
   invalidation (the decided persistent-LOD-store follow-on) is a re-derive, not
   a rewrite.
3. **The `ColumnSpan` payload is persistence-shaped.** Plain fixed-layout POD
   (`top: i32`, `block: Block`) — no `Option`, no `skip_serializing_if` — so a
   positional format (postcard, corrections #3) or the recon doc's 8-byte packed
   quad can serialize it verbatim when it eventually lives beside S3 region files.
   FF2b's volumetric summaries extend a column to a *stack* of these spans; the
   mesher already reasons in "a column is spans with tops and sides", so that
   extension needs no rewrite here.

## Perf and scale

Per-tile cost is close to 0022's: derivation + mesh ~1.87 ms/tile (vs ~1 ms —
the extra is the (N+2)² neighbour sampling and side faces), the full 1.2 km field
~0.35 s if done at once (vs ~0.22 s), budgeted 2 tiles/frame with no hitch. The
happy surprise: **greedy merging makes stepped tiles *cheaper* in geometry than
the smooth sheet** — worst-case average 698 tris/tile against the old fixed
2048/tile, because flat far ground collapses to a handful of merged quads.

Scale headroom checkpoint (`scale_headroom_checkpoint`, worst case = no cull):
the current 4-ring field is 188 tiles / ~21.5 MiB of mesh at 80 bytes/vertex.
Projected to the design target by adding doubling rings: ~5 km ≈ 329 tiles /
~38 MiB, ~10 km ≈ 376 tiles / ~43 MiB. Per-frame meshing stays ~3.7 ms (2 tiles)
regardless of radius — the budget bounds the frame, not the field — so a 10 km
field just fills in over ~3 s. Default radius unchanged (1.2 km); the numbers say
the headroom is there when the design wants it.

## The appearance change (for the user's eye)

**This milestone changes how the world looks: the smooth far horizon is now a
stepped voxel horizon.** That is intended (visuals.md § distance speaks the voxel
language), but the user ratifies looks from images:

- `0023-lit-stepped-horizon-ground` — lit, on the surface: near voxels roll into
  a stepped far horizon, continuous handoff, no seam.
- `0023-lit-high-vantage-rings` / `0023-fb-high-vantage-holes-check` — the LOD
  rings as a stepped, hazed skyline; **no sky holes** (walk-17 coverage theorem
  holds live), only faint diagonal stitch lines (the known cosmetic
  one-sided-normal seam, haze-hidden — deferred polish).
- `0023-fb-grazing-horizon` — grazing angle: far tiles continuous, **no pixel
  cracks**.
- `0023-fb-dig-no-phantom-floor` — dig test: real near geology, no phantom floor.

## What this resolves, and what it leaves

Resolved: the smooth-TIN far field (retired for stepped columns); the buried far
sheet (coverage cull, not buried lap); the pixel cracks between tiles (cured
inherently by stepped prisms; ring/near-seam skirts for the rest). Retained:
near/far parity (now quantized-exact), the coverage theorem, the streaming
budget, horizon presence at all vantages, S4 haze, fullbright + lit parity, the
one-shared-material constraint (both terrain materials still render the far mesh
via block atlas layers), depth hygiene (radial push re-argued for stepped
geometry).

Left filed (unchanged from 0022): the faint one-sided-normal stitch lines
(cosmetic, haze-hidden — a cross-tile normal halo is a larger change than this
milestone warranted); the far field is a top sheet, so looking up from deep in a
chasm still loses it (FF2b's volumetric spans own that — the `ColumnSpan` payload
is already stack-ready); far-field edits are still invisible (a summary, not a
cache); persisted summaries + edit dirty-rail remain the decided follow-on; far
meshing is still main-thread/budgeted (now a pure function, so async is a
drop-in).
