# 0087 — block is material: the first slice, and the bug it flushed out

*Build, 2026-07-23 (agent A1, merged `de984eb`). The present-tier half of the
block↔material collapse — the north star's step 1 — landed byte-identical where it
had to be, and surfaced a latent invariant violation that had been hiding behind
the very thing we deleted.*

## What shipped

`Block` is now a **1-byte** atom. The ratified end-state is `{ Air,
Material(MaterialId) }`; the intermediate that shipped is `{ Air,
Material(MaterialId), Stone, Dirt, Grass, Wood }` — the four legacy S1 tokens are
still live (dc-client `TerrainGen`, collapse's ocean/wilds/basement/ruin-posts, the
`dc:*` edit palette) and retiring them is explicitly the *next* slice (Crux 2). The
user ratified the intermediate as an evolutionary step on the ratified path.

One byte came from a **niche**: a private `#[repr(u8)]` fieldless enum backs
`MaterialId` with 26 contiguous variants, so the compiler knows 26..=255 are
invalid and packs `Air` + the four legacy tokens into the niche beside the material
payload. `MaterialId`'s public API is untouched and manual serde preserves the
raw-`u8` wire format, so the materials sidecar is byte-identical. Measured:
`size_of::<Block>()` 2→1; `Chunk::raw_byte_size()` **64 KiB → 32 KiB**.

`block_twin` — the fifteen-name match and its `_ => Stone` arm — is **deleted**.
`classify` is now `dominant_material().map(Block::Material)`: every material,
including the nine that used to fall through to `Stone` (snow, leaf-litter,
potsherd, knapping-debris, ash, scree, bone, gold-dust, olivine), wears its own
face. The render path got *shorter* — `block_layer`'s geology arms were pure
`Block→Material` re-translation the collapse removed.

## The appearance change was wider than the brief

The brief anticipated the nine new faces. What it did not anticipate: because the
old `block_twin` collapsed *all* members of a class to one block, the far field and
stored blocks showed **one rock per class**. With the block now the dithered
member's own material, **within-class members are distinct at the block/far tier** —
siltstone vs mudstone, diorite vs granite, conglomerate vs sandstone, andesite vs
basalt — matching what the near field already rendered from contents. It closes a
near/far seam, but it is a real visible change.

> blogworthy (lens: AI-native development): the appearance-ratification call. This
> is exactly the class of change our process says gets the user's eye from pictures
> before it locks (the smooth-TIN-shipped-unseen lesson). The agent flagged it
> loudly; the integrator sorted it as user-owned and surfaced it; the user chose to
> **merge on the identity argument** — accepting the within-class variety
> sight-unseen because the near field already renders it and the gates prove the
> data-identity. Ratifying-by-choosing-not-to-look is still ratifying, and the
> record shows it was a decision, not an omission.

## The bug the collapse flushed out

The collapse surfaced a latent **S-3 violation** that had been invisible precisely
because `block_twin` shared it: `generate_chunk` derived a buried single-event
block from the event's *recorded* member, while `material_ids` interned the
per-voxel *dithered* host. Two derivations of one truth, disagreeing — but the
disagreement was masked while every member of a class collapsed to the same block.
Delete the collapse and the two paths diverge visibly. Fixed so the buried block is
`classify` of the same dithered host, memoized identically. Goldens re-blessed with
the clean signature: **only the two medium block-tier hashes moved; material and
table hashes byte-identical; the small world entirely unchanged** — exactly what
"only block-tier bytes changed, and correctly" looks like.

This is the general lesson of a collapse-class refactor: **deleting a shared
summary does not create disagreements, it *reveals* the ones the summary was
hiding.** The far-field cold-vs-warm material-identity violation (spines S-9,
ROADMAP Observed) is the same family, still open — a summary masking a
disagreement until you look underneath it.

## The tail

Solidity→occupancy was *not* drained: keeping the `Air` token means `is_solid()`
still compiles on the new atom, so the compiler forced zero migrations — the
~80-site consolidation is its own slice, deferred honestly. The far-field span
still carries a `Block` token (the storage/wire migration tail). Legacy S1 retire
is Crux 2. Categories-registrable follows. The keystone — the *deep*-cell inventory
— is the A2 spike (journal/0086, S17), unmerged pending the commit-semantics call.

Gate verified independently on merged main: all six crates compiled from the main
worktree, every suite `ok`, the five signature tests present by name.
