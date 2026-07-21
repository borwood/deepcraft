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

use std::cell::RefCell;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use dc_core::{CHUNK_SIZE, ChunkPos};

use crate::app::{
    ChunkEntity, ChunkMap, CurrentScale, FloatingOrigin, Fullbright, FullbrightMaterialHandle,
    LoadedChunk, TerrainMaterialHandle, churn, to_render,
};
use crate::authority::Authority;
use crate::meshing::{MeshData, mesh_chunk};
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
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<ChunkMap>,
    mut authority: ResMut<Authority>,
) {
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
                if dist <= LOAD_RADIUS_M && !map.loaded.contains_key(&pos) {
                    missing.push(((dist * 1000.0) as u64, pos));
                }
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _)| *d);

    for (_, pos) in missing.into_iter().take(LOAD_BUDGET_PER_FRAME) {
        // Clone from the authority (lazily generated there, plus any applied
        // edits) — the cache never re-generates.
        let chunk = authority.world.chunk(pos).clone();
        // Render-only material contents (worldgen authority; None otherwise).
        // The block chunk above already warmed the generator's column cache.
        let contents = authority.chunk_contents(pos);
        // Border faces cull against the AUTHORITY (edits included, lazily
        // generating an unstreamed neighbour) — never the old wrong-world S1
        // fallback (journal/0017). Built AFTER the fetches above so the mutable
        // authority borrows don't overlap; generating a neighbour here also
        // warms the very cache this streamer is about to want.
        let authority_cell = RefCell::new(&mut *authority);
        let neighbor_solid =
            |x: i64, y: i64, z: i64| authority_cell.borrow_mut().is_solid_voxel(x, y, z);
        // Churn instrument (journal/0051): time the whole build — greedy mesh
        // + the Bevy vertex-buffer conversion, which is where a pool would bite.
        let build_start = std::time::Instant::now();
        let mesh_data = mesh_chunk(
            &chunk,
            pos,
            vscale.voxel_size_m() as f32,
            &neighbor_solid,
            contents.as_ref(),
        );
        let bevy_mesh = (!mesh_data.is_empty()).then(move || to_bevy_mesh(mesh_data));
        churn::record(&churn::NEAR_MESHES, &churn::NEAR_NANOS, build_start);
        let entity = if let Some(bevy_mesh) = bevy_mesh {
            // Spawn already positioned: `position_chunks` ran earlier this frame
            // and won't see this entity until the next one, and a default
            // transform would render one frame at the floating origin (flash).
            let (mx, my, mz) = pos.min_voxel();
            let min_m = glam::DVec3::new(mx as f64, my as f64, mz as f64) * vscale.voxel_size_m();
            let transform = Transform::from_translation(to_render(min_m - origin.0));
            let mut ent =
                commands.spawn((Mesh3d(meshes.add(bevy_mesh)), ChunkEntity(pos), transform));
            // Lit → LabPBR terrain material; `--fullbright` → unlit vertex color
            // (the walk-protocol diagnostic; the material must stay exactly as
            // before — pure vertex color, no lighting).
            if fullbright.0 {
                ent.insert(MeshMaterial3d(material.0.clone()));
            } else {
                ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
            }
            Some(ent.id())
        } else {
            None
        };
        map.loaded.insert(
            pos,
            LoadedChunk {
                chunk,
                contents,
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
