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
//! Every far chunk/tile is therefore pushed [`DEPTH_PUSH_FRAC`] of its coarse
//! voxel *away from the camera along the camera-forward axis*, separating every
//! face orientation in real depth.
//!
//! **Why a UNIFORM per-level push, not a per-tile radial one (corrections #11).**
//! The push originally moved each tile along its OWN center-to-viewer direction.
//! Adjacent same-level tiles then translated along *slightly different*
//! directions (differing by their angular separation ≈ tile_m/dist as seen from
//! the viewer), so their shared edge separated by push × (tile_m/dist) — 0.08 m
//! (L1) up to 0.9 m (L4). On the S1 volumetric shells that sliver merely exposed
//! a neighbour's own side geometry, but once the far field became a hollow
//! top-surface sheet (journal/0022–0023) the same offsets became see-through
//! slots — thin bright seams of background light, worst from altitude looking
//! down. The mesh was watertight; the *transform* stage reopened it.
//!
//! The cure: compute ONE shared push vector per LOD level per frame — direction
//! = the camera-forward axis (unit, shared by every tile of every level this
//! frame), magnitude = `DEPTH_PUSH_FRAC × that level's coarse voxel`. Every tile
//! of a level then undergoes the *identical rigid translation*, so shared edges
//! cannot separate — **same-level watertightness by construction** (the fix).
//! It is still a true world-space offset, never a depth bias / draw order
//! (corrections #1): translating along the view axis adds exactly `magnitude` of
//! view-space depth to *every* face orientation uniformly, and levels differ in
//! magnitude, so overlapping ring pairs (coarser = larger push) still separate
//! in the lap band. The camera-forward axis is always unit, so the old
//! `normalize_or_zero` degeneracy is gone. Transforms are recomputed from f64
//! every frame ([`position_far_tiles`] / [`position_far_chunks`]), so a
//! per-frame camera-forward direction is architecturally free.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use dc_core::farfield::{
    ColumnSpan, FAR_BOTTOM_UNBOUNDED, compose_column, level_stride, quantize_top,
};
use dc_core::{Block, CHUNK_SIZE, ChunkPos, VoxelScale};
use dc_worldgen::{Pregen, WorldGenerator};
use glam::DVec3;

use crate::app::{
    CurrentScale, FloatingOrigin, Fullbright, FullbrightMaterialHandle, TerrainMaterialHandle,
    churn, to_render,
};
use crate::authority::Authority;
use crate::farpyramid::{FarPyramid, NodeGrids};
use crate::meshing::{MeshData, block_layer, face_color, mesh_chunk};
use crate::meshtasks::{FarMeshOutput, FarMeshTasks, MAX_INFLIGHT_FAR};
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

/// Outer edge of the far field when nothing overrides it (`--horizon`).
pub const DEFAULT_FAR_MAX_M: f64 = 1200.0;

/// The single LOD range ladder — **one source of truth** for every far/near
/// distance the renderer uses (journal/0091). The nearfield border plus each
/// far LOD step's starting range *expressed as a distance past that border*;
/// the ring edges, the warm/cold reduction selection, and the near-field
/// load/unload radii all DERIVE from these fields, so no LOD range is defined
/// in two places that could silently drift apart (the constant-coupling defect
/// the 2026-07-24 audit named § 6.3: `REDUCTION_STANDOFF_M` set independently of
/// the L1 ring edge, `UNLOAD_RADIUS_M` set independently of the near cover).
///
/// **Every field is a range knob, shaped to later become a per-player perf
/// setting** — a lower-spec player would pull the coarse rings inward, a wider
/// view dial push them out, all by editing these numbers and letting everything
/// re-derive. No settings UI exists yet, and this slice builds none: these are
/// only coupled *named* knobs under one authority, ready for that future.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodLadder {
    /// The nearfield border (m): within it the near volumetric field covers the
    /// surface (far columns are CULLED there, never buried), and at it the finest
    /// far ring (LOD-1) begins. Also the near-coverage radius `near_covers` uses.
    pub nearfield_border_m: f64,
    /// How far PAST the border the near full-detail field still streams real
    /// chunks — the seam overlap band. Near load radius =
    /// `nearfield_border_m + overlap_past_border_m`; the far field laps under it.
    pub overlap_past_border_m: f64,
    /// Extra distance past the load radius before a streamed near chunk unloads
    /// (streaming hysteresis). Unload radius = load radius + this.
    pub unload_slack_m: f64,
    /// Each far LOD step's inner edge as a distance PAST the nearfield border.
    /// `step_past_border_m[k]` starts LOD-(k+2): index 0 → LOD-2, 1 → LOD-3,
    /// 2 → LOD-4. LOD-1 always starts at the border itself (its step ≡ 0). The
    /// shipped values reproduce the historical 2×/4×/8× full-detail ring ladder
    /// (256 / 512 / 1024 m) exactly. These are the primary per-player range
    /// knobs: shrink them to pull the coarse rings inward on a weaker machine.
    pub step_past_border_m: [f64; 3],
    /// Outer edge of the far field (the horizon, LOD-4's outer edge) — what
    /// `--horizon` sets. The only edge a wider horizon moves (journal/0042); the
    /// inner rings stay put because their edges are border-relative, not
    /// horizon-relative.
    pub far_max_m: f64,
}

impl LodLadder {
    /// The shipped ladder: border 112 m, load 128 m, unload 160 m, ring inner
    /// edges [112, 256, 512, 1024] m, horizon 1200 m — byte-identical to the
    /// pre-ladder scattered constants (pinned by
    /// `default_ring_edges_match_the_shipped_constants`), so a launch with no
    /// `--horizon` renders exactly as before the ladder existed.
    pub const DEFAULT: LodLadder = LodLadder {
        nearfield_border_m: 112.0,
        overlap_past_border_m: 16.0,
        unload_slack_m: 32.0,
        step_past_border_m: [144.0, 400.0, 912.0],
        far_max_m: DEFAULT_FAR_MAX_M,
    };

    /// The near full-detail load radius (m): the near field streams this sphere.
    #[inline]
    pub const fn load_radius_m(&self) -> f64 {
        self.nearfield_border_m + self.overlap_past_border_m
    }

    /// The near-field unload radius (m): load radius plus hysteresis slack. The
    /// number `streaming` streams by AND the far field's viewer-relative refresh
    /// reach — one definition (was the independent `LOAD_RADIUS_M + 32`).
    #[inline]
    pub const fn unload_radius_m(&self) -> f64 {
        self.load_radius_m() + self.unload_slack_m
    }

    /// The near-coverage radius (m): far columns whose 3-D distance to the viewer
    /// is under this are CULLED (the near field draws them). Equals the border,
    /// so culling exactly the border disc leaves the intended overlap band
    /// occluded and NO deeper buried geometry (journal/0022 walk-17). Independent
    /// of the horizon: widening the far field never changes what the near covers.
    #[inline]
    pub const fn near_cover_r_m(&self) -> f64 {
        self.nearfield_border_m
    }

    /// The five ring edges `[inner_L1, inner_L2, inner_L3, inner_L4, far_max]`,
    /// derived from the border + steps-past-border. Level L covers
    /// `[edges[L-1], edges[L])` by centre distance. Clamped monotone into
    /// `[border, far_max]` so an absurdly *short* horizon collapses the inner
    /// rings instead of inverting them.
    pub fn ring_edges(&self) -> [f64; 5] {
        let border = self.nearfield_border_m;
        let far_max = self.far_max_m.max(border);
        let e = |past: f64| (border + past).clamp(border, far_max);
        [
            border,
            e(self.step_past_border_m[0]),
            e(self.step_past_border_m[1]),
            e(self.step_past_border_m[2]),
            far_max,
        ]
    }

    /// This ladder with a different horizon (what `--horizon <km>` dials).
    pub fn with_far_max(self, far_max_m: f64) -> Self {
        Self { far_max_m, ..self }
    }
}

/// Full-detail radius in meters — the near load sphere. **Derived from the
/// single ladder** ([`LodLadder::load_radius_m`]); kept as a named const only
/// because `streaming` and `bench_storage` still spell it.
pub const FULL_DETAIL_RADIUS_M: f64 = LodLadder::DEFAULT.load_radius_m();
/// The LOD-1 ring starts this far *inside* the full-detail edge (the seam
/// overlap band). Ladder-derived ([`LodLadder::overlap_past_border_m`]).
pub const FAR_OVERLAP_M: f64 = LodLadder::DEFAULT.overlap_past_border_m;
/// Smallest / largest horizon a launch flag may ask for (meters). Below the
/// full-detail radius there is no far field to speak of; above ~64 km the
/// outermost ring's tile count runs away (see journal/0042's table).
pub const HORIZON_MIN_M: f64 = 200.0;
pub const HORIZON_MAX_M: f64 = 64_000.0;

/// The far field's ring geometry — **runtime** configuration (`--horizon`),
/// not a compile-time constant — carried as a [`LodLadder`] plus its derived
/// ring edges.
///
/// Why this stopped being a `const` (journal/0042): the shipped 1.2 km horizon
/// puts the camera *inside* every landform the worldgen builds — a mountain
/// range is 5–20 km across, so its macro shape never entered frame and the
/// journal/0040 walk could not judge the landform it was standing on. The
/// horizon had to become something a walker can dial per launch. Widening it
/// stretches only the **outermost** ring (the [`LodLadder`] steps are
/// border-relative), which is the cheap direction — the fine rings' tile scan
/// stays small and the extra area is covered by L4's coarse tiles.
///
/// [`Default`] reproduces the shipped ladder exactly (asserted in
/// `default_ring_edges_match_the_shipped_constants`), so a launch with no
/// `--horizon` renders byte-identically to before the knob existed.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct HorizonConfig {
    /// The single LOD range ladder this horizon derives everything from.
    pub ladder: LodLadder,
    /// Ring edges in meters (derived from `ladder`): level L covers
    /// `[ring_edges[L-1], ring_edges[L])` by center distance, L in 1..=4. A
    /// derived field, cached here for the hot want-set scan; never set directly.
    pub ring_edges: [f64; 5],
}

impl Default for HorizonConfig {
    fn default() -> Self {
        Self::from_ladder(LodLadder::DEFAULT)
    }
}

impl HorizonConfig {
    /// Build the horizon config from a ladder, deriving (and caching) the ring
    /// edges — the ONE place ring edges come from.
    pub fn from_ladder(ladder: LodLadder) -> Self {
        Self {
            ring_edges: ladder.ring_edges(),
            ladder,
        }
    }

    /// The default ladder with a different horizon — what `--horizon <km>`
    /// builds.
    pub fn with_far_max(far_max_m: f64) -> Self {
        Self::from_ladder(LodLadder::DEFAULT.with_far_max(far_max_m))
    }

    /// Inner edge of level `level`'s ring.
    #[inline]
    fn inner(&self, level: u8) -> f64 {
        self.ring_edges[usize::from(level) - 1]
    }

    /// Outer edge of level `level`'s ring.
    #[inline]
    fn outer(&self, level: u8) -> f64 {
        self.ring_edges[usize::from(level)]
    }

    /// Horizontal radius (m) within which the near volumetric field covers the
    /// surface, so far columns there are CULLED rather than buried. Ladder-derived
    /// ([`LodLadder::near_cover_r_m`]).
    #[inline]
    fn near_cover_r_m(&self) -> f64 {
        self.ladder.near_cover_r_m()
    }

    /// The near-field unload radius (m) — the far field's viewer-relative refresh
    /// reach (`tile_in_cull_band`) is measured against it. Ladder-derived, the
    /// same number `streaming::UNLOAD_RADIUS_M` streams by.
    #[inline]
    fn unload_radius_m(&self) -> f64 {
        self.ladder.unload_radius_m()
    }

    /// Camera far plane for this horizon. The shipped 3 km plane was 2.5× the
    /// 1.2 km field (headroom for the outermost ring's far corners); keeping the
    /// ratio means the default is unchanged and a wider horizon is not clipped.
    pub fn camera_far_m(&self) -> f32 {
        (2.5 * self.ladder.far_max_m).max(2.5 * DEFAULT_FAR_MAX_M) as f32
    }

    /// Distance fog range (start, end) in meters for this horizon. The shipped
    /// lit-pass values (150 / 1100 m) are tuned against the 1.2 km field; a
    /// horizon that reaches 10 km must push the haze out with it or the whole
    /// point of the knob — seeing macro landform shape — is fogged away. Scales
    /// with the horizon, so the default is byte-identical.
    pub fn fog_range_m(&self) -> (f32, f32) {
        let s = self.ladder.far_max_m / DEFAULT_FAR_MAX_M;
        ((150.0 * s) as f32, (1100.0 * s) as f32)
    }
}
/// Far chunks generated + meshed per frame (each costs ~2.5 ms of main-thread
/// time; the full 1 km field fills in a few seconds of streaming).
const FAR_BUDGET_PER_FRAME: usize = 3;
/// Hysteresis: a far chunk is only despawned once its center is this far
/// outside its ring, so ring membership doesn't thrash while walking.
const FAR_UNLOAD_SLACK_M: f64 = 48.0;
/// Fraction of a level's coarse voxel size that its chunks/tiles are pushed away
/// from the camera along the camera-forward axis, to separate coplanar faces in
/// depth (see module docs — one shared vector per level per frame). L1 ≈ 0.27 m
/// of view-space depth — angularly invisible, decisively beyond f32 depth
/// interpolation error.
const DEPTH_PUSH_FRAC: f64 = 0.15;

/// The shared anti-z-fight push for a LOD level, in world meters: `DEPTH_PUSH_FRAC`
/// of the level's coarse voxel along the camera-forward axis. Depends ONLY on the
/// level and the (per-frame, shared) camera forward — **never on a tile's own
/// position** — so every tile/chunk of a level is translated by the identical
/// vector and same-level shared edges cannot separate (corrections #11). Both far
/// paths (S1 chunks and FF2a tiles) route their push through this one function.
/// `forward` is always unit (the camera view axis), so no `normalize` is needed.
#[inline]
fn level_depth_push(base: VoxelScale, level: u8, forward: DVec3) -> DVec3 {
    forward * (DEPTH_PUSH_FRAC * coarse_scale(base, level).voxel_size_m())
}

/// The anti-z-fight translation for a far chunk: origin-relative position plus
/// the half-voxel downward seam bias plus the UNIFORM-per-level depth push.
/// `forward` is the camera-forward axis (unit), shared by every chunk this frame,
/// so all same-level chunks translate by the identical vector — their shared
/// edges cannot separate (corrections #11). The push magnitude depends only on
/// the level, so overlapping ring pairs still separate in the lap band.
fn far_transform_translation(
    base: VoxelScale,
    level: u8,
    pos: ChunkPos,
    forward: DVec3,
    origin_m: DVec3,
) -> Vec3 {
    let vs = coarse_scale(base, level).voxel_size_m();
    let (mx, my, mz) = pos.min_voxel();
    let min_m = DVec3::new(mx as f64, my as f64, mz as f64) * vs;
    let bias = DVec3::new(0.0, -0.5 * vs, 0.0);
    let away = level_depth_push(base, level, forward);
    to_render(min_m + bias + away - origin_m)
}

/// LOD level whose ring contains a chunk-center distance, `None` for the
/// full-detail region and beyond the far field.
pub fn level_for_distance(hz: &HorizonConfig, d: f64) -> Option<u8> {
    if !(hz.ring_edges[0]..hz.ring_edges[4]).contains(&d) {
        return None;
    }
    (1..=4u8).find(|level| d < hz.outer(*level))
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
pub fn wanted_far_positions(
    base: VoxelScale,
    viewer_m: DVec3,
    level: u8,
    hz: &HorizonConfig,
) -> Vec<ChunkPos> {
    let chunk_m = coarse_scale(base, level).voxels_to_meters(f64::from(CHUNK_SIZE));
    let outer = hz.outer(level);
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
                if level_for_distance(hz, d) == Some(level) {
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
    // The legacy S1 far mesh does NOT follow `--horizon` (journal/0042). It is a
    // volumetric 3-D shell, so its wanted-set is a *cube* of chunk positions:
    // stretching its outer ring grows the scan and the loaded set with the CUBE of
    // the horizon, not the square. It is also the phantom-old-world path
    // (journal/0017) that only the 3/4 keys reach. The knob belongs to the real
    // (worldgen) horizon; this one keeps the shipped 1.2 km geometry.
    let hz = HorizonConfig::default();

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
            d < hz.inner(*level) - FAR_UNLOAD_SLACK_M || d > hz.outer(*level) + FAR_UNLOAD_SLACK_M
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
        for pos in wanted_far_positions(base, player.pos_m, level, &hz) {
            if !map.loaded.contains_key(&(level, pos)) {
                let d = (far_chunk_center_m(base, level, pos) - player.pos_m).length();
                missing.push(((d * 1000.0) as u64, level, pos));
            }
        }
    }
    missing.sort_unstable_by_key(|(d, _, _)| *d);

    for (_, level, pos) in missing.into_iter().take(FAR_BUDGET_PER_FRAME) {
        let cscale = coarse_scale(base, level);
        // Churn instrument (journal/0051), same window as the near path.
        let build_start = std::time::Instant::now();
        // Perf window (journal/0080): the legacy S1 volumetric far mesh (keys 3/4
        // only; absent under the boot worldgen authority). Zero cost without perf.
        let _perf = crate::perf_span!("far_chunk.build");
        let chunk = terrain.0.generate_chunk(cscale, pos);
        // Faces cull against same-level generator samples, so a ring is
        // seamless internally; ring-to-ring boundaries are the accepted seam.
        // Coverage, not solidity (journal/0057). The far rings have no per-voxel
        // contents, so every solid far voxel is full height — which also means
        // the far field does NOT show the near field's partial-height tops. A
        // documented LOD difference, not a culling bug.
        let neighbor_fill = |x: i64, y: i64, z: i64| {
            crate::meshing::cover_frac(terrain.0.block_at(cscale, x, y, z), None)
        };
        let mesh_data = mesh_chunk(
            &chunk,
            pos,
            cscale.voxel_size_m() as f32,
            &neighbor_fill,
            None,
        );
        let bevy_mesh = (!mesh_data.is_empty()).then(move || to_bevy_mesh(mesh_data));
        churn::record(&churn::FAR_MESHES, &churn::FAR_NANOS, build_start);
        let entity = if let Some(bevy_mesh) = bevy_mesh {
            // Spawn already positioned (same reasoning as streaming.rs): a
            // default transform renders one frame at the floating origin.
            let transform = Transform::from_translation(far_transform_translation(
                base,
                level,
                pos,
                player.view_dir(),
                origin.0,
            ));
            let mut ent = commands.spawn((
                Mesh3d(meshes.add(bevy_mesh)),
                FarChunkEntity { level, pos },
                transform,
            ));
            if fullbright.0 {
                ent.insert(MeshMaterial3d(material.0.clone()));
            } else {
                ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
            }
            Some(ent.id())
        } else {
            None
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
    // One shared camera-forward push direction for every chunk this frame — the
    // uniform-per-level push that keeps same-level seams closed (corrections #11).
    let forward = player.view_dir();
    for (far, mut transform) in &mut chunks {
        transform.translation =
            far_transform_translation(scale.scale, far.level, far.pos, forward, origin.0);
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
// and are pushed slightly away from the camera so their coplanar faces separate
// in depth (corrections #1 — draw order can't fix coplanar z-fight; the uniform
// per-level push moves faces apart in real depth, coarser rings farther). The
// tiles are a 2-D annulus at the
// surface: looking up from deep in a chasm still loses the far field (accepted —
// FF2b's volumetric spans own the chasm case; the per-column payload is already
// a [`ColumnSpan`] so that extension needs no mesher rewrite).
// ===========================================================================

/// The FarPyramid reduced-node grids one tile needs, snapshotted on the frame
/// thread and moved into the derive task (journal/0084). A tile's 34² columns
/// touch at most the **3×3** node plan columns centred on `(tx, tz)`, so this is
/// that patch → each node column's [`NodeGrids`]. The grids are `Arc`-shared
/// owned data, so the snapshot is cheap to move and the task reads reduced nodes
/// without any `&mut FarPyramid` access crossing the thread boundary. Aliased so
/// the snapshot/spawn seam is not a bare map type (clippy::type_complexity).
type NodeSnapshot = HashMap<(i32, i32), NodeGrids>;

/// Far surface tiles generated + meshed per frame. The full horizon fills in a
/// second or two of streaming, same as the S1 far mesh.
const FAR_SURFACE_BUDGET_PER_FRAME: usize = 2;
/// Depth of the ring-transition / near-seam skirt, in this level's coarse
/// voxels. A stepped tile's *outer* boundary (where the next-coarser ring takes
/// over, or where the near field culls its columns) drops a short vertical apron
/// this deep so a stride mismatch at the transition can't open a pixel crack.
/// Same-level tile boundaries need no skirt (their steps are watertight).
const RING_SKIRT_COARSE_VOXELS: i64 = 2;

// The per-column payload ([`ColumnSpan`]) and its quantization
// ([`quantize_top`], [`level_stride`]) moved to `dc_core::farfield` with FF2b
// (journal/0070): a far column is now a STACK of spans — solid `[bottom, top)`
// runs, topmost first, the last one bottom-unbounded — derived headlessly by
// the two-sided node contract (reduce where chunks exist, synthesize where
// they never will) and composed here per tile. FF2a's single-span top sheet is
// exactly the one-span stack, so the synthesized (default) case renders
// byte-identically to journal/0023.

/// Make a resident node's reduced spans **geometry-safe** against the near
/// ground: no reduced span may raise the column's surface above the
/// floor-quantized synthesized top (`synth_top`), the same floor cold synthesis
/// uses (journal/0091). The block/material each span carries is untouched — only
/// its *height* is floored.
///
/// Why this is the prerequisite that lets the 176 m reduction standoff die: the
/// block pyramid's `MajorityNonAir` reduce ROUNDS the surface cell to nearest,
/// so a reduced coarse top can sit up to one coarse voxel ABOVE the true
/// surface, while FF2a synthesis FLOORS ([`quantize_top`]) and cannot. That one
/// coarse voxel of round-up is the only way warm data could poke through the
/// near opaque ground, so the old code refused reduced data anywhere the near
/// field draws (the standoff) — which is exactly what forced the finest band's
/// inner shell back to cold dither (the band inversion, audit § 5). Flooring the
/// warm surface to the same lattice line synthesis floors to closes the
/// poke-through *at the source*, so reduced data is now safe to use right up to
/// the near-cover edge.
///
/// A span whose bottom sits at or below `synth_top` while its top rises above it
/// is the rounded surface run: its top is clamped down to `synth_top` (it then
/// fuses with the synthesized ground below in [`compose_column`], so cold and
/// warm floor to the *identical* surface height — only their material can still
/// differ, which is fix (b), the S-9 reconciliation, not this slice). A span
/// wholly above `synth_top` (bottom > `synth_top`, an air gap beneath) is a real
/// overhang/structure the near field also renders, and is kept. A span wholly at
/// or below `synth_top` is subsurface/known-air and is kept.
fn floor_known_surface(
    synth_top: i32,
    known: &[(i32, i32, Vec<ColumnSpan>)],
) -> Vec<(i32, i32, Vec<ColumnSpan>)> {
    known
        .iter()
        .map(|(kb, kt, spans)| {
            let floored = spans
                .iter()
                .filter_map(|s| {
                    if s.top > synth_top && s.bottom <= synth_top {
                        // Grounded surface run rounded up past the floor: clamp
                        // its top to the floor. If that empties it (a lone coarse
                        // voxel sitting exactly on the floor line), drop it — the
                        // synthesized ground already reaches `synth_top`.
                        (synth_top > s.bottom).then_some(ColumnSpan {
                            top: synth_top,
                            bottom: s.bottom,
                            block: s.block,
                        })
                    } else {
                        Some(*s)
                    }
                })
                .collect();
            (*kb, *kt, floored)
        })
        .collect()
}

/// Whether the near volumetric field covers a far column at world `(wx_m, wz_m)`
/// whose stepped top is at `top_m` — a pure function of the viewer pose, so a
/// tile's culled set depends only on the viewer and not on near-chunk streaming
/// progress (deterministic, unit-testable). Altitude-aware: flying far above the
/// surface grows the vertical term, shrinking the covered disc to nothing, so the
/// horizon is never culled when viewed from the air (the key-2 high vantage).
fn near_covers(wx_m: f64, wz_m: f64, top_m: f64, viewer_m: DVec3, hz: &HorizonConfig) -> bool {
    let dx = wx_m - viewer_m.x;
    let dz = wz_m - viewer_m.z;
    let dy = top_m - viewer_m.y;
    let r = hz.near_cover_r_m();
    dx * dx + dy * dy + dz * dz < r * r
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
fn far_tile_in_ring(base: VoxelScale, level: u8, dist: f64, hz: &HorizonConfig) -> bool {
    let inner = hz.inner(level) - far_tile_m(base, level);
    (inner..hz.outer(level)).contains(&dist)
}

/// All level-L surface tiles wanted around a viewer (2D ring by horizontal
/// distance, with the one-tile inner overlap of [`far_tile_in_ring`]). Pure, so
/// a headless bench measures the same set the app streams.
pub fn wanted_far_tiles(
    base: VoxelScale,
    viewer_m: DVec3,
    level: u8,
    hz: &HorizonConfig,
) -> Vec<(i32, i32)> {
    let tile_m = far_tile_m(base, level);
    let outer = hz.outer(level);
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
                hz,
            ) {
                out.push((tx, tz));
            }
        }
    }
    out
}

/// The far tile's transform: tile origin (meters) minus the floating origin,
/// plus the UNIFORM-per-level depth push. **No sink** — the stepped columns are
/// FLOOR-quantized ([`quantize_top`]) so a far top already sits at or below the
/// near surface; the near volumetric terrain wins the overlap band with no
/// downward bias. The push survives (corrections #1): overlapping LOD rings still
/// present coplanar faces where their quantized tops coincide, and draw order
/// cannot resolve coplanar z-fight — only a real depth offset can. `forward` is
/// the camera-forward axis (unit, shared by every tile this frame); each level is
/// pushed `DEPTH_PUSH_FRAC` of its coarse voxel along it, so every tile of a
/// level moves by the identical vector — same-level shared edges cannot separate
/// (corrections #11, the seam fix) — while deeper (coarser) rings push farther
/// and sit behind nearer ones in the lap band.
fn far_tile_translation(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    y_ref_m: f64,
    forward: DVec3,
    origin_m: DVec3,
) -> Vec3 {
    let cvs = coarse_scale(base, level).voxel_size_m();
    let tile_m = cvs * f64::from(CHUNK_SIZE);
    let min = DVec3::new(f64::from(tx) * tile_m, y_ref_m, f64::from(tz) * tile_m);
    let away = level_depth_push(base, level, forward);
    to_render(min + away - origin_m)
}

/// Interior cell count per tile side (32 coarse columns).
const TILE_CELLS: usize = CHUNK_SIZE as usize;

/// Sample a tile's extended column grid: composed [`ColumnSpan`] **stacks** and
/// near-cover cull flags over indices −1..=`TILE_CELLS` on each axis (an (N+2)²
/// grid). The interior 0..`TILE_CELLS` is what the tile emits; the −1 /
/// `TILE_CELLS` ring is neighbour data for watertight boundary side faces (a
/// same-level neighbour tile samples the shared column identically — including
/// its reduced nodes, which are keyed by world node coords — so the steps
/// match). `sample(wx, wz)` returns the coarse authority's summary (surface
/// height in base voxels, surface block); `known(wx, wz)` returns the
/// fully-inserted reduced node answers over that column (empty = synthesize —
/// never "air", the A-5 rule). Composition is [`compose_column`]: reduced
/// nodes lay over the synthesized top sheet, which stands wherever the world
/// was never generated (the default case, forever) and wherever a node's
/// subtree is not resident. Resident reduced spans are first floored
/// geometry-safe ([`floor_known_surface`]) so they stay under the near ground —
/// which is what retired the old per-column distance standoff (journal/0091).
// The derivation boundary takes its world by parameter (pure-input rule);
// bundling them into a struct would only obscure the data flow.
#[allow(clippy::too_many_arguments)]
fn tile_column_stacks(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    viewer_m: DVec3,
    hz: &HorizonConfig,
    sample: &dyn Fn(i64, i64) -> (i32, Block),
    known: &mut dyn FnMut(i64, i64) -> Vec<(i32, i32, Vec<ColumnSpan>)>,
) -> (Vec<Vec<ColumnSpan>>, Vec<bool>) {
    let stride = level_stride(level);
    let base_vs = base.voxel_size_m();
    let m = TILE_CELLS + 2;
    let mut stacks = vec![Vec::new(); m * m];
    let mut culled = vec![false; m * m];
    for gj in -1..=TILE_CELLS as i64 {
        for gi in -1..=TILE_CELLS as i64 {
            let wx = (i64::from(tx) * TILE_CELLS as i64 + gi) * stride;
            let wz = (i64::from(tz) * TILE_CELLS as i64 + gj) * stride;
            let (h, block) = sample(wx, wz);
            let top = quantize_top(h, stride);
            let synth = ColumnSpan {
                top,
                bottom: FAR_BOTTOM_UNBOUNDED,
                block,
            };
            let k = (gj + 1) as usize * m + (gi + 1) as usize;
            let (wx_m, wz_m) = (wx as f64 * base_vs, wz as f64 * base_vs);
            let top_m = f64::from(top) * base_vs;
            culled[k] = near_covers(wx_m, wz_m, top_m, viewer_m, hz);
            // Warm-where-resident, cold-only-where-not (journal/0091, the standoff
            // removal). `known` returns reduced spans ONLY for nodes whose subtree
            // is fully inserted (the A-5 guard, `known_node_grids`); a partial or
            // never-visited node returns empty and `compose_column` falls back to
            // the synthesized top sheet — never air. No per-column distance gate:
            // the reduced surface is floored geometry-safe (`floor_known_surface`)
            // so it physically cannot poke through the near ground, which is what
            // let the 176 m standoff — and the band inversion it caused — go.
            let resident = floor_known_surface(top, &known(wx, wz));
            stacks[k] = compose_column(synth, &resident);
        }
    }
    (stacks, culled)
}

/// Build one far tile's VOXEL-STEPPED mesh (FF2a → FF2b) — a **pure function**
/// of plain data: the extended column span **stacks** (an (N+2)² grid: interior
/// 0..N plus a one-cell neighbour ring), the near-coverage `culled` mask over
/// the same grid, and `ring_edges` (per tile side: does it face a coarser ring /
/// world rim?). No ECS/world/authority read happens here — coverage and the
/// reduced-node composition are passed in as data (coordinator amendment 1), so
/// async far-meshing (a filed follow-on) is a drop-in and the mesher is
/// trivially testable.
///
/// Per span it emits: a greedy-merged top face for each column's TOPMOST span
/// (byte-identical to FF2a over single-span stacks — the whole synthesized
/// field), individual top faces for deeper spans, a bottom face where a span's
/// underside is exposed (finite `bottom` — overhangs/bridges from reduced
/// nodes; [`FAR_BOTTOM_UNBOUNDED`] means "the ground keeps going" and never
/// draws), and side walls for the parts of a span's extent the neighbour's
/// stack leaves uncovered (interval subtraction, [`subtract_neighbor_cover`] —
/// exposure is disjoint between the two sides of a shared plane, so walls are
/// emitted once and same-level seams stay watertight, the FF2a theorem
/// generalized). Ring-transition and near-seam skirts are unchanged. Returns
/// the mesh and the tile's y-reference (meters) — positions are tile-local so
/// the absolute height rides in the transform (floating-origin discipline).
/// Order of `ring_edges` matches the side-face direction order: +X, −X, +Z, −Z.
fn build_far_tile_mesh(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    stacks: &[Vec<ColumnSpan>],
    culled: &[bool],
    ring_edges: [bool; 4],
) -> (MeshData, f64) {
    let stride = level_stride(level);
    let base_vs = base.voxel_size_m();
    let n = TILE_CELLS;
    let m = n + 2;
    debug_assert_eq!(stacks.len(), m * m);
    debug_assert_eq!(culled.len(), m * m);
    let idx = |i: i64, j: i64| -> usize { (j + 1) as usize * m + (i + 1) as usize };

    let ox = i64::from(tx) * n as i64 * stride;
    let oz = i64::from(tz) * n as i64 * stride;

    // y_ref = the lowest emitted face height among unculled interior columns
    // (span tops, and finite span bottoms — overhang undersides reach below
    // their tops). Skirts may dip below it: y_ref is a reference for f32
    // precision, not a bound. If every column is culled, the tile draws nothing.
    let mut min_y: Option<i32> = None;
    for j in 0..n as i64 {
        for i in 0..n as i64 {
            if culled[idx(i, j)] {
                continue;
            }
            for s in &stacks[idx(i, j)] {
                let low = if s.bottom == FAR_BOTTOM_UNBOUNDED {
                    s.top
                } else {
                    s.bottom
                };
                min_y = Some(min_y.map_or(low, |v| v.min(low)));
            }
        }
    }
    let Some(min_y) = min_y else {
        return (MeshData::default(), 0.0);
    };
    let y_ref = f64::from(min_y) * base_vs;

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

    // --- Greedy-merged top faces (topmost span per column) ------------------
    // Merging keys on the top face itself (plane + block) — deeper structure
    // does not change a top face, so single-span synthesized fields merge
    // exactly as FF2a did.
    let surf = |i: i64, j: i64| -> Option<(i32, Block)> {
        stacks[idx(i, j)].first().map(|s| (s.top, s.block))
    };
    let mut consumed = vec![false; n * n];
    for j0 in 0..n {
        for i0 in 0..n {
            if consumed[j0 * n + i0] || culled[idx(i0 as i64, j0 as i64)] {
                continue;
            }
            let Some(key0) = surf(i0 as i64, j0 as i64) else {
                continue;
            };
            let mut w = 1;
            while i0 + w < n
                && !consumed[j0 * n + i0 + w]
                && !culled[idx((i0 + w) as i64, j0 as i64)]
                && surf((i0 + w) as i64, j0 as i64) == Some(key0)
            {
                w += 1;
            }
            let mut h = 1;
            'grow: while j0 + h < n {
                for k in 0..w {
                    let (ii, jj) = ((i0 + k) as i64, (j0 + h) as i64);
                    if consumed[(j0 + h) * n + i0 + k]
                        || culled[idx(ii, jj)]
                        || surf(ii, jj) != Some(key0)
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
            let ty = f64::from(key0.0);
            // +Y top, ccw seen from above (matches the near mesher's +Y winding).
            push_quad(
                [
                    [x0 as f64, ty, z0 as f64],
                    [x0 as f64, ty, z1 as f64],
                    [x1 as f64, ty, z1 as f64],
                    [x1 as f64, ty, z0 as f64],
                ],
                [0, 1, 0],
                key0.1,
            );
        }
    }

    // --- Deeper-span top faces + exposed bottom faces -----------------------
    // Deeper spans (overhangs/bridges from reduced nodes) are sparse relative
    // to the field, so their faces go out per-cell without greedy merging (a
    // filed polish item, journal/0070).
    for j in 0..n as i64 {
        for i in 0..n as i64 {
            if culled[idx(i, j)] {
                continue;
            }
            let (x0, x1) = ((ox + i * stride) as f64, (ox + (i + 1) * stride) as f64);
            let (z0, z1) = ((oz + j * stride) as f64, (oz + (j + 1) * stride) as f64);
            for (si, s) in stacks[idx(i, j)].iter().enumerate() {
                if si > 0 {
                    let ty = f64::from(s.top);
                    push_quad(
                        [[x0, ty, z0], [x0, ty, z1], [x1, ty, z1], [x1, ty, z0]],
                        [0, 1, 0],
                        s.block,
                    );
                }
                if s.bottom != FAR_BOTTOM_UNBOUNDED {
                    let by = f64::from(s.bottom);
                    // −Y underside, ccw seen from below.
                    push_quad(
                        [[x0, by, z0], [x1, by, z0], [x1, by, z1], [x0, by, z1]],
                        [0, -1, 0],
                        s.block,
                    );
                }
            }
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
    let mut pieces: Vec<(i32, i32)> = Vec::new();
    for j in 0..n as i64 {
        for i in 0..n as i64 {
            if culled[idx(i, j)] {
                continue;
            }
            let stack = &stacks[idx(i, j)];
            for (di, dj, normal, edge_ix) in dirs {
                let (ni, nj) = (i + di, j + dj);
                let neighbor = &stacks[idx(ni, nj)];
                let neighbor_culled = culled[idx(ni, nj)];
                let out_of_tile = !(0..n as i64).contains(&ni) || !(0..n as i64).contains(&nj);
                let ring_edge = out_of_tile && ring_edges[edge_ix];

                for (si, s) in stack.iter().enumerate() {
                    pieces.clear();
                    if neighbor_culled {
                        // Near/far seam: the near field covers the neighbour, so
                        // drop a skirt from this span's top to hide the handoff
                        // crack (never step *up* to it); a bounded span (an
                        // overhang) walls its full extent.
                        let bottom = if s.bottom == FAR_BOTTOM_UNBOUNDED {
                            s.top - skirt
                        } else {
                            s.bottom
                        };
                        pieces.push((bottom, s.top));
                    } else if ring_edge && si == 0 {
                        // Ring/world outer boundary: down to the neighbour, then
                        // a skirt below to cover the coarser ring's differing
                        // stride (FF2a rule, on the surface span).
                        let neighbor_top = neighbor.first().map_or(s.top, |ns| ns.top);
                        pieces.push((neighbor_top.min(s.top) - skirt, s.top));
                    } else {
                        // Ordinary exposed step, generalized to stacks: the
                        // parts of this span's extent the neighbour's stack
                        // leaves uncovered. Exposure is disjoint between the
                        // two sides of a shared plane, so every wall is emitted
                        // once — single-span stacks reduce to FF2a's "taller
                        // column walls down to the shorter".
                        subtract_neighbor_cover(s, neighbor, skirt, &mut pieces);
                    }
                    for &(ylo_i, yhi_i) in &pieces {
                        if yhi_i <= ylo_i {
                            continue;
                        }
                        let (cx0, cx1) = ((ox + i * stride) as f64, (ox + (i + 1) * stride) as f64);
                        let (cz0, cz1) = ((oz + j * stride) as f64, (oz + (j + 1) * stride) as f64);
                        let (ylo, yhi) = (f64::from(ylo_i), f64::from(yhi_i));
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
                        push_quad(corners, normal, s.block);
                    }
                }
            }
        }
    }

    (mesh, y_ref)
}

/// The vertical intervals of `span` a neighbour column's stack leaves exposed
/// — interval subtraction over the neighbour's solid runs, in i64 so the
/// [`FAR_BOTTOM_UNBOUNDED`] sentinel needs no special cases. A piece left open
/// at the bottom (a neighbour with no unbounded ground span, which
/// [`compose_column`] never produces) is defensively clamped to one skirt
/// below its own top rather than emitting a wall to the abyss.
fn subtract_neighbor_cover(
    span: &ColumnSpan,
    neighbor: &[ColumnSpan],
    skirt: i32,
    out: &mut Vec<(i32, i32)>,
) {
    let lo = |b: i32| -> i64 {
        if b == FAR_BOTTOM_UNBOUNDED {
            i64::MIN
        } else {
            i64::from(b)
        }
    };
    let mut pieces: Vec<(i64, i64)> = vec![(lo(span.bottom), i64::from(span.top))];
    for nspan in neighbor {
        let (nb, nt) = (lo(nspan.bottom), i64::from(nspan.top));
        let mut next = Vec::with_capacity(pieces.len() + 1);
        for (pb, pt) in pieces {
            if nt <= pb || nb >= pt {
                next.push((pb, pt));
                continue;
            }
            if pb < nb {
                next.push((pb, nb));
            }
            if nt < pt {
                next.push((nt, pt));
            }
        }
        pieces = next;
    }
    for (pb, pt) in pieces {
        if pt <= pb {
            continue;
        }
        let top = pt as i32;
        let bottom = if pb <= i64::from(i32::MIN) {
            top - skirt
        } else {
            pb as i32
        };
        out.push((bottom, top));
    }
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

/// Whether a tile is close enough to the viewer that its column derivation is
/// viewer-relative — the near-coverage cull could gate some of its columns — and
/// must therefore be refreshed as the viewer moves. Measured against the near
/// field's own reach (the ladder's unload radius, `>= near_cover`, journal/0091):
/// this is the band the near field draws into, so the tiles that lap it must
/// refresh their coverage cull, and it is also what walks freshly-resident
/// reduced data in behind a moving player — the tiles just outside the near field
/// re-derive on the next near-chunk crossing and pick up newly reduced nodes.
/// (With the reduction standoff retired the two bounds collapse into one
/// ladder-derived reach.)
fn tile_in_cull_band(
    base: VoxelScale,
    level: u8,
    tx: i32,
    tz: i32,
    viewer: DVec3,
    hz: &HorizonConfig,
) -> bool {
    far_tile_center_dist(base, level, tx, tz, viewer)
        < hz.near_cover_r_m().max(hz.unload_radius_m()) + far_tile_m(base, level)
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
    authority: Res<Authority>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    horizon: Res<HorizonConfig>,
    mut map: ResMut<FarSurfaceMap>,
    mut pyramid: ResMut<FarPyramid>,
    mut tasks: ResMut<FarMeshTasks>,
) {
    // Only the worldgen authority has a coarse summary; tear our tiles down when
    // the S1 authority is active (keys 3/4). Also drop any in-flight mesh tasks.
    if !authority.far_field_is_worldgen() {
        if !map.loaded.is_empty() {
            for (_, t) in map.loaded.drain() {
                if let Some(e) = t.entity {
                    commands.entity(e).despawn();
                }
            }
        }
        tasks.0.clear();
        return;
    }

    let base = scale.scale;
    // Scale switched: tiles are per-scale, rebuild them (and drop in-flight
    // tasks — their derived data is for the old scale).
    if scale.is_changed() && !map.loaded.is_empty() {
        for (_, t) in map.loaded.drain() {
            if let Some(e) = t.entity {
                commands.entity(e).despawn();
            }
        }
        tasks.0.clear();
    }

    let viewer = player.pos_m;
    let hz = &*horizon;
    let cur_chunk = viewer_near_chunk(base, viewer);

    // Unload tiles that left their ring (horizontal distance, hysteresis). The
    // inner bound matches the one-tile ring lap of `far_tile_in_ring` so lapped
    // tiles don't load-then-immediately-unload (journal/0022 walk 17).
    let left_ring = |level: u8, tx: i32, tz: i32| {
        let d = far_tile_center_dist(base, level, tx, tz, viewer);
        let inner = hz.inner(level) - far_tile_m(base, level);
        d < inner - FAR_UNLOAD_SLACK_M || d > hz.outer(level) + FAR_UNLOAD_SLACK_M
    };
    let to_unload: Vec<(u8, i32, i32)> = map
        .loaded
        .keys()
        .filter(|(level, tx, tz)| left_ring(*level, *tx, *tz))
        .copied()
        .collect();
    for key in to_unload {
        if let Some(t) = map.loaded.remove(&key)
            && let Some(e) = t.entity
        {
            commands.entity(e).despawn();
        }
    }
    // Cancel in-flight tile tasks whose tile left the ring before its mesh was
    // ready (dropping the `Task` detaches it) — otherwise the drain could try to
    // despawn an old entity the unload sweep already removed.
    tasks
        .0
        .retain(|(level, tx, tz), _| !left_ring(*level, *tx, *tz));

    // Newly-wanted (missing) tiles, nearest first — these appear the horizon. A
    // tile already meshing off-thread is neither loaded nor missing.
    let mut missing: Vec<(u64, u8, i32, i32)> = Vec::new();
    for level in 1..=4u8 {
        for (tx, tz) in wanted_far_tiles(base, viewer, level, hz) {
            if !map.loaded.contains_key(&(level, tx, tz)) && !tasks.0.contains_key(&(level, tx, tz))
            {
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
    // invisible, so the horizon (missing) takes priority. A tile already being
    // rebuilt off-thread is skipped (it's in `tasks`).
    let mut stale: Vec<(u64, u8, i32, i32)> = Vec::new();
    for (&(level, tx, tz), t) in &map.loaded {
        if tasks.0.contains_key(&(level, tx, tz)) {
            continue;
        }
        let in_band = tile_in_cull_band(base, level, tx, tz, viewer, hz);
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

    let ring_edges_of = |level: u8, tx: i32, tz: i32| -> [bool; 4] {
        let edge = |ntx: i32, ntz: i32| {
            !far_tile_in_ring(
                base,
                level,
                far_tile_center_dist(base, level, ntx, ntz, viewer),
                hz,
            )
        };
        [
            edge(tx + 1, tz),
            edge(tx - 1, tz),
            edge(tx, tz + 1),
            edge(tx, tz - 1),
        ]
    };
    // Snapshot the reduced-node grids the tile needs on the FRAME thread — the
    // ONLY impure input the derive reads besides the generator, and the one that
    // cannot cross to a task (`known_node_grids` takes `&mut FarPyramid`, a
    // main-thread Bevy resource). A tile's 34² columns touch at most the 3×3 node
    // plan columns centred on `(tx, tz)` (module note above), so this pre-fills
    // exactly the memoization set the on-thread `known` closure built lazily —
    // NOT a re-derivation of the coordinate logic, just the same call set. The
    // grids are `Arc`-shared, cheap to move. `far_tile.snapshot` is the residual
    // frame-thread cost; the generator sampling (the bulk of the old
    // `far_tile.derive`) now runs off-thread (journal/0084).
    let snapshot = |pyramid: &mut FarPyramid, level: u8, tx: i32, tz: i32| -> NodeSnapshot {
        let _perf = crate::perf_span!("far_tile.snapshot");
        let mut snap: NodeSnapshot = HashMap::new();
        for nz in (tz - 1)..=(tz + 1) {
            for nx in (tx - 1)..=(tx + 1) {
                snap.insert((nx, nz), pyramid.known_node_grids(level, nx, nz));
            }
        }
        snap
    };

    let hz_owned = *hz;
    // The shared `Arc<Pregen>` each tile task mints its OWN generator from (always
    // present here — this system runs only under the worldgen authority).
    let pregen = authority
        .worldgen_pregen()
        .expect("worldgen far-field runs only under the worldgen authority");
    let pool = AsyncComputeTaskPool::get();
    // Spawn a task that DERIVES the tile off-thread — a per-task `WorldGenerator`
    // (minted from the shared pregen, byte-identical to it, no `Mutex` contention)
    // for the coarse-surface sampling, plus the frame-thread node snapshot for
    // reduced data — and then meshes it. `far_tile.derive` (the generator
    // sampling) and `far_tile.mesh` now record on the task thread and drop out of
    // the frame `schedule` envelope (journal/0084). `old_entity` is the seam
    // tile's current entity, despawned by the drain once the fresh mesh is ready.
    let spawn_tile = |tasks: &mut FarMeshTasks,
                      level: u8,
                      tx: i32,
                      tz: i32,
                      node_snapshot: NodeSnapshot,
                      ring_edges: [bool; 4],
                      cull_chunk: Option<(i64, i64, i64)>,
                      pregen: Arc<Pregen>,
                      old_entity: Option<Entity>| {
        // Owned copies for the `async move` block (all `Copy`).
        let (base, viewer, hz_owned) = (base, viewer, hz_owned);
        let task = pool.spawn(async move {
            // Churn instrument (journal/0051): a task-thread wall over the derive +
            // mesh + vertex-buffer conversion.
            let build_start = std::time::Instant::now();
            let _build = crate::perf_span!("far_tile.build");
            // Per-task generator, minted off-thread from the shared pregen. Its
            // construction rides on THIS task thread, never the frame thread.
            let generator = RefCell::new(WorldGenerator::new_owned(pregen));
            let sample = |wx: i64, wz: i64| generator.borrow_mut().coarse_surface(wx, wz);
            let stride = level_stride(level);
            // `known` reads the frame-thread snapshot (the complete 3×3 node
            // columns the tile can touch), so no pyramid access crosses threads.
            let mut known = |wx: i64, wz: i64| -> Vec<(i32, i32, Vec<ColumnSpan>)> {
                let (cvx, cvz) = (wx.div_euclid(stride), wz.div_euclid(stride));
                let (nx, nz) = (cvx.div_euclid(32) as i32, cvz.div_euclid(32) as i32);
                let (lx, lz) = (cvx.rem_euclid(32) as usize, cvz.rem_euclid(32) as usize);
                node_snapshot
                    .get(&(nx, nz))
                    .map(|grids| {
                        grids
                            .iter()
                            .map(|(ny, grid)| {
                                let ext = 32 * stride as i32;
                                (ny * ext, (ny + 1) * ext, grid[lz * 32 + lx].clone())
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let (stacks, culled) = {
                let _perf = crate::perf_span!("far_tile.derive");
                tile_column_stacks(base, level, tx, tz, viewer, &hz_owned, &sample, &mut known)
            };
            let (mesh_data, y_ref) = {
                let _perf = crate::perf_span!("far_tile.mesh");
                build_far_tile_mesh(base, level, tx, tz, &stacks, &culled, ring_edges)
            };
            let mesh = (!mesh_data.is_empty()).then(|| to_bevy_mesh(mesh_data));
            churn::record(&churn::FAR_MESHES, &churn::FAR_NANOS, build_start);
            FarMeshOutput {
                level,
                tx,
                tz,
                mesh,
                y_ref,
                cull_chunk,
                old_entity,
            }
        });
        tasks.0.insert((level, tx, tz), task);
    };

    // Bound spawns by the in-flight cap so a meshing backlog can't grow unbounded.
    let mut budget =
        FAR_SURFACE_BUDGET_PER_FRAME.min(MAX_INFLIGHT_FAR.saturating_sub(tasks.0.len()));
    for (_, level, tx, tz) in missing {
        if budget == 0 {
            break;
        }
        let node_snapshot = snapshot(&mut pyramid, level, tx, tz);
        let ring_edges = ring_edges_of(level, tx, tz);
        let cull_chunk = tile_in_cull_band(base, level, tx, tz, viewer, hz).then_some(cur_chunk);
        spawn_tile(
            &mut tasks,
            level,
            tx,
            tz,
            node_snapshot,
            ring_edges,
            cull_chunk,
            pregen.clone(),
            None,
        );
        budget -= 1;
    }
    for (_, level, tx, tz) in stale {
        if budget == 0 {
            break;
        }
        let old = map.loaded.get(&(level, tx, tz)).and_then(|t| t.entity);
        let node_snapshot = snapshot(&mut pyramid, level, tx, tz);
        let ring_edges = ring_edges_of(level, tx, tz);
        let cull_chunk = tile_in_cull_band(base, level, tx, tz, viewer, hz).then_some(cur_chunk);
        spawn_tile(
            &mut tasks,
            level,
            tx,
            tz,
            node_snapshot,
            ring_edges,
            cull_chunk,
            pregen.clone(),
            old,
        );
        budget -= 1;
    }
}

/// Drain finished far-tile mesh tasks: despawn the seam tile's old entity (if a
/// rebuild), insert the new `Mesh`, spawn the already-positioned tile entity, and
/// record the `LoadedFarTile` bookkeeping (journal/0083). A tile appears — or a
/// seam refresh swaps in — a frame or more after its task was spawned; that lag
/// is the only observable change, and the tile mesh is byte-identical to the
/// synchronous path.
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn drain_far_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    scale: Res<CurrentScale>,
    origin: Res<FloatingOrigin>,
    player: Res<Player>,
    mut map: ResMut<FarSurfaceMap>,
    mut tasks: ResMut<FarMeshTasks>,
) {
    let base = scale.scale;
    // One shared camera-forward push direction for every tile this frame — the
    // uniform-per-level push that keeps same-level seams closed (corrections #11).
    let forward = player.view_dir();
    for out in crate::meshtasks::drain_finished(&mut tasks.0) {
        if let Some(old) = out.old_entity {
            commands.entity(old).despawn();
        }
        let entity = out.mesh.map(|bevy_mesh| {
            let transform = Transform::from_translation(far_tile_translation(
                base, out.level, out.tx, out.tz, out.y_ref, forward, origin.0,
            ));
            let mut ent = commands.spawn((
                Mesh3d(meshes.add(bevy_mesh)),
                FarTileEntity {
                    level: out.level,
                    tx: out.tx,
                    tz: out.tz,
                    y_ref_m: out.y_ref,
                },
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
            (out.level, out.tx, out.tz),
            LoadedFarTile {
                entity,
                cull_chunk: out.cull_chunk,
            },
        );
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
    // One shared camera-forward push direction for every tile this frame — the
    // uniform-per-level push that keeps same-level seams closed (corrections #11).
    let forward = player.view_dir();
    for (tile, mut transform) in &mut tiles {
        transform.translation = far_tile_translation(
            scale.scale,
            tile.level,
            tile.tx,
            tile.tz,
            tile.y_ref_m,
            forward,
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
        let hz = &HorizonConfig::default();
        assert_eq!(level_for_distance(hz, 0.0), None, "full detail");
        assert_eq!(level_for_distance(hz, 111.0), None, "still full detail");
        assert_eq!(
            level_for_distance(hz, 113.0),
            Some(1),
            "overlap band is LOD 1"
        );
        assert_eq!(level_for_distance(hz, 255.0), Some(1));
        assert_eq!(level_for_distance(hz, 256.0), Some(2));
        assert_eq!(level_for_distance(hz, 511.0), Some(2));
        assert_eq!(level_for_distance(hz, 512.0), Some(3));
        assert_eq!(level_for_distance(hz, 1023.0), Some(3));
        assert_eq!(level_for_distance(hz, 1024.0), Some(4));
        assert_eq!(level_for_distance(hz, 1199.0), Some(4));
        assert_eq!(level_for_distance(hz, 1200.0), None, "beyond the far field");
    }

    /// **The default must not change** (journal/0042). `--horizon` made the ring
    /// geometry runtime state; this pins the no-flag path to the literal
    /// constants the `const RING_EDGES_M` shipped, so a launch without the flag
    /// renders exactly as before.
    #[test]
    fn default_ring_edges_match_the_shipped_constants() {
        let hz = HorizonConfig::default();
        assert_eq!(
            hz.ring_edges,
            [112.0, 256.0, 512.0, 1024.0, 1200.0],
            "the shipped RING_EDGES_M, exactly"
        );
        assert_eq!(hz.ladder.far_max_m, 1200.0);
        assert_eq!(hz.near_cover_r_m(), 112.0, "the shipped NEAR_COVER_R_M");
        assert_eq!(hz.camera_far_m(), 3000.0, "the shipped camera far plane");
        assert_eq!(hz.fog_range_m(), (150.0, 1100.0), "the shipped fog range");
        // Asking for the default horizon explicitly is the same object.
        assert_eq!(HorizonConfig::with_far_max(DEFAULT_FAR_MAX_M), hz);
    }

    /// **The ladder is the single source of truth** (journal/0091): the ring
    /// edges, the near-coverage radius, and the near-field load/unload radii all
    /// DERIVE from the one [`LodLadder`] — no LOD range is defined in two places
    /// that could drift (the § 6.3 constant-coupling defect). This pins the
    /// derivations so a future edit to the ladder moves *everything* together,
    /// and a stray hardcoded radius somewhere else fails loudly.
    #[test]
    fn every_lod_range_derives_from_the_one_ladder() {
        let l = LodLadder::DEFAULT;
        // Ring edges = border + each step-past-border (LOD-1 at the border).
        assert_eq!(
            l.ring_edges(),
            [
                l.nearfield_border_m,
                l.nearfield_border_m + l.step_past_border_m[0],
                l.nearfield_border_m + l.step_past_border_m[1],
                l.nearfield_border_m + l.step_past_border_m[2],
                l.far_max_m,
            ],
            "ring edges are border + steps-past-border"
        );
        // Near radii are border-relative, one definition each.
        assert_eq!(l.near_cover_r_m(), l.nearfield_border_m);
        assert_eq!(
            l.load_radius_m(),
            l.nearfield_border_m + l.overlap_past_border_m
        );
        assert_eq!(l.unload_radius_m(), l.load_radius_m() + l.unload_slack_m);
        // The shipped absolute values, so the defaults are pinned in one spot.
        assert_eq!(l.near_cover_r_m(), 112.0);
        assert_eq!(l.load_radius_m(), 128.0);
        assert_eq!(l.unload_radius_m(), 160.0);
        // The back-compat consts and the streaming radii read the SAME ladder —
        // proof there is no second copy of these numbers anywhere.
        assert_eq!(FULL_DETAIL_RADIUS_M, l.load_radius_m());
        assert_eq!(FAR_OVERLAP_M, l.overlap_past_border_m);
        assert_eq!(crate::streaming::UNLOAD_RADIUS_M, l.unload_radius_m());
        // And the horizon config's derived edges equal the ladder's own.
        assert_eq!(HorizonConfig::default().ring_edges, l.ring_edges());
    }

    /// A wider horizon stretches the OUTERMOST ring and leaves the inner three
    /// exactly where they were — the cheap direction (see [`HorizonConfig`]).
    #[test]
    fn horizon_stretches_only_the_outer_ring() {
        for km in [3.0, 5.0, 10.0] {
            let hz = HorizonConfig::with_far_max(km * 1000.0);
            assert_eq!(
                &hz.ring_edges[..4],
                &[112.0, 256.0, 512.0, 1024.0],
                "inner rings must not move at {km} km"
            );
            assert_eq!(hz.ring_edges[4], km * 1000.0);
            // Still 5 edges / 4 levels, monotone, and the whole band is claimed.
            assert!(hz.ring_edges.windows(2).all(|w| w[0] <= w[1]));
            assert_eq!(level_for_distance(&hz, km * 1000.0 - 1.0), Some(4));
            assert_eq!(level_for_distance(&hz, km * 1000.0), None);
        }
        // An absurdly SHORT horizon collapses the inner edges instead of
        // inverting them (monotone by clamp), and never goes below the LOD-1
        // inner edge.
        let tiny = HorizonConfig::with_far_max(0.0);
        assert!(tiny.ring_edges.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(tiny.ring_edges, [112.0; 5]);
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
        let hz = &HorizonConfig::default();
        let a = wanted_far_positions(base, viewer, 1, hz);
        let b = wanted_far_positions(base, viewer, 1, hz);
        assert_eq!(a, b, "deterministic");
        assert!(!a.is_empty());
        for pos in &a {
            let d = (far_chunk_center_m(base, 1, *pos) - viewer).length();
            assert!(
                (hz.ring_edges[0]..hz.ring_edges[1]).contains(&d),
                "chunk at distance {d} outside LOD-1 ring"
            );
        }
    }

    #[test]
    fn wanted_far_tiles_are_ring_shaped_and_deterministic() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let viewer = DVec3::new(30.0, 980.0, -15.0);
        let hz = &HorizonConfig::default();
        let a = wanted_far_tiles(base, viewer, 2, hz);
        let b = wanted_far_tiles(base, viewer, 2, hz);
        assert_eq!(a, b, "deterministic");
        assert!(!a.is_empty());
        // The ring band carries the one-tile inner overlap (`far_tile_in_ring`):
        // level-2 tiles lap one L2 tile inward past ring edge 1 to close the
        // seam with L1 (journal/0022 walk 17).
        let inner = hz.ring_edges[1] - far_tile_m(base, 2);
        for &(tx, tz) in &a {
            let d = far_tile_center_dist(base, 2, tx, tz, viewer);
            assert!(
                (inner..hz.ring_edges[2]).contains(&d),
                "tile at horizontal distance {d} outside LOD-2 ring (inner {inner})"
            );
        }
    }

    /// No reduced nodes anywhere: the pure-synthesis (default) case.
    fn no_known(_wx: i64, _wz: i64) -> Vec<(i32, i32, Vec<ColumnSpan>)> {
        Vec::new()
    }

    /// Mesh a tile for tests: derive stacks + coverage from `sample`/`viewer`
    /// (no reduced nodes), then run the pure mesher. `ring_edges` all false =
    /// an interior tile with same-level neighbours on every side.
    fn mesh_tile(
        base: VoxelScale,
        level: u8,
        tx: i32,
        tz: i32,
        viewer: DVec3,
        ring_edges: [bool; 4],
        sample: &dyn Fn(i64, i64) -> (i32, Block),
    ) -> (MeshData, f64) {
        let hz = HorizonConfig::default();
        let (stacks, culled) =
            tile_column_stacks(base, level, tx, tz, viewer, &hz, sample, &mut no_known);
        build_far_tile_mesh(base, level, tx, tz, &stacks, &culled, ring_edges)
    }

    /// A viewer far enough that `near_covers` never fires — isolates meshing from
    /// the coverage cull.
    const FAR_VIEWER: DVec3 = DVec3::new(1.0e6, 1.0e6, 1.0e6);

    /// journal/0084: the far derive moves off-thread — a per-task
    /// [`WorldGenerator`] for the coarse-surface sampling plus a frame-thread
    /// snapshot of the [`FarPyramid`]'s known node grids (the `&mut FarPyramid`
    /// resource cannot cross to a task). Prove the offloaded inputs reproduce the
    /// on-thread [`tile_column_stacks`] byte-for-byte: a second generator over the
    /// same `Arc<Pregen>` samples identically, and the 3×3 node snapshot the frame
    /// thread pre-fills equals the lazy `known_node_grids` the on-thread `known`
    /// closure calls (it is the SAME call, not a re-derivation). The synthesis
    /// (empty-pyramid) case is the default forever, so it is what is exercised.
    #[test]
    fn offloaded_far_derive_matches_on_thread() {
        use dc_worldgen::{Extent, Pregen, WorldParams};
        let pregen = Arc::new(Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        }));
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let hz = HorizonConfig::default();
        // High above the surface so nothing is coverage-culled (the full stepped
        // geometry is derived — a non-vacuous tile).
        let viewer = DVec3::new(30.0, 5000.0, -15.0);
        let (level, tx, tz) = (2u8, 1i32, -1i32);
        let stride = level_stride(level);
        let mut pyramid = FarPyramid::default();

        // On-thread reference: one generator's coarse_surface + lazy pyramid known.
        let shared = RefCell::new(WorldGenerator::new_owned(pregen.clone()));
        let sample_shared = |wx: i64, wz: i64| shared.borrow_mut().coarse_surface(wx, wz);
        let mut node_cache: HashMap<(i32, i32), NodeGrids> = HashMap::new();
        let mut known_pyr = |wx: i64, wz: i64| -> Vec<(i32, i32, Vec<ColumnSpan>)> {
            let (cvx, cvz) = (wx.div_euclid(stride), wz.div_euclid(stride));
            let (nx, nz) = (cvx.div_euclid(32) as i32, cvz.div_euclid(32) as i32);
            let (lx, lz) = (cvx.rem_euclid(32) as usize, cvz.rem_euclid(32) as usize);
            node_cache
                .entry((nx, nz))
                .or_insert_with(|| pyramid.known_node_grids(level, nx, nz))
                .iter()
                .map(|(ny, grid)| {
                    let ext = 32 * stride as i32;
                    (ny * ext, (ny + 1) * ext, grid[lz * 32 + lx].clone())
                })
                .collect()
        };
        let (stacks_a, culled_a) = tile_column_stacks(
            base,
            level,
            tx,
            tz,
            viewer,
            &hz,
            &sample_shared,
            &mut known_pyr,
        );

        // Offloaded path: a per-task generator + the frame-thread 3×3 snapshot.
        let mut snap: NodeSnapshot = HashMap::new();
        for nz in (tz - 1)..=(tz + 1) {
            for nx in (tx - 1)..=(tx + 1) {
                snap.insert((nx, nz), pyramid.known_node_grids(level, nx, nz));
            }
        }
        let per_task = RefCell::new(WorldGenerator::new_owned(pregen.clone()));
        let sample_task = |wx: i64, wz: i64| per_task.borrow_mut().coarse_surface(wx, wz);
        let mut known_snap = |wx: i64, wz: i64| -> Vec<(i32, i32, Vec<ColumnSpan>)> {
            let (cvx, cvz) = (wx.div_euclid(stride), wz.div_euclid(stride));
            let (nx, nz) = (cvx.div_euclid(32) as i32, cvz.div_euclid(32) as i32);
            let (lx, lz) = (cvx.rem_euclid(32) as usize, cvz.rem_euclid(32) as usize);
            snap.get(&(nx, nz))
                .map(|grids| {
                    grids
                        .iter()
                        .map(|(ny, grid)| {
                            let ext = 32 * stride as i32;
                            (ny * ext, (ny + 1) * ext, grid[lz * 32 + lx].clone())
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let (stacks_b, culled_b) = tile_column_stacks(
            base,
            level,
            tx,
            tz,
            viewer,
            &hz,
            &sample_task,
            &mut known_snap,
        );

        assert_eq!(culled_a, culled_b, "cull mask differs off-thread");
        assert_eq!(stacks_a, stacks_b, "column stacks differ off-thread");
        assert!(
            stacks_a.iter().any(|s| !s.is_empty()),
            "no stacks derived (vacuous)"
        );
    }

    // `quantize_top`'s floor proof moved to dc-core with the function
    // (`dc_core::farfield::tests::quantize_top_floors_to_the_coarse_lattice`).

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

    /// FF2b: a reduced node's overhang stack meshes volumetrically — the
    /// floating slab gets its own top face, an exposed bottom face (−Y, which
    /// the FF2a top sheet could never emit), and interval side walls — while
    /// every synthesized single-span column stays FF2a-exact around it, and
    /// the reduced ground span fuses with the synthesized ground below the
    /// node (ungenerated ≠ empty).
    #[test]
    fn stacked_columns_mesh_overhangs_with_bottom_faces() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let hz = HorizonConfig::default();
        let flat = |_wx: i64, _wz: i64| (20i32, Block::Stone);
        let stride = level_stride(1);
        let (target_wx, target_wz) = (5 * stride, 5 * stride); // cell (5,5)
        let mut known = |wx: i64, wz: i64| -> Vec<(i32, i32, Vec<ColumnSpan>)> {
            if wx == target_wx && wz == target_wz {
                // Node extent [0, 64): the ground as reduced, plus a floating
                // slab 40..44 (an overhang the authority would reduce to).
                vec![(
                    0,
                    64,
                    vec![
                        ColumnSpan {
                            top: 44,
                            bottom: 40,
                            block: Block::Stone,
                        },
                        ColumnSpan {
                            top: 20,
                            bottom: 0,
                            block: Block::Stone,
                        },
                    ],
                )]
            } else {
                Vec::new()
            }
        };
        let (stacks, culled) =
            tile_column_stacks(base, 1, 0, 0, FAR_VIEWER, &hz, &flat, &mut known);
        let m = TILE_CELLS + 2;
        let k = (5 + 1) * m + (5 + 1);
        assert_eq!(stacks[k].len(), 2, "composed overhang stack");
        assert_eq!(
            stacks[k][1].bottom, FAR_BOTTOM_UNBOUNDED,
            "reduced ground fuses with the synthesized ground below the node"
        );
        let (mesh, y_ref) = build_far_tile_mesh(base, 1, 0, 0, &stacks, &culled, [false; 4]);
        assert!(
            mesh.normals.iter().any(|n| n[1] == -1.0),
            "the overhang's underside must emit a bottom face"
        );
        let base_vs = base.voxel_size_m() as f32;
        let has_top_44 =
            mesh.positions.iter().zip(&mesh.normals).any(|(p, nrm)| {
                nrm[1] > 0.0 && (p[1] + y_ref as f32 - 44.0 * base_vs).abs() < 1e-3
            });
        assert!(has_top_44, "the slab's own top face at base voxel 44");
        // The slab hangs over cell (5,5) only: its four side walls span the
        // interval 40..44 against single-span neighbours.
        let wall_at_slab_height = mesh
            .positions
            .iter()
            .zip(&mesh.normals)
            .filter(|(p, nrm)| nrm[1] == 0.0 && p[1] + y_ref as f32 > 39.0 * base_vs)
            .count();
        assert_eq!(
            wall_at_slab_height, 16,
            "four interval walls, four verts each"
        );
    }

    /// The reduction standoff is **retired** (journal/0091). Reduced data is
    /// used wherever the node subtree is resident, right up to the near-cover
    /// edge — no per-column distance gate forces cold synthesis over warm data
    /// (that gate, `REDUCTION_STANDOFF_M = 176 m`, landed inside the L1 ring and
    /// WAS the band inversion, audit § 5). What keeps warm safe near the player
    /// is no longer a standoff but the geometry-safe floor
    /// (`floor_known_surface`): a reduced surface that `MajorityNonAir` rounded
    /// one coarse voxel ABOVE the synthesized floor is clamped back to it, so
    /// warm and cold floor to the identical surface height and warm cannot poke
    /// through the near ground. Cold synthesis is used ONLY where the reduction
    /// is absent (never-visited / evicted). This replaces the retired
    /// `reduction_standoff_keeps_near_columns_synthesized`.
    #[test]
    fn reduction_used_where_resident_and_floored_under_the_near_surface() {
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let hz = HorizonConfig::default();
        let surf = 20i32; // synth floors to 20
        let flat = move |_wx: i64, _wz: i64| (surf, Block::Grass);
        let tile_m = far_tile_m(base, 1);
        // Viewer standing on the surface at the tile centre — the whole tile is
        // inside the old 176 m standoff, where reduced data used to be refused.
        let centre = DVec3::new(
            tile_m * 0.5,
            f64::from(surf) * base.voxel_size_m(),
            tile_m * 0.5,
        );

        // A resident node whose reduced surface rounded ONE coarse voxel high
        // (top 22 vs the synth floor 20) — the legitimate majority-vs-floor gap.
        let mut resident = |_wx: i64, _wz: i64| {
            vec![(
                0,
                64,
                vec![ColumnSpan {
                    top: 22,
                    bottom: 0,
                    block: Block::Stone,
                }],
            )]
        };
        let (stacks, _c) = tile_column_stacks(base, 1, 0, 0, centre, &hz, &flat, &mut resident);
        // Warm-where-resident: the reduction IS used here now, but floored — the
        // surface top is 20 (the synth floor), NOT 22, so warm cannot rise above
        // the near ground, while the reduced material (Stone) reaches the eye.
        for s in &stacks {
            assert_eq!(s.len(), 1, "reduced surface fuses with the synth ground");
            assert_eq!(
                s[0].top, 20,
                "warm top floored to the synth surface, not 22"
            );
            assert_eq!(s[0].block, Block::Stone, "reduced material reaches the eye");
        }

        // Cold-only-where-not-resident: an empty `known` (never-visited/evicted)
        // is pure synthesis — the sole remaining use of cold.
        let mut absent = |_wx: i64, _wz: i64| Vec::new();
        let (stacks, _c) = tile_column_stacks(base, 1, 0, 0, centre, &hz, &flat, &mut absent);
        for s in &stacks {
            assert_eq!(s.len(), 1);
            assert_eq!(s[0].top, 20);
            assert_eq!(s[0].block, Block::Grass, "cold synth where not resident");
        }
    }

    /// Warm geometry-safe (journal/0091), unit-level: `floor_known_surface`
    /// clamps a reduced surface run that rounded above the synth floor, so a
    /// composed warm column's surface can NEVER exceed the floored synthesized
    /// top — the property that lets warm run right up to the near field without
    /// poking through. A real overhang (an air gap beneath it) is left untouched.
    #[test]
    fn warm_reduce_is_floored_geometry_safe() {
        let synth_top = 20i32;
        // A grounded reduced run rounded two coarse voxels high.
        let rounded = vec![(
            0,
            64,
            vec![ColumnSpan {
                top: 24,
                bottom: 0,
                block: Block::Stone,
            }],
        )];
        let floored = floor_known_surface(synth_top, &rounded);
        assert_eq!(
            floored[0].2[0].top, synth_top,
            "rounded surface run clamped down to the floor"
        );
        let synth = ColumnSpan {
            top: synth_top,
            bottom: FAR_BOTTOM_UNBOUNDED,
            block: Block::Grass,
        };
        let col = compose_column(synth, &floored);
        assert!(
            col.iter().all(|s| s.top <= synth_top),
            "no warm span rises above the floored surface"
        );

        // An overhang wholly above the floor (air gap beneath) is REAL geometry
        // the near field also renders — kept, not clamped. Its grounded run below
        // the floor is subsurface and also untouched.
        let overhang = vec![(
            0,
            64,
            vec![
                ColumnSpan {
                    top: 40,
                    bottom: 32,
                    block: Block::Stone,
                },
                ColumnSpan {
                    top: 18,
                    bottom: 0,
                    block: Block::Stone,
                },
            ],
        )];
        let floored = floor_known_surface(synth_top, &overhang);
        assert_eq!(
            floored[0].2[0].top, 40,
            "overhang above the floor is preserved"
        );
        assert_eq!(floored[0].2[0].bottom, 32);
        assert_eq!(floored[0].2[1].top, 18, "subsurface run untouched");
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
        let hz = &HorizonConfig::default();

        // A distant viewer culls nothing — the full horizon renders.
        let (_s, culled_far) =
            tile_column_stacks(base, 1, 0, 0, FAR_VIEWER, hz, &sample, &mut no_known);
        assert!(
            culled_far.iter().all(|c| !c),
            "distant viewer culls no columns"
        );

        // A viewer standing on the surface at the L1 tile's centre puts every
        // column inside NEAR_COVER_R (an L1 tile is ~57 m < 112 m), so the tile is
        // fully covered and meshes to NOTHING — no buried sheet to dig into.
        let tile_m = far_tile_m(base, 1);
        let centre = DVec3::new(tile_m * 0.5, surf_m, tile_m * 0.5);
        let (stacks, culled) =
            tile_column_stacks(base, 1, 0, 0, centre, hz, &sample, &mut no_known);
        assert!(
            culled.iter().all(|&c| c),
            "a tile under the near field must have all columns culled"
        );
        let (mesh, _y) = build_far_tile_mesh(base, 1, 0, 0, &stacks, &culled, [false; 4]);
        assert!(
            mesh.is_empty(),
            "a fully-covered tile draws nothing (coverage, not buried geometry)"
        );
    }

    /// FF2a scale-headroom checkpoint, re-cut as the `--horizon` sweep
    /// (journal/0042). journal/0023 could only *project* the 5–10 km field
    /// (by assuming extra LOD rings); with the horizon a runtime knob the whole
    /// field can be built for real at each setting and MEASURED — tile count,
    /// mesh memory, derive+mesh ms/tile, and the coarsest ring's voxel step.
    /// Run with `--nocapture` to read the table; as a test it asserts the field
    /// is non-empty, per-tile cost is bounded, and — the load-bearing one — that
    /// the **per-frame** meshing budget does not grow with the horizon (a wider
    /// horizon buys more frames of streaming, never a hitch).
    #[test]
    fn horizon_sweep_measures_the_far_field_cost() {
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
        println!("\n=== --horizon sweep, MEASURED (worst case: no coverage cull) ===");
        println!("bytes/vertex = {BYTES_PER_VERT} (pos+nrm+col+uv+layers+weights)");
        println!(
            "| horizon | tiles (L1..L4) | tris | mesh MiB | ms/tile | frame ms (2 tiles) | \
             fill s @60fps | coarsest step |"
        );
        let mut baseline_ms_per_tile = 0.0f64;
        for km in [1.2f64, 3.0, 5.0, 10.0] {
            let hz = HorizonConfig::with_far_max(km * 1000.0);
            let mut tiles = 0usize;
            let mut tris = 0usize;
            let mut verts = 0usize;
            let mut per_ring = [0usize; 5];
            let t0 = Instant::now();
            for level in 1..=4u8 {
                for (tx, tz) in wanted_far_tiles(base, viewer, level, &hz) {
                    let (stacks, culled) = tile_column_stacks(
                        base,
                        level,
                        tx,
                        tz,
                        viewer,
                        &hz,
                        &sample,
                        &mut no_known,
                    );
                    let (mesh, _y) =
                        build_far_tile_mesh(base, level, tx, tz, &stacks, &culled, [true; 4]);
                    tiles += 1;
                    per_ring[level as usize] += 1;
                    tris += mesh.triangle_count();
                    verts += mesh.positions.len();
                }
            }
            let elapsed = t0.elapsed();
            let mesh_bytes = verts * BYTES_PER_VERT + tris * 3 * 4;
            let ms_per_tile = elapsed.as_secs_f64() * 1e3 / tiles.max(1) as f64;
            // The coarsest ring's effective voxel step — the legibility cost.
            let step = coarse_scale(base, 4).voxel_size_m();
            println!(
                "| {km} km | {tiles} {:?} | {tris} | {:.1} | {ms_per_tile:.3} | {:.3} | {:.1} | \
                 {step:.1} m |",
                &per_ring[1..],
                mesh_bytes as f64 / 1_048_576.0,
                FAR_SURFACE_BUDGET_PER_FRAME as f64 * ms_per_tile,
                tiles as f64 / FAR_SURFACE_BUDGET_PER_FRAME as f64 / 60.0,
            );
            assert!(tiles > 0, "the far field must produce tiles at {km} km");
            assert!(
                mesh_bytes / tiles < 200_000,
                "per-tile mesh unexpectedly large at {km} km"
            );
            // The budget is per FRAME, not per field: widening the horizon must
            // not raise the work done in any one frame. Per-tile cost is the
            // frame cost (× the fixed budget), so it must stay flat within a
            // generous timing-noise factor.
            if km == 1.2 {
                baseline_ms_per_tile = ms_per_tile;
            } else {
                assert!(
                    ms_per_tile < baseline_ms_per_tile * 4.0 + 1.0,
                    "per-frame meshing cost grew with the horizon \
                     ({ms_per_tile:.3} ms/tile vs {baseline_ms_per_tile:.3} at 1.2 km)"
                );
            }
        }
        println!();
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
            .map(|l| {
                wanted_far_tiles(base, viewer, l, &HorizonConfig::default())
                    .into_iter()
                    .collect()
            })
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
        let mut r = HorizonConfig::default().ring_edges[0]; // 112 m: LOD-1 inner edge
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

    /// The same coverage theorem at a STRETCHED horizon (journal/0042): widening
    /// the outermost ring must not open a new class of sky hole. Levels 1–3 are
    /// untouched by the knob, so the interesting seam is L3/L4 at 1024 m and the
    /// whole 1024 m → 8 km body of the stretched L4 ring. As above, the world's
    /// outer rim (near the 10 km edge) is legitimately ragged and excluded.
    #[test]
    fn a_stretched_horizon_opens_no_new_sky_holes() {
        use std::collections::HashSet;
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let hz = HorizonConfig::with_far_max(10_000.0);
        let viewer = DVec3::new(123.4, 1000.0, -77.6);
        let sets: Vec<HashSet<(i32, i32)>> = (1..=4u8)
            .map(|l| wanted_far_tiles(base, viewer, l, &hz).into_iter().collect())
            .collect();
        let covered = |px: f64, pz: f64| -> bool {
            (1..=4u8).any(|l| {
                let tile_m = far_tile_m(base, l);
                let tx = (px / tile_m).floor() as i32;
                let tz = (pz / tile_m).floor() as i32;
                sets[usize::from(l) - 1].contains(&(tx, tz))
            })
        };
        let mut holes = 0usize;
        let mut first_hole = None;
        let mut r = hz.ring_edges[0];
        while r < 8000.0 {
            for k in 0..720 {
                let a = std::f64::consts::TAU * f64::from(k) / 720.0;
                let (px, pz) = (viewer.x + r * a.cos(), viewer.z + r * a.sin());
                if !covered(px, pz) {
                    holes += 1;
                    if first_hole.is_none() {
                        first_hole = Some((r, a));
                    }
                }
            }
            r += 2.0;
        }
        assert_eq!(
            holes, 0,
            "stretched horizon has {holes} uncovered ground points; \
             first at radius/angle {first_hole:?}"
        );
    }

    /// Corrections #11, the fix pinned in world space. Two adjacent same-level
    /// tiles share the plane `x = (tx+1)·tile_m`. Each tile places a shared-edge
    /// physical point P at `P + push(level)`, where `push` is the anti-z-fight
    /// depth push. With the UNIFORM per-level push (`level_depth_push`, which takes
    /// no tile coordinate) both tiles use the byte-identical vector, so the two
    /// placements coincide exactly — the world-space seam gap is zero for an
    /// arbitrary, nasty viewer pose (high altitude, off-grid yaw, ~-45° pitch).
    ///
    /// The RETIRED per-tile radial scheme fails this: it pushed each tile along
    /// its OWN center→viewer direction, and the two directions differ by the
    /// tiles' angular separation, opening a real sub-pixel-to-pixel slot of
    /// background light (the thin bright seams the user reported). That failing
    /// differential is recomputed here on the retired formula purely to pin the
    /// mechanism — it is a rationale assertion, not kept-failing production code.
    /// The f32 render cast (`to_render`) is a separate, shared sub-millimetre
    /// effect (≈0.1 mm at 1.2 km, corrections #11) and is deliberately out of this
    /// world-space (f64 meters) proof.
    #[test]
    fn uniform_push_keeps_same_level_seams_watertight_in_world_space() {
        use crate::player::Player;
        let base = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        // Level 4: the largest coarse voxel → the largest radial differential the
        // old scheme would have opened (worst case, ≈0.9 m per corrections #11).
        let level = 4u8;
        let tile_m = far_tile_m(base, level);

        // A nasty, off-axis, high-altitude viewer looking down ~-45° — exactly the
        // vantage the field report used.
        let mut player = Player::new(DVec3::new(37.0, 900.0, -19.0));
        player.yaw = 0.7; // off the tile grid, so no symmetry hides a seam
        player.pitch = -0.78; // ~-45° down
        let forward = player.view_dir();
        let viewer = player.pos_m;

        // World position of a physical point P as placed by a given tile: its
        // transform origin (`min + push`) plus P's tile-local mesh coordinate
        // (`P - min`). The tile origin `min` really enters and cancels in f64 —
        // this mirrors `far_tile_translation` minus the shared floating-origin /
        // f32 render cast. Tiles A=(tx,tz) and B=(tx+1,tz) share the plane x=(tx+1)·tile_m.
        let (tx, tz) = (3i32, -2i32);
        let via = |tx: i32, tz: i32, p: DVec3| -> DVec3 {
            let min = DVec3::new(f64::from(tx) * tile_m, 0.0, f64::from(tz) * tile_m);
            (min + level_depth_push(base, level, forward)) + (p - min)
        };
        let shared_x = f64::from(tx + 1) * tile_m;

        for k in 0..=8 {
            let pz = (f64::from(tz) + f64::from(k) / 8.0) * tile_m;
            let p = DVec3::new(shared_x, 40.0, pz);
            let from_a = via(tx, tz, p);
            let from_b = via(tx + 1, tz, p);
            assert_eq!(
                from_a, from_b,
                "uniform push split a shared-edge point between adjacent tiles"
            );
        }
        // The push is tile-independent by construction (no tx/tz in its signature).
        assert_eq!(
            level_depth_push(base, level, forward),
            level_depth_push(base, level, forward)
        );

        // Rationale: the retired radial scheme opens a real world-space slot here.
        let cvs = coarse_scale(base, level).voxel_size_m();
        let old_away = |tx: i32, tz: i32| -> DVec3 {
            let min = DVec3::new(f64::from(tx) * tile_m, 0.0, f64::from(tz) * tile_m);
            let center = DVec3::new(min.x + tile_m * 0.5, 0.0, min.z + tile_m * 0.5);
            (center - viewer).normalize_or_zero() * (DEPTH_PUSH_FRAC * cvs)
        };
        let old_gap = (old_away(tx, tz) - old_away(tx + 1, tz)).length();
        assert!(
            old_gap > 0.05,
            "retired radial scheme should open a visible slot here (got {old_gap} m)"
        );
    }
}
