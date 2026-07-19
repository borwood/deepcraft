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

use std::collections::HashSet;

use bevy::prelude::*;
use dc_api::host::block_from_name;
use dc_api::schema::CommandKind;
use dc_api::{
    BlockChange, CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant,
    HostWorld, Payload, ReceiptEntry,
};
use dc_core::{CHUNK_SIZE_USIZE, ChunkPos, VoxelScale, local_voxel};
use dc_mcp_dev::envelope_for_tool_call;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::PLAYER_HEIGHT_M;
use crate::app::{ChunkMap, ChunkMaterial, CurrentScale, FloatingOrigin, Terrain, to_render};
use crate::mcp::{BridgeRequest, McpBridge};
use crate::meshing::mesh_chunk;
use crate::physdemo::PhysicsDemo;
use crate::player::Player;
use crate::streaming::to_bevy_mesh;
use crate::worldgen::TerrainGen;

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
    /// the MCP consumer's per-consumer seq.
    pending: Vec<(u64, oneshot::Sender<Value>)>,
    mcp_consumer: ConsumerId,
    mcp_token: CapabilityToken,
    player_consumer: ConsumerId,
    player_token: CapabilityToken,
}

impl Authority {
    /// Build the hosted world over the client's terrain generator at the given
    /// scale. The generator closure owns its own `TerrainGen` (same seed =>
    /// identical noise), keeping the closure `Send + Sync` and deterministic.
    pub fn new(seed: i32, player_voxels: u32) -> Self {
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, player_voxels);
        let terrain = TerrainGen::new(seed);
        let world = HostWorld::with_generator(
            seed as u64,
            Box::new(move |pos| terrain.generate_chunk(scale, pos)),
        );
        Self {
            world,
            accumulator: 0.0,
            pending: Vec::new(),
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

    /// Handle one MCP tool call against the dc-api surface (the registry-
    /// generated tools). Queries answer immediately from the last completed
    /// tick; commands are submitted and their reply is delivered when the
    /// receipt materializes at the tick boundary.
    pub fn handle_api_call(&mut self, tool: &str, args: &Value, reply: oneshot::Sender<Value>) {
        match envelope_for_tool_call(tool, args, &self.mcp_consumer, &self.mcp_token) {
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
                    Ok(ack) => self.pending.push((ack.consumer_seq, reply)),
                },
            },
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
            for (seq, reply) in self.pending.drain(..) {
                match receipts
                    .iter()
                    .find(|e| e.source == self.mcp_consumer && e.consumer_seq == seq)
                {
                    Some(entry) => {
                        let _ = reply.send(
                            serde_json::to_value(&entry.receipt)
                                .unwrap_or_else(|e| json!({"error": e.to_string()})),
                        );
                    }
                    None => still.push((seq, reply)),
                }
            }
            self.pending = still;
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
                    // Walker-safe teleport: feet snap to the terrain surface
                    // at (x, z) regardless of the requested y. (Terrain only —
                    // edits are rare enough that a buried result still shows
                    // in eye_in_solid.)
                    player.pos_m.y =
                        terrain.0.surface_height_m(player.pos_m.x, player.pos_m.z) + 0.05;
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
}
