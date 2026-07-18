//! S8 measurements: does the 8-eighths mixed-material model survive palette
//! compression under realistic deposition?
//!
//! Run with `cargo test -p dc-core --release --test s8_measurements -- --nocapture`
//! to see the tables; recorded numbers live in docs/spikes/S8-results.md.
//!
//! Everything here is seeded and deterministic — no wall clock, no ambient
//! entropy. The terrain generator reuses the S1 spike's noise parameters
//! (dc-client/src/worldgen.rs, read-only copy) so deposits form on the same
//! landscape family every other spike measured. The deposition processes are
//! deliberately simple local rules — plausible, not physical: the question is
//! the *distinct-state statistics* of correlated deposition, not sediment
//! science.
//!
//! Model: per (x,z) column, deposits are a bottom-up stack of eighth-layers
//! starting at the terrain surface (rounded up to an eighth boundary). Voxels
//! slice the stack on the world grid, so mixtures appear naturally where
//! layers of different materials straddle a voxel — exactly the alignment
//! effect the palette must survive.

use std::collections::HashMap;

use dc_core::materials::contents::VoxelContents;
use dc_core::materials::intern::{MaterialChunk, MixtureId, MixtureTable};
use dc_core::materials::{MATERIAL_COUNT, MaterialId};
use dc_core::{CHUNK_VOLUME, Chunk, ChunkPos, local_voxel};
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};

const SEED: u64 = 0x58_5EED;
/// Region: 8x8 chunk-columns = 256x256 voxel columns at 0.9 m.
const COLS: usize = 256;
const VOXEL_M: f64 = 0.9;
const EIGHTH_M: f64 = VOXEL_M / 8.0;
/// Cubic meters per 32^3 chunk at the S1 scale.
const CHUNK_M3: f64 = 32.0 * 32.0 * 32.0 * VOXEL_M * VOXEL_M * VOXEL_M;
/// S3 baselines (docs/spikes/S3-results.md).
const BASELINE_PALETTE_B_M3: f64 = 0.144;
const BASELINE_RAW_B_M3: f64 = 2.74;

// ---------------------------------------------------------------------------
// Deterministic rng (splitmix64 stream, as in dc-sim's rng module)
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Pick from weighted alternatives; weights need not normalize.
    fn pick(&mut self, weighted: &[(MaterialId, f64)]) -> MaterialId {
        let total: f64 = weighted.iter().map(|(_, w)| w).sum();
        let mut roll = self.f64() * total;
        for &(m, w) in weighted {
            if roll < w {
                return m;
            }
            roll -= w;
        }
        weighted.last().expect("non-empty weights").0
    }
}

// ---------------------------------------------------------------------------
// Terrain (S1 noise parameters, read-only copy of dc-client/src/worldgen.rs)
// ---------------------------------------------------------------------------

struct Terrain {
    hills: FastNoiseLite,
    chasm: FastNoiseLite,
}

impl Terrain {
    fn new(seed: i32) -> Self {
        let mut hills = FastNoiseLite::with_seed(seed);
        hills.set_noise_type(Some(NoiseType::OpenSimplex2));
        hills.set_fractal_type(Some(FractalType::FBm));
        hills.set_fractal_octaves(Some(4));
        hills.set_frequency(Some(0.008));
        let mut chasm = FastNoiseLite::with_seed(seed.wrapping_add(1));
        chasm.set_noise_type(Some(NoiseType::OpenSimplex2));
        chasm.set_frequency(Some(0.003));
        Self { hills, chasm }
    }

    /// Surface height in meters (S1's hills + carved ravines; caves omitted —
    /// deposition happens on the surface).
    fn surface_height_m(&self, xm: f64, zm: f64) -> f64 {
        const BASE_HEIGHT_M: f64 = 8.0;
        const HILL_AMP_M: f64 = 14.0;
        const CHASM_DEPTH_M: f64 = 90.0;
        const CHASM_HALF_WIDTH: f64 = 0.10;
        let h = f64::from(self.hills.get_noise_2d(xm as f32, zm as f32));
        let base = BASE_HEIGHT_M + h * HILL_AMP_M;
        let c = f64::from(self.chasm.get_noise_2d(xm as f32, zm as f32));
        let t = (1.0 - c.abs() / CHASM_HALF_WIDTH).clamp(0.0, 1.0);
        base - t * t * (3.0 - 2.0 * t) * CHASM_DEPTH_M
    }
}

// ---------------------------------------------------------------------------
// The deposition field
// ---------------------------------------------------------------------------

struct Field {
    /// World-voxel coordinate of column (0,0) — chunk-aligned.
    origin_vx: i64,
    origin_vz: i64,
    /// Per column: absolute eighth index (world y in eighth-of-voxel units)
    /// of the first free eighth above the terrain surface.
    surface_eighth: Vec<i64>,
    /// Per column: deposited eighth-layers, bottom-up, raw material ids.
    stacks: Vec<Vec<u8>>,
}

impl Field {
    fn new(terrain: &Terrain, origin_vx: i64, origin_vz: i64) -> Self {
        let mut surface_eighth = Vec::with_capacity(COLS * COLS);
        for z in 0..COLS {
            for x in 0..COLS {
                let xm = (origin_vx + x as i64) as f64 * VOXEL_M + VOXEL_M / 2.0;
                let zm = (origin_vz + z as i64) as f64 * VOXEL_M + VOXEL_M / 2.0;
                let h = terrain.surface_height_m(xm, zm);
                surface_eighth.push((h / EIGHTH_M).ceil() as i64);
            }
        }
        Self {
            origin_vx,
            origin_vz,
            surface_eighth,
            stacks: vec![Vec::new(); COLS * COLS],
        }
    }

    #[inline]
    fn idx(x: usize, z: usize) -> usize {
        x + z * COLS
    }

    /// Top of the column (surface + deposits), in absolute eighths.
    #[inline]
    fn top(&self, c: usize) -> i64 {
        self.surface_eighth[c] + self.stacks[c].len() as i64
    }

    fn deposit(&mut self, c: usize, m: MaterialId) {
        self.stacks[c].push(m.raw());
    }

    /// 4-neighborhood, clamped at the region edge (missing neighbors don't
    /// exist rather than reading as cliffs).
    fn neighbors(x: usize, z: usize) -> impl Iterator<Item = usize> {
        let x = x as isize;
        let z = z as isize;
        [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)]
            .into_iter()
            .filter(|&(nx, nz)| nx >= 0 && nz >= 0 && (nx as usize) < COLS && (nz as usize) < COLS)
            .map(|(nx, nz)| Field::idx(nx as usize, nz as usize))
    }

    /// Angle-of-repose threshold for a material, in eighths per column step
    /// (0.9 m horizontal): cohesive materials hold steeper piles.
    fn repose_eighths(m: MaterialId) -> i64 {
        4 + (m.props().cohesion * 20.0) as i64
    }

    /// One relaxation sweep: any column overtopping its lowest neighbor by
    /// more than the top material's repose threshold sheds one eighth
    /// downhill. Fixed scan order keeps it deterministic. Returns moves made.
    fn relax_sweep(&mut self) -> usize {
        let mut moves = 0;
        for z in 0..COLS {
            for x in 0..COLS {
                let c = Field::idx(x, z);
                let Some(&top_raw) = self.stacks[c].last() else {
                    continue;
                };
                let m = MaterialId::from_raw(top_raw).expect("valid id");
                let my_top = self.top(c);
                let Some(lowest) = Field::neighbors(x, z).min_by_key(|&n| self.top(n)) else {
                    continue;
                };
                if my_top - self.top(lowest) > Field::repose_eighths(m) {
                    self.stacks[c].pop();
                    self.stacks[lowest].push(top_raw);
                    moves += 1;
                }
            }
        }
        moves
    }

    fn relax(&mut self, max_sweeps: usize) {
        for _ in 0..max_sweeps {
            if self.relax_sweep() == 0 {
                break;
            }
        }
    }

    fn total_eighths(&self) -> usize {
        self.stacks.iter().map(Vec::len).sum()
    }

    /// Deterministic content hash for reproducibility checks.
    fn fingerprint(&self) -> u64 {
        let mut h = Rng::new(0);
        let mut acc = 0u64;
        for s in &self.stacks {
            h.0 = acc ^ (s.len() as u64);
            for &b in s {
                h.0 = h.0.wrapping_mul(31).wrapping_add(u64::from(b));
            }
            acc = h.next_u64();
        }
        acc
    }
}

// ---------------------------------------------------------------------------
// Deposition processes
// ---------------------------------------------------------------------------

/// (a) Wind-blown accumulation: snow/sand events; sheltered columns (below
/// their neighborhood) accumulate by exposure; exposed crests erode downwind.
fn wind_pass(field: &mut Field, rng: &mut Rng, pass: usize) {
    // Storm material: runs of snow with sand interludes.
    let m = if (pass / 8) % 3 == 2 {
        MaterialId::SAND
    } else {
        MaterialId::SNOW
    };
    for z in 0..COLS {
        for x in 0..COLS {
            let c = Field::idx(x, z);
            let my_top = field.top(c);
            let (mut sum, mut n) = (0i64, 0i64);
            for nb in Field::neighbors(x, z) {
                sum += field.top(nb);
                n += 1;
            }
            let mean = sum / n.max(1);
            let shelter = mean - my_top; // positive = hollow
            let p = if shelter > 0 {
                (shelter as f64 * 0.06).min(0.8)
            } else {
                0.02 // light fallout everywhere
            };
            if rng.f64() < p {
                field.deposit(c, m);
            }
        }
    }
    // Transport: crests shed downwind (+x) when loose material is on top.
    for z in 0..COLS {
        for x in 0..COLS.saturating_sub(1) {
            let c = Field::idx(x, z);
            let downwind = Field::idx(x + 1, z);
            if field.top(c) > field.top(downwind) + 3
                && let Some(&raw) = field.stacks[c].last()
            {
                let m = MaterialId::from_raw(raw).expect("valid id");
                if m.props().cohesion < 0.4 && rng.f64() < 0.5 {
                    field.stacks[c].pop();
                    field.stacks[downwind].push(raw);
                }
            }
        }
    }
    field.relax(2);
}

/// (b) Rockfall/scree: cliff bases (steep terrain drop) receive falling
/// scree/gravel that walks downhill to its resting point; repose relaxation
/// builds the talus apron.
fn rockfall_pass(field: &mut Field, rng: &mut Rng, sources: &[usize]) {
    for &src in sources {
        let m = if rng.f64() < 0.7 {
            MaterialId::SCREE
        } else {
            MaterialId::GRAVEL
        };
        // Fall downhill from the source to a locally stable column.
        let mut cur = src;
        for _ in 0..64 {
            let x = cur % COLS;
            let z = cur / COLS;
            let Some(lowest) = Field::neighbors(x, z).min_by_key(|&n| field.top(n)) else {
                break;
            };
            if field.top(cur) - field.top(lowest) > Field::repose_eighths(m) {
                cur = lowest;
            } else {
                break;
            }
        }
        field.deposit(cur, m);
    }
    field.relax(2);
}

/// (c) Midden growth: a settlement dumps mixed refuse at a point with small
/// scatter; cohesionless refuse slumps into a low mound.
fn midden_pass(field: &mut Field, rng: &mut Rng, center: (usize, usize)) {
    const MIX: [(MaterialId, f64); 5] = [
        (MaterialId::POTSHERD, 0.15),
        (MaterialId::ASH, 0.30),
        (MaterialId::BONE, 0.10),
        (MaterialId::LEAF_LITTER, 0.25),
        (MaterialId::LOAM, 0.20),
    ];
    for _ in 0..6 {
        let dx = rng.below(5) as isize - 2;
        let dz = rng.below(5) as isize - 2;
        let x = (center.0 as isize + dx).clamp(0, COLS as isize - 1) as usize;
        let z = (center.1 as isize + dz).clamp(0, COLS as isize - 1) as usize;
        let m = rng.pick(&MIX);
        field.deposit(Field::idx(x, z), m);
    }
    field.relax(1);
}

/// (d) Alluvial fan: water leaves a gully mouth carrying a graded load;
/// energy decays along the path, dropping coarse grains first and carrying
/// fines far — grain-size sorting by flow distance.
fn alluvial_pass(field: &mut Field, rng: &mut Rng, source: usize) {
    /// Energy below which a material settles out of the flow.
    fn settle_energy(m: MaterialId) -> f64 {
        match m {
            MaterialId::GRAVEL => 25.0,
            MaterialId::SAND => 15.0,
            MaterialId::SILT => 7.0,
            _ => 2.5, // clay
        }
    }
    let mut load: Vec<MaterialId> = Vec::new();
    for (m, n) in [
        (MaterialId::GRAVEL, 2),
        (MaterialId::SAND, 3),
        (MaterialId::SILT, 3),
        (MaterialId::CLAY, 2),
    ] {
        for _ in 0..n {
            load.push(m);
        }
    }
    let mut energy = 40.0f64;
    let mut cur = source;
    for _ in 0..400 {
        if load.is_empty() {
            return;
        }
        let x = cur % COLS;
        let z = cur / COLS;
        let next = Field::neighbors(x, z).min_by_key(|&n| field.top(n));
        let drop = next.map_or(0, |n| field.top(cur) - field.top(n));
        // Slope feeds energy; flat ground bleeds it.
        energy = energy * 0.92 + drop as f64 * 0.4;
        // Drop everything the flow can no longer carry (with a little
        // stochastic spread so fronts aren't knife edges).
        let mut i = 0;
        while i < load.len() {
            let jitter = 0.8 + 0.4 * rng.f64();
            if energy < settle_energy(load[i]) * jitter {
                field.deposit(cur, load[i]);
                load.swap_remove(i);
            } else {
                i += 1;
            }
        }
        match next {
            Some(n) if field.top(cur) > field.top(n) => cur = n,
            _ => {
                // Ponded: everything settles here.
                for m in load.drain(..) {
                    field.deposit(cur, m);
                }
                break;
            }
        }
    }
    // Anything still suspended at range settles at the terminus.
    let leftovers = std::mem::take(&mut load);
    for m in leftovers {
        field.deposit(cur, m);
    }
    field.relax(1);
}

// ---------------------------------------------------------------------------
// Voxelization + measurement
// ---------------------------------------------------------------------------

struct Measure {
    label: String,
    deposit_chunks: usize,
    /// (distinct mixtures, palette bits, encoded sidecar bytes) per chunk
    /// containing any deposit.
    per_chunk: Vec<(ChunkPos, usize, u8, usize)>,
    table_entries: usize,
    table_bytes: usize,
    total_eighths: usize,
}

impl Measure {
    fn sidecar_bytes(&self) -> usize {
        self.per_chunk.iter().map(|&(_, _, _, b)| b).sum()
    }

    fn bytes_per_m3(&self) -> f64 {
        self.sidecar_bytes() as f64 / (self.deposit_chunks as f64 * CHUNK_M3)
    }

    fn bytes_per_m3_with_table(&self) -> f64 {
        (self.sidecar_bytes() + self.table_bytes) as f64 / (self.deposit_chunks as f64 * CHUNK_M3)
    }

    fn distinct_stats(&self) -> (usize, usize, usize) {
        let mut v: Vec<usize> = self.per_chunk.iter().map(|&(_, d, _, _)| d).collect();
        v.sort_unstable();
        let median = v[v.len() / 2];
        (v[0], median, *v.last().expect("non-empty"))
    }

    fn max_bits(&self) -> u8 {
        self.per_chunk
            .iter()
            .map(|&(_, _, b, _)| b)
            .max()
            .unwrap_or(0)
    }

    fn print(&self) {
        let (dmin, dmed, dmax) = self.distinct_stats();
        println!("-- {} --", self.label);
        println!(
            "  deposit eighths: {} (~{:.0} m^3 of material)",
            self.total_eighths,
            self.total_eighths as f64 * EIGHTH_M * VOXEL_M * VOXEL_M
        );
        println!(
            "  deposit chunks: {}   region mixture table: {} entries, {} B ({:.1} B/entry)",
            self.deposit_chunks,
            self.table_entries,
            self.table_bytes,
            self.table_bytes as f64 / self.table_entries.max(1) as f64
        );
        println!(
            "  distinct mixtures/chunk: min {dmin} / median {dmed} / max {dmax}   palette bits: max {}",
            self.max_bits()
        );
        println!(
            "  sidecar bytes: {} total, {:.1} B/chunk avg",
            self.sidecar_bytes(),
            self.sidecar_bytes() as f64 / self.deposit_chunks.max(1) as f64
        );
        println!(
            "  bytes/m^3 over deposit chunks: {:.4} (+table: {:.4})  [base grid {BASELINE_PALETTE_B_M3}, raw {BASELINE_RAW_B_M3}]",
            self.bytes_per_m3(),
            self.bytes_per_m3_with_table()
        );
    }
}

/// Slice the field's eighth-stacks into voxels, intern every mixture, build
/// per-chunk MaterialChunks, and measure. Non-destructive.
fn measure(field: &Field, label: &str) -> Measure {
    let mut table = MixtureTable::new();
    let mut chunks: HashMap<ChunkPos, Vec<MixtureId>> = HashMap::new();
    for z in 0..COLS {
        for x in 0..COLS {
            let c = Field::idx(x, z);
            let stack = &field.stacks[c];
            if stack.is_empty() {
                continue;
            }
            let vx = field.origin_vx + x as i64;
            let vz = field.origin_vz + z as i64;
            let base = field.surface_eighth[c];
            let mut i = 0usize;
            while i < stack.len() {
                let e = base + i as i64;
                let vy = e.div_euclid(8);
                // Consume every eighth in this voxel.
                let mut mats: Vec<MaterialId> = Vec::with_capacity(8);
                while i < stack.len() && (base + i as i64).div_euclid(8) == vy {
                    mats.push(MaterialId::from_raw(stack[i]).expect("valid id"));
                    i += 1;
                }
                let contents = VoxelContents::debris_only(&mats).expect("<= 8 eighths per voxel");
                let id = table.intern(contents);
                let pos = ChunkPos::from_world_voxel(vx, vy, vz);
                let (lx, ly, lz) = local_voxel(vx, vy, vz);
                let dense = chunks
                    .entry(pos)
                    .or_insert_with(|| vec![MixtureId::EMPTY; CHUNK_VOLUME]);
                dense[Chunk::index(lx, ly, lz)] = id;
            }
        }
    }

    let mut per_chunk: Vec<(ChunkPos, usize, u8, usize)> = chunks
        .iter()
        .map(|(&pos, dense)| {
            let chunk = MaterialChunk::from_dense(dense);
            chunk.validate().expect("fresh chunk validates");
            chunk.validate_against(&table).expect("ids resolve");
            let bytes = chunk.to_sidecar().map_or(0, |s| s.data.len());
            (pos, chunk.palette().len(), chunk.index_bits(), bytes)
        })
        .collect();
    per_chunk.sort_by_key(|&(p, ..)| (p.y, p.z, p.x));

    Measure {
        label: label.to_string(),
        deposit_chunks: per_chunk.len(),
        per_chunk,
        table_entries: table.len(),
        table_bytes: table.encode().len(),
        total_eighths: field.total_eighths(),
    }
}

// ---------------------------------------------------------------------------
// Scenario setup
// ---------------------------------------------------------------------------

/// Chunk-aligned region origin: scan deterministically for a window holding
/// both a ravine (steep walls for scree) and ordinary hills.
fn pick_origin(terrain: &Terrain) -> (i64, i64) {
    for step in 0..64i64 {
        let ox = (step % 8) * 128 - 512;
        let oz = (step / 8) * 128 - 512;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for sz in 0..16 {
            for sx in 0..16 {
                let xm = (ox + sx * 16) as f64 * VOXEL_M;
                let zm = (oz + sz * 16) as f64 * VOXEL_M;
                let h = terrain.surface_height_m(xm, zm);
                lo = lo.min(h);
                hi = hi.max(h);
            }
        }
        if hi - lo > 40.0 && hi > 5.0 {
            return (ox, oz);
        }
    }
    (0, 0)
}

/// Cliff-base columns: columns whose terrain surface sits well below a
/// neighbor's (the wall towers over them) — rockfall sources land here.
fn scree_sources(field: &Field) -> Vec<usize> {
    let mut out = Vec::new();
    for z in 0..COLS {
        for x in 0..COLS {
            let c = Field::idx(x, z);
            let drop = Field::neighbors(x, z)
                .map(|n| field.surface_eighth[n] - field.surface_eighth[c])
                .max()
                .unwrap_or(0);
            if drop > 16 {
                out.push(c);
            }
        }
    }
    out
}

/// Flattest column in the central quarter — the settlement site.
fn midden_site(field: &Field) -> (usize, usize) {
    let mut best = (COLS / 2, COLS / 2);
    let mut best_score = i64::MAX;
    for z in (COLS / 4..3 * COLS / 4).step_by(4) {
        for x in (COLS / 4..3 * COLS / 4).step_by(4) {
            let c = Field::idx(x, z);
            let h = field.surface_eighth[c];
            if h < 8 {
                continue; // not in the ravine
            }
            let score: i64 = Field::neighbors(x, z)
                .map(|n| (field.surface_eighth[n] - h).abs())
                .sum();
            if score < best_score {
                best_score = score;
                best = (x, z);
            }
        }
    }
    best
}

/// Steepest column — the gully mouth feeding the alluvial fan.
fn alluvial_source(field: &Field) -> usize {
    let mut best = 0;
    let mut best_drop = i64::MIN;
    for z in 2..COLS - 2 {
        for x in 2..COLS - 2 {
            let c = Field::idx(x, z);
            let drop = Field::neighbors(x, z)
                .map(|n| field.surface_eighth[c] - field.surface_eighth[n])
                .max()
                .unwrap_or(0);
            if drop > best_drop {
                best_drop = drop;
                best = c;
            }
        }
    }
    best
}

fn run_wind(field: &mut Field, rng: &mut Rng, passes: usize, start: usize) {
    for p in start..start + passes {
        wind_pass(field, rng, p);
    }
    field.relax(8);
}

fn run_rockfall(field: &mut Field, rng: &mut Rng, passes: usize) {
    let sources = scree_sources(field);
    assert!(
        !sources.is_empty(),
        "region must contain cliffs for the scree scenario"
    );
    for _ in 0..passes {
        rockfall_pass(field, rng, &sources);
    }
    field.relax(8);
}

fn run_midden(field: &mut Field, rng: &mut Rng, passes: usize) {
    let site = midden_site(field);
    for _ in 0..passes {
        midden_pass(field, rng, site);
    }
    field.relax(8);
}

fn run_alluvial(field: &mut Field, rng: &mut Rng, passes: usize) {
    let source = alluvial_source(field);
    for _ in 0..passes {
        alluvial_pass(field, rng, source);
    }
    field.relax(8);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The headline S8 measurement: each process alone, the marginal cost of
/// stacking processes, and per-pass-block marginal cost for wind.
#[test]
fn deposition_measurements() {
    let terrain = Terrain::new(1337);
    let (ox, oz) = pick_origin(&terrain);
    println!(
        "== S8 deposition measurements: region 8x8 chunk-columns at voxel origin ({ox}, {oz}), seed {SEED:#x} =="
    );

    // --- each process on a fresh field ---
    let mut m_each: Vec<Measure> = Vec::new();
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED);
        // Marginal cost per pass block, measured on the way.
        for block in 0..4 {
            run_wind(&mut field, &mut rng, 24, block * 24);
            let m = measure(&field, &format!("wind after {} passes", (block + 1) * 24));
            m.print();
            if block == 3 {
                m_each.push(m);
            }
        }
    }
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 1);
        run_rockfall(&mut field, &mut rng, 200);
        let m = measure(&field, "rockfall/scree, 200 passes");
        m.print();
        m_each.push(m);
    }
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 2);
        run_midden(&mut field, &mut rng, 400);
        let m = measure(&field, "midden, 400 passes");
        m.print();
        m_each.push(m);
    }
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 3);
        run_alluvial(&mut field, &mut rng, 300);
        let m = measure(&field, "alluvial fan, 300 passes");
        m.print();
        m_each.push(m);
    }

    // --- all four processes stacked on one field: marginal cost per process ---
    println!("== combined field (marginal cost per additional process) ==");
    let mut field = Field::new(&terrain, ox, oz);
    let mut rng = Rng::new(SEED ^ 4);
    run_wind(&mut field, &mut rng, 96, 0);
    let c1 = measure(&field, "combined: wind");
    c1.print();
    run_rockfall(&mut field, &mut rng, 200);
    let c2 = measure(&field, "combined: +rockfall");
    c2.print();
    run_midden(&mut field, &mut rng, 400);
    let c3 = measure(&field, "combined: +midden");
    c3.print();
    run_alluvial(&mut field, &mut rng, 300);
    let c4 = measure(&field, "combined: +alluvial");
    c4.print();

    // --- worst chunks of the combined field, for the results doc ---
    println!("== combined field, 8 fattest chunks ==");
    let mut worst = c4.per_chunk.clone();
    worst.sort_by_key(|&(_, d, _, _)| std::cmp::Reverse(d));
    for &(pos, distinct, bits, bytes) in worst.iter().take(8) {
        println!(
            "  chunk ({:>3},{:>3},{:>3}): {distinct:>4} mixtures, {bits} bits, {bytes:>6} B, {:.3} B/m^3",
            pos.x,
            pos.y,
            pos.z,
            bytes as f64 / CHUNK_M3
        );
    }

    // --- the empirical claims the spike stands on ---
    for m in &m_each {
        let (_, _, dmax) = m.distinct_stats();
        assert!(
            dmax <= 512,
            "{}: realistic deposition should stay in the hundreds of distinct \
             mixtures per chunk (got {dmax})",
            m.label
        );
        assert!(
            m.max_bits() <= 9,
            "{}: palette width must stay single-digit bits (got {})",
            m.label,
            m.max_bits()
        );
    }
    // Stacking all four processes must not explode the region table.
    assert!(
        c4.table_entries <= 4096,
        "combined table exploded: {} entries",
        c4.table_entries
    );
    // Marginal growth: each added process adds states, but the union stays
    // within the same order of magnitude as the worst single process.
    assert!(c2.table_entries >= c1.table_entries);
    assert!(c3.table_entries >= c2.table_entries);
    assert!(c4.table_entries >= c3.table_entries);
}

/// The pathological cases: mixing engineered to maximize distinct states.
#[test]
fn pathological_gradient_mixing() {
    let terrain = Terrain::new(1337);
    let (ox, oz) = pick_origin(&terrain);
    println!("== S8 pathological cases ==");

    // (1) Continuous 4-material gradient: blend weights vary smoothly across
    // the region; every eighth rolls independently. This is the "smooth
    // gradient" the design worries about.
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 10);
        for z in 0..COLS {
            for x in 0..COLS {
                let c = Field::idx(x, z);
                let fx = x as f64 / (COLS - 1) as f64;
                let fz = z as f64 / (COLS - 1) as f64;
                let weights = [
                    (MaterialId::SAND, (1.0 - fx) * (1.0 - fz)),
                    (MaterialId::SILT, fx * (1.0 - fz)),
                    (MaterialId::ASH, (1.0 - fx) * fz),
                    (MaterialId::CLAY, fx * fz),
                ];
                for _ in 0..32 {
                    let m = rng.pick(&weights);
                    field.deposit(c, m);
                }
            }
        }
        let m = measure(
            &field,
            "pathological: 4-material continuous gradient, 4 voxels deep",
        );
        m.print();
        // The eighth quantization caps 4-material mixtures at C(11,3) = 165
        // full-voxel states (~494 including partial fills), so even an
        // adversarial gradient CANNOT explode a 4-material palette.
        let (_, _, dmax) = m.distinct_stats();
        assert!(
            m.table_entries <= 500,
            "4-material gradient must be capped by multiset combinatorics, got {}",
            m.table_entries
        );
        assert!(dmax <= 500);
    }

    // (2) Maximum entropy: every eighth uniform over all 12 materials — the
    // true worst case for the interner and the palette.
    {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 11);
        for z in 0..COLS {
            for x in 0..COLS {
                let c = Field::idx(x, z);
                for _ in 0..32 {
                    let m = MaterialId::from_raw(rng.below(MATERIAL_COUNT) as u8).expect("valid");
                    field.deposit(c, m);
                }
            }
        }
        let m = measure(
            &field,
            "pathological: 12-material uniform noise, 4 voxels deep",
        );
        m.print();
        let (_, _, dmax) = m.distinct_stats();
        println!("  (12-material multisets of 8 eighths: C(19,8) = 75582 possible states)");
        // This one DOES explode — the assertion documents the failure mode.
        assert!(
            dmax > 1500,
            "the max-entropy case should approach one state per deposit voxel, got {dmax}"
        );
        assert!(
            m.max_bits() >= 11,
            "palette width should blow past a byte in the max-entropy case"
        );
    }
}

/// Same seed, same field, same tables — the whole pipeline is deterministic.
#[test]
fn deposition_is_deterministic() {
    let terrain = Terrain::new(1337);
    let (ox, oz) = pick_origin(&terrain);
    let run = || {
        let mut field = Field::new(&terrain, ox, oz);
        let mut rng = Rng::new(SEED ^ 20);
        run_wind(&mut field, &mut rng, 24, 0);
        run_midden(&mut field, &mut rng, 60);
        field.fingerprint()
    };
    assert_eq!(
        run(),
        run(),
        "deposition must be a pure function of the seed"
    );
}

/// Relaxation conserves mass and terminates on a stable slope.
#[test]
fn repose_relaxation_conserves_and_stabilizes() {
    let terrain = Terrain::new(1337);
    let (ox, oz) = pick_origin(&terrain);
    let mut field = Field::new(&terrain, ox, oz);
    // A 40-eighth sand spike on one column.
    let c = Field::idx(128, 128);
    for _ in 0..40 {
        field.deposit(c, MaterialId::SAND);
    }
    let before = field.total_eighths();
    field.relax(200);
    assert_eq!(field.total_eighths(), before, "relaxation conserves mass");
    assert_eq!(field.relax_sweep(), 0, "relaxation reached a stable state");
    assert!(
        field.stacks[c].len() < 40,
        "the spike must have slumped ({} left)",
        field.stacks[c].len()
    );
}
