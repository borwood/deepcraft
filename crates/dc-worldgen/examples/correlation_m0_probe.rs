//! **S0 / M0′ — the stratigraphic-correlation pre-slice measurement run.**
//! (`docs/audits/2026-08-03-stratigraphic-correlation-design.md` § 6;
//! results in `docs/audits/2026-08-04-s0-correlation-measurements.md`.)
//!
//! The correlation design left five quantities unmeasured and named them as the
//! gate on its own build sequencing. This probe answers the record-side ones in
//! one deep run over the shipped world:
//!
//! - **(a)** per-cell **H** (recorded column thickness) and **stack depth**
//!   distributions — the tails § 1.2 R-B's worst case and § 6's expressed-interval
//!   count both inherit.
//! - **(c)** **shared-partition interval counts under EPOCH correlation.** The
//!   design asked for *fraction breakpoints under R-C*; the deposition clock
//!   landed since (journal/0154), so correlation matches **true epoch intervals**
//!   and the shared partition over a stencil is simply the **union of the epochs
//!   its parents carry**. That union count IS the per-column dot-product constant.
//! - **(d)** the **mixture-cap check** for P-1's ruled **M-C**: over adjacent
//!   stencils, how many distinct **species sets** does a correlated interval
//!   present, and how many distinct blended mixtures can those mint at the
//!   eighths quantum.
//! - **(e)** **epoch statistics** — distinct epochs per cell, and the epoch **gap**
//!   at unconformity-flagged contacts (the measurable-gap claim's first numbers).
//! - **(g)** the **total recorded unit count**, re-measured to settle ROADMAP
//!   § Observed *"ONE COUNT, TWO VALUES"*.
//!
//! Measurement only — no asserts, no gate cost. Run it explicitly:
//!
//! ```text
//! cargo run --release -p dc-worldgen --example correlation_m0_probe
//! ```

use std::collections::BTreeMap;
use std::time::Instant;

use dc_core::materials::MaterialId;
use dc_worldgen::deeptime::DeepField;
use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;

/// 256 epochs fit exactly four `u64`s — a per-cell presence bitset, so a stencil
/// union is four ORs and four popcounts rather than a set allocation.
#[derive(Clone, Copy, Default)]
struct EpochBits([u64; 4]);

impl EpochBits {
    fn set(&mut self, e: u8) {
        self.0[usize::from(e) >> 6] |= 1u64 << (u32::from(e) & 63);
    }
    fn or(self, o: Self) -> Self {
        EpochBits([
            self.0[0] | o.0[0],
            self.0[1] | o.0[1],
            self.0[2] | o.0[2],
            self.0[3] | o.0[3],
        ])
    }
    fn count(self) -> u32 {
        self.0.iter().map(|w| w.count_ones()).sum()
    }
}

/// mean / median / p95 / max over a sample, plus its size.
struct Dist {
    n: usize,
    mean: f64,
    median: f64,
    p95: f64,
    max: f64,
    min: f64,
}

fn dist(v: &mut [f64]) -> Dist {
    if v.is_empty() {
        return Dist {
            n: 0,
            mean: 0.0,
            median: 0.0,
            p95: 0.0,
            max: 0.0,
            min: 0.0,
        };
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let s: &[f64] = v;
    let n = s.len();
    let pick = |q: f64| s[((n as f64 - 1.0) * q).round() as usize];
    Dist {
        n,
        mean: s.iter().sum::<f64>() / n as f64,
        median: pick(0.5),
        p95: pick(0.95),
        max: s[n - 1],
        min: s[0],
    }
}

fn row(label: &str, d: &Dist, unit: &str) {
    println!(
        "  {label:<34} n={:<8} mean {:>10.3} · median {:>10.3} · p95 {:>10.3} · \
         max {:>10.3} · min {:>8.3}  {unit}",
        d.n, d.mean, d.median, d.p95, d.max, d.min
    );
}

/// Number of ordered compositions of `TOTAL` eighths into `k` **positive** parts
/// — the distinct mixtures a `k`-material blend can express at the fill's
/// quantum. `C(7, k-1)` for eight eighths.
fn eighths_states(k: usize) -> u64 {
    if k == 0 || k > 8 {
        return 0;
    }
    // C(7, k-1)
    let (n, r) = (7u64, (k - 1) as u64);
    let mut num = 1u64;
    let mut den = 1u64;
    for i in 0..r {
        num *= n - i;
        den *= i + 1;
    }
    num / den
}

fn main() {
    println!("=== S0 / M0' — stratigraphic-correlation pre-slice measurements ===");
    println!("seed {SEED} · {EXTENT:?} · production config (Pregen::run)\n");
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let pregen_s = t0.elapsed().as_secs_f64();
    let f: &DeepField = &pregen.deep;
    println!(
        "Pregen::run {pregen_s:.2} s · deep grid {w}x{w} = {cells} cells · cell {cm:.1} m · \
         field {mib:.2} MiB",
        w = f.w,
        cells = f.w * f.w,
        cm = f.cell_m,
        mib = f.resident_bytes() as f64 / (1024.0 * 1024.0),
    );

    // ---------------------------------------------------------------- (a)+(g)
    let t = Instant::now();
    let n = f.w * f.w;
    let mut total_units = 0u64;
    let mut cells_with_record = 0usize;
    let mut depth_all: Vec<f64> = Vec::with_capacity(n);
    let mut depth_rec: Vec<f64> = Vec::new();
    let mut h_rec: Vec<f64> = Vec::new();
    let mut h_all: Vec<f64> = Vec::with_capacity(n);
    let mut regolith_gap_max = 0.0f64;
    let mut max_unit_m = 0.0f64;
    let mut depth_hist: BTreeMap<u64, u64> = BTreeMap::new();

    // ------------------------------------------------------------------- (e)
    let mut bits: Vec<EpochBits> = vec![EpochBits::default(); n];
    let mut distinct_epochs: Vec<f64> = Vec::with_capacity(n);
    let mut distinct_epochs_rec: Vec<f64> = Vec::new();
    let mut distinct_chapters_rec: Vec<f64> = Vec::new();
    let mut unconf_gaps: Vec<f64> = Vec::new();
    let mut unconf_contacts = 0u64;
    let mut unconf_at_base = 0u64;
    let mut unconf_zero_gap = 0u64;
    let mut epoch_nonmonotone = 0u64;
    let mut epoch_min = 255u8;
    let mut epoch_max = 0u8;

    for (i, s) in f.strata.iter().enumerate() {
        let d = s.units.len() as u64;
        total_units += d;
        depth_all.push(d as f64);
        *depth_hist.entry(d).or_default() += 1;
        let mut h = 0.0f64;
        let mut chap = [false; 256];
        let mut prev: Option<(u8, u8)> = None; // (epoch, chapter)
        for u in &s.units {
            let t_m = u.thickness_m();
            h += t_m;
            max_unit_m = max_unit_m.max(t_m);
            let e = u.epoch();
            epoch_min = epoch_min.min(e);
            epoch_max = epoch_max.max(e);
            bits[i].set(e);
            chap[usize::from(u.chapter())] = true;
            if let Some((pe, _)) = prev {
                if e < pe {
                    epoch_nonmonotone += 1;
                }
                if u.unconformity() {
                    unconf_contacts += 1;
                    let gap = i32::from(e) - i32::from(pe);
                    if gap == 0 {
                        unconf_zero_gap += 1;
                    }
                    unconf_gaps.push(f64::from(gap));
                }
            } else if u.unconformity() {
                unconf_at_base += 1;
            }
            prev = Some((e, u.chapter()));
        }
        h_all.push(h);
        let de = bits[i].count() as f64;
        distinct_epochs.push(de);
        if d > 0 {
            cells_with_record += 1;
            depth_rec.push(d as f64);
            h_rec.push(h);
            distinct_epochs_rec.push(de);
            distinct_chapters_rec.push(chap.iter().filter(|b| **b).count() as f64);
        }
        if let Some(r) = f.regolith.get(i) {
            regolith_gap_max = regolith_gap_max.max((r - h).abs());
        }
    }
    let scan_s = t.elapsed().as_secs_f64();

    println!("\n--- (g) THE UNIT COUNT (ROADMAP Observed, \"ONE COUNT, TWO VALUES\") ---");
    println!("  total recorded units .............. {total_units}");
    println!(
        "  cells ............................ {n} ({cells_with_record} with a record, \
         {:.1} %)",
        100.0 * cells_with_record as f64 / n as f64
    );
    println!(
        "  mean units / cell (all) .......... {:.3}   (per recorded cell: {:.3})",
        total_units as f64 / n as f64,
        total_units as f64 / cells_with_record.max(1) as f64
    );
    println!("  record scan {scan_s:.2} s");

    println!("\n--- (a) PER-CELL STACK DEPTH AND H ---");
    row("stack depth, all cells", &dist(&mut depth_all), "units");
    row("stack depth, recorded cells", &dist(&mut depth_rec), "units");
    row("H = Sum(unit thickness), all", &dist(&mut h_all), "m");
    row("H = Sum(unit thickness), recorded", &dist(&mut h_rec), "m");
    println!("  max single unit thickness ........ {max_unit_m:.3} m");
    println!(
        "  max |DeepField::regolith - Sum(units)| = {regolith_gap_max:.6e} m  \
         (the journal/0053 finalize invariant)"
    );
    println!("  stack-depth histogram (depth: cells), head and tail:");
    let hi: Vec<(u64, u64)> = depth_hist.iter().map(|(k, v)| (*k, *v)).collect();
    if hi.len() <= 40 {
        for (d, c) in &hi {
            println!("    {d:>5}: {c}");
        }
    } else {
        for (d, c) in hi.iter().take(16) {
            println!("    {d:>5}: {c}");
        }
        println!("    ... ({} intermediate depths elided) ...", hi.len() - 32);
        for (d, c) in hi.iter().skip(hi.len() - 16) {
            println!("    {d:>5}: {c}");
        }
    }

    println!("\n--- (e) EPOCH STATISTICS (the deposition clock, journal/0154) ---");
    println!("  epoch range present in the record: {epoch_min} ..= {epoch_max}");
    row(
        "distinct epochs / cell (all)",
        &dist(&mut distinct_epochs),
        "epochs",
    );
    row(
        "distinct epochs / recorded cell",
        &dist(&mut distinct_epochs_rec),
        "epochs",
    );
    row(
        "distinct chapters / recorded cell",
        &dist(&mut distinct_chapters_rec),
        "chapters",
    );
    println!("  epoch NON-monotone up-stack ...... {epoch_nonmonotone} (must be 0)");
    println!(
        "  unconformity-flagged contacts .... {unconf_contacts} interior + {unconf_at_base} \
         at the record base (no predecessor, no gap defined)"
    );
    println!(
        "  ... of the interior ones, {unconf_zero_gap} have a ZERO epoch gap ({:.1} %)",
        100.0 * unconf_zero_gap as f64 / unconf_contacts.max(1) as f64
    );
    let mut nz: Vec<f64> = unconf_gaps.iter().copied().filter(|g| *g > 0.0).collect();
    row(
        "epoch gap at flagged contacts",
        &dist(&mut unconf_gaps),
        "epochs",
    );
    row("... excluding zero gaps", &dist(&mut nz), "epochs");

    // ------------------------------------------------------------------- (c)
    println!("\n--- (c) SHARED-PARTITION INTERVAL COUNTS UNDER EPOCH CORRELATION ---");
    println!(
        "  (supersedes the design's fraction-breakpoint count: the clock landed, so the\n   \
         shared partition over a stencil is the UNION of the epochs its parents carry.)"
    );
    let w = f.w;
    let mut u2: Vec<f64> = Vec::with_capacity((w - 1) * (w - 1));
    let mut u2_rec: Vec<f64> = Vec::new();
    let mut u3: Vec<f64> = Vec::with_capacity((w - 2) * (w - 2));
    let mut u3_rec: Vec<f64> = Vec::new();
    for iy in 0..w - 1 {
        for ix in 0..w - 1 {
            let b = bits[iy * w + ix]
                .or(bits[iy * w + ix + 1])
                .or(bits[(iy + 1) * w + ix])
                .or(bits[(iy + 1) * w + ix + 1]);
            let c = f64::from(b.count());
            u2.push(c);
            if c > 0.0 {
                u2_rec.push(c);
            }
        }
    }
    for iy in 0..w - 2 {
        for ix in 0..w - 2 {
            let mut b = EpochBits::default();
            for dy in 0..3 {
                for dx in 0..3 {
                    b = b.or(bits[(iy + dy) * w + ix + dx]);
                }
            }
            let c = f64::from(b.count());
            u3.push(c);
            if c > 0.0 {
                u3_rec.push(c);
            }
        }
    }
    row("2x2 stencil union intervals (all)", &dist(&mut u2), "epochs");
    row(
        "2x2 stencil union, non-empty",
        &dist(&mut u2_rec),
        "epochs",
    );
    row(
        "3x3 stencil union (straddling)",
        &dist(&mut u3),
        "epochs",
    );
    row(
        "3x3 stencil union, non-empty",
        &dist(&mut u3_rec),
        "epochs",
    );

    // ------------------------------------------------------------------- (d)
    println!("\n--- (d) MIXTURE-CAP CHECK FOR P-1's M-C ---");
    println!(
        "  Per 2x2 stencil and per correlated EPOCH interval: the set of distinct species\n  \
         the parents present. |set| = k is the mixture-cap's k; the interval is a CONTINUUM\n  \
         candidate iff every member of the set shares one Litho class (the derived predicate\n  \
         is FS-A's job -- same-class is the measurable proxy, flagged in the audit)."
    );
    let t = Instant::now();
    // Per cell, per epoch: the set of species present, as a 64-bit mask over
    // MaterialId (the vanilla set is far under 64 members). Flat CSR so the
    // stencil sweep does no allocation at all.
    let mut starts: Vec<u32> = Vec::with_capacity(n + 1);
    let mut cell_ep: Vec<u8> = Vec::new();
    let mut cell_mask: Vec<u64> = Vec::new();
    {
        let mut scratch = [0u64; 256];
        let mut touched: Vec<u8> = Vec::new();
        for s in &f.strata {
            starts.push(cell_ep.len() as u32);
            for u in &s.units {
                let e = usize::from(u.epoch());
                if scratch[e] == 0 {
                    touched.push(u.epoch());
                }
                scratch[e] |= 1u64 << u.species().raw();
            }
            touched.sort_unstable();
            for &e in &touched {
                cell_ep.push(e);
                cell_mask.push(scratch[usize::from(e)]);
                scratch[usize::from(e)] = 0;
            }
            touched.clear();
        }
        starts.push(cell_ep.len() as u32);
    }
    // Distinct species-SETS observed over stencil intervals, and how often.
    let mut set_counts: BTreeMap<u64, u64> = BTreeMap::new();
    let mut k_hist: BTreeMap<usize, u64> = BTreeMap::new();
    let mut intervals_total = 0u64;
    let mut intervals_multi = 0u64;
    {
        let mut merged = [0u64; 256];
        let mut touched: Vec<u8> = Vec::new();
        for iy in 0..w - 1 {
            for ix in 0..w - 1 {
                let idx = [
                    iy * w + ix,
                    iy * w + ix + 1,
                    (iy + 1) * w + ix,
                    (iy + 1) * w + ix + 1,
                ];
                for j in idx {
                    for k in starts[j] as usize..starts[j + 1] as usize {
                        let e = usize::from(cell_ep[k]);
                        if merged[e] == 0 {
                            touched.push(cell_ep[k]);
                        }
                        merged[e] |= cell_mask[k];
                    }
                }
                for &e in &touched {
                    let m = merged[usize::from(e)];
                    merged[usize::from(e)] = 0;
                    intervals_total += 1;
                    let k = m.count_ones() as usize;
                    *k_hist.entry(k).or_default() += 1;
                    if k > 1 {
                        intervals_multi += 1;
                        *set_counts.entry(m).or_default() += 1;
                    }
                }
                touched.clear();
            }
        }
    }
    let d_s = t.elapsed().as_secs_f64();
    println!(
        "\n  stencil-intervals scanned ........ {intervals_total} ({intervals_multi} carry more \
         than one species, {:.2} %)  [{d_s:.2} s]",
        100.0 * intervals_multi as f64 / intervals_total.max(1) as f64
    );
    println!("\n  k = distinct species in a stencil-interval:");
    for (k, c) in &k_hist {
        println!(
            "    k={k:<3} {c:>12}  ({:.3} %)  eighths states C(7,k-1) = {}",
            100.0 * *c as f64 / intervals_total.max(1) as f64,
            eighths_states(*k)
        );
    }
    println!(
        "\n  distinct species SETS with k>1 (the mixture inventory): {}",
        set_counts.len()
    );
    println!(
        "  {:<8} {:>4} {:>14} {:>10} {:>8}  members",
        "class", "k", "occurrences", "states", "cont?"
    );
    let mut rows: Vec<(u64, u64)> = set_counts.iter().map(|(m, c)| (*m, *c)).collect();
    rows.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    let mut continuum_states = 0u64;
    let mut discrete_sets = 0u64;
    let mut continuum_sets = 0u64;
    for (m, c) in &rows {
        let mats: Vec<MaterialId> = MaterialId::all()
            .filter(|x| (*m >> x.raw()) & 1 == 1)
            .collect();
        let k = mats.len();
        let classes: Vec<Litho> = mats.iter().map(|x| Litho::of_material(*x)).collect();
        let continuum = classes.windows(2).all(|p| p[0] == p[1]);
        if continuum {
            continuum_sets += 1;
            continuum_states += eighths_states(k);
        } else {
            discrete_sets += 1;
        }
        println!(
            "  {:<8} {:>4} {:>14} {:>10} {:>8}  {}",
            classes[0].code(),
            k,
            c,
            eighths_states(k),
            if continuum { "CONT" } else { "disc" },
            mats
                .iter()
                .map(|x| x.qualified_name().to_string())
                .collect::<Vec<_>>()
                .join(" + ")
        );
    }
    println!(
        "\n  CONTINUUM sets (same Litho class, M-C's mixture branch): {continuum_sets}\n  \
         DISCRETE sets (M-C's octaves cut, no mixture minted):     {discrete_sets}\n  \
         distinct blended mixtures the mixture branch can mint (upper bound at the\n  \
         fill's eighths quantum, summed over continuum sets): {continuum_states}"
    );

    println!("\n=== done ({:.2} s total) ===", t0.elapsed().as_secs_f64());
}
