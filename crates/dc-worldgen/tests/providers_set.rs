//! **The provider set itself** — `Providers::default()`, and the identity
//! report. Slot-specific falsifiers live in `providers_<slot>.rs`; the
//! byte-identity goldens live in `providers_golden.rs`.

use dc_worldgen::deeptime::providers::{ParentCell, Providers, Slot, WaveCell};

/// `Providers::default()` is the identity set: each slot holds exactly the
/// function that was inlined at the call site before the slice.
#[test]
fn the_default_providers_are_the_identity_functions() {
    let p = Providers::default();
    // "Every slot holds its identity function", stated by address.
    assert!(p.is_identity());
    // `wave_energy`'s identity returns the config rate unchanged, whatever it is.
    for rate in [0.0, 0.05, 1.0, 12.5] {
        let cell = WaveCell {
            index: 7,
            gx: 3,
            gy: 4,
            base_rate: rate,
        };
        assert_eq!((p.wave_energy)(cell).to_bits(), rate.to_bits());
    }
    // And `parent_p`'s identity is the uniform initial rock-P pool.
    for index in [0usize, 1, 999] {
        let cell = ParentCell {
            index,
            gx: index % 32,
            gy: index / 32,
        };
        assert_eq!((p.parent_p)(cell), 1.0);
    }
}

/// The identity report is the public shape, not just the bool: a default set
/// names no slots, and every slot in the struct is reachable through
/// [`Slot::ALL`] so the report cannot silently omit one.
#[test]
fn the_identity_report_names_no_slots_for_a_default_set() {
    assert!(Providers::default().non_identity_slots().is_empty());
    assert_eq!(
        Slot::ALL
            .iter()
            .map(|s| s.name())
            .collect::<Vec<_>>()
            .join(","),
        "wave_energy,depth_to_water,parent_p,outcrop_at",
        "Slot::ALL must enumerate every field of Providers, in field order"
    );
}
