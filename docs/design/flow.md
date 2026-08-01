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
  >
  > **✅ RESOLVED — § 11.5 ratified the principle THE SAME DAY this flag was written,
  > and the rule was RE-ARGUED FRESH and CAUTIOUSLY RATIFIED 2026-07-29 (see the
  > banner in § 11.5).** This flag was never stamped when § 11.5 landed 360 lines
  > below it, and three 2026-07-29 documents propagated "not ratified" from here —
  > corrections **#75**. The 2× note is also not the pairing's cost: it is
  > gross-vs-net (both directed halves kept), decided and shipped in the record.
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

#### 2.4.1 The BOUNDARY CONDITIONS — a silence this document had, closed by the build

**This section was under-written and the build had to supply it** (journal/0098, flagged
honestly as *"the document is underneath the work, not wrong"*). § 2.4 said head is
"elevation + pressure, plus buoyancy" and stopped — but **that formula is the easy half;
every modelling judgement lives in what supplies the boundary conditions.** Recorded here
so the design doc carries them rather than only `head.rs`:

| where | condition |
|---|---|
| submerged cell, or the domain border | **Dirichlet** at the sea stand / base level |
| **unconfined** cell holding free water (a lake, or a stream carrying ≥ `STREAM_ANCHOR_AREA` of drainage) | **Dirichlet** at that free-water elevation — the water table *outcrops* there |
| **unconfined** cell elsewhere | free, but **capped at its own ground**: a water table cannot stand above the land, it discharges — a **seepage face** |
| **confined** cell (a contiguous low-permeability cap over a permeable bed) | free, and **UNCAPPED** — the artesian degree of freedom |

> **That last row is the whole point, and it is the shape to preserve: artesian is not a
> special case in the code — it is the ABSENCE of a cap.** An unconfined cell is pinned or
> capped by its own surface; a confined one is not, so its head is whatever the material
> transmits to it from elsewhere.

Solved as steady `∇·(T ∇h) = 0` by Gauss–Seidel with **alternating forward/reverse raster
sweeps** — deterministic, and information crosses the whole grid in one sweep instead of
diffusing a cell at a time.

**Two traps this cost real work to find, recorded so they are not re-paid.** Both were
false-positive artesian counts, and both were caught *only* because the invariant was
written over a **whole world** rather than a constructed column — § 8's acceptance
philosophy earning its keep:
- **Lakes are not aquifers.** A lake cell is Dirichlet-pinned at its own water surface,
  legitimately *above* its ground — **248 of an initial 308 "artesian" columns were lakes.**
  Any artesian query must exclude them.
- **Priority-flood dust.** A further 232 false positives were ponding of ~2.5 × 10⁻⁵ m; a
  `PONDED_MIN_M` floor is required before "standing water" means anything.

**Honest limit, and it matters downstream:** with **no recharge term** (`R = 0` in
`∇·(T∇h) = −R`) artesian excesses come out in **metres**, not the hundreds a real Great
Artesian Basin gives. That is **stubs #19 (the recharge-free water table)**, heir = a real
water-balance climate. **Anything reading this field for reaction rates — notably the
"weathering is one process" arc, whose requisite R2 this field is meant to satisfy — is
reading gravity drainage with no real precipitation depth behind it, and must say so.**

### 2.5 The fluid has an IDENTITY (§ 5 foreclosure ③)

The flow carries a **fluid material id**, not an assumption of water. Otherwise
**lava tubes, magma, brine, CO₂ and ice** are foreclosed. Form = occupancy;
fluid = material; rheology = material properties (a glacier is a solid whose
viscosity is ~10¹³ — flow, not a special case).

> **🔖 OPEN EDGE — see [`material-genesis-notebook.md`](material-genesis-notebook.md) § 3.**
> This section **decided** that fluid is a *form of a material*. Two things it did not
> settle, and both are load-bearing for genesis: the **inventory** still treats
> `InvForm::Fluid` as *derived, never stored* (§12's own open list carries *"Fluid as a
> stored role vs derived"*), and **solutes have no established shape at all** — the
> working model is that a solution is *"the transport of loose in fluid"* (user,
> 2026-07-26), i.e. the § 13.3 load multiset riding a fluid rather than a separate
> concept. Every precipitation/crystallisation edge in the genesis conversation binds
> through this, so it is the nearest blocker there.

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
(continuation (a)): ~~a head field partitions flux across several receivers naturally,
where steepest-descent cannot.~~

> **The first sentence shipped as written; the struck one is FALSIFIED —
> corrections #54, 2026-07-25.** MFD is indeed a solve change and the record was
> not touched. But it does **not** need `dc:field/head`: the free regime's
> potential is the priority-flood `filled` surface, which was already computed,
> and routing surface water on the *bound* regime's plane would have made rivers
> cross their own drainage divides. See **§ 2.6.1** for what was actually built.

#### 2.6.1 BUILT 2026-07-25 — continuation (b), the MFD solve (journal/0109)

**Shipped, on by default** (`DeepConfig::mfd`, `mfd_exponent = 4.0`).
**⚠ If you arrived here by grepping `mfd_exponent`, read § 2.6.2 next:** uniform `p = 4` was
superseded by the **hybrid χ law** (shipped `p_hill = 1`, `p_chan = 16`), and § 2.6.2 states
that *"uniform `p` is a control, not a shipping mode."* This dated `BUILT 2026-07-25` block
is correct about its own day. The record's
shape did not move a byte: what changed is that the solve now hands the recorder
several non-zero out-faces per cell per epoch instead of one.

- **The partition is Holmgren (1994) with Quinn's contour width**:
  `wₖ ∝ Sₖᵖ · Lₖ`, `Sₖ = Δh / dₖ` along the **true flow-path length** (`1`/`√2`),
  `Lₖ` the face's contour width (`1`/`1/√2`). `p` is the **convergence exponent**:
  `p = 1` is Quinn's maximally dispersive form, ~~`p → ∞` is single-receiver D8
  exactly~~ **`p → ∞` is single-receiver *steepest-slope*** (**corrections #58**,
  2026-07-26 — the codebase's own `the_partition_follows_slope_not_drop` pins a
  case where the limit and `route_cell` choose **different** receivers, because
  `route_cell` takes the steepest **drop** and the partition takes the steepest
  **slope**; they differ on diagonals by the very `√2` this bullet introduces).
  The old solve is still a *near*-limit of the new one rather than a deleted
  alternative — the correction is to the word "exactly", not to the shape.
  **Since 2026-07-26 `p` is spatially varying — see § 2.6.2.**
- **The field partitioned is the FREE-SURFACE potential** — the priority-flood
  `filled` surface, which is `z_bed + depth`: bare ground where the land drains, a
  flat water surface inside every depression. That **is** head for the free regime
  (pressure head is zero at a free surface), so § 2.4's "flow descends potential"
  is satisfied by the plane the solve already computes.
  > **It is deliberately NOT `dc:field/head`, and the reason is § 2.4's own
  > qualification.** That plane is the **bound** regime's potential and it is built
  > to cross surface drainage divides (the Great Artesian Basin, karst piracy) —
  > routing *free surface* water down it would make rivers cross divides too, which
  > § 2.4 says binds the free regime. **Bound MFD — Darcy flux partitioned across
  > faces on `dc:field/head` — is real and unbuilt; it belongs to continuation (c)
  > with the free/bound edge.**
- **A representational floor** (`MFD_MIN_WEIGHT = 1 %`) drops sub-percent shares and
  renormalises. It is not physics: without it every land cell records an entry for
  every downslope neighbour and the archive pays megabytes for noise. The steepest
  share is ≥ 1/8 before the floor, so no cell is ever turned into a sink by it.
- **What had to be re-derived for a DAG**, since the old chain assumed a *tree*:
  - **The traversal order did not have to change**, and that is the finding. The
    priority-flood pop order is strictly ascending in `filled`; every routed edge —
    single or multi — descends `filled`; so the reversed order is a topological
    order of the **DAG** for exactly the reason it was one of the tree. What
    licensed the traversal was never the tree, it was the potential.
  - **Mass** survives on one condition: the shares sum to the whole with **no
    residue**. Normalised `f64` weights sum to `1 ± 1 ulp`, so the last weighted
    direction takes `q − Σ(earlier shares)` instead of `w·q`.
  - **The never-incise-below-the-receiver clamp** becomes *below the **lowest**
    receiver* — the cell still drains as long as it stays above its lowest outlet.
    In the single-receiver limit the D8 receiver **is** the lowest neighbour, so it
    reduces to today's rule exactly.
  - **The energy slope** becomes the share-weighted mean `Σ wₖ Sₖ`, which is not a
    smoothing choice: stream power is `Q·S`, so the total power released by a split
    discharge is `Σ Qₖ Sₖ = Q · Σ wₖ Sₖ`.
- **`recv` changed kind.** It is now the **argmax share** — a projection of the
  partition, not the routing. It is not meaningless (it still answers "which way
  does most of the water go") but it can no longer be read as "where the water
  went", and it is now a summary in ARCHITECTURE.md's sense. Continuation (e)
  should retire it, not merely supersede it.

#### 2.6.2 BUILT 2026-07-26 — continuation (b'), hybrid `p` (journal/0113)

§ 2.6.1 shipped **one exponent for the whole world**, and journal/0109 measured what
that costs in the same breath as it shipped: the **peak catchment collapsed
1,245 → 84 cells**. That number is not a tuning artefact, it is the statement that a
trunk river never accumulates — because a uniform `p` disperses at *every* cell and
the loss compounds down the chain. Named there as *"honestly the wrong long-run
shape"*, and this is its continuation.

**What was wrong is specific, and it is not "the number was too low."** Uniform `p`
applies **hillslope sheet-flow behaviour inside channels.** Real water spreads on an
unchannelised hillslope and **stays in its banks** once it is channelised, and one
exponent cannot say both. Holmgren's calibrated 4–6 band is exactly the *compromise*
a single-exponent scheme is forced into.

**The law: `p` ramps on the channelisation index `χ = A · S²`** — Montgomery &
Dietrich (1988, 1992)'s channel-initiation criterion — from `p_hill` at
`χ ≤ chi_lo` to `p_chan` at `χ ≥ chi_hi`, log-linear between; and **at `chi_hi` the
cell switches to SINGLE-RECEIVER, exactly.** `A` is the cell's drainage area
**lagged one epoch** (see below) and `S` its steepest dimensionless downslope
gradient on the free-surface potential. Shipped: `p_hill = 1`, `p_chan = 16`,
`chi_lo = 3×10⁻²`, `chi_hi = 1.2×10⁻¹`.

> **The hard switch is a MEASURED necessity, not a stylistic preference**, and it
> is why the law takes from *both* literature families rather than one. A smooth
> exponent cannot confine a channel on terrain this smooth: weights go as
> `(Sₖ/S_max)^p`, so a neighbour at 90 % of the steepest slope still keeps 19 % at
> `p = 16`; suppressing it below the representational floor needs `p > 44`, and a
> 95 % rival needs `p > 90`. **Measured: `p = 16` everywhere lifts the shipped
> world's peak catchment only 84 → 145 cells.** The trunk bleeds a fifth of its
> discharge at every hop, and a fifth per hop down a fifty-hop chain is
> everything. So above `chi_hi` the whole discharge goes down the steepest slope:
> **once flow is channelised it is confined, and confined flow takes one path.**
> The ramp beneath the switch is what keeps the transition continuous, so the
> switch fires from `p = p_chan` rather than out of a dispersive state.

> **Why `A·S²` and not `A` alone, which is the obvious choice and is also in the
> literature.** An area-only law would destroy what § 2.6.1 bought. **Deltas,
> alluvial-fan tops and braid plains are the places with the LARGEST `A`** — an
> area-only law makes them the most convergent ground on the world, and concurrent
> distributaries vanish. `A·S²` puts them back on the dispersive side, for the
> physically correct reason: **a delta is where a channel loses its confinement.**
> One law, three regimes:
>
> | regime | `A` | `S` | `χ` | `p` | behaviour |
> |---|---|---|---|---|---|
> | hillslope / interfluve | small | any | low | `p_hill` | sheet flow, spreads |
> | trunk river, gorge, incised valley | large | moderate–high | high | `p_chan` | stays in its banks |
> | delta top, fan, coastal plain | large | ≈ 0 | low | `p_hill` | splits — distributaries |

**Three things the build had to settle, recorded because none is obvious:**

- **The circularity, and the one-epoch lag.** `p` needs `A`; `A` is accumulated
  *from* the weights `p` produces. There is no fixed point to iterate to — a second
  accumulation pass uses an area its own weights then invalidate. So the exponent
  reads the **previous epoch's** drainage plane, which is the same shape as the S10
  biotic coupling and costs nothing (no extra storage, no extra pass). At epoch 0 the
  plane is zero, so the first epoch is maximally dispersive — the honest initial
  condition, and also the physical statement: *nothing is channelised until water has
  run once.*
- **Slopes are normalised by `S_max` before exponentiation — on the hybrid path
  only.** Algebraically a no-op (the renormalisation divides it back out), but it
  puts every base in `(0, 1]`, so a large `p_chan` can never underflow a whole
  partition to zero and **report a draining cell as a sink**; without it the safe
  exponent range is silently bounded by the world's smallest slope. It is **not**
  applied to a *uniform* law, and that is load-bearing rather than an omission:
  `x/1.0` is exact, so § 2.6.1's arithmetic survives bit for bit and **three
  cross-commit fixed points stay byte-reachable** — the pre-MFD world, the pre-2b
  scalar-load world and the anonymous-creep world. The first gate of this slice
  failed on exactly that, which is what found it. The price: a *uniform* law
  inherits § 2.6.1's exponent-range limit. Uniform `p` is a control, not a
  shipping mode.
- **The ramp is rounded to an integer.** A fractional exponent forces `powf` on
  `8 × cells × epochs` ≈ half a billion directions per production run; an integer
  goes through `powi`. At a 460 m tier where `p` is a coarse sub-grid dial, `7` vs
  `7.3` is false precision and the gen-time difference is not. Uniform mode does not
  round, so a probe may still sweep fractional `p`.

**Honest limit at this tier.** At 460 m a cell contains an entire
hillslope-and-channel system, so this is **not** "is this cell a channel" — it is a
**sub-grid parameterisation of how much of the cell's discharge is confined**.
`chi_lo`/`chi_hi` are therefore calibrated against *this world's own* `χ`
distribution (printed by `examples/hybrid_p_probe.rs`), never lifted from a field
study at 10 m. That calibration is **stub #26**.

**Measured on the shipped world** (seed 1337, `Extent::Medium`, journal/0113):

| | D8 | uniform `p = 4` | **hybrid** |
|---|---|---|---|
| peak catchment (cells) | 1,255 | **84** | **298** |
<!-- D8's peak reads 1,255 here against the 1,245 quoted in § 2.6.2 above (from
journal/0109) — the SAME quantity on a world that has since gained material-aware creep
and the 0114 recalibration path. Neither number is wrong; the reconciling sentence lived
only in journal/0113 and is carried here 2026-07-29 (baseline sweep S2/F7). -->

| p99 land catchment | 127.0 | 70.0 | **164.4** |
| top-1 % share of land drainage area | 0.0849 | 0.0248 | **0.0613** |
| land cells with catchment > 100 | 530 | **0** | **2,270** |
| **simultaneous divergence, `(cell, epoch)`** | **0** | 7,548,535 | **7,228,964** |

**95.8 % of § 2.6.1's simultaneous divergence survives**, and the per-chapter count
is *higher* than uniform `p`'s — which is the whole argument for `A·S²` over `A`
alone, measured rather than asserted. Note the p99 row: the hybrid solve carries
**more** moderately large channels than D8, because it collects each channel from a
*fan* of dispersive hillslope cells instead of a single tributary line, then keeps
what it collected. `chi_hi` is swept in the probe and the shipped value maximises
both concentration measures while retaining the most divergence — over-channelising
(35 % of land) *lowers* p99 and top-1 %, because thousands of parallel threads that
never merge are not a drainage network.

**What it does NOT do, stated so nobody measures for it.** journal/0111 measured this
world exporting **0.02 %** of its denudation by rivers, at 0.0110 m/Myr overall.
Hybrid `p` concentrates that 0.02 %. **It cannot move denudation, the facies
gradient, or the appearance of the world**, and the joint supply+transport
calibration that fixes the magnitude is a separate slice which must calibrate against
*this* solve.

**Who it is for.** `north-star.md`'s **refinement tier**, and specifically
**visible river channels** — § 4 says refinement is *"solving a small
boundary-value problem inside a cell, with face fluxes as Dirichlet conditions."*
**A channel cannot be refined out of a flux record that never concentrates.**

**A gap in the § 5 cadence vocabulary, named here.** `material-behavior.md` § 5 gives
the scheduler two axes — **ORDER** (~~topo-sort~~ **authored per world, validated by
`{reads, writes}`** — superseded 2026-07-26, corrections #65) and **RATE** (`period` + `dt`). It has
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
7. **~~The aggregation window should be a declared third scheduler axis~~ — RATIFIED
   2026-07-25 (user): DECLARE IT, never assume it.** See § 11.
7b. *(original wording, kept for the reasoning)* The aggregation window as a third axis (§ 2.6).
   § 5 has ORDER and RATE but no name for "how many epochs sum into one record
   entry" — yet that window sets the record's time resolution. Related: whether
   flow facts stay per-**chapter** (25 epochs, today) or go per-**epoch** (~25×
   the payload). Per-chapter cannot express *"the river moved at epoch 40"*, which
   was the framing that opened this arc. **User call, not yet made.**
8. **Simultaneous divergence needs an MFD solve, not a record change** (§ 2.6).
   ~~Sequenced with the potential/head field (continuation (a)), never ahead of it.~~
   **ANSWERED 2026-07-25 — see § 2.6.1.** The first half was right and shipped. The
   second half was **wrong and is corrections #54**: MFD needs *a* potential, and the
   free regime's one (the priority-flood `filled` surface = `z_bed + depth`) was
   already there. `dc:field/head` is the **bound** regime's plane and using it here
   would have made rivers cross their own divides. (a) *was* correctly sequenced
   before (b) — but because it made the record's **vertical** faces honest, not
   because the numerics needed it.

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

---

## 11. DECIDED 2026-07-25 (user) — the declared window, the self-describing record, the pairing rule

Four calls taken after the slice-1 measurement. Recorded here because § 2.6 proved
that an *implicit* constant had silently become an acceptance number.

### 11.1 The aggregation window is a DECLARED axis — never assumed

**RATIFIED** — *the WINDOW axis. The characterisation of the other two axes below was
setup, never part of what the user ratified, and its ORDER half has since been retired.*

`material-behavior.md` § 5 gives the scheduler ~~**ORDER** (topo-sort)~~ and
**RATE** (`period` + `dt`). It has no name for *"how many epochs sum into one record
entry"* — yet that window sets the record's time resolution, and at slice 1 it was an
implicit constant (one tectonic chapter = 25 epochs) that **produced the divergence
count the slice was accepted on**.

**🔴 THE STRUCK HALF — SUPERSEDED 2026-07-26 (user): ORDER IS AUTHORED, PER WORLD**, and
`{reads, writes}` became the **validator**, not the generator. See `ARCHITECTURE.md`
§ *The engine is plugin-agnostic, and pass ORDER is authored* (DECIDED 2026-07-26),
`material-behavior.md` § 5's ORDER bullet (struck the same day), and **corrections #65**.
*Struck rather than deleted because this section's argument rests on it: § 5 had names for
the axes it did have and **no name for the window** — which holds whichever way ORDER is
decided, so nothing in § 11.1 falls with the strike.*

> **A window that decides an acceptance number must be declared, not assumed.**

User's reasoning, recorded: *"freedom to future mods / ourselves (we are the first
modders)."* A mod authoring a pass must be able to state its own record granularity
the same way it states order and rate — **an undeclared constant is exactly the surface
a third party cannot reach.** This is the north-star's *"authored in a uniform,
self-declaring shape and tuned by data"* applied to the time axis.

**Consequence for § 5:** the cadence model grows a third axis —
**ORDER × RATE × WINDOW**. Sequenced, not built. *(Design-pass input filed 2026-08-01:
`ideas.md` § "The WINDOW axis: a closed aggregator vocabulary" — assistant-proposed, not
decided. First forced customer: χ, the fluvial mobility hint, deferred behind this axis
by record-terms ruling 2.)*

### 11.2 Chapter-vs-epoch resolution is a shipped DEFAULT, not an engine property

**RATIFIED** — *"if we have determined that this is tunable, with aggregation window
declared, then this is immaterial to the engine. it's a question of shipped default."*
Once 11.1 lands, per-chapter vs per-epoch flow facts stop being an architecture
question. **Default to the cheap end (coarser window) for dev-iteration speed; the knob
is exposed for stress tests.** *(Reading CONFIRMED by the user 2026-07-25: "cheap end
for dev iteration, precisely.")*

### 11.3 The record must SELF-DESCRIBE its completeness

**Standing contract**, arising from the marine-sink lever (§ 11.4). Any mode that
changes **what an absent entry means** must be **carried in the record**, never held as
external knowledge. Otherwise absence is ambiguous across worlds, and every consumer
must know how a world was generated in order to read it. **S-9 one level up** — *the
answer carries its resolution.*

### 11.4 The marine-sink lever — gamed out, and why the default is KEEP

**79.38 % of slice-1 entries (31.37 MiB of 40.66 MiB)** are ocean-sink faces carrying
only the cell's own seeded `1.0/epoch` source — nothing drained through them. Dropping
them leaves **9.29 MiB**, and is *information-preserving today* because the value is
derivable from the entry's own absence.

**Two future consumers it problematizes:**

1. **The marine transport pass** (turbidity currents, contourites, longshore drift —
   all listed representable in § 5). Absence would encode *"1.0/epoch of **freshwater
   runoff**"* — a **regime assumption**. Once marine flow is real (density-driven,
   tidal, thermohaline) that default is not merely missing but **wrong**, and absence
   can no longer distinguish **"no flow here"** from **"this regime is not modelled
   here yet."** It fails *silently*, because absence looks identical either way.
2. **The conservation audit, and any upstream walk** (the § 3 mass budget; § 13.8
   provenance). Both must **re-derive** the dropped 79 % — two implementations of one
   quantity that must agree forever, **with nothing able to fail when they drift**,
   since no stored value exists to compare against. That is ARCHITECTURE.md's *"a
   summary is not an authority"*: the derivation becomes the definition. A provenance
   walk additionally terminates at a hole it must know to synthesize, so **every reader
   inherits a shared secret.**

**Why a bare flag would be the worst option:** it makes absence mean two different
things depending on how a world was generated. Hence § 11.3 — if the drop ever ships,
**the record carries the mode.**

**DEFAULT: KEEP — RATIFIED 2026-07-25 (user: "your recommendation appears sound to me,
green").** 31 MiB against a 149 MiB field that is *already 13 MiB below where the
session started*; the record is young and every consumer of it is unbuilt. The honest,
complete default beats the clever one until a real consumer argues otherwise. **The
drop is not forbidden — it is gated on § 11.3**: if it ever ships, the record carries
the mode.

### 11.5 Pairing — the two-mode rule is GREEN; a THIRD mode is named and NOT covered

**RATIFIED (user):** flow pairs by **what confines it** —

- a **material horizon** (the contemporaneous land surface, or a permeable bed between
  aquitards) pairs by **horizon identity ≈ chapter**;
- a **potential surface that cuts across strata** (the water table, a head front) pairs
  by **elevation/head at that chapter**.

Slice 1 implements the first for lateral free-surface flux and records no bound flux,
so nothing is forced. Chapter is a legitimate global time surface (tectonic,
grid-wide), and each column binds it to its own slot independently — § 1.2 applied, not
violated.

> **NEWLY IDENTIFIED, AND NOT COVERED BY THAT GREEN — the conduit/void mode.** A
> **karst conduit** crossing from cell A to cell B is confined by **neither** a
> depositional horizon **nor** a potential surface: it is confined by **its own void
> geometry**, carved by past flow, and it may follow a bedding plane *or* cut across
> beds (a vadose shaft). It therefore pairs by **void connectivity** — a **third
> mode**. It arrives with continuation **(c)** (the free/bound edge + void intervals)
> and must be ruled on then. **Do not assume the two-mode rule extends to it.**
> **ASSIGNED to continuation (c) — RATIFIED 2026-07-25** (user: *"great catch: agree on
> add to c"*). (c) therefore ships **three** obligations, not two: the free/bound edge,
> void intervals, **and the conduit pairing rule.**

> **✅ RE-ARGUED FRESH AND CAUTIOUSLY RATIFIED 2026-07-29 (user, at the fluvial member
> design pass — the tier's first concrete consumer).** Re-derived against the channel
> operator rather than inherited, per the member-pass doctrine; it lands where this
> section landed, now stated as the **THREE-MODE CONFINEMENT RULE**: a flux fact pairs
> across a face by *what confines the flow* — **mode 1**, material horizon → by
> **chapter** (slot derived per column, never stored); **mode 2**, potential surface →
> by **elevation/head at that chapter** (carrying § 10.1's expiry: under deformation,
> confined-bed flow re-pairs by bed identity and only true water-table flow stays
> elevation-paired); **mode 3**, void conduit → by **void connectivity**. Three
> sharpenings from the re-argument:
> 1. **Eroding cells:** flux at chapter K attaches to the **surface as of K** — in an
>    eroding column that is an older unit's exposed top. A young fact on old rock is
>    cross-cutting (§ 1.2), correct and required, not a defect.
> 2. **Within-chapter resolution is the WINDOW axis (§ 11.1)**, not a pairing question.
> 3. **The "2×" in § 2.2's flag is gross-vs-net, not a pairing cost** — already decided,
>    already in the shipped record.
> **Mode 1 is built (`flux.rs`, lateral free flux) and the fluvial member rides it
> exclusively, stated loudly. "Cautiously" binds as in `refinement.md`: modes 2–3 are
> directions, ruled properly when continuation (c) forces them.**
