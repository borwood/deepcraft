//! The envelope/receipt wire shape (docs/API.md § Envelope).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::capability::CapabilityToken;
use crate::payload::{Payload, QueryData, Vec3i};

/// Simulation tick counter.
pub type Tick = u64;

/// Atomic-batch membership: all commands sharing (source, txn) in one tick
/// apply all-or-nothing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TxnId(pub u64);

/// What kind of consumer a command comes from; determines its priority class.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ConsumerKind {
    Player,
    Editor,
    Plugin,
    McpSession,
    Scheduler,
}

impl ConsumerKind {
    /// Priority class within a tick (lower applies first). Player input is
    /// highest; editors/plugins/MCP share a class; scheduled jobs run last.
    pub fn priority_class(self) -> u8 {
        match self {
            ConsumerKind::Player => 0,
            ConsumerKind::Editor | ConsumerKind::Plugin | ConsumerKind::McpSession => 1,
            ConsumerKind::Scheduler => 2,
        }
    }
}

/// A consumer identity: (kind, name). Total command order tie-breaks on this
/// after priority class, so the order is defined even across consumers in the
/// same class (API.md says "(tick, priority class, per-consumer seq)"; the
/// consumer id is the necessary disambiguator between consumers — see S5
/// results § proposed revisions).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ConsumerId {
    pub kind: ConsumerKind,
    pub name: String,
}

impl ConsumerId {
    pub fn new(kind: ConsumerKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }
}

/// The one command shape that crosses every boundary (in-process, WASM, MCP,
/// later the network) unchanged.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CommandEnvelope {
    /// Namespaced command id, e.g. `dc:world/set_block`. Must agree with the
    /// payload variant.
    pub id: String,
    pub source: ConsumerId,
    pub grant: CapabilityToken,
    pub payload: Payload,
    /// Tick the command should apply at; `None` = next tick boundary.
    /// A target at or before the current completed tick is rejected.
    #[serde(default)]
    pub target_tick: Option<Tick>,
    #[serde(default)]
    pub txn: Option<TxnId>,
}

/// One applied block mutation, echoed in effects.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BlockChange {
    pub pos: Vec3i,
    pub from: String,
    pub to: String,
}

/// Everything a command did. v0 receipts echo the FULL effect list; whether
/// that survives is API.md open question 4 — S5 measures it (see
/// [`Effects::summary`] and the S5 results doc).
///
/// NOTE: no `skip_serializing_if` here (or anywhere on wire types) — postcard
/// is not self-describing, so conditional field skipping corrupts the WASM
/// boundary encoding. Wire types serialize every field, always.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Effects {
    #[serde(default)]
    pub blocks_changed: Vec<BlockChange>,
    #[serde(default)]
    pub entities_spawned: Vec<u64>,
    #[serde(default)]
    pub items_defined: Vec<String>,
    #[serde(default)]
    pub subscriptions_created: Vec<u64>,
    #[serde(default)]
    pub characters_spawned: Vec<String>,
    // Appended (content-class registry milestone) — postcard field order is
    // wire identity, so new fields go at the end, always serialized.
    #[serde(default)]
    pub classes_defined: Vec<String>,
    #[serde(default)]
    pub class_members_defined: Vec<String>,
    // Appended (body-plan staircase steps 1–2) — postcard field order is wire
    // identity, so new fields go at the end, always serialized.
    #[serde(default)]
    pub body_plans_defined: Vec<String>,
    #[serde(default)]
    pub anim_clips_defined: Vec<String>,
}

impl Effects {
    pub fn summary(&self) -> EffectsSummary {
        EffectsSummary {
            blocks_changed: self.blocks_changed.len() as u64,
            entities_spawned: self.entities_spawned.len() as u64,
            items_defined: self.items_defined.len() as u64,
            subscriptions_created: self.subscriptions_created.len() as u64,
            characters_spawned: self.characters_spawned.len() as u64,
            classes_defined: self.classes_defined.len() as u64,
            class_members_defined: self.class_members_defined.len() as u64,
            body_plans_defined: self.body_plans_defined.len() as u64,
            anim_clips_defined: self.anim_clips_defined.len() as u64,
        }
    }
}

/// Count-only echo of [`Effects`], for the receipt-verbosity measurement.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct EffectsSummary {
    pub blocks_changed: u64,
    pub entities_spawned: u64,
    pub items_defined: u64,
    pub subscriptions_created: u64,
    pub characters_spawned: u64,
    pub classes_defined: u64,
    pub class_members_defined: u64,
    pub body_plans_defined: u64,
    pub anim_clips_defined: u64,
}

/// Why a command (or query) was refused.
#[derive(Clone, PartialEq, Debug, Error, Serialize, Deserialize)]
pub enum RejectReason {
    #[error("unknown command id `{id}`")]
    UnknownCommand { id: String },
    #[error("payload does not match command id `{id}` (payload is for `{payload_id}`)")]
    PayloadMismatch { id: String, payload_id: String },
    #[error("`{id}` is a query; submit it through the query path")]
    NotACommand { id: String },
    #[error("`{id}` is a command; queries cannot mutate")]
    NotAQuery { id: String },
    #[error("target tick {target} is not after current tick {current}")]
    TargetTickInPast { target: Tick, current: Tick },
    #[error("missing capability: needs {needed}")]
    MissingCapability { needed: String },
    #[error("invalid payload: {reason}")]
    PayloadInvalid { reason: String },
    #[error("unknown block name `{name}`")]
    UnknownBlock { name: String },
    #[error("`{key}` is already defined")]
    AlreadyDefined { key: String },
    #[error("region of {voxels} voxels exceeds the per-command cap of {max}")]
    RegionTooLarge { voxels: u64, max: u64 },
    #[error("unknown or foreign subscription {id}")]
    UnknownSubscription { id: u64 },
    #[error("unknown character `{name}`")]
    UnknownCharacter { name: String },
    #[error("transaction aborted: `{failed_id}` failed ({reason})")]
    TxnAborted {
        failed_id: String,
        reason: Box<RejectReason>,
    },
    // Appended (content-class registry milestone) — variant order is postcard
    // wire identity, so new reasons go at the end.
    #[error("unknown content class `{class}`")]
    UnknownClass { class: String },
    #[error("class contract violation: {reason}")]
    SchemaViolation { reason: String },
    // Appended (body-plan staircase step 3: parametric crouch) — variant order
    // is postcard wire identity, so new reasons go at the end.
    #[error("character `{character}` cannot stand: the space above is obstructed")]
    PostureBlocked { character: String },
    // Appended (per-character body plans) — variant order is postcard wire
    // identity, so new reasons go at the end.
    /// A spawn named a body plan that no pack has registered. The refusal is a
    /// receipt on purpose: silently falling back to the default would render a
    /// body the caller did not ask for, and the caller would never learn its pack
    /// failed to load (S-5: an identity default covers the *absent* case, not the
    /// *wrong* one).
    #[error(
        "unknown body plan `{name}` — define it (dc:registry/define_body_plan) before wearing it"
    )]
    UnknownBodyPlan { name: String },
}

/// Outcome half of a receipt.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CommandResult {
    Ok(Effects),
    Rejected(RejectReason),
}

impl CommandResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, CommandResult::Ok(_))
    }
}

/// What a submitted command came back with once (tick-quantized) it applied —
/// or was rejected.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CommandReceipt {
    /// Global apply-order sequence number (monotone across the world,
    /// including rejections).
    pub seq: u64,
    /// The tick the command applied on (for submit-time rejections: the
    /// current completed tick).
    pub tick_applied: Tick,
    pub result: CommandResult,
}

/// Acknowledgement that a command passed structural checks and is queued.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SubmitAck {
    /// Per-consumer submission sequence (the envelope's position in its
    /// consumer's order).
    pub consumer_seq: u64,
    /// The tick boundary it will apply at.
    pub scheduled_tick: Tick,
}

/// Outcome half of a query.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum QueryResult {
    Ok(QueryData),
    Rejected(RejectReason),
}

impl QueryResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, QueryResult::Ok(_))
    }
}

/// Answer to a query. Queries run against the last completed tick's state and
/// never mutate world state (`dc:events/poll` drains only its own channel).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryReceipt {
    /// The completed tick the answer reflects.
    pub tick_observed: Tick,
    pub result: QueryResult,
}

/// A receipt plus who it belongs to — the host's log entry shape.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ReceiptEntry {
    pub source: ConsumerId,
    pub command_id: String,
    pub consumer_seq: u64,
    pub receipt: CommandReceipt,
}
