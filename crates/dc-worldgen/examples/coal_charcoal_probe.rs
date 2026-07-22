//! Measurement harness for the two organic-facies fixes (journal/0063):
//!
//! 1. **Coal promotion on the burial axis.** Prints the joint distribution of
//!    every peat-derived unit's *thickness* against its *burial depth*
//!    (Σ thickness of the units above it), so the promotion threshold can be
//!    chosen against data rather than against a preserved golden. Run it on
//!    either side of the change: the pre-promotion population is
//!    `Peat ∪ Coal`, which is invariant under whichever rule promoted.
//! 2. **Charcoal expression.** Prints the fire-bed population (count, thickness
//!    quantiles) and then the *measured* world-side answer: over a stride of
//!    sampled chunk columns, what fraction of recorded voxel spans allocate at
//!    least one eighth to a charcoal member, using the generator's own
//!    [`ColumnFill`] + addressed draw.
//!
//! Run: `cargo run --release -p dc-worldgen --example coal_charcoal_probe`

use dc_core::materials::geology::{self, GeologySet};
use dc_worldgen::WorldGenerator;
use dc_worldgen::deeptime::Biofacies;
use dc_worldgen::fill::{ColumnFill, Plan, allocate, fill_draw};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;
const VOXEL_M: f64 = 0.9;

fn pct(a: usize, b: usize) -> f64 {
    if b == 0 {
        0.0
    } else {
        a as f64 * 100.0 / b as f64
    }
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

fn main() {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });

    // ---------- 1. the peat/coal population, on both axes -------------------
    // (thickness_m, burial_m, already_coal)
    let mut organics: Vec<(f64, f64, bool)> = Vec::new();
    let mut char_t: Vec<f64> = Vec::new();
    let mut char_cols = 0usize;
    let mut readable = 0usize;
    for s in &pregen.deep.strata {
        if s.units.is_empty() {
            continue;
        }
        readable += 1;
        let mut over = 0.0f64;
        let mut has_char = false;
        for u in s.units.iter().rev() {
            match u.tag.biota {
                Biofacies::Peat => organics.push((u.thickness_m, over, false)),
                Biofacies::Coal => organics.push((u.thickness_m, over, true)),
                Biofacies::Charcoal => {
                    char_t.push(u.thickness_m);
                    has_char = true;
                }
                _ => {}
            }
            over += u.thickness_m;
        }
        if has_char {
            char_cols += 1;
        }
    }
    println!("readable columns: {readable}");
    println!(
        "peat-derived units: {} ({} already tagged Coal by the shipped rule)",
        organics.len(),
        organics.iter().filter(|o| o.2).count()
    );

    // Burial-depth histogram of the buried (burial > 0) part of the population.
    let bins = [
        0.0,
        0.5,
        1.0,
        2.0,
        5.0,
        10.0,
        20.0,
        50.0,
        100.0,
        f64::INFINITY,
    ];
    println!("\nburial-depth histogram of peat-derived units (all, incl. surface):");
    for w in bins.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        let n = organics.iter().filter(|o| o.1 >= lo && o.1 < hi).count();
        println!(
            "  [{lo:>6.1}, {hi:>6.1}) m  {n:>7}  {:>5.2} %",
            pct(n, organics.len())
        );
    }

    // Candidate rules. `old` is the shipped thickness rule; the rest are burial
    // thresholds. For each: units promoted, columns with ≥1 coal unit, thickest
    // coal unit, total coal metres.
    let report = |name: String, keep: &dyn Fn(f64, f64) -> bool| {
        let hits: Vec<&(f64, f64, bool)> = organics.iter().filter(|o| keep(o.0, o.1)).collect();
        let total: f64 = hits.iter().map(|h| h.0).sum();
        let max = hits.iter().map(|h| h.0).fold(0.0f64, f64::max);
        println!(
            "  {name:<28} units {:>6}  total {:>9.1} m  thickest {:>7.2} m",
            hits.len(),
            total,
            max
        );
    };
    println!("\ncandidate promotion rules (unit-level):");
    report("OLD thickness ≥ 0.4 m".into(), &|t: f64, b: f64| {
        b > 0.0 && t >= 0.4
    });
    for d in [0.5f64, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 20.0, 50.0] {
        report(format!("burial ≥ {d} m"), &|_t, b| b >= d);
    }

    // Column-level counts for the same candidates (what the S10 table reports).
    let col_count = |keep: &dyn Fn(f64, f64) -> bool| -> usize {
        let mut n = 0;
        for s in &pregen.deep.strata {
            let mut over = 0.0f64;
            let mut hit = false;
            for u in s.units.iter().rev() {
                if matches!(u.tag.biota, Biofacies::Peat | Biofacies::Coal)
                    && keep(u.thickness_m, over)
                {
                    hit = true;
                }
                over += u.thickness_m;
            }
            if hit {
                n += 1;
            }
        }
        n
    };
    println!("\ncandidate promotion rules (column-level):");
    println!(
        "  {:<28} columns {:>6}  {:>5.2} %",
        "OLD thickness ≥ 0.4 m",
        col_count(&|t, b| b > 0.0 && t >= 0.4),
        pct(col_count(&|t, b| b > 0.0 && t >= 0.4), readable)
    );
    for d in [0.5f64, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 20.0, 50.0] {
        let c = col_count(&|_t, b| b >= d);
        println!(
            "  {:<28} columns {:>6}  {:>5.2} %",
            format!("burial ≥ {d} m"),
            c,
            pct(c, readable)
        );
    }

    // ---------- 2. the charcoal population ----------------------------------
    char_t.sort_by(f64::total_cmp);
    let n = char_t.len();
    let sum: f64 = char_t.iter().sum();
    println!(
        "\ncharcoal beds: {n} in {char_cols} columns ({:.2} % of readable)",
        pct(char_cols, readable)
    );
    if n > 0 {
        println!(
            "  thickness  mean {:.4} m  p50 {:.4}  p90 {:.4}  p99 {:.4}  max {:.4}",
            sum / n as f64,
            quantile(&char_t, 0.5),
            quantile(&char_t, 0.9),
            quantile(&char_t, 0.99),
            quantile(&char_t, 1.0)
        );
        let eighths = |t: f64| 8.0 * t / VOXEL_M;
        println!(
            "  share in eighths of a 0.9 m span:  mean {:.3}  p50 {:.3}  p99 {:.3}  max {:.3}",
            eighths(sum / n as f64),
            eighths(quantile(&char_t, 0.5)),
            eighths(quantile(&char_t, 0.99)),
            eighths(quantile(&char_t, 1.0))
        );
        println!(
            "  beds ≥ one whole eighth (0.1125 m): {} ({:.3} %); ≥ a whole voxel: {}",
            char_t.iter().filter(|t| **t >= VOXEL_M / 8.0).count(),
            pct(char_t.iter().filter(|t| **t >= VOXEL_M / 8.0).count(), n),
            char_t.iter().filter(|t| **t >= VOXEL_M).count()
        );
    }

    // ---------- 3. the measured world-side expression -----------------------
    // Walk a stride of chunk columns, slice each recorded column the way the
    // generator does, and count voxel spans by which materials win eighths.
    let set: GeologySet = geology::vanilla();
    let mut g = WorldGenerator::new(&pregen);
    let mut spans = 0usize;
    let mut charcoal_spans = 0usize;
    let mut charcoal_eighths = 0u64;
    let mut coal_spans = 0usize;
    let mut total_eighths = 0u64;
    let mut sampled_cols = 0usize;
    let stride = 149i64; // coprime-ish with the chunk grid; ~40x40 columns
    for cz in (-3000..3000).step_by(stride as usize) {
        for cx in (-3000..3000).step_by(stride as usize) {
            let col = g.column_record(cx, cz);
            if col.strata.events.is_empty() {
                continue;
            }
            sampled_cols += 1;
            let cf = ColumnFill::build(&col.strata, VOXEL_M);
            // One voxel column per chunk column (local 0,0) — the plan is shared,
            // only the addressed draw differs, so this samples the draw space
            // without paying 1024×.
            let (lx, lz) = (0i64, 0i64);
            let (vx, vz) = (cx * 32 + lx, cz * 32 + lz);
            let surf = i64::from(col.heights[0]);
            for d in 1..=cf.depth_count() as u32 {
                let vy = surf - i64::from(d);
                spans += 1;
                let parts: Vec<(usize, u8)> = match cf.plan(d) {
                    Some(Plan::Single(i)) => vec![(*i, 8u8)],
                    Some(Plan::Mixed(w)) => allocate(w, fill_draw(SEED, vx, vy, vz)),
                    None => continue,
                };
                let mut has_char = false;
                let mut has_coal = false;
                for (i, k) in parts {
                    total_eighths += u64::from(k);
                    let m = &set.member(col.strata.events[i].member);
                    if m.material.props().name == "charcoal" {
                        has_char = true;
                        charcoal_eighths += u64::from(k);
                    }
                    if m.material == dc_core::MaterialId::COAL {
                        has_coal = true;
                    }
                }
                if has_char {
                    charcoal_spans += 1;
                }
                if has_coal {
                    coal_spans += 1;
                }
            }
        }
    }
    println!("\nworld sample: {sampled_cols} recorded chunk columns, {spans} recorded voxel spans");
    println!(
        "  spans carrying ≥1 charcoal eighth: {charcoal_spans} ({:.4} %); charcoal eighths {charcoal_eighths} of {total_eighths} ({:.4} %)",
        pct(charcoal_spans, spans),
        charcoal_eighths as f64 * 100.0 / total_eighths.max(1) as f64
    );
    println!(
        "  spans carrying coal: {coal_spans} ({:.4} %)",
        pct(coal_spans, spans)
    );
}
