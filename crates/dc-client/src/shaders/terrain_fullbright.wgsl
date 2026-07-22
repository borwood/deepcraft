// deepcraft fullbright terrain shader (ROADMAP PBR-1 walk-14 ratification,
// journal/0020).
//
// The UNLIT splat variant: the walk protocol's AI-diagnostic register. No
// lighting, no normal/spec/emission, no texture sampling — pure flat albedo from
// the registry palette. A uniform or block face renders its one vertex color
// (dominant material albedo, or the block face color incl. grass's green top). A
// *mixed* face reproduces the old world-anchored 4×4-cell mixture speckle
// (journal/0010) shader-side: each cell picks ONE constituent by a world-anchored
// position hash weighted by the splat fractions, and paints that constituent's
// flat registry albedo. Zero mosaic geometry — the same single-quad splat mesh
// the lit path uses.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}

// One flat albedo per atlas layer. SIZE MUST EQUAL PALETTE_LEN in
// terrain_material.rs (= MATERIAL_COUNT 26 + 4 block-only = 30); the Rust guard
// `palette_len_matches_atlas` keeps them in lockstep.
struct Palette {
    albedo: array<vec4<f32>, 30>,
}

@group(3) @binding(0) var<uniform> palette: Palette;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) layers: vec4<u32>,
    @location(4) weights: vec4<f32>,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) layers: vec4<u32>,
    @location(3) @interpolate(flat) weights: vec4<f32>,
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
    out.color = v.color;
    out.uv = v.uv;
    out.layers = v.layers;
    out.weights = v.weights;
    return out;
}

// Deterministic hash of an integer lattice cell -> [0,1). Mirrors the placeholder
// generator's `_vhash` and the interim CPU dither: a pure function of the cell's
// world coordinate, so the speckle is world-anchored (stable under reload,
// generation order, and floating-origin rebases).
fn cell_hash(cell: vec2<f32>) -> f32 {
    let ix = u32(i32(cell.x));
    let iy = u32(i32(cell.y));
    var n = ix * 374761393u + iy * 668265263u;
    n = (n ^ (n >> 13u)) * 1274126177u;
    n = n ^ (n >> 16u);
    return f32(n & 0xFFFFFFu) / f32(0xFFFFFF);
}

@fragment
fn fragment(in: VsOut) -> @location(0) vec4<f32> {
    // Count active splat slots. A uniform / block face has exactly one -> flat
    // albedo (its vertex color). This is the pure diagnostic register: one flat
    // color per uniform face, no lighting.
    var slots = 0u;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (in.weights[i] > 0.0) {
            slots = slots + 1u;
        }
    }
    if (slots <= 1u) {
        return vec4<f32>(in.color.rgb, 1.0);
    }

    // Mixed face: world-anchored 4×4-cell speckle. The UV is world-anchored in
    // tile units (one tile per voxel), so floor(uv * 4) is a world-stable
    // 4-cells-per-voxel grid — the ratified 4×4-per-face look (journal/0010).
    // Each cell picks one constituent, weighted by the eighths fractions.
    let cell = floor(in.uv * 4.0);
    let h = cell_hash(cell);
    var acc = 0.0;
    var pick = 0u;
    for (var i = 0u; i < 4u; i = i + 1u) {
        acc = acc + in.weights[i];
        if (h < acc) {
            pick = i;
            break;
        }
    }
    return vec4<f32>(palette.albedo[in.layers[pick]].rgb, 1.0);
}
