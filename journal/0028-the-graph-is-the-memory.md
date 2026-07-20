# 0028 — The graph is the memory

2026-07-20. S11, the water locality spike. Results in
`docs/spikes/S11-results.md`; code additive and standalone in
`crates/dc-worldgen/src/water/`.

## What we were sent to break

The water notebook had already settled the important thing — **one quantity,
two regimes** — and then immediately produced a wrong claim and a good
correction, both in the same conversation. The wrong claim was mine: *water is
a function until an edit makes the function wrong, so re-derive everything from
current geometry.* The user broke it with two cases that take about four
seconds to state and cannot be answered by any amount of local derivation:

- Dig a kilometre-deep chasm from a lake bottom. Stand at the bottom. How does
  anything know water reaches here?
- Dig a kilometre-long channel tangent from a river. How does the far end know
  a source is at the other end?

The diagnosis that came out of that is the reason S11 exists: **the two regimes
have different locality.** Bound water relaxes against its neighbours and is
derivable within a halo. Free water is *connectivity*, and connectivity is not
local at any radius — whether water reaches a place is graph reachability
through geometry the player invented this afternoon. No halo sees it.

So the hypothesis: persist bodies, derive voxels. The graph is the memory. And
S11's job was to find out whether that is affordable, or whether it is one of
those architectures that is obviously right and quietly costs a millisecond per
voxel.

Two questions. Is bound water's halo actually small? Does the body graph stay
sparse when someone digs a maze?

## Q1, and the measurement that was measuring nothing

I built the smallest honest saturation relaxation — infiltration, gravity
percolation, permeability-limited lateral redistribution, water table *read* as
the top of the saturated zone — perturbed it with a drain, and measured the
halo by Chebyshev ring, the way S9 measured erosion's decay length.

The first table came back beautiful and wrong. Every permeability contrast, at
every drainage spacing, reported the same halo: the radius of the domain. A
perfectly flat profile — `d0=5.16 d16=3.10 d32=3.05`. Read at face value it
falsifies the whole hypothesis: bound water propagates without bound, there is
no halo, go home.

It is worth dwelling on how *convincing* a wrong measurement can be when it
agrees with a hypothesis you were told to try to break. I had been dispatched
to falsify something, and here was a clean falsification, reproducible across
fifteen configurations.

The mechanism was the boundary condition. I had built a **closed box with
recharge and one drain**. Such a system has no local equilibrium — water
accumulates until the domain saturates, and then the single drain is the only
sink in the world, so *of course* it eventually affects every column. The flat
profile was not a physical result. It was the shape of a box with nowhere for
water to go.

A real water table is not bounded by the edge of the world. It is pinned by the
**drainage network** — streams and springs and coastlines every few hundred
metres. That is the boundary condition, and once it is there the field has an
actual equilibrium and the measurement has meaning.

Two more corrections before the numbers were trustworthy, and I record them
because each would have produced a confident wrong answer on its own:

**The relaxation was oscillating, not relaxing.** Explicit diffusion needs
`k·c ≤ φ/(2·ndim)`; I had used a coefficient thirty times too large, so every
cell dumped its whole content into a neighbour and got it back next step. The
oscillation floor was around 1.0 storage units against a signal of 3. The
"noise" in my first profiles was not noise, it was the integrator ringing.

**Control and treated were compared at different times.** The reference ran
`steady` steps; the perturbed run ran `steady + budget`. So part of every
measured "halo" was simply the difference between a field at 2 500 steps and
the same field at 2 564 steps. Both runs now advance together and are
differenced at matched step counts, which cancels common-mode transient and
leaves only what the perturbation did.

And one parameter that quietly balanced on a knife edge: the deep sink's
per-step acceptance was exactly equal to the recharge, so the column sat at a
critical point and the low-contrast cases came back with flat, non-decaying
profiles for a fourth distinct reason. Recharge 0.02, sink capacity 0.02. That
one took embarrassingly long to see because the number was right there in both
places.

With all four fixed, the profiles are textbook:

```
contrast 10000:  d0=19.671 d2=2.4896 d4=0.0935 d6=0.0014 d8=0.0000
```

Geometric decay, a factor of ~25 per two cells, gone by d8. Halo **4–11 cells**
at a bounded post-edit budget; the *player-visible* halo — where the integer
water table moves a whole voxel — is **0–6 cells**.

The result that surprised me: **a sharper aquitard gives a smaller halo, not a
bigger one.** The brief specifically flagged the sharp loose-vs-packed soil
contrast as the risk case. It is the most local case measured. A tight floor
perches the aquifer at full saturation, which caps the head range a drawdown
cone can express; a leaky floor lets the perturbation drain away locally. Both
mechanisms are short-range. There was no configuration in which they conspired.

Compare S9's fluvial row, where boundary error *reappeared* at d14 and d28 as
drainage reroutes teleported the signal along receiver chains. Nothing like that
happens here, and the reason is structural: bound water has no advective term.
It is a pure relaxation and it measures like one. S9 classified the water table
as "a bounded relaxation" from the armchair; this is that classification with a
number attached, and the number is smaller than erosion's.

## Q2, and the thing I got wrong about the shape of the graph

I expected to spend this half fighting node explosion. The notebook sketched
bodies with inlets and outlets and links, and the adversarial scenario list —
a maze, a hundred channels, a spiral shaft, a comb of trenches — reads like a
list of ways to make a graph blow up.

It does not blow up, and the reason is that I had the shape wrong.

In a voxel world, **two bodies in the same air component are the same body.**
If a channel is connected to a river, it is not linked to the river; it *is*
the river. So the link structure the notebook imagined mostly evaporates: links
only exist *between* components, which means a waterfall lip or a gate or a
pipe. Across all seven scenarios the link count was **zero or one**.

What that leaves is a two-part system with a clean seam:

- A **persisted** body graph — volume, level, character, an anchor voxel.
  Twenty bytes a body. This carries the one thing no derivation can
  reconstruct: that water *is* here, and how much.
- A **derived** connectivity index — per-chunk air component labels, plus a
  union-find over `(chunk, label)` nodes joined across chunk faces. Dense,
  changes on every edit, and never saved, because it is a pure function of
  geometry.

The km-long channel is 80 chunk relabels and 118 coarse unions. The far end
reads wet in **45.9 ns**, because "is this voxel in the river's component" is
one `find`. There is no kilometre-scale search in the code path, not because it
was optimised away but because the question was never asked at voxel scale.

The growth curve is the answer to "does it stay sparse", and it is blunter than
I expected: through 1 624 edits and 508 084 dug voxels, the body count stayed
at **1**. Digging does not create water. It creates space, and space is the
derived index's problem. Forcing the true worst case — a body seeded into every
component that has none — tops out at **217 bodies in 4 400 bytes**, because the
ceiling on bodies is the component count, and components track the coarse index
rather than the edit count.

## Two mechanisms worth remembering

**Splits are cheap because we stopped being incremental.** Union-find cannot
un-union. Digging that *disconnects* — plugging a channel at its midpoint — is
the adversarial case for every incremental connectivity scheme, and the
literature on it is unpleasant. The way out was to notice that the coarse graph
is *small*: thousands of nodes for a heavily-dug world. So it is rebuilt
wholesale on every edit, and a split costs exactly what a merge costs (0.3 ms,
2 chunk relabels). The expensive incremental part — per-chunk voxel labelling —
stays incremental, because that is where the millions of voxels live. Being
incremental at one level and wholesale at the other is the whole trick.

That first shipped at **19.560 ms per one-voxel edit**, which is a
ship-blocker, and the fix is the second half of the same idea: rescanning every
chunk *face* on every edit is O(labelled chunks × 1024). Caching the distinct
label pairs per chunk-face and refreshing only the pairs that touch a
relabelled chunk makes a rebuild O(#adjacencies). **0.249 ms — 78× — with
byte-identical answers**, asserted against a from-scratch build.

**The ocean question answered itself in the cost column.** The user raised
this at dispatch: a sea is not a finite volume to fill and drain, and breaching
it must not empty it. I was told to measure what breaks if everything is
finite-volume rather than assume the answer.

What breaks is exactly what you would fear. A 7.3 M-voxel sea, breached into a
void half its own volume, **drops 13.18 metres** — a shoreline retreating across
the map because somebody dug a big cellar. So the character distinction is real
and not optional.

But the number I did not expect is the other one. A level-pinned body's level
comes from outside, so **nothing ever needs its capacity curve** — and the
hypsometry scan can refuse to walk it entirely. **415 ms → 0.0 ms.** Since an
ocean is most of the open volume in a world, that refusal is most of the cost.
Level-pinning is not a semantic concession to make seas behave; it is the
cheaper implementation, and it is cheaper precisely in proportion to the thing
that is biggest. The right model was also the fast one, which is the third time
this project has had that experience (the orogeny halo, the FF2a voxelization,
now this).

> blogworthy: "the right model was also the fast one, again" — three
> independent instances now where the physically honest representation turned
> out to be the cheaper one, and the expensive version was the one that faked
> it. Orogeny's decay-length halo (4× cheaper than the propagation-speed halo
> theory demanded), FF2a's voxelized far field (cured the crack class
> *inherently* and greedy-merged to fewer triangles than the smooth sheet), and
> now level-pinned reservoirs (deletes the capacity scan for the largest body
> in the world). The pattern is worth a post: modelling the cause instead of
> the appearance keeps producing this, and the ratified cost philosophy
> ("coarsen the cause, never delete it and fake the appearance") predicts it.

## Determinism, and where it was actually broken

The design decision is that **events do not do anything**. Each event is a
commutative, monotone mutation of graph state; a single deterministic fixpoint
solve then computes every level. Order-independence is by construction rather
than by ordering a queue — the same move S9b made when it reformulated
diffusion from scatter to gather.

Which is a nice claim, and the shuffle test passed it immediately, and it was
false in two places. Both were found by sitting down and asking "what pair of
events would break this", not by the test:

**A `Breach` and an `OutletBlocked` naming the same link in one batch.** If the
block landed first the link did not exist yet, so it silently no-oped; the
breach then created it *open*. Reverse the order and it ends *blocked*. Fixed
by having both events get-or-create the link and then apply only monotone
mutations — `open` only ever falls, `sill` only ever falls — so every
interleaving converges to the same state.

**Several `RegimeCross` events on one body.** Float addition commutes but does
not associate. `1e16 + 1 − 1e16` is 0 or 1 depending on where you start. Fixed
by accumulating crossings per body and summing them in a canonical order rather
than in arrival order.

The lesson is the one S9b already taught from the other side, and it did not
transfer for free: *order-independence must be designed in, not discovered*, and
a shuffle test only exercises the adversarial cases you thought to put in the
batch. Both defects were invisible until the colliding pair was written down
deliberately. Both are now in the batch, and 256 shuffles are byte-identical.

## Where it landed

GO, with the ocean call attached. Bound water's halo is 4–11 cells and gets
*more* local under the contrast that was supposed to be the risk. The body
graph does not grow with edits — it grows with components, ~20 bytes each, and
every scenario resolved in under 0.11 ms. Unload, drop everything, reload from
39 bytes: the derived water comes back byte-identical.

The two cases that broke the derive-everything model are answered in
microseconds. Standing at the bottom of a kilometre-deep chasm, the water level
is a field on one node. The far end of a kilometre-long channel is wet because
it is the same component as the river, and one `find` says so.

The user's instinct — *bulk flows should be event driven rather than
continuously checking whether they are still being fed* — turns out to have
been the architecture, not a performance note. Nothing polls. A body in
equilibrium is static data. That is what the notebook said in the first verbatim
capture, before any of this was built.

Four ratification calls are in the results doc, and the load-bearing one is the
first: `Finite` vs `Pinned` is a claim about the world model — that some water
has a level set by the world rather than by its own volume — and which bodies
sit on which side is the user's call, not a performance question.
