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
| 1 | `schedule` | 21775.377 | 20986 | 1037.61 | 32.3% |
| 2 | `system` | 12675.151 | 858464 | 14.76 | 18.8% |
| 3 | `sub app` | 8901.838 | 2270 | 3921.51 | 13.2% |
| 4 | `far_tile.derive` | 5885.981 | 2274 | 2588.38 | 8.7% |
| 5 | `stream_chunks` | 4178.122 | 1135 | 3681.16 | 6.2% |
| 6 | `neighbor_fill.gen` | 4029.689 | 2562 | 1572.87 | 6.0% |
| 7 | `mesh_chunk` | 2609.019 | 2562 | 1018.35 | 3.9% |
| 8 | `chunk.contents` | 1521.020 | 2539 | 599.06 | 2.3% |
| 9 | `multithreaded executor` | 1304.375 | 622302 | 2.10 | 1.9% |
| 10 | `far_pyramid.insert_l0` | 1286.245 | 2539 | 506.59 | 1.9% |
| 11 | `far_tile.mesh` | 1094.960 | 2274 | 481.51 | 1.6% |
| 12 | `far_tile.snapshot` | 629.471 | 2270 | 277.30 | 0.9% |
| 13 | `RenderContextState::apply` | 363.218 | 48805 | 7.44 | 0.5% |
| 14 | `queue_submit` | 350.324 | 1135 | 308.66 | 0.5% |
| 15 | `system_commands` | 277.445 | 257647 | 1.08 | 0.4% |
| 16 | `far_tile.build` | 168.616 | 2274 | 74.15 | 0.2% |
| 17 | `par_for_each` | 126.408 | 80075 | 1.58 | 0.2% |
| 18 | `present_frames` | 67.517 | 1135 | 59.49 | 0.1% |
| 19 | `main_render_schedule` | 59.473 | 1135 | 52.40 | 0.1% |
| 20 | `chunk.gen` | 31.393 | 2539 | 12.36 | 0.0% |
| 21 | `prepared_mesh_producer` | 21.676 | 7952 | 2.73 | 0.0% |
| 22 | `main_transparent_pass_3d` | 15.897 | 1117 | 14.23 | 0.0% |
| 23 | `physics.step` | 11.235 | 1135 | 9.90 | 0.0% |
| 24 | `write_previous_input_buffers` | 10.651 | 1135 | 9.38 | 0.0% |
| 25 | `main_opaque_pass_3d` | 9.729 | 1135 | 8.57 | 0.0% |
| 26 | `update_from` | 7.797 | 2270 | 3.43 | 0.0% |
| 27 | `indexed_cpu_metadata` | 7.148 | 10215 | 0.70 | 0.0% |
| 28 | `opaque_main_pass_3d` | 6.632 | 536 | 12.37 | 0.0% |
| 29 | `update` | 6.453 | 1135 | 5.69 | 0.0% |
| 30 | `write_current_input_buffers` | 6.428 | 1135 | 5.66 | 0.0% |
| 31 | `main app` | 6.387 | 1135 | 5.63 | 0.0% |
| 32 | `indexed_batch_sets` | 6.324 | 10215 | 0.62 | 0.0% |
| 33 | `camera_schedule` | 5.617 | 1135 | 4.95 | 0.0% |
| 34 | `entity_sync` | 3.690 | 1135 | 3.25 | 0.0% |
| 35 | `write_phase_instance_buffers` | 2.922 | 10215 | 0.29 | 0.0% |
| 36 | `handle_user_changes` | 2.527 | 1191 | 2.12 | 0.0% |
| 37 | `indexed_data` | 2.504 | 10215 | 0.25 | 0.0% |
| 38 | `check_conditions` | 2.299 | 14894 | 0.15 | 0.0% |
| 39 | `non_indexed_batch_sets` | 2.193 | 10215 | 0.21 | 0.0% |
| 40 | `non_indexed_cpu_metadata` | 2.151 | 10215 | 0.21 | 0.0% |
| 41 | `non_indexed_data` | 2.146 | 10215 | 0.21 | 0.0% |
| 42 | `indexed_gpu_metadata` | 2.140 | 10215 | 0.21 | 0.0% |
| 43 | `non_indexed_gpu_metadata` | 2.086 | 10215 | 0.20 | 0.0% |
| 44 | `write_work_item_buffers` | 2.043 | 4540 | 0.45 | 0.0% |
| 45 | `old_entity_cpu_culling` | 1.254 | 2270 | 0.55 | 0.0% |
| 46 | `collect_screenshots` | 1.206 | 1135 | 1.06 | 0.0% |
| 47 | `winit event_handler` | 0.954 | 1135 | 0.84 | 0.0% |
| 48 | `host.tick` | 0.916 | 1135 | 0.81 | 0.0% |
| 49 | `compute_contacts` | 0.614 | 1191 | 0.52 | 0.0% |
| 50 | `main_transmissive_pass_3d` | 0.432 | 1135 | 0.38 | 0.0% |
| 51 | `handle_user_changes_on_colliders` | 0.343 | 1191 | 0.29 | 0.0% |
| 52 | `old_entity_cpu_culling_removal` | 0.232 | 2270 | 0.10 | 0.0% |
| 53 | `compute_intersections` | 0.187 | 1191 | 0.16 | 0.0% |
| 54 | `to_bevy_mesh` | 0.019 | 10 | 1.91 | 0.0% |

Captured window: 20.0 s. Total captured self-time: 67490.472 ms (reference top-level wall: 34757.863 ms).
