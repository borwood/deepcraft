# 0024 — the anti-z-fight fix was the crack

*2026-07-20 · far-seam fix cycle (background agent; worktree branch for the
main session to integrate). Seed 1337, N=2 worldgen authority. Screenshots from
the Windows dev box (RTX 3070, Vulkan), release profile. References
corrections #11.*

> blogworthy: "the anti-z-fight fix was the crack; hollow geometry unmasked it."
> A fix that was provably correct in mesh space silently reopened the very seam
> it was covering for — one stage downstream, at the transform. The lesson isn't
> "watch your floats"; float precision was innocent by three orders of magnitude.
> It's that *watertightness is a property of the world-space surface, not the
> mesh* — and a per-entity transform computed from per-entity state is part of
> the seam contract.

## The report that three walks had walked past

The user kept seeing them: thin bright seams between the far patches, threads of
background light where the horizon should be solid. Repro was specific — up a
few hundred meters, pitch about −45°, look out across the far field. And a
discriminator that turned out to be the whole key: **v0 gen's original far mesh
never showed this.** Something the far field became had opened them.

This report had survived a remarkable amount of not-being-believed. Walk 17
filed "pixel gaps between far tiles" and we pinned them on T-junctions — the
classic smooth-heightfield crack where a finer ring's edge carries vertices the
coarser ring lacks. FF2a (journal/0023) voxelized the far field into stepped
prisms precisely to kill that crack class *inherently*, and it did: the
grazing-angle fullbright shot showed no cracks, and a same-level watertightness
argument (a step's side face emitted once by the taller column; neighbour tiles
sample the shared boundary column identically) proved the mesh seamless. We
wrote "crack class cured inherently" into the ROADMAP and moved on. The user
looked at the *same build* from altitude and still saw light through the gaps.
The integrator, looking at `0023-lit-high-vantage-rings.png`, called those
threads the cosmetic one-sided-normal stitch *shading*. The user's eye was the
instrument that was working: light coming *through* is not shading, it is a hole.

So the FF2a watertightness proof was true and the seams were real at the same
time. That is only possible if something reopens the seam *after* the mesher.

## The transform stage owned the crack

The mesher is not the last thing that touches a far tile's geometry. Every frame
each tile is re-placed from f64 relative to the floating origin, and folded into
that placement is the anti-z-fight **depth push**: corrections #1 established
that coplanar faces from overlapping LOD levels z-fight, that draw order / depth
bias cannot fix it (they change sort order, never the depth a fragment writes),
and that only a real world-space offset separates them. So each far tile was
pushed `DEPTH_PUSH_FRAC` (0.15) of its coarse voxel *away from the viewer* —
along its **own** center-to-viewer direction.

That "own direction" is the bug. Two adjacent same-level tiles have centers a
tile apart, so as seen from the viewer their center→viewer directions differ by
their angular separation, roughly `tile_m / distance`. Each tile is a rigid
translation, but along *slightly different* headings. Their shared edge —
watertight in mesh space, both tiles agree on it to the millimeter — is pulled
apart by `push × (tile_m / dist)`: about 0.08 m at L1, up to ~0.9 m at L4. A
sub-pixel-to-pixel slot, at range, backed by bright haze.

Why v0 never showed it, exactly as the user said: the S1 far mesh was
*volumetric shells*. A sub-meter lateral gap between two closed volumes exposes
the neighbour's own side geometry — you see more terrain, never the sky. The
moment the far field became a hollow top-surface **sheet** (journal/0022, then
FF2a), the identical offsets became see-through slots. FF2a cured the T-junction
in mesh space and *kept the push*, so the slots rode straight through the
"watertight" mesh. Grazing angles had hidden it twice: foreshortening shrinks
the slot and terrain backs it; only the top-down-from-altitude view opens the
cross-section against the sky. Float precision never entered — f32 at 1.2 km is
~0.1 mm, three orders of magnitude under the 0.08–0.9 m differential.

## The fix: one push per level, not one per tile

The push has to stay — coplanar z-fight is real and only a world-space offset
answers it (corrections #1). What has to change is that the push must not be a
function of the *tile's own position*, because that is what let neighbours move
differently.

So: compute **one shared push vector per LOD level per frame**. Direction is the
camera-forward axis (`Player::view_dir()`, shared by every tile of every level
this frame); magnitude is unchanged, `DEPTH_PUSH_FRAC × that level's coarse
voxel`. Both far paths now route through a single helper whose signature is the
whole argument:

```rust
fn level_depth_push(base: VoxelScale, level: u8, forward: DVec3) -> DVec3 {
    forward * (DEPTH_PUSH_FRAC * coarse_scale(base, level).voxel_size_m())
}
```

It takes no tile coordinate. Every tile of a level therefore translates by the
byte-identical vector, and a shared edge cannot separate — **same-level
watertightness by construction**. That is the fix, and it is a property of the
type signature, not of a tolerance.

It still separates every face orientation in real depth. Translating along the
view axis adds exactly `magnitude` of view-space depth (the depth of a point is
`dot(point − camera, forward)`, and `forward` is unit) — uniformly, to top faces
and side faces alike. Levels differ only in magnitude, so an overlapping ring
pair (coarser = larger push) still separates in the lap band, coarser sitting
behind. It remains a true world-space offset, never a bias. And because the
camera-forward axis is always unit, the old `normalize_or_zero` degeneracy (a
tile centered exactly on the viewer) is simply gone.

Both far paths got it — the FF2a `far_tile_translation` *and* the S1
`far_transform_translation`. The S1 volumetric mesh only *hid* the slots; it had
them too, and there was no reason to leave a latent defect in the keys-3/4 path.

The change is transform-only. No per-tile material, no extra component — the one
shared material / standard `Mesh` substrate is untouched, so FF2a's GPU-driven
multidraw batching (journal/0023 step 0) is unaffected.

One honest behavioral note: the push direction now tracks camera *orientation*,
not just position, so turning in place shifts a level's whole ring rigidly by up
to ~2 m (L4) along the view axis. Uniform, along depth, ~0.1° at 1 km — below
perception, and it is the price of the direction being shared. No swimming, no
relative motion within a level.

## Proving it past the transform

Corrections #11's lesson is that "watertight in mesh space" proves nothing if the
transform stage may move neighbours differently — the proof has to live in world
space, past the transform. So the pinning test reconstructs the world position of
a shared-edge point through *each* of two adjacent tiles' placements (`min + push
+ (P − min)`, the tile origin entering and cancelling in f64) for a deliberately
nasty viewer — 900 m up, off-grid yaw, ~−45° pitch — and asserts the two agree
exactly. They do: the gap is zero because the push carries no tile coordinate. As
rationale (not a kept-failing test) it recomputes the retired radial push for the
same two tiles and asserts it *would* have opened a >0.05 m slot — the mechanism,
pinned. The test is
`uniform_push_keeps_same_level_seams_watertight_in_world_space`. FF2a's coverage
theorem (`far_tiles_cover_the_rings_without_seams`) and the watertight/parity
tests stay green.

## The walk

High vantage (≈300 m up, feet at ~1006 m surface + 300, eye ~1306 m), pitch
−0.78 rad (~−45°), yaw swept N / NE / E / SE — the user's exact repro. `eye_in_solid`
was `false` at every pose. Lit (`0024-after-ne`, `0024-after-se`) and fullbright
(`0024-fb-ne`): the far field is a continuous stepped surface to the horizon,
**no thin bright slivers anywhere** in the sweep. Fullbright was decisive —
pure vertex color, no lighting to blame — and it also settled a red herring: a
large-scale green/brown diagonal across the far field *persists* under
fullbright, so it is surface-**block** data (grass- vs dirt-topped columns, a
real worldgen material boundary), not a seam and not lighting. The contact across
it is solid.

Z-fight regression check, the push's original job: a grazing pass
(`0024-fb-zfight-graze`, ~40 m up, ~−9°) across the near/far boundary and ring
laps showed no shimmer, no crack. A near-vertical top-down pass
(`0024-zfight-topdown`) shows a blue sky-diamond directly beneath the player —
that is the far field's accepted 2-D-annulus inner edge (112 m) seen from
straight above with the near field 300 m below and out of frame; it is
pre-existing, unrelated to the push, and not a tile seam. No z-fight anywhere.

## What this cost and what it bought

Two lines of arithmetic and a helper. The seams are gone by construction, the
z-fight the push exists to kill is untouched, and both far paths carry the same
guarantee. The expensive part was believing the report: a proof that is true in
its own frame can be the very thing breaking the frame downstream, and only the
user's altitude-and-look-down eye — plus a proof dragged out past the transform
into world space — could tell the difference.
