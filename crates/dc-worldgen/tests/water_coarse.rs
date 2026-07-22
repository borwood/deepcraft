//! S15 invariants — the coarse capacity summary against the **real** generator
//! and the **real** evicting chunk store.
//!
//! The measurements live in `examples/water_coarse_spike.rs` and
//! `docs/spikes/S15-results.md`; what is asserted here is the part that must not
//! rot: the summary **agrees with the authority it summarizes** (CLAUDE.md
//! § Conventions — "a summary is not an authority"), an edit delta is exact,
//! eviction is invisible, and the whole thing is order-independent.
//!
//! `Extent::Small` and small footprints throughout — these are invariants, not
//! a measurement sweep.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant, HostWorld,
    Payload, Vec3i, payload,
};
use dc_core::ChunkPos;
use dc_worldgen::water::{CAP_CELL, CapSummary, ExactCurve, summary_hash};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 1337;
const CELL: i64 = CAP_CELL;
/// Half a voxel, in voxels — the decision rule group 1 fixed in advance.
const HALF_VOXEL: f64 = 0.5;

type Gen = Arc<Mutex<WorldGenerator<'static>>>;

fn harness(budget: usize) -> (Gen, HostWorld) {
    let pregen = Arc::new(Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    }));
    let wgen = Arc::new(Mutex::new(WorldGenerator::new_owned(pregen)));
    let seam = wgen.clone();
    let mut host = HostWorld::with_generator(
        SEED,
        Box::new(move |pos: ChunkPos| seam.lock().expect("generator mutex").generate_chunk(pos)),
    );
    host.set_chunk_budget(budget);
    (wgen, host)
}

fn floor_oracle(g: &Gen) -> impl FnMut(i64, i64) -> i32 + '_ {
    move |x, z| {
        g.lock()
            .expect("generator mutex")
            .coarse_surface(x, z)
            .0
            .saturating_add(1)
    }
}

fn set_air(host: &mut HostWorld, pos: Vec3i) -> Vec<(Vec3i, bool)> {
    host.submit(CommandEnvelope {
        id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
        source: ConsumerId::new(ConsumerKind::Player, "s15-test"),
        grant: CapabilityToken::new(vec![
            Grant::WorldWrite { volume: None },
            Grant::WorldRead { volume: None },
        ]),
        payload: Payload::SetBlock(payload::SetBlock {
            pos,
            block: "dc:air".into(),
        }),
        target_tick: None,
        txn: None,
    })
    .expect("submit");
    let mut out = Vec::new();
    for entry in host.tick() {
        if let CommandResult::Ok(effects) = &entry.receipt.result {
            for c in &effects.blocks_changed {
                let opened = c.to == "dc:air" && c.from != "dc:air";
                let sealed = c.from == "dc:air" && c.to != "dc:air";
                if opened || sealed {
                    out.push((c.pos, opened));
                }
            }
        }
    }
    out
}

/// The exact voxel walk — the authority the summary must agree with.
fn exact(host: &mut HostWorld, keys: &[(i64, i64)], y0: i32, span: usize) -> ExactCurve {
    let mut curve = ExactCurve::new(y0, span);
    for &(kx, kz) in keys {
        for y in y0..y0 + span as i32 {
            for lz in 0..CELL {
                for lx in 0..CELL {
                    let p = Vec3i::new(kx * CELL + lx, i64::from(y), kz * CELL + lz);
                    if !host.block_at(p).is_solid() {
                        curve.add(y);
                    }
                }
            }
        }
    }
    curve
}

/// A 2×2 block of capacity cells near the world centre — a real container of
/// production terrain, small enough to walk exactly in a test.
fn footprint() -> Vec<(i64, i64)> {
    vec![(0, 0), (0, 1), (1, 0), (1, 1)]
}

#[test]
fn coarse_capacity_agrees_with_the_exact_voxel_walk() {
    let (g, mut host) = harness(4096);
    let keys = footprint();
    let mut sum = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for k in &keys {
            sum.ensure(*k, &mut oracle);
        }
    }
    let floor = sum.floor(&keys);
    // Several depths: the shallow ones are where the summary is most compressed
    // relative to the relief it stands for.
    for depth in [1, 2, 4, 8, 16, 32] {
        let level = f64::from(floor + depth);
        let y0 = floor - 2;
        let span = (depth + 4) as usize;
        let curve = exact(&mut host, &keys, y0, span);
        let volume = curve.capacity(level);
        assert!(volume > 0.0, "depth {depth}: empty container");
        let coarse = sum.level_for(&keys, volume, level + 64.0);
        let err = (coarse - level).abs();
        assert!(
            err <= HALF_VOXEL,
            "depth {depth}: coarse level {coarse} vs exact {level} — error {err} voxels \
             exceeds the half-voxel rule"
        );
    }
}

#[test]
fn an_edit_delta_is_exactly_the_change_the_voxel_walk_sees() {
    let (g, mut host) = harness(4096);
    let keys = footprint();
    let mut sum = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for k in &keys {
            sum.ensure(*k, &mut oracle);
        }
    }
    let floor = sum.floor(&keys);
    let y0 = floor - 20;
    let span = 40usize;
    let level = f64::from(floor + 8);

    let before_exact = exact(&mut host, &keys, y0, span).capacity(level);
    let before_coarse = sum.capacity(&keys, level);

    // Dig a small chamber below the floor, through the audited write path.
    let mut opened = 0i64;
    for dy in 1..=6i64 {
        for dz in 0..4i64 {
            for dx in 0..4i64 {
                let p = Vec3i::new(4 + dx, i64::from(floor) - dy, 4 + dz);
                let mut oracle = floor_oracle(&g);
                for (q, op) in set_air(&mut host, p) {
                    sum.apply_open(q.x, q.y, q.z, if op { 1 } else { -1 }, &mut oracle);
                    opened += if op { 1 } else { -1 };
                }
            }
        }
    }
    assert_eq!(
        opened, 96,
        "the chamber should open exactly 96 solid voxels"
    );

    let after_exact = exact(&mut host, &keys, y0, span).capacity(level);
    let after_coarse = sum.capacity(&keys, level);

    // The delta is an integer count on both sides, and they must match exactly.
    assert_eq!(after_exact - before_exact, 96.0);
    assert_eq!(after_coarse - before_coarse, 96.0);
}

#[test]
fn derived_water_survives_eviction_byte_identically() {
    let (g, mut host) = harness(100_000);
    let keys = footprint();
    let mut sum = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for k in &keys {
            sum.ensure(*k, &mut oracle);
        }
    }
    let floor = sum.floor(&keys);
    let y0 = floor - 8;
    let span = 24usize;

    // One edit, so the pinned-chunk half of the policy is exercised too.
    let edit = Vec3i::new(7, i64::from(floor) - 3, 7);
    {
        let mut oracle = floor_oracle(&g);
        for (q, op) in set_air(&mut host, edit) {
            sum.apply_open(q.x, q.y, q.z, if op { 1 } else { -1 }, &mut oracle);
        }
    }
    let before = exact(&mut host, &keys, y0, span);
    let before_hash = summary_hash(&sum);
    assert_eq!(host.chunk_residency().edited, 1);

    // Squeeze the store and storm it with fresh ground far away.
    host.set_chunk_budget(4);
    for i in 0..800i64 {
        host.block_at(Vec3i::new(500_000 + i * 32, 64, -500_000 - i * 32));
    }
    let residency = host.chunk_residency();
    assert!(
        residency.evicted > 0,
        "the storm did not evict: {residency:?}"
    );
    assert_eq!(residency.edited, 1, "the edited chunk was unpinned");

    let after = exact(&mut host, &keys, y0, span);
    assert_eq!(before, after, "the capacity curve changed across eviction");
    assert_eq!(before_hash, summary_hash(&sum));
    assert_eq!(
        host.block_at(edit),
        dc_core::Block::Air,
        "the edit was lost"
    );
}

#[test]
fn the_summary_is_double_run_and_order_independent_over_real_terrain() {
    let (g, _host) = harness(4096);
    let keys: Vec<(i64, i64)> = (0..6)
        .flat_map(|i| (0..6).map(move |j| (i - 3, j - 3)))
        .collect();
    let mut a = CapSummary::production();
    let mut b = CapSummary::production();
    let mut c = CapSummary::production();
    {
        let mut oracle = floor_oracle(&g);
        for k in &keys {
            a.ensure(*k, &mut oracle);
        }
        for k in keys.iter().rev() {
            b.ensure(*k, &mut oracle);
        }
        // A deterministic shuffle, so "order-independent" is not just "reversed".
        let shuffled: Vec<(i64, i64)> = (0..keys.len())
            .map(|i| keys[(i * 23) % keys.len()])
            .collect();
        let seen: BTreeSet<(i64, i64)> = shuffled.iter().copied().collect();
        assert_eq!(seen.len(), keys.len());
        for k in shuffled {
            c.ensure(k, &mut oracle);
        }
    }
    let h = summary_hash(&a);
    assert_eq!(h, summary_hash(&b));
    assert_eq!(h, summary_hash(&c));
}
