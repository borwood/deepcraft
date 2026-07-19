//! Character bodies in the render world: a jointed **biped**, rendered as an
//! instance of the `dc:body/biped` registry plan (docs/design/bodies.md steps
//! 1–2). Replaces the old two-cuboid figure. Purely a view: the authoritative
//! state lives in the hosted world (dc-api characters, stepped on the host
//! tick); this system mirrors each character into a segment hierarchy and
//! animates it on the render clock, origin-relative like everything the GPU
//! sees.
//!
//! **Firewall discipline.** The plan and clips are pure registry data
//! ([`dc_api::bodies`]); the pose is sampled entirely here on the client
//! ([`crate::body`]) and never read back into simulation. The only sim state
//! read is each body's velocity, to choose idle vs walk — a legal one-way read.
//!
//! v0: every character wears the one vanilla plan (per-character plans /
//! transmog are step 3+). The plan drives geometry and the verb→slot bindings;
//! `body.rs` drives the motion.

use std::collections::HashMap;

use bevy::prelude::*;
use dc_api::bodies::{AnimClip, BodyPlan, biped_clips, biped_plan};
use glam::DVec3;

use crate::app::{FloatingOrigin, Fullbright, to_render};
use crate::authority::Authority;
use crate::body::{AnimState, pose_for};

/// Root marker on a character's body root entity (translation = feet, rotation
/// = yaw). Which character it is lives in [`CharacterVisuals::bodies`].
#[derive(Component)]
pub struct CharacterBody;

/// Marker on every animatable transform in a body (the root and each joint) —
/// the query filter for the per-frame pose write.
#[derive(Component)]
pub struct BodySegment;

/// One rendered body: its root entity, its joint entities by segment name, and
/// its animation state.
pub struct BodyInstance {
    root: Entity,
    joints: HashMap<String, Entity>,
    anim: AnimState,
}

/// Handles for spawned character bodies plus the shared plan/clip/asset cache.
#[derive(Resource, Default)]
pub struct CharacterVisuals {
    pub bodies: HashMap<String, BodyInstance>,
    assets: Option<BodyAssets>,
}

/// The vanilla plan, its idle/walk clips, and the per-segment mesh+material,
/// built once and shared across every body.
struct BodyAssets {
    plan: BodyPlan,
    idle: AnimClip,
    walk: AnimClip,
    /// Segment name → (cuboid mesh, tinted material).
    segs: HashMap<String, (Handle<Mesh>, Handle<StandardMaterial>)>,
}

/// Mirror the authority's characters into animated biped bodies: spawn bodies
/// for new characters, move + pose existing ones, despawn bodies whose
/// character vanished (a scale switch rebuilds the authority).
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn sync_characters(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fullbright: Res<Fullbright>,
    origin: Res<FloatingOrigin>,
    authority: Res<Authority>,
    mut visuals: ResMut<CharacterVisuals>,
    mut transforms: Query<&mut Transform, With<BodySegment>>,
) {
    // Build the shared plan/clip/segment assets once.
    if visuals.assets.is_none() {
        let plan = biped_plan();
        let clips = biped_clips();
        let clip = |name: &str| {
            clips
                .iter()
                .find(|c| c.name == name)
                .cloned()
                .expect("vanilla biped clip present")
        };
        let mut segs = HashMap::new();
        for s in &plan.segments {
            let mesh = meshes.add(Cuboid::new(
                s.size_m[0] as f32,
                s.size_m[1] as f32,
                s.size_m[2] as f32,
            ));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(s.tint[0], s.tint[1], s.tint[2]),
                perceptual_roughness: 0.9,
                unlit: fullbright.0,
                ..default()
            });
            segs.insert(s.name.clone(), (mesh, material));
        }
        visuals.assets = Some(BodyAssets {
            idle: clip("dc:anim/biped_idle"),
            walk: clip("dc:anim/biped_walk"),
            plan,
            segs,
        });
    }

    let dt = f64::from(time.delta_secs());

    // Split the resource borrow: `assets` (read) and `bodies` (write) are
    // disjoint fields, so the sampler can read the plan/clips while the anim
    // states mutate.
    let visuals = &mut *visuals;
    let assets = visuals.assets.as_ref().expect("built above");

    let mut seen: Vec<String> = Vec::new();
    let mut to_spawn: Vec<(String, DVec3, f32, f64)> = Vec::new();

    for character in authority.world.characters() {
        seen.push(character.name.clone());
        let feet = DVec3::new(character.pos_m.x, character.pos_m.y, character.pos_m.z);
        let translation = to_render(feet - origin.0);
        let rotation = Quat::from_rotation_y(character.yaw);
        let speed =
            (character.vel_m.x * character.vel_m.x + character.vel_m.z * character.vel_m.z).sqrt();

        match visuals.bodies.get_mut(character.name.as_str()) {
            Some(instance) => {
                if let Ok(mut transform) = transforms.get_mut(instance.root) {
                    transform.translation = translation;
                    transform.rotation = rotation;
                }
                instance.anim.advance(dt, speed);
                let pose = pose_for(&instance.anim, &assets.idle, &assets.walk);
                for s in &assets.plan.segments {
                    let Some(&joint) = instance.joints.get(&s.name) else {
                        continue;
                    };
                    let Ok(mut tf) = transforms.get_mut(joint) else {
                        continue;
                    };
                    let mut t = Vec3::new(
                        s.pivot_m[0] as f32,
                        s.pivot_m[1] as f32,
                        s.pivot_m[2] as f32,
                    );
                    if s.parent.is_none() {
                        t.y += pose.root_bob_m as f32;
                    }
                    let e = pose.joints.get(&s.name).copied().unwrap_or([0.0, 0.0, 0.0]);
                    tf.translation = t;
                    tf.rotation =
                        Quat::from_euler(EulerRot::XYZ, e[0] as f32, e[1] as f32, e[2] as f32);
                }
            }
            None => to_spawn.push((character.name.clone(), feet, character.yaw, dt)),
        }
    }

    // Spawn new bodies (after the read-only pass over `bodies`).
    for (name, feet, yaw, _dt) in to_spawn {
        let translation = to_render(feet - origin.0);
        let rotation = Quat::from_rotation_y(yaw);
        let instance = spawn_body(&mut commands, assets, translation, rotation);
        visuals.bodies.insert(name, instance);
    }

    // Characters gone from the authority lose their bodies.
    let stale: Vec<String> = visuals
        .bodies
        .keys()
        .filter(|name| !seen.contains(name))
        .cloned()
        .collect();
    for name in stale {
        if let Some(instance) = visuals.bodies.remove(&name) {
            commands.entity(instance.root).despawn();
        }
    }
}

/// Spawn one biped body's entity hierarchy from the plan and return its
/// instance. Joint entities are spawned first (so parenting can wire an
/// arbitrary tree regardless of segment order), each carrying a mesh child
/// offset from its pivot; then the tree is stitched with `add_child`.
fn spawn_body(
    commands: &mut Commands,
    assets: &BodyAssets,
    translation: Vec3,
    rotation: Quat,
) -> BodyInstance {
    let root = commands
        .spawn((
            CharacterBody,
            BodySegment,
            Transform::from_translation(translation).with_rotation(rotation),
            Visibility::default(),
        ))
        .id();

    let mut joints: HashMap<String, Entity> = HashMap::new();
    for s in &assets.plan.segments {
        let (mesh, material) = assets.segs.get(&s.name).expect("segment asset").clone();
        let joint = commands
            .spawn((
                BodySegment,
                Transform::from_translation(Vec3::new(
                    s.pivot_m[0] as f32,
                    s.pivot_m[1] as f32,
                    s.pivot_m[2] as f32,
                )),
                Visibility::default(),
            ))
            .id();
        let cuboid = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(
                    s.offset_m[0] as f32,
                    s.offset_m[1] as f32,
                    s.offset_m[2] as f32,
                )),
            ))
            .id();
        commands.entity(joint).add_child(cuboid);
        joints.insert(s.name.clone(), joint);
    }
    // Wire the tree: each segment under its parent joint, roots under the body.
    for s in &assets.plan.segments {
        let joint = joints[&s.name];
        match &s.parent {
            Some(p) => commands.entity(joints[p]).add_child(joint),
            None => commands.entity(root).add_child(joint),
        };
    }

    BodyInstance {
        root,
        joints,
        anim: AnimState::default(),
    }
}
