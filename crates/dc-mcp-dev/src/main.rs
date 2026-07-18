//! Stdio entry point: `dc-mcp-dev [seed]` serves the dev MCP surface over
//! stdin/stdout for any MCP client (e.g. `claude mcp add deepcraft -- dc-mcp-dev`).

use std::sync::{Arc, Mutex};

use dc_api::{ConsumerId, ConsumerKind, HostWorld};
use dc_mcp_dev::{DevMcpServer, ToolLayer, dev_session_token};
use rmcp::ServiceExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seed: u64 = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(0);
    let layer = ToolLayer {
        world: Arc::new(Mutex::new(HostWorld::new(seed))),
        consumer: ConsumerId::new(ConsumerKind::McpSession, "stdio"),
        session_token: dev_session_token(),
        auto_tick: true,
    };
    let service = DevMcpServer::new(layer)
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
