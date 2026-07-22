//! **Slot `wave_energy`** — falsifiers for the littoral seam (journal/0060).

mod providers_common;
use providers_common::{production_pregen, surface_fingerprint};

/// A **non-identity** provider actually reaches the run — the seam is a seam,
/// not decoration. Halving the littoral rate everywhere must move the surface.
///
/// (This is the `full_agents` override falsifier's shape, one layer down.)
#[test]
fn a_non_identity_wave_provider_reaches_the_run() {
    use dc_worldgen::deeptime;
    use dc_worldgen::deeptime::providers::{Providers, WaveCell};
    fn half(c: WaveCell) -> f64 {
        c.base_rate * 0.5
    }
    let pregen = production_pregen();
    let base = deeptime::production_config(&pregen.grid, providers_common::SEED);
    let mut swapped = base;
    swapped.providers = Providers {
        wave_energy: half,
        ..Providers::default()
    };
    let a = deeptime::build_field_cfg(&pregen.grid, &base);
    let b = deeptime::build_field_cfg(&pregen.grid, &swapped);
    assert_ne!(
        surface_fingerprint(&a),
        surface_fingerprint(&b),
        "a halved wave-energy provider did not reach the littoral agent"
    );
}
