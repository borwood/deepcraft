//! **The strata recorder phase** — the facies tagging (energy band, deposition
//! tag, arriving species) and the pass that commits each cell's surface change
//! into its deep-time record.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::super::climate;
use super::super::grid::DeepGrid;
use super::super::lithology;
use super::super::recorder::{
    Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, MemberCtx, dep_tags,
};
use super::super::species::{MAX_DEEP_SPECIES, SpeciesAxis};
use super::transport::{ENERGY_LOW_MED, ENERGY_MED_HIGH, REFERENCE_KT};
use super::Erosion;
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

/// **What the movers delivered here, and whether they outvoted the remainder** —
/// the argmax of the arriving mixture (Movement 2b; re-graded to materials by P11
/// slice 2).
///
/// The cell's net gain has several sources, and **two of them carry an identity**:
/// the fluvial transport pass knows, per species, exactly what it set down
/// (`dep`), and hillslope creep knows, per species, exactly what came down the
/// slope into this cell (`creep`, Movement 2b continuation (b)). What is left in
/// `dh` — bedrock weathered to regolith in place, wind and wave, the biotic layer
/// — never rode any mover, and is the *incumbent*: the answer the record gives
/// when nothing that arrived outweighs it.
///
/// **The two movers are summed, not ranked.** A cell that receives half a metre of
/// sandstone from upstream and half a metre of the same rock off the slope above
/// has a metre of that rock, and pretending the two halves compete would make the
/// answer depend on which agent we asked first.
///
/// **Only *gains* are candidates**: a species creep took **away** from this cell is
/// not something the cell can be made of, so the negative entries are clamped out
/// of both the mixture and the remainder.
///
/// ## The tie rule, spelled out because it is now load-bearing
///
/// The scan is a **strict `>` over the union of the two rows walked in axis
/// order**, so the first-met maximum wins and the incumbent holds any tie with it.
/// Axis order is **descending settling energy, ties broken by `MaterialId`
/// index** — a total, deterministic, content-derived order (`species.rs`), not an
/// enum's declaration order and not the order the rows happen to be stored in.
/// Two materials with exactly equal arriving mass therefore resolve to the
/// **coarser** one, which is the same bias the deposition arithmetic above it has
/// (coarsest-first drawdown, the falling competence ceiling).
///
/// **STUB #25 — which *mover* delivered it is a different axis, and the record does
/// not carry one.** Colluvium and alluvium are separable only by signature, not by
/// label; the byte that would fix it costs ~42 MiB at today's `DepUnit` layout, and
/// the free version is a packed `(species, mover)` byte. See `stubs.md` § 25 and
/// `examples/colluvium_probe.rs`, which measures the signature the label is missing.
///
/// This is deliberately *not* a threshold on "was most of this transported" —
/// a threshold would be a second rule with a number in it. It is one comparison
/// over one mixture, and it degenerates exactly to the old behaviour when nothing
/// was carried here.
#[inline]
fn arriving_material(
    dh: f64,
    axis: &SpeciesAxis,
    dep_axis: &[u8],
    dep: &[f64],
    creep_axis: &[u8],
    creep: &[f64],
) -> Option<MaterialId> {
    let mut mix = [0.0f64; MAX_DEEP_SPECIES];
    let mut carried = 0.0;
    for (j, &k) in dep_axis.iter().enumerate() {
        let m = dep[j].max(0.0);
        mix[k as usize] += m;
        carried += m;
    }
    for (j, &k) in creep_axis.iter().enumerate() {
        let m = creep[j].max(0.0);
        mix[k as usize] += m;
        carried += m;
    }
    // The incumbent: what never rode a mover. A strict `>` against it means a tie
    // leaves the record saying what the environment implies, exactly as before.
    let mut best_m = dh - carried;
    let mut best: Option<usize> = None;
    for (k, &m) in mix.iter().take(axis.len()).enumerate() {
        if m > best_m {
            best_m = m;
            best = Some(k);
        }
    }
    best.map(|k| axis.material(k))
}

/// The metres a cell deposited this epoch — the audit's weight. Only positive
/// `dh` reaches the identity decision, so a negative reads as zero.
#[inline]
fn dh_at(dh: &[f64], i: usize) -> f64 {
    dh[i].max(0.0)
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
        let dep = self.dep_sp.vals();
        let creep = self.creep_sp.vals();
        let (axis, tlayout, clayout) = (&self.axis, &self.tlayout, &self.clayout);
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
        let audit = self.identity_audit;
        let species_at = |i: usize, tag: DepTag, prov: &mut u8| {
            let carried = if sorted {
                let (db, dks) = tlayout.row(i);
                let (cb, cks) = if creep.is_empty() {
                    (0, &[][..])
                } else {
                    clayout.row(i)
                };
                arriving_material(
                    dh[i],
                    axis,
                    dks,
                    &dep[db..db + dks.len()],
                    cks,
                    &creep[cb..cb + cks.len()],
                )
            } else {
                None
            };
            // **THE DEPOSITION DRAW IS RETIRED FOR TRANSPORTED DEPOSITS**
            // (P11 ruling 6, 2026-08-02). Identity is a *conserved quantity*
            // flowing through the mass arithmetic — erosion releases it, transport
            // carries it by id, deposition records what settled — so when a mover
            // outvoted the un-carried remainder there is nothing left to pick: the
            // rock that arrived IS the rock. The fitness envelope was selection
            // machinery for filling a class, and a filled class has no hole.
            //
            // Two cases still reach the draw, and both are genuine degeneracy
            // rather than missing information:
            //
            // 1. **the remainder won** — the metres were made here (bedrock
            //    weathered to regolith in place) or brought by a mover that
            //    carries no identity yet (wind, wave, the biotic layer). Nothing
            //    was transported, so the environment is the only witness;
            // 2. **a transformation edge** — basement a river quarried lands as
            //    coarse clastic detritus, and detrital peat/coal/charcoal lands as
            //    carbonaceous mud ([`lithology::deposited_transform`]). The
            //    *parent* does not determine which member of the destination the
            //    transformation makes; the conditions at the site do. So fitness
            //    runs on the destination class, under this cell's own climate.
            //
            // FS-A retires case 1's weathered half (release spectra say what a
            // parent rock sheds); what survives after that is primary formation.
            let temp_c = f64::from(climate::air_temp_c(lat[i / w], r[i] + h[i]));
            let draw_class = match carried {
                Some(m) => match lithology::deposited_transform(m) {
                    None => {
                        // The instrument (off in production, byte-inert): would
                        // this site's own climate have named a different rock?
                        // Evaluating it is exactly the work the slice exists to
                        // stop doing, which is why it is a flag and not a phase.
                        *prov = if audit {
                            let would = mem.surface(
                                i,
                                temp_c,
                                f64::from(precip[i]),
                                super::super::lithology::Litho::of_material(m),
                                dep_tags::TRANSPORT,
                                0,
                            );
                            if would == m { 1 } else { 3 }
                        } else {
                            1
                        };
                        return m;
                    }
                    Some(to) => to,
                },
                None => lithology::litho_of_tag(tag),
            };
            *prov = 2;
            mem.surface(
                i,
                temp_c,
                f64::from(precip[i]),
                draw_class,
                dep_tags::TRANSPORT,
                0,
            )
        };
        let deposit_at = |i: usize, prov: &mut u8| {
            let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea, k_t);
            (tag, species_at(i, tag, prov))
        };
        let n = self.n;
        if audit {
            // The audit plane rides beside the record, written per cell (disjoint),
            // and is reduced **scalar afterwards** so the totals are
            // order-independent whichever driver ran.
            let id_class = &mut self.id_class;
            id_class.fill(0);
            if parallel {
                grid.strata
                    .par_iter_mut()
                    .zip(id_class.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (s, prov))| {
                        record_cell(s, dh[i], chapter, || deposit_at(i, prov));
                    });
            } else {
                for (i, (s, prov)) in grid
                    .strata
                    .iter_mut()
                    .zip(id_class.iter_mut())
                    .enumerate()
                    .take(n)
                {
                    record_cell(s, dh[i], chapter, || deposit_at(i, prov));
                }
            }
            for i in 0..n {
                let k = self.id_class[i] as usize;
                if k != 0 {
                    // Index 3 is a subset of index 1: a transported metre whose
                    // identity the site's draw would not have produced is still a
                    // transported metre.
                    self.id_m[if k == 3 { 1 } else { k }] += dh_at(&self.dh, i);
                    if k == 3 {
                        self.id_m[3] += dh_at(&self.dh, i);
                    }
                }
            }
            return;
        }
        let mut sink = 0u8;
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                record_cell(s, dh[i], chapter, || deposit_at(i, &mut 0));
            });
        } else {
            for (i, s) in grid.strata.iter_mut().enumerate().take(n) {
                record_cell(s, dh[i], chapter, || deposit_at(i, &mut sink));
            }
        }
    }

    // ---- phase 10: eolian transport (SCALAR — per-row wind march) ---------
}
