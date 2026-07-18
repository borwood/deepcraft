//! The one command/query surface. Three consumers, one contract:
//!
//! 1. **WASM plugins** (wasmtime host, spike S5) — sandboxed, language-agnostic.
//! 2. **MCP** (in-process rmcp server, spike S5) — agents drive the same API.
//! 3. **In-game editors** (item/block/blueprint/model+anim authoring) — UI over
//!    the same commands, so anything an editor can do, a plugin or agent can do.
//!
//! Commands and queries are serializable data (serde), not trait calls, so the
//! same surface crosses the WASM boundary, the MCP boundary, and (later) the
//! network boundary unchanged. Capability-scoped: a consumer holds explicit
//! grants, not ambient authority.

pub const CRATE_ROLE: &str = "command/query surface for plugins, MCP, editors";
