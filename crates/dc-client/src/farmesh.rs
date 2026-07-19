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
use dc_core::{CHUNK_SIZE, ChunkPos, VoxelScale};
use glam::DVec3;

use crate::app::{
    ChunkMaterial, CurrentScale, FloatingOrigin, Fullbright, TerrainMaterialHandle, to_render,
};
use crate::meshing::mesh_chunk;
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
    material: Res<ChunkMaterial>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    terrain: Res<FarFieldTerrain>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<FarChunkMap>,
) {
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
}
