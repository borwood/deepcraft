//! **The denudation ledger and the flux-record exports** — where every metre
//! that leaves the domain is tallied (fluvial yield to the sea, regolith crept
//! off the coast, export across the border) and where the per-face load and area
//! planes the flow record reads are handed out.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use super::super::grid::{DeepConfig, DeepGrid};
use super::creep_kernel::eff_diff;
use super::mfd::MFD_DIRS;
use super::{Erosion, NEIGH4, coords_of, in_grid};

/// **Where the transport pass picked material up and where it put it down**,
/// summed over a whole run in metres (Movement 2b). All zero when material-aware
/// transport is off.
///
/// It exists because "the facies gradient did not express" is not one finding, it
/// is three, with three different heirs:
///
/// - **nothing was picked up** — the pass is not the thing shaping this landscape,
///   and the heir is the erosion budget, not the sorting rule;
/// - **it was picked up and set straight back down** (`by_competence` ≈
///   `entrained + incised`) — the flows cannot carry what the hillslopes supply,
///   and the heir is the competence calibration or the discharge;
/// - **it travelled and then fined** — the slice worked.
///
/// A single "did the gradient appear" number cannot tell those apart, and
/// guessing between them is how a slice gets tuned in the wrong place.
#[derive(Clone, Copy, Default, Debug)]
pub struct TransportLedger {
    /// Loose cover lifted into the load at its source (`loose→load`, § 13.6).
    pub entrained_m: f64,
    /// Bedrock detached into the load by incision.
    pub incised_m: f64,
    /// Set down because the flow ran out of **capacity** — the coarsest-first draw.
    pub deposited_by_capacity_m: f64,
    /// Set down because the flow ran out of **competence** — the falling ceiling.
    pub deposited_by_competence_m: f64,
    /// Set down at a sink (the sea, or the domain border), where everything
    /// suspended settles regardless.
    pub deposited_at_sink_m: f64,
    /// **The control the whole diagnosis turns on:** bedrock converted to regolith
    /// *in place* by the weathering phase, over the run. This material never enters
    /// a load and never travels, and if it dwarfs `entrained_m` then the archive is
    /// not a fluvial deposit at all and no amount of sorting can make it read like
    /// one.
    pub weathered_m: f64,
    /// The other control: regolith moved by **hillslope diffusion**, summed as the
    /// per-epoch gain side of the gather (so it is mass *moved*, not net change,
    /// which is zero by construction). Creep is the § 13.2 gravity/mass-wasting
    /// family — a transport agent this slice deliberately does not make
    /// material-aware — so this number is how much of the world's sediment routing
    /// the slice did **not** reach.
    pub diffused_m: f64,

    // ---- the DENUDATION itemisation (journal/0111) ------------------------
    //
    // Everything above measures material **moving inside** the landscape.
    // Denudation is a different question: how much leaves the *land system*
    // altogether. Weathering in place is not denudation; creeping one cell
    // downslope is not denudation. Crossing the shoreline is.
    //
    // These are the complete set of ways mass crosses from a subaerial cell to a
    // submerged one (or piles against the domain edge) in this engine, and they
    // are accumulated **only when [`Erosion::set_denudation_ledger`] is on** —
    // off, they are exactly zero, no branch fires, and the run is byte- *and
    // cost*-identical to production. They are read-only with respect to the
    // physics: every one is a `+=` on this struct.
    /// Fluvial load deposited at a sink that is **submerged** at the epoch's sea
    /// stand — sediment yield to the sea, the closest thing this engine has to
    /// what a gauging station or a cosmogenic-nuclide catchment average measures.
    pub sink_marine_m: f64,
    /// Fluvial load deposited at a sink that is a **subaerial domain-border**
    /// cell. Not denudation: it is still on land, piled against the edge of the
    /// simulated box. Broken out so it can never be quietly counted as export.
    /// `sink_marine_m + sink_border_m == deposited_at_sink_m` — the itemisation
    /// the gate asserts.
    pub sink_border_m: f64,
    /// Regolith **crept across the shoreline** by hillslope diffusion — the
    /// land→sea half of the gather's edge fluxes. Creep moves 918× the fluvial
    /// load on this world (corrections #55), so this is the term that decides the
    /// answer, and it is the one no prior instrument could see.
    pub creep_to_sea_m: f64,
    /// Shore material quarried by the **wave** agent and deposited offshore
    /// (loose cover first, then bedrock). By construction the sink is submerged,
    /// so all of it is export.
    pub wave_offshore_m: f64,
    /// The bedrock share of [`Self::wave_offshore_m`]. Wave attack lowers `R`
    /// *after* [`Erosion::track_exhumation`] has run, so this material is
    /// **missing from `grid.exhum`** and must be added back when bedrock erosion
    /// is totalled.
    pub wave_bedrock_m: f64,
    /// Airborne **dust settling on the sea** during the eolian march.
    pub eolian_to_sea_m: f64,
    /// **Cell-epochs in which the creep flux limiter bound** — the cell wanted to
    /// shed more regolith than it had, so [`diffuse_scale_cell`] clamped its export
    /// to its whole `H` (journal/0114).
    ///
    /// **This is the honesty check on the erosional calibration, and it measures a
    /// discretisation limit rather than a physics one.** Hillslope diffusion is
    /// explicit: a cell's potential outflow is `Σ_downhill diffusion · Δsurf`, and
    /// once that exceeds the regolith present, the pass stops being a diffusion and
    /// becomes *"move everything one cell downslope this epoch"*. At 460 m cells and
    /// 2.5 Myr per epoch that conveyor is a creep velocity of ~0.18 mm/yr, which is
    /// squarely inside the measured range for real soil creep — so the *rate* stays
    /// honest even where the *operator* has degenerated. What is lost is sub-cell
    /// structure, not magnitude.
    ///
    /// Reported so a calibration cannot quietly buy its denudation by pushing the
    /// whole world into that regime. **Counts only cells that actually held
    /// regolith** — see [`Self::creep_cell_epochs`]. Counted only when
    /// [`DeepConfig::denudation_ledger`](super::super::grid::DeepConfig::denudation_ledger)
    /// is on; zero, and not even summed, in production.
    pub creep_limited_cell_epochs: u64,
    /// Cell-epochs in which the diffusion pass ran **over a cell that had regolith
    /// to move** — the denominator for [`Self::creep_limited_cell_epochs`].
    ///
    /// The `h > 0` restriction is load-bearing, not tidiness. The limiter's predicate
    /// is `potential outflow > available cover`, which is *trivially* true at `h == 0`,
    /// so counting every cell would score the bare ocean floor and every stripped ridge
    /// as transport-limited and report a saturation that is really an absence. Same
    /// gating.
    ///
    /// **Since journal/0122 the unit is a cell-SUB-STEP**, because the pass sub-cycles
    /// (`DeepConfig::creep_substep`). Both this and
    /// [`Self::creep_limited_cell_epochs`] count every sub-step, so the *ratio* — which
    /// is the only thing anyone reads — is unchanged in meaning, and identical in value
    /// on any world that takes one sub-step. Named `_cell_epochs` still, because
    /// renaming a counter whose ratio is quoted in four journal entries buys nothing.
    pub creep_cell_epochs: u64,

    // ---- the CHAIN-B coupling terms (P2 audit, 2026-08-01 § 6.1 M1/M2) ----
    //
    // The calibration derivation is a one-parameter family in the mean cover
    // taper `⟨exp(−H/H*)⟩` — a quantity this repo had never printed — and the
    // audit's § 4.3 falsifies the tempting `exp(−⟨H⟩/H*)` substitute against the
    // corpus's own ladder by five orders of magnitude (Jensen's inequality: the
    // weathering happens in the thin-cover tail, the mean is set by the thick
    // bulk). So the true, area-weighted, run-integrated taper is accumulated
    // here, in the weathering phase itself, over the exact population the kernel
    // gates on (subaerial cells at the epoch's own sea stand). Same for the
    // modulator product, which decomposes the 1.34× composite of § 4.2.
    //
    // All three are accumulated **only when the denudation ledger is on** — off,
    // no branch fires and the run is byte- and cost-identical to production.
    /// `Σ exp(−H/H*)` over every (subaerial cell, epoch) the weathering phase
    /// visited — the numerator of the run-mean cover taper `⟨exp(−H/H*)⟩`.
    pub weather_taper_sum: f64,
    /// `Σ (biotic × weatherability) × frost` over the same population — the
    /// numerator of the run-mean modulator product `⟨mod⟩` (the composite the
    /// derivation could not decompose from D3 alone).
    pub weather_mod_sum: f64,
    /// The population: subaerial cell-epochs the weathering phase visited — the
    /// shared denominator of the two sums above.
    pub weather_cell_epochs: u64,
}

impl TransportLedger {
    /// **Total export from the subaerial land system** over the run, in metres of
    /// cell-thickness — the catchment-averaged denudation numerator. Border
    /// accumulation is deliberately *excluded*: it never left the land.
    pub fn exported_m(&self) -> f64 {
        self.sink_marine_m + self.creep_to_sea_m + self.wave_offshore_m + self.eolian_to_sea_m
    }
}

impl Erosion {
    /// Turn the **denudation ledger** on (journal/0111) — the five export
    /// counters on [`TransportLedger`] that measure what leaves the *land system*
    /// rather than what moves inside it.
    ///
    /// It is a flag rather than always-on for one reason: the shoreline-creep
    /// term needs a per-epoch sweep over every cell's four edges, which is real
    /// gen-time work for a number no production consumer reads. Off is therefore
    /// the S-5 identity floor — zero counters, zero branches, zero cost — and the
    /// measurement probe is the only caller that turns it on. **The gate asserts
    /// the surface plane is bit-identical either way**, which is what makes a
    /// number taken with the flag on a number about the shipped world.
    pub fn set_denudation_ledger(&mut self, on: bool) {
        self.denude = on;
    }

    /// Whether the denudation ledger is on.
    #[inline]
    pub fn is_denudation_ledger(&self) -> bool {
        self.denude
    }

    /// Split a sink's arriving load into **export** and **edge pile-up**
    /// (journal/0111). A sink is either a submerged cell — in which case the
    /// sediment has left the land system and is the yield a real catchment study
    /// would weigh — or a subaerial domain-border cell, in which case it has not
    /// left anything and is an artifact of simulating a box. Conflating the two
    /// would inflate denudation by whatever the border happens to catch, so the
    /// two are counted apart and the gate asserts they re-sum to the total.
    ///
    /// Read-only: it inspects the grid and adds to the ledger. No-op when the
    /// denudation ledger is off.
    #[inline]
    pub(super) fn tally_sink(&mut self, grid: &DeepGrid, c: usize, qin: f64) {
        if !self.denude {
            return;
        }
        if grid.surf_at(c) <= self.sea_level {
            self.ledger.sink_marine_m += qin;
        } else {
            self.ledger.sink_border_m += qin;
        }
    }

    /// **How much regolith creeps across the shoreline this epoch** — the
    /// land→sea half of the diffusion gather's edge fluxes (journal/0111).
    ///
    /// This is the term the answer turns on, because creep moves 918× what the
    /// rivers pick up on this world (corrections #55) and no instrument before
    /// this one could say how much of it actually *leaves*. It re-reads exactly
    /// the influx expression [`diffuse_net_cell`] uses for a submerged cell `j`
    /// from a higher subaerial neighbour `i` — the same donor scale, the same
    /// effective diffusivity — so the number is a partition of a flux the solve
    /// already computed, not a second model of it.
    ///
    /// Called after the gather's two passes and before they are applied, on the
    /// same frozen surface, so it sees the epoch the flux belongs to.
    pub(super) fn tally_creep_to_sea(&mut self, grid: &DeepGrid, _cfg: &DeepConfig, diff: f64) {
        let (w, sea) = (self.w, self.sea_level);
        let mut sum = 0.0;
        for j in 0..self.n {
            if self.surf[j] > sea {
                continue; // the receiving cell must be under water
            }
            let (gx, gy) = coords_of(j, w);
            for (dx, dy) in NEIGH4 {
                let Some(i) = in_grid(gx + dx, gy + dy, w) else {
                    continue;
                };
                // Only a *subaerial* donor standing above the water is denudation:
                // a subsea→subsea edge is marine redistribution, and a downhill
                // edge out of `j` is not influx at all.
                if self.surf[i] <= sea {
                    continue;
                }
                let d = self.surf[i] - self.surf[j];
                if d > 0.0 {
                    sum += eff_diff(diff, &grid.bio_resist, &self.sus_creep, i) * d * self.scale[i];
                }
            }
        }
        self.ledger.creep_to_sea_m += sum;
    }

    /// Turn the **flow record's** per-epoch per-face capture on. Off (the
    /// default) the buffers stay empty and neither [`Self::accumulate_area`] nor
    /// [`Self::transport`] touches them, so every direct caller of `step` keeps
    /// the byte-identical old behaviour.
    pub fn set_flux_record(&mut self, on: bool) {
        if on && self.out_load.len() != self.n {
            self.out_load = vec![0.0; self.n];
            self.out_area = vec![0.0; self.n * MFD_DIRS];
            self.out_face_load = vec![0.0; self.n * MFD_DIRS];
        } else if !on {
            self.out_load = Vec::new();
            self.out_area = Vec::new();
            self.out_face_load = Vec::new();
        }
    }

    /// **The transport ledger over the whole run** (all zero when material-aware
    /// transport is off). See [`TransportLedger`].
    pub fn transport_ledger(&self) -> TransportLedger {
        self.ledger
    }

    /// The suspended load each cell handed to its receivers in the last
    /// [`Self::transport`], **summed over faces** (empty unless
    /// [`Self::set_flux_record`] is on).
    pub fn out_load(&self) -> &[f64] {
        &self.out_load
    }

    /// **The discharge that left each cell through each of its eight lateral
    /// faces** this epoch (`n × 8`, aligned to `FaceKey::lateral`) — the flow
    /// record's `M`. Empty unless [`Self::set_flux_record`] is on. A cell whose
    /// eight entries are all zero is a **sink**: its discharge left through a
    /// boundary face, which is the record's call to make, not the solve's.
    pub fn out_face_area(&self) -> &[f32] {
        &self.out_area
    }

    /// **The suspended load that crossed each of the eight lateral faces** this
    /// epoch (`n × 8`) — the flow record's `L`, partitioned by exactly the same
    /// shares as the discharge. Empty unless [`Self::set_flux_record`] is on.
    pub fn out_face_load(&self) -> &[f32] {
        &self.out_face_load
    }
}
