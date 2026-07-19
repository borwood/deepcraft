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
use crate::edit;
use crate::farmesh;
use crate::mcp::{self, McpOptions};
use crate::physdemo;
use crate::player::{self, Player};
use crate::poststage::{PostStage, PostStagePlugin};
use crate::streaming;
use crate::terrain_material::{self, TerrainMaterial, TerrainMaterialPlugin};
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

/// Shared **unlit** vertex-colored material — the `--fullbright` diagnostic
/// path for chunk meshes (pure vertex color, no lighting).
#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

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

pub fn run(pack_selector: Option<String>, mcp_options: McpOptions, fullbright: bool) {
    // ROADMAP 3c-1: boot at N=2 (the ratified S1 scale) over the real
    // hierarchical worldgen authority — `Authority::new` maps player=2 voxels
    // to the worldgen authority (keys 3/4 stay the legacy S1 TerrainGen). The
    // spawn is seated on the worldgen's TRUE voxel surface.
    let boot_voxels = 2u32;
    let mut authority = Authority::new(BENCH_SEED, boot_voxels);
    let spawn = authority.find_open_spawn();
    let boot_title = title_text(boot_voxels, true, authority.authority_label());
    // The far-mesh rings still sample the legacy S1 `TerrainGen` (ROADMAP
    // Observed / journal/0017: the far field is not yet worldgen-shaped, so it
    // paints a ~1 km phantom old world below the real terrain). That generator
    // now lives ONLY inside the far-mesh's own resource — there is no ambient
    // `Terrain` fallback for a near-field system to reach a wrong world through.
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
    .add_plugins(TerrainMaterialPlugin)
    .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.95)))
    .insert_resource(Fullbright(fullbright))
    .insert_resource(far_terrain)
    .insert_resource(CurrentScale::new(boot_voxels))
    .insert_resource(FloatingOrigin(spawn))
    .insert_resource(Player::new(spawn))
    .insert_resource(ChunkMap::default())
    .insert_resource(farmesh::FarChunkMap::default())
    // The authoritative world for edits (client-through-dc-api milestone):
    // the worldgen authority built above, serving the streamed terrain.
    .insert_resource(authority)
    .insert_resource(DirtyChunks::default())
    .insert_resource(character::CharacterVisuals::default())
    .insert_resource(edit::CrosshairTarget::default())
    .add_systems(Startup, (setup, physdemo::setup, edit::setup_crosshair))
    .add_systems(
        Update,
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
            streaming::stream_chunks,
            farmesh::stream_far_chunks,
            update_title,
        )
            .chain(),
    );
    if let Some(bridge) = mcp::spawn_servers(mcp_options) {
        app.insert_resource(bridge);
    }
    app.run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Simple diffuse setup: one sun, flat ambient on the camera. Computed first
    // so the LabPBR terrain material lights against the same sun.
    let sun_rotation = Quat::from_euler(EulerRot::YXZ, 0.6, -1.0, 0.0);
    let sun_dir = sun_rotation * Vec3::Z; // a light points along -Z → toward sun is +Z

    // Fullbright chunk material: unlit, pure vertex color (walk diagnostic).
    commands.insert_resource(ChunkMaterial(materials.add(StandardMaterial {
        base_color: Color::WHITE, // multiplied by vertex colors
        perceptual_roughness: 0.95,
        unlit: true,
        ..default()
    })));
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
    commands.spawn((
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
        // World state seen by the shader pack's post stage. A light points
        // along its -Z, so "toward the sun" is the rotated +Z.
        PostStage {
            sun_dir: sun_rotation * Vec3::Z,
            ..default()
        },
        // The far field reaches 1.2 km (see farmesh.rs); the default 1 km far
        // plane would clip the outermost LOD ring.
        Projection::Perspective(PerspectiveProjection {
            far: 3000.0,
            ..default()
        }),
        AmbientLight {
            color: Color::WHITE,
            brightness: 400.0,
            affects_lightmapped_meshes: true,
        },
        Transform::default(),
    ));
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
            *authority = Authority::new(BENCH_SEED, n);
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
