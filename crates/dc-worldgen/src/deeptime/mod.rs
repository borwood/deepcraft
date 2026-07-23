//! **S9 spike — the deep-time tier.** A between-pregen-and-collapse epoch sim
//! that runs two-plane erosion over eons on a grid finer than the 14.7 km
//! pregen cell, recording a per-cell strata history whose units are tagged by
//! the environment measured at deposition. The deliverable is *measured
//! numbers and a defensible recommendation* on the A/B/C architecture question
//! (docs/design/earth-processes.md § S9, docs/spikes/S9-results.md), not a
//! shipped feature — nothing here touches the live collapse path or pregen
//! byte-identity.
//!
//! Pipeline per run:
//! 1. [`grid::build`] — resample pregen tectonics to the deep-time grid
//!    (bilinear elevation + uplift, addressed roughness jitter).
//! 2. [`climate::march`] — orographic precipitation on the current topography,
//!    re-marched periodically.
//! 3. [`erosion::Erosion::step`] × N — uplift, priority-flood drainage,
//!    mass-conserving stream-power transport with cover shielding, weathering,
//!    hillslope diffusion, and net-ΔH recording.
//!
//! Architecture candidates the spike weighs: **A** one coarse global mid-tier,
//! **B** landform-resolution global (~48 m, ~27 M cells), **C** coarse global +
//! bounded regional refinement (the halo theorem). See [`refine`] for the C
//! machinery and the decay-length measurement.

pub mod biotic;
pub mod climate;
pub mod erosion;
pub mod field;
pub mod grid;
pub mod isostasy;
pub mod lithology;
pub mod providers;
pub mod recorder;
pub mod refine;
pub mod tectonics;
pub mod weather_behavior;

pub use biotic::{
    BioticSim, COAL_BURIAL_M, COAL_MIN_M, COAL_ONSET_C, CellBiota, ROSTER, species_name,
};
pub use erosion::{Erosion, energy_band, flood_fill_serial, flood_fill_tiled};
pub use field::{
    DEEP_CELL_M, DEEP_ITERATIONS, DEEP_MAX_WIDTH, DeepField, DeepOverrides, build_field,
    build_field_cfg, build_field_with, production_config, production_config_with,
};
pub use grid::{
    DeepConfig, DeepGrid, SEA_LEVEL_M, build, build_cells, provenance_uplift, sea_level_at,
};
pub use lithology::{
    Agent, Litho, LithoResistance, REFERENCE_LITHO, blend_susceptibility, dominant_litho,
    exposed_litho, exposed_shares, litho_of_tag, resistance_of_material, susceptibility_table,
};
pub use providers::{BurialColumn, BuriedUnit, PaleoUnit, ParentCell, Providers, WaveCell};
pub use recorder::{Aridity, Biofacies, DeepStrata, DepEnv, DepTag, DepUnit, EnergyBand, Eolian};
pub use refine::{DecayProfile, RegionSpec, measure_decay};
pub use tectonics::{BoundaryKind, CrustKind, Plate};
pub use weather_behavior::{
    BedrockWeather, Form, Transform, Weather, WeatherAxis, WeatherCtx, WeatheringPass,
    weather_one_cell,
};

use crate::pregen::{CellGrid, Pregen};

/// A completed deep-time run: the evolved grid plus the mass ledger for the
/// conservation check.
pub struct DeepRun {
    pub grid: DeepGrid,
    pub erosion: Erosion,
    /// Total uplift added over the run (`Σ over iterations of Σ uplift`).
    pub uplift_total: f64,
    /// Total **biotic** mass added over the run (organic + charcoal burial —
    /// carbon fixed from the atmosphere, so a genuine external input like
    /// uplift). Zero when the biotic layer is off. The mass ledger is therefore
    /// `Δ(ΣR + ΣH) == uplift_total + biotic_total`.
    pub biotic_total: f64,
    /// `ΣR + ΣH` before the first iteration.
    pub mass_before: f64,
    /// The biotic state at the end of the run (`None` when the layer was off).
    /// Held so the spike harness can read community vectors; the production
    /// field drops it.
    pub biota: Option<BioticSim>,
    pub iterations: u32,
    /// **Total thickening applied** over the run (`Σ over iters of Σ blended
    /// forcing`, before the `t_crust` floor). Zero when tectonic history is off.
    /// With `exhum_total` this closes the thickness ledger `Δ(Σt_crust) ≈
    /// thickening_total − exhum_total` (§ 3.4), reported in the spike.
    pub thickening_total: f64,
    /// **Total bedrock exhumed** over the run (`Σ grid.exhum` at finalize). Zero
    /// when tectonic history is off.
    pub exhum_total: f64,
    /// **The chapter table** (§ 8): plate state at the start of each chapter
    /// (`K + 1` entries; the last is one past the final chapter for the ramp).
    /// Empty when tectonic history is off. ~5 KB — the entire tectonic history of
    /// a world, from which per-chapter deformation is re-derivable analytically.
    pub chapters: Vec<Vec<Plate>>,
}

/// Sum of bedrock + alluvium over the whole grid (the conserved quantity, up
/// to uplift input).
pub fn total_mass(grid: &DeepGrid) -> f64 {
    grid.r.iter().sum::<f64>() + grid.h.iter().sum::<f64>()
}

/// Build the deep-time grid from a pregenerated world and run the fixed
/// iteration schedule. Deterministic in `(pregen, cfg)`.
pub fn run(pregen: &Pregen, cfg: &DeepConfig) -> DeepRun {
    run_with(pregen, cfg, false)
}

/// Like [`run`], but with the data-parallel per-cell phases toggled. The
/// parallel path is **byte-identical** to the scalar path (S9b); this exists so
/// the harness can measure both and the determinism test can compare them.
pub fn run_with(pregen: &Pregen, cfg: &DeepConfig, parallel: bool) -> DeepRun {
    run_cells(&pregen.grid, cfg, parallel)
}

/// Like [`run_with`], but from the coarse [`CellGrid`] alone — the entry point
/// the production pregen pass uses, since it runs *inside* pregen (before a
/// whole [`Pregen`] exists). Same fixed iteration schedule, deterministic in
/// `(cells, cfg)`.
pub fn run_cells(cells: &CellGrid, cfg: &DeepConfig, parallel: bool) -> DeepRun {
    let mut grid = build_cells(cells, cfg);
    let mut erosion = Erosion::new(&grid);
    erosion.set_parallel(parallel);
    let mass_before = total_mass(&grid);
    climate::march(&mut grid, grid::sea_level_at(cfg, 0));
    // The S10 biotic layer, when enabled: initializing it turns on the grid's
    // biotic modifier planes at their identity values, so iteration 0's erosion
    // is byte-identical to a biology-free run (the lagged coupling, below).
    let mut biota = if cfg.biotic {
        Some(BioticSim::new(&mut grid, cfg, parallel))
    } else {
        None
    };

    // Tectonic history (§ 3): precompute the chapter table and the analytic
    // thickening forcing plane per chapter geometry (repaint-per-chapter, § 3.1),
    // to be blended per iteration across the chapter ramp. Off → empty, and the
    // step's forcing plane stays empty (byte-identical legacy path).
    let tec = TectonicSchedule::new(cfg, &grid);

    let mut uplift_total = 0.0;
    let mut biotic_total = 0.0;
    let mut thickening_total = 0.0;
    let mut blended = if cfg.tectonic_history {
        vec![0.0f64; grid.w * grid.w]
    } else {
        Vec::new()
    };
    for it in 0..cfg.iterations {
        let sl = grid::sea_level_at(cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            climate::march(&mut grid, sl);
        }
        if cfg.tectonic_history {
            let chapter = tec.blend_into(cfg, it, &mut blended);
            thickening_total += blended.iter().sum::<f64>();
            erosion.set_tectonic(chapter, &blended);
        }
        // Erosion consumes the modifiers biology wrote LAST epoch...
        uplift_total += erosion.step(&mut grid, cfg, sl);
        // ...then biology reads this epoch's fresh terrain, deposits its organic
        // record, and writes the modifiers the NEXT epoch's erosion consumes.
        // That one-epoch lag is what breaks the biology↔erosion cycle a
        // single-epoch pass graph would (correctly) refuse — ecology.md § 3.
        if let Some(b) = biota.as_mut() {
            biotic_total += b.step(&mut grid, &erosion, it);
        }
    }
    // Burial diagenesis: buried thick peat becomes coal.
    if let Some(b) = biota.as_ref() {
        b.finalize(&mut grid);
    }
    let exhum_total = grid.exhum.iter().sum::<f64>();
    DeepRun {
        grid,
        erosion,
        uplift_total,
        biotic_total,
        mass_before,
        biota,
        iterations: cfg.iterations,
        thickening_total,
        exhum_total,
        chapters: tec.table,
    }
}

/// The precomputed tectonic-history schedule: the chapter table plus one analytic
/// thickening forcing plane per chapter geometry, blended per iteration across
/// the chapter ramp. Empty when tectonic history is off.
struct TectonicSchedule {
    /// Plate state at the start of each chapter (`K + 1` entries).
    table: Vec<Vec<Plate>>,
    /// One forcing plane (m/iter thickening) per chapter geometry, row-major.
    planes: Vec<Vec<f64>>,
}

impl TectonicSchedule {
    fn new(cfg: &DeepConfig, grid: &DeepGrid) -> Self {
        if !cfg.tectonic_history {
            return Self {
                table: Vec::new(),
                planes: Vec::new(),
            };
        }
        let w = grid.w;
        let cell_m = grid.cell_m;
        let extent_km = w as f64 * cell_m / 1000.0;
        let table = tectonics::chapter_table(cfg, extent_km);
        let v_ref = tectonics::reference_velocity(cfg, extent_km);
        let planes: Vec<Vec<f64>> = table
            .iter()
            .map(|plates| {
                let mut plane = vec![0.0f64; w * w];
                for gy in 0..w {
                    for gx in 0..w {
                        let x = (gx as f64 + 0.5) * cell_m / 1000.0;
                        let y = (gy as f64 + 0.5) * cell_m / 1000.0;
                        plane[gy * w + gx] = tectonics::forcing_at(plates, cfg, v_ref, x, y);
                    }
                }
                plane
            })
            .collect();
        Self { table, planes }
    }

    /// Blend the two chapter planes for iteration `it` into `out`, returning the
    /// chapter index the recorder should stamp. The forcing is always in transit:
    /// within chapter `c` it lerps from plane `c` toward plane `c+1` (`ramp`), or
    /// steps to plane `c` (the negative control, `ramp_chapters = false`).
    fn blend_into(&self, cfg: &DeepConfig, it: u32, out: &mut [f64]) -> u8 {
        let k = cfg.chapters.max(1);
        let ipc = f64::from(cfg.iterations.max(1)) / f64::from(k);
        let pos = f64::from(it) / ipc;
        let c = (pos.floor() as usize).min(k as usize - 1);
        let frac = if cfg.ramp_chapters {
            (pos - c as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let a = &self.planes[c];
        let b = &self.planes[(c + 1).min(self.planes.len() - 1)];
        for (o, (av, bv)) in out.iter_mut().zip(a.iter().zip(b.iter())) {
            *o = av * (1.0 - frac) + bv * frac;
        }
        c as u8
    }
}
