//! Probe: what the biotic record's organic facies look like AFTER the collapse
//! tier's voxel quantization. Measurement tool for the organic-materials
//! milestone (journal/0026) — it answers "which of the four biotic signals can a
//! 0.9 m voxel column actually hold?", which decides which organic materials are
//! honest to ship.
//!
//! Run: `cargo run --release -p dc-worldgen --example organic_probe`

use dc_core::VoxelScale;
use dc_core::materials::geology;
use dc_worldgen::WorldGenerator;
use dc_worldgen::deeptime::{self, Biofacies, production_config};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

/// The three S10-measured sites (docs/spikes/S10-results.md § Read-quality).
const SITES: [(&str, i64, i64); 3] = [
    ("coal seam", 107_338, 58_787),
    ("paleosol cyclothem", 43_455, 105_805),
    ("charcoal fire record", -99_131, 13_303),
];

fn main() {
    let voxel_m = VoxelScale::from_player_height(1.8, 2).voxel_size_m();
    println!("voxel edge {voxel_m:.3} m; seed {SEED:#x}, Medium extent");
    let t0 = std::time::Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    println!("world-creation ritual: {:?}", t0.elapsed());

    for (why, vx, vz) in SITES {
        let Some(rec) = pregen.deep.record_at_voxel(vx, vz) else {
            println!("\n[{why}] ({vx},{vz}): no record");
            continue;
        };
        println!(
            "\n[{why}] world voxel ({vx},{vz}) — {} units, H {:.2} m",
            rec.units.len(),
            rec.total_m()
        );
        for u in rec.units.iter().rev() {
            let tv = (u.thickness_m() / voxel_m).round();
            println!(
                "   {:8.3} m  {:>4} vox  [{}] {}",
                u.thickness_m(),
                tv,
                u.tag().code(),
                if tv < 1.0 { "DROPPED (sub-voxel)" } else { "" }
            );
        }
    }

    // Whole-grid census: organic thickness recorded vs. thickness that survives
    // the `deposit_deep_history` sub-voxel drop.
    let mut rec_m = [0.0f64; 6];
    let mut kept_m = [0.0f64; 6];
    let mut units = [0usize; 6];
    let mut kept_units = [0usize; 6];
    let idx = |b: Biofacies| match b {
        Biofacies::Mineral => 0,
        Biofacies::Soil => 1,
        Biofacies::Peat => 2,
        Biofacies::Coal => 3,
        Biofacies::Charcoal => 4,
        Biofacies::Retro => 5,
    };
    for s in &pregen.deep.strata {
        for u in &s.units {
            let k = idx(u.tag().biota);
            rec_m[k] += u.thickness_m();
            units[k] += 1;
            if (u.thickness_m() / voxel_m).round() >= 1.0 {
                kept_m[k] += u.thickness_m();
                kept_units[k] += 1;
            }
        }
    }
    // The collapsed column at the coal site: what a player actually digs.
    let mut g = WorldGenerator::new(&pregen);
    let set = geology::vanilla();
    let (cx, cz) = (SITES[0].1.div_euclid(32), SITES[0].2.div_euclid(32));
    let (lx, lz) = (
        SITES[0].1.rem_euclid(32) as usize,
        SITES[0].2.rem_euclid(32) as usize,
    );
    let col = g.column_record(cx, cz);
    // P11 slice 3: the record is per column — read the site voxel's own SubCell.
    if let Some(sub) = col.record_for(lx, lz) {
        println!(
            "\ncollapsed chunk-column ({cx},{cz}) at the coal site — {} events, top first:",
            sub.strata().events.len()
        );
        for e in sub.strata().events.iter().rev().take(40) {
            let m = set.member(e.member);
            println!(
                "   {:>8.3} m  {:28} [{}]",
                e.thickness_m,
                m.id,
                m.class.trim_start_matches("dc:")
            );
        }
    } else {
        println!("\ncollapsed chunk-column ({cx},{cz}): no record at the site column");
    }

    // Ritual A/B: the deep-time sim with biology ON (production) vs OFF.
    for (label, biotic) in [("biology OFF", false), ("biology ON ", true)] {
        let mut cfg = production_config(&pregen.grid, SEED);
        cfg.biotic = biotic;
        let t = std::time::Instant::now();
        let run = deeptime::run_cells(&pregen.grid, &cfg, true);
        println!(
            "\ndeep-time ritual [{label}]: {:?}  ({} cells @ {:.0} m)",
            t.elapsed(),
            run.grid.w * run.grid.w,
            cfg.cell_m
        );
    }

    println!("\nfacies census over the whole deep grid (recorded → survives quantization):");
    for (k, name) in ["Mineral", "Soil", "Peat", "Coal", "Charcoal", "Retro"]
        .iter()
        .enumerate()
    {
        println!(
            "  {name:9}: {:>8} units ({:>8.0} m)  →  {:>8} units ({:>8.0} m) kept  [{:.1}% of units]",
            units[k],
            rec_m[k],
            kept_units[k],
            kept_m[k],
            100.0 * kept_units[k] as f64 / units[k].max(1) as f64
        );
    }
}
