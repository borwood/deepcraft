//! **S20 — what should the fact ledger COST, and where should it LIVE?**
//!
//! journal/0106 (R3) measured the per-depth weathering arc and moved the blocker:
//! gen time is affordable (25.7 s → 46.4 s), **residency is not** — the
//! `LedgerField` sidecar projects 17.45 MiB → 973 MiB and it is *resident*, it
//! ships inside the `DeepField`.
//!
//! The design conversation named the discriminator. There are **two queries with
//! opposite profiles**:
//!
//! - *"what is this rock?"* — needs **only the fold**, for **every chunk, forever**.
//!   Hot, global, must be resident.
//! - *"what happened here?"* — needs **the axes**, **rarely, at one voxel**. Cold,
//!   local, pageable.
//!
//! The crisis is storing a cold/local query's data at the residency of a hot/global
//! one. And the constraint that rules out the naive fix: **compaction must not mean
//! deletion** — collapsing to a per-slot scalar and discarding the facts makes a
//! summary the authority, the exact defect the arc exists to kill
//! (`ARCHITECTURE.md` § *"A summary is not an authority"*).
//!
//! **THE ENCODING HALF IS NOW SHIPPED** (journal/0108, S20 option 2c ratified
//! 2026-07-25). `Fact` is 8 bytes: a declared [`EdgeId`](dc_worldgen::deeptime::EdgeId)
//! and an `f32` fraction, with the narrowing done once at
//! `LedgerField::from_accumulators`. So this probe's § 2 stopped being a menu of
//! candidates and became a **guard on the shipped width**: the local shapes below
//! are kept because they are what makes the § 3.1 finding — *an edge id ALONE saves
//! nothing* — checkable, and that finding is now also asserted against a shipped
//! type (the gen-time accumulator `Fact<FracM>`, which has the edge id, does not
//! have the narrowing, and is still 16 B).
//!
//! **The paging arm builds no part of the arc and changes nothing shipped.** It
//! writes a **geometry-faithful synthetic file** to a temp directory and deletes it;
//! nothing here is wired into a run. The pager is the reserved continuation slice.
//!
//! # What is measured
//!
//! 1. **Geometry, re-derived** — not trusted from journal/0106. The `LedgerField`
//!    itemisation is reconstructed byte-for-byte against the record's own measured
//!    `footprint_bytes()`, then evaluated at the per-depth slot count using the
//!    causal triangle (a slot deposited in chapter `c` cannot weather before `c`).
//! 2. **Encoding** — `size_of` on five fact layouts (the pre-slice one; edge-id
//!    only; f32 only; both — which is what shipped; both as struct-of-arrays), plus
//!    the **per-world edge dictionary** the production record actually inhabits.
//! 3. **Paging** — a real file at the projected per-depth geometry, with page-in
//!    latency measured **cold** (Windows `FILE_FLAG_NO_BUFFERING`, so the read goes
//!    to the device and not to the OS page cache) and **warm** (buffered, cache
//!    hot), over a scattered deterministic sample of cells.
//! 4. **f32** — now that the record IS narrowed, two complementary things: the
//!    **resolution bound** the stored values themselves imply, over every weathered
//!    cell of the production record; and the **measured** accumulator→resident error
//!    on a reproduction of the persist step at the production fold depth. Both
//!    against the scale that matters: **one eighth of a voxel = 0.1125 m**.
//! 5. **Axis-drops** — resident MiB for dropping the chapter axis, the agent axis,
//!    or both, computed from the record's own measured multiplicities.
//!
//! `cargo run --release -p dc-worldgen --example s20_ledger_residency_probe`

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use dc_worldgen::deeptime::inventory::{
    BEDROCK_SEAM_MATERIAL, EdgeDict, F32_RELATIVE_RESOLUTION, Fact, FracM, InvForm,
    stored_fold_tolerance,
};
use dc_worldgen::deeptime::{
    DeepConfig, DeepStrata, WEATHERING_AGENTS, WeatherInputs, agent_share, build_field_cfg,
    empty_accumulator, finalize_ledgers, production_config, weather_bedrock_epoch,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The world `dc-client` boots — `BENCH_SEED` (`dc-client/src/bench.rs`) at
/// `WORLDGEN_EXTENT` (`dc-client/src/authority.rs`). A residency question is a
/// production-scale question, so the *report* runs here. (journal/0106's
/// `production_field` lesson: a helper named for an environment must BE it.)
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;

/// `SlotRun` is `{ slot: u32, start: u32 }` — private to `inventory.rs`, so its
/// size is restated here and then **proved** by [`Residency::check`], which
/// reconstructs the measured footprint byte-for-byte from this itemisation.
const SLOT_ROW_BYTES: usize = 8;
/// `LedgerField::cell_row_start` is one `u32` per cell plus a terminator.
const CELL_OFFSET_BYTES: usize = 4;

/// **One eighth of a voxel, in metres** — the scale the f32 error is judged
/// against. A deep quantity smaller than this cannot change a collapsed voxel.
/// The deep tier is fractional metres and quantizes to eighths only at collapse
/// (material-behavior.md §1).
const EIGHTH_M: f64 = 0.1125;

/// Cells sampled for the page-in latency measurement. Each cold read is a device
/// round trip, so this is the knob that sets the probe's I/O time.
const PAGE_SAMPLES: usize = 400;

/// Windows `FILE_FLAG_NO_BUFFERING` — bypasses the system page cache entirely, so
/// a read is a device read. Requires sector-aligned offset, length and buffer.
#[cfg(windows)]
const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
/// The alignment `FILE_FLAG_NO_BUFFERING` demands. 4096 satisfies both 512e and
/// 4Kn devices.
const SECTOR: u64 = 4096;

// ===========================================================================
// 1. GEOMETRY — re-derived from the production record, not trusted
// ===========================================================================

/// The `LedgerField` footprint, itemised so the same itemisation can be evaluated
/// at a different fact count / fact width.
#[derive(Clone, Copy)]
struct Residency {
    cells: usize,
    facts: usize,
    fact_bytes: usize,
    slot_rows: usize,
}

impl Residency {
    fn bytes(self) -> usize {
        self.facts * self.fact_bytes
            + self.slot_rows * SLOT_ROW_BYTES
            + (self.cells + 1) * CELL_OFFSET_BYTES
    }
    /// The itemisation must reconstruct the record's own measured footprint
    /// exactly — `finalize_ledgers` exact-sizes every array (journal/0100), so
    /// capacity == length and there is no slack to hide an error in.
    fn check(self, measured: usize) -> bool {
        self.bytes() == measured
    }
}

struct Geometry {
    extent: Extent,
    cells: usize,
    chapters: u32,
    epochs: u32,
    /// Cells that carry at least one weathering fact (the CSR sparsity numerator).
    weathering_cells: usize,

    // --- today ---
    today_facts: usize,
    today_rows: usize,
    today_measured_bytes: usize,
    /// `(cell, chapter)` pairs the pass actually fired in.
    cell_chapter_firings: u128,
    /// Measured, not assumed: `today_facts / cell_chapter_firings`. Should land on
    /// 3.0 (one fact per weathering agent) and is *reported* rather than hardcoded.
    facts_per_firing: f64,

    // --- per-depth, honouring the causal triangle ---
    /// `(cell, slot, chapter)` visits.
    pd_visits: u128,
    /// Distinct `(cell, slot)` that ever carry a fact — the CSR row count.
    pd_rows: usize,
    /// Facts a per-depth run would commit.
    pd_facts: usize,
    /// Per cell, the facts a per-depth run would commit there — the run lengths the
    /// paged prototype lays out on disk.
    pd_facts_per_cell: Vec<u32>,

    // --- encoding ---
    /// The **per-world edge dictionary** the production record inhabits — the thing
    /// a persisted ledger ships beside its facts (S20 § 3.2), built by the shipped
    /// `LedgerField::edge_dictionary`, not modelled here.
    edge_dict: EdgeDict,

    // --- f32 ---
    f32: F32Error,
}

/// The cost of storing `fraction_m` as f32 and widening on read, as the **bound the
/// stored values themselves imply**.
///
/// The shipped record is already narrowed, so the f64 original is gone and a direct
/// difference is not recoverable from it. The bound is: one round-to-nearest moves a
/// value by at most `2^-24` of itself, and *widen on read, sum in f64* means a fold's
/// total narrowing error is at most the SUM of its summands' roundings — never a
/// product, and never compounding, because the accumulator never narrows. The
/// **direct** measurement of the persist step is [`narrowing_audit`].
#[derive(Clone, Copy, Default)]
struct F32Error {
    /// Cells whose fold was bounded.
    cells: usize,
    /// Max / mean bound on |f64 fold − f32-stored fold|, metres.
    max_bound_m: f64,
    mean_bound_m: f64,
    /// The largest fold compared (metres) — the magnitude the error rides on.
    max_fold_m: f64,
    /// Max relative error over cells whose fold exceeds EPS.
    max_rel: f64,
    /// Individual stored `fraction_m` magnitudes (metres).
    frac_min_m: f64,
    frac_max_m: f64,
    /// Max |simple Σ − `weathering_product_m`| — the cross-check that the simple
    /// sum used here IS the fold the consumer reads.
    model_disagreement_m: f64,
    /// Summands in the deepest fold (facts folded for one cell's bedrock slot).
    max_summands: usize,
}

fn measure_geometry(extent: Extent) -> Geometry {
    let pregen = Pregen::run(WorldParams { seed: SEED, extent });
    let cfg = DeepConfig {
        weather_inventory: true,
        ..production_config(&pregen.grid, SEED)
    };
    let t = Instant::now();
    let field = build_field_cfg(&pregen.grid, &cfg);
    println!("  (deep run: {:.1} s)", t.elapsed().as_secs_f64());

    let cells = field.strata.len();
    let chapters = cfg.chapters.max(1);

    let mut cell_chapter_firings = 0u128;
    let mut pd_visits = 0u128;
    let mut pd_rows = 0usize;
    let mut weathering_cells = 0usize;
    let mut pd_facts_per_cell = vec![0u32; cells];
    let mut f32err = F32Error {
        frac_min_m: f64::INFINITY,
        ..F32Error::default()
    };
    let mut err_sum = 0.0f64;

    for (i, s) in field.strata.iter().enumerate() {
        let bedrock_slot = s.units.len();
        let Some(view) = field.ledgers.get(i) else {
            continue;
        };
        let facts = view.facts_for(bedrock_slot);
        if facts.is_empty() {
            continue;
        }
        weathering_cells += 1;

        // --- the chapters this cell actually fired in ---
        let mut fired: Vec<u8> = facts.iter().map(Fact::chapter).collect();
        fired.sort_unstable();
        fired.dedup();

        // --- the causal triangle: units are appended bottom-up and `chapter` is
        //     non-decreasing along the record, so the slots present at chapter c are
        //     a PREFIX of the final record. +1 for the bedrock seam span, present
        //     from epoch 0. (Units eroded away before the end are gone from the
        //     final record, so this is a LOWER BOUND on the depth a firing saw.)
        let mut slots_by_chapter = Vec::with_capacity(chapters as usize);
        let mut k = 0usize;
        for c in 0..chapters {
            while k < s.units.len() && u32::from(s.units[k].chapter) <= c {
                k += 1;
            }
            slots_by_chapter.push(k + 1);
        }
        let slots_at = |c: u8| slots_by_chapter[usize::from(c).min(slots_by_chapter.len() - 1)];

        let mut cell_visits = 0u128;
        for &c in &fired {
            cell_chapter_firings += 1;
            cell_visits += slots_at(c) as u128;
        }
        pd_visits += cell_visits;
        // A (cell, slot) carries a fact iff it existed by the cell's LAST firing
        // chapter — that is the CSR row count under per-depth.
        pd_rows += fired.last().map_or(0, |&c| slots_at(c));

        // --- f32: the bound the STORED values imply ---
        let mut fold = 0.0f64;
        let mut bound = 0.0f64;
        let mut summands = 0usize;
        for f in facts {
            let q = f.fraction_m();
            f32err.frac_min_m = f32err.frac_min_m.min(q);
            f32err.frac_max_m = f32err.frac_max_m.max(q);
            if f.to().1 == InvForm::Loose {
                fold += q;
                bound += q.abs() * F32_RELATIVE_RESOLUTION;
                summands += 1;
            }
        }
        let consumer = view.weathering_product_m(bedrock_slot);
        f32err.model_disagreement_m = f32err.model_disagreement_m.max((fold - consumer).abs());
        f32err.cells += 1;
        f32err.max_bound_m = f32err.max_bound_m.max(bound);
        err_sum += bound;
        f32err.max_fold_m = f32err.max_fold_m.max(fold);
        f32err.max_summands = f32err.max_summands.max(summands);
        if fold > 1e-9 {
            f32err.max_rel = f32err.max_rel.max(bound / fold);
        }

        pd_facts_per_cell[i] = u32::try_from(cell_visits).unwrap_or(u32::MAX);
    }
    if f32err.cells > 0 {
        f32err.mean_bound_m = err_sum / f32err.cells as f64;
    }
    if !f32err.frac_min_m.is_finite() {
        f32err.frac_min_m = 0.0;
    }

    let today_facts = field.ledgers.total_facts();
    let facts_per_firing = if cell_chapter_firings == 0 {
        0.0
    } else {
        today_facts as f64 / cell_chapter_firings as f64
    };
    // Per slot, the same agents commit the same facts per chapter, so the fact
    // count scales with the visited-slot count at the SAME measured facts-per-firing.
    let pd_facts = (pd_visits as f64 * facts_per_firing).round() as usize;
    for n in &mut pd_facts_per_cell {
        *n = (f64::from(*n) * facts_per_firing).round() as u32;
    }

    Geometry {
        extent,
        cells,
        chapters,
        epochs: cfg.iterations,
        weathering_cells,
        today_facts,
        today_rows: field.ledgers.slots_with_facts(),
        today_measured_bytes: field.ledgers.footprint_bytes(),
        cell_chapter_firings,
        facts_per_firing,
        pd_visits,
        pd_rows,
        pd_facts,
        pd_facts_per_cell,
        edge_dict: field.ledgers.edge_dictionary(),
        f32: f32err,
    }
}

impl Geometry {
    fn today(&self) -> Residency {
        Residency {
            cells: self.cells,
            facts: self.today_facts,
            fact_bytes: std::mem::size_of::<Fact>(),
            slot_rows: self.today_rows,
        }
    }
    fn per_depth(&self, fact_bytes: usize) -> Residency {
        self.per_depth_at(self.pd_facts, fact_bytes)
    }
    /// A per-depth residency at an arbitrary fact count — the axis-drop evaluator.
    fn per_depth_at(&self, facts: usize, fact_bytes: usize) -> Residency {
        Residency {
            cells: self.cells,
            facts,
            fact_bytes,
            slot_rows: self.pd_rows,
        }
    }
    /// The **fold-only resident projection** for the paged option: no facts at all,
    /// one f32 per non-empty `(cell, slot)` carried in the existing CSR rows, plus
    /// the per-cell file offset the pager needs.
    ///
    /// A row becomes `{ slot: u32, product: f32 }` — still 8 B, the width the index
    /// already costs, so **the fold rides free inside the index**. The extra term is
    /// `cell_fact_start: u64` per cell (the disk offset of that cell's run).
    fn fold_only_bytes(&self) -> usize {
        self.pd_rows * SLOT_ROW_BYTES
            + (self.cells + 1) * CELL_OFFSET_BYTES
            + (self.cells + 1) * std::mem::size_of::<u64>()
    }
}

// ===========================================================================
// 2. ENCODING — candidate fact layouts, measured with size_of
// ===========================================================================

/// **The PRE-SLICE shape** (endpoints + f64) — what `Fact` was before journal/0108,
/// restated locally so the reclaimed bytes are measured against something concrete
/// rather than remembered.
#[allow(dead_code)]
struct LegacyShape {
    chapter: u8,
    cause: u8,
    from: (u8, u8),
    to: (u8, u8),
    fraction_m: f64,
}

/// **Edge id only.** `from`/`to` replaced by a `u16` index into the declared
/// transition graph (S-8 / material-behavior §3). `fraction_m` stays f64.
///
/// **This is § 3.1's finding, and it is still true of a SHIPPED type**: the gen-time
/// accumulator `Fact<FracM>` is exactly this — the declared edge id, without the
/// narrowing — and it is 16 B, because the `f64`'s alignment pads the four reclaimed
/// bytes straight back. The two levers only pay together.
#[allow(dead_code)]
struct EdgeIdShape {
    chapter: u8,
    cause: u8,
    edge: u16,
    fraction_m: f64,
}

/// **f32 only.** Endpoints kept; `fraction_m` narrowed. 12 B — the other half of the
/// "only together" finding: narrowing alone leaves the six endpoint bytes aligned to
/// four and reclaims one word, not two.
#[allow(dead_code)]
struct F32Shape {
    chapter: u8,
    cause: u8,
    from: (u8, u8),
    to: (u8, u8),
    fraction_m: f32,
}

/// **Both — and this is what SHIPPED** (option 2c). Edge id + f32: 8 B, zero
/// padding. [`gate`] asserts `size_of::<Fact>()` equals it, so this local model
/// cannot quietly stop describing the real type (the `flow_cost_probe` failure mode,
/// twice).
#[allow(dead_code)]
struct BothShape {
    chapter: u8,
    cause: u8,
    edge: u16,
    fraction_m: f32,
}

/// **Both, struct-of-arrays** — `meta` packs `chapter | cause | edge` into a `u16`
/// (3 bits chapter, 2 bits cause, 11 bits edge = 2 048 declared edges) in its own
/// array beside an `f32` array. Padding disappears because there is no struct to
/// pad; the cost is that a fact is no longer one contiguous object.
const SOA_META_BYTES: usize = 2;
const SOA_FRAC_BYTES: usize = 4;

struct Encoding {
    label: &'static str,
    fact_bytes: usize,
}

fn encodings() -> Vec<Encoding> {
    vec![
        Encoding {
            label: "pre-0108 (endpoints + f64)",
            fact_bytes: std::mem::size_of::<LegacyShape>(),
        },
        Encoding {
            label: "edge id only (u16 + f64)",
            fact_bytes: std::mem::size_of::<EdgeIdShape>(),
        },
        Encoding {
            label: "f32 only (endpoints + f32)",
            fact_bytes: std::mem::size_of::<F32Shape>(),
        },
        Encoding {
            label: "edge id + f32 (AoS)  <- SHIPPED",
            fact_bytes: std::mem::size_of::<BothShape>(),
        },
        Encoding {
            label: "edge id + f32 (SoA, packed meta)",
            fact_bytes: SOA_META_BYTES + SOA_FRAC_BYTES,
        },
    ]
}

// ===========================================================================
// 3. PAGING — a real file at the projected per-depth geometry
// ===========================================================================

#[derive(Clone, Copy, Default)]
struct Latency {
    n: usize,
    mean_us: f64,
    p50_us: f64,
    p95_us: f64,
    max_us: f64,
    /// **Which read was the slowest**, in issue order. A tail of one on an
    /// otherwise tight distribution is a different fact depending on whether it is
    /// read #0 (handle / device warm-up, paid once per session) or read #217 (a
    /// stall any inspect can hit). Reporting the index is what makes the difference
    /// observable instead of assumed.
    max_at: usize,
    /// The very first read's latency, for the same reason.
    first_us: f64,
}

impl Latency {
    fn of(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        let n = samples.len();
        let max_at = samples
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        let pick = |q: f64| sorted[((n as f64 * q) as usize).min(n - 1)];
        Self {
            n,
            mean_us: sorted.iter().sum::<f64>() / n as f64,
            p50_us: pick(0.50),
            p95_us: pick(0.95),
            max_us: sorted[n - 1],
            max_at,
            first_us: samples[0],
        }
    }
}

struct Paging {
    path: PathBuf,
    file_bytes: u64,
    resident_bytes: usize,
    samples: usize,
    mean_run_bytes: f64,
    max_run_bytes: u64,
    cold: Option<Latency>,
    warm: Latency,
    write_s: f64,
}

/// Deterministic filler — the paging measurement is about **layout and size**, not
/// content, and no entropy may come from the wall clock (CLAUDE.md § Conventions).
/// A caller-owned constant seeds an LCG so the bytes are non-uniform (nothing
/// downstream can compress the file away) and reproducible.
fn filler_block(seed: u64, len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len + 8);
    let mut x = seed | 1;
    while out.len() < len {
        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        out.extend_from_slice(&x.to_le_bytes());
    }
    out.truncate(len);
    out
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Sample cells in a **scattered** deterministic order — a stride coprime with the
/// population, so consecutive reads land far apart in the file and neither the OS
/// nor the device sees a sequential pattern worth prefetching.
fn scattered_sample(pop: usize, want: usize) -> Vec<usize> {
    if pop == 0 {
        return Vec::new();
    }
    let want = want.min(pop);
    let mut stride = (pop / want.max(1)).max(1) | 1;
    while gcd(stride, pop) != 1 {
        stride += 2;
    }
    (0..want).map(|k| (k * stride) % pop).collect()
}

/// Read the window covering `[off, off + len)` through an unbuffered handle: the
/// window is aligned down / up to the sector size and read into a sector-aligned
/// sub-slice of `raw` (over-allocated by a sector so such a sub-slice exists).
fn aligned_read(f: &mut File, raw: &mut [u8], off: u64, len: u64) -> std::io::Result<usize> {
    let addr = raw.as_ptr() as usize;
    let base = addr.next_multiple_of(SECTOR as usize) - addr;
    let start = off - off % SECTOR;
    let span = (off - start + len)
        .next_multiple_of(SECTOR)
        .min((raw.len() - base) as u64);
    f.seek(SeekFrom::Start(start))?;
    // A read at EOF can come back short even unbuffered, and `read_exact` would
    // then error, so take what the device gives and report the count.
    f.read(&mut raw[base..base + span as usize])
}

/// The directories the paging arm is measured on. This machine has **two SSDs of
/// different classes** (an NVMe system drive and a SATA data drive), and page-in
/// latency is a property of the device, not of the design — so the answer is a
/// bracket, not a single number. `CARGO_TARGET_DIR` (or the workspace `target/`)
/// is the second location because it is on the repo's drive, which is where a
/// world save would plausibly live.
fn paging_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![std::env::temp_dir()];
    let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("target")
        },
        PathBuf::from,
    );
    if target.is_dir() && !dirs.contains(&target) {
        dirs.push(target);
    }
    dirs
}

fn measure_paging(geo: &Geometry, fact_bytes: usize, dir: &Path) -> std::io::Result<Paging> {
    let path = dir.join(format!(
        "dc-s20-facts-{SEED}-{}.bin",
        geo.extent.label().replace(' ', "-")
    ));

    // --- the per-cell run offsets (the pager's resident index) ---
    let mut offsets: Vec<u64> = Vec::with_capacity(geo.cells + 1);
    let mut acc = 0u64;
    for &n in &geo.pd_facts_per_cell {
        offsets.push(acc);
        acc += u64::from(n) * fact_bytes as u64;
    }
    offsets.push(acc);
    let file_bytes = acc;

    // --- write the file at the projected geometry ---
    let t = Instant::now();
    {
        let f = File::create(&path)?;
        let mut w = BufWriter::with_capacity(1 << 20, f);
        let block = filler_block(0x5320_C0DE, 1 << 20);
        let mut left = file_bytes;
        while left > 0 {
            let take = left.min(block.len() as u64) as usize;
            w.write_all(&block[..take])?;
            left -= take as u64;
        }
        w.flush()?;
        w.into_inner()
            .map_err(std::io::IntoInnerError::into_error)?
            .sync_all()?;
    }
    let write_s = t.elapsed().as_secs_f64();

    // --- the sample: cells that actually carry facts, scattered ---
    let with_facts: Vec<usize> = (0..geo.cells)
        .filter(|&i| geo.pd_facts_per_cell[i] > 0)
        .collect();
    let sample: Vec<usize> = scattered_sample(with_facts.len(), PAGE_SAMPLES)
        .into_iter()
        .map(|k| with_facts[k])
        .collect();

    let run_len = |i: usize| offsets[i + 1] - offsets[i];
    let max_run = sample.iter().map(|&i| run_len(i)).max().unwrap_or(0);
    let mean_run = if sample.is_empty() {
        0.0
    } else {
        sample.iter().map(|&i| run_len(i) as f64).sum::<f64>() / sample.len() as f64
    };

    // Scratch: the largest run in the field, rounded up, plus two sectors of slack
    // so an aligned sub-slice of the needed span always exists.
    let cap = (0..geo.cells)
        .map(run_len)
        .max()
        .unwrap_or(0)
        .next_multiple_of(SECTOR)
        + 2 * SECTOR;
    let mut raw = vec![0u8; cap as usize];

    // --- COLD: unbuffered, so the read goes to the device, not the page cache ---
    let cold = open_unbuffered(&path).ok().map(|mut f| {
        let mut us = Vec::with_capacity(sample.len());
        let mut sink = 0usize;
        for &i in &sample {
            let (off, len) = (offsets[i], run_len(i));
            let t = Instant::now();
            let n = aligned_read(&mut f, &mut raw, off, len).unwrap_or(0);
            us.push(t.elapsed().as_secs_f64() * 1e6);
            sink += n;
        }
        std::hint::black_box(sink);
        Latency::of(&us)
    });

    // --- WARM: buffered, cache hot (every sampled run pre-read once) ---
    let warm = {
        let mut f = File::open(&path)?;
        let mut buf = vec![0u8; cap as usize];
        for &i in &sample {
            f.seek(SeekFrom::Start(offsets[i]))?;
            let len = run_len(i) as usize;
            let _ = f.read(&mut buf[..len])?;
        }
        let mut us = Vec::with_capacity(sample.len());
        let mut sink = 0usize;
        for &i in &sample {
            let len = run_len(i) as usize;
            let t = Instant::now();
            f.seek(SeekFrom::Start(offsets[i]))?;
            let n = f.read(&mut buf[..len])?;
            us.push(t.elapsed().as_secs_f64() * 1e6);
            sink += n;
        }
        std::hint::black_box(sink);
        Latency::of(&us)
    };

    Ok(Paging {
        path,
        file_bytes,
        resident_bytes: geo.fold_only_bytes(),
        samples: sample.len(),
        mean_run_bytes: mean_run,
        max_run_bytes: max_run,
        cold,
        warm,
        write_s,
    })
}

#[cfg(windows)]
fn open_unbuffered(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING)
        .open(path)
}

/// Non-Windows: there is no portable way to defeat the page cache from user space,
/// so the cold arm reports nothing rather than a warm number wearing a cold label.
#[cfg(not(windows))]
fn open_unbuffered(_path: &Path) -> std::io::Result<File> {
    let _ = OpenOptions::new();
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unbuffered reads are measured on Windows only",
    ))
}

// ===========================================================================
// report
// ===========================================================================

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
fn mib64(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn report(geo: &Geometry, paging: &[Paging]) {
    println!(
        "\n=== S20: fact-ledger residency (seed {SEED}, {}) ===\n",
        geo.extent.label()
    );

    println!("--- 1. geometry (re-derived, not inherited) ---");
    println!(
        "  deep cells {}, epochs {}, chapters {}",
        geo.cells, geo.epochs, geo.chapters
    );
    println!(
        "  cells that ever weather                  {:>12}  ({:.1}%)",
        geo.weathering_cells,
        100.0 * geo.weathering_cells as f64 / geo.cells as f64
    );
    println!(
        "  TODAY  facts {:>12}  rows {:>10}   = {:>8.2} MiB  (measured {:.2} MiB, \
         itemisation {})",
        geo.today_facts,
        geo.today_rows,
        mib(geo.today().bytes()),
        mib(geo.today_measured_bytes),
        if geo.today().check(geo.today_measured_bytes) {
            "EXACT"
        } else {
            "MISMATCH"
        }
    );
    // Before/after on the SAME measured world — one run's arithmetic, not two runs'
    // memories, and stated as ABSOLUTES because a ratio rots when its denominator
    // moves.
    let today_legacy = Residency {
        fact_bytes: std::mem::size_of::<LegacyShape>(),
        ..geo.today()
    };
    println!(
        "  TODAY at the PRE-0108 width ({} B/fact)          = {:>8.2} MiB   -> shipped 8 B/fact \
         = {:>8.2} MiB   (reclaimed {:.2} MiB)",
        std::mem::size_of::<LegacyShape>(),
        mib(today_legacy.bytes()),
        mib(geo.today().bytes()),
        mib(today_legacy.bytes()) - mib(geo.today().bytes())
    );
    println!(
        "  (cell, chapter) firings {:>12}   facts per firing {:.4}  (the agent count, measured)",
        geo.cell_chapter_firings, geo.facts_per_firing
    );
    println!(
        "  PER-DEPTH (cell, slot, chapter) visits, triangular  {:>12}",
        geo.pd_visits
    );
    println!(
        "  PER-DEPTH facts {:>12}   rows {:>12}",
        geo.pd_facts, geo.pd_rows
    );

    println!("\n--- 2. encoding: fact widths (option 2c SHIPPED, journal/0108) ---");
    let base = geo.per_depth(std::mem::size_of::<LegacyShape>()).bytes();
    for e in encodings() {
        let r = geo.per_depth(e.fact_bytes);
        println!(
            "  {:<34} {:>2} B/fact   per-depth resident {:>8.2} MiB   ({:.0}% of pre-0108)",
            e.label,
            e.fact_bytes,
            mib(r.bytes()),
            100.0 * r.bytes() as f64 / base as f64
        );
    }
    println!(
        "  shipped `Fact` is {} B (local BothShape {} B — {}); the gen-time accumulator \
         `Fact<FracM>` is {} B",
        std::mem::size_of::<Fact>(),
        std::mem::size_of::<BothShape>(),
        if std::mem::size_of::<Fact>() == std::mem::size_of::<BothShape>() {
            "AGREE"
        } else {
            "DISAGREE — the projection's width model is wrong"
        },
        std::mem::size_of::<Fact<FracM>>()
    );
    println!(
        "  § 3.1 (an edge id ALONE saves nothing): EdgeIdShape {} B == pre-0108 {} B == the \
         SHIPPED gen-time accumulator {} B",
        std::mem::size_of::<EdgeIdShape>(),
        std::mem::size_of::<LegacyShape>(),
        std::mem::size_of::<Fact<FracM>>()
    );
    println!(
        "  per-world EDGE DICTIONARY (LedgerField::edge_dictionary) — {} inhabited:",
        geo.edge_dict.len()
    );
    for e in geo.edge_dict.entries() {
        println!(
            "     id {:#06x}   {} / {}  ->  {} / {}",
            e.id.raw(),
            e.from_material,
            e.from_form.name(),
            e.to_material,
            e.to_form.name()
        );
    }
    println!(
        "     re-derives from its own material NAMES against the live registry: {}",
        if geo.edge_dict.validate().is_ok() {
            "OK (a registry change would be DETECTED, never silently reinterpreted)"
        } else {
            "MISMATCH"
        }
    );

    println!("\n--- 3. paging: resident fold + facts on disk ---");
    if paging.is_empty() {
        println!("  (skipped)");
    }
    for p in paging {
        println!(
            "  file {:>10.2} MiB written in {:.1} s   at {}",
            mib64(p.file_bytes),
            p.write_s,
            p.path.display()
        );
        println!(
            "  RESIDENT fold-only projection            {:>8.2} MiB  \
             (rows {} x 8 B + cell offsets + disk offsets)",
            mib(p.resident_bytes),
            geo.pd_rows
        );
        println!(
            "  one cell's run: mean {:.0} B, max {} B   ({} cells sampled, scattered)",
            p.mean_run_bytes, p.max_run_bytes, p.samples
        );
        match p.cold {
            Some(c) => println!(
                "  COLD page-in (unbuffered, device)   mean {:>8.1} us  p50 {:>8.1}  \
                 p95 {:>8.1}  max {:>8.1} (at read #{})  first {:.1}   (n={})",
                c.mean_us, c.p50_us, c.p95_us, c.max_us, c.max_at, c.first_us, c.n
            ),
            None => println!("  COLD page-in                        (not measurable here)"),
        }
        println!(
            "  WARM page-in (buffered, cache hot)  mean {:>8.1} us  p50 {:>8.1}  \
             p95 {:>8.1}  max {:>8.1} (at read #{})  first {:.1}   (n={})",
            p.warm.mean_us,
            p.warm.p50_us,
            p.warm.p95_us,
            p.warm.max_us,
            p.warm.max_at,
            p.warm.first_us,
            p.warm.n
        );
        if let Some(c) = p.cold {
            let budget_us = 1e6 / 60.0;
            println!(
                "  => a 60 Hz frame is {budget_us:.0} us; a cold page-in p95 is {:.2}% of one \
                 frame, {:.2}% of 10 ms",
                100.0 * c.p95_us / budget_us,
                100.0 * c.p95_us / 10_000.0
            );
        }
        println!();
    }

    println!("\n--- 4. axis-drops (no disk) ---");
    let w = std::mem::size_of::<Fact>();
    let agents = geo.facts_per_firing.round().max(1.0) as usize;
    for (label, r) in [
        ("all axes (chapter x agent)", geo.per_depth(w)),
        (
            "drop CHAPTER (per agent)",
            geo.per_depth_at(geo.pd_rows * agents, w),
        ),
        (
            "drop AGENT (per chapter)",
            geo.per_depth_at(geo.pd_visits as usize, w),
        ),
        (
            "drop BOTH (one fact per slot)",
            geo.per_depth_at(geo.pd_rows, w),
        ),
    ] {
        println!(
            "  {:<32} facts {:>12}   {:>8.2} MiB   ({:.2}x today)",
            label,
            r.facts,
            mib(r.bytes()),
            r.bytes() as f64 / geo.today().bytes() as f64
        );
    }
    println!(
        "  mean chapters fired per weathering slot: {:.2}  (= visits / rows — the chapter axis's \
         own multiplicity)",
        geo.pd_visits as f64 / geo.pd_rows.max(1) as f64
    );

    println!("\n--- 5a. f32 storage: the bound the STORED record implies ---");
    let f = &geo.f32;
    println!(
        "  folds bounded {:>10}   deepest fold {} summands",
        f.cells, f.max_summands
    );
    println!(
        "  stored fraction_m magnitudes: min {:.6e} m   max {:.6e} m",
        f.frac_min_m, f.frac_max_m
    );
    println!(
        "  narrowing bound   max {:.6e} m   mean {:.6e} m   max relative {:.3e}",
        f.max_bound_m, f.mean_bound_m, f.max_rel
    );
    println!(
        "  largest fold {:.6} m; ONE EIGHTH OF A VOXEL = {EIGHTH_M} m",
        f.max_fold_m
    );
    println!(
        "  => the worst BOUND is {:.3e} of an eighth  (1 part in {:.3e})",
        f.max_bound_m / EIGHTH_M,
        EIGHTH_M / f.max_bound_m.max(f64::MIN_POSITIVE)
    );
    println!(
        "  cross-check: |simple sum - weathering_product_m| max {:.3e} m  (the simple sum IS the \
         consumer's fold)",
        f.model_disagreement_m
    );

    println!("\n--- 5b. f32 storage: the MEASURED persist-step error ---");
    narrowing_audit();
    tolerance_audit();
    println!();
}

/// **The measured accumulator to resident error** — a direct reproduction of the one
/// place in the tree that narrows (`LedgerField::from_accumulators`), at the
/// production fold shape.
///
/// § 5a can only bound the error, because the shipped record no longer holds the f64
/// original. This runs the real persist step on real accumulators: fire the shipped
/// `weather_bedrock_epoch` across the production chapter count with the unit tests'
/// own inputs, fold the f64 accumulator, `finalize_ledgers`, fold the narrowed
/// resident view, and difference them. Pure arithmetic — no world, no I/O.
fn narrowing_audit() {
    let inp = WeatherInputs {
        weathering: 0.02,
        h_star: 2.0,
        regolith_h: 1.0,
        biotic: 1.5,
        frost: 2.0,
    };
    println!(
        "  agents {}, inputs = the weather_inventory unit tests' own",
        WEATHERING_AGENTS.len()
    );
    for &(chapters, firings) in &[(1u8, 1u32), (1, 5), (8, 25), (8, 200)] {
        let mut acc = empty_accumulator();
        for e in 0..firings {
            let chapter = (e % u32::from(chapters)) as u8;
            weather_bedrock_epoch(&mut acc, chapter, &inp, 1.0);
        }
        let in_accumulator: FracM = acc
            .facts_for(0)
            .iter()
            .filter(|f| f.to().1 == InvForm::Loose)
            .map(Fact::fraction_m)
            .sum();
        let facts = acc.facts_for(0).len();
        let empty = DeepStrata::default();
        let resident = finalize_ledgers(vec![acc], std::slice::from_ref(&empty));
        let after = resident
            .get(0)
            .expect("cell 0")
            .weathering_product_m(empty.units.len());
        let err = (after - in_accumulator).abs();
        let tol = stored_fold_tolerance(after.max(in_accumulator));
        println!(
            "  {chapters} chapter(s) x {firings:>3} firings -> {facts:>2} facts, band {in_accumulator:.9} m; \
             narrowing err {err:.3e} m   (1e-12 {}, derived bound {tol:.3e} {})",
            if err < 1e-12 { "passes" } else { "FAILS" },
            if err <= tol { "passes" } else { "FAILS" }
        );
    }
    println!(
        "  => against ONE EIGHTH OF A VOXEL ({EIGHTH_M} m), the derived bound is at most \
         {:.3e} of an eighth at the largest production fold (6.09 m)",
        stored_fold_tolerance(6.09) / EIGHTH_M
    );
}

/// **The tolerance class map** — pure arithmetic, no world.
///
/// Every assertion in the tree that reads a stored `fraction_m` falls into one of two
/// classes, and **which class it lands in is decided by WHERE the narrowing happens**
/// (S20 § 4.3's final paragraph). Since journal/0108 narrows only at
/// `LedgerField::from_accumulators`:
///
/// - **stored vs stored — unaffected.** Both sides ride the same values, so narrowing
///   does not move the comparison. This covers every assertion that reads a
///   `FactLedger` (the gen-time accumulator is still `f64`, so those are *f64 vs
///   f64* and did not even change representation), and every assertion that compares
///   two quantities both read out of the resident `LedgerField`.
/// - **stored vs freshly-computed f64 — one site in the tree**, and it is exactly the
///   one § 4.3 predicted: `weather_inventory`'s
///   `bedrock_facts_key_stably_as_the_record_grows`, which straddles the persist
///   boundary. Its bound is re-derived from the storage's resolution
///   (`inventory::stored_fold_tolerance`), not widened until green.
///
/// This function prints the arithmetic behind that classification so the claim is a
/// number rather than an argument. The literals the unit tests assert against
/// (`0.5`, `0.6`, `1.25`, …) are listed with their f32 round-trip error **and with
/// the ledger they are actually read from**, because a value that never reaches the
/// resident record is never narrowed and its `1e-12` bound is still honest.
fn tolerance_audit() {
    let inp = WeatherInputs {
        weathering: 0.02,
        h_star: 2.0,
        regolith_h: 1.0,
        biotic: 1.5,
        frost: 2.0,
    };
    let m = BEDROCK_SEAM_MATERIAL;
    let shares: Vec<f64> = WEATHERING_AGENTS
        .iter()
        .map(|&a| agent_share(&inp, m, a))
        .collect();

    // **Why the accumulator must stay f64 — the counterfactual, measured.** If the
    // narrowing happened at the accumulator instead of at persist, every one of a
    // run's firings would round its own running total, and the error would COMPOUND
    // with the firing count instead of happening once. Same shares, same firings,
    // same final band; the only difference is where the `as f32` sits.
    println!("  where the narrowing sits, measured on the same shares:");
    for firings in [1u32, 10, 200] {
        let exact: f64 = shares.iter().map(|&s| s * f64::from(firings)).sum();
        // Narrow ONCE at persist (what ships): accumulate in f64, round at the end.
        let once: f64 = shares
            .iter()
            .map(|&s| f64::from((s * f64::from(firings)) as f32))
            .sum();
        // Narrow at EVERY add (what a narrowed accumulator would do).
        let every: f64 = shares
            .iter()
            .map(|&s| {
                let mut q = 0.0f32;
                for _ in 0..firings {
                    q = (f64::from(q) + s) as f32;
                }
                f64::from(q)
            })
            .sum();
        println!(
            "     {firings:>3} firings, band {exact:.9} m: narrow-ONCE err {:.3e} m   \
             narrow-at-EVERY-add err {:.3e} m   ({:.1}x worse)",
            (exact - once).abs(),
            (exact - every).abs(),
            (exact - every).abs() / (exact - once).abs().max(f64::MIN_POSITIVE)
        );
    }
    println!(
        "  f32 has 24 bits of mantissa: relative resolution 2^-24 = {:.3e} (= {:.3e} in the \
         library's F32_RELATIVE_RESOLUTION)",
        f64::from(f32::EPSILON) / 2.0,
        F32_RELATIVE_RESOLUTION
    );
    println!("  round-trip error of the literals the unit tests assert against, and the ledger");
    println!("  they are read from (an accumulator read is never narrowed):");
    for (v, from_accumulator) in [
        (0.5f64, true),
        (1.5, true),
        (0.6, true),
        (0.3, true),
        (0.2, true),
        (0.4, true),
        (1.25, true),
    ] {
        let e = (v - f64::from(v as f32)).abs();
        println!(
            "     {v:<6} -> f32 round-trip err {e:.3e}   read from {}   {}",
            if from_accumulator {
                "FactLedger (f64)"
            } else {
                "LedgerField (f32)"
            },
            if from_accumulator {
                "1e-12 still honest"
            } else {
                "needs stored_fold_tolerance"
            }
        );
    }
}

fn main() {
    println!("measuring...");
    let geo = measure_geometry(EXTENT);
    let paging: Vec<Paging> = paging_dirs()
        .iter()
        .filter_map(|d| measure_paging(&geo, std::mem::size_of::<Fact>(), d).ok())
        .collect();
    report(&geo, &paging);
    for p in &paging {
        let _ = std::fs::remove_file(&p.path);
        println!("(removed {})", p.path.display());
    }
}

/// **GATE** (journal/0103: an example that can fail belongs in the gate).
///
/// Runs at `Extent::Small`. Every invariant asserted here is **scale-free**: a
/// relation between counts derived from one record, or a property of a type's size
/// — never a MiB figure and never a microsecond count. The production magnitudes
/// are what `main` is for.
#[cfg(test)]
mod gate {
    use super::*;

    #[test]
    fn the_s20_residency_model_is_self_consistent() {
        let geo = measure_geometry(Extent::Small);

        // The itemisation must reconstruct the record's own measured footprint
        // byte-for-byte, or every projection below scales a wrong model. This is
        // also the assertion that fires if `Fact`, `SlotRun` or the CSR layout moves
        // under this probe (the flow_cost_probe failure mode, twice).
        assert!(
            geo.today().check(geo.today_measured_bytes),
            "ledger residency itemisation {} B != measured {} B",
            geo.today().bytes(),
            geo.today_measured_bytes
        );

        // The local width model must agree with the shipped type, or the encoding
        // table is fiction. Since journal/0108 the shipped `Fact` IS `BothShape`.
        assert_eq!(
            std::mem::size_of::<Fact>(),
            std::mem::size_of::<BothShape>(),
            "the local BothShape no longer models the shipped `Fact`"
        );
        // And the shipped fact carries NO padding: it is exactly its payload.
        assert_eq!(
            std::mem::size_of::<Fact>(),
            std::mem::size_of::<u8>() * 2 + std::mem::size_of::<u16>() + std::mem::size_of::<f32>(),
            "the shipped fact must be exactly its payload"
        );

        // The pass must have fired somewhere, or nothing below means anything.
        assert!(geo.cell_chapter_firings > 0, "no (cell, chapter) fired");
        assert!(geo.pd_visits >= geo.cell_chapter_firings);
        assert!(geo.pd_rows > 0 && geo.pd_facts > 0);

        // Narrowing must be strictly cheaper in bytes — half the encoding claim.
        let legacy = geo.per_depth(std::mem::size_of::<LegacyShape>()).bytes();
        let both = geo.per_depth(std::mem::size_of::<BothShape>()).bytes();
        assert!(both < legacy, "edge id + f32 must be strictly smaller");
        // **§ 3.1's finding, and it must stay live.** An edge id ALONE buys nothing
        // in an array of structs, because f64 alignment pads the struct straight
        // back. Asserted twice: against the local model, and against the SHIPPED
        // gen-time accumulator, which is that model in real code — it carries the
        // declared edge id and not the narrowing, and it is still the pre-slice
        // width. The two levers only pay together.
        assert_eq!(
            std::mem::size_of::<EdgeIdShape>(),
            std::mem::size_of::<LegacyShape>(),
            "edge id alone was expected to be padded back to the pre-0108 width"
        );
        assert_eq!(
            std::mem::size_of::<Fact<FracM>>(),
            std::mem::size_of::<EdgeIdShape>(),
            "the shipped accumulator IS the edge-id-only shape, and must still be padded back"
        );
        assert_eq!(
            std::mem::size_of::<Fact<FracM>>(),
            2 * std::mem::size_of::<Fact>(),
            "the narrowing is the whole difference between accumulator and resident width"
        );
        // f32 alone does not reach 8 B either — the other half of "only together".
        assert!(
            std::mem::size_of::<F32Shape>() > std::mem::size_of::<BothShape>(),
            "narrowing alone was expected to leave the endpoint bytes in place"
        );

        // The per-world edge dictionary must re-derive from its own material NAMES,
        // or a persisted world would silently reinterpret a registry change.
        assert!(!geo.edge_dict.is_empty(), "the record inhabits some edge");
        assert!(
            geo.edge_dict.validate().is_ok(),
            "the edge dictionary does not re-derive from its own names"
        );
        // It is a DICTIONARY, not a per-fact cost: strictly fewer entries than facts.
        assert!(
            geo.edge_dict.len() < geo.today_facts,
            "an edge dictionary with one entry per fact is not a dictionary"
        );

        // Every axis-drop is a strict reduction, and dropping both is the floor.
        let agents = geo.facts_per_firing.round().max(1.0) as usize;
        assert!(agents > 1, "the agent axis must have more than one value");
        assert!(geo.pd_rows * agents < geo.pd_facts, "drop-chapter reduces");
        assert!(
            (geo.pd_visits as usize) < geo.pd_facts,
            "drop-agent reduces"
        );

        // The simple sum this probe folds IS the fold the collapse consumer reads —
        // if it drifts, the f32 error figure is about the wrong quantity.
        assert!(
            geo.f32.model_disagreement_m < 1e-9,
            "the probe's fold disagrees with weathering_product_m by {:.3e} m",
            geo.f32.model_disagreement_m
        );

        // f32 storage must cost far less than an eighth of a voxel, or the shipped
        // encoding is unsound. Asserted on the BOUND the stored record implies, so
        // this stays true of any world rather than of a sampled difference.
        assert!(geo.f32.max_bound_m > 0.0, "no stored fraction at all");
        assert!(
            geo.f32.max_bound_m < EIGHTH_M / 1000.0,
            "the f32 narrowing bound {:.3e} m is within 3 orders of an eighth ({EIGHTH_M} m)",
            geo.f32.max_bound_m
        );

        // **The narrowing is REAL and it is bounded** — the persist step reproduced
        // directly, at the production fold shape. It must (a) actually cost
        // something, or the encoding claim is vacuous, and (b) stay inside the
        // derived tolerance, or the tolerance is wrong. This is what makes
        // `weather_inventory`'s re-derived bound evidence rather than a widening.
        let mut acc = empty_accumulator();
        for e in 0..200u32 {
            weather_bedrock_epoch(
                &mut acc,
                (e % 8) as u8,
                &WeatherInputs {
                    weathering: 0.02,
                    h_star: 2.0,
                    regolith_h: 1.0,
                    biotic: 1.5,
                    frost: 2.0,
                },
                1.0,
            );
        }
        let in_accumulator: FracM = acc
            .facts_for(0)
            .iter()
            .filter(|f| f.to().1 == InvForm::Loose)
            .map(Fact::fraction_m)
            .sum();
        let empty = DeepStrata::default();
        let resident = finalize_ledgers(vec![acc], std::slice::from_ref(&empty));
        let after = resident
            .get(0)
            .expect("cell 0")
            .weathering_product_m(empty.units.len());
        let err = (after - in_accumulator).abs();
        assert!(
            err > 0.0,
            "the persist step did not narrow anything — the encoding claim is vacuous"
        );
        assert!(
            err <= stored_fold_tolerance(after.max(in_accumulator)),
            "narrowing error {err:.3e} m exceeds the derived tolerance"
        );
        assert!(
            err < EIGHTH_M / 1000.0,
            "narrowing error {err:.3e} m is within 3 orders of an eighth"
        );

        // The paged prototype must lay out and read back a real file.
        let dir = paging_dirs().remove(0);
        let paging = measure_paging(&geo, std::mem::size_of::<Fact>(), &dir)
            .expect("the paged prototype writes and reads a file");
        assert_eq!(
            paging.file_bytes,
            geo.pd_facts_per_cell
                .iter()
                .map(|&n| u64::from(n) * std::mem::size_of::<Fact>() as u64)
                .sum::<u64>(),
            "the file is not the projected geometry"
        );
        assert!(paging.warm.n > 0 && paging.warm.mean_us > 0.0);
        assert!(
            paging.resident_bytes < geo.per_depth(std::mem::size_of::<Fact>()).bytes(),
            "the fold-only projection must be smaller than the fully-resident one"
        );
        let _ = std::fs::remove_file(&paging.path);
    }
}
