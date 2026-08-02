//! S10 biotic-layer falsifiers: determinism (byte-identical planes AND records
//! under a repeated seed, and scalar-vs-parallel identity), the abiotic path's
//! byte-identity with the pre-S10 engine, the recorder invariants under the
//! pedogenic overprint, the mass ledger with biotic carbon as an external input,
//! and the presence of the four target read-quality signals.
//!
//! Measured numbers live in docs/spikes/S10-results.md; these are the falsifiers.

use dc_core::materials::geology::{GeologySet, vanilla};
use dc_worldgen::deeptime::{
    self, Aridity, Biofacies, BurialColumn, COAL_MIN_M, DeepConfig, DeepGrid, DeepStrata, DepEnv,
    DepTag, DepositCtx, EnergyBand, Eolian, MemberCtx,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn cfg(seed: u64, cell_m: f64, iterations: u32, biotic: bool) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m,
        iterations,
        remarch_interval: 20,
        record: true,
        biotic,
        ..DeepConfig::default()
    }
}

fn small(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

fn medium(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Medium,
    })
}

#[test]
fn biotic_same_seed_is_byte_identical_planes_and_records() {
    let pregen = small(SEED);
    let c = cfg(SEED, 1000.0, 60, true);
    let a = deeptime::run(&pregen, &c);
    let b = deeptime::run(&pregen, &c);
    assert_eq!(a.grid.r, b.grid.r, "bedrock plane must be byte-identical");
    assert_eq!(a.grid.h, b.grid.h, "alluvium plane must be byte-identical");
    assert_eq!(
        a.grid.strata, b.grid.strata,
        "strata records (with biotic tags) must be byte-identical"
    );
    assert_eq!(
        a.grid.bio_weather, b.grid.bio_weather,
        "biotic weathering modifiers must be byte-identical"
    );
    assert_eq!(
        a.grid.bio_resist, b.grid.bio_resist,
        "biotic resistance modifiers must be byte-identical"
    );
    assert_eq!(
        a.biotic_total.to_bits(),
        b.biotic_total.to_bits(),
        "the biotic mass ledger must be byte-identical"
    );
}

#[test]
fn biotic_parallel_equals_scalar_byte_identical() {
    // The biotic step is a pure per-cell function of frozen inputs, so the
    // data-parallel driver must reproduce the scalar one to the bit — the same
    // contract the S9b erosion phases hold. Use a grid past the rayon fork floor.
    let pregen = small(SEED);
    let c = cfg(SEED, 200.0, 20, true);
    let scalar = deeptime::run_with(&pregen, &c, false);
    let parallel = deeptime::run_with(&pregen, &c, true);
    assert!(
        scalar.grid.w * scalar.grid.w >= 1 << 15,
        "grid must be past the parallel fork floor to exercise the rayon path"
    );
    assert_eq!(scalar.grid.r, parallel.grid.r, "bedrock plane");
    assert_eq!(scalar.grid.h, parallel.grid.h, "alluvium plane");
    assert_eq!(
        scalar.grid.strata, parallel.grid.strata,
        "strata records: parallel must be byte-identical to scalar"
    );
    assert_eq!(
        scalar.biotic_total.to_bits(),
        parallel.biotic_total.to_bits(),
        "biotic ledger: parallel must be byte-identical to scalar"
    );
}

#[test]
fn a_different_seed_diverges_biotically() {
    let a = deeptime::run(&small(SEED), &cfg(SEED, 1000.0, 60, true));
    let b = deeptime::run(&small(SEED ^ 0xABCD), &cfg(SEED ^ 0xABCD, 1000.0, 60, true));
    assert_ne!(
        a.grid.strata, b.grid.strata,
        "a different world must grow a different biotic record"
    );
}

#[test]
fn biotic_off_leaves_the_record_purely_mineral() {
    // The abiotic path must be byte-identical to the pre-S10 engine. The two
    // observable guarantees: the modifier planes stay EMPTY (so the erosion
    // kernels read their identity values `1.0` / `0.0`, and `x * 1.0 == x`
    // exactly), and every recorded unit carries the default `Mineral` facies —
    // i.e. the new merge-key axis has exactly one inhabited value, so units
    // merge exactly as they did before.
    let run = deeptime::run(&small(SEED), &cfg(SEED, 1000.0, 60, false));
    assert!(
        run.grid.bio_weather.is_empty() && run.grid.bio_resist.is_empty(),
        "biotic-off must not allocate modifier planes"
    );
    assert_eq!(run.biotic_total, 0.0, "biotic-off must add no biotic mass");
    assert!(
        run.biota.is_none(),
        "biotic-off must hold no community state"
    );
    for s in &run.grid.strata {
        for u in &s.units {
            assert_eq!(
                u.tag.biota,
                Biofacies::Mineral,
                "biotic-off must record only mineral units"
            );
        }
    }
}

#[test]
fn recorder_total_equals_alluvium_with_biotic_on() {
    // The pedogenic overprint adds thickness to an existing unit and can merge
    // it downward; both must preserve `sum(units) == H` exactly, or the record
    // stops mirroring the alluvium plane.
    let run = deeptime::run(&small(SEED), &cfg(SEED, 1000.0, 80, true));
    let mut worst = 0.0f64;
    for (i, s) in run.grid.strata.iter().enumerate() {
        worst = worst.max((s.total_m() - run.grid.h[i]).abs());
    }
    assert!(
        worst < 1e-6,
        "recorder invariant sum(units)==H violated under the biotic layer, worst {worst}"
    );
}

#[test]
fn mass_ledger_accounts_for_biotic_carbon() {
    // Photosynthesis fixes carbon from the air, so buried organic matter is a
    // genuine EXTERNAL mass input — like uplift, not a conservation violation.
    // The ledger must therefore close as uplift + biotic input.
    let run = deeptime::run(&small(SEED), &cfg(SEED, 1000.0, 80, true));
    let after = deeptime::total_mass(&run.grid);
    let residual = after - run.mass_before - run.uplift_total - run.biotic_total;
    assert!(
        residual.abs() < 1.0,
        "mass leaked: residual {residual} (uplift {}, biotic {})",
        run.uplift_total,
        run.biotic_total
    );
    assert!(
        run.biotic_total > 0.0,
        "the biotic layer must actually bury organic matter"
    );
}

/// Count columns carrying each of the four S10 target signals.
fn signal_columns(grid: &DeepGrid) -> (usize, usize, usize, usize) {
    let mut coal = 0;
    let mut paleosol = 0;
    let mut charcoal = 0;
    let mut retro = 0;
    for s in &grid.strata {
        if s.coal_seams(COAL_MIN_M) > 0 {
            coal += 1;
        }
        if s.paleosols() > 0 {
            paleosol += 1;
        }
        if s.charcoal_bands() > 0 {
            charcoal += 1;
        }
        if s.retro_surfaces() > 0 {
            retro += 1;
        }
    }
    (coal, paleosol, charcoal, retro)
}

#[test]
fn the_four_target_signals_reach_the_record() {
    // The read-quality falsifier: a full-length A-tier run must write all four
    // signals ecology.md § 3 asks for. This is the test that fails if a future
    // change quietly kills coal swamps or the fire regime.
    let run = deeptime::run(&medium(SEED), &cfg(SEED, 1000.0, 200, true));
    let (coal, paleosol, charcoal, retro) = signal_columns(&run.grid);
    assert!(coal > 0, "no coal seam reached the record");
    assert!(
        paleosol > 0,
        "no buried soil horizon (paleosol) reached the record"
    );
    assert!(charcoal > 0, "no charcoal band reached the record");
    assert!(retro > 0, "no retrogressive surface reached the record");
}

#[test]
fn biology_changes_the_landscape_it_grows_on() {
    // Biology is an EROSION term, not just an annotation (ecology.md § 1): root
    // cohesion resists hillslope creep and root acids accelerate weathering, so
    // a biotic run must produce a measurably different surface — otherwise the
    // lagged coupling is decorative and the cost buys only tags.
    let pregen = small(SEED);
    let abiotic = deeptime::run(&pregen, &cfg(SEED, 1000.0, 120, false));
    let biotic = deeptime::run(&pregen, &cfg(SEED, 1000.0, 120, true));
    let diff = abiotic
        .grid
        .r
        .iter()
        .zip(&biotic.grid.r)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    assert!(
        diff > 1.0,
        "the biotic run's bedrock surface is indistinguishable from the abiotic \
         one (max diff {diff} m) — the erosion/weathering feedback is not wired"
    );
}

// ---------------------------------------------------------------------------
// Burial diagenesis: the axis (journal/0063).

/// A column context for [`DeepStrata::promote_coal`] set to the **degenerate
/// 1 °C/m geotherm** (0 °C surface, gradient 1 °C/m) — under which the geotherm
/// temperature at a unit's mid-depth is numerically its burial depth in metres,
/// so an onset in "metres" behaves exactly as the retired identity did. This
/// keeps the axis falsifiers below reading as pure burial-depth tests.
/// The vanilla content set these coal falsifiers pick members from. Coal ships
/// **one** member, so which member `promote_coal` lands on is not what is under
/// test here — that it *re-picks under the burial context at all* is (P11 slice
/// 1); the axis assertions below are about `Biofacies`, unchanged.
fn member_ctx(set: &GeologySet) -> DepositCtx<'_> {
    MemberCtx::new(set, SEED, 0).at(
        0,
        dc_core::materials::geology::FormationContext {
            temp_c: 0.0,
            precip: 0.5,
            depth_m: 0.0,
        },
    )
}

fn degenerate_column() -> BurialColumn {
    BurialColumn {
        index: 0,
        gx: 0,
        gy: 0,
        surface_temp_c: 0.0,
        gradient_c_per_m: 1.0,
    }
}

fn organic_tag(biota: Biofacies) -> DepTag {
    DepTag {
        env: DepEnv::Subaerial,
        aridity: Aridity::Humid,
        energy: EnergyBand::Low,
        biota,
        eolian: Eolian::None,
    }
}

/// **The axis falsifier.** Coal is made by burial, not by duration. Until
/// journal/0063 `promote_coal` tested the seam's own *thickness*, which is a
/// statement about how long the swamp lasted; `CLASS_ORGANIC_COAL`'s contract
/// has always said the depth axis is the rank axis, and the burial depth was
/// derivable from the record all along (`Σ` of the overlying units).
///
/// The record below is constructed so the two rules give *opposite* answers on
/// both units, which is the only way to pin an axis rather than a threshold: the
/// old rule would have promoted the thick shallow bed and refused the thin deep
/// one.
#[test]
fn coal_promotion_reads_burial_depth_not_seam_thickness() {
    let mut s = DeepStrata::default();
    // Bottom-up: a THIN peat, buried under 21 m of section, then a THICK peat
    // under only 1 m, then nothing — the second peat is near the living surface.
    s.deposit(organic_tag(Biofacies::Peat), 0.05, 0);
    s.deposit(organic_tag(Biofacies::Mineral), 20.0, 0);
    s.deposit(organic_tag(Biofacies::Peat), 9.0, 0);
    s.deposit(organic_tag(Biofacies::Mineral), 1.0, 0);
    let before = s.total_m();

    let col = degenerate_column();
    s.promote_coal(col, 8.0, &member_ctx(&vanilla()));

    let biota: Vec<Biofacies> = s.units.iter().map(|u| u.tag.biota).collect();
    assert_eq!(
        biota[0],
        Biofacies::Coal,
        "a 5 cm bed under 21 m of section is COAL — the old thickness rule \
         (>= 0.4 m) would have left it peat forever"
    );
    assert_eq!(
        biota[2],
        Biofacies::Peat,
        "a 9 m bed under 1 m of section is still PEAT — the old thickness rule \
         would have promoted it on the strength of the swamp's duration alone"
    );
    // Promotion is a retagging, never a remassing: the finalize invariant.
    assert_eq!(
        s.total_m().to_bits(),
        before.to_bits(),
        "promote_coal moved mass"
    );
}

/// The topmost unit is the **living surface**, and it is never coal — at any
/// threshold, including the degenerate `0.0` where the burial rule would
/// otherwise promote it on `0.0 >= 0.0`. That is the one value at which the
/// invariant does *not* fall out of the arithmetic, which is exactly why it is
/// the value worth testing.
#[test]
fn the_living_surface_is_never_coal_whatever_the_threshold() {
    let mut s = DeepStrata::default();
    s.deposit(organic_tag(Biofacies::Peat), 40.0, 0);
    let col = degenerate_column();
    s.promote_coal(col, 0.0, &member_ctx(&vanilla()));
    assert_eq!(s.units[0].tag.biota, Biofacies::Peat);

    // And with something above it, the same peat IS coal at zero threshold.
    s.deposit(organic_tag(Biofacies::Mineral), 0.1, 0);
    s.promote_coal(col, 0.0, &member_ctx(&vanilla()));
    assert_eq!(s.units[0].tag.biota, Biofacies::Coal);
}
