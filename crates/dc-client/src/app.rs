//! The interactive S1 walking skeleton: window, chunk streaming, scale
//! switching, floating origin.
//!
//! Floating origin (see docs/spikes/S1-results.md): all simulation positions
//! are f64 **meters** ([`Player::pos_m`], chunk minima via `ChunkPos` x voxel
//! size). The GPU only ever sees positions relative to [`FloatingOrigin`],
//! which is re-snapped to the player whenever they wander 256 m from it.
//! Chunk/camera transforms are recomputed from f64 every frame, so a rebase is
//! just a resource write — nothing else moves.

use std::collections::HashMap;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::Msaa;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use dc_core::{Chunk, ChunkPos, VoxelScale};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::authority::{self, Authority, DirtyChunks};
use crate::bench::BENCH_SEED;
use crate::character;
use crate::console::{self, ConsolePlugin};
use crate::edgepass::{EdgeParams, EdgePassPlugin};
use crate::edit;
use crate::farmesh;
use crate::mcp::{self, McpOptions};
use crate::physdemo;
use crate::player::{self, Player};
use crate::poststage::{PostStage, PostStagePlugin};
use crate::streaming;
use crate::terrain_material::{
    self, FullbrightTerrainMaterial, TerrainMaterial, TerrainMaterialPlugin,
};
// The legacy S1 `TerrainGen` is no longer an ambient client resource: gameplay
// systems answer world questions from the active `Authority` (the S1-fallback
// sweep, journal/0017). It survives inside the authority module (the 3/4-key
// legacy authority) and in the far-mesh's own resource; here it is test-only.
#[cfg(test)]
use crate::worldgen::TerrainGen;
#[cfg(test)]
use dc_core::Block;

/// Distance from the origin (meters) at which we re-snap it to the player.
const ORIGIN_REBASE_M: f64 = 256.0;

/// The active voxel scale. Switched at runtime with keys 2/3/4.
#[derive(Resource, Clone, Copy)]
pub struct CurrentScale {
    pub player_voxels: u32,
    pub scale: VoxelScale,
}

impl CurrentScale {
    pub fn new(player_voxels: u32) -> Self {
        Self {
            player_voxels,
            scale: VoxelScale::from_player_height(PLAYER_HEIGHT_M, player_voxels),
        }
    }
}

/// World-space anchor (f64 meters) that render-space f32 coordinates are
/// measured from.
#[derive(Resource)]
pub struct FloatingOrigin(pub DVec3);

pub struct LoadedChunk {
    pub chunk: Chunk,
    /// Render-only per-voxel material contents (worldgen authority only;
    /// `None` for the S1 terrain authority and for debris-free chunks). The
    /// mesher dithers mixed voxels from these; edits do not update them
    /// (ROADMAP 3c-2, render-only — the block gate keeps stale contents from
    /// bleeding through after an edit).
    pub contents: Option<dc_core::ContentsGrid>,
    /// `None` when the chunk meshed to nothing (all air / fully buried).
    pub entity: Option<Entity>,
}

/// All currently loaded chunks, at the current scale.
///
/// A render/collision **cache** of the authoritative hosted world, never a
/// second source of truth. It answers only for chunks it holds; it does not
/// invent an answer for a chunk it does not — the former generator fallback
/// silently shipped the legacy S1 world into player collision, grounding, edit
/// targeting, physics, and mesh-border culling under the worldgen authority
/// (journal/0015–0017). Every gameplay solidity question now routes through
/// [`Authority::is_solid_voxel`], which lazily generates from the *active*
/// world. See docs/ARCHITECTURE.md § "One world-answer surface".
#[derive(Resource, Default)]
pub struct ChunkMap {
    pub loaded: HashMap<ChunkPos, LoadedChunk>,
}

/// Marks a chunk's render entity; the transform is recomputed from f64 every
/// frame by [`position_chunks`].
#[derive(Component)]
pub struct ChunkEntity(pub ChunkPos);

/// Shared **unlit** fullbright terrain material — the `--fullbright` diagnostic
/// path for chunk meshes (flat registry albedo, no lighting; mixed faces show
/// the world-anchored constituent speckle shader-side — ROADMAP PBR-1 walk-14,
/// journal/0020). Replaced the plain vertex-colored `StandardMaterial`, which
/// lost the mixture speckle when the mosaic collapsed to single quads.
#[derive(Resource)]
pub struct FullbrightMaterialHandle(pub Handle<FullbrightTerrainMaterial>);

/// Shared **lit** LabPBR terrain material (ROADMAP PBR-1) — the default chunk
/// material when not in fullbright. One instance for near-field, far field, and
/// legacy S1; per-voxel material selection lives in the mesh splat attributes.
#[derive(Resource)]
pub struct TerrainMaterialHandle(pub Handle<TerrainMaterial>);

pub fn to_render(v: DVec3) -> Vec3 {
    Vec3::new(v.x as f32, v.y as f32, v.z as f32)
}

/// First surface point spiraling out from the origin that is open ground
/// (above y = 2 m). The origin itself sits on the chasm floor at ~−82 m,
/// which made every new walker's first view a wall (journal/0003).
///
/// The final feet altitude comes from the **true voxel surface** under the
/// player's footprint ([`true_surface_m`]), not `surface_height_m` — the
/// analytic height under-reports on slopes and buried the spawn (journal/0004,
/// ROADMAP Observed). The ring search still uses the analytic height as a cheap
/// "is this column out of the chasm" filter.
///
/// The interactive boot now spawns over the ACTIVE authority
/// ([`Authority::find_open_spawn`], worldgen at N=2); this free form over an
/// explicit `TerrainGen` remains the S1-terrain reference the spawn/attach
/// tests exercise.
#[cfg(test)]
pub fn find_open_spawn(terrain: &TerrainGen, scale: VoxelScale) -> DVec3 {
    let (mut sx, mut sz) = (0.0, 0.0);
    'search: for ring in 0..48 {
        let d = f64::from(ring) * 12.0;
        for (ox, oz) in [
            (0.0, d),
            (0.0, -d),
            (d, 0.0),
            (-d, 0.0),
            (d, d),
            (-d, d),
            (d, -d),
            (-d, -d),
        ] {
            if terrain.surface_height_m(ox, oz) > 2.0 {
                (sx, sz) = (ox, oz);
                break 'search;
            }
        }
    }
    let solid = |x: i64, y: i64, z: i64| terrain.block_at(scale, x, y, z) != Block::Air;
    let surface = crate::worldgen::true_surface_m(
        &solid,
        |x, z| terrain.surface_height_m(x, z),
        scale,
        sx,
        sz,
        crate::player::PLAYER_WIDTH_M / 2.0,
    )
    .unwrap_or_else(|| terrain.surface_height_m(sx, sz));
    DVec3::new(sx, surface + 2.0, sz)
}

/// `--fullbright` diagnostic mode: chunk materials render unlit so agent
/// walks testing non-visual features see pure vertex color, never lighting.
#[derive(Resource, Clone, Copy)]
pub struct Fullbright(pub bool);

/// `--edges` diagnostic mode: the renderer adds crease/silhouette outlining
/// (edgepass.rs) so gross geometry is legible even on flat-albedo fullbright
/// terrain. Read by `setup` to opt the camera into the pass.
#[derive(Resource, Clone, Copy)]
pub struct Edges(pub bool);

pub fn run(
    pack_selector: Option<String>,
    mcp_options: McpOptions,
    fullbright: bool,
    edges: bool,
    gen_options: authority::GenOptions,
    horizon: farmesh::HorizonConfig,
) {
    // ROADMAP 3c-1: boot at N=2 (the ratified S1 scale) over the real
    // hierarchical worldgen authority — `Authority::new` maps player=2 voxels
    // to the worldgen authority (keys 3/4 stay the legacy S1 TerrainGen). The
    // spawn is seated on the worldgen's TRUE voxel surface.
    let boot_voxels = 2u32;
    let mut authority = Authority::new_with(BENCH_SEED, boot_voxels, &gen_options);
    let spawn = authority.find_open_spawn();
    let boot_title = title_text(boot_voxels, true, authority.authority_label());
    // Under the worldgen authority (boot, key 2) the horizon is now the
    // coarse-summary heightfield sampled from the authority's OWN surface
    // (journal/0022, `farmesh::stream_far_surface`) — the phantom S1 old-world is
    // gone. The legacy S1 `TerrainGen` far mesh survives ONLY for the S1
    // authority (keys 3/4), where it was never wrong; its generator lives inside
    // the far-mesh's own resource, with no ambient `Terrain` fallback for a
    // near-field system to reach a wrong world through.
    let far_terrain = farmesh::FarFieldTerrain::new(BENCH_SEED);

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: boot_title,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PostStagePlugin { pack_selector })
    .add_plugins(EdgePassPlugin { enabled: edges })
    .add_plugins(TerrainMaterialPlugin)
    .add_plugins(GpuProbePlugin)
    .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.95)))
    .insert_resource(Fullbright(fullbright))
    .insert_resource(Edges(edges))
    // The far field's ring geometry (`--horizon`, journal/0042). A resource, not
    // a const, so the horizon is a launch decision; the default reproduces the
    // shipped 1.2 km rings exactly.
    .insert_resource(horizon)
    .insert_resource(far_terrain)
    .insert_resource(CurrentScale::new(boot_voxels))
    .insert_resource(FloatingOrigin(spawn))
    .insert_resource(Player::new(spawn))
    .insert_resource(ChunkMap::default())
    .insert_resource(farmesh::FarChunkMap::default())
    .insert_resource(farmesh::FarSurfaceMap::default())
    // The authoritative world for edits (client-through-dc-api milestone):
    // the worldgen authority built above, serving the streamed terrain.
    .insert_resource(authority)
    // The gen-time options a key-2 scale switch rebuilds the world with
    // (deep-config plumbing, journal/0039).
    .insert_resource(gen_options)
    .insert_resource(DirtyChunks::default())
    .insert_resource(character::CharacterVisuals::default())
    .insert_resource(edit::CrosshairTarget::default())
    .add_systems(Startup, (setup, physdemo::setup, edit::setup_crosshair))
    .add_systems(
        Update,
        (
            // Front of the chain: the dev console eats keystrokes while open
            // and resets the input resources, so the gameplay systems below see
            // nothing while the player is typing. Nested so the outer tuple
            // stays within Bevy's 20-element system-tuple limit; both levels are
            // `.chain()`ed, so the whole gameplay block still runs in order
            // after `console_input`.
            console::console_input,
            (
                grab_mouse,
                switch_scale,
                player::update_player,
                update_origin,
                player::update_camera,
                edit::update_target,
                edit::apply_edits,
                edit::draw_target,
                authority::drain_bridge,
                authority::tick_authority,
                authority::remesh_dirty,
                character::sync_characters,
                physdemo::update,
                position_chunks,
                farmesh::position_far_chunks,
                farmesh::position_far_tiles,
                streaming::stream_chunks,
                farmesh::stream_far_chunks,
                farmesh::stream_far_surface,
                update_title,
            )
                .chain(),
        )
            .chain(),
    );
    // The bridge always exists (its channel also carries the dev console's
    // submissions); the MCP server threads inside are what `mcp_options` gates.
    let (bridge, console_tx) = mcp::spawn_servers(mcp_options);
    app.insert_resource(bridge);
    app.add_plugins(ConsolePlugin { bridge: console_tx });
    app.run();
}

/// FF2a step-0 instrumentation (env-gated by `DC_GPU_PROBE`, zero cost when
/// unset): does Bevy 0.19's GPU-driven multidraw path engage for our custom
/// `TerrainMaterial` meshes? Adds a render-app system that, after the phase
/// buffers are collected, logs the device's `GpuPreprocessingMode` and the count
/// of GPU-built indirect draw batches / multidraw sets for the `Opaque3d` phase
/// (which our lit terrain — near chunks + far tiles, one shared material — draws
/// into). `mode == Culling` with `batch_count > 0` proves indirect multidraw is
/// engaged; `sets < batches` proves meshes are merged into shared multidraws.
struct GpuProbePlugin;

impl Plugin for GpuProbePlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::batching::gpu_preprocessing::{
            GpuPreprocessingMode, GpuPreprocessingSupport, IndirectParametersBuffers,
        };
        use bevy::render::{Render, RenderApp, RenderSystems};

        if std::env::var("DC_GPU_PROBE").is_err() {
            return;
        }
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            probe_gpu_driven.after(RenderSystems::PrepareResourcesCollectPhaseBuffers),
        );

        fn probe_gpu_driven(
            support: Res<GpuPreprocessingSupport>,
            indirect: Res<IndirectParametersBuffers>,
            mut frame: Local<u32>,
        ) {
            *frame += 1;
            // Warm up (let terrain + far tiles stream) then log every ~2 s.
            if *frame < 180 || !(*frame).is_multiple_of(120) {
                return;
            }
            let mode = match support.max_supported_mode {
                GpuPreprocessingMode::None => "None (CPU uniforms, direct draws)",
                GpuPreprocessingMode::PreprocessingOnly => {
                    "PreprocessingOnly (GPU uniforms, direct draws — no multidraw)"
                }
                GpuPreprocessingMode::Culling => "Culling (GPU multidraw indirect eligible)",
            };
            match indirect.buffers.get(&std::any::TypeId::of::<
                bevy::core_pipeline::core_3d::Opaque3d,
            >()) {
                Some(phase) => info!(
                    "DC_GPU_PROBE f{}: mode={} | Opaque3d indexed batches={} sets={} | \
                     nonindexed batches={} sets={}",
                    *frame,
                    mode,
                    phase.indexed.batch_count(),
                    phase.batch_set_count(true),
                    phase.non_indexed.batch_count(),
                    phase.batch_set_count(false),
                ),
                None => info!(
                    "DC_GPU_PROBE f{}: mode={} | no Opaque3d indirect buffers \
                     (direct-draw path — indirect drawing not in use)",
                    *frame, mode
                ),
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut fullbright_materials: ResMut<Assets<FullbrightTerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    fullbright_mode: Res<Fullbright>,
    edges: Res<Edges>,
    horizon: Res<farmesh::HorizonConfig>,
) {
    // Simple diffuse setup: one sun, flat ambient on the camera. Computed first
    // so the LabPBR terrain material lights against the same sun.
    let sun_rotation = Quat::from_euler(EulerRot::YXZ, 0.6, -1.0, 0.0);
    let sun_dir = sun_rotation * Vec3::Z; // a light points along -Z → toward sun is +Z

    // Fullbright chunk material: unlit flat registry albedo, with the
    // world-anchored mixture speckle on mixed faces (the walk diagnostic).
    let fullbright = terrain_material::build_fullbright_material();
    commands.insert_resource(FullbrightMaterialHandle(
        fullbright_materials.add(fullbright),
    ));
    // Lit LabPBR terrain material (the default): assembles the placeholder
    // atlases and lights them against the scene sun.
    let terrain = terrain_material::build_terrain_material(&mut images, sun_dir);
    commands.insert_resource(TerrainMaterialHandle(terrain_materials.add(terrain)));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(sun_rotation),
    ));
    // World state seen by the shader pack's post stage. A light points along
    // its -Z, so "toward the sun" is the rotated +Z.
    let mut post = PostStage {
        sun_dir: sun_rotation * Vec3::Z,
        ..default()
    };
    // `--fullbright` disables distance fog entirely (journal/0030,
    // corrections #18): fullbright is the pure-data diagnostic register, and
    // atmospheric haze washed the 3.5 km massif vista to near-white in it,
    // making landform silhouette work impossible. The data-side fix pushes the
    // fog range beyond the 3 km far plane so `dc_fog_factor` is 0 for ALL
    // geometry — no pack-WGSL change, and lit-pass fog is untouched. (The
    // separate sky-haze pull, keyed off `is_sky`, is not distance fog and
    // stays, so the sky still meets the horizon.)
    // Distance haze is tuned against the far field's reach, so it has to travel
    // with `--horizon` (journal/0042): at the default it is byte-identically the
    // shipped 150 / 1100 m, and a 10 km horizon pushes it out proportionally —
    // otherwise the macro landform the wider horizon exists to show would be
    // fogged to white at 1.1 km, which is the exact failure journal/0030 hit.
    (post.fog_start_m, post.fog_end_m) = horizon.fog_range_m();
    if fullbright_mode.0 {
        post.fog_start_m = 1.0e9;
        post.fog_end_m = 2.0e9;
    }

    let camera = commands
        .spawn((
            Camera3d {
                // The S4 post stage samples scene depth (fog/haze), so the depth
                // texture must be bindable, not only an attachment.
                depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::TEXTURE_BINDING)
                    .into(),
                ..default()
            },
            // The post-stage contract v0 binds single-sample depth (see
            // poststage.rs); MSAA is off for the S4 slice.
            Msaa::Off,
            // The active shader pack owns the tonemap curve (visuals.md § post);
            // Bevy's built-in pass must not grade on top of it.
            Tonemapping::None,
            post,
            // The far field reaches `--horizon` (1.2 km by default, farmesh.rs);
            // the default 1 km far plane would clip the outermost LOD ring, and a
            // widened horizon needs the plane to travel with it.
            Projection::Perspective(PerspectiveProjection {
                far: horizon.camera_far_m(),
                ..default()
            }),
            AmbientLight {
                color: Color::WHITE,
                brightness: 400.0,
                affects_lightmapped_meshes: true,
            },
            Transform::default(),
        ))
        .id();
    // Opt the camera into the `--edges` pass. The pass itself is only scheduled
    // when the plugin is enabled (edgepass.rs), so this component is inert with
    // `--edges` off — the pure-data control stays byte-identical.
    if edges.0 {
        commands.entity(camera).insert(EdgeParams);
    }
}

/// Left click grabs the cursor for mouse look; Escape releases it.
fn grab_mouse(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

/// Keys 2/3/4: rebuild the world at player-height = N voxels. Same seed, same
/// landscape (noise is sampled in meter space) — only the resolution changes.
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
fn switch_scale(
    keys: Res<ButtonInput<KeyCode>>,
    mut scale: ResMut<CurrentScale>,
    mut map: ResMut<ChunkMap>,
    mut player: ResMut<Player>,
    mut authority: ResMut<Authority>,
    mut dirty: ResMut<DirtyChunks>,
    mut commands: Commands,
    chunk_entities: Query<Entity, With<ChunkEntity>>,
    gen_options: Res<authority::GenOptions>,
) {
    for (key, n) in [
        (KeyCode::Digit2, 2u32),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
    ] {
        if keys.just_pressed(key) && n != scale.player_voxels {
            *scale = CurrentScale::new(n);
            for entity in &chunk_entities {
                commands.entity(entity).despawn();
            }
            map.loaded.clear();
            // The voxel lattice changed under the hosted world: rebuild the
            // authority at the new scale. Edits do not survive a scale switch
            // (the lattice they lived on is gone); pending MCP replies are
            // dropped, which the server reports as a world reset.
            *authority = Authority::new_with(BENCH_SEED, n, &gen_options);
            dirty.0.clear();
            // The authority (key 2 = worldgen; 3/4 = S1 terrain) — and possibly
            // the whole world — changed under the player. Reseat the feet on
            // the new authority's TRUE voxel surface at (x, z) so a switch never
            // leaves them buried or falling. On a genuine miss (over open air)
            // keep the current altitude rather than dropping to a buried y.
            if let Some(surface) = authority.true_surface_m(
                player.pos_m.x,
                player.pos_m.z,
                crate::player::PLAYER_WIDTH_M / 2.0,
            ) {
                player.pos_m.y = surface + 0.05;
            }
            player.vel_m = DVec3::ZERO;
        }
    }
}

fn update_origin(player: Res<Player>, mut origin: ResMut<FloatingOrigin>) {
    if (player.pos_m - origin.0).length_squared() > ORIGIN_REBASE_M * ORIGIN_REBASE_M {
        origin.0 = player.pos_m;
    }
}

/// Place every chunk relative to the floating origin, from f64 world
/// coordinates, every frame.
fn position_chunks(
    origin: Res<FloatingOrigin>,
    scale: Res<CurrentScale>,
    mut chunks: Query<(&ChunkEntity, &mut Transform)>,
) {
    let vs = scale.scale.voxel_size_m();
    for (chunk, mut transform) in &mut chunks {
        let (mx, my, mz) = chunk.0.min_voxel();
        let min_m = DVec3::new(mx as f64, my as f64, mz as f64) * vs;
        transform.translation = to_render(min_m - origin.0);
    }
}

fn title_text(player_voxels: u32, fly: bool, authority: &str) -> String {
    let mode = if fly { "fly" } else { "walk" };
    format!(
        "deepcraft — player = {player_voxels} voxels ({:.2} m/voxel) — {authority} — {mode} \
         [click: capture mouse | Esc: release | LMB/RMB: break/place | F: fly/walk | \
         2: worldgen | 3/4: S1 scale | G: toss cube]",
        PLAYER_HEIGHT_M / f64::from(player_voxels),
    )
}

fn update_title(
    scale: Res<CurrentScale>,
    player: Res<Player>,
    authority: Res<Authority>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let title = title_text(scale.player_voxels, player.fly, authority.authority_label());
    if window.title != title {
        window.title = title;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::BENCH_SEED;

    #[test]
    fn spawn_search_finds_open_ground_off_the_chasm_floor() {
        let terrain = TerrainGen::new(BENCH_SEED);
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 3);
        // The origin is the chasm floor (journal/0003) — well below open ground.
        assert!(terrain.surface_height_m(0.0, 0.0) < 0.0);
        let spawn = find_open_spawn(&terrain, scale);
        let surface = terrain.surface_height_m(spawn.x, spawn.z);
        assert!(
            surface > 2.0,
            "spawn surface at {surface} m is not open ground"
        );
        // The spawn body is placed on the TRUE voxel surface, not the analytic
        // height — so the player is never embedded (journal/0004 buried it here).
        let vs = scale.voxel_size_m();
        let solid = |x: i64, y: i64, z: i64| terrain.block_at(scale, x, y, z) != Block::Air;
        let aabb = dc_core::Aabb::from_bottom_center(
            DVec3::new(spawn.x, spawn.y, spawn.z) / vs,
            (crate::player::PLAYER_WIDTH_M / 2.0) / vs,
            PLAYER_HEIGHT_M / vs,
        );
        assert!(
            !dc_core::aabb_overlaps_solid(&solid, aabb),
            "spawn body embedded in terrain at {spawn:?}"
        );
    }
}
