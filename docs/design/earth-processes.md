# Earth processes — the reality-first field notebook

Doctrine (user, DECIDED 2026-07-19): **everywhere in terrain and
subterrain, ask first what reality looks like and how it fundamentally
forms — without thinking of our code — then build the highest-fidelity
process sim we can achieve, working backward from reality.** Like orogeny,
but we fully own generation.

Method (RATIFIED 2026-07-19, operationalizing the doctrine):
1. **Field notebook first** — every wanted feature begins as a written
   causal account: feature → engines → sequence → what the record looks
   like. Before any code.
2. **Mechanism fidelity over resolution fidelity** — choose the coarsest
   state + process set that still generates the causal chain honestly.
3. **The record is the world** — strata/structures/landforms come from
   running history; present-day fields may only drive present-day
   processes (geology.md § formation context, generalized).
4. **Validation is a geologist's-eye walk** — a cut face must *read*;
   "how did this get here?" must have a true, discoverable answer. The
   same property powers prospecting/knowledge gameplay.

## The engines (working notes — sketches, NOT decisions)

The variety of Earth's surface and subsurface is a SMALL engine set
superposed over deep time. Per engine: what it does in reality → the
signatures a player would see → candidate sim abstraction (to be refined
against the orogeny recon, 2026-07-19).

### 1. Tectonics
Reality: plate motion; uplift, subsidence, rifting, subduction, arcs;
faulting and folding of everything already deposited.
Signatures: mountain belts with deformed cores; tilted/folded strata in
cliffs; fault scarps and offsets; basins that filled over eons.
Sim: S7 provenance is the static seed; deep-time needs epoch-stepped
uplift/subsidence histories per region and deformation applied to the
accumulated record (3e).

### 2. Igneous
Reality: melt generation → intrusion (slow cooling, coarse: granite;
country-rock contact effects) or extrusion (fast: basalt flows, ash);
dikes/sills cross-cutting older rock.
Signatures: plutons under orogens; flow stacks in rifts; dikes cutting
strata (cross-cutting = relative age — readable history).
Sim: province + emplacement-depth context (never surface climate —
geology.md); cross-cutting events in the strata record.

### 3. Weathering
Reality: mechanical (frost, roots) + chemical (dissolution, oxidation)
breakdown, rates set by climate and lithology; produces regolith/soil.
Signatures: soil depth varying with climate/slope; rounded vs angular
outcrops; karst on carbonates.
Sim: rate = f(climate, hardness/erodibility class params) feeding the
loose-material budget.

### 4. Transport + deposition (four agents, four signatures)
Reality: water (sorted, graded, channelized), wind (well-sorted fine
dunes/loess), ice (unsorted till, striations, U-valleys), gravity (talus,
landslides). Each environment leaves a distinct facies.
Signatures: fining-upward river stacks; dune cross-beds; erratic boulders
on till plains; scree fans under cliffs.
Sim: energy-threshold settling (S8, proven) generalizes per agent; facies
= class selection under agent+energy context.

### 5. Burial, diagenesis, metamorphism
Reality: pressure/temperature over time lithify sediment and transform
rock along P/T paths (shale→slate→schist→gneiss); exhumation exposes it.
Signatures: metamorphic cores of eroded mountain belts; grade zonation.
Sim: per-column burial-depth history from the epoch stack → P/T path →
class transform (the deep-time compaction loop, materials.md).

### 6. Sea level + climate cycles
Reality: transgression/regression cycles stack marine/terrestrial facies;
ice ages rewrite erosion regimes.
Signatures: alternating strata (the classic layered cliff); raised
beaches; glacial overprint on temperate landscapes.
Sim: epoch-indexed climate/sea-level curves as pregen state — the
paleo-context axis (3e) in its simplest honest form.

### 7. Structural deformation of the record
Reality: the accumulated record itself folds, faults, tilts, and erodes
into unconformities before deposition resumes.
Signatures: angular unconformities; folded strata under flat strata —
*time gaps you can see and reason about*.
Sim: deformation operators on the strata record between epochs; erosion
surfaces recorded as events (an unconformity is a first-class entry).

### 8. Groundwater, karst, hydrothermal
Reality: dissolution caves on water tables; mineral-charged fluids
depositing veins/ore along fractures and contacts.
Signatures: cave systems in carbonate; veins cutting country rock; ore
halos near intrusions.
Sim: water-table interplay (materials.md aquifers); vein/inclusion
emplacement via pore partials (geology.md); fracture networks from the
deformation history.

## Quarry results (orogeny recon 2026-07-19)

Full report: docs/design/orogeny-recon-2026-07-19.md. Headlines:
- Orogeny independently built and stress-tested our per-column
  strata-record-with-context-tags shape at 147k columns — the design is
  field-proven.
- Cost philosophy adopted (RATIFIED 2026-07-19): **"coarsen the cause,
  never delete it and fake the appearance."**
- Steal directly: uplift-value-as-fold-phase (coherent folding, no 3D
  solver, addressed-hash friendly); exhumation-exposes-cooked-rock
  (grade = burial + stripped overburden); orographic-march paleoclimate
  (precipitation, not residual humidity); run-length merge +
  thinnest-pair eviction; tag-by-measurement discipline;
  jitter-renormalized-to-period (phantom-fault lesson); caves reading
  solubility from the recorded volume.
- The bounded/unbounded theorem (drainage is global, erosion is bounded;
  decay-length halos beat propagation-speed halos ~4×) is the same
  theorem as our LOOKAHEAD_BOUNDS — and speaks directly to the rivers
  2-ring Observed item.
- Their compromises we refuse: lossy K=8 eviction, nearest-cell mosaic,
  one-sided folds, single drainage-deciding layer, collapsed (ahistorical)
  surface geometry, declared-prehistory instead of a simulated Wilson
  cycle.

## S9 deep-time spike — framing (blessed 2026-07-19, with recorded doubt)

Architecture candidates for the deep-time tier between pregen and
collapse: **A** single global mid-tier (~460 m cells, seconds at
creation); **B** full landform-resolution global (~48 m, ~27 M cells,
minutes, paid once); **C** coarse global + lazy bounded regional
refinement (orogeny's halo theorem + our pyramid).

The user blessed testing **C** — and recorded explicit skepticism: "not
yet convinced this is the genius holy grail golden superthink path for
perf and deep simulation — in an engine that we 100% own." The spike must
therefore treat **B as a live contender, not a strawman**: measure
full-resolution global cost/memory honestly (a couple of minutes once per
world may beat an entire refinement architecture on simplicity), and
judge all candidates on read-quality per the ratified method plus total
cost of ownership (code complexity counts).

Spike tasks: two-plane erosion + strata recorder + orographic march;
coarse tier at ~250 m / 500 m / 1 km; one bounded regional refinement
with measured decay length; **process classification** — for each engine
we care about, does it relax (refine locally) or advect (decide coarse)?
Rivers are the known advective wall (the 2-ring re-proof item).

**S9 VERDICT (measured 2026-07-19 — docs/spikes/S9-results.md):**
A (460 m) = 14.1 s / 52 MiB; B (48 m, 27.3 M cells) = **~63 min /
~3 GiB projected** on the scalar engine (corrections #8 — "minutes" was
wrong); C refinement ≈ 3.6 s/region. Decay length measured at **21
cells** for hillslope (clean relaxation); fluvial spikes to 26 via
drainage reroutes — **drainage/flow-area is the sole ADVECT**, everything
else in the catalog relaxes at 16–24 cells. Read-quality at 500 m already
tells true stories (pulsed arid fans, transgressive couplets, condensed
cyclic margins). **Recommendation: A always-on + C refinement — CONFIRMED by S9b
(2026-07-19, docs/spikes/S9b-results.md, corrections #9): parallelism
does NOT flip B on this hardware.** The determinism tax forecloses it:
the flood (98.5% of the step at B scale) has no byte-identical parallel
form; deterministic-parallel phases are a bandwidth-bound minority
(whole-step 1.2×, saturating at 8 threads); realistic parallel B is
15–70 min vs the 2-min flip target. Reopens only on ~32-core hardware
with a deterministic parallel flood (Barnes spill-graph, unbuilt) —
`examples/deeptime_par.rs --full-b` re-measures anywhere. B remains fine
for Small worlds. 3e must also:
calibrate iteration↔Myr against a real orogen, build the coarse→fine
drainage handoff + collapse stitching (halo width is now a measured
budget input), widen recorder tags (agent axis + grain continuum), and
replace placeholder sea-level/climate curves with epoch-indexed pregen
state.

## Standing question for every future pass

"What does this look like in reality, and how did it get there?" — if the
pass can't answer in those terms, it isn't ready to be declared in the
graph.
