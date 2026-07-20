//! S11 water spike invariants.
//!
//! Deliberately small grids: the measurement numbers live in
//! `examples/water_spike.rs` and `docs/spikes/S11-results.md`. What is asserted
//! here is the *properties* the spike's verdict rests on — conservation,
//! order-independence, and unload/reload identity — at a size that keeps the
//! workspace suite fast (the whole file runs in well under a second).

use std::collections::HashMap;

use dc_worldgen::water::{
    BodyGraph, BodyId, BodyKind, ConnIndex, Link, RockProps, SatField, VoxWorld, WaterEvent, bind,
    body::{level_for_volume, volume_below},
};

fn props() -> Vec<RockProps> {
    vec![
        RockProps {
            porosity: 1.0,
            perm: 0.0,
        },
        RockProps {
            porosity: 0.30,
            perm: 1.0,
        },
    ]
}

fn wet_field(n: usize, dy: usize) -> SatField {
    let mut f = SatField::new(n, dy, n, props());
    f.lateral_c = 0.03;
    for i in 0..f.sat.len() {
        f.sat[i] = ((i % 11) as f32) / 10.0;
    }
    f
}

// ---------------------------------------------------------------- bound water

#[test]
fn bound_water_relaxation_conserves_mass_exactly() {
    let mut f = wet_field(16, 12);
    let before = f.total_water();
    f.relax(200);
    let after = f.total_water() + f.crossed_to_free;
    assert!(
        (after - before).abs() / before < 1e-5,
        "bound water drifted: {before} -> {after}"
    );
}

#[test]
fn bound_water_relaxation_is_deterministic_bit_for_bit() {
    let mut a = wet_field(12, 10);
    let mut b = wet_field(12, 10);
    a.relax(64);
    b.relax(64);
    assert!(
        a.sat
            .iter()
            .map(|v| v.to_bits())
            .eq(b.sat.iter().map(|v| v.to_bits())),
        "relaxation is not bit-reproducible"
    );
}

#[test]
fn the_water_table_is_read_not_stored() {
    // water.md consequence 1: the table is the top of the saturated zone.
    let mut f = SatField::new(4, 8, 4, props());
    for y in 0..5 {
        for z in 0..4 {
            for x in 0..4 {
                let i = f.idx(x, y, z);
                f.sat[i] = 1.0;
            }
        }
    }
    let t = f.water_table();
    assert_eq!(
        t[0],
        Some(5.0),
        "table sits on top of the highest saturated voxel"
    );
    let mut dry = SatField::new(4, 8, 4, props());
    dry.sat.fill(0.0);
    assert_eq!(dry.water_table()[0], None, "a dry column has no table");
}

#[test]
fn a_seepage_drain_lowers_only_its_own_neighbourhood() {
    // The Q1 property in miniature: a drain's influence decays with distance.
    let n = 41;
    let dy = 10;
    let mut f = SatField::new(n, dy, n, props());
    f.lateral_c = 0.03;
    f.recharge = 0.02;
    for i in 0..f.sat.len() {
        f.sat[i] = 1.0;
    }
    // A leaky floor so the field has a finite equilibrium.
    for z in 0..n {
        for x in 0..n {
            let i = f.idx(x, 0, z);
            f.drain[i] = true;
        }
    }
    f.relax(400);
    let mut control = SatField::new(n, dy, n, props());
    control.lateral_c = f.lateral_c;
    control.recharge = f.recharge;
    control.sat.copy_from_slice(&f.sat);
    control.drain.copy_from_slice(&f.drain);
    let mut treated = SatField::new(n, dy, n, props());
    treated.lateral_c = f.lateral_c;
    treated.recharge = f.recharge;
    treated.sat.copy_from_slice(&f.sat);
    treated.drain.copy_from_slice(&f.drain);
    let c = n / 2;
    for y in 0..dy {
        let i = treated.idx(c, y, c);
        treated.drain[i] = true;
    }
    control.relax(200);
    treated.relax(200);

    let col =
        |f: &SatField, x: usize, z: usize| -> f32 { (0..dy).map(|y| f.sat[f.idx(x, y, z)]).sum() };
    let at0 = (col(&treated, c, c) - col(&control, c, c)).abs();
    let far = (col(&treated, 0, 0) - col(&control, 0, 0)).abs();
    assert!(at0 > 0.1, "the drain must change its own column ({at0})");
    assert!(
        far < at0 / 10.0,
        "influence must decay with distance: centre {at0}, corner {far}"
    );
}

// ---------------------------------------------------------------- body graph

fn hyps_for(w: &VoxWorld, conn: &mut ConnIndex) -> HashMap<u32, Vec<u32>> {
    conn.hypsometry(w)
}

#[test]
fn level_and_volume_are_inverses() {
    let hyp = vec![0, 0, 10, 10, 20, 20, 5];
    for v in [1.0f64, 5.0, 17.5, 40.0, 60.0] {
        let l = level_for_volume(&hyp, v);
        let back = volume_below(&hyp, l);
        assert!((back - v).abs() < 1e-3, "v={v} -> level {l} -> {back}");
    }
}

#[test]
fn event_resolution_is_order_independent() {
    let build = || {
        let mut g = BodyGraph::new();
        let ids: Vec<BodyId> = (0..12)
            .map(|i| g.add(BodyKind::Finite, [i, 0, 0], 40.0, 0.0))
            .collect();
        for i in 0..11usize {
            g.links.push(Link {
                a: ids[i],
                b: ids[i + 1],
                sill: 2.0 + (i % 3) as f32,
                open: true,
            });
        }
        let mut comp_of = HashMap::new();
        let mut hyps = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            let c = (i / 2) as u32;
            comp_of.insert(*id, c);
            hyps.insert(c, vec![8u32; 16]);
        }
        (g, ids, comp_of, hyps)
    };
    let (_, ids, _, _) = build();
    let events = vec![
        WaterEvent::RegimeCross {
            body: ids[0],
            delta: 200.0,
        },
        WaterEvent::Breach {
            a: ids[2],
            b: ids[9],
            sill: 5.0,
        },
        WaterEvent::OutletBlocked {
            a: ids[3],
            b: ids[4],
        },
        WaterEvent::InletCut {
            a: ids[6],
            b: ids[7],
        },
        WaterEvent::SpaceOpened { body: ids[5] },
        WaterEvent::Saturated { body: ids[1] },
        WaterEvent::RegimeCross {
            body: ids[10],
            delta: -15.0,
        },
        // The adversarial pair: a breach and a block naming the SAME link in
        // one batch. Whichever lands first, the link must end up existing,
        // blocked, and carrying the lower sill.
        WaterEvent::Breach {
            a: ids[8],
            b: ids[0],
            sill: 7.0,
        },
        WaterEvent::OutletBlocked {
            a: ids[0],
            b: ids[8],
        },
        WaterEvent::Breach {
            a: ids[0],
            b: ids[8],
            sill: 3.0,
        },
        // Several crossings into ONE body, with magnitudes chosen so that a
        // different summation order changes the low bits of the total.
        WaterEvent::RegimeCross {
            body: ids[11],
            delta: 1e16,
        },
        WaterEvent::RegimeCross {
            body: ids[11],
            delta: 1.0,
        },
        WaterEvent::RegimeCross {
            body: ids[11],
            delta: -1e16,
        },
        WaterEvent::RegimeCross {
            body: ids[11],
            delta: 3.0,
        },
    ];
    let run = |order: &[usize]| {
        let (mut g, _, comp_of, hyps) = build();
        let evs: Vec<WaterEvent> = order.iter().map(|&i| events[i]).collect();
        g.resolve(&evs, &comp_of, &hyps);
        g.to_bytes()
    };
    let identity: Vec<usize> = (0..events.len()).collect();
    let reference = run(&identity);
    let mut rng: u64 = 0x5111_0BED;
    for _ in 0..64 {
        let mut order = identity.clone();
        for i in (1..order.len()).rev() {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            order.swap(i, (rng % (i as u64 + 1)) as usize);
        }
        assert_eq!(
            run(&order),
            reference,
            "event resolution depended on arrival order: {order:?}"
        );
    }
}

#[test]
fn breaching_a_pinned_reservoir_does_not_drain_it() {
    // Scenario 4 in miniature. The sea is level-pinned; a void opened beneath
    // it fills to sea level and the sea does not move.
    let mut w = VoxWorld::solid(32, 32, 32);
    w.dig([0, 20, 0], [31, 27, 31]); // the sea
    w.dig([8, 4, 8], [23, 12, 23]); // a big void below it
    let mut conn = ConnIndex::build(&w);
    let mut g = BodyGraph::new();
    let sea = g.add(BodyKind::Pinned, [1, 21, 1], 0.0, 26.0);
    let (comp_of, _) = bind(&g, &mut conn, &w);
    g.resolve(&[], &comp_of, &hyps_for(&w, &mut conn));
    assert_eq!(g.get(g.resolve_id(sea)).level, 26.0);

    w.dig([15, 13, 15], [16, 19, 16]); // the breach shaft
    conn.apply_edit(&w, [15, 13, 15], [16, 19, 16]);
    let (comp_of, body_of) = bind(&g, &mut conn, &w);
    g.resolve(
        &[WaterEvent::SpaceOpened { body: sea }],
        &comp_of,
        &hyps_for(&w, &mut conn),
    );
    assert_eq!(
        g.get(g.resolve_id(sea)).level,
        26.0,
        "a breached sea must not drain"
    );
    assert!(
        g.water_at(&mut conn, &body_of, 12, 5, 12),
        "the void must fill to sea level"
    );
}

#[test]
fn a_finite_body_drains_into_the_void_it_is_breached_into() {
    let mut w = VoxWorld::solid(32, 32, 32);
    w.dig([8, 20, 8], [23, 27, 23]); // the lake: 16x8x16 = 2048 capacity
    w.dig([8, 4, 8], [23, 12, 23]); // the cave: 16x9x16
    let mut conn = ConnIndex::build(&w);
    let mut g = BodyGraph::new();
    let lake = g.add(BodyKind::Finite, [9, 21, 9], 16.0 * 16.0 * 4.0, 24.0);
    let (comp_of, _) = bind(&g, &mut conn, &w);
    g.resolve(&[], &comp_of, &hyps_for(&w, &mut conn));
    let before = g.get(g.resolve_id(lake)).level;
    assert!(before > 22.0, "lake starts high, got {before}");

    w.dig([15, 13, 15], [16, 19, 16]);
    conn.apply_edit(&w, [15, 13, 15], [16, 19, 16]);
    let (comp_of, _) = bind(&g, &mut conn, &w);
    let vol_before = g.get(g.resolve_id(lake)).volume;
    g.resolve(
        &[WaterEvent::SpaceOpened { body: lake }],
        &comp_of,
        &hyps_for(&w, &mut conn),
    );
    let after = g.get(g.resolve_id(lake)).level;
    assert!(
        after < 14.0,
        "the lake must drain into the cave, got {after}"
    );
    assert!(
        (g.get(g.resolve_id(lake)).volume - vol_before).abs() < 1e-6,
        "volume must be conserved through the breach"
    );
}

#[test]
fn the_far_end_of_a_dug_channel_is_the_same_body() {
    // Scenario 2 in miniature: no search at query time, and the answer is yes.
    let mut w = VoxWorld::solid(160, 32, 32);
    w.dig([0, 12, 0], [3, 16, 31]);
    let mut conn = ConnIndex::build(&w);
    let mut g = BodyGraph::new();
    let river = g.add(BodyKind::Pinned, [1, 13, 16], 0.0, 16.0);
    w.dig([4, 12, 15], [159, 15, 17]);
    conn.apply_edit(&w, [4, 12, 15], [159, 15, 17]);
    let (comp_of, body_of) = bind(&g, &mut conn, &w);
    g.resolve(
        &[WaterEvent::SpaceOpened { body: river }],
        &comp_of,
        &hyps_for(&w, &mut conn),
    );
    assert!(
        g.water_at(&mut conn, &body_of, 159, 13, 16),
        "the far end must be wet — it is the same body"
    );

    // Plug the middle: the far end becomes a different component with no body,
    // and the derivation correctly says dry.
    w.fill([80, 12, 15], [81, 15, 17]);
    conn.apply_edit(&w, [80, 12, 15], [81, 15, 17]);
    let (_, body_of) = bind(&g, &mut conn, &w);
    assert!(
        !g.water_at(&mut conn, &body_of, 159, 13, 16),
        "a split must be seen by the index"
    );
}

#[test]
fn a_sealed_basin_needs_storage_because_derivation_cannot_predict_it() {
    let mut w = VoxWorld::solid(24, 24, 24);
    w.dig([10, 6, 10], [13, 9, 13]);
    let mut conn = ConnIndex::build(&w);
    let mut g = BodyGraph::new();
    let (_, body_of) = bind(&g, &mut conn, &w);
    assert!(
        !g.water_at(&mut conn, &body_of, 11, 7, 11),
        "no body, no water — the derivation is right to say dry"
    );
    let bucket = g.add(BodyKind::Finite, [11, 7, 11], 16.0, 0.0);
    let (comp_of, body_of) = bind(&g, &mut conn, &w);
    g.resolve(&[], &comp_of, &hyps_for(&w, &mut conn));
    assert!(
        g.water_at(&mut conn, &body_of, 11, 6, 11),
        "the poured bucket must be there"
    );
    assert!(g.get(bucket).level > 6.0 && g.get(bucket).level < 8.0);
}

#[test]
fn the_graph_survives_unload_and_reload_with_identical_derived_water() {
    let mut w = VoxWorld::solid(64, 32, 64);
    w.dig([0, 12, 0], [63, 16, 3]);
    w.dig([20, 4, 20], [40, 10, 40]);
    for i in 0..6i64 {
        w.dig([8 + i * 8, 12, 4], [10 + i * 8, 16, 20 + i * 4]);
    }
    let mut conn = ConnIndex::build(&w);
    let mut g = BodyGraph::new();
    let river = g.add(BodyKind::Pinned, [1, 13, 1], 0.0, 16.0);
    let cave = g.add(BodyKind::Finite, [21, 5, 21], 1200.0, 0.0);
    let (comp_of, body_of) = bind(&g, &mut conn, &w);
    g.resolve(
        &[
            WaterEvent::SpaceOpened { body: river },
            WaterEvent::RegimeCross {
                body: cave,
                delta: 0.0,
            },
        ],
        &comp_of,
        &hyps_for(&w, &mut conn),
    );
    let before = g.derive_wet(&w, &mut conn, &body_of);
    assert!(before.iter().any(|b| *b != 0), "something must be wet");

    // Unload: only the persisted bytes and the geometry survive.
    let bytes = g.to_bytes();
    drop(conn);
    drop(g);

    let g = BodyGraph::from_bytes(&bytes);
    let mut conn = ConnIndex::build(&w);
    let (_, body_of) = bind(&g, &mut conn, &w);
    let after = g.derive_wet(&w, &mut conn, &body_of);
    assert_eq!(before, after, "derived water changed across unload/reload");
}

#[test]
fn the_index_is_a_pure_function_of_geometry_regardless_of_edit_order() {
    let digs: Vec<([i64; 3], [i64; 3])> = (0..6)
        .map(|i| ([4 + i * 8, 8, 4], [6 + i * 8, 12, 20 + (i % 3) * 6]))
        .collect();
    let run = |order: &[usize]| {
        let mut w = VoxWorld::solid(64, 24, 40);
        let mut conn = ConnIndex::build(&w);
        for &i in order {
            let (lo, hi) = digs[i];
            w.dig(lo, hi);
            conn.apply_edit(&w, lo, hi);
        }
        let mut sig = Vec::new();
        for z in (0..40).step_by(2) {
            for y in (0..24).step_by(2) {
                for x in (0..64).step_by(2) {
                    sig.push(conn.component(x, y, z));
                }
            }
        }
        (sig, conn.coarse_nodes())
    };
    let a = run(&[0, 1, 2, 3, 4, 5]);
    let b = run(&[5, 2, 0, 4, 1, 3]);
    assert_eq!(a, b, "the connectivity index depends on edit order");

    // And rebuilding from scratch agrees with the incremental path.
    let mut w = VoxWorld::solid(64, 24, 40);
    for (lo, hi) in &digs {
        w.dig(*lo, *hi);
    }
    let mut fresh = ConnIndex::build(&w);
    let mut sig = Vec::new();
    for z in (0..40).step_by(2) {
        for y in (0..24).step_by(2) {
            for x in (0..64).step_by(2) {
                sig.push(fresh.component(x, y, z));
            }
        }
    }
    assert_eq!(
        sig.len(),
        a.0.len(),
        "the two paths must sample the same grid"
    );
    // Component *ids* are canonical (lowest node wins), so they match exactly.
    assert_eq!(sig, a.0, "incremental index diverged from a fresh build");
}
