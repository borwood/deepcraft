//! **S11 spike** — water locality + the free-water body graph.
//!
//! Additive and standalone: nothing in the production world-generation path
//! calls this module. It exists to measure the two questions
//! `docs/design/water.md` dispatched, and its results live in
//! `docs/spikes/S11-results.md`.
//!
//! - **Q1 — is bound water local?** [`sat`] is the smallest honest saturation
//!   relaxation over the S8 pore model (infiltration + permeability-limited
//!   percolation and lateral redistribution) with the water table *read* as the
//!   top of the saturated zone. The halo of a perturbation is measured against
//!   permeability contrast, the way S9 measured erosion's decay length.
//! - **Q2 — does the free-water body graph stay sparse?** [`body`] persists
//!   bodies and derives voxels; [`conn`] is the derived two-level connectivity
//!   index that makes "which body is this voxel in?" an O(1) question with no
//!   search at any radius.
//!
//! Deep time is untouched — this is present-tier only.

//! **S15 spike** ([`coarse`]) extends this module with the capacity question
//! S11 could not ask: its 415 ms hypsometry scan walked a fully-resident toy
//! volume, so it is an honest number for a world that does not exist. `coarse`
//! is the per-coarse-cell hypsometric summary that replaces the walk against a
//! lazily generated, evicting world. Results: `docs/spikes/S15-results.md`.

pub mod body;
pub mod coarse;
pub mod conn;
pub mod sat;
pub mod vox;

use std::collections::HashMap;

pub use body::{Body, BodyGraph, BodyId, BodyKind, Link, ResolveStats, WaterEvent};
pub use coarse::{CAP_CELL, CAP_SUBSAMPLE, CapCell, CapSummary, ExactCurve, summary_hash};
pub use conn::ConnIndex;
pub use sat::{ROCK_VOID, RockProps, SATURATED, SatField};
pub use vox::VoxWorld;

/// Bind every live body to its air component, and each component to the
/// lowest-id body sitting in it.
///
/// This is the *rebinding* step a reload performs: the graph persists anchors,
/// the geometry supplies components, and the two are joined here. Deterministic
/// — ties resolve to the lowest body id.
pub fn bind(
    graph: &BodyGraph,
    conn: &mut ConnIndex,
    w: &VoxWorld,
) -> (HashMap<BodyId, u32>, HashMap<u32, BodyId>) {
    let mut comp_of: HashMap<BodyId, u32> = HashMap::new();
    let mut body_of: HashMap<u32, BodyId> = HashMap::new();
    let mut ids: Vec<BodyId> = graph
        .bodies
        .iter()
        .filter(|b| b.merged_into.is_none())
        .map(|b| b.id)
        .collect();
    ids.sort_unstable();
    for id in ids {
        let a = graph.get(id).anchor;
        if !w.in_bounds(a[0] as i64, a[1] as i64, a[2] as i64) {
            continue;
        }
        let Some(c) = conn.component(a[0] as usize, a[1] as usize, a[2] as usize) else {
            continue;
        };
        comp_of.insert(id, c);
        body_of.entry(c).or_insert(id);
    }
    (comp_of, body_of)
}
