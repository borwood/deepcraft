//! Player state, input, fly/walk movement, and the camera.
//!
//! Simulation positions are f64 meters (world space); rendering happens
//! relative to the floating origin. Walk mode runs dc-core's swept-AABB
//! collision in voxel units — the conversion happens here, at the edge.

use std::cell::RefCell;

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use dc_core::{Aabb, move_aabb};
use glam::DVec3;

use crate::PLAYER_HEIGHT_M;
use crate::app::{CurrentScale, FloatingOrigin, to_render};
use crate::authority::Authority;

/// Player collision width in meters (x and z).
pub const PLAYER_WIDTH_M: f64 = 0.6;
/// Eye height as a fraction of player height.
pub const EYE_FRACTION: f64 = 0.9;
const WALK_SPEED_M_S: f64 = 4.5;
const FLY_SPEED_M_S: f64 = 16.0;
/// The world's gravity. Read from dc-core's one authority rather than restated
/// here — this file used to hold an independent hand-typed `25.0` that matched
/// `CharacterConfig`'s only by coincidence, so the player and every character
/// fell at the same rate by luck (DECIDED 2026-08-02, user: gravity is a world
/// constant defaulting to Earth — `ARCHITECTURE.md`, and the two-authority
/// family in `bodies.md` § the sim owns the target).
const GRAVITY_M_S2: f64 = dc_core::DEFAULT_GRAVITY_M_S2;
/// Jump clears this many voxel heights (with a little margin), so the jump
/// arc is part of what changes between scales — deliberately so.
const JUMP_CLEARANCE_VOXELS: f64 = 1.3;
const MOUSE_SENSITIVITY: f32 = 0.0025;
/// Cap on the physics step so a hitch doesn't turn into a huge teleport.
const MAX_STEP_S: f64 = 0.05;

#[derive(Resource)]
pub struct Player {
    /// Feet position, world-space meters.
    pub pos_m: DVec3,
    /// Velocity in m/s (walk mode).
    pub vel_m: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fly: bool,
    pub on_ground: bool,
}

impl Player {
    pub fn new(pos_m: DVec3) -> Self {
        Self {
            pos_m,
            vel_m: DVec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            fly: true,
            on_ground: false,
        }
    }

    /// Unit view direction from yaw/pitch (bevy convention: -Z forward at
    /// yaw 0). Shared by the crosshair raycast and anything else that aims.
    pub fn view_dir(&self) -> DVec3 {
        let (sin_yaw, cos_yaw) = (f64::from(self.yaw.sin()), f64::from(self.yaw.cos()));
        let (sin_pitch, cos_pitch) = (f64::from(self.pitch.sin()), f64::from(self.pitch.cos()));
        DVec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }
}

pub fn update_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    cursor_options: Single<&CursorOptions>,
    mut authority: ResMut<Authority>,
    scale: Res<CurrentScale>,
    mut player: ResMut<Player>,
) {
    let dt = f64::from(time.delta_secs()).min(MAX_STEP_S);

    // Mouse look, only while the cursor is captured.
    if cursor_options.grab_mode != CursorGrabMode::None {
        player.yaw -= motion.delta.x * MOUSE_SENSITIVITY;
        player.pitch = (player.pitch - motion.delta.y * MOUSE_SENSITIVITY).clamp(-1.55, 1.55);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        player.fly = !player.fly;
        player.vel_m = DVec3::ZERO;
    }

    // Horizontal basis from yaw (bevy convention: -Z is forward at yaw 0).
    let (sin_yaw, cos_yaw) = (f64::from(player.yaw.sin()), f64::from(player.yaw.cos()));
    let forward = DVec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = DVec3::new(cos_yaw, 0.0, -sin_yaw);

    let axis = |neg: KeyCode, pos: KeyCode| -> f64 {
        f64::from(keys.pressed(pos)) - f64::from(keys.pressed(neg))
    };
    let wish =
        forward * axis(KeyCode::KeyS, KeyCode::KeyW) + right * axis(KeyCode::KeyA, KeyCode::KeyD);
    let wish = if wish.length_squared() > 0.0 {
        wish.normalize()
    } else {
        DVec3::ZERO
    };

    if player.fly {
        // Free fly, no clipping: Space/Shift for up/down.
        let up = axis(KeyCode::ShiftLeft, KeyCode::Space);
        player.pos_m += (wish + DVec3::new(0.0, up, 0.0)) * FLY_SPEED_M_S * dt;
        player.on_ground = false;
        return;
    }

    // Walk mode: swept-AABB against the voxel grid, with gravity and jump.
    let vscale = scale.scale;
    player.vel_m.x = wish.x * WALK_SPEED_M_S;
    player.vel_m.z = wish.z * WALK_SPEED_M_S;
    player.vel_m.y -= GRAVITY_M_S2 * dt;
    if player.on_ground && keys.just_pressed(KeyCode::Space) {
        let jump_height_m = JUMP_CLEARANCE_VOXELS * vscale.voxel_size_m();
        player.vel_m.y = (2.0 * GRAVITY_M_S2 * jump_height_m).sqrt();
    }

    // Meters -> voxel units for the sweep, back to meters for the state.
    let to_voxels = |v: DVec3| v * vscale.voxels_per_meter();
    let aabb = Aabb::from_bottom_center(
        to_voxels(player.pos_m),
        vscale.meters_to_voxels(PLAYER_WIDTH_M / 2.0),
        vscale.meters_to_voxels(PLAYER_HEIGHT_M),
    );
    // Collision reads the AUTHORITY's solidity (edits included), lazily
    // generating any unstreamed chunk at a streaming edge — never the client
    // cache's old wrong-world S1 fallback (journal/0017). The `RefCell` gives
    // the `Fn`-typed `VoxelQuery` interior-mutable access to the hosted world.
    let authority_cell = RefCell::new(&mut *authority);
    let solid = |x: i64, y: i64, z: i64| authority_cell.borrow_mut().is_solid_voxel(x, y, z);
    let result = move_aabb(&solid, aabb, to_voxels(player.vel_m * dt));

    player.pos_m += result.delta * vscale.voxel_size_m();
    if result.hit_y {
        player.vel_m.y = 0.0;
    }
    player.on_ground = result.on_ground;
}

pub fn update_camera(
    player: Res<Player>,
    origin: Res<FloatingOrigin>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let eye = player.pos_m + DVec3::new(0.0, EYE_FRACTION * PLAYER_HEIGHT_M, 0.0);
    camera.translation = to_render(eye - origin.0);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
}
