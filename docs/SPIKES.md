# Spike backlog

Ordered by risk: the things most likely to invalidate architecture come first.
Each spike has an exit criterion — a decision recorded in ARCHITECTURE.md (or a
short ADR in docs/), not just "it works." Throwaway code is allowed; conclusions
are not throwaway.

## S1 — Voxel scale walking skeleton  `[risk: reshapes every budget]`

Bevy app, generated terrain, fly + walk around. Same world built at
player-height = 2, 3, and 4 voxels. Cubic chunks (3D chunk positions) and a
floating origin from the start — this skeleton is also the proof of the 3D
lattice (dig/fly down through many chunk layers; no column assumptions).

- Measure: chunk memory, mesh gen time, meshed triangle counts, save size per
  world-meter³ at each scale.
- Feel: reach, jump arc, stair/slab conventions, door/corridor sizes, how block
  models read at each scale.
- **Exit**: voxel:meter ratio chosen and recorded; perf budget table for the
  chosen scale; bevy version pinned. Adds `bevy` to dc-client.

## S2 — Constraint ledger prototype  `[risk: novel design, brutal to retrofit]`

Headless dc-sim prototype, no rendering. A toy world of ~100 NPC agents across
a region graph, statistical tier only.

- Implement: committed-fact ledger (append-only), fluid state as seeded pure
  function of (seed, time, constraints), collapse-to-depth-N with synthesized
  frontier boundary conditions.
- Torture tests: observe → collapse → verify consistency with all prior
  observations; repeated partial observations of the same region; collapse
  determinism under replay; cascade bounding (observing one agent touches ≤ N
  neighbors).
- **Exit**: written design for ledger schema + collapse algorithm; the failure
  modes we found; go/no-go on "one system for live sim and deep-time history."

## S3 — LOD-aware chunk storage  `[risk: v1 serialization lock-in]`

- Chunk format storing/deriving downsampled levels (octree-style pyramid —
  cubic chunks make heightmap-only LOD a non-starter — plus per-column
  summaries, which also serve as the lazy heightmap/sky-exposure cache);
  palette compression; what the statistical sim tier reads vs what the distant
  mesher reads.
- Skylight with unbounded depth is this spike's hard problem: light queries may
  only touch loaded chunks + column summaries, never "the column above."
- Prove the Distant-Horizons-style far render from LOD data only, near render
  from full data, seam handling at the boundary.
- **Exit**: chunk format v1 spec; measured size/derive-cost numbers at the S1
  scale; the far-mesh path renders in the S1 skeleton.

## S4 — Shader hook contract  `[risk: pipeline shape hard to change later]`

- Decide forward vs deferred (or forward+ hybrid); define the pass list and
  G-buffer layout if deferred; define what a shader pack may see and override
  (the Iris lesson: packs work because the pipeline shape is a stable
  contract).
- Prototype: one custom WGSL pack (fog/tonemap/water-ish) loaded at runtime in
  the S1 skeleton via naga validation, surviving on DX12 + Vulkan + Metal
  (CI-check translation even though dev is Windows-only).
- **Exit**: pipeline decision + hook-surface v0 spec; a loadable example pack
  in-tree.

## S5 — Plugin host + MCP over one surface  `[risk: the core architectural bet]`

- dc-api v0: a thin real slice — place/break block, query region, spawn entity,
  register a simple item from data. Serializable commands, capability grants.
- wasmtime host running a demo plugin (built from Rust to wasm32) driving that
  surface; in-process rmcp MCP server exposing the *same* commands; confirm an
  agent session and a plugin can't tell each other's writes apart.
- **Exit**: dc-api conventions doc (naming, versioning, capability model);
  demo plugin + MCP session transcript in-tree. Adds `wasmtime`, `rmcp`.

## S6 — Physics bubble  `[risk: medium — known-solvable, but touches core loops]`

- Rapier island around dynamic bodies: on-demand voxel colliders in a bubble,
  swept-AABB characters unchanged; dropped items as rigidbodies.
- Detach a built voxel prop into a compound-collider rigidbody and back.
- **Exit**: collider-bubble strategy note; perf numbers (bodies vs tick time)
  at S1 scale. Adds `rapier3d`.

## S7 — Worldgen: coarse pregen + lazy pyramid  `[risk: medium-high — two systems and their seam]`

Design in docs/design/worldgen.md (bounded world, pregenerated coarse
deep-time sim, lazy collapse below region scale, unbounded border wilds,
extent as a player knob).

- Coarse pregen pipeline slice: closed-surface tectonics → climate/
  hydrology (rivers reach the sea by construction) → a thin history pass
  writing committed facts into the S2 ledger.
- Lazy pyramid below: the one collapse rule (base + neighbor-summary +
  parent) region → locale → chunk; border-wilds path (no history layer).
- Stress: walk 10k+ chunks in one direction incl. across the civilized/wilds
  boundary; no unbounded lookahead; deterministic regeneration from seed.
- **Measure pregen time vs extent** (the player knob needs honest labels) at
  ≥3 world sizes.
- **Exit**: level/resolution spec per pyramid level; pregen-time table;
  demonstrated ledger handoff (a pregen-era fact visible to a live
  statistical-tier query); topology recommendation.

## S8 — Material volume model storage  `[risk: state-space explosion vs palette]`

Design in docs/design/materials.md (voxel as 8 material eighths: structure /
debris / pore occupancy, packing, derived stratification).

- Prototype the interned-mixture-table + sidecar storage over S3's format-v1
  container; simulate realistic deposition (wind-blown snow, rockfall, a
  midden, alluvial fan) on S1 terrain and MEASURE distinct-state counts per
  chunk — the palette-survivability question is empirical.
- Extraction prototype: typed damage vs per-material resistance ordering.
- LOD downsample rule for mixed voxels (S3 made the rule pluggable).
- **Exit**: go/no-go on free-form mixtures vs curated mixture recipes;
  measured bytes/m³ with debris present; sidecar schema v1.

## Deliberately not spiked yet

- Multiplayer/networking — dc-api's serializable commands are the future wire
  surface; no further work until the surface stabilizes (post-S5).
- In-game editor UX — depends on S1 (scale), S5 (api), content-pack format.
- Console ports — protected by architecture rules, not by work (see
  ARCHITECTURE.md § Platforms).

## Suggested order

S1 and S2 first, in parallel if desired (S2 is headless and shares no code with
S1). Then S3 (needs S1's scale), S5 (unblocks editors/plugins/MCP), S4, S6, S7.
