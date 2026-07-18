// deepcraft "dusk" pack — post stage (hook format 0).
//
// A moody, desaturated grade: dropped exposure, violet-blue dusk haze that
// closes in earlier than the default fog range, partial desaturation, a cool
// color cast, and an ACES-style shoulder that crushes shadows. Visibly and
// deliberately different from the default pack — the point of the demo.

// Narkowicz's ACES filmic fit.
fn dusk_tonemap(c: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let d = 2.43;
    let e = 0.59;
    let f = 0.14;
    return clamp((c * (a * c + b)) / (c * (d * c + e) + f), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn dc_post(frag: DcPostIn) -> vec4<f32> {
    // Dusk swallows light: drop exposure before anything else.
    var color = frag.scene.rgb * 0.45;

    // Violet-blue haze, denser than the daytime range: pull the start in and
    // saturate well before the default fog end. The sky goes almost fully to
    // the dusk color — the sun is below the ridgeline.
    let dusk_haze = vec3<f32>(0.16, 0.14, 0.24);
    let fog = smoothstep(
        dc_world.fog_params.x * 0.4,
        dc_world.fog_params.y * 0.7,
        frag.view_depth_m,
    );
    color = mix(color, dusk_haze, max(fog * (1.0 - frag.is_sky), frag.is_sky * 0.92));

    // Desaturate toward isolation, then cool the cast.
    let gray = vec3<f32>(dc_luminance(color));
    color = mix(gray, color, 0.45);
    color = color * vec3<f32>(0.90, 0.94, 1.10);

    return vec4<f32>(dusk_tonemap(color), frag.scene.a);
}
