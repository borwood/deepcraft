//! **RATE — the cadence axis of the pass scheduler** (`material-behavior.md` § 5
//! *Cadence: order × rate × window*, RATIFIED 2026-07-24 from the user's
//! 2026-07-23 sketch in `ideas.md` § *Pass cadence*).
//!
//! A pass declares **how often it runs** and **how much world time one turn
//! covers**, and the runner hands it that time as `dt`. Two numbers:
//!
//! - **`period`** — epochs between firings. `1` fires every epoch; `20` is the
//!   climate re-march. This is the *coarse* direction: a pass that has nothing
//!   to respond to mid-chapter runs rarely.
//! - **`sub_turns`** — turns taken **per firing**. This is the *fine* direction —
//!   the user's *"weathering ×5 while tectonics ×1"*. A sub-turned pass is handed
//!   the cell state as the previous sub-turn left it, so it sees mid-epoch
//!   evolution that a once-per-epoch pass cannot.
//!
//! and one derived quantity, which is the whole point of the axis:
//!
//! ```text
//! dt = period / sub_turns          (epochs of world time, per turn)
//! ```
//!
//! The sketch's words for it: *"a phase is handed the cell state at its start,
//! and **duration is a scalar on its transformations**."* `dt` **is** that
//! duration, and it is the `rate × dt` time-base the S16 behavior model was
//! already written against.
//!
//! ## The base unit is the EPOCH, and the sketch said "chapter"
//!
//! The user's sketch phrases a phase length as *"a fraction of a chapter's
//! duration"*; these two numbers are in **epochs**, because the epoch is what the
//! runner's loop iterates. **Nothing of the sketch is lost by that choice and the
//! expressible set is strictly larger**: a chapter is a whole number of epochs, so
//! *"once per chapter"* is `Cadence::every(epochs_per_chapter)` and *"a third of a
//! chapter"* is a `period` a third that size, while `sub_turns` reaches *below* one
//! epoch, which a chapter-fraction cannot. The relative rates the sketch is about —
//! *"weathering ×5 while tectonics ×1"* — are identical either way.
//!
//! It is recorded rather than assumed because the chapter length is currently an
//! **implicit constant** (25 epochs; `flow.md` § 11.1 is the decision that
//! undeclared constants of exactly this kind must become declared, on the WINDOW
//! axis). A cadence stated in chapters would have inherited that constant; stated
//! in epochs it does not.
//!
//! ## Why the numbers are DATA and not constants
//!
//! *"THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP"* (user; `ARCHITECTURE.md`
//! § *The engine is plugin-agnostic, and pass ORDER is authored*). A pass ships a
//! **declared default** cadence — the pack author's opinion about its own process
//! — and a **world** may author a different one through a [`CadenceTable`]. The
//! engine executes whatever it is handed; it derives nothing. That is the same
//! split ORDER is heading for: the declaration is the pack's, the schedule is the
//! world's.
//!
//! An **empty** table is today's shipped schedule, exactly — which is what makes
//! this axis land against journal/0122's known-good fixed point without moving a
//! golden.
//!
//! ## What RATE is NOT
//!
//! **RATE does not own stability substepping** (user, 2026-07-29, rejecting the
//! widening: *"I really hate to make RATE more complex now. Couldn't substepping
//! be solved within the field instead, where it takes `dt` from outside and calcs
//! its own internal multiplier?"*). A field solver takes `dt` from here and
//! sub-divides it *internally* to stay inside its own von Neumann bound — because
//! only the kernel knows its own stability constant. `Erosion::diffuse` does
//! exactly that today by hand (`stubs.md` § 30, heir: the S-10 field-solver
//! primitive); this module supplies the `dt` it divides, and nothing more.
//!
//! Sub-turns and sub-steps therefore look alike and are owned by opposite sides:
//! **sub-turns are authored** (a modelling choice about temporal resolution, and
//! every pass in the epoch sees the intermediate state), **sub-steps are derived**
//! (a numerical choice about stability, invisible outside the solver).

use std::num::NonZeroU32;

/// One pass's declared cadence — the RATE axis as a value.
///
/// Both numbers are [`NonZeroU32`] so that a zero cadence — an infinite loop or a
/// division by zero, depending which one you got wrong — is **inexpressible**
/// rather than validated. The same move as `CoarseField` making the raw per-cell
/// read unsayable: the engine's job is to make the unsafe call impossible to
/// write, not to reject it at run time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cadence {
    period: NonZeroU32,
    sub_turns: NonZeroU32,
}

const ONE: NonZeroU32 = NonZeroU32::new(1).expect("1 != 0");

impl Cadence {
    /// Fire every epoch, one turn — `dt = 1.0`. The cadence of every pass in the
    /// erosion pipeline, and the default.
    pub const EVERY_EPOCH: Self = Self {
        period: ONE,
        sub_turns: ONE,
    };

    /// Fire every `period` epochs, one turn per firing — `dt = period`. The shape
    /// of the coarse-rate passes already in the loop (climate, geotherm, head).
    ///
    /// `0` is read as `1`: the config knob it is built from
    /// ([`DeepConfig::remarch_interval`](super::grid::DeepConfig::remarch_interval))
    /// has carried a `.max(1)` since the runner was written, and preserving it
    /// here is what keeps the shipped schedule bit-identical.
    #[must_use]
    pub const fn every(period: u32) -> Self {
        Self {
            period: match NonZeroU32::new(period) {
                Some(p) => p,
                None => ONE,
            },
            sub_turns: ONE,
        }
    }

    /// Fire every epoch, taking `sub_turns` turns each time — `dt = 1/sub_turns`.
    /// The fine direction of the sketch (*"weathering ×5"*).
    #[must_use]
    pub const fn sub_turned(sub_turns: u32) -> Self {
        Self {
            period: ONE,
            sub_turns: match NonZeroU32::new(sub_turns) {
                Some(s) => s,
                None => ONE,
            },
        }
    }

    /// The general form. Returns `None` for a zero on either axis — the only
    /// constructor that can fail, and the one an authored table goes through.
    #[must_use]
    pub fn new(period: u32, sub_turns: u32) -> Option<Self> {
        Some(Self {
            period: NonZeroU32::new(period)?,
            sub_turns: NonZeroU32::new(sub_turns)?,
        })
    }

    /// Epochs between firings.
    #[inline]
    #[must_use]
    pub const fn period(self) -> u32 {
        self.period.get()
    }

    /// Turns taken per firing.
    #[inline]
    #[must_use]
    pub const fn sub_turns(self) -> u32 {
        self.sub_turns.get()
    }

    /// **The phase length: epochs of world time one turn covers.** This is the
    /// number the runner hands the pass, and the number a rate-shaped
    /// transformation multiplies by.
    ///
    /// It is exactly `1.0` for [`Self::EVERY_EPOCH`] and exactly `period` for
    /// [`Self::every`] — the two shapes the shipped roster uses — so every pass
    /// that starts scaling by `dt` is byte-identical on the shipped world
    /// (`x * 1.0 == x` for f64).
    #[inline]
    #[must_use]
    pub fn dt(self) -> f64 {
        f64::from(self.period.get()) / f64::from(self.sub_turns.get())
    }

    /// **Total world time one firing covers** — `dt × sub_turns = period`. The
    /// invariant that makes sub-turning a *subdivision* rather than a speed-up:
    /// splitting a phase must not change how much time passed.
    #[inline]
    #[must_use]
    pub fn phase_total(self) -> f64 {
        self.dt() * f64::from(self.sub_turns.get())
    }
}

impl Default for Cadence {
    fn default() -> Self {
        Self::EVERY_EPOCH
    }
}

/// **The authored cadence table — RATE as world data.**
///
/// A pass declares a default cadence; a world may override it by pass id. An
/// **empty** table (the [`Default`]) means *"every pass keeps what it declared"*,
/// which is the shipped schedule, which is why the goldens do not move.
///
/// The keys are owned `String`s rather than `&'static str` deliberately: this is
/// the seam a **per-world manifest** feeds when order-as-data lands (the pass
/// architecture's continuation slot (b)), and a manifest's ids are read at run
/// time. Building the manifest format *now* would be building the general
/// mechanism before its caller exists (`session-workflow` § Seam-first #6) — so
/// this is a value that a loader can construct, and no loader yet.
///
/// Unknown ids are **not** an error here: a table may legitimately name a pass
/// that the active roster gated off (`full_agents`, `head_field`, …), exactly as
/// the roster's own `if` guards do. [`Self::unmatched`] exists so a caller that
/// wants to be strict can be.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CadenceTable {
    entries: Vec<(String, Cadence)>,
}

impl CadenceTable {
    /// The empty table — every pass keeps its declared default.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Author one pass's cadence. A repeated id replaces the earlier entry, so a
    /// table cannot hold two answers for one pass.
    #[must_use]
    pub fn with(mut self, pass_id: impl Into<String>, cadence: Cadence) -> Self {
        let id = pass_id.into();
        match self.entries.iter_mut().find(|(k, _)| *k == id) {
            Some(slot) => slot.1 = cadence,
            None => self.entries.push((id, cadence)),
        }
        self
    }

    /// The authored cadence for `pass_id`, or `None` to keep the declared one.
    #[must_use]
    pub fn get(&self, pass_id: &str) -> Option<Cadence> {
        self.entries
            .iter()
            .find(|(k, _)| k == pass_id)
            .map(|(_, c)| *c)
    }

    /// Apply the table to a declared default.
    #[inline]
    #[must_use]
    pub fn resolve(&self, pass_id: &str, declared: Cadence) -> Cadence {
        self.get(pass_id).unwrap_or(declared)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Ids this table authors that are not in `roster` — for a caller that wants
    /// to reject a typo rather than silently ignore it.
    #[must_use]
    pub fn unmatched<'a>(&'a self, roster: &[&str]) -> Vec<&'a str> {
        self.entries
            .iter()
            .map(|(k, _)| k.as_str())
            .filter(|k| !roster.contains(k))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_cadence_is_one_epoch_one_turn() {
        let c = Cadence::default();
        assert_eq!(c.period(), 1);
        assert_eq!(c.sub_turns(), 1);
        // Exactly 1.0, so a transformation that starts scaling by dt is
        // byte-identical on the shipped world.
        assert_eq!(c.dt().to_bits(), 1.0f64.to_bits());
    }

    /// **The subdivision invariant.** Splitting a phase into sub-turns must not
    /// change how much world time the firing covers — otherwise raising a pass's
    /// temporal resolution would silently raise its rate, which is the bug the
    /// axis exists to make impossible.
    #[test]
    fn sub_turns_partition_the_phase_they_do_not_extend_it() {
        for period in [1u32, 3, 20] {
            for sub in [1u32, 2, 5, 8] {
                let c = Cadence::new(period, sub).expect("nonzero");
                assert_eq!(c.dt(), f64::from(period) / f64::from(sub));
                assert!(
                    (c.phase_total() - f64::from(period)).abs() < 1e-12,
                    "{period}/{sub}: dt × sub_turns must be the period"
                );
            }
        }
    }

    #[test]
    fn a_zero_cadence_is_inexpressible() {
        assert!(Cadence::new(0, 1).is_none());
        assert!(Cadence::new(1, 0).is_none());
        // The `every`/`sub_turned` shorthands read 0 as 1, matching the `.max(1)`
        // the runner has always applied to `remarch_interval`.
        assert_eq!(Cadence::every(0), Cadence::EVERY_EPOCH);
        assert_eq!(Cadence::sub_turned(0), Cadence::EVERY_EPOCH);
    }

    #[test]
    fn an_empty_table_keeps_every_declared_cadence() {
        let t = CadenceTable::default();
        assert!(t.is_empty());
        assert_eq!(
            t.resolve("dc:deep/climate", Cadence::every(20)).period(),
            20
        );
    }

    #[test]
    fn the_table_authors_by_pass_id_and_holds_one_answer_each() {
        let t = CadenceTable::empty()
            .with("dc:deep/weather", Cadence::sub_turned(5))
            .with("dc:deep/weather", Cadence::sub_turned(3));
        assert_eq!(t.get("dc:deep/weather").unwrap().sub_turns(), 3);
        assert_eq!(t.get("dc:deep/tectonics"), None);
        assert_eq!(
            t.resolve("dc:deep/tectonics", Cadence::EVERY_EPOCH),
            Cadence::EVERY_EPOCH
        );
        assert_eq!(t.unmatched(&["dc:deep/weather"]), Vec::<&str>::new());
        assert_eq!(t.unmatched(&["dc:deep/frost"]), vec!["dc:deep/weather"]);
    }
}
