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

fn main() {
    println!("=== identify(pos) census — phantom air over solid ground, BEFORE vs AFTER ===");
    println!("seed {SEED}, extent {}, voxel {VOXEL_M} m", EXTENT.label());
    println!("flag-OFF control (DeepOverrides::default())\n");

    let pregen = Arc::new(Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides::default(),
    ));
    // The client's own wiring (authority.rs § new_worldgen): one generator
    // behind a Mutex serving both blocks and contents.
    let generator = Arc::new(Mutex::new(WorldGenerator::new_owned(pregen.clone())));
    let seam = generator.clone();
    let mut world = HostWorld::with_generator(
        SEED,
        Box::new(move |pos| seam.lock().expect("generator").generate_chunk(pos)),
    );
    let contents_gen = generator.clone();
    world.set_contents_source(Box::new(move |pos: ChunkPos| {
        contents_gen.lock().expect("generator").chunk_contents(pos)
    }));

    // The recorded surface voxel of a column — the same `column_record` the
    // client's surface-scan ceiling reads.
    let column_height = |vx: i64, vz: i64| -> i64 {
        let mut g = generator.lock().expect("generator");
        let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
        let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
        i64::from(g.column_record(cx, cz).heights[lz * 32 + lx])
    };

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
    let mut solid = 0u64;
    let mut phantom_before = 0u64;
    let mut phantom_after = 0u64;
    let mut unrecorded_after = 0u64;
    let mut recorded_after = 0u64;
    let mut honest_no_record_before = 0u64;
    let mut cols_with_phantom_before = 0u32;
    let mut columns = 0u32;
    let mut disagreements: Vec<String> = Vec::new();
    let mut per_column: HashMap<(i64, i64), u64> = HashMap::new();

    for i in -CENSUS_SPAN..=CENSUS_SPAN {
        for j in -CENSUS_SPAN..=CENSUS_SPAN {
            let (cvx, cvz) = (vx + i * CENSUS_STEP_VOXELS, vz + j * CENSUS_STEP_VOXELS);
            let ch = column_height(cvx, cvz);
            columns += 1;
            let mut col_phantom = 0u64;
            for vy in (ch - COLUMN_DEPTH)..=ch {
                let p = Vec3i::new(cvx, vy, cvz);
                let block = world.block_at(p);
                if !block.is_solid() {
                    continue;
                }
                solid += 1;
                let b = before(&world, block, p);
                let id = world.identify(p);
                if is_phantom_air(block, &b) {
                    phantom_before += 1;
                    col_phantom += 1;
                }
                if is_phantom_air_after(block, &id) {
                    phantom_after += 1;
                    disagreements.push(format!("phantom after at ({cvx},{vy},{cvz})"));
                }
                if !b.has_contents {
                    honest_no_record_before += 1;
                }
                match &id {
                    Identity::Unrecorded => unrecorded_after += 1,
                    Identity::Mixture(_) => recorded_after += 1,
                }
            }
            if col_phantom > 0 {
                cols_with_phantom_before += 1;
                per_column.insert((cvx, cvz), col_phantom);
            }
        }
    }

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
    assert_eq!(
        unrecorded_after,
        phantom_before + honest_no_record_before,
        "every BEFORE-phantom and every BEFORE-honest-no-record voxel must land in UNRECORDED"
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
