//! S7 pregen pipeline properties: topology, tectonics, climate, hydrology
//! (rivers provably reach the sea over the whole map), history causal
//! ordering, and pregen determinism.

use dc_sim::statistical::{Aspect, Subject, Value};
use dc_worldgen::pregen::history::NUM_EPOCHS;
use dc_worldgen::{Extent, Pregen, Provenance, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn pregen(extent: Extent) -> Pregen {
    Pregen::run(WorldParams { seed: SEED, extent })
}

#[test]
fn topology_world_ocean_ring_closes_the_disc() {
    for extent in [Extent::Small, Extent::Medium, Extent::Large] {
        let p = pregen(extent);
        let w = p.grid.w;
        // Every border cell is ocean: the disc sits in a world-ocean, so the
        // border wilds continue seamlessly in every horizontal direction.
        for gy in 0..w {
            for gx in 0..w {
                if gx == 0 || gy == 0 || gx == w - 1 || gy == w - 1 {
                    let c = p.grid.get(gx, gy).unwrap();
                    assert!(
                        !c.is_land(),
                        "border cell ({gx},{gy}) must be ocean at {extent:?}, got {} m",
                        c.elev_m
                    );
                }
            }
        }
        // And there is land to live on (the point of the disc).
        let land = p.grid.cells.iter().filter(|c| c.is_land()).count();
        let frac = land as f64 / p.grid.cells.len() as f64;
        assert!(
            (0.02..0.85).contains(&frac),
            "{extent:?}: implausible land fraction {frac}"
        );
    }
}

#[test]
fn tectonics_produces_provenance_diversity() {
    let p = pregen(Extent::Medium);
    let has = |prov: Provenance| p.grid.cells.iter().any(|c| c.provenance == prov);
    assert!(
        has(Provenance::Orogeny),
        "no mountain belt on a medium world"
    );
    assert!(has(Provenance::Shelf), "no continental shelf");
    assert!(has(Provenance::OceanFloor), "no ocean floor");
    // Mountains are actually high, trenches actually deep.
    let max = p
        .grid
        .cells
        .iter()
        .map(|c| c.elev_m)
        .fold(f64::MIN, f64::max);
    let min = p
        .grid
        .cells
        .iter()
        .map(|c| c.elev_m)
        .fold(f64::MAX, f64::min);
    assert!(max > 800.0, "highest cell only {max} m");
    assert!(min < -200.0, "deepest cell only {min} m");
}

#[test]
fn climate_latitude_gradient_and_rain_shadow() {
    let p = pregen(Extent::Medium);
    let w = p.grid.w;
    // North ocean rows are colder than south ocean rows.
    let row_temp = |gy: i32| {
        let cells: Vec<f64> = (0..w)
            .filter_map(|gx| p.grid.get(gx, gy))
            .filter(|c| !c.is_land())
            .map(|c| c.temp_c)
            .collect();
        cells.iter().sum::<f64>() / cells.len() as f64
    };
    assert!(
        row_temp(1) > row_temp(w - 2) + 10.0,
        "no latitude gradient: south {} vs north {}",
        row_temp(1),
        row_temp(w - 2)
    );
    // Rain shadow: land cells directly downwind of high terrain are drier on
    // average than land cells directly upwind (aggregated over the map).
    let mut upwind = Vec::new();
    let mut downwind = Vec::new();
    for gy in 0..w {
        for gx in 0..w {
            let c = p.grid.get(gx, gy).unwrap();
            if c.elev_m < 900.0 {
                continue;
            }
            let dx = dc_worldgen::pregen::climate::wind_dx(c.lat_deg);
            if let Some(u) = p.grid.get(gx - dx, gy).filter(|u| u.is_land()) {
                upwind.push(u.precip);
            }
            if let Some(d) = p.grid.get(gx + dx, gy).filter(|d| d.is_land()) {
                downwind.push(d.precip);
            }
        }
    }
    assert!(
        !upwind.is_empty() && !downwind.is_empty(),
        "no high terrain with land on both sides to measure"
    );
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    assert!(
        mean(&downwind) < mean(&upwind),
        "no rain shadow: downwind {} >= upwind {}",
        mean(&downwind),
        mean(&upwind)
    );
}

#[test]
fn every_river_reaches_the_sea_by_construction() {
    for extent in [Extent::Small, Extent::Medium, Extent::Large] {
        let p = pregen(extent);
        let n = p.grid.cells.len();
        let mut rivers = 0usize;
        for start in 0..n {
            if !p.grid.cells[start].is_land() {
                continue;
            }
            if p.grid.cells[start].river {
                rivers += 1;
            }
            // Follow flow pointers: must reach an ocean cell within n steps
            // (which also proves acyclicity), descending in filled elevation.
            let mut cur = start;
            let mut steps = 0usize;
            loop {
                match p.grid.cells[cur].flow_to {
                    None => {
                        assert!(
                            !p.grid.cells[cur].is_land(),
                            "{extent:?}: land cell {cur} is a dead end (no terminal basin was planned)"
                        );
                        break;
                    }
                    Some(next) => {
                        let next = next as usize;
                        assert!(
                            p.grid.cells[next].filled_m < p.grid.cells[cur].filled_m,
                            "{extent:?}: flow must strictly descend"
                        );
                        cur = next;
                        steps += 1;
                        assert!(steps <= n, "{extent:?}: flow cycle from cell {start}");
                    }
                }
            }
        }
        if extent != Extent::Small {
            assert!(rivers > 0, "{extent:?}: no rivers at all");
        }
    }
}

#[test]
fn history_facts_are_causally_ordered() {
    let p = pregen(Extent::Medium);
    assert!(!p.sites.is_empty(), "medium world produced no history");
    assert!(p.n_polities >= 2, "need at least two polities for conflict");
    for s in &p.sites {
        // Abandonment strictly postdates founding.
        if let Some(ab) = s.abandoned {
            assert!(
                ab > s.founded,
                "site {} abandoned at {} but founded at {}",
                s.slot,
                ab,
                s.founded
            );
        }
        // The ledger agrees with the summary.
        let subj = Subject::Site(s.slot);
        let founded = p.ledger.get(subj, s.founded, Aspect::SiteExists);
        assert_eq!(founded.map(|f| f.value), Some(Value::Exists(true)));
        if let Some(ab) = s.abandoned {
            let gone = p.ledger.get(subj, ab, Aspect::SiteExists);
            assert_eq!(gone.map(|f| f.value), Some(Value::Exists(false)));
        }
    }
    // Polity extents were chronicled.
    assert!(
        p.ledger
            .facts()
            .iter()
            .any(|f| f.aspect == Aspect::PolityExtent),
        "no polity extent facts"
    );
    // The history pass really ran collapses through the S2 engine.
    assert!(p.observe_count > 0, "history never used engine::observe");
    let pressure_facts = p
        .ledger
        .facts()
        .iter()
        .filter(|f| f.aspect == Aspect::RegionPressure)
        .count();
    assert!(pressure_facts > 0, "no committed pressure facts");
    // All fact times lie in the pregen era.
    for f in p.ledger.facts() {
        assert!(f.time <= NUM_EPOCHS, "fact beyond year zero: {f:?}");
    }
}

#[test]
fn pregen_is_deterministic_and_seed_sensitive() {
    let a = pregen(Extent::Medium);
    let b = pregen(Extent::Medium);
    assert_eq!(a.grid, b.grid, "grid must replay identically");
    assert_eq!(a.sites, b.sites, "history must replay identically");
    assert_eq!(
        a.ledger.content_hash(),
        b.ledger.content_hash(),
        "ledger must replay identically"
    );
    let c = Pregen::run(WorldParams {
        seed: SEED + 1,
        extent: Extent::Medium,
    });
    assert!(
        a.grid != c.grid || a.ledger.content_hash() != c.ledger.content_hash(),
        "different seeds must differ"
    );
}
