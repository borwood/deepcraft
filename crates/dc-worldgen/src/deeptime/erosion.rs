//! The two-plane erosion engine: bedrock stock `R` and alluvium stock `H`
//! evolved by uplift, priority-flood drainage, mass-conserving stream-power
//! transport with alluvial-cover shielding, bedrock weathering, and hillslope
//! diffusion — the SPACE-family model, stripped to the mechanism (orogeny recon
//! § erosion, earth-processes.md § 4).
//!
//! **Mass is conserved explicitly down the receiver chain.** Every metre that
//! leaves a cell (entrained alluvium or incised bedrock) becomes suspended
//! flux; every metre the flow can no longer carry is deposited as alluvium
//! downstream; whatever reaches a sink (sea or domain border) is deposited
//! there. The only external input is uplift, so over `N` iterations
//! `Δ(ΣR + ΣH) = N · Σuplift` — the falsifier the mass-conservation test asserts.
//!
//! **Never incise below the receiver**: bedrock lowering at a cell is clamped
//! so its bedrock top cannot drop below the receiver's surface — no runaway
//! knickpoint digs a hole its own outlet can't drain.
//!
//! Determinism: a fixed scan order everywhere, no wall clock, no ambient
//! entropy (the only draws are the addressed initial-roughness jitter in
//! grid.rs). A timing harness may wrap `Instant` *around* a run, never inside.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::grid::{DeepConfig, DeepGrid, SEA_LEVEL_M};
use super::recorder::{Aridity, DepEnv, DepTag, EnergyBand};

/// Strictly-descending fill increment (metres) — as in pregen hydrology.
const EPS: f64 = 0.001;

const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const NEIGH4: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// A priority-flood heap item ordered by filled elevation (min-heap via
/// `Reverse`), ties broken by index for determinism.
#[derive(PartialEq)]
struct Item {
    filled: f64,
    idx: u32,
}
impl Eq for Item {}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.filled
            .total_cmp(&other.filled)
            .then(self.idx.cmp(&other.idx))
    }
}

/// Reusable scratch for the erosion iteration (allocated once, reused every
/// step — the per-iteration working set the memory measurement counts).
pub struct Erosion {
    w: usize,
    n: usize,
    cell_m: f64,
    /// Sea level for the current step (set by [`Erosion::step`]); the dynamic
    /// paleo-sea-level stand drives shoreline transgression/regression, so a
    /// coastal column records alternating marine/subaerial bands (the classic
    /// layered cliff — earth-processes.md § 6). Mean is [`SEA_LEVEL_M`].
    sea_level: f64,
    surf: Vec<f64>,
    filled: Vec<f64>,
    done: Vec<bool>,
    recv: Vec<i32>,
    area: Vec<f64>,
    qs: Vec<f64>,
    order: Vec<u32>,
    dh: Vec<f64>,
    energy: Vec<f64>,
    scale: Vec<f64>,
    heap: BinaryHeap<Reverse<Item>>,
}

impl Erosion {
    pub fn new(grid: &DeepGrid) -> Self {
        let n = grid.w * grid.w;
        Self {
            w: grid.w,
            n,
            cell_m: grid.cell_m,
            sea_level: SEA_LEVEL_M,
            surf: vec![0.0; n],
            filled: vec![0.0; n],
            done: vec![false; n],
            recv: vec![-1; n],
            area: vec![0.0; n],
            qs: vec![0.0; n],
            order: Vec::with_capacity(n),
            dh: vec![0.0; n],
            energy: vec![0.0; n],
            scale: vec![0.0; n],
            heap: BinaryHeap::new(),
        }
    }

    /// Working-set footprint of the scratch arrays (bytes), for the memory
    /// budget — separate from the grid's own resident state.
    pub fn scratch_bytes(&self) -> usize {
        let f64s = self.surf.len()
            + self.filled.len()
            + self.area.len()
            + self.qs.len()
            + self.dh.len()
            + self.energy.len()
            + self.scale.len();
        f64s * 8 + self.recv.len() * 4 + self.order.capacity() * 4 + self.done.len()
    }

    #[inline]
    fn idx(&self, gx: i32, gy: i32) -> Option<usize> {
        if gx >= 0 && gy >= 0 && (gx as usize) < self.w && (gy as usize) < self.w {
            Some(gy as usize * self.w + gx as usize)
        } else {
            None
        }
    }

    #[inline]
    fn coords(&self, i: usize) -> (i32, i32) {
        ((i % self.w) as i32, (i / self.w) as i32)
    }

    #[inline]
    fn is_border(&self, i: usize) -> bool {
        let (gx, gy) = self.coords(i);
        gx == 0 || gy == 0 || gx as usize == self.w - 1 || gy as usize == self.w - 1
    }

    /// One deep-time iteration at the given `sea_level` stand. Returns the
    /// total uplift added this step (for the mass-conservation ledger).
    pub fn step(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, sea_level: f64) -> f64 {
        self.sea_level = sea_level;
        // 1. Uplift (metres/iteration), into bedrock.
        let mut uplift_added = 0.0;
        for i in 0..self.n {
            grid.r[i] += grid.uplift[i];
            uplift_added += grid.uplift[i];
        }

        // 2. Surface, priority-flood fill, D8 receivers, drainage area.
        for i in 0..self.n {
            self.surf[i] = grid.surf_at(i);
        }
        self.flood();
        self.route();
        self.accumulate_area();

        // 3. Mass-conserving stream-power transport pass (upstream → down).
        self.dh.iter_mut().for_each(|d| *d = 0.0);
        self.transport(grid, cfg);

        // 4. Bedrock weathering (subaerial): bedrock → regolith, cover-tapered.
        for i in 0..self.n {
            if grid.surf_at(i) > self.sea_level {
                let w = cfg.weathering * (-grid.h[i] / cfg.h_star).exp();
                grid.r[i] -= w;
                grid.h[i] += w;
                self.dh[i] += w;
            }
        }

        // 5. Hillslope diffusion of the regolith (conserving, flux-limited).
        self.diffuse(grid, cfg);

        // 6. Record net thickness change under the environment measured now.
        if cfg.record {
            self.record(grid);
        }
        uplift_added
    }

    /// Priority-flood depression fill (Barnes 2014) seeded from sea and border.
    fn flood(&mut self) {
        self.heap.clear();
        self.order.clear();
        for f in &mut self.filled {
            *f = f64::INFINITY;
        }
        self.done.iter_mut().for_each(|d| *d = false);
        for i in 0..self.n {
            if self.surf[i] <= self.sea_level || self.is_border(i) {
                self.filled[i] = self.surf[i];
                self.heap.push(Reverse(Item {
                    filled: self.filled[i],
                    idx: i as u32,
                }));
            }
        }
        while let Some(Reverse(item)) = self.heap.pop() {
            let i = item.idx as usize;
            if self.done[i] {
                continue;
            }
            self.done[i] = true;
            self.order.push(i as u32);
            let (gx, gy) = self.coords(i);
            for (dx, dy) in NEIGH8 {
                let Some(j) = self.idx(gx + dx, gy + dy) else {
                    continue;
                };
                if self.done[j] || self.filled[j].is_finite() {
                    continue;
                }
                self.filled[j] = self.surf[j].max(self.filled[i] + EPS);
                self.heap.push(Reverse(Item {
                    filled: self.filled[j],
                    idx: j as u32,
                }));
            }
        }
    }

    /// D8 steepest-descent receivers on filled elevation. Sea and border cells
    /// are sinks (`recv = -1`): incoming sediment settles there.
    fn route(&mut self) {
        for i in 0..self.n {
            if self.surf[i] <= self.sea_level || self.is_border(i) {
                self.recv[i] = -1;
                continue;
            }
            let (gx, gy) = self.coords(i);
            let mut best: Option<(f64, usize)> = None;
            for (dx, dy) in NEIGH8 {
                let Some(j) = self.idx(gx + dx, gy + dy) else {
                    continue;
                };
                if self.filled[j] < self.filled[i] && best.is_none_or(|(bf, _)| self.filled[j] < bf)
                {
                    best = Some((self.filled[j], j));
                }
            }
            self.recv[i] = best.map_or(-1, |(_, j)| j as i32);
        }
    }

    /// Drainage area (in cell units) accumulated downstream: process cells in
    /// descending filled order (reverse of the flood pop order), so a receiver
    /// has all its upstream contributions before it is itself routed on.
    fn accumulate_area(&mut self) {
        self.area.iter_mut().for_each(|a| *a = 1.0);
        for k in (0..self.order.len()).rev() {
            let i = self.order[k] as usize;
            let rc = self.recv[i];
            if rc >= 0 {
                self.area[rc as usize] += self.area[i];
            }
        }
    }

    /// Stream-power transport with cover shielding and explicit flux routing.
    fn transport(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        self.qs.iter_mut().for_each(|q| *q = 0.0);
        self.energy.iter_mut().for_each(|e| *e = 0.0);
        for k in (0..self.order.len()).rev() {
            let c = self.order[k] as usize;
            let rc = self.recv[c];
            let qin = self.qs[c];
            if rc < 0 {
                // Sink: everything suspended settles here (marine / border).
                grid.h[c] += qin;
                self.dh[c] += qin;
                self.energy[c] = 0.0;
                continue;
            }
            let rc = rc as usize;
            let s = ((self.filled[c] - self.filled[rc]).max(0.0)) / self.cell_m;
            let ae = if (cfg.m_exp - 0.5).abs() < 1e-9 {
                self.area[c].sqrt()
            } else {
                self.area[c].powf(cfg.m_exp)
            };
            let sn = if (cfg.n_exp - 1.0).abs() < 1e-9 {
                s
            } else {
                s.powf(cfg.n_exp)
            };
            let cap = cfg.k_transport * ae * sn;
            self.energy[c] = cap;
            let qs_out = if qin <= cap {
                let mut room = cap - qin;
                // Entrain alluvium first (transport-limited).
                let ent = grid.h[c].min(room);
                grid.h[c] -= ent;
                self.dh[c] -= ent;
                room -= ent;
                let mut carried = qin + ent;
                // Then incise bedrock, shielded by remaining cover.
                if room > 0.0 {
                    let shield = (-grid.h[c] / cfg.h_star).exp();
                    let inc_pot = cfg.k_bedrock * ae * sn * shield;
                    let floor = grid.surf_at(rc);
                    let max_inc = (grid.r[c] - floor).max(0.0);
                    let inc = inc_pot.min(room).min(max_inc);
                    grid.r[c] -= inc;
                    carried += inc;
                }
                carried
            } else {
                // Over capacity: deposit the excess as alluvium.
                let dep = qin - cap;
                grid.h[c] += dep;
                self.dh[c] += dep;
                cap
            };
            self.qs[rc] += qs_out;
        }
    }

    /// Flux-limited hillslope diffusion of the regolith along the surface
    /// gradient. Fluxes are computed from a frozen surface then applied, so the
    /// exchange is symmetric and conserves `ΣH` exactly; a cell never sheds more
    /// than it has (the scale factor caps outflux at `H`).
    fn diffuse(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        if cfg.diffusion <= 0.0 {
            return;
        }
        for i in 0..self.n {
            self.surf[i] = grid.surf_at(i);
        }
        // Pass 1: outflux sum per cell, and the per-cell limiter scale.
        for i in 0..self.n {
            let (gx, gy) = self.coords(i);
            let mut out = 0.0;
            for (dx, dy) in NEIGH4 {
                if let Some(j) = self.idx(gx + dx, gy + dy) {
                    let d = self.surf[i] - self.surf[j];
                    if d > 0.0 {
                        out += cfg.diffusion * d;
                    }
                }
            }
            self.area[i] = out; // reuse `area` as outflux scratch
            self.scale[i] = if out > grid.h[i] && out > 0.0 {
                grid.h[i] / out
            } else {
                1.0
            };
        }
        // Pass 2: apply scaled directed fluxes downhill.
        for i in 0..self.n {
            let (gx, gy) = self.coords(i);
            let si = self.surf[i];
            for (dx, dy) in NEIGH4 {
                if let Some(j) = self.idx(gx + dx, gy + dy) {
                    let d = si - self.surf[j];
                    if d > 0.0 {
                        let f = cfg.diffusion * d * self.scale[i];
                        grid.h[i] -= f;
                        grid.h[j] += f;
                        self.dh[i] -= f;
                        self.dh[j] += f;
                    }
                }
            }
        }
    }

    /// Record each cell's net thickness change this iteration under the tag
    /// measured now: environment from the final surface, aridity from the
    /// marched precip, energy band from the stream capacity used.
    fn record(&mut self, grid: &mut DeepGrid) {
        for i in 0..self.n {
            let dh = self.dh[i];
            if dh.abs() < 1e-9 {
                continue;
            }
            if dh > 0.0 {
                let tag = self.tag_at(grid, i);
                grid.strata[i].deposit(tag, dh);
            } else {
                grid.strata[i].erode(-dh);
            }
        }
    }

    fn tag_at(&self, grid: &DeepGrid, i: usize) -> DepTag {
        let env = if grid.surf_at(i) <= self.sea_level {
            DepEnv::Subsea
        } else {
            DepEnv::Subaerial
        };
        let aridity = if f64::from(grid.precip[i]) < 0.32 {
            Aridity::Arid
        } else {
            Aridity::Humid
        };
        let energy = energy_band(self.energy[i]);
        DepTag {
            env,
            aridity,
            energy,
        }
    }
}

/// Map a stream transport capacity to a facies energy band. Thresholds are in
/// the capacity units of the transport pass (metres/iteration); calibrated so
/// headwater hillslopes read Low, trunk rivers read High.
pub fn energy_band(cap: f64) -> EnergyBand {
    if cap < 0.002 {
        EnergyBand::Low
    } else if cap < 0.02 {
        EnergyBand::Medium
    } else {
        EnergyBand::High
    }
}
