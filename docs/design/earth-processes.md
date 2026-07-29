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
5. **The unsimulated remainder → procedural tricks** (RATIFIED
   2026-07-19, user, during the 3e-2 stitching discussion): simulation
   stops at some resolution everywhere, and no simulation-resolution
   edge may reach the eye as a square or analytic boundary — "if i see a
   square boundary I'll scream." Every grid-scale contact (facies
   boundaries at deep-cell scale, C-refinement region edges, collapse
   cells) must be dressed by noise/dither/blend/sampling tricks before
   it is visible. The 3d member-boundary dither is the pattern instance;
   contact-softening is therefore IN SCOPE for 3e-2, not optional.

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

**Direction DECIDED 2026-07-20 (user, session-4 gen review) — design pass
owed before any code; spike-class.** Two coupled fixes to the uplift plane,
diagnosed from "one-shot upheaval" and "why is gradation-to-peak 50 km":
1. **Tectonic history — uplift becomes uplift(t).** Plates (Voronoi seeds +
   velocities) advect through deep time; boundaries re-classified and the
   uplift plane repainted every N epochs — the same re-march pattern the
   climate already uses. Buys superimposed orogenies (old eroded belt +
   young sharp belt), dip/fold as *recorded* deformation (beds deposit
   flat, later differential uplift tilts them — the physical mechanism §7's
   relax-term approximates), meaningful unconformities, migrating arcs,
   and hotspot tracks (impossible without plate motion). Design pass must
   settle: chapter count/cadence, advection rules, how the recorder tags
   deformation, and whether mid-run uplift repainting destabilizes the
   erosion clamp (measure).
2. **Analytic boundary forcing.** Stop painting uplift onto 14.7 km cells
   (which truncates all tectonic forcing at ~15 km wavelength and bilinears
   it into the 460 m deep grid); evaluate uplift as a continuous function
   of exact distance-to-Voronoi-bisector × convergence rate, sampled at the
   deep grid's own resolution. Orogen width becomes a design parameter
   (fast convergence → narrow sharp belt), not a grid artifact.
   Prerequisite for the amplitude call: amplitude tuned against the smeared
   template would need re-tuning after this lands.
Related decision wanted from the same pass: plate-count/scale compression
(~14 plates over 251 km ⇒ ~60–70 km plates) is currently an emergent
constant, not a choice — it directly sets landform provinces per km of
travel and should be a stated, user-owned knob.

**Scope expanded 2026-07-20 (user: "all of these land — escalate the
foundational"):** the same design pass also owns, because they share one
data model:
3. **Crustal columns (2.5D)** — each deep cell carries crust thickness,
   density, and composition, not just surface height. Collision becomes
   crustal thickening; the physics runs on column integrals, never voxels.
4. **Isostasy + flexure** — the missing engine (this doc had eight and not
   this one): crust floats, so erosion unloads and the root rebounds
   (~0.8× — **use `tectonics.md` § 5.2's derived ρc/ρm ≈ 2800/3300 ≈ 0.85**, which
   carries its densities; this doc's round number is a gloss and the 0.80/0.85 choice
   swings the exhumation multiplier 5.0×↔6.7×, a 34 % move in the headline number of
   that argument — cross-reference added 2026-07-29, baseline sweep S2/F10), which is
   how relief persists for hundreds of Myr and how
   deep-formed rock is exhumed to the surface (metamorphic cores). Flexure
   bows crust down beside loads → foreland basins, the sediment trap next
   to every belt. Requires exactly the crustal columns above. This is also
   the dynamic answer to one-shot upheaval: rebound is uplift *responding*
   to erosion.
5. **Drainage export per chapter** — *(diagnosis corrected same day,
   corrections #20: deep-time drainage already re-routes every iteration;
   the stale authority is that collapse carves rivers from PRE-erosion
   pregen `Cell.river`/`discharge` chords.)* Export the deep sim's final
   drainage (and per-chapter snapshots) and retire the pregen chords;
   chaptered forcing ramps are what buy water gaps, terraces, and
   captures. Couples to the slated deep-hydrology work — the two designs
   must know about each other.
6. **Recorder event-entries** — the strata record is a deposition-ordered
   stack and cannot express anything that *modifies previous entries*:
   dikes, plutons, fault offsets, tilting. The record needs a second entry
   species (events that transform prior units) before the igneous engine
   lands its first dike. Data-model decision; cheaper before than after.
7. **Punctuation hooks** — chapter machinery should admit rare discrete
   events (flood-basalt provinces, mega-landslides, at most one impact
   structure per world) so worlds get individual biographies. Hooks only;
   each event type is its own later design.

### 2. Igneous
Reality: melt generation → intrusion (slow cooling, coarse: granite;
country-rock contact effects) or extrusion (fast: basalt flows, ash);
dikes/sills cross-cutting older rock.
Signatures: plutons under orogens; flow stacks in rifts; dikes cutting
strata (cross-cutting = relative age — readable history).
Sim: province + emplacement-depth context (never surface climate —
geology.md); cross-cutting events in the strata record.
**FLAG (2026-07-20, ratified):** blocked on § 1's recorder event-entry
decision — a deposition-ordered stack cannot represent a dike/pluton/
offset. Also owns **marker beds** when designed: one ash fall writes an
identifiable isochronous stripe region-wide, giving players strata
correlation ("this layer here is that layer there") and making fault
offsets solvable puzzles (follow the marker to find the displaced seam).

### 3. Weathering
Reality: mechanical (frost, roots) + chemical (dissolution, oxidation)
breakdown, rates set by climate and lithology; produces regolith/soil.
Signatures: soil depth varying with climate/slope; rounded vs angular
outcrops; karst on carbonates.
Sim: rate = f(climate, hardness/erodibility class params) feeding the
loose-material budget.

**Erodibility coupling — BUILT 2026-07-20 (journal/0029), off by default.**
The deep-time engine now modulates erosion by the resistance of the lithology
outcropping at each cell. The one decision that matters for everything
downstream: **resistance is agent-specific, not a single "erodibility" scalar.**
"rates set by lithology" (above) is really *four* statements, one per agent —
mechanical abrasion, chemical dissolution, frost/ice, wave attack — and a rock
resists each differently. **Limestone makes cliffs (mechanically strong) AND
caves (chemically soluble); one number cannot hold both.** So the property sheet
carries a mechanical competence axis *and* a `solubility` axis, and
`deeptime::lithology` gives each agent its own resistance. Today only the
mechanical agent exists (fluvial incision, cover entrainment, and — the
rate-limiting one on hillslopes — bedrock→regolith weathering, all scaled by the
same abrasion susceptibility). Dissolution, frost/ice and wave axes are
populated and dormant. The § 8 karst agent, when it lands, reads the
`solubility` axis exactly as this section's "karst on carbonates" signature
requires — no rewrite of the mechanical path.

### 4. Transport + deposition (four agents, four signatures)

**RATIFIED 2026-07-20: wind is agent #5, and the dormant axes go live.**
Eolian was never enumerated; deserts have arid *tags* but no arid
*landforms* (dune fields, loess sheets, deflation basins). The `Agent`
enum's exhaustive-match design (0029) makes the fifth agent a compile
error until every site answers for it — by construction. Same decision
activates the dormant **frost** (periglacial: temperature-gated
weathering — scree, blockfields, shattered summits) and **wave**
(littoral: sea cliffs, wave-cut platforms; sea-level cycling already
exists) resistance axes built in 0029. At 460 m resolution the deliverable
is dune-*field*/loess/periglacial *regions* in the record and surface;
individual dunes and scree cones are collapse-tier detail for the
sub-460 m band work.

**Follow-up slice RATIFIED 2026-07-20 (user) — zonal circulation profile,
dispatch after the eolian agent lands (same march, avoid mid-flight scope):**
replace the three-way wind sign bit (`wind_dx`) with smooth latitude
profiles — zonal wind as signed *magnitude* passing through ~zero at the
band boundaries, plus a subsidence factor peaking there. One object fixes
two diagnosed defects: (a) the hard direction flip at 30°/60° that would
print a dead-straight climate/vegetation line and opposite-migrating dune
fields across one grid row (an analytic boundary reaching the eye — the
scream rule); (b) missing subsidence aridity — Earth's 30° desert belt is
the Hadley descending limb, not rain shadow, and the calm horse latitudes
ARE that limb, so magnitude-through-zero and desert-belt suppression are
the same mechanism. Also seeds the wind-strength field that deflation
scaling and future gameplay wind (present-day weather layer) want.
Gameplay-impact trace (required per the session-workflow rule this slice
occasioned): deserts land where a player who knows Earth expects them;
no straight-line seams; dune-field orientation varies believably with
latitude.
Reality: water (sorted, graded, channelized), wind (well-sorted fine
dunes/loess), ice (unsorted till, striations, U-valleys), gravity (talus,
landslides). Each environment leaves a distinct facies.
Signatures: fining-upward river stacks; dune cross-beds; erratic boulders
on till plains; scree fans under cliffs.
Sim: energy-threshold settling (S8, proven) generalizes per agent; facies
= class selection under agent+energy context.

**Erosion-agent roster FLIPPED ON — DECIDED 2026-07-21 (user; journal/0034
built it, journal/0047 flipped it).** The `full_agents` flag (wind + frost +
wave, § 4 above and § 3's dormant frost/wave axes) is now ON in
`production_config`. Every world made from here on runs the full roster:
eolian deflation/deposition, temperature-gated frost weathering, and littoral
wave attack. Same world-fingerprint event as the biotic (S10), erodibility
(0029/0030) and tectonic-history (U8/0044) flips — terrain shape and the
strata record change everywhere, and worlds made before are not reproducible
under it (the byte-identical `DeepOverrides { full_agents: Some(false) }`
channel still reaches the pre-0034 path). **The seven agent magnitudes are
UNRATIFIED** — they ride at their `DeepConfig` defaults and are appearance-
class numbers the user will judge live, station by station, in a guided walk
(journal/0047's tour map); this flip ratifies the roster being ON, not the
numbers. The knobs, by name and current default: `eolian_deflation` 0.02,
`eolian_arid_precip` 0.32, `eolian_deposit_frac` 0.25, `frost_weathering_gain`
1.5, `frost_band_width_c` 12.0, `wave_erosion` 0.05, `wave_band_m` 30.0.

> **⚠ THE APPEARANCE HOLD ON THESE NUMBERS IS RELEASED — user, 2026-07-29.** *"I do not
> care if frost/wind magnitudes increase 45×, do not try for byte identicality on
> `calibrated_rates`, I do not care about my previous ratification on looks there. We just
> need to move fast."*
> **`scale_erosion_rates` may now scale `wave_erosion`, `eolian_deflation` and
> `frost_weathering_gain` with everything else.** The exclusion existed to protect three
> appearance calls the user judged live; the user has released them, **withdrawing their own
> prior ratification to do it**. Byte-identicality on the calibrated arm is explicitly
> **not wanted** — do not spend a slice preserving it.
> - **The live magnitudes tour is no longer a gate.** It may still be a walk worth taking; it
>   blocks nothing.
> - The paragraph above stands as the **dated record of why they were excluded** — that
>   reasoning was sound and is simply moot now. `stubs.md` § 28 is dissolved; it had recorded
>   that the calibration moved four core rates 45× and left these three behind, making them
>   45× weaker **relative to the landscape they act on** (wave quarry 0.10 % of export, eolian
>   dust 0.01 %).
> - *Recorded loudly because a withdrawn ratification is the corpus's worst-kept event: the
>   ratification is written down and the withdrawal usually is not, so a stale hold outlives
>   its own reason and blocks work nobody is blocked on.*

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
cyclic margins). **USER-RATIFIED for 3e (2026-07-19): A always-on + C refinement.**

> **⚠ SUPERSEDED IN PART — 2026-07-25, see [`flow.md`](flow.md).** The
> **expression half** of § 3e-2 (the channel-line corridor, the two-way negotiation,
> the refinement operator, and anything implying a *drawn* carve) is **retired**: flow
> is recorded as **flux on 3D faces + facts on strata**, and the channel is the
> **expression of a mass budget**, never a deformation. Two specific corrections:
> **(a)** decision 1's *"the corridor never crosses a drainage divide"* binds the
> **FREE/surface regime ONLY** — bound (groundwater) flow genuinely crosses surface
> divides (artesian basins, karst piracy), and carrying the rule over unqualified
> forecloses regional groundwater; **(b)** the driving field is **potential/head**,
> not elevation. What SURVIVES: drainage is advective and decided-once-coarse (the
> halo argument, corrections #8), and descent-along-potential is a hard constraint.

**3e-2 decisions (RATIFIED 2026-07-19, second session, unpacked walkthrough):**

1. **Drainage handoff**: drainage and contributing area are decided ONCE at
   the coarse A tier; C inherits them as fixed boundary fields and never
   re-routes (the halo theorem forecloses fine re-derivation — fluvial
   reach is global, corrections #8). Ratified **with the river-conditioning
   mechanism as the excuse (user)**: within a corridor ~one coarse cell
   wide, the channel line wanders by deterministic noise bent toward
   refined local lows, while the refined surface is simultaneously nudged
   so elevation descends monotonically along the flow path and
   cross-slopes tilt toward the channel — two-way negotiation below a
   frozen macro topology. Hard constraints: descent-along-flow is a hard
   refinement constraint (not cosmetic), and the corridor never crosses a
   drainage divide (no catchment theft — the line between lerp and
   re-routing).
2. **Width cap**: permanent architecture, not apology — "coarsen the
   cause, refine on approach" (user: "the cost of global and scale").
3. **Stitching**: elevation injects at the finer lattice level inside a
   refined region (shared-ancestor interpolation absorbs the boundary —
   the 0015 mechanism, third use); the strata record commits interior-only
   after simulating with the measured 16–24-cell halo; record contacts
   step at the commit boundary and at cell scale — and per method rule 5,
   every such contact must be dressed by procedural tricks before it
   reaches the eye (contact-softening IN SCOPE for 3e-2).
4. **Approach trigger**: proximity at region granularity within
   LOOKAHEAD_BOUNDS plus lead margin, refined async (~3.6 s measured);
   deterministic and approach-order-independent by construction given
   decision 1.
5. **Calibration (iteration↔Myr): RATIFIED 2026-07-19 (user)** — the
   **Phanerozoic register**: the recorded span calibrates to ~500 Myr, so
   every recorded unit is life-adjacent time (S10 biotic annotation valid
   anywhere in the column; evolution horizons fit inside the record; ages
   at human-intuition scale). The pre-record deep past is not simulated:
   basement ages are procedural flavor — "**procedural hacks for the
   boring billion**" (user) — assigned plausibly from province/depth, lore
   below the record. A world-creation calibration knob is the deferred
   eventual shape (sim-depth-knobs doctrine); ~500 Myr is the default.
   Must be labeled before persistence/knowledge commit ages as facts —
   after that, re-labeling is a compat break.

   > **FLAG 2026-07-26 — the register is stipulated and the engine's rates do
   > not honour it (journal/0111, corrections #56, stubs #24).** At 200 epochs
   > this register means **2.5 Myr per iteration**, and nobody had ever divided
   > the engine's metres-per-iteration constants by it. Measured on the shipped
   > world: catchment-averaged denudation **0.0110 m/Myr** — **9× slower than
   > the slowest surface ever measured on Earth** (McMurdo Dry Valleys /
   > hyperarid Atacama, 0.1–1 m/Myr), 493× below the global `10Be` outcrop
   > median (5.4, Portenga & Bierman 2011), and stripping **5.48 m** over the
   > full span where a real craton strips **5–10 km**. Denudation is 2.7 % of
   > rock uplift, so the landscape has never approached topographic steady
   > state. The *shape* of the model is right — weathering-limited, creep-routed,
   > rivers minor, regolith armouring its own front: a textbook low-relief
   > craton — and only the **rate** is wrong. **This § 3e's own owed item,
   > "calibrate iteration↔Myr against a real orogen", is the named heir**, and
   > `examples/denudation_probe.rs` is its acceptance instrument (its literature
   > band table is the test; the world should land in the 1–10 m/Myr
   > stable-craton band). The resulting numbers are appearance-class and
   > user-owned.
**Recommendation: A always-on + C refinement — CONFIRMED by S9b
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

## The payoff layer — ore genesis (direction DECIDED 2026-07-20, user)

Every engine above writes a findable resource signature, and **ore is the
reward for reading the world correctly** — the design principle that
decides which fidelity investments are worth it. Coal already closes this
loop (biofacies → seam → a player digs); nothing else does. The roster and
each deposit model are their own design pass (content is user-owned), but
the standing examples: placers downstream of eroding source lodes (pure
intersection of existing erosion × hydrology), hydrothermal veins along
faults and around intrusions, evaporites in closed arid basins (aridity is
already tagged), porphyry systems at arcs, impact-related ores. When a
process pass is designed, ask: what does this process leave behind that a
player who understands it can find?

## Standing question for every future pass

"What does this look like in reality, and how did it get there?" — if the
pass can't answer in those terms, it isn't ready to be declared in the
graph.

> **⚠ BACK-POINTER ADDED 2026-07-29 (baseline sweep S9/S9-3) — the ratification below is
> UNTOUCHED; this is the missing edge, not a reinterpretation.** `docs/design/stubs.md` § 28
> (*the-agent-magnitudes-the-calibration-left-behind*, added by `journal/0114`) records a
> consequence and says it *"is not recorded anywhere else"* — correct, and this is the site
> where it was missing: the erosion calibration moved the four core rates **45×** and left
> the wave/wind/frost agents where they were, so **relative to the landscape they act on
> they would be ~45× weaker than the day these magnitudes were judged.**
> **This is a PRE-FLIP blocker, not a live defect:** `EROSION_CALIBRATION = 45` sits behind
> `calibrated_rates`, **OFF in production** (`deeptime/field.rs`), so today's shipped world
> is the world these magnitudes were ratified against. **Whether the ratification survives
> the flip is the USER'S call and nothing here presumes it** — flagged so it is visible at
> the ratification site *before* the flag flips. See also `ROADMAP.md`: the live-magnitudes
> tour is more owed than before.

> **Magnitude ratification — DECIDED 2026-07-21 (user, live guided tour,
> journal/0049).** The wind and frost magnitudes are RATIFIED as-built
> (`eolian_deflation` 0.02, `eolian_arid_precip` 0.32, `eolian_deposit_frac`
> 0.25, `frost_weathering_gain` 1.5, `frost_band_width_c` 12.0) — judged
> live at their strongest stations; legibility debts belong to the
> veneer/carry-H/partials work, not these knobs. The WAVE magnitudes are
> NOT ratified: station 5 confirmed the null (0.68 m total at the world's
> most-attacked coast) and the user directed "more dramatic by default" —
> ~~retune slice Sequenced (target class: cuts that survive the 0.9 m voxel
> and read as platforms/notches, ~10–40× with `wave_band_m` widened for
> stranded terraces from the sea-level cycles)~~ **🔴 THE RETUNE WAS STRUCK BY THE USER THE
> SAME DAY (2026-07-21) — there is no retune slice.** Marked here 2026-07-29 (baseline sweep
> S9/S9-2); the strike has been on the board at `ROADMAP.md` § *Wave-magnitude retune* since
> the day this paragraph was written, and both sides cite journal/0049.
> > *"That whole mechanism changes after water machinery. that would be a **bandaid**,
> > against our standing rule against bandaids. can revisit later."* (user)
>
> **The confirmed null rides as-built** until the fetch model lands: **wave expression is a
> consumer of the water design pass, not a tuning slice.** *Note this doc still named a
> numeric target (`~10–40×`) for a slice the user refused as a bandaid — which is exactly
> what [[no-bandaid-tuning]] exists to prevent, sitting in the doc that owns the decision.*
> **Heir mechanism (user,
> same session): wave energy as a fact about the water body — fetch from
> S11's body graph × the 0037 wind field, evolving over the run — replaces
> the global constant when built.**
