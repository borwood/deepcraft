//! **Neighbour-relative census primitives** — the small pure functions every
//! grid-scale-structure measurement in this repo needs, in one place.
//!
//! They exist because of `corrections.md` **#61**: *a criterion over a global
//! aggregate cannot license a claim about local structure*. Relief, mean, min and
//! max are claims about **magnitude**; "the shape is preserved" is a claim about
//! **arrangement**, and only a neighbour-relative statistic can see it. Every
//! erosional slice since journal/0115 has to pair the two, so the pairing wants a
//! shared implementation rather than a fourth copy.
//!
//! **Why `src/` and not an example.** A Rust example is its own crate root, so a
//! sibling example cannot call this one's census — and journal/0115's finding was
//! that *three instruments agreed because they were the same instrument, copied*.
//! journal/0116 recorded the extraction as open and out of its scope; journal/0122
//! took it, because the fix slice needed the same four functions and copying them
//! would have been anti-shape **A-1** on the exact code path that taught us the
//! lesson.
//!
//! # The reference values, in closed form
//!
//! What makes [`acf4`] a *discriminator* rather than a number is that all three
//! reference points are derivable (journal/0116 § D1). For the concavity operator
//! `mean(8 neighbours) − self`:
//!
//! - over **white noise**: variance `1.125 σ²`, lag-1 covariance `−0.1875 σ²`, so
//!   `ACF(1) = −1/6 ≈ −0.167`;
//! - over a **perfect checkerboard**: the four diagonal neighbours carry the
//!   cell's own sign and the four cardinal ones the opposite, so `mean(8) = 0`,
//!   the concavity field *is* the checkerboard negated, `ACF(1) = −1`,
//!   `ACF(2) = +1`;
//! - over a **smooth** surface: `ACF(1) → +1`.
//!
//! So the question is never *"is the autocorrelation negative"* — an ordinary
//! noisy heightfield is already at −0.167. It is **how far past −1/6 it has
//! gone.**

/// The eight-neighbour offsets, in the same order the solve's own `NEIGH8` uses.
const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// The discrete Laplacian `mean(8 neighbours) − self` of a plane, over the whole
/// grid; `NaN` on the border ring where the stencil does not fit. The eight
/// neighbours are taken unconditionally — a land cell's concavity is measured
/// against its actual surroundings, sea or not.
///
/// Positive ⇒ the cell sits **below** its surroundings.
#[must_use]
pub fn laplacian8(plane: &[f64], w: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; w * w];
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let mut sum = 0.0;
            for (dx, dy) in NEIGH8 {
                sum += plane[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize];
            }
            out[i] = sum / 8.0 - plane[i];
        }
    }
    out
}

/// Pearson correlation of a list of pairs. `NaN` when either side is constant or
/// there are fewer than two pairs — a degenerate correlation is not zero, and
/// reporting it as zero is how a null gets mistaken for a measurement.
#[must_use]
pub fn pearson(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len() as f64;
    if n < 2.0 {
        return f64::NAN;
    }
    let (sa, sb) = pairs
        .iter()
        .fold((0.0, 0.0), |(sa, sb), (a, b)| (sa + a, sb + b));
    let (ma, mb) = (sa / n, sb / n);
    let (mut caa, mut cbb, mut cab) = (0.0f64, 0.0f64, 0.0f64);
    for &(a, b) in pairs {
        let (da, db) = (a - ma, b - mb);
        caa += da * da;
        cbb += db * db;
        cab += da * db;
    }
    if caa <= 0.0 || cbb <= 0.0 {
        return f64::NAN;
    }
    cab / (caa * cbb).sqrt()
}

/// Autocorrelation of a masked `w × w` field at lags 1..4 along one axis.
/// `along_x` selects +x, otherwise +y; the lag is bounded so a +x offset cannot
/// wrap onto the next row.
#[must_use]
pub fn acf4(field: &[f64], ok: &[bool], w: usize, along_x: bool) -> [f64; 4] {
    let mut out = [f64::NAN; 4];
    for (l, slot) in out.iter_mut().enumerate() {
        let lag = l + 1;
        let mut pairs: Vec<(f64, f64)> = Vec::new();
        for y in 0..w {
            for x in 0..w {
                let (x2, y2) = if along_x { (x + lag, y) } else { (x, y + lag) };
                if x2 >= w || y2 >= w {
                    continue;
                }
                let (i, j) = (y * w + x, y2 * w + x2);
                if ok[i] && ok[j] {
                    pairs.push((field[i], field[j]));
                }
            }
        }
        *slot = pearson(&pairs);
    }
    out
}

/// Fraction of adjacent valid pairs whose values have **strictly opposite sign**.
///
/// The closed-form companion to [`acf4`]: `P(opposite sign) = arccos(ρ)/π`, so
/// 0.55 at white noise, 1.00 at a perfect checkerboard, 0 at smooth.
#[must_use]
pub fn flip_rate(field: &[f64], ok: &[bool], w: usize, along_x: bool) -> f64 {
    let (mut hit, mut tot) = (0usize, 0usize);
    for y in 0..w {
        for x in 0..w {
            let (x2, y2) = if along_x { (x + 1, y) } else { (x, y + 1) };
            if x2 >= w || y2 >= w {
                continue;
            }
            let (i, j) = (y * w + x, y2 * w + x2);
            if ok[i] && ok[j] {
                tot += 1;
                if field[i] * field[j] < 0.0 {
                    hit += 1;
                }
            }
        }
    }
    hit as f64 / tot.max(1) as f64
}

/// Root-mean-square of a field over its valid mask.
#[must_use]
pub fn rms(field: &[f64], ok: &[bool]) -> f64 {
    let (mut s, mut n) = (0.0f64, 0usize);
    for (i, &v) in field.iter().enumerate() {
        if ok[i] {
            s += v * v;
            n += 1;
        }
    }
    if n == 0 {
        f64::NAN
    } else {
        (s / n as f64).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The closed-form references are what make the instrument a
    /// discriminator**, so they are asserted rather than trusted: a perfect
    /// checkerboard must read `ACF(1) = −1`, `ACF(2) = +1`, flip rate `1.0`.
    #[test]
    fn a_perfect_checkerboard_reads_at_its_closed_form_reference() {
        let w = 32;
        let plane: Vec<f64> = (0..w * w)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                if (x + y) % 2 == 0 { 1.0 } else { -1.0 }
            })
            .collect();
        let conc = laplacian8(&plane, w);
        let ok: Vec<bool> = conc.iter().map(|v| v.is_finite()).collect();
        let ax = acf4(&conc, &ok, w, true);
        let ay = acf4(&conc, &ok, w, false);
        assert!(
            (ax[0] + 1.0).abs() < 1e-9 && (ay[0] + 1.0).abs() < 1e-9,
            "checkerboard ACF(1) should be −1, got {ax:?} / {ay:?}"
        );
        assert!(
            (ax[1] - 1.0).abs() < 1e-9 && (ay[1] - 1.0).abs() < 1e-9,
            "checkerboard ACF(2) should be +1, got {ax:?} / {ay:?}"
        );
        assert!((flip_rate(&conc, &ok, w, true) - 1.0).abs() < 1e-12);
    }

    /// And a smooth plane must read at the other end, or "far past −1/6" has no
    /// upper reference to be far from.
    #[test]
    fn a_smooth_ramp_has_no_grid_scale_structure() {
        let w = 32;
        let plane: Vec<f64> = (0..w * w).map(|i| (i % w) as f64 * 3.0).collect();
        let conc = laplacian8(&plane, w);
        let ok: Vec<bool> = conc.iter().map(|v| v.is_finite()).collect();
        // A linear ramp has exactly zero Laplacian, so the concavity field is
        // constant and the correlation is degenerate — which the census reports
        // as NaN rather than as a number.
        assert!(rms(&conc, &ok) < 1e-12);
        assert!(acf4(&conc, &ok, w, true)[0].is_nan());
    }
}
