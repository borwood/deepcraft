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
//! ## Material / mixture model (ROADMAP PBR-1, docs/design/visuals.md)
//!
//! Every emitted face carries, per vertex:
//!
//! - a **world-anchored UV** in tile units (one texture tile per voxel, but the
//!   tile *origin* is the world voxel coordinate rather than a per-face `[0, 1]`
//!   reset — so the pattern is continuous across adjacent same-material voxels
//!   and a greedy quad spanning K voxels tiles K times). The old per-face
//!   `[0, 1]` window made every block present the *identical* tile image; with a
//!   directional placeholder texture that read as a legible per-block grid
//!   (journal/0020). The shader `fract`s the UV to sample; its integer part also
//!   gives the fullbright variant a **world-anchored 4×4 cell grid** for the
//!   mixture speckle with no extra vertex attribute;
//! - up to [`SPLAT_N`] **material atlas layers** + normalized **splat weights**
//!   — the LabPBR terrain material (terrain_material.rs) blends the layers with
//!   height/AO-driven contrast (heightlerp) so grains poke through instead of
//!   alpha-mushing (visuals.md § Material/texture model);
//! - a representative **vertex color** — the dominant constituent's registry
//!   albedo — kept so `--fullbright` renders pure unlit vertex color exactly as
//!   the walk protocol needs (journal/0004), and so a broken/absent texture path
//!   degrades to a colored surface rather than nothing.
//!
//! **Uniform** voxels resolve to a single material layer at weight 1 — so a
//! uniform-contents voxel finally samples its MATERIAL pack, not its block's
//! (siltstone renders differently from mudstone; this kills the walk-10
//! "member identity is render-invisible" item). **Mixed** voxels emit a single
//! quad carrying the top-N constituents by eighths fraction (the world-anchored
//! [`voxel_seed`] hash selects which N when a voxel holds more than N distinct
//! grains — it never does in the current world, but the rule is exercised by a
//! test). This replaces the interim 4×4 dither, which multiplied a mixed face
//! into 16 quads (journal/0010): mixed faces are single quads again.
//!
//! Block-colored faces (no per-voxel contents — the legacy S1 terrain, the far
//! field, benches) resolve to a single **block** atlas layer ([`block_layer`])
//! at weight 1, so the whole world is textured under one material.
//!
//! Loose-only contents (debris role, no structure) still render as
//! **partial-height** boxes (height = eighths / 8, snow-layer style); the
//! block-tier collider stays binary (the accepted, documented visible mismatch
//! — visuals.md). Worldgen does not yet emit sub-8 loose voxels, so partial
//! heights are exercised by tests until loose-material deposition lands.

use dc_core::{
    Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos, ContentsGrid, MATERIAL_COUNT, MaterialId,
    StructureShape, VoxelContents,
};

/// Max material layers blended per face (the heightlerp splat count). Four is
/// the classic splat ceiling; real voxels hold ≤ 3 distinct grains today, so
/// the over-N selection path is test-only headroom, not a live cost.
pub const SPLAT_N: usize = 4;

/// Block-only pack slugs, in the order they occupy atlas layers *after* the
/// `MATERIAL_COUNT` material layers. Consumed by the atlas loader and by
/// [`block_layer`]; keep the two in lockstep.
pub const BLOCK_ONLY_SLUGS: [&str; 4] = ["grass", "dirt", "stone", "wood"];

/// Total layers in the terrain texture atlas: one per registry [`MaterialId`],
/// then the block-only packs that have no material twin.
pub const ATLAS_LAYER_COUNT: u32 = MATERIAL_COUNT as u32 + BLOCK_ONLY_SLUGS.len() as u32;

/// CPU-side mesh: positions/normals/colors/uvs + per-vertex splat material data
/// plus triangle indices. All plain arrays (no renderer types) so the headless
/// bench meshes without a GPU.
#[derive(Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Representative albedo (dominant constituent / block face color). The
    /// `--fullbright` path renders this directly; the lit path uses it only as
    /// a tint fallback.
    pub colors: Vec<[f32; 4]>,
    /// Per-face texture coordinates, `[0, 1]²`.
    pub uvs: Vec<[f32; 2]>,
    /// Up to [`SPLAT_N`] atlas layer indices per vertex (unused slots repeat
    /// layer 0 with weight 0).
    pub mat_layers: Vec<[u32; 4]>,
    /// Normalized splat weights matching `mat_layers` (sum ≈ 1).
    pub mat_weights: Vec<[f32; 4]>,
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

/// World-anchored UV for one face corner (in tile units, one tile per voxel).
///
/// The tile origin is the world voxel coordinate, not a per-face `[0, 1]` reset:
/// the two in-plane world axes become U and V, so the pattern flows continuously
/// across adjacent same-material voxels (and a K-voxel greedy quad would tile
/// K times). At a corner the UV is integer-valued; the shader `fract`s it to
/// sample the tile, and `floor(uv * 4)` gives the fullbright speckle its
/// world-anchored 4×4 cell grid. `cy` is the corner's *geometric* height (the
/// partial-height fraction for a loose voxel's top), so a squished voxel keeps a
/// consistent world-anchored V.
#[inline]
fn corner_world_uv(normal: &[i64; 3], c: &[f32; 3], wx: i64, wy: i64, wz: i64) -> [f32; 2] {
    let (wx, wy, wz) = (wx as f32, wy as f32, wz as f32);
    if normal[0] != 0 {
        // ±X face: U = world Z, V = world Y.
        [wz + c[2], wy + c[1]]
    } else if normal[1] != 0 {
        // ±Y face: U = world X, V = world Z.
        [wx + c[0], wz + c[2]]
    } else {
        // ±Z face: U = world X, V = world Y.
        [wx + c[0], wy + c[1]]
    }
}

/// Albedo per block and face. Grass gets a green top and earthy sides so the
/// surface reads at a glance; the geology block colors match the material
/// registry albedos, so a uniform geology voxel reads identically whether
/// colored by block or by contents. Used for the vertex color (fullbright) on
/// block-colored faces, and by the far-field heightfield (farmesh.rs) for its
/// top-surface vertex color.
pub(crate) fn face_color(block: Block, normal_y: i64) -> [f32; 4] {
    match (block, normal_y) {
        (Block::Grass, 1) => [0.30, 0.62, 0.25, 1.0],
        (Block::Grass, _) => [0.38, 0.45, 0.22, 1.0],
        (Block::Dirt, _) => [0.42, 0.30, 0.19, 1.0],
        (Block::Stone, _) => [0.52, 0.52, 0.54, 1.0],
        (Block::Wood, _) => [0.44, 0.33, 0.17, 1.0],
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

/// The atlas layer a material samples: its registry id (materials occupy the
/// first `MATERIAL_COUNT` layers).
#[inline]
pub fn material_layer(m: MaterialId) -> u32 {
    u32::from(m.raw())
}

/// The atlas layer a block-colored face samples. Geology blocks share their
/// material pack (so a geology block with no per-voxel contents still reads as
/// its rock); the non-geology blocks map into the appended block-only layers,
/// ordered as [`BLOCK_ONLY_SLUGS`].
#[inline]
pub fn block_layer(block: Block) -> u32 {
    let base = MATERIAL_COUNT as u32;
    match block {
        Block::Grass => base, // BLOCK_ONLY_SLUGS[0]
        Block::Dirt => base + 1,
        Block::Stone => base + 2,
        Block::Wood => base + 3,
        Block::Mudstone => material_layer(MaterialId::MUDSTONE),
        Block::Sandstone => material_layer(MaterialId::SANDSTONE),
        Block::Granite => material_layer(MaterialId::GRANITE),
        Block::Basalt => material_layer(MaterialId::BASALT),
        Block::Air => 0, // never emitted
    }
}

/// The blocks whose voxels carry material contents (the recorded strata tier).
/// Gating contents consumption on the block keeps stale render-only contents
/// from bleeding through after an edit changes the block (edits do not yet
/// touch materials): a voxel edited to any other block falls back to its block
/// color/layer and full height.
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
    c.shape() == StructureShape::None && !c.debris().is_empty()
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

/// A voxel holds at most 8 eighths, so at most 8 distinct constituents.
type ConstituentBuf = [(MaterialId, u32); 8];

/// Distinct constituents of a voxel with their eighth counts, written into a
/// fixed stack buffer (no heap on the mesher's hot path); returns the count.
/// First-appearance order over the sorted `filled_slots`.
fn constituents_into(c: &VoxelContents, buf: &mut ConstituentBuf) -> usize {
    let mut len = 0usize;
    'next: for &m in c.filled_slots() {
        for e in buf[..len].iter_mut() {
            if e.0 == m {
                e.1 += 1;
                continue 'next;
            }
        }
        buf[len] = (m, 1);
        len += 1;
    }
    len
}

/// The dominant constituent (max eighths, ties broken by lowest id) — the
/// vertex color source for a mixed/uniform face.
fn dominant_material(c: &VoxelContents) -> MaterialId {
    let mut buf: ConstituentBuf = [(MaterialId::SAND, 0); 8];
    let len = constituents_into(c, &mut buf);
    buf[..len]
        .iter()
        .copied()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map_or(MaterialId::SAND, |(m, _)| m)
}

/// Deterministic, world-anchored per-voxel seed: a pure function of world voxel
/// coordinates. A given world position selects the same constituents across
/// reloads, generation orders, and runs.
#[inline]
pub fn voxel_seed(wx: i64, wy: i64, wz: i64) -> u64 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    for v in [wx as u64, wy as u64, wz as u64] {
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

/// Choose up to [`SPLAT_N`] constituents and return `(atlas layers, weights)`,
/// weights normalized to sum ≈ 1. When a voxel holds more distinct grains than
/// `SPLAT_N`, the world-anchored `seed` drives a fraction-weighted selection
/// without replacement — rarer grains still occasionally surface, exactly as
/// the interim per-cell dither weighted its pick (journal/0010). Constituents
/// are ordered dominant-first (weight desc, id asc), so slot 0 is the majority
/// grain.
pub fn top_splat(c: &VoxelContents, seed: u64) -> ([u32; SPLAT_N], [f32; SPLAT_N]) {
    let mut buf: ConstituentBuf = [(MaterialId::SAND, 0); 8];
    let len = constituents_into(c, &mut buf);

    // Reduce to at most SPLAT_N constituents, in place (the over-N selection is
    // the rare path — never reached in the current world).
    let chosen: &[(MaterialId, u32)] = if len <= SPLAT_N {
        &buf[..len]
    } else {
        let picked = weighted_select(&buf[..len], seed, SPLAT_N);
        buf[..SPLAT_N].copy_from_slice(&picked);
        &buf[..SPLAT_N]
    };

    // Dominant-first (weight desc, id asc): slot 0 is the majority grain.
    let mut sorted: [(MaterialId, u32); SPLAT_N] = [(MaterialId::SAND, 0); SPLAT_N];
    sorted[..chosen.len()].copy_from_slice(chosen);
    sorted[..chosen.len()].sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let total: u32 = sorted[..chosen.len()].iter().map(|(_, n)| *n).sum();

    let mut layers = [0u32; SPLAT_N];
    let mut weights = [0f32; SPLAT_N];
    if total > 0 {
        for (i, (m, n)) in sorted[..chosen.len()].iter().enumerate() {
            layers[i] = material_layer(*m);
            weights[i] = *n as f32 / total as f32;
        }
    } else {
        weights[0] = 1.0;
    }
    (layers, weights)
}

/// Fraction-weighted selection of `n` distinct constituents without
/// replacement, seeded by the world-anchored hash. Only reached when a voxel
/// holds more than `SPLAT_N` distinct grains (never in the current world).
fn weighted_select(cons: &[(MaterialId, u32)], seed: u64, n: usize) -> Vec<(MaterialId, u32)> {
    let mut pool: Vec<(MaterialId, u32)> = cons.to_vec();
    let mut state = seed | 1;
    let mut next = || {
        // xorshift64*, deterministic
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut out = Vec::with_capacity(n);
    while out.len() < n && !pool.is_empty() {
        let total: u32 = pool.iter().map(|(_, w)| *w).sum();
        let mut pick = (next() % u64::from(total)) as u32;
        let mut idx = 0;
        for (i, (_, w)) in pool.iter().enumerate() {
            if pick < *w {
                idx = i;
                break;
            }
            pick -= *w;
        }
        out.push(pool.remove(idx));
    }
    out
}

/// Mesh one chunk with culled faces. `neighbor_solid` is consulted (with
/// world-space voxel coordinates) only for the one-voxel shell outside the
/// chunk, so chunk borders cull correctly against neighbors. `contents`, when
/// present, is the chunk's render-only per-voxel material view (worldgen
/// authority); passing `None` is the plain block-layered mesher (S1 terrain, far
/// field, benches).
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
                let (wx, wy, wz) = (mx + x as i64, my + y as i64, mz + z as i64);
                let frac = voxel.as_ref().map_or(1.0, height_frac);

                // Per-voxel splat: uniform → one layer; mixed → top-N. All six
                // faces of a voxel share it (coherent across the voxel).
                let (layers, weights, contents_color) = match voxel.as_ref() {
                    Some(c) => {
                        let (l, w) = top_splat(c, voxel_seed(wx, wy, wz));
                        (l, w, Some(material_color(dominant_material(c))))
                    }
                    None => ([block_layer(block), 0, 0, 0], [1.0, 0.0, 0.0, 0.0], None),
                };

                for (normal, corners) in FACES.iter() {
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
                    let color = contents_color.unwrap_or_else(|| face_color(block, normal[1]));
                    emit_face(
                        &mut mesh,
                        (x, y, z),
                        (wx, wy, wz),
                        normal,
                        corners,
                        voxel_size_m,
                        frac,
                        color,
                        layers,
                        weights,
                    );
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
    local: (usize, usize, usize),
    world: (i64, i64, i64),
    normal: &[i64; 3],
    corners: &[[f32; 3]; 4],
    voxel_size_m: f32,
    frac: f32,
    color: [f32; 4],
    layers: [u32; SPLAT_N],
    weights: [f32; SPLAT_N],
) {
    let (x, y, z) = local;
    let (wx, wy, wz) = world;
    let base = mesh.positions.len() as u32;
    let n = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    for corner in corners.iter() {
        // Collapse the voxel top to its render height (partial-height boxes) so
        // geometry and the world-anchored V stay consistent.
        let geo = [
            corner[0],
            if corner[1] >= 1.0 { frac } else { corner[1] },
            corner[2],
        ];
        mesh.positions.push([
            (x as f32 + geo[0]) * voxel_size_m,
            (y as f32 + geo[1]) * voxel_size_m,
            (z as f32 + geo[2]) * voxel_size_m,
        ]);
        mesh.normals.push(n);
        mesh.colors.push(color);
        mesh.uvs.push(corner_world_uv(normal, &geo, wx, wy, wz));
        mesh.mat_layers.push(layers);
        mesh.mat_weights.push(weights);
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

    /// A one-voxel `ContentsGrid` placing `c` at (x,y,z), everything else empty.
    fn grid_with(x: usize, y: usize, z: usize, c: VoxelContents) -> ContentsGrid {
        let mut dense = vec![VoxelContents::EMPTY; CHUNK_VOLUME];
        dense[Chunk::index(x, y, z)] = c;
        ContentsGrid::from_dense(&dense)
    }

    #[test]
    fn single_voxel_emits_six_faces_with_uvs_and_layers() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Stone);
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 0.5, &no_neighbors, None);
        assert_eq!(mesh.triangle_count(), 12);
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.uvs.len(), 24);
        assert_eq!(mesh.mat_layers.len(), 24);
        assert_eq!(mesh.mat_weights.len(), 24);
        for l in &mesh.mat_layers {
            assert_eq!(l[0], block_layer(Block::Stone));
        }
        for w in &mesh.mat_weights {
            assert_eq!(*w, [1.0, 0.0, 0.0, 0.0]);
        }
        // UVs are world-anchored (tile origin = world voxel coord), so the
        // voxel at world (5,5,5) tiles from 5..6, not a per-face 0..1 window.
        assert!(mesh.uvs.contains(&[5.0, 5.0]));
        assert!(mesh.uvs.contains(&[6.0, 6.0]));
        assert!(
            !mesh.uvs.contains(&[0.0, 0.0]),
            "UVs are not per-face reset"
        );
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

        let mesh = mesh_chunk(&chunk, ChunkPos::new(2, -3, 1), 1.0, &no_neighbors, None);
        let expected_quads = 6 * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;
        assert_eq!(mesh.triangle_count(), expected_quads * 2);
        assert_eq!(CHUNK_VOLUME, 32 * 32 * 32);
    }

    #[test]
    fn border_faces_consult_neighbor_query_in_world_coords() {
        let mut chunk = Chunk::new();
        chunk.set(0, 0, 0, Block::Stone);
        let pos = ChunkPos::new(-1, 0, 0);
        let neighbor = |x: i64, y: i64, z: i64| (x, y, z) == (-33, 0, 0);
        let mesh = mesh_chunk(&chunk, pos, 1.0, &neighbor, None);
        assert_eq!(mesh.triangle_count(), 10);
    }

    #[test]
    fn uvs_are_world_anchored_and_continuous_across_voxels() {
        // Two stone voxels side by side along X. Their +Z faces should carry
        // UVs whose U spans [wx, wx+1] per voxel — so the right edge of the left
        // voxel (U = 6) meets the left edge of the right voxel (U = 6): the tile
        // pattern is continuous across the block boundary, not reset per face.
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Stone);
        chunk.set(6, 5, 5, Block::Stone);
        let mesh = mesh_chunk(&chunk, ChunkPos::new(0, 0, 0), 1.0, &no_neighbors, None);
        // Every UV is an integer at a corner (tile origin = world voxel coord).
        for uv in &mesh.uvs {
            assert_eq!(uv[0], uv[0].round(), "U not integer-anchored: {uv:?}");
            assert_eq!(uv[1], uv[1].round(), "V not integer-anchored: {uv:?}");
        }
        // The shared boundary U = 6 appears (left voxel's max U == right voxel's
        // min U): continuity, not a per-face 0..1 reset.
        let us: std::collections::HashSet<i64> = mesh.uvs.iter().map(|uv| uv[0] as i64).collect();
        assert!(us.contains(&5) && us.contains(&6) && us.contains(&7));
    }

    #[test]
    fn uniform_contents_sample_their_material_layer() {
        // A uniform-contents voxel samples the MATERIAL pack (siltstone), not
        // its block's — the walk-10 "member identity render-invisible" kill.
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, Block::Mudstone);
        let uniform =
            VoxelContents::debris_only(&[MaterialId::SILTSTONE; 8]).expect("8 debris eighths");
        let grid = grid_with(5, 5, 5, uniform);
        let mesh = mesh_chunk(
            &chunk,
            ChunkPos::new(0, 0, 0),
            0.9,
            &no_neighbors,
            Some(&grid),
        );
        // Single quad per face (no dither): 6 faces, 12 tris.
        assert_eq!(mesh.triangle_count(), 12);
        assert_ne!(
            material_layer(MaterialId::SILTSTONE),
            block_layer(Block::Mudstone)
        );
        for l in &mesh.mat_layers {
            assert_eq!(l[0], material_layer(MaterialId::SILTSTONE));
        }
        for w in &mesh.mat_weights {
            assert_eq!(*w, [1.0, 0.0, 0.0, 0.0]);
        }
        for c in &mesh.colors {
            assert_eq!(*c, material_color(MaterialId::SILTSTONE));
        }
    }

    #[test]
    fn mixed_contents_are_a_single_quad_with_splat_weights() {
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
        // Single quad per face — the mosaic's 16× geometry is gone.
        assert_eq!(mesh.triangle_count(), 12);
        for (l, w) in mesh.mat_layers.iter().zip(mesh.mat_weights.iter()) {
            assert_eq!(l[0], material_layer(MaterialId::SANDSTONE));
            assert_eq!(l[1], material_layer(MaterialId::GOLD_DUST));
            assert!((w[0] - 0.75).abs() < 1e-6, "sandstone weight {}", w[0]);
            assert!((w[1] - 0.25).abs() < 1e-6, "gold weight {}", w[1]);
            assert_eq!(w[2], 0.0);
            assert_eq!(w[3], 0.0);
        }
    }

    #[test]
    fn top_splat_is_world_anchored_and_deterministic() {
        let c = {
            let mut m = [MaterialId::SANDSTONE; 8];
            m[7] = MaterialId::GOLD_DUST;
            VoxelContents::debris_only(&m).unwrap()
        };
        let a = top_splat(&c, voxel_seed(10, -7, 42));
        let b = top_splat(&c, voxel_seed(10, -7, 42));
        assert_eq!(a, b);
        let (_, w) = a;
        assert!((w[0] - 7.0 / 8.0).abs() < 1e-6);
        assert!((w[1] - 1.0 / 8.0).abs() < 1e-6);
    }

    #[test]
    fn over_n_constituents_selected_by_seed_deterministically() {
        // Six distinct grains — more than SPLAT_N. Selection is seeded and
        // deterministic, and always returns exactly SPLAT_N of them.
        let c = VoxelContents::debris_only(&[
            MaterialId::SAND,
            MaterialId::GRAVEL,
            MaterialId::SILT,
            MaterialId::CLAY,
            MaterialId::ASH,
            MaterialId::LOAM,
        ])
        .unwrap();
        let mut buf: ConstituentBuf = [(MaterialId::SAND, 0); 8];
        assert!(constituents_into(&c, &mut buf) > SPLAT_N);
        let s = voxel_seed(3, 4, 5);
        let (l1, w1) = top_splat(&c, s);
        let (l2, w2) = top_splat(&c, s);
        assert_eq!((l1, w1), (l2, w2), "seeded selection is deterministic");
        let nz = w1.iter().filter(|w| **w > 0.0).count();
        assert_eq!(nz, SPLAT_N);
        let sum: f32 = w1.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "weights sum to {sum}");
    }

    #[test]
    fn perf_mixed_heavy_vs_uniform_triangle_counts() {
        use std::time::Instant;
        // A full 32³ Sandstone chunk under open sky: only the outer shell faces
        // are exposed (6·32² = 6144 faces).
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
             mixed(splat, single-quad) {mix_tris} tris {mix_ms:.3} ms"
        );
        // PBR-1 deliverable: mixed faces are single quads again — same triangle
        // count as uniform/block-only (the 4×4 dither's 16× is gone).
        assert_eq!(uni_tris, bo_tris);
        assert_eq!(mix_tris, bo_tris);
    }

    #[test]
    fn loose_only_contents_render_partial_height_with_an_exposed_top() {
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
        assert_eq!(mesh.triangle_count(), 12);
        for p in &mesh.positions {
            assert!(
                (p[1] - 5.0).abs() < 1e-6 || (p[1] - 5.5).abs() < 1e-6,
                "partial-height vertex y = {} not in {{5.0, 5.5}}",
                p[1]
            );
        }
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

    #[test]
    fn atlas_layers_are_distinct_and_in_range() {
        // Every material and every block-only pack occupies a distinct atlas
        // layer within ATLAS_LAYER_COUNT (the loader relies on this).
        let mut seen = std::collections::HashSet::new();
        for m in MaterialId::all() {
            let l = material_layer(m);
            assert!(l < ATLAS_LAYER_COUNT);
            assert!(seen.insert(l), "duplicate layer {l}");
        }
        for b in [Block::Grass, Block::Dirt, Block::Stone, Block::Wood] {
            let l = block_layer(b);
            assert!(l < ATLAS_LAYER_COUNT);
            assert!(seen.insert(l), "block layer {l} collides");
        }
        assert_eq!(seen.len(), ATLAS_LAYER_COUNT as usize);
    }
}
