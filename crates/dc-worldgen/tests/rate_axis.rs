//! **RATE — the cadence axis, end to end** (journal/0123;
//! `docs/dependency-graph.md` E3; `material-behavior.md` § 5 *Cadence: order ×
//! rate × window*, RATIFIED 2026-07-24 from the user's `ideas.md` § *Pass
//! cadence* sketch).
//!
//! The unit tests prove the arithmetic (`deeptime::cadence`), that the roster
//! carries it (`deeptime::runner`), and that `dt` scales what it should
//! (`deeptime::erosion`). This file proves the two things only a whole world can
//! say:
//!
//! 1. **The empty table is the shipped world, bit for bit.** journal/0122 left a
//!    known-good fixed point — the sub-cycled creep operator's goldens — and the
//!    axis had to land against it without moving anything. That makes *"did RATE
//!    reproduce it"* a hash comparison rather than a judgement call, which is why
//!    this slice was sequenced after that one.
//! 2. **An authored cadence actually reaches the sim.** The mirror half, and the
//!    one that is easy to skip: a table that changed no bit anywhere would satisfy
//!    (1) perfectly and be a decoration. So worlds are generated with a pass
//!    re-rated and asserted to *differ* — and to still conserve mass, because the
//!    quickest way to fake a rate change is to lose material.
//!
//! Sized at the goldens' own extent (`golden_pregen`, `Extent::Small`): every
//! claim here is a per-pass scheduling fact or a per-cell arithmetic one, both
//! scale-free.

use dc_worldgen::deeptime::{
    self, Cadence, CadenceTable, DeepConfig, DeepOverrides, build_field_cfg_cadence,
    production_config_with, run_cells_with_cadence,
};
use dc_worldgen::pregen::Pregen;

mod providers_common;
use providers_common::{
    GOLDEN_RECORD, GOLDEN_SURFACE, SEED as GOLDEN_SEED, golden_pregen, record_fingerprint,
    surface_fingerprint,
};

fn production(pregen: &Pregen) -> DeepConfig {
    production_config_with(&pregen.grid, GOLDEN_SEED, &DeepOverrides::default())
}

/// **THE ACCEPTANCE TEST.** The RATE axis, driven with an empty cadence table,
/// reproduces the shipped world's surface and strata fingerprints exactly — the
/// same two constants `providers_golden.rs` asserts against, reached through the
/// same distillation, so the comparison needs no equivalence argument of its own.
///
/// Both hashes, and by name, because they see different things: `GOLDEN_SURFACE`
/// is `R`/`H`, so it catches a mis-scaled uplift or creep; `GOLDEN_RECORD` is the
/// strata units, so it catches an epoch that ran a different number of times. A
/// slice that threaded `dt` into the erosion phases could plausibly move either
/// one alone.
#[test]
fn an_empty_cadence_table_is_the_shipped_world_bit_for_bit() {
    let pregen = golden_pregen();
    let field = build_field_cfg_cadence(&pregen.grid, &production(&pregen), &CadenceTable::empty());

    let surface = surface_fingerprint(&field);
    let record = record_fingerprint(&field);
    println!("RATE, empty table: surface = {surface:#018X}  record = {record:#018X}");
    assert_eq!(
        surface, GOLDEN_SURFACE,
        "the RATE axis moved the surface planes: {surface:#018X} != \
         {GOLDEN_SURFACE:#018X} — an empty cadence table must be today's schedule"
    );
    assert_eq!(
        record, GOLDEN_RECORD,
        "the RATE axis moved the strata record: {record:#018X} != \
         {GOLDEN_RECORD:#018X}"
    );
}

/// **The mirror half: an authored cadence reaches the sim**, on both directions
/// of the axis, with the mass ledger re-asserted through each.
///
/// **Sub-turns (the fine direction).** Sub-turning the hillslope pass ×2 halves
/// its phase length and runs it twice per epoch, each turn seeing the cover the
/// previous one left. The *total* creep time is unchanged — that is
/// `Cadence::phase_total`'s invariant — but the operator is non-linear (a flux
/// limiter that binds on the cells with the most cover), so resolving the epoch
/// more finely is a real modelling difference and the world must move.
///
/// *Why the non-linearity is the justification and not a caveat:* if creep were
/// linear, sub-turning would be a no-op and this assertion could not exist. The
/// reason a per-pass temporal-resolution knob is worth having **is** that the
/// processes it governs are not linear — `material-behavior.md` § 5's own argument
/// for separate declared passes over a fused monolith (*"a monolith's
/// shared-cadence-with-scaled-rates is only equivalent for linear ones"*).
///
/// **Period (the coarse direction).** Firing the eolian agent every third epoch
/// runs it a third as often. Asserted separately because `period` is otherwise
/// only exercised by passes that were already coarse before RATE existed, which
/// proves nothing about the table.
///
/// One base run shared by both arms — the runs are the expensive part.
#[test]
fn an_authored_cadence_moves_the_world_and_still_closes_the_mass_ledger() {
    let pregen = golden_pregen();
    let cfg = production(&pregen);
    let base = run_cells_with_cadence(&pregen.grid, &cfg, true, &CadenceTable::empty());
    let base_print = surface_print(&base);

    for (what, table) in [
        (
            "hillslope creep sub-turned ×2",
            CadenceTable::empty().with("dc:deep/diffuse", Cadence::sub_turned(2)),
        ),
        (
            "the eolian agent at period 3",
            CadenceTable::empty().with("dc:deep/eolian", Cadence::every(3)),
        ),
    ] {
        let rated = run_cells_with_cadence(&pregen.grid, &cfg, true, &table);
        let print = surface_print(&rated);
        println!("{what}: surface {print:#018X} (base {base_print:#018X})");
        assert_ne!(
            print, base_print,
            "{what} changed no elevation anywhere — the authored cadence is not \
             reaching the runner"
        );

        // The falsifier for a "rate change" that is really a leak:
        // `Δ(ΣR + ΣH) = uplift + biotic` must still close.
        let residual = deeptime::total_mass(&rated.grid)
            - rated.mass_before
            - rated.uplift_total
            - rated.biotic_total;
        assert!(
            residual.abs() < 1.0,
            "{what}: mass leaked, residual {residual}"
        );
        // And the **forcing** pass integrated exactly the same amount of time:
        // its cadence was not authored, so nothing about it may move. Cadence is
        // per-pass, and a clock leaking across passes is precisely the bug this
        // axis could plausibly introduce.
        //
        // `thickening_total` is the term to assert, **not** `uplift_total`: on the
        // tectonic path the latter is the *isostatic injection* ΣΔR, which is a
        // response to how much bedrock erosion removed and therefore legitimately
        // moves when creep does. (An earlier draft asserted it and failed for that
        // reason — the ledger name says "uplift", the quantity is a feedback.)
        // `thickening_total` is the analytic forcing plane summed per firing: no
        // erosion term enters it.
        assert_eq!(
            rated.thickening_total.to_bits(),
            base.thickening_total.to_bits(),
            "{what}: re-rating one pass changed how much time the FORCING pass \
             integrated"
        );
    }
}

/// A bare FNV-1a over the two terrain planes — enough to say *"this world is not
/// that world"*, without distilling a whole `DeepField` for a comparison that only
/// needs inequality. The golden test above uses the real fingerprints.
fn surface_print(run: &deeptime::DeepRun) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in run.grid.r.iter().chain(run.grid.h.iter()) {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}
