//! Top-down far-node synthesis — the second side of the octree node
//! contract's two-sided derivation rule (docs/design/octree-substrate.md § 3).
//!
//! Most far-field nodes cover terrain that has never been generated and never
//! will be (corrections #31: ungenerated ground is the default case). Where
//! the reduction pyramids have no children to reduce, a node **synthesizes**
//! from the worldgen coarse authority: [`WorldGenerator::coarse_surface`] —
//! the same elevation lattice + river carving + surface rule the near ground
//! collapses from, and the same source FF2a's tile streamer samples per
//! column (journal/0022/0023). This module recasts that per-column answer in
//! **node form**: a level-L block chunk whose columns are solid up to the
//! FF2a floor-quantized surface ([`dc_core::farfield::quantize_top`] — ONE
//! quantization, shared with the streamer, S-7).
//!
//! **Seeded** (contract rule): a pure function of (world seed via the
//! generator's pregen, node coords). No wall clock, no ambient entropy.
//!
//! **What synthesis cannot express** (declared, not faked): the coarse
//! authority is a surface summary — it has no overhang/cave data and no
//! sub-surface strata summary, so a synthesized column is one span carrying
//! the *surface* block all the way down (exactly what FF2a rendered: side
//! faces of far steps already wore the surface block). Sub-surface material
//! synthesis is the filed home of `collapse.rs::surface_sample`'s
//! summarization half (a separate sequenced slice); overhang synthesis waits
//! on an authority that records one. See docs/design/stubs.md.
//!
//! **The acceptance test is agreement** (S-3/S-7): where the two derivation
//! directions meet — a synthesized node vs the reduction of its
//! later-generated children — they agree statistically (an expectation, not
//! an exact value: reduction's majority vote rounds the surface cell to
//! nearest while synthesis floors it, a deliberate ≤1-coarse-voxel bias that
//! keeps the far top under the near surface). Proven below in
//! `synthesized_node_agrees_with_reduced_children_statistically`.

use dc_core::farfield::{level_stride, quantize_top};
use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos, PalettedChunk};

use crate::collapse::WorldGenerator;

impl WorldGenerator<'_> {
    /// Synthesize the block payload of a far node (level-L chunk at `pos`)
    /// from the coarse worldgen authority, top-down: each plan column is solid
    /// below its FF2a floor-quantized surface, carrying the surface block.
    ///
    /// Level voxel row `vy` covers base rows `[vy * 2^L, (vy+1) * 2^L)`; a
    /// column with quantized surface `q` (base voxels) is solid at rows
    /// `vy < q / 2^L` — meshing the result places the top face plane at
    /// exactly `q`, byte-identical to what the FF2a streamer draws for the
    /// same column (proven in `synthesized_node_tops_equal_the_ff2a_surface`).
    ///
    /// Deterministic in (pregen seed, level, pos); memoized only through the
    /// generator's existing coarse-surface caches.
    pub fn synthesize_far_node(&mut self, level: u8, pos: ChunkPos) -> PalettedChunk {
        let n = CHUNK_SIZE_USIZE;
        let stride = level_stride(level);
        let base_row = i64::from(pos.y) * n as i64;
        let mut dense = Chunk::new();
        let mut any = false;
        for z in 0..n {
            for x in 0..n {
                let wx = (i64::from(pos.x) * n as i64 + x as i64) * stride;
                let wz = (i64::from(pos.z) * n as i64 + z as i64) * stride;
                let (h, block) = self.coarse_surface(wx, wz);
                let q_row = i64::from(quantize_top(h, stride)) / stride;
                // Solid level rows: base_row + y < q_row.
                let top_local = (q_row - base_row).clamp(0, n as i64) as usize;
                for y in 0..top_local {
                    dense.set(x, y, z, block);
                }
                any |= top_local > 0;
            }
        }
        if !any {
            return PalettedChunk::uniform(Block::Air);
        }
        PalettedChunk::from_dense(&dense)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pregen::{Extent, Pregen, WorldParams};
    use dc_core::lod::{LodPyramid, child_positions};
    use dc_core::{MAX_LOD_LEVEL, farfield::node_column_spans};

    fn medium_world() -> Pregen {
        Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        })
    }

    /// Exact register (S-3: agreement is exact where derivation is
    /// deterministic): a synthesized node's per-column top face plane equals
    /// the FF2a streamer's answer — `quantize_top(coarse_surface)` — clamped
    /// to the node's extent. Same function, same lattice, no drift possible;
    /// this pins the "node form" recast to the shipped tile path.
    #[test]
    fn synthesized_node_tops_equal_the_ff2a_surface() {
        let pregen = medium_world();
        let mut g = WorldGenerator::new(&pregen);
        for level in 1..=3u8 {
            let stride = level_stride(level);
            // A node straddling the surface at the world origin.
            let (h0, _) = g.coarse_surface(0, 0);
            let ny = (i64::from(h0) / stride / 32) as i32;
            let pos = ChunkPos::new(0, ny, 0);
            let chunk = g.synthesize_far_node(level, pos);
            let spans = node_column_spans(level, pos, &chunk, None);
            for z in 0..4usize {
                for x in 0..4usize {
                    let wx = (i64::from(pos.x) * 32 + x as i64) * stride;
                    let wz = (i64::from(pos.z) * 32 + z as i64) * stride;
                    let (h, block) = g.coarse_surface(wx, wz);
                    let q = quantize_top(h, stride);
                    let col = &spans[z * 32 + x];
                    let node_floor = pos.y * 32 * stride as i32;
                    let node_ceil = (pos.y + 1) * 32 * stride as i32;
                    let expected = q.clamp(node_floor, node_ceil);
                    let got = col.first().map_or(node_floor, |s| s.top);
                    assert_eq!(got, expected, "L{level} column ({x},{z})");
                    if let Some(s) = col.first()
                        && s.top < node_ceil
                    {
                        assert_eq!(s.block, block, "surface block carries the span");
                    }
                }
            }
        }
    }

    /// Seeded synthesis: byte-identical across generator instances over the
    /// same pregen (pure function of seed + node coords; no ambient entropy).
    #[test]
    fn synthesis_is_deterministic() {
        let pregen = medium_world();
        let pos = ChunkPos::new(3, 34, -2);
        let a = WorldGenerator::new(&pregen).synthesize_far_node(2, pos);
        let b = WorldGenerator::new(&pregen).synthesize_far_node(2, pos);
        assert_eq!(a, b);
    }

    /// **The contract's acceptance test** (octree-substrate.md § 3: "the two
    /// directions must agree where they meet"): synthesize a node, then
    /// actually generate its full-resolution children, reduce them up the S3
    /// pyramid (`MajorityNonAir`), and compare per-column top face planes in
    /// coarse voxels.
    ///
    /// The register is statistical (S-7): synthesis FLOORS the surface into
    /// the coarse lattice (the near-parity guarantee) while the majority vote
    /// rounds each 2×2×2 cell to nearest, so reduction sits 0..1 coarse voxel
    /// ABOVE synthesis on average — a known, deliberate bias, never exceeding
    /// one coarse voxel of quantization plus sub-cell relief (the corner
    /// sample vs the cell's own columns). Tolerances, justified:
    /// - mean(reduced − synth) in [0, 1] coarse voxels: the floor-vs-round
    ///   offset is bounded by one quantization step and can never be negative
    ///   on average (flooring only lowers).
    /// - ≥ 90 % of columns within 1 coarse voxel, ≥ 99 % within 2: beyond the
    ///   quantization step, disagreement comes from relief inside the coarse
    ///   cell (corner sample vs majority) and from placed trees entering the
    ///   vote; both are sub-cell effects, so multi-voxel disagreement must be
    ///   rare or the two sides are not sampling the same world.
    #[test]
    fn synthesized_node_agrees_with_reduced_children_statistically() {
        let pregen = medium_world();
        let mut g = WorldGenerator::new(&pregen);

        let mut n_cols = 0usize;
        let mut sum_diff = 0i64;
        let mut within1 = 0usize;
        let mut within2 = 0usize;
        let mut max_abs = 0i64;

        // A spread of surface nodes at L1 and L2 across the Medium world.
        for level in 1..=2u8 {
            let stride = level_stride(level);
            for (px, pz) in [(0i32, 0i32), (5, -3), (-4, 7), (9, 9)] {
                // Node straddling the surface at its plan corner.
                let (h, _) =
                    g.coarse_surface(i64::from(px) * 32 * stride, i64::from(pz) * 32 * stride);
                let ny = (i64::from(h) / stride / 32) as i32;
                let pos = ChunkPos::new(px, ny, pz);
                let synth = g.synthesize_far_node(level, pos);

                // Generate every full-res child and reduce upward — the other
                // side of the contract, on the real machinery.
                let mut pyramid = LodPyramid::new(MAX_LOD_LEVEL, Box::new(dc_core::MajorityNonAir));
                let mut frontier = vec![pos];
                for _ in 0..level {
                    frontier = frontier.iter().flat_map(|p| child_positions(*p)).collect();
                }
                for l0 in frontier {
                    let chunk = g.generate_chunk(l0);
                    pyramid.insert_chunk(l0, PalettedChunk::from_dense(&chunk));
                }
                assert!(
                    pyramid.subtree_fully_inserted(level, pos),
                    "test setup: the subtree must be fully generated"
                );
                let reduced = pyramid
                    .get_or_derive(level, pos)
                    .expect("fully inserted subtree derives")
                    .clone();

                // Per-column top face plane, clamped to the node extent so
                // empty/full columns compare as the boundary they clamp to.
                let face_row = |chunk: &PalettedChunk, x: usize, z: usize| -> i64 {
                    for y in (0..32usize).rev() {
                        if chunk.get(x, y, z).is_solid() {
                            return y as i64 + 1;
                        }
                    }
                    0
                };
                for z in 0..32usize {
                    for x in 0..32usize {
                        let s = face_row(&synth, x, z);
                        let r = face_row(&reduced, x, z);
                        // Ignore columns clamped at the same boundary by both.
                        if s == r && (s == 0 || s == 32) {
                            continue;
                        }
                        let d = r - s; // coarse voxels; + = reduction higher
                        n_cols += 1;
                        sum_diff += d;
                        if d.abs() <= 1 {
                            within1 += 1;
                        }
                        if d.abs() <= 2 {
                            within2 += 1;
                        }
                        max_abs = max_abs.max(d.abs());
                    }
                }
            }
        }

        let mean = sum_diff as f64 / n_cols as f64;
        let f1 = within1 as f64 / n_cols as f64;
        let f2 = within2 as f64 / n_cols as f64;
        println!(
            "agreement: n={n_cols} mean(reduced-synth)={mean:.3} coarse voxels, \
             |d|<=1: {:.1}%, |d|<=2: {:.1}%, max|d|={max_abs}",
            f1 * 100.0,
            f2 * 100.0
        );
        assert!(n_cols > 4000, "the sample must actually exercise the seam");
        assert!(
            (0.0..=1.0).contains(&mean),
            "mean offset {mean:.3} outside the floor-vs-round band [0, 1]"
        );
        assert!(
            f1 >= 0.90,
            "only {:.1}% of columns within 1 coarse voxel",
            f1 * 100.0
        );
        assert!(
            f2 >= 0.99,
            "only {:.1}% of columns within 2 coarse voxels",
            f2 * 100.0
        );
    }
}
