# 0086 — the substrate that carries the behaviors

*Design pass, 2026-07-23. The prompt was a keystone build; what it became was the
spec for the whole content layer. `docs/design/material-behavior.md` is the
record; this is how it went.*

> blogworthy (lenses: AI-native development; deepsim-reflexions): a design pass
> where the assistant's job was to be corrected — four over-reaches caught by the
> user in a row, each one collapsing complexity rather than adding it, until the
> model was simpler than either of us started with.

## Where it started

The session opened on priorities: which sequenced items had gone stale after the
block↔material decision (2026-07-22) and the north star (2026-07-23). The block
collapse's "first slice" had already been absorbed into **Crux 1**, the deep-cell
material inventory — named the keystone at the 2026-07-23 close because both the
storage atom *and* the material-behavior model converge there.

The recon (`docs/audits/2026-07-23-block-consumer-inventory.md`) said the present
voxel is already a full inventory (`VoxelContents`) but the **deep cell has none**
— it stores a strata *record* (history), so a behavior "consumes bedrock, produces
regolith" has nothing to read-modify-write. S16 had thin-adaptered one. The user's
first correction of the day: the deep cell's inventory is **a stack indexed by
depth**, not a single mixture — a column of spans. Which is when the fill contract
(DECIDED 2026-07-21: "a column's fill is an ordered list of spans carrying
contents, form, fractional occupancy") turned out to *be* the shape. We weren't
inventing the keystone; we were lifting a ratified contract one tier and making it
mutable-by-behaviors.

## The user's framing: a substrate that solves other designs' problems

The instruction that set the whole pass: *"we're creating a system that solves
problems for other designs by giving the behaviors they would like to write the
proper substrate for expression."* So the substrate is proven complete only if the
hydro / cave / weathering / diagenesis behaviors the corpus already wants can be
written over it without a carve-out — "can it carry the defaults," applied to the
substrate. I read the hydrology-priors sweep and the water notebook for exactly
that: what does a pass need *exposed* to model flow through pores, weathering
subtracting partials, deposition down-flow, dissolution.

## Four over-reaches, each one a simplification

The design converged by the user removing complexity I kept adding:

1. **Forms are not content.** I proposed making forms SDK-registrable. The user:
   forms are *the machine* — a closed set (structure, loose, pore-fill, fluid,
   void) every part of the engine must reason about; "add a form the rest of the
   system doesn't know exists" is not a coherent content operation. Mods compose
   with base forms; they don't author new ones.
2. **"Packed" is not a new form.** I introduced a sixth mode for crumbled-in-place
   rock. The user: that's just **pore-fill** — structure converting in place to
   fines in its own opening pores (`structure_len↓`, `pore_len↑`, same material,
   conserved). No new role. Expressible in the current encoding today.
3. **Mods don't add edges.** I put "add a transition" on the mod list. The user:
   **every form→form transition exists programmatically, always, ungated** — the
   graph is complete by construction (KISS). "Future additions" means the *machine*
   grows a form and its edges fall out automatically, not mods patching moves. The
   payoff: the complete graph's edges *are* the process catalog — melting is
   `loose→fluid`, evaporation is `fluid→void`, cementation is `fluid→pore-fill`,
   deposition is `void→loose`.
4. **`column_summary`'s resolved flag is not "the renderer doing this today."**
   Fact-checking a claim I'd welded together: `fully_resolved` is real and tested,
   but in `dc-core`, dormant, with no renderer or lighting consumer. The far-field
   LOD's pre/post-visit discrepancy is a *different* system — and the user's own
   sharpening (cold-synthesize samples the surface block; warm-reduce folds in
   subsurface via `classify`) makes it a material-identity disagreement, a live
   violation of the consistency law, not an instance of it. A background
   spine-audit confirmed all of it and found the existing agreement test covers
   only tops/occupancy, so nothing catches the disagreement.

What survived each cut was cleaner: five forms, a complete machine graph, and a
small content vocabulary sitting on top.

## The model that emerged

- **Forms** (machine, closed): occupancy modes, with fractional occupancy as an
  orthogonal axis. Three zones per voxel (structure / pores / unreserved) let loose
  and structure coexist without contending — a slab with loose come-to-rest is a
  valid voxel, so fractional structure entering the world doesn't create a weird
  place where loose can't fall. Weathering *shrinks the reserved shape* and emits
  loose; inherent porosity leaves it high — same model, two behaviors, and
  soft-boundary caves fall out.
- **Edges** (machine, complete): all form→form moves, the process catalog.
- **Agents** (content): named rate-terms bound to an edge —
  `driver × per-material susceptibility`, summed on the edge (the S16 fold). The
  rate reads *live cell state and the neighbor halo*, not just the sheet — the
  self-excavation feedback (trapped water crumbles walls until a path opens) proved
  it, and it needs no special case: it's the weighted-move model with every edge
  nonzero.
- **Passes** (content), two shapes: **cellular** run edges (change material, local,
  agents fold in); **field** compute a field (uplift, flow, precip) and plant it as
  environment (global/numerical, no agents). The reciprocal loop — field computes
  environment, cellular consumes it, field recomputes — is S-4, carried by the
  chapter loop. Processes that are both (transport, tectonics) split into a field
  half and a cellular half, which is how "transport is the sole advect" and
  "cellular passes are local" hold at once.

## The road not taken: the monolith

The parked question was "agents as passes vs all-in-one." The clarifying move was
that it hides two questions: co-determinant agents folding into one rate (S16,
proven, correct) versus distinct transitions being separate passes or one monolith.
S16 proved the *fold*, not the monolith — the "eager fold-in" from that spike
conflated them. Separate declared passes win on the tiebreaker that matters for a
plugin-first engine: **a mod adds a process by declaring it, not by patching a fold
it cannot see.** A monolith has no declaration surface. The pass-runner rejecting
cycles is the feature that forces cross-pass feedback to play out over chapters (or
fuse a genuinely tight pair — a measured exception, never the default).

## What it leaves

Void resolved as the unoccupied complement, not a stored role (water fills it
either way). Coupling-timescale-vs-cadence is the load-bearing assumption behind
loop-carried feedback, and it is measured, not decided. Two agents dispatched off
this spec: the present-tier `Block={Air,Material}` collapse, and a spike prototyping
the deep cell's mutable working span-list with chapter-commit — the keystone,
de-risked before it is built.
