# 0031 — Edges, and the fog that hid the mountain

*2026-07-20. Two instrument fixes filed off journal/0030 and corrections
#18/#19: give `--fullbright` a way to SEE shape, and stop distance fog from
washing the pure-data register to white. Both land as agent-walk diagnostics —
dev-only, never part of the shipped look. This entry is the reasoning and the
before/after photography.*

## The blindness, restated

0030 published a wrong conclusion twice in one day because it trusted
`--fullbright` to answer a *geometry* question. Fullbright is unlit pure vertex
colour by design (it is exactly what proved the coal was fine in 0027 when the
renderer had crushed it to black — a *data* question it answers perfectly). But
on terrain built from one material, every face of every block is the same flat
colour, so a fully terraced hillside renders as a **featureless grey field**.
`0031-flank-fullbright.png` is that field: a 400 m staircase of benches climbing
into the range, and you can see exactly none of them — just a grey mass under a
blue sky, its only legible feature the horizon silhouette.

The user's proposal (corrections #18) was to give fullbright dark face borders.
The ratified shape: outline **crease and silhouette edges** — not a per-cube
wireframe.

## Why creases, not a wireframe

A wireframe draws every voxel edge. At arm's length that is the grid; at a
kilometre it is a death sentence, because a voxel is sub-pixel and a regular
grid sampled below Nyquist aliases into moiré — a shimmering interference field
that moves when you do. The grid is also not the information. What makes a bench
legible is the **step**: the discontinuity where a top face meets a riser
(a normal break) and where nearer geometry occludes farther (a depth break).
Those are sparse — one line per terrace, not one per voxel — so they stay
legible at range and, faded out before they go sub-pixel, never moiré.

So the pass outlines exactly two things and nothing else:

- **Creases** (the benches). No normal buffer is bound in post hook format 0,
  so normals are reconstructed from depth: linear view depth → view-space
  position (using the camera's own projection scale, passed in the pass
  uniform) → a *forward-difference* and a *backward-difference* surface normal
  at each pixel. On any plane — flat OR sloped — both normals equal the plane
  normal, so `1 - dot(n_fwd, n_bwd)` is zero and no line is drawn. At a bench
  the forward difference spans one face and the backward difference spans the
  other; the normals disagree; that is the line. This is the whole reason the
  raw depth Laplacian is *not* used for creases: view depth is not linear in
  screen space on a slope, so its curvature would paint an edge on every
  hillside. Orientation is the invariant; position is not.
- **Silhouettes.** A geometry pixel bordering sky is outlined against the
  horizon (a hard occluding contour). Between two geometry depths, a relative
  depth Laplacian catches the occluding contour while rejecting the constant
  gradient of a slope.

The two terms are combined with `max` and the whole thing multiplied by a
distance fade.

## The fade

Edges are full strength out to **350 m**, ramp linearly to zero by **1400 m**,
and beyond that are not drawn at all. This is the moiré cure by construction: a
sub-pixel voxel bench at kilometre range gets *no* outline rather than a
shimmering one. The crease term already self-attenuates (a fixed-size step
subtends fewer pixels with distance), but relying on that alone still leaves a
noisy far tail; a hard fade window is honest and cheap. Reported constants (all
in `shaders/edges.wgsl`, all dev-only appearance choices): crease
`smoothstep(0.15, 0.60)` on `1-dot` (≈32° break floor; a voxel bench is 90°);
silhouette `smoothstep(0.05, 0.20)` on the relative depth Laplacian; fade
`350 → 1400 m`; outline colour pure black at strength `0.85`.

`0031-flank-fullbright-edges.png` is the same vantage as the grey field above.
Every bench is now a drawn step: the near blocks read as blocks, the mid-slope
terracing reads as terracing, the far crest softens as the fade takes it. That
is the featureless field made legible.

## Where the pass lives, and the invariant it must not break

Edges are a **renderer** diagnostic, not part of any content pack's look, so
they are a **separate post pass** (`edgepass.rs`) that runs *after* the pack's
`post` stage on the final composited image — not a term folded into the post
prelude. Folding them in would force the term, and a new uniform field, into the
frozen hook-format-0 contract that every pack compiles against, and would spend
ALU in every pack's fragment shader even with the flag off. A content pack
should never have to implement a dev diagnostic.

The load-bearing invariant (the 0027 coal control): **`--fullbright` without
`--edges` must be byte-identical to today.** It is, and by the strongest
possible construction — with `--edges` off, `EdgePassPlugin::build` returns
early, so there is no extract, no pipeline, no pass. Not a pass-through pass:
*no pass*. The camera carries an inert `EdgeParams` marker that nothing reads
until the plugin is enabled. Edges compose either direction: `--fullbright
--edges` (the expected use) and lit `--edges` both work, because the pass reads
only scene colour plus the depth the post stage already binds
(`0031-flank-lit-edges.png`: the dark crease lines sit cleanly on top of the
lit LabPBR shading).

## The fog that hid the mountain

The second half. 0030's 3.5 km massif vista washed to near-white in *both*
passes, and the reason was not lighting — it was distance fog.
`0030-massif-after-fullbright.png` is the evidence: the summit dome dissolves
into pale haze, its lower slopes gone entirely, the silhouette barely there.
Fullbright exists to be a pure-data register; atmospheric haze does not belong
in it.

The fix is data-side and touches no pack WGSL: when `--fullbright` is set, the
app pushes the `PostStage` fog range beyond the 3 km far plane
(`fog_start_m = 1e9`, `fog_end_m = 2e9`), so `dc_fog_factor` is 0 for all
geometry and the default pack's own shader hazes nothing. The lit pass is
untouched — fog lives only in the post stage (grepped: no farmesh/terrain
material applies its own), so neutralizing the uniform is the whole change, and
lit keeps its atmosphere. `0031-massif-fog-fullbright.png` is the after: the
same dome, now crisp grey against clean sky, the flat top and the single steep
right flank finally a readable silhouette. (The separate sky-haze pull, keyed
off `is_sky` rather than the fog range, is not distance fog and stays, so the
sky still meets the horizon.)

## The moiré check, honestly

`0031-massif-fullbright-edges.png` is the payoff shot: fog off AND edges on, the
dome's terraces drawn as nested contour lines revealing the whole landform, the
summit silhouette clean. The specific thing 0030 asked me to check: **is there
moiré at range?** No. The densest edge region — the mid-dome, where terraces
stack — reads as clean nested contours, not a beating interference pattern; the
far crest fades to soft/faint under the 1400 m window and then to nothing. The
lit counterpart (`0031-massif-lit-edges.png`) doubly confirms the fog fix is
fullbright-only: there the summit still dissolves into haze exactly as before,
with the edges composing through it — lit fog is untouched.

Every pose reply reported `eye_in_solid: false`; the sun is fixed (S4's constant
0.35), so these across-launch comparisons are valid (corrections #19).

> blogworthy: **the outline is the information the flat colour threw away.**
> Fullbright deletes lighting on purpose — that is what makes it a clean data
> probe — but lighting was also the only thing telling one face of a voxel from
> another. Rather than put shading back (and reintroduce the noise fullbright
> exists to remove), reconstruct just the *discontinuities* from depth: the
> creases and the silhouettes, faded out before they can alias. Same flag, two
> registers — flat colour for "what is this material", sparse black lines for
> "what is this shape" — and neither contaminates the other.

## Files

New: `crates/dc-client/src/edgepass.rs`, `crates/dc-client/src/shaders/edges.wgsl`.
Touched: `main.rs` (`--edges` parse), `app.rs` (`Edges` resource, plugin,
`EdgeParams` on camera, fullbright fog neutralize in `setup`), `poststage.rs`
(a `PostStageSet` ordering label so the edge pass can schedule after it).
Screenshots (`journal/assets/`): `0031-flank-fullbright.png` (blind control),
`0031-flank-fullbright-edges.png` (benches legible), `0031-massif-fog-fullbright.png`
(fog fix after; before = `0030-massif-after-fullbright.png`, same world, fog on),
`0031-massif-fullbright-edges.png` (moiré check), `0031-flank-lit-edges.png` and
`0031-massif-lit-edges.png` (composability + lit fog intact).
