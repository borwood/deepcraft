//! **The MFD partition kernel** — the D8 steepest-descent receiver rule, the
//! multi-flow-direction weight partition and the two geometric constants
//! (`MFD_DIST`, `MFD_CONTOUR`) a multi-receiver partition needs, plus the
//! representational floor on a share.
//!
//! The exponent these weights are raised to is authored next door in `mfd_law`;
//! the phase that drives this kernel over the grid is `super::routing`.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**
//! The partition is erosion's own physics, not an engine primitive.

use super::mfd_law::MfdParams;
use super::{NEIGH8, coords_of, in_grid, is_border};

/// **The MFD partition** — the number of D8 directions, and the two geometric
/// constants a multi-receiver partition needs that a single-receiver one never
/// had to name.
///
/// `MFD_DIST` is the true flow-path length in cell widths (`1` cardinal, `√2`
/// diagonal): a steepest-*drop* rule can ignore it, a slope-weighted partition
/// cannot, or every diagonal is over-weighted by `√2` and the drainage net
/// acquires a systematic X-bias.
///
/// `MFD_CONTOUR` is Quinn (1991)'s **contour width** — the length of the cell
/// boundary the flow crosses, normalised to the cardinal case (`0.5Δ` cardinal,
/// `0.354Δ` diagonal ⇒ `1` and `1/√2`). It is the *width of the gate*, not the
/// steepness of the drop, and it is why a diagonal neighbour receives less than a
/// cardinal one at equal slope.
pub(super) const MFD_DIRS: usize = 8;
const SQRT2: f64 = std::f64::consts::SQRT_2;
pub(super) const MFD_DIST: [f64; MFD_DIRS] = [SQRT2, 1.0, SQRT2, 1.0, 1.0, SQRT2, 1.0, SQRT2];
const MFD_CONTOUR: [f64; MFD_DIRS] = [
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
];

/// **The representational floor on a partition share.** A neighbour allotted less
/// than this fraction of the cell's discharge is dropped and the survivors
/// renormalised.
///
/// It is not a physical parameter — it is what keeps the *record* honest and
/// affordable. Without it every land cell dumps an infinitesimal trickle into
/// every downslope neighbour, the flux record grows an entry for each, and the
/// archive pays megabytes to store noise that no consumer can distinguish from
/// zero. The steepest receiver always carries at least `1/8` of the discharge, so
/// a survivor always exists and no cell is ever turned into a sink by the floor.
///
/// **⚠ STUB #22 — a record-affordability constant that changes the physics.** The
/// floor is applied *here*, before renormalisation, so the surviving receivers are
/// handed the dropped share and the **solve** moves water it otherwise would not.
/// A requirement of the *record* has leaked into the *landscape*: by
/// ARCHITECTURE.md's test, if the record consumer vanished tomorrow this constant
/// would not exist in this shape. It is inert on the near-flat ground the slice is
/// for (shares near `1/6` there, far above the floor) and bites in the moderately
/// convergent regime, making the net slightly more channelised than the exponent
/// alone specifies — argued, **not measured**. Mass is unaffected: renormalisation
/// is exact and the residual rule makes the split exact.
/// **Heir:** whoever settles the record's size budget (flow.md § 9 item 7b, the
/// aggregation window; the marine-sink lever) — and they should decide whether the
/// solve may see this at all, the alternative being a floor applied only on the
/// way into the record. See `docs/design/stubs.md` § 22.
///
/// **2026-07-26 (hybrid `p`, journal/0113):** the floor became a *knob*
/// ([`MfdParams::min_weight`], [`DeepConfig::mfd_min_weight`]) rather than a
/// hard-coded constant, so its effect on the solve can be **measured end-to-end**
/// instead of argued. The default is unchanged and the stub is unchanged — a knob
/// is not an heir.
pub const MFD_MIN_WEIGHT: f64 = 0.01;

/// D8 steepest-descent receiver of cell `i` on the filled surface (`-1` = sink).
#[inline]
pub(super) fn route_cell(i: usize, w: usize, surf: &[f64], filled: &[f64], sea_level: f64) -> i32 {
    if surf[i] <= sea_level || is_border(i, w) {
        return -1;
    }
    let (gx, gy) = coords_of(i, w);
    let fi = filled[i];
    let mut best: Option<(f64, usize)> = None;
    for (dx, dy) in NEIGH8 {
        let Some(j) = in_grid(gx + dx, gy + dy, w) else {
            continue;
        };
        if filled[j] < fi && best.is_none_or(|(bf, _)| filled[j] < bf) {
            best = Some((filled[j], j));
        }
    }
    best.map_or(-1, |(_, j)| j as i32)
}

/// **The MFD partition of one cell's discharge** (flow.md § 2.6, § 2.4).
///
/// Writes the eight normalised out-weights of cell `i` into `w_out` (aligned to
/// [`NEIGH8`], and therefore to `FaceKey::lateral`), and returns the direction
/// carrying the largest share (`-1` when the cell is a sink and every weight is
/// zero).
///
/// **The field partitioned is the FREE-SURFACE POTENTIAL, and that is the point.**
/// `filled` is the priority-flood surface: bare ground where the land drains, and
/// the *spill-level water surface* inside every depression. For free-phase flow
/// that is `head = z_bed + depth` exactly — pressure head is zero at a free
/// surface, so a lake's potential is flat and its bed's elevation is irrelevant.
/// So the partition descends a potential rather than a topography, which is what
/// flow.md § 2.4 asks of the free regime. (`dc:field/head` is the **bound**
/// regime's potential — a water table that deliberately crosses surface divides.
/// Routing free surface water down it would make rivers cross divides too, which
/// § 2.4's own qualification forbids. Bound MFD is continuation (c)'s, not this
/// pass's.)
///
/// The weight is **Holmgren (1994)** with Quinn's contour width:
///
/// ```text
///   w_k  ∝  (Δh_k / d_k)^p · L_k          d_k = 1 or √2,  L_k = 1 or 1/√2
/// ```
///
/// `p` is the **convergence exponent**: `p = 1` is Quinn's maximally dispersive
/// form, large `p` is single-receiver steepest-slope. Nothing here is novel; what
/// it buys over D8 is that a cell's discharge can leave through more than one face
/// **within one epoch**, which is the entire difference between temporal
/// divergence (avulsion, which the record already had) and simultaneous divergence
/// (concurrent distributaries, which it structurally could not hold).
///
/// **`p` is SPATIALLY VARYING** (journal/0113). `area_i` is the cell's drainage
/// area from the *previous* epoch; the exponent is [`MfdParams::exponent_at`] of
/// the channelisation index `χ = area_i · S_max²`, so unchannelised ground
/// disperses and channelised ground stays in its banks. `MfdParams::uniform`
/// recovers journal/0109's single exponent with no ramp evaluated.
///
/// **The slopes are normalised by `S_max` before exponentiation.** Algebraically
/// that is a no-op — the renormalisation at the end divides it straight back out —
/// but it makes every base lie in `(0, 1]`, so a large `p_chan` can never underflow
/// a whole partition to zero and turn a draining cell into a sink. Without it the
/// safe exponent range is bounded by the world's smallest slope, which is a
/// coupling nobody would remember.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(super) fn partition_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    filled: &[f64],
    sea_level: f64,
    area_i: f64,
    cell_m: f64,
    mp: &MfdParams,
    w_out: &mut [f64],
) -> i32 {
    w_out.iter_mut().for_each(|v| *v = 0.0);
    if surf[i] <= sea_level || is_border(i, w) {
        return -1;
    }
    let (gx, gy) = coords_of(i, w);
    let fi = filled[i];
    // Pass 1: the downslope gradients, and the steepest of them — which is both
    // the normaliser and the `S` of the channelisation index. `steepest` is the
    // direction that takes the whole discharge when the cell is channelised.
    let mut slope = [0.0f64; MFD_DIRS];
    let mut s_max = 0.0f64;
    let mut steepest = -1i32;
    for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
        let Some(j) = in_grid(gx + dx, gy + dy, w) else {
            continue;
        };
        let drop = fi - filled[j];
        if drop <= 0.0 {
            continue;
        }
        let s = drop / MFD_DIST[d];
        slope[d] = s;
        if s > s_max {
            s_max = s;
            steepest = d as i32;
        }
    }
    if s_max <= 0.0 {
        return -1;
    }
    // The channelisation index. `s_max / cell_m` is the dimensionless gradient —
    // `slope` above is a rise per **cell width**, and leaving the cell size inside
    // `χ` would put a second, invisible resolution factor in a threshold that
    // already carries one through `A` (stub #26 owns what remains).
    let s_dim = s_max / cell_m;
    let chi = area_i * s_dim * s_dim;
    // **The channel switch.** Confined flow takes one path: the whole discharge
    // goes down the steepest slope, exactly. This is a *routing* statement, not a
    // large exponent — see `MfdParams::chi_hi` for why an exponent cannot do it.
    // The mass rules are trivially satisfied (one weighted direction, so it is
    // also the last, and it takes `q − 0`), and the traversal licence holds
    // because the chosen direction has `drop > 0` like every other weighted one.
    if mp.is_channel(chi) {
        w_out[steepest as usize] = 1.0;
        return steepest;
    }
    let p = mp.exponent_at(chi);
    // Integer exponents go through `powi` — the hybrid ramp is rounded so this is
    // the taken branch on every cell of a production run, where `powf` on
    // 8 directions × 297k cells × 200 epochs is half a billion transcendental
    // calls and would dominate the deep run. `powf` survives for the fractional
    // uniform exponents a probe may sweep.
    let pr = p.round();
    let int_p = (p == pr && (1.0..=64.0).contains(&pr)).then_some(pr as i32);
    // **Normalise by `S_max` — on the hybrid path only.** Algebraically the
    // division is a no-op (the renormalisation below divides it straight back
    // out), but it puts every base in `(0, 1]` so the ramp's large exponents
    // cannot underflow a whole partition to zero and report a draining cell as a
    // sink. It is *not* applied to a uniform law, and that is deliberate: `x/1.0`
    // is exact, so journal/0109's arithmetic survives **bit for bit** and the
    // uniform world stays a reachable cross-commit fixed point (the goldens in
    // `material_transport.rs` and `material_creep.rs` are pinned there). The cost
    // of that choice is that a *uniform* law keeps 0109's exponent-range limit —
    // see [`MfdParams::uniform`].
    let norm = if mp.is_uniform() { 1.0 } else { s_max };
    let mut sum = 0.0;
    for (d, &s) in slope.iter().enumerate() {
        if s <= 0.0 {
            continue;
        }
        let base = s / norm;
        let sp = match int_p {
            Some(k) => base.powi(k),
            None => base.powf(p),
        };
        let raw = sp * MFD_CONTOUR[d];
        w_out[d] = raw;
        sum += raw;
    }
    if sum <= 0.0 {
        return -1;
    }
    // Normalise, then apply the representational floor and renormalise over the
    // survivors. The steepest direction's base is exactly `1`, so its raw weight is
    // at least `1/√2` against a total of at most `8`; its share is therefore never
    // below `1/12`, the survivor set is never empty and `sum2 > 0` always.
    let mut sum2 = 0.0;
    for v in w_out.iter_mut() {
        let n = *v / sum;
        *v = if n >= mp.min_weight { n } else { 0.0 };
        sum2 += *v;
    }
    let mut best = -1i32;
    let mut best_w = 0.0;
    for (d, v) in w_out.iter_mut().enumerate() {
        if *v <= 0.0 {
            continue;
        }
        *v /= sum2;
        if *v > best_w {
            best_w = *v;
            best = d as i32;
        }
    }
    best
}

#[cfg(test)]
mod mfd_tests {
    use super::*;

    /// A 3×3 patch centred on cell 4, with the given drop (metres of filled
    /// potential) to each of the eight neighbours in [`NEIGH8`] order. Everything
    /// is far above the sea stand and the centre is interior.
    fn patch(drops: [f64; 8]) -> (usize, Vec<f64>, Vec<f64>) {
        let w = 3usize;
        let c = 4usize;
        let mut filled = vec![0.0; w * w];
        filled[c] = 100.0;
        for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
            let j = ((1 + dy) as usize) * w + ((1 + dx) as usize);
            filled[j] = 100.0 - drops[d];
        }
        let surf = filled.clone();
        (c, surf, filled)
    }

    /// The partition is a **probability**: one cell's shares sum to one. If they
    /// did not, discharge would be created or destroyed at every junction and the
    /// mass budget flow.md § 3 rests on would mean nothing.
    #[test]
    fn the_partition_weights_sum_to_one() {
        for p in [1.0f64, 1.1, 2.0, 4.0, 6.0] {
            let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.9, 0.25, 1.5, 4.0]);
            let mut w_out = [0.0f64; MFD_DIRS];
            let best = partition_cell(
                c,
                3,
                &surf,
                &filled,
                -1000.0,
                0.0,
                1.0,
                &MfdParams::uniform(p),
                &mut w_out,
            );
            assert!(
                best >= 0,
                "p={p}: a cell with downslope neighbours is a sink"
            );
            let sum: f64 = w_out.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-12,
                "p={p}: weights sum to {sum}, not 1"
            );
            assert!(w_out.iter().all(|&v| v >= 0.0));
        }
    }

    /// **`p → ∞` is a SINGLE-RECEIVER limit.** That property is what makes the
    /// exponent a *convergence knob* rather than a different model: the partition
    /// contains the thing it replaces as a limit, so "how much does MFD change the
    /// world" has a continuous answer instead of a discrete one.
    ///
    /// > **Not, however, `route_cell` — corrections #58.** journal/0109 and
    /// > flow.md § 2.6.1 both say *"`p → ∞` is single-receiver D8 **exactly**"*, and
    /// > it is not: the partition's limit is the steepest **slope** and `route_cell`
    /// > takes the steepest **drop**, which differ on diagonals by the very `√2`
    /// > path length that slice introduced. `the_partition_follows_slope_not_drop`
    /// > below pins a case where the two pick *different* receivers. The limit is
    /// > the physically correct one; it is the claim of identity that is wrong.
    ///
    /// Note what "large" has to mean, because it is the honest reading of the knob:
    /// `p` acts on the **ratio** of slopes, so two neighbours within a few percent
    /// of each other still share at `p = 16`. Collapse is a limit, not a threshold
    /// — which is precisely why `p = 4` leaves gorges convergent (their sidewalls
    /// are nowhere near the channel's slope) and delta tops divergent (theirs are).
    #[test]
    fn a_large_exponent_collapses_the_partition_onto_one_receiver() {
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.1, 0.25, 1.5, 2.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(16.0),
            &mut w_out,
        );
        assert!(best >= 0);
        assert_eq!(
            w_out.iter().filter(|&&v| v > 0.0).count(),
            1,
            "weights {w_out:?} did not collapse onto one receiver"
        );
        assert!((w_out[best as usize] - 1.0).abs() < 1e-12);
        assert_eq!(best, 3);
    }

    /// **The partition follows SLOPE, not drop.** Index 7 is a diagonal with a 4 m
    /// drop over `√2` cells (slope 2.83); index 3 a cardinal with 3 m over one cell
    /// (slope 3.0). The cardinal must take the larger share — a steepest-*drop*
    /// rule, which is what `route_cell` uses, picks the diagonal instead. Without
    /// the true flow-path length every diagonal is over-weighted by `√2` and the
    /// drainage net acquires a systematic X-bias.
    #[test]
    fn the_partition_follows_slope_not_drop() {
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.1, 0.25, 1.5, 4.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
        assert_eq!(best, 3, "the diagonal's √2 path length was not applied");
        assert!(w_out[3] > w_out[7]);
        // The steepest-DROP rule would have said otherwise — pinned, so the
        // difference between the two routings is a fact this file states rather
        // than a claim the prose makes.
        assert_eq!(route_cell(c, 3, &surf, &filled, -1000.0), 8);
    }

    /// **`p = 1` is maximally dispersive** (Quinn) and must genuinely spread:
    /// every downslope neighbour above the representational floor keeps a share.
    /// With equal drops all round, the cardinals win twice over — a shorter path
    /// (steeper slope) *and* a wider contour — so their share is exactly `2×` a
    /// diagonal's. That is the two `√2`s the partition carries, isolated.
    #[test]
    fn a_unit_exponent_spreads_across_every_downslope_neighbour() {
        let (c, surf, filled) = patch([1.0; 8]);
        let mut w_out = [0.0f64; MFD_DIRS];
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(1.0),
            &mut w_out,
        );
        assert_eq!(w_out.iter().filter(|&&v| v > 0.0).count(), 8);
        let ratio = w_out[1] / w_out[0];
        assert!(
            (ratio - 2.0).abs() < 1e-9,
            "cardinal/diagonal share ratio is {ratio}, expected 2 (√2 slope × √2 contour)"
        );
    }

    /// A cell with no downslope neighbour is a **sink** and every weight is zero —
    /// the record then reads it as a boundary-face exit, the same convention the
    /// single-receiver path used.
    #[test]
    fn a_cell_with_no_lower_neighbour_is_a_sink() {
        let (c, surf, filled) = patch([-1.0; 8]);
        let mut w_out = [1.0f64; MFD_DIRS];
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
        assert_eq!(best, -1);
        assert!(w_out.iter().all(|&v| v == 0.0));
    }

    /// The **representational floor** drops a share below [`MFD_MIN_WEIGHT`] and
    /// renormalises the survivors, so the sum is still exactly one. Without the
    /// renormalisation the floor would quietly delete discharge — a leak that would
    /// show up in the mass budget as a mystery rather than as a rule.
    #[test]
    fn the_weight_floor_renormalises_rather_than_deleting_discharge() {
        // One dominant cardinal and one very gentle one: at p = 4 the gentle
        // neighbour's share falls under a percent and is dropped.
        let (c, surf, filled) = patch([-1.0, -1.0, -1.0, 10.0, 0.3, -1.0, -1.0, -1.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
        let sum: f64 = w_out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "sum {sum} after the floor");
        assert!(
            w_out.iter().all(|&v| v == 0.0 || v >= MFD_MIN_WEIGHT),
            "a sub-floor weight survived: {w_out:?}"
        );
    }

    // ---- hybrid `p` (journal/0113) ----------------------------------------

    /// **The ramp is monotone non-decreasing in the channelisation index**, and
    /// hits its endpoints. This is the invariant that would catch a law wired
    /// backwards — dispersing channels and concentrating hillslopes — which no
    /// absolute count of anything could see, because both directions produce
    /// plausible-looking numbers.
    #[test]
    fn the_exponent_ramps_monotonically_from_hillslope_to_channel() {
        let mp = MfdParams::default();
        assert_eq!(mp.exponent_at(0.0), mp.p_hill);
        assert_eq!(mp.exponent_at(mp.chi_lo), mp.p_hill);
        assert_eq!(mp.exponent_at(mp.chi_hi), mp.p_chan);
        assert_eq!(mp.exponent_at(1.0e9), mp.p_chan);
        let mut prev = mp.exponent_at(0.0);
        for k in 0..400 {
            // Sweep χ across six decades either side of the ramp.
            let chi = 10f64.powf(-7.0 + 6.0 * f64::from(k) / 400.0);
            let p = mp.exponent_at(chi);
            assert!(p >= prev, "exponent fell from {prev} to {p} at chi={chi}");
            assert!((mp.p_hill..=mp.p_chan).contains(&p));
            prev = p;
        }
    }

    /// **Uniform mode is the ramp's degenerate case and evaluates no ramp at all**
    /// — `p_chan == p_hill` must return that exponent for every `χ`, including
    /// fractional exponents a probe may sweep (which must NOT be rounded).
    #[test]
    fn a_uniform_law_ignores_the_channelisation_index() {
        for p in [1.0f64, 1.5, 4.0, 7.25] {
            let mp = MfdParams::uniform(p);
            for chi in [0.0, 1e-9, 1e-3, 1.0, 1e6] {
                assert_eq!(mp.exponent_at(chi), p, "uniform p={p} moved at chi={chi}");
            }
        }
    }

    /// **The same cell, routed two ways by its drainage area alone.** One patch,
    /// one slope field — a moderately convergent junction with a clear steepest
    /// line and two near-rivals. Given a hillslope's area it must spread; given a
    /// trunk river's area it must not. That *is* the slice, isolated from the
    /// world: the partition now depends on how channelised the flow is, and on
    /// nothing else that changed.
    #[test]
    fn the_same_slope_field_disperses_on_a_hillslope_and_concentrates_in_a_channel() {
        let mp = MfdParams::default();
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 2.6, 2.2, 0.25, 1.5, 1.0]);
        // `cell_m = 100` makes the steepest gradient `2.6/100 = 0.026`, a
        // production-like value; `χ = A · 0.026²`, so `A = 1` is a hillslope
        // (χ = 6.8e-4, below `chi_lo`) and `A = 4000` is a channel (χ = 2.7,
        // above `chi_hi`). One slope field, two drainage areas.
        let mut hill = [0.0f64; MFD_DIRS];
        let mut chan = [0.0f64; MFD_DIRS];
        partition_cell(c, 3, &surf, &filled, -1000.0, 1.0, 100.0, &mp, &mut hill);
        partition_cell(c, 3, &surf, &filled, -1000.0, 4000.0, 100.0, &mp, &mut chan);
        let n_hill = hill.iter().filter(|&&v| v > 0.0).count();
        let n_chan = chan.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(n_chan, 1, "a channelised cell must route single-receiver");
        assert!(
            n_hill > n_chan,
            "hillslope kept {n_hill} receivers, channel kept {n_chan} — the law did not vary"
        );
        let top_hill = hill.iter().cloned().fold(0.0f64, f64::max);
        let top_chan = chan.iter().cloned().fold(0.0f64, f64::max);
        assert!(
            top_chan > top_hill,
            "the channel's steepest share ({top_chan}) is no larger than the hillslope's \
             ({top_hill})"
        );
        // Both are still probabilities — the ramp may not leak discharge.
        for w in [&hill, &chan] {
            let sum: f64 = w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
        }
    }

    /// **A large channel exponent must never turn a draining cell into a sink.**
    /// This is what the `S/S_max` normalisation buys: without it the raw weights
    /// are `S^p`, and on the gentle gradients this world actually has
    /// (`S ~ 1e-5` per cell width in places) a `p` of 64 underflows *every*
    /// direction to zero, the partition sums to zero, and the cell is reported as
    /// a sink — silently disconnecting a drainage network. Scale-free: it is a
    /// statement about floating-point range, not about grid size.
    #[test]
    fn a_steep_exponent_on_a_gentle_slope_still_finds_a_receiver() {
        let (c, surf, filled) = patch([1e-5, 2e-5, 5e-6, 3e-5, 9e-6, 2.5e-6, 1.5e-5, 4e-5]);
        // A **hybrid** law: the normalisation is deliberately not applied to a
        // uniform one (see `MfdParams::uniform`), so this pins the guarantee where
        // it exists. `chi` is far below `chi_lo`, so the exponent is `p_hill`.
        let mp = MfdParams {
            p_hill: 63.0,
            p_chan: 64.0,
            ..MfdParams::default()
        };
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(c, 3, &surf, &filled, -1000.0, 0.0, 1.0, &mp, &mut w_out);
        assert!(
            best >= 0,
            "a cell with eight downslope neighbours became a sink"
        );
        let sum: f64 = w_out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
    }

    /// **The floor is a knob now, and `0.0` disables it** — the measurement stub #22
    /// was owed. With the floor off, every downslope neighbour keeps its share no
    /// matter how small; with it on, the sub-floor ones are dropped and the rest
    /// renormalised. Both still sum to one.
    #[test]
    fn a_zero_floor_keeps_every_downslope_neighbour() {
        let (c, surf, filled) = patch([-1.0, -1.0, -1.0, 10.0, 0.3, -1.0, -1.0, -1.0]);
        let mut floored = [0.0f64; MFD_DIRS];
        let mut open = [0.0f64; MFD_DIRS];
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut floored,
        );
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams {
                min_weight: 0.0,
                ..MfdParams::uniform(4.0)
            },
            &mut open,
        );
        assert_eq!(floored.iter().filter(|&&v| v > 0.0).count(), 1);
        assert_eq!(open.iter().filter(|&&v| v > 0.0).count(), 2);
        for w in [&floored, &open] {
            let sum: f64 = w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
        }
    }
}
