# 0082 — the render-first wedge was already driven

> blogworthy (lenses: AI-native development; deepsim-reflexions): a brief is a
> hypothesis, and this one was written against a mental model of the mesher that
> the code had already outrun. The wedge it dispatched — "delete the Block→Material
> re-translation in the mesher" — turned out to be *already done* for the path it
> named, and *impossible* for the path it forgot. The deliverable is the finding,
> a guard test, and a corrected map for the integrator. This is the corpus
> outrunning the assistant, caught at work-time by a loud plea instead of a
> forced mess.

## The brief, and its premise

This was to be step 1 of the north star (materials are the universal substance
the API hangs off): the smallest, lowest-risk, byte-identical first move of the
block↔material collapse. The stated wedge:

> Today the mesher resolves a geology voxel's atlas layer by
> `block → block_layer(block) → material_layer(material)` — a Block→Material
> re-translation. The atlas is already material-keyed, so this is pure detour.
> Route contents-bearing faces `classify → material → material_layer` directly;
> `block_layer`'s geology arms then become dead and get deleted.

The brief flagged its own premise as a **hypothesis to verify**, and instructed:
if the wedge is *not* cleanly separable or *not* byte-identical without the Block
redefinition, **stop and report that as a finding — do not force a mess.** That
is exactly what happened.

## What the mesher actually does

`mesh_chunk` (`crates/dc-client/src/meshing.rs`) resolves a voxel's atlas layers
in one `match`:

```rust
let (layers, weights, contents_color) = match voxel.as_ref() {
    Some(c) => {
        let (l, w) = top_splat(c, voxel_seed(wx, wy, wz));   // → material_layer
        (l, w, Some(material_color(dominant_material(c))))
    }
    None => ([block_layer(block), 0, 0, 0], [1.0, 0.0, 0.0, 0.0], None),
};
```

`voxel` is `Some` exactly when the chunk was meshed **with a contents grid**, the
block **uses contents** (`block_uses_contents`, the seven geology blocks), and the
voxel's contents are **non-empty**. In that arm `top_splat` walks the contents'
constituents and emits `material_layer(m)` for each — it **never calls
`block_layer`**. The contents-bearing geology voxel already takes the direct
`contents → material → material_layer` route the wedge wanted to install. There
is no Block→Material re-translation on that path to delete. It was driven already
— installed piecemeal by journal/0010 (the mixture splat) and the partials work.

So where do `block_layer`'s geology arms (`Block::Mudstone =>
material_layer(MaterialId::MUDSTONE)`, …) actually get exercised? **Only the
`None` arm** — a solid block with no material to route through. Three live
producers of a geology block on that arm:

1. **The far-field reduction pyramid** (`farmesh.rs::push_quad`, line ~865):
   `let layer = block_layer(block)` for every top face of the coarse
   heightfield, where `block` is a `ColumnSpan.block` derived from the block
   pyramid (`MajorityNonAir` over real chunk blocks). Near the surface the world
   is geology (journal/0055 skins nearly every column), so this renders
   `block_layer(Block::Sandstone)` etc. in **production far rendering**.
2. **The block-only near/bench mesher** — `mesh_chunk(.., contents = None)`:
   `bench.rs`, `bench_storage.rs`, and the in-tree perf test all mesh full
   `Block::Sandstone` chunks with no contents grid, hitting the geology arm.
3. **Absent-contents geology voxels** in the near field itself — the trust gate
   (`block_uses_contents` + the `!is_empty()` filter) deliberately falls a
   geology block with a stale/empty contents record back to `block_layer(block)`.

## The finding (the loud plea)

**The geology arms of `block_layer` are not dead, and they are the same
mechanism as the four "legacy" arms the brief said to keep.** The brief's own
justification for keeping `grass/dirt/stone/wood` — *"those blocks arise only
from absent-contents voxels that have no material to route through"* — applies
**verbatim** to a geology block on the `None` arm: it, too, has no contents to
route through, which is precisely why it is on that arm. A block-only render path
(far pyramid, bench, absent-contents fallback) needs a layer for a geology block
that carries no material, and `block_layer`'s geology arms are the only thing
that answers. Deleting them would either break the far field (forbidden by this
slice) or make `block_layer` non-total over `Block`.

Two corollaries the investigation turned up:

- The near-field convergence the wedge sought **is real and worth pinning**: a
  geology block's *secondary* members do NOT collapse onto the block's layer —
  siltstone (a `Mudstone` twin) keeps `material_layer(SILTSTONE)`, distinct from
  `block_layer(Block::Mudstone) == material_layer(MUDSTONE)`. That distinction is
  the whole point of the direct route (the walk-10 "member identity is
  render-invisible" kill, `uniform_contents_sample_their_material_layer`).
- The brief's proposed acceptance test — *"for m in the geology materials:
  `material_layer(m) == block_layer(block_twin(m))`"* — only holds for the
  **seven primary** materials (those a block is named after). For the secondary
  members (siltstone, sand, conglomerate, gravel, diorite, andesite, charcoal,
  loam, …) it is **false by design**, because `block_twin` is many-to-one and
  the direct route resolves the member, not the block's canonical material.

## What landed

No deletion (it would not be byte-identical, per the finding). Instead the one
safe, valuable residue: a **guard test** that converts the accidental equivalence
into a defended invariant —
`meshing::tests::block_only_geology_layer_agrees_with_direct_material_layer`.
For the seven primary geology blocks it asserts `material_layer(m) ==
block_layer(block_twin(m))` (so the direct-material route and the block-only route
land on the identical atlas layer — a geology surface cannot flip texture the
instant it loses or gains a per-voxel contents record), and it asserts the
siltstone corollary stays *un*-equal (so the material route can never degenerate
back into the block route). This pins the two tables (`block_twin`, and
`block_layer`'s geology arms) against silent drift — the A-7 concern the seam
inventory already flagged on both.

## North-star convergence

This slice's convergence statement: **the near-field mesher already hangs its
atlas resolution off material, not block; the residual block→layer arms are the
block-only (contents-absent) render summary, which the north star retires by
redefining `Block = {Air, Material(MaterialId)}` — so the honest next crux is
that redefinition (Crux 1), not a mesher edit, and this guard test is the
regression net that redefinition will lean on.**

## The corrected map for the integrator

The render-first wedge, as a *deletion*, is a no-op that cannot be done safely in
isolation. The real remaining block→material render dependence is now precisely
located:

- **Near field, contents-bearing:** already converged. Nothing owed.
- **Near field, absent-contents geology fallback:** `block_layer` geology arm;
  retired when `block_uses_contents`/the trust gate is retired (parked downstream
  of the storage atom per the ratified order).
- **Far field (`farmesh.rs::push_quad`):** `block_layer` + `face_color` on the
  coarse heightfield — a *separate* slice, and the one place a real Block→layer
  round-trip still lives for geology. The far pyramid carries a material store
  alongside the block pyramid (`farpyramid.rs`), so routing the far top-face
  layer through material is feasible but is explicitly **out of scope** here
  (the brief forbade touching the far field / `ColumnSpan`).
- **Benches:** block-only by construction; follow `TerrainGen`'s retirement.

The block token's collapse (Crux 1, `Block = {Air, Material(MaterialId)}`)
subsumes all of these at once and is the correct next move — deleting
`block_layer`'s geology arms piecemeal ahead of it only trades one carve-out for
another.
