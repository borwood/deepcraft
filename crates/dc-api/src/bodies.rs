//! Body plans and animation clips as registry data (docs/design/bodies.md,
//! implementation staircase steps 1–2). The fourth instance of the
//! roles-as-contracts backbone (geology classes, form archetypes, geology
//! passes, now body plans) — but shaped for skeletons, not scalar contracts,
//! so it is its own small registry rather than a reuse of [`crate::classes`].
//!
//! **Two define verbs**, both ordinary [`crate::schema`] CommandSpecs (so every
//! MCP surface grows the tools automatically):
//!
//! - `dc:registry/define_anim_clip` — a named, per-plan animation clip:
//!   keyframed joint rotations plus an optional root bob, and a loop flag.
//!   Clips are standalone data; they name joints but do not name a plan, so a
//!   clip can be defined before any plan binds it (mechanism: this dodges the
//!   plan⇄clip chicken-and-egg — see below).
//! - `dc:registry/define_body_plan` — a named joint-tree of cuboid segments
//!   plus its **declared structure** (roles, modes) and its **action → clip
//!   bindings**. Validated at define time against the already-registered
//!   clips on the *you-supplied-what-you-claimed* rule (B0, ratified
//!   2026-08-01): every declared action binds a registered, joint-compatible
//!   clip, and every mode's bearing roles exist on some segment. **The action
//!   vocabulary is OPEN** — there is no known-verbs list and no mandatory
//!   verb; a bird declares `fly`, a tree declares no locomotion at all, and
//!   neither is the engine's business to refuse (north-star Deviation 2; the
//!   closed `KNOWN_VERBS`/`REQUIRED_VERBS` contract was retired here).
//!
//! **The define order** (clips, then the plan that binds them) is the honest
//! resolution of the contract's direction: the plan asserts "these actions are
//! covered", which can only be checked once the clips exist. Clips reference a
//! plan only implicitly, through the joint names they animate — validated when
//! a plan adopts them, exactly like a geology member validating against its
//! class contract at join time.
//!
//! **Roles and modes (B0, the declaration slice — posture-gait.md § 7b,
//! § 5).** A segment declares what its parts ARE (`roles`: a sole, the look
//! joint, a stinger — an open vocabulary, optionally anchored to a
//! segment-local point); a plan declares its locomotor **modes** and which
//! roles *bear the body* in each. Support is a per-segment capability
//! activated per mode — there is no body-level "support kind". What geometry
//! already determines (laterality, fore/hind, pairing) is DERIVED, never
//! declared. Consumers that need one segment for a role resolve
//! **unique-or-loud** ([`unique_role_segment`]); clip binding stays
//! address-exact for now (stubs.md § 34 — role binding for animation is the
//! gait bake's, before the firewall moves).
//!
//! **The firewall (bodies.md § determinism firewall) is LAW here.** These are
//! pure data — segment geometry and keyframes — with no rendering dependency,
//! so they live in dc-api like any other def. The *sampler and renderer* that
//! turn a clip into a posed skeleton are client-only (dc-client `body.rs`);
//! nothing in this module, and nothing the sim reads, ever touches a sampled
//! pose. The sim sees the swept-AABB mover (`crate::character`) and parametric
//! posture only; animation is cosmetic.
//!
//! **The default biped** ([`biped_plan`], [`biped_clips`]) is the in-repo
//! authored source — the default content, compiled in so it is deterministic
//! without a data load. [`default_body_pack`] emits that same content as a
//! recorded command batch (the default pack is the first pack), generated *from*
//! the authored source so pack and typed model cannot drift. **The client
//! renderer reads the REGISTRY**, not the authored source: the pack is submitted
//! through the one door at world construction (dc-client `authority.rs`), so the
//! default biped is genuinely loaded as the first pack rather than compiled into
//! the renderer (journal: "the second plan and the glue"). The authored functions
//! remain the *source* the pack is generated from, and the compiled-in default
//! the headless crates test against.
//!
//! **The experiment plans** ([`stout_plan`], [`longleg_plan`]) exist to test
//! claims, not to look good, and they live in [`experiment_body_pack`] — never in
//! the default pack.
//!
//! - `dc:body/stout` answers bodies.md § IK's *"one clip serves every mutation of
//!   a plan … across differing proportions"*: until there were two plans that
//!   claim had never met evidence. It shares the biped's eleven joint names and
//!   binds the biped's *unmodified* clips; only the geometry differs, and it
//!   differs deliberately hard (half-length legs, 1.6× arms, a wide trunk, a big
//!   head). Its ugliness is the measurement, not a defect.
//! - `dc:body/longleg` answers the question journal/0130 opened and could not
//!   close: **the foot-placement IK had never once engaged, on any plan**, because
//!   every plan's hip sits *higher* than its legs reach, so a sole on the ground
//!   is outside the solver's annulus before animation runs. `longleg` is the
//!   biped with **only the two leg bone lengths changed** — enough that ground
//!   contact is reachable and the knee has to bend. One variable, one control.

use serde::{Deserialize, Serialize};

use crate::envelope::{ConsumerId, Tick};

/// The body plan a character wears when nothing names one — the **identity
/// default** (spines.md § S-5): a seam whose default value reproduces the
/// pre-seam behaviour exactly, so adding per-character plan selection changes
/// nothing unbidden. Every existing character, and every character spawned
/// without a `body_plan`, wears this.
pub const DEFAULT_BODY_PLAN: &str = "dc:body/biped";

// `KNOWN_VERBS` and `REQUIRED_VERBS` were DELETED 2026-08-01 (B0, user
// ratification — ROADMAP close block: "the action vocabulary OPENS"). The
// closed list rejected a bird (`fly`) and a tree (no locomotion) at define
// time with no machine justification — the engine never reasoned about verbs;
// they were clip-slot keys — and it collided with north-star Deviation 2
// ("assume mods can do anything"). The check that replaced them is
// *you-supplied-what-you-claimed* ([`validate_plan`]).

/// One role a segment declares: what this part **is**, in an OPEN vocabulary —
/// `sole`, `look`, `face`, `mouth`, `stinger`, `belly`, `wing`… The engine
/// attaches no meaning to the string; *consumers* do (the posture bake reads
/// the roles a mode names as bearing; the client reads `look`). Roles are
/// **many-to-one over segments** by design — a fern's 300 leaflets all carry
/// `foliage` — and a consumer that needs exactly one resolves it
/// **unique-or-loud** via [`unique_role_segment`].
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RoleDef {
    /// Open role name, non-empty. Shared across bodies — this is what systems
    /// (and eventually animations — stubs.md § 34) bind to.
    pub role: String,
    /// Optional anchor for the role, in **segment-local** coordinates relative
    /// to this segment's pivot — a sole is the bottom FACE of the lower-leg
    /// box, not its centroid. `None` = the whole segment carries the role.
    #[serde(default)]
    pub at_m: Option<[f64; 3]>,
}

/// One locomotor mode a plan declares — `stand`, `high_walk`, `sprawl`,
/// `swim`… — and which ROLES bear the body's weight in it. Support is a
/// per-segment **capability** (a role), activated **per mode** (posture-gait.md
/// § 5, the alligator amendment): an alligator's belly bears in the sprawl,
/// not in the high walk, and touches nothing swimming. The posture bake keys
/// on mode; the mode names its bearing set.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ModeDef {
    /// Open mode name, non-empty, unique within the plan.
    pub mode: String,
    /// Roles that carry the body in this mode. Each must exist on at least one
    /// segment (checked at define time).
    pub bearing: Vec<String>,
}

/// One cuboid segment of a body plan's joint tree.
///
/// A segment is a joint (a pivot) with a cuboid hung off it. `pivot_m` is the
/// joint's offset from its **parent joint's** pivot; the segment rotates about
/// its own pivot during animation. `offset_m` places the cuboid's center
/// relative to that pivot, so a limb pivots at the shoulder while its box hangs
/// below. All in meters — bodies are fixed real-world size (the voxel scale
/// changes the world's resolution, not the people; dc-client character.rs).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SegmentDef {
    /// Unique within the plan; the anim-clip joint key and the socket-binding
    /// key (sockets are step 3+). **Read as an ADDRESS** (B0): a flat name
    /// (`trunk`, `leg_l_upper`) is the degenerate case of a path
    /// (`trunk/branch[2]/leaflet[3]`) — the grammar is reserved for generated
    /// bodies; nothing parses paths yet, and every consumer treats the string
    /// as an opaque equality key, which is why this is a zero-break reading.
    pub name: String,
    /// Parent segment name; `None` marks the single root.
    pub parent: Option<String>,
    /// Joint pivot offset from the parent joint's pivot (root: from the feet).
    pub pivot_m: [f64; 3],
    /// Cuboid dimensions (x, y, z).
    pub size_m: [f64; 3],
    /// Cuboid center relative to this segment's pivot.
    pub offset_m: [f64; 3],
    /// Flat RGB tint, v0 (textures are a later slice; assets/ is out of scope).
    pub tint: [f32; 3],
    /// What this part IS (B0). Empty = the segment declares nothing, which is
    /// the identity default: a role-less plan behaves exactly as before.
    #[serde(default)]
    pub roles: Vec<RoleDef>,
    // DELIBERATELY ABSENT, heirs named (B0 ratification, 2026-08-01):
    // `collide` — per-segment collider participation is B4's design question
    // (a folded wing vs a spread one suggests it may be MODE-scoped, and a
    // static bool here would pre-answer that); `kind` (Box/Card) waits on the
    // vegetation ratification; `composition` waits on the materials roster
    // (dependency-graph B6).
}

/// An action → clip binding. The set of `action`s present is the set of
/// actions the plan claims to support — an **OPEN vocabulary** (no known-verb
/// list, no mandatory entries; B0). The engine's check is that every claim is
/// backed: the clip exists and animates only this plan's joints. What the
/// driver does with an action a plan lacks is the driver's affair — the
/// client's locomotion driver asks for `idle`/`walk` by name and reports a
/// plan that lacks them rather than inventing a pose.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionDef {
    /// Open action name, non-empty, unique within the plan.
    pub action: String,
    /// Namespaced clip name; must be registered and joint-compatible.
    pub clip: String,
}

/// A body plan: a joint-tree of cuboid segments plus its declared structure
/// (roles, modes) and its action→clip bindings.
/// The payload of `dc:registry/define_body_plan`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BodyPlan {
    /// Namespaced name, e.g. `dc:body/biped`.
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub segments: Vec<SegmentDef>,
    /// Locomotor modes and their bearing roles (B0). Empty is the identity
    /// default — a mode-less plan is legal and declares nothing about support.
    #[serde(default)]
    pub modes: Vec<ModeDef>,
    pub actions: Vec<ActionDef>,
}

/// One joint's rotation at a keyframe: XYZ Euler angles in radians.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JointRot {
    /// Segment name this rotation targets (must exist in the plan that binds
    /// the clip — checked at plan-define time).
    ///
    /// ⚠ STAND-IN — stubs.md § 34 (`the-binding-key-that-is-a-name-wearing-a-role`).
    /// Clips bind by segment ADDRESS; `posture-gait.md` § 7b ratifies that
    /// animations bind to ROLES. The migration is deliberately deferred to the
    /// gait-bake design pass (ruled 2026-08-01) and MUST land before the
    /// firewall moves and clips become versioned sim assets. Cross-plan clip
    /// reuse works today because plans share names — measured, journal/0130 —
    /// which is a poor man's role: it fails silently when strings drift.
    pub segment: String,
    /// XYZ Euler rotation, radians, applied about the segment's pivot.
    pub euler: [f64; 3],
}

/// One keyframe: a time plus the pose (per-joint rotations + root bob) at that
/// time. Joints omitted from a keyframe are identity there.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Keyframe {
    /// Seconds from the clip start; strictly ascending across a clip, within
    /// `[0, duration_s]`.
    pub t: f64,
    /// Vertical offset of the whole body root at this time (the walk bob).
    #[serde(default)]
    pub root_bob_m: f64,
    pub rotations: Vec<JointRot>,
}

/// An animation clip: keyframed joint rotations over a duration, looping or
/// one-shot. The payload of `dc:registry/define_anim_clip`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnimClip {
    /// Namespaced name, e.g. `dc:anim/biped_walk`.
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub duration_s: f64,
    pub loops: bool,
    pub keyframes: Vec<Keyframe>,
}

/// A registered body plan, as the host stores it (validated; provenance).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BodyPlanDef {
    pub plan: BodyPlan,
    pub defined_tick: Tick,
    pub defined_by: ConsumerId,
}

/// A registered clip, as the host stores it (validated; provenance).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnimClipDef {
    pub clip: AnimClip,
    pub defined_tick: Tick,
    pub defined_by: ConsumerId,
}

fn all_finite(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite())
}

/// Validate a clip on its own (at `define_anim_clip` time): finite positive
/// duration, at least one keyframe, strictly-ascending in-range keyframe times,
/// finite angles/bob, no duplicate joint within a keyframe. Joint-vs-plan
/// compatibility is NOT checked here — a clip is standalone until a plan binds
/// it ([`validate_plan`]).
pub fn validate_clip(clip: &AnimClip) -> Result<(), String> {
    if !(clip.duration_s.is_finite() && clip.duration_s > 0.0) {
        return Err(format!(
            "clip `{}` needs a finite positive duration",
            clip.name
        ));
    }
    if clip.keyframes.is_empty() {
        return Err(format!("clip `{}` has no keyframes", clip.name));
    }
    let mut prev_t = f64::NEG_INFINITY;
    for (i, kf) in clip.keyframes.iter().enumerate() {
        if !kf.t.is_finite() || kf.t < 0.0 || kf.t > clip.duration_s {
            return Err(format!(
                "clip `{}` keyframe {i} time {} is outside [0, {}]",
                clip.name, kf.t, clip.duration_s
            ));
        }
        if kf.t <= prev_t {
            return Err(format!(
                "clip `{}` keyframe times must strictly ascend (at index {i})",
                clip.name
            ));
        }
        prev_t = kf.t;
        if !kf.root_bob_m.is_finite() {
            return Err(format!(
                "clip `{}` keyframe {i} has a non-finite root bob",
                clip.name
            ));
        }
        for (j, r) in kf.rotations.iter().enumerate() {
            if r.segment.is_empty() {
                return Err(format!(
                    "clip `{}` keyframe {i} has an empty joint name",
                    clip.name
                ));
            }
            if kf.rotations[..j].iter().any(|q| q.segment == r.segment) {
                return Err(format!(
                    "clip `{}` keyframe {i} names joint `{}` twice",
                    clip.name, r.segment
                ));
            }
            if !all_finite(&r.euler) {
                return Err(format!(
                    "clip `{}` keyframe {i} joint `{}` has non-finite angles",
                    clip.name, r.segment
                ));
            }
        }
    }
    Ok(())
}

/// Every segment carrying `role`, in plan order. Roles are many-to-one by
/// design (300 `foliage` leaflets); consumers that want the set take it whole.
pub fn segments_with_role<'a>(plan: &'a BodyPlan, role: &str) -> Vec<&'a SegmentDef> {
    plan.segments
        .iter()
        .filter(|s| s.roles.iter().any(|r| r.role == role))
        .collect()
}

/// Resolve `role` to **exactly one** segment, or say loudly why not — the
/// query-time half of unique-or-loud (B0). Zero matches is a legal absence
/// (`Ok(None)`: a blob has no look joint, and today's behaviour for a missing
/// name was the same feature-off); **more than one is an error naming the
/// contenders**, never a silent first-match — the silent miss is exactly the
/// failure mode roles exist to kill. Define-time cardinality *declarations*
/// are the gait-bake heir's to design (stubs.md § 34).
pub fn unique_role_segment<'a>(
    plan: &'a BodyPlan,
    role: &str,
) -> Result<Option<&'a SegmentDef>, String> {
    let found = segments_with_role(plan, role);
    match found.len() {
        0 => Ok(None),
        1 => Ok(Some(found[0])),
        _ => Err(format!(
            "plan `{}` declares role `{role}` on {} segments ({}); this consumer needs exactly one",
            plan.name,
            found.len(),
            found
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Validate a plan (at `define_body_plan` time) against the clips already
/// registered, resolved through `clip`. Enforces:
///
/// - a well-formed joint tree: unique non-empty names, exactly one root, every
///   parent present, no cycles, finite geometry, positive sizes;
/// - well-formed declarations: non-empty role names with finite anchors;
///   non-empty unique mode names whose bearing roles each exist on at least
///   one segment;
/// - the **you-supplied-what-you-claimed contract** (B0 — the open-vocabulary
///   successor of the retired verb→slot contract): every action name is
///   non-empty and bound once, and every bound clip exists and animates only
///   joints this plan declares. No action is mandatory and no name is checked
///   against a list.
pub fn validate_plan(
    plan: &BodyPlan,
    clip: impl Fn(&str) -> Option<AnimClip>,
) -> Result<(), String> {
    if plan.segments.is_empty() {
        return Err(format!("plan `{}` has no segments", plan.name));
    }
    // Names unique and non-empty; geometry finite; sizes positive.
    for (i, s) in plan.segments.iter().enumerate() {
        if s.name.is_empty() {
            return Err(format!("plan `{}` has an empty segment name", plan.name));
        }
        if plan.segments[..i].iter().any(|o| o.name == s.name) {
            return Err(format!(
                "plan `{}` has duplicate segment `{}`",
                plan.name, s.name
            ));
        }
        if !(all_finite(&s.pivot_m) && all_finite(&s.size_m) && all_finite(&s.offset_m)) {
            return Err(format!(
                "plan `{}` segment `{}` has non-finite geometry",
                plan.name, s.name
            ));
        }
        if !s.size_m.iter().all(|d| *d > 0.0) {
            return Err(format!(
                "plan `{}` segment `{}` has a non-positive cuboid dimension",
                plan.name, s.name
            ));
        }
        for r in &s.roles {
            if r.role.is_empty() {
                return Err(format!(
                    "plan `{}` segment `{}` declares an empty role name",
                    plan.name, s.name
                ));
            }
            if let Some(at) = &r.at_m
                && !all_finite(at)
            {
                return Err(format!(
                    "plan `{}` segment `{}` role `{}` has a non-finite anchor",
                    plan.name, s.name, r.role
                ));
            }
        }
    }
    // Exactly one root; every parent present.
    let root_count = plan.segments.iter().filter(|s| s.parent.is_none()).count();
    if root_count != 1 {
        return Err(format!(
            "plan `{}` must have exactly one root segment, found {root_count}",
            plan.name
        ));
    }
    for s in &plan.segments {
        if let Some(p) = &s.parent
            && !plan.segments.iter().any(|o| o.name == *p)
        {
            return Err(format!(
                "plan `{}` segment `{}` names missing parent `{}`",
                plan.name, s.name, p
            ));
        }
    }
    // No cycles: walking parents from any node reaches the root within N hops.
    let n = plan.segments.len();
    for s in &plan.segments {
        let mut cur = s;
        let mut hops = 0;
        while let Some(p) = &cur.parent {
            hops += 1;
            if hops > n {
                return Err(format!(
                    "plan `{}` has a parent cycle at segment `{}`",
                    plan.name, s.name
                ));
            }
            cur = plan
                .segments
                .iter()
                .find(|o| o.name == *p)
                .expect("parent presence checked above");
        }
    }
    // Modes: unique non-empty names; every bearing role exists on a segment.
    for (i, m) in plan.modes.iter().enumerate() {
        if m.mode.is_empty() {
            return Err(format!("plan `{}` has an empty mode name", plan.name));
        }
        if plan.modes[..i].iter().any(|o| o.mode == m.mode) {
            return Err(format!(
                "plan `{}` declares mode `{}` twice",
                plan.name, m.mode
            ));
        }
        for role in &m.bearing {
            if !plan
                .segments
                .iter()
                .any(|s| s.roles.iter().any(|r| r.role == *role))
            {
                return Err(format!(
                    "plan `{}` mode `{}` bears on role `{role}`, which no segment declares",
                    plan.name, m.mode
                ));
            }
        }
    }
    // The you-supplied-what-you-claimed contract (open vocabulary — no name
    // is checked against a list and nothing is mandatory).
    for (i, a) in plan.actions.iter().enumerate() {
        if a.action.is_empty() {
            return Err(format!("plan `{}` has an empty action name", plan.name));
        }
        if plan.actions[..i].iter().any(|o| o.action == a.action) {
            return Err(format!(
                "plan `{}` binds action `{}` twice",
                plan.name, a.action
            ));
        }
        let Some(c) = clip(&a.clip) else {
            return Err(format!(
                "plan `{}` action `{}` binds unregistered clip `{}`",
                plan.name, a.action, a.clip
            ));
        };
        for kf in &c.keyframes {
            for r in &kf.rotations {
                if !plan.segments.iter().any(|s| s.name == r.segment) {
                    return Err(format!(
                        "plan `{}` action `{}`: clip `{}` animates joint `{}`, not in the plan",
                        plan.name, a.action, a.clip, r.segment
                    ));
                }
            }
        }
    }
    Ok(())
}

// ------------------------------------------------- the default biped --

// ---------------------------------------------------------------- content --
// The authored content lives in submodules, split by CONCERN (2026-08-01):
// `default_pack` is the first content pack (compiled in for determinism);
// `experiments` are instruments with no standing as content. Re-exported so
// every existing `dc_api::bodies::*` path keeps working. `bake` is NOT
// content: the resting-posture bake (posture-gait member #0), an engine
// primitive over plan data — pure, deterministic, callable from any clock.

mod bake;
mod default_pack;
mod experiments;
mod gait;

pub use bake::{
    BakeOutcome, ChainPose, JointAngle, RestingPosture, bake_resting_posture, stance_chain,
    stance_chains,
};
pub use default_pack::{biped_clips, biped_plan, default_body_pack};
pub use experiments::{experiment_body_pack, longleg_plan, stout_plan};
pub use gait::{
    BandReport, GaitAtSpeed, GaitBakeOutcome, GaitKnobs, GaitVector, InstanceDelta, LimbAtSpeed,
    LimbGait, Regime, RootHeight, bake_gait, duty_exponent, pose,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
        move |name| clips.iter().find(|c| c.name == name).cloned()
    }

    /// B0's acceptance case for the OPEN vocabulary: a plan with NO locomotion
    /// at all — a tree — is legal. `REQUIRED_VERBS` used to reject this with
    /// one line, at define time, on no machine justification.
    #[test]
    fn a_plan_with_no_actions_is_legal() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.actions.clear();
        validate_plan(&plan, clip_lookup(&clips)).expect("a tree does not walk, and defines");
    }

    /// The other half: any action NAME is legal if the claim is backed — a
    /// bird declares `fly`. `KNOWN_VERBS` used to reject this.
    #[test]
    fn any_action_name_is_legal_when_backed() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.actions
            .iter_mut()
            .find(|a| a.action == "walk")
            .unwrap()
            .action = "fly".into();
        validate_plan(&plan, clip_lookup(&clips)).expect("fly is not the engine's business");
    }

    #[test]
    fn duplicate_action_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.actions[1].action = "idle".into(); // now bound twice
        let err = validate_plan(&plan, clip_lookup(&clips)).expect_err("duplicate action");
        assert!(err.contains("twice"), "{err}");
    }

    #[test]
    fn action_binding_missing_clip_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.actions
            .iter_mut()
            .find(|a| a.action == "walk")
            .unwrap()
            .clip = "dc:anim/nonexistent".into();
        assert!(validate_plan(&plan, clip_lookup(&clips)).is_err());
    }

    #[test]
    fn mode_bearing_an_undeclared_role_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.modes.push(ModeDef {
            mode: "swim".into(),
            bearing: vec!["fin".into()], // no segment declares `fin`
        });
        let err = validate_plan(&plan, clip_lookup(&clips)).expect_err("fin is undeclared");
        assert!(err.contains("fin"), "{err}");
    }

    #[test]
    fn duplicate_mode_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.modes.push(ModeDef {
            mode: "stand".into(),
            bearing: vec![],
        });
        assert!(validate_plan(&plan, clip_lookup(&clips)).is_err());
    }

    /// Unique-or-loud (B0): zero matches is a legal absence, one resolves, and
    /// MANY is an error that names the contenders — never a silent first-match.
    #[test]
    fn unique_role_segment_is_loud_on_ambiguity() {
        let plan = biped_plan();
        assert!(unique_role_segment(&plan, "tentacle").unwrap().is_none());
        assert_eq!(
            unique_role_segment(&plan, "look").unwrap().unwrap().name,
            "neck"
        );
        let err = unique_role_segment(&plan, "sole").expect_err("two soles");
        assert!(
            err.contains("leg_l_lower") && err.contains("leg_r_lower"),
            "the error must name the contenders: {err}"
        );
    }

    #[test]
    fn bad_parent_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.segments
            .iter_mut()
            .find(|s| s.name == "head")
            .unwrap()
            .parent = Some("ghost".into());
        let err = validate_plan(&plan, clip_lookup(&clips)).expect_err("missing parent");
        assert!(err.contains("ghost"), "{err}");
    }

    #[test]
    fn parent_cycle_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        // Make the root a child of its own descendant: a cycle, and zero roots.
        plan.segments
            .iter_mut()
            .find(|s| s.name == "trunk")
            .unwrap()
            .parent = Some("head".into());
        assert!(validate_plan(&plan, clip_lookup(&clips)).is_err());
    }

    #[test]
    fn clip_animating_unknown_joint_rejects() {
        let mut clips = biped_clips();
        // Point the walk clip at a joint the plan does not declare.
        clips
            .iter_mut()
            .find(|c| c.name == "dc:anim/biped_walk")
            .unwrap()
            .keyframes[0]
            .rotations
            .push(JointRot {
                segment: "tail".into(),
                euler: [0.1, 0.0, 0.0],
            });
        let err = validate_plan(&biped_plan(), clip_lookup(&clips)).expect_err("tail joint");
        assert!(err.contains("tail"), "{err}");
    }

    #[test]
    fn bad_clip_times_reject() {
        let mut c = biped_clips()[0].clone();
        c.keyframes[1].t = c.keyframes[0].t; // not strictly ascending
        assert!(validate_clip(&c).is_err());
        let mut c = biped_clips()[0].clone();
        c.duration_s = 0.0;
        assert!(validate_clip(&c).is_err());
    }
}
