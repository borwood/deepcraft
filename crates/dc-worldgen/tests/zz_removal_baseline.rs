//! TEMPORARY (journal/0121): the before-measurement for the bootstrap-history
//! removal. Deleted in the same slice; never merged.

use dc_core::{Block, ChunkPos};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

#[test]
fn baseline_residency_and_wood() {
    println!("size_of::<SiteSummary>() = {}", std::mem::size_of::<dc_worldgen::SiteSummary>());
    println!("size_of::<Fact>() = {}", std::mem::size_of::<dc_sim::statistical::Fact>());
    println!("size_of::<Cell>() = {}", std::mem::size_of::<dc_worldgen::Cell>());
    for extent in [Extent::Small, Extent::Medium, Extent::Large] {
        let p = Pregen::run(WorldParams { seed: SEED, extent });
        let facts = p.ledger.len() * std::mem::size_of::<dc_sim::statistical::Fact>();
        let sites = p.sites.len() * std::mem::size_of::<dc_worldgen::SiteSummary>();
        println!(
            "EXTENT {:?}: cells={} facts={} sites={} polities={} observes={} \
             approx_resident_bytes={} of which facts={} sites={} (history total={})",
            extent,
            p.grid.cells.len(),
            p.ledger.len(),
            p.sites.len(),
            p.n_polities,
            p.observe_count,
            p.approx_resident_bytes(),
            facts,
            sites,
            facts + sites,
        );
    }
}

#[test]
fn baseline_wood_census() {
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let abandoned: Vec<_> = p.sites.iter().filter(|s| s.abandoned.is_some()).collect();
    println!(
        "MEDIUM sites={} abandoned={}",
        p.sites.len(),
        abandoned.len()
    );
    let mut g = WorldGenerator::new(&p);
    let mut total = 0usize;
    let mut hot: Vec<(i32, i32, i32, usize)> = Vec::new();
    for s in &abandoned {
        let (sx, sz) = s.pos;
        let (ccx, ccz) = (sx.div_euclid(32), sz.div_euclid(32));
        for dz in -1..=1i64 {
            for dx in -1..=1i64 {
                let (cx, cz) = (ccx + dx, ccz + dz);
                let cy = g.surface_chunk_y(cx, cz);
                for y in [cy, cy + 1] {
                    let chunk = g.generate_chunk(ChunkPos::new(cx as i32, y, cz as i32));
                    let w = chunk.blocks().iter().filter(|b| **b == Block::Wood).count();
                    total += w;
                    if w > 0 {
                        hot.push((cx as i32, y, cz as i32, w));
                    }
                }
            }
        }
    }
    println!("WOOD TOTAL over {} abandoned sites = {total}", abandoned.len());
    println!("WOOD-BEARING CHUNKS ({}):", hot.len());
    for (cx, cy, cz, w) in &hot {
        println!("    (({cx}, {cy}, {cz}), {w}),");
    }
}

#[test]
fn baseline_wood_in_the_contract_sample() {
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let mut g = WorldGenerator::new(&p);
    let mut total = 0usize;
    for k in 0..40i64 {
        let (cx, cz) = (k * 13 - 200, (k % 7) * 17 - 60);
        let cy = g.surface_chunk_y(cx, cz);
        for y in [cy, cy - 1] {
            let chunk = g.generate_chunk(ChunkPos::new(cx as i32, y, cz as i32));
            total += chunk.blocks().iter().filter(|b| **b == Block::Wood).count();
        }
    }
    println!("WOOD in the contents_contract sample set = {total}");
}
