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

/// **The erosional amplitude, and the four rates it is an amplitude *of***
/// (journal/0114).
///
/// `weathering` makes regolith, `diffusion` creeps it downhill, `k_transport`
/// carries it in water and `k_bedrock` cuts rock. Those four are **one clock**, and
/// the reason they have to move together is measured, not asserted:
/// journal/0111 swept them apart and found that **neither pays alone** — 100× the
/// supply side bought 1.4×, 10× the transport side bought 1.7×, and together they
/// bought 132×, which is 59× more than their separate gains multiplied.
///
/// The mechanism is the cover taper `exp(−H/H*)`. Raise supply alone and the
/// regolith made **shields the rock that made it**; raise transport alone and there
/// is nothing to carry. So they move together.
///
/// **What a uniform scaling does NOT buy, measured rather than hoped.** The
/// hypothesis when this function was written was that scaling production and removal
/// together would leave the steady-state cover `H` where it was — and therefore
/// leave the taper, the only term carrying an absolute length, unengaged. It does
/// not: the ladder in `examples/denudation_probe.rs` shows mean regolith going
/// 4.6 m → 8.8 → 43.9 → 118.8 → 359 → 782 as the multiplier climbs. **The two levers
/// are not symmetric, because transport has a ceiling supply does not.** Hillslope
/// creep's flux limiter binds on ~89 % of the cells that HAVE regolith to move *at the
/// shipped rates already*, so the pass is a one-cell-per-epoch conveyor and raising `diffusion`
/// cannot make it faster (100× on transport alone buys 1.6×). That ceiling is what
/// [`EROSION_CALIBRATION`] is calibrated *under*, and `stubs.md` § 27 carries it.
///
/// **This is the one place that scaling is written.** Two consumers call it — the
/// shipped calibration ([`EROSION_CALIBRATION`], applied by [`production_config`])
/// and the [`DeepOverrides::erosion_budget`] dev lever — so the knob and the
/// default cannot drift into scaling different sets, which is the defect
/// stubs.md § 24 recorded.
///
/// Not scaled, deliberately: `wave_erosion`, `eolian_deflation` and
/// `frost_weathering_gain`. Those are the **agent magnitudes**, explicitly
/// unratified appearance numbers the user judges live (`production_config` §
/// full_agents), and each is a rate at a *place* (a shore, a dune field, a
/// periglacial band) rather than a term in the land-wide budget. Folding them in
/// here would smuggle four appearance calls into one calibration.
pub fn scale_erosion_rates(cfg: &mut DeepConfig, mult: f64) {
    cfg.weathering *= mult;
    cfg.diffusion *= mult;
    cfg.k_transport *= mult;
    cfg.k_bedrock *= mult;
}

/// **The calibrated erosional amplitude** (journal/0114) — the multiplier
/// [`scale_erosion_rates`] applies to the shipped world.
///
/// journal/0111 measured this world's catchment-averaged denudation at
/// **0.0110 m/Myr** against the ratified 500 Myr Phanerozoic register: 9× below the
/// slowest landscape ever measured on Earth (McMurdo Dry Valleys bedrock, ~0.19) and
/// 493× below the global `10Be` outcrop median (Portenga & Bierman 2011). The
/// **target** was the stable-craton band, **1–10 m/Myr**.
///
/// # ⚠ IT IS OFF, THE BAND WAS NOT REACHED, AND THAT IS THE RESULT
///
/// `45` is **not** the multiplier that lands in the craton band, and no multiplier is:
/// the measured ladder (`examples/denudation_probe.rs`, journal/0114) reaches 1 m/Myr
/// only past ~600×, and by then the world carries **hundreds of metres of mean
/// regolith**. The world cannot be scaled into the band, and the reason is the finding:
///
/// > **Export is proportional to mean regolith thickness, because the only working
/// > sediment router moves one cell per epoch and only the shoreline ring exports.**
/// > Hillslope creep's flux limiter binds on ~89 % of the cells that have regolith to
/// > move *already, at the shipped rates* — the pass is a conveyor, not a diffusion, and
/// > no increase in `diffusion` speeds it up (100× on transport alone buys 1.6×). Raise
/// > supply and the cover thickens until the taper `exp(−H/H*)` shuts the weathering
/// > front off, so the landscape buys denudation by burying itself.
///
/// **And turning it on costs three measured things**, which is why
/// [`production_config`] leaves `calibrated_rates` **false**:
///
/// 1. **Deep closed depressions at ANY multiplier above 1×** — 0 pits deeper than 1 m
///    at 1×, **44 at 5×** (deepest 45 m), 66 at 10×, 148 at 45× (deepest 112 m). The
///    never-incise-below-the-lowest-receiver clamp is defeated once erosion is fast
///    enough for the phases that run *after* incision to lower a cell further in the
///    same epoch. **A latent defect in the solve, not a property of this number** — it
///    was invisible only because the world barely eroded, and it is what has to be
///    fixed before the flag flips.
/// 2. The geotherm's coal-relocation claim collapses from a 1.37× to a 1.02×
///    separation, because 45× deposition makes burial depth rather than crustal
///    gradient the dominant control on coalification.
/// 3. Mean regolith reaches 43.9 m — the top of the published deeply-weathered-shield
///    range, against 4.6 m shipped.
///
/// # Where 45 came from, so the flip has a number to flip to
///
/// It is the largest multiplier that keeps the world's **shape**: journal/0111's
/// conclusion was *"the shape is right and the clock is wrong"*, so a multiplier that
/// moves the shape has stopped being a calibration. Relief within 5 %: **+4.6 %** at
/// 45×, +6.3 % at 50×. Three published bands corroborate it — mean regolith lands in
/// the 30–60 m deeply-weathered-shield range (Yilgarn, Guiana, Brazilian shield);
/// denudation enters the **0.1–1 m/Myr floor band** (McMurdo, Atacama), so the world
/// stops being below *every* published band, which was journal/0111's headline; and
/// `D1/D4`, erosion's authority over the topography, goes **0.027 → 0.91**.
///
/// The world's own Airy ceiling says the band was the right target even though it is out
/// of reach: compensation returns `(ρ_m − ρ_c)/ρ_m = 15.2 %` of each eroded metre as a
/// surface drop, so a steady-state landscape denudes at `U/0.152 ≈ 6.6 U`, and with the
/// measured `U ≈ 0.41 m/Myr` that is **2.70 m/Myr** — inside the craton band, from two
/// densities and a measured uplift that nobody chose for this purpose. **The rates can
/// be raised to meet it; the router cannot carry it.**
///
/// **Heirs:** the pit defect above (blocking), then an efficient long-distance sediment
/// router — rivers that actually carry (fluvial yield is 0.02 % of export), or a creep
/// operator not capped at one cell per timestep. `stubs.md` § 27.
///
/// `1.0` reproduces the shipped world exactly (`x * 1.0 == x` for f64).
pub const EROSION_CALIBRATION: f64 = 45.0;

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
    /// scales the global erosion rates *together* by this factor via
    /// [`scale_erosion_rates`], so the **relative** rates (and therefore the
    /// differential-erosion signal the erodibility coupling expresses) never change;
    /// only the total amount of material erosion is allowed to move. This is the
    /// TERRAIN (erosion) amplitude, distinct from `thickening_scale` above, which is
    /// the TECTONIC (orogenic) amplitude — the term collision the corpus already had
    /// to disambiguate (journal/0040, ROADMAP § the erodibility rider).
    ///
    /// **STUB #24 CLOSED 2026-07-26 (journal/0114): it now scales `diffusion` too.**
    /// It used to scale `weathering`, `k_transport` and `k_bedrock` and *not*
    /// `diffusion` — the process carrying **96 %** of this world's denudation
    /// (journal/0111, corrections #56) — so 100× moved catchment-averaged denudation
    /// by 1.4× and the knob could not move the quantity it is named after. It goes
    /// through the same [`scale_erosion_rates`] the shipped calibration does, which
    /// is what stops the two from ever scaling different sets again.
    ///
    /// **It multiplies whatever the calibration left**, so `--erosion-budget 2` always
    /// means "twice as much erosion as this world has" — before the calibration flag
    /// flips and after it.
    ///
    /// `None` = production default (the shipped calibration, multiplier `1×`).
    /// `Some(1.0)` is **byte-identical** to `None` (`x * 1.0 == x` exactly), so
    /// the flag's off-state is provably inert (asserted in the plumbing tests).
    /// A dev launch flag (`--erosion-budget <mult>`) sets it; the walkable
    /// cranked world it enables is the standing "conservative amplitude" call.
    pub erosion_budget: Option<f64>,
    /// Override [`DeepConfig::calibrated_rates`] — **the switch for the erosional
    /// calibration** (journal/0114). `None` = production default (**OFF**);
    /// `Some(true)` builds the world with all four erosion rate constants multiplied
    /// by [`EROSION_CALIBRATION`].
    ///
    /// **It is off because of what turning it on measured**, not out of caution: the
    /// published 1–10 m/Myr band is unreachable at any multiplier, and above 1× the
    /// incision clamp starts leaving deep closed depressions (44 pits at 5×, 148 at
    /// 45×, deepest 112 m) — a latent defect in the solve that only a world which
    /// actually erodes could expose. See [`EROSION_CALIBRATION`].
    ///
    /// The on path is not merely reachable, it is **pinned by name**:
    /// `tests/calibrated_rates.rs` asserts it reproduces `GOLDEN_SURFACE_CALIBRATED` /
    /// `GOLDEN_RECORD_CALIBRATED` bit for bit. That is the mirror of the
    /// single-receiver / scalar-load / anonymous-creep fixed points — those pin a past
    /// that stays reachable, this pins a future that is already built, so the eventual
    /// flip arrives as a diff rather than as an unmeasured surprise.
    ///
    /// A dev launch flag (`--calibrated-rates`) sets it.
    pub calibrated_rates: Option<bool>,
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
            && self.calibrated_rates.is_none()
    }
}

/// The production deep-time config for a given coarse grid: 460 m where it fits
/// under [`DEEP_MAX_WIDTH`], coarser for very large extents. Recorder on; the
/// sea-level/climate cycling defaults from [`DeepConfig`] drive read-quality.
///
/// **Byte-identical to `production_config_with(cells, seed, &Default::default())`
/// by construction** — it *is* that call. The equality used to be a property two
/// functions maintained in parallel (and a test asserted); since journal/0114 added
/// a step that must run in both, it is a call graph instead. The test stays, because
/// a structural guarantee that nobody checks is a comment.
pub fn production_config(cells: &CellGrid, seed: u64) -> DeepConfig {
    production_config_with(cells, seed, &DeepOverrides::default())
}

/// The production config **before any override or calibration is applied** — the
/// raw flag/rate literal. Private: every caller goes through
/// [`production_config`] or [`production_config_with`], which is what guarantees
/// nothing can obtain a config that skipped the calibration step.
fn production_config_base(cells: &CellGrid, seed: u64) -> DeepConfig {
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
        // **The erosional calibration is BUILT AND OFF** (journal/0114), and the
        // reason it is off is measured rather than cautious.
        //
        // journal/0111 found this world denuding at 0.0110 m/Myr — 9× slower than
        // the slowest landscape ever measured on Earth — and the brief was to
        // calibrate the four rate constants until denudation landed in the published
        // 1–10 m/Myr stable-craton band. The whole mechanism for that is here and
        // works: `EROSION_CALIBRATION`, one `scale_erosion_rates`, a pinned
        // fixed point, a launch flag, and a probe that derives the multiplier. What
        // the probe found is that **the band is not reachable at any multiplier**,
        // and that turning the flag on costs three things the shipped world should
        // not pay without the user seeing them:
        //
        // 1. **Deep closed depressions appear at ANY multiplier above 1×.** Measured
        //    on the `mfd_routing` fixture: 0 pits at 1×, **44 at 5×** (deepest 45 m),
        //    66 at 10×, 148 at 45× (deepest 112 m). The never-incise-below-the-lowest-
        //    receiver clamp is defeated once erosion is fast enough for the phases
        //    that run *after* incision to lower a cell further in the same epoch.
        //    **That is a latent defect in the solve, not a consequence of the number**
        //    — it was invisible only because the world barely eroded. It is the single
        //    most important thing journal/0114 found and it wants its own slice.
        // 2. The geotherm's central physical claim (coal relocates onto warm crust)
        //    collapses from a 1.37× to a 1.02× separation, because 45× deposition
        //    makes burial depth rather than crustal gradient the dominant control.
        // 3. Mean regolith reaches 43.9 m, at the top of the published
        //    deeply-weathered-shield range.
        //
        // So it ships **reachable and off**: `--calibrated-rates` sets
        // `DeepOverrides::calibrated_rates: Some(true)`, the ON world is pinned by
        // name (`tests/calibrated_rates.rs`), and flipping this line to `true` is the
        // whole change once the pit defect is fixed. **This is an appearance-class,
        // user-owned call and the evidence for it is journal/0114** — an agent
        // landing 148 unfilled pits and a broken coal mechanism into the world the
        // player walks, on its own authority and while the user was away, is not a
        // calibration, it is a regression with a good story.
        calibrated_rates: false,
        ..DeepConfig::default()
    }
}

/// The production config with gen-time [`DeepOverrides`] applied on top: start
/// from [`production_config_base`], then overwrite each flag the caller set. Every
/// `None` override inherits, so `production_config_with(cells, seed,
/// &DeepOverrides::default())` is **byte-identical** to `production_config(cells,
/// seed)` (asserted in the tests — and since journal/0114 the latter *is* this call
/// with an empty override, so the identity is structural as well as tested). This
/// is the single seam a launch flag reaches the deep-time run through.
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
    let mut cfg = production_config_base(cells, seed);
    // **The calibration is applied FIRST and the budget multiplies it**, which is
    // what makes `--erosion-budget 2` mean "twice this world" rather than "twice
    // some other world". It also makes one identity true bit-for-bit, and that
    // identity is the derivation's own falsifier (`tests/calibrated_rates.rs`):
    // `erosion_budget: Some(EROSION_CALIBRATION)` on the production config is the
    // world `calibrated_rates: Some(true)` builds, because both paths run the same
    // multiply on the same operands in the same order.
    if let Some(v) = overrides.calibrated_rates {
        cfg.calibrated_rates = v;
    }
    if cfg.calibrated_rates {
        scale_erosion_rates(&mut cfg, EROSION_CALIBRATION);
    }
    if let Some(v) = overrides.tectonic_history {
        cfg.tectonic_history = v;
    }
    if let Some(v) = overrides.full_agents {
        cfg.full_agents = v;
    }
    if let Some(v) = overrides.thickening_scale {
        cfg.thickening_scale = v;
    }
    // Erosion budget: scale the global erosion rates *together*
    // (`erodibility_probe` experiment B), so the relative rates the erodibility
    // coupling reads never move — only the total amount of erosion does. A
    // multiplier of `1.0` leaves each rate bit-for-bit unchanged (`x * 1.0 == x`
    // for f64), which is why `Some(1.0)` is byte-identical to `None` (the
    // falsifier in the plumbing tests). The multiply is unconditional on
    // `erodibility`: these are the base rates the run uses either way, and the
    // coupling — when on — modulates around them without changing this scaling.
    //
    // It goes through the SAME `scale_erosion_rates` the calibration above uses
    // (stubs #24, closed journal/0114): the knob's scope and the default's scope
    // are one function, so they cannot drift apart again.
    if let Some(mult) = overrides.erosion_budget {
        scale_erosion_rates(&mut cfg, mult);
    }
    if let Some(v) = overrides.weather_inventory {
        cfg.weather_inventory = v;
    }
    cfg
}

/// **One deep cell's mass-coupled read bundle** — the payload of
/// [`DeepField::cell_bundle`] (P11 slice 3, F2). Record, regolith `H` and the
/// weathering ledger of the SAME cell travel together; the ledger is private so
/// its bedrock slot can only be indexed by **this** record's length
/// ([`Self::weathering_product_m`]) — mixing parents is a type error, not a
/// discipline.
pub struct CellBundle<'a> {
    /// Row-major deep-cell index this bundle names.
    pub cell: usize,
    /// The cell's strata record.
    pub record: &'a DeepStrata,
    /// The same cell's regolith `H` (metres); `None` when the run kept no
    /// regolith plane (synthetic tests).
    pub regolith_m: Option<f64>,
    ledger: Option<LedgerView<'a>>,
}

impl CellBundle<'_> {
    /// Metres of loose weathering product from the bedrock seam, folded at THIS
    /// record's own bedrock slot (`record.units.len()`) — the only index that
    /// slot can legally take. `0.0` when inventory weathering is off.
    pub fn weathering_product_m(&self) -> f64 {
        self.ledger
            .as_ref()
            .map_or(0.0, |l| l.weathering_product_m(self.record.units.len()))
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
    build_field_cfg_cadence(cells, cfg, &super::cadence::CadenceTable::empty())
}

/// [`build_field_cfg`] with the world's **authored pass cadence** (the RATE axis,
/// journal/0123). An empty table is [`build_field_cfg`] exactly — same runner,
/// same schedule, same bits — which is what lets the RATE acceptance test compare
/// against the shipped goldens through the *same* distillation the goldens were
/// captured through, rather than through a second path that would have to be
/// argued equivalent.
pub fn build_field_cfg_cadence(
    cells: &CellGrid,
    cfg: &DeepConfig,
    cadence: &super::cadence::CadenceTable,
) -> DeepField {
    build_field_cfg_cadence_geology(cells, cfg, cadence, &dc_core::materials::geology::vanilla())
}

/// [`build_field_cfg_cadence`] over an **explicit geology content set** — the
/// deep tier's content door (P11 slice 1; see [`super::run_cells_with_geology`]
/// for why the deep sim now needs one, and for the plumbing gap that is still
/// open above it).
pub fn build_field_cfg_cadence_geology(
    cells: &CellGrid,
    cfg: &DeepConfig,
    cadence: &super::cadence::CadenceTable,
    geology: &dc_core::materials::geology::GeologySet,
) -> DeepField {
    let run = super::run_cells_with_geology(cells, cfg, true, cadence, geology);
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

    /// **The deep grid's [`Registration`] — the affine half of
    /// [`Self::deep_coords`], IN VOXELS.**
    ///
    /// Handed out so a consumer building a [`CoarseField`](dc_core::coarse::CoarseField)
    /// over these cells registers it the way this field actually samples, instead
    /// of reconstructing the pitch from a constant.
    ///
    /// ## ⚠ UNITS: VOXELS, and the pitch is NOT `DEEP_CELL_M`
    ///
    /// `cell_size` is `CELL_VOXELS · wp / w` **voxels** — the sampling pitch
    /// `deep_coords` implies, not a metre figure. Two traps it exists to close
    /// (member-#0 design pass, MM-6):
    ///
    /// - [`DEEP_CELL_M`] is **metres** (460.0). A voxel is 0.9 m at the N=2 player
    ///   scale, so reading 460 as a voxel count is an **11 % registration error**
    ///   — and `Registration`'s own doc-test in `dc-core` models 460 as voxels,
    ///   which is exactly how that mistake gets made.
    /// - Even `DEEP_CELL_M / 0.9` is wrong in general: `w` is **capped** at
    ///   [`DEEP_MAX_WIDTH`], so at [`Extent::Large`](crate::pregen::Extent::Large)
    ///   the real cell is ~1.8 km, and even below the cap `w` is a rounded cell
    ///   count, so the true pitch is `extent / w` rather than the target 460 m.
    ///   **Derive it from the grid, never from the constant.**
    ///
    /// Multiply `cell_size` by the voxel edge to get metres.
    ///
    /// This is a *derived view* of [`Self::deep_coords`], which stays the
    /// authority (it also carries the extent test this affine map does not — see
    /// there). Their agreement is pinned by
    /// `registration_agrees_with_deep_coords`.
    pub fn registration(&self) -> dc_core::coarse::Registration {
        // `deep_coords`, unfolded: gx = vx·w/(CELL_VOXELS·wp) + (wp/2)·w/wp − 0.5.
        // The constant term uses INTEGER `wp/2`, exactly as `deep_coords` does —
        // it is not `w/2` when `wp` is odd.
        let cell_size = CELL_VOXELS as f64 * self.wp as f64 / self.w as f64;
        let half = (self.wp / 2) as f64;
        let origin = -(half * self.w as f64 / self.wp as f64 - 0.5) * cell_size;
        dc_core::coarse::Registration::new(origin, origin, cell_size)
    }

    /// Continuous deep-grid coordinates (cell centres at integer coords) for a
    /// world voxel, or `None` when the voxel lies outside the civilized pregen
    /// extent (the border wilds have no deep-time history — the collapse layer
    /// falls back to the analytic elevation there).
    ///
    /// **The authority for this grid's registration**, and the `None` is half of
    /// what it says: [`Self::registration`] is the affine map alone and cannot
    /// answer "is this voxel inside the record at all", so a consumer that needs
    /// both asks here first.
    #[inline]
    pub fn deep_coords(&self, vx: i64, vz: i64) -> Option<(f64, f64)> {
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

    /// The strata record of deep cell `(ix, iy)`, **edge-clamped** to the grid —
    /// the *producer-side* enumeration a
    /// [`CoarseField`](dc_core::coarse::CoarseField) is constructed from.
    ///
    /// [`Self::record_at_voxel`] is the fine read and answers a world position;
    /// this answers a **cell index**, which is what filling a window of cells
    /// needs. It is not the raw per-cell read `CoarseField` forbids: that ban is on
    /// the *expression* side (a fine consumer fetching one cell's verdict and
    /// painting it across a span). Here the caller is assembling the field the
    /// consumer will then sample legally — the sanctioned construction path.
    ///
    /// The clamp is the same edge extension `record_at_voxel` applies after
    /// rounding, and the same one `CoarseField`'s bilinear stencil applies at its
    /// rim, so a window that reaches past the grid reads the rim cell rather than
    /// nothing.
    pub fn record_at_cell(&self, ix: i64, iy: i64) -> Option<&DeepStrata> {
        if self.strata.is_empty() {
            return None;
        }
        let ix = ix.clamp(0, self.w as i64 - 1) as usize;
        let iy = iy.clamp(0, self.w as i64 - 1) as usize;
        self.strata.get(iy * self.w + ix)
    }

    /// **The three mass-coupled reads of ONE deep cell, answered together**
    /// (P11 slice 3, design audit F2): the strata record, the regolith `H`, and
    /// the weathering ledger — by **cell index**, for the near path's per-column
    /// membership dither.
    ///
    /// The coupling is why this is one call and not three: `H` is *exactly* the
    /// sum of the same cell's record unit thicknesses (journal/0053's finalize
    /// invariant), the collapse tier's surficial veneer is the **difference** of
    /// the two, and the ledger's bedrock slot index is `record.units.len()` — a
    /// per-record quantity. Every one of those conserves mass only when all
    /// three name the **same cell**. A consumer that dithers the record and
    /// reads `H` or the ledger from anywhere else has written a Law-3 leak at
    /// every boundary column; routing the collapse tier exclusively through this
    /// bundle is what makes that un-bundled read inexpressible there.
    ///
    /// Edge-clamped like [`Self::record_at_cell`]. `None` when there is no
    /// record grid at all.
    pub fn cell_bundle(&self, ix: i64, iy: i64) -> Option<CellBundle<'_>> {
        if self.strata.is_empty() {
            return None;
        }
        let ix = ix.clamp(0, self.w as i64 - 1) as usize;
        let iy = iy.clamp(0, self.w as i64 - 1) as usize;
        let i = iy * self.w + ix;
        let record = self.strata.get(i)?;
        Some(CellBundle {
            cell: i,
            record,
            regolith_m: self.regolith.get(i).copied(),
            ledger: if self.ledgers.is_empty() {
                None
            } else {
                self.ledgers.get(i)
            },
        })
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
mod tests {
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

    /// **The summary agrees with the authority** (CLAUDE.md § "a summary is not an
    /// authority"): [`DeepField::registration`] is a *derived view* of
    /// [`DeepField::deep_coords`], which stays the one place the grid's
    /// registration is computed. A drift between them is a silent 11 %-class
    /// sampling error — precisely the MM-6 hazard the accessor exists to close —
    /// so it gets an agreement test rather than a comment.
    ///
    /// Tolerance, not bit-equality, and deliberately: the two evaluate the same
    /// affine map in a different operation order (`deep_coords` folds through the
    /// pregen coordinate; the registration divides once), so they agree to f64
    /// rounding rather than bit-for-bit. `1e-9` **cell widths** is ~4 µm of world
    /// at the shipped pitch — orders below any consequence, and tight enough that
    /// a real registration mistake (a half-cell shift, an odd-`wp` off-by-one, a
    /// metres/voxels confusion) fails by many orders.
    #[test]
    fn registration_agrees_with_deep_coords() {
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let field = DeepField::from_pregen(&pregen);
        let reg = field.registration();
        let mut checked = 0usize;
        for vx in [-300_000i64, -12_345, -1, 0, 1, 7_777, 250_000] {
            for vz in [-260_000i64, -9_001, 0, 3, 40_000, 310_000] {
                let Some((gx, gy)) = field.deep_coords(vx, vz) else {
                    continue;
                };
                let (rx, ry) = reg.cell_coords(vx as f64, vz as f64);
                assert!(
                    (gx - rx).abs() < 1e-9 && (gy - ry).abs() < 1e-9,
                    "registration disagrees at ({vx},{vz}): deep_coords ({gx},{gy}) \
                     vs registration ({rx},{ry})"
                );
                checked += 1;
            }
        }
        assert!(
            checked > 4,
            "only {checked} voxels landed inside the extent"
        );
        // And the units are VOXELS: the pitch times the voxel edge is the metre
        // cell, which is at least the target and generally not equal to it.
        let cell_m = reg.cell_size * 0.9;
        assert!(
            cell_m >= DEEP_CELL_M - 1e-6,
            "the derived cell is {cell_m} m, under the {DEEP_CELL_M} m target — \
             the registration is probably in metres, not voxels"
        );
    }

    /// [`DeepField::record_at_cell`] is the producer-side twin of
    /// [`DeepField::record_at_voxel`]: at the cell a voxel rounds to, the two must
    /// hand back the *same record*, or a `CoarseField` assembled from cell indices
    /// would be registered one cell off from the field it is meant to reconstruct.
    /// Pointer identity, because that is the strongest available statement.
    #[test]
    fn record_at_cell_is_the_producer_side_of_record_at_voxel() {
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let field = DeepField::from_pregen(&pregen);
        let mut checked = 0usize;
        for vx in [-90_000i64, -4_321, 0, 5, 61_000] {
            for vz in [-70_000i64, -37, 0, 12_345] {
                let Some((gx, gy)) = field.deep_coords(vx, vz) else {
                    continue;
                };
                let by_voxel = field.record_at_voxel(vx, vz).expect("inside the extent");
                let by_cell = field
                    .record_at_cell(gx.round() as i64, gy.round() as i64)
                    .expect("the same cell");
                assert!(
                    std::ptr::eq(by_voxel, by_cell),
                    "record_at_cell disagrees with record_at_voxel at ({vx},{vz})"
                );
                checked += 1;
            }
        }
        assert!(
            checked > 2,
            "only {checked} voxels landed inside the extent"
        );
        // The clamp is edge extension, not a `None`: a window that overhangs the
        // grid reads the rim cell, exactly as the bilinear stencil does.
        let rim = field.record_at_cell(-4, -4).expect("clamped to the rim");
        assert!(std::ptr::eq(rim, field.record_at_cell(0, 0).unwrap()));
    }
}
