//! **The provider seam's byte-identity proof** (journal/0060) — the
//! *cross-commit* golden, and nothing else.
//!
//! This file is deliberately small and deliberately alone. Every other provider
//! suite is named for the slot it exercises, so a seam conversion edits (or
//! adds) exactly one of those and **never this one**. The goldens are the
//! project's fixed point for deep-time generation; a file that no conversion has
//! a reason to open is a file whose constants cannot be "updated to match".
//!
//! ## Why the goldens are constants and not a self-comparison
//!
//! The obvious test — "build a field twice, once through the old entry and once
//! through the new one, assert equal" — is what `deep_config_plumbing.rs` does
//! for `DeepOverrides`, and it is a good test, but it is **circular** as a proof
//! that the *provider slice* changed nothing: both sides run post-slice code. If
//! the pointer indirection had perturbed an expression, both halves would be
//! perturbed identically and the test would still pass.
//!
//! So the real proof is a **golden captured from pre-slice `main`**
//! (`2434f37`, "journal/0056: the walk that found three things"). The file was
//! written and run *first*, against untouched `main`, and the two hashes in
//! `providers_common` were transcribed from that run. The provider conversions
//! then had to reproduce them.
//!
//! **To re-run this proof independently:**
//!
//! ```text
//! git worktree add ../preslice 2434f37
//! cp -r crates/dc-worldgen/tests/providers_common ../preslice/crates/dc-worldgen/tests/
//! cp crates/dc-worldgen/tests/providers_golden.rs ../preslice/crates/dc-worldgen/tests/
//! # `providers_common` and this file name nothing the slice introduced — they
//! # use only `deeptime`'s pre-slice public API — so both compile unchanged
//! # against pre-slice main. The per-slot suites do not; do not copy them.
//! cargo test -p dc-worldgen --release --test providers_golden -- --nocapture
//! ```
//!
//! The printed `surface fingerprint` / `record fingerprint` must equal
//! `GOLDEN_SURFACE` / `GOLDEN_RECORD`.
//!
//! The world hashed is the **production** config at `Extent::Small`
//! (`build_field`, which is `production_config`: biotic on, erodibility on,
//! tectonic history on, full agent roster on) — so all four converted seams are
//! live in the hashed run: `outcrop_at` through `expose`/`periglacial`/`eolian`/
//! `wave`, `wave_energy` through the littoral agent, `parent_p` through the
//! biotic layer's rock-phosphorus pool, `depth_to_water` through the four
//! waterlogging thresholds.

mod providers_common;
use providers_common::{GOLDEN_RECORD, GOLDEN_SURFACE, production_field};
use providers_common::{record_fingerprint, surface_fingerprint};

/// **The byte-identity acceptance test for the provider slice.** The production
/// world at [`SEED`] must hash to the values captured from pre-slice `main`.
///
/// This is the only test in the suite that can fail *because* a provider seam
/// was introduced: everything else the slice touches is either a compile-time
/// shape or a self-comparison.
#[test]
fn the_production_world_still_hashes_to_the_pre_slice_goldens() {
    let f = production_field();
    let surface = surface_fingerprint(&f);
    let record = record_fingerprint(&f);
    println!("surface fingerprint = {surface:#018X}");
    println!("record  fingerprint = {record:#018X}");
    assert_eq!(
        surface, GOLDEN_SURFACE,
        "the surface planes moved: {surface:#018X} != {GOLDEN_SURFACE:#018X} \
         (pre-slice main 2434f37)"
    );
    assert_eq!(
        record, GOLDEN_RECORD,
        "the strata record moved: {record:#018X} != {GOLDEN_RECORD:#018X} \
         (pre-slice main 2434f37)"
    );
}

/// The fingerprint is a fingerprint: two runs of the same world agree, so a
/// mismatch above means the world moved and not that the hash is unstable.
#[test]
fn the_fingerprint_is_reproducible_within_a_build() {
    let a = production_field();
    let b = production_field();
    assert_eq!(surface_fingerprint(&a), surface_fingerprint(&b));
    assert_eq!(record_fingerprint(&a), record_fingerprint(&b));
}
