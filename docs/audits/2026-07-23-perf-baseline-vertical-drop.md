# Perf baseline — the vertical drop (2026-07-23)

STATUS: **STUB — numbers PENDING.** The instrument (spans + aggregating layer +
`--perf-drop` capture mode) has landed; the ranked numbers below are placeholders
for the integrator to fill by running the capture in the main session (a windowed
GPU dc-client could not be run from the background agent's environment).

## The scenario

The symptom (user field report, walk 0071, ROADMAP Observed *"Chunk gen time is
now noticeable in vertical streaming"*): dropping from height so the adaptive
load volume streams chunks *below* the player gets so choppy that "time appears to
slow to a crawl." This capture reproduces it **deterministically** — no MCP, no
physics — by teleport-stepping the player straight down in fixed increments so a
continuous curtain of fresh below-ground chunks streams every frame.

To reproduce:

```
cargo run --release -p dc-client --features perf -- --perf-drop 20
```

(from the repo root; add `--fullbright` if you want the flat-albedo control — it
does not affect the timing). The player warms up for ~2 s (initial world load),
then the aggregate is **reset** and the drop begins: −30 m every 0.5 s for the
requested N seconds, after which this file is rewritten with the real ranked
table and the process exits `0`.

## The instrument

`tracing` spans on the hot paths (feature-gated behind `perf`; **zero cost when
off** — the `perf_span!` macro expands to nothing). A custom
`tracing_subscriber::Layer` (installed via bevy's `LogPlugin::custom_layer`)
computes **self-time** (exclusive of child spans) per span name and a call count,
into an `Arc<Mutex<PerfAggregate>>` that is also a Bevy `Resource`
(`crate::perf::PerfHandle`). This file is the **first** consumer of that
authority; the ratified in-game perf/debug overlay is the second (S-3: a summary
derived from the authority, never beside it — journal/0080).

Self-time, not wall-time-with-children, is the ranked metric: a parent span
(e.g. `stream_chunks`) shows only the time *not* attributed to a child
(`chunk.gen`, `mesh_chunk`, …), so the killers surface without double-counting.

`bevy/trace` is enabled alongside, so bevy's own per-system / render spans are
captured too and appear in the ranking beside ours.

## Ranked self-time (worst first)

PENDING — integrator to run `--perf-drop 20` and paste the generated table here.
The generator emits exactly this schema:

| # | span | self-time (ms) | calls | mean (µs) | % of captured |
|---|------|----------------|-------|-----------|---------------|
| 1 | PENDING | PENDING | PENDING | PENDING | PENDING |

Captured window: PENDING s. Total captured self-time: PENDING ms.

## The spans (what they wrap, and why those are the suspects)

- `stream_chunks` — the whole near-field streaming system (the prime suspect: a
  synchronous gen+mesh loop on the main thread, `streaming.rs`).
  - `chunk.gen` — `authority.world.chunk(pos).clone()`: the authority lazily
    generates the chunk here.
  - `chunk.contents` — `authority.chunk_contents(pos)`: the material contents grid.
  - `far_pyramid.insert_l0` — the far reduction pyramid climb per generated chunk.
  - `mesh_chunk` — greedy meshing (includes the neighbour-fill closure it calls).
    - `neighbor_fill.gen` — the lazily-generated neighbour **contents** resolution
      (bounded ≤6 per chunk), the hidden cost of border culling against the
      authority.
  - `to_bevy_mesh` — the Bevy vertex-buffer build (where a pool would bite).
- `stream_far_surface` / `far_tile.build` → `far_tile.derive`, `far_tile.mesh` —
  the far-field surface tile derive+mesh per frame.
- `far_chunk.mesh` — the legacy S1 far volumetric mesh (keys 3/4 only; absent
  under the boot worldgen authority).
- `host.tick` — the authority/host fixed-cadence tick, driven from the client.
- `physics.step` — the collider-tile build + rigid-body step (colliders are
  generated lazily inside `PhysicsWorld::advance` via the solidity closure).
