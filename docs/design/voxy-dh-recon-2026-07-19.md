# Voxy / Distant Horizons recon — the re-derived thread

Status: research record, 2026-07-19. Re-derives the user's pre-repo
research session (lost unrecorded — the defer=write-it-now rule exists
because of it). Produced by a verified web-research pass (22 claims
survived 3-vote adversarial verification, 3 refuted; primary sources =
the mods' own repos/wikis + a peer-reviewed I3D 2025 paper). Feeds the
far-field roadmap: FF2a (voxel-stepped far field), FF2b (coarse
volumetric summaries), and the eventual GPU-driven far renderer.

## Distant Horizons — the conventional, durable design

- **LOD ingestion**: full chunks summarized into progressively
  lower-fidelity LOD data by distance.
- **Persistence is the load-bearing organ**: LODs live in an on-disk
  SQLite database (`DistantHorizons.sqlite`, per dimension; real files
  run 1–11 GB). Edits update the store, so **player changes appear in
  distant terrain and survive sessions** — the derived-cache
  architecture our far field needs (summaries beside S3 region files,
  updated on write).
- **Far field renders as a separate, skybox-style layer** decoupled from
  near geometry, explicitly to dodge depth-precision/z-fighting at
  range. Caveat from their own tracker: separation *mitigates*, boundary
  overlap still fights (reversed-z / 32-bit depth discussed). We already
  live this lesson (walk-2 radial push; near/far overlap sink).
- Reported character: smooth, low-stutter, CPU-friendly — the
  conservative end of the spectrum.

## Voxy — the GPU-driven champion (architecture from source)

No design doc exists; re-derived from `github.com/MCRcortex/voxy` (dev
branch — file names drift):

- **Same persistence spine as DH**: full-res chunks voxelized into a
  multi-level LOD format in a per-world database (`voxyserver/`,
  deletable to regenerate; can import DH worlds).
- **GPU-generated draw commands**: `shaders/lod/gl46/cmdgen.comp` writes
  indirect DrawCommand structs into a GPU buffer with atomic slot
  allocation, per section, translucents split out — zero per-section CPU
  draw calls.
- **Bit-packed quads + vertex pulling**: `quad_format.glsl` packs a quad
  into a **uint64**: XYZ 5+5+5 bits, W/H 4+4, face 3, stateID 16,
  biomeID 9, lightID 8 — decoded in the vertex shader; no CPU vertex
  buffers. (Mesh-shader backends gl46mesh/nvmesh exist alongside.)
- **Two-stage GPU visibility**: frustum test (`frustum.glsl`, AABB vs 6
  planes) + hierarchical-Z occlusion (`hiz/hiz.comp` builds a 6-level
  max-depth mip pyramid; a raster pass tests draw boxes against it).
- **Why it wins** (qualitative — no controlled benchmark survived
  verification): design orientation. Indirect/compute submission kills
  driver overhead and CPU draw cost. A verifier dissent worth keeping:
  "favors GPU-bound setups" may be backwards — less CPU overhead helps
  CPU-bound rigs most.

## Aokana (I3D 2025, peer-reviewed) — the volumetric end-state candidate

Not a mod; a research framework, directly relevant to **FF2b**:

- **SVDAG octree summaries** with a concrete recipe: eight LOD-N chunks
  aggregate into one same-resolution LOD-N+1 chunk; a summary voxel is
  created when non-empty-child count meets a density threshold, colored
  by child average. That is a usable coarse-volumetric summarization
  rule.
- **No meshing at all**: indirect-dispatch compute ray-marches the SVDAG
  per 8×8 screen tile over frustum+Hi-Z-culled chunks into a 64-bit
  visibility buffer (ESVO-style). Reported: ten-billion-voxel scenes at
  ~6 ms; up to 4.8× faster / 9× less memory than prior SOTA
  (self-reported).

## Transfer map to deepcraft

| Technique | Where it lands here |
|---|---|
| Persistent LOD store updated on edit (DH/Voxy) | The far-field edit-visibility design (Observed): summaries derive from the AUTHORITY, subscribe to the edit dirty-rail, persist beside S3 region files |
| Separate far layer + depth hygiene (DH) | Already our shape; keep reversed-z/32-bit depth in mind at PBR-2 |
| Bit-packed quads + vertex pulling (Voxy) | **FF2a**: stepped columns greedy-mesh into packed quads (~8 bytes/quad); our layer/weight splat data can ride the same packing |
| GPU indirect cmdgen + Hi-Z culling (Voxy) | The eventual GPU-driven far pass; wgpu exposes `MULTI_DRAW_INDIRECT_COUNT` (Vulkan 1.2+/DX12) + `INDIRECT_FIRST_INSTANCE`; Bevy's opaque multidraw path (PR #16427 line) is the substrate — as of late 2024 Vulkan-only, identical-material limit; re-verify current status |
| Persistently-mapped pooled vertex buffers (AZDO) | Far-tile buffer management when tiles churn |
| SVDAG + tile ray-march (Aokana) | **FF2b** candidate: mesh-free volumetric far field, paired with the caves/water thread |

## Open questions (carried from verification)

1. Voxy's invalidation mechanism (edit → which LOD levels re-mesh,
   CPU or GPU?) — the store is proven, the dirty-tracking isn't.
2. No controlled Voxy-vs-DH benchmark exists; the gap is architectural
   consensus, not a measurement.
3. Voxy's downsampling heuristic (state/color/light selection per LOD)
   is uncharacterized — Aokana's density-threshold recipe is the
   documented alternative.
4. Bevy 2026 status of multidraw limitations — measure before building
   the bespoke cmdgen path.

## Sources

Verified pass (primary): `github.com/MCRcortex/voxy` (+README),
`gitlab.com/distant-horizons-team/distant-horizons` (+wiki),
arxiv 2505.02017 (Aokana), `bevyengine/bevy` PR #16427, wgpu Features
docs, nickmcd.me high-performance-voxel-engine,
`github.com/omar-owis/VoxelEngine`, vkguide gpu-driven chapter;
secondary: grokipedia/modrinth/gurugamer/technosports (perf hearsay
only). Supplementary from the first (aborted) pass, unread by the
verified pass but flagged relevant: `github.com/MCRcortex/nvidium`
(same author's GPU-driven near-field renderer — likely shares the
architecture), bazhenovc GPU occlusion-culling slides, jms55 Bevy
virtual-geometry write-up, thenumb.at Exile voxel meshing,
`kurtkuehnert/terrain_renderer` (Rust/Bevy terrain LOD), DH issues
#403/#437 (LOD perf/edit behavior).
