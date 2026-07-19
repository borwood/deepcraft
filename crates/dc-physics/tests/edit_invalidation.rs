//! World edits vs live bubbles (the S6 open question, wired by the
//! client-through-dc-api milestone): editing a voxel inside an active bubble
//! must invalidate the covering collider tile so the next step rebuilds the
//! compound from the new voxel data — and a sleeper resting on the edited
//! tile must wake and fall instead of floating on remembered geometry.

use std::cell::RefCell;
use std::collections::BTreeSet;

use dc_physics::{PhysicsConfig, PhysicsWorld};
use glam::DVec3;

/// N=2 voxel size (the decided scale).
const VS: f64 = 0.9;

/// An editable voxel world: a one-voxel-thick floor at y = -1 minus a set of
/// holes punched by "edits".
struct EditableFloor {
    holes: RefCell<BTreeSet<(i64, i64, i64)>>,
}

impl EditableFloor {
    fn new() -> Self {
        Self {
            holes: RefCell::new(BTreeSet::new()),
        }
    }

    fn punch(&self, x: i64, z: i64) {
        self.holes.borrow_mut().insert((x, -1, z));
    }

    fn query(&self) -> impl dc_core::VoxelQuery + '_ {
        |x: i64, y: i64, z: i64| y == -1 && !self.holes.borrow().contains(&(x, y, z))
    }
}

#[test]
fn edit_without_invalidation_is_stale_and_with_it_rebuilds_and_body_falls() {
    let floor = EditableFloor::new();
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    // Drop an item onto the floor and let it sleep.
    let item = world.spawn_item(DVec3::new(0.45, 1.5, 0.45), DVec3::splat(0.2), DVec3::ZERO);
    for _ in 0..1200 {
        world.step_fixed(VS, &floor.query());
        if world.is_asleep(item) {
            break;
        }
    }
    assert!(world.is_asleep(item), "item never slept");
    let rest_y = world.body_pose(item).expect("body").pos_m.y;
    let cuboids_before = world.stats().active_cuboids;
    assert!(cuboids_before > 0);

    // Punch out the floor voxels under (and around) the item — the whole tile
    // footprint, so the rebuilt tile scans to genuinely different geometry.
    for x in -4..8 {
        for z in -4..8 {
            floor.punch(x, z);
        }
    }

    // WITHOUT invalidation: the tile cache is a memory of the old scan — the
    // sleeper keeps resting on colliders for voxels that no longer exist.
    for _ in 0..30 {
        world.step_fixed(VS, &floor.query());
    }
    assert!(
        world.is_asleep(item),
        "stale tile must keep the body asleep"
    );
    assert_eq!(
        world.stats().active_cuboids,
        cuboids_before,
        "stale tiles must not have been rescanned"
    );

    // WITH invalidation (what the client's edit path calls per changed voxel):
    // the covering tile is dropped, the sleeper wakes, the rebuilt tiles scan
    // to the new (empty) geometry, and the body falls through the hole.
    let mut dropped_any = false;
    for x in -4..8 {
        for z in -4..8 {
            dropped_any |= world.invalidate_voxel(x, -1, z, VS);
        }
    }
    assert!(dropped_any, "at least one live tile covers the edit");
    assert!(!world.is_asleep(item), "edit under a sleeper must wake it");
    for _ in 0..60 {
        world.step_fixed(VS, &floor.query());
    }
    let fallen_y = world.body_pose(item).expect("body").pos_m.y;
    assert!(
        fallen_y < rest_y - 1.0,
        "body should fall through the edited floor: rest_y = {rest_y}, now = {fallen_y}"
    );
}

#[test]
fn invalidating_an_unscanned_tile_is_a_no_op() {
    let floor = EditableFloor::new();
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    let _item = world.spawn_item(DVec3::new(0.45, 0.5, 0.45), DVec3::splat(0.2), DVec3::ZERO);
    world.step_fixed(VS, &floor.query());
    // Far outside any bubble: nothing to invalidate, and nothing breaks.
    assert!(!world.invalidate_voxel(10_000, -1, 10_000, VS));
    // A known-empty tile inside the bubble (above the floor) also counts as
    // scanned state and is dropped + rescanned without effect on colliders.
    let cuboids = world.stats().active_cuboids;
    assert!(world.invalidate_voxel(0, 2, 0, VS));
    world.step_fixed(VS, &floor.query());
    assert_eq!(world.stats().active_cuboids, cuboids);
}
