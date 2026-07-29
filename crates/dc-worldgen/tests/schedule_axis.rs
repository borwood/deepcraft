//! **SCHEDULE — the clock invariant, over the real rosters** (journal/0124;
//! `ARCHITECTURE.md` § *Schedule — DECIDED 2026-07-29 (user): seed is an initial
//! condition, and epoch 0 fires for everyone*).
//!
//! The unit tests prove the arithmetic (`deeptime::schedule`) and that the runner
//! honours it (`deeptime::runner`). This file proves the one thing only the shipped
//! roster can say:
//!
//! > **Every pass integrates exactly as much world time as the world lasts.**
//!
//! That invariant became *assertable* only when E3 made `dt` a real number
//! (journal/0123) and *true* only when the epoch-0 skip rule was deleted. Under the
//! skip a period-20 pass in a 200-epoch world integrated 180 epochs while its
//! period-1 neighbours integrated 200 — the passes were on different clocks, which
//! is precisely what a scheduler exists to prevent.
//!
//! **No world is built here, and that is the point.** When a pass runs is a property
//! of the roster and the config, not of the terrain: the assertion is over
//! `deep_passes_with(cfg, table)`, which is what the runner is handed. It is
//! therefore scale-free by construction and costs milliseconds — the smallest
//! instrument that can see the question (CLAUDE.md § Gates, *size the test, not the
//! report*). The hash half of this slice's evidence lives in `rate_axis.rs` and
//! `providers_golden.rs`, which do build worlds.

use dc_worldgen::deeptime::runner::{deep_passes, deep_passes_with};
use dc_worldgen::deeptime::{Cadence, CadenceTable, DEEP_ITERATIONS, DeepConfig};

/// The production flag set — the config the goldens run — plus the two rosters
/// that differ from it in pass membership, so the claim covers every pass the
/// crate can schedule rather than the default seventeen.
fn rosters() -> Vec<(&'static str, DeepConfig)> {
    let all_on = DeepConfig {
        tectonic_history: true,
        biotic: true,
        full_agents: true,
        erodibility: true,
        iterations: DEEP_ITERATIONS,
        ..DeepConfig::default()
    };
    vec![
        (
            "weather_inventory on",
            DeepConfig {
                weather_inventory: true,
                ..all_on
            },
        ),
        (
            "legacy (no tectonic history, no agents)",
            DeepConfig {
                tectonic_history: false,
                full_agents: false,
                head_field: false,
                ..all_on
            },
        ),
        ("production", all_on),
    ]
}

/// **THE CLOCK INVARIANT.** Every pass in every roster integrates exactly the
/// run's elapsed time.
///
/// A firing at epoch `e` opens a phase covering `[e, e + period)`. Firings land on
/// the multiples of `period` **from zero**, so the phases tile the epoch axis and
/// the run's window `[0, iterations)` is covered exactly once — clipped only where
/// the world ends inside the final phase.
///
/// Run at several iteration counts on purpose. `DEEP_ITERATIONS` (200) is divisible
/// by both shipped coarse periods (20, 40), which would let a period-that-does-not-
/// divide bug hide; 199 and 7 are the cases where the final phase is genuinely
/// truncated, and 1 is the world that is over before the second epoch.
#[test]
fn every_pass_integrates_exactly_the_worlds_elapsed_time() {
    for (name, base) in rosters() {
        for iterations in [1u32, 7, 199, DEEP_ITERATIONS] {
            let cfg = DeepConfig { iterations, ..base };
            for p in deep_passes(&cfg) {
                let got = p.schedule.integrated_dt(iterations);
                assert!(
                    (got - f64::from(iterations)).abs() < 1e-9,
                    "{name}/{iterations} epochs: {} integrated {got} epochs of world \
                     time, not {iterations} — the passes are on different clocks",
                    p.id
                );
            }
        }
    }
}

/// **The invariant survives a world authoring the rate**, which is the case the
/// deleted skip rule silently broke: it was written for three seeded passes and
/// then inherited by anything a `CadenceTable` re-rated, so authoring a period onto
/// an erosion pass deleted that pass's first phase and nobody decided it.
///
/// Both directions of the RATE axis, because they enter the sum differently:
/// `period` changes how many firings there are, `sub_turns` changes how the phase
/// is divided between them.
#[test]
fn an_authored_cadence_cannot_desync_a_passs_clock() {
    let cfg = DeepConfig {
        tectonic_history: true,
        biotic: true,
        full_agents: true,
        erodibility: true,
        iterations: DEEP_ITERATIONS,
        ..DeepConfig::default()
    };
    let table = CadenceTable::empty()
        .with("dc:deep/diffuse", Cadence::sub_turned(5))
        .with("dc:deep/eolian", Cadence::every(3))
        .with("dc:deep/transport", Cadence::new(7, 2).expect("nonzero"))
        .with("dc:deep/climate", Cadence::every(9));
    for p in deep_passes_with(&cfg, &table) {
        let got = p.schedule.integrated_dt(cfg.iterations);
        assert!(
            (got - f64::from(cfg.iterations)).abs() < 1e-9,
            "{} integrated {got} epochs under an authored cadence, not {}",
            p.id,
            cfg.iterations
        );
    }
}

/// **Epoch 0 fires for everyone — no exceptions, no skip rule.** Stated directly
/// over the roster as well as through the clock, because the clock invariant is
/// what the rule *buys* and this is the rule itself: a reader who wants to know
/// whether their coarse pass runs in the first epoch should find the answer
/// asserted, not derived.
#[test]
fn epoch_zero_fires_every_pass_in_every_roster() {
    for (name, cfg) in rosters() {
        for p in deep_passes(&cfg) {
            let c = p
                .schedule
                .cadence()
                .unwrap_or_else(|| panic!("{name}: {} declares no in-loop step", p.id));
            assert!(
                p.schedule.fires(0),
                "{name}: {} (period {}) does not fire at epoch 0",
                p.id,
                c.period()
            );
            assert!(
                !p.schedule.seeds(),
                "{name}: {} declares a pre-loop seed — the shipped roster has none \
                 (journal/0124); a new one needs its initial condition named and \
                 this test updated",
                p.id
            );
        }
    }
}
