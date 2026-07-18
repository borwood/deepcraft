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

mod app;
mod bench;
mod meshing;
mod player;
mod streaming;
mod worldgen;

/// The player is always this tall in meters; the voxel scale decides how many
/// voxels that is.
pub const PLAYER_HEIGHT_M: f64 = 1.8;

fn main() {
    if std::env::args().any(|a| a == "--bench-scales") {
        bench::run();
    } else {
        app::run();
    }
}
