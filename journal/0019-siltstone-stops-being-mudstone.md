# 0019 — siltstone stops being mudstone

*2026-07-19 · PBR-1, the real material renderer (background agent; gates green
on the worktree branch, integration + milestone walk owed to the main session).*

For nine journal entries the ground has been lying about what it is made of.
Walk 10 named the lie precisely (journal/0011, ROADMAP Observed): the geology
pipeline picks *members* — siltstone here, mudstone there, diorite in the
basement — but a uniform-contents voxel painted its **block's** color, and
siltstone and mudstone are both the `Mudstone` block. The contact-smoothing that
3d built into the data was real and invisible. The interim dither (journal/0010)
only showed *mixed* voxels, and almost nothing is mixed; the vast uniform
majority of the world was flat block color. PBR-1 is the milestone that makes the
renderer read the material tier the sim has been recording all along.

## The shape of the replacement

The decided spec (docs/design/visuals.md § Material/texture model,
docs/rendering/PIPELINE.md § 5) is LabPBR three-texture packing sampled from
`texture_2d_array`s with **layer index = material id**, a custom Forward+
material, and — the part that kills the dither — mixed faces returning to single
quads with the constituents carried as **vertex attributes** and blended by
height/AO contrast (heightlerp), instead of the 4×4 mosaic that multiplied a
mixed face into 16 quads.

So the mesher stopped dithering. Every face now emits one quad carrying, per
vertex: a UV (one 16×16 tile per voxel face — the pixel-game texel density
visuals.md asks for), up to four material atlas layers, and four splat weights
from the voxel's eighth fractions. A *uniform* voxel is the degenerate case:
one layer at weight 1 — and that one layer is its **material** pack, not its
block's. Siltstone samples layer 17, mudstone samples layer 12. The walk-10
item is dead by construction: identity is the layer index, and the layer index
is the material id.

## One material for the whole world

The wrinkle that shaped the design: the atlas is indexed by `MaterialId`, but the
world is not made only of materials. The far field and the legacy S1 terrain
(keys 3/4) speak in *blocks* — grass, dirt, stone, wood — that have no material
twin and no `MaterialId`. A naive reading would render near-field geology
textured and everything else flat-colored, a visible seam at the load radius.

The fix is to widen the atlas past the material count: after the 22 material
layers come four **block-only** layers (grass, dirt, stone, wood), and the mesher
maps any block-colored face to one of them (geology blocks alias their material
layer; the placeholder packs already ship a `grass`/`dirt`/`stone`/`wood` set
with no material twin). Now a *single* `TerrainMaterial` renders the entire lit
world — near-field mixed geology, uniform strata, the far LOD rings, and the
legacy hill-field — because "which texture" is entirely a per-vertex layer pick,
never a material switch. The per-voxel identity lives in the mesh; the material is
stateless and shared.

> blogworthy: **the atlas that swallowed the block palette.** A voxel game has
> two vocabularies — a coarse *block* enum for gameplay and a fine *material*
> registry for geology — and the renderer wants exactly one. Appending the
> orphan blocks as extra texture-array layers past the material layers collapses
> both vocabularies into a single integer the vertex carries, so one Forward+
> material draws soil, sandstone, and a kilometre of distant hills with no
> branch and no seam.

## Keeping fullbright pure through a real renderer

The hard constraint was the sharpest design pressure. `--fullbright` — unlit,
pure vertex color — is the walk protocol's guarantee that lighting can never
masquerade as a geometry or data defect (journal/0004, CLAUDE.md). A textured
PBR material is the *opposite* of that guarantee. The two cannot be the same
material.

They don't have to share a material — only a mesh. Every chunk mesh now carries
*both* the representative vertex color (the dominant constituent's albedo) and
the splat attributes. At spawn the streamer picks the material by the flag: lit
runs gets the `TerrainMaterial` (samples the atlases, ignores vertex color);
`--fullbright` gets the unchanged unlit `StandardMaterial` (reads vertex color,
ignores the splat attributes Bevy's specializer simply doesn't bind). The
diagnostic mode is bit-unchanged in intent — unlit flat color — and it even
inherits the walk-10 kill for free, because the dominant-constituent color of a
uniform siltstone voxel is siltstone's albedo, not mudstone's. One mesh, two
materials, chosen at spawn: the renderer got real without the diagnostic losing
its meaning.

## The custom Forward+ material, in Bevy 0.19's post-render-graph world

The material itself is a straightforward `Material` impl once you accept that
Bevy 0.19 threw out the render graph (S4 already paid this tax for the post
stage, journal's S4 results): three `texture_2d_array` bindings plus a lighting
uniform in the material bind group (which is **group 3** in 0.19, not the group 2
the old docs show — `MATERIAL_BIND_GROUP_INDEX`), custom vertex attributes
(`Uint32x4` layers, `Float32x4` weights) declared in `specialize`, and the
surface shader embedded via `load_internal_asset!` so a vandalized `assets/` can
never blank the terrain — the same "never blank the screen" rule the shader-pack
loader lives by.

Two small mechanisms worth recording. The atlas is built by stacking N 16×16
layers into one tall image and calling Bevy's `reinterpret_stacked_2d_as_array`
— a `texture_2d_array` is just a stacked 2D image told to reinterpret its height.
And the lighting is deliberately *not* Bevy's clustered path: PBR-1's scope is a
single directional sun plus hemispherical ambient, so the shader lights from a
plain uniform. Shadows, point lights, tonemap/HDR, POM, and water are PBR-2; GI
is never (doctrine). Keeping the lighting hand-rolled and moderate also sidesteps
the walk-3 top-face blowout (near-vertical sun, no tonemap shoulder) until HDR
lands.

## The heightlerp and the top-N that never triggers

The splat blend is heightlerp: each layer's "elevation" is its splat weight plus
its LabPBR height sample, and layers within a narrow band of the maximum blend —
so a grain with a taller local height pokes through a lower neighbour with a
crisp pixel edge instead of the alpha-mush a linear blend would give (visuals.md:
"pixelated leaves visibly poke through sand"). For a uniform voxel this collapses
to sampling one layer — the common path is free.

"Top-N constituents chosen by the world-anchored hash" reads like it should be
load-bearing, and it is exactly not: a voxel holds eight eighths, real voxels
hold one to three distinct grains, and `SPLAT_N` is four. The over-N selection —
a fraction-weighted pick seeded by the voxel's world hash, mirroring how the old
per-cell dither weighted its pick — is genuinely reachable only by a synthetic
six-grain voxel in a test. It is headroom, not a hot path, and the hot path is
allocation-free (a fixed eight-slot constituent buffer, no `Vec` per voxel — the
mesher runs on the per-frame streaming budget).

## The five materials that had no packs

An honest snag: the atlas is indexed 0..22, but the placeholder pack set shipped
17 material packs (journal's placeholder-textures milestone) — the 3d
roster-proof widening (siltstone, conglomerate, diorite, andesite, olivine, ids
17–21) never got textures. Rather than render those as flat fallbacks, the
generator's material table was widened to mirror the full `MaterialId` registry
and regenerated: 26 packs now (22 materials + 4 block-only), deterministic, every
existing PNG byte-identical (the sha256-of-slug seeding held). A defensive
fallback still synthesizes a flat layer from the registry albedo for any missing
or malformed PNG, so the client degrades rather than crashes — but with the
regen, nothing triggers it.

## Numbers

Mesh of a full-solid 32³ chunk at N=2 (6144 exposed shell faces — the worst-case
fully-mixed band), 100-iteration mean, this machine:

- block-only (far field / legacy): **12 288 tris, ~2.0 ms**
- uniform contents (material layer): **12 288 tris, ~4.3 ms**
- mixed (splat, single quad): **12 288 tris, ~4.3 ms**

Against journal/0010's interim dither on the identical chunk: mixed-heavy was
**196 608 tris (16×), ~10.6 ms**. The mixed-face triangle count drops
**16× → 1×** — a fully-mixed chunk now costs the same geometry as a uniform one,
exactly the mosaic-collapse the scope promised. The contents path's CPU cost rose
over block-only (the per-voxel top-N sort) but fell well under the old dither,
and it pays only where contents exist.

## What the walk should photograph

Left for the main session's milestone walk (name the shots for
`journal/assets/0019-*`):

- **0019-siltstone-vs-mudstone**: a cut face where a siltstone member abuts a
  mudstone member under the same `Mudstone` block — the walk-10 kill, finally
  visible. (Find a member contact; `river_cells` / a deep cut.)
- **0019-placer-splat-closeup**: the alluvium from walk 9's river (seed 1337,
  ~60 km out) rendered as heightlerp splat rather than dither — gold grains
  poking through sandstone with a pixel edge, no glint.
- **0019-textured-outcrop**: a granite/basalt/mudstone cut at outcrop scale
  (the 0018 deep-time face) now LabPBR-lit, to judge the sun/ambient calibration
  against the walk-3 blowout note.
- **0019-fullbright-unchanged**: the same view under `--fullbright`, to confirm
  the diagnostic mode still reads as pure flat color.
- A far-field / near-field boundary shot to confirm the single-material world has
  no texture seam at the load radius (the phantom-far-world defect is separate,
  ROADMAP Observed).

## Walk 14 (main session): the black wall, and the NaN in the tangent frame

*Appended by the integrating session after the milestone walk. Merge
`c49566f`; gates re-run green on merged main (38 suites, case-sensitive).*

The first lit photograph of the quarry cut came back with the north wall
**pitch black** — not dark: black, emission-of-nothing black — while the
east/west walls and the floor rendered lit and textured beside it. A
no-shadow renderer (sun + hemispherical ambient) cannot legitimately
produce a black exterior face, so the walk had found a real defect within
its first three frames. The pattern (±Z faces black, ±X and top faces
fine) pointed at the one thing that differs per face orientation: the
tangent frame.

`tangent_frame` chose its helper vector as `(0,0,1)` for every non-top
face. For a ±Z face the helper is **parallel to the normal** —
`cross(helper, n)` is the zero vector, `normalize(0)` is NaN, and the NaN
propagates through the perturbed normal into every lighting term. The
±X faces worked by luck of the cross product; the ±Z faces rendered the
color of NaN, which the framebuffer clamps to black. One-line fix: the
helper is world-up for side faces (never parallel to any horizontal
normal) and X for top/bottom. Re-shot: every face lit.

> blogworthy: the color of NaN. A shader bug that produces *wrongly lit*
> faces gets caught by taste; one that produces NaN gets caught only by a
> walk, because NaN clamps to a black that looks like "shadow" until you
> remember the renderer has no shadows.

What the re-shot walk recorded (assets 0019-*): the **textured-outcrop**
and **strata-band-closeup** — the 0018 quarry under real lighting, grass
rim / brown soil-mudstone cap / black basalt band / pale granite mass, the
first photographs where the geology is *textured* and readable at outcrop
scale. The **lit-vista** — terrace risers shading differently by
orientation, top faces holding under the sun with no walk-3 blowout at
this calibration. And the **fullbright-unchanged** control, which was
honest enough to file a finding instead of a confirmation:

**Fullbright lost the mixture speckle.** The old mosaic gave the
diagnostic mode per-subquad dither colors; the single-quad splat mesh
carries one blended vertex color, so a mixed granite/olivine wall now
reads as a uniform pale field under fullbright. The unlit *path* is
unchanged (the hard constraint holds — flat color, no lighting, protocol
functional); the mixed-face *appearance* is not. Mixture presence is now
visible only in the lit path. Filed to Observed with the question it
raises: does the walk protocol want a fullbright splat variant, or is
lit-path-only mixture visibility acceptable?

Not photographed, owed: the **siltstone-vs-mudstone member contact** (the
walk-10 kill shot) — under grass cover a fine-clastic member contact is
not findable by eye; the hunt wants the pregen-introspection dev MCP
tools already in Observed. The placer closeup rides with it (same
scan-guided-site problem, ~60 km out).
