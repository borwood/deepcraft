//! Climate: compressed latitude bands, prevailing zonal winds, and a
//! moisture-advection march that produces orographic precipitation and rain
//! shadow from the tectonic elevation.
//!
//! **The circulation is a smooth latitude profile, not a three-way sign bit**
//! (journal/0037). Reality has three zonal cells — easterly trades below ~30°,
//! mid-latitude westerlies to ~60°, polar easterlies above — but the wind does
//! not *snap* between them: it eases through a calm belt at each boundary (the
//! horse latitudes at 30°, the polar front at 60°). [`zonal_wind`] returns a
//! signed *magnitude* that passes smoothly through ~zero at those boundaries, so
//! adjacent rows never carry full-strength wind in opposite directions — the
//! analytic-boundary "scream" (a dead-straight climate/vegetation line, dune
//! fields migrating opposite ways across one grid row) is gone by construction.
//!
//! The same boundaries carry the second half of the mechanism. The 30° horse
//! latitudes are the *descending limb of the Hadley cell*: dry air sinking,
//! suppressing rain. That is why Earth's great deserts ring 30°, independent of
//! any mountain. [`subsidence`] is that limb — a precipitation-suppression
//! factor peaking at 30° (and mildly at the poles), so a desert belt exists
//! **without** a rain shadow. Calm belt and desert belt are one object: the
//! wind eases *because* the air is sinking, and the air sinking *is* the desert.
//!
//! Each row is marched in wind order: moisture saturates over ocean and rains
//! out over land in proportion to windward uplift and to the subsidence-gated
//! moisture it still carries — the lee of a mountain belt is dry *because* the
//! belt is there (the rain-shadow property the pipeline test asserts), and the
//! 30° belt is dry *because* the air is descending there.

use super::{CellGrid, latitude_deg, temp_sea_level};

/// Peak surface wind magnitude of each zonal cell, normalized so the strongest
/// (the trade easterlies) is `1.0`. Earth's surface means run roughly
/// trades ≳ westerlies ≫ polar easterlies (~7 : ~6 : ~3 m/s); these are that
/// ratio, and the eolian deflation scaling reads them directly as a 0..1 wind
/// strength.
const TRADE_AMP: f64 = 1.0;
const WESTERLY_AMP: f64 = 0.9;
const POLAR_AMP: f64 = 0.45;

/// Peak precipitation suppression (0 = none, 1 = total) of the subtropical
/// descending limb, centred on 30° with a Gaussian half-width `SUBTROPICAL_SIGMA`
/// (FWHM ≈ 16°, so the desert belt spans roughly 22–38° — Earth's subtropical
/// deserts sit ~15–35°). Strong, because this limb is the primary aridity engine.
const SUBTROPICAL_SUBSIDENCE: f64 = 0.75;
const SUBTROPICAL_SIGMA: f64 = 7.0;
/// The polar high is a second, weaker descending limb: cold-dry, centred on the
/// pole with a broad `POLAR_SIGMA`, so its shoulder reaches the grid's northern
/// edge (78°) as a mild polar-desert hint without touching the wet 60° front.
const POLAR_SUBSIDENCE: f64 = 0.5;
const POLAR_SIGMA: f64 = 12.0;

/// Background-precipitation floor parameters (see [`convective_floor`]). Local
/// background rain is not a flat baseline: it exists only where air *rises*. Two
/// ascending sources give it — the equatorial ITCZ (deep tropical convection,
/// `ITCZ_*`, peaking at the equator) and the ~60° polar front (mid-latitude
/// storm track, `FRONT_*`). Between them the mid-latitude continental interior
/// gets only `FLOOR_BASE`: a flat, ocean-far interior at 40–55° is genuinely
/// semi-arid (the world's steppes), so it is *not* forced humid — which also
/// keeps interior runoff, and the rivers the collapse layer carves from it, near
/// the historical regime instead of cutting a canyon across every dry interior.
const FLOOR_BASE: f64 = 0.08;
const ITCZ_FLOOR: f64 = 0.40;
const ITCZ_SIGMA: f64 = 15.0;
const FRONT_FLOOR: f64 = 0.12;
const FRONT_SIGMA: f64 = 8.0;

/// Signed prevailing zonal wind at a latitude, as a normalized magnitude in
/// `[-1, 1]`: negative = easterly (air steps toward −x), positive = westerly
/// (+x). Peaks at each cell's centre and passes **smoothly through zero** at the
/// band boundaries.
///
/// Each of the three cells is a squared-sine lobe over its `[a, b]` band —
/// `sin(π·phase)²` — which is `0` *and* has zero slope at both edges. So the
/// whole profile is C¹-continuous across the 30° and 60° boundaries: the sign
/// only ever changes through a near-zero magnitude, never as a step. Signs match
/// the historical three-way bit in every band interior (easterly < 30°, westerly
/// 30–60°, easterly > 60°), so nothing downwind of the interiors moves; only the
/// hard flip at the boundaries is replaced by a calm belt.
///
/// Calm-belt width: with the squared-sine lobe `|wind|` falls below 10% of the
/// band's peak within ~10% of the band width of each edge — about ±3° for a 30°
/// band, i.e. a ~6°-wide calm belt straddling each boundary, matching the real
/// horse-latitude and polar-front breadth.
pub fn zonal_wind(lat_deg: f64) -> f64 {
    let lat = lat_deg.abs();
    let (a, b, amp, sign) = if lat < 30.0 {
        (0.0, 30.0, TRADE_AMP, -1.0)
    } else if lat < 60.0 {
        (30.0, 60.0, WESTERLY_AMP, 1.0)
    } else {
        (60.0, 90.0, POLAR_AMP, -1.0)
    };
    let phase = ((lat - a) / (b - a)).clamp(0.0, 1.0);
    let lobe = (std::f64::consts::PI * phase).sin().powi(2);
    sign * amp * lobe
}

/// Direction the moisture march (and the eolian march) step a row: the sign of
/// [`zonal_wind`]. Exactly on a calm-belt zero the wind has no direction, so we
/// default easterly (`-1`); the magnitude there is ~0, so the choice is
/// climatically inert (weak advection, local floor dominates — see [`rainout`]).
/// In every band interior this equals the historical three-way sign bit.
pub fn wind_dir(lat_deg: f64) -> i32 {
    if zonal_wind(lat_deg) > 0.0 { 1 } else { -1 }
}

/// Precipitation-suppression factor `[0, 1]` from descending-air subsidence:
/// `0` = no suppression, `1` = fully suppressed. Two Gaussian limbs — the strong
/// subtropical high at 30° (the Hadley descending limb, Earth's desert belt) and
/// a weaker, broad polar high at 90°. Deliberately **near-zero at the equator**
/// (the ITCZ is the ascending wet belt) and **near-zero at 60°** (the polar
/// front is ascending — the calm belt there is a *wet* storm track, not a
/// desert). So of the two wind calm belts, only the 30° one is also dry: that is
/// the physical unity this profile encodes.
pub fn subsidence(lat_deg: f64) -> f64 {
    let lat = lat_deg.abs();
    let gauss = |mu: f64, sigma: f64| (-0.5 * ((lat - mu) / sigma).powi(2)).exp();
    (SUBTROPICAL_SUBSIDENCE * gauss(30.0, SUBTROPICAL_SIGMA)
        + POLAR_SUBSIDENCE * gauss(90.0, POLAR_SIGMA))
    .clamp(0.0, 1.0)
}

/// Background convective/frontal precipitation floor by latitude (before the
/// subsidence gate). A latitude-shaped baseline, **not** a flat one: local
/// background rain exists only where air rises. A strong equatorial ITCZ bump
/// keeps low-latitude flat land wet with no orography; a modest polar-front bump
/// at 60° keeps that wet calm belt from reading bone-dry; and between them the
/// mid-latitude interior gets only `FLOOR_BASE`, staying honestly semi-arid (and
/// keeping interior runoff near the historical regime — see [`FLOOR_BASE`]).
pub fn convective_floor(lat_deg: f64) -> f64 {
    let lat = lat_deg.abs();
    let gauss = |mu: f64, sigma: f64| (-0.5 * ((lat - mu) / sigma).powi(2)).exp();
    FLOOR_BASE + ITCZ_FLOOR * gauss(0.0, ITCZ_SIGMA) + FRONT_FLOOR * gauss(60.0, FRONT_SIGMA)
}

/// One land-cell step of the moisture march — the **shared climate physics** the
/// pregen march ([`apply`]) and the deep-time re-march
/// ([`crate::deeptime::climate::march`]) both run, so paleoclimate and present
/// climate never diverge in mechanism.
///
/// Given the incoming advected `moisture` (0..1), the windward `uplift`
/// (kilometres climbed since the previous cell, ≥ 0), and `lat_deg`, returns
/// `(precip, moisture_out)`.
///
/// Rainout is modulated by **both** the moisture the front carries and the
/// subsidence gate: `precip = max(orographic, floor(lat))·supp`, where
/// `supp = 1 − subsidence(lat)` and `floor(lat)` is the latitude-shaped
/// [`convective_floor`]. The orographic term is the classic wring-out-on-the-climb
/// that makes lees dry; the floor is the local background baseline that keeps the
/// equatorial ITCZ and the 60° front from going bone-dry, **without** inflating a
/// wet windward slope (the floor is a floor, not an addition — so the change to
/// already-wet ground is only the subsidence gate) and **without** forcing the
/// semi-arid mid-latitude interior humid (the floor is small there). The gate
/// then drives the 30° descending limb and the poles below the arid threshold
/// with *no* mountain in front of them, while the equator keeps its wet baseline.
/// Only the orographic term draws the advected reservoir down — the floor is
/// locally sourced — and a small evapotranspiration recharge keeps the front from
/// collapsing to zero.
pub fn rainout(moisture: f64, uplift: f64, lat_deg: f64) -> (f64, f64) {
    let frac = (0.18 + 1.8 * uplift).min(0.85);
    let supp = 1.0 - subsidence(lat_deg);
    let oro = moisture * frac;
    let precip = (oro.max(convective_floor(lat_deg)) * supp).min(1.0);
    let moisture_out = (moisture - 0.5 * oro * supp + 0.03).clamp(0.0, 1.0);
    (precip, moisture_out)
}

pub fn apply(grid: &mut CellGrid) {
    let w = grid.w;
    for gy in 0..w {
        let lat = latitude_deg(w, f64::from(gy) + 0.5);
        let dir = wind_dir(lat);
        let xs: Vec<i32> = if dir > 0 {
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
                let (p, m) = rainout(moisture, uplift, lat);
                c.precip = p;
                moisture = m;
            }
            prev_elev = c.elev_m;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pregen::{LAT_NORTH, LAT_SOUTH};

    /// **Defect (a) dies: no adjacent-row direction reversal at full strength.**
    /// Sample the profile at grid-row resolution across the whole span; wherever
    /// the sign flips between adjacent rows, the magnitude at both rows must be
    /// near zero. A continuous profile guarantees this — the old three-way bit
    /// violated it at 30° and 60° (a ±1 step), printing a straight seam.
    #[test]
    fn zonal_wind_never_reverses_without_passing_through_zero() {
        let rows = 200;
        let lat = |k: usize| LAT_SOUTH + (LAT_NORTH - LAT_SOUTH) * k as f64 / rows as f64;
        for k in 0..rows {
            let (a, b) = (zonal_wind(lat(k)), zonal_wind(lat(k + 1)));
            if a.signum() != b.signum() && a != 0.0 && b != 0.0 {
                assert!(
                    a.abs() < 0.05 && b.abs() < 0.05,
                    "sign reversal at full strength: lat {:.2}→{:.2}, wind {a:.3}→{b:.3}",
                    lat(k),
                    lat(k + 1)
                );
            }
        }
    }

    /// Band interiors keep the historical directions (so nothing downwind of the
    /// interiors moves): easterly < 30°, westerly 30–60°, easterly > 60°.
    #[test]
    fn zonal_wind_signs_match_the_historical_bands() {
        assert!(zonal_wind(15.0) < 0.0, "trades must be easterly");
        assert!(zonal_wind(45.0) > 0.0, "mid-latitudes must be westerly");
        assert!(zonal_wind(75.0) < 0.0, "polar cell must be easterly");
        // Trades are the strongest cell; polar easterlies the weakest.
        assert!(zonal_wind(15.0).abs() > zonal_wind(75.0).abs());
    }

    /// The calm belts sit at the boundaries: |wind| is small within a few degrees
    /// of 30° and 60°, and large in the band centres.
    #[test]
    fn calm_belts_bracket_the_band_boundaries() {
        assert!(
            zonal_wind(30.0).abs() < 0.05,
            "30° must be calm (horse latitudes)"
        );
        assert!(
            zonal_wind(60.0).abs() < 0.05,
            "60° must be calm (polar front)"
        );
        assert!(zonal_wind(15.0).abs() > 0.8, "15° must be strong trades");
    }

    /// **Subsidence peaks at 30°, spares the equator and the 60° front.** This is
    /// the desert-belt engine: strong suppression at the Hadley limb, ~none where
    /// air rises.
    #[test]
    fn subsidence_peaks_at_thirty_and_spares_equator_and_front() {
        let s30 = subsidence(30.0);
        assert!(
            s30 > 0.6,
            "30° must be strongly suppressed (Hadley limb): {s30}"
        );
        assert!(
            subsidence(8.0) < 0.1,
            "low latitudes (ITCZ side) must be wet"
        );
        assert!(
            subsidence(60.0) < 0.1,
            "60° polar front must be wet (storm track)"
        );
        // The subtropical limb dominates its neighbours.
        assert!(s30 > subsidence(15.0));
        assert!(s30 > subsidence(45.0));
        // A mild polar-desert hint at the northern edge, but far below 30°.
        assert!(subsidence(78.0) > 0.15 && subsidence(78.0) < s30);
    }

    /// **Defect (b)/(c) at the physics level: the shared rainout makes 30° dry
    /// and the equator wet on flat land** (no uplift). The band that reads arid
    /// under the pipeline's own 0.32 threshold is the 30° limb, not the equator.
    #[test]
    fn rainout_makes_the_thirty_belt_arid_and_the_equator_wet_on_flat_land() {
        // Converge a flat-land march to its steady precip at a latitude.
        let flat = |lat: f64| {
            let mut m = 1.0;
            let mut p = 0.0;
            for _ in 0..64 {
                let (pi, mi) = rainout(m, 0.0, lat);
                p = pi;
                m = mi;
            }
            p
        };
        let equator = flat(8.0);
        let desert = flat(30.0);
        let temperate = flat(50.0);
        assert!(
            equator > 0.32,
            "equatorial flat land must stay wet: {equator}"
        );
        assert!(
            desert < 0.32,
            "the 30° belt must be arid on flat land: {desert}"
        );
        assert!(
            desert < equator && desert < temperate,
            "30° must dip below both neighbours"
        );
    }
}
