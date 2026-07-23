# 0083 — Async-offload CPU meshing off the frame thread

STUB — in progress (async-offload slice). Narrative to follow: the measured
before (docs/audits/2026-07-23), what was offloaded (near `mesh_chunk`, far
`build_far_tile_mesh` → `AsyncComputeTaskPool`), the `neighbor_fill.gen` /
`far_tile.derive` decision, and the north-star convergence (runtime clock kept
sacred; world byte-identical).
