# deepcraft

Internal codename. Public name TBD.

A voxel game that starts near Minecraft and grows toward Dwarf Fortress-in-3D:
plugin-first architecture, first-class MCP integration, in-game authoring tools
(items, blocks, blueprints, cube-based models + keyframe animations),
first-class distant-terrain LOD and custom shader support, globally-aware
worldgen with deep-time history, and a tiered "entire world stays simulated"
model based on deferred resolution of superposed state.

## Layout

| Crate | What it is |
|---|---|
| `crates/dc-core` | Voxel data model, chunk/LOD storage, ids, math. Headless, zero I/O. |
| `crates/dc-sim` | Tiered simulation: full ECS tier, coarse agent tier, statistical tier; the constraint ledger and bounded collapse. Headless. |
| `crates/dc-worldgen` | Hierarchical lazy generation (continent graph → region → chunk) and deep-time history (runs the sim tiers over pre-player time). Headless. |
| `crates/dc-api` | The one command/query surface consumed by WASM plugins, the in-process MCP server, and in-game editors. |
| `crates/dc-client` | The shell: window, renderer, input, audio. The only crate allowed to know about GPUs and OSes. |

Start with `docs/ARCHITECTURE.md`, then `docs/SPIKES.md` for what gets built next
and in what order.

## Building

```
cargo check --workspace
cargo test --workspace
```

Developed on Windows; Windows/macOS/Linux are first-class targets from day one
(CI checks all three). Console ports are a someday-goal that we protect by
keeping the platform layer thin — see ARCHITECTURE.md § Platforms.
