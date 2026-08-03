//! **Distribution-first strata expression** (docs/design/materials.md DECIDED
//! 2026-07-21; journal/0055) — the falsifiers for the one property that makes
//! stochastic rounding legitimate, and for the determinism it must not buy it
//! with.
//!
//! The claim under test is *not* "the world looks better". It is:
//!
//! 1. **Mixed voxels exist.** Before this slice the generator could not produce
//!    one outside a placer fan: every contact landed on a voxel boundary by
//!    construction, because each recorded unit was rounded to whole voxels
//!    independently.
//! 2. **The rounding is unbiased.** Over a neighbourhood, the *expected*
//!    composition equals the recorded composition. This is the whole argument
//!    for stochastic over deterministic rounding: flooring is a biased estimator
//!    that always loses mass — it deletes the thin bed everywhere, forever.
//! 3. **It is addressed, not sequential.** A voxel's contents depend on the
//!    world position and nothing else: not on chunk generation order, not on the
//!    chunk's `y`, not on cache warmth. (Floyd–Steinberg error diffusion would
//!    give a lower-variance dither and is forbidden for exactly this reason.)

use std::collections::BTreeMap;

use dc_core::materials::geology::vanilla;
use dc_core::{ChunkPos, classify};
use dc_worldgen::fill::{allocate, fill_draw, share_eighths};
use dc_worldgen::{Extent, Plan, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn medium() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    })
}

/// Chunk-columns spread across the map, with their recorded strata.
fn sample_columns(n: i64) -> Vec<(i64, i64)> {
    (0..n).map(|k| (k * 29 - 150, (k % 9) * 31 - 70)).collect()
}

/// **The defect, gone.** Recorded columns now slice into voxel spans that
/// straddle contacts — and a lot of them do, because the recorder's beds are
/// centimetres thick while a voxel is 90 cm.
#[test]
fn the_record_produces_mixed_voxel_spans() {
    let pregen = medium();
    let mut g = WorldGenerator::with_geology(&pregen, vanilla());
    let (mut cols_with_record, mut spans, mut mixed) = (0usize, 0usize, 0usize);
    for (cx, cz) in sample_columns(24) {
        let col = g.column_record(cx, cz);
        // P11 slice 3: the record is per column; the chunk-centre column's
        // SubCell is the census representative (I-5 judgment site).
        let Some(sub) = col.centre_record() else {
            continue;
        };
        if sub.strata().events.is_empty() {
            continue;
        }
        cols_with_record += 1;
        let fill = sub.fill();
        for d in 1..=fill.depth_count() as u32 {
            spans += 1;
            if matches!(fill.plan(d), Some(Plan::Mixed(_))) {
                mixed += 1;
            }
        }
    }
    println!(
        "{cols_with_record} recorded columns, {spans} voxel spans, {mixed} mixed \
         ({:.1}%)",
        100.0 * mixed as f64 / spans as f64
    );
    assert!(cols_with_record >= 10, "sample found no recorded columns");
    assert!(
        mixed > 100,
        "only {mixed} mixed spans — the sieve is still rounding each unit \
         independently"
    );
}

/// **Unbiasedness over a neighbourhood.** For a real mixed span in a real
/// column, average the allocated eighths over a 64×64 patch of world positions
/// and compare with the recorded share. Tolerance is a couple of hundredths of
/// an eighth — comfortably inside the sampling error of 4 096 draws, and far
/// tighter than the bias deterministic flooring would show (a material with a
/// 0.6-eighth share would come back 0.0, not 0.6).
#[test]
fn expected_composition_matches_the_recorded_composition() {
    let pregen = medium();
    let mut g = WorldGenerator::with_geology(&pregen, vanilla());
    let mut checked = 0usize;
    let mut worst = 0.0f64;
    for (cx, cz) in sample_columns(24) {
        let col = g.column_record(cx, cz);
        // P11 slice 3: per-column records; the chunk-centre column stands in
        // (I-5 judgment site — this test only needs A real mixed span).
        let Some(sub) = col.centre_record() else {
            continue;
        };
        if sub.strata().events.is_empty() {
            continue;
        }
        let fill = sub.fill();
        // The richest mixed span in this column — the hardest case, where the
        // most materials compete for eight eighths.
        let Some(weights) = (1..=fill.depth_count() as u32)
            .filter_map(|d| match fill.plan(d) {
                Some(Plan::Mixed(w)) => Some(w.clone()),
                _ => None,
            })
            .max_by_key(Vec::len)
        else {
            continue;
        };
        if weights.len() < 2 {
            continue;
        }
        let mut got: BTreeMap<usize, u64> = BTreeMap::new();
        let n = 64i64;
        for dx in 0..n {
            for dz in 0..n {
                let (vx, vz) = (cx * 32 + dx, cz * 32 + dz);
                let u = fill_draw(SEED, vx, 40, vz);
                let alloc = allocate(&weights, u);
                assert_eq!(
                    alloc.iter().map(|(_, k)| u32::from(*k)).sum::<u32>(),
                    8,
                    "a voxel must always be filled to exactly eight eighths"
                );
                for (i, k) in alloc {
                    *got.entry(i).or_default() += u64::from(k);
                }
            }
        }
        let samples = (n * n) as f64;
        for &(idx, w) in &weights {
            let want = share_eighths(w);
            let mean = got.get(&idx).copied().unwrap_or(0) as f64 / samples;
            worst = worst.max((mean - want).abs());
            assert!(
                (mean - want).abs() < 0.06,
                "column ({cx},{cz}) event {idx}: mean {mean:.4} eighths vs \
                 recorded share {want:.4} — the estimator is biased"
            );
            // The point of the whole exercise: a sub-eighth material is not
            // deleted, it is expressed *somewhere*.
            if want > 0.15 {
                assert!(
                    got.get(&idx).copied().unwrap_or(0) > 0,
                    "column ({cx},{cz}) event {idx} holds {want:.3} eighths of \
                     recorded share and was never expressed in {samples} voxels"
                );
            }
        }
        checked += 1;
    }
    println!(
        "unbiasedness checked on {checked} columns; worst |mean − share| = {worst:.4} eighths"
    );
    assert!(checked >= 8, "only {checked} columns carried a mixed span");
}

/// **Addressed, not sequential.** The same world voxel gets the same contents
/// whichever chunk asked for it first, from whichever chunk `y`, with whatever
/// cache warmth — the property Floyd–Steinberg error diffusion would destroy.
#[test]
fn fill_does_not_depend_on_generation_order() {
    let pregen = medium();
    let mut fwd = WorldGenerator::with_geology(&pregen, vanilla());
    let mut rev = WorldGenerator::with_geology(&pregen, vanilla());

    let cols: Vec<(i64, i64)> = (0..6i64).map(|k| (k * 19 - 40, k * 7 - 20)).collect();
    let mut positions = Vec::new();
    for &(cx, cz) in &cols {
        let cy = fwd.surface_chunk_y(cx, cz);
        positions.push(ChunkPos::new(cx as i32, cy - 1, cz as i32));
        positions.push(ChunkPos::new(cx as i32, cy - 2, cz as i32));
    }
    // Forward order, then the same chunks in reverse with a cold generator that
    // has additionally been warmed somewhere else entirely.
    for k in 0..20i64 {
        let _ = rev.surface_chunk_y(k * 41 + 900, k * 13 - 700);
    }
    let a: Vec<_> = positions
        .iter()
        .map(|p| (*p, fwd.generate_chunk(*p), fwd.chunk_contents(*p)))
        .collect();
    let mut b: Vec<_> = positions
        .iter()
        .rev()
        .map(|p| (*p, rev.generate_chunk(*p), rev.chunk_contents(*p)))
        .collect();
    b.reverse();

    let mut compared = 0usize;
    let mut mixed_seen = 0usize;
    for ((pa, ca, ga), (pb, cb, gb)) in a.iter().zip(&b) {
        assert_eq!(pa, pb);
        assert_eq!(ca.blocks(), cb.blocks(), "blocks moved with chunk order");
        assert_eq!(ga.is_some(), gb.is_some());
        let (Some(ga), Some(gb)) = (ga, gb) else {
            continue;
        };
        for z in 0..32usize {
            for y in 0..32usize {
                for x in 0..32usize {
                    let (va, vb) = (ga.get(x, y, z), gb.get(x, y, z));
                    assert_eq!(va, vb, "contents moved with chunk order at {pa:?}");
                    if !va.is_empty() {
                        // The fill contract, on the very voxels this slice
                        // creates: one opinion per voxel.
                        assert_eq!(ca.get(x, y, z), classify(&va));
                        if va.filled_slots().windows(2).any(|w| w[0] != w[1]) {
                            mixed_seen += 1;
                        }
                    }
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 100_000, "compared only {compared} voxels");
    assert!(
        mixed_seen > 0,
        "no heterogeneous voxel in the sample — the dither is not reaching the \
         material tier"
    );
    println!("{compared} voxels compared, {mixed_seen} of them heterogeneous");
}
