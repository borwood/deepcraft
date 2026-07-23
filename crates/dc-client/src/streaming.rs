//! Chunk streaming: load/unload in a true 3D radius around the player.
//!
//! This is the proof of the cubic-chunk lattice: the wanted set is a sphere of
//! chunk positions on all three axes, so descending into the chasm loads
//! chunks far below exactly the way walking north loads chunks far ahead.
//! Border faces mesh correctly on first try because out-of-map neighbor
//! queries fall back to the deterministic generator.
//!
//! Since the client-through-dc-api milestone, chunks stream in as **clones of
//! the authoritative hosted world's chunks** (terrain + applied edits), not
//! fresh generator output: the `ChunkMap` is a render/collision cache of the
//! authority, never a second source of truth. The generator fallback for
//! *unloaded* neighbors remains correct because the host uses the same
//! `TerrainGen` closure for never-edited chunks (an edited-but-unloaded
//! neighbor can mis-cull a border face until it streams in — accepted,
//! self-healing, and noted in journal/0002).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::tasks::AsyncComputeTaskPool;
use dc_core::{CHUNK_SIZE, ChunkPos};

use crate::app::{
    ChunkEntity, ChunkMap, CurrentScale, FloatingOrigin, Fullbright, FullbrightMaterialHandle,
    LoadedChunk, TerrainMaterialHandle, churn, to_render,
};
use crate::authority::{Authority, NeighborFill};
use crate::farpyramid::{FAR_PYRAMID_L0_BUDGET, FarPyramid};
use crate::meshing::{MeshData, NeighborShell, mesh_chunk};
use crate::meshtasks::{MAX_INFLIGHT_NEAR, NearMeshOutput, NearMeshTasks};
use crate::player::Player;
use crate::terrain_material::{ATTRIBUTE_MAT_LAYERS, ATTRIBUTE_MAT_WEIGHTS};

/// Chunks whose center is within this many meters of the player are loaded at
/// full detail. Meters, not chunks: every scale streams the same world volume.
/// S3 raised this from S1's 72 m; beyond it the far-mesh path (farmesh.rs)
/// renders LOD rings out to 1.2 km.
const LOAD_RADIUS_M: f64 = crate::farmesh::FULL_DETAIL_RADIUS_M;
/// Hysteresis: unload only beyond this distance. Also the radius the hosted
/// world's chunk budget is derived from (`Authority::chunk_budget_for`).
pub const UNLOAD_RADIUS_M: f64 = LOAD_RADIUS_M + 32.0;
/// Chunks generated + meshed per frame.
const LOAD_BUDGET_PER_FRAME: usize = 8;

// Bevy systems take their inputs as parameters by design; splitting this one
// to appease the 7-argument lint would only obscure the data flow.
#[allow(clippy::too_many_arguments)]
pub fn stream_chunks(
    mut commands: Commands,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    mut map: ResMut<ChunkMap>,
    mut authority: ResMut<Authority>,
    mut far_pyramid: ResMut<FarPyramid>,
    mut tasks: ResMut<NearMeshTasks>,
) {
    // Perf window (journal/0080): the whole-system parent span. Its self-time is
    // everything NOT inside a child phase below (unload sweep, want-set scan,
    // sort, spawn bookkeeping). Zero cost without `--features perf`.
    let _perf = crate::perf_span!("stream_chunks");
    let vscale = scale.scale;
    let chunk_m = vscale.voxels_to_meters(f64::from(CHUNK_SIZE));
    let center_of = |pos: ChunkPos| -> glam::DVec3 {
        glam::DVec3::new(
            (f64::from(pos.x) + 0.5) * chunk_m,
            (f64::from(pos.y) + 0.5) * chunk_m,
            (f64::from(pos.z) + 0.5) * chunk_m,
        )
    };

    // Unload far chunks.
    let to_unload: Vec<ChunkPos> = map
        .loaded
        .keys()
        .filter(|pos| (center_of(**pos) - player.pos_m).length() > UNLOAD_RADIUS_M)
        .copied()
        .collect();
    for pos in to_unload {
        if let Some(loaded) = map.loaded.remove(&pos)
            && let Some(entity) = loaded.entity
        {
            commands.entity(entity).despawn();
        }
    }
    // Cancel in-flight mesh tasks whose chunk left the load volume before their
    // mesh was ready (dropping the `Task` detaches it). Cheap, and it spares the
    // drain a GPU upload + entity spawn for a chunk about to unload.
    tasks
        .0
        .retain(|pos, _| (center_of(*pos) - player.pos_m).length() <= UNLOAD_RADIUS_M);

    // Collect missing chunks in a 3D radius, nearest first.
    let player_chunk = ChunkPos::from_world_voxel(
        vscale.voxel_at(player.pos_m.x),
        vscale.voxel_at(player.pos_m.y),
        vscale.voxel_at(player.pos_m.z),
    );
    let r = (LOAD_RADIUS_M / chunk_m).ceil() as i32;
    let mut missing: Vec<(u64, ChunkPos)> = Vec::new();
    for dy in -r..=r {
        for dz in -r..=r {
            for dx in -r..=r {
                let pos = ChunkPos::new(
                    player_chunk.x + dx,
                    player_chunk.y + dy,
                    player_chunk.z + dz,
                );
                let dist = (center_of(pos) - player.pos_m).length();
                // Skip chunks already loaded OR already meshing off-thread — an
                // in-flight chunk is neither loaded nor missing (journal/0083).
                if dist <= LOAD_RADIUS_M
                    && !map.loaded.contains_key(&pos)
                    && !tasks.0.contains_key(&pos)
                {
                    missing.push(((dist * 1000.0) as u64, pos));
                }
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _)| *d);

    // Only spawn as many tasks as keep the in-flight set under its cap, so a
    // meshing backlog can't grow without bound if the pool falls behind.
    let spawn_budget = LOAD_BUDGET_PER_FRAME.min(MAX_INFLIGHT_NEAR.saturating_sub(tasks.0.len()));
    let voxel_size_m = vscale.voxel_size_m() as f32;
    let pool = AsyncComputeTaskPool::get();
    for (_, pos) in missing.into_iter().take(spawn_budget) {
        // --- Main-thread gather of OWNED inputs (journal/0083) --------------
        // Clone from the authority (lazily generated there, plus any applied
        // edits) — the cache never re-generates. Each phase gets its own block
        // so its perf span drops before the next phase enters (journal/0080).
        // Chunk gen is 14 µs (the perf baseline) — it stays on the frame thread.
        let chunk = {
            let _perf = crate::perf_span!("chunk.gen");
            authority.world.chunk(pos).clone()
        };
        // Render-only material contents (worldgen authority; None otherwise).
        // The block chunk above already warmed the generator's column cache.
        let contents = {
            let _perf = crate::perf_span!("chunk.contents");
            authority.chunk_contents(pos)
        };
        // Feed the far reduction pyramid (FF2b, journal/0070): every generated
        // chunk climbs the block + material pyramids so far tiles over played
        // regions can render REDUCED geometry instead of the synthesized top
        // sheet. A fluid derived cache — never a second source of truth. Reads
        // the `FarPyramid` resource, so it stays on the frame thread.
        {
            let _perf = crate::perf_span!("far_pyramid.insert_l0");
            far_pyramid.insert_l0(pos, &chunk, contents.as_ref());
        }
        // Border faces cull against the AUTHORITY (edits included, lazily
        // generating an unstreamed neighbour) — never the old wrong-world S1
        // fallback (journal/0017). Occupancy, not solidity (journal/0057): a
        // border face culls against how much of the neighbouring cell is filled,
        // so a partial beside a shorter partial still emits its exposed band.
        //
        // The neighbour coverage is resolved HERE, on the frame thread, into an
        // owned `NeighborShell`: the resolution borrows `&mut Authority` (it
        // lazily generates + resolves neighbour contents — the `neighbor_fill.gen`
        // cost) and so cannot cross the thread boundary. `NeighborShell::resolve`
        // makes exactly the neighbour queries `mesh_chunk` would, so the offloaded
        // mesh is byte-identical (journal/0083; the crux decision — the 4.7 ms
        // neighbour resolution staying on-frame is the filed follow-on).
        let shell = {
            let fill = NeighborFill::new(&mut authority);
            NeighborShell::resolve(&chunk, pos, &|x, y, z| fill.fill(x, y, z))
        };

        // --- Off-thread pure mesh (journal/0083) ---------------------------
        // Move the owned inputs into an AsyncComputeTaskPool task. `mesh_chunk`
        // and the Bevy vertex-buffer conversion are a pure function of this data;
        // only the `Assets<Mesh>` insert + entity spawn (in `drain_near_meshes`)
        // must be back on the main thread. The `mesh_chunk` / `to_bevy_mesh` perf
        // spans now record on the task-pool thread and drop out of the frame's
        // `schedule` self-time envelope — how the win is measured.
        let task = pool.spawn(async move {
            // Churn instrument (journal/0051): time the whole build — greedy
            // mesh + the Bevy vertex-buffer conversion. Now a task-thread wall.
            let build_start = std::time::Instant::now();
            let mesh_data = {
                let _perf = crate::perf_span!("mesh_chunk");
                mesh_chunk(
                    &chunk,
                    pos,
                    voxel_size_m,
                    &|x, y, z| shell.cover(x, y, z),
                    contents.as_ref(),
                )
            };
            let mesh = (!mesh_data.is_empty()).then(|| {
                let _perf = crate::perf_span!("to_bevy_mesh");
                to_bevy_mesh(mesh_data)
            });
            churn::record(&churn::NEAR_MESHES, &churn::NEAR_NANOS, build_start);
            NearMeshOutput {
                pos,
                chunk,
                contents,
                mesh,
            }
        });
        tasks.0.insert(pos, task);
    }

    // Bound the far pyramid's L0 tier (S-1 knob; farthest chunks fall back to
    // synthesized far columns — a fidelity trade, never a correctness one).
    far_pyramid.enforce_budget(player_chunk, FAR_PYRAMID_L0_BUDGET);
}

/// Drain finished near-chunk mesh tasks: do the main-thread-only tail — insert
/// the built `Mesh` into `Assets<Mesh>`, spawn the already-positioned chunk
/// entity, and record the `LoadedChunk` cache entry (journal/0083). A chunk
/// becomes "loaded" here, a frame or more after its task was spawned in
/// [`stream_chunks`]; that lag is the only observable change (appearance order),
/// and the mesh itself is byte-identical to the synchronous path.
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn drain_near_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    scale: Res<CurrentScale>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<ChunkMap>,
    mut tasks: ResMut<NearMeshTasks>,
) {
    let voxel_size_m = scale.scale.voxel_size_m();
    for out in crate::meshtasks::drain_finished(&mut tasks.0) {
        let entity = out.mesh.map(|bevy_mesh| {
            // Spawn already positioned (see `stream_chunks`): a default transform
            // would render one frame at the floating origin (flash).
            let (mx, my, mz) = out.pos.min_voxel();
            let min_m = glam::DVec3::new(mx as f64, my as f64, mz as f64) * voxel_size_m;
            let transform = Transform::from_translation(to_render(min_m - origin.0));
            let mut ent = commands.spawn((
                Mesh3d(meshes.add(bevy_mesh)),
                ChunkEntity(out.pos),
                transform,
            ));
            if fullbright.0 {
                ent.insert(MeshMaterial3d(material.0.clone()));
            } else {
                ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
            }
            ent.id()
        });
        map.loaded.insert(
            out.pos,
            LoadedChunk {
                chunk: out.chunk,
                contents: out.contents,
                entity,
            },
        );
    }
}

pub fn to_bevy_mesh(data: MeshData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
    // Vertex color feeds the `--fullbright` unlit path; the lit terrain
    // material ignores it and samples the LabPBR atlases via the splat data.
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, data.colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs);
    mesh.insert_attribute(ATTRIBUTE_MAT_LAYERS, data.mat_layers);
    mesh.insert_attribute(ATTRIBUTE_MAT_WEIGHTS, data.mat_weights);
    mesh.insert_indices(Indices::U32(data.indices));
    mesh
}
