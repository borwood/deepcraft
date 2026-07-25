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

use dc_core::{Block, Chunk, ChunkPos, MaterialId, local_voxel, raycast_voxels};
use glam::DVec3;

use crate::bodies::{AnimClipDef, BodyPlanDef, validate_clip, validate_plan};
use crate::capability::requirement_for;
use crate::character::{
    CharacterConfig, CharacterState, Posture, standing_would_embed, step_character,
    valid_character_name,
};
use crate::classes::{ClassMemberDef, ContentClassDef, validate_class_contract, validate_params};
use crate::envelope::{
    BlockChange, CommandEnvelope, CommandReceipt, CommandResult, ConsumerId, Effects, QueryReceipt,
    QueryResult, ReceiptEntry, RejectReason, SubmitAck, Tick,
};
use crate::event::GameEvent;
use crate::identify::Identity;
use crate::payload::{ContentsView, EntityInfo, Payload, QueryData, Vec3i, Volume};
use crate::schema::{CommandKind, spec};

/// Per-command cap on fill/scan volume (voxels). Keeps one envelope from
/// stalling a tick; larger jobs batch across envelopes/ticks.
pub const MAX_REGION_VOXELS: u64 = 1 << 18; // 262,144 = a 64^3 box

/// Default max events returned by one poll.
pub const DEFAULT_POLL_MAX: u32 = 256;

/// Block name table. Since the block↔material collapse (journal/0087) a block
/// IS a material: every registry material resolves by its own `dc:` id
/// ([`MaterialId::qualified_name`]), alongside `dc:air` and the four legacy S1
/// tokens (`dc:stone`/`dc:dirt`/`dc:grass`/`dc:wood`) the walking-skeleton
/// terrain still emits. The data-driven block registry is a later slice.
pub fn block_from_name(name: &str) -> Option<Block> {
    Some(match name {
        "dc:air" => Block::Air,
        "dc:stone" => Block::Stone,
        "dc:dirt" => Block::Dirt,
        "dc:grass" => Block::Grass,
        "dc:wood" => Block::Wood,
        _ => Block::Material(MaterialId::from_qualified_name(name)?),
    })
}

pub fn block_name(block: Block) -> &'static str {
    match block {
        Block::Air => "dc:air",
        Block::Stone => "dc:stone",
        Block::Dirt => "dc:dirt",
        Block::Grass => "dc:grass",
        Block::Wood => "dc:wood",
        Block::Material(m) => m.qualified_name(),
    }
}

/// Build the `dc:world/get_contents` result: the full composition (the
/// authority) beside the stored, edit-aware `block` and the block the contents
/// themselves classify to.
///
/// The composition comes from [`Identity`], **not** raw
/// [`HostWorld::contents_at`] — so a voxel the world has no record for reports
/// `has_contents: false` and a `classified` that echoes the stored block,
/// instead of the pre-`identify` lie (`has_contents: true` +
/// `classified: dc:air` over solid stone, corrections #49).
fn contents_query_data(block: Block, identity: &Identity) -> QueryData {
    let classified = match identity.classified() {
        Some(b) => block_name(b),
        // The absent-contents rule: `classify` never runs on a voxel that was
        // never given a record — the stored block is all there is to report.
        None => block_name(block),
    };
    QueryData::Contents {
        block: block_name(block).to_string(),
        classified: classified.to_string(),
        has_contents: !identity.is_unrecorded(),
        contents: identity
            .mixture()
            .map(ContentsView::from_contents)
            .unwrap_or_default(),
    }
}

/// Every block name `set_block`/`fill` will resolve — the value source for the
/// `block` completion hook (schema.rs `Completer`). Kept in sync with
/// [`block_from_name`]: `dc:air`, the four legacy S1 tokens, then every registry
/// material by its qualified id.
pub static KNOWN_BLOCK_NAMES: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| {
        let mut names = vec!["dc:air", "dc:stone", "dc:dirt", "dc:grass", "dc:wood"];
        names.extend(MaterialId::all().map(MaterialId::qualified_name));
        names
    });

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
    RemoveBodyPlan(String),
    RemoveAnimClip(String),
}

/// Default cap on *resident generated-and-untouched* chunks (journal/0051).
/// Edited chunks are never counted here and never evicted. 2048 chunks is
/// ~64 MB at the current dense 32^3 `Chunk`; the client overrides it from its
/// streaming radius at the active scale ([`HostWorld::set_chunk_budget`]).
pub const DEFAULT_CHUNK_BUDGET: usize = 2048;

/// Fraction of the budget freed in one eviction sweep, as a divisor: the sweep
/// drops down to `budget - budget/CHUNK_EVICT_HYSTERESIS`, so the O(n log n)
/// sweep runs once per that many newly generated chunks instead of once per
/// chunk.
const CHUNK_EVICT_HYSTERESIS: usize = 8;

/// A resident chunk plus the bookkeeping eviction needs.
///
/// The doctrine (S11, "store only what the derivation cannot predict"): a
/// generated-and-untouched chunk is a pure function of (seed, generator, pos)
/// and can be dropped and re-derived byte-identically at any time. An *edited*
/// chunk holds information no derivation can reconstruct, so it is pinned
/// until the save layer exists to spill it.
struct ChunkSlot {
    chunk: Chunk,
    /// Set the first time any voxel in this chunk actually changes value.
    /// Once set it never clears — a rolled-back txn leaves the flag on, which
    /// errs toward retention (safe) rather than toward dropping an edit.
    edited: bool,
    /// Monotonic access stamp; the LRU key.
    touched: u64,
}

/// Chunk-store occupancy, for the client's memory probe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkResidency {
    /// Chunks currently held in memory (evictable + edited).
    pub resident: usize,
    /// Of those, the ones carrying at least one edit — never evicted.
    pub edited: usize,
    /// Lifetime count of generated-untouched chunks dropped.
    pub evicted: u64,
    /// The current cap on evictable chunks.
    pub budget: usize,
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

/// Optional per-chunk **material contents** source, parallel to
/// [`ChunkGenerator`]. The block generator classifies contents down to a single
/// [`Block`] per voxel before storage (that is all the sim/edit authority needs);
/// this seam keeps the *full* [`dc_core::VoxelContents`] reachable for the dev
/// inspector (`dc:world/get_contents`, the look-at readout) without the host
/// having to store 8× the bytes per voxel. It is a **pure function of `pos`** —
/// the render-authority's `chunk_contents`, blind to edits — so where a stored
/// block was edited it may disagree with the classified contents; the query
/// surfaces both so the divergence is legible. `None` (the default) = the host
/// has no contents to offer and `get_contents` answers block-only.
pub type ContentsSource = Box<dyn Fn(ChunkPos) -> Option<dc_core::ContentsGrid> + Send + Sync>;

/// The in-process world. Single-threaded, manual tick driver.
pub struct HostWorld {
    seed: u64,
    /// Base terrain for never-edited chunks. `None` = the built-in seeded
    /// hill-field (S5 behavior, byte-for-byte).
    generator: Option<ChunkGenerator>,
    /// Optional full-contents source for the dev inspector (`get_contents`,
    /// look-at readout). `None` = the host serves block-only contents queries.
    /// Never consulted by sim/edit/replay — purely a read-side instrument.
    contents_source: Option<ContentsSource>,
    /// Last completed tick. State always reflects exactly this tick.
    tick: Tick,
    /// Resident chunks. Generated-untouched entries are an evictable cache
    /// (bounded LRU, [`HostWorld::set_chunk_budget`]); edited entries are
    /// authoritative state and are pinned. See journal/0051.
    chunks: HashMap<ChunkPos, ChunkSlot>,
    /// Cap on the evictable (generated-untouched) half of `chunks`.
    chunk_budget: usize,
    /// Monotonic access counter feeding `ChunkSlot::touched`.
    touch_clock: u64,
    /// Lifetime evictions, for the memory probe.
    evicted_chunks: u64,
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
    /// Body plans and animation clips — the bodies registry (bodies.md steps
    /// 1–2). BTreeMaps: iteration order is part of the deterministic surface.
    body_plans: std::collections::BTreeMap<String, BodyPlanDef>,
    anim_clips: std::collections::BTreeMap<String, AnimClipDef>,
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
            contents_source: None,
            tick: 0,
            chunks: HashMap::new(),
            chunk_budget: DEFAULT_CHUNK_BUDGET,
            touch_clock: 0,
            evicted_chunks: 0,
            entities: Vec::new(),
            characters: std::collections::BTreeMap::new(),
            character_config: CharacterConfig::default(),
            next_entity_id: 1,
            items: std::collections::BTreeMap::new(),
            content_classes: std::collections::BTreeMap::new(),
            class_members: std::collections::BTreeMap::new(),
            body_plans: std::collections::BTreeMap::new(),
            anim_clips: std::collections::BTreeMap::new(),
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

    /// Install the full-contents source for the dev inspector (see
    /// [`ContentsSource`]). Additive and read-only: it never touches sim state,
    /// receipts, or replay identity — a world without one answers `get_contents`
    /// block-only. Idempotent; a later call replaces the source.
    pub fn set_contents_source(&mut self, source: ContentsSource) {
        self.contents_source = Some(source);
    }

    /// The raw [`dc_core::VoxelContents`] a voxel's **chunk grid** carries, as
    /// resolved through the installed [`ContentsSource`]. Resolves a whole
    /// chunk's contents grid and reads one voxel — a dev-query cost, off every
    /// hot path (gen time is not a constraint).
    ///
    /// **This is the source, not the answer.** Its `None` is a **chunk-level**
    /// fact (no source installed, or the whole 32³ chunk is empty), so
    /// `Some(VoxelContents::EMPTY)` conflates *"nothing is here"* with *"this
    /// voxel was never given a record"* — the corrections #49 defect. Ask
    /// [`HostWorld::identify`] for the honest per-voxel answer; reach for this
    /// only when you want the grid's literal contents.
    pub fn contents_at(&self, p: Vec3i) -> Option<dc_core::VoxelContents> {
        let source = self.contents_source.as_ref()?;
        let cpos = ChunkPos::from_world_voxel(p.x, p.y, p.z);
        let (lx, ly, lz) = local_voxel(p.x, p.y, p.z);
        let grid = source(cpos)?;
        Some(grid.get(lx, ly, lz))
    }

    /// **The honest identity surface** (journal/0101): what this voxel is made
    /// of, at one world position, untiered — or [`Identity::Unrecorded`] when
    /// the world has no composition record for it.
    ///
    /// Resident-first, derived-if-absent: the block half is the stored,
    /// edit-aware voxel ([`HostWorld::block_at`], which reads the resident
    /// chunk and falls back to the generator); the composition half comes from
    /// the installed [`ContentsSource`], which is a pure function of position
    /// today. See [`crate::identify`] for the enabler that separates *"no
    /// record"* from *"nothing here"*, and for the edit-blindness seam.
    /// (`&mut` because the resident-first block read may have to generate and
    /// cache the chunk — the same reason [`HostWorld::block_at`] takes it.)
    pub fn identify(&mut self, p: Vec3i) -> Identity {
        Identity::resolve(self.block_at(p), self.contents_at(p))
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

    pub fn body_plan(&self, name: &str) -> Option<&BodyPlanDef> {
        self.body_plans.get(name)
    }

    /// All body plans, in name order.
    pub fn body_plans(&self) -> impl Iterator<Item = &BodyPlanDef> {
        self.body_plans.values()
    }

    pub fn anim_clip(&self, name: &str) -> Option<&AnimClipDef> {
        self.anim_clips.get(name)
    }

    /// All animation clips, in name order.
    pub fn anim_clips(&self) -> impl Iterator<Item = &AnimClipDef> {
        self.anim_clips.values()
    }

    pub fn entities(&self) -> &[EntityInfo] {
        &self.entities
    }

    /// The block names this world resolves in `set_block`/`fill` — the value
    /// source the schema registry's `block` completion hook reads. `&self` is
    /// the future-facing seam: today the fixed [`KNOWN_BLOCK_NAMES`] table, but
    /// when blocks become data-driven this reports the world's registered set
    /// and the completion source does not change.
    pub fn block_names(&self) -> &'static [&'static str] {
        &KNOWN_BLOCK_NAMES[..]
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

    // ----------------------------------------------------------- eviction --

    /// Set the cap on resident *generated-and-untouched* chunks. Edited chunks
    /// are excluded from the cap and are never dropped.
    ///
    /// This is the headless seam for the client's viewer-derived working set:
    /// dc-api cannot know the streaming radius or the active voxel scale, so
    /// the consumer supplies the number as data (CLAUDE.md § Conventions — no
    /// rendering/OS dependency here). Correctness does not depend on the value:
    /// a dropped chunk re-derives byte-identically, so the budget only trades
    /// memory against regeneration work. Clamped to at least 1 so `chunk_at`
    /// can always keep the chunk it was asked for.
    pub fn set_chunk_budget(&mut self, budget: usize) {
        self.chunk_budget = budget.max(1);
        self.enforce_chunk_budget(None);
    }

    /// Chunk-store occupancy — what the memory probe reports.
    pub fn chunk_residency(&self) -> ChunkResidency {
        let edited = self.chunks.values().filter(|s| s.edited).count();
        ChunkResidency {
            resident: self.chunks.len(),
            edited,
            evicted: self.evicted_chunks,
            budget: self.chunk_budget,
        }
    }

    /// Drop least-recently-touched generated-untouched chunks until the
    /// evictable population is back under budget. `protect` is never dropped
    /// (the chunk the caller is about to hand out a reference to).
    ///
    /// Sweeps in batches (see [`CHUNK_EVICT_HYSTERESIS`]) so the sort is
    /// amortized across many generated chunks rather than run per insert.
    fn enforce_chunk_budget(&mut self, protect: Option<ChunkPos>) {
        let evictable = self.chunks.values().filter(|s| !s.edited).count();
        if evictable <= self.chunk_budget {
            return;
        }
        let target = self
            .chunk_budget
            .saturating_sub(self.chunk_budget / CHUNK_EVICT_HYSTERESIS)
            .max(1);
        let mut candidates: Vec<(u64, ChunkPos)> = self
            .chunks
            .iter()
            .filter(|(pos, slot)| !slot.edited && Some(**pos) != protect)
            .map(|(pos, slot)| (slot.touched, *pos))
            .collect();
        // Oldest touch first. Stamps are unique, so this order is total even
        // though the HashMap hands them over in an arbitrary order.
        candidates.sort_unstable_by_key(|(touched, _)| *touched);
        let drop_n = evictable.saturating_sub(target).min(candidates.len());
        for (_, pos) in candidates.into_iter().take(drop_n) {
            self.chunks.remove(&pos);
            self.evicted_chunks += 1;
        }
    }

    fn chunk_at(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.touch_clock += 1;
        let stamp = self.touch_clock;
        if !self.chunks.contains_key(&pos) {
            let c = self.generate_chunk(pos);
            self.chunks.insert(
                pos,
                ChunkSlot {
                    chunk: c,
                    edited: false,
                    touched: stamp,
                },
            );
            self.enforce_chunk_budget(Some(pos));
        }
        let slot = self.chunks.get_mut(&pos).expect("just inserted or present");
        slot.touched = stamp;
        &mut slot.chunk
    }

    /// Mark a chunk as carrying an edit — pinning it against eviction. The one
    /// place this is called from is [`HostWorld::set_block_raw`], the single
    /// voxel-writing path in this world.
    fn mark_edited(&mut self, pos: ChunkPos) {
        if let Some(slot) = self.chunks.get_mut(&pos) {
            slot.edited = true;
        }
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
        if prev != block {
            chunk.set(lx, ly, lz, block);
            // This chunk now holds information the generator cannot reproduce.
            // Pin it: eviction may only drop what re-derives (journal/0051).
            self.mark_edited(cpos);
        }
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
                        Undo::RemoveBodyPlan(name) => {
                            self.body_plans.remove(&name);
                        }
                        Undo::RemoveAnimClip(name) => {
                            self.anim_clips.remove(&name);
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
            Payload::DefineAnimClip(p) => {
                let clip = &p.0;
                if self.anim_clips.contains_key(&clip.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: clip.name.clone(),
                    });
                }
                validate_clip(clip).map_err(|reason| RejectReason::SchemaViolation { reason })?;
                self.anim_clips.insert(
                    clip.name.clone(),
                    AnimClipDef {
                        clip: clip.clone(),
                        defined_tick: t,
                        defined_by: env.source.clone(),
                    },
                );
                journal.push(Undo::RemoveAnimClip(clip.name.clone()));
                effects.anim_clips_defined.push(clip.name.clone());
            }
            Payload::DefineBodyPlan(p) => {
                let plan = &p.0;
                if self.body_plans.contains_key(&plan.name) {
                    return Err(RejectReason::AlreadyDefined {
                        key: plan.name.clone(),
                    });
                }
                // The verb→slot contract is checked here, against the clips
                // already registered — a plan that leaves a required slot
                // unfilled, or binds a missing/incompatible clip, rejects.
                validate_plan(plan, |name| {
                    self.anim_clips.get(name).map(|d| d.clip.clone())
                })
                .map_err(|reason| RejectReason::SchemaViolation { reason })?;
                self.body_plans.insert(
                    plan.name.clone(),
                    BodyPlanDef {
                        plan: plan.clone(),
                        defined_tick: t,
                        defined_by: env.source.clone(),
                    },
                );
                journal.push(Undo::RemoveBodyPlan(plan.name.clone()));
                effects.body_plans_defined.push(plan.name.clone());
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
            Payload::SetPosture(p) => {
                let Some(posture) = Posture::from_wire(&p.posture) else {
                    return Err(RejectReason::PayloadInvalid {
                        reason: format!(
                            "unknown posture `{}` (expected `standing` or `crouching`)",
                            p.posture
                        ),
                    });
                };
                let cfg = self.character_config;
                let current = self
                    .characters
                    .get(&p.character)
                    .ok_or_else(|| RejectReason::UnknownCharacter {
                        name: p.character.clone(),
                    })?
                    .clone();
                // Stand-up guard: rising from a crouch must not embed the taller
                // standing collider (crouching may have carried the body under a
                // low ceiling). The same solidity test the attach embed guard
                // uses; deterministic, so replay is untouched.
                if current.posture == Posture::Crouching && posture == Posture::Standing {
                    let embedded = {
                        let world = RefCell::new(&mut *self);
                        let solid = |x: i64, y: i64, z: i64| {
                            world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid()
                        };
                        standing_would_embed(current.pos_m, &cfg, &solid)
                    };
                    if embedded {
                        return Err(RejectReason::PostureBlocked {
                            character: p.character.clone(),
                        });
                    }
                }
                let character = self
                    .characters
                    .get_mut(&p.character)
                    .expect("presence checked above");
                journal.push(Undo::RestoreCharacter(Box::new(character.clone())));
                character.posture = posture;
            }
            // Queries never reach apply (submit rejects them).
            Payload::GetBlock(_)
            | Payload::GetContents(_)
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
            Payload::GetContents(p) => {
                let block = self.block_at(p.pos);
                contents_query_data(block, &self.identify(p.pos))
            }
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
                // The feet voxel, in the same world-voxel space `get_block`
                // takes — floor per axis, exactly as `eye_voxel` above (the
                // active scale's authoritative meters→voxel conversion).
                let pos_voxel = Vec3i::new(
                    (character.pos_m.x / voxel_size).floor() as i64,
                    (character.pos_m.y / voxel_size).floor() as i64,
                    (character.pos_m.z / voxel_size).floor() as i64,
                );
                QueryData::CharacterPose {
                    name: character.name.clone(),
                    pos: character.pos_m,
                    vel: character.vel_m,
                    yaw: character.yaw,
                    pitch: character.pitch,
                    on_ground: character.on_ground,
                    eye_in_solid: self.block_at(eye_voxel).is_solid(),
                    pos_voxel,
                    posture: character.posture.to_wire().to_string(),
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
                        contents: None,
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
                        // `None` means *no record here* and nothing else — the
                        // doc's promise, kept since `identify` (corrections
                        // #49: this used to hand back `Some(<empty view>)` for
                        // unrecorded rock, so a sensing character read a hit on
                        // solid stone whose composition was nothing).
                        let contents = self
                            .identify(voxel)
                            .mixture()
                            .map(ContentsView::from_contents);
                        QueryData::CharacterRaycast {
                            hit: true,
                            voxel: Some(voxel),
                            block: Some(block),
                            normal: Some(Vec3i::new(h.normal.0, h.normal.1, h.normal.2)),
                            distance_m: Some(distance_v * voxel_size),
                            contents,
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
            | Payload::DefineBodyPlan(_)
            | Payload::DefineAnimClip(_)
            | Payload::EventsSubscribe(_)
            | Payload::SpawnCharacter(_)
            | Payload::SetMoveIntent(_)
            | Payload::SetLook(_)
            | Payload::SetPosture(_)
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
                    eat(&block.ordinal().to_le_bytes());
                }
            }
        }
        h
    }
}
