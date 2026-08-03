//! The gait bake's GEOMETRY half, split out of `gait.rs` by concern. Everything
//! here reads declared geometry and returns a derived
//! quantity: which contacts form a girdle, what phase each takes, which chain
//! governs root height over which slice of the cycle, the pose gradient toward
//! touchdown, the mid-swing clearance solve, and the counter-swing limbs.
//!
//! **No name is ever read as a fact.** Laterality, fore/hind and pairing are
//! derived from geometry (B0's ratified rule), so a biped, a quadruped and a
//! millipede are the same code path and an evolved body that renamed every
//! segment still bakes.

use super::super::bake::{EPS_M, norm3, offset_from_root};
use super::super::{BodyPlan, JointAngle, SegmentDef, stance_chain};
use super::{GaitKnobs, LimbGait};

/// An active bearing contact, resolved in member #0's iteration order.
pub(super) struct Contact<'a> {
    pub(super) role: String,
    pub(super) seg: &'a SegmentDef,
    pub(super) anchor: [f64; 3],
    /// Anchor position at the rest pose, ground frame (root at x = z = 0).
    pub(super) ground: [f64; 3],
}

pub(super) fn knobs_are_finite(k: &GaitKnobs) -> bool {
    [
        k.cadence_scale,
        k.duty_exponent,
        k.bob_damping,
        k.swing_flexion,
        k.foot_clearance_ratio,
        k.transition_fr,
        k.arm_swing_amplitude,
    ]
    .iter()
    .all(|v| v.is_finite())
}

/// Report — never refuse — a kind-3 stand-in outside its published band.
pub(super) fn band_notes(k: &GaitKnobs) -> Vec<String> {
    let mut out = Vec::new();
    let mut check = |v: f64, lo: f64, hi: f64, name: &str, what: &str| {
        if !(lo..=hi).contains(&v) {
            out.push(format!(
                "`{name}` = {v} is outside the published band [{lo}, {hi}] — proceeding, \
                 because a clockwork golem may want to step wrong. It stands in for {what} \
                 (heir: B6, per-segment materials)"
            ));
        }
    };
    check(
        k.cadence_scale,
        0.8,
        1.25,
        "cadence_scale",
        "muscle power against limb inertia",
    );
    check(
        k.bob_damping,
        0.5,
        1.0,
        "bob_damping",
        "stance-knee flexion, pelvic list and ankle rocker (Saunders 1953)",
    );
    check(
        k.swing_flexion,
        0.0,
        1.0,
        "swing_flexion",
        "swing-leg energetics",
    );
    out
}

/// § 3.3: partition contacts into girdles by fore-aft anchor **z**, order
/// within a girdle by (x, z, y, plan order), and lay the gait type's phase
/// pattern over `(girdle, index)`. With no gait type authored the default is
/// `i/n` within a girdle and `1/(2·girdles)` between girdles — which for two
/// girdles of two is the lateral-sequence walk `{0, ¼, ½, ¾}`, derived rather
/// than special-cased.
pub(super) fn phase_offsets(
    plan: &BodyPlan,
    contacts: &[Contact],
    notes: &mut Vec<String>,
) -> Result<Vec<f64>, String> {
    // Girdles: cluster on z at the same "exactly equal up to f64 rounding"
    // epsilon member #0 uses. A mirrored pair shares z EXACTLY; a real fore/hind
    // pair differs by tens of centimetres. A body with millimetre fore-aft
    // asymmetry inside one pair would split into two girdles — recorded as the
    // known edge, heir: the first quadruped's design pass (member #0's F4).
    let mut zs: Vec<f64> = Vec::new();
    for c in contacts {
        if !zs.iter().any(|z| (z - c.ground[2]).abs() <= EPS_M) {
            zs.push(c.ground[2]);
        }
    }
    zs.sort_by(|a, b| a.partial_cmp(b).expect("finite anchor geometry"));
    let girdle_of = |c: &Contact| {
        zs.iter()
            .position(|z| (z - c.ground[2]).abs() <= EPS_M)
            .expect("every contact's z was collected")
    };
    if zs.len() > 1 {
        notes.push(format!(
            "plan `{}`: {} girdles derived from fore-aft contact geometry ({} contacts); \
             the inter-girdle offset is 1/(2·girdles) — the default pattern, not an \
             authored gait type",
            plan.name,
            zs.len(),
            contacts.len()
        ));
    }

    let mut phases = vec![0.0_f64; contacts.len()];
    for g in 0..zs.len() {
        let mut members: Vec<usize> = (0..contacts.len())
            .filter(|i| girdle_of(&contacts[*i]) == g)
            .collect();
        members.sort_by(|a, b| {
            let (ca, cb) = (&contacts[*a], &contacts[*b]);
            ca.ground[0]
                .partial_cmp(&cb.ground[0])
                .expect("finite anchor geometry")
                .then(
                    ca.ground[2]
                        .partial_cmp(&cb.ground[2])
                        .expect("finite anchor geometry"),
                )
                .then(
                    ca.ground[1]
                        .partial_cmp(&cb.ground[1])
                        .expect("finite anchor geometry"),
                )
                .then(a.cmp(b))
        });
        if members.is_empty() {
            return Err(format!("plan `{}`: an empty girdle was derived", plan.name));
        }
        let k = members.len() as f64;
        let base = g as f64 / (2.0 * zs.len() as f64);
        for (j, i) in members.iter().enumerate() {
            phases[*i] = (base + j as f64 / k).rem_euclid(1.0);
        }
    }
    Ok(phases)
}

/// The fraction of the cycle each contact governs root height for: the gap to
/// the next phase, wrapping. Sums to exactly one cycle by construction.
pub(super) fn compass_windows(phases: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..phases.len()).collect();
    order.sort_by(|a, b| phases[*a].partial_cmp(&phases[*b]).expect("finite phases"));
    let mut out = vec![0.0_f64; phases.len()];
    for (k, i) in order.iter().enumerate() {
        let next = order[(k + 1) % order.len()];
        let gap = if order.len() == 1 {
            1.0
        } else {
            (phases[next] - phases[*i]).rem_euclid(1.0)
        };
        out[*i] = gap;
    }
    out
}

/// The pose gradient toward touchdown: the chain rotates rigidly about its
/// attachment joint, so one radian of hip excursion is one radian on the
/// topmost bone about X and zero everywhere else. Exact for every chain member
/// #0 accepts (a vertical column under its attachment); the linear form is what
/// keeps `contact` a coefficient rather than a frame frozen at one speed.
pub(super) fn sagittal_gradient(chain: &[&SegmentDef]) -> Vec<JointAngle> {
    let top = chain.len() - 1;
    chain
        .iter()
        .enumerate()
        .map(|(i, s)| JointAngle {
            segment: s.name.clone(),
            euler: if i == top { [1.0, 0.0, 0.0] } else { [0.0; 3] },
        })
        .collect()
}

/// Mid-swing: the two-bone analytic solve that puts the contact anchor directly
/// under the attachment joint, lifted by the clearance ratio.
///
/// The knee bends **forward** (thigh positive about X, shank negative relative
/// to it) — the anatomical direction, and the one the authored clip uses.
///
/// ⚠ **SEAM — B7.** When joint rotation limits land, this is where an emitted
/// angle is checked against them and a violation refuses loudly. Today the only
/// bound is the chain's own reach annulus.
pub(super) fn clearance_pose(
    chain: &[&SegmentDef],
    anchor: [f64; 3],
    reach: f64,
    knobs: &GaitKnobs,
) -> Result<Vec<JointAngle>, String> {
    if chain.len() != 2 {
        return Err(format!(
            "the chain from `{}` has {} bones; the mid-swing clearance solve is a two-bone \
             analytic IK and covers exactly two (heir: the general redundant-chain solve, \
             which wants B7's joint limits to pick among its solutions)",
            chain[0].name,
            chain.len()
        ));
    }
    let l1 = norm3(chain[0].pivot_m); // attachment joint -> the middle joint
    let l2 = norm3(anchor); // the middle joint -> the contact anchor
    let inner = (l1 - l2).abs();
    let lift_floor = knobs.foot_clearance_ratio * reach;
    let lift_ceiling = reach - inner;
    // NaN-safe early-out: `!(a > b)` deliberately routes the incomparable case
    // (a degenerate measurement) into the error arm rather than proceeding.
    // Allowed, not restyled, so the author's semantics stay bit-for-bit —
    // `a <= b` would silently PROCEED on NaN. (Lint arrived red with the gait
    // merge and blocked an unrelated gate, 2026-08-03; owner may restyle to
    // `partial_cmp` if preferred.)
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(lift_ceiling > lift_floor + EPS_M) {
        return Err(format!(
            "the chain from `{}` cannot lift its anchor clear of the ground under its own \
             attachment (clearance {lift_floor:.4} m, inner reach limit leaves \
             {lift_ceiling:.4} m) — the two bones are too unequal to flex",
            chain[0].name
        ));
    }
    let lift = lift_floor + knobs.swing_flexion.clamp(0.0, 1.0) * (lift_ceiling - lift_floor);
    let d = reach - lift;
    let hip = (((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0)).acos();
    let knee_interior = (((l1 * l1 + l2 * l2 - d * d) / (2.0 * l1 * l2)).clamp(-1.0, 1.0)).acos();
    let knee = -(std::f64::consts::PI - knee_interior);
    Ok(vec![
        JointAngle {
            segment: chain[0].name.clone(),
            euler: [knee, 0.0, 0.0],
        },
        JointAngle {
            segment: chain[1].name.clone(),
            euler: [hip, 0.0, 0.0],
        },
    ])
}

/// **G3, resolved here rather than in member #0.** Every leaf chain that no
/// bearing contact claims, is laterally displaced from the midline, and hangs
/// as a vertical column under its attachment, counter-swings with the
/// **contralateral** bearing chain (§ 3.3 rule 4). Its mid-swing pose is its
/// rest — an arm hangs straight as it passes the hip.
///
/// A midline appendage (a head, a tail) gets nothing: its carriage is a taste
/// knob with identity 0, not a derived counter-swing. Laterality is read from
/// the sign of the attachment's x, never from a name (B0).
pub(super) fn counter_swing_limbs(
    plan: &BodyPlan,
    contacts: &[Contact],
    bearing: &[LimbGait],
    knobs: &GaitKnobs,
    notes: &mut Vec<String>,
) -> Vec<LimbGait> {
    let mut out: Vec<LimbGait> = Vec::new();
    let claimed: Vec<&str> = contacts
        .iter()
        .flat_map(|c| stance_chain(plan, c.seg))
        .map(|s| s.name.as_str())
        .collect();
    let bearing_x: Vec<f64> = contacts
        .iter()
        .map(|c| {
            let chain = stance_chain(plan, c.seg);
            offset_from_root(plan, chain[chain.len() - 1])[0]
        })
        .collect();

    for leaf in plan.segments.iter().filter(|s| {
        !plan
            .segments
            .iter()
            .any(|o| o.parent.as_deref() == Some(s.name.as_str()))
    }) {
        if claimed.contains(&leaf.name.as_str()) {
            continue;
        }
        let chain = stance_chain(plan, leaf);
        if chain.iter().any(|s| claimed.contains(&s.name.as_str())) {
            continue;
        }
        let attach = offset_from_root(plan, chain[chain.len() - 1]);
        if attach[0].abs() <= EPS_M {
            continue; // a midline appendage: taste, not a derived counter-swing
        }
        // The distal point: the bottom face centre of the leaf's own box — the
        // same geometry `with_sole` derives a declared anchor from, applied to
        // a chain that declares none.
        let distal = [
            leaf.offset_m[0],
            leaf.offset_m[1] - leaf.size_m[1] / 2.0,
            leaf.offset_m[2],
        ];
        let mut reach = norm3(distal);
        for s in &chain[..chain.len() - 1] {
            reach += norm3(s.pivot_m);
        }
        let base = offset_from_root(plan, leaf);
        let disp = [
            base[0] + distal[0] - attach[0],
            base[1] + distal[1] - attach[1],
            base[2] + distal[2] - attach[2],
        ];
        if disp[0].abs() > EPS_M || disp[2].abs() > EPS_M || (-disp[1] - reach).abs() > EPS_M {
            notes.push(format!(
                "chain from `{}` is not a vertical column under its attachment, so its \
                 zero-torque rest is not the identity and no counter-swing was derived for \
                 it (heir: the real resting minimisation — member #0's effort seam)",
                leaf.name
            ));
            continue;
        }
        // Contralateral: the bearing chain whose attachment sits on the other
        // side of the midline, nearest in fore-aft.
        let mate = bearing_x
            .iter()
            .enumerate()
            .filter(|(_, x)| x.signum() != attach[0].signum() && x.abs() > EPS_M)
            .min_by(|(a, _), (b, _)| {
                let da = (contacts[*a].ground[2] - base[2]).abs();
                let db = (contacts[*b].ground[2] - base[2]).abs();
                da.partial_cmp(&db).expect("finite anchor geometry")
            })
            .map(|(i, _)| i);
        let Some(mate) = mate else {
            notes.push(format!(
                "chain from `{}` has no contralateral bearing chain to counter-swing \
                 against; left un-driven rather than given an invented phase",
                leaf.name
            ));
            continue;
        };
        let neutral: Vec<JointAngle> = chain
            .iter()
            .map(|s| JointAngle {
                segment: s.name.clone(),
                euler: [0.0; 3],
            })
            .collect();
        out.push(LimbGait {
            contact_segment: leaf.name.clone(),
            role: String::new(),
            bearing: false,
            phase: bearing[mate].phase,
            amplitude: knobs.arm_swing_amplitude,
            duty_bias: 0.0,
            reach_m: reach,
            compass_window: 0.0,
            clearance: neutral.clone(),
            contact_per_rad: sagittal_gradient(&chain),
            neutral,
        });
    }
    out
}
