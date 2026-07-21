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
use super::tectonics::Plate;
use crate::pregen::{CELL_VOXELS, CellGrid, Pregen};

/// Target (finest) deep-time cell edge, metres — S9's A tier resolution.
pub const DEEP_CELL_M: f64 = 460.0;
/// Cap on the deep grid width (cells per side). Holds the boot ritual to
/// ~300 k cells / ~15 s regardless of extent; larger worlds get a coarser cell
/// rather than a minutes-long, RAM-heavy run (see module docs § Resolution).
pub const DEEP_MAX_WIDTH: usize = 550;
/// Fixed iteration schedule (S9's A — no convergence check in the sim logic).
pub const DEEP_ITERATIONS: u32 = 200;

/// Gen-time overrides for the production [`DeepConfig`] flags a world can be
/// booted with. Each field is an `Option`; `None` **inherits the production
/// default** ([`production_config`]). An all-`None` (`Default`) `DeepOverrides`
/// therefore yields a config — and so a [`DeepField`] — byte-identical to
/// production, which is what keeps every already-created world reproducible.
///
/// This override channel is the whole point of the deep-config plumbing slice:
/// a launch flag can flip `tectonic_history` / `full_agents` on and dial the
/// orogenic amplitude *without* touching `production_config`'s production
/// defaults and *without* extending [`crate::pregen::WorldParams`] (a bare
/// `{ seed, extent }` literal at ~30 call sites).
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepOverrides {
    /// Override [`DeepConfig::tectonic_history`]: the analytic tectonic-history
    /// bundle (chapters, crustal columns, smoothed Airy isostasy, drainage
    /// export). `None` = production default (**on** since the U8 flip,
    /// journal/0044); pass `Some(false)` to reach the legacy off path.
    pub tectonic_history: Option<bool>,
    /// Override [`DeepConfig::full_agents`]: the wind + frost + wave erosion
    /// roster. `None` = production default (off).
    pub full_agents: Option<bool>,
    /// Override [`DeepConfig::thickening_scale`]: the orogenic amplitude the
    /// analytic tectonic forcing multiplies (m/iter for a unit-rate boundary).
    /// Only bites when tectonic history is on. `None` = production default.
    pub thickening_scale: Option<f64>,
}

impl DeepOverrides {
    /// True when no override is set — the all-inherit case whose config is
    /// byte-identical to [`production_config`].
    pub fn is_empty(&self) -> bool {
        self.tectonic_history.is_none()
            && self.full_agents.is_none()
            && self.thickening_scale.is_none()
    }
}

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
        // **Tectonic history ON — U8 GO** (DECIDED 2026-07-21, user;
        // tectonics.md § U8, journal/0044). The user ratified the flip
        // *without* gating it on the appearance walk: "I'm going to tell you to
        // flip tectonics regardless of what it looks like because I want us to
        // make progress. We can correct mistakes later. The flip is pomp." So
        // production now runs the chaptered kinematic history — plate advection,
        // analytic boundary forcing, crustal columns, smoothed-Airy isostasy,
        // per-chapter drainage re-march — and the field KEEPS its tectonic-only
        // exports (`recv`/`area`/`lake` drainage, `exhum`/`t_crust`, the
        // `chapters` table) that were empty before. Same event class as the
        // biotic/erodibility flips above: this CHANGES TERRAIN SHAPE — and
        // strata, drainage, exhumation — for every world created from here on;
        // worlds made before this flip are not reproducible under it. The
        // orogenic amplitude (`thickening_scale`) rides at the `DeepConfig`
        // default of 80: U7 is deferred, because corrections #23 measured the
        // knob to buy no sub-km relief either way (it lifts the continent, it
        // does not make mountains), so its value is a later call.
        tectonic_history: true,
        ..DeepConfig::default()
    }
}

/// The production config with gen-time [`DeepOverrides`] applied on top: start
/// from [`production_config`], then overwrite each flag the caller set. Every
/// `None` override inherits, so `production_config_with(cells, seed,
/// &DeepOverrides::default())` is **byte-identical** to `production_config(cells,
/// seed)` (asserted in the tests). This is the single seam a launch flag reaches
/// the deep-time run through.
pub fn production_config_with(
    cells: &CellGrid,
    seed: u64,
    overrides: &DeepOverrides,
) -> DeepConfig {
    let mut cfg = production_config(cells, seed);
    if let Some(v) = overrides.tectonic_history {
        cfg.tectonic_history = v;
    }
    if let Some(v) = overrides.full_agents {
        cfg.full_agents = v;
    }
    if let Some(v) = overrides.thickening_scale {
        cfg.thickening_scale = v;
    }
    cfg
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
    /// **Exported final drainage** (§ 7.3 — tectonic-history only; empty
    /// otherwise). `recv[i]` is the D8 receiver of the last routing (`-1` = sink),
    /// `area[i]` the contributing area / discharge, `lake[i]` a depression-filled
    /// cell at the final sea stand. Consumers: the frozen macro drainage topology
    /// (3e-2 decision 1), the water-table pinning lattice (corrections #15), and
    /// the body graph's initial lakes/sea (water.md § S11). Retiring the stale
    /// pregen chord network of § 7.1(b) as the collapse carving source.
    pub recv: Vec<i32>,
    pub area: Vec<f64>,
    pub lake: Vec<bool>,
    /// **Exhumation** (m) and **crustal thickness** (m) per cell — the metamorphic-
    /// grade axes the collapse tier WILL read (§ 6.4): exported and, as of U8,
    /// populated in every production world, but currently consumed by nothing.
    /// The expression slice is Sequenced (ROADMAP: "tectonic expression at the
    /// collapse tier" — consume `exhum`/`t_crust` into metamorphic-grade classes;
    /// stubs.md § 4, "the absent metamorphic expresser"). Empty when tectonic
    /// history is off.
    pub exhum: Vec<f64>,
    pub t_crust: Vec<f64>,
    /// **The chapter table** (§ 8): plate state per chapter. Per-unit deformation
    /// (dip, provenance, fault traces) is *intended* to re-derive analytically
    /// from it at collapse resolution — the ~5 KB that would replace stored
    /// per-cell dip vectors — but that re-derivation is the same Sequenced
    /// collapse-tier slice as above (chapters → strata dip/fold/fault in cut
    /// faces); today the table is exported and read by nothing. Empty when
    /// tectonic history is off.
    pub chapters: Vec<Vec<Plate>>,
}

/// Run the always-on deep-time sim from the coarse grid and distil it to a
/// [`DeepField`]. Uses the **byte-identical parallel path** (S9b): the per-cell
/// phases fork across cells but reproduce the scalar result to the bit, so the
/// field is deterministic in `(cells, seed)`.
pub fn build_field(cells: &CellGrid, seed: u64) -> DeepField {
    build_field_cfg(cells, &production_config(cells, seed))
}

/// Run the always-on deep-time sim under the production config with gen-time
/// [`DeepOverrides`] applied, and distil it to a [`DeepField`]. Mirrors
/// [`build_field`] but through [`production_config_with`], so
/// `build_field_with(cells, seed, &DeepOverrides::default())` is byte-identical
/// to `build_field(cells, seed)` (asserted in the tests). The production pregen
/// pass calls this with the world's chosen overrides.
pub fn build_field_with(cells: &CellGrid, seed: u64, overrides: &DeepOverrides) -> DeepField {
    build_field_cfg(cells, &production_config_with(cells, seed, overrides))
}

/// Run the deep-time sim under an explicit [`DeepConfig`] and distil the field —
/// the entry the tectonic-history spike uses to exercise the flag. On the
/// tectonic-history path the drainage export and crustal/chapter planes are
/// populated; off, they are empty (and `surf`/`strata` are byte-identical to
/// [`build_field`]). Uses the byte-identical parallel path.
pub fn build_field_cfg(cells: &CellGrid, cfg: &DeepConfig) -> DeepField {
    let run = super::run_cells(cells, cfg, true);
    let w = run.grid.w;
    let cell_m = run.grid.cell_m;
    let surf: Vec<f64> = run
        .grid
        .r
        .iter()
        .zip(&run.grid.h)
        .map(|(r, h)| r + h)
        .collect();
    let (recv, area, lake, exhum, t_crust, chapters) = if cfg.tectonic_history {
        let recv = run.erosion.recv().to_vec();
        let area = run.erosion.area().to_vec();
        let filled = run.erosion.filled();
        let routed = run.erosion.routed_surface();
        // A lake is a cell whose depression fill sits above its own surface at the
        // final routing (a closed basin holding standing water).
        let lake: Vec<bool> = (0..w * w)
            .map(|i| filled[i] > routed[i] + 1e-6 && routed[i] > super::SEA_LEVEL_M)
            .collect();
        (
            recv,
            area,
            lake,
            run.grid.exhum.clone(),
            run.grid.t_crust.clone(),
            run.chapters.clone(),
        )
    } else {
        (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    };
    let strata = run.grid.strata;
    DeepField {
        w,
        wp: cells.w as usize,
        cell_m,
        surf,
        strata,
        recv,
        area,
        lake,
        exhum,
        t_crust,
        chapters,
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
        (self.surf.len() + self.area.len() + self.exhum.len() + self.t_crust.len())
            * std::mem::size_of::<f64>()
            + self.recv.len() * std::mem::size_of::<i32>()
            + self.lake.len()
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
