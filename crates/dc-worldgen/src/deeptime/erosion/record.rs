//! **The strata recorder phase** — the facies tagging (energy band, deposition
//! tag, arriving species) and the pass that commits each cell's surface change
//! into its deep-time record.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::super::climate;
use super::super::grid::DeepGrid;
use super::super::lithology::{self, Litho};
use super::super::recorder::{
    Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, MemberCtx, dep_tags,
};
use super::transport::{ENERGY_LOW_MED, ENERGY_MED_HIGH, REFERENCE_KT};
use super::{Erosion, SPECIES};
use dc_core::materials::MaterialId;

/// Map a stream transport capacity to a facies energy band, in a world whose
/// stream-transport coefficient is `k_transport`. Thresholds are in the capacity
/// units of the transport pass (metres/iteration); calibrated so headwater
/// hillslopes read Low, trunk rivers read High.
///
/// **The second argument arrived with the erosional calibration (journal/0114) and
/// it is not a convenience.** `cap = k_transport · A^m · S^n`, so a bare capacity
/// is `k_transport` times a position in the drainage network, and the boundaries
/// below describe the *position*. Left absolute, a calibration that raised
/// `k_transport` would have re-labelled essentially every depositional site on the
/// world **High energy** — `litho_of_tag` would then have recorded coarse clastic
/// everywhere, and the facies gradient the Movement 2b probes exist to measure
/// would have been erased by the same commit that was supposed to make the world
/// erode. Scaling with the reference keeps the classification invariant under a
/// pure change of rate, which is what a calibration is.
pub fn energy_band(cap: f64, k_transport: f64) -> EnergyBand {
    let s = k_transport / REFERENCE_KT;
    if cap < ENERGY_LOW_MED * s {
        EnergyBand::Low
    } else if cap < ENERGY_MED_HIGH * s {
        EnergyBand::Medium
    } else {
        EnergyBand::High
    }
}

/// The measured depositional tag for a cell given its final surface, precip, and
/// the transport capacity it saw this iteration.
#[inline]
fn tag_of(surf_i: f64, precip_i: f32, energy_i: f64, sea_level: f64, k_transport: f64) -> DepTag {
    let env = if surf_i <= sea_level {
        DepEnv::Subsea
    } else {
        DepEnv::Subaerial
    };
    let aridity = if f64::from(precip_i) < 0.32 {
        Aridity::Arid
    } else {
        Aridity::Humid
    };
    DepTag::mineral(env, aridity, energy_band(energy_i, k_transport))
}

/// Apply one cell's net thickness change to its strata record under `tag`,
/// stamped with the current tectonic `chapter` (0 when tectonic history is off).
///
/// **`tag` and `species` are LAZY, and that is a cost decision** (P11 slice 1).
/// Deposition-time member fitness is an inverse-CDF over a class's registered
/// members, and it allocates; evaluating it eagerly as a call argument would run
/// it for **every cell every epoch** — ~297 k × 200 on a production world — when
/// only the depositing minority can use the answer. A cell that eroded or did
/// nothing this epoch never asks which rock arrived, because none did.
#[inline]
fn record_cell(
    s: &mut DeepStrata,
    dh: f64,
    chapter: u8,
    deposit: impl FnOnce() -> (DepTag, MaterialId),
) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        let (tag, species) = deposit();
        s.deposit_as(tag, dh, chapter, species);
    } else {
        s.erode(-dh);
    }
}

/// **Which material the unit arriving at this cell is made of** (Movement 2b).
///
/// The cell's net gain has several sources, and **two of them now carry an
/// identity**: the fluvial transport pass knows, per species, exactly what it set
/// down (`dep`), and hillslope creep knows, per species, exactly what came down
/// the slope into this cell (`creep`, Movement 2b continuation (b)). What is left
/// in `dh` — bedrock weathered to regolith in place, wind and wave, the biotic
/// layer — never rode any mover, and keeps the answer the record has always
/// given: the tag's own lithology.
///
/// So the unit's **class** is the argmax of the whole mixture: each carried
/// species against the un-carried remainder, with ties going to the incumbent (a
/// strict `>` over fixed index order, so it is deterministic).
///
/// ⚠ **It answers the CLASS, not the rock** (P11 slice 1). The transport budgets
/// are still `Litho::COUNT`-wide, so this argmax is over classes; the *member* is
/// then chosen by fitness at deposition under the cell's own climate
/// ([`super::super::recorder::MemberCtx::surface`]). Slice 2 re-grades the budgets, at
/// which point the argmax is over materials and this function returns the rock
/// directly.
///
/// **The two movers are summed, not ranked.** A cell that receives half a metre of
/// fine clastic from upstream and half a metre of the same rock off the slope
/// above has a metre of that rock, and pretending the two halves compete would
/// make the answer depend on which agent we asked first.
///
/// **STUB #25 — which *mover* delivered it is a different axis, and the record does
/// not carry one.** Colluvium and alluvium are separable only by signature, not by
/// label; the byte that would fix it costs ~42 MiB at today's `DepUnit` layout, and
/// the free version is a packed `(species, mover)` byte. See `stubs.md` § 25 and
/// `examples/colluvium_probe.rs`, which measures the signature the label is missing.
///
/// Only *gains* are candidates: a species creep took **away** from this cell is
/// not something the cell can be made of, so the negative entries are clamped out
/// of both the mixture and the remainder.
///
/// This is deliberately *not* a threshold on "was most of this transported" —
/// a threshold would be a second rule with a number in it. It is one comparison
/// over one mixture, and it degenerates exactly to the old behaviour when nothing
/// was carried here.
#[inline]
fn arriving_species(dh: f64, dep: &[f64], creep: &[f64], tag_species: Litho) -> Litho {
    let mut mix = [0.0; SPECIES];
    let mut carried = 0.0;
    for k in 0..SPECIES {
        let m = dep[k] + creep.get(k).copied().unwrap_or(0.0).max(0.0);
        mix[k] = m;
        carried += m;
    }
    let mut best = tag_species;
    let mut best_m = dh - carried;
    let mut moved = false;
    for (k, &m) in mix.iter().enumerate() {
        if m > best_m {
            best_m = m;
            best = Litho::ALL[k];
            moved = true;
        }
    }
    // [`Litho::as_deposited`] answers *"what is this rock once a mover has set it
    // down"*, so it applies to the **carried** winner and not to the tag's own
    // default: the default was never carried anywhere and the record has always
    // been allowed to say what it says. (On today's world the distinction is
    // inert — the erosion recorder builds mineral tags only, and `litho_of_tag`
    // maps those to clastics, which `as_deposited` leaves alone — but the rule
    // should be right rather than accidentally right.)
    if moved { best.as_deposited() } else { best }
}

impl Erosion {
    /// Record each cell's net thickness change this iteration under the tag
    /// measured now. Each cell's `DeepStrata` is independent → byte-identical
    /// parallel (per-cell record ops, disjoint records).
    pub fn record(&mut self, grid: &mut DeepGrid, mem: MemberCtx<'_>) {
        let parallel = self.par();
        let sea = self.sea_level;
        let chapter = self.cur_chapter;
        // **The formation context of this geological day** (P11 slice 1). Lifted
        // before the record is borrowed mutably: `lat_deg` takes `&self`, and the
        // per-cell temperature is the same air-temperature model the biotic gate
        // and the frost agent read (one climate, never a second).
        let w = grid.w;
        let lat: Vec<f64> = (0..w).map(|gy| grid.lat_deg(gy)).collect();
        let (r, h, precip, energy) = (&grid.r, &grid.h, &grid.precip, &self.energy);
        // The reference the energy bands are relative to — the coefficient the
        // capacities in `energy` were actually produced with, carried from
        // `transport` rather than re-read from a config this phase does not take.
        let k_t = self.k_transport;
        let dh = &self.dh;
        let dep = &self.dep_sp;
        let creep = &self.creep_sp;
        let sorted = self.sorted;
        // **The two halves of a unit's identity, in order** (P11 slice 1):
        //
        // 1. the CLASS is what the movers determine — the argmax of everything
        //    that arrived (`arriving_species`), or the tag's own lithology where
        //    nothing rode a mover. That is still `Litho`-grade because the
        //    transport budgets are (slice 2 re-grades them);
        // 2. the MEMBER is what the environment determines — fitness over the
        //    registered members of that class, under this cell's temperature and
        //    precipitation *this epoch*, through an addressed draw.
        //
        // What used to happen instead: (1) was recorded and (2) was invented at
        // expression, hundreds of millions of sim-years later, from the climate of
        // the chunk's centre. The class is a fact about the load; the member is a
        // fact about the day. Both are known here and neither was written down.
        let species_at = |i: usize, tag: DepTag| {
            let t = lithology::litho_of_tag(tag);
            let class = if sorted {
                let cr = if creep.is_empty() {
                    &[][..]
                } else {
                    &creep[i * SPECIES..(i + 1) * SPECIES]
                };
                arriving_species(dh[i], &dep[i * SPECIES..(i + 1) * SPECIES], cr, t)
            } else {
                t
            };
            let temp_c = f64::from(climate::air_temp_c(lat[i / w], r[i] + h[i]));
            mem.surface(
                i,
                temp_c,
                f64::from(precip[i]),
                class,
                dep_tags::TRANSPORT,
                0,
            )
        };
        let deposit_at = |i: usize| {
            let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea, k_t);
            (tag, species_at(i, tag))
        };
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                record_cell(s, dh[i], chapter, || deposit_at(i));
            });
        } else {
            for (i, s) in grid.strata.iter_mut().enumerate().take(self.n) {
                record_cell(s, dh[i], chapter, || deposit_at(i));
            }
        }
    }

    // ---- phase 10: eolian transport (SCALAR — per-row wind march) ---------
}
