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
use std::rc::Rc;

use dc_core::{Block, Chunk, ChunkPos, VoxelScale};
use dc_sim::statistical::rng::draw_f64;

use crate::pregen::{CELL_VOXELS, Pregen, SALT_ELEV, SALT_RUIN, temp_sea_level};

/// Lattice level whose spacing is one region (8 192 voxels, 7.37 km).
pub const L_REGION: u8 = 1;
/// Lattice level whose spacing is one locale (512 voxels, 460.8 m).
pub const L_LOCALE: u8 = 5;
/// Lattice level whose spacing is one chunk-column (32 voxels, 28.8 m).
pub const L_COLUMN: u8 = 9;
/// Finest lattice level: one voxel (0.9 m).
pub const L_VOXEL: u8 = 14;

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
}

/// The lazy generator: owns the collapse caches and instrumentation, borrows
/// the pregen output. All state is derived — dropping a cache entry can never
/// change any answer (everything is a pure function of seed + pregen).
pub struct WorldGenerator<'a> {
    pregen: &'a Pregen,
    seed: u64,
    voxel_m: f64,
    sites_by_cell: HashMap<(i32, i32), Vec<SiteSpot>>,
    lattice_memo: HashMap<(u8, i64, i64), (f64, f64)>,
    region_cache: HashMap<(i64, i64), Rc<RegionRec>>,
    locale_cache: HashMap<(i64, i64), Rc<LocaleRec>>,
    column_cache: HashMap<(i64, i64), Rc<ColumnRec>>,
    trace: Trace,
    last_stats: ChunkStats,
}

impl<'a> WorldGenerator<'a> {
    pub fn new(pregen: &'a Pregen) -> Self {
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
        Self {
            pregen,
            seed: pregen.seed,
            voxel_m: scale.voxel_size_m(),
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

    /// Generate one 32³ chunk of blocks. Deterministic in
    /// `(seed, extent, pos)`; asserts [`LOOKAHEAD_BOUNDS`].
    pub fn generate_chunk(&mut self, pos: ChunkPos) -> Chunk {
        self.trace.clear();
        let col = self.column(i64::from(pos.x), i64::from(pos.z));
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
                    } else if vy >= h - i64::from(col.soil) {
                        Block::Dirt
                    } else {
                        Block::Stone
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

    /// The chunk-column record for chunk coordinates `(cx, cz)` — exposed for
    /// the seam/continuity tests, which reason about surface heights.
    pub fn column_record(&mut self, cx: i64, cz: i64) -> Rc<ColumnRec> {
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
        let v = if level == 0 {
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

    fn region(&mut self, rx: i64, rz: i64) -> Rc<RegionRec> {
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
        let rec = Rc::new(RegionRec {
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

    fn locale(&mut self, lx: i64, lz: i64) -> Rc<LocaleRec> {
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
        let rec = Rc::new(LocaleRec {
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
    fn column(&mut self, cx: i64, cz: i64) -> Rc<ColumnRec> {
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
        let rec = Rc::new(ColumnRec {
            heights,
            surface,
            soil,
            posts,
            wilds,
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
