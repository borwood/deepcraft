//! 3e-1 proofs: the deep-time A tier integrated into real world generation.
//!
//! - **Same-seed byte identity** of the integrated deep field (surface planes
//!   AND strata records), the S9 determinism falsifier lifted to the always-on
//!   pregen pass.
//! - **Read-quality**: the deep record still tells true stories after
//!   integration — a multi-unit tail, tag variety, and marine bands within sane
//!   bands (S9's metrics, ported against the integrated path).
//! - **Column stories survive to blocks**: a collapse column over a deep cell
//!   with a layered record renders a multi-band cliff (the recorded sequence,
//!   now walkable).

use dc_core::{Block, ChunkPos};
use dc_worldgen::deeptime::DepEnv;
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn medium() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    })
}

#[test]
fn integrated_deep_field_is_byte_identical_same_seed() {
    let a = medium();
    let b = medium();
    assert_eq!(
        a.deep.surf, b.deep.surf,
        "deep-time surface must regenerate byte-identically"
    );
    assert_eq!(
        a.deep.strata, b.deep.strata,
        "deep-time strata records must regenerate byte-identically"
    );
    // A different seed erodes a different world.
    let c = Pregen::run(WorldParams {
        seed: SEED ^ 0xABCD,
        extent: Extent::Medium,
    });
    assert_ne!(a.deep.surf, c.deep.surf, "different seeds must diverge");
}

#[test]
fn integrated_deep_record_tells_true_stories() {
    // Port S9's read-quality metrics against the integrated deep field.
    let p = medium();
    let mut record_bearing = 0usize;
    let mut total_units = 0usize;
    let mut multi_unit = 0usize; // >= 2 units
    let mut with_marine = 0usize; // carries a subsea band
    let mut with_subaerial = 0usize;
    let mut transgressive = 0usize; // both marine and subaerial (a couplet)
    let mut tag_varied = 0usize; // >= 2 distinct tags
    let mut unconformities = 0usize;
    for s in &p.deep.strata {
        if s.units.is_empty() {
            continue;
        }
        record_bearing += 1;
        total_units += s.units.len();
        if s.units.len() >= 2 {
            multi_unit += 1;
        }
        let marine = s.units.iter().any(|u| u.tag.env == DepEnv::Subsea);
        let subaerial = s.units.iter().any(|u| u.tag.env == DepEnv::Subaerial);
        with_marine += usize::from(marine);
        with_subaerial += usize::from(subaerial);
        transgressive += usize::from(marine && subaerial);
        if s.tag_variety() >= 2 {
            tag_varied += 1;
        }
        unconformities += s.unconformities();
    }
    let mean_units = total_units as f64 / record_bearing.max(1) as f64;
    let marine_frac = with_marine as f64 / record_bearing.max(1) as f64;
    eprintln!(
        "deep read-quality: record-bearing {record_bearing}, mean units {mean_units:.2}, \
         multi-unit {multi_unit}, marine {with_marine} ({marine_frac:.3}), \
         subaerial {with_subaerial}, transgressive {transgressive}, tag-varied {tag_varied}, \
         unconformities {unconformities}"
    );
    // The integrated field is alive: a real population of records…
    assert!(
        record_bearing > 1000,
        "too few record-bearing deep cells: {record_bearing}"
    );
    // …a fat multi-unit tail (accommodation, not uniform blanketing)…
    assert!(mean_units > 1.0, "mean units {mean_units} not > 1");
    assert!(
        multi_unit > 100,
        "too little multi-unit history: {multi_unit}"
    );
    // …both marine and subaerial facies present, and true couplets…
    assert!(
        (0.02..0.95).contains(&marine_frac),
        "marine fraction {marine_frac} outside sane band"
    );
    assert!(with_subaerial > 100, "no subaerial fans: {with_subaerial}");
    assert!(
        transgressive > 0,
        "no transgressive-regressive couplets recorded"
    );
    assert!(tag_varied > 100, "records lack tag variety: {tag_varied}");
    // …and readable erosional time-gaps.
    assert!(unconformities > 0, "no proto-unconformities recorded");
}

#[test]
fn collapse_column_story_comes_from_the_deep_record() {
    // A land column over a layered deep cell must render a multi-band cliff: the
    // recorded sequence (igneous basement / recorded sediments / active veneer)
    // as distinct strata blocks a walker can read.
    let p = medium();
    let mut g = WorldGenerator::new(&p);

    // Find the land column, over the sampled grid, with the richest strata.
    let mut best: Option<(i64, i64, usize)> = None;
    for cz in -40i64..40 {
        for cx in -40i64..40 {
            let (ccx, ccz) = (cx * 2, cz * 2);
            let col = g.column_record(ccx, ccz);
            if col.wilds || col.heights.iter().sum::<i32>() <= 0 {
                continue;
            }
            let n = col.strata.events.len();
            if best.is_none_or(|(_, _, m)| n > m) {
                best = Some((ccx, ccz, n));
            }
        }
    }
    let (cx, cz, n_events) = best.expect("sample found a land column");
    assert!(
        n_events >= 3,
        "richest column has only {n_events} strata events — the deep history is not reaching blocks"
    );

    // Render the surface chunk and count distinct strata block kinds stacked in
    // one voxel column (the readable cliff).
    let cy = g.surface_chunk_y(cx, cz);
    let chunk = g.generate_chunk(ChunkPos::new(cx as i32, cy, cz as i32));
    let mut best_distinct = 0usize;
    for z in 0..32usize {
        for x in 0..32usize {
            let mut kinds: Vec<Block> = Vec::new();
            for y in 0..32usize {
                let b = chunk.get(x, y, z);
                if matches!(
                    b,
                    Block::Mudstone | Block::Sandstone | Block::Granite | Block::Basalt
                ) && !kinds.contains(&b)
                {
                    kinds.push(b);
                }
            }
            best_distinct = best_distinct.max(kinds.len());
        }
    }
    assert!(
        best_distinct >= 2,
        "the richest column's cliff shows only {best_distinct} strata block kind(s) — \
         no readable multi-unit sequence"
    );
}
