//! A real MCP session against the generated tool surface: an in-process rmcp
//! client talks MCP (initialize, tools/list, tools/call) to the server over a
//! tokio duplex pipe — the same wire protocol as stdio, minus the OS pipes.

use std::sync::{Arc, Mutex};

use dc_api::schema;
use dc_api::{ConsumerId, ConsumerKind, HostWorld};
use dc_mcp_dev::{DevMcpServer, ToolLayer, dev_session_token};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

fn args(value: Value) -> serde_json::Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => panic!("arguments must be an object, got {other}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_session_list_define_build_scan() {
    let world = Arc::new(Mutex::new(HostWorld::new(11)));
    let layer = ToolLayer {
        world: world.clone(),
        consumer: ConsumerId::new(ConsumerKind::McpSession, "test-session"),
        session_token: dev_session_token(),
        auto_tick: true,
    };

    let (client_io, server_io) = tokio::io::duplex(1 << 16);
    let server = tokio::spawn(async move {
        let service = rmcp::serve_server(DevMcpServer::new(layer), server_io)
            .await
            .expect("server init");
        let _ = service.waiting().await;
    });
    let client = rmcp::serve_client((), client_io)
        .await
        .expect("client init");

    // 1. List tools: generated from the registry, one per command, valid names.
    let tools = client.peer().list_all_tools().await.expect("list tools");
    assert_eq!(tools.len(), schema::registry().len());
    for spec in schema::registry() {
        let tool = tools
            .iter()
            .find(|t| t.name == schema::mcp_tool_name(spec.id))
            .unwrap_or_else(|| panic!("missing tool for {}", spec.id));
        // The registry id survives into the description (provenance).
        assert!(tool.description.as_deref().unwrap_or("").contains(spec.id));
        assert_eq!(
            Value::Object((*tool.input_schema).clone()),
            (spec.payload_schema)(),
            "input schema for {} must be the registry schema",
            spec.id
        );
    }

    let call = |name: &'static str, arguments: Value| {
        let peer = client.peer().clone();
        async move {
            let result = peer
                .call_tool(CallToolRequestParams::new(name).with_arguments(args(arguments)))
                .await
                .expect("tool call");
            assert_ne!(result.is_error, Some(true), "tool errored");
            result.structured_content.expect("structured receipt")
        }
    };

    // 2. Define an item in the session's namespace.
    let receipt = call(
        "registry_define_item",
        json!({"name": "dev:probe", "display_name": "Probe", "description": "MCP-authored"}),
    )
    .await;
    assert_eq!(receipt["result"]["Ok"]["items_defined"][0], "dev:probe");
    assert_eq!(receipt["tick_applied"], 1); // auto-tick applied it immediately

    // 3. Place blocks: one set_block + one fill.
    let receipt = call(
        "world_set_block",
        json!({"pos": {"x": 40, "y": 3, "z": 40}, "block": "dc:wood"}),
    )
    .await;
    assert_eq!(
        receipt["result"]["Ok"]["blocks_changed"][0]["to"],
        "dc:wood"
    );
    let receipt = call(
        "world_fill",
        json!({
            "min": {"x": 41, "y": 3, "z": 40},
            "max": {"x": 43, "y": 3, "z": 40},
            "block": "dc:stone"
        }),
    )
    .await;
    assert_eq!(
        receipt["result"]["Ok"]["blocks_changed"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );

    // 4. Scan the row back.
    let scan = call(
        "world_scan_region",
        json!({
            "min": {"x": 40, "y": 3, "z": 40},
            "max": {"x": 43, "y": 3, "z": 40}
        }),
    )
    .await;
    assert_eq!(
        scan["result"]["Ok"]["Region"]["palette"],
        json!(["dc:wood", "dc:stone"])
    );
    assert_eq!(
        scan["result"]["Ok"]["Region"]["indices"],
        json!([0, 1, 1, 1])
    );

    // 5. Capability enforcement over MCP: the session token owns dev:*, not foo:*.
    let receipt = call(
        "registry_define_item",
        json!({"name": "foo:sneaky", "display_name": "Sneaky"}),
    )
    .await;
    let rejected = &receipt["result"]["Rejected"]["MissingCapability"];
    assert!(
        rejected["needed"]
            .as_str()
            .is_some_and(|s| s.contains("registry.define(foo)")),
        "expected namespace rejection, got {receipt}"
    );

    // 6. Unknown tool → protocol error (METHOD_NOT_FOUND), not a receipt.
    let err = client
        .peer()
        .call_tool(CallToolRequestParams::new("world_no_such_tool"))
        .await;
    assert!(err.is_err());

    client.cancel().await.expect("clean shutdown");
    server.abort();

    // The MCP writes really landed in the shared world.
    let mut w = world.lock().unwrap();
    assert!(w.item("dev:probe").is_some());
    let block = w.block_at(dc_api::Vec3i::new(42, 3, 40));
    assert_eq!(dc_api::host::block_name(block), "dc:stone");
}
