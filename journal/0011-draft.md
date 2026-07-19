# 0011 — the geology post-v1 slice: weather-blind granite, wandering contacts (DRAFT)

Draft. The walk section is a placeholder — the main session re-walks a former
hard family boundary and photographs the smoothed transition before this
entry is finalized.

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

## Walk (placeholder)

_To be filled by the main session: re-walk a former hard family boundary near a
placer fan / clastic contact under `--fullbright`, photograph the smoothed
material transition (before/after against a walk-8 chunk-line shot if one
exists), and confirm accessory speckle reads on an igneous exposure. Screenshots
to `journal/assets/0011-*`._

## Loose ends (→ ROADMAP Observed)

- Thickness/class-presence quantization (sandstone cap on/off via `flow_energy`
  rounding) still cuts on chunk lines — needs per-voxel-column context, not
  member dither.
- Igneous member differentiation is depth-only in v1; the tectonic-setting
  `Choice` fitness axis is deferred (province gate stays in the pass). The
  epoch-indexed paleo-context (3e) is where setting becomes real.
- The accessory presence gate and pore fraction (0.6 / 1 eighth) are picked, not
  tuned; the dither cell count (Observed) governs how the speckle reads.
