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
//! S3 additions: the far-mesh path (farmesh.rs — LOD rings out to 1.2 km by
//! default, `--horizon <km>` to reach further; journal/0042) and
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
mod devicelost;
mod edgepass;
mod edit;
mod farmesh;
mod farpyramid;
mod mcp;
mod mcp_character;
mod meshing;
mod perf;
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

/// The process exit code is part of the failure contract (journal/0054): two
/// recorded GPU crashes reported **exit 0** because the panics happened on
/// worker threads, so exit-code monitoring could not see them at all. `main`
/// therefore returns an `ExitCode` that `devicelost::finish` derives from what
/// actually happened, not from whichever `AppExit` the event loop produced.
fn main() -> std::process::ExitCode {
    // Installed before anything else can panic, so a panic in world generation
    // or plugin build is counted too.
    devicelost::install_panic_hook();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--bench-scales") {
        bench::run();
        devicelost::finish(&bevy::app::AppExit::Success)
    } else if args.iter().any(|a| a == "--bench-storage") {
        bench_storage::run();
        devicelost::finish(&bevy::app::AppExit::Success)
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
        //                      when `--tectonics` is on — it is the TECTONIC
        //                      amplitude the tectonic forcing multiplies.
        // - `--erosion-budget <mult>`  the EROSION budget multiplier
        //                      (`erodibility_probe` experiment B): scales
        //                      weathering / k_transport / k_bedrock TOGETHER by
        //                      <mult>, so relative rates never move — only the
        //                      total erosion does. Distinct from `--amplitude`
        //                      (which is tectonic, journal/0040); `1.0` is
        //                      byte-identical to omitting it. This is the dev
        //                      lever that makes the cranked "conservative
        //                      amplitude" world walkable (journal/0076).
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
        if let Some(v) = args.windows(2).find(|w| w[0] == "--erosion-budget") {
            match v[1].parse::<f64>() {
                Ok(n) => deep.erosion_budget = Some(n),
                Err(_) => {
                    eprintln!(
                        "--erosion-budget expects a number (multiplier), got `{}`; ignoring it",
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
        // `--horizon <km>`: how far the far field reaches, in kilometers
        // (journal/0042). The shipped 1.2 km horizon puts the camera INSIDE
        // every landform — a mountain range is 5–20 km across — so macro shape
        // never entered frame. Omit the flag and the horizon is exactly the
        // shipped 1.2 km; pass e.g. `--horizon 8` for an 8 km field. A garbage or
        // out-of-range value warns and falls back rather than aborting the boot
        // (the `--amplitude` behaviour).
        let horizon = match args.windows(2).find(|w| w[0] == "--horizon") {
            Some(v) => match v[1].parse::<f64>() {
                Ok(km)
                    if (farmesh::HORIZON_MIN_M..=farmesh::HORIZON_MAX_M)
                        .contains(&(km * 1000.0)) =>
                {
                    farmesh::HorizonConfig::with_far_max(km * 1000.0)
                }
                Ok(km) => {
                    eprintln!(
                        "--horizon {km} km is outside {:.1}–{:.0} km; using the default {:.1} km",
                        farmesh::HORIZON_MIN_M / 1000.0,
                        farmesh::HORIZON_MAX_M / 1000.0,
                        farmesh::DEFAULT_FAR_MAX_M / 1000.0,
                    );
                    farmesh::HorizonConfig::default()
                }
                Err(_) => {
                    eprintln!(
                        "--horizon expects a number (kilometers), got `{}`; using the default",
                        v[1]
                    );
                    farmesh::HorizonConfig::default()
                }
            },
            None => farmesh::HorizonConfig::default(),
        };
        // `--perf-drop <seconds>`: the deterministic vertical-drop capture
        // (perf window, journal/0080). Only meaningful with `--features perf`;
        // without it the flag is parsed and warned about, never crashes.
        let perf_drop = args
            .windows(2)
            .find(|w| w[0] == "--perf-drop")
            .and_then(|w| match w[1].parse::<f64>() {
                Ok(n) if n > 0.0 => Some(n),
                _ => {
                    eprintln!(
                        "--perf-drop expects a positive number of seconds, got `{}`; ignoring it",
                        w[1]
                    );
                    None
                }
            });
        #[cfg(not(feature = "perf"))]
        if perf_drop.is_some() {
            eprintln!(
                "--perf-drop needs a perf build (`--features perf`); ignoring it in this build"
            );
        }
        let exit = app::run(
            pack,
            mcp::McpOptions::parse(&args),
            fullbright,
            edges,
            gen_options,
            horizon,
            perf_drop,
        );
        devicelost::finish(&exit)
    }
}
