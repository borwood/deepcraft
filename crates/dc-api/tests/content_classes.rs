//! Content-class registry conformance (geology backbone slice 1):
//! classes-as-contracts through the command door — namespace ownership,
//! define-time schema validation, the vanilla geology pack as a recorded
//! command batch, and the bridge into the canonically ordered typed set.

use dc_api::classes::{
    geology_set_from_defs, vanilla_geology_pack, ParamEntry, ParamKind, ParamSpec, ParamValue,
};
use dc_api::payload::{DefineClassMember, DefineContentClass};
use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant, HostWorld,
    Payload, RejectReason,
};
use dc_core::materials::geology;

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

fn simple_class(name: &str) -> Payload {
    Payload::DefineContentClass(DefineContentClass {
        name: name.into(),
        doc: "test class".into(),
        params: vec![ParamSpec {
            name: "abundance".into(),
            kind: ParamKind::Number {
                min: 0.0,
                max: 100.0,
            },
            required: true,
        }],
    })
}

fn simple_member(name: &str, class: &str, abundance: f64) -> Payload {
    Payload::DefineClassMember(DefineClassMember {
        name: name.into(),
        class: class.into(),
        params: vec![ParamEntry {
            name: "abundance".into(),
            value: ParamValue::Number(abundance),
        }],
    })
}

#[test]
fn class_and_member_defines_are_namespace_owned() {
    let mut world = HostWorld::new(1);
    let (src, token) = definer("demo");

    // Own namespace: accepted.
    let r = apply(&mut world, envelope(&src, &token, simple_class("demo:stratum/test")));
    assert!(r.is_ok(), "own-namespace class define: {r:?}");
    assert!(world.content_class("demo:stratum/test").is_some());

    // Foreign namespace: rejected, nothing stored.
    let r = apply(&mut world, envelope(&src, &token, simple_class("other:stratum/test")));
    assert!(
        matches!(r, CommandResult::Rejected(RejectReason::MissingCapability { .. })),
        "foreign-namespace class define must reject: {r:?}"
    );
    assert!(world.content_class("other:stratum/test").is_none());

    // Members: the MEMBER's namespace is what the grant must own; the class
    // may live elsewhere. A second definer registers into demo's class.
    let (other_src, other_token) = definer("mod");
    let r = apply(
        &mut world,
        envelope(
            &other_src,
            &other_token,
            simple_member("mod:member/a", "demo:stratum/test", 2.0),
        ),
    );
    assert!(r.is_ok(), "cross-namespace membership: {r:?}");
    // …but it cannot mint members in namespaces it does not own.
    let r = apply(
        &mut world,
        envelope(
            &other_src,
            &other_token,
            simple_member("demo:member/forged", "demo:stratum/test", 2.0),
        ),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
}

#[test]
fn member_defines_are_schema_validated_at_define_time() {
    let mut world = HostWorld::new(2);
    let (src, token) = definer("demo");
    assert!(apply(&mut world, envelope(&src, &token, simple_class("demo:stratum/test"))).is_ok());

    // Unknown class.
    let r = apply(
        &mut world,
        envelope(&src, &token, simple_member("demo:member/x", "demo:stratum/nope", 1.0)),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::UnknownClass { .. })
    ));

    // Contract violations reject and store nothing.
    for (label, bad) in [
        (
            "missing required param",
            Payload::DefineClassMember(DefineClassMember {
                name: "demo:member/x".into(),
                class: "demo:stratum/test".into(),
                params: vec![],
            }),
        ),
        (
            "out-of-bounds value",
            simple_member("demo:member/x", "demo:stratum/test", 1e6),
        ),
        (
            "unknown param",
            Payload::DefineClassMember(DefineClassMember {
                name: "demo:member/x".into(),
                class: "demo:stratum/test".into(),
                params: vec![
                    ParamEntry {
                        name: "abundance".into(),
                        value: ParamValue::Number(1.0),
                    },
                    ParamEntry {
                        name: "sparkle".into(),
                        value: ParamValue::Number(1.0),
                    },
                ],
            }),
        ),
    ] {
        let r = apply(&mut world, envelope(&src, &token, bad));
        assert!(
            matches!(r, CommandResult::Rejected(RejectReason::SchemaViolation { .. })),
            "{label} must reject with SchemaViolation: {r:?}"
        );
        assert!(world.class_member("demo:member/x").is_none(), "{label}");
    }

    // A valid define lands; a duplicate is refused.
    assert!(apply(
        &mut world,
        envelope(&src, &token, simple_member("demo:member/x", "demo:stratum/test", 1.0))
    )
    .is_ok());
    let r = apply(
        &mut world,
        envelope(&src, &token, simple_member("demo:member/x", "demo:stratum/test", 1.0)),
    );
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::AlreadyDefined { .. })
    ));
}

#[test]
fn malformed_class_contracts_are_rejected() {
    let mut world = HostWorld::new(3);
    let (src, token) = definer("demo");
    let bad = Payload::DefineContentClass(DefineContentClass {
        name: "demo:stratum/bad".into(),
        doc: String::new(),
        params: vec![ParamSpec {
            name: "w".into(),
            kind: ParamKind::Range {
                min: 5.0,
                max: -5.0,
            },
            required: true,
        }],
    });
    let r = apply(&mut world, envelope(&src, &token, bad));
    assert!(matches!(
        r,
        CommandResult::Rejected(RejectReason::SchemaViolation { .. })
    ));
    assert!(world.content_class("demo:stratum/bad").is_none());
}

/// The vanilla geology pack applies through the door like any content pack,
/// and what the host then stores compiles into exactly the typed vanilla set
/// — the registry-to-worldgen bridge, proven end to end.
#[test]
fn vanilla_pack_applies_and_bridges_to_the_typed_set() {
    let mut world = HostWorld::new(4);
    let (src, token) = definer("dc");
    for payload in vanilla_geology_pack() {
        let r = apply(&mut world, envelope(&src, &token, payload));
        assert!(r.is_ok(), "vanilla pack define failed: {r:?}");
    }
    let compiled = geology_set_from_defs(world.content_classes(), world.class_members())
        .expect("registered defs compile");
    let typed = geology::vanilla();
    assert_eq!(compiled.members(), typed.members());
}

/// Registration order of member defines cannot change the compiled set —
/// the canonical-ordering rule holds through the whole registry path.
#[test]
fn member_registration_order_does_not_change_the_compiled_set() {
    let build = |permute: bool| {
        let mut world = HostWorld::new(5);
        let (src, token) = definer("dc");
        let mut pack = vanilla_geology_pack();
        if permute {
            // Keep class defines first (members need their classes), but
            // reverse the member defines.
            let members_start = pack
                .iter()
                .position(|p| matches!(p, Payload::DefineClassMember(_)))
                .expect("members in pack");
            pack[members_start..].reverse();
        }
        for payload in pack {
            let r = apply(&mut world, envelope(&src, &token, payload));
            assert!(r.is_ok(), "{r:?}");
        }
        geology_set_from_defs(world.content_classes(), world.class_members()).expect("compiles")
    };
    let forward = build(false);
    let reversed = build(true);
    assert_eq!(forward.members(), reversed.members());
    for m in forward.members() {
        assert_eq!(forward.member_index(&m.id), reversed.member_index(&m.id));
    }
}
