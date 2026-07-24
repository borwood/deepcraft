//! **The geotherm — the first field pass** (journal/0093). Falsifiers for the
//! `temperature` condition-field and the coal shift it drives.
//!
//! Run the coal-shift probe with output:
//! `cargo test -p dc-worldgen --release --test geotherm -- --nocapture`

use dc_worldgen::deeptime::{self, Biofacies, COAL_BURIAL_M, COAL_ONSET_C, DeepField};
use dc_worldgen::pregen::{Extent, LAT_NORTH, LAT_SOUTH, Pregen, WorldParams};

const SEED: u64 = 0x0B0A_57EE_0059;

fn production_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    deeptime::build_field(&pregen.grid, SEED)
}

/// Latitude (deg) of a deep row, the same compression the grid uses.
fn lat_deg(gy: usize, w: usize) -> f64 {
    LAT_SOUTH + (LAT_NORTH - LAT_SOUTH) * (gy as f64 + 0.5) / w as f64
}

/// One coalification candidate walked out of the record: its geotherm temperature
/// (°C) at mid-slab burial depth, and its overburden (m) to the top of the slab
/// (the degenerate rule's axis), and whether the geotherm actually promoted it.
struct Candidate {
    t_c: f64,
    overburden_m: f64,
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

/// **The coal shift — the walk reason.** Report coal before (the retired
/// degenerate 8 m burial rule) vs after (the geotherm), plus the candidate-unit
/// temperature distribution the onset is calibrated against. Asserts only that
/// coal is **plausible, not degenerate**: some coal, but far from all candidates.
#[test]
fn the_geotherm_coal_shift_is_plausible_not_degenerate() {
    let f = production_field();
    assert!(
        !f.geotherm.is_empty(),
        "production is tectonic-history on, so the temperature field must be populated"
    );

    let cands = candidates(&f);
    let total = cands.len();
    assert!(
        total > 0,
        "the world grew no coalification candidates at all"
    );

    let degenerate_coal = cands
        .iter()
        .filter(|c| c.overburden_m >= COAL_BURIAL_M)
        .count();
    let geotherm_coal = cands.iter().filter(|c| c.is_coal).count();

    let mut temps: Vec<f64> = cands.iter().map(|c| c.t_c).collect();
    temps.sort_by(f64::total_cmp);

    println!("\n=== geotherm coal shift (production Small, seed {SEED:#X}) ===");
    println!("candidate units (non-top peat/coal): {total}");
    println!(
        "  degenerate 8 m rule  → {degenerate_coal} coal ({:.1}% of candidates)",
        100.0 * degenerate_coal as f64 / total as f64
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
    for onset in [12.0, 15.0, 18.0, 20.0, 22.0, 25.0, 28.0, 30.0] {
        let frac = 100.0 * temps.iter().filter(|&&t| t >= onset).count() as f64 / total as f64;
        print!("  {onset:.0}C:{frac:.0}%");
    }
    println!("\n");

    // Plausible, not degenerate: the geotherm promoted some coal, but nowhere near
    // all of it (the stub #14 all-or-nothing failure), and left living peat behind.
    let frac = geotherm_coal as f64 / total as f64;
    assert!(
        geotherm_coal > 0,
        "the geotherm promoted no coal at all — onset {COAL_ONSET_C} C is too cold"
    );
    assert!(
        frac < 0.90,
        "the geotherm promoted {:.0}% of candidates — that is the degenerate \
         all-to-coal failure stub #14 warned of; raise COAL_ONSET_C",
        frac * 100.0
    );
    // And the coal that survives collapse must not vanish — the peat pool stays
    // real (a stronger diggable-seam claim is organic.rs's job).
    let surviving_peat = total - geotherm_coal;
    assert!(
        surviving_peat > 0,
        "every candidate became coal — no peat left"
    );
}

/// **DIAGNOSTIC (ignored by default).** On the Medium organic world, for a range
/// of trial onsets, report the max **contiguous** coal thickness across all
/// columns and how many columns carry >3 m — the calibration organic.rs's
/// diggable-seam test depends on. Run:
/// `cargo test -p dc-worldgen --release --test geotherm -- --ignored --nocapture`
#[test]
#[ignore]
fn medium_onset_for_thick_coal() {
    let pregen = Pregen::run(WorldParams {
        seed: 0x0D5E_ED57_2026,
        extent: Extent::Medium,
    });
    let f = deeptime::build_field(&pregen.grid, 0x0D5E_ED57_2026);
    let w = f.w;
    println!("\n=== Medium thick-coal vs onset (seed 0x0D5EED572026) ===");
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

/// The `temperature` field is a real per-cell field, every gradient in the
/// plausible band, and it genuinely varies across the map (hot crust exists).
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
    // The field is not flat — it carries a real per-cell tectonic signal (this
    // seed's crust runs boundary-dense and hot, so the realized span is at the
    // steep end of the model's full 12–55 C/km range; the rift-vs-craton spread
    // across the *whole* model is proven by the pure-function unit tests in
    // `geotherm.rs`, which do not depend on one world happening to hold a craton).
    assert!(
        hi - lo > 0.003,
        "the temperature field is flat ({lo:.4}..{hi:.4} C/m) — the geotherm is \
         reading no tectonic contrast at all"
    );
    println!("gradient span: {:.1}..{:.1} C/km", lo * 1000.0, hi * 1000.0);
}
