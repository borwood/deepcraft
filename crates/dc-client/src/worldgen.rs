//! Spike-local terrain generation for S1. The real hierarchical worldgen is
//! dc-worldgen's job (spike S7); this module exists only so the walking
//! skeleton has rolling hills, a deep chasm, and caves to walk through.
//!
//! Scale independence: all noise is sampled in **meter** space, so the same
//! seed produces the same landscape at every voxel scale — only the sampling
//! resolution changes. That is exactly the comparison S1 wants to make.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos, VoxelScale};
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

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::local_voxel;

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
}
