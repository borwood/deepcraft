//! Culled chunk meshing: one quad per solid face whose neighbor is not solid.
//!
//! Deliberately the simple mesher — S1's exit criterion accepts culled meshing;
//! greedy merging is an optimization for later. Output is plain vertex arrays
//! (no renderer types) so the headless bench can run it without a GPU.
//!
//! Vertices are **chunk-local meters**: local voxel coordinates scaled by the
//! voxel size. The chunk entity's transform places the chunk relative to the
//! floating origin, keeping every f32 the GPU sees small.
//!
//! ## Mixture materialization (ROADMAP 3c-2, docs/design/visuals.md)
//!
//! When a chunk carries per-voxel material [`ContentsGrid`] data (the worldgen
//! authority's render-only sidecar view), a voxel with **mixed** contents no
//! longer renders as one flat block color. Each of its faces is subdivided into
//! [`DITHER_CELLS`]² pixel cells, and each cell picks ONE constituent material's
//! registry albedo by a **world-anchored** position hash weighted by that
//! constituent's eighths fraction — quantized pixels composing ground, no
//! blending. Uniform voxels keep the single-color fast path (byte-identical to
//! before), so the dither only pays where contents are actually mixed.
//!
//! Loose-only contents (debris role, no structure) render as **partial-height**
//! boxes (height = eighths / 8, snow-layer style); the block-tier collider stays
//! binary (the accepted, documented visible mismatch — visuals.md). Worldgen
//! does not yet emit sub-8 loose voxels (recorded strata always fill 8 eighths
//! and map to a solid block), so partial heights are exercised by tests until
//! loose-material deposition lands.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos, ContentsGrid, MaterialId, VoxelContents};

/// CPU-side mesh: positions/normals/colors + triangle indices.
#[derive(Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// The six axis-aligned faces: unit normal and the four corner offsets
/// (counter-clockwise seen from outside, for `[0, 1, 2, 0, 2, 3]` indexing).
const FACES: [([i64; 3], [[f32; 3]; 4]); 6] = [
    // +X
    (
        [1, 0, 0],
        [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
    ),
    // -X
    (
        [-1, 0, 0],
        [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    ),
    // +Y
    (
        [0, 1, 0],
        [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    ),
    // -Y
    (
        [0, -1, 0],
        [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
    ),
    // +Z
    (
        [0, 0, 1],
        [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    ),
    // -Z
    (
        [0, 0, -1],
        [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    ),
];

/// Per-face subdivision for the mixture dither: each mixed face becomes
/// `DITHER_CELLS`² pixel cells. 4 reads as chunky pixels at the ratified N=2
/// (0.9 m voxel → ~22 cm cells) while keeping the triangle cost bounded
/// (a mixed face is 16 quads vs 1). **Flagged for ratification** — 8 is the
/// finer alternative the visuals road names, at 4× these triangles.
const DITHER_CELLS: u32 = 4;

/// Albedo per block and face. Vertex-colored, no textures (S1 scope). Grass
/// gets a green top and earthy sides so the surface reads at a glance.
fn face_color(block: Block, normal_y: i64) -> [f32; 4] {
    match (block, normal_y) {
        (Block::Grass, 1) => [0.30, 0.62, 0.25, 1.0],
        (Block::Grass, _) => [0.38, 0.45, 0.22, 1.0],
        (Block::Dirt, _) => [0.42, 0.30, 0.19, 1.0],
        (Block::Stone, _) => [0.52, 0.52, 0.54, 1.0],
        (Block::Wood, _) => [0.44, 0.33, 0.17, 1.0],
        // Geology block tier (ROADMAP 3c-1): fullbright-readable, distinct at a
        // glance. Mudstone red-brown, sandstone tan, granite pinkish-grey,
        // basalt near-black. These match the material registry albedos, so a
        // uniform geology voxel renders identically whether colored by block or
        // by contents.
        (Block::Mudstone, _) => [0.46, 0.26, 0.20, 1.0],
        (Block::Sandstone, _) => [0.76, 0.66, 0.44, 1.0],
        (Block::Granite, _) => [0.66, 0.56, 0.58, 1.0],
        (Block::Basalt, _) => [0.14, 0.14, 0.16, 1.0],
        (Block::Air, _) => [1.0, 0.0, 1.0, 1.0], // never emitted
    }
}

/// A material's registry albedo as an RGBA vertex color.
#[inline]
fn material_color(m: MaterialId) -> [f32; 4] {
    let a = m.props().albedo;
    [a[0], a[1], a[2], 1.0]
}

/// The blocks whose voxels carry material contents (the recorded strata tier,
/// ROADMAP 3c-1). Gating contents consumption on the block keeps stale
/// render-only contents from bleeding through after an edit changes the block
/// (edits do not yet touch materials — visuals.md / this milestone): a voxel
/// edited to any other block falls back to its block color and full height.
#[inline]
fn block_uses_contents(block: Block) -> bool {
    matches!(
        block,
        Block::Mudstone | Block::Sandstone | Block::Granite | Block::Basalt
    )
}

/// Loose-only contents: debris role, no structure. Renders partial-height.
#[inline]
fn is_loose_only(c: &VoxelContents) -> bool {
    c.shape() == dc_core::StructureShape::None && !c.debris().is_empty()
}

/// Height fraction (in eighths / 8) a voxel's contents render at: loose-only
/// contents render as a partial box (snow-layer style); everything else fills
/// the cell.
#[inline]
fn height_frac(c: &VoxelContents) -> f32 {
    if is_loose_only(c) {
        f32::from(c.debris().len() as u8) / 8.0
    } else {
        1.0
    }
}

/// Do these contents mix more than one constituent (over all occupied
/// eighths)? Only mixed voxels dither.
#[inline]
fn is_mixed(c: &VoxelContents) -> bool {
    let slots = c.filled_slots();
    slots.iter().any(|&m| m != slots[0])
}

/// Deterministic, **world-anchored** cell hash: a pure function of world voxel
/// coordinates + face index + subcell. A given world position therefore dithers
/// identically across chunk reloads, generation orders, and runs — the
/// determinism the visuals road asks for (tested directly).
#[inline]
fn dither_hash(wx: i64, wy: i64, wz: i64, face: usize, cu: u32, cv: u32) -> u64 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    for v in [
        wx as u64,
        wy as u64,
        wz as u64,
        face as u64,
        u64::from(cu),
        u64::from(cv),
    ] {
        h ^= v
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(h << 6)
            .wrapping_add(h >> 2);
        h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
    }
    h
}

/// The constituent one subcell renders, weighted by eighths fraction. The
/// occupied-eighth multiset [`VoxelContents::filled_slots`] lists each material
/// once per eighth it fills, so a uniform hash pick over it is a
/// fraction-weighted pick over constituents — and it keeps small fractions
/// (e.g. a 2/8 placer ore grain) naturally sparse, no ore highlighting.
#[inline]
fn dither_pick(
    c: &VoxelContents,
    wx: i64,
    wy: i64,
    wz: i64,
    face: usize,
    cu: u32,
    cv: u32,
) -> MaterialId {
    let slots = c.filled_slots();
    let h = dither_hash(wx, wy, wz, face, cu, cv);
    slots[(h % slots.len() as u64) as usize]
}

/// Mesh one chunk with culled faces. `neighbor_solid` is consulted (with
/// world-space voxel coordinates) only for the one-voxel shell outside the
/// chunk, so chunk borders cull correctly against neighbors. `contents`, when
/// present, is the chunk's render-only per-voxel material view (worldgen
/// authority); passing `None` is the plain block-colored mesher (S1 terrain,
/// far field, benches) and is byte-identical to before this milestone.
pub fn mesh_chunk(
    chunk: &Chunk,
    pos: ChunkPos,
    voxel_size_m: f32,
    neighbor_solid: &dyn Fn(i64, i64, i64) -> bool,
    contents: Option<&ContentsGrid>,
) -> MeshData {
    let mut mesh = MeshData::default();
    let (mx, my, mz) = pos.min_voxel();
    let n = CHUNK_SIZE_USIZE;

    for y in 0..n {
        for z in 0..n {
            for x in 0..n {
                let block = chunk.get(x, y, z);
                if !block.is_solid() {
                    continue;
                }
                // Render-only contents, gated on the block still being a
                // material-backed geology block (see block_uses_contents).
                let voxel = contents
                    .filter(|_| block_uses_contents(block))
                    .map(|g| g.get(x, y, z))
                    .filter(|c| !c.is_empty());
                let frac = voxel.as_ref().map_or(1.0, height_frac);
                let mixed = voxel.as_ref().is_some_and(is_mixed);
                let (wx, wy, wz) = (mx + x as i64, my + y as i64, mz + z as i64);

                for (face, (normal, corners)) in FACES.iter().enumerate() {
                    let nx = x as i64 + normal[0];
                    let ny = y as i64 + normal[1];
                    let nz = z as i64 + normal[2];
                    let covered = if (0..n as i64).contains(&nx)
                        && (0..n as i64).contains(&ny)
                        && (0..n as i64).contains(&nz)
                    {
                        chunk.get(nx as usize, ny as usize, nz as usize).is_solid()
                    } else {
                        neighbor_solid(mx + nx, my + ny, mz + nz)
                    };
                    // A partial (< full-height) voxel always shows its top: the
                    // cell above it is open even when the neighbor is solid.
                    let top_of_partial = frac < 1.0 && normal[1] == 1;
                    if covered && !top_of_partial {
                        continue;
                    }
                    if mixed {
                        emit_dithered_face(
                            &mut mesh,
                            x,
                            y,
                            z,
                            (wx, wy, wz),
                            face,
                            normal,
                            corners,
                            voxel_size_m,
                            frac,
                            voxel.as_ref().expect("mixed implies contents"),
                        );
                    } else {
                        let color = voxel
                            .as_ref()
                            .map(|c| material_color(c.filled_slots()[0]))
                            .unwrap_or_else(|| face_color(block, normal[1]));
                        emit_face(
                            &mut mesh,
                            x,
                            y,
                            z,
                            normal,
                            corners,
                            voxel_size_m,
                            frac,
                            color,
                        );
                    }
                }
            }
        }
    }
    mesh
}

/// Remap a unit-cube corner to chunk-local meters, collapsing the voxel's top
/// (`y == 1`) down to its render height fraction (partial-height boxes).
#[inline]
fn corner_m(x: usize, y: usize, z: usize, c: &[f32; 3], size: f32, frac: f32) -> [f32; 3] {
    let cy = if c[1] >= 1.0 { frac } else { c[1] };
    [
        (x as f32 + c[0]) * size,
        (y as f32 + cy) * size,
        (z as f32 + c[2]) * size,
    ]
}

#[expect(
    clippy::too_many_arguments,
    reason = "internal helper, flat is clearer"
)]
fn emit_face(
    mesh: &mut MeshData,
    x: usize,
    y: usize,
    z: usize,
    normal: &[i64; 3],
    corners: &[[f32; 3]; 4],
    voxel_size_m: f32,
    frac: f32,
    color: [f32; 4],
) {
    let base = mesh.positions.len() as u32;
    let n = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    for c in corners {
        mesh.positions
            .push(corner_m(x, y, z, c, voxel_size_m, frac));
        mesh.normals.push(n);
        mesh.colors.push(color);
    }
    mesh.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Emit one face as a `DITHER_CELLS`² grid of pixel cells, each a single
/// quantized albedo chosen by [`dither_pick`]. Cell corners are bilinear over
/// the (partial-height-remapped) face corners, so the dither and partial height
/// compose. Winding matches [`emit_face`].
#[expect(
    clippy::too_many_arguments,
    reason = "internal helper, flat is clearer"
)]
fn emit_dithered_face(
    mesh: &mut MeshData,
    x: usize,
    y: usize,
    z: usize,
    world: (i64, i64, i64),
    face: usize,
    normal: &[i64; 3],
    corners: &[[f32; 3]; 4],
    voxel_size_m: f32,
    frac: f32,
    voxel: &VoxelContents,
) {
    let n = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    let fc: [[f32; 3]; 4] =
        std::array::from_fn(|i| corner_m(x, y, z, &corners[i], voxel_size_m, frac));
    // P(s,t): c0 at (0,0), c1 at (0,1), c2 at (1,1), c3 at (1,0).
    let bilinear = |s: f32, t: f32| -> [f32; 3] {
        std::array::from_fn(|k| {
            fc[0][k] * (1.0 - s) * (1.0 - t)
                + fc[3][k] * s * (1.0 - t)
                + fc[1][k] * (1.0 - s) * t
                + fc[2][k] * s * t
        })
    };
    let cells = DITHER_CELLS;
    let step = 1.0 / cells as f32;
    let (wx, wy, wz) = world;
    for i in 0..cells {
        for j in 0..cells {
            let (s0, s1) = (i as f32 * step, (i + 1) as f32 * step);
            let (t0, t1) = (j as f32 * step, (j + 1) as f32 * step);
            let quad = [
                bilinear(s0, t0),
                bilinear(s0, t1),
                bilinear(s1, t1),
                bilinear(s1, t0),
            ];
            let color = material_color(dither_pick(voxel, wx, wy, wz, face, i, j));
            let base = mesh.positions.len() as u32;
            for p in quad {
                mesh.positions.push(p);
                mesh.normals.push(n);
                mesh.colors.push(color);
            }
            mesh.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::CHUNK_VOLUME;

    fn no_neighbors(_: i64, _: i64, _: i64) -> bool {
        false
    }

    /// A one-voxel `ContentsGrid` placing `c` at (x,y,z), everything else empty.
    fn grid_with(x: usize, y: usize, z: usize, c: VoxelContents) -> ContentsGrid {
        let mut dense = vec![VoxelContents::EMPTY; CHUNK_VOLUME];
        dense[Chunk::index(x, y, z)] = c;
        ContentsGrid::from_dense(&dense)
    }

    #[test]
    fn single_voxel_emits_six_faces() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Stone);
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 0.5, &no_neighbors, None);
        assert_eq!(mesh.triangle_count(), 12);
        assert_eq!(mesh.positions.len(), 24);
        // Scaled by voxel size: every coordinate within [2.5, 3.0].
        for p in &mesh.positions {
            for v in p {
                assert!((2.5..=3.0).contains(v), "vertex {p:?} out of range");
            }
        }
    }

    #[test]
    fn interior_faces_are_culled() {
        let mut chunk = Chunk::new();
        chunk.set(10, 10, 10, Block::Stone);
        chunk.set(11, 10, 10, Block::Dirt);
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 1.0, &no_neighbors, None);
        // Two cubes sharing a face: 10 faces, not 12.
        assert_eq!(mesh.triangle_count(), 20);
    }

    #[test]
    fn full_chunk_with_solid_neighbors_emits_nothing() {
        let mut chunk = Chunk::new();
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    chunk.set(x, y, z, Block::Stone);
                }
            }
        }
        let everything_solid = |_: i64, _: i64, _: i64| true;
        let mesh = mesh_chunk(
            &chunk,
            ChunkPos::new(2, -3, 1),
            1.0,
            &everything_solid,
            None,
        );
        assert!(mesh.is_empty(), "buried chunk must mesh to nothing");

        // Same chunk with air neighbors: exactly the 6 outer faces' quads.
        let mesh = mesh_chunk(&chunk, ChunkPos::new(2, -3, 1), 1.0, &no_neighbors, None);
        let expected_quads = 6 * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;
        assert_eq!(mesh.triangle_count(), expected_quads * 2);
        // Sanity: volume matches what we filled.
        assert_eq!(CHUNK_VOLUME, 32 * 32 * 32);
    }

    #[test]
    fn border_faces_consult_neighbor_query_in_world_coords() {
        let mut chunk = Chunk::new();
        chunk.set(0, 0, 0, Block::Stone); // chunk-corner voxel
        let pos = ChunkPos::new(-1, 0, 0); // world voxel (-32, 0, 0)
        let neighbor = |x: i64, y: i64, z: i64| (x, y, z) == (-33, 0, 0);
        let mesh = mesh_chunk(&chunk, pos, 1.0, &neighbor, None);
        // -X face culled by the neighbor; 5 faces remain.
        assert_eq!(mesh.triangle_count(), 10);
    }

    #[test]
    fn uniform_contents_take_the_single_color_fast_path() {
        // A geology voxel whose contents are one material renders exactly like
        // the plain block-colored mesher: same triangle count, same colors.
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Sandstone);
        let uniform =
            VoxelContents::debris_only(&[MaterialId::SANDSTONE; 8]).expect("8 debris eighths");
        let grid = grid_with(5, 5, 5, uniform);
        let with = mesh_chunk(
            &chunk,
            ChunkPos::new(0, 0, 0),
            0.9,
            &no_neighbors,
            Some(&grid),
        );
        let without = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 0.9, &no_neighbors, None);
        assert_eq!(with.triangle_count(), 12);
        assert_eq!(with.positions, without.positions);
        // Sandstone block color equals sandstone material albedo, so identical.
        assert_eq!(with.colors, without.colors);
    }

    #[test]
    fn mixed_contents_dither_each_face_into_cells() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Sandstone);
        // A placer-style mix: 6 sandstone + 2 gold-dust.
        let mut mats = [MaterialId::SANDSTONE; 8];
        mats[6] = MaterialId::GOLD_DUST;
        mats[7] = MaterialId::GOLD_DUST;
        let grid = grid_with(5, 5, 5, VoxelContents::debris_only(&mats).unwrap());
        let mesh = mesh_chunk(
            &chunk,
            ChunkPos::new(0, 0, 0),
            0.9,
            &no_neighbors,
            Some(&grid),
        );
        // Six faces, each DITHER_CELLS² quads → 2 triangles per quad.
        let cells = (DITHER_CELLS * DITHER_CELLS) as usize;
        assert_eq!(mesh.triangle_count(), 6 * cells * 2);
        // Both constituents' albedos appear; gold is the sparse minority.
        let gold = material_color(MaterialId::GOLD_DUST);
        let sand = material_color(MaterialId::SANDSTONE);
        let gold_cells = mesh.colors.iter().filter(|c| **c == gold).count();
        let sand_cells = mesh.colors.iter().filter(|c| **c == sand).count();
        assert!(gold_cells > 0, "gold-dust must appear");
        assert!(
            sand_cells > gold_cells,
            "6:2 mix must keep the ore the minority (sand {sand_cells}, gold {gold_cells})"
        );
        // No blending: every vertex color is exactly one of the two albedos.
        assert!(mesh.colors.iter().all(|c| *c == gold || *c == sand));
    }

    #[test]
    fn dither_is_world_anchored_and_deterministic() {
        // Same world coords → identical cell picks, every call.
        let c = {
            let mut m = [MaterialId::SANDSTONE; 8];
            m[7] = MaterialId::GOLD_DUST;
            VoxelContents::debris_only(&m).unwrap()
        };
        for face in 0..6 {
            for (i, j) in [(0, 0), (1, 2), (3, 3)] {
                let a = dither_pick(&c, 10, -7, 42, face, i, j);
                let b = dither_pick(&c, 10, -7, 42, face, i, j);
                assert_eq!(a, b, "pick must be a pure function of its inputs");
            }
        }
        // Different world positions with identical contents dither differently
        // (world-anchored, not per-voxel-local): compare the full cell pattern.
        let pattern = |wx: i64, wy: i64, wz: i64| -> Vec<MaterialId> {
            let mut out = Vec::new();
            for face in 0..6 {
                for i in 0..DITHER_CELLS {
                    for j in 0..DITHER_CELLS {
                        out.push(dither_pick(&c, wx, wy, wz, face, i, j));
                    }
                }
            }
            out
        };
        assert_ne!(
            pattern(0, 0, 0),
            pattern(1, 0, 0),
            "shifting x must change the dither"
        );
        assert_eq!(pattern(5, 5, 5), pattern(5, 5, 5));
    }

    #[test]
    fn dither_weighting_tracks_eighths_fraction() {
        // A 6:2 mix over a large sample lands near the 0.75 majority fraction.
        let mut m = [MaterialId::SANDSTONE; 8];
        m[6] = MaterialId::GOLD_DUST;
        m[7] = MaterialId::GOLD_DUST;
        let c = VoxelContents::debris_only(&m).unwrap();
        let (mut sand, mut total) = (0usize, 0usize);
        for wx in 0..40i64 {
            for wz in 0..40i64 {
                for face in 0..6 {
                    for i in 0..DITHER_CELLS {
                        for j in 0..DITHER_CELLS {
                            if dither_pick(&c, wx, 3, wz, face, i, j) == MaterialId::SANDSTONE {
                                sand += 1;
                            }
                            total += 1;
                        }
                    }
                }
            }
        }
        let frac = sand as f64 / total as f64;
        assert!(
            (frac - 0.75).abs() < 0.02,
            "sand fraction {frac} should track 6/8"
        );
    }

    /// Perf reference (ROADMAP 3c-2 deliverable): triangle counts and mesh time
    /// for a mixed-heavy chunk vs a uniform one. No hard threshold (machine
    /// dependent); prints with `--nocapture`.
    #[test]
    fn perf_mixed_heavy_vs_uniform() {
        use std::time::Instant;
        // A full 32³ Sandstone chunk under open sky: only the outer shell faces
        // are exposed (6·32² = 6144 faces) — a realistic worst case for a
        // fully-mixed band.
        let mut chunk = Chunk::new();
        for z in 0..CHUNK_SIZE_USIZE {
            for y in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    chunk.set(x, y, z, Block::Sandstone);
                }
            }
        }
        let uniform = VoxelContents::debris_only(&[MaterialId::SANDSTONE; 8]).unwrap();
        let mixed = {
            let mut m = [MaterialId::SANDSTONE; 8];
            m[6] = MaterialId::GOLD_DUST;
            m[7] = MaterialId::GOLD_DUST;
            VoxelContents::debris_only(&m).unwrap()
        };
        let uni_grid = ContentsGrid::from_dense(&vec![uniform; CHUNK_VOLUME]);
        let mix_grid = ContentsGrid::from_dense(&vec![mixed; CHUNK_VOLUME]);
        let pos = ChunkPos::new(0, 0, 0);

        let bench = |grid: Option<&ContentsGrid>| -> (usize, f64) {
            let iters = 100;
            // Warm + measure.
            let m = mesh_chunk(&chunk, pos, 0.9, &no_neighbors, grid);
            let start = Instant::now();
            for _ in 0..iters {
                let _ = mesh_chunk(&chunk, pos, 0.9, &no_neighbors, grid);
            }
            (
                m.triangle_count(),
                start.elapsed().as_secs_f64() * 1e3 / iters as f64,
            )
        };
        let (bo_tris, bo_ms) = bench(None);
        let (uni_tris, uni_ms) = bench(Some(&uni_grid));
        let (mix_tris, mix_ms) = bench(Some(&mix_grid));
        println!(
            "mesh 32³ full-solid shell @N=2: block-only {bo_tris} tris {bo_ms:.3} ms | \
             uniform-contents {uni_tris} tris {uni_ms:.3} ms | \
             mixed-heavy(4×4 dither) {mix_tris} tris {mix_ms:.3} ms"
        );
        // Uniform contents take the fast path (identical to block-only); mixed
        // multiplies each exposed face by DITHER_CELLS².
        assert_eq!(uni_tris, bo_tris);
        assert_eq!(mix_tris, bo_tris * (DITHER_CELLS * DITHER_CELLS) as usize);
    }

    #[test]
    fn loose_only_contents_render_partial_height_with_an_exposed_top() {
        // A 4/8 loose voxel under open sky: half-height box, and its top face is
        // emitted even though it is a solid block in the grid.
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Sandstone);
        let loose = VoxelContents::debris_only(&[MaterialId::SANDSTONE; 4]).unwrap();
        let grid = grid_with(5, 5, 5, loose);
        let mesh = mesh_chunk(
            &chunk,
            ChunkPos::new(0, 0, 0),
            1.0,
            &no_neighbors,
            Some(&grid),
        );
        // Uniform, so single-color faces; six of them, all present.
        assert_eq!(mesh.triangle_count(), 12);
        // Every vertex y is either the cell floor (5.0) or the half-height top
        // (5.5) — the box only rises to 4/8.
        for p in &mesh.positions {
            assert!(
                (p[1] - 5.0).abs() < 1e-6 || (p[1] - 5.5).abs() < 1e-6,
                "partial-height vertex y = {} not in {{5.0, 5.5}}",
                p[1]
            );
        }
        // The top face exists even with a solid voxel directly above.
        let mut buried = chunk.clone();
        buried.set(5, 6, 5, Block::Stone);
        let capped = mesh_chunk(
            &buried,
            ChunkPos::new(0, 0, 0),
            1.0,
            &no_neighbors,
            Some(&grid),
        );
        assert!(
            capped.positions.iter().any(|p| (p[1] - 5.5).abs() < 1e-6),
            "a partial voxel must still show its top under a solid neighbor"
        );
    }
}
