# 0022 — the first horizon

*2026-07-19 · the far-field horizon (background agent; worktree branch for the
main session to integrate). Seed 1337, N=2 worldgen authority. Builds directly on
journal/0017 § the far mesh (the shape) and journal/0018 § the empty horizon (the
symptom). Numbers from the Windows dev box, release profile.*

## What a walker actually saw

journal/0018 named it more sharply than any metric could: standing on the
worldgen surface, *"the near terrain rolls out to the load radius and then the
horizon is **sky**. Nothing. The world ends like a floating island."* Under the
worldgen authority the far LOD rings were still the legacy S1 `TerrainGen` — a
surface at ~8 m while the real terrain sat ~1000 m up — so the far mesh drew a
phantom old world a kilometre *below* the player's feet, visible only as
haze-bleached fragments off a cliff edge, and contributed **nothing to the
skyline**. Two defects wearing one cause: a phantom below, and no horizon at all.

So this milestone was never "fix the artifact." It was the sentence the user
promoted it to: *build the horizon.* The world gets a far field, over the
worldgen authority, for the first time.

## The shape was already decided — the question was where the summary comes from

journal/0017 priced the two honest paths and ruled on them. Full-res-generate-
then-downsample is ~5 min per kilometre — a non-starter at boot. The right answer
is a **coarse summary**: worldgen emits a per-column surface summary at coarse
resolution, and the far mesh consumes summaries, never a generator. The open
engineering question this milestone had to answer: *which* worldgen data is the
summary, and how do we get it without paying for a chunk we throw away?

The tempting answer was to sample the deep-time `DeepField` surface directly — it
exists at exactly the coarse scale the far field wants (460 m cells), it is the
eroded macro-surface, and it is already resident (~25 MiB). But it has two holes.
It returns `None` in the border wilds (the pyramid runs forever out there; the
`DeepField` does not) — which would put the empty horizon *back* the moment a
walker looks past the civilized extent. And it is only the macro-surface: the
near ground is the deep surface **plus** the sub-locale midpoint jitter the
collapse pyramid adds below level 5. Sampling the `DeepField` alone would put a
smooth far surface next to a jittered near one, and the boundary would not agree.

### The lattice already is the summary

The insight that made this clean: the near ground's height at a world column is
`floor(carve_rivers(lattice(L_VOXEL, vx, vz)) / voxel_m)`, and that `lattice`
call **already embeds the `DeepField`** — at level 5 (the locale, ≈ the deep
tier's own 460 m cell) it replaces the analytic elevation with a bilinear sample
of the deep surface (journal/0015 § the elevation composition), then refines with
addressed jitter down to the voxel. So the elevation lattice *is* the deep-time
surface plus its own detail, sampled at whatever stride you ask for, and it runs
into the wilds because the pyramid does.

The far field therefore samples the **same function the near ground collapses
from**, at a coarse stride: a new `WorldGenerator::coarse_surface(vx, vz)` that
runs the per-column kernel — lattice, river carving, the surface-block rule — for
one column, O(pyramid depth) and memoized, no full chunk. To avoid a second
divergent copy of that kernel, I first factored the near collapse's per-column
body out of `column()` into a shared `surface_sample`; `column()` calls it 1024
times, `coarse_surface` calls it once. The near ground and the far horizon are now
provably the same surface — the seam invariant test over 10 000 chunks stayed
green through the refactor, byte-for-byte.

> blogworthy: "the horizon was hiding in the lattice." The instinct was to reach
> for the deep-time surface — the obvious coarse data. But the coarse data that
> *agrees with the ground* was the ground's own elevation function, which had
> quietly folded the deep surface into itself two milestones earlier. The far
> field didn't need a new source; it needed to sample the source the near field
> was already using, one stride coarser.

## Agreement by construction — and the number that proves it

Because `coarse_surface` and the near `column()` call the identical kernel, and
because **height is independent of climate** (climate only tints the surface
block; it never moves the ground), a far sample that lands on a near column
returns *exactly* that column's height. Not "within tolerance" — equal to the
integer voxel. The far-corner grid at level L lands on world voxels at stride
`2^L`; every one of them coincides with a near column, so every far-tile corner
sits on the near ground. The only residual is the sub-coarse relief **dropped
between** corners — bounded by the level's coarse voxel (≈ 1.8 m of vertical wander
at the innermost L1 ring, angularly nothing at 112 m).

Measured, at the L1 boundary ring, over a full tile's 33×33 corners: **max
height mismatch 0 voxels.** The near/far seam is not stitched; it is the same
theorem the collapse pyramid was continuous by — shared samples of one field
(journal/0015 § the seam that didn't happen, wearing a far-field hat).

The remaining seam risk is *cosmetic*: a far tile sunk half a coarse voxel below
the true surface (so the opaque near terrain wins the overlap band), a small
radial depth push so adjacent LOD rings separate, and — the honest workhorse —
the S4 post-stage haze, which fades the far rings toward the sky and dissolves the
ring/tile boundaries into atmosphere (earth-processes.md method rule 5: no hard
grid boundary reaches the eye). The high-altitude smoke shot shows the ring seams
as barely-legible diagonals under the haze.

## The mesh: a top sheet, not a shell

The old S3 far mesh was a 3-D spherical shell of coarse *chunks*, greedy-meshed —
and journal's own Observed cluster measured **~2/3 of its triangles were sealed
cave surfaces** nobody could ever see. A summary far field has no business
meshing caves. So the worldgen far field is a **2-D annulus of heightfield
tiles**: each tile is a 32×32-cell patch whose 33×33 corner heights come from
`coarse_surface`, emitted as a shared-vertex quad sheet — **2048 triangles, top
surface only.** Smooth normals from the height gradient; world-anchored UVs at
base-voxel density so the texture pitch matches the near ground rather than
stretching one tile per coarse cell; one block atlas layer per vertex, so the
same terrain material (post-PBR-1) lights the horizon that lights the ground, no
seam in material either.

Gating is on the active authority: `Authority::far_field_is_worldgen()` routes
key 2 to the heightfield (`stream_far_surface`) and keys 3/4 to the untouched S1
volumetric far mesh — the S1 far field was never wrong for the S1 world, so it
keeps it. The worldgen summary path **never touches `TerrainGen`**; a tripwire
test asserts the S1 authority returns `None` from the summary and the worldgen
authority answers ~1 km up (a summary that secretly read S1 would land near y=0).

## Perf: the horizon is not a world-create tax

World-create is unchanged — the far field streams *after* spawn, so the ~13 s
ritual pays nothing. And the derivation is cheap: **1.377 µs per coarse column**,
so a whole 1.2 km four-ring far field (~160 k columns) derives in **~0.22 s** if
you did it all at once. It is instead budgeted at 2 tiles/frame, and the full
horizon filled in within a few seconds of the smoke run with no visible hitch.
Each tile is 1089 vertices / 2048 triangles against the old shell's thousands,
two-thirds of which were caves — a strict win in both directions.

Nothing new is held resident: the summary is derived on demand from the pyramid
that already exists (persisting it to region files is the noted follow-on, coupled
to S3 grouping — out of scope here by design).

## What this resolves, and what it leaves

Resolved: the empty horizon (there is one now); the phantom old world ~1 km down
(gone — the worldgen authority no longer draws S1 at all); the far mesh's
sealed-cave triangle waste (top sheet, no interiors); the far field sampling a
*different world* than the authority (it is now the authority's own surface). The
S1 `TerrainGen` is one step closer to `pub(in crate::authority)` — the far mesh
was its last near-namer, and under key 2 it no longer names it (keys 3/4 still do,
so the seal waits on retiring S1 entirely).

Deferred, honestly: the heightfield is a **top surface**, so looking up from deep
in a chasm loses the far field (the volumetric shell did extend down a chasm) —
accepted for "build the horizon a walker sees from the ground," noted for the
volumetric follow-on. Far-field edits are still invisible (a summary, not a cache
— unchanged). Persisted summaries (region-file storage) remain the follow-on. Far
meshing is still main-thread/budgeted. The ring/tile normal seam at tile edges is
one-sided-differenced (no cross-tile halo) — invisible under haze, a cheap polish
if it ever isn't.

> blogworthy: the accidental proof that correctness and cost point the same way
> again (journal/0017's refrain): the far field that is the authority's *own*
> surface is also the cheapest to derive, because it is a coarse sample of a
> pyramid that was going to run anyway — not a second world to keep in sync.

## For the integration walk

The smoke run (worktree scratch, not committed) confirmed the pipeline: clean
launch, worldgen authority, shaders compiled, no panic, and a real horizon in
both a ground-level and a 1080 m vantage. Shots the milestone walk should take
(fullbright *and* lit):

1. **Ground-level horizon** from a walker's eye on open worldgen surface — the
   before/after against journal/0018's `0018-empty-horizon-player-view` (same
   framing if findable): terrain now meets sky continuously instead of ending in
   void.
2. **High vantage (~1 km up), pitch down** — the four LOD rings as a hazed skyline
   disc; look for the ring/tile seams (should read as terrain, not as a grid).
3. **Near/far boundary at a walk** — pan across the ~112–128 m band and confirm no
   vertical gap where the fine near terrain hands off to the coarse sheet.
4. **A cliff-edge / steep angle** where journal/0018 saw the S1 phantom — confirm
   the phantom old world is gone (nothing ~1 km down).
5. **Chasm-descent** — confirm (and photograph) the accepted degradation: the far
   field is a top sheet, so it thins looking up from deep underground.

## Walk 17 (main session): the horizon exists — with holes in it

*Appended post-integration (merge, gates green on merged main, 38 suites).*

The before/after is the milestone: `0022-first-horizon-player-view` at the
exact framing of `0018-empty-horizon-player-view` — where the world ended
in sky at the load radius, terrain now rolls to a hazed skyline. The
steep-angle check at the old phantom coordinates shows worldgen far
ground where the S1 old-world used to lurk: **the phantom is gone**. The
high vantage shows the skyline as intended — hills dissolving into
atmosphere.

And the walk earned its keep again: the far sheet has **persistent
rectangular sky holes** — parallelogram gaps at fixed world positions
(`0022-skyline-high-vantage`, `0022-phantom-check-steep`), unchanged
after 15 s of streaming budget, so they are missing/culled tiles, not
latency. Whole-tile granularity says winding (backface-culled tiles) or
annulus coverage gap. Also filed: thin sky slivers at the near/far
overlap at grazing angles (the sink doesn't occlude everywhere), and
faint tile-edge stitch lines (the known one-sided-normal seam). Fix
cycle dispatched.

## The holes were a partition with no redundancy (fix cycle)

The two live vantages reproduced instantly and stably: parallelogram sky
gaps at fixed world positions, unmoved by any amount of streaming budget.
Neither hypothesis on the docket was right. It wasn't winding — the tile
mesher winds every tile the same way, so backface culling is all-or-nothing,
never a scattering of specific tiles. And it wasn't a missing index — the
`r`-radius scan reaches every tile position in every ring.

The mechanism is subtler and it is the reason the holes sit at *fixed*
positions. **A single level's far tiles tile the ground plane as a
partition** — no overlap, no gaps — so a ground point falls in *exactly one*
tile per level, and the far field covers that point iff that one tile is
in-band. Ring membership is decided by the tile's *center* distance
(`RING_EDGES_M`), and there's the trap: at an inter-ring boundary R a point's
finer-level tile can have its center just *past* R (rejected by the finer
ring's outer edge) while its coarser-level tile has its center just *short* of
R (rejected by the coarser ring's inner edge). Both containing tiles rejected,
and because the partition offers no third tile to fall back on, the whole cell
is sky. Which cells hit the double-rejection is fixed by the grid phase
against the ring circle — hence holes nailed to the world, not the walker.
The same failure at the *innermost* edge (a level-1 tile whose center falls
just inside the 112 m LOD-1 edge, with no far ring beneath it) is exactly the
grazing-angle sliver at the near/far handoff — one bug, two symptoms.

> blogworthy: "the horizon holes were a partition with no quorum." The far
> field looked redundant — overlapping LOD rings, a documented overlap band —
> but a *partition* has no redundancy inside a level, and center-distance ring
> assignment quietly punched a cell out of both levels at once. The overlap
> that mattered wasn't within a ring; it was *between* rings.

The fix restores the missing redundancy: each ring's band extends its inner
edge inward by exactly one of its own tiles (`far_tile_in_ring`), lapping the
coarser ring one tile under the finer one. A boundary point's containing
coarser tile has its center within one coarse half-diagonal (< a full tile) of
R, so a one-tile inward lap guarantees that tile is in-band — the seam is
provably closed. The lapped coarse cells sit under the finer ring and the
existing half-voxel sink keeps them occluded, so the repair costs ~10 extra
tiles per ring and no new artifact. For level 1 the same lap slides the far
sheet a full tile *under* the near volumetric field, which is precisely the
recommended sliver cure — so the primary holes and the secondary slivers fall
to one three-line change. (The unload hysteresis had to learn the lapped inner
edge too, or coarse tiles whose own size exceeds the 48 m slack would
load-then-instantly-unload and thrash.)

Proof. The holes are a coverage theorem, so the test is one:
`far_tiles_cover_the_rings_without_seams` sweeps the inner rings across the
256 m and 512 m seams at 0.25° × 1 m and asserts every ground point in the far
field's responsibility band lands in some wanted tile. On the pre-fix code it
fails loudly — **88 463 uncovered points**, the first at radius 112 (the very
near/far sliver). Post-fix it passes. And live, at the two walk-17 vantages
re-shot at the identical poses: the parallelograms are simply gone, the far
sheet rolls unbroken to the hazed skyline, and three extra yaw sweeps found no
holes elsewhere.

Left filed: the faint tile-edge **stitch lines** (one-sided differenced
normals, no cross-tile halo — the journal/0022 loose end). Still cosmetic,
still invisible under real haze, and a cross-tile normal halo is a larger
change than this seam fix warranted; it stays a cheap future polish.

## Walk 18 addendum: the sky holes stay closed on main

Post-integration spot-check at the walk-17 high vantage
(`0022-skyline-holes-fixed`): continuous terrain to the ridgeline, no
parallelograms, 18 s streaming wait. One faint stitch line remains
(filed cosmetic). The partition-with-no-redundancy fix holds on merged
main.
