//! **Where to stand to SEE what the geotherm did to coal** (journal/0093).
//!
//! journal/0093 recalibrated coalification onto a real geotherm temperature
//! (`COAL_ONSET_C` 8 → 22 °C) and, in doing so, **relocated** coal: it now
//! concentrates where the crust and the surface are warm rather than wherever
//! peat happened to be buried deepest. ROADMAP records the appearance walk as
//! owed. This is that walk's map — **coordinates only**. Nothing here tunes,
//! recalibrates or judges coal (a user-blessed placeholder); it answers *where
//! is a coal seam a bench will expose, and is the relocation legible on foot*.
//!
//! Three tiers, because the deep record and the world a player digs are not the
//! same thing (organic.rs's lesson: a thick *record* seam can bury below the
//! collapse column), and because a null needs a control:
//!
//! 1. **Census** — every deep cell's contiguous `Biofacies::Coal` (and `Peat`)
//!    runs, with the run's depth below the top of the record and its thickness.
//!    Cheap, world-wide, and the source of the distribution numbers.
//! 2. **Control** — the same census on the world journal/0093 and `organic.rs`
//!    measured (seed `0x0D5EED572026`, Medium). The production world's answer is
//!    a *zero*, and a zero from an instrument nobody has seen produce a non-zero
//!    is not evidence (CLAUDE.md § "pick the control that can SEE your
//!    question"). This is that control.
//! 3. **Verification** — for the shallowest/thickest candidates, actually
//!    generate the chunk column and count `Block::Material(..)` voxels per voxel
//!    column. That is the seam the player's bench cuts into, so a station's
//!    reported depth and thickness are **voxel** facts, not record facts.
//!
//! `cargo run --release -p dc-worldgen --example coal_walk_tour`

use dc_core::{Block, ChunkPos, MaterialId};
use dc_worldgen::WorldGenerator;
use dc_worldgen::deeptime::{
    self, Biofacies, COAL_ONSET_C, DEFAULT_CONTINENTAL_GRADIENT_C_PER_M, DeepField, SEA_LEVEL_M,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, LAT_NORTH, LAT_SOUTH, Pregen, WorldParams};

/// The world dc-client boots — `BENCH_SEED` / `WORLDGEN_EXTENT` in
/// `dc-client/src/authority.rs`, the same pair `s18_weathering_tour` uses.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
/// The world `tests/organic.rs` and journal/0093 measured coal on — the control.
const CONTROL_SEED: u64 = 0x0D5E_ED57_2026;
const VOXEL_M: f64 = 0.9;
/// Station A of this walk — the saprolite front, already chosen.
const STATION_A_XZ_M: (f64, f64) = (84_185.0, 9_212.0);
/// Border ring excluded from station picks (march edge artifacts, per tour_map.rs).
const EDGE_MARGIN: i64 = 4;
/// A bench is a shelf a few voxels deep; a seam deeper than this is a mine, not
/// a road-cut. Candidate filter for station selection only.
const BENCH_REACH_M: f64 = 14.0;
/// How many record candidates get the (expensive) chunk-generation check.
const VERIFY_BUDGET: usize = 24;
/// Chunk-column depth scanned below the surface chunk, in chunks (32 voxels each).
const SCAN_CHUNKS: i32 = 3;

// ---------------------------------------------------------------------------
// tier 1 — the record census
// ---------------------------------------------------------------------------

/// One contiguous run of same-biofacies units in a deep cell's record.
#[derive(Clone, Copy, Debug)]
struct Seam {
    cell: usize,
    /// Overburden above the run's top, metres — record depth below the surface.
    depth_m: f64,
    /// Sum of the run's unit thicknesses, metres.
    thick_m: f64,
}

/// The world-wide coal picture, as a value — so `main` prints it and the gate
/// asserts on it without either re-deriving the other's numbers.
struct Census {
    total_cells: usize,
    /// Interior cells whose record holds a non-top peat-or-coal unit.
    candidate_cells: usize,
    /// …of which carry at least one promoted (coal) unit.
    coal_cells: usize,
    /// …of which carry peat but no coal (the contrast pool).
    peat_only_cells: usize,
    /// Every contiguous coal run in every interior cell.
    coal_seams: Vec<Seam>,
    /// Per-cell shallowest coal run (station candidates).
    coal_best: Vec<Seam>,
    /// Per-cell shallowest *peat* run — the fallback station pool when the
    /// geotherm promoted nothing here.
    peat_best: Vec<Seam>,
    /// `true` where the cell carries peat but no coal (indexed by cell).
    peat_only: Vec<bool>,
    /// Every coalification candidate unit's geotherm temperature (°C), sorted.
    cand_temps: Vec<f64>,
}

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

/// Latitude (deg) of a deep row — the same compression the grid uses.
fn lat_deg(gy: usize, w: usize) -> f64 {
    LAT_SOUTH + (LAT_NORTH - LAT_SOUTH) * (gy as f64 + 0.5) / w as f64
}

/// Surface air temperature (°C) at a cell — the geotherm's upper boundary, and
/// (burial being shallow here) the dominant term in the coalification test.
fn surface_temp_c(f: &DeepField, i: usize) -> f64 {
    f64::from(deeptime::climate::air_temp_c(
        lat_deg(i / f.w, f.w),
        f.surf[i],
    ))
}

fn gradient(f: &DeepField, i: usize) -> f64 {
    f.geotherm
        .get(i)
        .copied()
        .unwrap_or(DEFAULT_CONTINENTAL_GRADIENT_C_PER_M)
}

/// Contiguous runs of `want` in one column, given the per-unit overburden.
fn runs(
    units: &[dc_worldgen::deeptime::DepUnit],
    over: &[f64],
    cell: usize,
    want: Biofacies,
) -> Vec<Seam> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for k in 0..=units.len() {
        let hit = k < units.len() && units[k].tag.biota == want;
        if hit {
            start.get_or_insert(k);
        } else if let Some(a) = start.take() {
            let b = k - 1;
            out.push(Seam {
                cell,
                depth_m: over[b],
                thick_m: (a..=b).map(|j| units[j].thickness_m).sum(),
            });
        }
    }
    out
}

/// Shallowest run wins; near-ties broken by thickness.
fn shallowest(v: &[Seam]) -> Option<Seam> {
    v.iter().copied().min_by(|x, y| {
        x.depth_m
            .total_cmp(&y.depth_m)
            .then(y.thick_m.total_cmp(&x.thick_m))
    })
}

/// Walk every interior cell's record for contiguous coal and peat runs, and
/// collect the coalification candidates' geotherm temperatures.
fn census(f: &DeepField) -> Census {
    let w = f.w;
    let mut coal_seams = Vec::new();
    let mut coal_best = Vec::new();
    let mut peat_best = Vec::new();
    let mut cand_temps = Vec::new();
    let mut peat_only = vec![false; f.strata.len()];
    let (mut candidate_cells, mut coal_cells, mut peat_only_cells) = (0, 0, 0);

    for (i, s) in f.strata.iter().enumerate() {
        if !interior(w, i) || s.units.is_empty() {
            continue;
        }
        let n = s.units.len();
        let top = n - 1;
        // Overburden above each unit, bottom-up accumulation (promote_coal's walk).
        let mut over = vec![0.0f64; n];
        let mut acc = 0.0;
        for k in (0..n).rev() {
            over[k] = acc;
            acc += s.units[k].thickness_m;
        }

        let surf_t = surface_temp_c(f, i);
        let grad = gradient(f, i);
        let mut has_candidate = false;
        let mut has_peat = false;
        for (k, u) in s.units.iter().enumerate() {
            if k != top && matches!(u.tag.biota, Biofacies::Peat | Biofacies::Coal) {
                has_candidate = true;
                if u.tag.biota == Biofacies::Peat {
                    has_peat = true;
                }
                cand_temps.push(deeptime::temperature_c(
                    surf_t,
                    grad,
                    over[k] + 0.5 * u.thickness_m,
                ));
            }
        }

        let mut cs = runs(&s.units, &over, i, Biofacies::Coal);
        let ps = runs(&s.units, &over, i, Biofacies::Peat);
        if has_candidate {
            candidate_cells += 1;
        }
        if cs.is_empty() {
            if has_peat {
                peat_only_cells += 1;
                peat_only[i] = true;
            }
        } else {
            coal_cells += 1;
        }
        if let Some(p) = shallowest(&cs) {
            coal_best.push(p);
        }
        if let Some(p) = shallowest(&ps) {
            peat_best.push(p);
        }
        coal_seams.append(&mut cs);
    }
    cand_temps.sort_by(f64::total_cmp);

    Census {
        total_cells: w * w,
        candidate_cells,
        coal_cells,
        peat_only_cells,
        coal_seams,
        coal_best,
        peat_best,
        peat_only,
        cand_temps,
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p * (sorted.len() - 1) as f64).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

fn spread(label: &str, mut v: Vec<f64>) {
    v.sort_by(f64::total_cmp);
    println!(
        "  {label:<26} min {:6.2}  p50 {:6.2}  p95 {:6.2}  max {:6.2}   (n={})",
        percentile(&v, 0.0),
        percentile(&v, 0.50),
        percentile(&v, 0.95),
        percentile(&v, 1.0),
        v.len()
    );
}

// ---------------------------------------------------------------------------
// coordinate conversion
// ---------------------------------------------------------------------------

/// World voxel centre of deep cell `idx` (inverse of `DeepField::deep_coords`).
fn idx_to_voxel(f: &DeepField, idx: usize) -> (i64, i64) {
    let (w, wp) = (f.w as f64, f.wp as f64);
    let half = (f.wp / 2) as f64;
    let (gx, gy) = ((idx % f.w) as f64, (idx / f.w) as f64);
    let v = |g: f64| (((g + 0.5) * wp / w - half) * CELL_VOXELS as f64).round() as i64;
    (v(gx), v(gy))
}

fn world_m(f: &DeepField, idx: usize) -> (f64, f64) {
    let (vx, vz) = idx_to_voxel(f, idx);
    (vx as f64 * VOXEL_M, vz as f64 * VOXEL_M)
}

fn dist_m(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

// ---------------------------------------------------------------------------
// tier 3 — the collapsed column a player actually digs
// ---------------------------------------------------------------------------

/// A verified station: a real voxel column with real voxels of the wanted
/// material in it.
#[derive(Clone, Copy, Debug)]
struct Station {
    cell: usize,
    /// The voxel column, world voxels.
    vx: i64,
    vz: i64,
    /// Surface voxel y at that column, and its top elevation in metres.
    surf_y: i32,
    surf_m: f64,
    /// Top / bottom material voxel y, and how many the column holds.
    top_y: i32,
    bot_y: i32,
    vox: u32,
    /// Metres from the surface down to the first such voxel.
    depth_m: f64,
    /// Metres of it in the column (`vox · VOXEL_M`).
    thick_m: f64,
    record: Seam,
}

/// Shallow beats thick (a bench reaches a few voxels), thickness breaks ties —
/// a one-voxel smear is not a seam you can see.
fn score(depth_m: f64, thick_m: f64) -> f64 {
    thick_m - 1.5 * depth_m
}

/// Scan the chunk column containing chunk `(cx, cz)` for a material; returns, per
/// voxel column (`lz * 32 + lx`), `(count, top_y, bot_y)`.
fn column_scan(g: &mut WorldGenerator, cx: i64, cz: i64, want: MaterialId) -> Vec<(u32, i32, i32)> {
    let mut out = vec![(0u32, i32::MIN, i32::MAX); 32 * 32];
    let top_cy = g.surface_chunk_y(cx, cz);
    for cy in (top_cy - SCAN_CHUNKS)..=top_cy {
        let pos = ChunkPos::new(cx as i32, cy, cz as i32);
        let (chunk, _) = g.generate_chunk_with_materials(pos);
        for lz in 0..32usize {
            for lx in 0..32usize {
                for y in 0..32usize {
                    if chunk.get(lx, y, lz) == Block::Material(want) {
                        let wy = cy * 32 + y as i32;
                        let e = &mut out[lz * 32 + lx];
                        e.0 += 1;
                        e.1 = e.1.max(wy);
                        e.2 = e.2.min(wy);
                    }
                }
            }
        }
    }
    out
}

/// Generate the chunk column at a candidate cell and pick the voxel column whose
/// material is shallowest-and-thickest. `None` when the collapse buried it or the
/// column is wilds.
fn verify(g: &mut WorldGenerator, f: &DeepField, seam: Seam, want: MaterialId) -> Option<Station> {
    let (vx0, vz0) = idx_to_voxel(f, seam.cell);
    let (cx, cz) = (vx0.div_euclid(32), vz0.div_euclid(32));
    let col = g.column_record(cx, cz);
    if col.wilds {
        return None;
    }
    let heights = col.heights.clone();
    let scan = column_scan(g, cx, cz, want);

    let mut pick: Option<Station> = None;
    for lz in 0..32i64 {
        for lx in 0..32i64 {
            let (vox, top_y, bot_y) = scan[(lz * 32 + lx) as usize];
            if vox == 0 {
                continue;
            }
            let surf_y = heights[(lz * 32 + lx) as usize];
            if top_y > surf_y {
                continue;
            }
            let depth_m = f64::from(surf_y - top_y) * VOXEL_M;
            let st = Station {
                cell: seam.cell,
                vx: cx * 32 + lx,
                vz: cz * 32 + lz,
                surf_y,
                surf_m: f64::from(surf_y + 1) * VOXEL_M,
                top_y,
                bot_y,
                vox,
                depth_m,
                thick_m: f64::from(vox) * VOXEL_M,
                record: seam,
            };
            if pick.is_none_or(|b| score(st.depth_m, st.thick_m) > score(b.depth_m, b.thick_m)) {
                pick = Some(st);
            }
        }
    }
    pick
}

/// Rank record candidates for verification: shallow, thick, on dry land, and
/// spatially de-duplicated so we never verify five cells of the same swamp.
fn pick_candidates(f: &DeepField, pool: &[Seam], budget: usize) -> Vec<Seam> {
    let mut cands: Vec<Seam> = pool
        .iter()
        .copied()
        .filter(|s| {
            s.depth_m <= BENCH_REACH_M && s.thick_m >= 1.0 && f.surf[s.cell] > SEA_LEVEL_M + 5.0
        })
        .collect();
    cands.sort_by(|a, b| score(b.depth_m, b.thick_m).total_cmp(&score(a.depth_m, a.thick_m)));
    let mut picked: Vec<Seam> = Vec::new();
    for s in cands {
        if picked.len() >= budget {
            break;
        }
        let (gx, gy) = ((s.cell % f.w) as i64, (s.cell / f.w) as i64);
        if picked.iter().all(|p| {
            let (px, py) = ((p.cell % f.w) as i64, (p.cell / f.w) as i64);
            (gx - px).abs().max((gy - py).abs()) >= 5
        }) {
            picked.push(s);
        }
    }
    picked
}

fn print_station(f: &DeepField, label: &str, s: &Station, material: &str) {
    let (x, z) = (s.vx as f64 * VOXEL_M, s.vz as f64 * VOXEL_M);
    println!("\n{label}");
    println!(
        "  world            : x {x:.0} m, z {z:.0} m   (voxel {}, {})",
        s.vx, s.vz
    );
    println!(
        "  surface          : {:.1} m  (surface voxel y {})",
        s.surf_m, s.surf_y
    );
    println!(
        "  {material:<16} : top {:.1} m below surface, {:.1} m thick ({} voxels, y {}..={})",
        s.depth_m, s.thick_m, s.vox, s.bot_y, s.top_y
    );
    println!(
        "  deep record      : run {:.2} m thick at {:.2} m depth; gradient {:.1} C/km, \
         surface T {:.1} C  (onset {COAL_ONSET_C} C)",
        s.record.thick_m,
        s.record.depth_m,
        gradient(f, s.cell) * 1000.0,
        surface_temp_c(f, s.cell)
    );
    println!(
        "  POSE (metres; yaw/pitch RADIANS): x {x:.0}, y {:.1}, z {z:.0}, yaw 0.0, pitch -0.45",
        s.surf_m + 2.0
    );
    println!(
        "  bench (VOXELS)   : world_fill dc:air  x {}..{}, z {}..{}, y {}..{}   \
         then stand ~15 m south and look at the cut face",
        s.vx - 12,
        s.vx + 12,
        s.vz - 12,
        s.vz + 12,
        s.bot_y - 2,
        s.surf_y
    );
    println!(
        "  from STATION A   : {:.0} m ({:.2} km)",
        dist_m((x, z), STATION_A_XZ_M),
        dist_m((x, z), STATION_A_XZ_M) / 1000.0
    );
}

/// **The geotherm's own claim, scored**: coal concentrates where the crust and
/// the surface are warm. Coal-bearing vs peat-only cells, side by side, split by
/// the `temperature` field's gradient — the same read on whichever world it is
/// handed, so the production world and the control are directly comparable.
fn warm_crust_split(f: &DeepField, c: &Census) {
    if f.geotherm.is_empty() {
        return;
    }
    println!("\n  --- the temperature field where the peat is ---");
    let coal_cells: Vec<usize> = c.coal_best.iter().map(|s| s.cell).collect();
    let peat_cells: Vec<usize> = (0..f.strata.len()).filter(|&i| c.peat_only[i]).collect();
    let mean = |v: &[usize], g: &dyn Fn(usize) -> f64| -> f64 {
        if v.is_empty() {
            return f64::NAN;
        }
        v.iter().map(|&i| g(i)).sum::<f64>() / v.len() as f64
    };
    for (label, set) in [
        ("coal cells", &coal_cells),
        ("peat-only cells", &peat_cells),
    ] {
        println!(
            "  {label:<16}: n {:>6}, mean gradient {:5.1} C/km, mean surface T {:5.1} C, \
             mean elev {:6.0} m",
            set.len(),
            mean(set, &|i| gradient(f, i)) * 1000.0,
            mean(set, &|i| surface_temp_c(f, i)),
            mean(set, &|i| f.surf[i]),
        );
    }
    for (label, lo, hi) in [
        ("craton   <20 C/km", 0.0, 0.020),
        ("stable   20-30", 0.020, 0.030),
        ("warm     30-40", 0.030, 0.040),
        ("rift/arc >=40", 0.040, 1.0),
    ] {
        let inb = |i: usize| {
            let g = gradient(f, i);
            g >= lo && g < hi
        };
        let n_coal = coal_cells.iter().filter(|&&i| inb(i)).count();
        let n_peat = peat_cells.iter().filter(|&&i| inb(i)).count();
        let denom = n_coal + n_peat;
        println!(
            "  {label:<18} coal {n_coal:>5}, peat-only {n_peat:>5}  → {:>5.1}% coal",
            if denom == 0 {
                f64::NAN
            } else {
                100.0 * n_coal as f64 / denom as f64
            }
        );
    }
}

// ---------------------------------------------------------------------------

fn main() {
    println!("=== coal walk tour — where the geotherm put coal (journal/0093) ===");
    println!(
        "seed {SEED}, extent {}, production config, onset {COAL_ONSET_C} C, {VOXEL_M} m voxels\n",
        EXTENT.label()
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let f = &pregen.deep;
    let c = census(f);

    // ---- 3. the distribution -------------------------------------------------
    println!(
        "--- CENSUS, production world (interior deep cells, {:.0} m/cell) ---",
        f.cell_m
    );
    println!(
        "  deep grid {}x{} = {} cells; peat/coal-bearing: {} ({:.2}%)",
        f.w,
        f.w,
        c.total_cells,
        c.candidate_cells,
        100.0 * c.candidate_cells as f64 / c.total_cells as f64
    );
    println!(
        "  cells carrying COAL     : {} ({:.1}% of peat/coal-bearing cells)",
        c.coal_cells,
        100.0 * c.coal_cells as f64 / c.candidate_cells.max(1) as f64
    );
    println!(
        "  cells with PEAT, no coal: {} ({:.1}%)",
        c.peat_only_cells,
        100.0 * c.peat_only_cells as f64 / c.candidate_cells.max(1) as f64
    );
    println!("  contiguous coal runs    : {}", c.coal_seams.len());
    // Belt and braces: the census skips a 4-cell border ring, so tally the RAW
    // record over the whole grid too. "No coal anywhere" must not be an artifact
    // of the edge margin.
    let (raw_coal, raw_peat) =
        f.strata
            .iter()
            .flat_map(|s| s.units.iter())
            .fold((0usize, 0usize), |(c, p), u| match u.tag.biota {
                Biofacies::Coal => (c + 1, p),
                Biofacies::Peat => (c, p + 1),
                _ => (c, p),
            });
    println!("  RAW units, whole grid   : {raw_coal} coal, {raw_peat} peat (no edge margin)");
    if !c.coal_seams.is_empty() {
        spread(
            "coal depth below surf (m)",
            c.coal_seams.iter().map(|s| s.depth_m).collect(),
        );
        spread(
            "coal run thickness (m)",
            c.coal_seams.iter().map(|s| s.thick_m).collect(),
        );
    }
    spread(
        "peat depth below surf (m)",
        c.peat_best.iter().map(|s| s.depth_m).collect(),
    );
    spread(
        "peat run thickness (m)",
        c.peat_best.iter().map(|s| s.thick_m).collect(),
    );

    // Why: the candidate units' temperature against the onset. Burial here is a
    // few metres, so `gradient·depth` is a fraction of a degree and the SURFACE
    // term sets T — this table is the whole explanation of the count above.
    println!("\n  --- candidate-unit temperature vs the {COAL_ONSET_C} C onset ---");
    let t = &c.cand_temps;
    println!(
        "  candidate units {} | T (C): p05 {:.1}  p50 {:.1}  p95 {:.1}  max {:.1}",
        t.len(),
        percentile(t, 0.05),
        percentile(t, 0.50),
        percentile(t, 0.95),
        percentile(t, 1.0)
    );
    print!("  coal fraction vs trial onset:");
    for onset in [4.0, 8.0, 12.0, 16.0, 20.0, 22.0, 26.0] {
        print!(
            "  {onset:.0}C:{:.0}%",
            100.0 * t.iter().filter(|&&x| x >= onset).count() as f64 / t.len().max(1) as f64
        );
    }
    println!();

    // Did coal follow the warm crust? Split by geothermal gradient.
    warm_crust_split(f, &c);

    // ---- 2. the control: can this instrument SEE coal at all? -----------------
    if c.coal_seams.is_empty() {
        println!(
            "\n--- CONTROL (seed {CONTROL_SEED:#X}, Medium — the world organic.rs and \
             journal/0093 measured) ---"
        );
        println!("  a zero from an unproven instrument is not evidence; building the control…");
        let ctl = Pregen::run(WorldParams {
            seed: CONTROL_SEED,
            extent: EXTENT,
        });
        let cc = census(&ctl.deep);
        println!(
            "  control world: {} coal cells, {} coal runs, {} peat-only cells",
            cc.coal_cells,
            cc.coal_seams.len(),
            cc.peat_only_cells
        );
        if !cc.coal_seams.is_empty() {
            spread(
                "control coal thick (m)",
                cc.coal_seams.iter().map(|s| s.thick_m).collect(),
            );
            println!(
                "  control candidate T (C): p50 {:.1}, p95 {:.1}, max {:.1}",
                percentile(&cc.cand_temps, 0.50),
                percentile(&cc.cand_temps, 0.95),
                percentile(&cc.cand_temps, 1.0)
            );
            // The geotherm's claim scored on a world that HAS coal — the split
            // the production world can only answer with a column of zeros.
            warm_crust_split(&ctl.deep, &cc);
            println!(
                "\n  ⇒ the census DOES see coal when coal exists. The production world's zero is real."
            );
        } else {
            println!(
                "  ⇒ the control is ALSO zero — either the instrument is blind or coal is gone \
                 world-wide. Treat the production zero as unproven."
            );
        }
    }

    // ---- 1. the station ------------------------------------------------------
    let mut g = WorldGenerator::new(&pregen);
    let coal_picked = pick_candidates(f, &c.coal_best, VERIFY_BUDGET);
    let mut coal_stations: Vec<Station> = coal_picked
        .iter()
        .filter_map(|&s| verify(&mut g, f, s, MaterialId::COAL))
        .collect();
    coal_stations
        .sort_by(|a, b| score(b.depth_m, b.thick_m).total_cmp(&score(a.depth_m, a.thick_m)));

    println!(
        "\n--- VERIFICATION (record candidates ≤{BENCH_REACH_M} m deep, ≥1 m thick, on land) ---"
    );
    println!(
        "  coal candidates verified by generating their chunk column: {} → {} surfaced diggable coal",
        coal_picked.len(),
        coal_stations.len()
    );

    if !coal_stations.is_empty() {
        println!("\n--- STATION B — the coal station (best 3, best first) ---");
        for (n, s) in coal_stations.iter().take(3).enumerate() {
            print_station(f, &format!("STATION B{}", n + 1), s, "coal seam");
        }
    } else {
        println!(
            "\n=== STATION B: NULL FOR COAL ===\n  \
             No coal exists on the world dc-client boots (seed {SEED}, {}). There is no \
             station from which coal can be seen, because there is no coal.\n  \
             What the geotherm did here is leave the peat as PEAT — so the fallback station \
             below is the ground that came CLOSEST to coalifying and did not.",
            EXTENT.label()
        );
        // Fallback: the warmest, shallowest, thickest peat — where coal would be.
        let mut warm: Vec<Seam> = c
            .peat_best
            .iter()
            .copied()
            .filter(|s| {
                s.depth_m <= BENCH_REACH_M && s.thick_m >= 1.0 && f.surf[s.cell] > SEA_LEVEL_M + 5.0
            })
            .collect();
        warm.sort_by(|a, b| surface_temp_c(f, b.cell).total_cmp(&surface_temp_c(f, a.cell)));
        let mut picked: Vec<Seam> = Vec::new();
        for s in warm {
            if picked.len() >= VERIFY_BUDGET {
                break;
            }
            let (gx, gy) = ((s.cell % f.w) as i64, (s.cell / f.w) as i64);
            if picked.iter().all(|p| {
                let (px, py) = ((p.cell % f.w) as i64, (p.cell / f.w) as i64);
                (gx - px).abs().max((gy - py).abs()) >= 5
            }) {
                picked.push(s);
            }
        }
        let mut peat_stations: Vec<Station> = picked
            .iter()
            .filter_map(|&s| verify(&mut g, f, s, MaterialId::PEAT))
            .collect();
        peat_stations
            .sort_by(|a, b| score(b.depth_m, b.thick_m).total_cmp(&score(a.depth_m, a.thick_m)));
        println!(
            "\n--- STATION B (fallback: the warmest diggable PEAT — {} of {} verified) ---",
            peat_stations.len(),
            picked.len()
        );
        if peat_stations.is_empty() {
            println!(
                "  AND NULL AGAIN: no peat survived collapse as diggable PEAT either. There is \
                 nothing organic to walk to on this world."
            );
        } else {
            for (n, s) in peat_stations.iter().take(3).enumerate() {
                print_station(f, &format!("STATION B{} (PEAT)", n + 1), s, "peat seam");
            }
            // Nearest such station to Station A — one walk beats two.
            if let Some(near) = peat_stations.iter().min_by(|a, b| {
                let d = |s: &Station| {
                    dist_m(
                        (s.vx as f64 * VOXEL_M, s.vz as f64 * VOXEL_M),
                        STATION_A_XZ_M,
                    )
                };
                d(a).total_cmp(&d(b))
            }) {
                print_station(
                    f,
                    "STATION B-near-A (PEAT, closest to Station A)",
                    near,
                    "peat seam",
                );
            }
        }
    }

    // ---- 2b. the contrast ----------------------------------------------------
    println!("\n--- CONTRAST STATION (coal here, peat-only there) ---");
    if c.coal_seams.is_empty() {
        println!(
            "  NULL. A contrast needs two ends and this world has only one: {} peat-only cells \
             and {} coal cells. There is no pair of nearby cells that differ in coal, at any \
             distance, because coal does not occur.",
            c.peat_only_cells, c.coal_cells
        );
    } else if let Some(s) = coal_stations.first() {
        let here = world_m(f, s.cell);
        let nearest = (0..f.strata.len())
            .filter(|&i| c.peat_only[i] && interior(f.w, i) && f.surf[i] > SEA_LEVEL_M + 5.0)
            .min_by(|&a, &b| dist_m(world_m(f, a), here).total_cmp(&dist_m(world_m(f, b), here)));
        match nearest {
            None => println!("  NULL: no peat-only cell anywhere on this world."),
            Some(i) => {
                let (px, pz) = world_m(f, i);
                let d = dist_m((px, pz), here);
                println!(
                    "  nearest peat-only cell to B1: world ({px:.0} m, {pz:.0} m), {d:.0} m away"
                );
                println!(
                    "    surface {:.1} m, gradient {:.1} C/km, surface T {:.1} C   \
                     [B1: {:.1} m, {:.1} C/km, {:.1} C]",
                    f.surf[i],
                    gradient(f, i) * 1000.0,
                    surface_temp_c(f, i),
                    f.surf[s.cell],
                    gradient(f, s.cell) * 1000.0,
                    surface_temp_c(f, s.cell),
                );
                let (vx, vz) = idx_to_voxel(f, i);
                let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
                let peat_vox: u32 = column_scan(&mut g, cx, cz, MaterialId::PEAT)
                    .iter()
                    .map(|e| e.0)
                    .sum();
                println!("    that chunk column holds {peat_vox} PEAT voxels");
                if d > 3_000.0 {
                    println!(
                        "    VERDICT: {:.1} km apart — a flight, not a walk. Real in the data, \
                         not legible on foot in one session.",
                        d / 1000.0
                    );
                } else if peat_vox == 0 {
                    println!(
                        "    VERDICT: NULL — peat in the RECORD there but no peat voxels survive \
                         collapse, so the other end of the contrast is invisible."
                    );
                } else {
                    println!(
                        "    VERDICT: walkable contrast — coal at B1, peat {:.0} m away.",
                        d
                    );
                }
            }
        }
    }

    // ---- 4. the two-station walk ---------------------------------------------
    println!("\n--- ONE WALK? ---");
    println!(
        "  STATION A (saprolite front): ({:.0} m, {:.0} m)",
        STATION_A_XZ_M.0, STATION_A_XZ_M.1
    );
    println!(
        "  Both stations are reached by teleport (`client_player_pose_set`), so separation \
         costs nothing but the distances above are what a walked traverse would be."
    );
}

/// **The gate's view of this instrument** (journal/0103).
///
/// The station search is a recommendation judged by the eye, and it costs a
/// Medium world plus ~25 chunk-column generations — neither belongs in the gate.
/// What *is* a claim is the census beneath it, and its properties are
/// **scale-free**:
///
/// - *the run decomposition is an itemisation of its own total* — the sum of a
///   cell's contiguous coal runs must equal the cell's total coal thickness, and
///   likewise for peat. Arithmetic over one column: wrong at every world size if
///   wrong at all (exactly the shape of defect `flow_cost_probe` shipped twice).
/// - *the pools are disjoint* — a cell marked peat-only carries no coal run.
/// - *every run is non-degenerate* — non-negative depth, positive thickness.
///
/// **Deliberately NOT asserted: that coal exists.** On the production world
/// (seed 1337, Medium) it does not — the candidate units sit ~14 °C below
/// `COAL_ONSET_C` — and this instrument's job is to report that honestly, not to
/// fail the build over an appearance call the user owns. The *control* world
/// carries the "the census can see coal" evidence, and it lives in `main` where
/// it costs a second Medium pregen.
///
/// Run at [`Extent::Small`].
#[cfg(test)]
mod gate {
    use super::*;

    #[test]
    fn the_run_decomposition_itemises_the_record_it_reads() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: Extent::Small,
        });
        let f = &pregen.deep;
        let c = census(f);
        assert!(
            c.candidate_cells > 0,
            "the world grew no coalification candidates at all — there is no record to itemise"
        );

        let mut per_cell: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
        for s in &c.coal_seams {
            *per_cell.entry(s.cell).or_default() += s.thick_m;
            assert!(
                s.depth_m >= 0.0 && s.thick_m > 0.0,
                "run at cell {} is degenerate: depth {} m, thickness {} m",
                s.cell,
                s.depth_m,
                s.thick_m
            );
            assert!(
                !c.peat_only[s.cell],
                "cell {} is marked peat-only yet carries a coal run",
                s.cell
            );
        }
        for (&cell, &sum) in &per_cell {
            let total: f64 = f.strata[cell]
                .units
                .iter()
                .filter(|u| u.tag.biota == Biofacies::Coal)
                .map(|u| u.thickness_m)
                .sum();
            assert!(
                (sum - total).abs() < 1e-9,
                "cell {cell}: contiguous coal runs sum to {sum} m but the record holds {total} m \
                 — the run decomposition is losing units"
            );
        }

        // The peat pool is the fallback station source; itemise it the same way.
        for s in &c.peat_best {
            let total: f64 = f.strata[s.cell]
                .units
                .iter()
                .filter(|u| u.tag.biota == Biofacies::Peat)
                .map(|u| u.thickness_m)
                .sum();
            assert!(
                s.thick_m > 0.0 && s.thick_m <= total + 1e-9,
                "cell {}: shallowest peat run {} m exceeds the cell's {} m of peat",
                s.cell,
                s.thick_m,
                total
            );
        }
    }
}
