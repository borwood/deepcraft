//! **FLOW continuation (a) — the head field and the vertical faces**
//! (`docs/design/flow.md` § 2.4, journal/0098).
//!
//! Slice 1 (journal/0096) recorded flux on 3D faces and left the **vertical**
//! (slot ↔ slot) ones structurally present and **honestly zero**: the drainage
//! solve is pure surface routing, so there was no number to write. This suite
//! answers the two questions that follow from filling them:
//!
//! 1. **Is the field a POTENTIAL, or an elevation wearing one's clothes?** The
//!    falsifier is artesian: head standing *above* the local ground. `H = y + sat`
//!    (`water.md`, explicitly unconfined and not Darcy) cannot express that at any
//!    resolution — it *is* an elevation. The constructed cases live beside the
//!    solve in `deeptime::head`; what is asserted here is that the production-shaped
//!    world's field is not a copy of its own topography.
//! 2. **Did filling them move a single voxel?** It must not. Lateral routing is
//!    untouched — the drainage solve stays steepest-descent-on-filled-elevation,
//!    and a multi-flow-direction partition (which head is what unlocks, flow.md
//!    § 2.6) is deliberately the next slice.
//!
//! These run a small deep-time world; the **production**-world numbers — seed
//! 1337, `Extent::Medium`, production flags — are the acceptance proper and live in
//! `examples/head_field_probe.rs`.

use dc_worldgen::deeptime::{
    DeepConfig, FaceKey, build_field_cfg, production_config, run_cells, sea_level_at,
    slot_for_chapter,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 0x0F10_0057_1CE1;

fn small_world() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

fn cfg_for(cells: &CellGrid) -> DeepConfig {
    production_config(cells, SEED)
}

// ---------------------------------------------------------------------------
// THE test of the slice: slice 1's honest zero becomes a real number.

/// **The A-4 discharge.** The vertical faces were a declared, honest hole; this is
/// the term that fills them. A zero count here means the head field computed
/// something nothing consumed — machinery built beside the machinery it was
/// supposed to feed.
#[test]
fn the_vertical_faces_are_no_longer_zero() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let census = f.flux.census();
    assert!(census.entries > 0, "the flow record is empty");
    assert!(
        census.vertical > 0,
        "ZERO vertical flux: the head field produced no exchange at all, so \
         slice 1's honest hole is still a hole (entries {}, lateral {})",
        census.entries,
        census.lateral
    );

    let down = f
        .flux
        .entries()
        .iter()
        .filter(|e| e.face == FaceKey::Down)
        .count();
    let up = f
        .flux
        .entries()
        .iter()
        .filter(|e| e.face == FaceKey::Up)
        .count();
    println!("vertical entries: down (recharge) {down}, up (artesian discharge) {up}");
    assert!(down > 0, "no infiltration anywhere — recharge is not wired");
    assert!(
        f.flux
            .entries()
            .iter()
            .filter(|e| e.face.is_vertical())
            .all(|e| e.magnitude > 0.0),
        "a zero-magnitude vertical face was stored — the record must stay sparse"
    );
    // The bound phase carries solute, not suspended clastic load, and dissolution
    // is dormant until continuation (c). An honest zero, asserted so the slice that
    // populates it deletes this on purpose (the A-1 discipline slice 1 set).
    assert!(
        f.flux
            .entries()
            .iter()
            .filter(|e| e.face.is_vertical())
            .all(|e| e.load == 0.0),
        "a vertical face carried suspended load — there is no honest number for \
         that until dissolution is switched on"
    );
}

/// Turn the field off and the faces return to slice 1's honest zero — the S-5
/// identity default, and the proof that the vertical entries come from the head
/// field and from nothing else.
#[test]
fn with_the_head_field_off_the_vertical_faces_return_to_zero() {
    let pregen = small_world();
    let off = DeepConfig {
        head_field: false,
        ..cfg_for(&pregen.grid)
    };
    let f = build_field_cfg(&pregen.grid, &off);
    assert!(f.head.is_empty(), "the field was planted with the flag off");
    assert_eq!(f.flux.census().vertical, 0);
    assert!(!f.flux.is_empty(), "the lateral record must be unaffected");
}

/// A vertical entry binds to **the chapter's slot**, exactly as every other entry
/// does — and that is not a convenience: during chapter `K`, chapter `K`'s unit
/// *was* the contemporaneous land surface, so flux crossing its face is precisely
/// that moment's recharge or discharge. `None` stays honest (the unconformity ate
/// the record), never a stale index.
#[test]
fn a_vertical_entry_binds_to_the_chapters_own_slot() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let mut resolved = 0usize;
    let mut eroded_away = 0usize;
    for i in 0..f.flux.cells() {
        for e in f.flux.entries_for(i) {
            if !e.face.is_vertical() {
                continue;
            }
            match slot_for_chapter(&f.strata[i], e.chapter) {
                Some(slot) => {
                    assert_eq!(f.strata[i].units[slot].chapter, e.chapter);
                    resolved += 1;
                }
                None => eroded_away += 1,
            }
        }
    }
    assert!(resolved + eroded_away > 0, "no vertical entries at all");
    println!("vertical entries: slot resolved {resolved}, chapter eroded away {eroded_away}");
}

// ---------------------------------------------------------------------------
// The field is a POTENTIAL, not an elevation.

/// **The invariant that makes "artesian" mean something.** Head may stand above the
/// ground *only* where a bed confines it — or where a **lake** stands on it, which
/// is the lake's own surface and is correct. Everywhere else an unconfined water
/// table cannot exceed the ground: it discharges through a seepage face. If the cap
/// ever failed, an exceedance would be a bug with a good name rather than an aquifer.
///
/// This is asserted on a **whole real world** rather than a constructed column,
/// because the cap is the kind of thing that holds in a fixture and leaks on a
/// landscape: the first run of this suite reported 232 "artesian" columns that were
/// a 2.5×10⁻⁵ m priority-flood residue read as a pond ([`PONDED_MIN_M`] now floors
/// it). Read on **subaerial** columns only: under the sea the potential is pinned at
/// the stand, and standing above the seabed means only "there is water above".
///
/// Whether an artesian column *occurs* is a separate, empirical question this
/// fixture is too small and too drowned to answer — that belongs to
/// `examples/head_field_probe.rs` on the production world. That it is
/// **representable** is proven by construction in
/// `deeptime::head::tests::head_can_exceed_the_local_surface_which_is_artesian`.
#[test]
fn an_unconfined_water_table_never_stands_above_its_own_ground() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    assert_eq!(f.head.len(), f.surf.len(), "the field was not planted");
    let sea = sea_level_at(&cfg, cfg.iterations.saturating_sub(1));

    let land: Vec<usize> = (0..f.head.len()).filter(|i| f.surf[*i] > sea).collect();
    assert!(!land.is_empty(), "the fixture world is entirely submerged");
    let mut below = 0usize;
    let mut artesian = 0usize;
    let mut lakes = 0usize;
    for i in &land {
        let (h, s) = (f.head[*i], f.surf[*i]);
        if h < s - 1e-6 {
            below += 1;
        } else if h > s + 1e-6 {
            // Standing water is pinned at its own surface, which is above the
            // ground by construction. That is a lake, not an aquifer.
            if f.lake.get(*i).copied().unwrap_or(false) {
                lakes += 1;
                continue;
            }
            artesian += 1;
            let hydro = dc_worldgen::deeptime::column_hydro(&f.strata[*i]);
            assert!(
                hydro.confined,
                "cell {i} has head {h} above its ground {s}, is not a lake, and is \
                 UNCONFINED — the seepage cap failed, so this is a bug and not an \
                 aquifer"
            );
        }
    }
    println!(
        "final sea stand {sea:.2} m · {} subaerial columns: water table BELOW \
         ground {below}, at ground {}, lakes {lakes}, ARTESIAN {artesian}",
        land.len(),
        land.len() - below - artesian - lakes
    );
}

// ---------------------------------------------------------------------------
// Byte-identity: the head field is a sidecar, not a change to the world.

/// **The hard scope boundary, as a test.** The head field plants a field and runs
/// no edges; lateral routing is untouched. So with the flag on and off, the
/// surface planes, the regolith, the strata record, the drainage export **and the
/// entire lateral flux record** must be bit-for-bit the same. If this fails, the
/// slice exceeded itself — head started deciding where rivers go, which is the MFD
/// slice, not this one.
///
/// (The cross-commit golden proof is separate and untouched:
/// `providers_golden.rs::the_production_world_still_hashes_to_the_pre_slice_goldens`.)
#[test]
fn the_head_field_is_a_sidecar_and_the_terrain_is_byte_identical() {
    let pregen = small_world();
    let on = DeepConfig {
        head_field: true,
        ..cfg_for(&pregen.grid)
    };
    let off = DeepConfig {
        head_field: false,
        ..cfg_for(&pregen.grid)
    };
    let a = build_field_cfg(&pregen.grid, &on);
    let b = build_field_cfg(&pregen.grid, &off);

    assert!(!a.head.is_empty() && b.head.is_empty());

    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&a.surf), bits(&b.surf), "the surface moved");
    assert_eq!(bits(&a.regolith), bits(&b.regolith), "the regolith moved");
    assert_eq!(bits(&a.area), bits(&b.area), "the drainage area moved");
    assert_eq!(bits(&a.exhum), bits(&b.exhum), "exhumation moved");
    assert_eq!(bits(&a.t_crust), bits(&b.t_crust), "crust moved");
    assert_eq!(bits(&a.geotherm), bits(&b.geotherm), "the geotherm moved");
    assert_eq!(a.recv, b.recv, "the receiver export moved");
    assert_eq!(a.lake, b.lake, "the lake mask moved");
    assert_eq!(a.strata, b.strata, "the strata record moved");

    // And the LATERAL half of the flow record is untouched — only the vertical
    // faces the head field owns were added.
    let lateral = |f: &dc_worldgen::deeptime::DeepField| {
        f.flux
            .entries()
            .iter()
            .filter(|e| !e.face.is_vertical())
            .copied()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        lateral(&a),
        lateral(&b),
        "the routed flux changed — the head field touched lateral routing, which \
         is exactly what this slice must not do"
    );
}

/// Two runs of the same world plant the same field — no wall clock, no ambient
/// entropy, and no data-dependent iteration count in the relaxation.
#[test]
fn the_head_field_is_deterministic_on_a_repeated_seed() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let a = build_field_cfg(&pregen.grid, &cfg);
    let b = build_field_cfg(&pregen.grid, &cfg);
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&a.head), bits(&b.head));
    assert_eq!(a.flux.entries(), b.flux.entries());
}

/// The parallel per-cell erosion phases must not perturb the field either — it is
/// relaxed off their byte-identical outputs, in a fixed scalar sweep order.
#[test]
fn parallel_and_scalar_plant_the_same_head_field() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let a = run_cells(&pregen.grid, &cfg, false);
    let b = run_cells(&pregen.grid, &cfg, true);
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&a.grid.head), bits(&b.grid.head));
    assert_eq!(a.flux.entries(), b.flux.entries());
}

/// Residency is sacred (CLAUDE.md): the field's footprint must be counted into the
/// `DeepField`'s own resident number rather than hiding beside it.
#[test]
fn the_head_field_reports_its_own_resident_cost() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let on = build_field_cfg(&pregen.grid, &cfg);
    let off = build_field_cfg(
        &pregen.grid,
        &DeepConfig {
            head_field: false,
            ..cfg
        },
    );
    let head_bytes = on.head.len() * std::mem::size_of::<f64>();
    assert!(head_bytes > 0);
    let vertical = on.flux.census().vertical;
    let delta = on.resident_bytes() - off.resident_bytes();
    // Exactly the plane plus the vertical entries it caused — nothing else grew.
    assert_eq!(
        delta,
        head_bytes + vertical * std::mem::size_of::<dc_worldgen::deeptime::FluxEntry>(),
        "the head field's residency is not accounted for: plane {head_bytes} B, \
         {vertical} vertical entries, measured delta {delta} B"
    );
    println!(
        "head field residency: plane {head_bytes} B + {vertical} vertical entries \
         = {delta} B on a {}-cell grid",
        on.head.len()
    );
}
