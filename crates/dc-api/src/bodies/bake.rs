//! The resting-posture bake — member #0 of the posture-gait tier
//! (`docs/design/posture-gait.md` § 7; spec: the greenlit design pass
//! `docs/audits/2026-08-02-posture-bake-member0-design.md`).
//!
//! A **pure function of the body definition** — no world input, no RNG, no
//! clock — so it is callable from ANY minting clock (pack build, deeptime
//! worldgen, define time; audit F1 as sharpened by corrections #86:
//! dc-worldgen can call dc-api, which is why the placement is not merely
//! right but forced). The result is **returned, not stored** (audit § 4):
//! a sibling of [`BodyPlan`], never a field of it, never registry state,
//! never sim-read (S-3 — the plan stays the authority).
//!
//! **The solve** ([`bake_resting_posture`]): each stance chain assumes the
//! configuration in which gravity exerts zero moment at every joint — the
//! vertical column under its attachment. For chains authored pointing
//! straight down (every shipped plan) that configuration IS the identity
//! rotation, the root lands at chain reach, and balance (CoM over base of
//! support) is **verified, not optimised**. Outputs are scale-free by
//! doctrine (decision 24 / posture-gait § 8: *bake ratios and angles, not
//! metres*); metres appear only as report numbers re-derived from the
//! plan's own lengths.
//!
//! **Slice-one posture axis:** this function bakes the STANDING posture
//! only. The posture axis of the bake key is registry content by
//! namespaced id eventually (`dc:posture/stand` — Q1 answered 2026-08-02,
//! user); slice one carries it as this function's fixed meaning rather
//! than a parameter, the explicitly-marked placeholder.
//!
//! **A-1 ledger** (each simplification, its identity, its heir — audit § 8):
//!
//! - **effort term = zero-torque static geometry**; heir: a real
//!   minimisation with preferred-angle/tendon terms — the bird's folded
//!   rest is in-domain and wrong behind this seam (audit § 2.4). The seam
//!   is the function signature: the output shape (angles per joint) does
//!   not change when the internals do.
//! - **mass = segment volume × density 1**; heir: per-segment materials
//!   (dependency-graph B6) — this CoM is the mass integral's first
//!   posture-side consumer. **B6-a (2026-08-04) moved the seam, not the
//!   number:** the CoM loop that used to sit inline here is now
//!   [`super::mass::mass_properties`] with a density of exactly 1.0 from
//!   [`super::mass::segment_densities`], bit-identically. The stand-in is
//!   unchanged and its heir is now **B6-c** (a declared
//!   `SegmentDef.composition` against a materials roster), which changes
//!   `segment_densities` and nothing else.
//! - **domain = ≥ 2 anchored point contacts, equal reaches, chains
//!   authored as vertical columns**; heirs: distributed/anchored support
//!   rules (gated on their own ratifications), the unequal-chain solve
//!   (audit F4 — the first quadruped's design pass), the real resting
//!   minimisation for non-vertical chains.
//! - **support patch = the bearing segment's bottom face** (Q3, yes);
//!   heir: whatever the collider member derives, if richer. The hull is
//!   internal to the balance check and deliberately NOT an exported field
//!   (A-4: machinery without a consumer).
//! - **stance width = authored hip spacing** (F5): feet sit directly
//!   below hips — the zero-lateral-torque identity, not the definition.
//!
//! First real consumer (the continuation slot, audit § 5): the renderer's
//! `build_plan_assets`, replacing the pinned `trunk.pivot_m[1]`-as-hip
//! read. Slice one's consumers are the tests and their printed report.

use super::{BodyPlan, SegmentDef, segments_with_role};

/// One joint's resting rotation: XYZ Euler, **radians**, same frame as
/// [`super::JointRot::euler`] (applied about the segment's pivot). All
/// zeros for the slice-one vertical-column solve — and for the shipped
/// plans zero IS the answer, not a stub (see `straight_legs_are_the_answer`).
#[derive(Clone, PartialEq, Debug)]
pub struct JointAngle {
    /// Segment (joint) name within the plan.
    pub segment: String,
    /// XYZ Euler rotation, radians.
    pub euler: [f64; 3],
}

/// One stance chain's resting pose, contact-first (the order
/// [`stance_chain`] derives: `[contact segment, .., topmost bone]`).
#[derive(Clone, PartialEq, Debug)]
pub struct ChainPose {
    /// Per-joint resting angles, contact-first.
    pub joints: Vec<JointAngle>,
    /// The chain's reach in metres — bone lengths from the chain's
    /// attachment joint down to the declared contact anchor. A report
    /// number (the plan's own lengths), not a baked authority.
    pub reach_m: f64,
}

/// A baked resting posture: scale-free outputs (angles + a height ratio)
/// plus the report numbers the acceptance demands. Returned, not stored.
#[derive(Clone, PartialEq, Debug)]
pub struct RestingPosture {
    /// Per-stance-chain joint angles, one entry per active bearing contact.
    pub chains: Vec<ChainPose>,
    /// Root height / governing chain reach — dimensionless, invariant
    /// under uniform scaling. Exactly 1.0 for slice one's straight
    /// columns whose hips sit level with the root.
    pub root_height_ratio: f64,
    /// Derived root height in metres (`root_height_ratio` × the governing
    /// chain's reach) — computed for reporting; consumers re-derive
    /// metres from the plan's own lengths at consumption.
    pub root_height_m: f64,
    /// Centre of mass at the resting pose, metres, ground frame (y = 0 at
    /// the ground, x/z body-local): [`super::mass::mass_properties`] over
    /// ALL segments at [`super::mass::segment_densities`], whose identity is
    /// density ≡ 1 (⚠ STAND-IN, `stubs.md` #40; heir: per-segment
    /// composition, B6-c). Re-derived from the integral here, never copied
    /// from it (S-3).
    pub com_m: [f64; 3],
    /// The balance verdict: CoM (x, z) strictly inside the convex hull of
    /// the bearing segments' bottom-face support patches (Q3). The hull
    /// itself is internal and not exported (A-4).
    pub com_in_support: bool,
}

/// The bake's answer: a posture, a legal absence, or a loud refusal.
#[derive(Clone, PartialEq, Debug)]
pub enum BakeOutcome {
    /// The resting posture for the named mode.
    Baked(RestingPosture),
    /// The plan declares zero modes — a tree. A legal absence, not an
    /// error (B0's identity default: a mode-less plan is legal and
    /// declares nothing about support), mirroring
    /// [`super::unique_role_segment`]'s `Ok(None)`.
    NothingDeclared,
    /// Out of the standing solve's domain, with the offending geometry
    /// named — never a silent nonsense posture (user ruling, posture-gait
    /// § 5 scoping banner).
    Unsupported {
        /// Names the mode/role/segments/geometry that took the plan out
        /// of domain, and the heir where one is filed.
        reason: String,
    },
}

/// Walk parents from `from` (a segment of `plan`) to the first branch
/// point (a segment with ≥ 2 children) or the root — the stance-chain
/// derivation, contact-first: `[from, .., topmost bone]`. Neither the
/// branch point nor the root is part of the chain.
///
/// **This is the SHARED derivation** (audit F6: hoist, don't duplicate):
/// dc-client's `leg_rigs` consumes it for the foot-placement IK and the
/// bake consumes it for the resting solve — one truth, two consumers.
/// Pure data + arithmetic; the chain is DERIVED, the contact is DECLARED.
pub fn stance_chain<'a>(plan: &'a BodyPlan, from: &'a SegmentDef) -> Vec<&'a SegmentDef> {
    let seg_by = |name: &str| plan.segments.iter().find(|s| s.name == name);
    let child_count = |name: &str| {
        plan.segments
            .iter()
            .filter(|s| s.parent.as_deref() == Some(name))
            .count()
    };
    let mut chain = vec![from];
    let mut cur = from;
    let mut hops = 0;
    while let Some(parent) = cur.parent.as_deref().and_then(seg_by) {
        if child_count(&parent.name) >= 2 || parent.parent.is_none() {
            break;
        }
        chain.push(parent);
        cur = parent;
        // Guard against an unvalidated plan's parent cycle; on a plan that
        // passed `validate_plan` this bound is never reached.
        hops += 1;
        if hops > plan.segments.len() {
            break;
        }
    }
    chain
}

/// The stance chain of every segment carrying `role`, in plan order —
/// [`stance_chain`] over [`segments_with_role`]. This is exactly the walk
/// dc-client's `leg_rigs` used to carry privately; hoisted here so the
/// bake and the renderer derive chains from one function (audit F6).
pub fn stance_chains<'a>(plan: &'a BodyPlan, role: &str) -> Vec<Vec<&'a SegmentDef>> {
    segments_with_role(plan, role)
        .into_iter()
        .map(|s| stance_chain(plan, s))
        .collect()
}

/// Accumulated pivot offset of `seg`'s joint from the ROOT's joint —
/// summing `pivot_m` up the parent walk and **excluding the root's own
/// `pivot_m`**, which is an authored rest origin, not a hip
/// (body-plan-structure § 5.1; `hip_height_is_an_output` pins this).
///
/// `pub(super)` so the GAIT bake reads the one derivation rather than
/// carrying a second copy (audit F6: hoist, don't duplicate) — the same
/// argument that hoisted [`stance_chain`] out of dc-client.
pub(super) fn offset_from_root(plan: &BodyPlan, seg: &SegmentDef) -> [f64; 3] {
    let mut acc = [0.0_f64; 3];
    let mut cur = seg;
    let mut hops = 0;
    while cur.parent.is_some() {
        for (a, p) in acc.iter_mut().zip(cur.pivot_m) {
            *a += p;
        }
        let Some(p) = cur
            .parent
            .as_deref()
            .and_then(|n| plan.segments.iter().find(|s| s.name == n))
        else {
            break;
        };
        cur = p;
        hops += 1;
        if hops > plan.segments.len() {
            break;
        }
    }
    acc
}

pub(super) fn norm3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Strictly convex hull (Andrew's monotone chain), counter-clockwise.
/// Collinear points are dropped; fewer than 3 distinct non-collinear
/// points yield a degenerate (< 3 vertex) result.
fn convex_hull(mut pts: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    pts.sort_by(|a, b| a.partial_cmp(b).expect("finite support-patch corners"));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut lower: Vec<[f64; 2]> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<[f64; 2]> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// `p` strictly inside a counter-clockwise convex polygon. A degenerate
/// hull (< 3 vertices) contains nothing — honestly false, never a
/// tolerance (audit § 2.3: a tolerance here would be a number with no
/// derivation).
fn strictly_inside(hull: &[[f64; 2]], p: [f64; 2]) -> bool {
    if hull.len() < 3 {
        return false;
    }
    hull.iter()
        .zip(hull.iter().cycle().skip(1))
        .all(|(a, b)| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]) > 0.0)
}

/// An active bearing contact: a role instance with a declared anchor.
struct Contact<'a> {
    role: &'a str,
    seg: &'a SegmentDef,
    anchor: [f64; 3],
    /// Anchor position at the rest pose, ground-frame x/z (root at x=z=0).
    xz: [f64; 2],
}

/// Equal-reach / vertical-column tolerance, metres. Not a tuned constant:
/// it is the "exactly equal, up to accumulated f64 rounding" epsilon the
/// audit's F4 refusal specifies — chains that genuinely differ (any
/// authored asymmetry is millimetres at least) are orders of magnitude
/// beyond it.
pub(super) const EPS_M: f64 = 1e-9;

/// Bake the resting posture of `plan` for the declared locomotor `mode` —
/// pure, deterministic, no world involvement (see the module doc for the
/// venue argument and the A-1 ledger).
///
/// - `plan` declares zero modes → [`BakeOutcome::NothingDeclared`] (a tree).
/// - `mode` not declared BY NAME → [`BakeOutcome::Unsupported`] naming it.
/// - the active bearing set is distributed (unanchored roles, or ≥ 3
///   point contacts on a single line), has fewer than two anchored point
///   contacts, reaches unequally beyond `1e-9` (audit F4), or is not
///   authored as vertical columns → [`BakeOutcome::Unsupported`] naming
///   the geometry and the heir.
/// - otherwise → [`BakeOutcome::Baked`]: identity stance angles, root at
///   chain reach, CoM verified against the bottom-face support hull.
pub fn bake_resting_posture(plan: &BodyPlan, mode: &str) -> BakeOutcome {
    if plan.modes.is_empty() {
        return BakeOutcome::NothingDeclared;
    }
    let Some(mode_def) = plan.modes.iter().find(|m| m.mode == mode) else {
        let declared: Vec<&str> = plan.modes.iter().map(|m| m.mode.as_str()).collect();
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` does not declare mode `{mode}` by name (declared: {})",
                plan.name,
                declared.join(", ")
            ),
        };
    };

    // Resolve the mode's bearing roles to contacts (B0: capability on the
    // part, activation by the mode). Anchored = a point contact;
    // unanchored = the whole segment bears — a distributed support.
    let mut unanchored: Vec<String> = Vec::new();
    let mut contacts: Vec<Contact> = Vec::new();
    for role in &mode_def.bearing {
        for seg in segments_with_role(plan, role) {
            for r in seg.roles.iter().filter(|r| r.role == *role) {
                match r.at_m {
                    None => unanchored.push(format!("`{role}` on `{}`", seg.name)),
                    Some(anchor) => {
                        let base = offset_from_root(plan, seg);
                        contacts.push(Contact {
                            role,
                            seg,
                            anchor,
                            xz: [base[0] + anchor[0], base[2] + anchor[2]],
                        });
                    }
                }
            }
        }
    }
    if !unanchored.is_empty() {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}`: bearing role(s) {} are distributed (unanchored \
                 — the whole segment carries the role), not a standing point-set; the \
                 standing solve covers anchored point contacts only (heir: \
                 distributed-support rules, gated on their own ratifications)",
                plan.name,
                unanchored.join(", ")
            ),
        };
    }
    if contacts.is_empty() {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}` activates zero anchored bearing point contacts \
                 — nothing carries the body in this mode",
                plan.name
            ),
        };
    }
    if contacts.len() < 2 {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}` yields {} anchored point contact; the standing \
                 solve needs at least 2 (slice-one narrowing, audit § 6)",
                plan.name,
                contacts.len()
            ),
        };
    }

    // ≥ 3 point contacts on a single line are a belly-drag wearing anchors,
    // not a standing stance: balance over a line is measure-zero and the
    // vertical-column solve has no answer for it. (Two contacts are always
    // collinear and are saved by the bottom-face patches instead — Q3.)
    if contacts.len() >= 3 {
        let p0 = contacts[0].xz;
        let (mut far, mut best) = (0.0_f64, p0);
        for c in &contacts {
            let d = (c.xz[0] - p0[0]).powi(2) + (c.xz[1] - p0[1]).powi(2);
            if d > far {
                far = d;
                best = c.xz;
            }
        }
        let span = far.sqrt();
        let collinear = span <= EPS_M
            || contacts.iter().all(|c| {
                let cross =
                    (best[0] - p0[0]) * (c.xz[1] - p0[1]) - (best[1] - p0[1]) * (c.xz[0] - p0[0]);
                (cross / span).abs() <= EPS_M
            });
        if collinear {
            let listing: Vec<String> = contacts
                .iter()
                .map(|c| format!("`{}` on `{}`", c.role, c.seg.name))
                .collect();
            return BakeOutcome::Unsupported {
                reason: format!(
                    "plan `{}` mode `{mode}`: the {} bearing point contacts ({}) lie on \
                     a single line — a distributed line bearing, not a standing support \
                     polygon; balance over a line is measure-zero (heir: \
                     distributed-support rules, gated on their own ratifications)",
                    plan.name,
                    contacts.len(),
                    listing.join(", ")
                ),
            };
        }
    }

    // Derive each contact's stance chain, its reach, and the root height
    // it implies. The vertical-column solve is exact only when every
    // active chain reaches equally (audit F4) and is authored pointing
    // straight down from its attachment (audit § 2.2's stated premise,
    // checked rather than assumed).
    let mut chains: Vec<ChainPose> = Vec::new();
    let mut heights: Vec<f64> = Vec::new();
    let mut reaches: Vec<f64> = Vec::new();
    for c in &contacts {
        let chain = stance_chain(plan, c.seg);
        let top = *chain.last().expect("chain contains the contact segment");
        let mut reach = norm3(c.anchor);
        for s in &chain[..chain.len() - 1] {
            reach += norm3(s.pivot_m);
        }
        if !(reach.is_finite() && reach > EPS_M) {
            return BakeOutcome::Unsupported {
                reason: format!(
                    "plan `{}` mode `{mode}`: the chain from `{}` has no reach ({reach} m)",
                    plan.name, c.seg.name
                ),
            };
        }
        // Straight-down check: the anchor must sit directly under the
        // chain's attachment joint, a full reach below it — else the
        // identity rotation is NOT the zero-torque column and this solve
        // would silently pose it wrong.
        let hip = offset_from_root(plan, top);
        let base = offset_from_root(plan, c.seg);
        let disp = [
            base[0] + c.anchor[0] - hip[0],
            base[1] + c.anchor[1] - hip[1],
            base[2] + c.anchor[2] - hip[2],
        ];
        if disp[0].abs() > EPS_M || disp[2].abs() > EPS_M || (-disp[1] - reach).abs() > EPS_M {
            return BakeOutcome::Unsupported {
                reason: format!(
                    "plan `{}` mode `{mode}`: the chain from `{}` is not authored as a \
                     vertical column under its attachment (anchor sits at [{:.3}, {:.3}, \
                     {:.3}] m from the hip against a {reach:.3} m reach); the identity \
                     rotation is the zero-torque answer only for straight-down chains \
                     (heir: the real resting minimisation, audit § 2.4)",
                    plan.name, c.seg.name, disp[0], disp[1], disp[2]
                ),
            };
        }
        heights.push(reach - hip[1]);
        reaches.push(reach);
        chains.push(ChainPose {
            joints: chain
                .iter()
                .map(|s| JointAngle {
                    segment: s.name.clone(),
                    euler: [0.0; 3],
                })
                .collect(),
            reach_m: reach,
        });
    }
    if reaches.iter().any(|r| (r - reaches[0]).abs() > EPS_M) {
        let listing: Vec<String> = contacts
            .iter()
            .zip(&reaches)
            .map(|(c, r)| format!("`{}` {r:.3} m", c.seg.name))
            .collect();
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}`: stance chains reach unequally ({}); the \
                 vertical-column solve is exact only for equal reaches — refused rather \
                 than silently pretended to cover (audit F4; heir: the first \
                 quadruped's design pass)",
                plan.name,
                listing.join(", ")
            ),
        };
    }
    if heights.iter().any(|h| (h - heights[0]).abs() > EPS_M) {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}`: equal-reach stance chains attach at unequal \
                 heights, so no single root height stands every contact on the ground \
                 (audit F4's sibling; heir: the first quadruped's design pass)",
                plan.name
            ),
        };
    }

    // The scale-free outputs, then the report numbers. The governing
    // chain's reach is chains[0]'s — all reaches are equal here, and
    // whether that denominator survives the first unequal-chain quadruped
    // is recorded as unanswerable in audit § 10.
    let governing_reach = reaches[0];
    let root_height_ratio = heights[0] / governing_reach;
    let root_height_m = root_height_ratio * governing_reach;

    // CoM at the resting pose (identity angles = the authored rest pose,
    // root at the derived height): mass-weighted box centres over ALL
    // segments — the stout's knuckle-dragging arms participate in the CoM
    // and not in support, which is the mode axis doing its job.
    //
    // **This loop used to live here** (B6-a, 2026-08-04). It is now the mass
    // integral's first consumer: `mass_properties` is the one authority and
    // this is a re-derivation of it, never a copy that can drift (S-3 — the
    // failure `root_bob_m` cost us twice, corrections #80/#93). At the
    // identity density ρ ≡ 1.0 the arithmetic is unchanged (`v * 1.0 == v`)
    // and the result is bit-identical to the volume proxy it replaces; the
    // proxy has stopped BEING the authority and started DERIVING from one,
    // which is the whole point of the slice (`stubs.md` #40).
    let densities = super::mass::segment_densities(plan);
    let mp = super::mass::mass_properties(plan, &densities, root_height_m);
    if !(mp.mass_kg.is_finite() && mp.mass_kg > 0.0) {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` has no mass to balance (volume {} m³, mass {})",
                plan.name, mp.volume_m3, mp.mass_kg
            ),
        };
    }
    let com_m = mp.com_m;

    // Base of support: the bottom-face patch of each bearing segment (Q3
    // — the box's x/z extents around the declared anchor, the same
    // geometry `with_sole` derives the anchor from), hulled. Strict
    // point anchors would make fore-aft balance measure-zero for a biped
    // (audit F2); feet have extent.
    let mut corners: Vec<[f64; 2]> = Vec::with_capacity(contacts.len() * 4);
    for c in &contacts {
        let (hx, hz) = (c.seg.size_m[0] / 2.0, c.seg.size_m[2] / 2.0);
        corners.push([c.xz[0] - hx, c.xz[1] - hz]);
        corners.push([c.xz[0] - hx, c.xz[1] + hz]);
        corners.push([c.xz[0] + hx, c.xz[1] - hz]);
        corners.push([c.xz[0] + hx, c.xz[1] + hz]);
    }
    if corners.iter().flatten().any(|x| !x.is_finite()) {
        return BakeOutcome::Unsupported {
            reason: format!(
                "plan `{}` mode `{mode}`: non-finite support-patch geometry",
                plan.name
            ),
        };
    }
    let hull = convex_hull(corners);
    let com_in_support = strictly_inside(&hull, [com_m[0], com_m[2]]);

    // B7 — the bake checks its OWN output before it publishes it: every joint
    // angle in every chain must be inside that joint's limits, else a loud
    // refusal naming the joint, the angle, the range, and which end was
    // declared versus derived (joint-limits audit § 5.2).
    //
    // **Vacuous by construction on the shipped plans, and that is correct**:
    // the resting angles are all exactly 0.0 and `min <= 0 <= max` is a
    // `validate_plan` invariant. It is a guard positioned BEFORE its trigger,
    // and it goes live at the two places already sequenced — member #0's
    // effort seam (a bird's tendon-held folded rest) and member #1's derived
    // gait poses, which is why B7 is upstream of the gait bake.
    let limits = super::limits::derive_joint_limits(plan);
    for chain in &chains {
        for ja in &chain.joints {
            if let Err(why) = limits.check(&ja.segment, ja.euler) {
                return BakeOutcome::Unsupported {
                    reason: format!(
                        "plan `{}` mode `{mode}`: the resting solve puts a joint outside its \
                         declared range — {why}",
                        plan.name
                    ),
                };
            }
        }
    }

    BakeOutcome::Baked(RestingPosture {
        chains,
        root_height_ratio,
        root_height_m,
        com_m,
        com_in_support,
    })
}

// The audit § 7 acceptance set — split by concern (file-size convention,
// 2026-08-01: source splits on ordinary module boundaries).
#[cfg(test)]
mod tests;
