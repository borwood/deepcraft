//! `--bench-storage`: headless measurement pass for S3.
//!
//! Same terrain as the S1 bench (seed 1337, 256x128x256 m region) at the
//! decided scale N=2 (0.9 m voxels). Measures, in order:
//!
//! 1. palette compression — encoded bytes/m^3 vs the 2.74 B/m^3 raw baseline,
//!    split by chunk class (uniform air / uniform solid / mixed);
//! 2. LOD pyramid derive time per level (real derivation via dc-core);
//! 3. column-summary build time, from full-res data and from LOD-1-only data
//!    (the "answer without loading full-res chunks" path), plus agreement;
//! 4. far-mesh cost for the 1.2 km ring set vs extrapolated full resolution.
//!
//! Single-threaded, release profile, no window/GPU.

use std::collections::HashMap;
use std::time::Instant;

use dc_core::{
    CHUNK_SIZE, Chunk, ChunkContainer, ChunkPos, ColumnSummaries, LodPyramid, PalettedChunk,
    VoxelScale,
};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::bench::{BENCH_SEED, REGION_MAX_M, REGION_MIN_M, format_bytes, group_thousands};
use crate::farmesh::{
    FULL_DETAIL_RADIUS_M, HorizonConfig, coarse_scale, far_chunk_center_m, wanted_far_positions,
};
use crate::meshing::mesh_chunk;
use crate::worldgen::TerrainGen;

/// The scale every S3 measurement runs at (DECIDED by S1: player = 2 voxels,
/// voxel = 0.9 m).
const PLAYER_VOXELS: u32 = 2;

/// S1's measured raw baseline at N=2, for the comparison column.
const RAW_BASELINE_B_PER_M3: f64 = 2.74;
/// S1's measured culled-mesh density at N=2, for far-mesh extrapolation.
const S1_TRIS_PER_M2: f64 = 30.8;
/// S1's measured per-chunk mesh time at N=2 (seconds), for extrapolation.
const S1_MESH_S_PER_CHUNK: f64 = 0.73e-3;

pub fn run() {
    let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, PLAYER_VOXELS);
    let generator = TerrainGen::new(BENCH_SEED);
    let chunk_m = scale.voxels_to_meters(f64::from(CHUNK_SIZE));
    let chunk_vol_m3 = chunk_m * chunk_m * chunk_m;

    println!(
        "S3 storage bench: seed {BENCH_SEED}, N={PLAYER_VOXELS} ({:.3} m voxels), region {:.0}x{:.0}x{:.0} m",
        scale.voxel_size_m(),
        REGION_MAX_M[0] - REGION_MIN_M[0],
        REGION_MAX_M[1] - REGION_MIN_M[1],
        REGION_MAX_M[2] - REGION_MIN_M[2],
    );
    println!();

    // ---- generate the region (level-0 truth) --------------------------------
    let lo = REGION_MIN_M.map(|m| (m / chunk_m).floor() as i32);
    let hi = REGION_MAX_M.map(|m| ((m / chunk_m).ceil() as i32) - 1);
    let gen_start = Instant::now();
    let mut chunks: HashMap<ChunkPos, Chunk> = HashMap::new();
    for cy in lo[1]..=hi[1] {
        for cz in lo[2]..=hi[2] {
            for cx in lo[0]..=hi[0] {
                let pos = ChunkPos::new(cx, cy, cz);
                chunks.insert(pos, generator.generate_chunk(scale, pos));
            }
        }
    }
    println!(
        "generated {} level-0 chunks in {:.2} s",
        chunks.len(),
        gen_start.elapsed().as_secs_f64()
    );
    println!();

    // ---- 1. palette compression --------------------------------------------
    #[derive(Default, Clone, Copy)]
    struct ClassStat {
        chunks: u64,
        bytes: u64,
    }
    let mut uniform_air = ClassStat::default();
    let mut uniform_solid = ClassStat::default();
    let mut mixed = ClassStat::default();

    let compress_start = Instant::now();
    let mut paletted: HashMap<ChunkPos, PalettedChunk> = HashMap::new();
    for (pos, chunk) in &chunks {
        paletted.insert(*pos, PalettedChunk::from_dense(chunk));
    }
    let compress_s = compress_start.elapsed().as_secs_f64();

    let encode_start = Instant::now();
    let mut total_bytes = 0u64;
    for p in paletted.values() {
        let bytes = ChunkContainer::new(p.clone()).encode();
        let stat = match p.is_uniform() {
            Some(b) if !b.is_solid() => &mut uniform_air,
            Some(_) => &mut uniform_solid,
            None => &mut mixed,
        };
        stat.chunks += 1;
        stat.bytes += bytes.len() as u64;
        total_bytes += bytes.len() as u64;
    }
    let encode_s = encode_start.elapsed().as_secs_f64();

    // Round-trip sanity on a few chunks.
    for p in paletted.values().take(5) {
        let container = ChunkContainer::new(p.clone());
        assert_eq!(
            ChunkContainer::decode(&container.encode())
                .expect("round-trip")
                .voxels,
            *p
        );
    }

    let raw_bytes = (chunks.len() * Chunk::raw_byte_size()) as u64;
    println!("palette compression ({} chunks):", chunks.len());
    println!(
        "| Class | Chunks | Encoded bytes | B/chunk | B/m^3 | vs raw {RAW_BASELINE_B_PER_M3} B/m^3 |"
    );
    println!("|---|---|---|---|---|---|");
    for (name, stat) in [
        ("all-air (uniform)", uniform_air),
        ("all-solid (uniform)", uniform_solid),
        ("mixed (surface/caves)", mixed),
    ] {
        if stat.chunks == 0 {
            continue;
        }
        let per_chunk = stat.bytes as f64 / stat.chunks as f64;
        let per_m3 = per_chunk / chunk_vol_m3;
        println!(
            "| {name} | {} | {} | {:.1} | {:.4} | {:.1}x smaller |",
            stat.chunks,
            group_thousands(stat.bytes),
            per_chunk,
            per_m3,
            RAW_BASELINE_B_PER_M3 / per_m3,
        );
    }
    let overall_per_m3 = total_bytes as f64 / (chunks.len() as f64 * chunk_vol_m3);
    println!(
        "| **all** | {} | {} | {:.1} | {:.4} | {:.1}x smaller |",
        chunks.len(),
        group_thousands(total_bytes),
        total_bytes as f64 / chunks.len() as f64,
        overall_per_m3,
        RAW_BASELINE_B_PER_M3 / overall_per_m3,
    );
    println!(
        "raw dense: {} ({}); compress {compress_s:.3} s, encode {encode_s:.3} s",
        format_bytes(raw_bytes),
        group_thousands(raw_bytes),
    );
    println!();

    // ---- 2. LOD pyramid derive time per level ------------------------------
    let mut pyramid = LodPyramid::default();
    let insert_start = Instant::now();
    for (pos, p) in &paletted {
        pyramid.insert_chunk(*pos, p.clone());
    }
    let insert_s = insert_start.elapsed().as_secs_f64();

    println!("LOD pyramid (majority-non-air rule; insert of level 0 took {insert_s:.3} s):");
    println!("| Level | Chunks | Derive time | ms/chunk |");
    println!("|---|---|---|---|");
    for level in 1..=pyramid.max_level() {
        let mut positions = pyramid.known_positions(level);
        positions.sort_unstable_by_key(|p| (p.y, p.z, p.x));
        let t = Instant::now();
        for pos in &positions {
            pyramid.get_or_derive(level, *pos);
        }
        let dt = t.elapsed().as_secs_f64();
        println!(
            "| {level} | {} | {:.1} ms | {:.3} |",
            positions.len(),
            dt * 1e3,
            dt * 1e3 / positions.len() as f64,
        );
    }
    println!();

    // ---- 3. column summaries ------------------------------------------------
    let vx_lo = i64::from(lo[0]) * i64::from(CHUNK_SIZE);
    let vx_hi = (i64::from(hi[0]) + 1) * i64::from(CHUNK_SIZE);
    let vz_lo = i64::from(lo[2]) * i64::from(CHUNK_SIZE);
    let vz_hi = (i64::from(hi[2]) + 1) * i64::from(CHUNK_SIZE);
    let vy_lo = i64::from(lo[1]) * i64::from(CHUNK_SIZE);
    let vy_hi = (i64::from(hi[1]) + 1) * i64::from(CHUNK_SIZE);
    let columns = ((vx_hi - vx_lo) * (vz_hi - vz_lo)) as u64;

    let t = Instant::now();
    let mut full_summaries = ColumnSummaries::new(vy_lo, vy_hi);
    for z in vz_lo..vz_hi {
        for x in vx_lo..vx_hi {
            full_summaries.get_or_build(&mut pyramid, x, z);
        }
    }
    let full_s = t.elapsed().as_secs_f64();

    // LOD-only pyramid: ONLY the derived level-1 chunks, inserted as if loaded
    // from disk. Every query below answers without any full-res chunk.
    let mut lod_only = LodPyramid::default();
    for pos in pyramid.known_positions(1) {
        let chunk = pyramid
            .get_or_derive(1, pos)
            .expect("derived above")
            .clone();
        lod_only.insert_lod_chunk(1, pos, chunk);
    }
    let t = Instant::now();
    let mut lod_summaries = ColumnSummaries::new(vy_lo, vy_hi);
    for z in vz_lo..vz_hi {
        for x in vx_lo..vx_hi {
            lod_summaries.get_or_build(&mut lod_only, x, z);
        }
    }
    let lod_s = t.elapsed().as_secs_f64();

    // Agreement between the LOD-1 surface and the full-res surface.
    let mut both = 0u64;
    let mut within = [0u64; 3]; // <=1, <=2, <=4 voxels
    let mut max_dev = 0i64;
    for z in vz_lo..vz_hi {
        for x in vx_lo..vx_hi {
            let a = full_summaries.get_or_build(&mut pyramid, x, z).top_solid_y;
            let b = lod_summaries.get_or_build(&mut lod_only, x, z).top_solid_y;
            if let (Some(a), Some(b)) = (a, b) {
                both += 1;
                let dev = (a - b).abs();
                max_dev = max_dev.max(dev);
                for (i, bound) in [1i64, 2, 4].iter().enumerate() {
                    if dev <= *bound {
                        within[i] += 1;
                    }
                }
            }
        }
    }

    println!(
        "column summaries ({} columns, y in [{vy_lo}, {vy_hi})):",
        group_thousands(columns)
    );
    println!("| Source | Build time | us/column |");
    println!("|---|---|---|");
    println!(
        "| full-res + LOD | {full_s:.2} s | {:.2} |",
        full_s * 1e6 / columns as f64
    );
    println!(
        "| LOD-1 only (no full-res loaded) | {lod_s:.2} s | {:.2} |",
        lod_s * 1e6 / columns as f64
    );
    println!(
        "LOD-1 surface vs full-res surface: {:.1}% within 1 voxel, {:.1}% within 2, {:.1}% within 4; max deviation {max_dev} voxels ({} columns)",
        100.0 * within[0] as f64 / both as f64,
        100.0 * within[1] as f64 / both as f64,
        100.0 * within[2] as f64 / both as f64,
        group_thousands(both),
    );
    println!();

    // ---- 4. far-mesh cost for the 1.2 km field ------------------------------
    let viewer = DVec3::new(0.0, generator.surface_height_m(0.0, 0.0) + 2.0, 0.0);
    // The S3 bench measures the SHIPPED (default) horizon; `--horizon`
    // (journal/0042) is a launch-time knob on the interactive client, and this
    // table is the reference number the spike results doc quotes.
    let hz = HorizonConfig::default();
    println!(
        "far mesh: viewer at ({:.0}, {:.0}, {:.0}) m, rings {:?}",
        viewer.x, viewer.y, viewer.z, hz.ring_edges
    );
    println!("| Level | Ring (m) | Chunks | Gen time | Mesh time | Triangles |");
    println!("|---|---|---|---|---|---|");
    let mut far_tris = 0u64;
    let mut far_total_s = 0.0f64;
    for level in 1..=4u8 {
        let cscale = coarse_scale(scale, level);
        let positions = wanted_far_positions(scale, viewer, level, &hz);
        let mut gen_s = 0.0f64;
        let mut mesh_s = 0.0f64;
        let mut tris = 0u64;
        for pos in &positions {
            let t = Instant::now();
            let chunk = generator.generate_chunk(cscale, *pos);
            gen_s += t.elapsed().as_secs_f64();
            // The far field carries no per-voxel contents: coverage is binary.
            let neighbor_fill = |x: i64, y: i64, z: i64| {
                crate::meshing::cover_frac(generator.block_at(cscale, x, y, z), None)
            };
            let t = Instant::now();
            let mesh = mesh_chunk(
                &chunk,
                *pos,
                cscale.voxel_size_m() as f32,
                &neighbor_fill,
                None,
            );
            mesh_s += t.elapsed().as_secs_f64();
            tris += mesh.triangle_count() as u64;
        }
        // Sanity: every wanted chunk really is in its ring.
        for pos in &positions {
            let d = (far_chunk_center_m(scale, level, *pos) - viewer).length();
            assert!(
                d >= hz.ring_edges[usize::from(level) - 1] && d < hz.ring_edges[usize::from(level)]
            );
        }
        println!(
            "| {level} | {:.0}-{:.0} | {} | {:.2} s | {:.2} s | {} |",
            hz.ring_edges[usize::from(level) - 1],
            hz.ring_edges[usize::from(level)],
            positions.len(),
            gen_s,
            mesh_s,
            group_thousands(tris),
        );
        far_tris += tris;
        far_total_s += gen_s + mesh_s;
    }

    // Full-res comparators over the same annulus (extrapolated from S1's
    // measured N=2 numbers; actually generating it would take minutes).
    let inner = FULL_DETAIL_RADIUS_M - 16.0;
    let outer = hz.ring_edges[4];
    let annulus_m2 = std::f64::consts::PI * (outer * outer - inner * inner);
    let shell_m3 = 4.0 / 3.0 * std::f64::consts::PI * (outer.powi(3) - inner.powi(3));
    let full_res_chunks = shell_m3 / chunk_vol_m3;
    let full_res_tris = annulus_m2 * S1_TRIS_PER_M2;
    let full_res_mesh_s = full_res_chunks * S1_MESH_S_PER_CHUNK;
    println!();
    println!(
        "far field total: {} triangles, {:.2} s gen+mesh (single-threaded)",
        group_thousands(far_tris),
        far_total_s
    );
    println!(
        "full-res same annulus (extrapolated from S1 @ {S1_TRIS_PER_M2} tris/m^2): ~{} triangles ({:.0}x more), \
         ~{} 3D-shell chunks = {} raw dense, ~{:.0} s mesh time ({:.0}x more)",
        group_thousands(full_res_tris as u64),
        full_res_tris / far_tris as f64,
        group_thousands(full_res_chunks as u64),
        format_bytes((full_res_chunks * Chunk::raw_byte_size() as f64) as u64),
        full_res_mesh_s,
        full_res_mesh_s / far_total_s,
    );

    // Determinism spot check: the whole pipeline re-run must byte-match.
    let re = PalettedChunk::from_dense(
        &generator.generate_chunk(scale, ChunkPos::new(lo[0], lo[1], lo[2])),
    );
    assert_eq!(
        ChunkContainer::new(re.clone()).encode(),
        ChunkContainer::new(paletted[&ChunkPos::new(lo[0], lo[1], lo[2])].clone()).encode(),
        "regenerated chunk must encode identically"
    );
}
