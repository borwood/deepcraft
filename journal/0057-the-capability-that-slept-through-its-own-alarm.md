# 0057 — The capability that slept through its own alarm

*2026-07-21*

You could see the sky through the ground.

Not a streaming transient — the user walked three kilometres, stood still for
twenty seconds, and shot the same view twice
(`assets/0056-nearfar-check-after-3km.png`,
`assets/0056-holes-after-settle.png`). Identical. Horizontal bands of nothing,
cut clean through the hillsides, in a world that had been solid the day before.
The user's own diagnosis was better than any instrument I could have pointed at
it: *"the bands you see are missing side faces. these partials mostly have no
side faces — some of them do, following no apparent pattern."*

That last clause is the whole bug, stated as an observation. The pattern is
real; it just isn't visible from outside the mesher.

## The assumption, and where it was written down

journal/0010 shipped partial-height rendering: loose-only contents (debris role,
no structure) render as a box of height `eighths / 8`, snow-layer style, top
face always exposed. It was honest about the fact that nothing could feed it:

> Every voxel inside a recorded stratum gets a solid geology block AND exactly
> 8 debris (or 8 structure) eighths — full height. Sub-8 loose voxels only
> arrive with loose-material deposition […] So partial-height is a
> **tested-but-dormant** mesher capability […] and it will light up for free
> the day deposition produces its first sub-full column.

It did not light up for free. It lit up broken, and it took a walk to find out.

The reason is one line of arithmetic that nobody re-read, because it was never
*stated* as a dependency — it was recorded as prose in a module doc comment:

> Worldgen does not yet emit sub-8 loose voxels, so partial heights are
> exercised by tests until loose-material deposition lands.

That sentence became false on 2026-07-21 (journal/0055: the deep-time record
now decides what the world is skinned with, and the top of nearly every column
is a sub-8 loose partial). Nothing in the build had any reason to notice. A doc
comment is not a tripwire.

## The mechanism

The mesher's culling test was block-tier boolean:

```rust
neighbor_solid: &dyn Fn(i64, i64, i64) -> bool
…
if covered && !top_of_partial { continue; }
```

`covered` meant "the neighbouring cell's *block* is not Air". A 5/8 partial
standing next to a 3/8 partial therefore had its entire side face culled — the
neighbour was "solid", so the face was "hidden". But the neighbour only reaches
3/8 up the wall. The 2/8 band above it is exposed, and it was drawn by nobody:
not by the tall voxel (culled) and not by the short one (its own face, at 3/8,
is genuinely covered).

Which explains the "no apparent pattern" exactly. A partial keeps a side face
only where its neighbour is **air** (open cliff edge) or a genuinely full voxel
— and loses it everywhere the neighbour is another partial, whatever the
heights. Across a hillside of partials, that reads as bands of missing wall at
one elevation per terrace: sky through the ground.

The capability from 0010 had been *tested*, and the tests all passed, because
every test placed its synthetic 4/8 voxel in **open air**. Beside air, the old
culling and the correct culling agree. The one configuration the world would
actually produce — a partial beside a partial — was the one configuration no
test built.

## The fix: coverage, not solidity

A neighbour no longer answers a boolean. It answers *how much of its cell it
fills*, in `[0, 1]`:

```rust
neighbor_fill: &dyn Fn(i64, i64, i64) -> f32
```

and the six faces resolve against it by span rather than by veto:

- **side faces** emit the band `[cover, frac]`, and are culled only when
  `cover >= frac`. The quad is clipped to the part of the wall the neighbour
  fails to reach — one quad, not two, so nothing coplanar is ever double-drawn.
- **the top face** is emitted when nothing rests on the plane, or whenever we
  are a partial (existing snow-layer behaviour, kept verbatim: our top is below
  the cell ceiling, so it is exposed even under a solid voxel). A partial
  *above* still hides a full voxel's top — a 1/8 layer covers the whole
  footprint, it is short, not narrow.
- **the bottom face** is emitted unless the cell below is *full*. This one is
  new and today unreachable — partials sit at the top of columns, so nothing
  stands on them — but it is the same defect mirrored, and leaving it in place
  would be leaving the next dormant bug in the file.

`emit_face` grew from a single `frac` to a `(y_lo, y_hi)` span, and the
world-anchored UV follows the geometry unchanged, so a band tiles continuously
with the full faces around it (journal/0020's continuity property survives: the
V coordinate of a band's lower edge is `wy + cover`, which is exactly the V of
the neighbour's top edge).

**Nothing was invented to compute occupancy.** `height_frac` used to re-derive
"loose-only" from `shape()` and `debris()` privately; it now calls dc-core's
`VoxelContents::is_loose_only` and `loose_eighths` (journal/0052 added those
precisely so no consumer would hold a second opinion, and the mesher was
quietly holding one). The new `cover_frac(block, contents)` is the single rule,
and — this is the part that matters for the border — **the cross-chunk path
calls the same function**. An interior face and a border face now cull by
identical arithmetic, which is the property the tile-crack and buried-sheet
field reports cost so much to establish the first time.

## The border, and the cost of asking a better question

Making the border query occupancy-aware is not free the way making the interior
one was. Interior coverage is a lookup in a `ContentsGrid` we already hold.
Border coverage needs the *neighbouring chunk's* contents, and the authority
resolves contents a whole chunk at a time (`material_ids` → `MaterialChunk` →
`resolve_contents`). Asking it per border voxel would be four thousand chunk
resolutions per chunk meshed.

So `NeighborFill` (authority.rs) memoizes the resolved grid per chunk position
for the life of one mesh build. A chunk's entire border interrogation — every
face of every voxel in its one-voxel shell — touches at most the six chunks it
adjoins. It also short-circuits before the cache when the neighbour's block is
one that never carries contents (`block_uses_contents`), which is every Air
query and every legacy block, so the common case costs a block lookup and
nothing else. The `RefCell` is inherited from the code it replaces: the query is
handed to `mesh_chunk` as a `Fn` while the authority underneath is `&mut`,
because it lazily generates the unstreamed neighbour it is asked about
(journal/0017's rule that borders cull against the authority, never the client
cache's old wrong world).

The far-field rings and both benches pass `None` contents, so their neighbour
closures became `cover_frac(block, None)` — binary, as before, byte-identical
output.

## What it costs in triangles

A synthetic stepped 32³ surface chunk, every column ending in a loose partial of
a different height, meshed twice: once with the partials, once with every top
rounded up to 8/8 (which is exactly the geometry block-tier culling produced).
The test `partial_tops_triangle_cost_vs_block_tier_culling` prints the pair:

```
stepped 32³ surface chunk: block-tier culling 8696 tris,
                           occupancy-aware culling 11320 tris (+30.17%)
```

**+2624 triangles, which is +1312 quads on a chunk with 1024 surface columns** —
about 1.3 thin bands per column, exactly the shape you would predict from four
side neighbours of which most differ in loose depth. The cost is additive in
*surface*, never multiplicative in *volume*. That distinction is the lesson of
0010's 4×4 dither, which multiplied a mixed face into sixteen quads and had to
be undone in 0052; a fix that scales with surface is a fix you can keep.

The synthetic chunk is deliberately spiky (partial heights vary pseudo-randomly
per column), so its percentage is an upper bound on the real world's, where
neighbouring columns' loose depths are spatially correlated and many adjacent
pairs will be equal — and equal heights cull exactly as before.

## What this entry cannot claim

I could not walk the world. dc-client was running with the user inside it for
the whole implementation, and on Windows the live `.exe` is file-locked, so the
visual confirmation belongs to whoever re-walks the coordinates in
`0056-holes-after-settle.png`. Everything here is proven by unit tests and by
mechanism. If the bands are still there after this lands, the culling arithmetic
is not the (only) cause and this entry is the first thing to distrust.

## Two more assumptions the partials-first world falsifies

Deliberately hunted for, since one of these had already cost a walk:

1. **`block_uses_contents` gates render height, and the 0055 fallback
   vocabulary is Dirt/Stone.** A voxel whose block is Dirt renders full height
   no matter what its contents say — so a 3/8 loose top under a Dirt block is
   drawn as a full cube. Not a hole (coverage stays consistent on both sides of
   the gate, which is why this is a cosmetic lie and not a crack), but it means
   partial-height expression is silently conditional on which block the record
   resolved to. Worth a decision, not a patch.
2. **The far field is block-tier and always will be at these strides.** Far
   rings carry no contents, so the horizon renders every column full-height
   while the near field renders its partial tops — a systematic sub-voxel step
   at the LOD boundary. At coarse-ring voxel sizes it is far below a pixel; it
   is recorded here so nobody re-derives it as a bug.

> blogworthy: *a capability that shipped dormant with an explicit "this will
> light up for free" note, and the alarm clock nobody set.* The interesting part
> isn't the arithmetic — it's that the dependency was recorded in the right
> place (a doc comment, right above the code) and still failed, because prose
> can't fail a build. The tests that "proved" the feature all placed the
> synthetic partial in open air; the one arrangement the world would inevitably
> produce was the one the test author had no reason to imagine, because at the
> time the world could not produce it.
