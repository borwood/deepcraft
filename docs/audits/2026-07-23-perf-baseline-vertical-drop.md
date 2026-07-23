# Perf baseline — the vertical drop (2026-07-23)

Captured by `cargo run --release -p dc-client --features perf -- --perf-drop 20`.
The player warms up ~2s (world load), the aggregate is reset, then the player
teleport-steps -30 m every 0.5s for 20s so a curtain of fresh
below-ground chunks streams every frame (the walk-0071 vertical-drop symptom).

## The instrument

`tracing` spans on the hot paths (feature-gated behind `perf`; zero cost when off).
A custom `tracing_subscriber::Layer` (installed via bevy's `LogPlugin::custom_layer`)
computes **self-time** (exclusive of child spans) per span name into an
`Arc<Mutex<PerfAggregate>>` that is also the Bevy `PerfHandle` resource. This file
is the first consumer of that authority; the in-game perf/debug overlay is the heir
that reads the same resource live (S-3 — journal/0080). `bevy/trace` is on, so
bevy's own system/render spans rank beside ours.

Self-time, not wall-time-with-children, is the ranked metric: a parent span shows
only the time not attributed to a child, so killers surface without double-counting.
The `%` column is over total captured self-time (sums to ~100).

## Ranked self-time (worst first)

| # | span | self-time (ms) | calls | mean (µs) | % of captured |
|---|------|----------------|-------|-----------|---------------|
| 1 | `schedule` | 23261.344 | 19564 | 1188.99 | 40.9% |
| 2 | `system` | 11035.849 | 784464 | 14.07 | 19.4% |
| 3 | `sub app` | 6040.006 | 2072 | 2915.06 | 10.6% |
| 4 | `far_tile.derive` | 3959.498 | 2072 | 1910.95 | 7.0% |
| 5 | `stream_chunks` | 2704.702 | 1036 | 2610.72 | 4.8% |
| 6 | `mesh_chunk` | 2341.123 | 2460 | 951.68 | 4.1% |
| 7 | `neighbor_fill.gen` | 1762.145 | 346 | 5092.90 | 3.1% |
| 8 | `chunk.contents` | 1358.305 | 2460 | 552.16 | 2.4% |
| 9 | `far_pyramid.insert_l0` | 1142.437 | 2460 | 464.41 | 2.0% |
| 10 | `multithreaded executor` | 1112.454 | 568938 | 1.96 | 2.0% |
| 11 | `far_tile.mesh` | 931.381 | 2072 | 449.51 | 1.6% |
| 12 | `RenderContextState::apply` | 312.994 | 44544 | 7.03 | 0.6% |
| 13 | `queue_submit` | 303.325 | 1036 | 292.78 | 0.5% |
| 14 | `system_commands` | 225.488 | 235094 | 0.96 | 0.4% |
| 15 | `par_for_each` | 102.884 | 71695 | 1.44 | 0.2% |
| 16 | `present_frames` | 56.276 | 1036 | 54.32 | 0.1% |
| 17 | `main_render_schedule` | 46.308 | 1036 | 44.70 | 0.1% |
| 18 | `chunk.gen` | 25.236 | 2460 | 10.26 | 0.0% |
| 19 | `prepared_mesh_producer` | 14.670 | 7252 | 2.02 | 0.0% |
| 20 | `main_transparent_pass_3d` | 13.922 | 1032 | 13.49 | 0.0% |
| 21 | `physics.step` | 10.584 | 1036 | 10.22 | 0.0% |
| 22 | `write_previous_input_buffers` | 9.218 | 1035 | 8.91 | 0.0% |
| 23 | `main_opaque_pass_3d` | 7.956 | 1035 | 7.69 | 0.0% |
| 24 | `update_from` | 6.720 | 2072 | 3.24 | 0.0% |
| 25 | `update` | 5.982 | 1036 | 5.77 | 0.0% |
| 26 | `indexed_cpu_metadata` | 5.520 | 9315 | 0.59 | 0.0% |
| 27 | `write_current_input_buffers` | 5.332 | 1035 | 5.15 | 0.0% |
| 28 | `main app` | 5.258 | 1036 | 5.07 | 0.0% |
| 29 | `opaque_main_pass_3d` | 4.887 | 415 | 11.78 | 0.0% |
| 30 | `indexed_batch_sets` | 4.843 | 9315 | 0.52 | 0.0% |
| 31 | `camera_schedule` | 4.653 | 1036 | 4.49 | 0.0% |
| 32 | `far_tile.build` | 4.640 | 2072 | 2.24 | 0.0% |
| 33 | `entity_sync` | 3.103 | 1036 | 2.99 | 0.0% |
| 34 | `write_phase_instance_buffers` | 2.484 | 9315 | 0.27 | 0.0% |
| 35 | `check_conditions` | 2.238 | 13697 | 0.16 | 0.0% |
| 36 | `handle_user_changes` | 2.178 | 1140 | 1.91 | 0.0% |
| 37 | `indexed_data` | 1.991 | 9315 | 0.21 | 0.0% |
| 38 | `non_indexed_batch_sets` | 1.831 | 9315 | 0.20 | 0.0% |
| 39 | `non_indexed_cpu_metadata` | 1.823 | 9315 | 0.20 | 0.0% |
| 40 | `non_indexed_data` | 1.811 | 9315 | 0.19 | 0.0% |
| 41 | `indexed_gpu_metadata` | 1.793 | 9315 | 0.19 | 0.0% |
| 42 | `non_indexed_gpu_metadata` | 1.791 | 9315 | 0.19 | 0.0% |
| 43 | `write_work_item_buffers` | 1.317 | 4140 | 0.32 | 0.0% |
| 44 | `collect_screenshots` | 1.043 | 1036 | 1.01 | 0.0% |
| 45 | `host.tick` | 0.919 | 1036 | 0.89 | 0.0% |
| 46 | `old_entity_cpu_culling` | 0.880 | 2072 | 0.42 | 0.0% |
| 47 | `winit event_handler` | 0.548 | 1036 | 0.53 | 0.0% |
| 48 | `main_transmissive_pass_3d` | 0.370 | 1035 | 0.36 | 0.0% |
| 49 | `handle_user_changes_on_colliders` | 0.311 | 1140 | 0.27 | 0.0% |
| 50 | `compute_contacts` | 0.222 | 1140 | 0.19 | 0.0% |
| 51 | `old_entity_cpu_culling_removal` | 0.203 | 2072 | 0.10 | 0.0% |
| 52 | `compute_intersections` | 0.171 | 1140 | 0.15 | 0.0% |
| 53 | `to_bevy_mesh` | 0.014 | 8 | 1.70 | 0.0% |

Captured window: 20.0 s. Total captured self-time: 56852.979 ms (reference top-level wall: 25781.936 ms).
