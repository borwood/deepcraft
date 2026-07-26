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
    self, DeepConfig, DeepField, DeepRun, build_field_cfg, production_config,
    production_config_with, run_cells,
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
        // **The anonymous-creep world was routed by a UNIFORM `p = 4`**, and a
        // fixed point is only reachable if *all* of the configuration it was
        // captured under is reproduced, not most of it. Hybrid `p` (journal/0113)
        // made the shipped exponent spatially varying the same day this fixture
        // landed, so it pins the flat ramp `p_chan == p_hill == 4.0`, which
        // `MfdParams::exponent_at` short-circuits (no ramp, no channel switch).
        // Without this the constants below would quietly become "anonymous creep
        // under post-b' routing", which is a claim about nothing.
        mfd_exponent: 4.0,
        mfd_exponent_channel: 4.0,
        // **And the pre-calibration RATES** (journal/0114). The erosional
        // calibration multiplied `weathering` / `diffusion` / `k_transport` /
        // `k_bedrock` by 45; a fixed point captured before it is only reachable by
        // reproducing that too. Same discipline as the `p = 4` pin below/above:
        // reaching a fixed point means reproducing ALL of the configuration it was
        // captured under, not most of it. Without this the constants would silently
        // become "the old solve at the NEW erosional clock", which is a claim about
        // no commit that ever existed.
        ..production_config_with(cells, SEED, &uncalibrated())
    }
}

/// The pre-calibration rate constants — see [`cfg_for`].
fn uncalibrated() -> dc_worldgen::DeepOverrides {
    dc_worldgen::DeepOverrides {
        calibrated_rates: Some(false),
        ..Default::default()
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
/// The off-half's constants are the fluvial-only world — **plus corrections #57**,
/// which rides in the same commit and applies to any mover, so it moves the
/// fluvial path too. See [`providers_common::GOLDEN_SURFACE_ANONYMOUS_CREEP`],
/// which carries that separation and the prior values; that the constant moved at
/// all is the evidence #57 was needed.
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

// ---------------------------------------------------------------------------
// 4. What a mover is allowed to have delivered.

/// **No unit a mover set down claims to be an in-place organic** — the rule
/// `Litho::as_deposited` already made for basement, extended to peat, coal and
/// charcoal because they are made where they lie rather than delivered
/// (corrections #57, journal/0112).
///
/// This is the one behavioural surprise of the slice and it is worth a named
/// guard. Charcoal is a **fire event**, capped at 0.04 m; the fluvial pass could
/// always have picked one up and re-deposited it, but it moves 0.109 % of this
/// world's sediment, so a transported organic never won a cell's mixture argmax.
/// Creep moves 918× more — and the first production run with it on produced a
/// voxel that was **8/8 charcoal**, because thin fire beds crept downslope, won
/// the argmax at a low-deposition cell, and then merged across epochs under one
/// mineral tag into a stratum the cap exists to forbid. The rule was always
/// incomplete; only the magnitude was new.
///
/// Asserted over the whole record rather than at the site that found it, because
/// "a stratum of charcoal" is wrong wherever it appears.
#[test]
fn no_deposited_unit_claims_to_be_an_in_place_organic() {
    use dc_worldgen::deeptime::lithology::Litho;
    let pregen = small_world();
    let f = field(&pregen.grid, true, true);
    let mut offenders = 0usize;
    let mut thickest = 0.0f64;
    for u in f.strata.iter().flat_map(|s| s.units.iter()) {
        // The *biotic* layer lays peat, coal and charcoal directly and honestly —
        // it deposits in place and never through a load — and it stamps its own
        // biofacies on the tag. What must not exist is a unit the **erosion
        // recorder** built (a mineral tag) that nonetheless claims to be one of
        // those in-place products.
        if u.tag.biota.is_organic() {
            continue;
        }
        if matches!(
            u.species,
            Litho::OrganicPeat | Litho::OrganicCoal | Litho::OrganicCharcoal
        ) {
            offenders += 1;
            thickest = thickest.max(u.thickness_m);
        }
    }
    assert_eq!(
        offenders, 0,
        "{offenders} mineral-tagged units claim to be an in-place organic \
         (thickest {thickest:.3} m) — a mover carried a fire bed or a peat and the \
         record called what landed a seam"
    );
}

// ---------------------------------------------------------------------------
// 5. The identity is an attribution — and where it stops being one.

/// **Within an epoch, creep identity cannot move a metre of terrain. Across
/// epochs it moves the world, and by a named route.**
///
/// The separation this slice is built on is that the per-species creep plane never
/// touches `H`: `grid.h` still moves by the scalar `netdiff`, the same four-edge
/// gather the anonymous path ran, so a defect in the identity arithmetic can
/// produce a wrong *rock* but never a wrong *elevation*. `record` reads the plane
/// after `diffuse` has already applied the scalar, and nothing else does.
///
/// **A one-epoch run is the only place that is observable, and finding that out
/// was the useful part of writing this test.** The first draft tried to isolate it
/// by turning the erodibility coupling off, and the terrain moved anyway; then by
/// turning `full_agents` off too (the frost multiplier, the eolian deflation
/// susceptibility and the wave attack rate each read `outcrop_shares` as well),
/// and it *still* moved. The last route is not a coupling that can be switched
/// off at all: the record's rock is what `outcrop_shares` publishes, that
/// composition is what the fluvial load **entrains**, and the competence ceiling
/// rains a species out by its settling velocity — so changing what a hillslope is
/// made of changes how much of it the river can hold. There is no configuration in
/// which the identity exists and is inert, which is the strongest available
/// statement that this is physics rather than bookkeeping.
///
/// So the claim is pinned where it is exactly true: **the first epoch, with the
/// agent roster off**. Both arms enter with the same (empty) record, every phase
/// runs on identical inputs, and the only thing that can differ is what `record`
/// writes at the end. The roster has to go because the wind and wave agents run
/// **after** the recorder — deliberately, so their own facies reach the record —
/// and would read this epoch's freshly-written species back into a rate before the
/// epoch is out.
#[test]
fn creep_identity_moves_no_terrain_in_the_epoch_it_is_measured_in() {
    let pregen = small_world();
    let base = DeepConfig {
        material_transport: true,
        iterations: 1,
        full_agents: false,
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
        "creep identity moved terrain inside the epoch that produced it — the \
         species plane is supposed to be read only by the recorder"
    );
    assert_ne!(
        providers_common::record_fingerprint(&off),
        providers_common::record_fingerprint(&on),
        "…and one epoch of creep must already have changed what the record says \
         the rock IS, or the plane is not reaching the recorder at all"
    );
}
