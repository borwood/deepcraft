//! Deterministic, *addressable* randomness for the statistical tier.
//!
//! Two primitives, both hand-rolled (no ambient entropy, no OS randomness,
//! no wall clock — the crate rules in `lib.rs` apply):
//!
//! - **Counter-based draws** ([`mix`], [`draw_f64`]): a splitmix64-style
//!   finalizer over an explicit key tuple. Every random number the simulation
//!   consumes is *addressed* by `(world_seed, sample_index, salt, entity,
//!   tick)` rather than pulled from a stream. This is load-bearing: bounded
//!   collapse re-simulates different *subsets* of the world per query, and a
//!   sequential stream would hand different entities different numbers
//!   depending on scope. Addressed draws mean region 7 at tick 12 in sample 3
//!   sees the same noise whether the scope is depth-2 or unbounded — scope
//!   changes are then *only* boundary-condition changes (and this doubles as
//!   common-random-numbers variance reduction when comparing depths).
//!
//! - **[`Pcg32`]**: a standard PCG-XSH-RR 64/32 generator for the few places
//!   where a short sequential stream keyed off one seed is more convenient
//!   (e.g. weight-proportional selection during collapse).

/// splitmix64 finalizer. Bijective on `u64`, good avalanche.
#[inline]
pub fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash an explicit key tuple into a `u64`. Order-sensitive.
#[inline]
pub fn mix(parts: &[u64]) -> u64 {
    // Fold with a fixed IV; each part is absorbed through the finalizer so
    // sequential keys (agent ids, ticks) don't produce correlated outputs.
    let mut h = 0x243F_6A88_85A3_08D3u64; // pi digits, nothing up the sleeve
    for &p in parts {
        h = splitmix64(h ^ p);
    }
    h
}

/// One uniform draw in `[0, 1)` addressed by a key tuple (53-bit mantissa).
#[inline]
pub fn draw_f64(parts: &[u64]) -> f64 {
    (mix(parts) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

// ---------------------------------------------------------------------------
// The randomness provider: every caller gets its own band of the hash
// ---------------------------------------------------------------------------

/// **A named band of the hash.** One `Domain` = one decision the world makes,
/// and two domains are statistically independent *by construction of the hash*,
/// not by the care of whoever wrote the call site.
///
/// ## Why this exists (journal/0105, user-ratified 2026-07-25)
///
/// The retired convention was: pick a salt constant by hand, or — worse — take
/// an existing draw and slice some bits out of it, and write a comment claiming
/// the bits you took are disjoint from the ones somebody else took.
/// `pore_rider_share` did exactly that. **The comment was wrong** (its "low
/// digits" were inside the twenty bits the eighth-allocation consumes, making
/// one draw a *deterministic function* of the other), and nothing in the
/// language, the tests or the review could have caught it, because a claim about
/// bit ranges is not a claim any gate can see.
///
/// A convention that depends on every future caller reading a comment correctly
/// is the same defect one layer up. So: **a caller no longer chooses bits, it
/// names a purpose.** `Draws::of::<D>(seed)` hands back a stream nobody else
/// has, and the two ways to collide are both closed at compile time —
///
/// - **reusing a salt** is impossible: salts are only spelled once, inside the
///   [`draw_domains!`] list, whose generated `const` assertion rejects a
///   duplicate value before the crate builds;
/// - **reusing a name** is impossible: each entry generates a type, so a
///   repeated name is a duplicate definition.
///
/// Reuse is still *available* — two call sites of the same conceptual draw
/// should share a domain — but only by writing that domain's name, which is the
/// deliberate act. There is no way to reach a stream by writing a number.
///
/// ## What it does not solve
///
/// **Tag space inside a domain.** A domain's address tuple is still the caller's
/// to lay out, and hand-offset tags (`tag`, `tag + 1024`) are hand-rolled
/// sub-domains with the old failure mode at smaller scale. Named, not fixed.
pub trait Domain {
    /// The domain's band of the hash. Spelled exactly once, in the crate's
    /// [`draw_domains!`] list.
    const SALT: u64;
    /// The domain's name, for diagnostics and for the uniqueness listing.
    const NAME: &'static str;
}

/// **One caller's stream.** Carries the world seed and the domain's salt; every
/// value it yields is a pure function of `(seed, domain, address)` — no wall
/// clock, no ambient entropy, no dependence on generation order (the crate rule
/// in `lib.rs`, and CLAUDE.md § Conventions). Same position, same domain, same
/// value, forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draws {
    seed: u64,
    salt: u64,
}

impl Draws {
    /// The stream for domain `D` under `seed`. **The only way to open a new
    /// stream**, and the reason a call site cannot silently land in someone
    /// else's band.
    #[inline]
    pub fn of<D: Domain>(seed: u64) -> Self {
        Draws {
            seed,
            salt: D::SALT,
        }
    }

    /// Re-open a stream whose domain was **recorded as data**.
    ///
    /// This is the one honest hole in the guarantee and it has exactly one
    /// customer: `dc-worldgen`'s `StrataEvent` stores the `sel_salt` its member
    /// was selected from, so the per-voxel member dither can re-run *that*
    /// event's selection draw years of sim-time later. The salt there is not a
    /// call site choosing a band, it is a **record of which band was used** —
    /// replaying it is reading, not issuing. Anything that is *choosing* must go
    /// through [`Draws::of`]; the long name is the friction that keeps it so.
    #[inline]
    pub fn from_recorded_salt(seed: u64, salt: u64) -> Self {
        Draws { seed, salt }
    }

    /// The salt, for the one case that has to persist it (see
    /// [`Draws::from_recorded_salt`]).
    #[inline]
    pub fn recorded_salt(self) -> u64 {
        self.salt
    }

    /// A uniform draw in `[0, 1)` at `addr` within this domain.
    #[inline]
    pub fn unit(self, addr: &[u64]) -> f64 {
        (self.bits(addr) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// The full 64 hashed bits at `addr` within this domain — for callers that
    /// need an integer rather than a unit interval (a seed for a sub-stream, a
    /// modulo pick).
    #[inline]
    pub fn bits(self, addr: &[u64]) -> u64 {
        let mut h = splitmix64(0x243F_6A88_85A3_08D3u64 ^ self.seed);
        h = splitmix64(h ^ self.salt);
        for &p in addr {
            h = splitmix64(h ^ p);
        }
        h
    }
}

/// Pairwise-distinct check over a domain listing, evaluated at **compile time**
/// by the assertion [`draw_domains!`] generates.
#[doc(hidden)]
pub const fn all_salts_distinct(list: &[(&str, u64)]) -> bool {
    let mut i = 0;
    while i < list.len() {
        let mut j = i + 1;
        while j < list.len() {
            if list[i].1 == list[j].1 {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

/// **Declare a crate's draw domains.** One list, one place, checked by the
/// compiler.
///
/// Each entry generates a zero-sized type implementing [`Domain`], and the macro
/// emits `ALL_DOMAINS` plus a `const` assertion that no two salts are equal — so
/// a copy-pasted salt is a **build failure**, not a subtly correlated world. A
/// repeated *name* is a duplicate type definition, which is the same story from
/// the other side.
///
/// Salts are written explicitly rather than derived from position so that a
/// domain's band is stable when the list is reordered or an entry is retired:
/// these numbers are baked into every world ever generated, and a list whose
/// values move when you sort it would silently re-roll the planet.
///
/// ```ignore
/// dc_sim::draw_domains! {
///     /// Plate seeding.
///     Plate = 0x5700_0001;
///     /// The eighth allocation in a mixed voxel.
///     GeoFill = 0x5700_000F;
///}
/// ```
#[macro_export]
macro_rules! draw_domains {
    ($( $(#[$meta:meta])* $name:ident = $salt:expr ; )*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $name;

            impl $crate::statistical::rng::Domain for $name {
                const SALT: u64 = $salt;
                const NAME: &'static str = stringify!($name);
            }
        )*

        /// Every domain this crate issues, `(name, salt)`, in declaration order.
        /// Exists so the uniqueness guarantee is *inspectable* as well as
        /// enforced — a test can print it, and the compile-time assertion below
        /// reads it.
        pub const ALL_DOMAINS: &[(&str, u64)] = &[ $( (stringify!($name), $salt) ),* ];

        const _: () = assert!(
            $crate::statistical::rng::all_salts_distinct(ALL_DOMAINS),
            "two draw domains share a salt — every domain owns its own band of \
             the hash, so the duplicate must be given a new value or the two call \
             sites must share one domain deliberately"
        );
    };
}

/// PCG-XSH-RR 64/32 (O'Neill 2014), implemented from the paper.
#[derive(Debug, Clone)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    const MULT: u64 = 6_364_136_223_846_793_005;

    /// Explicitly seeded; `stream` selects one of 2^63 independent sequences.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(Self::MULT).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (((hi << 32) | lo) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcg32_reference_vector() {
        // Reference output for seed=42, stream=54 from the PCG minimal C
        // implementation (pcg32_srandom_r(42, 54)).
        let mut rng = Pcg32::new(42, 54);
        let expected: [u32; 6] = [
            0xa15c_02b7,
            0x7b47_f409,
            0xba1d_3330,
            0x83d2_f293,
            0xbfa4_784b,
            0xcbed_606e,
        ];
        for e in expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    fn draws_are_deterministic_and_distinct() {
        let a = draw_f64(&[1, 2, 3]);
        let b = draw_f64(&[1, 2, 3]);
        let c = draw_f64(&[1, 2, 4]);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!((0.0..1.0).contains(&a));
    }
}
