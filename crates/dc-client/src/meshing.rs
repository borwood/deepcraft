//! Culled chunk meshing: one quad per solid face whose neighbor is not solid.
//!
//! Deliberately the simple mesher — S1's exit criterion accepts culled meshing;
//! greedy merging is an optimization for later. Output is plain vertex arrays
//! (no renderer types) so the headless bench can run it without a GPU.
//!
//! Vertices are **chunk-local meters**: local voxel coordinates scaled by the
//! voxel size. The chunk entity's transform places the chunk relative to the
//! floating origin, keeping every f32 the GPU sees small.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos};

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
        // basalt near-black.
        (Block::Mudstone, _) => [0.46, 0.26, 0.20, 1.0],
        (Block::Sandstone, _) => [0.76, 0.66, 0.44, 1.0],
        (Block::Granite, _) => [0.66, 0.56, 0.58, 1.0],
        (Block::Basalt, _) => [0.14, 0.14, 0.16, 1.0],
        (Block::Air, _) => [1.0, 0.0, 1.0, 1.0], // never emitted
    }
}

/// Mesh one chunk with culled faces. `neighbor_solid` is consulted (with
/// world-space voxel coordinates) only for the one-voxel shell outside the
/// chunk, so chunk borders cull correctly against neighbors.
pub fn mesh_chunk(
    chunk: &Chunk,
    pos: ChunkPos,
    voxel_size_m: f32,
    neighbor_solid: &dyn Fn(i64, i64, i64) -> bool,
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
                for (normal, corners) in &FACES {
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
                    if covered {
                        continue;
                    }
                    emit_face(&mut mesh, x, y, z, normal, corners, voxel_size_m, block);
                }
            }
        }
    }
    mesh
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
    block: Block,
) {
    let base = mesh.positions.len() as u32;
    let n = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    let color = face_color(block, normal[1]);
    for c in corners {
        mesh.positions.push([
            (x as f32 + c[0]) * voxel_size_m,
            (y as f32 + c[1]) * voxel_size_m,
            (z as f32 + c[2]) * voxel_size_m,
        ]);
        mesh.normals.push(n);
        mesh.colors.push(color);
    }
    mesh.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::CHUNK_VOLUME;

    fn no_neighbors(_: i64, _: i64, _: i64) -> bool {
        false
    }

    #[test]
    fn single_voxel_emits_six_faces() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Stone);
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 0.5, &no_neighbors);
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
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 1.0, &no_neighbors);
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
        let mesh = mesh_chunk(&chunk, ChunkPos::new(2, -3, 1), 1.0, &everything_solid);
        assert!(mesh.is_empty(), "buried chunk must mesh to nothing");

        // Same chunk with air neighbors: exactly the 6 outer faces' quads.
        let mesh = mesh_chunk(&chunk, ChunkPos::new(2, -3, 1), 1.0, &no_neighbors);
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
        let mesh = mesh_chunk(&chunk, pos, 1.0, &neighbor);
        // -X face culled by the neighbor; 5 faces remain.
        assert_eq!(mesh.triangle_count(), 10);
    }
}
