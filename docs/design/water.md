# Water — the field notebook

> **⚠ READ [`flow.md`](flow.md) FIRST (ratified 2026-07-25).** The **river /
> drainage / channel-expression half** of this notebook is **superseded**: flow is
> now one process recorded as **flux on 3D faces + facts on strata**, the receiver
> tree is retired (it cannot represent divergence — no deltas), and the carve is a
> **mass budget**, never a drawn shape. **What survives and is load-bearing:** the
> DECIDED two-regime model (free/bound — now unified as an *occupancy*, i.e. an edge
> on the form-transition graph), *persist bodies / derive voxels*, the measured S11
> results, corrections #14 (**bodies-not-links; a spring is a derived outlet**), and
> the § Session-capture karst analysis (**zero cave-specific code at the present
> tier**). The **S14 spike is superseded as posed** — face flux answers its
> seamlessness half structurally.

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

## Session capture, 2026-07-21 — caves ↔ hydrology, one system (off-thread session; PROPOSALS except user words)

An off-main-thread session audited chunk generation ("what does a chunk
know at generation time") and ran the caves/hydrology integration question
against the corpus. The user judged the analysis "likely sound" — captured
here so the thread survives the session boundary. **Nothing below is
DECIDED**; user-owned calls are flagged.

**The integration shape (per tier, all reusing canon spines):**

- **Deep time**: karst enters as *another agent on the one water field* —
  dissolution = solubility (from the recorded volume, the orogeny recipe)
  × flow (the same drainage the erosion step already routes) × phreatic
  residence (below the epoch's table). It writes per-cell void/conduit
  capacity into the record, and **feeds back into flow routing** —
  conduit capture is what produces losing streams, dry valleys over cave
  systems, springs at base level; without the feedback you get rivers
  flowing intact over networks that should have swallowed them.
- **Karst is stream-power's chemical twin**: the erode-transport-deposit
  architecture of `erosion.rs` in the solute phase (capacity = solubility
  × flow, not slope × discharge). Speleothem/tufa is the deposit end of
  the same transport ("eating it away and depositing it elsewhere", § 2
  above). Mass ledger extends: `Δ(ΣR+ΣH) == uplift + biotic − solution
  export` — dissolved load reaching the sea is a real sink, same
  epistemic status as the evaporation observation.
- **Collapse**: recorded conduit capacity refines (bounded, addressed,
  scrambled-position judged) to **void intervals per column**; the
  collapse-time water table decides which voids are flooded at year zero
  — the genesis handoff that seeds the body graph.
- **Present tier — the punchline: S11 needs zero cave-specific code.** A
  cave is an air component in the connectivity index; a flooded conduit
  is a body; a spring is a body outlet where the table meets the surface
  (derived, not authored). Conduit-vs-matrix flow — the thing that would
  wreck a naive saturation field — is exactly the free/bound two-regime
  split, rediscovered by karst hydrology's own conduit/matrix
  distinction.

**Genuinely new findings (work-shaped, need sequencing):**

1. **Two drainage opinions exist.** Pregen cell hydrology and the deep
   tier's per-epoch drainage compute the same physical thing at two
   resolutions — the same disease as two water tables, one level up.
   Proposed: deep drainage becomes *the* spine, cell graph derived or
   demoted to initialization. Consequence with teeth: the history pass
   sites settlements against pregen rivers (order: tectonics → climate →
   hydrology → history → deep-time), so it would need **resequencing
   after deep time** to site against the final eroded world's drainage.
   Pipeline surgery — decide deliberately, not mid-karst-slice.
2. **The deep sim's per-epoch drainage is computed and discarded**
   (`DeepField` keeps only `surf` + `strata`). Paleo-channels are the
   erosional-cave feedstock ("possibly derivable from data already held",
   § sweep) — needs a recorder axis (channels, and the table per
   chapter). Cost unmeasured; estimate is hundreds of KB, and the eolian
   record already carries a memory FLAG, so measure first.
3. **The spike that gates the river-primitive retirement:** can regional
   drainage refine at landform resolution under *coarse boundary fluxes*
   with a measurable halo? S9 named drainage the single advective wall
   but never tested the boundary-conditioned form. If yes → channels
   become part of the eroded surface and the RiverSeg chain dies
   honestly. If no → some channel inscription survives at collapse, and
   the honest framing is a refinement OPERATOR (lattice-midpoint class
   machinery), never a persisted primitive.
4. **Column-model verdict (proposal): don't kill ColumnRec — kill the
   contract's growth.** A per-column interval log is fully general for 3D
   solids (caves = void intervals beside deposition events; real
   stratigraphy is described this way; orogeny field-proved the shape at
   147k columns). What must stop is new consumers baking in "one height,
   solid below" (the authority edit-ceiling, far-mesh column reasoning,
   surface scans accrete on it now). Proposed: a one-paragraph
   ARCHITECTURE decision naming the interval log as the target fill
   contract, current single-height as its degenerate case, riding
   as-built per no-bandaid until the fork. **User-owned scope call.**
5. **Water-table placement (open question 5, sharpened):** likely BOTH —
   a coarse paleo table per chapter in the deep record (cave morphology
   and speleothems want history) AND a present-tier locale relaxation
   over the strata's permeability (S11's 4–11 halo makes it affordable).
   Needs the spike, not an assumption.

**Acceptance-test proposal for the first karst slice:** the encounter is a
stream that vanishes into a sink and a spring that returns it downstream —
make *that* read-quality the test of the slice, not void statistics. It
exercises the whole loop (record → capture → refinement → body seeding)
and it is the thing 460 m coarsening most threatens.

## User directive, 2026-07-21 — rendering water: partials/structure, placeholder texture, honest seams

Captured at session start (user): when we reach rendering water in-world,
(a) take full advantage of **partials and structure**; (b) **placeholder
texture is appropriate** at that point; (c) the requirement is to *leave
the seams* "for us to do more interesting things with rendering it such
as textures showing direction of flow, waves, etc."

Assistant reading (PROPOSAL beyond the user's words): this is the 0049
station-2 forms/partials directive reaching water — fluid already
occupies eighths and pores at the material tier (materials.md), so
water's render path enters through the same partials/forms substrate sand
will, not a bespoke water-mesh system. And the seams are **data seams,
not art seams**: flow direction and wave energy are facts the water
system owns or will own (body-graph inlets/outlets = direction; the fetch
heir = wave energy per body, journal/0049 station 5). The extension point
to preserve is "per-surface water attributes sourced from the body/flow
data" — future rendering upgrades read sim facts, never invent visual
noise. Appearance-class discipline applies: the placeholder rides until
the user upgrades it from pictures (the 0048 substances-not-portraits
lesson).

## SPIKE SPEC (drafted 2026-07-21, NOT dispatched, NOT ratified) — S14: can drainage refine under coarse boundary conditions?

Written up now rather than later because this one question gates the whole
river-primitive retirement, and re-deriving it costs the same thinking twice
("defer = write it now"). **This is a specification awaiting the user's
go-ahead, not a decision and not scheduled work.**

**The question.** S9 measured erosion as bounded/haloable and named drainage
**the single advective wall** — the one genuinely global computation. But S9
tested drainage in its *unconditioned* form. The unasked question is whether
drainage refines inside a bounded region window when the **inflows across the
window boundary are pinned by the coarse graph** — i.e. every upstream
catchment's discharge is supplied as a boundary condition rather than
re-derived. If yes, channels become part of the eroded surface at landform
resolution and today's `RiverSeg` carve dies honestly. If no, some channel
inscription survives at the collapse tier — and the honest framing then is a
**refinement operator** (lattice-midpoint class machinery, no world-fact
status), never a persisted primitive.

**Why it is not obviously answered by S9's halo result.** Erosion's halo is a
*decay-length* argument (influence attenuates with distance). Drainage is a
*connectivity* argument — a divide shifting one cell can re-route an entire
catchment. The hypothesis under test is that pinning boundary inflow converts
the global dependency into a local one *given* the divide structure the coarse
graph already fixed. The failure mode to hunt: refinement moving a divide such
that the refined interior disagrees with the coarse boundary it was handed.

**Measurement groups (each needs a number, not an argument):**
1. **Agreement** — refined-region discharge/flow-direction vs the coarse
   graph's own answer at coincident points. This is the S13-style "does the
   fine instrument reproduce the coarse source" check.
2. **Halo decay** — perturb the interior; measure how far the effect reaches
   toward the boundary, as S9 and S11 did (S9 erosion 16–24 cells; S11 bound
   water 4–11). A drainage number in that family is the GO signal.
3. **Divide instability rate** — how often refinement re-routes a catchment
   across the window boundary, and the magnitude when it does. This is the
   mechanism most likely to kill the approach; measure it directly rather
   than inferring it from (1).
4. **Seam continuity** — two adjacent refined windows must agree where they
   meet, to the same standard the elevation lattice already holds (the
   `s7_walk` interior-step invariant is the precedent).
5. **Cost** — wall-clock and memory per refined region, against the ratified
   ritual budget and the approach-time streaming budget. C-refinement
   (`deeptime/refine.rs`) is the existing machinery; reuse it.
6. **Negative control** — the same measurements with boundary conditions
   *deliberately unpinned*, to show the pinning is what buys the locality
   (the byte-identical-when-off discipline, applied to a claim instead of a
   flag).

**Decision rule, stated before the measurement** (so the result cannot be
rationalized): a halo in the S9/S11 family plus a divide-instability rate low
enough that (4) holds ⇒ **rivers become refined terrain and the primitive
retires**. Otherwise ⇒ **the carve survives as a named refinement operator**,
and `RiverSeg`'s heir is that operator rather than the erosion record.

**Sequencing note.** This spike is downstream of a decision it should NOT
prejudge: the *two-drainage-opinions* finding above (pregen cell hydrology vs
the deep tier's per-epoch drainage). If deep drainage becomes the spine, this
spike refines *that* field; if not, it refines the cell graph's. Settle the
spine question first or the spike measures the wrong field.

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

## SPIKE SPEC (drafted 2026-07-22, DISPATCHED — see § S15 ANSWERED below) — S15: what does the free-water graph cost against a LAZILY GENERATED, EVICTING world?

**Written because S11's numbers do not transfer and should stop being quoted
as though they do.** S11 ran on `water/vox.rs` — a deliberately toy one-bit
solid/air volume, fully resident, explicitly not the production world. There is
no lazy generation anywhere in that harness, so every capacity scan it timed
walked memory that was simply there. **415 ms is an honest number for a world
that does not exist.**

**Not blocked by the two-drainage-opinions decision** that S14 waits on: this
asks about *free-water bodies against lazy chunks*, which is orthogonal to
whether channels become refined terrain. Dispatchable independently.

**The question (user, 2026-07-22).** A capacity scan means "walk the container
to learn volume↔level". Against the real generator that is a **generation
storm** — and worse, the chunk store is a bounded LRU, so the storm **evicts
the ground the player is standing on**, right when they are actively digging
the trench that caused it. The user's proposed answer: *"if cells / the coarse
regions know roughly their level (and remember if it changes — remembering
player edits) then that math could be simpler."*

**Why it should work, stated so the spike can falsify it.** Capacity is
**additive**: volume below level L is a sum over regions, so a per-coarse-cell
hypsometric summary makes the body's curve a sum, and coarse cells are already
fully resident. Edits become a **delta, not a rescan** — digging removes a known
volume below a known level, and `set_block_raw` is the single audited
voxel-writing path (journal/0051) to hang that increment on. Sub-cell relief is
available without chunks: `coarse_surface` derives any column's surface at
**1.377 µs/column**, memoized, generating nothing (journal/0022).
Expected accuracy property, also to be falsified: level error is `ΔV / surface
area`, so the coarse estimate is **most accurate exactly where an exact scan is
most expensive** (big lakes) and worst where exact is cheap (flooded shafts).

**Measurement groups — each needs a number, not an argument:**

1. **Coarse capacity accuracy.** Coarse-derived level vs an exact voxel-walked
   level over real basins in a production world, as a function of body surface
   area. *Decision rule, fixed in advance:* if coarse-derived level lands within
   **half a voxel (0.45 m)** above some area threshold, the coarse path ships and
   exact scans are reserved for bodies below it. Half a voxel because that is
   where the shoreline moves.
   Fold in **one** number, not a study: the exact-scan cost at the *largest*
   body still handed to the exact path, so the fallback the decision rule
   creates is bounded rather than open-ended.
2. *(**CUT 2026-07-22 by the user**: "measure the storm" — chunks generated,
   wall time, eviction damage for a naive scan against the real store. Cut on
   the grounds that **no outcome of it changes a decision**: there is no world
   in which the answer comes back "the storm is fine." A measurement whose
   every result leads to the same action is theatre. The qualitative fact — a
   scan generates chunks and thrashes a bounded LRU while the player is
   digging — is sufficient to reject scan-per-breach.)*
3. **Incremental maintenance.** Dig through the audited write path, adjust the
   summaries, compare against a fresh exact computation; measure **drift over a
   long edit session** (the S11 conservation-drift analogue). Must stay under
   group 1's half-voxel threshold.
4. **The connectivity hypothesis — hunt the falsifier.** Claim: *connectivity
   only changes where someone edits, and edits only happen where chunks are
   loaded* (to breach a lake you must dig, and to dig you must be there). The
   adversarial case is a breach joining two bodies through terrain nobody ever
   loaded. **Try to construct it.** S11 found two real determinism violations by
   writing the adversarial case, not by the shuffle passing.
5. **Eviction identity.** S11's reload-from-39-bytes, redone against the real
   evicting store: does a body's derived water survive its chunks being evicted
   and re-derived byte-identically?

**Determinism, as ever:** double-run byte-identical; order-independent over
shuffled edit batches.

**Explicitly out of scope:** rendering water, the water cycle, karst,
dissolution, cementation. This is the storage-and-cost architecture of free
water against lazy generation, nothing else.

## LEANING, NOT DECIDED (user, 2026-07-22) — one octree substrate: FF2b, bulk flow, and the statistical tier

> **RESOLVED same day → `docs/design/octree-substrate.md`** (DECIDED
> 2026-07-22, live session): one substrate = the existing S3 chunk pyramid,
> named, with a payload contract; FF2b-minimal is the first build slice.
> Water's stratum of the node payload stays requirements-only — the hydrology
> pause holds. The section below is kept as the record of the leaning and its
> reasoning.

Recorded at the user's instruction as a **discoverable leaning**, explicitly
deferred: *"i agree that we defer and make discoverable note of leaning."*
Nothing here is ratified and nothing should be built against it yet.

**The question that prompted it** was already filed in this file and unasked:
*"Bulk-flow octrees may share substrate with FF2b's volumetric summary octrees
(SVDAG/Aokana candidate)… the shared representation question has not been
asked. Worth checking before either is built."* One half is the user's own § 2
notebook entry (bulk flow as octrees of continuous flow); the other is
visuals.md's FF2b, already sequenced to land **with** the underground/overhang
features rather than before them.

**The user's lean: YES — one substrate, and it extends further than water.**
*"I tend to think yes, and I tend to think it will be useful for more
statistical far-field events in the future; we already have a commitment from
day one about observation collapse and the quantum state of distant systems —
particularly social, what a society is doing, what an NPC is doing."*

**The day-one commitment it connects to** (ARCHITECTURE.md § "Simulation:
tiers + constraint ledger"): the Statistical tier is *"distributions over
possible states, derived on demand"*; **fluid state** *"evolves as a seeded
pure function of (region seed, time, committed constraints) — derived, not
stored; replays identically"*; **collapse** is bounded to depth N, and *"at the
frontier we synthesize plausible boundary conditions from the statistical tier
instead of recursing."*

**Three observations from the session that argue for the unification:**

1. **The chunk store is already committed/fluid, for terrain.** `HostWorld`
   pins *edited* chunks (immutable facts) and evicts *untouched* ones because
   they re-derive byte-identically (journal/0051). The v1 distinction is
   already shipped in the storage layer under a different name.
2. **The lazy water model derived this session IS the fluid-state
   definition.** "A basin's level is closed-form between scheduled events,
   evaluated when someone looks" and "a seeded pure function of (region seed,
   time, committed constraints)" are the same object, arrived at independently.
3. **The frontier rule is the same rule three times** — the far field's outer
   ring, drainage-decided-once's pinned boundary inflows, and collapse's
   synthesized frontier conditions are one pattern: bounded derivation with a
   coarse boundary condition.

**An integrator split, PROPOSED then WITHDRAWN in the same conversation**, kept
because the pushback is the useful part. Proposed: *one pattern, two indices* —
an octree for volumetric things (terrain, water, caves, far-field appearance),
a region graph for subjects (agents, societies, journeys), on the grounds that
an NPC's state is a property of a subject rather than of a volume. The user's
one-line refutation: **"where is the duke?"** A subject *has* a position, that
position is itself a fluid fact, and the questions a player actually asks are
spatial — who is in this valley, what is over that ridge. `ideas.md` had
already settled it: *"journeys are distributions over routes; meeting one is
bounded collapse conditioned on route + time."* Subjects were never outside the
spatial index.

**Where it landed: one index, plus a relation graph over it.** Containment
cannot express a trade route, a political tie, or a drainage path — but that
shape is already in the engine twice (`DeepField.recv` is exactly a relation
graph over spatial cells). And an octree is **scale-free by construction**, so
bounded collapse to depth N and octree depth are the same kind of bound: the
structure that stops observing one mind from collapsing the planet is the
structure that stops drawing a horizon from meshing the world.

**Discipline on scope (skill § seam-first, practice 6):** do not design one
substrate for three consumers before any exists. Ask the representation
question now — this file has been saying to for two days — but let the **first
two consumers define the node payload** (renderer: contents; water: storage,
permeability, level), with social recorded as *"this pattern is intended to
extend there"* rather than designed for in advance.

**Consequence for S16 (unwritten):** draft it against an **octree node**, not
a per-deep-cell summary. A single value per 460 m column cannot express a
confining bed with an aquifer beneath it — the first thing on the user's own
encounters list — and `MixtureDownsampleRule` (2×2×2 contents → one parent,
built, tested, unrendered) *is already an octree reduction step*.
## S15 ANSWERED (2026-07-22) — docs/spikes/S15-results.md, journal/0062

The spike § SPIKE SPEC S15 dispatched — *what does the free-water graph cost
against a lazily generated, evicting world?* — is complete. **Agent
recommendation GO**, and nothing in it needs ratification: the knobs are
engineering calibrations of the same status as S11's halo, so they ride as
measured. The user's proposal — *"if cells / the coarse regions know roughly
their level (and remember if it changes — remembering player edits) then that
math could be simpler"* — survived, and two of the spec's own claims did not.

- **The coarse path ships above 26 m² of water surface.** A per-cell
  hypsometric summary (32-column cells = one chunk footprint, 4×4 sub-samples
  standing in for 1 024 columns — a **64× compression**) reproduces the exact
  voxel-walked level within half a voxel for **781 of 791** real bodies in the
  production world, up to 212 000 m². Over those queries: **coarse 109 ms and
  0 chunks generated** against **exact 64 104 ms and 6 444 chunks**. The
  fallback the decision rule creates is bounded — the largest body still handed
  to the exact walk is 23 m², at 0.2–3.2 ms and ≤2 chunks.
- **Edits are exact integer deltas on the audited write path**, so incremental
  maintenance adds no error of its own: **drift 1.9 × 10⁻⁵ m over 20 000
  edits**, with the residual compression error held constant across the whole
  session, order-independent over shuffled batches.
- **FALSIFIED — "the narrow flooded shaft is the worst case"** (the spec's own
  predicted accuracy property; corrections #30). The `ΔV/area` law holds, but a
  shaft is *dug*, and a dug void is recorded exactly rather than sampled:
  measured error **0.000000 m**. The mechanism's real worst case is a small
  **natural** depression, where sub-cell relief is unresolved and there are no
  edits to correct it.
- **FALSIFIED — the connectivity claim's second clause** (corrections #31).
  *"Connectivity only changes where someone edits"* is TRUE, and for a strong
  reason: terrain is a pure function of the seed and an evicted chunk re-derives
  byte-identically. *"…and therefore the consequence is inside the loaded set"*
  is FALSE — **one plug voxel** removed from a 640-voxel tunnel joins a body
  **288 m beyond the edit**, and **972 of 1 215** far-apart basin floors are
  already one body through terrain nobody has ever loaded. A container spanning
  never-generated ground is not the adversarial case; it is the default.
- **The natural-sill falsifier is not constructible today** — `generate_chunk`
  is a pure heightfield, so all the sub-level air of a basin is one component by
  construction. That dates the result: **when the cave families land it becomes
  constructible at will, and it should be the first thing re-run.**
- **Eviction identity re-proved against the REAL store**: capacity curve and
  derived water byte-identical across 3 120 evictions at an 8-chunk budget with
  an edit pinned inside the body — S11's reload-from-39-bytes, now against a
  bounded LRU instead of a resident toy volume.

**The one design question this raises, and it belongs in this notebook:**
capacity below a coarse cell's floor plane comes **only from edits**. That is
complete for today's heightfield world and wrong the moment caves exist — a
natural void under a lake is capacity the summary cannot see. The collapse
tier's *"recorded conduit capacity → void intervals per column"* (§ Session
capture 2026-07-21) is the **same axis read from the other end**. The cave
thread and the capacity mechanism share it; decide it once, here, rather than
discovering it twice.
