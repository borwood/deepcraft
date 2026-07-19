//! Spike-local terrain generation for S1. The real hierarchical worldgen is
//! dc-worldgen's job (spike S7); this module exists only so the walking
//! skeleton has rolling hills, a deep chasm, and caves to walk through.
//!
//! Scale independence: all noise is sampled in **meter** space, so the same
//! seed produces the same landscape at every voxel scale — only the sampling
//! resolution changes. That is exactly the comparison S1 wants to make.

use dc_core::{
    Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos, VoxelQuery, VoxelScale, column_top_solid_y,
};
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};

/// Base terrain altitude in meters.
const BASE_HEIGHT_M: f64 = 8.0;
/// Rolling-hill amplitude in meters.
const HILL_AMP_M: f64 = 14.0;
/// How deep the chasm floor cuts below the surface, in meters.
const CHASM_DEPTH_M: f64 = 90.0;
/// Half-width of the chasm in noise-mask units (bigger = wider ravines).
const CHASM_HALF_WIDTH: f64 = 0.10;
/// 3D cave noise threshold; higher = rarer caves.
const CAVE_THRESHOLD: f32 = 0.58;
/// Dirt layer thickness in meters (below the grass voxel).
const DIRT_DEPTH_M: f64 = 1.0;

pub struct TerrainGen {
    hills: FastNoiseLite,
    chasm: FastNoiseLite,
    caves: FastNoiseLite,
}

impl TerrainGen {
    pub fn new(seed: i32) -> Self {
        let mut hills = FastNoiseLite::with_seed(seed);
        hills.set_noise_type(Some(NoiseType::OpenSimplex2));
        hills.set_fractal_type(Some(FractalType::FBm));
        hills.set_fractal_octaves(Some(4));
        hills.set_frequency(Some(0.008)); // ~125 m base wavelength

        let mut chasm = FastNoiseLite::with_seed(seed.wrapping_add(1));
        chasm.set_noise_type(Some(NoiseType::OpenSimplex2));
        chasm.set_frequency(Some(0.003)); // ~330 m wavelength ravine paths

        let mut caves = FastNoiseLite::with_seed(seed.wrapping_add(2));
        caves.set_noise_type(Some(NoiseType::OpenSimplex2));
        caves.set_frequency(Some(0.045)); // ~20 m cave features

        Self {
            hills,
            chasm,
            caves,
        }
    }

    /// Terrain surface height in meters at a horizontal position in meters.
    /// Rolling hills, with deep winding ravines carved where the low-frequency
    /// chasm mask crosses zero.
    pub fn surface_height_m(&self, xm: f64, zm: f64) -> f64 {
        let h = f64::from(self.hills.get_noise_2d(xm as f32, zm as f32));
        let base = BASE_HEIGHT_M + h * HILL_AMP_M;

        let c = f64::from(self.chasm.get_noise_2d(xm as f32, zm as f32));
        let t = (1.0 - c.abs() / CHASM_HALF_WIDTH).clamp(0.0, 1.0);
        let carve = t * t * (3.0 - 2.0 * t) * CHASM_DEPTH_M; // smoothstep
        base - carve
    }

    /// 3D cave carve at a position in meters.
    fn is_cave(&self, xm: f64, ym: f64, zm: f64) -> bool {
        self.caves.get_noise_3d(xm as f32, ym as f32, zm as f32) > CAVE_THRESHOLD
    }

    /// Block for a voxel given the surface height of its column. `ym` is the
    /// voxel-center altitude in meters.
    fn block_in_column(&self, xm: f64, ym: f64, zm: f64, surface_m: f64, voxel_m: f64) -> Block {
        if ym >= surface_m {
            return Block::Air;
        }
        if self.is_cave(xm, ym, zm) {
            return Block::Air;
        }
        let depth = surface_m - ym;
        if depth < voxel_m {
            Block::Grass
        } else if depth < voxel_m + DIRT_DEPTH_M {
            Block::Dirt
        } else {
            Block::Stone
        }
    }

    /// Block at a single world-space voxel coordinate. Deterministic and
    /// identical to what [`Self::generate_chunk`] would put there — used for
    /// cross-chunk neighbor queries without materializing the neighbor.
    pub fn block_at(&self, scale: VoxelScale, vx: i64, vy: i64, vz: i64) -> Block {
        let vs = scale.voxel_size_m();
        let xm = (vx as f64 + 0.5) * vs;
        let ym = (vy as f64 + 0.5) * vs;
        let zm = (vz as f64 + 0.5) * vs;
        self.block_in_column(xm, ym, zm, self.surface_height_m(xm, zm), vs)
    }

    /// Generate a full chunk at the given scale.
    pub fn generate_chunk(&self, scale: VoxelScale, pos: ChunkPos) -> Chunk {
        let vs = scale.voxel_size_m();
        let (mx, my, mz) = pos.min_voxel();

        // Surface height is 2D; cache it per column.
        let mut heights = [[0.0f64; CHUNK_SIZE_USIZE]; CHUNK_SIZE_USIZE];
        let mut col_meters = [[(0.0f64, 0.0f64); CHUNK_SIZE_USIZE]; CHUNK_SIZE_USIZE];
        for (lz, row) in heights.iter_mut().enumerate() {
            for (lx, h) in row.iter_mut().enumerate() {
                let xm = (mx + lx as i64) as f64 * vs + vs * 0.5;
                let zm = (mz + lz as i64) as f64 * vs + vs * 0.5;
                *h = self.surface_height_m(xm, zm);
                col_meters[lz][lx] = (xm, zm);
            }
        }

        let mut chunk = Chunk::new();
        for ly in 0..CHUNK_SIZE_USIZE {
            let ym = (my + ly as i64) as f64 * vs + vs * 0.5;
            for lz in 0..CHUNK_SIZE_USIZE {
                for lx in 0..CHUNK_SIZE_USIZE {
                    let (xm, zm) = col_meters[lz][lx];
                    let block = self.block_in_column(xm, ym, zm, heights[lz][lx], vs);
                    if block != Block::Air {
                        chunk.set(lx, ly, lz, block);
                    }
                }
            }
        }
        chunk
    }
}

/// Edit allowance above the **authority's own** per-column surface where the
/// downward scan begins, in meters.
///
/// This is *not* slop against an estimate of the terrain — journal/0016. The
/// `analytic` closure the scan is seeded with is now the authority's exact
/// column-surface height (the collapse `ColumnRec` for worldgen; `block_in_column`'s
/// own half-voxel-exact bound for the legacy S1 terrain), so the ceiling always
/// sits at or above the true *generated* top solid voxel. The only thing that
/// can rise above it is a stack of player **edits** placed on the ground, and
/// this is the headroom for that. It is deliberately small: a ceiling derived
/// from the terrain's provenance can never drift from the terrain again (the
/// bug journal/0015 diagnosed — an 8 m S1-sized headroom over a *pre-deep-time*
/// estimate started the window ~100 m below the moved ground).
pub const SURFACE_SCAN_EDIT_HEADROOM_M: f64 = 4.0;

/// How far below each column's own ceiling [`true_surface_m`] probes before
/// giving up on that column. The ceiling now tracks the true surface per
/// column, so this only has to span the deepest *edit* dug down from the
/// surface plus generous slack; a genuine miss returns `None` (loud), never a
/// buried point.
pub const SURFACE_SCAN_DEPTH_M: f64 = 96.0;

/// The **true** walking surface in meters for a body of footprint half-width
/// `footprint_half_m` centered at `(xm, zm)`: the highest solid voxel's top
/// face across every column the footprint covers, read from the live solidity
/// query `solid` (terrain AND edits), at the active `scale`. `None` when *no*
/// solid voxel is found under the whole footprint within the scan window — an
/// honest "there is nothing to stand on here" the caller must reject rather
/// than seat a body on a plausible-looking buried y (journal/0015).
///
/// This is the fix for the `surface_height_m` under-report (journal/0006) and,
/// with the honest ceiling below, for the deep-time ceiling regression
/// (journal/0015–0016). A body's footprint spans neighbouring columns, and on a
/// slope the highest of those can stand meters above the value at the center —
/// so the resting surface is the *max* of the per-column top faces.
///
/// `analytic` is the authority's **exact** per-column surface height — the
/// collapse `ColumnRec` height for the worldgen authority (deep-time-aware, so
/// the window tracks the real ground) or the S1 `surface_height_m` (a
/// half-voxel-exact upper bound by construction). It seeds each column's
/// downward scan a small [`SURFACE_SCAN_EDIT_HEADROOM_M`] above its own surface,
/// so the window is derived from the terrain's own provenance, never an
/// independent estimate that could drift from it. It is sampled *per footprint
/// column*, not once — a shared ceiling from the center column would start below
/// a steeper edge column's real surface and miss its top voxels.
pub fn true_surface_m(
    solid: &impl VoxelQuery,
    analytic: impl Fn(f64, f64) -> f64,
    scale: VoxelScale,
    xm: f64,
    zm: f64,
    footprint_half_m: f64,
) -> Option<f64> {
    let vs = scale.voxel_size_m();
    let x0 = scale.voxel_at(xm - footprint_half_m);
    let x1 = scale.voxel_at(xm + footprint_half_m);
    let z0 = scale.voxel_at(zm - footprint_half_m);
    let z1 = scale.voxel_at(zm + footprint_half_m);
    let mut top: Option<i64> = None;
    for x in x0..=x1 {
        for z in z0..=z1 {
            // Each column scans down from a small edit-headroom above its OWN
            // authority surface — an honest ceiling, not an estimate.
            let (cx, cz) = ((x as f64 + 0.5) * vs, (z as f64 + 0.5) * vs);
            let h = analytic(cx, cz);
            let ceil_y = scale.voxel_at(h + SURFACE_SCAN_EDIT_HEADROOM_M);
            let floor_y = scale.voxel_at(h - SURFACE_SCAN_DEPTH_M);
            if let Some(y) = column_top_solid_y(solid, x, z, ceil_y, floor_y) {
                // The ceiling must sit strictly ABOVE the true top solid voxel:
                // if the topmost voxel in the window is itself solid, the window
                // started at or below the ground and may be hiding solid above
                // it — exactly the deep-time-outran-the-ceiling failure
                // (journal/0015). In generated terrain (no tall edits) this can
                // never happen; the assertion catches a ceiling that drifts
                // below the terrain again.
                debug_assert!(
                    y < ceil_y,
                    "surface scan ceiling below solid ground at column ({x}, {z}): \
                     top solid voxel {y} reached the ceiling {ceil_y} (analytic h = {h:.2} m); \
                     the scan window is starting inside the terrain"
                );
                top = Some(top.map_or(y, |t| t.max(y)));
            }
        }
    }
    // Rest on the solid voxel's top face, or report the honest miss.
    top.map(|y| (y + 1) as f64 * vs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::{Aabb, aabb_overlaps_solid, local_voxel};

    #[test]
    fn block_at_matches_generate_chunk() {
        let generator = TerrainGen::new(42);
        let scale = VoxelScale::from_player_height(1.8, 3);
        // A chunk straddling the surface, and one deep down.
        for pos in [ChunkPos::new(0, 0, 0), ChunkPos::new(-1, -2, 1)] {
            let chunk = generator.generate_chunk(scale, pos);
            let (mx, my, mz) = pos.min_voxel();
            for (lx, ly, lz) in [(0, 0, 0), (31, 31, 31), (5, 17, 23), (16, 8, 4)] {
                let (wx, wy, wz) = (mx + lx as i64, my + ly as i64, mz + lz as i64);
                let (llx, lly, llz) = local_voxel(wx, wy, wz);
                assert_eq!(
                    generator.block_at(scale, wx, wy, wz),
                    chunk.get(llx, lly, llz),
                    "mismatch at world voxel ({wx}, {wy}, {wz})"
                );
            }
        }
    }

    #[test]
    fn same_seed_same_terrain() {
        let a = TerrainGen::new(7);
        let b = TerrainGen::new(7);
        for (x, z) in [(0.5, 0.5), (-200.0, 340.0), (1e4, -1e4)] {
            assert_eq!(
                a.surface_height_m(x, z).to_bits(),
                b.surface_height_m(x, z).to_bits()
            );
        }
    }

    #[test]
    fn chasm_region_exists_and_is_deep() {
        // Scan a broad area for the chasm; the mask crosses zero somewhere in
        // any few-hundred-meter window at frequency 0.003.
        let generator = TerrainGen::new(1337);
        let mut deepest = f64::INFINITY;
        for zi in -60..60 {
            for xi in -60..60 {
                let (xm, zm) = (f64::from(xi) * 8.0, f64::from(zi) * 8.0);
                deepest = deepest.min(generator.surface_height_m(xm, zm));
            }
        }
        assert!(
            deepest < -40.0,
            "expected a deep chasm below -40 m in the scanned region, deepest = {deepest}"
        );
    }

    /// The under-report, quantified — and its fix.
    ///
    /// `surface_height_m` is analytic and per-column; a body's footprint spans
    /// neighbouring columns, and on a slope the highest of those stands above
    /// the analytic value at the center. Clamping feet to `analytic + eps` then
    /// embeds the body in that higher ground (the buried spawns/teleports of
    /// journal/0004–0005). This test measures the gap over a broad grid, proves
    /// it can bury a body, and proves [`true_surface_m`] never does.
    #[test]
    fn surface_height_m_underreports_true_voxel_surface() {
        let generator = TerrainGen::new(1337);
        let scale = VoxelScale::from_player_height(1.8, 3);
        let vs = scale.voxel_size_m();
        let half = 0.3; // player footprint half-width (0.6 m wide)
        let solid = |x: i64, y: i64, z: i64| generator.block_at(scale, x, y, z) != Block::Air;

        let mut worst_gap = 0.0f64;
        let mut worst_at = (0.0, 0.0);
        let mut embedded_by_analytic = 0usize;
        let mut samples = 0usize;

        // Scan a broad grid straddling the chasm walls (steep gradients live
        // there), at non-grid offsets so we sample mid-column too.
        let mut zi = -400;
        while zi <= 400 {
            let mut xi = -400;
            while xi <= 400 {
                let (xm, zm) = (f64::from(xi) + 0.37, f64::from(zi) + 0.61);
                let analytic = generator.surface_height_m(xm, zm);
                // Skip the open chasm interior (nothing to stand on near here).
                if analytic > -20.0 {
                    samples += 1;
                    let truth = true_surface_m(
                        &solid,
                        |x, z| generator.surface_height_m(x, z),
                        scale,
                        xm,
                        zm,
                        half,
                    )
                    .expect("a land column above -20 m always has a surface");
                    let gap = truth - analytic;
                    if gap > worst_gap {
                        worst_gap = gap;
                        worst_at = (xm, zm);
                    }
                    // Would feet clamped just above the analytic height be
                    // embedded? (Mirrors the old spawn/teleport: analytic + a
                    // small margin.)
                    let feet_analytic = analytic + 0.05;
                    let aabb = Aabb::from_bottom_center(
                        glam::DVec3::new(xm, feet_analytic, zm) / vs,
                        half / vs,
                        1.8 / vs,
                    );
                    if aabb_overlaps_solid(&solid, aabb) {
                        embedded_by_analytic += 1;
                    }

                    // The fix: feet on the true surface are never embedded.
                    let feet_true = truth + 0.05;
                    let aabb_true = Aabb::from_bottom_center(
                        glam::DVec3::new(xm, feet_true, zm) / vs,
                        half / vs,
                        1.8 / vs,
                    );
                    assert!(
                        !aabb_overlaps_solid(&solid, aabb_true),
                        "true_surface_m left a body embedded at ({xm:.2}, {zm:.2})"
                    );
                }
                xi += 7;
            }
            zi += 7;
        }

        // Print the measured numbers (see with `--nocapture`; captured in
        // journal/0006).
        println!(
            "surface under-report: worst gap = {worst_gap:.2} m at ({:.1}, {:.1}); \
             analytic-clamped feet embedded at {embedded_by_analytic}/{samples} sites",
            worst_at.0, worst_at.1
        );

        // The bug is real: the analytic height under-reports by more than a
        // voxel somewhere, and that buries bodies.
        assert!(
            worst_gap > vs,
            "expected an under-report exceeding one voxel ({vs:.2} m); got {worst_gap:.2} m"
        );
        assert!(
            embedded_by_analytic > 0,
            "expected analytic-clamped placement to embed a body somewhere"
        );
    }
}
