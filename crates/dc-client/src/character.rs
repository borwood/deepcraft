//! Character bodies in the render world: a minimal but unmistakable
//! two-cuboid figure (torso + head, distinct colors) per character in the
//! authority. Purely a view: the authoritative state lives in the hosted
//! world (dc-api characters, stepped on the host tick); this system mirrors
//! it into entities each frame, origin-relative like everything the GPU sees.

use std::collections::HashMap;

use bevy::prelude::*;
use glam::DVec3;

use crate::app::{FloatingOrigin, Fullbright, to_render};
use crate::authority::Authority;

/// Torso: feet up to 75% of height; head: the rest. Meters (fixed body size,
/// like the player — the voxel scale changes the world's resolution, not the
/// people).
const HEIGHT_M: f32 = 1.8;
const TORSO_TOP_M: f32 = 1.35;
const TORSO_WIDTH_M: f32 = 0.6;
const TORSO_DEPTH_M: f32 = 0.35;
const HEAD_SIDE_M: f32 = 0.45;

/// Root entity of one character's body; translation = feet, rotation = yaw.
/// (Which character it is lives in [`CharacterVisuals::bodies`].)
#[derive(Component)]
pub struct CharacterBody;

/// Handles for spawned character bodies plus their shared assets.
#[derive(Resource, Default)]
pub struct CharacterVisuals {
    pub bodies: HashMap<String, Entity>,
    assets: Option<BodyAssets>,
}

struct BodyAssets {
    torso_mesh: Handle<Mesh>,
    head_mesh: Handle<Mesh>,
    torso_material: Handle<StandardMaterial>,
    head_material: Handle<StandardMaterial>,
}

/// Mirror the authority's characters into render entities: spawn bodies for
/// new characters, move existing ones (feet position + yaw), despawn bodies
/// whose character vanished (scale switch rebuilds the authority).
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn sync_characters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fullbright: Res<Fullbright>,
    origin: Res<FloatingOrigin>,
    authority: Res<Authority>,
    mut visuals: ResMut<CharacterVisuals>,
    mut transforms: Query<&mut Transform, With<CharacterBody>>,
) {
    // Shared assets, once.
    if visuals.assets.is_none() {
        let mut material = |color: Color| {
            materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.9,
                unlit: fullbright.0,
                ..default()
            })
        };
        visuals.assets = Some(BodyAssets {
            torso_mesh: meshes.add(Cuboid::new(TORSO_WIDTH_M, TORSO_TOP_M, TORSO_DEPTH_M)),
            head_mesh: meshes.add(Cuboid::new(
                HEAD_SIDE_M,
                HEIGHT_M - TORSO_TOP_M,
                HEAD_SIDE_M,
            )),
            // A companion should read instantly against terrain greens/greys:
            // warm signal-orange torso, pale head.
            torso_material: material(Color::srgb(0.9, 0.42, 0.12)),
            head_material: material(Color::srgb(0.95, 0.85, 0.7)),
        });
    }

    let mut seen: Vec<&str> = Vec::new();
    for character in authority.world.characters() {
        seen.push(&character.name);
        let feet = DVec3::new(character.pos_m.x, character.pos_m.y, character.pos_m.z);
        let translation = to_render(feet - origin.0);
        let rotation = Quat::from_rotation_y(character.yaw);
        match visuals.bodies.get(character.name.as_str()) {
            Some(&entity) => {
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation = translation;
                    transform.rotation = rotation;
                }
            }
            None => {
                let assets = visuals.assets.as_ref().expect("created above");
                let body = commands
                    .spawn((
                        CharacterBody,
                        Transform::from_translation(translation).with_rotation(rotation),
                        Visibility::default(),
                        children![
                            (
                                Mesh3d(assets.torso_mesh.clone()),
                                MeshMaterial3d(assets.torso_material.clone()),
                                Transform::from_xyz(0.0, TORSO_TOP_M / 2.0, 0.0),
                            ),
                            (
                                Mesh3d(assets.head_mesh.clone()),
                                MeshMaterial3d(assets.head_material.clone()),
                                Transform::from_xyz(0.0, (TORSO_TOP_M + HEIGHT_M) / 2.0, 0.0),
                            ),
                        ],
                    ))
                    .id();
                visuals.bodies.insert(character.name.clone(), body);
            }
        }
    }

    // Characters gone from the authority lose their bodies.
    let stale: Vec<String> = visuals
        .bodies
        .keys()
        .filter(|name| !seen.contains(&name.as_str()))
        .cloned()
        .collect();
    for name in stale {
        if let Some(entity) = visuals.bodies.remove(&name) {
            commands.entity(entity).despawn();
        }
    }
}
