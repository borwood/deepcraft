//! The WASM plugin ABI — request/response shapes carried across the guest
//! boundary, postcard-encoded (docs/API.md § Wire formats: "WASM boundary:
//! compact serde (postcard)").
//!
//! Call convention (documented in full in docs/spikes/S5-results.md):
//!
//! - Guest exports: `memory`, `dc_alloc(len: u32) -> ptr: u32`,
//!   `dc_free(ptr: u32, len: u32)`, `dc_run()` (demo entry),
//!   `dc_tick()` (called by the host after each tick so the plugin can poll
//!   events and react).
//! - Host import (module `"dc"`): `call(req_ptr: u32, req_len: u32) -> u64`.
//!   The request bytes are a postcard [`PluginRequest`]; the host writes a
//!   postcard [`PluginResponse`] into guest memory it obtains from
//!   `dc_alloc` and returns `(ptr << 32) | len`. The guest owns the response
//!   buffer and frees it with `dc_free`.
//! - The host validates the envelope's *claimed* grant against the plugin's
//!   installed token (`CapabilityToken::covers_token`) and rejects
//!   escalations before the world ever sees the envelope.

use serde::{Deserialize, Serialize};

use crate::envelope::{CommandEnvelope, CommandReceipt, QueryReceipt, ReceiptEntry, SubmitAck};

/// What a plugin can ask of the host.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PluginRequest {
    /// Queue a command for the next tick boundary.
    Submit(CommandEnvelope),
    /// Run a query against the last completed tick.
    Query(CommandEnvelope),
    /// Fetch this plugin's receipts produced since the last drain. Receipts
    /// are tick-boundary artifacts, so a submitting plugin sees them
    /// asynchronously — typically from its `dc_tick` callback.
    DrainReceipts,
}

/// What comes back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PluginResponse {
    /// Command accepted and queued; the receipt is produced at the tick
    /// boundary (host-side receipt log).
    Submitted(SubmitAck),
    /// Command rejected before queueing (structural, or grant escalation at
    /// the boundary).
    SubmitRejected(CommandReceipt),
    /// Query answer.
    Query(QueryReceipt),
    /// Receipts for this plugin since the last drain, in apply order.
    Receipts(Vec<ReceiptEntry>),
    /// The host could not parse or process the request itself.
    Error(String),
}

/// Encode helper (postcard).
pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).expect("postcard encoding of ABI types cannot fail")
}

/// Decode helper (postcard).
pub fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, String> {
    postcard::from_bytes(bytes).map_err(|e| e.to_string())
}

/// Pack a guest (ptr, len) pair into the u64 return of `dc.call`.
pub fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | len as u64
}

/// Unpack the u64 return of `dc.call`.
pub fn unpack_ptr_len(packed: u64) -> (u32, u32) {
    ((packed >> 32) as u32, packed as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Postcard is not self-describing: every wire type must round-trip
    /// exactly (this guards against e.g. `skip_serializing_if` sneaking onto
    /// an envelope/receipt type and silently corrupting the WASM boundary).
    #[test]
    fn postcard_roundtrips_receipts_with_sparse_effects() {
        use crate::envelope::{
            CommandReceipt, CommandResult, ConsumerId, ConsumerKind, Effects, ReceiptEntry,
        };
        let entry = ReceiptEntry {
            source: ConsumerId::new(ConsumerKind::Plugin, "p"),
            command_id: "dc:events/subscribe".into(),
            consumer_seq: 0,
            receipt: CommandReceipt {
                seq: 1,
                tick_applied: 1,
                result: CommandResult::Ok(Effects {
                    subscriptions_created: vec![1],
                    ..Effects::default()
                }),
            },
        };
        let response = PluginResponse::Receipts(vec![entry]);
        let decoded: PluginResponse = decode(&encode(&response)).expect("round-trip");
        assert_eq!(decoded, response);
    }

    #[test]
    fn ptr_len_roundtrip() {
        assert_eq!(
            unpack_ptr_len(pack_ptr_len(0xDEAD_BEEF, 42)),
            (0xDEAD_BEEF, 42)
        );
        assert_eq!(unpack_ptr_len(pack_ptr_len(0, 0)), (0, 0));
        assert_eq!(
            unpack_ptr_len(pack_ptr_len(u32::MAX, u32::MAX)),
            (u32::MAX, u32::MAX)
        );
    }
}
