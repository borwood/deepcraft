# 0051 — The chunk store learns to forget

journal/0050 ended at a diagnosis and stopped there on purpose: the leak was
`HostWorld.chunks`, but the cure — eviction on authoritative state — is a
policy decision with a correctness edge, and a diagnosis slice had no business
making it. This entry makes it.

## The policy, and why it is allowed to exist at all

The whole fix rests on one asymmetry.

A chunk that has been **generated and never touched** is a pure function of
(seed, generator, position). Dropping it costs a regeneration and nothing else:
the bytes that come back are the same bytes. A chunk that carries an **edit**
holds information no derivation can reconstruct. Dropping it destroys a player's
work.

That is the S11 water doctrine — *store only what the derivation cannot
predict* — pointed at the chunk store instead of at a fluid field. So the store
splits in two along that line: an **evictable cache** of generated-untouched
chunks under a hard cap, and a **pinned set** of edited chunks that is never
touched by eviction.

Mechanically it is small. `HashMap<ChunkPos, Chunk>` becomes
`HashMap<ChunkPos, ChunkSlot>` where the slot adds two fields: `edited: bool`
and `touched: u64` (a monotonic access stamp). `chunk_at` — the one lazy-fill
door every voxel query in the world goes through — bumps the stamp on every
touch, and after inserting a freshly generated chunk it runs the budget sweep.
`set_block_raw` — audited as the **single** voxel-writing path in `HostWorld`,
with `chunk_at` private and the public `chunk()` handing out `&Chunk` only —
sets `edited`. Everything else (the command surface's `SetBlock` and `Fill`,
txn rollback, the client's edit.rs, every MCP write) reaches voxels through
that one function, so there is exactly one place the flag can be missed, and it
isn't.

Two choices inside that worth naming:

- **LRU, not a distance prune.** Distance-from-viewer is the obvious policy and
  it was the wrong one to reach for first: `HostWorld` is headless (dc-api may
  not depend on rendering or OS), so a viewer position would have to be pushed
  in as data every frame, and it would be *wrong* data for the query traffic
  that is not viewer-shaped — grounding scans, `surface:true` teleport probes,
  raycasts, border-face culls against unstreamed neighbours. Recency already
  encodes all of that, needs no input from anyone, and costs one `u64` per
  slot. The one thing the client does supply is the *size* of the working set,
  once, as a number: `Authority::chunk_budget_for` derives it from the
  streaming sphere at the active voxel scale (~2 360 chunks at the boot scale),
  clamped to [1 024, 16 384]. Correctness is indifferent to the value — it
  trades RAM against regeneration and nothing else — which is exactly the
  property that makes a client-supplied number safe here.
- **Sweeping in batches.** The sweep drops to ⅞ of the budget, so its
  `O(n log n)` sort runs once per ~295 newly generated chunks rather than once
  per chunk. The measured storm generates ~180 chunks/second, so this is a sort
  of ~2 400 `(u64, ChunkPos)` pairs every ~1.6 s.

An edited-then-rolled-back chunk keeps its flag. That is deliberate: the flag
errs toward retaining, and the only cost of being wrong in that direction is
memory.

## What the instrument said

Same repro as 0050 — `DC_MEM_PROBE=1`, `--horizon 3`, teleport to a fresh
±15 km coordinate every 1.8 s via the client's own MCP, RSS sampled externally
— but run **A/B on one binary**. `DC_CHUNK_BUDGET` (a measurement escape hatch
in the `DC_MEM_PROBE` family) set to 100 000 000 reproduces the pre-fix
behaviour exactly, because a budget nothing can exceed is a store that never
evicts. One build, two policies, no cross-build confound.

| run | jumps | RSS | slope |
|-----|------:|-----|------:|
| eviction OFF (`DC_CHUNK_BUDGET` huge) | 57 | 990 → 2 551 MB | **+27.4 MB/jump** |
| eviction ON, first 57 jumps | 57 | 973 → 1 042 MB | +1.2 MB/jump (warm-up) |
| eviction ON, next 147 jumps | 147 | 1 072 → 1 072 MB | **0.00 MB/jump** |

The fix run then kept storming for a total of ~210 jumps / 9 minutes and
finished where it started, oscillating in a ±20 MB band. That is the win
condition: flat, not merely shallower. Nothing else on that path retains.

The probe's new `host_chunks` field made the mechanism visible directly rather
than inferentially, which is the part 0050 could only reason about from code:

```
eviction OFF, t=70..120s:   host_chunks 14366 → 23781   evicted 0
eviction ON,  t=90..120s:   host_chunks  2129 →  2351   evicted 17464 → 22792   budget 2360
```

The store fills to its cap and then *stays* there while ~180 chunks/second flow
through it. By the end of the long session it had evicted 99 752 chunks and was
still sitting at 2 314 resident.

**A number in 0050 was wrong, harmlessly.** That entry estimated ~33 KB per
chunk and inferred "~750 chunks materialized per jump". `Block` is `#[repr(u16)]`
and `Chunk` is a dense `[Block; 32768]`, so a chunk is **64 KB**, and the
measured fill rate is ~335 chunks/jump. The diagnosis was unaffected — it only
ever needed "unbounded × large" — but the arithmetic is now measured instead of
estimated.

## The honest answer about edited chunks

Edited chunks accumulate for the whole session. That is in scope as designed —
the save layer is the decided heir — but "in scope as designed" is not the same
as "fine", so: at 64 KB per chunk, a player who edits voxels in 10 000 distinct
chunks in one session pins **~640 MB** that nothing will release until they
quit. Ten thousand chunks is a lot of ground (a 32-voxel cube each, so roughly
a 3 km × 3 km × 30 m slab if they touched every chunk in it), but it is not
absurd for a long building session, and the growth is invisible today because
nothing reports it. Now it does: the probe prints `host_edited` every 10 s, and
a dc-api test asserts the accumulation explicitly so that the day someone makes
edited chunks evictable, a test fails and says why. The real fix is spilling
edits to the save layer and dropping the clean-again slots; that is filed, not
done.

## Pooling: measured, and not built

The brief asked for pooling of chunk and far/LOD objects. 0050 had already
falsified pooling as the *leak* (mesh and tile counts oscillate 4–704 per
rebuild and are freed correctly), so the remaining question was whether it buys
anything as **allocation-churn reduction**. That needs a number, so the probe
grew one: every near-chunk and far-tile mesh build is timed and counted
(`churn::record`, two relaxed atomics and an `Instant` per built mesh).

Under the storm: **~130 near + ~32 far meshes per second, 3.5 s of build time
per 10 s of wall clock — 35 % of wall time.** That is a big number, and for
about a minute it looked like the pooling case had just made itself.

It hadn't. Two things kill it:

1. **That 35 % is per-voxel compute, not allocation.** A near build iterates
   32 768 voxels × 6 faces with a splat/contents lookup per solid voxel; a far
   build's dominant term is `tile_column_spans` sampling the worldgen coarse
   summary. The pool-able part is seven `Vec`s that grow by `push` — a handful
   of reallocs against millions of iterations.
2. **The seam cannot pool without adding a copy.** `to_bevy_mesh` does
   `mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)` — Bevy
   *takes ownership* of each attribute `Vec` and moves it to the render world.
   Our buffers are never handed back to us to reuse. A pool would have to keep
   its own scratch buffers and **copy** them into fresh `Vec`s at the
   `insert_attribute` boundary, converting zero copies per attribute into one.
   Pooling here would be a net loss, plus a permanent maintenance cost and a
   future leak surface.

So: **not built**, with numbers and a mechanism rather than a shrug. The one
residual idea that survives the argument — reusing `MeshData`'s buffers *inside*
`mesh_chunk`, which would at least kill the growth-reallocs — lives in
`meshing.rs`, outside this slice's write-set, and is worth at most a couple of
percent of a 35 % window. Filed to Observed with the measurement attached so
the next person starts from a number.

> blogworthy: "the pool that would have been slower than no pool". The user
> asked for pooling; the honest answer was no, and the reason is a two-line
> ownership fact about the engine seam (`insert_attribute` moves the Vec) that
> no amount of profiling-by-intuition would have surfaced. The instrument that
> nearly *argued for* the pool — 35 % of wall time in "mesh build" — is the same
> instrument that killed it once the window was read correctly. Measuring the
> right quantity and *attributing* it correctly are two different skills, and
> the second one is where this nearly went wrong.

## Proof obligations

Correctness here is one hard invariant — evict-then-regenerate must be
byte-identical — and one safety property — an edit must never be lost. Both are
tested (`crates/dc-api/tests/chunk_eviction.rs`), against a deliberately
high-entropy generator (every voxel decided by a hash of seed/pos/index) so that
"identical" is a real statement and not an artifact of a mostly-air world:

- `evicted_chunks_regenerate_byte_identically` — 3 seeds × 3 watched chunks,
  300 fresh chunks of pressure against a 16-chunk budget, full `blocks()`
  comparison after the eviction.
- `region_hash_is_independent_of_the_budget` — the end-to-end version: a world
  at budget 100 000 and one thrashing at budget 4 hash the same region equal.
- `an_edited_chunk_survives_eviction_pressure` — 400 chunks of pressure against
  a budget of 8; the edit reads back, and `host_edited` stays 1.
- `a_no_op_write_does_not_pin_a_chunk` — writing the block that is already
  there must not cost a permanent 64 KB.
- `edited_chunks_accumulate_beyond_the_budget` and `the_builtin_world_evicts_too`.

And live, over MCP, because a unit test cannot prove the *client's* wiring: a
`world_set_block` at (0, 100, 0), then 40 fresh-ground jumps (≈13 000 chunks
generated and evicted through the store), then `world_get_block` — `dc:wood`,
with `host_edited=1` in the probe the whole way. Neither the baseline nor the
9-minute fix session logged a `DeviceLost`, a panic, or any `ERROR` line.

## Repro

```
# baseline (pre-fix behaviour, same binary):
DC_MEM_PROBE=1 DC_CHUNK_BUDGET=100000000 dc-client --horizon 3   # ~27 MB/jump
# fixed:
DC_MEM_PROBE=1 dc-client --horizon 3                             # flat
# both: teleport to fresh +-15 km coords every 1.8 s over the dev MCP,
# sample RSS with Get-Process; watch host_chunks/host_evicted in the probe.
```
