//! The finite, coarse, pregenerated deep-time pipeline (S7).
//!
//! At world creation a bounded grid of coarse cells (14.7456 km each — the
//! design doc's 10–100 km band) runs, in order:
//!
//! 1. [`tectonics`] — plates, boundary classification, elevation provenance;
//! 2. [`climate`] — latitude bands, prevailing winds, orographic
//!    precipitation and rain shadow;
//! 3. [`hydrology`] — priority-flood depression filling, flow graph,
//!    discharge accumulation; every river reaches the sea *by construction*;
//! 4. [`history`] — a thin settlement/expansion/conflict sim over
//!    [`history::NUM_EPOCHS`] epochs, run against dc-sim's statistical tier
//!    (site pressures are collapsed via `engine::observe`) and committed as
//!    facts into the S2 constraint ledger.
//!
//! **Topology: continent-disc in a world-ocean** (the S7 recommendation, see
//! docs/spikes/S7-results.md for the rationale). The square cell grid is
//! centred on the world origin; cells beyond ~0.4·extent from the centre are
//! forced oceanic, so the civilized interior is a disc and the border wilds —
//! abyssal ocean, polar ice at extreme latitudes — extend beyond the grid in
//! every horizontal direction via [`Pregen::cell_view`]'s synthesized cells.
//!
//! Everything is a pure function of the world seed: no wall clock, no ambient
//! entropy, no iteration-order dependence.

pub mod climate;
pub mod history;
pub mod hydrology;
pub mod tectonics;

use dc_sim::statistical::rng::Draws;
use dc_sim::statistical::{Ledger, ToyWorld};

use crate::draws::Wilds;
use serde::{Deserialize, Serialize};

pub use history::{SiteSummary, YEAR_ZERO_TICK};

/// Chunks per coarse-cell edge: 512 · 28.8 m = 14.7456 km.
pub const CELL_CHUNKS: i64 = 512;
/// Voxels per coarse-cell edge (2^14).
pub const CELL_VOXELS: i64 = CELL_CHUNKS * 32;


/// The player-facing world-size knob: coarse cells per grid edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Extent {
    /// 5×5 cells — ~74 km across.
    Small,
    /// 17×17 cells — ~251 km across (the design doc's default class).
    Medium,
    /// 69×69 cells — ~1017 km across.
    Large,
}

impl Extent {
    pub const fn cells(self) -> i32 {
        match self {
            Extent::Small => 5,
            Extent::Medium => 17,
            Extent::Large => 69,
        }
    }

    pub fn extent_km(self) -> f64 {
        f64::from(self.cells()) * CELL_VOXELS as f64 * 0.9 / 1000.0
    }

    pub const fn label(self) -> &'static str {
        match self {
            Extent::Small => "small (~74 km)",
            Extent::Medium => "medium (~251 km)",
            Extent::Large => "large (~1017 km)",
        }
    }

    /// Parse the player-facing extent name from a launch argument:
    /// `small` | `medium` | `large`, case-insensitive and whitespace-trimmed.
    /// `None` for anything else (the caller turns that into a usage error).
    pub fn from_arg(s: &str) -> Option<Extent> {
        match s.trim().to_ascii_lowercase().as_str() {
            "small" => Some(Extent::Small),
            "medium" => Some(Extent::Medium),
            "large" => Some(Extent::Large),
            _ => None,
        }
    }
}

/// World-creation parameters. Everything downstream derives from these.
#[derive(Debug, Clone, Copy)]
pub struct WorldParams {
    pub seed: u64,
    pub extent: Extent,
}

/// Where a cell's elevation came from — the tectonic provenance record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Continental interior.
    Craton,
    /// Continent–continent convergence: mountain belt.
    Orogeny,
    /// Convergence with subduction: volcanic arc.
    Arc,
    /// Divergence on land.
    Rift,
    /// Subduction trench.
    Trench,
    /// Divergent oceanic ridge.
    Ridge,
    /// Continental shelf (ocean floor pinned shallow next to land).
    Shelf,
    /// Plain oceanic crust.
    OceanFloor,
    /// Transform boundary.
    Transform,
}

/// Per-provenance terrain roughness (metres) — the amplitude the lazy
/// elevation pyramid feeds its refinement jitter from.
pub fn provenance_roughness(p: Provenance) -> f64 {
    match p {
        Provenance::Craton => 90.0,
        Provenance::Orogeny => 420.0,
        Provenance::Arc => 260.0,
        Provenance::Rift => 180.0,
        Provenance::Trench => 80.0,
        Provenance::Ridge => 60.0,
        Provenance::Shelf => 40.0,
        Provenance::OceanFloor => 25.0,
        Provenance::Transform => 140.0,
    }
}

/// One pregenerated coarse cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub plate: u16,
    pub elev_m: f64,
    pub provenance: Provenance,
    pub lat_deg: f64,
    pub temp_c: f64,
    /// Annual precipitation, normalized 0..1.
    pub precip: f64,
    /// Depression-filled elevation (hydrology); >= `elev_m`.
    pub filled_m: f64,
    /// Downstream cell index (steepest descent on filled elevation).
    /// `None` for ocean cells — they *are* the sea.
    pub flow_to: Option<u32>,
    /// Accumulated drainage (sum of upstream precip).
    pub discharge: f64,
    /// The outflow edge of this cell carries a river.
    pub river: bool,
    /// Depression filling raised this cell: standing water.
    pub lake: bool,
}

impl Cell {
    pub fn is_land(&self) -> bool {
        self.elev_m > 0.0
    }
}

/// The bounded coarse grid, `w`×`w` cells, centred on the world origin.
/// Cell `(gx, gy)` covers world voxels
/// `[(gx - w/2)·CELL_VOXELS, (gx - w/2 + 1)·CELL_VOXELS)` on each axis
/// (`gy` maps to the world z axis; +z is north).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellGrid {
    pub w: i32,
    pub cells: Vec<Cell>,
}

impl CellGrid {
    pub fn idx(&self, gx: i32, gy: i32) -> Option<usize> {
        if gx >= 0 && gx < self.w && gy >= 0 && gy < self.w {
            Some((gy * self.w + gx) as usize)
        } else {
            None
        }
    }

    pub fn get(&self, gx: i32, gy: i32) -> Option<&Cell> {
        self.idx(gx, gy).map(|i| &self.cells[i])
    }

    pub fn coords(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.w, idx as i32 / self.w)
    }

    /// World-voxel coordinate of a cell's centre (grid coords may lie outside
    /// the grid; the mapping is uniform).
    pub fn cell_center_voxel(&self, gx: i32, gy: i32) -> (i64, i64) {
        let half = i64::from(self.w / 2);
        (
            (i64::from(gx) - half) * CELL_VOXELS + CELL_VOXELS / 2,
            (i64::from(gy) - half) * CELL_VOXELS + CELL_VOXELS / 2,
        )
    }

    /// Grid coordinates of the cell containing a world voxel (may lie outside
    /// the grid for wilds positions).
    pub fn cell_of_voxel(&self, vx: i64, vz: i64) -> (i64, i64) {
        let half = i64::from(self.w / 2);
        (
            vx.div_euclid(CELL_VOXELS) + half,
            vz.div_euclid(CELL_VOXELS) + half,
        )
    }
}

/// Latitude bands are compressed onto the grid: the south edge sits at 8°,
/// the north edge at 78°, linear in z and extended (then clamped) beyond the
/// grid — walking north out of the civilized world runs into polar ice.
pub const LAT_SOUTH: f64 = 8.0;
pub const LAT_NORTH: f64 = 78.0;
/// Latitude at or above which wilds synthesis produces ice waste.
pub const LAT_ICE: f64 = 84.0;

pub fn latitude_deg(w: i32, gy_center: f64) -> f64 {
    (LAT_SOUTH + (LAT_NORTH - LAT_SOUTH) * gy_center / f64::from(w)).clamp(0.0, 89.0)
}

/// Sea-level air temperature (°C) at a latitude.
pub fn temp_sea_level(lat_deg: f64) -> f64 {
    31.0 - 0.52 * lat_deg
}

/// What the lazy pyramid needs to know about any cell coordinate — a real
/// pregenerated cell inside the grid, or a synthesized border-wilds cell
/// outside it. Wilds cells have no history and hostile parameters; they are a
/// pure function of `(seed, coords)` and extend forever.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellView {
    pub elev_m: f64,
    pub rough_m: f64,
    pub lat_deg: f64,
    pub precip: f64,
    /// True when synthesized outside the pregen grid (border wilds).
    pub wilds: bool,
}

/// Everything world creation produced: the coarse grid, the committed-fact
/// ledger (settlement history), the statistical-tier overlay world the live
/// sim keeps querying after year zero, and site summaries for the lazy layer.
pub struct Pregen {
    pub seed: u64,
    pub extent: Extent,
    pub grid: CellGrid,
    pub ledger: Ledger,
    pub overlay: ToyWorld,
    pub sites: Vec<SiteSummary>,
    pub n_polities: u32,
    /// Number of `engine::observe` collapses the history pass performed.
    pub observe_count: u32,
    /// The validated pass graph this world was built with; the lazy layer
    /// runs its collapse-phase (strata) passes per column.
    pub pipeline: crate::pipeline::Pipeline,
    /// The always-on deep-time field (3e-1): the eroded surface driving the
    /// collapse macro-terrain and the per-cell strata record driving
    /// depositional formation context. Built by the `dc:pass/deep-time` pass.
    pub deep: crate::deeptime::DeepField,
}

impl Pregen {
    /// Run the full coarse pipeline. Deterministic in `params`.
    ///
    /// The pregen stages are no longer a hand-ordered list: the vanilla
    /// pass graph ([`crate::pipeline::Pipeline::vanilla`]) topo-sorts them
    /// from their declared reads/writes; the declarations force exactly the
    /// legacy tectonics → climate → hydrology → history order, so this is
    /// output-preserving (S7 byte-identity tests prove it).
    pub fn run(params: WorldParams) -> Self {
        Self::run_with(params, &crate::deeptime::DeepOverrides::default())
    }

    /// Run the full coarse pipeline with gen-time [`DeepOverrides`] applied to
    /// the deep-time pass (the sealed-path door: `full_agents` / `tectonic_history`
    /// / amplitude flipped on at world creation). Deterministic in
    /// `(params, overrides)`. An empty `DeepOverrides` is byte-identical to
    /// [`Pregen::run`] — no existing world's output changes.
    pub fn run_with(params: WorldParams, overrides: &crate::deeptime::DeepOverrides) -> Self {
        let pipeline = crate::pipeline::Pipeline::vanilla().expect("vanilla pass graph is valid");
        let mut ctx = crate::pipeline::PregenCtx {
            seed: params.seed,
            w: params.extent.cells(),
            grid: None,
            history: None,
            deep: None,
            deep_overrides: *overrides,
        };
        pipeline.run_pregen(&mut ctx);
        let grid = ctx.grid.expect("tectonics pass creates the grid");
        let history = ctx.history.expect("history pass runs");
        let deep = ctx.deep.expect("deep-time pass runs");
        Self {
            seed: params.seed,
            extent: params.extent,
            grid,
            ledger: history.ledger,
            overlay: history.overlay,
            sites: history.sites,
            n_polities: history.n_polities,
            observe_count: history.observe_count,
            pipeline,
            deep,
        }
    }

    /// The sim tick at which pregenerated history ends and live play begins.
    pub fn year_zero(&self) -> u32 {
        YEAR_ZERO_TICK
    }

    /// The cell view for any cell coordinate — pregen inside the grid, wilds
    /// synthesis outside it. This is the only door the lazy pyramid has into
    /// the coarse world, so the wilds path exercises the same machinery.
    pub fn cell_view(&self, gx: i64, gy: i64) -> CellView {
        if gx >= 0
            && gx < i64::from(self.grid.w)
            && gy >= 0
            && gy < i64::from(self.grid.w)
            && let Some(c) = self.grid.get(gx as i32, gy as i32)
        {
            return CellView {
                elev_m: c.elev_m,
                rough_m: provenance_roughness(c.provenance),
                lat_deg: c.lat_deg,
                precip: c.precip,
                wilds: false,
            };
        }
        // Border wilds: unbounded, hostile, historyless.
        let w = self.grid.w;
        let lat = latitude_deg(w, gy as f64 + 0.5);
        let noise = Draws::of::<Wilds>(self.seed).unit(&[gx as u64, gy as u64]);
        if lat >= LAT_ICE {
            // Polar ice waste: land ice slightly above sea level.
            CellView {
                elev_m: 25.0 + 35.0 * noise,
                rough_m: 60.0,
                lat_deg: lat,
                precip: 0.08,
                wilds: true,
            }
        } else {
            // Abyssal world-ocean, deepening away from the grid.
            let beyond = (-gx)
                .max(gx - i64::from(w) + 1)
                .max(-gy)
                .max(gy - i64::from(w) + 1)
                .max(0) as f64;
            CellView {
                elev_m: -160.0 - 55.0 * beyond.min(4.0) + 25.0 * (noise * 2.0 - 1.0),
                rough_m: 25.0,
                lat_deg: lat,
                precip: 0.05,
                wilds: true,
            }
        }
    }

    /// Rough resident footprint of the pregen output (struct payloads only;
    /// the honest "what the pause bought you" number for the size knob).
    pub fn approx_resident_bytes(&self) -> usize {
        self.grid.cells.len() * std::mem::size_of::<Cell>()
            + self.ledger.len() * std::mem::size_of::<dc_sim::statistical::Fact>()
            + self.sites.len() * std::mem::size_of::<SiteSummary>()
            + self.deep.resident_bytes()
    }
}
