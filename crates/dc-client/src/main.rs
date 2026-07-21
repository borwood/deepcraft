//! The shell: window, renderer (Bevy/wgpu), input, audio.
//!
//! The only crate allowed to know about GPUs and operating systems. Everything
//! it does to the world goes through dc-api like every other consumer. Keeping
//! this layer thin is what keeps the someday-console-port a "fund a port"
//! problem instead of a rewrite (docs/ARCHITECTURE.md § Platforms).
//!
//! S1 additions: the voxel-scale walking skeleton (see docs/SPIKES.md § S1 and
//! docs/spikes/S1-results.md). Run with `--bench-scales` for the headless
//! measurement pass; run with no arguments for the interactive app.
//!
//! S3 additions: the far-mesh path (farmesh.rs — LOD rings out to 1.2 km) and
//! `--bench-storage`, the headless S3 measurement pass (palette compression,
//! LOD derive cost, column summaries, far-mesh cost; see
//! docs/spikes/S3-results.md).
//!
//! S4 additions: runtime shader packs (shaderpack.rs, poststage.rs; contract
//! in docs/rendering/PIPELINE.md). `--pack <name>` selects a pack directory
//! under assets/packs/ (default: `default`; try `dusk`); a broken pack falls
//! back to the built-in default with a warning, never a crash.
//!
//! S6 additions: the physics demo (physdemo.rs) — **G** tosses a rigid-body
//! cube that collides with loaded terrain through dc-physics' collider
//! bubbles (see docs/spikes/S6-results.md).
//!
//! Client-through-dc-api milestone: the game hosts dc-api's `HostWorld` as
//! the edit authority (authority.rs), player LMB/RMB edits become
//! `dc:world/set_block` commands (edit.rs), and an in-client MCP server over
//! streamable HTTP (mcp.rs; default port 7777, `--mcp-port <n>`, `--no-mcp`)
//! lets an agent drive and observe the running game — including
//! `client_screenshot` captures into journal/assets. See journal/0002.
//!
//! Character-MCP milestone: a SECOND MCP surface on port 7778
//! (mcp_character.rs; `--mcp-character-port <n>`, `--no-mcp-character`;
//! `--no-mcp` disables both) hosts embodied character sessions — an AI
//! attaches to one character (character.rs renders its jointed biped body via
//! the body-plan registry, animated by body.rs — bodies.md steps 1–2), drives
//! it through collision-checked movement commands, and
//! perceives only through its senses, under a token attenuated to exactly
//! that character. See docs/API.md § Characters and journal/0005.

mod app;
mod authority;
mod bench;
mod bench_storage;
mod body;
mod character;
mod console;
mod edgepass;
mod edit;
mod farmesh;
mod mcp;
mod mcp_character;
mod meshing;
mod physdemo;
mod player;
mod poststage;
mod shaderpack;
mod streaming;
mod terrain_material;
mod worldgen;

/// The player is always this tall in meters; the voxel scale decides how many
/// voxels that is.
pub const PLAYER_HEIGHT_M: f64 = 1.8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--bench-scales") {
        bench::run();
    } else if args.iter().any(|a| a == "--bench-storage") {
        bench_storage::run();
    } else {
        let pack = args
            .windows(2)
            .find(|w| w[0] == "--pack")
            .map(|w| w[1].clone());
        // `--fullbright`: unlit materials — the agent-walk diagnostic mode.
        // Anything non-shader-related should be tested with this on, so
        // lighting/tonemap output never masquerades as a geometry or data
        // defect (journal/0004's lesson, generalized).
        let fullbright = args.iter().any(|a| a == "--fullbright");
        // `--edges`: renderer-added crease/silhouette outlining — the shape
        // diagnostic. Composable with `--fullbright` (the expected use: benches
        // legible on the flat-albedo field) and with the lit pass. When off,
        // the edge pass is never scheduled (edgepass.rs), so `--fullbright`
        // alone stays the byte-identical pure-data control (corrections #18).
        let edges = args.iter().any(|a| a == "--edges");
        // Deep-config plumbing (journal/0039): gen-time flags that flip the
        // always-off deep-time knobs ON at world creation. They only bite the
        // worldgen authority (boot / key 2); the legacy S1 terrain (keys 3/4)
        // ignores them. Mirror the `--fullbright` boolean and `--pack <v>`
        // windows(2) parsing patterns above.
        //
        // - `--tectonics`      analytic tectonic history (chapters, isostasy,
        //                      crustal columns, drainage export) — reshapes the
        //                      terrain, so it is a deliberate world-creation call.
        // - `--full-agents`    the wind + frost + wave erosion roster.
        // - `--amplitude <n>`  orogenic thickening scale (m/iter). Only bites
        //                      when `--tectonics` is on — it is the amplitude the
        //                      tectonic forcing multiplies.
        // - `--extent <small|medium|large>`  world size (default: medium).
        let mut deep = dc_worldgen::DeepOverrides::default();
        if args.iter().any(|a| a == "--tectonics") {
            deep.tectonic_history = Some(true);
        }
        if args.iter().any(|a| a == "--full-agents") {
            deep.full_agents = Some(true);
        }
        if let Some(v) = args.windows(2).find(|w| w[0] == "--amplitude") {
            match v[1].parse::<f64>() {
                Ok(n) => deep.thickening_scale = Some(n),
                Err(_) => {
                    eprintln!(
                        "--amplitude expects a number (m/iter), got `{}`; ignoring it",
                        v[1]
                    );
                }
            }
        }
        let extent = match args.windows(2).find(|w| w[0] == "--extent") {
            Some(v) => match dc_worldgen::Extent::from_arg(&v[1]) {
                Some(e) => e,
                None => {
                    eprintln!(
                        "--extent expects small|medium|large, got `{}`; using the default",
                        v[1]
                    );
                    authority::GenOptions::default().extent
                }
            },
            None => authority::GenOptions::default().extent,
        };
        let gen_options = authority::GenOptions { extent, deep };
        app::run(
            pack,
            mcp::McpOptions::parse(&args),
            fullbright,
            edges,
            gen_options,
        );
    }
}
