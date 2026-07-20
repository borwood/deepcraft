//! A dense solid/air voxel volume for the S11 water spike.
//!
//! Deliberately minimal and **standalone**: the spike must not thread water
//! through the production collapse path (scope discipline), so it owns its own
//! toy volume. One bit per voxel; edits are box dig/fill, which is what the
//! adversarial digging scenarios need.

/// Solid/air occupancy over a fixed box, one bit per voxel.
#[derive(Clone)]
pub struct VoxWorld {
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
    bits: Vec<u64>,
}

impl VoxWorld {
    /// A fully solid volume.
    pub fn solid(dx: usize, dy: usize, dz: usize) -> Self {
        let n = dx * dy * dz;
        Self {
            dx,
            dy,
            dz,
            bits: vec![u64::MAX; n.div_ceil(64)],
        }
    }

    /// A fully open volume.
    pub fn air(dx: usize, dy: usize, dz: usize) -> Self {
        let n = dx * dy * dz;
        Self {
            dx,
            dy,
            dz,
            bits: vec![0; n.div_ceil(64)],
        }
    }

    #[inline]
    pub fn cells(&self) -> usize {
        self.dx * self.dy * self.dz
    }

    #[inline]
    pub fn in_bounds(&self, x: i64, y: i64, z: i64) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && (x as usize) < self.dx
            && (y as usize) < self.dy
            && (z as usize) < self.dz
    }

    /// Row-major index: `(y * dz + z) * dx + x`.
    #[inline]
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (y * self.dz + z) * self.dx + x
    }

    #[inline]
    pub fn is_solid(&self, x: usize, y: usize, z: usize) -> bool {
        let i = self.idx(x, y, z);
        self.bits[i >> 6] & (1u64 << (i & 63)) != 0
    }

    /// Out-of-bounds reads as solid — the volume is a sealed box.
    #[inline]
    pub fn is_solid_i(&self, x: i64, y: i64, z: i64) -> bool {
        if !self.in_bounds(x, y, z) {
            return true;
        }
        self.is_solid(x as usize, y as usize, z as usize)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, solid: bool) {
        let i = self.idx(x, y, z);
        let (w, b) = (i >> 6, 1u64 << (i & 63));
        if solid {
            self.bits[w] |= b;
        } else {
            self.bits[w] &= !b;
        }
    }

    /// Set an inclusive box, clipped to bounds. Returns the number of voxels
    /// whose state actually changed (the honest "edit size").
    pub fn set_box(&mut self, lo: [i64; 3], hi: [i64; 3], solid: bool) -> usize {
        let mut changed = 0;
        let x0 = lo[0].max(0) as usize;
        let y0 = lo[1].max(0) as usize;
        let z0 = lo[2].max(0) as usize;
        let x1 = hi[0].min(self.dx as i64 - 1);
        let y1 = hi[1].min(self.dy as i64 - 1);
        let z1 = hi[2].min(self.dz as i64 - 1);
        if x1 < x0 as i64 || y1 < y0 as i64 || z1 < z0 as i64 {
            return 0;
        }
        for y in y0..=(y1 as usize) {
            for z in z0..=(z1 as usize) {
                for x in x0..=(x1 as usize) {
                    if self.is_solid(x, y, z) != solid {
                        self.set(x, y, z, solid);
                        changed += 1;
                    }
                }
            }
        }
        changed
    }

    /// Dig (open) an inclusive box.
    pub fn dig(&mut self, lo: [i64; 3], hi: [i64; 3]) -> usize {
        self.set_box(lo, hi, false)
    }

    /// Fill (seal) an inclusive box.
    pub fn fill(&mut self, lo: [i64; 3], hi: [i64; 3]) -> usize {
        self.set_box(lo, hi, true)
    }

    /// Total open voxels — used by the conservation checks.
    pub fn air_count(&self) -> usize {
        let padding = self.bits.len() * 64 - self.cells();
        self.bits
            .iter()
            .map(|w| w.count_zeros() as usize)
            .sum::<usize>()
            - padding
    }
}
