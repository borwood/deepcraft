//! **The joint supply + transport calibration** (journal/0114, stubs #24,
//! corrections #56/#59).
//!
//! journal/0111 measured this world's catchment-averaged denudation at
//! **0.0110 m/Myr** against the ratified 500 Myr Phanerozoic register: 9× slower
//! than the slowest landscape ever measured on Earth, 493× below the global `10Be`
//! outcrop median, stripping **5.48 m** where a real craton strips 5–10 km. The
//! shape of the landscape was right and the clock was wrong.
//!
//! The repair is **one multiplier over four rate constants** — `weathering`,
//! `diffusion`, `k_transport`, `k_bedrock` — because the same entry measured that
//! neither the supply side nor the transport side pays alone. They are coupled
//! through the cover taper `exp(−H/H*)`, the only term in the system carrying an
//! absolute length: raise supply alone and the regolith made shields the rock that
//! made it, raise transport alone and there is nothing to carry.
//!
//! **⚠ THE CALIBRATION IS BUILT AND OFF, and this suite is what keeps it honest.**
//! `examples/denudation_probe.rs` found the published band unreachable at *any*
//! multiplier, and found that turning the flag on opens deep closed depressions the
//! incision clamp does not hold (0 pits at 1×, 44 at 5×, 148 at 45×). So the shipped
//! world is untouched and the calibrated one is a **pinned, reachable second path** —
//! the mirror of the single-receiver / scalar-load / anonymous-creep fixed points,
//! except that those pin a past and this pins a future.
//!
//! This suite pins the five things that had to be true:
//!
//! 1. the **identity pair** — off is the shipped world bit for bit, on moves it and
//!    lands on its own pinned constants;
//! 2. the **derivation identity** — the calibrated world *is* a uniform
//!    [`EROSION_CALIBRATION`]× erosion budget on the raw constants, which is what
//!    makes the probe's ladder a measurement OF that world rather than beside it;
//! 3. **`erosion_budget` reaches `diffusion`** — the stubs #24 falsifier, which
//!    would have failed on every commit before this one;
//! 4. the calibration moves **exactly** those four rates and nothing else;
//! 5. production is honestly **uncalibrated**, so nobody flips the default without
//!    noticing that a named test says it should not be flipped yet.
//!
//! Production-scale numbers — D1, D3, the balance ratio and the published band —
//! live in `examples/denudation_probe.rs`, which is the acceptance instrument.

use dc_worldgen::deeptime::{
    DeepConfig, DeepOverrides, EROSION_CALIBRATION, build_field, build_field_with,
    production_config, production_config_with, scale_erosion_rates,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

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

fn calibrated() -> DeepOverrides {
    DeepOverrides {
        calibrated_rates: Some(true),
        ..DeepOverrides::default()
    }
}

// ---------------------------------------------------------------------------
// 1. The identity pair.

/// **Both halves, because either alone is gameable.** A test that only asserted
/// "off is the pre-calibration world" would pass against a calibration that does
/// nothing; a test that only asserted "on is different" would pass against one that
/// broke something.
///
/// Both halves are pinned to *named constants*, which is what makes this a
/// cross-commit claim in both directions: the OFF half against the shipped goldens
/// (the calibration must not have leaked into production), and the ON half against
/// `GOLDEN_SURFACE_CALIBRATED` (the world behind the flag must not drift unmeasured
/// while it waits for its flip).
#[test]
fn calibrated_rates_off_is_production_and_on_moves_it() {
    let pregen = small_world();
    let off = build_field(&pregen.grid, SEED);
    let on = build_field_with(&pregen.grid, SEED, &calibrated());

    let off_surface = providers_common::surface_fingerprint(&off);
    let off_record = providers_common::record_fingerprint(&off);
    assert_eq!(
        off_surface,
        providers_common::GOLDEN_SURFACE,
        "the calibration is OFF in production, so the shipped world must still be the \
         shipped world: {off_surface:#018X}"
    );
    assert_eq!(off_record, providers_common::GOLDEN_RECORD);

    let on_surface = providers_common::surface_fingerprint(&on);
    let on_record = providers_common::record_fingerprint(&on);
    println!("calibrated surface = {on_surface:#018X}");
    println!("calibrated record  = {on_record:#018X}");
    assert_ne!(
        on_surface, off_surface,
        "the calibration did not move the surface — a {EROSION_CALIBRATION}x change to every \
         erosion rate in the engine reached no terrain, so it is not wired"
    );
    assert_eq!(
        on_surface,
        providers_common::GOLDEN_SURFACE_CALIBRATED,
        "the CALIBRATED world moved without its constant moving: {on_surface:#018X}. \
         It is pinned so the eventual flip is a diff and not a surprise."
    );
    assert_eq!(on_record, providers_common::GOLDEN_RECORD_CALIBRATED);
}

// ---------------------------------------------------------------------------
// 2. The derivation identity.

/// **The calibrated world IS a uniform erosion budget on the raw constants** — bit
/// for bit, not approximately.
///
/// This is the load-bearing test of the whole slice, and it is load-bearing for a
/// reason that is easy to miss. `examples/denudation_probe.rs` derives
/// [`EROSION_CALIBRATION`] by sweeping a **uniform multiplier over the production
/// config** and reading what each row does. That sweep is only evidence about the
/// world behind the flag if the row at `EROSION_CALIBRATION` and that world are the
/// same world.
///
/// They are, structurally: `production_config_with` applies the calibration and then
/// the budget, both through the same [`scale_erosion_rates`], so the two paths run
/// an identical multiply on identical operands in an identical order and IEEE-754
/// gives an identical result. But "structurally" is an argument, and an argument is
/// not a fixed point — a later slice that reorders those two steps, or gives the
/// budget its own scaling again, breaks the derivation silently and leaves the
/// probe's table quietly describing a different world. This test is what makes that
/// impossible.
///
/// *Why it is scale-free.* It is a statement about arithmetic on a config struct,
/// not about a landscape: the same two multiplies happen per rate at any world size.
/// The smallest world that runs every phase proves it.
#[test]
fn the_calibrated_world_is_a_uniform_erosion_budget_on_the_raw_rates() {
    let pregen = small_world();

    // First at the config level, where the claim is exact and legible.
    let shipped = production_config_with(&pregen.grid, SEED, &calibrated());
    let rebuilt = production_config_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            erosion_budget: Some(EROSION_CALIBRATION),
            ..DeepOverrides::default()
        },
    );
    for (name, a, b) in [
        ("weathering", shipped.weathering, rebuilt.weathering),
        ("diffusion", shipped.diffusion, rebuilt.diffusion),
        ("k_transport", shipped.k_transport, rebuilt.k_transport),
        ("k_bedrock", shipped.k_bedrock, rebuilt.k_bedrock),
    ] {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "{name}: the shipped calibration ({a}) is not bit-identical to a uniform \
             {EROSION_CALIBRATION}x erosion budget on the raw rate ({b}) — the probe's \
             derivation ladder no longer describes the world it claims to"
        );
    }

    // Then at the world level, which is what the probe actually measures.
    let a = build_field_with(&pregen.grid, SEED, &calibrated());
    let b = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            erosion_budget: Some(EROSION_CALIBRATION),
            ..DeepOverrides::default()
        },
    );
    assert_eq!(
        providers_common::surface_fingerprint(&a),
        providers_common::surface_fingerprint(&b),
        "the two paths agree on every rate constant and disagree on the world"
    );
    assert_eq!(
        providers_common::record_fingerprint(&a),
        providers_common::record_fingerprint(&b),
    );
}

// ---------------------------------------------------------------------------
// 3. The stubs #24 falsifier.

/// **The erosion budget reaches the process that does the eroding.**
///
/// stubs #24 recorded a knob documented as *"**the** TERRAIN (erosion) amplitude"*
/// that scaled `weathering`, `k_transport` and `k_bedrock` and **not** `diffusion` —
/// the process carrying **96 %** of this world's denudation. Measured, production
/// untouched: at 100× it moved catchment-averaged denudation by **1.4×**. A knob
/// that cannot move the quantity it is named after is a summary wearing an
/// authority's clothes.
///
/// **This test would have failed on every commit before journal/0114**, which is the
/// property that makes it a falsifier rather than a description. It is deliberately
/// a config-level assertion: the defect was one missing line in a list, and the
/// cheapest test that can see a missing line is one that reads the list.
#[test]
fn the_erosion_budget_reaches_every_rate_including_diffusion() {
    let pregen = small_world();
    let base = production_config(&pregen.grid, SEED);
    let doubled = production_config_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            erosion_budget: Some(2.0),
            ..DeepOverrides::default()
        },
    );
    for (name, a, b) in [
        ("weathering", base.weathering, doubled.weathering),
        ("diffusion", base.diffusion, doubled.diffusion),
        ("k_transport", base.k_transport, doubled.k_transport),
        ("k_bedrock", base.k_bedrock, doubled.k_bedrock),
    ] {
        assert_eq!(
            (a * 2.0).to_bits(),
            b.to_bits(),
            "a 2x erosion budget did not reach `{name}` ({a} -> {b}) — the amplitude \
             knob does not cover the rate it claims to"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. The scope, stated as an exclusion.

/// **The calibration moves exactly four rates and nothing else.**
///
/// The positive half (those four moved) is covered above. This is the negative half,
/// and it is the one that catches the plausible future mistake: someone folding
/// `wave_erosion` or `frost_weathering_gain` into [`scale_erosion_rates`] because
/// "they are erosion too". They are the **agent magnitudes** — explicitly unratified
/// appearance numbers the user judges live, station by station — and each is a rate
/// at a *place* rather than a term in the land-wide budget. Smuggling four
/// appearance calls into one calibration is exactly the move `production_config`'s
/// full-agents comment forbids.
///
/// Written as a total `Debug` comparison after restoring the four scaled fields by
/// hand, so it cannot be defeated by adding a field: any *other* field the scaling
/// touched shows up as a diff.
#[test]
fn the_calibration_scales_exactly_the_four_rate_constants() {
    let pregen = small_world();
    let base = production_config(&pregen.grid, SEED);
    let mut scaled = base;
    scale_erosion_rates(&mut scaled, 7.0);
    // Restore the four the scaling is *allowed* to move, to their pre-scaling bits.
    scaled.weathering = base.weathering;
    scaled.diffusion = base.diffusion;
    scaled.k_transport = base.k_transport;
    scaled.k_bedrock = base.k_bedrock;
    assert_eq!(
        format!("{base:?}"),
        format!("{scaled:?}"),
        "scale_erosion_rates moved a field outside the four rate constants — \
         the calibration's scope has grown, and an agent magnitude is an \
         appearance call the user owns"
    );
}

/// **The `Default` config does not claim to be calibrated.** The raw constants in
/// `DeepConfig::default()` are the uncalibrated ones, and the flag says so; only
/// `production_config` applies the multiplier and sets it. A default that claimed
/// otherwise would be a summary disagreeing with its own authority.
#[test]
fn a_default_config_is_honestly_uncalibrated_and_production_is_not() {
    let pregen = small_world();
    let raw = DeepConfig::default();
    assert!(
        !raw.calibrated_rates,
        "DeepConfig::default() claims calibrated rates while holding the raw constants"
    );
    let prod = production_config(&pregen.grid, SEED);
    assert!(
        !prod.calibrated_rates,
        "the calibration is OFF in production until the pit defect it exposed is          fixed (journal/0114) — if this fires, that flip needs its own evidence"
    );
    let cal = production_config_with(&pregen.grid, SEED, &calibrated());
    assert!(cal.calibrated_rates);
    assert_eq!(
        (raw.weathering * EROSION_CALIBRATION).to_bits(),
        cal.weathering.to_bits(),
        "the calibrated weathering is not the raw default times EROSION_CALIBRATION"
    );
    assert_eq!(
        (raw.diffusion * EROSION_CALIBRATION).to_bits(),
        cal.diffusion.to_bits(),
        "the calibrated diffusion is not the raw default times EROSION_CALIBRATION"
    );
}
