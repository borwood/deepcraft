//! S6 demo: **G** tosses a physics cube from the camera.
//!
//! Deliberately minimal — no pickup, no detach UI (detach/reattach is proven
//! headless in dc-physics). Each cube is a dc-physics dropped item; terrain
//! collision comes from the collider bubble fed by the same solidity closure
//! the player's swept-AABB uses (`ChunkMap::is_solid`, generator fallback for
//! unloaded chunks). Physics runs on dc-physics' fixed 60 Hz accumulator
//! inside the Update chain; rendering reads poses each frame and, like
//! everything else, positions them relative to the floating origin.

use bevy::prelude::*;
use dc_physics::{PhysicsConfig, PhysicsWorld, RigidBodyHandle};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::app::{ChunkMap, CurrentScale, FloatingOrigin, Terrain, to_render};
use crate::player::Player;

/// Cube half-extent in meters (0.4 m cubes: item-sized, clearly sub-voxel).
const CUBE_HALF_M: f64 = 0.2;
/// Toss speed along the view direction, m/s.
const TOSS_SPEED_M_S: f64 = 8.0;
/// Spawn this far in front of the eye so the cube never starts inside the
/// player's own AABB.
const SPAWN_AHEAD_M: f64 = 1.2;

#[derive(Resource)]
pub struct PhysicsDemo {
    world: PhysicsWorld,
    cubes: Vec<(RigidBodyHandle, Entity)>,
    /// Scale the current world was built for; a 2/3/4 rebuild resets physics
    /// (the voxel lattice the bubbles collide with changed under us).
    built_for_voxels: u32,
}

#[derive(Resource)]
pub struct CubeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let side = (2.0 * CUBE_HALF_M) as f32;
    commands.insert_resource(CubeAssets {
        mesh: meshes.add(Cuboid::new(side, side, side)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.35, 0.2),
            perceptual_roughness: 0.6,
            ..default()
        }),
    });
    commands.insert_resource(PhysicsDemo {
        world: PhysicsWorld::new(PhysicsConfig::default()),
        cubes: Vec::new(),
        built_for_voxels: 0,
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    terrain: Res<Terrain>,
    scale: Res<CurrentScale>,
    map: Res<ChunkMap>,
    origin: Res<FloatingOrigin>,
    player: Res<Player>,
    assets: Res<CubeAssets>,
    mut demo: ResMut<PhysicsDemo>,
    mut transforms: Query<&mut Transform>,
) {
    // Scale switch rebuilt the world: drop the cubes with it.
    if demo.built_for_voxels != scale.player_voxels {
        for (_, entity) in demo.cubes.drain(..) {
            commands.entity(entity).despawn();
        }
        demo.world = PhysicsWorld::new(PhysicsConfig::default());
        demo.built_for_voxels = scale.player_voxels;
    }

    if keys.just_pressed(KeyCode::KeyG) {
        let (sin_yaw, cos_yaw) = (f64::from(player.yaw.sin()), f64::from(player.yaw.cos()));
        let (sin_pitch, cos_pitch) = (f64::from(player.pitch.sin()), f64::from(player.pitch.cos()));
        let dir = DVec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch);
        let eye = player.pos_m + DVec3::new(0.0, 0.9 * PLAYER_HEIGHT_M, 0.0);
        let pos = eye + dir * SPAWN_AHEAD_M;
        let handle = demo
            .world
            .spawn_item(pos, DVec3::splat(CUBE_HALF_M), dir * TOSS_SPEED_M_S);
        let entity = commands
            .spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.material.clone()),
                Transform::from_translation(to_render(pos - origin.0)),
            ))
            .id();
        demo.cubes.push((handle, entity));
    }

    // Fixed-timestep stepping, decoupled from frame rate. Solidity is the same
    // query the player's collision uses.
    let vscale = scale.scale;
    let solid = |x: i64, y: i64, z: i64| map.is_solid(&terrain.0, vscale, x, y, z);
    demo.world
        .advance(f64::from(time.delta_secs()), vscale.voxel_size_m(), &solid);

    // Sync render transforms from body poses (f64 world meters -> origin-
    // relative f32, like chunks and the camera).
    for (handle, entity) in &demo.cubes {
        if let Some(pose) = demo.world.body_pose(*handle)
            && let Ok(mut transform) = transforms.get_mut(*entity)
        {
            transform.translation = to_render(pose.pos_m - origin.0);
            transform.rotation = Quat::from_xyzw(pose.rot.x, pose.rot.y, pose.rot.z, pose.rot.w);
        }
    }
}
