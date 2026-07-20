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
// The worldgen horizon: a VOXEL-STEPPED coarse-summary far field (FF2a,
// journal/0023 — replaces the smooth-TIN heightfield of journal/0022).
//
// Under the worldgen authority the far rings are NOT a second terrain: they are
// the authority's OWN surface, sampled coarsely. Each far *tile* is a 32×32-cell
// patch of coarse **columns** whose heights come from
// `Authority::worldgen_coarse_surface` (the same elevation lattice + river
// carving the near ground collapses from). Where journal/0022 drew a smooth
// interpolated sheet, FF2a speaks the voxel language (visuals.md § distance
// speaks the voxel language): each column's summary height is **quantized to the
// level's coarse-voxel step** and the tile meshes as stepped prisms — greedy-
// merged top faces at the quantized height + exposed vertical side faces between
// neighbour columns of differing height. Still a top surface only (no sealed
// cave interiors), so a mostly-flat tile is a handful of merged quads, not the
// old shell's thousands of cave triangles.
//
// **Three properties fall out of the voxelization (the FF2a wins):**
//  - *Near/far parity, no sink hack.* The quantized top FLOORS the sampled
//    surface to the coarse lattice ([`quantize_top`]), so a far column top is
//    always ≤ the near-field surface at a coinciding column — the near
//    volumetric terrain wins the overlap band with no downward "sink" bias. At a
//    stride-aligned column the far top equals the near voxel top exactly.
//  - *Crack class cured inherently.* Adjacent columns (within a tile AND across a
//    same-level tile boundary — both sample the identical shared column) share
//    face planes; the side face is emitted once, by the taller column only, so
//    same-level seams are watertight with no T-junctions. Only ring-to-ring
//    (differing stride) and near/far boundaries get a modest downward skirt.
//  - *No buried sheet.* Far columns the near volumetric field covers are CULLED
//    ([`near_covers`]), not lapped underneath — so excavating a near chunk never
//    exposes a phantom far floor. Seam redundancy is coverage, not buried
//    geometry (replaces journal/0022 walk-17's one-tile inner lap + half-voxel
//    sink).
//
// Adjacent LOD rings still overlap (the inter-ring lap of [`far_tile_in_ring`])
// and are pushed slightly away from the viewer so their coplanar faces separate
// in depth (corrections #1 — draw order can't fix coplanar z-fight; the radial
// push moves faces apart in real depth). The tiles are a 2-D annulus at the
// surface: looking up from deep in a chasm still loses the far field (accepted —
// FF2b's volumetric spans own the chasm case; the per-column payload is already
// a [`ColumnSpan`] so that extension needs no mesher rewrite).
// ===========================================================================

/// Far surface tiles generated + meshed per frame. The full horizon fills in a
/// second or two of streaming, same as the S1 far mesh.
const FAR_SURFACE_BUDGET_PER_FRAME: usize = 2;
/// Horizontal radius (m) within which the near volumetric field is treated as
/// covering the surface, so far columns there are CULLED rather than buried.
/// Equals the far-overlap inset of the full-detail radius: the near field
/// streams a [`FULL_DETAIL_RADIUS_M`] sphere, so culling far columns whose 3-D
/// distance to the viewer is under this inset leaves exactly the intended
/// ~[`FAR_OVERLAP_M`] occluded overlap band (far quantized below near, near
/// opaque on top) and NO deeper buried geometry — the coverage-logic replacement
/// for journal/0022 walk-17's buried inner lap.
const NEAR_COVER_R_M: f64 = FULL_DETAIL_RADIUS_M - FAR_OVERLAP_M;
/// Depth of the ring-transition / near-seam skirt, in this level's coarse
/// voxels. A stepped tile's *outer* boundary (where the next-coarser ring takes
/// over, or where the near field culls its columns) drops a short vertical apron
/// this deep so a stride mismatch at the transition can't open a pixel crack.
/// Same-level tile boundaries need no skirt (their steps are watertight).
const RING_SKIRT_COARSE_VOXELS: i64 = 2;

/// An extensible per-column far-field summary span (FF2a). Today a column is a
/// single solid span described by its quantized `top` (base voxels, N=2 base
/// scale) and the `block` at the surface. FF2b's coarse *volumetric* summaries
/// extend a column to a STACK of these (top/bottom pairs with per-span material,
/// for overhangs and caves) — the stepped mesher already reasons in "a column is
/// a set of spans with tops and sides", so that extension needs no rewrite here.
///
/// **Persistence-shaped** (coordinator amendment 3): this is the payload the
/// decided follow-on will store beside S3 region files (voxy-dh-recon transfer
/// map). It is plain fixed-layout POD — no `Option`, no `skip_serializing_if` —
/// so a positional format (postcard, corrections #3) or the recon doc's compact
/// 8-byte packed encoding can serialize it verbatim. A tile's spans are derived
/// per-tile in isolation ([`tile_column_spans`]) and re-derivable on demand, so
/// edit-driven invalidation is a drop-in, not a rewrite (amendment 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnSpan {
    /// Quantized top, base voxels — a coarse-voxel boundary at or below the
    /// sampled surface (see [`quantize_top`]).
    pub top: i32,
    /// The surface block (drives the atlas layer + fullbright vertex color).
    pub block: Block,
}

/// Base voxels per coarse voxel at level L (the sampling / quantization stride).
#[inline]
fn level_stride(level: u8) -> i64 {
    1i64 << level
}

/// Quantize a base-voxel surface height to the level's coarse-voxel lattice,
/// FLOORING to the coarse boundary at or below the surface. Flooring is the
/// mechanism that retires journal/0022's half-voxel sink: the stepped column top
/// is always ≤ the true (hence the near-field) surface, so the opaque near
/// terrain wins the overlap band with no bias hack, and a stride-aligned column
/// matches the near voxel top exactly (quantized-exact parity).
#[inline]
fn quantize_top(h: i32, stride: i64) -> i32 {
    let s = stride as i32;
    h.div_euclid(s) * s
}

/// Whether the near volumetric field covers a far column at world `(wx_m, wz_m)`
/// whose stepped top is at `top_m` — a pure function of the viewer pose, so a
/// tile's culled set depends only on the viewer and not on near-chunk streaming
/// progress (deterministic, unit-testable). Altitude-aware: flying far above the
/// surface grows the vertical term, shrinking the covered disc to nothing, so the
/// horizon is never culled when viewed from the air (the key-2 high vantage).
fn near_covers(wx_m: f64, wz_m: f64, top_m: f64, viewer_m: DVec3) -> bool {
    let dx = wx_m - viewer_m.x;
    let dz = wz_m - viewer_m.z;
    let dy = top_m - viewer_m.y;
    dx * dx + dy * dy + dz * dz < NEAR_COVER_R_M * NEAR_COVER_R_M
}

/// A loaded far tile's bookkeeping: the mesh entity (`None` = the tile meshed to
/// nothing — e.g. every column culled by near coverage) and, for tiles near the
/// near/far seam, the viewer near-chunk the tile's column-cull was computed for.
/// `Some(chunk)` marks a **seam tile** whose culled column set depends on the
/// viewer, so it is rebuilt when the viewer crosses into a new near-chunk (the
/// coverage boundary moved); `None` marks a stable tile with no cullable columns
/// that never rebuilds for viewer motion.
pub struct LoadedFarTile {
    pub entity: Option<Entity>,
    pub cull_chunk: Option<(i64, i64, i64)>,
}

/// All loaded far surface tiles (worldgen authority). Keyed by (level, tx, tz);
/// tiles are a 2D annulus, so there is no chunk-Y in the key.
#[derive(Resource, Default)]
pub struct FarSurfaceMap {
    pub loaded: HashMap<(u8, i32, i32), LoadedFarTile>,
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

/// Whether a level-L tile at horizontal `dist` belongs to level L's ring —
/// with a one-tile **inner overlap** that laps this ring under the next-finer
/// one (or, for L1, under the near volumetric field).
///
/// **Why the overlap (journal/0022 walk 17 — the sky holes).** Far tiles tile
/// the ground plane *without overlap*, so a point lies in exactly ONE tile per
/// level. Ring membership is by tile-*center* distance, so at an inter-ring
/// boundary R a point can land in a tile that both levels reject at once: the
/// finer tile whose center sits just *past* R (excluded from the finer ring by
/// its outer edge) and the coarser tile whose center sits just *short* of R
/// (excluded from the coarser ring by its inner edge). With no third tile to
/// cover it (tiling is a partition), that whole cell is a fixed-position sky
/// hole. A boundary point's containing coarser tile has its center within one
/// coarse half-diagonal (< one tile) of R, so lapping the coarser ring exactly
/// one of its own tiles inward guarantees that tile is in-band, closing the
/// seam. The overlapping coarse cells sit under the finer ring; FF2a floor-
/// quantizes them below the finer ring's surface (no sink needed) and the radial
/// depth push separates the coplanar faces. For L1 the inner lap makes tiles
/// *present* across the near/far band, but their columns under the near field are
/// CULLED by coverage ([`near_covers`]) rather than buried — the FF2a
/// replacement for journal/0022's buried lap.
fn far_tile_in_ring(base: VoxelScale, level: u8, dist: f64) -> bool {
    let inner = RING_EDGES_M[usize::from(level) - 1] - far_tile_m(base, level);
    let outer = RING_EDGES_M[usize::from(level)];
    (inner..outer).contains(&dist)
}

/// All level-L surface tiles wanted around a viewer (2D ring by horizontal
/// distance, with the one-tile inner overlap of [`far_tile_in_ring`]). Pure, so
/// a headless bench measures the same set the app streams.
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
            if far_tile_in_ring(
                base,
                level,
                far_tile_center_dist(base, level, tx, tz, viewer_m),
            ) {
                out.push((tx, tz));
            }
        }
    }
    out
}

/// The far tile's transform: tile origin (meters) minus the floating origin,
/// plus the radial depth push. **No sink** — the stepped columns are FLOOR-
/// quantized ([`quantize_top`]) so a far top already sits at or below the near
/// surface; the near volumetric terrain wins the overlap band with no downward
/// bias. The radial push survives (corrections #1): overlapping LOD rings still
/// present coplanar faces where their quantized tops coincide, and draw order
/// cannot resolve coplanar z-fight — only a real depth offset can. Each level is
/// pushed `DEPTH_PUSH_FRAC` of its coarse voxel away from the viewer, so deeper
/// (coarser) rings sit behind nearer ones in the lap band.
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
    let center = DVec3::new(min.x + tile_m * 0.5, y_ref_m, min.z + tile_m * 0.5);
    let away = (center - viewer_m).normalize_or_zero() * (DEPTH_PUSH_FRAC * cvs);
    to_render(min + away - origin_m)
}

/// Interior cell count per tile side (32 coarse columns).
const TILE_CELLS: usize = CHUNK_SIZE as usize;

/// Sample a tile's extended column grid: quantized [`ColumnSpan`]s and near-cover
/// cull flags over indices −1..=`TILE_CELLS` on each axis (an (N+2)² grid). The
/// interior 0..`TILE_CELLS` is what the tile emits; the −1 / `TILE_CELLS` ring is
/// neighbour data for watertight boundary side faces (a same-level neighbour tile
/// samples the shared column identically, so the step matches). `sample(wx, wz)`
/// returns the summary (surface height in base voxels, surface block) at a world
/// base-voxel column; every sampled column is a real near column (stride-aligned
/// base voxel), so quantized tops agree with the near ground by construction.
fn tile_column_spans(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    viewer_m: DVec3,
    sample: &dyn Fn(i64, i64) -> (i32, Block),
) -> (Vec<ColumnSpan>, Vec<bool>) {
    let stride = level_stride(level);
    let base_vs = base.voxel_size_m();
    let m = TILE_CELLS + 2;
    let mut spans = vec![
        ColumnSpan {
            top: 0,
            block: Block::Stone,
        };
        m * m
    ];
    let mut culled = vec![false; m * m];
    for gj in -1..=TILE_CELLS as i64 {
        for gi in -1..=TILE_CELLS as i64 {
            let wx = (i64::from(tx) * TILE_CELLS as i64 + gi) * stride;
            let wz = (i64::from(tz) * TILE_CELLS as i64 + gj) * stride;
            let (h, block) = sample(wx, wz);
            let top = quantize_top(h, stride);
            let k = (gj + 1) as usize * m + (gi + 1) as usize;
            spans[k] = ColumnSpan { top, block };
            culled[k] = near_covers(
                wx as f64 * base_vs,
                wz as f64 * base_vs,
                f64::from(top) * base_vs,
                viewer_m,
            );
        }
    }
    (spans, culled)
}

/// Build one far tile's VOXEL-STEPPED mesh (FF2a) — a **pure function** of plain
/// data: the extended quantized column `spans` (an (N+2)² grid: interior 0..N
/// plus a one-cell neighbour ring), the near-coverage `culled` mask over the same
/// grid, and `ring_edges` (per tile side: does it face a coarser ring / world
/// rim?). No ECS/world/authority read happens here — coverage is passed in as
/// data (coordinator amendment 1), so async far-meshing (a filed follow-on) is a
/// drop-in and the mesher is trivially testable.
///
/// Emits greedy-merged top faces at each column's quantized height, exposed
/// vertical side faces between neighbour columns of differing height (emitted
/// once, by the taller column, so same-level tile seams are watertight and never
/// double-wall), and a modest downward skirt where a coarser ring takes over or
/// the near field culls the column. Returns the mesh and the tile's y-reference
/// (min emitted top, meters) — positions are tile-local so the absolute height
/// rides in the transform (floating-origin discipline). Order of `ring_edges`
/// matches the side-face direction order: +X, −X, +Z, −Z.
fn build_far_tile_mesh(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    spans: &[ColumnSpan],
    culled: &[bool],
    ring_edges: [bool; 4],
) -> (MeshData, f64) {
    let stride = level_stride(level);
    let base_vs = base.voxel_size_m();
    let n = TILE_CELLS;
    let m = n + 2;
    debug_assert_eq!(spans.len(), m * m);
    debug_assert_eq!(culled.len(), m * m);
    let idx = |i: i64, j: i64| -> usize { (j + 1) as usize * m + (i + 1) as usize };

    let ox = i64::from(tx) * n as i64 * stride;
    let oz = i64::from(tz) * n as i64 * stride;

    // y_ref = the lowest emitted (unculled) top. If every column is culled by
    // near coverage, the tile draws nothing.
    let mut min_top: Option<i32> = None;
    for j in 0..n as i64 {
        for i in 0..n as i64 {
            if !culled[idx(i, j)] {
                let t = spans[idx(i, j)].top;
                min_top = Some(min_top.map_or(t, |mt| mt.min(t)));
            }
        }
    }
    let Some(min_top) = min_top else {
        return (MeshData::default(), 0.0);
    };
    let y_ref = f64::from(min_top) * base_vs;

    let mut mesh = MeshData::default();
    // Push one quad (4 world-base-voxel corners, ccw seen from outside) with the
    // block's atlas layer / fullbright color and a world-anchored UV.
    let mut push_quad = |corners: [[f64; 3]; 4], normal: [i64; 3], block: Block| {
        let b = mesh.positions.len() as u32;
        let nrm = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
        let color = face_color(block, normal[1]);
        let layer = block_layer(block);
        for c in corners {
            let (fx, fy, fz) = (c[0], c[1], c[2]);
            mesh.positions.push([
                ((fx - ox as f64) * base_vs) as f32,
                (fy * base_vs - y_ref) as f32,
                ((fz - oz as f64) * base_vs) as f32,
            ]);
            mesh.normals.push(nrm);
            mesh.colors.push(color);
            // World-anchored UV in base-voxel units: the two in-plane world axes
            // for this face orientation, so the texture tiles at the near-ground
            // pitch (a coarse quad spans `stride` tile units).
            let uv = if normal[1] != 0 {
                [fx as f32, fz as f32]
            } else if normal[0] != 0 {
                [fz as f32, fy as f32]
            } else {
                [fx as f32, fy as f32]
            };
            mesh.uvs.push(uv);
            mesh.mat_layers.push([layer, 0, 0, 0]);
            mesh.mat_weights.push([1.0, 0.0, 0.0, 0.0]);
        }
        mesh.indices.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
    };

    // --- Greedy-merged top faces -------------------------------------------
    let mut consumed = vec![false; n * n];
    for j0 in 0..n {
        for i0 in 0..n {
            if consumed[j0 * n + i0] || culled[idx(i0 as i64, j0 as i64)] {
                continue;
            }
            let span0 = spans[idx(i0 as i64, j0 as i64)];
            let mut w = 1;
            while i0 + w < n
                && !consumed[j0 * n + i0 + w]
                && !culled[idx((i0 + w) as i64, j0 as i64)]
                && spans[idx((i0 + w) as i64, j0 as i64)] == span0
            {
                w += 1;
            }
            let mut h = 1;
            'grow: while j0 + h < n {
                for k in 0..w {
                    let (ii, jj) = ((i0 + k) as i64, (j0 + h) as i64);
                    if consumed[(j0 + h) * n + i0 + k]
                        || culled[idx(ii, jj)]
                        || spans[idx(ii, jj)] != span0
                    {
                        break 'grow;
                    }
                }
                h += 1;
            }
            for jj in j0..j0 + h {
                for ii in i0..i0 + w {
                    consumed[jj * n + ii] = true;
                }
            }
            let (x0, x1) = (ox + i0 as i64 * stride, ox + (i0 + w) as i64 * stride);
            let (z0, z1) = (oz + j0 as i64 * stride, oz + (j0 + h) as i64 * stride);
            let ty = f64::from(span0.top);
            // +Y top, ccw seen from above (matches the near mesher's +Y winding).
            push_quad(
                [
                    [x0 as f64, ty, z0 as f64],
                    [x0 as f64, ty, z1 as f64],
                    [x1 as f64, ty, z1 as f64],
                    [x1 as f64, ty, z0 as f64],
                ],
                [0, 1, 0],
                span0.block,
            );
        }
    }

    // --- Side faces + skirts ------------------------------------------------
    let skirt = (RING_SKIRT_COARSE_VOXELS * stride) as i32;
    // The four neighbour directions, tagged with their `ring_edges` index (+X,
    // −X, +Z, −Z): a cell on that tile edge whose `ring_edges` flag is set faces
    // a coarser ring / the world rim and gets a downward skirt.
    let dirs: [(i64, i64, [i64; 3], usize); 4] = [
        (1, 0, [1, 0, 0], 0),
        (-1, 0, [-1, 0, 0], 1),
        (0, 1, [0, 0, 1], 2),
        (0, -1, [0, 0, -1], 3),
    ];
    for j in 0..n as i64 {
        for i in 0..n as i64 {
            if culled[idx(i, j)] {
                continue;
            }
            let top = spans[idx(i, j)].top;
            let block = spans[idx(i, j)].block;
            for (di, dj, normal, edge_ix) in dirs {
                let (ni, nj) = (i + di, j + dj);
                let neighbor_top = spans[idx(ni, nj)].top;
                let neighbor_culled = culled[idx(ni, nj)];
                let out_of_tile = !(0..n as i64).contains(&ni) || !(0..n as i64).contains(&nj);
                let ring_edge = out_of_tile && ring_edges[edge_ix];

                let bottom = if neighbor_culled {
                    // Near/far seam: the near field covers the neighbour, so drop
                    // a skirt to hide the handoff crack (never step *up* to it).
                    top - skirt
                } else if ring_edge {
                    // Ring/world outer boundary: down to the neighbour, then a
                    // skirt below to cover the coarser ring's differing stride.
                    neighbor_top.min(top) - skirt
                } else if neighbor_top < top {
                    // Ordinary exposed step: the taller column walls down to the
                    // shorter neighbour, emitted once (only the taller side sees
                    // neighbour < top) — so same-level seams never double-wall.
                    neighbor_top
                } else {
                    continue;
                };
                if bottom >= top {
                    continue;
                }

                let (cx0, cx1) = ((ox + i * stride) as f64, (ox + (i + 1) * stride) as f64);
                let (cz0, cz1) = ((oz + j * stride) as f64, (oz + (j + 1) * stride) as f64);
                let (ylo, yhi) = (f64::from(bottom), f64::from(top));
                let corners = match normal {
                    [1, 0, 0] => [
                        [cx1, ylo, cz0],
                        [cx1, yhi, cz0],
                        [cx1, yhi, cz1],
                        [cx1, ylo, cz1],
                    ],
                    [-1, 0, 0] => [
                        [cx0, ylo, cz1],
                        [cx0, yhi, cz1],
                        [cx0, yhi, cz0],
                        [cx0, ylo, cz0],
                    ],
                    [0, 0, 1] => [
                        [cx1, ylo, cz1],
                        [cx1, yhi, cz1],
                        [cx0, yhi, cz1],
                        [cx0, ylo, cz1],
                    ],
                    _ => [
                        [cx0, ylo, cz0],
                        [cx0, yhi, cz0],
                        [cx1, yhi, cz0],
                        [cx1, ylo, cz0],
                    ],
                };
                push_quad(corners, normal, block);
            }
        }
    }

    (mesh, y_ref)
}

/// The viewer's near-field chunk (base-voxel chunk coords) — the granularity at
/// which the coverage cull is refreshed. A seam tile rebuilt for one near-chunk
/// stays valid until the viewer crosses into the next (~one near chunk of
/// motion), keeping the cull boundary fresh without per-frame churn.
fn viewer_near_chunk(base: VoxelScale, viewer: DVec3) -> (i64, i64, i64) {
    (
        base.voxel_at(viewer.x).div_euclid(i64::from(CHUNK_SIZE)),
        base.voxel_at(viewer.y).div_euclid(i64::from(CHUNK_SIZE)),
        base.voxel_at(viewer.z).div_euclid(i64::from(CHUNK_SIZE)),
    )
}

/// Whether a tile is close enough to the viewer that some of its columns could be
/// culled by near coverage — the band where the coverage cull is viewer-relative
/// and must be refreshed as the viewer moves.
fn tile_in_cull_band(base: VoxelScale, level: u8, tx: i32, tz: i32, viewer: DVec3) -> bool {
    far_tile_center_dist(base, level, tx, tz, viewer) < NEAR_COVER_R_M + far_tile_m(base, level)
}

/// Stream the worldgen horizon: the voxel-stepped coarse-summary rings (FF2a).
/// Runs only under the worldgen authority; the S1 authority uses
/// [`stream_far_chunks`]. Budgeted/incremental like the S1 far mesh. Beyond
/// filling newly-wanted tiles, it refreshes **seam tiles** in place (no blink)
/// when the viewer crosses into a new near-chunk, so the near-coverage column
/// cull tracks the moving near field (the buried-sheet fix).
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
            for (_, t) in map.loaded.drain() {
                if let Some(e) = t.entity {
                    commands.entity(e).despawn();
                }
            }
        }
        return;
    }

    let base = scale.scale;
    // Scale switched: tiles are per-scale, rebuild them.
    if scale.is_changed() && !map.loaded.is_empty() {
        for (_, t) in map.loaded.drain() {
            if let Some(e) = t.entity {
                commands.entity(e).despawn();
            }
        }
    }

    let viewer = player.pos_m;
    let cur_chunk = viewer_near_chunk(base, viewer);

    // Unload tiles that left their ring (horizontal distance, hysteresis). The
    // inner bound matches the one-tile ring lap of `far_tile_in_ring` so lapped
    // tiles don't load-then-immediately-unload (journal/0022 walk 17).
    let to_unload: Vec<(u8, i32, i32)> = map
        .loaded
        .keys()
        .filter(|(level, tx, tz)| {
            let d = far_tile_center_dist(base, *level, *tx, *tz, viewer);
            let inner = RING_EDGES_M[usize::from(*level) - 1] - far_tile_m(base, *level);
            let outer = RING_EDGES_M[usize::from(*level)];
            d < inner - FAR_UNLOAD_SLACK_M || d > outer + FAR_UNLOAD_SLACK_M
        })
        .copied()
        .collect();
    for key in to_unload {
        if let Some(t) = map.loaded.remove(&key)
            && let Some(e) = t.entity
        {
            commands.entity(e).despawn();
        }
    }

    // Newly-wanted (missing) tiles, nearest first — these appear the horizon.
    let mut missing: Vec<(u64, u8, i32, i32)> = Vec::new();
    for level in 1..=4u8 {
        for (tx, tz) in wanted_far_tiles(base, viewer, level) {
            if !map.loaded.contains_key(&(level, tx, tz)) {
                let d = far_tile_center_dist(base, level, tx, tz, viewer);
                missing.push(((d * 1000.0) as u64, level, tx, tz));
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _, _, _)| *d);

    // Seam tiles whose coverage cull is stale: either they were built for a
    // different near-chunk, or they are newly in the cull band and have never
    // been culled. Refreshed in place (no blink), nearest first, with leftover
    // budget after the missing fills — staleness in the occluded overlap band is
    // invisible, so the horizon (missing) takes priority.
    let mut stale: Vec<(u64, u8, i32, i32)> = Vec::new();
    for (&(level, tx, tz), t) in &map.loaded {
        let in_band = tile_in_cull_band(base, level, tx, tz, viewer);
        let needs = match t.cull_chunk {
            Some(c) => c != cur_chunk,
            None => in_band,
        };
        if needs {
            let d = far_tile_center_dist(base, level, tx, tz, viewer);
            stale.push(((d * 1000.0) as u64, level, tx, tz));
        }
    }
    stale.sort_unstable_by_key(|(d, _, _, _)| *d);

    let sample = |wx: i64, wz: i64| {
        authority
            .worldgen_coarse_surface(wx, wz)
            .expect("worldgen far-field summary under the worldgen authority")
    };
    // Derive the tile's summary + coverage inputs here (the impure boundary:
    // reads the authority + viewer), then mesh purely from that plain data.
    let ring_edges_of = |level: u8, tx: i32, tz: i32| -> [bool; 4] {
        let edge = |ntx: i32, ntz: i32| {
            !far_tile_in_ring(
                base,
                level,
                far_tile_center_dist(base, level, ntx, ntz, viewer),
            )
        };
        [
            edge(tx + 1, tz),
            edge(tx - 1, tz),
            edge(tx, tz + 1),
            edge(tx, tz - 1),
        ]
    };
    let build = |commands: &mut Commands,
                 meshes: &mut Assets<Mesh>,
                 level: u8,
                 tx: i32,
                 tz: i32|
     -> LoadedFarTile {
        let (spans, culled) = tile_column_spans(base, level, tx, tz, viewer, &sample);
        let ring_edges = ring_edges_of(level, tx, tz);
        let (mesh_data, y_ref) =
            build_far_tile_mesh(base, level, tx, tz, &spans, &culled, ring_edges);
        let cull_chunk = tile_in_cull_band(base, level, tx, tz, viewer).then_some(cur_chunk);
        if mesh_data.is_empty() {
            // Every column culled (fully under the near field): a real, tracked
            // "meshed to nothing" so it isn't re-attempted every frame.
            return LoadedFarTile {
                entity: None,
                cull_chunk,
            };
        }
        let transform = Transform::from_translation(far_tile_translation(
            base, level, tx, tz, y_ref, viewer, origin.0,
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
        LoadedFarTile {
            entity: Some(ent.id()),
            cull_chunk,
        }
    };

    let mut budget = FAR_SURFACE_BUDGET_PER_FRAME;
    for (_, level, tx, tz) in missing {
        if budget == 0 {
            break;
        }
        let loaded = build(&mut commands, &mut meshes, level, tx, tz);
        map.loaded.insert((level, tx, tz), loaded);
        budget -= 1;
    }
    for (_, level, tx, tz) in stale {
        if budget == 0 {
            break;
        }
        let old = map.loaded.get(&(level, tx, tz)).and_then(|t| t.entity);
        let loaded = build(&mut commands, &mut meshes, level, tx, tz);
        if let Some(e) = old {
            commands.entity(e).despawn();
        }
        map.loaded.insert((level, tx, tz), loaded);
        budget -= 1;
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
        // The ring band carries the one-tile inner overlap (`far_tile_in_ring`):
        // level-2 tiles lap one L2 tile inward past RING_EDGES_M[1] to close the
        // seam with L1 (journal/0022 walk 17).
        let inner = RING_EDGES_M[1] - far_tile_m(base, 2);
        for &(tx, tz) in &a {
            let d = far_tile_center_dist(base, 2, tx, tz, viewer);
            assert!(
                (inner..RING_EDGES_M[2]).contains(&d),
                "tile at horizontal distance {d} outside LOD-2 ring (inner {inner})"
            );
        }
    }

    /// Mesh a tile for tests: derive spans + coverage from `sample`/`viewer`,
    /// then run the pure mesher. `ring_edges` all false = an interior tile with
    /// same-level neighbours on every side.
    fn mesh_tile(
        base: VoxelScale,
        level: u8,
        tx: i32,
        tz: i32,
        viewer: DVec3,
        ring_edges: [bool; 4],
        sample: &dyn Fn(i64, i64) -> (i32, Block),
    ) -> (MeshData, f64) {
        let (spans, culled) = tile_column_spans(base, level, tx, tz, viewer, sample);
        build_far_tile_mesh(base, level, tx, tz, &spans, &culled, ring_edges)
    }

    /// A viewer far enough that `near_covers` never fires — isolates meshing from
    /// the coverage cull.
    const FAR_VIEWER: DVec3 = DVec3::new(1.0e6, 1.0e6, 1.0e6);

    #[test]
    fn quantize_top_floors_to_the_coarse_lattice() {
        // Floor to the level's coarse voxel: always ≤ h, exact when aligned, and
        // never off by a whole coarse voxel (the no-sink downward bias).
        for level in 0..=4u8 {
            let s = level_stride(level);
            for h in -40i32..=40 {
                let q = quantize_top(h, s);
                assert_eq!(q % s as i32, 0, "not on the coarse lattice");
                assert!(q <= h, "quantized top rose above the surface");
                assert!(
                    h - q < s as i32,
                    "quantization error exceeds a coarse voxel"
                );
            }
        }
    }

    #[test]
    fn far_tile_is_stepped_voxel_columns_not_a_smooth_sheet() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        // A flat tile greedy-merges its whole top to ONE quad (2 tris), with no
        // side faces (equal neighbours, interior edges) — the voxel-language win.
        let flat = |_wx: i64, _wz: i64| (20i32, Block::Grass);
        let (mesh, _y) = mesh_tile(base, 1, 0, 0, FAR_VIEWER, [false; 4], &flat);
        assert_eq!(
            mesh.triangle_count(),
            2,
            "flat tile should merge to one quad"
        );
        for n in &mesh.normals {
            assert_eq!(n[1], 1.0, "flat top normals point straight up");
        }

        // A stepped surface emits horizontal side faces (the voxel steps) — a
        // smooth TIN never would.
        let stepped = |wx: i64, _wz: i64| {
            let cell = wx.div_euclid(level_stride(1));
            (if cell.rem_euclid(2) == 0 { 6 } else { 12 }, Block::Stone)
        };
        let (mesh, yr) = mesh_tile(base, 1, 0, 0, FAR_VIEWER, [false; 4], &stepped);
        let has_side = mesh.normals.iter().any(|n| n[1] == 0.0);
        assert!(has_side, "stepped columns must emit vertical side faces");
        // Every emitted top sits on the coarse lattice (steps, not a slope).
        let base_vs = base.voxel_size_m() as f32;
        for (p, nrm) in mesh.positions.iter().zip(&mesh.normals) {
            if nrm[1] > 0.0 {
                let world_y = p[1] + (yr as f32);
                let steps = world_y / (base_vs * level_stride(1) as f32);
                assert!((steps - steps.round()).abs() < 1e-3, "top off the lattice");
            }
        }
        // Block-colored, single atlas layer at full weight (one shared material).
        for w in &mesh.mat_weights {
            assert_eq!(*w, [1.0, 0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn far_columns_quantize_the_near_ground_below_it() {
        // Near/far parity, quantized-exact and no sink: the far column top equals
        // floor(near_height / stride) * stride, is never above the near surface,
        // and is within one coarse voxel of it.
        use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let mut g = WorldGenerator::new(&pregen);
        for level in 1..=3u8 {
            let stride = level_stride(level);
            for j in 0..8i64 {
                for i in 0..8i64 {
                    let (wx, wz) = (i * stride, j * stride);
                    let (h, _) = g.coarse_surface(wx, wz);
                    let col = g.column_record(wx.div_euclid(32), wz.div_euclid(32));
                    let (lx, lz) = (wx.rem_euclid(32) as usize, wz.rem_euclid(32) as usize);
                    let near = col.heights[lz * 32 + lx];
                    assert_eq!(h, near, "far summary must equal the near column height");
                    let q = quantize_top(h, stride);
                    assert_eq!(
                        q,
                        near.div_euclid(stride as i32) * stride as i32,
                        "quantized-exact parity"
                    );
                    assert!(q <= near, "far top rose above near (would need a sink)");
                    assert!(near - q < stride as i32, "error exceeds one coarse voxel");
                }
            }
        }
    }

    #[test]
    fn near_coverage_culls_far_columns_instead_of_burying_them() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let surf = 100i32;
        let surf_m = f64::from(surf) * base.voxel_size_m();
        let sample = |_wx: i64, _wz: i64| (surf, Block::Grass);

        // A distant viewer culls nothing — the full horizon renders.
        let (_s, culled_far) = tile_column_spans(base, 1, 0, 0, FAR_VIEWER, &sample);
        assert!(
            culled_far.iter().all(|c| !c),
            "distant viewer culls no columns"
        );

        // A viewer standing on the surface at the L1 tile's centre puts every
        // column inside NEAR_COVER_R (an L1 tile is ~57 m < 112 m), so the tile is
        // fully covered and meshes to NOTHING — no buried sheet to dig into.
        let tile_m = far_tile_m(base, 1);
        let centre = DVec3::new(tile_m * 0.5, surf_m, tile_m * 0.5);
        let (spans, culled) = tile_column_spans(base, 1, 0, 0, centre, &sample);
        assert!(
            culled.iter().all(|&c| c),
            "a tile under the near field must have all columns culled"
        );
        let (mesh, _y) = build_far_tile_mesh(base, 1, 0, 0, &spans, &culled, [false; 4]);
        assert!(
            mesh.is_empty(),
            "a fully-covered tile draws nothing (coverage, not buried geometry)"
        );
    }

    /// FF2a scale-headroom checkpoint (deliverable 4). Measures REAL per-tile
    /// vertex bytes + build time on the worldgen authority across the current
    /// 1.2 km field, then projects tile count / memory for a 5–10 km draw
    /// distance (the design target). Run with `--nocapture` to read the numbers;
    /// as a test it just asserts the field is non-empty and bounded. Does NOT
    /// change the shipped radius.
    #[test]
    fn scale_headroom_checkpoint() {
        use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};
        use std::cell::RefCell;
        use std::time::Instant;

        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        });
        let g = RefCell::new(WorldGenerator::new(&pregen));
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let sample = |wx: i64, wz: i64| g.borrow_mut().coarse_surface(wx, wz);
        // Viewer high above so nothing is coverage-culled — worst case (every
        // wanted tile emits its full stepped geometry).
        let viewer = DVec3::new(0.0, 5000.0, 0.0);

        // pos(12) + normal(12) + color(16) + uv(8) + layers(16) + weights(16).
        const BYTES_PER_VERT: usize = 12 + 12 + 16 + 8 + 16 + 16;
        let mut tiles = 0usize;
        let mut tris = 0usize;
        let mut verts = 0usize;
        let mut per_ring = [0usize; 5];
        let t0 = Instant::now();
        for level in 1..=4u8 {
            for (tx, tz) in wanted_far_tiles(base, viewer, level) {
                let (spans, culled) = tile_column_spans(base, level, tx, tz, viewer, &sample);
                let (mesh, _y) =
                    build_far_tile_mesh(base, level, tx, tz, &spans, &culled, [true; 4]);
                tiles += 1;
                per_ring[level as usize] += 1;
                tris += mesh.triangle_count();
                verts += mesh.positions.len();
            }
        }
        let elapsed = t0.elapsed();
        let mesh_bytes = verts * BYTES_PER_VERT + tris * 3 * 4;
        let avg_tile_bytes = mesh_bytes / tiles.max(1);
        let ms_per_tile = elapsed.as_secs_f64() * 1e3 / tiles.max(1) as f64;
        // Projection: extend by doubling LOD rings; tiles-per-ring stays ~flat
        // (annulus area and tile area both scale ~4× per doubling). Extra rings
        // to reach R from the current 1.2 km outer edge:
        let tiles_per_ring = tiles / 4;
        let project = |r: f64| -> (usize, f64) {
            let extra = (r / RING_EDGES_M[4]).log2().ceil().max(0.0) as usize;
            let t = tiles + extra * tiles_per_ring;
            (t, (t * avg_tile_bytes) as f64 / 1_048_576.0)
        };
        let (t5, mb5) = project(5_000.0);
        let (t10, mb10) = project(10_000.0);
        println!("\n=== FF2a scale-headroom checkpoint (worst case: no coverage cull) ===");
        println!("bytes/vertex = {BYTES_PER_VERT} (pos+nrm+col+uv+layers+weights)");
        println!(
            "current 1.2 km field: {tiles} tiles {per_ring:?} (L1..L4), {tris} tris, {verts} verts",
        );
        println!(
            "  mesh memory {:.2} MiB, avg {avg_tile_bytes} bytes/tile ({:.1} tris/tile avg)",
            mesh_bytes as f64 / 1_048_576.0,
            tris as f64 / tiles.max(1) as f64,
        );
        println!(
            "  derive+mesh {:.3} s total, {ms_per_tile:.3} ms/tile (budget 2 tiles/frame)",
            elapsed.as_secs_f64(),
        );
        println!("projected 5 km:  ~{t5} tiles, ~{mb5:.1} MiB mesh");
        println!("projected 10 km: ~{t10} tiles, ~{mb10:.1} MiB mesh");
        println!(
            "per-frame meshing stays ~{:.3} ms (2 tiles) regardless of radius — no hitch; \
             full 10 km fill ~{:.1} s at 2 tiles/frame @60fps\n",
            2.0 * ms_per_tile,
            t10 as f64 / 2.0 / 60.0,
        );
        assert!(tiles > 0, "the far field must produce tiles");
        assert!(avg_tile_bytes < 200_000, "per-tile mesh unexpectedly large");
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

    /// Walk 17's sky holes, as a coverage theorem. The far tiles of a single
    /// level partition the ground plane, so a point lies in exactly one tile per
    /// level; the far field covers a point iff *some* level's containing tile is
    /// wanted. This sweeps the inner rings densely across the L1/L2 (256 m) and
    /// L2/L3 (512 m) seams and asserts every ground point in the far field's
    /// responsibility band is covered by at least one tile. FAILS on the pre-fix
    /// code — at each inter-ring boundary some cells are rejected by both the
    /// finer ring (center past the outer edge) and the coarser ring (center short
    /// of the inner edge), leaving fixed-position sky holes. (The outermost L4
    /// edge is deliberately excluded: its huge tiles make the world's outer rim
    /// legitimately ragged, and that rim dissolves in haze — not a seam bug.)
    #[test]
    fn far_tiles_cover_the_rings_without_seams() {
        use std::collections::HashSet;
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        // An arbitrary, grid-unaligned viewer so no seam hides behind a lattice
        // symmetry.
        let viewer = DVec3::new(123.4, 1000.0, -77.6);
        let sets: Vec<HashSet<(i32, i32)>> = (1..=3u8)
            .map(|l| wanted_far_tiles(base, viewer, l).into_iter().collect())
            .collect();
        let covered = |px: f64, pz: f64| -> bool {
            (1..=3u8).any(|l| {
                let tile_m = far_tile_m(base, l);
                let tx = (px / tile_m).floor() as i32;
                let tz = (pz / tile_m).floor() as i32;
                sets[usize::from(l) - 1].contains(&(tx, tz))
            })
        };
        // Sweep from the LOD-1 inner edge into the L3 body, densely in radius
        // (every 1 m) and angle (0.25°). The cap stays clear of the L3/L4 seam:
        // level-4 tiles are so large that the world's outer rim (dropped where a
        // tile center passes 1200 m) reaches inward far enough to entangle the
        // 1024 m seam, and that rim is a legitimate, haze-dissolved world edge —
        // not the seam bug under test. Within [112, 800] every point that the
        // far field must cover is coverable by levels 1–3 alone.
        let r_max = 800.0;
        let mut holes = 0usize;
        let mut first_hole = None;
        let mut r = RING_EDGES_M[0]; // 112 m: LOD-1 inner edge
        while r < r_max {
            let steps = 1440; // 0.25° angular resolution
            for k in 0..steps {
                let a = std::f64::consts::TAU * f64::from(k) / f64::from(steps);
                let px = viewer.x + r * a.cos();
                let pz = viewer.z + r * a.sin();
                if !covered(px, pz) {
                    holes += 1;
                    if first_hole.is_none() {
                        first_hole = Some((r, a));
                    }
                }
            }
            r += 1.0;
        }
        assert_eq!(
            holes, 0,
            "far field has {holes} uncovered ground points (sky holes); \
             first at radius/angle {first_hole:?}"
        );
    }
}
