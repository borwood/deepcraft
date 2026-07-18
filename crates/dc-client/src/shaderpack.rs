//! Shader-pack loading and validation (S4 — hook surface v0).
//!
//! A pack is a directory: a `pack.toml` manifest naming the hook format it
//! targets plus one WGSL file per stage it overrides (docs/rendering/
//! PIPELINE.md). Loading composes each stage as `<stage prelude> + <pack
//! source>` — the prelude owns the bindings, the world-state struct, and the
//! `@fragment` entry point; the pack provides the stage body function — and
//! then validates the composed module through naga (the same frontend wgpu
//! will use). A pack that fails *any* check is rejected whole and the caller
//! falls back to the built-in default pack: a broken pack must never crash
//! the client or half-apply.
//!
//! This module is deliberately renderer-free (std + naga + serde only) so the
//! portability tests can exercise exactly the code the app runs, headless.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The hook-surface version this client implements. Packs must match exactly;
/// see docs/rendering/PIPELINE.md § Versioning.
pub const HOOK_FORMAT: u32 = 0;

/// Prepended to every pack's `post` stage source (the stage's entire
/// interface; see the file itself).
pub const POST_PRELUDE: &str = include_str!("shaders/post_prelude.wgsl");

/// The built-in copy of the default pack's post stage. `include_str!` so the
/// fallback exists even if `assets/` is missing or vandalized on disk.
const DEFAULT_POST_SOURCE: &str = include_str!("../../../assets/packs/default/post.wgsl");

/// The fragment entry point every composed post stage must expose (declared
/// by the prelude wrapper).
pub const POST_ENTRY_POINT: &str = "dc_post_main";

/// A validated, ready-to-compile shader pack.
#[derive(Debug)]
pub struct ShaderPack {
    pub name: String,
    /// Composed (prelude + pack) WGSL for the post stage, already validated.
    pub post_wgsl: String,
}

#[derive(Debug)]
pub enum PackError {
    Io(String),
    Manifest(String),
    /// The pack targets a hook format this client does not implement.
    Format {
        pack: u32,
        supported: u32,
    },
    /// The composed WGSL failed naga parsing or validation.
    Shader(String),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::Io(e) => write!(f, "io: {e}"),
            PackError::Manifest(e) => write!(f, "manifest: {e}"),
            PackError::Format { pack, supported } => write!(
                f,
                "pack targets hook format {pack}, this client supports {supported}"
            ),
            PackError::Shader(e) => write!(f, "shader: {e}"),
        }
    }
}

/// `pack.toml` schema, hook format 0. Unknown keys and unknown stage names
/// are errors: a stage the client cannot honor must fail loudly at load, not
/// silently drop part of the pack's look.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    pack: ManifestPack,
    #[serde(default)]
    stages: ManifestStages,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestPack {
    name: String,
    format: u32,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ManifestStages {
    /// WGSL file for the post stage, relative to the manifest. Absent =
    /// fall back to the default pack's post stage.
    post: Option<String>,
}

/// Compose a pack's post-stage source with the stage prelude.
pub fn compose_post(pack_source: &str) -> String {
    format!("{POST_PRELUDE}\n{pack_source}")
}

/// Parse and validate a composed WGSL module through naga, and check that
/// `entry_point` exists as a fragment entry. Errors are rendered to strings
/// with source context so the log line a pack author sees is actionable.
pub fn validate_wgsl(label: &str, wgsl: &str, entry_point: &str) -> Result<(), PackError> {
    let module = naga::front::wgsl::parse_str(wgsl)
        .map_err(|e| PackError::Shader(format!("{label}: {}", e.emit_to_string(wgsl))))?;

    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::default(),
    )
    .validate(&module)
    .map_err(|e| PackError::Shader(format!("{label}: {}", e.emit_to_string(wgsl))))?;

    let has_entry = module
        .entry_points
        .iter()
        .any(|ep| ep.stage == naga::ShaderStage::Fragment && ep.name == entry_point);
    if !has_entry {
        return Err(PackError::Shader(format!(
            "{label}: no fragment entry point `{entry_point}`"
        )));
    }
    Ok(())
}

/// Load and fully validate the pack in `dir`. Any failure rejects the pack.
pub fn load_pack(dir: &Path) -> Result<ShaderPack, PackError> {
    let manifest_path = dir.join("pack.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| PackError::Io(format!("{}: {e}", manifest_path.display())))?;
    let manifest: Manifest =
        toml::from_str(&manifest_text).map_err(|e| PackError::Manifest(e.to_string()))?;

    if manifest.pack.format != HOOK_FORMAT {
        return Err(PackError::Format {
            pack: manifest.pack.format,
            supported: HOOK_FORMAT,
        });
    }

    let post_source = match &manifest.stages.post {
        Some(rel) => {
            let path = dir.join(rel);
            std::fs::read_to_string(&path)
                .map_err(|e| PackError::Io(format!("{}: {e}", path.display())))?
        }
        None => DEFAULT_POST_SOURCE.to_string(),
    };

    let post_wgsl = compose_post(&post_source);
    validate_wgsl(
        &format!("{}/post", manifest.pack.name),
        &post_wgsl,
        POST_ENTRY_POINT,
    )?;

    Ok(ShaderPack {
        name: manifest.pack.name,
        post_wgsl,
    })
}

/// The built-in default pack (compiled into the binary). Panics only if the
/// in-tree default pack is itself invalid, which the test suite prevents.
pub fn builtin_default() -> ShaderPack {
    let post_wgsl = compose_post(DEFAULT_POST_SOURCE);
    validate_wgsl("built-in default/post", &post_wgsl, POST_ENTRY_POINT)
        .expect("the built-in default pack must always validate");
    ShaderPack {
        name: "default (built-in)".to_string(),
        post_wgsl,
    }
}

/// Resolve a `--pack` argument to a directory: a bare name looks under
/// `assets/packs/<name>` (relative to the working directory, like Bevy's own
/// asset root); anything with a path separator is used as a path.
pub fn resolve_pack_dir(selector: Option<&str>) -> PathBuf {
    match selector {
        Some(s) if s.contains('/') || s.contains('\\') => PathBuf::from(s),
        Some(s) => Path::new("assets/packs").join(s),
        None => PathBuf::from("assets/packs/default"),
    }
}

/// Load the selected pack, falling back to the built-in default on any
/// error. Never fails, never panics on user input: the fallback is the
/// contract's "broken pack" behavior.
pub fn load_pack_or_default(selector: Option<&str>) -> ShaderPack {
    let dir = resolve_pack_dir(selector);
    match load_pack(&dir) {
        Ok(pack) => {
            bevy::log::info!("shader pack `{}` loaded from {}", pack.name, dir.display());
            pack
        }
        Err(e) => {
            bevy::log::warn!(
                "shader pack at {} rejected ({e}); falling back to built-in default pack",
                dir.display()
            );
            builtin_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_packs_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs")
    }

    #[test]
    fn in_tree_packs_load_and_validate() {
        for name in ["default", "dusk"] {
            let pack = load_pack(&repo_packs_dir().join(name))
                .unwrap_or_else(|e| panic!("pack `{name}` must load: {e}"));
            assert_eq!(pack.name, name);
        }
    }

    #[test]
    fn builtin_default_is_valid() {
        let pack = builtin_default();
        assert!(pack.post_wgsl.contains("dc_post_main"));
    }

    #[test]
    fn broken_wgsl_is_rejected() {
        let composed = compose_post("fn dc_post(frag: DcPostIn) -> vec4<f32> { return 3; }");
        let err = validate_wgsl("test/post", &composed, POST_ENTRY_POINT).unwrap_err();
        assert!(matches!(err, PackError::Shader(_)), "got: {err:?}");
    }

    #[test]
    fn missing_stage_function_is_rejected() {
        // Valid WGSL, but the pack forgot to define dc_post: composition
        // fails because the prelude wrapper calls it.
        let composed = compose_post("fn unrelated() -> f32 { return 1.0; }");
        assert!(validate_wgsl("test/post", &composed, POST_ENTRY_POINT).is_err());
    }

    #[test]
    fn wrong_format_is_rejected() {
        let dir = std::env::temp_dir().join("dc-s4-wrong-format");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("pack.toml"),
            "[pack]\nname = \"future\"\nformat = 99\n",
        )
        .unwrap();
        let err = load_pack(&dir).unwrap_err();
        assert!(
            matches!(err, PackError::Format { pack: 99, .. }),
            "got: {err:?}"
        );
    }

    #[test]
    fn unknown_stage_name_is_rejected() {
        let dir = std::env::temp_dir().join("dc-s4-unknown-stage");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("pack.toml"),
            "[pack]\nname = \"x\"\nformat = 0\n[stages]\nwater = \"water.wgsl\"\n",
        )
        .unwrap();
        let err = load_pack(&dir).unwrap_err();
        assert!(matches!(err, PackError::Manifest(_)), "got: {err:?}");
    }

    #[test]
    fn manifest_without_stages_falls_back_to_default_post() {
        let dir = std::env::temp_dir().join("dc-s4-no-stages");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("pack.toml"),
            "[pack]\nname = \"bare\"\nformat = 0\n",
        )
        .unwrap();
        let pack = load_pack(&dir).expect("stage-less pack must load");
        assert_eq!(pack.post_wgsl, compose_post(DEFAULT_POST_SOURCE));
    }

    /// The S4 cross-backend insurance: every WGSL entry point of every
    /// in-tree pack must validate AND translate to HLSL (DX12), SPIR-V
    /// (Vulkan), and MSL (Metal). Dev is Windows-only; this is what keeps
    /// packs honest on the other backends (docs/SPIKES.md § S4).
    mod portability {
        use super::*;

        fn compose_all_stages(pack_dir: &Path) -> Vec<(String, String)> {
            let pack = load_pack(pack_dir).expect("in-tree pack must load");
            vec![(format!("{}/post", pack.name), pack.post_wgsl)]
        }

        fn parse_and_validate(label: &str, wgsl: &str) -> (naga::Module, naga::valid::ModuleInfo) {
            let module = naga::front::wgsl::parse_str(wgsl)
                .unwrap_or_else(|e| panic!("{label}: parse: {}", e.emit_to_string(wgsl)));
            let info = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::default(),
            )
            .validate(&module)
            .unwrap_or_else(|e| panic!("{label}: validate: {}", e.emit_to_string(wgsl)));
            (module, info)
        }

        #[test]
        fn every_pack_stage_translates_to_all_backends() {
            let mut stages_checked = 0;
            for name in ["default", "dusk"] {
                for (label, wgsl) in compose_all_stages(&repo_packs_dir().join(name)) {
                    let (module, info) = parse_and_validate(&label, &wgsl);
                    assert!(!module.entry_points.is_empty(), "{label}: no entry points");

                    // HLSL (DX12), all entry points.
                    let mut hlsl = String::new();
                    let hlsl_options = naga::back::hlsl::Options::default();
                    let hlsl_pipeline = naga::back::hlsl::PipelineOptions::default();
                    naga::back::hlsl::Writer::new(&mut hlsl, &hlsl_options, &hlsl_pipeline)
                        .write(&module, &info, None)
                        .unwrap_or_else(|e| panic!("{label}: HLSL: {e}"));
                    assert!(!hlsl.is_empty(), "{label}: empty HLSL");

                    // SPIR-V (Vulkan), all entry points.
                    let spv = naga::back::spv::write_vec(
                        &module,
                        &info,
                        &naga::back::spv::Options::default(),
                        None,
                    )
                    .unwrap_or_else(|e| panic!("{label}: SPIR-V: {e}"));
                    assert!(!spv.is_empty(), "{label}: empty SPIR-V");

                    // MSL (Metal), all entry points. MSL 2.0 matches what
                    // wgpu targets on contemporary macOS.
                    let msl_options = naga::back::msl::Options {
                        lang_version: (2, 0),
                        ..Default::default()
                    };
                    let (msl, _) = naga::back::msl::write_string(
                        &module,
                        &info,
                        &msl_options,
                        &naga::back::msl::PipelineOptions::default(),
                    )
                    .unwrap_or_else(|e| panic!("{label}: MSL: {e}"));
                    assert!(!msl.is_empty(), "{label}: empty MSL");

                    stages_checked += 1;
                }
            }
            assert_eq!(stages_checked, 2, "expected one post stage per pack");
        }
    }
}
