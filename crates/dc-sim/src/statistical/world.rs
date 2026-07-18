//! The S2 toy world: a small region graph plus ~100 NPC agents, and the
//! *transition model* — the probability distributions that define how fluid
//! state evolves from one tick to the next.
//!
//! Nothing in here stores evolving state. The world owns only immutable
//! structure (seed, graph, agent homes) and pure functions returning
//! *distributions over next states*. The engine samples from these; the same
//! functions could in principle be fed to an exact inference pass.

use serde::{Deserialize, Serialize};

use super::rng::{draw_f64, mix};

pub type RegionId = u8;
pub type AgentId = u16;
/// Discrete sim time. Tick 0 is the deterministic initial condition.
pub type Tick = u32;

pub const NUM_REGIONS: u8 = 20;
pub const NUM_AGENTS: u16 = 100;
/// Hostile-mob pressure per region: 0 = calm, 1 = tense, 2 = raided.
pub const MAX_PRESSURE: u8 = 2;

// Salts separating the addressed-draw key spaces (see rng.rs).
pub(crate) const SALT_HOME: u64 = 0x01;
pub(crate) const SALT_DANGER: u64 = 0x02;
pub(crate) const SALT_REGION_STEP: u64 = 0x03;
pub(crate) const SALT_AGENT_STEP: u64 = 0x04;
pub(crate) const SALT_FRONTIER_PRIOR: u64 = 0x05;
pub(crate) const SALT_COLLAPSE: u64 = 0x06;

/// What an agent is doing this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Behavior {
    Farming,
    /// Trading in a neighbouring region (located there this tick, home kept).
    Trading(RegionId),
    Fortifying,
    /// Fled to a neighbouring region (relocates: home becomes the target).
    Fled(RegionId),
    /// Absorbing.
    Dead,
}

impl Behavior {
    /// Stable numeric encoding for hashing (ledger content hash, collapse seeds).
    pub(crate) fn key(self) -> u64 {
        match self {
            Behavior::Farming => 0x100,
            Behavior::Trading(r) => 0x200 | u64::from(r),
            Behavior::Fortifying => 0x300,
            Behavior::Fled(r) => 0x400 | u64::from(r),
            Behavior::Dead => 0x500,
        }
    }
}

/// Full fluid state of one agent at one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentState {
    /// Region the agent considers home (changes only by fleeing).
    pub home: RegionId,
    /// Region the agent is physically in this tick.
    pub region: RegionId,
    pub behavior: Behavior,
}

impl AgentState {
    pub fn is_alive(&self) -> bool {
        self.behavior != Behavior::Dead
    }
}

/// Immutable world structure + transition model. Everything derives from `seed`.
#[derive(Debug, Clone)]
pub struct ToyWorld {
    pub seed: u64,
    adjacency: Vec<Vec<RegionId>>,
    agent_home: Vec<RegionId>,
}

impl ToyWorld {
    pub fn new(seed: u64) -> Self {
        // A ring of NUM_REGIONS with a few chords: arbitrary-ish adjacency
        // with interesting graph distances (diameter ~5), fixed topology so
        // tests can reason about distances.
        let n = NUM_REGIONS as usize;
        let mut adjacency: Vec<Vec<RegionId>> = vec![Vec::new(); n];
        let add = |adj: &mut Vec<Vec<RegionId>>, a: usize, b: usize| {
            adj[a].push(b as RegionId);
            adj[b].push(a as RegionId);
        };
        for i in 0..n {
            add(&mut adjacency, i, (i + 1) % n);
        }
        for (a, b) in [(0, 10), (5, 15), (3, 8), (12, 17), (2, 18)] {
            add(&mut adjacency, a, b);
        }
        for neigh in &mut adjacency {
            neigh.sort_unstable();
        }
        let agent_home = (0..NUM_AGENTS)
            .map(|a| (mix(&[seed, SALT_HOME, u64::from(a)]) % u64::from(NUM_REGIONS)) as RegionId)
            .collect();
        Self {
            seed,
            adjacency,
            agent_home,
        }
    }

    pub fn neighbors(&self, r: RegionId) -> &[RegionId] {
        &self.adjacency[r as usize]
    }

    /// Initial (tick-0) home of an agent; also its scope anchor.
    pub fn agent_home(&self, a: AgentId) -> RegionId {
        self.agent_home[a as usize]
    }

    /// Regions within graph distance `depth` of `center` (`None` = all).
    pub fn ball(
        &self,
        center: RegionId,
        depth: Option<u32>,
    ) -> std::collections::BTreeSet<RegionId> {
        let mut dist = vec![u32::MAX; NUM_REGIONS as usize];
        let mut queue = std::collections::VecDeque::new();
        dist[center as usize] = 0;
        queue.push_back(center);
        while let Some(r) = queue.pop_front() {
            for &n in self.neighbors(r) {
                if dist[n as usize] == u32::MAX {
                    dist[n as usize] = dist[r as usize] + 1;
                    queue.push_back(n);
                }
            }
        }
        let limit = depth.unwrap_or(u32::MAX);
        (0..NUM_REGIONS)
            .filter(|&r| dist[r as usize] <= limit)
            .collect()
    }

    /// Graph diameter (max shortest-path distance).
    pub fn diameter(&self) -> u32 {
        (0..NUM_REGIONS)
            .map(|c| {
                let ball = self.ball(c, None);
                debug_assert_eq!(ball.len(), NUM_REGIONS as usize);
                // Re-run BFS capturing max distance.
                (0..=u32::from(NUM_REGIONS))
                    .find(|&d| self.ball(c, Some(d)).len() == NUM_REGIONS as usize)
                    .unwrap()
            })
            .max()
            .unwrap()
    }

    /// Per-region hostile-mob affinity in `[0, 0.5]`, fixed by the seed.
    pub fn base_danger(&self, r: RegionId) -> f64 {
        (mix(&[self.seed, SALT_DANGER, u64::from(r)]) % 1000) as f64 / 1000.0 * 0.5
    }

    /// Deterministic tick-0 pressure.
    pub fn initial_pressure(&self, r: RegionId) -> u8 {
        let d = self.base_danger(r);
        if d < 0.20 {
            0
        } else if d < 0.35 {
            1
        } else {
            2
        }
    }

    /// Deterministic tick-0 agent state: farming at home.
    pub fn initial_agent_state(&self, a: AgentId) -> AgentState {
        let home = self.agent_home(a);
        AgentState {
            home,
            region: home,
            behavior: Behavior::Farming,
        }
    }

    /// Distribution over next pressure level for `region`, given its current
    /// level and the average level of its neighbours. All reachable outcomes
    /// have strictly positive probability (structural zeros only at the level
    /// bounds) — this "full one-step support" property is what makes bounded
    /// collapse safe: changing the scope changes *weights*, never the support.
    pub fn pressure_transition(
        &self,
        region: RegionId,
        level: u8,
        neighbor_avg: f64,
    ) -> Vec<(u8, f64)> {
        let d = self.base_danger(region);
        let l = f64::from(level);
        let mut p_up = (0.04 + 0.45 * d + 0.20 * (neighbor_avg / f64::from(MAX_PRESSURE))
            - 0.10 * l)
            .clamp(0.02, 0.9);
        let mut p_down = (0.30 - 0.25 * d + 0.08 * l).clamp(0.02, 0.9);
        if level == MAX_PRESSURE {
            p_up = 0.0;
        }
        if level == 0 {
            p_down = 0.0;
        }
        let stay = 1.0 - p_up - p_down;
        debug_assert!(stay > 0.0);
        let mut out = Vec::with_capacity(3);
        if p_down > 0.0 {
            out.push((level - 1, p_down));
        }
        out.push((level, stay));
        if p_up > 0.0 {
            out.push((level + 1, p_up));
        }
        out
    }

    /// Unconditioned marginal over pressure levels for `region`, used to
    /// synthesise boundary conditions at the collapse frontier. In the real
    /// system this would come from the statistical tier's cached summaries;
    /// here it is a closed-form stand-in parameterised by the region's danger.
    pub fn prior_pressure_weights(&self, region: RegionId) -> [f64; 3] {
        let d = self.base_danger(region);
        let w0 = 0.60 - 0.50 * d;
        let w2 = 0.15 + 0.70 * d;
        let w1 = 1.0 - w0 - w2;
        [w0, w1, w2]
    }

    /// A frontier-synthesised pressure level for an out-of-scope region:
    /// drawn from the unconditioned prior, addressed by (sample, region, tick)
    /// so every consumer within one sample sees the same synthesised value.
    pub(crate) fn frontier_pressure(&self, sample: u64, region: RegionId, t: Tick) -> u8 {
        let w = self.prior_pressure_weights(region);
        let r = draw_f64(&[
            self.seed,
            SALT_FRONTIER_PRIOR,
            sample,
            u64::from(region),
            u64::from(t),
        ]) * (w[0] + w[1] + w[2]);
        if r < w[0] {
            0
        } else if r < w[0] + w[1] {
            1
        } else {
            2
        }
    }

    /// Distribution over next agent states, given the pressure of the region
    /// the agent currently occupies. Death is always possible (so a committed
    /// "dead" fact is never structurally impossible going forward) and Dead is
    /// absorbing (the only structural zero in the agent model).
    pub fn agent_transition(&self, state: &AgentState, pressure: u8) -> Vec<(AgentState, f64)> {
        if state.behavior == Behavior::Dead {
            return vec![(*state, 1.0)];
        }
        let p_die = [0.002, 0.012, 0.06][pressure as usize];
        let (w_farm, w_trade, w_fort, w_fled) = match pressure {
            0 => (0.55, 0.30, 0.10, 0.05),
            1 => (0.30, 0.15, 0.40, 0.15),
            _ => (0.05, 0.05, 0.45, 0.45),
        };
        let live = 1.0 - p_die;
        let mut out = Vec::new();
        out.push((
            AgentState {
                behavior: Behavior::Dead,
                ..*state
            },
            p_die,
        ));
        out.push((
            AgentState {
                home: state.home,
                region: state.home,
                behavior: Behavior::Farming,
            },
            live * w_farm,
        ));
        out.push((
            AgentState {
                home: state.home,
                region: state.home,
                behavior: Behavior::Fortifying,
            },
            live * w_fort,
        ));
        let trade_dests = self.neighbors(state.home);
        for &n in trade_dests {
            out.push((
                AgentState {
                    home: state.home,
                    region: n,
                    behavior: Behavior::Trading(n),
                },
                live * w_trade / trade_dests.len() as f64,
            ));
        }
        let flee_dests = self.neighbors(state.region);
        for &n in flee_dests {
            out.push((
                AgentState {
                    home: n,
                    region: n,
                    behavior: Behavior::Fled(n),
                },
                live * w_fled / flee_dests.len() as f64,
            ));
        }
        debug_assert!((out.iter().map(|(_, p)| p).sum::<f64>() - 1.0).abs() < 1e-9);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_structure_is_seed_deterministic() {
        let a = ToyWorld::new(7);
        let b = ToyWorld::new(7);
        assert_eq!(a.agent_home, b.agent_home);
        assert_eq!(a.adjacency, b.adjacency);
        assert!(a.diameter() >= 3, "graph should have non-trivial distances");
    }

    #[test]
    fn transition_distributions_are_normalised() {
        let w = ToyWorld::new(99);
        for r in 0..NUM_REGIONS {
            for level in 0..=MAX_PRESSURE {
                let d = w.pressure_transition(r, level, 1.0);
                let total: f64 = d.iter().map(|(_, p)| p).sum();
                assert!((total - 1.0).abs() < 1e-12);
            }
        }
        let s = w.initial_agent_state(0);
        for pressure in 0..=MAX_PRESSURE {
            let d = w.agent_transition(&s, pressure);
            let total: f64 = d.iter().map(|(_, p)| p).sum();
            assert!((total - 1.0).abs() < 1e-9);
        }
    }
}
