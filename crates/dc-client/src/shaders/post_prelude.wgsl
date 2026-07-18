// deepcraft post-stage prelude — hook surface v0.
//
// This file is prepended verbatim to a pack's `post` stage source before
// naga validation and pipeline creation (docs/rendering/PIPELINE.md § The
// `post` stage). It is the *entire* interface a post stage sees; everything
// here is frozen for hook format 0. A pack must define exactly one function:
//
//     fn dc_post(frag: DcPostIn) -> vec4<f32>
//
// and may call every helper and read every binding declared below. The
// `@fragment` entry point lives here, not in the pack: the wrapper samples
// the stage inputs once and hands the pack plain values, so packs stay
// portable across backends and cannot bypass the contract surface.

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// World state visible to the post stage (one uniform, std140-safe vec4s).
// Field packing is part of the format-0 contract:
struct DcWorldState {
    // xyz: normalized direction TOWARD the sun (world space).
    // w:   time of day in [0, 1): 0.0 = midnight, 0.5 = noon.
    sun_dir_time: vec4<f32>,
    // rgb: haze/fog color (linear).
    // a:   sky-haze amount in [0, 1] — how strongly the sky itself should be
    //      pulled toward the haze color (packs may reinterpret).
    fog_color: vec4<f32>,
    // x: fog start (meters, view depth); y: fog end (meters).
    // z: wetness in [0, 1] (weather state; 0 = dry).
    // w: camera near plane (meters) — needed to linearize depth.
    fog_params: vec4<f32>,
}

@group(0) @binding(0) var dc_scene: texture_2d<f32>;
@group(0) @binding(1) var dc_scene_sampler: sampler;
// Scene depth (reverse-Z: 1.0 at the near plane, 0.0 at infinity/sky).
@group(0) @binding(2) var dc_depth: texture_2d<f32>;
@group(0) @binding(3) var<uniform> dc_world: DcWorldState;

// Everything a post stage receives per fragment.
struct DcPostIn {
    uv: vec2<f32>,
    // Scene color at uv (linear).
    scene: vec4<f32>,
    // Linearized view depth in meters. Sky pixels report DC_SKY_DEPTH_M.
    view_depth_m: f32,
    // 1.0 for sky pixels (nothing rendered), 0.0 for geometry. An f32 so
    // packs can feed it straight into mix().
    is_sky: f32,
}

// The view depth reported for sky pixels; far beyond any fog end.
const DC_SKY_DEPTH_M: f32 = 1.0e8;

// Fog factor in [0, 1] from the world-state fog range.
fn dc_fog_factor(view_depth_m: f32) -> f32 {
    return smoothstep(dc_world.fog_params.x, dc_world.fog_params.y, view_depth_m);
}

// Rec. 709 relative luminance of a linear color.
fn dc_luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn dc_post_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Sample before any branching so derivatives stay uniform.
    let scene = textureSample(dc_scene, dc_scene_sampler, in.uv);
    let raw_depth = textureLoad(dc_depth, vec2<i32>(in.position.xy), 0).x;

    // Bevy's perspective projection is infinite reverse-Z: depth = near / z.
    let near = dc_world.fog_params.w;
    let sky = raw_depth < 1.0e-8;
    let is_sky = select(0.0, 1.0, sky);
    let view_depth_m = select(near / max(raw_depth, 1.0e-8), DC_SKY_DEPTH_M, sky);

    return dc_post(DcPostIn(in.uv, scene, view_depth_m, is_sky));
}
