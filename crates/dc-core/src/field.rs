//! **The E4 field-solver primitive** — the frozen-snapshot, doubly-limited
//! diffusion gather (spines § S-10) as an engine kernel.
//!
//! Extracted 2026-08-03 (E4-1, byte-identically) from the hillslope-creep pass
//! that grew it (`dc-worldgen::deeptime::erosion`, journal/0122; the clean cut
//! journal/0139 exposed). Design: `docs/audits/2026-08-03-e4-implicit-kernel-design.md`
//! § 3; the extraction is that audit's slice E4-1 (`Scheme::Explicit` only — the
//! unconditionally stable scheme is E4-2 and does not exist yet).
//!
//! **⚠ NEEDS RATIFICATION (venue, audit pick U-3).** This module sits in
//! `dc-core` on the audit's recommendation (precedent: [`crate::coarse::CoarseField`],
//! the other primitive that makes an unsafe consumer move inexpressible). The
//! placement is an integrator call under the standing autonomy protocol, not a
//! user ruling. It also adds `rayon` to `dc-core`'s dependencies (the
//! scalar↔parallel bit-identity promise is the kernel's, so the parallel driver
//! must live here) — flagged with the venue.
//!
//! # The ownership rule (E4's rule, `docs/dependency-graph.md` § 1)
//!
//! Four inputs set the stability threshold and they have four owners: the
//! **stencil** is the kernel's; **`dx`** and **`dt`** are the engine's; the
//! **coefficient field** is content. Only the kernel can know its own constant.
//! A pass declares its physics — the coefficient field ([`CoeffField`]), the
//! potential, the inventory the lower obstacle guards — and hands in an
//! authored rate. It **cannot** state an integration step or a sub-step count:
//! [`FieldKernel::plan`] derives the sub-cycle from the kernel's own bound, and
//! [`ExplicitPlan::sub_rate`] is the only rate a step can run at. **The unsafe
//! call is inexpressible** (the one deliberate door is
//! [`FieldKernel::plan_unbounded_legacy`], which exists so the pre-bound golden
//! fixed points stay reachable — see its docs).
//!
//! # Flux-form
//!
//! The kernel never returns a new state. A step produces the donor-limiter
//! plane, the **antisymmetric per-edge fluxes** ([`EdgeFluxes`]), and their
//! per-cell net gather; the caller owns the state and applies the net. Every
//! unit that leaves one cell arrives in exactly one other by construction
//! (IEEE-754 subtraction is exactly antisymmetric, and one stored edge value
//! serves both endpoints), which is what makes mass-exactness a property of the
//! contract rather than of solver convergence — and the flux return is the door
//! a future sorting consumer (per-flux species arithmetic) needs.
//!
//! # What is deliberately NOT here yet (E4-1 scope)
//!
//! - **`dx` is folded into the authored coefficient** — the convention the
//!   shipped content already uses (`cfg.diffusion` is per-edge dimensionless;
//!   the grid's 460 m never appears in the kernel). Stated rather than silently
//!   normalised; the `dx`-aware signature waits for a second grid pitch to
//!   force it (audit § 3.3, a recorded deferral).
//! - **The potential is frozen by the caller** and handed in as a slice. The
//!   creep pass builds `s = r + h` in a scratch plane shared with its other
//!   phases; composing `datum + state` inside the kernel is the audit's § 3.1
//!   shape and rides E4-2's problem struct.
//! - **One scheme.** `Scheme::Explicit` — today's sub-cycled monotone scheme,
//!   byte for byte. `ImplicitBE` (audit § 2, K2) is E4-2.

use rayon::prelude::*;

use gather::{flux_cell, net_cell, peak_coeff, scale_cell};

mod gather;
#[cfg(test)]
mod tests;

/// The stencil the kernel discretises over. The stability constant is a
/// property of the discretisation, which is why it lives with this enum and
/// nowhere else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stencil {
    /// The 4-cardinal-neighbour Laplacian on a square grid.
    FourNeighbour,
}

impl Stencil {
    /// **The per-edge coefficient at which this stencil's explicit step stops
    /// being monotone** — the bound the kernel sub-cycles to respect.
    ///
    /// # Where 1/8 comes from — derived, never tuned (journal/0122)
    ///
    /// Write one explicit step on the frozen potential as
    /// `h_i ← h_i + a·Σ_j (s_j − s_i)` over the four cardinal neighbours, with
    /// `a` the per-edge coefficient. The von Neumann amplification factor is
    /// `g(k) = 1 − 2a(2 − cos k_x − cos k_y)`, so:
    ///
    /// - `a ≤ 1/4` ⇒ `g ≥ −1`: **stable**, but the grid-scale (Nyquist) mode is
    ///   reflected with its amplitude intact — `g(π,π) = −1` is a period-2
    ///   flip-flop that never decays.
    /// - `a ≤ 1/8` ⇒ `g ≥ 0`: **monotone**. No mode may change sign, so a
    ///   checkerboard cannot survive a step, let alone be created by one.
    ///
    /// The teacher was hillslope creep: the shipped world's peak effective
    /// coefficient sat 2.1× past this bound (calibrated, 100.8×), the inventory
    /// limiter capped the flipped mode at each cell's stock, and a divergence
    /// read as a stable 40 m grid-scale limit cycle — deaf to `dt` refinement,
    /// which is why the bound must live with the kernel rather than with any
    /// clock (corrections #72; spines § S-10; `stubs.md` § 30).
    #[inline]
    pub const fn monotone_max_edge_coeff(self) -> f64 {
        match self {
            Stencil::FourNeighbour => MONOTONE_MAX_EDGE_COEFF,
        }
    }
}

/// [`Stencil::FourNeighbour`]'s monotonicity bound — see
/// [`Stencil::monotone_max_edge_coeff`] for the derivation. Exported as a
/// constant because content-side instruments report their peak coefficient
/// against it.
pub const MONOTONE_MAX_EDGE_COEFF: f64 = 0.125;

/// The integration scheme. E4-1 ships the explicit sub-cycled scheme only; the
/// unconditionally stable implicit scheme (audit § 2, K2) is E4-2 and will be a
/// second variant here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    /// Explicit forward step, sub-cycled to the stencil's monotonicity bound.
    Explicit,
}

/// **The content-declared coefficient field** — the per-cell diffusivity at the
/// applied rate. This is the one input the kernel executes but cannot own: what
/// a slope's diffusivity is *made of* (a config rate reduced by biotic
/// resistance, scaled by lithology — or a permeability, or a conductivity) is
/// pass physics.
///
/// # Contract
///
/// - **Pure and rate-linear.** `at` must be a pure function of `(rate, i)`, and
///   (at most) linear in `rate`: the kernel measures the field's peak at the
///   authored rate and then steps at `rate / n`, relying on the field to scale
///   down with it. A field that ignored `rate` would defeat the bound the
///   kernel exists to hold.
/// - **Deterministic and `Sync`** — it is called from the parallel driver, and
///   the kernel's scalar↔parallel bit-identity promise rides on it.
pub trait CoeffField: Sync {
    /// The per-cell coefficient at the applied rate.
    fn at(&self, rate: f64, i: usize) -> f64;
}

/// **The antisymmetric per-edge fluxes of one step** — the flux-form return.
///
/// Two planes over the grid: `east[i]` is the signed flux across the edge from
/// cell `i` to its east neighbour (`i+1`), `south[i]` to its south neighbour
/// (`i+w`); positive = leaving `i`. Border edges (no neighbour) hold `0.0`.
/// One stored value serves both endpoints, so the pair of cells sharing an edge
/// cannot disagree about what crossed it — antisymmetry by construction, not by
/// test.
#[derive(Default)]
pub struct EdgeFluxes {
    pub(crate) east: Vec<f64>,
    pub(crate) south: Vec<f64>,
}

impl EdgeFluxes {
    /// An empty flux record; the kernel sizes it on first use.
    pub fn new() -> Self {
        Self::default()
    }

    /// Eastward signed flux per cell (`0.0` on the east border).
    pub fn east(&self) -> &[f64] {
        &self.east
    }

    /// Southward signed flux per cell (`0.0` on the south border).
    pub fn south(&self) -> &[f64] {
        &self.south
    }

    /// Resident bytes, for the caller's scratch accounting.
    pub fn bytes(&self) -> usize {
        (self.east.len() + self.south.len()) * std::mem::size_of::<f64>()
    }

    fn ensure(&mut self, n: usize) {
        if self.east.len() != n {
            self.east = vec![0.0; n];
            self.south = vec![0.0; n];
        }
    }
}

/// **One diffusion problem, as the pass declares it** — pure physics plus the
/// driver knob. The pass states *what* diffuses (state, potential, coefficient
/// field); it cannot state how finely to integrate it (audit § 3.1's problem
/// struct, E4-1 subset).
pub struct DiffusionProblem<'a, C: CoeffField> {
    /// Grid width; the potential/state planes are `w × (len / w)`.
    pub w: usize,
    /// The frozen potential the fluxes read (`datum + state`, built by the
    /// caller — see the module docs on the freeze).
    pub potential: &'a [f64],
    /// The conserved quantity the lower obstacle guards: a cell cannot ship
    /// more than it holds.
    pub state: &'a [f64],
    /// The content-declared coefficient field.
    pub coeff: &'a C,
    /// Data-parallel drive (byte-identical to scalar by construction).
    pub parallel: bool,
}

/// The caller-owned output planes of one [`FieldKernel::step`]: the donor
/// limiter, the edge fluxes, and their per-cell net gather. The caller applies
/// `net` to its state; the kernel never does.
pub struct StepScratch<'a> {
    /// Per-cell donor limiter (`≤ 1`; `< 1` where the cell wanted to ship more
    /// than it holds) — the lower obstacle, written by the step.
    pub scale: &'a mut [f64],
    /// The antisymmetric edge fluxes, written by the step.
    pub fluxes: &'a mut EdgeFluxes,
    /// Per-cell net (Σ influx − Σ outflux), gathered from the fluxes in the
    /// stencil's fixed edge order — what the caller adds to its state.
    pub net: &'a mut [f64],
}

/// One planned integration of an authored rate: the kernel's derived sub-step
/// count and the only rate a step may run at. Plain data — it borrows nothing.
#[derive(Clone, Copy, Debug)]
pub struct ExplicitPlan {
    substeps: u32,
    sub_rate: f64,
    peak_coeff: f64,
}

impl ExplicitPlan {
    /// How many sub-steps the plan takes to keep every per-edge coefficient
    /// inside the kernel's bound. `1` when the authored rate already sits
    /// inside it. A diagnostic, not a knob.
    #[inline]
    pub fn substeps(&self) -> u32 {
        self.substeps
    }

    /// The per-sub-step rate — `rate / substeps`. Exposed because content-side
    /// arithmetic that *partitions* a flux the kernel computed (a species
    /// split, a shoreline tally) must evaluate its coefficient field at the
    /// same rate the kernel did.
    #[inline]
    pub fn sub_rate(&self) -> f64 {
        self.sub_rate
    }

    /// The largest per-cell coefficient the plan divided down — the numerator
    /// of its sub-step count, at the authored rate.
    #[inline]
    pub fn peak_coeff(&self) -> f64 {
        self.peak_coeff
    }
}

/// **The field-solver kernel** — stencil + scheme, and nothing else. Stateless;
/// construct it `const` next to the pass that declares against it.
#[derive(Clone, Copy, Debug)]
pub struct FieldKernel {
    stencil: Stencil,
    scheme: Scheme,
}

impl FieldKernel {
    /// The explicit sub-cycled kernel over the given stencil.
    pub const fn explicit(stencil: Stencil) -> Self {
        Self {
            stencil,
            scheme: Scheme::Explicit,
        }
    }

    /// **Derive the sub-cycle from the kernel's own bound** (the S-10 ruling:
    /// the kernel takes `dt` from outside and computes its own internal
    /// multiplier). Measures the coefficient field's peak over the grid at the
    /// authored rate (an order-independent `f64::max` fold), then divides the
    /// rate until every per-edge coefficient sits inside
    /// [`Stencil::monotone_max_edge_coeff`].
    ///
    /// The max is over **cells, not the config**: a content field folds
    /// per-cell factors into the rate, and it is the worst cell that decides
    /// whether *any* cell may oscillate. One hot coefficient sub-divides
    /// everywhere — substepping is deliberately NOT locally adaptive
    /// (spines § S-10: local adaptivity breaks order-independence).
    pub fn plan<C: CoeffField>(&self, rate: f64, cells: usize, coeff: &C) -> ExplicitPlan {
        match self.scheme {
            Scheme::Explicit => {}
        }
        let peak = peak_coeff(rate, cells, coeff);
        let bound = self.stencil.monotone_max_edge_coeff();
        let substeps = if peak > bound {
            // `ceil` of a finite positive ratio; the `max(1)` is belt-and-braces
            // against a denormal reduction, not a live case.
            ((peak / bound).ceil() as u32).max(1)
        } else {
            1
        };
        ExplicitPlan {
            substeps,
            sub_rate: rate / f64::from(substeps),
            peak_coeff: peak,
        }
    }

    /// **The pre-bound operator, kept reachable on purpose: one raw step at the
    /// stated rate, whatever the coefficient.** This is the ONE door past the
    /// monotonicity bound, and it exists because golden fixed points were
    /// captured under the unbounded operator and must stay reachable
    /// (`DeepConfig::creep_substep: false`;
    /// `GOLDEN_SURFACE_UNBOUNDED_CREEP` / `GOLDEN_RECORD_UNBOUNDED_CREEP`) and
    /// because the reference arm of the operator tests drives single raw steps
    /// by hand. Do not reach for it from a production pass: past the bound the
    /// grid-scale mode flips sign and the inventory limiter turns the
    /// divergence into a limit cycle that reads as stable (journal/0116/0122).
    pub fn plan_unbounded_legacy<C: CoeffField>(
        &self,
        rate: f64,
        cells: usize,
        coeff: &C,
    ) -> ExplicitPlan {
        let substeps = 1u32;
        ExplicitPlan {
            substeps,
            sub_rate: rate / f64::from(substeps),
            peak_coeff: peak_coeff(rate, cells, coeff),
        }
    }

    /// **One explicit sub-step, flux-form**, on a frozen potential:
    ///
    /// 1. the **donor limiter** plane — each cell's requested outflux summed on
    ///    the frozen potential, clamped at its inventory (`state`): the lower
    ///    obstacle, `scale[i] = min(1, state_i / Σ requested)`;
    /// 2. the **antisymmetric edge fluxes** — per edge, the donor's coefficient
    ///    times the potential drop times the donor's limiter, stored once and
    ///    read by both endpoints;
    /// 3. the **net gather** — each cell sums its own four edges from the flux
    ///    planes, in the stencil's fixed order (W, E, N, S).
    ///
    /// The kernel does **not** apply the net: the caller owns the state. Every
    /// plane is a pure function of the frozen inputs with disjoint writes, so
    /// the parallel driver is byte-identical to the scalar one by construction.
    ///
    /// The step integrates at `plan.sub_rate()` and at no other rate — which is
    /// the whole inexpressibility argument, so keep it true.
    pub fn step<C: CoeffField>(
        &self,
        plan: &ExplicitPlan,
        p: &DiffusionProblem<'_, C>,
        out: StepScratch<'_>,
    ) {
        let (w, potential, state, coeff, parallel) =
            (p.w, p.potential, p.state, p.coeff, p.parallel);
        let n = potential.len();
        debug_assert_eq!(state.len(), n);
        debug_assert_eq!(out.scale.len(), n);
        debug_assert_eq!(out.net.len(), n);
        debug_assert_eq!(n % w, 0);
        let rate = plan.sub_rate;
        let StepScratch { scale, fluxes, net } = out;
        fluxes.ensure(n);
        // Pass 1: per-cell donor limiter on the frozen potential.
        if parallel {
            scale
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, sc)| *sc = scale_cell(i, w, potential, state[i], rate, coeff));
        } else {
            for (i, sc) in scale.iter_mut().enumerate() {
                *sc = scale_cell(i, w, potential, state[i], rate, coeff);
            }
        }
        // Pass 2: the antisymmetric edge fluxes (each cell writes only its own
        // east/south entries — disjoint).
        {
            let scale = &*scale;
            let (east, south) = (&mut fluxes.east, &mut fluxes.south);
            if parallel {
                east.par_iter_mut()
                    .zip(south.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (e, s))| {
                        let (fe, fs) = flux_cell(i, w, potential, scale, rate, coeff);
                        *e = fe;
                        *s = fs;
                    });
            } else {
                for (i, (e, s)) in east.iter_mut().zip(south.iter_mut()).enumerate() {
                    let (fe, fs) = flux_cell(i, w, potential, scale, rate, coeff);
                    *e = fe;
                    *s = fs;
                }
            }
        }
        // Pass 3: net gather from the flux planes.
        {
            let (east, south) = (&fluxes.east[..], &fluxes.south[..]);
            if parallel {
                net.par_iter_mut()
                    .enumerate()
                    .for_each(|(i, nd)| *nd = net_cell(i, w, east, south));
            } else {
                for (i, nd) in net.iter_mut().enumerate() {
                    *nd = net_cell(i, w, east, south);
                }
            }
        }
    }
}
