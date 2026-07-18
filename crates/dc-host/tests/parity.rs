//! The S5 core proof: the same command sequence executed (a) natively,
//! (b) via the WASM plugin, (c) via the MCP tool layer produces identical
//! world state (region hash) and identical receipts modulo source/seq —
//! consumers are indistinguishable at the API layer. Plus wasm-level replay
//! determinism.

mod common;

use std::sync::{Arc, Mutex};

use dc_api::schema::mcp_tool_name;
use dc_api::{
    CommandEnvelope, CommandReceipt, ConsumerId, ConsumerKind, HostWorld, ReceiptEntry, Vec3i,
    Volume,
};
use dc_host::{PluginHost, demo_script};
use dc_mcp_dev::ToolLayer;

/// Hash a region generously containing the build yard plus surrounding
/// terrain (so lazily generated chunks are compared too).
fn world_hash(world: &mut HostWorld) -> u64 {
    world.region_hash(Volume::new(Vec3i::new(-8, -8, -8), Vec3i::new(18, 18, 18)))
}

/// Receipts modulo source (and modulo nothing else: seq, consumer_seq,
/// tick_applied, and effects must all coincide when tick boundaries match).
fn normalize(log: &[ReceiptEntry]) -> Vec<(String, u64, CommandReceipt)> {
    log.iter()
        .map(|e| (e.command_id.clone(), e.consumer_seq, e.receipt.clone()))
        .collect()
}

const SEED: u64 = 99;

/// (a) Native: typed envelopes straight into the host world.
fn run_native() -> HostWorld {
    let mut world = HostWorld::new(SEED);
    let source = ConsumerId::new(ConsumerKind::Plugin, "native-runner");
    for payload in demo_script::demo_payloads() {
        world
            .submit(CommandEnvelope {
                id: payload.command_id().to_string(),
                source: source.clone(),
                grant: demo_script::demo_token(),
                payload,
                target_tick: None,
                txn: None,
            })
            .expect("native submit");
    }
    world.tick();
    world
}

/// (b) WASM: the demo plugin submits the same script through postcard
/// envelopes over the `dc.call` ABI.
fn run_wasm() -> PluginHost {
    let mut host = PluginHost::load(
        &common::plugin_wasm_path(),
        HostWorld::new(SEED),
        ConsumerId::new(ConsumerKind::Plugin, "demo-builder"),
        demo_script::demo_token(),
    )
    .expect("load plugin");
    host.run().expect("dc_run");
    host.world_mut().tick();
    host
}

/// (c) MCP: the same script as JSON tool calls through the generated tool
/// layer (auto_tick off so the tick boundary matches the other two paths).
fn run_mcp() -> Arc<Mutex<HostWorld>> {
    let world = Arc::new(Mutex::new(HostWorld::new(SEED)));
    let layer = ToolLayer {
        world: world.clone(),
        consumer: ConsumerId::new(ConsumerKind::McpSession, "parity-session"),
        session_token: demo_script::demo_token(),
        auto_tick: false,
    };
    for payload in demo_script::demo_payloads() {
        let tool = mcp_tool_name(payload.command_id());
        // The tool boundary carries the inner payload object of the
        // externally-tagged Payload enum.
        let outer = serde_json::to_value(&payload).expect("serialize payload");
        let arguments = outer
            .as_object()
            .and_then(|m| m.values().next())
            .expect("externally tagged payload")
            .clone();
        let ack = layer.call(&tool, &arguments).expect("mcp tool call");
        assert!(
            ack.get("consumer_seq").is_some(),
            "expected a SubmitAck, got {ack}"
        );
    }
    world.lock().unwrap().tick();
    world
}

#[test]
fn native_wasm_and_mcp_consumers_are_indistinguishable() {
    let mut native = run_native();
    let mut wasm = run_wasm();
    let mcp = run_mcp();
    let mut mcp_world = mcp.lock().unwrap();

    // Identical world state.
    let native_hash = world_hash(&mut native);
    assert_eq!(native_hash, world_hash(wasm.world_mut()), "wasm == native");
    assert_eq!(native_hash, world_hash(&mut mcp_world), "mcp == native");

    // Identical entities and registry contents.
    assert_eq!(native.entities(), wasm.world().entities());
    assert_eq!(native.entities(), mcp_world.entities());
    for w in [&native, wasm.world(), &mcp_world] {
        assert!(w.item("demo:builder_wand").is_some());
    }

    // Identical receipts modulo source.
    let native_receipts = normalize(native.receipt_log());
    assert_eq!(
        native_receipts,
        normalize(wasm.world().receipt_log()),
        "wasm receipts == native receipts (modulo source)"
    );
    assert_eq!(
        native_receipts,
        normalize(mcp_world.receipt_log()),
        "mcp receipts == native receipts (modulo source)"
    );
}

/// Replay determinism at the wasm level: the same plugin, seed, and command
/// log produce a bit-identical world (docs/API.md principle 3).
#[test]
fn wasm_replay_is_deterministic() {
    let mut first = run_wasm();
    let mut second = run_wasm();
    assert_eq!(
        world_hash(first.world_mut()),
        world_hash(second.world_mut())
    );
    assert_eq!(first.world().receipt_log(), second.world().receipt_log());

    // And a different seed produces different terrain.
    let mut other = PluginHost::load(
        &common::plugin_wasm_path(),
        HostWorld::new(SEED + 1),
        ConsumerId::new(ConsumerKind::Plugin, "demo-builder"),
        demo_script::demo_token(),
    )
    .expect("load plugin");
    other.run().expect("dc_run");
    other.world_mut().tick();
    assert_ne!(world_hash(first.world_mut()), world_hash(other.world_mut()));
}
