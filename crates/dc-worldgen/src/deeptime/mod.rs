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
pub mod cadence;
pub mod census;
pub mod climate;
pub mod erosion;
pub mod field;
pub mod flux;
pub mod geotherm;
pub mod grid;
pub mod head;
pub mod inventory;
pub mod isostasy;
pub mod lithology;
pub mod providers;
pub mod recorder;
pub mod refine;
pub mod runner;
pub mod schedule;
pub mod tectonics;
pub mod weather_behavior;
pub mod weather_inventory;

pub use biotic::{
    BioticSim, COAL_BURIAL_M, COAL_MIN_M, COAL_ONSET_C, CellBiota, ROSTER, species_name,
};
pub use cadence::{Cadence, CadenceTable};
pub use erosion::{
    CREEP_MAX_EDGE_COEFF, Erosion, MfdParams, competence_ceiling, energy_band, flood_fill_serial,
    flood_fill_tiled,
};
pub use field::{
    DEEP_CELL_M, DEEP_ITERATIONS, DEEP_MAX_WIDTH, DeepField, DeepOverrides, EROSION_CALIBRATION,
    build_field, build_field_cfg, build_field_cfg_cadence, build_field_with, production_config,
    production_config_with, scale_erosion_rates,
};
pub use flux::{
    FACE_SLOTS, FaceKey, FlowCause, FlowForm, FluidId, FluxAccum, FluxCensus, FluxEntry,
    FluxRecord, LATERAL_FACES, slot_for_chapter,
};
pub use geotherm::{
    BurialColumn, DEFAULT_CONTINENTAL_GRADIENT_C_PER_M, FIELD_TEMPERATURE, GEOTHERM_PERIOD,
    gradient_c_per_m, temperature_c,
};
pub use grid::{
    DeepConfig, DeepGrid, SEA_LEVEL_M, build, build_cells, provenance_uplift, sea_level_at,
};
pub use head::{
    AQUIFER_K_MIN, AQUITARD_K_MAX, BASEMENT_AQUIFER_M, CONFINING_CAP_M, ColumnHydro, FIELD_HEAD,
    FLUID_DENSITY_REL, HEAD_PERIOD, HEAD_RELAX_SWEEPS, STREAM_ANCHOR_AREA, column_hydro,
    permeability_of, vertical_exchange,
};
pub use inventory::{
    BEDROCK_SEAM_MATERIAL, BEDROCK_SEAM_THICKNESS_M, Cause, EdgeDict, EdgeDictEntry,
    EdgeDictMismatch, EdgeId, FORM_COUNT, Fact, FactLedger, FracM, Granularity, InvCtx, InvForm,
    InvSpan, LedgerField, LedgerView, Portion, StoredFrac, UnitProvenance, WorkingInventory,
    build_identity, build_working, collapse_top_voxel, commit_chapter, compose_bedrock,
    compose_unit, derive_base, derive_bedrock, is_declared_edge, quantize_to_eighths,
    stored_fold_tolerance, structure_stock_m, surface_regolith_m,
};
pub use inventory::{F32_RELATIVE_RESOLUTION, FOLD_DEPTH_HEADROOM, NEAR_ZERO_FLOOR};
pub use lithology::{
    Agent, Litho, LithoResistance, REFERENCE_LITHO, blend_susceptibility, dominant_litho,
    exposed_litho, exposed_shares, litho_of_tag, resistance_of_material, settling_table,
    susceptibility_table,
};
pub use providers::{PaleoUnit, ParentCell, Providers, WaveCell};
pub use recorder::{Aridity, Biofacies, DeepStrata, DepEnv, DepTag, DepUnit, EnergyBand, Eolian};
pub use refine::{DecayProfile, RegionSpec, measure_decay};
pub use schedule::Schedule;
pub use tectonics::{BoundaryKind, CrustKind, Plate};
pub use weather_behavior::{
    BedrockWeather, Form, Transform, Weather, WeatherAxis, WeatherCtx, WeatheringPass,
    weather_one_cell,
};
pub use weather_inventory::{
    WEATHERING_AGENTS, WeatherInputs, agent_share, empty_accumulator, finalize_ledgers,
    susceptibility, weather_bedrock_epoch, weather_cell, weather_column, weather_rate,
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
    /// **The inventory-weathering ledger record** (journal/0094) — the accumulating
    /// saprolite band the `dc:deep/weather_inventory` pass grew across the loop,
    /// re-keyed onto the final record (bedrock facts at `strata.units.len()`).
    /// **Empty** when `weather_inventory` is off (byte-identical). Indexed by the
    /// same cell as `grid.strata`; the `DeepField` carries it as its `ledgers`
    /// sidecar. One grid-wide record with the cell as a CSR row (journal/0102), not
    /// a per-cell owning struct.
    pub weather_ledgers: LedgerField,
    /// **The face-flux record** (FLOW slice 1, flow.md § 2) — per-chapter flux on
    /// 3D faces, the representation that replaces the exported receiver tree.
    /// Empty when `flow_record` is off.
    pub flux: flux::FluxRecord,
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
    run_cells_with_cadence(cells, cfg, parallel, &cadence::CadenceTable::empty())
}

/// [`run_cells`] with the world's **authored pass cadence** (the RATE axis,
/// `material-behavior.md` § 5; journal/0123).
///
/// An empty table — what [`run_cells`] passes — means every pass keeps the
/// cadence it declared, which is the shipped schedule, bit for bit. A non-empty
/// one is a world stating *"weathering ×5 while tectonics ×1"*, and the runner
/// executes it without deriving anything.
///
/// **This is the seam, not the format.** When authored ORDER lands (the pass
/// architecture's continuation slot (b)) it brings a per-world manifest with it,
/// and the manifest's cadence section constructs the [`CadenceTable`] handed
/// here. Inventing that format now would be building the general mechanism ahead
/// of its caller, so the parameter exists and the loader does not.
pub fn run_cells_with_cadence(
    cells: &CellGrid,
    cfg: &DeepConfig,
    parallel: bool,
    cadence: &cadence::CadenceTable,
) -> DeepRun {
    let mut grid = build_cells(cells, cfg);
    let mut erosion = Erosion::new(&grid);
    erosion.set_parallel(parallel);
    // FLOW slice 1: arm the transport pass's per-face load capture. Off ⇒ the
    // buffer stays empty and transport never touches it (byte-identical).
    erosion.set_flux_record(cfg.flow_record);
    // FLOW continuation (b/b'): the MFD partition under the hybrid-`p` law.
    // `None` ⇒ the single-receiver D8 solve, byte for byte;
    // `mfd_exponent_channel == mfd_exponent` ⇒ journal/0109's uniform `p`.
    erosion.set_mfd(cfg.mfd.then_some(MfdParams {
        p_hill: cfg.mfd_exponent,
        p_chan: cfg.mfd_exponent_channel,
        chi_lo: cfg.mfd_chi_lo,
        chi_hi: cfg.mfd_chi_hi,
        min_weight: cfg.mfd_min_weight,
    }));
    // Movement 2b: the load becomes a multiset of (lithology, quantity). Off ⇒
    // every species vector stays empty and the pass carries a scalar mass, byte
    // for byte.
    erosion.set_material_transport(cfg.material_transport);
    // journal/0111: the read-only denudation counters. Off ⇒ no branch fires and
    // the shoreline-creep sweep is never called — byte- and cost-identical.
    erosion.set_denudation_ledger(cfg.denudation_ledger);
    // Movement 2b continuation (b): the gravity/mass-wasting member of the same
    // family. **After** `set_material_transport`, which it is gated on — creep
    // moves the composition that pass publishes. Off ⇒ the creep plane stays empty
    // and the diffusion phase is anonymous again, byte for byte.
    erosion.set_material_creep(cfg.material_creep);
    let mass_before = total_mass(&grid);

    // --- pre-loop construction (owned state the epoch loop threads) ---
    // The biotic-layer init (which turns on the grid's biotic modifier planes at
    // their identity values, so iteration 0's erosion is byte-identical to a
    // biology-free run — the lagged coupling), and the precomputed
    // tectonic-history schedule (§ 3: the chapter table + one analytic thickening
    // plane per chapter geometry, blended per iteration across the chapter ramp;
    // off → empty, byte-identical legacy path).
    //
    // **THREE PRE-LOOP PASS SEEDS USED TO SIT HERE, AND THEY ARE GONE**
    // (journal/0124; `ARCHITECTURE.md` § *Schedule — DECIDED 2026-07-29*). The
    // climate march, the geotherm and the head field were each run once here and
    // then skipped at epoch 0 by the runner. Deleting the skip rule deleted their
    // reason to exist: each of the three re-establishes its whole field from the
    // current state, and each now does so *inside* epoch 0, ahead of its own only
    // reader — so the pre-loop copy was a write nothing observed. **The seeds were
    // repair for the skip, not initial conditions**, and the audit that found so
    // is in journal/0124, per pass.
    //
    // What remains below is genuinely different in kind: it **constructs owned
    // state** that the ctx carries, rather than writing a grid plane a pass will
    // overwrite. When declared epochs land (ROADMAP slot (d)) this is the material
    // for a real `Schedule::Seed` roster; today it is not pass-shaped.
    let biota = if cfg.biotic {
        Some(BioticSim::new(&mut grid, cfg, parallel))
    } else {
        None
    };
    let tec = TectonicSchedule::new(cfg, &grid);
    let blended = if cfg.tectonic_history {
        vec![0.0f64; grid.w * grid.w]
    } else {
        Vec::new()
    };

    // --- the deep-time pass-runner drives the epoch loop (runner.rs) ---
    // The four phases the old hand-written loop ran — climate, tectonic forcing,
    // erosion (decomposed into its sub-passes), biotic — are now self-declaring
    // passes; the runner topo-sorts them and fires each at its cadence. The
    // biology↔erosion one-epoch lag is a declared loop-carried edge; climate's
    // `remarch_interval` is its cadence.
    // Per-cell inventory-weathering accumulators (journal/0094) — bedrock-only
    // ledgers keyed at the stable sentinel slot 0, accumulated by the in-loop
    // `dc:deep/weather_inventory` pass. Empty (and the pass absent) when the flag is
    // off ⇒ byte-identical.
    let weather_ledgers = if cfg.weather_inventory {
        vec![weather_inventory::empty_accumulator(); grid.w * grid.w]
    } else {
        Vec::new()
    };

    let grid_w = grid.w;
    let schedule = runner::DeepSchedule::new(runner::deep_passes_with(cfg, cadence))
        .expect("the deep-time pass graph is valid");
    let mut ctx = runner::DeepStepCtx {
        cfg,
        grid,
        erosion,
        biota,
        tec,
        blended,
        epoch: 0,
        sea_level: grid::sea_level_at(cfg, 0),
        dt: 1.0,
        uplift_total: 0.0,
        biotic_total: 0.0,
        thickening_total: 0.0,
        weather_ledgers,
        flux: if cfg.flow_record {
            flux::FluxAccum::new(grid_w)
        } else {
            flux::FluxAccum::inactive()
        },
    };
    schedule.run(&mut ctx);
    let runner::DeepStepCtx {
        mut grid,
        erosion,
        biota,
        tec,
        uplift_total,
        biotic_total,
        thickening_total,
        weather_ledgers,
        flux: flux_accum,
        ..
    } = ctx;
    // Close the final chapter and sort the archive into its cell-major index.
    let flux = flux_accum.finish();

    // Burial diagenesis: buried thick peat becomes coal (post-loop, unchanged).
    if let Some(b) = biota.as_ref() {
        b.finalize(&mut grid);
    }
    // **Re-relax the head field on the FINAL terrain.** It is a coarse-rate pass, so
    // the plane the loop leaves behind was relaxed at its last firing — twenty epochs
    // of uplift and incision before the surface the field actually ships. A consumer
    // reading `head` beside `surf` must not have to know the cadence to know they are
    // the same moment, so the field is planted once more here, exactly as coal
    // promotion runs post-loop. This cannot perturb anything already decided: the flux
    // record's vertical faces were accumulated per epoch against *contemporaneous*
    // ground and the archive is already closed above.
    if cfg.head_field {
        let n = grid.w * grid.w;
        let ground: Vec<f64> = (0..n).map(|i| grid.surf_at(i)).collect();
        let sea = grid::sea_level_at(cfg, cfg.iterations.saturating_sub(1));
        head::march(
            &mut grid,
            erosion.filled(),
            erosion.routed_surface(),
            &ground,
            erosion.area(),
            sea,
        );
    }
    // Re-key the accumulated saprolite ledgers onto the final record (bedrock facts
    // → `strata.units.len()`), so the collapse consumer reads them at the same slot
    // it always has. Off ⇒ empty ⇒ byte-identical. Done after `b.finalize` so the
    // record is final (coal promotion does not change the unit count, but keying is
    // taken against the record the field ships).
    let weather_ledgers = if cfg.weather_inventory {
        weather_inventory::finalize_ledgers(weather_ledgers, &grid.strata)
    } else {
        LedgerField::default()
    };
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
        weather_ledgers,
        flux,
    }
}

/// The precomputed tectonic-history schedule: the chapter table plus one analytic
/// thickening forcing plane per chapter geometry, blended per iteration across
/// the chapter ramp. Empty when tectonic history is off.
///
/// `pub` only so it can be a field of the runner's [`runner::DeepStepCtx`]; its
/// fields and methods stay module-private (the runner is a descendant module and
/// reaches them anyway).
pub struct TectonicSchedule {
    /// Plate state at the start of each chapter (`K + 1` entries).
    table: Vec<Vec<Plate>>,
    /// One forcing plane (m/iter thickening) per chapter geometry, row-major.
    planes: Vec<Vec<f64>>,
}

impl TectonicSchedule {
    /// The advected plate set at the start of a chapter — the geotherm field pass
    /// reads it to classify each cell's tectonic setting
    /// ([`tectonics::dominant_kind`]). `pub(crate)` so the sibling
    /// [`geotherm`](crate::deeptime::geotherm) module (and the runner) can reach
    /// the table without it becoming a public plate dump. Empty table (tectonic
    /// history off) yields an empty slice, and the geotherm pass is not scheduled
    /// there anyway.
    pub(crate) fn plates_at(&self, chapter: u8) -> &[Plate] {
        let c = (chapter as usize).min(self.table.len().saturating_sub(1));
        self.table.get(c).map_or(&[], Vec::as_slice)
    }

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
