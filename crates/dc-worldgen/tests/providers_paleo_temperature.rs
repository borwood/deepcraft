//! **Slot `paleo_temperature`** — falsifiers for the at-deposition temperature
//! seam (journal/0078). The world-level byte-identity proof is the golden in
//! `providers_golden.rs` (this seam is value-level, so per journal/0060 the
//! default byte-identity test suffices there); the *consultation* proof — that
//! `deposit_deep_history` reads the provider and not `ctx.temp_c` — is the
//! white-box test in `geology.rs`. This suite pins the manifest-level
//! properties the frozen-content-set rule depends on.

use dc_worldgen::deeptime::PaleoUnit;
use dc_worldgen::deeptime::providers::{Providers, Slot, identity_paleo_temperature};

/// **`Some(identity)` is a resolution, not an absence** (journal/0064). Handing
/// the slot the very function it would have fallen back to still reports it as
/// **supplied** — the manifest answers *"did an heir answer this?"*, not *"does
/// the answer happen to equal the old one?"*. Only the first is decidable.
#[test]
fn explicitly_supplying_the_identity_paleo_temperature_still_counts_as_supplied() {
    let p = Providers {
        paleo_temperature: Some(identity_paleo_temperature),
        ..Providers::default()
    };
    assert_eq!(p.non_identity_slots(), vec![Slot::PaleoTemperature]);
    assert!(!p.is_identity());
    // …and it still answers identically, because it is the same function.
    let u = PaleoUnit {
        cx: 1,
        cz: 2,
        chapter: 4,
        present_temp_c: 13.5,
    };
    assert_eq!(
        p.paleo_temperature(u).to_bits(),
        identity_paleo_temperature(u).to_bits()
    );
}

/// The default set leaves the slot `None`, so a default world is the pre-seam
/// world — the property the golden verifies at the world level.
#[test]
fn the_default_set_does_not_supply_paleo_temperature() {
    let p = Providers::default();
    assert!(!p.non_identity_slots().contains(&Slot::PaleoTemperature));
    let u = PaleoUnit {
        cx: 0,
        cz: 0,
        chapter: 0,
        present_temp_c: 7.25,
    };
    assert_eq!(p.paleo_temperature(u), 7.25, "None routes to the identity");
}

/// The report names the swapped slot by its verbatim identifier — the token a
/// manifest stores and a grep finds.
#[test]
fn the_report_names_paleo_temperature_when_swapped() {
    fn hot(_: PaleoUnit) -> f64 {
        999.0
    }
    let p = Providers {
        paleo_temperature: Some(hot),
        ..Providers::default()
    };
    assert_eq!(
        p.non_identity_slots()
            .iter()
            .map(|s| s.name())
            .collect::<Vec<_>>(),
        vec!["paleo_temperature"],
    );
}
