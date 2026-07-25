//! **Does `world_get_contents` report `dc:air` over solid ground?**
//! (diagnosis probe for journal/0097's Observed entry, 2026-07-25)
//!
//! A walk reported that on an untouched column in the flag-OFF control,
//! `world_get_contents` read `dc:air` for voxels 288–299 while the player's
//! `eye_in_solid` said `true` at 296 and 291. This probe reconstructs, headless
//! and offline, **exactly what `dc:world/get_contents` would answer** for every
//! voxel of a column, from the same two sources the live query uses:
//!
//! - `block`  = `WorldGenerator::generate_chunk(pos)` — the block the
//!   `HostWorld` stores and `block_at` (hence `eye_in_solid`) reads.
//! - contents = `WorldGenerator::chunk_contents(pos)` — the `ContentsSource`
//!   the client installs (`dc-client/src/authority.rs:226`).
//! - `has_contents` = `contents.is_some()` (`dc-api/src/host.rs:83`), i.e.
//!   **whether the whole 32³ chunk had any recorded voxel**, and
//!   `classified` = `classify(contents)` (`dc-api/src/host.rs:76-79`).
//!
//! The claim under test: an **unrecorded basement** voxel (solid `Block::Stone`,
//! `MixtureId::EMPTY`) that happens to share a chunk with a recorded voxel is
//! reported as `has_contents: true`, `classified: "dc:air"` — while the very
//! same physical situation one chunk lower is reported honestly as
//! `has_contents: false`, `classified: "dc:stone"`. If so, the reported "air
//! band" is a **chunk-granular reporting artifact**, not a hole in the world.
//!
//! Nothing here is on a generation path: every readout is a pure derivation of
//! the same pregen the world boots from, at the client's `BENCH_SEED`.
//!
//! `cargo run --release -p dc-worldgen --example contents_air_over_solid_probe`

use std::collections::HashMap;
use std::sync::Arc;

use dc_core::{Block, Chunk, ChunkPos, ContentsGrid, VoxelContents, classify, local_voxel};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};
use dc_worldgen::{DeepOverrides, WorldGenerator};

/// The client's `BENCH_SEED` — the world every walk stands in.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const VOXEL_M: f64 = 0.9;

/// journal/0097 STATION 1 — deep cell (439, 276), world metres, surface 273.1 m.
/// The tour's top-ranked banded station and the walk's own vantage.
const STATION_M: (f64, f64) = (84_185.0, 9_212.0);

/// Extra columns, spread wide, for the "is it position-dependent?" census.
const CENSUS_STEP_VOXELS: i64 = 4096;
const CENSUS_SPAN: i64 = 6;

fn block_name(b: Block) -> String {
    match b {
        Block::Air => "dc:air".into(),
        Block::Stone => "dc:stone".into(),
        Block::Dirt => "dc:dirt".into(),
        Block::Grass => "dc:grass".into(),
        Block::Wood => "dc:wood".into(),
        Block::Material(m) => m.qualified_name().to_string(),
    }
}

/// What `dc:world/get_contents` would answer at one world voxel.
struct Answer {
    block: Block,
    has_contents: bool,
    contents: Option<VoxelContents>,
}

impl Answer {
    /// `dc-api/src/host.rs:76-79`: `classified` is `classify(contents)` when a
    /// record is present, else it echoes the stored block.
    fn classified(&self) -> Block {
        match &self.contents {
            Some(c) => classify(c),
            None => self.block,
        }
    }
    /// The defect shape: the world is solid, the query says the voxel
    /// classifies to air, and it claims a record backs that answer.
    fn is_phantom_air(&self) -> bool {
        self.block.is_solid() && self.has_contents && self.classified() == Block::Air
    }
    /// The honest shape: solid, no record, and the query says so.
    fn is_honest_no_record(&self) -> bool {
        self.block.is_solid() && !self.has_contents
    }
}

/// Per-chunk cache: generating a 32³ chunk (and resolving its contents grid)
/// per probed voxel would be ruinous, and both are pure functions of `pos`.
type ChunkCache = HashMap<ChunkPos, (Chunk, Option<ContentsGrid>)>;

fn answer_at(
    wg: &mut WorldGenerator<'static>,
    cache: &mut ChunkCache,
    vx: i64,
    vy: i64,
    vz: i64,
) -> Answer {
    let cp = ChunkPos::from_world_voxel(vx, vy, vz);
    let (lx, ly, lz) = local_voxel(vx, vy, vz);
    let entry = cache
        .entry(cp)
        .or_insert_with(|| (wg.generate_chunk(cp), wg.chunk_contents(cp)));
    let block = entry.0.get(lx, ly, lz);
    let contents = entry.1.as_ref().map(|g| g.get(lx, ly, lz));
    Answer {
        block,
        has_contents: contents.is_some(),
        contents,
    }
}

fn column_height(wg: &mut WorldGenerator<'static>, vx: i64, vz: i64) -> i64 {
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
    i64::from(wg.column_record(cx, cz).heights[lz * 32 + lx])
}

fn main() {
    println!("=== contents `dc:air` over SOLID ground — is it a record hole or a report? ===");
    println!("seed {SEED}, extent {}, voxel {VOXEL_M} m", EXTENT.label());
    println!("flag-OFF control (DeepOverrides::default(), i.e. --weather-inventory OFF)\n");

    let pregen = Arc::new(Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides::default(),
    ));
    let mut wg = WorldGenerator::new_owned(pregen);
    let mut cache: ChunkCache = HashMap::new();

    // ---------------------------------------------------------------- station
    let (vx, vz) = (
        (STATION_M.0 / VOXEL_M).round() as i64,
        (STATION_M.1 / VOXEL_M).round() as i64,
    );
    let h = column_height(&mut wg, vx, vz);
    println!(
        "--- STATION 1: metres ({:.0}, {:.0}) = voxel ({vx}, {vz}); surface voxel h = {h} ({:.1} m)",
        STATION_M.0,
        STATION_M.1,
        h as f64 * VOXEL_M
    );
    println!(
        "    chunk column x={} z={}; chunk-y boundaries every 32 voxels\n",
        vx.div_euclid(32),
        vz.div_euclid(32)
    );
    println!("      vy | chunk_y | block            | has_contents | classified        | contents");
    println!("    -----+---------+------------------+--------------+-------------------+---------");
    for vy in (h - 48..=h + 2).rev() {
        let a = answer_at(&mut wg, &mut cache, vx, vy, vz);
        let marker = if a.is_phantom_air() {
            "  <== PHANTOM AIR (solid, record claimed, classifies to air)"
        } else {
            ""
        };
        let boundary = if vy.rem_euclid(32) == 0 {
            " *chunk floor*"
        } else {
            ""
        };
        println!(
            "    {vy:>4} | {:>7} | {:<16} | {:<12} | {:<17} | {}{}{}",
            vy.div_euclid(32),
            block_name(a.block),
            a.has_contents,
            block_name(a.classified()),
            match &a.contents {
                Some(c) if c.is_empty() => "EMPTY".to_string(),
                Some(_) => "recorded".to_string(),
                None => "-".to_string(),
            },
            boundary,
            marker,
        );
    }

    // ---------------------------------------------------------------- census
    println!("\n--- CENSUS: is it position-dependent or everywhere the record ends?");
    println!("    for each column, every voxel from h down to h-64 is classified\n");
    println!("      column (voxel x,z) |    h | solid | phantom-air | honest-no-record | recorded");
    println!("    ---------------------+------+-------+-------------+------------------+---------");
    let (mut tot_solid, mut tot_phantom, mut tot_honest, mut tot_rec) = (0u64, 0u64, 0u64, 0u64);
    let mut columns_with_phantom = 0u32;
    let mut columns = 0u32;
    for i in -CENSUS_SPAN..=CENSUS_SPAN {
        for j in -CENSUS_SPAN..=CENSUS_SPAN {
            let (cvx, cvz) = (vx + i * CENSUS_STEP_VOXELS, vz + j * CENSUS_STEP_VOXELS);
            let h = column_height(&mut wg, cvx, cvz);
            let (mut solid, mut phantom, mut honest, mut rec) = (0u64, 0u64, 0u64, 0u64);
            for vy in (h - 64)..=h {
                let a = answer_at(&mut wg, &mut cache, cvx, vy, cvz);
                if !a.block.is_solid() {
                    continue;
                }
                solid += 1;
                if a.is_phantom_air() {
                    phantom += 1;
                } else if a.is_honest_no_record() {
                    honest += 1;
                } else {
                    rec += 1;
                }
            }
            columns += 1;
            if phantom > 0 {
                columns_with_phantom += 1;
            }
            tot_solid += solid;
            tot_phantom += phantom;
            tot_honest += honest;
            tot_rec += rec;
            if i.abs() <= 1 && j.abs() <= 1 {
                println!(
                    "      {cvx:>9},{cvz:<9} | {h:>4} | {solid:>5} | {phantom:>11} | {honest:>16} | {rec:>8}"
                );
            }
        }
    }
    println!(
        "\n    TOTAL over {columns} columns x 65 voxels: solid {tot_solid}, \
         PHANTOM-AIR {tot_phantom} ({:.1} %), honest-no-record {tot_honest} ({:.1} %), recorded {tot_rec} ({:.1} %)",
        100.0 * tot_phantom as f64 / tot_solid.max(1) as f64,
        100.0 * tot_honest as f64 / tot_solid.max(1) as f64,
        100.0 * tot_rec as f64 / tot_solid.max(1) as f64,
    );
    println!(
        "    columns showing at least one PHANTOM-AIR voxel: {columns_with_phantom} / {columns}"
    );
    println!(
        "\n    Reading: PHANTOM-AIR and honest-no-record are the SAME physical voxel\n\
         \x20   (unrecorded basement, Block::Stone, MixtureId::EMPTY). Which of the two\n\
         \x20   a walker is told depends only on whether the 32^3 chunk happens to\n\
         \x20   contain any recorded voxel (MaterialChunk::is_all_empty, intern.rs:313)."
    );
}
