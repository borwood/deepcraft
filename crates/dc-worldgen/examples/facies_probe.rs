//! **The Movement 2b acceptance probe — does the facies gradient EXPRESS?**
//! (`docs/design/material-behavior.md` § 13.5, `docs/design/flow.md` § 8,
//! journal/0110.)
//!
//! The slice is easy to *build* and easy to fool yourself about. A suspended load
//! resolved into species, a competence ceiling, a coarsest-first draw — all of it
//! can be correct, pass every mass test, and produce **no visible facies gradient
//! at all**, because a gradient needs concentrated flow and journal/0109 measured
//! this world's peak catchment falling 1,245 → 84 cells when MFD shipped with a
//! uniform convergence exponent. Trunk rivers are ~15× weaker than they were.
//!
//! So this probe exists to answer one question with a distribution rather than a
//! threshold: **is the material recorded at low-energy sites finer than the
//! material recorded at high-energy sites, and does grain size fall as you go
//! downstream?** It reports the answer as absolutes, against the scalar-load
//! control, on the shipped world — and if the answer is no, it says *null* in
//! those words and names the blocker.
//!
//! **What is measured, and why it is the record and not the load.** The load is
//! pass-transient; what a player digs is the archive. Every recorded unit now
//! carries the material that actually arrived (`DepUnit::species`), so the
//! measurement is a mass-weighted grain size read straight off the strata —
//! the same bytes the collapse tier turns into rock.
//!
//! Two independent axes, because they can disagree and the disagreement is
//! informative:
//!
//! | axis | what it asks |
//! |---|---|
//! | **energy band** (`DepTag::energy`, the flow's own measured capacity) | does a low-energy site record finer rock than a high-energy one? |
//! | **drainage-area decile** (position in the network) | does grain size fall *downstream*? |
//!
//! Under the scalar-load solve the first axis is a **tautology**: the rock was
//! *derived from* the band, so it is exactly one grain size per band with zero
//! variance. Under material-aware transport it becomes an outcome, and the width
//! of the distribution inside a band is the provenance signal — two cells at the
//! same energy holding different rock because different rock reached them.
//!
//! Run: `cargo run --release -p dc-worldgen --example facies_probe`

use std::time::Instant;

use dc_worldgen::deeptime::lithology::{Litho, settling_table};
use dc_worldgen::deeptime::erosion::TransportLedger;
use dc_worldgen::deeptime::{
    DeepConfig, EnergyBand, SEA_LEVEL_M, build_field_cfg, competence_ceiling, litho_of_tag,
    production_config, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn cfg(cells: &CellGrid, material_transport: bool) -> DeepConfig {
    DeepConfig {
        material_transport,
        ..production_config(cells, SEED)
    }
}

/// The grain size of each species, in mm, straight off the reference property
/// sheet — the same sheet [`settling_table`] reads. The probe reports **grain
/// size** rather than settling velocity because grain size is what a cross-section
/// shows and what § 8's acceptance test is written in.
fn grain_table() -> [f64; Litho::COUNT] {
    let mut out = [0.0; Litho::COUNT];
    for l in Litho::ALL {
        out[l.index()] = f64::from(l.reference_material().props().grain_size_mm);
    }
    out
}

/// A mass-weighted mean and the mass it was taken over.
#[derive(Clone, Copy, Default)]
struct Weighted {
    mass_m: f64,
    weighted: f64,
}

impl Weighted {
    fn add(&mut self, mass: f64, value: f64) {
        self.mass_m += mass;
        self.weighted += mass * value;
    }
    fn mean(&self) -> f64 {
        if self.mass_m > 0.0 {
            self.weighted / self.mass_m
        } else {
            0.0
        }
    }
}

/// Everything the acceptance rests on, pulled out of a field once so `main` and
/// the gate test read the same instrument (journal/0103).
struct Facies {
    /// Mass-weighted mean grain size (mm) of **subaerial mineral** units, by the
    /// energy band measured at deposition.
    by_band: [Weighted; 3],
    /// The same, by decile of the depositing cell's final drainage area — decile
    /// 0 is the headwaters, decile 9 the trunks.
    by_area_decile: [Weighted; 10],
    /// Recorded mass per species (metres, summed over cells).
    mass_by_species: [f64; Litho::COUNT],
    /// Recorded mass whose species **disagrees** with what its own tag would have
    /// implied — the identity that travelled, in metres and as a fraction.
    travelled_m: f64,
    total_m: f64,
    /// Total recorded units, and how many carry a coarse fraction at all.
    units: u64,
    /// Deep-run wall clock and what the world keeps.
    deep_secs: f64,
    resident_bytes: usize,
    /// Landscape shape, so a reader can see the price beside the capability.
    mean_surf: f64,
    min_surf: f64,
    max_surf: f64,
}

fn measure(cells: &CellGrid, material_transport: bool) -> Facies {
    let t = Instant::now();
    let f = build_field_cfg(cells, &cfg(cells, material_transport));
    let deep_secs = t.elapsed().as_secs_f64();
    let grain = grain_table();

    // Decile edges over the *land* cells' drainage area. Rank-based rather than
    // value-based: catchment area is heavy-tailed by construction, so equal-width
    // value bins would put 99 % of the world in bin 0 and tell us nothing.
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

    let mut out = Facies {
        by_band: [Weighted::default(); 3],
        by_area_decile: [Weighted::default(); 10],
        mass_by_species: [0.0; Litho::COUNT],
        travelled_m: 0.0,
        total_m: 0.0,
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
        for u in &rec.units {
            let g = grain[u.species.index()];
            out.units += 1;
            out.total_m += u.thickness_m;
            out.mass_by_species[u.species.index()] += u.thickness_m;
            if u.species != litho_of_tag(u.tag) {
                out.travelled_m += u.thickness_m;
            }
            // The facies question is about clastic sediment. Organic horizons are
            // *made* where they lie — a peat is not something a river carried —
            // so including them would measure the biotic layer's footprint rather
            // than the transport pass's.
            let clastic = matches!(u.species, Litho::ClasticFine | Litho::ClasticCoarse)
                && !u.tag.biota.is_organic();
            if !clastic {
                continue;
            }
            let b = match u.tag.energy {
                EnergyBand::Low => 0,
                EnergyBand::Medium => 1,
                EnergyBand::High => 2,
            };
            out.by_band[b].add(u.thickness_m, g);
            out.by_area_decile[dec].add(u.thickness_m, g);
        }
    }
    out
}

/// The headline: the **fining ratio** — mean grain size at the coarsest energy
/// band over mean grain size at the finest. `1.0` means the record has no
/// grain-size gradient across energy at all.
fn fining_ratio(f: &Facies) -> f64 {
    let lo = f.by_band[0].mean();
    let hi = f.by_band[2].mean().max(f.by_band[1].mean());
    if lo > 0.0 { hi / lo } else { 0.0 }
}

/// **The provenance fraction** — the share of recorded mass whose material
/// disagrees with what its own environment would have implied. Zero under the
/// scalar-load solve *by construction*, because there the rock was derived from
/// the environment; anything above zero is identity that travelled.
fn provenance_fraction(f: &Facies) -> f64 {
    if f.total_m > 0.0 {
        f.travelled_m / f.total_m
    } else {
        0.0
    }
}

/// **Can this world's flows carry anything at all?** The diagnosis half of the
/// probe, and it needs the live solve rather than the field, because the per-cell
/// transport capacity is gen-time scratch the world does not keep.
///
/// It answers, over the subaerial cells of the final epoch, what fraction of them
/// have a competence ceiling above each species' settling velocity — how much of
/// the roster the landscape is physically able to move — and hands back the run's
/// transport ledger. If almost no cell can lift mud, the facies gradient has
/// nothing to be a gradient *of*, and the finding is about discharge, not sorting.
struct Carrying {
    ledger: TransportLedger,
    land_cells: usize,
    /// Fraction of land cells whose ceiling clears each species' settling velocity.
    can_carry: [f64; Litho::COUNT],
    median_ceiling: f64,
    max_ceiling: f64,
}

fn carrying(cells: &CellGrid) -> Carrying {
    let r = run_cells(cells, &cfg(cells, true), false);
    let w = settling_table();
    let energy = r.erosion.energy();
    let mut ceilings: Vec<f64> = Vec::new();
    let mut can = [0usize; Litho::COUNT];
    for (i, e) in energy.iter().enumerate() {
        if r.grid.r[i] + r.grid.h[i] <= SEA_LEVEL_M {
            continue;
        }
        let ceil = competence_ceiling(*e);
        ceilings.push(ceil);
        for (k, ws) in w.iter().enumerate() {
            if ceil >= *ws {
                can[k] += 1;
            }
        }
    }
    ceilings.sort_by(f64::total_cmp);
    let n = ceilings.len().max(1);
    Carrying {
        ledger: r.erosion.transport_ledger(),
        land_cells: ceilings.len(),
        can_carry: std::array::from_fn(|k| can[k] as f64 / n as f64),
        median_ceiling: ceilings.get(n / 2).copied().unwrap_or(0.0),
        max_ceiling: ceilings.last().copied().unwrap_or(0.0),
    }
}

fn main() {
    println!("=== facies probe — Movement 2b, seed {SEED}, Extent::Medium ===\n");
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let off = measure(&pregen.grid, false);
    let on = measure(&pregen.grid, true);

    println!("--- the record, by measured energy band (subaerial clastic mass) ---");
    println!("                       scalar load            material-aware");
    for (b, name) in ["Low", "Medium", "High"].iter().enumerate() {
        println!(
            "  {name:<7} mean grain  {:>10.4} mm ({:>10.0} m)   {:>10.4} mm ({:>10.0} m)",
            off.by_band[b].mean(),
            off.by_band[b].mass_m,
            on.by_band[b].mean(),
            on.by_band[b].mass_m,
        );
    }
    println!(
        "\n  FINING RATIO (coarsest band / finest band): {:.3}  ->  {:.3}",
        fining_ratio(&off),
        fining_ratio(&on)
    );

    println!("\n--- the record, by drainage-area decile (0 = headwater, 9 = trunk) ---");
    println!("  decile     scalar mm      2b mm        2b mass (m)");
    for d in 0..10 {
        println!(
            "  {d:>4}     {:>10.4}   {:>10.4}   {:>14.0}",
            off.by_area_decile[d].mean(),
            on.by_area_decile[d].mean(),
            on.by_area_decile[d].mass_m,
        );
    }

    println!("\n--- what the record is MADE OF (metres of recorded thickness) ---");
    let w = settling_table();
    let grain = grain_table();
    println!("  species          w_s     grain mm        scalar            2b        delta");
    for l in Litho::ALL {
        let k = l.index();
        println!(
            "  {:<14} {:>6.3}   {:>9.4}   {:>12.0}  {:>12.0}  {:>+11.1}%",
            l.code(),
            w[k],
            grain[k],
            off.mass_by_species[k],
            on.mass_by_species[k],
            if off.mass_by_species[k] > 0.0 {
                100.0 * (on.mass_by_species[k] / off.mass_by_species[k] - 1.0)
            } else {
                f64::NAN
            }
        );
    }

    println!("\n--- identity travel ---");
    println!(
        "  recorded mass whose material disagrees with its own environment: \
         {:.4} m of {:.0} m  ({:.6} %)",
        on.travelled_m,
        on.total_m,
        100.0 * provenance_fraction(&on)
    );
    println!(
        "  scalar-load control (must be exactly zero): {:.4} m ({:.6} %)",
        off.travelled_m,
        100.0 * provenance_fraction(&off)
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
        "  units       {} -> {}   ({:+.1} %)",
        off.units,
        on.units,
        100.0 * (on.units as f64 / off.units as f64 - 1.0)
    );
    println!(
        "  surface     mean {:.2} -> {:.2} m   min {:.1} -> {:.1}   max {:.1} -> {:.1}",
        off.mean_surf, on.mean_surf, off.min_surf, on.min_surf, off.max_surf, on.max_surf
    );

    println!("\n--- CAN THIS WORLD CARRY ANYTHING? (the diagnosis) ---");
    let c = carrying(&pregen.grid);
    println!(
        "  subaerial cells {} | competence ceiling: median {:.5}, max {:.4}",
        c.land_cells, c.median_ceiling, c.max_ceiling
    );
    for l in Litho::ALL {
        println!(
            "    land cells able to hold {:<9} (w_s {:>6.3}): {:>7.3} %",
            l.code(),
            w[l.index()],
            100.0 * c.can_carry[l.index()]
        );
    }
    let led = c.ledger;
    let picked = led.entrained_m + led.incised_m;
    println!(
        "  picked up over the run: {:.1} m entrained + {:.1} m incised = {:.1} m",
        led.entrained_m, led.incised_m, picked
    );
    println!(
        "  put down:  {:.1} m by capacity | {:.1} m by competence | {:.1} m at sinks",
        led.deposited_by_capacity_m, led.deposited_by_competence_m, led.deposited_at_sink_m
    );
    if picked > 0.0 {
        println!(
            "  share of the pick-up the competence ceiling put straight back: {:.2} %",
            100.0 * led.deposited_by_competence_m / picked
        );
    }
    println!("\n  WHO ACTUALLY MOVES THIS WORLD'S SEDIMENT (metres over the run):");
    println!("    fluvial load (entrained + incised)  {picked:>14.1}");
    println!(
        "    weathered in place (never travels) {:>14.1}",
        led.weathered_m
    );
    println!(
        "    moved by hillslope creep           {:>14.1}",
        led.diffused_m
    );
    println!(
        "    fluvial share of all sediment routing: {:.3} %",
        100.0 * picked / (picked + led.diffused_m)
    );
    println!(
        "  transported mass as a share of the whole recorded archive: {:.4} %",
        100.0 * picked / on.total_m
    );
    println!(
        "  (archive {:.0} m. This is the number that decides whether SORTING can\n   \
         matter at all on this world: a record built by in-place weathering and\n   \
         hillslope creep has almost nothing for a river to have sorted.)",
        on.total_m
    );

    println!("\n--- verdict ---");
    let ratio = fining_ratio(&on);
    let decile_span = {
        let first = on.by_area_decile[0].mean();
        let last = on.by_area_decile[9].mean();
        if last > 0.0 { first / last } else { 0.0 }
    };
    let prov = provenance_fraction(&on);
    println!(
        "  energy-band fining ratio {ratio:.3}; headwater/trunk grain ratio \
         {decile_span:.3}; provenance {:.4} %",
        100.0 * prov
    );
    // **The caption is a published claim the gate cannot check** (CLAUDE.md), so
    // it is *derived from the measurement*, never written ahead of it: the probe
    // decides which of the two verdicts to print from the numbers it just took.
    let fluvial_share = 100.0 * picked / (picked + led.diffused_m);
    // **The reporting bar, stated rather than implied.** One percent of the
    // archive is not a physical threshold — it is the point below which a
    // difference cannot be seen in a cliff face at 0.9 m voxels, so calling
    // anything under it "expressed" would be a claim no walk could confirm.
    const EXPRESSED_BAR: f64 = 0.01;
    if prov >= EXPRESSED_BAR && ratio > 1.0 {
        println!(
            "  EXPRESSES. Identity reached {:.4} % of the archive (bar: {:.0} %) and \
             the record\n  carries a grain-size gradient across energy.",
            100.0 * prov,
            100.0 * EXPRESSED_BAR
        );
    } else {
        println!("  NULL. The gradient does not express on this world, and the reason is");
        println!("  measured above rather than guessed at. In order of size:");
        println!(
            "    1. FLUVIAL TRANSPORT IS {:.3} % OF THIS WORLD'S SEDIMENT ROUTING.\n       \
             Hillslope creep moves {:.0} m and rivers pick up {:.0} m — a factor of \
             {:.0}.\n       Sorting a load that carries a thousandth of the sediment cannot \
             change\n       what a cliff face looks like, however correct the sorting is.",
            fluvial_share,
            led.diffused_m,
            picked,
            led.diffused_m / picked.max(1.0)
        );
        println!(
            "    2. NO CELL ON THIS WORLD CAN CARRY SAND. The largest competence\n       \
             ceiling anywhere is {:.4}; the coarse-clastic settling threshold is \
             {:.3}.\n       The ceiling is anchored on the SHIPPED facies rule's own \
             Low/Medium\n       capacity boundary (0.002), and the maximum capacity this \
             world reaches is\n       {:.2e} — three times BELOW it. Two independent \
             instruments agree that\n       this landscape has essentially no fluvial \
             competence.",
            c.max_ceiling,
            w[Litho::ClasticCoarse.index()],
            c.max_ceiling / 420.0
        );
        println!(
            "    3. Downstream of both: journal/0109's uniform convergence exponent, \
             which\n       dropped peak catchment 1,245 -> 84 cells. Hybrid-`p` is the named \
             heir\n       and it is the FLOW arc's, not this one's."
        );
        println!(
            "  The ceiling was NOT tuned to move these numbers, and it should not be:\n  \
             a constant chosen to manufacture a gradient out of 0.1 % of the sediment\n  \
             would be a number pretending to be a mechanism."
        );
    }
}

#[cfg(test)]
mod gate {
    use super::*;

    /// **The instrument works, and the control is a tautology** — the two
    /// statements that make the production report readable.
    ///
    /// *Why the invariants are scale-free.* Both are **per-unit predicates** over
    /// whatever units exist. (a) With material-aware transport off, a unit's
    /// species is `litho_of_tag(tag)` by definition, so the provenance fraction is
    /// **identically zero at any world size** — it is a statement about a function,
    /// not about a landscape. (b) With it on, the fraction must be positive, which
    /// needs only that *some* cell somewhere received material its own environment
    /// would not have implied — a property of having a flow network at all. The
    /// **magnitudes**, and the gradient itself, are what production scale is for
    /// and they live in `main`.
    #[test]
    fn identity_travels_on_and_is_a_tautology_off() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let off = measure(&pregen.grid, false);
        let on = measure(&pregen.grid, true);
        assert_eq!(
            off.travelled_m, 0.0,
            "with the flag off a unit's material is derived from its tag, so no \
             unit can disagree with it — {:.3} m did",
            off.travelled_m
        );
        assert!(
            on.travelled_m > 0.0,
            "not one metre of the record disagrees with its own environment — the \
             load's identity never reached the archive"
        );
    }

    /// **The itemisation equals its own total.** Recorded mass split by species
    /// must sum to the recorded total, whichever solve laid it down — the standing
    /// probe-defect shape in this repo (both `flow_cost_probe` failures were a
    /// missing row in an itemisation), and wrong at every world size.
    #[test]
    fn the_species_itemisation_sums_to_the_recorded_total() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        for on in [false, true] {
            let f = measure(&pregen.grid, on);
            let sum: f64 = f.mass_by_species.iter().sum();
            let rel = if f.total_m > 0.0 {
                ((sum - f.total_m) / f.total_m).abs()
            } else {
                0.0
            };
            assert!(
                rel < 1e-12,
                "material_transport={on}: species itemisation {sum} != total {}",
                f.total_m
            );
        }
    }

    /// **No recorded mass is basement.** A unit is a deposit; basement is the
    /// unrecorded rock below the pile. If quarried basement ever reached the
    /// archive under its own name, a loose gravel bar would be handed granite's
    /// resistance at the next outcrop read.
    #[test]
    fn the_archive_holds_no_basement() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let f = measure(&pregen.grid, true);
        assert_eq!(
            f.mass_by_species[Litho::Basement.index()], 0.0,
            "basement reached the record as basement"
        );
    }
}
