# Ecology, organisms, and evolution

> **⚠ DORMANT SINCE AUTHORING — REVISIT BEFORE BUILDING ON THIS (2026-07-26, user).**
> This remains a **ratified user design**; `CLAUDE.md`'s *existence-is-not-standing*
> doctrine targets **unratified bootstrap content** and has **no authority here**. (An
> earlier draft of that doctrine wrongly listed "ecology" among the things we have no
> design for — an assistant insertion, corrected the same day.) But the conversation has
> been **dormant since 2026-07-19 and is at risk of going stale**: the project is not
> aiming at it while it solves **non-bio earth processes**, and the architecture has moved
> substantially underneath it — the north star ratified 2026-07-23, the material/behaviour
> substrate, and **2026-07-26's plugin-agnostic + authored-order decision**, which changes
> what "a pass" even is. **The design should be revisited in light of those evolutions, and
> is open to reconsideration by the user as the project's philosophy has matured.** Four
> `stubs.md` entries name ecology as their heir; they are not invalidated, but they inherit
> this caveat.

Status: substrate RATIFIED 2026-07-19 (user: "this reads absolutely
right"); the evolution architecture below is the USER'S DESIGN, recorded
2026-07-19 as the frame to build toward (not v1, but v1 must not foreclose
it). Sections marked *(Claude)* are my elaborations — proposals until
reacted to. Supersedes ROADMAP Sequenced 4/5 and the biomes-ecology
agenda's framing question: **species is the primitive; biome is a
diagnosis.**

## 0. The reality-first account (per earth-processes.md doctrine)

Distribution on Earth = **tolerance** (Shelford's ranges) × **limitation**
(Liebig's minimum — growth caps on the *scarcest* resource, so the
operator is `min`, never a weighted mean) × **dispersal** (nothing grows
where propagules never arrived — ranges are fronts, refugia, islands) ×
**interaction** (crowding, shading, allelopathy, facilitation) ×
**history** (what arrived first; what burned last century; what the ice
did).

Existing vocabulary we adopt rather than invent:
- **Niche** — Hutchinson's n-dimensional hypervolume; *fundamental*
  (physiological) vs *realized* (what competitors leave). This is our
  class-contract + fitness pattern exactly.
- **Succession mechanisms** — Connell & Slatyer: **facilitation**
  (lichen → soil → moss → grass), **inhibition** (incumbents resist:
  shade, allelopathy, root mats), **tolerance** (successors endure lower
  resources). Modern framing: **state-and-transition with alternative
  stable states**, not a single climax.
- **Niche construction / ecosystem engineers** — organisms modify their
  own abiotic world.
- **CLORPT** (Jenny): soil = f(Climate, **Organisms**, Relief, Parent
  material, Time). The canonical soil equation already has life in it —
  soil is where geology and biology couple.
- **Walker & Syers chronosequence**: over deep time on a stable surface,
  rock-derived **phosphorus depletes monotonically** while
  biologically-fixed **nitrogen accumulates then plateaus**, ending in
  P-limited **retrogression** — ancient landscapes carry weird,
  sclerophyllous, nutrient-miserly vegetation. This is "mineral
  depletion/replenishment" with a known deep-time shape.
- **Disturbance regime** — fire return interval, flood recurrence,
  windthrow; peak diversity at intermediate disturbance.
- **Trophic transfer** (~10%/level) — why fauna can be *derived* from
  primary production rather than separately simulated at world scale.

## 1. Flora → geology (why this earns its cost)

Biology is an erosion term, a sediment source, and a rock-forming
process:
- **Vegetation invented meandering rivers** — pre-Devonian channels are
  overwhelmingly braided; rooted banks are what let a sinuous channel
  hold form.
- Land plants **accelerated chemical weathering** several-fold (root
  acids, mycorrhizae) and drew down CO₂.
- **Coal** is plant matter that outran decomposition; **chalk and most
  limestone are corpses**; banded iron records cyanobacterial oxygen.
- **Paleosols** mark every long stable interval in a column;
  **bioturbation** erases lamination that would otherwise record floods.

So the biotic layer writes into the same two-plane deep-time loop:
root cohesion → erosion resistance; biological weathering → regolith
production; litter/peat → depositable organic material; all recorded with
tags.

## 2. Biome is a diagnosis, not a primitive — DECIDED 2026-07-19

Simulate species with niches propagating over a historied landscape;
assemblages **emerge**; "temperate steppe" is a cluster label applied
afterward by a registry-defined classifier. Wins: real ecotones instead
of blend zones; refugia, invasion fronts and disjunct populations for
free; multiple classifiers can read one world without regenerating it.

**A "biome pack" is therefore an ORGANISM pack** — species niches,
bio-materials, and their relationships (facilitates, inhibits, eats,
burns). Symmetric with the geology pack, same registry, same
class-contract machinery.

## 3. The bounded substrate (RATIFIED 2026-07-19)

**Per-species (registry niche contract):** tolerance ranges over existing
context axes; nutrient demands (N, P, cations); traits — dispersal kernel
radius, longevity, shade tolerance, N-fixing flag, litter quality, root
depth, erosion-resistance contribution, flammability.

**Per-cell state (~10 scalars + a vector):** soil depth, organic matter,
N, P, base cations, moisture regime, time-since-disturbance, and a
**community vector** = top-K species with cover fractions. **K bounds
co-occurrence, never roster size** (the S8 mixture-cap lesson — it is
what makes a rich organism roster affordable, exactly as it made the rich
mineral roster affordable).

**Six processes per epoch step:**
1. **Suitability** — `min` over tolerances (Liebig).
2. **Dispersal** — bounded kernel from neighbours (a *relaxing* process
   by S9's classification ⇒ haloable, C-refinable).
3. **Competition** — finite capacity, weighted by incumbency
   (inhibition) and prior facilitation.
4. **Nutrient cycling** — uptake, litter, decomposition, weathering
   release, leaching loss (the Walker & Syers curve emerges).
5. **Niche construction** — write back to soil depth, erosion
   resistance, weathering rate, fuel load.
6. **Disturbance** — fire from fuel × aridity, flood from hydrology;
   resets succession, leaves a tagged mark.

**Biotic tags join the strata record**: organic horizon, peat, charcoal,
shell bed, bioturbated flag, paleosol. The payoff is readable in a cut
face — a coal seam where a swamp persisted, charcoal where it burned, a
paleosol marking ten thousand quiet years, retrogressive scrub on an
ancient surface.

**Derivation chain**: A-tier runs millennial propagation → C refines
regionally on approach → individual plant placement at chunk scale comes
from the cell's community vector by addressed hashing (the same
select-from-class machinery geology uses).

**Lagged coupling — required (Claude, flagged):** biology modifies
erosion; erosion modifies terrain; terrain sets climate and soil; soil
sets biology. The pass graph will (correctly) refuse that cycle. Fix:
biology reads *last* epoch's terrain and writes modifiers consumed by the
*next* step. Not a hack — it is how the physics works at these
timescales — but it is a stated architectural rule.

## 4. Evolution — the USER'S DESIGN (2026-07-19; post-v1, but v1 must not foreclose it)

**An organism definition is a teleology: where the org primitive must
arrive by a stated horizon.**

- An **ancient** org def has an opinion only about its shape in the
  *starting* epoch. Evolution then controls whether we observe
  descendants in the present, and how they appear and distribute.
- A **present-epoch** org def **weights the sim** so that one branch
  reaches that target as completely as possible.
- An **ahistorical origin override flag** may seed an org with no
  history, by distribution rules that were not *arrived at* but are
  otherwise **indistinguishable** from distributions that were.
- This implies an **org body-plan builder** — org structure primitives
  plus knobs for gradating and branching between them. And it implies
  NPCs eventually get the same treatment.

### What this is, formally *(Claude)*

Standard evolution sim is an **initial-value problem**: seed ancestors,
run mutation+selection, accept whatever soup emerges — unauthorable,
unpredictable content. This design is a **two-point boundary-value
problem**: pin the ancient form at t₀ and the modern form at t_now; the
*lineage between them is simulated*, with real environmental selection
deciding the path, the timing, the branch structure, the geography — and
whether it arrives at all.

Mechanism *(Claude, proposed)*: the target is a **prior over mutation**
(guided variation — mutations toward the target are likelier to be
sampled/retained), **not** a thumb on selection. The environment still
selects honestly. Therefore **the world can falsify the target**: a pack
author proposes a creature, and the sim answers whether it could have
arisen, and *where*.

### Consequences worth designing for *(Claude, proposed)*

- **Mid-horizon pins are fossils.** An org pinned at epoch 3 that need
  not survive to the present *is* a fossil species — and because the
  strata record already tags deposition with context, its remains land
  in the layers that were its habitat when it lived. Digging up a
  trilobite that actually lived in *your* world's Cambrian seafloor, in
  the rock that was that seafloor, is the doctrine's "how did this get
  here?" test at its most powerful.
- **The target specifies WHAT, the world decides WHERE.** If the pinned
  form doesn't fit where an author imagined, the sim places it where the
  form works (or extinguishes it). Same "propose, world disposes"
  pattern as geology class selection.
- **Allopatric speciation is free.** Lineages split when populations are
  isolated — and isolation is *emergent* from simulated terrain. Real
  mechanism, zero extra machinery, only possible because we simulate
  geography.
- **The plan-params sketch is the mutation substrate.** bodies.md's
  PROPOSED scalar/bool plan parameters turn out to be exactly what
  evolution operates on: scalars = allometric drift; bools = segment
  gain/loss; plan forking = cladogenesis. The org body-plan builder and
  the mob body-plan builder are one system.
- **Ahistorical orgs have no answer to "how did this get here?"** — the
  knowledge/inspection system will find a blank where a lineage should
  be. For fantastical organisms that is *diegetically perfect* (they
  were made, or they arrived); worth designing as a feature, not
  papering over.

### The stack this implies

```
species  — niche + body plan + lineage/evolutionary history
individual — body (plan instance) + controller + identity
```
Fauna are organisms with richer controllers; the character/controller/body
split (API.md, bodies.md) already anticipated this.

## 5. Open forks (user-owned)

1. **Fauna in v1?** Derived pressure field (grazing, bioturbation,
   dispersal assistance — cheap, geologically meaningful) vs creatures
   you meet (drags in NPC-intelligence + body-plan tracks).
2. **Pack-addition blast radius.** Adding an organism pack changes the
   evolutionary tree → distributions → *and terrain*, since biology is
   an erosion term. Materials packs never changed landforms; org packs
   can. Policy options: bake biotic erosion at world creation
   (pack-add affects ungenerated regions only, matching the existing
   DF-like seed policy), or accept landform drift. **Needs deciding
   before biology couples to erosion.**
3. **Evolution epoch resolution** — how many evolutionary steps per
   deep-time iteration; species-distribution grain (A-tier 460 m?).
4. **Educational posture** — how explicit is the teaching? (The claim
   that makes it work: the game teaches by *being causally honest*, so
   player intuitions transfer to the real world; not by lecturing.)

## 6. Sequenced next

**S10 biotic-layer spike** (before any commitment): community vector +
the six processes on the S9/3e-1 A-tier; measure cost delta against the
~14 s ritual, and read-quality — do we get coal seams, paleosols,
charcoal bands, retrogressive surfaces? Evolution is explicitly NOT in
S10; S10 exists to prove the substrate that evolution will later ride.

**RUN 2026-07-20 — measured, not yet ratified** (journal/0025,
docs/spikes/S10-results.md). All six processes implemented on the A tier,
off by default. **All four signals present and legible**: coal (thickest
seam 24 m), paleosols (22.9 % of columns, incl. genuine cyclothems with
at-deposition climate tags), charcoal (20.2 %), retrogression (23.5 %, with
the *geography* right — only erosion-untouched surfaces starve). **Cost**:
ritual 15.2 s → 25.2 s (1.66×), linear in cells, and the world keeps only
+6 MiB (the rest is transient). Determinism intact incl. scalar↔parallel
byte-identity. Spike-level findings that bear on this doc:

- The **lagged coupling rule of § 3 works as designed** and was the least
  troublesome part of the build — erosion consumes last epoch's biotic
  modifiers, biology writes next epoch's.
- **Biology is measurably an erosion term** (falsifier test: the biotic
  run's bedrock surface differs from the abiotic one), so § 5 **open fork 2
  (pack-addition blast radius) is now live** — adding an organism pack would
  change terrain. It needs deciding before biology ships coupled to erosion.
- The § 3 **top-K cap was not stressed** (7-species roster fits whole), so
  its value is assumed rather than measured by S10.
- The doc is silent on world-genesis colonization; the spike needed a weak
  background propagule term to avoid a bootstrap deadlock (a species that is
  nowhere can never be anywhere). FLAGGED in the results doc.

The GO/NO-GO and five ratification calls are the user's — ROADMAP
§ Sequenced, "S10 follow-through".

## DECIDED 2026-07-20 (user) — S10 GO, biology ships

- **GO on the biotic layer.** `production_config`'s `biotic` flag is flipped
  ON; biology is a shipped part of world generation, not an experiment.
- **The 25 s ritual is acceptable** — explicitly, with headroom: the user's
  frame is that Dwarf Fortress takes minutes, and "when we have full history,
  we'll likely take much longer." **World-generation time is not a design
  constraint we optimize against by default.** The mitigation is not a faster
  ritual but an *alternative to running one* — see the ready-made-worlds
  direction (ideas.md § entering a world without generating one), which the
  user re-raised at this decision.
- **Coal, and every other missing organic material, gets built immediately**
  ("asap"). The measured gap: `deep_class` (geology.rs) selects a content
  class from `DepTag`'s env/energy only, so the `Biofacies` axis the S10
  recorder writes never reaches material selection — an organic unit
  collapses as ordinary clastic and the 24 m seam is unminable. This closes
  that.
- Riding as-built pending later judgement (NOT re-opened here): the 7-species
  roster and its ~25 constants, and the signal densities (a fifth of columns
  carrying fire records) as an aesthetic question. Both are calibrated to the
  current precip/P fields — if the orographic march or the P economy is
  retuned, they need **re-measuring, not re-guessing** (journal/0025).
- **Still open, still the user's**: § 5 fork 2 (pack-addition blast radius).
  S10 proved biology is an erosion term, so an organism pack changes terrain.
  Undecided as of this entry.

## SHIPPED 2026-07-20 — the flip is live, biology reaches the material tier

journal/0026. The GO action is done and the gap above is closed.

- **`production_config`'s `biotic` is ON.** Every new world runs the six
  processes over deep time; the `DeepField` carries organic facies.
- **`deep_class` consults `Biofacies` first and lets it win where inhabited.**
  Three new content classes — `dc:stratum/organic-{coal,peat,soil}` — each a
  contract for one *measured* facies, filled by one vanilla member (coal, peat,
  carbonaceous mudstone). `Soil` and `Retro` share the soil class. The
  class-share invariant and registration-order independence both hold.
  **AMENDED 2026-07-22 (journal/0063): four classes.**
  `dc:stratum/organic-charcoal` / `dc:geo/charcoal` joined them — the `Charcoal`
  facies stopped wearing its mineral host's identity. It is an **inclusion**
  class by measurement: a fire bed is capped at 0.04 m, so it never fills a
  voxel; it wins single eighths under the distribution-first fill (measured:
  0.394 % of recorded voxel spans).
- **Why "wins" and not "blends":** the 24 m seam's own tag is `Sa/A/L` —
  subaerial, **arid**, **low** energy. Flow energy describes the grains a
  passing current carried; it says nothing about plant matter outrunning decay,
  which is the whole reason the rock exists. § 3's waterlogging-vs-rainfall
  separation (S10 design choice 8) is what puts an arid tag on a coal swamp, and
  it is correct: swamps sit where drainage collects, not where rain falls. The
  organic classes therefore leave the **precip axis open** and let depth work.
- **The record's resolution limit is now measured, and it bounds what ecology
  can express as material.** Against the 0.9 m voxel: coal 89.6 % of units
  survive, Soil 48.5 %, Retro 6 % of units (56 % of thickness), Peat 2.3 %, and
  **Charcoal 0 of 158 310** (mean bed ~3.5 cm). So the fire record — one of the
  four signals this doc asked for and S10 delivered — **exists in the record and
  is invisible in the world.** No charcoal material was shipped; a band that
  thin is an inclusion, not a layer (geology.md § inclusions), and that is filed.
  This is a general lesson for § 3's derivation chain: **a facies the A tier can
  record is not automatically a facies the collapse tier can show**, and the
  sieve is the voxel edge.
- **Not shipped, honestly**: coal rank (needs ~1–2 km burial; we record ~100 m —
  the class's depth axis is reserved as the rank axis), and a distinct
  retrogressive material (retrogression is a phosphorus fact about a *community*;
  the property sheet has no nutrient axis — the day we track weathering products
  is the day that class gains a second member).
- **Cost correction**: the ratified 25 s ritual was the spike harness's scalar
  path. Shipped it is **13.79 s**, biology's marginal cost **+2.6 s** — see
  journal/corrections.md #12.
- **Still open, still the user's**: § 5 fork 2 (pack-addition blast radius).
  Unchanged by this milestone, and now shipping in a world where biology really
  does move rock.

## DECIDED 2026-07-21 (user) — vegetation joins deep time; the veneer is a placeholder

**Vegetation becomes part of the deep-time sim.** In a sense it already is —
peat and coal exist in the record *for this reason*, as the preserved residue of
communities that lived somewhere for a long time. The decision extends that from
the buried residue to the living cover.

**Runtime gen reads the latest ecological state from deep time and expresses it
per member.** Not a painted biome lookup: each member has a **proliferation
pattern** — how it propagates, what it eats, what minerals / warmth / light
amount / spacing it needs (examples, not a literal schema) — and placement falls
out of **fitness in a system**, species related to each other and to their
environment. A region that has not been geologically active for a long while is
nutrient-sapped and hosts only hardy scrub **because that is how it happened**,
not because it was painted there afterwards. This is § 2's *"biome is a
diagnosis, not a primitive"* (DECIDED 2026-07-19) carried into the runtime tier,
and it is what the S10 substrate already computes at depth — the Walker & Syers
retrogression chronosequence (23.5 % of columns, nobody scripted it) is exactly
the nutrient-sapped landscape described above, currently invisible to a player.

**The current surface veneer is waiting-for-the-correct-implementation.**
`Block::Grass`/`Dirt` chosen from year-zero climate and a lapse-rate threshold
is **not** an ecology and must not be treated as one. It is explicitly and
loudly temporary in the sense of geology.md § Expression of the ledger.
**Do not bandaid it.** No tuning of the grass/dirt/stone thresholds, no slope
term, no new heuristic — the correct implementation replaces it wholesale.
Reasoning about grass is suspended until ecology gets its proper design pass.

**When that pass opens, it starts from `docs/design/biomes-ecology-agenda.md`** —
the standing agenda whose § "The questions only the user can answer" has never
been worked through, plus § 4 (evolution, the user's design) and § 5 (open
forks) here.

## Note 2026-07-21 (user) — charcoal/coal rest on placeholder vegetation

The S10 biotic layer produces coal, peat, and charcoal beds from a **community
vector** — an abstraction with no actual vegetation in it. There is no concept
of the plants that burned, accumulated, or starved; the fire disturbance burns
a number, not a forest. So the organic facies are real *outputs* resting on a
placeholder *input*: legitimate under the loudly-temporary rule, but **they
must be revisited and integrated when the ecology pass lands** — when
vegetation has members with proliferation patterns, the fire/peat/coal
mechanisms should consume those members, not the community scalar. Filed so
the ecology pass inherits this obligation explicitly.
