//! **Slot `parent_p`** — falsifiers for the parent-material phosphorus seam
//! (journal/0060), including the pass-level granularity claim.

mod providers_common;
use providers_common::{SEED, production_pregen};

/// The `parent_p` seam is **pass-level**: the plane is materialized once at
/// `BioticSim::new` and read by index, never called per cell inside the epoch
/// loop. The observable consequence is that the plane exists and is `n` long.
#[test]
fn the_parent_phosphorus_plane_is_materialized_once() {
    use dc_worldgen::deeptime::{BioticSim, DeepConfig, build_cells};
    let pregen = production_pregen();
    let cfg = DeepConfig {
        seed: SEED,
        cell_m: 4000.0,
        iterations: 1,
        biotic: true,
        ..DeepConfig::default()
    };
    let mut grid = build_cells(&pregen.grid, &cfg);
    let n = grid.w * grid.w;
    let sim = BioticSim::new(&mut grid, &cfg, false);
    assert_eq!(sim.parent_p().len(), n, "the plane is one entry per cell");
    assert!(
        sim.parent_p().iter().all(|v| *v == 1.0),
        "the identity provider gives a uniform 1.0 parent-P field"
    );
}
