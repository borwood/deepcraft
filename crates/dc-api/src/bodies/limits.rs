//! **Joint rotation limits (B7)** — the declaration vocabulary, and the
//! derived default that fills in whatever a body does not declare.
//!
//! Spec: `docs/audits/2026-08-02-joint-limits-b7-design.md` (the design pass and
//! its four ruling banners); ruling: `docs/design/bodies.md` § *Joint rotation
//! limits* (DECIDED 2026-08-02, user).
//!
//! A **pure function of the body definition** — no world, no RNG, no clock — so
//! it is callable from any minting clock (pack build, deeptime worldgen, define
//! time), exactly like [`super::bake_resting_posture`] beside it (corrections
//! #86). The result is **returned, never stored** (S-3): a sibling of
//! [`BodyPlan`], never a field of it, memoized by consumers.
//!
//! # The law — L3, "the child chain's distal extent must not enter an ancestor"
//!
//! For a joint (a segment) rotating about one segment-local axis, take the
//! **distal points** of the subtree hanging off it — the corners of each
//! subtree box that are farthest from the joint's own pivot, plus every
//! declared role anchor — and find the first rotation angle at which any of
//! them is strictly inside any **strict ancestor's** box. That angle is the
//! derived bound; the two rotation directions are searched independently, so
//! the law can in principle return an asymmetric range.
//!
//! **Why distal points and not whole boxes** (audit § 3.1's L1): a parent and
//! child box *abut at the pivot* by construction, so any ε > 0 sweeps the
//! child's PROXIMAL corner into the parent and a full box-vs-box test returns
//! 0° for every joint on every plan. Rescuing that needs an exclusion radius,
//! which is a tuning constant with no derivation. Testing only the far end of
//! each box is the tuning-free escape: those points start a whole bone-length
//! from the pivot.
//!
//! **⚠ WHERE THIS IMPLEMENTATION DIVERGES FROM THE DESIGN DOCUMENT, and why.**
//! The audit's § 3.1 prose says *"the eight corners of the child subtree's
//! TERMINAL box"*; its § 3.2 table cannot be reproduced by that rule, and is
//! not internally consistent either:
//!
//! - *"eight corners"* is degenerate — the terminal box's proximal corners are
//!   the L1 failure the law exists to avoid (the biped knee returns 0°, not
//!   155°, if they are tested). Every row of the table is reproducible only
//!   with the **distal** corners.
//! - *"terminal box"* contradicts the table's own hip rows, which are derived
//!   from the **thigh's** distal corner (the rotating segment's own box) — the
//!   shank, the actual terminal box, never re-enters the trunk at all. Drop the
//!   rotating segment's own box and both hip rows become `Undetermined`, taking
//!   § 3.2's headline finding (stout 77° vs biped 150°) and § 5.2's number for
//!   gait member #1 with them.
//! - The neck rows go the other way: they are derived from the **head's**
//!   distal corner with the neck's own box omitted. Include it — as the hip
//!   rows do — and it binds first.
//!
//! The only self-consistent reading is **every box in the rotating subtree,
//! distal corners only**, and that is what is built here. The measured
//! divergences are stamped on the audit's header (mutable header, immutable
//! body).
//!
//! # What this can and cannot claim (audit § 1, § 3.5)
//!
//! It is an **impossibility bound**, not an anatomy model: it forbids only
//! poses that put the body's own boxes through each other. Where it has no
//! answer it says [`Bound::Undetermined`] with a reason — a loud, named
//! absence, never a silent ∞.
//!
//! ⚠ STAND-IN — five entries in `stubs.md`, all added with this module:
//!
//! - **B7-a** `the-fold-sense-is-declared-because-our-bodies-have-no-front` —
//!   it supplies a hinge's magnitude and structurally **cannot supply its
//!   SIGN**: every shipped plan has `pivot_m[2] = offset_m[2] = 0` on every
//!   segment, so the bodies are mirror-symmetric fore-and-aft and there is no
//!   anterior datum. Every derived range here is exactly symmetric.
//! - **B7-b** `the-derived-limit-tests-distal-extent-only` — ancestors only, so
//!   **shaft contact** (`dc:body/longleg`'s hip) and **siblings** (a shank
//!   through the other thigh) are missed; heir is a swept-volume test with an
//!   articular neighbourhood, which is B6-adjacent.
//! - **B7-c** `the-euler-box-over-approximates-a-ball-joint` — heir: swing-cone
//!   + twist. Marked at [`DofDef`].
//! - **B7-d** `a-dof-axis-can-only-be-x-y-or-z` — heir: a non-Euler pose.
//!   Marked at [`Axis`].
//! - **B7-e** `a-limit-that-cannot-know-soft-tissue` — the derivation is in band
//!   where an end-range is **bony** and over-predicts by 3–7× where it is
//!   **ligamentous**, because at density ≡ 1 there is no force to model passive
//!   tissue with. Heir: **B6** (and it is the third member of `stubs.md`
//!   `mass-is-volume-until-b6`).

use super::bake::offset_from_root;
use super::{Axis, BodyPlan, DofDef, SegmentDef};

/// One end of one degree of freedom, with its provenance attached — the whole
/// point of the type (a validator error has to say *which end was declared and
/// which was derived*, audit § 4.3).
#[derive(Clone, PartialEq, Debug)]
pub enum Bound {
    /// Authored on the plan. Overrides the derivation for this end alone
    /// (audit § 4.2 — composition is **per end of per axis**).
    Declared(f64),
    /// Derived by L3 from the plan's own geometry, radians.
    Derived(f64),
    /// The derivation has no answer here, and says why. Treated as unbounded by
    /// every consumer — a legal absence (the `NothingDeclared` shape the
    /// resting bake already ships), never a silent infinity.
    Undetermined {
        /// Names the geometry that made the question unanswerable.
        reason: String,
    },
}

impl Bound {
    /// The bound in radians, or `None` when it is [`Bound::Undetermined`].
    pub fn value(&self) -> Option<f64> {
        match self {
            Bound::Declared(v) | Bound::Derived(v) => Some(*v),
            Bound::Undetermined { .. } => None,
        }
    }

    /// `"DECLARED"` / `"derived"` / `"undetermined"` — for error messages.
    pub fn provenance(&self) -> &'static str {
        match self {
            Bound::Declared(_) => "DECLARED",
            Bound::Derived(_) => "derived",
            Bound::Undetermined { .. } => "undetermined",
        }
    }

    fn describe(&self) -> String {
        match self {
            Bound::Declared(v) => format!("{v:+.6} rad ({:+.3}°, DECLARED)", v.to_degrees()),
            Bound::Derived(v) => format!("{v:+.6} rad ({:+.3}°, derived)", v.to_degrees()),
            Bound::Undetermined { reason } => format!("unbounded (undetermined: {reason})"),
        }
    }
}

/// One resolved degree of freedom: an axis and its two independently-resolved
/// ends.
#[derive(Clone, PartialEq, Debug)]
pub struct DofLimit {
    pub axis: Axis,
    pub min: Bound,
    pub max: Bound,
}

impl DofLimit {
    /// Is `v` inside this DOF's range? An [`Bound::Undetermined`] end is
    /// unbounded.
    pub fn contains(&self, v: f64) -> bool {
        self.min.value().is_none_or(|lo| v >= lo) && self.max.value().is_none_or(|hi| v <= hi)
    }

    /// The largest magnitude this DOF admits, or `None` when either end is
    /// unbounded — the quantity the IK's reachable **sector** is cut from
    /// (audit § 5.3).
    pub fn max_magnitude(&self) -> Option<f64> {
        match (self.min.value(), self.max.value()) {
            (Some(lo), Some(hi)) => Some(lo.abs().max(hi.abs())),
            _ => None,
        }
    }
}

/// One joint's resolved limits. **An EMPTY `dofs` is a WELD**, not an absence:
/// the segment does not rotate at all (audit § 4.2's `Some([])`, and § 4.4's
/// derived default for the root).
#[derive(Clone, PartialEq, Debug)]
pub struct JointLimit {
    /// Segment (joint) name within the plan.
    pub segment: String,
    /// The degrees of freedom this joint has. An axis absent from this list is
    /// **not a DOF** — a rotation about it is a define-time error, which is the
    /// distinction a plain Euler box (audit § 2.2's candidate A) cannot make.
    pub dofs: Vec<DofLimit>,
    /// True when the DOF *set* came from a declaration (`SegmentDef::dofs` was
    /// `Some(..)`), so an error can say whether a missing axis was authored
    /// away or simply never derived.
    pub dofs_declared: bool,
}

impl JointLimit {
    /// This joint's DOF on `axis`, or `None` when the axis is not a DOF.
    pub fn dof(&self, axis: Axis) -> Option<&DofLimit> {
        self.dofs.iter().find(|d| d.axis == axis)
    }

    /// The DOF set, rendered for an error message.
    fn dof_set(&self) -> String {
        if self.dofs.is_empty() {
            "none — this joint is WELDED".to_string()
        } else {
            self.dofs
                .iter()
                .map(|d| d.axis.name())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    /// Check one XYZ Euler triple against this joint. `Ok(())`, or a message
    /// naming the joint, the axis, the value, both bounds, and **which end was
    /// declared versus derived** (audit § 4.3's two worked examples).
    pub fn check(&self, euler: [f64; 3]) -> Result<(), String> {
        for (i, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            let v = euler[i];
            match self.dof(axis) {
                None => {
                    if v != 0.0 {
                        return Err(format!(
                            "joint `{}` is rotated about {} by {v:+.6} rad ({:+.3}°), but {} \
                             degrees of freedom: {}. Declare a {} DOF or key only the axes it \
                             has.",
                            self.segment,
                            axis.name(),
                            v.to_degrees(),
                            if self.dofs_declared {
                                "it declares"
                            } else {
                                "it has"
                            },
                            self.dof_set(),
                            axis.name(),
                        ));
                    }
                }
                Some(d) => {
                    if !d.contains(v) {
                        return Err(format!(
                            "joint `{}` is rotated about {} by {v:+.6} rad ({:+.3}°), outside \
                             its range [{}, {}]",
                            self.segment,
                            axis.name(),
                            v.to_degrees(),
                            d.min.describe(),
                            d.max.describe(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// A plan's resolved joint limits — **returned, not stored** (S-3). Declared
/// bounds have already overridden derived ones per end; consumers read this and
/// never re-resolve.
#[derive(Clone, PartialEq, Debug)]
pub struct JointLimits {
    /// One entry per segment, in plan order.
    pub joints: Vec<JointLimit>,
    /// Band reports (audit § 4.3 check 4): a declared bound OUTSIDE the derived
    /// one is **reported, not refused** — the rule the gait docket ratified for
    /// band violations (*a clockwork golem may want to step wrong*). Each names
    /// both numbers.
    pub reports: Vec<String>,
}

impl JointLimits {
    /// The limits of one joint by segment name.
    pub fn joint(&self, segment: &str) -> Option<&JointLimit> {
        self.joints.iter().find(|j| j.segment == segment)
    }

    /// [`JointLimit::check`] for a named joint. A joint this plan does not have
    /// is not this function's error to report — `validate_plan` already rejects
    /// a clip animating an unknown joint — so it passes.
    pub fn check(&self, segment: &str, euler: [f64; 3]) -> Result<(), String> {
        match self.joint(segment) {
            Some(j) => j.check(euler),
            None => Ok(()),
        }
    }
}

// ------------------------------------------------------------ the geometry --

/// An axis-aligned box in some joint's pivot frame.
#[derive(Clone, Copy, Debug)]
struct Box3 {
    lo: [f64; 3],
    hi: [f64; 3],
}

impl Box3 {
    fn of(center: [f64; 3], half: [f64; 3]) -> Self {
        Box3 {
            lo: [
                center[0] - half[0],
                center[1] - half[1],
                center[2] - half[2],
            ],
            hi: [
                center[0] + half[0],
                center[1] + half[1],
                center[2] + half[2],
            ],
        }
    }
}

/// Ties in "farthest corner" are exact for a z-centred box (four corners share
/// the maximum), so the comparison needs a hair of slack rather than `==`.
const DISTAL_EPS_M: f64 = 1e-12;

/// The corners of `b` farthest from the origin — **the distal extent**. See the
/// module doc for why the proximal corners are excluded.
fn distal_corners(b: &Box3) -> Vec<[f64; 3]> {
    let mut corners = Vec::with_capacity(8);
    for &x in &[b.lo[0], b.hi[0]] {
        for &y in &[b.lo[1], b.hi[1]] {
            for &z in &[b.lo[2], b.hi[2]] {
                corners.push([x, y, z]);
            }
        }
    }
    let d2 = |p: &[f64; 3]| p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
    let max = corners.iter().map(d2).fold(0.0_f64, f64::max);
    corners
        .into_iter()
        .filter(|p| d2(p) >= max - DISTAL_EPS_M)
        .collect()
}

/// Wrap into `[0, 2π)`.
fn wrap_tau(a: f64) -> f64 {
    a.rem_euclid(std::f64::consts::TAU)
}

/// Every angle `ψ ∈ [0, 2π)` at which `R·cos ψ` (or `R·sin ψ`, when `sine`)
/// crosses `c` — the boundary crossings of one box face.
fn crossings(r: f64, c: f64, sine: bool, out: &mut Vec<f64>) {
    if r <= 0.0 || c.abs() > r {
        return;
    }
    let t = (c / r).clamp(-1.0, 1.0);
    if sine {
        let a = t.asin();
        out.push(wrap_tau(a));
        out.push(wrap_tau(std::f64::consts::PI - a));
    } else {
        let a = t.acos();
        out.push(wrap_tau(a));
        out.push(wrap_tau(-a));
    }
}

/// Why one (point, ancestor) pair never produced an entry — the raw material
/// for a [`Bound::Undetermined`] reason.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Miss {
    /// The rotation-invariant coordinate never overlaps the ancestor's extent.
    InvariantAxis,
    /// The swept circle never reaches one of the ancestor's rotating faces.
    OutOfRadius,
    /// Both extents are individually reachable but never simultaneously.
    NeverSimultaneous,
    /// The point is **already inside** the ancestor at rest, so it never
    /// *enters*. This is not a limit and must not be reported as one — see
    /// [`first_entry`]'s note.
    AlreadyInside,
}

/// The first angle, searching in `dir` (`+1` / `-1`) from 0 and stopping at
/// ±π, at which `p` rotated about `axis` is **strictly inside** `b`.
///
/// Closed form: a rotation about one axis maps the other two coordinates as
/// `p' = R·cos(θ+φ)`, `q' = R·sin(θ+φ)`, so every face crossing is one `acos`
/// or `asin`. The crossings partition the circle; the first arc whose midpoint
/// is inside gives the entry angle exactly (its own left endpoint) — no search,
/// no tolerance.
///
/// The ±π cap is not a magic number: an Euler component is read wrapped into
/// `(-π, π]`, so a bound of larger magnitude constrains nothing that the other
/// end does not already constrain.
///
/// **A point already inside the box at θ = 0 is [`Miss::AlreadyInside`], never a
/// zero limit.** The authored rest is the reference the law measures departure
/// from; a rest-pose overlap is the author's geometry, not a joint's range, and
/// reporting it as `[0, 0]` would weld a limb for a reason that has nothing to
/// do with rotation. `dc:body/stout` is exactly this case and it took a build to
/// find: its upper arms are drawn **flush** against its trunk (arm
/// `x ∈ [0.39, 0.55]` against trunk `x ∈ [-0.39, 0.39]` — the audit's own
/// *"zero-measure graze at x = 0.39 exactly"*), and in f64 that graze lands a
/// few ulps INSIDE. Read as an entry it welded both shoulders and rejected the
/// shipped idle clip on the shipped pack.
fn first_entry(p: [f64; 3], b: &Box3, axis: Axis, dir: f64) -> Result<f64, Miss> {
    let ia = axis.index();
    let ip = (ia + 1) % 3;
    let iq = (ia + 2) % 3;
    // Contact at rest, within the accumulated-rounding epsilon: the graze is
    // decided by ULPs otherwise, and `dc:body/stout`'s two shoulders landed on
    // OPPOSITE sides of it (one flush-inside, one flush-outside → a 0° bound
    // on one arm and not the other). `EPS_M` is the resting bake's own
    // "exactly equal, up to accumulated f64 rounding" epsilon, borrowed with
    // its derivation intact: a nanometre, against authored geometry that
    // differs by millimetres at least.
    let e = super::bake::EPS_M;
    if (0..3).all(|i| b.lo[i] - e < p[i] && p[i] < b.hi[i] + e) {
        return Err(Miss::AlreadyInside);
    }
    // The rotation-invariant coordinate: if it misses, no angle can help.
    if !(b.lo[ia] < p[ia] && p[ia] < b.hi[ia]) {
        return Err(Miss::InvariantAxis);
    }
    let r = (p[ip] * p[ip] + p[iq] * p[iq]).sqrt();
    let phi = p[iq].atan2(p[ip]);
    let inside = |psi: f64| {
        let (u, v) = (r * psi.cos(), r * psi.sin());
        b.lo[ip] < u && u < b.hi[ip] && b.lo[iq] < v && v < b.hi[iq]
    };
    // A point on the rotation axis itself never moves.
    if r <= 0.0 {
        return Err(Miss::OutOfRadius);
    }
    let mut cuts: Vec<f64> = Vec::with_capacity(8);
    crossings(r, b.lo[ip], false, &mut cuts);
    crossings(r, b.hi[ip], false, &mut cuts);
    crossings(r, b.lo[iq], true, &mut cuts);
    crossings(r, b.hi[iq], true, &mut cuts);
    if cuts.is_empty() {
        // No face is ever crossed, and rest is outside: never inside.
        return Err(Miss::OutOfRadius);
    }
    // Walk the circle from ψ0 = φ in `dir`, arc by arc; the first arc whose
    // interior is inside the box starts at the entry angle.
    let psi0 = wrap_tau(phi);
    let mut offsets: Vec<f64> = cuts
        .iter()
        .map(|c| {
            let d = wrap_tau(if dir > 0.0 { c - psi0 } else { psi0 - c });
            if d <= 0.0 { std::f64::consts::TAU } else { d }
        })
        .collect();
    offsets.push(std::f64::consts::TAU);
    offsets.sort_by(|a, b| a.partial_cmp(b).expect("finite crossing offsets"));
    offsets.dedup();
    let mut start = 0.0_f64;
    for &end in &offsets {
        let mid = 0.5 * (start + end);
        if inside(psi0 + dir * mid) {
            return if start <= std::f64::consts::PI {
                Ok(start)
            } else {
                Err(Miss::NeverSimultaneous)
            };
        }
        start = end;
    }
    Err(Miss::NeverSimultaneous)
}

/// `seg` and every descendant, in plan order.
fn subtree<'a>(plan: &'a BodyPlan, seg: &'a SegmentDef) -> Vec<&'a SegmentDef> {
    let mut out = vec![seg];
    let mut i = 0;
    while i < out.len() {
        let name = out[i].name.clone();
        for s in &plan.segments {
            if s.parent.as_deref() == Some(name.as_str()) && !out.iter().any(|o| o.name == s.name) {
                out.push(s);
            }
        }
        i += 1;
        if i > plan.segments.len() {
            break; // an unvalidated plan's cycle; validate_plan rejects those
        }
    }
    out
}

/// Every point of the subtree hanging off `seg`, in `seg`'s pivot frame: the
/// distal corners of each subtree box plus every declared role anchor.
fn distal_points(plan: &BodyPlan, seg: &SegmentDef) -> Vec<[f64; 3]> {
    let origin = offset_from_root(plan, seg);
    let mut out = Vec::new();
    for d in subtree(plan, seg) {
        let base = offset_from_root(plan, d);
        let center = [
            base[0] + d.offset_m[0] - origin[0],
            base[1] + d.offset_m[1] - origin[1],
            base[2] + d.offset_m[2] - origin[2],
        ];
        let half = [d.size_m[0] / 2.0, d.size_m[1] / 2.0, d.size_m[2] / 2.0];
        out.extend(distal_corners(&Box3::of(center, half)));
        for r in &d.roles {
            if let Some(at) = r.at_m {
                out.push([
                    base[0] + at[0] - origin[0],
                    base[1] + at[1] - origin[1],
                    base[2] + at[2] - origin[2],
                ]);
            }
        }
    }
    out
}

/// Every strict ancestor of `seg` as a named box in `seg`'s pivot frame.
fn ancestor_boxes<'a>(plan: &'a BodyPlan, seg: &'a SegmentDef) -> Vec<(&'a str, Box3)> {
    let origin = offset_from_root(plan, seg);
    let mut out = Vec::new();
    let mut cur = seg;
    let mut hops = 0;
    while let Some(p) = cur
        .parent
        .as_deref()
        .and_then(|n| plan.segments.iter().find(|s| s.name == n))
    {
        let base = offset_from_root(plan, p);
        let center = [
            base[0] + p.offset_m[0] - origin[0],
            base[1] + p.offset_m[1] - origin[1],
            base[2] + p.offset_m[2] - origin[2],
        ];
        let half = [p.size_m[0] / 2.0, p.size_m[1] / 2.0, p.size_m[2] / 2.0];
        out.push((p.name.as_str(), Box3::of(center, half)));
        cur = p;
        hops += 1;
        if hops > plan.segments.len() {
            break;
        }
    }
    out
}

/// The derived bound for one joint about one axis in one direction, or the
/// reason there is none.
fn derived_bound(plan: &BodyPlan, seg: &SegmentDef, axis: Axis, dir: f64) -> Bound {
    let ancestors = ancestor_boxes(plan, seg);
    if ancestors.is_empty() {
        return Bound::Undetermined {
            reason: format!("`{}` has no ancestor segment to collide with", seg.name),
        };
    }
    let points = distal_points(plan, seg);
    let mut best: Option<f64> = None;
    let mut misses: Vec<String> = Vec::new();
    for (name, b) in &ancestors {
        let mut anc_best: Option<f64> = None;
        let mut anc_miss = Miss::NeverSimultaneous;
        for p in &points {
            match first_entry(*p, b, axis, dir) {
                Ok(t) => anc_best = Some(anc_best.map_or(t, |x: f64| x.min(t))),
                // Most-specific miss wins the reason: an already-inside point
                // is the sharpest thing to say, then an axis that never
                // overlaps, then a radius that never reaches.
                Err(Miss::AlreadyInside) => anc_miss = Miss::AlreadyInside,
                Err(Miss::InvariantAxis) => {
                    if anc_miss != Miss::AlreadyInside {
                        anc_miss = Miss::InvariantAxis;
                    }
                }
                Err(Miss::OutOfRadius) => {
                    if matches!(anc_miss, Miss::NeverSimultaneous) {
                        anc_miss = Miss::OutOfRadius;
                    }
                }
                Err(Miss::NeverSimultaneous) => {}
            }
        }
        match anc_best {
            Some(t) => best = Some(best.map_or(t, |x: f64| x.min(t))),
            None => misses.push(format!(
                "`{name}`: {}",
                match anc_miss {
                    Miss::InvariantAxis => format!(
                        "the chain's {} extent is invariant under a {} rotation and never \
                         overlaps this segment",
                        axis.name(),
                        axis.name()
                    ),
                    Miss::OutOfRadius =>
                        "the chain's distal extent sweeps a circle that never reaches this \
                         segment's faces"
                            .to_string(),
                    Miss::NeverSimultaneous =>
                        "the chain's distal extent clears this segment (its extents are never \
                         satisfied at the same angle)"
                            .to_string(),
                    Miss::AlreadyInside =>
                        "the chain's distal extent already intersects this segment at REST — a \
                         graze in the authored geometry, not a rotation limit"
                            .to_string(),
                }
            )),
        }
    }
    match best {
        Some(t) => Bound::Derived(dir * t),
        None => Bound::Undetermined {
            reason: format!(
                "no rotation of `{}` about {} within ±180° brings its chain's distal extent \
                 inside an ancestor — {}",
                seg.name,
                axis.name(),
                misses.join("; ")
            ),
        },
    }
}

// ------------------------------------------------------------ the function --

/// Derive `plan`'s joint limits, with any DECLARED bound overriding the derived
/// one **per end of per axis** (audit § 4.2 — the finest granularity that is
/// still meaningful, so a declaration is never larger than the fact it asserts
/// and an author never has to restate a derived number).
///
/// - `SegmentDef::dofs == None` on a non-root segment → three DOFs, all six
///   bounds derived. **The identity default** (S-5).
/// - `SegmentDef::dofs == None` on the ROOT → **welded**, zero DOFs (audit
///   § 4.4, ruled provisionally 2026-08-03: body orientation belongs to the
///   facing system, so a clip keying the root is a define-time error).
/// - `Some([])` → welded.
/// - `Some(v)` → exactly those axes; any axis not listed is **not a DOF**.
///
/// Pure and deterministic; a handful of `asin`/`acos` per joint per ancestor,
/// **once per plan**, and the per-frame path gains zero work. Consumers
/// memoize; nothing stores it on the plan (S-3).
pub fn derive_joint_limits(plan: &BodyPlan) -> JointLimits {
    let mut joints = Vec::with_capacity(plan.segments.len());
    let mut reports = Vec::new();
    for seg in &plan.segments {
        let derive = |axis: Axis| DofLimit {
            axis,
            min: derived_bound(plan, seg, axis, -1.0),
            max: derived_bound(plan, seg, axis, 1.0),
        };
        let (dofs, dofs_declared) = match &seg.dofs {
            None if seg.parent.is_none() => (Vec::new(), false),
            None => (
                vec![derive(Axis::X), derive(Axis::Y), derive(Axis::Z)],
                false,
            ),
            Some(declared) => {
                let mut out = Vec::with_capacity(declared.len());
                for d in declared {
                    let mut resolved = derive(d.axis);
                    resolve_end(
                        plan,
                        seg,
                        d,
                        &mut resolved.min,
                        d.min_rad,
                        true,
                        &mut reports,
                    );
                    resolve_end(
                        plan,
                        seg,
                        d,
                        &mut resolved.max,
                        d.max_rad,
                        false,
                        &mut reports,
                    );
                    out.push(resolved);
                }
                (out, true)
            }
        };
        joints.push(JointLimit {
            segment: seg.name.clone(),
            dofs,
            dofs_declared,
        });
    }
    JointLimits { joints, reports }
}

/// Apply one declared end over its derived counterpart, recording a band report
/// when the declaration lies OUTSIDE the derivation (audit § 4.3 check 4 —
/// reported, never refused).
fn resolve_end(
    plan: &BodyPlan,
    seg: &SegmentDef,
    dof: &DofDef,
    slot: &mut Bound,
    declared: Option<f64>,
    is_min: bool,
    reports: &mut Vec<String>,
) {
    let Some(v) = declared else { return };
    if let Bound::Derived(d) = slot {
        let outside = if is_min { v < *d } else { v > *d };
        if outside {
            reports.push(format!(
                "plan `{}` joint `{}` declares a {} bound about {} of {v:+.6} rad ({:+.3}°) \
                 OUTSIDE its derived self-contact bound of {d:+.6} rad ({:+.3}°) — reported, \
                 not refused: a clockwork golem may want to bend wrong",
                plan.name,
                seg.name,
                if is_min { "lower" } else { "upper" },
                dof.axis.name(),
                v.to_degrees(),
                d.to_degrees(),
            ));
        }
    }
    *slot = Bound::Declared(v);
}

#[cfg(test)]
mod tests;
