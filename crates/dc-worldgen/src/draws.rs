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
    /// **The near-path record-membership dither** (P11 slice 3, ruling 5 — the
    /// consumer of MM-1's `CoarseField::sample_source_cell`). Per voxel column:
    /// *which deep cell's strata record skins this column*, drawn from the
    /// bilinear stencil weights of the ~460 m record grid. Registered as its own
    /// domain (the journal/0141 salt lesson: two decisions must never share a
    /// stream) — independent of [`GeoClass`], which is the FAR tier's surface
    /// class membership at the same grid, and of [`GeoDeep`], the within-class
    /// member re-pick. Source: `Octaves` (the unbiased coherent source,
    /// journal/0128) — `Coherent` would re-import corrections #39's majority
    /// amplification at this joint, and the amplified party would be the home
    /// cell, i.e. exactly the straight border this dither exists to kill.
    NearRecordMembership = 0x5700_0012;

    // ---- registered, call sites not yet converted (module docs, hole 2) ----
    /// Deep-time surface roughness jitter (`deeptime/grid.rs`).
    DeepTimeRoughness = 0x5900_0001;
    /// **Deposition-time member fitness** (P11 slice 1): which registered member
    /// a depositing agent's class resolves to *at the moment the bed is laid*,
    /// under that epoch's own formation context.
    ///
    /// A **separate domain from [`GeoDeep`]**, which addresses the *collapse*
    /// tier's re-selection of an already-recorded unit. The two are different
    /// decisions at different tiers on different grids (deep cell vs chunk
    /// column) and must not share a stream — reusing `GeoDeep`'s salt here would
    /// make a bed's recorded identity a deterministic function of the draw the
    /// expression tier makes over it, which is the exact defect the pore-offset
    /// domain ([`GeoPore`]) was cut for.
    ///
    /// **Tag space inside the domain** (module docs, hole 1) names the *depositor*
    /// — see `deeptime::recorder::dep_tags` — and the address is
    /// `[tag, cell, chapter, k]`: per-cell, so the parallel record phase is
    /// byte-identical to the scalar one, and per-**chapter** so the tie-break is
    /// fixed while the fitness weights keep moving every epoch. Identity then
    /// turns over when the shifting CDF crosses the fixed draw — at a real change
    /// in conditions — rather than on a per-step coin.
    ///
    /// ⚠ **Half of this domain is RETIRED** (P11 slice 2, ruling 6). A transported
    /// deposit no longer draws: identity comes from the arriving composition term
    /// — what the mover actually carried — so there is nothing left to pick. What
    /// still reaches this stream is the genuine-degeneracy remainder: material made
    /// where it lies, the movers that carry no identity yet (wind, wave, the biotic
    /// layer), and the transformation edges (basement → coarse detritus, detrital
    /// organics → carbonaceous mud), whose destination member is a fact about the
    /// site rather than about the parent. FS-A retires the weathered half.
    ///
    /// ⚠ **THE VALUE MOVED, 2026-08-02, and every world's member picks moved with
    /// it.** It was `0x5900_0002`, which is the value `deeptime/refine.rs` had been
    /// spelling by hand as `SALT_DT_PERTURB` since before the domain list existed —
    /// a **real collision** between two live decisions, found by the 2026-08-02
    /// spine-audit and invisible to the `draw_domains!` duplicate check because one
    /// of the two was not in the list. The refinement perturbation kept the number
    /// (it is registered below as [`DeepTimePerturb`], byte-identically) and this
    /// domain took the next unused value, because the perturbation is a *spike*
    /// whose measured decay profile is a dated record and this is production
    /// identity the golden re-capture was going to move anyway.
    DeepMember = 0x5900_0003;
    /// **The refinement experiment's boundary-condition perturbation**
    /// (`deeptime/refine.rs`) — the `±bump_m` addressed jitter applied to the outer
    /// halo ring, whose penetration the decay profile measures.
    ///
    /// **Registered 2026-08-02 at the value it already had**, so the perturbation
    /// stream is unchanged to the bit: `Draws::bits` folds `(seed, salt, addr…)`
    /// through exactly the chain `draw_f64(&[seed, SALT, addr…])` did, asserted by
    /// `the_registered_perturbation_domain_is_the_hand_rolled_salt`. It was the
    /// second half of a **collision** with [`DeepMember`]; see that domain's banner.
    DeepTimePerturb = 0x5900_0002;
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
/// journal/0125).
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
pub struct Coherent {
    draws: Draws,
    stride: i64,
}

impl Coherent {
    /// A coherent source over `draws`'s domain, with a `stride`-voxel field cell.
    pub fn new(draws: Draws, stride: i64) -> Self {
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

// ───────────────────────── the octaves DitherSource ──────────────────────────

/// **Field-cell edges of the octave ladder, in VOXELS — pairwise-coprime primes,
/// and that is the whole point.**
///
/// A single-octave corner field is *linear along x inside a cell*, so all of its
/// curvature — every kink in every contact it draws — lives exactly on the cell
/// lattice. That is the mechanism behind the squares: `interp_select_draw` ran at
/// **one** wavelength, the 32-voxel chunk, so every patch was chunk-sized and
/// every kink sat on the 28.8 m grid (corrections #45: *world-anchored and
/// C0-continuous, and the defect is single-octave*; the same day's diagnosis:
/// *"fix = octaves, not resolution"* — a finer single octave just makes smaller
/// squares).
///
/// Powers of two would put every coarse octave's kinks **on top of** the fine
/// ones, so a dyadic ladder starting at 512 still has a 32-voxel lattice in it.
/// Distinct primes share no common multiple below their product, so no lattice
/// survives at any scale a player can see — measured by
/// [`tests::the_octave_field_has_no_kink_lattice_at_the_chunk_scale`].
///
/// Ratios are ≈ 2 (1.98, 2.02, 2.08, 1.97, 2.38, 1.86, 2.33), so the ladder is a
/// lacunarity-2 fBm in everything but the exact alignment. The head of it,
/// **509 voxels = 458.1 m at the N=2 player scale, is the ~460 m deep cell** —
/// the coarsest scale the *record* itself resolves, so the selection field varies
/// at the scale of the thing it is selecting within, and below.
const OCTAVE_STRIDES: [i64; MAX_OCTAVES] = [509, 257, 127, 61, 31, 13, 7, 3];

/// Per-octave lattice offsets, voxels. The strides alone are coprime, but every
/// lattice still passes through the origin, so without an offset all octaves
/// share a kink at `(0, 0)` and its multiples of the stride product. Arbitrary
/// fixed numbers (they are baked into world identity like any salt); their only
/// requirement is that they are not multiples of their octave's stride.
const OCTAVE_OFFSETS: [(i64, i64); MAX_OCTAVES] = [
    (0, 0),
    (73, 149),
    (211, 37),
    (19, 97),
    (7, 23),
    (5, 11),
    (3, 2),
    (1, 1),
];

/// Ceiling on the ladder — the length of [`OCTAVE_STRIDES`].
pub const MAX_OCTAVES: usize = 8;

/// **The octaves [`DitherSource`]** — a *normal-score-transformed* fractional
/// Brownian value-noise field, and the second production impl of dc-core's
/// caller-owned-entropy seam (E5 member #0, continuation slot (a)).
///
/// It exists because [`Coherent`] has exactly one wavelength. Where the *shares*
/// a draw indexes vary at 460 m and the *voxels* are 0.9 m, a single-wavelength
/// source can only express one patch size, and the corpus has watched that read
/// as a grid twice: the near field's 28.8 m member squares (U3, corrections #45)
/// and the user's 2026-07-29 field report on the cold tier (*"the bilinear noise
/// does not actually approximate what loaded chunks look like well, it sticks
/// out poorly"*). **Structure at every scale is the thing a single octave cannot
/// give at any bias.**
///
/// ## Why it is not just a sum of octaves — the sum is unusable
///
/// The obvious construction (add `L` corner fields with amplitudes `pᵏ`, divide
/// by `Σ p^k`) is **catastrophically wrong for a source feeding an inverse-CDF
/// draw**, and the arithmetic says so before any world is built. A normalised
/// sum of 6 octaves of bilinear uniforms is a sum of 24 independent uniforms
/// with weights ≤ 0.13; its standard deviation is
/// `sqrt(Σ aₖ²·Σⱼwⱼ² / 12) / Σaₖ ≈ 0.085`. It is a narrow bell around ½ that
/// essentially never leaves `[0.25, 0.75]` — so **any class whose CDF band lies
/// outside the middle half is never drawn at all.** That is the dice-sum
/// distortion (spines § 4 carve-out 1, corrections #39) magnified until it stops
/// being a bias and becomes a truncation. Adding octaves makes it *worse*, never
/// better: independent summands multiply the density's Fourier coefficients
/// toward a Gaussian, and a Gaussian on a unit interval is the opposite of
/// uniform.
///
/// ## The construction, and why the marginal is uniform BY CONSTRUCTION
///
/// The cure is to stop fighting the Gaussianity and use it. **Draw the corner
/// values from a unit normal instead of a uniform.** A bilinear blend of
/// independent normals *is* normal — exactly, not approximately — and so is a
/// weighted sum of those blends, with a variance that is a closed form of the
/// interpolation weights:
///
/// ```text
/// S(p)  = Σₖ aₖ · Σⱼ wₖⱼ(p)·gₖⱼ ,   gₖⱼ ~ N(0,1) iid
/// σ²(p) = Σₖ aₖ² · Σⱼ wₖⱼ(p)²  =  Σₖ aₖ² · sx·sz ,  sx = (1−fx)² + fx²
/// u(p)  = Φ( S(p) / σ(p) )
/// ```
///
/// `S/σ` is standard normal at **every** position, so `u` is **uniform on
/// `[0,1)` at every position** — the unbiasedness a `ShareVec::draw` consumer
/// needs is a property of the construction rather than a measured tolerance.
/// This is the standard geostatistical **truncated-Gaussian facies simulation**
/// move (thresholding a Gaussian random field at the quantiles of the target
/// proportions), arriving here from the other direction: the record supplies the
/// proportions, `ShareVec::draw` supplies the thresholds, and this supplies the
/// field. It makes `Octaves` the **first unbiased coherent source in the tree**
/// — corrections #39's majority amplification is a property of [`Coherent`],
/// not of coherence (measured side by side in
/// [`tests::the_octave_source_is_unbiased_where_the_coherent_source_amplifies`]).
///
/// Two approximations remain, both bounded and both measured rather than
/// asserted: the corner normals come from a 1024-entry midpoint-quantile table
/// (so each corner is a 1024-level discretisation of `N(0,1)`, renormalised to
/// exactly unit variance), and `Φ` is Abramowitz & Stegun 7.1.26
/// (`|ε| ≤ 1.5e-7`). `the_octave_field_marginal_is_uniform` measures the
/// end-to-end deviation against a Kolmogorov–Smirnov bound.
///
/// ## Persistence is derived, not tuned
///
/// `persistence` sets the spectrum: amplitude `aₖ = persistenceᵏ` over a
/// lacunarity-2 ladder is an fBm of Hurst exponent `H = −log₂(persistence)`, and
/// the **level sets of a 2D fBm have fractal dimension `2 − H`** — level sets
/// being exactly what this field's contacts are. Published fractal dimensions
/// for traced geological boundaries (facies contacts, coastlines, outcrop
/// margins) cluster at **D ≈ 1.2–1.3**, so [`MEMBER_PERSISTENCE`] `= 0.6` gives
/// `H = 0.737` and `D ≈ 1.26` — inside the published band. It is a plausibility
/// anchor from the literature rather than a fit to a deepcraft measurement, and
/// it is named as one; what matters is that it is **not** a number chosen
/// because a frame looked right (CLAUDE.md § *a closed system cannot detect its
/// own scale error*).
///
/// ## Cost
///
/// `octaves` × 4 hashes per call, against [`Coherent`]'s 4, plus ~10 flops per
/// octave and one `exp`. That is the whole reason the near path's member dither
/// was hoisted to **once per voxel column per event** instead of once per voxel
/// in the same slice (journal/0129): the hoist is worth ~32× and pays for the
/// ladder several times over.
pub struct Octaves {
    draws: Draws,
    octaves: usize,
    persistence: f64,
}

/// Octave count for the near path's member dither: `509 … 13` voxels, i.e.
/// **458 m down to 11.7 m**. The floor is deliberate — a "member" patch thinner
/// than about ten voxels is not a rock body, it is per-voxel speckle, and the
/// sub-voxel scale already belongs to the eighths-allocation draw
/// (`GeoFill`). The head is the deep cell (see [`OCTAVE_STRIDES`]).
pub const MEMBER_OCTAVES: usize = 6;

/// Persistence for the near path's member dither — `H = 0.737`, contact-trace
/// fractal dimension `D ≈ 1.26`, inside the published 1.2–1.3 band for traced
/// geological boundaries. See [`Octaves`] § *Persistence is derived, not tuned*.
pub const MEMBER_PERSISTENCE: f64 = 0.6;

impl Octaves {
    /// An octaves source over `draws`'s domain. `octaves` selects how many rungs
    /// of [`OCTAVE_STRIDES`] to sum, from the coarsest; `persistence` is the
    /// per-octave amplitude ratio (see the type docs — it is an fBm Hurst
    /// exponent in disguise).
    ///
    /// **This is the whole authoring surface a pack gets** (ruled 2026-07-29:
    /// *per-voxel stays engine-executed; content decides through data*) — select
    /// the source by id, hand it these two numbers, never a per-voxel body.
    pub fn new(draws: Draws, octaves: usize, persistence: f64) -> Self {
        assert!(
            (1..=MAX_OCTAVES).contains(&octaves),
            "octaves must be 1..={MAX_OCTAVES}, got {octaves}"
        );
        assert!(
            persistence > 0.0 && persistence <= 1.0,
            "persistence must be in (0, 1], got {persistence}"
        );
        Octaves {
            draws,
            octaves,
            persistence,
        }
    }

    /// The source the near path's member dither runs on: [`MEMBER_OCTAVES`]
    /// rungs at [`MEMBER_PERSISTENCE`].
    pub fn member(draws: Draws) -> Self {
        Self::new(draws, MEMBER_OCTAVES, MEMBER_PERSISTENCE)
    }
}

impl dc_core::coarse::DitherSource for Octaves {
    fn uniform(&self, wx: i64, wz: i64, salt: u64) -> f64 {
        let mut sum = 0.0f64;
        let mut var = 0.0f64;
        let mut amp = 1.0f64;
        for k in 0..self.octaves {
            let stride = OCTAVE_STRIDES[k];
            let (offx, offz) = OCTAVE_OFFSETS[k];
            let (ox, oz) = (wx + offx, wz + offz);
            let cx = ox.div_euclid(stride);
            let cz = oz.div_euclid(stride);
            let fx = (ox.rem_euclid(stride) as f64 + 0.5) / stride as f64;
            let fz = (oz.rem_euclid(stride) as f64 + 0.5) / stride as f64;
            // The octave index is IN THE ADDRESS. Without it two octaves whose
            // cell coordinates happen to coincide numerically would hash to the
            // same corner value and stop being independent — which would break
            // both the variance formula and the uniformity that rests on it.
            let g = |dx: i64, dz: i64| {
                gauss::corner_normal(self.draws, salt, k as u64, cx + dx, cz + dz)
            };
            let top = g(0, 0) * (1.0 - fx) + g(1, 0) * fx;
            let bot = g(0, 1) * (1.0 - fx) + g(1, 1) * fx;
            sum += amp * (top * (1.0 - fz) + bot * fz);
            // Σⱼ wⱼ² factorises: ((1−fx)² + fx²)·((1−fz)² + fz²).
            let sx = (1.0 - fx) * (1.0 - fx) + fx * fx;
            let sz = (1.0 - fz) * (1.0 - fz) + fz * fz;
            var += amp * amp * sx * sz;
            amp *= self.persistence;
        }
        // `var` is bounded below by the coarsest octave's 0.25 (amp = 1, sx and
        // sz ≥ ½ each), so this never divides by zero.
        gauss::phi(sum / var.sqrt()).clamp(0.0, 1.0 - f64::EPSILON)
    }
}

/// **The normal-score machinery [`Octaves`] rests on**, kept private and local.
///
/// These are general statistics and they would sit more naturally in
/// `dc_sim::statistical`. They are here because there is exactly **one**
/// consumer: promoting a utility to a shared crate before a second caller exists
/// is anti-shape A-4 (build beside the thing that needs it, move it when
/// something else asks). If a second consumer appears, this module is what
/// moves.
mod gauss {
    use dc_sim::statistical::rng::Draws;
    use std::sync::LazyLock;

    /// Quantile table resolution. 1024 midpoint quantiles of `N(0,1)`, so a
    /// corner value is a 1024-level discretisation — the outermost levels sit at
    /// `Φ⁻¹(1/2048) ≈ ∓3.48σ`, which is deep enough that the truncation is
    /// invisible once four corners of six octaves are summed.
    const LEVELS: usize = 1024;

    /// Midpoint quantiles of the standard normal, **renormalised to exactly unit
    /// variance** for this discrete distribution.
    ///
    /// The renormalisation is not cosmetic: [`super::Octaves`]'s variance formula
    /// assumes `Var(gₖⱼ) = 1` exactly, and the table's raw second moment is
    /// slightly below 1 (a midpoint rule under-weights the tails it truncates).
    /// Dividing by its own RMS makes the assumption true of the numbers actually
    /// used, so the uniformity of `Φ(S/σ)` does not inherit a quantisation bias.
    /// The mean is exactly zero by the symmetry of midpoint quantiles.
    static NORMAL_Q: LazyLock<[f64; LEVELS]> = LazyLock::new(|| {
        let mut q = [0.0f64; LEVELS];
        for (i, slot) in q.iter_mut().enumerate() {
            *slot = inv_phi((i as f64 + 0.5) / LEVELS as f64);
        }
        let rms = (q.iter().map(|v| v * v).sum::<f64>() / LEVELS as f64).sqrt();
        for slot in q.iter_mut() {
            *slot /= rms;
        }
        q
    });

    /// One octave corner's unit-normal value, addressed by
    /// `(domain, salt→tag, octave, cell)`. Ten bits of the hash index the
    /// quantile table; the remaining bits are unused, which is fine — the
    /// coherence, not the entropy per corner, is what this field is short of.
    #[inline]
    pub(super) fn corner_normal(draws: Draws, salt: u64, octave: u64, cx: i64, cz: i64) -> f64 {
        let bits = draws.bits(&[salt, octave, cx as u64, cz as u64]);
        NORMAL_Q[(bits >> 54) as usize]
    }

    /// The standard normal CDF, from Abramowitz & Stegun 7.1.26's `erf`
    /// (`|ε| ≤ 1.5e-7` on `erf`, hence on `Φ`). Deterministic in the same sense
    /// the rest of the generator is: pure arithmetic plus `exp`, which the
    /// collapse tier already depends on for world identity
    /// (`collapse.rs`'s flow-energy decay, `geology.rs`'s soft fitness).
    #[inline]
    pub(super) fn phi(z: f64) -> f64 {
        0.5 * (1.0 + erf(z * std::f64::consts::FRAC_1_SQRT_2))
    }

    #[inline]
    fn erf(x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + 0.327_591_1 * x);
        let poly = t
            * (0.254_829_592
                + t * (-0.284_496_736
                    + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
        sign * (1.0 - poly * (-x * x).exp())
    }

    /// Inverse standard normal CDF — Peter Acklam's rational approximation
    /// (relative error < 1.15e-9). Used **once**, to build [`NORMAL_Q`]; it is
    /// not on any per-voxel path.
    fn inv_phi(p: f64) -> f64 {
        const A: [f64; 6] = [
            -3.969_683_028_665_376e1,
            2.209_460_984_245_205e2,
            -2.759_285_104_469_687e2,
            1.383_577_518_672_69e2,
            -3.066_479_806_614_716e1,
            2.506_628_277_459_239e0,
        ];
        const B: [f64; 5] = [
            -5.447_609_879_822_406e1,
            1.615_858_368_580_409e2,
            -1.556_989_798_598_866e2,
            6.680_131_188_771_972e1,
            -1.328_068_155_288_572e1,
        ];
        const C: [f64; 6] = [
            -7.784_894_002_430_293e-3,
            -3.223_964_580_411_365e-1,
            -2.400_758_277_161_838e0,
            -2.549_732_539_343_734e0,
            4.374_664_141_464_968e0,
            2.938_163_982_698_783e0,
        ];
        const D: [f64; 4] = [
            7.784_695_709_041_462e-3,
            3.224_671_290_700_398e-1,
            2.445_134_137_142_996e0,
            3.754_408_661_907_416e0,
        ];
        const P_LOW: f64 = 0.024_25;
        if p < P_LOW {
            let q = (-2.0 * p.ln()).sqrt();
            (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
                / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
        } else if p <= 1.0 - P_LOW {
            let q = p - 0.5;
            let r = q * q;
            (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
                / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
        } else {
            let q = (-2.0 * (1.0 - p).ln()).sqrt();
            -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
                / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `Φ` and `Φ⁻¹` must actually be inverses, or the table is a table of
        /// the wrong numbers and every claim above it is void.
        #[test]
        fn phi_and_inv_phi_round_trip() {
            for i in 1..200 {
                let p = i as f64 / 200.0;
                let z = inv_phi(p);
                assert!(
                    (phi(z) - p).abs() < 2e-7,
                    "Φ(Φ⁻¹({p})) = {} — off by more than A&S's own error",
                    phi(z)
                );
            }
        }

        /// The table is what makes `Var(g) = 1` true rather than assumed. If this
        /// drifts, [`super::super::Octaves`]'s `σ²` is wrong and the marginal
        /// stops being uniform — silently, and everywhere.
        #[test]
        fn the_quantile_table_has_unit_variance_and_zero_mean() {
            let q = &*NORMAL_Q;
            let mean = q.iter().sum::<f64>() / LEVELS as f64;
            let var = q.iter().map(|v| v * v).sum::<f64>() / LEVELS as f64;
            assert!(mean.abs() < 1e-12, "table mean {mean} is not zero");
            assert!((var - 1.0).abs() < 1e-12, "table variance {var} is not one");
        }
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

    /// **The byte-identity proof for [`DeepTimePerturb`]'s registration**
    /// (P11 slice 2, the salt fix).
    ///
    /// `refine.rs` spelled `0x5900_0002` by hand and folded it as
    /// `draw_f64(&[seed, SALT, gx, gy])`. Opening the registered domain must
    /// produce the same bits at every address, or the decay experiment's dated
    /// measurements would silently be describing a different perturbation field.
    /// `Draws::bits` starts from the same IV, absorbs the seed, then the salt, then
    /// the address — the identical chain — and this asserts it rather than arguing
    /// it.
    #[test]
    fn the_registered_perturbation_domain_is_the_hand_rolled_salt() {
        use dc_sim::statistical::rng::draw_f64;
        const HAND_ROLLED: u64 = 0x5900_0002;
        assert_eq!(<DeepTimePerturb as Domain>::SALT, HAND_ROLLED);
        for seed in [0u64, 1337, 987_654_321] {
            let d = Draws::of::<DeepTimePerturb>(seed);
            for (gx, gy) in [(0u64, 0u64), (1, 0), (37, 91), (511, 511)] {
                assert_eq!(
                    d.unit(&[gx, gy]),
                    draw_f64(&[seed, HAND_ROLLED, gx, gy]),
                    "the registered domain moved the perturbation stream at                      seed {seed}, ({gx}, {gy})"
                );
            }
        }
    }

    /// **And the collision it fixed cannot come back.** `DeepMember` and
    /// `DeepTimePerturb` were the same number until 2026-08-02 — two live decisions
    /// on one stream, which the `draw_domains!` duplicate check could not see
    /// because only one of them was in the list. Now both are, so the compile-time
    /// assertion covers it; this asserts the *independence* the compile-time check
    /// only implies.
    #[test]
    fn the_member_draw_and_the_perturbation_no_longer_share_a_stream() {
        assert_ne!(
            <DeepMember as Domain>::SALT,
            <DeepTimePerturb as Domain>::SALT
        );
        let (a, b) = (
            Draws::of::<DeepMember>(1337),
            Draws::of::<DeepTimePerturb>(1337),
        );
        for addr in [&[0u64, 0][..], &[3, 7], &[91, 12]] {
            assert_ne!(a.unit(addr), b.unit(addr));
        }
    }

    /// The list is what the compile-time check reads, so it must actually list
    /// everything. A domain that never reaches `ALL_DOMAINS` is a domain the
    /// duplicate check cannot see.
    #[test]
    fn every_domain_is_listed_and_distinct() {
        // 18 since P11 slice 3 added `NearRecordMembership` (the near-path
        // record-membership dither's own stream).
        assert_eq!(ALL_DOMAINS.len(), 18, "a domain was added without a test");
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

    // ─────────────────────── the octaves source (member #0) ──────────────────
    //
    // The law tests [`Coherent`]'s mirror, plus the two this source exists for:
    // a uniform marginal (which `Coherent` does NOT have) and no kink lattice at
    // the chunk scale (which is the whole of corrections #45).

    /// Determinism first, because everything else is a property of a function
    /// that must BE one: same seed, same domain, same address, same value; a
    /// different seed, salt, or position moves it. No wall clock, no ambient
    /// entropy, no dependence on evaluation order.
    #[test]
    fn the_octave_source_is_a_pure_function_of_seed_domain_and_address() {
        use dc_core::coarse::DitherSource;
        let a = Octaves::member(Draws::of::<GeoDeep>(1337));
        let b = Octaves::member(Draws::of::<GeoDeep>(1337));
        for (wx, wz) in [(0i64, 0i64), (12_345, -6_789), (-1, 1), (71_000, -2_700)] {
            assert_eq!(a.uniform(wx, wz, 3), b.uniform(wx, wz, 3));
        }
        let other_seed = Octaves::member(Draws::of::<GeoDeep>(1338));
        let other_domain = Octaves::member(Draws::of::<GeoFill>(1337));
        assert_ne!(a.uniform(10, 10, 0), other_seed.uniform(10, 10, 0));
        assert_ne!(a.uniform(10, 10, 0), other_domain.uniform(10, 10, 0));
        assert_ne!(a.uniform(10, 10, 0), a.uniform(10, 10, 1));
        assert_ne!(a.uniform(10, 10, 0), a.uniform(11, 10, 0));
    }

    /// **The marginal is uniform — the property the construction exists for.**
    ///
    /// `S/σ` is standard normal at every position by the algebra in [`Octaves`]'s
    /// docs, so `Φ(S/σ)` is `U[0,1)` there. This measures the end-to-end
    /// deviation, which carries the two bounded approximations (the 1024-level
    /// corner table and A&S's `Φ`) plus sampling noise.
    ///
    /// **Positions are deliberately far apart and mutually prime-strided.** A
    /// contiguous window would be dominated by the coarsest octave — 509 voxels
    /// wide, so a 200-voxel window holds a *fraction* of one independent draw and
    /// the empirical CDF would measure that draw, not the marginal. At a 1031/1033
    /// pitch every sample is past every octave's correlation length.
    ///
    /// The bound is **derived, not fitted**: the Kolmogorov–Smirnov 99 % critical
    /// value at `n = 10_000` is `1.63/√n = 0.0163`, and the approximation error
    /// contributes ~1e-3, so 0.02 is the honest line. A failure here means the
    /// variance formula and the field have stopped agreeing.
    #[test]
    fn the_octave_field_marginal_is_uniform() {
        use dc_core::coarse::DitherSource;
        let src = Octaves::member(Draws::of::<GeoDeep>(1337));
        let mut vals = Vec::with_capacity(10_000);
        for i in 0..100i64 {
            for j in 0..100i64 {
                vals.push(src.uniform(i * 1031 - 50_000, j * 1033 - 50_000, 0));
            }
        }
        vals.sort_by(f64::total_cmp);
        let n = vals.len() as f64;
        let mut worst = 0.0f64;
        for (i, &v) in vals.iter().enumerate() {
            worst = worst.max((v - i as f64 / n).abs());
        }
        assert!(
            worst < 0.02,
            "octave marginal deviates from uniform by {worst:.4} (KS 99 % at n=10000 is 0.0163)"
        );
    }

    /// **The unbiasedness the far site had to live without.**
    ///
    /// `coarse.rs`'s own law test measures [`Coherent`] rendering a 0.6 majority
    /// at **> 0.63** — the dice-sum distortion, whose sign corrections #39 had to
    /// correct once already. Through this source the same share vector renders at
    /// 0.6, because the marginal is uniform. Same instrument, side by side, so the
    /// comparison is not across two test fixtures' assumptions.
    ///
    /// This is what makes the octaves source safe for the *member* dither in a
    /// way a plain octave sum would not have been: a source that pushed splits
    /// toward the majority would erase exactly the minority members the near-path
    /// slice exists to let through.
    #[test]
    fn the_octave_source_is_unbiased_where_the_coherent_source_amplifies() {
        use dc_core::coarse::{CoarseField, Registration, ShareVec};
        let sv = ShareVec::from_shares([0.6, 0.4]);
        let field = CoarseField::from_cells(1, 1, Registration::new(0.0, 0.0, 1.0), vec![sv]);
        // Positions past every correlation length, so each is an independent draw
        // of the marginal (see `the_octave_field_marginal_is_uniform`).
        let measure = |src: &dyn dc_core::coarse::DitherSource| -> f64 {
            let mut c = [0u64; 2];
            for i in 0..100i64 {
                for j in 0..100i64 {
                    let (wx, wz) = (i * 1031 - 50_000, j * 1033 - 50_000);
                    if let Some(k) = field.sample_dithered((wx, wz), src, 5, 6) {
                        c[k] += 1;
                    }
                }
            }
            c[0] as f64 / (c[0] + c[1]) as f64
        };
        let coherent = measure(&Coherent::new(Draws::of::<GeoDeep>(1337), 32));
        let octaves = measure(&Octaves::member(Draws::of::<GeoDeep>(1337)));
        eprintln!(
            "a 0.6/0.4 share vector renders as: coherent {coherent:.4}, octaves {octaves:.4} \
             (recorded share 0.6)"
        );
        // 3σ of a binomial at n = 10_000, p = 0.6 is 0.0147.
        assert!(
            (octaves - 0.6).abs() < 0.015,
            "octaves rendered the 0.6 majority at {octaves:.4} — the marginal is not uniform"
        );
        assert!(
            coherent > 0.63,
            "the coherent source is supposed to amplify the majority past 0.63 (it measured \
             {coherent:.4}); if that has changed, corrections #39 and this comparison both move"
        );
    }

    /// **No kink lattice at the chunk scale — corrections #45, as a gate.**
    ///
    /// A bilinear corner field is *linear along x inside a cell*, so its second
    /// difference along x is **exactly zero** everywhere except on the cell
    /// lattice, where the slope changes. That is the squares: all of a
    /// single-octave field's structure is concentrated on one grid, and for the
    /// member dither that grid was the 32-voxel chunk. The measurement is the
    /// ratio of mean `|Δ²u|` on the 32-lattice to mean `|Δ²u|` off it.
    ///
    /// - [`Coherent`] at stride 32: off-lattice curvature is **identically 0**, so
    ///   the ratio is infinite. Asserted as the exact zero it is, because that is
    ///   the mechanism, not a magnitude.
    /// - [`Octaves`]: no rung is 32 or a divisor of it (all strides are distinct
    ///   primes), so the 32-lattice is not special and the ratio is ~1.
    ///
    /// **Scale-free:** a second difference at three adjacent voxels is a local
    /// arithmetic property of the field, identical at any world size or extent —
    /// nothing about it depends on how much world exists around it.
    #[test]
    fn the_octave_field_has_no_kink_lattice_at_the_chunk_scale() {
        use dc_core::coarse::DitherSource;
        let curvature = |src: &dyn DitherSource| -> (f64, f64) {
            let (mut on, mut on_n, mut off, mut off_n) = (0.0f64, 0u64, 0.0f64, 0u64);
            for z in 0..64i64 {
                for x in 200i64..1_200 {
                    let d2 = (src.uniform(x - 1, z, 0) - 2.0 * src.uniform(x, z, 0)
                        + src.uniform(x + 1, z, 0))
                    .abs();
                    // "On the lattice" is *the three-point window straddling a
                    // 32-voxel line*, which is phases 0 and 31 — not phase 0
                    // alone. Getting that wrong put a chunk-line kink into the
                    // off-lattice bucket and made a stride-32 field look as if it
                    // had interior curvature (2.7e-4 instead of 1e-17).
                    if (x - 1).div_euclid(32) != (x + 1).div_euclid(32) {
                        on += d2;
                        on_n += 1;
                    } else {
                        off += d2;
                        off_n += 1;
                    }
                }
            }
            (on / on_n as f64, off / off_n as f64)
        };
        let (c_on, c_off) = curvature(&Coherent::new(Draws::of::<GeoDeep>(1337), 32));
        eprintln!("coherent(32): |Δ²u| on-lattice {c_on:.3e}, off-lattice {c_off:.3e}");
        // Off-lattice curvature is zero in exact arithmetic and float rounding
        // residue in practice, so the assertion is on the ORDER: the lattice
        // carries the structure by ten-plus decimal digits.
        assert!(
            c_off < 1e-15 && c_on / c_off > 1e9,
            "a stride-32 bilinear field must have essentially NO curvature off the 32-lattice \
             (on {c_on:.3e}, off {c_off:.3e}) — that concentration is the mechanism behind the \
             28.8 m squares"
        );
        let (o_on, o_off) = curvature(&Octaves::member(Draws::of::<GeoDeep>(1337)));
        eprintln!("octaves: |Δ²u| on-lattice {o_on:.3e}, off-lattice {o_off:.3e}");
        let ratio = o_on / o_off;
        assert!(
            (0.5..2.0).contains(&ratio),
            "the octave field's 32-voxel lattice is still special: |Δ²u| on-lattice {o_on:.3e} \
             vs off-lattice {o_off:.3e} (ratio {ratio:.2})"
        );
    }

    /// **Power at every scale, measured as a variogram.**
    ///
    /// `V(L) = E[(u(p+L) − u(p))²]`. A single-octave field **saturates at its own
    /// stride** — past one cell the corner draws are independent, so `V(32)` and
    /// `V(512)` are the same number and the field has exactly one characteristic
    /// length. That single length is what the eye reads as a patch size. The
    /// octave ladder is still climbing at 512 because its coarsest rung is 509.
    ///
    /// **Scale-free:** a lag-differenced second moment over a fixed lag set is a
    /// per-pair arithmetic property of the field; the world's extent does not
    /// enter it.
    #[test]
    fn the_octave_field_carries_variance_at_every_scale_where_one_octave_saturates() {
        use dc_core::coarse::DitherSource;
        let vario = |src: &dyn DitherSource, lag: i64| -> f64 {
            let mut acc = 0.0f64;
            let mut n = 0u64;
            for z in 0..48i64 {
                for x in 0..48i64 {
                    let (px, pz) = (x * 37 - 800, z * 41 - 900);
                    let d = src.uniform(px + lag, pz, 0) - src.uniform(px, pz, 0);
                    acc += d * d;
                    n += 1;
                }
            }
            acc / n as f64
        };
        let coh = Coherent::new(Draws::of::<GeoDeep>(1337), 32);
        let oct = Octaves::member(Draws::of::<GeoDeep>(1337));
        for lag in [1i64, 4, 16, 32, 64, 128, 512] {
            eprintln!(
                "V({lag:>3})  coherent(32) {:.5}   octaves {:.5}",
                vario(&coh, lag),
                vario(&oct, lag)
            );
        }
        // Saturation is at TWO strides, not one: at lag 32 exactly, `u(x)` and
        // `u(x+32)` still share the cell corner between them, so they are not yet
        // independent. Past 64 the single octave has nothing left to say.
        let (c64, c512) = (vario(&coh, 64), vario(&coh, 512));
        assert!(
            (c512 / c64 - 1.0).abs() < 0.15,
            "the single octave must be saturated past two strides: V(64) {c64:.4}, V(512) {c512:.4}"
        );
        let (o32, o512) = (vario(&oct, 32), vario(&oct, 512));
        assert!(
            o512 / o32 > 1.3,
            "the octave ladder must still be climbing past the chunk scale: V(32) {o32:.4}, \
             V(512) {o512:.4} (ratio {:.2})",
            o512 / o32
        );
    }

    /// Two salts of one octaves source are independent — the tag-space sibling of
    /// [`two_salts_of_the_coherent_source_are_independent`], and the same
    /// assumption `sample_dithered`'s two draws rest on. The bound is that test's,
    /// for the same reason (each value is a blend of many corner draws, so the
    /// null distribution of the sample correlation is wider than a raw stream's).
    #[test]
    fn two_salts_of_the_octaves_source_are_independent() {
        use dc_core::coarse::DitherSource;
        let src = Octaves::member(Draws::of::<GeoDeep>(1337));
        let n = 20_000usize;
        let (mut sx, mut sy, mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 0..n {
            let (wx, wz) = ((i as i64 % 137) * 71 - 4_000, (i as i64 / 137) * 73 - 5_000);
            let (x, y) = (src.uniform(wx, wz, 0), src.uniform(wx, wz, 1));
            sx += x;
            sy += y;
            sxy += x * y;
            sxx += x * x;
            syy += y * y;
        }
        let nf = n as f64;
        let r = (sxy - sx * sy / nf) / ((sxx - sx * sx / nf) * (syy - sy * sy / nf)).sqrt();
        assert!(r.abs() < 0.08, "two octave salts correlate at r = {r:+.4}");
    }

    /// **The retirement receipt for `geology::interp_select_draw`** (journal/0129).
    ///
    /// The near path's member dither used to call a chunk-addressed helper —
    /// `interp_select_draw(seed, salt, tag, cx, cz, fx, fz)` with
    /// `fx = (x + 0.5)/32` — and that helper is bit-for-bit a stride-32
    /// [`Coherent`] read at the absolute voxel. Only the *address arithmetic*
    /// could differ, so that is what this pins: the retirement moved the call to a
    /// `DitherSource`, and the field it was reading is unchanged. It is also what
    /// lets the A/B probe reconstruct the pre-slice member field exactly, in the
    /// same binary as the new one (journal/0125's one-binary comparison shape).
    #[test]
    fn a_stride_32_coherent_is_the_retired_chunk_addressed_selection_field() {
        use dc_core::coarse::DitherSource;
        let draws = Draws::from_recorded_salt(1337, 0x5700_000E);
        let src = Coherent::new(draws, 32);
        for (cx, cz) in [(0i64, 0i64), (3, -7), (-12, 40)] {
            for (x, z) in [(0usize, 0usize), (5, 31), (16, 16), (31, 0)] {
                let fx = (x as f64 + 0.5) / 32.0;
                let fz = (z as f64 + 0.5) / 32.0;
                assert_eq!(
                    interp_corner_field(draws, 2, cx, cz, fx, fz),
                    src.uniform(cx * 32 + x as i64, cz * 32 + z as i64, 2),
                    "the retired chunk-addressed field and a stride-32 Coherent must agree \
                     bit-for-bit at ({cx},{cz})+({x},{z})"
                );
            }
        }
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
