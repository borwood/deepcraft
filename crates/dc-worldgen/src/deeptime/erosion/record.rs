//! **The strata recorder phase** — the facies tagging (energy band, deposition
//! tag, arriving species) and the pass that commits each cell's surface change
//! into its deep-time record.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::super::climate;
use super::super::flux::FlowCause;
use super::super::grid::DeepGrid;
use super::super::lithology;
use super::super::recorder::{
    Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, MOVER_NONE, MemberCtx, dep_tags,
};
use super::super::species::{MAX_DEEP_SPECIES, SpeciesAxis};
use super::Erosion;
use super::transport::{ENERGY_LOW_MED, ENERGY_MED_HIGH, REFERENCE_KT};
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
    epoch: u8,
    deposit: impl FnOnce() -> (DepTag, MaterialId, u8),
) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        let (tag, species, mover) = deposit();
        s.deposit_moved(tag, dh, chapter, epoch, species, mover);
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
/// **The mover axis — `stubs.md` § 25, DISCHARGED 2026-08-02/03 (P11 slice 3).** The
/// record DOES carry one: every unit holds 3 mover bits, `MOVER_NONE` meaning *made in
/// place*, and it is **in the merge key**. This function derives the dominant arriving
/// mover from the same argmax that names the rock; wind writes `Eolian`, waves
/// `Marine`, pedogenesis and diagenesis `NONE`. The caveat that remains is that it is a
/// **plurality verdict, not per-metre provenance** — `carried` is computed here and
/// discarded, so the record keeps *which* mover dominated and loses *by how much*.
/// `examples/colluvium_probe.rs` measures the colluvium/alluvium signature.
///
/// ⚠ **This comment said the opposite until 2026-08-05** — it described #25 as unbuilt,
/// three days after the byte shipped, and a session reading it published the wrong
/// conclusion in a read-first artifact before an independent trace caught it. **A-2**,
/// and the two-directional-pointer lesson: `stubs.md` knew it was discharged; the code
/// it discharged did not.
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
) -> Option<(MaterialId, u8)> {
    let mut mix = [0.0f64; MAX_DEEP_SPECIES];
    // The fluvial half alone, so the winner's dominant mover is derivable
    // (creep's share of species `k` is `mix[k] - mix_dep[k]`). Purely a
    // *report* about the same argmax — the species answer is unchanged.
    let mut mix_dep = [0.0f64; MAX_DEEP_SPECIES];
    let mut carried = 0.0;
    for (j, &k) in dep_axis.iter().enumerate() {
        let m = dep[j].max(0.0);
        mix[k as usize] += m;
        mix_dep[k as usize] += m;
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
    // The winning species' dominant mover: fluvial vs hillslope gravity, by the
    // same ≥ bias the arithmetic above has (water outranks the slope on a tie —
    // an arbitrary but stated rule; the two are equal-mass ties only).
    best.map(|k| {
        let mover = if mix_dep[k] >= mix[k] - mix_dep[k] {
            FlowCause::Fluvial as u8
        } else {
            FlowCause::Gravity as u8
        };
        (axis.material(k), mover)
    })
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
        // The deposition clock (O-2b): the raw tick the epoch loop stamped via
        // `set_epoch`, written into every unit this pass deposits.
        let epoch = self.cur_epoch;
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
        // **A unit's identity, and where it comes from** (P11 slice 1, re-graded
        // by slice 2's ruling 6).
        //
        // Slice 1 wrote this as two halves in order: the movers determine the
        // CLASS, the environment determines the MEMBER by fitness. That was the
        // honest shape *while the budgets were class-wide* — the movers could not
        // say which rock, only which kind. They can now. So the halves collapsed:
        // where a mover outvoted the un-carried remainder, **the arriving
        // composition IS the identity** and no draw runs. The environment answers
        // only where nothing was transported, or where deposition genuinely
        // transforms the rock.
        //
        // What used to happen before either slice: the class was recorded and the
        // member was invented at expression, hundreds of millions of sim-years
        // later, from the climate of the chunk's centre.
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
            // (Status 2026-08-02: the spectra exist and the inventory-weathering
            // pass emits through them — this record-path draw is NOT yet retired;
            // that wire-up rides behind P11 slice 3's packed DepUnit.)
            let temp_c = f64::from(climate::air_temp_c(lat[i / w], r[i] + h[i]));
            let (draw_class, mover) = match carried {
                Some((m, mover)) => match lithology::deposited_transform(m) {
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
                        return (m, mover);
                    }
                    // A transformation edge is still a delivery: the mover that
                    // brought the parent is the mover of the product.
                    Some(to) => (to, mover),
                },
                // The un-carried remainder won: made where it lies.
                None => (lithology::litho_of_tag(tag), MOVER_NONE),
            };
            *prov = 2;
            (
                mem.surface(
                    i,
                    temp_c,
                    f64::from(precip[i]),
                    draw_class,
                    dep_tags::TRANSPORT,
                    0,
                ),
                mover,
            )
        };
        let deposit_at = |i: usize, prov: &mut u8| {
            let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea, k_t);
            let (species, mover) = species_at(i, tag, prov);
            (tag, species, mover)
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
                        record_cell(s, dh[i], chapter, epoch, || deposit_at(i, prov));
                    });
            } else {
                for (i, (s, prov)) in grid
                    .strata
                    .iter_mut()
                    .zip(id_class.iter_mut())
                    .enumerate()
                    .take(n)
                {
                    record_cell(s, dh[i], chapter, epoch, || deposit_at(i, prov));
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
                record_cell(s, dh[i], chapter, epoch, || deposit_at(i, &mut 0));
            });
        } else {
            for (i, s) in grid.strata.iter_mut().enumerate().take(n) {
                record_cell(s, dh[i], chapter, epoch, || deposit_at(i, &mut sink));
            }
        }
    }

    // ---- phase 10: eolian transport (SCALAR — per-row wind march) ---------
}
