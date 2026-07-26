//! The deep-time grid: a square, equal-area cell lattice covering the pregen
//! civilized extent at an arbitrary (finer) resolution, plus the initial
//! bedrock/uplift fields derived from the pregen tectonics by bilinear
//! resampling.
//!
//! Equal-area cells are load-bearing for the mass-conserving sediment router:
//! a metre of thickness leaving one cell arrives as a metre at its receiver, so
//! volume conservation reduces to thickness bookkeeping (erosion.rs).
//!
//! Everything is a pure function of `(seed, config, pregen)`: the only entropy
//! is an addressed roughness jitter on the initial bedrock (a new SALT), so the
//! deep-time run is deterministic and seed-sensitive like the rest of worldgen.

use dc_sim::statistical::rng::draw_f64;

use crate::pregen::{
    CELL_VOXELS, CellGrid, LAT_NORTH, LAT_SOUTH, Pregen, Provenance, provenance_roughness,
};

use super::recorder::DeepStrata;

/// Addressed-draw salt for the deep-time initial-bedrock roughness jitter.
/// Distinct high byte from the pregen salts (`0x5700_*`) so the address spaces
/// never collide.
pub(crate) const SALT_DT_ROUGH: u64 = 0x5900_0001;

/// Sea level, metres. Matches pregen (`elev_m > 0` is land).
pub const SEA_LEVEL_M: f64 = 0.0;

/// Physics + run knobs for a deep-time erosion run. Defaults are calibrated
/// (see `docs/spikes/S9-results.md`) so orogenic belts build hundreds of metres
/// of net relief against erosion over a few hundred iterations.
#[derive(Clone, Copy, Debug)]
pub struct DeepConfig {
    pub seed: u64,
    /// Deep-time cell edge, metres.
    pub cell_m: f64,
    /// Fixed iteration count (deterministic; no convergence check in the sim
    /// logic — a timing harness may wrap `Instant` around, never inside).
    pub iterations: u32,
    /// Re-march orographic precipitation every this many iterations.
    pub remarch_interval: u32,
    /// Populate the strata recorder (read-quality). Off = erosion-only, the
    /// cheaper B datapoint.
    pub record: bool,
    /// Run the **S10 biotic layer** (community vector + the six processes,
    /// `deeptime::biotic`): organic/charcoal/paleosol/retrogression annotations
    /// in the record, plus root-cohesion and biological-weathering feedback into
    /// erosion. Off by default — with it off, every code path is byte-identical
    /// to the pre-S10 engine (the modifier planes stay empty and read as the
    /// identity `1.0` / `0.0`). Requires `record` for the strata annotations.
    pub biotic: bool,
    /// Run the **erodibility coupling** (`deeptime::lithology`): erosion rates
    /// modulated per cell per epoch by the *agent-specific* resistance of the
    /// lithology outcropping there, instead of one global `k_bedrock` for every
    /// rock in the world. Off by default — with it off every code path is
    /// byte-identical to the uncoupled engine (the susceptibility planes stay
    /// empty and read as the identity `1.0`, and `x * 1.0 == x` exactly).
    ///
    /// Turning this on changes `DeepField` — and therefore terrain shape — for
    /// every world created afterwards, the same class of event as the S10
    /// biotic flip.
    pub erodibility: bool,
    /// **Erodibility contrast** — the exponent applied to the resistance ratio
    /// (`(reference / resistance) ^ contrast`). The property sheet's smash
    /// resistances span only ~3× because they are calibrated for tool time,
    /// while real erodibility spans orders of magnitude; this is the knob that
    /// re-expands that range to landform scale. `1.0` = take the sheet
    /// literally, `0.0` = no coupling at all (every multiplier 1.0).
    pub erodibility_contrast: f64,
    /// **Erodibility contrast for hillslope diffusion**, separately knobbed and
    /// deliberately weaker than the fluvial one. Creep acts on regolith, whose
    /// mobility is governed by root cohesion (the S10 `resist` term) and
    /// moisture far more than by the competence of the parent rock — but a
    /// competent bed does armour its own slope with coarse talus, so the
    /// coupling is real, just softer.
    pub erodibility_diffusion_contrast: f64,
    /// **Stability bound** on the feedback: no cell's erosion rate may exceed
    /// `erodibility_max ×` or fall below `1/erodibility_max ×` the reference
    /// rate, whatever the contrast knob says. Differential erosion is
    /// self-reinforcing (erode soft → expose hard → slow down), which is the
    /// mechanism that carves benches *and* the mechanism that could stall a
    /// cell forever; this clamp is what keeps the feedback bounded. It also
    /// bounds the frost/wave/wind agents' susceptibilities (they share this
    /// `susceptibility_table` clamp — journal/0034).
    pub erodibility_max: f64,
    /// **The full erosion-agent roster** (wind + frost + wave, journal/0034):
    /// activates eolian deflation/deposition (a fifth [`super::lithology::Agent`]),
    /// the temperature-gated frost weathering multiplier, and littoral wave
    /// attack at the current sea-level stand. Off by default — with it off every
    /// added code path is skipped and the run is **byte-identical** to the
    /// pre-0034 engine; and even *on* with the three rate knobs below at zero it
    /// is byte-identical (the strong off-path proof, the 0029/S10 pattern). The
    /// three agents compose with the lithology resistance system exactly as
    /// abrasion does (each reads its own agent axis), and each is bounded by the
    /// same stability clamp. Turning this on changes `DeepField` — and therefore
    /// terrain shape — for every world created afterwards, the same class of
    /// event as the erodibility flip. **The production flip is the user's.**
    pub full_agents: bool,
    /// **Wind: base deflation** (metres of loose cover entrained per epoch) in a
    /// maximally arid, unvegetated cell before the agent susceptibility and the
    /// aridity/vegetation gates scale it down. Wind only redistributes loose `H`
    /// (never bedrock), so it is mass-neutral on the ledger. `0.0` disables the
    /// term while the flag is on (byte-identity proof).
    pub eolian_deflation: f64,
    /// **Wind: aridity threshold** — normalized precipitation below which a cell
    /// is a deflation source (the driest cells deflate most). Matches the
    /// recorder's [`super::recorder::Aridity::Arid`] cutoff so the wind agent and
    /// the facies tag agree about where the desert is.
    pub eolian_arid_precip: f64,
    /// **Wind: settling fraction** — the fraction of the airborne load a
    /// vegetated or humid downwind cell traps as loess/dune per epoch. The
    /// desert interior passes dust through; the margin catches it.
    pub eolian_deposit_frac: f64,
    /// **Frost: peak weathering gain** — the extra bedrock→regolith weathering
    /// multiplier a maximally frost-susceptible rock receives at the centre of
    /// the freeze–thaw band (`0°C`). Freeze–thaw is *not* monotonic with cold:
    /// it is maximal where water repeatedly crosses the phase boundary, so the
    /// multiplier peaks at `0°C` and falls to `1.0` outside the band. `0.0`
    /// disables the term while the flag is on.
    pub frost_weathering_gain: f64,
    /// **Frost: band half-width** (°C) about `0°C` over which freeze–thaw
    /// enhancement ramps from its peak to nothing. A wide band lets high summits
    /// and cold high-latitude ground share the periglacial signature.
    pub frost_band_width_c: f64,
    /// **Wave: base littoral erosion** (metres per epoch) cut at a shoreline cell
    /// right at sea level, before the wave-agent susceptibility and the freeboard
    /// taper. The cut is clamped so a cell can never be lowered below the current
    /// sea stand (a wave-cut platform forms *at* sea level, it does not dig a
    /// hole). `0.0` disables the term while the flag is on.
    pub wave_erosion: f64,
    /// **Wave: reach band** (metres of freeboard above the current sea stand)
    /// within which a shore cell adjacent to open water is attacked. Because the
    /// sea-level curve cycles, the band sweeps up and down the coast over the run,
    /// which is what records raised and drowned wave-cut features (§ 6).
    pub wave_band_m: f64,
    /// Uplift rate scale, metres/iteration for a unit-rate (orogenic) province.
    pub uplift_scale: f64,
    /// Stream transport coefficient (capacity `= k_t · A^m · S^n`).
    pub k_transport: f64,
    /// Bedrock incision coefficient (shielded by alluvial cover).
    pub k_bedrock: f64,
    /// Drainage-area exponent `m`.
    pub m_exp: f64,
    /// Slope exponent `n`.
    pub n_exp: f64,
    /// Alluvial-cover shielding scale `H*` (metres): bedrock incision is
    /// multiplied by `exp(-H / H*)`.
    pub h_star: f64,
    /// Hillslope diffusivity (dimensionless per iteration on the surface).
    pub diffusion: f64,
    /// Bedrock→regolith weathering rate (metres/iteration), tapered by cover.
    pub weathering: f64,
    /// Initial-bedrock roughness jitter as a fraction of provenance roughness.
    pub rough_jitter: f64,
    /// Paleo-sea-level oscillation amplitude (metres) about [`SEA_LEVEL_M`] — a
    /// deterministic transgression/regression curve (earth-processes.md § 6).
    /// Zero for a static shoreline (the clean decay experiment).
    pub sea_level_amp: f64,
    /// Sea-level cycle period (iterations).
    pub sea_level_period: u32,

    // ---- Tectonic history (tectonics.md, RATIFIED 2026-07-20) --------------
    /// **The tectonic-history flag** (§ 12). Off by default — with it off every
    /// added path is skipped, no column/forcing planes are allocated, the
    /// isostasy phase no-ops, `provenance_uplift` drives the run exactly as
    /// today, and every existing world reproduces **byte-identically**. On: the
    /// whole bundle (chapters + analytic forcing + crustal columns + smoothed
    /// Airy isostasy + chapter-stamped record + drainage export). The items do
    /// not flip separately (thickening without isostasy is meaningless; chapters
    /// without analytic forcing re-smear). Turning it on CHANGES TERRAIN SHAPE
    /// for every world made afterwards — the production flip is the user's
    /// appearance call (U8), like the erodibility/biotic flips.
    pub tectonic_history: bool,
    /// **Characteristic plate diameter, km** (U1 / § 10, option B, default 65):
    /// plate count is derived from area and this knob, so province density is
    /// extent-uniform (fixing the found defect that the old `clamp(24)` made
    /// Large worlds ~3× sparser per km than Medium).
    pub plate_scale_km: f64,
    /// **Chapter count K** (U2 / § 3.1, default 8): the run divides into K equal
    /// chapters; each advects the plates, re-classifies boundaries, and repaints
    /// the thickening forcing. K=8 gives Earth-orogeny-length chapters (62.5 Myr)
    /// with 2–4 legible superpositions per place at the Phanerozoic register.
    pub chapters: u32,
    /// **Ramp chapter transitions** (§ 3.1.4, default true): the active forcing
    /// is a linear blend from the previous chapter's plane to the new one across
    /// the chapter, so a transverse river saws through a rising axis (the water-
    /// gap mechanism) instead of being dammed. `false` = step the forcing at the
    /// boundary — the negative control that must kill the water gaps (§ SPIKE
    /// 4a).
    pub ramp_chapters: bool,
    /// **Orogen half-width W, km** (U4 / § 4.1, default 25): the continent–
    /// continent belt half-width, scaled by `1/√v_conv` so fast convergence gives
    /// a narrow sharp belt. A first-class design parameter *in km*, not a ring
    /// count — the 50 km gradation-to-peak artifact must collapse to ≈ this value
    /// (§ SPIKE 5).
    pub orogen_width_km: f64,
    /// **Arc–trench gap, km** (U4 / § 4.1, default 50): the offset of the volcanic
    /// arc onto the overriding side. Real gaps are ~100–250 km; ours compress with
    /// the world.
    pub arc_gap_km: f64,
    /// **Total advection over the run, in plate widths** (U5 / § 3.2, default 1):
    /// how far boundaries migrate over a world's life. Earth-true speed is
    /// impossible at our extent and undesirable at any (the map churns to noise);
    /// what the record needs is boundaries moving *relative to columns*, ~1 plate
    /// width per run.
    pub advection_plate_widths: f64,
    /// **Thickening scale, m/iter** for a unit-rate (fast) orogenic boundary
    /// (§ 4.2). The amplitude the analytic forcing multiplies; the amplitude call
    /// (dismal-mountains cause 3, U7) is made against *this* forcing with § SPIKE
    /// 7 data, not before it.
    pub thickening_scale: f64,
    /// **Flexural wavelength Λ_flex, km** (§ 6.1, default 50): the smoothing
    /// half-width of the isostatic load. Smoothing the compensation *is* plate
    /// rigidity. Resolution-independent because it is stated in km.
    pub flex_wavelength_km: f64,
    /// **Isostatic relax rate λ_iso** (§ 6.1, default 0.5): the fraction of the
    /// gap to Airy equilibrium the bedrock closes per iteration. Mantle response
    /// at 2.5 Myr/iter is effectively instant, so this is a numerical-stability
    /// choice, measured not guessed (§ SPIKE 1/8).
    pub iso_rate: f64,
    /// **The resolved provider set** ([`super::providers::Providers`]) — the
    /// slots where an unbuilt system will one day answer a question this sim
    /// currently answers with a constant (outcropping lithology, littoral wave
    /// energy, parent-material phosphorus).
    ///
    /// Fixed at world creation and carried here so it reaches the run through the
    /// same `production_config_with` → `build_field_with` → `PregenCtx` path the
    /// flag overrides use. `Default` is the **identity set**: bit-for-bit the
    /// pre-seam world (`tests/providers.rs`, goldens from pre-slice `main`).
    ///
    /// Generation-affecting, therefore frozen at world creation (ARCHITECTURE.md
    /// § *The content set is frozen at world creation*, DECIDED 2026-07-22).
    ///
    /// Appended last (wire discipline, corrections #3) and `Copy`, so every
    /// `..DeepConfig::default()` literal in the tree keeps working untouched.
    pub providers: super::providers::Providers,

    /// **The inventory-weathering flag** (the first-real-behavior slice,
    /// material-behavior.md §4/§11; **journal/0094 made it a per-epoch process**). Off
    /// by default — with it off, the `dc:deep/weather_inventory` pass is **absent**, no
    /// per-cell [`FactLedger`](super::inventory::FactLedger) is built, and the collapsed
    /// world is **byte-identical** (the S-5 identity default). On: the pass runs
    /// **inside the deep-time loop, every epoch**, weathering each subaerial cell's
    /// bedrock `Structure` seam into `Loose` saprolite on that epoch's live terrain and
    /// **accumulating** the cause-carrying facts across the whole run; the resulting
    /// ledger sidecar on the `DeepField` is folded by the collapse into a basal
    /// weathering-front band (now ≥1 voxel).
    ///
    /// This is a **material-transformation** authority that runs *alongside* the
    /// existing `erosion.rs` `R`/`H` height weathering, which is left in place: the
    /// pass READS the contemporaneous regolith cover but WRITES ONLY the ledger — it
    /// never touches `R`/`H` (the two-authorities split, material-behavior.md §11
    /// continuation slot). Turning it on is a **walk-gated appearance** change for
    /// every world made afterwards — the production flip is the user's, like the
    /// erodibility/biotic/tectonic-history flips. Appended last (wire discipline).
    pub weather_inventory: bool,

    /// **The flow record** (FLOW slice 1, `docs/design/flow.md` § 2 — RATIFIED
    /// 2026-07-25). **On by default.** With it on, the `dc:deep/flow_record` pass
    /// runs every epoch and accumulates the routed discharge into **flux on
    /// faces, per tectonic chapter** ([`super::flux::FluxRecord`]) — the
    /// representation that replaces the exported receiver tree, which is one
    /// out-edge per cell and therefore **cannot represent divergence at all**
    /// (no distributaries, braids, fans or deltas).
    ///
    /// It is a **pure sidecar**: it reads the drainage solve's own outputs and
    /// writes only its own record, never `R`/`H`/the strata — the same
    /// two-authorities discipline the geotherm and the inventory-weathering pass
    /// keep. So the collapsed world is **byte-identical with the flag either
    /// way** (asserted by name in `tests/flux_record.rs`, and the production
    /// goldens in `providers_golden.rs` are untouched). Off is therefore not an
    /// identity-preserving *fallback* — it is the **residency** switch: the
    /// record is the largest thing the ritual keeps, and flow.md § 9.1 explicitly
    /// defers "what is resident vs re-derived" to measurement. Turn it off to run
    /// the ritual at the pre-slice footprint. Appended last (wire discipline).
    pub flow_record: bool,

    /// **The head field** (FLOW continuation (a), `docs/design/flow.md` § 2.4 —
    /// RATIFIED 2026-07-25). **On by default.** With it on, the `dc:deep/head`
    /// **field pass** relaxes the `head` condition-field (`dc:field/head`) — the
    /// **potential** flow descends, `elevation + pressure head` — and its
    /// vertical-exchange plane, from the live topography and the live strata
    /// record. Its first consumer is the flux record's **vertical (slot↔slot)
    /// faces**, which FLOW slice 1 left structurally present and honestly **zero**
    /// because the surface solve had no vertical term.
    ///
    /// Like the geotherm and the flow record it is a **pure sidecar**: it plants a
    /// field and runs no edges, never touching `R`/`H`/the strata, so the collapsed
    /// world is **byte-identical with the flag either way** (asserted by name in
    /// `tests/head_field.rs`; the production goldens are untouched). It does **not**
    /// change lateral routing — the drainage solve stays
    /// steepest-descent-on-filled-elevation, and a multi-flow-direction partition
    /// (which head is what unlocks — flow.md § 2.6) is deliberately the NEXT slice.
    ///
    /// Off is therefore not an identity-preserving *fallback* but the **cost**
    /// switch: it buys the head plane plus the vertical flux entries in residency,
    /// and its relaxation sweeps in gen time. Appended last (wire discipline).
    pub head_field: bool,

    /// **Multiple-flow-direction routing** (FLOW continuation (b),
    /// `docs/design/flow.md` § 2.6 — the MFD *solve* change). **On by default.**
    ///
    /// With it **off**, the drainage solve routes each cell's whole discharge to
    /// its single steepest-descent D8 receiver: within one epoch a cell has
    /// **exactly one** out-face, so *simultaneous* divergence — a delta with two
    /// channels flowing at once — is **structurally impossible**, and every
    /// divergence in the flux record is **temporal** (avulsion across the chapter's
    /// 25 epochs). With it **on**, each cell partitions its discharge across every
    /// downslope neighbour on the free-surface potential, weighted by
    /// [`DeepConfig::mfd_exponent`] — so concurrent distributaries become
    /// representable *and* actually occur.
    ///
    /// Unlike the flow record and the head field this is **not** a sidecar: it
    /// changes what the erosion pass moves, so the world moves with it. Off is the
    /// **pre-MFD identity path**, byte-identical to the single-receiver solve
    /// (asserted by name in `tests/mfd_routing.rs`). Appended last (wire discipline).
    pub mfd: bool,

    /// **The MFD convergence exponent `p`** (Holmgren 1994), read only when
    /// [`DeepConfig::mfd`] is on. Each downslope neighbour `k` receives a share
    /// `w_k ∝ S_k^p · L_k`, where `S_k` is the free-surface potential gradient
    /// along the true flow-path length and `L_k` the face's contour width (Quinn
    /// 1991: `1` cardinal, `1/√2` diagonal).
    ///
    /// **`p` is the degree to which flow concentrates.** `p = 1` is the maximally
    /// dispersive Quinn form; `p → ∞` recovers single-receiver D8 exactly. The
    /// default `4.0` sits in Holmgren's calibrated 4–6 band and has the property
    /// the landscape needs: on steep ground a 2:1 slope ratio becomes a 16:1 share
    /// ratio, so **gorges stay gorges**, while on the low-relief surfaces where
    /// distributaries physically live — fans, braid plains, delta tops — the
    /// near-equal slopes genuinely split. Appended last (wire discipline).
    pub mfd_exponent: f64,

    /// **Material-aware transport** (Movement 2b first slice, `material-behavior.md`
    /// § 13.3–13.6 — "identity travels + sorted deposition"). **On by default.**
    ///
    /// With it **off**, the transport pass carries a scalar mass: a cell entrains
    /// `min(H, room)` metres of *something*, hands it downstream, and whatever
    /// exceeds the next cell's capacity is deposited as *something else*. Nothing
    /// about the material survives the journey, so the rock a unit is made of has
    /// to be **inferred from the environment** at the receiver
    /// ([`litho_of_tag`](super::lithology::litho_of_tag)).
    ///
    /// With it **on**, the load is a multiset of `(lithology, quantity)` — a
    /// suspended inventory riding the chain — sorted by **settling velocity**
    /// ([`settling_table`](super::lithology::settling_table)). Deposition is the
    /// **falling competence ceiling**: a species whose settling velocity exceeds
    /// what the cell's energy can hold rains out, coarsest first, and the fines
    /// ride on. So a distal cell deposits mud not because it is a low-energy place
    /// but because *there is no gravel left in the water*, which is the difference
    /// between a facies model and a facies **consequence**.
    ///
    /// Like [`DeepConfig::mfd`] and unlike the record/head sidecars, this is **not**
    /// a sidecar: the competence ceiling changes where mass is set down, so the
    /// world moves with it. Off is the pre-2b identity path, byte-identical to the
    /// scalar solve (asserted by name in `tests/material_transport.rs`). Appended
    /// last (wire discipline).
    pub material_transport: bool,
}

/// The paleo-sea-level stand at iteration `it`: a deterministic sinusoid about
/// the mean, zero at `it == 0`. No wall clock, pure in `(cfg, it)`.
pub fn sea_level_at(cfg: &DeepConfig, it: u32) -> f64 {
    if cfg.sea_level_amp == 0.0 || cfg.sea_level_period == 0 {
        return SEA_LEVEL_M;
    }
    let phase = std::f64::consts::TAU * f64::from(it) / f64::from(cfg.sea_level_period);
    SEA_LEVEL_M + cfg.sea_level_amp * phase.sin()
}

impl Default for DeepConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            cell_m: 500.0,
            iterations: 200,
            remarch_interval: 20,
            record: true,
            biotic: false,
            erodibility: false,
            erodibility_contrast: 2.5,
            erodibility_diffusion_contrast: 1.0,
            erodibility_max: 5.0,
            full_agents: false,
            eolian_deflation: 0.02,
            eolian_arid_precip: 0.32,
            eolian_deposit_frac: 0.25,
            frost_weathering_gain: 1.5,
            frost_band_width_c: 12.0,
            wave_erosion: 0.05,
            wave_band_m: 30.0,
            uplift_scale: 3.0,
            k_transport: 0.0016,
            k_bedrock: 0.0011,
            m_exp: 0.5,
            n_exp: 1.0,
            h_star: 3.0,
            diffusion: 0.12,
            weathering: 0.02,
            rough_jitter: 0.18,
            sea_level_amp: 35.0,
            sea_level_period: 50,
            tectonic_history: false,
            plate_scale_km: 65.0,
            chapters: 8,
            ramp_chapters: true,
            orogen_width_km: 25.0,
            arc_gap_km: 50.0,
            advection_plate_widths: 1.0,
            thickening_scale: 80.0,
            flex_wavelength_km: 50.0,
            iso_rate: 0.5,
            weather_inventory: false,
            flow_record: true,
            head_field: true,
            mfd: true,
            mfd_exponent: 4.0,
            material_transport: true,
            providers: super::providers::Providers::default(),
        }
    }
}

/// Relative uplift rate by tectonic provenance. Positive = uplift (orogenic
/// belts fastest); negative = subsidence (trenches, ocean floor). Multiplied by
/// [`DeepConfig::uplift_scale`] to get metres/iteration.
pub fn provenance_uplift(p: Provenance) -> f64 {
    match p {
        Provenance::Orogeny => 1.0,
        Provenance::Arc => 0.6,
        Provenance::Rift => 0.18,
        Provenance::Transform => 0.22,
        Provenance::Craton => 0.03,
        Provenance::Shelf => 0.0,
        Provenance::Ridge => -0.02,
        Provenance::Trench => -0.12,
        Provenance::OceanFloor => -0.05,
    }
}

/// The deep-time state grid (square, `w × w` equal-area cells).
pub struct DeepGrid {
    pub w: usize,
    pub cell_m: f64,
    /// Bedrock top elevation (metres).
    pub r: Vec<f64>,
    /// Alluvium thickness (metres, ≥ 0).
    pub h: Vec<f64>,
    /// Uplift rate (metres/iteration, from pregen tectonics).
    pub uplift: Vec<f64>,
    /// Marched precipitation (normalized 0..1).
    pub precip: Vec<f32>,
    /// Per-cell strata record (empty when `record` is off).
    pub strata: Vec<DeepStrata>,
    /// **Biotic weathering multiplier** per cell (S10). Empty when the biotic
    /// layer is off, and the erosion weathering phase then treats it as a
    /// uniform `1.0` — byte-identical to the pre-S10 path. When biology runs it
    /// carries root-acid / mycorrhizal weathering acceleration (ecology.md § 1),
    /// written each epoch for the *next* step (lagged coupling, ecology.md § 3).
    pub bio_weather: Vec<f32>,
    /// **Biotic hillslope resistance** per cell (S10), `0..1`: root cohesion
    /// suppresses regolith creep (ecology.md § 1). Empty when off; diffusion then
    /// reads a uniform `0.0` resistance — byte-identical to the pre-S10 path.
    pub bio_resist: Vec<f32>,
    /// **Crustal thickness (m)** per cell — the conserved stock under the
    /// tectonic-history inversion (§ 5.1). Continental ~35 km, oceanic ~7 km.
    /// Empty (and the whole isostasy/thickening machinery skipped) when
    /// [`DeepConfig::tectonic_history`] is off.
    pub t_crust: Vec<f64>,
    /// **Crust kind index** per cell ([`super::tectonics::CrustKind`]): sets the
    /// Airy density. Empty when tectonic history is off.
    pub crust_kind: Vec<u8>,
    /// **Cumulative bedrock exhumed (m)** per cell — Σ of R-lowering by incision
    /// and weathering (§ 5.2). The metamorphic-grade input for the collapse tier
    /// (§ 6.4). Empty when tectonic history is off.
    pub exhum: Vec<f64>,
    /// **The `temperature` condition-field** (`dc:field/temperature`, §14): the
    /// per-cell **geothermal gradient** (°C/m) the geotherm field pass
    /// ([`super::geotherm`]) plants, so `T(depth) = surface_T + gradient·depth`.
    /// Empty when tectonic history is off (no crustal state to solve over — coal
    /// then reads [`super::geotherm::DEFAULT_CONTINENTAL_GRADIENT_C_PER_M`]).
    pub geotherm: Vec<f64>,
    /// **The `head` condition-field** (`dc:field/head`, §14; flow.md § 2.4): the
    /// per-cell **hydraulic potential** (metres) the head field pass
    /// ([`super::head`]) relaxes — *elevation + pressure head*, not an elevation.
    /// Where a confining bed caps a permeable one this stands **above** the local
    /// ground, which is artesian and which the unconfined `H = y + sat` proxy
    /// cannot express. Empty when
    /// [`DeepConfig::head_field`] is off (the pass is absent) ⇒ byte-identical.
    pub head: Vec<f64>,
    /// **The head field's vertical-exchange plane**: the signed per-epoch rate
    /// across each column's top — **positive = down** (infiltration/recharge),
    /// **negative = up** (artesian rise). In the flux record's own currency (one
    /// unit = one cell-epoch of the seeded source; see [`super::head`]).
    ///
    /// A **cache of [`super::head::vertical_exchange`], never an authority**: it is
    /// recomputed with the field at the field's own cadence and a test re-derives
    /// it from `{head, ground, record}` and demands equality. It is gen-time only
    /// — the `DeepField` keeps [`Self::head`] (the field) and the flux record (the
    /// consequence), not this. Empty when the head field is off.
    pub head_exchange: Vec<f32>,
    /// South→north latitude span the grid is compressed onto (pregen bands).
    lat_south: f64,
    lat_north: f64,
}

impl DeepGrid {
    /// Assemble a grid from prebuilt bedrock + uplift planes (used by the
    /// regional-refinement builder, refine.rs). Alluvium starts at zero,
    /// precipitation at the over-ocean nominal, and the recorder is off (the
    /// decay experiment is erosion-only).
    pub fn from_parts(w: usize, cell_m: f64, r: Vec<f64>, uplift: Vec<f64>) -> Self {
        let n = w * w;
        debug_assert_eq!(r.len(), n);
        debug_assert_eq!(uplift.len(), n);
        Self {
            w,
            cell_m,
            r,
            h: vec![0.0f64; n],
            uplift,
            precip: vec![0.6f32; n],
            strata: Vec::new(),
            bio_weather: Vec::new(),
            bio_resist: Vec::new(),
            t_crust: Vec::new(),
            crust_kind: Vec::new(),
            exhum: Vec::new(),
            geotherm: Vec::new(),
            head: Vec::new(),
            head_exchange: Vec::new(),
            lat_south: LAT_SOUTH,
            lat_north: LAT_NORTH,
        }
    }

    /// Surface elevation `R + H` of cell `i`.
    #[inline]
    pub fn surf_at(&self, i: usize) -> f64 {
        self.r[i] + self.h[i]
    }

    /// Latitude (degrees) of grid row `gy`, compressed onto the pregen band.
    #[inline]
    pub fn lat_deg(&self, gy: usize) -> f64 {
        let f = (gy as f64 + 0.5) / self.w as f64;
        self.lat_south + (self.lat_north - self.lat_south) * f
    }

    /// Rough resident footprint of the grid state (bytes) — the honest
    /// "what the deep-time pass costs in memory" number.
    pub fn resident_bytes(&self) -> usize {
        let planes = (self.r.len()
            + self.h.len()
            + self.uplift.len()
            + self.t_crust.len()
            + self.exhum.len()
            + self.geotherm.len()
            + self.head.len())
            * std::mem::size_of::<f64>()
            + (self.precip.len()
                + self.bio_weather.len()
                + self.bio_resist.len()
                + self.head_exchange.len())
                * std::mem::size_of::<f32>()
            + self.crust_kind.len();
        let strata_structs = self.strata.len() * std::mem::size_of::<DeepStrata>();
        let strata_heap: usize = self.strata.iter().map(DeepStrata::heap_bytes).sum();
        planes + strata_structs + strata_heap
    }

    /// Total recorded strata units across all cells (a record-size metric).
    pub fn total_units(&self) -> usize {
        self.strata.iter().map(|s| s.units.len()).sum()
    }
}

/// Build the deep-time grid from a pregenerated world at the config resolution.
/// Thin wrapper over [`build_cells`] for the spike harnesses that hold a whole
/// [`Pregen`]; the production pipeline pass builds from the [`CellGrid`] alone
/// (it runs *inside* pregen, before a `Pregen` exists).
pub fn build(pregen: &Pregen, cfg: &DeepConfig) -> DeepGrid {
    build_cells(&pregen.grid, cfg)
}

/// Build the deep-time grid from the coarse pregen [`CellGrid`] at the config
/// resolution. Initial bedrock is the pregen elevation resampled bilinearly
/// plus an addressed roughness jitter; the uplift field is the per-provenance
/// rate, resampled the same way. Alluvium starts at zero everywhere.
pub fn build_cells(cells: &CellGrid, cfg: &DeepConfig) -> DeepGrid {
    let wp = cells.w as usize;
    // World extent in metres, and the deep grid width covering it.
    let extent_m = wp as f64 * CELL_VOXELS as f64 * 0.9;
    let w = (extent_m / cfg.cell_m).round().max(2.0) as usize;

    // Per-pregen-cell source fields (elevation, roughness, uplift rate).
    let mut src_elev = vec![0.0f64; wp * wp];
    let mut src_rough = vec![0.0f64; wp * wp];
    let mut src_uplift = vec![0.0f64; wp * wp];
    for gy in 0..wp {
        for gx in 0..wp {
            let c = cells.get(gx as i32, gy as i32).expect("in grid");
            let i = gy * wp + gx;
            src_elev[i] = c.elev_m;
            src_rough[i] = provenance_roughness(c.provenance);
            src_uplift[i] = provenance_uplift(c.provenance) * cfg.uplift_scale;
        }
    }

    let n = w * w;
    let mut r = vec![0.0f64; n];
    let mut uplift = vec![0.0f64; n];
    for gy in 0..w {
        for gx in 0..w {
            // Deep cell centre in continuous pregen-cell coordinates.
            let px = (gx as f64 + 0.5) / w as f64 * wp as f64 - 0.5;
            let py = (gy as f64 + 0.5) / w as f64 * wp as f64 - 0.5;
            let elev = bilinear(&src_elev, wp, px, py);
            let rough = bilinear(&src_rough, wp, px, py);
            let up = bilinear(&src_uplift, wp, px, py);
            let i = gy * w + gx;
            let jitter = (draw_f64(&[cfg.seed, SALT_DT_ROUGH, gx as u64, gy as u64]) * 2.0 - 1.0)
                * rough
                * cfg.rough_jitter;
            r[i] = elev + jitter;
            uplift[i] = up;
        }
    }

    let strata = if cfg.record {
        vec![DeepStrata::default(); n]
    } else {
        Vec::new()
    };

    // Tectonic-history crustal columns (§ 5.2). Off → empty planes, and every
    // consumer (thickening, isostasy, exhumation) is skipped, so the run is
    // byte-identical to the pre-tectonic-history engine.
    let (t_crust, crust_kind, exhum, geotherm) = if cfg.tectonic_history {
        let (t, k) = super::tectonics::seed_columns(cfg, w, cfg.cell_m);
        // `geotherm` is sized here and filled by the field pass's pre-loop seed;
        // empty off the tectonic path, where the field has no crustal state.
        (t, k, vec![0.0f64; n], vec![0.0f64; n])
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    DeepGrid {
        w,
        cell_m: cfg.cell_m,
        r,
        h: vec![0.0f64; n],
        uplift,
        precip: vec![0.6f32; n],
        strata,
        bio_weather: Vec::new(),
        bio_resist: Vec::new(),
        t_crust,
        crust_kind,
        exhum,
        geotherm,
        // The head field is sized by its own pass on first march (and left empty
        // when the flag is off, so the pass is absent and the run is
        // byte-identical).
        head: Vec::new(),
        head_exchange: Vec::new(),
        lat_south: LAT_SOUTH,
        lat_north: LAT_NORTH,
    }
}

/// Bilinear sample of a `wp × wp` field at continuous cell coordinates
/// `(px, py)` (cell centres at integer coords), clamped at the border.
fn bilinear(field: &[f64], wp: usize, px: f64, py: f64) -> f64 {
    let clamp = |v: f64| v.clamp(0.0, wp as f64 - 1.0);
    let x = clamp(px);
    let y = clamp(py);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(wp - 1);
    let y1 = (y0 + 1).min(wp - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let at = |xx: usize, yy: usize| field[yy * wp + xx];
    let a = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
    let b = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
    a * (1.0 - fy) + b * fy
}
