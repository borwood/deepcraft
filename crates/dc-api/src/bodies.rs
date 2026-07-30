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
//!   plus its **verb → anim-slot bindings**. Validated at define time against
//!   the already-registered clips: a plan that leaves a *required* verb's slot
//!   unfilled, or binds a slot to a missing/joint-incompatible clip, is
//!   rejected. This is the schema-checked-at-define-time rule (bodies.md
//!   § body plans) realized for the verb→slot contract.
//!
//! **The define order** (clips, then the plan that binds them) is the honest
//! resolution of the contract's direction: the plan asserts "these verbs are
//! covered", which can only be checked once the clips exist. Clips reference a
//! plan only implicitly, through the joint names they animate — validated when
//! a plan adopts them, exactly like a geology member validating against its
//! class contract at join time.
//!
//! **The firewall (bodies.md § determinism firewall) is LAW here.** These are
//! pure data — segment geometry and keyframes — with no rendering dependency,
//! so they live in dc-api like any other def. The *sampler and renderer* that
//! turn a clip into a posed skeleton are client-only (dc-client `body.rs`);
//! nothing in this module, and nothing the sim reads, ever touches a sampled
//! pose. The sim sees the swept-AABB mover (`crate::character`) and parametric
//! posture only; animation is cosmetic.
//!
//! **The vanilla biped** ([`biped_plan`], [`biped_clips`]) is the in-repo
//! authored source — the default content, compiled in so it is deterministic
//! without a data load. [`vanilla_body_pack`] emits that same content as a
//! recorded command batch ("vanilla is the first pack"), generated *from* the
//! authored source so pack and typed model cannot drift. **The client renderer
//! reads the REGISTRY**, not the authored source: the pack is submitted through
//! the one door at world construction (dc-client `authority.rs`), so the vanilla
//! biped is genuinely loaded as the first pack rather than compiled into the
//! renderer (journal: "the second plan and the glue"). The authored functions
//! remain the *source* the pack is generated from, and the compiled-in default
//! the headless crates test against.
//!
//! **The second plan** ([`stout_plan`]) exists to test one claim, not to look
//! good: bodies.md § IK says two-bone IK "makes one clip serve every mutation of
//! a plan … across differing proportions", and until there were two plans that
//! claim had never met evidence. `dc:body/stout` shares the biped's eleven joint
//! names and binds the biped's *unmodified* clips; only the geometry differs, and
//! it differs deliberately hard (half-length legs, 1.6× arms, a wide trunk, a
//! big head). Its ugliness is the measurement, not a defect.

use serde::{Deserialize, Serialize};

use crate::envelope::{ConsumerId, Tick};
use crate::payload::{DefineAnimClip, DefineBodyPlan, Payload};

/// The body plan a character wears when nothing names one — the **identity
/// default** (spines.md § S-5): a seam whose default value reproduces the
/// pre-seam behaviour exactly, so adding per-character plan selection changes
/// nothing unbidden. Every existing character, and every character spawned
/// without a `body_plan`, wears this.
pub const DEFAULT_BODY_PLAN: &str = "dc:body/biped";

/// The v0 driver-verb vocabulary a plan may declare slots for (bodies.md
/// § verb→animation-slot contract: "move forward…, jump, idle…"). v0 keeps the
/// smallest useful set; the list grows append-only as the locomotion set does.
pub const KNOWN_VERBS: &[&str] = &["idle", "walk", "jump"];

/// The verbs a body plan **must** fill to define. Locomotion (idle + walk) is
/// mandatory — a plan that can be driven has to say what standing and walking
/// look like. `jump` is a known verb but optional (bodies.md: jump may ride a
/// minimal authored pose or a documented fallback); the biped fills it anyway
/// so the slot is exercised end to end.
pub const REQUIRED_VERBS: &[&str] = &["idle", "walk"];

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
    /// key (sockets are step 3+).
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
}

/// A verb → clip binding. The set of `verb`s present is the set of verbs the
/// plan supports; every [`REQUIRED_VERBS`] entry must appear.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnimSlot {
    /// A [`KNOWN_VERBS`] entry.
    pub verb: String,
    /// Namespaced clip name; must be registered and joint-compatible.
    pub clip: String,
}

/// A body plan: a joint-tree of cuboid segments plus its verb→slot bindings.
/// The payload of `dc:registry/define_body_plan`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BodyPlan {
    /// Namespaced name, e.g. `dc:body/biped`.
    pub name: String,
    #[serde(default)]
    pub doc: String,
    pub segments: Vec<SegmentDef>,
    pub slots: Vec<AnimSlot>,
}

/// One joint's rotation at a keyframe: XYZ Euler angles in radians.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JointRot {
    /// Segment name this rotation targets (must exist in the plan that binds
    /// the clip — checked at plan-define time).
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

/// Is `verb` a known driver verb?
pub fn verb_is_known(verb: &str) -> bool {
    KNOWN_VERBS.contains(&verb)
}

/// Validate a plan (at `define_body_plan` time) against the clips already
/// registered, resolved through `clip`. Enforces:
///
/// - a well-formed joint tree: unique non-empty names, exactly one root, every
///   parent present, no cycles, finite geometry, positive sizes;
/// - the **verb→slot contract**: every slot verb is known, no verb is bound
///   twice, every [`REQUIRED_VERBS`] entry is bound, every bound clip exists
///   and animates only joints this plan declares.
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
    // The verb→slot contract.
    for (i, slot) in plan.slots.iter().enumerate() {
        if !verb_is_known(&slot.verb) {
            return Err(format!(
                "plan `{}` slot binds unknown verb `{}` (known: {:?})",
                plan.name, slot.verb, KNOWN_VERBS
            ));
        }
        if plan.slots[..i].iter().any(|o| o.verb == slot.verb) {
            return Err(format!(
                "plan `{}` binds verb `{}` twice",
                plan.name, slot.verb
            ));
        }
        let Some(c) = clip(&slot.clip) else {
            return Err(format!(
                "plan `{}` slot `{}` binds unregistered clip `{}`",
                plan.name, slot.verb, slot.clip
            ));
        };
        for kf in &c.keyframes {
            for r in &kf.rotations {
                if !plan.segments.iter().any(|s| s.name == r.segment) {
                    return Err(format!(
                        "plan `{}` slot `{}`: clip `{}` animates joint `{}`, not in the plan",
                        plan.name, slot.verb, slot.clip, r.segment
                    ));
                }
            }
        }
    }
    for req in REQUIRED_VERBS {
        if !plan.slots.iter().any(|s| s.verb == *req) {
            return Err(format!(
                "plan `{}` leaves required verb `{}` unfilled",
                plan.name, req
            ));
        }
    }
    Ok(())
}

// ------------------------------------------------- the vanilla biped --

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
    }
}

/// The vanilla biped body plan (`dc:body/biped`): trunk root, neck, head, two
/// upper/lower arms, two upper/lower legs — a modest step above the two-cuboid
/// companion (bodies.md staircase step 1: fidelity, not a rig opera). The
/// companion, and characters generally, are rendered as instances of this plan.
///
/// Geometry: feet at the root origin, +Y up, −Z forward (bevy). Pivots stack
/// child-from-parent so a limb rotates at its joint.
pub fn biped_plan() -> BodyPlan {
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
        seg(
            "neck",
            Some("trunk"),
            [0.0, 0.5, 0.0],
            [0.14, 0.12, 0.14],
            [0.0, 0.06, 0.0],
            SKIN,
        ),
        seg(
            "head",
            Some("neck"),
            [0.0, 0.12, 0.0],
            [0.28, 0.28, 0.28],
            [0.0, 0.14, 0.0],
            SKIN,
        ),
        // Arms: shoulder pivots on the trunk, boxes hang down.
        seg(
            "arm_l_upper",
            Some("trunk"),
            [0.33, 0.45, 0.0],
            [0.13, 0.3, 0.13],
            [0.0, -0.15, 0.0],
            LIMB,
        ),
        seg(
            "arm_l_lower",
            Some("arm_l_upper"),
            [0.0, -0.3, 0.0],
            [0.11, 0.28, 0.11],
            [0.0, -0.14, 0.0],
            SKIN,
        ),
        seg(
            "arm_r_upper",
            Some("trunk"),
            [-0.33, 0.45, 0.0],
            [0.13, 0.3, 0.13],
            [0.0, -0.15, 0.0],
            LIMB,
        ),
        seg(
            "arm_r_lower",
            Some("arm_r_upper"),
            [0.0, -0.3, 0.0],
            [0.11, 0.28, 0.11],
            [0.0, -0.14, 0.0],
            SKIN,
        ),
        // Legs: hip pivots on the trunk, boxes reach to the feet.
        seg(
            "leg_l_upper",
            Some("trunk"),
            [0.14, 0.0, 0.0],
            [0.18, 0.45, 0.2],
            [0.0, -0.225, 0.0],
            LIMB,
        ),
        seg(
            "leg_l_lower",
            Some("leg_l_upper"),
            [0.0, -0.45, 0.0],
            [0.16, 0.43, 0.18],
            [0.0, -0.215, 0.0],
            LIMB,
        ),
        seg(
            "leg_r_upper",
            Some("trunk"),
            [-0.14, 0.0, 0.0],
            [0.18, 0.45, 0.2],
            [0.0, -0.225, 0.0],
            LIMB,
        ),
        seg(
            "leg_r_lower",
            Some("leg_r_upper"),
            [0.0, -0.45, 0.0],
            [0.16, 0.43, 0.18],
            [0.0, -0.215, 0.0],
            LIMB,
        ),
    ];
    let slots = vec![
        AnimSlot {
            verb: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        AnimSlot {
            verb: "walk".into(),
            clip: "dc:anim/biped_walk".into(),
        },
        AnimSlot {
            verb: "jump".into(),
            clip: "dc:anim/biped_jump".into(),
        },
    ];
    BodyPlan {
        name: "dc:body/biped".into(),
        doc: "The vanilla humanoid: jointed limbs, a neck, several segments — \
              above Minecraft, still cuboid (bodies.md)."
            .into(),
        segments,
        slots,
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
        seg(
            "neck",
            Some("trunk"),
            [0.0, 0.62, 0.0],
            [0.22, 0.08, 0.22],
            [0.0, 0.04, 0.0],
            SKIN,
        ),
        seg(
            "head",
            Some("neck"),
            [0.0, 0.08, 0.0],
            [0.44, 0.44, 0.44],
            [0.0, 0.22, 0.0],
            SKIN,
        ),
        // Arms: 1.6× the biped's, shouldered out on the wide trunk.
        seg(
            "arm_l_upper",
            Some("trunk"),
            [0.47, 0.55, 0.0],
            [0.16, 0.48, 0.16],
            [0.0, -0.24, 0.0],
            LIMB,
        ),
        seg(
            "arm_l_lower",
            Some("arm_l_upper"),
            [0.0, -0.48, 0.0],
            [0.14, 0.45, 0.14],
            [0.0, -0.225, 0.0],
            SKIN,
        ),
        seg(
            "arm_r_upper",
            Some("trunk"),
            [-0.47, 0.55, 0.0],
            [0.16, 0.48, 0.16],
            [0.0, -0.24, 0.0],
            LIMB,
        ),
        seg(
            "arm_r_lower",
            Some("arm_r_upper"),
            [0.0, -0.48, 0.0],
            [0.14, 0.45, 0.14],
            [0.0, -0.225, 0.0],
            SKIN,
        ),
        // Legs: half the biped's bone lengths, splayed wider under the trunk.
        seg(
            "leg_l_upper",
            Some("trunk"),
            [0.2, 0.0, 0.0],
            [0.26, 0.225, 0.28],
            [0.0, -0.1125, 0.0],
            LIMB,
        ),
        seg(
            "leg_l_lower",
            Some("leg_l_upper"),
            [0.0, -0.225, 0.0],
            [0.24, 0.215, 0.26],
            [0.0, -0.1075, 0.0],
            LIMB,
        ),
        seg(
            "leg_r_upper",
            Some("trunk"),
            [-0.2, 0.0, 0.0],
            [0.26, 0.225, 0.28],
            [0.0, -0.1125, 0.0],
            LIMB,
        ),
        seg(
            "leg_r_lower",
            Some("leg_r_upper"),
            [0.0, -0.225, 0.0],
            [0.24, 0.215, 0.26],
            [0.0, -0.1075, 0.0],
            LIMB,
        ),
    ];
    // The biped's clips, unmodified — that is the whole experiment.
    let slots = vec![
        AnimSlot {
            verb: "idle".into(),
            clip: "dc:anim/biped_idle".into(),
        },
        AnimSlot {
            verb: "walk".into(),
            clip: "dc:anim/biped_walk".into(),
        },
        AnimSlot {
            verb: "jump".into(),
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
        slots,
    }
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

/// The vanilla biped's authored clips: `idle`, `walk`, and a minimal one-shot
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

/// The vanilla body content as a recorded command batch — clips first, then the
/// plans that bind them (the define order the contract requires). Content packs
/// are just registry command batches; this is the bodies pack, and **the running
/// game loads its bodies through it** (dc-client `authority.rs` submits it at
/// world construction under a `registry.define(dc)` grant). Generated from the
/// authored source ([`biped_plan`]/[`stout_plan`]/[`biped_clips`]) so pack and
/// typed model cannot drift (proven by `tests/bodies.rs`'s
/// `vanilla_pack_defines_through_the_door` and
/// `registry_content_equals_the_authored_source`).
///
/// Both plans bind the SAME three clips, which is why the clips are emitted once
/// and the two `DefineBodyPlan`s follow: a clip is standalone data, and "one clip
/// set, many plans" is the shape bodies.md § body plans asserts.
pub fn vanilla_body_pack() -> Vec<Payload> {
    let mut out = Vec::new();
    for clip in biped_clips() {
        out.push(Payload::DefineAnimClip(DefineAnimClip(clip)));
    }
    out.push(Payload::DefineBodyPlan(DefineBodyPlan(biped_plan())));
    out.push(Payload::DefineBodyPlan(DefineBodyPlan(stout_plan())));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip_lookup(clips: &[AnimClip]) -> impl Fn(&str) -> Option<AnimClip> + '_ {
        move |name| clips.iter().find(|c| c.name == name).cloned()
    }

    #[test]
    fn vanilla_biped_is_well_formed() {
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
        validate_plan(&stout, clip_lookup(&clips)).expect("stout plan is valid");
        // Same joint set as the biped, so one clip set serves both.
        let mut a: Vec<&str> = biped_plan().segments.iter().map(|s| &*s.name).collect();
        let mut b: Vec<&str> = stout.segments.iter().map(|s| &*s.name).collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "stout must share the biped's joint names");
        // Same clip bindings, verbatim — no stout-specific clips exist.
        assert_eq!(
            stout.slots,
            biped_plan().slots,
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

    #[test]
    fn missing_required_slot_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.slots.retain(|s| s.verb != "walk"); // drop a required verb
        let err = validate_plan(&plan, clip_lookup(&clips)).expect_err("walk is required");
        assert!(err.contains("walk"), "{err}");
    }

    #[test]
    fn slot_binding_missing_clip_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.slots
            .iter_mut()
            .find(|s| s.verb == "walk")
            .unwrap()
            .clip = "dc:anim/nonexistent".into();
        assert!(validate_plan(&plan, clip_lookup(&clips)).is_err());
    }

    #[test]
    fn unknown_verb_rejects() {
        let clips = biped_clips();
        let mut plan = biped_plan();
        plan.slots[0].verb = "moonwalk".into();
        let err = validate_plan(&plan, clip_lookup(&clips)).expect_err("unknown verb");
        assert!(err.contains("moonwalk"), "{err}");
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
