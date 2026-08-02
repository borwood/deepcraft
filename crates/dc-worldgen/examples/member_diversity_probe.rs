//! **The P11 slice-1 acceptance probe — does the deep record carry MEMBERS?**
//! (ROADMAP § *MEMBERS INTO DEEP HISTORY* build sequence step 1;
//! `docs/audits/2026-08-01-members-into-history-design.md`.)
//!
//! Before this slice, `DepUnit::species` was one of **seven classes** and the
//! member that fills a voxel was invented at expression, per chunk, under a
//! formation context sampled at the chunk centre. The record therefore held
//! *exactly one material per class* — whichever `Litho::reference_material`
//! named — and no amount of expression-time cleverness could recover a
//! distinction deep time never wrote down. That is the census finding P11 exists
//! to answer (journal/0129 § 5): **expression cannot diversify what history never
//! distinguished.**
//!
//! So the acceptance question is a distribution, not a threshold: **for each
//! class the deep tier can deposit, how many distinct `MaterialId`s does the
//! shipped record actually hold, and in what proportion?**
//!
//! ## Read the nulls honestly
//!
//! **Only 4 of the vanilla set's 10 classes have more than one member**, and only
//! **two of those** are classes the deep record can deposit (clastic-fine:
//! mudstone + siltstone; clastic-coarse: sandstone + conglomerate — the other two
//! multi-member classes are igneous, which the deep record never carries). The
//! four organic classes ship one member each.
//!
//! **A one-member class CANNOT diversify, and a `1` in its row is not a failure.**
//! It is the correct answer, and reading it as a null would be reading the
//! content set as a defect in the mechanism. The rows that carry the claim are
//! the multi-member ones.
//!
//! ## The before, and why it is reconstructible in one binary
//!
//! The pre-P11 record is recoverable exactly, from the same run, with no second
//! build: every unit's class is `Litho::of_material(u.species)` and the material
//! the old record would have carried is that class's `reference_material()`. So
//! **BEFORE is the same walk with the identity collapsed to its class's
//! reference** — one material per class by construction — and AFTER is the
//! recorded identity. The comparison is not across two machine states
//! (journal/0125's measurement shape).
//!
//! It also reports the **merge-key split factor** the design audit left unpriced
//! (§ 6a, I4): member grade sharpens `deposit_as`'s merge key, so two beds that
//! coalesced as one `ClasticFine` unit split when one is mudstone and the other
//! siltstone. The probe counts the units the record holds against the units it
//! *would* hold if adjacent same-class units re-merged.
//!
//! ## The slice-2 half: **where does a unit's identity come from?**
//!
//! Slice 1 answered *"does the record hold members"*. Ruling 6 asks a sharper
//! question: **whose fact is a bed's name — the load's, or the site's?** Slice 2
//! retires the deposition draw for transported deposits, so identity is now a
//! conserved quantity flowing through the mass arithmetic rather than a roll at
//! the end of it.
//!
//! The falsifier is not "is the code path taken" — that is trivially checkable and
//! proves nothing. It is: **of the metres whose identity came from the arriving
//! composition, how many would the deposition site's own climate have named
//! differently?** Those metres are the ones whose rock is a fact about their
//! *source*. If the number were near zero, source composition and site climate
//! would be agreeing anyway and the ruling would have bought correctness of
//! principle with no expression. `DeepConfig::identity_audit` evaluates exactly
//! the counterfactual draw the slice exists to stop evaluating, which is why it is
//! an instrument and off in production.
//!
//! Run: `cargo run --release -p dc-worldgen --example member_diversity_probe`

use std::time::Instant;

use dc_core::materials::geology::{GeologySet, vanilla};
use dc_core::materials::{MATERIAL_COUNT, MaterialId};
use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{DeepConfig, DeepField, build_field, production_config, run_cells};
use dc_worldgen::geology::deep_class_of_species;
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

/// Per-class tallies over the whole recorded archive.
#[derive(Clone, Default)]
struct ClassTally {
    /// Recorded metres per material inside this class.
    mass_m: [f64; MATERIAL_COUNT],
    /// Recorded units per material inside this class.
    units: [u64; MATERIAL_COUNT],
}

impl ClassTally {
    fn distinct(&self) -> usize {
        self.units.iter().filter(|&&n| n > 0).count()
    }
    fn total_units(&self) -> u64 {
        self.units.iter().sum()
    }
    fn total_m(&self) -> f64 {
        self.mass_m.iter().sum()
    }
}

/// Everything the acceptance rests on, pulled out of a field once so `main` and
/// the gate test read the same instrument (journal/0103).
struct Diversity {
    /// One tally per depositional class, indexed by `Litho::index()`.
    by_class: [ClassTally; Litho::COUNT],
    /// How many members the *content set* registers per class — the ceiling each
    /// row could possibly reach, and the honest denominator for a null.
    registered: [usize; Litho::COUNT],
    /// Units the record holds.
    units: u64,
    /// Units the record would hold if adjacent units of the same CLASS re-merged
    /// — i.e. what the pre-P11 merge key would have produced over this run's
    /// deposition sequence. The ratio is the split factor.
    units_class_merged: u64,
    total_m: f64,
    deep_secs: f64,
    resident_bytes: usize,
}

fn registered_per_class(set: &GeologySet) -> [usize; Litho::COUNT] {
    let mut out = [0usize; Litho::COUNT];
    for l in Litho::ALL {
        if l == Litho::Basement {
            continue;
        }
        out[l.index()] = set
            .class(deep_class_of_species(l))
            .map_or(0, |c| c.members().len());
    }
    out
}

fn measure_field(f: &DeepField, deep_secs: f64) -> Diversity {
    let set = vanilla();
    let mut by_class: [ClassTally; Litho::COUNT] = Default::default();
    let (mut units, mut units_class_merged, mut total_m) = (0u64, 0u64, 0.0f64);
    for s in &f.strata {
        let mut prev: Option<(Litho, u8)> = None;
        for u in &s.units {
            let l = Litho::of_material(u.species);
            let t = &mut by_class[l.index()];
            t.mass_m[u.species.raw() as usize] += u.thickness_m;
            t.units[u.species.raw() as usize] += 1;
            units += 1;
            total_m += u.thickness_m;
            // The counterfactual merge: the pre-P11 key was (tag, chapter,
            // class), so two adjacent units differing only in material would
            // have been one. Approximated by (class, chapter) — tag equality is
            // implied for a same-class run laid by the same depositor, and this
            // is deliberately the *generous* reading, so the split factor it
            // reports is a lower bound.
            let key = (l, u.chapter);
            if prev != Some(key) {
                units_class_merged += 1;
            }
            prev = Some(key);
        }
    }
    Diversity {
        by_class,
        registered: registered_per_class(&set),
        units,
        units_class_merged,
        total_m,
        deep_secs,
        resident_bytes: f.resident_bytes(),
    }
}

fn measure(cells: &CellGrid) -> Diversity {
    let t = Instant::now();
    let f = build_field(cells, SEED);
    let secs = t.elapsed().as_secs_f64();
    measure_field(&f, secs)
}

/// **Where the record's metres got their names** (P11 slice 2, ruling 6):
/// `(from the arriving composition, from the fitness draw, and — a subset of the
/// first — from the arriving composition where the site's own draw would have
/// disagreed)`.
struct Provenance {
    transported_m: f64,
    drawn_m: f64,
    site_would_disagree_m: f64,
    secs: f64,
}

fn measure_provenance(cells: &CellGrid) -> Provenance {
    let cfg = DeepConfig {
        identity_audit: true,
        ..production_config(cells, SEED)
    };
    let t = Instant::now();
    let run = run_cells(cells, &cfg, true);
    let secs = t.elapsed().as_secs_f64();
    let m = run.erosion.identity_provenance_m();
    Provenance {
        transported_m: m[1],
        drawn_m: m[2],
        site_would_disagree_m: m[3],
        secs,
    }
}

fn main() {
    let extent = Extent::Medium;
    let t = Instant::now();
    let pregen = Pregen::run(WorldParams { seed: SEED, extent });
    println!("pregen: {:.2} s", t.elapsed().as_secs_f64());
    let d = measure(&pregen.grid);

    println!("\n=== MEMBER DIVERSITY IN THE DEEP RECORD (seed {SEED}, {extent:?}) ===");
    println!(
        "deep run {:.2} s · {} recorded units · {:.1} m recorded · field {:.2} MiB",
        d.deep_secs,
        d.units,
        d.total_m,
        d.resident_bytes as f64 / (1024.0 * 1024.0)
    );
    println!(
        "\nBEFORE (pre-P11): every unit of a class carried that class's REFERENCE \
         material — one material per class, by construction."
    );
    println!("AFTER: the material fitness chose at deposition, recorded.\n");
    println!(
        "{:<12} {:>10} {:>10} {:>12}  share by recorded metres",
        "class", "members", "distinct", "units"
    );
    for l in Litho::ALL {
        if l == Litho::Basement {
            continue;
        }
        let t = &d.by_class[l.index()];
        if t.total_units() == 0 {
            println!(
                "{:<12} {:>10} {:>10} {:>12}  (nothing recorded)",
                l.code(),
                d.registered[l.index()],
                0,
                0
            );
            continue;
        }
        let mut parts: Vec<String> = Vec::new();
        let tot = t.total_m();
        for m in MaterialId::all() {
            let mm = t.mass_m[m.raw() as usize];
            if mm > 0.0 {
                parts.push(format!(
                    "{} {:.1}%",
                    m.qualified_name(),
                    100.0 * mm / tot.max(f64::MIN_POSITIVE)
                ));
            }
        }
        println!(
            "{:<12} {:>10} {:>10} {:>12}  {}",
            l.code(),
            d.registered[l.index()],
            t.distinct(),
            t.total_units(),
            parts.join(" · ")
        );
    }

    let multi: Vec<Litho> = Litho::ALL
        .into_iter()
        .filter(|l| *l != Litho::Basement && d.registered[l.index()] > 1)
        .collect();
    let single: Vec<Litho> = Litho::ALL
        .into_iter()
        .filter(|l| *l != Litho::Basement && d.registered[l.index()] == 1)
        .collect();
    println!(
        "\nmulti-member classes (the rows that can carry the claim): {}",
        multi
            .iter()
            .map(|l| l.code())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "one-member classes (a `1` here is CORRECT, not a null): {}",
        single
            .iter()
            .map(|l| l.code())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "\nmerge-key split factor: {} units recorded vs {} under the pre-P11 \
         class-only key = {:.4}x",
        d.units,
        d.units_class_merged,
        d.units as f64 / d.units_class_merged.max(1) as f64
    );
    println!(
        "residency consequence at 16 B/unit: {:.2} MiB vs {:.2} MiB",
        d.units as f64 * 16.0 / (1024.0 * 1024.0),
        d.units_class_merged as f64 * 16.0 / (1024.0 * 1024.0)
    );

    // ---- P11 slice 2: whose fact is a bed's name? -------------------------
    let p = measure_provenance(&pregen.grid);
    let total = p.transported_m + p.drawn_m;
    println!(
        "\n=== IDENTITY PROVENANCE (ruling 6 — the draw retirement) ===\n\
         audited run {:.2} s (the counterfactual draw is the cost; production \
         does not pay it)",
        p.secs
    );
    println!(
        "  from the ARRIVING COMPOSITION (no draw)  {:>12.1} m  {:>6.2} %",
        p.transported_m,
        100.0 * p.transported_m / total.max(1e-9)
    );
    println!(
        "  from the FITNESS DRAW                    {:>12.1} m  {:>6.2} %",
        p.drawn_m,
        100.0 * p.drawn_m / total.max(1e-9)
    );
    println!(
        "  ...of the transported metres, the ones the SITE would have named\n\
           differently: {:.1} m ({:.2} % of transported, {:.2} % of all record)",
        p.site_would_disagree_m,
        100.0 * p.site_would_disagree_m / p.transported_m.max(1e-9),
        100.0 * p.site_would_disagree_m / total.max(1e-9)
    );
    println!(
        "  → that last figure is the OUTCOME: metres whose rock is a fact about\n\
           their SOURCE and not about the climate where they landed."
    );
}

#[cfg(test)]
mod gate {
    use super::*;
    use dc_worldgen::deeptime::{production_config, run_cells};

    /// **The invariant is scale-free**: "a class with two registered members has
    /// units of both in the archive" is a statement about the *selection*
    /// machinery running per deposition event, and a deposition event is a
    /// per-cell-per-epoch fact. A `Small` world runs the same passes over fewer
    /// cells; if fitness is not running at deposition, every class collapses to
    /// one material at every extent.
    ///
    /// It asserts an invariant, never a snapshot — no share, no MiB figure.
    #[test]
    fn a_multi_member_class_records_more_than_one_material() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let d = measure(&pregen.grid);
        let mut checked = 0;
        for l in Litho::ALL {
            if l == Litho::Basement || d.registered[l.index()] < 2 {
                continue;
            }
            let t = &d.by_class[l.index()];
            if t.total_units() == 0 {
                continue; // nothing of this class was deposited at this extent
            }
            checked += 1;
            assert!(
                t.distinct() > 1,
                "{} has {} registered members and {} recorded units but only {} \
                 distinct material(s) — deposition-time fitness is not running",
                l.code(),
                d.registered[l.index()],
                t.total_units(),
                t.distinct()
            );
        }
        assert!(
            checked > 0,
            "no multi-member class was exercised — the probe proved nothing"
        );
    }

    /// **A one-member class holds exactly one material**, and it must be *that*
    /// member — not the reference table's answer by coincidence. This is the
    /// honest-null half: it pins that the nulls are the content set's shape and
    /// not a broken selector.
    #[test]
    fn a_one_member_class_holds_exactly_that_member() {
        let set = vanilla();
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let d = measure(&pregen.grid);
        for l in Litho::ALL {
            if l == Litho::Basement || d.registered[l.index()] != 1 {
                continue;
            }
            let t = &d.by_class[l.index()];
            if t.total_units() == 0 {
                continue;
            }
            let only = set
                .class(deep_class_of_species(l))
                .and_then(|c| c.members().first().copied())
                .map(|i| set.member(i).material)
                .expect("a one-member class has a member");
            assert_eq!(t.distinct(), 1, "{} split with one member", l.code());
            assert!(
                t.units[only.raw() as usize] > 0,
                "{} recorded something other than its only member",
                l.code()
            );
        }
    }

    /// **The instrument cannot perturb the world it measures** (P11 slice 2).
    ///
    /// `identity_audit` evaluates a counterfactual draw beside the real one. A
    /// draw is entropy, and entropy that leaked into the recorded identity would
    /// make the measurement a report about the measuring. Asserted as record
    /// equality, which is stricter than a tolerance and is the only honest bar for
    /// an instrument.
    ///
    /// Scale-free: it is a statement about one code path's side effects.
    #[test]
    fn the_identity_audit_is_bit_inert() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let cfg = production_config(&pregen.grid, SEED);
        let off = run_cells(&pregen.grid, &cfg, true);
        let on = run_cells(
            &pregen.grid,
            &DeepConfig {
                identity_audit: true,
                ..cfg
            },
            true,
        );
        assert_eq!(off.grid.r, on.grid.r, "the audit moved the bedrock plane");
        assert_eq!(off.grid.h, on.grid.h, "the audit moved the alluvium plane");
        assert_eq!(off.grid.strata, on.grid.strata, "the audit moved the record");
        // And it actually measured something: a production world deposits.
        let m = on.erosion.identity_provenance_m();
        assert!(
            m[1] + m[2] > 0.0,
            "the audit tallied no metres at all — it is not wired to the record"
        );
        assert!(
            m[3] <= m[1],
            "the disagreement subset ({}) exceeds the transported total ({})",
            m[3],
            m[1]
        );
    }

    /// **Determinism**: same seed, same config, byte-identical record. Fitness at
    /// deposition introduced a new draw into the hottest per-cell path, and the
    /// parallel record phase forks across cells — so the address must be the cell
    /// index and never an iteration counter.
    #[test]
    fn the_record_is_identical_across_two_runs_of_one_seed() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let cfg = production_config(&pregen.grid, SEED);
        // (a) same seed + config, twice: the record is a pure function of them.
        let a = run_cells(&pregen.grid, &cfg, true);
        let again = run_cells(&pregen.grid, &cfg, true);
        assert_eq!(
            a.grid.strata, again.grid.strata,
            "two runs of one seed produced different records"
        );
        // (b) the parallel record phase forks across cells, so the draw address
        // must be the cell index and never an iteration counter.
        let b = run_cells(&pregen.grid, &cfg, false);
        assert_eq!(
            a.grid.strata.len(),
            b.grid.strata.len(),
            "cell count moved between the parallel and scalar paths"
        );
        for (i, (x, y)) in a.grid.strata.iter().zip(&b.grid.strata).enumerate() {
            assert_eq!(x, y, "cell {i}: parallel and scalar records disagree");
        }
    }
}
