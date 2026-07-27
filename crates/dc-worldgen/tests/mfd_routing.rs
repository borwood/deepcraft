//! **FLOW continuation (b) — the MFD solve** (`docs/design/flow.md` § 2.6,
//! journal/0109).
//!
//! Slice 1 gave the record a shape that *admits* divergence. It did not make
//! divergence happen within an epoch: the solve handed back one receiver per cell,
//! so every divergence in the archive was **temporal** (avulsion across a
//! chapter's 25 epochs) and simultaneous divergence — a delta with two channels
//! flowing at once — was structurally impossible at any cadence.
//!
//! This suite's load-bearing pair is therefore a **before/after on the same
//! world**: with `mfd` off the simultaneous-divergence count must be *exactly*
//! zero (not small — zero, by construction), and with it on it must be positive.
//! A test that only asserted the second half would pass against a bug that
//! counted temporal divergence twice.
//!
//! The rest are the invariants MFD had to re-derive for a DAG: the partition sums
//! to the whole, mass is still conserved down a multi-receiver chain, every edge
//! descends the potential (which is what makes the priority-flood order still a
//! topological order), and the incision clamp still forbids a pit. Production-scale
//! numbers live in `examples/mfd_probe.rs`.

use dc_worldgen::deeptime::{
    self, DeepConfig, DeepField, DeepRun, build_field_cfg, production_config,
    production_config_with, run_cells,
};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

mod providers_common;

const SEED: u64 = 0x0FD8_1337_0B0B;
/// D8 direction table — the same order as `FaceKey::lateral` and the solve's own
/// `NEIGH8`, so a face code indexes straight into it.
const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn small_world() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}

fn cfg_for(cells: &CellGrid, mfd: bool) -> DeepConfig {
    DeepConfig {
        mfd,
        // **Held scalar on purpose** (Movement 2b, journal/0110). This suite asks
        // a question about *routing*; material-aware transport changes what the
        // routing carries, and leaving it on would mean every number here moved
        // for two reasons at once — including the pre-MFD fixed point below, which
        // would stop being reachable at all.
        material_transport: false,
        ..production_config(cells, SEED)
    }
}

fn field(cells: &CellGrid, mfd: bool) -> DeepField {
    build_field_cfg(cells, &cfg_for(cells, mfd))
}

fn run(cells: &CellGrid, mfd: bool) -> DeepRun {
    run_cells(cells, &cfg_for(cells, mfd), false)
}

// ---------------------------------------------------------------------------
// The load-bearing pair.

/// **THE test of the slice**, and it is a pair because either half alone is
/// gameable. With single-receiver routing a cell has exactly one out-face per
/// epoch, so the simultaneous count is **identically zero** — that is the
/// structural claim flow.md § 2.6 makes, asserted rather than quoted. With MFD on
/// the same world must produce a positive count, which is the capability.
#[test]
fn simultaneous_divergence_is_zero_without_mfd_and_positive_with_it() {
    let pregen = small_world();
    let off = field(&pregen.grid, false);
    let on = field(&pregen.grid, true);

    assert_eq!(
        off.flux.simultaneous.cell_epochs, 0,
        "single-receiver routing produced a simultaneous divergence — it cannot; \
         either the counter or the routing is wrong"
    );
    assert_eq!(off.flux.simultaneous.cell_chapters, 0);
    assert_eq!(off.flux.simultaneous.max_out_faces, 0);

    assert!(
        on.flux.simultaneous.cell_epochs > 0,
        "MFD produced NO simultaneous divergence: the partition never split a \
         cell's discharge, so the slice achieved nothing the receiver tree did not"
    );
    assert!(on.flux.simultaneous.max_out_faces >= 2);
    assert!(on.flux.simultaneous.cell_chapters > 0);
}

/// The **temporal** divergence the record already had must survive MFD, not be
/// replaced by it. They are different quantities over the same keys and conflating
/// them is the confusion § 2.6 exists to prevent, so both are asserted alive — and
/// the simultaneous set must be a *subset* of the temporal one, which is a real
/// consistency check between the counter and the record it rides on.
#[test]
fn temporal_divergence_survives_alongside_the_simultaneous_kind() {
    let pregen = small_world();
    let on = field(&pregen.grid, true);
    let c = on.flux.census();
    assert!(c.divergent > 0, "the chapter-window divergence vanished");
    assert!(
        c.divergent as u64 >= on.flux.simultaneous.cell_chapters,
        "a (cell,chapter) that diverged *within an epoch* must also count as \
         diverging *within the chapter*: {} simultaneous vs {} temporal",
        on.flux.simultaneous.cell_chapters,
        c.divergent
    );
}

// ---------------------------------------------------------------------------
// The DAG invariants, read off the live solve rather than the aggregated archive.

/// **The partition sums to the whole, exactly.** Every unit of discharge a cell
/// accumulates leaves it across its faces with **no residue**: the per-face planes
/// the solve writes must total the cell's own drainage area to the bit-width they
/// are stored at.
///
/// This is the invariant a partition breaks silently. Normalised `f64` weights sum
/// to `1 ± 1 ulp`, so `Σ wₖ·q` is not `q`; the fix is that the last weighted
/// direction takes the residual instead of its weight. Without it the leak is
/// invisible per hop and unattributable after a thousand of them.
#[test]
fn the_partition_leaves_no_residue() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let out = r.erosion.out_face_area();
    let area = r.erosion.area();
    let n = area.len();
    assert_eq!(out.len(), n * 8, "the per-face plane must be n x 8");
    let mut checked = 0usize;
    for i in 0..n {
        let total: f64 = (0..8).map(|d| f64::from(out[i * 8 + d])).sum();
        if total == 0.0 {
            continue; // a sink: the discharge left through a boundary face.
        }
        // The planes are f32 (the record's own width), so the comparison is at
        // f32 resolution against an f64 accumulator — a relative bound, never an
        // absolute one, because drainage areas span six orders of magnitude.
        let rel = (total - area[i]).abs() / area[i].max(1.0);
        assert!(
            rel < 1e-5,
            "cell {i} routed {total} of its {} units of discharge (rel err {rel})",
            area[i]
        );
        checked += 1;
    }
    assert!(checked > 0, "no cell routed laterally at all");
}

/// **Every routed edge descends the potential** — which is precisely what makes
/// the priority-flood pop order a valid topological order of the flow graph, tree
/// or DAG. The single-receiver chain relied on a *tree* for its traversal; what
/// actually licensed it was the potential ordering, and MFD inherits that
/// unchanged.
///
/// Checked on the live final epoch (`filled` and the per-face planes as the solve
/// left them), not on the chapter-aggregated archive, because the archive sums 25
/// epochs of a *moving* terrain and cannot answer the question.
#[test]
fn every_routed_edge_descends_the_free_surface_potential() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let w = r.grid.w;
    let filled = r.erosion.filled();
    let out = r.erosion.out_face_area();
    let mut checked = 0usize;
    for i in 0..w * w {
        for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
            if out[i * 8 + d] <= 0.0 {
                continue;
            }
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let (nx, ny) = (x + dx, y + dy);
            assert!(
                nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < w,
                "cell {i} routed off the grid"
            );
            let j = ny as usize * w + nx as usize;
            assert!(
                filled[j] < filled[i],
                "cell {i} routed to {j} which is NOT lower on the filled potential \
                 ({} -> {}) — the flow graph has a cycle and the traversal order is \
                 no longer topological",
                filled[i],
                filled[j]
            );
            checked += 1;
        }
    }
    assert!(checked > 0);
}

/// **Mass down a DAG.** The header's falsifier — `Δ(ΣR + ΣH) = uplift + biotic` —
/// must still hold when a cell's outgoing load is split across several receivers.
#[test]
fn mass_is_conserved_with_mfd_on() {
    let pregen = small_world();
    let r = run(&pregen.grid, true);
    let residual = deeptime::total_mass(&r.grid) - r.mass_before - r.uplift_total - r.biotic_total;
    assert!(
        residual.abs() < 1.0,
        "mass leaked with MFD on: residual {residual} (uplift {}, biotic {})",
        r.uplift_total,
        r.biotic_total
    );
}

/// **The never-incise-below-the-LOWEST-receiver clamp.** Generalised from one
/// receiver to many, the guarantee is that a cell can still drain: it may be cut
/// down to, but not below, the surface of its lowest outlet. Cut past that and it
/// becomes a pit its own outlets cannot drain — the runaway knickpoint the clamp
/// exists to forbid.
///
/// Read off the final terrain: no interior cell may sit more than a metre below
/// **every** one of its eight neighbours.
#[test]
fn no_interior_cell_is_cut_below_all_of_its_neighbours() {
    let pregen = small_world();
    let f = field(&pregen.grid, true);
    let w = f.w;
    let mut pits = 0usize;
    let mut lake_pits = 0usize;
    let mut deepest = 0.0f64;
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let lowest = NEIGH8
                .into_iter()
                .map(|(dx, dy)| f.surf[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize])
                .fold(f64::INFINITY, f64::min);
            // Priority-flood tolerates shallow closed depressions (it fills rather
            // than carves), so the bar is a real pit, not a dimple.
            if f.surf[i] < lowest - 1.0 {
                pits += 1;
                deepest = deepest.max(lowest - f.surf[i]);
                if f.lake.get(i).copied().unwrap_or(false) {
                    lake_pits += 1;
                }
            }
        }
    }
    println!(
        "interior pits deeper than 1 m: {pits} (deepest {deepest:.2} m, of which \
         {lake_pits} are recorded lakes)"
    );
    assert_eq!(
        pits, 0,
        "MFD incision left {pits} interior cells more than a metre below every \
         neighbour (deepest {deepest:.2} m, {lake_pits} of them lakes) — the \
         multi-receiver clamp is not holding"
    );
}

// ---------------------------------------------------------------------------
// The identity path, and the argmax projection.

/// **MFD off is the pre-slice solve.** The partition is a second path beside the
/// single-receiver one, not a refactor of it, so the old world stays reachable —
/// and the flag genuinely moves the world when it is on, which is what stops the
/// identity claim from being vacuously true of a no-op.
#[test]
fn mfd_off_leaves_the_single_receiver_solve_untouched_and_on_moves_the_world() {
    let pregen = small_world();
    let a = field(&pregen.grid, false);
    let b = field(&pregen.grid, false);
    assert_eq!(a.surf, b.surf, "the off path is not deterministic");
    assert_eq!(a.recv, b.recv);

    let on = field(&pregen.grid, true);
    assert_ne!(
        a.surf, on.surf,
        "MFD changed no elevation anywhere: it is not reaching the erosion pass"
    );
}

/// **The pre-MFD fixed point is still reachable, and still exact.** MFD moved the
/// shipped world — that is the slice's whole point and the goldens moved with it —
/// but a moved golden is only an *authorized* move if the thing it used to
/// describe is still there. So the fixture world built with `mfd: false` must
/// reproduce the goldens as they stood on pre-MFD `main`, byte for byte, on both
/// the surface planes and the strata record.
///
/// It lives here rather than in `providers_golden.rs` on purpose: that file is the
/// cross-commit golden for the **shipped** configuration and its whole discipline
/// is that no slice has a reason to open it. This is a different claim — "the path
/// I did not take is unchanged" — and it belongs to the slice that introduced the
/// fork.
#[test]
fn the_single_receiver_path_still_hashes_to_the_pre_mfd_goldens() {
    use providers_common::{
        GOLDEN_RECORD_SINGLE_RECEIVER, GOLDEN_SURFACE_SINGLE_RECEIVER, SEED as GOLDEN_SEED,
        golden_pregen, record_fingerprint, surface_fingerprint,
    };
    let pregen = golden_pregen();
    let cfg = DeepConfig {
        mfd: false,
        // Pre-MFD `main` had no material-aware transport either (journal/0110
        // shipped it later), so reaching that fixed point means turning off both
        // forks, not one — and, since journal/0114, not three: it also predates the
        // erosional calibration, which multiplied `weathering` / `diffusion` /
        // `k_transport` / `k_bedrock` by 45. Reaching a fixed point means
        // reproducing ALL of the configuration it was captured under.
        material_transport: false,
        ..production_config_with(
            &pregen.grid,
            GOLDEN_SEED,
            &dc_worldgen::DeepOverrides {
                calibrated_rates: Some(false),
                ..Default::default()
            },
        )
    };
    let f = build_field_cfg(&pregen.grid, &cfg);
    assert_eq!(
        surface_fingerprint(&f),
        GOLDEN_SURFACE_SINGLE_RECEIVER,
        "the single-receiver surface no longer matches pre-MFD main — MFD did not \
         merely add a path, it perturbed the old one"
    );
    assert_eq!(
        record_fingerprint(&f),
        GOLDEN_RECORD_SINGLE_RECEIVER,
        "the single-receiver strata record no longer matches pre-MFD main"
    );
}

/// The **argmax projection**: `recv` under MFD is the direction carrying the
/// largest share, so it must still be among the faces the record kept — the same
/// shadow relation slice 1 asserted, which is what stops the exported drainage
/// network from becoming a rival authority (ARCHITECTURE.md, "a summary is not an
/// authority").
#[test]
fn the_receiver_projection_is_still_among_the_recorded_faces() {
    let pregen = small_world();
    let f = field(&pregen.grid, true);
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
        assert!(
            f.flux
                .out_faces(i, last)
                .any(|e| e.face.delta() == Some((dx, dy))),
            "cell {i}'s argmax receiver {j} is not among the final chapter's faces"
        );
        checked += 1;
    }
    assert!(checked > 0);
}
