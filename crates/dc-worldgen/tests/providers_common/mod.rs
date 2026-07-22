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

/// FNV-1a-64 over the surface planes of a production `DeepField`, captured from
/// pre-slice `main` (`2434f37`). See `providers_golden.rs` to re-derive.
pub const GOLDEN_SURFACE: u64 = 0x7B89_68FD_90E0_4062;
/// FNV-1a-64 over the strata record of the same field, same provenance.
pub const GOLDEN_RECORD: u64 = 0xA53B_D77F_769D_7FF4;

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

/// The production deep-time world at [`SEED`], `Extent::Small` — the world the
/// goldens describe.
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
