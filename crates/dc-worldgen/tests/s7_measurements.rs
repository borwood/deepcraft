//! S7 measurements: pregen wall time / memory vs extent (the honest labels for
//! the world-size knob), and chunk-generation latency through the full pyramid,
//! cold vs warm parent caches.
//!
//! **Four columns left the table 2026-07-28** (journal/0121) — facts, sites,
//! polities, observes — with the bootstrap settlement-history pass that produced
//! them. On production-Medium they were 153 facts / 13 sites / 2 polities / 90
//! collapses, contributing **5,520 bytes of 377,364,589** to the memory column.
//!
//! Run with `-- --nocapture` to see the tables.

use std::time::Instant;

use dc_core::ChunkPos;
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

#[test]
fn pregen_time_vs_extent() {
    println!("| extent | cells | wall ms | approx bytes |");
    println!("|---|---|---|---|");
    for extent in [Extent::Small, Extent::Medium, Extent::Large] {
        let t0 = Instant::now();
        let p = Pregen::run(WorldParams { seed: SEED, extent });
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "| {} | {}x{} | {ms:.1} | {} |",
            extent.label(),
            p.grid.w,
            p.grid.w,
            p.approx_resident_bytes(),
        );
        // Sanity: the pause must stay a ritual, not a wait.
        assert!(ms < 60_000.0, "{extent:?} pregen took {ms} ms");
    }
}

#[test]
fn chunk_latency_cold_vs_warm() {
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&p);

    // Cold: chunks >1 coarse cell apart (fresh region/locale/column caches
    // and mostly-fresh lattice ancestry each time).
    let mut cold = Vec::new();
    let mut warm = Vec::new();
    for k in 0..40i64 {
        let cx = -12_000 + k * 1200;
        let cz = if k % 2 == 0 { 700 } else { -900 };
        let cy = g.surface_chunk_y(cx, cz); // primes the column: excluded
        // ...so re-measure a *different* cold column right next to the walk
        // axis: use (cx, cz+64), a different locale and column.
        let t0 = Instant::now();
        let _ = g.generate_chunk(ChunkPos::new(cx as i32, cy, (cz + 64) as i32));
        cold.push(t0.elapsed().as_secs_f64() * 1000.0);
        // Warm: the adjacent chunk shares locale/region/lattice ancestry.
        let t0 = Instant::now();
        let _ = g.generate_chunk(ChunkPos::new((cx + 1) as i32, cy, (cz + 64) as i32));
        warm.push(t0.elapsed().as_secs_f64() * 1000.0);
        // Warmest: the same column, different y slab (column cache hit).
        let t0 = Instant::now();
        let _ = g.generate_chunk(ChunkPos::new((cx + 1) as i32, cy - 1, (cz + 64) as i32));
        warm.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    let stats = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        let p95 = v[(v.len() as f64 * 0.95) as usize - 1];
        let max = *v.last().unwrap();
        (mean, p95, max)
    };
    let (cm, cp, cx) = stats(&mut cold);
    let (wm, wp, wx) = stats(&mut warm);
    println!("| path | mean ms | p95 ms | max ms |");
    println!("|---|---|---|---|");
    println!("| cold (new region/locale/column) | {cm:.3} | {cp:.3} | {cx:.3} |");
    println!("| warm (adjacent chunk / cached column) | {wm:.3} | {wp:.3} | {wx:.3} |");
    assert!(cm < 250.0, "cold chunk generation too slow: {cm} ms");
}
