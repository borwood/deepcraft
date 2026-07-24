//! **The geotherm — the first §5 *field pass*** (material-behavior.md §5/§14,
//! DECIDED 2026-07-24). It stands up the `temperature` **condition-field**: a
//! per-cell **geothermal gradient** (°C/m) computed from the tectonic state, such
//! that `T(depth) = surface_T + gradient · depth`.
//!
//! ## What a field pass is, and why the geotherm is one
//!
//! A **field pass** computes a value per cell via a solve over the material state
//! and plants it as **environment** the cellular passes and the formation
//! predicates read (§5). It **never runs edges** and **never touches the form
//! inventory** — output is *a field*, not changed material. The geotherm is the
//! first one: it reads `{surface_temperature (climate), crustal_thickness
//! (t_crust), crust_kind, tectonic_setting (province)}` and writes the
//! [`FIELD_TEMPERATURE`] field. It runs on the deep-time pass-runner
//! ([`super::runner`]) as a declared pass at a **low cadence** — heat flow
//! evolves far slower than the surface climate, so `period > 1`.
//!
//! ## The gradient model (v1 LINEAR)
//!
//! `gradient = f(tectonic heat flow)`. The primary signal is the **tectonic
//! setting** ([`BoundaryKind`]): active spreading/extension/arc volcanism runs
//! hot (steep gradient), a cold subducting slab runs cold, and a stable interior
//! sits at its crust type's baseline. A **crustal-thickness modifier** then reads
//! `t_crust` as a *deviation from that crust kind's own seed thickness*: crust
//! thinned below baseline (a rift) runs hotter, crust thickened above it (an
//! orogenic root, an old shield) runs colder. That deviation is what separates a
//! **rifted** continental cell (thin, hot) from a **cratonic** one (thick, cold)
//! though both are `Continental` — the "old/thick cratonic crust shallow"
//! behaviour §14 asks for.
//!
//! **v1 is linear in depth** (`T = surface_T + gradient·z`). A nonlinear /
//! mantle-heat profile is a ROADMAP followup, deliberately not built here.
//!
//! ## What it subsumes
//!
//! The degenerate `burial_temp_c` provider (0 °C surface, exactly 1 °C/m, so the
//! "temperature" was the overburden in metres) is **retired** by this pass
//! (`stubs.md` §14). Coal rank now reads a real `T(depth)` off this field, and
//! [`COAL_ONSET_C`](super::COAL_ONSET_C) is recalibrated to a real onset
//! temperature in the same slice — the "retire together" the stub warned of (a
//! real gradient against the old 8.0 onset would turn the whole record to coal,
//! since surface air temperature alone clears 8 °C almost everywhere).

use super::grid::{DeepConfig, DeepGrid};
use super::tectonics::{self, BoundaryKind, CrustKind};

/// The opaque id of the **`temperature` condition-field** (§14) — the first
/// member of the condition-field vocabulary. Consumers read the field *by this
/// id*; it is not an ad-hoc private plane. The geotherm field pass is its sole
/// producer today; formation predicates and the metamorphism heir will read it.
pub const FIELD_TEMPERATURE: &str = "dc:field/temperature";

/// The pass cadence (epochs per firing) of the geotherm field pass. Coarser than
/// the climate re-march ([`DeepConfig::remarch_interval`], 20) because crustal
/// heat flow evolves slower than the surface moisture field — the RATE axis of
/// the runner (§5 "order × rate"), made concrete for the first field pass.
pub const GEOTHERM_PERIOD: u32 = 40;

/// Fallback geothermal gradient (°C/m) where no crustal state exists — i.e. when
/// tectonic history is off, so the geotherm pass never ran and `grid.geotherm` is
/// empty. A continental-average ~25 °C/km, the number the "no province
/// information" case earns. Consumers that read the field on a tectonic-history
/// world never touch this — the plane is populated there.
pub const DEFAULT_CONTINENTAL_GRADIENT_C_PER_M: f64 = 0.025;

/// The **geothermal gradient** (°C per metre) for a cell — v1 linear model. Reads
/// the tectonic **setting** (province), the **crust kind**, and the **crustal
/// thickness**; see the module docs for the physics. Bounded to a plausible
/// 12–55 °C/km so the self-reinforcing thickness feedback cannot run away.
pub fn gradient_c_per_m(setting: BoundaryKind, crust: CrustKind, t_crust_m: f64) -> f64 {
    // Heat flow by active tectonic setting (°C/km). Rifts/ridges/arcs steep;
    // trenches (cold subducting slab) cold; collisional welts cool near-surface;
    // a plate interior sits at its crust kind's baseline.
    let setting_grad_c_per_km = match setting {
        BoundaryKind::Ridge => 50.0, // seafloor spreading centre
        BoundaryKind::Rift => 45.0,  // active continental extension
        BoundaryKind::Arc => 42.0,   // volcanic arc
        BoundaryKind::Transform => 27.0,
        BoundaryKind::Orogeny => 25.0, // thick collisional root, cool near-surface
        BoundaryKind::Trench => 16.0,  // cold subducting slab
        BoundaryKind::Interior => match crust {
            CrustKind::Oceanic => 32.0, // cooling but still-warm oceanic lithosphere
            CrustKind::Transitional => 26.0, // passive margin
            CrustKind::Continental => 22.0, // stable continental interior
        },
    };
    // Crustal-thickness modifier: a deviation from this crust kind's *own* seed
    // thickness, so a normal 7 km ocean column is not read as "anomalously thin"
    // — only crust genuinely thinned (rift) or thickened (orogen root / craton)
    // relative to its baseline moves. Thinner → hotter, thicker → colder;
    // ±4 °C/km per ±10 km.
    let ref_thick_m = crust.seed_thickness();
    let thick_adj_c_per_km = ((ref_thick_m - t_crust_m) / 10_000.0) * 4.0;
    let grad_c_per_km = (setting_grad_c_per_km + thick_adj_c_per_km).clamp(12.0, 55.0);
    grad_c_per_km / 1000.0
}

/// The `temperature` field evaluated at a depth: `T = surface_T + gradient·depth`
/// (v1 linear). The one place the field's linear form lives, so a consumer never
/// re-derives it. `surface_t_c` is the field's upper boundary condition (the
/// climate surface temperature); `gradient_c_per_m` is what the geotherm planted.
#[inline]
pub fn temperature_c(surface_t_c: f64, gradient_c_per_m: f64, depth_m: f64) -> f64 {
    surface_t_c + gradient_c_per_m * depth_m
}

/// Everything the coalification pass knows about a *column* rather than a unit:
/// where it is, its surface temperature (the geotherm's upper boundary
/// condition), and the **geothermal gradient** the geotherm planted there. The
/// per-unit burial depth is walked by [`promote_coal`](super::recorder::DeepStrata::promote_coal)
/// itself.
///
/// Moved here from the retired `burial_temp_c` provider: with a real geotherm the
/// temperature is no longer a stateless `fn` of overburden — it needs the
/// per-cell gradient the field carries, which a provider `fn` could not hold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BurialColumn {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
    /// Air temperature (°C) at this column's present surface — the geotherm's
    /// upper boundary condition.
    pub surface_temp_c: f64,
    /// The geothermal gradient (°C/m) at this column, from the `temperature`
    /// field ([`gradient_c_per_m`]).
    pub gradient_c_per_m: f64,
}

/// **The geotherm field-pass solve.** Compute the per-cell geothermal gradient
/// into `grid.geotherm` from the current crustal state and tectonic setting. A
/// no-op when there is no crustal state (`t_crust` empty — tectonic history off),
/// which is why the pass is only scheduled on the tectonic-history path.
///
/// Called by the runner's `geotherm` pass at its cadence, and once pre-loop as
/// the coarse-rate seed (like the climate march).
pub(crate) fn march(
    grid: &mut DeepGrid,
    plates: &[super::tectonics::Plate],
    v_ref: f64,
    cfg: &DeepConfig,
) {
    if grid.t_crust.is_empty() {
        return;
    }
    let w = grid.w;
    let n = w * w;
    if grid.geotherm.len() != n {
        grid.geotherm = vec![0.0f64; n];
    }
    let cell_km = grid.cell_m / 1000.0;
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let x = (gx as f64 + 0.5) * cell_km;
            let y = (gy as f64 + 0.5) * cell_km;
            let setting = tectonics::dominant_kind(plates, cfg, v_ref, x, y);
            let crust = CrustKind::from_index(grid.crust_kind[i]);
            grid.geotherm[i] = gradient_c_per_m(setting, crust, grid.t_crust[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Temperature sanity — the §14 ordering.** A rift runs hotter than a
    /// craton, and T rises with depth. Stated as a pure-function test because the
    /// gradient model is a pure function of `(setting, crust, t_crust)`.
    #[test]
    fn rift_is_hotter_than_craton_and_temperature_rises_with_depth() {
        // Rift: active extension on thinned (25 km) continental crust.
        let rift = gradient_c_per_m(BoundaryKind::Rift, CrustKind::Continental, 25_000.0);
        // Craton: a stable continental interior on thick (45 km) old crust.
        let craton = gradient_c_per_m(BoundaryKind::Interior, CrustKind::Continental, 45_000.0);
        assert!(
            rift > craton,
            "rift gradient {rift} must exceed craton gradient {craton}"
        );

        // The craton lands in the §14 "shallow ~15–20 °C/km" band; the rift in
        // the "steep ~40–50" band.
        assert!(
            (0.015..=0.020).contains(&craton),
            "craton gradient {craton} °C/m outside the shallow band"
        );
        assert!(
            (0.040..=0.055).contains(&rift),
            "rift gradient {rift} °C/m outside the steep band"
        );

        // T rises monotonically with depth at a fixed surface + gradient.
        let surf = 15.0;
        let (shallow, deep) = (
            temperature_c(surf, craton, 100.0),
            temperature_c(surf, craton, 1000.0),
        );
        assert!(deep > shallow && shallow > surf);
    }

    /// An arc runs hot; a trench (cold subducting slab) runs cold — the two ends
    /// of the setting ladder the design names.
    #[test]
    fn arc_is_hot_and_trench_is_cold() {
        let arc = gradient_c_per_m(BoundaryKind::Arc, CrustKind::Transitional, 20_000.0);
        let trench = gradient_c_per_m(BoundaryKind::Trench, CrustKind::Oceanic, 7_000.0);
        assert!(
            arc > trench,
            "arc {arc} must be hotter than trench {trench}"
        );
        assert!(arc >= 0.040, "an arc should be steep");
        assert!(trench <= 0.020, "a trench should be shallow");
    }

    /// The gradient is bounded even under an absurd crustal thickness — the
    /// runaway-feedback guard.
    #[test]
    fn the_gradient_is_clamped_to_a_plausible_band() {
        let thin = gradient_c_per_m(BoundaryKind::Rift, CrustKind::Continental, 0.0);
        let thick = gradient_c_per_m(BoundaryKind::Interior, CrustKind::Continental, 200_000.0);
        assert!((0.012..=0.055).contains(&thin));
        assert!((0.012..=0.055).contains(&thick));
    }
}
