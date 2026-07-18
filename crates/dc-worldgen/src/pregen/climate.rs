//! Climate: compressed latitude bands, prevailing zonal winds, and a
//! moisture-advection march that produces orographic precipitation and rain
//! shadow from the tectonic elevation.
//!
//! Winds are purely zonal by band (trade easterlies below 30°, westerlies to
//! 60°, polar easterlies above). Each row is marched in wind order: moisture
//! saturates over ocean, precipitates over land in proportion to uplift, and
//! depletes downwind — the lee of a mountain belt is dry *because* the belt
//! is there, which is the property the pipeline test asserts.

use super::{CellGrid, latitude_deg, temp_sea_level};

/// Zonal wind direction for a latitude: the sign of the x-step air takes.
pub fn wind_dx(lat_deg: f64) -> i32 {
    if lat_deg < 30.0 {
        -1
    } else if lat_deg < 60.0 {
        1
    } else {
        -1
    }
}

pub fn apply(grid: &mut CellGrid) {
    let w = grid.w;
    for gy in 0..w {
        let lat = latitude_deg(w, f64::from(gy) + 0.5);
        let dx = wind_dx(lat);
        let xs: Vec<i32> = if dx > 0 {
            (0..w).collect()
        } else {
            (0..w).rev().collect()
        };
        // Upwind of the grid is world ocean: air arrives saturated.
        let mut moisture = 1.0f64;
        let mut prev_elev = -100.0f64;
        for gx in xs {
            let i = grid.idx(gx, gy).expect("in grid");
            let c = &mut grid.cells[i];
            c.lat_deg = lat;
            c.temp_c = temp_sea_level(lat) - 6.5 * c.elev_m.max(0.0) / 1000.0;
            if c.elev_m <= 0.0 {
                moisture = 1.0;
                c.precip = 0.6; // over-ocean rainfall; unused by drainage
            } else {
                let uplift = ((c.elev_m - prev_elev.max(0.0)) / 1000.0).max(0.0);
                let frac = (0.18 + 1.8 * uplift).min(0.85);
                let p = (moisture * frac).min(1.0);
                c.precip = p;
                // Partial depletion + weak evapotranspiration recharge keeps
                // continental interiors semi-arid rather than bone dry while
                // preserving the orographic shadow.
                moisture = (moisture - 0.5 * p + 0.03).clamp(0.0, 1.0);
            }
            prev_elev = c.elev_m;
        }
    }
}
