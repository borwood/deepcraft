//! Hierarchical lazy world generation and deep-time history.
//!
//! "Theoretically infinite, globally aware" is resolved by hierarchy with
//! bounded neighborhoods: each level (continent graph → region → chunk) is
//! generated lazily, and a level may depend only on a bounded neighborhood of
//! the level above it. Rivers are planned at region-graph scale before any
//! chunk in the region materializes; trade routes at faction-graph scale;
//! no level ever needs unbounded lookahead.
//!
//! Deep-time history (geological, then social/polis/territory) is produced by
//! running dc-sim's coarse and statistical tiers over pre-player time, so
//! worldgen history and live far-simulation are one system, not two.
//!
//! Rules: headless, deterministic, biomes/features defined as data via the
//! registry (vanilla content is just the first content pack).

pub const CRATE_ROLE: &str = "hierarchical lazy worldgen + deep-time history";
