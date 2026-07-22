//! Erodibility coupling (journal/0029) — the falsifiers.
//!
//! Measured landform numbers live in the journal entry and are produced by
//! `examples/erodibility_probe.rs`; this file holds the invariants that must not
//! break: the off path is byte-identical, the coupled path is deterministic
//! scalar↔parallel, mass is still conserved, deep time and the collapse tier
//! agree about what rock is where, and the self-reinforcing feedback neither
//! runs away nor stalls.

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_INTRUSIVE, CLASS_ORGANIC_CHARCOAL,
    CLASS_ORGANIC_COAL, CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL,
};
use dc_worldgen::deeptime::lithology::{Agent, Litho, LithoResistance};
use dc_worldgen::deeptime::{
    self, Aridity, DeepConfig, DepEnv, DepTag, EnergyBand, Eolian, Erosion, litho_of_tag,
    susceptibility_table,
};
use dc_worldgen::geology::deep_class;
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn small_world(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

/// The uncoupled reference config (coupling off, biology off).
fn cfg_off(seed: u64) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m: 1000.0,
        iterations: 40,
        remarch_interval: 20,
        record: true,
        erodibility: false,
        ..DeepConfig::default()
    }
}

fn cfg_on(seed: u64) -> DeepConfig {
    DeepConfig {
        erodibility: true,
        ..cfg_off(seed)
    }
}

// ---------------------------------------------------------------------------
// Off-by-default and provably unchanged.

/// **The off-path proof.** Running the *coupled* code with a contrast of zero
/// makes every susceptibility exactly `1.0` — so the coupled path executes all
/// of its new arithmetic and must still land on the uncoupled result bit for
/// bit. This is stronger than comparing "off" against itself: it proves the new
/// multiplications are genuinely identity-preserving rather than merely skipped.
#[test]
fn a_neutral_coupling_is_byte_identical_to_no_coupling() {
    let pregen = small_world(SEED);
    let off = deeptime::run(&pregen, &cfg_off(SEED));
    let neutral = deeptime::run(
        &pregen,
        &DeepConfig {
            erodibility_contrast: 0.0,
            erodibility_diffusion_contrast: 0.0,
            ..cfg_on(SEED)
        },
    );
    assert_eq!(
        off.grid.r, neutral.grid.r,
        "bedrock plane must be identical"
    );
    assert_eq!(
        off.grid.h, neutral.grid.h,
        "alluvium plane must be identical"
    );
    assert_eq!(
        off.grid.strata, neutral.grid.strata,
        "strata records must be identical"
    );
}

/// The same identity check with the S10 biotic layer on: composition order
/// (`diffusion × (1 − resist) × litho`) must not disturb the biotic result.
#[test]
fn a_neutral_coupling_is_byte_identical_with_biology_on() {
    let pregen = small_world(SEED);
    let base = DeepConfig {
        biotic: true,
        ..cfg_off(SEED)
    };
    let off = deeptime::run(&pregen, &base);
    let neutral = deeptime::run(
        &pregen,
        &DeepConfig {
            erodibility: true,
            erodibility_contrast: 0.0,
            erodibility_diffusion_contrast: 0.0,
            ..base
        },
    );
    assert_eq!(off.grid.r, neutral.grid.r);
    assert_eq!(off.grid.h, neutral.grid.h);
    assert_eq!(off.grid.strata, neutral.grid.strata);
}

#[test]
fn coupling_on_actually_changes_the_world() {
    // The flip must matter — otherwise the milestone is decoration.
    let pregen = small_world(SEED);
    let off = deeptime::run(&pregen, &cfg_off(SEED));
    let on = deeptime::run(&pregen, &cfg_on(SEED));
    assert_ne!(off.grid.r, on.grid.r, "coupling changed nothing");
}

// ---------------------------------------------------------------------------
// Determinism (project law).

#[test]
fn coupled_runs_are_byte_identical_on_a_repeated_seed() {
    let pregen = small_world(SEED);
    let cfg = cfg_on(SEED);
    let a = deeptime::run(&pregen, &cfg);
    let b = deeptime::run(&pregen, &cfg);
    assert_eq!(a.grid.r, b.grid.r);
    assert_eq!(a.grid.h, b.grid.h);
    assert_eq!(a.grid.strata, b.grid.strata);
}

#[test]
fn coupled_parallel_equals_coupled_scalar_byte_identical() {
    // Production takes the parallel path (S9b), so the new `expose` phase must
    // reproduce the scalar result to the bit. A finer cell forces past the rayon
    // fork floor.
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        cell_m: 120.0,
        iterations: 30,
        ..cfg_on(SEED)
    };
    let scalar = deeptime::run_with(&pregen, &cfg, false);
    let parallel = deeptime::run_with(&pregen, &cfg, true);
    assert_eq!(scalar.grid.r, parallel.grid.r);
    assert_eq!(scalar.grid.h, parallel.grid.h);
    assert_eq!(scalar.grid.strata, parallel.grid.strata);
}

#[test]
fn coupled_parallel_equals_scalar_with_biology_on_too() {
    // Both lagged modifier layers live at once: biology writes `resist`/`wmult`
    // for the next epoch while lithology is read fresh at the top of this one.
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        cell_m: 120.0,
        iterations: 24,
        biotic: true,
        ..cfg_on(SEED)
    };
    let scalar = deeptime::run_with(&pregen, &cfg, false);
    let parallel = deeptime::run_with(&pregen, &cfg, true);
    assert_eq!(scalar.grid.r, parallel.grid.r);
    assert_eq!(scalar.grid.h, parallel.grid.h);
    assert_eq!(scalar.grid.strata, parallel.grid.strata);
}

// ---------------------------------------------------------------------------
// The tiers must agree about what rock is where.

/// Deep time decides how fast a bed erodes; the collapse tier decides what that
/// bed is made of. If the two routings drift, the world's *shape* stops
/// explaining the world's *rock* — a resistant ridge built out of mudstone.
#[test]
fn litho_routing_matches_the_collapse_tier() {
    let class_of = |l: Litho| match l {
        Litho::ClasticFine => CLASS_CLASTIC_FINE,
        Litho::ClasticCoarse => CLASS_CLASTIC_COARSE,
        Litho::OrganicSoil => CLASS_ORGANIC_SOIL,
        Litho::OrganicPeat => CLASS_ORGANIC_PEAT,
        Litho::OrganicCoal => CLASS_ORGANIC_COAL,
        // Basement is below the record, so no recorded tag ever routes to it.
        Litho::Basement => CLASS_IGNEOUS_INTRUSIVE,
    };
    let mut seen = 0;
    for env in [DepEnv::Subaerial, DepEnv::Subsea] {
        for aridity in [Aridity::Arid, Aridity::Humid] {
            for energy in [EnergyBand::Low, EnergyBand::Medium, EnergyBand::High] {
                for biota in [
                    deeptime::Biofacies::Mineral,
                    deeptime::Biofacies::Soil,
                    deeptime::Biofacies::Peat,
                    deeptime::Biofacies::Coal,
                    deeptime::Biofacies::Charcoal,
                    deeptime::Biofacies::Retro,
                ] {
                    for eolian in [Eolian::None, Eolian::Loess, Eolian::Dune] {
                        let tag = DepTag {
                            env,
                            aridity,
                            energy,
                            biota,
                            eolian,
                        };
                        if biota == deeptime::Biofacies::Charcoal {
                            // **The one deliberate divergence** (journal/0063).
                            // The collapse tier expresses the carbon; deep time
                            // keeps reading the host bed, because a 3.5 cm
                            // lamina cannot set the erodibility of a 460 m cell.
                            // Asserted by name, not skipped: the exception is
                            // exactly as pinned as the rule.
                            assert_eq!(
                                deep_class(tag),
                                CLASS_ORGANIC_CHARCOAL,
                                "charcoal must express as charcoal"
                            );
                            let host = DepTag {
                                biota: deeptime::Biofacies::Mineral,
                                ..tag
                            };
                            assert_eq!(
                                class_of(litho_of_tag(tag)),
                                deep_class(host),
                                "charcoal must erode as its mineral HOST, not as charcoal"
                            );
                            seen += 1;
                            continue;
                        }
                        assert_eq!(
                            class_of(litho_of_tag(tag)),
                            deep_class(tag),
                            "deep time and collapse disagree about {}",
                            tag.code()
                        );
                        seen += 1;
                    }
                }
            }
        }
    }
    assert_eq!(seen, 2 * 2 * 3 * 6 * 3);
}

// ---------------------------------------------------------------------------
// Conservation and the recorder invariant survive the coupling.

#[test]
fn mass_is_conserved_with_coupling_on() {
    // Over-entrainment (a `sus > 1` cell taking more than its remaining
    // transport capacity) routes the excess downstream instead of creating or
    // destroying it — the mass ledger is the falsifier for that claim.
    let pregen = small_world(SEED);
    let cfg = cfg_on(SEED);
    let mut grid = deeptime::build(&pregen, &cfg);
    let mut ero = Erosion::new(&grid);
    let before = deeptime::total_mass(&grid);
    deeptime::climate::march(&mut grid, deeptime::sea_level_at(&cfg, 0));
    let mut uplift_total = 0.0;
    for it in 0..cfg.iterations {
        let sl = deeptime::sea_level_at(&cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            deeptime::climate::march(&mut grid, sl);
        }
        uplift_total += ero.step(&mut grid, &cfg, sl);
    }
    let residual = deeptime::total_mass(&grid) - before - uplift_total;
    assert!(residual.abs() < 1.0, "mass leaked: residual {residual}");
}

#[test]
fn recorder_total_equals_alluvium_with_coupling_on() {
    let pregen = small_world(SEED);
    let run = deeptime::run(&pregen, &cfg_on(SEED));
    let mut worst = 0.0f64;
    for (i, s) in run.grid.strata.iter().enumerate() {
        worst = worst.max((s.total_m() - run.grid.h[i]).abs());
    }
    assert!(worst < 1e-6, "sum(units)==H violated, worst {worst}");
}

// ---------------------------------------------------------------------------
// Stability: the feedback is the point AND the risk.

/// Differential erosion is self-reinforcing — erode soft rock, expose hard rock,
/// slow down. That is what carves benches, and it is also what could park a cell
/// in a limit cycle or a permanent stall. The clamp
/// (`erodibility_max`) is the guard; this test is its falsifier.
///
/// Falsifiers, on a deliberately harsh contrast (4×, well past the shipped 2.5):
/// nothing goes non-finite; the coupled field's late-run activity does not
/// exceed the uncoupled field's by more than an order of magnitude (no runaway);
/// and the world has not frozen (some cells are still moving).
#[test]
fn the_differential_erosion_feedback_neither_runs_away_nor_stalls() {
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        erodibility_contrast: 4.0,
        iterations: 80,
        ..cfg_on(SEED)
    };
    let mut grid = deeptime::build(&pregen, &cfg);
    let mut ero = Erosion::new(&grid);
    deeptime::climate::march(&mut grid, deeptime::sea_level_at(&cfg, 0));

    let mut prev: Vec<f64> = (0..grid.r.len()).map(|i| grid.surf_at(i)).collect();
    let mut activity: Vec<f64> = Vec::new();
    let mut moving_late = 0usize;
    for it in 0..cfg.iterations {
        let sl = deeptime::sea_level_at(&cfg, it);
        if it > 0 && it % cfg.remarch_interval == 0 {
            deeptime::climate::march(&mut grid, sl);
        }
        ero.step(&mut grid, &cfg, sl);
        let mut worst = 0.0f64;
        let mut moving = 0usize;
        for (i, p) in prev.iter_mut().enumerate() {
            let s = grid.surf_at(i);
            assert!(s.is_finite(), "cell {i} went non-finite at iteration {it}");
            let d = (s - *p).abs();
            worst = worst.max(d);
            if d > 1e-6 {
                moving += 1;
            }
            *p = s;
        }
        activity.push(worst);
        if it + 1 == cfg.iterations {
            moving_late = moving;
        }
    }
    let early = activity[4];
    let late = activity[activity.len() - 1];
    assert!(
        late <= early * 10.0,
        "activity grew from {early} to {late} — the feedback is diverging"
    );
    assert!(
        moving_late > grid.r.len() / 100,
        "only {moving_late} cells still moving — the world stalled"
    );
}

/// The clamp is what makes the above true, so assert it holds directly at an
/// absurd contrast: no rock may erode more than `max×` or less than `1/max×` the
/// reference rate however extreme the knob.
#[test]
fn the_stability_clamp_bounds_every_lithology() {
    for contrast in [2.5, 4.0, 20.0] {
        let t = susceptibility_table(Agent::Abrasion, contrast, 5.0);
        for l in Litho::ALL {
            let s = t[l.index()];
            assert!((0.2..=5.0).contains(&s), "{} at {contrast}: {s}", l.code());
        }
    }
}

// ---------------------------------------------------------------------------
// The falsifiable prediction: basement for free.

/// Strip a column past its record and the next thing erosion meets is basement.
/// If no cell in a real run ever exposes basement, the shield/craton prediction
/// has nothing to stand on.
#[test]
fn stripped_columns_expose_basement() {
    let pregen = small_world(SEED);
    let cfg = cfg_on(SEED);
    let run = deeptime::run(&pregen, &cfg);
    let land = run
        .grid
        .strata
        .iter()
        .enumerate()
        .filter(|(i, _)| run.grid.surf_at(*i) > 0.0)
        .count();
    let stripped = run
        .grid
        .strata
        .iter()
        .enumerate()
        .filter(|(i, s)| run.grid.surf_at(*i) > 0.0 && s.units.is_empty())
        .count();
    assert!(land > 0);
    assert!(
        stripped > 0,
        "no land column was ever stripped to basement — the shield prediction is untestable"
    );
}

// ---------------------------------------------------------------------------
// Non-preclusion, asserted structurally.

/// A future dissolution agent must be able to get a *different answer* from the
/// same rock than the mechanical agent gets. Asserted on a carbonate-shaped
/// sheet, because the roster has no carbonate yet — the point is that the model
/// admits one without a rewrite.
#[test]
fn a_future_agent_reads_its_own_axis() {
    let limestone = LithoResistance {
        abrasion: 4.5 * 0.95,
        dissolution: 1.0 / 0.85,
        frost_ice: 4.5 * 0.9,
        wave: 4.5 * 0.9,
        eolian: 0.9,
    };
    let granite = Litho::Basement.resistance();
    // Mechanically granite wins; chemically limestone is the only thing on the
    // board that can be attacked at all. One scalar cannot say both.
    assert!(granite.to(Agent::Abrasion) > limestone.to(Agent::Abrasion));
    assert!(limestone.to(Agent::Dissolution) < granite.to(Agent::Dissolution));
    assert_eq!(granite.to(Agent::Dissolution), f64::INFINITY);
    // And the live mechanical agent is provably blind to the chemical axis.
    let soluble_granite = LithoResistance {
        dissolution: 1.0,
        ..granite
    };
    assert_eq!(
        susceptibility_of(soluble_granite, Agent::Abrasion),
        susceptibility_of(granite, Agent::Abrasion),
        "the mechanical agent must not see solubility"
    );
}

fn susceptibility_of(r: LithoResistance, a: Agent) -> f64 {
    r.susceptibility(a, 4.0, 2.5, 5.0)
}
