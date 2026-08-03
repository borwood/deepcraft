//! The kernel's stencil arithmetic — the per-cell gather functions the step
//! drives. Private to [`super`]: the stencil walk is the kernel's own business,
//! and keeping these unsayable from outside is half of what the extraction
//! bought (spines § S-10; the arithmetic is `erosion/creep_kernel.rs`'s
//! `diffuse_scale_cell` / `diffuse_net_cell` at `f3e5d17`, moved — E4-1 —
//! with the net re-expressed as flux-computation + gather, bit-identically).

use super::CoeffField;

/// The 4-neighbour stencil's fixed edge order: west, east, north, south.
/// Summation order is part of the bit-identity contract — do not reorder.
pub(super) const NEIGH4: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[inline]
pub(super) fn coords_of(i: usize, w: usize) -> (i32, i32) {
    ((i % w) as i32, (i / w) as i32)
}

/// Neighbour index if `(gx, gy)` is inside a `w`-wide grid of `n` cells.
#[inline]
pub(super) fn in_grid(gx: i32, gy: i32, w: usize, n: usize) -> Option<usize> {
    if gx >= 0 && gy >= 0 && (gx as usize) < w {
        let i = (gy as usize) * w + (gx as usize);
        if i < n { Some(i) } else { None }
    } else {
        None
    }
}

/// Per-cell requested-outflux sum → donor limiter scale on the frozen
/// potential, using the donor's own coefficient. `min(1, inventory / Σ
/// requested)` — the lower obstacle.
#[inline]
pub(super) fn scale_cell<C: CoeffField>(
    i: usize,
    w: usize,
    potential: &[f64],
    inventory: f64,
    rate: f64,
    coeff: &C,
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = potential[i];
    let mut out = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w, potential.len()) {
            let d = si - potential[j];
            if d > 0.0 {
                out += coeff.at(rate, i) * d;
            }
        }
    }
    if out > inventory && out > 0.0 {
        inventory / out
    } else {
        1.0
    }
}

/// The signed flux across one edge: the donor's coefficient × the potential
/// drop × the donor's limiter, positive when leaving `i`. Computed once per
/// edge; both endpoints read the same bits, which is the whole conservation
/// argument (IEEE-754 subtraction is exactly antisymmetric, so this value is
/// bit-identical to what either endpoint's own gather would have computed).
#[inline]
fn edge_flux<C: CoeffField>(
    i: usize,
    j: usize,
    potential: &[f64],
    scale: &[f64],
    rate: f64,
    coeff: &C,
) -> f64 {
    let d = potential[i] - potential[j];
    if d > 0.0 {
        coeff.at(rate, i) * d * scale[i]
    } else if d < 0.0 {
        -(coeff.at(rate, j) * (-d) * scale[j])
    } else {
        0.0
    }
}

/// Cell `i`'s two owned edge fluxes (east, south); `0.0` on a border edge.
#[inline]
pub(super) fn flux_cell<C: CoeffField>(
    i: usize,
    w: usize,
    potential: &[f64],
    scale: &[f64],
    rate: f64,
    coeff: &C,
) -> (f64, f64) {
    let (gx, gy) = coords_of(i, w);
    let n = potential.len();
    let e = match in_grid(gx + 1, gy, w, n) {
        Some(j) => edge_flux(i, j, potential, scale, rate, coeff),
        None => 0.0,
    };
    let s = match in_grid(gx, gy + 1, w, n) {
        Some(j) => edge_flux(i, j, potential, scale, rate, coeff),
        None => 0.0,
    };
    (e, s)
}

/// Net at cell `i`, gathered from the stored edge fluxes in the stencil's
/// fixed order — west, east, north, south — the same summation order the
/// embedded gather used, so the sum is bit-identical to it (`x + (−v)` and
/// `x − v` are the same IEEE-754 operation, and a skipped zero edge cannot
/// perturb the accumulator, which is never `−0.0`).
#[inline]
pub(super) fn net_cell(i: usize, w: usize, east: &[f64], south: &[f64]) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let n = east.len();
    let mut net = 0.0;
    if let Some(j) = in_grid(gx - 1, gy, w, n) {
        net += east[j];
    }
    if in_grid(gx + 1, gy, w, n).is_some() {
        net -= east[i];
    }
    if let Some(j) = in_grid(gx, gy - 1, w, n) {
        net += south[j];
    }
    if in_grid(gx, gy + 1, w, n).is_some() {
        net -= south[i];
    }
    net
}

/// The coefficient field's peak over the grid at the applied rate — a
/// sequential `f64::max` fold (order-independent, so the parallel question
/// never arises).
pub(super) fn peak_coeff<C: CoeffField>(rate: f64, cells: usize, coeff: &C) -> f64 {
    (0..cells).map(|i| coeff.at(rate, i)).fold(0.0f64, f64::max)
}
