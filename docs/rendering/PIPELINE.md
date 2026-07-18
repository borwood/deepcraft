# deepcraft rendering pipeline & shader-pack hook contract

Status: v0, decided by spike S4 (2026-07-18; findings in
docs/spikes/S4-results.md). This document is the contract: the pipeline
shape, the pass list, and the surface a shader pack may see and override.
The Iris/OptiFine lesson this document exists to honor: **packs survive
because the pipeline shape is a stable, versioned contract** — pass
structure, buffer contents, and uniforms change only with a format bump,
never silently.

Serves the art direction in docs/design/visuals.md ("elevated pixel game"):
the default pack IS that document; other packs may take the same world
elsewhere.

## 1. Pipeline shape — DECIDED: clustered forward (Forward+), not deferred

The main opaque/translucent passes are **clustered forward** on Bevy 0.19's
Core3d pipeline, with Bevy's existing depth/normal **prepass** kept on as
the source of screen-space inputs for later stages. We do not commit to a
G-buffer layout, and there is deliberately no deferred lighting pass.

Justification against the requirements (visuals.md):

- **Many colored point lights** (lava, forges, bioluminescence): clustered
  forward handles hundreds of lights; Bevy 0.19 moved light clustering to
  the GPU (~20x faster clustering path). Deferred's classic advantage —
  decoupling lighting cost from geometry cost under heavy overdraw — buys
  little in a voxel world whose culled meshes have near-zero overdraw for
  opaques.
- **Real darkness, no ambient floor**: a lighting-model property, orthogonal
  to pipeline shape. Forward keeps per-material control of it (SSS,
  emission) without squeezing those through G-buffer channels.
- **POM terrain + LabPBR materials**: parallax and its self-shadowing want
  the material's own textures and tangent frame in one shader — natural in
  forward, painful through a G-buffer (POM must run before depth/normals are
  final, i.e. in the geometry pass anyway).
- **Blended granular surfaces (S8 mixtures)**: heightlerp splat blending of
  several material texture triplets per surface needs arbitrary per-material
  data in the surface shader. Deferred would force either a fat G-buffer
  (blend results written per channel) or blend-in-lighting hacks.
- **Toggleable sun shadows + shafts, haze, water, post**: all of these hang
  off pass boundaries (shadow maps, depth, screen color), identical work in
  either shape.
- **Pack overridability**: Iris proves packs don't need *our* renderer to be
  deferred — they need **named stages with stable inputs**. We promise stage
  interfaces (uniforms + textures), not a G-buffer memory layout, which is
  exactly the part of a deferred design that ossifies worst.
- **MSAA / transparency**: forward keeps both simple; deferred forfeits MSAA
  and needs a separate forward path for translucents anyway (water is
  first-class here).

What Bevy 0.19 gives us vs what we own:

| Piece | v0 owner | Trajectory |
|---|---|---|
| Window/surface, camera driver, view management | Bevy | keep |
| Opaque/translucent phases, clustered lights, prepass | Bevy | keep until terrain-material hook lands, then our material within Bevy's phases |
| Terrain material/shader | Bevy `StandardMaterial` (vertex colors) | ours (hook `opaque.terrain`, format 1) |
| Sun shadow cascades | Bevy (off in v0) | Bevy machinery, our knobs + pack hook |
| Sky | Bevy clear color | ours (hook `sky`) |
| Tonemapping | **ours via pack `post` stage** (Bevy's pass set `Tonemapping::None`) | ours |
| Post chain (fog/haze/grade) | **ours: the pack `post` stage (implemented)** | ours |
| Upscaling/present | Bevy | keep |
| Shader loading/validation | **ours: naga-validated runtime packs (implemented)** | ours |

Risk we accept: Bevy 0.19 replaced the render graph with ECS schedules
(`Core3d` schedule + `Core3dSystems` sets), so "insert a pass" is now
"add a system between two sets". If Bevy churns this again, only
`dc-client/src/poststage.rs` (and future stage plumbing) is exposed; the
pack-facing contract below is renderer-agnostic on purpose.

## 2. Pass list and hook points

Frame order, with hook status. **Hook names are the contract**; a stage a
pack does not override runs the default pack's implementation of it.

| # | Stage | Contents | Pack hook (format) |
|---|---|---|---|
| 1 | `shadow` | sun cascade shadow maps, toggleable | reserved (≥1) |
| 2 | `prepass` | depth (reverse-Z) + view normals + motion | read-only input, never overridable |
| 3 | `sky` | sky dome/atmosphere color | reserved (≥1) |
| 4 | `opaque` | terrain + entities, clustered forward; the terrain surface shader (LabPBR + splat, § 5) is the hook | reserved (≥1) |
| 5 | `water` | translucent pass: animated surface, depth fog, shore blend | reserved (≥1) |
| 6 | `volumetrics` | light shafts, ground fog; toggleable, degrades to nothing | reserved (≥2) |
| 7 | `post` | aerial perspective/haze, grade, tonemap — one fullscreen pass | **live (0) — implemented** |
| 8 | `ui` | Bevy UI, after all grading | never overridable |

Intermediate targets promised to packs, by stage availability:

- `scene color` — the HDR-capable view target (ping-ponged per post-style
  stage, Bevy `ViewTarget::post_process_write`).
- `scene depth` — reverse-Z (1.0 near, 0.0 sky), single-sample in v0.
- later formats add: shadow maps (`shadow`), view normals (`prepass`),
  water depth (`water`). Not exposed in format 0.

Stability promises:

1. A stage name, its position in frame order, and its declared inputs are
   frozen within a hook format.
2. New stages/inputs may be **added** in a new format; existing ones never
   change meaning silently.
3. A pack states the one format it targets; the loader refuses any other
   (no "probably compatible" loading — Iris's compatibility heuristics are a
   tarpit we opt out of).
4. The default pack is maintained in-tree against the current format and is
   the fallback for every rejected pack.

## 3. Pack manifest and loading

A pack is a directory:

```
assets/packs/<name>/
  pack.toml       -- manifest
  post.wgsl       -- one WGSL file per overridden stage
```

`pack.toml`, hook format 0:

```toml
[pack]
name = "dusk"     # display name
format = 0        # hook-surface version targeted; exact match required

[stages]
post = "post.wgsl"  # stage -> file, relative to the manifest
```

Rules (all enforced by `dc-client/src/shaderpack.rs`, all tested):

- **Selection**: `--pack <name>` resolves `assets/packs/<name>`; a selector
  containing a path separator is used as a directory path; no flag loads
  `assets/packs/default`.
- **Composition**: each stage compiles as `<stage prelude> + <pack file>`.
  The prelude (in-tree, versioned with the format) owns the bind group
  declarations, world-state struct, helpers, and the actual entry point; the
  pack provides one well-known function (for `post`:
  `fn dc_post(frag: DcPostIn) -> vec4<f32>`). Packs therefore cannot
  redeclare bindings, and interface drift is a compile error, not a render
  glitch.
- **Validation**: the composed WGSL runs through naga (same major version
  as the renderer's wgpu) — parse, full validation, entry-point presence —
  at load time, before any GPU object exists.
- **Strictness**: unknown manifest keys and unknown stage names are errors.
  A stage we cannot honor must fail the whole pack loudly; silently dropping
  part of a pack's look is worse than rejecting it.
- **Fallback**: any failure (missing dir, bad manifest, wrong format, WGSL
  error) logs one actionable line (with naga's source-context diagnostics)
  and falls back to the **built-in** default pack (compiled into the binary
  via `include_str!`), so even a vandalized `assets/` cannot crash or blank
  the client. A pack failure is never a panic.
- **Omitted stages** fall back per-stage to the default pack's source.

## 4. Stage interface: `post` (format 0, implemented)

Everything a `post` stage sees, verbatim from
`dc-client/src/shaders/post_prelude.wgsl` (the prelude is the normative
copy):

Bind group 0:

| Binding | Name | Type | Contents |
|---|---|---|---|
| 0 | `dc_scene` | `texture_2d<f32>` | scene color (linear) |
| 1 | `dc_scene_sampler` | `sampler` | linear filtering |
| 2 | `dc_depth` | `texture_2d<f32>` | scene depth, reverse-Z, single-sample |
| 3 | `dc_world` | `uniform DcWorldState` | world state, below |

`DcWorldState` (vec4-packed; packing is part of the contract):

| Field | Contents |
|---|---|
| `sun_dir_time.xyz` | normalized direction toward the sun, world space |
| `sun_dir_time.w` | time of day [0,1): 0 = midnight, 0.5 = noon |
| `fog_color.rgb` | haze/fog color, linear |
| `fog_color.a` | sky-haze amount [0,1] |
| `fog_params.x/.y` | fog start / end (meters, view depth) |
| `fog_params.z` | weather wetness [0,1] |
| `fog_params.w` | camera near plane (m) — depth linearization |

The prelude's wrapper entry point samples these once and calls the pack's
`dc_post` with plain values (`uv`, linear `scene` color, linearized
`view_depth_m`, `is_sky`), plus helpers `dc_fog_factor` and `dc_luminance`.
Growth within format 0 is prohibited; format 1 may extend `DcWorldState`
(append-only) and add bindings (append-only), because packs compile against
the prelude they ship-target, not a copy of it.

World-state values are set by the app on the camera (`PostStage` component)
and extracted per frame; when weather/day-cycle systems land they write the
same component — the shader contract does not change.

## 5. Material texture contract (spec'd for the `opaque` hook, format ≥1)

Adopted verbatim from visuals.md: **LabPBR-compatible three-texture
packing** (LabPBR v1.3 conventions; this buys the existing artist/tooling
ecosystem):

| Texture | R | G | B | A |
|---|---|---|---|---|
| basecolor | albedo R | albedo G | albedo B | opacity |
| normal | normal X | normal Y (reconstruct Z = `sqrt(1 - x² - y²)`) | ambient occlusion | height (drives POM) |
| specular | perceptual smoothness (`roughness = (1 - s)²`) | F0 (0–229 linear; 230–255 predefined metals; 255 = albedo-as-F0) | 0–64 porosity / 65–255 SSS | emission (0–254; 255 unused) |

Notes tying this to the sim:

- **Porosity is sim-driven**: the artist channel is the *base* porosity of
  the material; the per-voxel effective porosity from the materials system
  (S8: structure fill, packed pores) modulates it. Rain-darkening and
  wetness respond to simulated state, not artist guesses.
- **Emission drives colored light emitters** (the future colored-light
  store reads the same channel).

Texture array strategy:

- One `texture_2d_array` per triplet member (basecolor / normal / specular
  arrays), **layer index = material id**. Fixed resolution per pack — 16×16
  or 32×32 (one of, never mixed) — declared in the manifest when the
  `opaque` hook lands. Arrays keep one bind group for all terrain and make
  the splat blend a per-fragment layer pick, not a texture switch.

Mixture/splat inputs (from S8's model): a surface voxel exposes up to a few
materials (dominant structure + debris/pore contributions). The mesher
emits per-vertex splat data; the terrain shader blends the LabPBR triplets
with **height-driven contrast blending** (heightlerp: leaves poke through
sand; no alpha mush) using the normal-alpha height channel.

**Per-voxel sim data path — DECIDED for v0: vertex attributes.** The mesher
already rebuilds a chunk's mesh on any voxel change, and S8's sim data
(mixture composition, porosity, emission) only changes on events that dirty
the mesh anyway. So v0 packs per-vertex: material id + splat weights + one
packed u32 of sim channels (porosity, wetness bias, emission). Rejected for
v0: a per-chunk storage buffer indexed by voxel (better when sim channels
update *faster* than remeshing — revisit if weather wetness ever becomes
per-voxel rather than the global uniform), and a material-index texture
(3D texture per chunk: pays VRAM for interior voxels that meshes never
show). This decision binds the mesher and the `opaque` hook, not format 0.

## 6. Reserved hooks — spec sketches and open questions

- **`shadow`** (sun cascades + shafts feed): Bevy cascade machinery with our
  knobs (resolution, cascade count, off-switch — chasm shafts must degrade
  gracefully). Pack hook: shadow-space vertex displacement (Iris's shadow
  program lesson: packs want to bend the shadow pass, e.g. waving foliage).
  Open: cascade layout under cubic chunks with no bounded world height;
  does the far field cast?
- **`sky`**: full replacement hook (dome shader given sun/moon/time/weather).
  Open: is the sky a real pass or folded into `post` for pixels with
  `is_sky` (v0 packs already fake it there)?
- **`opaque.terrain`**: the LabPBR + splat + POM surface shader (§ 5) as a
  replaceable material body, prelude-composed like `post`. Open: how much of
  Bevy's PBR lighting glue is stable enough to expose in a prelude; bindless
  texture arrays vs plain arrays on DX12/Metal.
- **`water`**: translucent pass with its own depth read (shore blend, depth
  fog); pixelated dynamic detail per visuals.md. Open: refraction target
  (needs opaque color copy — an extra promised intermediate), and whether
  water writes depth for `volumetrics`.
- **`volumetrics`**: shafts/ground fog between `water` and `post`, reading
  shadow maps + depth; toggleable; budget decides raymarch vs layered fake.
  Open: half-res + upsample contract (does the pack see the upsampler?).
- **Colored light storage** is a lighting-spike decision (flood-fill voxel
  light vs many clustered point lights vs both); the pack contract only
  promises the *result* (lit scene color + emission channel), so it does not
  block on this.

## 7. Versioning

- `HOOK_FORMAT` (currently **0**) lives in `dc-client/src/shaderpack.rs`
  and in every `pack.toml`. Exact match required.
- A format bump ships: updated preludes, updated default pack, a migration
  note in this file. Old packs keep working only if we ship a shim prelude
  for their format (not promised; decide per bump).
- The portability gate (below) runs on every format's in-tree packs.

## 8. Portability gate

Dev is Windows-only; the contract is not. Every WGSL entry point of every
in-tree pack must pass, in CI (`cargo test -p dc-client`,
`shaderpack::tests::portability`):

1. naga parse + full validation (the load-time path, exactly);
2. translation to HLSL (DX12, SM 5.1);
3. translation to SPIR-V (Vulkan);
4. translation to MSL (Metal 2.0).

The naga crate version is kept in lockstep with the wgpu major that Bevy
pins (29.x for Bevy 0.19), so "validates in the loader" and "compiles in
the renderer" cannot drift apart.
