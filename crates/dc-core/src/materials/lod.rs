//! LOD downsampling for mixed-material voxels (spike S8; answers S3 open
//! question #3, "MajorityNonAir votes on block ids — what does 'majority'
//! mean with 8 material slots?").
//!
//! The block-grid [`crate::lod::DownsampleRule`] reduces `[Block; 8]` cells
//! and cannot see sidecar data, so material LOD gets a parallel pluggable
//! rule with the *same cell geometry*: [`MixtureDownsampleRule`] reduces the
//! 2x2x2 cell of child [`VoxelContents`] (64 eighths) to one parent
//! [`VoxelContents`] (8 eighths), and [`derive_material_lod_chunk`] mirrors
//! [`crate::lod::derive_lod_chunk`]'s octant layout exactly, so the two
//! pyramids compose voxel-for-voxel (tested against [`crate::lod::LodPyramid`]).
//!
//! Default rule [`DominantClassDebrisAware`]:
//!
//! - occupancy votes in **eighths**, not blocks: the parent voxel is non-empty
//!   iff >= 32 of the 64 child eighths are occupied (ties -> occupied,
//!   matching MajorityNonAir's tie-to-solid). A cell of half-full debris
//!   voxels counts exactly as much material as it holds — debris awareness.
//! - class by volume: structural fill vs loose (debris + pore fill). The
//!   dominant class decides the parent's form; the loser is folded in rather
//!   than lost (structure-dominant keeps loose volume as pore fill;
//!   debris-dominant folds minority rubble into the debris mix).
//! - within the winning form, material slots are apportioned by largest
//!   remainder over the cell's eighth counts (deterministic: remainder ties
//!   break by ascending id), so the parent mixture is the child mixture at
//!   1/8 resolution.
//!
//! Invariant (proved in tests, induction over levels): if the block grid
//! marks exactly the non-empty material voxels solid, then wherever material
//! LOD is non-empty, [`crate::lod::MajorityNonAir`] block LOD is solid
//! (32+ occupied eighths force 4+ non-empty child voxels). Material LOD
//! never claims volume the block pyramid dissolved.

use std::collections::BTreeMap;

use super::MaterialId;
use super::contents::{StructureShape, VOXEL_EIGHTHS, VoxelContents};
use super::intern::{MaterialChunk, MixtureId, MixtureTable};
use crate::chunk::{CHUNK_SIZE_USIZE, CHUNK_VOLUME, Chunk};

/// Reduces a 2x2x2 cell of child contents to one parent contents value.
/// Implementations must be pure functions of the cell so derivation stays
/// deterministic and cacheable (same contract as the block-grid rule).
pub trait MixtureDownsampleRule {
    fn reduce(&self, cell: &[VoxelContents; 8]) -> VoxelContents;
}

/// Default rule: dominant material class with debris awareness. See module
/// docs.
#[derive(Clone, Copy, Debug, Default)]
pub struct DominantClassDebrisAware;

impl MixtureDownsampleRule for DominantClassDebrisAware {
    fn reduce(&self, cell: &[VoxelContents; 8]) -> VoxelContents {
        let mut structure: BTreeMap<MaterialId, u32> = BTreeMap::new();
        let mut loose: BTreeMap<MaterialId, u32> = BTreeMap::new();
        let (mut s_total, mut loose_total) = (0u32, 0u32);
        for c in cell {
            for &m in c.structure() {
                *structure.entry(m).or_insert(0) += 1;
                s_total += 1;
            }
            for &m in c.pore_fill().iter().chain(c.debris()) {
                *loose.entry(m).or_insert(0) += 1;
                loose_total += 1;
            }
        }
        let occupied = s_total + loose_total;
        let cell_eighths = 8 * u32::from(VOXEL_EIGHTHS);
        if occupied * 2 < cell_eighths {
            return VoxelContents::EMPTY;
        }

        if s_total >= loose_total {
            // Structure-dominant: a full shape whose fill density mirrors the
            // cell's structural volume; surviving loose volume packs the pores.
            let fill = round_to_parent_eighths(s_total).clamp(1, 8);
            let fill_mats = scale_to_slots(&structure, fill);
            let pores = VOXEL_EIGHTHS - fill;
            let pore_slots = round_to_parent_eighths(loose_total).min(pores);
            let pore_mats = scale_to_slots(&loose, pore_slots);
            VoxelContents::new(StructureShape::Full, &fill_mats, &pore_mats, &[])
                .expect("scaled fill fits Full capacity by construction")
        } else {
            // Debris-dominant: everything (minority rubble included) reads as
            // one loose mix at 1/8 resolution.
            let mut all = loose;
            for (m, c) in structure {
                *all.entry(m).or_insert(0) += c;
            }
            let slots = round_to_parent_eighths(occupied).clamp(1, 8);
            let mats = scale_to_slots(&all, slots);
            VoxelContents::debris_only(&mats).expect("at most 8 slots by construction")
        }
    }
}

/// Child-cell eighths -> parent eighths, round half up (64 child eighths make
/// 8 parent eighths).
#[inline]
fn round_to_parent_eighths(child_eighths: u32) -> u8 {
    ((child_eighths + 4) / 8).min(8) as u8
}

/// Apportion `slots` parent eighths among materials proportionally to their
/// cell counts: largest-remainder method, remainder ties broken by ascending
/// id. Returns the expanded material list (ascending id — already canonical).
fn scale_to_slots(counts: &BTreeMap<MaterialId, u32>, slots: u8) -> Vec<MaterialId> {
    let total: u32 = counts.values().sum();
    let k = u32::from(slots);
    if total == 0 || k == 0 {
        return Vec::new();
    }
    // (material, floor quota, remainder), in ascending-id order.
    let mut alloc: Vec<(MaterialId, u32, u32)> = counts
        .iter()
        .map(|(&m, &c)| (m, c * k / total, c * k % total))
        .collect();
    let assigned: u32 = alloc.iter().map(|(_, q, _)| q).sum();
    // Hand the leftover slots to the largest remainders (ties: lowest id —
    // alloc is id-ordered and the sort is stable).
    let mut order: Vec<usize> = (0..alloc.len()).collect();
    order.sort_by(|&a, &b| alloc[b].2.cmp(&alloc[a].2));
    for &i in order.iter().take((k - assigned) as usize) {
        alloc[i].1 += 1;
    }
    let mut out = Vec::with_capacity(k as usize);
    for (m, q, _) in alloc {
        for _ in 0..q {
            out.push(m);
        }
    }
    out
}

/// Derive one parent material chunk from its (up to 8) children, interning
/// any newly produced parent mixtures into the shared region `table`.
/// Missing children read as all-empty. `children` is indexed by
/// [`crate::lod::child_positions`] order; cell geometry matches
/// [`crate::lod::derive_lod_chunk`] exactly.
///
/// # Panics
/// Panics if a child references a mixture id absent from `table` (the caller
/// owns the region invariant that chunks and table travel together).
pub fn derive_material_lod_chunk(
    children: &[Option<&MaterialChunk>; 8],
    table: &mut MixtureTable,
    rule: &dyn MixtureDownsampleRule,
) -> MaterialChunk {
    let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
    let half = CHUNK_SIZE_USIZE / 2;
    for y in 0..CHUNK_SIZE_USIZE {
        for z in 0..CHUNK_SIZE_USIZE {
            for x in 0..CHUNK_SIZE_USIZE {
                let octant = usize::from(x >= half)
                    | (usize::from(z >= half) << 1)
                    | (usize::from(y >= half) << 2);
                let Some(child) = children[octant] else {
                    continue;
                };
                let (bx, by, bz) = (
                    (x * 2) % CHUNK_SIZE_USIZE,
                    (y * 2) % CHUNK_SIZE_USIZE,
                    (z * 2) % CHUNK_SIZE_USIZE,
                );
                let mut cell = [VoxelContents::EMPTY; 8];
                let mut any = false;
                for (i, slot) in cell.iter_mut().enumerate() {
                    let id = child.get(bx + (i & 1), by + ((i >> 2) & 1), bz + ((i >> 1) & 1));
                    if id != MixtureId::EMPTY {
                        *slot = *table
                            .get(id)
                            .expect("child chunk references mixture id present in region table");
                        any = true;
                    }
                }
                if any {
                    let reduced = rule.reduce(&cell);
                    dense[Chunk::index(x, y, z)] = table.intern(reduced);
                }
            }
        }
    }
    MaterialChunk::from_dense(&dense)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkPos;
    use crate::lod::{LodPyramid, MajorityNonAir, child_positions};
    use crate::palette::PalettedChunk;
    use crate::voxel::Block;

    fn full_of(m: MaterialId) -> VoxelContents {
        VoxelContents::debris_only(&[m; 8]).unwrap()
    }

    #[test]
    fn under_half_occupancy_reduces_to_empty() {
        let rule = DominantClassDebrisAware;
        // 3 full voxels of 8 = 24 of 64 eighths -> empty.
        let mut cell = [VoxelContents::EMPTY; 8];
        cell[0] = full_of(MaterialId::SAND);
        cell[1] = full_of(MaterialId::SAND);
        cell[2] = full_of(MaterialId::SAND);
        assert_eq!(rule.reduce(&cell), VoxelContents::EMPTY);
        // A fourth -> exactly half -> occupied (tie goes to solid), at the
        // half-volume it actually holds: 4 of 8 parent eighths.
        cell[3] = full_of(MaterialId::SAND);
        assert_eq!(
            rule.reduce(&cell),
            VoxelContents::debris_only(&[MaterialId::SAND; 4]).unwrap()
        );
    }

    #[test]
    fn eighths_vote_not_voxels() {
        // 8 voxels each holding 4 sand eighths: MajorityNonAir over "is the
        // voxel non-empty" would see 8/8 solid, but the *volume* is exactly
        // half — 32 of 64 -> occupied with 4 of 8 parent eighths filled.
        let rule = DominantClassDebrisAware;
        let half_full = VoxelContents::debris_only(&[MaterialId::SAND; 4]).unwrap();
        let cell = [half_full; 8];
        assert_eq!(
            rule.reduce(&cell),
            VoxelContents::debris_only(&[MaterialId::SAND; 4]).unwrap()
        );
        // 8 voxels of 3 eighths = 24 of 64 -> empty, however many voxels are
        // "occupied". This is the debris awareness MajorityNonAir lacks.
        let thin = VoxelContents::debris_only(&[MaterialId::SAND; 3]).unwrap();
        assert_eq!(rule.reduce(&[thin; 8]), VoxelContents::EMPTY);
    }

    #[test]
    fn mixture_proportions_survive_at_eighth_resolution() {
        let rule = DominantClassDebrisAware;
        // 48 sand + 16 snow of 64 -> 6 sand + 2 snow of 8.
        let sand = full_of(MaterialId::SAND);
        let snow = full_of(MaterialId::SNOW);
        let cell = [sand, sand, sand, sand, sand, sand, snow, snow];
        let reduced = rule.reduce(&cell);
        let mut expected = [MaterialId::SAND; 8];
        expected[6] = MaterialId::SNOW;
        expected[7] = MaterialId::SNOW;
        assert_eq!(reduced, VoxelContents::debris_only(&expected).unwrap());
    }

    #[test]
    fn structure_dominance_keeps_loose_as_pore_fill() {
        let rule = DominantClassDebrisAware;
        // 6 voxels of full scree structure (48 eighths), 2 voxels of full
        // silt debris (16): structure wins, silt survives in the pores.
        let wall =
            VoxelContents::new(StructureShape::Full, &[MaterialId::SCREE; 8], &[], &[]).unwrap();
        let silt = full_of(MaterialId::SILT);
        let cell = [wall, wall, wall, wall, wall, wall, silt, silt];
        let reduced = rule.reduce(&cell);
        assert_eq!(reduced.shape(), StructureShape::Full);
        assert_eq!(reduced.structure(), &[MaterialId::SCREE; 6]);
        assert_eq!(reduced.pore_fill(), &[MaterialId::SILT; 2]);
        assert!(reduced.debris().is_empty());
    }

    #[test]
    fn debris_dominance_folds_minority_rubble_in() {
        let rule = DominantClassDebrisAware;
        // 2 voxels of half-filled quarter structure (2 scree eighths each),
        // 6 voxels of full sand: debris wins, scree becomes part of the mix.
        let stub =
            VoxelContents::new(StructureShape::Quarter, &[MaterialId::SCREE; 2], &[], &[]).unwrap();
        let sand = full_of(MaterialId::SAND);
        let cell = [stub, stub, sand, sand, sand, sand, sand, sand];
        let reduced = rule.reduce(&cell);
        assert_eq!(reduced.shape(), StructureShape::None);
        // 48 sand + 4 scree of 52 occupied -> 52/8 rounds to 7 slots:
        // sand 48*7/52 = 6 r 24, scree 4*7/52 = 0 r 28 -> scree takes the
        // leftover slot.
        assert_eq!(
            reduced.debris(),
            &[
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SCREE
            ]
        );
    }

    #[test]
    fn scale_to_slots_is_exact_and_deterministic() {
        let mut counts = BTreeMap::new();
        counts.insert(MaterialId::SAND, 21u32);
        counts.insert(MaterialId::SNOW, 21u32);
        counts.insert(MaterialId::ASH, 22u32);
        let out = scale_to_slots(&counts, 8);
        assert_eq!(out.len(), 8);
        // 21/64*8 = 2 r 40, 21 -> 2 r 40, 22 -> 2 r 48: ash takes a leftover
        // slot first (largest remainder), then the sand/snow tie breaks by id
        // (sand < ash? no — ordering is by remainder desc, ties lowest id:
        // sand id 0 beats snow id 2).
        let count = |m: MaterialId| out.iter().filter(|&&x| x == m).count();
        assert_eq!(count(MaterialId::ASH), 3);
        assert_eq!(count(MaterialId::SAND), 3);
        assert_eq!(count(MaterialId::SNOW), 2);
        assert!(out.is_sorted(), "output is canonical (ascending id)");
        assert_eq!(out, scale_to_slots(&counts, 8));
    }

    /// Build a mound scene: full-debris voxels below a sloped height field,
    /// as parallel material chunks + block chunks (Dirt where non-empty).
    fn mound_scene(table: &mut MixtureTable) -> (Vec<MaterialChunk>, Vec<PalettedChunk>) {
        let sand_full = table.intern(full_of(MaterialId::SAND));
        let snow_full = table.intern(full_of(MaterialId::SNOW));
        let mixed = table.intern(
            VoxelContents::debris_only(&[
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SAND,
                MaterialId::SNOW,
                MaterialId::SNOW,
                MaterialId::SNOW,
                MaterialId::SNOW,
            ])
            .unwrap(),
        );
        let mut material_chunks = Vec::new();
        let mut block_chunks = Vec::new();
        for (ci, pos) in child_positions(ChunkPos::new(0, 0, 0)).iter().enumerate() {
            let mut ids = vec![MixtureId::EMPTY; CHUNK_VOLUME];
            let mut dense = Chunk::new();
            let base_y = pos.y * 32;
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let wx = pos.x as usize * 32 + x;
                    let wz = pos.z as usize * 32 + z;
                    // Sloped mound height in world voxels.
                    let h = (wx + wz) / 3;
                    for y in 0..CHUNK_SIZE_USIZE {
                        let wy = base_y as usize + y;
                        if wy < h {
                            // Vary material at 4-voxel granularity so some
                            // 2x2x2 cells are uniform (real deposits are
                            // spatially correlated, not per-voxel noise).
                            let id = match (wx / 4 + wy / 4 + wz / 4 + ci) % 3 {
                                0 => sand_full,
                                1 => snow_full,
                                _ => mixed,
                            };
                            ids[Chunk::index(x, y, z)] = id;
                            dense.set(x, y, z, Block::Dirt);
                        }
                    }
                }
            }
            material_chunks.push(MaterialChunk::from_dense(&ids));
            block_chunks.push(PalettedChunk::from_dense(&dense));
        }
        (material_chunks, block_chunks)
    }

    #[test]
    fn composes_with_the_block_pyramid_over_two_levels() {
        let mut table = MixtureTable::new();
        let (material_chunks, block_chunks) = mound_scene(&mut table);

        // Block side: the S3 pyramid with the default rule.
        let mut pyramid = LodPyramid::new(2, Box::new(MajorityNonAir));
        for (i, pos) in child_positions(ChunkPos::new(0, 0, 0)).iter().enumerate() {
            pyramid.insert_chunk(*pos, block_chunks[i].clone());
        }

        // Material side: derive L1 from the 8 children, then L2 from the one
        // L1 (its 7 siblings empty).
        let rule = DominantClassDebrisAware;
        let children: [Option<&MaterialChunk>; 8] =
            std::array::from_fn(|i| Some(&material_chunks[i]));
        let l1 = derive_material_lod_chunk(&children, &mut table, &rule);
        let l1_children: [Option<&MaterialChunk>; 8] =
            std::array::from_fn(|i| if i == 0 { Some(&l1) } else { None });
        let l2 = derive_material_lod_chunk(&l1_children, &mut table, &rule);

        // Every derived mixture is interned and resolvable.
        l1.validate_against(&table).expect("L1 ids resolve");
        l2.validate_against(&table).expect("L2 ids resolve");

        // Composition invariant: material-LOD-occupied implies block-LOD
        // solid, at both levels (material occupancy needs >= 32 eighths,
        // which forces >= 4 non-empty children — exactly MajorityNonAir's
        // threshold met).
        let block_l1 = pyramid
            .get_or_derive(1, ChunkPos::new(0, 0, 0))
            .expect("block L1 derives")
            .clone();
        let mut checked = 0u32;
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    if l1.get(x, y, z) != MixtureId::EMPTY {
                        assert!(
                            block_l1.get(x, y, z).is_solid(),
                            "material L1 occupied where block L1 is air at ({x},{y},{z})"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 1000, "the scene must actually exercise the rule");
        // Interior full-debris cells survive with their mixture intact.
        let sand_full = table.intern(full_of(MaterialId::SAND));
        assert!(
            l1.palette().contains(&sand_full),
            "uniform interiors stay recognizable at L1"
        );
        // Level 2 keeps the invariant against the block pyramid too.
        let block_l2 = pyramid
            .get_or_derive(2, ChunkPos::new(0, 0, 0))
            .expect("block L2 derives")
            .clone();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    if l2.get(x, y, z) != MixtureId::EMPTY {
                        assert!(
                            block_l2.get(x, y, z).is_solid(),
                            "material L2 occupied where block L2 is air at ({x},{y},{z})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn derivation_is_deterministic() {
        let build = || {
            let mut table = MixtureTable::new();
            let (material_chunks, _) = mound_scene(&mut table);
            let children: [Option<&MaterialChunk>; 8] =
                std::array::from_fn(|i| Some(&material_chunks[i]));
            let l1 = derive_material_lod_chunk(&children, &mut table, &DominantClassDebrisAware);
            (l1.encode(), table.encode())
        };
        assert_eq!(build(), build());
    }

    #[test]
    fn missing_children_read_as_empty() {
        let mut table = MixtureTable::new();
        let full = table.intern(full_of(MaterialId::GRAVEL));
        let ids = vec![full; CHUNK_VOLUME];
        let child = MaterialChunk::from_dense(&ids);
        let mut children: [Option<&MaterialChunk>; 8] = [None; 8];
        children[1] = Some(&child); // octant dx=1
        let parent = derive_material_lod_chunk(&children, &mut table, &DominantClassDebrisAware);
        assert_eq!(parent.get(16, 0, 0), full);
        assert_eq!(parent.get(31, 15, 15), full);
        assert_eq!(parent.get(15, 0, 0), MixtureId::EMPTY);
        assert_eq!(parent.get(16, 16, 0), MixtureId::EMPTY);
    }
}
