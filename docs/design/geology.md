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
*(Superseded on the ore axis by § Ore — DECIDED 2026-07-20: the full v1 ore
roster; engineering pass in ores.md. This paragraph's "one ore vector" was
the geology-backbone scope, not the ore roster's.)*
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
- ~~Open: hardness→erodibility coupling~~ **TAKEN 2026-07-20 (journal/0029,
  DECIDED user: "sequence erodibility first").** The coupling is built, off by
  default (`DeepConfig::erodibility`). The load-bearing design decision: erosion
  resistance is **agent-specific, never a single scalar** — a material's
  property sheet gained a `solubility` axis alongside its mechanical
  `extraction_resistance`, and `deeptime::lithology::LithoResistance` carries one
  resistance *per erosion agent* (abrasion / dissolution / frost-ice / wave), so
  a future carbonate can be mechanically competent (cliffs) AND soluble (caves)
  at once. Only the abrasion agent is wired to the (live) mechanical erosion;
  the other three axes are populated and dormant, waiting for their agents. This
  is what keeps karst/glacial/littoral implementable without a rewrite. Still
  open: caves/water-table interplay
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

> **SHIP UPDATE 2026-07-20 (journal/0026 — organic strata join the roster).**
> Three classes were added on the classes-as-contracts pattern, driven by the
> deep-time recorder's `Biofacies` axis rather than by the clastic env/energy
> rule: `dc:stratum/organic-coal` (coal), `dc:stratum/organic-peat` (peat), and
> `dc:stratum/organic-soil` (carbonaceous mudstone, filling both the `Soil` and
> `Retro` facies). Two roster decisions were made by MEASUREMENT and are worth
> keeping as precedent for the carbonate/metamorphic slices: (1) **charcoal got
> no member** because a fire bed averages ~3.5 cm and *none* of 158 310 of them
> survives the 0.9 m voxel — the § inclusions representation (pore/debris
> partials) is the honest form for a sub-voxel facies, and the resolution sieve
> is a real constraint on what any class can express; (2) **coal rank got no
> ladder** because lignite→anthracite is discriminated by burial depth at
> ~1–2 km and our record tops out near 100 m — the class documents its depth
> axis AS the rank axis so a later pack can fill it without moving a seam, which
> is the class-share invariant paying rent.

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

  *Refined 2026-07-19 (user, second session): the refuse-to-build stance is
  the interim, not the end state. Eventual shape for unfilled framework
  classes: world creation raises a named warning and skips them — a **loud**
  skip, which keeps the principle above intact (the rejected thing was
  silent dependence on pack coincidence, not degradation per se). Hard
  enforcement rides as-built until this is worth building; current shape
  precludes nothing (same checkpoint, different policy).*

## Ore — roster, representation, endowment, naming — DECIDED 2026-07-20 (user)

**The payoff-layer principle** (earth-processes § payoff layer) instantiated:
every v1 ore is the terminus of a process chain we actually simulate, and is
findable by a player who understands that chain.

**v1 roster (ratified):** coal (shipped) · **banded iron** (marine chemical
sediment on old craton chapters — chapter-gated once tectonic history lands)
· **bog iron** (wetland + biotic; the surface-accessible starter iron) ·
**redbed copper** (arid-basin sediments + buried organics; green staining in
red sandstone) · **orogenic gold** (veins in exhumed metamorphic cores →
eroded → **placers** downstream; the flagship end-to-end chain across
tectonic history + isostasy + hydrology) · **evaporites** (salt/gypsum,
closed arid basins + sea-level cycles). Deferred with their engines: tin/
tungsten (plutons), porphyry copper (intrusions), fault/hydrothermal veins
(faulting). No ore before its process.

**Representation — the accessory-inclusion pattern generalized (user).**
Ore is a MATERIAL inside a HOST, expressed in the existing eighths/partial
mix system exactly as the 3d accessory class does it (olivine precedent:
host rock in structure slots, inclusion in pore slots, face-dither
rendering). All three forms, most ores taking several:
1. **pore partials in a host block** — the canonical vein/disseminated form;
   the host stays itself ("the sandstone part of copper-in-sandstone is just
   a block with a mix of sandstone and ore partials");
2. **loose partials** — placer concentrates in gravels, the loose-material
   landscape form;
3. **entire blocks** — massive ore (coal already; rich cores where the
   process honestly makes them).
**Grade IS the eighths count** — ore "peeks through a little or a lot
depending on how many partials," so richness is visible, continuous, and
needs no separate grade mechanic. Raw-form color/texture stays realistic.

**Endowment (ratified):** soft guarantee at Medium+ — every v1 ore present
at least marginally, but quality/abundance honestly uneven, so "famously
rich gold country" exists without unwinnable worlds.

**Identification & naming (ratified):** system ids stay simple
(`dc:ore/...`). Display naming is owned by the future culture/language
layer (ideas.md § procedural languages — currently a sketch, not a full
doc). Where a metal has genuinely distinct raw forms (bog iron vs banded
iron), those are DISTINCT materials that later refine to the same metal —
identification knowledge is real knowledge. Prospecting is the skill;
up close, realistic partials peeking through the host are self-announcing.

## Expression of the ledger — DECIDED 2026-07-21 (user)

**Runtime generation is local refinement over the coarse deep-time ledger.**
The deep-time sim owns the history; the collapse/chunk tier does not invent a
parallel account of the world, it *collapses the ledger into local expression*.
Climate is not an input the surface tier consults independently — climate is
already **in** the history, and that is the proper relationship for it to have.

**Everything in the ledger must be expressed.** Where the record holds a fact —
provenance, P/T path, consolidation state, ecological state — the world a
player walks must show it. An unexpressed term is not a neutral omission; it is
the runtime silently disagreeing with the history that produced it.

**Inexpressed is permitted in exactly one case**: the thing that would express
it is not built yet. Then it must be **explicitly and loudly temporary** —
never a quiet gap, never a heuristic standing in for a term the ledger already
holds. This generalizes the **pack-degradation doctrine** (API.md, DECIDED
2026-07-19, same user): loudness is the invariant; no world content may
silently depend on something other than the recorded cause. The cost philosophy
ratified from the orogeny recon is the same rule stated for budgets —
*"coarsen the cause, never delete it and fake the appearance."*

**Consequence for placement rules.** Material placement keyed on present-day
slope/elevation/temperature is a *mock*, not a model, wherever the ledger holds
the answer. Slope and elevation are supplemental to provenance, never a
substitute for it. Known holdouts at the time of this decision, all filed to
ROADMAP Observed: the surface veneer rule (`collapse.rs::surface_sample`);
`exhum`/`t_crust` shipped explicitly as "the metamorphic-grade axes the collapse
tier reads" and consumed by nothing; the deep sim's `H` (regolith) plane
computed and then discarded by `DeepField`, with soil depth re-invented from
present-day precipitation.

**Holdout (c) closed 2026-07-21 — carry-`H` (journal/0053).** `DeepField` now
carries the regolith plane, and both the soil band and the clastic veneer budget
read it instead of rainfall. The slice's finding is doctrine-relevant beyond its
own scope: **`Σ(recorded unit thicknesses) ≡ H` exactly** — the strata record is
not a sample of the loose column, it *is* the loose column decomposed into beds.
So the surficial veneer is properly defined as the part of that column whole
voxels cannot resolve, amalgamated into one body — which is *form following
provenance* applied to the sub-voxel constraint (§ above): the thin beds are not
deleted, they change form into a surficial mantle, exactly as bioturbation and
creep do it in the field. The fill is mass-conserving against the ledger except
where the veneer's 8-voxel cap bites (stubs.md § 12).

**Form follows provenance.** What a material *is* and what **form** it takes —
deposited loose, partial in another's pore space, whole block, inclusion — are
both readings of the same history. A basement rock exhumed after a long
high-P/T residence should carry inclusions that formed at depth *because that is
what happened to it*. The sub-voxel constraint does not exempt this: a facies
thinner than the 0.9 m voxel (the charcoal precedent — mean bed 3.5 cm, **0 of
158 310** survived the sieve) is expressed as an **inclusion**, not deleted and
not faked as a layer. Changing form is how sub-voxel reality stays expressed.

**Per-voxel provenance is a first-class output** (user, same conversation):
it must be possible to query a voxel and read the ledger for its current
configuration — started as X, heat and pressure did Y, moved because of Z.
*(Integrator note, NOT yet ratified: taken literally this constrains every
stage to carry its reasoning forward rather than collapsing to a final value.
The framing is proposed, not decided — see ROADMAP.)*

### Genesis addendum — DECIDED 2026-07-21 (user)

**A stub is a candidate to be subsumed into a spine, system, or process — the
expresser.** Stubs are permanently legitimate in exactly one place: **genesis**
— the something-from-nothing that seeds the world (initial bedrock, plate
seeds, the level-0 corners). Everywhere else a stub lives — in spines, systems,
or processes — it is an instance of the loudly-temporary rule above: known,
loud, and holding a seat for the expresser that subsumes it. The registry of
these is `docs/design/stubs.md` (the stub inventory); an unlisted stub is a
defect in the inventory, not a licence.

### Enhancement doctrine — DECIDED 2026-07-21 (user, revising the framing above)

Simulation first — and **procedural tricks can be a kind of honest**. The
revision: a local procedural process is legitimate when it answers exactly one
question — *"what does this local point probably look like inside this
regional field of history?"* Local refinement is **enhancement** (the
spy-show enhance: resolving plausible detail inside a grainy-but-true image).
If it were feasible we would run a generative model conditioned on the
regional field; since it is not, deterministic procedures stand in for it.

Two requirements make an enhancement honest:

1. **Deterministic relationship with the larger field of facts** — same
   field, same answer, always; the enhancement is a pure function of the
   recorded history it refines.
2. **The local procedures are a reflection of the deep-sim processes** that
   produced the field — local process and deep process **must agree**.
   *In what sense they must agree is an explicitly open design question*
   (user: "i do not know in what sense they must agree, but on a high level
   that appears clear to me") — recorded as the guiding constraint, to be
   sharpened slice by slice rather than resolved here.

Consequence for any sub-deep-cell detail (the S13 recalibration included): a
local rule is honest **to the degree it is conditioned on the field** — jitter
scaled by recorded provenance roughness is conditioned; a field-blind constant
is not. "Fake the appearance" (the anti-pattern in the cost philosophy) now
means precisely: detail with **no deterministic tie back to the field**.
