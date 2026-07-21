//! Deep-config flag plumbing (journal/0039) — the falsifiers for the gen-time
//! override channel that lets a world boot with `tectonic_history` /
//! `full_agents` / a chosen amplitude ON.
//!
//! The load-bearing invariant is **byte-identity of the empty-override path**:
//! an all-`None` [`DeepOverrides`] must reproduce the sealed production path to
//! the bit, so every already-created world stays reproducible. Then the two
//! "the override actually bites" checks (tectonic history populates the drainage
//! export; full agents perturb the surface), and the [`Extent`] arg parser.

use dc_worldgen::deeptime::{
    self, DeepOverrides, build_field, build_field_with, production_config, production_config_with,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0DEE_9C04_2039;

fn small_world(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

// ---------------------------------------------------------------------------
// The empty-override byte-identity proof.

/// `production_config_with(.., &default)` must equal `production_config` — the
/// override struct is inert when empty. Compared through `Debug` (DeepConfig has
/// no `PartialEq`), a total field-by-field check.
#[test]
fn empty_overrides_yield_the_production_config() {
    let pregen = small_world(SEED);
    let base = production_config(&pregen.grid, SEED);
    let with = production_config_with(&pregen.grid, SEED, &DeepOverrides::default());
    assert_eq!(
        format!("{base:?}"),
        format!("{with:?}"),
        "empty overrides perturbed the production config"
    );
    assert!(DeepOverrides::default().is_empty());
}

/// `build_field` and `build_field_with(.., &default)` must be byte-identical on
/// every plane the world keeps — the direct proof that threading the override
/// channel changed nothing for a default boot.
#[test]
fn empty_overrides_build_a_byte_identical_field() {
    let pregen = small_world(SEED);
    let base = build_field(&pregen.grid, SEED);
    let with = build_field_with(&pregen.grid, SEED, &DeepOverrides::default());
    assert_eq!(base.w, with.w);
    assert_eq!(base.cell_m.to_bits(), with.cell_m.to_bits());
    assert_eq!(base.surf, with.surf, "surface plane diverged");
    assert_eq!(base.strata, with.strata, "strata record diverged");
    assert_eq!(base.recv, with.recv, "drainage receiver diverged");
    assert_eq!(base.area, with.area, "drainage area diverged");
    assert_eq!(base.lake, with.lake, "lake mask diverged");
    assert_eq!(base.exhum, with.exhum, "exhumation plane diverged");
    assert_eq!(base.t_crust, with.t_crust, "crust plane diverged");
    assert_eq!(base.chapters.len(), with.chapters.len(), "chapter count");
}

/// The headline claim, at the `Pregen` seam the client actually uses:
/// `Pregen::run(p)` and `Pregen::run_with(p, &default)` land on a byte-identical
/// `DeepField`, so the new door does not disturb the old one.
#[test]
fn pregen_run_equals_run_with_default_overrides() {
    let a = small_world(SEED);
    let b = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: Extent::Small,
        },
        &DeepOverrides::default(),
    );
    assert_eq!(a.deep.surf, b.deep.surf, "surface plane diverged");
    assert_eq!(a.deep.strata, b.deep.strata, "strata record diverged");
    assert_eq!(a.deep.recv, b.deep.recv, "drainage receiver diverged");
    assert_eq!(a.deep.exhum, b.deep.exhum, "exhumation plane diverged");
    assert_eq!(a.deep.t_crust, b.deep.t_crust, "crust plane diverged");
    assert_eq!(
        a.deep.chapters.len(),
        b.deep.chapters.len(),
        "chapter count"
    );
}

// ---------------------------------------------------------------------------
// The overrides actually bite.

/// With `tectonic_history: Some(true)` the field's tectonic-only exports —
/// chapters, exhumation, drainage receiver — become non-empty (they are empty on
/// the production default, see `field.rs` and `tectonic_history.rs`).
#[test]
fn tectonic_history_override_populates_the_tectonic_exports() {
    let pregen = small_world(SEED);
    let off = build_field(&pregen.grid, SEED);
    assert!(
        off.chapters.is_empty(),
        "production default is tectonics-off"
    );
    assert!(off.recv.is_empty());
    assert!(off.exhum.is_empty());

    let on = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            tectonic_history: Some(true),
            ..DeepOverrides::default()
        },
    );
    assert!(!on.chapters.is_empty(), "chapter table not exported");
    assert!(!on.recv.is_empty(), "drainage receiver not exported");
    assert!(!on.exhum.is_empty(), "exhumation plane not exported");
}

/// With `full_agents: Some(true)` the eroded surface differs from production —
/// the wind/frost/wave roster genuinely reaches the run.
#[test]
fn full_agents_override_changes_the_surface() {
    let pregen = small_world(SEED);
    let off = build_field(&pregen.grid, SEED);
    let on = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            full_agents: Some(true),
            ..DeepOverrides::default()
        },
    );
    assert_ne!(
        off.surf, on.surf,
        "full_agents changed nothing on the surface"
    );
}

/// `amplitude` (thickening_scale) rides through the override, and — since it
/// only bites with tectonics on — moving it changes a tectonic-history field.
#[test]
fn amplitude_override_rides_through_with_tectonics() {
    let pregen = small_world(SEED);
    let low = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            tectonic_history: Some(true),
            thickening_scale: Some(40.0),
            ..DeepOverrides::default()
        },
    );
    let high = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            tectonic_history: Some(true),
            thickening_scale: Some(320.0),
            ..DeepOverrides::default()
        },
    );
    assert_ne!(
        low.surf, high.surf,
        "amplitude did not reach the tectonic run"
    );
}

// ---------------------------------------------------------------------------
// Extent arg parsing.

#[test]
fn extent_from_arg_round_trips_the_three_names_and_rejects_garbage() {
    assert_eq!(Extent::from_arg("small"), Some(Extent::Small));
    assert_eq!(Extent::from_arg("medium"), Some(Extent::Medium));
    assert_eq!(Extent::from_arg("large"), Some(Extent::Large));
    // Case-insensitive and whitespace-trimmed.
    assert_eq!(Extent::from_arg("LARGE"), Some(Extent::Large));
    assert_eq!(Extent::from_arg("  Medium "), Some(Extent::Medium));
    // Garbage rejected.
    assert_eq!(Extent::from_arg("huge"), None);
    assert_eq!(Extent::from_arg(""), None);
    assert_eq!(Extent::from_arg("smallish"), None);
}

/// Sanity: the plumbing keeps `deeptime` re-exports reachable from the crate
/// root (`dc_worldgen::DeepOverrides`) the way the client imports them.
#[test]
fn deep_overrides_is_reexported_at_crate_root() {
    let _ = dc_worldgen::DeepOverrides::default();
    // And the module path used by these tests resolves too.
    let _ = deeptime::DeepOverrides::default();
}
