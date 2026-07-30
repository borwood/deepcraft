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
//! active authority (dc-api's pluggable-generator addition), so the world the
//! host serves is what the client streams. ROADMAP 3c-1 gives that seam a
//! choice: at N=2 (boot, key 2) it is the real hierarchical worldgen
//! (dc-worldgen), one `WorldGenerator` behind a `Mutex` serving both the chunk
//! seam and the surface-scan ceiling; at keys 3/4 it is the legacy S1
//! [`TerrainGen`]. The near-field streamed chunks (ChunkMap) are clones of the
//! authority; the unloaded-neighbour and far-mesh fallbacks still sample
//! `TerrainGen` directly — a documented seam at the load-radius edge and in the
//! far rings (ROADMAP Observed) until the far field becomes worldgen-shaped.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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
use dc_worldgen::{DeepOverrides, Extent, Pregen, WorldGenerator, WorldParams};
use glam::DVec3;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::PLAYER_HEIGHT_M;
use crate::app::{
    ChunkMap, CurrentScale, FloatingOrigin, Fullbright, FullbrightMaterialHandle,
    TerrainMaterialHandle, to_render,
};
use crate::mcp::{BridgeRequest, McpBridge};
use crate::meshing::mesh_chunk;
use crate::physdemo::PhysicsDemo;
use crate::player::{PLAYER_WIDTH_M, Player};
use crate::streaming::to_bevy_mesh;
use crate::worldgen::{TerrainGen, true_surface_m};

/// World extent baked into the worldgen authority when `--extent` is not given.
/// Medium is the design doc's default class; its pregen runs in ~15 ms
/// (journal/0007), trivial at boot.
const WORLDGEN_EXTENT: Extent = Extent::Medium;

/// The gen-time knobs a launch flag can dial into the worldgen authority
/// (deep-config plumbing, journal/0039): the world [`Extent`] and the deep-time
/// [`DeepOverrides`] (`--tectonics`, `--full-agents`, `--amplitude`,
/// `--erosion-budget`, `--weather-inventory`). Bundled so
/// they thread from `app::run` down to [`Pregen::run_with`] as one value rather
/// than a growing argument list, and held as a Bevy [`Resource`] so a key-2
/// scale switch rebuilds the world with the same options. `Default` = Medium
/// extent + empty overrides, which is byte-identical to the pre-plumbing boot.
#[derive(Resource, Clone, Debug)]
pub struct GenOptions {
    pub extent: Extent,
    pub deep: DeepOverrides,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            extent: WORLDGEN_EXTENT,
            deep: DeepOverrides::default(),
        }
    }
}

/// The active surface authority — the source of both never-edited terrain (the
/// `HostWorld`'s chunk generator) and the per-column analytic ceiling the
/// surface scan starts from.
///
/// - `Terrain`: the legacy S1 [`TerrainGen`] (keys 3/4, any scale).
/// - `Worldgen`: the real hierarchical worldgen (ROADMAP 3c-1), N=2-baked. The
///   `Arc<Mutex<..>>` is the SAME generator the `ChunkGenerator` closure holds,
///   so the analytic ceiling and the streamed chunks come from one world and
///   one cache. The `Mutex` is what makes the (now `Arc`-cached, `Send`)
///   generator fit the `Fn + Send + Sync` seam; it hides nothing about
///   determinism — the closure stays a pure function of `pos`.
///
///   The `pregen` is the SAME `Arc<Pregen>` that generator is built over, kept
///   beside it so a background mesh task can mint its OWN
///   `WorldGenerator::new_owned(pregen.clone())` and resolve neighbour contents /
///   far coarse surface off-thread with NO contention on the shared `Mutex`
///   (journal/0084). Generation is a pure function of `(pregen, pos)`, so a
///   per-task generator over this same pregen is byte-identical to the shared
///   one — recomputing a cold cache costs generator time, which is free under
///   the two-clocks doctrine.
enum SurfaceAuthority {
    Terrain(TerrainGen),
    Worldgen {
        generator: Arc<Mutex<WorldGenerator<'static>>>,
        pregen: Arc<Pregen>,
    },
}

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
    /// The active scale — the voxel lattice the hosted world lives on (N=2 for
    /// the worldgen authority; 3/4 for the legacy terrain authority).
    scale: VoxelScale,
    /// The active surface authority. Its analytic per-column height seeds the
    /// scan ceiling for spawn / `surface:true` teleport / `surface:true`
    /// attach; the snap and embed guard read the hosted world's real solidity
    /// (edits included) — the analytic is only a cheap upper bound
    /// (journal/0006). For the worldgen variant this is the SAME generator the
    /// chunk seam serves from.
    surface: SurfaceAuthority,
}

impl Authority {
    /// Build the hosted world for the given player-height scale, choosing the
    /// authority the way the scale keys do (ROADMAP 3c-1, **NEEDS
    /// RATIFICATION**): **N=2 (key 2) = the real hierarchical worldgen** (the
    /// ratified S1 scale, geology visible + walkable), any other scale (keys
    /// 3/4) = the legacy S1 [`TerrainGen`] at that resolution. dc-worldgen is
    /// N=2-baked, so its authority only makes sense at N=2.
    ///
    /// The default-options convenience over [`Authority::new_with`]. Production
    /// (app.rs) always boots through `new_with` to carry the launch flags'
    /// [`GenOptions`], so this is only reached from tests that don't exercise
    /// gen-time overrides — hence `#[cfg(test)]` (a bin crate flags an
    /// otherwise-uncalled `pub fn` as dead).
    #[cfg(test)]
    pub fn new(seed: i32, player_voxels: u32) -> Self {
        Self::new_with(seed, player_voxels, &GenOptions::default())
    }

    /// Like [`Authority::new`], but with the gen-time [`GenOptions`] (extent +
    /// deep-time overrides) the launch flags parsed. The options only bite on
    /// the worldgen authority (key 2); the legacy S1 terrain authority (keys
    /// 3/4) ignores them. Passing `&GenOptions::default()` reproduces
    /// [`Authority::new`] exactly.
    pub fn new_with(seed: i32, player_voxels: u32, opts: &GenOptions) -> Self {
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, player_voxels);
        if player_voxels == 2 {
            Self::new_worldgen(seed, scale, opts)
        } else {
            Self::new_terrain(seed, scale)
        }
    }

    /// The legacy S1 terrain authority (keys 3/4). The generator closure owns
    /// its own `TerrainGen` (same seed => identical noise), keeping the closure
    /// `Send + Sync` and deterministic.
    fn new_terrain(seed: i32, scale: VoxelScale) -> Self {
        let terrain = TerrainGen::new(seed);
        let world = Self::host_world(seed, scale, move |pos| terrain.generate_chunk(scale, pos));
        Self::finish(
            scale,
            world,
            SurfaceAuthority::Terrain(TerrainGen::new(seed)),
        )
    }

    /// The worldgen authority (key 2 / boot): build the world history (the
    /// pregen — the "generating world history…" moment, ~15 ms at Medium),
    /// then wrap ONE `WorldGenerator` behind a `Mutex` and serve chunks through
    /// it. The same `Arc` backs both the chunk seam and the surface-scan
    /// ceiling, so streamed terrain and surface queries are one world.
    ///
    /// Determinism: the closure locks the generator and calls `generate_chunk`,
    /// which is a pure function of `pos` — the collapse caches memoize but
    /// never reorder output (proven byte-identical in dc-worldgen and by
    /// [`tests::worldgen_seam_is_order_independent`]).
    fn new_worldgen(seed: i32, scale: VoxelScale, opts: &GenOptions) -> Self {
        let pregen = Arc::new(Pregen::run_with(
            WorldParams {
                seed: seed as u64,
                extent: opts.extent,
            },
            &opts.deep,
        ));
        let generator = Arc::new(Mutex::new(WorldGenerator::new_owned(pregen.clone())));
        let seam = generator.clone();
        let mut world = Self::host_world(seed, scale, move |pos| {
            crate::devicelost::lock_forgiving(&seam).generate_chunk(pos)
        });
        // Dev inspector (`dc:world/get_contents`, the look-at readout): the full
        // per-voxel contents, resolved through the SAME generator the chunk seam
        // and `chunk_contents` serve from — the render authority, a pure function
        // of pos (blind to edits). Read-only; never touches sim/replay identity.
        let contents_gen = generator.clone();
        world.set_contents_source(Box::new(move |pos| {
            crate::devicelost::lock_forgiving(&contents_gen).chunk_contents(pos)
        }));
        Self::finish(
            scale,
            world,
            SurfaceAuthority::Worldgen { generator, pregen },
        )
    }

    /// Wrap a chunk generator in a `HostWorld` with the client's character
    /// config for `scale` (shared by both authority constructors).
    fn host_world(
        seed: i32,
        scale: VoxelScale,
        generator: impl Fn(ChunkPos) -> dc_core::Chunk + Send + Sync + 'static,
    ) -> HostWorld {
        let mut world = HostWorld::with_generator(seed as u64, Box::new(generator));
        // Character bodies share the player's dimensions and dynamics
        // (dc-api's defaults ARE the player constants); only the voxel size
        // follows the active scale. Part of the replay identity.
        world.set_character_config(dc_api::CharacterConfig {
            voxel_size_m: scale.voxel_size_m(),
            tick_dt_s: HOST_TICK_DT,
            ..dc_api::CharacterConfig::default()
        });
        world
    }

    /// Cap on *generated-and-untouched* chunks resident in the hosted world,
    /// derived from the client's own streaming volume at `scale`
    /// (journal/0051). dc-api is headless and cannot know the streaming radius
    /// or the voxel scale, so the client supplies the working-set size as data.
    ///
    /// The shape: the sphere of chunks the streamer will hold at the unload
    /// radius, doubled — the streamed set is the near-field floor, and the
    /// slack absorbs the query traffic that reaches *outside* it (grounding
    /// scans, surface probes, raycasts, border-face culls against unstreamed
    /// neighbours). Clamped: never below 1024 (a coarse scale would otherwise
    /// derive a budget smaller than one frame's work), never above 16 384
    /// (~512 MB — at the finest scales the derived sphere runs to tens of
    /// thousands of chunks and the ceiling has to win, at the cost of some
    /// regeneration). Correctness is indifferent to the number: an evicted
    /// chunk re-derives byte-identically, so this only trades RAM for
    /// generator work.
    fn chunk_budget_for(scale: VoxelScale) -> usize {
        // Measurement escape hatch, in the `DC_MEM_PROBE` family: an explicit
        // budget, so the RSS repro can be run A/B on ONE binary — a huge value
        // reproduces the pre-eviction behaviour (journal/0051) exactly.
        if let Ok(v) = std::env::var("DC_CHUNK_BUDGET")
            && let Ok(n) = v.parse::<usize>()
        {
            return n.max(1);
        }
        let chunk_m = scale.voxels_to_meters(f64::from(dc_core::CHUNK_SIZE));
        let r = crate::streaming::UNLOAD_RADIUS_M / chunk_m + 1.0;
        let sphere = (4.0 / 3.0) * std::f64::consts::PI * r * r * r;
        ((sphere * 2.0) as usize).clamp(1024, 16_384)
    }

    /// Load the **first pack**: the vanilla bodies content, as a registry
    /// command batch through the one door.
    ///
    /// `dc_api::bodies::vanilla_body_pack()` had existed since the body-plan
    /// milestone on the stated principle "vanilla is the first pack", and
    /// **nothing but a dc-api unit test had ever called it** — the renderer
    /// reached for `biped_plan()`/`biped_clips()` as compiled-in Rust, so the
    /// door was built and never travelled (spines.md § A-4). This is the
    /// traversal: the clips and the biped plan arrive as `DefineAnimClip` /
    /// `DefineBodyPlan` commands under a `registry.define(dc)` grant, and
    /// `character.rs` reads the resulting registry. First-party content ships
    /// through the same surface a third party would use — north-star
    /// § core/plugin boundary, *"first-party content ships through the same SDK,
    /// not a privileged internal path"*.
    ///
    /// The batch is **submitted, not applied**: it lands at the first tick
    /// boundary like every other command, ahead of anything a consumer submits
    /// later (total order), so a character spawned at any point after boot always
    /// finds its plan registered. The renderer simply has no body assets to build
    /// until then, which costs nothing — no character can exist before a tick.
    ///
    /// **The second batch is the retargeting experiment, not content.**
    /// `experiment_body_pack()` registers `dc:body/stout`, the deliberately
    /// ill-proportioned instrument that tests bodies.md's IK claim. It is loaded
    /// separately, and *after* vanilla (it binds vanilla's clips), so that
    /// "vanilla" keeps meaning the vanilla content — deleting the experiment is
    /// this one `chain` plus `dc_api::bodies::{stout_plan, experiment_body_pack}`.
    /// *Existence is not standing*: an unratified body inside the default pack is
    /// how bootstrap fabrication becomes something a later session assumes.
    fn load_body_packs(world: &mut HostWorld) {
        let source = ConsumerId::new(ConsumerKind::Plugin, "vanilla-pack");
        let token = CapabilityToken::new(vec![Grant::RegistryDefine {
            namespace: "dc".into(),
        }]);
        let batch = dc_api::bodies::vanilla_body_pack()
            .into_iter()
            .chain(dc_api::bodies::experiment_body_pack());
        for payload in batch {
            let envelope = CommandEnvelope {
                id: payload.command_id().to_string(),
                source: source.clone(),
                grant: token.clone(),
                payload,
                target_tick: None,
                txn: None,
            };
            if let Err(entry) = world.submit(envelope) {
                // Structurally impossible (the batches are generated from the
                // typed source), but a silent body-less world would be baffling.
                error!("body pack rejected: {:?}", entry.receipt.result);
            }
        }
    }

    /// Assemble the consumer identities / tokens shared by every authority.
    fn finish(scale: VoxelScale, mut world: HostWorld, surface: SurfaceAuthority) -> Self {
        world.set_chunk_budget(Self::chunk_budget_for(scale));
        Self::load_body_packs(&mut world);
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
            scale,
            surface,
        }
    }

    /// The render-only per-voxel material contents of a chunk over the ACTIVE
    /// authority, or `None` when there are none (the S1 terrain authority, or a
    /// debris-free worldgen chunk). This is the seam's parallel material path
    /// (ROADMAP 3c-2): it locks the SAME `WorldGenerator` the chunk seam serves
    /// blocks from — so the column caches are already warm — and resolves the
    /// order-dependent mixture table away into canonical [`ContentsGrid`] before
    /// anything leaves the generator. Blocks (replay identity) come from the
    /// `HostWorld`; these contents never touch sim state, receipts, or replay.
    pub fn chunk_contents(&self, pos: ChunkPos) -> Option<dc_core::ContentsGrid> {
        match &self.surface {
            SurfaceAuthority::Terrain(_) => None,
            SurfaceAuthority::Worldgen { generator, .. } => {
                crate::devicelost::lock_forgiving(generator).chunk_contents(pos)
            }
        }
    }

    /// The shared `Arc<Pregen>` a background mesh task mints its OWN
    /// `WorldGenerator` from, to resolve neighbour contents / far coarse surface
    /// off-thread without touching the shared generator `Mutex` (journal/0084).
    /// `None` under the S1 terrain authority (which has no worldgen contents or
    /// coarse summary — its border culls are binary, needing no generator). A
    /// per-task generator over this pregen is byte-identical to the shared one.
    pub fn worldgen_pregen(&self) -> Option<Arc<Pregen>> {
        match &self.surface {
            SurfaceAuthority::Terrain(_) => None,
            SurfaceAuthority::Worldgen { pregen, .. } => Some(pregen.clone()),
        }
    }

    /// A short label for the active authority (window title / diagnostics).
    pub fn authority_label(&self) -> &'static str {
        match self.surface {
            SurfaceAuthority::Terrain(_) => "S1-terrain",
            SurfaceAuthority::Worldgen { .. } => "worldgen",
        }
    }

    /// Whether the active far field is the worldgen coarse-summary **horizon**
    /// (key 2) rather than the legacy S1 volumetric far mesh (keys 3/4). The
    /// far-field horizon (journal/0022) is built from the worldgen authority's
    /// OWN surface summary; the S1 authority keeps the S1 far mesh that was never
    /// wrong for the S1 world.
    pub fn far_field_is_worldgen(&self) -> bool {
        matches!(self.surface, SurfaceAuthority::Worldgen { .. })
    }

    /// The coarse far-field surface summary — surface height (in active-scale
    /// voxels) and surface block — at a world voxel column, sampled from the
    /// worldgen authority's OWN elevation lattice (journal/0022). `None` under the
    /// S1 authority (which renders its own far mesh). The generator `Mutex` gives
    /// the interior mutability its pyramid memoization needs, so this reads
    /// through `&self`; the SAME generator the chunk seam and the surface-scan
    /// ceiling use — one world, no second opinion (docs/ARCHITECTURE.md § One
    /// world-answer surface).
    ///
    /// journal/0084 moved the far-derive's coarse sampling off-thread onto a
    /// per-task generator (which calls `WorldGenerator::coarse_surface` directly,
    /// bypassing this `Mutex`-locking wrapper), so production no longer calls this
    /// — it survives as the shared-path oracle the per-task path is proven equal
    /// to (`tests::per_task_generator_is_byte_identical_to_the_shared_one`).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn worldgen_coarse_surface(&self, vx: i64, vz: i64) -> Option<(i32, dc_core::Block)> {
        match &self.surface {
            SurfaceAuthority::Terrain(_) => None,
            SurfaceAuthority::Worldgen { generator, .. } => {
                Some(crate::devicelost::lock_forgiving(generator).coarse_surface(vx, vz))
            }
        }
    }

    /// The per-column analytic surface height (meters) at `(xm, zm)` over the
    /// active authority: an UPPER bound on the true voxel surface used to seed
    /// each column's downward scan. For `Terrain`, `surface_height_m`; for
    /// `Worldgen`, the collapse `ColumnRec` height (surface voxel top) at that
    /// column.
    fn analytic_height_m(&self, xm: f64, zm: f64) -> f64 {
        match &self.surface {
            SurfaceAuthority::Terrain(t) => t.surface_height_m(xm, zm),
            SurfaceAuthority::Worldgen { generator, .. } => {
                let scale = self.scale;
                let (vx, vz) = (scale.voxel_at(xm), scale.voxel_at(zm));
                let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
                let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
                let col = crate::devicelost::lock_forgiving(generator).column_record(cx, cz);
                f64::from(col.heights[lz * 32 + lx]) * scale.voxel_size_m()
            }
        }
    }

    /// The TRUE walking surface (meters) under a footprint at `(xm, zm)` over
    /// the ACTIVE authority: the highest solid voxel top across the footprint
    /// columns, read from the hosted world's real solidity (edits included),
    /// each column scanned from its own **exact** authority ceiling — the
    /// collapse `ColumnRec` height for worldgen (deep-time-aware, journal/0015),
    /// `surface_height_m` for S1. `None` when nothing solid is under the whole
    /// footprint: an honest miss the caller must reject, never a buried point.
    /// Works over both authorities — the only difference is where the per-column
    /// ceiling comes from.
    pub fn true_surface_m(&mut self, xm: f64, zm: f64, footprint_half_m: f64) -> Option<f64> {
        let scale = self.scale;
        // Split the borrow: the analytic ceiling reads `self.surface`
        // (immutably / via the generator mutex), the solidity reads
        // `self.world` (mutably, for lazy chunk generation). Disjoint fields.
        let surface = &self.surface;
        let world = RefCell::new(&mut self.world);
        let solid =
            |x: i64, y: i64, z: i64| world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid();
        match surface {
            SurfaceAuthority::Terrain(t) => true_surface_m(
                &solid,
                |x, z| t.surface_height_m(x, z),
                scale,
                xm,
                zm,
                footprint_half_m,
            ),
            SurfaceAuthority::Worldgen { generator, .. } => {
                let worldgen = generator.clone();
                let analytic = |x: f64, z: f64| {
                    let (vx, vz) = (scale.voxel_at(x), scale.voxel_at(z));
                    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
                    let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
                    let col = crate::devicelost::lock_forgiving(&worldgen).column_record(cx, cz);
                    f64::from(col.heights[lz * 32 + lx]) * scale.voxel_size_m()
                };
                true_surface_m(&solid, analytic, scale, xm, zm, footprint_half_m)
            }
        }
    }

    /// Solidity at a **world voxel** coordinate over the active scale, read
    /// from the **authoritative** hosted world (terrain + edits), lazily
    /// generating the containing chunk if it has not streamed yet.
    ///
    /// This is the client's single world-answer surface for solidity (doctrine,
    /// docs/ARCHITECTURE.md § "One world-answer surface"). Every gameplay
    /// system that asks "is this voxel solid?" — player collision, character
    /// ground-finding, the crosshair edit raycast, physics collider tiles, and
    /// mesh-border face culling — routes here or through the closure it backs.
    ///
    /// The client-side [`ChunkMap`] is a render/collision *cache*: it answers
    /// only for chunks it holds and never invents an answer for one it does
    /// not. Its former miss path fell back to the legacy S1 [`TerrainGen`] — a
    /// *different world* under the worldgen authority (S1 ground ~8 m, worldgen
    /// ~1000 m) — which silently shipped a wrong-world answer into every one of
    /// those systems for any not-yet-streamed chunk (journal/0015–0017).
    pub fn is_solid_voxel(&mut self, x: i64, y: i64, z: i64) -> bool {
        self.block_voxel(x, y, z).is_solid()
    }

    /// The block at a world voxel over the active authority (lazily generating
    /// the containing chunk) — the un-thresholded form of
    /// [`Self::is_solid_voxel`]. The mesher's occupancy-aware culling needs the
    /// block itself, because whether a voxel's *contents* are consulted for its
    /// render height is gated on the block (journal/0057).
    pub fn block_voxel(&mut self, x: i64, y: i64, z: i64) -> dc_core::Block {
        self.world.block_at(Vec3i::new(x, y, z))
    }

    /// Is the voxel at world position `p` (meters) solid? Meters-typed shim over
    /// [`Self::is_solid_voxel`] at the active scale — what `eye_in_solid`
    /// consults (journal/0015).
    pub fn is_solid_m(&mut self, p: DVec3) -> bool {
        let scale = self.scale;
        self.is_solid_voxel(
            scale.voxel_at(p.x),
            scale.voxel_at(p.y),
            scale.voxel_at(p.z),
        )
    }

    /// Open-ground spawn over the active authority: spiral out from the origin
    /// to a column whose analytic surface is clearly above sea level, then seat
    /// the feet on the TRUE voxel surface there (never the analytic height,
    /// which under-reports on slopes — journal/0006). Mirrors the free
    /// `crate::app::find_open_spawn` but over whichever authority is live.
    pub fn find_open_spawn(&mut self) -> DVec3 {
        let (mut sx, mut sz) = (0.0f64, 0.0f64);
        'search: for ring in 0..64 {
            let d = f64::from(ring) * 12.0;
            for (ox, oz) in [
                (0.0, d),
                (0.0, -d),
                (d, 0.0),
                (-d, 0.0),
                (d, d),
                (-d, d),
                (d, -d),
                (-d, -d),
            ] {
                if self.analytic_height_m(ox, oz) > 2.0 {
                    (sx, sz) = (ox, oz);
                    break 'search;
                }
            }
        }
        // The ring search only accepts columns whose analytic surface is
        // clearly above sea level, so the true-surface scan finds ground there;
        // if it somehow does not (all air under the footprint), fall back to the
        // analytic height rather than panicking — a deterministic, above-ground
        // landing.
        let surface = self
            .true_surface_m(sx, sz, PLAYER_WIDTH_M / 2.0)
            .unwrap_or_else(|| self.analytic_height_m(sx, sz));
        DVec3::new(sx, surface + 2.0, sz)
    }

    /// Freeze a character on session disconnect (API.md § Characters, DECIDED
    /// 2026-07-19): zero its move intent so the body stands where it was left.
    /// Submitted as an ordinary zero move-intent command under the surface's
    /// parent token, so it rides the same receipted, tick-quantized rail as any
    /// controller verb — the replay identity is unchanged. No-op if the
    /// character is already gone.
    pub fn freeze_character(&mut self, name: &str) {
        if self.world.character(name).is_none() {
            return;
        }
        let envelope = CommandEnvelope {
            id: dc_api::ids::CHARACTER_SET_MOVE_INTENT.to_string(),
            source: self.character_surface_consumer.clone(),
            grant: self.character_parent_token.clone(),
            payload: Payload::SetMoveIntent(dc_api::payload::SetMoveIntent {
                character: name.to_string(),
                dx: 0.0,
                dz: 0.0,
                speed: 0.0,
            }),
            target_tick: None,
            txn: None,
        };
        let _ = self.world.submit(envelope);
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
    /// `body_plan` names the registered plan the new body **wears**; `None` is the
    /// identity default (`dc:body/biped`), so every existing caller is unchanged.
    /// An unregistered name is refused by the host with a receipt, and the refusal
    /// is surfaced here as `ok:false, code:"unknown_body_plan"` — a silent fallback
    /// would render a body the session did not ask for.
    pub fn handle_character_attach(
        &mut self,
        name: &str,
        pos: dc_api::payload::Vec3f,
        surface: bool,
        body_plan: Option<String>,
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
        // Fail the plan name BEFORE the placement work: an unregistered plan is a
        // content error, and answering it with "obstructed" would be a lie.
        if let Some(plan) = &body_plan
            && self.world.body_plan(plan).is_none()
        {
            let known: Vec<&str> = self
                .world
                .body_plans()
                .map(|d| d.plan.name.as_str())
                .collect();
            let _ = reply.send(json!({
                "ok": false,
                "code": "unknown_body_plan",
                "character": name,
                "body_plan": plan,
                "registered": known,
                "error": format!(
                    "cannot attach `{name}`: no registered body plan `{plan}`. \
                     Registered plans: {known:?}."
                ),
            }));
            return;
        }

        // Body dimensions and voxel lattice from the world's character config.
        let cfg = *self.world.character_config();
        let vs = cfg.voxel_size_m;
        let half_m = cfg.width_m / 2.0;

        // Optional surface snap: rest the feet on the true voxel surface under
        // the footprint at the requested x/z, over the ACTIVE authority (the
        // world's real solidity, edits included; the authority's analytic
        // height only seeds each column's scan ceiling).
        let mut feet = pos;
        if surface {
            match self.true_surface_m(pos.x, pos.z, half_m) {
                Some(y) => feet.y = y + 0.05,
                None => {
                    // No ground under the footprint (open air / chasm): refuse
                    // rather than snap the body to a buried fallback y.
                    let _ = reply.send(json!({
                        "ok": false,
                        "code": "no_surface",
                        "character": name,
                        "error": format!(
                            "cannot attach `{name}`: no solid ground found under the footprint \
                             at ({:.2}, {:.2}) to drop onto. Choose a position over land.",
                            pos.x, pos.z
                        ),
                    }));
                    return;
                }
            }
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
                body_plan,
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
    mut commands: Commands,
) {
    let Some(bridge) = bridge else { return };
    // Poison-tolerant (journal/0054): the previous `let Ok(..) else return`
    // would silently and PERMANENTLY stop draining the MCP bridge after one
    // unrelated panic — an agent's tool calls would just stop being answered,
    // with nothing said. A poisoned queue is still a queue.
    let mut rx = crate::devicelost::lock_forgiving(&bridge.rx);
    while let Ok(request) = rx.try_recv() {
        match request {
            BridgeRequest::Api { tool, args, reply } => {
                authority.handle_api_call(&tool, &args, reply);
            }
            BridgeRequest::PoseGet { reply } => {
                let _ = reply.send(pose_json(&player, &mut authority));
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
                if let Some(y) = yaw {
                    player.yaw = y;
                }
                if let Some(p) = pitch {
                    player.pitch = p.clamp(-1.55, 1.55);
                }
                // A `surface` teleport ALWAYS reports `surface_snapped`
                // (journal/0018); a plain teleport reports the ordinary pose.
                let value = if surface {
                    surface_teleport_reply(&mut player, &mut authority)
                } else {
                    pose_json(&player, &mut authority)
                };
                let _ = reply.send(value);
            }
            BridgeRequest::Screenshot { name, reply } => {
                crate::mcp::take_screenshot(&mut commands, &name, reply);
            }
            BridgeRequest::CharacterAttach {
                name,
                pos,
                surface,
                body_plan,
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
                authority.handle_character_attach(&name, pos, surface, body_plan, reply);
            }
            BridgeRequest::CharacterApi {
                tool,
                args,
                session_character,
                reply,
            } => {
                authority.handle_character_api(&tool, &args, &session_character, reply);
            }
            BridgeRequest::CharacterFreeze { name } => {
                // A character session disconnected: freeze its body where it
                // was left (API.md § Characters, DECIDED 2026-07-19).
                authority.freeze_character(&name);
            }
        }
    }
}

fn pose_json(player: &Player, authority: &mut Authority) -> Value {
    // Eye = camera height: feet + 90% of player height (player.rs camera).
    // If this voxel is solid, every screenshot is backface nonsense — the
    // walk-3 lesson (journal/corrections.md #3): the walker must know. Read the
    // AUTHORITATIVE hosted world, not the client `ChunkMap` (whose unstreamed
    // fallback is the wrong world under the worldgen authority — journal/0015).
    let eye = player.pos_m + glam::DVec3::new(0.0, PLAYER_HEIGHT_M * 0.9, 0.0);
    let eye_in_solid = authority.is_solid_m(eye);
    // The walker speaks two languages: `pos` is meters, block queries
    // (`world_get_block`, `scan_region`) are world voxels. Echo the feet voxel
    // in the SAME conversion the authority uses (`scale.voxel_at`), so a
    // cross-check never needs a mental unit conversion (corrections #10 — a
    // meters-vs-voxels misread cost a full agent cycle).
    let scale = authority.scale;
    let pos_voxel = json!({
        "x": scale.voxel_at(player.pos_m.x),
        "y": scale.voxel_at(player.pos_m.y),
        "z": scale.voxel_at(player.pos_m.z),
    });
    json!({
        "pos": { "x": player.pos_m.x, "y": player.pos_m.y, "z": player.pos_m.z },
        "pos_voxel": pos_voxel,
        "yaw": player.yaw,
        "pitch": player.pitch,
        "fly": player.fly,
        "on_ground": player.on_ground,
        "eye_in_solid": eye_in_solid,
    })
}

/// Build the reply for a `surface`-flagged teleport: snap the feet to the TRUE
/// voxel surface under the player's footprint at (x, z) — over the ACTIVE
/// authority, reading the hosted world's live solidity (edits included), from
/// an honest per-column ceiling that tracks the deep-time surface (journal/0004,
/// 0006, 0015). On a genuine miss (no ground under the footprint) the position
/// is LEFT as requested. Either way `surface_snapped` is ALWAYS present — true
/// when the scan seated the feet, false on a miss. Absence-means-success was the
/// instrument ambiguity walk 13 filed (journal/0018): silence is not a reading.
fn surface_teleport_reply(player: &mut Player, authority: &mut Authority) -> Value {
    let snapped =
        match authority.true_surface_m(player.pos_m.x, player.pos_m.z, PLAYER_WIDTH_M / 2.0) {
            Some(y) => {
                player.pos_m.y = y + 0.05;
                player.vel_m = glam::DVec3::ZERO;
                true
            }
            None => false,
        };
    let mut value = pose_json(player, authority);
    value["surface_snapped"] = json!(snapped);
    if !snapped {
        value["surface_error"] =
            json!("no solid ground under the footprint; position left as requested");
    }
    value
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
    // Perf window (journal/0080): the authority/host fixed-cadence tick driven
    // from the client (world.tick + receipt routing). Zero cost without `perf`.
    let receipts = {
        let _perf = crate::perf_span!("host.tick");
        authority.advance(f64::from(time.delta_secs()))
    };
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

/// Occupancy-aware neighbour coverage for the mesher's cross-chunk shell
/// (journal/0057).
///
/// The mesher culls a side face against how much of the adjacent cell the
/// neighbour actually fills, so a boolean [`Authority::is_solid_voxel`] is no
/// longer a sufficient answer at a chunk border: it reported a 3/8 loose top as
/// *fully* covering, and the exposed band of the taller partial beside it was
/// drawn by nobody — sky through the ground. Coverage needs the neighbour's
/// **contents**, which the authority resolves a whole chunk at a time and would
/// be ruinous to re-resolve per border voxel. So this memoizes the resolved grid
/// per chunk: one chunk's entire border query touches at most the six chunks
/// around it, however many thousand voxel queries it makes.
///
/// Borrowing is by `RefCell` because the query is handed to `mesh_chunk` as a
/// `Fn`, and the authority underneath it is `&mut` (it lazily generates the
/// unstreamed neighbour it is asked about — the journal/0017 rule that border
/// faces cull against the AUTHORITY, never the client cache's old wrong world).
pub struct NeighborFill<'a> {
    authority: RefCell<&'a mut Authority>,
    contents: RefCell<HashMap<ChunkPos, Option<dc_core::ContentsGrid>>>,
}

impl<'a> NeighborFill<'a> {
    pub fn new(authority: &'a mut Authority) -> Self {
        Self {
            authority: RefCell::new(authority),
            contents: RefCell::new(HashMap::new()),
        }
    }

    /// Fraction of the cell at world voxel `(x, y, z)` the world fills there —
    /// the same `[0, 1]` coverage `mesh_chunk` computes for its own interior
    /// voxels, so a border culls by identical arithmetic to an interior face.
    pub fn fill(&self, x: i64, y: i64, z: i64) -> f32 {
        let block = self.authority.borrow_mut().block_voxel(x, y, z);
        if !crate::meshing::block_uses_contents(block) {
            // Air, or a block that never carries contents: binary coverage, and
            // no reason to resolve a contents grid at all.
            return crate::meshing::cover_frac(block, None);
        }
        let cp = ChunkPos::from_world_voxel(x, y, z);
        let mut cache = self.contents.borrow_mut();
        let grid = cache.entry(cp).or_insert_with(|| {
            // Perf window (journal/0080): the lazily-generated neighbour contents
            // resolution — the hidden cost of border culling against the
            // authority. Bounded to ≤6 per chunk build (memoized per chunk); it
            // nests under `mesh_chunk`, so it is charged to itself, not the mesh.
            let _perf = crate::perf_span!("neighbor_fill.gen");
            self.authority.borrow().chunk_contents(cp)
        });
        let (lx, ly, lz) = local_voxel(x, y, z);
        let c = grid.as_ref().map(|g| g.get(lx, ly, lz));
        crate::meshing::cover_frac(block, c.as_ref())
    }
}

/// Rebuild meshes for edited chunks (and their affected neighbors).
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource"
)]
pub fn remesh_dirty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<FullbrightMaterialHandle>,
    terrain_mat: Res<TerrainMaterialHandle>,
    fullbright: Res<Fullbright>,
    mut authority: ResMut<Authority>,
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
            // Border faces cull against the AUTHORITY's occupancy (edits
            // included), lazily generating an unstreamed neighbour — never the
            // client cache's old wrong-world S1 fallback (journal/0017).
            let fill = NeighborFill::new(&mut authority);
            let neighbor_fill = |x: i64, y: i64, z: i64| fill.fill(x, y, z);
            // Reuse the chunk's existing render-only contents: an edit changes
            // blocks, not materials (ROADMAP 3c-2), and the mesher's block gate
            // keeps stale contents from a re-typed voxel out of the dither.
            mesh_chunk(
                &loaded.chunk,
                pos,
                vscale.voxel_size_m() as f32,
                &neighbor_fill,
                loaded.contents.as_ref(),
            )
        };
        let loaded = map.loaded.get_mut(&pos).expect("checked above");
        if let Some(entity) = loaded.entity.take() {
            commands.entity(entity).despawn();
        }
        if !mesh_data.is_empty() {
            let (mx, my, mz) = pos.min_voxel();
            let min_m = glam::DVec3::new(mx as f64, my as f64, mz as f64) * vscale.voxel_size_m();
            let mut ent = commands.spawn((
                Mesh3d(meshes.add(to_bevy_mesh(mesh_data))),
                crate::app::ChunkEntity(pos),
                // Spawn already positioned (see streaming.rs on the flash).
                Transform::from_translation(to_render(min_m - origin.0)),
            ));
            if fullbright.0 {
                ent.insert(MeshMaterial3d(material.0.clone()));
            } else {
                ent.insert(MeshMaterial3d(terrain_mat.0.clone()));
            }
            loaded.entity = Some(ent.id());
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::app::LoadedChunk;
    use dc_api::schema::mcp_tool_name;
    use dc_api::{Vec3i, Volume, payload};
    use dc_core::{Block, Chunk, MaterialId};

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
                contents: None,
                entity: None,
            },
        );
        map.loaded.insert(
            ChunkPos::new(1, 0, 0),
            LoadedChunk {
                chunk: Chunk::new(),
                contents: None,
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
                None,
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
            None,
            tx,
        );
        authority.tick_now();
        assert_eq!(rx.try_recv().expect("attach")["ok"], json!(true));
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "scout",
            dc_api::payload::Vec3f::new(9.0, 9.0, 9.0),
            false,
            None,
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

    /// **The untraveled door, travelled.** `vanilla_body_pack()` existed since the
    /// body-plan milestone and only a dc-api unit test called it; the renderer used
    /// compiled-in `biped_plan()`/`biped_clips()`. Now the running game *loads* it.
    ///
    /// The assertions are the re-housing proof from the client side: the pack is
    /// queued at construction and applies at the first tick (like every command),
    /// and what the registry then hands the renderer is `==` to the authored source
    /// it used to call — so the rendered frame is unchanged by construction.
    #[test]
    fn the_vanilla_body_pack_loads_through_the_one_door() {
        // Legacy S1 terrain authority (key 3): no worldgen pregen, so this is a
        // cheap world. The pack load is scale-independent.
        let mut authority = Authority::new(1337, 3);
        assert!(
            authority.world.body_plan("dc:body/biped").is_none(),
            "the pack is SUBMITTED at construction, not applied — it lands at a \
             tick boundary like every other command"
        );
        authority.tick_now();

        // Both plans and the one clip set are now registered.
        assert_eq!(authority.world.body_plans().count(), 2);
        assert_eq!(authority.world.anim_clips().count(), 3);
        // And they are the authored source, byte for byte: this is what makes the
        // registry route a re-housing rather than a change.
        assert_eq!(
            authority
                .world
                .body_plan("dc:body/biped")
                .expect("biped")
                .plan,
            dc_api::bodies::biped_plan(),
            "the plan the renderer reads must equal the one it used to call"
        );
        assert_eq!(
            authority
                .world
                .body_plan("dc:body/stout")
                .expect("stout")
                .plan,
            dc_api::bodies::stout_plan()
        );
        for authored in dc_api::bodies::biped_clips() {
            assert_eq!(
                authority
                    .world
                    .anim_clip(&authored.name)
                    .unwrap_or_else(|| panic!("clip {} registered", authored.name))
                    .clip,
                authored
            );
        }
    }

    /// Per-character plan selection through the attach flow: a named plan is worn
    /// and reads back, and an unregistered name is a **receipt**, not a silent
    /// fallback to the default body.
    #[test]
    fn attach_wears_a_named_plan_and_refuses_an_unknown_one() {
        let mut authority = Authority::new(1337, 3);
        authority.tick_now(); // the pack lands

        // The default: no plan named.
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "plain",
            dc_api::payload::Vec3f::new(0.3, 40.0, 0.3),
            false,
            None,
            tx,
        );
        authority.tick_now();
        assert_eq!(rx.try_recv().expect("attach")["ok"], json!(true));
        assert_eq!(
            authority
                .world
                .character("plain")
                .expect("spawned")
                .body_plan,
            dc_api::bodies::DEFAULT_BODY_PLAN
        );

        // A registered second plan. Same feet as above — the spot is known clear
        // (the embed guard passed for `plain`), and characters do not collide with
        // each other, so this isolates the plan argument as the only variable.
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "squat",
            dc_api::payload::Vec3f::new(0.3, 40.0, 0.3),
            false,
            Some("dc:body/stout".to_string()),
            tx,
        );
        authority.tick_now();
        assert_eq!(rx.try_recv().expect("attach")["ok"], json!(true));
        assert_eq!(
            authority
                .world
                .character("squat")
                .expect("spawned")
                .body_plan,
            "dc:body/stout"
        );

        // An unregistered plan: refused with a receipt, and nothing spawned.
        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "ghost",
            dc_api::payload::Vec3f::new(0.3, 40.0, 0.3),
            false,
            Some("mod:body/nonexistent".to_string()),
            tx,
        );
        let reply = rx
            .try_recv()
            .expect("an unknown plan is refused before any placement work");
        assert_eq!(reply["ok"], json!(false), "{reply}");
        assert_eq!(reply["code"], json!("unknown_body_plan"), "{reply}");
        authority.tick_now();
        assert!(
            authority.world.character("ghost").is_none(),
            "a refused attach spawns nothing"
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
        )
        .expect("the open spawn column has a surface");

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
            None,
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
            None,
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
            None,
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

    /// A sample of chunk positions around the worldgen spawn column (guaranteed
    /// land), spanning the surface (some strata, some air, some deep).
    fn worldgen_sample_positions(a: &mut Authority) -> Vec<ChunkPos> {
        let spawn = a.find_open_spawn();
        let scale = a.scale;
        let scx = scale.voxel_at(spawn.x).div_euclid(32);
        let scz = scale.voxel_at(spawn.z).div_euclid(32);
        let scy = scale.voxel_at(spawn.y).div_euclid(32);
        let mut positions = Vec::new();
        for cz in -2..=2 {
            for cx in -2..=2 {
                for cy in -1..=1 {
                    positions.push(ChunkPos::new(
                        (scx + cx) as i32,
                        (scy + cy) as i32,
                        (scz + cz) as i32,
                    ));
                }
            }
        }
        positions
    }

    /// The build box for a body-sized AABB seated at `feet` over the worldgen
    /// authority, and whether it overlaps solid — the embed check the spawn,
    /// teleport, and attach paths all rely on.
    fn body_embedded(a: &mut Authority, feet: DVec3) -> bool {
        let cfg = *a.world.character_config();
        let vs = cfg.voxel_size_m;
        let body = Aabb::from_bottom_center(feet / vs, cfg.width_m / 2.0 / vs, cfg.height_m / vs);
        let world = RefCell::new(&mut a.world);
        let solid =
            |x: i64, y: i64, z: i64| world.borrow_mut().block_at(Vec3i::new(x, y, z)).is_solid();
        aabb_overlaps_solid(&solid, body)
    }

    /// The regression test walk 12 would have caught (journal/0015–0016): over a
    /// **deep-time (worldgen) world**, [`Authority::true_surface_m`] must return a
    /// point that is genuinely the surface — air at the feet, solid immediately
    /// below — not a plausible y buried inside terrain the pre-deep-time ceiling
    /// could no longer see. The deep-time surface sits ~1000 m up, ~100× above
    /// the S1 estimate the old 8 m headroom was sized against, so a ceiling that
    /// did not track deep time would start the scan below the ground and bury
    /// every seated body.
    #[test]
    fn worldgen_true_surface_is_really_the_surface() {
        let mut a = Authority::new(1337, 2);
        let scale = a.scale;
        let vs = scale.voxel_size_m();
        let half = PLAYER_WIDTH_M / 2.0;
        // The S1 estimate for the same columns: proof deep-time moved the ground
        // enormously far from the field the old headroom assumed.
        let s1 = TerrainGen::new(1337);

        let mut checked = 0usize;
        let mut max_gap_vs_s1 = 0.0f64;
        let mut zi = -6i64;
        while zi <= 6 {
            let mut xi = -6i64;
            while xi <= 6 {
                let (xm, zm) = (xi as f64 * 47.0 + 0.3, zi as f64 * 53.0 + 0.7);
                let analytic = a.analytic_height_m(xm, zm);
                let ts = a
                    .true_surface_m(xm, zm, half)
                    .expect("a worldgen land column has a surface");
                // Genuinely the surface, not the old buried fallback. Two
                // guarantees together pin it: (1) a player-sized body seated at
                // `ts` is NOT embedded — the resting height is clear of solid;
                // (2) `ts` sits in the honest surface band, at or above the
                // column's own deep-time-aware analytic height (never below it).
                // The regression returned `analytic - 96..220 m` — a point ~100 m
                // *inside* the mountain — which this lower bound rejects, while
                // the footprint-max can only push `ts` *up* to a taller
                // neighbour, never down.
                assert!(
                    !body_embedded(&mut a, DVec3::new(xm, ts + 0.05, zm)),
                    "seated body embedded at ({xm:.1}, {zm:.1}), ts = {ts:.2}, analytic = {analytic:.2}"
                );
                assert!(
                    ts >= analytic - vs,
                    "true surface {ts:.2} is buried below the analytic ceiling {analytic:.2} \
                     at ({xm:.1}, {zm:.1}) — the deep-time regression signature"
                );
                let s1h = s1.surface_height_m(xm, zm);
                max_gap_vs_s1 = max_gap_vs_s1.max(ts - s1h);
                checked += 1;
                xi += 1;
            }
            zi += 1;
        }
        assert!(
            checked >= 100,
            "expected a broad column sample, got {checked}"
        );
        // The whole point: deep-time drove the surface hundreds of meters off the
        // S1-scale estimate, so the honest ceiling had to come from the deep-time
        // record. (Seed 1337 sits the worldgen surface ~1000 m up; S1 ~8 m.)
        assert!(
            max_gap_vs_s1 > 100.0,
            "deep-time surface should tower over the S1 estimate; max gap {max_gap_vs_s1:.1} m"
        );
    }

    /// Seating a body over the worldgen authority never embeds it: the player
    /// surface-teleport math across a broad column grid, and a `surface:true`
    /// character attach dropped from far above. Extends the S1-only
    /// `attach_rejects_embedded_allows_clear_and_snaps_to_surface` to the
    /// deep-time world where the regression lived.
    #[test]
    fn worldgen_surface_seating_never_embeds() {
        let mut a = Authority::new(1337, 2);

        // find_open_spawn seats the player 2 m above ground — trivially clear,
        // but the surface it chose must itself be real ground. That surface is
        // the MAX over the columns the body's footprint covers
        // (worldgen.rs::true_surface_m — a body rests on the highest column it
        // straddles, not the one under its navel), so the voxel proving it real
        // is solid under *some* footprint column. Probing only the centre column
        // held by luck until the erodibility flip dropped the origin column one
        // voxel below its neighbours and left the centre probe in air, with the
        // seating perfectly correct (journal/0030).
        let spawn = a.find_open_spawn();
        let half = PLAYER_WIDTH_M / 2.0;
        let on_ground = [
            (spawn.x, spawn.z),
            (spawn.x - half, spawn.z - half),
            (spawn.x - half, spawn.z + half),
            (spawn.x + half, spawn.z - half),
            (spawn.x + half, spawn.z + half),
        ]
        .into_iter()
        .any(|(x, z)| a.is_solid_m(DVec3::new(x, spawn.y - 2.5, z)));
        assert!(on_ground, "open spawn is not above real ground: {spawn:?}");

        // Attach with surface:true from 5 km up lands the body standing on the
        // deep-time ground, not embedded and not at a buried fallback y.
        let (tx, mut rx) = oneshot::channel();
        a.handle_character_attach(
            "walker",
            dc_api::payload::Vec3f::new(spawn.x, 5000.0, spawn.z),
            true,
            None,
            tx,
        );
        a.tick_now();
        let reply = rx.try_recv().expect("surface attach resolves");
        assert_eq!(reply["ok"], json!(true), "{reply}");
        let walker = a.world.character("walker").expect("spawned").clone();
        let feet = DVec3::new(walker.pos_m.x, walker.pos_m.y, walker.pos_m.z);
        assert!(
            !body_embedded(&mut a, feet),
            "worldgen surface-attach landed embedded at {feet:?}"
        );

        // The teleport math over a grid: true_surface + 0.05 never embeds.
        for (xm, zm) in [(0.3, 0.7), (120.0, -90.0), (-260.0, 310.0), (500.0, 500.0)] {
            if let Some(ts) = a.true_surface_m(xm, zm, PLAYER_WIDTH_M / 2.0) {
                assert!(
                    !body_embedded(&mut a, DVec3::new(xm, ts + 0.05, zm)),
                    "teleport seat embedded at ({xm}, {zm}), ts = {ts:.2}"
                );
            }
        }
    }

    /// `eye_in_solid` must read the AUTHORITY, not the client `ChunkMap`'s
    /// unstreamed fallback to the legacy S1 terrain — a *different world* under
    /// the worldgen authority (journal/0015). Deep inside the worldgen ground the
    /// eye is solid even though the S1 field there is pure air, and clear sky
    /// reads not-solid.
    #[test]
    fn eye_in_solid_reads_the_authority_not_stale_s1() {
        let mut a = Authority::new(1337, 2);
        let scale = a.scale;
        // A point well below the worldgen surface (~1000 m) but far above the S1
        // surface (~8 m): the old fallback would call it air.
        let deep = DVec3::new(0.3, 900.0, 0.7);
        let s1 = TerrainGen::new(1337);
        assert_eq!(
            s1.block_at(
                scale,
                scale.voxel_at(deep.x),
                scale.voxel_at(deep.y),
                scale.voxel_at(deep.z)
            ),
            Block::Air,
            "precondition: the stale S1 fallback reports air at 900 m"
        );
        assert!(
            a.is_solid_m(deep),
            "eye_in_solid must see the worldgen ground the S1 fallback misses"
        );
        // Clear sky is honestly not solid.
        assert!(!a.is_solid_m(DVec3::new(0.3, 3000.0, 0.7)));
    }

    /// The S1-fallback-sweep tripwire (journal/0017). With an **empty** chunk
    /// cache — the state right after a teleport, exactly when every gameplay
    /// system asks its solidity question — a known-solid worldgen voxel deep
    /// below the ~1000 m surface must answer *solid* through every public
    /// solidity path a gameplay system reaches: the point query
    /// ([`Authority::is_solid_voxel`]) and the `Fn` closure the collision,
    /// raycast, and physics sites build over it (`dc-core`'s `move_aabb`,
    /// `raycast_voxels`, and the `VoxelQuery` the physics step consumes).
    ///
    /// On pre-sweep `main` these sites consulted `ChunkMap::is_solid`, whose
    /// empty-cache miss fell back to the legacy S1 [`TerrainGen`] (surface
    /// ~8 m), which calls this voxel **air** — the assertions below would fail.
    /// It therefore catches all six routed sites at once: player collision
    /// (`player.rs`), character ground-finding (`character.rs`), the crosshair
    /// edit raycast (`edit.rs`), physics collider tiles (`physdemo.rs`), and
    /// mesh-border culling (`streaming.rs` + `remesh_dirty`).
    #[test]
    fn empty_cache_solidity_paths_read_the_worldgen_authority() {
        use dc_core::{move_aabb, raycast_voxels};

        let mut a = Authority::new(1337, 2);
        let scale = a.scale;
        let vpm = scale.voxels_per_meter();
        // A voxel deep in the worldgen ground (surface ~1000 m). The same point
        // the eye_in_solid tripwire uses — proven solid for this seed.
        let deep_m = DVec3::new(0.3, 900.0, 0.7);
        let (dx, dy, dz) = (
            scale.voxel_at(deep_m.x),
            scale.voxel_at(deep_m.y),
            scale.voxel_at(deep_m.z),
        );

        // The wrong world the old empty-cache fallback consulted: pure air here.
        let s1 = TerrainGen::new(1337);
        assert_eq!(
            s1.block_at(scale, dx, dy, dz),
            Block::Air,
            "precondition: the old ChunkMap-miss fallback (S1) calls this voxel air"
        );

        // 1) The point query every site funnels through.
        assert!(
            a.is_solid_voxel(dx, dy, dz),
            "is_solid_voxel must read the worldgen authority with an empty cache"
        );

        // 2) The `Fn` closure the collision / raycast / physics sites build over
        //    it — no ChunkMap in sight, so it can only be the authority.
        let cell = RefCell::new(&mut a);
        let solid = |x: i64, y: i64, z: i64| cell.borrow_mut().is_solid_voxel(x, y, z);

        assert!(solid(dx, dy, dz), "closure form reads the authority");

        // Raycast straight down from high sky finds the worldgen surface (the S1
        // fallback would only ever hit ~8 m, far outside a ray aimed at 900 m).
        let ray_origin = DVec3::new(
            dx as f64 + 0.5,
            scale.voxel_at(2000.0) as f64 + 0.5,
            dz as f64 + 0.5,
        );
        let hit = raycast_voxels(&solid, ray_origin, DVec3::new(0.0, -1.0, 0.0), 3000.0 * vpm);
        assert!(
            hit.is_some(),
            "a downward ray finds the worldgen ground through the authority"
        );

        // Player-collision path: a body dropped just above the solid voxel is
        // stopped by it (the most serious old defect — no collision at
        // streaming edges).
        let half = PLAYER_WIDTH_M / 2.0 * vpm;
        let aabb = Aabb::from_bottom_center(
            DVec3::new(dx as f64 + 0.5, dy as f64 + 2.0, dz as f64 + 0.5),
            half,
            PLAYER_HEIGHT_M * vpm,
        );
        let result = move_aabb(&solid, aabb, DVec3::new(0.0, -4.0, 0.0));
        assert!(
            result.on_ground && result.hit_y,
            "collision must land on the worldgen ground, not fall through the S1 phantom"
        );
    }

    /// Decision A determinism proof at the SEAM: two independent worldgen
    /// authorities from one seed, generating the same chunk set in OPPOSITE
    /// orders, produce byte-identical chunks. The collapse caches memoize but
    /// never reorder output — the `ChunkGenerator` closure stays a pure
    /// function of `pos`. Also confirms geology blocks actually appear.
    #[test]
    fn worldgen_seam_is_order_independent() {
        const SEED: i32 = 1337;
        let mut a = Authority::new(SEED, 2);
        let positions = worldgen_sample_positions(&mut a);
        let forward: Vec<(ChunkPos, Vec<Block>)> = positions
            .iter()
            .map(|&p| (p, a.world.chunk(p).blocks().to_vec()))
            .collect();

        // A fresh authority, same seed, chunks fetched in reverse order.
        let mut b = Authority::new(SEED, 2);
        for &p in positions.iter().rev() {
            let got = b.world.chunk(p).blocks().to_vec();
            let want = &forward.iter().find(|(q, _)| *q == p).expect("pos").1;
            assert_eq!(&got, want, "chunk {p:?} depends on fetch order");
        }

        let saw_geology = forward.iter().any(|(_, bs)| {
            bs.iter().any(|b| {
                matches!(b, Block::Material(m) if [
                    MaterialId::MUDSTONE,
                    MaterialId::SANDSTONE,
                    MaterialId::GRANITE,
                    MaterialId::BASALT,
                ]
                .contains(m))
            })
        });
        assert!(
            saw_geology,
            "expected geology blocks in the sampled worldgen chunks"
        );
    }

    /// journal/0084: the async-offload follow-on mints a PER-TASK
    /// [`WorldGenerator`] from the shared `Arc<Pregen>` (via
    /// [`Authority::worldgen_pregen`]) and resolves neighbour contents /
    /// far coarse surface through it, off the frame thread. Generation is a pure
    /// function of `(pregen, pos)`, so a per-task generator MUST produce
    /// byte-identical results to the shared one behind the authority's `Mutex` —
    /// that byte-identity is the load-bearing correctness invariant of the whole
    /// offload. Proven here for both offloaded paths (`chunk_contents` and
    /// `coarse_surface`), over surface chunks/columns that carry real data.
    #[test]
    fn per_task_generator_is_byte_identical_to_the_shared_one() {
        const SEED: i32 = 1337;
        let mut authority = Authority::new(SEED, 2);
        let pregen = authority
            .worldgen_pregen()
            .expect("worldgen authority exposes its pregen");
        // A per-task generator, exactly as a mesh task mints it.
        let mut per_task = WorldGenerator::new_owned(pregen.clone());

        // Neighbour contents over the surface sample box: the shared (authority)
        // path vs the per-task generator, byte-for-byte. Non-vacuous: at least one
        // sampled chunk must carry real contents.
        let positions = worldgen_sample_positions(&mut authority);
        let mut saw_contents = false;
        for &p in &positions {
            let shared = authority.chunk_contents(p);
            let task = per_task.chunk_contents(p);
            assert_eq!(shared, task, "per-task chunk_contents differs at {p:?}");
            saw_contents |= shared.is_some();
        }
        assert!(
            saw_contents,
            "no sampled chunk carried contents (test would be vacuous)"
        );

        // Far coarse surface over a spread of columns (the far-derive path).
        for (vx, vz) in [(0i64, 0i64), (1000, -500), (-321, 777), (64, 96)] {
            let shared = authority
                .worldgen_coarse_surface(vx, vz)
                .expect("worldgen coarse surface");
            let task = per_task.coarse_surface(vx, vz);
            assert_eq!(
                shared, task,
                "per-task coarse_surface differs at ({vx},{vz})"
            );
        }
    }

    /// journal/0084 construction-cost check: the offload mints ONE per-task
    /// `WorldGenerator` per mesh task (near: up to `LOAD_BUDGET_PER_FRAME`/frame;
    /// far: `FAR_SURFACE_BUDGET_PER_FRAME`/frame), so minting must be cheap. It is
    /// an `Arc<Pregen>` clone plus `assemble` (build the site index, empty caches)
    /// — O(sites), no world generation. The cost also rides on the TASK thread,
    /// never the frame thread, and is amortized over the many generator queries
    /// each task makes (≤6 cold neighbour contents, or 34² coarse samples). Prints
    /// the mean with `--nocapture`; no hard threshold (machine-dependent).
    #[test]
    fn per_task_generator_construction_is_cheap() {
        use std::hint::black_box;
        use std::time::Instant;
        let pregen = Arc::new(Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        }));
        const N: u32 = 500;
        let start = Instant::now();
        for _ in 0..N {
            black_box(WorldGenerator::new_owned(pregen.clone()));
        }
        let micros = start.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!(
            "per-task WorldGenerator::new_owned mean: {micros:.1} µs over {N} \
             (Medium pregen; rides the task thread, amortized over the task's queries)"
        );
    }

    /// Perf reference (Decision A deliverable): cold-chunk generation through
    /// the worldgen seam, to compare against the S7 headless 0.711 ms mean.
    /// Prints with `--nocapture`; no hard threshold (machine-dependent).
    #[test]
    fn worldgen_seam_cold_chunk_timing() {
        use std::time::Instant;
        const SEED: i32 = 1337;
        let mut a = Authority::new(SEED, 2);
        let positions = worldgen_sample_positions(&mut a);
        // Fresh authority so every fetch is a cold chunk through the seam.
        let mut b = Authority::new(SEED, 2);
        let start = Instant::now();
        for &p in &positions {
            let _ = b.world.chunk(p);
        }
        let elapsed = start.elapsed();
        let mean_ms = elapsed.as_secs_f64() * 1000.0 / positions.len() as f64;
        println!(
            "worldgen seam cold-chunk mean: {mean_ms:.3} ms over {} chunks \
             (S7 headless surface mean: 0.711 ms)",
            positions.len()
        );
    }

    /// Perf reference (S1-fallback sweep, journal/0017): the mesh-border culling
    /// cost when neighbour solidity comes from the AUTHORITY (lazily generating
    /// an unstreamed neighbour through worldgen) vs. the old S1-analytic
    /// fallback. Both passes mesh the SAME worldgen chunks; only the
    /// border-neighbour query differs, so the delta is the sweep's meshing tax.
    /// Prints with `--nocapture`; no hard threshold (machine-dependent).
    #[test]
    fn mesh_border_culling_cost_authority_vs_s1() {
        use crate::meshing::mesh_chunk;
        use std::time::Instant;
        const SEED: i32 = 1337;

        let mut a = Authority::new(SEED, 2);
        let positions = worldgen_sample_positions(&mut a);
        let scale = a.scale;
        let vs = scale.voxel_size_m() as f32;

        // (A) OLD: border faces cull against the legacy S1 analytic field.
        let s1 = TerrainGen::new(SEED);
        let chunks_a: Vec<_> = positions
            .iter()
            .map(|&p| (p, a.world.chunk(p).clone()))
            .collect();
        let start_a = Instant::now();
        for (p, chunk) in &chunks_a {
            let neighbor = |x: i64, y: i64, z: i64| {
                crate::meshing::cover_frac(s1.block_at(scale, x, y, z), None)
            };
            let _ = mesh_chunk(chunk, *p, vs, &neighbor, None);
        }
        let ms_a = start_a.elapsed().as_secs_f64() * 1000.0 / chunks_a.len() as f64;

        // (B) NEW: border faces cull against the authority (lazy worldgen
        // neighbour). A fresh authority so neighbour generation is cold.
        let mut b = Authority::new(SEED, 2);
        let chunks_b: Vec<_> = positions
            .iter()
            .map(|&p| (p, b.world.chunk(p).clone()))
            .collect();
        let fill = NeighborFill::new(&mut b);
        let neighbor = |x: i64, y: i64, z: i64| fill.fill(x, y, z);
        let start_b = Instant::now();
        for (p, chunk) in &chunks_b {
            let _ = mesh_chunk(chunk, *p, vs, &neighbor, None);
        }
        let ms_b = start_b.elapsed().as_secs_f64() * 1000.0 / chunks_b.len() as f64;

        println!(
            "mesh-border culling mean per chunk over {} chunks: S1-analytic (old) {ms_a:.3} ms, \
             authority lazy-neighbour cold (new) {ms_b:.3} ms",
            positions.len()
        );
    }

    /// Decision E: a session disconnect zeroes the character's move intent and
    /// the body comes to rest where it was left (API.md § Characters, DECIDED
    /// 2026-07-19). Here `freeze_character` stands in for the session-teardown
    /// `Drop` (proven to emit the request in mcp_character.rs).
    #[test]
    fn freeze_zeroes_move_intent_and_body_rests() {
        let seed = 1337;
        let mut authority = Authority::new(seed, 3);
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 3);
        let terrain = TerrainGen::new(seed);
        let spawn = crate::app::find_open_spawn(&terrain, scale);

        let (tx, mut rx) = oneshot::channel();
        authority.handle_character_attach(
            "scout",
            dc_api::payload::Vec3f::new(spawn.x, spawn.y + 2.0, spawn.z),
            true,
            None,
            tx,
        );
        authority.tick_now();
        assert_eq!(rx.try_recv().expect("attach")["ok"], json!(true));
        for _ in 0..40 {
            authority.tick_now();
        }

        // Drive it: a persistent walk intent.
        let (tx, _rx) = oneshot::channel();
        authority.handle_character_api(
            "character_set_move_intent",
            &json!({ "character": "scout", "dx": 1.0, "dz": 0.0, "speed": 1.0 }),
            "scout",
            tx,
        );
        for _ in 0..5 {
            authority.tick_now();
        }
        let moving = authority.world.character("scout").expect("exists").clone();
        assert!(
            moving.input.move_dir != (0.0, 0.0) && moving.input.speed > 0.0,
            "precondition: the walk intent is set"
        );

        // Disconnect: freeze. The command applies on the next tick.
        authority.freeze_character("scout");
        authority.tick_now();
        let frozen = authority.world.character("scout").expect("exists").clone();
        assert_eq!(frozen.input.move_dir, (0.0, 0.0), "move dir cleared");
        assert_eq!(frozen.input.speed, 0.0, "speed cleared");

        // And the body is at rest: no horizontal drift over further ticks.
        let x0 = frozen.pos_m.x;
        let z0 = frozen.pos_m.z;
        for _ in 0..4 {
            authority.tick_now();
        }
        let rested = authority.world.character("scout").expect("exists");
        assert!(
            (rested.pos_m.x - x0).abs() < 1e-6 && (rested.pos_m.z - z0).abs() < 1e-6,
            "body kept moving after freeze"
        );
        assert!(rested.vel_m.x.abs() < 1e-9 && rested.vel_m.z.abs() < 1e-9);
    }

    /// Instrument fix (corrections #10, journal/0016): the player pose reply
    /// echoes the feet **voxel** coordinate beside the meters, in the SAME
    /// conversion the authority (and `world_get_block`) uses — so a walker can
    /// cross-check a meters pose against block queries with no mental unit math.
    /// A meters-vs-voxels misread cost a full agent cycle.
    #[test]
    fn pose_reply_echoes_feet_voxel_in_get_block_frame() {
        let mut authority = Authority::new(1337, 3);
        let scale = authority.scale;
        // A non-grid, non-integer pose so floor-division actually matters.
        let player = Player::new(DVec3::new(10.3, 5.7, -3.2));

        let reply = pose_json(&player, &mut authority);
        let (vx, vy, vz) = (
            scale.voxel_at(player.pos_m.x),
            scale.voxel_at(player.pos_m.y),
            scale.voxel_at(player.pos_m.z),
        );
        assert_eq!(reply["pos_voxel"]["x"], json!(vx));
        assert_eq!(reply["pos_voxel"]["y"], json!(vy));
        assert_eq!(reply["pos_voxel"]["z"], json!(vz));

        // The meters pose is still present and unrenamed (append-only).
        assert_eq!(reply["pos"]["x"], json!(player.pos_m.x));

        // The echoed voxel is exactly the `world_get_block` argument for these
        // feet: the point query at pos_voxel and the meters query at pos resolve
        // to the same block, because both go through the one `scale.voxel_at`.
        assert_eq!(
            authority.is_solid_voxel(vx, vy, vz),
            authority.is_solid_m(player.pos_m),
        );
    }

    /// Instrument fix (journal/0018, walk 13): a `surface:true` teleport reply
    /// ALWAYS carries `surface_snapped` — true when the scan seated the feet,
    /// false on a miss (position left as requested). Absence-means-success was
    /// the ambiguity; silence is not a reading.
    #[test]
    fn surface_teleport_reply_reports_snapped_on_success_and_miss() {
        let mut authority = Authority::new(1337, 3);
        let scale = authority.scale;

        // Success: over a real land column the scan seats the feet, and the
        // requested (far-above) y is discarded for the surface.
        let spawn = authority.find_open_spawn();
        let mut player = Player::new(DVec3::new(spawn.x, spawn.y + 100.0, spawn.z));
        let reply = surface_teleport_reply(&mut player, &mut authority);
        assert_eq!(reply["surface_snapped"], json!(true), "{reply}");
        assert!(reply.get("surface_error").is_none());
        assert!(
            (player.pos_m.y - (spawn.y - 2.0)).abs() < 1.0,
            "feet snapped to the surface (~{:.2}), not the requested {:.2}",
            spawn.y - 2.0,
            spawn.y + 100.0,
        );

        // Miss: carve a tall air shaft over the footprint columns — wider and
        // taller than any column's scan window at S1 amplitude, so a cave under
        // the footprint cannot leave a solid voxel the scan still reaches. With
        // nothing to stand on, the teleport reports a miss and leaves the
        // requested position untouched.
        let half = PLAYER_WIDTH_M / 2.0;
        let (cx, cz) = (spawn.x, spawn.z);
        // Absolute bounds that bracket the whole S1 surface band (deepest chasm
        // ~ -90 m, tallest hill ~ +22 m) plus generous slack for the scan depth.
        let y_bot = scale.voxel_at(-220.0);
        let y_top = scale.voxel_at(40.0);
        let x0 = scale.voxel_at(cx - half) - 2;
        let x1 = scale.voxel_at(cx + half) + 2;
        let z0 = scale.voxel_at(cz - half) - 2;
        let z1 = scale.voxel_at(cz + half) + 2;
        authority.submit_player(Payload::Fill(payload::Fill {
            min: Vec3i::new(x0, y_bot, z0),
            max: Vec3i::new(x1, y_top, z1),
            block: "dc:air".into(),
        }));
        authority.tick_now();

        // Self-check: the carve really cleared the footprint's scan column (edits
        // override generated solid). If this trips, the miss below would be a
        // false negative, so localize it here.
        for x in scale.voxel_at(cx - half)..=scale.voxel_at(cx + half) {
            for z in scale.voxel_at(cz - half)..=scale.voxel_at(cz + half) {
                for y in y_bot..=y_top {
                    assert!(
                        !authority.is_solid_voxel(x, y, z),
                        "carve left solid at ({x}, {y}, {z})"
                    );
                }
            }
        }

        let mut player = Player::new(DVec3::new(cx, 500.0, cz));
        let reply = surface_teleport_reply(&mut player, &mut authority);
        assert_eq!(reply["surface_snapped"], json!(false), "{reply}");
        assert!(reply["surface_error"].is_string());
        assert!(
            (player.pos_m.y - 500.0).abs() < 1e-9,
            "a miss leaves the position as requested, got y = {}",
            player.pos_m.y
        );
    }

    /// Far-field authority gating (journal/0022): the worldgen authority (key 2)
    /// answers the coarse far-field summary from its OWN elevation lattice, and
    /// that summary sits ~1 km up with the real terrain (not the S1 phantom ~8 m
    /// down). The S1 authority (keys 3/4) returns `None` from the summary path —
    /// the worldgen far field NEVER touches `TerrainGen`, and the S1 far field
    /// keeps its own S1 mesh. This is the tripwire the doctrine rests on: a
    /// worldgen far horizon that read the S1 generator would answer near y=0 here.
    #[test]
    fn far_field_summary_gates_on_authority() {
        let seed = 1337;

        // Worldgen authority: summary present, and it is the ~1 km-up real world.
        let mut worldgen = Authority::new(seed, 2);
        assert!(worldgen.far_field_is_worldgen());
        let scale = VoxelScale::from_player_height(PLAYER_HEIGHT_M, 2);
        let spawn = worldgen.find_open_spawn(); // a worldgen land column
        let (vx, vz) = (scale.voxel_at(spawn.x), scale.voxel_at(spawn.z));
        let (h, _block) = worldgen
            .worldgen_coarse_surface(vx, vz)
            .expect("worldgen authority answers its own far-field summary");
        // The worldgen surface is high above the S1 ~8 m surface: a summary that
        // secretly read TerrainGen would land near y=0, not up near the terrain.
        assert!(
            h > 200,
            "worldgen far summary height {h} voxels should be far above the S1 phantom"
        );

        // S1 authority: no worldgen summary — the S1 far mesh owns keys 3/4, and
        // the worldgen summary path never invokes the S1 generator.
        let s1 = Authority::new(seed, 3);
        assert!(!s1.far_field_is_worldgen());
        assert!(s1.worldgen_coarse_surface(vx, vz).is_none());
    }
}
