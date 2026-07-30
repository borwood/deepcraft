//! In-client MCP server: rmcp over **streamable HTTP on localhost**, running
//! on its own tokio thread and bridged to the ECS via channels.
//!
//! Surface = two layers:
//! - the dc-api command/query tools, GENERATED from the schema registry
//!   exactly like dc-mcp-dev (literally the same code —
//!   [`dc_mcp_dev::registry_tools`] / [`dc_mcp_dev::envelope_for_tool_call`],
//!   extracted there for sharing); the ECS-side [`crate::authority`] stamps
//!   envelopes with the session's dev-grant token, so tool callers never pass
//!   grants;
//! - three client-shell tools that are deliberately OUTSIDE the registry
//!   (they touch the window/renderer, not the world): `client_screenshot`,
//!   `client_player_pose_get`, `client_player_pose_set`.
//!
//! Transport: rmcp's `StreamableHttpService` (stateful sessions,
//! loopback-only Host validation by default) driven by a minimal hyper/1
//! HTTP1 accept loop — no router needed. Default port 7777 (`--mcp-port`),
//! `--no-mcp` disables the whole thread.
//!
//! Threading: tool calls run on the tokio thread; every request crosses to
//! the ECS as a [`BridgeRequest`] over an unbounded channel and awaits its
//! reply on a oneshot. Queries answer within a frame; commands answer when
//! their receipt materializes at the host tick boundary; screenshots answer
//! when the captured frame has been written to disk.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
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

/// Default localhost port for the in-client dev-surface MCP server.
pub const DEFAULT_MCP_PORT: u16 = 7777;
/// Default localhost port for the embodied character-surface MCP server
/// (docs/API.md § two MCP surfaces). On by default, like the dev surface:
/// both are loopback-only and the second proves the two-surface design every
/// time the game runs.
pub const DEFAULT_MCP_CHARACTER_PORT: u16 = 7778;

/// Command-line options for the MCP servers. `None` = that surface disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct McpOptions {
    /// Dev surface (broad dev grants). `--mcp-port <n>`.
    pub port: Option<u16>,
    /// Character surface (one attenuated character per session).
    /// `--mcp-character-port <n>`, `--no-mcp-character` to disable.
    pub character_port: Option<u16>,
}

impl McpOptions {
    /// Parse `--no-mcp` (disables BOTH surfaces) / `--mcp-port <n>` /
    /// `--no-mcp-character` / `--mcp-character-port <n>` out of the process
    /// arguments.
    pub fn parse(args: &[String]) -> Self {
        if args.iter().any(|a| a == "--no-mcp") {
            return Self {
                port: None,
                character_port: None,
            };
        }
        let flag = |name: &str, default: u16| {
            args.windows(2)
                .find(|w| w[0] == name)
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(default)
        };
        let character_port = if args.iter().any(|a| a == "--no-mcp-character") {
            None
        } else {
            Some(flag("--mcp-character-port", DEFAULT_MCP_CHARACTER_PORT))
        };
        Self {
            port: Some(flag("--mcp-port", DEFAULT_MCP_PORT)),
            character_port,
        }
    }

    pub fn any_enabled(&self) -> bool {
        self.port.is_some() || self.character_port.is_some()
    }
}

/// One request from the MCP session to the ECS.
pub enum BridgeRequest {
    /// A dc-api registry tool call (command or query).
    Api {
        tool: String,
        args: Value,
        reply: oneshot::Sender<Value>,
    },
    PoseGet {
        reply: oneshot::Sender<Value>,
    },
    PoseSet {
        pos: Option<[f64; 3]>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        /// Clamp feet to the terrain surface at (x, z) — walker-safe teleport.
        surface: bool,
        reply: oneshot::Sender<Value>,
    },
    Screenshot {
        name: String,
        reply: oneshot::Sender<Value>,
    },
    /// Character-surface session attach: spawn-or-attach to one character
    /// (spawns with the surface's parent token if it does not exist; default
    /// position = just in front of the player). `surface` drops the new body
    /// onto the true voxel surface at the requested x/z before the embed guard.
    CharacterAttach {
        name: String,
        pos: Option<[f64; 3]>,
        surface: bool,
        /// Registered body plan the new body wears; `None` = `dc:body/biped`
        /// (the identity default). Ignored when attaching to an existing
        /// character — a body swap is transmog, a separate verb (bodies.md).
        body_plan: Option<String>,
        reply: oneshot::Sender<Value>,
    },
    /// A character-surface tool call. The ECS side derives the session's
    /// token by attenuating the surface parent token to exactly
    /// `session_character` — the tool arguments never carry grants, and a
    /// payload naming any other character fails capability enforcement in
    /// the host.
    CharacterApi {
        tool: String,
        args: Value,
        session_character: String,
        reply: oneshot::Sender<Value>,
    },
    /// A character session ended: freeze that character's body (zero its move
    /// intent) — API.md § Characters, DECIDED 2026-07-19. Fire-and-forget (the
    /// sender is a session-teardown `Drop`, which cannot await a reply).
    CharacterFreeze {
        name: String,
    },
}

/// ECS resource: the receiving end of the bridge (drained every frame by
/// [`crate::authority::drain_bridge`]).
#[derive(Resource)]
pub struct McpBridge {
    pub rx: Mutex<mpsc::UnboundedReceiver<BridgeRequest>>,
}

/// Screenshot names are bare slugs — MCP callers are semi-trusted, and the
/// name becomes a filesystem path under `journal/assets/`. No separators, no
/// traversal, no extensions; lowercase to match the journal's asset naming.
pub fn valid_screenshot_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Spawn a Bevy screenshot capture of the next rendered frame and reply with
/// the absolute PNG path once it is on disk. Called from the ECS side
/// (`drain_bridge`); the name has already been validated at the tool boundary,
/// but re-check here — this is the function that touches the filesystem.
pub fn take_screenshot(commands: &mut Commands, name: &str, reply: oneshot::Sender<Value>) {
    if !valid_screenshot_name(name) {
        let _ = reply.send(json!({
            "ok": false,
            "error": "invalid screenshot name: bare slug required ([a-z0-9_-], max 64 chars)",
        }));
        return;
    }
    let dir = match std::env::current_dir() {
        Ok(cwd) => cwd.join("journal").join("assets"),
        Err(e) => {
            let _ = reply.send(json!({ "ok": false, "error": format!("cwd: {e}") }));
            return;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        let _ = reply.send(json!({ "ok": false, "error": format!("create journal/assets: {e}") }));
        return;
    }
    let path = dir.join(format!("{name}.png"));
    let path_string = path.display().to_string();
    let mut reply = Some(reply);
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>| {
            // Save ourselves (rather than bevy's `save_to_disk`) so failures
            // reach the MCP caller instead of only the log.
            let result = captured
                .image
                .clone()
                .try_into_dynamic()
                .map_err(|e| e.to_string())
                .and_then(|img| {
                    img.to_rgb8()
                        .save_with_format(&path, image::ImageFormat::Png)
                        .map_err(|e| e.to_string())
                });
            if let Some(tx) = reply.take() {
                let _ = tx.send(match result {
                    Ok(()) => json!({ "ok": true, "path": path_string.clone() }),
                    Err(e) => json!({ "ok": false, "error": e }),
                });
            }
        },
    );
}

/// The three client-shell tools (hand-authored: they are not dc-api commands
/// and deliberately live outside the schema registry).
pub fn client_tools() -> Vec<Tool> {
    let obj = |props: Value, required: Value| -> serde_json::Map<String, Value> {
        match json!({
            "type": "object",
            "properties": props,
            "required": required,
            "additionalProperties": false,
        }) {
            Value::Object(map) => map,
            _ => unreachable!(),
        }
    };
    vec![
        Tool::new(
            "client_screenshot",
            "Capture the next rendered frame to journal/assets/<name>.png and \
             return the absolute path. `name` must be a bare slug \
             ([a-z0-9_-], max 64 chars) — no path separators or extensions.",
            obj(
                json!({ "name": { "type": "string", "description": "bare slug, becomes <name>.png" } }),
                json!(["name"]),
            ),
        ),
        Tool::new(
            "client_player_pose_get",
            "Read the player pose: feet position in BOTH world meters (`pos`) \
             and world voxels (`pos_voxel`, the coordinate world_get_block \
             takes), yaw/pitch in radians, fly mode, ground contact, and \
             eye_in_solid (true = the camera is inside terrain and screenshots \
             will show backface nonsense — move before shooting).",
            obj(json!({}), json!([])),
        ),
        Tool::new(
            "client_player_pose_set",
            "Teleport and/or aim the player (dev-grant tool): any of pos \
             (feet, world meters), yaw, pitch (radians). Returns the resulting \
             pose — feet in meters (`pos`) and world voxels (`pos_voxel`), plus \
             eye_in_solid (if true, the view is buried — adjust before \
             screenshotting). With surface:true the reply always carries \
             `surface_snapped` (true = feet seated on the surface, false = no \
             ground under the footprint, position left as requested). Velocity \
             is zeroed on teleport.",
            obj(
                json!({
                    "pos": {
                        "type": "object",
                        "description": "feet position in world meters",
                        "properties": {
                            "x": { "type": "number" },
                            "y": { "type": "number" },
                            "z": { "type": "number" },
                        },
                        "required": ["x", "y", "z"],
                        "additionalProperties": false,
                    },
                    "yaw": { "type": "number", "description": "radians, 0 = -Z" },
                    "pitch": {
                        "type": "number",
                        "description": "radians, NEGATIVE looks down, positive up, clamped to ±1.55",
                    },
                    "surface": {
                        "type": "boolean",
                        "description": "clamp feet to the terrain surface at (x, z), ignoring pos.y — walker-safe teleport",
                    },
                }),
                json!([]),
            ),
        ),
    ]
}

/// The rmcp handler: registry tools + client tools, every call bridged to the
/// ECS.
#[derive(Clone)]
pub struct ClientMcpServer {
    tx: mpsc::UnboundedSender<BridgeRequest>,
}

impl ClientMcpServer {
    pub fn new(tx: mpsc::UnboundedSender<BridgeRequest>) -> Self {
        Self { tx }
    }
}

/// Map a tool name + JSON arguments onto the [`BridgeRequest`] it denotes,
/// paired with the oneshot the ECS side answers on. Shared by the in-client MCP
/// server (`call_tool`) and the dev console (`crate::console`) so both reach the
/// world through the one bridge with one mapping — client-shell tools decoded
/// here, dc-api registry tools forwarded as [`BridgeRequest::Api`]. `Ok(None)`
/// means the name is neither a client tool nor a registry command.
pub(crate) fn bridge_request_for(
    name: &str,
    args: &Value,
) -> Result<Option<(BridgeRequest, oneshot::Receiver<Value>)>, String> {
    let (tx, rx) = oneshot::channel();
    let request = match name {
        "client_screenshot" => {
            let shot_name = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or("`name` (string) is required")?;
            if !valid_screenshot_name(shot_name) {
                return Err(
                    "invalid screenshot name: bare slug required ([a-z0-9_-], max 64 chars)".into(),
                );
            }
            BridgeRequest::Screenshot {
                name: shot_name.to_string(),
                reply: tx,
            }
        }
        "client_player_pose_get" => BridgeRequest::PoseGet { reply: tx },
        "client_player_pose_set" => {
            let pos = match args.get("pos") {
                None | Some(Value::Null) => None,
                Some(p) => {
                    let component = |k: &str| {
                        p.get(k)
                            .and_then(Value::as_f64)
                            .ok_or_else(|| format!("pos.{k} (number) is required"))
                    };
                    Some([component("x")?, component("y")?, component("z")?])
                }
            };
            let angle = |k: &str| args.get(k).and_then(Value::as_f64).map(|v| v as f32);
            BridgeRequest::PoseSet {
                pos,
                yaw: angle("yaw"),
                pitch: angle("pitch"),
                surface: args
                    .get("surface")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                reply: tx,
            }
        }
        _ if dc_api::schema::id_for_mcp_tool(name).is_some() => BridgeRequest::Api {
            tool: name.to_string(),
            args: args.clone(),
            reply: tx,
        },
        _ => return Ok(None),
    };
    Ok(Some((request, rx)))
}

impl ServerHandler for ClientMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "deepcraft in-client dev surface: the dc-api command/query tools \
             (generated from the schema registry; commands apply at the \
             running game's host tick boundaries against the live world the \
             player is walking in) plus client_screenshot (captures the next \
             rendered frame to journal/assets) and client_player_pose get/set \
             (teleport + look). The session holds a broad dev-grant token.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = dc_mcp_dev::registry_tools();
        tools.extend(client_tools());
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = Value::Object(request.arguments.unwrap_or_default());
        let (bridge_request, rx) = match bridge_request_for(&request.name, &args) {
            Ok(Some(pair)) => pair,
            Ok(None) => {
                return Err(ErrorData::new(
                    ErrorCode::METHOD_NOT_FOUND,
                    format!("unknown tool `{}`", request.name),
                    None,
                ));
            }
            Err(message) => {
                return Ok(CallToolResult::error(vec![ContentBlock::text(message)]));
            }
        };
        if self.tx.send(bridge_request).is_err() {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                "client is shutting down",
            )]));
        }
        match rx.await {
            Ok(value) => Ok(CallToolResult::structured(value)),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "request dropped by the client (world reset or shutdown)",
            )])),
        }
    }
}

/// Build the streamable-HTTP tower service for a bridge sender. Shared by the
/// live server thread and the in-process session test (`sse_extras: false`
/// disables keep-alive pings and priming events so a response stream contains
/// exactly the response and then ends — collectable in a test).
pub fn http_service(
    tx: mpsc::UnboundedSender<BridgeRequest>,
    sse_extras: bool,
) -> StreamableHttpService<ClientMcpServer, LocalSessionManager> {
    let mut config = StreamableHttpServerConfig::default();
    if !sse_extras {
        config.sse_keep_alive = None;
        config.sse_retry = None;
    }
    StreamableHttpService::new(
        move || Ok(ClientMcpServer::new(tx.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    )
}

/// Start the enabled MCP surfaces, each on its own thread + port, all
/// bridging into one ECS-side channel: the dev surface (broad grants, port
/// 7777) and the character surface (one attenuated character per session,
/// port 7778 — see [`crate::mcp_character`]). Returns the bridge resource the
/// ECS drains AND a sender clone for the in-game dev console
/// (`crate::console`), which rides the SAME channel as a third consumer. The
/// channel (and thus the console) exists even when both MCP surfaces are
/// disabled (`--no-mcp`); only the server threads are conditional.
pub fn spawn_servers(options: McpOptions) -> (McpBridge, mpsc::UnboundedSender<BridgeRequest>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let console_tx = tx.clone();
    if !options.any_enabled() {
        info!(
            "MCP: both surfaces disabled (--no-mcp); the in-game dev console still shares the bridge"
        );
    }
    if let Some(port) = options.port {
        let tx = tx.clone();
        std::thread::Builder::new()
            .name("dc-mcp-http".into())
            .spawn(move || run_server(port, http_service(tx, true), "dev surface"))
            .expect("spawn MCP thread");
    }
    if let Some(port) = options.character_port {
        let tx = tx.clone();
        std::thread::Builder::new()
            .name("dc-mcp-character".into())
            .spawn(move || {
                run_server(
                    port,
                    crate::mcp_character::http_service(tx, true),
                    "character surface",
                )
            })
            .expect("spawn character MCP thread");
    }
    (McpBridge { rx: Mutex::new(rx) }, console_tx)
}

/// A current-thread tokio runtime accepting HTTP/1 connections on
/// `127.0.0.1:<port>` and serving one surface's streamable-HTTP MCP protocol.
fn run_server<H: ServerHandler>(
    port: u16,
    service: StreamableHttpService<H, LocalSessionManager>,
    label: &'static str,
) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("MCP: failed to build tokio runtime: {e}");
            return;
        }
    };
    runtime.block_on(async move {
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("MCP: cannot bind 127.0.0.1:{port}: {e} (use --mcp-port / --mcp-character-port or --no-mcp)");
                return;
            }
        };
        info!("MCP: {label} on http://127.0.0.1:{port}/mcp");
        loop {
            let (stream, _peer) = match listener.accept().await {
                Ok(accepted) => accepted,
                Err(e) => {
                    warn!("MCP: accept failed: {e}");
                    continue;
                }
            };
            let service = service.clone();
            tokio::spawn(async move {
                let io = hyper_util::rt::TokioIo::new(stream);
                let hyper_service = hyper::service::service_fn(
                    move |req: hyper::Request<hyper::body::Incoming>| {
                        let service = service.clone();
                        async move { Ok::<_, std::convert::Infallible>(service.handle(req).await) }
                    },
                );
                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    debug!("MCP: connection ended: {e}");
                }
            });
        }
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::authority::Authority;
    use crate::authority::tests::{edit_script, script_hash};
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};

    #[test]
    fn screenshot_names_are_bare_slugs_only() {
        for good in ["walk-01", "0002-first-edit", "a", "snap_2"] {
            assert!(valid_screenshot_name(good), "{good:?} should be valid");
        }
        for bad in [
            "",
            "..",
            "../escape",
            "a/b",
            "a\\b",
            "name.png",
            "UPPER",
            "with space",
            "c:evil",
            &"x".repeat(65),
        ] {
            assert!(!valid_screenshot_name(bad), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn options_parse_flags() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            McpOptions::parse(&args(&["game"])),
            McpOptions {
                port: Some(DEFAULT_MCP_PORT),
                character_port: Some(DEFAULT_MCP_CHARACTER_PORT),
            }
        );
        assert_eq!(
            McpOptions::parse(&args(&["game", "--mcp-port", "9000"])),
            McpOptions {
                port: Some(9000),
                character_port: Some(DEFAULT_MCP_CHARACTER_PORT),
            }
        );
        assert_eq!(
            McpOptions::parse(&args(&["game", "--mcp-character-port", "9001"])),
            McpOptions {
                port: Some(DEFAULT_MCP_PORT),
                character_port: Some(9001),
            }
        );
        assert_eq!(
            McpOptions::parse(&args(&["game", "--no-mcp-character"])),
            McpOptions {
                port: Some(DEFAULT_MCP_PORT),
                character_port: None,
            }
        );
        // --no-mcp kills both surfaces.
        assert_eq!(
            McpOptions::parse(&args(&["game", "--no-mcp", "--mcp-character-port", "9001"])),
            McpOptions {
                port: None,
                character_port: None,
            }
        );
    }

    /// POST one JSON-RPC message through a real streamable-HTTP service and
    /// return (session id header, data payloads). This is the actual MCP wire
    /// protocol — headers, session management, SSE framing — minus the TCP
    /// socket (the S5 session test's "identical to stdio minus the OS pipes"
    /// pattern, one transport further on). Generic over the handler so the
    /// character-surface session test (mcp_character.rs) shares it.
    pub(crate) async fn post<H: ServerHandler>(
        service: &StreamableHttpService<H, LocalSessionManager>,
        session: Option<&str>,
        body: Value,
    ) -> (Option<String>, Vec<Value>) {
        let mut builder = http::Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:7777/mcp")
            .header("host", "127.0.0.1:7777")
            .header("accept", "application/json, text/event-stream")
            .header("content-type", "application/json")
            .header("mcp-protocol-version", "2025-03-26");
        if let Some(id) = session {
            builder = builder.header("mcp-session-id", id);
        }
        let request = builder
            .body(Full::new(Bytes::from(body.to_string())))
            .expect("request");
        let response = service.handle(request).await;
        assert!(response.status().is_success(), "HTTP {}", response.status());
        let session_id = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let text = String::from_utf8_lossy(&bytes);
        let payloads = if content_type.starts_with("text/event-stream") {
            text.lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(str::trim)
                .filter(|data| !data.is_empty()) // priming events carry no data
                .map(|data| serde_json::from_str(data).expect("SSE data is JSON"))
                .collect()
        } else if text.is_empty() {
            Vec::new() // 202 accepted (notification)
        } else {
            vec![serde_json::from_str(&text).expect("JSON body")]
        };
        (session_id, payloads)
    }

    /// Items 3/4/5 of the milestone, end to end: a real MCP session over the
    /// streamable HTTP transport performs set_blocks + a scan against the
    /// client's hosted configuration (ECS stand-in pump driving the host
    /// tick), and the resulting world hash matches a direct-host run of the
    /// same script.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mcp_http_session_edits_scan_and_hash_parity() {
        const SEED: i32 = 1337;
        let (tx, mut rx) = mpsc::unbounded_channel();

        // ECS stand-in: drain the bridge into an Authority, tick the host on
        // a (fast) fixed cadence. Runs until every sender is dropped.
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pump_shutdown = shutdown.clone();
        let pump = std::thread::spawn(move || {
            let mut authority = Authority::new(SEED, 3);
            while !pump_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                while let Ok(request) = rx.try_recv() {
                    match request {
                        BridgeRequest::Api { tool, args, reply } => {
                            authority.handle_api_call(&tool, &args, reply);
                        }
                        _ => panic!("headless pump only serves dc-api tools"),
                    }
                }
                authority.tick_now();
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            authority
        });

        let service = http_service(tx, false);

        // initialize -> session id + server info.
        let (session, payloads) = post(
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
        let session = session.expect("initialize must issue a session id");
        assert!(
            payloads[0]["result"]["capabilities"]["tools"].is_object(),
            "tools capability advertised: {}",
            payloads[0]
        );
        post(
            &service,
            Some(&session),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        )
        .await;

        // tools/list: the registry surface plus the three client tools.
        let (_, payloads) = post(
            &service,
            Some(&session),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        )
        .await;
        let tools = payloads[0]["result"]["tools"]
            .as_array()
            .expect("tool list");
        assert_eq!(tools.len(), dc_api::schema::registry().len() + 3);
        for name in [
            "world_set_block",
            "world_scan_region",
            "client_screenshot",
            "client_player_pose_get",
            "client_player_pose_set",
        ] {
            assert!(
                tools.iter().any(|t| t["name"] == name),
                "missing tool {name}"
            );
        }

        // The scripted edit session, as tools/call requests over the wire.
        for (id, payload) in (10..).zip(edit_script()) {
            let tool = dc_api::schema::mcp_tool_name(payload.command_id());
            let outer = serde_json::to_value(&payload).expect("serialize");
            let args = outer
                .as_object()
                .and_then(|m| m.values().next())
                .expect("externally tagged")
                .clone();
            let (_, payloads) = post(
                &service,
                Some(&session),
                json!({
                    "jsonrpc": "2.0", "id": id, "method": "tools/call",
                    "params": { "name": tool, "arguments": args },
                }),
            )
            .await;
            let receipt = &payloads[0]["result"]["structuredContent"];
            assert!(
                receipt["result"]["Ok"].is_object(),
                "set_block receipt: {}",
                payloads[0]
            );
        }

        // Scan a row back through the wire and check the palette.
        let (_, payloads) = post(
            &service,
            Some(&session),
            json!({
                "jsonrpc": "2.0", "id": 99, "method": "tools/call",
                "params": { "name": "world_scan_region", "arguments": {
                    "min": { "x": 2, "y": 40, "z": 2 },
                    "max": { "x": 4, "y": 40, "z": 2 },
                }},
            }),
        )
        .await;
        let scan = &payloads[0]["result"]["structuredContent"]["result"]["Ok"]["Region"];
        assert_eq!(scan["palette"], json!(["dc:stone", "dc:wood"]));
        assert_eq!(scan["indices"], json!([0, 0, 1]));

        // Shut the pump down and compare against a direct-host run.
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut via_mcp = pump.join().expect("pump");
        let mut direct = Authority::new(SEED, 3);
        for payload in edit_script() {
            direct.submit_player(payload);
        }
        direct.tick_now();
        assert_eq!(
            script_hash(&mut direct),
            script_hash(&mut via_mcp),
            "world hash via MCP-over-HTTP == direct host"
        );
    }
}
