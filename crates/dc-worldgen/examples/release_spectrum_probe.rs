//! **FS-A — what the release spectrum did to the world's shed mass** (2026-08-02).
//!
//! The weathering pass (`dc:deep/weather_inventory`) now emits `Structure→Loose`
//! **through the source rock's declared release spectrum** (U7/R2,
//! `material-behavior.md` §3): granite bedrock sheds grus — gravel and sand
//! modes plus clay fines — instead of an ungraded lump. This probe reports, on
//! the world `dc-client` boots, **how many metres of shed mass took each grain
//! grade**, and locates the strongest exemplars (the tour-map signal for the
//! post-merge walk: stripped upland vs distal basin cut face).
//!
//! **Caption honesty, stated up front:**
//!
//! - `weather_inventory` is **OFF in production by default** (the flip is the
//!   user's — `DeepConfig::weather_inventory`); this probe builds the shipped
//!   world's terrain WITH the pass on, the same override the
//!   `--weather-inventory` launch flag applies.
//! - Grades are **computed at emission and NOT yet recorded**: the grain state's
//!   one home is P11 slice 3's packed `DepUnit` (U5), and until it merges the
//!   split lands in `weather_inventory::grain_write_seam`, deliberately inert.
//!   The itemisation below is derived from the **recorded band** through the
//!   SAME `split_quantities` arithmetic the pass emits with — one
//!   implementation, two consumers (S-3), so report and pass cannot disagree.
//! - Today's only weathering source is STUB #16's flat granite basement, so the
//!   worldwide spectrum IS granite's authored grus table; per-material spread
//!   across the roster arrives with the genesis/emplacement heir.
//!
//! `cargo run --release -p dc-worldgen --example release_spectrum_probe`

use dc_core::materials::release::{GRAIN_GRADE_COUNT, GrainGrade, split_quantities};
use dc_worldgen::deeptime::inventory::{BEDROCK_SEAM_MATERIAL, InvForm};
use dc_worldgen::deeptime::{DeepConfig, build_field_cfg, production_config};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The world `dc-client` boots — `BENCH_SEED` at `WORLDGEN_EXTENT` (the same
/// pinning `s20_ledger_residency_probe` documents).
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;

/// The world-level itemisation: shed mass per grain grade, plus everything the
/// caption needs to stay honest.
struct Spectrum {
    extent: Extent,
    cells: usize,
    cell_m: f64,
    /// Cells whose bedrock seam carries at least one weathering fact.
    weathering_cells: usize,
    /// Σ over cells of the recorded weathering band (metres of loose shed).
    total_band_m: f64,
    /// The band itemised per grade through the declared spectrum.
    by_grade_m: [f64; GRAIN_GRADE_COUNT],
    /// Worst per-cell |split re-sum − band| — the Law-3 residual, which the
    /// remainder-exact split keeps at (near) zero.
    max_cell_residual_m: f64,
    /// The strongest exemplars: `(band_m, gx, gy)`, descending — the tour-map
    /// half. The deepest saprolite bands sit where subaerial residence was
    /// longest and cover thinnest.
    top: Vec<(f64, usize, usize)>,
}

fn measure(extent: Extent) -> Spectrum {
    let pregen = Pregen::run(WorldParams { seed: SEED, extent });
    let cfg = DeepConfig {
        weather_inventory: true,
        ..production_config(&pregen.grid, SEED)
    };
    let field = build_field_cfg(&pregen.grid, &cfg);

    let products = BEDROCK_SEAM_MATERIAL
        .release_products(InvForm::Structure, InvForm::Loose)
        .expect("the bedrock seam material declares a weathering spectrum");

    let w = field.w;
    let mut out = Spectrum {
        extent,
        cells: field.strata.len(),
        cell_m: field.cell_m,
        weathering_cells: 0,
        total_band_m: 0.0,
        by_grade_m: [0.0; GRAIN_GRADE_COUNT],
        max_cell_residual_m: 0.0,
        top: Vec::new(),
    };
    for (i, s) in field.strata.iter().enumerate() {
        let Some(view) = field.ledgers.get(i) else {
            continue;
        };
        let band = view.weathering_product_m(s.units.len());
        if band <= 0.0 {
            continue;
        }
        out.weathering_cells += 1;
        out.total_band_m += band;
        // The SAME split the pass emitted with, applied to the recorded band.
        let mut cell_sum = 0.0f64;
        for (p, q) in split_quantities(products, band) {
            out.by_grade_m[p.grade.raw() as usize] += q;
            cell_sum += q;
        }
        out.max_cell_residual_m = out.max_cell_residual_m.max((cell_sum - band).abs());
        out.top.push((band, i % w, i / w));
    }
    out.top
        .sort_by(|a, b| b.0.partial_cmp(&a.0).expect("bands are finite"));
    out.top.truncate(5);
    out
}

fn report(s: &Spectrum) {
    println!(
        "\n=== FS-A: the release spectrum on the shipped world (seed {SEED}, {}) ===\n",
        s.extent.label()
    );
    println!(
        "  NOTE  `weather_inventory` is OFF in production by default; this world ran the\n\
         \x20       pass ON (the --weather-inventory override). Grades are computed at\n\
         \x20       emission and NOT yet recorded — the grain byte lands with P11 slice 3's\n\
         \x20       packed DepUnit; until then the split feeds `grain_write_seam` (inert).\n\
         \x20       Source rock: STUB #16's flat granite basement, so this is granite's\n\
         \x20       authored grus spectrum expressed worldwide.\n"
    );
    println!(
        "  cells {}   weathering cells {}  ({:.1}%)",
        s.cells,
        s.weathering_cells,
        100.0 * s.weathering_cells as f64 / s.cells as f64
    );
    println!(
        "  total shed (saprolite band) {:.3} m summed over cells   mean {:.4} m per weathering cell\n",
        s.total_band_m,
        s.total_band_m / (s.weathering_cells.max(1) as f64)
    );
    println!("  --- metres of shed mass per grain grade (the caption number) ---");
    for g in GrainGrade::ALL {
        let m = s.by_grade_m[g.raw() as usize];
        println!(
            "    {:<7} {:>12.3} m   {:>5.1}%",
            g.name(),
            m,
            100.0 * m / s.total_band_m.max(f64::MIN_POSITIVE)
        );
    }
    println!(
        "    itemisation == total: worst per-cell residual {:.3e} m (remainder-exact split)\n",
        s.max_cell_residual_m
    );
    println!("  --- strongest exemplars (tour-map candidates; deep-grid cells, {} m each) ---", s.cell_m);
    for (band, gx, gy) in &s.top {
        println!("    band {band:>8.3} m   deep cell ({gx:>3}, {gy:>3})");
    }
    println!(
        "\n  (walk framing: a stripped upland shows the thickest in-place grus — coarse\n\
         \x20  gravel+sand modes; the distal basin face shows what transport DID with the\n\
         \x20  fines. The walk itself is main-session driven, post-merge.)\n"
    );
}

fn main() {
    println!("measuring...");
    let s = measure(EXTENT);
    report(&s);
}

/// **GATE** (journal/0103: an example that can fail belongs in the gate).
///
/// Runs at `Extent::Small`. Every assertion is **scale-free**: the itemisation
/// invariant is per-cell arithmetic (a split that must re-sum to its own band —
/// wrong at every world size if wrong at any), and the spectrum-coverage claims
/// are properties of the declared table, not of magnitudes. The production
/// magnitudes are `main`'s job.
#[cfg(test)]
mod gate {
    use super::*;
    use dc_core::materials::release::validate_release_registry;

    #[test]
    fn the_release_spectrum_itemises_the_shed_mass_exactly() {
        // The declaration-time closure must hold before anything downstream
        // means anything (shares sum to 1 exactly, tables canonical).
        assert!(
            validate_release_registry().is_ok(),
            "the vanilla release registry does not validate"
        );

        let s = measure(Extent::Small);

        // The pass must have fired somewhere, or the itemisation is vacuous.
        assert!(s.weathering_cells > 0, "no cell weathered");
        assert!(s.total_band_m > 0.0);

        // ITEMISATION == TOTAL, per cell (Law 3 through the report path): the
        // split of each cell's band re-sums to that band. The split is
        // remainder-exact by construction; the bound is f64 fold noise on a
        // ≤5-term sum, not a tuned tolerance.
        assert!(
            s.max_cell_residual_m <= 1e-12 * s.total_band_m.max(1.0),
            "worst per-cell residual {:.3e} m",
            s.max_cell_residual_m
        );
        // And the grade columns re-sum to the total band (the world-level fold
        // of the same identity; same derived bound class).
        let sum: f64 = s.by_grade_m.iter().sum();
        assert!(
            (sum - s.total_band_m).abs() <= 1e-9 * s.total_band_m.max(1.0),
            "grade columns {sum} != total band {}",
            s.total_band_m
        );

        // The itemisation carries ONLY the declared spectrum's grades: granite's
        // grus table has no scree rung, so scree must be exactly zero, and every
        // declared rung must be inhabited (every share is validated non-zero).
        let products = BEDROCK_SEAM_MATERIAL
            .release_products(InvForm::Structure, InvForm::Loose)
            .expect("granite declares a weathering spectrum");
        for g in GrainGrade::ALL {
            let declared = products.iter().any(|p| p.grade == g);
            let inhabited = s.by_grade_m[g.raw() as usize] > 0.0;
            assert_eq!(
                declared,
                inhabited,
                "{}: declared={declared} inhabited={inhabited}",
                g.name()
            );
        }

        // The tour-map half returned real candidates with in-range coordinates.
        assert!(!s.top.is_empty());
        let w = (s.cells as f64).sqrt() as usize;
        for &(band, gx, gy) in &s.top {
            assert!(band > 0.0 && gx < w && gy < w);
        }
    }
}
