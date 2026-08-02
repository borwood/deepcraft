//! **The transport chain** — entrainment, stream capacity, the competence
//! ceiling, deposition, and the species split that turns a metre of *something*
//! into metres of *named things*.
//!
//! Downstream-ordered and scalar by construction: the flux chain's summation
//! order is semantics here, so this phase never parallelises.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use super::super::grid::{DeepConfig, DeepGrid};
use super::super::lithology;
use super::creep_kernel::sus_at;
use super::mfd::{MFD_DIRS, MFD_DIST};
use super::{Erosion, NEIGH8, SPECIES, coords_of};

/// **The competence ceiling, per unit of transport capacity** — the one
/// calibration constant material-aware transport adds, in
/// `settle_energy`-units per (metre/iteration) of stream capacity, **stated
/// relative to `k_transport`**.
///
/// `material-behavior.md` § 13.5: *"sorting is the falling ceiling; we write the
/// ceiling, not the sort."* This is that ceiling. A flow of capacity `cap` can
/// hold in suspension every species whose settling velocity is at most
/// `COMPETENCE_PER_KT · k_transport · cap`; everything heavier rains out
/// **wherever it is**, regardless of whether the flow still has capacity to spare.
/// Capacity is the *total mass* limit and competence is the *size/density* limit,
/// and § 13.5 is explicit that both are needed: without competence a flow with
/// spare capacity carries boulders to the sea, and nothing ever fines downstream.
///
/// **Where the number comes from — and see corrections #59 for what was wrong with
/// the way that used to be said.** The *ratio* is fixed by an anchor that already
/// ships; the old doc comment stated that anchor as an **absolute capacity**, which
/// silently held `k_transport = 0.0016` inside it and called itself "not a tuning
/// knob" on the strength of a derivation from a constant that was itself about to
/// be recalibrated. [`energy_band`] calls a capacity of
/// `ENERGY_LOW_MED · k_transport / REFERENCE_KT` the Low/Medium boundary, and
/// `litho_of_tag` turns exactly that boundary into the coarse/fine clastic split —
/// so that boundary is *already* the world's stated "energy at which sand stops
/// moving". `settle_energy` puts the coarse-clastic reference sheet at `≈0.84`, and
/// at the historical `k_transport = 0.0016` the boundary is `0.002`, giving
/// `0.84 / 0.002 = 420`. The ceiling therefore crosses the coarse-clastic threshold
/// at precisely the capacity the shipped facies rule already crosses it at, and the
/// rest of the roster arranges itself around that.
///
/// It is **linear** in capacity because capacity is already a stream-power proxy
/// (`k·A^m·S^n`) and competence in a real channel scales with a power of stream
/// power; linear is the simplest form that spans the roster, in the same
/// plausible-not-tuned register as S9's physics constants. Chosen and written
/// **before** the outcome probe was run, and not revisited after.
///
/// **RE-EXPRESSED relative to `k_transport` 2026-07-26 (journal/0114), and the
/// value it produces at the historical `k_transport` is unchanged to the bit.**
/// `420` was `0.84 / 0.002`, and `0.002` was a boundary written in absolute
/// capacity because `k_transport` had never moved. The joint calibration moves it,
/// and left as an absolute this constant would have made every calibrated flow
/// competent to carry basement — sorting would vanish, "does sand move" would read
/// yes for a reason that is an artifact, and the facies gradient would be destroyed
/// rather than measured. The relative form says what was always meant: the ceiling
/// is a statement about **where in a drainage network you are**, not about the
/// value of a rate constant.
const COMPETENCE_SCALE: f64 = 420.0;

/// **The `k_transport` every threshold in this section was written against.**
///
/// `COMPETENCE_SCALE`, [`ENERGY_LOW_MED`] and [`ENERGY_MED_HIGH`] are numbers in
/// units of `k_transport · A^m · S^n`, and they were chosen when `k_transport` was
/// `0.0016` and had never moved. Naming that value is what lets the calibration
/// scale the coefficient and carry the thresholds along instead of silently
/// re-labelling the world.
///
/// **Every ratio below is formed as `k / REFERENCE_KT`, deliberately**: `x / x` is
/// exactly `1.0` in IEEE-754, so at the historical coefficient each threshold is
/// bit-identical to the absolute constant it replaced, and the pre-calibration
/// world reproduces its goldens to the bit.
pub(super) const REFERENCE_KT: f64 = 0.0016;

/// **The Low/Medium energy boundary** at [`REFERENCE_KT`] — the capacity at which
/// the shipped facies rule says sand stops moving. Scaled with `k_transport` by
/// [`energy_band`]; see [`REFERENCE_KT`].
pub(super) const ENERGY_LOW_MED: f64 = 0.002;
/// **The Medium/High energy boundary** at [`REFERENCE_KT`] — the trunk threshold.
/// See [`ENERGY_LOW_MED`].
pub(super) const ENERGY_MED_HIGH: f64 = 0.02;

/// **The competence ceiling of a flow with transport capacity `cap`**, in a world
/// whose stream-transport coefficient is `k_transport` — the largest settling
/// velocity ([`lithology::settling_table`]) the flow can hold in suspension. See
/// [`COMPETENCE_SCALE`] for where the constant comes from and [`REFERENCE_KT`] for
/// why the second argument exists.
///
/// Public because the invariant *"nothing leaves a cell that the cell could not
/// carry"* is checked against it from outside, and a test that re-derived the
/// ceiling would be checking its own arithmetic rather than the pass's.
#[inline]
pub fn competence_ceiling(cap: f64, k_transport: f64) -> f64 {
    COMPETENCE_SCALE * (REFERENCE_KT / k_transport) * cap
}

/// **Split a bulk quantity into species by a composition, exactly** — the one
/// place a metre of *something* becomes metres of *named things* in this pass.
///
/// `shares` is a per-[`Litho`] composition (an [`outcrop_shares`] read, or the
/// bedrock composition below the record). Normalised `f64` shares do not sum to
/// `1` to the bit, so a plain `share × total` per species leaves a residue and the
/// itemisation stops equalling its own total. **The last non-zero share takes
/// `total − Σ(earlier)`** — journal/0109's residual rule, on the species axis.
///
/// **This is the anti-leak rule journal/0110 had to state twice, and it is why
/// there is one function rather than three.** The tempting shortcut — split the
/// total exactly *somewhere else*, then apportion species by fraction here —
/// makes every species round against a shared denominator: the total stays
/// perfect and each species drifts, invisibly and unattributably. Entrainment,
/// incision and hillslope creep all route through this, so no caller can invent
/// its own budget.
///
/// An all-zero composition returns all zeros: the caller moved bulk it has no
/// identity for, and fabricating one would be worse than recording none.
#[inline]
pub(super) fn split_by_shares(total: f64, shares: &[f64]) -> [f64; SPECIES] {
    let mut out = [0.0; SPECIES];
    let Some(last) = (0..SPECIES).rev().find(|&k| shares[k] > 0.0) else {
        return out;
    };
    let mut given = 0.0;
    for k in 0..SPECIES {
        let sh = shares[k];
        if sh <= 0.0 {
            continue;
        }
        let v = if k == last { total - given } else { sh * total };
        given += v;
        out[k] = v;
    }
    out
}

impl Erosion {
    /// Turn **material-aware transport** on (Movement 2b, `material-behavior.md`
    /// § 13.3–13.6). Off is the scalar-load solve, byte for byte, because every
    /// species vector stays empty and every branch that reads one is skipped.
    ///
    /// The settling order is computed once here: a stable descending sort of the
    /// species by [`lithology::settling_table`], ties broken by index so the draw
    /// order is deterministic across platforms.
    pub fn set_material_transport(&mut self, on: bool) {
        self.sorted = on;
        if !on {
            self.qs_sp = Vec::new();
            self.shares = Vec::new();
            self.dep_sp = Vec::new();
            return;
        }
        self.w_settle = lithology::settling_table();
        let mut order: [usize; SPECIES] = std::array::from_fn(|i| i);
        let w = self.w_settle;
        order.sort_by(|&a, &b| w[b].total_cmp(&w[a]).then(a.cmp(&b)));
        self.ws_order = order;
        if self.qs_sp.len() != self.n * SPECIES {
            self.qs_sp = vec![0.0; self.n * SPECIES];
            self.dep_sp = vec![0.0; self.n * SPECIES];
        }
    }

    /// Whether material-aware transport is on.
    #[inline]
    pub fn is_material_transport(&self) -> bool {
        self.sorted
    }

    /// The settling velocity of each species, indexed by [`Litho::index`] (all
    /// zero when material-aware transport is off).
    pub fn settling(&self) -> &[f64; SPECIES] {
        &self.w_settle
    }

    /// **The largest relative per-species split residue over the whole run**
    /// (`0.0` when material-aware transport is off, or when every split was exact).
    ///
    /// This is the *local* half of the per-species mass proof, the direct analogue
    /// of journal/0109's `the_partition_leaves_no_residue`: for every cell, every
    /// species and every epoch, the shares handed to the receivers summed to what
    /// the cell held. The *global* half — a share written to the record but never
    /// added to a neighbour — is the whole-world `Δ(ΣR + ΣH) == uplift + biotic`
    /// ledger, which this cannot see and which cannot see this.
    pub fn max_species_split_residue(&self) -> f64 {
        self.split_residue
    }

    /// **What the transport pass set down at each cell in the last epoch, per
    /// species** (`n × SPECIES` metres; empty when material-aware transport is
    /// off). The measurement surface for the downstream-fining gradient — and the
    /// authority [`Self::record`] names the arriving unit from.
    pub fn deposited_species(&self) -> &[f64] {
        &self.dep_sp
    }

    /// **The suspended load still in flight after the last transport**
    /// (`n × SPECIES` metres; empty when material-aware transport is off). Cell
    /// `c`'s slice is what it handed onward — the quantity the competence
    /// invariant is read against.
    pub fn load_species(&self) -> &[f64] {
        &self.qs_sp
    }

    /// The transport capacity seen at each cell in the last epoch (`cap`, the
    /// stream-power budget) — the flow's own energy, which is what sets its
    /// competence ceiling.
    pub fn energy(&self) -> &[f64] {
        &self.energy
    }

    /// The **per-cell load exchange** of the transport phase, factored out so the
    /// single-receiver and MFD chains share one arithmetic (they must: two copies
    /// of a mass budget is the drift flow.md § 3 exists to prevent).
    ///
    /// `s` is the energy slope the cell's flow descends and `floor` the elevation
    /// its bedrock may not be cut below. Returns the load handed onward. Writes
    /// `energy[c]` and `dh[c]` and mutates the cell's own `R`/`H` — never a
    /// neighbour's, which is what leaves the *routing* of the result to the caller.
    #[inline]
    fn exchange_cell(
        &mut self,
        grid: &mut DeepGrid,
        cfg: &DeepConfig,
        c: usize,
        qin: f64,
        s: f64,
        floor: f64,
    ) -> f64 {
        let ae = if (cfg.m_exp - 0.5).abs() < 1e-9 {
            self.area[c].sqrt()
        } else {
            self.area[c].powf(cfg.m_exp)
        };
        let sn = if (cfg.n_exp - 1.0).abs() < 1e-9 {
            s
        } else {
            s.powf(cfg.n_exp)
        };
        let cap = cfg.k_transport * ae * sn;
        self.energy[c] = cap;
        // The erodibility coupling's fluvial multiplier for this cell: the
        // abrasion susceptibility of whatever lithology outcrops here.
        // Exactly `1.0` when the coupling is off.
        let sus = sus_at(&self.sus_flow, c);
        let base = c * SPECIES;
        if qin <= cap {
            let mut room = cap - qin;
            // Entrain the exposed cover. Transport-limited, now scaled by
            // how detachable that rock is: a weak mudstone hands the flow
            // everything it can carry, a competent sandstone hands over less
            // than the flow has room for and the difference is what leaves a
            // resistant bed standing proud. This is where bed-to-bed
            // differential erosion lives (see `expose`).
            //
            // `sus > 1` (rock softer than the fine-clastic reference) can
            // push entrainment past this cell's remaining capacity; that is
            // physical and mass-safe — the excess is routed downstream as
            // suspended load and the receiver, seeing `qin > cap`, deposits
            // it. Over-entrainment also drives `room` negative, which skips
            // incision: a thick soft cover shields the bedrock beneath it,
            // which is correct.
            // `H` can carry a sub-ULP negative from round-off; on the scalar path
            // that has always flowed straight through (and `x.max(0.0)` would not
            // be byte-identical), but a *negative entrainment* would mean handing
            // the load a negative quantity of a named material, which is not a
            // thing. Floored only on the sorted path, where `avail` is otherwise
            // the same expression bit for bit.
            let avail = if self.sorted {
                grid.h[c].max(0.0)
            } else {
                grid.h[c]
            };
            let ent = avail.min(room * sus);
            grid.h[c] -= ent;
            self.dh[c] -= ent;
            room -= ent;
            // **Entrainment is where identity enters the load** (§ 13.6: the
            // `loose→load` removal at the source). What comes off is what is lying
            // here — the record's own near-surface composition — so a reach cutting
            // a sandstone bench hands the flow sand and a stripped column hands it
            // basement debris. The last non-zero share takes the residual, so the
            // split is exact by construction rather than exact-to-an-ulp: the same
            // discipline the MFD face split needs, for the same reason.
            if self.sorted && ent > 0.0 {
                self.ledger.entrained_m += ent;
                let add = split_by_shares(ent, &self.shares[base..base + SPECIES]);
                for (k, a) in add.iter().enumerate() {
                    self.qs_sp[base + k] += a;
                }
            }
            let mut carried = qin + ent;
            // Then incise bedrock, shielded by remaining cover and scaled by
            // the same susceptibility. Where cover is thin enough for this
            // term to matter at all, the outcropping lithology is what the
            // flow is grinding — and on a stripped column that is basement.
            if room > 0.0 {
                let shield = (-grid.h[c] / cfg.h_star).exp();
                let inc_pot = cfg.k_bedrock * ae * sn * shield * sus;
                let max_inc = (grid.r[c] - floor).max(0.0);
                let inc = inc_pot.min(room).min(max_inc);
                grid.r[c] -= inc;
                carried += inc;
                // Incision detaches material from **below the record**, and the
                // composition seam answers that question too — an empty section is
                // whatever lies beneath the pile (`bedrock_sp`). Nothing is named
                // here; on today's world the answer comes back as the hardest,
                // coarsest thing there is, which is why a headwater reach cutting
                // rock rather than reworking cover puts gravel into the load.
                if self.sorted && inc > 0.0 {
                    self.ledger.incised_m += inc;
                    let add = split_by_shares(inc, &self.bedrock_sp);
                    for (k, a) in add.iter().enumerate() {
                        self.qs_sp[base + k] += a;
                    }
                }
            }
            if self.sorted {
                self.settle_above_competence(grid, c, cap, cfg.k_transport);
                self.qs_sp[base..base + SPECIES].iter().sum()
            } else {
                carried
            }
        } else {
            // Over capacity: deposit the excess as alluvium.
            let dep = qin - cap;
            if self.sorted {
                // **Coarsest first.** The load is drawn down in descending settling
                // velocity, so the excess the flow cannot hold is paid for out of
                // the heaviest fraction it is carrying — a bar of gravel and sand,
                // not an average of everything in the water. The fines that survive
                // this cell are what reaches the next one, and *that* is the
                // downstream gradient: a distal cell deposits mud because there is
                // nothing else left, not because it is a low-energy place.
                let mut remaining = dep;
                let mut placed = 0.0;
                for &k in &self.ws_order {
                    if remaining <= 0.0 {
                        break;
                    }
                    let have = self.qs_sp[base + k];
                    if have <= 0.0 {
                        continue;
                    }
                    let take = if have <= remaining {
                        self.qs_sp[base + k] = 0.0;
                        have
                    } else {
                        self.qs_sp[base + k] = have - remaining;
                        remaining
                    };
                    self.dep_sp[base + k] += take;
                    placed += take;
                    remaining -= take;
                }
                grid.h[c] += placed;
                self.dh[c] += placed;
                self.ledger.deposited_by_capacity_m += placed;
                self.settle_above_competence(grid, c, cap, cfg.k_transport);
                self.qs_sp[base..base + SPECIES].iter().sum()
            } else {
                grid.h[c] += dep;
                self.dh[c] += dep;
                cap
            }
        }
    }

    /// **COMPETENCE — the falling ceiling** (Movement 2b, `material-behavior.md`
    /// § 13.5). Every species in cell `c`'s outgoing load whose settling velocity
    /// exceeds what a flow of capacity `cap` can hold is set down **here**, however
    /// much capacity the flow has to spare.
    ///
    /// Capacity says *how much* a flow can carry; competence says *what*. Only the
    /// second one produces a facies: without it a stream with spare capacity would
    /// carry boulders to the sea and nothing would ever fine downstream. § 13.5's
    /// instruction is exact — *"sorting is the falling ceiling; we write the
    /// ceiling, not the sort"* — and this is the whole of the sort: no list is
    /// reordered anywhere, the ceiling simply falls as energy falls and the load is
    /// whatever is still under it.
    ///
    /// It runs **last**, on the load actually leaving, so it covers the material
    /// this cell just entrained or incised as well as what arrived: a reach that
    /// prises loose a grain size it cannot lift drops it straight back, which is
    /// the honest outcome and keeps the invariant clean — **nothing leaves a cell
    /// that the cell could not carry**. (That is deliberately *not* armouring: the
    /// grain stays in the ordinary loose cover and is tried again next epoch, so no
    /// permanent lag or pavement forms. Selective *entrainment* — the fine-side,
    /// cohesion-driven half of Hjulström's curve, which is what actually armours a
    /// bed — is § 13.4 and is deferred.)
    #[inline]
    fn settle_above_competence(&mut self, grid: &mut DeepGrid, c: usize, cap: f64, k_t: f64) {
        let ceiling = competence_ceiling(cap, k_t);
        let base = c * SPECIES;
        let mut rained = 0.0;
        for k in 0..SPECIES {
            if self.w_settle[k] > ceiling {
                let m = self.qs_sp[base + k];
                if m > 0.0 {
                    self.qs_sp[base + k] = 0.0;
                    self.dep_sp[base + k] += m;
                    rained += m;
                }
            }
        }
        if rained > 0.0 {
            grid.h[c] += rained;
            self.dh[c] += rained;
            self.ledger.deposited_by_competence_m += rained;
        }
    }

    /// Stream-power transport with cover shielding and explicit flux routing.
    /// Suspended load flows down the receiver chain (`qs[rc] += qs_out`), so a
    /// cell needs its full upstream load before it runs — a serial chain, kept
    /// scalar for byte-identity. Zeroes `dh` (the recorder's net-ΔH scratch).
    ///
    /// ## The two invariants MFD had to re-derive
    ///
    /// **Mass down a DAG.** The single-receiver chain conserved mass because each
    /// cell's `qs_out` had exactly one destination. Under MFD it has several, and
    /// the budget survives on one condition: **the shares sum to the whole, with no
    /// residue.** Normalised `f64` weights do not sum to `1` to the bit, so the
    /// last weighted direction takes `qs_out − Σ(earlier shares)` rather than
    /// `w · qs_out`. That makes the split *exact* by construction rather than
    /// exact-to-an-ulp per hop, and a per-hop ulp compounds down a thousand-cell
    /// chain. Everything above the split — entrainment, incision, deposition — is
    /// untouched, which is why one `exchange_cell` serves both paths.
    ///
    /// **The never-incise-below-the-receiver clamp** becomes *below the **lowest**
    /// receiver*. The clamp exists so no runaway knickpoint digs a hole its own
    /// outlet cannot drain — and with several outlets, the cell still drains as
    /// long as it stays above the lowest of them. Cutting to the *highest* would be
    /// arbitrarily stricter; cutting past the lowest makes the cell a pit. In the
    /// single-receiver limit the D8 receiver **is** the lowest neighbour, so the
    /// generalisation reduces to today's rule exactly rather than approximately.
    ///
    /// **The energy slope becomes the share-weighted mean** `Σ w_k · S_k`. Stream
    /// power is `Q·S`; split the discharge and the total power released is
    /// `Σ Q_k·S_k = Q·Σ w_k S_k`. So the weighted mean is not a smoothing choice —
    /// it is the slope that keeps the cell's energy budget equal to the sum of the
    /// budgets of the flows leaving it.
    pub fn transport(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        // The coefficient this epoch's capacities are built from, kept so the
        // recorder can classify them against the same reference (journal/0114).
        self.k_transport = cfg.k_transport;
        self.dh.iter_mut().for_each(|d| *d = 0.0);
        self.qs.iter_mut().for_each(|q| *q = 0.0);
        self.energy.iter_mut().for_each(|e| *e = 0.0);
        // Flow-record side-buffers (empty ⇒ inert). Written, never read, by the
        // transport chain — they cannot perturb an f64 anywhere.
        self.out_load.iter_mut().for_each(|q| *q = 0.0);
        self.out_face_load.iter_mut().for_each(|q| *q = 0.0);
        self.qs_sp.iter_mut().for_each(|q| *q = 0.0);
        self.dep_sp.iter_mut().for_each(|q| *q = 0.0);
        let record = !self.out_face_load.is_empty();
        let sorted = self.sorted;
        if self.mfd.is_none() {
            for k in (0..self.order.len()).rev() {
                let c = self.order[k] as usize;
                let rc = self.recv[c];
                let base = c * SPECIES;
                let qin = if sorted {
                    self.qs_sp[base..base + SPECIES].iter().sum()
                } else {
                    self.qs[c]
                };
                if rc < 0 {
                    // Sink: everything suspended settles here (marine / border).
                    grid.h[c] += qin;
                    self.dh[c] += qin;
                    self.energy[c] = 0.0;
                    if sorted {
                        for s in 0..SPECIES {
                            self.dep_sp[base + s] += self.qs_sp[base + s];
                            self.qs_sp[base + s] = 0.0;
                        }
                        self.ledger.deposited_at_sink_m += qin;
                    }
                    self.tally_sink(grid, c, qin);
                    continue;
                }
                let rc = rc as usize;
                let s = ((self.filled[c] - self.filled[rc]).max(0.0)) / self.cell_m;
                let floor = grid.surf_at(rc);
                let qs_out = self.exchange_cell(grid, cfg, c, qin, s, floor);
                if sorted {
                    // One receiver: the whole multiset moves, species by species.
                    let rb = rc * SPECIES;
                    for s in 0..SPECIES {
                        self.qs_sp[rb + s] += self.qs_sp[base + s];
                    }
                } else {
                    self.qs[rc] += qs_out;
                }
                // The load crossing cell→receiver this epoch — the flow atom's `L`.
                // Captured here because it is unrecoverable afterwards: `qs[rc]` is a
                // sum over every contributor, with no unique factorization.
                if record {
                    self.out_load[c] = qs_out;
                    let (gx, gy) = coords_of(c, self.w);
                    let (jx, jy) = coords_of(rc, self.w);
                    let d = NEIGH8
                        .iter()
                        .position(|&(dx, dy)| (gx + dx, gy + dy) == (jx, jy))
                        .expect("the receiver is a D8 neighbour");
                    self.out_face_load[c * MFD_DIRS + d] = qs_out as f32;
                }
            }
            return;
        }
        for k in (0..self.order.len()).rev() {
            let c = self.order[k] as usize;
            let base = c * MFD_DIRS;
            let sbase = c * SPECIES;
            let qin = if sorted {
                self.qs_sp[sbase..sbase + SPECIES].iter().sum()
            } else {
                self.qs[c]
            };
            // One sweep over the partition for the three things the exchange needs:
            // is there an outlet at all, what slope does the flow descend, and how
            // low may the bed be cut.
            let mut s_bar = 0.0;
            let mut floor = f64::INFINITY;
            let mut last = usize::MAX;
            for (d, &dist) in MFD_DIST.iter().enumerate() {
                let wt = self.mfd_w[base + d];
                if wt <= 0.0 {
                    continue;
                }
                let j = self.mfd_neighbour(c, d);
                let drop = (self.filled[c] - self.filled[j]).max(0.0);
                s_bar += wt * (drop / (self.cell_m * dist));
                floor = floor.min(grid.surf_at(j));
                last = d;
            }
            if last == usize::MAX {
                // Sink: everything suspended settles here (marine / border).
                grid.h[c] += qin;
                self.dh[c] += qin;
                self.energy[c] = 0.0;
                if sorted {
                    for s in 0..SPECIES {
                        self.dep_sp[sbase + s] += self.qs_sp[sbase + s];
                        self.qs_sp[sbase + s] = 0.0;
                    }
                    self.ledger.deposited_at_sink_m += qin;
                }
                self.tally_sink(grid, c, qin);
                continue;
            }
            let qs_out = self.exchange_cell(grid, cfg, c, qin, s_bar, floor);
            if record {
                self.out_load[c] = qs_out;
            }
            if !sorted {
                let mut given = 0.0;
                for d in 0..MFD_DIRS {
                    let wt = self.mfd_w[base + d];
                    if wt <= 0.0 {
                        continue;
                    }
                    let share = if d == last {
                        qs_out - given
                    } else {
                        wt * qs_out
                    };
                    given += share;
                    let j = self.mfd_neighbour(c, d);
                    self.qs[j] += share;
                    if record {
                        self.out_face_load[base + d] = share as f32;
                    }
                }
                continue;
            }
            // **The residual trick, PER SPECIES** — re-derived, not ported.
            //
            // journal/0109's scalar rule ("the last weighted direction takes
            // `q − Σ(earlier)`") makes one split exact. The obvious generalisation
            // — split the *total* exactly and then apportion each species by its
            // fraction of the total — is **wrong**, and wrong in the silent
            // direction: it re-derives the per-species amount from a shared
            // quantity, so each species picks up its own rounding against a common
            // denominator and the sum of a species over all its faces no longer
            // equals what the cell held. The leak is per-species, invisible in the
            // total, and unattributable afterwards because `qs_sp[j][s]` is a sum
            // over contributors with no unique factorisation. So each species runs
            // its **own** budget: for species `s`, the last weighted direction
            // takes `q_s − Σ(earlier shares of s)`, and every species is exact on
            // its own terms.
            //
            // The face scalar the flux record stores is then the **sum of the
            // species shares on that face** — not a second split of the total. One
            // arithmetic, so the record and the budget cannot disagree (flow.md
            // § 3), exactly as 0109 required of the scalar version.
            let mut face_total = [0.0f64; MFD_DIRS];
            for s in 0..SPECIES {
                let q_s = self.qs_sp[sbase + s];
                if q_s <= 0.0 {
                    continue;
                }
                let mut given = 0.0;
                for (d, ft) in face_total.iter_mut().enumerate() {
                    let wt = self.mfd_w[base + d];
                    if wt <= 0.0 {
                        continue;
                    }
                    let share = if d == last { q_s - given } else { wt * q_s };
                    given += share;
                    *ft += share;
                    let j = self.mfd_neighbour(c, d);
                    self.qs_sp[j * SPECIES + s] += share;
                }
                // The audit: what the receivers were handed, against what the cell
                // held. Relative, because loads span many orders of magnitude.
                let residue = ((given - q_s) / q_s).abs();
                if residue > self.split_residue {
                    self.split_residue = residue;
                }
            }
            if record {
                for (d, ft) in face_total.iter().enumerate() {
                    if self.mfd_w[base + d] > 0.0 {
                        self.out_face_load[base + d] = *ft as f32;
                    }
                }
            }
        }
    }

    // ---- phase 7: bedrock weathering (PARALLEL — per-cell independent) -----
}
