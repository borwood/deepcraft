//! **A tripwire on every shipped world artifact** — the sweep journal/0124 filed
//! and did not run, plus the expression-side twin journal/0125 found the same
//! evening from the other side of the collapse tier.
//!
//! That entry added `GOLDEN_FLUX` because the `Schedule` slice moved the flow
//! record and a full 834-test gate said nothing: the record is a pure sidecar to
//! both existing goldens (`GOLDEN_SURFACE` = terrain, `GOLDEN_RECORD` = strata),
//! so neither could ever see it. The generalisation it left open is the reason
//! this file exists:
//!
//! > An artifact the ritual ships with no tripwire on it cannot have an
//! > **authorized** move, because nobody can see it move.
//!
//! The whole golden discipline rests on telling an *explained* move from an
//! *unexplained* one — *"the testing world is a scratch pad"* (CLAUDE.md §
//! Conventions): a hash move produced by ratified semantics is re-captured with
//! the **why** recorded on the constant, and an unexplained one is a defect to
//! chase. That distinction is simply unavailable for anything ungoverned.
//!
//! ## The enumeration, taken from the struct rather than from memory
//!
//! Every member of [`DeepField`], classified. The classification is **enforced by
//! the compiler**, not by this comment — see
//! [`every_deepfield_member_is_classified`], whose exhaustive destructure stops
//! compiling the day a member is added.
//!
//! | member | classification | evidence |
//! |---|---|---|
//! | `w`, `wp`, `cell_m` | COVERED | hashed by `surface_fingerprint` (the shape header) |
//! | `surf`, `regolith` | COVERED | `GOLDEN_SURFACE` |
//! | `strata` | COVERED | `GOLDEN_RECORD` |
//! | `recv`, `area`, `lake` | COVERED | `surface_fingerprint` (the drainage export) |
//! | `exhum`, `t_crust` | COVERED | `surface_fingerprint` — **not** uncovered, contrary to the candidate list the sweep was filed with |
//! | `flux` | COVERED | `GOLDEN_FLUX` (`tests/flux_record.rs`, journal/0124) — on *that* suite's fixture, not this one |
//! | `geotherm` | **UNCOVERED → [`GOLDEN_GEOTHERM`]** | its own doc says *"Not part of the surface fingerprint"* |
//! | `head` | **UNCOVERED → [`GOLDEN_HEAD`]** | same sentence, one field newer |
//! | `chapters` | **UNCOVERED → [`GOLDEN_CHAPTERS`]** | `surface_fingerprint` hashes `chapters.len()` and nothing inside it |
//! | `ledgers` | NOT SHIPPED | empty in every world a player gets (`weather_inventory` is off); the off-state is tripwired by `s18_first_behavior_weathering.rs::identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical`, and re-asserted here as part of the enumeration. **The commit that flips the flag on owes a `GOLDEN_LEDGER`** — that is the whole content of "not shipped, so not goldened" |
//!
//! And one row that is not a `DeepField` member at all:
//!
//! | artifact | classification | evidence |
//! |---|---|---|
//! | `WorldGenerator::coarse_surface` — **the far field**, every metre of ground beyond the loaded radius | **UNCOVERED AUTHORITY → [`GOLDEN_FAR_SURFACE`]** | expression-side, and found from the other end: journal/0125's member-#0 slice changed the far draw twice over and **no hash in the workspace moved**. `contents_contract` hashes `generate_chunk_with_materials`, which has not consulted `surface_class` since journal/0074; the deep-time goldens sit upstream of the collapse tier entirely |
//!
//! **The two halves are one finding.** journal/0124 discovered the record side
//! (a shipped artifact moved, and the gate was structurally blind); journal/0125
//! discovered the expression side (an acceptance criterion that could not fire).
//! They were filed as separate ROADMAP entries by separate agents on the same
//! evening, and they close here together, because the sentence is the same one:
//! *what is not fingerprinted cannot have an authorized move.*
//!
//! ## One fixture, one build
//!
//! Every assertion reads the **same** fixture the existing goldens were captured
//! on — `providers_common`'s (seed `0x0B0A_57EE_0059`, `Extent::Small`) — built
//! **once** for the suite. Consistency with the incumbent goldens beats
//! cleverness: a move that trips `GOLDEN_SURFACE` and `GOLDEN_HEAD` together is
//! then a statement about one world rather than a coincidence across two.
//!
//! The far-field tripwire shares that fixture: it builds a [`WorldGenerator`] over
//! the *same* `Pregen` the field is distilled from, so no extra world is generated
//! and a move that trips both is one world's move.
//!
//! ## One hash per artifact, deliberately
//!
//! Not one blob hash over all four. A blob would say *"something moved"*; the
//! author of the move then has to bisect the artifact by hand, which is precisely
//! the throwaway-harness work journal/0124 had to do. **A move must name its
//! artifact.**

mod providers_common;

use dc_core::materials::geology;
use dc_worldgen::WorldGenerator;
use dc_worldgen::deeptime::{DeepField, Plate, build_field};
use dc_worldgen::pregen::Pregen;
use providers_common::{SEED, golden_pregen};
use std::sync::OnceLock;

/// The fixture's pregen, built once for the whole suite. World builds are the
/// expensive part of this gate; the assertions are microseconds.
///
/// Held rather than discarded because the far-field tripwire needs a
/// [`WorldGenerator`], which needs a `Pregen`. `providers_common::golden_field`
/// is *literally* `Pregen::run` followed by `build_field` — this pair is that
/// function with the intermediate kept, so the record-side constants are
/// reproducible by anyone who calls `golden_field()` and no extra world is built.
fn pregen() -> &'static Pregen {
    static PREGEN: OnceLock<Pregen> = OnceLock::new();
    PREGEN.get_or_init(golden_pregen)
}

/// The fixture field — `golden_field()`'s second half, over [`pregen`].
fn field() -> &'static DeepField {
    static FIELD: OnceLock<DeepField> = OnceLock::new();
    FIELD.get_or_init(|| build_field(&pregen().grid, SEED))
}

// ---------------------------------------------------------------------------
// The fingerprint. FNV-1a-64 by hand, for the reason `providers_common` states:
// it depends on nothing but the bytes — no derived `Hash` discriminant encoding
// (a rustc implementation detail) and no `DefaultHasher` (explicitly unstable
// across releases).

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.byte(b);
        }
    }
    /// `to_bits` — a bit-for-bit read, so a 1-ULP drift cannot hide.
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
    fn i32(&mut self, v: i32) {
        self.u64(u64::from(v as u32));
    }
}

/// **A condition-field plane**, cell by cell in the plane's own row-major order —
/// used for both `geotherm` and `head`.
///
/// *Normalization: none needed and none applied.* The plane is a `Vec<f64>`
/// indexed by cell, so its iteration order is the storage order — deterministic
/// by construction. (A golden that depended on a `HashMap`'s order would be a
/// flaky tripwire, which is worse than no tripwire.) The length is hashed first,
/// so a truncation cannot alias a value change.
///
/// The two planes share this function and still get **separate constants**: what
/// must be per-artifact is the *hash you compare against*, not the code that
/// computes it — a shared computation is one place for a bug, two constants are
/// two named failures.
fn plane_fingerprint(plane: &[f64]) -> u64 {
    let mut h = Fnv::new();
    h.usize(plane.len());
    for v in plane {
        h.f64(*v);
    }
    h.0
}

/// **The chapter table** — plate state per chapter, `Vec<Vec<Plate>>`.
///
/// Both levels are `Vec`s in construction order (chapter index, then plate
/// index), so again no normalization is required. Every level's length is hashed
/// before its contents, so a plate lost from chapter 3 cannot be masked by one
/// gained in chapter 4. `continental` is hashed as an explicit `0`/`1` byte
/// rather than through a derived encoding, for the reason `providers_common`
/// gives: the derived encoding is a rustc implementation detail, and a golden
/// that silently changes with the toolchain is not a golden.
fn chapters_fingerprint(chapters: &[Vec<Plate>]) -> u64 {
    let mut h = Fnv::new();
    h.usize(chapters.len());
    for chapter in chapters {
        h.usize(chapter.len());
        for p in chapter {
            h.f64(p.x);
            h.f64(p.y);
            h.f64(p.vx);
            h.f64(p.vy);
            h.byte(u8::from(p.continental));
        }
    }
    h.0
}

// ---------------------------------------------------------------------------
// The far field. Expression-side, and the only member of this suite that is not
// a `DeepField` member — see the module doc's seventeenth row.

/// Sample stride, in world voxels. **Deliberately not a power of two.** The
/// coarse cell is `CELL_VOXELS = 16384` and the chunk is 32, so any power-of-two
/// stride lands every sample on the same phase of both lattices and a golden
/// built from it would be blind to exactly the seam artifacts the far field is
/// prone to. `2039` is prime.
const FAR_STRIDE: i64 = 509;
/// Samples per axis, centred on the origin: `192² = 36,864` columns spanning
/// ±48,864 voxels ≈ **±44 km**. The `Extent::Small` fixture is 5 × 16,384 =
/// 81,920 voxels across (±36.9 km), so the sample covers the whole civilized
/// extent **and reaches ~7 km into the border wilds on every side** — where the
/// pyramid runs forever and the record does not. A far-field golden that stopped
/// at the civilized edge would not be hashing the half of this artifact that has
/// no other instrument at all.
///
/// **The density was measured, not guessed, and the measurement matters** — see
/// [`GOLDEN_FAR_SURFACE`] § *what this sample does and does not exercise*. A first
/// draft used stride 2039 × 48 (2,304 columns) and caught **six** columns fronting
/// anything but the ocean block, on a fixture that is overwhelmingly sea. Denser
/// and finer-strided reaches 160 — still thin, but no longer a sample that could
/// miss a surface-class change by phase luck alone.
const FAR_SAMPLES: i64 = 192;

/// **The far field** — `(height, block)` at a fixed strided sample of world
/// columns, in a fixed scan order.
///
/// *Normalization: the sample IS the normalization.* `coarse_surface` is a query,
/// not a container, so there is no stored order to inherit — the loop order below
/// is the canonical one, and it is row-major over a closed-form coordinate set
/// with no iteration over any map. The height is an `i32` and the block is hashed
/// as its `ordinal()`, the same stable encoding `contents_contract` uses, rather
/// than through a derived `Hash`.
fn far_surface_fingerprint(g: &mut WorldGenerator<'_>) -> u64 {
    let mut h = Fnv::new();
    let half = FAR_SAMPLES / 2;
    for j in 0..FAR_SAMPLES {
        for i in 0..FAR_SAMPLES {
            let (vx, vz) = ((i - half) * FAR_STRIDE, (j - half) * FAR_STRIDE);
            let (height, block) = g.coarse_surface(vx, vz);
            h.i32(height);
            h.u64(u64::from(block.ordinal()));
        }
    }
    h.0
}

// ---------------------------------------------------------------------------
// The constants.

/// FNV-1a-64 over the exported `geotherm` plane of the golden fixture.
///
/// **Captured 2026-07-29 (journal/0126) — a first capture, not a move.** The
/// plane has been exported and populated in every production world since
/// journal/0093 and was never hashed across commits: `surface_fingerprint` does
/// not touch it (its own doc comment says so, in the sentence that added it), and
/// the two byte-identity guards that *do* read it — `flux_record.rs` and
/// `head_field.rs` — assert it is unmoved by **their own flag**, which is a
/// different claim. A pass could have changed the geothermal gradient of the whole
/// world without a single test noticing.
///
/// **What an authorized re-capture owes.** The gradient is planted by
/// `dc:deep/geotherm` off the crustal state, so this constant moves whenever the
/// crust moves — but `exhum`/`t_crust` are inside `GOLDEN_SURFACE`, so a move
/// *here* with `GOLDEN_SURFACE` **held** is the interesting case: it means the
/// geotherm **rule** changed, not the world under it. State which, name the
/// mechanism, keep the prior value below in the house style, and say what it did
/// to coal rank — that is this plane's only in-run consumer (`biotic.rs`, at
/// finalize).
///
/// Captured over **25,600 cells** (the fixture's 160² deep grid).
/// **Moved 2026-08-01 by P11 slice 1** — the deep record went member-grade
/// (journal/pending-p11-slice1). See `providers_common` § P11 for the two
/// mechanisms and for why a moved *surface* hash here is rounding rather than a
/// re-tuned world. Prior value, kept for audit: `0xDBDE_C405_EBE0_239E`.
/// **The geotherm RULE did not change** — this constant tracks `t_crust`, which
/// tracks exhumation, which tracks the erosion the rounding perturbed. It moved
/// *with* `GOLDEN_SURFACE`, which is the case its own doc calls uninteresting.
const GOLDEN_GEOTHERM: u64 = 0xBC6E_77CD_3D52_0246;

/// FNV-1a-64 over the exported `head` plane of the golden fixture.
///
/// **Captured 2026-07-29 (journal/0126) — a first capture, not a move.** And this
/// is the member the sweep most obviously owed: journal/0124's own slice **moved
/// the head field's epoch-0 solve**, traced the consequence into the flow record's
/// vertical faces, and reported the exported plane bit-identical — *measured with
/// a throwaway harness*, because nothing in the tree pinned it either.
///
/// **What an authorized re-capture owes.** `head` is *elevation + pressure head*,
/// not an elevation: it is relaxed against the drainage solve's `filled`/`routed`/
/// `area`, so it moves with any routing change. A move here with `GOLDEN_SURFACE`
/// **held** means the potential's own solve changed. Name the mechanism, say what
/// it did to the vertical face family (`head` is the only thing that feeds them —
/// `GOLDEN_FLUX` is the corroborating hash), and keep the prior value.
///
/// Captured over **25,600 cells** (the fixture's 160² deep grid).
/// **Moved 2026-08-01 by P11 slice 1** — the deep record went member-grade
/// (journal/pending-p11-slice1). See `providers_common` § P11 for the two
/// mechanisms and for why a moved *surface* hash here is rounding rather than a
/// re-tuned world. Prior value, kept for audit: `0x1013_984C_2B7B_BCF9`.
/// **Two causes here, and the second is a real capability gain**: the routing
/// moved with the surface, *and* `head::permeability_of` now reads the recorded
/// material's own sheet instead of its class's reference — so a siltstone aquitard
/// (k = 0.08) stops being a mudstone one (k = 0.02). The head field is the first
/// consumer in the tree to see member grade as physics rather than as albedo.
const GOLDEN_HEAD: u64 = 0x9AC3_4801_9AD6_FE1B;

/// FNV-1a-64 over the exported chapter table of the golden fixture.
///
/// **Captured 2026-07-29 (journal/0126) — a first capture, and the one the
/// enumeration nearly missed.** `surface_fingerprint` *does* hash
/// `f.chapters.len()`, which reads as coverage and is not: the number of chapters
/// is a config constant, so that byte pins the config and says nothing whatever
/// about plate positions, velocities or continentality. The table could have been
/// rebuilt from a different seed salt, advected at a different rate, or had every
/// plate flipped oceanic, and the length would not have twitched. **A hashed
/// length is not a hashed artifact** — the same trap as `has_contents` answering
/// per-chunk (corrections #49): a real number, about the wrong thing.
///
/// **What an authorized re-capture owes.** The table is a pure function of
/// `(seed, extent, plate_scale_km, advection_plate_widths, chapters)`, so it moves
/// only when the *kinematics* change — and because nothing in production reads it
/// (`spines.md` § 3), a move here can otherwise be completely silent. Say which
/// tectonic input changed and keep the prior value.
///
/// Captured over **9 chapters × 3 plates = 27 plate records** — the smallest of the
/// three artifacts by three orders of magnitude, and the one that could move
/// furthest with the least noticed. (`plate_count` floors at 3, which is where a
/// Small world lands.)
const GOLDEN_CHAPTERS: u64 = 0xF25B_E0C3_CF39_AC55;

/// FNV-1a-64 over the far field of the golden fixture — 2,304 strided columns of
/// `(height, block)` from `WorldGenerator::coarse_surface`.
///
/// **Captured 2026-07-29 (journal/0126) — a first capture, and the sharpest of the
/// four, because it was found by an acceptance criterion that COULD NOT FIRE.**
/// Member #0's far slice (journal/0125) was briefed *"goldens move — re-capture
/// with the why"*, and it changed the far surface class draw twice over: nearest
/// deep cell → `CoarseField::sample_dithered` membership dither, plus a canonical
/// class-order change. **Not one hash in the workspace moved.**
/// `generated_world_is_byte_identical_to_the_pre_contract_goldens` passed
/// untouched — correctly, because `contents_contract` hashes
/// `generate_chunk_with_materials`, and `generate_chunk` has not consulted
/// `surface_class` since journal/0074; and `providers_golden` / `rate_axis` /
/// `creep_operator` hash the deep-time planes, upstream of the collapse tier
/// entirely. Every metre of ground beyond the loaded radius was fingerprinted by
/// nothing, and the slice's gate would have been just as green had it broken the
/// far field outright.
///
/// **The reason it stayed invisible is the reason it belongs in this suite.** The
/// far field is not untested — it has a class-split floor, a near/far statistical
/// agreement test, a far-tile mesh budget. But **a behavioural test cannot notice
/// that no fingerprint exists**, and a fingerprint is the only instrument that
/// answers *"did the world change?"* rather than *"is the world still plausible?"*
/// That is the record-side lesson (journal/0124) arriving at the expression side
/// under its own power, the same evening and from a different agent — which is why
/// the two ROADMAP entries closed in one commit.
///
/// **What an authorized re-capture owes.** This constant moves whenever the far
/// draw moves — the elevation lattice, the river carving, the surface rule, the
/// class window, or the member draw beneath it. Say **which**, and say whether the
/// near field moved with it: `coarse_surface` and `column` are documented to agree
/// *by construction* (ARCHITECTURE.md § One world-answer surface), so a move here
/// with `contents_contract`'s three hashes **held** is either a far-only change —
/// legitimate, the near path has not consulted `surface_class` since
/// journal/0074 — or the two tiers drifting apart, which is a defect with a
/// good disguise. **That question is not answerable without this constant**, which
/// is the whole argument for it.
///
/// # ⚠ What this sample does and does not exercise — measured, and a caveat
///
/// The `providers_common` fixture is used for consistency with the four record-side
/// constants and because it costs no extra world build. It is also, measured while
/// this constant was captured, **an almost entirely submarine world**: over the
/// 36,864 sampled columns the surface height runs **−2,642 … −3** and only **160
/// columns (0.43 %)** front with anything but the ocean block. A coarser scan out
/// to ±56,000 voxels puts the highest ground in the sample at **+7 voxels**.
///
/// So be precise about the coverage this buys:
///
/// - **The height field is exercised completely.** Every column carries a real,
///   varying elevation — the lattice, the river carving and the pyramid are under
///   the hash everywhere, which is the far field's dominant output (it is the
///   silhouette).
/// - **The surface-CLASS draw — the thing journal/0125 actually changed — is
///   exercised thinly**, by those 160 columns. A class-order change would still
///   trip this hash, and
///   [`the_far_sample_spreads_over_a_real_world_and_into_the_wilds`] asserts the
///   sample has not degenerated to one block; but this world cannot exercise the
///   membership dither the way a land-bearing one would.
/// - **Residue, filed on the board:** a far-field golden on a fixture with real
///   continent. The natural home is `contents_contract.rs`, whose Medium seeds are
///   the worlds with land, which is exactly where the ROADMAP heir spec put it
///   before this suite took it. *Recorded rather than quietly shipped: a golden
///   whose coverage of its own motivating change is 0.43 % is a real tripwire and
///   a partial one, and calling it either without the number would be the A-3
///   shape.*
/// **Moved 2026-08-01 by P11 slice 1** — the deep record went member-grade
/// (journal/pending-p11-slice1). See `providers_common` § P11 for the two
/// mechanisms and for why a moved *surface* hash here is rounding rather than a
/// re-tuned world. Prior value, kept for audit: `0x1424_7B7C_AB51_EFA5`.
/// **This one is a real content move**: the far field samples the surface
/// *block*, and a surface that used to be `dc:mudstone` by table lookup is now
/// whichever fine clastic deposition-time fitness actually chose.
const GOLDEN_FAR_SURFACE: u64 = 0xF64E_7377_9F48_B696;

// ---------------------------------------------------------------------------
// The tripwires. One per artifact — see the module doc's "one hash per artifact".

/// The far field is a fixed point, and moving it is a decision.
#[test]
fn the_far_surface_is_a_fixed_point() {
    let mut g = WorldGenerator::with_geology(pregen(), geology::vanilla());
    let print = far_surface_fingerprint(&mut g);
    println!(
        "far surface fingerprint = {print:#018X}  ({} columns, stride {FAR_STRIDE})",
        FAR_SAMPLES * FAR_SAMPLES
    );
    assert_eq!(
        print, GOLDEN_FAR_SURFACE,
        "the far field moved: {print:#018X} != {GOLDEN_FAR_SURFACE:#018X} — \
         authorize it with a journal entry naming the mechanism, then re-capture"
    );
}

/// **The sample must actually see a world**, or the constant above is a hash of a
/// constant.
///
/// A strided sample is only as good as its spread: if every column came back at
/// the same height, or every block were the same, the fingerprint would still be
/// stable and still be worthless — the failure mode a snapshot cannot self-report.
/// So this asserts the sample carries relief and more than one surface material,
/// and that it reaches beyond the civilized extent into the wilds (which is where
/// the far field is the *only* answer, the record having none).
#[test]
fn the_far_sample_spreads_over_a_real_world_and_into_the_wilds() {
    let mut g = WorldGenerator::with_geology(pregen(), geology::vanilla());
    let half = FAR_SAMPLES / 2;
    let mut heights = Vec::new();
    let mut blocks = std::collections::BTreeSet::new();
    for j in 0..FAR_SAMPLES {
        for i in 0..FAR_SAMPLES {
            let (vx, vz) = ((i - half) * FAR_STRIDE, (j - half) * FAR_STRIDE);
            let (h, b) = g.coarse_surface(vx, vz);
            heights.push(h);
            blocks.insert(b.ordinal());
        }
    }
    let lo = *heights.iter().min().expect("the sample is not empty");
    let hi = *heights.iter().max().expect("the sample is not empty");
    println!(
        "far sample: {} columns, height {lo}..{hi}, {} distinct surface blocks",
        heights.len(),
        blocks.len()
    );
    assert!(
        hi - lo > 50,
        "the far sample is flat ({lo}..{hi}) — it is not seeing terrain"
    );
    assert!(
        blocks.len() > 1,
        "every sampled column fronts with the same block — the sample is blind to \
         the surface rule it is supposed to guard"
    );
    // The civilized extent is `cells × CELL_VOXELS` wide; the sample must overrun
    // it, because the wilds are the half of the far field with no other instrument.
    let reach = half * FAR_STRIDE;
    let civilized_half = i64::from(pregen().extent.cells()) * 16384 / 2;
    assert!(
        reach > civilized_half,
        "the sample reaches {reach} voxels but the civilized half-extent is \
         {civilized_half} — it never enters the border wilds"
    );
}

/// The `temperature` condition-field is a fixed point, and moving it is a decision.
#[test]
fn the_geotherm_plane_is_a_fixed_point() {
    let f = field();
    assert!(
        !f.geotherm.is_empty(),
        "the geotherm plane is empty — a tripwire over nothing is not a tripwire"
    );
    let print = plane_fingerprint(&f.geotherm);
    println!(
        "geotherm fingerprint = {print:#018X}  ({} cells)",
        f.geotherm.len()
    );
    assert_eq!(
        print, GOLDEN_GEOTHERM,
        "the geotherm plane moved: {print:#018X} != {GOLDEN_GEOTHERM:#018X} — \
         authorize it with a journal entry naming the mechanism, then re-capture"
    );
}

/// The `head` condition-field is a fixed point, and moving it is a decision.
#[test]
fn the_head_plane_is_a_fixed_point() {
    let f = field();
    assert!(
        !f.head.is_empty(),
        "the head plane is empty — a tripwire over nothing is not a tripwire"
    );
    let print = plane_fingerprint(&f.head);
    println!("head fingerprint = {print:#018X}  ({} cells)", f.head.len());
    assert_eq!(
        print, GOLDEN_HEAD,
        "the head plane moved: {print:#018X} != {GOLDEN_HEAD:#018X} — authorize \
         it with a journal entry naming the mechanism, then re-capture"
    );
}

/// The chapter table is a fixed point, and moving it is a decision.
#[test]
fn the_chapter_table_is_a_fixed_point() {
    let f = field();
    assert!(
        !f.chapters.is_empty(),
        "the chapter table is empty — a tripwire over nothing is not a tripwire"
    );
    let print = chapters_fingerprint(&f.chapters);
    let plates: usize = f.chapters.iter().map(Vec::len).sum();
    println!(
        "chapters fingerprint = {print:#018X}  ({} chapters, {plates} plate records)",
        f.chapters.len()
    );
    assert_eq!(
        print, GOLDEN_CHAPTERS,
        "the chapter table moved: {print:#018X} != {GOLDEN_CHAPTERS:#018X} — \
         authorize it with a journal entry naming the mechanism, then re-capture"
    );
}

/// **A hashed length is not a hashed artifact**, asserted rather than asserted-in-prose.
///
/// `surface_fingerprint` hashes `chapters.len()`, and that byte is what made the
/// chapter table *look* covered for three sweeps. This shows the two hashes see
/// different things: mutate a plate's position and [`chapters_fingerprint`] moves
/// while a length-only digest cannot. It is the falsifier for
/// [`GOLDEN_CHAPTERS`]'s own reason to exist — without it, "the new constant adds
/// coverage" would be a claim rather than a demonstration.
#[test]
fn the_chapter_length_byte_is_blind_to_what_the_chapter_table_says() {
    let f = field();
    let before = chapters_fingerprint(&f.chapters);

    // One plate, nudged one kilometre east. Chapter count, plate count and every
    // other byte of the table are untouched — so a length-only digest is
    // identical by construction, and this fingerprint must not be.
    let mut mutated = f.chapters.clone();
    mutated[0][0].x += 1.0;

    assert_eq!(
        mutated.len(),
        f.chapters.len(),
        "the mutation must leave the length alone, or it proves nothing"
    );
    assert_eq!(mutated[0].len(), f.chapters[0].len());
    assert_ne!(
        chapters_fingerprint(&mutated),
        before,
        "the chapter fingerprint is blind to a plate's position — it is hashing \
         the container, not the content"
    );
}

/// **The enumeration itself, enforced by the compiler.**
///
/// CLAUDE.md records that a wrong count inside a read-first justification is
/// *"exactly what an enumeration-completeness check would catch, and we still
/// have none"*. This is one, for the artifact inventory: the exhaustive
/// destructure below **stops compiling** the day a member is added to
/// [`DeepField`], so the next author cannot ship an unclassified artifact by
/// omission the way `flux` was shipped unguarded for four days. A prose table
/// (the module doc's) goes stale in silence; a pattern cannot.
///
/// **⚠ Its reach is exactly one struct, and the far field is the proof that
/// matters.** `coarse_surface` is a *query*, not a member, so nothing here could
/// ever have caught it — it took a different agent, from the other side of the
/// collapse tier, the same evening. A completeness check is only complete over the
/// thing it enumerates, and saying so is part of shipping one.
///
/// Each binding is then *used* in the assertion that states its classification,
/// so the check cannot be satisfied by a row of underscores.
#[test]
fn every_deepfield_member_is_classified() {
    // Adding a member to `DeepField` breaks THIS LINE. That is the whole test.
    let DeepField {
        w,
        wp,
        cell_m,
        surf,
        regolith,
        strata,
        ledgers,
        recv,
        area,
        lake,
        flux,
        exhum,
        t_crust,
        geotherm,
        head,
        chapters,
    } = field();

    // COVERED by `surface_fingerprint` — the shape header.
    assert!(*w > 0 && *wp > 0 && *cell_m > 0.0);
    // COVERED by `GOLDEN_SURFACE`.
    assert_eq!(surf.len(), w * w);
    assert_eq!(regolith.len(), w * w);
    // COVERED by `GOLDEN_RECORD`.
    assert_eq!(strata.len(), w * w);
    // COVERED by `surface_fingerprint` — the drainage export.
    assert_eq!(recv.len(), w * w);
    assert_eq!(area.len(), w * w);
    assert_eq!(lake.len(), w * w);
    // COVERED by `surface_fingerprint` — and this pair is where the candidate
    // list the sweep was filed with turned out to be wrong. They are hashed.
    assert_eq!(exhum.len(), w * w);
    assert_eq!(t_crust.len(), w * w);
    // COVERED by `GOLDEN_FLUX` (`tests/flux_record.rs`), on that suite's own
    // fixture rather than this one — deliberately not re-pinned here, because a
    // second hash of the same artifact is a second thing to re-capture, not a
    // second thing detected.
    assert!(!flux.is_empty());
    // NEWLY COVERED — the three constants above.
    assert_eq!(geotherm.len(), w * w);
    assert_eq!(head.len(), w * w);
    assert!(!chapters.is_empty());
    // NOT SHIPPED, and this is the tripwire for that fact: `weather_inventory`
    // is off in every world a player gets, so the artifact does not exist to be
    // hashed. If this assertion ever fails, the flag flipped — and the same
    // commit owes a `GOLDEN_LEDGER`, because from that moment the ledger is a
    // shipped artifact with no tripwire on it, which is the whole defect this
    // suite exists to close. (The off-state's byte-identity half is
    // `s18_first_behavior_weathering.rs`; what is added here is the enumeration's
    // claim that emptiness is *why* it needs no hash.)
    assert!(
        ledgers.is_empty(),
        "the shipped world grew a fact ledger — `weather_inventory` is on, and \
         this artifact now needs a golden of its own"
    );
}

// **No "the fingerprint is reproducible" test here, deliberately.** The obvious
// one — build the fixture twice and compare — would double this suite's only
// real cost (one deep-time world) to re-prove a claim
// `providers_golden.rs::the_fingerprint_is_reproducible_within_a_build` already
// makes on the *same* fixture. What is left to prove would be that these three
// functions are deterministic over a `&[f64]` and a `&[Vec<Plate>]`, which is
// not a thing that can fail. "Size the test, not the report" (CLAUDE.md § Gates).
