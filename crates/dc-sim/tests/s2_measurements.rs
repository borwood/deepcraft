//! S2 measurements: how much the depth bound distorts distributions, and
//! order-of-magnitude timings for queries and collapses.
//!
//! Run with `cargo test -p dc-sim --test s2_measurements -- --nocapture` to
//! see the tables; recorded numbers live in docs/spikes/S2-results.md.
//!
//! Timing uses `std::time::Instant` — measurement of the test harness, not
//! simulation input; the sim itself never reads a clock.

use std::time::Instant;

use dc_sim::statistical::{
    Aspect, Distribution, Ledger, Params, Subject, ToyWorld, Value, force_fact, observe, query,
};

const SEED: u64 = 0x00D5_EED5_2026;

fn behavior_dist(
    w: &ToyWorld,
    ledger: &Ledger,
    agent: u16,
    t: u32,
    depth: Option<u32>,
    k: u32,
) -> Distribution {
    let p = Params {
        k_samples: k,
        depth,
    };
    query(
        w,
        ledger,
        Subject::Agent(agent),
        t,
        Aspect::AgentBehavior,
        p,
    )
    .0
}

/// How far does bounding the collapse depth bend the answer? Compare the
/// depth-N distributions against the unbounded one (total variation).
/// Addressed draws mean all depths share the same underlying noise, so the
/// differences below are scope effects, not sampling noise.
#[test]
fn distortion_from_bounded_depth() {
    let w = ToyWorld::new(SEED);
    let agent = 0u16;
    let anchor = w.agent_home(agent);
    let t = 32;
    let k = 512;
    let diameter = w.diameter();

    // Scenario 1: empty ledger — pure frontier-synthesis distortion.
    let ledger = Ledger::new();
    let unbounded = behavior_dist(&w, &ledger, agent, t, None, k);
    println!("-- distortion vs unbounded (empty ledger, agent {agent}, t={t}, K={k}) --");
    for depth in 1..=diameter {
        let d = behavior_dist(&w, &ledger, agent, t, Some(depth), k);
        println!("  depth {depth}: TV = {:.4}", d.tv_distance(&unbounded));
    }
    // At depth >= diameter the scope is the whole graph: no frontier exists,
    // so the bounded run must be *identical* to the unbounded one.
    let full = behavior_dist(&w, &ledger, agent, t, Some(diameter), k);
    assert_eq!(full.tv_distance(&unbounded), 0.0);

    // Scenario 2: a strong committed fact 3 hops away — bounds shallower than
    // the fact's distance cannot see it and must eat the inconsistency as
    // silent distortion.
    let far_region = *w
        .ball(anchor, Some(3))
        .difference(&w.ball(anchor, Some(2)))
        .next()
        .expect("a region at exactly distance 3");
    let mut ledger = Ledger::new();
    for ft in [8u32, 10, 12] {
        force_fact(
            &w,
            &mut ledger,
            Subject::Region(far_region),
            ft,
            Aspect::RegionPressure,
            Value::Pressure(2),
            Params {
                k_samples: k,
                depth: None,
            },
        )
        .unwrap();
    }
    let unbounded_f = behavior_dist(&w, &ledger, agent, t, None, k);
    println!(
        "-- distortion vs unbounded (region {far_region} raided at t=8..12, distance 3 from anchor {anchor}) --"
    );
    for depth in 1..=diameter {
        let d = behavior_dist(&w, &ledger, agent, t, Some(depth), k);
        println!("  depth {depth}: TV = {:.4}", d.tv_distance(&unbounded_f));
    }
    let full_f = behavior_dist(&w, &ledger, agent, t, Some(diameter), k);
    assert_eq!(full_f.tv_distance(&unbounded_f), 0.0);

    // The committed fact itself is invisible below depth 3 and certain at or
    // above it — quantify that edge directly.
    let q = |depth| {
        query(
            &w,
            &ledger,
            Subject::Region(far_region),
            10,
            Aspect::RegionPressure,
            Params {
                k_samples: k,
                depth,
            },
        )
        .0
    };
    // Anchored on the fact's own region every depth sees it; anchored on the
    // agent the shallow scope ignores it. Demonstrate via the agent-anchored
    // horizon: shallow reports don't extend to the fact at all.
    let (_, shallow_report) = query(
        &w,
        &ledger,
        Subject::Agent(agent),
        4,
        Aspect::AgentBehavior,
        Params {
            k_samples: k,
            depth: Some(2),
        },
    );
    assert_eq!(
        shallow_report.horizon, 4,
        "facts outside a depth-2 scope must not extend its horizon"
    );
    let (_, deep_report) = query(
        &w,
        &ledger,
        Subject::Agent(agent),
        4,
        Aspect::AgentBehavior,
        Params {
            k_samples: k,
            depth: Some(3),
        },
    );
    assert_eq!(
        deep_report.horizon, 12,
        "in-scope facts extend the horizon for smoothing"
    );
    let on_fact = q(Some(0));
    assert!((on_fact.prob(&Value::Pressure(2)) - 1.0).abs() < 1e-12);
}

/// Order-of-magnitude timings; prints, asserts nothing beyond sanity.
#[test]
fn perf_sanity() {
    let w = ToyWorld::new(SEED);
    let ledger = Ledger::new();
    let agent = 0u16;
    let k = 1024;
    let t = 48;

    let start = Instant::now();
    let (d, r) = query(
        &w,
        &ledger,
        Subject::Agent(agent),
        t,
        Aspect::AgentBehavior,
        Params {
            k_samples: k,
            depth: Some(3),
        },
    );
    let bounded = start.elapsed();
    println!(
        "query  depth=3   K={k} T={t}: {bounded:?}  ({} regions, {} agents, ESS {:.0})",
        r.touched_regions.len(),
        r.touched_agents.len(),
        r.effective_samples
    );
    assert!(!d.is_empty());

    let start = Instant::now();
    let (d, r) = query(
        &w,
        &ledger,
        Subject::Agent(agent),
        t,
        Aspect::AgentBehavior,
        Params {
            k_samples: k,
            depth: None,
        },
    );
    let unbounded = start.elapsed();
    println!(
        "query  unbounded K={k} T={t}: {unbounded:?}  ({} regions, {} agents, ESS {:.0})",
        r.touched_regions.len(),
        r.touched_agents.len(),
        r.effective_samples
    );
    assert!(!d.is_empty());

    let mut ledger = Ledger::new();
    let start = Instant::now();
    observe(
        &w,
        &mut ledger,
        Subject::Agent(agent),
        t,
        Aspect::AgentBehavior,
        Params {
            k_samples: k,
            depth: Some(3),
        },
    )
    .unwrap();
    let collapse = start.elapsed();
    println!("observe depth=3  K={k} T={t}: {collapse:?}");
}
