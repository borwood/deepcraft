//! **The Movement 2b (b) acceptance probe — does COLLUVIUM reach the archive?**
//! (`docs/design/material-behavior.md` § 13.2, journal/0112, corrections #55.)
//!
//! journal/0110 made the *fluvial* load material-aware and its probe returned a
//! null: identity reached **0.000006 %** of the archive. The cause was not the
//! sorting rule. It was that **fluvial transport is 0.109 % of this world's
//! sediment routing** — over the run the rivers pick up 659.5 m and hillslope
//! creep moves 605,117 m, a factor of 918 — and creep carried no identity at all.
//! The world's dominant sediment router was anonymous.
//!
//! This probe measures what happened when it stopped being. One question, asked
//! as absolutes against the same control the fluvial slice used:
//!
//! > **What fraction of the recorded archive carries a material its own
//! > environment would not have implied?**
//!
//! **And a second question the first one cannot answer.** Colluvium and alluvium
//! are different rocks in a cliff face — *locally derived and poorly sorted* against
//! *far-travelled and sorted* — so a probe that reported one provenance number
//! would be crediting the family rather than the member. The record carries no
//! **mover** axis (`DepUnit` is sixteen bytes with no padding left; see stubs.md
//! #25), so the two are not distinguishable by a *label*. They are distinguishable
//! by their **signature**, and this probe measures it two ways:
//!
//! | axis | colluvium | alluvium |
//! |---|---|---|
//! | **where the provenance sits** (drainage-area decile) | hillslopes — low decile | valleys and trunks — high decile |
//! | **how sorted the column is** (distinct species per record) | poorly — a mixture off the slope above | well — the ceiling dropped one fraction here |
//!
//! Run: `cargo run --release -p dc-worldgen --example colluvium_probe`

use std::time::Instant;

use dc_worldgen::deeptime::erosion::TransportLedger;
use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{
    DeepConfig, SEA_LEVEL_M, build_field_cfg, litho_of_tag, production_config, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// The A/B: material-aware transport is **on** in both arms, and only the
/// gravity/mass-wasting member moves. That is what makes the delta attributable
/// to creep rather than to the family — journal/0110's number is the `false` arm.
fn cfg(cells: &CellGrid, material_creep: bool) -> DeepConfig {
    DeepConfig {
        material_transport: true,
        material_creep,
        ..production_config(cells, SEED)
    }
}

/// Everything the acceptance rests on, pulled out of a field once so `main` and
/// the gate test read the same instrument (journal/0103).
struct Colluvium {
    /// Recorded mass whose species **disagrees** with what its own tag would have
    /// implied — the identity that travelled — in metres, and the archive total.
    travelled_m: f64,
    total_m: f64,
    /// The same split by decile of the depositing cell's final drainage area:
    /// decile 0 is the headwaters (hillslope), decile 9 the trunks.
    travelled_by_decile: [f64; 10],
    total_by_decile: [f64; 10],
    /// Mean count of distinct species in a cell's whole record, by the same
    /// decile — the poorly-sorted/well-sorted axis.
    species_per_column: [f64; 10],
    columns_by_decile: [f64; 10],
    /// Recorded mass per species (metres, summed over cells).
    mass_by_species: [f64; Litho::COUNT],
    /// Total recorded units — the merge-key cost of the new identity.
    units: u64,
    /// Deep-run wall clock and what the world keeps.
    deep_secs: f64,
    resident_bytes: usize,
    /// Landscape shape, so a reader can see the price beside the capability.
    mean_surf: f64,
    min_surf: f64,
    max_surf: f64,
}

fn measure(cells: &CellGrid, material_creep: bool) -> Colluvium {
    let t = Instant::now();
    let f = build_field_cfg(cells, &cfg(cells, material_creep));
    let deep_secs = t.elapsed().as_secs_f64();

    // Rank-based deciles over the *land* cells' drainage area — catchment area is
    // heavy-tailed by construction, so equal-width value bins would put 99 % of
    // the world in bin 0 and tell us nothing.
    let mut areas: Vec<f64> = f
        .area
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, _)| f.surf[*i] > SEA_LEVEL_M)
        .map(|(_, a)| a)
        .collect();
    areas.sort_by(f64::total_cmp);
    let decile_of = |a: f64| -> usize {
        if areas.is_empty() {
            return 0;
        }
        let rank = areas.partition_point(|&x| x < a);
        ((rank * 10) / areas.len()).min(9)
    };

    let mut out = Colluvium {
        travelled_m: 0.0,
        total_m: 0.0,
        travelled_by_decile: [0.0; 10],
        total_by_decile: [0.0; 10],
        species_per_column: [0.0; 10],
        columns_by_decile: [0.0; 10],
        mass_by_species: [0.0; Litho::COUNT],
        units: 0,
        deep_secs,
        resident_bytes: f.resident_bytes(),
        mean_surf: 0.0,
        min_surf: f64::INFINITY,
        max_surf: f64::NEG_INFINITY,
    };
    for &s in &f.surf {
        out.mean_surf += s;
        out.min_surf = out.min_surf.min(s);
        out.max_surf = out.max_surf.max(s);
    }
    out.mean_surf /= f.surf.len() as f64;

    for (i, rec) in f.strata.iter().enumerate() {
        let dec = decile_of(f.area[i]);
        let mut present = [false; Litho::COUNT];
        for u in &rec.units {
            out.units += 1;
            out.total_m += u.thickness_m();
            out.total_by_decile[dec] += u.thickness_m();
            out.mass_by_species[Litho::of_material(u.species()).index()] += u.thickness_m();
            present[Litho::of_material(u.species()).index()] = true;
            if Litho::of_material(u.species()) != litho_of_tag(u.tag()) {
                out.travelled_m += u.thickness_m();
                out.travelled_by_decile[dec] += u.thickness_m();
            }
        }
        if !rec.units.is_empty() {
            out.species_per_column[dec] += present.iter().filter(|p| **p).count() as f64;
            out.columns_by_decile[dec] += 1.0;
        }
    }
    for d in 0..10 {
        if out.columns_by_decile[d] > 0.0 {
            out.species_per_column[d] /= out.columns_by_decile[d];
        }
    }
    out
}

/// **The provenance fraction** — the share of recorded mass whose material
/// disagrees with what its own environment would have implied. Zero under the
/// scalar-load solve *by construction*; anything above it is identity that
/// travelled and survived to the archive.
fn provenance_fraction(c: &Colluvium) -> f64 {
    if c.total_m > 0.0 {
        c.travelled_m / c.total_m
    } else {
        0.0
    }
}

/// The audits the field does not keep — they live on the live solve, so this
/// needs its own run.
struct Audits {
    itemisation: f64,
    conservation: f64,
    creep_faces: usize,
    ledger: TransportLedger,
    chapters: usize,
}

fn audits(cells: &CellGrid) -> Audits {
    let r = run_cells(cells, &cfg(cells, true), true);
    Audits {
        itemisation: r.erosion.max_creep_itemisation_residue(),
        conservation: r.erosion.max_creep_conservation_residue(),
        creep_faces: r.erosion.creep_outflux_faces(),
        ledger: r.erosion.transport_ledger(),
        // The chapter table carries `K + 1` entries (the last is one past the
        // final chapter, for the ramp), so the number of chapters is one less.
        chapters: r.chapters.len().saturating_sub(1).max(1),
    }
}

fn main() {
    println!("=== colluvium probe — Movement 2b (b), seed {SEED}, Extent::Medium ===\n");
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let off = measure(&pregen.grid, false);
    let on = measure(&pregen.grid, true);

    println!("--- THE NUMBER: honest provenance in the archive ---");
    println!(
        "  anonymous creep (journal/0110's world): {:>12.4} m of {:>10.0} m  ({:.6} %)",
        off.travelled_m,
        off.total_m,
        100.0 * provenance_fraction(&off)
    );
    println!(
        "  creep carries identity:                 {:>12.4} m of {:>10.0} m  ({:.6} %)",
        on.travelled_m,
        on.total_m,
        100.0 * provenance_fraction(&on)
    );
    let gain = provenance_fraction(&on) / provenance_fraction(&off).max(f64::MIN_POSITIVE);
    println!(
        "  delta: {:+.4} m absolute, a factor of {:.0} on the fraction",
        on.travelled_m - off.travelled_m,
        gain
    );

    println!("\n--- WHERE the provenance sits (decile 0 = hillslope, 9 = trunk) ---");
    println!("  decile   anonymous %      identity %     archive mass (m)   species/column");
    for d in 0..10 {
        let po = if off.total_by_decile[d] > 0.0 {
            100.0 * off.travelled_by_decile[d] / off.total_by_decile[d]
        } else {
            0.0
        };
        let pn = if on.total_by_decile[d] > 0.0 {
            100.0 * on.travelled_by_decile[d] / on.total_by_decile[d]
        } else {
            0.0
        };
        println!(
            "  {d:>4}   {po:>11.6}   {pn:>13.6}   {:>16.0}   {:>7.3} -> {:.3}",
            on.total_by_decile[d], off.species_per_column[d], on.species_per_column[d],
        );
    }

    println!("\n--- CAN YOU TELL COLLUVIUM FROM ALLUVIUM? (the facies contrast) ---");
    // The record carries no **mover** axis (stubs.md #25), so the two are not
    // separable by a label. They are separable by **signature**, and the signature
    // is sortedness: a colluvial column holds a mixture off the slope above, an
    // alluvial one holds what the falling ceiling dropped here. Reported as the
    // hillslope/valley split of distinct species per recorded column, before and
    // after — a ratio, not a pinned number.
    let mean = |v: &[f64], w: &[f64]| -> f64 {
        let cols: f64 = w.iter().sum();
        if cols > 0.0 {
            v.iter().zip(w).map(|(s, c)| s * c).sum::<f64>() / cols
        } else {
            0.0
        }
    };
    let (h_off, h_on) = (
        mean(&off.species_per_column[..5], &off.columns_by_decile[..5]),
        mean(&on.species_per_column[..5], &on.columns_by_decile[..5]),
    );
    let (v_off, v_on) = (
        mean(&off.species_per_column[5..], &off.columns_by_decile[5..]),
        mean(&on.species_per_column[5..], &on.columns_by_decile[5..]),
    );
    println!("  distinct species per recorded column      anonymous     identity");
    println!("    hillslope columns (deciles 0-4)        {h_off:>10.3}   {h_on:>10.3}");
    println!("    valley + trunk columns (deciles 5-9)   {v_off:>10.3}   {v_on:>10.3}");
    println!(
        "  hillslope/valley sortedness ratio          {:>10.3}   {:>10.3}",
        h_off / v_off.max(f64::MIN_POSITIVE),
        h_on / v_on.max(f64::MIN_POSITIVE)
    );

    println!("\n--- what the record is MADE OF (metres of recorded thickness) ---");
    println!("  species             anonymous          identity        delta");
    for l in Litho::ALL {
        let k = l.index();
        println!(
            "  {:<14}  {:>14.0}  {:>14.0}  {:>+11.2}%",
            l.code(),
            off.mass_by_species[k],
            on.mass_by_species[k],
            if off.mass_by_species[k] > 0.0 {
                100.0 * (on.mass_by_species[k] / off.mass_by_species[k] - 1.0)
            } else {
                f64::NAN
            }
        );
    }

    println!("\n--- mass, per species, on the creep path ---");
    let a = audits(&pregen.grid);
    println!(
        "  largest relative gap between the creep itemisation and the metres the\n  \
         terrain actually moved (every cell, every epoch):        {:.3e}",
        a.itemisation
    );
    println!(
        "  largest relative amount of any species creep created or destroyed\n  \
         (every species, every epoch):                            {:.3e}",
        a.conservation
    );
    println!(
        "  creep moved {:.1} m over the run; the rivers picked up {:.1} m",
        a.ledger.diffused_m,
        a.ledger.entrained_m + a.ledger.incised_m
    );

    println!("\n--- cost and shape ---");
    println!(
        "  deep run    {:.2} s -> {:.2} s   (delta {:+.2} s)",
        off.deep_secs,
        on.deep_secs,
        on.deep_secs - off.deep_secs
    );
    println!(
        "  resident    {:.2} MiB -> {:.2} MiB (delta {:+.2} MiB)",
        mib(off.resident_bytes),
        mib(on.resident_bytes),
        mib(on.resident_bytes) - mib(off.resident_bytes)
    );
    println!(
        "  units       {} -> {}   ({:+.2} %)",
        off.units,
        on.units,
        100.0 * (on.units as f64 / off.units as f64 - 1.0)
    );
    println!(
        "  surface     mean {:.2} -> {:.2} m   min {:.1} -> {:.1}   max {:.1} -> {:.1}",
        off.mean_surf, on.mean_surf, off.min_surf, on.min_surf, off.max_surf, on.max_surf
    );

    println!("\n--- what a GRAVITY mover would cost the flow record (stubs.md #18) ---");
    // The flux record is keyed (cell, chapter, face) and creep crosses up to four
    // faces per cell per epoch. This is the entry count it would carry if the
    // mass-wasting mover were recorded — measured, so the affordability question
    // is answered with a number rather than a shrug.
    let entries = a.creep_faces * a.chapters;
    println!(
        "  donor faces carrying creep in the final epoch: {} over {} chapters\n  \
         = {} entries x 16 B = {:.1} MiB added to a {:.1} MiB world",
        a.creep_faces,
        a.chapters,
        entries,
        mib(entries * 16),
        mib(on.resident_bytes)
    );

    println!("\n--- verdict ---");
    // **The caption is a published claim the gate cannot check** (CLAUDE.md), so it
    // is derived from the measurement rather than written ahead of it.
    const EXPRESSED_BAR: f64 = 0.01;
    let prov = provenance_fraction(&on);
    let hill: f64 = on.travelled_by_decile[..5].iter().sum();
    let valley: f64 = on.travelled_by_decile[5..].iter().sum();
    if prov >= EXPRESSED_BAR {
        println!(
            "  EXPRESSES. Honest provenance reached {:.4} % of the archive (bar: {:.0} %),\n  \
             against {:.6} % when creep was anonymous. {:.1} % of it sits in the lower\n  \
             five drainage deciles — the hillslopes, which is where colluvium belongs.",
            100.0 * prov,
            100.0 * EXPRESSED_BAR,
            100.0 * provenance_fraction(&off),
            100.0 * hill / (hill + valley).max(f64::MIN_POSITIVE),
        );
    } else {
        println!(
            "  NULL. Honest provenance reached only {:.6} % of the archive, against a\n  \
             {:.0} % reporting bar — below the point where a difference could be seen in\n  \
             a cliff face at 0.9 m voxels. Creep moved {:.0} m over the run, so the\n  \
             identity had every opportunity; what it did NOT do was disagree with the\n  \
             local environment often enough to show. Nothing was tuned.",
            100.0 * prov,
            100.0 * EXPRESSED_BAR,
            a.ledger.diffused_m,
        );
    }
    println!(
        "  Colluvium is NOT sorted here and that is deliberate: creep is diffusive,\n  \
         so every edge moves the donor's whole composition in proportion. No\n  \
         competence ceiling, no settling draw. The contrast against alluvium is the\n  \
         finding, not a defect."
    );
}

#[cfg(test)]
mod gate {
    use super::*;

    /// **Creep moves material and invents none of it** — the two mass statements
    /// this slice stands on.
    ///
    /// *Why the invariants are scale-free.* Both are properties of a **single
    /// edge**, asserted as running maxima over every edge in the world. (a) The
    /// itemisation check says `Σ_species` of an edge's split equals the edge flux,
    /// which is `split_by_shares`' residual rule — arithmetic, not landscape.
    /// (b) The conservation check says the two endpoints of an edge compute the
    /// same split of the same flux with opposite signs, which holds because
    /// IEEE-754 subtraction is exactly antisymmetric — also arithmetic. A bigger
    /// grid has more edges and no new kinds of edge, so a violation that exists at
    /// production scale exists here. The **magnitudes** are what production scale
    /// is for and they live in `main`.
    #[test]
    fn creep_conserves_every_species_at_every_junction() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let r = run_cells(&pregen.grid, &cfg(&pregen.grid, true), true);
        let item = r.erosion.max_creep_itemisation_residue();
        let cons = r.erosion.max_creep_conservation_residue();
        assert!(
            item < 1e-12,
            "the creep species itemisation stopped equalling the metres the terrain \
             moved: relative gap {item:e}"
        );
        assert!(
            cons < 1e-12,
            "creep created or destroyed a species: relative amount {cons:e}"
        );
        assert!(
            r.erosion.creep_outflux_faces() > 0,
            "no cell shed any creep at all — the audits above are vacuous"
        );
    }

    /// **Identity that travels by creep reaches the archive, and anonymous creep
    /// leaves none.**
    ///
    /// *Why the invariant is scale-free.* It is a **per-unit predicate**: with the
    /// creep member off, the only identity in the record is what the fluvial pass
    /// carried, so turning creep on can only ever *add* disagreeing mass — never
    /// remove it below the fluvial floor. The claim asserted is the direction of
    /// that inequality, which needs one hillslope cell somewhere to have received
    /// material its own environment would not have implied. The **size** of the
    /// gain is a fact about a particular landscape and lives in `main`.
    #[test]
    fn colluvial_identity_reaches_the_archive() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let off = measure(&pregen.grid, false);
        let on = measure(&pregen.grid, true);
        assert!(
            on.travelled_m > off.travelled_m,
            "creep carried no identity into the archive: {:.6} m with it on against \
             {:.6} m with it off",
            on.travelled_m,
            off.travelled_m
        );
    }

    /// **The itemisation equals its own total.** Recorded mass split by species
    /// must sum to the recorded total in both arms — the standing probe-defect
    /// shape in this repo, and wrong at every world size.
    #[test]
    fn the_species_itemisation_sums_to_the_recorded_total() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        for creep in [false, true] {
            let c = measure(&pregen.grid, creep);
            let sum: f64 = c.mass_by_species.iter().sum();
            let rel = if c.total_m > 0.0 {
                ((sum - c.total_m) / c.total_m).abs()
            } else {
                0.0
            };
            assert!(
                rel < 1e-12,
                "material_creep={creep}: species itemisation {sum} != total {}",
                c.total_m
            );
        }
    }
}
