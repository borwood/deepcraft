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

## Storage: the open dragon (spike S8)

Naive cost is ~8 × material-id per voxel — unaffordable as a base grid.
Survival strategy: eighths are coarse quantization; unordered multisets
canonicalize (one state for any ordering); deposition is spatially correlated
so mixtures repeat regionally. Plan: intern mixture states in a region table;
chunk palettes index into it; rich voxels live in a format-v1 sidecar so
debris-free terrain pays nothing. **Empirical question**: do realistic
deposition processes stay palette-friendly, or do smooth gradients explode
distinct-state counts? If gradients blow up, snap mixtures to a curated
recipe set (possibly better game design anyway — nameable, learnable strata).

S8 also owes: the LOD downsample rule for mixed voxels (S3 made the rule
pluggable), angle-of-repose settling on dirty voxels, and the render-blend
prototype.

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
