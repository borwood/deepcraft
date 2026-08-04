//! TEMPORARY capture instrument for the deposition-clock slice (O-2b, ruled
//! 2026-08-04) — DELETED in the same arc, before the slice merges beyond it.
//!
//! Proves, on the golden fixture, the two identity claims the re-capture rests
//! on, in their strongest available form:
//!
//! 1. **The record is byte-identical apart from the new axis**: hashing the
//!    NEW world through the OLD fingerprint shape (no epoch byte) reproduces
//!    the pre-clock `GOLDEN_RECORD` constant exactly — unit counts, tags,
//!    thicknesses, chapters, species, strips, all of it.
//! 2. **The unit count is identical**, printed for the report.

mod providers_common;
use providers_common::golden_field;

/// The record fingerprint EXACTLY as it stood before the epoch byte joined
/// (pre-2026-08-04 shape) — a frozen copy, used once to prove identity.
fn record_fingerprint_pre_clock(f: &dc_worldgen::deeptime::DeepField) -> u64 {
    use dc_worldgen::deeptime::{Aridity, Biofacies, DepEnv, EnergyBand, Eolian};
    struct Fnv(u64);
    impl Fnv {
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
            self.u64(v.to_bits());
        }
        fn usize(&mut self, v: usize) {
            self.u64(v as u64);
        }
    }
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    h.usize(f.strata.len());
    for s in &f.strata {
        h.usize(s.units.len());
        h.u64(u64::from(s.strips));
        for u in &s.units {
            h.byte(match u.tag().env {
                DepEnv::Subaerial => 0,
                DepEnv::Subsea => 1,
            });
            h.byte(match u.tag().aridity {
                Aridity::Arid => 0,
                Aridity::Humid => 1,
            });
            h.byte(match u.tag().energy {
                EnergyBand::Low => 0,
                EnergyBand::Medium => 1,
                EnergyBand::High => 2,
            });
            h.byte(match u.tag().biota {
                Biofacies::Mineral => 0,
                Biofacies::Soil => 1,
                Biofacies::Peat => 2,
                Biofacies::Coal => 3,
                Biofacies::Charcoal => 4,
                Biofacies::Retro => 5,
            });
            h.byte(match u.tag().eolian {
                Eolian::None => 0,
                Eolian::Loess => 1,
                Eolian::Dune => 2,
            });
            h.f64(u.thickness_m());
            h.byte(u8::from(u.unconformity()));
            h.byte(u.chapter());
            h.byte(u.species().raw());
        }
    }
    h.0
}

/// The pre-clock `GOLDEN_RECORD` (captured 2026-08-03, P11 slice 3).
const PRE_CLOCK_GOLDEN_RECORD: u64 = 0x84E4_2349_B872_6684;

#[test]
fn the_clock_changed_no_byte_of_the_record_but_its_own_axis() {
    let f = golden_field();
    let legacy = record_fingerprint_pre_clock(&f);
    let units: usize = f.strata.iter().map(|s| s.units.len()).sum();
    let new = providers_common::record_fingerprint(&f);
    let surface = providers_common::surface_fingerprint(&f);
    println!("legacy-shape record hash = {legacy:#018X}");
    println!("new record hash          = {new:#018X}");
    println!("surface hash             = {surface:#018X}");
    println!("total units              = {units}");
    let epochs: std::collections::BTreeSet<u8> = f
        .strata
        .iter()
        .flat_map(|s| s.units.iter().map(|u| u.epoch()))
        .collect();
    println!(
        "distinct epochs recorded  = {} (min {:?}, max {:?})",
        epochs.len(),
        epochs.iter().next(),
        epochs.iter().next_back()
    );
    assert_eq!(
        legacy, PRE_CLOCK_GOLDEN_RECORD,
        "the record moved in a hashed byte OTHER than the epoch axis"
    );
}
