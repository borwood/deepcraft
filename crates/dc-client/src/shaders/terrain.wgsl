// deepcraft terrain surface shader (ROADMAP PBR-1, docs/rendering/PIPELINE.md § 5).
//
// A custom Forward+ material: LabPBR three-texture packing sampled from three
// texture_2d_arrays (basecolor / normal+AO / specular), layer index = material
// id. A face carries up to four material layers + splat weights per vertex
// (the mesher's top-N constituents); this shader blends them with
// height/AO-driven contrast (heightlerp) so grains poke through rather than
// alpha-mush (visuals.md § Material/texture model). Lighting is a single
// directional sun + hemispherical ambient (PBR-2 adds shadows, point lights,
// tonemap/HDR; never GI). Fullbright is a *different* material (unlit vertex
// color) — this shader is the lit path only.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
}

struct TerrainLighting {
    // xyz: normalized direction toward the sun (world space); w unused.
    sun_dir: vec4<f32>,
    // rgb: sun color; a: sun intensity.
    sun_color: vec4<f32>,
    // rgb: hemispherical ambient toward the sky (up); a: unused.
    sky_color: vec4<f32>,
    // rgb: hemispherical ambient toward the ground (down); a: unused.
    ground_color: vec4<f32>,
}

@group(3) @binding(0) var<uniform> lighting: TerrainLighting;
@group(3) @binding(1) var basecolor_tex: texture_2d_array<f32>;
@group(3) @binding(2) var normal_tex: texture_2d_array<f32>;
@group(3) @binding(3) var specular_tex: texture_2d_array<f32>;
@group(3) @binding(4) var atlas_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) layers: vec4<u32>,
    @location(4) weights: vec4<f32>,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) layers: vec4<u32>,
    @location(4) weights: vec4<f32>,
}

@vertex
fn vertex(v: Vertex) -> VsOut {
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
    let world_pos = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(v.position, 1.0),
    );
    var out: VsOut;
    out.clip = position_world_to_clip(world_pos.xyz);
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize(mesh_functions::mesh_normal_local_to_world(v.normal, v.instance_index));
    out.uv = v.uv;
    out.layers = v.layers;
    out.weights = v.weights;
    return out;
}

// A stable tangent frame for an axis-aligned voxel face normal. The helper
// must never be parallel to n: world-up works for every side face (±X AND
// ±Z — a (0,0,1) helper is parallel to ±Z normals, whose cross is zero and
// whose normalize is NaN, which painted every ±Z face black in walk 14);
// X serves the top/bottom faces where up is parallel instead.
fn tangent_frame(n: vec3<f32>) -> mat3x3<f32> {
    let helper = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.y) < 0.99);
    let t = normalize(cross(helper, n));
    let b = cross(n, t);
    return mat3x3<f32>(t, b, n);
}

@fragment
fn fragment(in: VsOut) -> @location(0) vec4<f32> {
    let uv = fract(in.uv);

    // Heightlerp: elevation per layer = splat weight + texture height; the
    // winning layers within a small band blend, so grains poke through with a
    // crisp pixel edge instead of alpha-mushing (visuals.md).
    var elev = vec4<f32>(-1.0, -1.0, -1.0, -1.0);
    var maxe = -1.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (in.weights[i] > 0.0) {
            let hgt = textureSampleLevel(normal_tex, atlas_sampler, uv, i32(in.layers[i]), 0.0).a;
            let e = in.weights[i] + hgt * 0.6;
            elev[i] = e;
            maxe = max(maxe, e);
        }
    }
    let band = 0.25;
    var bw = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (in.weights[i] > 0.0) {
            let b = max(0.0, elev[i] - (maxe - band));
            bw[i] = b;
            wsum = wsum + b;
        }
    }
    if (wsum <= 0.0) {
        // Degenerate (all-equal heights at weight 0 edge): fall back to weights.
        bw = in.weights;
        wsum = in.weights.x + in.weights.y + in.weights.z + in.weights.w;
    }

    var base = vec3<f32>(0.0);
    var tnrm = vec2<f32>(0.0);
    var ao = 0.0;
    var smoothness = 0.0;
    var f0g = 0.0;
    var emission = 0.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (bw[i] > 0.0) {
            let w = bw[i] / wsum;
            let layer = i32(in.layers[i]);
            let bc = textureSampleLevel(basecolor_tex, atlas_sampler, uv, layer, 0.0);
            let nm = textureSampleLevel(normal_tex, atlas_sampler, uv, layer, 0.0);
            let sp = textureSampleLevel(specular_tex, atlas_sampler, uv, layer, 0.0);
            base = base + bc.rgb * w;
            tnrm = tnrm + (nm.rg * 2.0 - 1.0) * w;
            ao = ao + nm.b * w;
            smoothness = smoothness + sp.r * w;
            f0g = f0g + sp.g * w;
            // LabPBR emission: 255 == none.
            let em = select(sp.a, 0.0, sp.a >= 254.0 / 255.0);
            emission = emission + em * w;
        }
    }

    // Reconstruct the perturbed world normal from the blended tangent normal.
    let tnz = sqrt(max(0.0, 1.0 - dot(tnrm, tnrm)));
    let tbn = tangent_frame(normalize(in.world_normal));
    let world_n = normalize(tbn * vec3<f32>(tnrm, tnz));

    // Directional sun + hemispherical ambient.
    let l = normalize(lighting.sun_dir.xyz);
    let ndotl = max(dot(world_n, l), 0.0);
    let sun = lighting.sun_color.rgb * lighting.sun_color.a * ndotl;
    let hemi = mix(lighting.ground_color.rgb, lighting.sky_color.rgb, world_n.y * 0.5 + 0.5);
    let ambient = hemi * ao;
    var color = base * (sun + ambient);

    // Cheap Blinn spec from perceptual smoothness + F0 proxy.
    let vdir = normalize(view.world_position - in.world_pos);
    let h = normalize(l + vdir);
    let ndoth = max(dot(world_n, h), 0.0);
    let gloss = mix(4.0, 96.0, smoothness);
    let f0 = mix(0.03, 0.85, f0g);
    let spec = f0 * pow(ndoth, gloss) * ndotl;
    color = color + lighting.sun_color.rgb * lighting.sun_color.a * spec;

    // Emission is albedo-colored (LabPBR: emitters read the basecolor).
    color = color + base * emission * 2.0;

    return vec4<f32>(color, 1.0);
}
