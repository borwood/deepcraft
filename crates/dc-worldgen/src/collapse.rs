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
    GeoMemberIdx, GeologySet,
};
use dc_core::{
    Block, CHUNK_VOLUME, Chunk, ChunkPos, ContentsGrid, MaterialChunk, MixtureId, MixtureTable,
    StructureShape, VoxelContents, VoxelScale,
};
use dc_sim::statistical::rng::draw_f64;

use crate::geology::{StrataCtx, StrataEvent, StrataRec, dithered_member};
use crate::pipeline::PipelineError;
use crate::pregen::{CELL_VOXELS, Pregen, Provenance, SALT_ELEV, SALT_RUIN, temp_sea_level};

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

/// Per-refinement amplitude decay for elevation jitter.
const AMP_DECAY: f64 = 0.55;

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
    pub soil: u8,
    /// Ruin posts: (local x, local z, post height in voxels).
    pub posts: Vec<(u8, u8, u8)>,
    /// True when generated beyond the pregen grid (border wilds).
    pub wilds: bool,
    /// The ordered deposition log (geology strata passes). Empty where no
    /// pass deposited (ocean, wilds): the legacy soil band applies there.
    pub strata: StrataRec,
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
        let col = self.column(i64::from(pos.x), i64::from(pos.z));
        // The strata record as top-down bands: (cumulative depth, block).
        // Depth 1 = directly under the surface voxel.
        let bands = strata_bands(&self.geology, &col.strata);
        let mut chunk = Chunk::new();
        let base_y = i64::from(pos.y) * 32;
        for z in 0..32usize {
            for x in 0..32usize {
                let i = z * 32 + x;
                let h = i64::from(col.heights[i]);
                for y in 0..32usize {
                    let vy = base_y + y as i64;
                    let b = if vy > h {
                        Block::Air
                    } else if vy == h {
                        col.surface[i]
                    } else if bands.is_empty() {
                        // No deposition record (ocean, wilds): legacy soil.
                        if vy >= h - i64::from(col.soil) {
                            Block::Dirt
                        } else {
                            Block::Stone
                        }
                    } else {
                        // The 3-band fill consumes the record: recorded
                        // strata as bands, unrecorded basement below = stone.
                        let depth = (h - vy) as u32;
                        bands
                            .iter()
                            .find(|&&(end, _)| depth <= end)
                            .map_or(Block::Stone, |&(_, b)| b)
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
        let events = event_spans(&col.strata);
        let mut dense = vec![MixtureId::EMPTY; CHUNK_VOLUME];
        if !events.is_empty() {
            let base_y = i64::from(pos.y) * 32;
            // Memoize per (event, resolved host member): the boundary dither
            // re-selects the host member per voxel-column, so the intern key is
            // the pair, not the event alone (one intern per distinct mixture in
            // this chunk — still bounded, and interning is idempotent).
            let mut memo: HashMap<(usize, GeoMemberIdx), MixtureId> = HashMap::new();
            for z in 0..32usize {
                for x in 0..32usize {
                    let h = i64::from(col.heights[z * 32 + x]);
                    for y in 0..32usize {
                        let vy = base_y + y as i64;
                        if vy >= h {
                            continue; // surface voxel and air carry no record
                        }
                        let depth = (h - vy) as u32;
                        let Some(k) = events.iter().position(|&(end, _)| depth <= end) else {
                            continue; // unrecorded basement
                        };
                        let event = &events[k].1;
                        // Per-voxel-column host member (family contacts wander
                        // off the chunk grid); ore/accessory stay per-event.
                        let host = dithered_member(&self.geology, self.seed, event, cx, cz, x, z);
                        let id = *memo.entry((k, host)).or_insert_with(|| {
                            self.materials
                                .intern(contents_for_event(&self.geology, host, event))
                        });
                        dense[Chunk::index(x, y, z)] = id;
                    }
                }
            }
        }
        dense
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
        self.column(cx, cz)
    }

    /// Chunk y containing the highest surface voxel of this chunk footprint.
    pub fn surface_chunk_y(&mut self, cx: i64, cz: i64) -> i32 {
        let col = self.column(cx, cz);
        let max = col.heights.iter().copied().max().expect("1024 heights");
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
    fn climate_at(&mut self, vx: i64, vz: i64) -> (f64, f64) {
        let half = f64::from(self.pregen.grid.w) / 2.0;
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
        let soil = if precip > 0.5 {
            3
        } else if precip > 0.2 {
            2
        } else {
            1
        };
        let mut heights = vec![0i32; 1024];
        let mut surface = vec![Block::Stone; 1024];
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
                let (raw, _) = self.lattice(L_VOXEL, vx, vz);
                let (elev, riverbed) = carve_rivers(raw, vx as f64, vz as f64, &locale.segs);
                let h = (elev / self.voxel_m).floor() as i32;
                let t = temp_sl - 6.5 * elev.max(0.0) / 1000.0;
                let i = (z * 32 + x) as usize;
                heights[i] = h;
                // Riverbeds, deserts, and the blighted wilds-fringe read as
                // bare Dirt; frozen or abyssal surfaces as Stone; temperate
                // watered land grows Grass.
                let bare = riverbed || precip < 0.10 || (locale.fringe && precip < 0.35);
                surface[i] = if elev <= -1.0 {
                    if elev > -35.0 {
                        Block::Dirt
                    } else {
                        Block::Stone
                    }
                } else if t < -4.0 {
                    Block::Stone
                } else if bare {
                    Block::Dirt
                } else {
                    Block::Grass
                };
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
            deep_units,
            wilds,
            geology: &self.geology,
            strata: StrataRec::default(),
            alluvium: None,
        };
        self.pregen.pipeline.run_strata(&mut strata_ctx);
        let strata = strata_ctx.strata;

        let rec = Arc::new(ColumnRec {
            heights,
            surface,
            soil,
            posts,
            wilds,
            strata,
        });
        self.column_cache.insert((cx, cz), rec.clone());
        rec
    }

    /// Ruin posts from abandoned sites: committed pregen history, visible in
    /// the terrain. Each post is a point, so it lands in exactly one column.
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

/// Block-tier reading of a class member: the four v1 strata classes get their
/// own visible block (ROADMAP 3c-1) so walked geology reads at a glance —
/// clastic-fine → Mudstone, clastic-coarse → Sandstone, intrusive → Granite,
/// extrusive → Basalt. Unknown future classes (and the placer ore, which never
/// forms a band of its own — it rides *inside* a clastic event) fall back to
/// Stone. The voxel *contents* still carry the real member via the material
/// sidecar; these blocks are the stand-in the data-driven registry (3c-2)
/// supersedes.
fn block_for_member(set: &GeologySet, member: GeoMemberIdx) -> Block {
    match set.member(member).class.as_str() {
        c if c == CLASS_CLASTIC_FINE => Block::Mudstone,
        c if c == CLASS_CLASTIC_COARSE => Block::Sandstone,
        c if c == CLASS_IGNEOUS_INTRUSIVE => Block::Granite,
        c if c == CLASS_IGNEOUS_EXTRUSIVE => Block::Basalt,
        _ => Block::Stone,
    }
}

/// The record as top-down `(cumulative depth, block)` bands.
fn strata_bands(set: &GeologySet, strata: &StrataRec) -> Vec<(u32, Block)> {
    let mut bands = Vec::with_capacity(strata.events.len());
    let mut acc = 0u32;
    for e in strata.events.iter().rev() {
        acc += u32::from(e.thickness_vox);
        bands.push((acc, block_for_member(set, e.member)));
    }
    bands
}

/// The record as top-down `(cumulative depth, event)` spans.
fn event_spans(strata: &StrataRec) -> Vec<(u32, StrataEvent)> {
    let mut spans = Vec::with_capacity(strata.events.len());
    let mut acc = 0u32;
    for e in strata.events.iter().rev() {
        acc += u32::from(e.thickness_vox);
        spans.push((acc, *e));
    }
    spans
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
