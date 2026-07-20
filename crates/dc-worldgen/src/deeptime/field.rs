//! The production deep-time field: the distilled, always-on output of the
//! A-tier erosion sim that the collapse layer samples.
//!
//! 3e-1 promotes the S9 spike engine (`erosion`/`grid`/`recorder`) from a
//! measurement harness to a real pregen pass. `Pregen::run` now runs the
//! deep-time sim once at world creation (the "generating world history…"
//! ritual) and keeps its two useful outputs — the final eroded **surface**
//! (elevation) and the per-cell **strata record** (the tagged deposition log) —
//! as a [`DeepField`]. The collapse layer reads elevation from it (drives the
//! macro-terrain lattice) and reads the record from it (the depositional
//! formation-context source, replacing the year-zero climate shim for deep
//! strata — geology.md § formation context).
//!
//! The full `DeepGrid` (with uplift/precip planes and the erosion scratch) is
//! dropped after the run; only `surf` + `strata` survive into the world state.
//!
//! ## Resolution vs. extent (FLAG — a deviation from a literal fixed 460 m)
//!
//! S9's A tier is 460 m. At [`Extent::Medium`](crate::pregen::Extent::Medium)
//! that is a 545² grid, ~14 s / ~52 MiB — the ratified ritual. But cell count
//! grows with the *square* of world extent: a literal 460 m at
//! [`Extent::Large`](crate::pregen::Extent::Large) (~1017 km) is a 2211² ≈
//! 4.9 M-cell run — minutes and gigabytes, which also blows the
//! `pregen_time_vs_extent` <60 s budget. So the production config **caps the
//! grid width** at [`DEEP_MAX_WIDTH`]: Small/Medium keep 460 m as specified;
//! Large gets a coarser cell (~1.8 km) that holds the ritual budget. The
//! read-quality is coarser at Large (S9 measured true stories already at the
//! 1 km sweep), and the C-refinement slice (3e-2) is where landform detail for
//! approached Large regions comes back. FLAGGED for the integrating session.

use super::grid::DeepConfig;
use super::recorder::DeepStrata;
use crate::pregen::{CELL_VOXELS, CellGrid, Pregen};

/// Target (finest) deep-time cell edge, metres — S9's A tier resolution.
pub const DEEP_CELL_M: f64 = 460.0;
/// Cap on the deep grid width (cells per side). Holds the boot ritual to
/// ~300 k cells / ~15 s regardless of extent; larger worlds get a coarser cell
/// rather than a minutes-long, RAM-heavy run (see module docs § Resolution).
pub const DEEP_MAX_WIDTH: usize = 550;
/// Fixed iteration schedule (S9's A — no convergence check in the sim logic).
pub const DEEP_ITERATIONS: u32 = 200;

/// The production deep-time config for a given coarse grid: 460 m where it fits
/// under [`DEEP_MAX_WIDTH`], coarser for very large extents. Recorder on; the
/// sea-level/climate cycling defaults from [`DeepConfig`] drive read-quality.
pub fn production_config(cells: &CellGrid, seed: u64) -> DeepConfig {
    let wp = cells.w as f64;
    let extent_m = wp * CELL_VOXELS as f64 * 0.9;
    let cell_m = (extent_m / DEEP_MAX_WIDTH as f64).max(DEEP_CELL_M);
    DeepConfig {
        seed,
        cell_m,
        iterations: DEEP_ITERATIONS,
        record: true,
        // **S10 GO** (ecology.md § DECIDED 2026-07-20, user): biology is a
        // shipped part of world generation, not an experiment. The ritual grows
        // 15 s → 25 s at every extent (`DEEP_MAX_WIDTH` makes the +10 s flat,
        // not extent-scaled) and the world *keeps* only +6 MiB. The record's
        // organic facies now reach material selection through
        // `geology::deep_class`, so the coal the sim writes is coal a player
        // can dig.
        biotic: true,
        // **Erodibility ON** (ratified by the user 2026-07-20, journal/0030:
        // "flip it, i want to see"). Erosion is lithology-aware: the incision,
        // entrainment and — the term that carries the hillslope signal —
        // bedrock→regolith weathering rates are scaled per cell per epoch by
        // the resistance of the unit outcropping there (journal/0029,
        // corrections #17). This CHANGES TERRAIN SHAPE for every world created
        // from here on; worlds made before this flip are not reproducible under
        // it. The resistance is agent-specific by construction, so the karst,
        // cryosphere and littoral agents land by adding a term rather than by
        // renegotiating this one.
        erodibility: true,
        ..DeepConfig::default()
    }
}

/// The distilled deep-time output the world keeps: the eroded final surface and
/// the per-cell strata record, plus the coordinate bridge back to the pregen
/// grid. Sampled by the collapse layer (elevation + depositional context).
pub struct DeepField {
    /// Deep grid width (cells per side).
    pub w: usize,
    /// Pregen grid width (cells per side) — half of it centres the world.
    pub wp: usize,
    /// Deep cell edge, metres.
    pub cell_m: f64,
    /// Final surface elevation `R + H` per cell, metres (row-major, `w × w`).
    pub surf: Vec<f64>,
    /// Per-cell strata record, bottom-up units tagged at deposition.
    pub strata: Vec<DeepStrata>,
}

/// Run the always-on deep-time sim from the coarse grid and distil it to a
/// [`DeepField`]. Uses the **byte-identical parallel path** (S9b): the per-cell
/// phases fork across cells but reproduce the scalar result to the bit, so the
/// field is deterministic in `(cells, seed)`.
pub fn build_field(cells: &CellGrid, seed: u64) -> DeepField {
    let cfg = production_config(cells, seed);
    let run = super::run_cells(cells, &cfg, true);
    let grid = run.grid;
    let surf: Vec<f64> = grid.r.iter().zip(&grid.h).map(|(r, h)| r + h).collect();
    DeepField {
        w: grid.w,
        wp: cells.w as usize,
        cell_m: grid.cell_m,
        surf,
        strata: grid.strata,
    }
}

impl DeepField {
    /// Build directly from a [`Pregen`] (the spike/test convenience path).
    pub fn from_pregen(pregen: &Pregen) -> Self {
        build_field(&pregen.grid, pregen.seed)
    }

    /// Continuous deep-grid coordinates (cell centres at integer coords) for a
    /// world voxel, or `None` when the voxel lies outside the civilized pregen
    /// extent (the border wilds have no deep-time history — the collapse layer
    /// falls back to the analytic elevation there).
    #[inline]
    fn deep_coords(&self, vx: i64, vz: i64) -> Option<(f64, f64)> {
        let half = (self.wp / 2) as f64;
        // Continuous pregen-cell coordinate (integer = cell centre), matching
        // the collapse layer's `climate_at` convention.
        let px = vx as f64 / CELL_VOXELS as f64 + half - 0.5;
        let py = vz as f64 / CELL_VOXELS as f64 + half - 0.5;
        let wpf = self.wp as f64;
        if px < 0.0 || px > wpf - 1.0 || py < 0.0 || py > wpf - 1.0 {
            return None;
        }
        // Pregen coordinate → deep coordinate (inverse of grid::build_cells).
        let gx = (px + 0.5) / wpf * self.w as f64 - 0.5;
        let gy = (py + 0.5) / wpf * self.w as f64 - 0.5;
        Some((gx, gy))
    }

    /// Bilinear deep-time surface elevation (metres) at a world voxel, or `None`
    /// in the wilds. Continuous by construction — the elevation lattice can
    /// sample it per point without seams.
    pub fn surface_at_voxel(&self, vx: i64, vz: i64) -> Option<f64> {
        let (gx, gy) = self.deep_coords(vx, vz)?;
        Some(bilinear(&self.surf, self.w, gx, gy))
    }

    /// The strata record of the deep cell **nearest** the world voxel, or `None`
    /// in the wilds / when there is no record grid. Nearest (not bilinear): a
    /// variable-length unit sequence cannot be interpolated, so the facies story
    /// steps at the ~460 m deep-cell grid — a geologically legitimate scale,
    /// coarser than the 28.8 m chunk grid, so corrections #6's chunk-line
    /// cutover does not recur (the per-voxel member dither still smooths contacts
    /// *within* a facies). FLAGGED sampling choice.
    pub fn record_at_voxel(&self, vx: i64, vz: i64) -> Option<&DeepStrata> {
        if self.strata.is_empty() {
            return None;
        }
        let (gx, gy) = self.deep_coords(vx, vz)?;
        let ix = (gx.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        let iy = (gy.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        self.strata.get(iy * self.w + ix)
    }

    /// Rough resident footprint (bytes) — the honest "what the ritual keeps in
    /// memory" number.
    pub fn resident_bytes(&self) -> usize {
        self.surf.len() * std::mem::size_of::<f64>()
            + self.strata.len() * std::mem::size_of::<DeepStrata>()
            + self
                .strata
                .iter()
                .map(DeepStrata::heap_bytes)
                .sum::<usize>()
    }
}

/// Bilinear sample of a `w × w` field at continuous cell coordinates (cell
/// centres at integer coords), clamped at the border.
fn bilinear(field: &[f64], w: usize, gx: f64, gy: f64) -> f64 {
    let clamp = |v: f64| v.clamp(0.0, w as f64 - 1.0);
    let x = clamp(gx);
    let y = clamp(gy);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(w - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let at = |xx: usize, yy: usize| field[yy * w + xx];
    let a = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
    let b = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
    a * (1.0 - fy) + b * fy
}
