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

## DECIDED 2026-07-20 (user) — one quantity, two regimes

**Water is ONE conserved quantity existing in two regimes, not two
systems.** User ratified: "one quantity, two regimes — that's quite right."

- **Bound water** — occupying pores and the empty eighths of loose
  partials (materials.md's existing fluid model). Moves slowly, by
  settling and permeability. This is groundwater.
- **Free water** — occupying open space. Moves fast, by flow. This is
  rivers, lakes, waterfalls, the sea.

**Rationale (the argument that carried it):** every phenomenon named in
the notebook is a **transition between the regimes**, not a behaviour of
either one. Absorbing into the next porous layer is free→bound. A spring
is bound→free. Dripping from a ceiling is bound→free at low rate into a
void. Waterlogged debris and quicksand are bound at saturation. If the two
were separate systems, each of those would need bespoke coupling code;
as one quantity they are the same transition read in different settings.

### Consequences that follow immediately

1. **The water table is READ, not modelled.** It is the top of the
   saturated zone — where bound water reaches saturation and meets open
   space. Nothing stores "the water table"; it is a query over the
   saturation field. This is why S9 could classify it as a *bounded
   relaxation* (haloable, C-refinable) rather than an advective field: it
   equilibrates locally and never has to be traced from a source.
2. **Vadose vs phreatic falls out for free**, and with it cave
   morphology. Below the water table is phreatic (saturated — where
   dissolution happens, the orogeny-proven karst regime). Above it is
   vadose (air-filled — where dripping, flowstone and speleothems happen).
   The same conduit changes character when the table drops past it, which
   is exactly what real caves do. **No cave-morphology system is needed;
   it is the regime boundary moving through rock over time.**
3. **Aquifer and aquitard are material facts, not authored features** —
   permeability comes from the property sheet, and the loose-vs-packed
   soil contrast (ideas.md § soil is loose but packable) supplies the
   sharp permeability contrast that makes an aquitard an aquitard.
4. **S10's waterlogging proxy has a defined retirement**: waterlogging
   becomes "the water table is at or near the surface here", read from the
   field. Biology reads the real quantity; the proxy is deleted.
5. **Conservation is the invariant to protect** — whatever the two regimes
   use for representation, the transition between them must neither create
   nor destroy water. That invariant is the natural test target (cf. S10's
   mass ledger `Δ(ΣR+ΣH) == uplift + biotic`).

### NOT decided by this entry

The *representation* of each regime (the bulk-flow octree, the saturation
field), the event vocabulary, timescale ownership, sub-resolution water,
and capillary action all remain open below.

## The hypothesis S11 tests (2026-07-20) — persist bodies, derive voxels

Not decided; this is the model the spike is dispatched to falsify or
support. It arose from the user breaking an earlier, wronger claim.

**The wrong claim (assistant, retracted same conversation):** "water is a
function until an edit makes the function wrong — re-derive everything
from current geometry." The user broke it with two cases: a km-deep chasm
dug from a lake bottom (standing at the bottom, how does anything know
water reaches here?) and a km-long channel dug tangent from a river (how
does the far end know a source is at the other end?).

**Why it was wrong:** the two regimes have different *locality*.

- **Bound water is local.** Saturation relaxes against neighbours; S9's
  "bounded relaxation" classification holds. Derivable within a halo.
- **Free water is CONNECTIVITY, which is not local at any radius.**
  Whether water reaches a place is graph reachability through geometry the
  player invented. No halo sees it; re-deriving per frame would mean
  flood-filling the world.

**The hypothesis:** the user's own event-driven instinct is the answer —
*the graph is the memory*. Nothing asks per frame whether water could be
somewhere; a breach event links a volume to a source, and the link is only
revisited when another event touches it.

- **Persist BODIES and their links** — volume, level, inlet(s), outlet(s).
  A flooded cave is one node. A km channel joined to a river is a link on
  the river's body. Sparse; thousands cost nothing.
- **Derive VOXELS** — per-voxel water comes from the body's level plus
  local geometry, like a lake surface. Dense but free.
- **Persistence rule**: the body graph is world state (like an edit) and
  must survive unload/reload. Voxel water must not be persisted. Dropping
  a body node loses the one thing no derivation can reconstruct.
- **Storage rule (sharpened)**: store only what the derivation cannot
  predict — transients in flight, and bodies in containers the derivation
  says should be dry (a bucket poured into a sealed stone basin).

**Ocean/sea scale (user, at dispatch)**: a sea is not a finite volume to
fill and drain — breaching it must not empty it. This suggests bodies need
a character distinction (level-pinned reservoirs vs finite volumes), which
S11 must exercise rather than assume.

## Stance on conservation (user, 2026-07-20) — lossy is allowed, and honest

Stated while S11 was dispatched, before its results:

> i appreciate the fight for water volume conservation and present
> history. at outset i didn't believe it actually possible even if it's
> obviously ideal. ... i anticipate the answer is probably "where we
> compromise" and shades of procedural tricks. it's going to be lossy /
> not fully conservative of volume - neither is reality (the water cycle
> is half gaseous) - and ultimately players get to exploit the gaps in the
> sim, and this won't be the only one. may come with the territory.
> shouldn't stop us from thinking up genius architectures, but still. and
> we'll use every perf hack in the book, which goes for the whole game.

Recorded as project stance:

- **Perfect volume conservation is the ideal, not the requirement.** The
  expected landing place is "where we compromise" plus procedural tricks.
- **Players will exploit the gaps, and that comes with the territory** —
  this will not be the only such gap in the game.
- **Every perf hack in the book is sanctioned**, for water and generally.
- **But the compromise is the LAST step, not the first**: "shouldn't stop
  us from thinking up genius architectures." Design for the honest
  mechanism, then compromise deliberately where measurement forces it.

**Assistant observation (PROPOSAL, not ratified):** the user's gaseous
aside may be load-bearing rather than rhetorical. *The water cycle is half
gaseous* means evaporation/transpiration is a **real sink, not an excuse**.
If the liquid model carries an honest evaporation term, then "lossy"
stops being a compromise and becomes "conserved with a modelled sink" —
a much stronger position, and it gives the slop a principled home rather
than scattering it through the implementation. Note S10's biology already
demands water (transpiration is implicit in its moisture tolerances), so
the sink has a consumer waiting. Worth deciding WHERE the loss lives
before S11's results tempt us to hide it in whichever mechanism is
convenient.

### S11 ANSWERED THIS (2026-07-20) — docs/spikes/S11-results.md, journal/0028

Spike complete; **agent recommendation GO**, four calls NEEDS RATIFICATION
(ROADMAP § Sequenced). The hypothesis above survived, with three amendments
the measurement forced:

- **Bound water is local**: halo **4–11 cells** at a bounded post-edit budget,
  **0–6** for the player-visible integer water table, geometric decay, no
  advective spikes. S9's "bounded relaxation" classification now has a number,
  and it is *smaller* than erosion's 16–24. A sharp aquitard makes it **more**
  local, not less — the loose-vs-packed contrast (§ consequence 3) is the best
  case, not the risk case.
- **Amendment 1 — connectivity does most of the graph's work.** Two bodies in
  the same air component *are* one body; links only exist *between* components
  and measured 0–1 in every scenario. The km channel joined to a river is not
  "a link on the river's body" as sketched above — **it is the river**. See
  corrections #14.
- **Amendment 2 — the ocean's character distinction is necessary AND cheaper.**
  A finite sea breached into a large void drops 13.18 m. A pinned sea needs no
  capacity curve at all, so its derived cost goes 415 ms → 0.0 ms. Pinning is
  the fast path, not a concession.
- **Amendment 3 — the dense structure is the DERIVED index, not the persisted
  graph.** Bodies are ~20 bytes and do not grow with edits (1 body through
  1 624 edits; ceiling 217 = the component count). What changes on every edit
  is the connectivity index, and that is a pure function of geometry —
  never persisted, proven by byte-identical reload from 39 bytes.

Still open below and untouched by S11: the bulk-flow octree (though note free
water in equilibrium measured as **static data with a level**, which is what
§ 2's "creates no new blocks so long as its outlet connects" predicted),
sub-resolution water, capillary action, cave families, deep-time water field
placement.

## DECIDED 2026-07-20 (user) — S11 ratification calls

S11 returned GO with four calls. Answered:

1. **`Pinned` vs `Finite` reservoirs: PINNED, default ON, and it must be
   TOGGLEABLE.** User: "i guess pinned but we have to be able to turn it
   off and see what happens. simple as. default on." So a sea holds its
   level rather than draining when breached — but the finite behaviour
   stays reachable as a switch, because watching an ocean drain into a
   chasm is worth being able to see. (Note the measurement agrees with the
   world model: pinning needs no capacity curve, taking that derived cost
   415 ms → 0.0 ms.)
2. **The ~12-cell halo: it is a KNOB, not a constant.** User doctrine,
   stated generally: **"where there's a cell range, there's a knob. we
   don't know what may be more perf in the future when other unbuilt
   systems are also running. knob now."** Applies beyond water — any
   measured radius/halo/range should ship adjustable rather than baked.
3. **Body identity across merges** — RECLASSIFIED as an engineering
   choice, not a user call (see below). Rides as-built; revisit if and
   when bodies acquire durable identity (a named lake, a quest site), at
   which point it becomes a design question rather than an implementation
   one.
4. **Whether the connectivity index is truly never persisted** —
   likewise engineering. It is derived state and S11 proved byte-identical
   reload without it; it stays underived-and-unpersisted unless a
   measurement says otherwise. Shares the open question with the far-field
   summary store, and should be decided with it rather than separately.

**Process note (assistant error, recorded deliberately):** calls 3 and 4
should never have been put to the user. The ratification rule covers
game feel, art, and scope forks — user-owned choices. Body identity across
merges and index persistence are implementation questions with measurable
answers; forwarding an agent's "NEEDS RATIFICATION" list without filtering
it pushed engineering decisions onto the user. The integrator's job is to
triage that list, not relay it.

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
