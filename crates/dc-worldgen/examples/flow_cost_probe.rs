//! **The flow-record cost probe** (S19 — `docs/design/flow.md` § 9 Q1/Q2).
//!
//! `flow.md` was ratified 2026-07-25 with its cost UNMEASURED: *"Face flux ×
//! slots × chapters is unmeasured. Ratified stance: gen is free, cost taken as it
//! comes, faithfulness above all. Resident memory is still sacred — measure, then
//! decide what is resident vs re-derived."* This probe is that measurement.
//!
//! It measures **today's world geometry** — the multipliers the flow record will
//! be built on — and projects the resident cost of four candidate layouts from
//! them. It writes nothing and generates nothing; it reads the production
//! `DeepField` and does arithmetic.
//!
//! What it measures:
//! 1. grid geometry (cells, `cell_m`, chapters, epochs, epochs/chapter),
//! 2. the **units-per-cell distribution** (min/mean/median/p95/max/total) — the
//!    slot multiplier that decides everything — split land / marine / empty,
//! 3. the **causal (slot, chapter) pair count** `Σ_units (K − unit.chapter)`: a
//!    slot deposited in chapter `c` cannot carry flow facts from before `c`,
//!    so this is the honest dense ceiling, not `slots × K`,
//! 4. today's `DeepField` residency, decomposed by part,
//! 5. projections for layouts L1–L4 and the 1× / 4× / 10× knees.
//!
//! Every assumption is a named `const` below so it is falsifiable, not buried.
//!
//! `cargo run --release -p dc-worldgen --example flow_cost_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{
    DeepField, DeepOverrides, DeepStrata, DepUnit, FactLedger, SEA_LEVEL_M, build_field_with,
    production_config,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;

// ---------------------------------------------------------------------------
// ASSUMPTIONS (stated, so they can be argued with)
// ---------------------------------------------------------------------------

/// **A1 — flux magnitude width.** `f32`. Flux is a rate the refinement tier uses
/// as a Dirichlet boundary condition; f32 carries ~7 significant digits, which is
/// far beyond what a 460 m cell's discharge is known to. f64 would double every
/// number below.
const BYTES_FLUX: usize = 4;

/// **A2 — dense lateral face fan-out, D4.** Von Neumann: N/E/S/W. Faces are
/// *shared*, so a dense grid stores each face once: cell `i` owns its E and S
/// faces (the N/W faces belong to its neighbours). Hence **2 owned lateral faces
/// per cell** in the shared layout.
const LATERAL_OWNED_D4: usize = 2;

/// **A3 — dense lateral face fan-out, D8.** Moore: the four cardinals plus four
/// diagonals. Owned: E, S, SE, SW → **4 owned lateral faces per cell**. The
/// existing drainage solve is D8 (`Erosion::recv` is a D8 receiver), so D8 is the
/// like-for-like successor; D4 is the cheaper honest alternative.
const LATERAL_OWNED_D8: usize = 4;

/// **A4 — slot alignment across a lateral face.** Strata slots do *not* align
/// between adjacent columns (slot 3 in cell A is not slot 3's depth in cell B).
/// A truly *shared* per-slot face therefore needs a pairing rule the record does
/// not have today. Two sub-models are reported:
///   - **shared**: `LATERAL_OWNED_*` entries per (cell, slot) — assumes slot index
///     pairs across the face;
///   - **per-cell**: `2 ×` that (every cell stores all its directions, no sharing)
///     — assumes the pairing is resolved at read time by depth, not index.
/// The per-cell model is the safe upper bound; the shared model is the prize
/// `flow.md` § 2.2 claims ("a face is shared … agree from both sides by
/// construction").
const UNSHARED_MULTIPLIER: usize = 2;

/// **A5 — vertical faces.** slot↔slot within a column: `slots − 1` per cell,
/// which is ≈ 1 per slot. Modelled as exactly 1 per slot (over-counts by one
/// face per non-empty cell; < 0.5 % here).
const VERTICAL_PER_SLOT: usize = 1;

/// **A6 — boundary faces.** One top (atmosphere) face per cell per chapter, plus
/// one seaward/base-level face per marine cell per chapter. Bottom-of-column
/// (basement) faces are not counted: the record has no slot below unit 0.
const TOP_FACES_PER_CELL: usize = 1;

/// **A7 — sparse entry cost.** A non-zero face becomes `(key: u32, flux: f32)`.
/// The `u32` key packs (direction, slot) within a per-cell-per-chapter CSR row;
/// 460 m grids are 550² so a global key would need `u64`, which is why the layout
/// is CSR rather than a flat hash map.
const BYTES_SPARSE_ENTRY: usize = 4 + BYTES_FLUX;

/// **A8 — sparse index overhead.** A CSR row offset (`u32`) per cell per chapter,
/// paid whether or not the row is empty. This is the cost sparsity cannot escape.
const BYTES_CSR_ROW: usize = 4;

/// **A9 — the § 1.3 atom's extra fields, per fact.** `form/phase` is 1 bit,
/// `fluid` a `MaterialId` (16-bit in this tree), `cause` a `Cause` discriminant
/// (1 byte — the enum has < 16 inhabitants), `load` an `f32` (metres of sediment
/// carried per unit flux). Packed: `u16 fluid + u8 cause|form + u8 pad + f32 load`
/// = 8 bytes. This is the *incremental* cost over a bare flux entry.
const BYTES_ATOM_EXTRA: usize = 8;

/// **A10 — sparsity fractions modelled.** The true fraction of faces carrying
/// meaningful flux cannot be known until the sibling's solve exists; these bracket
/// it. 1 % ≈ "only trunk channels"; 5 % ≈ "channel network"; 20 % ≈ "every cell
/// that ever saw water".
const SPARSITY: [f64; 4] = [0.01, 0.05, 0.20, 1.00];

const MIB: f64 = 1024.0 * 1024.0;

fn mib(b: f64) -> f64 {
    b / MIB
}

/// Order statistic of a pre-sorted slice.
fn quantile(sorted: &[usize], q: f64) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let k = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[k.min(sorted.len() - 1)]
}

struct Dist {
    n: usize,
    min: usize,
    mean: f64,
    median: usize,
    p95: usize,
    max: usize,
    total: usize,
}

fn dist(mut v: Vec<usize>) -> Dist {
    v.sort_unstable();
    let total: usize = v.iter().sum();
    Dist {
        n: v.len(),
        min: v.first().copied().unwrap_or(0),
        mean: if v.is_empty() {
            0.0
        } else {
            total as f64 / v.len() as f64
        },
        median: quantile(&v, 0.5),
        p95: quantile(&v, 0.95),
        max: v.last().copied().unwrap_or(0),
        total,
    }
}

fn print_dist(label: &str, d: &Dist) {
    println!(
        "  {label:<26} n={:<7} min {:<3} mean {:<8.3} med {:<3} p95 {:<3} max {:<4} total {}",
        d.n, d.min, d.mean, d.median, d.p95, d.max, d.total
    );
}

/// The baseline residency decomposition — the same accounting
/// [`DeepField::resident_bytes`] does, but itemised so each part's fraction of
/// the whole is visible. Asserted equal to `resident_bytes()` at run time, so
/// this stays a *view* of the shipped helper rather than a second mechanism.
struct Baseline {
    surf: usize,
    regolith: usize,
    area: usize,
    exhum: usize,
    t_crust: usize,
    geotherm: usize,
    recv: usize,
    lake: usize,
    strata_structs: usize,
    strata_heap: usize,
    ledger_structs: usize,
    ledger_heap: usize,
}

impl Baseline {
    fn of(f: &DeepField) -> Self {
        let f64s = std::mem::size_of::<f64>();
        Self {
            surf: f.surf.len() * f64s,
            regolith: f.regolith.len() * f64s,
            area: f.area.len() * f64s,
            exhum: f.exhum.len() * f64s,
            t_crust: f.t_crust.len() * f64s,
            geotherm: f.geotherm.len() * f64s,
            recv: f.recv.len() * std::mem::size_of::<i32>(),
            lake: f.lake.len(),
            strata_structs: f.strata.len() * std::mem::size_of::<DeepStrata>(),
            strata_heap: f.strata.iter().map(DeepStrata::heap_bytes).sum(),
            ledger_structs: f.ledgers.len() * std::mem::size_of::<FactLedger>(),
            ledger_heap: f.ledgers.iter().map(FactLedger::footprint_bytes).sum(),
        }
    }

    fn rows(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("surf (f64/cell)", self.surf),
            ("regolith (f64/cell)", self.regolith),
            ("area (f64/cell)", self.area),
            ("exhum (f64/cell)", self.exhum),
            ("t_crust (f64/cell)", self.t_crust),
            ("geotherm (f64/cell)", self.geotherm),
            ("recv (i32/cell)", self.recv),
            ("lake (bool/cell)", self.lake),
            ("strata structs", self.strata_structs),
            ("strata heap (DepUnit)", self.strata_heap),
            ("ledger structs", self.ledger_structs),
            ("ledger heap (Fact)", self.ledger_heap),
        ]
    }

    fn total(&self) -> usize {
        self.rows().iter().map(|(_, b)| b).sum()
    }
}

/// The measured geometry the projections multiply.
struct Geom {
    cells: usize,
    chapters: usize,
    /// Σ over units of 1 — total stratum slots in the world.
    slots: usize,
    /// Σ over units of `(K − unit.chapter)` — the causally-possible (slot, chapter)
    /// pairs (a slot cannot carry a fact from before it was deposited).
    slot_chapters_causal: usize,
    /// `slots × K` — the naive dense ceiling, for contrast.
    slot_chapters_naive: usize,
    marine_cells: usize,
    /// Cells whose flow record could only ever be a top/boundary face: no slots.
    empty_cells: usize,
}

/// One projected layout: a name, the total bytes, and the arithmetic that got
/// there (printed verbatim so the number can be re-derived by hand).
struct Layout {
    name: String,
    bytes: f64,
    arithmetic: String,
}

fn main() {
    println!("=== S19 flow-record cost probe (docs/design/flow.md § 9 Q1) ===");
    println!("seed {SEED}, extent {}\n", EXTENT.label());

    let t_all = Instant::now();
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let t_pregen = t0.elapsed().as_secs_f64();

    // The PRODUCTION field is `pregen.deep` (weather_inventory OFF — the shipped
    // default). `s18_weathering_tour` builds a second field with the flag ON; we
    // build that too, because the ledger is the only part of today's residency
    // that is itself a fact record, and it is the closest structural analogue of
    // the flow record we already pay for.
    let t1 = Instant::now();
    let field_wi = build_field_with(
        &pregen.grid,
        SEED,
        &DeepOverrides {
            weather_inventory: Some(true),
            ..DeepOverrides::default()
        },
    );
    let t_wi = t1.elapsed().as_secs_f64();

    let field = &pregen.deep;
    let cfg = production_config(&pregen.grid, SEED);

    // ---- 1. geometry ------------------------------------------------------
    let w = field.w;
    let cells = w * w;
    let chapters = cfg.chapters as usize;
    let epochs = cfg.iterations as usize;
    println!("--- 1. measured grid geometry ---");
    println!("  deep grid            : {w} x {w} = {cells} cells");
    println!("  cell edge            : {:.1} m (DEEP_CELL_M target)", field.cell_m);
    println!(
        "  covered extent       : {:.1} km per side",
        w as f64 * field.cell_m / 1000.0
    );
    println!("  epochs (iterations)  : {epochs}");
    println!("  chapters K           : {chapters}");
    println!(
        "  epochs per chapter   : {:.1}",
        epochs as f64 / chapters as f64
    );
    println!("  sizeof(DepUnit)      : {} B", std::mem::size_of::<DepUnit>());
    println!(
        "  sizeof(DeepStrata)   : {} B\n",
        std::mem::size_of::<DeepStrata>()
    );

    // ---- 2/3. the unit distribution, split land / marine ------------------
    let mut all: Vec<usize> = Vec::with_capacity(cells);
    let mut land: Vec<usize> = Vec::new();
    let mut marine: Vec<usize> = Vec::new();
    let mut chapter_hist = vec![0usize; 256];
    let mut slot_chapters_causal = 0usize;
    let mut empty_cells = 0usize;
    for i in 0..cells {
        let n = field.strata[i].units.len();
        all.push(n);
        if n == 0 {
            empty_cells += 1;
        }
        if field.surf[i] > SEA_LEVEL_M {
            land.push(n);
        } else {
            marine.push(n);
        }
        for u in &field.strata[i].units {
            chapter_hist[u.chapter as usize] += 1;
            slot_chapters_causal += chapters.saturating_sub(u.chapter as usize);
        }
    }
    let d_all = dist(all);
    let d_land = dist(land);
    let d_marine = dist(marine);

    println!("--- 2. units (stratum slots) per cell ---");
    print_dist("all cells", &d_all);
    print_dist("land (surf > 0 m)", &d_land);
    print_dist("marine (surf <= 0 m)", &d_marine);
    println!(
        "  cells with EMPTY record   : {empty_cells} ({:.1}% of {cells})",
        100.0 * empty_cells as f64 / cells as f64
    );
    println!(
        "  land / marine split       : {} land ({:.1}%) / {} marine ({:.1}%)\n",
        d_land.n,
        100.0 * d_land.n as f64 / cells as f64,
        d_marine.n,
        100.0 * d_marine.n as f64 / cells as f64
    );

    println!("--- 2b. slot chapter stamps (the flow record's time axis) ---");
    for (c, &n) in chapter_hist.iter().enumerate().take(chapters) {
        println!(
            "  chapter {c:<2}: {n:>8} slots ({:.1}%)",
            100.0 * n as f64 / d_all.total.max(1) as f64
        );
    }
    let slot_chapters_naive = d_all.total * chapters;
    println!(
        "  (slot, chapter) pairs  naive slots*K = {slot_chapters_naive}, \
         causal Sum(K - chapter) = {slot_chapters_causal}  ({:.1}% of naive)\n",
        100.0 * slot_chapters_causal as f64 / slot_chapters_naive.max(1) as f64
    );

    let geom = Geom {
        cells,
        chapters,
        slots: d_all.total,
        slot_chapters_causal,
        slot_chapters_naive,
        marine_cells: d_marine.n,
        empty_cells,
    };

    // ---- 4. baseline residency -------------------------------------------
    let base = Baseline::of(field);
    let base_total = base.total();
    assert_eq!(
        base_total,
        field.resident_bytes(),
        "itemised baseline must agree with DeepField::resident_bytes (the shipped helper)"
    );
    println!("--- 4. today's DeepField residency (production, weather_inventory OFF) ---");
    for (name, b) in base.rows() {
        println!(
            "  {name:<24} {:>12} B  {:>8.2} MiB  {:>6.2}%",
            b,
            mib(b as f64),
            100.0 * b as f64 / base_total as f64
        );
    }
    println!(
        "  {:<24} {:>12} B  {:>8.2} MiB  (DeepField::resident_bytes agrees)\n",
        "TOTAL", base_total, mib(base_total as f64)
    );

    let base_wi = Baseline::of(&field_wi);
    println!(
        "  for contrast, weather_inventory ON: total {:.2} MiB (+{:.2} MiB, ledger heap {:.2} MiB \
         over {} facts)",
        mib(base_wi.total() as f64),
        mib(base_wi.total() as f64 - base_total as f64),
        mib(base_wi.ledger_heap as f64),
        field_wi
            .ledgers
            .iter()
            .map(FactLedger::total_facts)
            .sum::<usize>()
    );
    println!(
        "  (the ledger is today's only fact-shaped record; it is the structural analogue \
         of the flow record)\n"
    );

    // ---- 5. projections ---------------------------------------------------
    let layouts = project(&geom, base_total as f64);
    println!("--- 5. projected flow-record cost (T = today's DeepField = {:.2} MiB) ---", mib(base_total as f64));
    println!(
        "  {:<58} {:>12}  {:>9}  {:>8}",
        "layout", "bytes", "MiB", "x today"
    );
    for l in &layouts {
        println!(
            "  {:<58} {:>12.0}  {:>9.2}  {:>7.2}x",
            l.name,
            l.bytes,
            mib(l.bytes),
            l.bytes / base_total as f64
        );
    }
    println!("\n  arithmetic:");
    for l in &layouts {
        println!("    {:<58} = {}", l.name, l.arithmetic);
    }

    // ---- 6. the knee ------------------------------------------------------
    println!("\n--- 6. the knee: what crosses 1x / 4x / 10x of T ---");
    knee_sparsity(&geom, base_total as f64);
    knee_slots(&geom, base_total as f64);
    knee_chapters(&geom, base_total as f64);

    println!(
        "\ntimings: Pregen::run (incl. production deep-time run) {t_pregen:.1} s; \
         second field with weather_inventory ON {t_wi:.1} s; total {:.1} s",
        t_all.elapsed().as_secs_f64()
    );
}

/// Dense lateral entries for one (slot, chapter) pair, both fan-outs and both
/// sharing models.
fn lateral_faces(owned: usize, shared: bool) -> usize {
    if shared {
        owned
    } else {
        owned * UNSHARED_MULTIPLIER
    }
}

fn project(g: &Geom, _t: f64) -> Vec<Layout> {
    let mut out = Vec::new();
    let sc = g.slot_chapters_causal as f64;
    let flux = BYTES_FLUX as f64;

    // ---- L1: dense lateral only ------------------------------------------
    for (tag, owned) in [("D4", LATERAL_OWNED_D4), ("D8", LATERAL_OWNED_D8)] {
        for (stag, shared) in [("shared", true), ("per-cell", false)] {
            let f = lateral_faces(owned, shared) as f64;
            let bytes = sc * f * flux;
            out.push(Layout {
                name: format!("L1 dense lateral, {tag} {stag}"),
                bytes,
                arithmetic: format!(
                    "{} slot-chapters x {f} faces x {flux} B",
                    g.slot_chapters_causal
                ),
            });
        }
    }

    // ---- L2: + vertical + boundary ---------------------------------------
    // Vertical: 1 per (slot, chapter). Boundary: top face per cell per chapter,
    // plus a seaward face per marine cell per chapter.
    let boundary = ((g.cells * TOP_FACES_PER_CELL + g.marine_cells) * g.chapters) as f64;
    for (tag, owned) in [("D4", LATERAL_OWNED_D4), ("D8", LATERAL_OWNED_D8)] {
        for (stag, shared) in [("shared", true), ("per-cell", false)] {
            let f = lateral_faces(owned, shared) as f64 + VERTICAL_PER_SLOT as f64;
            let bytes = sc * f * flux + boundary * flux;
            out.push(Layout {
                name: format!("L2 = L1 + vertical + boundary, {tag} {stag}"),
                bytes,
                arithmetic: format!(
                    "{} slot-chapters x {f} faces x {flux} B + {boundary:.0} boundary faces x {flux} B",
                    g.slot_chapters_causal
                ),
            });
        }
    }

    // ---- L3: sparse (D8 per-cell face universe — the pessimistic universe) --
    let universe = sc * (lateral_faces(LATERAL_OWNED_D8, false) + VERTICAL_PER_SLOT) as f64 + boundary;
    let csr = (g.cells * g.chapters * BYTES_CSR_ROW) as f64;
    for s in SPARSITY {
        let bytes = universe * s * BYTES_SPARSE_ENTRY as f64 + csr;
        out.push(Layout {
            name: format!("L3 sparse @ {:>4.0}% of faces", s * 100.0),
            bytes,
            arithmetic: format!(
                "{universe:.0} faces x {s} x {} B + CSR {csr:.0} B",
                BYTES_SPARSE_ENTRY
            ),
        });
    }

    // ---- L4: L3 + the atom's extra fields ---------------------------------
    for s in SPARSITY {
        let per = (BYTES_SPARSE_ENTRY + BYTES_ATOM_EXTRA) as f64;
        let bytes = universe * s * per + csr;
        out.push(Layout {
            name: format!("L4 sparse + atom @ {:>4.0}% of faces", s * 100.0),
            bytes,
            arithmetic: format!("{universe:.0} faces x {s} x {per} B + CSR {csr:.0} B"),
        });
    }
    out
}

/// At what sparsity does the sparse layout cross 1x / 4x / 10x of today?
fn knee_sparsity(g: &Geom, t: f64) {
    let boundary = ((g.cells * TOP_FACES_PER_CELL + g.marine_cells) * g.chapters) as f64;
    let universe = g.slot_chapters_causal as f64
        * (lateral_faces(LATERAL_OWNED_D8, false) + VERTICAL_PER_SLOT) as f64
        + boundary;
    let csr = (g.cells * g.chapters * BYTES_CSR_ROW) as f64;
    for (label, per) in [
        ("L3 (flux only, 8 B/face)", BYTES_SPARSE_ENTRY as f64),
        (
            "L4 (flux + atom, 16 B/face)",
            (BYTES_SPARSE_ENTRY + BYTES_ATOM_EXTRA) as f64,
        ),
    ] {
        print!("  sparsity crossing, {label:<28}:");
        for m in [1.0, 4.0, 10.0] {
            let s = (m * t - csr) / (universe * per);
            if s <= 0.0 {
                print!("  {m:.0}x: already exceeded by CSR index alone");
            } else if s > 1.0 {
                print!("  {m:.0}x: never (>100% dense)");
            } else {
                print!("  {m:.0}x: {:.2}%", s * 100.0);
            }
        }
        println!();
    }
    println!(
        "    (CSR index floor alone = {:.2} MiB = {:.3}x T — paid at ANY sparsity)",
        csr / MIB,
        csr / t
    );
}

/// At what mean units-per-cell does the DENSE layout cross the multiples? Holds
/// the measured causal/naive ratio and chapter count fixed and scales slots.
fn knee_slots(g: &Geom, t: f64) {
    let causal_ratio = g.slot_chapters_causal as f64 / g.slot_chapters_naive.max(1) as f64;
    let per_slot_chapter =
        (lateral_faces(LATERAL_OWNED_D8, false) + VERTICAL_PER_SLOT) as f64 * BYTES_FLUX as f64;
    let bytes_per_mean_unit =
        g.cells as f64 * g.chapters as f64 * causal_ratio * per_slot_chapter;
    let measured_mean = g.slots as f64 / g.cells as f64;
    print!("  mean units/cell crossing, L2 D8 per-cell dense:");
    for m in [1.0, 4.0, 10.0] {
        print!("  {m:.0}x: {:.2}", m * t / bytes_per_mean_unit);
    }
    println!("   [measured mean = {measured_mean:.2}]");
}

/// At what chapter count K does the dense layout cross the multiples? Scales K
/// with the per-chapter slot population held at the measured value.
fn knee_chapters(g: &Geom, t: f64) {
    let per_chapter_bytes = g.slot_chapters_causal as f64 / g.chapters as f64
        * (lateral_faces(LATERAL_OWNED_D8, false) + VERTICAL_PER_SLOT) as f64
        * BYTES_FLUX as f64;
    print!("  chapter-count crossing, L2 D8 per-cell dense:");
    for m in [1.0, 4.0, 10.0] {
        print!("  {m:.0}x: K = {:.1}", m * t / per_chapter_bytes);
    }
    println!("   [measured K = {}, empty cells {}]", g.chapters, g.empty_cells);
}
