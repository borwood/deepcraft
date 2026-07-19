//! Shared test plumbing: build (once) and locate the demo plugin wasm.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// Build plugins/demo-builder for wasm32-unknown-unknown (in its own target
/// dir, so it does not contend with the workspace build lock held by the
/// enclosing `cargo test`) and return the artifact path.
pub fn plugin_wasm_path() -> PathBuf {
    static WASM: OnceLock<PathBuf> = OnceLock::new();
    WASM.get_or_init(|| {
        // Resolve the manifest dir at RUNTIME: the compile-time env! path is
        // baked into the cached artifact, and with agent worktrees sharing
        // one CARGO_TARGET_DIR the binary may have been compiled in a
        // since-deleted worktree (NotADirectory at spawn — found the hard
        // way when a full-workspace gate run picked up a stale flavor).
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
        let plugin_dir = Path::new(&manifest_dir)
            .ancestors()
            .nth(2)
            .expect("workspace root")
            .join("plugins")
            .join("demo-builder");
        let cargo = std::env::var("CARGO")
            .unwrap_or_else(|_| option_env!("CARGO").unwrap_or("cargo").to_string());
        let status = Command::new(cargo)
            .current_dir(&plugin_dir)
            .env("CARGO_TARGET_DIR", plugin_dir.join("target"))
            .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
            .status()
            .expect("spawn cargo for the demo plugin");
        assert!(status.success(), "demo plugin build failed");
        let wasm = plugin_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("demo_builder.wasm");
        assert!(wasm.exists(), "missing artifact {}", wasm.display());
        wasm
    })
    .clone()
}
