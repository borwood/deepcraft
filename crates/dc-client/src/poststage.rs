//! The `post` stage of the S4 hook surface, wired into Bevy 0.19's Core3d
//! schedule (docs/rendering/PIPELINE.md § Pass list).
//!
//! Bevy 0.19 replaced the render *graph* with plain ECS schedules: a "pass"
//! is a system added to the [`Core3d`] schedule inside one of the
//! [`Core3dSystems`] sets. This plugin adds exactly one such system —
//! [`post_stage`] — in `Core3dSystems::PostProcess`, after Bevy's own
//! `tonemapping` (inert here: the camera is not HDR and uses
//! `Tonemapping::None`; the pack owns the curve per visuals.md) and before
//! `upscaling` (which runs after the whole set). The pass ping-pongs the
//! view target via [`ViewTarget::post_process_write`] and draws one
//! fullscreen triangle with the pack's composed fragment shader.
//!
//! The pack's WGSL was already validated through naga at load time
//! (shaderpack.rs); a shader that somehow still fails pipeline compilation
//! leaves the cached pipeline unready and the pass simply skips — degraded
//! visuals, never a crash.

use bevy::asset::Handle;
use bevy::camera::{Camera, Projection};
use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::schedule::{Core3d, Core3dSystems};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer_sized};
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FilterMode, FragmentState, LoadOp, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline,
    SpecializedRenderPipelines, StoreOp, TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::sync_component::SyncComponent;
use bevy::render::view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget};
use bevy::render::{GpuResourceAppExt, Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;

use crate::shaderpack;

/// World state the post stage sees, set on the camera by the app (main
/// world). Extracted every frame into [`PostStageUniform`] — the WGSL-side
/// layout is `DcWorldState` in shaders/post_prelude.wgsl and its packing is
/// part of the hook format 0 contract.
#[derive(Component, Clone)]
pub struct PostStage {
    /// Time of day in [0, 1): 0.0 = midnight, 0.5 = noon.
    pub time_of_day: f32,
    /// Normalized direction toward the sun, world space.
    pub sun_dir: Vec3,
    /// Haze/fog color, linear.
    pub fog_color: Vec3,
    /// How strongly the sky is pulled toward the haze color, [0, 1].
    pub sky_haze: f32,
    /// Fog start / end, meters of view depth.
    pub fog_start_m: f32,
    pub fog_end_m: f32,
    /// Weather wetness state, [0, 1].
    pub wetness: f32,
}

impl Default for PostStage {
    fn default() -> Self {
        Self {
            time_of_day: 0.35,
            sun_dir: Vec3::Y,
            fog_color: Vec3::new(0.72, 0.80, 0.94),
            sky_haze: 0.35,
            fog_start_m: 150.0,
            fog_end_m: 1100.0,
            wetness: 0.0,
        }
    }
}

impl SyncComponent for PostStage {
    type Target = PostStageUniform;
}

impl ExtractComponent for PostStage {
    type QueryData = (&'static Self, &'static Projection);
    type QueryFilter = With<Camera>;
    type Out = PostStageUniform;

    fn extract_component((stage, projection): QueryItem<Self::QueryData>) -> Option<Self::Out> {
        let near = match projection {
            Projection::Perspective(p) => p.near,
            Projection::Orthographic(o) => o.near,
            Projection::Custom(_) => 0.1,
        };
        Some(PostStageUniform {
            sun_dir_time: stage.sun_dir.normalize_or_zero().extend(stage.time_of_day),
            fog_color: stage.fog_color.extend(stage.sky_haze),
            fog_params: Vec4::new(stage.fog_start_m, stage.fog_end_m, stage.wetness, near),
        })
    }
}

/// GPU-side mirror of `DcWorldState` (see the prelude for field semantics).
#[doc(hidden)]
#[derive(Component, ShaderType, Clone)]
pub struct PostStageUniform {
    sun_dir_time: Vec4,
    fog_color: Vec4,
    fog_params: Vec4,
}

/// Handle to the active pack's composed post-stage shader.
#[derive(Resource, Clone)]
struct PostStageShader(Handle<Shader>);

/// Loads the selected shader pack (falling back to the built-in default on
/// any error) and installs its post stage.
pub struct PostStagePlugin {
    /// The `--pack` CLI argument, if any.
    pub pack_selector: Option<String>,
}

impl Plugin for PostStagePlugin {
    fn build(&self, app: &mut App) {
        // Loaded here (not in main) so the log plugin is already up and the
        // accept/fall-back line is visible to the user.
        let pack = shaderpack::load_pack_or_default(self.pack_selector.as_deref());
        let shader =
            Shader::from_wgsl(pack.post_wgsl, format!("dc-pack://{}/post.wgsl", pack.name));
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(shader);

        app.add_plugins((
            ExtractComponentPlugin::<PostStage>::default(),
            UniformComponentPlugin::<PostStageUniform>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // The handle is inserted directly into the render world (rather than
        // extracted) so it exists by the time `RenderStartup` runs.
        render_app.insert_resource(PostStageShader(handle));

        render_app
            .init_gpu_resource::<SpecializedRenderPipelines<PostStagePipeline>>()
            .add_systems(RenderStartup, init_post_stage_pipeline)
            .add_systems(
                Render,
                prepare_post_stage_pipelines.in_set(RenderSystems::Prepare),
            )
            .add_systems(
                Core3d,
                post_stage
                    .after(tonemapping)
                    .in_set(Core3dSystems::PostProcess),
            );
    }
}

#[derive(Resource)]
struct PostStagePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    fullscreen_shader: FullscreenShader,
    fragment_shader: Handle<Shader>,
}

fn init_post_stage_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
    shader: Res<PostStageShader>,
) {
    // Bind group 0 of the post-stage contract, in prelude binding order.
    let layout = BindGroupLayoutDescriptor::new(
        "dc_post_stage_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // @binding(0) dc_scene
                texture_2d(TextureSampleType::Float { filterable: true }),
                // @binding(1) dc_scene_sampler
                sampler(SamplerBindingType::Filtering),
                // @binding(2) dc_depth (reverse-Z, non-filterable)
                texture_2d(TextureSampleType::Float { filterable: false }),
                // @binding(3) dc_world
                uniform_buffer_sized(false, Some(PostStageUniform::min_size())),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..Default::default()
    });

    commands.insert_resource(PostStagePipeline {
        layout,
        sampler,
        fullscreen_shader: fullscreen_shader.clone(),
        fragment_shader: shader.0.clone(),
    });
}

impl SpecializedRenderPipeline for PostStagePipeline {
    type Key = TextureFormat;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("dc_post_stage_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                entry_point: Some(shaderpack::POST_ENTRY_POINT.into()),
                targets: vec![Some(ColorTargetState {
                    format: key,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

#[derive(Component)]
struct PostStagePipelineId(CachedRenderPipelineId);

fn prepare_post_stage_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PostStagePipeline>>,
    pipeline: Res<PostStagePipeline>,
    views: Query<(Entity, &ExtractedView), With<PostStageUniform>>,
) {
    for (entity, view) in &views {
        let id = pipelines.specialize(&pipeline_cache, &pipeline, view.target_format);
        commands.entity(entity).insert(PostStagePipelineId(id));
    }
}

/// The pass itself: one fullscreen triangle, pack fragment shader, view
/// target ping-pong. Runs per camera in `Core3dSystems::PostProcess`.
fn post_stage(
    view: ViewQuery<(&ViewTarget, &ViewDepthTexture, &PostStagePipelineId, &Msaa)>,
    pipeline: Res<PostStagePipeline>,
    pipeline_cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<PostStageUniform>>,
    mut ctx: RenderContext,
) {
    let (view_target, depth, pipeline_id, msaa) = view.into_inner();

    // The v0 contract binds a single-sample depth texture; the S4 camera
    // runs Msaa::Off. MSAA views skip the stage rather than mis-bind.
    if msaa.samples() != 1 {
        return;
    }
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
        return; // Still compiling (or failed): skip, never crash.
    };
    let Some(uniform_binding) = uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = ctx.render_device().create_bind_group(
        Some("dc_post_stage_bind_group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipeline.sampler,
            depth.view(),
            uniform_binding,
        )),
    );

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("dc_post_stage"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post_process.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Default::default()),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_render_pipeline(render_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..3, 0..1);
}
