# 0010 — the ground learns to show its grains (DRAFT)

*DRAFT — background agent (ROADMAP 3c-2, material-tier geology in the client).
Gates green on the worktree branch; the main session integrates, walks a placer
fan, and photographs the first visible mixtures before this entry is finalized.
Placeholder walk section below.*

Journal/0008 ended on an honest complaint: a single-material wall under
fullbright flat shading is a *featureless color field* — the first quarry shot
was a full frame of undifferentiated granite pink. The banding carried all the
information; within-material structure carried none. 3c-2 is the interim answer
the visuals road decided (2026-07-19): every material carries an albedo, and a
mixed voxel's faces render a **world-anchored deterministic dither** — pixels
composing ground, no textures, no shaders. The later splat pipeline replaces the
*color source*, not the plumbing this entry builds.

## The seam already had a shape; the materials needed a parallel lane

3c-1 pushed only **blocks** through the client's chunk seam (journal/0008): the
`HostWorld` closure calls `generate_chunk`, block bytes are order-independent,
and the order-sensitive `MixtureTable` — whose ids are generation-order-dependent
by documented S8 design — stayed home. 3c-2 needed the *materials* at the mesher
without letting that landmine anywhere near sim state, receipts, or replay.

The resolution is a parallel lane on the *same* `Arc<Mutex<WorldGenerator>>` the
block seam already holds. A new `WorldGenerator::chunk_contents(pos)` runs the
material half of `generate_chunk_with_materials` (factored out so the two share
one classification loop and one warm column cache) and then does the one thing
that disarms the landmine: it **resolves** the chunk's `MixtureId` palette
against the region table into a `ContentsGrid` — a palette of canonical
`VoxelContents` plus one index per voxel — *before anything leaves the
generator*. `MixtureId` is order-dependent; `VoxelContents` is canonical
(multiset order destroyed at construction, S8) and order-independent. So the
render path emits only the order-independent thing. The client fetches blocks
through the `HostWorld` (replay identity, untouched) and contents through
`Authority::chunk_contents` on the same lock (caches already warm from the block
fetch). A debris-free chunk resolves to `None` and costs nothing downstream,
exactly as it attaches no storage sidecar — the S8 zero-cost invariant carried
into the render lane.

> blogworthy: "resolve at the boundary" — how to feed an order-dependent intern
> table to a renderer without ever letting an order-dependent id escape: keep
> the ids on the generation side of one lock, hand out only the canonical
> contents they point at, and the determinism-sensitive world never learns the
> table exists.

## What the dither surfaced: mixtures live only where the placer put them

The dither rule is simple: subdivide each mixed face into 4×4 pixel cells; each
cell picks ONE constituent's albedo by a position-seeded hash (world coords +
face + cell), weighted by that constituent's eighths fraction. The weighting
falls out for free — `VoxelContents::filled_slots()` lists each material once
per eighth it occupies, so a *uniform* hash pick over that eight-slot multiset
*is* a fraction-weighted pick over constituents. A 6-sandstone / 2-gold voxel
picks gold in ~2 cells of 16, and nobody had to special-case ore: small
fractions stay sparse because they *are* sparse. No glint, no highlight — the
visuals road's "subtle ore" is the absence of code, not the presence of it.

Building it surfaced the real shape of the current world: **the only mixed
voxels that exist are placer-enriched clastic**. Every other recorded voxel is
either 8/8 debris of one clastic material (uniform → single-color fast path,
byte-identical to 3c-1) or 8/8 igneous structural fill (uniform). The placer
pass substitutes gold-dust grains into some eighths of a clastic-coarse event;
that substitution is the *only* thing in the vanilla world that makes
`filled_slots()` heterogeneous. So the dither's entire visible footprint today
is the placer fans — which is exactly why the walk that proves this milestone is
a walk to a river's alluvium, not a random dig. Uniform geology looks identical
to 3c-1 (the block palette and the material albedos were set equal on purpose);
the fans are where the ground grows grain.

## Partial-height: a mesher capability with nothing to render yet

Loose-only contents (debris role, no structure) render as partial-height boxes —
height = eighths/8, snow-layer style, top face always exposed even under a solid
neighbor. The collider stays binary per block tier (the accepted, documented
visible mismatch). But an honest look at the worldgen block-tier reader closed
the loop the task flagged: *the "block says Air but loose eighths exist" case
cannot occur yet.* Every voxel inside a recorded stratum gets a solid geology
block AND exactly 8 debris (or 8 structure) eighths — full height. Sub-8 loose
voxels only arrive with loose-material deposition (gravity-by-march, breaking-as-
debris), which isn't built. So partial-height is a **tested-but-dormant** mesher
capability: the geometry is proven by a synthetic 4/8 voxel (half-height box,
exposed top), and it will light up for free the day deposition produces its first
sub-full column. No machinery was invented to feed it.

The dither and partial height compose through one trick: cell corners are
bilinear over the *height-remapped* face corners, so a partial mixed face is
still 16 cells, just a shorter box.

## A guard against the edit that hasn't happened

Edits don't touch materials yet (documented). That leaves a trap: a voxel edited
from Sandstone to Stone keeps its render-only contents, and a naive mesher would
dither the new Stone voxel as sandstone. The gate is the block itself — contents
are consumed only while the voxel is still one of the four geology material
blocks. Any edit changes the block away, and the voxel falls back to its block
color. Stale contents can't bleed through because the block, not the contents,
decides whether the contents are trusted.

## Numbers

Mesh of a full-solid 32³ sandstone shell at N=2 (6144 exposed faces, the
worst-case fully-mixed band), 100-iteration mean:

- block-only (S1 terrain / far field): **12 288 tris, ~1.33 ms**
- uniform contents (fast path): **12 288 tris, ~1.64 ms** (same tris; the
  ~0.3 ms is the per-voxel contents lookup)
- mixed-heavy, 4×4 dither: **196 608 tris, ~10.6 ms** — exactly 16× the
  triangles, since every exposed face becomes 16 quads.

Read the 16× as a ceiling for a *fully*-mixed chunk. Real placer bands are thin
slabs a few voxels deep, so the actual per-frame cost is a small fraction of a
uniform chunk's — the dither pays only where contents are genuinely mixed, and
almost nothing is.

## Design choices flagged for ratification

- **Dither cell count = 4×4** per face. Reads as ~22 cm chunky pixels at N=2;
  8×8 is the finer alternative the visuals road names, at 4× the triangles.
- **Sidecar transport = resolve-to-`ContentsGrid` at the generator boundary**
  (not shipping `MaterialChunk` + table to the client). One source of truth
  (the decided sidecar path), no intern id ever crosses the lock.

## Walk N: the placer fan *(placeholder — main session)*

*The main session walks to a river's alluvium under `--fullbright`, digs to the
clastic-coarse band, and photographs the first sandstone-with-gold dither
against a uniform sandstone wall — the before/after the milestone exists to
show. Screenshots land in `journal/assets/0010-*`. `eye_in_solid` checked on
every pose.*
