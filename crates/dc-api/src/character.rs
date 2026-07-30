//! Character primitive — a persistent named body with a controller binding
//! (docs/API.md § Characters, controllers, and the two MCP surfaces).
//!
//! Added by the character-MCP-surface milestone (additive). A character is a
//! world entity with a real body: feet position, yaw/pitch, and an AABB with
//! player-style dimensions, stepped against dc-core's swept-AABB collision on
//! every host tick. Whoever holds `Grant::CharacterControl` for its name is
//! its controller; the controller verbs (`dc:character/set_move_intent`,
//! `set_look`, `jump`) are ordinary commands through the one door —
//! tick-quantized, receipted, replayable — that write the character's
//! *controller input*, and the tick step integrates that input into motion.
//!
//! Stepping model (documented choice): characters integrate **on the host
//! tick** with a fixed `tick_dt_s`, not per rendered frame. Commands already
//! apply at tick boundaries, so inputs and motion share one clock and
//! `seed + command log + tick count = identical trajectory` — the replay
//! property costs nothing extra. The client's 20 Hz host cadence makes a tick
//! 50 ms of sim time.
//!
//! All world I/O here is in **meters** (the entity convention, [`Vec3f`]);
//! the conversion to voxel units happens at the sweep edge via
//! [`CharacterConfig::voxel_size_m`], exactly like the client's player
//! movement code (dc-client player.rs, the reference for these dynamics).

use glam::DVec3;
use serde::{Deserialize, Serialize};

use dc_core::{Aabb, VoxelQuery, aabb_overlaps_solid, move_aabb};

use crate::payload::Vec3f;

/// Body dimensions and movement dynamics for character simulation.
/// The defaults mirror the client's player at the scale-3 voxel size
/// (1.8 m / 3 voxels); the client overrides `voxel_size_m` per scale.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct CharacterConfig {
    /// Edge length of one voxel in meters (couples the meter-space body to
    /// the voxel-space collision query).
    pub voxel_size_m: f64,
    /// Body height in meters (the AABB's Y extent).
    pub height_m: f64,
    /// Body width in meters (the AABB's X and Z extent).
    pub width_m: f64,
    /// Eye height as a fraction of body height.
    pub eye_fraction: f64,
    /// Full walk speed in m/s (a move intent's `speed` scales this).
    pub walk_speed_m_s: f64,
    pub gravity_m_s2: f64,
    /// Jump apex clears this many voxel heights (scale-dependent on purpose,
    /// like the player's).
    pub jump_clearance_voxels: f64,
    /// Simulated seconds per host tick.
    pub tick_dt_s: f64,
}

impl Default for CharacterConfig {
    fn default() -> Self {
        Self {
            voxel_size_m: 1.8 / 3.0,
            height_m: 1.8,
            width_m: 0.6,
            eye_fraction: 0.9,
            walk_speed_m_s: 4.5,
            gravity_m_s2: 25.0,
            jump_clearance_voxels: 1.3,
            tick_dt_s: 1.0 / 20.0,
        }
    }
}

/// Parametric posture — the sim-visible half of the crouch firewall split
/// (docs/design/bodies.md § determinism firewall). v0 postures: `Standing` and
/// `Crouching`. This is *discrete, deterministic* state that alters the
/// swept-AABB collider height at the tick boundary; the cosmetic pose (spine
/// lowered, legs bent via IK, head keeps look) is derived client-side from it
/// and never read back. Set by the `dc:character/set_posture` controller verb.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Posture {
    #[default]
    Standing,
    Crouching,
}

/// The swept-AABB height multiplier a crouching body collides with. Chosen at
/// 0.6× — an honest "hunker": low enough to clear a two-thirds-height gap a
/// standing body cannot, high enough that the mover still reads as a person and
/// not a slab. Deterministic and a single discrete step, so it is firewall-legal
/// sim state.
pub const CROUCH_HEIGHT_FACTOR: f64 = 0.6;

impl Posture {
    /// Fraction of full body height this posture's collider occupies.
    pub fn height_factor(self) -> f64 {
        match self {
            Posture::Standing => 1.0,
            Posture::Crouching => CROUCH_HEIGHT_FACTOR,
        }
    }

    /// Parse the wire string (`standing` | `crouching`).
    pub fn from_wire(s: &str) -> Option<Posture> {
        match s {
            "standing" => Some(Posture::Standing),
            "crouching" => Some(Posture::Crouching),
            _ => None,
        }
    }

    /// The wire string (`standing` | `crouching`) — the inverse of
    /// [`Posture::from_wire`] and the exact vocabulary `dc:character/set_posture`
    /// accepts, so a `dc:character/pose` readback round-trips straight back into
    /// a posture command (walk-11 loose end: a driver could not read its own
    /// posture back).
    pub fn to_wire(self) -> &'static str {
        match self {
            Posture::Standing => "standing",
            Posture::Crouching => "crouching",
        }
    }
}

/// Controller input — what the controller verbs write and the tick step
/// consumes. Persist between ticks: a move intent keeps the character walking
/// until countermanded (set speed 0 to stop); a jump request is consumed by
/// the next step (it fires only if the character is on the ground then).
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct CharacterInput {
    /// Horizontal world-space movement direction (x, z); normalized by the
    /// step, zero = no horizontal intent.
    pub move_dir: (f64, f64),
    /// Fraction of full walk speed, clamped to [0, 1].
    pub speed: f64,
    /// One-shot jump request.
    pub jump: bool,
}

/// A character's full body state. Stored by the host, keyed by name.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CharacterState {
    pub name: String,
    /// Feet position, world-space meters.
    pub pos_m: Vec3f,
    /// Velocity, m/s.
    pub vel_m: Vec3f,
    /// Radians; bevy convention, 0 = looking toward -Z.
    pub yaw: f32,
    /// Radians; NEGATIVE looks down, positive up.
    pub pitch: f32,
    pub on_ground: bool,
    pub input: CharacterInput,
    /// Discrete collider posture (standing / crouching). Appended field: wire
    /// order is postcard identity, and `serde(default)` keeps old logs (which
    /// predate posture) decoding to `Standing`.
    #[serde(default)]
    pub posture: Posture,
    /// Which registered body plan this character **wears** (bodies.md § body
    /// plans; the renderer's per-character plan selection). Cosmetic by the
    /// determinism firewall — the collider comes from the world-global
    /// [`CharacterConfig`], never from the plan — but it is character state, so
    /// it lives here and rides the command log like everything else.
    ///
    /// Appended field: postcard order is wire identity, so it stays last, and
    /// `serde(default)` resolves pre-plan logs to the **identity default**
    /// [`crate::bodies::DEFAULT_BODY_PLAN`] rather than an empty string — a
    /// character recorded before plans existed was wearing the biped.
    #[serde(default = "default_body_plan")]
    pub body_plan: String,
}

/// serde's default for [`CharacterState::body_plan`]: the identity default.
fn default_body_plan() -> String {
    crate::bodies::DEFAULT_BODY_PLAN.to_string()
}

impl CharacterState {
    /// A new character wearing the identity-default body plan.
    pub fn new(name: impl Into<String>, pos_m: Vec3f) -> Self {
        Self::with_body_plan(name, pos_m, crate::bodies::DEFAULT_BODY_PLAN)
    }

    /// A new character wearing a named body plan. The caller (the host's
    /// `SpawnCharacter` arm) is responsible for having checked that the plan is
    /// registered — a name that is not gets a receipt, never a silent default.
    pub fn with_body_plan(
        name: impl Into<String>,
        pos_m: Vec3f,
        body_plan: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            pos_m,
            vel_m: Vec3f::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            input: CharacterInput::default(),
            posture: Posture::Standing,
            body_plan: body_plan.into(),
        }
    }

    /// Effective collider height in meters for the current posture (full height
    /// standing, scaled down crouching). The one place posture touches physics.
    pub fn effective_height_m(&self, cfg: &CharacterConfig) -> f64 {
        cfg.height_m * self.posture.height_factor()
    }

    /// Unit view direction from yaw/pitch (same convention as the client's
    /// player: -Z forward at yaw 0, negative pitch looks down).
    pub fn view_dir(&self) -> DVec3 {
        let (sin_yaw, cos_yaw) = (f64::from(self.yaw.sin()), f64::from(self.yaw.cos()));
        let (sin_pitch, cos_pitch) = (f64::from(self.pitch.sin()), f64::from(self.pitch.cos()));
        DVec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }

    /// Eye position in world meters. Follows posture: a crouching body's eyes
    /// ride at `eye_fraction` of the *shortened* collider, so its senses and
    /// `eye_in_solid` see from where the head actually is.
    pub fn eye_m(&self, cfg: &CharacterConfig) -> DVec3 {
        DVec3::new(
            self.pos_m.x,
            self.pos_m.y + cfg.eye_fraction * self.effective_height_m(cfg),
            self.pos_m.z,
        )
    }
}

/// Character names are bare slugs: they become consumer identities, grant
/// scopes, and MCP-visible keys. Same alphabet as the client's screenshot
/// names.
pub fn valid_character_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Integrate one tick of character motion: intent -> velocity, gravity, jump,
/// then a swept-AABB move against the voxel world (dc-core collision — the
/// same walk path as the player). `world` answers solidity in **voxel**
/// coordinates.
pub fn step_character(c: &mut CharacterState, cfg: &CharacterConfig, world: &impl VoxelQuery) {
    let dt = cfg.tick_dt_s;
    let to_voxels = 1.0 / cfg.voxel_size_m;

    // Horizontal velocity straight from intent (no inertia, like the player).
    let (dx, dz) = c.input.move_dir;
    let len = (dx * dx + dz * dz).sqrt();
    let (nx, nz) = if len > 1e-12 && len.is_finite() {
        (dx / len, dz / len)
    } else {
        (0.0, 0.0)
    };
    let speed = c.input.speed.clamp(0.0, 1.0) * cfg.walk_speed_m_s;
    c.vel_m.x = nx * speed;
    c.vel_m.z = nz * speed;
    c.vel_m.y -= cfg.gravity_m_s2 * dt;

    // A jump request is consumed by this step; it fires only from the ground.
    if c.input.jump {
        if c.on_ground {
            let jump_height_m = cfg.jump_clearance_voxels * cfg.voxel_size_m;
            c.vel_m.y = (2.0 * cfg.gravity_m_s2 * jump_height_m).sqrt();
        }
        c.input.jump = false;
    }

    let aabb = Aabb::from_bottom_center(
        DVec3::new(c.pos_m.x, c.pos_m.y, c.pos_m.z) * to_voxels,
        (cfg.width_m / 2.0) * to_voxels,
        c.effective_height_m(cfg) * to_voxels,
    );
    let delta_v = DVec3::new(c.vel_m.x, c.vel_m.y, c.vel_m.z) * dt * to_voxels;
    let result = move_aabb(world, aabb, delta_v);

    c.pos_m.x += result.delta.x * cfg.voxel_size_m;
    c.pos_m.y += result.delta.y * cfg.voxel_size_m;
    c.pos_m.z += result.delta.z * cfg.voxel_size_m;
    if result.hit_y {
        c.vel_m.y = 0.0;
    }
    c.on_ground = result.on_ground;
}

/// Would a *standing* body at `feet` embed in solid terrain? The stand-up guard
/// for a `crouching → standing` posture change: crouching may have carried the
/// body under a low ceiling, and standing back up must not push the taller
/// collider into solid voxels. Reuses the same `aabb_overlaps_solid` test the
/// attach embed guard uses (dc-client authority.rs) — one honest solidity check,
/// deterministic. `world` answers in **voxel** coordinates.
pub fn standing_would_embed(feet: Vec3f, cfg: &CharacterConfig, world: &impl VoxelQuery) -> bool {
    let to_voxels = 1.0 / cfg.voxel_size_m;
    let aabb = Aabb::from_bottom_center(
        DVec3::new(feet.x, feet.y, feet.z) * to_voxels,
        (cfg.width_m / 2.0) * to_voxels,
        cfg.height_m * to_voxels,
    );
    aabb_overlaps_solid(world, aabb)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat floor: everything below voxel y = 0 is solid.
    fn floor(_x: i64, y: i64, _z: i64) -> bool {
        y < 0
    }

    fn cfg() -> CharacterConfig {
        CharacterConfig::default()
    }

    #[test]
    fn character_names_are_bare_slugs() {
        for good in ["scout", "npc-7", "a_b", "x"] {
            assert!(valid_character_name(good), "{good:?}");
        }
        for bad in ["", "Scout", "a b", "a/b", "a:b", &"x".repeat(65)] {
            assert!(!valid_character_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn falls_settles_walks_and_jumps() {
        let cfg = cfg();
        let mut c = CharacterState::new("t", Vec3f::new(0.5, 3.0, 0.5));
        // Fall to the floor.
        for _ in 0..100 {
            step_character(&mut c, &cfg, &floor);
        }
        assert!(c.on_ground);
        assert!(
            c.pos_m.y.abs() < 1e-9,
            "feet on floor surface (y=0 m), got {}",
            c.pos_m.y
        );

        // Walk +X at full speed for one second of ticks.
        c.input.move_dir = (1.0, 0.0);
        c.input.speed = 1.0;
        let ticks = (1.0 / cfg.tick_dt_s) as usize;
        let x0 = c.pos_m.x;
        for _ in 0..ticks {
            step_character(&mut c, &cfg, &floor);
        }
        assert!(
            (c.pos_m.x - x0 - cfg.walk_speed_m_s).abs() < 1e-6,
            "walked {} m in 1 s",
            c.pos_m.x - x0
        );

        // Stop, then jump: apex clears the configured voxel height.
        c.input = CharacterInput::default();
        step_character(&mut c, &cfg, &floor);
        assert!(c.on_ground);
        c.input.jump = true;
        let mut apex: f64 = 0.0;
        for _ in 0..40 {
            step_character(&mut c, &cfg, &floor);
            apex = apex.max(c.pos_m.y);
        }
        assert!(!c.input.jump, "jump request consumed");
        assert!(c.on_ground, "landed again");
        let want = cfg.jump_clearance_voxels * cfg.voxel_size_m;
        assert!(apex > want * 0.8, "apex {apex} clears ~{want}");
    }

    #[test]
    fn airborne_jump_request_is_dropped() {
        let cfg = cfg();
        let mut c = CharacterState::new("t", Vec3f::new(0.5, 5.0, 0.5));
        c.input.jump = true;
        step_character(&mut c, &cfg, &floor); // airborne: no boost
        assert!(!c.input.jump);
        assert!(c.vel_m.y < 0.0, "still falling, jump dropped");
    }

    #[test]
    fn walls_block_and_slide() {
        let cfg = cfg();
        // Wall at voxel x >= 3 (i.e. 1.8 m in meters at default scale).
        let world = |x: i64, y: i64, _z: i64| y < 0 || x >= 3;
        let mut c = CharacterState::new("t", Vec3f::new(0.5, 0.0, 0.5));
        c.on_ground = true;
        c.input.move_dir = (1.0, 1.0);
        c.input.speed = 1.0;
        for _ in 0..60 {
            step_character(&mut c, &cfg, &world);
        }
        let wall_m = 3.0 * cfg.voxel_size_m;
        assert!(
            c.pos_m.x + cfg.width_m / 2.0 <= wall_m + 1e-9,
            "stopped at the wall"
        );
        assert!(c.pos_m.z > 3.0, "kept sliding along z");
    }

    #[test]
    fn view_dir_matches_convention() {
        let mut c = CharacterState::new("t", Vec3f::new(0.0, 0.0, 0.0));
        let v = c.view_dir();
        assert!(
            (v - DVec3::new(0.0, 0.0, -1.0)).length() < 1e-9,
            "yaw 0 = -Z"
        );
        c.pitch = -std::f32::consts::FRAC_PI_2;
        let v = c.view_dir();
        assert!(v.y < -0.999, "negative pitch looks down");
    }
}
