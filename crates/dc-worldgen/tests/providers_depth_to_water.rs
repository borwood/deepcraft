//! **Slot `depth_to_water`** — falsifiers for the water-table seam
//! (journal/0061): both arms of the seam, on the real production grid.

mod providers_common;
use providers_common::{GOLDEN_RECORD, GOLDEN_SURFACE, SEED, production_pregen};
use providers_common::{record_fingerprint, surface_fingerprint};

/// The identity `depth_to_water` leaves the plane empty — the empty-plane +
/// identity-accessor shape, stated through the slot rather than the free
/// function, so a mis-wired `default()` fails here.
#[test]
fn the_identity_water_table_slot_leaves_the_plane_empty() {
    use dc_worldgen::deeptime::providers::{Providers, WaterPass};
    let (precip, r, h) = (vec![0.4f32; 9], vec![25.0f64; 9], vec![2.0f64; 9]);
    let (area, recv, filled) = (vec![12.0f64; 9], vec![-1i32; 9], vec![27.5f64; 9]);
    let mut plane = vec![0.5f32; 9];
    Providers::default().depth_to_water(
        WaterPass {
            w: 3,
            epoch: 0,
            precip: &precip,
            r: &r,
            h: &h,
            area: &area,
            recv: &recv,
            filled: &filled,
        },
        &mut plane,
    );
    assert!(plane.is_empty());
}

/// **The `depth_to_water` agreement test** — the strongest statement this slice
/// can make, and the one ARCHITECTURE.md § *A summary is not an authority* asks
/// for: *a test asserting the summary AGREES with the authority*.
///
/// The seam has two paths: the **empty plane**, where `wet_at` computes the
/// three-term proxy inline (what the identity provider selects), and the
/// **materialized plane**, where a provider filled `n` entries at the pass
/// boundary and `wet_at` reads by index. Registering a provider that
/// materializes *exactly the proxy* runs the whole production world down the
/// second path and lands on the pre-slice goldens — so the two branches are the
/// same function, and the pass-level plumbing itself perturbs nothing.
///
/// Without this, the byte-identity proof only ever exercises the empty branch,
/// and the plane path would ship untested until an heir arrived to break it.
#[test]
fn the_materialized_proxy_plane_agrees_with_the_inline_proxy() {
    use dc_worldgen::deeptime;
    use dc_worldgen::deeptime::providers::{Providers, WaterPass, identity_wet_index};
    fn proxy_plane(pass: WaterPass<'_>, out: &mut Vec<f32>) {
        let n = pass.w * pass.w;
        out.clear();
        out.reserve(n);
        for i in 0..n {
            out.push(identity_wet_index(
                pass.precip[i],
                pass.r[i] + pass.h[i],
                pass.area[i],
            ));
        }
    }
    let pregen = production_pregen();
    let mut cfg = deeptime::production_config(&pregen.grid, SEED);
    cfg.providers = Providers {
        depth_to_water: Some(proxy_plane),
        ..Providers::default()
    };
    assert!(
        !cfg.providers.is_identity(),
        "this must be a registered non-default provider, not the identity slot"
    );
    let f = deeptime::build_field_cfg(&pregen.grid, &cfg);
    assert_eq!(
        surface_fingerprint(&f),
        GOLDEN_SURFACE,
        "the materialized-plane path disagrees with the inline proxy (surface)"
    );
    assert_eq!(
        record_fingerprint(&f),
        GOLDEN_RECORD,
        "the materialized-plane path disagrees with the inline proxy (record)"
    );
}

/// The `depth_to_water` seam is a seam: a world where every cell's water table
/// sits at the surface is not this world. Four consumers threshold `wet` — the
/// peat-former waterlog gate, the decomposition drain factor, the fire dryness
/// term, and the peat-site test — so saturating the plane has to move the
/// organic record.
#[test]
fn a_non_identity_water_table_reaches_the_run() {
    use dc_worldgen::deeptime;
    use dc_worldgen::deeptime::providers::{Providers, Slot, WaterPass};
    fn drowned(pass: WaterPass<'_>, out: &mut Vec<f32>) {
        out.clear();
        out.resize(pass.w * pass.w, 1.0);
    }
    let pregen = production_pregen();
    let base = deeptime::production_config(&pregen.grid, SEED);
    let mut swapped = base;
    swapped.providers = Providers {
        depth_to_water: Some(drowned),
        ..Providers::default()
    };
    // The identity report is what a manifest would record for this world.
    assert_eq!(
        swapped.providers.non_identity_slots(),
        vec![Slot::DepthToWater]
    );
    let a = deeptime::build_field_cfg(&pregen.grid, &base);
    let b = deeptime::build_field_cfg(&pregen.grid, &swapped);
    assert_ne!(
        record_fingerprint(&a),
        record_fingerprint(&b),
        "a saturated water-table provider did not reach the biotic layer"
    );
}

/// The `depth_to_water` seam is **pass-level, once per epoch**: the provider is
/// called at the top of `BioticSim::step`, not per cell. Counting is not
/// possible through a `fn` pointer without state, so the observable is the
/// plane's *shape* — one entry per cell, refreshed each epoch — plus the fact
/// that the identity leaves it empty and therefore allocates nothing.
#[test]
fn the_identity_water_table_plane_is_empty() {
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
    let sim = BioticSim::new(&mut grid, &cfg, false);
    assert!(
        sim.wet().is_empty(),
        "the identity provider must leave the water-table plane unallocated"
    );
}
