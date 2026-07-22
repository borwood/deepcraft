//! **The provider set itself** — `Providers::default()`, and the identity
//! report. Slot-specific falsifiers live in `providers_<slot>.rs`; the
//! byte-identity goldens live in `providers_golden.rs`.

use dc_worldgen::deeptime::providers::{ParentCell, Providers, Slot, WaveCell};

/// `Providers::default()` is the identity set: every slot is `None`, so every
/// accessor runs exactly the computation that was inlined at the call site
/// before the slice.
#[test]
fn the_default_providers_are_the_identity_functions() {
    let p = Providers::default();
    // "No slot has been supplied", stated as a field check.
    assert!(p.is_identity());
    // `wave_energy`'s identity returns the config rate unchanged, whatever it is.
    for rate in [0.0, 0.05, 1.0, 12.5] {
        let cell = WaveCell {
            index: 7,
            gx: 3,
            gy: 4,
            base_rate: rate,
        };
        assert_eq!(p.wave_energy(cell).to_bits(), rate.to_bits());
    }
    // And `parent_p`'s identity is the uniform initial rock-P pool.
    for index in [0usize, 1, 999] {
        let cell = ParentCell {
            index,
            gx: index % 32,
            gy: index / 32,
        };
        assert_eq!(p.parent_p(cell), 1.0);
    }
}

/// **The corrections #32 configuration.**
///
/// #32 was: `Providers::default().is_identity()` returned **false**, naming
/// `outcrop_at`, because the check compared `fn` addresses and an `#[inline]`
/// identity had been instantiated at two addresses in two codegen units. On
/// `main` the same code was green only because the optimizer could see both
/// sides of the comparison at once and fold the answer — the assertion was
/// passing for a reason unrelated to what it asserted.
///
/// This test reproduces exactly the conditions that broke it and that hid it:
/// the set is built **in another crate** (an integration test is its own
/// compilation unit, which is where the two addresses came from) and handed
/// through [`std::hint::black_box`] behind an `#[inline(never)]` boundary, so
/// nothing about the answer can be constant-folded at the assertion.
///
/// Under the address comparison this is the shape that fails. Under
/// `Option<fn>` it cannot: see the module note in `providers/mod.rs` — the
/// answer is `is_some()` on four fields, and an optimizer that folds it folds
/// it to the *right* value because there is no address in the expression.
#[test]
fn a_default_set_that_cannot_be_constant_folded_still_reports_no_slots() {
    #[inline(never)]
    fn opaque_default() -> Providers {
        std::hint::black_box(Providers::default())
    }
    let a = opaque_default();
    let b = opaque_default();
    assert!(
        std::hint::black_box(&a).is_identity(),
        "an opaquely-built default set reported: {:?}",
        a.non_identity_slots()
    );
    assert!(std::hint::black_box(&b).is_identity());
    for slot in Slot::ALL {
        assert!(
            !a.is_supplied(*slot) && !b.is_supplied(*slot),
            "slot {slot} reported as supplied on a default set"
        );
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
        "wave_energy,depth_to_water,parent_p,outcrop_at,burial_temp_c",
        "Slot::ALL must enumerate every field of Providers, in field order"
    );
}
