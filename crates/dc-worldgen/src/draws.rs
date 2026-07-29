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
//! naming a domain — **everywhere the conversion reached.** This paragraph
//! claimed "no expression anywhere in worldgen turns a number into randomness"
//! from 2026-07-26 to 2026-07-29, and it was **false when written**:
//! `deeptime/tectonics.rs` predates the conversion and never entered it (residue
//! 3 below). The claim enumerated what the conversion touched and generalised to
//! what exists — the corrections #64 mechanism, caught by the first spine-audit.
//!
//! ## Values are explicit, and that is deliberate
//!
//! A domain's salt is baked into every world ever generated from it. Deriving
//! salts from list position would mean sorting this file re-rolls the planet, so
//! the numbers are written out and the compiler — not the reader — checks that
//! they are distinct.
//!
//! ## The one honest hole left, and the residues of the other
//!
//! 1. **Tag space inside a domain is still hand-laid.** `GeoSelect` addresses
//!    four different veneer passes as tags 0–3, and `GeoAccessory` uses
//!    `tag` and `tag + 1024` for its presence gate and its selection. Those are
//!    hand-rolled sub-domains with the same failure mode at smaller scale. The
//!    provider does not solve them; a `Domain` per decision would, and the cost
//!    is a longer list.
//! 2. **`deeptime/`'s call sites are converted** (2026-07-26, the three-line
//!    follow-on journal/0105 left for whoever held those files):
//!    `deeptime/grid.rs`'s bedrock jitter now opens [`DeepTimeRoughness`] and
//!    `deeptime/biotic.rs`'s ignition and flood rolls open [`BioticFire`] and
//!    [`BioticFlood`], all through `Draws::of`. Byte-identical — `Draws::bits`
//!    folds `(seed, salt, addr…)` through exactly the chain `draw_f64(&[seed,
//!    SALT, addr…])` did.
//!
//!    **Two of the three local `const SALT_*` are gone with their call sites.**
//!    Once `biotic.rs`'s two rolls opened their domains, `SALT_BIO_FIRE` /
//!    `SALT_BIO_FLOOD` had no reader but the agreement test itself — a copy
//!    guarded against an authority nothing else consulted, which asserts nothing
//!    — so they were deleted and the two assertions with them. The domains above
//!    are now the sole spelling of those numbers.
//!
//!    `SALT_DT_ROUGH` survives, and honestly: `deeptime/refine.rs` has a
//!    **second** roughness-jitter site — a fourth call site journal/0105's
//!    "three" did not count — which still reads the constant. It was left to its
//!    owner rather than converted from outside, so the constant stays a real copy
//!    with a real reader and keeps its agreement assertion.
//!
//! 3. **`deeptime/tectonics.rs` never entered the conversion at all** (found by
//!    the 2026-07-29 spine-audit). `SALT_TEC_POS` / `SALT_TEC_VEL` /
//!    `SALT_TEC_CRUST` (`tectonics.rs:51-53`) are a **third** numbering prefix
//!    (`0x5D00_*`) with seven live `draw_f64` sites (`:115-121`, `:409`) —
//!    shipped plate seeding and crust jitter, not a spike. No world is wrong
//!    (the prefixes cannot collide), but for these salts the
//!    duplicate-fails-to-compile property is asserted by prose, not enforced by
//!    the macro. Conversion is owed to this file's owner, byte-identical the
//!    same way `grid.rs`'s was: `Draws::bits` folds `(seed, salt, addr…)`
//!    through exactly the chain `draw_f64(&[seed, SALT, addr…])` does.

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
    // 0x5700_0005 .. 0x5700_0009 were `SitePos` / `Overlay` / `Expand` / `Sack`
    // / `Ruin` — the bootstrap settlement-history and ruin-post domains, removed
    // 2026-07-28 (journal/0121) with the content that opened them. **The gap is
    // deliberate and the numbers are retired, not free.** A salt is baked into
    // every world ever generated from it, so re-issuing one of these to a new
    // decision would silently make two unrelated decisions share a stream across
    // every saved world and every old journal capture. Take the next unused
    // value; never fill a hole.
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

/// **The coherent [`DitherSource`]** — dc-worldgen's production impl of dc-core's
/// caller-owned-entropy seam, wrapping [`interp_corner_field`] (member #0,
/// journal/0124).
///
/// dc-core is headless and holds no RNG, so `CoarseField::sample_dithered` takes
/// its addressed uniform through a trait and the *implementation* IS the SOURCE
/// axis (`coarse.rs` § "the source axis"). This is the **coherent** end of that
/// axis: a bilinear corner-hash field over `stride`-voxel cells, low-frequency
/// enough that a coarse consumer can point-sample it. It is the same field the
/// class dither has read since journal/0073 — the far field point-samples the
/// surface class at a wide stride and white noise aliases there into a
/// mesh-doubling speckle. Its cost is the dice-sum CDF distortion, which
/// **amplifies the majority** class (corrections #39 fixed the sign); the honest
/// heir for that is the far field summarizing shares at its own resolution, which
/// belongs to the octree node contract (ruled 2026-07-29).
///
/// ## The salt mapping: `salt` → **tag**, inside a domain fixed at construction
///
/// `DitherSource::uniform` has ONE salt dimension; this crate's entropy is
/// addressed by **(domain, tag, address)**. A domain is a *type* (`draw_domains!`)
/// and cannot be selected by a runtime `u64`, so the only honest mapping is:
///
/// - the **domain** is chosen by whoever constructs the source (`Draws::of::<D>`),
///   which is also where the seed enters;
/// - the caller's **`salt` becomes the `tag`** within that domain.
///
/// So one `Coherent` is one domain's coherent field, and a consumer that wants two
/// independent draws at the same position passes two salts — exactly what
/// `sample_dithered` does (membership, then class). This inherits **hole 1** of
/// this module's docs (hand-laid tag space inside a domain) rather than curing it:
/// the tags a `Coherent` hands out are as hand-laid as [`GeoSelect`]'s 0–3, and the
/// compile-time duplicate check does not see them. The cure is a `Domain` per
/// decision, and it is the same cure hole 1 already names.
pub(crate) struct Coherent {
    draws: Draws,
    stride: i64,
}

impl Coherent {
    /// A coherent source over `draws`'s domain, with a `stride`-voxel field cell.
    pub(crate) fn new(draws: Draws, stride: i64) -> Self {
        debug_assert!(stride > 0, "a coherent field cell must be at least 1 voxel");
        Coherent { draws, stride }
    }
}

impl dc_core::coarse::DitherSource for Coherent {
    #[inline]
    fn uniform(&self, wx: i64, wz: i64, salt: u64) -> f64 {
        let cx = wx.div_euclid(self.stride);
        let cz = wz.div_euclid(self.stride);
        let fx = (wx.rem_euclid(self.stride) as f64 + 0.5) / self.stride as f64;
        let fz = (wz.rem_euclid(self.stride) as f64 + 0.5) / self.stride as f64;
        interp_corner_field(self.draws, salt, cx, cz, fx, fz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_sim::statistical::rng::Domain;

    /// The registry is total: `deeptime/`'s remaining locally-spelled constant is
    /// the same number as its registered domain, so the compile-time uniqueness
    /// check actually covers it. **This is the agreement test the "a summary is
    /// not an authority" rule asks for** — the list is the authority, the local
    /// `const` is the copy, and they must not drift.
    ///
    /// **It used to hold three pairs and now holds one, and the shrink is the
    /// deliverable** (2026-07-26). The biotic pair went when `biotic.rs`'s call
    /// sites converted: with no production reader left, `SALT_BIO_FIRE` /
    /// `SALT_BIO_FLOOD` were copies of an authority that nothing else read, and
    /// an agreement test between a constant and *itself under another name*
    /// asserts nothing. They were deleted rather than kept alive with an
    /// `#[allow(dead_code)]`; [`BioticFire`] / [`BioticFlood`] above are now the
    /// only spelling of those numbers, which is a stronger guarantee than any
    /// test — a copy that does not exist cannot drift.
    ///
    /// `SALT_DT_ROUGH` stays because it still has a genuine second reader that is
    /// **not** this test: `deeptime/refine.rs`'s roughness jitter, a fourth
    /// call site journal/0105's "three" did not count. Converting that one is
    /// what would let this test retire entirely.
    #[test]
    fn the_deeptime_constants_agree_with_their_registered_domains() {
        assert_eq!(
            crate::deeptime::grid::SALT_DT_ROUGH,
            <DeepTimeRoughness as Domain>::SALT
        );
    }

    /// The list is what the compile-time check reads, so it must actually list
    /// everything. A domain that never reaches `ALL_DOMAINS` is a domain the
    /// duplicate check cannot see.
    #[test]
    fn every_domain_is_listed_and_distinct() {
        assert_eq!(ALL_DOMAINS.len(), 15, "a domain was added without a test");
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

    /// **Two salts of one [`Coherent`] source are independent.**
    ///
    /// `CoarseField::sample_dithered` takes TWO salts at one position — one picks
    /// which cell's shares to draw from (the cake law), one draws the class within
    /// it — and the unbiasedness argument assumes they are independent draws. Under
    /// the salt→tag mapping they are two tags of one domain, so this is the
    /// tag-space sibling of [`two_domains_are_independent_at_the_same_address`],
    /// measured through the *interpolated* field rather than the raw stream:
    /// coherence within one tag must not become correlation across two.
    ///
    /// The bound is looser than the raw-stream test's 0.03 on purpose — each value
    /// here is a bilinear blend of four corner draws, so 20,000 sample positions
    /// are drawn from far fewer independent corners and the sample correlation has
    /// a correspondingly wider null distribution.
    #[test]
    fn two_salts_of_the_coherent_source_are_independent() {
        use dc_core::coarse::DitherSource;
        let src = Coherent::new(Draws::of::<GeoClass>(1337), 32);
        let n = 20_000usize;
        let (mut sx, mut sy, mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 0..n {
            let (wx, wz) = ((i as i64 % 137) * 13 - 800, (i as i64 / 137) * 11 - 700);
            let (x, y) = (src.uniform(wx, wz, 0), src.uniform(wx, wz, 1));
            sx += x;
            sy += y;
            sxy += x * y;
            sxx += x * x;
            syy += y * y;
        }
        let nf = n as f64;
        let r = (sxy - sx * sy / nf) / ((sxx - sx * sx / nf) * (syy - sy * sy / nf)).sqrt();
        assert!(
            r.abs() < 0.08,
            "the membership and class salts correlate at r = {r:+.4}"
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
