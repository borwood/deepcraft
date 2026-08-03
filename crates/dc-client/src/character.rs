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
//! ([`crate::body`]) and never read back into simulation. The sim state read is
//! each body's **speed** (which grades the derived gait up its Froude ladder),
//! its **posture**, and its **target trunk facing** — all legal one-way reads.
//! Note the third is new (user call #5, 2026-08-02): the sim owns the facing
//! target because B4 buckets colliders by yaw and B5 resolves damage against
//! the nominal pose; the client owns only the turn rate toward it.
//!
//! **Plans come from the REGISTRY, per character.** This used to call
//! `biped_plan()` / `biped_clips()` as compiled-in Rust; the default pack now
//! arrives as a registry command batch at world construction
//! (`authority.rs::load_body_packs`) and this system reads
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
use dc_api::bodies::{AnimClip, BodyPlan, GaitVector};
use dc_core::{VoxelQuery, VoxelScale};
use glam::DVec3;

use crate::app::{CurrentScale, FloatingOrigin, Fullbright, to_render};
use crate::authority::Authority;
use crate::body::{
    AnimState, Cervical, LegRig, Reach, derived_gait, derived_root_delta_m, fk_foot_local,
    leg_rigs, pose_for, resolve_orientation, root_offset_m, solve_leg_ik,
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

/// One plan resolved for rendering: the plan itself, the clips its action
/// bindings name, the per-segment mesh+material, and the leg rigs derived from
/// its own proportions. Built once per plan and shared across every body of it.
struct BodyAssets {
    plan: BodyPlan,
    /// The non-locomotion clip that rides as an additive layer (design § 6).
    /// **There is no `walk` any more** — locomotion is [`BodyAssets::gait`],
    /// and `dc:anim/biped_walk` retired as content 2026-08-02 (user call #3).
    idle: AnimClip,
    /// The DERIVED gait (`dc_api::bodies::bake_gait`), baked once per plan
    /// against the world's gravity. `None` = the bake declined and this body
    /// stands in its resting pose with clips only (identity fallback, warned
    /// once at build) — it never moves wrongly.
    gait: Option<GaitVector>,
    /// Segment name → (cuboid mesh, tinted material).
    segs: HashMap<String, (Handle<Mesh>, Handle<StandardMaterial>)>,
    /// The legs' IK rigs (empty if the plan declares no `sole` roles), derived
    /// from THIS plan's bone lengths — the whole retargeting story is here.
    /// The rigs stay AUTHORED geometry; the bake's `root_delta_m` is applied
    /// at the consumer (the pose loop), not inside the rig.
    legs: Vec<LegRig>,
    /// The plan's DECLARED cervical range (B7 § 5.4), resolved once here — the
    /// home of the two `NECK_*_CLAMP_RAD` constants that used to be world-global
    /// and blind to the plan.
    cervical: Cervical,
    /// The DERIVED resting root height, metres (posture bake, 2026-08-02:
    /// `bake_resting_posture(plan, "stand")` — the feet pin the pelvis, so
    /// the root lands at chain reach: biped 0.880, stout 0.440, longleg
    /// 1.020). `None` = the bake declined and this plan renders with the
    /// authored pivot as hip, exactly as it always did (identity fallback,
    /// warned once at build).
    derived_root_m: Option<f64>,
    /// `derived_root_m − authored root pivot Y`, or `0.0` on the identity
    /// fallback — the delta the pose loop adds to the root segment's
    /// translation AND to the IK hip, so the rendered body and the solver
    /// agree on where the pelvis is.
    root_delta_m: f64,
    /// v0 face cue: a small dark brow band parented to the face segment's front
    /// (−Z) face, so orientation is photographable (placeholder until head
    /// textures).
    face: (Handle<Mesh>, Handle<StandardMaterial>),
    /// The segment the look-at drives — the plan's unique `look` role, resolved
    /// once here (unique-or-loud; `None` = feature off, which is what a silent
    /// name miss used to mean). Was `s.name == "neck"`.
    look_joint: Option<String>,
    /// The segment wearing the face cue — the plan's unique `face` role.
    /// Was `s.name == "head"`.
    face_segment: Option<String>,
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
    // The WORLD's gravity, read from the one authority rather than restated —
    // the gait's Froude chain closes against it, and a gait baked at the wrong
    // `g` is coherent and wrong (the closed-system scale error, one tier down).
    let gravity_m_s2 = authority.world.character_config().gravity_m_s2;

    // Resolve every plan in use from the REGISTRY (not a compiled-in call), once
    // per plan. Done before the authority is borrowed mutably for the ground
    // query below: the registry read and the solidity read are separate passes.
    for character in &characters {
        let name = character.body_plan.as_str();
        if visuals.plans.contains_key(name) || visuals.missing_plans.contains(name) {
            continue;
        }
        match build_plan_assets(
            name,
            &authority,
            gravity_m_s2,
            &mut meshes,
            &mut materials,
            fullbright.0,
        ) {
            Some(assets) => {
                match assets.derived_root_m {
                    Some(root_m) => info!(
                        "body plan `{name}` resolved from the registry for rendering \
                         (derived standing root {root_m:.3} m, {:+.3} m vs authored)",
                        assets.root_delta_m
                    ),
                    None => info!(
                        "body plan `{name}` resolved from the registry for rendering \
                         (authored root pivot as hip — posture bake declined, see warning)"
                    ),
                }
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
    let CharacterVisuals { bodies, plans, .. } = &mut *visuals;
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
        let plan_changed = bodies
            .get(character.name.as_str())
            .is_some_and(|i| i.plan != character.body_plan);
        if plan_changed && let Some(instance) = bodies.remove(character.name.as_str()) {
            commands.entity(instance.root).despawn();
        }

        match bodies.get_mut(character.name.as_str()) {
            Some(instance) => {
                // The gait is graded CONTINUOUSLY over Froude — no threshold,
                // no state, no crossfade (user call #1). Idle is the ladder's
                // degenerate limit and needs no branch here or anywhere.
                instance.anim.advance(dt, assets.gait.as_ref(), speed);
                // The trunk chases the SIM's target facing (user call #5): the
                // sim owns the target, the client owns the approach.
                instance.anim.steer(dt, f64::from(character.facing_yaw));
                let pose = pose_for(&instance.anim, assets.gait.as_ref(), &[&assets.idle]);
                // Trunk faces travel; head/neck follow the look (the walk-8 gap).
                let orient = resolve_orientation(
                    instance.anim.trunk_yaw,
                    f64::from(character.yaw),
                    f64::from(character.pitch),
                    assets.cervical,
                );

                // ---- THE ONE VERTICAL COMPOSITION (design § 4.3) ------------
                // Evaluated once per body per frame and read by BOTH the IK hip
                // and the render root below. Nothing else in this loop may add
                // a vertical term — that is asserted structurally by
                // `the_render_root_and_the_ik_hip_read_one_root_offset`, and it
                // is what makes corrections #80's drift impossible rather than
                // merely fixed.
                let root_offset = root_offset_m(
                    assets.gait.as_ref(),
                    assets.root_delta_m,
                    instance.anim.stepped_froude,
                    instance.anim.stepped_phase,
                    character.posture,
                );

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
                    // The DERIVED hip: authored hip + THE ONE ROOT OFFSET —
                    // literally the same number the root segment's translation
                    // takes below, so the solver and the rendered pelvis cannot
                    // disagree. Reading it from a binding rather than
                    // re-composing it is the whole fix.
                    let hip_y = feet.y + leg.hip_local[1] + root_offset;
                    let (dfx, dfz) = rotate_y_xz(trunk, 0.0, fz);
                    let foot_x = feet.x + hx + dfx;
                    let foot_z = feet.z + hz + dfz;
                    let foot_y = hip_y + fy;
                    if let Some(ground) =
                        ground_top_m(&solid, vscale, foot_x, foot_z, foot_y, half_voxel)
                    {
                        let adjust = ground - foot_y;
                        if adjust.abs() > 1e-3 && adjust.abs() <= half_voxel {
                            // B7: the limits ride ON the rig, so there is no
                            // way to ask for a pose that ignores the joint.
                            let ik = solve_leg_ik(leg, [0.0, fy + adjust, fz]);
                            // One added condition (B7 § 5.3): skip the override
                            // when the solve leaves the LIMIT SET, leaving the
                            // clip pose — the foot floats honestly rather than
                            // lying about contact, exactly as it already does
                            // outside the half-voxel window. `BeyondExtension`
                            // is the annulus's outer edge, not a limit, and
                            // keeps applying as it always did.
                            let within_limits = !matches!(
                                ik.reach,
                                Reach::BeyondFlexion { .. } | Reach::JointBlocked { .. }
                            );
                            if within_limits && ik.upper_x.is_finite() && ik.lower_x.is_finite() {
                                // Applied exactly. The 11.25° snap that used to sit
                                // here was removed 2026-08-01 (bodies.md § stepped
                                // animation): it could not express the ~1° a planted
                                // foot needs, so the solved answer was rounded away.
                                leg_overrides.insert(leg.upper.clone(), [ik.upper_x, 0.0, 0.0]);
                                leg_overrides.insert(leg.lower.clone(), [ik.lower_x, 0.0, 0.0]);
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
                        // The rendered root: authored pivot + THE ONE ROOT
                        // OFFSET. Identity 0.0 when the plan can neither bake a
                        // resting posture nor a gait, so a fallback plan
                        // renders exactly as it always did.
                        t.y += root_offset as f32;
                    }
                    // IK override on a leg joint; the declared look joint
                    // composes the look on top of its clip pose (B0 — was a
                    // silent `== "neck"` name check); everything else is the
                    // clip pose.
                    let e = if let Some(o) = leg_overrides.get(&s.name) {
                        *o
                    } else {
                        let base = pose.joints.get(&s.name).copied().unwrap_or([0.0, 0.0, 0.0]);
                        if Some(s.name.as_str()) == assets.look_joint.as_deref() {
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
                character.facing_yaw,
            )),
        }
    }

    // Spawn new bodies (after the read-only pass over `bodies`).
    for (name, plan, feet, facing_yaw) in to_spawn {
        let Some(assets) = plans.get(plan.as_str()) else {
            continue;
        };
        let translation = to_render(feet - origin.0);
        let instance = spawn_body(&mut commands, assets, &plan, translation, facing_yaw);
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
/// the leg rigs derived from this plan's own proportions, the baked resting-root
/// delta (the derived hip — computed once per plan here, exactly the
/// consumer-memoization the posture-bake audit § 3(b) placed), and the face cue.
///
/// `None` when the registry holds no such plan, or the plan's required slots bind
/// clips that are not registered — a content failure the caller reports rather
/// than papering over with a hard-coded biped.
fn build_plan_assets(
    name: &str,
    authority: &Authority,
    gravity_m_s2: f64,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    fullbright: bool,
) -> Option<BodyAssets> {
    let plan = authority.world.body_plan(name)?.plan.clone();
    // Actions → clips through the plan's own bindings, not a hard-coded clip
    // name. The vocabulary is open (B0); `idle` here is what THIS driver asks
    // for — a plan that lacks it is a content failure the caller reports
    // (`None`), never a pose the engine invents.
    //
    // **`walk` is no longer asked for.** Locomotion is derived below, so a plan
    // needs no walk clip and the default pack ships none (user call #3).
    let clip_for = |action: &str| -> Option<AnimClip> {
        let bound = plan.actions.iter().find(|a| a.action == action)?;
        Some(authority.world.anim_clip(&bound.clip)?.clip.clone())
    };
    let idle = clip_for("idle")?;
    // Role queries, resolved once per plan (B0, unique-or-loud): ambiguity
    // names the contenders and disables the feature; absence is feature-off —
    // exactly what a missed name equality used to mean, minus the silence.
    let look_joint = match dc_api::unique_role_segment(&plan, "look") {
        Ok(seg) => seg.map(|s| s.name.clone()),
        Err(e) => {
            warn!("{e}; look-at disabled for this plan");
            None
        }
    };
    let face_segment = match dc_api::unique_role_segment(&plan, "face") {
        Ok(seg) => seg.map(|s| s.name.clone()),
        Err(e) => {
            warn!("{e}; face cue disabled for this plan");
            None
        }
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
            unlit: fullbright,
            ..default()
        });
        segs.insert(s.name.clone(), (mesh, material));
    }
    let legs = leg_rigs(&plan);
    // B7's declared cervical range, and its band reports. A declared bound
    // WIDER than the plan's own self-contact geometry is reported, never
    // refused (a clockwork golem may want to bend wrong) — so it warns once,
    // here, beside every other per-plan derivation.
    let cervical = Cervical::of(&plan);
    for report in &dc_api::bodies::derive_joint_limits(&plan).reports {
        warn!("body plan `{name}`: {report}");
    }
    // The DERIVED resting root (posture-bake consumer slice, 2026-08-02; audit
    // § 5): stop pinning the pelvis at the authored hip — bake the standing
    // posture once per plan and cache the root delta beside the rigs. On any
    // decline (a tree's legal absence, an out-of-domain geometry) the IDENTITY
    // FALLBACK is today's behaviour exactly: authored pivot as hip, delta 0,
    // one warn naming why.
    let (derived_root_m, root_delta_m) = match derived_root_delta_m(&plan) {
        Ok(delta) => {
            let authored = plan
                .segments
                .iter()
                .find(|s| s.parent.is_none())
                .map_or(0.0, |s| s.pivot_m[1]);
            (Some(authored + delta), delta)
        }
        Err(why) => {
            warn!(
                "body plan `{name}`: resting-posture bake declined ({why}); rendering \
                 with the authored root pivot as hip (identity fallback)"
            );
            (None, 0.0)
        }
    };
    // The DERIVED GAIT (gait member #1's consumer slice, 2026-08-02): baked
    // once per plan against the world's gravity, then evaluated per frame at
    // the body's own Froude number. On a decline the body stands still with
    // clips only — the identity fallback, warned once naming why, because a
    // body that cannot bake a gait must not move wrongly.
    let gait = match derived_gait(&plan, gravity_m_s2) {
        Ok(g) => Some(g),
        Err(why) => {
            warn!(
                "body plan `{name}`: gait bake declined ({why}); this body will stand \
                 in its resting pose and play only its non-locomotion clips"
            );
            None
        }
    };
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
        gait,
        segs,
        legs,
        cervical,
        derived_root_m,
        root_delta_m,
        face: (face_mesh, face_material),
        look_joint,
        face_segment,
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
    facing_yaw: f32,
) -> BodyInstance {
    let root = commands
        .spawn((
            CharacterBody,
            BodySegment,
            Transform::from_translation(translation)
                .with_rotation(Quat::from_rotation_y(facing_yaw)),
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
        // v0 face cue: a dark brow band on the face segment's front (−Z) face,
        // so the body's facing is photographable (walk-8: orientation was
        // unverifiable on a featureless head). Placeholder until head textures
        // land; the segment is the plan's declared `face` role (B0 — was a
        // silent `== "head"` name check).
        if Some(s.name.as_str()) == assets.face_segment.as_deref() {
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
        // Start at the sim's target facing so the trunk doesn't swing to it
        // from an arbitrary zero on the first steps.
        anim: AnimState::facing(f64::from(facing_yaw)),
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
