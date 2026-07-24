# 0091 — The ladder that ends the standoff

The 2026-07-24 far-field diagnosis (`docs/audits/2026-07-24-lod-pre-post-visit-diagnosis.md`)
named a "haunted" horizon: the finest visible LOD band showed a first-generation
member/class dither on the side nearest the player, while every coarser band
beyond it rendered faithfully. Not a subsystem split, not an eviction race — one
distance gate. `REDUCTION_STANDOFF_M = 176 m`: per far column, inside 176 m the
reduced (warm) data was *not even consulted*, only the cold floor-quantized
synthesis drew; beyond it warm-reduce composed in wherever a node's subtree was
resident. And 176 m lands **inside** the L1 ring (112–256 m), so L1's inner
shell was cold while L2/L3/L4 (all ≥ 256 m) were warm. Finest band cold, coarser
bands faithful — the reported inversion, static and deterministic given the
constants.

This entry is the rendering-only fix (fix "a"): kill the inversion without
touching world data. The material *disagreement* between cold dither and warm
dominant-subsurface — the S-9 consistency violation — is a separate slice (fix
"b", coupled to the mixture arc); after this fix cold and warm still differ in
*material* at the resident/non-resident frontier, and that is correct.

## Why the standoff existed, and why it could finally go

The standoff was not arbitrary. The block pyramid reduces with `MajorityNonAir`,
which **rounds the surface cell to nearest**: a coarse voxel that is majority-
solid becomes fully solid, so a reduced coarse top can sit up to one coarse voxel
*above* the true surface. Cold synthesis FLOORS (`quantize_top`) and cannot. So
inside the near field's draw radius — where the opaque near ground is right there
— a rounded-up warm corner could poke through it. The blunt cure was to refuse
warm data entirely within a standoff wide enough to cover the near draw radius.

The insight of this fix: the poke-through is a *geometry* defect with a *geometry*
cure. Floor the warm surface the same way cold floors, and warm physically cannot
rise above the near ground — at which point the distance gate has nothing left to
protect. So the change is four coupled moves:

1. **Warm geometry-safe.** `floor_known_surface` clamps any resident reduced
   surface run that straddles or grounds at the synthesized floor
   (`bottom <= synth_top < top`) down to `synth_top`. It fuses with the
   synthesized ground below in `compose_column`, so cold and warm now floor to
   the **identical** surface height; only the block/material each carries can
   still differ (that residual is fix b). A real overhang — a span wholly above
   the floor with an air gap beneath — is left untouched, because the near field
   renders that same real geometry; it is not phantom poke-through.

2. **Standoff removed.** `tile_column_stacks` no longer computes a
   `beyond_standoff` distance test. It composes `compose_column(synth,
   floor_known_surface(top, known(wx,wz)))` for every column: **warm wherever the
   node subtree is resident, cold only where it is not.** The A-5 guard is
   untouched — `known_node_grids` still serves reduced spans only for
   fully-inserted subtrees, so a partial subtree still falls back to synthesis,
   never to air.

3. **One LOD ladder.** The distances used to be scattered and uncoupled — the
   audit's § 6.3 defect: `REDUCTION_STANDOFF_M` set independently of the L1 ring
   edge, `UNLOAD_RADIUS_M = LOAD + 32` set independently of the near cover, the
   ring edges as `2×/4×/8×` multipliers in yet another spot. Now a single
   `LodLadder` holds the **nearfield border** plus **each far step's range as a
   distance past that border**, and everything derives from it:
   - ring edges = `border + step_past_border` (LOD-1 at the border);
   - near-coverage radius = the border;
   - near load radius = `border + overlap_past_border`;
   - near unload radius = `load + unload_slack` — the same number `streaming`
     streams by and `tile_in_cull_band` measures its refresh reach against;
   - `HorizonConfig`, `streaming::LOAD_RADIUS_M`/`UNLOAD_RADIUS_M`, and the
     back-compat consts (`FULL_DETAIL_RADIUS_M`, `FAR_OVERLAP_M`) all read the one
     `LodLadder::DEFAULT`. `every_lod_range_derives_from_the_one_ladder` pins it;
     a grep confirms no LOD range is defined in two places.

4. **Ranges as knobs.** Every `LodLadder` field is a named range knob, shaped so
   the whole struct can later become a per-player perf setting (a weaker machine
   pulls the coarse rings inward by shrinking `step_past_border_m`; a wider view
   dial pushes them out). No settings UI is built here — only coupled named knobs
   under one authority. The shipped defaults reproduce today's ring geometry
   exactly (border 112, load 128, unload 160, edges [112,256,512,1024,1200]), so
   this is a refactor of *where the numbers live*, plus the deliberate change to
   *which columns are warm*.

## What the tests became

- `reduction_standoff_keeps_near_columns_synthesized` (which asserted "no
  reduction consulted inside the standoff") is **retired**, replaced by
  `reduction_used_where_resident_and_floored_under_the_near_surface`: a viewer
  standing on the surface — deep inside the old standoff — now DOES use the
  reduction, but a reduced top of 22 over a synth floor of 20 renders at 20 (the
  floor), carrying the reduced material; where `known` is empty the column is
  pure cold synth.
- `warm_reduce_is_floored_geometry_safe` proves the floor at unit level: a
  rounded surface run clamps to the floor, no composed warm span exceeds it, and
  an overhang is preserved.
- `default_ring_edges_match_the_shipped_constants` keeps pinning the byte-exact
  defaults; `every_lod_range_derives_from_the_one_ladder` is the new coupling
  proof.
- The dc-worldgen **world-hash goldens are unmoved** — this touches only the
  dc-client render path; no generator behavior changed.

## What stays for fix (b)

Cold-synth (surface member/class dither) and warm-reduce (`classify` of the
dominant *subsurface* material) still pick different *materials* for the same
coarse box — the S-9 consistency-law violation. After this fix they floor to the
same height, so the seam is no longer a geometry discontinuity; it is a material
discontinuity at the resident/non-resident frontier, expected and correct until
the mixture-arc reconciliation makes the two producers agree on identity.

> blogworthy (lens 3 — reflexions in a deepsim codebase): the standoff was a
> distance gate compensating for a rounding choice two eras away. The cure was
> not a wider gate or a cleverer gate but deleting the gate by fixing the
> rounding at its source, then collecting the four distances that had drifted
> apart into one ladder they all derive from. A gate that guards an invariant
> another layer already violates is a smell; make the invariant true and the
> gate evaporates.
