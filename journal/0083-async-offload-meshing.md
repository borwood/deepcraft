# 0083 — CPU meshing leaves the frame thread

The perf window (journal/0080) was built to answer one question the walk-0071
vertical-drop hitch had left open: when the world scrolls a curtain of fresh
below-ground chunks past the camera every frame, *what* is eating the frame? The
sequenced answer everyone expected was "synchronous chunk generation on the main
schedule" — `streaming.rs`'s `authority.world.chunk(pos).clone()` on the hot
path, the thing journal/0080 and the ROADMAP both fingered as the async-offload
target. The instrument said otherwise, and this slice is what the measurement
retargeted us to.

## The measurement overturns the hypothesis

`docs/audits/2026-07-23-perf-baseline-vertical-drop.md` — a real `--perf-drop 20`
capture on the windowed client — ranks per-span **self-time** during the drop.
Chunk generation is **0.1 % / 14 µs per call**. It is not the problem; it never
was. The problem is **CPU meshing**, top to bottom:

| span | self-time | %/frame | per call |
|------|-----------|---------|----------|
| `far_tile.derive` | 3839 ms | 10.3 % | 2038 µs |
| `mesh_chunk` | 3048 ms | 8.2 % | 1221 µs |
| `neighbor_fill.gen` | 1546 ms | 4.1 % | **4715 µs** |
| `far_tile.mesh` | 982 ms | 2.6 % | 521 µs |

So the async-offload slice changed shape mid-flight: **not** "move gen
off-thread" (gen is free — and gen time is not a constraint anyway, so we would
never have bought much) but "move the pure CPU *meshing* off-thread while it
stays a byte-identical function of owned data." Meshing reads no world state — it
is `(chunk, contents, neighbour-coverage) → MeshData` for the near path and
`(column-span stacks, cull mask, ring edges) → MeshData` for the far path — so
determinism is never in question. This is render-only work; the world stays
byte-identical. That is the north-star convergence in one sentence: **the runtime
clock is sacred, and this slice serves it by moving hot CPU work off the frame
thread without changing one bit of the world.**

## The shape: gather owned inputs on main, mesh on a task, upload on main

The standard Bevy pattern, one module (`meshtasks.rs`) plus a rewire of the two
streamers:

1. **Main thread — gather owned inputs.** `stream_chunks` still generates the
   chunk (14 µs), resolves its render-only contents, and feeds the far pyramid,
   all on the frame thread. Then it resolves the neighbour coverage into an owned
   `NeighborShell` (below).
2. **Off-thread — the pure mesh.** The owned inputs are `move`d into a
   `AsyncComputeTaskPool` task that runs `mesh_chunk` + `to_bevy_mesh` and hands
   back a `NearMeshOutput { pos, chunk, contents, mesh }`. The `chunk`/`contents`
   ride *into* the task and *back out* (moved, never cloned) so the drain can
   build the `LoadedChunk` cache entry.
3. **Main thread — the GPU tail.** `drain_near_meshes` polls finished tasks
   (`block_on(poll_once(&mut task))`, never blocking), inserts the `Mesh` into
   `Assets<Mesh>`, spawns the already-positioned entity, and records the
   `LoadedChunk`. A chunk becomes *loaded* here, a frame or more after its task
   was spawned.
4. **Budget.** The streamer spawns at most `LOAD_BUDGET_PER_FRAME` new tasks and
   never lets the in-flight set exceed a cap (`MAX_INFLIGHT_NEAR = 64`), so a
   meshing backlog cannot grow without bound; an in-flight chunk is skipped by
   the want-set scan (it is neither loaded nor missing) and cancelled if it
   leaves the load volume before its mesh is ready (dropping the `Task` detaches
   it).

The far path takes the identical shape. `build_far_tile_mesh` was *already*
written as a pure function of plain span data (journal/0070 filed "async
far-meshing is a drop-in" — it was): the streamer derives the tile on the frame
thread, moves the derived `DerivedTile` into a task that runs the pure build +
`to_bevy_mesh`, and `drain_far_meshes` does the GPU tail — including despawning a
seam tile's old entity once the fresh one is ready (no blink).

## The `neighbor_fill.gen` crux — measure, then decide

Meshing needs neighbour coverage to cull border faces. Today that comes from
`NeighborFill`, which lazily resolves an unstreamed neighbour's contents through
a `&mut Authority` borrow that **cannot cross a thread** — and it costs 4.7 ms
per call. To offload meshing we must hand it the neighbour data as *owned* data.
Two roads:

- **(a)** snapshot the neighbour inputs cheaply and fold the expensive resolution
  *into* the off-thread task — best, because the 4.7 ms also leaves the frame
  thread;
- **(b)** if gathering is irreducibly on-main, offload just the meshing and leave
  neighbour gathering on the frame thread — a partial win.

I investigated what the 4.7 ms actually *is*, and chose **(b)**, deliberately.

The 4.7 ms is `Authority::chunk_contents(neighbour)` — locking the single
`WorldGenerator` (an `Arc<Mutex<..>>`) and resolving a whole neighbour chunk's
material contents cold. The tempting (a) is to clone that `Arc` into the task and
resolve there. **That is a latency trap.** The frame thread's *own* per-frame
generator work — `chunk.gen`, `chunk.contents`, and the far `far_tile.derive` —
already locks the same mutex, serially, on the main thread. If a background task
holds that lock for milliseconds doing cold contents resolution, the frame
thread's next 14-µs `chunk.gen` blocks behind it. We would have moved the stall,
not removed it — and we cannot measure the contention without a windowed GPU
client, which the agent environment cannot run. The far derive is worse still:
`tile_column_stacks` reads not only the generator but the `FarPyramid` resource,
whose `known_node_grids` takes `&mut self` — a main-thread-owned Bevy resource
that simply cannot be handed to a task.

So this slice resolves neighbour coverage and far-tile derivation on the frame
thread as owned data, and offloads the pure mesh. The mechanism that makes the
near half work is `NeighborShell`: on the main thread it walks the chunk with the
*identical* solid-voxel / out-of-chunk-face predicate `mesh_chunk` uses and
records the coverage for exactly the world positions the mesher will query. The
offloaded `mesh_chunk` then reads a shell-backed closure instead of a live
authority borrow, and produces byte-identical `MeshData` — pinned by
`meshing::tests::shell_backed_mesh_equals_direct_mesh`, which meshes a
border-heavy chunk both ways and asserts every vertex array equal. Keeping
`NeighborShell::resolve` in the same file as `mesh_chunk`, right beside it, is
the drift defence: the two must iterate in lockstep, and the guard test locks it.

**The filed follow-on (a):** push `neighbor_fill.gen` (4.1 %) and
`far_tile.derive` (10.3 %) off-thread by giving each task its *own*
`WorldGenerator` minted from the shared `Arc<Pregen>` — no mutex contention, and
recomputation of cold caches is free under the two-clocks doctrine (gen time is
not a constraint). That is a real change to how `Authority` exposes its
generator, and it wants the contention measured on a live client, so it is
sequenced, not forced. A forced mess here would have been the wrong trade; the
partial win is the honest one.

## What the integrator should see move

Re-capture `--perf-drop 20` after merge. The self-time layer is **per-thread**
(perf.rs), so the offloaded spans record on task-pool threads and drop OUT of the
frame-thread `schedule` envelope: **`mesh_chunk` (8.2 %) and `far_tile.mesh`
(2.6 %) should leave the frame envelope entirely** — ~11 % of frame self-time,
gone from the main thread. `neighbor_fill.gen` (4.1 %) and `far_tile.derive`
(10.3 %) stay on the frame thread by design (the crux decision above) and are the
next target. The spans still fire — now on a task thread — so the win is directly
readable as their disappearance from the frame-thread ranking.

## Determinism + the one observable change

No headless crate touched; no worldgen/sim/authority world-state or logic
changed. The `HostWorld`, the generator, the contents resolution, the far
derivation are byte-for-byte what they were — this is entirely the dc-client
render path. The single observable change is **appearance ORDER / timing**: an
async completion is not strictly nearest-first (task *spawn* still is — the
streamers sort their want-set), and a seam-tile refresh swaps in a frame or two
later instead of same-frame. One rare corner rides along: an edit to a chunk in
the ~1–2 frames it is first streaming meshes the pre-edit snapshot; it self-heals
on any later edit to that chunk or a neighbour. No geometry, colour, or material
changes — the integrator should eyeball the drop post-merge to confirm the
appearance is unchanged, but the mesh-equivalence test already proves the bytes.

> blogworthy: the instrument overturned the plan. The whole async-offload slice
> was sequenced against "synchronous gen is the killer"; the perf window we built
> the day before said gen is 14 µs and *meshing* is the killer, and the slice
> retargeted itself to the evidence. Lens 1 (AI-native: the brief is a
> hypothesis, the measurement is the authority) and lens 3 (deepsim architecture:
> the two-clocks doctrine made "recompute per-task instead of sharing a
> contended lock" the *right* follow-on, not a hack).
