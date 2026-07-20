//! **Tectonic history** (tectonics.md § 3–4): plate kinematics advected through
//! chapters, and the analytic boundary forcing that replaces the painted 2-ring
//! uplift plane.
//!
//! The one load-bearing move is the *inversion*: surface uplift stops being the
//! input. The input is **plate kinematics** — seeds + velocities, advected
//! through `K` chapters — and each chapter's boundary geometry is evaluated
//! **analytically at deep-grid resolution** (exact distance-to-bisector ×
//! convergence). What that forcing drives is not elevation, it is **crustal
//! thickening** ([`super::isostasy`] then derives elevation from the columns).
//!
//! ## Why the forcing is analytic and grid-free
//!
//! The shipped machine paints ±1400 m·m orogeny onto 14.7 km pregen cells over a
//! 2-ring falloff, then bilinears it into the 460 m deep grid — so nothing
//! narrower than ~15 km wavelength can exist and belt flanks smear to ~50 km
//! (earth-processes § 1). Here the forcing is `amplitude(type, v_conv) ×
//! exp(−(d/W)²)` with **no grid term anywhere in the expression** — `d` is the
//! exact signed distance from a deep cell to the plate-pair bisector, and `W`
//! (orogen half-width) is a design parameter *in kilometres*. The only
//! wavelengths in the forcing are the designed ones (method rule 5 satisfied at
//! the source, not dressed after). The 50 km smear must measurably die (§ SPIKE
//! 5).
//!
//! ## Advection is an honest compression (§ 3.2)
//!
//! Earth plates cross a 251 km world dozens of times over 500 Myr; that churns
//! the map to noise. The design parameter is **total advection over the run, in
//! plate widths** ([`super::grid::DeepConfig::advection_plate_widths`], default
//! ~1). Boundaries migrate across provinces — an arc sweeps, a passive margin
//! turns convergent — without erasing the map's identity. Columns are Eulerian
//! (they do not carry their record sideways), the stated casualty (§ 13): no
//! terrane docking, no lateral strike-slip offset of landscapes.
//!
//! ## Determinism
//!
//! All entropy is addressed draws keyed by `(seed, SALT_TEC_*, plate, field)` —
//! a fresh high-byte family `0x5D00_*` (pregen is `0x5700_*`, deep-time erosion
//! `0x5900_*`, biotic `0x5B00_*`), so the address spaces never collide. Advection
//! and re-classification are pure arithmetic on drawn state; the chapter
//! schedule is fixed by config.

use dc_sim::statistical::rng::draw_f64;

use super::grid::DeepConfig;

/// Addressed-draw salts for the tectonic-history layer. Fresh high byte
/// (`0x5D00_*`) distinct from every existing family. (The design named `0x5B00`,
/// but the biotic layer already owns that byte — journal/0036 records the
/// re-address.)
const SALT_TEC_POS: u64 = 0x5D00_0001;
const SALT_TEC_VEL: u64 = 0x5D00_0002;
const SALT_TEC_CRUST: u64 = 0x5D00_0003;

// Reference mantle/crust densities and the Airy calibration constant live in
// `super::isostasy`; this module only produces thickening rates and the seed.

/// A boundary type classified from the analytic convergence and the two plates'
/// continental flags — the existing taxonomy, now continuous.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoundaryKind {
    /// Continent–continent convergence: mountain belt.
    Orogeny,
    /// Ocean–continent or ocean–ocean convergence, overriding side: volcanic arc.
    Arc,
    /// The subducting side of a convergent margin: trench (thinning).
    Trench,
    /// Divergence on continental crust: rift (thinning).
    Rift,
    /// Divergence on oceanic crust: ridge (young thin crust).
    Ridge,
    /// Strike-slip: near-zero net thickening.
    Transform,
    /// No boundary within kernel reach: plate interior.
    Interior,
}

/// One plate: position (km), velocity (km/chapter), continental flag.
#[derive(Clone, Copy, Debug)]
pub struct Plate {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub continental: bool,
}

/// The plate count for an extent under the `plate_scale_km` knob (U1, § 10). The
/// characteristic plate diameter is `plate_scale_km`; `n ≈ (extent/d)²`, floored
/// at 3. Extent-uniform by construction, which fixes the found defect that the
/// old `clamp(w²/20, 3, 24)` made Large worlds ~3× sparser per km than Medium.
pub fn plate_count(extent_km: f64, plate_scale_km: f64) -> usize {
    let d = plate_scale_km.max(1.0);
    ((extent_km / d).powi(2).round() as usize).max(3)
}

/// The characteristic plate diameter (km) for `n` plates on an extent.
pub fn plate_diameter_km(extent_km: f64, n: usize) -> f64 {
    extent_km / (n as f64).sqrt()
}

/// Build the chapter-0 plate set from the seed, in kilometres. Continental
/// probability is higher toward the disc centre (mirroring pregen's
/// continent-in-ocean-ring topology). Velocity magnitude is set so total
/// advection over the run is `advection_plate_widths` plate diameters.
pub fn build_plates(cfg: &DeepConfig, extent_km: f64) -> Vec<Plate> {
    let n = plate_count(extent_km, cfg.plate_scale_km);
    let d = plate_diameter_km(extent_km, n);
    let center = extent_km / 2.0;
    // Per-chapter step so K chapters cover `advection_plate_widths × d`.
    let per_chapter = cfg.advection_plate_widths * d / f64::from(cfg.chapters.max(1));
    (0..n)
        .map(|p| {
            let p64 = p as u64;
            let x = draw_f64(&[cfg.seed, SALT_TEC_POS, p64, 0]) * extent_km;
            let y = draw_f64(&[cfg.seed, SALT_TEC_POS, p64, 1]) * extent_km;
            let ang = draw_f64(&[cfg.seed, SALT_TEC_VEL, p64, 0]) * std::f64::consts::TAU;
            let speed = per_chapter * (0.5 + 0.5 * draw_f64(&[cfg.seed, SALT_TEC_VEL, p64, 1]));
            let r = ((x - center).powi(2) + (y - center).powi(2)).sqrt();
            let p_cont = if r < extent_km * 0.33 { 0.85 } else { 0.25 };
            let continental = draw_f64(&[cfg.seed, SALT_TEC_POS, p64, 2]) < p_cont;
            Plate {
                x,
                y,
                vx: ang.cos() * speed,
                vy: ang.sin() * speed,
                continental,
            }
        })
        .collect()
}

/// The plate set advected to the start of chapter `c` (`pos += vel · c`). Chapter
/// 0 is the built set. Velocities are fixed (Wilson-cycle rotation is a later
/// knob, § 3.1).
pub fn plates_at_chapter(base: &[Plate], c: u32) -> Vec<Plate> {
    let dt = f64::from(c);
    base.iter()
        .map(|p| Plate {
            x: p.x + p.vx * dt,
            y: p.y + p.vy * dt,
            ..*p
        })
        .collect()
}

/// The whole chapter table: plate state at the start of each of the `K` chapters.
/// ~n_plates × 5 f64 per chapter — the entire tectonic history of a world is a
/// few KB, from which per-chapter deformation is re-derivable analytically
/// (§ 8.2). Chapter `K` (one past the last) is appended so the final chapter can
/// ramp toward a defined next state.
pub fn chapter_table(cfg: &DeepConfig, extent_km: f64) -> Vec<Vec<Plate>> {
    let base = build_plates(cfg, extent_km);
    (0..=cfg.chapters)
        .map(|c| plates_at_chapter(&base, c))
        .collect()
}

/// Indices and squared distances of the `k` nearest plate seeds to `(x, y)`,
/// nearest first. `k` is small (3), so an insertion scan over all plates is
/// cheapest (no spatial index needed at these counts — § 4.1).
fn nearest_k(plates: &[Plate], x: f64, y: f64, k: usize) -> Vec<(usize, f64)> {
    let mut best: Vec<(usize, f64)> = Vec::with_capacity(k + 1);
    for (i, p) in plates.iter().enumerate() {
        let d2 = (p.x - x).powi(2) + (p.y - y).powi(2);
        let pos = best.partition_point(|&(_, bd)| bd <= d2);
        if pos < k {
            best.insert(pos, (i, d2));
            best.truncate(k);
        }
    }
    best
}

/// The convergence rate (km/chapter, closing positive) of plate `b` toward plate
/// `a`, projected on the line joining them.
fn convergence(a: &Plate, b: &Plate) -> f64 {
    let (ex, ey) = (b.x - a.x, b.y - a.y);
    let len = (ex * ex + ey * ey).sqrt().max(1e-9);
    let (ex, ey) = (ex / len, ey / len);
    // a closing on b if a moves +e; b closing on a if b moves -e.
    (a.vx - b.vx) * ex + (a.vy - b.vy) * ey
}

/// Signed perpendicular distance (km) from `(x, y)` to the perpendicular
/// bisector of plates `a, b`, **positive on `a`'s side** (the plate `(x, y)`
/// belongs to when `a` is its nearest seed).
fn signed_bisector_distance(a: &Plate, b: &Plate, x: f64, y: f64) -> f64 {
    let (mx, my) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    // Unit vector from b toward a; (x−m)·û > 0 on a's side.
    let (ux, uy) = (a.x - b.x, a.y - b.y);
    let len = (ux * ux + uy * uy).sqrt().max(1e-9);
    (x - mx) * (ux / len) + (y - my) * (uy / len)
}

/// Classify a boundary from convergence (normalized to the reference velocity)
/// and the two plates' continental flags — the existing thresholds, continuous.
fn classify(vc_norm: f64, a_cont: bool, b_cont: bool) -> BoundaryKind {
    if vc_norm > 0.1 {
        match (a_cont, b_cont) {
            (true, true) => BoundaryKind::Orogeny,
            // The continental side overrides (arc); the oceanic side subducts.
            (true, false) => BoundaryKind::Arc,
            (false, true) => BoundaryKind::Trench,
            (false, false) => BoundaryKind::Arc, // island arc on this side
        }
    } else if vc_norm < -0.1 {
        if a_cont {
            BoundaryKind::Rift
        } else {
            BoundaryKind::Ridge
        }
    } else {
        BoundaryKind::Transform
    }
}

/// The thickening amplitude (dimensionless, × `thickening_scale`) for a boundary
/// kind at a normalized convergence. Positive = thicken, negative = thin.
fn type_amplitude(kind: BoundaryKind, vc_norm: f64) -> f64 {
    let rate = vc_norm.abs().min(2.0); // saturating convergence factor
    match kind {
        BoundaryKind::Orogeny => rate,
        BoundaryKind::Arc => 0.5 * rate,
        BoundaryKind::Trench => -0.35 * rate,
        BoundaryKind::Rift => -0.30 * rate,
        BoundaryKind::Ridge => -0.05 * rate,
        BoundaryKind::Transform | BoundaryKind::Interior => 0.0,
    }
}

/// Reference velocity (km/chapter) that normalizes convergence — the per-chapter
/// advection step, so `vc_norm` is O(1) at a typical closing boundary regardless
/// of the (compressed) absolute speed.
pub fn reference_velocity(cfg: &DeepConfig, extent_km: f64) -> f64 {
    let n = plate_count(extent_km, cfg.plate_scale_km);
    let d = plate_diameter_km(extent_km, n);
    (cfg.advection_plate_widths * d / f64::from(cfg.chapters.max(1))).max(1e-6)
}

/// The **analytic thickening forcing** at world position `(x_km, y_km)` for the
/// given chapter's advected plate set (§ 4.1). Metres of crustal thickness per
/// iteration; sums the contributions of the boundaries to the `k = 3` nearest
/// seeds, each a smooth compact bump with the boundary's sidedness offset. No
/// grid term anywhere — the only wavelengths are `W` and the arc/trench offsets.
pub fn forcing_at(plates: &[Plate], cfg: &DeepConfig, v_ref: f64, x: f64, y: f64) -> f64 {
    let near = nearest_k(plates, x, y, 3);
    if near.len() < 2 {
        return 0.0;
    }
    let (ai, _) = near[0];
    let a = &plates[ai];
    let w0 = cfg.orogen_width_km.max(1.0);
    let mut acc = 0.0;
    for &(bi, _) in near.iter().skip(1) {
        let b = &plates[bi];
        let vc = convergence(a, b);
        let vc_norm = vc / v_ref;
        let kind = classify(vc_norm, a.continental, b.continental);
        let amp = type_amplitude(kind, vc_norm);
        if amp == 0.0 {
            continue;
        }
        let d = signed_bisector_distance(a, b, x, y);
        // Sidedness: orogeny/rift/ridge are symmetric bumps on the boundary
        // (centre d≈0); arcs sit inland on the overriding side (positive offset),
        // trenches hug the boundary on the subducting side.
        let (offset, width) = match kind {
            BoundaryKind::Orogeny => {
                // Fast convergence → narrow sharp belt (1/√v_conv), the ratified
                // intent (earth-processes § 1 item 2).
                (0.0, w0 / vc_norm.abs().max(0.3).sqrt())
            }
            BoundaryKind::Arc => (cfg.arc_gap_km, w0 * 0.8),
            BoundaryKind::Trench => (0.15 * cfg.arc_gap_km, w0 * 0.5),
            BoundaryKind::Rift => (0.0, w0),
            BoundaryKind::Ridge => (0.0, w0),
            BoundaryKind::Transform | BoundaryKind::Interior => (0.0, w0),
        };
        let dd = (d - offset) / width;
        acc += amp * (-(dd * dd)).exp();
    }
    acc * cfg.thickening_scale
}

/// The dominant boundary kind at `(x_km, y_km)` for a chapter's plates — the
/// analytic provenance query (§ 4.3), for the measurement probe. Returns the kind
/// whose |amplitude·kernel| dominates, else [`BoundaryKind::Interior`].
pub fn dominant_kind(
    plates: &[Plate],
    cfg: &DeepConfig,
    v_ref: f64,
    x: f64,
    y: f64,
) -> BoundaryKind {
    let near = nearest_k(plates, x, y, 3);
    if near.len() < 2 {
        return BoundaryKind::Interior;
    }
    let (ai, _) = near[0];
    let a = &plates[ai];
    let w0 = cfg.orogen_width_km.max(1.0);
    let mut best = (0.0f64, BoundaryKind::Interior);
    for &(bi, _) in near.iter().skip(1) {
        let b = &plates[bi];
        let vc_norm = convergence(a, b) / v_ref;
        let kind = classify(vc_norm, a.continental, b.continental);
        let amp = type_amplitude(kind, vc_norm);
        if amp == 0.0 {
            continue;
        }
        let d = signed_bisector_distance(a, b, x, y);
        let width = w0;
        let dd = d / width;
        let mag = (amp * (-(dd * dd)).exp()).abs();
        if mag > best.0 {
            best = (mag, kind);
        }
    }
    best.1
}

/// Crust kind of a deep cell, for the isostasy density lookup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrustKind {
    Continental,
    Oceanic,
    /// Transitional (continental shelf): intermediate density.
    Transitional,
}

impl CrustKind {
    #[inline]
    pub fn index(self) -> u8 {
        match self {
            CrustKind::Continental => 0,
            CrustKind::Oceanic => 1,
            CrustKind::Transitional => 2,
        }
    }
    #[inline]
    pub fn from_index(i: u8) -> Self {
        match i {
            0 => CrustKind::Continental,
            1 => CrustKind::Oceanic,
            _ => CrustKind::Transitional,
        }
    }
    /// Seed crustal thickness (m) for this kind (§ 5.1): continental ~35 km,
    /// oceanic ~7 km, transitional ~20 km.
    #[inline]
    pub fn seed_thickness(self) -> f64 {
        match self {
            CrustKind::Continental => 35_000.0,
            CrustKind::Oceanic => 7_000.0,
            CrustKind::Transitional => 20_000.0,
        }
    }
}

/// Seed the per-deep-cell crustal columns (§ 5.2): `crust_kind` from the
/// chapter-0 plate the cell belongs to (with a shelf band where a continental
/// cell sits near oceanic neighbours), `t_crust` from the kind plus an addressed
/// jitter. Returns `(t_crust, crust_kind_index)`, row-major `w × w`.
pub fn seed_columns(cfg: &DeepConfig, w: usize, cell_m: f64) -> (Vec<f64>, Vec<u8>) {
    let extent_km = w as f64 * cell_m / 1000.0;
    let base = build_plates(cfg, extent_km);
    let mut t_crust = vec![0.0f64; w * w];
    let mut kind = vec![0u8; w * w];
    // Pass 1: continental/oceanic from the nearest plate.
    for gy in 0..w {
        for gx in 0..w {
            let x = (gx as f64 + 0.5) * cell_m / 1000.0;
            let y = (gy as f64 + 0.5) * cell_m / 1000.0;
            let near = nearest_k(&base, x, y, 1);
            let cont = near.first().is_some_and(|&(i, _)| base[i].continental);
            kind[gy * w + gx] = if cont {
                CrustKind::Continental.index()
            } else {
                CrustKind::Oceanic.index()
            };
        }
    }
    // Pass 2: a continental cell adjacent to oceanic crust is a shelf
    // (transitional). Then seed thickness from kind + jitter.
    let snapshot = kind.clone();
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let mut k = CrustKind::from_index(snapshot[i]);
            if k == CrustKind::Continental {
                let near_ocean =
                    [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .any(|&(dx, dy)| {
                            let (nx, ny) = (gx as i32 + dx, gy as i32 + dy);
                            nx >= 0
                                && ny >= 0
                                && (nx as usize) < w
                                && (ny as usize) < w
                                && CrustKind::from_index(snapshot[ny as usize * w + nx as usize])
                                    == CrustKind::Oceanic
                        });
                if near_ocean {
                    k = CrustKind::Transitional;
                }
            }
            kind[i] = k.index();
            let jit = draw_f64(&[cfg.seed, SALT_TEC_CRUST, gx as u64, gy as u64]) * 2.0 - 1.0;
            t_crust[i] = k.seed_thickness() * (1.0 + 0.06 * jit);
        }
    }
    (t_crust, kind)
}
