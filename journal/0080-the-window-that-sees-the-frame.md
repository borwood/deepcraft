# 0080 — the window that sees the frame

We measure gen-time to the millisecond. We have S-spike results, ritual A/Bs,
budget counters, a churn instrument that times mesh builds. And yet when the user
dropped from height on walk 0071 and watched "time appear to slow to a crawl,
sometimes," we had **no way to say where the frame went**. Gen vs meshing vs the
host tick vs neighbour generation — all of it was one opaque hitch. The project
that instruments prehistory to the epoch could not instrument its own present
frame. This slice is the first crack of the "perf window" (ROADMAP): the
instrument that turns the hitch into a ranked list of names.

> blogworthy: the deepsim codebase that could see a million years of geology but
> not the last 16 milliseconds of its own frame — and the S-3 discipline that
> kept the fix from becoming a throwaway file-dumper. (lens 3: reflexions in a
> deepsim codebase; lens 1: AI-native development — a slice built to a named
> future heir it was told not to build.)

## The gap, and the shape the answer had to take

The naive version of this slice is a profiler that runs, writes a text file at
the end, and exits. That would have answered the immediate question (what's slow
on a vertical drop?) and been the wrong architecture the moment we asked the
second question. Because the user had already named the **final** shape, and
ratified it (docs/design/ideas.md § "The perf/debug overlay is player-facing",
2026-07-23): *"final shape for game includes profiling with a switch in-game on a
perf/debug overlay; players like that kind of thing."* An in-game overlay reads
timing **live**, every frame — not from a file written at process exit.

So this is S-3 — *a summary is derived from the authority, never beside it* —
applied to timing data. The **authority** is a queryable in-memory structure:
`perf::PerfAggregate`, holding per-span cumulative self-time and a call count,
wrapped in an `Arc<Mutex<..>>` and inserted as the `PerfHandle` Bevy resource.
The `docs/audits/` ranked-table dump is the **first consumer** — it locks the
handle, calls `ranked()`, and formats markdown. The overlay is the **named heir**:
it will lock the same handle, call the same `ranked()`, and draw it, live. Not
built here (the brief was explicit: leave it a clean seam), but the aggregate is
already the thing it will read. The disappearing-consumer test passes by
construction: delete the docs dump tomorrow and the aggregate still exists in
exactly this shape, because the overlay needs it.

That is the load-bearing decision of the slice, and it cost almost nothing to
honour — but only because it was decided before the first line, not after.

## What the spans wrap, and why those are the suspects

The instrument is `tracing` spans on the hot paths, named with **stable literal
names** so the aggregate keys never drift. The prime suspect is
`streaming.rs::stream_chunks`: a **synchronous** gen-and-mesh loop on the main
thread, `LOAD_BUDGET_PER_FRAME` chunks per frame. On a vertical drop the adaptive
load volume streams a continuous curtain of fresh below-ground chunks, so every
one of those is a cold generate + a cold mesh, on the frame thread. The spans cut
that loop into its phases:

- `stream_chunks` — the whole system (parent). Its self-time is what's left after
  the children: the unload sweep, the want-set scan, the sort, spawn bookkeeping.
- `chunk.gen` — `authority.world.chunk(pos).clone()`, where the authority lazily
  generates the block chunk.
- `chunk.contents` — `authority.chunk_contents(pos)`, the per-voxel material grid.
- `far_pyramid.insert_l0` — the far reduction pyramid climb per generated chunk.
- `mesh_chunk` — greedy meshing.
  - `neighbor_fill.gen` — the **hidden cost**. Border-face culling asks the
    authority for neighbour occupancy, and the authority lazily *generates* the
    neighbour to answer. This is invisible in any profile that only names
    `mesh_chunk`. It nests under `mesh_chunk`, so it is charged to itself; it is
    bounded to ≤6 chunk-contents resolutions per built chunk (memoized per chunk).
- `to_bevy_mesh` — the Bevy vertex-buffer build (where a pool would bite —
  journal/0051's churn instrument names the same object; this is its span form).

And beyond streaming: `host.tick` (the authority's fixed-cadence world tick,
driven from the client — the "sim time dilating" hypothesis lives or dies here),
`physics.step` (collider tiles are generated lazily inside `PhysicsWorld::advance`
via the solidity closure), and the far-field derive: `far_tile.build` →
`far_tile.derive` / `far_tile.mesh`, plus the legacy S1 `far_chunk.build`.

Alongside our spans, the `perf` feature turns on **bevy's own `trace`** feature,
so bevy's per-system and render spans land in the same aggregate and rank beside
ours — the render half of the frame is not a blind spot.

## Self-time, not wall-time — the layer

The ranked metric is **self-time**: a span's inclusive elapsed *minus* the
inclusive time of its children. A parent like `stream_chunks` that is 90 %
`chunk.gen` should show ~10 %, or the ranking double-counts and the parent always
"wins." The aggregation is a `tracing_subscriber::Layer` installed into bevy's
own subscriber via `LogPlugin::custom_layer` — **never a second global
subscriber**, which would collide with bevy's `LogPlugin`. It keeps a per-thread
stack of `(name, enter-instant, accumulated-child-nanos)` frames: on exit, a
frame's self-time is its elapsed minus its accumulated children, and its full
elapsed is added to the parent's child-accumulator. Per-thread because bevy's
multi-threaded executor runs each system on one thread and the scoped span guards
nest on that thread — a thread-local stack attributes correctly without locking on
the hot enter path (the mutex is taken only on exit, to record).

A neat consequence of using bevy's re-exports (`bevy::log::info_span!`,
`bevy::log::tracing_subscriber`, `bevy::log::BoxedLayer`): **zero new
dependencies**, and no risk of a second `tracing-subscriber` version whose
`Layer`/`Registry` types wouldn't match `LogPlugin::custom_layer`'s `BoxedLayer`.

## Zero cost when off — runtime is sacred

Runtime is the sacred clock (CLAUDE.md), so the default build must pay **nothing**.
The `perf` cargo feature is off by default; it gates `bevy/trace`, the layer
install, the custom spans, and the capture mode. The spans are opened by a
`perf_span!` macro that is the whole trick: under `--features perf` it expands to
`bevy::log::info_span!("name").entered()`; **with the feature off it expands to a
zero-sized `PerfGuard`**. Every call site is `let _perf = perf_span!("x");`, so
the binding is a real value in both builds — which matters more than it looks.
The obvious no-op designs both fail the `-D warnings` gate: a statement macro that
expands to nothing leaves a stray `;` (`redundant_semicolons`), and an expression
macro returning `()` trips `clippy::let_unit_value` at the binding. A named ZST
guard sidesteps both and compiles out to nothing. Verified: `cargo clippy
--workspace --all-targets --release` (default, no perf) is clean, and the code
inside `enabled` — the aggregate, layer, and capture driver — is entirely behind
`#[cfg(feature = "perf")]`, so it is not merely dead-stripped, it is not compiled.

## The capture: a deterministic drop, no MCP, no physics

`--perf-drop <seconds>` (meaningful only with `--features perf`) scripts the
known-bad scenario deterministically. After a ~2 s warm-up (world load), it
**resets the aggregate** — so the captured window is the drop, not the load — and
then teleport-steps the player straight down, −30 m every 0.5 s, for N seconds.
Downward *teleport*-stepping, not gravity: the boot default is fly mode, so a
direct `pos_m.y` decrement is not fought by collision, and there is no physics
dependence to make the capture non-reproducible. The streamer chases the player
down, generating a fresh curtain of below-ground chunks every frame — the walk
0071 symptom, on rails. When the window elapses it writes the ranked artifact to
`docs/audits/2026-07-23-perf-baseline-vertical-drop.md` and exits `0` (the honest
exit contract, journal/0054).

## What the first ranked list showed

PENDING — a windowed GPU dc-client could not be run from the background
implementation environment, so the numbers are the integrator's to capture with
`cargo run --release -p dc-client --features perf -- --perf-drop 20`. The artifact
is landed as a schema-complete stub; the capture mode rewrites it in place with
the real ranked table (span, self-time, calls, mean µs, % of captured self-time,
worst-first). The hypothesis the instrument exists to test: that `chunk.gen` +
`chunk.contents` + `neighbor_fill.gen` — synchronous generation on the frame
thread — dominate, which is exactly what the sequenced next slice (async-offload
onto `AsyncComputeTaskPool`) is designed to move off the critical path.

## Gates

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets --release`,
`cargo clippy -p dc-client --all-targets --release --features perf`, and
`cargo test --workspace --release` — all green, with `cargo clean -p dc-client
--release` before the test gate and the `Checking`/`Compiling dc-client` lines
verified against this worktree's path (corrections #27/#34). No headless crate was
touched: every span sits at a dc-client call boundary. Determinism is untouched —
this is observability only, no gen/sim/mesh logic changed.
