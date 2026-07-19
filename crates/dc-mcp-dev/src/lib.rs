//! `dc-mcp-dev`: the dev-surface MCP server over dc-api (docs/API.md § two
//! MCP surfaces).
//!
//! The tool list is **generated** from `dc_api::schema::registry()` — there is
//! no hand-written tool anywhere in this crate; adding a command to the
//! registry adds the MCP tool. Tool names are the registry ids with the `dc:`
//! prefix dropped and `/` mapped to `_` (`dc:world/set_block` →
//! `world_set_block`) because MCP tool names cannot contain `:` or `/`.
//!
//! Trust boundary: an MCP session holds a session-ambient
//! [`CapabilityToken`]; tool callers never pass grants. Every envelope this
//! layer builds carries the session token, so capability enforcement happens
//! in the world exactly as for every other consumer.
//!
//! Tick model: `auto_tick = true` (the dev default) completes a tick after
//! each submitted command and returns the command's receipt — interactive
//! dev-tool ergonomics. `auto_tick = false` returns the [`SubmitAck`] and
//! leaves tick driving to whoever owns the world (used by the parity proof to
//! keep tick boundaries identical across consumers).

use std::sync::{Arc, Mutex};

use dc_api::schema::{self, CommandKind};
use dc_api::{CapabilityToken, CommandEnvelope, ConsumerId, HostWorld};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolCallError {
    #[error("unknown tool `{0}`")]
    UnknownTool(String),
    #[error("bad arguments: {0}")]
    BadArguments(String),
    #[error("world lock poisoned")]
    Poisoned,
}

/// The MCP tool list generated from the schema registry — shared by every
/// MCP surface over dc-api (this dev server and the in-client server;
/// docs/API.md principle 5: the tool list is never hand-written).
pub fn registry_tools() -> Vec<Tool> {
    schema::registry()
        .iter()
        .map(|spec| {
            let schema_obj = match (spec.payload_schema)() {
                Value::Object(map) => map,
                other => panic!("payload schema for {} is not an object: {other}", spec.id),
            };
            let description = format!(
                "{} [{}] (id: {}; requires {})",
                spec.doc,
                match spec.kind {
                    CommandKind::Command => "command: applies at the next tick boundary",
                    CommandKind::Query => "query: reads the last completed tick",
                },
                spec.id,
                spec.capability,
            );
            Tool::new(schema::mcp_tool_name(spec.id), description, schema_obj)
        })
        .collect()
}

/// Decode one MCP tool call into the typed [`CommandEnvelope`] it denotes,
/// stamped with the session's ambient identity and token (tool callers never
/// pass grants). Registry-driven: the tool name maps back to the command id
/// and the registry's `decode_json` builds the payload — there is no per-tool
/// code anywhere. Extracted from `ToolLayer::call` by the client-through-dc-api
/// milestone so the in-client MCP server shares it instead of duplicating it.
pub fn envelope_for_tool_call(
    tool_name: &str,
    arguments: &Value,
    consumer: &ConsumerId,
    session_token: &CapabilityToken,
) -> Result<(&'static dc_api::schema::CommandSpec, CommandEnvelope), ToolCallError> {
    let id = schema::id_for_mcp_tool(tool_name)
        .ok_or_else(|| ToolCallError::UnknownTool(tool_name.to_string()))?;
    let spec = schema::spec(id).expect("id came from the registry");
    let payload = (spec.decode_json)(arguments).map_err(ToolCallError::BadArguments)?;
    let envelope = CommandEnvelope {
        id: id.to_string(),
        source: consumer.clone(),
        grant: session_token.clone(),
        payload,
        target_tick: None,
        txn: None,
    };
    Ok((spec, envelope))
}

/// The generated tool layer: registry-driven dispatch onto a shared world.
pub struct ToolLayer {
    pub world: Arc<Mutex<HostWorld>>,
    pub consumer: ConsumerId,
    pub session_token: CapabilityToken,
    pub auto_tick: bool,
}

impl ToolLayer {
    /// The MCP tool list, generated from the schema registry.
    pub fn tools() -> Vec<Tool> {
        registry_tools()
    }

    /// Dispatch one tool call. Returns the JSON the MCP client sees:
    /// - queries → the [`dc_api::QueryReceipt`];
    /// - commands with `auto_tick` → the command's [`dc_api::CommandReceipt`];
    /// - commands without `auto_tick` → the [`dc_api::SubmitAck`]
    ///   (receipts arrive when the world owner ticks);
    /// - submit-time rejections → the rejection [`dc_api::CommandReceipt`].
    pub fn call(&self, tool_name: &str, arguments: &Value) -> Result<Value, ToolCallError> {
        let (spec, envelope) =
            envelope_for_tool_call(tool_name, arguments, &self.consumer, &self.session_token)?;
        let mut world = self.world.lock().map_err(|_| ToolCallError::Poisoned)?;
        let response = match spec.kind {
            CommandKind::Query => serde_json::to_value(world.query(&envelope)),
            CommandKind::Command => match world.submit(envelope) {
                Err(entry) => serde_json::to_value(&entry.receipt),
                Ok(ack) if !self.auto_tick => serde_json::to_value(ack),
                Ok(ack) => {
                    let receipts = world.tick();
                    let mine = receipts
                        .iter()
                        .find(|e| e.source == self.consumer && e.consumer_seq == ack.consumer_seq)
                        .expect("auto-ticked command must produce a receipt");
                    serde_json::to_value(&mine.receipt)
                }
            },
        };
        response.map_err(|e| ToolCallError::BadArguments(e.to_string()))
    }
}

/// The rmcp server: a thin async skin over [`ToolLayer`].
#[derive(Clone)]
pub struct DevMcpServer {
    layer: Arc<ToolLayer>,
}

impl DevMcpServer {
    pub fn new(layer: ToolLayer) -> Self {
        Self {
            layer: Arc::new(layer),
        }
    }
}

impl ServerHandler for DevMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "deepcraft dev surface: every tool is a dc-api command or query \
             (generated from the schema registry). Commands apply at tick \
             boundaries and return receipts; queries read the last completed \
             tick.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(ToolLayer::tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        match self.layer.call(&request.name, &arguments) {
            Ok(value) => Ok(CallToolResult::structured(value)),
            Err(ToolCallError::UnknownTool(name)) => Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("unknown tool `{name}`"),
                None,
            )),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(
                e.to_string(),
            )])),
        }
    }
}

/// The broad dev-session token: unbounded world read/write, entity spawn,
/// event subscription, and `dev:*` item authorship.
pub fn dev_session_token() -> CapabilityToken {
    CapabilityToken::new(vec![
        dc_api::Grant::WorldRead { volume: None },
        dc_api::Grant::WorldWrite { volume: None },
        dc_api::Grant::EntitySpawn,
        dc_api::Grant::EventsSubscribe,
        dc_api::Grant::RegistryDefine {
            namespace: "dev".into(),
        },
    ])
}
