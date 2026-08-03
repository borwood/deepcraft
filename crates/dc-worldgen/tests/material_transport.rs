//! **Movement 2b, first slice — material-aware transport**
//! (`docs/design/material-behavior.md` § 13.3–13.7, journal/0110).
//!
//! Before this slice the erosion chain moved a **scalar**. A cell entrained
//! `min(H, room)` metres of *something*, handed it downstream, and whatever
//! exceeded the next cell's capacity was set down as *something else*; the rock a
//! recorded unit was made of had to be inferred from the environment at the
//! receiver. That inference is not wrong, it is **blind to provenance** — it
//! cannot know that a distal cell has no gravel to drop because the gravel rained
//! out twenty cells upstream at the mountain front.
//!
//! So the load becomes a multiset of `(lithology, quantity)`, sorted by settling
//! velocity, and deposition is a **falling competence ceiling**. This suite pins
//! the four things that had to be true for that to mean anything:
//!
//! 1. the identity pair — off is the old solve exactly, on moves the world;
//! 2. **mass, per species**, down a multi-receiver DAG;
//! 3. the ceiling actually falls, and nothing leaves a cell it could not carry;
//! 4. a unit's recorded material can genuinely disagree with its environment.
//!
//! Production-scale numbers — the downstream fining distribution the slice is
//! *accepted* on — live in `examples/facies_probe.rs`.

use dc_worldgen::deeptime::lithology::{Litho, settling_table};
use dc_worldgen::deeptime::{
    self, DeepConfig, DeepField, DeepRun, build_field_cfg, litho_of_tag, production_config_with,
    run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

mod providers_common;

/// **The golden fixture's seed**, not a fresh one: the off-half of the identity
/// pair is a cross-commit claim against `providers_common`'s captured constants,
/// and a claim about a different world would prove nothing.
const SEED: u64 = providers_common::SEED;

fn small_world() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

fn cfg_for(cells: &CellGrid, material_transport: bool) -> DeepConfig {
    DeepConfig {
        material_transport,
        // **The pre-2b world was routed by a UNIFORM `p = 4`**, and reaching a
        // fixed point means reproducing all of the configuration it was captured
        // under, not most of it (the same discipline `mfd_routing.rs` states when
        // it turns off *both* forks to reach the pre-MFD goldens). Hybrid `p`
        // (journal/0113) made the shipped exponent spatially varying, so this
        // fixture pins the flat ramp `p_chan == p_hill == 4.0` — which
        // `MfdParams::exponent_at` short-circuits, evaluating no ramp at all.
        // Without this the scalar-load constants would silently become "pre-2b
        // transport under post-b' routing", which is not a cross-commit claim
        // about anything.
        mfd_exponent: 4.0,
        mfd_exponent_channel: 4.0,
        // **And the UNBOUNDED hillslope operator** (journal/0122). The transport
        // pass now sub-cycles each epoch to keep its per-edge coefficient inside
        // `CREEP_MAX_EDGE_COEFF`; every fixed point below was captured before it,
        // under a single raw step. Same discipline as the two pins above —
        // reaching a fixed point means reproducing ALL of the configuration it was
        // captured under. Without this the constants would silently become "the
        // old solve under the NEW integrator", which is a claim about no commit
        // that ever existed.
        creep_substep: false,
        // **And the pre-calibration RATES** (journal/0114). The erosional calibration
        // multiplied `weathering` / `diffusion` / `k_transport` / `k_bedrock` by 45;
        // a fixed point captured before it is only reachable by reproducing that
        // too — the same discipline as the `p = 4` pin above. Without this the
        // constants would silently become "the old solve at the NEW erosional
        // clock", which is a claim about no commit that ever existed.
        ..production_config_with(
            cells,
            SEED,
            &dc_worldgen::DeepOverrides {
                calibrated_rates: Some(false),
                ..Default::default()
            },
        )
    }
}

fn field(cells: &CellGrid, on: bool) -> DeepField {
    build_field_cfg(cells, &cfg_for(cells, on))
}

fn run(cells: &CellGrid, on: bool) -> DeepRun {
    run_cells(cells, &cfg_for(cells, on), false)
}

// ---------------------------------------------------------------------------
// 1. The identity pair.

/// **Both halves, because either alone is gameable.** A test that only asserted
/// "off is byte-identical to the pre-slice world" would pass against a flag that
/// does nothing; a test that only asserted "on is different" would pass against a
/// flag that broke something. The pre-slice fingerprints are the shipped
/// `providers_golden` constants — captured before this slice existed — so the
/// off-half is a genuine cross-commit statement and not a self-comparison.
#[test]
fn material_transport_off_is_the_pre_slice_world_and_on_moves_it() {
    let pregen = small_world();
    let off = field(&pregen.grid, false);
    let on = field(&pregen.grid, true);

    let off_surface = providers_common::surface_fingerprint(&off);
    let off_record = providers_common::record_fingerprint(&off);
    println!("scalar-load surface = {off_surface:#018X}");
    println!("scalar-load record  = {off_record:#018X}");
    assert_eq!(
        off_surface,
        providers_common::GOLDEN_SURFACE_SCALAR_LOAD,
        "with material transport OFF the surface must be the pre-slice world: \
         {off_surface:#018X}"
    );
    assert_eq!(
        off_record,
        providers_common::GOLDEN_RECORD_SCALAR_LOAD,
        "with material transport OFF the record must be the pre-slice world: \
         {off_record:#018X}"
    );

    let on_surface = providers_common::surface_fingerprint(&on);
    let on_record = providers_common::record_fingerprint(&on);
    println!("shipped surface = {on_surface:#018X}");
    println!("shipped record  = {on_record:#018X}");
    assert_ne!(
        on_surface, off_surface,
        "material-aware transport changed NOTHING about the terrain — the \
         competence ceiling never bit, so the slice is pipework, not behaviour"
    );
}

// ---------------------------------------------------------------------------
// 2. Mass, per species.

/// **The per-species residual rule, verified rather than trusted.**
///
/// journal/0109 fixed the scalar leak by giving the last weighted direction
/// `q − Σ(earlier shares)`. The tempting generalisation is to split the *total*
/// exactly and then hand each species its fraction of that total — and it leaks,
/// silently, because each species picks up its own rounding against a shared
/// denominator while the total stays exact. This test constructs the case and
/// shows both: the shared-total rule loses mass on a species, the per-species rule
/// does not.
///
/// It is a statement about arithmetic, not about the world, so it runs on a
/// hand-written partition rather than a simulation.
#[test]
fn a_shared_total_leaks_per_species_and_an_own_budget_does_not() {
    // Weights that do not sum to 1 in binary — the normal case for a normalised
    // f64 partition, and the reason the residual rule exists at all.
    let w = [0.1f64, 0.2, 0.3, 0.15, 0.25];
    // Species quantities spanning several orders of magnitude, which is what an
    // alluvial load actually looks like.
    let q = [1.0f64, 3.7e-5, 2.4e3, 9.81e-12];
    let total: f64 = q.iter().sum();

    // (a) THE WRONG RULE: one exact split of the total, then apportion.
    let mut given = 0.0;
    let mut wrong = [0.0f64; 4];
    for (d, &wt) in w.iter().enumerate() {
        let share = if d == w.len() - 1 {
            total - given
        } else {
            wt * total
        };
        given += share;
        for (s, out) in wrong.iter_mut().enumerate() {
            *out += share * (q[s] / total);
        }
    }

    // (b) THE RULE THIS SLICE USES: every species runs its own budget.
    let mut right = [0.0f64; 4];
    for (s, out) in right.iter_mut().enumerate() {
        let mut given_s = 0.0;
        for (d, &wt) in w.iter().enumerate() {
            let share = if d == w.len() - 1 {
                q[s] - given_s
            } else {
                wt * q[s]
            };
            given_s += share;
        }
        *out = given_s;
    }

    for s in 0..4 {
        assert_eq!(
            right[s], q[s],
            "species {s}: its own budget must close exactly, {} != {}",
            right[s], q[s]
        );
    }
    let leaked = (0..4).any(|s| wrong[s] != q[s]);
    assert!(
        leaked,
        "the shared-total rule happened to be exact here, so this test is not \
         demonstrating the hazard it exists to demonstrate — pick harder numbers"
    );
}

/// **The local half of the mass proof, measured on a whole world.** For every
/// cell, every species and every epoch, the shares the receivers were handed
/// summed to what the cell held. This is the direct analogue of journal/0109's
/// `the_partition_leaves_no_residue` — but per species, which is the statement a
/// shared-total split would fail while the scalar total still looked perfect.
#[test]
fn no_species_leaks_at_its_own_junction() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let residue = r.erosion.max_species_split_residue();
    assert!(
        residue < 1e-12,
        "a per-species split left a relative residue of {residue} — the last \
         direction is not taking the residual for that species"
    );
}

/// **The global half.** A share written into the flux record but never added to a
/// neighbour would pass the residue test above and fail this one: the whole-world
/// ledger `Δ(ΣR + ΣH) == uplift + biotic`, over a full production run with
/// material-aware transport on.
#[test]
fn mass_is_conserved_with_material_transport_on() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let residual = deeptime::total_mass(&r.grid) - r.mass_before - r.uplift_total - r.biotic_total;
    assert!(
        residual.abs() < 1.0,
        "mass leaked with material transport on: residual {residual} \
         (uplift {}, biotic {})",
        r.uplift_total,
        r.biotic_total
    );
}

// ---------------------------------------------------------------------------
// 3. The ceiling.

/// **The settling order is a property of the materials, and it is the one the
/// property sheet states.** Not a hand table: basement (3 mm at SG 2.7) must
/// outrank coarse clastic (0.3 mm at 2.35) must outrank fine clastic (0.004 mm at
/// 2.4). If this ever inverts, the "coarsest first" draw is drawing the wrong
/// thing and every facies claim in the slice is void.
#[test]
fn the_settling_order_comes_out_of_the_property_sheet() {
    let w = settling_table();
    assert!(
        w[Litho::Basement.index()] > w[Litho::ClasticCoarse.index()],
        "basement debris must settle before sand"
    );
    assert!(
        w[Litho::ClasticCoarse.index()] > w[Litho::ClasticFine.index()],
        "sand must settle before mud"
    );
    assert!(
        w.iter().all(|v| *v > 0.0),
        "a species with no settling velocity would never rain out: {w:?}"
    );
}

/// **Nothing leaves a cell that the cell could not carry** — the competence
/// invariant, read off a whole world rather than argued.
///
/// The falsifier is the load still in flight at the end of the last epoch: for
/// every cell, every species with a non-zero outgoing load must have a settling
/// velocity at or below the ceiling that cell's own transport capacity sets. A
/// species over the ceiling still in the water means the sweep missed material
/// the cell entrained or incised after the sweep ran, which is exactly the
/// ordering bug the sweep was moved to the end to prevent.
///
/// **Scale-free:** it is a per-cell predicate over a per-cell quantity — no
/// neighbourhood, no accumulation, no dependence on how many cells exist.
#[test]
fn nothing_over_the_competence_ceiling_is_still_in_the_water() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let w = r.erosion.settling();
    let load = r.erosion.load_species();
    let layout = r.erosion.transport_layout();
    let energy = r.erosion.energy();
    // The ceiling is stated relative to the world's own `k_transport`
    // (journal/0114) — a capacity is that coefficient times a position in the
    // drainage network, so the reference has to travel with it.
    let k_t = cfg_for(&pregen.grid, true).k_transport;
    let mut offenders = 0usize;
    for (c, e) in energy.iter().enumerate() {
        let ceiling = deeptime::competence_ceiling(*e, k_t);
        // P11 slice 2: the load is CSR-sparse over the species axis, so the row
        // index says which species a cell can hold at all and the axis says how
        // fast each of them settles.
        let (b, ks) = layout.row(c);
        for (j, &k) in ks.iter().enumerate() {
            if load[b + j] > 0.0 && w[k as usize] > ceiling {
                offenders += 1;
            }
        }
    }
    assert_eq!(
        offenders, 0,
        "{offenders} (cell, species) pairs are carrying material heavier than \
         their own competence ceiling"
    );
}

// ---------------------------------------------------------------------------
// 4. Identity actually travels.

/// **The unit's material and its environment can now disagree — and do.**
///
/// This is the claim the slice exists to make. With the flag off a unit's species
/// is `litho_of_tag(tag)` by construction, so *every* unit agrees with its
/// environment; with it on, agreement becomes an outcome rather than a
/// definition, and a world in which they never disagree is a world where identity
/// did not travel.
#[test]
fn a_recorded_unit_can_be_made_of_something_its_environment_would_not_imply() {
    let pregen = small_world();
    let off = field(&pregen.grid, false);
    let on = field(&pregen.grid, true);

    let disagreements = |f: &DeepField| {
        f.strata
            .iter()
            .flat_map(|s| s.units.iter())
            .filter(|u| Litho::of_material(u.species()) != litho_of_tag(u.tag()))
            .count()
    };
    assert_eq!(
        disagreements(&off),
        0,
        "with the flag off a unit's species is a pure function of its tag; a \
         disagreement means the default is not the tag-derived one"
    );
    assert!(
        disagreements(&on) > 0,
        "not one recorded unit is made of anything other than what its \
         environment implies — the load's identity never reached the record, so \
         this slice is pipework"
    );
}

/// **No recorded unit is made of basement.** A unit is by definition a *deposit*;
/// basement is the unrecorded rock below the pile. `Litho::as_deposited` turns
/// quarried basement into the coarse clastic detritus it is once it lands, and if
/// that ever slipped, a loose gravel bar would be handed granite's resistance at
/// the next epoch's outcrop read and the world would slowly turn to shield.
#[test]
fn no_recorded_unit_claims_to_be_basement() {
    let pregen = small_world();
    let f = field(&pregen.grid, true);
    let basement_units = f
        .strata
        .iter()
        .flat_map(|s| s.units.iter())
        .filter(|u| Litho::of_material(u.species()) == Litho::Basement)
        .count();
    assert_eq!(
        basement_units, 0,
        "{basement_units} recorded units claim to be basement — a deposit cannot be"
    );
}

/// `DepUnit` gained an axis and must not have gained a byte: the record is the
/// largest thing the ritual keeps (S19 measured it at 90.8 % of `DeepField`
/// residency), so a `DepUnit` that grew would cost megabytes across a production
/// world. The species fits the existing padding.
///
/// Stated against a **mirror of the pre-slice layout** rather than against a
/// number, so it asserts the invariant ("the axis is free") instead of pinning a
/// snapshot a future field would have to be talked out of.
#[test]
fn the_species_axis_costs_the_record_nothing() {
    // The pre-P11 claim was "species fits the 16-byte layout's padding". Since
    // P11 slice 3 the unit is PACKED (8 bytes: u32 bitfield + u32 fixed-point
    // thickness), and the axis claim got stronger, not weaker: species (6 bits),
    // the mover (3) and the grain reservation (3) all ride INSIDE half the old
    // footprint. The invariant this test keeps is the direction — axes must not
    // grow the unit — asserted against the packed layout's own compile-checked
    // size.
    assert_eq!(
        std::mem::size_of::<dc_worldgen::deeptime::DepUnit>(),
        8,
        "an axis grew the packed DepUnit past its 8 bytes"
    );
}
