//! **The pore-rider decorrelation probe** (journal/0105).
//!
//! A contact voxel at a weathering front makes *two* stochastic decisions:
//! `allocate_partial` splits the voxel's eight eighths between the bands that
//! overlap it, and `pore_rider_share` then splits each band's winnings between
//! parent rock and weathering product. Until 2026-07-25 both read one number —
//! `pore_rider_share` sliced bits 8–10 out of the very `fill_draw` the
//! allocation consumes, under a comment claiming that was disjoint from it. It
//! was not (ROADMAP Observed, found by journal/0103), and the user's call was
//! *"this needs fixed either way. Decorrelate."*
//!
//! Three questions, three parts, and this probe answers all three from **one
//! run**: it reproduces the retired offset formula in one line beside the new
//! draw, so before and after are measured on the same voxels of the same world
//! rather than across two checkouts.
//!
//! 1. **How correlated was it?** "They share bits" is a code reading. Part 1
//!    reports *predictability* — how often the pore offset can be read straight
//!    out of the fill offset — over every 3-bit window the fill offset has.
//! 2. **What did the coupling do to the arithmetic?** Part 2: the residual
//!    correlation between the two decisions, the entropy of the pore dither
//!    *conditioned on what the allocation did*, and the one that turned out to
//!    matter — sibling bands inside a single voxel sharing one offset, so their
//!    rounding errors add instead of cancelling.
//! 3. **Was it VISIBLE?** Part 3 measures spatial structure, because banding is
//!    spatial structure: one chunk-column's 32×32 contact plane (every voxel
//!    column there shares one record and one fill plan, so the *only* thing that
//!    varies across the plane is the draw), its autocorrelation at lags 1–4 in
//!    both axes, its sign-run lengths, and an ASCII map of the field.
//! 4. **What moved?** Part 4 counts the decisions and voxels whose expression
//!    changed, against the recorded voxels sampled.
//!
//! The reconstruction runs off the **public generation path** — `column_record`,
//! `ColumnFill`, `allocate_partial`, `pore_draw`, `pore_rider_share` — and
//! mirrors `collapse::WorldGenerator::mixed_at` arm for arm. It covers the
//! **buried** `Mixed` voxels (`plan(depth+1)`, a full eight eighths); the surface
//! voxel's partial is left out, deliberately, so every sample is one law.
//!
//! `cargo run --release -p dc-worldgen --example pore_decorrelation_probe`

use std::collections::HashMap;

use rayon::prelude::*;

use dc_core::materials::geology::vanilla;
use dc_worldgen::collapse::WorldGenerator;
use dc_worldgen::fill::{
    FILL_OFFSET_BITS, allocate_partial, fill_draw, fill_offset, is_loose, pore_draw,
    pore_rider_share, share_eighths,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{DeepOverrides, Plan};

/// The production world the weathering probes read (journal/0099, /0103).
const SEED: u64 = 1337;
/// Border ring excluded from station picks (march edge artifacts).
const EDGE_MARGIN: i64 = 4;
/// Chunk-columns sampled for the population statistics.
const REPORT_CHUNKS: usize = 48;

/// **The retired offset, reproduced exactly.**
///
/// ```ignore
/// let uq = (u * 4096.0) as u64 & 7;
/// ```
///
/// `u * 4096` is the top twelve bits of the fill draw and `& 7` keeps the lowest
/// three of those — bits 8–10 of the [`fill_offset`] the allocation consumes,
/// which is `FILL_OFFSET_BITS` = 20 wide. Hence the shift below: this is not an
/// *approximation* of the old code, it is the old code.
const RETIRED_SHIFT: u32 = 8;

fn retired_offset(u: f64) -> u64 {
    (u * 4096.0) as u64 & 7
}

/// The retired `pore_rider_share`, likewise reproduced exactly, so "what moved"
/// is a measurement rather than a recollection.
fn retired_share(cnt: u8, k8: u8, off: u64) -> u8 {
    let n = (u64::from(cnt) * u64::from(k8.min(8)) + off) / 8;
    (n as u8).min(cnt)
}

/// One pore-rider rounding decision, both ways.
#[derive(Clone, Copy)]
struct Rider {
    /// Index into the column's recorded events — the band whose host carries it.
    event: usize,
    /// Eighths of the voxel the host band won from `allocate_partial`.
    cnt: u8,
    /// The band's recorded pore share, in eighths.
    k8: u8,
    /// The host's *fractional* entitlement in eighths, of which `cnt` is the
    /// stochastic rounding. `cnt - entitle` is the allocation's own residual.
    entitle: f64,
    /// The whole `FILL_OFFSET_BITS`-wide offset the allocation saw.
    uq: u64,
    off_old: u64,
    off_new: u64,
    g_old: u8,
    g_new: u8,
}

impl Rider {
    /// The allocation's rounding residual, mean zero by construction.
    fn alloc_residual(&self) -> f64 {
        f64::from(self.cnt) - self.entitle
    }
    /// The rider's rounding residual in eighths, mean zero over the offset.
    fn pore_residual(&self, new: bool) -> f64 {
        let g = if new { self.g_new } else { self.g_old };
        f64::from(g) - f64::from(self.cnt) * f64::from(self.k8) / 8.0
    }
}

/// One buried `Mixed` voxel that carries at least one pore rider.
struct VoxelSample {
    lx: usize,
    lz: usize,
    riders: Vec<Rider>,
}

impl VoxelSample {
    fn total_residual(&self, new: bool) -> f64 {
        self.riders.iter().map(|r| r.pore_residual(new)).sum()
    }
}

/// Everything one chunk-column contributes.
#[derive(Default)]
struct ChunkSample {
    voxels: Vec<VoxelSample>,
    /// Buried voxels with a recorded fill plan — the denominator for "what
    /// fraction of the world moved".
    recorded_voxels: usize,
    mixed_voxels: usize,
}

/// World voxel centre of deep cell `idx` (lifted from
/// `examples/weathering_profile_probe.rs`).
fn idx_to_voxel(w: usize, wp: usize, idx: usize) -> (i64, i64) {
    let (wf, wpf) = (w as f64, wp as f64);
    let (gx, gy) = ((idx % w) as f64, (idx / w) as f64);
    let px = (gx + 0.5) / wf * wpf - 0.5;
    let py = (gy + 0.5) / wf * wpf - 0.5;
    let half = (wp / 2) as f64;
    (
        ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
    )
}

fn interior(w: usize, idx: usize) -> bool {
    let (gx, gy) = ((idx % w) as i64, (idx / w) as i64);
    let w = w as i64;
    gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
}

/// Every banded interior cell of a pregen, strongest band first.
fn banded_cells(pregen: &Pregen) -> Vec<usize> {
    let w = pregen.deep.w;
    let band = |i: usize| -> f64 {
        pregen.deep.ledgers.get(i).map_or(0.0, |l| {
            l.weathering_product_m(pregen.deep.strata[i].units.len())
        })
    };
    let mut banded: Vec<(usize, f64)> = (0..w * w)
        .filter(|&i| interior(w, i) && band(i) > 1e-6)
        .map(|i| (i, band(i)))
        .collect();
    banded.sort_by(|a, b| b.1.total_cmp(&a.1));
    banded.into_iter().map(|(i, _)| i).collect()
}

/// The production world this probe reads, with the weathering inventory **on** —
/// the only thing in the world that emits a *loose* pore rider, and therefore the
/// only thing that makes a pore-rider decision exist at all.
fn production_world(extent: Extent) -> Pregen {
    Pregen::run_with(
        WorldParams { seed: SEED, extent },
        &DeepOverrides {
            weather_inventory: Some(true),
            ..DeepOverrides::default()
        },
    )
}

/// **The control that explains the goldens.** The same seed and extent with
/// `DeepOverrides::default()` — i.e. `weather_inventory` **off**, which is the
/// shipped default (`grid.rs`: *"the production flip is the user's"*). Measured
/// at the very chunk-columns the flag-on run found its strongest fronts in, so
/// the two numbers are about the same places.
fn shipped_default_world(extent: Extent) -> Pregen {
    Pregen::run(WorldParams { seed: SEED, extent })
}

/// **The reconstruction.** Every buried `Mixed` voxel of one chunk-column, with
/// its pore-rider decisions resolved both ways.
///
/// This mirrors `mixed_at` arm for arm, and the order of the arms is
/// load-bearing: a placer `ore` with `k8 > 0` matches *first* and consumes the
/// event, so those voxels make no pore-rider decision at all and must not be
/// counted here.
fn sample_chunk(pregen: &Pregen, cx: i64, cz: i64) -> ChunkSample {
    let set = vanilla();
    let seed = pregen.seed;
    let mut generator = WorldGenerator::new(pregen);
    let rec = generator.column_record(cx, cz);
    let mut out = ChunkSample::default();
    for lz in 0..32usize {
        for lx in 0..32usize {
            // P11 slice 3: each voxel column's own SubCell (record + fill).
            let Some(sub) = rec.record_for(lx, lz) else {
                continue;
            };
            let fill = sub.fill();
            let h = i64::from(rec.heights[lz * 32 + lx]);
            let (vx, vz) = (cx * 32 + lx as i64, cz * 32 + lz as i64);
            // `plan(1)` is the surface voxel's partial (journal/0074); the buried
            // column starts at `plan(2)`, always a full eight eighths.
            for p in 2..=fill.depth_count() {
                let Some(plan) = fill.plan(p as u32) else {
                    continue;
                };
                out.recorded_voxels += 1;
                let Plan::Mixed(w) = plan else { continue };
                out.mixed_voxels += 1;
                let vy = h - (p as i64 - 1);
                let u = fill_draw(seed, vx, vy, vz);
                let uq = fill_offset(u);
                let mut riders = Vec::new();
                for (k, cnt) in allocate_partial(w, u, 8) {
                    let e = &sub.strata().events[k];
                    if e.ore.is_some_and(|(_, k8)| k8 > 0) {
                        continue; // the placer arm wins the match first
                    }
                    let Some((rider, k8)) = e.accessory else {
                        continue;
                    };
                    if k8 == 0 || !is_loose(&set, rider) {
                        continue; // a structural accessory is dropped at contacts
                    }
                    let entitle = w
                        .iter()
                        .find(|(j, _)| *j == k)
                        .map_or(0.0, |&(_, x)| share_eighths(x));
                    let off_old = retired_offset(u);
                    let d = pore_draw(seed, vx, vy, vz, k);
                    riders.push(Rider {
                        event: k,
                        cnt,
                        k8,
                        entitle,
                        uq,
                        off_old,
                        off_new: d.offset(),
                        g_old: retired_share(cnt, k8, off_old),
                        g_new: pore_rider_share(cnt, k8, d),
                    });
                }
                if !riders.is_empty() {
                    out.voxels.push(VoxelSample { lx, lz, riders });
                }
            }
        }
    }
    out
}

/// The chunk-columns the census walks: the strongest banded cells, one
/// chunk-column each.
fn census_chunks(pregen: &Pregen, want: usize) -> Vec<(i64, i64)> {
    let (w, wp) = (pregen.deep.w, pregen.deep.wp);
    let mut seen: Vec<(i64, i64)> = Vec::new();
    for cell in banded_cells(pregen) {
        let (vx, vz) = idx_to_voxel(w, wp, cell);
        let key = (vx.div_euclid(32), vz.div_euclid(32));
        if !seen.contains(&key) {
            seen.push(key);
        }
        if seen.len() >= want {
            break;
        }
    }
    seen
}

// ---------------------------------------------------------------- statistics

fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return 0.0;
    }
    sxy / (sxx * syy).sqrt()
}

/// Mutual information in **bits** between two discrete labellings.
///
/// A plug-in estimator is biased *upward* on finite samples, which is exactly
/// the direction that would flatter this slice — so every call site here is
/// paired with a control drawn from an address that is independent by
/// construction, and the control's value is printed as the floor.
fn mutual_information(pairs: &[(usize, usize)]) -> f64 {
    let n = pairs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mut joint: HashMap<(usize, usize), f64> = HashMap::new();
    let mut ma: HashMap<usize, f64> = HashMap::new();
    let mut mb: HashMap<usize, f64> = HashMap::new();
    for &(a, b) in pairs {
        *joint.entry((a, b)).or_default() += 1.0;
        *ma.entry(a).or_default() += 1.0;
        *mb.entry(b).or_default() += 1.0;
    }
    joint
        .iter()
        .map(|(&(a, b), &c)| {
            let pab = c / n;
            let pa = ma[&a] / n;
            let pb = mb[&b] / n;
            pab * (pab / (pa * pb)).log2()
        })
        .sum()
}

/// Shannon entropy in bits of a labelling.
fn entropy(labels: &[usize]) -> f64 {
    let n = labels.len() as f64;
    if n < 1.0 {
        return 0.0;
    }
    let mut counts: HashMap<usize, f64> = HashMap::new();
    for &l in labels {
        *counts.entry(l).or_default() += 1.0;
    }
    -counts
        .values()
        .map(|c| (c / n) * (c / n).log2())
        .sum::<f64>()
}

/// Bin a residual in `(-1, 1)` into eight cells, for the MI estimate.
fn residual_bin(r: f64) -> usize {
    (((r + 1.0) * 4.0) as isize).clamp(0, 7) as usize
}

// ---------------------------------------------------------------- the census

/// The whole measurement, as one value — so `main` can print it and the gate can
/// assert on it without either re-deriving the other's numbers (CLAUDE.md
/// § Gates).
struct Census {
    chunks: usize,
    recorded_voxels: usize,
    mixed_voxels: usize,
    rider_voxels: usize,
    decisions: usize,
    /// Voxels carrying two or more rider decisions — where sharing one offset
    /// stops being a curiosity and starts adding errors.
    multi_rider_voxels: usize,
    /// Part 1: match rate of each 3-bit window of the fill offset against the
    /// pore offset, `(shift, old rate, new rate)`.
    windows: Vec<(u32, f64, f64)>,
    /// Part 2.
    r_alloc_pore_old: f64,
    r_alloc_pore_new: f64,
    mi_alloc_pore_old: f64,
    mi_alloc_pore_new: f64,
    mi_floor: f64,
    /// Entropy of the pore offset conditioned on `(event, cnt)` — the dither's
    /// surviving randomness given everything the allocation decided. Max 3 bits.
    cond_entropy_old: f64,
    cond_entropy_new: f64,
    cond_entropy_min_old: f64,
    cond_entropy_min_new: f64,
    cond_groups: usize,
    /// Sibling riders in one voxel: correlation of their residuals.
    r_sibling_old: f64,
    r_sibling_new: f64,
    sibling_pairs: usize,
    /// `Var(Σ residuals in a voxel) / Σ Var(residual)`. Independent decisions
    /// give 1.0; perfectly coupled decisions give the number of siblings.
    var_ratio_old: f64,
    var_ratio_new: f64,
    /// Part 4.
    decisions_moved: usize,
    voxels_moved: usize,
    eighths_moved: i64,
}

fn measure(pregen: &Pregen, chunks: &[(i64, i64)]) -> Census {
    let samples: Vec<ChunkSample> = chunks
        .par_iter()
        .map(|&(cx, cz)| sample_chunk(pregen, cx, cz))
        .collect();

    let recorded_voxels = samples.iter().map(|s| s.recorded_voxels).sum();
    let mixed_voxels = samples.iter().map(|s| s.mixed_voxels).sum();
    let voxels: Vec<&VoxelSample> = samples.iter().flat_map(|s| s.voxels.iter()).collect();
    let riders: Vec<Rider> = voxels
        .iter()
        .flat_map(|v| v.riders.iter().copied())
        .collect();

    // Part 1 — predictability, over every window the fill offset has.
    let n = riders.len().max(1) as f64;
    let windows: Vec<(u32, f64, f64)> = (0..=(FILL_OFFSET_BITS - 3))
        .map(|shift| {
            let mut old_hits = 0usize;
            let mut new_hits = 0usize;
            for r in &riders {
                let win = (r.uq >> shift) & 7;
                old_hits += usize::from(win == r.off_old);
                new_hits += usize::from(win == r.off_new);
            }
            (shift, old_hits as f64 / n, new_hits as f64 / n)
        })
        .collect();

    // Part 2 — the two decisions against each other.
    let alloc: Vec<f64> = riders.iter().map(Rider::alloc_residual).collect();
    let pore_old: Vec<f64> = riders.iter().map(|r| r.pore_residual(false)).collect();
    let pore_new: Vec<f64> = riders.iter().map(|r| r.pore_residual(true)).collect();
    let pairs = |p: &[f64]| -> Vec<(usize, usize)> {
        alloc
            .iter()
            .zip(p)
            .map(|(&a, &b)| (residual_bin(a), residual_bin(b)))
            .collect()
    };
    // The floor: an offset from an address that cannot be related to the fill
    // draw, put through the same estimator on the same samples.
    let control: Vec<f64> = riders
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let d = pore_draw(pregen.seed ^ 0xA5A5, i as i64, 0, 0, r.event);
            f64::from(retired_share(r.cnt, r.k8, d.offset()))
                - f64::from(r.cnt) * f64::from(r.k8) / 8.0
        })
        .collect();

    // Conditional dither entropy: group by (event, cnt) — everything the
    // allocation decided about this band — and ask what randomness is left.
    let mut groups: HashMap<(usize, u8), (Vec<usize>, Vec<usize>)> = HashMap::new();
    for r in &riders {
        let g = groups.entry((r.event, r.cnt)).or_default();
        g.0.push(r.off_old as usize);
        g.1.push(r.off_new as usize);
    }
    let big: Vec<&(Vec<usize>, Vec<usize>)> =
        groups.values().filter(|(a, _)| a.len() >= 64).collect();
    let mean_or = |v: &[f64]| {
        if v.is_empty() {
            f64::NAN
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let h_old: Vec<f64> = big.iter().map(|(a, _)| entropy(a)).collect();
    let h_new: Vec<f64> = big.iter().map(|(_, b)| entropy(b)).collect();

    // Siblings in one voxel.
    let mut sib_a_old = Vec::new();
    let mut sib_b_old = Vec::new();
    let mut sib_a_new = Vec::new();
    let mut sib_b_new = Vec::new();
    for v in &voxels {
        for i in 0..v.riders.len() {
            for j in (i + 1)..v.riders.len() {
                sib_a_old.push(v.riders[i].pore_residual(false));
                sib_b_old.push(v.riders[j].pore_residual(false));
                sib_a_new.push(v.riders[i].pore_residual(true));
                sib_b_new.push(v.riders[j].pore_residual(true));
            }
        }
    }
    let multi: Vec<&VoxelSample> = voxels
        .iter()
        .copied()
        .filter(|v| v.riders.len() >= 2)
        .collect();
    // **The error-cancellation ratio.** Both residuals are mean-zero over their
    // offset by construction, so `Σ_voxels (Σ_riders r)²` is the voxel total's
    // second moment and `Σ_voxels Σ_riders r²` is what it would be if the riders
    // were independent. Independent decisions give 1.0; decisions that always
    // round the same way give the band count. Written as a moment ratio rather
    // than a variance ratio so voxels with different band counts compose.
    let var_ratio = |new: bool| -> f64 {
        let mut joint = 0.0;
        let mut apart = 0.0;
        for v in &multi {
            joint += v.total_residual(new).powi(2);
            apart += v
                .riders
                .iter()
                .map(|r| r.pore_residual(new).powi(2))
                .sum::<f64>();
        }
        if apart <= 0.0 { 0.0 } else { joint / apart }
    };

    // Part 4 — what moved.
    let decisions_moved = riders.iter().filter(|r| r.g_old != r.g_new).count();
    let voxels_moved = voxels
        .iter()
        .filter(|v| v.riders.iter().any(|r| r.g_old != r.g_new))
        .count();
    let eighths_moved: i64 = riders
        .iter()
        .map(|r| i64::from(r.g_new) - i64::from(r.g_old))
        .sum();

    Census {
        chunks: chunks.len(),
        recorded_voxels,
        mixed_voxels,
        rider_voxels: voxels.len(),
        decisions: riders.len(),
        multi_rider_voxels: multi.len(),
        windows,
        r_alloc_pore_old: pearson(&alloc, &pore_old),
        r_alloc_pore_new: pearson(&alloc, &pore_new),
        mi_alloc_pore_old: mutual_information(&pairs(&pore_old)),
        mi_alloc_pore_new: mutual_information(&pairs(&pore_new)),
        mi_floor: mutual_information(&pairs(&control)),
        cond_entropy_old: mean_or(&h_old),
        cond_entropy_new: mean_or(&h_new),
        cond_entropy_min_old: h_old.iter().copied().fold(f64::INFINITY, f64::min),
        cond_entropy_min_new: h_new.iter().copied().fold(f64::INFINITY, f64::min),
        cond_groups: big.len(),
        r_sibling_old: pearson(&sib_a_old, &sib_b_old),
        r_sibling_new: pearson(&sib_a_new, &sib_b_new),
        sibling_pairs: sib_a_old.len(),
        var_ratio_old: var_ratio(false),
        var_ratio_new: var_ratio(true),
        decisions_moved,
        voxels_moved,
        eighths_moved,
    }
}

// -------------------------------------------------------- part 3: the plane

/// A 32×32 contact plane: the per-voxel-column total pore residual inside one
/// chunk-column, where every column shares one record and one fill plan, so the
/// **only** thing varying across the field is the draw. If the two coupled draws
/// print a pattern anywhere, they print it here.
struct Plane {
    field_old: Vec<f64>,
    field_new: Vec<f64>,
    covered: usize,
}

/// Autocorrelation of a 32×32 field at `lag`, along x and along z.
fn autocorr(field: &[f64], lag: usize) -> (f64, f64) {
    let m = field.iter().sum::<f64>() / field.len() as f64;
    let c = |a: usize, b: usize| (field[a] - m) * (field[b] - m);
    let denom: f64 = field.iter().map(|v| (v - m) * (v - m)).sum();
    if denom <= 0.0 {
        return (0.0, 0.0);
    }
    let mut sx = 0.0;
    let mut sz = 0.0;
    let mut nx = 0usize;
    let mut nz = 0usize;
    for z in 0..32 {
        for x in 0..32 {
            if x + lag < 32 {
                sx += c(z * 32 + x, z * 32 + x + lag);
                nx += 1;
            }
            if z + lag < 32 {
                sz += c(z * 32 + x, (z + lag) * 32 + x);
                nz += 1;
            }
        }
    }
    let scale = denom / field.len() as f64;
    (
        sx / (nx.max(1) as f64 * scale),
        sz / (nz.max(1) as f64 * scale),
    )
}

/// Mean length of same-sign runs along the x rows. A field with no spatial
/// structure gives 2.0 (a fair coin's geometric mean run).
fn mean_run_length(field: &[f64]) -> f64 {
    let m = field.iter().sum::<f64>() / field.len() as f64;
    let mut runs = 0usize;
    let mut cells = 0usize;
    for z in 0..32 {
        let mut prev: Option<bool> = None;
        for x in 0..32 {
            let s = field[z * 32 + x] >= m;
            if prev != Some(s) {
                runs += 1;
            }
            prev = Some(s);
            cells += 1;
        }
    }
    cells as f64 / runs.max(1) as f64
}

fn plane(pregen: &Pregen, cx: i64, cz: i64) -> Plane {
    let s = sample_chunk(pregen, cx, cz);
    let mut field_old = vec![0.0f64; 32 * 32];
    let mut field_new = vec![0.0f64; 32 * 32];
    let mut touched = vec![false; 32 * 32];
    for v in &s.voxels {
        let i = v.lz * 32 + v.lx;
        field_old[i] += v.total_residual(false);
        field_new[i] += v.total_residual(true);
        touched[i] = true;
    }
    Plane {
        field_old,
        field_new,
        covered: touched.iter().filter(|t| **t).count(),
    }
}

fn ascii_map(field: &[f64]) -> String {
    let m = field.iter().sum::<f64>() / field.len() as f64;
    let sd = {
        let v = field.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / field.len() as f64;
        v.sqrt().max(1e-12)
    };
    let mut out = String::new();
    for z in 0..32 {
        out.push_str("    ");
        for x in 0..32 {
            let t = (field[z * 32 + x] - m) / sd;
            out.push(match t {
                t if t < -1.0 => '#',
                t if t < -0.25 => '+',
                t if t <= 0.25 => '.',
                t if t <= 1.0 => '-',
                _ => ' ',
            });
        }
        out.push('\n');
    }
    out
}

// ------------------------------------------------------------------- report

fn report(c: &Census) {
    println!("\n=== PART 1 — the shared bits ===");
    println!(
        "{} rider decisions in {} voxels ({} of them multi-band), over {} Mixed and {} recorded \
         voxels in {} chunk-columns",
        c.decisions,
        c.rider_voxels,
        c.multi_rider_voxels,
        c.mixed_voxels,
        c.recorded_voxels,
        c.chunks
    );
    println!(
        "\nHow often the pore offset can be READ OUT of the fill offset, per 3-bit window\n\
         (chance is 12.50 %; the allocation's offset is {FILL_OFFSET_BITS} bits wide):"
    );
    println!("      shift   retired      decorrelated");
    for &(shift, o, n) in &c.windows {
        let flag = if o > 0.5 { "  <== the coupling" } else { "" };
        println!(
            "      {shift:>5}   {:>7.2} %   {:>10.2} %{flag}",
            100.0 * o,
            100.0 * n
        );
    }
    let worst_new = c.windows.iter().map(|&(_, _, n)| n).fold(0.0f64, f64::max);
    println!(
        "\n  retired: the pore offset is a DETERMINISTIC function of the fill offset — \
         100 % at shift {RETIRED_SHIFT}."
    );
    println!(
        "  decorrelated: no window predicts it better than {:.2} % (chance 12.50 %).",
        100.0 * worst_new
    );

    println!("\n=== PART 2 — what the coupling did to the arithmetic ===");
    println!(
        "allocation residual vs pore residual      r = {:+.4} (retired)  {:+.4} (decorrelated)",
        c.r_alloc_pore_old, c.r_alloc_pore_new
    );
    println!(
        "  mutual information                        {:.4} bits          {:.4} bits   \
         (estimator floor {:.4})",
        c.mi_alloc_pore_old, c.mi_alloc_pore_new, c.mi_floor
    );
    println!(
        "dither entropy given (band, allocation)   {:.3} bits          {:.3} bits   \
         (max 3.000, {} groups)",
        c.cond_entropy_old, c.cond_entropy_new, c.cond_groups
    );
    println!(
        "  worst single group                        {:.3} bits          {:.3} bits",
        c.cond_entropy_min_old, c.cond_entropy_min_new
    );
    println!(
        "sibling riders IN ONE VOXEL               r = {:+.4}          {:+.4}   \
         ({} pairs)",
        c.r_sibling_old, c.r_sibling_new, c.sibling_pairs
    );
    println!(
        "  Var(voxel total) / Σ Var(rider)           {:.3}×              {:.3}×   \
         (1.0 = errors cancel; N = they add)",
        c.var_ratio_old, c.var_ratio_new
    );

    println!("\n=== PART 4 — what moved ===");
    println!(
        "rider decisions changed: {} of {} ({:.1} %)",
        c.decisions_moved,
        c.decisions,
        100.0 * c.decisions_moved as f64 / c.decisions.max(1) as f64
    );
    println!(
        "voxels changed:          {} of {} rider voxels ({:.1} %) = {:.2} % of the {} recorded \
         voxels sampled",
        c.voxels_moved,
        c.rider_voxels,
        100.0 * c.voxels_moved as f64 / c.rider_voxels.max(1) as f64,
        100.0 * c.voxels_moved as f64 / c.recorded_voxels.max(1) as f64,
        c.recorded_voxels
    );
    println!(
        "net product eighths:     {:+} over {} decisions ({:+.4} eighths/decision) — a \
         REDISTRIBUTION, not a gain",
        c.eighths_moved,
        c.decisions,
        c.eighths_moved as f64 / c.decisions.max(1) as f64
    );
}

fn report_plane(p: &Plane, cx: i64, cz: i64) {
    println!("\n=== PART 3 — is it VISIBLE? ===");
    println!(
        "One chunk-column's contact plane at ({cx}, {cz}): 32×32 voxel columns, ONE record, ONE \
         fill plan.\n{} of 1024 columns carry a pore-rider decision. Banding is spatial \
         structure, so this measures spatial structure.",
        p.covered
    );
    println!("\n      lag    retired (x, z)        decorrelated (x, z)");
    for lag in 1..=4usize {
        let (ox, oz) = autocorr(&p.field_old, lag);
        let (nx, nz) = autocorr(&p.field_new, lag);
        println!("      {lag:>3}    {ox:+.4}, {oz:+.4}     {nx:+.4}, {nz:+.4}");
    }
    println!(
        "\n  mean same-sign run along x:  {:.3} (retired)   {:.3} (decorrelated)   \
         [2.000 = no structure]",
        mean_run_length(&p.field_old),
        mean_run_length(&p.field_new)
    );
    println!("\n  retired field (# low, . mean, ' ' high):");
    print!("{}", ascii_map(&p.field_old));
    println!("  decorrelated field:");
    print!("{}", ascii_map(&p.field_new));
}

fn main() {
    let t0 = std::time::Instant::now();
    let pregen = production_world(Extent::Medium);
    println!(
        "production world seed {SEED:#X}, Extent::Medium, weather-inventory ON — built in {:.1} s",
        t0.elapsed().as_secs_f64()
    );
    let chunks = census_chunks(&pregen, REPORT_CHUNKS);
    assert!(
        !chunks.is_empty(),
        "no banded cell anywhere — nothing carries a pore rider, so this probe is blind"
    );
    let c = measure(&pregen, &chunks);
    report(&c);
    let p = plane(&pregen, chunks[0].0, chunks[0].1);
    report_plane(&p, chunks[0].0, chunks[0].1);

    // Part 5 — why the goldens did not move.
    let off = shipped_default_world(Extent::Medium);
    let c_off = measure(&off, &chunks);
    println!("\n=== PART 5 — the SHIPPED default world, same chunk-columns ===");
    println!(
        "weather_inventory OFF (DeepOverrides::default(), the shipped flip): \
         {} pore-rider decisions in {} Mixed voxels of {} recorded.",
        c_off.decisions, c_off.mixed_voxels, c_off.recorded_voxels
    );
    println!(
        "weather_inventory ON  (this probe's world):                          \
         {} pore-rider decisions in {} Mixed voxels of {} recorded.",
        c.decisions, c.mixed_voxels, c.recorded_voxels
    );
    println!(
        "\n  The weathering front is the only producer of a LOOSE pore rider, and it is off by\n  \
         default. So the world every golden hashes has nothing for this slice to move — which is\n  \
         why `contents_contract`'s fingerprints are byte-identical across it. The world that\n  \
         moves is the one behind --weather-inventory, and Part 4 is its size."
    );
    println!("\ntotal {:.1} s", t0.elapsed().as_secs_f64());
}

/// **The gate** (CLAUDE.md § Gates, journal/0103: an example that can fail
/// belongs in the gate).
///
/// Run at [`Extent::Small`]. Every claim here is a statement about *one voxel's
/// two draws* — a per-decision predicate — so it is scale-free by construction;
/// a bigger extent buys more samples of the same law, not a different law. The
/// production magnitudes stay in the example at [`Extent::Medium`].
///
/// What these tests defend is not a number, it is a **structure**: that the pore
/// rider's offset is drawn independently of the allocation in its voxel and of
/// its sibling bands' offsets. The unit tests in `fill.rs` assert that against
/// the raw draws; these assert it survives contact with the real world's
/// records, which is where the retired code looked innocent.
#[cfg(test)]
mod gate {
    use super::*;

    use std::sync::OnceLock;

    /// Four chunk-columns — 4,096 voxel columns, tens of thousands of
    /// decisions. The claims are per-decision, so this is about having a
    /// population, not about world scale.
    const GATE_CHUNKS: usize = 4;

    /// **[`Extent::Medium`], and `Small` was tried first.** CLAUDE.md § Gates
    /// asks for the smallest extent that still exercises the invariant; here
    /// that is Medium, **measured**: at `Extent::Small` this census finds
    /// **zero** buried `Mixed` voxels carrying a loose pore rider (the small
    /// world's fronts land in the surface partial or in `Single` voxels), and a
    /// gate with no samples asserts nothing. The **invariant** is still
    /// scale-free — every claim below is about one voxel's two draws — Small
    /// simply cannot show it one.
    fn gate_census() -> &'static Census {
        static CENSUS: OnceLock<Census> = OnceLock::new();
        CENSUS.get_or_init(|| {
            let pregen = production_world(Extent::Medium);
            let chunks = census_chunks(&pregen, GATE_CHUNKS);
            assert!(!chunks.is_empty(), "no banded cell anywhere");
            let c = measure(&pregen, &chunks);
            assert!(
                c.decisions >= 500,
                "only {} pore-rider decisions sampled — too thin to bound anything",
                c.decisions
            );
            c
        })
    }

    /// **The fix, on real world data.** No 3-bit window of the offset the
    /// allocation consumed predicts the offset the pore rider used. The retired
    /// code scores 100 % at shift 8 and the same test data proves it, so this is
    /// a before/after inside one run rather than a bare threshold.
    #[test]
    fn the_pore_offset_is_no_longer_readable_out_of_the_fill_offset() {
        let c = gate_census();
        let retired = c
            .windows
            .iter()
            .find(|(s, _, _)| *s == RETIRED_SHIFT)
            .expect("shift 8 is inside the offset width");
        assert!(
            retired.1 > 0.999,
            "the retired formula scores {:.2} % at shift {RETIRED_SHIFT} — this probe has \
             stopped reproducing the code it is the control for",
            100.0 * retired.1
        );
        for &(shift, _, new) in &c.windows {
            assert!(
                (new - 0.125).abs() < 0.04,
                "bits {shift}..{} of the fill offset predict the pore offset {:.2} % of the \
                 time over {} real decisions (chance 12.5 %) — the two draws in a contact \
                 voxel have been re-coupled",
                shift + 3,
                100.0 * new,
                c.decisions,
            );
        }
    }

    /// **Sibling bands in one voxel round independently.** This is the half of
    /// the coupling with teeth: a weathering front puts several thin bands of one
    /// parent in a single contact voxel, and on a shared offset their rounding
    /// errors add rather than cancel, so the voxel's total product swings wider
    /// than the estimator claims.
    #[test]
    fn sibling_riders_in_one_voxel_round_independently() {
        let c = gate_census();
        assert!(
            c.sibling_pairs >= 100,
            "only {} sibling pairs — a multi-band contact voxel has become rare enough \
             that this instrument cannot see the coupling it exists to bound",
            c.sibling_pairs
        );
        // The control, not the claim: the retired formula must still show the
        // coupling on this data, or the before/after is meaningless. The bound
        // is loose on purpose — the exact figure depends on the sampled bands'
        // `cnt·k8 mod 8` mix (0.465 at four chunk-columns, 0.488 at the
        // production forty-eight) — but the *decorrelated* number is ~0.000, so
        // anything in this neighbourhood distinguishes them decisively.
        assert!(
            c.r_sibling_old > 0.3,
            "the retired formula correlates siblings at r = {:+.3} — the control this test \
             measures against has stopped reproducing it",
            c.r_sibling_old
        );
        assert!(
            c.r_sibling_new.abs() < 0.10,
            "sibling riders in one voxel correlate at r = {:+.3} over {} pairs — the event \
             index has fallen out of the pore address",
            c.r_sibling_new,
            c.sibling_pairs,
        );
        assert!(
            (c.var_ratio_new - 1.0).abs() < 0.30,
            "a multi-rider voxel's total residual has variance {:.2}× the sum of its riders' \
             own — independent roundings give 1.0; a coupled set gives the band count",
            c.var_ratio_new,
        );
    }

    /// The rider's expected share is still `cnt · k8 / 8`: the decorrelation
    /// **redistributes** product between voxels and must not create or destroy
    /// it. `fill.rs` asserts this exactly over the eight offsets; this asserts it
    /// on the world's actual `(cnt, k8)` population, where the offsets are drawn
    /// rather than enumerated.
    #[test]
    fn the_decorrelation_moves_product_without_creating_it() {
        let c = gate_census();
        let per = c.eighths_moved as f64 / c.decisions as f64;
        assert!(
            per.abs() < 0.05,
            "the decorrelation shifted {:+} eighths over {} decisions ({per:+.4} each) — a \
             re-addressing must redistribute product, never mint it",
            c.eighths_moved,
            c.decisions,
        );
        assert!(
            c.decisions_moved > 0,
            "not one decision changed — the new draw is not reaching the expression"
        );
    }
}
