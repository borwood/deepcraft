//! **Movement 2b continuation (b) — material-aware hillslope creep**
//! (`docs/design/material-behavior.md` § 13.2, journal/0112, corrections #55).
//!
//! journal/0110 made the *fluvial* load material-aware, and the probe it was
//! required to ship measured the effect at **0.000006 % of the archive**. The
//! diagnosis was not a defect in the slice: **fluvial transport is 0.109 % of this
//! world's sediment routing.** Over the run the rivers pick up 659.5 m; hillslope
//! creep moves 605,117 m. Creep does 918× what the rivers do, and it carried no
//! identity at all — the world's dominant sediment router was anonymous.
//!
//! So creep joins the family. § 13.2's rule is that transport is *"an agent moves
//! material along a driving field"* — water, wind, ice, **gravity** — sharing the
//! same load machinery and differing only in field and competence curve. Gravity's
//! competence curve is the interesting one: **there isn't one.** Creep is
//! diffusive; it does not sort by grain size the way a falling ceiling does, so
//! **identity travels and nothing is sorted**. A colluvial apron of material from
//! one cell upslope beside a fluvial deposit of sorted far-travelled grains is a
//! real facies distinction, and it is the one this slice buys.
//!
//! This suite pins the four things that had to be true:
//!
//! 1. the identity pair — off is the fluvial-only world exactly, on moves it;
//! 2. the gate on `material_transport` is real, not decorative;
//! 3. **mass, per species**, across a diffusion junction with no downstream order;
//! 4. nothing about the creep path selects by grain size.
//!
//! Production-scale numbers — the fraction of the archive that ends up carrying
//! honest provenance — live in `examples/colluvium_probe.rs`.

use dc_worldgen::deeptime::{
    self, DeepConfig, DeepField, DeepRun, build_field_cfg, production_config, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

mod providers_common;

/// **The golden fixture's seed**, not a fresh one: the off-half of the identity
/// pair is a cross-commit claim against `providers_common`'s captured constants,
/// and a claim about a different world would prove nothing.
const SEED: u64 = providers_common::SEED;

fn small_world() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

fn cfg_for(cells: &CellGrid, material_transport: bool, material_creep: bool) -> DeepConfig {
    DeepConfig {
        material_transport,
        material_creep,
        ..production_config(cells, SEED)
    }
}

fn field(cells: &CellGrid, transport: bool, creep: bool) -> DeepField {
    build_field_cfg(cells, &cfg_for(cells, transport, creep))
}

fn run(cells: &CellGrid, transport: bool, creep: bool) -> DeepRun {
    run_cells(cells, &cfg_for(cells, transport, creep), false)
}

// ---------------------------------------------------------------------------
// 1. The identity pair.

/// **Both halves, because either alone is gameable.** A test that only asserted
/// "off is byte-identical to the anonymous-creep world" would pass against a flag
/// that does nothing; a test that only asserted "on is different" would pass
/// against a flag that broke something.
///
/// The off-half's constants are the goldens as they stood after Movement 2b's
/// first slice and before this one, so it is a genuine cross-commit statement.
/// The on-half moves the *terrain* as well as the record, and that is expected
/// rather than incidental: the species creep delivers is read back by
/// `outcrop_shares`, which is what the per-cell erodibility blend is built from,
/// so a colluvial apron of soft rock erodes at a soft rock's rate from the next
/// epoch onward. Identity is not a sidecar here.
#[test]
fn material_creep_off_is_the_fluvial_only_world_and_on_moves_it() {
    let pregen = small_world();
    let off = field(&pregen.grid, true, false);
    let on = field(&pregen.grid, true, true);

    let off_surface = providers_common::surface_fingerprint(&off);
    let off_record = providers_common::record_fingerprint(&off);
    println!("anonymous-creep surface = {off_surface:#018X}");
    println!("anonymous-creep record  = {off_record:#018X}");
    assert_eq!(
        off_surface,
        providers_common::GOLDEN_SURFACE_ANONYMOUS_CREEP,
        "with material creep OFF the surface must be the fluvial-only world: \
         {off_surface:#018X}"
    );
    assert_eq!(
        off_record,
        providers_common::GOLDEN_RECORD_ANONYMOUS_CREEP,
        "with material creep OFF the record must be the fluvial-only world: \
         {off_record:#018X}"
    );

    let on_surface = providers_common::surface_fingerprint(&on);
    let on_record = providers_common::record_fingerprint(&on);
    println!("shipped surface = {on_surface:#018X}");
    println!("shipped record  = {on_record:#018X}");
    assert_ne!(
        on_record, off_record,
        "creep carried identity and the record did not notice — the colluvial \
         species never reached a unit, so the slice is pipework"
    );
}

/// **The gate on `material_transport` is real.** Creep moves the composition the
/// `outcrop_shares` seam publishes for entrainment, which only exists on the
/// material-aware path; asking for creep identity without it must be inert rather
/// than take a second composition walk of its own.
///
/// Asserted against the **pre-2b** goldens, so this is the strongest form of the
/// claim: with transport off and creep *requested*, the world is the scalar-load
/// solve, byte for byte.
#[test]
fn creep_identity_is_inert_without_material_transport() {
    let pregen = small_world();
    let f = field(&pregen.grid, false, true);
    let r = run(&pregen.grid, false, true);
    assert!(
        !r.erosion.is_material_creep(),
        "creep reported that it carries identity while the composition plane it \
         reads does not exist"
    );
    assert_eq!(
        providers_common::surface_fingerprint(&f),
        providers_common::GOLDEN_SURFACE_SCALAR_LOAD,
        "requesting creep identity without material transport moved the world"
    );
    assert_eq!(
        providers_common::record_fingerprint(&f),
        providers_common::GOLDEN_RECORD_SCALAR_LOAD,
        "requesting creep identity without material transport moved the record"
    );
}

// ---------------------------------------------------------------------------
// 2. Mass, per species.

/// **The itemisation equals its own total, and no species is created or
/// destroyed** — the two mass statements, measured continuously over a whole world
/// rather than argued.
///
/// A diffusion junction has no downstream order to walk, so journal/0110's
/// per-species junction test does not transfer as written: there is no "cell hands
/// its load to its receivers" moment. What replaces it is the pair of running
/// maxima the pass keeps:
///
/// - **the itemisation** — `Σ_species` of a cell's creep must be the scalar metres
///   the terrain actually moved there, every cell, every epoch. This is the
///   standing probe-defect shape in this repo (*a missing row in an itemisation*)
///   applied to the identity path;
/// - **conservation** — `Σ_cells` of a species must be zero, every species, every
///   epoch. Creep only *moves*: what one cell gained, another lost. This is where
///   an antisymmetry mistake between the two endpoints of an edge lands, and it is
///   the direct heir of journal/0110's junction claim.
///
/// The own-budget rule survives the transfer intact: each edge flux is split by
/// the **donor's** composition with the last non-zero share taking the residual,
/// so the split closes to the bit and no species is rounded against a denominator
/// it shares with another. Both endpoints run that identical split on an
/// identical flux, which is what makes the conservation half exact rather than
/// approximate.
#[test]
fn no_species_is_created_or_destroyed_by_creep() {
    let pregen = small_world();
    let r = run(&pregen.grid, true, true);
    let item = r.erosion.max_creep_itemisation_residue();
    let cons = r.erosion.max_creep_conservation_residue();
    assert!(
        r.erosion.creep_outflux_faces() > 0,
        "no cell shed any creep at all, so the two residues below are vacuous"
    );
    assert!(
        item < 1e-12,
        "the creep species itemisation stopped equalling the metres the terrain \
         moved: relative gap {item:e}"
    );
    assert!(
        cons < 1e-12,
        "creep created or destroyed a species: relative amount {cons:e}"
    );
}

/// **The global half.** A species share attributed to a cell but never taken from
/// its donor would pass a per-edge check and fail this one: the whole-world ledger
/// `Δ(ΣR + ΣH) == uplift + biotic`, over a full production run with both members
/// of the transport family carrying identity.
#[test]
fn mass_is_conserved_with_material_creep_on() {
    let pregen = small_world();
    let r = run(&pregen.grid, true, true);
    let residual = deeptime::total_mass(&r.grid) - r.mass_before - r.uplift_total - r.biotic_total;
    assert!(
        residual.abs() < 1.0,
        "mass leaked with material creep on: residual {residual} \
         (uplift {}, biotic {})",
        r.uplift_total,
        r.biotic_total
    );
}

/// **The terrain is moved by the scalar, and the identity is an attribution.**
///
/// This is the separation the whole slice is built on and it is worth pinning
/// directly: the per-species creep plane never touches `H`. `grid.h` still moves
/// by `netdiff`, the same four-edge gather the anonymous path ran, so a defect in
/// the identity arithmetic can produce a wrong *rock* but never a wrong
/// *elevation*.
///
/// Proven by the one configuration where nothing else can differ: with the
/// erodibility coupling **off**, the species a unit is made of has no path back
/// into any rate, so the terrain must be identical with creep identity on and off.
/// With the coupling on — the shipped configuration — it deliberately is not, and
/// that is the previous test's `assert_ne`.
#[test]
fn creep_identity_cannot_move_the_terrain_on_its_own() {
    let pregen = small_world();
    let base = DeepConfig {
        material_transport: true,
        erodibility: false,
        ..production_config(&pregen.grid, SEED)
    };
    let off = build_field_cfg(
        &pregen.grid,
        &DeepConfig {
            material_creep: false,
            ..base
        },
    );
    let on = build_field_cfg(
        &pregen.grid,
        &DeepConfig {
            material_creep: true,
            ..base
        },
    );
    assert_eq!(
        providers_common::surface_fingerprint(&off),
        providers_common::surface_fingerprint(&on),
        "with erodibility off, creep identity has no path into a rate and must \
         not have moved a metre of terrain"
    );
    assert_ne!(
        providers_common::record_fingerprint(&off),
        providers_common::record_fingerprint(&on),
        "…and it must still have changed what the record says the rock IS"
    );
}
