//! Player edits as commands: crosshair voxel raycast + LMB break / RMB place.
//!
//! Input handlers never touch the `ChunkMap` — they submit
//! `dc:world/set_block` envelopes (player priority class) into the embedded
//! [`Authority`]; the visible change arrives when the host tick's receipt is
//! applied to the cache and the chunk remeshes. The raycast is dc-core's DDA
//! walk over the same solidity query collision uses, so what you aim at is
//! exactly what you stand on.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use dc_core::{RaycastHit, raycast_voxels};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::app::{ChunkMap, CurrentScale, FloatingOrigin, Terrain, to_render};
use crate::authority::Authority;
use crate::player::{EYE_FRACTION, PLAYER_WIDTH_M, Player};
use dc_api::{Payload, Vec3i, payload};

/// Edit reach in meters (converted to voxel units at the current scale).
const REACH_M: f64 = 5.0;
/// The block RMB places (a placeable-block selection UI is a later slice).
const PLACE_BLOCK: &str = "dc:stone";

/// The voxel under the crosshair this frame, if any.
#[derive(Resource, Default)]
pub struct CrosshairTarget(pub Option<RaycastHit>);

/// Raycast from the eye along the view direction against the same solidity
/// query used for collision (loaded cache first, generator fallback).
pub fn update_target(
    terrain: Res<Terrain>,
    scale: Res<CurrentScale>,
    map: Res<ChunkMap>,
    player: Res<Player>,
    mut target: ResMut<CrosshairTarget>,
) {
    let vscale = scale.scale;
    let eye_m = player.pos_m + DVec3::new(0.0, EYE_FRACTION * PLAYER_HEIGHT_M, 0.0);
    // Meters -> voxel units is a uniform scale, so the direction is unchanged.
    let origin_v = eye_m * vscale.voxels_per_meter();
    let solid = |x: i64, y: i64, z: i64| map.is_solid(&terrain.0, vscale, x, y, z);
    target.0 = raycast_voxels(
        &solid,
        origin_v,
        player.view_dir(),
        vscale.meters_to_voxels(REACH_M),
    );
}

/// LMB breaks the targeted voxel, RMB places against the targeted face — both
/// as `dc:world/set_block` commands. Only acts while the cursor was already
/// captured before this frame's click (the capturing click itself must not
/// edit), tracked via `was_locked`.
pub fn apply_edits(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor_options: Single<&CursorOptions>,
    target: Res<CrosshairTarget>,
    scale: Res<CurrentScale>,
    player: Res<Player>,
    mut authority: ResMut<Authority>,
    mut was_locked: Local<bool>,
) {
    let locked = cursor_options.grab_mode != CursorGrabMode::None;
    let was = *was_locked;
    *was_locked = locked;
    if !(locked && was) {
        return;
    }
    let Some(hit) = target.0 else {
        return;
    };
    if mouse.just_pressed(MouseButton::Left) {
        authority.submit_player(Payload::SetBlock(payload::SetBlock {
            pos: Vec3i::new(hit.voxel.0, hit.voxel.1, hit.voxel.2),
            block: "dc:air".into(),
        }));
    }
    if mouse.just_pressed(MouseButton::Right) && hit.normal != (0, 0, 0) {
        let place = hit.adjacent();
        if !voxel_overlaps_player(place, &player, scale.scale) {
            authority.submit_player(Payload::SetBlock(payload::SetBlock {
                pos: Vec3i::new(place.0, place.1, place.2),
                block: PLACE_BLOCK.into(),
            }));
        }
    }
}

/// Would placing a block in this voxel intersect the player's collision box?
fn voxel_overlaps_player(
    voxel: (i64, i64, i64),
    player: &Player,
    vscale: dc_core::VoxelScale,
) -> bool {
    let to_v = vscale.voxels_per_meter();
    let half_w = (PLAYER_WIDTH_M / 2.0) * to_v;
    let height = PLAYER_HEIGHT_M * to_v;
    let p = player.pos_m * to_v; // feet, voxel units
    let (min, max) = (
        DVec3::new(p.x - half_w, p.y, p.z - half_w),
        DVec3::new(p.x + half_w, p.y + height, p.z + half_w),
    );
    let (vx, vy, vz) = (voxel.0 as f64, voxel.1 as f64, voxel.2 as f64);
    min.x < vx + 1.0
        && max.x > vx
        && min.y < vy + 1.0
        && max.y > vy
        && min.z < vz + 1.0
        && max.z > vz
}

/// A small always-on crosshair dot at screen center, so walks (human or
/// agent-driven via screenshots) can see what the player is aiming at.
pub fn setup_crosshair(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            margin: UiRect {
                left: Val::Px(-2.0),
                top: Val::Px(-2.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
    ));
}

/// Wireframe outline around the targeted voxel (cheap gizmo cuboid, positioned
/// origin-relative like everything the GPU sees).
pub fn draw_target(
    target: Res<CrosshairTarget>,
    origin: Res<FloatingOrigin>,
    scale: Res<CurrentScale>,
    mut gizmos: Gizmos,
) {
    let Some(hit) = target.0 else {
        return;
    };
    let vs = scale.scale.voxel_size_m();
    let center_m = DVec3::new(
        (hit.voxel.0 as f64 + 0.5) * vs,
        (hit.voxel.1 as f64 + 0.5) * vs,
        (hit.voxel.2 as f64 + 0.5) * vs,
    );
    gizmos.cube(
        Transform::from_translation(to_render(center_m - origin.0))
            .with_scale(Vec3::splat(vs as f32 * 1.004)),
        Color::srgba(0.05, 0.05, 0.05, 0.9),
    );
}
