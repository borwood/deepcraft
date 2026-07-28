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
//! **Nothing in production calls this module's engine** (2026-07-28,
//! journal/0121). Its one caller — dc-worldgen's bootstrap settlement-history
//! pass — was removed as unratified content, and the S2 *shapes* named above are
//! instanced elsewhere in the tree (spines § S-2, § S-9) rather than here. The
//! sibling [`rng`] module is a different matter entirely: it is the workspace's
//! draw provider and is on every worldgen path.
//!
//! # KEPT DELIBERATELY — read this before proposing to delete it, or to build on it
//!
//! **User ruling, 2026-07-28.** Zero callers is *not* a backlog item here and this
//! module is *not* awaiting disposal. *"The statistical system is genuinely
//! intended, though I can't say whether as-is it will fit the desired shape when we
//! actually do move on to implementing the civ/socia part of the default pack and
//! the engine affordances."* Both the primitive **and** the toy stay.
//!
//! Four things a reader who lands here needs, in order:
//!
//! 1. **The code that read this is gone, and that says nothing about the shapes.**
//!    The removal disposed of unratified *content*; it was not a verdict on the
//!    ledger design. Do not read the empty consumer set as a defect.
//! 2. **The primitive is the deliverable** — the append-only ledger of committed
//!    facts, state as a pure function of `(seed, time, ledger)`, depth-bounded
//!    collapse with frontier synthesis, and above all **addressed rather than
//!    streamed randomness** (`docs/spikes/S2-results.md`).
//! 3. **It is a CANDIDATE, not a commitment.** It is meant to be **visited again and
//!    checked against real requirements** when the default pack turns toward
//!    eco / socia / civ concepts — not adopted on sight because it is here and it
//!    compiles. Whether this shape fits cannot be answered until those requirements
//!    exist. *(Existence is not standing — in both directions: it is no argument for
//!    deletion, and no argument for adoption.)*
//! 4. **When that thread may open is a USER CALL, and there is no checkable gate.**
//!    Engine plus all non-bio earth science first, in the ratified SDK-plugin shape
//!    → **then ecology** → **then social concepts**. *"Much work and reflection will
//!    be done before the USER decides it is time."* See
//!    `docs/design/worldgen.md` § *Sequencing*. Do not infer the gate is met.
//!
//! The settlement/civ vocabulary in [`ledger`] (`Subject::{Site, Polity}`,
//! `Aspect::{SiteExists, ..}`, `SiteEventKind`, `Value::{Exists, ..}`) is
//! **producer-less example vocabulary and NOT a schema to build on** — kept by the
//! same ruling because deleting variants of a `Serialize` enum was wider than the
//! removal's scope, not because it is a design anyone ratified.
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
pub use ledger::{AppendOutcome, Aspect, Fact, Ledger, LedgerError, SiteEventKind, Subject, Value};
pub use world::{AgentId, AgentState, Behavior, NUM_AGENTS, NUM_REGIONS, RegionId, Tick, ToyWorld};
