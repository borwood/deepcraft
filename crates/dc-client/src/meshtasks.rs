//! Off-thread CPU meshing (journal/0083 — the async-offload slice).
//!
//! Meshing is a **pure function** of owned input data → `MeshData` (near:
//! [`crate::meshing::mesh_chunk`]; far: [`crate::farmesh::build_far_tile_mesh`]).
//! It reads no world/ECS state, so it is safe to run on a
//! [`bevy::tasks::AsyncComputeTaskPool`] thread while the frame thread keeps only
//! the cheap main-thread-only work: gathering owned inputs (chunk gen is 14 µs —
//! journal/0083's baseline), the GPU-side `Assets<Mesh>` insert, and entity
//! spawn. The perf baseline (docs/audits/2026-07-23) showed CPU meshing — not
//! chunk generation — was the per-frame killer during a vertical drop:
//! `mesh_chunk` 8.2 %, `far_tile.mesh` 2.6 % of frame self-time. Those spans now
//! record on a task-pool thread and drop OUT of the frame-thread `schedule`
//! envelope (the self-time layer is per-thread — perf.rs), which is exactly how
//! the win is measured.
//!
//! **Determinism is not at risk.** This is the render path only; the offloaded
//! functions are the same pure functions on the same owned inputs, so the
//! `MeshData` is byte-identical whichever thread builds it
//! (`crate::meshing::tests::shell_backed_mesh_equals_direct_mesh`). The only
//! observable change is chunk/tile appearance ORDER: an async completion may not
//! be strict nearest-first, and a seam-tile refresh swaps in a frame or two
//! later. Task spawn is still nearest-first (the streamers sort their want-set),
//! so the ordering drift is small.
//!
//! What stays on the main thread, and why (the `neighbor_fill.gen` /
//! `far_tile.derive` crux, journal/0083): the neighbour-coverage resolution the
//! near mesher needs (`neighbor_fill.gen`, 4.7 ms/call) and the far coarse-
//! surface derivation (`far_tile.derive`, 10.3 %) both read main-thread-owned
//! state — the `Authority`'s single `WorldGenerator` behind a `Mutex`, and the
//! `FarPyramid` resource (`known_node_grids` takes `&mut self`). Moving them
//! off-thread would either share that generator `Mutex` across threads (a
//! background lock-holder would stall the frame thread's own `chunk.gen` — a
//! contention regression we cannot measure without a windowed client) or hand a
//! `&mut` Bevy resource to a task (impossible). So this slice offloads the pure
//! meshing and resolves neighbour/derive inputs on the main thread as OWNED data
//! first. Pushing those two off-thread — via a per-task `WorldGenerator` minted
//! from the shared `Arc<Pregen>` (no mutex contention; recomputation is free
//! under the two-clocks doctrine) — is the filed follow-on (ROADMAP).

use std::collections::HashMap;
use std::hash::Hash;

use bevy::prelude::*;
use bevy::tasks::{Task, block_on, poll_once};
use dc_core::{Chunk, ContentsGrid};

/// Cap on near-mesh tasks in flight. The main thread spawns at most
/// `LOAD_BUDGET_PER_FRAME` new tasks per frame (streaming.rs), and this bounds
/// the total so a meshing backlog cannot grow without limit if the pool falls
/// behind (each near mesh is ~1 ms; the pool clears the budget well within a
/// frame in practice).
pub const MAX_INFLIGHT_NEAR: usize = 64;
/// Cap on far-tile mesh tasks in flight (same rationale; far tiles are larger
/// but fewer per frame — `FAR_SURFACE_BUDGET_PER_FRAME`).
pub const MAX_INFLIGHT_FAR: usize = 48;

/// One finished near-chunk mesh, handed back to the main thread by the polling
/// system. Carries the owned `chunk`/`contents` back out (they were moved INTO
/// the task to avoid a clone) so the drain can build the `LoadedChunk` cache
/// entry, plus the built `Mesh` (`None` = meshed to nothing — buried/air).
pub struct NearMeshOutput {
    pub pos: dc_core::ChunkPos,
    pub chunk: Chunk,
    pub contents: Option<ContentsGrid>,
    pub mesh: Option<Mesh>,
}

/// One finished far-surface-tile mesh. `old_entity` is the entity a **seam
/// refresh** rebuilt-in-place must despawn once the new mesh is ready (no blink);
/// `None` for a newly-appearing tile. `y_ref`/`cull_chunk` come back from the
/// pure build so the drain can place the transform and record the tile's cull
/// bookkeeping.
pub struct FarMeshOutput {
    pub level: u8,
    pub tx: i32,
    pub tz: i32,
    pub mesh: Option<Mesh>,
    pub y_ref: f64,
    pub cull_chunk: Option<(i64, i64, i64)>,
    pub old_entity: Option<Entity>,
}

/// In-flight near-chunk mesh tasks, keyed by chunk position. The streamer skips
/// positions already in flight (they are neither loaded nor missing), and the
/// drain removes an entry the frame its task completes.
#[derive(Resource, Default)]
pub struct NearMeshTasks(pub HashMap<dc_core::ChunkPos, Task<NearMeshOutput>>);

/// In-flight far-surface-tile mesh tasks, keyed by `(level, tx, tz)`.
#[derive(Resource, Default)]
pub struct FarMeshTasks(pub HashMap<(u8, i32, i32), Task<FarMeshOutput>>);

/// Poll every task in `tasks`, returning the outputs of those that finished this
/// call and removing them from the map. Never blocks: `poll_once` polls each
/// future exactly once, so an unfinished task is left in place (its `&mut`
/// borrow released) and retried next frame.
pub fn drain_finished<K, V>(tasks: &mut HashMap<K, Task<V>>) -> Vec<V>
where
    K: Copy + Eq + Hash,
{
    let mut finished_keys = Vec::new();
    let mut out = Vec::new();
    for (key, task) in tasks.iter_mut() {
        if let Some(value) = block_on(poll_once(task)) {
            out.push(value);
            finished_keys.push(*key);
        }
    }
    for key in finished_keys {
        tasks.remove(&key);
    }
    out
}
