# 0084 — the per-task generator: neighbour fill and far derive leave the frame thread

journal/0083 moved the pure *meshing* off the frame thread and stopped there, on
purpose. Two per-frame costs stayed behind because both read something the mesh
task could not: `neighbor_fill.gen` (~5 ms/call — resolving an unstreamed
neighbour chunk's material contents) locks the single shared `WorldGenerator`
behind the `Authority`'s `Mutex`; `far_tile.derive` (~1.9 ms/call, the biggest
single frame-thread span in the post-offload capture) locks that same generator
*and* reads the `&mut FarPyramid` Bevy resource. 0083 measured the trap and chose
the partial win: sharing that one `Mutex` across threads would let a background
lock-holder stall the frame thread's own 14 µs `chunk.gen`, moving the stall
instead of removing it. It filed the real fix and refused to force it.

This slice is that filed fix. The mechanism is one idea: **mint a per-task
`WorldGenerator` from the shared `Arc<Pregen>`.** Each background task resolves
its own neighbour contents / derives its own far surface against its *own*
generator — no shared mutex to contend on. The load-bearing correctness claim is
that this changes nothing the player sees: **generation is a pure function of
`(pregen, pos)`**, so a per-task generator over the same `Arc<Pregen>` produces
**byte-identical** output to the shared one. Its caches start cold, but a cold
cache and a warm cache compute the same answer — and recomputing it is free under
the two-clocks doctrine (gen time is not a constraint; chunk gen is 14 µs).

## North-star convergence

Runtime is sacred; gen time is free. This slice spends the free clock
(recomputing cold generator caches, off-thread, per task) to buy back the sacred
one (the frame thread stops paying ~5 ms neighbour-contents and the bulk of a
~1.9 ms far-derive per call), and changes not one bit of the world. It is the
two-clocks doctrine used exactly as designed: *better mechanism, not less world.*

## PRIMARY — `neighbor_fill.gen` into the near-mesh task

The near mesher culls a border face against how much of the adjacent cell the
neighbour fills. That coverage is `(block, contents-at-that-voxel)`: the **block**
must stay edit-aware (an edit changes blocks, and a chunk streaming in beside an
edited neighbour must see the edit — journal/0017), but the **contents** are pure
terrain render-only data (they never touch edits, receipts, or replay). Only the
contents resolution is the 5 ms; the block read is a cheap cached lookup. So the
split falls along that seam:

- `NeighborShellPlan::gather` runs on the frame thread and reads each border
  neighbour's **block** through the authority (edits included, lazily generating
  the neighbour — the same block gen the near path already pays). A block that
  carries no contents (air, full-height solid) gets its coverage *immediately*; a
  contents-bearing block is **deferred**, recording only its position, block, and
  neighbour chunk.
- `NeighborShellPlan::resolve` runs *inside the mesh task*, minting a per-task
  `WorldGenerator` (lazily, only if a deferred position needs it) and resolving
  each needed neighbour chunk's contents through it. `neighbor_fill.gen` now
  records on the task thread and drops out of the frame `schedule` envelope.

Byte-identity is exact because the block comes from the same edit-aware source and
the contents from a per-task generator that is byte-identical to the shared one.
`NeighborShellPlan` lives right beside `NeighborShell` and walks the identical
solid-voxel / out-of-chunk-face iteration `mesh_chunk` makes, so the two stay in
lockstep — pinned by `meshing::tests::plan_backed_mesh_equals_direct_mesh` (mesh
via the plan == mesh via a unified live `neighbor_fill` closure). The old
`NeighborShell::resolve` survives as the guard-test oracle.

> An aside the test earned: my first synthetic neighbour-block formula
> (`(x+y+z) % 3`) happened to map *every one* of the test chunk's clustered border
> positions onto air or full-solid, so the deferred-contents path was never
> exercised. The `assert!(plan.needs_contents())` line fired and caught the
> vacuous setup before the equality check could pass for the wrong reason. A
> non-vacuity assertion is cheap insurance against a green test that proves
> nothing.

## SECONDARY — `far_tile.derive`, offloaded (it *was* cleanly separable)

The far derive (`tile_column_stacks`) interleaves two impure reads per column:
`sample` (the generator's `coarse_surface` — 34² ≈ 1156 calls per tile, the bulk
of the cost) and `known` (the `FarPyramid`'s reduced node grids — a `&mut`
resource that cannot cross to a task). 0083 assumed this entanglement was the
blocker. It is not, because of one fact already documented in the code: **a tile's
34² columns touch at most the 3×3 patch of node plan columns centred on
`(tx, tz)`.** That set is a pure function of the tile coordinates — no generator,
no standoff logic — so the frame thread can **snapshot** exactly those nine
`known_node_grids` (owned, `Arc`-shared) and hand them to the task. The task then
mints a per-task generator for the `sample` half and reads the snapshot for the
`known` half. No `&mut FarPyramid` crosses the boundary; the 1156 coarse samples
leave the frame thread. What stays on the frame thread is only the snapshot itself
— the ≤9 `known_node_grids` calls, recorded under a new `far_tile.snapshot` span
so the integrator can see the residual — and that pyramid-derive work was already
being paid *inside* the old on-thread derive.

The snapshot is not a re-derivation of any logic (the A-1 trap): it is the *same*
`known_node_grids` call the on-thread `known` closure made, hoisted onto the frame
thread and pre-filled, so the task's `known` becomes a pure map lookup. Pinned by
`farmesh::tests::offloaded_far_derive_matches_on_thread` (offloaded
per-task-gen + snapshot derive == on-thread shared-gen + pyramid derive,
byte-for-byte).

## The per-task generator: how it is minted, and what it costs

`Authority` now holds the `Arc<Pregen>` beside the shared `Arc<Mutex<
WorldGenerator>>` (the `SurfaceAuthority::Worldgen` variant) and exposes it via
`worldgen_pregen()` — `None` under the S1 terrain authority (which has no worldgen
contents or coarse summary, so its border culls are binary and need no generator
at all; the near path simply never defers, the far path never runs). A task mints
`WorldGenerator::new_owned(pregen.clone())`: an `Arc` clone plus `assemble` (build
the site index, allocate empty caches) — **O(sites), no world generation.**

Measured: **9.9 µs mean** (`per_task_generator_construction_is_cheap`, Medium
pregen, 500 iterations). This is negligible three times over: it rides the *task*
thread, never the frame thread; it is minted at most once per task (near:
≤`LOAD_BUDGET_PER_FRAME`=8/frame; far: `FAR_SURFACE_BUDGET_PER_FRAME`=2/frame);
and it is amortized over everything that task then asks the generator (≤6 cold
neighbour contents at ~5 ms each, or 1156 coarse samples). Pooling or reuse would
be premature — 9.9 µs against milliseconds of amortized work is not worth the
`!Sync`-generator lifecycle complexity it would add. Filed as a non-need.

## Determinism

No headless crate touched (dc-worldgen/dc-core/dc-sim/dc-api unchanged); this is
entirely the dc-client render path. The world is byte-identical: blocks still come
edit-aware from the `HostWorld`; contents and coarse surface come from a per-task
generator proven equal to the shared one
(`authority::tests::per_task_generator_is_byte_identical_to_the_shared_one` — same
`chunk_contents` and `coarse_surface` over surface chunks/columns that carry real
data). The only observable change remains the one 0083 already accepted:
appearance **order/timing** (async completion is not strictly nearest-first; task
*spawn* still is). The `perf` spans still fire — now on task threads — which is how
the win is measured.

## What the integrator should see move

Re-capture `--perf-drop 20`. The self-time layer is per-thread (perf.rs), so:

- **`neighbor_fill.gen`** (was 3.1 % / 5092 µs mean on the frame thread) leaves the
  frame `schedule` envelope — it now records on a task thread.
- **`far_tile.derive`** (was 7.0 % / 1911 µs mean, the biggest single frame-thread
  span) leaves the frame envelope; a small residue remains as the new
  **`far_tile.snapshot`** span (the ≤9 per-tile `known_node_grids` calls, formerly
  folded inside the on-thread derive). If `far_tile.snapshot` is itself large in
  the re-capture, the pyramid-derive is the next target — but the generator
  sampling, the bulk, is now off-thread.

Together with 0083's `mesh_chunk` (8.2 %) and `far_tile.mesh` (2.6 %), essentially
all of the near/far meshing *and* its input resolution now lives off the frame
thread. Eyeball the drop post-merge to confirm the appearance is unchanged (the
byte-equality tests already prove the mesh bytes).

> blogworthy: the two-clocks doctrine made "recompute per-task instead of sharing
> a contended lock" the *correct* mechanism, not a hack — and 0083's discipline
> (measure the contention trap, take the partial win, *file* the real fix instead
> of forcing it) is exactly what let this slice land the real fix cleanly one day
> later. Lens 3 (deepsim architecture: the right primitive — a pure generator over
> a shared immutable pregen — dissolves a threading problem that looked like it
> needed a lock). Lens 1 (AI-native: the filed follow-on as a first-class artifact
> that a later agent picks up verbatim).
