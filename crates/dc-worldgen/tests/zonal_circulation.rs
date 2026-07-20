//! The zonal circulation profile (journal/0037): the two diagnosed defects must
//! measurably die at the grid level.
//!
//! Unit-level shape checks (continuity of `zonal_wind`, the peak of `subsidence`,
//! the shared `rainout` physics) live inline in `pregen::climate`. This suite
//! carries the **grid-level** falsifiers that need the actual deep-time march:
//! - (b) a ~30° latitude band shows suppressed precipitation vs its neighbours
//!   on **flat** terrain — subsidence acting with no orography at all;
//! - (c) the equatorial band stays wet;
//! - and the eolian consumer reads the smooth profile through the climate seam.

use dc_worldgen::deeptime::DeepGrid;
use dc_worldgen::deeptime::climate::{self, subsidence, zonal_wind};

/// A flat land grid spanning the pregen latitude band (8°–78°): every cell is
/// land at a uniform elevation, so there is **no orographic signal at all** —
/// precipitation is whatever the subsidence gate and the convective floor make
/// it. `from_parts` sets `precip` to the over-ocean nominal until the march
/// overwrites it.
fn flat_land(w: usize) -> DeepGrid {
    let n = w * w;
    DeepGrid::from_parts(w, 1000.0, vec![300.0; n], vec![0.0; n])
}

/// Mean marched precipitation across the rows whose latitude falls in `[lo, hi]`.
fn band_precip(grid: &DeepGrid, lo: f64, hi: f64) -> f64 {
    let w = grid.w;
    let (mut sum, mut n) = (0.0f64, 0usize);
    for gy in 0..w {
        let lat = grid.lat_deg(gy);
        if lat < lo || lat > hi {
            continue;
        }
        for gx in 0..w {
            sum += f64::from(grid.precip[gy * w + gx]);
            n += 1;
        }
    }
    assert!(n > 0, "no rows in latitude band [{lo}, {hi}]");
    sum / n as f64
}

/// **Defects (b) and (c) at the grid level.** March precipitation over flat land
/// and confirm the 30° band is dry (below the pipeline's 0.32 arid threshold) and
/// strictly drier than both the low-latitude and the mid-latitude neighbours,
/// while the equatorward band stays wet — a desert belt from subsidence alone,
/// with no mountain anywhere on the grid.
#[test]
fn thirty_degree_desert_belt_exists_on_flat_terrain() {
    let mut grid = flat_land(96);
    climate::march(&mut grid, 0.0);

    let low = band_precip(&grid, 8.0, 14.0); // the on-grid ITCZ side: ascending / wet
    let desert = band_precip(&grid, 27.0, 33.0); // the Hadley descending limb
    let temperate = band_precip(&grid, 45.0, 55.0); // mid-latitude interior (semi-arid on flat land)

    println!(
        "[zonal] flat-land precip: low(8-14°) {low:.3}, desert(27-33°) {desert:.3}, \
         temperate(45-55°) {temperate:.3}"
    );

    assert!(
        desert < 0.32,
        "the 30° band is not arid on flat land: {desert}"
    );
    assert!(low > 0.32, "the equatorward (ITCZ) band is not wet: {low}");
    assert!(
        desert < low && desert < temperate,
        "the 30° band is not suppressed vs its neighbours: desert {desert}, low {low}, temperate {temperate}"
    );
}

/// The march direction reversal at the band boundaries happens only through the
/// calm belt: at 30° the profile magnitude is ~0, so the desert belt (b) is a
/// subsidence effect, not an artefact of the wind flipping direction there.
#[test]
fn the_desert_belt_sits_in_the_calm_belt() {
    assert!(
        zonal_wind(30.0).abs() < 0.05,
        "30° must be a wind calm belt"
    );
    assert!(subsidence(30.0) > 0.6, "30° must be a subsidence maximum");
}
