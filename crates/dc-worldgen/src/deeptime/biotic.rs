//! **S10 — the biotic layer on the deep-time A-tier.** A per-cell ecological
//! community, propagated over the same eons the two-plane erosion runs, that
//! (a) writes organic material into the strata record — coal seams, paleosols,
//! charcoal bands, retrogressive surfaces — and (b) feeds back as an erosion and
//! weathering term (root cohesion resists hillslope creep; root acids /
//! mycorrhizae accelerate chemical weathering). This is the spike that measures
//! whether the community-vector substrate designed in `docs/design/ecology.md`
//! § 3 earns its cost and reads legibly. Evolution is explicitly OUT of S10.
//!
//! ## The community vector + soil state (ecology.md § 3)
//!
//! Per cell we carry a small vegetation-cover vector over a fixed vanilla
//! species [`ROSTER`] plus ~7 soil scalars ([`CellBiota`]). The design's
//! **top-K co-occurrence cap** (the S8 mixture-cap lesson — K bounds
//! co-occurrence, never roster size) is what makes a *large* organism roster
//! affordable; at this spike's 7-species roster the full vector already fits, so
//! the cap is not stressed here (FLAGGED: with a production-scale roster the
//! stored state becomes the top-K community vector, not the full roster — the
//! cover buffer would carry `[(species, cover); K]`).
//!
//! ## The six processes per epoch (ecology.md § 3), as ordered read/write phases
//!
//! Each epoch step, per cell, runs the six designed processes as an explicit
//! creator/modifier/reader sequence over the biotic state (the pass-graph
//! discipline, applied inside the loop — the pregen [`Pipeline`](crate::pipeline)
//! itself topo-sorts *once*, so the per-epoch biotic pipeline is expressed here
//! as a fixed ordered phase list instead):
//!
//! 1. **Suitability** — `min` over each species' tolerance ranges (Liebig's
//!    minimum: growth caps on the *scarcest* resource, so the operator is `min`,
//!    never a weighted mean). Reads climate + soil + nutrients.
//! 2. **Dispersal** — a bounded neighbour kernel gates which species can arrive
//!    (a *relaxing* process by S9's classification — no global reach). Reads the
//!    frozen previous-epoch cover of the 8 neighbours.
//! 3. **Competition** — finite cover capacity allocated by suitability weighted
//!    by incumbency (inhibition: established shade-casters resist). Reads/writes
//!    cover.
//! 4. **Nutrient cycling** — uptake, litter, decomposition, rock-P weathering
//!    release, leaching. The Walker & Syers curve (P depletes monotonically, N
//!    accumulates then plateaus, ending in P-limited retrogression) *emerges*.
//! 5. **Niche construction** — write back soil depth, the erosion-resistance and
//!    weathering multipliers the *next* erosion step consumes, and the fuel load.
//! 6. **Disturbance** — fire from fuel × aridity (deterministic addressed
//!    ignition), flood from drainage area; resets succession and leaves a tagged
//!    mark (a charcoal band).
//!
//! ## Lagged coupling — the pass-graph cycle, resolved (ecology.md § 3)
//!
//! Biology modifies erosion; erosion modifies terrain; terrain sets climate and
//! soil; soil sets biology. A single-epoch pass graph would (correctly) refuse
//! that cycle. The fix, as designed: biology reads *this* epoch's post-erosion
//! terrain and writes modifiers the *next* epoch's erosion consumes. So the run
//! loop is `erosion.step` (consuming last epoch's biotic modifiers) → then
//! `biotic.step` (reading the fresh surface, writing next epoch's modifiers +
//! this epoch's organic deposits). On epoch 0 the modifiers are the defaults
//! (`1.0` weathering, `0.0` resistance), so epoch 0's erosion is byte-identical
//! to a biology-free run.
//!
//! ## Determinism
//!
//! All biotic entropy is addressed draws seeded by `(seed, SALT, gx, gy, epoch)`
//! (project law — no wall clock, no ambient randomness). Neighbour reads use a
//! frozen previous-cover snapshot, and every other read/write is cell-local, so
//! the whole step is a pure per-cell function — the scalar and data-parallel
//! drivers produce byte-identical output, exactly as the erosion per-cell phases
//! do (S9b).

use dc_sim::statistical::rng::draw_f64;

use super::erosion::Erosion;
use super::geotherm::{self, BurialColumn};
use super::grid::{DeepConfig, DeepGrid, SEA_LEVEL_M};
use super::providers::{ParentCell, Providers, WaterPass, wet_at};
use super::recorder::{Aridity, Biofacies, DepEnv, DepTag, EnergyBand};

/// Addressed-draw salts for the biotic layer. Distinct high byte from pregen
/// (`0x5700_*`) and deep-time erosion (`0x5900_*`) so the address spaces never
/// collide.
const SALT_BIO_FIRE: u64 = 0x5B00_0001;
const SALT_BIO_FLOOD: u64 = 0x5B00_0002;

/// Legibility floor (metres) the read-quality scan uses for "a coal seam you
/// could see in a cut face". A **reporting** threshold on seam thickness, not a
/// process: it decides which seams get counted, never which peat becomes coal.
///
/// Until journal/0063 this constant was doing both jobs, and the second one was
/// on the wrong axis — see [`COAL_BURIAL_M`] and
/// [`DeepStrata::promote_coal`](super::recorder::DeepStrata::promote_coal).
pub const COAL_MIN_M: f64 = 0.4;

/// **Overburden (metres) at which buried peat becomes coal** — the burial
/// threshold `promote_coal` applies at run finalize (earth-processes.md § 5).
///
/// Coalification is driven by pressure and temperature, both of which rise with
/// burial. On Earth the peat→lignite transition wants of order 10²–10³ m of
/// section. Our recorded column is the deep sim's *regolith* plane `H`, whose
/// deepest cells carry ~100 m and whose median subaerial cell carries a few
/// metres, so an Earth-calibrated threshold would promote **nothing anywhere**:
/// measured on the production Medium world, of 35 382 peat-derived units exactly
/// **13** lie under 50 m of section and **one** under 100 m. There is no coal in
/// a world that only ever buries peat under four metres of mud.
///
/// So the number is calibrated to *this* record's burial distribution rather
/// than to Earth's, and the calibration statement is deliberately a shape and
/// not a target: **coal is what happens to the peat that got buried deepest.**
/// 8 m is the ~90th percentile of the measured overburden distribution
/// (histogram in journal/0063), which promotes 11 % of peat-derived units —
/// 3 888 units in 2 216 columns, 0.78 % of readable columns. The shipped
/// thickness rule promoted 19 008 units in 12 892 columns (4.52 %); the axis
/// change is a 5.8× *reduction* in coal, because thick-and-shallow is common in
/// this world and deep is not. That reduction is the fix's deliverable, not its
/// cost.
///
/// **Since journal/0093 the heir has LANDED** — the geotherm ([`super::geotherm`],
/// the first §5 field pass). Coal rank no longer thresholds this depth; it
/// thresholds the geotherm's `temperature` field at the burial depth against
/// [`COAL_ONSET_C`] (now a real °C, no longer this depth wearing degrees). This
/// constant is retained only as the **degenerate-baseline threshold** the
/// coal-shift probe measures against (the "before": overburden ≥ 8 m), so
/// journal/0093 can report how far the geotherm moved coal. It drives no
/// promotion any more.
pub const COAL_BURIAL_M: f64 = 8.0;

/// **The coalification onset temperature (°C)** — the threshold
/// [`DeepStrata::promote_coal`](super::recorder::DeepStrata::promote_coal)
/// applies to the **geotherm** temperature at a candidate's burial depth
/// (`T = surface_T + gradient·depth`, [`super::geotherm`]) at run finalize.
///
/// **Recalibrated with the geotherm** (journal/0093). It used to be an *alias*
/// for [`COAL_BURIAL_M`] (8.0), because the degenerate `burial_temp_c` provider's
/// "temperature" was literally the overburden in metres, so an 8.0 onset *was*
/// the 8 m burial rule. The moment a real geotherm lands that aliasing is
/// catastrophic: surface **air** temperature alone clears 8 °C almost everywhere
/// a peat swamp forms (warm wet lowlands), so `surface_T + gradient·depth ≥ 8`
/// would promote essentially the whole record — the "retire together" warning
/// `stubs.md` §14 wrote out in full.
///
/// So this is now a genuine onset temperature. Earth's peat→lignite transition is
/// ~50 °C; but this sim's record buries peat only a few metres to ~100 m, so the
/// burial term (`gradient·depth`, a few °C) is small next to the surface term
/// (~10–30 °C by latitude/altitude). An Earth-true 50 °C onset would therefore
/// promote **nothing**. The number is instead calibrated to *this* record's
/// temperature distribution (journal/0093 measured the candidate-unit `T` on the
/// production Small world), chosen so the geotherm coal fraction stays the same
/// order as the retired 8 m rule — plausible, not all-or-nothing. What the
/// geotherm *changes* is not the count but the **place**: coal now concentrates
/// where the crust is warm (warm lowlands, steep-gradient rift/arc crust) instead
/// of wherever peat happened to be buried deepest regardless of climate. That
/// shift is the reason the world is walked (§14: "we measure the shift").
///
/// **Measured (journal/0093, production Small, seed 0x…0059).** The candidate
/// units' geotherm temperatures cluster tightly at ~21–26 °C — because burial is
/// shallow (a few metres to ~100 m), so the `gradient·depth` term is a fraction
/// of a degree and the *surface* term (warm wet lowlands, where peat forms) sets
/// the temperature. `25.0 °C` promotes **23 %** of candidates there, against the
/// retired 8 m rule's **12 %** — the same order, plausibly not degenerate, and
/// the coal it selects has moved to the warm crust. (At 22 °C it was 60 %, at
/// 28 °C zero: the cluster is narrow, so the onset lives inside it.)
///
/// The exact number is not precious (coal is a placeholder until real biology),
/// but it **must not degenerate** — the `geotherm` test pins the coal fraction
/// into a sane band.
pub const COAL_ONSET_C: f64 = 25.0;

/// Number of species in the vanilla organism roster (the biotic analogue of the
/// vanilla geology set). K-cap not stressed at this size — see module docs.
pub const ROSTER: usize = 7;

/// A species' niche contract (ecology.md § 3): tolerance ranges over the context
/// axes, nutrient demands, and traits. A registry-defined organism pack would
/// publish these; the spike ships a fixed vanilla roster.
#[derive(Clone, Copy, Debug)]
struct Niche {
    name: &'static str,
    /// Temperature tolerance (°C), with a soft margin outside.
    temp_lo: f32,
    temp_hi: f32,
    /// Moisture tolerance (normalized precip 0..1).
    moist_lo: f32,
    moist_hi: f32,
    /// Minimum soil depth (m) for full suitability (facilitation gate).
    soil_min: f32,
    /// Phosphorus demand (the Liebig term that drives retrogression).
    p_demand: f32,
    /// Nitrogen demand (fixers exempt).
    n_demand: f32,
    /// Fixes atmospheric N (raises soil N → facilitates successors).
    n_fixer: bool,
    /// Competitive incumbency weight (shade / inhibition).
    shade: f32,
    /// Organic litter produced per unit cover per epoch.
    litter: f32,
    /// Root-cohesion contribution per unit cover (erosion resistance, 0..1).
    root: f32,
    /// Biological-weathering boost per unit cover.
    weather: f32,
    /// Fuel / flammability per unit cover.
    flammable: f32,
    /// Preference for waterlogged sites (peat formers → 1.0).
    waterlog: f32,
}

/// The vanilla organism roster: a minimal successional series that exercises all
/// four target signals. FLAGGED (ecology.md is silent on the S10 roster): these
/// niches are the simplest set consistent with the doc's succession mechanism
/// (facilitation lichen→soil→forest; inhibition by shade-casters; the Walker &
/// Syers P-limited endgame). Real rosters are registry/pack content.
/// **Calibration note (measured, not guessed).** The moisture ranges below are
/// set against the *actual* land distribution the orographic march produces on
/// this engine — p05 0.046, p50 0.059, p75 0.069, p95 0.151, max 0.805 — not an
/// imagined uniform 0..1 field. An earlier draft used 0..1-style thresholds
/// (forest at `moist_lo = 0.40`) and every later-successional species was
/// stillborn: nothing but lichen could satisfy a single tolerance term anywhere
/// on the map. Likewise the P demands are set against the measured available-P
/// spread, which is strongly bimodal (rejuvenated surfaces fertile, ancient
/// stable ones depleted) — that spread IS the chronosequence.
const NICHES: [Niche; ROSTER] = [
    // 0 — pioneer lichen/crust: colonizes bare rock, huge weathering (makes
    // soil), negligible cohesion. The facilitation engine.
    Niche {
        name: "lichen",
        temp_lo: -18.0,
        temp_hi: 34.0,
        moist_lo: 0.02,
        moist_hi: 1.0,
        soil_min: 0.0,
        p_demand: 0.008,
        n_demand: 0.02,
        n_fixer: false,
        shade: 0.10,
        litter: 0.015,
        root: 0.02,
        weather: 1.6,
        flammable: 0.05,
        waterlog: 0.0,
    },
    // 1 — moss/grass sward: thin soil, moderate cohesion + litter.
    Niche {
        name: "grass",
        temp_lo: -6.0,
        temp_hi: 30.0,
        moist_lo: 0.045,
        moist_hi: 0.35,
        soil_min: 0.04,
        p_demand: 0.020,
        n_demand: 0.06,
        n_fixer: false,
        shade: 0.30,
        litter: 0.09,
        root: 0.28,
        weather: 0.4,
        flammable: 0.45,
        waterlog: 0.1,
    },
    // 2 — N-fixing shrub: raises soil N, facilitates the forest.
    Niche {
        name: "n-fixer",
        temp_lo: -2.0,
        temp_hi: 32.0,
        moist_lo: 0.050,
        moist_hi: 0.35,
        soil_min: 0.08,
        p_demand: 0.040,
        n_demand: 0.02,
        n_fixer: true,
        shade: 0.50,
        litter: 0.13,
        root: 0.42,
        weather: 0.5,
        flammable: 0.5,
        waterlog: 0.0,
    },
    // 3 — forest: needs the wet tail AND a fertile soil, so it is restricted to
    // humid, rejuvenated ground — the first casualty of P depletion.
    Niche {
        name: "forest",
        temp_lo: 2.0,
        temp_hi: 30.0,
        moist_lo: 0.090,
        moist_hi: 1.0,
        soil_min: 0.28,
        p_demand: 0.060,
        n_demand: 0.06,
        n_fixer: false,
        shade: 0.92,
        litter: 0.21,
        root: 0.62,
        weather: 0.8,
        flammable: 0.38,
        waterlog: 0.0,
    },
    // 4 — peat sedge/swamp: gated on WATERLOGGING (not rainfall), buries organic
    // faster than it decomposes → peat → coal.
    Niche {
        name: "peat-sedge",
        temp_lo: -6.0,
        temp_hi: 28.0,
        moist_lo: 0.050,
        moist_hi: 1.0,
        soil_min: 0.06,
        p_demand: 0.020,
        n_demand: 0.05,
        n_fixer: false,
        shade: 0.62,
        litter: 0.34,
        root: 0.30,
        weather: 0.2,
        flammable: 0.10,
        waterlog: 1.0,
    },
    // 5 — sclerophyll scrub: the most phosphorus-miserly species in the roster,
    // so it inherits the ancient depleted surfaces — the retrogression endpoint.
    Niche {
        name: "sclerophyll",
        temp_lo: 0.0,
        temp_hi: 35.0,
        moist_lo: 0.030,
        moist_hi: 0.16,
        soil_min: 0.12,
        p_demand: 0.006,
        n_demand: 0.03,
        n_fixer: false,
        shade: 0.42,
        litter: 0.05,
        root: 0.34,
        weather: 0.3,
        flammable: 0.72,
        waterlog: 0.0,
    },
    // 6 — fire grass: flammable, fast post-fire recoloniser (disturbance
    // specialist).
    Niche {
        name: "fire-grass",
        temp_lo: -3.0,
        temp_hi: 35.0,
        moist_lo: 0.035,
        moist_hi: 0.22,
        soil_min: 0.02,
        p_demand: 0.015,
        n_demand: 0.04,
        n_fixer: false,
        shade: 0.20,
        litter: 0.06,
        root: 0.22,
        weather: 0.3,
        flammable: 0.82,
        waterlog: 0.05,
    },
];

// --- process/rate constants (tuned so the four signals appear at A resolution;
// the calibration is plausible, not fit to Earth — see S10-results.md) --------

/// Succession rate: fraction of the gap to the competitive target closed per
/// epoch (≈ decades-to-centuries succession over the 200-epoch run).
const SUCCESSION_RATE: f32 = 0.18;
/// Background propagule pressure from the regional species pool (see the
/// dispersal phase — this is what keeps the model out of a bootstrap deadlock).
const BACKGROUND_PROPAGULE: f32 = 0.03;
/// Propagule pressure for the two pioneer colonists (lichen, fire-grass).
const PIONEER_PROPAGULE: f32 = 0.12;
/// Propagule pressure at which the dispersal gate saturates. Below it a species
/// is dispersal-limited (range fronts); above it, competition alone decides.
const ARRIVE_HALF: f32 = 0.15;
/// Competitive-rank floor, so a shade-intolerant pioneer still competes for the
/// bare ground it is good at.
const COMP_FLOOR: f32 = 0.15;
/// Initial rock-derived phosphorus pool (finite → Walker & Syers depletion).
///
/// **This is now the identity value of a provider slot**, not the rule: the pool
/// is a property of the *parent material*, and a uniform constant is a stand-in
/// for a petrology that does not exist yet
/// ([`providers::Providers::parent_p`](super::providers::Providers::parent_p)).
/// [`BioticSim::new`] materializes a per-cell plane from that slot once, and both
/// the initial pool and the rejuvenation cap read the plane rather than this
/// constant. With the identity provider the plane is `1.0` everywhere, so the
/// arithmetic is bit-for-bit what it was.
const P_ROCK_INIT: f32 = 1.0;
/// Initial available P / N / cations on a fresh surface.
const P_AVAIL_INIT: f32 = 0.06;
const N_INIT: f32 = 0.05;
const CAT_INIT: f32 = 0.5;
/// Fraction of the rock-P pool weathered to available P per epoch (veg-boosted).
/// The pool is finite, so this is the **Walker & Syers** clock: release decays
/// as the parent material's phosphorus is used up.
const P_WEATHER_RATE: f32 = 0.015;
/// Per-epoch loss of available P (leaching + occlusion into unavailable forms).
/// Available P therefore tracks the *release rate* rather than accumulating, so
/// it falls as the rock pool depletes — the retrogression driver. Calibrated so
/// a fresh surface sits comfortably above the forest's P demand (0.19) while a
/// depleted one falls below the retrogression threshold (0.012): that ~70×
/// dynamic range is what makes the chronosequence legible rather than uniform.
const P_LOSS: f32 = 0.020;
/// Fresh rock-P returned per metre of surface stripped (erosion) or of mineral
/// sediment delivered: young/rejuvenated surfaces stay fertile, ancient stable
/// ones do not. This is what makes retrogression *geographic* rather than
/// merely a function of elapsed time.
const P_FRESH: f32 = 0.80;
/// N fixed per unit fixer cover per epoch, plus a trickle of atmospheric N.
const N_FIX_RATE: f32 = 0.03;
const N_DEPOSITION: f32 = 0.002;
/// Nutrient uptake efficiency (fraction of demand drawn from the available pool).
const UPTAKE: f32 = 0.5;
/// Leaching loss fraction per epoch (moisture-scaled) for N / cations.
const LEACH: f32 = 0.02;
/// Available-P threshold below which a stable surface goes retrogressive.
const P_RETRO_THRESH: f32 = 0.012;
/// Minimum time-since-disturbance (epochs) for retrogression.
const RETRO_AGE: u16 = 40;
/// Soil-depth gain per metre of net organic accumulation (facilitation).
const SOIL_GAIN: f32 = 1.4;
/// Soil depth lost per metre of surface stripped by erosion.
const SOIL_STRIP: f32 = 0.6;
/// Soil-depth cap (m).
const SOIL_MAX: f32 = 2.0;
/// Minimum organic deposit (m/epoch) worth recording (avoids unit spray).
const ORG_MIN_DEPOSIT: f64 = 0.0008;
/// Maximum mineral deposition (m/epoch) that still permits a soil horizon to
/// form and be recorded. Above this the site is aggrading too fast for a profile
/// to develop — litter is diluted into the mineral unit instead. This is real
/// pedology (soils form during depositional hiatuses) and it is also what keeps
/// the record's unit count from doubling: organic units only appear in quiet
/// intervals, where consecutive epochs merge into one horizon.
const SOIL_HIATUS_MAX: f64 = 0.010;
/// The same gate for peat: a swamp keeps pace with somewhat faster aggradation.
const PEAT_HIATUS_MAX: f64 = 0.030;
/// Fire: fraction of standing organic converted to a charcoal residue band. Most
/// burned carbon leaves for the atmosphere; only a thin residue is preserved.
const FIRE_CHAR_FRAC: f64 = 0.02;
/// Fire: cover kill fraction (fire-adapted species resist — see `fire_kill`).
const FIRE_BASE_KILL: f32 = 0.75;
/// Fuel threshold below which ignition is impossible.
const FUEL_MIN: f32 = 0.15;
/// Ignition-probability scale. At the ratified Phanerozoic calibration one epoch
/// is ~2.5 Myr, so a "fire event" in the record is a fire *regime* interval, not
/// a single burn — this sets how often such an interval leaves a charcoal bed.
const IGNITION_SCALE: f64 = 0.15;
/// Minimum charcoal residue (m) that survives as a discrete, recordable BED.
/// Thinner residues are bioturbated and dispersed into the soil horizon rather
/// than preserved as a lamina — so only substantial fires on well-developed
/// fuel loads enter the record as their own unit. (This is also the single
/// biggest lever on record size: without it, every ignition doubles a column's
/// unit count for a band no cut face could resolve.)
const CHAR_MIN_BAND: f64 = 0.020;
/// Drainage area (cell units) above which a cell is flood-prone.
const FLOOD_AREA: f64 = 120.0;

/// Per-cell community + soil state (ecology.md § 3: ~10 scalars + a community
/// vector). Dropped after the run — only the strata annotations survive.
#[derive(Clone, Copy, Debug)]
pub struct CellBiota {
    /// Vegetation cover fraction per roster species (the community vector).
    pub cover: [f32; ROSTER],
    /// Organic soil depth proxy (m) — facilitation gate for later successors.
    pub soil: f32,
    /// Available nitrogen pool.
    pub n: f32,
    /// Available phosphorus pool.
    pub p_avail: f32,
    /// Finite rock-derived phosphorus pool (depletes monotonically).
    pub p_rock: f32,
    /// Base cations pool.
    pub cations: f32,
    /// Epochs since the last disturbance (fire/flood) — the successional clock.
    pub tsd: u16,
    /// Last epoch's surface elevation, so the layer can see whether the site is
    /// being stripped (erosion → thin young soil on fresh, P-rich rock) or is
    /// stable (deep old soil, P-starved — the chronosequence).
    pub prev_surf: f64,
    /// Last epoch's alluvium thickness. The difference is exactly this epoch's
    /// *mineral* deposition (erosion runs between two biotic steps), which gates
    /// soil formation: a soil horizon needs a depositional **hiatus**; a rapidly
    /// aggrading floodplain buries litter before a profile can develop.
    pub prev_h: f64,
}

impl Default for CellBiota {
    fn default() -> Self {
        Self {
            cover: [0.0; ROSTER],
            soil: 0.0,
            n: N_INIT,
            p_avail: P_AVAIL_INIT,
            p_rock: P_ROCK_INIT,
            cations: CAT_INIT,
            tsd: 0,
            prev_surf: f64::NAN,
            prev_h: 0.0,
        }
    }
}

/// The per-cell outcome of one epoch step (computed in the pure parallel phase,
/// applied in the sequential phase so the record ops and ledger stay ordered).
#[derive(Clone, Copy)]
struct Outcome {
    cell: CellBiota,
    /// Biotic weathering multiplier for the next erosion step (≥ 1.0).
    weather: f32,
    /// Root-cohesion hillslope resistance for the next erosion step (0..1).
    resist: f32,
    /// Organic deposited this epoch (m) and its facies tag.
    org_deposit: f64,
    org_tag: DepTag,
    /// Charcoal residue deposited this epoch (m), if a fire occurred.
    charcoal: f64,
    char_tag: DepTag,
}

/// The biotic simulation state carried across epochs (like [`Erosion`]'s
/// scratch). Dropped after the run; only the strata annotations persist.
pub struct BioticSim {
    w: usize,
    n: usize,
    parallel: bool,
    seed: u64,
    cells: Vec<CellBiota>,
    /// Frozen previous-epoch cover, for the dispersal neighbour kernel.
    prev_cover: Vec<[f32; ROSTER]>,
    /// Per-cell outcome scratch (reused each epoch).
    out: Vec<Outcome>,
    /// **The parent-material phosphorus plane** — the rock-P endowment of the
    /// material each cell's soil is forming on, materialized ONCE from
    /// [`providers::Providers::parent_p`](super::providers::Providers::parent_p)
    /// at construction.
    ///
    /// This is the *pass-level* half of the provider slice: the value is a
    /// property of the parent rock and does not change over the run, so calling
    /// the provider inside the epoch loop would be paying `n × iterations` calls
    /// for `n` distinct answers. The loop reads this plane by index.
    ///
    /// It is both the **initial** pool and the **cap** rejuvenation restores
    /// toward — a stripped surface exposes fresh parent material, and "fresh"
    /// means *this cell's* parent material, not a global constant.
    parent_p: Vec<f32>,
    /// **The water-table plane** — how wet each cell's *site* is, materialized
    /// from
    /// [`providers::Providers::depth_to_water`](super::providers::Providers::depth_to_water)
    /// once per epoch at the top of [`BioticSim::step`].
    ///
    /// **Empty under the identity provider**, which is the point: an empty plane
    /// makes [`providers::wet_at`](super::providers::wet_at) fall through to the
    /// pre-seam three-term proxy, so "provider absent" allocates nothing and is
    /// byte-identical by construction rather than by arithmetic luck. This is
    /// the `bio_weather` / `frost` empty-plane discipline `erosion.rs` already
    /// uses, which is exactly what this seam lacked.
    ///
    /// Re-materialized every epoch, unlike [`Self::parent_p`]: parent material
    /// is fixed for the run, a water table follows the surface the erosion sim
    /// is rewriting.
    wet: Vec<f32>,
    /// The resolved provider set, copied out of the config at construction —
    /// `Option<fn>` pointers, so this is `Copy` and the epoch loop never touches
    /// the config. Only the `depth_to_water` slot is asked here; it is stored
    /// whole so a future pass-level slot needs no new field.
    providers: Providers,
}

impl BioticSim {
    /// Initialize the biotic state for a grid and turn on the grid's biotic
    /// modifier planes (`bio_weather = 1.0`, `bio_resist = 0.0`), so the very
    /// first erosion step is byte-identical to a biology-free run.
    ///
    /// Also materializes the **parent-material phosphorus plane** from
    /// [`providers::Providers::parent_p`](super::providers::Providers::parent_p)
    /// — once, here, never in the epoch loop. Takes the whole [`DeepConfig`]
    /// rather than a bare seed since 2026-07-22 (journal/0060), because the
    /// provider set rides in the config.
    pub fn new(grid: &mut DeepGrid, cfg: &DeepConfig, parallel: bool) -> Self {
        let (seed, w) = (cfg.seed, grid.w);
        let n = grid.w * grid.w;
        let providers = cfg.providers;
        let parent_p: Vec<f32> = (0..n)
            .map(|index| {
                providers.parent_p(ParentCell {
                    index,
                    gx: index % w,
                    gy: index / w,
                }) as f32
            })
            .collect();
        grid.bio_weather = vec![1.0f32; n];
        grid.bio_resist = vec![0.0f32; n];
        let blank = Outcome {
            cell: CellBiota::default(),
            weather: 1.0,
            resist: 0.0,
            org_deposit: 0.0,
            org_tag: DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::Low),
            charcoal: 0.0,
            char_tag: DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::Low),
        };
        let cells: Vec<CellBiota> = parent_p
            .iter()
            .map(|p| CellBiota {
                p_rock: *p,
                ..CellBiota::default()
            })
            .collect();
        Self {
            w: grid.w,
            n,
            parallel,
            seed,
            cells,
            prev_cover: vec![[0.0; ROSTER]; n],
            out: vec![blank; n],
            parent_p,
            // Empty on purpose: the identity `depth_to_water` keeps it empty
            // every epoch, and an empty plane *is* the pre-seam expression.
            wet: Vec::new(),
            providers,
        }
    }

    /// The parent-material phosphorus plane (one entry per cell) this run was
    /// initialized with — the materialized answer of the `parent_p` provider.
    pub fn parent_p(&self) -> &[f32] {
        &self.parent_p
    }

    /// The materialized water-table plane as of the last [`Self::step`] —
    /// **empty under the identity provider**, where wetness is computed inline
    /// from the three-term proxy instead. Non-empty means an heir supplied a
    /// field.
    pub fn wet(&self) -> &[f32] {
        &self.wet
    }

    /// The community + soil state of one cell (spike diagnostics / column dumps).
    pub fn cell(&self, i: usize) -> Option<&CellBiota> {
        self.cells.get(i)
    }

    /// The whole per-cell community field (spike diagnostics).
    pub fn cells(&self) -> &[CellBiota] {
        &self.cells
    }

    /// Working-set footprint (bytes) — the honest "what the biotic layer costs in
    /// memory" number, separate from the grid planes and erosion scratch.
    pub fn scratch_bytes(&self) -> usize {
        self.cells.len() * std::mem::size_of::<CellBiota>()
            + self.prev_cover.len() * std::mem::size_of::<[f32; ROSTER]>()
            + self.out.len() * std::mem::size_of::<Outcome>()
    }

    /// One epoch of the six biotic processes over the whole grid, reading the
    /// post-erosion surface. Returns the total organic mass added to `H` this
    /// epoch (the external biotic-carbon input for the mass ledger).
    pub fn step(&mut self, grid: &mut DeepGrid, ero: &Erosion, epoch: u32) -> f64 {
        // Freeze previous cover for the dispersal kernel.
        for (dst, c) in self.prev_cover.iter_mut().zip(&self.cells) {
            *dst = c.cover;
        }

        // ---- pass boundary: materialize the water table ---------------------
        // Once per epoch, before any cell is stepped — never inside the loop.
        // The heir is a field solved over the drainage network, so it gets the
        // network; the identity leaves `self.wet` empty and the per-cell
        // accessor falls through to the pre-seam expression.
        let providers = self.providers;
        providers.depth_to_water(
            WaterPass {
                w: grid.w,
                epoch,
                precip: &grid.precip,
                r: &grid.r,
                h: &grid.h,
                area: ero.area(),
                recv: ero.recv(),
                filled: ero.filled(),
            },
            &mut self.wet,
        );

        // Pure per-cell compute (parallel-safe: reads frozen/own state only).
        let (w, seed) = (self.w, self.seed);
        let cells = &self.cells;
        let prev = &self.prev_cover;
        let area = ero.area();
        let parent_p = &self.parent_p;
        let wet = &self.wet;
        let compute = |i: usize| -> Outcome {
            step_cell(i, w, seed, epoch, cells, prev, grid, area, parent_p, wet)
        };
        if self.parallel && self.n >= (1 << 15) {
            use rayon::prelude::*;
            self.out
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, o)| *o = compute(i));
        } else {
            for i in 0..self.n {
                self.out[i] = compute(i);
            }
        }

        // Sequential apply: update state, deposit into the record, sum the
        // ledger — a fixed 0..n order so scalar and parallel agree to the bit.
        // Organic units carry the same tectonic chapter the erosion recorder is
        // stamping this epoch (0 when tectonic history is off — byte-identical).
        let chapter = ero.current_chapter();
        let mut bio_input = 0.0f64;
        for i in 0..self.n {
            let o = self.out[i];
            self.cells[i] = o.cell;
            grid.bio_weather[i] = o.weather;
            grid.bio_resist[i] = o.resist;
            if o.org_deposit > 0.0 {
                grid.h[i] += o.org_deposit;
                // Pedogenesis OVERPRINTS the surface material rather than
                // stacking a lamina: a stable surface becomes one thick horizon.
                grid.strata[i].overprint_top(o.org_tag, o.org_deposit, chapter);
                bio_input += o.org_deposit;
            }
            if o.charcoal > 0.0 {
                grid.h[i] += o.charcoal;
                grid.strata[i].deposit(o.char_tag, o.charcoal, chapter);
                bio_input += o.charcoal;
            }
        }
        bio_input
    }

    /// Finalize: promote **buried** peat to coal across the whole grid (burial
    /// diagenesis — the control is the temperature each unit has seen, read from
    /// the geotherm's `temperature` field at the burial depth; see
    /// [`COAL_ONSET_C`] and [`super::geotherm`]). Preserves `sum(units) == H`
    /// (only tags change).
    ///
    /// The per-column boundary conditions — where the column is, how warm its
    /// surface is, and the geothermal gradient there — are assembled here: the
    /// surface temperature is the geotherm's upper boundary condition (the record
    /// does not know it), and the gradient is the `temperature` field the geotherm
    /// pass planted on the grid. The row latitudes are lifted out first: `lat_deg`
    /// takes `&grid` and the loop holds `&mut grid.strata`, so a `w`-long vector
    /// sidesteps the borrow.
    pub fn finalize(&self, grid: &mut DeepGrid) {
        let lat: Vec<f64> = (0..grid.w).map(|gy| grid.lat_deg(gy)).collect();
        let DeepGrid {
            ref r,
            ref h,
            ref geotherm,
            ref mut strata,
            ..
        } = *grid;
        let w = self.w;
        for (index, s) in strata.iter_mut().enumerate() {
            let (gx, gy) = (index % w, index / w);
            let surface_temp_c =
                f64::from(super::climate::air_temp_c(lat[gy], r[index] + h[index]));
            // The geotherm's gradient at this column (the `temperature` field).
            // Off the tectonic path the field is empty — fall back to a
            // continental average, so coal still forms on a non-tectonic world.
            let gradient_c_per_m = geotherm
                .get(index)
                .copied()
                .unwrap_or(geotherm::DEFAULT_CONTINENTAL_GRADIENT_C_PER_M);
            s.promote_coal(
                BurialColumn {
                    index,
                    gx,
                    gy,
                    surface_temp_c,
                    gradient_c_per_m,
                },
                COAL_ONSET_C,
            );
        }
    }
}

/// Air temperature (°C) at a cell: the shared climate model
/// ([`super::climate::air_temp_c`]) — a latitude gradient minus an altitude
/// lapse. Delegated so the biotic suitability gate and the frost erosion agent
/// read one identical temperature (byte-identical to the pre-0034 inline form).
#[inline]
fn temperature(grid: &DeepGrid, gy: usize, surf: f64) -> f32 {
    super::climate::air_temp_c(grid.lat_deg(gy), surf)
}

/// A plateau tolerance term in 0..1: 1 inside `[lo, hi]`, ramping to 0 over a
/// margin outside (Shelford's tolerance range, ecology.md § 0).
#[inline]
fn tol(v: f32, lo: f32, hi: f32, margin: f32) -> f32 {
    if v >= lo && v <= hi {
        1.0
    } else if v < lo {
        (1.0 - (lo - v) / margin).max(0.0)
    } else {
        (1.0 - (v - hi) / margin).max(0.0)
    }
}

/// The six processes for one cell, as a pure function of frozen inputs — see the
/// module docs for the read/write phase discipline.
#[allow(clippy::too_many_arguments)]
fn step_cell(
    i: usize,
    w: usize,
    seed: u64,
    epoch: u32,
    cells: &[CellBiota],
    prev_cover: &[[f32; ROSTER]],
    grid: &DeepGrid,
    area: &[f64],
    // The materialized `parent_p` plane — an indexed read, never a provider
    // call, because parent material does not change inside the epoch loop.
    parent_p: &[f32],
    // The materialized `depth_to_water` plane for THIS epoch — empty under the
    // identity provider, in which case `wet_at` computes the pre-seam proxy.
    wet_plane: &[f32],
) -> Outcome {
    let gx = i % w;
    let gy = i / w;
    let mut cell = cells[i];
    let h_now = grid.h[i];
    let surf = grid.r[i] + h_now;
    let moist = grid.precip[i];

    let mineral_low = DepTag::mineral(
        DepEnv::Subaerial,
        if moist < 0.32 {
            Aridity::Arid
        } else {
            Aridity::Humid
        },
        EnergyBand::Low,
    );

    // What erosion did to this cell since the last biotic step (erosion runs
    // between two biotic steps, so these deltas are purely its work):
    //   d_surf      — net surface change (negative = the site is being stripped)
    //   mineral_dep — this epoch's mineral deposition, which gates soil formation
    let d_surf = if cell.prev_surf.is_nan() {
        0.0
    } else {
        surf - cell.prev_surf
    };
    let mineral_dep = h_now - cell.prev_h;
    cell.prev_surf = surf;
    cell.prev_h = h_now;

    // Subaqueous cells carry no land community: reset to bare, no soil signal.
    if surf <= SEA_LEVEL_M {
        cell.cover = [0.0; ROSTER];
        cell.tsd = cell.tsd.saturating_add(1);
        return Outcome {
            cell,
            weather: 1.0,
            resist: 0.0,
            org_deposit: 0.0,
            org_tag: mineral_low,
            charcoal: 0.0,
            char_tag: mineral_low,
        };
    }

    // Rejuvenation: stripping thins the soil and exposes fresh, phosphorus-
    // bearing rock; delivered sediment brings fresh mineral P too. Stable
    // surfaces get neither — they age into deep, P-starved profiles (the
    // Walker & Syers chronosequence, made geographic).
    //
    // "Fresh" means *this cell's* parent material: the cap is the `parent_p`
    // plane, not a global constant (providers.rs § `parent_p`). Identity
    // provider ⇒ every entry is `P_ROCK_INIT`, so the arithmetic is unchanged.
    let p_rock_max = parent_p[i];
    if d_surf < 0.0 {
        let strip = (-d_surf) as f32;
        cell.soil = (cell.soil - strip * SOIL_STRIP).max(0.0);
        cell.p_rock = (cell.p_rock + strip * P_FRESH).min(p_rock_max);
    } else if mineral_dep > 0.0 {
        cell.p_rock = (cell.p_rock + mineral_dep as f32 * P_FRESH).min(p_rock_max);
    }

    let temp = temperature(grid, gy, surf);

    // **Waterlogging** — distinct from rainfall. A peat swamp needs a high water
    // table, which means a low-lying, poorly-drained site that *collects* water,
    // not merely a rainy one (a wet mountainside sheds its water and grows
    // forest, not peat). This is what puts coal swamps on lowlands.
    //
    // Asked of the `depth_to_water` provider slot, not computed here: the
    // three-term proxy that used to sit inline (climate moisture + a bonus for
    // sitting near base level + a bonus for receiving upslope drainage) is now
    // the slot's *identity*, reached through the empty-plane branch of `wet_at`.
    // The heir is the S11 saturation field — a real water table (providers.rs
    // § `depth_to_water`; water.md DECIDED 2026-07-20, consequence 4).
    let wet = wet_at(wet_plane, i, moist, surf, area[i]);

    // ---- Process 1: Suitability (Liebig min over tolerances) --------------
    let mut suit = [0.0f32; ROSTER];
    for (s, niche) in NICHES.iter().enumerate() {
        let t_temp = tol(temp, niche.temp_lo, niche.temp_hi, 8.0);
        let t_moist = tol(moist, niche.moist_lo, niche.moist_hi, 0.05);
        let t_soil = if niche.soil_min <= 0.0 {
            1.0
        } else {
            (cell.soil / niche.soil_min).min(1.0)
        };
        // Liebig nutrient terms (fixers are N-independent).
        let t_p = (cell.p_avail / niche.p_demand).min(1.0);
        let t_n = if niche.n_fixer {
            1.0
        } else {
            (cell.n / niche.n_demand).min(1.0)
        };
        // Waterlog gate: peat formers need a wet site; others tolerate dry.
        let t_wet = if niche.waterlog > 0.5 {
            ((wet - 0.30) / 0.15).clamp(0.0, 1.0)
        } else {
            1.0
        };
        suit[s] = t_temp.min(t_moist).min(t_soil).min(t_p).min(t_n).min(t_wet);
    }

    // ---- Process 2: Dispersal (bounded neighbour kernel) ------------------
    // Propagule pressure: own prior cover, half the best neighbour's (the
    // bounded kernel that makes ranges *fronts*), and a small background rain
    // from the regional species pool.
    //
    // FLAGGED design choice (ecology.md is silent on world-genesis
    // colonization): without the background term the model deadlocks — a
    // species that is nowhere yet can never be anywhere, so only the pioneers
    // ever exist and every later-successional niche stays empty forever. The
    // background rain says "these species exist in the region"; because it is
    // an order of magnitude weaker than the neighbour term, spread is still
    // neighbour-driven and still produces invasion fronts.
    let mut pp = [0.0f32; ROSTER];
    for s in 0..ROSTER {
        let mut best_nb = 0.0f32;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < w {
                    best_nb = best_nb.max(prev_cover[ny as usize * w + nx as usize][s]);
                }
            }
        }
        let background = if s == 0 || s == 6 {
            PIONEER_PROPAGULE
        } else {
            BACKGROUND_PROPAGULE
        };
        pp[s] = prev_cover[i][s].max(0.5 * best_nb).max(background);
    }

    // ---- Process 3: Competition (finite capacity, incumbency-weighted) ----
    // Dispersal enters as a saturating *gate* (has it arrived in force?), NOT as
    // a competitive magnitude. Otherwise the pioneers' propagule advantage would
    // also make them the equilibrium winners and succession would never
    // proceed — pioneers win the race to arrive; later-successional species win
    // the contest once everyone is present (Connell & Slatyer tolerance).
    let mut comp = [0.0f32; ROSTER];
    let mut comp_sum = 0.0f32;
    for s in 0..ROSTER {
        let gate = (pp[s] / ARRIVE_HALF).min(1.0);
        // Intrinsic competitive rank (shade-casting / stature) × suitability,
        // then incumbency: an established shade-caster resists displacement.
        let c = suit[s]
            * gate
            * (COMP_FLOOR + NICHES[s].shade)
            * (1.0 + NICHES[s].shade * prev_cover[i][s]);
        comp[s] = c;
        comp_sum += c;
    }
    // The site's carrying capacity is set by the BEST suitability present, so a
    // hostile cell supports little cover. (Normalizing shares alone would fill
    // every cell to 100 % cover no matter how unsuitable — competition decides
    // *who*, suitability decides *how much*.)
    let capacity = suit.iter().copied().fold(0.0f32, f32::max);
    let mut total_cover = 0.0f32;
    for (cov, &c) in cell.cover.iter_mut().zip(comp.iter()) {
        let target = if comp_sum > 1e-6 {
            capacity * c / comp_sum
        } else {
            0.0
        };
        *cov += SUCCESSION_RATE * (target - *cov);
        if *cov < 1e-4 {
            *cov = 0.0;
        }
        total_cover += *cov;
    }

    // ---- Process 4: Nutrient cycling (Walker & Syers emerges) -------------
    // Decomposition rate first — it governs both the nutrient return and how
    // much litter escapes into the record. Fast when warm and well-drained,
    // throttled when waterlogged or cold → net organic burial in swamps (peat).
    let temp_factor = ((temp - 2.0) / 22.0).clamp(0.15, 1.0);
    let drain_factor = (1.0 - (wet - 0.30) / 0.40).clamp(0.10, 1.0);
    let decomp_frac = (0.9 * temp_factor * drain_factor).clamp(0.05, 0.98);

    // Rock-P weathering release (vegetation-accelerated), depleting a FINITE
    // pool: the release rate itself decays as the parent material is used up.
    let dp = P_WEATHER_RATE * cell.p_rock * (0.4 + total_cover.min(1.0));
    cell.p_rock -= dp;
    cell.p_avail += dp;
    // N fixation + atmospheric deposition.
    let mut fixer_cover = 0.0f32;
    for (&cov, niche) in cell.cover.iter().zip(NICHES.iter()) {
        if niche.n_fixer {
            fixer_cover += cov;
        }
    }
    cell.n += N_FIX_RATE * fixer_cover + N_DEPOSITION;
    // Uptake into biomass...
    let mut p_uptake = 0.0f32;
    let mut n_uptake = 0.0f32;
    let mut veg_litter = 0.0f32;
    for (&cov, niche) in cell.cover.iter().zip(NICHES.iter()) {
        p_uptake += cov * niche.p_demand;
        n_uptake += cov * niche.n_demand;
        veg_litter += cov * niche.litter;
    }
    let p_taken = (p_uptake * UPTAKE).min(cell.p_avail);
    let n_taken = (n_uptake * UPTAKE).min(cell.n);
    cell.p_avail -= p_taken;
    cell.n -= n_taken;
    // ...and the decomposition return, which CLOSES the cycle: the decomposed
    // fraction of this epoch's uptake comes straight back. Only the fraction
    // that escapes decomposition is lost — buried in the record. That is why a
    // peat bog is nutrient-poor: burial *is* the leak (mechanism, not a knob).
    cell.p_avail += p_taken * decomp_frac;
    cell.n += n_taken * decomp_frac;
    // Leaching (moisture-scaled) for N and cations; P is lost by leaching AND
    // occlusion into unavailable forms, so available P tracks the release rate
    // instead of accumulating — the retrogression driver.
    let leach = LEACH * moist;
    cell.n = (cell.n * (1.0 - leach)).max(0.0);
    cell.cations = (cell.cations * (1.0 - leach)).max(0.0);
    cell.p_avail = (cell.p_avail * (1.0 - P_LOSS)).max(0.0);

    let net_org = f64::from(veg_litter) * f64::from(1.0 - decomp_frac);

    // ---- Retrogression state (Walker & Syers end): a stable, P-starved,
    // developed surface collapses to sclerophyll.
    let retro = cell.p_avail < P_RETRO_THRESH && cell.tsd > RETRO_AGE && cell.soil > 0.2;

    // ---- Process 5: Niche construction (write next-epoch modifiers) -------
    cell.soil = (cell.soil + SOIL_GAIN * net_org as f32).min(SOIL_MAX);
    let mut root = 0.0f32;
    let mut weather_boost = 0.0f32;
    let mut fuel = 0.0f32;
    for (&cov, niche) in cell.cover.iter().zip(NICHES.iter()) {
        root += cov * niche.root;
        weather_boost += cov * niche.weather;
        fuel += cov * niche.flammable;
    }
    let resist = root.clamp(0.0, 0.9);
    let weather = 1.0 + weather_boost; // ≥ 1.0: biology only accelerates it
    // Dryness raises effective fuel (cured biomass burns; a waterlogged site
    // does not, however much biomass it carries).
    let dryness = (1.0 - wet).clamp(0.0, 1.0);
    let eff_fuel = fuel * (0.4 + 0.6 * dryness);

    // Biotic facies of the organic deposit — and whether a horizon can form at
    // all. A soil profile needs a depositional **hiatus**: where mineral
    // sediment is piling on faster than `SOIL_HIATUS_MAX`, litter is diluted
    // into the mineral unit and no organic horizon is recorded. A swamp keeps
    // pace with somewhat faster aggradation (peat outgrows the mud).
    let peat_site = cell.cover[4] > 0.15 || (wet > 0.42 && net_org > 0.004);
    let hiatus_cap = if peat_site {
        PEAT_HIATUS_MAX
    } else {
        SOIL_HIATUS_MAX
    };
    let can_form_horizon = mineral_dep <= hiatus_cap;
    let biota = if net_org < ORG_MIN_DEPOSIT || !can_form_horizon {
        Biofacies::Mineral
    } else if peat_site {
        Biofacies::Peat
    } else if retro {
        Biofacies::Retro
    } else {
        Biofacies::Soil
    };
    let org_tag = DepTag {
        biota,
        ..mineral_low
    };

    // ---- Process 6: Disturbance (fire, flood) -----------------------------
    let mut charcoal = 0.0f64;
    let char_tag = DepTag {
        biota: Biofacies::Charcoal,
        ..mineral_low
    };
    cell.tsd = cell.tsd.saturating_add(1);

    // Fire: deterministic addressed ignition, likelier with more (dry) fuel.
    if eff_fuel > FUEL_MIN {
        let ignition_p = f64::from((eff_fuel - FUEL_MIN) * (0.3 + 0.7 * dryness)) * IGNITION_SCALE;
        let roll = draw_f64(&[seed, SALT_BIO_FIRE, gx as u64, gy as u64, u64::from(epoch)]);
        if roll < ignition_p {
            // Standing organic proxy = soil organic near the surface; a fraction
            // chars into the record, the rest is lost to the atmosphere.
            let residue = f64::from(cell.soil) * FIRE_CHAR_FRAC;
            // Only a substantial residue survives as its own bed; thinner
            // charcoal is dispersed into the soil and leaves no unit.
            charcoal = if residue >= CHAR_MIN_BAND {
                residue
            } else {
                0.0
            };
            // Burn cover; fire-adapted species (5 sclerophyll, 6 fire-grass)
            // resist. Releases a nutrient pulse (ash).
            for s in 0..ROSTER {
                let kill = fire_kill(s);
                cell.cover[s] *= 1.0 - kill;
            }
            cell.n += 0.01;
            cell.p_avail += 0.004;
            cell.soil *= 0.7;
            cell.tsd = 0;
        }
    }

    // Flood: valley cells with large drainage area get bioturbated / reset.
    if area[i] > FLOOD_AREA {
        let roll = draw_f64(&[seed, SALT_BIO_FLOOD, gx as u64, gy as u64, u64::from(epoch)]);
        if roll < 0.15 {
            for s in 0..ROSTER {
                cell.cover[s] *= 0.6;
            }
            cell.tsd = 0;
        }
    }

    let org_deposit = if biota == Biofacies::Mineral {
        0.0
    } else {
        net_org
    };

    Outcome {
        cell,
        weather,
        resist,
        org_deposit,
        org_tag,
        charcoal,
        char_tag,
    }
}

/// Fire kill fraction for a species: fire-adapted species (sclerophyll,
/// fire-grass) resist; others burn near-completely.
#[inline]
fn fire_kill(s: usize) -> f32 {
    match s {
        5 => FIRE_BASE_KILL * 0.3, // sclerophyll: fire-adapted
        6 => FIRE_BASE_KILL * 0.2, // fire-grass: resprouts fast
        _ => FIRE_BASE_KILL,
    }
}

/// The vanilla roster's species names (for spike diagnostics / column dumps).
pub fn species_name(s: usize) -> &'static str {
    NICHES.get(s).map_or("?", |ni| ni.name)
}
