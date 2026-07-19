//! Voxel raycast (DDA / Amanatides & Woo grid traversal).
//!
//! Added by the client-through-dc-api milestone: the crosshair "which voxel am
//! I looking at" query for player edits — and, later, anything else that needs
//! an exact voxel-perfect line-of-sight walk (diegetic senses, turret aim).
//!
//! Like [`crate::collision`], everything here is in **voxel units** (1.0 = one
//! voxel edge) over an unbounded 3D lattice queried through [`VoxelQuery`];
//! callers working in meters convert with [`crate::VoxelScale`] at the edge.
//! The walk visits every voxel the ray passes through, in order, so it cannot
//! skip a thin wall the way fixed-step sampling can.

use glam::DVec3;

use crate::collision::VoxelQuery;

/// What a raycast hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaycastHit {
    /// The solid voxel that was hit (world voxel coordinates).
    pub voxel: (i64, i64, i64),
    /// Unit normal of the face the ray entered through — the axis step that
    /// moved the walk into the hit voxel, negated. `(0, 0, 0)` when the ray
    /// origin was already inside a solid voxel (there is no entry face).
    pub normal: (i64, i64, i64),
}

impl RaycastHit {
    /// The voxel adjacent to the hit face — where a placed block goes.
    /// Equals the hit voxel itself for an inside-solid start (normal 0).
    pub fn adjacent(&self) -> (i64, i64, i64) {
        (
            self.voxel.0 + self.normal.0,
            self.voxel.1 + self.normal.1,
            self.voxel.2 + self.normal.2,
        )
    }
}

/// Walk the voxel grid from `origin` along `dir` until a solid voxel is found
/// or the ray has traveled `max_distance` (voxel units). Returns the first
/// solid voxel and the face it was entered through.
///
/// - `dir` need not be normalized (it is normalized internally); a zero/non-
///   finite direction returns `None`.
/// - If `origin` is inside a solid voxel the hit is that voxel with a zero
///   normal and distance 0 — callers that place blocks against the hit face
///   should treat a zero normal as "don't place".
pub fn raycast_voxels(
    world: &impl VoxelQuery,
    origin: DVec3,
    dir: DVec3,
    max_distance: f64,
) -> Option<RaycastHit> {
    let len = dir.length();
    if !len.is_finite() || len <= 0.0 || max_distance <= 0.0 {
        return None;
    }
    let dir = dir / len;

    let mut voxel = [
        origin.x.floor() as i64,
        origin.y.floor() as i64,
        origin.z.floor() as i64,
    ];
    if world.is_solid(voxel[0], voxel[1], voxel[2]) {
        return Some(RaycastHit {
            voxel: (voxel[0], voxel[1], voxel[2]),
            normal: (0, 0, 0),
        });
    }

    let o = [origin.x, origin.y, origin.z];
    let d = [dir.x, dir.y, dir.z];
    let mut step = [0i64; 3];
    // Distance along the ray to the next grid plane per axis, and the distance
    // between successive planes on that axis.
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for a in 0..3 {
        if d[a] > 0.0 {
            step[a] = 1;
            t_max[a] = ((voxel[a] + 1) as f64 - o[a]) / d[a];
            t_delta[a] = 1.0 / d[a];
        } else if d[a] < 0.0 {
            step[a] = -1;
            t_max[a] = (o[a] - voxel[a] as f64) / -d[a];
            t_delta[a] = 1.0 / -d[a];
        }
    }

    loop {
        // Advance along the axis whose next plane is nearest.
        let axis = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };
        if t_max[axis] > max_distance {
            return None;
        }
        voxel[axis] += step[axis];
        t_max[axis] += t_delta[axis];
        if world.is_solid(voxel[0], voxel[1], voxel[2]) {
            let mut normal = [0i64; 3];
            normal[axis] = -step[axis];
            return Some(RaycastHit {
                voxel: (voxel[0], voxel[1], voxel[2]),
                normal: (normal[0], normal[1], normal[2]),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_aligned_hits_first_wall_voxel_with_entry_face() {
        // Wall filling x >= 5.
        let world = |x: i64, _y: i64, _z: i64| x >= 5;
        let hit = raycast_voxels(
            &world,
            DVec3::new(0.5, 0.5, 0.5),
            DVec3::new(1.0, 0.0, 0.0),
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (5, 0, 0));
        assert_eq!(hit.normal, (-1, 0, 0));
        assert_eq!(hit.adjacent(), (4, 0, 0));

        // Straight down onto a floor below y = 0.
        let floor = |_x: i64, y: i64, _z: i64| y < 0;
        let hit = raycast_voxels(
            &floor,
            DVec3::new(0.2, 3.5, 0.9),
            DVec3::new(0.0, -2.0, 0.0), // non-normalized on purpose
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (0, -1, 0));
        assert_eq!(hit.normal, (0, 1, 0));
    }

    #[test]
    fn diagonal_ray_lands_on_the_plane_it_actually_crosses() {
        // Floor at y < 0; a 45-degree downward diagonal from (0.5, 2.5, 0.5)
        // crosses y = 0 at x = 3.0 exactly — the walk must end in a floor
        // voxel adjacent to that crossing with a +Y entry face.
        let floor = |_x: i64, y: i64, _z: i64| y < 0;
        let hit = raycast_voxels(
            &floor,
            DVec3::new(0.5, 2.5, 0.5),
            DVec3::new(1.0, -1.0, 0.0),
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.normal, (0, 1, 0));
        assert_eq!(hit.voxel.1, -1);
        assert!(
            (2..=3).contains(&hit.voxel.0),
            "entered floor near x = 3, got {:?}",
            hit.voxel
        );

        // A diagonal through a solid column cannot skip it: wall only at
        // exactly (2, 2, 2); ray passes through that voxel's interior.
        let world = |x: i64, y: i64, z: i64| (x, y, z) == (2, 2, 2);
        let hit = raycast_voxels(
            &world,
            DVec3::new(0.5, 0.5, 0.5),
            DVec3::new(1.0, 1.0, 1.0),
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (2, 2, 2));
    }

    #[test]
    fn negative_coordinates_walk_correctly() {
        let floor = |_x: i64, y: i64, _z: i64| y <= -5;
        let hit = raycast_voxels(
            &floor,
            DVec3::new(-10.5, -1.25, -20.75),
            DVec3::new(0.0, -1.0, 0.0),
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (-11, -5, -21));
        assert_eq!(hit.normal, (0, 1, 0));

        // Horizontal toward negative x, wall at x <= -4.
        let wall = |x: i64, _y: i64, _z: i64| x <= -4;
        let hit = raycast_voxels(
            &wall,
            DVec3::new(-0.5, 0.5, -0.5),
            DVec3::new(-1.0, 0.0, 0.0),
            100.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (-4, 0, -1));
        assert_eq!(hit.normal, (1, 0, 0));
    }

    #[test]
    fn starting_inside_solid_reports_zero_normal_hit() {
        let all_solid = |_x: i64, _y: i64, _z: i64| true;
        let hit = raycast_voxels(
            &all_solid,
            DVec3::new(7.3, -2.9, 0.1),
            DVec3::new(0.0, 1.0, 0.0),
            10.0,
        )
        .expect("hit");
        assert_eq!(hit.voxel, (7, -3, 0));
        assert_eq!(hit.normal, (0, 0, 0));
        assert_eq!(hit.adjacent(), hit.voxel);
    }

    #[test]
    fn max_distance_and_degenerate_directions() {
        let wall = |x: i64, _y: i64, _z: i64| x >= 5;
        // Wall face at x = 5 is 4.5 away from origin x = 0.5.
        let origin = DVec3::new(0.5, 0.5, 0.5);
        assert!(raycast_voxels(&wall, origin, DVec3::new(1.0, 0.0, 0.0), 4.4).is_none());
        assert!(raycast_voxels(&wall, origin, DVec3::new(1.0, 0.0, 0.0), 4.6).is_some());
        // Zero and non-finite directions never hit.
        assert!(raycast_voxels(&wall, origin, DVec3::ZERO, 10.0).is_none());
        assert!(raycast_voxels(&wall, origin, DVec3::new(f64::NAN, 0.0, 0.0), 10.0).is_none());
        // Zero reach never hits.
        assert!(raycast_voxels(&wall, origin, DVec3::new(1.0, 0.0, 0.0), 0.0).is_none());
    }
}
