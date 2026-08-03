//! Body-plan / anim-clip registry conformance (docs/design/bodies.md steps
//! 1–2, reshaped by B0 2026-08-01): plans and clips as data through the
//! command door — namespace ownership, the define-order (clips then plan),
//! the you-supplied-what-you-claimed contract checked at define time (the
//! action vocabulary is OPEN; the verb→slot contract is retired), and the
//! default body pack as a recorded command batch that round-trips into the
//! authored source.

use dc_api::bodies::{
    ActionDef, DEFAULT_BODY_PLAN, biped_clips, biped_plan, default_body_pack, experiment_body_pack,
    longleg_plan, stout_plan,
};
use dc_api::payload::{DefineAnimClip, DefineBodyPlan, QueryData, SpawnCharacter, Vec3f};
use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant, HostWorld,
    Payload, RejectReason,
};

fn envelope(source: &ConsumerId, grant: &CapabilityToken, payload: Payload) -> CommandEnvelope {
    CommandEnvelope {
        id: payload.command_id().to_string(),
        source: source.clone(),
        grant: grant.clone(),
        payload,
        target_tick: None,
        txn: None,
    }
}

fn definer(ns: &str) -> (ConsumerId, CapabilityToken) {
    (
        ConsumerId::new(ConsumerKind::Plugin, format!("{ns}-pack")),
        CapabilityToken::new(vec![Grant::RegistryDefine {
            namespace: ns.into(),
        }]),
    )
}

/// Submit one define and tick; return its result.
fn apply(world: &mut HostWorld, env: CommandEnvelope) -> CommandResult {
    let ack = world.submit(env).expect("structurally valid");
    let receipts = world.tick();
    receipts
        .into_iter()
        .find(|r| r.consumer_seq == ack.consumer_seq)
        .expect("receipt")
        .receipt
        .result
}

/// Install the default pack's clips + plan through the door, then the experiment
/// plans (which bind the default pack's clips, so order matters); return the
/// world. This is the batch order dc-client boots with.
fn world_with_default_bodies() -> HostWorld {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for payload in default_body_pack()
        .into_iter()
        .chain(experiment_body_pack())
    {
        let r = apply(&mut world, envelope(&src, &token, payload));
        assert!(r.is_ok(), "body pack define rejected: {r:?}");
    }
    world
}

#[test]
fn default_pack_defines_through_the_door() {
    let world = world_with_default_bodies();
    assert!(world.body_plan("dc:body/biped").is_some());
    assert!(world.anim_clip("dc:anim/biped_idle").is_some());
    assert!(world.anim_clip("dc:anim/biped_jump").is_some());
    // The stored plan is byte-for-byte the authored source (no drift).
    assert_eq!(world.body_plan("dc:body/biped").unwrap().plan, biped_plan());
}

/// **The re-housing proof.** The client renderer used to call `biped_plan()` /
/// `biped_clips()` as compiled-in Rust and now reads the *registry*. The render
/// is unchanged iff what the registry hands back is `==` to what the compiled-in
/// call handed back — segment for segment (geometry, pivots, offsets, tints) and
/// keyframe for keyframe (times, joint eulers). Structural equality of the
/// exact inputs to the renderer is a stronger statement than a screenshot diff:
/// every downstream line of `character.rs`/`body.rs` is a pure function of these
/// values, so equal inputs give an identical frame by construction.
#[test]
fn registry_content_equals_the_authored_source() {
    let world = world_with_default_bodies();
    // Plans.
    assert_eq!(
        world.body_plan("dc:body/biped").expect("biped").plan,
        biped_plan(),
        "the plan the renderer now reads must equal the one it used to call"
    );
    assert_eq!(
        world.body_plan("dc:body/stout").expect("stout").plan,
        stout_plan()
    );
    assert_eq!(
        world.body_plan("dc:body/longleg").expect("longleg").plan,
        longleg_plan()
    );
    // Clips, each one, by name.
    for authored in biped_clips() {
        let stored = world
            .anim_clip(&authored.name)
            .unwrap_or_else(|| panic!("clip {} registered", authored.name));
        assert_eq!(stored.clip, authored, "clip {} drifted", authored.name);
    }
    // And nothing else snuck into the bodies registry: the default biped plus the
    // two experiment plans, and one clip set for all three.
    assert_eq!(world.body_plans().count(), 3);
    assert_eq!(world.anim_clips().count(), biped_clips().len());
}

/// **The experiments are not default content.** `default_body_pack` carries the
/// clip set and the biped only; the ill-proportioned `dc:body/stout` and the
/// deliberately over-long `dc:body/longleg` ride their own batch, which registers
/// **no clips at all** — they bind the default pack's. That is both the
/// experiments' whole design (one clip set, three plans) and the reason they can
/// be deleted without touching the default pack (*existence is not standing*).
#[test]
fn the_experiment_pack_is_separate_and_carries_no_clips() {
    let names = |pack: &[Payload]| -> (Vec<String>, Vec<String>) {
        let clips = pack
            .iter()
            .filter_map(|p| match p {
                Payload::DefineAnimClip(DefineAnimClip(c)) => Some(c.name.clone()),
                _ => None,
            })
            .collect();
        let plans = pack
            .iter()
            .filter_map(|p| match p {
                Payload::DefineBodyPlan(DefineBodyPlan(pl)) => Some(pl.name.clone()),
                _ => None,
            })
            .collect();
        (clips, plans)
    };
    let default_pack = default_body_pack();
    let (v_clips, v_plans) = names(&default_pack);
    // The default pack ships TWO clips since 2026-08-02: `dc:anim/biped_walk`
    // was retired as content (user call #3) because locomotion is DERIVED, so
    // what is left is the idle breath and the jump one-shot. Counted against
    // the authored source rather than a literal, so the two cannot drift.
    assert_eq!(
        v_clips.len(),
        biped_clips().len(),
        "the pack carries exactly the authored clip set: {v_clips:?}"
    );
    assert!(
        !v_clips.iter().any(|c| c == "dc:anim/biped_walk"),
        "the retired walk clip must not ship: {v_clips:?}"
    );
    assert_eq!(v_plans, vec!["dc:body/biped".to_string()]);
    // Clips before the plan — the define order the contract requires.
    let first_plan = default_pack
        .iter()
        .position(|p| matches!(p, Payload::DefineBodyPlan(_)))
        .unwrap();
    let last_clip = default_pack
        .iter()
        .rposition(|p| matches!(p, Payload::DefineAnimClip(_)))
        .unwrap();
    assert!(last_clip < first_plan, "clips must be defined before plans");

    let (e_clips, e_plans) = names(&experiment_body_pack());
    assert!(
        e_clips.is_empty(),
        "the experiments author NO clips — they reuse the default pack's, which \
         is the measurement: {e_clips:?}"
    );
    assert_eq!(
        e_plans,
        vec!["dc:body/stout".to_string(), "dc:body/longleg".to_string()]
    );
}

/// And the experiment pack **cannot** load on its own: submitted into a world
/// without the default pack's clips, the you-supplied-what-you-claimed
/// contract rejects it (its actions bind unregistered clips). This is the
/// ordering constraint stated as a test rather than as a comment.
#[test]
fn the_experiment_pack_requires_the_default_packs_clips() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for payload in experiment_body_pack() {
        let r = apply(&mut world, envelope(&src, &token, payload));
        assert!(
            matches!(
                r,
                CommandResult::Rejected(RejectReason::SchemaViolation { .. })
            ),
            "the experiment plans bind clips they do not author: {r:?}"
        );
    }
    assert!(world.body_plan("dc:body/stout").is_none());
    assert!(world.body_plan("dc:body/longleg").is_none());
}

// ------------------------------------ per-character plan selection (S-5) --

fn spawn(world: &mut HostWorld, name: &str, plan: Option<&str>) -> CommandResult {
    let src = ConsumerId::new(ConsumerKind::McpSession, "dev");
    let token = CapabilityToken::new(vec![Grant::EntitySpawn]);
    apply(
        world,
        envelope(
            &src,
            &token,
            Payload::SpawnCharacter(SpawnCharacter {
                name: name.into(),
                pos: Vec3f::new(0.5, 40.0, 0.5),
                body_plan: plan.map(Into::into),
            }),
        ),
    )
}

/// The identity default: a spawn that names no plan gets `dc:body/biped`, and it
/// works in a world where **no pack has been loaded at all** — the plan is
/// cosmetic, so the engine's character primitive must not require content.
#[test]
fn spawn_without_a_plan_gets_the_identity_default() {
    let mut world = HostWorld::new(1);
    let r = spawn(&mut world, "scout", None);
    assert!(
        r.is_ok(),
        "a plan-less world still spawns characters: {r:?}"
    );
    assert_eq!(
        world.character("scout").expect("spawned").body_plan,
        DEFAULT_BODY_PLAN
    );
}

/// A named plan that a pack registered is worn; the pose query reads it back, so
/// a driver confirms what it got rather than trusting the spawn receipt.
#[test]
fn spawn_with_a_registered_plan_wears_it_and_reads_back() {
    let mut world = world_with_default_bodies();
    assert!(spawn(&mut world, "squat", Some("dc:body/stout")).is_ok());
    assert_eq!(
        world.character("squat").expect("spawned").body_plan,
        "dc:body/stout"
    );
    // Readback through the character's own proprioception query.
    let src = ConsumerId::new(ConsumerKind::McpSession, "dev");
    let token = CapabilityToken::new(vec![Grant::CharacterControl { character: None }]);
    let env = envelope(
        &src,
        &token,
        Payload::CharacterPose(dc_api::payload::CharacterPose {
            character: "squat".into(),
        }),
    );
    let receipt = world.query(&env);
    match receipt.result {
        dc_api::QueryResult::Ok(QueryData::CharacterPose { body_plan, .. }) => {
            assert_eq!(body_plan, "dc:body/stout");
        }
        other => panic!("expected a pose readback, got {other:?}"),
    }
}

/// **A plan name nobody registered is refused with a receipt, never silently
/// defaulted** — and no character is created. Silently defaulting would render a
/// body the caller did not ask for and hide a pack that failed to load.
#[test]
fn spawn_with_an_unregistered_plan_is_refused_with_a_receipt() {
    let mut world = world_with_default_bodies();
    let r = spawn(&mut world, "ghost", Some("mod:body/nonexistent"));
    match r {
        CommandResult::Rejected(RejectReason::UnknownBodyPlan { name }) => {
            assert_eq!(name, "mod:body/nonexistent");
        }
        other => panic!("expected UnknownBodyPlan, got {other:?}"),
    }
    assert!(
        world.character("ghost").is_none(),
        "a refused spawn creates nothing"
    );
    // Same refusal in a world with no plans registered at all — including for
    // the default plan's own name when it is named EXPLICITLY. Naming a plan is
    // a claim about content; omitting one is not.
    let mut bare = HostWorld::new(2);
    let r = spawn(&mut bare, "scout", Some(DEFAULT_BODY_PLAN));
    assert!(
        matches!(
            r,
            CommandResult::Rejected(RejectReason::UnknownBodyPlan { .. })
        ),
        "an explicitly named plan is checked even when it is the default: {r:?}"
    );
}

#[test]
fn plan_before_its_clips_is_rejected() {
    // Defining the plan first (no clips registered yet) violates the contract:
    // its required slots bind clips that do not exist.
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    let r = apply(
        &mut world,
        envelope(
            &src,
            &token,
            Payload::DefineBodyPlan(DefineBodyPlan(biped_plan())),
        ),
    );
    assert!(
        matches!(
            r,
            CommandResult::Rejected(RejectReason::SchemaViolation { .. })
        ),
        "plan before its clips must reject: {r:?}"
    );
    assert!(world.body_plan("dc:body/biped").is_none());
}

/// B0: the action vocabulary is OPEN and nothing is mandatory — a plan with
/// no locomotion at all (a tree) defines through the door. `REQUIRED_VERBS`
/// used to reject exactly this, at define time, on no machine justification.
#[test]
fn a_plan_with_no_actions_defines_through_the_door() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for clip in biped_clips() {
        apply(
            &mut world,
            envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
        );
    }
    let mut plan = biped_plan();
    plan.actions.clear();
    let r = apply(
        &mut world,
        envelope(&src, &token, Payload::DefineBodyPlan(DefineBodyPlan(plan))),
    );
    assert!(r.is_ok(), "a tree does not walk, and defines: {r:?}");
    assert!(world.body_plan("dc:body/biped").is_some());
}

#[test]
fn action_bound_to_missing_clip_rejects() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for clip in biped_clips() {
        apply(
            &mut world,
            envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
        );
    }
    let mut plan = biped_plan();
    // Rebind an action to a clip that was never registered.
    plan.actions
        .iter_mut()
        .find(|a| a.action == "jump")
        .unwrap()
        .clip = "dc:anim/ghost".into();
    let r = apply(
        &mut world,
        envelope(&src, &token, Payload::DefineBodyPlan(DefineBodyPlan(plan))),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::SchemaViolation { .. })
    ));
}

#[test]
fn bad_parent_rejects_at_define_time() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for clip in biped_clips() {
        apply(
            &mut world,
            envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
        );
    }
    let mut plan = biped_plan();
    plan.segments
        .iter_mut()
        .find(|s| s.name == "head")
        .unwrap()
        .parent = Some("ghost".into());
    let r = apply(
        &mut world,
        envelope(&src, &token, Payload::DefineBodyPlan(DefineBodyPlan(plan))),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::SchemaViolation { .. })
    ));
    assert!(world.body_plan("dc:body/biped").is_none());
}

#[test]
fn a_new_action_needs_zero_new_payload_variants() {
    // The whole point of data-defined plans: adding a plan that supports its
    // own actions needs no new Rust types — and since B0 the names need no
    // engine's permission either (`ooze` is nobody's known verb). A foreign
    // plugin defines its own clip + plan joining the machinery.
    let mut world = world_with_default_bodies();
    let (src, token) = definer("mod");
    // A tiny standalone clip for the plugin's own one-segment plan.
    let clip = dc_api::bodies::AnimClip {
        name: "mod:anim/blob_idle".into(),
        doc: String::new(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![dc_api::bodies::Keyframe {
            t: 0.0,
            rotations: vec![dc_api::bodies::JointRot {
                segment: "body".into(),
                euler: [0.0, 0.0, 0.0],
            }],
        }],
    };
    let walk = dc_api::bodies::AnimClip {
        name: "mod:anim/blob_walk".into(),
        ..clip.clone()
    };
    apply(
        &mut world,
        envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
    );
    apply(
        &mut world,
        envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(walk))),
    );
    let plan = dc_api::bodies::BodyPlan {
        name: "mod:body/blob".into(),
        doc: String::new(),
        segments: vec![dc_api::bodies::SegmentDef {
            name: "body".into(),
            parent: None,
            pivot_m: [0.0, 0.0, 0.0],
            size_m: [1.0, 1.0, 1.0],
            offset_m: [0.0, 0.5, 0.0],
            tint: [0.5, 0.5, 0.5],
            roles: vec![],
        }],
        modes: vec![],
        actions: vec![
            ActionDef {
                action: "idle".into(),
                clip: "mod:anim/blob_idle".into(),
            },
            ActionDef {
                action: "ooze".into(),
                clip: "mod:anim/blob_walk".into(),
            },
        ],
    };
    let r = apply(
        &mut world,
        envelope(&src, &token, Payload::DefineBodyPlan(DefineBodyPlan(plan))),
    );
    assert!(r.is_ok(), "plugin plan joins the machinery: {r:?}");
    assert!(world.body_plan("mod:body/blob").is_some());
}

#[test]
fn body_defs_are_namespace_owned() {
    let mut world = world_with_default_bodies();
    let (src, token) = definer("demo");
    // A demo-namespaced clip is fine.
    let clip = dc_api::bodies::AnimClip {
        name: "demo:anim/x".into(),
        doc: String::new(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![dc_api::bodies::Keyframe {
            t: 0.0,
            rotations: vec![],
        }],
    };
    assert!(
        apply(
            &mut world,
            envelope(
                &src,
                &token,
                Payload::DefineAnimClip(DefineAnimClip(clip.clone()))
            )
        )
        .is_ok()
    );
    // The dc-namespaced clip is not the demo grant's to define.
    let foreign = dc_api::bodies::AnimClip {
        name: "dc:anim/intruder".into(),
        ..clip
    };
    let r = apply(
        &mut world,
        envelope(
            &src,
            &token,
            Payload::DefineAnimClip(DefineAnimClip(foreign)),
        ),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
    assert!(world.anim_clip("dc:anim/intruder").is_none());
}
