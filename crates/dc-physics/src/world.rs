//! [`PhysicsWorld`]: fixed-timestep rapier stepping with collider bubbles.

use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use dc_core::VoxelQuery;
use glam::DVec3;
use rapier3d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier3d::geometry::{BroadPhaseBvh, ColliderBuilder, ColliderSet, NarrowPhase, SharedShape};
use rapier3d::math::{Pose, Vector as RVector};
use rapier3d::pipeline::PhysicsPipeline;

use crate::bubble::{BubbleConfig, BubbleSet, TileEntry, TileKey, tile_of};
use crate::merge::merge_boxes;
use crate::prop::{rotate_f, rotate_i, snap_rotation};

/// Tuning for a [`PhysicsWorld`].
#[derive(Clone, Copy, Debug)]
pub struct PhysicsConfig {
    /// Fixed simulation step, seconds. Stepping is accumulator-driven and
    /// independent of frame rate.
    pub fixed_dt: f64,
    /// Downward gravity, m/s^2. Defaults to the game's character gravity
    /// (player.rs uses 25.0) so dropped items and the player agree about how
    /// heavy the world feels.
    pub gravity_m_s2: f64,
    /// Cap on catch-up steps per [`PhysicsWorld::advance`] call; beyond this
    /// the accumulator is dropped (slow-frame spiral protection).
    pub max_steps_per_advance: u32,
    pub bubble: BubbleConfig,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            fixed_dt: 1.0 / 60.0,
            gravity_m_s2: 25.0,
            max_steps_per_advance: 8,
            bubble: BubbleConfig::default(),
        }
    }
}

/// Instrumentation for the S6 measurements, refreshed every fixed step.
#[derive(Clone, Copy, Debug, Default)]
pub struct StepStats {
    /// Tiles currently holding a collider.
    pub active_tiles: usize,
    /// Tiles cached as known-empty (scanned, no solid voxels).
    pub known_empty_tiles: usize,
    /// Total merged cuboids across all active tile compounds.
    pub active_cuboids: usize,
    /// Tiles scanned/built during the last step's bubble refresh.
    pub tiles_built_last: usize,
    /// Tiles dropped during the last step's bubble refresh.
    pub tiles_dropped_last: usize,
    /// Wall time of the last bubble refresh, milliseconds.
    pub last_bubble_ms: f64,
    /// Wall time of the last rapier step (excluding the bubble refresh), ms.
    pub last_step_ms: f64,
}

/// A body pose in world space: f64 meters + orientation.
#[derive(Clone, Copy, Debug)]
pub struct BodyPose {
    pub pos_m: DVec3,
    pub rot: glam::Quat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropError {
    /// A prop needs at least one voxel.
    Empty,
    /// The voxel bounding box is unreasonably large for one rigid body.
    TooLarge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReattachError {
    /// Handle does not name a live detached prop.
    UnknownProp,
    /// Re-attachment only happens once the body has gone to sleep.
    NotAsleep,
}

struct PropData {
    /// Voxel offsets relative to the detach-time minimum corner (the body's
    /// local frame origin). Sorted, distinct.
    offsets: Vec<[i64; 3]>,
}

/// Item friction/restitution: slightly grippy, slightly bouncy — enough to
/// tumble convincingly and still settle fast.
const ITEM_FRICTION: f32 = 0.7;
const ITEM_RESTITUTION: f32 = 0.2;
/// Terrain (bubble tile) surface parameters.
const TERRAIN_FRICTION: f32 = 0.9;
const TERRAIN_RESTITUTION: f32 = 0.0;
/// Props are heavier and deader than items.
const PROP_FRICTION: f32 = 0.9;
const PROP_RESTITUTION: f32 = 0.05;
/// Sanity cap on a single prop's bounding volume, voxels.
const MAX_PROP_BOUNDING_VOXELS: i64 = 1 << 21;

/// The dynamic-tier physics island. See crate docs for scope and rationale.
pub struct PhysicsWorld {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
    params: IntegrationParameters,
    config: PhysicsConfig,
    accumulator: f64,
    bubbles: BubbleSet,
    props: HashMap<RigidBodyHandle, PropData>,
    stats: StepStats,
}

impl PhysicsWorld {
    pub fn new(config: PhysicsConfig) -> Self {
        let params = IntegrationParameters {
            dt: config.fixed_dt as f32,
            ..Default::default()
        };
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            params,
            config,
            accumulator: 0.0,
            bubbles: BubbleSet::default(),
            props: HashMap::new(),
            stats: StepStats::default(),
        }
    }

    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }

    pub fn stats(&self) -> &StepStats {
        &self.stats
    }

    /// Number of live rigid bodies.
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Advance by a (variable) frame delta: runs zero or more fixed steps from
    /// the accumulator. Returns how many fixed steps ran. `voxel_size_m` and
    /// `solid` describe the voxel world for the bubble refresh.
    pub fn advance(&mut self, frame_dt_s: f64, voxel_size_m: f64, solid: &impl VoxelQuery) -> u32 {
        self.accumulator += frame_dt_s.max(0.0);
        let max_budget = f64::from(self.config.max_steps_per_advance) * self.config.fixed_dt;
        if self.accumulator > max_budget {
            // Spiral-of-death guard: drop time we cannot catch up on.
            self.accumulator = max_budget;
        }
        let mut steps = 0;
        while self.accumulator >= self.config.fixed_dt {
            self.step_fixed(voxel_size_m, solid);
            self.accumulator -= self.config.fixed_dt;
            steps += 1;
        }
        steps
    }

    /// Run exactly one fixed step: refresh the collider bubbles, then step
    /// rapier. Deterministic given identical prior command sequences (see
    /// crate docs).
    pub fn step_fixed(&mut self, voxel_size_m: f64, solid: &impl VoxelQuery) {
        self.refresh_bubbles(voxel_size_m, solid);

        let t0 = Instant::now();
        self.pipeline.step(
            RVector::new(0.0, -(self.config.gravity_m_s2 as f32), 0.0),
            &self.params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd,
            &(),
            &(),
        );
        self.stats.last_step_ms = t0.elapsed().as_secs_f64() * 1e3;
    }

    /// Spawn a dropped item: a small dynamic cuboid with friction/restitution,
    /// CCD, and sleep enabled. Position is the cuboid center in world meters.
    pub fn spawn_item(
        &mut self,
        center_m: DVec3,
        half_extents_m: DVec3,
        vel_m_s: DVec3,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(to_r(center_m))
            .linvel(to_r(vel_m_s))
            .ccd_enabled(true)
            .build();
        let handle = self.bodies.insert(body);
        let collider = ColliderBuilder::cuboid(
            half_extents_m.x as f32,
            half_extents_m.y as f32,
            half_extents_m.z as f32,
        )
        .friction(ITEM_FRICTION)
        .restitution(ITEM_RESTITUTION)
        .build();
        self.colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        handle
    }

    /// Remove a body (item or detached prop) and its colliders.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        self.props.remove(&handle);
        self.bodies.remove(
            handle,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    pub fn body_pose(&self, handle: RigidBodyHandle) -> Option<BodyPose> {
        let body = self.bodies.get(handle)?;
        let t = body.translation();
        let r = body.rotation();
        Some(BodyPose {
            pos_m: DVec3::new(f64::from(t.x), f64::from(t.y), f64::from(t.z)),
            rot: glam::Quat::from_xyzw(r.x, r.y, r.z, r.w),
        })
    }

    pub fn is_asleep(&self, handle: RigidBodyHandle) -> bool {
        self.bodies.get(handle).is_some_and(|b| b.is_sleeping())
    }

    /// Detach a connected set of voxels (world voxel coordinates) into ONE
    /// dynamic rigid body with a compound collider of greedy-merged cuboids at
    /// voxel-perfect fidelity. The caller clears the voxels from the world;
    /// this crate never writes chunks.
    pub fn detach_prop(
        &mut self,
        voxels: &[(i64, i64, i64)],
        voxel_size_m: f64,
    ) -> Result<RigidBodyHandle, PropError> {
        if voxels.is_empty() {
            return Err(PropError::Empty);
        }
        // BTreeSet: dedupe + deterministic order for offsets and merge input.
        let cells: BTreeSet<(i64, i64, i64)> = voxels.iter().copied().collect();
        let mut min = [i64::MAX; 3];
        let mut max = [i64::MIN; 3];
        for &(x, y, z) in &cells {
            for (i, v) in [x, y, z].into_iter().enumerate() {
                min[i] = min[i].min(v);
                max[i] = max[i].max(v);
            }
        }
        let dims = [
            max[0] - min[0] + 1,
            max[1] - min[1] + 1,
            max[2] - min[2] + 1,
        ];
        if dims.iter().product::<i64>() > MAX_PROP_BOUNDING_VOXELS {
            return Err(PropError::TooLarge);
        }
        let offsets: Vec<[i64; 3]> = cells
            .iter()
            .map(|&(x, y, z)| [x - min[0], y - min[1], z - min[2]])
            .collect();

        let local: BTreeSet<(i32, i32, i32)> = offsets
            .iter()
            .map(|o| (o[0] as i32, o[1] as i32, o[2] as i32))
            .collect();
        let boxes = merge_boxes(
            [dims[0] as i32, dims[1] as i32, dims[2] as i32],
            |x, y, z| local.contains(&(x, y, z)),
        );

        let vs = voxel_size_m as f32;
        let children: Vec<(Pose, SharedShape)> = boxes
            .iter()
            .map(|b| {
                let half = [
                    b.size[0] as f32 * vs * 0.5,
                    b.size[1] as f32 * vs * 0.5,
                    b.size[2] as f32 * vs * 0.5,
                ];
                let center = RVector::new(
                    b.min[0] as f32 * vs + half[0],
                    b.min[1] as f32 * vs + half[1],
                    b.min[2] as f32 * vs + half[2],
                );
                (
                    Pose::from_translation(center),
                    SharedShape::cuboid(half[0], half[1], half[2]),
                )
            })
            .collect();

        // Body local origin = the minimum voxel corner at detach time, so the
        // initial pose leaves every voxel exactly where it was in the world.
        let origin_m = DVec3::new(min[0] as f64, min[1] as f64, min[2] as f64) * voxel_size_m;
        let body = RigidBodyBuilder::dynamic()
            .translation(to_r(origin_m))
            .ccd_enabled(true)
            .build();
        let handle = self.bodies.insert(body);
        let collider = ColliderBuilder::new(SharedShape::compound(children))
            .friction(PROP_FRICTION)
            .restitution(PROP_RESTITUTION)
            .build();
        self.colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        self.props.insert(handle, PropData { offsets });
        Ok(handle)
    }

    /// Re-attach an asleep prop: snap its pose to the nearest lattice
    /// alignment (nearest of the 24 axis rotations + nearest voxel-center
    /// translation) and return the world voxel coordinates the caller should
    /// write the blocks back into. Removes the body on success.
    pub fn reattach_prop(
        &mut self,
        handle: RigidBodyHandle,
        voxel_size_m: f64,
    ) -> Result<Vec<(i64, i64, i64)>, ReattachError> {
        if !self.props.contains_key(&handle) {
            return Err(ReattachError::UnknownProp);
        }
        let body = self.bodies.get(handle).ok_or(ReattachError::UnknownProp)?;
        if !body.is_sleeping() {
            return Err(ReattachError::NotAsleep);
        }
        let r = body.rotation();
        let rot = snap_rotation(glam::Quat::from_xyzw(r.x, r.y, r.z, r.w));
        let t = body.translation();
        // Translation in voxel units.
        let t_v = [
            f64::from(t.x) / voxel_size_m,
            f64::from(t.y) / voxel_size_m,
            f64::from(t.z) / voxel_size_m,
        ];
        // Center of the anchor voxel (local offset 0) after the snapped
        // rotation, then snapped to the nearest voxel center. Every other
        // voxel differs by an exact integer step, so one snap aligns them all.
        let rh = rotate_f(&rot, [0.5, 0.5, 0.5]);
        let anchor = [
            (t_v[0] + rh[0] - 0.5).round() as i64,
            (t_v[1] + rh[1] - 0.5).round() as i64,
            (t_v[2] + rh[2] - 0.5).round() as i64,
        ];
        let data = self.props.get(&handle).expect("checked above");
        let out: Vec<(i64, i64, i64)> = data
            .offsets
            .iter()
            .map(|&o| {
                let ro = rotate_i(&rot, o);
                (anchor[0] + ro[0], anchor[1] + ro[1], anchor[2] + ro[2])
            })
            .collect();
        self.remove_body(handle);
        Ok(out)
    }

    /// Invalidate the collider tile covering an edited world voxel — the S6
    /// open question "world edits vs live bubbles", wired by the
    /// client-through-dc-api milestone (additive API).
    ///
    /// One `BTreeMap` remove: the tile entry (built *or* known-empty) is
    /// dropped so the next fixed step rescans the voxels through the solidity
    /// closure and rebuilds the compound collider if any bubble still wants
    /// the tile. Tile colliders are parentless (static), so removing one wakes
    /// nothing by itself — dynamic bodies whose bubble padding overlaps the
    /// tile are woken explicitly (in arena order, which is deterministic given
    /// identical command history), so a sleeper resting on the edited tile
    /// re-settles against the new geometry instead of floating on a memory.
    /// Returns `true` if a tile entry was dropped (false = no bubble had ever
    /// scanned that tile; nothing to invalidate).
    pub fn invalidate_voxel(&mut self, x: i64, y: i64, z: i64, voxel_size_m: f64) -> bool {
        let ts = self.config.bubble.tile_size_voxels;
        let key = (tile_of(x, ts), tile_of(y, ts), tile_of(z, ts));
        let Some(entry) = self.bubbles.tiles.remove(&key) else {
            return false;
        };
        if let Some(h) = entry.collider {
            self.colliders
                .remove(h, &mut self.islands, &mut self.bodies, false);
        }
        // Wake nearby dynamic bodies: same padding rule as the bubble itself
        // (margin only; velocity is zero for the sleepers this exists for).
        let tile_m = ts as f64 * voxel_size_m;
        let pad = self.config.bubble.margin_m as f32;
        let t_min = [
            (key.0 as f64 * tile_m) as f32 - pad,
            (key.1 as f64 * tile_m) as f32 - pad,
            (key.2 as f64 * tile_m) as f32 - pad,
        ];
        let t_max = [
            ((key.0 + 1) as f64 * tile_m) as f32 + pad,
            ((key.1 + 1) as f64 * tile_m) as f32 + pad,
            ((key.2 + 1) as f64 * tile_m) as f32 + pad,
        ];
        let to_wake: Vec<RigidBodyHandle> = self
            .bodies
            .iter()
            .filter(|(_, body)| body.is_dynamic())
            .filter(|(_, body)| {
                body.colliders().iter().any(|ch| {
                    let aabb = self.colliders[*ch].compute_aabb();
                    aabb.mins.x <= t_max[0]
                        && aabb.maxs.x >= t_min[0]
                        && aabb.mins.y <= t_max[1]
                        && aabb.maxs.y >= t_min[1]
                        && aabb.mins.z <= t_max[2]
                        && aabb.maxs.z >= t_min[2]
                })
            })
            .map(|(h, _)| h)
            .collect();
        for h in to_wake {
            if let Some(body) = self.bodies.get_mut(h) {
                body.wake_up(true);
            }
        }
        true
    }

    /// Recompute the wanted tile set from every dynamic body's bubble and
    /// apply the difference: build entering tiles, drop leaving ones. This is
    /// the only place voxel geometry becomes rapier colliders.
    fn refresh_bubbles(&mut self, voxel_size_m: f64, solid: &impl VoxelQuery) {
        let t0 = Instant::now();
        let ts = self.config.bubble.tile_size_voxels;
        let margin = self.config.bubble.margin_m;
        let dt = self.config.fixed_dt;

        // Wanted tiles, in deterministic order.
        let mut wanted: BTreeSet<TileKey> = BTreeSet::new();
        for (_, body) in self.bodies.iter() {
            if !body.is_dynamic() {
                continue;
            }
            // Bubble = collider AABB + margin + one step of travel.
            let vel = body.linvel();
            let travel = f64::from(vel.length()) * dt;
            let pad = (margin + travel) as f32;
            for ch in body.colliders() {
                let aabb = self.colliders[*ch].compute_aabb();
                let min_v = [
                    ((f64::from(aabb.mins.x - pad)) / voxel_size_m).floor() as i64,
                    ((f64::from(aabb.mins.y - pad)) / voxel_size_m).floor() as i64,
                    ((f64::from(aabb.mins.z - pad)) / voxel_size_m).floor() as i64,
                ];
                let max_v = [
                    ((f64::from(aabb.maxs.x + pad)) / voxel_size_m).floor() as i64,
                    ((f64::from(aabb.maxs.y + pad)) / voxel_size_m).floor() as i64,
                    ((f64::from(aabb.maxs.z + pad)) / voxel_size_m).floor() as i64,
                ];
                for tx in tile_of(min_v[0], ts)..=tile_of(max_v[0], ts) {
                    for ty in tile_of(min_v[1], ts)..=tile_of(max_v[1], ts) {
                        for tz in tile_of(min_v[2], ts)..=tile_of(max_v[2], ts) {
                            wanted.insert((tx, ty, tz));
                        }
                    }
                }
            }
        }

        // Trailing edge: drop tiles no bubble wants anymore.
        let stale: Vec<TileKey> = self
            .bubbles
            .tiles
            .keys()
            .filter(|k| !wanted.contains(*k))
            .copied()
            .collect();
        let mut dropped = 0;
        for key in stale {
            if let Some(entry) = self.bubbles.tiles.remove(&key) {
                if let Some(h) = entry.collider {
                    self.colliders
                        .remove(h, &mut self.islands, &mut self.bodies, false);
                }
                dropped += 1;
            }
        }

        // Leading edge: build tiles that just entered a bubble.
        let mut built = 0;
        for key in wanted {
            if self.bubbles.tiles.contains_key(&key) {
                continue;
            }
            let entry = self.build_tile(key, voxel_size_m, solid);
            self.bubbles.tiles.insert(key, entry);
            built += 1;
        }

        self.stats.tiles_built_last = built;
        self.stats.tiles_dropped_last = dropped;
        self.stats.active_tiles = self
            .bubbles
            .tiles
            .values()
            .filter(|e| e.collider.is_some())
            .count();
        self.stats.known_empty_tiles = self.bubbles.tiles.len() - self.stats.active_tiles;
        self.stats.active_cuboids = self.bubbles.tiles.values().map(|e| e.cuboids).sum();
        self.stats.last_bubble_ms = t0.elapsed().as_secs_f64() * 1e3;
    }

    /// Scan one tile through the solidity closure and insert its compound
    /// collider (or record it as known-empty).
    fn build_tile(
        &mut self,
        key: TileKey,
        voxel_size_m: f64,
        solid: &impl VoxelQuery,
    ) -> TileEntry {
        let ts = self.config.bubble.tile_size_voxels;
        let base = [key.0 * ts, key.1 * ts, key.2 * ts];
        let n = ts as i32;
        let boxes = merge_boxes([n, n, n], |x, y, z| {
            solid.is_solid(
                base[0] + i64::from(x),
                base[1] + i64::from(y),
                base[2] + i64::from(z),
            )
        });
        if boxes.is_empty() {
            return TileEntry {
                collider: None,
                cuboids: 0,
            };
        }
        let vs = voxel_size_m as f32;
        let children: Vec<(Pose, SharedShape)> = boxes
            .iter()
            .map(|b| {
                let half = [
                    b.size[0] as f32 * vs * 0.5,
                    b.size[1] as f32 * vs * 0.5,
                    b.size[2] as f32 * vs * 0.5,
                ];
                let center = RVector::new(
                    b.min[0] as f32 * vs + half[0],
                    b.min[1] as f32 * vs + half[1],
                    b.min[2] as f32 * vs + half[2],
                );
                (
                    Pose::from_translation(center),
                    SharedShape::cuboid(half[0], half[1], half[2]),
                )
            })
            .collect();
        let cuboids = children.len();
        let tile_min_m = RVector::new(
            (base[0] as f64 * voxel_size_m) as f32,
            (base[1] as f64 * voxel_size_m) as f32,
            (base[2] as f64 * voxel_size_m) as f32,
        );
        // Parentless collider => fixed/static, world-anchored.
        let collider = ColliderBuilder::new(SharedShape::compound(children))
            .translation(tile_min_m)
            .friction(TERRAIN_FRICTION)
            .restitution(TERRAIN_RESTITUTION)
            .build();
        let handle = self.colliders.insert(collider);
        TileEntry {
            collider: Some(handle),
            cuboids,
        }
    }
}

#[inline]
fn to_r(v: DVec3) -> RVector {
    RVector::new(v.x as f32, v.y as f32, v.z as f32)
}
