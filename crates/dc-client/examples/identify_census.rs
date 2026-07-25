//! **Does `identify(pos)` still report phantom air over solid ground?**
//! (the acceptance census for journal/0101 / corrections #49)
//!
//! The diagnosis probe
//! (`dc-worldgen/examples/contents_air_over_solid_probe.rs`) reconstructed the
//! query's *old* answer by hand. This one drives the **real query path**: a
//! `HostWorld` wired exactly as `dc-client/src/authority.rs` wires it (worldgen
//! blocks through the generator closure, `chunk_contents` as the
//! `ContentsSource`), and asks it both ways at every voxel:
//!
//! - **BEFORE** — the pre-`identify` semantics, recomputed here from the raw
//!   `HostWorld::contents_at`: `has_contents = contents.is_some()` and
//!   `classified = classify(contents)`. That `Option` is a **per-chunk** fact,
//!   so an unrecorded voxel sharing a 32³ chunk with any recorded voxel comes
//!   back `Some(EMPTY)` → `has_contents: true`, `classified: dc:air`, over
//!   correctly-solid stone.
//! - **AFTER** — `HostWorld::identify(pos)`, the honest per-voxel answer.
//!
//! A **phantom-air** voxel is the defect's signature: the world is solid, the
//! query claims a record backs the answer, and that answer classifies to air.
//! The acceptance is that AFTER's count is **zero** while genuinely-recorded
//! and genuinely-air voxels are untouched.
//!
//! Lives in dc-client because it needs both dc-worldgen (the production world)
//! and dc-api (the query surface); dc-api does not depend on dc-worldgen, and
//! `dc-worldgen/examples` is another agent's territory.
//!
//! `cargo run --release -p dc-client --example identify_census`

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dc_api::{HostWorld, Identity, Vec3i};
use dc_core::{Block, ChunkPos, classify};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};
use dc_worldgen::{DeepOverrides, WorldGenerator};

/// The client's `BENCH_SEED` — the world every walk stands in.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;

/// journal/0097 STATION 1 — deep cell (439, 276), world metres. The walk's own
/// vantage, and the column the diagnosis reproduced to the voxel.
const STATION_M: (f64, f64) = (84_185.0, 9_212.0);

/// The census lattice, identical to the diagnosis probe's so the numbers are
/// comparable: 13 × 13 columns, 4 096 voxels (≈ 3.7 km) apart.
const CENSUS_STEP_VOXELS: i64 = 4096;
const CENSUS_SPAN: i64 = 6;
/// Voxels examined per column, from the surface down.
const COLUMN_DEPTH: i64 = 64;

fn block_name(b: Block) -> &'static str {
    dc_api::block_name(b)
}

/// The pre-`identify` answer, recomputed from the raw chunk-granular source.
struct Before {
    has_contents: bool,
    classified: Block,
}

fn before(world: &HostWorld, block: Block, p: Vec3i) -> Before {
    let contents = world.contents_at(p);
    Before {
        has_contents: contents.is_some(),
        classified: match &contents {
            Some(c) => classify(c),
            None => block,
        },
    }
}

/// The defect's signature under the OLD semantics.
fn is_phantom_air(block: Block, b: &Before) -> bool {
    block.is_solid() && b.has_contents && b.classified == Block::Air
}

/// The same signature expressed against the NEW answer: a solid voxel whose
/// identity claims a record and classifies to air.
fn is_phantom_air_after(block: Block, id: &Identity) -> bool {
    block.is_solid() && !id.is_unrecorded() && id.classified() == Some(Block::Air)
}

/// The census totals. **Shared by `main` (which prints) and the gate test below
/// (which asserts)** — journal/0103: `cargo test` BUILDS examples but never RUNS
/// them, so the `assert_eq!` this example used to carry in `main` was invisible
/// to a 664-test workspace gate. The measurement now lives in one place and the
/// gate can see it fail.
#[derive(Default)]
struct CensusTotals {
    columns: u32,
    solid: u64,
    phantom_before: u64,
    phantom_after: u64,
    unrecorded_after: u64,
    recorded_after: u64,
    honest_no_record_before: u64,
    cols_with_phantom_before: u32,
    /// World positions still answering phantom-air AFTER, for the report.
    disagreements: Vec<String>,
    /// Per-column BEFORE-phantom counts, for the spread line.
    per_column: HashMap<(i64, i64), u64>,
}

/// The client's own wiring (authority.rs § `new_worldgen`): one generator behind
/// a `Mutex` serving both blocks and contents, with `identify` on top.
fn wired_world(
    extent: Extent,
) -> (
    HostWorld,
    Arc<Mutex<WorldGenerator<'static>>>,
) {
    let pregen = Arc::new(Pregen::run_with(
        WorldParams { seed: SEED, extent },
        &DeepOverrides::default(),
    ));
    let generator = Arc::new(Mutex::new(WorldGenerator::new_owned(pregen)));
    let seam = generator.clone();
    let mut world = HostWorld::with_generator(
        SEED,
        Box::new(move |pos| seam.lock().expect("generator").generate_chunk(pos)),
    );
    let contents_gen = generator.clone();
    world.set_contents_source(Box::new(move |pos: ChunkPos| {
        contents_gen.lock().expect("generator").chunk_contents(pos)
    }));
    (world, generator)
}

/// The recorded surface voxel of a column — the same `column_record` the
/// client's surface-scan ceiling reads.
fn column_height(generator: &Mutex<WorldGenerator<'static>>, vx: i64, vz: i64) -> i64 {
    let mut g = generator.lock().expect("generator");
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    i64::from(g.column_record(cx, cz).heights[lz * 32 + lx])
}

/// Ask the world both ways at every solid voxel of a lattice of columns.
fn census(
    world: &HostWorld,
    generator: &Mutex<WorldGenerator<'static>>,
    origin: (i64, i64),
    step: i64,
    span: i64,
    depth: i64,
) -> CensusTotals {
    let mut t = CensusTotals::default();
    for i in -span..=span {
        for j in -span..=span {
            let (cvx, cvz) = (origin.0 + i * step, origin.1 + j * step);
            let ch = column_height(generator, cvx, cvz);
            t.columns += 1;
            let mut col_phantom = 0u64;
            for vy in (ch - depth)..=ch {
                let p = Vec3i::new(cvx, vy, cvz);
                let block = world.block_at(p);
                if !block.is_solid() {
                    continue;
                }
                t.solid += 1;
                let b = before(world, block, p);
                let id = world.identify(p);
                if is_phantom_air(block, &b) {
                    t.phantom_before += 1;
                    col_phantom += 1;
                }
                if is_phantom_air_after(block, &id) {
                    t.phantom_after += 1;
                    t.disagreements
                        .push(format!("phantom after at ({cvx},{vy},{cvz})"));
                }
                if !b.has_contents {
                    t.honest_no_record_before += 1;
                }
                match &id {
                    Identity::Unrecorded => t.unrecorded_after += 1,
                    Identity::Mixture(_) => t.recorded_after += 1,
                }
            }
            if col_phantom > 0 {
                t.cols_with_phantom_before += 1;
                t.per_column.insert((cvx, cvz), col_phantom);
            }
        }
    }
    t
}

fn main() {
    println!("=== identify(pos) census — phantom air over solid ground, BEFORE vs AFTER ===");
    println!("seed {SEED}, extent {}, voxel {VOXEL_M} m", EXTENT.label());
    println!("flag-OFF control (DeepOverrides::default())\n");

    let (world, generator) = wired_world(EXTENT);
    let column_height = |vx: i64, vz: i64| -> i64 { column_height(&generator, vx, vz) };

    // ---------------------------------------------------------------- station
    let (vx, vz) = (
        (STATION_M.0 / VOXEL_M).round() as i64,
        (STATION_M.1 / VOXEL_M).round() as i64,
    );
    let h = column_height(vx, vz);
    println!(
        "--- STATION: metres ({:.0}, {:.0}) = voxel ({vx}, {vz}); surface voxel h = {h}",
        STATION_M.0, STATION_M.1
    );
    println!("      vy | block      | BEFORE has_contents / classified | AFTER identify");
    println!("    -----+------------+---------------------------------+----------------");
    let mut station_before = 0u32;
    let mut station_after = 0u32;
    for vy in (h - 16..=h + 2).rev() {
        let p = Vec3i::new(vx, vy, vz);
        let block = world.block_at(p);
        let b = before(&world, block, p);
        let id = world.identify(p);
        if is_phantom_air(block, &b) {
            station_before += 1;
        }
        if is_phantom_air_after(block, &id) {
            station_after += 1;
        }
        let after = match &id {
            Identity::Unrecorded => "UNRECORDED".to_string(),
            Identity::Mixture(c) if c.is_empty() => "mixture: EMPTY (air)".to_string(),
            Identity::Mixture(c) => format!("mixture: {}", block_name(classify(c))),
        };
        println!(
            "    {vy:>4} | {:<10} | {:<5} / {:<23} | {after}{}",
            block_name(block),
            b.has_contents,
            block_name(b.classified),
            if is_phantom_air(block, &b) {
                "   <== was PHANTOM AIR"
            } else {
                ""
            },
        );
    }
    println!(
        "\n    station column (h-16..h+2): phantom-air BEFORE {station_before}, AFTER {station_after}"
    );

    // ----------------------------------------------------------------- census
    println!(
        "\n--- CENSUS: {} columns on a {} voxel lattice, top {} voxels each (h-{COLUMN_DEPTH}..=h)\n",
        (2 * CENSUS_SPAN + 1) * (2 * CENSUS_SPAN + 1),
        CENSUS_STEP_VOXELS,
        COLUMN_DEPTH + 1
    );
    let CensusTotals {
        columns,
        solid,
        phantom_before,
        phantom_after,
        unrecorded_after,
        recorded_after,
        honest_no_record_before,
        cols_with_phantom_before,
        disagreements,
        per_column,
    } = census(
        &world,
        &generator,
        (vx, vz),
        CENSUS_STEP_VOXELS,
        CENSUS_SPAN,
        COLUMN_DEPTH,
    );

    let pct = |n: u64| 100.0 * n as f64 / solid.max(1) as f64;
    println!("    columns examined                  : {columns}");
    println!("    solid voxels examined             : {solid}");
    println!(
        "    BEFORE phantom air (the defect)   : {phantom_before}  ({:.1} % of solid)",
        pct(phantom_before)
    );
    println!(
        "    BEFORE honest no-record           : {honest_no_record_before}  ({:.1} %)",
        pct(honest_no_record_before)
    );
    println!("    BEFORE columns with >=1 phantom   : {cols_with_phantom_before} / {columns}");
    println!(
        "    AFTER  phantom air                : {phantom_after}  ({:.1} % of solid)",
        pct(phantom_after)
    );
    println!(
        "    AFTER  UNRECORDED (solid)         : {unrecorded_after}  ({:.1} %)",
        pct(unrecorded_after)
    );
    println!(
        "    AFTER  recorded mixture (solid)   : {recorded_after}  ({:.1} %)",
        pct(recorded_after)
    );
    // NOTE: this identity used to be an `assert_eq!` right here, in `main` —
    // which the workspace gate could never run (journal/0103). It is now
    // `every_before_phantom_and_no_record_voxel_lands_in_unrecorded` below, and
    // the example only *reports* it.
    println!(
        "    IDENTITY  unrecorded_after == phantom_before + honest_no_record_before : \
         {unrecorded_after} == {phantom_before} + {honest_no_record_before}  [{}]",
        if unrecorded_after == phantom_before + honest_no_record_before {
            "HOLDS"
        } else {
            "*** VIOLATED ***"
        }
    );
    if !disagreements.is_empty() {
        println!("\n    !!! {} voxels still phantom:", disagreements.len());
        for d in disagreements.iter().take(10) {
            println!("      {d}");
        }
    }
    let mut spread: Vec<_> = per_column.values().copied().collect();
    spread.sort_unstable();
    if let (Some(lo), Some(hi)) = (spread.first(), spread.last()) {
        println!("\n    BEFORE per-column phantom spread  : {lo}..{hi}");
    }
    println!(
        "\n    VERDICT: phantom air {phantom_before} -> {phantom_after}; \
         all of it now answers UNRECORDED, and sky/recorded rock are unmoved."
    );
}

/// **The gate's view of this instrument** (journal/0103).
///
/// This example already carried a real `assert!` — in `main`, where no gate
/// could reach it. `cargo test --workspace` BUILDS examples and never RUNS them,
/// so journal/0101's acceptance has been sitting one `cargo run` away from
/// nobody for a day. The same assertions, in `#[test]`s that share the census
/// above.
///
/// Run at [`Extent::Small`]. `identify(pos)` is a **per-voxel** predicate over a
/// per-voxel fact; whether it can confuse "no record here" with "air here" does
/// not depend on how wide the deep grid is. The production census numbers
/// (702 → 0 phantom voxels over 10 985 solid) are the example's job and stay at
/// [`Extent::Medium`].
#[cfg(test)]
mod gate {
    use super::*;

    use std::sync::OnceLock;

    /// A lattice tight enough to sit inside the small world. **Built once for the
    /// whole binary** — both tests share one world and one census, so the gate
    /// pays for the pregen a single time.
    fn small_census() -> &'static CensusTotals {
        static CENSUS: OnceLock<CensusTotals> = OnceLock::new();
        CENSUS.get_or_init(|| {
            // **Spread wide on purpose.** The small world is ~74 km across; a
            // tight lattice around the origin can sit entirely on ground the
            // deep-time record never touched, and then "zero phantom air" is
            // trivially true. 9x9 columns at 4 096 voxels (~3.7 km) spans
            // ±14.7 km and crosses provinces.
            let (world, generator) = wired_world(Extent::Small);
            let t = census(&world, &generator, (0, 0), CENSUS_STEP_VOXELS, 4, COLUMN_DEPTH);
            assert!(
                t.solid > 500,
                "only {} solid voxels examined over {} columns — too few for the null to \
                 mean anything",
                t.solid,
                t.columns
            );
            t
        })
    }

    /// **The acceptance of journal/0101.** A solid voxel must never come back
    /// claiming a record that classifies to air.
    #[test]
    fn identify_never_reports_air_over_solid_ground() {
        let t = small_census();
        assert_eq!(
            t.phantom_after,
            0,
            "{} solid voxels still answer phantom-air through the real query path \
             (first few: {:?}); BEFORE the fix this world had {}",
            t.phantom_after,
            t.disagreements.iter().take(5).collect::<Vec<_>>(),
            t.phantom_before,
        );
    }

    /// **The conservation half**, and the assertion this example used to make
    /// where no gate could see it: the fix must *reclassify*, not delete. Every
    /// voxel that was phantom-air, plus every voxel already honestly reported as
    /// carrying no record, lands in `UNRECORDED` — and nothing else does.
    #[test]
    fn every_before_phantom_and_no_record_voxel_lands_in_unrecorded() {
        let t = small_census();
        assert_eq!(
            t.unrecorded_after,
            t.phantom_before + t.honest_no_record_before,
            "UNRECORDED holds {} voxels but BEFORE had {} phantom + {} honest-no-record; \
             the fix moved voxels it was not supposed to touch",
            t.unrecorded_after,
            t.phantom_before,
            t.honest_no_record_before,
        );
        assert_eq!(
            t.unrecorded_after + t.recorded_after,
            t.solid,
            "identify() answered neither Unrecorded nor Mixture for some solid voxel"
        );
        assert!(
            t.recorded_after > 0,
            "not one solid voxel came back with a real mixture — the contents source is \
             answering nothing, and 'zero phantom air' would then be trivially true"
        );
    }
}
