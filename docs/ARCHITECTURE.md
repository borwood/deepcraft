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

## One world-answer surface (2026-07-19, RATIFIED 2026-07-19)

Drafted by the S1-fallback sweep (journal/0017); user-ratified same day.

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

## Modularity and performance as build constraints — DECIDED 2026-07-21 (user)

**Build every system API-first, in remove-and-plugin shape.** Anticipate that
any subsystem will be pulled out and replaced — because several will be, and
some are placeholders today (the surface veneer awaiting ecology is the live
example). A system whose boundary is a real API can be replaced; one whose
behaviour has leaked into its callers cannot. This is the same instinct that
produced roles-as-contracts and classes-as-contracts, stated as a general build
constraint rather than rediscovered per slice.

The corollary is the honest-placeholder rule: where a subsystem is standing in
for one not yet built, it is accepted **as-is and marked temporary** — never
incrementally patched toward looking finished. Bandaiding a placeholder buys
appearance at the cost of the seam you will need when the real thing lands.

**Performance is a standing doctrine, not a per-slice afterthought.** Memory and
throughput work belongs in the design of every domain from the start:

- **Recycling and pooling** — reuse LOD objects, chunk buffers, meshes, and
  scratch allocations rather than churning them. (Prior art already surveyed:
  `docs/design/voxy-dh-recon-2026-07-19.md` lists persistently-mapped pooled
  vertex buffers as the transfer for far-tile churn.)
- **Mipmaps** for terrain texturing — named here because the corpus contained no
  record of it before this entry.
- **Perf-first memory usage in all domains**, not only rendering.

Recorded 2026-07-21 after the user observed that this conversation had happened
before and was never written down — the "defer = write it now" rule
(`.claude/skills/session-workflow`) failing in the direction it exists to
prevent. Distance/far-field performance in particular is expected to improve
from mipmaps + pooling rather than from reducing draw distance.

## The fill contract: contents are authoritative, the block is derived — DECIDED 2026-07-21 (user)

**What fills a voxel is its contents; the block is a pure derived
classification of them.** `classify(contents) -> Block` is a total function in
dc-core, and `block == classify(contents)` is the invariant. Two opinions about
what a voxel *is* become structurally impossible, because one is a function of
the other.

**Scope of the invariant, as built (2026-07-21, journal/0052).** This entry
first said "every voxel"; implementation showed that cannot be true yet, and the
honest rule is:

> for every voxel with **non-empty contents**, `block == classify(contents)`.

A voxel with no contents record is **unclassified — not classified as Air**.
`classify` does answer `Block::Air` for empty contents, but the generator does
not apply it where it never wrote a record: the surface veneer (stubs.md § 2),
the legacy soil band and the unrecorded basement below the deep-time record,
ocean floor, the border wilds, and ruin posts all keep their legacy blocks. The
exception is **enumerated and pinned by test** — a geology block appearing
without a record fails the suite — so it can only shrink, and it shrinks by
itself as each stub acquires its heir. Wording corrected by the integrator; the
decision itself is unchanged.

**Why this, and why now.** Generation currently produces blocks and materials on
two parallel paths, and journal/0010 installed a trust gate in which the *block*
decides whether contents are believed — correct while worldgen only ever emitted
8/8 full voxels, and a bug factory the moment it emits fractions. Four
independent threads then arrived demanding the same change: partials-first
emission (sand's fractional top, journal/0049), vegetation as a fraction on a
substrate, soil's loose/packed form axis, and the caves/water thread's voids and
occupants (docs/design/water.md). They are one contract change, not four
features.

**The generalization**: a column's fill is an **ordered list of spans**, each
carrying (contents, form, fractional occupancy). Today's single-height,
solid-below-surface column is the degenerate case of that list. New consumers
should be written against "a column is a set of spans", not against "one height,
solid below" — the latter contract already has accreting consumers (the edit
allowance ceiling, far-field column summaries, surface scans) and each one
raises the price of the fork.

**Consequences accepted with the decision:**

- **`Block` does not grow form-aware variants.** It stays the coarse
  render/storage/far-field summary in its existing vocabulary; loose clastic
  classifies to its class block. Form lives in contents. The immediate cost —
  loose and structural clastic being visually indistinguishable — is accepted,
  with form-dependent texture variants filed as a later visuals decision.
- **Solidity moves off the block** onto an occupancy threshold (visuals.md
  already reserved "solid ≥ 4/8"). `Block::is_solid` degrades to a shim exact
  only for full voxels.
- **Contents are promoted from render-only to authoritative.** journal/0010's
  boundary ("the interning is render-only and never reaches sim state") moves
  deliberately: collision, water, and loose-material gravity read occupancy.
  The MixtureTable landmine rules are UNCHANGED and absolute — canonical,
  order-independent `VoxelContents` cross the seam; order-dependent `MixtureId`s
  never do (journal/0007–0008, 0010's "resolve at the boundary").
- **One occupancy answer, not four.** Water fill (free eighths + pores), the
  loose gravity march, compaction, and sim light all want per-voxel
  matter/capacity fraction. They read one primitive rather than each
  re-deriving occupancy from a bool — the private-re-derivation failure mode
  this decision exists to prevent.

**Fractions come only from the ledger** (user, same conversation): partial
occupancy expresses recorded quantity and recorded variance — the record's
metres and its real heterogeneity — never cosmetic noise. See
`docs/design/materials.md` § "The forms design pass" for the full ratification
set (sand-as-form, the root-lattice soil model, grass suspended).

## A summary is not an authority — DECIDED 2026-07-21 (user)

**The rule.** When you write a cheap answer because a consumer cannot afford
the real one, the cheap answer must be *derived from* the real one — never
become it.

**The test, asked before the commit lands:** *"if this consumer disappeared
tomorrow, would this code still exist in this shape?"* If no, it is a summary
wearing an authority's clothes. It gets a `docs/design/stubs.md` entry naming
its heir, and — where the authority exists to compare against — **a test
asserting the summary AGREES with the authority rather than replacing it.**

**Why this needed to be written down.** The failure mode is not a stub. A stub
looks like a fake: it says placeholder, it names an heir, someone wrote it
knowing. **A leaked requirement looks like working code that passes tests**,
which is why `stubs.md`'s "an unlisted stub is a defect in this inventory"
did not catch a single one of the four instances below. Corrections #29 is the
same failure one layer down: the assumption that made face-culling safe lived
in a doc comment, and prose cannot fail a build.

**The class, with the four instances found in one audit (2026-07-21):**

| the stand-in | written because | what it became |
|---|---|---|
| `collapse.rs::surface_sample`'s surface branch | the far field cannot afford a full column at 3 km | **the world's surface material rule** |
| `deeptime/lithology.rs::Litho::reference_material` | the deep sim has no material properties, only 6 classes | **the sim believes loose regolith is sandstone** |
| `fill.rs::is_loose` (a class-string test) | expression needed form before a form axis existed | **it IS the form rule** |
| `biotic.rs` `bio_resist` (a rate multiplier) | roots needed representing before root materials existed | **roots are a number, not a material** |

Each was correct when written. Each hardened into the definition of the thing
it stood in for. **A stand-in becomes the definition unless something stops
it.**

**The proven implementation pattern is already in-tree**: journal/0055's
integrator-added test asserts the far horizon and the near ground agree on
surface *material*, not merely on height. That is the agreement test this rule
generalizes. Had it covered *contents* rather than *block*, the surface-branch
defect could not have survived a build.

**Cost of not having had it:** journal/0055 shipped under the headline
"distribution-first expression — the record skins the world" while the one
part of the world a player looks at kept a dominant-collapse paint job. The
slice's own headline claim was false for the surface.

## The content set is frozen at world creation — DECIDED 2026-07-22 (user)

**The rule.** A world's generation-affecting content set is fixed when the
world is created. **Plugins that affect generation cannot be added to an
existing world.** Non-generating content can: render/shader packs, recipes,
items that no worldgen pass emplaces.

**Supersedes** geology.md's earlier seed-stability line ("adding materials
changes ungenerated regions of existing worlds — accepted, DF-like"). That
policy is withdrawn.

**Why (user, verbatim in substance):** the world's history is going to be a
dwarf-fortress-class social simulation, and it will care about materials.
*"It just wouldn't make sense to try to add content to the game which would
change the very mythology of it post-generation."* Anything that changes
history is flatly out. **No post-hoc history rewrites: it was real and it
mattered.**

**The accepted cost, stated by the user rather than discovered later:** a
content plugin added to an existing world is **inert / ungenerated at best**.
A creative-mode player could still spawn its content; a survival player could
still build recipes from it that use *existing* materials; recipes depending
on **new materials being found in the world are simply stranded**. The
cultures of the world would not know about new recipes either — the player
could teach them. This is judged honest to the simulation.

**The partition this implies** — the line is not "core vs mod", it is
**"does it participate in generation?"**:

| class | may be added to an existing world |
|---|---|
| render/shader packs (S4 `--pack`), purely visual | **yes** — already outside world identity by construction |
| recipes, items, content no gen pass emplaces | **yes**, inert where it depends on absent materials |
| geology members, passes, deep-sim providers, anything a pass `selects` from | **no** |

**Consequence this policy FIXES, for free.** `HostWorld` evicts
generated-and-untouched chunks and re-derives them byte-identically ("store
only what the derivation cannot predict"). Terrain a player has visited but
not edited is therefore *re-derived*, not stored — so a content-set change
would silently rewrite already-walked ground while their edits stayed pinned
beside it. Freezing the content set removes that failure mode entirely.

**The sharper half, owed as work:** the same argument applies with more force
to a plugin that goes **missing** on load. A world generated with
`mw:magic-water` and reopened without it re-derives its droppable chunks
*wrong*. So the content set must be **recorded in the save and validated on
load**, and a missing generation-affecting plugin must be a hard refusal, not
a warning. Whether anything records the content set today is unverified —
under audit.

**AMENDED 2026-07-22 (user), same conversation — plugins are MARKED, not
banned.** Plugins present at generation time are **marked as generational**;
a missing generational plugin on load is a **HARD REFUSAL**, never a warning
(the droppable-chunk re-derivation would silently rewrite already-walked
ground). New plugins *may* be added afterwards — they simply **do not
contribute to world generation**. They may still contribute to *present-day*
history, but only through an in-world **discovery vector**: the player
introduces a recipe, or a discovery mechanism fires (DF-style strange mood).
So the line is not "frozen world, inert additions" but **"history is written
forward, never backward"** — new content enters the world the way new
knowledge enters a culture, not by retroactively having always been there.

## Provider seams — the shape (partially DECIDED 2026-07-22, user)

The constructive half of § "A summary is not an authority": rather than a
stand-in *value* hardening into a definition, a system declares a **seam** —
a named provider with an explicit contract — so an unbuilt system's obligations
are visible at its call sites. Derived from the 34-seam inventory
(2026-07-22): ecology owns 9, hydrology 6, materials 6, social sim 5.

**DECIDED (user):**

- **Shape.** A `Providers` struct of **plain fn pointers** — the `PassBody`
  discipline: deterministic, no captured state, no closures, no trait objects.
  Resolved **once at world build**. Every slot carries an **identity default**
  reproducing today's behaviour exactly, so "provider absent" is provably
  byte-identical — the proof shape the four existing flags (`biotic`,
  `erodibility`, `full_agents`, `tectonic_history`) already use.
- **Effective reads.** A provider **declares its own reads**, and a pass's
  effective reads = **its declared reads ∪ the reads of every provider it
  imports**. Without this a provider is a *hidden edge* in the pass graph: a
  pass importing `depth_to_water` depends on `Hydrology` whether it declared it
  or not, the topo-sort is silently wrong, and the failure is order-dependent.
- **Conflicts warn, they do not fail closed.** Two plugins supplying one
  provider is not an error. The user is asked, at world creation:
  *"Warning: there are two conflicting providers for X. Proceed with the
  last-in-order provider? This may result in unexpected generation behaviour.
  [proceed with last-in-order] [I want to disable one of them]"* — deliberately
  softer than the pass graph's fail-closed rule for two creators of one
  resource, because the user, not the engine, should own which of two mods
  wins.

**PROPOSED, NOT YET RATIFIED** (integrator; do not build against these):

- *Providers resolved are part of world identity* — a world generated with a
  provider and one generated with its identity fallback are different worlds,
  so the manifest would record the resolved table and the conflict choice.
  (The conflict dialog appears to force this: a choice that changes generation
  must be recorded or the world stops being reproducible.)
- *Providers produce planes; consumers read planes* — a provider used inside a
  hot loop is materialized at a pass boundary, once per iteration, preserving
  the scalar↔parallel byte-identity invariant. Value-level providers legal only
  at cold call sites.
- *A purity contract*, since fn pointers do not enforce one: pure in declared
  inputs, no globals, no interior mutability, no wall clock, and all entropy
  from caller-owned seeds passed in, never ambient.

**Not built by the first slice.** The first conversion slice (`outcrop_at`,
`wave_energy`, `parent_p`) deliberately builds the struct and the identity
defaults ONLY — no registry, no loader, no declaration/validation machinery.
Those are to be designed from what the conversions teach.
