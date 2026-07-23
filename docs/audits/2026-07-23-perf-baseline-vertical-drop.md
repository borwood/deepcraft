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
| 1 | `schedule` | 17698.008 | 18284 | 967.95 | 47.4% |
| 2 | `system` | 4438.209 | 711802 | 6.24 | 11.9% |
| 3 | `far_tile.derive` | 3839.223 | 1884 | 2037.80 | 10.3% |
| 4 | `mesh_chunk` | 3048.306 | 2496 | 1221.28 | 8.2% |
| 5 | `neighbor_fill.gen` | 1546.404 | 328 | 4714.65 | 4.1% |
| 6 | `chunk.contents` | 1346.256 | 2496 | 539.37 | 3.6% |
| 7 | `far_pyramid.insert_l0` | 1158.112 | 2496 | 463.99 | 3.1% |
| 8 | `multithreaded executor` | 1056.671 | 522465 | 2.02 | 2.8% |
| 9 | `far_tile.mesh` | 981.786 | 1884 | 521.12 | 2.6% |
| 10 | `sub app` | 952.052 | 1884 | 505.34 | 2.6% |
| 11 | `RenderContextState::apply` | 293.466 | 40500 | 7.25 | 0.8% |
| 12 | `queue_submit` | 289.307 | 942 | 307.12 | 0.8% |
| 13 | `system_commands` | 212.341 | 212579 | 1.00 | 0.6% |
| 14 | `par_for_each` | 100.512 | 65205 | 1.54 | 0.3% |
| 15 | `present_frames` | 57.409 | 942 | 60.94 | 0.2% |
| 16 | `far_tile.build` | 55.738 | 1884 | 29.58 | 0.1% |
| 17 | `main_render_schedule` | 45.681 | 942 | 48.49 | 0.1% |
| 18 | `stream_chunks` | 45.210 | 942 | 47.99 | 0.1% |
| 19 | `chunk.gen` | 34.192 | 2496 | 13.70 | 0.1% |
| 20 | `prepared_mesh_producer` | 13.258 | 6594 | 2.01 | 0.0% |
| 21 | `main_transparent_pass_3d` | 12.780 | 938 | 13.62 | 0.0% |
| 22 | `physics.step` | 9.384 | 942 | 9.96 | 0.0% |
| 23 | `write_previous_input_buffers` | 8.013 | 941 | 8.52 | 0.0% |
| 24 | `main_opaque_pass_3d` | 7.612 | 941 | 8.09 | 0.0% |
| 25 | `update_from` | 6.342 | 1884 | 3.37 | 0.0% |
| 26 | `update` | 5.429 | 942 | 5.76 | 0.0% |
| 27 | `indexed_cpu_metadata` | 4.996 | 8469 | 0.59 | 0.0% |
| 28 | `main app` | 4.963 | 942 | 5.27 | 0.0% |
| 29 | `write_current_input_buffers` | 4.617 | 941 | 4.91 | 0.0% |
| 30 | `opaque_main_pass_3d` | 4.421 | 393 | 11.25 | 0.0% |
| 31 | `camera_schedule` | 4.339 | 942 | 4.61 | 0.0% |
| 32 | `indexed_batch_sets` | 4.211 | 8469 | 0.50 | 0.0% |
| 33 | `entity_sync` | 2.649 | 942 | 2.81 | 0.0% |
| 34 | `handle_user_changes` | 2.044 | 1151 | 1.78 | 0.0% |
| 35 | `write_phase_instance_buffers` | 1.931 | 8469 | 0.23 | 0.0% |
| 36 | `check_conditions` | 1.914 | 12578 | 0.15 | 0.0% |
| 37 | `indexed_data` | 1.628 | 8469 | 0.19 | 0.0% |
| 38 | `non_indexed_data` | 1.478 | 8469 | 0.17 | 0.0% |
| 39 | `non_indexed_gpu_metadata` | 1.468 | 8469 | 0.17 | 0.0% |
| 40 | `non_indexed_cpu_metadata` | 1.452 | 8469 | 0.17 | 0.0% |
| 41 | `indexed_gpu_metadata` | 1.446 | 8469 | 0.17 | 0.0% |
| 42 | `non_indexed_batch_sets` | 1.415 | 8469 | 0.17 | 0.0% |
| 43 | `winit event_handler` | 1.362 | 1652 | 0.82 | 0.0% |
| 44 | `write_work_item_buffers` | 1.196 | 3764 | 0.32 | 0.0% |
| 45 | `old_entity_cpu_culling` | 1.067 | 1884 | 0.57 | 0.0% |
| 46 | `collect_screenshots` | 0.943 | 942 | 1.00 | 0.0% |
| 47 | `host.tick` | 0.804 | 942 | 0.85 | 0.0% |
| 48 | `main_transmissive_pass_3d` | 0.339 | 941 | 0.36 | 0.0% |
| 49 | `handle_user_changes_on_colliders` | 0.287 | 1151 | 0.25 | 0.0% |
| 50 | `compute_contacts` | 0.203 | 1151 | 0.18 | 0.0% |
| 51 | `compute_intersections` | 0.189 | 1151 | 0.16 | 0.0% |
| 52 | `old_entity_cpu_culling_removal` | 0.178 | 1884 | 0.09 | 0.0% |

Captured window: 20.0 s. Total captured self-time: 37313.243 ms (reference top-level wall: 16516.918 ms).
