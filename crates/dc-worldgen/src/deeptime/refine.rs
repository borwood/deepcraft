//! Architecture **C**: bounded regional refinement and the empirical
//! decay-length measurement that decides whether it is sound.
//!
//! The halo theorem (orogeny recon § bounded/unbounded): a *relaxation* process
//! has a finite influence radius, so a region can be refined with a halo and
//! its interior will match a global fine run once the halo is wide enough; an
//! *advective* process (drainage — no halo bounds a basin) does not. To size a
//! halo honestly you must **measure** the decay, not assume the propagation
//! speed (orogeny found the decay-length halo ~4× cheaper than the theory
//! demanded).
//!
//! The experiment here builds a small fine domain over a pregen sub-window
//! (small enough to afford a full "global fine" reference), then perturbs only
//! its outer `halo` ring — the boundary-condition error a real refined region
//! inherits from the coarse tier — and measures how far that error penetrates.
//! A *small* bump probes the relaxation decay; a *large* bump reroutes drainage
//! and exposes the advective wall (the error stops decaying). Both fall out of
//! the same function at two magnitudes.

use dc_sim::statistical::rng::draw_f64;

use crate::pregen::{CELL_VOXELS, Pregen, provenance_roughness};

use super::climate;
use super::erosion::Erosion;
use super::grid::{DeepConfig, DeepGrid, provenance_uplift};

/// Addressed-draw salt for the boundary-condition perturbation.
const SALT_DT_PERTURB: u64 = 0x5900_0002;

/// A square fine sub-window of the pregen extent to refine.
#[derive(Clone, Copy, Debug)]
pub struct RegionSpec {
    /// Min-corner of the window in continuous pregen-cell coordinates.
    pub px0: f64,
    pub py0: f64,
    /// Fine cells per side.
    pub cells: usize,
    /// Fine cell edge, metres.
    pub cell_m: f64,
}

/// Per-ring error of a perturbed refinement vs. the reference fine run, indexed
/// by Chebyshev distance (in cells) from the domain boundary.
pub struct DecayProfile {
    pub cell_m: f64,
    pub halo: usize,
    pub bump_m: f64,
    /// `(distance_cells, max_err_m, mean_err_m, cell_count)`, inner→outer.
    pub rings: Vec<(usize, f64, f64, u64)>,
}

impl DecayProfile {
    /// Cells *beyond the perturbed halo edge* to which the boundary error still
    /// exceeds `threshold` — the deepest offending interior ring. Rings inside
    /// the perturbed source band (`d < halo`) are excluded; they carry the bump
    /// itself, not its penetration. Uses the deepest offending ring, not the
    /// first quiet one, because the error field is non-monotonic (an isolated
    /// drainage reroute spikes deep inside an otherwise quiet interior — the
    /// advective signal). Zero = the error dies at the halo edge.
    pub fn penetration_cells(&self, threshold: f64) -> usize {
        self.rings
            .iter()
            .filter(|r| r.0 >= self.halo && r.1 >= threshold)
            .map(|r| r.0)
            .max()
            .map_or(0, |d| d + 1 - self.halo)
    }

    /// Bulk envelope (cells beyond the halo edge): the shallowest ring past the
    /// source band whose error has fallen below `threshold` — how fast the
    /// *bulk* field relaxes, ignoring deep isolated spikes. For a pure
    /// relaxation process this equals [`Self::penetration_cells`]; the gap
    /// between them is the advective (drainage) reach.
    pub fn envelope_cells(&self, threshold: f64) -> usize {
        for r in &self.rings {
            if r.0 >= self.halo && r.1 < threshold {
                return r.0 - self.halo;
            }
        }
        self.rings.len().saturating_sub(self.halo)
    }

    /// True when the boundary error decays below `threshold` before the domain
    /// centre — the halo theorem holds (relaxation). False = advective.
    pub fn decays_within_domain(&self, threshold: f64) -> bool {
        self.penetration_cells(threshold) + self.halo <= self.rings.len().saturating_sub(1)
    }

    /// The residual error floor (metres): the smallest per-ring max error
    /// reached anywhere in the interior.
    pub fn error_floor(&self) -> f64 {
        self.rings.iter().map(|r| r.1).fold(f64::INFINITY, f64::min)
    }

    /// The deepest ring's error (metres): how much the boundary condition still
    /// disturbs the domain centre.
    pub fn interior_error(&self) -> f64 {
        self.rings.last().map_or(0.0, |r| r.1)
    }
}

/// Build a fine deep-time grid over a rectangular pregen-coordinate window.
/// Same construction as [`super::grid::build`] but restricted to the window, so
/// the domain is small enough to run a full reference.
fn build_window(pregen: &Pregen, cfg: &DeepConfig, region: &RegionSpec) -> DeepGrid {
    let w = region.cells;
    // Pregen-cell coordinates span of the window.
    let span_pregen = region.cells as f64 * region.cell_m / (CELL_VOXELS as f64 * 0.9);
    let sample = |px: f64, py: f64| -> (f64, f64, f64) {
        // Bilinear over pregen cell centres (clamped), returning
        // (elev, roughness, uplift_rate).
        let wp = pregen.grid.w;
        let cx = px.clamp(0.0, f64::from(wp) - 1.0);
        let cy = py.clamp(0.0, f64::from(wp) - 1.0);
        let x0 = cx.floor() as i32;
        let y0 = cy.floor() as i32;
        let x1 = (x0 + 1).min(wp - 1);
        let y1 = (y0 + 1).min(wp - 1);
        let fx = cx - f64::from(x0);
        let fy = cy - f64::from(y0);
        let get = |gx: i32, gy: i32| {
            let c = pregen.grid.get(gx, gy).expect("clamped in grid");
            (
                c.elev_m,
                provenance_roughness(c.provenance),
                provenance_uplift(c.provenance),
            )
        };
        let (e00, r00, u00) = get(x0, y0);
        let (e10, r10, u10) = get(x1, y0);
        let (e01, r01, u01) = get(x0, y1);
        let (e11, r11, u11) = get(x1, y1);
        let lerp = |a: f64, b: f64, c: f64, d: f64| {
            let top = a * (1.0 - fx) + b * fx;
            let bot = c * (1.0 - fx) + d * fx;
            top * (1.0 - fy) + bot * fy
        };
        (
            lerp(e00, e10, e01, e11),
            lerp(r00, r10, r01, r11),
            lerp(u00, u10, u01, u11) * cfg.uplift_scale,
        )
    };

    let n = w * w;
    let mut r = vec![0.0f64; n];
    let mut uplift = vec![0.0f64; n];
    for gy in 0..w {
        for gx in 0..w {
            let px = region.px0 + (gx as f64 + 0.5) / w as f64 * span_pregen;
            let py = region.py0 + (gy as f64 + 0.5) / w as f64 * span_pregen;
            let (elev, rough, up) = sample(px, py);
            let i = gy * w + gx;
            let jitter = (draw_f64(&[cfg.seed, super::grid::SALT_DT_ROUGH, gx as u64, gy as u64])
                * 2.0
                - 1.0)
                * rough
                * cfg.rough_jitter;
            r[i] = elev + jitter;
            uplift[i] = up;
        }
    }
    DeepGrid::from_parts(w, region.cell_m, r, uplift)
}

/// Chebyshev distance (cells) to the nearest domain boundary.
fn ring_dist(gx: usize, gy: usize, w: usize) -> usize {
    gx.min(gy).min(w - 1 - gx).min(w - 1 - gy)
}

/// Perturb the outer `halo`-cell ring's bedrock by up to `±bump_m` (addressed
/// jitter): the boundary-condition error a refined region inherits.
fn perturb_ring(grid: &mut DeepGrid, halo: usize, bump_m: f64, seed: u64) {
    let w = grid.w;
    for gy in 0..w {
        for gx in 0..w {
            if ring_dist(gx, gy, w) < halo {
                let i = gy * w + gx;
                let j = draw_f64(&[seed, SALT_DT_PERTURB, gx as u64, gy as u64]) * 2.0 - 1.0;
                grid.r[i] += j * bump_m;
            }
        }
    }
}

/// Run the decay experiment: a reference fine run vs. an identical run whose
/// outer `halo` ring bedrock is perturbed by `bump_m`, both over the same
/// window. Returns the per-ring error profile (erosion-only, no recorder).
pub fn measure_decay(
    pregen: &Pregen,
    cfg: &DeepConfig,
    region: &RegionSpec,
    halo: usize,
    bump_m: f64,
) -> DecayProfile {
    let mut run_cfg = *cfg;
    run_cfg.record = false;

    let mut reference = build_window(pregen, &run_cfg, region);
    let mut perturbed = build_window(pregen, &run_cfg, region);
    perturb_ring(&mut perturbed, halo, bump_m, run_cfg.seed);

    let mut e_ref = Erosion::new(&reference);
    let mut e_pert = Erosion::new(&perturbed);
    let geology = dc_core::materials::geology::vanilla();
    let mem = super::recorder::MemberCtx::new(&geology, run_cfg.seed, 0);
    let sl0 = super::grid::sea_level_at(&run_cfg, 0);
    climate::march(&mut reference, sl0);
    climate::march(&mut perturbed, sl0);
    for it in 0..run_cfg.iterations {
        let sl = super::grid::sea_level_at(&run_cfg, it);
        if it > 0 && it % run_cfg.remarch_interval == 0 {
            climate::march(&mut reference, sl);
            climate::march(&mut perturbed, sl);
        }
        // The decay experiment runs with the recorder OFF (`run_cfg.record`
        // is false), so no identity is ever written and the content set is
        // never consulted; it is passed explicitly rather than defaulted so
        // the day this experiment does record, the omission is a compile
        // error and not a silent vanilla assumption (P11 slice 1).
        e_ref.step(&mut reference, &run_cfg, sl, mem);
        e_pert.step(&mut perturbed, &run_cfg, sl, mem);
    }

    let w = reference.w;
    let bins = w / 2 + 1;
    let mut max_err = vec![0.0f64; bins];
    let mut sum_err = vec![0.0f64; bins];
    let mut cnt = vec![0u64; bins];
    for gy in 0..w {
        for gx in 0..w {
            let d = ring_dist(gx, gy, w);
            let i = gy * w + gx;
            let e = (reference.surf_at(i) - perturbed.surf_at(i)).abs();
            max_err[d] = max_err[d].max(e);
            sum_err[d] += e;
            cnt[d] += 1;
        }
    }
    let rings = (0..bins)
        .map(|d| {
            let c = cnt[d].max(1);
            (d, max_err[d], sum_err[d] / c as f64, cnt[d])
        })
        .collect();
    DecayProfile {
        cell_m: region.cell_m,
        halo,
        bump_m,
        rings,
    }
}
