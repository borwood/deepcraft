//! Demo WASM plugin (S5): drives dc-api through the `dc.call` host import.
//!
//! ABI (see dc_api::abi and docs/spikes/S5-results.md):
//! - imports `dc.call(req_ptr, req_len) -> packed(ptr, len)` — postcard
//!   `PluginRequest` in, postcard `PluginResponse` out (response buffer is
//!   allocated in our memory by the host via our `dc_alloc` export and owned
//!   by us afterwards);
//! - exports `memory`, `dc_alloc`, `dc_free`, `dc_run` (session entry) and
//!   `dc_tick` (post-tick callback: drain receipts, poll events, react).
//!
//! Behavior: subscribe to BlockChanged in the build yard, define
//! `demo:builder_wand`, build a hut, spawn a deer (the shared demo script),
//! then attempt a namespace-escalating `bar:contraband` definition (which the
//! host must reject at the boundary), and on every tick react to *foreign*
//! block changes in the yard by capping them with a wood block.

use std::sync::Mutex;

use dc_api::abi::{self, PluginRequest, PluginResponse};
use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, GameEvent, Payload,
    QueryData, QueryResult, Vec3i,
};

/// Shared with dc-host (single source of truth for the parity proof).
#[allow(dead_code)] // the plugin uses a subset of the shared helpers
mod script {
    include!("../../../crates/dc-host/src/demo_script.rs");
}

#[link(wasm_import_module = "dc")]
unsafe extern "C" {
    fn call(req_ptr: *const u8, req_len: u32) -> u64;
}

// ------------------------------------------------------------- allocation --

/// Allocate `len` bytes the host can write a response into. Exact-capacity
/// (`vec![0; len]`) so `dc_free`/response handling can reconstruct the Vec.
#[unsafe(no_mangle)]
pub extern "C" fn dc_alloc(len: u32) -> u32 {
    let mut buf = vec![0u8; len as usize];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as u32
}

/// Free a buffer previously handed out by `dc_alloc` (if the guest chooses
/// not to consume it in place).
///
/// # Safety
/// `ptr`/`len` must come from a `dc_alloc(len)` call whose buffer has not
/// already been freed or consumed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dc_free(ptr: u32, len: u32) {
    drop(unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize) });
}

// ------------------------------------------------------------ host bridge --

fn host_call(req: &PluginRequest) -> Option<PluginResponse> {
    let bytes = abi::encode(req);
    let packed = unsafe { call(bytes.as_ptr(), bytes.len() as u32) };
    let (ptr, len) = abi::unpack_ptr_len(packed);
    if ptr == 0 {
        return None;
    }
    // Take ownership of the response buffer the host wrote via dc_alloc.
    let resp = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize) };
    abi::decode(&resp).ok()
}

fn me() -> ConsumerId {
    ConsumerId::new(ConsumerKind::Plugin, "demo-builder")
}

fn envelope(payload: Payload, grant: CapabilityToken) -> CommandEnvelope {
    CommandEnvelope {
        id: payload.command_id().to_string(),
        source: me(),
        grant,
        payload,
        target_tick: None,
        txn: None,
    }
}

/// Submit with least authority: claim only the grants this payload needs.
fn submit(payload: Payload) -> Option<PluginResponse> {
    let grant = CapabilityToken::new(script::minimal_grants_for(&payload));
    host_call(&PluginRequest::Submit(envelope(payload, grant)))
}

fn query(payload: Payload) -> Option<QueryResult> {
    let grant = CapabilityToken::new(script::minimal_grants_for(&payload));
    match host_call(&PluginRequest::Query(envelope(payload, grant)))? {
        PluginResponse::Query(receipt) => Some(receipt.result),
        _ => None,
    }
}

// ------------------------------------------------------------------ state --

static SUBSCRIPTION: Mutex<Option<u64>> = Mutex::new(None);

// -------------------------------------------------------------- lifecycle --

/// Session entry: run the shared demo script, then attempt the namespace
/// escalation the host is expected to block.
#[unsafe(no_mangle)]
pub extern "C" fn dc_run() {
    for payload in script::demo_payloads() {
        submit(payload);
    }
    // Escalation attempt: claim `registry.define(bar)`, which our installed
    // token does not hold. The host boundary must reject this before the
    // world ever sees it.
    submit(Payload::DefineItem(dc_api::payload::DefineItem {
        name: "bar:contraband".into(),
        display_name: "Contraband".into(),
        description: None,
    }));
}

/// Post-tick callback: learn our subscription id from receipts, then poll
/// events and cap every foreign non-air block change with wood.
#[unsafe(no_mangle)]
pub extern "C" fn dc_tick() {
    let mut sub_slot = SUBSCRIPTION.lock().expect("single-threaded");
    if sub_slot.is_none()
        && let Some(PluginResponse::Receipts(entries)) = host_call(&PluginRequest::DrainReceipts)
    {
        for entry in entries {
            if let CommandResult::Ok(effects) = &entry.receipt.result
                && let Some(id) = effects.subscriptions_created.first()
            {
                *sub_slot = Some(*id);
            }
        }
    }
    let Some(sub) = *sub_slot else { return };
    drop(sub_slot);

    let Some(QueryResult::Ok(QueryData::Events { events, .. })) =
        query(Payload::EventsPoll(dc_api::payload::EventsPoll {
            subscription: sub,
            max: None,
        }))
    else {
        return;
    };
    for event in events {
        if let GameEvent::BlockChanged { pos, to, cause, .. } = event
            && cause != me()
            && to != "dc:air"
        {
            submit(Payload::SetBlock(dc_api::payload::SetBlock {
                pos: Vec3i::new(pos.x, pos.y + 1, pos.z),
                block: "dc:wood".into(),
            }));
        }
    }
}
