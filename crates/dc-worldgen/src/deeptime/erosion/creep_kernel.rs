//! **The content side of the hillslope-creep kernel call** — creep's declared
//! coefficient field ([`CreepCoeff`] over [`eff_diff`]), the per-sub-step
//! orchestration ([`Erosion::diffuse_step`]), the species split that rides the
//! same edge fluxes, and the species-conservation audit.
//!
//! **The operator itself is EXTRACTED (E4-1, 2026-08-03).** The gather
//! arithmetic, the monotonicity bound and the sub-cycle derivation this file
//! used to embed live in [`dc_core::field`] now — `FieldKernel`, spines
//! § S-10's first engine field-solver primitive (**⚠ NEEDS RATIFICATION —
//! venue**, audit pick U-3: `docs/audits/2026-08-03-e4-implicit-kernel-design.md`
//! § 3.5). What remains here is exactly what the partition assigns to content:
//! the coefficient's composition (biotic × lithology, journal/0029), what to do
//! with the fluxes (the species split, the tallies), and the audits.

use rayon::prelude::*;

use dc_core::field::{CoeffField, ExplicitPlan, FieldKernel, Stencil, StepScratch};

use super::super::grid::{DeepConfig, DeepGrid};
use super::super::species::{SpeciesLayout, csr_rows_mut, split_row_into};
use super::{Erosion, NEIGH4, coords_of, in_grid};

/// The kernel creep declares against: explicit scheme, 4-neighbour stencil —
/// today's operator, byte for byte, behind the E4 API.
pub(super) const CREEP_KERNEL: FieldKernel = FieldKernel::explicit(Stencil::FourNeighbour);

/// **Creep's coefficient field, as the pass declares it** — the E4 `CoeffField`
/// wrapper over [`eff_diff`]. The kernel evaluates it at its own sub-divided
/// rate; the composition (rate × biotic × lithology, in that pinned order) is
/// content physics and stays here.
pub(super) struct CreepCoeff<'a> {
    pub resist: &'a [f32],
    pub sus: &'a [f64],
}

impl CoeffField for CreepCoeff<'_> {
    #[inline]
    fn at(&self, rate: f64, i: usize) -> f64 {
        eff_diff(rate, self.resist, self.sus, i)
    }
}

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

// **`diffuse_net_cell`, `diffuse_scale_cell` and `CREEP_MAX_EDGE_COEFF` LIVED
// HERE AND ARE EXTRACTED** (E4-1, 2026-08-03). The gather arithmetic is
// `dc_core::field::gather` (the net re-expressed as antisymmetric edge fluxes
// + a fixed-order gather, bit-identically -- asserted at fixture scale by
// `field/tests.rs::the_flux_form_gather_is_the_embedded_gather_to_the_bit` and
// at world scale by the golden suite staying green); the bound, with its von
// Neumann derivation and the journal/0116/0122 history, is
// `dc_core::field::MONOTONE_MAX_EDGE_COEFF`, re-exported by `super` under its
// old name `CREEP_MAX_EDGE_COEFF` for the probes and tests that report against
// it.

/// **Material-aware hillslope creep** (Movement 2b continuation (b),
/// `material-behavior.md` § 13.2 — the **gravity / mass-wasting** member of the
/// transport family). The per-species net thickness change at cell `i`, gathered
/// from exactly the same four edges, with exactly the same fluxes, as the
/// kernel's own gather (`dc_core::field`, since E4-1).
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
/// the kernel's scalar net gather, unchanged and byte-identical; this vector only
/// says *what* the metres were made of. That separation is deliberate: identity
/// riding a second arithmetic could not perturb `H` even if it were wrong.
///
/// **Returns the gross traffic** through the cell — the sum of every edge flux, in
/// or out. That is the audit's denominator, and it has to be: a cell that sheds as
/// much as it gains has a net near zero with real material moving through it, and
/// dividing a rounding error by *that* would report a leak where there is only
/// cancellation.
///
/// **Sparse since P11 slice 2.** `out` is cell `i`'s row in the *creep* layout —
/// the window dilated by one 4-ring, so it holds everything cell `i` and its four
/// neighbours can donate — and the donor's composition is its own row in the
/// *window* layout. Both are ascending in axis order and the source is a subset of
/// the destination, so [`split_row_into`] is a merge walk. **A negative
/// contribution is written by subtracting the split's own output**, not by
/// splitting a negative quantity, so the two endpoints of an edge still see bit-
/// identical magnitudes.
pub(super) struct CreepEdges<'a> {
    pub w: usize,
    pub surf: &'a [f64],
    pub scale: &'a [f64],
    pub diffusion: f64,
    pub resist: &'a [f32],
    pub sus: &'a [f64],
    /// The window layout and its share plane — where a donor's composition lives.
    pub window: &'a SpeciesLayout,
    pub shares: &'a [f64],
    /// The creep layout — where `out` lives.
    pub clayout: &'a SpeciesLayout,
}

#[inline]
pub(super) fn diffuse_species_cell(
    i: usize,
    ctx: &CreepEdges<'_>,
    out: &mut [f64],
    accumulate: bool,
) -> f64 {
    let CreepEdges {
        w,
        surf,
        scale,
        diffusion,
        resist,
        sus,
        window,
        shares,
        clayout,
    } = *ctx;
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut gross = 0.0;
    if !accumulate {
        out.fill(0.0);
    }
    let (_, dst_axis) = clayout.row(i);
    // One scratch row per call, on the stack: the split writes here and is then
    // added into (or subtracted from) `out`, so a shed edge is the exact negation
    // of the gained edge at the other endpoint.
    let mut edge = [0.0f64; super::super::species::MAX_DEEP_SPECIES];
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d == 0.0 {
                continue;
            }
            let (donor, f) = if d > 0.0 {
                (i, eff_diff(diffusion, resist, sus, i) * d * scale[i])
            } else {
                (j, eff_diff(diffusion, resist, sus, j) * (-d) * scale[j])
            };
            let (sb, src_axis) = window.row(donor);
            let e = &mut edge[..dst_axis.len()];
            e.fill(0.0);
            split_row_into(f, src_axis, &shares[sb..sb + src_axis.len()], dst_axis, e);
            if d > 0.0 {
                for (o, v) in out.iter_mut().zip(e.iter()) {
                    *o -= *v;
                }
            } else {
                for (o, v) in out.iter_mut().zip(e.iter()) {
                    *o += *v;
                }
            }
            gross += f;
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
    /// One sub-step of the hillslope operator, at the plan's own sub-rate
    /// (E4-1: the limiter, the edge fluxes and the net gather are the kernel's;
    /// this method is the pass's per-sub-step orchestration — tallies, the
    /// species split, and the apply). `carry_on` is set for every sub-step
    /// after the first: the species itemisation then *accumulates* rather than
    /// overwrites, so the epoch's creep plane is the sum of its sub-steps and
    /// [`Self::record`] still reads one epoch's worth of arriving colluvium.
    pub(super) fn diffuse_step(
        &mut self,
        grid: &mut DeepGrid,
        cfg: &DeepConfig,
        plan: &ExplicitPlan,
        carry_on: bool,
    ) {
        let parallel = self.par();
        let w = self.w;
        let diff = plan.sub_rate();
        // Freeze the surface.
        self.build_surface(grid);
        // Kernel passes (dc_core::field, flux-form): the donor limiter plane,
        // the antisymmetric edge fluxes, and their net gather. The kernel never
        // applies — the pass owns the state and applies `netdiff` below, after
        // its own read-only instruments have seen the frozen epoch.
        {
            let coeff = CreepCoeff {
                resist: &grid.bio_resist,
                sus: &self.sus_creep,
            };
            CREEP_KERNEL.step(
                plan,
                &dc_core::field::DiffusionProblem {
                    w,
                    potential: &self.surf,
                    state: &grid.h,
                    coeff: &coeff,
                    parallel,
                },
                StepScratch {
                    scale: &mut self.scale,
                    fluxes: &mut self.creep_flux,
                    net: &mut self.netdiff,
                },
            );
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
            let Erosion {
                surf,
                scale,
                sus_creep,
                window,
                shares,
                clayout,
                creep_sp,
                creep_gross,
                ..
            } = self;
            let ctx = CreepEdges {
                w,
                surf,
                scale,
                diffusion: diff,
                resist: &grid.bio_resist,
                sus: sus_creep,
                window,
                shares: shares.vals(),
                clayout,
            };
            let gross = &mut creep_gross[..];
            let mut rows = csr_rows_mut(creep_sp.vals_mut(), clayout);
            if parallel {
                rows.par_iter_mut()
                    .zip(gross.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (out, g))| {
                        let f = diffuse_species_cell(i, &ctx, out, carry_on);
                        if carry_on {
                            *g += f;
                        } else {
                            *g = f;
                        }
                    });
            } else {
                for (i, (out, g)) in rows.iter_mut().zip(gross.iter_mut()).enumerate() {
                    let f = diffuse_species_cell(i, &ctx, out, carry_on);
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
        let aw = self.axis.len();
        let mut sum_s = vec![0.0f64; aw];
        let mut abs_s = vec![0.0f64; aw];
        let mut worst_item = self.creep_itemisation_residue;
        let vals = self.creep_sp.vals();
        for (i, (&total, &gross)) in net_total.iter().zip(self.creep_gross.iter()).enumerate() {
            let (b, ks) = self.clayout.row(i);
            let mut net = 0.0;
            for (j, &k) in ks.iter().enumerate() {
                let v = vals[b + j];
                net += v;
                sum_s[k as usize] += v;
                abs_s[k as usize] += v.abs();
            }
            if gross > 0.0 {
                let rel = (net - total).abs() / gross;
                if rel > worst_item {
                    worst_item = rel;
                }
            }
        }
        self.creep_itemisation_residue = worst_item;
        for k in 0..aw {
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

    /// A seven-wide dense row and the axis codes that address it — the shape the
    /// class-grade split used to take, re-expressed as a sparse row so the four
    /// laws below still say exactly what they said.
    const SPECIES: usize = 7;
    fn codes() -> [u8; SPECIES] {
        std::array::from_fn(|i| i as u8)
    }
    fn split_by_shares(total: f64, shares: &[f64]) -> [f64; SPECIES] {
        let mut out = [0.0; SPECIES];
        split_row_into(total, &codes(), shares, &codes(), &mut out);
        out
    }

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
