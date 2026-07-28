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
//!    band, unrecorded basement) are outside the contract by the
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
            blocks.u16(b.ordinal());
        }
        materials.bytes(&mat.encode());
    }
    let mut table = Fnv::new();
    table.bytes(&g.mixture_table().encode());
    (blocks.0, materials.0, table.0)
}

/// `(seed, extent, block hash, material hash, table hash)`.
///
/// **Moved 2026-07-22 by the surface-branch removal (journal/0074) — authorized.**
/// The near surface voxel stopped being a parallel `surface_class` consult of the
/// deep record and became the record's **top span** through [`dc_worldgen::
/// ColumnFill`] (`plan(1)`), the same fill machinery as every buried voxel; the
/// buried column shifted down one record span to make room for the surface it now
/// owns. Both Medium worlds moved on **all three** hashes: the surface block (now
/// `classify` of the top-span partial rather than the drawn deep-record class),
/// the surface *and* shifted-buried contents (materials), and the mixture table
/// (the surface can now be a genuinely **mixed** partial — a state the
/// single-member surface path could not construct). **Small did not move at all**,
/// and the mechanism is exactly the surface-branch removal's blind spot:
/// Small's sampled chunks carry **no strata record** (empty `col.strata`), so
/// `ColumnFill` is empty, the surface takes the unchanged year-zero fallback block,
/// the buried column takes the unchanged legacy soil band, and there is nothing to
/// re-route — a record-less column is byte-identical under this slice by
/// construction. `block_equals_classify_of_contents` still passes with
/// absent-contents blocks limited to Air and Stone (the proof the surface's new
/// contents classify consistently).
///
/// The values immediately before this move (the composed A1+B1 tree), kept so it
/// is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x8A55_33FA_FAD8_66BF, 0xA0AA_B320_4308_0D7C, 0x93DE_D6C8_983E_D6F4)
/// (0x0000_0000_0000_0539, "medium", 0x08A9_920E_E5D8_BD27, 0x6D5E_9A98_61E3_84AC, 0x4A1F_F915_9667_15AF)
/// (0x0000_00C1_1A7E_2026, "small",  0x83A4_FD28_11CB_A19D, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// **Moved 2026-07-22 by the share-weighted susceptibility blend (journal/0072) —
/// authorized (audit site A1).** Erosion's four consumption sites stopped mapping
/// the outcrop verdict to one susceptibility-table entry and now blend the table by
/// the near-surface window's per-`Litho` shares (the `outcrop_shares` seam), so the
/// per-agent erosion rate field follows the thickness contours continuously instead
/// of stepping at the plurality crossover (the walk-0071 S-4 flag). This changes
/// erosion *rates* → bedrock surface **geometry**, so all three hashes moved on both
/// Medium worlds. **Small moved on its BLOCK hash only** — materials and table are
/// byte-identical — the same shape journal/0068 established: Small's sampled columns
/// express no deep record as contents (their solids are unrecorded basement Stone
/// and the veneer stub, both absent-contents), so the shifting bedrock surface
/// changes only which voxels are Stone vs Air, while the contents-derived material
/// and table hashes stay put. `block_equals_classify_of_contents` still passes with
/// absent-contents blocks limited to Air and Stone — the proof this is geometry, not
/// a classify regression.
///
/// The values immediately before this move (journal/0068's thickness-dominance
/// rule), kept so it is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x18BD_0AFA_8356_C969, 0x2FB1_6035_5F22_5986, 0x35AE_2FDA_A2DA_D8CD)
/// (0x0000_0000_0000_0539, "medium", 0x9A74_7558_4730_A767, 0x505A_ED87_E036_DEFE, 0xD8C3_2E22_82E9_3081)
/// (0x0000_00C1_1A7E_2026, "small",  0x5B85_62A0_A30E_2E1D, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// **Moved 2026-07-22 by the outcrop thickness-dominance rule (journal/0068) —
/// authorized, and the FIRST slice to move the Small control.** `exposed_litho`
/// now returns the lithology dominating the record's near-surface 0.9 m window
/// (deficit below a short record → basement) instead of the topmost unit, and
/// that feeds the erosion susceptibility tables. On the production Medium record
/// ~40 % of cells changed the rock they outcrop, so both Medium worlds moved on
/// all three hashes.
///
/// **The Small world moved on its BLOCK hash only** — materials and mixture table
/// are byte-identical. Every prior slice left Small wholly untouched and this file
/// called it "the control", on the belief that Small "runs no deep-time record".
/// That belief was imprecise: `Pregen` runs an **always-on** deep-time field at
/// every extent, and the Small world has ~20 700 recorded deep cells, 92 % of
/// which change outcrop under the new rule (measured, `examples/
/// outcrop_dominance_probe.rs`). Prior slices changed record *labels* and
/// *expression* — which Small does not surface as contents in the sampled columns,
/// so its material sidecar and table never moved. This slice is the first to change
/// erosion *rates*, which move the bedrock surface **geometry**; the sampled Small
/// columns express no deep record as contents (their solids are unrecorded
/// basement Stone and the veneer stub, both absent-contents), so the shifting
/// surface changes only which voxels are Stone vs Air — the block hash — while the
/// contents-derived material and table hashes stay put. `block_equals_classify_of_
/// contents` still passes with absent-contents blocks limited to Air and Stone,
/// which is the proof this is geometry and not a classify regression.
///
/// The values immediately before this move, kept so it is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x96CF_74C8_7207_F1BE, 0xAB18_73E0_8F0A_5527, 0x0920_6E0D_D4C9_2793)
/// (0x0000_0000_0000_0539, "medium", 0xA38F_E5B3_01E9_6A82, 0x12E5_D922_7DFD_9C70, 0xBC22_E827_1E70_AA0F)
/// (0x0000_00C1_1A7E_2026, "small",  0x024F_5F94_8C2E_39CC, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// **Moved 2026-07-22 by the organic-facies pair (journal/0063) — authorized.**
/// Two deliberate behaviour changes landed together, and both are visible here:
///
/// - **coal promotion moved onto the burial axis.** Buried peat becomes coal
///   when *its own overburden* reaches `COAL_BURIAL_M`, not when the seam is
///   thick. Coal is now rarer and deeper — measured on the production Medium
///   world, 12 892 → 2 216 coal-bearing deep cells — so columns that used to
///   band coal now band peat, and both hashes move.
/// - **charcoal stopped wearing its host's identity.** A fire bed routes to
///   `CLASS_ORGANIC_CHARCOAL` and competes for eighths like any other unit, so
///   the material sidecar carries charcoal where it previously carried the
///   surrounding mudstone, and the mixture table interns combinations that could
///   not exist before. Measured: 0.39 % of recorded voxel spans carry at least
///   one charcoal eighth.
///
/// **The Small world did not move**, for the third slice running: it runs no
/// deep-time record, so it has neither peat to promote nor a fire bed to
/// express. That is the control, and it held.
///
/// The values immediately before this move, kept so it is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x385D_BBFA_470A_DC40, 0xCDBA_4FF1_0691_FC2C, 0xD8D5_222E_2864_F931)
/// (0x0000_0000_0000_0539, "medium", 0xE6E4_1C61_159D_5B42, 0x3C69_3E20_80FC_8C8E, 0x69F5_739D_D620_597B)
/// (0x0000_00C1_1A7E_2026, "small",  0x024F_5F94_8C2E_39CC, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// **Moved 2026-07-21 by the surface member dither (journal/0058) —
/// authorized.** journal/0055 gave the surface voxel its own member resolution
/// but left it in `column`, drawn **once per 32×32 chunk footprint** at the
/// chunk's centre — so the world's skin quantized into 28.8 m rectilinear
/// patches of one member's albedo (`journal/assets/0056-surface-quantized-per-
/// chunk.png`), the same chunk-line family cutover corrections #6 retired for
/// the buried fill. The resolution moved into the shared `surface_sample`
/// kernel and now uses the per-voxel-column 3c-2 boundary dither, at the
/// voxel's own fractional position.
///
/// Exactly what moved, and it is the shape the change predicts:
///
/// - **the block hashes did NOT move, either medium world.** That is the
///   within-class invariance the whole change rests on: every member of a
///   vanilla class shares a `block_twin`, so dithering *within* a class cannot
///   change `classify` of the contents. Asserted, not hoped — by
///   `collapse::tests::surface_voxel_routes_through_columnfill_per_voxel`
///   (renamed from `surface_member_is_dithered_not_chunk_quantized` in
///   journal/0074, when the surface voxel moved onto `ColumnFill`).
/// - **the material hashes moved** — the sidecar is where the member lives, and
///   most of the world's surface columns now resolve a different one from their
///   chunk's centre pick.
/// - **the mixture tables moved**, because the surface voxel's single-member
///   partial fills intern member/eighth combinations the per-chunk pick could
///   not construct.
/// - **the Small world did not move at all**, again: no deep-time record, so no
///   record-derived surface member to dither. The control held.
///
/// The values immediately before this move, kept so it is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x385D_BBFA_470A_DC40, 0xFA60_7EFC_CAE1_98C9, 0xAA01_3DEE_63DB_7AF5)
/// (0x0000_0000_0000_0539, "medium", 0xE6E4_1C61_159D_5B42, 0x4206_56E5_EF00_1196, 0xE70C_7DF4_B89F_18CB)
/// (0x0000_00C1_1A7E_2026, "small",  0x024F_5F94_8C2E_39CC, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
///
/// **Moved 2026-07-21 by the distribution-first slice (journal/0055) —
/// authorized.** The world genuinely changed, and this is the largest move the
/// goldens have taken. `deposit_deep_history` stopped rounding each recorded
/// unit to whole voxels and dropping anything under half a voxel; the recorded
/// column now survives in metres to the voxel boundary and is quantized **once**,
/// per voxel span, by addressed stochastic rounding
/// (`dc_worldgen::fill`). Consequences visible in the fingerprints:
///
/// - **blocks moved** — three-quarters of the recorded sediment pile was being
///   deleted by the old rounding rule and now reaches the ground, so recorded
///   columns are deeper and their band boundaries land where the record puts
///   them rather than on voxel lines;
/// - **the mixture table moved for the first time** (it survived journal/0053
///   untouched): contact voxels now hold genuinely mixed contents, so states
///   exist that the generator could never previously construct;
/// - **the Small world did not move at all** — it runs no deep-time record, so
///   there is nothing to slice differently, and its sampled surfaces were
///   already Dirt/Stone under the retired climate rule. That is the control.
///
/// The same slice then folded in the **surface voxel** (materials.md
/// § Sequencing AMENDED 2026-07-21, user: *"we don't have to have this
/// problematic of deciding which material to skin the world with when the record
/// already says"*). Where a record exists the surface block is now
/// `classify(contents)` of the column's top-of-column remainder, not a year-zero
/// climate threshold, and **grass is not expressed at all**. That moved the
/// medium goldens a second time, from:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x3764_06F3_6ABB_3A5E, 0x70B9_5244_6872_27C5, 0x1C9C_58DD_D310_B772)
/// (0x0000_0000_0000_0539, "medium", 0xDC15_2485_BCF7_EFD2, 0x13FE_3E01_AE0A_4958, 0x2AC6_F556_B746_825E)
/// ```
///
/// and it **shrank the absent-contents exception**: `block_equals_classify_of_
/// contents` now reports only Air and Stone (unrecorded basement) among
/// contents-less blocks. The 81 920 veneer-stub Dirt voxels it used to excuse
/// carry real contents.
///
/// The previous values, from the carry-`H` slice, kept so the move is auditable:
///
/// ```text
/// (0x0000_0D5E_ED57_2026, "medium", 0x5863_4C80_C5C1_4D62, 0x9B67_5DAB_C68D_BDBC, 0xBD39_3CF4_10DF_8067)
/// (0x0000_0000_0000_0539, "medium", 0x54CD_8922_B21E_852A, 0x89F9_9D67_61D2_B38C, 0x4123_D801_8035_82D8)
/// (0x0000_00C1_1A7E_2026, "small",  0x024F_5F94_8C2E_39CC, 0x3222_7B87_48CB_0F75, 0xD0A3_9718_6727_310C)
/// ```
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
// **Moved 2026-07-23 by the block↔material collapse (journal/0087) —
// authorized.** Only the **block** hashes moved, and only for the two medium
// worlds; the **material and table hashes are byte-identical** to the previous
// capture, and the record-less **small world did not move at all**. That is the
// exact signature the collapse predicts: `Block` is now `{ Air, Material(id),
// legacy }` and a buried voxel's block is the dithered member's own material
// (`classify` of the very contents interned) rather than the class's block twin
// — so the block-tier bytes change while the material sidecar the blocks
// summarize is untouched. The `block == classify(contents)` invariant now holds
// at the MEMBER level (it held only at the class level before, masked by
// `block_twin`), which is what surfaced and fixed the buried-voxel member
// divergence in `collapse::generate_chunk`. Both invariant tests
// (`block_equals_classify_of_contents`, the mixture-table round trip) green.
//
// Previous values (surface-branch-removal tree, journal/0074), kept auditable:
//   medium 0x0D5EED572026 blocks 0x4A36838BA76E3999
//   medium 0x539          blocks 0x421C2B6824DCE4F7
//
// **Moved 2026-07-25 by MFD routing (journal/0109, FLOW continuation (b)) —
// authorized, and it is a genuine PHYSICS move rather than a seam conversion.**
// Every prior move in this file came from re-plumbing how a voxel is composed;
// this one changes *where the water goes*, and routing is upstream of erosion, so
// the terrain itself moved.
//
// **The Small world is the interesting row, and it is the confirmation.** Both
// media moved on all three hashes, as expected. Small moved on **blocks only** —
// its `materials` and `table` hashes are **byte-identical** to the pre-MFD
// capture. That is precisely the signature a terrain change predicts and a record
// change forbids: Small's sampled chunks carry **no strata record**, so their
// contents are the unchanged year-zero fallback (see the doc comment above, and
// journal/0074's diagnosis of the same blind spot) — but a record-less column's
// *block* still comes from the surface height, and MFD moved the surface. So this
// is the **first** slice to move Small's block hash while leaving its materials
// untouched, and that asymmetry is independent evidence that what changed is the
// landscape and not the archive. Prior values, kept auditable:
//
//   medium 0x0D5EED572026  blocks 0x46250D80CAB250CA  materials 0xECBD087F1C41B0F3  table 0xE0255C0E5E6BBBBC
//   medium 0x539           blocks 0x11C26F4F7FDC8E14  materials 0x7A269D592F98C6C4  table 0xC3B92A0BBCC37030
//   small  0xC11A7E2026    blocks 0x83A4FD2811CBA19D  (materials/table unmoved)
//
// **Moved 2026-07-26 by material-aware transport (journal/0110, Movement 2b) —
// authorized, and a physics move of the same class as MFD above.** The suspended
// load became a multiset of `(lithology, quantity)`; deposition became a falling
// **competence ceiling**, so a grain is set down where the flow stops being able
// to hold it rather than where the mass budget happens to overflow; and a recorded
// unit now carries **the material that arrived** instead of the one its
// environment implied. Terrain moved (the ceiling changes where mass lands) and
// the archive moved with it.
//
// **The Small row reads the opposite way to MFD's, and that is the confirmation
// this time.** Small moved on **blocks only** again — `materials` and `table` are
// byte-identical — which says the same thing it said for MFD: Small's sampled
// chunks carry no strata record, so their contents are the unchanged year-zero
// fallback, while their block still follows a surface that moved. Both media moved
// on all three. Prior values, kept auditable:
//
//   medium 0x0D5EED572026  blocks 0x324B5794DE970CE4  materials 0x066A7463EA779B1D  table 0x7AE04323CEE0EDAA
//   medium 0x539           blocks 0xF071D5DC154FB2D4  materials 0xC1AA830057B948FA  table 0xFC5028203EFCF9D1
//   small  0xC11A7E2026    blocks 0x6D12F2FC4240638D  (materials/table unmoved)
//
// **Moved 2026-07-26 by material-aware hillslope CREEP (journal/0112, Movement 2b
// continuation (b)) — authorized, and the largest-authority physics move of the
// three above.** journal/0110 gave the *rivers* an identity and measured the effect
// at 0.000006 % of the archive, because fluvial transport is 0.109 % of this
// world's sediment routing (corrections #55). Hillslope creep moves 918x more, and
// it carried no identity at all. Now it does: every diffusive edge moves the
// donor's whole near-surface composition, unsorted, and the receiving cell records
// what actually came down the slope.
//
// **The Small row reads the same way it did for MFD and 2b** — blocks only, with
// `materials` and `table` byte-identical — for the same reason: Small's sampled
// chunks carry no strata record, so their contents are the unchanged year-zero
// fallback while their block follows a surface that moved. Both media moved on all
// three, and this time the archive move is the *point* rather than the side effect:
// the recorded composition went 91 % fine clastic to 26 % fine / 36 % coarse / 38 %
// carbonaceous soil, because a hillslope no longer records "mud, because this is a
// quiet place" but whatever crept down onto it.
//
// The same commit also carries **corrections #57** (`Litho::as_deposited` now refuses
// to file a *moved* peat, coal or charcoal as the in-place product it cannot be), so
// these hashes fold two changes; the intermediate state was never a shipped world.
// Prior values, kept auditable:
//
//   medium 0x0D5EED572026  blocks 0xABAFD31A268A41C9  materials 0xF55E9B53E7707DC9  table 0x1C1E2105F9B70B4B
//   medium 0x539           blocks 0x3CBE180104171798  materials 0xA84E2386F89C48C3  table 0xE7B2211FF942A2D2
//   small  0xC11A7E2026    blocks 0xDB4D1F79DE34197C  (materials/table unmoved)
//
// **Moved again 2026-07-26 by hybrid `p` (journal/0113)** — the MFD convergence
// exponent became spatially varying, so the drainage network concentrates
// (shipped peak catchment 84 → 265 cells) and the world moved with it, exactly as
// it did for MFD itself. The Small row reads the same way it always has — blocks
// only, `materials`/`table` byte-identical — for the same structural reason: its
// sampled chunks carry no strata record. Prior values, kept auditable:
//
//   medium 0x0D5EED572026  blocks 0xEF92F1C63DFDD5E3  materials 0x5DDE4337F3524E35  table 0x04F98B720BBCFC5F
//   medium 0x539           blocks 0xEE7C48C9609583E9  materials 0x78A554C9B2377421  table 0x03A23041441FD346
//   small  0xC11A7E2026    blocks 0x24F1B1491662C29D  (materials/table unmoved)
const GOLDENS: [(u64, &str, u64, u64, u64); 3] = [
    (
        0x0000_0D5E_ED57_2026,
        "medium",
        0xE19B_A53B_71A3_1DB4,
        0xD5BA_01C4_97D7_870B,
        0xC4E4_49ED_EF97_5AAC,
    ),
    (
        0x0000_0000_0000_0539,
        "medium",
        0x5B7C_3AFB_9E85_5E98,
        0x0C62_936D_AD80_AFA4,
        0x091C_7E29_2E6B_A539,
    ),
    (
        0x0000_00C1_1A7E_2026,
        "small",
        0x4B40_7E53_AB7D_DCDC,
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
    // `DC_PRINT_GOLDENS` prints, and *only* prints. It must never suppress the
    // assertion: an env var that can silently disable a correctness gate is the
    // corrections #27 false-green in miniature, and it bit us — an exploratory
    // run with the flag set reported this suite green against stale goldens
    // (journal/0058). The escape hatch was redundant anyway: the failure message
    // below already carries the got-vs-want values a golden update needs.
    assert!(
        mismatches.is_empty(),
        "the generated world moved — if that is intended, a journal entry must \
         authorize it and these become the new goldens:\n{}",
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
                            *absent.entry(block.ordinal()).or_default() += 1;
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
        Block::Air.ordinal(),
        Block::Stone.ordinal(),
        Block::Dirt.ordinal(),
        Block::Grass.ordinal(),
        Block::Wood.ordinal(),
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
