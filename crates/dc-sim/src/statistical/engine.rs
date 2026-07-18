//! Query, observe (collapse), and forced-fact commitment over the toy world.
//!
//! **Fluid state is derived, never stored.** A query about `(subject, time,
//! aspect)` re-simulates K candidate histories forward from tick 0, each a
//! pure function of `(world seed, sample index, ledger)`:
//!
//! - every stochastic choice uses an *addressed* draw keyed by
//!   `(seed, sample, salt, entity, tick)` (see `rng.rs`), so the same entity
//!   sees the same noise regardless of query scope;
//! - at each tick, each entity's transition distribution is *conditioned* on
//!   the ledger: candidate outcomes violating any committed fact about that
//!   entity at that tick are filtered out, the sample's weight is multiplied
//!   by the retained probability mass, and the next state is drawn from the
//!   renormalised remainder (sequential importance sampling — the sample is
//!   forced through every fact and weighted by how plausible that forcing
//!   was);
//! - facts about times *later* than the query time also condition the answer
//!   (smoothing): the simulation horizon extends to the latest in-scope fact,
//!   so "alive at t=30" correctly forbids "dead at t=10" in every history
//!   that survives with weight > 0.
//!
//! The answer to a distribution query is the weighted histogram of the target
//! aspect across samples. A **collapse** (`observe`) picks *one* surviving
//! history — weight-proportionally, seeded by `(world seed, ledger content
//! hash, target)` — reads the observed aspect off it, and appends the result
//! to the ledger as a new committed fact.
//!
//! **Bounded collapse.** The scope of a query is the BFS ball of regions
//! within graph distance `depth` of the subject's anchor region; only agents
//! whose tick-0 home lies in the ball are simulated. Where the ball's edge
//! regions have out-of-scope neighbours (the *frontier*), those neighbours'
//! pressure is synthesised per (sample, region, tick) from the unconditioned
//! prior instead of being simulated — observing one mind must not collapse
//! the planet. Committed facts about out-of-scope subjects are ignored; that
//! is the price of the bound, and the distortion it causes is measured in
//! `tests/s2_measurements.rs`.

use std::collections::{BTreeMap, BTreeSet};

use super::ledger::{AppendOutcome, Aspect, Fact, Ledger, LedgerError, Subject, Value};
use super::rng::{Pcg32, draw_f64, mix};
use super::world::{
    AgentId, AgentState, RegionId, SALT_AGENT_STEP, SALT_COLLAPSE, SALT_REGION_STEP, Tick, ToyWorld,
};

/// Tuning knobs for a query/collapse.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    /// Number of sampled histories per query.
    pub k_samples: u32,
    /// Collapse bound: regions within this graph distance of the anchor are
    /// simulated; `None` simulates the whole world (exact model, no frontier).
    pub depth: Option<u32>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            k_samples: 256,
            depth: Some(3),
        }
    }
}

/// Instrumentation for cascade bounding: exactly what a query touched.
#[derive(Debug, Clone)]
pub struct Report {
    /// Regions actually simulated (== the depth-N ball around the anchor).
    pub touched_regions: BTreeSet<RegionId>,
    /// Agents actually simulated (tick-0 home inside the ball).
    pub touched_agents: BTreeSet<AgentId>,
    /// Out-of-scope regions consulted only for frontier prior synthesis.
    pub frontier_regions: BTreeSet<RegionId>,
    /// Last tick simulated (>= query time if later facts forced smoothing).
    pub horizon: Tick,
    /// Sum of importance weights across samples (0 => no consistent history found).
    pub total_weight: f64,
    /// Effective sample size (1 / sum of squared normalised weights).
    pub effective_samples: f64,
}

/// A discrete distribution over observed values (normalised).
#[derive(Debug, Clone, PartialEq)]
pub struct Distribution {
    probs: BTreeMap<Value, f64>,
}

impl Distribution {
    fn from_weighted(pairs: impl IntoIterator<Item = (Value, f64)>) -> Self {
        let mut probs: BTreeMap<Value, f64> = BTreeMap::new();
        let mut total = 0.0;
        for (v, w) in pairs {
            if w > 0.0 {
                *probs.entry(v).or_insert(0.0) += w;
                total += w;
            }
        }
        if total > 0.0 {
            for p in probs.values_mut() {
                *p /= total;
            }
        }
        Self { probs }
    }

    pub fn prob(&self, v: &Value) -> f64 {
        self.probs.get(v).copied().unwrap_or(0.0)
    }

    pub fn support(&self) -> impl Iterator<Item = (&Value, f64)> {
        self.probs.iter().map(|(v, &p)| (v, p))
    }

    /// True when no consistent history was found (dead ensemble).
    pub fn is_empty(&self) -> bool {
        self.probs.is_empty()
    }

    /// Total variation distance: `0.5 * sum |p - q|` over the union support.
    pub fn tv_distance(&self, other: &Distribution) -> f64 {
        let keys: BTreeSet<&Value> = self.probs.keys().chain(other.probs.keys()).collect();
        0.5 * keys
            .into_iter()
            .map(|k| (self.prob(k) - other.prob(k)).abs())
            .sum::<f64>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObserveError {
    #[error(
        "no history consistent with the ledger and {subject:?} {aspect:?} at t={time} (searched {k_samples} samples)"
    )]
    Contradiction {
        subject: Subject,
        aspect: Aspect,
        time: Tick,
        k_samples: u32,
    },
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// The anchor region a query's scope is centred on.
fn anchor(world: &ToyWorld, subject: Subject) -> RegionId {
    match subject {
        // Anchoring on the *tick-0* home keeps scope membership a pure
        // function of the query, independent of any sampled trajectory.
        // Limitation (documented in S2-results.md): agents that wander far
        // from home are simulated with frontier priors for their surroundings.
        Subject::Agent(a) => world.agent_home(a),
        Subject::Region(r) => r,
        // Worldgen-history subjects (S7) are ledger-only: they condition
        // nothing dynamically and cannot be query targets.
        Subject::Site(_) | Subject::Polity(_) => {
            unreachable!("worldgen subjects are not simulated by this engine")
        }
    }
}

struct Scope {
    regions: BTreeSet<RegionId>,
    frontier: BTreeSet<RegionId>,
    agents: Vec<AgentId>,
    /// Facts about in-scope agents, keyed by (agent, tick).
    agent_facts: BTreeMap<(AgentId, Tick), Vec<Fact>>,
    /// Facts about in-scope regions, keyed by (region, tick).
    region_facts: BTreeMap<(RegionId, Tick), Vec<Fact>>,
    horizon: Tick,
}

fn build_scope(
    world: &ToyWorld,
    ledger: &Ledger,
    subject: Subject,
    time: Tick,
    depth: Option<u32>,
) -> Scope {
    let regions = world.ball(anchor(world, subject), depth);
    let frontier: BTreeSet<RegionId> = regions
        .iter()
        .flat_map(|&r| world.neighbors(r).iter().copied())
        .filter(|n| !regions.contains(n))
        .collect();
    let agents: Vec<AgentId> = (0..world.num_agents())
        .filter(|&a| regions.contains(&world.agent_home(a)))
        .collect();
    let agent_set: BTreeSet<AgentId> = agents.iter().copied().collect();

    let mut agent_facts: BTreeMap<(AgentId, Tick), Vec<Fact>> = BTreeMap::new();
    let mut region_facts: BTreeMap<(RegionId, Tick), Vec<Fact>> = BTreeMap::new();
    let mut horizon = time;
    for f in ledger.facts() {
        match f.subject {
            Subject::Agent(a) if agent_set.contains(&a) => {
                horizon = horizon.max(f.time);
                agent_facts.entry((a, f.time)).or_default().push(*f);
            }
            Subject::Region(r) if regions.contains(&r) => {
                horizon = horizon.max(f.time);
                region_facts.entry((r, f.time)).or_default().push(*f);
            }
            _ => {} // Out of scope: ignored under this bound.
        }
    }
    Scope {
        regions,
        frontier,
        agents,
        agent_facts,
        region_facts,
        horizon,
    }
}

/// Sample from a discrete distribution (assumed positive total mass) using a
/// single uniform draw in [0, 1).
fn sample_from<T: Copy>(dist: &[(T, f64)], total: f64, u: f64) -> T {
    let mut acc = 0.0;
    let target = u * total;
    for &(v, p) in dist {
        acc += p;
        if target < acc {
            return v;
        }
    }
    dist.last().expect("non-empty distribution").0
}

/// Simulate one candidate history; returns the target value at `time` and the
/// history's importance weight (0 if it violates any structural constraint).
#[allow(clippy::too_many_arguments)]
fn simulate_sample(
    world: &ToyWorld,
    scope: &Scope,
    subject: Subject,
    time: Tick,
    aspect: Aspect,
    k: u64,
) -> (Option<Value>, f64) {
    let seed = world.seed;
    let mut weight = 1.0f64;

    // Deterministic tick-0 state.
    let mut levels: BTreeMap<RegionId, u8> = scope
        .regions
        .iter()
        .map(|&r| (r, world.initial_pressure(r)))
        .collect();
    let mut agents: Vec<(AgentId, AgentState)> = scope
        .agents
        .iter()
        .map(|&a| (a, world.initial_agent_state(a)))
        .collect();

    // Facts about tick 0 must match the deterministic initial condition.
    for (&(r, t), facts) in &scope.region_facts {
        if t == 0 && facts.iter().any(|f| !f.satisfied_by_pressure(levels[&r])) {
            return (None, 0.0);
        }
    }
    for &(a, ref s) in &agents {
        if let Some(facts) = scope.agent_facts.get(&(a, 0))
            && facts.iter().any(|f| !f.satisfied_by_agent(s))
        {
            return (None, 0.0);
        }
    }

    let mut recorded = None;
    let record = |t: Tick,
                  levels: &BTreeMap<RegionId, u8>,
                  agents: &[(AgentId, AgentState)],
                  out: &mut Option<Value>| {
        if t != time {
            return;
        }
        *out = Some(match subject {
            Subject::Region(r) => Value::Pressure(levels[&r]),
            Subject::Agent(a) => {
                let state = agents
                    .iter()
                    .find(|(id, _)| *id == a)
                    .map(|(_, s)| *s)
                    .expect("query subject must be in scope");
                match aspect {
                    Aspect::AgentRegion => Value::Region(state.region),
                    Aspect::AgentBehavior => Value::Behavior(state.behavior),
                    Aspect::AgentAlive => Value::Alive(state.is_alive()),
                    _ => unreachable!("non-agent aspect on agent subject"),
                }
            }
            Subject::Site(_) | Subject::Polity(_) => {
                unreachable!("worldgen subjects are not simulated by this engine")
            }
        });
    };
    record(0, &levels, &agents, &mut recorded);

    for t in 0..scope.horizon {
        // Pressure a consumer sees for a region at tick t: simulated if in
        // scope, otherwise synthesised from the unconditioned prior.
        let pressure_at = |r: RegionId, levels: &BTreeMap<RegionId, u8>| -> f64 {
            match levels.get(&r) {
                Some(&l) => f64::from(l),
                None => f64::from(world.frontier_pressure(k, r, t)),
            }
        };

        // 1. Region pressure step (all regions advance off the tick-t field).
        let mut next_levels = BTreeMap::new();
        for (&r, &level) in &levels {
            let neigh = world.neighbors(r);
            let avg =
                neigh.iter().map(|&n| pressure_at(n, &levels)).sum::<f64>() / neigh.len() as f64;
            let mut dist = world.pressure_transition(r, level, avg);
            if let Some(facts) = scope.region_facts.get(&(r, t + 1)) {
                dist.retain(|&(l, _)| facts.iter().all(|f| f.satisfied_by_pressure(l)));
                let mass: f64 = dist.iter().map(|(_, p)| p).sum();
                if mass <= 0.0 {
                    return (None, 0.0);
                }
                weight *= mass;
            }
            let total: f64 = dist.iter().map(|(_, p)| p).sum();
            let u = draw_f64(&[seed, k, SALT_REGION_STEP, u64::from(r), u64::from(t)]);
            next_levels.insert(r, sample_from(&dist, total, u));
        }

        // 2. Agent step, reacting to tick-t pressure of their current region.
        for (a, state) in &mut agents {
            let local = pressure_at(state.region, &levels).round() as u8;
            let mut dist = world.agent_transition(state, local);
            if let Some(facts) = scope.agent_facts.get(&(*a, t + 1)) {
                dist.retain(|(s, _)| facts.iter().all(|f| f.satisfied_by_agent(s)));
                let mass: f64 = dist.iter().map(|(_, p)| p).sum();
                if mass <= 0.0 {
                    return (None, 0.0);
                }
                weight *= mass;
            }
            let total: f64 = dist.iter().map(|(_, p)| p).sum();
            let u = draw_f64(&[seed, k, SALT_AGENT_STEP, u64::from(*a), u64::from(t)]);
            *state = sample_from(&dist, total, u);
        }

        levels = next_levels;
        record(t + 1, &levels, &agents, &mut recorded);
    }

    (recorded, weight)
}

fn run_ensemble(
    world: &ToyWorld,
    ledger: &Ledger,
    subject: Subject,
    time: Tick,
    aspect: Aspect,
    params: Params,
) -> (Vec<(Value, f64)>, Report) {
    assert!(
        matches!(
            (subject, aspect),
            (Subject::Agent(_), Aspect::AgentRegion)
                | (Subject::Agent(_), Aspect::AgentBehavior)
                | (Subject::Agent(_), Aspect::AgentAlive)
                | (Subject::Region(_), Aspect::RegionPressure)
        ),
        "aspect {aspect:?} does not apply to subject {subject:?}"
    );
    let scope = build_scope(world, ledger, subject, time, params.depth);
    let mut outcomes = Vec::with_capacity(params.k_samples as usize);
    let mut total_weight = 0.0;
    let mut sq_norm = 0.0;
    for k in 0..u64::from(params.k_samples) {
        let (value, weight) = simulate_sample(world, &scope, subject, time, aspect, k);
        if let Some(v) = value
            && weight > 0.0
        {
            outcomes.push((v, weight));
            total_weight += weight;
        }
    }
    if total_weight > 0.0 {
        sq_norm = outcomes
            .iter()
            .map(|(_, w)| (w / total_weight).powi(2))
            .sum::<f64>();
    }
    let report = Report {
        touched_regions: scope.regions.clone(),
        touched_agents: scope.agents.iter().copied().collect(),
        frontier_regions: scope.frontier.clone(),
        horizon: scope.horizon,
        total_weight,
        effective_samples: if sq_norm > 0.0 { 1.0 / sq_norm } else { 0.0 },
    };
    (outcomes, report)
}

/// Distribution query: what states could `subject` be in at `time`, with what
/// probabilities, given everything committed in the ledger?
pub fn query(
    world: &ToyWorld,
    ledger: &Ledger,
    subject: Subject,
    time: Tick,
    aspect: Aspect,
    params: Params,
) -> (Distribution, Report) {
    let (outcomes, report) = run_ensemble(world, ledger, subject, time, aspect, params);
    (Distribution::from_weighted(outcomes), report)
}

/// Observe: collapse the given aspect to one concrete value consistent with
/// every committed fact, and commit the result to the ledger.
///
/// Deterministic given (world seed, ledger content, target, params). If the
/// exact aspect is already committed, returns the committed value without
/// touching anything (idempotent re-observation).
pub fn observe(
    world: &ToyWorld,
    ledger: &mut Ledger,
    subject: Subject,
    time: Tick,
    aspect: Aspect,
    params: Params,
) -> Result<(Value, Report), ObserveError> {
    if let Some(fact) = ledger.get(subject, time, aspect) {
        let value = fact.value;
        return Ok((
            value,
            Report {
                touched_regions: BTreeSet::new(),
                touched_agents: BTreeSet::new(),
                frontier_regions: BTreeSet::new(),
                horizon: time,
                total_weight: 1.0,
                effective_samples: f64::from(params.k_samples),
            },
        ));
    }
    let (outcomes, report) = run_ensemble(world, ledger, subject, time, aspect, params);
    if report.total_weight <= 0.0 {
        return Err(ObserveError::Contradiction {
            subject,
            aspect,
            time,
            k_samples: params.k_samples,
        });
    }
    // Weight-proportional pick of one surviving history, seeded by the ledger
    // content so replays collapse identically.
    let collapse_seed = mix(&[
        world.seed,
        SALT_COLLAPSE,
        ledger.content_hash(),
        subject.key(),
        u64::from(time),
        aspect.key(),
    ]);
    let u = Pcg32::new(collapse_seed, SALT_COLLAPSE).next_f64();
    let value = sample_from(&outcomes, report.total_weight, u);
    ledger.append(time, subject, aspect, value)?;
    Ok((value, report))
}

/// Commit an externally scripted/forced fact.
///
/// Policy: **reject, don't re-weight.** A fact inconsistent with the ledger
/// (structurally, or with no surviving history among `k_samples` candidates)
/// is refused and the ledger is left untouched. See S2-results.md for why.
pub fn force_fact(
    world: &ToyWorld,
    ledger: &mut Ledger,
    subject: Subject,
    time: Tick,
    aspect: Aspect,
    value: Value,
    params: Params,
) -> Result<Report, ObserveError> {
    // Dry-run the append on a scratch copy: catches aspect/value mismatches,
    // exact-key conflicts, and cross-aspect structural contradictions.
    let mut trial = ledger.clone();
    let outcome = trial.append(time, subject, aspect, value)?;
    if let AppendOutcome::AlreadyCommitted(_) = outcome {
        return Ok(Report {
            touched_regions: BTreeSet::new(),
            touched_agents: BTreeSet::new(),
            frontier_regions: BTreeSet::new(),
            horizon: time,
            total_weight: 1.0,
            effective_samples: f64::from(params.k_samples),
        });
    }
    // Dynamic consistency: does at least one sampled history satisfy the
    // ledger *plus* the forced fact?
    let (_, report) = run_ensemble(world, &trial, subject, time, aspect, params);
    if report.total_weight <= 0.0 {
        return Err(ObserveError::Contradiction {
            subject,
            aspect,
            time,
            k_samples: params.k_samples,
        });
    }
    ledger
        .append(time, subject, aspect, value)
        .expect("trial append succeeded");
    Ok(report)
}
