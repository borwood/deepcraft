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
//!   rivers and the civilized-fringe flag. *Settlement sites and their ruin
//!   posts also rode these records until 2026-07-28 (journal/0121), when the
//!   bootstrap history content was removed; nothing stands in for them.* Each record is built
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
//! synthesized cell views ([`Pregen::cell_view`]) — hostile
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
    StructureShape, VoxelContents, VoxelScale, classify,
};
use dc_sim::statistical::rng::Draws;

use crate::draws::{Elev, GeoClass, GeoSelect, interp_corner_field};
use crate::fill::{
    ColumnFill, Plan, allocate_partial, fill_draw, mixed_contents, pore_draw, pore_rider_share,
};
use crate::geology::{StrataCtx, StrataEvent, StrataRec, deep_class_of_species, dithered_member};
use crate::pipeline::PipelineError;
use crate::pregen::{CELL_VOXELS, Pregen, Provenance, temp_sea_level};

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

/// Base state of a region: a pure function of (seed, coords, parent cell
/// neighbourhood). No recursion into other regions.
struct RegionBase {
    civilized: bool,
    segs: Vec<RiverSeg>,
}

/// Collapsed region: base + 1-ring base summary + parent (pregen cell).
pub struct RegionRec {
    /// Civilized region bordering the wilds (from the 1-ring base summary).
    fringe: bool,
    segs: Vec<RiverSeg>,
}

struct LocaleBase {
    fringe: bool,
    segs: Vec<RiverSeg>,
}

/// Collapsed locale: base + 1-ring base summary + parent region.
pub struct LocaleRec {
    fringe: bool,
    segs: Vec<RiverSeg>,
}

/// Collapsed chunk-column: the 32×32 voxel-column surface of one chunk
/// footprint — heights, surface blocks, soil depth.
pub struct ColumnRec {
    /// Surface voxel y per voxel column, indexed `z * 32 + x`.
    pub heights: Vec<i32>,
    pub surface: Vec<Block>,
    /// Regolith band depth, whole voxels — the deep sim's carried `H` plane
    /// quantized by [`regolith_voxels`] (or, in the wilds only, the genesis
    /// precip rule). Applies where no strata pass deposited (ocean, wilds);
    /// **may be 0**, which is a column with bedrock at the surface.
    pub soil: u8,
    /// True when generated beyond the pregen grid (border wilds).
    pub wilds: bool,
    /// The ordered deposition log (geology strata passes). Empty where no
    /// pass deposited (ocean, wilds): the legacy soil band applies there.
    pub strata: StrataRec,
    /// **How many eighths of the surface voxel the ground occupies**, per voxel
    /// column: `ceil((elev − h·0.9) / 0.9 · 8)`, at least 1 — the top-of-column
    /// remainder (journal/0055). The surface voxel is the *top span* of
    /// [`ColumnFill`] (`plan(1)`) expressed as a partial of this many eighths,
    /// so it routes through the **same** fill machinery as every buried voxel
    /// (journal/0074 — the surface-branch removal). 8 where the ground fills the
    /// whole surface voxel. Meaningless where the column has no record top span
    /// (border wilds, subaqueous, bare-province): there the surface voxel keeps
    /// the year-zero fallback block ([`Self::surface`]) and carries no contents.
    pub surface_eighths: Vec<u8>,
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
    /// **The member of that class this voxel column resolves to**
    /// (journal/0058), under the same per-voxel-column boundary dither the
    /// buried fill uses. `None` under a fallback, or when the class cannot
    /// resolve a member at all — in which case [`Self::block`] still carries the
    /// class's block twin and the surface voxel simply gets no contents.
    pub member: Option<GeoMemberIdx>,
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
    /// The resolved provider set for this world (the collapse tier's half of the
    /// seam the deep-time sim carries in `DeepConfig`). `Providers::default()` —
    /// every slot identity — until an heir is resolved at world build, so a
    /// default generator is byte-identical to the pre-seam collapse path. The
    /// clastic strata pass reads `paleo_temperature` through it (journal/0078).
    providers: crate::deeptime::providers::Providers,
    /// Region-scale mixture intern table (S8 `materials/mixtures-v0` path);
    /// ids are first-intern order, deterministic given generation order.
    materials: MixtureTable,
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
        let seed = pregen.seed;
        Self {
            pregen,
            seed,
            voxel_m: scale.voxel_size_m(),
            geology,
            providers: crate::deeptime::providers::Providers::default(),
            materials: MixtureTable::new(),
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
        let has_record = !col.strata.events.is_empty();
        // A buried single-event voxel's block is `classify` of the very contents
        // [`Self::material_ids`] interns for it — the SAME per-voxel-column
        // dithered host member, keyed and memoized identically. Before the
        // block↔material collapse (journal/0087) every member of a content class
        // shared a `block_twin`, so the event's recorded member sufficed and this
        // was a flat per-event table; now the block IS the member's material, so
        // it MUST track the dither or `block == classify(contents)` breaks
        // (contents_contract). The memo is `(event, host) -> Block`, the block
        // twin of `material_ids`' `(event, host) -> MixtureId` memo.
        let mut block_memo: HashMap<(usize, GeoMemberIdx), Block> = HashMap::new();
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
                    } else if !has_record {
                        // No deposition record (ocean, wilds): legacy soil.
                        if vy >= h - i64::from(col.soil) {
                            Block::Dirt
                        } else {
                            Block::Stone
                        }
                    } else {
                        // The record fills the column; below it is unrecorded
                        // basement = stone. Every recorded voxel's block is
                        // `classify(contents)` of the very contents the material
                        // path builds — one opinion per voxel, journal/0052.
                        //
                        // **`depth + 1`, not `depth`** (journal/0074): the record's
                        // topmost span (`plan(1)`) is the *surface* voxel now, not
                        // the voxel below it. `depth = h − vy` is 1 for the first
                        // buried voxel, so it reads `plan(2)` = the record's second
                        // span. The whole buried column shifts down one record span
                        // to make room for the surface it now owns.
                        let depth = (h - vy) as u32;
                        match fill.plan(depth + 1) {
                            None => Block::Stone,
                            Some(Plan::Single(k)) => {
                                let event = col.strata.events[*k];
                                let host =
                                    dithered_member(&self.geology, self.seed, &event, cx, cz, x, z);
                                *block_memo.entry((*k, host)).or_insert_with(|| {
                                    classify(&contents_for_event(&self.geology, host, &event))
                                })
                            }
                            Some(Plan::Mixed(w)) => {
                                classify(&self.mixed_at(&col.strata, w, vx, vy, vz, 8))
                            }
                        }
                    };
                    if b != Block::Air {
                        chunk.set(x, y, z, b);
                    }
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
        // **The surface voxel is the record's top span** (journal/0074): the
        // same `plan(1)` the block path resolves it from, expressed as the
        // per-column partial. It used to be a parallel `surface_fill` painted by
        // `surface_class` (the deep-record consult) — that path is gone from the
        // near field and now lives only in the far summary. A column with no
        // record top span keeps the year-zero fallback block and no contents,
        // which is the shrinking absent-contents exception.
        {
            let mut memo: HashMap<(usize, GeoMemberIdx, u8), MixtureId> = HashMap::new();
            for z in 0..32usize {
                for x in 0..32usize {
                    let i = z * 32 + x;
                    let vy = i64::from(col.heights[i]);
                    let y = vy - base_y;
                    if !(0..32).contains(&y) {
                        continue;
                    }
                    let (vx, vz) = (cx * 32 + x as i64, cz * 32 + z as i64);
                    // The surface voxel is `plan(1)` — the record's topmost span
                    // (the buried column is shifted down one span to make room).
                    let Some(plan) = fill.plan(1) else {
                        continue; // fallback column: no record to skin it with
                    };
                    let n = col.surface_eighths[i];
                    // Memo key: `Single` interns per (event, dithered host, n);
                    // `Mixed` is position-addressed and rarely repeats, so it is
                    // keyed on a sentinel and simply re-interned (idempotent).
                    let id = match plan {
                        Plan::Single(k) => {
                            let event = col.strata.events[*k];
                            let host =
                                dithered_member(&self.geology, self.seed, &event, cx, cz, x, z);
                            *memo.entry((*k, host, n)).or_insert_with(|| {
                                self.materials
                                    .intern(mixed_contents(&self.geology, &[(host, n)]))
                            })
                        }
                        Plan::Mixed(_) => {
                            let c = self.surface_voxel_contents(&col.strata, plan, n, vx, vy, vz);
                            self.materials.intern(c)
                        }
                    };
                    dense[Chunk::index(x, y as usize, z)] = id;
                }
            }
        }
        if fill.depth_count() > 1 {
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
                        // `depth + 1`: the surface owns `plan(1)`, so the first
                        // buried voxel reads `plan(2)` (journal/0074).
                        let depth = (h - vy) as u32;
                        let id = match fill.plan(depth + 1) {
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
                                let c = self.mixed_at(&col.strata, w, vx, vy, vz, 8);
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

    /// Contents of one **mixed** voxel: the addressed stochastic allocation of
    /// `n` eighths among the events overlapping its span. `n == 8` for a full
    /// buried voxel; a partial `n < 8` is the surface voxel's top-of-column
    /// remainder (journal/0074), which shares this exact machinery — the whole
    /// point of the surface-branch removal.
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
        n: u8,
    ) -> VoxelContents {
        let u = fill_draw(self.seed, vx, vy, vz);
        let mut parts: Vec<(GeoMemberIdx, u8)> = Vec::with_capacity(weights.len() + 1);
        for (k, cnt) in allocate_partial(weights, u, n) {
            let e = &strata.events[k];
            // A placer enrichment substitutes into its host's OWN eighths — the
            // same substitution `contents_for_event` does, scaled to whatever
            // share of this voxel the host event won. Without it a fluvial fan
            // thin enough to be a mixed voxel (which, since the veneer's residue
            // went to zero, is most of them) would pan no gold at all.
            match (e.ore, e.accessory) {
                (Some((ore, k8)), _) if k8 > 0 => {
                    let g = k8.min(cnt);
                    if cnt > g {
                        parts.push((e.member, cnt - g));
                    }
                    parts.push((ore, g));
                }
                // **A LOOSE pore rider rides through the contact too**
                // (journal/0099). A weathering-front band is its parent rock
                // with the product in its pores; dropping the rider here — which
                // is what this path did until 0099 — would express the front's
                // contact voxels as pure parent rock and lose the very mass the
                // ledger conserves. Its share is **proportional** to the eighths
                // its host actually won (unlike the `ore` rider above, whose
                // `min` semantics are left exactly as they were: changing them
                // would move every placer voxel in the world). It rounds on
                // **its own** addressed draw, keyed by the event, so it is
                // independent both of the allocation that produced `cnt` and of
                // the sibling bands' riders in this same voxel (journal/0105).
                //
                // A *structural* accessory (the 1/8 igneous inclusion) is still
                // dropped at contacts — the pre-0099 carve-out, kept so no
                // existing world moves; filed as a loose end on stubs.md #20.
                (_, Some((rider, k8))) if k8 > 0 && crate::fill::is_loose(&self.geology, rider) => {
                    let g = pore_rider_share(cnt, k8, pore_draw(self.seed, vx, vy, vz, k));
                    if cnt > g {
                        parts.push((e.member, cnt - g));
                    }
                    if g > 0 {
                        parts.push((rider, g));
                    }
                }
                _ => parts.push((e.member, cnt)),
            }
        }
        mixed_contents(&self.geology, &parts)
    }

    /// The **surface voxel's contents**: the record's top span ([`ColumnFill`]
    /// `plan(1)`) expressed as a partial of `n` eighths — the top-of-column
    /// remainder (journal/0055), routed through the **same** fill machinery as
    /// every buried voxel (journal/0074, the surface-branch removal). This is the
    /// expression the old `surface_sample`/`surface_class` parallel path was
    /// summarizing; that path survives only as the far-field summary
    /// ([`Self::coarse_surface`]).
    ///
    /// A `Single` top span keeps the per-voxel-column member dither
    /// ([`dithered_member`], journal/0058) — the same dither the buried single
    /// voxels use — so the world's skin does not quantize into 28.8 m chunk
    /// patches. A `Mixed` top span allocates its eighths by the addressed draw,
    /// scaled to the partial. `block == classify(contents)` holds by
    /// construction: the member dither is within-class (block-invariant), and
    /// [`Self::column`] sets the surface block to `classify` of exactly these
    /// contents.
    fn surface_voxel_contents(
        &self,
        strata: &StrataRec,
        plan: &Plan,
        n: u8,
        vx: i64,
        vy: i64,
        vz: i64,
    ) -> VoxelContents {
        match plan {
            Plan::Single(k) => {
                let event = strata.events[*k];
                let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
                let (x, z) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
                let host = dithered_member(&self.geology, self.seed, &event, cx, cz, x, z);
                mixed_contents(&self.geology, &[(host, n)])
            }
            Plan::Mixed(w) => self.mixed_at(strata, w, vx, vy, vz, n),
        }
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

    /// Surface voxel height, continuous elevation, and the **year-zero fallback
    /// block** at one world voxel column — the part of the surface answer that
    /// needs no record. The near path ([`Self::column`]) reads this per voxel
    /// column for the height and derives the surface *material* from the strata
    /// record's top span (journal/0074, the surface-branch removal), so it never
    /// consults `surface_class`. The fallback block stands only where a column
    /// has no record to skin it with: subaqueous (`clastic_pass` returns early
    /// below sea level), the border wilds (no deep-time run out there —
    /// stubs.md § Genesis), and the frozen/abyssal cases. Grass is not expressed
    /// at all (ratification 4: an ecology state, not a block identity chosen by a
    /// threshold), so the vocabulary is Dirt/Stone only.
    ///
    /// Height is independent of climate, so a far sample lands on *exactly* the
    /// near column's height where the two coincide (journal/0022).
    fn surface_height(
        &mut self,
        vx: i64,
        vz: i64,
        segs: &[RiverSeg],
        temp_sl: f64,
    ) -> (i32, f64, Block) {
        let (raw, _) = self.lattice(L_VOXEL, vx, vz);
        let (elev, _riverbed) = carve_rivers(raw, vx as f64, vz as f64, segs);
        let h = (elev / self.voxel_m).floor() as i32;
        let t = temp_sl - 6.5 * elev.max(0.0) / 1000.0;
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
        (h, elev, block)
    }

    /// The far-field surface **summary** at one world voxel column: the height,
    /// and a surface block **drawn from the deep-time record** (`surface_class`
    /// → the B1 membership dither → a member of the drawn class). Consumed by
    /// the far field through [`Self::coarse_surface`], which point-samples it at
    /// a wide stride without paying a full column collapse.
    ///
    /// **This is a summary, not the authority** (journal/0074, spines § S-3). The
    /// near ground's surface voxel is the record's top span expressed through
    /// [`ColumnFill`] ([`Self::surface_voxel_contents`]); this reads the *deep*
    /// record's top-0.9 m window directly, because the far field cannot afford to
    /// run the strata passes to build a [`StrataRec`]. The two are held to a
    /// **statistical agreement test** (`coarse_surface_agrees_with_the_near_
    /// column_surface`), which replaces the journal/0055 shared-kernel structural
    /// guarantee the surface-branch removal retired. Where the far consumer's
    /// coarse sampling would alias white noise, the class draw reads the coherent
    /// bilinear source (journal/0073; spines § 4 carve-out 1).
    ///
    /// **The member dither** (journal/0058) lives in this branch: resolved
    /// per voxel column off the interpolated selection field, so a far
    /// heightfield built from this summary does not quantize into 28.8 m member
    /// patches. Since the block↔material collapse (journal/0087) the block IS the
    /// dithered member's material, so member variety is now visible in the far
    /// field too (consistent with the near field); the member's **class** stays
    /// invariant under the dither, which is what near/far agreement is held to.
    fn surface_sample(
        &mut self,
        vx: i64,
        vz: i64,
        segs: &[RiverSeg],
        fringe: bool,
        temp_sl: f64,
        precip: f64,
    ) -> SurfaceSample {
        let (h, elev, fallback) = self.surface_height(vx, vz, segs, temp_sl);
        let class = if elev > 0.0 {
            self.surface_class(vx, vz)
        } else {
            None
        };
        if let Some(class) = class {
            // **The member dither lives here** (journal/0058). Until 0058 the
            // near path resolved ONE member per 32×32 chunk footprint, drawn at
            // the chunk centre, so 28.8 m of ground shared a single member's
            // albedo and the surface quantized into hard rectilinear patches
            // (`journal/assets/0056-surface-quantized-per-chunk.png`) — the very
            // chunk-line cutover the 3c-2 boundary dither exists to prevent
            // (corrections #6, journal/0011). The buried fill never had that
            // problem because [`dithered_member`] re-picks per voxel column.
            // This is the same draw at the same salt/tag, evaluated at the
            // voxel's own fractional position instead of the chunk's centre —
            // so the contact wanders off the chunk grid, and the chunk centre
            // remains one sample of the very same field.
            //
            // It is safe to put it in the SHARED kernel — and it has to be
            // there, for the same reason the class consult is (ARCHITECTURE.md
            // § One world-answer surface). Since the block↔material collapse
            // (journal/0087) the block DOES move with the dither — it is now the
            // chosen member's own material — but the member's **class does not**,
            // so dithering within a class cannot flip which class the surface
            // reads as. (That is exactly the property journal/0055's judgment
            // call 2 could NOT rely on for mixed voxels, where a swap can flip
            // which of two classes wins a 4–4 tie. A single-member surface voxel
            // has no cross-class tie to flip.)
            let form = FormationContext {
                temp_c: temp_sl,
                precip,
                depth_m: 0.0,
            };
            let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
            let fx = (vx.rem_euclid(32) as f64 + 0.5) / 32.0;
            let fz = (vz.rem_euclid(32) as f64 + 0.5) / 32.0;
            let u = interp_corner_field(Draws::of::<GeoSelect>(self.seed), 4, cx, cz, fx, fz);
            let member = self.geology.select(class, &form, u).map(|(i, _)| i);
            let block = member
                .map(|m| Block::Material(self.geology.member(m).material))
                .or_else(|| self.class_block(class));
            if let Some(block) = block {
                return SurfaceSample {
                    h,
                    elev_m: elev,
                    block,
                    class: Some(class),
                    member,
                };
            }
        }
        // No record class resolved: the fallback block, computed once in
        // `surface_height` (subaqueous / wilds / frozen, each legitimate by
        // absence of a record).
        let _ = (fringe, precip);
        SurfaceSample {
            h,
            elev_m: elev,
            block: fallback,
            class: None,
            member: None,
        }
    }

    /// **What the record says this column is made of at the surface**: a content
    /// class **drawn** from the metre-shares of the recorded column's topmost
    /// 0.9 m. `None` where there is nothing to read.
    ///
    /// Metres, not units — the same `round Σ` discipline the rest of the slice
    /// installs. A voxel whose top 0.9 m is 40 laminae of silt and 3 of sand is
    /// weighted by the silt, regardless of how the bed *count* falls.
    ///
    /// **The verdict is a dither, not a plurality** (audit B1, S-4 move B,
    /// journal/0073). The class here is a **non-interpolable** cause — a strata
    /// unit list sampled `record_at_voxel` NEAREST at the 460 m deep-cell grid —
    /// so the legal cure for its boundary is not to threshold late (there is no
    /// continuous cause to threshold) but to **dither membership**: draw a class
    /// from the window's per-class metre shares. A window that is 55 % coarse-
    /// clastic and 45 % fine skins the two classes in shifting proportion across
    /// itself, so the frontier between two deep cells that each voted a
    /// *different* plurality winner dissolves into an interfingered gradient
    /// rather than a razor-straight 460 m square (the `0070-*-vantage`
    /// checkerboard).
    ///
    /// The draw reads the **coherent** bilinear corner-hash field
    /// ([`interp_select_draw`], the same field the surface *member* dither uses,
    /// journal/0058) at a **distinct salt** — deliberately NOT per-voxel white
    /// noise. The far field point-samples this class through `coarse_surface` at
    /// a wide stride, and white noise aliases there into a coarse speckle that
    /// doubled the far-tile mesh (journal/0073); a field coherent over a chunk
    /// forms sub-chunk class patches that mesh cheaply at every scale.
    ///
    /// **Since journal/0074 this feeds only the far-field summary** — the near
    /// ground's surface voxel is the record's top span through `ColumnFill`, not
    /// this class draw. Near and far are therefore no longer identical by
    /// construction; they are held to a **statistical** agreement test
    /// (`coarse_surface_agrees_with_the_near_column_surface`). The trade the
    /// coherence buys is a bias that **amplifies the majority** class (the
    /// bilinear value is not uniform — corrections #39 corrected the sign from
    /// the earlier "toward 50/50" reading), the same bias the member dither
    /// accepts; the unbiased end-state is the far field *summarizing* the shares,
    /// which the `CoarseField<T>` extraction owns. The **member within** the drawn
    /// class is dithered separately by the caller; this draw picks the class, that
    /// one picks the member, distinct salts throughout.
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
                let c = deep_class_of_species(u.species);
                match by_class.iter_mut().find(|(k, _)| *k == c) {
                    Some((_, m)) => *m += take,
                    None => by_class.push((c, take)),
                }
            }
            if acc >= voxel_m / 2.0 {
                // Canonical class order so the CDF the draw indexes is
                // independent of the order the recorder laid the units — the same
                // order-independence the retired plurality got from its `c < bc`
                // tie-break. `acc` is the sum of the shares, so `m / acc` is each
                // class's probability and the draw is the inverse-CDF sample.
                // Ties need no special case.
                by_class.sort_unstable_by(|a, b| a.0.cmp(b.0));
                // **A COHERENT draw, not per-voxel white noise** (journal/0073;
                // NEEDS RATIFICATION — deviates from the audit's white-noise-à-la-
                // SALT_GEO_FILL prescription). It is the *same* bilinear
                // corner-hash field the surface MEMBER dither uses (journal/0058),
                // at a DISTINCT salt. Why coherent, measured: the far field
                // POINT-SAMPLES this class through `coarse_surface` at a wide
                // stride, and per-voxel white noise aliases into coarse speckle —
                // it doubled the 1.2 km far-tile mesh (21.5 → 43.5 MiB, over the
                // per-tile budget). The bilinear field instead varies over a
                // chunk, so the class forms sub-chunk patches whose *composition*
                // shifts across the 460 m frontier: the checkerboard dissolves
                // into an interfingered gradient that meshes cheaply at every
                // scale. (Before journal/0074 this drove the near ground too, so
                // near and far were EXACTLY equal; now it is the far summary only
                // and near/far agreement is statistical.) Cost of coherence: the
                // bilinear value is not uniform, so the split is biased to
                // **amplify the majority** class (corrections #39 corrected the
                // sign from the earlier "toward 50/50") — the identical bias the
                // member dither already lives with (0058). Unbiased white noise is
                // the correct end-state once the far field *summarizes* the share
                // vector instead of point-sampling it — that lives in the
                // `CoarseField<T>` extraction (audit Part 2).
                let (ccx, ccz) = (vx.div_euclid(32), vz.div_euclid(32));
                let fx = (vx.rem_euclid(32) as f64 + 0.5) / 32.0;
                let fz = (vz.rem_euclid(32) as f64 + 0.5) / 32.0;
                let u = interp_corner_field(Draws::of::<GeoClass>(self.seed), 0, ccx, ccz, fx, fz);
                return draw_class(&by_class, acc, u);
            }
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

    /// The block a content class fronts with when no member draw resolved, via
    /// its first (id-sorted) member's material. Since the collapse (journal/0087)
    /// a block is a material, so this is the first member's own identity — the
    /// fallback the far surface shows where the class is known but the member
    /// draw came up empty.
    fn class_block(&self, class: &str) -> Option<Block> {
        let m = *self.geology.class(class)?.members().first()?;
        Some(Block::Material(self.geology.member(m).material))
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
            let u = Draws::of::<Elev>(self.seed)
                .unit(&[u64::from(level), i as u64, j as u64])
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
        RegionBase { civilized, segs }
    }

    fn locale(&mut self, lx: i64, lz: i64) -> Arc<LocaleRec> {
        self.trace.locales.insert((lx, lz));
        if let Some(l) = self.locale_cache.get(&(lx, lz)) {
            return l.clone();
        }
        let base = self.locale_base(lx, lz);
        let rec = Arc::new(LocaleRec {
            fringe: base.fringe,
            segs: base.segs,
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
        LocaleBase {
            fringe: parent.fringe,
            segs,
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
        // Eighths of the surface voxel the ground fills — the top-of-column
        // remainder. The surface *material* is derived below, from the record's
        // top span, once the strata passes have run (journal/0074): the near path
        // no longer consults `surface_class` at all.
        let mut surface_eighths = vec![0u8; 1024];
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
                // Height + the year-zero **fallback** block only. No deep-record
                // class consult in the near path (journal/0074): the surface
                // material is the record's top span, derived below. The fallback
                // stands only where there is no record to skin the column with.
                let (h, elev_m, fallback) = self.surface_height(vx, vz, &locale.segs, temp_sl);
                heights[i] = h;
                surface[i] = fallback;
                // **The top-of-column remainder** (journal/0055): `h` floors the
                // continuous surface, so the surface voxel is filled from its
                // floor up to the real ground — a genuine partial. Ceil, and at
                // least one eighth: `h = floor(elev/0.9)` means there IS ground
                // in this voxel, and rounding it away would silently drop a
                // voxel of world height.
                let frac = (elev_m - f64::from(h) * self.voxel_m) / self.voxel_m;
                surface_eighths[i] = (frac * 8.0).ceil().clamp(1.0, 8.0) as u8;
            }
        }
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
        // **The weathering fold** (the first-real-behavior slice): read the deep
        // cell's `base + facts` weathering product (metres of loose regolith the
        // bedrock seam yielded). `0.0` unless `weather_inventory` is on — the S-5
        // identity default. The ledger is sampled at the SAME nearest cell as
        // `deep_units`, so its bedrock slot index is `deep_units.len()`.
        let deep_weathering_m = self
            .pregen
            .deep
            .ledger_at_voxel(cx * 32 + 16, cz * 32 + 16)
            .map_or(0.0, |l| l.weathering_product_m(deep_units.len()));
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
            deep_weathering_m,
            wilds,
            geology: &self.geology,
            providers: self.providers,
            strata: StrataRec::default(),
            alluvium: None,
        };
        self.pregen.pipeline.run_strata(&mut strata_ctx);
        let strata = strata_ctx.strata;

        // **Skin the surface voxel from the record's top span** (journal/0074 —
        // the surface-branch removal). The surface voxel is [`ColumnFill`]
        // `plan(1)`, the record's topmost span, expressed as the per-column
        // partial — the SAME span, through the SAME fill machinery, that the
        // block and material paths resolve every buried voxel from. There is no
        // parallel "what is the surface made of" computation any more: the
        // history the strata passes just wrote IS the surface. `block ==
        // classify(contents)` holds by construction — the block is set to
        // `classify` of exactly the contents [`Self::material_ids`] will intern.
        // Columns with no record top span (subaqueous, wilds, bare-province)
        // keep the year-zero fallback block set above and carry no contents (the
        // shrinking absent-contents exception).
        let fill = ColumnFill::build(&strata, self.voxel_m);
        if let Some(plan) = fill.plan(1) {
            for z in 0..32i64 {
                for x in 0..32i64 {
                    let i = (z * 32 + x) as usize;
                    let (vx, vz) = (cx * 32 + x, cz * 32 + z);
                    let vy = i64::from(heights[i]);
                    let contents =
                        self.surface_voxel_contents(&strata, plan, surface_eighths[i], vx, vy, vz);
                    surface[i] = classify(&contents);
                }
            }
        }

        let rec = Arc::new(ColumnRec {
            heights,
            surface,
            soil,
            wilds,
            strata,
            surface_eighths,
        });
        self.column_cache.insert((cx, cz), rec.clone());
        rec
    }
}

fn avg2(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// **The S-4 move-B membership draw** (audit B1): pick one class from the
/// surface window's per-class metre shares by inverse-CDF against one addressed
/// uniform `u ∈ [0, 1)`. `shares` must already be in canonical (class-id) order
/// so the CDF is independent of recorder order; `total` is `Σ shares` (the
/// caller's `acc`), so `m / total` is each class's probability. Returns `None`
/// only for an empty slice — the caller never calls it in that case.
///
/// Unbiased by the same argument `crate::fill::allocate` rests on: averaged over
/// the draw, class `i` is chosen with probability exactly `share_i`, so a
/// neighbourhood's expected class composition equals the recorded composition.
/// The plurality it replaces was a biased estimator that gave the whole cell to
/// one winner and stepped hard at the neighbour that voted the other way.
fn draw_class(shares: &[(&'static str, f64)], total: f64, u: f64) -> Option<&'static str> {
    let mut cum = 0.0f64;
    for &(c, m) in shares {
        cum += m / total;
        if u < cum {
            return Some(c);
        }
    }
    // The shares sum to 1 by construction; a floating-point residue at the very
    // top of the unit interval lands on the last class.
    shares.last().map(|(c, _)| *c)
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
        let k = eighths.clamp(1, 7);
        let acc = set.member(acc_member).material;
        if k >= 5 && crate::fill::is_loose(set, acc_member) {
            // **Loose-dominant pore rider** (journal/0099): past half a voxel of
            // loose product there is no rock skeleton left to call structure —
            // the parent survives as *clasts* in the product, which is what the
            // top of a weathering front (grus, corestones in clay) is. This is
            // the same ≥4/8 rule [`crate::fill::mixed_contents`] applies at a
            // contact, so the two expression paths cannot disagree about the form
            // of the same band. Unreachable before 0099: the only pore rider was
            // the igneous accessory, always 1/8 and never loose.
            let mut debris = [acc; 8];
            for slot in debris.iter_mut().take(usize::from(8 - k)) {
                *slot = host_mat;
            }
            VoxelContents::debris_only(&debris).expect("8 debris eighths fit an open voxel")
        } else {
            // Host rock in the structure slots; the rider in its pores.
            let structure = [host_mat; 8];
            let pore = [acc; 8];
            VoxelContents::new(
                StructureShape::Full,
                &structure[..usize::from(8 - k)],
                &pore[..usize::from(k)],
                &[],
            )
            .expect("host structure + accessory pore fill is canonical")
        }
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
        self, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, FormationWindow, GeoHabit, GeoMemberDef,
        GeoMemberIdx, GeologySet,
    };
    use dc_core::{Block, MaterialId, classify};

    use super::contents_for_event;
    use crate::WorldGenerator;
    use crate::geology::StrataEvent;
    use crate::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams, temp_sea_level};

    /// The block a member's event must classify to: since the block↔material
    /// collapse (journal/0087) that is the member's **own material** — a block IS
    /// a material. The regression this guards is the fill contract's dominance
    /// rule: no enrichment shape (placer ore ≤3/8, accessory pore inclusion) may
    /// usurp the structural/host member, so `classify(contents_for_event(..))`
    /// wears the member's own identity for every member of every registered set.
    /// (Before 2026-07-21 a parallel class-to-block table computed this; the
    /// collapse retired even the class summary in between.)
    fn block_for_member(set: &GeologySet, member: GeoMemberIdx) -> Block {
        Block::Material(set.member(member).material)
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

    /// The far horizon must **statistically agree** with the ground on what the
    /// surface is made of — the S-7 register the octree node contract sanctions
    /// (docs/design/octree-substrate.md; FF2b's +0.938-within-1 precedent,
    /// journal/0070).
    ///
    /// **This test changed meaning with journal/0074 (the surface-branch
    /// removal).** Before: the surface block came from the *shared*
    /// `surface_sample` kernel, so near and far were equal **by construction**,
    /// and this test asserted exact equality. Now the near ground's surface voxel
    /// is the record's top span expressed through [`ColumnFill`] (the real
    /// authority — [`WorldGenerator::surface_voxel_contents`]), while the far
    /// field's [`WorldGenerator::coarse_surface`] reads the deep-record top
    /// window directly as a **summary** (it cannot afford to run the strata
    /// passes). The journal/0055 structural guarantee is retired; this
    /// **statistical agreement** replaces it, and it is deliberately strong
    /// enough that a future drift between horizon and ground fails loudly.
    ///
    /// The two do not agree exactly because (a) the near path routes the *veneer*
    /// (clastic fan) on top of the deep history where the far window sees only
    /// the deep record, and (b) the far class draw carries the coherent-source
    /// bias (majority-amplifying, corrections #39; spines § 4 carve-out 1) — a
    /// sign *favourable* to this agreement, so (a) is the disagreement's source.
    /// They must still agree on the **large majority** of land columns, and where
    /// they disagree it must be the minority-class / veneer boundary, not a
    /// systematic split.
    #[test]
    fn coarse_surface_agrees_with_the_near_column_surface() {
        // Only land columns that surface a geology material on both sides carry
        // information; the Dirt/Stone fallback vocabulary is shared trivially and
        // would inflate the agreement. Restrict to those, and — since the
        // block↔material collapse (journal/0087) — compare at **content-class**
        // granularity: a block now carries its dithered MEMBER's material, and
        // near (the record's top span through `ColumnFill`) and far (the coarse
        // window draw) legitimately dither *different members within a class*
        // (journal/0074). The invariant that must hold is agreement on the
        // class the surface is made of, exactly what the retired `block_twin`
        // summarized to — not which member won the within-class dither.
        let geo_class = |b: Block| -> Option<u8> {
            let Block::Material(m) = b else { return None };
            let r = m.raw();
            let is = |x: MaterialId| x.raw() == r;
            if is(MaterialId::MUDSTONE)
                || is(MaterialId::SILTSTONE)
                || is(MaterialId::SILT)
                || is(MaterialId::CLAY)
            {
                Some(0) // fine clastic
            } else if is(MaterialId::SANDSTONE)
                || is(MaterialId::CONGLOMERATE)
                || is(MaterialId::SAND)
                || is(MaterialId::GRAVEL)
            {
                Some(1) // coarse clastic
            } else if is(MaterialId::GRANITE) || is(MaterialId::DIORITE) {
                Some(2) // igneous intrusive
            } else if is(MaterialId::BASALT) || is(MaterialId::ANDESITE) {
                Some(3) // igneous extrusive
            } else if is(MaterialId::COAL) || is(MaterialId::CHARCOAL) {
                Some(4) // organic coal band
            } else if is(MaterialId::PEAT) {
                Some(5)
            } else if is(MaterialId::CARBONACEOUS_MUDSTONE) {
                Some(6)
            } else {
                None // loose accessory / soil: not a geology surface class
            }
        };
        let (mut agree, mut compared) = (0usize, 0usize);
        for seed in [0x0D5E_ED57_2026u64, 1337] {
            let pregen = Pregen::run(WorldParams {
                seed,
                extent: Extent::Medium,
            });
            let mut g = WorldGenerator::new(&pregen);
            for cx in -6..=6i64 {
                for cz in -6..=6i64 {
                    let col = g.column_record(cx, cz);
                    for lz in 0..32usize {
                        for lx in 0..32usize {
                            let (vx, vz) = (cx * 32 + lx as i64, cz * 32 + lz as i64);
                            let near = col.surface[lz * 32 + lx];
                            let (_h, far) = g.coarse_surface(vx, vz);
                            // Compare only where the record surfaces a geology
                            // class on both sides (a shared fallback is not
                            // agreement about anything).
                            let (Some(nc), Some(fc)) = (geo_class(near), geo_class(far)) else {
                                continue;
                            };
                            compared += 1;
                            if nc == fc {
                                agree += 1;
                            }
                        }
                    }
                }
            }
        }
        let frac = agree as f64 / compared as f64;
        println!(
            "near/coarse surface agreement: {agree}/{compared} = {:.4} \
             (geology-surfacing columns, seeds 0x0D5EED572026 + 1337, Medium)",
            frac
        );
        assert!(
            compared > 20_000,
            "only {compared} geology-surfacing columns"
        );
        // Floor set from the measured agreement (journal/0074) with margin, so a
        // real drift between horizon and ground trips it. NOT a by-construction
        // equality any more — see the doc comment.
        //
        // **RE-DERIVED 2026-07-26 (journal/0112, material-aware hillslope creep):
        // 0.88 → 0.80, measured 0.9171 → 0.8225.** Read the number before reading
        // the change: both sides of this comparison are **unbiased draws from the
        // same window shares**, so the agreement rate is bounded above by the
        // window's own **homogeneity**, not by the quality of the summary. The old
        // floor was set against a record that was **91 % fine clastic** — where two
        // independent draws agree most of the time for free. Creep carrying
        // identity redistributed the archive to 26 % fine / 36 % coarse / 38 %
        // carbonaceous soil and nearly doubled the distinct species in a hillslope
        // column (1.091 → 1.952), so the same mechanism now has real entropy to
        // summarize. The collision floor for two *independent* draws on the new
        // composition is ≈0.34; the measured 0.8225 is far above it, which is the
        // evidence that the summary still tracks the ground rather than having
        // drifted from it.
        //
        // **This is a snapshot masquerading as an invariant, and the heir should
        // retire it** (CLAUDE.md: *assert invariants, never snapshots* — a test
        // pinned to yesterday's number fails because a colleague changed the world,
        // which is worse than the defect it guards). The honest assertion is
        // **unbiasedness**: that the far draw's class distribution matches the
        // near expression's, per class, rather than that they coincide per voxel.
        // Filed as a needs-measurement item with journal/0112; the floor is kept
        // meanwhile because a *biased* summary would collapse it far below 0.80.
        assert!(
            frac >= 0.80,
            "near/coarse surface agreement {frac:.4} below floor 0.80 — the far \
             summary has drifted from the ground expression it summarizes"
        );
    }

    /// **The surface voxel routes through `ColumnFill` and is dithered per
    /// voxel** (journal/0074, the surface-branch removal). The surface voxel is
    /// the record's top span (`plan(1)`) through the SAME fill machinery as every
    /// buried voxel, so it inherits that machinery's per-voxel addressed
    /// allocation — the world's skin does not quantize into 28.8 m chunk patches.
    ///
    /// Two claims:
    ///
    /// 1. **`block == classify(contents)` at the surface**: [`Self::column`] sets
    ///    the surface block to `classify` of exactly the contents
    ///    [`Self::surface_voxel_contents`] builds; this recomputes those contents
    ///    and asserts the block matches — the S-3 routing proof at the surface
    ///    voxel specifically (contents-contract proves it globally).
    /// 2. **The dither is live**: for a `Mixed` top span (the common case since
    ///    journal/0055), the per-voxel allocation gives the surface voxel varying
    ///    contents across the 32×32 footprint — a revert to a chunk-quantized
    ///    surface would make every column identical.
    #[test]
    fn surface_voxel_routes_through_columnfill_per_voxel() {
        use crate::fill::{ColumnFill, Plan};
        use dc_core::VoxelContents;
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        });
        let mut g = WorldGenerator::new(&pregen);
        let (mut mixed_top, mut varied) = (0usize, 0usize);
        for cx in -8..=8i64 {
            for cz in -8..=8i64 {
                let col = g.column_record(cx, cz);
                let fill = ColumnFill::build(&col.strata, 0.9);
                // Mixed top spans exercise the per-voxel allocation; Single spans
                // (vanishingly rare post-0055) go through `dithered_member`.
                let plan = match fill.plan(1) {
                    Some(p @ Plan::Mixed(_)) => p,
                    _ => continue,
                };
                mixed_top += 1;
                let mut seen: Vec<VoxelContents> = Vec::new();
                for z in 0..32i64 {
                    for x in 0..32i64 {
                        let i = (z * 32 + x) as usize;
                        let (vx, vz) = (cx * 32 + x, cz * 32 + z);
                        let vy = i64::from(col.heights[i]);
                        let c = g.surface_voxel_contents(
                            &col.strata,
                            plan,
                            col.surface_eighths[i],
                            vx,
                            vy,
                            vz,
                        );
                        // The S-3 routing proof at the surface: the block IS
                        // classify of the top-span contents, not a parallel paint.
                        assert_eq!(
                            col.surface[i],
                            classify(&c),
                            "chunk ({cx},{cz}) column {i}: surface block {:?} != classify of \
                             its top-span contents — the surface is not routed through the fill",
                            col.surface[i],
                        );
                        if !seen.contains(&c) {
                            seen.push(c);
                        }
                    }
                }
                if seen.len() > 1 {
                    varied += 1;
                }
            }
        }
        println!(
            "surface routing: {varied} of {mixed_top} mixed-top-span chunks vary the \
             surface voxel contents per position (per-voxel allocation is live)"
        );
        assert!(
            mixed_top > 20,
            "only {mixed_top} mixed-top-span chunks sampled"
        );
        // Per-voxel addressed allocation of a genuine multi-member span varies
        // the contents across the footprint; a chunk-quantized surface gives 0.
        assert!(
            varied * 2 >= mixed_top,
            "only {varied} of {mixed_top} mixed-top-span chunks vary the surface voxel per \
             position — the surface is not sharing the per-voxel fill dither"
        );
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

    /// **Move B is unbiased** (audit B1, journal/0073): the class-membership
    /// draw, averaged over the addressed uniform, chooses each class with
    /// frequency equal to its recorded metre share. Same proof shape as
    /// `fill::allocation_is_unbiased_over_the_draw` — this is the whole argument
    /// for a dither over the retired plurality, which gave a 55/45 window's
    /// entire 460 m cell to the 55 % class and stepped hard at its neighbour.
    #[test]
    fn draw_class_is_unbiased_over_the_draw() {
        use std::collections::HashMap;
        // `total` is `Σ shares` (the caller's `acc`); the third case is
        // un-normalized (metres, not fractions) to exercise `m / total`.
        let cases: Vec<(Vec<(&'static str, f64)>, f64)> = vec![
            (vec![("a", 0.55), ("b", 0.45)], 1.0),
            (vec![("a", 0.50), ("b", 0.30), ("c", 0.20)], 1.0),
            (vec![("a", 0.09), ("b", 0.81)], 0.90),
        ];
        for (shares, total) in &cases {
            const N: usize = 20_000;
            let mut hits: HashMap<&str, usize> = HashMap::new();
            for k in 0..N {
                let u = (k as f64 + 0.5) / N as f64;
                let c = super::draw_class(shares, *total, u).expect("non-empty window");
                *hits.entry(c).or_default() += 1;
            }
            for &(c, m) in shares {
                let freq = *hits.get(c).unwrap_or(&0) as f64 / N as f64;
                let want = m / total;
                assert!(
                    (freq - want).abs() < 1e-2,
                    "class {c}: chosen {freq} of the time, recorded share {want}"
                );
            }
        }
    }

    /// **The class-membership dither (`draw_class`) is live in the far summary**
    /// (audit B1, journal/0073; re-homed by journal/0074, the surface-branch
    /// removal). `draw_class` no longer paints the near ground — that is the
    /// record's top span through [`ColumnFill`] now — it is the far field's cheap
    /// class summary (`coarse_surface` → `surface_class` → `draw_class`), the
    /// thing whose S-4 checkerboard cure B1 shipped.
    ///
    /// A deep cell's top-window class shares are constant across the whole cell
    /// (`record_at_voxel` is NEAREST at the 460 m grid). Since the block↔material
    /// collapse (journal/0087) the far block carries the dithered MEMBER's
    /// material, so this test tracks the surfaced **class** (the material's
    /// content class), not the raw block — otherwise the within-class member
    /// dither would masquerade as a class split. If two *far* samples of the same
    /// deep cell surface two different *classes*, that split can only be the class
    /// membership dither. The retired plurality would give exactly 0 splits.
    /// Grouping is by the record's pointer identity (same cell ⇒ same rec).
    #[test]
    fn far_surface_class_dither_splits_multiclass_deep_cells() {
        use std::collections::HashMap;
        let pregen = Pregen::run(WorldParams {
            seed: 1337,
            extent: Extent::Medium,
        });
        let mut g = WorldGenerator::new(&pregen);
        // The surfaced material's content class — NOT the Air/Dirt/Stone fallback
        // vocabulary, NOT ruin Wood. Intra-cell variation among these classes is
        // the class dither (member variation within one class is NOT).
        let geo_class = |b: Block| -> Option<u8> {
            let Block::Material(m) = b else { return None };
            let r = m.raw();
            let is = |x: MaterialId| x.raw() == r;
            if is(MaterialId::MUDSTONE)
                || is(MaterialId::SILTSTONE)
                || is(MaterialId::SILT)
                || is(MaterialId::CLAY)
            {
                Some(0)
            } else if is(MaterialId::SANDSTONE)
                || is(MaterialId::CONGLOMERATE)
                || is(MaterialId::SAND)
                || is(MaterialId::GRAVEL)
            {
                Some(1)
            } else if is(MaterialId::GRANITE) || is(MaterialId::DIORITE) {
                Some(2)
            } else if is(MaterialId::BASALT) || is(MaterialId::ANDESITE) {
                Some(3)
            } else if is(MaterialId::COAL) || is(MaterialId::CHARCOAL) {
                Some(4)
            } else if is(MaterialId::PEAT) {
                Some(5)
            } else if is(MaterialId::CARBONACEOUS_MUDSTONE) {
                Some(6)
            } else {
                None
            }
        };
        // A 460 m deep cell is ~511 voxels ≈ 16 chunks wide. Spread the sample
        // chunks at a ~one-cell stride so each lands in a *distinct* deep cell
        // over a wide area, then sample the FAR summary (`coarse_surface`) across
        // each chunk's columns — a multi-class cell reveals its split.
        let mut by_cell: HashMap<usize, Vec<u8>> = HashMap::new();
        for i in 0..15i64 {
            for j in 0..15i64 {
                let (cx, cz) = (i * 17 - 120, j * 17 - 120);
                for lz in 0..32i64 {
                    for lx in 0..32i64 {
                        let (vx, vz) = (cx * 32 + lx, cz * 32 + lz);
                        let (_h, b) = g.coarse_surface(vx, vz);
                        let Some(class) = geo_class(b) else {
                            continue;
                        };
                        let Some(rec) = pregen.deep.record_at_voxel(vx, vz) else {
                            continue;
                        };
                        let key = std::ptr::from_ref(rec) as usize;
                        let seen = by_cell.entry(key).or_default();
                        if !seen.contains(&class) {
                            seen.push(class);
                        }
                    }
                }
            }
        }
        let cells = by_cell.len();
        let split = by_cell.values().filter(|s| s.len() > 1).count();
        println!(
            "far class dither: {split} of {cells} sampled deep cells surface >1 geology class"
        );
        assert!(
            cells > 20,
            "only {cells} deep cells sampled a geology surface"
        );
        // The plurality signature is exactly 0 (a deep cell's shares are
        // constant, so plurality → one class → one block for the whole cell).
        // The floor guards a revert without being brittle to world drift.
        assert!(
            split >= 20,
            "only {split} of {cells} deep cells surface more than one geology class — \
             the far class membership dither is not live (plurality would give exactly 0)"
        );
    }
}
