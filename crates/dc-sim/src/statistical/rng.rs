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
