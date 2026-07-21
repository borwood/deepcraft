# 0052 — one opinion per voxel

*2026-07-21 · the fill contract, built. Background agent; Phase 1 landed,
Phase 2 deferred with a spec, the collapse-cache evict gap closed.*

The session that ratified the fill contract (ARCHITECTURE.md § "The fill
contract", materials.md § "The forms design pass") handed down one sentence:
**contents are the single source of truth; the block is a pure derived
classification.** This entry is what happened when someone tried to make that
sentence true of the generator without moving the world by a single byte.

## The two opinions were never in the same room

The generator had two paths. `generate_chunk` walked the strata record and
asked `block_for_member(class_of(member))` — a seven-arm match from content
class to block. `material_ids` walked the same record and asked
`contents_for_event(member, event)` — the canonical `VoxelContents`
constructors. Neither consulted the other. They agreed because they were
written on the same afternoon from the same table, and journal/0010 installed a
*trust gate* downstream (the mesher believes contents only while the block is
still one of the geology blocks) so that when they eventually disagreed, the
block would win.

That gate is correct exactly as long as worldgen emits nothing but 8/8 voxels.
The moment it emits a fraction, the contents know something true that the block
cannot express, and "the block wins" becomes "the truth loses". So the fix is
not to synchronize the two opinions. It is to delete one of them.

## Where the classification actually lives

The obvious move — teach `classify` the content classes — is wrong, and finding
out why was the useful part of the day.

`block_for_member` maps a **class** to a block. `classify` sees only
`VoxelContents`, which is a multiset of `MaterialId`s: the class is gone by
then, deliberately (contents are the storage atom; they don't carry registry
identity, and they must not, or an order-dependent registry index would ride
into the canonical form and reopen the MixtureTable landmine). Making
`classify` class-aware means handing it a `GeologySet` — which makes the block
tier a function of the registry, so a pack that reorders members could in
principle move blocks, and the purity the invariant depends on is gone.

The resolution is to notice that the class→block table was standing in for
something simpler. Mudstone and siltstone read as `Block::Mudstone` not because
they share a *class* but because they are the same rock at the block tier — the
block vocabulary is coarser than the material vocabulary, and the coarsening is
a property of the material. So `block_twin(MaterialId) -> Block` replaced it,
and `classify` became pure over contents:

- **which multiset speaks** — structural fill, else debris, else pore fill
  (structure is what a voxel *is*: granite with an olivine pore inclusion is
  granite; with no structure the loose fill speaks, so a sand blanket is sand);
- **who wins inside it** — the most abundant material, ties to the lowest id.

Both halves are order-independent by construction: the segments are already
canonically sorted, and the tie-break reads the sort rather than the insertion
history. The generator's blocks are now derived once per stratum event from the
same `contents_for_event` constructor the material path uses. There is one
opinion, and the other path is a caller of it.

A quiet dividend: the relocation is *more* faithful than the table it replaced,
not less. The geology tests register a pack that binds `MaterialId::SILT` into
the fine-clastic class and `MaterialId::GRAVEL` into the coarse-clastic one. The
old table answered from the class; the new one answers from the material — and
because loose fines fold into the mudstone band and loose coarse into the
sandstone band, it answers *identically*. The one case where the two genuinely
diverge is a pack binding a material into a class whose block twin disagrees
with the material's own nature, and there the new answer is the one we want:
the material is authoritative, because the material is what is in the voxel.

## Proving "no bytes moved" instead of asserting it

The existing geology suite proves the world regenerates identically — two runs
of the *same* code. That is determinism, not neutrality; it would happily stay
green while the whole world shifted. What the contract needed was a comparison
across the change itself.

So the fingerprint sampler was written first, against the **unmodified**
generator: 80 chunks over three seeds and two extents, each chunk column taken
at its surface chunk and one chunk below it so the sample reaches buried strata
rather than only the air interface, FNV-1a over block bytes, material sidecar
bytes and the mixture table. Those numbers were captured, pasted into the test
as goldens, and only then was `collapse.rs` rewired. The rewired generator
reproduces them exactly. A golden that moves means the world moved — which is a
bug until an entry like this one says otherwise.

The invariant test is the other half: over the same sample, every voxel whose
contents are non-empty must carry exactly `classify(contents)`.

## The rule for voxels that have nothing to say

ARCHITECTURE.md states the invariant for "every voxel". It cannot hold there
yet, and the reason is worth writing down rather than quietly scoping.

A voxel above the surface is air and has no record. A voxel *at* the surface
carries the surface-veneer stub block (stubs.md § 2) and has no record. Below
the deep-time record the column is legacy soil and unrecorded basement — no
record. Ruin posts are wood dropped on top of the terrain — no record. All of
these are stubs with heirs already named; none of them is a place where two
opinions can disagree, because there is only one.

So the enforced rule is: **a voxel with no contents record is unclassified, not
classified as Air.** `classify` answers `Block::Air` for empty contents (a voxel
with nothing in it is air, as far as contents go), but the generator does not
apply it where it never wrote a record. The test enumerates the blocks allowed
to appear without one — air, the veneer stubs, the legacy soil band, basement
stone, ruin wood — and fails on anything else. A geology block with no contents
would mean the two paths disagree about *where* the record is, which is the
exact failure the contract exists to prevent. The exception can only shrink, and
it shrinks by itself as each stub gets its heir.

## The fraction that had nowhere to stand

Phase 2 was to emit the first honest fraction: the eolian sand blanket
(journal/0049 station 2 measured 2.09 m, floored to two 0.9 m voxels — 0.32 of
a voxel, about 2.6 eighths, thrown away at `deposit_deep_history`'s
`(thickness_m / voxel_m).round()`). It did not land, and the reason is
structural rather than an implementation difficulty.

The remainder is real ledger — exactly the kind of fraction the doctrine
sanctions. But *where does it go?* A partial voxel is a voxel that is not full,
and in a column that is solid from the surface down, a not-full voxel with solid
material above it is a **void under the ground**. The only place a fraction can
stand honestly is the top of the column.

And the top of the column is already taken. The topmost voxel is the surface
veneer — a stub block with no contents at all — and the eolian units sit under
the year-zero veneer the clastic pass deposits above them, which is computed as
a whole-voxel budget with no metres remainder to carry. So expressing the sand
blanket's fraction means making the surface voxel a partial loose top, which
means retiring the surface-veneer stub. Ratification 4 of the forms pass says in
so many words that the veneer's retirement "is its own slice, not a side
effect."

Landing Phase 2 would have meant either opening voids inside solid ground or
taking a slice the user reserved. Neither is worth a fraction that nothing
renders yet — journal/0010's partial-height mesher path is still dormant and
this slice does not touch the client. So Phase 2 is deferred with a spec (see
below), and Phase 1's byte-identity proof stays meaningful, which is the more
valuable of the two outcomes.

> blogworthy: "delete one of the opinions" — the difference between keeping two
> derivations in sync (a permanent tax, and a bug the first time someone edits
> one) and making disagreement structurally impossible by defining one as a pure
> function of the other. Plus the honest way to prove a refactor changed nothing:
> capture the fingerprint against the *old* code first, or you are only proving
> the new code is deterministic.

One detail worth carrying forward, because it shortens that slice
considerably: **clastic strata are already emitted in loose form.**
`contents_for_event` builds every fine and coarse clastic voxel as
`debris_only([host; 8])` — eight loose eighths, no structure. So the fractional
top is not a form change at all; it is a *quantity* change on one voxel, 8
eighths to k. Everything else the fraction needs already exists.

### The follow-on, precisely

1. Give `StrataEvent` a `top_eighths: u8` (0 = full), set only where the record
   has a real remainder: `frac = thickness_m / voxel_m - floor(...)`, eighths =
   `round(frac * 8)`, emitted only when the event is the **topmost** deposited
   stratum of the column and nothing is deposited above it.
2. Retire the surface-veneer stub for columns that have a record, so the
   topmost voxel *is* the record's top and can be partial. This is the slice
   ratification 4 reserved; it wants the user in the room.
3. Emit the top voxel as `VoxelContents::debris_only(&[host_material; k])` —
   loose form, no structure, the host material of the coarse-clastic member
   (sand is a *form*, not a new identity: ratification 2).
4. `classify` already handles it: loose clastic → the class block, unchanged.
   The mesher's partial-height path lights up for free (journal/0010) the day
   the first sub-8 loose voxel exists.
5. The invariant test needs no change; the golden fingerprints DO move, and that
   move is the deliverable of that slice.

## Operator note: the shared target directory lies

Twice in this session the build came back with `error[E0432]: unresolved import
`dc_core::classify` — no `classify` in the root` against a `dc-core` whose
`lib.rs` plainly re-exports it, and whose `classify.rs` was sitting right
there. The tell is in cargo's own output: the "Compiling" list named `dc-sim`
and `dc-worldgen` and **not** `dc-core`. Agent worktrees share one
`CARGO_TARGET_DIR` with the main checkout, a sibling agent was building
concurrently, and the freshness bookkeeping for `dc-core` came back wrong.
`cargo clean -p dc-core --release` and the build is correct again. Worth
knowing before spending an hour looking for a missing `pub`: when a symbol that
is demonstrably in the source is "not in the root", suspect the cache, not the
code.

## A cache that only got swept on the way past

Small, unrelated, and folded in here because it is three lines: journal/0050
found that `WorldGenerator::evict()` — the sweep that caps `lattice_memo`,
`locale_cache` and `region_cache` — was only reachable from `generate_chunk`.
Every far-field-only path (`coarse_surface`, `column_record`, `surface_elev_m`,
`surface_chunk_y`, `lattice_point`) could grow those maps without bound as long
as no chunk was generated, which is precisely what a horizon-streaming client
does. The public sampling entry points now sweep too. The module contract
already states that dropping a cache entry cannot change any answer — every
value is a pure function of seed and pregen — so this is determinism-neutral by
construction, and the byte-identity goldens above cover it: they are computed
through a generator that now evicts on every sampling call.
