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

## Assistant observations on the above (PROPOSALS — not ratified)

Kept separate from the user's notebook on purpose.

- **Karst has a hard prerequisite we have already filed: carbonate rock.**
  Dissolution needs soluble stone, and carbonate is currently an owed
  milestone (geology.md/ROADMAP: "Carbonate follows"; sea-floor and wilds
  columns keep empty strata records because subaqueous sedimentation *is*
  the carbonate milestone). So **karst is gated on carbonate**, and
  carbonate is gated on subaqueous deposition. That is a real sequencing
  edge, not a detail.
- **Glacial caves imply a cryosphere** — ice as an agent with its own
  extent and history. Deep time has climate, but ice as a modelled agent
  is a larger dependency than the other three families. Flag before it is
  assumed cheap.
- **Littoral caves need a coastline with wave energy over time** — sea
  level is already known per deep-tier epoch (flooded basins, spill
  levels), so the shoreline is derivable; wave energy is not modelled yet.
- **Erosional caves may be the cheapest first family**: paleo-drainage is
  something the deep-time tier already computes (drainage is the sole
  advect term). An abandoned conduit is a former channel that the water
  table dropped below — arguably derivable from data we already have.
- **The event-driven rail already exists in spirit.** Edit → dirty set →
  targeted re-derivation is the pattern used for chunk remeshing, collider
  tiles, and the (owed) far-field summary invalidation. Bulk-flow
  invalidation looks like the same rail with different payload.
- **Determinism constraint to solve EARLY**: event-driven resolution must
  be order-independent, or replay and the parallel path diverge. S9b
  measured exactly this class of problem for flood/erosion (scatter→gather
  reformulation, byte-identity proven). Whatever fires flow events needs a
  canonical order by construction, not by luck.
- **Possible shared substrate with FF2b.** FF2b (coarse volumetric far-
  field summaries) is already paired in ROADMAP with "the caves/underground
  thread of the water design pass", and its candidate representation is an
  SVDAG octree (Aokana). Bulk-flow octrees and volumetric summary octrees
  may want the same machinery — worth checking before either is built.
- **S10 built a water-table proxy already.** The biotic layer models
  waterlogging separately from rainfall (climate moisture + bonuses for
  sitting near base level and receiving upslope drainage) because peat
  needs standing water, not merely rain. That proxy is a flat stand-in for
  a real water table; when the real field exists, biology should read it
  and the proxy should be deleted. Two systems privately approximating the
  same physical quantity is the thing to avoid.

## Open questions (carried, unanswered)

1. What are the *encounters* — the moments a player meets water that no
   other game gives them? (Asked; the user's answer is pending, and the
   notebook should be driven by it.)
2. Sub-resolution water (a trickle, a damp seam, a puddle) — representation
   below one voxel.
3. Relationship between the bulk-flow octree and the per-voxel porosity /
   saturation field (S8) — are they one system at two scales, or two?
4. Does the near-field flow network persist, or re-derive from the water
   table on load? (Persistence question, same family as far-field summaries.)
5. Where does the deep-time water field live relative to the A/C tiers?
