//! S7 thesis proof: the year-zero ledger handoff.
//!
//! Pregen history commits facts (through the S2 engine) at ticks `0..=12`;
//! live play is any tick beyond. This test shows there is **one system and no
//! seam**: a fact committed by worldgen in epoch 7 is (a) reproduced with
//! probability 1 by a live query at the fact's time, (b) *conditions* live
//! queries after year zero, and (c) coexists with new live observations
//! appended to the very same ledger — while the abandoned site it describes
//! is readable both from the ledger and from the terrain (ruins).

use dc_core::{Block, ChunkPos};
use dc_sim::statistical::world::RegionId;
use dc_sim::statistical::{Aspect, Ledger, Params, Subject, Value, observe, query};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

const LIVE_PARAMS: Params = Params {
    k_samples: 256,
    depth: Some(2),
};

#[test]
fn pregen_facts_constrain_live_statistical_queries() {
    let mut p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let year_zero = p.year_zero();
    assert!(!p.sites.is_empty());

    // (a) Consistency across the year-zero boundary: every committed pregen
    // pressure fact is reproduced with probability 1 by a live query run on
    // the same overlay world + ledger.
    let pressure_facts: Vec<_> = p
        .ledger
        .facts()
        .iter()
        .filter(|f| f.aspect == Aspect::RegionPressure)
        .copied()
        .collect();
    assert!(
        !pressure_facts.is_empty(),
        "history committed no pressure facts"
    );
    for f in pressure_facts.iter().take(5) {
        let (dist, _) = query(
            &p.overlay,
            &p.ledger,
            f.subject,
            f.time,
            Aspect::RegionPressure,
            LIVE_PARAMS,
        );
        assert!(
            (dist.prob(&f.value) - 1.0).abs() < 1e-9,
            "committed fact {f:?} not reproduced: p={}",
            dist.prob(&f.value)
        );
    }

    // (b) Pregen facts condition play-time queries (t > year zero): compare
    // the conditioned distribution with the unconditioned one. Some region
    // with a committed fact must be visibly constrained.
    let empty = Ledger::new();
    let t_live = year_zero + 3;
    let mut best_tv = 0.0f64;
    let mut subjects: Vec<Subject> = pressure_facts.iter().map(|f| f.subject).collect();
    subjects.dedup();
    for s in subjects.into_iter().take(12) {
        let (with, _) = query(
            &p.overlay,
            &p.ledger,
            s,
            t_live,
            Aspect::RegionPressure,
            LIVE_PARAMS,
        );
        let (without, _) = query(
            &p.overlay,
            &empty,
            s,
            t_live,
            Aspect::RegionPressure,
            LIVE_PARAMS,
        );
        best_tv = best_tv.max(with.tv_distance(&without));
    }
    assert!(
        best_tv > 0.02,
        "pregen facts do not measurably condition live queries (max TV {best_tv})"
    );

    // (c) Live observation appends to the same ledger and stays consistent.
    let live_subject = pressure_facts[0].subject;
    let before = p.ledger.len();
    let (v, _) = observe(
        &p.overlay,
        &mut p.ledger,
        live_subject,
        year_zero + 5,
        Aspect::RegionPressure,
        LIVE_PARAMS,
    )
    .expect("live observe over the pregen ledger");
    assert_eq!(p.ledger.len(), before + 1, "live fact appended");
    let (dist, _) = query(
        &p.overlay,
        &p.ledger,
        live_subject,
        year_zero + 5,
        Aspect::RegionPressure,
        LIVE_PARAMS,
    );
    assert!((dist.prob(&v) - 1.0).abs() < 1e-9, "live fact not honored");
}

#[test]
fn abandoned_site_fact_is_visible_in_ledger_and_terrain() {
    let p = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let sacked = p
        .sites
        .iter()
        .find(|s| s.abandoned.is_some())
        .expect("this seed's history sacks at least one site");
    let epoch = sacked.abandoned.unwrap();
    assert!(epoch > sacked.founded, "abandonment postdates founding");

    // Ledger view: the site existed, then did not.
    let subj = Subject::Site(sacked.slot);
    assert_eq!(
        p.ledger
            .get(subj, sacked.founded, Aspect::SiteExists)
            .map(|f| f.value),
        Some(Value::Exists(true))
    );
    assert_eq!(
        p.ledger
            .get(subj, epoch, Aspect::SiteExists)
            .map(|f| f.value),
        Some(Value::Exists(false))
    );

    // The sack was decided by an S2 collapse: a committed pressure fact for
    // the site's overlay region at the sack epoch says "raided".
    let region = Subject::Region(sacked.slot as RegionId);
    assert_eq!(
        p.ledger
            .get(region, epoch, Aspect::RegionPressure)
            .map(|f| f.value),
        Some(Value::Pressure(2)),
        "the sacking pressure collapse is in the ledger"
    );

    // Terrain view: ruins stand where the committed history says a site fell.
    let mut g = WorldGenerator::new(&p);
    let (sx, sz) = sacked.pos;
    let (ccx, ccz) = (sx.div_euclid(32), sz.div_euclid(32));
    let mut wood = 0usize;
    for dz in -1..=1i64 {
        for dx in -1..=1i64 {
            let (cx, cz) = (ccx + dx, ccz + dz);
            let cy = g.surface_chunk_y(cx, cz);
            for y in [cy, cy + 1] {
                let chunk = g.generate_chunk(ChunkPos::new(cx as i32, y, cz as i32));
                wood += chunk.blocks().iter().filter(|b| **b == Block::Wood).count();
            }
        }
    }
    assert!(wood > 0, "no ruin posts at the abandoned site");
}
