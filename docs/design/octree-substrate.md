# The octree substrate — one spatial index for far field, water, and (later) the statistical tier

Design pass opened 2026-07-22 (live session), from the leaning recorded in
`water.md` § "LEANING, NOT DECIDED — one octree substrate". Decisions D1–D3
below are **DECIDED (user, 2026-07-22)**. The node contract (§ 3) is
**PROPOSED, not ratified** — nothing is briefed against it until the user
ratifies it.

## 0. Priors (the sweep — what already existed before this pass)

The finding that shaped the pass: **the octree mostly exists, in pieces, under
different names** — the S-2 story ("implemented three times under three names")
repeating at the structural level.

1. **The S3 chunk pyramid is an octree.** 32³ cubic chunks at 3D positions;
   level-L+1 chunks derive from 8 children (`dc-core/src/lod.rs`,
   `LodPyramid::get_or_derive`, `MajorityNonAir`). ARCHITECTURE.md committed on
   day one: *"LOD is a 3D lattice (octree-style pyramid, not heightmap LOD)"*.
2. **The material reduction step is built, tested, unrendered**:
   `MixtureDownsampleRule` / `DominantClassDebrisAware`
   (`dc-core/src/materials/lod.rs`) — 64 child eighths → 8 parent eighths,
   occupancy voted in eighths, losing class folded in rather than lost, proven
   to compose voxel-for-voxel with the block pyramid. `docs/spines.md` § 3
   row 1; this pass is what finally consumes it.
3. **FF2a left the socket ready** (journal/0023): a far column is already a
   potential *stack* of `ColumnSpan`s to the mesher; the mesher is a pure
   function of plain span data; the payload is persistence-shaped POD.
4. **The persistence spine is decided in outline**
   (`voxy-dh-recon-2026-07-19.md`): summaries derive from the authority,
   subscribe to the edit dirty-rail, persist beside S3 region files. And
   `HostWorld` already implements committed/fluid for full-res chunks — pin
   edited, evict untouched, byte-identical re-derive (journal/0051).
5. **Water's requirements are on file**: bulk flow = octree of continuous-flow
   volumes, *static data while its outlet connects*, event-driven
   (`water.md` § 2–4); S15 proved a 64× hypsometric compression answers levels
   coarsely (`S15-results.md`); S16 is unwritten and drafts against an octree
   **node**, not a per-deep-cell summary (`water.md` § consequence for S16).
6. **Ratified constraints that bind this pass**: one shared terrain material,
   variety is data (multidraw batching — voxy-dh addendum); the far field
   rides Bevy's engine GPU-driven mesh path, bespoke cmdgen and vertex pulling
   rejected (same addendum); no simulation-resolution edge may reach the eye
   (S-4); **rendering is never an observer** — the vista-as-augury constraint
   (ROADMAP, light.md § 5): drawing far state must never commit facts;
   headless crates never depend on rendering.

## 1. DECIDED (user, 2026-07-22, live session)

**D1 — One substrate, and it is the existing chunk pyramid.** The octree that
FF2b, bulk water flow, and (later) the statistical tier share is the S3 LOD
lattice, *named and given a payload contract* — not a new spatial structure. A
node is `(level L, ChunkPos)`; L0 is the 32³ authority chunk; a level-L voxel
edge is `0.9 · 2^L` m. Scope discipline from the leaning stands: **the first
two consumers (renderer, water) define the payload**; statistical/social is
recorded as an intended extension (§ 4), designed for by *not precluding*, not
by building.

**D2 — First build slice: FF2b-minimal.** Wire the two existing reduction
pyramids (block + `MixtureDownsampleRule` material) through FF2a's stepped
mesher: coarse volumetric far chunks as stepped geometry, replacing the
top-sheet-only far field. This empties spines § 3 row 1, gives chasms and
overhangs a far-field answer, and forces the node contract to be real.
Persistence + dirty-rail (far edits visible) is the follow-on slice, not this
one.

**D3 — Stepped all the way.** At every level, distance speaks the voxel
language: coarser chunks mesh through the same stepped-column register as
FF2a, one shared material, variety as data. Considered alternatives are
recorded in § 5 — the strongest of them (vista-as-augury) turned out to be
**complementary, not an alternative**: it governs far-visible *live* state,
not terrain geometry.

## 2. What it looks like in game (the ratification image)

Stand on a ridge: the horizon is tens of kilometres of real, specific world —
that mesa is *the* mesa, because the far field is a reduction of the
authority, not a painting. Dig a pit-mine for a week; walking back from 5 km
out, the scar is on the skyline (follow-on slice). Look down a chasm: the deep
dark is coarse-but-real geology, not fog. Later, the same tree that answers
"what is over that ridge" answers "where is the duke".

## 3. The node contract, v0 — PROPOSED (not ratified)

A node's payload has three strata, each with a declared **reduction rule**
(child → parent) and a declared **source** — every field derives from the
authority or from the children's same field, never from a side channel (S-3).

1. **Contents** *(exists)*: the paletted block chunk (S3 pyramid,
   `MajorityNonAir`) and the material chunk (`MixtureDownsampleRule`).
   Deterministic, cacheable, and already proven mutually consistent (material
   LOD never claims volume the block pyramid dissolved). This is the
   renderer's payload; FF2b-minimal consumes exactly this.
2. **Occupancy / sky summary** *(partially exists as per-column summaries)*:
   per-node occupied-eighth count and sky-open column mask — serving the
   adaptive load volume, lazy heightmap / sky-exposure caches
   (ARCHITECTURE.md § cubic chunks), and far-field coverage culling. Derived
   from stratum 1.
3. **Water summary** *(requirements only — hydrology stays paused; S16 drafts
   against this stratum)*: storage volume (derivable from stratum 1's
   pore/free eighths via the existing occupancy primitives), a permeability
   aggregate, a level summary (S15's hypsometric form, generalized per node),
   and an **outlet-connectivity slot** for bulk flow. Named so the unbuilt
   system's obligations are readable at the seam (S-5); not built.

**Rules riding the contract:**

- **Committed/fluid at node granularity (S-2).** An edit dirties the node's
  ancestor path to the root; untouched nodes re-derive byte-identically and
  may be evicted. Persisted summaries live beside S3 region files (the DH
  lesson). This formalizes what `HostWorld` already does at L0.
- **Rendering is never an observer.** No derivation performed for the eye may
  commit facts (vista-as-augury constraint, § 0.6).
- **Agreement tests are the acceptance tests** (S-3/S-7): coarse claims match
  exact walks — exactly where derivation is deterministic, statistically where
  quantization is deliberately unbiased (the 0055/S15 pattern).
- **Halo/knob rule (S-1):** every consumer declares its depth/radius bound,
  and where there's a range there's a knob.
- **Headless split:** the substrate lives in dc-core (+dc-worldgen consumers);
  dc-client is one consumer among several.

## 4. Intended extensions (recorded, not designed)

- **Statistical tier / social**: distributions over possible states hang on
  nodes; bounded collapse depth N and octree depth are the same kind of bound
  (the leaning's closing observation). "Where is the duke" stays answerable
  because subjects have positions and positions are fluid facts in the same
  index.
- **Relation graph over the index**: trade routes, political ties, drainage —
  edges between nodes, the shape `DeepField.recv` already has over spatial
  cells. Containment alone cannot express these; the graph rides *over* the
  one index, it does not replace it.

## 5. Considered alternatives for the far register (D3's record)

- **Imposter / skyline billboard cards** (the classic open-world trick):
  rejected — a pre-baked painting *beside* the authority (A-1/S-3 shape), a
  second art pipeline, and edit propagation becomes re-baking imagery instead
  of re-deriving data.
- **Aokana-style ray-marched SVDAG** (mesh-free volumetric): already
  effectively rejected by the Bevy-mesh-path decision (voxy-dh addendum), and
  it would split the lighting/tonemap register in two — the parity the far
  field currently keeps for free (journal/0004's lesson standing).
- **A painterly "far register"**: the honest version of this instinct already
  exists in the corpus as **vista-as-augury** — far-visible *live* state
  (smoke, herds, societies) rendered as a read of the *distribution* without
  committing facts. That is about the statistical overlay, not terrain
  geometry, so it composes with stepped-all-the-way rather than competing.
  Note also that at extreme range a coarse step subtends under a pixel, so the
  register question only ever concerned the mid-far band.

## 6. Open questions (filed, none blocking D2)

1. Where the runtime pyramid lives for the far field (client-side derive
   today; host-owned once persistence lands) and the region-file coupling
   format (the recon's 8-byte packed quad remains a candidate *storage*
   format only).
2. Eviction policy for derived levels (the S-2 rule says evictable; the knob
   wants a number).
3. FF2b-minimal meshing detail: derive per-tile `ColumnSpan` *stacks* from
   coarse chunks vs meshing coarse chunks directly — the mesher already
   reasons in span stacks, so the first is the conservative path; the slice
   brief decides after measuring.
4. Statistical-tier attachment point — deferred with its own design pass.
