//! The `--edges` dev diagnostic: a renderer-owned crease/silhouette outlining
//! pass, run after the pack's `post` stage (poststage.rs) on the final scene.
//!
//! **Why a separate pass and not a prelude term.** Edges are a *renderer*
//! diagnostic, not part of any content pack's look. Folding them into the
//! `post` prelude would (a) force the term — and a new uniform field — into the
//! frozen hook-format-0 contract every pack compiles against, and (b) add ALU
//! to every pack's fragment shader even with edges off. A standalone pass keeps
//! the pack contract untouched, and — crucially for the separation invariant
//! (corrections #18, the 0027 coal control) — when `--edges` is OFF this plugin
//! adds no systems at all, so the render path is byte-identical to today. There
//! is no "pass-through pass": there is simply no pass.
//!
//! It composes with either upstream: `--fullbright --edges` (benches legible on
//! the flat-albedo field) and lit `--edges` both work, because the pass reads
//! only the composited scene color plus the prepass depth the post stage
//! already binds. The edge shader (shaders/edges.wgsl) reconstructs normals
//! from depth — no normal buffer exists in format 0 — and fades edges out with
//! distance so far-field voxel benches soften instead of aliasing into moiré.

use bevy::asset::{Handle, load_internal_asset, uuid_handle};
use bevy::camera::{Camera, Projection};
use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::schedule::{Core3d, Core3dSystems};
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

use crate::poststage::PostStageSet;

/// Embedded edge shader (in-tree, compiled into the binary — no runtime asset
/// dependency; same robustness as the terrain material's embedded shaders).
const EDGE_SHADER: Handle<Shader> = uuid_handle!("3e5d7c9a-2b4f-4d1e-9a6c-7f8b0d1e2a3c");

/// The fragment entry point in shaders/edges.wgsl.
const EDGE_ENTRY_POINT: &str = "dc_edge_main";

/// Present on the camera to opt it into the edge pass. Carries the projection
/// scale + near plane the shader needs to reconstruct view-space position (and
/// thus normals) from depth. Extracted every frame so a resized window's aspect
/// stays correct.
#[derive(Component, Clone, Default)]
pub struct EdgeParams;

impl SyncComponent for EdgeParams {
    type Target = EdgeUniform;
}

impl ExtractComponent for EdgeParams {
    type QueryData = &'static Projection;
    type QueryFilter = With<Camera>;
    type Out = EdgeUniform;

    fn extract_component(projection: QueryItem<Self::QueryData>) -> Option<Self::Out> {
        // proj.y = cot(fov/2); proj.x = proj.y / aspect. These are the two
        // diagonal scales of a perspective matrix, enough to turn a pixel + its
        // linear depth into a metric view-space position.
        let (proj_x, proj_y, near) = match projection {
            Projection::Perspective(p) => {
                let y = 1.0 / (p.fov * 0.5).tan();
                (y / p.aspect_ratio, y, p.near)
            }
            // Edges are a perspective-camera diagnostic; degrade to an identity
            // reconstruction rather than misbehave.
            Projection::Orthographic(o) => (1.0, 1.0, o.near),
            Projection::Custom(_) => (1.0, 1.0, 0.1),
        };
        Some(EdgeUniform {
            proj: Vec4::new(proj_x, proj_y, near, 0.0),
        })
    }
}

/// GPU-side mirror of `EdgeUniform` in shaders/edges.wgsl.
#[doc(hidden)]
#[derive(Component, ShaderType, Clone)]
pub struct EdgeUniform {
    proj: Vec4,
}

/// Installs the edge pass. Constructed with `enabled` from the `--edges` flag;
/// when disabled, `build` adds nothing — no extract, no pipeline, no pass — so
/// the render path (and `--fullbright` alone) stays byte-identical.
pub struct EdgePassPlugin {
    pub enabled: bool,
}

impl Plugin for EdgePassPlugin {
    fn build(&self, app: &mut App) {
        if !self.enabled {
            return;
        }

        load_internal_asset!(app, EDGE_SHADER, "shaders/edges.wgsl", Shader::from_wgsl);

        app.add_plugins((
            ExtractComponentPlugin::<EdgeParams>::default(),
            UniformComponentPlugin::<EdgeUniform>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_gpu_resource::<SpecializedRenderPipelines<EdgePipeline>>()
            .add_systems(RenderStartup, init_edge_pipeline)
            .add_systems(
                Render,
                prepare_edge_pipelines.in_set(RenderSystems::Prepare),
            )
            .add_systems(
                Core3d,
                // After the pack's post stage: we outline the final composited
                // picture (in fullbright, that is the flat-albedo field).
                edge_stage
                    .after(PostStageSet)
                    .in_set(Core3dSystems::PostProcess),
            );
    }
}

#[derive(Resource)]
struct EdgePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    fullscreen_shader: FullscreenShader,
}

fn init_edge_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    // Same bind-group shape as the post stage: scene color, sampler, depth,
    // and the edge uniform.
    let layout = BindGroupLayoutDescriptor::new(
        "dc_edge_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer_sized(false, Some(EdgeUniform::min_size())),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..Default::default()
    });

    commands.insert_resource(EdgePipeline {
        layout,
        sampler,
        fullscreen_shader: fullscreen_shader.clone(),
    });
}

impl SpecializedRenderPipeline for EdgePipeline {
    type Key = TextureFormat;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("dc_edge_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: EDGE_SHADER,
                entry_point: Some(EDGE_ENTRY_POINT.into()),
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
struct EdgePipelineId(CachedRenderPipelineId);

fn prepare_edge_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<EdgePipeline>>,
    pipeline: Res<EdgePipeline>,
    views: Query<(Entity, &ExtractedView), With<EdgeUniform>>,
) {
    for (entity, view) in &views {
        let id = pipelines.specialize(&pipeline_cache, &pipeline, view.target_format);
        commands.entity(entity).insert(EdgePipelineId(id));
    }
}

/// The pass: one fullscreen triangle, edge shader, view-target ping-pong. Runs
/// only for cameras carrying `EdgeParams` (extracted to `EdgeUniform`).
fn edge_stage(
    view: ViewQuery<(&ViewTarget, &ViewDepthTexture, &EdgePipelineId, &Msaa)>,
    pipeline: Res<EdgePipeline>,
    pipeline_cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<EdgeUniform>>,
    mut ctx: RenderContext,
) {
    let (view_target, depth, pipeline_id, msaa) = view.into_inner();

    // Single-sample depth binding, matching the post stage's v0 contract.
    if msaa.samples() != 1 {
        return;
    }
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
        return;
    };
    let Some(uniform_binding) = uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = ctx.render_device().create_bind_group(
        Some("dc_edge_bind_group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipeline.sampler,
            depth.view(),
            uniform_binding,
        )),
    );

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("dc_edge_stage"),
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
