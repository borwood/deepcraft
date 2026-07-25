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

/// The census totals, as a value — so `main` can print them and the gate test
/// below can assert on them without re-deriving anything (journal/0103:
/// `cargo test` builds examples but never runs them, so a probe's claim only
/// reaches the gate through a `#[test]` that shares the probe's code).
struct ColumnRow {
    vx: i64,
    vz: i64,
    h: i64,
    solid: u64,
    phantom: u64,
    honest: u64,
    recorded: u64,
}

#[derive(Default)]
struct Census {
    per_column: Vec<ColumnRow>,
    columns: u32,
    columns_with_phantom: u32,
    solid: u64,
    /// Solid, a record is claimed for it, and it classifies to **air**.
    phantom: u64,
    /// Solid and honestly reported as carrying no record.
    honest: u64,
    /// Solid and carrying a real, non-empty mixture.
    recorded: u64,
    /// **The generator-level invariant.** A solid voxel whose contents record is
    /// non-empty and yet classifies to air would be a genuine hole in the world —
    /// the thing journal/0097's walk feared. Every `phantom` voxel above is an
    /// *empty* record (unrecorded basement reported at chunk granularity); this
    /// counter is the different, load-bearing question, and it must stay zero.
    solid_with_nonempty_record_classifying_air: u64,
}

/// Census a lattice of columns: for each, every solid voxel from `h` down to
/// `h - depth` is classified the way `dc:world/get_contents` would have.
fn census(
    wg: &mut WorldGenerator<'static>,
    cache: &mut ChunkCache,
    origin: (i64, i64),
    step: i64,
    span: i64,
    depth: i64,
) -> Census {
    let mut c = Census::default();
    for i in -span..=span {
        for j in -span..=span {
            let (vx, vz) = (origin.0 + i * step, origin.1 + j * step);
            let h = column_height(wg, vx, vz);
            let mut row = ColumnRow {
                vx,
                vz,
                h,
                solid: 0,
                phantom: 0,
                honest: 0,
                recorded: 0,
            };
            for vy in (h - depth)..=h {
                let a = answer_at(wg, cache, vx, vy, vz);
                if !a.block.is_solid() {
                    continue;
                }
                row.solid += 1;
                if a.contents.as_ref().is_some_and(|x| !x.is_empty())
                    && a.classified() == Block::Air
                {
                    c.solid_with_nonempty_record_classifying_air += 1;
                }
                if a.is_phantom_air() {
                    row.phantom += 1;
                } else if a.is_honest_no_record() {
                    row.honest += 1;
                } else {
                    row.recorded += 1;
                }
            }
            c.columns += 1;
            if row.phantom > 0 {
                c.columns_with_phantom += 1;
            }
            c.solid += row.solid;
            c.phantom += row.phantom;
            c.honest += row.honest;
            c.recorded += row.recorded;
            c.per_column.push(row);
        }
    }
    c
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
    let c = census(
        &mut wg,
        &mut cache,
        (vx, vz),
        CENSUS_STEP_VOXELS,
        CENSUS_SPAN,
        64,
    );
    for row in c.per_column.iter().filter(|r| {
        (r.vx - vx).abs() <= CENSUS_STEP_VOXELS && (r.vz - vz).abs() <= CENSUS_STEP_VOXELS
    }) {
        println!(
            "      {:>9},{:<9} | {:>4} | {:>5} | {:>11} | {:>16} | {:>8}",
            row.vx, row.vz, row.h, row.solid, row.phantom, row.honest, row.recorded
        );
    }
    println!(
        "\n    TOTAL over {} columns x 65 voxels: solid {}, \
         PHANTOM-AIR {} ({:.1} %), honest-no-record {} ({:.1} %), recorded {} ({:.1} %)",
        c.columns,
        c.solid,
        c.phantom,
        100.0 * c.phantom as f64 / c.solid.max(1) as f64,
        c.honest,
        100.0 * c.honest as f64 / c.solid.max(1) as f64,
        c.recorded,
        100.0 * c.recorded as f64 / c.solid.max(1) as f64,
    );
    println!(
        "    columns showing at least one PHANTOM-AIR voxel: {} / {}",
        c.columns_with_phantom, c.columns
    );
    println!(
        "    solid voxels with a NON-EMPTY record that classify to AIR (a real hole): {}",
        c.solid_with_nonempty_record_classifying_air
    );
    println!(
        "\n    Reading: PHANTOM-AIR and honest-no-record are the SAME physical voxel\n\
         \x20   (unrecorded basement, Block::Stone, MixtureId::EMPTY). Which of the two\n\
         \x20   a walker is told depends only on whether the 32^3 chunk happens to\n\
         \x20   contain any recorded voxel (MaterialChunk::is_all_empty, intern.rs:313)."
    );
}

/// **The gate's view of this instrument** (journal/0103).
///
/// This probe's *original* subject — the chunk-granular `has_contents` — was
/// fixed at the dc-api layer (journal/0101, corrections #49), and it is
/// `dc-client/examples/identify_census.rs` that guards that fix through the real
/// query path. What survives here is the **generator-level** claim underneath the
/// whole diagnosis, and it is the one that would be a genuine world defect if it
/// broke:
///
/// > *A solid voxel never carries a non-empty contents record that classifies to
/// > **air**.*
///
/// Unrecorded basement is `Block::Stone` with an EMPTY record — honest, and what
/// the fixed query reports as `UNRECORDED`. A **non-empty** record classifying to
/// air would be a real hole under solid ground: `ColumnFill::build`'s
/// `if cov <= 0.0 { break }` exists precisely to prevent it, and that guard has
/// never had a test.
///
/// Run at [`Extent::Small`]. The invariant is per-voxel and holds pointwise, so
/// grid width buys only more samples; the *production* census numbers stay in the
/// example at [`Extent::Medium`].
#[cfg(test)]
mod gate {
    use super::*;

    use std::sync::OnceLock;

    /// **Built once for the whole binary** — both tests share one small world and
    /// one census; sizing the gate is part of the conversion (CLAUDE.md § Gates).
    fn small_census() -> &'static Census {
        static CENSUS: OnceLock<Census> = OnceLock::new();
        CENSUS.get_or_init(|| {
            let pregen = Arc::new(Pregen::run_with(
                WorldParams {
                    seed: SEED,
                    extent: Extent::Small,
                },
                &DeepOverrides::default(),
            ));
            let mut wg = WorldGenerator::new_owned(pregen);
            let mut cache: ChunkCache = HashMap::new();
            // A lattice wide enough to cross provinces, inside the small world.
            census(&mut wg, &mut cache, (0, 0), 512, 5, 64)
        })
    }

    #[test]
    fn no_solid_voxel_carries_a_nonempty_record_that_classifies_to_air() {
        let c = small_census();
        assert!(
            c.solid > 0,
            "the census found no solid voxel at all over {} columns — the probe is \
             looking at nothing and its null proves nothing",
            c.columns
        );
        assert_eq!(
            c.solid_with_nonempty_record_classifying_air, 0,
            "{} solid voxels carry a NON-EMPTY contents record that classifies to AIR — \
             a real hole under solid ground, not the chunk-granularity report artifact \
             journal/0097 diagnosed (solid {}, recorded {})",
            c.solid_with_nonempty_record_classifying_air, c.solid, c.recorded,
        );
    }

    /// The other half of journal/0097's falsified premise (corrections #49): the
    /// band the walk read as air is **solid stone with an empty record**, not a
    /// gap. Stated as an identity so it cannot drift: every solid voxel is
    /// exactly one of phantom / honest-no-record / recorded.
    #[test]
    fn every_solid_voxel_is_accounted_for_exactly_once() {
        let c = small_census();
        assert_eq!(
            c.phantom + c.honest + c.recorded,
            c.solid,
            "the three buckets do not partition the solid voxels"
        );
        assert!(
            c.recorded > 0,
            "not one solid voxel carries a real mixture — the contents path produced \
             nothing, which no amount of 'no phantom air' makes acceptable"
        );
    }
}
