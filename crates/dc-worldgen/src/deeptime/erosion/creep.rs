//! **The hillslope-creep pass** — the driver that takes the epoch it is given
//! and sub-divides it internally into as many explicit steps as the operator's
//! own monotonicity bound requires, plus the material-creep toggle and the
//! instrumentation the probes read.
//!
//! The step it sub-cycles, and the gather kernels under it, are
//! `super::creep_kernel`.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**
//! The kernel it drives is the embedded S-10 primitive (E4's future claim).

use super::super::grid::{DeepConfig, DeepGrid};
use super::creep_kernel::{CREEP_MAX_EDGE_COEFF, eff_diff};
use super::{Erosion, SPECIES};

impl Erosion {
    /// Turn **material-aware hillslope creep** on (Movement 2b continuation (b),
    /// `material-behavior.md` § 13.2 — the gravity/mass-wasting member of the
    /// transport family). Off is the anonymous-creep path, byte for byte, because
    /// the species plane stays empty and [`Self::record`] falls back to the
    /// fluvial-only mixture.
    ///
    /// **Gated on [`Self::set_material_transport`]**, and that is not a
    /// convenience: what creep moves is the near-surface composition the
    /// `outcrop_shares` seam publishes into [`Self::shares`], which only exists on
    /// the material-aware path. Asking for creep identity without it would have to
    /// take a *second* composition walk beside the one already running — the
    /// re-invention-next-door this project keeps catching itself at (spines A-4).
    pub fn set_material_creep(&mut self, on: bool) {
        self.creep_carries = on && self.sorted;
        if !self.creep_carries {
            self.creep_sp = Vec::new();
            self.creep_gross = Vec::new();
            return;
        }
        if self.creep_sp.len() != self.n * SPECIES {
            self.creep_sp = vec![0.0; self.n * SPECIES];
            self.creep_gross = vec![0.0; self.n];
        }
    }

    /// Whether material-aware hillslope creep is on.
    #[inline]
    pub fn is_material_creep(&self) -> bool {
        self.creep_carries
    }

    /// **What hillslope creep delivered to each cell in the last epoch, per
    /// species** (`n × SPECIES` metres, signed; empty when creep carries no
    /// identity). The colluvial half of the mixture [`Self::record`] names the
    /// arriving unit from.
    pub fn creep_species(&self) -> &[f64] {
        &self.creep_sp
    }

    /// **The largest relative gap between the creep itemisation and its own
    /// total**, over every (cell, epoch) of the run. `0.0` when creep carries no
    /// identity.
    pub fn max_creep_itemisation_residue(&self) -> f64 {
        self.creep_itemisation_residue
    }

    /// **The largest relative amount of any species creep created or destroyed**,
    /// over every (species, epoch) of the run. `0.0` when creep carries no
    /// identity. Creep only moves material, so the honest value is zero to
    /// round-off.
    pub fn max_creep_conservation_residue(&self) -> f64 {
        self.creep_conservation_residue
    }

    /// How many donor→receiver creep faces carried flux in the last epoch — the
    /// entry count a gravity-caused flow record would pay per chapter
    /// (stubs.md #18).
    pub fn creep_outflux_faces(&self) -> usize {
        self.creep_faces
    }

    /// **Flux-limited hillslope diffusion of the regolith, sub-cycled to the
    /// monotonicity bound** (journal/0122).
    ///
    /// The inner step is unchanged and is the gather form: fluxes are computed
    /// from a frozen surface and a frozen per-cell limiter, then each cell sums
    /// its own in/out edges. It conserves `ΣH` exactly (each edge's flux is
    /// referenced identically from both endpoints) and is byte-identical
    /// scalar↔parallel.
    ///
    /// # What was wrong, and what the sub-cycle fixes
    ///
    /// The step above is an **explicit** Laplacian, and an explicit Laplacian has
    /// a period-2 grid-scale mode whenever its per-edge coefficient exceeds
    /// [`CREEP_MAX_EDGE_COEFF`]. At `diffusion = 5.4` (the shipped rate under
    /// `EROSION_CALIBRATION`) the coefficient is **43× past** that bound, and the
    /// flux limiter — which caps a cell's export at its *entire inventory* rather
    /// than at the amount that would level the pair — turns the divergence into a
    /// **saturated flip-flop**: cell A hands B all of its cover, B is now higher
    /// and hands it back, forever, at an amplitude set by the cover rather than by
    /// the rate. That is exactly why journal/0116 measured the limiter to be
    /// *deaf to the time step* (96.0 → 94.7 % binding under a 4× refinement) while
    /// the regolith checkerboard *sharpened* (ACF −0.82 → −0.93).
    ///
    /// So the fix is **not** a smaller `dt` asked of the caller, and it is not a
    /// cap on the rate either — capping would reinstate the one-cell-per-epoch
    /// conveyor that `stubs.md` #27 is about. The pass takes the epoch it is given
    /// and **splits it internally** into `n` steps each of which respects the
    /// bound, with `n` derived from the rate the config actually states:
    ///
    /// ```text
    /// n = ceil( max_cell eff_diff / CREEP_MAX_EDGE_COEFF )
    /// ```
    ///
    /// `n = 1` reproduces the previous operator **bit for bit** (`x / 1.0 == x`),
    /// so every world whose peak effective diffusivity already sat inside the
    /// bound is untouched.
    ///
    /// **The shipped configuration is NOT such a world** — and the sentence that
    /// used to end this paragraph said it was (anti-shape A-2, caught by the
    /// 2026-07-29 spine-audit). journal/0122 measured the shipped world at **2.1×
    /// past** the bound; it takes **`n = 2`**, `DeepConfig::creep_substep` defaults
    /// **on**, and **its goldens moved with the fix**. A reader of this file alone
    /// would have concluded that no golden could have moved, which is exactly
    /// backwards. The reachable `n = 1` fixed point is
    /// `creep_substep: false` — asserted by name in `tests/creep_operator.rs`
    /// against `GOLDEN_SURFACE_UNBOUNDED_CREEP` / `GOLDEN_RECORD_UNBOUNDED_CREEP` —
    /// and `the_shipped_world_has_cells_past_the_bound` in the same file is the
    /// standing check that the claim above stays true.
    ///
    /// **Stability is therefore independent of the rate the caller states**, which
    /// is the property the brief asked for: the operator does not require a small
    /// `dt` to behave, because it no longer takes the caller's `dt` as its
    /// integration step — it takes `dt` as the *amount of world time to integrate*
    /// (RATE, journal/0123) and picks its own step inside it. The cost is `n×` the pass, paid at gen time, where this
    /// project's doctrine is explicit that time is not the constraint and
    /// simulation is never cheapened to save it.
    ///
    /// **The max is over cells, not over the config**, because [`eff_diff`] folds
    /// in biotic root cohesion and the lithology's creep susceptibility — a
    /// peat-dominated cell can carry several times the config rate, and it is the
    /// worst cell that decides whether *any* cell may oscillate. `f64::max` is
    /// order-independent, so the reduction cannot make the run non-deterministic.
    pub fn diffuse(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, dt: f64) {
        // **`dt` from outside; the sub-cycle from inside** (RATE, journal/0123).
        // `cfg.diffusion` is a rate *per epoch*; the phase length the runner hands
        // this pass says how many epochs this turn covers, so the diffusivity the
        // turn actually applies is the product. Then — and only then — the
        // stability sub-cycle below divides that down to the von Neumann bound.
        //
        // The two divisions are NOT the same knob and must stay in this order:
        // `dt` is **authored** (how much world time passed, a modelling choice),
        // `n_sub` is **derived** (how finely this operator must integrate it to
        // stay monotone, a numerical fact only the operator knows). That is the
        // split the user ruled on 2026-07-29 when RATE was nearly widened to own
        // substepping: *"couldn't substepping be solved within the field instead,
        // where it takes `dt` from outside and calcs its own internal multiplier?"*
        // Yes — and this is what that looks like. `stubs.md` § 30's remaining half
        // is hoisting the derived multiplier into the S-10 field-solver kernel so
        // no future pass author has to write the analysis; the authored half is
        // discharged here.
        //
        // `dt = 1.0` (the shipped roster) is bit-identical to the pre-RATE
        // operator: `x * 1.0 == x` exactly for f64, so `rate == cfg.diffusion` and
        // every quantity below is the same bit pattern it was.
        let rate = cfg.diffusion * dt;
        if rate <= 0.0 {
            return;
        }
        let d_max = self.max_eff_creep(grid, rate);
        let n_sub = if !cfg.creep_substep {
            // The pre-journal/0122 operator, bit for bit: one raw step at whatever
            // coefficient the config states. Kept reachable so the goldens captured
            // under it stay fixed points (`DeepConfig::creep_substep`).
            1
        } else if d_max > CREEP_MAX_EDGE_COEFF {
            // `ceil` of a finite positive ratio; the `max(1)` is belt-and-braces
            // against a denormal reduction, not a live case.
            ((d_max / CREEP_MAX_EDGE_COEFF).ceil() as u32).max(1)
        } else {
            1
        };
        self.creep_substeps = n_sub;
        self.creep_peak_coeff = d_max;
        let diff_sub = rate / f64::from(n_sub);
        // The species itemisation is audited against the epoch's TOTAL ΔH, so a
        // sub-cycled epoch needs somewhere to accumulate it. Allocated only when
        // both sub-cycling and identity-carrying creep are live, so the shipped
        // (n = 1) configuration's scratch residency does not move.
        let accumulate = n_sub > 1 && self.creep_carries;
        if accumulate {
            if self.netdiff_acc.len() == self.n {
                self.netdiff_acc.fill(0.0);
            } else {
                self.netdiff_acc = vec![0.0; self.n];
            }
        }
        self.creep_faces = 0;
        for s in 0..n_sub {
            self.diffuse_step(grid, cfg, diff_sub, s > 0);
            if accumulate {
                for (a, nd) in self.netdiff_acc.iter_mut().zip(self.netdiff.iter()) {
                    *a += *nd;
                }
            }
        }
        if self.creep_carries {
            self.audit_creep_species(accumulate);
        }
    }

    /// **The largest effective creep diffusivity anywhere on the grid** — the
    /// quantity [`diffuse`](Self::diffuse) divides by [`CREEP_MAX_EDGE_COEFF`] to
    /// pick its sub-cycle count. Read by the operator probe so the report can say
    /// *which* cells forced the count.
    pub fn max_eff_creep(&self, grid: &DeepGrid, diffusion: f64) -> f64 {
        let (resist, sus) = (&grid.bio_resist, &self.sus_creep);
        (0..self.n)
            .map(|i| eff_diff(diffusion, resist, sus, i))
            .fold(0.0f64, f64::max)
    }

    /// **How many sub-steps the last [`diffuse`](Self::diffuse) took.** `1` means
    /// the epoch's stated rate already sat inside [`CREEP_MAX_EDGE_COEFF`] and the
    /// operator was the pre-journal/0122 one bit for bit.
    #[inline]
    pub fn creep_substeps(&self) -> u32 {
        self.creep_substeps
    }

    /// **The peak effective creep diffusivity the last [`Self::diffuse`] divided
    /// down**, on the planes that epoch actually held. Use this, never a post-run
    /// [`Self::max_eff_creep`], to check the sub-step count against its own input:
    /// the biotic layer rewrites `bio_resist` after erosion, so a re-derivation is
    /// one epoch out of phase and disagrees by a few per cent.
    #[inline]
    pub fn creep_peak_coeff(&self) -> f64 {
        self.creep_peak_coeff
    }
}

#[cfg(test)]
mod hillslope_operator_tests {
    use super::*;

    /// A bare grid with flat bedrock and a **checkerboard regolith cover** — the
    /// smallest world on which the hypothesis journal/0116 refused to promote can
    /// be settled. Nothing here is stochastic and nothing is seeded: the whole
    /// experiment is arithmetic on one initial condition.
    fn checkerboard(w: usize, cover: f64) -> DeepGrid {
        let n = w * w;
        let mut grid = DeepGrid::from_parts(w, 460.0, vec![0.0; n], vec![0.0; n]);
        for i in 0..n {
            let (x, y) = (i % w, i / w);
            grid.h[i] = if (x + y) % 2 == 0 { cover } else { 0.0 };
        }
        grid
    }

    /// A config that carries **nothing but the diffusion coefficient** — every
    /// other phase is irrelevant because the test calls `diffuse` directly.
    fn creep_cfg(diffusion: f64) -> DeepConfig {
        DeepConfig {
            diffusion,
            erodibility: false,
            biotic: false,
            ..DeepConfig::default()
        }
    }

    /// The amplitude of the grid-scale mode over a window `margin` cells in from
    /// the border, signed by the parity that started high. Positive ⇒ still in
    /// phase with the initial condition; negative ⇒ **flipped**.
    ///
    /// **The margin is load-bearing, not tidiness.** A border cell has three
    /// neighbours (a corner two), so its amplification factor is not the interior
    /// one and its deviation contaminates any window it is in. That contamination
    /// spreads inward exactly one cell per step, so a window `margin` cells in is
    /// clean for `margin` steps — which is how many every test below takes. An
    /// earlier draft of this helper measured the whole interior and reported a
    /// *sign flip* on the shipped arm, which is the defect these tests exist to
    /// detect: the instrument was manufacturing the signature.
    const MARGIN: usize = 8;
    fn signed_amplitude(grid: &DeepGrid) -> f64 {
        let w = grid.w;
        let (lo, hi) = (MARGIN, w - MARGIN);
        // The window is even in both axes, so it holds equal counts of the two
        // parities and the window mean of an undisturbed checkerboard is exactly
        // `cover / 2` — no bias enters through the reference.
        debug_assert_eq!((hi - lo) % 2, 0);
        let mut sum = 0.0;
        let mut n = 0usize;
        for y in lo..hi {
            for x in lo..hi {
                sum += grid.h[y * w + x];
                n += 1;
            }
        }
        let mean = sum / n as f64;
        let mut acc = 0.0;
        for y in lo..hi {
            for x in lo..hi {
                let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
                acc += sign * (grid.h[y * w + x] - mean);
            }
        }
        acc / n as f64
    }

    /// **`dt` IS A TRUE MULTIPLIER OF THE RATE, TO THE BIT** (RATE, journal/0123).
    ///
    /// The claim the axis rests on is that a phase length is *time*: half the rate
    /// for twice as long is the same amount of creep. Asserted as **bit equality**
    /// rather than a tolerance, because it is not an approximation — `rate =
    /// cfg.diffusion × dt` is one multiply, and doubling is exact in binary
    /// floating point, so `0.12 × 2.0` and `0.24 × 1.0` are the same `f64`. A
    /// tolerance here would hide the interesting failure: the operator quietly
    /// dividing by `dt` somewhere else as well.
    ///
    /// Scale-free: a per-cell arithmetic identity, with no length in it.
    #[test]
    fn dt_scales_the_creep_rate_and_zero_stops_it() {
        let cover = 4.0;
        // Same product, stated two ways: an authored phase length of 2 epochs at
        // half the per-epoch rate, against one epoch at the whole rate.
        let mut slow = checkerboard(24, cover);
        let mut long_step = Erosion::new(&slow);
        long_step.diffuse(&mut slow, &creep_cfg(0.12), 2.0);

        let mut fast = checkerboard(24, cover);
        let mut one_epoch = Erosion::new(&fast);
        one_epoch.diffuse(&mut fast, &creep_cfg(0.24), 1.0);

        for (i, (a, b)) in slow.h.iter().zip(fast.h.iter()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "cell {i}: half the rate for twice the phase length must be the \
                 same creep ({a} != {b}) — `dt` is not scaling the rate"
            );
        }
        // And the sub-cycle count follows the *scaled* rate, not the config one:
        // a longer phase is a bigger step and needs finer integration, which is
        // the whole reason the two divisions have to compose.
        assert_eq!(long_step.creep_substeps(), one_epoch.creep_substeps());
        assert_eq!(
            long_step.creep_peak_coeff().to_bits(),
            one_epoch.creep_peak_coeff().to_bits()
        );

        // A zero-length phase moves nothing — the degenerate case that proves the
        // pass is reading `dt` at all rather than ignoring it in the common path.
        let mut still = checkerboard(24, cover);
        let before = still.h.clone();
        Erosion::new(&still).diffuse(&mut still, &creep_cfg(0.24), 0.0);
        assert_eq!(still.h, before, "a zero-length phase eroded something");
    }

    /// **The forcing passes are rate-shaped too**, and their `dt` scaling is exact
    /// for the same reason: one multiply against a per-epoch plane. The ledger
    /// total must scale with the grid, or the mass-conservation falsifier would be
    /// accounting for a different amount of time than the world got.
    #[test]
    fn dt_scales_the_uplift_plane_and_its_ledger_together() {
        // A grid with a non-uniform uplift plane and a flat bedrock datum.
        fn uplifting(w: usize) -> DeepGrid {
            let mut g = checkerboard(w, 0.0);
            g.uplift = (0..w * w).map(|i| 0.25 + i as f64 * 0.001).collect();
            g
        }
        let w = 16;
        let plane = uplifting(w).uplift;
        let expect: f64 = plane.iter().sum();

        let mut doubled = uplifting(w);
        let mut er = Erosion::new(&doubled);
        let ledger = er.apply_uplift(&mut doubled, 2.0);
        for (i, u) in plane.iter().enumerate() {
            assert_eq!(
                doubled.r[i].to_bits(),
                (*u * 2.0).to_bits(),
                "cell {i}: uplift did not scale by the phase length"
            );
        }
        assert_eq!(
            ledger.to_bits(),
            (expect * 2.0).to_bits(),
            "the ledger must account for the same phase length the grid got"
        );

        let mut zero = uplifting(w);
        let mut er = Erosion::new(&zero);
        assert_eq!(er.apply_uplift(&mut zero, 0.0), 0.0);
        assert!(
            zero.r.iter().all(|r| *r == 0.0),
            "a zero-length phase uplifted something"
        );
    }

    /// **THE HYPOTHESIS, ISOLATED.** journal/0116 wrote it and explicitly declined
    /// to promote it: *"a donor-cell scheme that moves everything downslope has a
    /// period-2 mode by construction — A gives all its cover to B, B is now higher
    /// and gives it back."* Every number in that entry was *consistent* with it and
    /// none of them isolated it, because every number came from a world in which
    /// eight other passes were also running.
    ///
    /// This is the isolation. Flat bedrock, a checkerboard of cover, the creep
    /// pass and nothing else, at the **calibrated** rate (`0.12 × 45`). The
    /// pre-journal/0122 operator's arithmetic is exact and can be read off by
    /// hand: a high cell's requested outflux is `4 × 5.4 × cover`, i.e. `21.6×` its
    /// own inventory, so the limiter binds and it ships **all** of it, `cover/4`
    /// per edge; each low cell has four high neighbours and therefore receives
    /// exactly `cover`. **The two populations swap, to the bit, forever.**
    ///
    /// Run under the sub-cycled operator the same initial condition decays
    /// monotonically and never changes sign — which is the other half of the
    /// claim: it is the *coefficient*, not the geometry, that made a checkerboard
    /// a fixed point of the pass.
    ///
    /// Scale-free: the argument is a two-population recurrence with no length in
    /// it, so a 12×12 grid tests it as completely as a 288×288 one.
    #[test]
    fn the_calibrated_rate_has_an_exact_period_2_mode_and_the_bound_removes_it() {
        let cover = 40.0;
        let calibrated = 0.12 * 45.0;
        let cfg = creep_cfg(calibrated);

        // --- the operator as it stood: one raw step at the stated rate ---------
        let mut grid = checkerboard(32, cover);
        let mut er = Erosion::new(&grid);
        let a0 = signed_amplitude(&grid);
        let mut legacy = Vec::new();
        for _ in 0..6 {
            er.diffuse_step(&mut grid, &cfg, calibrated, false);
            legacy.push(signed_amplitude(&grid));
        }
        // Period 2: strict sign alternation with the amplitude *conserved* rather
        // than decaying. Both halves matter — a decaying alternation would just be
        // a stable scheme resolving a rough initial condition.
        for (k, a) in legacy.iter().enumerate() {
            assert_eq!(
                a.is_sign_negative(),
                k % 2 == 0,
                "step {k}: the legacy operator did not alternate (amplitudes {legacy:?})"
            );
            assert!(
                (a.abs() - a0.abs()).abs() < 1e-9,
                "step {k}: amplitude {a} is not the initial {a0} — inside the clean \
                 window the flip-flop is exactly amplitude-preserving, which is what \
                 makes it a *mode* rather than a transient"
            );
        }

        // --- the sub-cycled operator, same rate, same initial condition --------
        let mut grid = checkerboard(32, cover);
        let mut er = Erosion::new(&grid);
        er.diffuse(&mut grid, &cfg, 1.0);
        assert_eq!(
            er.creep_substeps(),
            (calibrated / CREEP_MAX_EDGE_COEFF).ceil() as u32,
            "the sub-cycle count must follow from the rate, not from a table"
        );
        // 44 sub-steps at a per-edge coefficient of 5.4/44 = 0.1227 give the
        // grid-scale mode an amplification of `(1 − 8a)^44 = 0.0182^44`, i.e. it is
        // annihilated inside the first epoch rather than flipped. What the loop
        // then asserts is that it **stays** annihilated and never changes sign —
        // the residual is border relaxation diffusing inward, six orders of
        // magnitude below the mode it replaced.
        for step in 0..6 {
            let a = signed_amplitude(&grid);
            assert!(
                a > -a0 * 1e-6,
                "step {step}: the sub-cycled operator flipped the grid-scale mode \
                 ({a} against an initial {a0}) — the monotonicity bound is not holding"
            );
            assert!(
                a.abs() < a0 * 1e-3,
                "step {step}: the checkerboard is still here, {a} of {a0}"
            );
            er.diffuse(&mut grid, &cfg, 1.0);
        }
    }

    /// **And the shipped rate was never oscillating** — the fact that makes
    /// journal/0116's *"saturation alone is not sufficient"* qualification and this
    /// operator's bound the same statement. At `diffusion = 0.12` the limiter still
    /// binds (a cell with 4.6 m of cover on a steep edge wants to shed more than it
    /// has), but `0.12 < 1/8`, so the mode decays instead of flipping and the pass
    /// takes exactly one sub-step.
    #[test]
    fn the_shipped_rate_sits_inside_the_bound_and_decays() {
        let shipped = 0.12;
        assert!(shipped < CREEP_MAX_EDGE_COEFF);
        let cfg = creep_cfg(shipped);
        let mut grid = checkerboard(32, 4.6);
        let mut er = Erosion::new(&grid);
        let mut prev = signed_amplitude(&grid);
        for step in 0..8 {
            er.diffuse(&mut grid, &cfg, 1.0);
            assert_eq!(
                er.creep_substeps(),
                1,
                "step {step} sub-cycled at the shipped rate"
            );
            let a = signed_amplitude(&grid);
            assert!(
                a >= 0.0 && a < prev,
                "step {step}: {prev} → {a} is not a decay"
            );
            prev = a;
        }
    }

    /// **Mass is exact and nothing goes negative**, at any rate, sub-cycled or
    /// not. Non-negotiable, and scale-free: a per-cell predicate and a grid-wide
    /// sum, neither of which knows how big the world is.
    #[test]
    fn the_operator_conserves_mass_exactly_and_never_goes_negative() {
        for rate in [0.12, 1.0, 5.4, 40.0] {
            let cfg = creep_cfg(rate);
            // Rough bedrock the cover has to chase, so the limiter is genuinely
            // engaged rather than the flat case conserving mass trivially.
            let w = 16;
            let n = w * w;
            let r: Vec<f64> = (0..n)
                .map(|i| {
                    let (x, y) = ((i % w) as f64, (i / w) as f64);
                    100.0 - 3.0 * x + 7.0 * ((x * 0.7).sin() + (y * 1.3).cos())
                })
                .collect();
            let mut grid = DeepGrid::from_parts(w, 460.0, r, vec![0.0; n]);
            for i in 0..n {
                grid.h[i] = if i % 3 == 0 { 30.0 } else { 2.0 };
            }
            let before: f64 = grid.h.iter().sum();
            let mut er = Erosion::new(&grid);
            for _ in 0..20 {
                er.diffuse(&mut grid, &cfg, 1.0);
                assert!(
                    grid.h.iter().all(|v| *v >= -1e-9),
                    "rate {rate}: regolith went negative"
                );
            }
            let after: f64 = grid.h.iter().sum();
            assert!(
                (after - before).abs() / before < 1e-12,
                "rate {rate}: ΣH moved {before} → {after}"
            );
        }
    }

    /// **Order independence** — the property `water/sat.rs` gets "by construction"
    /// and this pass gets the same way: every flux is a pure function of a frozen
    /// surface and a frozen per-cell limiter, so the scalar and the data-parallel
    /// drivers must agree **bit for bit**, including across sub-steps, where a
    /// mistake in the accumulation would show up as a drift rather than as a
    /// wrong answer.
    #[test]
    fn the_sub_cycled_operator_is_bit_identical_scalar_and_parallel() {
        let cfg = creep_cfg(5.4);
        let mut a = checkerboard(64, 40.0);
        let mut b = checkerboard(64, 40.0);
        let mut ea = Erosion::new(&a);
        let mut eb = Erosion::new(&b);
        ea.set_parallel(false);
        eb.set_parallel(true);
        for _ in 0..4 {
            ea.diffuse(&mut a, &cfg, 1.0);
            eb.diffuse(&mut b, &cfg, 1.0);
        }
        assert_eq!(a.h, b.h, "scalar and parallel sub-cycling disagree");
    }

    /// **`n = 1` is the old operator, bit for bit** — `x / 1.0 == x`, asserted
    /// rather than argued. (This once read "the shipped world is untouched";
    /// that claim retired when `creep_substep` went default-ON with 2 sub-steps
    /// and the goldens moved. The predicate this test pins is unchanged —
    /// spine-audit residue fix, 2026-07-29.)
    #[test]
    fn inside_the_bound_the_driver_is_the_raw_step() {
        let cfg = creep_cfg(0.12);
        let mut a = checkerboard(32, 4.6);
        let mut b = checkerboard(32, 4.6);
        let mut ea = Erosion::new(&a);
        let mut eb = Erosion::new(&b);
        for _ in 0..5 {
            ea.diffuse(&mut a, &cfg, 1.0);
            eb.diffuse_step(&mut b, &cfg, 0.12, false);
        }
        assert_eq!(a.h, b.h);
    }
}
