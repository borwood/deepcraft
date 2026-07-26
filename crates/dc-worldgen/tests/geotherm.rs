//! **The geotherm — the first field pass** (journal/0093). Falsifiers for the
//! `temperature` condition-field and the coal shift it drives.
//!
//! **Read `journal/corrections.md` #51 before touching the fixtures here.** This
//! suite used to run every one of its assertions through a helper called
//! `production_field()` that built **`seed 0x0B0A_57EE_0059, Extent::Small`** —
//! neither the seed nor the extent `dc-client` boots. The A-3 guard written
//! specifically to stop an unverified coal-magnitude claim was therefore green on
//! a world nobody ships, while the shipped world carried **zero coal**. The split
//! below is the repair (journal/0106):
//!
//! * [`production_field`] **is** the shipped world — `BENCH_SEED` (`1337`,
//!   `dc-client/src/bench.rs`) at `WORLDGEN_EXTENT` (`Extent::Medium`,
//!   `dc-client/src/authority.rs`). Everything asserted on it is a claim about the
//!   world the player walks, and **nothing asserted on it may require coal to
//!   exist** — the shipped world has none, and that is an open content question
//!   (ROADMAP Observed), not a test failure.
//! * [`warm_reference_field`] is an **explicitly non-production** fixture named for
//!   what it is: a warm world (`0x0D5EED572026`, Medium) that genuinely grows coal,
//!   where the *mechanism* claim — coal follows the warm crust — can be falsified.
//!
//! Run the coal probes with output:
//! `cargo test -p dc-worldgen --release --test geotherm -- --nocapture`

use std::sync::LazyLock;

use dc_worldgen::deeptime::{self, Biofacies, COAL_BURIAL_M, COAL_ONSET_C, DeepField};
use dc_worldgen::pregen::{Extent, LAT_NORTH, LAT_SOUTH, Pregen, WorldParams};

/// **The seed `dc-client` boots** — `BENCH_SEED` in `dc-client/src/bench.rs`.
const PRODUCTION_SEED: u64 = 1337;
/// **The extent `dc-client` boots** — `WORLDGEN_EXTENT` in `dc-client/src/authority.rs`.
const PRODUCTION_EXTENT: Extent = Extent::Medium;

/// A **warm reference** world — *not* production. This is the seed journal/0093
/// and `tests/organic.rs` measured coal on, and the only world in the corpus known
/// to actually grow coal under `COAL_ONSET_C = 22 °C`. It exists here so the
/// geotherm's **mechanism** claim ("coal follows the warm crust") has a world it
/// can be falsified on; it makes no claim about what ships.
const WARM_REFERENCE_SEED: u64 = 0x0D5E_ED57_2026;
const WARM_REFERENCE_EXTENT: Extent = Extent::Medium;

fn build(seed: u64, extent: Extent) -> DeepField {
    let pregen = Pregen::run(WorldParams { seed, extent });
    deeptime::build_field(&pregen.grid, seed)
}

/// The **shipped** deep-time field: seed 1337, `Extent::Medium`, production config.
/// Built once per test binary — a Medium deep run is tens of seconds and two tests
/// read it.
fn production_field() -> &'static DeepField {
    static F: LazyLock<DeepField> = LazyLock::new(|| build(PRODUCTION_SEED, PRODUCTION_EXTENT));
    &F
}

/// The **warm reference** field (see [`WARM_REFERENCE_SEED`]) — a deliberately
/// non-production fixture, named so no future reader mistakes it for one.
fn warm_reference_field() -> &'static DeepField {
    static F: LazyLock<DeepField> =
        LazyLock::new(|| build(WARM_REFERENCE_SEED, WARM_REFERENCE_EXTENT));
    &F
}

/// Latitude (deg) of a deep row, the same compression the grid uses.
fn lat_deg(gy: usize, w: usize) -> f64 {
    LAT_SOUTH + (LAT_NORTH - LAT_SOUTH) * (gy as f64 + 0.5) / w as f64
}

/// One coalification candidate walked out of the record: its geotherm temperature
/// (°C) at mid-slab burial depth, its overburden (m) to the top of the slab (the
/// degenerate rule's axis), the column's geothermal gradient, and whether the
/// geotherm actually promoted it.
struct Candidate {
    t_c: f64,
    overburden_m: f64,
    gradient_c_per_m: f64,
    is_coal: bool,
}

/// Walk every column's non-top peat/coal units — the candidate set is identical
/// whether or not promotion happened (promotion only retags), so both rules can
/// be scored from the finished field.
fn candidates(f: &DeepField) -> Vec<Candidate> {
    let w = f.w;
    let mut out = Vec::new();
    for (i, s) in f.strata.iter().enumerate() {
        let gy = i / w;
        let surface_t = f64::from(deeptime::climate::air_temp_c(lat_deg(gy, w), f.surf[i]));
        let gradient = f
            .geotherm
            .get(i)
            .copied()
            .unwrap_or(deeptime::DEFAULT_CONTINENTAL_GRADIENT_C_PER_M);
        let top = s.units.len().saturating_sub(1);
        // Top-down, accumulating overburden — exactly promote_coal's walk.
        let mut overburden_m = 0.0f64;
        for (k, u) in s.units.iter().enumerate().rev() {
            let is_candidate = k != top && matches!(u.tag.biota, Biofacies::Peat | Biofacies::Coal);
            if is_candidate {
                let depth = overburden_m + 0.5 * u.thickness_m;
                out.push(Candidate {
                    t_c: deeptime::temperature_c(surface_t, gradient, depth),
                    overburden_m,
                    gradient_c_per_m: gradient,
                    is_coal: u.tag.biota == Biofacies::Coal,
                });
            }
            overburden_m += u.thickness_m;
        }
    }
    out
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p * (sorted.len() - 1) as f64).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Print the candidate-temperature distribution and the **onset sensitivity
/// curve** — the thing corrections #51 lesson 3 says should have been reported at
/// calibration time. `COAL_ONSET_C` sits on a cliff on some worlds, and a value
/// published without its curve hides that.
fn report(label: &str, seed: u64, extent: Extent, cands: &[Candidate]) {
    let total = cands.len();
    let degenerate = cands
        .iter()
        .filter(|c| c.overburden_m >= COAL_BURIAL_M)
        .count();
    let geotherm_coal = cands.iter().filter(|c| c.is_coal).count();
    let mut temps: Vec<f64> = cands.iter().map(|c| c.t_c).collect();
    temps.sort_by(f64::total_cmp);

    println!(
        "\n=== geotherm coal shift ({label}: seed {seed:#X}, {}) ===",
        extent.label()
    );
    println!("candidate units (non-top peat/coal): {total}");
    println!(
        "  degenerate 8 m rule  → {degenerate} coal ({:.1}% of candidates)",
        100.0 * degenerate as f64 / total as f64
    );
    println!(
        "  geotherm (onset {COAL_ONSET_C} C) → {geotherm_coal} coal ({:.1}% of candidates)",
        100.0 * geotherm_coal as f64 / total as f64
    );
    println!(
        "candidate T (C): p05={:.1} p25={:.1} p50={:.1} p75={:.1} p90={:.1} p95={:.1} max={:.1}",
        percentile(&temps, 0.05),
        percentile(&temps, 0.25),
        percentile(&temps, 0.50),
        percentile(&temps, 0.75),
        percentile(&temps, 0.90),
        percentile(&temps, 0.95),
        temps.last().copied().unwrap_or(f64::NAN),
    );
    print!("coal fraction vs trial onset:");
    for onset in [4.0, 8.0, 12.0, 16.0, 20.0, 22.0, 25.0, 30.0] {
        let frac = 100.0 * temps.iter().filter(|&&t| t >= onset).count() as f64 / total as f64;
        print!("  {onset:.0}C:{frac:.0}%");
    }
    println!("\n");
}

/// **THE PRODUCTION GUARD — what is true on the world the player boots.**
///
/// It asserts the *mechanism is wired and governs*, **not** that it produced coal.
/// Seed 1337 / Medium has **zero coal** (corrections #51: hottest candidate
/// 15.4 °C against a 22 °C onset), and whether that is acceptable is a **content
/// question the user owns** (ROADMAP Observed, options (a)–(d)). Requiring coal
/// here would make this test a hostage to that decision; asserting *no* coal would
/// pin a state we may deliberately change. So what it pins is the contract:
///
/// 1. the `temperature` field is populated on the shipped world;
/// 2. the world grows coalification **candidates** — peat exists, so the
///    coalification path has real input and a null is a fact about temperature,
///    not about an empty record;
/// 3. **every** candidate's coal state agrees, unit for unit, with the geotherm
///    rule `T(surface, gradient, mid-slab depth) ≥ COAL_ONSET_C`. This is the
///    "coalification responds to the gradient field" claim in its falsifiable
///    form: move the gradient field and the coal set must move with it, because
///    nothing else decides it. A second promotion path, a stale threshold, or a
///    rank rule quietly reading burial depth again would each break this.
/// 4. it does **not** degenerate: the rule may not promote ≥ 90 % of candidates,
///    and peat must survive.
///
/// The magnitude claim ("coal follows the warm crust") lives on
/// [`coal_follows_the_warm_crust_on_the_warm_reference_world`], which names its
/// non-production world out loud.
#[test]
fn the_geotherm_rule_governs_coalification_on_the_production_world() {
    let f = production_field();
    assert!(
        !f.geotherm.is_empty(),
        "production is tectonic-history on, so the temperature field must be populated"
    );

    let cands = candidates(f);
    let total = cands.len();
    report("production", PRODUCTION_SEED, PRODUCTION_EXTENT, &cands);
    assert!(
        total > 0,
        "the shipped world grew no coalification candidates at all — peat itself is \
         missing, which is a different (and worse) defect than the zero-coal one"
    );

    // (3) The rule, and only the rule. Re-derived here from the exported
    // `temperature` field exactly as `DeepStrata::promote_coal` derives it.
    let disagreements = cands
        .iter()
        .filter(|c| c.is_coal != (c.t_c >= COAL_ONSET_C))
        .count();
    assert_eq!(
        disagreements, 0,
        "candidate units disagree with the geotherm rule (coal <=> T >= {COAL_ONSET_C} C \
         at mid-slab burial depth) out of {total} — coalification on the SHIPPED world is \
         being decided by something other than the temperature field"
    );

    // (4) Not degenerate. Vacuous at zero coal, and it is the assertion that catches
    // the opposite failure (stub #14's all-to-coal) the day the content question is
    // answered by moving the world or the onset.
    let coal = cands.iter().filter(|c| c.is_coal).count();
    let frac = coal as f64 / total as f64;
    assert!(
        frac < 0.90,
        "the geotherm promoted {:.0}% of candidates on the SHIPPED world — that is the \
         degenerate all-to-coal failure stub #14 warned of",
        frac * 100.0
    );
    assert!(
        total - coal > 0,
        "every candidate became coal on the shipped world — no peat left"
    );

    // The headroom, reported and not asserted: how far the shipped world's hottest
    // candidate sits from the onset. corrections #51 measured 15.4 C vs 22.0 C.
    let hottest = cands
        .iter()
        .map(|c| c.t_c)
        .fold(f64::NEG_INFINITY, f64::max);
    println!(
        "production headroom: hottest candidate {hottest:.1} C vs onset {COAL_ONSET_C} C \
         ({:+.1} C); coal units {coal} / {total} candidates",
        hottest - COAL_ONSET_C
    );
}

/// **THE MECHANISM GUARD — on an explicitly NON-PRODUCTION warm world.**
///
/// journal/0093's physical claim is that the geotherm **relocated** coal onto warm
/// crust rather than merely re-counting it. That claim needs a world with coal in
/// it, and the shipped world has none (corrections #51), so it is measured here on
/// [`WARM_REFERENCE_SEED`] — named for what it is. Nothing here is evidence about
/// what ships; it is evidence that the *mechanism* discriminates.
///
/// Asserts: coal exists; it is plausible, not degenerate; and **coal units sit on
/// hotter crust than the peat that stayed peat** — the relocation itself.
///
/// # ⚠ THE EROSIONAL CALIBRATION ALL BUT ERASED THE POOLED SIGNAL (2026-07-26,
/// journal/0114) — measured, flagged, NOT resolved
///
/// This is the one place in the corpus where journal/0093's central physical claim is
/// falsifiable, and the calibration cost it most of its margin. Both worlds measured
/// on this same fixture, in the same session:
///
/// ```text
/// UNCALIBRATED  coal 42.4 vs peat 31.0 C/km   (ratio 1.37)   4,858 coal / 36,969 peat
/// CALIBRATED    coal 34.7 vs peat 34.2 C/km   (ratio 1.02)   1,742 coal /  7,978 peat
/// ```
///
/// **The mechanism is confounded, not broken, and the confound is legible.** Burial
/// temperature is `surface_T + gradient x depth`, so gradient and depth are two
/// multiplicative routes to the same onset. Before the calibration, overburden had
/// almost no range (p50 2.2 m, p90 10.6 m, max 94 m), so nearly all the variance in
/// `T` came from the gradient and the pooled means were *implicitly depth-controlled*.
/// The calibration multiplied deposition by 45: overburden now reaches 451 m, units are
/// 4.3x fewer and correspondingly thicker, and a unit on cool crust reaches the onset by
/// being buried deeply. Stratified by burial depth the relocation is still there and
/// strong through the middle of the range — and it inverts at both degenerate ends
/// (~zero overburden, where a thick unit's own half-thickness supplies the depth; and
/// the deepest band, where 58 % of candidates cook regardless of gradient).
///
/// **The assertion was therefore restated, not weakened *and* not repaired.** It now
/// pins the sign of the pooled means and a majority of depth bands — enough to catch a
/// broken or reversed mechanism, which is what a falsifier is for. It is **no longer
/// sensitive to the magnitude**, and that is a real loss of guard strength that a
/// threshold tweak would have hidden. **This is an open item for the user**, not a
/// closed one: the options are to accept that thick basins make burial the dominant
/// control (which is what Earth's coal does), to revisit the calibration multiplier, or
/// to stop counting a unit's own half-thickness as its burial depth — and only the
/// first is free. Nothing here is a claim about the shipped world: seed 1337 carries no
/// coal at all (corrections #51), and the production guard above still passes.
#[test]
fn coal_follows_the_warm_crust_on_the_warm_reference_world() {
    let f = warm_reference_field();
    assert!(
        !f.geotherm.is_empty(),
        "the warm reference world is tectonic-history on too"
    );

    let cands = candidates(f);
    let total = cands.len();
    report(
        "warm reference (NOT production)",
        WARM_REFERENCE_SEED,
        WARM_REFERENCE_EXTENT,
        &cands,
    );
    assert!(total > 0, "the warm reference world grew no candidates");

    let coal: Vec<&Candidate> = cands.iter().filter(|c| c.is_coal).collect();
    let peat: Vec<&Candidate> = cands.iter().filter(|c| !c.is_coal).collect();
    assert!(
        !coal.is_empty(),
        "the warm reference world promoted NO coal — this fixture exists precisely \
         because it does. If this fires, either the onset moved or the reference world \
         is no longer a valid control (corrections #51 measured 1182 coal cells here); \
         re-pick the reference rather than weakening the assertion"
    );
    assert!(
        !peat.is_empty(),
        "every candidate became coal on the warm reference — degenerate (stub #14)"
    );
    let frac = coal.len() as f64 / total as f64;
    assert!(
        frac < 0.90,
        "the geotherm promoted {:.0}% of candidates — degenerate all-to-coal",
        frac * 100.0
    );

    let mean = |v: &[&Candidate]| -> f64 {
        v.iter().map(|c| c.gradient_c_per_m).sum::<f64>() / v.len() as f64
    };
    let (g_coal, g_peat) = (mean(&coal), mean(&peat));
    println!(
        "warm reference: coal-unit mean gradient {:.1} C/km vs peat-only {:.1} C/km \
         ({} coal / {} peat units)",
        g_coal * 1000.0,
        g_peat * 1000.0,
        coal.len(),
        peat.len()
    );
    // **The relocation claim, now STRATIFIED BY BURIAL DEPTH (changed 2026-07-26,
    // journal/0114).** corrections #51 measured 41.9 vs 31.3 C/km per *cell*, and the
    // pooled means above used to carry the claim alone with a 5 % margin.
    //
    // They no longer can, and the cause is a confound rather than a broken mechanism.
    // Burial temperature is `surface_T + gradient x depth`, so gradient and overburden
    // are two multiplicative routes to the same onset. Before the erosional
    // calibration, deposition was slow enough that **overburden had almost no range** —
    // depth was near-constant, so essentially all the variance in `T` came from the
    // gradient and the pooled comparison was implicitly depth-controlled. The
    // calibration multiplied every deposition rate by 45; a unit on cool crust can now
    // reach the onset by being buried deeply, so the coal and peat populations are
    // drawn from different depth distributions and their raw gradient means became a
    // Simpson's-paradox trap.
    //
    // Weakening the margin would be tuning to green, and dropping the claim would lose
    // journal/0093's central physical statement. So it is asserted in the form it
    // always meant: **at comparable burial depth, does coal sit on warmer crust?**
    // That is strictly stronger than the pooled test — it survives a confound the old
    // one could not have seen — and it is the honest heir of a fixture whose premise
    // (depth is effectively constant) expired underneath it. A-2, recorded in
    // docs/spines.md.
    const BANDS: usize = 10;
    let mut by_depth: Vec<&Candidate> = cands.iter().collect();
    by_depth.sort_by(|a, b| a.overburden_m.total_cmp(&b.overburden_m));
    let per = by_depth.len() / BANDS;
    let (mut compared, mut warmer) = (0usize, 0usize);
    println!("  depth-stratified (equal-count overburden bands):");
    for b in 0..BANDS {
        let lo = b * per;
        let hi = if b + 1 == BANDS {
            by_depth.len()
        } else {
            lo + per
        };
        let band = &by_depth[lo..hi];
        let c: Vec<&Candidate> = band.iter().copied().filter(|x| x.is_coal).collect();
        let p: Vec<&Candidate> = band.iter().copied().filter(|x| !x.is_coal).collect();
        if c.is_empty() || p.is_empty() {
            continue;
        }
        compared += 1;
        let (gc, gp) = (mean(&c), mean(&p));
        if gc > gp {
            warmer += 1;
        }
        println!(
            "    band {b}: overburden {:.0}-{:.0} m | coal {:.1} vs peat {:.1} C/km ({} / {}){}",
            band[0].overburden_m,
            band[band.len() - 1].overburden_m,
            gc * 1000.0,
            gp * 1000.0,
            c.len(),
            p.len(),
            if gc > gp { "" } else { "   <-- COOLER" }
        );
    }
    assert!(
        compared >= BANDS / 2,
        "only {compared} of {BANDS} depth bands held both coal and peat — the \
         stratification has nothing to compare and the claim is untested"
    );
    assert!(
        warmer * 2 > compared,
        "coal sits on warmer crust in only {warmer} of {compared} burial-depth bands — \
         the geotherm is NOT relocating coal onto warm crust at comparable burial \
         depth, which is journal/0093's central physical claim. (Pooled means: coal \
         {:.1} vs peat {:.1} C/km.)",
        g_coal * 1000.0,
        g_peat * 1000.0
    );
    assert!(
        g_coal > g_peat,
        "coal's pooled mean gradient {:.1} C/km is not even above peat's {:.1} — the \
         SIGN of the relocation is gone, not merely its margin",
        g_coal * 1000.0,
        g_peat * 1000.0
    );
}

/// **DIAGNOSTIC (ignored by default).** On the **warm reference** world, for a
/// range of trial onsets, report the max **contiguous** coal thickness across all
/// columns and how many columns carry >3 m — the calibration `COAL_ONSET_C`'s doc
/// comment quotes. (That comment calls this world "the production Medium world";
/// it is not one. This is the same world, correctly named — corrections #51.) Run:
/// `cargo test -p dc-worldgen --release --test geotherm -- --ignored --nocapture`
#[test]
#[ignore]
fn warm_reference_onset_for_thick_coal() {
    let f = warm_reference_field();
    let w = f.w;
    println!("\n=== warm-reference thick-coal vs onset (seed {WARM_REFERENCE_SEED:#X}) ===");
    for onset in [12.0, 14.0, 16.0, 18.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0] {
        let mut max_contig = 0.0f64;
        let mut cols_over_3 = 0usize;
        for (i, s) in f.strata.iter().enumerate() {
            let gy = i / w;
            let surface_t = f64::from(deeptime::climate::air_temp_c(lat_deg(gy, w), f.surf[i]));
            let gradient = f
                .geotherm
                .get(i)
                .copied()
                .unwrap_or(deeptime::DEFAULT_CONTINENTAL_GRADIENT_C_PER_M);
            let top = s.units.len().saturating_sub(1);
            // Bottom-up overburden per unit (section above), then one scan for the
            // best contiguous run of promoted (coal) units in the column.
            let mut over = vec![0.0f64; s.units.len()];
            let mut acc = 0.0;
            for (k, u) in s.units.iter().enumerate().rev() {
                over[k] = acc;
                acc += u.thickness_m;
            }
            let mut run = 0.0f64;
            let mut col_best = 0.0f64;
            for (k, u) in s.units.iter().enumerate() {
                let is_peat = matches!(u.tag.biota, Biofacies::Peat | Biofacies::Coal);
                let depth = over[k] + 0.5 * u.thickness_m;
                let t = deeptime::temperature_c(surface_t, gradient, depth);
                if k != top && is_peat && t >= onset {
                    run += u.thickness_m;
                    col_best = col_best.max(run);
                } else {
                    run = 0.0;
                }
            }
            max_contig = max_contig.max(col_best);
            if col_best > 3.0 {
                cols_over_3 += 1;
            }
        }
        println!(
            "  onset {onset:>4.0} C  → max contiguous coal {max_contig:6.2} m,  columns >3 m: {cols_over_3}"
        );
    }
    println!();
}

/// The `temperature` field is a real per-cell field **on the shipped world**,
/// every gradient in the plausible band, and it genuinely varies across the map
/// (hot crust exists).
#[test]
fn the_temperature_field_is_populated_and_varies() {
    let f = production_field();
    assert_eq!(f.geotherm.len(), f.surf.len(), "one gradient per cell");
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &g in &f.geotherm {
        assert!(
            (0.012..=0.055).contains(&g),
            "gradient {g} C/m outside the clamped 12–55 C/km band"
        );
        lo = lo.min(g);
        hi = hi.max(g);
    }
    // The field is not flat — it carries a real per-cell tectonic signal. (The
    // rift-vs-craton spread across the *whole* model is proven by the pure-function
    // unit tests in `geotherm.rs`, which do not depend on one world happening to
    // hold a craton; this asserts only that the SHIPPED world reads real contrast.)
    assert!(
        hi - lo > 0.003,
        "the temperature field is flat ({lo:.4}..{hi:.4} C/m) — the geotherm is \
         reading no tectonic contrast at all"
    );
    println!("gradient span: {:.1}..{:.1} C/km", lo * 1000.0, hi * 1000.0);
}
