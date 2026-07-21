//! Where the terrain roughness goes (S13 — `docs/spikes/S13-results.md`).
//!
//! journal/0040 walked the highest crest of a `--tectonics` world and measured
//! **7.2 m of relief over 1.75 km, identical to the decimetre** at
//! `--amplitude` 80 and 160, while absolute elevation moved by a kilometre.
//! This probe reproduces that number headlessly and then decomposes it.
//!
//! It measures, and never changes, five things:
//!
//! 1. **The walk baseline** — the same seed / extent / flags the client boots
//!    with, transects at the same 250 m stride, including at the walk's own
//!    literal coordinates.
//! 2. **The decay schedule** — the jitter actually injected at each of the 14
//!    refinement levels, empirically (child minus its own parent average), next
//!    to the analytic `rough · AMP_DECAY^level`.
//! 3. **Relief vs. window size** — max−min over nested windows from 100 m to
//!    50 km. The curve is the deliverable: it names the missing wavelengths.
//! 4. **Attribution** — the same curve for the zero-jitter surface (pure
//!    bilinear over the level-5 deep samples) and for the jitter residual, so
//!    deep field and lattice are separated rather than argued about.
//! 5. **The bilinear low-pass** — raw deep-grid cell-to-cell relief against
//!    what survives the lattice's 460.8 m resample of a 460 m field.
//!
//! **Four sites, not one.** The world's highest cell is by definition a local
//! maximum, so its neighbourhood is flat *by construction* — measuring only
//! there would manufacture the conclusion. So every curve is also reported at
//! the steepest land cell, at a median-elevation land cell, and at the exact
//! spot journal/0040 stood on.
//!
//! Nothing here is on a generation path: it calls the read-only
//! `WorldGenerator::surface_elev_m` / `lattice_point` measurement window and
//! `DeepField`'s public fields. Run:
//!
//! `cargo run --release -p dc-worldgen --example roughness_probe`

use dc_worldgen::WorldGenerator;
use dc_worldgen::collapse::{AMP_DECAY, L_DEEP, L_VOXEL};
use dc_worldgen::deeptime::DeepOverrides;
use dc_worldgen::pregen::{
    CELL_VOXELS, Extent, Pregen, Provenance, WorldParams, provenance_roughness,
};

/// The client's `BENCH_SEED` (dc-client/src/bench.rs), which is what every walk
/// so far has been driven through.
const SEED: u64 = 1337;
/// `GenOptions::default().extent` — the client's boot extent.
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;

/// Sampling windows for the relief curve, metres.
const WINDOWS: [f64; 9] = [
    100.0, 250.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 25000.0, 50000.0,
];
/// Samples per axis in each window's grid (nested across windows).
const GRID_N: i64 = 65;

/// journal/0040's summit survey position, metres.
const WALK_XZ_M: (f64, f64) = (10500.0, -15000.0);

struct Site {
    name: &'static str,
    vx: i64,
    vz: i64,
}

/// One sampled column, decomposed. `radius` is the Chebyshev offset from the
/// site in voxels, which is what selects a sample into a window.
struct Sample {
    radius: i64,
    /// The shipped surface, rivers carved.
    final_m: f64,
    /// The same, before river carving (level-14 lattice).
    lattice: f64,
    /// Bilinear over the level-5 lattice: the deep field with no jitter.
    no_jitter: f64,
    /// `lattice - no_jitter`: the refinement pyramid's own contribution.
    jitter: f64,
    /// `DeepField::surface_at_voxel`, sampled directly.
    deep: f64,
}

fn main() {
    println!("=== S13: where the terrain roughness goes ===");
    println!(
        "seed {SEED}, extent {}, N=2 ({VOXEL_M} m voxels), tectonic_history ON",
        EXTENT.label()
    );
    println!("AMP_DECAY = {AMP_DECAY}, L_DEEP = {L_DEEP}, L_VOXEL = {L_VOXEL}\n");

    let a80 = build(80.0);
    let a160 = build(160.0);

    println!("--- world summary ---");
    for (label, p) in [("amp  80", &a80), ("amp 160", &a160)] {
        println!(
            "{label}: deep grid {}x{} @ {:.1} m/cell; pregen {}x{}; max surf {:.1} m",
            p.deep.w,
            p.deep.w,
            p.deep.cell_m,
            p.deep.wp,
            p.deep.wp,
            p.deep.surf.iter().copied().fold(f64::MIN, f64::max),
        );
    }
    println!();

    // Sites are chosen on the amp-80 world and reused verbatim on amp 160, so
    // the A/B is over literally the same ground (journal/0040's discipline).
    let sites = pick_sites(&a80);
    println!("--- sites (all chosen on the amp-80 deep field) ---");
    for s in &sites {
        println!(
            "{:<10} voxel ({:>8}, {:>8}) = ({:>9.0} m, {:>9.0} m)",
            s.name,
            s.vx,
            s.vz,
            s.vx as f64 * VOXEL_M,
            s.vz as f64 * VOXEL_M
        );
    }
    println!();

    section_1_walk_baseline(&a80, &a160, &sites);
    section_2_decay(&a80, &sites);
    section_3_relief_curve(&a80, &a160, &sites);
    section_5_bilinear(&a80, &sites);
}

fn build(thickening_scale: f64) -> Pregen {
    let t0 = std::time::Instant::now();
    let p = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides {
            tectonic_history: Some(true),
            thickening_scale: Some(thickening_scale),
            ..Default::default()
        },
    );
    eprintln!("built amp {thickening_scale} world in {:?}", t0.elapsed());
    p
}

/// The four measurement sites: the world's highest deep cell, the steepest land
/// cell, a median-elevation land cell, and journal/0040's own coordinates.
fn pick_sites(p: &Pregen) -> Vec<Site> {
    let d = &p.deep;
    let w = d.w;
    let mut best_h = (0usize, f64::MIN);
    let mut best_g = (0usize, f64::MIN);
    let mut land: Vec<(usize, f64)> = Vec::new();
    for gy in 0..w {
        for gx in 0..w {
            let i = gy * w + gx;
            let s = d.surf[i];
            if s <= 0.0 {
                continue;
            }
            land.push((i, s));
            if s > best_h.1 {
                best_h = (i, s);
            }
            if gx + 1 < w && gy + 1 < w {
                let g = (s - d.surf[i + 1]).abs().max((s - d.surf[i + w]).abs());
                if g > best_g.1 {
                    best_g = (i, g);
                }
            }
        }
    }
    land.sort_by(|a, b| a.1.total_cmp(&b.1));
    let median = land[land.len() / 2].0;
    let mut sites: Vec<Site> = vec![
        mk("crest", deep_idx_to_voxel(p, best_h.0)),
        mk("steepest", deep_idx_to_voxel(p, best_g.0)),
        mk("median", deep_idx_to_voxel(p, median)),
    ];
    sites.push(Site {
        name: "walk-0040",
        vx: (WALK_XZ_M.0 / VOXEL_M).round() as i64,
        vz: (WALK_XZ_M.1 / VOXEL_M).round() as i64,
    });
    sites
}

fn mk(name: &'static str, v: (i64, i64)) -> Site {
    Site {
        name,
        vx: v.0,
        vz: v.1,
    }
}

/// `DeepField::deep_coords`' own pregen-grid centring term. **Integer** division
/// — `(self.wp / 2) as f64`, not `wp as f64 / 2.0`. Getting this wrong offsets
/// every site by half a pregen cell (7.4 km) and silently measures the wrong
/// ground; the `round-trip:` line in section 5 is the guard that caught it.
fn deep_half(p: &Pregen) -> f64 {
    (p.deep.wp / 2) as f64
}

/// World-voxel centre of a deep cell (the inverse of `DeepField::deep_coords`).
fn deep_idx_to_voxel(p: &Pregen, idx: usize) -> (i64, i64) {
    let w = p.deep.w as f64;
    let wp = p.deep.wp as f64;
    let (gx, gy) = ((idx % p.deep.w) as f64, (idx / p.deep.w) as f64);
    let px = (gx + 0.5) / w * wp - 0.5;
    let py = (gy + 0.5) / w * wp - 0.5;
    let half = deep_half(p);
    (
        ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
    )
}

/// Deep-grid coordinates of a world voxel (rounded to the nearest cell).
fn voxel_to_deep_cell(p: &Pregen, vx: i64, vz: i64) -> (i64, i64) {
    let d = &p.deep;
    let half = deep_half(p);
    let px = vx as f64 / CELL_VOXELS as f64 + half - 0.5;
    let py = vz as f64 / CELL_VOXELS as f64 + half - 0.5;
    (
        ((px + 0.5) / d.wp as f64 * d.w as f64 - 0.5).round() as i64,
        ((py + 0.5) / d.wp as f64 * d.w as f64 - 0.5).round() as i64,
    )
}

// ---------------------------------------------------------------- section 1

/// Reproduce the walk: 8 samples at a 250 m stride, 1.75 km, on both worlds, in
/// four directions (the walk's transect axis is not recorded, so report all of
/// them rather than guess one). Heights are the quantized voxel surface the
/// client's `pose_set { surface: true }` reports, up to a constant offset.
fn section_1_walk_baseline(a80: &Pregen, a160: &Pregen, sites: &[Site]) {
    println!("--- 1. walk baseline: 8 samples @ 250 m stride (1.75 km) ---");
    let stride = (250.0 / VOXEL_M).round() as i64;
    let dirs: [(&str, i64, i64); 4] =
        [("+x", 1, 0), ("-z", 0, -1), ("+x+z", 1, 1), ("+x-z", 1, -1)];
    for s in sites {
        for (label, p) in [("amp  80", a80), ("amp 160", a160)] {
            let mut g = WorldGenerator::new(p);
            for (dl, dx, dz) in dirs {
                let mut hs = Vec::new();
                for k in -2..6i64 {
                    let h = g
                        .coarse_surface(s.vx + dx * k * stride, s.vz + dz * k * stride)
                        .0;
                    hs.push(f64::from(h) * VOXEL_M);
                }
                let relief = hs.iter().copied().fold(f64::MIN, f64::max)
                    - hs.iter().copied().fold(f64::MAX, f64::min);
                let joined = hs
                    .iter()
                    .map(|h| format!("{h:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!(
                    "{:<10} {label} {dl:<5} {joined}  relief {relief:.1} m",
                    s.name
                );
            }
        }
    }
    println!();
}

// ---------------------------------------------------------------- section 2

/// The empirical decay schedule: for each refinement level, the jitter actually
/// added on top of the point's own parent average.
fn section_2_decay(p: &Pregen, sites: &[Site]) {
    println!("--- 2. per-level jitter (empirical vs analytic) ---");
    for s in sites {
        let mut g = WorldGenerator::new(p);
        let rough = g.lattice_point(L_VOXEL, s.vx, s.vz).1;
        println!(
            "site {} — lattice roughness {rough:.1} m (Craton {:.0} / Orogeny {:.0})",
            s.name,
            provenance_roughness(Provenance::Craton),
            provenance_roughness(Provenance::Orogeny)
        );
        println!(
            "{:<6} {:>12} {:>12} {:>12} {:>12} {:>8}",
            "level", "spacing m", "analytic amp", "rms |delta|", "max |delta|", "n"
        );
        for level in 1..=L_VOXEL {
            let spacing_m = (1i64 << (L_VOXEL - level)) as f64 * VOXEL_M;
            let (ci, cj) = (s.vx >> (L_VOXEL - level), s.vz >> (L_VOXEL - level));
            let mut sum2 = 0.0;
            let mut max = 0.0f64;
            let mut n = 0usize;
            for dj in -8..=8i64 {
                for di in -8..=8i64 {
                    let (i, j) = (ci + di, cj + dj);
                    if i & 1 == 0 && j & 1 == 0 {
                        continue; // inherited point: no jitter is drawn there
                    }
                    let child = g.lattice_point(level, i, j).0;
                    let parent = parent_avg(&mut g, level, i, j).0;
                    let d = (child - parent).abs();
                    sum2 += d * d;
                    max = max.max(d);
                    n += 1;
                }
            }
            let rms = (sum2 / n as f64).sqrt();
            let analytic = rough * AMP_DECAY.powi(i32::from(level));
            let tag = if level == L_DEEP {
                "   <- L_DEEP: elevation REPLACED by the deep-time surface"
            } else {
                ""
            };
            println!(
                "{level:<6} {spacing_m:>12.1} {analytic:>12.2} {rms:>12.2} {max:>12.2} {n:>8}{tag}"
            );
        }
        let total: f64 = (1..=L_VOXEL)
            .map(|l| rough * AMP_DECAY.powi(i32::from(l)))
            .sum();
        let surviving: f64 = (L_DEEP + 1..=L_VOXEL)
            .map(|l| rough * AMP_DECAY.powi(i32::from(l)))
            .sum();
        println!(
            "  budget at rough={rough:.0} m: levels 1..14 sum {total:.1} m; levels {}..14 (all that survives L_DEEP) {surviving:.1} m ({:.1} %)\n",
            L_DEEP + 1,
            100.0 * surviving / total
        );
    }
    // Provenance-level budget: what the roughness parameter promises vs. what
    // any of it can physically reach the walked surface as.
    println!("provenance budget (independent of site):");
    for prov in [
        Provenance::Craton,
        Provenance::Orogeny,
        Provenance::Arc,
        Provenance::Rift,
        Provenance::OceanFloor,
    ] {
        let r = provenance_roughness(prov);
        let t: f64 = (1..=L_VOXEL)
            .map(|l| r * AMP_DECAY.powi(i32::from(l)))
            .sum();
        let s: f64 = (L_DEEP + 1..=L_VOXEL)
            .map(|l| r * AMP_DECAY.powi(i32::from(l)))
            .sum();
        // u is uniform in [-1,1] => sd = amp/sqrt(3); levels are independent.
        let sd: f64 = ((L_DEEP + 1..=L_VOXEL)
            .map(|l| {
                let a = r * AMP_DECAY.powi(i32::from(l));
                a * a / 3.0
            })
            .sum::<f64>())
        .sqrt();
        let pname = format!("{prov:?}");
        println!(
            "  {pname:<12} rough {r:>5.0} m: scheduled {t:>7.1} m, surviving worst-case {s:>6.1} m, surviving 1-sigma {sd:>5.2} m"
        );
    }
    println!();
}

fn parent_avg(g: &mut WorldGenerator<'_>, level: u8, i: i64, j: i64) -> (f64, f64) {
    // Mirrors collapse::WorldGenerator::lattice's midpoint parentage exactly.
    let (pi, pj) = (i >> 1, j >> 1);
    match (i & 1 == 0, j & 1 == 0) {
        (true, true) => g.lattice_point(level - 1, pi, pj),
        (false, true) => avg2(
            g.lattice_point(level - 1, pi, pj),
            g.lattice_point(level - 1, pi + 1, pj),
        ),
        (true, false) => avg2(
            g.lattice_point(level - 1, pi, pj),
            g.lattice_point(level - 1, pi, pj + 1),
        ),
        (false, false) => avg2(
            avg2(
                g.lattice_point(level - 1, pi, pj),
                g.lattice_point(level - 1, pi + 1, pj),
            ),
            avg2(
                g.lattice_point(level - 1, pi, pj + 1),
                g.lattice_point(level - 1, pi + 1, pj + 1),
            ),
        ),
    }
}

fn avg2(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

// ---------------------------------------------------------------- section 3/4

/// Relief (max−min) vs. sampling window, with the deep/lattice attribution.
///
/// The point set is the union of every window's grid, so relief is monotone in
/// window size by construction and a large window still sees the fine samples.
fn section_3_relief_curve(a80: &Pregen, a160: &Pregen, sites: &[Site]) {
    println!("--- 3/4. relief vs sampling window, and its attribution ---");
    println!("  final     = the shipped surface (WorldGenerator::surface_elev_m), rivers carved");
    println!("  lattice   = the same surface BEFORE river carving (level-14 lattice)");
    println!("  no-jitter = bilinear over the level-5 lattice (deep field only)");
    println!("  jitter    = lattice - no-jitter (the refinement pyramid's own contribution)");
    println!("  deep-raw  = DeepField::surface_at_voxel sampled directly\n");

    let mut offsets: Vec<(i64, i64)> = Vec::new();
    for &w in &WINDOWS {
        let half_vox = (w / VOXEL_M / 2.0).round() as i64;
        let stride = (2 * half_vox / (GRID_N - 1)).max(1);
        for jz in 0..GRID_N {
            for jx in 0..GRID_N {
                offsets.push((-half_vox + jx * stride, -half_vox + jz * stride));
            }
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    println!(
        "point set: {} distinct offsets per site/world\n",
        offsets.len()
    );

    for s in sites {
        for (label, p) in [("amp  80", a80), ("amp 160", a160)] {
            let mut g = WorldGenerator::new(p);
            let mut rows: Vec<Sample> = Vec::with_capacity(offsets.len());
            for &(dx, dz) in &offsets {
                let (vx, vz) = (s.vx + dx, s.vz + dz);
                let final_m = g.surface_elev_m(vx, vz);
                let raw = g.lattice_point(L_VOXEL, vx, vz).0;
                let nj = no_jitter(&mut g, vx, vz);
                rows.push(Sample {
                    radius: dx.abs().max(dz.abs()),
                    final_m,
                    lattice: raw,
                    no_jitter: nj,
                    jitter: raw - nj,
                    deep: p.deep.surface_at_voxel(vx, vz).unwrap_or(f64::NAN),
                });
            }
            println!("site {} / {label}", s.name);
            println!(
                "{:>10} {:>12} {:>12} {:>12} {:>12} {:>12} {:>8}",
                "window m", "final", "lattice", "no-jitter", "jitter", "deep-raw", "n"
            );
            for &w in &WINDOWS {
                let half_vox = (w / VOXEL_M / 2.0).round() as i64;
                let sel: Vec<&Sample> = rows.iter().filter(|r| r.radius <= half_vox).collect();
                let rel = |f: fn(&Sample) -> f64| -> f64 {
                    let mut lo = f64::MAX;
                    let mut hi = f64::MIN;
                    for r in &sel {
                        let v = f(r);
                        if v.is_finite() {
                            lo = lo.min(v);
                            hi = hi.max(v);
                        }
                    }
                    if hi < lo { f64::NAN } else { hi - lo }
                };
                println!(
                    "{w:>10.0} {:>12.2} {:>12.2} {:>12.2} {:>12.2} {:>12.2} {:>8}",
                    rel(|r| r.final_m),
                    rel(|r| r.lattice),
                    rel(|r| r.no_jitter),
                    rel(|r| r.jitter),
                    rel(|r| r.deep),
                    sel.len()
                );
            }
            println!();
        }
    }
}

/// The surface the lattice would produce with the jitter draws suppressed:
/// repeated midpoint averaging from level 5 with no displacement IS bilinear
/// interpolation of the level-5 values, so this is exact, not an approximation.
fn no_jitter(g: &mut WorldGenerator<'_>, vx: i64, vz: i64) -> f64 {
    let span = 1i64 << (L_VOXEL - L_DEEP); // 512 voxels
    let fi = vx as f64 / span as f64;
    let fj = vz as f64 / span as f64;
    let (i0, j0) = (fi.floor() as i64, fj.floor() as i64);
    let (tx, tz) = (fi - i0 as f64, fj - j0 as f64);
    let e = |g: &mut WorldGenerator<'_>, i: i64, j: i64| g.lattice_point(L_DEEP, i, j).0;
    let a = e(g, i0, j0) * (1.0 - tx) + e(g, i0 + 1, j0) * tx;
    let b = e(g, i0, j0 + 1) * (1.0 - tx) + e(g, i0 + 1, j0 + 1) * tx;
    a * (1.0 - tz) + b * tz
}

// ---------------------------------------------------------------- section 5

/// How much of the deep field's own structure survives the collapse layer's
/// 460.8 m resample of a 460 m grid, and what the raw grid actually holds.
fn section_5_bilinear(p: &Pregen, sites: &[Site]) {
    println!("--- 5. the deep grid itself, and the 460 m bilinear low-pass ---");
    let d = &p.deep;
    for s in sites {
        let (gx, gy) = voxel_to_deep_cell(p, s.vx, s.vz);
        println!(
            "site {} — deep cell ({gx}, {gy}) of {}x{}",
            s.name, d.w, d.w
        );
        // Round-trip check: the cell's own stored value, what the collapse layer
        // samples out of it, and what the walked surface ends up at. If these
        // three disagree the coordinate bridge is broken and nothing else here
        // means anything.
        {
            let mut g = WorldGenerator::new(p);
            let cell_v = d.surf[gy as usize * d.w + gx as usize];
            let samp = d.surface_at_voxel(s.vx, s.vz).unwrap_or(f64::NAN);
            let nj = no_jitter(&mut g, s.vx, s.vz);
            let fin = g.surface_elev_m(s.vx, s.vz);
            println!(
                "  round-trip: surf[cell] {cell_v:.2} | surface_at_voxel {samp:.2} | no-jitter {nj:.2} | final {fin:.2}"
            );
        }
        println!(
            "{:>10} {:>10} {:>10} {:>12} {:>12} {:>12}",
            "radius", "span m", "cells", "relief m", "mean |dS|", "max |dS|"
        );
        for radius in [1i64, 2, 5, 11, 22, 54] {
            let mut lo = f64::MAX;
            let mut hi = f64::MIN;
            let mut sum = 0.0;
            let mut max = 0.0f64;
            let mut n = 0usize;
            let mut cells = 0usize;
            for jy in gy - radius..=gy + radius {
                for jx in gx - radius..=gx + radius {
                    if jx < 0 || jy < 0 || jx >= d.w as i64 || jy >= d.w as i64 {
                        continue;
                    }
                    let sv = d.surf[jy as usize * d.w + jx as usize];
                    lo = lo.min(sv);
                    hi = hi.max(sv);
                    cells += 1;
                    for (ox, oy) in [(1i64, 0i64), (0, 1)] {
                        let (nx, ny) = (jx + ox, jy + oy);
                        if nx >= d.w as i64 || ny >= d.w as i64 {
                            continue;
                        }
                        let t = d.surf[ny as usize * d.w + nx as usize];
                        let dv = (sv - t).abs();
                        sum += dv;
                        max = max.max(dv);
                        n += 1;
                    }
                }
            }
            println!(
                "{radius:>10} {:>10.0} {cells:>10} {:>12.2} {:>12.3} {:>12.2}",
                (2 * radius + 1) as f64 * d.cell_m,
                hi - lo,
                sum / n as f64,
                max
            );
        }
        // The resample: the lattice reads the 460 m field at 460.8 m spacing.
        let mut g = WorldGenerator::new(p);
        let (ci, cj) = (s.vx >> (L_VOXEL - L_DEEP), s.vz >> (L_VOXEL - L_DEEP));
        let mut sum = 0.0;
        let mut max = 0.0f64;
        let mut n = 0usize;
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for dj in -22..=22i64 {
            for di in -22..=22i64 {
                let a = g.lattice_point(L_DEEP, ci + di, cj + dj).0;
                lo = lo.min(a);
                hi = hi.max(a);
                for (ox, oy) in [(1i64, 0i64), (0, 1)] {
                    let b = g.lattice_point(L_DEEP, ci + di + ox, cj + dj + oy).0;
                    let dv = (a - b).abs();
                    sum += dv;
                    max = max.max(dv);
                    n += 1;
                }
            }
        }
        println!(
            "  level-5 lattice (460.8 m spacing) over the same ~20 km: relief {:.2} m, mean step {:.3} m, max step {:.2} m\n",
            hi - lo,
            sum / n as f64,
            max
        );
    }
}
