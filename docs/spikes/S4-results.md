# S4 — Shader hook contract: results

Status: spike complete, 2026-07-18. Contract: **docs/rendering/PIPELINE.md**
(the main deliverable — pipeline shape decision + hook surface v0). Code:
`dc-client/src/{shaderpack,poststage}.rs`, `dc-client/src/shaders/
post_prelude.wgsl`, `assets/packs/{default,dusk}/` (kept — this is the v0
pack loader and post stage, not throwaway). Exit criteria met: pipeline
decision recorded, hook-surface v0 spec'd, a loadable example pack in-tree,
cross-backend translation checked in CI.

## How to run

- `cargo run --release -p dc-client` — default pack (clear-day haze).
- `cargo run --release -p dc-client -- --pack dusk` — the override demo:
  desaturated violet dusk grade, visibly different from the default.
- `cargo run --release -p dc-client -- --pack <path-or-name>` — any pack
  directory; a broken pack logs one WARN with naga diagnostics and falls
  back to the built-in default. Never crashes.
- `cargo test -p dc-client` — pack validation rules + the portability gate
  (naga validation + HLSL/SPIR-V/MSL translation of every pack stage).

## Research findings

### LabPBR (shaderlabs.org, v1.3) — adopted in visuals.md, spec'd in PIPELINE.md § 5

Two packed textures beside basecolor. Normal: XY in RG (Z reconstructed),
AO in B (linear, 255 = unoccluded), height in A (POM range = top 25%,
0 = deepest; keep ≥ 1 to dodge POM artifacts). Specular: R perceptual
smoothness with `roughness = (1 − s)²`; G is F0 — 0–229 linear
reflectance, 230–255 a predefined-metal table (iron=230 … silver=237,
255 = albedo-as-F0); B splits 0–64 porosity / 65–255 SSS; A emission with
0–254 meaningful and 255 reserved. Two consequences we bake into the
contract: porosity and SSS share a channel (fine: porous things don't
scatter, scattering things aren't porous), and emission color comes from
albedo — which is exactly what the future colored-light store wants.

### Iris/OptiFine pass structure (shaders.properties) — the proven shape of this genre

Program order: `setup*` (compute) → `begin*` → `shadow` → `shadowcomp*` →
`prepare*` → gbuffers (opaque) → `deferred*` → gbuffers (translucent) →
`composite*` → `final`. Two program families: geometry programs
("gbuffers"-style, per-object, with a documented **fallback chain** — absent
programs fall back to more generic ones) and fullscreen quad programs
(numbered 1–99 per stage). Lessons taken: (1) the stage list + what each
stage may read IS the pack contract — Iris packs survived a decade of
Minecraft versions because that shape stayed stable; (2) per-stage fallback
(our per-stage fall-back-to-default-pack) beats all-or-nothing packs;
(3) Iris's *implicit* versioning (feature sniffing, compat heuristics) is a
tarpit — we version explicitly (`format = N`, exact match) instead.

### Bevy 0.19 extension points

- **The render graph is gone.** 0.19 replaced `Node`/`RenderLabel`/edge
  APIs with plain ECS schedules: `Core3d` is a schedule run per camera;
  passes are systems in `Core3dSystems::{Prepass, MainPass,
  EarlyPostProcess, PostProcess}` sets, ordered with `.after()/.before()`.
  Inserting a pass is ~1 system + 1 pipeline resource + 1 prepare system.
- `ViewTarget::post_process_write()` gives the ping-pong source/destination
  pair for post-style passes; `FullscreenShader` provides the shared
  fullscreen-triangle vertex stage.
- Runtime shaders are just `Assets<Shader>::add(Shader::from_wgsl(...))` —
  no asset-dir requirement, so the loader can read pack files itself and
  validate through naga *before* the renderer ever sees them. The
  `PipelineCache` compiles asynchronously; an unready pipeline is a skipped
  pass, not a stall or crash.
- Bevy 0.19 pins wgpu 29 → naga 29; dc-client depends on naga 29 directly
  (`wgsl-in` at runtime, plus `hlsl-out`/`spv-out`/`msl-out` as
  dev-features), so loader validation and renderer compilation use the same
  frontend by construction.

## What the slice proved

- **Runtime pack loading works end-to-end**: `--pack dusk` loads
  `assets/packs/dusk/{pack.toml,post.wgsl}`, composes it with the in-tree
  prelude, validates through naga, compiles through Bevy's pipeline cache,
  and renders — one system in `Core3dSystems::PostProcess` (after Bevy's
  inert tonemapping, before upscaling) drawing one fullscreen triangle.
- **The default look is itself a pack** (visuals.md requirement): the
  default clear-day haze/tonemap goes through the identical manifest →
  compose → validate → pipeline path, and doubles as the compiled-in
  fallback (`include_str!`), so a vandalized assets dir still renders.
- **Override is visibly real**: dusk drops exposure 0.45×, pulls fog in to
  0.4×start/0.7×end with a violet haze, desaturates 55%, cools the cast and
  applies an ACES-fit shoulder — confirmed side-by-side on the live app.
- **Broken packs degrade exactly as specified**: missing dir, bad
  manifest, wrong `format`, unknown stage name, WGSL that doesn't parse,
  WGSL missing `dc_post` — each rejects the whole pack with one WARN line
  (naga's source-context diagnostics included) and falls back. All paths
  unit-tested; the runtime path manually verified.
- **Cross-backend portability is a CI gate, not a hope**: every composed
  entry point of both packs validates and translates to HLSL (SM 5.1),
  SPIR-V, and MSL 2.0 in `cargo test -p dc-client` (headless, no GPU).
- **Depth-based work in post is viable**: the stage linearizes Bevy's
  infinite reverse-Z (`view_z = near / depth`) from the bound depth texture;
  distance haze over the 1.2 km S3 far field works and visually swallows
  the LOD ring seams (the visuals.md "haze is load-bearing" claim held).

## Bevy friction encountered

1. **The 0.19 schedule migration invalidated most documentation.** Official
   examples/web docs still show `render_graph::Node` + `RenderLabel`; the
   real 0.19 API (systems in `Core3d`, `ViewQuery`, `RenderContext` as a
   SystemParam, `BindGroupLayoutDescriptor` + `PipelineCache::
   get_bind_group_layout`) had to be read out of the published crate source
   (`bevy_post_process::motion_blur` is the best in-tree template).
2. **`RenderStartup` vs extraction ordering**: a resource the pipeline-init
   system needs (the pack's shader handle) cannot arrive via
   `ExtractResourcePlugin` — extraction hasn't run when `RenderStartup`
   does. Inserting the handle directly into the render app at plugin build
   is the working pattern.
3. **Bevy's tonemapping had to be explicitly disarmed** (`Tonemapping::None`
   on the camera) — it lives in the same `PostProcess` set; ordering against
   it (`.after(tonemapping)`) is required either way to avoid schedule
   ambiguity with a second fullscreen pass.
4. **Depth binding needs opt-in texture usage**
   (`Camera3d::depth_texture_usages |= TEXTURE_BINDING`) and is
   MSAA-shaped: the sampled depth texture type changes with sample count, so
   the v0 contract pins `Msaa::Off` (the pass skips MSAA views rather than
   mis-bind). A format-1 prelude must either carry both layouts (Bevy's
   motion blur ships dual MSAA/non-MSAA layouts) or keep MSAA out of the
   contract.
5. Minor: `FragmentState`'s default entry point differs from a pack's named
   one (`entry_point: Some("dc_post_main")` required); `Projection` is an
   enum with a `Custom` arm that must be handled when extracting `near`.

## Open questions (owed to later spikes/formats)

1. **HDR.** v0 grades an LDR (non-HDR camera) target, so the default pack's
   tonemap is identity and dusk's ACES fit is a stylistic curve, not real
   range compression. Real darkness + earned firelight (visuals.md) wants
   `Camera::hdr = true` and exposure control — decide when the lighting
   spike lands; the contract already reserves the curve to the pack.
2. **Sky as a pass vs sky in post.** Both v0 packs paint sky pixels inside
   `post` (no ray direction available — only `is_sky`). A real `sky` hook
   needs the inverse view-projection in `DcWorldState` (format 1 appendix)
   or its own dome pass.
3. **MSAA in the contract** (friction #4): dual-layout preludes or a
   permanent single-sample promise for pack-visible depth.
4. **Multi-stage packs.** Format 0 has exactly one overridable stage; the
   manifest, loader, and per-stage fallback already generalize (keyed stage
   table), but prelude-per-stage discipline is unproven at, say, five
   stages. Reassess when `water` or `sky` becomes hook #2.
5. **Pack-declared parameters.** Iris packs expose user-tweakable options
   (screen menus generated from shader comments). Our manifest has no
   `[options]` section yet; adding one is additive within a format.
6. **Where world state comes from.** `PostStage` values are currently
   app-set constants (fixed 0.35 time-of-day matching the fixed sun). The
   day-cycle/weather systems that will drive them (and couple wetness to
   S8's porosity) don't exist yet; the uniform packing is sized for that
   future, not this present.
