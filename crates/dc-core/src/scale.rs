//! Voxel scale: how many voxels fit in a meter.
//!
//! This is the parameter spike S1 exists to resolve. Nothing anywhere may
//! hard-code a voxel:meter ratio — everything that needs one takes a
//! [`VoxelScale`].

use serde::{Deserialize, Serialize};

/// Voxels-per-meter conversion. Constructed from "the player is H meters and
/// N voxels tall", the framing the design actually argues about.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct VoxelScale {
    voxels_per_meter: f64,
}

impl VoxelScale {
    /// A scale where a player of `player_height_m` meters stands
    /// `player_height_voxels` voxels tall.
    ///
    /// # Panics
    /// Panics if either argument is not positive and finite.
    pub fn from_player_height(player_height_m: f64, player_height_voxels: u32) -> Self {
        assert!(
            player_height_m.is_finite() && player_height_m > 0.0 && player_height_voxels > 0,
            "player height must be positive"
        );
        Self {
            voxels_per_meter: f64::from(player_height_voxels) / player_height_m,
        }
    }

    /// A scale where one voxel is `voxel_size_m` meters on a side. Used by the
    /// far-mesh path to sample coarse LOD grids (a level-L voxel is simply a
    /// `2^L`-times-larger voxel).
    ///
    /// # Panics
    /// Panics if `voxel_size_m` is not positive and finite.
    pub fn from_voxel_size_m(voxel_size_m: f64) -> Self {
        assert!(
            voxel_size_m.is_finite() && voxel_size_m > 0.0,
            "voxel size must be positive"
        );
        Self {
            voxels_per_meter: 1.0 / voxel_size_m,
        }
    }

    #[inline]
    pub fn voxels_per_meter(self) -> f64 {
        self.voxels_per_meter
    }

    /// Edge length of one voxel in meters.
    #[inline]
    pub fn voxel_size_m(self) -> f64 {
        1.0 / self.voxels_per_meter
    }

    /// Convert a length/coordinate in meters to voxel units.
    #[inline]
    pub fn meters_to_voxels(self, m: f64) -> f64 {
        m * self.voxels_per_meter
    }

    /// Convert a length/coordinate in voxel units to meters.
    #[inline]
    pub fn voxels_to_meters(self, v: f64) -> f64 {
        v / self.voxels_per_meter
    }

    /// World-space voxel coordinate containing a world-space position in meters.
    #[inline]
    pub fn voxel_at(self, m: f64) -> i64 {
        self.meters_to_voxels(m).floor() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_height_scales() {
        for n in [2u32, 3, 4] {
            let s = VoxelScale::from_player_height(1.8, n);
            assert!((s.voxel_size_m() - 1.8 / f64::from(n)).abs() < 1e-12);
            // The player really is N voxels tall at this scale.
            assert!((s.meters_to_voxels(1.8) - f64::from(n)).abs() < 1e-12);
        }
    }

    #[test]
    fn conversions_roundtrip() {
        let s = VoxelScale::from_player_height(1.8, 3);
        for m in [-123.456, 0.0, 0.9, 7777.25] {
            assert!((s.voxels_to_meters(s.meters_to_voxels(m)) - m).abs() < 1e-9);
        }
    }

    #[test]
    fn voxel_at_floors() {
        let s = VoxelScale::from_player_height(2.0, 2); // 1 voxel = 1 m
        assert_eq!(s.voxel_at(0.5), 0);
        assert_eq!(s.voxel_at(-0.5), -1);
        assert_eq!(s.voxel_at(2.0), 2);
    }
}
