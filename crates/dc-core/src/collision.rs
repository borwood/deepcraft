//! Swept-AABB character collision against the voxel grid.
//!
//! This is the walk path from ARCHITECTURE.md § Physics: characters are our
//! problem, not a physics engine's. All coordinates here are in **voxel
//! units** (1.0 == one voxel edge); callers working in meters convert with
//! [`crate::VoxelScale`] first.
//!
//! Algorithm: per-axis sweep in Y → X → Z order. For each axis we scan every
//! voxel cell the AABB would pass through along that axis and clamp the
//! displacement to the nearest solid face. Because the scan covers the whole
//! swept range (not fixed time steps), a fast-moving box cannot tunnel through
//! a thin floor. Clamping one axis before sweeping the next is what produces
//! wall sliding for free.

use glam::DVec3;

/// Read-only solidity query over the (unbounded, 3D) voxel world.
pub trait VoxelQuery {
    /// Is the voxel at this world-space voxel coordinate solid?
    fn is_solid(&self, x: i64, y: i64, z: i64) -> bool;
}

impl<F: Fn(i64, i64, i64) -> bool> VoxelQuery for F {
    fn is_solid(&self, x: i64, y: i64, z: i64) -> bool {
        self(x, y, z)
    }
}

/// Axis-aligned box in voxel units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl Aabb {
    /// Box from a bottom-center position, half-width (x/z) and height (y).
    pub fn from_bottom_center(bottom_center: DVec3, half_width: f64, height: f64) -> Self {
        Self {
            min: DVec3::new(
                bottom_center.x - half_width,
                bottom_center.y,
                bottom_center.z - half_width,
            ),
            max: DVec3::new(
                bottom_center.x + half_width,
                bottom_center.y + height,
                bottom_center.z + half_width,
            ),
        }
    }

    pub fn translated(self, d: DVec3) -> Self {
        Self {
            min: self.min + d,
            max: self.max + d,
        }
    }
}

/// Outcome of a swept move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveResult {
    /// Displacement actually applied (voxel units).
    pub delta: DVec3,
    /// Whether each axis was clamped by a collision.
    pub hit_x: bool,
    pub hit_y: bool,
    pub hit_z: bool,
    /// Clamped while moving downward — i.e. standing on something.
    pub on_ground: bool,
}

/// Tolerance for "exactly touching a face" comparisons, in voxel units.
const EPS: f64 = 1e-7;

/// Sweep `aabb` by `delta` through the voxel world, clamping against solid
/// voxels. Returns the displacement actually possible; the caller applies it.
pub fn move_aabb(world: &impl VoxelQuery, aabb: Aabb, delta: DVec3) -> MoveResult {
    let mut min = [aabb.min.x, aabb.min.y, aabb.min.z];
    let mut max = [aabb.max.x, aabb.max.y, aabb.max.z];
    let wanted = [delta.x, delta.y, delta.z];
    let mut applied = [0.0f64; 3];
    let mut hit = [false; 3];

    // Y first so gravity settles before horizontal motion (stable ground
    // contact), then X, then Z.
    for axis in [1usize, 0, 2] {
        let (d, h) = sweep_axis(world, &min, &max, axis, wanted[axis]);
        min[axis] += d;
        max[axis] += d;
        applied[axis] = d;
        hit[axis] = h;
    }

    MoveResult {
        delta: DVec3::new(applied[0], applied[1], applied[2]),
        hit_x: hit[0],
        hit_y: hit[1],
        hit_z: hit[2],
        on_ground: hit[1] && wanted[1] < 0.0,
    }
}

/// Clamp displacement `d` along `axis` for a box currently at `[min, max]`.
/// Returns the permitted displacement and whether it was clamped.
fn sweep_axis(
    world: &impl VoxelQuery,
    min: &[f64; 3],
    max: &[f64; 3],
    axis: usize,
    mut d: f64,
) -> (f64, bool) {
    if d == 0.0 {
        return (0.0, false);
    }
    let ua = (axis + 1) % 3;
    let va = (axis + 2) % 3;
    // Cells overlapped on the two perpendicular axes. The ±EPS shrink means a
    // box exactly flush with a face does not count as overlapping the cell on
    // the far side of that face.
    let u0 = floor_i(min[ua] + EPS);
    let u1 = floor_i(max[ua] - EPS);
    let v0 = floor_i(min[va] + EPS);
    let v1 = floor_i(max[va] - EPS);

    let mut hit = false;
    if d > 0.0 {
        // Leading face is `max[axis]`; scan cells ahead of it, nearest first.
        let start = floor_i(max[axis] + EPS);
        let end = floor_i(max[axis] + d);
        let mut a = start;
        while a <= end {
            if slab_has_solid(world, axis, a, ua, u0, u1, va, v0, v1) {
                d = (a as f64 - max[axis]).clamp(0.0, d);
                hit = true;
                break;
            }
            a += 1;
        }
    } else {
        // Leading face is `min[axis]`; scan cells below/behind it, nearest first.
        let start = floor_i(min[axis] - EPS);
        let end = floor_i(min[axis] + d);
        let mut a = start;
        while a >= end {
            if slab_has_solid(world, axis, a, ua, u0, u1, va, v0, v1) {
                d = ((a + 1) as f64 - min[axis]).clamp(d, 0.0);
                hit = true;
                break;
            }
            a -= 1;
        }
    }
    (d, hit)
}

/// Any solid voxel in the 1-cell-thick slab at `a` along `axis`, spanning
/// `[u0, u1] x [v0, v1]` on the other axes?
#[expect(
    clippy::too_many_arguments,
    reason = "internal helper, flat is clearer"
)]
fn slab_has_solid(
    world: &impl VoxelQuery,
    axis: usize,
    a: i64,
    ua: usize,
    u0: i64,
    u1: i64,
    va: usize,
    v0: i64,
    v1: i64,
) -> bool {
    let mut pos = [0i64; 3];
    pos[axis] = a;
    for u in u0..=u1 {
        for v in v0..=v1 {
            pos[ua] = u;
            pos[va] = v;
            if world.is_solid(pos[0], pos[1], pos[2]) {
                return true;
            }
        }
    }
    false
}

#[inline]
fn floor_i(x: f64) -> i64 {
    x.floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat world: everything at voxel y < 4 is solid.
    fn flat_floor(_x: i64, y: i64, _z: i64) -> bool {
        y < 4
    }

    fn player_at(bottom_center: DVec3) -> Aabb {
        // ~Minecraft-proportioned box: 0.9 wide, 2.7 tall (3-voxel player).
        Aabb::from_bottom_center(bottom_center, 0.45, 2.7)
    }

    #[test]
    fn falls_onto_ground_and_stands() {
        let start = player_at(DVec3::new(0.0, 6.0, 0.0));
        let r = move_aabb(&flat_floor, start, DVec3::new(0.0, -5.0, 0.0));
        assert!(r.hit_y && r.on_ground);
        assert!((r.delta.y - (-2.0)).abs() < 1e-6, "delta.y = {}", r.delta.y);

        // Standing on the ground: further gravity produces no movement but
        // still reports ground contact.
        let standing = start.translated(r.delta);
        let r2 = move_aabb(&flat_floor, standing, DVec3::new(0.0, -0.5, 0.0));
        assert!(r2.on_ground);
        assert!(r2.delta.y.abs() < 1e-6);
    }

    #[test]
    fn wall_slide_preserves_tangential_motion() {
        // Solid wall filling x >= 6, floor below y = 4.
        let world = |x: i64, y: i64, _z: i64| y < 4 || x >= 6;
        let aabb = player_at(DVec3::new(5.0, 4.0, 0.0)); // max.x = 5.45
        let r = move_aabb(&world, aabb, DVec3::new(2.0, 0.0, 2.0));
        assert!(r.hit_x && !r.hit_z);
        // X clamps to the wall face at 6.0, Z keeps its full motion: a slide.
        assert!((r.delta.x - 0.55).abs() < 1e-6, "delta.x = {}", r.delta.x);
        assert!((r.delta.z - 2.0).abs() < 1e-6, "delta.z = {}", r.delta.z);
    }

    #[test]
    fn cannot_enter_one_voxel_gap_narrower_than_player() {
        // Wall slab at z = 8 with a single-voxel doorway at x = 3.
        let world = |x: i64, y: i64, z: i64| y < 4 || (z == 8 && x != 3);
        // Player is 1.5 voxels wide, centered on the gap: overlaps x cells 2..=4.
        let aabb = Aabb::from_bottom_center(DVec3::new(3.5, 4.0, 6.0), 0.75, 2.7);
        let r = move_aabb(&world, aabb, DVec3::new(0.0, 0.0, 4.0));
        assert!(r.hit_z);
        // Clamped at the wall face z = 8 (front face of the slab): moved
        // 8 - (6 + 0.75) = 1.25, nowhere near the requested 4.
        assert!((r.delta.z - 1.25).abs() < 1e-6, "delta.z = {}", r.delta.z);

        // Sanity: a sub-voxel-wide box does fit through the same gap.
        let narrow = Aabb::from_bottom_center(DVec3::new(3.5, 4.0, 6.0), 0.4, 2.7);
        let r2 = move_aabb(&world, narrow, DVec3::new(0.0, 0.0, 4.0));
        assert!(!r2.hit_z);
        assert!((r2.delta.z - 4.0).abs() < 1e-6);
    }

    #[test]
    fn no_tunneling_through_thin_floor_at_high_speed() {
        // A single-voxel-thick platform at y = 10, nothing below it.
        let world = |_x: i64, y: i64, _z: i64| y == 10;
        let aabb = player_at(DVec3::new(0.0, 40.0, 0.0));
        // One enormous step: 10_000 voxels straight down.
        let r = move_aabb(&world, aabb, DVec3::new(0.0, -10_000.0, 0.0));
        assert!(r.hit_y && r.on_ground);
        let landed = aabb.translated(r.delta);
        assert!(
            (landed.min.y - 11.0).abs() < 1e-6,
            "landed at {}",
            landed.min.y
        );
    }

    #[test]
    fn diagonal_high_speed_still_lands_on_floor() {
        let world = |_x: i64, y: i64, _z: i64| y < 4;
        let aabb = player_at(DVec3::new(0.0, 50.0, 0.0));
        let r = move_aabb(&world, aabb, DVec3::new(500.0, -500.0, 500.0));
        assert!(r.on_ground);
        let landed = aabb.translated(r.delta);
        assert!((landed.min.y - 4.0).abs() < 1e-6);
        // Horizontal motion is unobstructed in this world.
        assert!((r.delta.x - 500.0).abs() < 1e-6);
        assert!((r.delta.z - 500.0).abs() < 1e-6);
    }

    #[test]
    fn resting_flush_against_wall_does_not_jitter() {
        let world = |x: i64, y: i64, _z: i64| y < 4 || x >= 6;
        // Already flush with the wall (max.x exactly 6.0).
        let aabb = Aabb {
            min: DVec3::new(5.1, 4.0, -0.45),
            max: DVec3::new(6.0, 6.7, 0.45),
        };
        let r = move_aabb(&world, aabb, DVec3::new(1.0, 0.0, 0.0));
        assert!(r.hit_x);
        assert!(r.delta.x.abs() < 1e-9);
        // And moving away is unimpeded.
        let r2 = move_aabb(&world, aabb, DVec3::new(-1.0, 0.0, 0.0));
        assert!(!r2.hit_x);
        assert!((r2.delta.x + 1.0).abs() < 1e-9);
    }
}
