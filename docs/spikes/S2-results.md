# S2 results — constraint ledger prototype

Status: spike complete, 2026-07-18. Code lives in `crates/dc-sim/src/statistical/`
(toy world, disposable; the *shapes* below are the deliverable), torture tests in
`crates/dc-sim/tests/s2_torture.rs`, measurements in
`crates/dc-sim/tests/s2_measurements.rs`.

## What was built

A headless statistical-tier prototype: 20 regions (ring + 5 chords, diameter 6),
100 agents, behavior space `{Farming, Trading(dest), Fortifying, Fled(dest),
Dead}`, per-region hostile-mob pressure `{0,1,2}` with neighbor diffusion.
Fluid state is **derived, never stored**: agent/region state at tick T is a pure
function of `(world seed, T, ledger)`. All randomness is hand-rolled (splitmix64
finalizer + PCG32, reference-vector-tested); no ambient entropy anywhere.

The single most load-bearing implementation choice: **randomness is addressed,
not streamed**. Every stochastic decision draws `u = hash(world_seed, sample_k,
salt, entity_id, tick)`. Bounded collapse re-simulates different *subsets* of
the world per query; with a sequential PRNG stream, scope changes would
scramble which entity gets which number. With addressed draws, changing the
depth bound changes *only* boundary conditions — which also gives
common-random-numbers variance reduction for free when measuring distortion.

## Ledger schema (as designed)

```rust
Fact {
    seq:     u64,      // append index, never reused; ledger is append-only
    time:    Tick,     // the sim time the fact is ABOUT (not commit time)
    subject: Subject,  // Agent(id) | Region(id)
    aspect:  Aspect,   // AgentRegion | AgentBehavior | AgentAlive | RegionPressure
    value:   Value,    // one concrete value, typed per aspect
}
```

Decisions baked into the schema, all validated by tests:

- **Fact time ≠ commit time.** A traveler's report commits facts about the
  past. `time` indexes into sim history; `seq` only orders commits.
- **Aspects enable partial observation.** `AgentAlive` commits strictly less
  than `AgentBehavior`; later observations *narrow* earlier ones (monotone
  test). This is the hook for rumor-grade information without a separate
  uncertainty mechanism.
- **The ledger validates structure only** (aspect/value pairing, exact-key
  conflicts, the Alive↔Behavior implication at one subject-time). Whether a
  fact is *dynamically* reachable is the engine's job. Keeping the ledger dumb
  keeps it auditable and serializable (all types derive serde).
- **Content hash** (order-sensitive fold over facts) seeds collapses: same
  seed + same ledger content ⇒ bit-identical collapse. Determinism needs no
  hidden state.

## Collapse algorithm

Query = "distribution over `(subject, time, aspect)`"; observe = query + pick
one + commit. Both run the same sampler:

1. **Scope**: BFS ball of regions within graph distance N of the subject's
   anchor (agent's tick-0 home, or the region itself). Only agents homed in
   the ball are simulated. Facts about out-of-scope subjects are ignored
   (measured cost below).
2. **Horizon**: max(query time, latest in-scope fact time). Facts *later* than
   the query time condition the answer (smoothing): "alive at 18" forbids
   "dead at 6" — asserted by test.
3. **K samples**, each a full trajectory from the deterministic tick-0 state:
   at each tick, each entity's one-step transition distribution is
   **conditioned** on the ledger — candidate outcomes violating any committed
   fact about that entity at that tick are filtered out, the sample's weight is
   multiplied by the retained probability mass, and the next state is drawn
   from the renormalized remainder (sequential importance sampling: samples
   are *forced through* every fact and weighted by the plausibility of the
   forcing). An empty filtered distribution kills the sample (weight 0).
4. **Query** returns the weighted histogram of the target aspect. **Observe**
   picks one surviving trajectory weight-proportionally, seeded by
   `hash(world_seed, ledger.content_hash(), subject, time, aspect)`, reads the
   aspect off it, appends the fact. Re-observing a committed aspect is an
   idempotent read.

Consistency is then structural, not enforced after the fact: every committed
fact was sampled from a conditioned ensemble, so it lies in the support of the
dynamics; every later derivation conditions on it and reproduces it with
probability exactly 1 (asserted to 1e-12 across observation order, query
depth, and anchor choice).

## Frontier synthesis

At the ball's edge, in-scope regions have out-of-scope neighbors. Those
neighbors' pressure is *synthesized*: drawn from a closed-form unconditioned
marginal parameterized by the region's danger affinity, addressed by
`(sample, region, tick)` so every consumer inside one sample sees the same
synthesized value. Agents that wander out of scope keep simulating but read
synthesized pressure too. Nothing recurses — an observe at depth N provably
touches only the depth-N ball (instrumented: the report carries
`touched_regions`, `touched_agents`, `frontier_regions`; the cascade test
asserts equality with the ball and that frontier regions sit just outside).

In the real system the frontier prior must come from the statistical tier's
cached summaries (per-column/per-region aggregates), not a formula.

## Sampling vs exact inference

Sampling was chosen deliberately. Exact inference over this model is a joint
DBN over ~120 coupled variables per tick (agents couple through the shared
pressure field); exact marginals are intractable beyond toy sizes, while
sampling is anytime, handles arbitrary aspects and partial facts with a
satisfaction predicate, and — critically — the *support* of the conditioned
distribution is what consistency relies on, and forced-transition sampling
preserves support exactly. Costs accepted: probabilities carry sampling noise
(deterministic noise, given the seeding), and two real failure modes below
(weight degeneracy, finite-K false contradiction). A plausible hybrid for
production: exact per-region pressure marginals (3-state chain) + sampled
agents.

## Contradiction policy: reject, don't re-weight

`force_fact` (scripted/external facts) refuses any fact that is structurally
inconsistent (exact-key conflict, Alive↔Behavior clash) or dynamically
unreachable (zero surviving samples with the fact added), and leaves the
ledger untouched. Rationale:

- The ledger's one invariant is immutability of committed facts. Re-weighting
  ("accept and make the past bend") silently changes what previously committed
  facts *mean* — the monotone-ledger property would hold syntactically and be
  violated semantically.
- Rejection surfaces the bug at the source (a bad script, a bad quest writer)
  instead of corrupting history forever.
- The narrative use cases that seem to want re-weighting (lies, unreliable
  narrators, propaganda) are better modeled *above* the ledger: commit
  "B **claimed** X" as a fact about B, not X. The ledger stores ground truth
  leaks only.

Tested: all three rejection classes bounce with the ledger byte-identical.

## Measured distortion from bounded depth

TV distance vs the unbounded run; agent 0 (anchor region 15), behavior at
t=32, K=512, seed `0xD5EED52026`. Diameter of the toy graph is 6; agent 0's
eccentricity is 5, so depth ≥ 5 is exact by construction (asserted TV = 0).

| depth | empty ledger | region 3 (distance 3) raided at t=8,10,12 |
|---|---|---|
| 1 | 0.098 | 0.367 |
| 2 | 0.065 | 0.334 |
| 3 | 0.029 | **0.273** |
| 4 | 0.016 | 0.032 |
| 5 | 0.000 | 0.000 |

Findings:

1. **Unconditioned frontier synthesis is cheap.** With no facts, depth 3 is
   within ~3% TV of exact and it decays smoothly. The bound works.
2. **Out-of-scope facts are silently ignored, and it shows.** With a strong
   committed fact 3 hops away, depth ≤ 2 distorts by ~0.33–0.37 TV.
   Consistency is *not* violated — the fact re-engages whenever it is in
   scope — but shallow queries are visibly wrong about the world's mood.
3. **The surprise: a fact exactly on the frontier is honored but still badly
   distorted** (depth 3 sees and enforces the raid, yet TV is 0.273, barely
   better than not seeing it). The raid is forced, but everything *around* it
   is synthesized from calm unconditioned priors, so its knock-on effects
   (neighbor pressure diffusion → agent flight) are suppressed. Distortion
   only collapses once the bound exceeds the fact's distance by ≥ 1.
   **Design rule: the collapse bound must cover the farthest relevant fact
   plus its influence shell — or scope must be fact-aware** (expand the ball
   to include committed facts' neighborhoods).

## Performance sanity

Dev profile (own code opt-level 1, deps 3), Windows dev box, T=48, K=1024:

- query, depth 3 (12 regions, 60 agents): **~0.7 s**
- query, unbounded (20 regions, 100 agents): **~1.2 s**
- observe, depth 3: **~0.7 s** (a collapse ≈ a query + O(K) selection)

≈ 200 ns per agent·tick·sample, scaling linearly in K × T × |scope|. K=256
(plenty for gameplay-grade distributions) puts a depth-3 query under ~200 ms.
Fine for a spike; a shipping far-sim needs derivation caching — which the
design already permits, since a query is a pure function of
`(seed, ledger content hash, target, params)` and is therefore cacheable by
key with no invalidation logic beyond "ledger grew".

## Failure modes discovered

- **Weight degeneracy / dead samples.** Conditioning is tick-local: a sample
  can wander into a state from which the next fact is one-step-unreachable and
  die (weight 0). Dense or improbable fact sets shrink the effective sample
  size; with finite K a *satisfiable* ledger can yield zero survivors — which
  `force_fact` would misreport as contradiction. Not hit in any test (ESS
  stayed ≈ K), but structural. Mitigations when it bites: particle resampling,
  a backward pass ("which states at t can still reach the facts?") used as a
  proposal, or fact-aware proposals.
- **Frontier-adjacent facts** (finding 3 above).
- **Horizon growth.** Smoothing extends every query's simulated horizon to the
  latest in-scope fact, and everything re-derives from t=0. Cost is O(horizon).
  Harmless at T≈50; fatal for deep time — see below.
- **Static anchor vs mobile agents.** Scope is anchored at the agent's tick-0
  home so that scope is a pure function of the query; an agent that migrates
  far from home is simulated against synthesized surroundings. Acceptable for
  a spike, wrong for long-lived wanderers.
- **Scope-dependent probabilities.** Two observers collapsing the same aspect
  under different depths get identical *consistency* but different
  *probabilities* (different fact subsets + frontiers), so collapse outcomes
  depend on the observation path. Arguably flavor, arguably a bug; flagged as
  an open question.

## Go/no-go: one system for live far-sim and deep-time worldgen (S7's question)

**Qualified GO.** Evidence from this spike:

- The machinery is already time-symmetric. The traveler's-report and smoothing
  tests are literally retro-dictive history queries: facts about the past,
  committed later, constraining both directions. "Deep-time history" is the
  same `query`/`observe` loop run over pre-player ticks, with worldgen
  committing facts (a ruin exists here; this dynasty fell) that the live sim
  must forever honor — exactly the ledger contract, no second system.
- Determinism from `(seed, ledger)` means CI can replay history, and worldgen
  output needs no storage beyond the ledger itself.

The GO carries four conditions, any of which failing would flip it:

1. **Checkpoint/summary facts.** Deriving from t=0 is O(history length);
   deep time needs committed coarse summaries (per-epoch region aggregates) so
   derivation restarts from the nearest checkpoint. This must be designed into
   the ledger (S7), not bolted on.
2. **Real frontier priors.** The closed-form stand-in must become the
   statistical tier's cached summaries; the interface (a marginal per
   region/aspect/time-bucket) is clear, the cache design is not.
3. **Fact-aware scope** (or bound ≥ fact distance + 1), per the distortion
   finding — otherwise deep-time facts near scope edges silently warp live
   queries.
4. **ESS monitoring.** Dense millennia-old ledgers must not starve the
   sampler; resampling or backward proposals needed before ledgers get dense.

## Open design questions

1. Are checkpoints committed facts (append-only, frozen forever — possibly
   *over*-freezing) or a separate derived cache keyed by ledger prefix hash?
2. Who chooses collapse depth — caller, a cost budget, or fact-aware
   auto-expansion? The distortion table says fixed small depths are unsafe
   near strong facts.
3. Aspect algebra: partial aspects worked (Alive ⊂ Behavior). Do we need
   set/interval facts ("pressure ≤ 1 all week", "somewhere in these three
   regions")? Satisfaction predicates generalize trivially; keeping the
   sampler efficient under them does not.
4. Continuous aspects (position, stockpiles): conditioning-by-filtering needs
   a tolerance/kernel story that discrete aspects dodge entirely.
5. Ledger scale: one global append-only log won't index well at millions of
   facts — shard by region? By epoch after checkpointing?
6. Is scope-dependent collapse probability (same aspect, different observer
   depth ⇒ different distribution) acceptable, or does the API need a
   canonical scope per subject?
