//! The authoritative world inside the client (client-through-dc-api
//! milestone).
//!
//! The client embeds dc-api's [`HostWorld`] as the single authority for world
//! **edits**: player input and MCP tool calls both become `CommandEnvelope`s
//! through the one door, the host ticks on a fixed cadence inside the Update
//! chain, and the receipts' effects are what mutate everything downstream —
//! the [`ChunkMap`] render/collision cache, the remesh dirty set, and the
//! dc-physics collider tiles. The `ChunkMap` is no longer a second authority:
//! chunks stream in as clones of the host's (terrain + edits), and edits only
//! reach it by receipt.
//!
//! Terrain reconciliation: the host is built with a generator closure over the
//! client's S1 [`TerrainGen`] (dc-api's pluggable-generator addition), so the
//! world the host serves is byte-identical to the world the client streams.
//! The far-mesh path still samples `TerrainGen` LOD grids directly — edits are
//! not visible beyond the full-detail radius (documented limit).

use std::cell::RefCell;
use std::collections::HashSet;

use bevy::prelude::*;
use dc_api::host::block_from_name;
use dc_api::payload::Vec3i;
use dc_api::schema::CommandKind;
use dc_api::{
    BlockChange, CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant,
    HostWorld, Payload, ReceiptEntry,
};
use dc_core::{Aabb, CHUNK_SIZE_USIZE, ChunkPos, VoxelScale, aabb_overlaps_solid, local_voxel};
use dc_mcp_dev::envelope_for_tool_call;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::PLAYER_HEIGHT_M;
use crate::app::{ChunkMap, ChunkMaterial, CurrentScale, FloatingOrigin, Terrain, to_render};
use crate::mcp::{BridgeRequest, McpBridge};
use crate::meshing::mesh_chunk;
use crate::physdemo::PhysicsDemo;
use crate::player::{PLAYER_WIDTH_M, Player};
use crate::streaming::to_bevy_mesh;
use crate::worldgen::{TerrainGen, true_surface_m};

/// Fixed host tick cadence, seconds. 20 Hz: an edit's receipt (and therefore
/// its remesh) lands within 50 ms of the tick boundary — imperceptible against
/// the swing animation we don't have yet.
pub const HOST_TICK_DT: f64 = 1.0 / 20.0;
/// Catch-up cap per frame (slow-frame spiral protection, same idea as
/// dc-physics' accumulator).
const MAX_TICKS_PER_FRAME: f64 = 4.0;

/// The embedded authoritative world plus the client's consumer identities.
#[derive(Resource)]
pub struct Authority {
    pub world: HostWorld,
    accumulator: f64,
    /// MCP command replies waiting for their tick-boundary receipt, keyed by
    /// (consumer identity, per-consumer seq).
    pending: Vec<(ConsumerId, u64, oneshot::Sender<Value>)>,
    /// Character-surface attach flows waiting for their spawn receipt:
    /// (surface consumer seq, character name, reply).
    pending_attach: Vec<(u64, String, oneshot::Sender<Value>)>,
    mcp_consumer: ConsumerId,
    mcp_token: CapabilityToken,
    player_consumer: ConsumerId,
    player_token: CapabilityToken,
    /// The character surface's parent authority: spawn characters + control
    /// any. Sessions never hold this — each session's token is ATTENUATED
    /// from it to exactly one character (attenuation can narrow, never
    /// widen — dc-api capability tests prove the partial order).
    character_parent_token: CapabilityToken,
    /// Consumer identity for surface-side acts (the attach flow's spawns).
    character_surface_consumer: ConsumerId,
    /// A private terrain generator (same seed as the world's) used only to pick
    /// the scan ceiling for the attach surface-snap — the analytic height is a
    /// cheap upper bound on the true voxel surface. Never the placement source
    /// itself: the snap and the embed guard read the world's real solidity.
    terrain: TerrainGen,
}

impl Authority {
    /// Build the hosted world over the client's terrain generator at the given
    /// scale. The generator closure owns its own `TerrainGen` (same seed =>
    /// identical noise), keeping the closure `Send + Sync` and deterministic.
    pub fn new(seed: i32, player_voxels: u32) -> Self {
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, player_voxels);
        let terrain = TerrainGen::new(seed);
        let mut world = HostWorld::with_generator(
            seed as u64,
            Box::new(move |pos| terrain.generate_chunk(scale, pos)),
        );
        // Character bodies share the player's dimensions and dynamics
        // (dc-api's defaults ARE the player constants); only the voxel size
        // follows the active scale. Part of the replay identity.
        world.set_character_config(dc_api::CharacterConfig {
            voxel_size_m: scale.voxel_size_m(),
            tick_dt_s: HOST_TICK_DT,
            ..dc_api::CharacterConfig::default()
        });
        Self {
            world,
            accumulator: 0.0,
            pending: Vec::new(),
            pending_attach: Vec::new(),
            mcp_consumer: ConsumerId::new(ConsumerKind::McpSession, "client-http"),
            // The in-client MCP session is a dev-grant session (documented in
            // journal/0002): unbounded world read/write, entity spawn, event
            // subscription, `dev:*` item authorship — the same grant set as
            // the standalone dc-mcp-dev surface.
            mcp_token: dc_mcp_dev::dev_session_token(),
            player_consumer: ConsumerId::new(ConsumerKind::Player, "local"),
            // v0 dev posture: the local player may edit anywhere. Survival
            // scoping (reach, gamemode) belongs to a later slice.
            player_token: CapabilityToken::new(vec![Grant::WorldWrite { volume: None }]),
            character_parent_token: CapabilityToken::new(vec![
                Grant::EntitySpawn,
                Grant::CharacterControl { character: None },
            ]),
            character_surface_consumer: ConsumerId::new(
                ConsumerKind::McpSession,
                "character-surface",
            ),
            terrain: TerrainGen::new(seed),
        }
    }

    /// Submit a player-sourced command (highest priority class). Receipts are
    /// tick-boundary artifacts consumed by [`tick_authority`]; submit-time
    /// rejections land in the receipt log and are simply dropped here.
    pub fn submit_player(&mut self, payload: Payload) {
        let envelope = CommandEnvelope {
            id: payload.command_id().to_string(),
            source: self.player_consumer.clone(),
            grant: self.player_token.clone(),
            payload,
            target_tick: None,
            txn: None,
        };
        let _ = self.world.submit(envelope);
    }

    /// Handle one MCP tool call against the dc-api surface with the dev
    /// session's identity and broad dev-grant token (the 7777 surface).
    pub fn handle_api_call(&mut self, tool: &str, args: &Value, reply: oneshot::Sender<Value>) {
        let consumer = self.mcp_consumer.clone();
        let token = self.mcp_token.clone();
        self.handle_api_call_as(consumer, token, tool, args, reply);
    }

    /// Handle one character-surface tool call: same registry-generated tool
    /// layer, but the identity is the session's character and the token is
    /// the parent token ATTENUATED to exactly that character. A payload that
    /// names any other character (or any non-character command) fails
    /// capability enforcement inside the host — the session's reach is the
    /// grant, not the tool list.
    pub fn handle_character_api(
        &mut self,
        tool: &str,
        args: &Value,
        session_character: &str,
        reply: oneshot::Sender<Value>,
    ) {
        let token =
            match self
                .character_parent_token
                .attenuate(vec![dc_api::Grant::CharacterControl {
                    character: Some(session_character.to_string()),
                }]) {
                Ok(token) => token,
                Err(e) => {
                    let _ = reply.send(json!({ "error": format!("attenuation failed: {e}") }));
                    return;
                }
            };
        // API.md § Characters: a character acts with the SAME priority class
        // as human player input — an embodied session is player-tier, not
        // tool-tier.
        let consumer = ConsumerId::new(
            ConsumerKind::Player,
            format!("character-{session_character}"),
        );
        self.handle_api_call_as(consumer, token, tool, args, reply);
    }

    /// Registry tool dispatch under an explicit identity + token. Queries
    /// answer immediately from the last completed tick; commands are
    /// submitted and their reply is delivered when the receipt materializes
    /// at the tick boundary.
    fn handle_api_call_as(
        &mut self,
        consumer: ConsumerId,
        token: CapabilityToken,
        tool: &str,
        args: &Value,
        reply: oneshot::Sender<Value>,
    ) {
        match envelope_for_tool_call(tool, args, &consumer, &token) {
            Err(e) => {
                let _ = reply.send(json!({ "error": e.to_string() }));
            }
            Ok((spec, envelope)) => match spec.kind {
                CommandKind::Query => {
                    let receipt = self.world.query(&envelope);
                    let _ = reply.send(
                        serde_json::to_value(receipt)
                            .unwrap_or_else(|e| json!({"error": e.to_string()})),
                    );
                }
                CommandKind::Command => match self.world.submit(envelope) {
                    Err(entry) => {
                        let _ = reply.send(
                            serde_json::to_value(&entry.receipt)
                                .unwrap_or_else(|e| json!({"error": e.to_string()})),
                        );
                    }
                    Ok(ack) => self.pending.push((consumer, ack.consumer_seq, reply)),
                },
            },
        }
    }

    /// Character-surface attach flow: bind a session to `name`, spawning the
    /// character (with the surface's parent token, not the session's) if it
    /// does not exist. The reply carries `{ ok, character, spawned }`; the
    /// session's MCP handler binds itself to the name only on `ok`.
    ///
    /// Placement is guarded (walk-5 finding, journal/0005): an embedded spawn is
    /// a permanent statue (swept collision won't move an interpenetrating box,
    /// and there is no despawn verb by design). If the body's AABB at `pos`
    /// overlaps solid voxels the attach is refused with a machine-readable
    /// receipt (`ok:false`, `code:"obstructed"`) and nothing is spawned. With
    /// `surface: true` the feet are first snapped to the TRUE voxel surface at
    /// `pos`'s x/z (mirroring the player's `surface` teleport) — reading the
    /// live world's solidity (edits included), not the analytic height that
    /// under-reports (ROADMAP Observed).
    pub fn handle_character_attach(
        &mut self,
        name: &str,
        pos: dc_api::payload::Vec3f,
        surface: bool,
        reply: oneshot::Sender<Value>,
    ) {
        if !dc_api::character::valid_character_name(name) {
            let _ = reply.send(json!({
                "ok": false,
                "code": "invalid_name",
                "error": "invalid character name: bare slug required ([a-z0-9_-], max 64 chars)",
            }));
            return;
        }
        if self.world.character(name).is_some() {
            let _ = reply.send(json!({ "ok": true, "character": name, "spawned": false }));
            return;
        }

        // Body dimensions and voxel lattice from the world's character config.
        let cfg = *self.world.character_config();
        let vs = cfg.voxel_size_m;
        let scale = VoxelScale::from_voxel_size_m(vs);
        let half_m = cfg.width_m / 2.0;

        // Optional surface snap: rest the feet on the true voxel surface under
        // the footprint at the requested x/z (the world's real solidity, edits
        // included; `terrain` only seeds each column's scan ceiling).
        let mut feet = pos;
        if surface {
            let terrain = &self.terrain;
            let world = RefCell::new(&mut self.world);
            let solid = |x: i64, y: i64, z: i64| {
                world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid()
            };
            feet.y = true_surface_m(
                &solid,
                |x, z| terrain.surface_height_m(x, z),
                scale,
                pos.x,
                pos.z,
                half_m,
            ) + 0.05;
        }

        // Embed guard: reject rather than create a stuck statue.
        let body = Aabb::from_bottom_center(
            glam::DVec3::new(feet.x, feet.y, feet.z) / vs,
            half_m / vs,
            cfg.height_m / vs,
        );
        let embedded = {
            let world = RefCell::new(&mut self.world);
            let solid = |x: i64, y: i64, z: i64| {
                world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid()
            };
            aabb_overlaps_solid(&solid, body)
        };
        if embedded {
            let _ = reply.send(json!({
                "ok": false,
                "code": "obstructed",
                "character": name,
                "error": format!(
                    "cannot attach `{name}`: body would be embedded in solid terrain at \
                     ({:.2}, {:.2}, {:.2}). Pass surface:true to drop onto the surface, \
                     or choose an open position.",
                    feet.x, feet.y, feet.z
                ),
            }));
            return;
        }

        let envelope = CommandEnvelope {
            id: dc_api::ids::CHARACTER_SPAWN.to_string(),
            source: self.character_surface_consumer.clone(),
            grant: self.character_parent_token.clone(),
            payload: Payload::SpawnCharacter(dc_api::payload::SpawnCharacter {
                name: name.to_string(),
                pos: feet,
            }),
            target_tick: None,
            txn: None,
        };
        match self.world.submit(envelope) {
            Err(entry) => {
                let _ = reply.send(json!({
                    "ok": false,
                    "error": format!("spawn rejected: {:?}", entry.receipt.result),
                }));
            }
            Ok(ack) => {
                self.pending_attach
                    .push((ack.consumer_seq, name.to_string(), reply));
            }
        }
    }

    /// Advance the fixed cadence by a frame delta; returns every receipt
    /// entry produced by the ticks that ran (possibly none).
    pub fn advance(&mut self, frame_dt: f64) -> Vec<ReceiptEntry> {
        self.accumulator =
            (self.accumulator + frame_dt.max(0.0)).min(HOST_TICK_DT * MAX_TICKS_PER_FRAME);
        let mut receipts = Vec::new();
        while self.accumulator >= HOST_TICK_DT {
            self.accumulator -= HOST_TICK_DT;
            receipts.extend(self.tick_now());
        }
        receipts
    }

    /// Complete one host tick immediately and route pending MCP replies.
    /// (Also the drive used by headless tests and pumps.)
    pub fn tick_now(&mut self) -> Vec<ReceiptEntry> {
        let receipts = self.world.tick();
        if !self.pending.is_empty() {
            let mut still = Vec::new();
            for (consumer, seq, reply) in self.pending.drain(..) {
                match receipts
                    .iter()
                    .find(|e| e.source == consumer && e.consumer_seq == seq)
                {
                    Some(entry) => {
                        let _ = reply.send(
                            serde_json::to_value(&entry.receipt)
                                .unwrap_or_else(|e| json!({"error": e.to_string()})),
                        );
                    }
                    None => still.push((consumer, seq, reply)),
                }
            }
            self.pending = still;
        }
        if !self.pending_attach.is_empty() {
            let mut still = Vec::new();
            for (seq, name, reply) in self.pending_attach.drain(..) {
                match receipts
                    .iter()
                    .find(|e| e.source == self.character_surface_consumer && e.consumer_seq == seq)
                {
                    Some(entry) => {
                        let value = match &entry.receipt.result {
                            dc_api::CommandResult::Ok(_) => {
                                json!({ "ok": true, "character": name, "spawned": true })
                            }
                            dc_api::CommandResult::Rejected(reason) => json!({
                                "ok": false,
                                "error": format!("spawn rejected: {reason}"),
                            }),
                        };
                        let _ = reply.send(value);
                    }
                    None => still.push((seq, name, reply)),
                }
            }
            self.pending_attach = still;
        }
        receipts
    }
}

/// Chunks whose meshes an edit at a world voxel invalidates: the containing
/// chunk plus every face-adjacent chunk the voxel borders (culled meshing only
/// consults the 6-neighborhood, so edge/corner-diagonal chunks are unaffected).
pub fn dirty_chunks_for_voxel(x: i64, y: i64, z: i64) -> Vec<ChunkPos> {
    let pos = ChunkPos::from_world_voxel(x, y, z);
    let (lx, ly, lz) = local_voxel(x, y, z);
    let mut out = vec![pos];
    let max = CHUNK_SIZE_USIZE - 1;
    if lx == 0 {
        out.push(ChunkPos::new(pos.x - 1, pos.y, pos.z));
    } else if lx == max {
        out.push(ChunkPos::new(pos.x + 1, pos.y, pos.z));
    }
    if ly == 0 {
        out.push(ChunkPos::new(pos.x, pos.y - 1, pos.z));
    } else if ly == max {
        out.push(ChunkPos::new(pos.x, pos.y + 1, pos.z));
    }
    if lz == 0 {
        out.push(ChunkPos::new(pos.x, pos.y, pos.z - 1));
    } else if lz == max {
        out.push(ChunkPos::new(pos.x, pos.y, pos.z + 1));
    }
    out
}

/// Apply authoritative block changes to the render/collision cache and return
/// the set of loaded chunks needing a remesh. Changes in chunks that are not
/// loaded are ignored — the cache refills from the host when they stream in.
pub fn apply_block_changes<'a>(
    map: &mut ChunkMap,
    changes: impl IntoIterator<Item = &'a BlockChange>,
) -> HashSet<ChunkPos> {
    let mut dirty = HashSet::new();
    for change in changes {
        let (x, y, z) = (change.pos.x, change.pos.y, change.pos.z);
        let pos = ChunkPos::from_world_voxel(x, y, z);
        let Some(block) = block_from_name(&change.to) else {
            continue; // the host validated it; unreachable in practice
        };
        if let Some(loaded) = map.loaded.get_mut(&pos) {
            let (lx, ly, lz) = local_voxel(x, y, z);
            loaded.chunk.set(lx, ly, lz, block);
            for d in dirty_chunks_for_voxel(x, y, z) {
                if map.loaded.contains_key(&d) {
                    dirty.insert(d);
                }
            }
        }
    }
    dirty
}

/// Loaded chunks whose meshes must be rebuilt after edits.
#[derive(Resource, Default)]
pub struct DirtyChunks(pub HashSet<ChunkPos>);

/// Drain MCP bridge requests into the authority (dc-api tool calls) or the
/// client shell (pose, screenshots). Runs every frame, before the host tick.
pub fn drain_bridge(
    bridge: Option<Res<McpBridge>>,
    mut authority: ResMut<Authority>,
    mut player: ResMut<Player>,
    terrain: Res<Terrain>,
    scale: Res<CurrentScale>,
    map: Res<ChunkMap>,
    mut commands: Commands,
) {
    let Some(bridge) = bridge else { return };
    let Ok(mut rx) = bridge.rx.lock() else { return };
    while let Ok(request) = rx.try_recv() {
        match request {
            BridgeRequest::Api { tool, args, reply } => {
                authority.handle_api_call(&tool, &args, reply);
            }
            BridgeRequest::PoseGet { reply } => {
                let _ = reply.send(pose_json(&player, &map, &terrain, scale.scale));
            }
            BridgeRequest::PoseSet {
                pos,
                yaw,
                pitch,
                surface,
                reply,
            } => {
                if let Some(p) = pos {
                    player.pos_m = glam::DVec3::new(p[0], p[1], p[2]);
                    player.vel_m = glam::DVec3::ZERO;
                }
                if surface {
                    // Walker-safe teleport: feet snap to the TRUE voxel surface
                    // under the player's footprint at (x, z), regardless of the
                    // requested y. Reads the live solidity (loaded edits, with a
                    // terrain fallback) via ChunkMap, not the analytic
                    // `surface_height_m` that under-reports on slopes and left
                    // the camera buried (journal/0004, ROADMAP Observed).
                    let vscale = scale.scale;
                    let solid = |x: i64, y: i64, z: i64| map.is_solid(&terrain.0, vscale, x, y, z);
                    player.pos_m.y = true_surface_m(
                        &solid,
                        |x, z| terrain.0.surface_height_m(x, z),
                        vscale,
                        player.pos_m.x,
                        player.pos_m.z,
                        PLAYER_WIDTH_M / 2.0,
                    ) + 0.05;
                    player.vel_m = glam::DVec3::ZERO;
                }
                if let Some(y) = yaw {
                    player.yaw = y;
                }
                if let Some(p) = pitch {
                    player.pitch = p.clamp(-1.55, 1.55);
                }
                let _ = reply.send(pose_json(&player, &map, &terrain, scale.scale));
            }
            BridgeRequest::Screenshot { name, reply } => {
                crate::mcp::take_screenshot(&mut commands, &name, reply);
            }
            BridgeRequest::CharacterAttach {
                name,
                pos,
                surface,
                reply,
            } => {
                let pos = match pos {
                    Some(p) => dc_api::payload::Vec3f::new(p[0], p[1], p[2]),
                    None => {
                        // Default: two meters in front of the player, one
                        // meter up — it lands where the player is looking and
                        // settles under gravity (a walker-visible entrance).
                        let (sin_yaw, cos_yaw) =
                            (f64::from(player.yaw.sin()), f64::from(player.yaw.cos()));
                        let spot =
                            player.pos_m + glam::DVec3::new(-sin_yaw * 2.0, 1.0, -cos_yaw * 2.0);
                        dc_api::payload::Vec3f::new(spot.x, spot.y, spot.z)
                    }
                };
                authority.handle_character_attach(&name, pos, surface, reply);
            }
            BridgeRequest::CharacterApi {
                tool,
                args,
                session_character,
                reply,
            } => {
                authority.handle_character_api(&tool, &args, &session_character, reply);
            }
        }
    }
}

fn pose_json(player: &Player, map: &ChunkMap, terrain: &Terrain, scale: VoxelScale) -> Value {
    // Eye = camera height: feet + 90% of player height (player.rs camera).
    // If this voxel is solid, every screenshot is backface nonsense — the
    // walk-3 lesson (journal/corrections.md #3): the walker must know.
    let eye = player.pos_m + glam::DVec3::new(0.0, PLAYER_HEIGHT_M * 0.9, 0.0);
    let eye_in_solid = map.is_solid(
        &terrain.0,
        scale,
        scale.voxel_at(eye.x),
        scale.voxel_at(eye.y),
        scale.voxel_at(eye.z),
    );
    json!({
        "pos": { "x": player.pos_m.x, "y": player.pos_m.y, "z": player.pos_m.z },
        "yaw": player.yaw,
        "pitch": player.pitch,
        "fly": player.fly,
        "on_ground": player.on_ground,
        "eye_in_solid": eye_in_solid,
    })
}

/// Tick the hosted world on its fixed cadence and route the resulting effects:
/// cache application + remesh dirtying + physics tile invalidation.
pub fn tick_authority(
    time: Res<Time>,
    scale: Res<CurrentScale>,
    mut authority: ResMut<Authority>,
    mut map: ResMut<ChunkMap>,
    mut dirty: ResMut<DirtyChunks>,
    mut demo: ResMut<PhysicsDemo>,
) {
    let receipts = authority.advance(f64::from(time.delta_secs()));
    if receipts.is_empty() {
        return;
    }
    let voxel_m = scale.scale.voxel_size_m();
    let changes: Vec<&BlockChange> = receipts
        .iter()
        .filter_map(|entry| match &entry.receipt.result {
            CommandResult::Ok(effects) => Some(&effects.blocks_changed),
            CommandResult::Rejected(_) => None,
        })
        .flatten()
        .collect();
    for change in &changes {
        // S6 open question, closed: an edit inside a live bubble drops the
        // covering collider tile so the next physics step rescans it.
        demo.physics_mut()
            .invalidate_voxel(change.pos.x, change.pos.y, change.pos.z, voxel_m);
    }
    dirty
        .0
        .extend(apply_block_changes(&mut map, changes.iter().copied()));
}

/// Rebuild meshes for edited chunks (and their affected neighbors).
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn remesh_dirty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterial>,
    terrain: Res<Terrain>,
    scale: Res<CurrentScale>,
    origin: Res<FloatingOrigin>,
    mut map: ResMut<ChunkMap>,
    mut dirty: ResMut<DirtyChunks>,
) {
    if dirty.0.is_empty() {
        return;
    }
    let vscale = scale.scale;
    let positions: Vec<ChunkPos> = dirty.0.drain().collect();
    for pos in positions {
        if !map.loaded.contains_key(&pos) {
            continue;
        }
        let mesh_data = {
            let loaded = &map.loaded[&pos];
            let neighbor_solid = |x: i64, y: i64, z: i64| map.is_solid(&terrain.0, vscale, x, y, z);
            mesh_chunk(
                &loaded.chunk,
                pos,
                vscale.voxel_size_m() as f32,
                &neighbor_solid,
            )
        };
        let loaded = map.loaded.get_mut(&pos).expect("checked above");
        if let Some(entity) = loaded.entity.take() {
            commands.entity(entity).despawn();
        }
        if !mesh_data.is_empty() {
            let (mx, my, mz) = pos.min_voxel();
            let min_m = glam::DVec3::new(mx as f64, my as f64, mz as f64) * vscale.voxel_size_m();
            loaded.entity = Some(
                commands
                    .spawn((
                        Mesh3d(meshes.add(to_bevy_mesh(mesh_data))),
                        MeshMaterial3d(material.0.clone()),
                        crate::app::ChunkEntity(pos),
                        // Spawn already positioned (see streaming.rs on the
                        // spawn-frame flash).
                        Transform::from_translation(to_render(min_m - origin.0)),
                    ))
                    .id(),
            );
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::app::LoadedChunk;
    use dc_api::schema::mcp_tool_name;
    use dc_api::{Vec3i, Volume, payload};
    use dc_core::{Block, Chunk};

    #[test]
    fn dirty_set_interior_border_and_corner() {
        // Interior voxel: exactly the containing chunk.
        assert_eq!(
            dirty_chunks_for_voxel(5, 5, 5),
            vec![ChunkPos::new(0, 0, 0)]
        );

        // Face-border voxel: both chunks sharing the face.
        let d = dirty_chunks_for_voxel(32, 5, 5); // lx == 0 of chunk (1,0,0)
        assert_eq!(d.len(), 2);
        assert!(d.contains(&ChunkPos::new(1, 0, 0)));
        assert!(d.contains(&ChunkPos::new(0, 0, 0)));

        let d = dirty_chunks_for_voxel(31, 5, 5); // lx == 31 of chunk (0,0,0)
        assert_eq!(d.len(), 2);
        assert!(d.contains(&ChunkPos::new(0, 0, 0)));
        assert!(d.contains(&ChunkPos::new(1, 0, 0)));

        // Negative-side border too.
        let d = dirty_chunks_for_voxel(-1, 0, 0);
        assert!(d.contains(&ChunkPos::new(-1, 0, 0)));
        assert!(d.contains(&ChunkPos::new(0, 0, 0)));

        // Corner voxel: containing chunk + three face neighbors (edge/corner
        // diagonals are not consulted by culled meshing).
        let d = dirty_chunks_for_voxel(0, 0, 0);
        assert_eq!(d.len(), 4);
        for p in [
            ChunkPos::new(0, 0, 0),
            ChunkPos::new(-1, 0, 0),
            ChunkPos::new(0, -1, 0),
            ChunkPos::new(0, 0, -1),
        ] {
            assert!(d.contains(&p), "missing {p:?}");
        }
    }

    #[test]
    fn cache_application_updates_loaded_chunks_and_skips_unloaded() {
        let mut map = ChunkMap::default();
        map.loaded.insert(
            ChunkPos::new(0, 0, 0),
            LoadedChunk {
                chunk: Chunk::new(),
                entity: None,
            },
        );
        map.loaded.insert(
            ChunkPos::new(1, 0, 0),
            LoadedChunk {
                chunk: Chunk::new(),
                entity: None,
            },
        );
        let changes = [
            // Border voxel of chunk (0,0,0): dirties both loaded chunks.
            BlockChange {
                pos: Vec3i::new(31, 4, 4),
                from: "dc:air".into(),
                to: "dc:stone".into(),
            },
            // A change in an unloaded chunk: ignored, dirties nothing.
            BlockChange {
                pos: Vec3i::new(4, 100, 4),
                from: "dc:air".into(),
                to: "dc:wood".into(),
            },
        ];
        let dirty = apply_block_changes(&mut map, changes.iter());
        assert_eq!(
            map.loaded[&ChunkPos::new(0, 0, 0)].chunk.get(31, 4, 4),
            Block::Stone
        );
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&ChunkPos::new(0, 0, 0)));
        assert!(dirty.contains(&ChunkPos::new(1, 0, 0)));
    }

    /// The scripted edit session used by the parity proofs: N set_blocks in
    /// one tick (breaks and places, including a chunk-border voxel).
    pub(crate) fn edit_script() -> Vec<Payload> {
        let mut script = Vec::new();
        for (pos, block) in [
            (Vec3i::new(2, 40, 2), "dc:stone"),
            (Vec3i::new(3, 40, 2), "dc:stone"),
            (Vec3i::new(4, 40, 2), "dc:wood"),
            (Vec3i::new(31, 40, 2), "dc:stone"), // chunk-border voxel
            (Vec3i::new(32, 40, 2), "dc:stone"), // ...and its neighbor
            (Vec3i::new(2, 41, 2), "dc:stone"),
            (Vec3i::new(2, 41, 2), "dc:air"), // edit then un-edit
            (Vec3i::new(-3, 12, -5), "dc:stone"), // near/below terrain surface
        ] {
            script.push(Payload::SetBlock(payload::SetBlock {
                pos,
                block: block.into(),
            }));
        }
        script
    }

    pub(crate) fn script_hash(authority: &mut Authority) -> u64 {
        authority
            .world
            .region_hash(Volume::new(Vec3i::new(-8, 8, -8), Vec3i::new(36, 44, 8)))
    }

    /// Item 4 of the milestone: a scripted edit session yields an identical
    /// world hash whether the envelopes are submitted directly (typed, player
    /// path) or through the registry-generated MCP tool layer (JSON path) —
    /// the S5 parity pattern extended to the client's hosted configuration
    /// (TerrainGen generator, headless, no window).
    #[test]
    fn edit_session_parity_direct_vs_mcp_tool_layer() {
        const SEED: i32 = 1337;

        // (a) Direct: typed envelopes through the player path.
        let mut direct = Authority::new(SEED, 3);
        for payload in edit_script() {
            direct.submit_player(payload);
        }
        let direct_receipts = direct.tick_now();

        // (b) MCP tool layer: the same payloads as JSON tool calls.
        let mut mcp = Authority::new(SEED, 3);
        let mut replies = Vec::new();
        for payload in edit_script() {
            let tool = mcp_tool_name(payload.command_id());
            let outer = serde_json::to_value(&payload).expect("serialize");
            let args = outer
                .as_object()
                .and_then(|m| m.values().next())
                .expect("externally tagged")
                .clone();
            let (tx, rx) = oneshot::channel();
            mcp.handle_api_call(&tool, &args, tx);
            replies.push(rx);
        }
        let mcp_receipts = mcp.tick_now();

        // Identical world state.
        assert_eq!(script_hash(&mut direct), script_hash(&mut mcp));

        // Identical receipts modulo source (S5 normalization).
        let normalize = |log: &[ReceiptEntry]| -> Vec<(String, u64, dc_api::CommandReceipt)> {
            log.iter()
                .map(|e| (e.command_id.clone(), e.consumer_seq, e.receipt.clone()))
                .collect()
        };
        assert_eq!(normalize(&direct_receipts), normalize(&mcp_receipts));

        // Every MCP reply materialized as a receipt at the tick boundary.
        for (i, mut rx) in replies.into_iter().enumerate() {
            let value = rx
                .try_recv()
                .unwrap_or_else(|_| panic!("reply {i} missing"));
            assert!(value.get("result").is_some(), "reply {i}: {value}");
        }

        // And a different seed diverges (the generator is really in the loop).
        let mut other = Authority::new(SEED + 1, 3);
        for payload in edit_script() {
            other.submit_player(payload);
        }
        other.tick_now();
        assert_ne!(script_hash(&mut direct), script_hash(&mut other));
    }

    /// Character milestone: the identical scripted movement session, driven
    /// through the character surface's own layers (attach flow + JSON tool
    /// layer under the session-attenuated token), replays to a bit-identical
    /// final pose at the client's scale over the real TerrainGen — and the
    /// session's token cannot reach any other character.
    #[test]
    fn character_session_replays_identically_and_stays_caged() {
        fn run_session(seed: i32) -> dc_api::CharacterState {
            let mut authority = Authority::new(seed, 3);
            let (tx, mut rx) = oneshot::channel();
            authority.handle_character_attach(
                "scout",
                dc_api::payload::Vec3f::new(0.3, 40.0, 0.3),
                false,
                tx,
            );
            authority.tick_now();
            let reply = rx.try_recv().expect("attach resolves at the tick boundary");
            assert_eq!(reply["ok"], json!(true), "{reply}");
            assert_eq!(reply["spawned"], json!(true));

            let call = |authority: &mut Authority, tool: &str, args: Value| {
                let (tx, _rx) = oneshot::channel();
                authority.handle_character_api(tool, &args, "scout", tx);
            };
            call(
                &mut authority,
                "character_set_move_intent",
                json!({ "character": "scout", "dx": 1.0, "dz": 0.25, "speed": 1.0 }),
            );
            for _ in 0..40 {
                authority.tick_now();
            }
            call(
                &mut authority,
                "character_set_look",
                json!({ "character": "scout", "yaw": 0.8, "pitch": -0.3 }),
            );
            call(
                &mut authority,
                "character_jump",
                json!({ "character": "scout" }),
            );
            for _ in 0..30 {
                authority.tick_now();
            }
            authority.world.character("scout").expect("exists").clone()
        }

        let a = run_session(1337);
        let b = run_session(1337);
        assert_eq!(a, b, "same seed + same JSON session = identical pose");
        assert_eq!(a.pos_m.x.to_bits(), b.pos_m.x.to_bits());
        assert_eq!(a.pos_m.y.to_bits(), b.pos_m.y.to_bits());
        assert_eq!(a.pos_m.z.to_bits(), b.pos_m.z.to_bits());
        // The generator is in the loop: another seed's terrain, another path.
        let c = run_session(1338);
        assert_ne!(a.pos_m.y.to_bits(), c.pos_m.y.to_bits());

        // Attenuation is the cage: the scout session naming another character
        // is refused by capability enforcement in the host, and attaching to
        // an existing character does not respawn it.
        let mut authority = Authority::new(1337, 3);
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "scout",
            dc_api::payload::Vec3f::new(0.3, 40.0, 0.3),
            false,
            tx,
        );
        authority.tick_now();
        assert_eq!(rx.try_recv().expect("attach")["ok"], json!(true));
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "scout",
            dc_api::payload::Vec3f::new(9.0, 9.0, 9.0),
            false,
            tx,
        );
        let reply = rx.try_recv().expect("existing attach answers immediately");
        assert_eq!(reply["ok"], json!(true));
        assert_eq!(reply["spawned"], json!(false));

        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_api(
            "character_pose",
            &json!({ "character": "other" }),
            "scout",
            tx,
        );
        let reply = rx.try_recv().expect("query answers immediately");
        assert!(
            reply["result"]["Rejected"]["MissingCapability"].is_object(),
            "foreign character denied: {reply}"
        );
    }

    /// The streamed cache is a copy of the authoritative world: a chunk
    /// fetched from the host carries both terrain and applied edits.
    #[test]
    fn host_chunk_serves_terrain_plus_edits() {
        let mut authority = Authority::new(7, 3);
        // Terrain parity with the client generator at the same scale.
        let terrain = TerrainGen::new(7);
        let vscale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 3);
        let pos = ChunkPos::new(0, 0, 0);
        let expected = terrain.generate_chunk(vscale, pos);
        assert_eq!(authority.world.chunk(pos).blocks(), expected.blocks());

        // Now edit one voxel and re-fetch: terrain + edit.
        authority.submit_player(Payload::SetBlock(payload::SetBlock {
            pos: Vec3i::new(1, 1, 1),
            block: "dc:wood".into(),
        }));
        authority.tick_now();
        assert_eq!(authority.world.chunk(pos).get(1, 1, 1), Block::Wood);
    }

    /// Walk-5 fix: attach guards placement. A body dropped into solid terrain is
    /// refused (no stuck statue, journal/0005); clear air spawns; and
    /// `surface: true` snaps the body onto the true voxel surface and lands it
    /// standing — never embedded.
    #[test]
    fn attach_rejects_embedded_allows_clear_and_snaps_to_surface() {
        let seed = 1337;
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 3);
        let terrain = TerrainGen::new(seed);
        let spawn = crate::app::find_open_spawn(&terrain, scale);
        let (sx, sz) = (spawn.x, spawn.z);
        let vs = scale.voxel_size_m();
        let half = PLAYER_WIDTH_M / 2.0;

        // Reference true surface at the spawn column.
        let solid = |x: i64, y: i64, z: i64| terrain.block_at(scale, x, y, z) != Block::Air;
        let true_surf = true_surface_m(
            &solid,
            |x, z| terrain.surface_height_m(x, z),
            scale,
            sx,
            sz,
            half,
        );

        // Precondition: feet a few meters below the surface really are embedded
        // (self-check so a cave at this column would surface as a failure).
        let buried_feet = true_surf - 3.0;
        let buried_aabb = Aabb::from_bottom_center(
            glam::DVec3::new(sx, buried_feet, sz) / vs,
            half / vs,
            PLAYER_HEIGHT_M / vs,
        );
        assert!(
            aabb_overlaps_solid(&solid, buried_aabb),
            "test precondition: {buried_feet} m should be inside the ground"
        );

        // 1) Embedded spawn is refused with a machine-readable receipt, and no
        //    character is created.
        let mut authority = Authority::new(seed, 3);
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "buried",
            dc_api::payload::Vec3f::new(sx, buried_feet, sz),
            false,
            tx,
        );
        let reply = rx.try_recv().expect("embed guard answers immediately");
        assert_eq!(reply["ok"], json!(false), "{reply}");
        assert_eq!(reply["code"], json!("obstructed"), "{reply}");
        authority.tick_now();
        assert!(
            authority.world.character("buried").is_none(),
            "an obstructed attach must spawn no statue"
        );

        // 2) Clear air spawns fine.
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "flyer",
            dc_api::payload::Vec3f::new(sx, true_surf + 50.0, sz),
            false,
            tx,
        );
        authority.tick_now();
        let reply = rx.try_recv().expect("clear attach resolves");
        assert_eq!(reply["ok"], json!(true), "{reply}");
        assert_eq!(reply["spawned"], json!(true));
        assert!(authority.world.character("flyer").is_some());

        // 3) surface:true snaps to the true surface regardless of the requested
        //    y, and the landed body is not embedded.
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "walker",
            dc_api::payload::Vec3f::new(sx, true_surf + 100.0, sz),
            true,
            tx,
        );
        authority.tick_now();
        let reply = rx.try_recv().expect("surface attach resolves");
        assert_eq!(reply["ok"], json!(true), "{reply}");
        let walker = authority
            .world
            .character("walker")
            .expect("spawned")
            .clone();
        assert!(
            (walker.pos_m.y - (true_surf + 0.05)).abs() < vs,
            "surface-snapped feet {} near the true surface {true_surf}",
            walker.pos_m.y
        );
        let landed = Aabb::from_bottom_center(
            glam::DVec3::new(walker.pos_m.x, walker.pos_m.y, walker.pos_m.z) / vs,
            half / vs,
            PLAYER_HEIGHT_M / vs,
        );
        assert!(
            !aabb_overlaps_solid(&solid, landed),
            "surface-snapped body must not be embedded"
        );
    }
}
