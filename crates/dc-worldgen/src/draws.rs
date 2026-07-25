//! **Every band of the hash this crate issues, in one list.**
//!
//! Worldgen makes a few dozen independent random decisions — where plates seed,
//! which member fills a stratum, which materials win a voxel's leftover eighths
//! — and every one of them must be *independent of every other*, because they
//! are all functions of the same position. The mechanism for that is
//! [`dc_sim::statistical::rng::Draws`]: a caller names a purpose and receives
//! its own stream.
//!
//! ## Why the list exists, and why it is a list (journal/0105)
//!
//! It used to be seventeen hand-written `const SALT_*` in `pregen/mod.rs`, plus
//! three more in `deeptime/` under a *different* numbering prefix, plus — the
//! defect that started this — one call site that skipped salts altogether and
//! sliced three bits out of a *neighbouring draw*, with a comment claiming the
//! bits it took were disjoint from the bits the neighbour used. They were not.
//! Nothing could have caught it: a claim about bit ranges is not something a
//! gate can see, and "the next person will read the comment" is not a mechanism.
//!
//! Now: the salt is spelled **once**, here; a duplicate value fails to compile
//! (the `const` assertion `draw_domains!` generates); a duplicate name fails to
//! compile (it is a duplicate type); and a call site reaches a stream only by
//! naming a domain. There is no expression anywhere in worldgen that turns a
//! number into randomness.
//!
//! ## Values are explicit, and that is deliberate
//!
//! A domain's salt is baked into every world ever generated from it. Deriving
//! salts from list position would mean sorting this file re-rolls the planet, so
//! the numbers are written out and the compiler — not the reader — checks that
//! they are distinct.
//!
//! ## The two honest holes, both named
//!
//! 1. **Tag space inside a domain is still hand-laid.** `GeoSelect` addresses
//!    four different veneer passes as tags 0–3, and `GeoAccessory` uses
//!    `tag` and `tag + 1024` for its presence gate and its selection. Those are
//!    hand-rolled sub-domains with the same failure mode at smaller scale. The
//!    provider does not solve them; a `Domain` per decision would, and the cost
//!    is a longer list.
//! 2. **`deeptime/`'s three domains are registered here but their call sites
//!    still spell the constant locally** — `SALT_DT_ROUGH` in `deeptime/grid.rs`,
//!    `SALT_BIO_FIRE`/`SALT_BIO_FLOOD` in `deeptime/biotic.rs`. Registering them
//!    is what makes the uniqueness check total (nothing can now re-issue
//!    `0x5900_0001`), and `the_deeptime_constants_agree_with_their_registered_domains`
//!    keeps the two spellings in agreement. Converting the call sites is a
//!    three-line follow-on, left to whoever holds those files.

use dc_sim::statistical::rng::Draws;

dc_sim::draw_domains! {
    /// Plate seeding: position, angle, speed and continental/oceanic character.
    Plate = 0x5700_0001;
    /// Per-cell tectonic noise.
    Cell = 0x5700_0002;
    /// The border wilds' regolith noise.
    Wilds = 0x5700_0003;
    /// The multi-level elevation cascade.
    Elev = 0x5700_0004;
    /// Within-cell site jitter.
    SitePos = 0x5700_0005;
    /// The seed handed to the statistical overlay world.
    Overlay = 0x5700_0006;
    /// Polity expansion.
    Expand = 0x5700_0007;
    /// Site sacking.
    Sack = 0x5700_0008;
    /// Ruin layout.
    Ruin = 0x5700_0009;
    /// Year-zero veneer strata: which member fills the class a pass selected.
    /// Tags 0–3 address the four veneer passes (see hole 1 in the module docs).
    GeoSelect = 0x5700_000A;
    /// Veneer bed thickness.
    GeoThick = 0x5700_000B;
    /// Placer ore enrichment.
    GeoOre = 0x5700_000C;
    /// Accessory-inclusion presence gate + selection (3d pore partials).
    GeoAccessory = 0x5700_000D;
    /// Deep-time-derived depositional strata: member selection draw (3e-1).
    /// Distinct tag space from the year-zero veneer's [`GeoSelect`] so the two
    /// never collide, and the per-voxel member dither addresses each deep unit
    /// uniquely.
    GeoDeep = 0x5700_000E;
    /// **The eighth-allocation draw** for distribution-first strata expression
    /// (materials.md DECIDED 2026-07-21). Addressed by *world voxel position*,
    /// not by chunk or column: a voxel's composition must not depend on which
    /// chunk was generated first, on the chunk's `y`, or on any iteration order.
    /// One draw per mixed voxel decides which materials win the leftover eighths.
    GeoFill = 0x5700_000F;
    /// **The surface-class membership dither** (audit B1, S-4 move B;
    /// journal/0073). The surface class is drawn from the record's top-window
    /// per-class metre shares, so the categorical class frontier between two
    /// 460 m deep cells becomes an interfingered gradient instead of a stepped
    /// line. The draw reads the *coherent* bilinear corner-hash field
    /// (`interp_select_draw`), NOT per-voxel white noise — because the far field
    /// point-samples this class at a wide stride and white noise aliases into a
    /// mesh-doubling speckle there (journal/0073).
    GeoClass = 0x5700_0010;
    /// **The pore-rider rounding offset** (journal/0105). A weathering front's
    /// product rides in its host band's pore slots, and the whole eighths it
    /// wins inside a *contact* voxel are stochastically rounded from the host's
    /// own winnings (`fill::pore_rider_share`). That is a **second** decision in
    /// the same voxel as [`GeoFill`]'s allocation, and it needs its own entropy.
    ///
    /// It had none until 2026-07-25: it sliced bits 8–10 out of the very
    /// `GeoFill` draw `allocate_partial` consumes, so the pore offset was a
    /// *deterministic function* of the allocation offset — and every rider in a
    /// multi-band contact voxel shared one offset, so their rounding errors
    /// added instead of cancelling. This domain is why that cannot recur
    /// **whatever bit widths either draw grows into**; the event index in the
    /// address is what separates one band's decision from its neighbour's inside
    /// a single voxel.
    GeoPore = 0x5700_0011;

    // ---- registered, call sites not yet converted (module docs, hole 2) ----
    /// Deep-time surface roughness jitter (`deeptime/grid.rs`).
    DeepTimeRoughness = 0x5900_0001;
    /// Biotic fire ignition (`deeptime/biotic.rs`).
    BioticFire = 0x5B00_0001;
    /// Biotic flood (`deeptime/biotic.rs`).
    BioticFlood = 0x5B00_0002;
}

/// A **bilinear corner-hash field** over the cell grid, in one domain.
///
/// Four corner draws of the enclosing cell, interpolated by `(fx, fz)`. Used
/// wherever a *coherent* field is wanted rather than per-voxel white noise —
/// the class dither aliases into mesh-doubling speckle if it is drawn white
/// (journal/0073).
pub(crate) fn interp_corner_field(
    draws: Draws,
    tag: u64,
    cx: i64,
    cz: i64,
    fx: f64,
    fz: f64,
) -> f64 {
    let corner = |dx: i64, dz: i64| draws.unit(&[tag, (cx + dx) as u64, (cz + dz) as u64]);
    let (u00, u10, u01, u11) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
    let a = u00 * (1.0 - fx) + u10 * fx;
    let b = u01 * (1.0 - fx) + u11 * fx;
    (a * (1.0 - fz) + b * fz).clamp(0.0, 1.0 - f64::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_sim::statistical::rng::Domain;

    /// The registry is total: `deeptime/`'s locally-spelled constants are the
    /// same numbers as their registered domains, so the compile-time uniqueness
    /// check actually covers them. **This is the agreement test the "a summary
    /// is not an authority" rule asks for** — the list is the authority, the
    /// local `const` is the copy, and they must not drift.
    #[test]
    fn the_deeptime_constants_agree_with_their_registered_domains() {
        assert_eq!(
            crate::deeptime::grid::SALT_DT_ROUGH,
            <DeepTimeRoughness as Domain>::SALT
        );
        assert_eq!(
            crate::deeptime::biotic::SALT_BIO_FIRE,
            <BioticFire as Domain>::SALT
        );
        assert_eq!(
            crate::deeptime::biotic::SALT_BIO_FLOOD,
            <BioticFlood as Domain>::SALT
        );
    }

    /// The list is what the compile-time check reads, so it must actually list
    /// everything. A domain that never reaches `ALL_DOMAINS` is a domain the
    /// duplicate check cannot see.
    #[test]
    fn every_domain_is_listed_and_distinct() {
        assert_eq!(ALL_DOMAINS.len(), 20, "a domain was added without a test");
        let mut salts: Vec<u64> = ALL_DOMAINS.iter().map(|(_, s)| *s).collect();
        salts.sort_unstable();
        let n = salts.len();
        salts.dedup();
        assert_eq!(salts.len(), n, "two domains share a salt");
    }

    /// **The guarantee, measured.** Two different domains at the *same* address
    /// must look like two independent uniforms — not merely "different numbers",
    /// which any typo also produces. Correlation over 20,000 shared addresses,
    /// and the fraction of addresses where they agree to three bits.
    ///
    /// This is the test the retired hand-sliced offset would have failed: its
    /// three bits agreed with the allocation's offset **100 %** of the time.
    #[test]
    fn two_domains_are_independent_at_the_same_address() {
        let a = Draws::of::<GeoFill>(1337);
        let b = Draws::of::<GeoPore>(1337);
        let n = 20_000usize;
        let (mut sx, mut sy, mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut agree3 = 0usize;
        for i in 0..n {
            let addr = [i as u64, (i * 7 % 91) as u64, (i / 13) as u64];
            let (x, y) = (a.unit(&addr), b.unit(&addr));
            sx += x;
            sy += y;
            sxy += x * y;
            sxx += x * x;
            syy += y * y;
            agree3 += usize::from((x * 8.0) as u64 == (y * 8.0) as u64);
        }
        let nf = n as f64;
        let r = (sxy - sx * sy / nf) / ((sxx - sx * sx / nf) * (syy - sy * sy / nf)).sqrt();
        assert!(
            r.abs() < 0.03,
            "two domains correlate at r = {r:+.4} over {n} shared addresses"
        );
        let rate = agree3 as f64 / nf;
        assert!(
            (rate - 0.125).abs() < 0.02,
            "two domains agree to three bits {:.2} % of the time (chance 12.5 %)",
            100.0 * rate
        );
    }

    /// Determinism, which the whole tier rests on: same seed, same domain, same
    /// address, same value — and a different *seed* moves it.
    #[test]
    fn a_stream_is_a_pure_function_of_seed_domain_and_address() {
        let a = Draws::of::<GeoFill>(9);
        assert_eq!(a.unit(&[1, 2, 3]), Draws::of::<GeoFill>(9).unit(&[1, 2, 3]));
        assert_ne!(
            a.unit(&[1, 2, 3]),
            Draws::of::<GeoFill>(10).unit(&[1, 2, 3])
        );
        assert_ne!(a.unit(&[1, 2, 3]), a.unit(&[1, 2, 4]));
    }
}
