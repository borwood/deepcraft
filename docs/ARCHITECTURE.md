# deepcraft architecture

Status: pre-spike sketch, 2026-07-18. Everything here is a working hypothesis
until the spike named next to it lands (see SPIKES.md).

## Shape of the system

```
                ┌─────────────────────────────────────────────┐
                │                 dc-client                   │
                │   window · renderer (Bevy/wgpu) · input     │
                │   the ONLY crate that knows GPU/OS          │
                └──────────────────────┬──────────────────────┘
                                       │
 WASM plugins ──┐                      │                    ┌── in-game editors
 (wasmtime)     │                      │                    │   (items, blocks,
                ▼                      ▼                    ▼    blueprints,
                ┌─────────────────────────────────────────────┐  models+anims)
                │                  dc-api                     │
                │   serializable commands/queries,            │
                │   capability-scoped grants                  │
                └──────────┬──────────────────────┬───────────┘
    MCP (rmcp, ────────────┘                      │
    in-process)                                   ▼
                ┌─────────────────────────────────────────────┐
                │          dc-sim        ·      dc-worldgen   │
                │   tiered simulation,   ·  hierarchical lazy │
                │   constraint ledger    ·  gen + deep-time   │
                └──────────────────────┬──────────────────────┘
                                       ▼
                ┌─────────────────────────────────────────────┐
                │                  dc-core                    │
                │   voxel model · LOD-aware chunks · math     │
                └─────────────────────────────────────────────┘
```

The load-bearing bet: **one API, many consumers.** Scripts, MCP agents, and the
in-game editors all speak dc-api. Anything an editor can author, an agent can
author. Vanilla content is just the first content pack going through the same
door.

Everything below dc-client is headless and deterministic: deep-time history
runs in CI, the sim is testable without a GPU, and multiplayer (later) reuses
the same serializable command surface.

## Platforms

- First-class from day one: Windows (dev machine), macOS, Linux. CI checks all
  three on every push even though development happens on Windows.
- Consoles: someday, via porting house. There are no public Rust console
  toolchains and no public wgpu console backends, so we don't build for them —
  we just refuse to preclude them: platform/windowing/input stays inside
  dc-client, no native graphics APIs outside the wgpu abstraction, no OS
  assumptions in headless crates.

## Rendering (decisions owed early, see S3/S4)

- **Distant terrain is first-class, not a retrofit.** Chunk storage in dc-core
  is LOD-aware from v1: chunks can store/derive downsampled pyramids or
  per-column summaries; the mesher targets multiple LOD levels. The statistical
  sim tier reads the same summaries.
- **Custom shaders are a contract — DECIDED by S4 (2026-07-18): clustered
  forward (Forward+), deferred rejected.** Culled voxel meshes have near-zero
  opaque overdraw (deferred's advantage evaporates); Bevy 0.19 clusters
  colored point lights; POM/LabPBR/splat-blending want a forward surface
  shader, not G-buffer channels. Pack overridability comes from named stages
  with stable inputs (the Iris lesson), not pipeline shape: hook surface v0
  in docs/rendering/PIPELINE.md (`post` live, others reserved), packs are
  WGSL validated via naga with per-stage fallback to the default pack, and
  cross-backend translation (HLSL/SPIR-V/MSL) is CI-tested. Art direction in
  docs/design/visuals.md; the default look is itself a pack.

## World structure: cubic chunks (3D lattice)

Chunks have 3D positions — a lattice of cubes, not a 2D lattice of full-height
columns (Minecraft loads strictly by column; true 3D addressing is what the
Cubic Chunks mod existed for). Traveling down loads deeper chunks the same way
traveling north loads farther ones. Vastness of scale — especially depth — is a
design pillar. Consequences we accept and design for from v1:

- **Skylight/heightmaps are no longer local.** "Under open sky" can't be
  answered inside one chunk. Per-column summaries (already required for distant
  LOD and the statistical sim tier) double as lazy heightmap / sky-exposure
  caches; nothing may assume a bounded world height.
- **LOD is a 3D lattice too** (octree-style pyramid, not heightmap LOD):
  looking down a megachasm needs coarse chunks *below* the player.
- **Floating origin from day one**: f64 world coordinates in sim, camera-
  relative f32 on the GPU. Cheap now, a rewrite later.
- **Depth is a worldgen axis**: deep-time geological history generates literal
  strata; caves/aquifers/lava at region scale, not per-column noise hacks.

**Chunk shape — DECIDED 2026-07-18: chunks stay 32³ cubes.** The "tall chunks"
instinct (keep what's below loaded when overlooking a chasm; don't simulate the
surface when a kilometer deep) is implemented as *policy over cubes*, not as
chunk geometry:

- **Adaptive load volume**: base sphere around the player plus a downward
  (or any-direction) extension wherever column summaries report open air —
  chasms stream downward; solid plains don't drag bedrock columns into memory.
  Cubes keep remesh granularity (one edit = one 32³ remesh), frustum culling,
  and the S3 octree uniform; post-S3 palette compression makes all-stone/all-air
  cubes nearly free, removing tall chunks' only real advantage.
- **Sim tiers are 3D volumes**: full-sim is a bubble, so a deep-earth society
  800 m below the player sits in the coarse/statistical tier despite horizontal
  distance zero — same machinery as a village beyond the mountains. Deep
  cultures accumulate ledger history without ever being fully simulated until
  approached (or observed diegetically).

## Voxel scale

**DECIDED 2026-07-18 (S1 feel pass): player height = 2 voxels, voxel = 0.9 m.**
Rationale: S1 measured the 2→4 blowup at exactly 8× memory / ~4× triangles for
the same world volume; N=2 is the cheapest tier and the fidelity gap is closed
by **sub-voxel block shapes** (half/quarter blocks, stairs, etc.) — block
states resolved at meshing/collision time, not a finer world grid. The grid
stays 0.9 m everywhere; `VoxelScale` remains the single source of truth.
S1 budget baseline at N=2: 2.74 B/m³ raw chunks, ~31 tris/m² culled-meshed
(pre-palette-compression, pre-greedy-meshing — both improve from here).

## Physics

- Characters and simple movers: swept-AABB vs the voxel grid, owned by us
  (game feel lives here). Not a physics engine problem.
- Dynamic tier (Rapier, S6): dropped items as real rigidbodies, detachable
  voxel props, ragdolls. The voxel world is never a global collider set —
  colliders are generated on demand in bubbles around dynamic bodies.
  Detaching a built prop needs a voxel-cluster → compound-collider step.

## Simulation: tiers + constraint ledger (the novel part, S2)

Whole world stays "alive" through stepped abstraction by observer proximity:

| Tier | What runs | Cadence |
|---|---|---|
| Full | entity-level ECS sim | every tick |
| Coarse | agent-level goals, settlement jobs | slow ticks |
| Statistical | distributions over possible states | derived on demand |

Vocabulary (precise on purpose — this is deferred resolution under observation
constraints, not the WFC texture algorithm):

- **Committed fact**: any observation that has leaked to an observer (traveler's
  report, letter, psionic glimpse, visiting in person). Appended to the
  **constraint ledger**, immutable forever.
- **Fluid state**: superposed, resamplable. Evolves as a seeded pure function
  of (region seed, time, committed constraints) — derived, not stored; replays
  identically.
- **Collapse**: promoting fluid → concrete by sampling a history consistent
  with *all* committed facts. Bounded to depth N; at the frontier we synthesize
  plausible boundary conditions from the statistical tier instead of recursing
  (observing one mind must not collapse the planet).

The committed/fluid distinction exists in the sim from v1 — it is brutal to
retrofit.

## Worldgen: hierarchical, lazy, bounded (S7)

"Theoretically infinite" and "globally aware" coexist through hierarchy with
bounded neighborhoods: each level (continent graph → region → chunk) is lazily
generated, and a level may depend only on a bounded neighborhood of the level
above. Rivers are planned at region-graph scale before any chunk materializes;
trade routes at faction-graph scale. No level needs unbounded lookahead.

Deep-time history (geological, then social/territorial) = dc-sim's coarse and
statistical tiers run over pre-player time. Worldgen history and live
far-simulation are one system, not two.

## One world-answer surface (2026-07-19, **NEEDS RATIFICATION**)

Drafted by the S1-fallback sweep (journal/0017); the user ratifies doctrine.

**The client has exactly one authority for "what is the world here?" — the
active `Authority` (the embedded `HostWorld`).** Every gameplay question about
world state — is this voxel solid, what is the surface height, what does a
raycast hit — is answered by the authority, which generates lazily and includes
edits. There is no second world.

**Caches never fall back to a generator they do not own.** The client `ChunkMap`
is a render/collision *cache* of the authority: it answers only for chunks it
holds and returns an explicit miss otherwise. It must never invent an answer for
an absent chunk by sampling a generator — because the generator it would reach
for may describe a *different world* than the active authority. This is exactly
what shipped the legacy S1 terrain (~8 m surface) into player collision,
grounding, edit targeting, physics, and mesh-border culling under the worldgen
authority (~1000 m surface): a silent wrong-world fallback at every streaming
edge (journal/0015–0017).

The rule is structural, not vigilance: the fallback method is deleted, and the
ambient wrong-world generator is not an ECS resource, so a new consumer cannot
inject it by accident. A tripwire test asserts a deep worldgen voxel answers
solid through every public solidity path with an empty cache
(`empty_cache_solidity_paths_read_the_worldgen_authority`).

Corollary (deferred): the legacy S1 `TerrainGen` should become unnameable
outside the authority module once its last near-namer — the far-mesh's own
`FarFieldTerrain` — moves onto a worldgen-shaped summary far field.

## Content model

Items, blocks, biomes, blueprints, models, animations are all data-driven
registry entries. Models are transformed cubes with keyframe animations —
steal the shape of Blockbench's geometry/anim JSON rather than inventing one.
Authored in-game, emitted as content-pack data, consumed by the renderer; no
engine animation stack.
