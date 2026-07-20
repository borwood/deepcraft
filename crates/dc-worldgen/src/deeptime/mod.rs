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
pub mod lithology;
pub mod recorder;
pub mod refine;

pub use biotic::{BioticSim, COAL_MIN_M, CellBiota, ROSTER, species_name};
pub use erosion::{Erosion, energy_band, flood_fill_serial, flood_fill_tiled};
pub use field::{
    DEEP_CELL_M, DEEP_ITERATIONS, DEEP_MAX_WIDTH, DeepField, build_field, production_config,
};
pub use grid::{
    DeepConfig, DeepGrid, SEA_LEVEL_M, build, build_cells, provenance_uplift, sea_level_at,
};
pub use lithology::{
    Agent, Litho, LithoResistance, REFERENCE_LITHO, exposed_litho, litho_of_tag,
    resistance_of_material, susceptibility_table,
};
pub use recorder::{Aridity, Biofacies, DeepStrata, DepEnv, DepTag, DepUnit, EnergyBand};
pub use refine::{DecayProfile, RegionSpec, measure_decay};

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
        Some(BioticSim::new(&mut grid, cfg.seed, parallel))
    } else {
        None
    };
    let mut uplift_total = 0.0;
    let mut biotic_total = 0.0;
    for it in 0..cfg.iterations {
        let sl = grid::sea_level_at(cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            climate::march(&mut grid, sl);
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
    DeepRun {
        grid,
        erosion,
        uplift_total,
        biotic_total,
        mass_before,
        biota,
        iterations: cfg.iterations,
    }
}
