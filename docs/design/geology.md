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

## v1 content — DECIDED 2026-07-18: minimal-but-complete

v1 proves the class machinery end-to-end with the smallest honest set:
**clastic sediment + igneous (intrusive/extrusive) + one ore vector**
(placer is the cheapest — it falls out of S8's alluvial grain-sorting).
Next after v1: chemical sediment (carbonate) — caves-in-carbonate-on-
water-tables is orogeny-proven and gameplay-rich — then metamorphic grades
and further ore vectors.

- Full candidate-class roster (for the sequence, not v1): clastic sediment
  (fine/coarse), chemical sediment (carbonate, evaporite), igneous
  intrusive/extrusive, metamorphic grades, ore (hydrothermal / placer /
  magmatic), regolith/soil.
- Candidate passes: S7's pipeline extended with strata *recording*
  (deposition logged per-cell with climate-at-deposition tags — the orogeny
  0045 pattern, which is also our materials deep-time loop); intrusion;
  metamorphism by burial/exhumation; karst; ore-genesis vectors; glacial
  (later). Placer ores fall out of S8's alluvial grain-sorting almost free.
- Open: hardness→erodibility coupling; caves/water-table interplay
  (aquifers in porous stone — materials.md); deep-earth exotic classes for
  the trog layers (depth is a first-class axis for us in a way surface games
  never had).

## Formation context — DECIDED 2026-07-19 (correcting a v1 shim)

> **SHIP UPDATE 2026-07-19 (3e-1 — the shim is dead for deep strata).** The
> deep-time A tier is now always-on in `Pregen::run` (a declared pregen pass:
> reads Elevation/Provenance/Climate, creates DeepElevation + DeepStrata), and
> the collapse clastic pass reads its per-cell record as the **at-deposition**
> formation-context source. A depositional stratum's member fitness now
> evaluates against the environment *measured when it was laid down* — paleo
> precipitation from the recorder's aridity tag, temperature from the column's
> latitude (a paleo-temperature curve is a later 3e slice), burial depth from
> the overlying record — **not** the year-zero climate. So the shim below is
> obituary for deep strata: year-zero climate remains legitimate **only for the
> active surficial veneer** (soil, still-forming alluvium — which the pass keeps
> depositing on top, and which the placer reworks). The recorded units sit
> between the igneous basement and that veneer, so a cut face reads marine mud
> under arid fill under recent veneer — the record of a landscape that ran.
> Igneous stays province/depth-driven (already weather-blind). See journal/0015,
> `deeptime/field.rs`, `geology.rs::deposit_deep_history`.

The v1 implementation evaluates ALL class fitness — including igneous —
against the column's **present-day** temp/precip (plus a depth constant).
This is a documented shim, ratified as acceptable **only for the surficial
veneer** (year-zero deposition under the year-zero climate is the truth for
recent sediments; the fine/coarse choice within a deposit is fluvial-energy
sedimentology and unimpeachable). It is WRONG for everything else and must
not ossify:

- **Formation context must become epoch-indexed.** Deposition events draw
  paleo-context from the deep-time story; igneous fitness consumes tectonic
  setting + emplacement depth (never surface weather); metamorphism
  consumes the P/T path from burial/exhumation. The architecture already
  supports this — classes declare their own param contracts, the strata
  record tags events with context, the pass graph is where a paleo-context
  provider slots in. Which context flows is the fix, not the machinery.
- **Near-term task**: strip climate windows from the igneous class sheets
  (fitness becomes province/depth-driven — the pass already gates on
  `Provenance`).
- The paleo-context ("T/P history") axis joins the sequenced deep-time
  work (S2 checkpoint facts, pregen history). Inspiration source: the
  orogeny repo's earth-process sim (deep-time material movement over
  eons) — mine for mechanism, not as a map; we fully own gen here.
- Note: the chunk-line family cutover (ROADMAP Observed) is an orthogonal
  quantization artifact — it would occur under perfect paleo-context too.

## Roster, inclusions, and unfilled slots — DECIDED 2026-07-19

- **Rich vanilla mineral roster.** The default pack does not shy away from
  a wide Earth-mineral/metal/gem set. Parity is free by construction:
  properties-not-recipes (any metal makes the armor, any gem fills the
  ring — form archetypes × property sheets, DF-style), and the S8 mixture
  cap binds **per-locale co-occurrence, not global roster size** — fitness ×
  normalized abundance keeps local k small no matter how wide the pack.
  Legibility of many similar materials is the knowledge system's job
  (inspection, thoughts), not the recipe system's.
- **Inclusions are pore partials.** The standard representation for
  accessory minerals and in-situ ore: a voxel whose structure slots are the
  host rock and whose pore slots carry the accessory (olivine in basalt,
  vein ore in country rock) — the same eighths machinery the placer already
  uses (gold-dust inside sandstone events). Veins, phenocrysts, and gem
  pockets are one representation; `extraction_sequence` already recovers
  eighth-wise; visuals stay subtle by construction (§ ore legibility,
  visuals.md). Post-v1 pass extension: igneous/metamorphic passes emit
  accessory pore fill via class selection.
- **Unfilled slots cannot break generation — enforced at two layers.**
  (1) Define-time: a gen pass that *consumes* a class it also *introduces*
  must register at least one fallback member in the same registry batch —
  a pass is a pack; the magic-terrain-without-magic-materials mistake is
  structurally impossible. Fallback members are ordinary members (canonical
  order, normalized abundance), so later packs diversify, never invalidate.
  (2) World-build-time: the pipeline refuses to build if any class a pass
  selects from has zero members, naming pass and class — same
  named-culprit philosophy as the cycle/ambiguous-writer rejections.
  Silent skip / "other rules fill gaps" is rejected on principle: world
  content must never be a function of installed-pack coincidence; two
  same-seed worlds differ only by declared pack differences, loudly.
