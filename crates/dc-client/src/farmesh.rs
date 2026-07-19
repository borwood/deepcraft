//! Far-mesh path (S3): terrain beyond the full-detail radius rendered from
//! LOD level 1+ chunks — coarser voxels, bigger cubes, level by distance.
//!
//! Level-L chunks are 32^3 cubes of `2^L`-sized voxels on the level-L lattice
//! (the same lattice the dc-core LOD pyramid uses: level-L chunk P covers
//! level-0 chunks `2^L * P .. 2^L * (P+1)`). In this spike client the level-L
//! voxel data comes from sampling the deterministic generator on the coarse
//! grid (`VoxelScale` with a `2^L`-sized voxel) rather than from a persisted
//! LOD pyramid: without a save layer, deriving 1 km of pyramid would require
//! full-res-generating 1 km of world (~5 min), while coarse sampling is
//! O(coarse voxels) and produces the same *kind* of data the pyramid stores.
//! The production path (derive on save via `dc_core::lod`, read cached LOD
//! here) is exercised and measured headlessly in `--bench-storage`; the seam
//! and streaming behavior proven here are independent of where the coarse
//! voxels came from.
//!
//! **Seam handling (documented tradeoff — overlap + downward bias):** the
//! LOD-1 ring starts [`FAR_OVERLAP_M`] inside the full-detail radius, so the
//! band is covered by both meshes and there is never a sky-gap at the
//! boundary. Far meshes are biased down by half a coarse voxel so coarse
//! geometry tends to poke *under* the full-detail surface instead of through
//! it. Cost: up to half a coarse voxel of "sunken" far terrain and occasional
//! coarse corners showing through fine terrain in the overlap band; at 100+ m
//! that reads as terrain noise. Skirts/stitching are the polished alternative
//! and out of spike scope. Between adjacent LOD rings there is no overlap:
//! both rings sample the same generator, so silhouettes match to within one
//! coarse voxel and the residual cracks at ring boundaries are accepted.
//!
//! Because the Y bias is a whole number of finer voxels, overlapping levels
//! produce exactly coplanar faces — and Y bias does nothing for vertical
//! faces — which z-fight (the two tessellations of one plane interpolate
//! depth with different per-pixel float error; draw order can't fix that).
//! Each far chunk is therefore pushed [`DEPTH_PUSH_FRAC`] of its coarse voxel
//! *away from the viewer along the view direction*, separating every face
//! orientation in real depth. Adjacent same-level chunks shift by
//! near-identical vectors, so no visible gaps open; deeper levels push
//! further, so ring pairs separate too.

use std::collections::HashMap;

use bevy::prelude::*;
use dc_core::{Block, CHUNK_SIZE, ChunkPos, VoxelScale};
use glam::DVec3;

use crate::app::{
    CurrentScale, FloatingOrigin, Fullbright, FullbrightMaterialHandle, TerrainMaterialHandle,
    to_render,
};
use crate::authority::Authority;
use crate::meshing::{MeshData, block_layer, face_color, mesh_chunk};
use crate::player::Player;
use crate::streaming::to_bevy_mesh;
use crate::worldgen::TerrainGen;

/// The legacy S1 [`TerrainGen`] the far-mesh rings still sample from — owned
/// here and NOWHERE else in the near-field pipeline.
///
/// **KNOWN DEFECT (ROADMAP Observed, journal/0017).** Under the worldgen
/// authority (surface ~1000 m) this generator's surface sits at ~8 m, so the
/// far rings paint a *phantom old world* ~1 km below the real terrain that
/// dissolves as the player approaches and near-field chunks stream in. Sourcing
/// far rings from a coarse worldgen summary is renderer-scale work (a persisted
/// LOD/summary pyramid — see the far-field Observed items); until then the far
/// field is honestly, loudly wrong here rather than silently wrong inside a
/// shared cache. The S1-fallback sweep removed every *near-field* consumer of
/// this generator; this resource is the single sanctioned survivor.
#[derive(Resource)]
pub struct FarFieldTerrain(pub(crate) TerrainGen);

impl FarFieldTerrain {
    pub fn new(seed: i32) -> Self {
        Self(TerrainGen::new(seed))
    }
}

/// Full-detail radius in meters (S1 shipped 72 m; S3 raises it and hangs the
/// far field beyond it).
pub const FULL_DETAIL_RADIUS_M: f64 = 128.0;
/// The LOD-1 ring starts this far *inside* the full-detail edge (the seam
/// overlap band; see module docs).
pub const FAR_OVERLAP_M: f64 = 16.0;
/// Outer edge of the far field.
pub const FAR_MAX_M: f64 = 1200.0;
/// Ring edges in meters: level L covers `[RING_EDGES_M[L-1], RING_EDGES_M[L])`
/// by chunk-center distance, L in 1..=4.
pub const RING_EDGES_M: [f64; 5] = [
    FULL_DETAIL_RADIUS_M - FAR_OVERLAP_M,
    256.0,
    512.0,
    1024.0,
    FAR_MAX_M,
];
/// Far chunks generated + meshed per frame (each costs ~2.5 ms of main-thread
/// time; the full 1 km field fills in a few seconds of streaming).
const FAR_BUDGET_PER_FRAME: usize = 3;
/// Hysteresis: a far chunk is only despawned once its center is this far
/// outside its ring, so ring membership doesn't thrash while walking.
const FAR_UNLOAD_SLACK_M: f64 = 48.0;
/// Fraction of a level's coarse voxel size that its chunks are pushed away
/// from the viewer to separate coplanar faces in depth (see module docs).
/// L1 ≈ 0.27 m at ≥112 m distance — angularly invisible, decisively beyond
/// f32 depth interpolation error.
const DEPTH_PUSH_FRAC: f64 = 0.15;

/// The anti-z-fight translation for a far chunk: origin-relative position plus
/// the half-voxel downward seam bias plus the radial depth push.
fn far_transform_translation(
    base: VoxelScale,
    level: u8,
    pos: ChunkPos,
    viewer_m: DVec3,
    origin_m: DVec3,
) -> Vec3 {
    let vs = coarse_scale(base, level).voxel_size_m();
    let (mx, my, mz) = pos.min_voxel();
    let min_m = DVec3::new(mx as f64, my as f64, mz as f64) * vs;
    let bias = DVec3::new(0.0, -0.5 * vs, 0.0);
    let center = far_chunk_center_m(base, level, pos);
    let away = (center - viewer_m).normalize_or_zero() * (DEPTH_PUSH_FRAC * vs);
    to_render(min_m + bias + away - origin_m)
}

/// LOD level whose ring contains a chunk-center distance, `None` for the
/// full-detail region and beyond the far field.
pub fn level_for_distance(d: f64) -> Option<u8> {
    if !(RING_EDGES_M[0]..RING_EDGES_M[4]).contains(&d) {
        return None;
    }
    (1..=4u8).find(|level| d < RING_EDGES_M[usize::from(*level)])
}

/// The level-L sampling scale: one voxel is `2^L` base voxels.
pub fn coarse_scale(base: VoxelScale, level: u8) -> VoxelScale {
    VoxelScale::from_voxel_size_m(base.voxel_size_m() * f64::from(1u32 << level))
}

/// World-space center (meters) of a level-L chunk position.
pub fn far_chunk_center_m(base: VoxelScale, level: u8, pos: ChunkPos) -> DVec3 {
    let chunk_m = coarse_scale(base, level).voxels_to_meters(f64::from(CHUNK_SIZE));
    DVec3::new(
        (f64::from(pos.x) + 0.5) * chunk_m,
        (f64::from(pos.y) + 0.5) * chunk_m,
        (f64::from(pos.z) + 0.5) * chunk_m,
    )
}

/// All level-L chunk positions wanted around a viewer (3D spherical ring —
/// cubic chunks: the far field extends down a chasm exactly like it extends
/// north). Pure so the headless bench measures the same set the app streams.
pub fn wanted_far_positions(base: VoxelScale, viewer_m: DVec3, level: u8) -> Vec<ChunkPos> {
    let chunk_m = coarse_scale(base, level).voxels_to_meters(f64::from(CHUNK_SIZE));
    let outer = RING_EDGES_M[usize::from(level)];
    let center_chunk = ChunkPos::new(
        (viewer_m.x / chunk_m).floor() as i32,
        (viewer_m.y / chunk_m).floor() as i32,
        (viewer_m.z / chunk_m).floor() as i32,
    );
    let r = (outer / chunk_m).ceil() as i32 + 1;
    let mut out = Vec::new();
    for dy in -r..=r {
        for dz in -r..=r {
            for dx in -r..=r {
                let pos = ChunkPos::new(
                    center_chunk.x + dx,
                    center_chunk.y + dy,
                    center_chunk.z + dz,
                );
                let d = (far_chunk_center_m(base, level, pos) - viewer_m).length();
                if level_for_distance(d) == Some(level) {
                    out.push(pos);
                }
            }
        }
    }
    out
}

/// Marks a far-mesh entity; transform recomputed from f64 every frame by
/// [`position_far_chunks`].
#[derive(Component)]
pub struct FarChunkEntity {
    pub level: u8,
    pub pos: ChunkPos,
}

/// All currently loaded far chunks. Voxel data is NOT retained — far chunks
/// exist only as meshes (`None` = meshed to nothing, e.g. sky).
#[derive(Resource, Default)]
pub struct FarChunkMap {
    pub loaded: HashMap<(u8, ChunkPos), Option<Entity>>,
}

// Bevy systems take their inputs as parameters by design; splitting this one
// to appease the 7-argument lint would only obscure the data flow.
#[allow(clippy::too_many_arguments)]
pub fn stream_far_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    terrain: Res<FarFieldTerrain>,
    authority: Res<Authority>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<FarChunkMap>,
) {
    // Under the worldgen authority the horizon is the coarse-summary heightfield
    // (`stream_far_surface`), not this legacy S1 volumetric far mesh (which paints
    // a phantom old world ~1 km down — journal/0017/0022). Keep the S1 far mesh
    // only for the S1 authority (keys 3/4); tear ours down when it's inactive.
    if authority.far_field_is_worldgen() {
        if !map.loaded.is_empty() {
            for (_, entity) in map.loaded.drain() {
                if let Some(entity) = entity {
                    commands.entity(entity).despawn();
                }
            }
        }
        return;
    }

    let base = scale.scale;

    // Scale switched (keys 2/3/4): far meshes are per-scale, rebuild them.
    if scale.is_changed() && !map.loaded.is_empty() {
        for (_, entity) in map.loaded.drain() {
            if let Some(entity) = entity {
                commands.entity(entity).despawn();
            }
        }
    }

    // Unload chunks that left their ring (with hysteresis).
    let to_unload: Vec<(u8, ChunkPos)> = map
        .loaded
        .keys()
        .filter(|(level, pos)| {
            let d = (far_chunk_center_m(base, *level, *pos) - player.pos_m).length();
            let inner = RING_EDGES_M[usize::from(*level) - 1];
            let outer = RING_EDGES_M[usize::from(*level)];
            d < inner - FAR_UNLOAD_SLACK_M || d > outer + FAR_UNLOAD_SLACK_M
        })
        .copied()
        .collect();
    for key in to_unload {
        if let Some(Some(entity)) = map.loaded.remove(&key) {
            commands.entity(entity).despawn();
        }
    }

    // Collect missing positions across all rings, nearest first.
    let mut missing: Vec<(u64, u8, ChunkPos)> = Vec::new();
    for level in 1..=4u8 {
        for pos in wanted_far_positions(base, player.pos_m, level) {
            if !map.loaded.contains_key(&(level, pos)) {
                let d = (far_chunk_center_m(base, level, pos) - player.pos_m).length();
                missing.push(((d * 1000.0) as u64, level, pos));
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _, _)| *d);

    for (_, level, pos) in missing.into_iter().take(FAR_BUDGET_PER_FRAME) {
        let cscale = coarse_scale(base, level);
        let chunk = terrain.0.generate_chunk(cscale, pos);
        // Faces cull against same-level generator samples, so a ring is
        // seamless internally; ring-to-ring boundaries are the accepted seam.
        let neighbor_solid =
            |x: i64, y: i64, z: i64| terrain.0.block_at(cscale, x, y, z).is_solid();
        let mesh_data = mesh_chunk(
            &chunk,
            pos,
            cscale.voxel_size_m() as f32,
            &neighbor_solid,
            None,
        );
        let entity = if mesh_data.is_empty() {
            None
        } else {
            // Spawn already positioned (same reasoning as streaming.rs): a
            // default transform renders one frame at the floating origin.
            let transform = Transform::from_translation(far_transform_translation(
                base,
                level,
                pos,
                player.pos_m,
                origin.0,
            ));
            let mut ent = commands.spawn((
                Mesh3d(meshes.add(to_bevy_mesh(mesh_data))),
                FarChunkEntity { level, pos },
                transform,
            ));
            if fullbright.0 {
                ent.insert(MeshMaterial3d(material.0.clone()));
            } else {
                ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
            }
            Some(ent.id())
        };
        map.loaded.insert((level, pos), entity);
    }
}

/// Place far chunks relative to the floating origin from f64, every frame,
/// with the seam bias and anti-z-fight depth push (module docs).
pub fn position_far_chunks(
    origin: Res<FloatingOrigin>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    mut chunks: Query<(&FarChunkEntity, &mut Transform)>,
) {
    for (far, mut transform) in &mut chunks {
        transform.translation =
            far_transform_translation(scale.scale, far.level, far.pos, player.pos_m, origin.0);
    }
}

// ===========================================================================
// The worldgen horizon: a coarse-summary heightfield far field (journal/0022).
//
// Under the worldgen authority the far rings are NOT a second terrain: they are
// the authority's OWN surface, sampled coarsely. Each far *tile* is a 32×32-cell
// patch of coarse columns whose corner heights come from
// `Authority::worldgen_coarse_surface` (the same elevation lattice + river
// carving the near ground collapses from). We emit only the **top surface** —
// no sealed cave interiors, no undersides — so a tile is 2·32² triangles instead
// of a volumetric coarse chunk's shell, and the silhouette is the real terrain's,
// ~1 km up, not the S1 phantom ~1 km down. Continuity with the near field is by
// construction: where a far corner lands on a near column the heights are equal
// (journal/0022); the sheet is sunk half a coarse voxel so the near volumetric
// terrain wins the overlap band, and pushed slightly away from the viewer so
// adjacent LOD rings separate in depth (same anti-z-fight reasoning as the S1
// far mesh above). The tiles are a 2D annulus at the surface, not a 3D shell:
// looking up from deep in a chasm loses the far field (accepted — this milestone
// builds the horizon a walker sees from the ground; the volumetric-down case is
// a noted follow-on).
// ===========================================================================

/// Far surface tiles generated + meshed per frame (each samples 33² coarse
/// columns — a warm pyramid makes that ~1 ms). The full horizon fills in a
/// second or two of streaming, same as the S1 far mesh.
const FAR_SURFACE_BUDGET_PER_FRAME: usize = 2;
/// Fraction of a coarse voxel the far sheet is sunk *below* the true surface so
/// the near-field volumetric terrain occludes it in the overlap band (and so
/// deeper, larger-voxel rings sit under nearer ones at ring seams).
const SURFACE_SINK_FRAC: f64 = 0.5;

/// All loaded far surface tiles (worldgen authority). Keyed by (level, tx, tz);
/// tiles are a 2D annulus, so there is no chunk-Y in the key.
#[derive(Resource, Default)]
pub struct FarSurfaceMap {
    pub loaded: HashMap<(u8, i32, i32), Option<Entity>>,
}

/// A far surface tile entity; its transform is recomputed from f64 every frame
/// by [`position_far_tiles`].
#[derive(Component)]
pub struct FarTileEntity {
    pub level: u8,
    pub tx: i32,
    pub tz: i32,
    /// Tile Y origin (meters) = its minimum corner height, so the mesh's own f32
    /// positions stay small and the absolute height rides in the transform (the
    /// floating-origin discipline, same as near chunks).
    pub y_ref_m: f64,
}

/// Horizontal edge length (meters) of a level-L surface tile (32 coarse voxels).
fn far_tile_m(base: VoxelScale, level: u8) -> f64 {
    coarse_scale(base, level).voxels_to_meters(f64::from(CHUNK_SIZE))
}

/// Horizontal (x,z) distance from the viewer to a tile's center — the far
/// surface is a 2D annulus at the terrain surface, so ring membership is by
/// ground-plane distance, not the 3D chunk-center distance the S1 shell uses.
fn far_tile_center_dist(base: VoxelScale, level: u8, tx: i32, tz: i32, viewer_m: DVec3) -> f64 {
    let tile_m = far_tile_m(base, level);
    let cx = (f64::from(tx) + 0.5) * tile_m;
    let cz = (f64::from(tz) + 0.5) * tile_m;
    ((cx - viewer_m.x).powi(2) + (cz - viewer_m.z).powi(2)).sqrt()
}

/// All level-L surface tiles wanted around a viewer (2D ring by horizontal
/// distance). Pure, so a headless bench measures the same set the app streams.
pub fn wanted_far_tiles(base: VoxelScale, viewer_m: DVec3, level: u8) -> Vec<(i32, i32)> {
    let tile_m = far_tile_m(base, level);
    let outer = RING_EDGES_M[usize::from(level)];
    let center_tx = (viewer_m.x / tile_m).floor() as i32;
    let center_tz = (viewer_m.z / tile_m).floor() as i32;
    let r = (outer / tile_m).ceil() as i32 + 1;
    let mut out = Vec::new();
    for dz in -r..=r {
        for dx in -r..=r {
            let (tx, tz) = (center_tx + dx, center_tz + dz);
            if level_for_distance(far_tile_center_dist(base, level, tx, tz, viewer_m))
                == Some(level)
            {
                out.push((tx, tz));
            }
        }
    }
    out
}

/// The far tile's transform: tile origin (meters) minus the floating origin,
/// plus the downward surface sink and the radial depth push (see section docs).
fn far_tile_translation(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    y_ref_m: f64,
    viewer_m: DVec3,
    origin_m: DVec3,
) -> Vec3 {
    let cvs = coarse_scale(base, level).voxel_size_m();
    let tile_m = cvs * f64::from(CHUNK_SIZE);
    let min = DVec3::new(f64::from(tx) * tile_m, y_ref_m, f64::from(tz) * tile_m);
    let sink = DVec3::new(0.0, -SURFACE_SINK_FRAC * cvs, 0.0);
    let center = DVec3::new(min.x + tile_m * 0.5, y_ref_m, min.z + tile_m * 0.5);
    let away = (center - viewer_m).normalize_or_zero() * (DEPTH_PUSH_FRAC * cvs);
    to_render(min + sink + away - origin_m)
}

/// Number of corner samples per tile side (33: one more than the 32 cells).
const TILE_CORNERS: usize = CHUNK_SIZE as usize + 1;

/// Build one far tile's top-surface heightfield mesh from the coarse summary.
/// `sample(wx, wz)` returns (surface height in base voxels, surface block) at a
/// world base-voxel column. Shared-vertex quad sheet (1089 verts, 2048 tris),
/// smooth normals from the height gradient, world-anchored UVs at base-voxel
/// density (so tiling matches the near field), one block atlas layer per vertex.
/// Returns the mesh and the tile's y-reference (min corner height, meters).
fn build_far_tile_mesh(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    sample: &dyn Fn(i64, i64) -> (i32, Block),
) -> (MeshData, f64) {
    const N: usize = TILE_CORNERS;
    let stride = 1i64 << level; // base voxels per coarse voxel
    let base_vs = base.voxel_size_m();
    let cvs = base_vs * stride as f64; // coarse voxel size (horizontal spacing)
    let corner_world = |i: usize, j: usize| -> (i64, i64) {
        (
            (i64::from(tx) * i64::from(CHUNK_SIZE) + i as i64) * stride,
            (i64::from(tz) * i64::from(CHUNK_SIZE) + j as i64) * stride,
        )
    };

    let mut hgt = [0i32; N * N];
    let mut blk = [Block::Stone; N * N];
    for j in 0..N {
        for i in 0..N {
            let (wx, wz) = corner_world(i, j);
            let (h, b) = sample(wx, wz);
            hgt[j * N + i] = h;
            blk[j * N + i] = b;
        }
    }
    let min_h = hgt.iter().copied().min().unwrap_or(0);
    let y_ref = f64::from(min_h) * base_vs;

    let mut mesh = MeshData::default();
    for j in 0..N {
        for i in 0..N {
            let h_m = f64::from(hgt[j * N + i]) * base_vs;
            mesh.positions.push([
                (i as f64 * cvs) as f32,
                (h_m - y_ref) as f32,
                (j as f64 * cvs) as f32,
            ]);
            // Smooth normal from the height gradient (central difference where
            // it has both neighbours, one-sided at the tile edge).
            let (il, ir) = (i.saturating_sub(1), (i + 1).min(N - 1));
            let (jl, jr) = (j.saturating_sub(1), (j + 1).min(N - 1));
            let span_x = (ir - il) as f64 * cvs;
            let span_z = (jr - jl) as f64 * cvs;
            let dhx = if span_x > 0.0 {
                f64::from(hgt[j * N + ir] - hgt[j * N + il]) * base_vs / span_x
            } else {
                0.0
            };
            let dhz = if span_z > 0.0 {
                f64::from(hgt[jr * N + i] - hgt[jl * N + i]) * base_vs / span_z
            } else {
                0.0
            };
            let nrm = DVec3::new(-dhx, 1.0, -dhz).normalize();
            mesh.normals
                .push([nrm.x as f32, nrm.y as f32, nrm.z as f32]);
            let block = blk[j * N + i];
            mesh.colors.push(face_color(block, 1));
            let (wx, wz) = corner_world(i, j);
            // World-anchored UV at base-voxel density (the near field's units):
            // a coarse quad spans `stride` tile units, so the texture repeats at
            // the same pitch as the near ground, not stretched one-per-cell.
            mesh.uvs.push([wx as f32, wz as f32]);
            mesh.mat_layers.push([block_layer(block), 0, 0, 0]);
            mesh.mat_weights.push([1.0, 0.0, 0.0, 0.0]);
        }
    }
    // Two triangles per cell, wound +Y-up (matches the near mesher's +Y face so
    // backface culling keeps the top visible).
    for j in 0..N - 1 {
        for i in 0..N - 1 {
            let v00 = (j * N + i) as u32;
            let v01 = ((j + 1) * N + i) as u32;
            let v11 = ((j + 1) * N + i + 1) as u32;
            let v10 = (j * N + i + 1) as u32;
            mesh.indices.extend([v00, v01, v11, v00, v11, v10]);
        }
    }
    (mesh, y_ref)
}

/// Stream the worldgen horizon: the coarse-summary surface heightfield rings.
/// Runs only under the worldgen authority; the S1 authority uses
/// [`stream_far_chunks`]. Budgeted/incremental like the S1 far mesh.
#[allow(clippy::too_many_arguments)]
pub fn stream_far_surface(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    authority: Res<Authority>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<FarSurfaceMap>,
) {
    // Only the worldgen authority has a coarse summary; tear our tiles down when
    // the S1 authority is active (keys 3/4).
    if !authority.far_field_is_worldgen() {
        if !map.loaded.is_empty() {
            for (_, e) in map.loaded.drain() {
                if let Some(e) = e {
                    commands.entity(e).despawn();
                }
            }
        }
        return;
    }

    let base = scale.scale;
    // Scale switched: tiles are per-scale, rebuild them.
    if scale.is_changed() && !map.loaded.is_empty() {
        for (_, e) in map.loaded.drain() {
            if let Some(e) = e {
                commands.entity(e).despawn();
            }
        }
    }

    // Unload tiles that left their ring (horizontal distance, hysteresis).
    let to_unload: Vec<(u8, i32, i32)> = map
        .loaded
        .keys()
        .filter(|(level, tx, tz)| {
            let d = far_tile_center_dist(base, *level, *tx, *tz, player.pos_m);
            let inner = RING_EDGES_M[usize::from(*level) - 1];
            let outer = RING_EDGES_M[usize::from(*level)];
            d < inner - FAR_UNLOAD_SLACK_M || d > outer + FAR_UNLOAD_SLACK_M
        })
        .copied()
        .collect();
    for key in to_unload {
        if let Some(Some(e)) = map.loaded.remove(&key) {
            commands.entity(e).despawn();
        }
    }

    // Collect missing tiles across all rings, nearest first.
    let mut missing: Vec<(u64, u8, i32, i32)> = Vec::new();
    for level in 1..=4u8 {
        for (tx, tz) in wanted_far_tiles(base, player.pos_m, level) {
            if !map.loaded.contains_key(&(level, tx, tz)) {
                let d = far_tile_center_dist(base, level, tx, tz, player.pos_m);
                missing.push(((d * 1000.0) as u64, level, tx, tz));
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _, _, _)| *d);

    for (_, level, tx, tz) in missing.into_iter().take(FAR_SURFACE_BUDGET_PER_FRAME) {
        let sample = |wx: i64, wz: i64| {
            authority
                .worldgen_coarse_surface(wx, wz)
                .expect("worldgen far-field summary under the worldgen authority")
        };
        let (mesh_data, y_ref) = build_far_tile_mesh(base, level, tx, tz, &sample);
        let transform = Transform::from_translation(far_tile_translation(
            base,
            level,
            tx,
            tz,
            y_ref,
            player.pos_m,
            origin.0,
        ));
        let mut ent = commands.spawn((
            Mesh3d(meshes.add(to_bevy_mesh(mesh_data))),
            FarTileEntity {
                level,
                tx,
                tz,
                y_ref_m: y_ref,
            },
            transform,
        ));
        if fullbright.0 {
            ent.insert(MeshMaterial3d(material.0.clone()));
        } else {
            ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
        }
        map.loaded.insert((level, tx, tz), Some(ent.id()));
    }
}

/// Place far surface tiles relative to the floating origin from f64, every
/// frame (same reasoning as [`position_far_chunks`]).
pub fn position_far_tiles(
    origin: Res<FloatingOrigin>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    mut tiles: Query<(&FarTileEntity, &mut Transform)>,
) {
    for (tile, mut transform) in &mut tiles {
        transform.translation = far_tile_translation(
            scale.scale,
            tile.level,
            tile.tx,
            tile.tz,
            tile.y_ref_m,
            player.pos_m,
            origin.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PLAYER_HEIGHT_M;

    #[test]
    fn ring_selection_by_distance() {
        assert_eq!(level_for_distance(0.0), None, "full detail");
        assert_eq!(level_for_distance(111.0), None, "still full detail");
        assert_eq!(level_for_distance(113.0), Some(1), "overlap band is LOD 1");
        assert_eq!(level_for_distance(255.0), Some(1));
        assert_eq!(level_for_distance(256.0), Some(2));
        assert_eq!(level_for_distance(511.0), Some(2));
        assert_eq!(level_for_distance(512.0), Some(3));
        assert_eq!(level_for_distance(1023.0), Some(3));
        assert_eq!(level_for_distance(1024.0), Some(4));
        assert_eq!(level_for_distance(1199.0), Some(4));
        assert_eq!(level_for_distance(1200.0), None, "beyond the far field");
    }

    #[test]
    fn coarse_scale_doubles_per_level() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        assert!((coarse_scale(base, 1).voxel_size_m() - 1.8).abs() < 1e-12);
        assert!((coarse_scale(base, 4).voxel_size_m() - 14.4).abs() < 1e-12);
    }

    #[test]
    fn wanted_positions_are_ring_shaped_and_deterministic() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let viewer = DVec3::new(10.0, 5.0, -20.0);
        let a = wanted_far_positions(base, viewer, 1);
        let b = wanted_far_positions(base, viewer, 1);
        assert_eq!(a, b, "deterministic");
        assert!(!a.is_empty());
        for pos in &a {
            let d = (far_chunk_center_m(base, 1, *pos) - viewer).length();
            assert!(
                (RING_EDGES_M[0]..RING_EDGES_M[1]).contains(&d),
                "chunk at distance {d} outside LOD-1 ring"
            );
        }
    }

    #[test]
    fn wanted_far_tiles_are_ring_shaped_and_deterministic() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let viewer = DVec3::new(30.0, 980.0, -15.0);
        let a = wanted_far_tiles(base, viewer, 2);
        let b = wanted_far_tiles(base, viewer, 2);
        assert_eq!(a, b, "deterministic");
        assert!(!a.is_empty());
        for &(tx, tz) in &a {
            let d = far_tile_center_dist(base, 2, tx, tz, viewer);
            assert!(
                (RING_EDGES_M[1]..RING_EDGES_M[2]).contains(&d),
                "tile at horizontal distance {d} outside LOD-2 ring"
            );
        }
    }

    #[test]
    fn far_tile_is_a_top_surface_sheet() {
        // A far tile is the TOP surface only — 32×32 cells × 2 triangles, no
        // sealed cave interiors or undersides (the S3 far-mesh waste is gone).
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let sample = |wx: i64, wz: i64| (((wx + wz).rem_euclid(7)) as i32, Block::Grass);
        let (mesh, _y) = build_far_tile_mesh(base, 1, 3, -2, &sample);
        assert_eq!(mesh.triangle_count(), 2 * 32 * 32);
        assert_eq!(mesh.positions.len(), TILE_CORNERS * TILE_CORNERS);
        assert_eq!(mesh.mat_layers.len(), mesh.positions.len());
        assert_eq!(mesh.uvs.len(), mesh.positions.len());
        // Every normal points up — a heightfield sheet, and the +Y winding keeps
        // the top visible under backface culling.
        for n in &mesh.normals {
            assert!(n[1] > 0.0, "far surface normal not up: {n:?}");
        }
        // Block-colored: one atlas layer at full weight, no splat mix.
        for w in &mesh.mat_weights {
            assert_eq!(*w, [1.0, 0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn far_tile_corner_y_equals_sampled_height() {
        // The mesh faithfully carries the summary: a corner's world Y (local +
        // y_ref) equals the sampled surface height at that world column.
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let base_vs = base.voxel_size_m();
        let h_at = |wx: i64, wz: i64| (wx.rem_euclid(5) + wz.rem_euclid(3)) as i32;
        let sample = |wx: i64, wz: i64| (h_at(wx, wz), Block::Stone);
        let (mesh, y_ref) = build_far_tile_mesh(base, 2, 1, -1, &sample);
        let stride = 1i64 << 2;
        let n = TILE_CORNERS;
        let (i, j) = (5usize, 7usize);
        let (wx, wz) = (
            (i64::from(1) * i64::from(CHUNK_SIZE) + i as i64) * stride,
            (i64::from(-1) * i64::from(CHUNK_SIZE) + j as i64) * stride,
        );
        let expect = f64::from(h_at(wx, wz)) * base_vs;
        let got = f64::from(mesh.positions[j * n + i][1]) + y_ref;
        assert!(
            (got - expect).abs() < 1e-3,
            "corner Y {got} != sampled {expect}"
        );
    }

    /// The milestone's continuity guarantee, measured: every far-horizon corner
    /// height equals the near ground's height at the same voxel — zero boundary
    /// mismatch by construction (journal/0022). The sub-coarse relief dropped
    /// *between* corners is the only residual, and it is bounded by the level's
    /// coarse voxel; here we assert the corners themselves agree exactly.
    #[test]
    fn far_horizon_heights_agree_with_the_near_ground() {
        use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let mut g = WorldGenerator::new(&pregen);
        let stride = 1i64 << 1; // level-1 far ring
        let mut max_mismatch = 0i32;
        for j in 0..i64::from(CHUNK_SIZE) + 1 {
            for i in 0..i64::from(CHUNK_SIZE) + 1 {
                let (wx, wz) = (i * stride, j * stride);
                let (far, _) = g.coarse_surface(wx, wz);
                let col = g.column_record(wx.div_euclid(32), wz.div_euclid(32));
                let (lx, lz) = (wx.rem_euclid(32) as usize, wz.rem_euclid(32) as usize);
                let near = col.heights[lz * 32 + lx];
                max_mismatch = max_mismatch.max((far - near).abs());
            }
        }
        assert_eq!(
            max_mismatch, 0,
            "far horizon corner height must equal the near ground exactly"
        );
    }
}
