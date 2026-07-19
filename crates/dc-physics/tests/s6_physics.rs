//! S6 behavior tests: items settle and sleep, bubbles refresh incrementally,
//! prop detach -> settle -> reattach round-trips, and stepping is
//! deterministic across fresh worlds.

use std::collections::BTreeSet;

use dc_physics::{PhysicsConfig, PhysicsWorld};
use glam::DVec3;

/// N=2 voxel size (the decided scale, docs/ARCHITECTURE.md § Voxel scale).
const VS: f64 = 0.9;

/// Flat world: every voxel below y=0 is solid, so the floor's top surface is
/// at exactly 0.0 m.
fn flat_floor(_x: i64, y: i64, _z: i64) -> bool {
    y < 0
}

#[test]
fn item_falls_settles_and_sleeps() {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    let item = world.spawn_item(
        DVec3::new(0.35, 5.0, 0.2),
        DVec3::splat(0.2),
        DVec3::new(0.5, 0.0, -0.3),
    );
    let mut slept_at = None;
    for step in 0..1200 {
        world.step_fixed(VS, &flat_floor);
        if world.is_asleep(item) {
            slept_at = Some(step);
            break;
        }
    }
    let slept_at = slept_at.expect("item never went to sleep in 20 s");
    let pose = world.body_pose(item).expect("body exists");
    assert!(
        (pose.pos_m.y - 0.2).abs() < 0.05,
        "item should rest with its center one half-extent above the floor, y = {} (slept at step {slept_at})",
        pose.pos_m.y
    );
}

#[test]
fn bubble_refresh_is_incremental_and_colliders_are_dropped() {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    // Spawn resting just above the floor: the bubble stabilizes immediately.
    let item = world.spawn_item(DVec3::new(0.0, 0.25, 0.0), DVec3::splat(0.2), DVec3::ZERO);

    world.step_fixed(VS, &flat_floor);
    let first_built = world.stats().tiles_built_last;
    assert!(first_built > 0, "first step must build the initial bubble");
    assert!(world.stats().active_tiles > 0, "floor tiles must be active");
    assert!(world.stats().active_cuboids > 0);

    // A stationary body must not trigger rebuilds after the bubble exists.
    for step in 0..10 {
        world.step_fixed(VS, &flat_floor);
        if step >= 2 {
            assert_eq!(
                world.stats().tiles_built_last,
                0,
                "stationary body rebuilt tiles at step {step}"
            );
            assert_eq!(world.stats().tiles_dropped_last, 0);
        }
    }

    // Removing the body must release every collider on the next refresh.
    world.remove_body(item);
    world.step_fixed(VS, &flat_floor);
    assert_eq!(world.stats().active_tiles, 0, "colliders must be dropped");
    assert_eq!(world.stats().known_empty_tiles, 0);
    assert_eq!(world.stats().active_cuboids, 0);
}

#[test]
fn moving_body_only_touches_bubble_edges() {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    // Toss an item sideways; while it travels, per-step tile work must stay a
    // small fraction of the initial full-bubble build.
    world.spawn_item(
        DVec3::new(0.0, 3.0, 0.0),
        DVec3::splat(0.2),
        DVec3::new(6.0, 0.0, 0.0),
    );
    world.step_fixed(VS, &flat_floor);
    let initial = world.stats().tiles_built_last;
    let mut max_incremental = 0;
    for _ in 0..120 {
        world.step_fixed(VS, &flat_floor);
        max_incremental = max_incremental.max(world.stats().tiles_built_last);
    }
    assert!(
        max_incremental < initial,
        "incremental rebuild ({max_incremental} tiles) should stay below the \
         initial bubble build ({initial} tiles)"
    );
}

/// An asymmetric ~100-voxel prop: a 6x4x4 slab (96) plus a 4-voxel tail on
/// one edge. Asymmetry matters — a cube would make the congruence check
/// vacuous.
fn prop_voxels(base: (i64, i64, i64)) -> Vec<(i64, i64, i64)> {
    let mut v = Vec::new();
    for x in 0..6 {
        for y in 0..4 {
            for z in 0..4 {
                v.push((base.0 + x, base.1 + y, base.2 + z));
            }
        }
    }
    for x in 0..4 {
        v.push((base.0 + x, base.1 + 4, base.2));
    }
    assert_eq!(v.len(), 100);
    v
}

/// Translate a voxel set so its minimum corner is the origin.
fn normalize(cells: &[(i64, i64, i64)]) -> BTreeSet<(i64, i64, i64)> {
    let min = cells.iter().fold((i64::MAX, i64::MAX, i64::MAX), |m, c| {
        (m.0.min(c.0), m.1.min(c.1), m.2.min(c.2))
    });
    cells
        .iter()
        .map(|c| (c.0 - min.0, c.1 - min.1, c.2 - min.2))
        .collect()
}

/// Do two voxel sets have congruent shapes (equal up to one of the 24 axis
/// rotations plus translation)?
fn congruent(a: &[(i64, i64, i64)], b: &[(i64, i64, i64)]) -> bool {
    let nb = normalize(b);
    let rots: [fn((i64, i64, i64)) -> (i64, i64, i64); 4] = [
        |v| v,
        |(x, y, z)| (-z, y, x),
        |(x, y, z)| (-x, y, -z),
        |(x, y, z)| (z, y, -x),
    ];
    // Full check over all 24 rotations: every proper axis rotation decomposes
    // as `up ∘ yaw` where `yaw` fixes +Y and `up` sends +Y to one of the six
    // axis directions.
    let ups: [fn((i64, i64, i64)) -> (i64, i64, i64); 6] = [
        |v| v,
        |(x, y, z)| (x, -y, -z),
        |(x, y, z)| (x, z, -y),
        |(x, y, z)| (x, -z, y),
        |(x, y, z)| (y, -x, z),
        |(x, y, z)| (-y, x, z),
    ];
    for up in ups {
        for rot in rots {
            let rotated: Vec<(i64, i64, i64)> = a.iter().map(|&v| up(rot(v))).collect();
            let na = normalize(&rotated);
            if na == nb {
                return true;
            }
        }
    }
    false
}

#[test]
fn prop_detach_settle_reattach_roundtrip() {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    let original = prop_voxels((2, 8, -3));

    let prop = world
        .detach_prop(&original, VS)
        .expect("valid prop detaches");

    // Not asleep yet: reattach must refuse.
    assert_eq!(
        world.reattach_prop(prop, VS),
        Err(dc_physics::ReattachError::NotAsleep)
    );

    let mut slept = false;
    for _ in 0..3600 {
        world.step_fixed(VS, &flat_floor);
        if world.is_asleep(prop) {
            slept = true;
            break;
        }
    }
    assert!(slept, "prop never settled in 60 simulated seconds");

    let reattached = world.reattach_prop(prop, VS).expect("asleep prop reattaches");

    // Round-trip properties: same voxel count, no duplicates, congruent shape.
    assert_eq!(reattached.len(), original.len(), "voxel count changed");
    let distinct: BTreeSet<_> = reattached.iter().copied().collect();
    assert_eq!(distinct.len(), reattached.len(), "reattach produced duplicates");
    assert!(
        congruent(&original, &reattached),
        "reattached shape is not congruent with the original"
    );
    // It settled onto the floor: lowest voxel sits at or just above y = 0.
    let min_y = reattached.iter().map(|c| c.1).min().unwrap();
    assert!((0..=1).contains(&min_y), "prop settled at voxel y = {min_y}");

    // The body is gone.
    assert!(world.body_pose(prop).is_none());
    assert_eq!(
        world.reattach_prop(prop, VS),
        Err(dc_physics::ReattachError::UnknownProp)
    );
}

/// A slightly hilly deterministic world so the determinism test exercises
/// slopes and multi-contact stacks, not just a flat plane.
fn bumpy(x: i64, y: i64, _z: i64) -> bool {
    let h = ((x.rem_euclid(7)) - 3).abs() - 3; // sawtooth in [-3, 0]
    y < h
}

fn run_script() -> Vec<u64> {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    let mut handles = Vec::new();
    // Deterministic LCG spawn script: 24 items tossed into the bumpy world.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut rand = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    for _ in 0..24 {
        let pos = DVec3::new(rand() * 8.0 - 4.0, 4.0 + rand() * 6.0, rand() * 8.0 - 4.0);
        let vel = DVec3::new(rand() * 6.0 - 3.0, 0.0, rand() * 6.0 - 3.0);
        handles.push(world.spawn_item(pos, DVec3::splat(0.2), vel));
    }
    // A prop in the mix: detachment must be deterministic too.
    world
        .detach_prop(&prop_voxels((-4, 10, -4)), VS)
        .expect("prop detaches");

    for _ in 0..600 {
        world.step_fixed(VS, &bumpy);
    }

    // Bit-exact fingerprint of every final pose.
    let mut bits = Vec::new();
    for (_, body) in world.bodies.iter() {
        let t = body.translation();
        let r = body.rotation();
        for v in [t.x, t.y, t.z, r.x, r.y, r.z, r.w] {
            bits.push(u64::from(v.to_bits()));
        }
    }
    bits
}

#[test]
fn identical_scripts_produce_bit_identical_final_poses() {
    let a = run_script();
    let b = run_script();
    assert!(!a.is_empty());
    assert_eq!(a, b, "two fresh worlds diverged on the same spawn script");
}
