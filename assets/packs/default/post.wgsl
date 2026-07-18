// deepcraft default pack — post stage (hook format 0).
//
// Clear-day aerial perspective: geometry hazes toward the fog color with
// view depth (load-bearing for the 1+ km far field — it hides LOD seams,
// docs/design/visuals.md § Atmosphere), and the sky is pulled slightly
// toward the same haze so the far terrain meets the horizon. Tonemapping is
// identity in v0: the skeleton renders LDR; the curve becomes real when HDR
// lands (PIPELINE.md § Open questions).

fn dc_post(frag: DcPostIn) -> vec4<f32> {
    let haze = dc_world.fog_color.rgb;
    let fog = dc_fog_factor(frag.view_depth_m);

    // Haze geometry toward the atmosphere color; leave the sky mostly as-is,
    // blended just enough that the far field dissolves into it.
    let grounded = mix(frag.scene.rgb, haze, fog * (1.0 - frag.is_sky));
    let color = mix(grounded, haze, frag.is_sky * dc_world.fog_color.a);

    return vec4<f32>(color, frag.scene.a);
}
