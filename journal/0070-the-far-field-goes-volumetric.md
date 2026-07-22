# 0070 — the far field goes volumetric

*2026-07-22 · FF2b-minimal (background agent; worktree branch for the main
session to integrate). Rides the octree node contract v0.1
(docs/design/octree-substrate.md, D2 + the synthesized-node rider). Seed 1337,
production Medium world; numbers from the Windows dev box, release profile.*

## The slice, and why it is mostly invisible

FF2a (journal/0023) left the far field a stepped **top sheet**: one
`ColumnSpan` per column, a `top` and a `block`, sampled from the worldgen
coarse authority. The ratified node contract says a far node's render payload
is the real thing — the S3 block pyramid and the material reduction — derived
**two-sidedly**: reduce upward where full-res children exist, synthesize
top-down where they never will. This slice makes both sides real.

Here is the honest part, stated up front because the appearance section
depends on it: **today's worldgen terrain is column-shaped**. No caves, no
overhangs (visuals.md said exactly this when it scoped FF2a as "full visual
fidelity today"). So the volumetric machinery this slice builds has almost
nothing volumetric to *show* yet over generated terrain, and over
never-generated terrain the synthesized answer is deliberately byte-identical
to FF2a. The slice's value is the contract made real and held to agreement
tests — the geometry (a pit on the skyline, a chasm's deep dark) arrives when
the dirty-rail slice makes edits far-visible and when worldgen grows
overhangs. Building the register before the world can speak it is the point:
FF2a did the same for span stacks, and this slice cashed that cheque without
rewriting the mesher's reasoning.

## What landed

**The vocabulary moved down a crate.** `ColumnSpan` now lives in
`dc_core::farfield` and gains a `bottom` (with `FAR_BOTTOM_UNBOUNDED = i32::MIN`
as "the ground keeps going" — a plain i32, not an `Option`, so the payload
stays positional-format-safe, corrections #3). `quantize_top` and
`level_stride` moved with it, so the FF2a streamer and the new node synthesis
share **one** quantization (S-7: no second rule, ever). A far column is now a
stack of spans, topmost first, always ending unbounded.

**Reduce, where the world was generated.** `dc-client/src/farpyramid.rs`
feeds every chunk the near streamer generates into the S3 `LodPyramid`
(blocks) and a render-local material store that derives coarse levels through
`derive_material_lod_chunk` / `DominantClassDebrisAware` — the machinery that
sat in spines § 3 row 1 as "built, nothing renders it" since S8. It renders
now: `node_column_spans` extracts a node's per-column span stacks and refines
each span's block through `classify()` of the reduced mixture, so the far
mesh's material variety flows through the material pyramid. Row 1 is removed
from § 3 in this commit — the first row to leave the index, which is the
index's success condition.

**Synthesize, where it never was.** `dc-worldgen/src/far.rs` fills a node
top-down from `coarse_surface` — the same elevation lattice + river carving +
surface rule the near ground collapses from, the same source FF2a sampled per
column, recast in node form: each column solid below its floor-quantized
surface, carrying the surface block. Pure function of (pregen seed, node
coords); no wall clock, no ambient entropy. What the coarse authority cannot
say, the synthesis does not invent: no overhangs, no sub-surface strata
(stubs.md § 15 names the uniform-block stand-in and its heir — the
`surface_sample` summarization half, whose legal home the contract already
declared to be this seam).

**Composition, with A-5 teeth.** `compose_column` lays reduced node answers
over the synthesized backdrop. The guard is
`LodPyramid::subtree_fully_inserted`: a node is only *known* if every full-res
chunk of its subtree was inserted — a derived chunk over a partial subtree
reads its missing children as air, and that silence must never reach the far
mesh as "nothing there" (corrections #31's lesson, now enforced at the seam
where it would leak). Partial subtrees synthesize. Known air, though, is real:
a fully-generated node that reduced to air truncates the synthesized claim.

**The mesher speaks stacks.** Greedy-merged top faces for each column's
surface span (keyed on the face itself, so the single-span synthesized field
merges exactly as FF2a did), per-cell top faces for deeper spans, **bottom
faces** where a span's underside is exposed — the face class a top sheet can
never emit — and side walls by interval subtraction against the neighbour's
stack, which degenerates to FF2a's "taller column walls down to the shorter,
emitted once" for single spans. Same-level watertightness survives by the same
argument as 0023: exposure at a shared plane is disjoint between the two
sides, so every wall is emitted exactly once.

## The agreement test, and the number it found

The contract's acceptance test: synthesize a node, then actually generate its
children (8 chunks per L1 node, 64 per L2), reduce them up the real pyramid,
and compare per-column top face planes. Eight surface nodes across the Medium
world, L1 and L2:

```
n=8192 columns   mean(reduced − synth) = +0.938 coarse voxels
|d| <= 1: 100.0 %      max |d| = 1
```

The tolerance is stated as: mean in [0, 1] coarse voxels, ≥ 90 % within 1,
≥ 99 % within 2. Justification: synthesis **floors** the surface into the
coarse lattice (FF2a's load-bearing choice — the far top must stay under the
near ground), while `MajorityNonAir` votes the surface cell to nearest **with
ties to solid**. On flat ground the surface cell always holds at least half
its sub-voxels, so the vote rounds *up* — reduction sits exactly one coarse
voxel above synthesis almost everywhere, which is the measured +0.938 (the
deficit from 1.0 is sloped cells where the vote loses). The two sides bracket
the true surface from below and near-above; disagreement beyond one coarse
voxel would mean they are not sampling the same world, and there was none in
8 192 columns.

> blogworthy (procgen-vs-priors, deepsim reflexions): "the two derivations
> that bracket the truth." Floor-quantized synthesis and majority-vote
> reduction were built years apart in project-time, for different reasons —
> near-parity vs thin-wall survival — and when the node contract forced them
> to meet, their disagreement turned out to be a *constant* you can derive on
> paper: +1 coarse voxel, ties-to-solid vs floor. The agreement test didn't
> just pass; it measured the design.

The exact register rides beside the statistical one: a synthesized node's
tops equal `quantize_top(coarse_surface)` clamped to the node — same function,
no drift possible — and synthesis is byte-deterministic across generator
instances.

## The reduction standoff (a seam rule the near field demanded)

That +1 coarse voxel has a consequence: a reduced far top may legitimately sit
*above* the true surface, and the near mesh draws out to 160 m — a reduced
coarse corner inside that radius would poke through the walkable ground,
resurrecting journal/0022's sink problem from the other side. So reduction
only engages beyond `REDUCTION_STANDOFF_M` (176 m = the near unload radius +
slack); inside it the floor-quantized synthesis stands, whose ≤-the-surface
guarantee is what the overlap band was built on. The seam-tile refresh band
was widened to cover the standoff, which is also what walks reduced data in
behind a moving player: leave an area, and the tiles just outside the standoff
re-derive on your next near-chunk crossing and pick up the freshly reduced
nodes.

The pyramid itself is a fluid derived cache (S-2): budgeted
(`FAR_PYRAMID_L0_BUDGET` = 8192 L0 chunks, the S-1 knob; eviction policy is
the contract's open question 2), evicted farthest-first, and everything in it
re-derives byte-identically. Dropping it costs fidelity, never correctness.

## Tripwire numbers (octree doc § 6.5, measured on Medium)

Worst case (no coverage cull), synthesized field, stack mesher:

| horizon | tiles (L1..L4) | tris | mesh MiB | ms/tile | frame ms (2 tiles) | fill s @60fps |
|---|---|---|---|---|---|---|
| 1.2 km | 188 [56, 56, 56, 20] | 131 k | 21.5 | 3.45 | 6.9 | 1.6 |
| 3 km | 288 [56, 56, 56, 120] | 194 k | 31.9 | 3.12 | 6.2 | 2.4 |
| 5 km | 540 [56, 56, 56, 372] | 351 k | 57.5 | 3.60 | 7.2 | 4.5 |
| 10 km | 1 648 [56, 56, 56, 1 480] | 1.02 M | 167.7 | 4.94 | 9.9 | 13.7 |

The 10 km row stretches L4 (the `--horizon` knob's quadratic direction,
journal/0042) rather than adding rings as 0023's projection did — that is why
it reads heavier than 0023's "376 tiles / 43 MiB". Even so: memory grows
sub-linearly in area (69× the area for 7.8× the mesh — greedy merge eats flat
far ground), per-frame cost stays budget-bounded, and nothing approaches the
Aokana wire. Past ~10 km the standing answer remains "more LOD levels", not a
longer L4 and not a ray-marcher. Reduction-side residency: the near streamer's
sphere is ~500 L0 chunks; a long walk saturates at the 8192 budget
(≈ tens of MiB of palette-compressed chunks + material store).

## Appearance (the user's ratification section)

**Intended: near-zero delta.** The synthesized field — everything beyond
played regions, i.e. essentially every pixel of every horizon today — meshes
byte-identically to FF2a (single-span stacks, same greedy merge, same skirts,
same quantization). The deltas that do exist, each flagged:

1. **Reduced regions sit one coarse voxel higher.** Where the player has
   walked (beyond the standoff), far tiles now show the reduction's answer:
   ~1 coarse voxel (1.8–3.6 m at L1/L2) closer to the true surface than
   FF2a's floored sheet, with a matching one-voxel ledge at the
   reduction/synthesis boundary. At 200+ m this is sub-step noise, but it is
   real and it is new.
2. **Bottom faces exist now.** Nothing generates them over today's terrain
   (no overhangs in worldgen; trees mostly dissolve under the majority vote),
   but any reduced overhang — including, after the dirty-rail slice, player
   megastructures — will render an underside where FF2a showed sky-through.
3. `--fullbright` and `--edges` ride unchanged: the far mesh still goes
   through the same `MeshData` → `to_bevy_mesh` path, one shared terrain
   material, block-driven vertex colour.

Screenshots (lit for shape, fullbright for the one material question; vantage
family of 0023's asset set):

- `0070-lit-stepped-horizon-ground` — on the surface at spawn: near voxels
  roll into a stepped far horizon, continuous handoff, no seam, no holes.
  (The palette differs from 0023's grass world because journal/0055 reskinned
  the surface from the record — that delta is 0055's, not this slice's.)
- `0070-lit-high-vantage-rings` — default 1.2 km horizon from ~400 m up; the
  rings as a hazed stepped skyline, no sky holes.
- `0070-fb-high-vantage-patch-check` — fullbright (the instrument that can
  see a material question): the dark region in the lit shot is a sharply
  bounded grey-material square at ~deep-cell scale — a pre-existing
  authority-side frontier (over never-generated ground, where synthesis is
  FF2a-identical by construction), filed in ROADMAP Observed as the
  surface-material sibling of the S-4 `regolith_at_voxel` mosaic.
- `0070-lit-10km-horizon-vantage` — the projected ring config LIVE
  (`--horizon 10`, fog scaled out with it): 1 648 tiles filled in ~14 s,
  continuous field to a 10 km skyline, no holes, no cracks — and the
  deep-cell material checkerboard visible to the horizon (the Observed
  item's scale).
- `0070-lit-reduced-behind-standoff` — looking back at the streamed spawn
  region from beyond the reduction standoff: reduced tiles in place of
  synthesis, no poke-through, no frontier crack (the delta is the expected
  ≤1 coarse voxel and reads as nothing at range — which is the point).

On today's terrain these are intentionally indistinguishable-or-near from
0023's — that *is* the acceptance criterion for the synthesized path.

## Filed follow-ons (what this slice deliberately did not do)

- **Persistence + dirty-rail** (the decided next slice): far edits are still
  invisible; the pyramid is session-local and edit-blind (an edited chunk
  re-feeds only if it re-streams).
- **Synthesized sub-surface strata** (stubs.md § 15): synthesis paints the
  column with the surface block; the heir is the `surface_sample`
  summarization half landing at this seam as a per-node strata summary.
- **Overhang synthesis**: the coarse authority records none; do not invent.
- **Partial-coverage composition**: today a node is all-or-nothing
  (`subtree_fully_inserted`); per-column knowledge masks would let a
  half-generated node contribute its known half.
- **Deep-span greedy merge**: overhang faces are emitted per-cell; fine while
  sparse, worth merging when worlds grow real cave fields.
- **L3/L4 reduction reach**: full subtrees at L3 need 32 768 L0 chunks — the
  budget effectively caps reduction at L1/L2 today, which matches where the
  standoff lets it show anyway.

> blogworthy (AI-native development): "the brief that demanded the default
> case." v0 of the node contract described only bottom-up reduction; the
> gap-hunt (corrections #31) forced the rider that most of the far field is
> never-generated ground, and this slice's synthesized path — the *boring*
> path, the one with nothing new to show in a screenshot — is where all the
> contract's teeth (seeded purity, ungenerated ≠ empty, agreement) actually
> bit.
