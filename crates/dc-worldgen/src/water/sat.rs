//! Bound water: a saturation field over the S8 pore model, and the smallest
//! honest relaxation that moves it.
//!
//! water.md DECIDED (one quantity, two regimes): bound water occupies pores and
//! the empty eighths of loose partials. Here that is one scalar per voxel —
//! `sat` ∈ [0, 1], the filled fraction of that voxel's pore volume. Water volume
//! in a voxel is `sat * porosity`, in voxel-volume units.
//!
//! The relaxation is three sub-steps, each a **gather from a frozen snapshot**
//! (the S9b reformulation): infiltration, gravity percolation, and a
//! permeability-limited lateral redistribution driven by head. Every lateral
//! edge flux is computed once as a pure function of the unordered pair, so it is
//! exactly antisymmetric and mass is conserved to the bit regardless of the
//! iteration driver.
//!
//! The **water table** is not stored. It is read as the top of the saturated
//! zone, per water.md consequence 1.
//!
//! **E4-1b (filed 2026-08-03, heir: `dc_core::field::FieldKernel`).** The
//! lateral step below is the second S-10 instance and was NOT converted when
//! creep's was extracted (E4-1): it needs the pair-permeability coefficient
//! rule (`PairMin`) and the **upper** (pore-space) obstacle, plus f32/3D
//! adaptation — the shape-proof slice the E4 design audit sequences as E4-2b
//! (`docs/audits/2026-08-03-e4-implicit-kernel-design.md` § 3.4). This module
//! is unconsumed machinery (spines § 3), so the conversion is a shape check on
//! the API, not a build order.

/// Per-rock pore facts. In production these come from the property sheet
/// (materials.md: porosity from structure fill, permeability a granular
/// property); the spike carries a small table so it stays standalone.
#[derive(Clone, Copy, Debug)]
pub struct RockProps {
    /// Pore fraction of the voxel available to fluid.
    pub porosity: f32,
    /// Relative permeability (voxel/step per unit head).
    pub perm: f32,
}

/// Open space — the free-water regime. Bound water reaching it crosses regimes.
pub const ROCK_VOID: u8 = 0;

/// Saturation threshold at which a voxel counts as "saturated zone".
pub const SATURATED: f32 = 0.999;

/// A bound-water field over a solid volume.
pub struct SatField {
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
    /// Rock id per voxel; `ROCK_VOID` is open space.
    pub rock: Vec<u8>,
    /// Property table indexed by rock id.
    pub props: Vec<RockProps>,
    /// Saturation per voxel, [0, 1] of pore volume.
    pub sat: Vec<f32>,
    /// Seepage sinks: water arriving here leaves the bound regime.
    pub drain: Vec<bool>,
    /// Recharge per step, in voxel-volumes of water, applied to the topmost
    /// non-void voxel of each column.
    pub recharge: f32,
    /// Lateral conductance scale (the Darcy coefficient, folded with dt).
    pub lateral_c: f32,
    /// Gravity percolation scale (folded with dt).
    pub vertical_c: f32,
    /// Running total of water that crossed bound → free (drains + voids).
    pub crossed_to_free: f64,
    /// Running total of recharge admitted.
    pub admitted: f64,
    // Scratch, reused between steps so the step allocates nothing.
    frozen_w: Vec<f32>,
    frozen_space: Vec<f32>,
    vflux: Vec<f32>,
    l_out: Vec<f32>,
    l_in: Vec<f32>,
    gross_out: Vec<f32>,
    gross_in: Vec<f32>,
}

impl SatField {
    pub fn new(dx: usize, dy: usize, dz: usize, props: Vec<RockProps>) -> Self {
        let n = dx * dy * dz;
        Self {
            dx,
            dy,
            dz,
            rock: vec![1; n],
            props,
            sat: vec![0.0; n],
            drain: vec![false; n],
            recharge: 0.0,
            lateral_c: 0.25,
            vertical_c: 1.0,
            crossed_to_free: 0.0,
            admitted: 0.0,
            frozen_w: vec![0.0; n],
            frozen_space: vec![0.0; n],
            vflux: vec![0.0; n],
            l_out: vec![0.0; n],
            l_in: vec![0.0; n],
            gross_out: vec![0.0; n],
            gross_in: vec![0.0; n],
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (y * self.dz + z) * self.dx + x
    }

    #[inline]
    fn porosity(&self, i: usize) -> f32 {
        self.props[self.rock[i] as usize].porosity
    }

    #[inline]
    fn perm(&self, i: usize) -> f32 {
        self.props[self.rock[i] as usize].perm
    }

    #[inline]
    fn is_void(&self, i: usize) -> bool {
        self.rock[i] == ROCK_VOID
    }

    /// Total bound water in the field (voxel-volumes).
    pub fn total_water(&self) -> f64 {
        (0..self.sat.len())
            .map(|i| (self.sat[i] * self.porosity(i)) as f64)
            .sum()
    }

    /// The water table: for each column, the elevation of the top of the
    /// saturated zone, or `None` where nothing is saturated. Read, never stored.
    pub fn water_table(&self) -> Vec<Option<f32>> {
        let mut out = vec![None; self.dx * self.dz];
        for z in 0..self.dz {
            for x in 0..self.dx {
                let mut top = None;
                for y in (0..self.dy).rev() {
                    let i = self.idx(x, y, z);
                    if !self.is_void(i) && self.sat[i] >= SATURATED {
                        top = Some(y as f32 + 1.0);
                        break;
                    }
                }
                out[z * self.dx + x] = top;
            }
        }
        out
    }

    /// One relaxation step. Pure gather from a frozen snapshot in every phase.
    pub fn step(&mut self) {
        self.infiltrate();
        self.freeze();
        self.percolate();
        self.freeze();
        self.redistribute();
        self.seep();
    }

    fn freeze(&mut self) {
        for i in 0..self.sat.len() {
            let phi = self.porosity(i);
            let w = self.sat[i] * phi;
            self.frozen_w[i] = w;
            self.frozen_space[i] = (phi - w).max(0.0);
        }
    }

    /// Recharge into the topmost non-void voxel of every column.
    fn infiltrate(&mut self) {
        if self.recharge <= 0.0 {
            return;
        }
        for z in 0..self.dz {
            for x in 0..self.dx {
                for y in (0..self.dy).rev() {
                    let i = self.idx(x, y, z);
                    if self.is_void(i) {
                        continue;
                    }
                    let phi = self.porosity(i);
                    if phi <= 0.0 {
                        break;
                    }
                    let space = phi - self.sat[i] * phi;
                    let take = self.recharge.min(space.max(0.0));
                    self.sat[i] += take / phi;
                    self.admitted += take as f64;
                    break;
                }
            }
        }
    }

    /// Gravity drainage: every voxel has exactly one downward edge, so the
    /// outflow is bounded by its own water and no limiter is needed.
    fn percolate(&mut self) {
        let n = self.sat.len();
        for f in self.vflux[..n].iter_mut() {
            *f = 0.0;
        }
        for y in 1..self.dy {
            for z in 0..self.dz {
                for x in 0..self.dx {
                    let i = self.idx(x, y, z);
                    if self.is_void(i) {
                        continue;
                    }
                    let below = self.idx(x, y - 1, z);
                    let w = self.frozen_w[i];
                    if w <= 0.0 {
                        continue;
                    }
                    // A void below is a regime crossing, not a pore transfer:
                    // the whole conductance of THIS voxel applies and the
                    // receiving space is unbounded.
                    let (k, space) = if self.is_void(below) {
                        (self.perm(i), f32::INFINITY)
                    } else {
                        (self.perm(i).min(self.perm(below)), self.frozen_space[below])
                    };
                    let f = w.min(space).min(k * self.vertical_c);
                    self.vflux[i] = f.max(0.0);
                }
            }
        }
        for y in 0..self.dy {
            for z in 0..self.dz {
                for x in 0..self.dx {
                    let i = self.idx(x, y, z);
                    if self.is_void(i) {
                        // Water that reached open space left the bound regime.
                        continue;
                    }
                    let phi = self.porosity(i);
                    if phi <= 0.0 {
                        continue;
                    }
                    let mut w = self.frozen_w[i] - self.vflux[i];
                    if y + 1 < self.dy {
                        let above = self.idx(x, y + 1, z);
                        if !self.is_void(above) {
                            w += self.vflux[above];
                        } else {
                            // Free water standing on rock infiltrates: that is
                            // handled by `infiltrate`, not here.
                        }
                    }
                    self.sat[i] = (w / phi).clamp(0.0, 1.0);
                }
            }
        }
        // Account whatever fell into a void.
        for y in 0..self.dy {
            for z in 0..self.dz {
                for x in 0..self.dx {
                    let i = self.idx(x, y, z);
                    if self.is_void(i) && y + 1 < self.dy {
                        let above = self.idx(x, y + 1, z);
                        if !self.is_void(above) {
                            self.crossed_to_free += self.vflux[above] as f64;
                        }
                    }
                }
            }
        }
    }

    /// Hydraulic head of a voxel: elevation plus its own saturation. An
    /// unconfined proxy — a saturated voxel's head is its top face.
    #[inline]
    fn head(&self, i: usize, y: usize) -> f32 {
        y as f32 + self.sat[i]
    }

    /// Permeability-limited lateral redistribution.
    ///
    /// Each lateral edge's flux is computed **once**, from the unordered pair,
    /// and scaled by two frozen per-voxel limiters (`l_out` on the donor,
    /// `l_in` on the receiver). Both limiters are functions of the frozen
    /// state, so the edge value is identical read from either endpoint: mass is
    /// conserved exactly and the result cannot depend on traversal order.
    fn redistribute(&mut self) {
        let n = self.sat.len();
        for i in 0..n {
            self.gross_out[i] = 0.0;
            self.gross_in[i] = 0.0;
        }
        // Pass 1: gross edge magnitudes, to build the limiters.
        self.for_each_lateral_edge(|s, i, j, f| {
            if f > 0.0 {
                s.gross_out[i] += f;
                s.gross_in[j] += f;
            } else if f < 0.0 {
                s.gross_out[j] += -f;
                s.gross_in[i] += -f;
            }
        });
        for i in 0..n {
            let w = self.frozen_w[i];
            let space = self.frozen_space[i];
            self.l_out[i] = if self.gross_out[i] > w {
                w / self.gross_out[i]
            } else {
                1.0
            };
            self.l_in[i] = if self.gross_in[i] > space {
                space / self.gross_in[i]
            } else {
                1.0
            };
        }
        // Pass 2: apply. `net` accumulates into gross_in (reused as scratch).
        for i in 0..n {
            self.gross_in[i] = 0.0;
        }
        self.for_each_lateral_edge(|s, i, j, f| {
            let (src, dst, mag) = if f > 0.0 {
                (i, j, f)
            } else if f < 0.0 {
                (j, i, -f)
            } else {
                return;
            };
            let a = mag * s.l_out[src] * s.l_in[dst];
            s.gross_in[src] -= a;
            s.gross_in[dst] += a;
        });
        for i in 0..n {
            if self.is_void(i) {
                continue;
            }
            let phi = self.porosity(i);
            if phi <= 0.0 {
                continue;
            }
            let w = self.frozen_w[i] + self.gross_in[i];
            self.sat[i] = (w / phi).clamp(0.0, 1.0);
        }
    }

    /// Visit each unordered lateral edge exactly once, with the raw signed flux
    /// (positive = `i` → `j`). `i` is always the lower linear index.
    fn for_each_lateral_edge<F: FnMut(&mut Self, usize, usize, f32)>(&mut self, mut f: F) {
        for y in 0..self.dy {
            for z in 0..self.dz {
                for x in 0..self.dx {
                    let i = self.idx(x, y, z);
                    if self.is_void(i) {
                        continue;
                    }
                    for (nx, nz) in [(x + 1, z), (x, z + 1)] {
                        if nx >= self.dx || nz >= self.dz {
                            continue;
                        }
                        let j = self.idx(nx, y, nz);
                        if self.is_void(j) {
                            continue;
                        }
                        let k = self.perm(i).min(self.perm(j));
                        if k <= 0.0 {
                            continue;
                        }
                        let dh = self.head(i, y) - self.head(j, y);
                        let raw = k * dh * self.lateral_c;
                        if raw != 0.0 {
                            f(self, i, j, raw);
                        }
                    }
                }
            }
        }
    }

    /// Seepage faces: a drain voxel dumps whatever reached it out of the bound
    /// regime (a spring, a dug void, a pumped shaft).
    fn seep(&mut self) {
        for i in 0..self.sat.len() {
            if self.drain[i] && self.sat[i] > 0.0 {
                self.crossed_to_free += (self.sat[i] * self.porosity(i)) as f64;
                self.sat[i] = 0.0;
            }
        }
    }

    /// Run `n` relaxation steps.
    pub fn relax(&mut self, n: u32) {
        for _ in 0..n {
            self.step();
        }
    }
}
