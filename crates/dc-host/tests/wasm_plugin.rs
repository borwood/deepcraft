//! The demo plugin, end to end: subscribe, define an item, build a structure,
//! react to a foreign block_changed event — all through postcard envelopes
//! over the `dc.call` ABI. Plus boundary capability enforcement (a plugin can
//! never exercise more authority than its installed token).

mod common;

use dc_api::host::block_name;
use dc_api::{
    CapabilityToken, CommandEnvelope, ConsumerId, ConsumerKind, Grant, HostWorld, Payload, Vec3i,
    payload::SetBlock,
};
use dc_host::{PluginHost, demo_script};

fn v(x: i64, y: i64, z: i64) -> Vec3i {
    Vec3i::new(x, y, z)
}

fn plugin_consumer() -> ConsumerId {
    ConsumerId::new(ConsumerKind::Plugin, "demo-builder")
}

fn editor_write(world: &mut HostWorld, pos: Vec3i, block: &str) {
    let editor = ConsumerId::new(ConsumerKind::Editor, "test-editor");
    let token = CapabilityToken::new(vec![Grant::WorldWrite { volume: None }]);
    world
        .submit(CommandEnvelope {
            id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
            source: editor,
            grant: token,
            payload: Payload::SetBlock(SetBlock {
                pos,
                block: block.into(),
            }),
            target_tick: None,
            txn: None,
        })
        .expect("editor submit");
}

#[test]
fn demo_plugin_builds_defines_and_reacts() {
    let mut host = PluginHost::load(
        &common::plugin_wasm_path(),
        HostWorld::new(5),
        plugin_consumer(),
        demo_script::demo_token(),
    )
    .expect("load plugin");

    // Session entry: submits the whole script (plus one contraband attempt
    // the boundary rejects before the world sees it).
    host.run().expect("dc_run");

    // Tick-quantized: nothing has applied yet.
    let probe = host.world_mut().block_at(v(2, 1, 2));
    assert_eq!(block_name(probe), "dc:air");

    let receipts = host.world_mut().tick();
    assert_eq!(
        receipts.len(),
        demo_script::demo_payloads().len(),
        "every script command produced a receipt; the contraband attempt \
         never reached the world"
    );
    assert!(
        receipts.iter().all(|r| r.receipt.result.is_ok()),
        "all script commands applied: {receipts:#?}"
    );

    // The hut is standing.
    assert_eq!(
        block_name(host.world_mut().block_at(v(2, 1, 2))),
        "dc:stone"
    );
    assert_eq!(
        block_name(host.world_mut().block_at(v(6, 1, 6))),
        "dc:stone"
    );
    assert_eq!(block_name(host.world_mut().block_at(v(2, 2, 2))), "dc:wood");
    assert_eq!(block_name(host.world_mut().block_at(v(6, 4, 5))), "dc:wood");
    // Doorway punched out.
    assert_eq!(block_name(host.world_mut().block_at(v(4, 2, 2))), "dc:air");
    assert_eq!(block_name(host.world_mut().block_at(v(4, 3, 2))), "dc:air");
    // Item authored, in the plugin's namespace only.
    assert!(host.world().item("demo:builder_wand").is_some());
    assert!(
        host.world().item("bar:contraband").is_none(),
        "grant escalation must be stopped at the boundary"
    );
    // Deer spawned.
    assert_eq!(host.world().entities().len(), 1);
    assert_eq!(host.world().entities()[0].kind, "dc:deer");

    // Plugin observes the tick: drains receipts (learning its subscription
    // id) and polls events — all of which are its own, so no reaction.
    host.notify_tick().expect("dc_tick");
    let receipts = host.world_mut().tick();
    assert!(
        receipts.is_empty(),
        "no foreign events => no reaction: {receipts:#?}"
    );

    // A foreign write inside the yard triggers the reaction: the plugin caps
    // it with wood one voxel above.
    editor_write(host.world_mut(), v(4, 1, 8), "dc:grass");
    host.world_mut().tick();
    host.notify_tick().expect("dc_tick reaction");
    let receipts = host.world_mut().tick();
    assert_eq!(receipts.len(), 1, "exactly one reaction: {receipts:#?}");
    assert_eq!(receipts[0].source, plugin_consumer());
    assert_eq!(block_name(host.world_mut().block_at(v(4, 2, 8))), "dc:wood");

    // The reaction's own event is self-caused: no feedback loop.
    host.notify_tick().expect("dc_tick quiescent");
    assert!(host.world_mut().tick().is_empty());
}

/// Installing a narrower token than the plugin's manifest disables exactly
/// the escalated commands: define_item claims registry.define(demo), which
/// the boundary rejects, while the build commands still apply.
#[test]
fn narrower_install_disables_only_the_uncovered_commands() {
    let installed = CapabilityToken::new(vec![
        Grant::WorldRead {
            volume: Some(demo_script::build_volume()),
        },
        Grant::WorldWrite {
            volume: Some(demo_script::build_volume()),
        },
        Grant::EntitySpawn,
        Grant::EventsSubscribe,
        // No RegistryDefine at all.
    ]);
    let mut host = PluginHost::load(
        &common::plugin_wasm_path(),
        HostWorld::new(5),
        plugin_consumer(),
        installed,
    )
    .expect("load plugin");
    host.run().expect("dc_run");
    let receipts = host.world_mut().tick();
    // One command (define_item) was boundary-rejected and never queued.
    assert_eq!(receipts.len(), demo_script::demo_payloads().len() - 1);
    assert!(receipts.iter().all(|r| r.receipt.result.is_ok()));
    assert!(host.world().item("demo:builder_wand").is_none());
    // The hut still went up.
    assert_eq!(
        block_name(host.world_mut().block_at(v(2, 1, 2))),
        "dc:stone"
    );
}
