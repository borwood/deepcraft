// deepcraft `--edges` dev diagnostic — crease/silhouette outlining.
//
// A renderer-owned post pass (edgepass.rs), NOT a shader-pack stage: it runs
// after the pack's `post` stage on the final composited scene color, so it is
// never something a content pack has to implement, and when `--edges` is off
// the pass is not scheduled at all (the pack render path is byte-identical).
//
// Why crease/silhouette and not a per-cube wireframe: on single-material
// terrain `--fullbright` paints every face one flat colour, so a terraced
// hillside is a featureless field (journal/0030, corrections #18). What makes a
// bench legible is the STEP — a normal discontinuity (top face -> riser) and a
// depth discontinuity (silhouette) — not the voxel grid, which aliases into
// moiré once a voxel is sub-pixel at kilometre range. So we outline exactly
// those two discontinuities and fade the whole term with distance.
//
// No normal buffer is bound in hook format 0, so normals are RECONSTRUCTED from
// depth: linear view depth -> view-space position (using the camera's own
// projection scale, passed in the uniform) -> a forward-difference and a
// backward-difference surface normal. On any plane (flat OR sloped) both
// normals equal the plane normal, so slopes never draw an edge; at a crease the
// two differ, which is the bench. This is why the raw depth Laplacian alone is
// not used for creases: view depth is not linear in screen space on a slope, so
// its curvature would false-trigger on every hillside.

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// x: projection x-scale (= cot(fov/2)/aspect); y: projection y-scale
// (= cot(fov/2)); z: camera near plane (m), to linearize reverse-Z depth;
// w: unused.
struct EdgeUniform {
    proj: vec4<f32>,
}

@group(0) @binding(0) var edge_scene: texture_2d<f32>;
@group(0) @binding(1) var edge_sampler: sampler;
@group(0) @binding(2) var edge_depth: texture_2d<f32>;
@group(0) @binding(3) var<uniform> edge: EdgeUniform;

// --- Tunable constants (reported in the journal entry; dev-only, so a change
// here is a diagnostic-appearance choice, not a content decision). ---

// Crease term: edge = 1 - dot(forward_normal, backward_normal). ~0.15 is a
// ~32 degrees orientation break; a voxel bench is 90 degrees (== 1.0). Gentle
// terrain undulation (a few degrees) stays below CREASE_LO and draws nothing.
const CREASE_LO: f32 = 0.15;
const CREASE_HI: f32 = 0.60;

// Silhouette term: relative Laplacian of linear depth. Rejects constant depth
// gradients (slopes); spikes at an occluding contour between two geometry
// depths. Deliberately conservative — the sky silhouette is handled separately.
const DEPTH_LO: f32 = 0.05;
const DEPTH_HI: f32 = 0.20;

// Distance fade: full-strength edges out to FADE_START_M, linearly to zero at
// FADE_END_M. Beyond FADE_END_M no edge is drawn AT ALL, which is what kills
// far-field moiré by construction (a sub-pixel voxel bench at 3.5 km simply
// gets no outline rather than a shimmering one).
const FADE_START_M: f32 = 350.0;
const FADE_END_M: f32 = 1400.0;

// Appearance: how dark the outline goes, and its colour.
const EDGE_STRENGTH: f32 = 0.85;
const EDGE_COLOR: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

const SKY_RAW: f32 = 1.0e-8;

fn load_raw(coord: vec2<i32>, dims: vec2<i32>) -> f32 {
    let c = clamp(coord, vec2<i32>(0), dims - vec2<i32>(1));
    return textureLoad(edge_depth, c, 0).x;
}

// Linear view depth in meters from a reverse-Z raw sample (near / raw).
fn lin_depth(raw: f32, near: f32) -> f32 {
    return near / max(raw, SKY_RAW);
}

// View-space position for a pixel at linear depth z. The sign of z is
// irrelevant to the normal comparisons below (a global handedness flip cancels
// in the dot product), so z is carried positive.
fn view_pos(px: vec2<f32>, dims: vec2<f32>, z: f32) -> vec3<f32> {
    let uv = px / dims;
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    return vec3<f32>(ndc.x * z / edge.proj.x, ndc.y * z / edge.proj.y, z);
}

@fragment
fn dc_edge_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(edge_scene, edge_sampler, in.uv);
    let near = edge.proj.z;

    let dims_i = vec2<i32>(textureDimensions(edge_depth));
    let dims_f = vec2<f32>(dims_i);
    let p = vec2<i32>(in.position.xy);
    let fp = in.position.xy;

    let raw_c = load_raw(p, dims_i);
    // Interior sky pixels never outline — the geometry side of a silhouette
    // draws the line, so the sky stays clean.
    if (raw_c < SKY_RAW) {
        return scene;
    }
    let zc = lin_depth(raw_c, near);

    // Four-neighbour cross.
    let ol = vec2<i32>(-1, 0);
    let or = vec2<i32>(1, 0);
    let ou = vec2<i32>(0, -1);
    let od = vec2<i32>(0, 1);

    let raw_l = load_raw(p + ol, dims_i);
    let raw_r = load_raw(p + or, dims_i);
    let raw_u = load_raw(p + ou, dims_i);
    let raw_d = load_raw(p + od, dims_i);

    let sky_l = raw_l < SKY_RAW;
    let sky_r = raw_r < SKY_RAW;
    let sky_u = raw_u < SKY_RAW;
    let sky_d = raw_d < SKY_RAW;

    // A geometry pixel bordering sky is a silhouette against the horizon.
    var sky_edge = 0.0;
    if (sky_l || sky_r || sky_u || sky_d) {
        sky_edge = 1.0;
    }

    // For the geometry-vs-geometry terms, treat a sky neighbour as coincident
    // with the centre so it contributes nothing here (sky_edge already has it).
    let zl = select(lin_depth(raw_l, near), zc, sky_l);
    let zr = select(lin_depth(raw_r, near), zc, sky_r);
    let zu = select(lin_depth(raw_u, near), zc, sky_u);
    let zd = select(lin_depth(raw_d, near), zc, sky_d);

    // Silhouette (occluding contour) between two geometry depths: relative
    // Laplacian rejects the constant gradient of a slope.
    let de = (abs(zl + zr - 2.0 * zc) + abs(zu + zd - 2.0 * zc)) / zc;
    let depth_edge = smoothstep(DEPTH_LO, DEPTH_HI, de);

    // Crease: reconstructed forward/backward normals disagree only where the
    // surface orientation breaks (a bench), never on a plane.
    let pc = view_pos(fp, dims_f, zc);
    let pl = view_pos(fp + vec2<f32>(ol), dims_f, zl);
    let pr = view_pos(fp + vec2<f32>(or), dims_f, zr);
    let pu = view_pos(fp + vec2<f32>(ou), dims_f, zu);
    let pd = view_pos(fp + vec2<f32>(od), dims_f, zd);
    let n_fwd = normalize(cross(pr - pc, pu - pc));
    let n_bwd = normalize(cross(pc - pl, pc - pd));
    let crease_raw = 1.0 - dot(n_fwd, n_bwd);
    let crease = smoothstep(CREASE_LO, CREASE_HI, crease_raw);

    let edge_amt = max(max(depth_edge, crease), sky_edge);

    // Fade with distance so far-field edges soften instead of shimmering.
    let fade = 1.0 - smoothstep(FADE_START_M, FADE_END_M, zc);
    let e = edge_amt * fade * EDGE_STRENGTH;

    return vec4<f32>(mix(scene.rgb, EDGE_COLOR, e), scene.a);
}
