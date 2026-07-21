# Material volume model — design draft

Status: design discussion captured 2026-07-18; storage feasibility is spike S8.
Chunk format v1 (S3) reserves versioned sidecar sections for this model.

## The voxel as a container of eighths

A voxel is 8 volume-eighths. Each eighth holds exactly one material id (or is
empty). A voxel's loose contents are an **unordered multiset** — deposit order
is deliberately not tracked (mixing destroys information; that's what mixing
is). Three occupancy roles:

- **Structure**: a shape (full = 8 capacity, slab = 4, quarter = 2, stairs,
  …) occupied by structural materials (stones, etc.). The shape *reserves* its
  capacity in slots; filled/capacity = **density**, its complement porosity.
  A slab with 1/4 structural slots filled is porous and brittle. Fill may be
  heterogeneous (rubble/breccia — mixed stone types in one structure; the
  gameplay mechanics of *how* heterogeneous fill arises are open design).
- **Debris**: loose granular material (sand, gravel, snow, leaves, potsherds,
  knapping waste, …) filling unreserved volume in eighth increments, freely
  mixed. Renders as heightmap-blended layers of the constituents.
- **Pore occupants**: the structure's *unfilled reserved slots* can admit
  fine materials (mud, silt — a pore-filler type/subtype) and fluids.
  Packed pores alter the structure: insulation, permeability (waterproofing),
  perhaps strength/weight. Pack a crumbling wall with mud = wattle-and-daub,
  emergent from the physics rather than a recipe.

Fluids occupy empty eighths and pores (aquifers in porous stone, waterlogged
debris, quicksand — free consequences of the model).

## Packing

Two paths into pores:

1. **Deliberate**: a player/NPC action packs suitable debris into an adjacent
   structure's pores (tool/skill TBD — game design open).
2. **Overflow**: when new debris arrives and the cell has no free eighths,
   fines whose grain size fits the pores are shifted into them before
   accumulation spills to the cell above. (Falling debris checks: free debris
   slots → packable pores → stack above.)

## Granular material property sheet

Each granular material carries: **density** (stratification sort key, weight),
**grain size** (what fits into which pores; sieving), **cohesion** (angle of
repose, slumping), **per-damage-type extraction resistance** (see below),
**permeability** and **insulation** contributions when packed.

## Extraction

Block damage is *typed* (dig, chop, smash, cut, sieve, …). Sustained
application of a damage type yields materials in ascending resistance order
under that type — leaves come out of a leaf/sand/gravel mix almost
immediately, then sand, then gravel. Tool choice reorders and can destroy:
a careless pick pulverizes the potsherds a trowel would recover — archaeology
lives in the extraction mechanics, not a minigame.

## Stratification is derived, not ticked

Stratification degree = pure function of (contents, time since last
disturbance, agitation history from the ledger). Never simulated per-voxel:

- **Worldgen deposits** arrive fully stratified (deep time baked in).
- **Player-era deposits** stratify lazily — computed on observation from
  time-undisturbed; heavies band downward by density.
- **Agitation** (water flow, vibration, digging) resets/accelerates the clock.

Consequence: deposits are *readable history* — clean bands = old and
undisturbed; chaotic mix = recent activity. Mining a stratified deposit yields
clean sequential bands; a fresh mix yields by extraction resistance.

Compaction closes the deep-time loop: debris under overburden, over ledger
time, migrates into a structure slot as sedimentary stone. Worldgen strata,
gameplay middens, and geology are one process at different tick rates.

## Storage — RESOLVED by S8 (2026-07-18): GO on free-form mixtures

The feared gradient explosion is **combinatorially impossible** for small
material sets: eighth quantization caps a k-material locale at `C(k+8,8)−1`
distinct canonical states. Measured: 2-material processes (wind, scree)
saturate at exactly 45 states and stay flat forever after; an adversarial
continuous 4-material gradient caps at exactly 495. Real deposit chunks cost
0.14–0.38 B/m³ (vs 0.144 debris-free, 2.74 raw); even 12-material uniform
noise — which no plausible process produces — stays 2.9× under raw.
Debris-free chunks attach no sidecar and pay zero bytes.

Implementation (dc-core::materials, sidecars `materials/slots-v0` +
`materials/mixtures-v0`): region-interned canonical mixtures, chunk palettes
indexing the intern table, riding format v1. Curated recipes are NOT needed
for storage — retained only as an optional cap-and-snap guardrail. Extraction
ordering, lazy stratification, and the mixed-voxel LOD rule are implemented
and property-tested; details in docs/spikes/S8-results.md.

Still owed by later work: sparse sidecar encoding for thin drapes (index
array, not the table, dominates cost), region-table lifecycle/compaction,
angle-of-repose settling, the render-blend prototype (S4 input), pore-packing
and heterogeneous-structure-fill mechanics.

## Materials and forms (added 2026-07-18)

A **material** is one identity with one property sheet. A **form** is a
presentation of it in context:

- **Emplaced**: structure (shape slots), debris eighths, pore fill — above.
- **Item**: discrete object — world pickup (a dc-physics rigidbody), in
  inventory, in hand/socket.

**Form archetypes are registry entries binding to material classes** (chunk,
shard, brick, powder, …) — the same contract pattern as geology passes
(docs/design/geology.md). New material × existing archetypes = its whole item
family for free; a new archetype retroactively covers every qualifying
material.

**Breaking is a sampled, conservative distribution.** Typed damage × the
material's fracture stats × tool → a seeded (replay-deterministic) sample
over outcomes; mass that doesn't become discrete drops **remains in the voxel
as debris eighths** (mining brittle shale leaves a shale drape in the hole —
sievable, shovelable, or left for archaeologists). Nothing is deleted;
middens happen by physics. Granular/sub-threshold outcomes deposit as debris;
discrete outcomes persist as rigidbody pickups with the sleep→inert-item
handoff (S6).

**Terminology — noted 2026-07-19**: the design term for these unconsolidated
eighths is **loose (granular) materials** — "debris" implies a provenance
(broken from something) that fresh snow, dune sand, or river gravel don't
have. The S8 *storage role* keyword stays `debris` (decided wire format);
the rename is vocabulary, not bytes.

**Loose-material mechanics (user sketches 2026-07-19, future work)**:
- **Gravity by vertical march** — unsupported loose eighths fall
  column-wise (the loose-snow/sand/gravel Minecraft behavior), never as
  rigidbodies; deterministic, tick-boundary, cheap.
- **Body interaction/compaction** — a body standing on loose material can
  transform the bottom layer in place (loose snow → packed snow under a
  boot) when solid backing exists beneath, leaving the rest loose. Pairs
  with the deferred "sinking in partials" movement rules (visuals.md §
  mixture road). Compaction-by-use is the shallow-time cousin of the
  deep-time compaction loop above.

**Bodies/sockets**: designed — see docs/design/bodies.md (body plans,
sockets, animation, all ratified state lives there).

**Inventory — DECIDED 2026-07-18: mass/volume with encumbrance, behind a
realism knob.** The foundations (every form has real mass/volume; carry
capacity; encumbrance effects) are built once; a gameplay-mode knob scales
them from Minecraft-breezy to hardcore sim (a full granite voxel is a
two-handed drag or a cart job). Development default: tuned way low so it
never impedes testing. Same knob doctrine as world extent.

**Fracture — DECIDED 2026-07-18: per-damage-type outcome weights**, not one
brittleness scalar. Each (material, damage type) row carries weights over
outcome forms (smash → mostly shards; careful dig → mostly chunks) — extends
S8's typed-resistance table, and makes tool choice narratively legible.

## Open questions

- Heterogeneous structure fill: what gameplay produces it (construction with
  mixed rubble? partial mineral replacement over deep time?).
- Packing actions: tool, skill, NPC labor integration.
- Stratification rate constants; does *active* sorting (alluvial deposition)
  get its own fast path near water?
- Pore size model: single scalar from structure material + porosity, or
  per-material pore spectra?
- Freeze–thaw: water packed in pores + cold → cracking/spalling (delicious,
  deferred).

## Pore packability — DECIDED 2026-07-20 (user)

**Rule:** a material may be packed into a host's pores iff
`filler_grain_size ≤ K_PORE × host_grain_size`, one constant `K_PORE ≈ 0.25`
(derived: ideal-packing interstices fit 0.22–0.41 D; the geotechnical filter
criterion lands at D/4–D/5). For heterogeneous structure fill the host grain
is the **minimum** grain among structural components (finest grains set the
throat). Enforcement lives in a single shared `fits_in_pores(filler, host)`
helper consulted by every *transport-time* depositing process —
`VoxelContents` stays pure volume accounting by design.

**Exemption:** formation-context emplacement (worldgen authored — magmatic/
diagenetic inclusion, e.g. olivine in basalt) bypasses the mechanical rule:
those crystals grew in place; the pore is representational (3d decision).
The rule governs infiltration, not genesis.

**Composability:** sieve resistance ≡ grain size (registry invariant), so
what-packs-in and what-sieves-out-first are the same axis by construction.

## DECIDED 2026-07-21 (user, journal/0049 tour) — sand first; partials-first emission

**Sand is the first loose material to spawn in the world, emitted as
partials** (dune/loess blankets express as full voxels + a partial-height
top, not quantized-away). The substrate is largely built: S8 loose
mechanics + journal/0010's dormant partial-height loose rendering. And the
direction it generalizes: **"everything should be spawned in partials"** —
world gen emitting partial occupancies as the default, with a **forms
mechanism across all systems** (partials in structures, loose volumes,
bedforms — variety that breaks the cube matrix). The forms pass is
design-pass-sized and Sequenced; this entry records the direction so no
slice quantizes away something partials should carry.

## The forms design pass — ratifications of 2026-07-21 (live session)

The pass opened as a main-session conversation (ROADMAP § Sequenced,
journal/0049 station 2). Decisions landed so far, each from the user in
conversation:

1. **Fractions come only from the ledger (RATIFIED).** Partial occupancy
   is the expression of *recorded* quantity (the record's metres → span
   eighths + remainder) and *recorded* variance — never cosmetic noise.
   This scopes the defect-jitter idea (ideas.md § rock is not monolithic)
   as **property-dependent expression of real record variance**, user:
   "i think i agree with you here re: property-dependent expression of
   real record variance." Partials-first emission is consuming the
   ledger, not decorating it.

2. **Sand is a FORM of an existing clastic material (DECIDED).** No new
   "sand" identity: sand is the loose form of the clastic-coarse member —
   "one material has one property [sheet]"; within-identity grain-size
   gradation is not modelled (revisit later if a wall appears; user: "go
   with forms. maybe it gets revisited at a later time but go with
   form"). **Known consequence, accepted:** loose clastic is currently
   textured identically to structural — indistinguishable in a cut face —
   which implies a later *visual* decision (form-dependent texture
   variants; files to the visuals road, not this pass).

3. **The soil model (user).** Anything with roots carries a **root
   material in STRUCTURE form** — porous, holding the soil in its pores.
   This is an anti-erosion model for free: loose soil with no matrix
   falls with gravity; the root lattice is what holds a slope. Soil
   placements come **from simulation, in both forms**: layers of
   structural dirt (most structure slots filled) with loose layers
   typically above, and packing downward into porous rock below (the
   packable-soil thread, ideas.md § soil is loose but packable).

4. **Grass is suspended — do not express it (DECIDED).** Grass/turf is an
   ecology question: a *state riding on the substrate materials* of a
   block / its loose eighths, deferred to the ecology pass. The forms
   slices express the materials the sim says are present and do NOT paint
   grass. **Appearance change pre-ratified by the user: "it'll be a
   mostly brown world for a bit."** (This names the heir direction for
   the surface-veneer stub — stubs.md § 2 — but the veneer's retirement
   is its own slice, not a side effect.) Turf presentation sketch,
   captured for later, NOT to be built now: **ligatures + anisotropic
   variants** — a seeded dirt material occupying loose eighths or packed
   pores whose green displays only on the top surface, side faces getting
   a roots-texture additive instead. Non-grasslike ground cover:
   thoughts suspended entirely for later passes.

5. **OPEN — the block-tier contract for partial voxels.** The user's
   naive assumption: the block "reports its honest fractions" (a derived,
   honest classification of contents). Wants the consumer audit — who
   asks what the block says, who might in the future — and a regression
   check once the contract is decided. Discussion live in-session; see
   the audit below when it lands in this doc or the journal.
