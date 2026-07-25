//! The one command/query surface. Three consumers, one contract:
//!
//! 1. **WASM plugins** (wasmtime host, crates/dc-host) — sandboxed,
//!    language-agnostic.
//! 2. **MCP** (in-process rmcp server, crates/dc-mcp-dev) — agents drive the
//!    same API; the tool list is generated from [`schema::registry`].
//! 3. **In-game editors** (later) — UI over the same commands.
//!
//! Commands and queries are serializable data (serde), not trait calls, so the
//! same envelope crosses the WASM boundary (postcard), the MCP boundary
//! (JSON), and later the network boundary unchanged. Capability-scoped, deny
//! by default: a consumer holds explicit grants, not ambient authority.
//!
//! Module map:
//! - [`envelope`] — CommandEnvelope / CommandReceipt / QueryReceipt wire shape
//! - [`payload`] — typed payloads + the `Payload` union
//! - [`capability`] — grants, tokens, attenuation, requirements
//! - [`character`] — the character primitive: a persistent named body stepped
//!   on the host tick, driven by controller-verb commands (API.md § Characters)
//! - [`event`] — event kinds and delivery records
//! - [`schema`] — the machine-readable command registry (consumers generate
//!   from this; the MCP tool list is never hand-written)
//! - [`identify`] — `identify(pos)`: the honest, untiered per-voxel identity
//!   answer (a uniform mixture, or `Unrecorded`)
//! - [`host`] — the reference in-process world implementing the surface's
//!   semantics (tick quantization, total order, txn atomicity, enforcement)
//! - [`abi`] — the WASM-boundary request/response shapes (postcard)

pub mod abi;
pub mod bodies;
pub mod capability;
pub mod character;
pub mod classes;
pub mod envelope;
pub mod event;
pub mod host;
pub mod identify;
pub mod payload;
pub mod schema;

pub use bodies::{
    AnimClip, AnimClipDef, AnimSlot, BodyPlan, BodyPlanDef, JointRot, Keyframe, SegmentDef,
};
pub use capability::{CapabilityToken, Grant, Requirement};
pub use character::{CharacterConfig, CharacterInput, CharacterState, Posture};
pub use classes::{ClassMemberDef, ContentClassDef, ParamEntry, ParamKind, ParamSpec, ParamValue};
pub use envelope::{
    BlockChange, CommandEnvelope, CommandReceipt, CommandResult, ConsumerId, ConsumerKind, Effects,
    EffectsSummary, QueryReceipt, QueryResult, ReceiptEntry, RejectReason, SubmitAck, Tick, TxnId,
};
pub use event::{EventKind, GameEvent};
pub use identify::Identity;
pub use host::{
    ChunkGenerator, ChunkResidency, ContentsSource, HostWorld, block_from_name, block_name,
};
pub use payload::{EntityInfo, Payload, QueryData, Vec3f, Vec3i, Volume, ids};
