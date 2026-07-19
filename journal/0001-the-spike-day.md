# 0001 — The spike day

*2026-07-18 · founding*

> blogworthy: eight architecture spikes in one day via parallel agents — and
> the two that produced novel results: the combinatorial cap that makes mixed-
> material voxels compressible (S8), and history-as-constraint-ledger with a
> proven no-seam handoff from worldgen to live simulation (S2+S7).

The repo was created, the architecture drafted, and seven of eight planned
spikes were designed, run, and merged in one day — each as a background agent
in an isolated worktree, reviewed and merged to main with gates green.

## What was decided (details in docs/)

- Bespoke Rust/Bevy stack over Unreal; five-crate headless/client split
  (ARCHITECTURE.md).
- Voxel scale **N=2** (0.9 m voxels), fidelity via sub-voxel shapes (S1).
- **32³ cube chunks**, 3D lattice, adaptive load volume, floating origin.
- Constraint ledger: fluid state derived from (seed, time, committed facts);
  bounded fact-aware collapse — qualified GO (S2).
- Chunk format v1: palette + versioned sidecars, 19× compression; octree LOD;
  column summaries; far-mesh to 1.2 km (S3).
- dc-api one-door command surface: three-consumer parity (native/wasm/MCP)
  proven, deny-by-default capabilities, replay determinism (S5, API.md).
- Materials: 8-eighths mixed voxels — **GO on free-form mixtures**; the
  combinatorial cap C(k+8,8)−1 makes gradient explosion impossible for small
  material sets (S8, design/materials.md).
- Visuals: "elevated pixel game", LabPBR packing, sim-driven porosity
  (design/visuals.md); **Forward+** pipeline, WGSL shader packs with naga
  validation + cross-backend CI, `dusk` example pack (S4, rendering/PIPELINE.md).
- Worldgen: bounded continent-disc world, pregenerated coarse deep-time
  (tectonics→climate→hydrology→history as ledger facts), lazy pyramid below,
  unbounded border wilds, extent as player knob. Year-zero ledger handoff
  proven — pregen history renders as ruins in chunks and constrains live
  queries (S7, design/worldgen.md). Pregen is ms-scale: the creation-ritual
  budget buys history richness, not terrain.

## Field notes

The user's first walk (S1 feel pass) settled the scale decision. Their second
walk caught two rendering defects no test had flagged: one-frame chunk flashes
(spawn-frame transform at the floating origin) and z-fighting between LOD
levels (grid-aligned seam bias ⇒ exactly coplanar faces). Both fixed same-day
(`64729c5`); both are in corrections.md as falsified claims. The lesson is
orogeny's: walks catch what metrics miss. Hence the observability push now in
flight.

## Process lessons

- Two parallel build-heavy agents saturated the machine and hung it ~15 min.
  Standing rule since: one cargo invocation at a time anywhere, `--jobs 4`,
  shared `CARGO_TARGET_DIR` for worktrees (see CLAUDE.md).
- Design conversations run in the gaps while spikes execute — API, materials,
  visuals, and worldgen were all designed "between" spikes.

## Status

- [x] S1 scale · S2 ledger · S3 storage/LOD · S4 shaders · S5 API · S7 worldgen · S8 materials
- [ ] S6 physics (in flight)
- [ ] client-through-dc-api + observability harness (screenshots, journals,
  in-client MCP) — next, queued behind S6
