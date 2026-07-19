//! Body-plan / anim-clip registry conformance (docs/design/bodies.md steps
//! 1–2): plans and clips as data through the command door — namespace
//! ownership, the define-order (clips then plan), the verb→slot contract
//! checked at define time, and the vanilla body pack as a recorded command
//! batch that round-trips into the authored source.

use dc_api::bodies::{AnimSlot, biped_clips, biped_plan, vanilla_body_pack};
use dc_api::payload::{DefineAnimClip, DefineBodyPlan};
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

/// Install the vanilla clips + plan through the door; return the world.
fn world_with_vanilla_bodies() -> HostWorld {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for payload in vanilla_body_pack() {
        let r = apply(&mut world, envelope(&src, &token, payload));
        assert!(r.is_ok(), "vanilla body pack define rejected: {r:?}");
    }
    world
}

#[test]
fn vanilla_pack_defines_through_the_door() {
    let world = world_with_vanilla_bodies();
    assert!(world.body_plan("dc:body/biped").is_some());
    assert!(world.anim_clip("dc:anim/biped_idle").is_some());
    assert!(world.anim_clip("dc:anim/biped_walk").is_some());
    assert!(world.anim_clip("dc:anim/biped_jump").is_some());
    // The stored plan is byte-for-byte the authored source (no drift).
    assert_eq!(world.body_plan("dc:body/biped").unwrap().plan, biped_plan());
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

#[test]
fn missing_required_slot_rejects_at_define_time() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for clip in biped_clips() {
        apply(
            &mut world,
            envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
        );
    }
    let mut plan = biped_plan();
    plan.slots.retain(|s| s.verb != "walk"); // drop the required walk slot
    let r = apply(
        &mut world,
        envelope(&src, &token, Payload::DefineBodyPlan(DefineBodyPlan(plan))),
    );
    match r {
        CommandResult::Rejected(RejectReason::SchemaViolation { reason }) => {
            assert!(reason.contains("walk"), "{reason}");
        }
        other => panic!("expected a schema violation naming walk, got {other:?}"),
    }
}

#[test]
fn slot_bound_to_missing_clip_rejects() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("dc");
    for clip in biped_clips() {
        apply(
            &mut world,
            envelope(&src, &token, Payload::DefineAnimClip(DefineAnimClip(clip))),
        );
    }
    let mut plan = biped_plan();
    // Rebind walk to a clip that was never registered.
    plan.slots
        .iter_mut()
        .find(|s| s.verb == "walk")
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
fn a_new_verb_needs_zero_new_payload_variants() {
    // The whole point of data-defined plans: adding a plan that supports a new
    // verb (here still within the known set) needs no new Rust types. A foreign
    // plugin defines its own clip + plan joining the machinery.
    let mut world = world_with_vanilla_bodies();
    let (src, token) = definer("mod");
    // A tiny standalone clip for the plugin's own one-segment plan.
    let clip = dc_api::bodies::AnimClip {
        name: "mod:anim/blob_idle".into(),
        doc: String::new(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![dc_api::bodies::Keyframe {
            t: 0.0,
            root_bob_m: 0.0,
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
        }],
        slots: vec![
            AnimSlot {
                verb: "idle".into(),
                clip: "mod:anim/blob_idle".into(),
            },
            AnimSlot {
                verb: "walk".into(),
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
    let mut world = world_with_vanilla_bodies();
    let (src, token) = definer("demo");
    // A demo-namespaced clip is fine.
    let clip = dc_api::bodies::AnimClip {
        name: "demo:anim/x".into(),
        doc: String::new(),
        duration_s: 1.0,
        loops: true,
        keyframes: vec![dc_api::bodies::Keyframe {
            t: 0.0,
            root_bob_m: 0.0,
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
