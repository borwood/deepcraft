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
//!
//! **The circulation profile is shared with pregen** (journal/0037): the zonal
//! wind is a smooth signed magnitude ([`zonal_wind`]) that eases through a calm
//! belt at each band boundary, and the moisture march runs the identical
//! per-cell [`rainout`] physics — orographic wring-out plus a subsidence gate
//! ([`subsidence`]) that makes the 30° Hadley descending limb a desert with no
//! mountain in front of it. Deep time and pregen therefore share one climate
//! model, not two that can drift apart.

use super::grid::DeepGrid;

/// The circulation profile — defined once in [`crate::pregen::climate`] and
/// re-exported here so the deep-time consumers (this march and the eolian agent)
/// read the same wind and subsidence as pregen. `zonal_wind` is a signed 0..1
/// magnitude (the eolian agent scales deflation by `|zonal_wind|`); `wind_dir`
/// is its sign; `subsidence` is the precip-suppression factor; `rainout` is the
/// shared per-land-cell march step.
pub use crate::pregen::climate::{rainout, subsidence, wind_dir, zonal_wind};

/// Air temperature (°C) at a cell: a latitude gradient minus an altitude lapse.
/// The **single** climate temperature model deep time carries — the biotic layer
/// (`super::biotic`) and the frost agent (`super::erosion`) both read it, so
/// "where does it freeze" has one honest answer. Deep time has no pregen
/// `temp_c` plane (only precipitation is marched), but latitude and the eroding
/// surface *are* present every epoch, and that is enough for an honest gate:
/// `30 − 0.55·|lat|` at sea level, cooled `6.5 °C/km` up the current surface.
#[inline]
pub fn air_temp_c(lat_deg: f64, surf: f64) -> f32 {
    let lat = lat_deg.abs();
    let sea_temp = 30.0 - 0.55 * lat;
    let lapse = 6.5 * (surf.max(0.0) / 1000.0);
    (sea_temp - lapse) as f32
}

/// Re-march precipitation over the current surface. `surf` is `R + H`;
/// `sea_level` is the current paleo-stand. Writes normalized precip (0..1) into
/// `grid.precip`.
pub fn march(grid: &mut DeepGrid, sea_level: f64) {
    let w = grid.w;
    let mut raw = vec![0.0f32; w * w];
    for gy in 0..w {
        let lat = grid.lat_deg(gy);
        let dir = wind_dir(lat);
        // March order along x, in wind direction.
        let xs: Vec<usize> = if dir > 0 {
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
                // The exact same land-cell physics as the pregen march: moisture-
                // and subsidence-modulated rainout with a local convective floor.
                let (p, m) = rainout(moisture, uplift, lat);
                raw[i] = p as f32;
                moisture = m;
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
