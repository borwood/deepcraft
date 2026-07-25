//! **Where to stand to SEE what the geotherm did to coal** (journal/0093).
//!
//! journal/0093 recalibrated coalification onto a real geotherm temperature
//! (`COAL_ONSET_C` 8 → 22 °C) and, in doing so, **relocated** coal: it now
//! concentrates where the crust and the surface are warm rather than wherever
//! peat happened to be buried deepest. ROADMAP records the appearance walk as
//! owed. This is that walk's map — **coordinates only**. Nothing here tunes,
//! recalibrates or judges coal (a user-blessed placeholder); it answers *where
//! is a coal seam a bench will expose, and is the relocation legible on foot*.
//!
//! Two tiers, because the deep record and the world a player digs are not the
//! same thing (organic.rs's lesson: a thick *record* seam can bury below the
//! collapse column):
//!
//! 1. **Census** — every deep cell's contiguous `Biofacies::Coal` runs, with
//!    the run's depth below the top of the record and its thickness. Cheap,
//!    world-wide, and the source of the distribution numbers.
//! 2. **Verification** — for the shallowest/thickest candidates, actually
//!    generate the chunk column and count `Block::Material(COAL)` voxels per
//!    voxel column. That is the seam the player's bench cuts into, so the
//!    station's reported depth and thickness are voxel facts, not record facts.
//!
//! `cargo run --release -p dc-worldgen --example coal_walk_tour`

use dc_core::{Block, ChunkPos, MaterialId};
use dc_worldgen::deeptime::{self, Biofacies, COAL_ONSET_C, DeepField, SEA_LEVEL_M};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, LAT_NORTH, LAT_SOUTH, Pregen, WorldParams};
use dc_worldgen::{Extent as _E, WorldGenerator};

/// The world dc-client boots (same as `s18_weathering_tour`).
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;
/// Station A of this walk — the saprolite front, already chosen.
const STATION_A_XZ_M: (f64, f64) = (84_185.0, 9_212.0);
/// Border ring excluded from station picks (march edge artifacts, per tour_map.rs).
const EDGE_MARGIN: i64 = 4;
/// A bench is a shelf a few voxels deep; a seam deeper than this is a mine, not
/// a road-cut. Candidate filter for station selection only.
const BENCH_REACH_M: f64 = 12.0;

fn main() {
    let _ = (_E::Medium, COAL_ONSET_C);
}
