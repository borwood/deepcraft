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
//! dropped after the run; `surf` + `regolith` + `strata` (plus the tectonic
//! exports) survive into the world state. `regolith` — the `H` plane — was
//! summed into `surf` and discarded until journal/0053; carrying it is what lets
//! the collapse tier read soil depth from the recorded cause instead of
//! re-inventing it from present-day precipitation.
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
use super::inventory::{FactLedger, FracM, LedgerField, LedgerView, build_working};
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
    /// roster. `None` = production default (**on** since the roster flip
    /// 2026-07-21, journal/0047); pass `Some(false)` to reach the pre-0034 path.
    pub full_agents: Option<bool>,
    /// Override [`DeepConfig::thickening_scale`]: the orogenic amplitude the
    /// analytic tectonic forcing multiplies (m/iter for a unit-rate boundary).
    /// Only bites when tectonic history is on. `None` = production default.
    pub thickening_scale: Option<f64>,
    /// **The erosion budget multiplier** (`erodibility_probe` experiment B):
    /// scales the three global erosion rates — bedrock→regolith `weathering`,
    /// stream-power `k_transport`, and bedrock incision `k_bedrock` — *together*
    /// by this factor, so the **relative** rates (and therefore the differential-
    /// erosion signal the erodibility coupling expresses) never change; only the
    /// total amount of material erosion is allowed to move. This is the TERRAIN
    /// (erosion) amplitude, distinct from `thickening_scale` above, which is the
    /// TECTONIC (orogenic) amplitude — the term collision the corpus already had
    /// to disambiguate (journal/0040, ROADMAP § the erodibility rider).
    ///
    /// **STUB #24 — THIS KNOB CANNOT REACH THE PROCESS THAT DOES THE ERODING.**
    /// It scales the three rates above and **not `diffusion`**, and hillslope creep
    /// carries **96 %** of this world's denudation (journal/0111, corrections #56).
    /// Measured: **100× moves catchment-averaged denudation by 1.4×.** Supply and
    /// transport are coupled through the cover taper `exp(−H/H*)`, so neither pays
    /// alone (100× budget → 1.4×; 10× creep → 1.7×) and together they pay 132×.
    /// Do not "fix" this by adding `diffusion` to the list — the heir is the owed
    /// `earth-processes.md` § 3e calibration of iteration↔Myr against a real orogen,
    /// and the acceptance instrument is `examples/denudation_probe.rs`. The
    /// resulting numbers are an **appearance-class, user-owned** call.
    ///
    /// `None` = production default (the shipped calibration, multiplier `1×`).
    /// `Some(1.0)` is **byte-identical** to `None` (`x * 1.0 == x` exactly), so
    /// the flag's off-state is provably inert (asserted in the plumbing tests).
    /// A dev launch flag (`--erosion-budget <mult>`) sets it; the walkable
    /// cranked world it enables is the standing "conservative amplitude" call.
    pub erosion_budget: Option<f64>,
    /// Override [`DeepConfig::weather_inventory`]: the in-loop, per-epoch
    /// **accumulating** inventory-weathering pass (journal/0094) that grows a basal
    /// saprolite band on each subaerial cell's working inventory across the deep-time
    /// run. `None` = production default (**off**, the S-5 identity floor); pass
    /// `Some(true)` to turn the pass on for a flag-on walk. A dev launch flag
    /// (`--weather-inventory`) sets it.
    pub weather_inventory: Option<bool>,
}

impl DeepOverrides {
    /// True when no override is set — the all-inherit case whose config is
    /// byte-identical to [`production_config`].
    pub fn is_empty(&self) -> bool {
        self.tectonic_history.is_none()
            && self.full_agents.is_none()
            && self.thickening_scale.is_none()
            && self.erosion_budget.is_none()
            && self.weather_inventory.is_none()
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
        // **Full erosion-agent roster ON** (DECIDED 2026-07-21, user; journal/0034
        // § Knobs, journal/0047). The user ratified turning the roster on as the
        // next flip ("we turn on agents next"). So production now runs the wind +
        // frost + wave agents: eolian deflation/deposition (a fifth
        // `lithology::Agent`) redistributes loose cover into dune fields and
        // downwind loess; the temperature-gated frost multiplier strips extra
        // regolith in the periglacial band about 0 °C; littoral wave attack cuts
        // coasts toward the current sea stand. Same event class as the
        // biotic/erodibility/tectonic flips above: this CHANGES TERRAIN SHAPE —
        // and the strata record's eolian facies — for every world created from
        // here on; worlds made before this flip are not reproducible under it.
        //
        // The seven agent MAGNITUDES ride at their `DeepConfig` defaults
        // (`eolian_deflation` 0.02, `eolian_arid_precip` 0.32, `eolian_deposit_frac`
        // 0.25, `frost_weathering_gain` 1.5, `frost_band_width_c` 12.0,
        // `wave_erosion` 0.05, `wave_band_m` 30.0) and are EXPLICITLY UNRATIFIED:
        // they are appearance-class numbers the user will judge live, station by
        // station, in a guided walk of this world (journal/0047's tour map). This
        // flip ratifies turning the roster ON; the LIVE MAGNITUDES TOUR — not this
        // line — ratifies the numbers. Do not tune them here.
        full_agents: true,
        ..DeepConfig::default()
    }
}

/// The production config with gen-time [`DeepOverrides`] applied on top: start
/// from [`production_config`], then overwrite each flag the caller set. Every
/// `None` override inherits, so `production_config_with(cells, seed,
/// &DeepOverrides::default())` is **byte-identical** to `production_config(cells,
/// seed)` (asserted in the tests). This is the single seam a launch flag reaches
/// the deep-time run through.
///
/// **Where the provider set is resolved.** [`DeepConfig::providers`] is fixed
/// here, at world build, by [`production_config`]'s `..DeepConfig::default()` —
/// today, unconditionally to the identity set. When a content pack can *supply*
/// a provider, this function is the one place that resolution happens, so the
/// resolved set stays a pure function of the world's frozen content set
/// (ARCHITECTURE.md § *The content set is frozen at world creation*). No
/// override field exists yet, deliberately: nothing can select one, and a
/// selection channel with no selectors is the exact stand-in-becomes-definition
/// shape the seam is meant to avoid.
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
    // Erosion budget: scale the three global erosion rates *together*
    // (`erodibility_probe` experiment B), so the relative rates the erodibility
    // coupling reads never move — only the total amount of erosion does. A
    // multiplier of `1.0` leaves each rate bit-for-bit unchanged (`x * 1.0 == x`
    // for f64), which is why `Some(1.0)` is byte-identical to `None` (the
    // falsifier in the plumbing tests). The multiply is unconditional on
    // `erodibility`: these are the base rates the run uses either way, and the
    // coupling — when on — modulates around them without changing this scaling.
    if let Some(mult) = overrides.erosion_budget {
        cfg.weathering *= mult;
        cfg.k_transport *= mult;
        cfg.k_bedrock *= mult;
    }
    if let Some(v) = overrides.weather_inventory {
        cfg.weather_inventory = v;
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
    /// **Final regolith thickness `H` per cell, metres** (row-major, `w × w`) —
    /// the loose, mobile cover the deep sim weathered off bedrock, transported
    /// and deposited across the whole run. Bedrock elevation is `surf - regolith`.
    ///
    /// Carried since 2026-07-21 (journal/0053). Before that the distillation kept
    /// only the sum `surf = r + h` and threw `h` away, so the collapse tier
    /// re-invented soil depth from *present-day precipitation* (stubs.md § 3) —
    /// which is why a deflation basin the sim had scoured to `H ≈ 0` still wore
    /// three voxels of topsoil (journal/0049 station 1). This plane is the
    /// recorded cause; `collapse.rs::column` and `geology.rs::clastic_pass` take
    /// their soil / veneer depth from it.
    ///
    /// Cost: one `f64` per deep cell — `w²·8` bytes, the same order as `surf`,
    /// which the ritual already pays.
    pub regolith: Vec<f64>,
    /// Per-cell strata record, bottom-up units tagged at deposition.
    pub strata: Vec<DeepStrata>,
    /// **The grid-wide transformation-fact ledger** (the first-real-behavior
    /// weathering slice, material-behavior.md §1) — the S17 keystone made a
    /// **production** artifact. Indexed by the same cell as [`Self::strata`]; each
    /// cell's LAST slot is the bedrock seam's `Structure→Loose` weathering facts,
    /// cause-carrying (frost/biotic/chemical). **Empty unless
    /// [`DeepConfig::weather_inventory`](super::grid::DeepConfig::weather_inventory)
    /// is on** — off, the field is byte-identical and this record is empty (the S-5
    /// identity default). The collapse folds `base + facts`
    /// ([`LedgerView::weathering_product_m`]) into a basal weathering-front band.
    ///
    /// **ONE record for the whole grid, with the cell as a CSR row** (journal/0102,
    /// the [`flux`](super::flux) shape ported). This was `Vec<FactLedger>` — a
    /// per-cell *owning container*, which is a header × 297,025 cells (13.60 MiB)
    /// before it stores anything, in a field where 75.8 % of cells never weather.
    /// Read a cell with [`LedgerField::get`] / [`Self::ledger_at_voxel`], both of
    /// which hand back a borrowed [`LedgerView`].
    ///
    /// Sidecar rather than a `facts` field grown onto `DepUnit` (which is `Copy` and
    /// read across the merged collapse/erosion/biotic files); the eventual home is a
    /// `RecordedUnit { base, facts }` on `DeepStrata` (inventory.rs).
    pub ledgers: LedgerField,
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
    /// **The face-flux record** (FLOW slice 1, `docs/design/flow.md` § 2 —
    /// RATIFIED 2026-07-25): per-tectonic-chapter **flux on 3D faces**, the
    /// representation that supersedes [`Self::recv`] above.
    ///
    /// `recv` is one out-edge per cell — a spanning tree, which can represent
    /// convergence and **structurally cannot represent divergence at all** (no
    /// distributaries, braids, fans or deltas), exported as *the last routing*
    /// after two hundred epochs of the process were discarded. This keeps **every
    /// chapter**, and keeps it on **shared faces**, so a refinement built on it
    /// agrees from both sides of a cell boundary by construction.
    ///
    /// Empty when [`DeepConfig::flow_record`](super::grid::DeepConfig::flow_record)
    /// is off. It is a pure sidecar — the surface planes and the strata record are
    /// byte-identical with the flag either way — and it is the **largest thing the
    /// ritual keeps**; see [`Self::resident_bytes`] and flow.md § 9.1.
    ///
    /// **Nothing at runtime expresses it yet, deliberately** (the slice is the
    /// recording half only): the world stays honestly river-less rather than
    /// gaining a second fake. `docs/spines.md` § 3 carries the row.
    pub flux: super::flux::FluxRecord,
    /// **Exhumation** (m) and **crustal thickness** (m) per cell. Exported and,
    /// as of U8, populated in every production world. The **exported** planes are
    /// still read by no *collapse-tier* consumer (`docs/spines.md` § 3,
    /// built-but-unconsumed; A-2 — prose cannot fail a build, so the status is
    /// stated here, not implied). These are the *intended* metamorphic-grade axes
    /// (§ 6.4): the day a cut face should show an aureole rather than plain
    /// basement, a metamorphism pass reads the P/T path off these planes into
    /// grade classes (slate/schist/gneiss). The expression slice is **Sequenced**
    /// ("tectonic expression at the collapse tier"; `stubs.md` § 4, "the absent
    /// metamorphic expresser").
    ///
    /// *(The in-sim `t_crust` **value** is now read by two passes: `isostasy()`
    /// (`erosion.rs`) and — since journal/0093 — the **geotherm field pass**
    /// ([`super::geotherm`]), which reads crustal heat flow off it to plant the
    /// `temperature` field. That is why this note is careful to say the
    /// *unconsumed* axis is the **exported** plane, not the value: metamorphism
    /// (the P side of the ladder) still awaits its expresser, but the T side — the
    /// geotherm feeding coal rank — has landed.)* Empty when tectonic history is
    /// off.
    pub exhum: Vec<f64>,
    pub t_crust: Vec<f64>,
    /// **The `temperature` condition-field** (`dc:field/temperature`, §14) — the
    /// per-cell **geothermal gradient** (°C/m) the geotherm field pass planted
    /// ([`super::geotherm`]), so `T(depth) = surface_T + gradient·depth`. Its
    /// first consumer is coal rank (inside the run, at finalize); it is exported
    /// here so `temperature` is a real read field for the measurement probes and
    /// the future metamorphism heir (`exhum` = P, this = T → grade). Empty when
    /// tectonic history is off. **Not part of the surface fingerprint** — a new
    /// field, covered by its own tests, so the pre-existing planes' goldens are
    /// unaffected.
    pub geotherm: Vec<f64>,
    /// **The `head` condition-field** (`dc:field/head`, §14; flow.md § 2.4) — the
    /// per-cell **hydraulic potential** (metres) the head field pass planted
    /// ([`super::head`]). *Elevation + pressure head*, **not** an elevation: where a
    /// confining bed caps a permeable one it stands **above** the local ground, and
    /// that is artesian — precisely the state the unconfined `H = y + sat` proxy
    /// (`water.md`) cannot express.
    ///
    /// Its first consumer is **inside the run**: the flow record's vertical
    /// (slot↔slot) faces, which FLOW slice 1 left structurally present and honestly
    /// zero. It is exported here so `head` is a real read field for the measurement
    /// probes and for the heirs that need the potential rather than its
    /// consequences — refinement as a boundary-value problem (continuation (b)) and
    /// the free/bound edge with void intervals (continuation (c)). Empty when
    /// [`DeepConfig::head_field`](super::grid::DeepConfig::head_field) is off.
    ///
    /// **Not part of the surface fingerprint** — a new field, covered by its own
    /// tests, so the pre-existing planes' goldens are unaffected.
    pub head: Vec<f64>,
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
    let (recv, area, lake, exhum, t_crust, geotherm, chapters) = if cfg.tectonic_history {
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
            run.grid.geotherm.clone(),
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
            Vec::new(),
        )
    };
    // **Inventory weathering — now a per-epoch PROCESS** (journal/0094, Movement 3;
    // material-behavior.md §4/§11). The `dc:deep/weather_inventory` runner pass ran
    // *inside* the deep-time loop, every epoch, accumulating the saprolite band on
    // each epoch's live terrain; the run hands back the finished, record-keyed
    // ledgers. This REPLACES S18's post-hoc one-shot over the frozen end-state (stub
    // #17). It writes only this sidecar — the `R`/`H` height weathering in
    // `erosion.rs` is untouched (§11 two-authorities split). Off ⇒ empty Vec ⇒
    // byte-identical (S-5 identity default).
    let ledgers = run.weather_ledgers;
    // The regolith plane, carried (journal/0053) rather than summed away.
    let regolith = run.grid.h;
    // **Reclaim the growth slack** (S19, 2026-07-25). Each cell's `units` Vec grows by
    // `push` across the epoch loop, so it carries allocator doubling slack: measured
    // **live 84.47 MiB vs capacity 138.49 MiB on a production world — 54.02 MiB of pure
    // waste, 39 % of the record heap and 33 % of ALL `DeepField` residency.** The record
    // is append-only during the compile and **read-only forever after**, so the capacity
    // is dead the moment the loop ends. Runtime residency is first-class (CLAUDE.md);
    // gen time is free, so the one-time copy is the right trade. Asserted by
    // `strata_is_shrunk_to_fit_after_the_compile`.
    let mut strata = run.grid.strata;
    for s in &mut strata {
        s.units.shrink_to_fit();
    }
    // The `head` condition-field, carried out of the run (its exchange companion is
    // gen-time scratch and is dropped with the grid — it is a cache of the field,
    // never an authority beside it). Empty when the head pass was absent.
    let head = run.grid.head;
    DeepField {
        w,
        wp: cells.w as usize,
        cell_m,
        surf,
        regolith,
        strata,
        ledgers,
        recv,
        area,
        lake,
        flux: run.flux,
        exhum,
        t_crust,
        geotherm,
        head,
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

    /// **Regolith thickness `H` (metres) of the deep cell nearest the world
    /// voxel**, or `None` in the wilds — the recorded loose-cover depth the
    /// collapse tier reads instead of guessing soil from present-day
    /// precipitation.
    ///
    /// **Nearest, not bilinear — deliberately**, and this is the load-bearing
    /// choice. journal/0053 measured that `H` is *exactly* the sum of the
    /// cell's own [`record_at_voxel`](Self::record_at_voxel) unit thicknesses:
    /// the recorder logs every metre of loose cover the sim deposits, so the
    /// record IS the regolith column, decomposed. The collapse tier uses the
    /// difference between the two (total column minus what whole voxels can
    /// express) as its surficial veneer, and that subtraction only conserves
    /// mass if both terms name the **same cell**. Interpolating one and not the
    /// other would leak or invent loose material at every cell boundary. So this
    /// steps at the ~460 m deep-cell grid exactly as the record does.
    ///
    /// Registration is the shared [`Self::deep_coords`] convention (integer
    /// `wp/2` centring — do not reintroduce the half-cell shift of journal/0043).
    pub fn regolith_at_voxel(&self, vx: i64, vz: i64) -> Option<f64> {
        if self.regolith.is_empty() {
            return None;
        }
        let (gx, gy) = self.deep_coords(vx, vz)?;
        let ix = (gx.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        let iy = (gy.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        self.regolith.get(iy * self.w + ix).copied()
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

    /// The **weathering fact ledger** of the deep cell **nearest** the world voxel,
    /// or `None` when inventory weathering is off (`ledgers` empty) or in the wilds.
    /// Nearest, exactly like [`Self::record_at_voxel`] — the ledger is index-parallel
    /// to `strata`, so it steps at the same ~460 m deep-cell grid the record does.
    /// The collapse reads this to fold `base + facts` (the weathering-front band).
    ///
    /// Returns a **borrowed [`LedgerView`]**, not `&FactLedger`: since journal/0102
    /// there is no per-cell struct to hand out a reference to (that struct was the
    /// 13.60 MiB of headers). The read surface is unchanged.
    pub fn ledger_at_voxel(&self, vx: i64, vz: i64) -> Option<LedgerView<'_>> {
        if self.ledgers.is_empty() {
            return None;
        }
        let (gx, gy) = self.deep_coords(vx, vz)?;
        let ix = (gx.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        let iy = (gy.round() as i64).clamp(0, self.w as i64 - 1) as usize;
        self.ledgers.get(iy * self.w + ix)
    }

    // ---- R/H unification: the derived views (Movement 2a) -----------------
    //
    // material-behavior.md §13.6, ratified. The per-cell **working inventory is
    // the authority** for surface material; the scalar `R`/`H` planes are its
    // **materialized views**. The persistent, reconciled per-cell surface-`Loose`
    // inventory IS the strata record (ungated, `record:true`): the deposition
    // pass reconciled each epoch's net ΔH into it as deposit (`void→Loose`) /
    // erode (`Loose→void`) facts — the inventory's own edge primitives, keyed by
    // `DepTag` → material through the *current* `deep_class`/`litho_of_tag` rule —
    // at the epoch (pass/chapter) boundary. journal/0053's finalize invariant
    // (`Σ unit.thickness == H`) makes the record the authority `H` derives from.
    //
    // These methods MATERIALIZE `H`/`R` from that inventory (scratch-first
    // reconcile: hot loop on planes, inventory rebuilt at the boundary). The
    // stored `surf`/`regolith` planes remain the exact byte-identical cache the
    // collapse reads; the byte-identity agreement tests assert these derived
    // views reproduce them, proving the inventory is the authority structurally.

    /// **Derive the surface regolith `H` (metres) at cell `i` from the working
    /// inventory** — the positional §13.6 rule: build the cell's inventory from
    /// its record (`Loose` cover) + the bedrock `Structure` seam, then take the
    /// `Loose` **above the topmost `Structure`** (cave fill excluded). Over the
    /// production record this equals the [`Self::regolith`] plane within the
    /// recorder residual (the agreement test). `0.0` where there is no record.
    pub fn derive_regolith_at(&self, i: usize) -> FracM {
        let Some(strata) = self.strata.get(i) else {
            return 0.0;
        };
        let ledger = FactLedger::empty_with_bedrock(strata);
        build_working(strata, &ledger).derived_regolith_m()
    }

    /// **Derive the bedrock-top elevation `R` (metres) at cell `i`.** In the
    /// two-plane engine `R` is a signed elevation **datum**, not a structural
    /// stock (Movement 2a finding), so it derives as `surf − H` — the elevation of
    /// the topmost `Structure` contact — which recovers the scalar `R` plane
    /// (`surf − regolith`) within the recorder residual. `Σ Structure`
    /// ([`super::WorkingInventory::derived_structure_stock_m`]) is the *stock*
    /// beneath that datum, a distinct quantity.
    pub fn derive_bedrock_at(&self, i: usize) -> f64 {
        self.surf.get(i).copied().unwrap_or(0.0) - self.derive_regolith_at(i)
    }

    /// Rough resident footprint (bytes) — the honest "what the ritual keeps in
    /// memory" number.
    pub fn resident_bytes(&self) -> usize {
        (self.surf.len()
            + self.regolith.len()
            + self.area.len()
            + self.exhum.len()
            + self.t_crust.len()
            + self.geotherm.len()
            // FLOW continuation (a): the `head` condition-field.
            + self.head.len())
            * std::mem::size_of::<f64>()
            + self.recv.len() * std::mem::size_of::<i32>()
            + self.lake.len()
            + self.strata.len() * std::mem::size_of::<DeepStrata>()
            + self
                .strata
                .iter()
                .map(DeepStrata::heap_bytes)
                .sum::<usize>()
            // The grid-wide fact ledger (journal/0102). **No per-cell struct term** —
            // there is no per-cell struct any more; that was the 13.60 MiB.
            + self.ledgers.footprint_bytes()
            // FLOW slice 1: the face-flux record. Reported, never truncated —
            // gen time is free, residency is not (flow.md § 9.1).
            + self.flux.resident_bytes()
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

#[cfg(test)]
mod shrink_tests {
    use super::*;
    use crate::pregen::{Extent, Pregen, WorldParams};

    /// **S19 free win (2026-07-25).** Every cell's `units` Vec grows by `push` across
    /// the epoch loop and so carries allocator doubling slack; the record is read-only
    /// after the compile, so that capacity is dead weight. Measured on a production
    /// world: 54.02 MiB, **33 % of all `DeepField` residency**. `build_field` shrinks
    /// each cell to fit — this asserts it actually happened (capacity == len for every
    /// cell), which is what makes the reclaim real rather than intended.
    #[test]
    fn strata_is_shrunk_to_fit_after_the_compile() {
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let field = DeepField::from_pregen(&pregen);
        let slack: usize = field
            .strata
            .iter()
            .map(|s| s.units.capacity() - s.units.len())
            .sum();
        let nonempty = field.strata.iter().filter(|s| !s.units.is_empty()).count();
        assert!(
            nonempty > 0,
            "the world must actually have a record for this test to mean anything"
        );
        assert_eq!(
            slack, 0,
            "every cell's units Vec must be shrunk to fit after the compile \
             ({nonempty} non-empty cells carried {slack} slack entries)"
        );
    }
}
