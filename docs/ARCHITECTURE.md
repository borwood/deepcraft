# deepcraft architecture

Status: ~~pre-spike sketch, 2026-07-18. Everything here is a working hypothesis
until the spike named next to it lands (see SPIKES.md).~~
**⚠ THAT STATUS LINE IS RETIRED — corrected 2026-07-29 (baseline sweep S1/#4).** It was true
the day it was written and no decision reversed it; it simply **stopped describing the file.**
This document now carries **twelve dated DECIDED / RATIFIED blocks, eight of them marked
`(user)`** — including the 2026-07-26 plugin-agnostic / authored-ORDER ruling that is the
newest architectural decision in the corpus. Both read-first surfaces already state the
corrected reading (`CLAUDE.md` item 3, *"decisions with dates"*; `spines.md` § file roles,
*"what we **decided**, and why, with dates"*); only this file did not.

**Read it as:** *the sketch that was written 2026-07-18, then amended in place by dated
decisions.* **A block marked DECIDED or RATIFIED with a date is settled and is NOT provisional**
— unmarked prose from the original sketch still is. *(This mattered: line 3 told a cold agent
that user ratifications recorded here were hypotheses.)*

*See also `SPIKES.md`, which the retired line deferred to for "has the spike landed?" —
it has no completion state and cannot answer; its own banner now says so.*

## Shape of the system

*The diagram below names five crates. **`ls crates/` holds eight:** it omits **`dc-host`**
(the headless wasm plugin host — `crates/dc-host/{src/lib.rs, tests/wasm_plugin.rs,
tests/parity.rs}`), **`dc-mcp-dev`**, and **`dc-physics`** (which this document discusses in
prose but never draws). Verified 2026-07-29 (baseline sweep S1/#13). No decision rests on the
omission; recorded because this roster and `CLAUDE.md`'s headless-crates list disagree with
each other as well as with the tree.*

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

> **⏸ THE SOCIAL/TERRITORIAL HALF IS ON HOLD — user, 2026-07-28.** The **geological** half
> is live and shipping. The **social/territorial** half describes an intention, not a
> pipeline: its implementation was unratified bootstrap content and was removed
> (`journal/0121`), leaving dc-sim's statistical tier with **zero production callers**
> (`spines.md` § 3). *"We do want these systems **eventually**: they are effectively on
> hold."* The one-system/no-seam property is the requirement worth preserving and is the
> first thing to re-derive when the layer is designed for real.
> **Gate:** engine + all non-bio earth science in the ratified SDK-plugin shape → ecology →
> social. *"Sufficiently complete" is a **USER call**;* see
> [`design/worldgen.md`](design/worldgen.md) § *Sequencing*.
>
> *This banner was owed on 2026-07-28 and missed: the ruling said "mark ON HOLD, not strike"
> and named `worldgen.md` as the template, and the template was applied to exactly one
> document. Found by the baseline sweep the same day. **The sibling sites — `:105-107`,
> `:153-154`, `:164-166` — carry the same status and are covered by this banner.***

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
ocean floor, and the border wilds ~~, and ruin posts~~ all keep their legacy
blocks. (**Ruin posts left the list 2026-07-28 by deletion** — journal/0121, the
bootstrap-history removal — not by acquiring a record. Wording only; the decision
is unchanged.) The
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

- **Shape.** A `Providers` struct of **`Option<fn>` slots** — the `PassBody`
  discipline: deterministic, no captured state, no closures, no trait objects.
  Resolved **once at world build**. **`None` means identity**: the slot's own
  identity function reproduces today's behaviour exactly, so "provider absent"
  is provably byte-identical — the proof shape the four existing flags
  (`biotic`, `erodibility`, `full_agents`, `tectonic_history`) already use.
  *(REFINED 2026-07-22, user: "roll with that proposal". Was "plain fn
  pointers" with absence inferred by comparing addresses against `default()`.
  Rust guarantees `fn`-pointer address uniqueness in neither direction —
  identical-code folding merges, per-codegen-unit instantiation splits — and
  both fired: corrections #32. Absence is now **structural**, a discriminant
  stored at construction, so the failure is impossible rather than guarded by
  a docstring. Consequence, deliberate: `Some(identity_fn)` counts as
  **supplied**, because the decidable question for a world manifest is "did an
  heir answer this slot?", not "does the answer happen to equal the old one?")*
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
  > **⚠ TWO PREMISES IN THESE TWO BULLETS EXPIRED — cross-reference added 2026-07-29
  > (baseline sweep S1/#8, S1/#9). Both DECISIONS survive; both stated JUSTIFICATIONS were
  > written in derive-and-reject terms**, and § *The engine is plugin-agnostic, and pass ORDER
  > is authored* (DECIDED 2026-07-26, user) — **seventy lines below, with no cross-reference
  > either way** — retires that frame:
  > - *"the **topo-sort** is silently wrong"* — under author-and-validate there **is no
  >   topo-sort** to be silently wrong. The requirement to declare effective reads is
  >   **unchanged**; it is now a **validation** input rather than an ordering input.
  > - *"deliberately softer than the pass graph's **fail-closed rule for two creators of one
  >   resource**"* — that section states *"the rejection that forces revision tokens
  >   **disappears**"* once the order is authored, so the contrast case carrying this
  >   argument is the thing being retired. **The user-owns-the-conflict conclusion is
  >   unaffected and is the part that was decided.**
  >
  > *This is spines A-2 (a justification outliving its premise) with the unusual property
  > that the expiry and the justification are in ONE document.*

- **The resolved provider table is part of world identity** (DECIDED
  2026-07-22, user). A world generated with a provider and one generated with
  that provider's identity fallback are **different worlds**, so the manifest
  records the resolved table *and the answer given to the conflict dialog*.
  This is forced by the two decisions above: a choice that changes generation
  must be recorded, or the world stops being reproducible and the frozen-set
  hard refusal has nothing to check against. A fallback firing at world
  creation is therefore a permanent fact about that world, not a runtime
  nicety — it is warned about at creation, recorded, and legible afterwards
  ("this world was generated with N unresolved providers").
- **Headless creation resolves last-in-order and logs loudly** (DECIDED
  2026-07-22, user). The conflict dialog cannot be the only resolution path —
  worlds are created non-interactively by the test suite (dozens per gate
  run), by probes, by background agents, and by any future dedicated server.
  Non-interactive resolution takes the last-in-order provider and logs; because
  the resolution is recorded in world identity, reproducibility holds either
  way. Refusing when non-interactive was rejected as contrary to the
  warn-don't-fail spirit.

**PROPOSED, NOT YET RATIFIED** (integrator; do not build against these):
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

---

## The engine is plugin-agnostic, and pass ORDER is authored — DECIDED 2026-07-26 (user)

> *"Our passes are PLUGINS. Our passes should be viewed through the lens of THIRD PARTY
> MODS. We are our own first modders. **THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC,
> FULLSTOP, EMPHATICALLY.**"* — user

**The product this serves**, stated by the user the same day and recorded because every
argument below descends from it: *a highly customizable voxel-based crafting-game
**generator** — plugin based, each world able to be its own unique sim composed from
declared plugins built on engine primitives*, whose **first plugin pack** is the
earth-like world generator we are building. The engine is not "our simulation with a mod
API bolted on." The default pack is one pack.

### What this indicts today

`deeptime::runner::DeepAxis` is a **closed enum inside `dc-worldgen` whose variants are
the names of our own pipeline stages** — `Forced`, `Incised`, `Weathered`, `Diffused`,
`Compensated`, `Windblown`, `Settled`. The engine knows the default pack's pass roster by
name. That is the violation, and it is not incidental: it is the *consequence* of choosing
to **derive** pass order from declarations rather than **accept** it as data.

### The two models, and why the second one wins

- **Derive-and-reject (today).** The kernel infers an order from `{reads, writes}` and
  rejects `AmbiguousWriters` — two writers the declarations leave unordered. N passes that
  transform one shared `TERRAIN` in place are therefore *unschedulable*, so each stage must
  write a distinct token naming its output, and the chain of tokens **is** the order.
- **Author-and-validate (DECIDED).** Order is **data on the world** — chosen per world,
  alongside seed and epoch count. `{reads, writes}` stop being the ordering input and become
  the **validator**: a pass reading a resource nothing writes, a read satisfied only by a
  later pass (a lag — make it explicit), a genuine cycle.

**Author-and-validate is strictly more expressive.** "Ambiguity" is only a defect when the
engine is trying to infer a sequence; given an authored one it is just the sequence, and
the rejection that forces revision tokens **disappears**. Derivation buys exactly one
thing — the author need not state an order — and costs the ability to *have* one, which is
the premise of worlds being unique sims.

**And an engine cannot derive a third party's intended order.** Requiring it to try means
the declarations must encode that order, and encoding it in engine-owned resource names is
precisely how `DeepAxis` came to exist. The mechanism and the violation are the same
choice.

### The claim that was replaced, and the sentence that refutes it

`material-behavior.md` §5 recorded ORDER as *"derived from `{reads, writes}` by topo-sort…
**This replaces a hand-declared 'canonical order'**"*. That is now **superseded**, and it
was never true in the strong sense it was written: journal/0090's own summary says a
linear relaxation pipeline *"does **not** fall out of a dataflow graph for free… you have
to **name each revision as a distinct resource** for the topo-sort to reproduce a fixed
sequence."* **The order does not fall out of the declarations; it is fed into them.**
Filed as corrections #65.

### What is engine-owned and what is not

- **Engine:** the pass-graph kernel (validate an authored order), the epoch/sub-turn clock
  and `dt`, field-**solver primitives**, the cell/record storage, refinement primitives
  (north-star § the refinement tier).
- **Content:** passes, their order, their cadence, the **fields themselves** (a field is a
  declarative plugin over solver primitives — no engine-baked field vocabulary), materials,
  and their behaviours.

**What a REFINEMENT PRIMITIVE is — DECIDED 2026-07-29 (user).** The tier was posed but never
defined, so it sat in the list above as a name with no content. The user's definition:

> *"basically any primitive responsible for the way things are actually drawn in the runtime
> game where we interpolate/upscale fine chunks from coarse cells; plugins will be able to
> define this behavior using refinement primitives exposed by the SDK. **Totally necessary for
> a plugin-agnostic engine.**"*

So the tier owns the **coarse → fine reconstruction**, and it is engine-owned for the same
reason the solver primitives are: a pack that cannot author how its own materials are
upscaled is not a pack, it is a carve-out. **The engine currently has ZERO refinement
primitives** — the tier is decided and undesigned, and its first member is unscoped. Read the
list of engine-owned items above as **non-exhaustive** (user, flagged twice).

*Candidate first members, not yet ruled:* `dc-core`'s `sample_dithered` (the cake law, built
and called by nothing) and an octaves/facies-driven design the user recalls ratifying — under
investigation 2026-07-29, because the corpus may hold **several contradictory ratified designs
for one problem**.

**The S2 statistical tier is HELD as a probable future primitive — DECIDED 2026-07-29 (user).**
journal/0121 removed the bootstrap history content and thereby left `engine::{query, observe,
force_fact}`, `Ledger` and `ToyWorld` with **zero production consumers workspace-wide** — the
largest `spines.md` § 3 row in the file, created by that slice. The ruling: it is *likely a
primitive awaiting a consumer*. **Hold — do not dispose, do not find it a consumer, and do not
reopen the discussion until bio/eco readiness** (which is itself a user call; progress on earth
science does not entitle anyone to open it).

> **This is `spines.md` § 3's third exit — HELD AS A CANDIDATE — and § 3 already documents it**
> (added 2026-07-28, journal/0120, by this same ruling; the S2 row is its worked example).
> The three exits are **consumed** · **deleted** (no standing — journal/0121) · **held**. The
> distinction is load-bearing, because a zero-consumer row read as a backlog item demanding a
> consumer is precisely the misreading that nearly preserved the history content. § 3's own
> rule for this state: **a held row must say so explicitly and name what unblocks the
> judgement**, or it is indistinguishable from a consumed-row-waiting-to-happen.
>
> *(An earlier version of this note said § 3 "only knows one" exit. It knew all three, a day
> before I wrote that. Corrected 2026-07-29 — asserting a corpus gap without opening the file
> is the exact failure this session has been cataloguing, and it landed in `ARCHITECTURE.md`.)*

**Corollary — resource ids are opaque and open.** The kernel is already generic over
`Axis: Copy + Ord`; the closed enum is a caller's choice, not a kernel constraint. A pack
declares its own resource ids. Where the default pack exposes ids a third party may depend
on, that is a **pack** surface with the ordinary obligations of one — not an engine ABI.

### RATE — ratified 2026-07-24, ~~never built, and now first in line~~ **BUILT 2026-07-29**

> **✅ BUILT 2026-07-29 — journal/0123, `dependency-graph.md` E3.** Cadence is authored data
> (`CadenceTable` over each pass's declared default), sub-turns execute with the cell state
> carried between them, and `dt` is live in the rate-shaped passes converted so far. The
> acceptance test below was met as a **hash comparison**: an empty cadence table reproduces the
> shipped world bit for bit (`GOLDEN_SURFACE 0x15A6_B756_7A84_29FB` / `GOLDEN_RECORD
> 0x820B_A198_49DD_234A`, `crates/dc-worldgen/tests/rate_axis.rs`).
>
> **RATE does NOT own stability substepping** (user ruling, 2026-07-29) — that is the S-10
> field-solver kernel's, and `stubs.md` § 30's remaining half. **And the arc does not close
> here:** authored ORDER, the open vocabulary, and declared epochs are still owed.
>
> *The section below is kept as written because it is the argument that sequenced the slice,
> and because its central claim was confirmed from the other side rather than superseded.*

`material-behavior.md` §5's RATE axis (a pass's phase length; `dt` scaling its
transformations; a high-rate pass running several sub-turns while a low-rate one runs once)
was ratified from the user's 2026-07-23 sketch **and never built** — `dt` is pinned to
`1.0` and no transform scales by it.

**The 2026-07-26 blocker is what its absence produces.** `erosion.rs::diffuse_scale_cell`
caps a cell's export at its **entire regolith inventory**, with no `dt` in the expression,
so the operator has a period-2 mode independent of timestep (journal/0116) — measured
concavity ACF(1) **−0.87 / −0.91** against a shipped world's **+0.38**. The heir's own
prescription is *"a hillslope operator whose transfer stays a function of `rate × dt`."*
**That is the RATE axis.** So the fix is not a patch beside the architecture; it is the
architecture's first real consumer, and the world becoming stable is its acceptance test.

**How it actually resolved, 2026-07-29, and the resolution sharpens the argument.** The
operator was fixed *first* (journal/0122) by sub-cycling **inside the pass** — the pass, given
no engine clock, grew its own — which confirmed *"this blocker is what RATE's absence
produces"* from the other side. RATE then landed against the resulting known-good fixed point,
so its acceptance test was a hash comparison rather than a judgement about whether a landscape
looked right. **The prescription split in two along the ownership line:** `rate × dt` is the
engine's (authored, RATE) and the stability sub-division is the *kernel's* (derived, E4). Both
now sit four lines apart in `Erosion::diffuse`, which is where E4 will lift the second one
from.
