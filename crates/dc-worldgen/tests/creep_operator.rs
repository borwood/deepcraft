//! **The sub-cycled hillslope-transport operator** (journal/0122) — the fork this
//! slice introduced, and the fixed point on the other side of it.
//!
//! `DeepConfig::creep_substep` is the fifth member of the family `mfd`,
//! `material_transport`, `material_creep` and `calibrated_rates` belong to: a
//! second path beside an old one rather than a refactor of it, so the world the
//! project used to generate stays reachable and stays hashed. The discipline that
//! makes a moved golden an *authorized* move rather than a lost fixed point is
//! that somebody still asserts the old value by name. That is this file.
//!
//! Why the operator changed at all is `stubs.md` #29 and journal/0116's register:
//! the pass is an **explicit** Laplacian whose per-edge coefficient at the
//! calibrated rates sits ~100× past the bound at which such an operator stops
//! oscillating, and its flux limiter — capping export at the cell's whole
//! inventory rather than at the amount that would level the pair — turned the
//! divergence into an exactly amplitude-preserving period-2 flip-flop. The mode is
//! isolated on a bare grid in `erosion.rs::hillslope_operator_tests`; the world's
//! before/after is `examples/creep_operator_probe.rs`.

use dc_worldgen::deeptime::{
    self, CREEP_MAX_EDGE_COEFF, DeepConfig, DeepOverrides, build_field_cfg, production_config_with,
    run_cells,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

mod providers_common;
use providers_common::{
    GOLDEN_RECORD_UNBOUNDED_CREEP, GOLDEN_SURFACE_UNBOUNDED_CREEP, SEED as GOLDEN_SEED,
    golden_pregen, record_fingerprint, surface_fingerprint,
};

fn cfg(pregen: &Pregen, substep: bool) -> DeepConfig {
    DeepConfig {
        creep_substep: substep,
        ..production_config_with(&pregen.grid, GOLDEN_SEED, &DeepOverrides::default())
    }
}

/// **The pre-journal/0122 world is still reachable, and still exact.** The
/// sub-cycled operator moved the shipped goldens — that is the slice's whole point
/// and they moved with it — but a moved golden is only *authorized* if the thing
/// it used to describe is still there.
///
/// Both halves, because either alone is gameable: with the flag off the fixture
/// must reproduce the pre-slice constants bit for bit, and with it on the world
/// must actually move, or the flag is a no-op wearing a fix's clothes.
#[test]
fn creep_substep_off_is_the_unbounded_world_and_on_moves_it() {
    let pregen = golden_pregen();
    let off = build_field_cfg(&pregen.grid, &cfg(&pregen, false));
    let on = build_field_cfg(&pregen.grid, &cfg(&pregen, true));

    let off_surface = surface_fingerprint(&off);
    let off_record = record_fingerprint(&off);
    println!("unbounded-creep surface = {off_surface:#018X}");
    println!("unbounded-creep record  = {off_record:#018X}");
    assert_eq!(
        off_surface, GOLDEN_SURFACE_UNBOUNDED_CREEP,
        "with sub-cycling OFF the surface must be the pre-0122 world: \
         {off_surface:#018X}"
    );
    assert_eq!(
        off_record, GOLDEN_RECORD_UNBOUNDED_CREEP,
        "with sub-cycling OFF the strata record must be the pre-0122 world: \
         {off_record:#018X}"
    );
    assert_ne!(
        surface_fingerprint(&on),
        off_surface,
        "sub-cycling changed no elevation anywhere: it is not reaching the pass"
    );
}

/// **Mass is still exact with the pass sub-cycled.** The inner step's
/// conservation was never in doubt — each edge's flux is referenced identically
/// from both endpoints — but a driver that runs it `n` times and accumulates a
/// species itemisation alongside is a new place to lose material, so the run-wide
/// falsifier `Δ(ΣR + ΣH) = uplift + biotic` is re-asserted through it.
#[test]
fn mass_is_conserved_with_the_pass_sub_cycled() {
    let pregen = golden_pregen();
    let r = run_cells(&pregen.grid, &cfg(&pregen, true), false);
    let residual = deeptime::total_mass(&r.grid) - r.mass_before - r.uplift_total - r.biotic_total;
    assert!(
        residual.abs() < 1.0,
        "mass leaked with the hillslope pass sub-cycled: residual {residual} \
         (uplift {}, biotic {})",
        r.uplift_total,
        r.biotic_total
    );
    // And the identity the creep plane carries closes against the epoch's total
    // ΔH, which is the audit a sub-cycled epoch could plausibly break: it sums
    // `n` sub-steps of species against `n` sub-steps of scalar.
    assert!(
        r.erosion.max_creep_itemisation_residue() < 1e-9,
        "the sub-cycled creep itemisation does not equal its own total: {}",
        r.erosion.max_creep_itemisation_residue()
    );
    assert!(
        r.erosion.max_creep_conservation_residue() < 1e-9,
        "a species was created or destroyed across sub-steps: {}",
        r.erosion.max_creep_conservation_residue()
    );
}

/// **The shipped world sub-cycles, and that is the finding rather than an
/// accident.** `diffusion = 0.12` sits just inside [`CREEP_MAX_EDGE_COEFF`], so
/// the *config* rate was never the problem — but [`eff_diff`] folds in the
/// lithology's creep susceptibility, and a peat-rich cell carries several times
/// the config rate. The shipped world therefore has cells past the bound too, and
/// the pass takes more than one sub-step on it.
///
/// Asserted rather than narrated because it is exactly the claim that decides
/// whether the goldens had to move: had the peak sat inside the bound, `n` would
/// be 1 everywhere and this slice would have been byte-identical.
///
/// [`eff_diff`]: dc_worldgen::deeptime::CREEP_MAX_EDGE_COEFF
#[test]
fn the_shipped_world_has_cells_past_the_bound() {
    let pregen = Pregen::run(WorldParams {
        seed: GOLDEN_SEED,
        extent: Extent::Small,
    });
    let c = cfg(&pregen, true);
    let r = run_cells(&pregen.grid, &c, false);
    let d_max = r.erosion.max_eff_creep(&r.grid, c.diffusion);
    println!(
        "shipped peak effective creep diffusivity {d_max:.4} against a bound of \
         {CREEP_MAX_EDGE_COEFF}, sub-steps {}",
        r.erosion.creep_substeps()
    );
    assert!(
        d_max > CREEP_MAX_EDGE_COEFF,
        "the shipped world's peak effective creep diffusivity is {d_max}, inside \
         the bound — if this ever becomes true the shipped goldens should not have \
         moved and the move needs re-justifying"
    );
    assert!(c.diffusion < CREEP_MAX_EDGE_COEFF);
    assert!(
        f64::from(r.erosion.creep_substeps()) >= d_max / CREEP_MAX_EDGE_COEFF,
        "the sub-step count does not cover the peak coefficient"
    );
}
