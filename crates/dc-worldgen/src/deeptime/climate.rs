//! Orographic precipitation for the deep-time grid: a prevailing-wind 1D
//! moisture march with a cross-wind blur, re-marched on the *current*
//! topography every ~20 iterations (never the final topography — the
//! anachronism orogeny's first paleoclimate version shipped, recon § paleo).
//!
//! The plane is **precipitation**, not residual humidity: air saturates over
//! sea, rains out in proportion to the windward uplift it climbs, and depletes
//! downwind, so a lee sits dry *because* the range in front of it is there. The
//! 1D advection streaks along rows; a box blur across the wind (orogeny's fix)
//! spreads the signal so a north–south range shadows a coherent lee band
//! instead of a comb of dry rows.

use super::grid::DeepGrid;

/// Zonal wind x-step for a latitude band (trade easterlies, mid-latitude
/// westerlies, polar easterlies) — the pregen climate bands, reused.
pub fn wind_dx(lat_deg: f64) -> i32 {
    if lat_deg < 30.0 {
        -1
    } else if lat_deg < 60.0 {
        1
    } else {
        -1
    }
}

/// Re-march precipitation over the current surface. `surf` is `R + H`;
/// `sea_level` is the current paleo-stand. Writes normalized precip (0..1) into
/// `grid.precip`.
pub fn march(grid: &mut DeepGrid, sea_level: f64) {
    let w = grid.w;
    let mut raw = vec![0.0f32; w * w];
    for gy in 0..w {
        let lat = grid.lat_deg(gy);
        let dx = wind_dx(lat);
        // March order along x, in wind direction.
        let xs: Vec<usize> = if dx > 0 {
            (0..w).collect()
        } else {
            (0..w).rev().collect()
        };
        let mut moisture = 1.0f64;
        let mut prev_elev = -100.0f64;
        for gx in xs {
            let i = gy * w + gx;
            let elev = grid.surf_at(i);
            if elev <= sea_level {
                // Over sea: air re-saturates; nominal over-ocean rainfall.
                moisture = 1.0;
                raw[i] = 0.6;
            } else {
                let uplift = ((elev - prev_elev.max(0.0)) / 1000.0).max(0.0);
                let frac = (0.18 + 1.8 * uplift).min(0.85);
                let p = (moisture * frac).min(1.0);
                raw[i] = p as f32;
                // Partial depletion + weak evapotranspiration recharge keeps
                // interiors semi-arid rather than bone dry (pregen climate).
                moisture = (moisture - 0.5 * p + 0.03).clamp(0.0, 1.0);
            }
            prev_elev = elev;
        }
    }
    // Cross-wind box blur (3-wide, perpendicular to the zonal flow ≈ the y
    // axis) to break row streaking. Winds are zonal, so blur along y.
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let mut sum = 0.0f32;
            let mut n = 0.0f32;
            for dy in -1i32..=1 {
                let ny = gy as i32 + dy;
                if ny >= 0 && (ny as usize) < w {
                    sum += raw[ny as usize * w + gx];
                    n += 1.0;
                }
            }
            grid.precip[i] = sum / n;
        }
    }
}
