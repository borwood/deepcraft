# Water — the field notebook

Status: **OPEN FIELD NOTEBOOK, 2026-07-20.** Field-notebook first, per the
earth-processes method (ROADMAP: "Design doc before any code"). Nothing
here is decided unless it says DECIDED with a date. This file exists to
catch the thread as it is spoken — the user lost a long write-up to an
accident once already; capture is now immediate and verbatim-first.

Ratified 2026-07-19 as a design pass: groundwater as "another dimension
for the flow to go"; water table / aquifers (S8 per-voxel porosity is the
waiting substrate); ponds and sub-resolution water; visible/flowing water
(couples to PBR-2 water); lakes and inland seas already implicit as deep-
tier flooded basins with known spill levels. **Groundwater ↔ CAVES
flagged by the user** — speleogenesis as the eventual cave story.

---

## User notebook, 2026-07-20 (verbatim capture, first pass)

Captured as spoken; the user was mid-writing and flagged more to come.

> most important things to anticipate: water -> caves. karst, littoral,
> erosional, glacial. also bulk water flow -> octrees of continuous-flow
> (creates no new blocks as long as its directional outlet connects to
> receiving flow volumes). in minecraft a water block is immobile, it
> creates flows on its empty faces. our water is mobile and if
> continuously fed, immobile + a producer of new blocks or a sustainer of
> existing bulk flows. better if bulk flows are event driven than
> continuous check it's being fed.

### 1. Water → caves; four speleogenesis families

Caves are to be **the record of water**, not noise. The user names four
families, which are four different *agents*:

- **Karst** — dissolution of soluble rock.
- **Littoral** — sea caves; wave energy at a coastline.
- **Erosional** — mechanical carving; undercut channels, abandoned
  conduits.
- **Glacial** — ice as the agent.

Each is (agent × rock × time), the same shape geology.md already uses for
rock formation. This *replaces* today's caves, which are S1 noise carving
(filed in ROADMAP as such).

### 2. Bulk flow as octrees of continuous flow

- A body of moving water is represented as an **octree of continuous-flow
  volumes**, not as a per-voxel cellular automaton.
- **It creates no new blocks so long as its directional outlet connects to
  receiving flow volumes** — i.e. a flow in equilibrium, with somewhere to
  go, is *static data*. Cost is paid on change, not on existence.

### 3. Our water vs Minecraft's water (the user's contrast)

- **Minecraft**: a water block is **immobile**; it spawns flow blocks on
  its empty faces. Water is a cellular automaton over blocks.
- **Ours**: water is **mobile**. And *if continuously fed*, it becomes
  **immobile + a producer** — either producing new blocks, or sustaining
  existing bulk flows.
- So a voxel of water has (at least) two regimes: transient/moving, and
  fed/steady-but-productive. Steady state is not "nothing happening"; it
  is a maintained condition with an upstream cause.

### 4. Event-driven, not polled

- **Bulk flows should be event driven** rather than continuously checking
  whether they are still being fed.
- Implication (assistant's reading, to confirm): the flow network is a
  graph carrying invariants; a change — an edit, a blocked outlet, a newly
  opened space, a supply change — invalidates specific nodes and fires
  re-resolution there, rather than every flow polling its own supply.

---

### User notebook, 2026-07-20 (second pass — fluid as a material state)

> our water can fill loose partials and structure pores - fluid
> (generalized) can do so and can *settle* (prior discussion on settling
> heavies in loose mixes: fluid obeys different rules than mere weight).
> settling: attempting to leave one block and occupy spaces below.
> drippin from ceiling, or absorbing into the next porous layer.
> generated world would have such things - groundwater etc - and in the
> deepsim time, one imagines water moving through porous limestone eating
> it away and depositing it elsewhere: also worth considering if we could
> ever model capillary action - a water partial moving along the *ceiling*
> or *side face* before stochastically dropping to the floor. more
> complication - would imply a whole other state for partial fluids, but
> flashy.
>
> features: galciars, lakes, lagoons, oceans, inland seas, waterfalls, etc
> etc. literally whatever you encounter on earth.

Unpacked:

- **Fluid is a generalized material state, not a water special case.**
  It fills **loose partials** (the empty eighths of a partially-filled
  loose voxel) *and* **structure pores** — both already in materials.md.
- **Settling is the fluid's own rule**, and explicitly **not** the solids'
  rule. The existing solid mechanism is `settle_energy = sqrt(grain_size ×
  specific_gravity)` (journal/0007 — the placer falls out of it). The user
  names that as the prior discussion and says **fluid obeys different
  rules than mere weight**.
- **Settling defined**: *attempting to leave one block and occupy spaces
  below*. Two named expressions of the same rule — **dripping from a
  ceiling**, and **absorbing into the next porous layer**.
- **Deep-sim consequence, stated**: water moving through porous limestone
  **eating it away and depositing it elsewhere**. That is dissolution +
  re-precipitation as ONE transport process — i.e. the karst conduit and
  the speleothem are the same mechanism read at two ends. Pairs exactly
  with the orogeny-proven recipe below (solubility read from the recorded
  volume).
- **Capillary action — speculative, flagged by the user as "flashy"**: a
  water partial travelling along a **ceiling** or **side face** before
  **stochastically dropping** to the floor. The user notes the cost
  honestly: it would imply **a whole other state for partial fluids**
  (adhesion/attachment, not just occupancy). Filed as wanted-if-affordable,
  not scoped.
- **Feature target: "literally whatever you encounter on earth"** —
  glaciers, lakes, lagoons, oceans, inland seas, waterfalls, and the rest.
  This is a completeness statement about the default pack (see ideas.md
  § content packs and the default world's scope) more than a feature list.

## PRIORS ALREADY IN THE CORPUS (swept 2026-07-20 at the user's prompt)

**Read this section before proposing anything.** The user flagged a
discovery failure: the assistant opened this notebook and "observed"
several things the corpus had already recorded, some of them years-deep in
prior thinking. What follows is the sweep, so the notebook starts from what
is known rather than re-deriving it.

- **Architecture, DECIDED 2026-07-18 (ARCHITECTURE.md)**: "Depth is a
  worldgen axis: deep-time geological history generates literal strata;
  **caves/aquifers/lava at region scale, not per-column noise hacks**."
  Caves-from-process is not a new direction — it is the standing
  architectural commitment, and today's S1 noise caves are the placeholder
  it already disowns.
- **Fluid representation is already settled at the material tier**
  (materials.md): "**Fluids occupy empty eighths and pores** (aquifers in
  porous stone, waterlogged debris, quicksand — free consequences of the
  model)." So water in rock is not a new storage question: it is pore
  occupancy in the S8 model, and aquifers/quicksand/waterlogging were
  named as falling out of it.
- **earth-processes.md § 8 is literally "Groundwater, karst,
  hydrothermal"** — an existing process entry with reality/signatures/sim:
  dissolution caves on water tables; mineral-charged fluids depositing
  veins/ore along fractures and contacts; sim = water-table interplay
  (materials.md aquifers) + vein/inclusion emplacement via pore partials +
  fracture networks from the deformation history. Related entries: § 2
  weathering already ends "**karst on carbonates**"; § 4 transport already
  lists **ice** as one of the four agents (unsorted till, striations,
  U-valleys); § 6 already has **ice ages rewriting erosion regimes**.
- **geology.md already sequences it**: "Next after v1: chemical sediment
  (carbonate) — **caves-in-carbonate-on-water-tables is orogeny-proven and
  gameplay-rich**". Candidate passes already include **karst** and
  **glacial (later)**. Open question already filed there: "caves/water-table
  interplay (aquifers in porous stone)".
- **The karst mechanism is PROVEN in the orogeny mod** (orogeny-recon
  § quarry): "**Caves read solubility from the recorded volume** (conduits
  live inside coalesced carbonate bodies **on a subdued, perched water
  table**) — solubility is a **volume property, not a surface map**.
  Determinism hygiene: **judge the whole volume at the same scrambled
  position**." This is a working recipe, not a sketch, and it already
  answers the "how does karst pick where to dissolve" question.
- **S9 already classified the water table's computational character**
  (S9-results.md § parallelism table): "Groundwater / karst / hydrothermal
  (§8) — **relax** (water table mildly regional); solubility from the
  recorded volume (local); **water table a bounded relaxation**." So the
  water table is a *relaxing* field, not an advective one — which means it
  is **haloable and C-refinable**, unlike drainage, which S9 measured as
  the single advective wall.
- **S10 has already built a water-table PROXY** (2026-07-20): waterlogging
  modelled separately from rainfall — climate moisture plus bonuses for
  sitting near base level and for receiving upslope drainage — because peat
  needs standing water, not rain. It works (coal swamps land on lowlands).
  When the real field exists, **biology should read it and the proxy should
  be deleted**; two systems privately approximating one physical quantity
  is the thing to avoid.
- **Packable soil is upstream of this** (ideas.md, user 2026-07-20):
  porosity/permeability differ sharply between loose and packed soil, so
  the soil-consolidation model feeds aquifer behaviour directly.

### What survived the sweep as genuinely new (assistant, PROPOSALS)

- **Erosional caves may be the cheapest first family** — deep time already
  computes drainage every epoch, so paleo-channels exist in the record; an
  abandoned conduit is a former channel the water table later dropped
  below. Possibly derivable from data already held (the same shape as S10's
  coal: already in the record before the world could contain it).
- **Bulk-flow octrees may share substrate with FF2b's volumetric summary
  octrees** (SVDAG/Aokana candidate) — ROADMAP already pairs FF2b with "the
  caves/underground thread of the water design pass", but the shared
  *representation* question has not been asked. Worth checking before
  either is built.
- **Event-driven flow invalidation looks like the existing dirty rail**
  (edit → dirty set → targeted re-derivation; used by remeshing, collider
  tiles, owed far-field summaries) with a different payload.
- **Order-independence must be designed in, not discovered** — S9b won
  exactly this fight for flood/erosion via a scatter→gather reformulation
  with byte-identity proven. The orogeny hygiene rule ("judge the whole
  volume at the same scrambled position") is the same lesson from the other
  direction.

## Open questions (carried, unanswered)

1. What are the *encounters* — the moments a player meets water that no
   other game gives them? (Asked; the user's answer is pending, and the
   notebook should be driven by it.)
2. **Sub-resolution water** (a trickle, a damp seam, a puddle) — pore
   occupancy covers water *in* rock, but a film or a rivulet in open space
   is not obviously an eighth.
3. **Bulk-flow octree vs the per-voxel pore/saturation field**: one system
   at two scales, or two systems with a boundary? (Partially pre-answered:
   pores are settled material-tier machinery; the octree is the open half.)
4. Does the near-field flow network persist, or re-derive from the water
   table on load? (Same family as the owed far-field summary persistence.)
5. Where does the deep-time water field live relative to the A/C tiers —
   given S9 says it *relaxes* and is therefore haloable?
6. **Which cave family ships first**, and does it wait on carbonate?
   (Karst is orogeny-proven but carbonate-gated; erosional may be nearly
   free; littoral needs wave energy; glacial needs ice as an agent.)
