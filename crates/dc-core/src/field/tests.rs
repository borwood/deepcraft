//! Kernel unit tests. The load-bearing one is the first: the flux-form step
//! must reproduce the embedded gather it was extracted from **to the bit** —
//! that is E4-1's whole contract, asserted here at fixture scale and by the
//! full golden suite at world scale. All fixtures are deterministic arithmetic;
//! the invariants are scale-free (per-cell predicates and identities with no
//! length in them), so small grids test them completely.

use super::gather::{NEIGH4, coords_of, in_grid};
use super::*;

/// A homogeneous field: coefficient = the rate everywhere.
struct Uniform;
impl CoeffField for Uniform {
    fn at(&self, rate: f64, _i: usize) -> f64 {
        rate
    }
}

/// A heterogeneous field: the rate scaled by a per-cell factor (zeros
/// included) — the shape every content field has.
struct PerCell(Vec<f64>);
impl CoeffField for PerCell {
    fn at(&self, rate: f64, i: usize) -> f64 {
        rate * self.0[i]
    }
}

/// A deterministic rough fixture: bumpy potential, patchy inventory (so the
/// limiter genuinely binds), heterogeneous coefficient with zero cells.
fn fixture(w: usize) -> (Vec<f64>, Vec<f64>, PerCell) {
    let n = w * w;
    let potential: Vec<f64> = (0..n)
        .map(|i| {
            let (x, y) = ((i % w) as f64, (i / w) as f64);
            100.0 - 3.0 * x + 7.0 * ((x * 0.7).sin() + (y * 1.3).cos())
        })
        .collect();
    let state: Vec<f64> = (0..n)
        .map(|i| match i % 4 {
            0 => 30.0,
            1 => 0.0,
            _ => 2.0,
        })
        .collect();
    let factor: Vec<f64> = (0..n)
        .map(|i| match i % 5 {
            0 => 0.0,
            k => 0.25 * k as f64,
        })
        .collect();
    (potential, state, PerCell(factor))
}

fn run_step<C: CoeffField>(
    kernel: &FieldKernel,
    plan: &ExplicitPlan,
    w: usize,
    potential: &[f64],
    state: &[f64],
    coeff: &C,
    parallel: bool,
) -> (Vec<f64>, Vec<f64>) {
    let n = potential.len();
    let mut scale = vec![0.0; n];
    let mut fluxes = EdgeFluxes::new();
    let mut net = vec![0.0; n];
    kernel.step(
        plan,
        &DiffusionProblem {
            w,
            potential,
            state,
            coeff,
            parallel,
        },
        StepScratch {
            scale: &mut scale,
            fluxes: &mut fluxes,
            net: &mut net,
        },
    );
    (scale, net)
}

/// **The reference: the embedded gather this kernel was extracted from**
/// (`erosion/creep_kernel.rs::diffuse_net_cell` at `f3e5d17`, generalised over
/// the coefficient field) — each cell recomputes every edge term itself, in
/// NEIGH4 order.
fn reference_net_cell<C: CoeffField>(
    i: usize,
    w: usize,
    potential: &[f64],
    scale: &[f64],
    rate: f64,
    coeff: &C,
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = potential[i];
    let mut net = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w, potential.len()) {
            let d = si - potential[j];
            if d > 0.0 {
                net -= coeff.at(rate, i) * d * scale[i];
            } else if d < 0.0 {
                net += coeff.at(rate, j) * (-d) * scale[j];
            }
        }
    }
    net
}

#[test]
fn the_flux_form_gather_is_the_embedded_gather_to_the_bit() {
    let w = 16;
    let (potential, state, coeff) = fixture(w);
    let n = w * w;
    let kernel = FieldKernel::explicit(Stencil::FourNeighbour);
    for rate in [0.05, 0.7, 5.4] {
        let plan = kernel.plan_unbounded_legacy(rate, n, &coeff);
        let (scale, net) = run_step(&kernel, &plan, w, &potential, &state, &coeff, false);
        for i in 0..n {
            let reference = reference_net_cell(i, w, &potential, &scale, rate, &coeff);
            assert_eq!(
                net[i].to_bits(),
                reference.to_bits(),
                "rate {rate}, cell {i}: flux-form net {} != embedded gather {}",
                net[i],
                reference
            );
        }
    }
}

#[test]
fn scalar_and_parallel_steps_are_bit_identical() {
    let w = 24;
    let (potential, state, coeff) = fixture(w);
    let n = w * w;
    let kernel = FieldKernel::explicit(Stencil::FourNeighbour);
    let plan = kernel.plan(3.3, n, &coeff);
    let (scale_s, net_s) = run_step(&kernel, &plan, w, &potential, &state, &coeff, false);
    let (scale_p, net_p) = run_step(&kernel, &plan, w, &potential, &state, &coeff, true);
    assert_eq!(scale_s, scale_p, "scalar and parallel limiter disagree");
    assert_eq!(net_s, net_p, "scalar and parallel net disagree");
}

/// The plan is the S-10 ruling: the rate from outside, the internal multiplier
/// from the kernel's own bound.
#[test]
fn the_plan_derives_its_substep_count_from_the_bound() {
    let kernel = FieldKernel::explicit(Stencil::FourNeighbour);
    let n = 64;
    // Inside the bound: one step, at the stated rate, bit for bit.
    let inside = kernel.plan(0.12, n, &Uniform);
    assert_eq!(inside.substeps(), 1);
    assert_eq!(inside.sub_rate().to_bits(), 0.12f64.to_bits());
    assert_eq!(inside.peak_coeff().to_bits(), 0.12f64.to_bits());
    // Past the bound: ceil(peak / bound), and the sub-rate is the division.
    let past = kernel.plan(5.4, n, &Uniform);
    assert_eq!(
        past.substeps(),
        (5.4f64 / MONOTONE_MAX_EDGE_COEFF).ceil() as u32
    );
    assert_eq!(
        past.sub_rate().to_bits(),
        (5.4f64 / f64::from(past.substeps())).to_bits()
    );
    assert!(past.sub_rate() <= MONOTONE_MAX_EDGE_COEFF);
    // The legacy door: one raw step at the stated rate, peak still honest.
    let raw = kernel.plan_unbounded_legacy(5.4, n, &Uniform);
    assert_eq!(raw.substeps(), 1);
    assert_eq!(raw.sub_rate().to_bits(), 5.4f64.to_bits());
    assert_eq!(raw.peak_coeff().to_bits(), 5.4f64.to_bits());
}

/// The lower obstacle: applying the net can never take a cell below zero
/// (beyond round-off), at any rate — the limiter is the kernel's, not the
/// caller's.
#[test]
fn the_donor_limiter_keeps_state_non_negative() {
    let w = 16;
    let (potential, state, coeff) = fixture(w);
    let n = w * w;
    let kernel = FieldKernel::explicit(Stencil::FourNeighbour);
    for rate in [0.12, 5.4, 40.0] {
        let plan = kernel.plan(rate, n, &coeff);
        let (_, net) = run_step(&kernel, &plan, w, &potential, &state, &coeff, false);
        for i in 0..n {
            assert!(
                state[i] + net[i] >= -1e-9,
                "rate {rate}, cell {i}: state {} + net {} went negative",
                state[i],
                net[i]
            );
        }
    }
}
