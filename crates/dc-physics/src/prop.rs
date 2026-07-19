//! Detachable voxel props: pose snapping back onto the voxel lattice.
//!
//! A detached prop is one rigid body whose compound collider is the greedy box
//! cover of its voxels ([`crate::merge`]), voxel-perfect. While simulated its
//! pose is arbitrary; re-attaching requires a pose on the lattice again. The
//! snap has two parts:
//!
//! - **Rotation**: the nearest of the 24 axis-aligned orientations (the
//!   rotation group of the cube), found by maximizing `|q_body . q_axis|`.
//!   Each such orientation is an integer signed-permutation matrix with
//!   determinant +1, so it maps voxel offsets to voxel offsets *exactly* —
//!   no floating-point error can change the shape.
//! - **Translation**: with the rotation snapped, all voxel centers differ by
//!   exact integer steps, so snapping the *anchor* voxel's center to the
//!   nearest lattice center places every other voxel on the lattice too.
//!
//! Because an integer rotation is a bijection on the lattice and offsets are
//! distinct, the reattached voxel set always has the same cardinality and a
//! congruent shape — the round-trip property `tests/s6_physics.rs` checks.

/// Integer rotation matrix (row-major), always one of the 24 proper
/// axis-aligned rotations.
pub(crate) type IntRot = [[i64; 3]; 3];

/// Apply the integer rotation to an integer vector.
#[inline]
pub(crate) fn rotate_i(m: &IntRot, v: [i64; 3]) -> [i64; 3] {
    let mut out = [0i64; 3];
    for (i, row) in m.iter().enumerate() {
        out[i] = row[0] * v[0] + row[1] * v[1] + row[2] * v[2];
    }
    out
}

/// Apply the integer rotation to an f64 vector.
#[inline]
pub(crate) fn rotate_f(m: &IntRot, v: [f64; 3]) -> [f64; 3] {
    let mut out = [0.0f64; 3];
    for (i, row) in m.iter().enumerate() {
        out[i] = row[0] as f64 * v[0] + row[1] as f64 * v[1] + row[2] as f64 * v[2];
    }
    out
}

/// The 24 proper axis-aligned rotations, paired with their quaternions.
pub(crate) fn axis_rotations() -> Vec<(IntRot, glam::Quat)> {
    let mut out = Vec::with_capacity(24);
    // All signed permutation matrices with determinant +1.
    let axes = [
        [1i64, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];
    for x in axes {
        for y in axes {
            // Orthogonal iff they occupy different coordinate axes.
            if x.iter().zip(&y).any(|(a, b)| *a != 0 && *b != 0) {
                continue;
            }
            // z = x cross y makes the determinant +1 by construction.
            let z = [
                x[1] * y[2] - x[2] * y[1],
                x[2] * y[0] - x[0] * y[2],
                x[0] * y[1] - x[1] * y[0],
            ];
            // Store row-major: rows are the images' components; columns are
            // the images of the basis vectors.
            let m: IntRot = [
                [x[0], y[0], z[0]],
                [x[1], y[1], z[1]],
                [x[2], y[2], z[2]],
            ];
            let mat = glam::Mat3::from_cols(
                glam::Vec3::new(x[0] as f32, x[1] as f32, x[2] as f32),
                glam::Vec3::new(y[0] as f32, y[1] as f32, y[2] as f32),
                glam::Vec3::new(z[0] as f32, z[1] as f32, z[2] as f32),
            );
            out.push((m, glam::Quat::from_mat3(&mat)));
        }
    }
    debug_assert_eq!(out.len(), 24);
    out
}

/// The axis-aligned rotation nearest to `q` (max `|dot|` — quaternion double
/// cover means q and -q are the same rotation).
pub(crate) fn snap_rotation(q: glam::Quat) -> IntRot {
    let mut best = ([[1, 0, 0], [0, 1, 0], [0, 0, 1]], -1.0f32);
    for (m, cand) in axis_rotations() {
        let d = q.dot(cand).abs();
        if d > best.1 {
            best = (m, d);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_24_distinct_axis_rotations() {
        let rots = axis_rotations();
        assert_eq!(rots.len(), 24);
        for (m, _) in &rots {
            // Determinant must be +1 (proper rotation).
            let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
            assert_eq!(det, 1, "improper rotation {m:?}");
        }
        let mats: Vec<IntRot> = rots.iter().map(|(m, _)| *m).collect();
        for (i, a) in mats.iter().enumerate() {
            for b in &mats[i + 1..] {
                assert_ne!(a, b, "duplicate rotation");
            }
        }
    }

    #[test]
    fn identityish_pose_snaps_to_identity() {
        let q = glam::Quat::from_euler(glam::EulerRot::YXZ, 0.05, -0.03, 0.02);
        assert_eq!(snap_rotation(q), [[1, 0, 0], [0, 1, 0], [0, 0, 1]]);
    }

    #[test]
    fn quarter_turn_about_y_snaps_to_exact_quarter_turn() {
        let q = glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + 0.04);
        let m = snap_rotation(q);
        // +90 deg about Y maps +X to -Z and +Z to +X.
        assert_eq!(rotate_i(&m, [1, 0, 0]), [0, 0, -1]);
        assert_eq!(rotate_i(&m, [0, 0, 1]), [1, 0, 0]);
        assert_eq!(rotate_i(&m, [0, 1, 0]), [0, 1, 0]);
    }

    #[test]
    fn rotation_is_lattice_bijection() {
        for (m, _) in axis_rotations() {
            // Rotating the 8 corner offsets of a cube permutes them (up to sign).
            let mut images = Vec::new();
            for v in [
                [1, 2, 3],
                [-1, 2, 3],
                [1, -2, 3],
                [1, 2, -3],
                [4, 5, 6],
            ] {
                let r = rotate_i(&m, v);
                let norm2 = |a: [i64; 3]| a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
                assert_eq!(norm2(r), norm2(v), "rotation changed length");
                images.push(r);
            }
            images.sort_unstable();
            images.dedup();
            assert_eq!(images.len(), 5, "rotation collapsed distinct points");
        }
    }
}
