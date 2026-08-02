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
use crate::payload::{DefineAnimClip, DefineBodyPlan, Payload};

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

const TORSO: [f32; 3] = [0.9, 0.42, 0.12]; // signal-orange (companion legacy)
const SKIN: [f32; 3] = [0.95, 0.85, 0.7];
const LIMB: [f32; 3] = [0.35, 0.4, 0.55];

fn seg(
    name: &str,
    parent: Option<&str>,
    pivot: [f64; 3],
    size: [f64; 3],
    offset: [f64; 3],
    tint: [f32; 3],
) -> SegmentDef {
    SegmentDef {
        name: name.into(),
        parent: parent.map(Into::into),
        pivot_m: pivot,
        size_m: size,
        offset_m: offset,
        tint,
        roles: Vec::new(),
    }
}

/// Negate a lateral coordinate for a mirror without minting `-0.0`: the
/// authored right side had literal `0.0`s where the left does, and `-0.0` is a
/// different bit pattern — the mirror must reproduce the hand-typed plan
/// byte for byte (the seam-first acceptance test).
fn mirror_coord(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { -x }
}

/// A left-side segment mirrored across the X=0 plane. The `_r_` rows of the
/// authored plans are EXACTLY the `_l_` rows with lateral coordinates negated
/// (verified against the previously hand-typed literals — body-plan-structure
/// design pass § 5.4), so the mirror is now structural instead of a
/// coincidence of hand-typing — the defect that opened this arc, fixed at the
/// **producer**: `BodyPlan` stays flat, and the gait bake never needs to know
/// two limbs are partners (phase assignment derives from contact geometry).
fn mirrored(src: &SegmentDef, name: &str, parent: Option<&str>) -> SegmentDef {
    let mut s = src.clone();
    s.name = name.into();
    s.parent = parent.map(Into::into);
    s.pivot_m[0] = mirror_coord(s.pivot_m[0]);
    s.offset_m[0] = mirror_coord(s.offset_m[0]);
    for r in &mut s.roles {
        if let Some(at) = &mut r.at_m {
            at[0] = mirror_coord(at[0]);
        }
    }
    s
}

/// Declare `sole` on a lower-leg segment, anchored at the **bottom face** of
/// its box — derived from the segment's own geometry rather than typed beside
/// it, so a re-authored bone cannot leave a stale anchor behind
/// ([`longleg_plan`] mutates exactly these fields and re-derives).
fn with_sole(mut s: SegmentDef) -> SegmentDef {
    s.roles.push(RoleDef {
        role: "sole".into(),
        at_m: Some([
            s.offset_m[0],
            s.offset_m[1] - s.size_m[1] / 2.0,
            s.offset_m[2],
        ]),
    });
    s
}

/// Declare an unanchored role on a segment (the whole segment carries it).
fn with_role(mut s: SegmentDef, role: &str) -> SegmentDef {
    s.roles.push(RoleDef {
        role: role.into(),
        at_m: None,
    });
    s
}

/// The default biped body plan (`dc:body/biped`): trunk root, neck, head, two
/// upper/lower arms, two upper/lower legs — a modest step above the two-cuboid
/// companion (bodies.md staircase step 1: fidelity, not a rig opera). The
/// companion, and characters generally, are rendered as instances of this plan.
///
/// Geometry: feet at the root origin, +Y up, −Z forward (bevy). Pivots stack
/// child-from-parent so a limb rotates at its joint.
pub fn biped_plan() -> BodyPlan {
    // The left limbs are authored; the right limbs are MIRRORED (B0 — the
    // symmetry is structural now, not a coincidence of hand-typing). Roles:
    // soles on the lower legs (bottom face), `look` on the neck, `face` on
    // the head — only what has a consumer today.
    let arm_l_upper = seg(
        "arm_l_upper",
        Some("trunk"),
        [0.33, 0.45, 0.0],
        [0.13, 0.3, 0.13],
        [0.0, -0.15, 0.0],
        LIMB,
    );
    let arm_l_lower = seg(
        "arm_l_lower",
        Some("arm_l_upper"),
        [0.0, -0.3, 0.0],
        [0.11, 0.28, 0.11],
        [0.0, -0.14, 0.0],
        SKIN,
    );
    let leg_l_upper = seg(
        "leg_l_upper",
        Some("trunk"),
        [0.14, 0.0, 0.0],
        [0.18, 0.45, 0.2],
        [0.0, -0.225, 0.0],
        LIMB,
    );
    let leg_l_lower = with_sole(seg(
        "leg_l_lower",
        Some("leg_l_upper"),
        [0.0, -0.45, 0.0],
        [0.16, 0.43, 0.18],
        [0.0, -0.215, 0.0],
        LIMB,
    ));
    let arm_r_upper = mirrored(&arm_l_upper, "arm_r_upper", Some("trunk"));
    let arm_r_lower = mirrored(&arm_l_lower, "arm_r_lower", Some("arm_r_upper"));
    let leg_r_upper = mirrored(&leg_l_upper, "leg_r_upper", Some("trunk"));
    let leg_r_lower = mirrored(&leg_l_lower, "leg_r_lower", Some("leg_r_upper"));
    let segments = vec![
        // Trunk root: pivot at the hips (~0.9 m); torso box rises from there.
        seg(
            "trunk",
            None,
            [0.0, 0.9, 0.0],
            [0.5, 0.5, 0.28],
            [0.0, 0.25, 0.0],
            TORSO,
        ),
        with_role(
            seg(
                "neck",
                Some("trunk"),
                [0.0, 0.5, 0.0],
                [0.14, 0.12, 0.14],
                [0.0, 0.06, 0.0],
                SKIN,
            ),
            "look",
        ),
        with_role(
            seg(
                "head",
                Some("neck"),
                [0.0, 0.12, 0.0],
                [0.28, 0.28, 0.28],
                [0.0, 0.14, 0.0],
                SKIN,
            ),
            "face",
        ),
        // Arms: shoulder pivots on the trunk, boxes hang down.
        arm_l_upper,
        arm_l_lower,
        arm_r_upper,
        arm_r_lower,
        // Legs: hip pivots on the trunk, boxes reach to the feet.
        leg_l_upper,
        leg_l_lower,
        leg_r_upper,
        leg_r_lower,
    ];
    let modes = vec![ModeDef {
        mode: "stand".into(),
        bearing: vec!["sole".into()],
    }];
    let actions = vec![
        ActionDef {
            action: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        ActionDef {
            action: "walk".into(),
            clip: "dc:anim/biped_walk".into(),
        },
        ActionDef {
            action: "jump".into(),
            clip: "dc:anim/biped_jump".into(),
        },
    ];
    BodyPlan {
        name: "dc:body/biped".into(),
        doc: "The default humanoid: jointed limbs, a neck, several segments — \
              above Minecraft, still cuboid (bodies.md)."
            .into(),
        segments,
        modes,
        actions,
    }
}

/// **The second plan** (`dc:body/stout`) — the instrument that tests bodies.md's
/// retargeting claim, authored in exactly the same shape as [`biped_plan`] and
/// deliberately NOT tasteful.
///
/// Same eleven joint names as the biped, and it binds the biped's three
/// *unmodified* clips (`dc:anim/biped_{idle,walk,jump}`), so [`validate_plan`]
/// accepts it and one clip set genuinely has to serve both bodies. Only the
/// geometry differs, and it differs as hard as the joint topology allows:
///
/// | measure                  | `dc:body/biped` | `dc:body/stout` | ratio |
/// |--------------------------|-----------------|-----------------|-------|
/// | leg (hip→sole)           | 0.88 m          | 0.44 m          | 0.50× |
/// | arm (shoulder→fingertip) | 0.58 m          | 0.93 m          | 1.60× |
/// | hip height               | 0.90 m          | 0.46 m          | 0.51× |
/// | trunk width × depth      | 0.50 × 0.28 m   | 0.78 × 0.50 m   | 1.56× / 1.79× |
/// | head edge                | 0.28 m          | 0.44 m          | 1.57× |
/// | standing height          | 1.80 m          | 1.60 m          | 0.89× |
///
/// The arms reach to 0.08 m above the ground when hanging: a knuckle-dragger.
/// That is the point — it puts the biped's authored swing angles, which are
/// *scale-free*, next to the clips' authored root bob and the renderer's
/// foot-IK correction window, which are **absolute metres**. Anything that
/// survives 0.5× legs and 1.6× arms is genuinely retargeting; anything that
/// does not is a proportion assumption we had never had a second body to find.
///
/// **Tints are identical to the biped's on purpose** — a side-by-side frame then
/// carries only the shape difference, with no colour cue to read as art. The
/// collider is unchanged either way: [`crate::CharacterConfig`] is world-global,
/// so the sim sees the same 1.8 m box under both plans (see the report's honest
/// list of what this experiment does NOT cover).
pub fn stout_plan() -> BodyPlan {
    // Same authoring shape as [`biped_plan`]: left limbs authored, right limbs
    // mirrored, roles on what has a consumer.
    let arm_l_upper = seg(
        "arm_l_upper",
        Some("trunk"),
        [0.47, 0.55, 0.0],
        [0.16, 0.48, 0.16],
        [0.0, -0.24, 0.0],
        LIMB,
    );
    let arm_l_lower = seg(
        "arm_l_lower",
        Some("arm_l_upper"),
        [0.0, -0.48, 0.0],
        [0.14, 0.45, 0.14],
        [0.0, -0.225, 0.0],
        SKIN,
    );
    let leg_l_upper = seg(
        "leg_l_upper",
        Some("trunk"),
        [0.2, 0.0, 0.0],
        [0.26, 0.225, 0.28],
        [0.0, -0.1125, 0.0],
        LIMB,
    );
    let leg_l_lower = with_sole(seg(
        "leg_l_lower",
        Some("leg_l_upper"),
        [0.0, -0.225, 0.0],
        [0.24, 0.215, 0.26],
        [0.0, -0.1075, 0.0],
        LIMB,
    ));
    let arm_r_upper = mirrored(&arm_l_upper, "arm_r_upper", Some("trunk"));
    let arm_r_lower = mirrored(&arm_l_lower, "arm_r_lower", Some("arm_r_upper"));
    let leg_r_upper = mirrored(&leg_l_upper, "leg_r_upper", Some("trunk"));
    let leg_r_lower = mirrored(&leg_l_lower, "leg_r_lower", Some("leg_r_upper"));
    let segments = vec![
        // Trunk root: hips at 0.46 m — half the biped's, matching the halved
        // legs — on a trunk 1.56× wider and 1.79× deeper.
        seg(
            "trunk",
            None,
            [0.0, 0.46, 0.0],
            [0.78, 0.62, 0.5],
            [0.0, 0.31, 0.0],
            TORSO,
        ),
        // Neck: short and thick, so the head sits almost on the shoulders.
        with_role(
            seg(
                "neck",
                Some("trunk"),
                [0.0, 0.62, 0.0],
                [0.22, 0.08, 0.22],
                [0.0, 0.04, 0.0],
                SKIN,
            ),
            "look",
        ),
        with_role(
            seg(
                "head",
                Some("neck"),
                [0.0, 0.08, 0.0],
                [0.44, 0.44, 0.44],
                [0.0, 0.22, 0.0],
                SKIN,
            ),
            "face",
        ),
        // Arms: 1.6× the biped's, shouldered out on the wide trunk.
        arm_l_upper,
        arm_l_lower,
        arm_r_upper,
        arm_r_lower,
        // Legs: half the biped's bone lengths, splayed wider under the trunk.
        leg_l_upper,
        leg_l_lower,
        leg_r_upper,
        leg_r_lower,
    ];
    let modes = vec![ModeDef {
        mode: "stand".into(),
        bearing: vec!["sole".into()],
    }];
    // The biped's clips, unmodified — that is the whole experiment.
    let actions = vec![
        ActionDef {
            action: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        ActionDef {
            action: "walk".into(),
            clip: "dc:anim/biped_walk".into(),
        },
        ActionDef {
            action: "jump".into(),
            clip: "dc:anim/biped_jump".into(),
        },
    ];
    BodyPlan {
        name: "dc:body/stout".into(),
        doc: "A squat, long-armed humanoid: half-length legs, 1.6x arms, a wide \
              trunk and a big head. Same eleven joints as dc:body/biped and it \
              binds the SAME clips — it exists to measure how far one clip set \
              retargets across proportions (bodies.md § IK), not to look good."
            .into(),
        segments,
        modes,
        actions,
    }
}

/// **The third plan** (`dc:body/longleg`) — the control that lets the
/// foot-placement IK *engage at all*.
///
/// journal/0130 measured that the solver has **never once solved**: 176/176
/// sampled frames were beyond the leg's reach and clamped to full extension. The
/// cause is geometric and predates every clip. [`biped_plan`]'s hip pivot sits at
/// **0.900 m** while its legs reach **0.880 m** (0.45 + 0.43), so the sole target
/// at ground level is *outside* the solver's annulus `[|l1−l2|, l1+l2]` before any
/// animation runs. [`stout_plan`] inherits the same sign of error (hip 0.46, reach
/// 0.44). The rendered result is a **hover**, not a bob.
///
/// This plan is [`biped_plan`] with **exactly two numbers changed** — the two leg
/// bone lengths — so the ONLY difference is the one under test:
///
/// | measure            | `dc:body/biped` | `dc:body/longleg` |
/// |--------------------|-----------------|-------------------|
/// | hip height         | 0.900 m         | 0.900 m (same)    |
/// | upper bone `l1`    | 0.450 m         | 0.520 m           |
/// | lower bone `l2`    | 0.430 m         | 0.500 m           |
/// | reach `l1 + l2`    | 0.880 m         | 1.020 m           |
/// | hip − reach        | **+0.020 m**    | **−0.120 m**      |
/// | trunk/arms/head    | —               | identical         |
///
/// **Why 0.120 m of slack, and not less or more.** The number is derived, not
/// picked to look right. The IK target for a planted sole is `(0, −hip_y, fz)`
/// where `fz` is however far forward the *clip* swings that foot, so the reach the
/// solver actually needs is `hypot(hip_y, fz)` — and `fz` itself scales with the
/// bone lengths, so the demand grows as the legs do. Sweeping the authored clip
/// set (`idle` 24 frames, `walk` 12, `jump` 8, both legs, at the renderer's 12 fps
/// grid) gives a fixed point just above **0.102 m** of slack; below it the walk's
/// stride extremes saturate the clamp again, and there is no slack at all that
/// covers every frame without pushing the *resting* knee past a squat:
///
/// | slack | resting knee bend | samples still beyond reach (of 88) |
/// |-------|-------------------|-----------------------------------|
/// | −0.020 (today's biped) | 0.0° (clamped straight) | **88** — every one |
/// | +0.030 | 29.2° | 32 |
/// | +0.050 | 37.3° | 14 |
/// | **+0.120 (this plan)** | **56.2°** | **2** |
/// | +0.180 | 67.1° | 0 |
///
/// 0.120 m is the smallest slack that plants **every** `idle` and `jump` frame and
/// all but two of `walk`'s, while keeping the resting bend under 60°. That the two
/// columns trade against each other at all is the finding: the free variable
/// nobody is using is the root's *vertical* travel (bodies.md § IK, the open user
/// call), and this plan is the instrument that made the trade visible, not a
/// proposal to resolve it.
///
/// **No standing as content** (*existence is not standing*): it rides
/// [`experiment_body_pack`], binds the biped's unmodified clips, and its rest pose
/// is deliberately wrong — un-IK'd, the soles sit 0.120 m *below* the floor. The
/// IK is what lifts them, which is precisely what it is here to demonstrate.
pub fn longleg_plan() -> BodyPlan {
    let mut plan = biped_plan();
    plan.name = "dc:body/longleg".into();
    plan.doc = "The default biped with ONLY its two leg bone lengths changed \
                (0.45/0.43 -> 0.52/0.50, reach 0.880 -> 1.020 m against an \
                unchanged 0.900 m hip). It exists so a sole on the ground is \
                inside the IK's annulus and the knee must bend to reach it — the \
                control journal/0130 lacked. Not content."
        .into();
    // l1 = |lower.pivot| (hip→knee); l2 = |sole anchor| (knee→sole). Both
    // bones grow; the boxes grow with them so the rendered limb is the bone.
    // The legs are found by DECLARATION now (B0): the sole-bearing segments
    // are the lower bones and their parents the upper — the `starts_with
    // ("leg_")` prefix mutation this loop used to run is retired with the
    // naming coupling it leaned on.
    const L1: f64 = 0.52;
    const L2: f64 = 0.50;
    let lowers: Vec<String> = segments_with_role(&plan, "sole")
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let uppers: Vec<String> = plan
        .segments
        .iter()
        .filter(|s| lowers.contains(&s.name))
        .filter_map(|s| s.parent.clone())
        .collect();
    for s in &mut plan.segments {
        if uppers.contains(&s.name) {
            s.size_m[1] = L1;
            s.offset_m[1] = -L1 / 2.0;
        } else if lowers.contains(&s.name) {
            s.pivot_m[1] = -L1;
            s.size_m[1] = L2;
            s.offset_m[1] = -L2 / 2.0;
            // The sole anchor is derived from these fields — re-derive it so
            // the declaration cannot go stale against the bone it sits on.
            for r in &mut s.roles {
                if r.role == "sole" {
                    r.at_m = Some([
                        s.offset_m[0],
                        s.offset_m[1] - s.size_m[1] / 2.0,
                        s.offset_m[2],
                    ]);
                }
            }
        }
    }
    plan
}

fn kf(t: f64, bob: f64, rots: &[(&str, [f64; 3])]) -> Keyframe {
    Keyframe {
        t,
        root_bob_m: bob,
        rotations: rots
            .iter()
            .map(|(s, e)| JointRot {
                segment: (*s).into(),
                euler: *e,
            })
            .collect(),
    }
}

/// The default biped's authored clips: `idle`, `walk`, and a minimal one-shot
/// `jump` (the documented fallback pose — bodies.md step 2). Angles are XYZ
/// Euler radians; walking swings limbs about X (the sagittal plane).
pub fn biped_clips() -> Vec<AnimClip> {
    // Idle: a slow breath — a faint arm splay and a small vertical bob, 2 s loop.
    let idle = AnimClip {
        name: "dc:anim/biped_idle".into(),
        doc: "Standing rest: a slow breathing bob.".into(),
        duration_s: 2.0,
        loops: true,
        keyframes: vec![
            kf(
                0.0,
                0.0,
                &[
                    ("arm_l_upper", [0.06, 0.0, 0.05]),
                    ("arm_r_upper", [0.06, 0.0, -0.05]),
                ],
            ),
            kf(
                1.0,
                0.015,
                &[
                    ("arm_l_upper", [-0.02, 0.0, 0.08]),
                    ("arm_r_upper", [-0.02, 0.0, -0.08]),
                ],
            ),
            kf(
                2.0,
                0.0,
                &[
                    ("arm_l_upper", [0.06, 0.0, 0.05]),
                    ("arm_r_upper", [0.06, 0.0, -0.05]),
                ],
            ),
        ],
    };
    // Walk: contralateral swing, a knee bend on the trailing leg, a step bob.
    // 1 s loop, four poses (two strides). Left-forward at t=0.
    let walk = AnimClip {
        name: "dc:anim/biped_walk".into(),
        doc: "Contralateral limb swing with a step bob (1 s loop).".into(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![
            kf(
                0.0,
                0.0,
                &[
                    ("leg_l_upper", [0.6, 0.0, 0.0]),
                    ("leg_l_lower", [-0.15, 0.0, 0.0]),
                    ("leg_r_upper", [-0.5, 0.0, 0.0]),
                    ("leg_r_lower", [0.5, 0.0, 0.0]),
                    ("arm_l_upper", [-0.5, 0.0, 0.05]),
                    ("arm_r_upper", [0.5, 0.0, -0.05]),
                ],
            ),
            kf(
                0.25,
                0.04,
                &[
                    ("leg_l_upper", [0.05, 0.0, 0.0]),
                    ("leg_l_lower", [-0.05, 0.0, 0.0]),
                    ("leg_r_upper", [-0.05, 0.0, 0.0]),
                    ("leg_r_lower", [0.2, 0.0, 0.0]),
                    ("arm_l_upper", [0.0, 0.0, 0.05]),
                    ("arm_r_upper", [0.0, 0.0, -0.05]),
                ],
            ),
            kf(
                0.5,
                0.0,
                &[
                    ("leg_l_upper", [-0.5, 0.0, 0.0]),
                    ("leg_l_lower", [0.5, 0.0, 0.0]),
                    ("leg_r_upper", [0.6, 0.0, 0.0]),
                    ("leg_r_lower", [-0.15, 0.0, 0.0]),
                    ("arm_l_upper", [0.5, 0.0, 0.05]),
                    ("arm_r_upper", [-0.5, 0.0, -0.05]),
                ],
            ),
            kf(
                0.75,
                0.04,
                &[
                    ("leg_l_upper", [-0.05, 0.0, 0.0]),
                    ("leg_l_lower", [0.2, 0.0, 0.0]),
                    ("leg_r_upper", [0.05, 0.0, 0.0]),
                    ("leg_r_lower", [-0.05, 0.0, 0.0]),
                    ("arm_l_upper", [0.0, 0.0, 0.05]),
                    ("arm_r_upper", [0.0, 0.0, -0.05]),
                ],
            ),
            kf(
                1.0,
                0.0,
                &[
                    ("leg_l_upper", [0.6, 0.0, 0.0]),
                    ("leg_l_lower", [-0.15, 0.0, 0.0]),
                    ("leg_r_upper", [-0.5, 0.0, 0.0]),
                    ("leg_r_lower", [0.5, 0.0, 0.0]),
                    ("arm_l_upper", [-0.5, 0.0, 0.05]),
                    ("arm_r_upper", [0.5, 0.0, -0.05]),
                ],
            ),
        ],
    };
    // Jump: the minimal fallback — a crouch load then an extended reach, 0.6 s
    // one-shot. Real jump blending is later; the slot must exist and play.
    let jump = AnimClip {
        name: "dc:anim/biped_jump".into(),
        doc: "Minimal one-shot: crouch load into an extended reach (fallback).".into(),
        duration_s: 0.6,
        loops: false,
        keyframes: vec![
            kf(
                0.0,
                0.0,
                &[
                    ("leg_l_upper", [0.5, 0.0, 0.0]),
                    ("leg_l_lower", [-0.9, 0.0, 0.0]),
                    ("leg_r_upper", [0.5, 0.0, 0.0]),
                    ("leg_r_lower", [-0.9, 0.0, 0.0]),
                    ("arm_l_upper", [0.3, 0.0, 0.1]),
                    ("arm_r_upper", [0.3, 0.0, -0.1]),
                ],
            ),
            kf(
                0.3,
                0.12,
                &[
                    ("leg_l_upper", [-0.1, 0.0, 0.0]),
                    ("leg_l_lower", [0.1, 0.0, 0.0]),
                    ("leg_r_upper", [-0.1, 0.0, 0.0]),
                    ("leg_r_lower", [0.1, 0.0, 0.0]),
                    ("arm_l_upper", [-2.4, 0.0, 0.1]),
                    ("arm_r_upper", [-2.4, 0.0, -0.1]),
                ],
            ),
            kf(
                0.6,
                0.0,
                &[
                    ("leg_l_upper", [0.1, 0.0, 0.0]),
                    ("leg_l_lower", [-0.2, 0.0, 0.0]),
                    ("leg_r_upper", [0.1, 0.0, 0.0]),
                    ("leg_r_lower", [-0.2, 0.0, 0.0]),
                    ("arm_l_upper", [-0.4, 0.0, 0.1]),
                    ("arm_r_upper", [-0.4, 0.0, -0.1]),
                ],
            ),
        ],
    };
    vec![idle, walk, jump]
}

/// The **default pack's** body content as a recorded command batch — clips first,
/// then the plans that bind them (the define order the contract requires). Content
/// packs are just registry command batches; this is the bodies pack, and **the
/// running game loads its bodies through it** (dc-client `authority.rs` submits it
/// at world construction under a `registry.define(dc)` grant). Generated from the
/// authored source ([`biped_plan`]/[`biped_clips`]) so pack and typed model cannot
/// drift (proven by `tests/bodies.rs`'s `default_pack_defines_through_the_door`
/// and `registry_content_equals_the_authored_source`).
///
/// The experiment plans are deliberately **NOT** in here — see
/// [`experiment_body_pack`].
pub fn default_body_pack() -> Vec<Payload> {
    let mut out = Vec::new();
    for clip in biped_clips() {
        out.push(Payload::DefineAnimClip(DefineAnimClip(clip)));
    }
    out.push(Payload::DefineBodyPlan(DefineBodyPlan(biped_plan())));
    out
}

/// **The body experiments' pack — instruments, with NO standing as content.**
///
/// - [`stout_plan`] tests bodies.md's claim that two-bone IK retargets one clip
///   set across proportions.
/// - [`longleg_plan`] tests whether the foot-placement IK solves *at all* when the
///   ground is inside the leg's reach — the control journal/0130 lacked.
///
/// Neither is a creature anybody designed, ratified, or wants in the world. They
/// ride their own batch rather than [`default_body_pack`] precisely so that the
/// **default pack** stays the default content. *Existence is not standing*
/// (CLAUDE.md): an unratified body sitting inside the default pack would, in three
/// months, be something a session found in the tree and assumed belonged there.
/// Deleting the experiments is one call site in dc-client `authority.rs` plus this
/// function and the two plan functions.
///
/// It emits **only plans**: the clips they bind are the default pack's,
/// unmodified, which is the entire point — so this batch must be submitted *after*
/// [`default_body_pack`], or the verb→slot contract rejects it for binding
/// unregistered clips.
pub fn experiment_body_pack() -> Vec<Payload> {
    vec![
        Payload::DefineBodyPlan(DefineBodyPlan(stout_plan())),
        Payload::DefineBodyPlan(DefineBodyPlan(longleg_plan())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
        move |name| clips.iter().find(|c| c.name == name).cloned()
    }

    #[test]
    fn default_biped_is_well_formed() {
        let clips = biped_clips();
        for c in &clips {
            validate_clip(c).unwrap_or_else(|e| panic!("clip {} invalid: {e}", c.name));
        }
        validate_plan(&biped_plan(), clip_lookup(&clips)).expect("biped plan is valid");
    }

    /// The second plan validates against the **unmodified** biped clips — the
    /// precondition of the whole retargeting experiment. If this ever fails,
    /// somebody edited a clip or a joint name and the instrument is gone.
    #[test]
    fn stout_is_well_formed_and_binds_the_biped_clips() {
        let clips = biped_clips();
        let stout = stout_plan();
        let biped = biped_plan();
        validate_plan(&stout, clip_lookup(&clips)).expect("stout plan is valid");
        // Same joint set as the biped, so one clip set serves both.
        let mut a: Vec<&str> = biped.segments.iter().map(|s| &*s.name).collect();
        let mut b: Vec<&str> = stout.segments.iter().map(|s| &*s.name).collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "stout must share the biped's joint names");
        // Same clip bindings, verbatim — no stout-specific clips exist.
        assert_eq!(
            stout.actions, biped.actions,
            "stout binds the biped's clips unmodified"
        );
    }

    /// The proportions are the experiment's independent variable, so pin the
    /// *ratios* (not the absolute numbers, which are free to be re-authored):
    /// the second plan must differ a LOT or it measures nothing.
    #[test]
    fn stout_proportions_differ_materially() {
        // Hip→sole: the upper bone (the lower segment's pivot offset) plus the
        // lower bone (its box's centre offset plus half its length).
        let leg = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "leg_l_lower").unwrap();
            lower.pivot_m[1].abs() + lower.offset_m[1].abs() + lower.size_m[1] / 2.0
        };
        let arm = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "arm_l_lower").unwrap();
            lower.pivot_m[1].abs() + lower.offset_m[1].abs() + lower.size_m[1] / 2.0
        };
        let (bip, sto) = (biped_plan(), stout_plan());
        let leg_ratio = leg(&sto) / leg(&bip);
        let arm_ratio = arm(&sto) / arm(&bip);
        assert!(
            leg_ratio <= 0.6,
            "stout legs must be at most 0.6x the biped's, got {leg_ratio:.3}"
        );
        assert!(
            arm_ratio >= 1.4,
            "stout arms must be at least 1.4x the biped's, got {arm_ratio:.3}"
        );
        // Hips ride at the top of the legs in both plans (the geometry is
        // self-consistent: a plan whose hip height did not match its leg length
        // would float or sink for reasons that are not about retargeting).
        for p in [&bip, &sto] {
            let hip = p
                .segments
                .iter()
                .find(|s| s.name == "trunk")
                .unwrap()
                .pivot_m[1];
            let sole_gap = hip - leg(p);
            assert!(
                (0.0..=0.05).contains(&sole_gap),
                "plan `{}` hip {hip} vs leg {} leaves a {sole_gap} m rest gap",
                p.name,
                leg(p)
            );
        }
    }

    /// **The third plan's defining property, asserted as a property and not as a
    /// pair of magnitudes**: `dc:body/longleg` is the biped with a leg reach
    /// *greater* than its hip height, so a sole at ground level lands **inside**
    /// the IK annulus `[|l1−l2|, l1+l2]` instead of outside it. Every other plan
    /// has the opposite sign, which is why the solver had never solved
    /// (journal/0130).
    ///
    /// Also pins the isolation: **only the leg bones differ from the biped.** If a
    /// later re-authoring drifts the trunk, the arms or the head, the control stops
    /// being a control and this fails loudly rather than quietly measuring two
    /// variables at once.
    #[test]
    fn longleg_reaches_the_ground_and_changes_nothing_else() {
        let clips = biped_clips();
        let long = longleg_plan();
        let biped = biped_plan();
        validate_plan(&long, clip_lookup(&clips)).expect("longleg plan is valid");
        assert_eq!(
            long.actions, biped.actions,
            "longleg binds the default clips unmodified"
        );

        // hip→sole reach and hip height, from the same fields the renderer's
        // `leg_rigs` reads.
        let rig = |p: &BodyPlan| {
            let lower = p.segments.iter().find(|s| s.name == "leg_l_lower").unwrap();
            let l1 = lower.pivot_m[1].abs();
            let l2 = lower.offset_m[1].abs() + lower.size_m[1] / 2.0;
            let hip = p
                .segments
                .iter()
                .find(|s| s.name == "trunk")
                .unwrap()
                .pivot_m[1];
            (hip, l1 + l2)
        };
        let (hip_b, reach_b) = rig(&biped);
        let (hip_l, reach_l) = rig(&long);
        assert!(
            reach_b < hip_b,
            "the biped's ground target is unreachable by construction \
             (reach {reach_b} < hip {hip_b}) — the premise of the experiment"
        );
        assert!(
            reach_l > hip_l,
            "longleg must be able to REACH the ground: reach {reach_l} <= hip {hip_l}"
        );
        assert!(
            (hip_l - hip_b).abs() < 1e-12,
            "the hip must not move — the slack has to come from the bones alone"
        );
        // Slack big enough that the knee is unmistakably bent, not a hair off
        // straight: the bend at rest is 2·acos(hip / reach) for near-equal bones.
        let bend = 2.0 * (hip_l / reach_l).acos();
        assert!(
            bend > 45.0f64.to_radians(),
            "the resting knee must be visibly bent, got {:.1}°",
            bend.to_degrees()
        );

        // Nothing but the legs differs from the biped.
        for a in &biped.segments {
            let b = long
                .segments
                .iter()
                .find(|s| s.name == a.name)
                .unwrap_or_else(|| panic!("longleg keeps the biped's joint `{}`", a.name));
            if a.name.starts_with("leg_") {
                continue;
            }
            assert_eq!(a, b, "segment `{}` must be untouched", a.name);
        }
        assert_eq!(
            biped.segments.len(),
            long.segments.len(),
            "same eleven joints"
        );
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

    /// The mirror helper reproduces the previously hand-typed right side byte
    /// for byte — the seam-first acceptance test for making the symmetry
    /// structural. Pivots pinned to the old literals; zeros must stay +0.0.
    #[test]
    fn mirror_reproduces_the_authored_right_side() {
        let plan = biped_plan();
        let by = |n: &str| plan.segments.iter().find(|s| s.name == n).unwrap();
        assert_eq!(by("arm_r_upper").pivot_m, [-0.33, 0.45, 0.0]);
        assert_eq!(by("leg_r_upper").pivot_m, [-0.14, 0.0, 0.0]);
        for (l, r) in [
            ("arm_l_upper", "arm_r_upper"),
            ("arm_l_lower", "arm_r_lower"),
            ("leg_l_upper", "leg_r_upper"),
            ("leg_l_lower", "leg_r_lower"),
        ] {
            let (l, r) = (by(l), by(r));
            assert_eq!(l.size_m, r.size_m);
            assert_eq!(l.tint, r.tint);
            assert_eq!(l.pivot_m[0], -r.pivot_m[0]);
            assert_eq!(l.pivot_m[1], r.pivot_m[1]);
            assert_eq!(l.pivot_m[2], r.pivot_m[2]);
            // No -0.0 minted where the left has 0.0 (byte-identity, not just ==).
            for v in [r.pivot_m, r.offset_m] {
                for x in v {
                    if x == 0.0 {
                        assert!(x.is_sign_positive(), "mirror minted a -0.0 in {v:?}");
                    }
                }
            }
        }
    }

    /// The declared sole anchors sit on the bottom FACE of their bone's box —
    /// including after `longleg_plan`'s mutation, which must re-derive them.
    #[test]
    fn sole_anchors_sit_on_the_bottom_face() {
        for plan in [biped_plan(), stout_plan(), longleg_plan()] {
            let soles = segments_with_role(&plan, "sole");
            assert_eq!(soles.len(), 2, "plan `{}` declares two soles", plan.name);
            for s in soles {
                let at = s.roles.iter().find(|r| r.role == "sole").unwrap().at_m;
                let expect = [
                    s.offset_m[0],
                    s.offset_m[1] - s.size_m[1] / 2.0,
                    s.offset_m[2],
                ];
                assert_eq!(
                    at,
                    Some(expect),
                    "plan `{}` segment `{}`: sole anchor off the bottom face",
                    plan.name,
                    s.name
                );
            }
        }
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
