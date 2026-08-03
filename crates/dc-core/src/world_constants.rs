//! World-level physical constants — the values a WORLD defines, that both the
//! sim and the content passes read.
//!
//! **Gravity is a world constant and it defaults to Earth** (DECIDED 2026-08-02,
//! user — `docs/ARCHITECTURE.md` § *Gravity is a WORLD constant, and it defaults
//! to Earth*). It lives here, in the crate every other crate depends on, because
//! its consumers span all of them: the character step and the player walker
//! today, and the geo passes (lithostatic pressure `ρ·g·h`, grain settling
//! velocity, transport capacity, isostasy, hillslope diffusion) as each adopts
//! it.
//!
//! **This module exists to end a two-authority defect, not merely to hold a
//! number.** Before 2026-08-02 the value `25.0` was written independently in
//! `dc-api`'s `CharacterConfig::default` and in `dc-client`'s `player.rs`, and
//! the two agreed only by coincidence of hand-typing — the same shape as
//! `root_bob_m` (corrections #80, #93), the gaze (#94) and trunk facing. Add a
//! consumer by **reading this constant**, never by restating it.
//!
//! **`25.0` was never chosen.** It was a bring-up artifact, surfaced when the
//! gait bake's build found the design pass had derived its whole Froude table at
//! `9.81` against a world running at `25.0`; the user's ruling was that nobody
//! had consciously picked it and that gravity should be world-defined with an
//! Earth default. *Existence is not standing, applied to a physical constant.*
//!
//! # Per-world gravity is the point, not a someday
//!
//! The **seam is this value's location**; the **loader is E7's per-world
//! manifest** — the same consumer `CadenceTable` waits on
//! (`docs/dependency-graph.md`). The constant is the default a world takes when
//! its manifest says nothing. Building the loader ahead of its caller is what
//! seam-first forbids, so the field exists and the loader does not.
//!
//! # ⚠ The name is `DEFAULT_`, not `EARTH_`, and that is deliberate
//!
//! It was `EARTH_GRAVITY_M_S2` for exactly one commit. The user caught it:
//! *"we're both thinking that in the future, a player may be able to configure
//! their world with a different gravity rate when starting a new world, right?
//! is this name generic enough?"* It was not. `EARTH_` names **what the value
//! is** (a physical fact); every consumer reads it as **what a world gets when
//! nothing says otherwise** (a policy that currently equals Earth's). Naming the
//! fact and using it as the policy is the same one-word-two-concepts trap that
//! cost this arc the gaze/neck-bend confusion (corrections #94) and the bob —
//! and here it actively misleads, because `PhysicsConfig` reading
//! `EARTH_GRAVITY_M_S2` reads as *physics is pinned to Earth* rather than
//! *this world has not said otherwise*.
//!
//! If a consumer ever genuinely needs **Earth as a reference** — comparing a
//! derived rate against a published band measured on Earth — that is a second,
//! differently-named constant, added with its consumer. Not before.
//!
//! # ⚠ STILL OPEN: the CONFIGURATION is three fields, even though the default is one
//!
//! This constant collapses three hardcoded literals into one authority. It does
//! **not** yet give a world one place to *set* gravity: `CharacterConfig`,
//! `PhysicsConfig` and `dc-client`'s player each hold their own value, all
//! merely defaulting here. **A world that wants Mars gravity today must set it in
//! three places** — the same defect one level up, moved from the literal to the
//! field. The end state is one world-level value the three derive from, and it
//! arrives with E7's manifest. Recorded here rather than left implicit, because
//! a fix that resolves the visible half of a defect and leaves the other half
//! unnamed is how the visible half comes back.
//!
//! # ⚠ Gravity becomes WORLD IDENTITY the moment a pass reads it
//!
//! No worldgen pass consumes gravity today — every `gravity` hit in
//! `dc-worldgen` is a doc comment describing the mechanism beside code that
//! never receives it. That is exactly why the Earth default landed when it did:
//! with no pass consuming `g`, changing it moves **no terrain golden**. Once a
//! pass takes `g` as an argument, gravity joins the seed and the frozen content
//! set as world identity, and the same change becomes a whole-world re-capture.
//!
//! **A pass adopting gravity does NOT get to factor `g` back out of an
//! empirically fitted constant and call the result derived** — a fitted constant
//! with gravity extracted is still fitted (`ARCHITECTURE.md` § *A summary is not
//! an authority*, and the measure-against-the-literature rule). Each adoption is
//! its own slice with its own literature check.

/// Standard gravity at Earth's surface, m/s² — the default a world takes when
/// nothing overrides it (CODATA/ISO 80000 standard gravity is 9.806 65; 9.81 is
/// the conventional engineering rounding and is what the gait bake's published
/// Froude bands were derived against).
///
/// **Read this; do not restate it.** See the module docs for why that rule
/// exists and what it cost before.
pub const DEFAULT_GRAVITY_M_S2: f64 = 9.81;
