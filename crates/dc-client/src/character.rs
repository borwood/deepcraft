//! Character bodies in the render world: a jointed cuboid figure, rendered as an
//! instance of **whichever registry body plan the character wears**
//! (docs/design/bodies.md steps 1–2). Purely a view: the authoritative
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
//! **Plans come from the REGISTRY, per character.** This used to call
//! `biped_plan()` / `biped_clips()` as compiled-in Rust; the vanilla content now
//! arrives as a registry command batch at world construction
//! (`authority.rs::load_vanilla_body_pack`) and this system reads
//! `HostWorld::body_plan` / `anim_clip` for **whatever plan each character
//! wears** (`CharacterState::body_plan`, default `dc:body/biped`). Segment
//! meshes/materials and the derived leg rigs are cached per plan, so N bodies of
//! one plan still share one asset set.
//!
//! Transmog (a *running* body changing plan) has no verb yet; if the state's plan
//! ever differs from the rendered one, the body is rebuilt — the same path a
//! vanished character takes.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use dc_api::Posture;
use dc_api::bodies::{AnimClip, BodyPlan};
use dc_core::{VoxelQuery, VoxelScale};
use glam::DVec3;

use crate::app::{CurrentScale, FloatingOrigin, Fullbright, to_render};
use crate::authority::Authority;
use crate::body::{
    AnimState, CROUCH_ROOT_DROP_M, LegRig, fk_foot_local, leg_rigs, pose_for, resolve_orientation,
    solve_leg_ik, stepped_angle,
};

/// Root marker on a character's body root entity (translation = feet, rotation
/// = yaw). Which character it is lives in [`CharacterVisuals::bodies`].
#[derive(Component)]
pub struct CharacterBody;

/// Marker on every animatable transform in a body (the root and each joint) —
/// the query filter for the per-frame pose write.
#[derive(Component)]
pub struct BodySegment;

/// One rendered body: its root entity, its joint entities by segment name, the
/// plan it was built from, and its animation state.
pub struct BodyInstance {
    root: Entity,
    joints: HashMap<String, Entity>,
    anim: AnimState,
    /// The registered plan name this hierarchy was spawned from. A character
    /// whose state names a different plan is rebuilt (no transmog verb yet).
    plan: String,
}

/// Handles for spawned character bodies plus the per-plan asset cache.
#[derive(Resource, Default)]
pub struct CharacterVisuals {
    pub bodies: HashMap<String, BodyInstance>,
    /// Plan name → its meshes/materials/clips/leg rigs, built on first sighting
    /// of a character wearing it and shared by every body of that plan.
    plans: HashMap<String, BodyAssets>,
    /// Plan names already reported as unbuildable, so a body the registry cannot
    /// serve warns once instead of every frame.
    missing_plans: HashSet<String>,
}

/// One plan resolved for rendering: the plan itself, the clips its verb→slot
/// bindings name, the per-segment mesh+material, and the leg rigs derived from
/// its own proportions. Built once per plan and shared across every body of it.
struct BodyAssets {
    plan: BodyPlan,
    idle: AnimClip,
    walk: AnimClip,
    /// Segment name → (cuboid mesh, tinted material).
    segs: HashMap<String, (Handle<Mesh>, Handle<StandardMaterial>)>,
    /// The legs' IK rigs (empty if the plan has no `leg_*_upper/lower`), derived
    /// from THIS plan's bone lengths — the whole retargeting story is here.
    legs: Vec<LegRig>,
    /// v0 face cue: a small dark brow band parented to the head's front (−Z)
    /// face, so orientation is photographable (placeholder until head textures).
    face: (Handle<Mesh>, Handle<StandardMaterial>),
}

/// Mirror the authority's characters into animated bodies: resolve each
/// character's body plan from the registry, spawn bodies for new characters,
/// move + pose existing ones, despawn bodies whose character vanished (a scale
/// switch rebuilds the authority).
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
    let dt = f64::from(time.delta_secs());

    // Collect the characters first (releasing the shared authority borrow),
    // then build the foot-IK ground query over the SAME authority. Foot
    // placement reads the world the body actually stands in — edits included,
    // lazily generating an unstreamed chunk — never the old S1 far-mesh phantom
    // the render cache used to fall back to (walk-11 loose end / journal/0017).
    // Generating a chunk here is a pure, deterministic memoization of the host;
    // it never touches sim/replay state, so the animation firewall holds.
    let characters: Vec<dc_api::CharacterState> = authority.world.characters().cloned().collect();

    // Resolve every plan in use from the REGISTRY (not a compiled-in call), once
    // per plan. Done before the authority is borrowed mutably for the ground
    // query below: the registry read and the solidity read are separate passes.
    for character in &characters {
        let name = character.body_plan.as_str();
        if visuals.plans.contains_key(name) || visuals.missing_plans.contains(name) {
            continue;
        }
        match build_plan_assets(name, &authority, &mut meshes, &mut materials, fullbright.0) {
            Some(assets) => {
                info!("body plan `{name}` resolved from the registry for rendering");
                visuals.plans.insert(name.to_string(), assets);
            }
            None => {
                // Only reachable before the pack's first tick, or if a pack
                // failed to load — the host refuses a spawn naming an
                // unregistered plan, so this cannot be a stale character.
                warn!(
                    "character `{}` wears body plan `{name}`, which the registry \
                     cannot serve (plan or its idle/walk clip missing) — not rendered",
                    character.name
                );
                visuals.missing_plans.insert(name.to_string());
            }
        }
    }

    // Split the resource borrow: the per-plan assets (read) and `bodies` (write)
    // are disjoint fields, so the sampler can read a plan/clips while the anim
    // states mutate.
    let CharacterVisuals {
        bodies,
        plans,
        missing_plans: _,
    } = &mut *visuals;
    let plans = &*plans;

    let mut seen: Vec<String> = Vec::new();
    let mut to_spawn: Vec<(String, String, DVec3, f32)> = Vec::new();

    let authority_cell = RefCell::new(&mut *authority);
    let solid = |x: i64, y: i64, z: i64| authority_cell.borrow_mut().is_solid_voxel(x, y, z);

    for character in &characters {
        let Some(assets) = plans.get(character.body_plan.as_str()) else {
            continue; // warned above; nothing to render for this body yet
        };
        seen.push(character.name.clone());
        let feet = DVec3::new(character.pos_m.x, character.pos_m.y, character.pos_m.z);
        let translation = to_render(feet - origin.0);
        let speed =
            (character.vel_m.x * character.vel_m.x + character.vel_m.z * character.vel_m.z).sqrt();

        // A body whose rendered plan no longer matches its state is rebuilt
        // (transmog has no verb yet; when it gets one, this is the seam).
        if let Some(instance) = bodies.get(character.name.as_str())
            && instance.plan != character.body_plan
        {
            let instance = bodies.remove(character.name.as_str()).expect("just read");
            commands.entity(instance.root).despawn();
        }

        match bodies.get_mut(character.name.as_str()) {
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
            None => to_spawn.push((
                character.name.clone(),
                character.body_plan.clone(),
                feet,
                character.yaw,
            )),
        }
    }

    // Spawn new bodies (after the read-only pass over `bodies`).
    for (name, plan, feet, yaw) in to_spawn {
        let Some(assets) = plans.get(plan.as_str()) else {
            continue;
        };
        let translation = to_render(feet - origin.0);
        let instance = spawn_body(&mut commands, assets, &plan, translation, yaw);
        bodies.insert(name, instance);
    }

    // Characters gone from the authority lose their bodies. `seen` holds only the
    // characters this frame could render, so an unrenderable body is also cleaned
    // up — the honest outcome when a plan cannot be resolved.
    let stale: Vec<String> = bodies
        .keys()
        .filter(|name| !seen.contains(name))
        .cloned()
        .collect();
    for name in stale {
        if let Some(instance) = bodies.remove(&name) {
            commands.entity(instance.root).despawn();
        }
    }
}

/// Resolve one plan name into render assets **from the registry**: the plan, the
/// clips its `idle`/`walk` slots bind, per-segment cuboid mesh + tinted material,
/// the leg rigs derived from this plan's own proportions, and the face cue.
///
/// `None` when the registry holds no such plan, or the plan's required slots bind
/// clips that are not registered — a content failure the caller reports rather
/// than papering over with a hard-coded biped.
fn build_plan_assets(
    name: &str,
    authority: &Authority,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fullbright: bool,
) -> Option<BodyAssets> {
    let plan = authority.world.body_plan(name)?.plan.clone();
    // Verbs → clips through the plan's own bindings, not a hard-coded clip name:
    // a plan is free to bind any registered clip to `idle`/`walk`.
    let clip_for = |verb: &str| -> Option<AnimClip> {
        let slot = plan.slots.iter().find(|s| s.verb == verb)?;
        Some(authority.world.anim_clip(&slot.clip)?.clip.clone())
    };
    let idle = clip_for("idle")?;
    let walk = clip_for("walk")?;

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
            unlit: fullbright,
            ..default()
        });
        segs.insert(s.name.clone(), (mesh, material));
    }
    let legs = leg_rigs(&plan);
    // The v0 face cue: a thin dark quad across the head's upper front.
    let face_mesh = meshes.add(Cuboid::new(0.2, 0.06, 0.02));
    let face_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.11),
        perceptual_roughness: 0.9,
        unlit: fullbright,
        ..default()
    });
    Some(BodyAssets {
        plan,
        idle,
        walk,
        segs,
        legs,
        face: (face_mesh, face_material),
    })
}

/// Spawn one body's entity hierarchy from its plan and return the instance.
/// Joint entities are spawned first (so parenting can wire an arbitrary tree
/// regardless of segment order), each carrying a mesh child offset from its
/// pivot; then the tree is stitched with `add_child`. Nothing here is
/// biped-specific — it walks whatever tree the plan declares.
fn spawn_body(
    commands: &mut Commands,
    assets: &BodyAssets,
    plan_name: &str,
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
        plan: plan_name.to_string(),
    }
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
