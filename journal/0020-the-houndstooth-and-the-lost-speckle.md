# 0020 — the houndstooth and the lost speckle

*2026-07-19 · PBR-1 render polish, two walk-14 ratifications (background agent;
gates green on the worktree branch, live-verified lit + fullbright on the RTX
3070, integration + milestone walk owed to the main session).*

Walk 14 photographed the first lit quarry cut (journal/0019 § walk 14) and came
back with two complaints the user ratified into follow-ups (visuals.md § PBR-1
walk-14 ratifications): the granite walls wore an obvious repeating **houndstooth**
where same-material blocks met, and `--fullbright` — the walk protocol's
lighting-free diagnostic register — had gone quiet about mixtures. This entry
fixes both.

## What the houndstooth actually was

The tempting story is a broken seam: the textures don't tile, so every block edge
is a visible discontinuity. That story is **wrong**, and it matters that it's
wrong, because the ROADMAP records "seamless tiling verified" (the generator's
`_seam_check` asserts it on every regenerate) and I did not want to falsify a
true claim.

Two checks killed the seam hypothesis. The generator's wrap-continuity self-check
was still green. And tiling a single 16×16 placeholder basecolor 4×4 offline
showed a field with **no** hard edges at the tile boundaries — the tiles wrap
cleanly. Whatever the walk saw, it was not a seam.

What it was: **legible per-voxel repetition**. At one 16×16 tile per 0.9 m voxel,
the *same tile image* repeats every block, and the placeholder tiles carried a
strong **low-frequency directional signature** — coarse materials drew their
value-noise at freq 2–4 (blobs 4–8 texels across), and a deep relief amplitude
(up to 2.6) plus a high-contrast AO ramp plus a wide quantized albedo spread
(0.17) turned those blobs into bold, shaded, diagonal contours. Repeat that
signature at the voxel pitch and the eye locks onto the period instantly. The
tiles were seamless; the *pattern* was loud. (Real Minecraft stone repeats every
block too and reads fine — because it's isotropic high-frequency grain, not a
low-frequency directional blob.)

So: not a seam. `journal/corrections.md` gets **nothing** — the seamless claim
holds. This is the precise distinction the mission asked me to draw.

> blogworthy: **seam vs. signature.** A repeating-texture artifact has two
> completely different causes that look similar at a glance — edges that don't
> match (a seam) versus edges that match fine but carry a pattern the eye can
> phase-lock (a signature). The fix for the first is UV/authoring continuity; the
> fix for the second is killing the low-frequency direction in the texture. We
> had the second and nearly filed a bug against the first.

## Fix, part 1a: world-anchored UVs

The mesher had been emitting a per-face `[0, 1]` UV window — every face reset the
tile to its own corner. I replaced it with a **world-anchored planar UV**: the
tile origin is the world voxel coordinate, the two in-plane world axes become U
and V, so a wall of same-material voxels tiles *continuously* (and a greedy quad,
if the mesher ever emits one, tiles K times over its K voxels). At a corner the
UV is integer-valued; the shader `fract`s it to sample.

Honesty about what this buys: for the *current seamless* tiles, world-anchored
UVs are close to visually neutral in the lit path — a seamless tile phase-aligned
per-face is already continuous across the boundary, so this change alone does not
erase the houndstooth. Its lit value is correctness and robustness (it's the
scheme authored non-seamless textures and greedy meshing will need). Its
*load-bearing* value is Part 2: the integer part of a world-anchored UV gives the
fullbright speckle a **world-stable cell grid** for free — `floor(uv * 4)` is a
4-cells-per-voxel lattice anchored in world space, exactly what a world-anchored
per-cell dither needs, with no extra vertex attribute and no dependence on the
floating origin (which would make a `world_pos`-derived grid swim on rebase).

## Fix, part 1b: de-directionalize the placeholder tiles

This is the mechanism that actually removes the visible artifact. In
`tools/gen_placeholder_textures.py`, four gentle changes, all keeping per-material
identity hues and all keeping the seam self-check green:

- **Raise the frequency floor** (2 → 4) so no material draws a giant low-freq
  blob, and rebuild the height field as a **three-octave fractal** weighted
  toward the fixed freq-8 and freq-16 octaves — the tile reads as isotropic grain
  rather than a directional blob.
- **Narrow the albedo shade spread** (0.17 → 0.11): gentler per-texel value steps,
  the pixel-game quantization intact but the contrast that made the repeat bold
  gone.
- **Soften the normal relief** (max 2.6 → 1.5) and the **AO contrast**
  (`0.55 + 0.45 h` → `0.72 + 0.28 h`), so baked shading doesn't re-draw the blob
  as a dark repeating grid under the lit path.

All 26 packs regenerated deterministically; the self-check still passes. Tiling
the new granite basecolor 5×5 offline reads as clean isotropic grain, and the
live lit spawn (grass + dirt surface) shows Minecraft-grade pixel noise with no
legible period.

## Fix, part 2: the fullbright splat variant

The old mosaic gave `--fullbright` per-subquad dither colors; PBR-1's single-quad
splat mesh carried one blended vertex color, so a mixed face read as a uniform
field (journal/0019 § walk 14, filed to Observed). The user made this REQUIRED:
screenshot auditability needs an AI viewer to *notice mixtures* without lighting
noise.

The constraint is that fullbright must stay the pure diagnostic register — flat
albedo, no lighting, no normal/spec/emission, and (the user's addition)
**albedo-only, never the LabPBR basecolor textures**, because a flat color is
easiest for shape detection. So the lit `TerrainMaterial` cannot serve this; it
needed its own material.

`FullbrightTerrainMaterial` is a second custom unlit `Material` sharing the same
chunk mesh (the streamer already picks lit-vs-fullbright per entity — now it picks
between two custom materials instead of custom-vs-`StandardMaterial`). Its shader:

- Counts the active splat slots. A **uniform or block face has one** → return the
  mesh's vertex color directly: one flat albedo per face. This preserves grass's
  green top, the block face tones, the dominant-material color, and the legacy S1
  keys 3/4 — all exactly as before.
- A **mixed face** reproduces the ratified 4×4-cell look (journal/0010)
  shader-side: `floor(uv * 4)` is the world-anchored cell, a hash of that cell
  picks **one** constituent weighted by the splat fractions, and the fragment
  paints that constituent's **flat registry albedo** — read from a `palette`
  uniform (one `Vec4` per atlas layer, built from `MaterialId::props().albedo`),
  **not** from any texture. Zero mosaic geometry, zero lighting.

Uniform faces flat, mixed faces speckled, all from the splat attributes the mesh
already carried — the data plumbing PBR-1 built, with a different color source, as
the mixture road promised.

## The gates-green shader that blanked the screen

The sharpest lesson of the session was cheap only because I ran the game.
`cargo fmt`, `cargo clippy -D warnings`, and 300-odd workspace tests all passed
with the fullbright shader containing `var active = 0u;` — and `active` is a
**reserved keyword in WGSL**. A custom material's shader is compiled by naga at
*pipeline build time*, on the GPU, at runtime; no CPU gate sees it. The first
`--fullbright` launch logged one line — `error: name 'active' is a reserved
keyword` — and would have rendered the entire terrain untextured (or blank) while
every gate stayed green. Renamed to `slots`; re-ran; clean.

> blogworthy: **the gate that isn't.** In an engine that JIT-compiles shaders at
> pipeline-build time, the shader is code your whole CI never runs. `fmt`,
> `clippy -D warnings`, and the full test suite are all green while a one-word
> typo blanks the screen. The only gate for an embedded WGSL shader is launching
> the binary and reading the log — the walk protocol's live smoke run is not
> ceremony, it's the compiler.

## What the walk should still photograph

The lit de-directionalization and the flat-fullbright-uniform behavior are
verified live (smoke shots in the worktree scratch). Not verified in-world, and
owed to the milestone walk:

- **The fullbright mixture speckle on a real mixed face.** Uniform spawn strata
  have no mixed faces; the speckle wants a placer band (the seed-1337 alluvium
  ~60 km out, journal/0010) or a member contact — the same scan-guided-site
  problem the walk-10 kill shot has, blocked on the owed pregen-introspection dev
  MCP tools. The shader compiles and the mesh carries the splat weights the
  fragment keys on (unit-tested); only the photograph is owed.
- **A geology cut (granite/basalt) at outcrop scale** under the new textures, to
  confirm the houndstooth is gone at the exact framing walk 14 shot
  (`0019-textured-outcrop`, `0019-strata-band-closeup`) — the before/after pair.

## Numbers and scope

Two custom materials now share one mesh; the fullbright palette is 26 `Vec4`s
(416 bytes) in a uniform. World-anchored UVs are f32 world voxel coordinates —
integer at corners, so tiles resolve exactly to ~±16 M voxels and acceptably to
the playable range; extreme deep-time coordinates would eventually lose texel
precision (noted, far field is a known defect anyway). No headless crate, no wire
type, and no lit-path lighting math was touched — the walk-14 tangent-frame fix
stands as-is.

## Walk 15 (main session): verified

*Appended post-integration (merge `016ac1b`, gates green on merged main —
38 suites, case-sensitive).* Re-shot the walk-14 framings
(`0020-outcrop-isotropic`, `0020-strata-band-closeup`): the houndstooth is
gone — granite reads as isotropic grain, block boundaries within
same-material runs are no longer legible; a faint fine-scale repetition
remains at close range, inherent to 16×16 placeholder tiles, rides until
real texture authoring. And the fullbright variant closed its own
verification gap: the agent had no mixed faces at spawn, but the quarry
walls are pore-partial mixtures everywhere — `0020-fullbright-flat-albedo`
shows the restored world-anchored olivine speckle on flat granite albedo,
the walk-13 diagnostic look, exactly. Screenshot auditability restored on
real geology.
