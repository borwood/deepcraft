# Flow — the one process

**RATIFIED 2026-07-25 (user, "standing blessed").** This document supersedes the
river half of `water.md` and **retires `earth-processes.md` § 3e-2's expression
half** (its *constraint* half survives, qualified — see § 7). It is the design
under which drainage, rivers, groundwater, caves, karst, glaciers, wind, and
mass-wasting are **one process recorded one way**.

> **The prize (§ 8) is the acceptance test:** cut a cliff face and read a real
> stratigraphic story off it, with **no landform-specific code path anywhere**.

---

## 0. The principle

Strip the names away and there is one process:

> **Matter moves down a potential gradient, through a medium that resists it,
> carrying a load it exchanges with the substrate as conditions change.**

Erosion is load taken up. Deposition is load given back. A void is where matter
left and nothing replaced it. A stratum is where matter arrived. A landform is
the **integral of the exchange**.

**There is no separate "carving."** There is only the budget, spent somewhere.
Any mechanism that *draws* a channel instead of *spending* a budget is a second
model of the carve and is forbidden by this document (see § 6, `RiverSeg`).

Regimes differ in four numbers and one field — **viscosity, density, competence,
resistance**, and *the field they follow*:

| regime | field | competence |
|---|---|---|
| fluvial | head | ceiling ∝ energy |
| eolian | wind | silt–sand only |
| glacial | ice surface | everything, indiscriminate |
| gravity / mass-wasting | slope | cohesion-limited |
| turbidity / contour | density | episodic, enormous |
| hydrothermal / magmatic | thermal + head | solute + melt |
| solute (karst) | head | dissolution-limited |

`material-behavior.md` § 13.2 already ratified *"transport is a FAMILY, not a
water thing."* This extends it one step: it is **the family of all flow**, and
erosion is what flow does to solids on the way past.

---

## 1. The three collapses (RATIFIED 2026-07-25)

### 1.1 Free and bound are an OCCUPANCY, not two systems

`material-behavior.md` § 2 gives forms as *"the closed set of occupancy modes,"*
and **Fluid is already one**. Therefore:

> **Free** = fluid occupying **open space**. **Bound** = fluid occupying **pore
> space**. **free↔bound is an EDGE on the form-transition graph (S-8)** — the
> fluid's exact analogue of `Structure→Loose`.

Consequences that are now *free* rather than bespoke:

- A **spring** is the edge firing at the surface — a **derived outlet
  condition**, never an authored feature (already forced by corrections #14).
- A **cave** is free-phase flow below the surface, persisted as a **void
  interval**.
- **Karst** is the dissolution agent summing on the weathering edge —
  `Cause::Dissolution` **already exists in the enum, dormant** (corrections #17:
  *"karst arrives by adding a term to the phase this fix already couples"*).
- **Cementation / diagenesis** is deposition in the **bound** phase (load given
  back into pores) — the pore-fill form already carries it.
- **Speleothem** is deposition into a **void** from a bound→free transition.
- The two regimes keep their measured, architectural locality difference: bound
  is a bounded relaxation (S11 halo 4–11 cells); free is **connectivity, not
  local at any radius**. That difference is why free water persists **bodies**
  and derives voxels, while bound water is derived in a halo.

### 1.2 In a stratigraphic record, depth IS time

The palimpsest collapses two axes into one. A flow fact bound to a unit slot
answers *"where in the strata"* and *"when"* with the same integer. Bury it and
it is history; erode down to it and it is exposed again — automatically, because
that is what a record does.

**An unconformity is not missing data — it is a flow signature.** It says: *here,
flow removed the time.*

Two rules that keep this honest (§ 5 adversarial pass):

- **Correlate by CHAPTER, never by slot index.** Depth is time *within a column*.
  Across columns, surfaces are **diachronous** (a shoreline deposit is one slot,
  many ages). `DepUnit` carries a chapter stamp; use it.
- **The base is superposition; the FACTS are cross-cutting.** A *young* fact on an
  *old* slot — a dike, a cave fill, a dissolution front — is correct and required.
  This is the geological principle of cross-cutting relationships, and our
  chapter-stamped facts already express it. Do not "fix" it.

### 1.3 The atom

> at **(cell, stratum-slot)**, flux crossed **faces F** with magnitude **M**, in
> **form P** (free/bound), carrying **load L** of **fluid f**, under **cause C**
> (the mover), at **chapter K**.

Amazon, aquifer, esker, lava tube, delta, gorge, delta-front turbidite: each is a
**query over that atom**. Nothing on that list gets its own type.

---

## 2. Flux on FACES — the representation (RATIFIED 2026-07-25)

### 2.1 Why the receiver tree dies

`flow_to: Option<u32>` is **one out-edge per cell**. A tree can represent
convergence. It **structurally cannot represent divergence**.

> **No distributaries. No braiding. No anabranch, no alluvial fan, no delta.**
> Not "poorly modeled" — **unrepresentable**. The Mississippi delta cannot exist
> in a receiver tree.

It is also, today, computed at **~14.7 km** on **pre-erosion** topography, i.e.
routing water across a landscape deep time subsequently destroys.

### 2.2 What replaces it

**Flux across faces**, carrying magnitude + the atom of § 1.3. This yields:

- **Convergence** (many in-faces) **and divergence** (many out-faces) — deltas,
  braids, distributaries, fans.
- **Parallel flows through one cell** — multiple independent crossings, which a
  spanning tree forbids by construction.
- **Seamlessness as an invariant, not an achievement:** a face is **shared** —
  cell A's east face **is** cell B's west face. Refinement built on face data
  agrees from both sides *by construction*.

  > **⚠ UNDER-SPECIFIED — the slot-pairing rule (S19, 2026-07-25).** "The face is
  > shared" holds at the **cell-pair** level, but adjacent columns do **not** have
  > aligned slot indices, so *which slot pairs with which* is a rule this document
  > owes and does not yet state. **Pairing by slot index is silently wrong** — § 1.2
  > already forbids index correlation, because surfaces are **diachronous**.
  > Candidate rule, **NOT RATIFIED**: contemporaneous/**free** flow pairs by
  > **chapter**; **bound** flow pairs by **paleo-elevation at that chapter** (water
  > moves laterally through material at the same head, regardless of when that
  > material was deposited). These are genuinely different rules and the split may
  > be more than an early slice should carry. S19 also notes the pairing choice is
  > uniformly a **2×** on cost — the same 2× as D4/D8 face counts, and easily
  > confused with it. **Resolve before any refinement/expression slice; a slice that
  > picks a rule must say so loudly rather than pick one silently.**
- **3e-2's divide constraint becomes structural**: refinement may place a channel
  anywhere *inside* a cell, but it must enter and exit through the **recorded
  faces**. The rule stops being something to remember.

### 2.3 Faces are 3D (§ 5 foreclosure ①)

**Lateral faces alone forecloses most of the hydrosphere.** The face set is:

- **Lateral faces** — cell ↔ cell, *per stratum slot*.
- **Vertical faces** — slot ↔ slot, *within a column*.
- **Boundary faces** — the top face exchanges with the **atmosphere** (a source
  for precipitation, a **sink for evaporation/transpiration**); the seaward
  boundary with the ocean (base level).

Without vertical faces, **infiltration, springs, artesian rise, karst capture,
evaporation and precipitation have no representation at all.** A losing stream
*is*: lateral flux at slot N → **vertical** flux down → bound flux at slot N−5 →
spring where phase flips at the surface.

### 2.4 The driving field is POTENTIAL, not elevation (§ 5 foreclosure ②)

"Flow goes downhill" forecloses **artesian basins, capillary rise, thermohaline
circulation, hydrothermal convection, and density/turbidity currents.** The field
pass computes a **potential/head field** (elevation + pressure, and for density
flows a buoyancy term); flow descends *potential*.

> **The free regime's divide constraint does NOT bind the bound regime.** Regional
> aquifers and karst conduits genuinely cross surface drainage divides — the Great
> Artesian Basin, the Ogallala, karst piracy. `3e-2` decision 1's "never crosses a
> drainage divide" is hereby **qualified: it binds FREE/surface refinement only.**
> Carrying it over unqualified would foreclose regional groundwater.

Note the existing bound-water proxy `H = y + sat` is explicitly *unconfined, not a
Darcy solver* (hydrology-priors § 321) — confined/artesian behaviour is exactly
what a real potential field adds.

### 2.5 The fluid has an IDENTITY (§ 5 foreclosure ③)

The flow carries a **fluid material id**, not an assumption of water. Otherwise
**lava tubes, magma, brine, CO₂ and ice** are foreclosed. Form = occupancy;
fluid = material; rheology = material properties (a glacier is a solid whose
viscosity is ~10¹³ — flow, not a special case).

### 2.6 Divergence: what is structural vs what is TUNABLE (2026-07-25, user-raised)

The slice-1 measurement (175,320 divergent `(cell,chapter)` pairs, 7.378 %) mixes
two claims that must be kept apart, because only one of them is a property of the
design:

**STRUCTURAL — true at any cadence, and the reason the receiver tree dies.** A
receiver has **exactly one out-edge per cell**; a face record has **N**. The
representation *admits* divergence unconditionally. No tuning makes a tree able to
hold a delta, and none makes face flux unable to.

**TUNABLE — the observed COUNT is a product of the aggregation window.** Two
cadences exist and they are different knobs:

| knob | what it is | today | effect on divergence |
|---|---|---|---|
| **pass `period`** (§5 RATE) | how often the pass FIRES — a *sampling* rate | `1` (every epoch: integrate, never sample) | none — firing less often would *lose* epochs, not merge them |
| **the aggregation window** | how many epochs sum into one record entry — the record's *time granularity* | one tectonic **chapter** = 25 epochs | **this is the whole effect** |

Within a single epoch the solve hands back **one receiver per cell**, so at a
one-epoch window the divergence count is **zero** and the tree structure reasserts.
Widen the window and divergence appears, because the terrain moves under the flow
and the steepest-descent receiver **switches**. Set `K` (chapters) and you set the
count.

> **Therefore: every divergence currently in the record is TEMPORAL (avulsion),
> never SIMULTANEOUS (concurrent distributaries).** Avulsion is the honest physical
> origin of braid plains and fans, so recording it as divergence is faithful — but a
> delta with two channels flowing *at once* is **not yet representable**, and no
> cadence setting makes it so.

**What would make it so: a multi-flow-direction (MFD) solve** — and that is
deliberately NOT a record change. The record may only describe water the erosion
pass actually moved (else the flux record and the mass budget disagree, which is § 3's
whole point). **MFD is a SOLVE change and belongs with the potential/head field**
(continuation (a)): a head field partitions flux across several receivers naturally,
where steepest-descent cannot.

**A gap in the § 5 cadence vocabulary, named here.** `material-behavior.md` § 5 gives
the scheduler two axes — **ORDER** (topo-sort) and **RATE** (`period` + `dt`). It has
**no name for the aggregation window**, yet that window is what decides the record's
time resolution and, here, an acceptance number. `period=1` + chapter-bucketing is the
correct pairing (integrate everything, aggregate at the record's own semantic
granularity — chapters are real: `DepUnit::chapter` stamps units and units never merge
across a chapter boundary). But the window deserves to be a **declared, first-class
third axis** rather than an implicit constant, especially before episodic events (§ 5
limit 2) force sub-chapter resolution. **Open — see § 9 Q7.**

---

## 3. The carve is a MASS BUDGET

The erosion / weathering / dissolution passes already move material. The flux
record says **where** and **how much**. The channel is what remains when the moved
material is subtracted **along the recorded path**, and deposition biases toward
the low points on that path.

> **The flux record is the substrate other processes bias to — not a second
> geometry model.** (This is the standing objection that killed the operator
> sketch of 2026-07-24: an operator that *deforms* to imitate the result is
> forbidden; an expression that *spends the budget* is the design.)

`material-behavior.md` § 13.8's flow-biased sub-cell fill is the consumer: it
finally has a path to bias toward.

---

## 4. Tiers and the single seam

| tier | may be | must be |
|---|---|---|
| **Deeptime compiler** | arbitrarily rich, slow, global, iterative (gen is free) | deterministic, seed-owned |
| **The record** | large | **the only channel between tiers** |
| **Refinement operators** | clever | **pure fn of (record, shared face data, position)** |
| **Present runtime** | event-driven, lazy | derive what it can; persist only connectivity |

> **The record is the only seam.** Deeptime writes; refinement reads. If refinement
> needs something, **deeptime must have recorded it.** This is why "compute
> drainage every epoch and discard it" is the deepest defect in the present
> design: we ran the process and threw away the evidence.

**The pure-fn chunk constrains everything above it.** A chunk must produce
identical voxels regardless of which neighbours are resident. Refinement may
depend on the record, on **shared face data**, and on **position-addressed
noise** — *never* on a neighbour's refined output. Face flux is precisely the
primitive that satisfies this, which is why **the seamlessness requirement and the
divergence requirement have the same answer.**

Refinement is therefore not "carving a shape" but **solving a small
boundary-value problem inside a cell**, with face fluxes as Dirichlet conditions
and the load budget as mass.

---

## 5. Adversarial pass — foreclosure check (2026-07-25, user-requested)

Every phenomenon below was tested against the model. **Three forecloses were found
and fixed in-document** (① § 2.3, ② § 2.4, ③ § 2.5). The rest are representable;
the last group are honest, named limits.

**Representable:** river/stream/channel · meander/braid/anabranch · confluence &
distributary · delta & alluvial fan · **knickpoint retreat (Niagara)** as a
migrating incision signature across chapters · gorge/canyon as sustained incision
+ unconformity · floodplain/levee/oxbow/terrace as a channel that *moved* · lake /
inland sea / playa · **the mono-ocean as persistent base level across
supercontinents** · abyssal plain · cave / phreatic tube / vadose canyon ·
sinkhole & cenote · **spring/resurgence (derived)** · losing stream & dry valley ·
aquifer/aquitard, water table, vadose/phreatic zones · infiltration, percolation ·
seep, wetland, oasis · **artesian basin (needs § 2.4)** · turbidity current,
submarine canyon & fan · contourite · longshore drift, estuary · glacier, ice
stream, U-valley, fjord · moraine, **esker (a river inside ice)**, outwash ·
dune/loess/yardang/desert pavement · landslide, debris flow, lahar, talus,
colluvium · **lava flow & lava tube (needs § 2.5)** · dike/sill emplacement ·
hydrothermal circulation & vein/ore fluids · dissolution & speleothem/tufa/
travertine · cementation/diagenesis · **placer concentration** · permafrost as a
seasonal aquiclude (fluid phase change) · **antecedent rivers** (incision keeping
pace with uplift, since both are per-epoch) · groundwater residence time / fossil
water (a fact attribute).

**Named limits — honest, not fatal:**

1. **Oscillatory / tidal flow** nets to ≈ 0 and would read as "no flow." Needs a
   **gross-energy term separate from net flux** (tidal flats, herringbone
   cross-bedding). Not in the first slice.
2. **Episodic catastrophes** (turbidite, jökulhlaup, lahar) average away inside a
   chapter, yet the *deposit* (a graded bed) is the entire signature. Needs event
   facts or a sub-chapter mechanism — relates to the § 5 rate/cadence axis.
3. **Sub-cell lateral channel migration** (meander belts at 460 m) is
   refinement-tier; the record can carry a mobility/energy hint, not the path.
4. **Evaporite deposition** precipitates because the **carrier left**, not because
   competence fell — a distinct deposition trigger from the competence ceiling.

---

## 6. What this retires

- **`pregen/hydrology.rs`** — a D8 receiver tree at ~14.7 km on **pre-erosion**
  topography. Wrong resolution, wrong time, wrong topology.
- **`RiverSeg` / `carve_rivers` / `BANK` / `RIVER_REACH`** (`collapse.rs`) — the
  second carve model. Its riverbed flag is **discarded at both call sites**
  (`let (elev, _riverbed) = …`), and it produces no visible rivers in the shipped
  world. It is a chord-drawing deformation with no relationship to any material
  the erosion passes moved.
- **`Cell::{flow_to, river, discharge}`** — the tree's leftovers.
- **`earth-processes.md` § 3e-2, expression half** — superseded. Its
  *constraint* half survives **qualified by § 2.4** (free regime only).

## 7. What stays — the tenacious spines

- **The pass-runner** — flow passes declare `{reads, writes, rate}`.
- **The field / cellular split (§ 5)** — and it is *exactly* right: the **flow
  field** (potential, routing, table) is a **field pass**; **load exchange**
  (erode/dissolve/deposit) is a **cellular pass running edges**. The advective
  non-locality lives in the field; the edges stay local. The most load-bearing
  thing we already own.
- **S-9 base + facts / the `FactLedger`** — the home for flow facts.
- **Forms + the transition graph (S-8)** — subsumes free/bound.
- **Agents summing on edges (§ 4)** — `Cause` becomes **the mover**: Fluvial,
  Eolian, Glacial, Gravity, Marine, Hydrothermal, alongside Dissolution.
- **Condition-fields over field-ids (§ 14)** — head, saturation, temperature are
  one shape; the geotherm proved it.
- **The drainage *solve*** — priority-flood / route / accumulate is good numerics.
  Only its **output representation** changes (tree → face flux).
- **S11's runtime model** — bodies-not-links (corrections #14), `sat.rs` bound
  water, **zero cave-specific code**.
- **The interval-log fill contract** (proposed, `water.md` § Session capture)
  becomes **necessary** — voids/conduits are intervals, not a heightfield.

---

## 8. The prize — stated as the acceptance test

Not void statistics. Not a metric. **A cross-section.**

> Cut a cliff face and read it: *a channel gravel with its placer streak, fining
> upward into floodplain silt; an unconformity where the river left and took ten
> chapters with it; a carbonate above, dissolved along its bedding into a phreatic
> tube, now dry because base level fell; the tube's floor littered with collapse
> breccia; a spring line where the table still meets the hillside two hundred
> metres downslope.*

Every one of those is **the same atom queried at a different slot.** If the model
produces that face **without a single landform-specific code path**, it is
faithful.

---

## 9. Open questions (carried)

1. **Cost — MEASURED 2026-07-25, `docs/spikes/S19-flow-record-cost-results.md`.**
   Ratified stance (user): *"gen is free, cost taken as it comes, faithfulness above
   all"* — but resident memory is still sacred. What S19 settled:
   - Geometry: **297,025 cells @ 460 m, K = 8 chapters, 200 epochs**; land 14.9 % of
     cells but **76.8 % of all stratum slots**. Units/cell is **strongly bimodal**
     (land mean 96.0, marine mean 5.1, overall mean 18.64, median 5, max 478) — **any
     allocation sized on the mean is wrong in both directions.**
   - Baseline `DeepField` residency **T = 162.57 MiB**, of which the record is 90.8 %.
   - **Flow facts are causally triangular**: a slot deposited in chapter `c` cannot
     carry a fact from before `c`, so the real count is `Σ(K − chapter)` = **57.7 %**
     of the naive `slots × K` — **a free 1.73×**. Never allocate the rectangle.
   - **The dense/sparse fork is decisive**: dense layouts land at **1.9×–5.5× T**;
     sparse-with-the-full-atom crosses **1× T at 4.30 % face-sparsity, 4× at 17.94 %**.
     The CSR index floor is negligible (0.056× T) — **the payload is the whole
     constraint.**
   - **The measured anti-pattern to avoid:** `Vec<Vec<_>>` keyed per (cell, slot) —
     today's `FactLedger` — is **98.8 % empty inner Vecs, 89 % of its heap being empty
     headers** (+156.91 MiB, 1.96× T, when `weather_inventory` is on).
   - **Residual unknown:** the actual non-zero face fraction, which cannot be known
     before the recording slice's solve exists. That single number closes Q1.
2. **Sub-face parallel channels.** One flux per face merges two parallel channels
   crossing the same face; refinement may re-split them from finer topography.
   Whether the *record* must distinguish them is deferred to measurement.
3. **The gross-energy term** for oscillatory regimes (§ 5 limit 1).
4. **Episodic events** within a chapter (§ 5 limit 2).
5. **Which fluids ship first.** Water is slice 1; lava/ice/brine ride the same
   atom but need their own competence curves.
6. **S14 is superseded as posed.** It asked "can drainage refine under coarse
   boundary conditions?" Face flux answers the *seamlessness* half structurally;
   the open half is only how finely the interior may be expressed.
7. **The aggregation window should be a declared third scheduler axis** (§ 2.6).
   § 5 has ORDER and RATE but no name for "how many epochs sum into one record
   entry" — yet that window sets the record's time resolution. Related: whether
   flow facts stay per-**chapter** (25 epochs, today) or go per-**epoch** (~25×
   the payload). Per-chapter cannot express *"the river moved at epoch 40"*, which
   was the framing that opened this arc. **User call, not yet made.**
8. **Simultaneous divergence needs an MFD solve, not a record change** (§ 2.6).
   Sequenced with the potential/head field (continuation (a)), never ahead of it.

---

## 10. Justifications that WILL expire (write them down now)

Recorded 2026-07-25 at the user's direction — *"deformation/compaction is/are a
beast that WILL exist, just a question of when."* These are **A-2 traps armed in
advance**: rules that are correct today **only because a feature is unbuilt**, and
that will fail silently rather than loudly when it lands. Prose cannot fail a build
(corrections #29), so each names the trigger that invalidates it.

### 10.1 Paleo-elevation pairing is trivial ONLY because strata are layer-cake

**Expires when: structural deformation lands** (`stubs.md` sibling gap — *"every
stratum lies horizontal regardless of history"*; the `unconformity` flag is the only
deformation modelled).

Pairing bound flow by elevation (§ 2.4) is easy today because a bed at depth *d* in
cell A is at depth *d* in cell B — layers are flat, so elevation and bed identity
coincide. **The moment beds tilt or fold they diverge**, and "same elevation" stops
meaning "same aquifer." A **confined aquifer is a permeable bed between aquitards,
and water flows ALONG the bed** — up-dip, down-dip, wherever it goes. So under
deformation, confined flow must pair by **bed identity (≈ chapter)** and only
*unconfined* water-table flow pairs by elevation. **The free/bound split is a proxy
that fails on the confined case**; the real discriminator is *what confines the
flow*: a **material horizon** (contemporaneous surface, or a bed) pairs by horizon
identity; a **potential surface that cuts across strata** (the water table, a head
front) pairs by elevation/head.

### 10.2 Paleo-elevation is a running sum ONLY because nothing compacts

**Expires when: compaction / lithification changes a unit's thickness over time.**

Slot elevation at chapter `c` is currently `surface(c) − Σ(thickness above)`, a plain
sum, because a `DepUnit`'s thickness never changes after deposition. Under compaction
the sum is no longer valid — thickness becomes a function of burial depth and time,
so **every paleo-elevation query becomes history-dependent**, and any cached or
materialized elevation goes stale. Anything that stores derived elevation must be
re-derivable, or it becomes a summary wearing an authority's clothes.

### 10.3 Pairing is IDENTITY, never CONNECTIVITY — keep them apart

Not an expiry but the rule that keeps 10.1/10.2 tractable, and it is easy to lose:
**pairing decides which slots are ADJACENT; permeability decides whether flow
PASSES.** Chapter-pairing correlates *contemporaneous* units, but facies change
laterally — a sand in A may be a mud in B at the same chapter: same time, **not the
same aquifer**. Get identity right and let the flux be **zero** where the contrast
blocks it. Any attempt to make the pairing rule itself encode hydraulic connection
will fuse two concerns into a rule nobody can reason about later.
