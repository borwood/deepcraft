//! **SCHEDULE — whether a pass runs before the loop, inside it, or both**
//! (`ARCHITECTURE.md` § *Schedule — DECIDED 2026-07-29 (user): seed is an initial
//! condition, and epoch 0 fires for everyone*).
//!
//! [`super::cadence`] (RATE) answers *how often, and for how much world time*.
//! This axis answers the question underneath it: **does this pass step at all,
//! and does anything establish its state before the loop opens?** They are
//! separate because a cadence cannot express *run-once-only* and a "seeded" flag
//! beside a cadence cannot express *seed-then-step* — the elimination that
//! produced the sum type is recorded in `ARCHITECTURE.md`.
//!
//! ```text
//! Seed              pre-loop once, never in-loop.  Integrates ZERO time.
//! Step(Cadence)     in-loop only, at that cadence.
//! SeedAndStep(Cadence)   both.
//! ```
//!
//! ## A seed is an INITIAL CONDITION, and that is a testable claim
//!
//! A seed **establishes t=0 state and integrates zero time**. The runner enforces
//! the second half rather than asking for it: a seeding body is handed
//! [`DeepStepCtx::dt`](super::runner::DeepStepCtx::dt) `= 0.0`, so every
//! rate-shaped transformation in it multiplies by zero. A pass whose "seed" is
//! really its step body run early cannot survive that, which is exactly the
//! discrimination the audit below had to make by hand.
//!
//! ## Epoch 0 fires for everyone, and the old skip rule is DELETED
//!
//! The rule this replaced lived in the runner: *a coarse-rate pass does not fire
//! at epoch 0*. It was written for the three passes seeded before the loop, and
//! then silently inherited by any pass a **world** authored a coarse period onto
//! ([`super::cadence::CadenceTable`]) — a semantics nobody decided.
//!
//! **It was also, on its own terms, a clock desync.** A firing at epoch `e` opens
//! a phase covering `[e, e + period)`; the phases tile the epoch axis **from
//! zero**. Skipping the firing at 0 does not shift the tiling, it *deletes the
//! first tile*, so a period-20 pass in a 200-epoch world integrated 180 epochs of
//! world time while its period-1 neighbours integrated 200. The pass experienced
//! less world time than the world. [`Schedule::integrated_dt`] is that quantity as
//! a number, and `tests/schedule_axis.rs` asserts it equals the run length for
//! every pass at every period — the invariant that only became assertable once E3
//! made `dt` real (journal/0123).
//!
//! ## The audit that came with the ruling (journal/0124)
//!
//! Three passes were seeded before the loop — `dc:deep/climate`, `dc:deep/geotherm`,
//! `dc:deep/head`. **All three turned out to be `Step`**, and all three pre-loop
//! blocks were deleted, because a seed is only an initial condition if something
//! **observes** it before the pass itself first steps — and under "epoch 0 fires
//! for everyone" every one of them first steps inside epoch 0, ahead of its own
//! only reader. The seeds were not initial conditions; they were **repair for the
//! skip**, and they died with it. The per-pass reasoning is in journal/0124.
//!
//! So [`Schedule::Seed`] and [`Schedule::SeedAndStep`] ship with **no production
//! declarer** — recorded in `docs/spines.md` § 3. They are executed and tested
//! (`runner.rs::the_runner_seeds_before_it_steps_and_a_seed_integrates_no_time`), not
//! decorative, and the honest reading is
//! that the deep-time roster's genuine setup work — the biotic layer's identity
//! planes, the tectonic chapter table — is *not pass-shaped yet*. That conversion
//! is the heir named on [`Schedule::Seed`].

use super::cadence::Cadence;

/// **One pass's schedule** — the SCHEDULE axis as a value, sitting above
/// [`Cadence`] the way *whether* sits above *how often*.
///
/// Not to be confused with [`DeepSchedule`](super::runner::DeepSchedule), which is
/// the validated, topo-sorted **roster**. This is one pass's answer; that is the
/// whole world's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Schedule {
    /// **Pre-loop once, never in-loop** — an initial condition. Establishes the
    /// t=0 state some later pass reads, and integrates **zero** time (the runner
    /// hands it `dt = 0.0`).
    ///
    /// **Heir: declared epochs** (the 2026-07-23 pass-cadence sketch's unbuilt
    /// half; ROADMAP continuation slot (d)). `Seed` is *"member of the setup
    /// epoch"* said small — one anonymous epoch before the numbered ones, with no
    /// name, no ordering against other epochs and no way for a world to author
    /// another. When epochs become authored data this variant converts
    /// mechanically into membership of the epoch named `setup`, and should.
    Seed,
    /// **In-loop only**, at this cadence. The shipped roster is entirely this.
    Step(Cadence),
    /// **Both**: an initial condition *and* a stepping pass. The variant that a
    /// `seeded: bool` beside a cadence could express and a `rate: 0` sentinel
    /// could not, which is why the axis is a sum type.
    SeedAndStep(Cadence),
}

impl Schedule {
    /// Step every epoch, one turn — the default, and every pass in the erosion
    /// pipeline.
    pub const EVERY_EPOCH: Self = Self::Step(Cadence::EVERY_EPOCH);

    /// Step every `period` epochs, one turn per firing. The shape of the three
    /// coarse-rate passes (climate, geotherm, head).
    #[must_use]
    pub const fn every(period: u32) -> Self {
        Self::Step(Cadence::every(period))
    }

    /// The in-loop cadence, or `None` for a pure [`Self::Seed`] — which has no
    /// cadence because it has no steps to space out.
    #[inline]
    #[must_use]
    pub const fn cadence(self) -> Option<Cadence> {
        match self {
            Self::Seed => None,
            Self::Step(c) | Self::SeedAndStep(c) => Some(c),
        }
    }

    /// Does this pass run **before** the loop?
    #[inline]
    #[must_use]
    pub const fn seeds(self) -> bool {
        matches!(self, Self::Seed | Self::SeedAndStep(_))
    }

    /// **The firing test.** `Some(cadence)` if this pass steps on `epoch`.
    ///
    /// `epoch % period == 0` — **and nothing else**. Epoch 0 fires for everyone
    /// (DECIDED 2026-07-29); the `epoch > 0` clause that used to sit here was the
    /// skip rule, and it is gone. A pure seed never fires.
    #[inline]
    #[must_use]
    pub fn firing(self, epoch: u32) -> Option<Cadence> {
        self.cadence().filter(|c| epoch.is_multiple_of(c.period()))
    }

    /// [`Self::firing`] as a predicate.
    #[inline]
    #[must_use]
    pub fn fires(self, epoch: u32) -> bool {
        self.firing(epoch).is_some()
    }

    /// Re-rate this schedule, **preserving the variant**. This is how a world's
    /// [`CadenceTable`](super::cadence::CadenceTable) reaches the roster.
    ///
    /// A [`Self::Seed`] is returned unchanged: a world may say how often a pass
    /// steps, but it may not hand a step body to a pass that has none. Promoting
    /// a seed to a stepper is a **declared-epochs** question, not a rate one.
    #[inline]
    #[must_use]
    pub const fn with_cadence(self, cadence: Cadence) -> Self {
        match self {
            Self::Seed => Self::Seed,
            Self::Step(_) => Self::Step(cadence),
            Self::SeedAndStep(_) => Self::SeedAndStep(cadence),
        }
    }

    /// **The clock, as a number**: epochs of world time this pass integrates over
    /// a run of `iterations` epochs.
    ///
    /// A firing at epoch `e` opens a phase covering `[e, e + period)` — that is
    /// what [`Cadence::phase_total`] *is* — and firings land on the multiples of
    /// `period` from zero, so the phases **tile** the epoch axis. The run's window
    /// `[0, iterations)` is therefore covered exactly once, except that the run may
    /// end *inside* the final phase, which is the only reason the last term is
    /// clipped. The result is exactly `iterations`, at every period, and that is
    /// the invariant `tests/schedule_axis.rs` asserts for every pass in every
    /// roster.
    ///
    /// A [`Self::Seed`] integrates **zero**, by construction and by definition.
    #[must_use]
    pub fn integrated_dt(self, iterations: u32) -> f64 {
        let Some(c) = self.cadence() else {
            return 0.0;
        };
        let mut total = 0.0;
        let mut e = 0u32;
        while e < iterations {
            total += c.phase_total().min(f64::from(iterations - e));
            e += c.period();
        }
        total
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::EVERY_EPOCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_schedule_steps_every_epoch_and_does_not_seed() {
        let s = Schedule::default();
        assert_eq!(s, Schedule::Step(Cadence::EVERY_EPOCH));
        assert!(!s.seeds());
        assert_eq!(s.cadence(), Some(Cadence::EVERY_EPOCH));
    }

    /// **The rule that replaced the skip rule**, stated as narrowly as it is
    /// implemented: firing is `epoch % period == 0`, so epoch 0 fires for every
    /// period, and a pure seed fires never.
    #[test]
    fn epoch_zero_fires_for_every_period_and_never_for_a_seed() {
        for period in [1u32, 2, 20, 40] {
            let s = Schedule::every(period);
            assert!(s.fires(0), "period {period} must fire at epoch 0");
            assert!(s.fires(period));
            assert!(s.fires(period * 3));
            if period > 1 {
                assert!(!s.fires(1));
                assert!(!s.fires(period - 1));
            }
        }
        assert!(!Schedule::Seed.fires(0));
        assert!(!Schedule::Seed.fires(7));
        assert_eq!(Schedule::Seed.cadence(), None);
        assert!(Schedule::Seed.seeds());
        assert!(Schedule::SeedAndStep(Cadence::every(20)).fires(0));
        assert!(Schedule::SeedAndStep(Cadence::every(20)).seeds());
    }

    /// **The clock invariant, in the small.** `tests/schedule_axis.rs` runs it over
    /// the real rosters; this proves the arithmetic itself, including the cases the
    /// production periods never reach — a period that does *not* divide the run.
    #[test]
    fn every_period_integrates_exactly_the_elapsed_time() {
        for iterations in [1u32, 7, 20, 200, 201] {
            for period in [1u32, 2, 3, 20, 40, 199, 500] {
                for sub in [1u32, 2, 5] {
                    let s = Schedule::Step(Cadence::new(period, sub).expect("nonzero"));
                    let got = s.integrated_dt(iterations);
                    assert!(
                        (got - f64::from(iterations)).abs() < 1e-9,
                        "period {period}/{sub} over {iterations} epochs integrated \
                         {got}, not {iterations}"
                    );
                }
            }
        }
        // A seed integrates zero time — the whole content of "a seed is an
        // initial condition".
        assert_eq!(Schedule::Seed.integrated_dt(200), 0.0);
        // And a seed-and-step integrates the run, exactly like a step: the seed
        // adds no time to the clock.
        assert!(
            (Schedule::SeedAndStep(Cadence::every(20)).integrated_dt(200) - 200.0).abs() < 1e-9
        );
    }

    /// **What the skip rule cost, measured.** Kept as a test rather than a
    /// paragraph because it is the argument for the whole slice: the deleted
    /// `epoch > 0` clause deleted the FIRST TILE, not a boundary case.
    #[test]
    fn the_deleted_skip_rule_lost_a_whole_period_of_world_time() {
        let iterations = 200u32;
        for period in [20u32, 40] {
            let honest = Schedule::every(period).integrated_dt(iterations);
            // The old rule: `epoch % period == 0 && (period == 1 || epoch > 0)`.
            let skipped: f64 = (1..iterations)
                .filter(|e| e.is_multiple_of(period))
                .map(|e| f64::from(period).min(f64::from(iterations - e)))
                .sum();
            assert_eq!(honest, f64::from(iterations));
            assert_eq!(
                skipped,
                f64::from(iterations - period),
                "the skip rule cost exactly one period"
            );
        }
    }

    /// A world re-rates a stepping pass and cannot re-shape a seeding one.
    #[test]
    fn re_rating_preserves_the_variant() {
        let c = Cadence::sub_turned(5);
        assert_eq!(Schedule::EVERY_EPOCH.with_cadence(c), Schedule::Step(c));
        assert_eq!(
            Schedule::SeedAndStep(Cadence::every(20)).with_cadence(c),
            Schedule::SeedAndStep(c)
        );
        assert_eq!(
            Schedule::Seed.with_cadence(c),
            Schedule::Seed,
            "a world may rate a step; it may not invent one"
        );
    }
}
