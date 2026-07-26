//! Measurement harness for the thickness-dominance outcrop rule (journal/0068).
//!
//! For every cell of the production Medium deep-time record it computes two
//! outcrop lithologies:
//!
//! - **old rule** — the topmost unit's lithology (`units.last()`), or `Basement`
//!   for an empty record: the pre-journal/0068 `exposed_litho`;
//! - **new rule** — the lithology dominating the topmost
//!   `OUTCROP_DOMINANCE_WINDOW_M` of the record: the shipped `exposed_litho`.
//!
//! It reports the fraction of cells whose outcrop changed, the litho→litho
//! transition table over the changed cells, and how many cells now outcrop
//! `OrganicCharcoal` (expected ≈ zero — a fire bed is capped at 0.04 m and can
//! essentially never dominate a 0.9 m window).
//!
//! Run: `cargo run --release -p dc-worldgen --example outcrop_dominance_probe`

use dc_worldgen::deeptime::lithology::{Litho, exposed_litho};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn pct(a: usize, b: usize) -> f64 {
    if b == 0 {
        0.0
    } else {
        a as f64 * 100.0 / b as f64
    }
}

/// The pre-journal/0068 rule: the top unit's lithology, or basement if empty.
fn old_rule(units: &[dc_worldgen::deeptime::DepUnit]) -> Litho {
    match units.last() {
        Some(u) => u.species,
        None => Litho::Basement,
    }
}

fn main() {
    // The Small control the brief expected to be inert — reported first so the
    // "does Small move?" question is answered explicitly.
    let small = Pregen::run(WorldParams {
        seed: 0x00C1_1A7E_2026,
        extent: Extent::Small,
    });
    let (mut s_total, mut s_rec, mut s_changed) = (0usize, 0usize, 0usize);
    for s in &small.deep.strata {
        s_total += 1;
        if !s.units.is_empty() {
            s_rec += 1;
        }
        if old_rule(&s.units) != exposed_litho(&s.units) {
            s_changed += 1;
        }
    }
    println!(
        "SMALL control (seed 0x00C11A7E2026): {s_total} cells, {s_rec} recorded, \
         {s_changed} changed outcrop ({:.3} % of recorded)\n",
        pct(s_changed, s_rec)
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });

    let n = Litho::COUNT;
    // transition[old][new] over changed cells.
    let mut transition = vec![vec![0usize; n]; n];
    let mut total = 0usize;
    let mut recorded = 0usize;
    let mut changed = 0usize;
    let mut new_charcoal = 0usize;
    let mut old_charcoal = 0usize;

    for s in &pregen.deep.strata {
        total += 1;
        if !s.units.is_empty() {
            recorded += 1;
        }
        let old = old_rule(&s.units);
        let new = exposed_litho(&s.units);
        if new == Litho::OrganicCharcoal {
            new_charcoal += 1;
        }
        if old == Litho::OrganicCharcoal {
            old_charcoal += 1;
        }
        if old != new {
            changed += 1;
            transition[old.index()][new.index()] += 1;
        }
    }

    println!("production Medium record: {total} cells ({recorded} with a non-empty record)");
    println!(
        "outcrop changed old→new: {changed} cells  ({:.3} % of all, {:.3} % of recorded)",
        pct(changed, total),
        pct(changed, recorded)
    );
    println!(
        "outcrop == OrganicCharcoal: new rule {new_charcoal}, old rule {old_charcoal} \
         (expected ≈ 0 for the new rule)"
    );

    println!("\ntransition table over the {changed} changed cells (rows = old, cols = new):");
    print!("{:>10}", "");
    for l in Litho::ALL {
        print!("{:>10}", l.code());
    }
    println!();
    for old in Litho::ALL {
        let row = &transition[old.index()];
        if row.iter().all(|&c| c == 0) {
            continue;
        }
        print!("{:>10}", old.code());
        for new in Litho::ALL {
            let c = row[new.index()];
            if c == 0 {
                print!("{:>10}", ".");
            } else {
                print!("{c:>10}");
            }
        }
        println!();
    }
}
