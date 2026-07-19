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

pub mod climate;
pub mod erosion;
pub mod grid;
pub mod recorder;
pub mod refine;

pub use erosion::{Erosion, energy_band};
pub use grid::{DeepConfig, DeepGrid, SEA_LEVEL_M, build, provenance_uplift, sea_level_at};
pub use recorder::{Aridity, DeepStrata, DepEnv, DepTag, DepUnit, EnergyBand};
pub use refine::{DecayProfile, RegionSpec, measure_decay};

use crate::pregen::Pregen;

/// A completed deep-time run: the evolved grid plus the mass ledger for the
/// conservation check.
pub struct DeepRun {
    pub grid: DeepGrid,
    pub erosion: Erosion,
    /// Total uplift added over the run (`Σ over iterations of Σ uplift`).
    pub uplift_total: f64,
    /// `ΣR + ΣH` before the first iteration.
    pub mass_before: f64,
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
    let mut grid = build(pregen, cfg);
    let mut erosion = Erosion::new(&grid);
    let mass_before = total_mass(&grid);
    climate::march(&mut grid, grid::sea_level_at(cfg, 0));
    let mut uplift_total = 0.0;
    for it in 0..cfg.iterations {
        let sl = grid::sea_level_at(cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            climate::march(&mut grid, sl);
        }
        uplift_total += erosion.step(&mut grid, cfg, sl);
    }
    DeepRun {
        grid,
        erosion,
        uplift_total,
        mass_before,
        iterations: cfg.iterations,
    }
}
