//! Tiered world simulation and the constraint ledger.
//!
//! Three tiers of fidelity, chosen by proximity/relevance to observers:
//! - **Full**: entity-level simulation (ECS), every tick.
//! - **Coarse**: agent-level simulation on slow ticks (NPC goals, settlement jobs).
//! - **Statistical**: superposed state — distributions over what *could* be true,
//!   evolved as a seeded pure function of (seed, time, committed constraints).
//!
//! The load-bearing concepts (see docs/ARCHITECTURE.md § Simulation):
//! - **Committed facts**: anything ever observed is appended to the constraint
//!   ledger and is immutable.
//! - **Collapse**: promoting statistical state to concrete state by sampling a
//!   history consistent with all committed facts, bounded to depth N with
//!   synthesized boundary conditions at the frontier (no global cascades).
//!
//! Rules: headless, deterministic (no wall clock, no ambient randomness — all
//! entropy flows from seeds owned by the caller). Deep-time worldgen history is
//! this same crate run over pre-player time, driven by dc-worldgen.

pub const CRATE_ROLE: &str = "tiered simulation + constraint ledger";
