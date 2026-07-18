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

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use dc_core::{Block, Chunk, ChunkPos, VoxelScale, local_voxel};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::bench::BENCH_SEED;
use crate::farmesh;
use crate::player::{self, Player};
use crate::streaming;
use crate::worldgen::TerrainGen;

/// Distance from the origin (meters) at which we re-snap it to the player.
const ORIGIN_REBASE_M: f64 = 256.0;

#[derive(Resource)]
pub struct Terrain(pub TerrainGen);

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
    /// `None` when the chunk meshed to nothing (all air / fully buried).
    pub entity: Option<Entity>,
}

/// All currently loaded chunks, at the current scale.
#[derive(Resource, Default)]
pub struct ChunkMap {
    pub loaded: HashMap<ChunkPos, LoadedChunk>,
}

impl ChunkMap {
    /// Solidity at a world voxel. Falls back to sampling the (deterministic)
    /// generator for chunks that aren't loaded, so collision and border
    /// meshing never see a hole where a chunk merely hasn't streamed in yet.
    pub fn is_solid(
        &self,
        terrain: &TerrainGen,
        scale: VoxelScale,
        x: i64,
        y: i64,
        z: i64,
    ) -> bool {
        match self.loaded.get(&ChunkPos::from_world_voxel(x, y, z)) {
            Some(loaded) => {
                let (lx, ly, lz) = local_voxel(x, y, z);
                loaded.chunk.get(lx, ly, lz).is_solid()
            }
            None => terrain.block_at(scale, x, y, z) != Block::Air,
        }
    }
}

/// Marks a chunk's render entity; the transform is recomputed from f64 every
/// frame by [`position_chunks`].
#[derive(Component)]
pub struct ChunkEntity(pub ChunkPos);

/// Shared vertex-colored material for all chunk meshes.
#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

pub fn to_render(v: DVec3) -> Vec3 {
    Vec3::new(v.x as f32, v.y as f32, v.z as f32)
}

pub fn run() {
    let terrain = TerrainGen::new(BENCH_SEED);
    let spawn = DVec3::new(0.0, terrain.surface_height_m(0.0, 0.0) + 2.0, 0.0);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title_text(3, true),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.95)))
        .insert_resource(Terrain(terrain))
        .insert_resource(CurrentScale::new(3))
        .insert_resource(FloatingOrigin(spawn))
        .insert_resource(Player::new(spawn))
        .insert_resource(ChunkMap::default())
        .insert_resource(farmesh::FarChunkMap::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                grab_mouse,
                switch_scale,
                player::update_player,
                update_origin,
                player::update_camera,
                position_chunks,
                farmesh::position_far_chunks,
                streaming::stream_chunks,
                farmesh::stream_far_chunks,
                update_title,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(ChunkMaterial(materials.add(StandardMaterial {
        base_color: Color::WHITE, // multiplied by vertex colors
        perceptual_roughness: 0.95,
        ..default()
    })));

    // Simple diffuse setup: one sun, flat ambient on the camera.
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.6, -1.0, 0.0)),
    ));
    commands.spawn((
        Camera3d::default(),
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
fn switch_scale(
    keys: Res<ButtonInput<KeyCode>>,
    mut scale: ResMut<CurrentScale>,
    mut map: ResMut<ChunkMap>,
    mut player: ResMut<Player>,
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
            // The re-voxelized surface can differ by up to a voxel; nudge up
            // so the player is never left embedded in the new ground.
            player.pos_m.y += scale.scale.voxel_size_m();
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

fn title_text(player_voxels: u32, fly: bool) -> String {
    let mode = if fly { "fly" } else { "walk" };
    format!(
        "deepcraft S1 — player = {player_voxels} voxels ({:.2} m/voxel) — {mode} \
         [click: capture mouse | Esc: release | F: fly/walk | 2/3/4: scale]",
        PLAYER_HEIGHT_M / f64::from(player_voxels),
    )
}

fn update_title(
    scale: Res<CurrentScale>,
    player: Res<Player>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let title = title_text(scale.player_voxels, player.fly);
    if window.title != title {
        window.title = title;
    }
}
