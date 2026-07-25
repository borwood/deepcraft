//! **FLOW slice 1 — the recording half** (`docs/design/flow.md`, journal/0096).
//!
//! The acceptance question this suite answers is not "does it compile" but *"can
//! the record say a thing a receiver tree cannot say?"* A tree represents
//! convergence; it **structurally cannot represent divergence at all**. So the
//! divergence count is the falsifier: if it is zero, the primitive did not
//! actually change and the slice failed, however green everything else is.
//!
//! These run a small deep-time world (fast); the **production**-world numbers —
//! seed 1337, `Extent::Medium`, production flags — are the acceptance proper and
//! live in `examples/flux_record_probe.rs`.

use dc_worldgen::deeptime::{
    DeepConfig, FaceKey, FlowCause, FlowForm, FluidId, build_field_cfg, production_config,
    run_cells, slot_for_chapter,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 0x0F10_0057_1CE1;

fn small_world() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

/// A production-shaped config on a small grid: every flag the shipped world runs,
/// so the record under test is the record the world keeps.
fn cfg_for(cells: &CellGrid) -> DeepConfig {
    production_config(cells, SEED)
}

// ---------------------------------------------------------------------------
// The load-bearing pair: divergence and convergence.

/// **THE test of the slice.** A cell whose flux left through **two or more
/// faces** in one chapter is divergence — a distributary, a braid, a fan head.
/// `recv: Vec<i32>` is one out-edge per cell, so a receiver tree's count of this
/// is identically zero, by construction, forever. A non-zero count here is the
/// proof that the representation, not merely the plumbing, changed.
#[test]
fn divergence_is_representable_which_a_receiver_tree_forbids() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let census = f.flux.census();
    assert!(
        census.entries > 0,
        "the flow record is empty — the pass did not run"
    );
    assert!(
        census.divergent > 0,
        "ZERO divergence: every cell has at most one out-face per chapter, which \
         is exactly what the receiver tree already did. The slice has not \
         achieved its purpose. (entries {}, max out-faces {})",
        census.entries,
        census.max_out_faces
    );
    assert!(
        census.max_out_faces >= 2,
        "max out-faces {} — a tree's ceiling",
        census.max_out_faces
    );
}

/// Convergence — many in-faces — is the half a tree *could* already do; it must
/// still be present, and it must be readable as a query over the neighbours'
/// stored half-faces rather than a second stored copy.
#[test]
fn convergence_is_representable() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let census = f.flux.census();
    assert!(
        census.convergent > 0,
        "no confluence anywhere in the record (entries {})",
        census.entries
    );
    assert!(census.max_in_faces >= 2);
}

// ---------------------------------------------------------------------------
// The deepest defect: the process was run 200× and only the last frame kept.

/// The record must carry **every chapter**, not the final routing. This is the
/// defect the arc exists to fix: `recv`/`area` are documented as "the **last**
/// routing", so every paleo-flow signature the design wants was computed and
/// thrown away.
#[test]
fn per_chapter_history_is_retained_not_just_the_final_epoch() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    assert!(cfg.chapters > 1, "the fixture must span several chapters");
    let f = build_field_cfg(&pregen.grid, &cfg);

    let mut seen = vec![false; 256];
    for e in f.flux.entries() {
        seen[e.chapter as usize] = true;
    }
    let chapters_present = seen.iter().filter(|s| **s).count();
    assert_eq!(
        chapters_present, cfg.chapters as usize,
        "the record holds {chapters_present} of {} chapters",
        cfg.chapters
    );

    // And the history must be *history* — some cell's flow actually moved between
    // chapters, or "per chapter" would be an expensive way to store one frame.
    let last = f.flux.chapters - 1;
    let moved = (0..f.flux.cells()).any(|i| {
        let early: Vec<FaceKey> = f.flux.out_faces(i, 0).map(|e| e.face).collect();
        let late: Vec<FaceKey> = f.flux.out_faces(i, last).map(|e| e.face).collect();
        !early.is_empty() && !late.is_empty() && early != late
    });
    assert!(
        moved,
        "no cell's flow path differs between chapter 0 and {last}"
    );
}

/// The atom's stratum slot is **derived, not stored** (S-2): `DepUnit::chapter`
/// already stamps it. The derivation must land on a unit of that chapter, and
/// must answer `None` — honestly — when erosion has stripped that chapter away,
/// where a stored index would have gone stale and pointed at a stranger.
#[test]
fn the_stratum_slot_is_derived_from_the_chapter_stamp() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let mut checked = 0usize;
    let mut absent = 0usize;
    for (i, strata) in f.strata.iter().enumerate() {
        for k in 0..f.flux.chapters {
            if f.flux.out_face_count(i, k) == 0 {
                continue;
            }
            match slot_for_chapter(strata, k) {
                Some(slot) => {
                    assert_eq!(
                        strata.units[slot].chapter, k,
                        "slot {slot} of cell {i} is not chapter {k}"
                    );
                    checked += 1;
                }
                // The chapter flowed but left no surviving stratum: erosion took
                // it. That is an unconformity, and `None` is the right answer.
                None => absent += 1,
            }
        }
    }
    assert!(checked > 0, "no chapter resolved to a slot");
    println!("slots resolved {checked}, chapters eroded away (honest None) {absent}");
}

// ---------------------------------------------------------------------------
// Seamlessness: the face is shared, so it is stored once.

/// `A.east` **is** `B.west`. The in-flux of B through that face must be exactly
/// the entry A stored — one number, not two that could drift. This is why the
/// seamlessness requirement and the divergence requirement have the same answer.
///
/// **Paired by chapter**, never by slot index (flow.md § 1.2: surfaces are
/// diachronous, so slot N in two columns is not one moment).
#[test]
fn a_face_is_shared_so_bs_in_flux_is_exactly_as_stored() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let rec = &f.flux;
    let w = rec.w;
    let mut matched = 0usize;
    for i in 0..rec.cells() {
        for k in 0..rec.chapters {
            for e in rec.out_faces(i, k) {
                let Some((dx, dy)) = e.face.delta() else {
                    continue; // vertical / boundary faces have no sibling cell
                };
                let (gx, gy) = ((i % w) as i32, (i / w) as i32);
                let (nx, ny) = (gx + dx, gy + dy);
                if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= w {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                let ins = rec.in_faces(j, k);
                let found = ins.iter().any(|(from, entry)| *from == i && entry == e);
                assert!(
                    found,
                    "cell {i}'s {} face in chapter {k} is not visible as an \
                     in-face of {j} — the shared face disagreed with itself",
                    e.face.label()
                );
                matched += 1;
            }
        }
    }
    assert!(matched > 0, "no lateral faces to check");
}

// ---------------------------------------------------------------------------
// What is honestly EMPTY — asserted, so a later slice cannot populate it silently.

/// **Slice 1's honest empty, deleted on purpose.** This test used to assert
/// `census.vertical == 0` — the surface solve had no vertical term, so there was
/// no number to write and fabricating one would have been worse than the zero
/// (A-1). It was written to force the slice that filled them to come here and
/// remove it deliberately; **continuation (a) is that slice**, and the head field
/// (`dc:field/head`) is what made an honest number exist. The positive claim now
/// lives in `tests/head_field.rs::the_vertical_faces_are_no_longer_zero`.
///
/// What survives here is what did **not** change: the face vocabulary still spans
/// all three families (flow.md § 2.3 foreclosure ①), and the atom's remaining
/// seams are still exactly one fluid, one form and one mover — the head field
/// records **free-phase** exchange across a column's top, and the free↔bound
/// occupancy edge is still continuation (c)'s to switch on.
#[test]
fn the_face_vocabulary_still_spans_three_families_and_the_atom_seams_are_intact() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    assert!(FaceKey::Down.is_vertical() && FaceKey::Up.is_vertical());
    assert!(FaceKey::Ocean.is_boundary() && FaceKey::E.is_lateral());

    assert!(f.flux.entries().iter().all(|e| e.fluid == FluidId::WATER));
    assert!(f.flux.entries().iter().all(|e| e.form == FlowForm::Free));
    assert!(
        f.flux
            .entries()
            .iter()
            .all(|e| e.cause == FlowCause::Fluvial)
    );
}

/// The lateral faces carry real magnitude, the boundary faces carry the discharge
/// that left the model, and the seaward boundary is actually populated (a world
/// whose flow never reached the sea would be a broken solve, not a sparse one).
#[test]
fn lateral_and_boundary_faces_are_populated() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let census = f.flux.census();
    assert!(census.lateral > 0, "no lateral flux at all");
    assert!(
        census.boundary_ocean > 0 || census.boundary_base > 0,
        "nothing ever reached base level"
    );
    assert!(
        f.flux.entries().iter().all(|e| e.magnitude > 0.0),
        "a zero-magnitude face was stored — the record must be sparse"
    );
}

// ---------------------------------------------------------------------------
// Agreement with the authority it replaces (a summary must agree, ARCHITECTURE.md).

/// The retired receiver tree must be a **shadow of** the new record, not a rival:
/// the final chapter's out-faces of a cell must contain the face `recv` points
/// along. If they ever disagreed, one of them would be lying about the same solve.
#[test]
fn the_receiver_export_agrees_with_the_final_chapters_faces() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let w = f.flux.w;
    let last = f.flux.chapters - 1;
    let mut checked = 0usize;
    for i in 0..f.flux.cells() {
        let rc = f.recv[i];
        if rc < 0 {
            continue;
        }
        let j = rc as usize;
        let (dx, dy) = (
            (j % w) as i32 - (i % w) as i32,
            (j / w) as i32 - (i / w) as i32,
        );
        let has = f
            .flux
            .out_faces(i, last)
            .any(|e| e.face.delta() == Some((dx, dy)));
        assert!(
            has,
            "cell {i} routes to {j} in the final epoch, but the final chapter's \
             faces do not include that direction"
        );
        checked += 1;
    }
    assert!(checked > 0);
}

// ---------------------------------------------------------------------------
// Byte-identity: the record is a sidecar, not a change to the world.

/// The whole slice must not move a single voxel. The record reads the drainage
/// solve's outputs and writes only itself — so with the flag on and off, the
/// surface planes, the regolith, the strata record and the old drainage export
/// must be **bit-for-bit** the same.
///
/// (The cross-commit golden proof is separate and untouched:
/// `providers_golden.rs::the_production_world_still_hashes_to_the_pre_slice_goldens`.)
#[test]
fn the_flow_record_is_a_sidecar_and_the_world_is_byte_identical() {
    let pregen = small_world();
    let on = DeepConfig {
        flow_record: true,
        ..cfg_for(&pregen.grid)
    };
    let off = DeepConfig {
        flow_record: false,
        ..cfg_for(&pregen.grid)
    };
    let a = build_field_cfg(&pregen.grid, &on);
    let b = build_field_cfg(&pregen.grid, &off);

    assert!(!a.flux.is_empty(), "the on-run recorded nothing");
    assert!(b.flux.is_empty(), "the off-run recorded something");

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
}

/// Two runs of the same world produce the same record — no wall clock, no
/// ambient entropy, no iteration-order dependence in the accumulator.
#[test]
fn the_flow_record_is_deterministic_on_a_repeated_seed() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let a = build_field_cfg(&pregen.grid, &cfg);
    let b = build_field_cfg(&pregen.grid, &cfg);
    assert_eq!(a.flux.len(), b.flux.len());
    assert_eq!(a.flux.entries(), b.flux.entries());
}

/// The record is the ritual's largest keepsake; its footprint must be reportable
/// and must be counted into the field's own resident number rather than hiding.
#[test]
fn the_record_reports_its_own_resident_cost() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let f = build_field_cfg(&pregen.grid, &cfg);
    let flux_bytes = f.flux.resident_bytes();
    assert!(flux_bytes > 0);
    assert!(
        f.resident_bytes() > flux_bytes,
        "the field's resident total does not include the flow record"
    );
    // 16 bytes an entry plus the CSR index, and nothing else.
    assert_eq!(
        flux_bytes,
        std::mem::size_of::<dc_worldgen::deeptime::FluxRecord>()
            + f.flux.len() * 16
            + (f.flux.cells() + 1) * 4
    );
}

/// The parallel per-cell phases must not perturb the record either (the
/// accumulator runs off their byte-identical outputs).
#[test]
fn parallel_and_scalar_record_the_same_flux() {
    let pregen = small_world();
    let cfg = cfg_for(&pregen.grid);
    let a = run_cells(&pregen.grid, &cfg, false);
    let b = run_cells(&pregen.grid, &cfg, true);
    assert_eq!(a.flux.entries(), b.flux.entries());
}
