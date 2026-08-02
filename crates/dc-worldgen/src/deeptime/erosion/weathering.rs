//! **Weathering** — the bedrock-to-regolith conversion phase, the outcrop
//! exposure that feeds it, and the periglacial (frost) agent folded into its
//! rate.
//!
//! The per-cell weathering kernel itself lives in the north-star behavior shape
//! (`super::super::weather_behavior`, S16); what remains here is the phase.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::super::climate;
use super::super::grid::{DeepConfig, DeepGrid};
use super::super::lithology::{self, Agent, Litho};
use super::super::weather_behavior;
use super::creep_kernel::sus_at;
use super::{Erosion, SPECIES};

// The per-cell weathering kernel now lives in the north-star behavior shape
// (`weather_behavior::weather_one_cell` / `WeatheringPass`, S16). The domain
// narrative that used to sit here — **why it is the rate-limiting phase on
// hillslopes** (journal/0029: diffusion is flux-limited by the regolith actually
// available, so the landscape's lowering rate collapses to the rate bedrock is
// *converted* to regolith, which is why weathering had to be coupled at all),
// and **why the factor is a sum/product over agents** (in-place weathering is not
// one process; today the mechanical/abrasion term plus the periglacial frost
// term, and the day the dissolution agent lands a limestone weathers *fast*
// through the chemical term while resisting the mechanical one — the karst story
// arriving without a rewrite) — carries over unchanged; the composition order is
// pinned in `BedrockWeather::weather_rate`.

/// The biotic weathering multiplier at cell `i`: `1.0` when the biotic layer is
/// off (empty slice), so the abiotic weathering rate is byte-identical.
#[inline]
fn wmult_at(bio_weather: &[f32], i: usize) -> f64 {
    if bio_weather.is_empty() {
        1.0
    } else {
        f64::from(bio_weather[i])
    }
}

/// The frost weathering multiplier at cell `i`: `≥ 1.0` when the periglacial
/// agent is on, and the exact identity `1.0` when the plane is empty (the frost
/// agent is off). Multiplied onto the weathering rate, so `× 1.0` keeps the
/// frost-off path byte-identical (journal/0034).
#[inline]
fn frost_at(frost: &[f64], i: usize) -> f64 {
    if frost.is_empty() { 1.0 } else { frost[i] }
}

impl Erosion {
    /// The lithology outcropping at each cell as of the last [`Self::expose`]
    /// (empty when the erodibility coupling is off) — read by the measurement
    /// probe to attribute landform statistics to rock type.
    pub fn exposed(&self) -> impl Iterator<Item = Litho> + '_ {
        self.litho.iter().map(|&b| Litho::ALL[b as usize])
    }

    /// Determine which lithology outcrops at each cell and cache its
    /// agent-specific rate multipliers for this epoch (the erodibility
    /// coupling, `deeptime::lithology`).
    ///
    /// Runs at the **top** of the step, so every phase in the epoch sees one
    /// consistent answer to "what rock is at the surface here" — the record is
    /// only rewritten at the end of the step, so this reads the true current
    /// state, not a lagged one (unlike the S10 biotic modifiers, which are
    /// deliberately one epoch behind to break the biology↔erosion cycle).
    ///
    /// **The exposed unit is the top of the record, and an empty record means
    /// basement.** That single fallback is where resistant shield and craton
    /// landscapes come from: strip a column past its whole sedimentary history
    /// and what the flow meets next is igneous basement, the hardest thing in
    /// the world. Nobody wrote a shield rule.
    ///
    /// A note on *which* plane the contrast between beds rides. The brief said
    /// "modulate bedrock incision", but in the two-plane model the strata record
    /// mirrors `H`, not `R` — `R` is basement everywhere. So bed-to-bed contrast
    /// (sandstone standing over mudstone) necessarily rides on **cover
    /// entrainment**, and the incision term carries the basement contrast. Both
    /// are coupled here, through the same per-cell susceptibility, because
    /// wherever cover is thin enough for incision to matter the outcropping
    /// lithology *is* what the flow is grinding.
    ///
    /// Purely per-cell → byte-identical parallel. When the coupling is off this
    /// leaves the planes empty and every consumer reads the exact identity.
    pub fn expose(&mut self, grid: &DeepGrid, cfg: &DeepConfig) {
        // Movement 2b: the same near-surface window, read for a different
        // question — not "how fast does this cell erode" but "**what is it made
        // of**", which is the identity entrainment lifts into the load. One walk,
        // two consumers; the erodibility blend and the entrainment composition can
        // never disagree about what is lying at the surface.
        if self.sorted {
            if self.shares.len() != self.n * SPECIES {
                self.shares = vec![0.0; self.n * SPECIES];
            }
            // What lies below the record, asked of the seam with an empty section.
            self.bedrock_sp
                .copy_from_slice(cfg.providers.outcrop_shares(&[]).shares());
            let strata = &grid.strata;
            let providers = cfg.providers;
            let compose = |i: usize, out: &mut [f64]| {
                let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
                out.copy_from_slice(providers.outcrop_shares(units).shares());
            };
            if self.par() {
                self.shares
                    .par_chunks_mut(SPECIES)
                    .enumerate()
                    .for_each(|(i, out)| compose(i, out));
            } else {
                for (i, out) in self.shares.chunks_mut(SPECIES).enumerate() {
                    compose(i, out);
                }
            }
        }
        if !cfg.erodibility {
            self.litho.clear();
            self.sus_flow.clear();
            self.sus_creep.clear();
            return;
        }
        // Six-entry tables, rebuilt each epoch (cheap, and keeps the knobs live
        // if a harness mutates the config between steps).
        let flow_tab = lithology::susceptibility_table(
            Agent::Abrasion,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let creep_tab = lithology::susceptibility_table(
            Agent::Abrasion,
            cfg.erodibility_diffusion_contrast,
            cfg.erodibility_max,
        );
        if self.litho.len() != self.n {
            self.litho = vec![0u8; self.n];
            self.sus_flow = vec![1.0; self.n];
            self.sus_creep = vec![1.0; self.n];
        }
        let strata = &grid.strata;
        // The outcrop-shares seam (providers.rs § `outcrop_shares`): identity = the
        // per-Litho shares of the near-surface window; heir = structural deformation
        // (dip/fold), a pinned pair with `outcrop_at`. Erosion blends the
        // susceptibility table by share rather than taking the window's argmax and
        // stepping the rate discontinuously at the plurality crossover — the S-4
        // flag from walk-0071, cured by construction (journal/0072). Argmax is the
        // degenerate case: a single-lithology window blends to that rock's rate bit
        // for bit. `litho[i]` (the debug outcrop, read by `exposed()`) stays the
        // dominant share.
        let providers = cfg.providers;
        let per_cell = |i: usize| -> (u8, f64, f64) {
            let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
            let shares = providers.outcrop_shares(units);
            let k = lithology::dominant_litho(&shares).index() as u8;
            (
                k,
                lithology::blend_susceptibility(&shares, &flow_tab),
                lithology::blend_susceptibility(&shares, &creep_tab),
            )
        };
        if self.par() {
            self.litho
                .par_iter_mut()
                .zip(self.sus_flow.par_iter_mut())
                .zip(self.sus_creep.par_iter_mut())
                .enumerate()
                .for_each(|(i, ((l, f), c))| {
                    let (li, fi, ci) = per_cell(i);
                    *l = li;
                    *f = fi;
                    *c = ci;
                });
        } else {
            for i in 0..self.n {
                let (li, fi, ci) = per_cell(i);
                self.litho[i] = li;
                self.sus_flow[i] = fi;
                self.sus_creep[i] = ci;
            }
        }
    }

    // ---- phase 1c: periglacial frost (PARALLEL — per-cell independent) -----

    /// Compute the **temperature-gated frost weathering multiplier** per cell for
    /// this epoch (the frost agent, journal/0034). Leaves the plane empty — read
    /// as the identity `1.0` — when `full_agents` is off, so the frost-off path is
    /// byte-identical.
    ///
    /// **Freeze–thaw is maximal in a band around `0°C`, not monotonic with cold.**
    /// Rock shatters where water repeatedly crosses the phase boundary inside its
    /// pores and joints; a permanently frozen summit barely weathers, and so does
    /// a warm lowland. So the multiplier peaks at `0°C` and tapers linearly to
    /// `1.0` at `±frost_band_width_c`. Temperature is the shared climate model
    /// ([`climate::air_temp_c`]) — latitude minus an elevation lapse — so the
    /// periglacial band rides *up* the mountains and *down* the latitudes exactly
    /// where the biotic layer already agrees it freezes (deep time carries no
    /// pregen `temp_c`, but latitude and the eroding surface are enough for an
    /// honest gate).
    ///
    /// The enhancement is weighted by the rock's **frost/ice** resistance axis
    /// ([`Agent::FrostIce`]): a permeable, poorly-cemented bed shatters where a
    /// tight granite endures — the axis 0029 built and left dormant, now read. It
    /// is independent of the erodibility flag (it reads the exposed lithology
    /// straight from the record top), so periglacial shattering works whether or
    /// not the mechanical coupling is on. Purely per-cell → byte-identical
    /// parallel.
    pub fn periglacial(&mut self, grid: &DeepGrid, cfg: &DeepConfig) {
        if !cfg.full_agents {
            self.frost.clear();
            return;
        }
        let frost_tab = lithology::susceptibility_table(
            Agent::FrostIce,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        if self.frost.len() != self.n {
            self.frost = vec![1.0; self.n];
        }
        let (w, gain, width) = (self.w, cfg.frost_weathering_gain, cfg.frost_band_width_c);
        let strata = &grid.strata;
        let providers = cfg.providers;
        let (r, h) = (&grid.r, &grid.h);
        let per_cell = |i: usize| -> f64 {
            let gy = i / w;
            let surf = r[i] + h[i];
            let t = f64::from(climate::air_temp_c(grid.lat_deg(gy), surf));
            // Triangular freeze–thaw band centred on 0 °C.
            let band = if width > 0.0 {
                (1.0 - (t / width).abs()).max(0.0)
            } else {
                0.0
            };
            if band <= 0.0 {
                return 1.0;
            }
            let shares =
                providers.outcrop_shares(strata.get(i).map_or(&[][..], |s| s.units.as_slice()));
            1.0 + gain * band * lithology::blend_susceptibility(&shares, &frost_tab)
        };
        if self.par() {
            self.frost
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, f)| *f = per_cell(i));
        } else {
            for i in 0..self.n {
                self.frost[i] = per_cell(i);
            }
        }
    }

    // ---- phase 2: surface snapshot ----------------------------------------

    /// Subaerial bedrock → regolith, cover-tapered, scaled by the per-cell biotic
    /// weathering multiplier (S10 lagged coupling — `grid.bio_weather`, empty and
    /// therefore uniform `1.0` when the biotic layer is off), the lithologic
    /// abrasion susceptibility, and the periglacial **frost** multiplier
    /// (journal/0034, empty and uniform `1.0` when the frost agent is off). Purely
    /// local per cell → byte-identical parallel.
    pub fn weather(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        // A Movement 2b **control**, not a term: how much regolith this world makes
        // in place, against how much its rivers ever pick up (`TransportLedger`).
        // Reading a plane cannot perturb it, and it is summed only when
        // material-aware transport is on, so the scalar path is untouched.
        let h_before = if self.sorted {
            grid.h.iter().sum::<f64>()
        } else {
            0.0
        };
        let parallel = self.par();
        let (sea, weathering, h_star) = (self.sea_level, cfg.weathering, cfg.h_star);
        let dh = &mut self.dh;
        let bio = &grid.bio_weather;
        let sus = &self.sus_flow;
        let frost = &self.frost;
        // **S16: routed through the north-star weathering behavior shape.** Each
        // cell builds a `WeatherCtx` view over the height adapter and runs the
        // `WeatheringPass`; `weather_one_cell` is byte-identical to the old
        // `weather_cell` by construction — same operands, same f64 grouping
        // (`base × ((biotic × weatherability) × frost) × taper`), same
        // `R -= q; H += q; dH += q` transfer (docs/spikes/S16). The rate factors
        // are surfaced by name: `wmult` = biotic, the blended `sus` =
        // weatherability, `frost` = the periglacial agent multiplier.
        if parallel {
            grid.r
                .par_iter_mut()
                .zip(grid.h.par_iter_mut())
                .zip(dh.par_iter_mut())
                .enumerate()
                .for_each(|(i, ((r, h), d))| {
                    weather_behavior::weather_one_cell(
                        r,
                        h,
                        d,
                        sea,
                        weathering,
                        h_star,
                        wmult_at(bio, i),
                        sus_at(sus, i),
                        frost_at(frost, i),
                    );
                });
        } else {
            for (i, ((r, h), d)) in grid
                .r
                .iter_mut()
                .zip(grid.h.iter_mut())
                .zip(dh.iter_mut())
                .enumerate()
            {
                weather_behavior::weather_one_cell(
                    r,
                    h,
                    d,
                    sea,
                    weathering,
                    h_star,
                    wmult_at(bio, i),
                    sus_at(sus, i),
                    frost_at(frost, i),
                );
            }
        }
        if self.sorted {
            self.ledger.weathered_m += grid.h.iter().sum::<f64>() - h_before;
        }
    }

    /// The per-cell **frost (periglacial) weathering multiplier** from the last
    /// step (`≥ 1.0` where the frost agent bit, exactly the identity `1.0`/empty
    /// when the frost agent is off). Read-only; the inventory-weathering pass reads
    /// it as the frost agent's driver (material-behavior.md §4). Mirrors [`area`].
    pub fn frost(&self) -> &[f64] {
        &self.frost
    }
}
