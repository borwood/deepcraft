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

use std::cell::RefCell;
use std::collections::HashMap;

use bevy::prelude::*;
use dc_api::Posture;
use dc_api::bodies::{AnimClip, BodyPlan, biped_clips, biped_plan};
use dc_core::{VoxelQuery, VoxelScale};
use glam::DVec3;

use crate::app::{CurrentScale, FloatingOrigin, Fullbright, to_render};
use crate::authority::Authority;
use crate::body::{
    AnimState, CROUCH_ROOT_DROP_M, pose_for, resolve_orientation, solve_leg_ik, stepped_angle,
};

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

/// A leg's two-bone rig, derived from the plan once: the hip joint position in
/// body-local meters and the two bone lengths, for the foot-placement IK.
struct LegRig {
    upper: String,
    lower: String,
    /// Hip joint offset from the body root (feet), meters.
    hip_local: [f64; 3],
    /// Upper bone length (hip→knee) and lower bone length (knee→sole), meters.
    l1: f64,
    l2: f64,
}

/// The vanilla plan, its idle/walk clips, and the per-segment mesh+material,
/// built once and shared across every body.
struct BodyAssets {
    plan: BodyPlan,
    idle: AnimClip,
    walk: AnimClip,
    /// Segment name → (cuboid mesh, tinted material).
    segs: HashMap<String, (Handle<Mesh>, Handle<StandardMaterial>)>,
    /// The two legs' IK rigs (empty if the plan has no `leg_*_upper/lower`).
    legs: Vec<LegRig>,
    /// v0 face cue: a small dark brow band parented to the head's front (−Z)
    /// face, so orientation is photographable (placeholder until head textures).
    face: (Handle<Mesh>, Handle<StandardMaterial>),
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
    mut authority: ResMut<Authority>,
    scale: Res<CurrentScale>,
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
        // Derive each leg's IK rig from the plan: hip = parent(upper).pivot +
        // upper.pivot; l1 = |lower.pivot| (hip→knee); l2 = |lower.offset.y| +
        // lower.size.y/2 (knee→sole). Generic over any `leg_*_upper/_lower`.
        let seg_by = |name: &str| plan.segments.iter().find(|s| s.name == name);
        let mut legs = Vec::new();
        for upper in plan
            .segments
            .iter()
            .filter(|s| s.name.starts_with("leg_") && s.name.ends_with("_upper"))
        {
            let lower_name = upper.name.replace("_upper", "_lower");
            let Some(lower) = seg_by(&lower_name) else {
                continue;
            };
            let hip_local = match upper.parent.as_deref().and_then(seg_by) {
                Some(parent) => [
                    parent.pivot_m[0] + upper.pivot_m[0],
                    parent.pivot_m[1] + upper.pivot_m[1],
                    parent.pivot_m[2] + upper.pivot_m[2],
                ],
                None => upper.pivot_m,
            };
            let l1 =
                (lower.pivot_m[0].powi(2) + lower.pivot_m[1].powi(2) + lower.pivot_m[2].powi(2))
                    .sqrt();
            let l2 = lower.offset_m[1].abs() + lower.size_m[1] / 2.0;
            legs.push(LegRig {
                upper: upper.name.clone(),
                lower: lower_name,
                hip_local,
                l1,
                l2,
            });
        }
        // The v0 face cue: a thin dark quad across the head's upper front.
        let face_mesh = meshes.add(Cuboid::new(0.2, 0.06, 0.02));
        let face_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.11),
            perceptual_roughness: 0.9,
            unlit: fullbright.0,
            ..default()
        });
        visuals.assets = Some(BodyAssets {
            idle: clip("dc:anim/biped_idle"),
            walk: clip("dc:anim/biped_walk"),
            plan,
            segs,
            legs,
            face: (face_mesh, face_material),
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

    // Collect the characters first (releasing the shared authority borrow),
    // then build the foot-IK ground query over the SAME authority. Foot
    // placement reads the world the body actually stands in — edits included,
    // lazily generating an unstreamed chunk — never the old S1 far-mesh phantom
    // the render cache used to fall back to (walk-11 loose end / journal/0017).
    // Generating a chunk here is a pure, deterministic memoization of the host;
    // it never touches sim/replay state, so the animation firewall holds.
    let characters: Vec<dc_api::CharacterState> = authority.world.characters().cloned().collect();
    let authority_cell = RefCell::new(&mut *authority);
    let solid = |x: i64, y: i64, z: i64| authority_cell.borrow_mut().is_solid_voxel(x, y, z);

    for character in &characters {
        seen.push(character.name.clone());
        let feet = DVec3::new(character.pos_m.x, character.pos_m.y, character.pos_m.z);
        let translation = to_render(feet - origin.0);
        let speed =
            (character.vel_m.x * character.vel_m.x + character.vel_m.z * character.vel_m.z).sqrt();

        match visuals.bodies.get_mut(character.name.as_str()) {
            Some(instance) => {
                instance.anim.advance(dt, speed);
                instance
                    .anim
                    .steer(dt, character.vel_m.x, character.vel_m.z);
                let pose = pose_for(&instance.anim, &assets.idle, &assets.walk);
                // Trunk faces travel; head/neck follow the look (the walk-8 gap).
                let orient = resolve_orientation(
                    instance.anim.trunk_yaw,
                    f64::from(character.yaw),
                    f64::from(character.pitch),
                );
                let crouch_drop = if character.posture == Posture::Crouching {
                    CROUCH_ROOT_DROP_M
                } else {
                    0.0
                };

                // Root: feet position; the trunk faces travel, not the look.
                if let Ok(mut transform) = transforms.get_mut(instance.root) {
                    transform.translation = translation;
                    transform.rotation = Quat::from_rotation_y(orient.trunk_yaw as f32);
                }

                // Foot-placement IK: seat each foot on the ground under it,
                // overriding the clip's leg swing only where the terrain differs
                // (within a half-voxel); beyond the cap the foot floats honestly.
                let vscale = scale.scale;
                let half_voxel = vscale.voxel_size_m() * 0.5;
                let trunk = instance.anim.trunk_yaw;
                let mut leg_overrides: HashMap<String, [f64; 3]> = HashMap::new();
                for leg in &assets.legs {
                    let cu = pose.joints.get(&leg.upper).map_or(0.0, |e| e[0]);
                    let cl = pose.joints.get(&leg.lower).map_or(0.0, |e| e[0]);
                    let (fy, fz) = fk_foot_local(leg.l1, leg.l2, cu, cl);
                    let (hx, hz) = rotate_y_xz(trunk, leg.hip_local[0], leg.hip_local[2]);
                    let hip_y = feet.y + leg.hip_local[1] - crouch_drop;
                    let (dfx, dfz) = rotate_y_xz(trunk, 0.0, fz);
                    let foot_x = feet.x + hx + dfx;
                    let foot_z = feet.z + hz + dfz;
                    let foot_y = hip_y + fy;
                    if let Some(ground) =
                        ground_top_m(&solid, vscale, foot_x, foot_z, foot_y, half_voxel)
                    {
                        let adjust = ground - foot_y;
                        if adjust.abs() > 1e-3 && adjust.abs() <= half_voxel {
                            let ik = solve_leg_ik(leg.l1, leg.l2, [0.0, fy + adjust, fz]);
                            if ik.upper_x.is_finite() && ik.lower_x.is_finite() {
                                leg_overrides.insert(
                                    leg.upper.clone(),
                                    [stepped_angle(ik.upper_x), 0.0, 0.0],
                                );
                                leg_overrides.insert(
                                    leg.lower.clone(),
                                    [stepped_angle(ik.lower_x), 0.0, 0.0],
                                );
                            }
                        }
                    }
                }

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
                        // Root bob, plus the crouch spine drop (cosmetic half of
                        // the parametric-crouch split).
                        t.y += (pose.root_bob_m - crouch_drop) as f32;
                    }
                    // IK override on a leg joint; the neck composes the look on
                    // top of its clip pose; everything else is the clip pose.
                    let e = if let Some(o) = leg_overrides.get(&s.name) {
                        *o
                    } else {
                        let base = pose.joints.get(&s.name).copied().unwrap_or([0.0, 0.0, 0.0]);
                        if s.name == "neck" {
                            [
                                base[0] + orient.neck_pitch,
                                base[1] + orient.neck_yaw,
                                base[2],
                            ]
                        } else {
                            base
                        }
                    };
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
        let instance = spawn_body(&mut commands, assets, translation, yaw);
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
    yaw: f32,
) -> BodyInstance {
    let root = commands
        .spawn((
            CharacterBody,
            BodySegment,
            Transform::from_translation(translation).with_rotation(Quat::from_rotation_y(yaw)),
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
        // v0 face cue: a dark brow band on the head's front (−Z) face, so the
        // body's facing is photographable (walk-8: orientation was unverifiable
        // on a featureless head). Placeholder until head textures land.
        if s.name == "head" {
            let (fm, fmat) = assets.face.clone();
            let face = commands
                .spawn((
                    Mesh3d(fm),
                    MeshMaterial3d(fmat),
                    Transform::from_translation(Vec3::new(
                        s.offset_m[0] as f32,
                        s.offset_m[1] as f32 + 0.03,
                        s.offset_m[2] as f32 - (s.size_m[2] as f32) / 2.0 - 0.005,
                    )),
                ))
                .id();
            commands.entity(joint).add_child(face);
        }
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
        anim: AnimState {
            // Start facing the spawn yaw so the trunk doesn't swing to face
            // travel from an arbitrary zero on the first steps.
            trunk_yaw: f64::from(yaw),
            ..AnimState::default()
        },
    }
}

/// Foot position of a two-bone leg in its sagittal (y, z) plane, from the clip's
/// hip/knee X-rotations — the inverse of [`solve_leg_ik`]'s reconstruction, used
/// to find where the animation currently places the foot before re-seating it.
fn fk_foot_local(l1: f64, l2: f64, upper_x: f64, lower_x: f64) -> (f64, f64) {
    let ky = -l1 * upper_x.cos();
    let kz = -l1 * upper_x.sin();
    let total = upper_x + lower_x;
    (ky - l2 * total.cos(), kz - l2 * total.sin())
}

/// Rotate a body-local horizontal offset `(x, z)` by trunk yaw into world XZ
/// (bevy: `from_rotation_y(yaw)` maps `x' = x cos + z sin`, `z' = −x sin + z cos`).
fn rotate_y_xz(yaw: f64, x: f64, z: f64) -> (f64, f64) {
    let (s, c) = yaw.sin_cos();
    (x * c + z * s, -x * s + z * c)
}

/// The top face height (meters) of the highest solid voxel under a foot column,
/// scanning a half-voxel window around `near_y_m`. Reads `solid` — the active
/// authority's solidity (edits included), a legal one-way world read for
/// cosmetic foot placement. `None` when the window holds no solid (the foot
/// then floats).
fn ground_top_m(
    solid: &impl VoxelQuery,
    scale: VoxelScale,
    x_m: f64,
    z_m: f64,
    near_y_m: f64,
    half_voxel_m: f64,
) -> Option<f64> {
    let vs = scale.voxel_size_m();
    let vx = scale.voxel_at(x_m);
    let vz = scale.voxel_at(z_m);
    let vy_hi = scale.voxel_at(near_y_m + half_voxel_m);
    let vy_lo = scale.voxel_at(near_y_m - half_voxel_m) - 1;
    let mut vy = vy_hi;
    while vy >= vy_lo {
        if solid.is_solid(vx, vy, vz) {
            return Some((vy as f64 + 1.0) * vs);
        }
        vy -= 1;
    }
    None
}
