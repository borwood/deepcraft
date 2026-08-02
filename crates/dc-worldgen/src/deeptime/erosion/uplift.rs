//! **The vertical drivers** — uplift application, crustal thickening, the
//! isostatic response, and the exhumation bookkeeping that rides them. The only
//! external mass input to the two-plane model enters here.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::super::grid::{DeepConfig, DeepGrid};
use super::Erosion;

impl Erosion {
    /// Add the blended analytic thickening rate into the crustal columns
    /// (§ 4.2/§ 5.2). Positive rates thicken (orogeny/arc), negative thin
    /// (rift/trench/ridge); `t_crust` is floored so a column cannot thin to
    /// nothing. Purely per-cell → byte-identical parallel. Elevation is untouched
    /// here — that is isostasy's job.
    ///
    /// **`dt` is the phase length** (RATE, journal/0123): the forcing plane is a
    /// thickening *rate* per epoch, so a turn covering `dt` epochs adds `f × dt`.
    /// `dt = 1.0` is bit-identical to the pre-RATE pass.
    pub fn apply_thickening(&mut self, grid: &mut DeepGrid, dt: f64) {
        let forcing = &self.forcing;
        if forcing.is_empty() {
            return;
        }
        if self.par() {
            grid.t_crust
                .par_iter_mut()
                .zip(forcing.par_iter())
                .for_each(|(t, f)| *t = (*t + *f * dt).max(1000.0));
        } else {
            for (t, f) in grid.t_crust.iter_mut().zip(forcing.iter()) {
                *t = (*t + *f * dt).max(1000.0);
            }
        }
    }

    /// Snapshot bedrock before the erosion phases, so [`Self::track_exhumation`]
    /// can measure the R-lowering the phases produce (incision + weathering) and
    /// attribute it to the column — never the later isostatic motion.
    pub fn snapshot_bedrock(&mut self, grid: &DeepGrid) {
        self.r_snap.copy_from_slice(&grid.r);
    }

    /// Every metre of bedrock the erosion phases removed this step decrements the
    /// crustal thickness and grows cumulative exhumation (§ 5.2 — "every metre of
    /// bedrock converted or incised decrements `t_crust` and increments `exhum`").
    /// Deposition grows `H`, not `t_crust`, so it is not counted here. Per-cell →
    /// byte-identical parallel.
    pub fn track_exhumation(&mut self, grid: &mut DeepGrid) {
        let snap = &self.r_snap;
        if self.par() {
            grid.exhum
                .par_iter_mut()
                .zip(grid.t_crust.par_iter_mut())
                .zip(grid.r.par_iter())
                .zip(snap.par_iter())
                .for_each(|(((e, t), r), s)| {
                    let removed = (*s - *r).max(0.0);
                    *e += removed;
                    *t -= removed;
                });
        } else {
            for (((e, t), r), s) in grid
                .exhum
                .iter_mut()
                .zip(grid.t_crust.iter_mut())
                .zip(grid.r.iter())
                .zip(snap.iter())
            {
                let removed = (*s - *r).max(0.0);
                *e += removed;
                *t -= removed;
            }
        }
    }

    /// **Airy compensation of the smoothed load** (§ 6.1). Computes the flexural-
    /// wavelength-smoothed crust and sediment loads, derives the Airy equilibrium
    /// surface per cell, and relaxes bedrock toward it at `iso_rate`. Returns the
    /// summed injected ΔR (the mass-ledger external input). The smoothing is a
    /// fixed-order separable blur — scalar in both drivers, so isostasy is
    /// byte-identical scalar↔parallel and is the run's only new serial cost of
    /// note (§ SPIKE 3/8).
    pub fn isostasy(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) -> f64 {
        use super::super::isostasy;
        let radius = isostasy::flex_radius_cells(cfg.flex_wavelength_km, self.cell_m);
        let t_bar = isostasy::box_smooth(&grid.t_crust, self.w, radius);
        let h_bar = isostasy::box_smooth(&grid.h, self.w, radius);
        let rate = cfg.iso_rate;
        let mut injected = 0.0f64;
        for i in 0..self.n {
            let rho_c =
                isostasy::rho_crust(super::super::tectonics::CrustKind::from_index(grid.crust_kind[i]));
            let e_eq = isostasy::equilibrium(t_bar[i], h_bar[i], rho_c);
            let surf = grid.r[i] + grid.h[i];
            let d_r = rate * (e_eq - surf);
            grid.r[i] += d_r;
            injected += d_r;
        }
        injected
    }

    // ---- phase 1: uplift into bedrock -------------------------------------

    /// Add the per-cell uplift into bedrock. Returns the total added.
    ///
    /// **`dt` is the phase length** (RATE, journal/0123): `grid.uplift` is metres
    /// *per epoch*, so a turn covering `dt` epochs adds `u × dt` — and the ledger
    /// total it returns scales with it, or the mass-conservation check would be
    /// reading a different amount of time than the grid got. `dt = 1.0` is
    /// bit-identical to the pre-RATE pass.
    pub fn apply_uplift(&mut self, grid: &mut DeepGrid, dt: f64) -> f64 {
        if self.par() {
            grid.r
                .par_iter_mut()
                .zip(grid.uplift.par_iter())
                .for_each(|(r, u)| *r += *u * dt);
        } else {
            for i in 0..self.n {
                grid.r[i] += grid.uplift[i] * dt;
            }
        }
        self.uplift_sum * dt
    }

    // ---- phase 1b: expose lithology (PARALLEL — per-cell independent) ------
}
