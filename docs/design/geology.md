# Geology — design seed

Status: extensibility backbone agreed 2026-07-18; the content half (which
classes and passes make v1) is under discussion. Idea quarry: orogeny's
stratigraphy-v1 (journal 0045 there) — in-progress and Minecraft-constrained;
mine it, don't copy it.

## Backbone: everything is a pack

**Processes bind to classes, not instances.** Vanilla geology is just the
first geology pack.

- **Classes are contracts** — parameter schemas, not labels. Registering a
  material into `stratum.igneous-extrusive` or `ore.hydrothermal` means
  implementing that class's schema (formation T/P window, abundance weight,
  vein habit, hardness/erodibility, …). Validated at define time by the
  dc-api registry (namespace-owned, schema-checked — S5 machinery).
- **Selection defends determinism**: a pass asks a class for a member fitting
  context C — fitness (formation-condition distance) × abundance × seed.
  Members canonically ordered by namespaced id (registration order never
  changes worlds); abundance normalized within class (mods diversify worlds,
  never inflate them). Consequence for the seed-stability policy: adding
  materials changes ungenerated regions of existing worlds — accepted,
  DF-like, must be documented player-facing.
- **Processes are plugins too.** A creation vector (glacial till, evaporite
  basin, ley-line crystallization) registers as a pass declaring: phase
  (pregen epoch / lazy-collapse contributor / runtime-derived), what it
  READS, what it WRITES. Declared reads/writes let the pipeline topo-sort
  passes and detect cycles — the geology→soil→ecology→history coupling-order
  problem becomes a graph problem instead of a hand-maintained list.
- **Context vocabulary**: the queryable axes class conditions bind to —
  depth, T/P history, climate-at-deposition, tectonic province, fault
  proximity, host class (S7 already produces several). OPEN: plugin-published
  axes (a pass publishing "glacial coverage" as a plane later passes/biomes
  read) — maximal power, hardest contract; leaning core-axes-only for v1,
  extensible later.
- The pattern generalizes: biomes-as-diagnosis consume the same axes;
  ecology niches are classes; history could take cultural vectors. Roles +
  contexts + processes, all registry data.

## v1 content (TO DECIDE — the ongoing conversation)

- Candidate classes: clastic sediment (fine/coarse), chemical sediment
  (carbonate, evaporite), igneous intrusive/extrusive, metamorphic grades,
  ore (hydrothermal / placer / magmatic), regolith/soil.
- Candidate passes: S7's pipeline extended with strata *recording*
  (deposition logged per-cell with climate-at-deposition tags — the orogeny
  0045 pattern, which is also our materials deep-time loop); intrusion;
  metamorphism by burial/exhumation; karst; ore-genesis vectors; glacial
  (later). Placer ores fall out of S8's alluvial grain-sorting almost free.
- Open: hardness→erodibility coupling; caves/water-table interplay
  (aquifers in porous stone — materials.md); deep-earth exotic classes for
  the trog layers (depth is a first-class axis for us in a way surface games
  never had).
