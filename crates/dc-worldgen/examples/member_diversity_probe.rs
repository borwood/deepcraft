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
//! Run: `cargo run --release -p dc-worldgen --example member_diversity_probe`

use std::time::Instant;

use dc_core::materials::geology::{GeologySet, vanilla};
use dc_core::materials::{MATERIAL_COUNT, MaterialId};
use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{DeepField, build_field, production_config, run_cells};
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

fn main() {
    let extent = Extent::Medium;
    let t = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent,
    });
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
        "{:<12} {:>10} {:>10} {:>12}  {}",
        "class", "members", "distinct", "units", "share by recorded metres"
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
}

#[cfg(test)]
mod gate {
    use super::*;

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
