//! **Slot `outcrop_at`** — falsifiers for the outcrop seam (journal/0060).

/// The identity `outcrop_at` is the pre-slice `exposed_litho`: top of the
/// record, or `Basement` when the record is empty.
#[test]
fn the_identity_outcrop_is_the_top_of_the_record() {
    use dc_worldgen::deeptime::providers::identity_outcrop_at;
    use dc_worldgen::deeptime::{
        Aridity, DepEnv, DepTag, DepUnit, EnergyBand, Litho, litho_of_tag,
    };
    assert_eq!(identity_outcrop_at(None), Litho::Basement);
    let tag = DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low);
    let unit = DepUnit {
        tag,
        thickness_m: 1.0,
        unconformity: false,
        chapter: 0,
    };
    assert_eq!(identity_outcrop_at(Some(&unit)), litho_of_tag(tag));
}
