//! **Isostasy and flexure** (tectonics.md § 6): elevation is a *derived*
//! quantity. The crustal columns ([`super::tectonics`]) float on the mantle, so
//! erosion-unloading rebounds the root, relief persists for hundreds of Myr, and
//! deep-formed rock is exhumed — all from one mechanism instead of four patches.
//!
//! ## Why Airy at the *flexural wavelength*, not per-cell (§ 6.1)
//!
//! The brief said "Airy-style local compensation at minimum". Taken literally
//! per-cell that is physically wrong and numerically dangerous: real lithosphere
//! supports loads elastically below the flexural wavelength (tens of km), so a
//! 460 m column does not float independently. Per-cell Airy would make every
//! incised valley floor rebound against its own ridges — an erosion↔rebound
//! feedback at exactly the grid scale, a stability hazard aimed straight at the
//! erodibility clamp and a grid-wavelength artifact of the kind method rule 5
//! forbids. So the v1 mechanism is **Airy compensation of the *smoothed* load**:
//! smoothing the compensation is what plate rigidity does. It buys the two
//! signatures that matter — erosion-unloading rebound (~ρc/ρm ≈ 0.85 of removed
//! thickness) and the **foreland moat** (a smoothed root beside a belt where the
//! surface load is not → the flank pulled below its local Airy height → the
//! sediment trap).
//!
//! ## Determinism / parallelism
//!
//! The smoothing is a fixed-order separable prefix-sum blur — **scalar in both
//! drivers** (the established serial-floor pattern, like the priority flood), so
//! it is byte-identical scalar↔parallel by construction, and it is the only new
//! per-iteration serial cost of note (§ SPIKE 3). The relax step is a pure
//! per-cell write.

use super::tectonics::CrustKind;

/// Mantle density (kg/m³).
pub const RHO_MANTLE: f64 = 3300.0;
/// Sediment (alluvium) density (kg/m³).
pub const RHO_SED: f64 = 2400.0;

/// Crust density for a kind (kg/m³): continental 2800, oceanic 2950, shelf 2870
/// (§ 5.1 — a per-kind constant, not a per-cell plane; no consumer needs per-cell
/// density variation composition kind does not already carry).
#[inline]
pub fn rho_crust(kind: CrustKind) -> f64 {
    match kind {
        CrustKind::Continental => 2800.0,
        CrustKind::Oceanic => 2950.0,
        CrustKind::Transitional => 2870.0,
    }
}

/// Airy reference constant (m), calibrated so a quiet 35 km continental column
/// sits at ~+400 m and a 7 km oceanic column at ~−4000 m (§ 6.1):
/// `35000·(3300−2800)/3300 − C_REF = +400`.
pub const C_REF: f64 = 4900.0;

/// The equilibrium (fully-compensated) surface elevation for a column of
/// smoothed thickness `t_bar` (crust) and `h_bar` (sediment) with crust density
/// `rho_c`:
/// `e = (t_bar·(ρm−ρc) + h_bar·(ρm−ρs))/ρm − C_REF`.
#[inline]
pub fn equilibrium(t_bar: f64, h_bar: f64, rho_c: f64) -> f64 {
    (t_bar * (RHO_MANTLE - rho_c) + h_bar * (RHO_MANTLE - RHO_SED)) / RHO_MANTLE - C_REF
}

/// Separable box-blur smoothing of a `w × w` field with the given cell radius, in
/// place-free form (returns a new vector). Fixed scan order (rows then columns),
/// prefix-sum per line so the cost is O(n) independent of the radius. Border
/// windows shrink (clamped), which keeps the mean well-defined at the edge. This
/// is the flexural smoothing: a load spread over the plate's rigid wavelength.
pub fn box_smooth(field: &[f64], w: usize, radius: usize) -> Vec<f64> {
    if radius == 0 {
        return field.to_vec();
    }
    let mut tmp = vec![0.0f64; w * w];
    // Horizontal pass.
    for y in 0..w {
        let row = &field[y * w..y * w + w];
        // Prefix sums (length w+1).
        let mut pre = vec![0.0f64; w + 1];
        for x in 0..w {
            pre[x + 1] = pre[x] + row[x];
        }
        for x in 0..w {
            let lo = x.saturating_sub(radius);
            let hi = (x + radius + 1).min(w);
            let sum = pre[hi] - pre[lo];
            tmp[y * w + x] = sum / (hi - lo) as f64;
        }
    }
    // Vertical pass.
    let mut out = vec![0.0f64; w * w];
    let mut col = vec![0.0f64; w];
    let mut pre = vec![0.0f64; w + 1];
    for x in 0..w {
        for y in 0..w {
            col[y] = tmp[y * w + x];
        }
        pre[0] = 0.0;
        for y in 0..w {
            pre[y + 1] = pre[y] + col[y];
        }
        for y in 0..w {
            let lo = y.saturating_sub(radius);
            let hi = (y + radius + 1).min(w);
            let sum = pre[hi] - pre[lo];
            out[y * w + x] = sum / (hi - lo) as f64;
        }
    }
    out
}

/// The flexural smoothing radius (cells) for a flexural wavelength in km at a
/// given cell size — resolution-independent because it is stated in km.
#[inline]
pub fn flex_radius_cells(flex_wavelength_km: f64, cell_m: f64) -> usize {
    ((flex_wavelength_km * 1000.0 / cell_m).round() as usize).max(1)
}
