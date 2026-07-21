//! The forms/partials **fill contract** (docs/design/materials.md § forms
//! design pass, ratified 2026-07-21):
//!
//! > Contents are the single source of truth; the block is a pure derived
//! > classification. For every voxel that carries contents,
//! > `block == classify(contents)`.
//!
//! Two proofs live here.
//!
//! 1. **The invariant**, over several seeds and many chunks: every voxel whose
//!    contents are non-empty must carry exactly `classify(contents)`. Voxels
//!    with no contents record (air, the surface veneer stub, the legacy soil
//!    band, unrecorded basement, ruin posts) are outside the contract by the
//!    absent-contents rule — asserted here too, so that exception cannot widen
//!    silently.
//!
//! 2. **Byte identity across the rewire.** Moving the block tier onto
//!    `classify` must not change the world by a single byte. The fingerprints
//!    below were captured by running this exact sampler against the
//!    *pre-rewire* generator (blocks computed by the old class→block table) and
//!    pasted in; the rewired generator must reproduce them. Run with
//!    `DC_PRINT_GOLDENS=1` to print the current values.
//!
//! If a golden ever moves, the world moved. That is a bug until a journal
//! entry says otherwise.

use std::collections::BTreeMap;

use dc_core::materials::geology;
use dc_core::{Block, ChunkPos, VoxelContents, classify};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

/// FNV-1a over a byte stream — the same cheap fingerprint tests/geology.rs uses.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01B3);
    }
    fn bytes(&mut self, bs: &[u8]) {
        for &b in bs {
            self.byte(b);
        }
    }
    fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }
}

/// A deterministic spread of surface chunks: a spiral of chunk columns around
/// the origin plus a few far excursions, each taken at its own surface chunk y
/// and one chunk below it (so the sample reaches buried strata, not just the
/// air/surface interface).
fn sample_positions(g: &mut WorldGenerator<'_>) -> Vec<ChunkPos> {
    let mut out = Vec::new();
    for k in 0..40i64 {
        let (cx, cz) = (k * 13 - 200, (k % 7) * 17 - 60);
        let cy = g.surface_chunk_y(cx, cz);
        out.push(ChunkPos::new(cx as i32, cy, cz as i32));
        out.push(ChunkPos::new(cx as i32, cy - 1, cz as i32));
    }
    out
}

/// Blocks + material sidecar + mixture table, fingerprinted over the sample.
fn world_fingerprint(seed: u64, extent: Extent) -> (u64, u64, u64) {
    let pregen = Pregen::run(WorldParams { seed, extent });
    let mut g = WorldGenerator::with_geology(&pregen, geology::vanilla());
    let positions = sample_positions(&mut g);
    let mut blocks = Fnv::new();
    let mut materials = Fnv::new();
    for pos in positions {
        let (chunk, mat) = g.generate_chunk_with_materials(pos);
        for b in chunk.blocks() {
            blocks.u16(*b as u16);
        }
        materials.bytes(&mat.encode());
    }
    let mut table = Fnv::new();
    table.bytes(&g.mixture_table().encode());
    (blocks.0, materials.0, table.0)
}

/// `(seed, extent, block hash, material hash, table hash)`.
///
/// **Moved 2026-07-21 by the carry-`H` slice (journal/0053) — authorized.** The
/// world genuinely changed: soil/regolith depth stopped being a guess from
/// present-day precipitation and became the deep sim's recorded loose-column
/// thickness `H`. Every land column's veneer thickness moved, some columns bared
/// out to rock for the first time, and ocean/wilds soil bands re-depthed. The
/// previous values, captured against the pre-rewire generator on 2026-07-21 and
/// kept here so the move is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x2028_2534_A728_3142, 0x9EF6_F01B_F072_70C2, 0xBD39_3CF4_10DF_8067)
/// (0x0000_0000_0000_0539, "medium", 0xF584_F3B2_28AC_B00A, 0x51DC_F658_A7F0_8EC9, 0x4123_D801_8035_82D8)
/// (0x0000_00C1_1A7E_2026, "small",  0x641A_2C85_7A74_0F8C, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// Note the *mixture table* hashes did not move at all, and the small world's
/// material hash did not either: the change is one of thickness and extent, not
/// of which materials exist or how they are interned.
const GOLDENS: [(u64, &str, u64, u64, u64); 3] = [
    (
        0x0000_0D5E_ED57_2026,
        "medium",
        0x5863_4C80_C5C1_4D62,
        0x9B67_5DAB_C68D_BDBC,
        0xBD39_3CF4_10DF_8067,
    ),
    (
        0x0000_0000_0000_0539,
        "medium",
        0x54CD_8922_B21E_852A,
        0x89F9_9D67_61D2_B38C,
        0x4123_D801_8035_82D8,
    ),
    (
        0x0000_00C1_1A7E_2026,
        "small",
        0x024F_5F94_8C2E_39CC,
        0x3222_7B87_48CB_0F75,
        0xD0A3_9718_6727_310C,
    ),
];

fn extent_of(name: &str) -> Extent {
    match name {
        "small" => Extent::Small,
        _ => Extent::Medium,
    }
}

#[test]
fn generated_world_is_byte_identical_to_the_pre_contract_goldens() {
    let print = std::env::var("DC_PRINT_GOLDENS").is_ok();
    let mut mismatches = Vec::new();
    for (seed, extent, gb, gm, gt) in GOLDENS {
        let (b, m, t) = world_fingerprint(seed, extent_of(extent));
        if print {
            println!(
                "    (\n        {seed:#018X},\n        \"{extent}\",\n        \
                 {b:#018X},\n        {m:#018X},\n        {t:#018X},\n    ),"
            );
        }
        if (b, m, t) != (gb, gm, gt) {
            mismatches.push(format!(
                "seed {seed:#x} ({extent}): blocks {b:#018X} (want {gb:#018X}), \
                 materials {m:#018X} (want {gm:#018X}), table {t:#018X} (want {gt:#018X})"
            ));
        }
    }
    assert!(
        print || mismatches.is_empty(),
        "the generated world moved — deriving the block from contents must be \
         byte-neutral:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn block_equals_classify_of_contents() {
    let mut checked = 0usize;
    let mut with_contents = 0usize;
    // Which legacy blocks the absent-contents rule is covering, and how often —
    // printed so the exception's shape is visible, asserted so it cannot widen
    // to a block that ought to have carried a record.
    let mut absent: BTreeMap<u16, usize> = BTreeMap::new();

    for (seed, extent) in [
        (0x0D5E_ED57_2026u64, Extent::Medium),
        (1337, Extent::Medium),
        (0x00C1_1A7E_2026, Extent::Small),
    ] {
        let pregen = Pregen::run(WorldParams { seed, extent });
        let mut g = WorldGenerator::with_geology(&pregen, geology::vanilla());
        for pos in sample_positions(&mut g) {
            let chunk = g.generate_chunk(pos);
            let grid = g.chunk_contents(pos);
            for z in 0..32usize {
                for y in 0..32usize {
                    for x in 0..32usize {
                        let block = chunk.get(x, y, z);
                        let contents = grid
                            .as_ref()
                            .map_or(VoxelContents::EMPTY, |gr| gr.get(x, y, z));
                        checked += 1;
                        if contents.is_empty() {
                            *absent.entry(block as u16).or_default() += 1;
                            continue;
                        }
                        with_contents += 1;
                        assert_eq!(
                            block,
                            classify(&contents),
                            "block/contents disagree at {pos:?} local ({x},{y},{z}): \
                             block {block:?}, contents {contents:?}"
                        );
                    }
                }
            }
        }
    }
    assert!(
        with_contents > 100_000,
        "only {with_contents} voxels carried contents — the sample is not \
         reaching buried strata"
    );
    println!(
        "block == classify(contents): {with_contents} contents-bearing voxels of \
         {checked}; absent-contents blocks {absent:?}"
    );
    // The absent-contents rule, pinned: only air, the surface-veneer stub
    // blocks (Grass/Dirt/Stone), the legacy soil band (Dirt/Stone), unrecorded
    // basement (Stone) and ruin posts (Wood) may lack a record. A geology block
    // without contents would mean the two paths disagree about WHERE the record
    // is, which is the exact failure the contract exists to prevent.
    let legacy: [u16; 5] = [
        Block::Air as u16,
        Block::Stone as u16,
        Block::Dirt as u16,
        Block::Grass as u16,
        Block::Wood as u16,
    ];
    for (&b, &n) in &absent {
        let allowed = legacy.contains(&b);
        assert!(
            allowed,
            "block id {b} appears with no contents record ({n} voxels) — the \
             absent-contents rule covers only air, the veneer stub, the legacy \
             soil band, unrecorded basement and ruin posts"
        );
    }
}

/// The classification is a *pure function of contents*: re-classifying the
/// chunk's own contents grid twice, and classifying an independently rebuilt
/// grid, must agree. Guards against anyone smuggling generator state (member
/// identity, mixture ids, iteration order) into the derivation.
#[test]
fn classification_does_not_depend_on_generator_state() {
    let pregen = Pregen::run(WorldParams {
        seed: 0x0D5E_ED57_2026,
        extent: Extent::Medium,
    });
    let mut a = WorldGenerator::with_geology(&pregen, geology::vanilla());
    let mut b = WorldGenerator::with_geology(&pregen, geology::vanilla());
    // `a` is warmed over a wide area first, so its caches and mixture table are
    // in a completely different state from `b`'s when the two are compared.
    for k in 0..30i64 {
        let _ = a.surface_chunk_y(k * 31 - 400, k * 17);
    }
    let mut compared = 0usize;
    for k in 0..12i64 {
        let (cx, cz) = (k * 23 - 80, k * 9 - 30);
        let cy = b.surface_chunk_y(cx, cz);
        let pos = ChunkPos::new(cx as i32, cy - 1, cz as i32);
        let ga = a.chunk_contents(pos);
        let gb = b.chunk_contents(pos);
        assert_eq!(ga.is_some(), gb.is_some());
        let (Some(ga), Some(gb)) = (ga, gb) else {
            continue;
        };
        for z in 0..32usize {
            for y in 0..32usize {
                for x in 0..32usize {
                    let (ca, cb) = (ga.get(x, y, z), gb.get(x, y, z));
                    assert_eq!(ca, cb, "contents diverged under different cache warmth");
                    assert_eq!(classify(&ca), classify(&cb));
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 10_000, "compared only {compared} voxels");
}
