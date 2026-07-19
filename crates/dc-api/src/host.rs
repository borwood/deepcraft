//! The reference host world backing the API for S5.
//!
//! Placement decision (S5 scope note): this lives IN dc-api rather than a
//! separate dc-host crate because it is the *reference implementation* of the
//! command surface's semantics (tick quantization, total order, txn
//! atomicity, capability enforcement) — the thing every consumer test runs
//! against. The wasmtime plugin host (crates/dc-host) and the MCP server
//! (crates/dc-mcp-dev) are thin boundary layers over this. When dc-sim grows
//! a real world, it implements the same semantics and this stays as the
//! conformance reference.
//!
//! World model: dc-core 32^3 chunks for blocks (lazily generated from a seed:
//! a low hill-field with grass tops between y=-3 and y=-1, air above), a Vec
//! of simple entities, a manual [`HostWorld::tick`] driver. No rendering.

use std::cell::RefCell;
use std::collections::HashMap;

use dc_core::{Block, Chunk, ChunkPos, local_voxel, raycast_voxels};
use glam::DVec3;

use crate::capability::requirement_for;
use crate::character::{CharacterConfig, CharacterState, step_character, valid_character_name};
use crate::classes::{ClassMemberDef, ContentClassDef, validate_class_contract, validate_params};
use crate::envelope::{
    BlockChange, CommandEnvelope, CommandReceipt, CommandResult, ConsumerId, Effects, QueryReceipt,
    QueryResult, ReceiptEntry, RejectReason, SubmitAck, Tick,
};
use crate::event::GameEvent;
use crate::payload::{EntityInfo, Payload, QueryData, Vec3i, Volume};
use crate::schema::{CommandKind, spec};

/// Per-command cap on fill/scan volume (voxels). Keeps one envelope from
/// stalling a tick; larger jobs batch across envelopes/ticks.
pub const MAX_REGION_VOXELS: u64 = 1 << 18; // 262,144 = a 64^3 box

/// Default max events returned by one poll.
pub const DEFAULT_POLL_MAX: u32 = 256;

/// v0 block name table (hard-coded S1 block set; the data-driven block
/// registry is a later slice — items are data-driven already).
pub fn block_from_name(name: &str) -> Option<Block> {
    Some(match name {
        "dc:air" => Block::Air,
        "dc:stone" => Block::Stone,
        "dc:dirt" => Block::Dirt,
        "dc:grass" => Block::Grass,
        "dc:wood" => Block::Wood,
        "dc:mudstone" => Block::Mudstone,
        "dc:sandstone" => Block::Sandstone,
        "dc:granite" => Block::Granite,
        "dc:basalt" => Block::Basalt,
        _ => return None,
    })
}

pub fn block_name(block: Block) -> &'static str {
    match block {
        Block::Air => "dc:air",
        Block::Stone => "dc:stone",
        Block::Dirt => "dc:dirt",
        Block::Grass => "dc:grass",
        Block::Wood => "dc:wood",
        Block::Mudstone => "dc:mudstone",
        Block::Sandstone => "dc:sandstone",
        Block::Granite => "dc:granite",
        Block::Basalt => "dc:basalt",
    }
}

/// A data-driven item definition, as stored.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub defined_tick: Tick,
    pub defined_by: ConsumerId,
}

#[derive(Clone, Debug)]
struct Subscription {
    owner: ConsumerId,
    kinds: Vec<crate::event::EventKind>,
    volume: Option<Volume>,
    queue: Vec<GameEvent>,
}

impl Subscription {
    fn matches(&self, ev: &GameEvent) -> bool {
        if !self.kinds.is_empty() && !self.kinds.contains(&ev.kind()) {
            return false;
        }
        match (self.volume, ev.pos_hint()) {
            (None, _) => true,
            (Some(vol), Some(pos)) => vol.contains(pos),
            (Some(_), None) => false,
        }
    }
}

#[derive(Clone, Debug)]
struct QueuedCmd {
    scheduled: Tick,
    priority: u8,
    consumer_seq: u64,
    env: CommandEnvelope,
}

/// Undo journal entry for txn rollback.
enum Undo {
    SetBlock { pos: Vec3i, prev: Block },
    PopEntity,
    RemoveItem(String),
    RemoveSub(u64),
    RemoveCharacter(String),
    RestoreCharacter(Box<CharacterState>),
    RemoveContentClass(String),
    RemoveClassMember(String),
}

/// Max range of a character sense raycast, meters.
pub const MAX_SENSE_RAYCAST_M: f64 = 50.0;
/// Max half-extent of a `sense_surroundings` scan, voxels.
pub const MAX_SENSE_RADIUS_VOXELS: u32 = 16;

/// Pluggable base-terrain generator: the authoritative contents of a chunk
/// that has never been edited. Must be a pure function of `pos` (all entropy
/// baked in at construction) or replay determinism breaks.
///
/// Added by the client-through-dc-api milestone (additive): the client embeds
/// a `HostWorld` as the edit authority and needs it to serve the *same*
/// terrain the client streams (the S1 `TerrainGen`), not the built-in
/// hill-field. The built-in generator remains the default for `new` so every
/// existing consumer/test is unchanged.
pub type ChunkGenerator = Box<dyn Fn(ChunkPos) -> Chunk + Send + Sync>;

/// The in-process world. Single-threaded, manual tick driver.
pub struct HostWorld {
    seed: u64,
    /// Base terrain for never-edited chunks. `None` = the built-in seeded
    /// hill-field (S5 behavior, byte-for-byte).
    generator: Option<ChunkGenerator>,
    /// Last completed tick. State always reflects exactly this tick.
    tick: Tick,
    chunks: HashMap<ChunkPos, Chunk>,
    entities: Vec<EntityInfo>,
    /// Persistent named characters, stepped every tick (BTreeMap: the step
    /// order is part of the deterministic replay identity).
    characters: std::collections::BTreeMap<String, CharacterState>,
    character_config: CharacterConfig,
    next_entity_id: u64,
    items: std::collections::BTreeMap<String, ItemDef>,
    /// Content classes (contracts) and their members — the second registry
    /// (BTreeMaps: iteration order is part of the deterministic surface).
    content_classes: std::collections::BTreeMap<String, ContentClassDef>,
    class_members: std::collections::BTreeMap<String, ClassMemberDef>,
    subs: std::collections::BTreeMap<u64, Subscription>,
    next_sub_id: u64,
    queue: Vec<QueuedCmd>,
    consumer_seqs: HashMap<ConsumerId, u64>,
    apply_seq: u64,
    log: Vec<ReceiptEntry>,
}

impl HostWorld {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generator: None,
            tick: 0,
            chunks: HashMap::new(),
            entities: Vec::new(),
            characters: std::collections::BTreeMap::new(),
            character_config: CharacterConfig::default(),
            next_entity_id: 1,
            items: std::collections::BTreeMap::new(),
            content_classes: std::collections::BTreeMap::new(),
            class_members: std::collections::BTreeMap::new(),
            subs: std::collections::BTreeMap::new(),
            next_sub_id: 1,
            queue: Vec::new(),
            consumer_seqs: HashMap::new(),
            apply_seq: 0,
            log: Vec::new(),
        }
    }

    /// A world whose base terrain comes from `generator` instead of the
    /// built-in hill-field. The seed still salts everything non-terrain and is
    /// part of the replay identity; the generator must be deterministic.
    pub fn with_generator(seed: u64, generator: ChunkGenerator) -> Self {
        let mut world = Self::new(seed);
        world.generator = Some(generator);
        world
    }

    /// Last completed tick.
    pub fn current_tick(&self) -> Tick {
        self.tick
    }

    /// Full receipt log, in apply order (submit-time rejections included).
    pub fn receipt_log(&self) -> &[ReceiptEntry] {
        &self.log
    }

    pub fn item(&self, name: &str) -> Option<&ItemDef> {
        self.items.get(name)
    }

    pub fn items(&self) -> impl Iterator<Item = &ItemDef> {
        self.items.values()
    }

    pub fn content_class(&self, name: &str) -> Option<&ContentClassDef> {
        self.content_classes.get(name)
    }

    /// All content classes, in name order.
    pub fn content_classes(&self) -> impl Iterator<Item = &ContentClassDef> {
        self.content_classes.values()
    }

    pub fn class_member(&self, name: &str) -> Option<&ClassMemberDef> {
        self.class_members.get(name)
    }

    /// All class members, in name order.
    pub fn class_members(&self) -> impl Iterator<Item = &ClassMemberDef> {
        self.class_members.values()
    }

    pub fn entities(&self) -> &[EntityInfo] {
        &self.entities
    }

    /// Configure character body dimensions / dynamics (the client sets
    /// `voxel_size_m` from its active scale). Part of the replay identity:
    /// replays must use the same config.
    pub fn set_character_config(&mut self, config: CharacterConfig) {
        self.character_config = config;
    }

    pub fn character_config(&self) -> &CharacterConfig {
        &self.character_config
    }

    pub fn character(&self, name: &str) -> Option<&CharacterState> {
        self.characters.get(name)
    }

    /// All characters, in deterministic (name) order.
    pub fn characters(&self) -> impl Iterator<Item = &CharacterState> {
        self.characters.values()
    }

    // ------------------------------------------------------------ terrain --

    fn mix64(mut x: u64) -> u64 {
        // splitmix64 finalizer.
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }

    /// Deterministic base-terrain surface height for a column: in {-3,-2,-1},
    /// constant over 4x4-column cells. Grass at the surface, stone below.
    fn floor_height(&self, x: i64, z: i64) -> i64 {
        let cx = (x >> 2) as u64;
        let cz = (z >> 2) as u64;
        let n = Self::mix64(self.seed ^ Self::mix64(cx ^ cz.rotate_left(32)));
        -3 + (n % 3) as i64
    }

    fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        if let Some(generator) = &self.generator {
            return generator(pos);
        }
        let mut chunk = Chunk::new();
        let (mx, my, mz) = pos.min_voxel();
        for lz in 0..dc_core::CHUNK_SIZE_USIZE {
            for lx in 0..dc_core::CHUNK_SIZE_USIZE {
                let h = self.floor_height(mx + lx as i64, mz + lz as i64);
                for ly in 0..dc_core::CHUNK_SIZE_USIZE {
                    let wy = my + ly as i64;
                    let block = match wy.cmp(&h) {
                        std::cmp::Ordering::Less => Block::Stone,
                        std::cmp::Ordering::Equal => Block::Grass,
                        std::cmp::Ordering::Greater => Block::Air,
                    };
                    if block != Block::Air {
                        chunk.set(lx, ly, lz, block);
                    }
                }
            }
        }
        chunk
    }

    fn chunk_at(&mut self, pos: ChunkPos) -> &mut Chunk {
        if !self.chunks.contains_key(&pos) {
            let c = self.generate_chunk(pos);
            self.chunks.insert(pos, c);
        }
        self.chunks.get_mut(&pos).expect("just inserted")
    }

    /// Authoritative contents of a chunk (lazily generated; includes every
    /// applied edit). Render/collision caches clone from here — added by the
    /// client-through-dc-api milestone (additive) so the client's `ChunkMap`
    /// can be a cache of the hosted world instead of a second authority.
    pub fn chunk(&mut self, pos: ChunkPos) -> &Chunk {
        self.chunk_at(pos)
    }

    /// Read a voxel (lazily generating its chunk).
    pub fn block_at(&mut self, p: Vec3i) -> Block {
        let cpos = ChunkPos::from_world_voxel(p.x, p.y, p.z);
        let (lx, ly, lz) = local_voxel(p.x, p.y, p.z);
        self.chunk_at(cpos).get(lx, ly, lz)
    }

    fn set_block_raw(&mut self, p: Vec3i, block: Block) -> Block {
        let cpos = ChunkPos::from_world_voxel(p.x, p.y, p.z);
        let (lx, ly, lz) = local_voxel(p.x, p.y, p.z);
        let chunk = self.chunk_at(cpos);
        let prev = chunk.get(lx, ly, lz);
        chunk.set(lx, ly, lz, block);
        prev
    }

    // ------------------------------------------------------------- submit --

    fn next_consumer_seq(&mut self, source: &ConsumerId) -> u64 {
        let ctr = self.consumer_seqs.entry(source.clone()).or_insert(0);
        let seq = *ctr;
        *ctr += 1;
        seq
    }

    fn reject_now(
        &mut self,
        env: &CommandEnvelope,
        consumer_seq: u64,
        reason: RejectReason,
    ) -> ReceiptEntry {
        self.apply_seq += 1;
        let entry = ReceiptEntry {
            source: env.source.clone(),
            command_id: env.id.clone(),
            consumer_seq,
            receipt: CommandReceipt {
                seq: self.apply_seq,
                tick_applied: self.tick,
                result: CommandResult::Rejected(reason),
            },
        };
        self.log.push(entry.clone());
        entry
    }

    /// Queue a command for the next (or a future) tick boundary. Structural
    /// problems (unknown id, id/payload mismatch, query-as-command, past
    /// target tick) reject immediately with a logged receipt; everything else
    /// — including capability enforcement — is judged at apply time.
    pub fn submit(&mut self, env: CommandEnvelope) -> Result<SubmitAck, Box<ReceiptEntry>> {
        let consumer_seq = self.next_consumer_seq(&env.source);
        let Some(spec) = spec(&env.id) else {
            let reason = RejectReason::UnknownCommand { id: env.id.clone() };
            return Err(Box::new(self.reject_now(&env, consumer_seq, reason)));
        };
        if env.payload.command_id() != env.id {
            let reason = RejectReason::PayloadMismatch {
                id: env.id.clone(),
                payload_id: env.payload.command_id().to_string(),
            };
            return Err(Box::new(self.reject_now(&env, consumer_seq, reason)));
        }
        if spec.kind == CommandKind::Query {
            let reason = RejectReason::NotACommand { id: env.id.clone() };
            return Err(Box::new(self.reject_now(&env, consumer_seq, reason)));
        }
        let scheduled = env.target_tick.unwrap_or(self.tick + 1);
        if scheduled <= self.tick {
            let reason = RejectReason::TargetTickInPast {
                target: scheduled,
                current: self.tick,
            };
            return Err(Box::new(self.reject_now(&env, consumer_seq, reason)));
        }
        self.queue.push(QueuedCmd {
            scheduled,
            priority: env.source.kind.priority_class(),
            consumer_seq,
            env,
        });
        Ok(SubmitAck {
            consumer_seq,
            scheduled_tick: scheduled,
        })
    }

    // --------------------------------------------------------------- tick --

    /// Complete the next tick: apply everything due, in the documented total
    /// order `(tick, priority class, consumer id, per-consumer seq)`.
    /// Returns the receipts produced this tick (also appended to the log).
    pub fn tick(&mut self) -> Vec<ReceiptEntry> {
        let t = self.tick + 1;
        let mut due: Vec<QueuedCmd> = Vec::new();
        let mut rest: Vec<QueuedCmd> = Vec::new();
        for q in self.queue.drain(..) {
            if q.scheduled <= t {
                due.push(q);
            } else {
                rest.push(q);
            }
        }
        self.queue = rest;
        due.sort_by(|a, b| {
            (a.priority, &a.env.source, a.consumer_seq).cmp(&(
                b.priority,
                &b.env.source,
                b.consumer_seq,
            ))
        });

        let mut produced = Vec::new();
        let mut i = 0;
        while i < due.len() {
            // Gather this command's txn group (same source + txn id).
            let group_end = match due[i].env.txn {
                None => i + 1,
                Some(txn) => {
                    // Members sort adjacently only if their seqs are adjacent,
                    // so collect by scan; order within the group is the sorted
                    // (i.e. per-consumer seq) order because sort is stable per
                    // consumer.
                    let src = due[i].env.source.clone();
                    let mut members = vec![due.remove(i)];
                    let mut j = i;
                    while j < due.len() {
                        if due[j].env.txn == Some(txn) && due[j].env.source == src {
                            members.push(due.remove(j));
                        } else {
                            j += 1;
                        }
                    }
                    let n = members.len();
                    for (k, m) in members.into_iter().enumerate() {
                        due.insert(i + k, m);
                    }
                    i + n
                }
            };
            let group: Vec<QueuedCmd> = due[i..group_end].to_vec();
            produced.extend(self.apply_group(t, group));
            i = group_end;
        }
        // Character bodies integrate after this tick's commands have applied,
        // so a controller verb takes effect on the tick it lands on.
        self.step_characters();
        self.tick = t;
        produced
    }

    /// Step every character one tick against the voxel world, in name order.
    /// The solidity query lazily generates chunks like any other consumer
    /// (hence the RefCell: `move_aabb` wants a `Fn`, chunk generation wants
    /// `&mut self`; single-threaded and non-reentrant, so this is safe).
    fn step_characters(&mut self) {
        if self.characters.is_empty() {
            return;
        }
        let cfg = self.character_config;
        let mut characters = std::mem::take(&mut self.characters);
        {
            let world = RefCell::new(&mut *self);
            let solid = |x: i64, y: i64, z: i64| {
                world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid()
            };
            for character in characters.values_mut() {
                step_character(character, &cfg, &solid);
            }
        }
        self.characters = characters;
    }

    /// Apply an all-or-nothing group (a lone command is a group of one).
    fn apply_group(&mut self, t: Tick, group: Vec<QueuedCmd>) -> Vec<ReceiptEntry> {
        let mut journal: Vec<Undo> = Vec::new();
        let mut events: Vec<GameEvent> = Vec::new();
        let mut applied: Vec<(QueuedCmd, Effects)> = Vec::new();
        let mut failure: Option<(QueuedCmd, RejectReason)> = None;
        let mut iter = group.into_iter();
        for q in iter.by_ref() {
            match self.apply_one(t, &q.env, &mut journal, &mut events) {
                Ok(effects) => applied.push((q, effects)),
                Err(reason) => {
                    failure = Some((q, reason));
                    break;
                }
            }
        }
        let mut entries = Vec::new();
        match failure {
            None => {
                // Commit: deliver events, emit Ok receipts in order.
                self.deliver(events);
                for (q, effects) in applied {
                    self.apply_seq += 1;
                    entries.push(ReceiptEntry {
                        source: q.env.source.clone(),
                        command_id: q.env.id.clone(),
                        consumer_seq: q.consumer_seq,
                        receipt: CommandReceipt {
                            seq: self.apply_seq,
                            tick_applied: t,
                            result: CommandResult::Ok(effects),
                        },
                    });
                }
            }
            Some((failed, reason)) => {
                // Roll back everything this group did, newest first.
                for undo in journal.into_iter().rev() {
                    match undo {
                        Undo::SetBlock { pos, prev } => {
                            self.set_block_raw(pos, prev);
                        }
                        Undo::PopEntity => {
                            self.entities.pop();
                            self.next_entity_id -= 1;
                        }
                        Undo::RemoveItem(name) => {
                            self.items.remove(&name);
                        }
                        Undo::RemoveSub(id) => {
                            self.subs.remove(&id);
                            self.next_sub_id -= 1;
                        }
                        Undo::RemoveCharacter(name) => {
                            self.characters.remove(&name);
                        }
                        Undo::RestoreCharacter(prev) => {
                            self.characters.insert(prev.name.clone(), *prev);
                        }
                        Undo::RemoveContentClass(name) => {
                            self.content_classes.remove(&name);
                        }
                        Undo::RemoveClassMember(name) => {
                            self.class_members.remove(&name);
                        }
                    }
                }
                let failed_id = failed.env.id.clone();
                let aborted = |r: RejectReason| RejectReason::TxnAborted {
                    failed_id: failed_id.clone(),
                    reason: Box::new(r),
                };
                for (q, _) in applied {
                    self.apply_seq += 1;
                    entries.push(ReceiptEntry {
                        source: q.env.source.clone(),
                        command_id: q.env.id.clone(),
                        consumer_seq: q.consumer_seq,
                        receipt: CommandReceipt {
                            seq: self.apply_seq,
                            tick_applied: t,
                            result: CommandResult::Rejected(aborted(
                                RejectReason::PayloadInvalid {
                                    reason: "aborted by transaction sibling".into(),
                                },
                            )),
                        },
                    });
                }
                // The failing command reports its own reason (wrapped when it
                // took siblings down with it, bare when it was alone).
                self.apply_seq += 1;
                let result = if failed.env.txn.is_some() {
                    CommandResult::Rejected(aborted(reason))
                } else {
                    CommandResult::Rejected(reason)
                };
                entries.push(ReceiptEntry {
                    source: failed.env.source.clone(),
                    command_id: failed.env.id.clone(),
                    consumer_seq: failed.consumer_seq,
                    receipt: CommandReceipt {
                        seq: self.apply_seq,
                        tick_applied: t,
                        result,
                    },
                });
                // Unapplied txn siblings after the failure: aborted too.
                for q in iter {
                    self.apply_seq += 1;
                    entries.push(ReceiptEntry {
                        source: q.env.source.clone(),
                        command_id: q.env.id.clone(),
                        consumer_seq: q.consumer_seq,
                        receipt: CommandReceipt {
                            seq: self.apply_seq,
                            tick_applied: t,
                            result: CommandResult::Rejected(aborted(
                                RejectReason::PayloadInvalid {
                                    reason: "aborted by transaction sibling".into(),
                                },
                            )),
                        },
                    });
                }
            }
        }
        self.log.extend(entries.iter().cloned());
        entries
    }

    /// Apply one command's mutation, journaling undos and buffering events.
    fn apply_one(
        &mut self,
        t: Tick,
        env: &CommandEnvelope,
        journal: &mut Vec<Undo>,
        events: &mut Vec<GameEvent>,
    ) -> Result<Effects, RejectReason> {
        // Capability enforcement — deny by default, single choke point.
        let req = requirement_for(&env.payload)
            .map_err(|reason| RejectReason::PayloadInvalid { reason })?;
        if !env.grant.covers(&req) {
            return Err(RejectReason::MissingCapability {
                needed: req.to_string(),
            });
        }
        let mut effects = Effects::default();
        match &env.payload {
            Payload::SetBlock(p) => {
                let block =
                    block_from_name(&p.block).ok_or_else(|| RejectReason::UnknownBlock {
                        name: p.block.clone(),
                    })?;
                let prev = self.set_block_raw(p.pos, block);
                if prev != block {
                    journal.push(Undo::SetBlock { pos: p.pos, prev });
                    effects.blocks_changed.push(BlockChange {
                        pos: p.pos,
                        from: block_name(prev).into(),
                        to: block_name(block).into(),
                    });
                    events.push(GameEvent::BlockChanged {
                        pos: p.pos,
                        from: block_name(prev).into(),
                        to: block_name(block).into(),
                        cause: env.source.clone(),
                        tick: t,
                    });
                }
            }
            Payload::Fill(p) => {
                let vol = Volume::new(p.min, p.max);
                if vol.voxel_count() > MAX_REGION_VOXELS {
                    return Err(RejectReason::RegionTooLarge {
                        voxels: vol.voxel_count(),
                        max: MAX_REGION_VOXELS,
                    });
                }
                let block =
                    block_from_name(&p.block).ok_or_else(|| RejectReason::UnknownBlock {
                        name: p.block.clone(),
                    })?;
                for y in vol.min.y..=vol.max.y {
                    for z in vol.min.z..=vol.max.z {
                        for x in vol.min.x..=vol.max.x {
                            let pos = Vec3i::new(x, y, z);
                            let prev = self.set_block_raw(pos, block);
                            if prev != block {
                                journal.push(Undo::SetBlock { pos, prev });
                                effects.blocks_changed.push(BlockChange {
                                    pos,
                                    from: block_name(prev).into(),
                                    to: block_name(block).into(),
                                });
                                events.push(GameEvent::BlockChanged {
                                    pos,
                                    from: block_name(prev).into(),
                                    to: block_name(block).into(),
                                    cause: env.source.clone(),
                                    tick: t,
                                });
                            }
                        }
                    }
                }
            }
            Payload::EntitySpawn(p) => {
                let id = self.next_entity_id;
                self.next_entity_id += 1;
                self.entities.push(EntityInfo {
                    id,
                    kind: p.kind.clone(),
                    pos: p.pos,
                });
                journal.push(Undo::PopEntity);
                effects.entities_spawned.push(id);
                events.push(GameEvent::EntitySpawned {
                    id,
                    kind: p.kind.clone(),
                    pos: p.pos,
                    cause: env.source.clone(),
                    tick: t,
                });
            }
            Payload::DefineItem(p) => {
                if self.items.contains_key(&p.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: p.name.clone(),
                    });
                }
                self.items.insert(
                    p.name.clone(),
                    ItemDef {
                        name: p.name.clone(),
                        display_name: p.display_name.clone(),
                        description: p.description.clone(),
                        defined_tick: t,
                        defined_by: env.source.clone(),
                    },
                );
                journal.push(Undo::RemoveItem(p.name.clone()));
                effects.items_defined.push(p.name.clone());
                events.push(GameEvent::ItemDefined {
                    name: p.name.clone(),
                    cause: env.source.clone(),
                    tick: t,
                });
            }
            Payload::DefineContentClass(p) => {
                if self.content_classes.contains_key(&p.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: p.name.clone(),
                    });
                }
                validate_class_contract(&p.params)
                    .map_err(|reason| RejectReason::SchemaViolation { reason })?;
                self.content_classes.insert(
                    p.name.clone(),
                    ContentClassDef {
                        name: p.name.clone(),
                        doc: p.doc.clone(),
                        params: p.params.clone(),
                        defined_tick: t,
                        defined_by: env.source.clone(),
                    },
                );
                journal.push(Undo::RemoveContentClass(p.name.clone()));
                effects.classes_defined.push(p.name.clone());
            }
            Payload::DefineClassMember(p) => {
                if self.class_members.contains_key(&p.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: p.name.clone(),
                    });
                }
                let class = self.content_classes.get(&p.class).ok_or_else(|| {
                    RejectReason::UnknownClass {
                        class: p.class.clone(),
                    }
                })?;
                validate_params(&class.params, &p.params)
                    .map_err(|reason| RejectReason::SchemaViolation { reason })?;
                self.class_members.insert(
                    p.name.clone(),
                    ClassMemberDef {
                        name: p.name.clone(),
                        class: p.class.clone(),
                        params: p.params.clone(),
                        defined_tick: t,
                        defined_by: env.source.clone(),
                    },
                );
                journal.push(Undo::RemoveClassMember(p.name.clone()));
                effects.class_members_defined.push(p.name.clone());
            }
            Payload::EventsSubscribe(p) => {
                let id = self.next_sub_id;
                self.next_sub_id += 1;
                self.subs.insert(
                    id,
                    Subscription {
                        owner: env.source.clone(),
                        kinds: p.kinds.clone(),
                        volume: p.volume,
                        queue: Vec::new(),
                    },
                );
                journal.push(Undo::RemoveSub(id));
                effects.subscriptions_created.push(id);
            }
            Payload::SpawnCharacter(p) => {
                if !valid_character_name(&p.name) {
                    return Err(RejectReason::PayloadInvalid {
                        reason: format!(
                            "character name `{}` is not a bare slug ([a-z0-9_-], max 64)",
                            p.name
                        ),
                    });
                }
                if ![p.pos.x, p.pos.y, p.pos.z].iter().all(|v| v.is_finite()) {
                    return Err(RejectReason::PayloadInvalid {
                        reason: "character position must be finite".into(),
                    });
                }
                if self.characters.contains_key(&p.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: p.name.clone(),
                    });
                }
                self.characters
                    .insert(p.name.clone(), CharacterState::new(p.name.clone(), p.pos));
                journal.push(Undo::RemoveCharacter(p.name.clone()));
                effects.characters_spawned.push(p.name.clone());
            }
            Payload::SetMoveIntent(p) => {
                if ![p.dx, p.dz, p.speed].iter().all(|v| v.is_finite()) {
                    return Err(RejectReason::PayloadInvalid {
                        reason: "move intent must be finite".into(),
                    });
                }
                let character = self.characters.get_mut(&p.character).ok_or_else(|| {
                    RejectReason::UnknownCharacter {
                        name: p.character.clone(),
                    }
                })?;
                journal.push(Undo::RestoreCharacter(Box::new(character.clone())));
                character.input.move_dir = (p.dx, p.dz);
                character.input.speed = p.speed.clamp(0.0, 1.0);
            }
            Payload::SetLook(p) => {
                if !p.yaw.is_finite() || !p.pitch.is_finite() {
                    return Err(RejectReason::PayloadInvalid {
                        reason: "look angles must be finite".into(),
                    });
                }
                let character = self.characters.get_mut(&p.character).ok_or_else(|| {
                    RejectReason::UnknownCharacter {
                        name: p.character.clone(),
                    }
                })?;
                journal.push(Undo::RestoreCharacter(Box::new(character.clone())));
                character.yaw = p.yaw;
                character.pitch = p.pitch.clamp(-1.55, 1.55);
            }
            Payload::Jump(p) => {
                let character = self.characters.get_mut(&p.character).ok_or_else(|| {
                    RejectReason::UnknownCharacter {
                        name: p.character.clone(),
                    }
                })?;
                journal.push(Undo::RestoreCharacter(Box::new(character.clone())));
                character.input.jump = true;
            }
            // Queries never reach apply (submit rejects them).
            Payload::GetBlock(_)
            | Payload::ScanRegion(_)
            | Payload::EntityQuery(_)
            | Payload::EventsPoll(_)
            | Payload::CharacterPose(_)
            | Payload::SenseRaycast(_)
            | Payload::SenseSurroundings(_) => {
                return Err(RejectReason::NotACommand { id: env.id.clone() });
            }
        }
        Ok(effects)
    }

    /// Deliver committed events into matching subscription queues.
    fn deliver(&mut self, events: Vec<GameEvent>) {
        for ev in events {
            for sub in self.subs.values_mut() {
                if sub.matches(&ev) {
                    sub.queue.push(ev.clone());
                }
            }
        }
    }

    // -------------------------------------------------------------- query --

    /// Run a query against the last completed tick's state. Queries never
    /// mutate world state; `events/poll` drains only the caller's own
    /// subscription channel. (`&mut self` is lazy chunk generation + queue
    /// draining, not simulation.)
    pub fn query(&mut self, env: &CommandEnvelope) -> QueryReceipt {
        let reject = |tick: Tick, reason: RejectReason| QueryReceipt {
            tick_observed: tick,
            result: QueryResult::Rejected(reason),
        };
        let tick = self.tick;
        let Some(spec) = spec(&env.id) else {
            return reject(tick, RejectReason::UnknownCommand { id: env.id.clone() });
        };
        if env.payload.command_id() != env.id {
            return reject(
                tick,
                RejectReason::PayloadMismatch {
                    id: env.id.clone(),
                    payload_id: env.payload.command_id().to_string(),
                },
            );
        }
        if spec.kind == CommandKind::Command {
            return reject(tick, RejectReason::NotAQuery { id: env.id.clone() });
        }
        let req = match requirement_for(&env.payload) {
            Ok(r) => r,
            Err(reason) => return reject(tick, RejectReason::PayloadInvalid { reason }),
        };
        if !env.grant.covers(&req) {
            return reject(
                tick,
                RejectReason::MissingCapability {
                    needed: req.to_string(),
                },
            );
        }
        let data = match &env.payload {
            Payload::GetBlock(p) => QueryData::Block {
                block: block_name(self.block_at(p.pos)).into(),
            },
            Payload::ScanRegion(p) => {
                let vol = Volume::new(p.min, p.max);
                if vol.voxel_count() > MAX_REGION_VOXELS {
                    return reject(
                        tick,
                        RejectReason::RegionTooLarge {
                            voxels: vol.voxel_count(),
                            max: MAX_REGION_VOXELS,
                        },
                    );
                }
                self.scan_data(vol)
            }
            Payload::EntityQuery(p) => {
                let entities = self
                    .entities
                    .iter()
                    .filter(|e| {
                        p.volume.is_none_or(|v| v.contains(e.pos.floor_voxel()))
                            && p.kind.as_ref().is_none_or(|k| *k == e.kind)
                    })
                    .cloned()
                    .collect();
                QueryData::Entities { entities }
            }
            Payload::EventsPoll(p) => {
                let max = p.max.unwrap_or(DEFAULT_POLL_MAX) as usize;
                let Some(sub) = self.subs.get_mut(&p.subscription) else {
                    return reject(
                        tick,
                        RejectReason::UnknownSubscription { id: p.subscription },
                    );
                };
                if sub.owner != env.source {
                    // Foreign subscriptions are indistinguishable from
                    // nonexistent ones — no probing.
                    return reject(
                        tick,
                        RejectReason::UnknownSubscription { id: p.subscription },
                    );
                }
                let n = max.min(sub.queue.len());
                let events: Vec<GameEvent> = sub.queue.drain(..n).collect();
                let remaining = sub.queue.len() as u64;
                QueryData::Events { events, remaining }
            }
            Payload::CharacterPose(p) => {
                let Some(character) = self.characters.get(&p.character).cloned() else {
                    return reject(
                        tick,
                        RejectReason::UnknownCharacter {
                            name: p.character.clone(),
                        },
                    );
                };
                let eye = character.eye_m(&self.character_config);
                let voxel_size = self.character_config.voxel_size_m;
                let eye_voxel = Vec3i::new(
                    (eye.x / voxel_size).floor() as i64,
                    (eye.y / voxel_size).floor() as i64,
                    (eye.z / voxel_size).floor() as i64,
                );
                QueryData::CharacterPose {
                    name: character.name.clone(),
                    pos: character.pos_m,
                    vel: character.vel_m,
                    yaw: character.yaw,
                    pitch: character.pitch,
                    on_ground: character.on_ground,
                    eye_in_solid: self.block_at(eye_voxel).is_solid(),
                }
            }
            Payload::SenseRaycast(p) => {
                let Some(character) = self.characters.get(&p.character).cloned() else {
                    return reject(
                        tick,
                        RejectReason::UnknownCharacter {
                            name: p.character.clone(),
                        },
                    );
                };
                let dir = match p.dir {
                    Some(d) => DVec3::new(d.x, d.y, d.z),
                    None => character.view_dir(),
                };
                if !dir.is_finite() || dir.length_squared() <= 0.0 {
                    return reject(
                        tick,
                        RejectReason::PayloadInvalid {
                            reason: "raycast direction must be finite and nonzero".into(),
                        },
                    );
                }
                let dir = dir.normalize();
                let max_m = p
                    .max_distance_m
                    .unwrap_or(MAX_SENSE_RAYCAST_M)
                    .clamp(0.0, MAX_SENSE_RAYCAST_M);
                let voxel_size = self.character_config.voxel_size_m;
                let origin_v = character.eye_m(&self.character_config) / voxel_size;
                // Direction is unitless: meters-space and voxel-space agree.
                let hit = {
                    let world = RefCell::new(&mut *self);
                    let solid = |x: i64, y: i64, z: i64| {
                        world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid()
                    };
                    raycast_voxels(&solid, origin_v, dir, max_m / voxel_size)
                };
                match hit {
                    None => QueryData::CharacterRaycast {
                        hit: false,
                        voxel: None,
                        block: None,
                        normal: None,
                        distance_m: None,
                    },
                    Some(h) => {
                        let voxel = Vec3i::new(h.voxel.0, h.voxel.1, h.voxel.2);
                        // Distance to the entry face: the ray crossed the grid
                        // plane on the normal's axis (zero normal = eye already
                        // inside solid, distance 0).
                        let distance_v = match h.normal {
                            (0, 0, 0) => 0.0,
                            (nx, ny, nz) => {
                                let (axis, negative_normal) = if nx != 0 {
                                    (0, nx < 0)
                                } else if ny != 0 {
                                    (1, ny < 0)
                                } else {
                                    (2, nz < 0)
                                };
                                let coord = [h.voxel.0, h.voxel.1, h.voxel.2][axis];
                                // Negative normal = ray was moving +axis and
                                // entered through the low face.
                                let plane = if negative_normal { coord } else { coord + 1 } as f64;
                                let (o, d) = (origin_v[axis], dir[axis]);
                                ((plane - o) / d).max(0.0)
                            }
                        };
                        let block = block_name(self.block_at(voxel)).to_string();
                        QueryData::CharacterRaycast {
                            hit: true,
                            voxel: Some(voxel),
                            block: Some(block),
                            normal: Some(Vec3i::new(h.normal.0, h.normal.1, h.normal.2)),
                            distance_m: Some(distance_v * voxel_size),
                        }
                    }
                }
            }
            Payload::SenseSurroundings(p) => {
                let Some(character) = self.characters.get(&p.character) else {
                    return reject(
                        tick,
                        RejectReason::UnknownCharacter {
                            name: p.character.clone(),
                        },
                    );
                };
                if p.radius > MAX_SENSE_RADIUS_VOXELS {
                    return reject(
                        tick,
                        RejectReason::PayloadInvalid {
                            reason: format!(
                                "surroundings radius {} exceeds the sense range of {} voxels",
                                p.radius, MAX_SENSE_RADIUS_VOXELS
                            ),
                        },
                    );
                }
                let voxel_size = self.character_config.voxel_size_m;
                let center = Vec3i::new(
                    (character.pos_m.x / voxel_size).floor() as i64,
                    (character.pos_m.y / voxel_size).floor() as i64,
                    (character.pos_m.z / voxel_size).floor() as i64,
                );
                let r = i64::from(p.radius);
                let vol = Volume::new(
                    Vec3i::new(center.x - r, center.y - r, center.z - r),
                    Vec3i::new(center.x + r, center.y + r, center.z + r),
                );
                self.scan_data(vol)
            }
            Payload::SetBlock(_)
            | Payload::Fill(_)
            | Payload::EntitySpawn(_)
            | Payload::DefineItem(_)
            | Payload::DefineContentClass(_)
            | Payload::DefineClassMember(_)
            | Payload::EventsSubscribe(_)
            | Payload::SpawnCharacter(_)
            | Payload::SetMoveIntent(_)
            | Payload::SetLook(_)
            | Payload::Jump(_) => {
                return reject(tick, RejectReason::NotAQuery { id: env.id.clone() });
            }
        };
        QueryReceipt {
            tick_observed: tick,
            result: QueryResult::Ok(data),
        }
    }

    /// Palette + indices scan of an inclusive volume (shared by
    /// `world/scan_region` and the character's `sense_surroundings`).
    fn scan_data(&mut self, vol: Volume) -> QueryData {
        let mut palette: Vec<String> = Vec::new();
        let mut indices = Vec::with_capacity(vol.voxel_count() as usize);
        for y in vol.min.y..=vol.max.y {
            for z in vol.min.z..=vol.max.z {
                for x in vol.min.x..=vol.max.x {
                    let name = block_name(self.block_at(Vec3i::new(x, y, z)));
                    let idx = match palette.iter().position(|p| p == name) {
                        Some(i) => i,
                        None => {
                            palette.push(name.to_string());
                            palette.len() - 1
                        }
                    };
                    indices.push(idx as u32);
                }
            }
        }
        QueryData::Region {
            min: vol.min,
            max: vol.max,
            palette,
            indices,
        }
    }

    // ----------------------------------------------------------- hashing --

    /// FNV-1a hash of an inclusive region's block contents, for parity and
    /// determinism proofs. Reads lazily-generated terrain like any consumer.
    pub fn region_hash(&mut self, vol: Volume) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01B3);
            }
        };
        for y in vol.min.y..=vol.max.y {
            for z in vol.min.z..=vol.max.z {
                for x in vol.min.x..=vol.max.x {
                    let block = self.block_at(Vec3i::new(x, y, z));
                    eat(&(block as u16).to_le_bytes());
                }
            }
        }
        h
    }
}
