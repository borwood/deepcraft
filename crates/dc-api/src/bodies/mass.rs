//! **The body mass integral (B6-a)** — mass, volume and centre of mass over a
//! body plan's segments, at a caller-supplied per-segment density.
//!
//! Spec: `docs/audits/2026-08-03-b6-body-composition-design.md` § 4 and its
//! § 6.3 sequencing row **B6-a** (*integral only, no roster, no renderer,
//! byte-identical*). Read its **mutable header** first — the pass predates the
//! 2026-08-04 term-space ruling and its § 4 facet-split recommendation is
//! withdrawn pending comparison.
//!
//! A **pure function of the body definition** — no world, no RNG, no clock —
//! so it is callable from any minting clock (pack build, deeptime worldgen,
//! define time), exactly like [`super::bake_resting_posture`] and
//! [`super::derive_joint_limits`] beside it (corrections #86). The result is
//! **returned, never stored** (S-3): a sibling of [`BodyPlan`], never a field
//! of it, never registry state, never sim-read. Its day-one consumer is
//! `bake_resting_posture`, whose CoM loop is now written **in terms of this
//! function** rather than beside it — the S-3 move `root_bob_m` cost us twice
//! (corrections #80, #93), and the reason this slice is worth building even
//! though no output moves.
//!
//! # What this is NOT, and the boundary is the point
//!
//! - **It never sees a material.** [`mass_properties`] takes `&[f64]` — a
//!   density per segment — and nothing else. **A-7 diagnostic, applied where
//!   it was tempting:** *what property is the integral reaching for?*
//!   **Density.** So density is what it takes, and no `MaterialId`, no
//!   material name and no registry lookup appears in this module or in any
//!   other body module.
//! - **It supplies INERTIA and no ACTUATION.** Mass, centre of mass and (in a
//!   later slice) moment of inertia are the *load* half of every force-shaped
//!   hole in the gait knobs. The *power* half — joint torque, muscle force,
//!   push-off impulse — is a property of an actuator, and no density is one.
//!   Marked in `stubs.md` as `an-inertia-with-no-actuation`.
//! - **It is not stiffness.** `stubs.md` #50: two materials of identical
//!   density can differ in elastic modulus by orders of magnitude, so
//!   `bob_damping` (`stubs.md` #44 S3) and the soft-tissue end-range
//!   (`stubs.md` #49) are **not** absorbed here, whatever those entries once
//!   claimed.
//! - **`subtree_inertia` is deliberately NOT here.** The design pass § 4
//!   proposes it *"and if [the `cadence_scale`] derivation is not in the same
//!   slice, `subtree_inertia` does not ship in it. A second function with no
//!   caller is A-4 with a nicer signature."* This slice does not derive
//!   `cadence_scale` (its actuation numerator does not exist), and the only
//!   site in the tree that would call it is the gait report test at
//!   `bodies/gait/tests/report.rs:306`, which is an instrument, not
//!   production. **Not built, on the pass's own condition.**
//!
//! # The invariance that makes this an S-5 conversion
//!
//! Stronger than an identity default, and it is why the acceptance is *not*
//! "a number moved": **a UNIFORM density factors out of every animation-side
//! output**, algebraically, not approximately.
//!
//! - centre of mass: `Σ(Vᵢ ρ cᵢ) / Σ(Vᵢ ρ) = Σ(Vᵢ cᵢ) / Σ(Vᵢ)` for common ρ;
//! - `root_height_ratio`: pure geometry — density never enters;
//! - a limb's free-swing frequency `ω = √(g M d / I)`: `M ∝ ρ`, `I ∝ ρ`, `d`
//!   independent of ρ, so ρ cancels **completely**.
//!
//! So the whole animation payoff of per-segment density is **heterogeneity**,
//! and a build that added a uniform density and reported the gait unchanged
//! would have confirmed an identity, not found a defect. At the identity
//! ρ ≡ 1.0 the result is **bit-identical** to the volume proxy it replaces
//! (`v * 1.0 == v` exactly), which is the acceptance this slice is held to.
//!
//! # A-1 ledger (each simplification, its identity, its heir)
//!
//! - **every density is 1.0** ([`segment_densities`]) — heir: **B6-c**, a
//!   declared `SegmentDef.composition` resolved against a materials roster.
//!   The seam exists so the proxy stops *being* the authority and starts
//!   *deriving from* one; the roster it would read does not exist, and a
//!   field nobody can fill is A-4 with a schema attached.
//! - **mass is a box product** — the segment is a cuboid of uniform density,
//!   so the centroid is the box centre. Heir: the same composition, whose
//!   radial order a future consumer may read (bounded at ≤ 2.6 % of a limb's
//!   swing inertia, design pass § 3.3).

use super::{BodyPlan, bake::offset_from_root};

#[cfg(test)]
mod tests;

/// Mass, volume and centre of mass of a body plan at the resting pose.
///
/// **Returned, never stored** (S-3) — a sibling of [`BodyPlan`], memoized by
/// consumers if they want it, never a field of the plan and never registry
/// state. Every field is a *derived* quantity: if the plan moves, this is
/// stale by construction, which is exactly why it is not kept.
#[derive(Clone, PartialEq, Debug)]
pub struct MassProperties {
    /// Summed segment box volume, m³. Independent of density — the geometry
    /// half, kept because it is the only quantity the pre-B6 code had and
    /// because a whole-body density (`mass_kg / volume_m3`) is the one
    /// externally checkable number a body has (design pass § 5.3: published,
    /// intensive, and the check a closed system cannot otherwise make).
    pub volume_m3: f64,
    /// Whole-body mass, kg — `Σ Vᵢ ρᵢ`. At the identity ρ ≡ 1.0 this is
    /// bit-identical to `volume_m3` and reads "0.168 kg" for the shipped
    /// biped, which is the stand-in speaking.
    ///
    /// ⚠ **Never a literature check.** The shipped biped is **2.43×** a human
    /// by volume (design pass § 5.3) — it is a caricature, and its mass is
    /// *supposed* to be off by that factor. Density is the checkable
    /// quantity; mass is not.
    pub mass_kg: f64,
    /// Per-segment mass, kg, **index-aligned with `plan.segments`** — the
    /// per-part half the whole-body figures are summed from. Consumers that
    /// need a subtree (a limb about its hip) build it from these rather than
    /// re-deriving box volumes.
    pub segment_mass_kg: Vec<f64>,
    /// Centre of mass at the resting pose, metres, ground frame (y = 0 at the
    /// ground, x/z body-local) — the frame [`super::RestingPosture::com_m`]
    /// already uses.
    ///
    /// Meaningful only when `mass_kg > 0`; a massless plan divides by zero and
    /// the caller is expected to have refused first (the resting bake does).
    pub com_m: [f64; 3],
}

impl MassProperties {
    /// Whole-body bulk density, kg/m³ — `mass_kg / volume_m3`.
    ///
    /// **The one externally checkable number a body has.** Whole-body density
    /// is published, intensive and measurable (≈ 1010 kg/m³ for a human
    /// including residual lung gas; 1040–1070 by hydrostatic weighing after
    /// correcting for it — the two conventions measure different things).
    /// A plan whose mixture integrates far outside that band is telling you
    /// the *mixture* is wrong, and that is the only claim this comparison
    /// supports (CLAUDE.md § *A closed system cannot detect its own scale
    /// error*).
    ///
    /// At the identity ρ ≡ 1.0 it is exactly 1.0 and says nothing.
    pub fn bulk_density_kg_m3(&self) -> f64 {
        self.mass_kg / self.volume_m3
    }
}

/// Per-segment bulk density, index-aligned with `plan.segments`.
///
/// **The named seam whose identity is 1.0** (S-5). Until a segment can declare
/// what it is made of, every density is exactly 1.0 and [`mass_properties`]
/// reproduces the volume proxy *bit for bit* — `stubs.md` #40's stand-in, no
/// longer a literal in the middle of a CoM loop but a function with a name, a
/// documented identity and a named heir.
///
/// **Heir: B6-c** — `SegmentDef.composition` (an ordered, outward-going
/// mixture of `{ material, share }`, the user's `ideas.md` shape) resolved
/// against a materials roster by declared name. That roster does not exist:
/// there is no tissue material and no `define_material` door, so a
/// composition field would be unfillable today. When it lands, **only this
/// function changes** — the integral takes `&[f64]` and never learns what a
/// material is (A-7).
pub fn segment_densities(plan: &BodyPlan) -> Vec<f64> {
    vec![1.0; plan.segments.len()]
}

/// Mass, volume and centre of mass over **all** segments of `plan` at the
/// resting pose, with `root_height_m` placing the root above the ground.
///
/// `densities` is index-aligned with `plan.segments` — normally
/// [`segment_densities`]'s output. Each segment is a cuboid of uniform
/// density, so its mass is `size_m.product() * ρ` and its centroid is the box
/// centre; the whole-body centre of mass is the mass-weighted mean of those
/// centres.
///
/// **All** segments participate, including ones that carry no bearing role —
/// the stout's knuckle-dragging arms are in the CoM and not in the support
/// polygon, which is the mode axis doing its job.
///
/// # Panics
///
/// If `densities.len() != plan.segments.len()`. The pairing is positional and
/// a mismatch is a programming error, not authored data: a silent fallback to
/// 1.0 for a missing entry would be a wrong answer wearing an identity
/// default's clothes (A-3).
pub fn mass_properties(plan: &BodyPlan, densities: &[f64], root_height_m: f64) -> MassProperties {
    assert_eq!(
        densities.len(),
        plan.segments.len(),
        "plan `{}`: densities are index-aligned with segments ({} given for {} segments)",
        plan.name,
        densities.len(),
        plan.segments.len()
    );
    let mut volume_m3 = 0.0_f64;
    let mut mass_kg = 0.0_f64;
    let mut moment = [0.0_f64; 3];
    let mut segment_mass_kg = Vec::with_capacity(plan.segments.len());
    for (s, rho) in plan.segments.iter().zip(densities) {
        let v = s.size_m[0] * s.size_m[1] * s.size_m[2];
        let m = v * rho;
        let base = offset_from_root(plan, s);
        let center = [
            base[0] + s.offset_m[0],
            root_height_m + base[1] + s.offset_m[1],
            base[2] + s.offset_m[2],
        ];
        volume_m3 += v;
        mass_kg += m;
        segment_mass_kg.push(m);
        for (acc, c) in moment.iter_mut().zip(center) {
            *acc += m * c;
        }
    }
    MassProperties {
        volume_m3,
        mass_kg,
        segment_mass_kg,
        com_m: [
            moment[0] / mass_kg,
            moment[1] / mass_kg,
            moment[2] / mass_kg,
        ],
    }
}
