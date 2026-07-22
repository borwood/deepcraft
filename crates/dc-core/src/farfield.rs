//! FF2b far-field node derivation — the render stratum of the octree node
//! contract (docs/design/octree-substrate.md § 3, stratum 1) expressed as
//! per-column span **stacks**: plain data the far mesher consumes.
//!
//! A far column is a stack of [`ColumnSpan`]s — solid vertical runs in world
//! base voxels, topmost first. FF2a (journal/0023) shipped the single-span
//! form in dc-client; FF2b moves the type here (derivation is headless) and
//! adds the `bottom` bound so coarse *volumetric* geometry (chasms, overhangs)
//! has a payload. The two derivation directions of the ratified contract meet
//! in this vocabulary:
//!
//! - **Reduce upward** where children exist: [`node_column_spans`] reads a
//!   level-L node's block chunk (the S3 pyramid, [`crate::lod::MajorityNonAir`])
//!   plus its material chunk ([`crate::materials::lod::MixtureDownsampleRule`])
//!   and emits the node's span stacks. The span block is refined through
//!   [`crate::classify::classify`] where the reduced contents are non-empty, so
//!   the far mesh's material variety flows through the material pyramid — the
//!   consumer spines § 3 row 1 was waiting for.
//! - **Synthesize top-down** where they don't (most of the field, forever):
//!   dc-worldgen's `synthesize_far_node` fills a node from the coarse worldgen
//!   authority using the same [`quantize_top`] the FF2a streamer uses — ONE
//!   quantization, no parallel rule (S-7).
//!
//! [`compose_column`] lays reduced node answers over the synthesized backdrop
//! (the two-sided derivation rule at column granularity). **Ungenerated is not
//! empty** (contract rule / A-5): a consumer may only substitute reduced data
//! where the node's subtree is fully inserted
//! ([`crate::lod::LodPyramid::subtree_fully_inserted`]); everywhere else the
//! synthesized span stands, so missing children never leak into far rendering
//! as air.

use crate::chunk::{CHUNK_SIZE_USIZE, ChunkPos};
use crate::classify::classify;
use crate::materials::intern::{MaterialChunk, MixtureId, MixtureTable};
use crate::palette::PalettedChunk;
use crate::voxel::Block;

/// Sentinel bottom for a span that continues below anything the far field will
/// ever draw ("the ground keeps going"). The mesher never emits a bottom face
/// for it. A plain `i32` rather than `Option` keeps the payload fixed-layout
/// POD (postcard is positional — corrections #3).
pub const FAR_BOTTOM_UNBOUNDED: i32 = i32::MIN;

/// One solid vertical run of a far column, in world **base voxels**: solid
/// over `[bottom, top)`, so `top` is the face plane the top face renders at.
/// `block` drives the atlas layer / fullbright vertex colour for every face
/// the span emits (FF2a's per-span fidelity, unchanged).
///
/// **Persistence-shaped** (journal/0023 amendment 3): fixed-layout POD, no
/// `Option`, no `skip_serializing_if` — a positional wire format can carry it
/// verbatim when far summaries persist beside S3 region files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnSpan {
    /// Top face plane, base voxels — for synthesized spans this is
    /// [`quantize_top`] of the sampled surface (a coarse-lattice boundary at or
    /// below the true surface).
    pub top: i32,
    /// Bottom face plane, base voxels, or [`FAR_BOTTOM_UNBOUNDED`].
    pub bottom: i32,
    /// The block every face of this span renders as.
    pub block: Block,
}

/// Base voxels per coarse voxel at level L (the sampling / quantization
/// stride). Moved from the FF2a streamer (dc-client) — one definition.
#[inline]
pub fn level_stride(level: u8) -> i64 {
    1i64 << level
}

/// Quantize a base-voxel surface height to the level's coarse-voxel lattice,
/// FLOORING to the coarse boundary at or below the surface (FF2a's load-bearing
/// choice, journal/0023: the stepped far top is always ≤ the near surface, so
/// the opaque near terrain wins the overlap band with no sink bias). Moved from
/// the FF2a streamer (dc-client) so top-down synthesis and the tile streamer
/// share ONE quantization (S-7: no second rule).
#[inline]
pub fn quantize_top(h: i32, stride: i64) -> i32 {
    let s = stride as i32;
    h.div_euclid(s) * s
}

/// Per-plan-column span stacks of one level-L node chunk, in world base
/// voxels. Index `z * 32 + x` (plan cell of the node), spans topmost-first.
///
/// `blocks` is the node's reduced (or synthesized) block chunk on the level-L
/// lattice; `pos` its level-L chunk position, which anchors the world-Y frame:
/// level voxel row `vy` spans base rows `[vy * stride, (vy+1) * stride)`.
///
/// Where `materials` is present, a span whose top voxel carries non-empty
/// reduced contents renders as `classify(contents)` — the material pyramid
/// ([`crate::materials::lod::derive_material_lod_chunk`]) reaching the eye.
/// Empty contents (a debris-free chunk, or the S1 authority) fall back to the
/// block pyramid's own block, which the composition invariant guarantees is
/// present wherever material LOD claims volume.
pub fn node_column_spans(
    level: u8,
    pos: ChunkPos,
    blocks: &PalettedChunk,
    materials: Option<(&MaterialChunk, &MixtureTable)>,
) -> Vec<Vec<ColumnSpan>> {
    let n = CHUNK_SIZE_USIZE;
    let stride = level_stride(level);
    let base_row = i64::from(pos.y) * n as i64;
    let mut out = vec![Vec::new(); n * n];
    for z in 0..n {
        for x in 0..n {
            let stacks = &mut out[z * n + x];
            let mut y = n;
            while y > 0 {
                y -= 1;
                if !blocks.get(x, y, z).is_solid() {
                    continue;
                }
                // A solid run [ylo..=yhi] (descending scan: y is the run top).
                let yhi = y;
                let mut ylo = y;
                while ylo > 0 && blocks.get(x, ylo - 1, z).is_solid() {
                    ylo -= 1;
                }
                y = ylo;
                let block = span_block(blocks, materials, x, yhi, z);
                stacks.push(ColumnSpan {
                    top: (((base_row + yhi as i64) + 1) * stride) as i32,
                    bottom: ((base_row + ylo as i64) * stride) as i32,
                    block,
                });
            }
        }
    }
    out
}

/// The block a span renders as: the reduced contents' [`classify`] where the
/// material chunk speaks, else the block chunk's own voxel (see
/// [`node_column_spans`]).
fn span_block(
    blocks: &PalettedChunk,
    materials: Option<(&MaterialChunk, &MixtureTable)>,
    x: usize,
    y: usize,
    z: usize,
) -> Block {
    if let Some((mc, table)) = materials {
        let id = mc.get(x, y, z);
        if id != MixtureId::EMPTY
            && let Some(contents) = table.get(id)
        {
            let b = classify(contents);
            if b != Block::Air {
                return b;
            }
        }
    }
    blocks.get(x, y, z)
}

/// Compose one far column from the two-sided derivation: `known` node answers
/// (reduced from real children) laid over the synthesized backdrop span.
///
/// `synth` is the top-sheet answer (`bottom == FAR_BOTTOM_UNBOUNDED`). `known`
/// holds `(extent_bottom, extent_top, spans)` per fully-inserted node covering
/// this column — extents in base voxels, non-overlapping, any order; spans as
/// [`node_column_spans`] produced them. Within a known extent the reduction is
/// the answer (its air is real knowledge); outside every extent the synthesized
/// claim stands (ungenerated ≠ empty). Touching spans merge, keeping the upper
/// span's block, so a surface node's reduced spans fuse seamlessly with the
/// synthesized ground below it. The result is descending, disjoint, and always
/// ends in an unbounded span.
pub fn compose_column(
    synth: ColumnSpan,
    known: &[(i32, i32, Vec<ColumnSpan>)],
) -> Vec<ColumnSpan> {
    debug_assert_eq!(synth.bottom, FAR_BOTTOM_UNBOUNDED);
    // Synth pieces = the synthesized span minus every known extent. i64 math
    // so the UNBOUNDED sentinel needs no special casing.
    let mut pieces: Vec<(i64, i64)> = vec![(i64::from(FAR_BOTTOM_UNBOUNDED), i64::from(synth.top))];
    for &(kb, kt, _) in known {
        let (kb, kt) = (i64::from(kb), i64::from(kt));
        let mut next = Vec::with_capacity(pieces.len() + 1);
        for (pb, pt) in pieces {
            if kt <= pb || kb >= pt {
                next.push((pb, pt));
                continue;
            }
            if pb < kb {
                next.push((pb, kb));
            }
            if kt < pt {
                next.push((kt, pt));
            }
        }
        pieces = next;
    }

    let mut spans: Vec<ColumnSpan> = Vec::new();
    for (pb, pt) in pieces {
        if pt > pb {
            spans.push(ColumnSpan {
                top: pt as i32,
                bottom: if pb <= i64::from(FAR_BOTTOM_UNBOUNDED) {
                    FAR_BOTTOM_UNBOUNDED
                } else {
                    pb as i32
                },
                block: synth.block,
            });
        }
    }
    for (_, _, kspans) in known {
        spans.extend_from_slice(kspans);
    }
    // Descending by top; disjoint by construction (known extents don't overlap
    // each other, and synth pieces avoid them).
    spans.sort_unstable_by_key(|s| std::cmp::Reverse(s.top));
    // Merge touching spans; the upper span's block wins (it owns the top face).
    let mut merged: Vec<ColumnSpan> = Vec::with_capacity(spans.len());
    for s in spans {
        if let Some(last) = merged.last_mut()
            && last.bottom != FAR_BOTTOM_UNBOUNDED
            && last.bottom == s.top
        {
            last.bottom = s.bottom;
            continue;
        }
        merged.push(s);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    use crate::materials::MaterialId;
    use crate::materials::contents::VoxelContents;

    fn synth(top: i32, block: Block) -> ColumnSpan {
        ColumnSpan {
            top,
            bottom: FAR_BOTTOM_UNBOUNDED,
            block,
        }
    }

    #[test]
    fn quantize_top_floors_to_the_coarse_lattice() {
        // Moved with the function from dc-client (journal/0023's proof): on the
        // lattice, never above the surface, error under one coarse voxel.
        for level in 0..=4u8 {
            let s = level_stride(level);
            for h in -40i32..=40 {
                let q = quantize_top(h, s);
                assert_eq!(q % s as i32, 0, "not on the coarse lattice");
                assert!(q <= h, "quantized top rose above the surface");
                assert!(h - q < s as i32, "error exceeds a coarse voxel");
            }
        }
    }

    #[test]
    fn node_spans_read_runs_with_world_anchoring() {
        // Level-2 node at chunk (0, 3, 0): stride 4, base row 96.
        let mut dense = Chunk::new();
        // Column (5, 7): solid rows 0..=9, air 10..=11, solid 12..=13 (an
        // overhang), rest air.
        for y in 0..=9 {
            dense.set(5, y, 7, Block::Stone);
        }
        dense.set(5, 12, 7, Block::Grass);
        dense.set(5, 13, 7, Block::Grass);
        let chunk = PalettedChunk::from_dense(&dense);
        let spans = node_column_spans(2, ChunkPos::new(0, 3, 0), &chunk, None);
        let col = &spans[7 * 32 + 5];
        assert_eq!(
            col,
            &vec![
                ColumnSpan {
                    top: (96 + 14) * 4,
                    bottom: (96 + 12) * 4,
                    block: Block::Grass,
                },
                ColumnSpan {
                    top: (96 + 10) * 4,
                    bottom: 96 * 4,
                    block: Block::Stone,
                },
            ],
            "descending spans, world-anchored, run-top block"
        );
        assert!(spans[0].is_empty(), "untouched columns have no spans");
    }

    #[test]
    fn node_spans_refine_block_through_reduced_materials() {
        // A solid stone column whose top voxel carries sand contents: the span
        // renders as classify(sand) — the material pyramid reaching the eye.
        let mut dense = Chunk::new();
        for y in 0..4 {
            dense.set(0, y, 0, Block::Stone);
        }
        let chunk = PalettedChunk::from_dense(&dense);
        let mut table = MixtureTable::new();
        let sand = table.intern(VoxelContents::debris_only(&[MaterialId::SAND; 8]).unwrap());
        let mut ids = vec![MixtureId::EMPTY; crate::chunk::CHUNK_VOLUME];
        ids[Chunk::index(0, 3, 0)] = sand;
        let mc = MaterialChunk::from_dense(&ids);
        let spans = node_column_spans(0, ChunkPos::new(0, 0, 0), &chunk, Some((&mc, &table)));
        let expected = classify(table.get(sand).unwrap());
        assert_ne!(expected, Block::Air);
        assert_eq!(spans[0][0].block, expected, "classify() decides the span");
        assert_eq!(spans[0][0].top, 4);
        // Without materials the block chunk's own voxel answers.
        let bare = node_column_spans(0, ChunkPos::new(0, 0, 0), &chunk, None);
        assert_eq!(bare[0][0].block, Block::Stone);
    }

    #[test]
    fn compose_with_no_known_nodes_is_the_synth_span() {
        let s = synth(40, Block::Grass);
        assert_eq!(compose_column(s, &[]), vec![s], "the default case, forever");
    }

    #[test]
    fn compose_merges_a_reduced_surface_node_onto_the_synth_ground() {
        // Known node [32, 64) says: solid up to 42 (reduction rounded up past
        // the synth's floored 40). Below 32 the synth continues. One fused span.
        let known = vec![(
            32,
            64,
            vec![ColumnSpan {
                top: 42,
                bottom: 32,
                block: Block::Stone,
            }],
        )];
        assert_eq!(
            compose_column(synth(40, Block::Grass), &known),
            vec![ColumnSpan {
                top: 42,
                bottom: FAR_BOTTOM_UNBOUNDED,
                block: Block::Stone,
            }],
            "reduced span fuses with the synthesized ground below"
        );
    }

    #[test]
    fn compose_lets_known_air_truncate_the_synth_claim() {
        // A fully-inserted node [32, 64) reduced to all-air, over a synth top
        // of 50: the reduction KNOWS rows 32..50 are air, the synth keeps only
        // what lies outside the known extent.
        let known = vec![(32, 64, Vec::new())];
        assert_eq!(
            compose_column(synth(50, Block::Grass), &known),
            vec![ColumnSpan {
                top: 32,
                bottom: FAR_BOTTOM_UNBOUNDED,
                block: Block::Grass,
            }],
            "known air wins inside its extent; ungenerated ground survives below"
        );
    }

    #[test]
    fn compose_keeps_overhangs_and_gaps_between_known_nodes() {
        // Known surface node [64, 96) has an overhang stack; the gap [32, 64)
        // is NOT known — the synthesized claim stands there (ungenerated is not
        // empty, A-5), and merges with the overhang's grounded lower span.
        let known = vec![(
            64,
            96,
            vec![
                ColumnSpan {
                    top: 90,
                    bottom: 84,
                    block: Block::Grass,
                },
                ColumnSpan {
                    top: 76,
                    bottom: 64,
                    block: Block::Stone,
                },
            ],
        )];
        let got = compose_column(synth(70, Block::Dirt), &known);
        assert_eq!(
            got,
            vec![
                ColumnSpan {
                    top: 90,
                    bottom: 84,
                    block: Block::Grass,
                },
                ColumnSpan {
                    top: 76,
                    bottom: FAR_BOTTOM_UNBOUNDED,
                    block: Block::Stone,
                },
            ],
            "overhang preserved; known lower span fuses with unbounded synth ground"
        );
        // Invariant every composed column carries: it ends unbounded.
        assert_eq!(got.last().unwrap().bottom, FAR_BOTTOM_UNBOUNDED);
    }
}
