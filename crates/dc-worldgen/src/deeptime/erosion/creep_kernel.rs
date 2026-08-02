//! **The hillslope-creep operator kernel** — the per-cell *gather* functions
//! (each cell sums its own in/out edge fluxes from a frozen surface and a frozen
//! per-cell limiter, so scalar and parallel agree to the bit), the monotonicity
//! bound they must respect, the single explicit step that drives them over the
//! grid, and the species-conservation audit of that step.
//!
//! **Which side of the engine/pack partition this sits on** (north star): this
//! is the **engine-shaped primitive embedded inside a content pass** — spines
//! § S-10's field-solver gather. Extracting it is the deferred E4 arc; the rest
//! of `super` is pass/content logic on the plugin side, and this is the part
//! that is not.

use rayon::prelude::*;

use super::super::grid::{DeepConfig, DeepGrid};
use super::transport::split_by_shares;
use super::{Erosion, NEIGH4, SPECIES, coords_of, in_grid};

/// A per-cell erodibility multiplier, or the exact identity `1.0` when the
/// plane is empty (coupling off). `x * 1.0` is bit-exact for every finite `x`,
/// which is what makes the uncoupled path byte-identical.
#[inline]
pub(super) fn sus_at(sus: &[f64], i: usize) -> f64 {
    if sus.is_empty() { 1.0 } else { sus[i] }
}

/// The effective per-cell hillslope diffusivity.
///
/// **Composition order is deliberate and fixed** (journal/0029): the config
/// diffusivity is reduced first by the cell's biotic root-cohesion resistance
/// (S10 `resist`), then by the lithology's abrasion susceptibility. Rock first
/// in *meaning* — what the slope is made of — biology second, applied to the
/// slope biology actually lives on; but in *arithmetic* the biotic factor is
/// applied first and the lithic factor multiplied onto the right, because f64
/// multiplication is not associative and the order has to be pinned for
/// byte-identity. Written the other way round, turning coupling off would not
/// reproduce the S10 result bit for bit.
///
/// With both layers off this is exactly `diffusion`; with only biology on it is
/// exactly the S10 expression.
#[inline]
pub(super) fn eff_diff(diffusion: f64, resist: &[f32], sus: &[f64], i: usize) -> f64 {
    let biotic = if resist.is_empty() {
        diffusion
    } else {
        diffusion * (1.0 - f64::from(resist[i]))
    };
    if sus.is_empty() {
        biotic
    } else {
        biotic * sus[i]
    }
}

/// Net hillslope-diffusion thickness change at cell `i` (metres), gathered from
/// its four edges on the frozen surface with the frozen per-cell limiter
/// `scale`. Outflux edges (i higher) use `scale[i]` and the donor `i`'s effective
/// diffusivity; influx edges (neighbour higher) use the donor `j`'s `scale[j]`
/// and `j`'s effective diffusivity — exactly the flux the scatter form moved, so
/// the two conserve mass identically. Summation order is fixed (`NEIGH4`).
#[inline]
pub(super) fn diffuse_net_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    scale: &[f64],
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut net = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                net -= eff_diff(diffusion, resist, sus, i) * d * scale[i];
            } else if d < 0.0 {
                net += eff_diff(diffusion, resist, sus, j) * (-d) * scale[j];
            }
        }
    }
    net
}

/// **The per-edge coefficient at which an explicit 4-neighbour Laplacian stops
/// oscillating** — the bound the hillslope-transport operator sub-cycles to
/// respect (journal/0122).
///
/// # Where 1/8 comes from — derived, never tuned
///
/// Write one epoch of hillslope diffusion on the frozen surface as
/// `h_i ← h_i + a·Σ_j (s_j − s_i)` over the four cardinal neighbours, with `a`
/// the per-edge coefficient ([`eff_diff`]). The von Neumann amplification factor
/// is `g(k) = 1 − 2a(2 − cos k_x − cos k_y)`, so:
///
/// - `a ≤ 1/4` ⇒ `g ≥ −1`: **stable**, but the grid-scale (Nyquist) mode is
///   reflected with its amplitude intact — `g(π,π) = −1` is a period-2
///   flip-flop that never decays.
/// - `a ≤ 1/8` ⇒ `g ≥ 0`: **monotone**. No mode may change sign, so a
///   checkerboard cannot survive a step, let alone be created by one.
///
/// **What the shipped world actually sits at, since the config rate is not the
/// whole story.** `diffusion = 0.12` is 4 % *inside* this bound — but [`eff_diff`]
/// folds in the lithology's creep susceptibility, and peat is the softest thing in
/// the world, so the shipped grid's **peak effective coefficient is 0.261**: 2.1×
/// past. Under the calibration it is **12.60**, or **100.8× past**. That gap is the
/// whole defect: a 2.1× excursion on a handful of soft cells carrying 4.6 m of cover
/// produces a wobble the surface absorbs (shipped `conc(h)` ACF −0.10, no
/// checkerboard), while a 100.8× excursion everywhere carrying 41 m produces
/// −0.82 and a 40 m grid-scale residual.
///
/// *The two facts journal/0116 could not reconcile — "saturation alone is not
/// sufficient" and "the limiter is deaf to the step" — are the same fact read from
/// either side of this constant.* The coefficient decides that the grid-scale mode
/// flips sign; the limiter decides how far, by capping the export at the cell's
/// inventory, which is what turns a divergence into a finite period-2 limit cycle.
///
/// **This is why journal/0116's 4× time-step refinement read as a null.** It took
/// the coefficient 5.4 → 1.35, which is still **11× past the bound**; the
/// experiment was sound and the refinement was an order of magnitude too small to
/// reach the register it was testing. See `corrections.md` #72.
pub const CREEP_MAX_EDGE_COEFF: f64 = 0.125;

/// Per-cell diffusion outflux sum → limiter scale on the frozen surface (using
/// the donor cell's biotic-reduced effective diffusivity).
#[inline]
pub(super) fn diffuse_scale_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    h: f64,
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut out = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                out += eff_diff(diffusion, resist, sus, i) * d;
            }
        }
    }
    if out > h && out > 0.0 { h / out } else { 1.0 }
}

/// **Material-aware hillslope creep** (Movement 2b continuation (b),
/// `material-behavior.md` § 13.2 — the **gravity / mass-wasting** member of the
/// transport family). The per-species net thickness change at cell `i`, gathered
/// from exactly the same four edges, with exactly the same fluxes, as
/// [`diffuse_net_cell`].
///
/// **Colluvium is not sorted, and that is the point.** Creep is diffusive and
/// gravity-driven: it has no competence ceiling, no settling draw, no
/// coarsest-first. Every edge moves the **donor's whole composition in
/// proportion** — the near-surface window the `outcrop_shares` seam already reads
/// each epoch, which for a stripped column is honestly the bedrock beneath. So a
/// colluvial apron is *locally derived and poorly sorted*, against a fluvial
/// deposit's *far-travelled and sorted*, and that contrast is a real facies
/// distinction rather than a second copy of the river's rule.
///
/// **Why this cannot leak.** The edge flux is antisymmetric to the bit — cell `i`
/// computes `eff_diff(i)·(sᵢ − sⱼ)·scale[i]` and cell `j` computes
/// `eff_diff(i)·−(sⱼ − sᵢ)·scale[i]`, and IEEE-754 subtraction is exactly
/// antisymmetric — and **both endpoints split it by the same donor composition
/// through the same [`split_by_shares`]**, so what leaves `i` of a species is bit
/// for bit what arrives at `j`. No species is created or destroyed anywhere on the
/// grid, which is the creep analogue of journal/0110's per-species junction test
/// and is asserted as one.
///
/// **This is an attribution, never a mass authority.** The terrain still moves by
/// the scalar [`diffuse_net_cell`], unchanged and byte-identical; this vector only
/// says *what* the metres were made of. That separation is deliberate: identity
/// riding a second arithmetic could not perturb `H` even if it were wrong.
///
/// **Returns the gross traffic** through the cell — the sum of every edge flux, in
/// or out. That is the audit's denominator, and it has to be: a cell that sheds as
/// much as it gains has a net near zero with real material moving through it, and
/// dividing a rounding error by *that* would report a leak where there is only
/// cancellation.
#[expect(
    clippy::too_many_arguments,
    reason = "the diffusion kernel's own arity"
)]
#[inline]
pub(super) fn diffuse_species_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    scale: &[f64],
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
    shares: &[f64],
    out: &mut [f64],
    accumulate: bool,
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut gross = 0.0;
    if !accumulate {
        out.fill(0.0);
    }
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                let f = eff_diff(diffusion, resist, sus, i) * d * scale[i];
                let s = split_by_shares(f, &shares[i * SPECIES..(i + 1) * SPECIES]);
                for k in 0..SPECIES {
                    out[k] -= s[k];
                }
                gross += f;
            } else if d < 0.0 {
                let f = eff_diff(diffusion, resist, sus, j) * (-d) * scale[j];
                let s = split_by_shares(f, &shares[j * SPECIES..(j + 1) * SPECIES]);
                for k in 0..SPECIES {
                    out[k] += s[k];
                }
                gross += f;
            }
        }
    }
    gross
}

/// The number of edges cell `i` **sends** creep across this epoch — the face count
/// a gravity-caused [`super::super::flux::FluxEntry`] would need if the flow record grew
/// the mass-wasting mover (stubs.md #18's `cause`). Counted rather than recorded,
/// because the count is the cost estimate that decides whether recording it is
/// affordable, and a probe that guessed it would be guessing the answer.
#[inline]
pub(super) fn diffuse_outflux_faces(i: usize, w: usize, surf: &[f64], scale: &[f64]) -> usize {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut n = 0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w)
            && si - surf[j] > 0.0
            && scale[i] > 0.0
        {
            n += 1;
        }
    }
    n
}

impl Erosion {
    /// One sub-step of the hillslope gather, at the sub-step diffusivity
    /// `diff`. `carry_on` is set for every sub-step after the first: the species
    /// itemisation then *accumulates* rather than overwrites, so the epoch's
    /// creep plane is the sum of its sub-steps and [`Self::record`] still reads
    /// one epoch's worth of arriving colluvium.
    pub(super) fn diffuse_step(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, diff: f64, carry_on: bool) {
        let parallel = self.par();
        let w = self.w;
        // Freeze the surface.
        self.build_surface(grid);
        // Pass 1: per-cell limiter scale on the frozen surface.
        {
            let surf = &self.surf;
            let scale = &mut self.scale;
            let h = &grid.h;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            if parallel {
                scale.par_iter_mut().enumerate().for_each(|(i, sc)| {
                    *sc = diffuse_scale_cell(i, w, surf, h[i], diff, resist, sus)
                });
            } else {
                for i in 0..self.n {
                    scale[i] = diffuse_scale_cell(i, w, surf, h[i], diff, resist, sus);
                }
            }
        }
        // Pass 2: gather net ΔH per cell.
        {
            let surf = &self.surf;
            let scale = &self.scale;
            let netdiff = &mut self.netdiff;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            if parallel {
                netdiff.par_iter_mut().enumerate().for_each(|(i, nd)| {
                    *nd = diffuse_net_cell(i, w, surf, scale, diff, resist, sus)
                });
            } else {
                for (i, nd) in netdiff.iter_mut().enumerate() {
                    *nd = diffuse_net_cell(i, w, surf, scale, diff, resist, sus);
                }
            }
        }
        // The other Movement 2b control: how much regolith **creep** moves, against
        // how much the rivers do. The gain side only — the net is zero by
        // construction, so summing it would report nothing.
        if self.sorted {
            self.ledger.diffused_m += self.netdiff.iter().map(|v| v.max(0.0)).sum::<f64>();
        }
        // journal/0111: of the creep above, how much crossed the shoreline and so
        // actually LEFT the land. Read-only, on the same frozen surface, before
        // the gather is applied. Off ⇒ not even called.
        if self.denude {
            self.tally_creep_to_sea(grid, cfg, diff);
            // journal/0114: and how often the limiter bound — the discretisation
            // honesty check on the calibration. `scale[i] < 1.0` is exactly "this
            // cell wanted to shed more than it had".
            //
            // **Both counters skip cells with no regolith, and that is the whole
            // point of the measurement.** `out > h` is trivially true at `h == 0`,
            // so a denominator of every cell would score the bare ocean floor and
            // every stripped ridge as "transport-limited" and report a saturation
            // that is really just an absence. The question is *of the cells that had
            // something to move, how many shipped all of it* — anything else is a
            // statistic about emptiness.
            for (i, sc) in self.scale.iter().enumerate() {
                if grid.h[i] > 0.0 {
                    self.ledger.creep_cell_epochs += 1;
                    if *sc < 1.0 {
                        self.ledger.creep_limited_cell_epochs += 1;
                    }
                }
            }
        }
        // Pass 3 (Movement 2b continuation (b)): **the same fluxes, carrying
        // identity.** Runs only when creep carries material; the terrain below is
        // applied from `netdiff` either way, so this pass cannot move a metre of
        // rock — it can only name the metres the pass above already moved.
        if self.creep_carries {
            let surf = &self.surf;
            let scale = &self.scale;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            let shares = &self.shares;
            let creep = &mut self.creep_sp;
            let gross = &mut self.creep_gross;
            if parallel {
                creep
                    .par_chunks_mut(SPECIES)
                    .zip(gross.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (out, g))| {
                        let f = diffuse_species_cell(
                            i, w, surf, scale, diff, resist, sus, shares, out, carry_on,
                        );
                        if carry_on {
                            *g += f;
                        } else {
                            *g = f;
                        }
                    });
            } else {
                for (i, (out, g)) in creep.chunks_mut(SPECIES).zip(gross.iter_mut()).enumerate() {
                    let f = diffuse_species_cell(
                        i, w, surf, scale, diff, resist, sus, shares, out, carry_on,
                    );
                    if carry_on {
                        *g += f;
                    } else {
                        *g = f;
                    }
                }
            }
            // Summed across sub-steps on purpose: a gravity-caused flow record
            // would pay one entry per face **per sub-step**, so the cost estimate
            // this counter exists to be (stubs.md #18) has to count them all.
            self.creep_faces += (0..self.n)
                .map(|i| diffuse_outflux_faces(i, w, &self.surf, &self.scale))
                .sum::<usize>();
        }
        // Apply: h += net, dh += net (disjoint per-cell writes).
        if parallel {
            grid.h
                .par_iter_mut()
                .zip(self.dh.par_iter_mut())
                .zip(self.netdiff.par_iter())
                .for_each(|((h, d), nd)| {
                    *h += *nd;
                    *d += *nd;
                });
        } else {
            for i in 0..self.n {
                grid.h[i] += self.netdiff[i];
                self.dh[i] += self.netdiff[i];
            }
        }
    }

    /// **The two creep-identity audits, taken every epoch** (Movement 2b
    /// continuation (b)).
    ///
    /// 1. **The itemisation equals its own total.** `Σ_species creep_sp[cell]` must
    ///    be the scalar `netdiff[cell]` the terrain moved. This is the check that
    ///    catches the failure this repo keeps catching — *a missing row in an
    ///    itemisation* — and it is the one that would fire if a species were ever
    ///    dropped from a split. It cannot be bit-exact because the two sums visit
    ///    the same terms in different orders (four edges of seven species against
    ///    seven species of four edges), so it is relative, scaled by the traffic
    ///    through the cell rather than by the net — a cell that gains as much as it
    ///    sheds has a net near zero and real material moving through it.
    /// 2. **No species is created or destroyed.** `Σ_cells creep_sp[·][s]` must be
    ///    zero: creep only *moves*. This is the global half, and it is where an
    ///    antisymmetry mistake between the two endpoints of an edge would land —
    ///    the direct analogue of journal/0110's `no_species_leaks_at_its_own
    ///    _junction`, taken over the whole grid because a diffusion junction has no
    ///    downstream order to walk.
    ///
    /// Running maxima rather than stored planes, for journal/0110's reason: a leak
    /// anywhere is a leak, and the per-cell per-species plane is not worth its
    /// megabytes to assert a scalar.
    pub(super) fn audit_creep_species(&mut self, sub_cycled: bool) {
        // The total the itemisation must equal. A sub-cycled epoch's creep plane
        // is the sum of its sub-steps, so it is audited against the summed ΔH —
        // never against the last sub-step's, which would report a leak of
        // everything the earlier sub-steps moved.
        let net_total: &[f64] = if sub_cycled {
            &self.netdiff_acc
        } else {
            &self.netdiff
        };
        let mut sum_s = [0.0; SPECIES];
        let mut abs_s = [0.0; SPECIES];
        let mut worst_item = self.creep_itemisation_residue;
        for (i, row) in self.creep_sp.chunks(SPECIES).enumerate() {
            let mut net = 0.0;
            for (k, &v) in row.iter().enumerate() {
                net += v;
                sum_s[k] += v;
                abs_s[k] += v.abs();
            }
            let gross = self.creep_gross[i];
            if gross > 0.0 {
                let rel = (net - net_total[i]).abs() / gross;
                if rel > worst_item {
                    worst_item = rel;
                }
            }
        }
        self.creep_itemisation_residue = worst_item;
        for k in 0..SPECIES {
            if abs_s[k] > 0.0 {
                let rel = sum_s[k].abs() / abs_s[k];
                if rel > self.creep_conservation_residue {
                    self.creep_conservation_residue = rel;
                }
            }
        }
    }

    // ---- phase 9: strata recorder (PARALLEL — per-cell independent) --------
}

// ---------------------------------------------------------------------------
// Movement 2b continuation (b) — the creep split. Its own module so the two
// slices' unit tests do not share a namespace.

#[cfg(test)]
mod creep_tests {
    use super::*;

    /// **The split closes exactly.** `Σ_species` of a composition split is the
    /// quantity that went in, to the bit — not to an epsilon. That is the whole
    /// content of the residual rule, and it is what makes the itemisation audit's
    /// tolerance a statement about *summation order* rather than about a leak we
    /// decided to tolerate.
    #[test]
    fn a_composition_split_closes_to_the_bit() {
        // Shares that do not sum to 1 in binary — the normal case for a
        // normalised f64 composition, and the reason the rule exists.
        let mut shares = [0.0; SPECIES];
        for (k, v) in [0.17, 0.03, 0.31, 0.0, 0.29, 0.11, 0.09]
            .into_iter()
            .enumerate()
        {
            shares[k] = v;
        }
        for total in [1.0f64, 3.7e-5, 2.4e3, 9.81e-12, 0.0] {
            let out = split_by_shares(total, &shares);
            let sum: f64 = out.iter().sum();
            assert_eq!(
                sum, total,
                "the split of {total} summed to {sum}; the last non-zero share is \
                 not taking the residual"
            );
        }
    }

    /// **A species with a zero share receives nothing**, whatever the residual
    /// rule does. A composition that says "there is no basement here" must not
    /// have basement fall out of the arithmetic at the end.
    #[test]
    fn an_absent_species_stays_absent() {
        let mut shares = [0.0; SPECIES];
        shares[1] = 0.5;
        shares[4] = 0.5;
        let out = split_by_shares(7.0, &shares);
        for (k, v) in out.iter().enumerate() {
            if shares[k] == 0.0 {
                assert_eq!(*v, 0.0, "species {k} materialised out of a zero share");
            }
        }
    }

    /// **An all-zero composition splits into nothing** — the honest answer when a
    /// cell has no identity to give. The alternative (spreading the quantity
    /// evenly, or handing it to species 0) would be fabricating provenance, which
    /// is strictly worse than recording none.
    #[test]
    fn an_unknown_composition_fabricates_no_identity() {
        let out = split_by_shares(42.0, &[0.0; SPECIES]);
        assert!(out.iter().all(|v| *v == 0.0), "{out:?}");
    }

    /// **CREEP DOES NOT SORT — and this is the test that would catch it starting
    /// to.**
    ///
    /// Colluvium is poorly sorted because gravity is not selective: a diffusive
    /// flux moves the donor's whole composition in proportion, with no competence
    /// ceiling and no settling draw. So the *only* thing that may decide how a
    /// creep flux resolves into species is the composition — never the settling
    /// velocity, never the grain size, never the quantity.
    ///
    /// Asserted as **scale invariance**: doubling the flux must double every
    /// species' share, exactly. A competence ceiling or a coarsest-first draw is by
    /// construction non-linear in the quantity (it thresholds, or it drains one
    /// species before touching the next), so any sorting rule that crept into this
    /// path would break this equality. It is arithmetic and therefore scale-free.
    #[test]
    fn the_creep_split_is_linear_in_the_quantity_so_nothing_is_sorted() {
        let mut shares = [0.0; SPECIES];
        for (k, v) in [0.4, 0.0, 0.05, 0.25, 0.2, 0.0, 0.1]
            .into_iter()
            .enumerate()
        {
            shares[k] = v;
        }
        let one = split_by_shares(1.0, &shares);
        let many = split_by_shares(1024.0, &shares);
        for k in 0..SPECIES {
            assert_eq!(
                many[k],
                one[k] * 1024.0,
                "species {k} did not scale with the flux — something in the creep \
                 path is selecting by grain size"
            );
        }
    }

    /// **The two endpoints of a creep edge see the same flux, to the bit.** The
    /// whole per-species conservation argument rests on IEEE-754 subtraction being
    /// exactly antisymmetric, so the donor's `sᵢ − sⱼ` and the receiver's
    /// `−(sⱼ − sᵢ)` are the same number and split the same way. If that ever
    /// stopped holding, every species would leak at every junction.
    #[test]
    fn a_surface_difference_is_exactly_antisymmetric() {
        for (a, b) in [
            (1234.5678901234, 1234.5678901233),
            (1e-300, 3e-300),
            (0.1, 0.2),
            (1e17, 1.0),
            (-4321.9, 8765.1),
        ] {
            assert_eq!(a - b, -(b - a), "({a}, {b}) broke the antisymmetry");
        }
    }
}
