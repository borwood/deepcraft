//! Greedy merging of solid voxels into larger collision cuboids.
//!
//! Strategy (the documented S6 choice): **greedy rectangles per horizontal
//! layer, plus a column merge**. For each y layer we run the classic two-axis
//! greedy sweep — extend a run of solid cells along +x, then widen it along +z
//! while every covered cell is solid — and each resulting rectangle becomes a
//! 1-voxel-tall box. A third cheap pass merges rectangles with *identical*
//! (x, z, w, d) footprints on adjacent layers into taller boxes, which
//! collapses walls, floors, and prop slabs into single cuboids. Everything is
//! O(cells), single-pass, and deterministic (fixed scan order). Optimal box
//! cover is NP-hard; this is deliberately the cheap 80% answer the spike scope
//! asks for.

use std::collections::HashMap;

/// An axis-aligned box of voxels in local (region-relative) coordinates.
/// Covers voxels `min[i] .. min[i] + size[i]` on each axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelBox {
    pub min: [i32; 3],
    pub size: [i32; 3],
}

impl VoxelBox {
    pub fn volume(&self) -> i64 {
        i64::from(self.size[0]) * i64::from(self.size[1]) * i64::from(self.size[2])
    }
}

/// Cover exactly the solid cells of a `dims[0] x dims[1] x dims[2]` region
/// with disjoint boxes. `solid` is queried once per cell.
pub fn merge_boxes(dims: [i32; 3], solid: impl Fn(i32, i32, i32) -> bool) -> Vec<VoxelBox> {
    let [nx, ny, nz] = dims;
    assert!(nx >= 0 && ny >= 0 && nz >= 0, "negative region dims");
    let mut boxes: Vec<VoxelBox> = Vec::new();
    // Footprint (x, z, w, d) -> index of the box whose top layer is y - 1.
    // Only consulted per exact key; iteration order never matters.
    let mut below: HashMap<(i32, i32, i32, i32), usize> = HashMap::new();
    let idx = |x: i32, z: i32| (x + z * nx) as usize;

    let mut mask = vec![false; (nx * nz).max(0) as usize];
    for y in 0..ny {
        for z in 0..nz {
            for x in 0..nx {
                mask[idx(x, z)] = solid(x, y, z);
            }
        }
        let mut current: HashMap<(i32, i32, i32, i32), usize> = HashMap::new();
        for z in 0..nz {
            let mut x = 0;
            while x < nx {
                if !mask[idx(x, z)] {
                    x += 1;
                    continue;
                }
                // Extend the run along +x.
                let mut w = 1;
                while x + w < nx && mask[idx(x + w, z)] {
                    w += 1;
                }
                // Widen along +z while the whole row stays solid.
                let mut d = 1;
                'widen: while z + d < nz {
                    for xi in x..x + w {
                        if !mask[idx(xi, z + d)] {
                            break 'widen;
                        }
                    }
                    d += 1;
                }
                // Consume the rectangle so later scans skip it.
                for zi in z..z + d {
                    for xi in x..x + w {
                        mask[idx(xi, zi)] = false;
                    }
                }
                let footprint = (x, z, w, d);
                if let Some(&bi) = below.get(&footprint) {
                    // Identical footprint directly below: grow that box upward.
                    boxes[bi].size[1] += 1;
                    current.insert(footprint, bi);
                } else {
                    boxes.push(VoxelBox {
                        min: [x, y, z],
                        size: [w, 1, d],
                    });
                    current.insert(footprint, boxes.len() - 1);
                }
                x += w;
            }
        }
        below = current;
    }
    boxes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Boxes must be disjoint, in-bounds, and cover exactly the solid cells.
    fn assert_exact_cover(dims: [i32; 3], solid: &dyn Fn(i32, i32, i32) -> bool) -> Vec<VoxelBox> {
        let boxes = merge_boxes(dims, solid);
        let mut covered: HashSet<(i32, i32, i32)> = HashSet::new();
        for b in &boxes {
            for i in 0..3 {
                assert!(b.min[i] >= 0 && b.min[i] + b.size[i] <= dims[i], "{b:?}");
                assert!(b.size[i] >= 1, "{b:?}");
            }
            for y in b.min[1]..b.min[1] + b.size[1] {
                for z in b.min[2]..b.min[2] + b.size[2] {
                    for x in b.min[0]..b.min[0] + b.size[0] {
                        assert!(covered.insert((x, y, z)), "overlap at ({x},{y},{z})");
                        assert!(solid(x, y, z), "box covers non-solid ({x},{y},{z})");
                    }
                }
            }
        }
        for y in 0..dims[1] {
            for z in 0..dims[2] {
                for x in 0..dims[0] {
                    if solid(x, y, z) {
                        assert!(covered.contains(&(x, y, z)), "uncovered solid ({x},{y},{z})");
                    }
                }
            }
        }
        boxes
    }

    #[test]
    fn full_region_is_one_box() {
        let boxes = assert_exact_cover([4, 5, 6], &|_, _, _| true);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0], VoxelBox { min: [0, 0, 0], size: [4, 5, 6] });
    }

    #[test]
    fn empty_region_is_no_boxes() {
        assert!(assert_exact_cover([8, 8, 8], &|_, _, _| false).is_empty());
    }

    #[test]
    fn single_voxel() {
        let boxes = assert_exact_cover([3, 3, 3], &|x, y, z| (x, y, z) == (1, 2, 0));
        assert_eq!(boxes, vec![VoxelBox { min: [1, 2, 0], size: [1, 1, 1] }]);
    }

    #[test]
    fn flat_floor_merges_to_one_box_per_slab() {
        // Two-voxel-thick floor: one box (layer rectangles column-merge).
        let boxes = assert_exact_cover([16, 8, 16], &|_, y, _| y < 2);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].size, [16, 2, 16]);
    }

    #[test]
    fn checkerboard_covers_exactly() {
        let boxes = assert_exact_cover([8, 2, 8], &|x, y, z| (x + y + z) % 2 == 0);
        let total: i64 = boxes.iter().map(VoxelBox::volume).sum();
        assert_eq!(total, 8 * 2 * 8 / 2);
    }

    #[test]
    fn l_shape_merges_into_few_boxes() {
        // An L: a 6x1x2 bar plus a 2x1x4 leg.
        let solid = |x: i32, y: i32, z: i32| y == 0 && ((z < 2 && x < 6) || (x < 2 && z < 6));
        let boxes = assert_exact_cover([6, 1, 6], &solid);
        assert!(boxes.len() <= 2, "expected <= 2 boxes, got {boxes:?}");
    }
}
