//! **The stop-motion rate**: how many poses a cycle gets, and how long each one
//! is held (`docs/design/bodies.md` § *Stepped animation*, the `▶▶ DECIDED
//! 2026-08-04` banner).
//!
//! # What this replaces, and why it is not the same shape
//!
//! `body::ANIM_FPS = 12.0` was a **world-global absolute constant**: every body,
//! every clip, every cadence sampled on one grid anchored at absolute `t = 0`.
//! That was harmless while there was one authored clip at one cadence and
//! "frames per cycle" was a number nobody had to think about. Derived gait made
//! cadence a per-body function of the body's own leg length, so a fixed rate
//! started **aliasing a derived cadence**: at 4.5 m/s the shipped biped got 6.98
//! frames per cycle, the longleg 7.74, the stout **4.29** — low *and* fractional,
//! so its samples drifted through the cycle and beat against it. The user saw the
//! hitch at the 0148 walk and diagnosed it unprompted. **Anti-shape A-1, fourth
//! retirement in this arc.**
//!
//! The replacement quantizes **per cycle**, not per second:
//!
//! ```text
//! N = round(cycle_duration × target_fps),  clamped to N ≥ 2
//! ```
//!
//! **The point is that `N` is an INTEGER, not that it is larger.** An integer `N`
//! lands on the same phases every cycle; a fractional one drifts through the
//! cycle and beats against it. It is also why the biped always read stable — it
//! was already within 0.03 of an integer.
//!
//! **The floor of 2 is the user's ruling** (*"there can't be fewer than 2 frames
//! per cycle"*) and it is doing real work: bare `round()` returns **1** for a
//! cycle near `1/fps` and **0** below `1/(2·fps)`. It converts the Nyquist edge
//! into a defined outcome instead of a degenerate one.
//!
//! # The target is a CLIENT setting
//!
//! Framerate is **approach**, never target (`bodies.md` § *THE SIM OWNS THE
//! TARGET*, third instance). [`AnimRate`] is a client performance/visual
//! setting — **not pack content and not sim state**. Two clients at different
//! targets must render differently and resolve nothing differently; see
//! `body`'s `the_target_fps_never_reaches_a_sim_visible_quantity`.
//!
//! **Deferred, ruled but not built here:** the pack may declare a per-body
//! **max** target fps (*"preserves toy-like or mechanical aesthetics for a robot
//! in a smooth world"*) — a cap, not a value, so the client's setting still
//! governs downward. It wants a `BodyPlan` field, so it rides the B3 wire
//! window; `stubs.md` § BODIES carries it.
//!
//! # A one-off is just a cycle that does not repeat
//!
//! (User.) If its duration is known, identical formula — there is no
//! cycle-vs-one-off branch anywhere in this module, only [`AnimRate::cycle`]
//! answering `None` when the duration is not known.

/// The default stop-motion target, poses per second.
///
/// **A default for a setting, not a new world-global constant** — the shape
/// [`AnimRate`] exists to retire. It is `12.0` because that reproduces the
/// shipped biped's look: at its 1.72 Hz cadence the old fixed grid gave 6.98
/// poses per cycle and this one gives **7**, a difference of one rounding.
/// Every other body changes, which is the point.
pub const DEFAULT_TARGET_FPS: f64 = 12.0;

/// The floor on poses per cycle — **the user's ruling**, 2026-08-04.
///
/// Not a tuning number: it is the Nyquist edge given a defined outcome. Below
/// `1/(2·fps)` of cycle duration, `round()` returns 0 and the grid degenerates
/// entirely; at `1/fps` it returns 1 and a cycle renders as a single held pose.
pub const MIN_POSES_PER_CYCLE: u32 = 2;

/// The client's stop-motion target. Constructed once at launch and read; it
/// holds no state and quantizes nothing by itself.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AnimRate {
    target_fps: f64,
}

impl Default for AnimRate {
    fn default() -> Self {
        Self {
            target_fps: DEFAULT_TARGET_FPS,
        }
    }
}

impl AnimRate {
    /// A target of `target_fps` poses per second, or `None` if that is not a
    /// finite positive number.
    ///
    /// **Refused rather than clamped**, and deliberately unbounded above and
    /// below: a bound here would be exactly the invented absolute constant this
    /// module retires. A silly target is not ill-defined — a very low one is
    /// held by [`MIN_POSES_PER_CYCLE`], and a very high one converges
    /// continuously on unquantized motion.
    #[must_use]
    pub fn new(target_fps: f64) -> Option<Self> {
        (target_fps.is_finite() && target_fps > 0.0).then_some(Self { target_fps })
    }

    /// Poses per second.
    #[must_use]
    pub fn target_fps(self) -> f64 {
        self.target_fps
    }

    /// The time step of a cycle whose duration is **not known** — `1/fps`, the
    /// grid this module replaces. The one place the old global behaviour
    /// survives, and it survives only where there is no cycle to divide.
    #[must_use]
    pub fn step_s(self) -> f64 {
        1.0 / self.target_fps
    }

    /// `N = round(cycle_duration × target_fps)`, clamped to
    /// [`MIN_POSES_PER_CYCLE`] — **the ruling, in one line**.
    ///
    /// `None` when the duration is not usable: not finite, not positive, or so
    /// long that `N` leaves `u32`. The last case is not an error — it is the
    /// **limit**. As a cycle lengthens its `N` grows and its phase grid gets
    /// finer (its *time* step stays `≈ 1/fps`), so an unusably long cycle is one
    /// whose quantization has continuously vanished. A body slowing to a stop
    /// walks that limit: cadence → 0, `N` → ∞, and the grid dissolves rather
    /// than snapping.
    #[must_use]
    pub fn cycle(self, cycle_duration_s: f64) -> Option<CycleGrid> {
        if !(cycle_duration_s.is_finite() && cycle_duration_s > 0.0) {
            return None;
        }
        let n = (cycle_duration_s * self.target_fps).round();
        if !n.is_finite() || n > f64::from(u32::MAX) {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "bounded above by the u32::MAX test and below by the max(); n is a rounded non-negative f64"
        )]
        let poses = (n.max(0.0) as u32).max(MIN_POSES_PER_CYCLE);
        Some(CycleGrid {
            poses,
            cycle_s: cycle_duration_s,
        })
    }
}

/// One animation's own grid: `N` poses spread over a cycle of known duration.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CycleGrid {
    poses: u32,
    cycle_s: f64,
}

impl CycleGrid {
    /// `N` — poses per cycle. An integer by construction; that is the fix.
    ///
    /// `#[cfg(test)]`: production consumes the grid, never the count — it asks
    /// for a [`CycleGrid::step_s`] to blend or a [`PhaseGrid`] to quantize on.
    /// `N` is the **reported** quantity: what the probe prints and what the
    /// acceptance tests assert. Gated rather than left `pub` so the split reads
    /// as deliberate instead of as an accessor nobody happened to call.
    #[cfg(test)]
    #[must_use]
    pub fn poses(self) -> u32 {
        self.poses
    }

    /// `Δt = duration / N` — **the quantity the per-bone blend averages**
    /// (user's mechanism 4, implementation note). Frames-per-cycle is not
    /// commensurable across different cycle durations; a time step is.
    #[must_use]
    pub fn step_s(self) -> f64 {
        self.cycle_s / f64::from(self.poses)
    }

    /// This anim's own phase grid — exactly `N` buckets, hit exactly.
    #[must_use]
    pub fn phases(self) -> PhaseGrid {
        PhaseGrid {
            buckets: f64::from(self.poses),
        }
    }

    /// The phase grid a bone whose blended time step is `dt_s` sees on **this**
    /// cycle. Bucket count `duration / Δt`, which is fractional whenever the
    /// blend moved it — that is the accepted cost of the blend, and it is small
    /// because every `Δt` in the blend derives from the same target and differs
    /// only by rounding.
    ///
    /// Returns this grid's own [`CycleGrid::phases`] when `dt_s` is this grid's
    /// own step, so a **single owner is a single owner**: the integer path is
    /// taken exactly, with no float round-trip through `duration / (duration/N)`.
    #[must_use]
    pub fn phases_at_step(self, dt_s: f64) -> PhaseGrid {
        if dt_s == self.step_s() {
            return self.phases();
        }
        PhaseGrid {
            buckets: self.cycle_s / dt_s,
        }
    }
}

/// A phase grid: how many held poses one cycle is divided into. Real-valued,
/// because a blended bone's bucket count need not be a whole number — an anim's
/// own grid always is ([`CycleGrid::phases`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PhaseGrid {
    buckets: f64,
}

impl PhaseGrid {
    /// Poses per cycle on this grid — real-valued, so this is the number that
    /// says whether the blend moved a bone off its anim's whole `N`.
    /// `#[cfg(test)]` for the same reason as [`CycleGrid::poses`]: reported, not
    /// consumed.
    #[cfg(test)]
    #[must_use]
    pub fn buckets(self) -> f64 {
        self.buckets
    }

    /// Hold `phase` (a cycle fraction) at the start of its bucket.
    ///
    /// **Phase in, phase out — the whole reason this is not the old
    /// `quantize_time`.** The old one floored a *time* onto a grid anchored at
    /// absolute `t = 0` and wrapped afterwards, so a looping clip whose duration
    /// was not a whole number of frames sampled a different set of sub-frame
    /// phases every cycle (ROADMAP § Observed). Quantizing the phase cannot do
    /// that: the grid is anchored to the **cycle**, so cycle *k* samples exactly
    /// what cycle 0 sampled.
    #[must_use]
    pub fn quantize(self, phase: f64) -> f64 {
        if !phase.is_finite() || !(self.buckets.is_finite() && self.buckets > 0.0) {
            return phase;
        }
        let p = phase.clamp(0.0, 1.0);
        (p * self.buckets).floor() / self.buckets
    }
}

/// **The per-bone rate**: the time step of a bone owned by several anims, each
/// with its own cycle and its own `N`.
///
/// `Σ w·Δt / Σ w` — *"a weighted blend is a weighted blend, a single owner is a
/// single owner"* (user, mechanism 4). It is **interpolation, not arbitration**:
/// there is no precedence rule to get wrong, and the two-blended-cycles problem
/// dissolves instead of being adjudicated.
///
/// **An accumulator rather than a function over a slice, and that is a
/// perf decision with a receipt.** The renderer blends one of these *per bone
/// per body per frame*; collecting each bone's owners into a `Vec` first cost a
/// heap allocation per bone and measured **+1.4 µs per body per frame** against
/// a 3.3 µs baseline. Runtime is sacred (CLAUDE.md § Conventions), so ownership
/// streams in and nothing is collected.
///
/// **Where ownership comes from today:** the composition in `body::pose_for` is
/// additive with no blend weights anywhere, so each of a bone's `k` owners has
/// proportion `1/k` and this is a mean. The `ownership` argument is the seam for
/// the day layers carry real weights (crossfades, action layers, B3's blend
/// tree); the identity it must preserve is that any single owner reduces to its
/// own `Δt`, asserted in this module's tests.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct StepBlend {
    acc: f64,
    weight: f64,
}

impl StepBlend {
    /// Fold one owner in. A non-finite or non-positive step or ownership is
    /// **ignored**, not propagated: an owner that cannot state a rate has no
    /// opinion about this bone, which is different from wanting rate zero.
    pub fn add(&mut self, step_s: f64, ownership: f64) {
        if !(step_s.is_finite() && step_s > 0.0) || !(ownership.is_finite() && ownership > 0.0) {
            return;
        }
        self.acc += step_s * ownership;
        self.weight += ownership;
    }

    /// The blended step, or `None` when nothing owns this bone — which is a
    /// different statement from "rate zero", and the caller has no rate to
    /// impose.
    #[must_use]
    pub fn step_s(self) -> Option<f64> {
        (self.weight > 0.0).then(|| self.acc / self.weight)
    }
}

/// [`StepBlend`] over a ready-made owner list — the shape the tests read in.
/// `#[cfg(test)]` because production streams (see the accumulator's note); one
/// authority either way, since this is written in terms of it.
#[cfg(test)]
#[must_use]
pub fn blended_step_s(owners: &[(f64, f64)]) -> Option<f64> {
    let mut blend = StepBlend::default();
    for &(step_s, ownership) in owners {
        blend.add(step_s, ownership);
    }
    blend.step_s()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_is_an_integer_and_never_below_two() {
        let rate = AnimRate::default();
        // A cycle exactly at the target: 12 poses.
        assert_eq!(rate.cycle(1.0).expect("1 s cycle").poses(), 12);
        // The user's floor, doing the work bare round() cannot: a cycle at
        // 1/fps rounds to 1, and one below 1/(2·fps) rounds to 0.
        assert_eq!(rate.cycle(1.0 / 12.0).expect("one frame").poses(), 2);
        assert_eq!(rate.cycle(1.0 / 30.0).expect("half a frame").poses(), 2);
        assert_eq!(rate.cycle(1e-9).expect("absurdly fast").poses(), 2);
        // …and it is a clamp, not a floor on the arithmetic: anything with room
        // above the floor keeps its rounded value.
        assert_eq!(rate.cycle(0.357).expect("the stout").poses(), 4);
    }

    #[test]
    fn an_unusable_duration_has_no_grid() {
        let rate = AnimRate::default();
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(rate.cycle(bad).is_none(), "{bad} must not yield a grid");
        }
        // The limit case: a cycle so long its N leaves u32 has, by then, a phase
        // grid finer than any float can express as motion.
        assert!(rate.cycle(1e12).is_none());
    }

    #[test]
    fn the_step_is_the_cycle_divided_by_n() {
        let rate = AnimRate::new(12.0).expect("a legal target");
        let g = rate.cycle(0.581).expect("the biped's cycle");
        assert_eq!(g.poses(), 7);
        assert!((g.step_s() - 0.581 / 7.0).abs() < 1e-15);
        // Every Δt is ≈ 1/fps and differs only by rounding — the reason
        // per-bone rates cluster tightly around the target however many layers
        // exist (user's mechanism 4, observation).
        assert!((g.step_s() - rate.step_s()).abs() < rate.step_s() * 0.5);
    }

    #[test]
    fn a_rate_must_be_finite_and_positive() {
        for bad in [0.0, -12.0, f64::NAN, f64::INFINITY] {
            assert!(AnimRate::new(bad).is_none(), "{bad} is not a target");
        }
        assert_eq!(
            AnimRate::new(DEFAULT_TARGET_FPS),
            Some(AnimRate::default()),
            "the default must be reachable through the constructor"
        );
    }

    #[test]
    fn a_phase_grid_holds_a_phase_at_its_bucket_start() {
        let g = AnimRate::default().cycle(1.0).expect("1 s").phases();
        assert_eq!(g.buckets(), 12.0);
        assert!((g.quantize(0.0) - 0.0).abs() < 1e-15);
        assert!((g.quantize(0.08) - 0.0).abs() < 1e-15);
        assert!((g.quantize(0.09) - 1.0 / 12.0).abs() < 1e-15);
        assert!((g.quantize(0.999) - 11.0 / 12.0).abs() < 1e-15);
        // Exactly N distinct values over a cycle, which is the ruling.
        let seen: std::collections::BTreeSet<u64> = (0..10_000)
            .map(|k| g.quantize(f64::from(k) / 10_000.0).to_bits())
            .collect();
        assert_eq!(seen.len(), 12);
    }

    /// **A cycle grid is anchored to the CYCLE, so cycle *k* samples what cycle
    /// 0 sampled — bit for bit.** This is the property the retired
    /// `quantize_time` could not have: it floored a time onto a grid anchored at
    /// absolute `t = 0`, so a clip whose duration was not a whole number of
    /// frames drifted through its own loop (ROADMAP § Observed, and the reason
    /// `looping_wraps_to_the_same_phase` had to be loosened to a tolerance in
    /// 2026-08-01).
    #[test]
    fn the_grid_does_not_drift_across_cycles() {
        // 0.7 s: the pack-author duration the Observed entry names as the one
        // that triggers the old defect immediately (8.4 frames at 12 fps).
        let g = AnimRate::default().cycle(0.7).expect("0.7 s").phases();
        for k in 0..64 {
            let t = 0.31 + 0.7 * f64::from(k);
            let phase = (t / 0.7).rem_euclid(1.0);
            assert_eq!(
                g.quantize(phase).to_bits(),
                g.quantize((0.31_f64 / 0.7).rem_euclid(1.0)).to_bits(),
                "cycle {k} sampled a different phase than cycle 0"
            );
        }
    }

    #[test]
    fn a_single_owner_is_a_single_owner() {
        let dt = 0.0831;
        assert_eq!(blended_step_s(&[(dt, 1.0)]), Some(dt));
        // Ownership 1.0 against a co-owner at 0.0 is still a single owner.
        assert_eq!(blended_step_s(&[(dt, 1.0), (0.05, 0.0)]), Some(dt));
        // …and any positive weight alone reduces, because the blend normalises.
        assert_eq!(blended_step_s(&[(dt, 0.31)]), Some(dt));
        assert_eq!(blended_step_s(&[]), None);
        assert_eq!(blended_step_s(&[(dt, 0.0)]), None);
        assert_eq!(blended_step_s(&[(f64::NAN, 1.0)]), None);
    }

    #[test]
    fn a_three_way_blend_is_the_weighted_mean() {
        let steps = [(0.08, 1.0), (0.09, 1.0), (0.10, 1.0)];
        let blended = blended_step_s(&steps).expect("three owners");
        assert!((blended - 0.09).abs() < 1e-15);
        // Equal proportions is the shipped case (the composition carries no
        // blend weights), so a mean and a proportional blend agree.
        let proportional =
            blended_step_s(&[(0.08, 1.0 / 3.0), (0.09, 1.0 / 3.0), (0.10, 1.0 / 3.0)])
                .expect("three owners");
        assert!((blended - proportional).abs() < 1e-15);
    }

    /// The blended grid is fractional, and the exact path is preserved for the
    /// bone nobody shares — the property that keeps a bearing chain on a whole
    /// `N` while an arm rides a compromise.
    #[test]
    fn a_shared_bone_gets_a_fractional_grid_and_an_owned_bone_does_not() {
        let rate = AnimRate::default();
        let gait = rate.cycle(0.581).expect("the biped's cycle"); // N = 7
        let clip = rate.cycle(2.0).expect("the idle clip"); // N = 24
        let alone = blended_step_s(&[(gait.step_s(), 1.0)]).expect("one owner");
        assert_eq!(
            gait.phases_at_step(alone),
            gait.phases(),
            "a bone nobody shares must sit on its own whole N"
        );
        assert_eq!(gait.phases_at_step(alone).buckets(), 7.0);
        let shared =
            blended_step_s(&[(gait.step_s(), 1.0), (clip.step_s(), 1.0)]).expect("two owners");
        let g = gait.phases_at_step(shared);
        assert!(
            (g.buckets() - 7.0).abs() > 1e-9 && (g.buckets() - 7.0).abs() < 1.0,
            "a shared bone's bucket count moves off the integer, but barely: {}",
            g.buckets()
        );
    }
}
