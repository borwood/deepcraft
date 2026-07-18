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
- **Custom shaders are a contract.** User shader packs need a stable surface:
  which passes exist, what the G-buffer contains, which uniforms/hooks packs
  may touch (the Iris/OptiFine lesson). Forward-vs-deferred and the hook
  surface get decided by spike S4, before renderer growth makes it expensive.
  WGSL is the pack language; naga translates per backend.

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

## Voxel scale

Player height is 3 or 4 voxels (vs Minecraft's 2). This is a world-resolution
budget decision, not aesthetics: halving voxel size is 8× voxels per world
volume — memory, meshing, worldgen, save size, physics query density. It also
sets reach, jump height, stair conventions, and every block model's real scale.
Resolved empirically by S1; until then nothing hard-codes a voxel:meter ratio.

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

## Content model

Items, blocks, biomes, blueprints, models, animations are all data-driven
registry entries. Models are transformed cubes with keyframe animations —
steal the shape of Blockbench's geometry/anim JSON rather than inventing one.
Authored in-game, emitted as content-pack data, consumed by the renderer; no
engine animation stack.
