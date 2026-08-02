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
use super::super::species::{build_creep_layout, build_local_layout, csr_rows_mut, mask_of_dense};
use super::super::weather_behavior;
use super::Erosion;
use super::creep_kernel::sus_at;

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

/// **One agent's rate table, at whichever grade the run has content for**
/// (P11 slice 2).
///
/// Three passes besides `expose` blend a susceptibility table by a cell's window —
/// frost, wind and wave — and each of them should read *the rock*, not its class,
/// for exactly the reason `expose` does: a permeable siltstone shatters where a
/// tight mudstone endures, and the class view cannot say so.
///
/// The two arms are not two rules. [`Self::Class`] is the **degenerate door**: it
/// is what these passes answered before this slice and it is what a run with no
/// registered content set can honestly answer at all
/// ([`SpeciesAxis::empty`](super::super::species::SpeciesAxis::empty)). Where the
/// axis is live, every material that *is* its class's reference member blends to
/// the same number bit for bit, so the member arm is a strict generalisation
/// rather than a replacement.
pub(super) enum SusTable {
    Class(Box<[f64; Litho::COUNT]>),
    /// Indexed by axis order, with a scratch row for the per-cell window walk.
    Member(Vec<f64>),
}

impl SusTable {
    pub(super) fn build(
        axis: &super::super::species::SpeciesAxis,
        agent: Agent,
        contrast: f64,
        cap: f64,
    ) -> Self {
        if axis.is_empty() {
            SusTable::Class(Box::new(lithology::susceptibility_table(
                agent, contrast, cap,
            )))
        } else {
            SusTable::Member(lithology::member_susceptibility_table(
                axis, agent, contrast, cap,
            ))
        }
    }

    /// The blended multiplier for one cell's near-surface window. `row` is a
    /// caller-owned scratch buffer at least `axis.len()` wide (unused on the class
    /// arm), so the per-cell call allocates nothing.
    pub(super) fn blend(
        &self,
        axis: &super::super::species::SpeciesAxis,
        providers: &super::super::providers::Providers,
        units: &[super::super::recorder::DepUnit],
        row: &mut [f64],
    ) -> f64 {
        match self {
            // The degenerate door: with no content set there is no member row for
            // the seam to answer with, so this arm walks the class window directly
            // — the pre-slice identity, byte-identical. Production always has
            // content and always goes through the seam.
            SusTable::Class(tab) => {
                lithology::blend_susceptibility(&lithology::exposed_shares(units), tab)
            }
            SusTable::Member(tab) => {
                let aw = axis.len();
                providers.outcrop_shares(axis, units, &mut row[..aw]);
                let codes: [u8; super::super::species::MAX_DEEP_SPECIES] =
                    std::array::from_fn(|i| i as u8);
                lithology::blend_member_susceptibility(&codes[..aw], &row[..aw], tab)
            }
        }
    }
}

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
        // **The member-grade path** (P11 slice 2). The near-surface window is read
        // once per cell per pass and answers *three* questions at member grade —
        // what is lying here (the `shares` plane entrainment and creep lift), how
        // fast it wears (the two susceptibility blends), and which rock outcrops
        // (the debug verdict). One walk, three consumers, exactly as the class-grade
        // path had one walk and two: the erodibility blend and the entrainment
        // composition can never disagree about what is at the surface.
        //
        // It costs **two window walks per cell per epoch**, which is what the
        // class-grade path already cost (the `shares` compose and the `sus`
        // per-cell closure each called the seam). The first walk derives the
        // presence mask and the rates; the layouts are built from the masks; the
        // second scatters the shares into the CSR rows the layout has just sized.
        if self.sorted {
            self.expose_member(grid, cfg);
            return;
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
        let per_cell = |i: usize| -> (u8, f64, f64) {
            let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
            // The degenerate door (no content set): the class window walk direct,
            // exactly as it was before P11 slice 2.
            let shares = lithology::exposed_shares(units);
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

    /// **The member-grade window pass** (P11 slice 2) — the composition, the two
    /// erodibility blends and the outcrop verdict, from one walk per cell.
    ///
    /// This is where *"mudstone stops eroding at siltstone's rate"* actually
    /// happens. The class-grade path coarsened a recorded `MaterialId` back through
    /// [`Litho::of_material`] before accumulating the window, because the tables
    /// downstream were class-keyed; the accumulator here is per **material** and
    /// the table it blends is
    /// [`member_susceptibility_table`](lithology::member_susceptibility_table),
    /// whose reference is the named anchor rock rather than a class. Every material
    /// that *is* its class's reference member keeps its old multiplier bit for bit;
    /// every other member gets its own, which is the point of the slice.
    fn expose_member(&mut self, grid: &DeepGrid, cfg: &DeepConfig) {
        let n = self.n;
        let parallel = self.par();
        let axis = self.axis.clone();
        let axis = &axis;
        let aw = axis.len();
        let strata = &grid.strata;
        let erod = cfg.erodibility;
        // Per-agent rate rows: one per (agent, epoch), never one per cell, so a
        // dense row over the axis is honest here — the `powf` is paid `axis.len()`
        // times an epoch, exactly as the class-grade table paid it seven times.
        let flow_tab = lithology::member_susceptibility_table(
            axis,
            Agent::Abrasion,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let creep_tab = lithology::member_susceptibility_table(
            axis,
            Agent::Abrasion,
            cfg.erodibility_diffusion_contrast,
            cfg.erodibility_max,
        );
        if erod && self.litho.len() != n {
            self.litho = vec![0u8; n];
            self.sus_flow = vec![1.0; n];
            self.sus_creep = vec![1.0; n];
        }
        if !erod {
            self.litho.clear();
            self.sus_flow.clear();
            self.sus_creep.clear();
        }
        // Pass 1: the walk. Mask (always), rates and verdict (when coupled).
        // Sequential — the walk is short (the top 0.9 m of a record) and the
        // parallel driver would need a second ragged decomposition for no gain.
        {
            // The dense row's axis codes are `0..aw`; the blend anchors on its own
            // argmax exactly as the class-grade one does.
            let codes: Vec<u8> = (0..aw as u8).collect();
            let mut row = vec![0.0f64; aw];
            let providers = cfg.providers;
            let Erosion {
                masks,
                litho,
                sus_flow,
                sus_creep,
                ..
            } = self;
            for i in 0..n {
                let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
                providers.outcrop_shares(axis, units, &mut row);
                masks[i] = mask_of_dense(&row);
                if erod {
                    sus_flow[i] = lithology::blend_member_susceptibility(&codes, &row, &flow_tab);
                    sus_creep[i] = lithology::blend_member_susceptibility(&codes, &row, &creep_tab);
                    // The debug outcrop stays the dominant *class*, derived from the
                    // dominant material — a summary of the quantity, never a second
                    // stored label (S-3).
                    let mut d = 0usize;
                    for k in 1..aw {
                        if row[k] > row[d] {
                            d = k;
                        }
                    }
                    litho[i] = Litho::of_material(axis.material(d)).index() as u8;
                }
            }
        }
        // The layouts the three composition planes ride, sized from the masks.
        build_local_layout(&mut self.window, n, &self.masks);
        if self.creep_carries {
            build_creep_layout(&mut self.clayout, self.w, &self.masks);
            self.creep_sp.reset_for(&self.clayout);
            if self.creep_gross.len() != n {
                self.creep_gross = vec![0.0; n];
            }
        }
        self.shares.reset_for(&self.window);
        // Pass 2: scatter the shares into the rows the layout has just sized.
        {
            let Erosion { window, shares, .. } = self;
            let window = &*window;
            let mut rows = csr_rows_mut(shares.vals_mut(), window);
            let providers = cfg.providers;
            let compose = |i: usize, out: &mut [f64]| {
                let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
                let (_, ks) = window.row(i);
                let mut dense = [0.0f64; super::super::species::MAX_DEEP_SPECIES];
                providers.outcrop_shares(axis, units, &mut dense[..aw]);
                for (j, &k) in ks.iter().enumerate() {
                    out[j] = dense[k as usize];
                }
            };
            if parallel {
                rows.par_iter_mut()
                    .enumerate()
                    .for_each(|(i, out)| compose(i, out));
            } else {
                for (i, out) in rows.iter_mut().enumerate() {
                    compose(i, out);
                }
            }
        }
        // What lies below the record: the same walk asked with an empty section,
        // which is entirely whatever is beneath the pile. One entry, share `1.0`.
        self.bedrock_axis = vec![axis.basement_slot() as u8];
        self.bedrock_sp = vec![1.0];
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
        let frost_tab = SusTable::build(
            &self.axis,
            Agent::FrostIce,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        if self.frost.len() != self.n {
            self.frost = vec![1.0; self.n];
        }
        let (w, gain, width) = (self.w, cfg.frost_weathering_gain, cfg.frost_band_width_c);
        let parallel = self.par();
        let n = self.n;
        let strata = &grid.strata;
        let providers = cfg.providers;
        let Erosion { frost, axis, .. } = self;
        let axis = &*axis;
        let (r, h) = (&grid.r, &grid.h);
        let per_cell = |i: usize, row: &mut [f64]| -> f64 {
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
            let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
            1.0 + gain * band * frost_tab.blend(axis, &providers, units, row)
        };
        let aw = axis.len();
        if parallel {
            frost
                .par_iter_mut()
                .enumerate()
                .for_each_init(|| vec![0.0f64; aw], |row, (i, f)| *f = per_cell(i, row));
        } else {
            let mut row = vec![0.0f64; aw];
            for (i, f) in frost.iter_mut().enumerate().take(n) {
                *f = per_cell(i, &mut row);
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
