//! **The mass-neutral surface agents** — littoral wave cutting at the current sea
//! stand, and eolian deflation with its downwind loess/dune deposition. Both run
//! *after* the recorder and **self-record**, so their distinct facies reach the
//! strata.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use super::super::climate;
use super::super::flux::FlowCause;
use super::super::grid::{DeepConfig, DeepGrid};
use super::super::lithology::{self, Agent};
use super::super::providers::WaveCell;
use super::super::recorder::{Aridity, DepEnv, DepTag, EnergyBand, Eolian, MemberCtx, dep_tags};
use super::weathering::SusTable;
use super::{Erosion, NEIGH8, coords_of, in_grid};

impl Erosion {
    /// **Wind deflation and downwind loess/dune deposition** (the eolian agent,
    /// [`Agent::Eolian`], journal/0034). A 1D march along the **prevailing-wind**
    /// direction the climate already uses ([`climate::zonal_wind`] — never a
    /// second wind): dry, unvegetated, subaerial cells hand loose cover to an
    /// airborne load; vegetated or humid downwind cells trap it. In the arid
    /// source zone the trapped sand records as a **dune field** ([`Eolian::Dune`],
    /// coarse); on the damp margin the fine silt records as a **loess** sheet
    /// ([`Eolian::Loess`], fine). At 460 m the unit is the *region*, not the
    /// individual dune (earth-processes.md § 4).
    ///
    /// **Smooth zonal profile** (journal/0037): the march direction is the sign
    /// of `zonal_wind`, and the deflation rate is scaled by its **magnitude**
    /// `|zonal_wind| ∈ [0, 1]`. So in a calm belt (the horse latitudes at 30°,
    /// the polar front at 60°) the wind neither picks up nor carries — dune fields
    /// fade to nothing there rather than reversing direction at full strength
    /// across a single grid row (the analytic-boundary "scream" the old three-way
    /// `wind_dx` bit produced).
    ///
    /// Wind only **redistributes** loose `H` (it never touches bedrock, and never
    /// adds external mass), so the mass ledger `Δ(ΣR+ΣH) == uplift + biotic` is
    /// untouched: each row conserves its own airborne load, and whatever is still
    /// aloft at the downwind edge settles there. The deflation susceptibility is
    /// the rock's **eolian** resistance axis (cohesion — loose sand blows, crusted
    /// clay resists), scaled by how arid and how bare the cell is.
    ///
    /// Runs **after** the recorder and self-records (like the biotic layer), so
    /// its own facies reach the record rather than being lumped under the epoch's
    /// fluvial tag. Scalar in both drivers (a row carries a serial load), so it is
    /// deterministic and scalar↔parallel byte-identical.
    pub fn wind(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, mem: MemberCtx<'_>) {
        let record = !grid.strata.is_empty();
        let chapter = self.cur_chapter;
        let epoch = self.cur_epoch;
        let sus_tab = SusTable::build(
            &self.axis,
            Agent::Eolian,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let mut row = vec![0.0f64; self.axis.len()];
        let providers = cfg.providers;
        let (w, thr, sea) = (self.w, cfg.eolian_arid_precip, self.sea_level);
        let (defl, dep_frac) = (cfg.eolian_deflation, cfg.eolian_deposit_frac);
        for gy in 0..w {
            // Direction and strength both come from the shared smooth profile: the
            // sign steps the row, the magnitude (0..1) scales deflation so calm
            // belts do no eolian work (journal/0037).
            let wind = climate::zonal_wind(grid.lat_deg(gy));
            let dir = if wind > 0.0 { 1 } else { -1 };
            let wind_mag = wind.abs();
            let mut load = 0.0f64;
            let mut last_land: Option<usize> = None;
            for s in 0..w {
                let gx = if dir > 0 { s } else { w - 1 - s };
                let i = gy * w + gx;
                let surf = grid.r[i] + grid.h[i];
                if surf <= sea {
                    // Over water: the airborne load settles out (dust on the sea).
                    if load > 0.0 {
                        // journal/0111: dust that reaches the sea has left the land.
                        if self.denude {
                            self.ledger.eolian_to_sea_m += load;
                        }
                        grid.h[i] += load;
                        if record {
                            let tag = DepTag {
                                eolian: Eolian::Loess,
                                ..DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low)
                            };
                            // Dust settling on the sea: the wind carries no
                            // identity of its own (§ 13.2's wind member is
                            // deferred), so the class is the tag's — but the
                            // MEMBER is picked by fitness here, today.
                            let m = mem.surface(
                                i,
                                f64::from(climate::air_temp_c(grid.lat_deg(gy), surf)),
                                f64::from(grid.precip[i]),
                                lithology::litho_of_tag(tag),
                                dep_tags::EOLIAN,
                                0,
                            );
                            grid.strata[i].deposit_moved(
                                tag,
                                load,
                                chapter,
                                epoch,
                                m,
                                FlowCause::Eolian as u8,
                            );
                        }
                        load = 0.0;
                    }
                    continue;
                }
                last_land = Some(i);
                let precip = f64::from(grid.precip[i]);
                let arid = if thr > 0.0 {
                    ((thr - precip) / thr).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let veg = if grid.bio_resist.is_empty() {
                    0.0
                } else {
                    f64::from(grid.bio_resist[i])
                };
                // Deflation: dry, bare cells hand loose cover to the wind. Floor
                // available cover at zero first — `H` can carry a sub-ULP negative
                // from fp round-off, and `clamp(0.0, neg)` would panic.
                let units = grid.strata.get(i).map_or(&[][..], |s| s.units.as_slice());
                let sus = sus_tab.blend(&self.axis, &providers, units, &mut row);
                let avail = grid.h[i].max(0.0);
                let pickup = (defl * sus * arid * (1.0 - veg) * wind_mag).clamp(0.0, avail);
                if pickup > 0.0 {
                    grid.h[i] -= pickup;
                    load += pickup;
                    if record {
                        grid.strata[i].erode(pickup);
                    }
                }
                // Deposition: vegetated or humid ground traps the load.
                let trap = (1.0 - arid).max(veg).clamp(0.0, 1.0);
                let drop = load * dep_frac * trap;
                if drop > 0.0 {
                    grid.h[i] += drop;
                    load -= drop;
                    if record {
                        // Dune field in the hyper-arid sand-source core, loess on
                        // the semi-arid downwind margin. (0.7 is an appearance-
                        // class split — a user-owned magnitude, journal/0034.)
                        let (facies, energy, ar) = if arid > 0.7 {
                            (Eolian::Dune, EnergyBand::High, Aridity::Arid)
                        } else if precip < thr {
                            (Eolian::Loess, EnergyBand::Low, Aridity::Arid)
                        } else {
                            (Eolian::Loess, EnergyBand::Low, Aridity::Humid)
                        };
                        let tag = DepTag {
                            eolian: facies,
                            ..DepTag::mineral(DepEnv::Subaerial, ar, energy)
                        };
                        let m = mem.surface(
                            i,
                            f64::from(climate::air_temp_c(grid.lat_deg(gy), surf)),
                            precip,
                            lithology::litho_of_tag(tag),
                            dep_tags::EOLIAN,
                            1,
                        );
                        grid.strata[i].deposit_moved(
                            tag,
                            drop,
                            chapter,
                            epoch,
                            m,
                            FlowCause::Eolian as u8,
                        );
                    }
                }
            }
            // Conserve the row: whatever is still aloft settles at the downwind
            // land edge (so Σ eolian ΔH over the row is zero — pure redistribution).
            if load > 0.0
                && let Some(i) = last_land
            {
                grid.h[i] += load;
                if record {
                    let tag = DepTag {
                        eolian: Eolian::Loess,
                        ..DepTag::mineral(DepEnv::Subaerial, Aridity::Arid, EnergyBand::Low)
                    };
                    let m = mem.surface(
                        i,
                        f64::from(climate::air_temp_c(grid.lat_deg(gy), grid.r[i] + grid.h[i])),
                        f64::from(grid.precip[i]),
                        lithology::litho_of_tag(tag),
                        dep_tags::EOLIAN,
                        2,
                    );
                    grid.strata[i].deposit_moved(tag, load, chapter, epoch, m, FlowCause::Eolian as u8);
                }
            }
        }
    }

    // ---- phase 11: littoral wave attack (SCALAR — shore cells) ------------

    /// **Wave-cut littoral erosion** at the current sea-level stand (the wave
    /// agent, [`Agent::Wave`], journal/0034). A cell is attacked when it stands in
    /// the freeboard band `(sea, sea + wave_band_m]` **and** touches open water (a
    /// subsea neighbour) — the shoreline. Waves cut it down toward sea level,
    /// clamped so the cell never drops below the stand (a wave-cut platform forms
    /// *at* sea level, it does not dig a hole), scaled by the rock's **wave**
    /// resistance axis (cohesion/jointing — the axis 0029 built and left dormant)
    /// and a freeboard taper (strongest right at the waterline).
    ///
    /// The quarried volume goes **offshore** into the deepest adjacent subsea cell
    /// as marine sediment, so the term is mass-neutral (`ΣR+ΣH` unchanged): loose
    /// cover is entrained first, then bedrock, and the sum is deposited into the
    /// sink. Because the sea-level curve cycles (§ 6), the attacked band sweeps up
    /// and down the coast over the run — which is what records **raised and
    /// drowned wave-cut features** in a column over successive stands.
    ///
    /// Scalar (it writes a neighbour's cell), so deterministic and byte-identical
    /// scalar↔parallel. Only the thin shore band does work.
    pub fn wave(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, mem: MemberCtx<'_>) {
        let (base_rate, band) = (cfg.wave_erosion, cfg.wave_band_m);
        // The configured rate is still the *off switch* — a `0.0` global rate
        // means "no littoral term at all", provider or no provider, which is the
        // byte-identity escape `tests/full_agents.rs` already leans on. The
        // provider is consulted only for cells that survive this gate.
        if base_rate <= 0.0 || band <= 0.0 {
            return;
        }
        // The wave-energy seam (providers.rs § `wave_energy`): identity = the
        // configured global rate; heir = fetch (S11 body graph) × zonal wind.
        let providers = cfg.providers;
        let record = !grid.strata.is_empty();
        let chapter = self.cur_chapter;
        let epoch = self.cur_epoch;
        let (w, sea) = (self.w, self.sea_level);
        let sus_tab = SusTable::build(
            &self.axis,
            Agent::Wave,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let mut row = vec![0.0f64; self.axis.len()];
        for i in 0..self.n {
            let free = grid.r[i] + grid.h[i] - sea;
            if free <= 0.0 || free > band {
                continue; // below water, or too high up the shore to be reached
            }
            // Deepest adjacent subsea cell is the offshore sink (deterministic).
            let (gx, gy) = coords_of(i, w);
            let mut sink: Option<usize> = None;
            let mut sink_surf = f64::INFINITY;
            for (dx, dy) in NEIGH8 {
                if let Some(j) = in_grid(gx + dx, gy + dy, w) {
                    let sj = grid.r[j] + grid.h[j];
                    if sj <= sea && sj < sink_surf {
                        sink_surf = sj;
                        sink = Some(j);
                    }
                }
            }
            let Some(j) = sink else {
                continue; // not on the coast — no open water adjacent
            };
            let units = grid.strata.get(i).map_or(&[][..], |s| s.units.as_slice());
            let sus = sus_tab.blend(&self.axis, &providers, units, &mut row);
            let taper = (1.0 - free / band).clamp(0.0, 1.0);
            let rate = providers.wave_energy(WaveCell {
                index: i,
                gx: gx as usize,
                gy: gy as usize,
                base_rate,
            });
            let cut = (rate * sus * taper).min(free);
            if cut <= 0.0 {
                continue;
            }
            // Entrain loose cover first, then quarry bedrock; the sum goes offshore.
            let removed_h = grid.h[i].min(cut);
            grid.h[i] -= removed_h;
            grid.r[i] -= cut - removed_h;
            grid.h[j] += cut;
            // journal/0111: `j` is a subsea cell by construction, so every metre
            // of this is export from the land system — and the bedrock share is
            // exhumation `track_exhumation` cannot see, because wave runs after it.
            if self.denude {
                self.ledger.wave_offshore_m += cut;
                self.ledger.wave_bedrock_m += cut - removed_h;
            }
            if record {
                if removed_h > 0.0 {
                    grid.strata[i].erode(removed_h);
                }
                let tag = DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low);
                let m = mem.surface(
                    j,
                    // `j`'s own row: the bed is laid where the sediment lands.
                    f64::from(climate::air_temp_c(
                        grid.lat_deg(j / w),
                        grid.r[j] + grid.h[j],
                    )),
                    f64::from(grid.precip[j]),
                    lithology::litho_of_tag(tag),
                    dep_tags::WAVE,
                    0,
                );
                grid.strata[j].deposit_moved(tag, cut, chapter, epoch, m, FlowCause::Marine as u8);
            }
        }
    }
}
