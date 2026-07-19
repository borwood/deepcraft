# 0011 — the geology post-v1 slice: weather-blind granite, wandering contacts

*2026-07-19 · 3d (background agent; integrated + walk-verified by the main
session, merge `ba666e6`, 36 suites green on merged main).*

ROADMAP 3d bundled four small mechanics and a roster proof, all decided
2026-07-19. They share a spine: the v1 geology shipped a *shape* — classes as
contracts, a pass graph, a strata record — with the smallest honest content.
3d fills in the content the shape was built for, and in doing so surfaces two
things the shape had been quietly getting wrong.

## The granite was reading the weather

The v1 shim evaluated *every* class's fitness against the column's present-day
temp/precip, igneous included. Ratified as acceptable only for the surficial
veneer (year-zero climate is the truth for recent clastic sediment; the
fine/coarse split within a deposit is fluvial-energy sedimentology and
unimpeachable). But an intrusive granite forms kilometres down over millions of
years — the surface weather at year zero has nothing to say about it. The shim
handed it the weather anyway.

The correction is not to the machinery — the class-contract selection already
supports per-class fitness — but to *which context flows*. So the contract
itself now differs by class kind. Clastic and placer carry the `temp_c`/`precip`
windows; igneous and accessory carry **none**. `geology_param_specs(class)`
omits the weather params for the province/depth-driven classes, so an igneous
member that tried to supply `temp_c` is rejected as an unknown param at define
time — a granite is now *structurally* prevented from being handed the weather,
not merely discouraged. Internally the worldgen igneous windows leave the
weather axes unbounded (`FormationWindow::igneous`), so fitness is 1 regardless
of climate; a test swings temp/precip across their whole range at fixed depth
and asserts the selection never moves.

What stripping the weather surfaced: with weather gone, the *only* thing
differentiating two intrusives is emplacement depth (plus the pass's province
gate, which stays where the v1 design put it). That is honest — it is what
differentiates plutons in the real world — but it means the roster's second
intrusive (diorite) is a shallower-seated body, not a "warmer" one. Depth, not
weather, is the igneous knob. We chose to keep the province gate in the pass
rather than adding a tectonic-setting `Choice` param to the fitness model;
geology.md offered "depth_m and/or setting" and depth alone is the smaller
honest step. Flagged for ratification.

## The seam was the wrong cell

Walk 8 reported material families cutting hard on chunk lines, with a candidate
mechanism filed to Observed: at N=2 the chunk equals the S7 column-quantization
cell, so *cell-stepped climate context* flips selection on chunk borders.

Half of that is right and half is wrong, and the wrong half is instructive.

The pregen climate/provenance **cell** is 16 384 voxels — 512 chunks, ~14.7 km
— *not* the chunk. And `climate_at` was already bilinear between cell centres:
climate is a smooth field that does not step at chunk borders at all. So climate
is not the culprit. What *does* equal the chunk at N=2 is the **chunk-column
collapse unit** (`L_COLUMN`, 32 voxels), and the strata passes run once per
chunk-column. Every field they read — the single centre climate sample, the
single centre `flow_energy`, the footprint-mean elevation, the per-chunk
selection hash, the integer-rounded thicknesses — is piecewise-constant across
the whole 32×32 footprint and uncorrelated with its neighbours. The family is
uniform per chunk and flips on the grid because *selection is quantized to the
collapse unit*, not because any climate field steps. "The cell equals the
chunk" was true of the wrong cell (corrections #6).

The fix works at the material tier, where the artifact lives (walk 8 said
*material* families). Each event records the address of the selection field it
filled (`sel_salt`, `sel_tag`) and its formation context. At fill time the host
member is re-resolved **per voxel-column** from a bilinear field whose four
corners are the chunk-column selection hashes. Because neighbouring chunks share
a corner, the field is C0-continuous across borders: a class's member-partition
boundary becomes a smooth curve that wanders like a facies contact and can fall
*inside* a chunk — which a per-chunk selection, constant across all 32 columns,
can never do. The record still stores the chunk-centre member as its canonical
identity (so the abundance and byte-identity proofs are unchanged); the dither
only changes which member's albedo the mesher paints, and only within a class,
so the block tier (which maps by class) is untouched.

> blogworthy: the seam that was the wrong cell — a quantization artifact whose
> named mechanism ("cell-stepped climate") was falsified by measuring *which*
> cell it rode, and the C0-continuous corner-hash dither that turns a chunk grid
> into facies contacts without a drop of iteration-order entropy.

The transect test isolates the mechanism: a two-member, equal-fitness class over
a 12-chunk sweep, asserting both members interleave and at least one contact
falls interior to a chunk. Registration order cannot move a contact (canonical
order wins); same seed twice is identical (the byte-identity suite, which now
runs through the dither, still holds).

The thickness/class-presence quantization — a sandstone cap appearing or
vanishing as `flow_energy` crosses a rounding threshold at a chunk line — is a
*separate* per-chunk artifact. It needs per-voxel-column context, not member
dither, and is left as a loose end.

## Unfilled slots can't break the world, at two layers

geology.md's "a pass is a pack": a gen pass must never select from a class with
no members, or world content becomes a function of installed-pack coincidence.
Enforced twice, both naming the culprit:

- **Define-time** (`validate_pack`): a content-pack batch must ship a member for
  every class it declares in the same batch. We hold *every* declared class to
  the rule, a hair stronger than the "consumes a class it also introduces"
  wording — it is crate-boundary-clean, because dc-api cannot see the worldgen
  passes to know which classes a pass consumes.
- **World-build-time** (`Pipeline::check_class_satisfiability`, run by the new
  fallible `WorldGenerator::try_with_geology`): the build refuses if any class a
  pass's declared `selects` names is empty. Reads exactly like the existing
  cycle and ambiguous-writer rejections: `pass X selects from class Y, which has
  no members`. The convenience `with_geology` constructors panic on an
  unsatisfiable set; vanilla is always complete.

## Accessories as pore partials

The standard representation for an inclusion is a voxel whose structure slots
are the host rock and whose pore slots carry the accessory — the same eighths
machinery the placer already uses for gold-dust-in-sandstone, in igneous dress.
The igneous pass now selects a `dc:accessory/mafic` member (vanilla:
`dc:geo/olivine`) under the host's province/depth context and emplaces it
sparsely as a single pore eighth in the host rock, through the `VoxelContents`
canonical constructors. The 3c-2 face dither renders that one-eighth pore as
scattered cells with zero renderer-side code — olivine speckling a basalt flow,
by construction.

## The roster, widened as proof

Not the full rich roster — a proof of the posture. A second member joins each v1
class: siltstone (fine), conglomerate (coarse), diorite (intrusive), andesite
(extrusive), plus olivine (accessory). Ten vanilla members across six classes,
each with an honest property sheet and albedo. The abundance-normalization and
registration-order-independence proofs pass unchanged with the wider roster —
which is the whole point: diversifying a class redistributes its share, it never
inflates it, and canonical order makes registration sequence irrelevant to the
last byte.

## Walk 10: the fix that photographs, and the fix that can't yet

The accessory half photographs beautifully: a pit deepened to basement
shows **olivine pore-partials as sparse sage cells in the pink granite**
(`assets/0011-olivine-in-granite.png`) — the 3c-2 dither rendering the new
inclusions with zero renderer changes, sparse and undemonstrative, as the
subtle-ore doctrine wants.

The member-contact half produced the walk's real finding: a 160×160-voxel
overburden strip, meant to show wandering mudstone/siltstone contacts in
plan view, rendered as **one featureless field** — because **member
identity is render-invisible**. Uniform-contents voxels take the
single-color fast path and paint the *block* color, and siltstone and
mudstone are both the `Mudstone` block; the wandering contact exists in
the data (the transect test proves interleaving and an interior contact)
but cannot reach the eye until uniform contents render their *material*
albedo instead of their block color. Filed to Observed — it's a small,
honest extension of the interim renderer (make real data visible), not a
bandaid tune.

Which also sharpens the record on walk 8: what the user saw was most
likely the **class-presence quantization** (a sandstone cap appearing via
`flow_energy` rounding, per-chunk) — the loose end 3d explicitly left —
compounded by member flips that were, it turns out, never visible anyway.
The user-facing seam is therefore *not yet fixed in the visible world*:
one mechanism is smoothed but invisible, the other is diagnosed and
sequenced. The honest scoreboard matters more than a victory lap.

## Loose ends (→ ROADMAP Observed)

- Thickness/class-presence quantization (sandstone cap on/off via `flow_energy`
  rounding) still cuts on chunk lines — needs per-voxel-column context, not
  member dither.
- Igneous member differentiation is depth-only in v1; the tectonic-setting
  `Choice` fitness axis is deferred (province gate stays in the pass). The
  epoch-indexed paleo-context (3e) is where setting becomes real.
- The accessory presence gate and pore fraction (0.6 / 1 eighth) are picked, not
  tuned; the dither cell count (Observed) governs how the speckle reads.
