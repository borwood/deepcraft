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
        let planes = (self.r.len() + self.h.len() + self.uplift.len()) * std::mem::size_of::<f64>()
            + (self.precip.len() + self.bio_weather.len() + self.bio_resist.len())
                * std::mem::size_of::<f32>();
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
