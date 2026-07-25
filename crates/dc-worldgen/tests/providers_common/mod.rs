//! Shared fixtures for the provider-seam suites (journal/0060, 0061, 0063):
//! the seed, the pre-slice goldens, and the deterministic fingerprint.
//!
//! Not a test target — a directory under `tests/` with no `main.rs` is not
//! auto-discovered by cargo, so this compiles once into each suite that
//! declares `mod providers_common;`.
//!
//! It lives apart from the suites for the same reason the source module was
//! split: a seam conversion must never have cause to edit the file that holds
//! the byte-identity proof. The goldens are constants here, and
//! `providers_golden.rs` is the only place they are asserted.

// Each suite uses a different subset of these helpers; the module is compiled
// once per suite, so anything a given suite does not call is dead there.
#![allow(dead_code)]

use dc_worldgen::deeptime::{
    Aridity, Biofacies, DeepField, DepEnv, EnergyBand, Eolian, build_field,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The seed the goldens were captured at.
pub const SEED: u64 = 0x0B0A_57EE_0059;

/// FNV-1a-64 over the surface planes of a production `DeepField`.
///
/// **Moved 2026-07-22 by the share-weighted susceptibility blend (journal/0072) —
/// authorized (audit site A1).** Erosion stopped mapping the outcrop *verdict* to
/// one susceptibility-table entry and now blends the table by the near-surface
/// window's per-`Litho` **shares** (the `outcrop_shares` seam), so the per-agent
/// erosion rate field is continuous where the old argmax stepped at the plurality
/// crossover (the walk-0071 S-4 flag). The rates move the erosion *input*, so both
/// the surface planes and the strata record moved. Prior values — set by
/// journal/0068's thickness-dominance rule — kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x344C_89FF_023F_7BAE
/// GOLDEN_RECORD  0xEA71_458F_0AB9_7A05
/// ```
///
/// See `providers_golden.rs` to re-derive.
///
/// **`GOLDEN_SURFACE` did NOT move with the geotherm (journal/0093)** — the first
/// field pass touches no surface plane (`surf`/`regolith`/drainage/`exhum`/
/// `t_crust`), so this value still equals pre-slice `main`. That the surface holds
/// while the record moves is the independent check that the geotherm changed coal
/// and nothing else.
///
/// **Moved 2026-07-25 by MFD routing (journal/0109) — authorized, and this one is
/// a genuine physics change rather than a seam conversion.** Every prior move in
/// this file came from a rate or a tag; this one changes *where the water goes*, so
/// the surface, the drainage export and the record all moved together. The
/// single-receiver world is still reachable and still hashed — see
/// [`GOLDEN_SURFACE_SINGLE_RECEIVER`], asserted by name in `tests/mfd_routing.rs`,
/// which is what makes this an authorized move rather than a lost fixed point.
/// Prior value (pre-MFD `main`), kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x176D_40F1_1CCB_006A
/// ```
pub const GOLDEN_SURFACE: u64 = 0x176D_40F1_1CCB_006A;

/// **The pre-MFD fixed point, still reachable.** The same fixture world built with
/// [`DeepConfig::mfd`](dc_worldgen::deeptime::DeepConfig) **off** must reproduce
/// the goldens as they stood before FLOW continuation (b) (journal/0109).
///
/// This is what turns "MFD moved the world" from a lost fixed point into a
/// *declared* one: the old solve is a second path, not a deleted path, and it is
/// proven byte-identical rather than assumed to be. Asserted in
/// `tests/mfd_routing.rs::the_single_receiver_path_still_hashes_to_the_pre_mfd_goldens`
/// — deliberately **not** in `providers_golden.rs`, which stays the cross-commit
/// golden for the *shipped* configuration and nothing else.
pub const GOLDEN_SURFACE_SINGLE_RECEIVER: u64 = 0x176D_40F1_1CCB_006A;
/// The strata-record half of [`GOLDEN_SURFACE_SINGLE_RECEIVER`].
pub const GOLDEN_RECORD_SINGLE_RECEIVER: u64 = 0x4A20_745B_3879_7C8A;
/// FNV-1a-64 over the strata record of the same field.
///
/// **Moved 2026-07-24 by the geotherm (journal/0093) — authorized.** The first
/// §5 field pass retired the degenerate `burial_temp_c` provider and recalibrated
/// coalification onto a real geotherm temperature (`COAL_ONSET_C` 8 → 22 °C), so
/// the record's Coal/Peat tags moved (coal follows warm crust now, ~60 % of the
/// Small world's peat candidates vs the old 8 m rule's ~12 %). This is an
/// intentional world change, not a byte-identity regression — which is why only
/// the *record* hash moved and [`GOLDEN_SURFACE`] held. Prior value (journal/0072
/// share-weighted blend), kept for audit:
///
/// ```text
/// GOLDEN_RECORD 0xC9C6_D6F6_E908_9653
/// ```
///
/// **Moved 2026-07-25 by MFD routing (journal/0109) — authorized.** See
/// [`GOLDEN_SURFACE`]. Prior value (pre-MFD `main`), kept for audit and still
/// asserted under `mfd: false` as [`GOLDEN_RECORD_SINGLE_RECEIVER`]:
///
/// ```text
/// GOLDEN_RECORD 0x4A20_745B_3879_7C8A
/// ```
pub const GOLDEN_RECORD: u64 = 0x4A20_745B_3879_7C8A;

// ---------------------------------------------------------------------------
// A deterministic fingerprint (FNV-1a 64), written by hand so it depends on
// nothing but the bytes: no `#[derive(Hash)]` discriminant encoding, no
// `DefaultHasher` (whose output is explicitly not stable across releases).

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
    fn f64(&mut self, v: f64) {
        // `to_bits` — a bit-for-bit read, so a 1-ULP drift cannot hide.
        self.u64(v.to_bits());
    }
    fn i32(&mut self, v: i32) {
        self.u64(v as u32 as u64);
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
}

// Explicit enum codes rather than derived `Hash`: the derived encoding is a
// rustc implementation detail, and a golden that silently changes with the
// toolchain is not a golden.
fn env_code(e: DepEnv) -> u8 {
    match e {
        DepEnv::Subaerial => 0,
        DepEnv::Subsea => 1,
    }
}
fn aridity_code(a: Aridity) -> u8 {
    match a {
        Aridity::Arid => 0,
        Aridity::Humid => 1,
    }
}
fn energy_code(e: EnergyBand) -> u8 {
    match e {
        EnergyBand::Low => 0,
        EnergyBand::Medium => 1,
        EnergyBand::High => 2,
    }
}
fn biota_code(b: Biofacies) -> u8 {
    match b {
        Biofacies::Mineral => 0,
        Biofacies::Soil => 1,
        Biofacies::Peat => 2,
        Biofacies::Coal => 3,
        Biofacies::Charcoal => 4,
        Biofacies::Retro => 5,
    }
}
fn eolian_code(e: Eolian) -> u8 {
    match e {
        Eolian::None => 0,
        Eolian::Loess => 1,
        Eolian::Dune => 2,
    }
}

/// Every plane the world *keeps* except the record: elevation, regolith, the
/// drainage export, exhumation/crust, and the shape metadata.
pub fn surface_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.w);
    h.usize(f.wp);
    h.f64(f.cell_m);
    h.usize(f.surf.len());
    for v in &f.surf {
        h.f64(*v);
    }
    h.usize(f.regolith.len());
    for v in &f.regolith {
        h.f64(*v);
    }
    h.usize(f.recv.len());
    for v in &f.recv {
        h.i32(*v);
    }
    for v in &f.area {
        h.f64(*v);
    }
    for v in &f.lake {
        h.byte(u8::from(*v));
    }
    h.usize(f.exhum.len());
    for v in &f.exhum {
        h.f64(*v);
    }
    for v in &f.t_crust {
        h.f64(*v);
    }
    h.usize(f.chapters.len());
    h.0
}

/// The tagged deposition log, unit by unit: every tag axis, thickness bits,
/// unconformity flag, chapter, and the per-cell strip count.
pub fn record_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.strata.len());
    for s in &f.strata {
        h.usize(s.units.len());
        h.u64(u64::from(s.strips));
        for u in &s.units {
            h.byte(env_code(u.tag.env));
            h.byte(aridity_code(u.tag.aridity));
            h.byte(energy_code(u.tag.energy));
            h.byte(biota_code(u.tag.biota));
            h.byte(eolian_code(u.tag.eolian));
            h.f64(u.thickness_m);
            h.byte(u8::from(u.unconformity));
            h.byte(u.chapter);
        }
    }
    h.0
}

/// The **golden fixture** world at [`SEED`], `Extent::Small`, built under the
/// production *config* — the world the goldens describe.
///
/// **NOT the world `dc-client` boots** (that is seed `1337` at `Extent::Medium`;
/// see `tests/geotherm.rs`). The name is historical and the distinction is
/// load-bearing: **corrections #51** records a coal magnitude claim that went
/// unfalsified for a day because an identically-named helper was mistaken for the
/// shipped world. What lives here is legitimately seed-independent — byte-identity
/// and derived-vs-scalar agreement hold on *any* fixed world — so this fixture is
/// sound for the goldens and **must not be used to accept a magnitude, a count, or
/// any claim about what a player will find**. `production_*` should be renamed
/// `golden_*`; that rename ripples into `providers_golden.rs` and the comments in
/// `flux_record.rs` / `head_field.rs`, so it is sequenced rather than done here.
pub fn production_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    build_field(&pregen.grid, SEED)
}

/// The pregen grid the deep run is built from, for suites that need to swap a
/// provider into `production_config` rather than take the default world.
pub fn production_pregen() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}
