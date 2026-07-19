# Orogeny recon — earth-process mechanism report (2026-07-19)

Reference notes from a read-only recon of the user's orogeny repo
(C:\Users\13203\repos\orogeny, MC terrain-mod experiment). Idea quarry for
the 3e epoch-indexed formation context work — inspiration, not a map; we
fully own generation and can do better where orogeny compromised for
Minecraft.

## The founding split

Orogeny's thesis (their journal/0010): *a density function can't run a
simulation, but it can sample one that ran elsewhere.* Two worlds — a
stateful coarse sim baked once to a binary (384² cells @ 48 blocks/cell,
~200–320 erosion iterations, 14 plates, K=8 strata slots), and a stateless
per-column sampler reading it (~5–8 µs/column). Their cost philosophy,
worth adopting verbatim: **"compromise at a layer of abstraction that
still supports generation, rather than at the phenomenon itself — coarsen
the cause, never delete it and fake the appearance."**

## Deep-time loop

Continuous fixed-iteration two-plane erosion (SPACE model, stripped):
bedrock stock R + alluvium stock H; each step uplift adds to R,
priority-flood + D8 recompute drainage, stream-power entrains/erodes/
deposits, hillslope diffusion. **Surface geometry is collapsed (final
plane); composition is a recorded history** — per cell, an ordered stack
of (tag, thickness) units. The single most transplant-relevant idea:
the shipped artifact carries a compressed per-column event log, not a
final class. Their monolithic loop had to bolt on what our pass graph
declares (recorder threaded as an optional arg; climate snapshot in a
caller hook; runtime throw to enforce recorder presence).

## Tectonics

Static plates (centers + drift vectors, seeded); analytic Voronoi
boundaries via nearest-two-centers, made fractal by warping the LOOKUP
coordinate (cells stay convex, math stays valid); convergence = relative
drift on the inter-plate axis → orogeny/rift/transform; uplift = static
rate field; mountains = time-integral of uplift minus erosion.

**Declared prehistory** (their 0045): no simulated passive-margin eon, so
they reconstruct two paleo-stands per cell (t=0 surface; pre-orogeny
lowstand) and derive marine cover analytically — carbonate cap where the
highest stand reached the photic band — then consume the cover top-down by
the measured erosion, so breached anticlines expose basement and synclines
keep caps. Admitted limit: "arc age inversion" (analytic volcanic pile
always resolves above sim-time marine — wrong, invisible at their scale).
**We can simulate the Wilson cycle instead of reconstructing it.**

**Fold-phase trick (steal directly)**: folding is applied at sample time
as amp·sin(2π·uplift/period + phase-noise) — **the phase coordinate IS the
uplift value**, so iso-phase surfaces are uplift contours, fold axes track
the range front by construction, limbs mirror across crests, and it's
origin-independent and addressed-hash friendly. Their honest limit: the
fold displacement is one-sided against the erosion surface (syncline cores
not conserved) — "parallel-fold conservation needs a real 3D structural
model," which our generator can host and their sampler couldn't.

## Stratigraphy (their journal 0045, "stratigraphy-v1")

Per-column SoA record: count/tags/thick arrays, K=8 slots, oldest at
bottom; run-length merge of same-tag deposits; when full, **evict by
fusing the thinnest adjacent pair** ("history compresses where it mattered
least"); finalize asserts sum(thick)==H per cell (a falsifier, not a
hope). Erosion pops recorded history off the top; floodplain densification
relabels the oldest material — the record IS the material state,
path-dependent for real. Tags are **named by what was measured at the
event** (their correction #11): SUBSEA, CARB, ALLUV_HUMID, ALLUV_ARID,
PARALIC — climate-at-deposition from snapshots re-marched every 20 steps
on *current* topography, never final-climate anachronism.

Transplants: run-length merge; thinnest-pair eviction as a bounded-K
policy (though our constraint is generation cost, not shipped-binary size
— we may want unbounded K or different compression); tag-by-measurement
discipline (load-bearing for our context tags); their hardcoded
tag→lithology switch is what our class-contract selection does cleanly.

## Paleo-context

Rock identity from formation-time conditions, two layers: (1) unit tags
carry formation environment permanently; the resolver adds burial — CARB
becomes marble+lapis only where pre-sim carbonate sits in an orogen core:
**uplift rate doesn't cook rock, exhumation exposes cooked rock**;
metamorphic grade = structural depth + eroded-overburden gain. (2)
Paleoclimate via a cheap deterministic **1D orographic march** (moisture
picked up over sea, rained out on windward slopes — the plane is
precipitation, not residual humidity; their first version got that wrong
and every crest read dry). Limits: aridity bucketed to 2 tags (their
resolver needed no more); 1D advection needs a box-blur against
row-streaking. Our P/T-path ambitions store richer per-unit context.

## Erosion / sediment routing — the bounded/unbounded theorem

Stream power A^m·S^n; suspended flux explicitly conserved down the D8
receiver chain; alluvial cover shields bedrock (1−e^(−H/H*)); never incise
below the receiver. **The deepest reusable insight**: erosion is bounded
(finite influence radius, regionizable with a halo) but **drainage is
genuinely global** (no halo bounds a basin) — so unbounded quantities live
coarse-and-global, computed once; bounded quantities regionalize. Their
measured surprise: relaxation processes need a *decay-length* halo (~31
cells), not the *propagation-speed* halo theory demanded (60) — the
correct architecture was also ~4× cheaper. Maps directly onto our
LOOKAHEAD_BOUNDS and the rivers/2-ring Observed item. Their unresolved
caveat: one drainage-deciding layer means block-scale channels can never
refine beyond it — a coarse-bin lock-in we don't inherit.

Also: floodplain densification **converts substrate to deposit without
raising terrain** (entrenchment-tapered); three rejected framings
documented before the shipped one.

## Filling the 3D world

`makeMat(bx,bz)` builds a per-column context once (everything
column-invariant hoisted); probing y is compares + one hash — zero noise
calls, zero allocation. Structural depth sd = (baseY − y) + fold-field;
the SAME displacement drives units and their internal bedding, so outcrop
bands wander with the fold coherently. Fill order top-down: recorded stack
→ arc volcanic pile → pre-sim marine cover → pluton (porphyry-copper roof)
→ rift basalt (amethyst flow-tops) → country rock graded by
burial+exhumation. Dikes/plutons cross-cut because they're depth-indexed
in R, not conformable units. Strata terrace operator: K=4 layer table,
per-repeat jitter **renormalized to sum exactly the period** (naive jitter
= phantom faults at every seam, measured); mean-centered so benches/cliff
bands emerge. Caves read solubility **from the recorded volume** (conduits
live inside coalesced carbonate bodies on a subdued, perched water table)
— solubility is a volume property, not a surface map. Determinism hygiene:
"judge the whole volume at the same scrambled position."

Coarse-bin artifacts we refuse: nearest-cell stack lookup (fault-block
mosaic), 48-block-cell hardness coupling, collapsed surface geometry
(only composition is historical).

## Method lessons (their journal + 12-entry corrections.md)

- Adversarial review before ship caught 11 defects in stratigraphy-v1
  alone (world-spanning ore horizons, origin-dependent fold phase,
  marble-without-burial, physics-gated-on-observer).
- **Measure the right metric**: lithology-selective erosion looked inert
  under bucket means; per-layer removal showed a 2.6× carbonate signal and
  a dendritic under-incision web. "The wrong metric nearly shipped a wrong
  conclusion."
- **Name buckets by measurement, not interpretation** (their corrections
  #9/#11/#12 are all this one bug class) — the discipline our context-tag
  design lives or dies on.
- Failed framings recorded as first-class artifacts (strata-v0's and
  sediment-v2's rejected designs documented in the shipped files).

## What we refuse (owning generation)

No stateless-sampler constraint, no datapack degradation, no
two-implementation parity tax, no shipped-bin size budget → no forced K=8
lossy eviction, no single drainage-deciding layer, real 3D fold
conservation available, sub-cell lithology-selective incision available,
surface geometry can be historical too.
