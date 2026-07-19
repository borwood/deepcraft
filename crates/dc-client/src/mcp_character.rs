//! The EMBODIED character MCP surface (docs/API.md § two MCP surfaces):
//! streamable HTTP on `127.0.0.1:7778` (`--mcp-character-port`,
//! `--no-mcp-character`), sharing the dev surface's bridge/server machinery
//! (mcp.rs) but exposing a completely different — and deliberately tiny —
//! tool list.
//!
//! Session model: each MCP session begins unbound. `character_attach`
//! spawns-or-attaches to ONE character (the spawn happens under the
//! surface's parent token on the ECS side; the session never holds spawn
//! authority), and from then on every tool call runs under a token
//! **attenuated to exactly that character** — its control verbs and its
//! senses, nothing else. There are no world tools, no registry, no
//! screenshots here: the session perceives through the character's eyes
//! (`character_sense_raycast`), near field (`character_sense_surroundings`),
//! and proprioception (`character_pose`) only.
//!
//! The tool list is still generated: it is the schema registry filtered to
//! the `dc:character/` domain (minus `spawn_character`, which is dev-grant),
//! plus the one hand-authored session tool `character_attach`. The
//! registry tools keep their `character` argument — a session must name its
//! own character, and naming any other fails capability enforcement in the
//! host (the token is the cage, not the tool list; see the session test).

use std::sync::{Arc, Mutex};

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};

use crate::mcp::BridgeRequest;

/// Registry-generated character tools: the `dc:character/` domain minus the
/// dev-grant spawn (sessions spawn through `character_attach` only).
pub fn character_registry_tools() -> Vec<Tool> {
    dc_mcp_dev::registry_tools()
        .into_iter()
        .filter(|tool| {
            tool.name.starts_with("character_") && tool.name != "character_spawn_character"
        })
        .collect()
}

/// The one hand-authored session tool.
pub fn attach_tool() -> Tool {
    Tool::new(
        "character_attach",
        "Bind this session to ONE character: attaches to it if it exists, \
         spawns it (feet at `pos`, world meters; default just in front of \
         the player) if not. From then on this session's token covers exactly \
         that character's control and senses — every other tool call must \
         name it. One attach per session. A spawn whose body would be embedded \
         in solid terrain is refused (ok:false, code:\"obstructed\") rather \
         than creating a stuck statue; pass surface:true to drop the body onto \
         the true surface at pos's x/z first.",
        match json!({
            "type": "object",
            "properties": {
                "character": {
                    "type": "string",
                    "description": "bare slug name ([a-z0-9_-], max 64), e.g. scout",
                },
                "pos": {
                    "type": "object",
                    "description": "spawn position (feet, world meters); ignored when attaching to an existing character",
                    "properties": {
                        "x": { "type": "number" },
                        "y": { "type": "number" },
                        "z": { "type": "number" },
                    },
                    "required": ["x", "y", "z"],
                    "additionalProperties": false,
                },
                "surface": {
                    "type": "boolean",
                    "description": "drop the new body onto the true voxel surface at pos's x/z (ignoring pos.y); ignored when attaching to an existing character",
                },
            },
            "required": ["character"],
            "additionalProperties": false,
        }) {
            Value::Object(map) => map,
            _ => unreachable!(),
        },
    )
}

/// Fires freeze-on-disconnect when a session ends. Held by `Arc` inside the
/// per-session handler and shared across every clone rmcp makes to serve a
/// request, so its `Drop` runs EXACTLY once — when the last handle to the
/// session drops (session close / timeout / shutdown), never on a transient
/// per-call clone. On drop, if the session had attached to a character, it
/// zeroes that character's move intent (API.md § Characters, DECIDED
/// 2026-07-19): the abandoned body stands where it was left instead of walking
/// on forever under its last intent.
struct SessionEnd {
    tx: mpsc::UnboundedSender<BridgeRequest>,
    attached: Arc<Mutex<Option<String>>>,
}

impl Drop for SessionEnd {
    fn drop(&mut self) {
        let name = self.attached.lock().ok().and_then(|guard| guard.clone());
        if let Some(name) = name {
            // Fire-and-forget: a closed bridge (client already shutting down)
            // simply means nothing left to freeze.
            let _ = self.tx.send(BridgeRequest::CharacterFreeze { name });
        }
    }
}

/// Per-session handler: created fresh per MCP session (the service factory),
/// so `attached` is session state — exactly the "session spawns-or-attaches
/// to one character" contract.
#[derive(Clone)]
pub struct CharacterMcpServer {
    tx: mpsc::UnboundedSender<BridgeRequest>,
    attached: Arc<Mutex<Option<String>>>,
    /// Session-teardown guard (freeze-on-disconnect). `Arc` so it drops once,
    /// when the session — not a per-call clone — ends.
    _session_end: Arc<SessionEnd>,
}

impl CharacterMcpServer {
    pub fn new(tx: mpsc::UnboundedSender<BridgeRequest>) -> Self {
        let attached = Arc::new(Mutex::new(None));
        let session_end = Arc::new(SessionEnd {
            tx: tx.clone(),
            attached: attached.clone(),
        });
        Self {
            tx,
            attached,
            _session_end: session_end,
        }
    }

    fn attached(&self) -> Option<String> {
        self.attached.lock().ok().and_then(|guard| guard.clone())
    }

    async fn bridge(&self, request: BridgeRequest, rx: oneshot::Receiver<Value>) -> CallToolResult {
        if self.tx.send(request).is_err() {
            return CallToolResult::error(vec![ContentBlock::text("client is shutting down")]);
        }
        match rx.await {
            Ok(value) => CallToolResult::structured(value),
            Err(_) => CallToolResult::error(vec![ContentBlock::text(
                "request dropped by the client (world reset or shutdown)",
            )]),
        }
    }
}

impl ServerHandler for CharacterMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "deepcraft character surface: an EMBODIED session. Call \
             character_attach once to spawn-or-attach to one character; the \
             session is then attenuated to exactly that character — drive its \
             body with character_set_move_intent / character_set_look / \
             character_jump (commands, applied at the game's tick boundaries, \
             moving a real collision-checked body the player can see) and \
             perceive ONLY through its senses: character_pose \
             (proprioception), character_sense_raycast (its gaze), \
             character_sense_surroundings (near blocks, radius <= 16). There \
             are no world tools or screenshots on this surface.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = vec![attach_tool()];
        tools.extend(character_registry_tools());
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = Value::Object(request.arguments.unwrap_or_default());
        if request.name == "character_attach" {
            let Some(name) = args.get("character").and_then(Value::as_str) else {
                return Ok(CallToolResult::error(vec![ContentBlock::text(
                    "`character` (string) is required",
                )]));
            };
            if let Some(current) = self.attached() {
                return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "session already attached to `{current}`"
                ))]));
            }
            let pos = match args.get("pos") {
                None | Some(Value::Null) => None,
                Some(p) => {
                    let component = |k: &str| p.get(k).and_then(Value::as_f64);
                    match (component("x"), component("y"), component("z")) {
                        (Some(x), Some(y), Some(z)) => Some([x, y, z]),
                        _ => {
                            return Ok(CallToolResult::error(vec![ContentBlock::text(
                                "pos requires numeric x, y, z",
                            )]));
                        }
                    }
                }
            };
            let surface = args
                .get("surface")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let (reply, rx) = oneshot::channel();
            let result = self
                .bridge(
                    BridgeRequest::CharacterAttach {
                        name: name.to_string(),
                        pos,
                        surface,
                        reply,
                    },
                    rx,
                )
                .await;
            // Bind the session only when the world said yes.
            if let Some(content) = &result.structured_content
                && content.get("ok").and_then(Value::as_bool) == Some(true)
                && let Ok(mut guard) = self.attached.lock()
            {
                *guard = Some(name.to_string());
            }
            return Ok(result);
        }

        // Registry character tools, under the session's attenuated identity.
        let is_character_tool = character_registry_tools()
            .iter()
            .any(|tool| tool.name == request.name);
        if !is_character_tool {
            return Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("unknown tool `{}`", request.name),
                None,
            ));
        }
        let Some(session_character) = self.attached() else {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                "session not attached: call character_attach first",
            )]));
        };
        let (reply, rx) = oneshot::channel();
        Ok(self
            .bridge(
                BridgeRequest::CharacterApi {
                    tool: request.name.to_string(),
                    args,
                    session_character,
                    reply,
                },
                rx,
            )
            .await)
    }
}

/// The character surface's streamable-HTTP tower service. The factory makes
/// a FRESH handler per session — that is what makes `attached` session
/// state. `sse_extras: false` is the in-process test mode (see
/// [`crate::mcp::http_service`]).
pub fn http_service(
    tx: mpsc::UnboundedSender<BridgeRequest>,
    sse_extras: bool,
) -> StreamableHttpService<CharacterMcpServer, LocalSessionManager> {
    let mut config = StreamableHttpServerConfig::default();
    if !sse_extras {
        config.sse_keep_alive = None;
        config.sse_retry = None;
    }
    StreamableHttpService::new(
        move || Ok(CharacterMcpServer::new(tx.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Authority;
    use crate::mcp::tests::post;

    /// Freeze-on-disconnect (Decision E): dropping a session that had attached
    /// to a character emits a `CharacterFreeze` for it — the teardown `Drop`
    /// fires exactly once when the last handle to the session goes away.
    #[test]
    fn dropping_an_attached_session_emits_a_freeze() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let server = CharacterMcpServer::new(tx);
        // Stand in for a successful attach binding the session.
        *server.attached.lock().unwrap() = Some("scout".to_string());
        // A per-call clone (as rmcp makes to serve a request) must NOT freeze.
        let clone = server.clone();
        drop(clone);
        assert!(rx.try_recv().is_err(), "a per-call clone must not freeze");
        // The session ending (last handle dropped) freezes exactly once.
        drop(server);
        match rx.try_recv() {
            Ok(BridgeRequest::CharacterFreeze { name }) => assert_eq!(name, "scout"),
            Ok(_) => panic!("expected a CharacterFreeze request"),
            Err(e) => panic!("expected a freeze on session drop, got {e}"),
        }
        assert!(rx.try_recv().is_err(), "freeze fires exactly once");
    }

    /// A session that never attached freezes nothing on drop.
    #[test]
    fn dropping_an_unattached_session_is_silent() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let server = CharacterMcpServer::new(tx);
        drop(server);
        assert!(rx.try_recv().is_err(), "no attach, nothing to freeze");
    }

    /// The whole character surface, end to end over the real streamable-HTTP
    /// wire (ECS stand-in pump): the tool list is ONLY the embodied set; a
    /// session attaches to one character, drives it through real collision,
    /// senses through its body — and the attenuated token denies every
    /// out-of-scope act (the capability-enforcement item of the milestone).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn character_session_is_embodied_and_attenuated() {
        const SEED: i32 = 1337;
        let (tx, mut rx) = mpsc::unbounded_channel();

        // ECS stand-in: drain the bridge into an Authority, tick the host.
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pump_shutdown = shutdown.clone();
        let pump = std::thread::spawn(move || {
            let mut authority = Authority::new(SEED, 3);
            while !pump_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                while let Ok(request) = rx.try_recv() {
                    match request {
                        BridgeRequest::CharacterAttach {
                            name,
                            pos,
                            surface,
                            reply,
                        } => {
                            let pos = pos.expect("test always passes pos");
                            authority.handle_character_attach(
                                &name,
                                dc_api::payload::Vec3f::new(pos[0], pos[1], pos[2]),
                                surface,
                                reply,
                            );
                        }
                        BridgeRequest::CharacterApi {
                            tool,
                            args,
                            session_character,
                            reply,
                        } => {
                            authority.handle_character_api(&tool, &args, &session_character, reply);
                        }
                        _ => panic!("character pump serves only character requests"),
                    }
                }
                authority.tick_now();
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        });

        let service = http_service(tx, false);

        // initialize + session.
        let (session, _) = post(
            &service,
            None,
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": { "name": "test", "version": "0" },
                },
            }),
        )
        .await;
        let session = session.expect("session id");
        post(
            &service,
            Some(&session),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        )
        .await;

        // The tool list is exactly the embodied set: attach + the six
        // character verbs/senses. No world_*, no registry, no screenshots,
        // and no raw character_spawn_character.
        let (_, payloads) = post(
            &service,
            Some(&session),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        )
        .await;
        let tools: Vec<&str> = payloads[0]["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .map(|t| t["name"].as_str().expect("name"))
            .collect();
        let mut sorted = tools.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            vec![
                "character_attach",
                "character_jump",
                "character_pose",
                "character_sense_raycast",
                "character_sense_surroundings",
                "character_set_look",
                "character_set_move_intent",
                "character_set_posture",
            ]
        );

        let call = |name: &'static str, args: Value, id: u64| {
            let service = &service;
            let session = session.clone();
            async move {
                let (_, payloads) = post(
                    service,
                    Some(&session),
                    json!({
                        "jsonrpc": "2.0", "id": id, "method": "tools/call",
                        "params": { "name": name, "arguments": args },
                    }),
                )
                .await;
                payloads[0].clone()
            }
        };

        // Before attach: the surface refuses to act.
        let reply = call("character_pose", json!({ "character": "scout" }), 10).await;
        assert_eq!(reply["result"]["isError"], json!(true), "{reply}");

        // A world tool is not merely denied — it does not exist here.
        let reply = call("world_set_block", json!({}), 11).await;
        assert!(reply["error"].is_object(), "world tools absent: {reply}");
        let reply = call("character_spawn_character", json!({}), 12).await;
        assert!(reply["error"].is_object(), "raw spawn absent: {reply}");

        // Attach: spawns the character through the surface's parent token,
        // over open ground (the origin is the chasm floor — journal/0003)
        // and well ABOVE it: `surface_height_m` under-reports the voxel
        // surface (ROADMAP Observed), so spawning at the helper's height can
        // embed the body. Dropping in from +20 m lets gravity find the truth.
        let spawn = crate::app::find_open_spawn(
            &crate::worldgen::TerrainGen::new(SEED),
            dc_core::VoxelScale::from_player_height(crate::PLAYER_HEIGHT_M, 3),
        );
        let reply = call(
            "character_attach",
            json!({ "character": "scout",
                    "pos": { "x": spawn.x, "y": spawn.y + 20.0, "z": spawn.z } }),
            13,
        )
        .await;
        let content = &reply["result"]["structuredContent"];
        assert_eq!(content["ok"], json!(true), "{reply}");
        assert_eq!(content["spawned"], json!(true));

        // One character per session.
        let reply = call("character_attach", json!({ "character": "second" }), 14).await;
        assert_eq!(reply["result"]["isError"], json!(true), "{reply}");

        // Its own pose is visible; wait for gravity to put it on the ground.
        let mut grounded = false;
        for id in 20..80u64 {
            let reply = call("character_pose", json!({ "character": "scout" }), id).await;
            let pose = &reply["result"]["structuredContent"]["result"]["Ok"]["CharacterPose"];
            assert!(pose.is_object(), "{reply}");
            if pose["on_ground"] == json!(true) {
                grounded = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(grounded, "character settled under gravity");

        // Senses work through its body (while it stands on solid ground).
        let reply = call(
            "character_sense_surroundings",
            json!({ "character": "scout", "radius": 2 }),
            90,
        )
        .await;
        let region = &reply["result"]["structuredContent"]["result"]["Ok"]["Region"];
        assert_eq!(region["indices"].as_array().expect("indices").len(), 125);
        let reply = call(
            "character_sense_raycast",
            json!({ "character": "scout", "dir": { "x": 0.0, "y": -1.0, "z": 0.0 } }),
            91,
        )
        .await;
        let ray = &reply["result"]["structuredContent"]["result"]["Ok"]["CharacterRaycast"];
        assert_eq!(ray["hit"], json!(true), "looking down hits the ground");

        // Drive it: a move intent through the wire moves the body. West is
        // open at this spawn (east is a cliff face taller than a jump); a
        // real walker jumps as it goes, so this one does too.
        let reply = call(
            "character_set_move_intent",
            json!({ "character": "scout", "dx": -1.0, "dz": 0.0, "speed": 1.0 }),
            100,
        )
        .await;
        assert!(
            reply["result"]["structuredContent"]["result"]["Ok"].is_object(),
            "{reply}"
        );
        let mut moved = false;
        let mut last_pose = json!(null);
        for id in (101..221u64).step_by(2) {
            call("character_jump", json!({ "character": "scout" }), id).await;
            let reply = call("character_pose", json!({ "character": "scout" }), id + 1).await;
            last_pose =
                reply["result"]["structuredContent"]["result"]["Ok"]["CharacterPose"].clone();
            let x = last_pose["pos"]["x"].as_f64().expect("pos.x");
            if x < spawn.x - 0.7 {
                moved = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(
            moved,
            "move intent walked the body west from {spawn:?}; last pose {last_pose}"
        );

        // ATTENUATION CANNOT WIDEN: the session's token covers exactly its
        // character. Naming any other is refused by the host's capability
        // enforcement (not by tool-shape games).
        let reply = call(
            "character_set_move_intent",
            json!({ "character": "other", "dx": 1.0, "dz": 0.0, "speed": 1.0 }),
            300,
        )
        .await;
        assert!(
            reply["result"]["structuredContent"]["result"]["Rejected"]["MissingCapability"]
                .is_object(),
            "cross-character control denied: {reply}"
        );
        let reply = call("character_pose", json!({ "character": "other" }), 301).await;
        assert!(
            reply["result"]["structuredContent"]["result"]["Rejected"]["MissingCapability"]
                .is_object(),
            "cross-character senses denied: {reply}"
        );

        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        pump.join().expect("pump");
    }
}
