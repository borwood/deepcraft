//! The lazy collapse pyramid below region scale: region → locale →
//! chunk-column → chunk, ending in a dc-core [`Chunk`] of [`Block`]s at the
//! N=2 (0.9 m) scale.
//!
//! One rule everywhere (docs/design/worldgen.md):
//!
//! > collapsed(cell) = f(base(cell), summary of its 1-ring neighbours'
//! > **base** states, collapsed(parent))
//!
//! Two instantiations of the rule live here:
//!
//! - **Semantic records** ([`RegionRec`], [`LocaleRec`], [`ColumnRec`]):
//!   rivers, sites/ruins, the civilized-fringe flag. Each record is built
//!   from its own base state, a summary over its 1-ring neighbours' base
//!   states, and its collapsed parent (regions parent to pregen cells).
//! - **The elevation lattice**: midpoint-displacement refinement over
//!   `(elevation, roughness)` lattice points, 15 binary levels from coarse
//!   cell corners (level 0, 16 384-voxel spacing) to voxels (level 14).
//!   Midpoints are the purest form of the rule — parent values plus a
//!   bounded, addressed jitter — and give elevation continuity for free:
//!   adjacent columns share lattice ancestors, so there are no seams to
//!   stitch.
//!
//! **Bounded lookahead is instrumented and asserted.** Every
//! [`WorldGenerator::generate_chunk`] traces the distinct cells consulted per
//! level and asserts them against [`LOOKAHEAD_BOUNDS`] — constants
//! independent of position, extent, and cache warmth.
//!
//! **Border wilds**: outside the pregen grid the pyramid keeps running on
//! synthesized cell views ([`Pregen::cell_view`]) — no history, hostile
//! parameters, forever. There is no separate wilds code path below the cell
//! level; that is the point.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE, CLASS_IGNEOUS_INTRUSIVE,
    FormationContext, GeoMemberIdx, GeologySet,
};
use dc_core::{
    Block, CHUNK_VOLUME, Chunk, ChunkPos, ContentsGrid, MaterialChunk, MixtureId, MixtureTable,
    StructureShape, VoxelContents, VoxelScale, block_twin, classify,
};
use dc_sim::statistical::rng::draw_f64;

use crate::fill::{ColumnFill, Plan, allocate, fill_draw, mixed_contents};
use crate::geology::{
    StrataCtx, StrataEvent, StrataRec, deep_class, dithered_member, interp_select_draw,
};
use crate::pipeline::PipelineError;
use crate::pregen::{
    CELL_VOXELS, Pregen, Provenance, SALT_ELEV, SALT_GEO_SELECT, SALT_RUIN, temp_sea_level,
};

/// Lattice level whose spacing is one region (8 192 voxels, 7.37 km).
pub const L_REGION: u8 = 1;
/// Lattice level whose spacing is one locale (512 voxels, 460.8 m).
pub const L_LOCALE: u8 = 5;
/// Lattice level whose spacing is one chunk-column (32 voxels, 28.8 m).
pub const L_COLUMN: u8 = 9;
/// Finest lattice level: one voxel (0.9 m).
pub const L_VOXEL: u8 = 14;
/// Lattice level at which the deep-time surface drives elevation (3e-1): the
/// locale scale (512 voxels ≈ 460.8 m) ≈ the deep tier's own 460 m cell. At and
/// above this scale the macro-terrain is the eroded deep-time surface (bilinear,
/// continuous); finer levels keep the addressed midpoint jitter for sub-460 m
/// relief, so no sub-locale detail is lost and the deep terrain drives the rest.
pub const L_DEEP: u8 = L_LOCALE;

/// Per-refinement amplitude decay for elevation jitter. Public so the S13
/// roughness probe reads the shipped constant instead of restating it.
pub const AMP_DECAY: f64 = 0.55;

/// Hard per-chunk lookahead bounds (distinct cells consulted per level while
/// generating one chunk). Constants by design: if generation ever needs more,
/// the pyramid has grown unbounded lookahead and the assert fires.
pub struct LookaheadBounds {
    pub cells: usize,
    pub regions: usize,
    pub locales: usize,
    pub columns: usize,
    pub lattice_points: usize,
}

pub const LOOKAHEAD_BOUNDS: LookaheadBounds = LookaheadBounds {
    cells: 96,
    regions: 40,
    locales: 24,
    columns: 2,
    lattice_points: 4200,
};

/// What one `generate_chunk` call actually consulted.
#[derive(Debug, Clone, Default)]
pub struct ChunkStats {
    pub cells: usize,
    pub regions: usize,
    pub locales: usize,
    pub columns: usize,
    pub lattice_points: usize,
    /// Distinct lattice points per refinement level (0 = cell corners).
    pub lattice_per_level: [usize; 15],
}

#[derive(Default)]
struct Trace {
    cells: HashSet<(i64, i64)>,
    regions: HashSet<(i64, i64)>,
    locales: HashSet<(i64, i64)>,
    columns: HashSet<(i64, i64)>,
    lattice: HashSet<(u8, i64, i64)>,
}

impl Trace {
    fn clear(&mut self) {
        self.cells.clear();
        self.regions.clear();
        self.locales.clear();
        self.columns.clear();
        self.lattice.clear();
    }

    fn stats(&self) -> ChunkStats {
        let mut per_level = [0usize; 15];
        for &(l, _, _) in &self.lattice {
            per_level[usize::from(l)] += 1;
        }
        ChunkStats {
            cells: self.cells.len(),
            regions: self.regions.len(),
            locales: self.locales.len(),
            columns: self.columns.len(),
            lattice_points: self.lattice.len(),
            lattice_per_level: per_level,
        }
    }
}

/// A planned river edge in voxel coordinates (cell centre to cell centre),
/// with water-surface elevations at both ends.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RiverSeg {
    ax: f64,
    az: f64,
    bx: f64,
    bz: f64,
    width: f64,
    wa: f64,
    wb: f64,
}

/// Bank falloff distance beyond the channel width (voxels).
const BANK: f64 = 16.0;
/// Maximum horizontal reach of a river's influence (voxels).
const RIVER_REACH: f64 = 40.0 + BANK;

/// A settlement footprint the lazy layer can see (position + fate).
#[derive(Debug, Clone, Copy, PartialEq)]
struct SiteSpot {
    id: u32,
    x: i64,
    z: i64,
    abandoned: bool,
}

/// Base state of a region: a pure function of (seed, coords, parent cell
/// neighbourhood). No recursion into other regions.
struct RegionBase {
    civilized: bool,
    segs: Vec<RiverSeg>,
    site: Option<SiteSpot>,
}

/// Collapsed region: base + 1-ring base summary + parent (pregen cell).
pub struct RegionRec {
    /// Civilized region bordering the wilds (from the 1-ring base summary).
    fringe: bool,
    segs: Vec<RiverSeg>,
    site: Option<SiteSpot>,
}

struct LocaleBase {
    fringe: bool,
    segs: Vec<RiverSeg>,
    sites: Vec<SiteSpot>,
}

/// Collapsed locale: base + 1-ring base summary (neighbouring locales'
/// sites, whose footprints may cross the boundary) + parent region.
pub struct LocaleRec {
    fringe: bool,
    segs: Vec<RiverSeg>,
    sites: Vec<SiteSpot>,
}

/// Collapsed chunk-column: the 32×32 voxel-column surface of one chunk
/// footprint — heights, surface blocks, soil depth, ruin posts.
pub struct ColumnRec {
    /// Surface voxel y per voxel column, indexed `z * 32 + x`.
    pub heights: Vec<i32>,
    pub surface: Vec<Block>,
    /// Regolith band depth, whole voxels — the deep sim's carried `H` plane
    /// quantized by [`regolith_voxels`] (or, in the wilds only, the genesis
    /// precip rule). Applies where no strata pass deposited (ocean, wilds);
    /// **may be 0**, which is a column with bedrock at the surface.
    pub soil: u8,
    /// Ruin posts: (local x, local z, post height in voxels).
    pub posts: Vec<(u8, u8, u8)>,
    /// True when generated beyond the pregen grid (border wilds).
    pub wilds: bool,
    /// The ordered deposition log (geology strata passes). Empty where no
    /// pass deposited (ocean, wilds): the legacy soil band applies there.
    pub strata: StrataRec,
    /// **The surface voxel's fill** (journal/0055), per voxel column: the member
    /// the record skins this column with and how many eighths of the surface
    /// voxel the ground actually occupies (`ceil((elev − h·0.9) / 0.9 · 8)`,
    /// at least 1 — the top-of-column remainder, expressed at last).
    ///
    /// `None` where a fallback applies and the surface voxel carries no contents:
    /// the border wilds, subaqueous columns, and columns with neither a record
    /// nor an igneous province. Those are the remainder of the fill contract's
    /// absent-contents exception, and the list only shrinks.
    pub surface_fill: Vec<Option<(GeoMemberIdx, u8)>>,
}

/// One evaluation of the shared surface kernel ([`WorldGenerator::surface_sample`]).
pub struct SurfaceSample {
    /// Surface voxel y.
    pub h: i32,
    /// The continuous surface elevation, metres — what `h` floors.
    pub elev_m: f64,
    /// The block, from the record where one exists (see the kernel's docs).
    pub block: Block,
    /// The content class the record skins this column with; `None` under a
    /// fallback.
    pub class: Option<&'static str>,
}

/// How a [`WorldGenerator`] holds its pregen output: `Borrowed` (the original
/// borrowing API — tests and headless callers keep an owned `Pregen` on the
/// stack) or `Owned` (an `Arc<Pregen>`, so the generator is `'static` and can
/// live inside the client's `Send + Sync` `ChunkGenerator` seam behind a
/// `Mutex`). Both deref to the same `Pregen`, so every `self.pregen.…` reads
/// identically and generation is byte-for-byte the same regardless of which
/// form built it.
pub enum PregenSource<'a> {
    Borrowed(&'a Pregen),
    Owned(Arc<Pregen>),
}

impl std::ops::Deref for PregenSource<'_> {
    type Target = Pregen;
    fn deref(&self) -> &Pregen {
        match self {
            PregenSource::Borrowed(p) => p,
            PregenSource::Owned(p) => p,
        }
    }
}

/// The lazy generator: owns the collapse caches and instrumentation, borrows
/// (or `Arc`-owns) the pregen output. All state is derived — dropping a cache
/// entry can never change any answer (everything is a pure function of seed +
/// pregen). The caches are `Arc` (not `Rc`) so a `Mutex<WorldGenerator>` is
/// `Send`, which the client's chunk seam requires; nothing about the atomics
/// affects determinism.
pub struct WorldGenerator<'a> {
    pregen: PregenSource<'a>,
    seed: u64,
    voxel_m: f64,
    /// The registered geology content the strata passes select from.
    geology: GeologySet,
    /// Region-scale mixture intern table (S8 `materials/mixtures-v0` path);
    /// ids are first-intern order, deterministic given generation order.
    materials: MixtureTable,
    sites_by_cell: HashMap<(i32, i32), Vec<SiteSpot>>,
    lattice_memo: HashMap<(u8, i64, i64), (f64, f64)>,
    region_cache: HashMap<(i64, i64), Arc<RegionRec>>,
    locale_cache: HashMap<(i64, i64), Arc<LocaleRec>>,
    column_cache: HashMap<(i64, i64), Arc<ColumnRec>>,
    trace: Trace,
    last_stats: ChunkStats,
}

impl<'a> WorldGenerator<'a> {
    pub fn new(pregen: &'a Pregen) -> Self {
        Self::with_geology(pregen, dc_core::materials::geology::vanilla())
    }

    /// A generator over an explicit geology content set (registered class
    /// members). `new` uses the vanilla set. Panics if the set leaves a class
    /// a registered pass selects from empty — see [`Self::try_with_geology`]
    /// for the fallible, named-culprit form.
    pub fn with_geology(pregen: &'a Pregen, geology: GeologySet) -> Self {
        Self::build(PregenSource::Borrowed(pregen), geology)
    }

    /// [`Self::with_geology`], but refuses to build (naming pass and class)
    /// when the content set leaves any selected class empty — the
    /// world-build-time half of the class-satisfiability enforcement
    /// (geology.md § unfilled slots).
    pub fn try_with_geology(
        pregen: &'a Pregen,
        geology: GeologySet,
    ) -> Result<Self, PipelineError> {
        Self::build_checked(PregenSource::Borrowed(pregen), geology)
    }
}

impl WorldGenerator<'static> {
    /// A `'static`, `Arc`-owning generator: the same generator, but it carries
    /// its `Pregen` by `Arc` instead of borrowing it, so a `Mutex` around it is
    /// `Send + Sync` and it can back the client's `ChunkGenerator` closure. The
    /// vanilla geology set.
    pub fn new_owned(pregen: Arc<Pregen>) -> Self {
        Self::with_geology_owned(pregen, dc_core::materials::geology::vanilla())
    }

    /// [`Self::new_owned`] over an explicit geology content set.
    pub fn with_geology_owned(pregen: Arc<Pregen>, geology: GeologySet) -> Self {
        Self::build(PregenSource::Owned(pregen), geology)
    }

    /// The fallible, named-culprit form of [`Self::with_geology_owned`].
    pub fn try_with_geology_owned(
        pregen: Arc<Pregen>,
        geology: GeologySet,
    ) -> Result<Self, PipelineError> {
        Self::build_checked(PregenSource::Owned(pregen), geology)
    }
}

impl<'a> WorldGenerator<'a> {
    fn build(pregen: PregenSource<'a>, geology: GeologySet) -> Self {
        Self::build_checked(pregen, geology)
            .expect("registered geology satisfies the pass graph (vanilla is complete)")
    }

    fn build_checked(pregen: PregenSource<'a>, geology: GeologySet) -> Result<Self, PipelineError> {
        // Class-satisfiability enforcement (geology.md § unfilled slots): a
        // world must not build if a pass selects from an empty class.
        pregen.pipeline.check_class_satisfiability(&geology)?;
        Ok(Self::assemble(pregen, geology))
    }

    fn assemble(pregen: PregenSource<'a>, geology: GeologySet) -> Self {
        let scale = VoxelScale::from_player_height(1.8, 2); // N=2: 0.9 m voxels
        let mut sites_by_cell: HashMap<(i32, i32), Vec<SiteSpot>> = HashMap::new();
        for s in &pregen.sites {
            sites_by_cell.entry(s.cell).or_default().push(SiteSpot {
                id: s.slot,
                x: s.pos.0,
                z: s.pos.1,
                abandoned: s.abandoned.is_some(),
            });
        }
        for spots in sites_by_cell.values_mut() {
            spots.sort_by_key(|s| s.id);
        }
        let seed = pregen.seed;
        Self {
            pregen,
            seed,
            voxel_m: scale.voxel_size_m(),
            geology,
            materials: MixtureTable::new(),
            sites_by_cell,
            lattice_memo: HashMap::new(),
            region_cache: HashMap::new(),
            locale_cache: HashMap::new(),
            column_cache: HashMap::new(),
            trace: Trace::default(),
            last_stats: ChunkStats::default(),
        }
    }

    /// Instrumentation for the chunk most recently generated.
    pub fn last_stats(&self) -> &ChunkStats {
        &self.last_stats
    }

    /// The region-scale mixture intern table accumulated by
    /// [`Self::generate_chunk_with_materials`] (the S8 `materials/mixtures-v0`
    /// sidecar payload is `mixture_table().encode()`).
    pub fn mixture_table(&self) -> &MixtureTable {
        &self.materials
    }

    /// Generate one 32³ chunk of blocks. Deterministic in
    /// `(seed, extent, pos)`; asserts [`LOOKAHEAD_BOUNDS`].
    pub fn generate_chunk(&mut self, pos: ChunkPos) -> Chunk {
        self.trace.clear();
        let (cx, cz) = (i64::from(pos.x), i64::from(pos.z));
        let col = self.column(cx, cz);
        // The recorded column sliced into voxel spans (crate::fill) — the SAME
        // slicing the material path uses, so the two cannot disagree about where
        // the record is or what is in it.
        let fill = ColumnFill::build(&col.strata, self.voxel_m);
        // Per-event block, for spans a single event covers. Provably constant
        // across the chunk footprint: the per-voxel-column member dither
        // re-selects only inside the event's own content class, and every member
        // of a class shares a block twin.
        let event_blocks = self.event_blocks(&col.strata);
        let mut chunk = Chunk::new();
        let base_y = i64::from(pos.y) * 32;
        for z in 0..32usize {
            for x in 0..32usize {
                let i = z * 32 + x;
                let h = i64::from(col.heights[i]);
                let (vx, vz) = (cx * 32 + x as i64, cz * 32 + z as i64);
                for y in 0..32usize {
                    let vy = base_y + y as i64;
                    let b = if vy > h {
                        Block::Air
                    } else if vy == h {
                        col.surface[i]
                    } else if event_blocks.is_empty() {
                        // No deposition record (ocean, wilds): legacy soil.
                        if vy >= h - i64::from(col.soil) {
                            Block::Dirt
                        } else {
                            Block::Stone
                        }
                    } else {
                        // The record fills the column; below it is unrecorded
                        // basement = stone. A mixed voxel's block is
                        // `classify(contents)` of the very contents the material
                        // path builds — one opinion per voxel, journal/0052.
                        let depth = (h - vy) as u32;
                        match fill.plan(depth) {
                            None => Block::Stone,
                            Some(Plan::Single(k)) => event_blocks[*k],
                            Some(Plan::Mixed(w)) => {
                                classify(&self.mixed_at(&col.strata, w, vx, vy, vz))
                            }
                        }
                    };
                    if b != Block::Air {
                        chunk.set(x, y, z, b);
                    }
                }
            }
        }
        for &(px, pz, ph) in &col.posts {
            let i = usize::from(pz) * 32 + usize::from(px);
            let h = i64::from(col.heights[i]);
            for dy in 1..=i64::from(ph) {
                let vy = h + dy;
                if vy >= base_y && vy < base_y + 32 {
                    chunk.set(
                        usize::from(px),
                        (vy - base_y) as usize,
                        usize::from(pz),
                        Block::Wood,
                    );
                }
            }
        }
        self.last_stats = self.trace.stats();
        self.assert_bounds();
        self.evict();
        chunk
    }

    /// Generate a chunk plus its material contents: every buried voxel inside
    /// the recorded strata resolves to a canonical [`VoxelContents`]
    /// (constructor-only — never hand-assembled slots), interned through the
    /// region [`MixtureTable`] and palette-compressed as a [`MaterialChunk`]
    /// (the decided `materials/slots-v0` / `mixtures-v0` sidecar path).
    /// Chunks with no recorded strata in range return an all-empty material
    /// chunk, which attaches no sidecar and pays nothing.
    pub fn generate_chunk_with_materials(&mut self, pos: ChunkPos) -> (Chunk, MaterialChunk) {
        let chunk = self.generate_chunk(pos);
        let dense = self.material_ids(pos);
        (chunk, MaterialChunk::from_dense(&dense))
    }

    /// The material half of [`Self::generate_chunk_with_materials`]: one
    /// interned [`MixtureId`] per voxel (dense, [`Chunk::index`] order). Every
    /// buried voxel inside the recorded strata resolves to a canonical
    /// [`VoxelContents`] (constructor-only — never hand-assembled slots),
    /// interned through the region [`MixtureTable`]. Shared by the storage path
    /// (above) and the render path ([`Self::chunk_contents`]); reuses the
    /// per-column cache, so calling it after `generate_chunk` costs only the
    /// per-voxel classification, not another column collapse.
    fn material_ids(&mut self, pos: ChunkPos) -> Vec<MixtureId> {
        let (cx, cz) = (i64::from(pos.x), i64::from(pos.z));
        let col = self.column(cx, cz);
        let fill = ColumnFill::build(&col.strata, self.voxel_m);
        let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
        let base_y = i64::from(pos.y) * 32;
        // **The surface voxel carries contents now** (journal/0055), which
        // shrinks the fill contract's absent-contents exception: it used to be
        // skipped outright (`vy >= h`) and painted by a climate threshold.
        {
            let mut memo: HashMap<(GeoMemberIdx, u8), MixtureId> = HashMap::new();
            for z in 0..32usize {
                for x in 0..32usize {
                    let i = z * 32 + x;
                    let vy = i64::from(col.heights[i]);
                    let y = vy - base_y;
                    if !(0..32).contains(&y) {
                        continue;
                    }
                    let Some(part) = col.surface_fill[i] else {
                        continue; // fallback column: no record to skin it with
                    };
                    let id = *memo.entry(part).or_insert_with(|| {
                        self.materials
                            .intern(mixed_contents(&self.geology, &[part]))
                    });
                    dense[Chunk::index(x, y as usize, z)] = id;
                }
            }
        }
        if fill.depth_count() > 0 {
            // Memoize per (event, resolved host member): the boundary dither
            // re-selects the host member per voxel-column, so the intern key is
            // the pair, not the event alone (one intern per distinct mixture in
            // this chunk — still bounded, and interning is idempotent).
            let mut memo: HashMap<(usize, GeoMemberIdx), MixtureId> = HashMap::new();
            for z in 0..32usize {
                for x in 0..32usize {
                    let h = i64::from(col.heights[z * 32 + x]);
                    let (vx, vz) = (cx * 32 + x as i64, cz * 32 + z as i64);
                    for y in 0..32usize {
                        let vy = base_y + y as i64;
                        if vy >= h {
                            continue; // air, and the surface voxel done above
                        }
                        let depth = (h - vy) as u32;
                        let id = match fill.plan(depth) {
                            None => continue, // unrecorded basement
                            Some(Plan::Single(k)) => {
                                let k = *k;
                                let event = col.strata.events[k];
                                // Per-voxel-column host member (family contacts
                                // wander off the chunk grid); ore/accessory stay
                                // per-event.
                                let host =
                                    dithered_member(&self.geology, self.seed, &event, cx, cz, x, z);
                                *memo.entry((k, host)).or_insert_with(|| {
                                    self.materials.intern(contents_for_event(
                                        &self.geology,
                                        host,
                                        &event,
                                    ))
                                })
                            }
                            Some(Plan::Mixed(w)) => {
                                let c = self.mixed_at(&col.strata, w, vx, vy, vz);
                                self.materials.intern(c)
                            }
                        };
                        dense[Chunk::index(x, y, z)] = id;
                    }
                }
            }
        }
        dense
    }

    /// The block each recorded event summarizes to, indexed by event. Empty for
    /// a column with no record (ocean, wilds), which is what both fill paths
    /// test to fall back to the legacy soil band.
    fn event_blocks(&self, strata: &StrataRec) -> Vec<Block> {
        strata
            .events
            .iter()
            .map(|e| classify(&contents_for_event(&self.geology, e.member, e)))
            .collect()
    }

    /// Contents of one **mixed** voxel: the addressed stochastic allocation of
    /// eight eighths among the events overlapping its 0.9 m span.
    ///
    /// **The canonical (undithered) member is used here, deliberately.** The
    /// per-voxel-column dither exists to walk a family contact off the chunk
    /// grid; a mixed voxel already has its own position-addressed draw doing
    /// that job, and re-selecting a member for every one of up to eight
    /// candidates per voxel would cost more than the whole rest of the fill.
    /// Keeping it out has a second, load-bearing effect: because the dither only
    /// re-picks *within* a class, running it here could flip which material id
    /// wins `classify`'s tie-break between two classes — which would put the
    /// block and the contents into disagreement, the one thing the fill contract
    /// forbids.
    fn mixed_at(
        &self,
        strata: &StrataRec,
        weights: &[(usize, u64)],
        vx: i64,
        vy: i64,
        vz: i64,
    ) -> VoxelContents {
        let u = fill_draw(self.seed, vx, vy, vz);
        let mut parts: Vec<(GeoMemberIdx, u8)> = Vec::with_capacity(weights.len() + 1);
        for (k, n) in allocate(weights, u) {
            let e = &strata.events[k];
            // A placer enrichment substitutes into its host's OWN eighths — the
            // same substitution `contents_for_event` does, scaled to whatever
            // share of this voxel the host event won. Without it a fluvial fan
            // thin enough to be a mixed voxel (which, since the veneer's residue
            // went to zero, is most of them) would pan no gold at all.
            match e.ore {
                Some((ore, k8)) if k8 > 0 => {
                    let g = k8.min(n);
                    if n > g {
                        parts.push((e.member, n - g));
                    }
                    parts.push((ore, g));
                }
                _ => parts.push((e.member, n)),
            }
        }
        mixed_contents(&self.geology, &parts)
    }

    /// The **render-only** material view of a chunk: its per-voxel canonical
    /// [`VoxelContents`], resolved into a [`ContentsGrid`] so no order-dependent
    /// [`MixtureId`] escapes the generator (the MixtureTable-id landmine,
    /// journal/0007–0008). `None` when the chunk has no recorded strata in range
    /// — a debris-free chunk, which carries no render data and costs nothing
    /// downstream, exactly as it attaches no storage sidecar. This is the seam's
    /// parallel path (docs/design/visuals.md § Mixture rendering road, 3c-2): the
    /// client fetches blocks through the `HostWorld` closure and contents through
    /// this method on the same `Arc<Mutex<WorldGenerator>>`; the interning it
    /// does is render-only and never reaches sim state, receipts, or replay.
    pub fn chunk_contents(&mut self, pos: ChunkPos) -> Option<ContentsGrid> {
        let dense = self.material_ids(pos);
        let mc = MaterialChunk::from_dense(&dense);
        if mc.is_all_empty() {
            return None;
        }
        mc.resolve_contents(&self.materials)
    }

    /// The chunk-column record for chunk coordinates `(cx, cz)` — exposed for
    /// the seam/continuity tests (which reason about surface heights) and the
    /// client's surface-scan ceiling over the worldgen authority.
    pub fn column_record(&mut self, cx: i64, cz: i64) -> Arc<ColumnRec> {
        let rec = self.column(cx, cz);
        self.evict();
        rec
    }

    /// Surface height (voxels) and surface block at ONE world voxel column,
    /// given its locale's river segments + fringe flag and its climate. The
    /// per-column kernel shared by the full [`Self::column`] collapse and the
    /// coarse far-field summary ([`Self::coarse_surface`]) — so the distant
    /// horizon and the ground underfoot are the SAME surface function sampled at
    /// different strides. Height is independent of climate, so a far sample
    /// lands on *exactly* the near column's height where the two coincide
    /// (journal/0022).
    ///
    /// **Since journal/0055 the record decides what the world is skinned with**
    /// (materials.md § Sequencing AMENDED 2026-07-21, user). The year-zero
    /// climate thresholds that painted Grass/Dirt/Stone — stubs.md § 2, the
    /// cause of the razor-straight grass/dirt frontier — are gone wherever a
    /// deep-time record exists: the surface block is the *content class the
    /// record's topmost 0.9 m is made of*. **Grass is not expressed at all**
    /// (ratification 4: it is an ecology state riding on substrate materials,
    /// not a block identity chosen by a threshold), so the fallback vocabulary
    /// is Dirt/Stone only.
    ///
    /// The consult lives **here**, in the shared kernel, so `coarse_surface`
    /// inherits it structurally: deriving it only in [`Self::column`] would turn
    /// the ground sandstone-and-mudstone while the horizon stayed painted, and
    /// the LOD boundary would become a visible lie.
    fn surface_sample(
        &mut self,
        vx: i64,
        vz: i64,
        segs: &[RiverSeg],
        fringe: bool,
        temp_sl: f64,
        precip: f64,
    ) -> SurfaceSample {
        let (raw, _) = self.lattice(L_VOXEL, vx, vz);
        let (elev, riverbed) = carve_rivers(raw, vx as f64, vz as f64, segs);
        let h = (elev / self.voxel_m).floor() as i32;
        let class = if elev > 0.0 {
            self.surface_class(vx, vz)
        } else {
            None
        };
        if let Some(class) = class
            && let Some(block) = self.class_block(class)
        {
            return SurfaceSample {
                h,
                elev_m: elev,
                block,
                class: Some(class),
            };
        }
        // ---- fallbacks, each legitimate by absence of a record ----------
        // Subaqueous columns (`clastic_pass` returns early below sea level),
        // the border wilds (no deep-time run exists out there — stubs.md
        // § Genesis), and the frozen/abyssal cases. Grass is gone: `bare` no
        // longer selects between Dirt and Grass, only the frozen threshold
        // still speaks, and everything else is Dirt.
        let t = temp_sl - 6.5 * elev.max(0.0) / 1000.0;
        let _ = (riverbed, fringe, precip);
        let block = if elev <= -1.0 {
            if elev > -35.0 {
                Block::Dirt
            } else {
                Block::Stone
            }
        } else if t < -4.0 {
            Block::Stone
        } else {
            Block::Dirt
        };
        SurfaceSample {
            h,
            elev_m: elev,
            block,
            class: None,
        }
    }

    /// **What the record says this column is made of at the surface**: the
    /// content class holding the most metres in the recorded column's topmost
    /// 0.9 m. `None` where there is nothing to read.
    ///
    /// Metres, not units — the same `round Σ` discipline the rest of the slice
    /// installs. A voxel whose top 0.9 m is 40 laminae of silt and 3 of sand is
    /// skinned by the silt, regardless of how the bed *count* falls.
    ///
    /// When the record runs out before half a voxel (the 0.2 % bare-rock case
    /// journal/0053 bought), the surface voxel is basement, and the class is the
    /// one [`crate::geology::igneous_pass`] would emplace for the column's
    /// tectonic province — the same rule, so near and far and the buried record
    /// all name the same rock. With no igneous province either, there is no
    /// record to read and the fallback applies.
    fn surface_class(&mut self, vx: i64, vz: i64) -> Option<&'static str> {
        let voxel_m = self.voxel_m;
        let mut acc = 0.0f64;
        let mut best: Option<(&'static str, f64)> = None;
        if let Some(rec) = self.pregen.deep.record_at_voxel(vx, vz) {
            let mut by_class: Vec<(&'static str, f64)> = Vec::new();
            for u in rec.units.iter().rev() {
                if acc >= voxel_m {
                    break;
                }
                let take = u.thickness_m.min(voxel_m - acc);
                if take <= 0.0 {
                    continue;
                }
                acc += take;
                let c = deep_class(u.tag);
                match by_class.iter_mut().find(|(k, _)| *k == c) {
                    Some((_, m)) => *m += take,
                    None => by_class.push((c, take)),
                }
            }
            // Dominant by metres; ties break on the class id so the answer is
            // independent of the order the recorder happened to lay them.
            for (c, m) in by_class {
                if best.is_none_or(|(bc, bm)| m > bm || (m == bm && c < bc)) {
                    best = Some((c, m));
                }
            }
        }
        if acc >= voxel_m / 2.0 {
            return best.map(|(c, _)| c);
        }
        // Bare rock: basement, named the way the igneous pass names it.
        let (gx, gy) = self.pregen.grid.cell_of_voxel(vx, vz);
        let provenance = match (i32::try_from(gx), i32::try_from(gy)) {
            (Ok(x), Ok(y)) => self.pregen.grid.get(x, y).map(|c| c.provenance)?,
            _ => return None,
        };
        match provenance {
            Provenance::Orogeny | Provenance::Arc => Some(CLASS_IGNEOUS_INTRUSIVE),
            Provenance::Rift => Some(CLASS_IGNEOUS_EXTRUSIVE),
            _ => None,
        }
    }

    /// The block a content class summarizes to, via its first (id-sorted)
    /// member. Every member of a vanilla class shares a block twin — that is the
    /// property `classify` relies on everywhere else in the fill — so this
    /// answers the same block the near path's `classify(contents)` will, whichever
    /// member the column's own selection draw picks.
    fn class_block(&self, class: &str) -> Option<Block> {
        let m = *self.geology.class(class)?.members().first()?;
        Some(block_twin(self.geology.member(m).material))
    }

    /// The coarse far-field summary at one world voxel column: surface height
    /// (voxels, N=2 base scale) and surface block, sampled from the SAME
    /// elevation lattice + river carving + surface rule the near-field
    /// [`Self::column`] collapses from. A far heightfield built from these agrees
    /// with the near ground **by construction** — identical height where a far
    /// sample lands on a near column, within the dropped sub-coarse relief in
    /// between (journal/0022). Cost is O(pyramid depth) per call and memoized; it
    /// never full-resolution-generates a chunk, and it runs into the border wilds
    /// too (the pyramid runs forever), so the horizon never dissolves into empty
    /// sky. This is the worldgen authority's OWN answer for the far field — no
    /// second terrain opinion (docs/ARCHITECTURE.md § One world-answer surface).
    pub fn coarse_surface(&mut self, vx: i64, vz: i64) -> (i32, Block) {
        // Same locale the near column resolves to (512 = 2^9 voxels): its river
        // segments carve the far surface exactly as they carve the near one.
        let locale = self.locale(vx.div_euclid(512), vz.div_euclid(512));
        let (temp_sl, precip) = self.climate_at(vx, vz);
        let s = self.surface_sample(vx, vz, &locale.segs, locale.fringe, temp_sl, precip);
        self.evict();
        (s.h, s.block)
    }

    /// **Measurement only** (S13, `docs/spikes/S13-results.md`): the
    /// *continuous* surface elevation in metres at one world voxel column — the
    /// exact value [`Self::surface_sample`] floors into a voxel height, river
    /// carving included. Nothing in generation calls this; it exists so a probe
    /// can measure relief without paying the 0.9 m quantization step, and it
    /// cannot change any generated output (it is the same pure function the
    /// column collapse already evaluates, read one step earlier).
    pub fn surface_elev_m(&mut self, vx: i64, vz: i64) -> f64 {
        let locale = self.locale(vx.div_euclid(512), vz.div_euclid(512));
        let (raw, _) = self.lattice(L_VOXEL, vx, vz);
        let (elev, _) = carve_rivers(raw, vx as f64, vz as f64, &locale.segs);
        self.evict();
        elev
    }

    /// **Measurement only** (S13): one elevation-lattice point's
    /// `(elevation m, roughness m)` — the read-only window onto the refinement
    /// pyramid a roughness probe needs to attribute relief per level. A pure
    /// derivation; consulting it can never change a generated chunk.
    pub fn lattice_point(&mut self, level: u8, i: i64, j: i64) -> (f64, f64) {
        let v = self.lattice(level, i, j);
        self.evict();
        v
    }

    /// **Measurement only** (journal/0053): the column's year-zero
    /// `(sea-level temperature °C, precipitation)` — the exact pair the collapse
    /// consults, exposed so a probe can print the *retired* precipitation-driven
    /// soil rule alongside the regolith-driven one it replaced. A pure
    /// derivation; consulting it can never change a generated chunk.
    pub fn climate_probe(&mut self, vx: i64, vz: i64) -> (f64, f64) {
        let c = self.climate_at(vx, vz);
        self.evict();
        c
    }

    /// Chunk y containing the highest surface voxel of this chunk footprint.
    pub fn surface_chunk_y(&mut self, cx: i64, cz: i64) -> i32 {
        let col = self.column(cx, cz);
        let max = col.heights.iter().copied().max().expect("1024 heights");
        self.evict();
        i64::from(max).div_euclid(32) as i32
    }

    fn assert_bounds(&self) {
        let s = &self.last_stats;
        let b = &LOOKAHEAD_BOUNDS;
        assert!(
            s.cells <= b.cells
                && s.regions <= b.regions
                && s.locales <= b.locales
                && s.columns <= b.columns
                && s.lattice_points <= b.lattice_points,
            "unbounded lookahead: {s:?}"
        );
    }

    /// Caches are pure derivations; eviction can never change any answer.
    ///
    /// Called from [`Self::generate_chunk`] **and from every public sampling
    /// entry point** ([`Self::coarse_surface`], [`Self::column_record`],
    /// [`Self::surface_chunk_y`], [`Self::surface_elev_m`],
    /// [`Self::lattice_point`]). Before that it was reachable only through
    /// chunk generation, so a client streaming a horizon — far-field summaries
    /// only, no chunks — grew `lattice_memo`/`locale_cache`/`region_cache`
    /// without bound (journal/0050). Determinism-neutral by the contract above:
    /// every cached value is a pure function of seed + pregen, so dropping one
    /// costs a recomputation and nothing else.
    fn evict(&mut self) {
        if self.column_cache.len() > 8192 {
            self.column_cache.clear();
        }
        if self.locale_cache.len() > 65536 {
            self.locale_cache.clear();
        }
        if self.region_cache.len() > 65536 {
            self.region_cache.clear();
        }
        if self.lattice_memo.len() > 2_000_000 {
            // Keep the coarse levels (cheap, widely shared); drop fine ones.
            self.lattice_memo.retain(|k, _| k.0 <= L_COLUMN);
        }
    }

    // ----- elevation lattice ------------------------------------------------

    /// `(elevation m, roughness m)` at lattice point `(level, i, j)`; the
    /// point's world position is `(i, j) * 2^(14 - level)` voxels.
    fn lattice(&mut self, level: u8, i: i64, j: i64) -> (f64, f64) {
        self.trace.lattice.insert((level, i, j));
        if let Some(&v) = self.lattice_memo.get(&(level, i, j)) {
            return v;
        }
        let mut v = if level == 0 {
            self.corner0(i, j)
        } else {
            let (pi, pj) = (i >> 1, j >> 1);
            let parent = match (i & 1 == 0, j & 1 == 0) {
                (true, true) => self.lattice(level - 1, pi, pj),
                (false, true) => avg2(
                    self.lattice(level - 1, pi, pj),
                    self.lattice(level - 1, pi + 1, pj),
                ),
                (true, false) => avg2(
                    self.lattice(level - 1, pi, pj),
                    self.lattice(level - 1, pi, pj + 1),
                ),
                (false, false) => avg2(
                    avg2(
                        self.lattice(level - 1, pi, pj),
                        self.lattice(level - 1, pi + 1, pj),
                    ),
                    avg2(
                        self.lattice(level - 1, pi, pj + 1),
                        self.lattice(level - 1, pi + 1, pj + 1),
                    ),
                ),
            };
            let u = draw_f64(&[self.seed, SALT_ELEV, u64::from(level), i as u64, j as u64])
                .mul_add(2.0, -1.0);
            (
                parent.0 + u * parent.1 * AMP_DECAY.powi(i32::from(level)),
                parent.1,
            )
        };
        // 3e-1: at the deep tier's own resolution (level L_DEEP ≈ 460 m ≈ the
        // locale), replace the analytic elevation with the eroded deep-time
        // surface (bilinear → C0-continuous, so adjacent locale corners differ
        // gently over the 512-voxel span and the ≤6-voxel seam invariant holds).
        // Roughness is untouched — it still schedules the finer midpoint jitter.
        // Outside the deep grid (the border wilds) the sample is absent and the
        // analytic value stands, so the boundary keeps its single-field
        // continuity (both sides ≈ the shared pregen edge elevation).
        if level == L_DEEP {
            let shift = u32::from(L_VOXEL - L_DEEP);
            if let Some(e) = self.pregen.deep.surface_at_voxel(i << shift, j << shift) {
                v.0 = e;
            }
        }
        self.lattice_memo.insert((level, i, j), v);
        v
    }

    /// Level-0 lattice anchor: the average of the four coarse cells sharing
    /// the corner (pregen cells inside the grid, wilds synthesis outside).
    fn corner0(&mut self, i: i64, j: i64) -> (f64, f64) {
        let half = i64::from(self.pregen.grid.w / 2);
        let mut elev = 0.0;
        let mut rough = 0.0;
        for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
            let v = self.cell_view_traced(i + half + dx, j + half + dy);
            elev += v.elev_m;
            rough += v.rough_m;
        }
        (elev / 4.0, rough / 4.0)
    }

    fn cell_view_traced(&mut self, gx: i64, gy: i64) -> crate::pregen::CellView {
        self.trace.cells.insert((gx, gy));
        self.pregen.cell_view(gx, gy)
    }

    /// Bilinear `(sea-level temp °C, precip)` between cell centres.
    ///
    /// Registration: `half` is the **integer** grid-centre offset `w / 2` (the
    /// array index of the cell straddling the world origin), cast to `f64` —
    /// NOT `f64::from(w) / 2.0`. For the odd `w` every preset uses (5/17/69) the
    /// float form is `w/2 + 0.5`, which would shift the continuous coordinate
    /// half a cell so integer grid coords land on cell *corners* instead of
    /// centres; bilinear at a cell centre would then blend the four neighbours
    /// and the whole climate field would sit ~7.4 km (`CELL_VOXELS/2 · 0.9 m`)
    /// off the terrain it tints. This matches `cell_of_voxel` /
    /// `cell_center_voxel` / `DeepField::deep_coords` / `grid::build_cells`, all
    /// integer `w / 2`. See journal/0043 and the S13 flag; regression-guarded by
    /// `climate_at_reproduces_cells_own_climate_at_centre`.
    fn climate_at(&mut self, vx: i64, vz: i64) -> (f64, f64) {
        let half = f64::from(self.pregen.grid.w / 2);
        let gx = vx as f64 / CELL_VOXELS as f64 + half - 0.5;
        let gy = vz as f64 / CELL_VOXELS as f64 + half - 0.5;
        let (x0, y0) = (gx.floor(), gy.floor());
        let (tx, ty) = (gx - x0, gy - y0);
        let mut lat = 0.0;
        let mut precip = 0.0;
        for (dx, dy, w) in [
            (0.0, 0.0, (1.0 - tx) * (1.0 - ty)),
            (1.0, 0.0, tx * (1.0 - ty)),
            (0.0, 1.0, (1.0 - tx) * ty),
            (1.0, 1.0, tx * ty),
        ] {
            let v = self.cell_view_traced((x0 + dx) as i64, (y0 + dy) as i64);
            lat += w * v.lat_deg;
            precip += w * v.precip;
        }
        (temp_sea_level(lat), precip)
    }

    // ----- semantic records: region → locale → column ----------------------

    fn region(&mut self, rx: i64, rz: i64) -> Arc<RegionRec> {
        self.trace.regions.insert((rx, rz));
        if let Some(r) = self.region_cache.get(&(rx, rz)) {
            return r.clone();
        }
        let base = self.region_base(rx, rz);
        // Summary over the 1-ring neighbours' *base* states.
        let mut wilds_adjacent = false;
        for dz in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let nb = self.region_base(rx + dx, rz + dz);
                if !nb.civilized {
                    wilds_adjacent = true;
                }
            }
        }
        let mut segs = base.segs;
        // Canonical order so overlapping rivers blend identically no matter
        // which region collected them.
        segs.sort_by(|a, b| {
            (a.ax, a.az, a.bx, a.bz)
                .partial_cmp(&(b.ax, b.az, b.bx, b.bz))
                .expect("finite coords")
        });
        segs.dedup();
        let rec = Arc::new(RegionRec {
            fringe: base.civilized && wilds_adjacent,
            segs,
            site: base.site,
        });
        self.region_cache.insert((rx, rz), rec.clone());
        rec
    }

    /// Pure function of (seed, region coords, parent-cell neighbourhood).
    fn region_base(&mut self, rx: i64, rz: i64) -> RegionBase {
        self.trace.regions.insert((rx, rz));
        let region_vox = CELL_VOXELS / 2; // 8192
        let (vx0, vz0) = (rx * region_vox, rz * region_vox);
        let (gx, gy) = self
            .pregen
            .grid
            .cell_of_voxel(vx0 + region_vox / 2, vz0 + region_vox / 2);
        let pv = self.cell_view_traced(gx, gy);
        // "Civilized" here means inside the pregen grid (where history can
        // exist); the fringe flag marks the boundary ring against the wilds.
        let civilized = !pv.wilds;
        // Rivers: outflow segments of the parent cell's 5×5 neighbourhood
        // (2-ring). Any river within reach of this region has an endpoint
        // cell in that neighbourhood.
        let mut segs = Vec::new();
        for dy in -2..=2i64 {
            for dx in -2..=2i64 {
                let (cgx, cgy) = (gx + dx, gy + dy);
                self.trace.cells.insert((cgx, cgy));
                let (Ok(cgx32), Ok(cgy32)) = (i32::try_from(cgx), i32::try_from(cgy)) else {
                    continue;
                };
                let Some(c) = self.pregen.grid.get(cgx32, cgy32) else {
                    continue; // wilds cells have no rivers
                };
                if !c.river {
                    continue;
                }
                let Some(to) = c.flow_to else { continue };
                let (tgx, tgy) = self.pregen.grid.coords(to as usize);
                let a = self.pregen.grid.cell_center_voxel(cgx32, cgy32);
                let b = self.pregen.grid.cell_center_voxel(tgx, tgy);
                let to_cell = &self.pregen.grid.cells[to as usize];
                segs.push(RiverSeg {
                    ax: a.0 as f64,
                    az: a.1 as f64,
                    bx: b.0 as f64,
                    bz: b.1 as f64,
                    width: (6.0 + 10.0 * c.discharge.sqrt()).min(40.0),
                    wa: c.filled_m,
                    wb: to_cell.filled_m,
                });
            }
        }
        // The parent cell's site, if its centre falls inside this region.
        let site = (i32::try_from(gx).ok())
            .zip(i32::try_from(gy).ok())
            .and_then(|key| self.sites_by_cell.get(&key))
            .and_then(|spots| {
                spots
                    .iter()
                    .find(|s| {
                        s.x >= vx0 && s.x < vx0 + region_vox && s.z >= vz0 && s.z < vz0 + region_vox
                    })
                    .copied()
            });
        RegionBase {
            civilized,
            segs,
            site,
        }
    }

    fn locale(&mut self, lx: i64, lz: i64) -> Arc<LocaleRec> {
        self.trace.locales.insert((lx, lz));
        if let Some(l) = self.locale_cache.get(&(lx, lz)) {
            return l.clone();
        }
        let base = self.locale_base(lx, lz);
        // 1-ring base summary: neighbouring locales' sites whose footprints
        // may cross into this locale.
        let mut sites = base.sites;
        for dz in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                for s in self.locale_base(lx + dx, lz + dz).sites {
                    if !sites.iter().any(|t| t.id == s.id) {
                        sites.push(s);
                    }
                }
            }
        }
        sites.sort_by_key(|s| s.id);
        let rec = Arc::new(LocaleRec {
            fringe: base.fringe,
            segs: base.segs,
            sites,
        });
        self.locale_cache.insert((lx, lz), rec.clone());
        rec
    }

    /// Pure function of (seed, locale coords, collapsed parent region).
    fn locale_base(&mut self, lx: i64, lz: i64) -> LocaleBase {
        self.trace.locales.insert((lx, lz));
        let parent = self.region(lx >> 4, lz >> 4);
        let (vx0, vz0) = (lx * 512, lz * 512);
        let (cx, cz) = ((vx0 + 256) as f64, (vz0 + 256) as f64);
        // Keep rivers whose influence can reach this locale: centre-to-segment
        // distance within half-diagonal (362) + reach (56), rounded up.
        let segs = parent
            .segs
            .iter()
            .filter(|s| seg_point_dist(s, cx, cz) <= 362.0 + RIVER_REACH + 12.0)
            .copied()
            .collect();
        let margin = 64i64;
        let sites = parent
            .site
            .iter()
            .filter(|s| {
                s.x >= vx0 - margin
                    && s.x < vx0 + 512 + margin
                    && s.z >= vz0 - margin
                    && s.z < vz0 + 512 + margin
            })
            .copied()
            .collect();
        LocaleBase {
            fringe: parent.fringe,
            segs,
            sites,
        }
    }

    /// Collapse one chunk-column (the chunk footprint's 32×32 voxel columns).
    fn column(&mut self, cx: i64, cz: i64) -> Arc<ColumnRec> {
        self.trace.columns.insert((cx, cz));
        if let Some(c) = self.column_cache.get(&(cx, cz)) {
            return c.clone();
        }
        let locale = self.locale(cx >> 4, cz >> 4);
        let (temp_sl, precip) = self.climate_at(cx * 32 + 16, cz * 32 + 16);
        // **Regolith depth from the recorded cause** (journal/0053, retiring the
        // soil half of stubs.md § 3). The deep-time sim spends its whole run
        // weathering bedrock into loose cover, moving it, and dropping it; that
        // per-cell thickness `H` is now carried in the `DeepField` instead of
        // being summed into the surface and thrown away. `None` only in the
        // border wilds, where there is no deep-time history to read — see
        // [`wilds_regolith_voxels`].
        let regolith_m = self
            .pregen
            .deep
            .regolith_at_voxel(cx * 32 + 16, cz * 32 + 16);
        let soil = match regolith_m {
            Some(h) => regolith_voxels(h, self.voxel_m),
            None => wilds_regolith_voxels(precip),
        };
        let mut heights = vec![0i32; 1024];
        let mut surface = vec![Block::Stone; 1024];
        // The class the shared kernel says the record skins each voxel column
        // with, plus the eighths of the surface voxel the ground occupies.
        let mut surface_class: Vec<Option<(&'static str, u8)>> = vec![None; 1024];
        let mut wilds = true;
        for z in 0..32i64 {
            for x in 0..32i64 {
                let (vx, vz) = (cx * 32 + x, cz * 32 + z);
                let (gx, gy) = self.pregen.grid.cell_of_voxel(vx, vz);
                if gx >= 0
                    && gx < i64::from(self.pregen.grid.w)
                    && gy >= 0
                    && gy < i64::from(self.pregen.grid.w)
                {
                    wilds = false;
                }
                let i = (z * 32 + x) as usize;
                let s = self.surface_sample(vx, vz, &locale.segs, locale.fringe, temp_sl, precip);
                heights[i] = s.h;
                surface[i] = s.block;
                // **The top-of-column remainder** (journal/0055): `h` floors the
                // continuous surface, so the surface voxel is filled from its
                // floor up to the real ground — a genuine partial. Ceil, and at
                // least one eighth: `h = floor(elev/0.9)` means there IS ground
                // in this voxel, and rounding it away would silently drop a
                // voxel of world height.
                surface_class[i] = s.class.map(|c| {
                    let frac = (s.elev_m - f64::from(s.h) * self.voxel_m) / self.voxel_m;
                    let n = (frac * 8.0).ceil().clamp(1.0, 8.0) as u8;
                    (c, n)
                });
            }
        }
        let posts = self.ruin_posts(cx, cz, &locale);

        // ---- geology strata passes (collapse phase, bounded context) ----
        // Provenance of the parent cell (already in this column's 1-ring
        // climate consult set; traced for the lookahead instrumentation).
        let (pgx, pgy) = self.pregen.grid.cell_of_voxel(cx * 32 + 16, cz * 32 + 16);
        self.trace.cells.insert((pgx, pgy));
        let provenance = match (i32::try_from(pgx), i32::try_from(pgy)) {
            (Ok(gx32), Ok(gy32)) => self
                .pregen
                .grid
                .get(gx32, gy32)
                .map_or(Provenance::OceanFloor, |c| c.provenance),
            _ => Provenance::OceanFloor,
        };
        let mean_h = heights.iter().map(|&h| f64::from(h)).sum::<f64>() / 1024.0;
        let elev_m = mean_h * self.voxel_m;
        // Fluvial energy: discharge-scaled channel width, decaying with
        // distance from the nearest channel (canonical seg order ⇒ adjacent
        // columns agree).
        let (ccx, ccz) = ((cx * 32 + 16) as f64, (cz * 32 + 16) as f64);
        let flow_energy = locale
            .segs
            .iter()
            .map(|s| s.width * (-seg_point_dist(s, ccx, ccz) / 24.0).exp())
            .fold(0.0f64, f64::max);
        // The deep-time depositional record for this column: the strata of the
        // nearest 460 m deep cell (the at-deposition formation-context source).
        // Empty in the wilds / where the deep sim laid nothing down.
        let deep_units = self
            .pregen
            .deep
            .record_at_voxel(cx * 32 + 16, cz * 32 + 16)
            .map_or(&[][..], |s| s.units.as_slice());
        let mut strata_ctx = StrataCtx {
            seed: self.seed,
            cx,
            cz,
            temp_c: temp_sl,
            precip,
            provenance,
            elev_m,
            flow_energy,
            voxel_m: self.voxel_m,
            regolith_m,
            deep_units,
            wilds,
            geology: &self.geology,
            strata: StrataRec::default(),
            alluvium: None,
        };
        self.pregen.pipeline.run_strata(&mut strata_ctx);
        let strata = strata_ctx.strata;

        // **Skin the surface voxel from the record.** The kernel already named
        // the class (and the far field is using that same answer); here we
        // resolve it to a member under the column's own formation context and
        // build the partial contents. `block == classify(contents)` holds by
        // construction: every member of a class shares a block twin, so
        // whichever member the draw picks classifies to the block
        // `surface_sample` already returned.
        let mut member_of: Vec<(&'static str, Option<GeoMemberIdx>)> = Vec::new();
        let form = FormationContext {
            temp_c: temp_sl,
            precip,
            depth_m: 0.0,
        };
        let mut surface_fill = vec![None; 1024];
        for i in 0..1024usize {
            let Some((class, n)) = surface_class[i] else {
                continue;
            };
            let member = match member_of.iter().find(|(c, _)| *c == class) {
                Some((_, m)) => *m,
                None => {
                    let u = interp_select_draw(self.seed, SALT_GEO_SELECT, 4, cx, cz, 0.5, 0.5);
                    let m = self.geology.select(class, &form, u).map(|(i, _)| i);
                    member_of.push((class, m));
                    m
                }
            };
            let Some(member) = member else {
                continue; // an unfillable class: keep the fallback block
            };
            surface[i] = classify(&mixed_contents(&self.geology, &[(member, n)]));
            surface_fill[i] = Some((member, n));
        }

        let rec = Arc::new(ColumnRec {
            heights,
            surface,
            soil,
            posts,
            wilds,
            strata,
            surface_fill,
        });
        self.column_cache.insert((cx, cz), rec.clone());
        rec
    }

    /// Ruin posts from abandoned sites: committed pregen history, visible in
    /// the terrain. Each post is a point, so it lands in exactly one column.
    ///
    /// **STUB (docs/design/stubs.md § 1 — LOUDLY TEMPORARY).** The
    /// *abandonment fact* is real ledger; the *posts* are a rule-of-thumb
    /// stand-in for what an abandoned settlement leaves behind. Heir: the
    /// social sim + ecology (dwarf-fortress-class civilization history — a
    /// post gets there because someone put it there). Culture-related
    /// artifacts are placeholder wholesale; do not bandaid, do not extend,
    /// until ecology lands and the social sim gets its design pass.
    fn ruin_posts(&self, cx: i64, cz: i64, locale: &LocaleRec) -> Vec<(u8, u8, u8)> {
        let (vx0, vz0) = (cx * 32, cz * 32);
        let mut posts = Vec::new();
        for s in &locale.sites {
            if !s.abandoned {
                continue;
            }
            for k in 0..10u64 {
                let ang = draw_f64(&[self.seed, SALT_RUIN, u64::from(s.id), k, 0])
                    * std::f64::consts::TAU;
                let r = 6.0 + 12.0 * draw_f64(&[self.seed, SALT_RUIN, u64::from(s.id), k, 1]);
                let px = s.x + (r * ang.cos()) as i64;
                let pz = s.z + (r * ang.sin()) as i64;
                if px >= vx0 && px < vx0 + 32 && pz >= vz0 && pz < vz0 + 32 {
                    let h =
                        2 + (draw_f64(&[self.seed, SALT_RUIN, u64::from(s.id), k, 2]) * 2.0) as u8;
                    posts.push(((px - vx0) as u8, (pz - vz0) as u8, h));
                }
            }
        }
        posts
    }
}

fn avg2(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// **The quantization rule for the carried regolith plane** (journal/0053).
///
/// `H` is metres of loose cover; voxels are 0.9 m. The rule is *round to
/// nearest whole voxel*, clamped to `0..=MAX_REGOLITH_VOX`:
///
/// - **Round, not floor.** Floor would systematically shave up to a whole voxel
///   of soil off every column in the world and make the whole map barer than the
///   ledger says. Round is the minimum-error whole-voxel quantizer: the
///   half-open bands are `H < 0.45 m → 0`, `0.45..1.35 → 1`, `1.35..2.25 → 2`,
///   and so on.
/// - **Zero is reachable, and that is the point.** A cell the wind has scoured
///   to `H ≈ 0` gets no soil band at all — bedrock at the surface, which is what
///   a deflation basin is. The old precip rule had a floor of 1 voxel, so no
///   column in the world could ever be bare (journal/0049 station 1: "always
///   going to have topsoil").
/// - **Whole voxels only.** The sub-voxel remainder — the 0.3 m of grit that
///   rounds to nothing — is *not* faked as a thin layer here. Expressing a
///   partial thickness is the forms/partials slice's job (materials.md § the
///   forms design pass), and it is blocked on the surface-veneer retirement.
///   Until then a sub-half-voxel cover is honestly not expressed as a block.
fn regolith_voxels(thickness_m: f64, voxel_m: f64) -> u8 {
    (thickness_m / voxel_m)
        .round()
        .clamp(0.0, f64::from(MAX_REGOLITH_VOX)) as u8
}

/// Cap on the expressed regolith band, voxels. Matches the clastic veneer's
/// long-standing budget ceiling: a 30 m alluvial pile is real in the ledger and
/// is expressed as *strata*, not as one absurd topsoil band.
const MAX_REGOLITH_VOX: u8 = 8;

/// **Genesis fallback for the border wilds** (stubs.md § Genesis: border-wilds
/// cell synthesis is permanently legitimate). Beyond the pregen grid there is no
/// deep-time run and therefore no `H` to read, so the wilds keep the year-zero
/// precipitation rule they always had — unchanged, and now *scoped* to the one
/// place where inventing a number is legitimate because there is no recorded
/// cause to consult.
fn wilds_regolith_voxels(precip: f64) -> u8 {
    if precip > 0.5 {
        3
    } else if precip > 0.2 {
        2
    } else {
        1
    }
}

/// Canonical voxel contents for one stratum event, given the **resolved host
/// member** for this voxel-column (the boundary dither picks it) — always
/// through the [`VoxelContents`] constructors (canonical form is a hard
/// invariant). Clastic strata are loose debris (with placer ore grains
/// substituted into their eighths where the placer pass enriched the event);
/// igneous strata are full structural fill, with accessory minerals carried in
/// the host rock's **pore slots** where the pass emplaced them (3d pore
/// partials — the placer pattern in igneous dress).
fn contents_for_event(set: &GeologySet, host: GeoMemberIdx, e: &StrataEvent) -> VoxelContents {
    let host_mat = set.member(host).material;
    let class = set.member(host).class.as_str();
    if class == CLASS_CLASTIC_FINE || class == CLASS_CLASTIC_COARSE {
        let mut debris = [host_mat; 8];
        if let Some((ore_member, eighths)) = e.ore {
            let ore = set.member(ore_member).material;
            for slot in debris.iter_mut().take(usize::from(eighths.min(8))) {
                *slot = ore;
            }
        }
        VoxelContents::debris_only(&debris).expect("8 debris eighths fit an open voxel")
    } else if let Some((acc_member, eighths)) = e.accessory {
        // Host rock in the structure slots; accessory mineral in the pores.
        let k = eighths.clamp(1, 7);
        let acc = set.member(acc_member).material;
        let structure = [host_mat; 8];
        let pore = [acc; 8];
        VoxelContents::new(
            StructureShape::Full,
            &structure[..usize::from(8 - k)],
            &pore[..usize::from(k)],
            &[],
        )
        .expect("host structure + accessory pore fill is canonical")
    } else {
        VoxelContents::new(StructureShape::Full, &[host_mat; 8], &[], &[])
            .expect("full structural fill is canonical")
    }
}

fn seg_point_dist(s: &RiverSeg, px: f64, pz: f64) -> f64 {
    let (dx, dz) = (s.bx - s.ax, s.bz - s.az);
    let len2 = dx * dx + dz * dz;
    let t = if len2 > 0.0 {
        (((px - s.ax) * dx + (pz - s.az) * dz) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (qx, qz) = (s.ax + t * dx, s.az + t * dz);
    ((px - qx).powi(2) + (pz - qz).powi(2)).sqrt()
}

/// Blend the elevation toward the river channel where segments pass close by.
/// Continuous in position; segments are canonically ordered, so adjacent
/// columns compute identical results.
fn carve_rivers(mut elev: f64, px: f64, pz: f64, segs: &[RiverSeg]) -> (f64, bool) {
    let mut riverbed = false;
    for s in segs {
        let (dx, dz) = (s.bx - s.ax, s.bz - s.az);
        let len2 = dx * dx + dz * dz;
        let t = if len2 > 0.0 {
            (((px - s.ax) * dx + (pz - s.az) * dz) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (qx, qz) = (s.ax + t * dx, s.az + t * dz);
        let d = ((px - qx).powi(2) + (pz - qz).powi(2)).sqrt();
        if d >= s.width + BANK {
            continue;
        }
        let water = s.wa + t * (s.wb - s.wa);
        let depth = 2.0 + s.width * 0.08;
        let target = (water - depth).min(elev);
        let c = if d <= s.width {
            1.0
        } else {
            let u = 1.0 - (d - s.width) / BANK;
            u * u * (3.0 - 2.0 * u) // smoothstep
        };
        elev = elev * (1.0 - c) + target * c;
        if c > 0.6 && elev < water {
            riverbed = true;
        }
    }
    (elev, riverbed)
}

#[cfg(test)]
mod tests {
    use dc_core::materials::geology::{
        self, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE,
        CLASS_IGNEOUS_INTRUSIVE, CLASS_ORGANIC_COAL, CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL,
        FormationWindow, GeoHabit, GeoMemberDef, GeoMemberIdx, GeologySet,
    };
    use dc_core::{Block, MaterialId, classify};

    use super::contents_for_event;
    use crate::WorldGenerator;
    use crate::geology::StrataEvent;
    use crate::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams, temp_sea_level};

    /// **The retired class-to-block table**, kept only as the regression oracle
    /// for the fill contract: before 2026-07-21 this is how `generate_chunk`
    /// computed a band's block, in parallel with (and in ignorance of) the
    /// contents. `classify(contents_for_event(..))` must reproduce it for every
    /// member of every registered set — that equivalence is what makes the
    /// rewire byte-neutral, and it is asserted below rather than assumed.
    fn block_for_member(set: &GeologySet, member: GeoMemberIdx) -> Block {
        match set.member(member).class.as_str() {
            c if c == CLASS_CLASTIC_FINE => Block::Mudstone,
            c if c == CLASS_CLASTIC_COARSE => Block::Sandstone,
            c if c == CLASS_IGNEOUS_INTRUSIVE => Block::Granite,
            c if c == CLASS_IGNEOUS_EXTRUSIVE => Block::Basalt,
            c if c == CLASS_ORGANIC_COAL => Block::Coal,
            c if c == CLASS_ORGANIC_PEAT => Block::Peat,
            c if c == CLASS_ORGANIC_SOIL => Block::CarbonaceousMudstone,
            _ => Block::Stone,
        }
    }

    fn probe_event(member: GeoMemberIdx) -> StrataEvent {
        StrataEvent {
            member,
            thickness_m: 3.6,
            temp_c: 12.0,
            precip: 0.5,
            depth_m: 10.0,
            sel_salt: 0,
            sel_tag: 0,
            ore: None,
            accessory: None,
        }
    }

    /// The equivalence the byte-identity goldens rest on, stated directly: for
    /// every member of a registered set, and for every enrichment shape the
    /// strata passes can attach (bare, placer ore up to its 3/8 ceiling,
    /// accessory pore inclusion), the block derived from the event's contents
    /// equals what the retired class table answered.
    ///
    /// Run over vanilla AND over a pack that binds *loose* materials into rock
    /// classes (`SILT` as a fine clastic, `GRAVEL` as a coarse one — exactly
    /// what tests/geology.rs registers), because that pack is where a
    /// material-keyed table could have diverged from a class-keyed one.
    #[test]
    fn classify_reproduces_the_retired_class_table() {
        let mut sets = vec![geology::vanilla()];
        {
            let mut members = geology::vanilla_members();
            members.push(GeoMemberDef {
                id: "zz:geo/siltstone".into(),
                class: CLASS_CLASTIC_FINE.into(),
                material: MaterialId::SILT,
                window: FormationWindow::ANY,
                abundance: 1.0,
                habit: GeoHabit::Blanket,
                hardness: 0.3,
                erodibility: 0.75,
            });
            members.push(GeoMemberDef {
                id: "aa:geo/greywacke".into(),
                class: CLASS_CLASTIC_COARSE.into(),
                material: MaterialId::GRAVEL,
                window: FormationWindow::ANY,
                abundance: 1.0,
                habit: GeoHabit::Blanket,
                hardness: 0.55,
                erodibility: 0.45,
            });
            let mut b = GeologySet::builder();
            for class in geology::v1_classes() {
                b.declare_class(class).unwrap();
            }
            for m in members {
                b.add_member(m).unwrap();
            }
            sets.push(b.build());
        }

        let mut checked = 0usize;
        for set in &sets {
            let ore = set.member_index("dc:geo/gold-dust");
            let acc = set.member_index("dc:geo/olivine");
            for (i, def) in set.members().iter().enumerate() {
                let member = GeoMemberIdx(i as u16);
                let want = block_for_member(set, member);
                let mut events = vec![probe_event(member)];
                for k in 1..=3u8 {
                    let mut e = probe_event(member);
                    e.ore = ore.map(|o| (o, k));
                    events.push(e);
                }
                for k in 1..=7u8 {
                    let mut e = probe_event(member);
                    e.accessory = acc.map(|a| (a, k));
                    events.push(e);
                }
                for e in &events {
                    // The ore/accessory enrichments only apply to the class
                    // that carries them; `contents_for_event` ignores the
                    // irrelevant one, so every combination is legal input.
                    let got = classify(&contents_for_event(set, member, e));
                    assert_eq!(
                        got, want,
                        "member {} ({}): classify -> {got:?}, retired table -> {want:?}",
                        def.id, def.class
                    );
                    checked += 1;
                }
            }
        }
        assert!(
            checked >= 200,
            "only {checked} member/enrichment combinations"
        );
    }

    fn small(seed: u64) -> Pregen {
        Pregen::run(WorldParams {
            seed,
            extent: Extent::Small,
        })
    }

    /// Climate registration (S13 flag; journal/0043). `climate_at` bilinearly
    /// interpolates the coarse per-cell climate; its voxel→grid mapping MUST
    /// anchor integer grid coordinates at cell CENTRES — the convention
    /// `cell_of_voxel` / `cell_center_voxel` / `DeepField::deep_coords` /
    /// `grid::build_cells` all share (integer `w / 2`). Ground truth by
    /// construction: bilinear evaluated at a node returns that node's value
    /// exactly, so a voxel sitting on a cell's own centre must read back that
    /// cell's own baked `(temp, precip)` — no neighbour blend. A half-cell
    /// mis-registration (float `w / 2.0`, which for ODD `w` is `w/2 + 0.5`)
    /// averages the four neighbours instead, i.e. reads climate ~7.4 km
    /// (`CELL_VOXELS/2 · 0.9 m`) north-and-east of the terrain it tints.
    /// Every preset is odd (Small 5, Medium 17, Large 69), so the parity bug is
    /// live at all sizes; exercised at Small AND Medium here. Interior cells
    /// only — the four bilinear neighbours must all be in-grid.
    #[test]
    fn climate_at_reproduces_cells_own_climate_at_centre() {
        for extent in [Extent::Small, Extent::Medium] {
            let pregen = Pregen::run(WorldParams {
                seed: 0x00C1_1A7E_2026,
                extent,
            });
            let w = pregen.grid.w;
            let mut g = WorldGenerator::new(&pregen);
            let mut checked = 0usize;
            for gy in 1..w - 1 {
                for gx in 1..w - 1 {
                    let (vx, vz) = pregen.grid.cell_center_voxel(gx, gy);
                    let (temp, precip) = g.climate_at(vx, vz);
                    let cv = pregen.cell_view(i64::from(gx), i64::from(gy));
                    let want_temp = temp_sea_level(cv.lat_deg);
                    assert!(
                        (temp - want_temp).abs() < 1e-9,
                        "temp at cell ({gx},{gy}) centre = {temp}, cell's own = {want_temp} \
                         (w={w}) — bilinear must reproduce a node exactly at its centre"
                    );
                    assert!(
                        (precip - cv.precip).abs() < 1e-9,
                        "precip at cell ({gx},{gy}) centre = {precip}, cell's own = {} \
                         (w={w}) — bilinear must reproduce a node exactly at its centre",
                        cv.precip
                    );
                    checked += 1;
                }
            }
            assert!(checked >= 9, "sampled {checked} interior cells (w={w})");
        }
    }

    /// The far-field summary is the SAME surface function the near ground
    /// collapses from: `coarse_surface` must return the exact per-column height
    /// `column_record` computes at every coinciding voxel — height agreement by
    /// construction (journal/0022), the guarantee the near/far horizon seam rests
    /// on. Sampled across several chunks and both civilized + fringe columns.
    #[test]
    fn coarse_surface_matches_near_column_height() {
        let pregen = Pregen::run(WorldParams {
            seed: 0x0D5E_ED57_2026,
            extent: Extent::Small,
        });
        let mut g = WorldGenerator::new(&pregen);
        let mut checked = 0usize;
        for cx in -3..=3i64 {
            for cz in -3..=3i64 {
                let col = g.column_record(cx, cz);
                for &(lx, lz) in &[(0usize, 0usize), (7, 19), (16, 16), (31, 31)] {
                    let (vx, vz) = (cx * 32 + lx as i64, cz * 32 + lz as i64);
                    let near = col.heights[lz * 32 + lx];
                    let (far, _block) = g.coarse_surface(vx, vz);
                    assert_eq!(
                        far, near,
                        "coarse_surface height {far} != near column height {near} at voxel ({vx},{vz})"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 100, "sampled {checked} columns");
    }

    /// Summaries are a pure function of the world seed: two generators over the
    /// same seed produce byte-identical coarse surfaces, and a different seed
    /// diverges (the generator is really in the loop). The determinism the
    /// persisted-summary follow-on will rely on.
    #[test]
    fn coarse_surface_is_seed_deterministic() {
        let pa = small(1337);
        let pb = small(1337);
        let pc = small(1338);
        let mut a = WorldGenerator::new(&pa);
        let mut b = WorldGenerator::new(&pb);
        let mut c = WorldGenerator::new(&pc);
        let mut any_diff = false;
        for k in -40..=40i64 {
            let (vx, vz) = (k * CELL_VOXELS / 7, -k * 53);
            assert_eq!(a.coarse_surface(vx, vz), b.coarse_surface(vx, vz));
            if a.coarse_surface(vx, vz) != c.coarse_surface(vx, vz) {
                any_diff = true;
            }
        }
        assert!(
            any_diff,
            "a different seed must produce a different surface"
        );
    }

    /// Deriving a whole far-field's worth of coarse summary is well under a
    /// second (journal/0022 perf claim): the horizon streams in, it is not a
    /// world-create tax. ~160 k coarse columns — the order of a full 1.2 km, four
    /// LOD-ring far field — sampled from a warm generator. Prints the measured
    /// number for the journal; asserts only a generous ceiling so it is a guard,
    /// not a flake.
    #[test]
    fn coarse_surface_far_field_derivation_is_sub_second() {
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        });
        let mut g = WorldGenerator::new(&pregen);
        let stride = 2i64; // level-1 coarse stride, the densest ring
        let side = 400i64; // 400² = 160 000 coarse columns
        let t0 = std::time::Instant::now();
        let mut acc = 0i64;
        for j in 0..side {
            for i in 0..side {
                let (h, _b) = g.coarse_surface(i * stride, j * stride);
                acc = acc.wrapping_add(h as i64);
            }
        }
        let dt = t0.elapsed();
        let n = side * side;
        println!(
            "coarse_surface far-field derivation: {n} columns in {:?} ({:.3} µs/column), checksum {acc}",
            dt,
            dt.as_secs_f64() * 1e6 / n as f64
        );
        assert!(
            dt.as_secs_f64() < 3.0,
            "far-field summary derivation {dt:?} should be well under a world-create budget"
        );
    }

    /// The far field runs into the border wilds too (no `None`, no empty sky):
    /// far out past the pregen grid the pyramid still answers a height, so the
    /// horizon is built everywhere the player can look (journal/0022).
    #[test]
    fn coarse_surface_answers_in_the_border_wilds() {
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Small,
        });
        let mut g = WorldGenerator::new(&pregen);
        // Well outside any Small-extent grid (millions of voxels out).
        let (h, _b) = g.coarse_surface(50_000_000, -50_000_000);
        // A finite height (not NaN / not a panic) is all we assert — the wilds
        // have their own hostile surface, but they HAVE one.
        assert!(h.abs() < 1_000_000, "wilds height {h} is finite and sane");
    }
}
