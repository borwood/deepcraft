//! S2 spike: the statistical tier prototype — superposed NPC state, the
//! constraint ledger, and bounded collapse.
//!
//! This module is the headless toy implementation built to answer S2's design
//! questions (see docs/SPIKES.md § S2 and docs/spikes/S2-results.md for the
//! conclusions). The *shapes* here — `Ledger` of committed `Fact`s, fluid
//! state as a pure function of `(seed, time, ledger)`, depth-bounded collapse
//! with frontier synthesis — are the deliverable; the toy world itself is
//! disposable.
//!
//! Entry points:
//! - [`world::ToyWorld`]: region graph + agents + transition model.
//! - [`ledger::Ledger`]: append-only committed facts.
//! - [`engine::query`]: distribution over possible states at a time.
//! - [`engine::observe`]: collapse one aspect and commit it.
//! - [`engine::force_fact`]: commit a scripted fact (rejected on contradiction).

pub mod engine;
pub mod ledger;
pub mod rng;
pub mod world;

pub use engine::{Distribution, ObserveError, Params, Report, force_fact, observe, query};
pub use ledger::{AppendOutcome, Aspect, Fact, Ledger, LedgerError, Subject, Value};
pub use world::{AgentId, AgentState, Behavior, NUM_AGENTS, NUM_REGIONS, RegionId, Tick, ToyWorld};
