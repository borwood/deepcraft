//! **R3 — what would PER-DEPTH weathering cost?** (ROADMAP Sequenced,
//! *"WEATHERING IS ONE PROCESS — SAPROLITE IS A STATE ALONG IT"*, requisite R3.)
//!
//! R1 (the `head` field) and R2 (real vertical flux) are met. R3 is the last
//! requisite and it is **pure measurement**: *"per-depth × per-cell × per-epoch
//! over 297 k cells with multi-slot columns is far larger than today's per-cell
//! scalar, and the deep run is already 35–85 s."* This probe puts numbers on
//! "far larger".
//!
//! **THIS PROBE BUILDS NO PART OF THE CHANGE.** It measures the *work a per-depth
//! pass would do* using only the substrate's existing public API, on the world the
//! player boots. Nothing here is a weathering implementation and nothing here is
//! wired into a run — the "per-depth firing" below is a **cost model**, deliberately
//! kept in `examples/` where it cannot be mistaken for a pass.
//!
//! # What is measured
//!
//! Today `dc:deep/weather_inventory` fires **once per subaerial cell per epoch**
//! and weathers exactly one thing: the materialized bedrock seam (a single span,
//! built over an *empty* record — journal/0094's span-index crux). Its per-cell
//! cost is therefore **O(1) in the record's depth**. The arc makes weathering a
//! rate evaluated at **each stratum slot**, so the same firing becomes O(slots).
//!
//! 1. **The tuple ratio.** `(cell, slot, epoch)` visits against today's
//!    `(cell, epoch)` visits, over the firings that actually happened (read out of
//!    the ledger, not assumed). This is the multiplier.
//! 2. **The causal triangle.** A slot deposited in chapter `c` **cannot** weather
//!    before `c`, so the per-depth pass at epoch `e` sees only the slots already
//!    laid down. The record's units carry their `chapter`, so the triangular count
//!    is exact — and it is reported against the naive full-depth count so the
//!    saving is a measured number rather than an argument.
//! 3. **Per-invocation wall clock.** Today's firing (`weather_bedrock_epoch`) and
//!    the per-depth cost model, timed over the same sample of real production
//!    columns, at the same firing count.
//! 4. **Gen-time.** The deep run is timed with the pass OFF and ON, so the pass's
//!    own wall clock is a *measured difference*, not a share guessed from a
//!    profile; the projection scales that by (3).
//! 5. **Residency.** A per-slot rate needs per-slot state. The `LedgerField`
//!    footprint is itemised, the itemisation is checked against the measured
//!    footprint byte-for-byte, and the same itemisation is then evaluated at the
//!    per-depth slot count.
//!
//! `cargo run --release -p dc-worldgen --example perdepth_weathering_cost_probe`

use std::hint::black_box;
use std::time::Instant;

use dc_worldgen::deeptime::inventory::{Fact, FactLedger, InvForm, build_working, commit_chapter};
use dc_worldgen::deeptime::weather_inventory::{empty_accumulator, weather_bedrock_epoch};
use dc_worldgen::deeptime::{
    DeepConfig, DeepStrata, WEATHERING_AGENTS, WeatherInputs, agent_share, build_field_cfg,
    production_config,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The world `dc-client` boots — `BENCH_SEED` (`dc-client/src/bench.rs`) at
/// `WORLDGEN_EXTENT` (`dc-client/src/authority.rs`). R3 is a production-scale
/// question, so the *report* runs here.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;

/// Cells drawn (by stride, over cells that carry a record) for the timing
/// micro-benchmark. Large enough that the column-depth distribution is
/// represented; small enough that 200 firings × 3 variants stays seconds.
const TIMING_SAMPLE: usize = 1200;

/// `SlotRun` is `{ slot: u32, start: u32 }` — private to `inventory.rs`, so its
/// size is restated here and then **proved** by [`Residency::check`], which
/// reconstructs the measured footprint byte-for-byte from this itemisation.
const SLOT_ROW_BYTES: usize = 8;
/// `LedgerField::cell_row_start` is one `u32` per cell plus a terminator.
const CELL_OFFSET_BYTES: usize = 4;

// ---------------------------------------------------------------------------
// the per-depth COST MODEL (not a pass, not an implementation)
// ---------------------------------------------------------------------------

/// **The work one per-depth firing would do** on a cell, for costing only.
///
/// Shape, taken from the arc's own stated heir (*"`structure → pore_fill` rather
/// than `structure → structure`: retained parent structure with weathering product
/// in its pores"*): re-derive the column from `base + facts` (S-2, exactly as
/// today), then for **each span** evaluate the three-agent sum and move that share
/// of the span's parent portion onto `PoreFill`. One fact per (slot, agent) per
/// chapter — the per-slot state the residency question is about.
///
/// What it deliberately does **not** model: the reactant-transport field the arc
/// says must supply the depth term (that is a *field* pass, §5's split — and it is
/// per-cell-per-epoch, i.e. today's cost class, not this one). So this is the
/// **cellular** half's cost, which is the half that multiplies by depth.
fn per_depth_firing_cost_model(
    strata: &DeepStrata,
    ledger: &mut FactLedger,
    chapter: u8,
    inputs: &WeatherInputs,
    dt: f64,
) -> f64 {
    let mut inv = build_working(strata, ledger);
    let mut total = 0.0;
    for span in 0..inv.spans.len() {
        // The parent this slot's rate acts on: the span's first solid portion.
        let Some(parent) = inv.spans[span]
            .portions
            .iter()
            .copied()
            .find(|p| matches!(p.form, InvForm::Structure | InvForm::Loose))
        else {
            continue;
        };
        for &cause in &WEATHERING_AGENTS {
            let share = agent_share(inputs, parent.material, cause) * dt;
            if share <= 0.0 {
                continue;
            }
            total += inv.ctx_for(chapter, cause).move_form(
                span,
                parent.material,
                parent.form,
                InvForm::PoreFill,
                share,
            );
        }
    }
    commit_chapter(&mut inv, ledger);
    total
}

// ---------------------------------------------------------------------------
// the projection
// ---------------------------------------------------------------------------

/// Per-cell history read out of the finished world: which chapters the pass
/// actually fired in, and how many slots existed by each of them.
struct CellHistory {
    /// Chapters (ascending, deduplicated) this cell committed weathering facts in.
    chapters: Vec<u8>,
    /// `slots_by_chapter[c]` = spans a per-depth firing in chapter `c` would visit
    /// (units already deposited by `c`, plus the bedrock seam span). One entry per
    /// tectonic chapter — eight of them, not a 256-wide table per cell.
    slots_by_chapter: Vec<u32>,
    /// Spans at the end of the run (`units + bedrock seam`).
    slots_final: usize,
}

impl CellHistory {
    fn slots_at(&self, chapter: u8) -> u128 {
        let c = usize::from(chapter).min(self.slots_by_chapter.len().saturating_sub(1));
        u128::from(self.slots_by_chapter[c])
    }
}

struct Counts {
    cells: usize,
    epochs: u32,
    chapters: u32,
    epochs_per_chapter: u32,
    /// `cells × epochs` — the loop the pass is driven over, unchanged by the arc.
    loop_visits: u128,
    /// `(cell, chapter)` pairs the pass actually fired in.
    cell_chapter_firings: u128,
    /// `(cell, slot, chapter)` triples a per-depth pass would visit, **respecting
    /// the causal triangle** (slots deposited by that chapter only).
    slot_chapter_triangular: u128,
    /// The same, ignoring the triangle (final record depth at every chapter) — the
    /// number you get if you forget that a stratum cannot weather before it exists.
    slot_chapter_flat: u128,
}

impl Counts {
    /// Epochs-per-chapter cancels in the ratio, so the multiplier is exact at
    /// chapter resolution even though the firing set is only known per chapter.
    fn multiplier_triangular(&self) -> f64 {
        self.slot_chapter_triangular as f64 / self.cell_chapter_firings as f64
    }
    fn multiplier_flat(&self) -> f64 {
        self.slot_chapter_flat as f64 / self.cell_chapter_firings as f64
    }
    /// Today's `(cell, epoch)` firings, as bounds. A cell is known to have been
    /// subaerial for *at least one* epoch of each chapter it left facts in; the
    /// upper bound assumes all of them.
    fn firings_lo(&self) -> u128 {
        self.cell_chapter_firings
    }
    fn firings_hi(&self) -> u128 {
        self.cell_chapter_firings * u128::from(self.epochs_per_chapter)
    }
}

struct Timing {
    firings: usize,
    today_ns: f64,
    per_depth_triangular_ns: f64,
    per_depth_flat_ns: f64,
}

impl Timing {
    fn ratio_triangular(&self) -> f64 {
        self.per_depth_triangular_ns / self.today_ns
    }
    fn ratio_flat(&self) -> f64 {
        self.per_depth_flat_ns / self.today_ns
    }
}

/// The `LedgerField` footprint, itemised so the same itemisation can be evaluated
/// at the per-depth slot count.
#[derive(Clone, Copy)]
struct Residency {
    cells: usize,
    facts: usize,
    slot_rows: usize,
}

impl Residency {
    fn bytes(self) -> usize {
        self.facts * std::mem::size_of::<Fact>()
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

struct Projection {
    extent: Extent,
    counts: Counts,
    timing: Timing,
    /// Deep run with the pass OFF / ON (seconds). `None` when the baseline run was
    /// skipped (the gate does not pay for it).
    deep_off_s: Option<f64>,
    deep_on_s: f64,
    today: Residency,
    today_measured_bytes: usize,
    per_depth: Residency,
}

impl Projection {
    /// The pass's own measured wall clock (seconds) — the difference between the
    /// two deep runs, not a share guessed off a profile.
    fn pass_cost_s(&self) -> Option<f64> {
        self.deep_off_s.map(|off| self.deep_on_s - off)
    }
}

fn project(extent: Extent, measure_baseline: bool) -> Projection {
    let pregen = Pregen::run(WorldParams { seed: SEED, extent });
    let base_cfg = production_config(&pregen.grid, SEED);

    // The pass OFF — the baseline the pass's own cost is measured against. The
    // flag is the S-5 identity default, so this is the same world minus the pass.
    let deep_off_s = measure_baseline.then(|| {
        let t = Instant::now();
        black_box(build_field_cfg(&pregen.grid, &base_cfg));
        t.elapsed().as_secs_f64()
    });

    let cfg = DeepConfig {
        weather_inventory: true,
        ..base_cfg
    };
    let t = Instant::now();
    let field = build_field_cfg(&pregen.grid, &cfg);
    let deep_on_s = t.elapsed().as_secs_f64();

    // --- 1/2. tuple counts, straight out of the finished world -----------------
    let cells = field.strata.len();
    let chapters = cfg.chapters.max(1);
    let mut hist: Vec<CellHistory> = Vec::with_capacity(cells);
    for (i, s) in field.strata.iter().enumerate() {
        let bedrock_slot = s.units.len();
        let mut fired: Vec<u8> = field
            .ledgers
            .get(i)
            .map(|v| {
                v.facts_for(bedrock_slot)
                    .iter()
                    .map(Fact::chapter)
                    .collect()
            })
            .unwrap_or_default();
        fired.sort_unstable();
        fired.dedup();
        // Units are appended bottom-up, so `chapter` is non-decreasing along the
        // record: the slots existing at chapter c are a PREFIX of the final record.
        // +1 for the bedrock seam span, which exists from epoch 0. (Units later
        // eroded away are gone from the final record, so this is a lower bound on
        // the depth a firing actually saw — stated in the report.)
        let mut slots_by_chapter = Vec::with_capacity(chapters as usize);
        let mut k = 0usize;
        for c in 0..chapters {
            while k < s.units.len() && u32::from(s.units[k].chapter) <= c {
                k += 1;
            }
            slots_by_chapter.push(u32::try_from(k + 1).expect("slot count fits in u32"));
        }
        hist.push(CellHistory {
            chapters: fired,
            slots_by_chapter,
            slots_final: s.units.len() + 1,
        });
    }

    let mut cell_chapter_firings = 0u128;
    let mut slot_chapter_triangular = 0u128;
    let mut slot_chapter_flat = 0u128;
    for h in &hist {
        for &c in &h.chapters {
            cell_chapter_firings += 1;
            slot_chapter_triangular += h.slots_at(c);
            slot_chapter_flat += h.slots_final as u128;
        }
    }
    let counts = Counts {
        cells,
        epochs: cfg.iterations,
        chapters,
        epochs_per_chapter: cfg.iterations / chapters,
        loop_visits: cells as u128 * u128::from(cfg.iterations),
        cell_chapter_firings,
        slot_chapter_triangular,
        slot_chapter_flat,
    };

    // --- 3. per-invocation wall clock over real production columns -------------
    let timing = time_firings(&field.strata, &field.regolith, &cfg, &counts);

    // --- 5. residency ----------------------------------------------------------
    let today = Residency {
        cells,
        facts: field.ledgers.total_facts(),
        slot_rows: field.ledgers.slots_with_facts(),
    };
    // Per slot, the same agents commit the same facts per chapter, so the fact
    // count scales with the visited-slot count. Rows are per (cell, slot) that ever
    // carries a fact = the slots present at the cell's LAST firing chapter.
    let facts_per_cell_chapter = if cell_chapter_firings == 0 {
        0.0
    } else {
        today.facts as f64 / cell_chapter_firings as f64
    };
    let per_depth_rows: usize = hist
        .iter()
        .map(|h| h.chapters.last().map_or(0, |&c| h.slots_at(c) as usize))
        .sum();
    let per_depth = Residency {
        cells,
        facts: (slot_chapter_triangular as f64 * facts_per_cell_chapter).round() as usize,
        slot_rows: per_depth_rows,
    };

    Projection {
        extent,
        counts,
        timing,
        deep_off_s,
        deep_on_s,
        today,
        today_measured_bytes: field.ledgers.footprint_bytes(),
        per_depth,
    }
}

/// Time today's firing and the per-depth cost model over the **same** sample of
/// real production columns and the **same** firing schedule (every chapter, every
/// epoch), so the ratio is a like-for-like per-invocation cost.
fn time_firings(
    strata: &[DeepStrata],
    regolith: &[f64],
    cfg: &DeepConfig,
    counts: &Counts,
) -> Timing {
    // Stride-sample cells that carry a record; the depth distribution is what the
    // per-depth cost is a function of, so the sample must not be the deepest cells.
    let with_record: Vec<usize> = (0..strata.len())
        .filter(|&i| !strata[i].units.is_empty())
        .collect();
    let step = (with_record.len() / TIMING_SAMPLE.max(1)).max(1);
    let sample: Vec<usize> = with_record.iter().copied().step_by(step).collect();

    let inputs_for = |i: usize| WeatherInputs {
        weathering: cfg.weathering,
        h_star: cfg.h_star,
        regolith_h: regolith.get(i).copied().unwrap_or(0.0),
        // The biotic/frost planes are gen-time only and are not exported on the
        // field; the drivers' *values* do not change the shape of the work, and
        // both are 1.0-identity when their agent is off.
        biotic: 1.0,
        frost: 1.0,
    };
    let (chapters, per_chapter) = (counts.chapters, counts.epochs_per_chapter.max(1));
    let firings = sample.len() * (chapters * per_chapter) as usize;

    // (a) today: one bedrock span, built over an EMPTY record.
    let t = Instant::now();
    let mut acc = 0.0f64;
    for &i in &sample {
        let inputs = inputs_for(i);
        let mut ledger = empty_accumulator();
        for c in 0..chapters {
            for _ in 0..per_chapter {
                acc += weather_bedrock_epoch(&mut ledger, c as u8, &inputs, 1.0);
            }
        }
    }
    black_box(acc);
    let today_ns = t.elapsed().as_nanos() as f64 / firings as f64;

    // (b) per-depth, causally triangular: the record truncated to the slots that
    //     existed by each chapter.
    let t = Instant::now();
    let mut acc = 0.0f64;
    for &i in &sample {
        let inputs = inputs_for(i);
        let mut ledger = FactLedger::default();
        for c in 0..chapters {
            let mut truncated = strata[i].clone();
            truncated.units.retain(|u| u32::from(u.chapter) <= c);
            for _ in 0..per_chapter {
                acc += per_depth_firing_cost_model(&truncated, &mut ledger, c as u8, &inputs, 1.0);
            }
        }
    }
    black_box(acc);
    let per_depth_triangular_ns = t.elapsed().as_nanos() as f64 / firings as f64;

    // (c) per-depth, flat: full final depth at every epoch — the cost of forgetting
    //     the triangle.
    let t = Instant::now();
    let mut acc = 0.0f64;
    for &i in &sample {
        let inputs = inputs_for(i);
        let mut ledger = FactLedger::default();
        for c in 0..chapters {
            for _ in 0..per_chapter {
                acc += per_depth_firing_cost_model(&strata[i], &mut ledger, c as u8, &inputs, 1.0);
            }
        }
    }
    black_box(acc);
    let per_depth_flat_ns = t.elapsed().as_nanos() as f64 / firings as f64;

    Timing {
        firings,
        today_ns,
        per_depth_triangular_ns,
        per_depth_flat_ns,
    }
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn report(p: &Projection) {
    let c = &p.counts;
    println!(
        "\n=== R3: per-depth weathering cost projection (seed {SEED}, {}) ===\n",
        p.extent.label()
    );
    println!("--- 1. the tuple counts ---");
    println!(
        "  deep cells {}, epochs {}, chapters {} ({} epochs/chapter)",
        c.cells, c.epochs, c.chapters, c.epochs_per_chapter
    );
    println!(
        "  loop visits (cell x epoch, gate included)      {:>16}",
        c.loop_visits
    );
    println!(
        "  TODAY  (cell, epoch) firings                   {:>16} .. {} (lo/hi)",
        c.firings_lo(),
        c.firings_hi()
    );
    println!(
        "  PER-DEPTH (cell, slot, chapter), triangular    {:>16}",
        c.slot_chapter_triangular
    );
    println!(
        "  PER-DEPTH (cell, slot, chapter), flat          {:>16}",
        c.slot_chapter_flat
    );
    println!(
        "  => MULTIPLIER  triangular {:.2}x   flat {:.2}x   (the causal triangle saves {:.1}%)",
        c.multiplier_triangular(),
        c.multiplier_flat(),
        100.0 * (1.0 - c.multiplier_triangular() / c.multiplier_flat())
    );

    println!(
        "\n--- 2. per-invocation wall clock ({} firings each) ---",
        p.timing.firings
    );
    println!(
        "  today (one bedrock span)          {:>9.0} ns",
        p.timing.today_ns
    );
    println!(
        "  per-depth, triangular             {:>9.0} ns   ({:.1}x)",
        p.timing.per_depth_triangular_ns,
        p.timing.ratio_triangular()
    );
    println!(
        "  per-depth, flat                   {:>9.0} ns   ({:.1}x)",
        p.timing.per_depth_flat_ns,
        p.timing.ratio_flat()
    );

    println!("\n--- 3. gen time ---");
    println!("  deep run, pass ON                 {:>9.1} s", p.deep_on_s);
    match (p.deep_off_s, p.pass_cost_s()) {
        (Some(off), Some(cost)) => {
            println!("  deep run, pass OFF                {off:>9.1} s");
            println!(
                "  => the pass itself                {cost:>9.1} s  ({:.1}% of the run)",
                100.0 * cost / p.deep_on_s
            );
            let proj = cost * p.timing.ratio_triangular();
            println!(
                "  => per-depth (triangular)         {proj:>9.1} s  (+{:.1} s on the deep run, \
                 {:.1} s -> {:.1} s)",
                proj - cost,
                p.deep_on_s,
                p.deep_on_s - cost + proj
            );
            let proj_flat = cost * p.timing.ratio_flat();
            println!(
                "  => per-depth (flat, no triangle)  {proj_flat:>9.1} s  (+{:.1} s)",
                proj_flat - cost
            );
        }
        _ => println!("  (baseline run skipped)"),
    }

    println!("\n--- 4. residency (the LedgerField sidecar) ---");
    println!(
        "  today      facts {:>10}  slot rows {:>10}  = {:>8.2} MiB  (measured {:.2} MiB, \
         itemisation {})",
        p.today.facts,
        p.today.slot_rows,
        mib(p.today.bytes()),
        mib(p.today_measured_bytes),
        if p.today.check(p.today_measured_bytes) {
            "EXACT"
        } else {
            "MISMATCH"
        }
    );
    println!(
        "  per-depth  facts {:>10}  slot rows {:>10}  = {:>8.2} MiB  (+{:.2} MiB, {:.1}x)",
        p.per_depth.facts,
        p.per_depth.slot_rows,
        mib(p.per_depth.bytes()),
        mib(p.per_depth.bytes()) - mib(p.today.bytes()),
        p.per_depth.bytes() as f64 / p.today.bytes() as f64
    );
    println!(
        "  (one Fact is {} B; the sidecar is gen-time resident and ships in the DeepField)",
        std::mem::size_of::<Fact>()
    );
    println!();
}

fn main() {
    let p = project(EXTENT, true);
    report(&p);
}

/// **GATE** (journal/0103: an example that can fail belongs in the gate).
///
/// Runs at `Extent::Small` and skips the pass-OFF baseline. Every invariant
/// asserted here is **scale-free**: they are relations between counts derived from
/// one record (an itemisation equalling its own measured total; a triangular sum
/// bounded by its flat sum), not magnitudes. The production magnitudes are what
/// `main` is for.
#[cfg(test)]
mod gate {
    use super::*;

    #[test]
    fn the_perdepth_projection_is_self_consistent() {
        let p = project(Extent::Small, false);

        // The residency itemisation must reconstruct the record's own measured
        // footprint byte-for-byte. This is what makes the per-depth number a
        // projection of a *validated* model rather than an arithmetic guess — and
        // it is the assertion that fires if `Fact`, `SlotRun` or the CSR layout
        // moves under this probe (the flow_cost_probe failure mode, twice).
        assert!(
            p.today.check(p.today_measured_bytes),
            "ledger residency itemisation {} B != measured {} B — the model this \
             projection scales is wrong",
            p.today.bytes(),
            p.today_measured_bytes
        );

        // The pass must have fired somewhere, or nothing below means anything.
        assert!(
            p.counts.cell_chapter_firings > 0,
            "weather_inventory fired in no (cell, chapter) at all"
        );

        // A per-depth pass visits at least one slot per firing (the bedrock seam is
        // always present), so the multiplier cannot be below 1.
        assert!(
            p.counts.slot_chapter_triangular >= p.counts.cell_chapter_firings,
            "per-depth visits {} < today's {} — a per-depth firing cannot see fewer \
             slots than today's one",
            p.counts.slot_chapter_triangular,
            p.counts.cell_chapter_firings
        );

        // The causal triangle is a real saving, not a rhetorical one: a slot
        // deposited in chapter c is invisible to every firing before c, so the
        // triangular count is strictly below the flat one on any world whose record
        // grows during the run.
        assert!(
            p.counts.slot_chapter_triangular < p.counts.slot_chapter_flat,
            "triangular {} is not below flat {} — either the record does not grow \
             across chapters on this world, or the chapter prefix is mis-derived",
            p.counts.slot_chapter_triangular,
            p.counts.slot_chapter_flat
        );

        // The projected residency is the today figure scaled by the same visited-slot
        // ratio, so it must sit on the same side of it as the count does.
        assert!(
            p.per_depth.bytes() > p.today.bytes(),
            "per-slot state cannot be cheaper than one-slot state"
        );

        // Both timing arms did real work (the cost model is not silently a no-op).
        assert!(
            p.timing.today_ns > 0.0 && p.timing.per_depth_triangular_ns > 0.0,
            "a timing arm measured zero — the sample or the schedule is empty"
        );
    }
}
