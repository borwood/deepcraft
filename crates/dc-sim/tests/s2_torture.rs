//! S2 torture tests: determinism, consistency, monotonicity, cascade
//! bounding, contradiction policy, and the traveler's-report scenario.
//!
//! These are the spike's exit criterion — the design claims in
//! docs/spikes/S2-results.md are exactly the properties asserted here.

use dc_sim::statistical::{
    Aspect, Behavior, Ledger, LedgerError, NUM_AGENTS, NUM_REGIONS, ObserveError, Params, RegionId,
    Subject, ToyWorld, Value, force_fact, observe, query,
};

const SEED: u64 = 0x00D5_EED5_2026;

fn world() -> ToyWorld {
    ToyWorld::new(SEED)
}

fn params() -> Params {
    Params {
        k_samples: 200,
        depth: Some(3),
    }
}

/// First agent whose tick-0 home is `region`.
fn agent_in(world: &ToyWorld, region: RegionId) -> Option<u16> {
    (0..NUM_AGENTS).find(|&a| world.agent_home(a) == region)
}

/// A region housing at least two agents, with two of its agents.
fn region_with_two_agents(world: &ToyWorld) -> (RegionId, u16, u16) {
    for r in 0..NUM_REGIONS {
        let agents: Vec<u16> = (0..NUM_AGENTS)
            .filter(|&a| world.agent_home(a) == r)
            .collect();
        if agents.len() >= 2 {
            return (r, agents[0], agents[1]);
        }
    }
    panic!("100 agents over 20 regions must double up somewhere");
}

// ---------------------------------------------------------------------------
// (a) Determinism: same seed + same ledger => identical distributions and
// identical collapse results across runs.
// ---------------------------------------------------------------------------

#[test]
fn determinism_same_seed_same_ledger() {
    let w = world();
    let a = Subject::Agent(0);

    // Identical queries from scratch produce bit-identical distributions.
    let ledger = Ledger::new();
    let (d1, r1) = query(&w, &ledger, a, 24, Aspect::AgentBehavior, params());
    let (d2, r2) = query(&w, &ledger, a, 24, Aspect::AgentBehavior, params());
    assert_eq!(d1, d2);
    assert_eq!(r1.total_weight, r2.total_weight);

    // A fresh world value (new ToyWorld, same seed) changes nothing.
    let (d3, _) = query(&world(), &ledger, a, 24, Aspect::AgentBehavior, params());
    assert_eq!(d1, d3);

    // An identical observation sequence replayed from scratch commits
    // identical facts and returns identical values.
    let run = |w: &ToyWorld| -> (Vec<Value>, Ledger) {
        let mut ledger = Ledger::new();
        let mut values = Vec::new();
        for (subject, t, aspect) in [
            (Subject::Agent(0), 10, Aspect::AgentBehavior),
            (Subject::Agent(3), 15, Aspect::AgentAlive),
            (Subject::Region(w.agent_home(0)), 12, Aspect::RegionPressure),
            (Subject::Agent(0), 20, Aspect::AgentRegion),
        ] {
            let (v, _) = observe(w, &mut ledger, subject, t, aspect, params()).unwrap();
            values.push(v);
        }
        (values, ledger)
    };
    let (v1, l1) = run(&w);
    let (v2, l2) = run(&world());
    assert_eq!(v1, v2);
    assert_eq!(l1, l2);
}

// ---------------------------------------------------------------------------
// (b) Consistency: queries after an observation agree with the committed
// fact; interleaved observations of overlapping systems never contradict.
// ---------------------------------------------------------------------------

#[test]
fn consistency_queries_agree_with_committed_facts() {
    let w = world();
    let mut ledger = Ledger::new();
    let a = Subject::Agent(0);
    let t1 = 12;

    let (v, _) = observe(&w, &mut ledger, a, t1, Aspect::AgentBehavior, params()).unwrap();

    // Re-derivation of T1 puts probability 1 on the committed value.
    let (d, _) = query(&w, &ledger, a, t1, Aspect::AgentBehavior, params());
    assert!(
        (d.prob(&v) - 1.0).abs() < 1e-12,
        "committed fact must be certain"
    );

    // A partial view of the same tick is implied by the full fact.
    let alive = matches!(v, Value::Behavior(b) if b != Behavior::Dead);
    let (d_alive, _) = query(&w, &ledger, a, t1, Aspect::AgentAlive, params());
    assert!((d_alive.prob(&Value::Alive(alive)) - 1.0).abs() < 1e-12);

    // The fact holds under a *different* collapse bound too.
    let wide = Params {
        depth: Some(5),
        ..params()
    };
    let (d_wide, _) = query(&w, &ledger, a, t1, Aspect::AgentBehavior, wide);
    assert!((d_wide.prob(&v) - 1.0).abs() < 1e-12);
}

#[test]
fn consistency_committed_death_propagates_forward() {
    let w = world();
    let mut ledger = Ledger::new();
    let b = Subject::Agent(7);
    force_fact(
        &w,
        &mut ledger,
        b,
        12,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Dead),
        params(),
    )
    .unwrap();
    // Dead is absorbing: every later query must be certain of it.
    let (d, _) = query(&w, &ledger, b, 20, Aspect::AgentBehavior, params());
    assert!((d.prob(&Value::Behavior(Behavior::Dead)) - 1.0).abs() < 1e-12);
    let (d_alive, _) = query(&w, &ledger, b, 30, Aspect::AgentAlive, params());
    assert!((d_alive.prob(&Value::Alive(false)) - 1.0).abs() < 1e-12);
}

#[test]
fn consistency_future_facts_smooth_the_past() {
    let w = world();
    let mut ledger = Ledger::new();
    let c = Subject::Agent(11);
    force_fact(
        &w,
        &mut ledger,
        c,
        18,
        Aspect::AgentAlive,
        Value::Alive(true),
        params(),
    )
    .unwrap();
    // Alive at 18 forbids dead at 6 in every surviving history.
    let (d, _) = query(&w, &ledger, c, 6, Aspect::AgentBehavior, params());
    assert_eq!(d.prob(&Value::Behavior(Behavior::Dead)), 0.0);
    let (d_alive, _) = query(&w, &ledger, c, 6, Aspect::AgentAlive, params());
    assert!((d_alive.prob(&Value::Alive(true)) - 1.0).abs() < 1e-12);
}

#[test]
fn consistency_interleaved_overlapping_observations() {
    let w = world();
    let mut ledger = Ledger::new();
    let home_a = w.agent_home(0);
    let a = Subject::Agent(0);
    // A second agent whose scope overlaps A's, and A's own home region.
    let nearby = *w
        .ball(home_a, Some(2))
        .iter()
        .find(|&&r| r != home_a && agent_in(&w, r).is_some())
        .expect("some nearby region houses an agent");
    let d_agent = Subject::Agent(agent_in(&w, nearby).unwrap());
    let region = Subject::Region(home_a);

    // Interleave observations of overlapping systems at interleaved times.
    let (va, _) = observe(&w, &mut ledger, a, 10, Aspect::AgentBehavior, params()).unwrap();
    let (vd, _) = observe(
        &w,
        &mut ledger,
        d_agent,
        11,
        Aspect::AgentBehavior,
        params(),
    )
    .unwrap();
    let (vr, _) = observe(
        &w,
        &mut ledger,
        region,
        12,
        Aspect::RegionPressure,
        params(),
    )
    .unwrap();
    let (va2, _) = observe(&w, &mut ledger, a, 14, Aspect::AgentRegion, params()).unwrap();

    // Every committed fact remains certain under later re-derivation,
    // regardless of which anchor the query uses.
    for (subject, t, aspect, v) in [
        (a, 10, Aspect::AgentBehavior, va),
        (d_agent, 11, Aspect::AgentBehavior, vd),
        (region, 12, Aspect::RegionPressure, vr),
        (a, 14, Aspect::AgentRegion, va2),
    ] {
        let (dist, _) = query(&w, &ledger, subject, t, aspect, params());
        assert!(
            (dist.prob(&v) - 1.0).abs() < 1e-12,
            "{subject:?} {aspect:?} at t={t} drifted from committed {v:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// (c) Monotone ledger: committing never rewrites the past; observations only
// narrow, never contradict.
// ---------------------------------------------------------------------------

#[test]
fn monotone_ledger_prefix_is_immutable() {
    let w = world();
    let mut ledger = Ledger::new();
    observe(
        &w,
        &mut ledger,
        Subject::Agent(2),
        8,
        Aspect::AgentBehavior,
        params(),
    )
    .unwrap();
    observe(
        &w,
        &mut ledger,
        Subject::Region(5),
        9,
        Aspect::RegionPressure,
        params(),
    )
    .unwrap();
    let snapshot: Vec<_> = ledger.facts().to_vec();

    observe(
        &w,
        &mut ledger,
        Subject::Agent(2),
        16,
        Aspect::AgentAlive,
        params(),
    )
    .unwrap();
    force_fact(
        &w,
        &mut ledger,
        Subject::Agent(40),
        10,
        Aspect::AgentAlive,
        Value::Alive(true),
        params(),
    )
    .unwrap();

    assert!(ledger.len() > snapshot.len());
    assert_eq!(&ledger.facts()[..snapshot.len()], &snapshot[..]);
}

#[test]
fn monotone_observation_narrows_never_contradicts() {
    let w = world();
    let mut ledger = Ledger::new();
    let a = Subject::Agent(5);
    let t = 14;

    // Partial observation first...
    let (alive, _) = observe(&w, &mut ledger, a, t, Aspect::AgentAlive, params()).unwrap();
    // ...then the full aspect: must refine the partial fact, not fight it.
    let (behavior, _) = observe(&w, &mut ledger, a, t, Aspect::AgentBehavior, params()).unwrap();
    match (alive, behavior) {
        (Value::Alive(true), Value::Behavior(b)) => assert_ne!(b, Behavior::Dead),
        (Value::Alive(false), Value::Behavior(b)) => assert_eq!(b, Behavior::Dead),
        other => panic!("unexpected value shapes: {other:?}"),
    }

    // The earlier partial fact is still certain.
    let (d, _) = query(&w, &ledger, a, t, Aspect::AgentAlive, params());
    assert!((d.prob(&alive) - 1.0).abs() < 1e-12);

    // Re-observing an already committed aspect is an idempotent read.
    let len = ledger.len();
    let (again, _) = observe(&w, &mut ledger, a, t, Aspect::AgentBehavior, params()).unwrap();
    assert_eq!(again, behavior);
    assert_eq!(ledger.len(), len);
}

// ---------------------------------------------------------------------------
// (d) Cascade bounding: an observe at depth N touches at most the depth-N
// neighbourhood (instrumented and asserted).
// ---------------------------------------------------------------------------

#[test]
fn cascade_is_bounded_to_depth_n() {
    let w = world();
    let agent = 0u16;
    let anchor = w.agent_home(agent);

    for depth in [1u32, 2, 3] {
        let mut ledger = Ledger::new();
        let p = Params {
            k_samples: 200,
            depth: Some(depth),
        };
        let (_, report) = observe(
            &w,
            &mut ledger,
            Subject::Agent(agent),
            16,
            Aspect::AgentBehavior,
            p,
        )
        .unwrap();

        let ball = w.ball(anchor, Some(depth));
        assert_eq!(
            report.touched_regions, ball,
            "depth-{depth} collapse must touch exactly the depth-{depth} ball"
        );
        for &a in &report.touched_agents {
            assert!(ball.contains(&w.agent_home(a)));
        }
        // Frontier regions are consulted for priors only and sit just outside.
        for &f in &report.frontier_regions {
            assert!(!ball.contains(&f));
            assert!(w.neighbors(f).iter().any(|n| ball.contains(n)));
        }
    }

    // The bound is real: a shallow collapse does not touch the whole world.
    let mut ledger = Ledger::new();
    let p = Params {
        k_samples: 200,
        depth: Some(1),
    };
    let (_, report) = observe(
        &w,
        &mut ledger,
        Subject::Agent(agent),
        16,
        Aspect::AgentBehavior,
        p,
    )
    .unwrap();
    assert!(report.touched_regions.len() < NUM_REGIONS as usize);
    assert!(report.touched_agents.len() < NUM_AGENTS as usize);
}

// ---------------------------------------------------------------------------
// (e) Impossible observations: the policy is reject-don't-reweight. A forced
// fact that contradicts the ledger bounces off; the ledger never mutates.
// ---------------------------------------------------------------------------

#[test]
fn impossible_forced_fact_is_rejected_ledger_untouched() {
    let w = world();
    let mut ledger = Ledger::new();
    let a = Subject::Agent(4);

    // Exact-key conflict: same subject-time-aspect, different value.
    force_fact(
        &w,
        &mut ledger,
        a,
        8,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Farming),
        params(),
    )
    .unwrap();
    let before = ledger.facts().to_vec();
    let err = force_fact(
        &w,
        &mut ledger,
        a,
        8,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Dead),
        params(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ObserveError::Ledger(LedgerError::ConflictsWithCommitted { .. })
    ));
    assert_eq!(ledger.facts(), &before[..]);

    // Dynamic contradiction: dead at 10 can never be farming at 15.
    let b = Subject::Agent(9);
    force_fact(
        &w,
        &mut ledger,
        b,
        10,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Dead),
        params(),
    )
    .unwrap();
    let before = ledger.facts().to_vec();
    let err = force_fact(
        &w,
        &mut ledger,
        b,
        15,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Farming),
        params(),
    )
    .unwrap_err();
    assert!(matches!(err, ObserveError::Contradiction { .. }));
    assert_eq!(ledger.facts(), &before[..]);

    // Cross-aspect structural contradiction at the same tick.
    let c = Subject::Agent(13);
    force_fact(
        &w,
        &mut ledger,
        c,
        6,
        Aspect::AgentAlive,
        Value::Alive(false),
        params(),
    )
    .unwrap();
    let before = ledger.facts().to_vec();
    let err = force_fact(
        &w,
        &mut ledger,
        c,
        6,
        Aspect::AgentBehavior,
        Value::Behavior(Behavior::Fortifying),
        params(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ObserveError::Ledger(LedgerError::ConflictsWithCommitted { .. })
    ));
    assert_eq!(ledger.facts(), &before[..]);

    // Malformed fact: value from the wrong aspect family.
    let err = force_fact(
        &w,
        &mut ledger,
        c,
        7,
        Aspect::AgentBehavior,
        Value::Pressure(2),
        params(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ObserveError::Ledger(LedgerError::AspectValueMismatch { .. })
    ));
}

// ---------------------------------------------------------------------------
// (f) Traveler's report: facts committed second-hand about a distant place
// must hold up when the place is later visited in person.
// ---------------------------------------------------------------------------

#[test]
fn travelers_report_survives_the_visit() {
    let w = world();
    let mut ledger = Ledger::new();
    let (x, b, c) = region_with_two_agents(&w);
    let (t_report, t_meet, t_visit) = (10u32, 14u32, 20u32);

    // Meeting traveler B away from X commits partial facts about X's recent
    // state: the report. (B's own whereabouts at the meeting also commit.)
    let (p_report, _) = observe(
        &w,
        &mut ledger,
        Subject::Region(x),
        t_report,
        Aspect::RegionPressure,
        params(),
    )
    .unwrap();
    let (c_alive_report, _) = observe(
        &w,
        &mut ledger,
        Subject::Agent(c),
        t_report,
        Aspect::AgentAlive,
        params(),
    )
    .unwrap();
    observe(
        &w,
        &mut ledger,
        Subject::Agent(b),
        t_meet,
        Aspect::AgentRegion,
        params(),
    )
    .unwrap();

    // Later, visiting X in person: fresh observations of the same systems.
    let (_, _) = observe(
        &w,
        &mut ledger,
        Subject::Region(x),
        t_visit,
        Aspect::RegionPressure,
        params(),
    )
    .unwrap();
    let (c_behavior, _) = observe(
        &w,
        &mut ledger,
        Subject::Agent(c),
        t_visit,
        Aspect::AgentBehavior,
        params(),
    )
    .unwrap();

    // The in-person view of the reported tick still matches the report.
    let (d, _) = query(
        &w,
        &ledger,
        Subject::Region(x),
        t_report,
        Aspect::RegionPressure,
        params(),
    );
    assert!((d.prob(&p_report) - 1.0).abs() < 1e-12);
    let (d_c, _) = query(
        &w,
        &ledger,
        Subject::Agent(c),
        t_report,
        Aspect::AgentAlive,
        params(),
    );
    assert!((d_c.prob(&c_alive_report) - 1.0).abs() < 1e-12);

    // And the visit's own observations respect the report's implications:
    // an agent reported dead cannot be found alive later.
    if c_alive_report == Value::Alive(false) {
        assert_eq!(c_behavior, Value::Behavior(Behavior::Dead));
    }

    // Re-deriving C's full behavior at the reported tick never contradicts
    // the partial fact from the report.
    let (d_beh, _) = query(
        &w,
        &ledger,
        Subject::Agent(c),
        t_report,
        Aspect::AgentBehavior,
        params(),
    );
    let reported_alive = c_alive_report == Value::Alive(true);
    for (v, p) in d_beh.support() {
        if p > 0.0 {
            let is_alive = !matches!(v, Value::Behavior(Behavior::Dead));
            assert_eq!(is_alive, reported_alive);
        }
    }
}
