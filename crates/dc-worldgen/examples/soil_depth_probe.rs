//! **The carry-`H` gameplay-impact probe** (journal/0053).
//!
//! Until 2026-07-21 the deep-time sim's regolith plane `H` — the loose cover it
//! spends its whole run weathering off bedrock, blowing around and dropping —
//! was summed into the surface elevation and thrown away, and the collapse tier
//! re-invented soil depth from *present-day precipitation* (`stubs.md` § 3).
//! This probe prints, at named coordinates a walker has actually stood on, the
//! recorded `H`, what the retired precipitation rule said, and what the world
//! now expresses.
//!
//! Sites 1–5 are journal/0049's five guided-tour stations (same world: the
//! client's `BENCH_SEED` = 1337 at `Extent::Medium`), so the numbers here line
//! up with the verdicts the user gave standing in them. Sites 6–7 are the
//! world's own extremes — the thickest and thinnest recorded regolith on land —
//! found by scanning the carried plane.
//!
//! Nothing here is on a generation path: every readout is a pure derivation of
//! the same pregen the world boots from.
//!
//! `cargo run --release -p dc-worldgen --example soil_depth_probe`

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE, CLASS_IGNEOUS_INTRUSIVE,
    GeologySet, vanilla,
};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{Plan, WorldGenerator};

/// The client's `BENCH_SEED` — the world every walk so far has stood in.
const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;

/// journal/0049's five stations, in world metres.
const STATIONS: [(&str, f64, f64); 5] = [
    ("1 deflation basin (13.06 m stripped)", 5993.0, 14732.0),
    ("2 dune field (2.09 m of sand)", 107183.0, 9672.0),
    ("3 loess margin (2.47 m)", 82346.0, 24391.0),
    ("4 periglacial summit (11.8 m frost)", -4586.0, -3206.0),
    ("5 wave coast (0.68 m)", 95224.0, 22091.0),
];

/// **The retired rule**: soil band depth in voxels from present-day precip
/// (`collapse.rs::column`, pre-0053). Note the floor of 1 — under this rule no
/// column in the world could ever be bare.
fn old_soil_voxels(precip: f64) -> u32 {
    if precip > 0.5 {
        3
    } else if precip > 0.2 {
        2
    } else {
        1
    }
}

/// **The retired veneer budget** (`geology.rs::clastic_pass`, pre-0053), with
/// the fluvial term left out so the two sides compare like for like — it is the
/// same additive term in both.
fn old_veneer_voxels(precip: f64) -> u32 {
    (1.0 + precip * 2.5).round().clamp(1.0, 8.0) as u32
}

/// **The rule now**: round `H` to the nearest whole voxel, clamp `0..=8`.
fn new_voxels(h_m: f64) -> u32 {
    (h_m / VOXEL_M).round().clamp(0.0, 8.0) as u32
}

fn main() {
    println!("=== carry-H soil-depth probe (journal/0053) ===");
    println!(
        "seed {SEED}, extent {}, N=2 ({VOXEL_M} m voxels)\n",
        EXTENT.label()
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });
    let set = vanilla();
    let deep_w = pregen.deep.w;
    let cell_m = pregen.deep.cell_m;
    println!(
        "deep grid {deep_w}x{deep_w} @ {cell_m:.0} m/cell; regolith plane carried: {} cells, {} bytes",
        pregen.deep.regolith.len(),
        pregen.deep.regolith.len() * std::mem::size_of::<f64>()
    );
    let mb = |b: usize| b as f64 / (1024.0 * 1024.0);
    println!(
        "DeepField resident: {:.2} MB total, of which the H plane is {:.2} MB\n",
        mb(pregen.deep.resident_bytes()),
        mb(pregen.deep.regolith.len() * std::mem::size_of::<f64>())
    );

    // Whole-world distribution of the carried plane over subaerial cells, and
    // how the two rules quantize it.
    distribution(&pregen);

    // The world's own extremes on land: thickest and thinnest recorded regolith.
    let (thick, thin) = extremes(&pregen);
    let mut sites: Vec<(String, i64, i64)> = STATIONS
        .iter()
        .map(|(n, mx, mz)| {
            (
                (*n).to_string(),
                (mx / VOXEL_M).round() as i64,
                (mz / VOXEL_M).round() as i64,
            )
        })
        .collect();
    for (label, idx) in [
        ("6 THICKEST regolith on land", thick),
        ("7 THINNEST regolith on land", thin),
    ] {
        let (vx, vz) = idx_to_voxel(deep_w, pregen.deep.wp, idx);
        sites.push((label.to_string(), vx, vz));
    }

    let mut g = WorldGenerator::with_geology(&pregen, set.clone());
    for (name, vx, vz) in &sites {
        let h_m = pregen.deep.regolith_at_voxel(*vx, *vz);
        let rec: Vec<f64> = pregen
            .deep
            .record_at_voxel(*vx, *vz)
            .map(|s| s.units.iter().map(|u| u.thickness_m()).collect())
            .unwrap_or_default();
        site(&mut g, &set, name, *vx, *vz, h_m, &rec);
    }
}

/// How much of the world the change actually moves: the carried `H` histogram
/// over subaerial deep cells, and the two rules' voxel verdicts side by side.
fn distribution(pregen: &Pregen) {
    let f = &pregen.deep;
    let (mut land, mut bare, mut sum, mut max) = (0usize, 0usize, 0.0f64, 0.0f64);
    let mut hist = [0usize; 9];
    for i in 0..f.w * f.w {
        if f.surf[i] <= 0.0 {
            continue;
        }
        land += 1;
        let h = f.regolith[i];
        sum += h;
        max = max.max(h);
        let v = new_voxels(h) as usize;
        hist[v.min(8)] += 1;
        if v == 0 {
            bare += 1;
        }
    }
    println!("--- carried H over {land} subaerial deep cells ---");
    println!("  mean {:.2} m, max {max:.2} m", sum / land as f64);
    println!("  quantized to voxels: {hist:?}  (index = voxel count)");

    // How much of H the strata record accounts for. In METRES the two are
    // identical (the recorder logs every metre the sim deposits) — printed to
    // show that. In VOXELS they are not: the 0.9 m sieve drops every bed thinner
    // than half a voxel, and that residue is what the veneer amalgamates.
    let (mut rsum, mut ressum, mut resbare, mut rescap) = (0.0f64, 0.0f64, 0usize, 0usize);
    let mut reshist = [0usize; 9];
    for i in 0..f.w * f.w {
        if f.surf[i] <= 0.0 {
            continue;
        }
        let rec: f64 = f.strata[i].units.iter().map(|u| u.thickness_m()).sum();
        rsum += rec;
        // Voxels the record actually EXPRESSES: each unit rounded to whole
        // voxels, sub-half-voxel beds dropped by the sieve (deposit_deep_history).
        let rec_vox: f64 = f.strata[i]
            .units
            .iter()
            .map(|u| (u.thickness_m() / VOXEL_M).round())
            .filter(|t| *t >= 1.0)
            .sum();
        let res = ((f.regolith[i] / VOXEL_M).round() - rec_vox).max(0.0) * VOXEL_M;
        ressum += res;
        let v = new_voxels(res) as usize;
        reshist[v.min(8)] += 1;
        if v == 0 {
            resbare += 1;
        }
        if v >= 8 {
            rescap += 1;
        }
    }
    println!(
        "  recorded deposition Σ: mean {:.2} m; residue (H − Σrecord): mean {:.2} m",
        rsum / land as f64,
        ressum / land as f64
    );
    println!("  residue quantized    : {reshist:?}");
    println!(
        "  residue BARE {resbare} ({:.1}%), CAPPED at 8 {rescap} ({:.1}%)",
        100.0 * resbare as f64 / land as f64,
        100.0 * rescap as f64 / land as f64
    );
    println!(
        "  BARE (0 voxels of soil): {bare} cells = {:.1}% of land — under the retired \n\
         \x20 precipitation rule this number was structurally 0 (its floor was 1 voxel).\n",
        100.0 * bare as f64 / land as f64
    );

    // ---- THE SIEVE, before and after (journal/0055) ------------------------
    //
    // "Sieve loss" = the fraction of recorded metres that never reaches a voxel.
    //
    // BEFORE: `Σ round(tᵢ / 0.9)` — each unit rounded on its own, sub-half-voxel
    // beds dropped outright. AFTER: `round(Σ tᵢ / 0.9)` — one quantization for
    // the whole column, so the only loss left is the ≤ half-voxel rounding of
    // the column's own total, and it is signed (columns round up as often as
    // down) rather than a one-way deletion.
    let (mut rec_m, mut old_m, mut new_m, mut new_abs) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut old_zero = 0usize;
    for i in 0..f.w * f.w {
        if f.surf[i] <= 0.0 {
            continue;
        }
        let units = &f.strata[i].units;
        let rec: f64 = units.iter().map(|u| u.thickness_m()).sum();
        let old: f64 = units
            .iter()
            .map(|u| (u.thickness_m() / VOXEL_M).round())
            .filter(|t| *t >= 1.0)
            .sum::<f64>()
            * VOXEL_M;
        let new = (rec / VOXEL_M).round() * VOXEL_M;
        rec_m += rec;
        old_m += old;
        new_m += new;
        new_abs += (new - rec).abs();
        if old <= 0.0 && rec > 0.0 {
            old_zero += 1;
        }
    }
    println!("--- THE SIEVE: recorded metres that reach voxels ---");
    println!(
        "  recorded          : {:.3} m/cell mean",
        rec_m / land as f64
    );
    println!(
        "  OLD Σround(tᵢ/0.9): {:.3} m/cell — {:.1}% of the pile LOST",
        old_m / land as f64,
        100.0 * (1.0 - old_m / rec_m)
    );
    println!(
        "  NEW round(Σtᵢ/0.9): {:.3} m/cell — {:+.1}% net, {:.3} m/cell mean |error|",
        new_m / land as f64,
        100.0 * (new_m / rec_m - 1.0),
        new_abs / land as f64
    );
    println!(
        "  cells expressing NOTHING under the old rule: {old_zero} ({:.1}% of land); \
         under the new rule a cell expresses nothing only when its whole column \
         rounds under half a voxel.\n",
        100.0 * old_zero as f64 / land as f64
    );
}

/// Thickest / thinnest recorded regolith among subaerial deep cells (interior
/// only — the eolian march piles its remainder on the downwind land edge).
fn extremes(pregen: &Pregen) -> (usize, usize) {
    let f = &pregen.deep;
    let w = f.w as i64;
    let (mut hi, mut lo) = ((0usize, -1.0f64), (0usize, f64::MAX));
    for i in 0..f.w * f.w {
        let (gx, gy) = (i as i64 % w, i as i64 / w);
        if gx < 8 || gy < 8 || gx >= w - 8 || gy >= w - 8 || f.surf[i] <= 0.0 {
            continue;
        }
        if f.regolith[i] > hi.1 {
            hi = (i, f.regolith[i]);
        }
        if f.regolith[i] < lo.1 {
            lo = (i, f.regolith[i]);
        }
    }
    (hi.0, lo.0)
}

/// World-voxel centre of a deep cell — the inverse of `DeepField::deep_coords`
/// (note the **integer** `wp / 2` centring: journal/0043's half-cell fix).
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

/// One site: the recorded cause, both rules' verdicts, and the strata the
/// generator actually laid down there.
fn site(
    g: &mut WorldGenerator<'_>,
    set: &GeologySet,
    name: &str,
    vx: i64,
    vz: i64,
    h_m: Option<f64>,
    rec_units: &[f64],
) {
    let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
    let (_, precip) = g.climate_probe(vx, vz);
    let col = g.column_record(cx, cz);
    let surf_vox = col.heights[16 * 32 + 16];

    println!("--- SITE {name} ---");
    println!(
        "  world ({:.0} m, {:.0} m) | voxel ({vx}, {vz}) | chunk col ({cx}, {cz}) | surface y {surf_vox}",
        vx as f64 * VOXEL_M,
        vz as f64 * VOXEL_M
    );
    // What the record expresses as whole voxels, and the un-expressible residue
    // the veneer now amalgamates (the actual new budget, no-river term).
    let rec_vox: u32 = rec_units
        .iter()
        .map(|t| (t / VOXEL_M).round())
        .filter(|t| *t >= 1.0)
        .sum::<f64>() as u32;
    let veneer = h_m.map_or_else(
        || old_veneer_voxels(precip),
        |h| {
            ((h / VOXEL_M).round() as u32)
                .saturating_sub(rec_vox)
                .min(8)
        },
    );

    match h_m {
        Some(h) => println!(
            "  recorded regolith H : {h:.3} m  ({:.2} voxels) = Σ of {} recorded units",
            h / VOXEL_M,
            rec_units.len()
        ),
        None => println!("  recorded regolith H : NONE (border wilds — genesis fallback)"),
    }
    println!("  record expresses    : {rec_vox} whole voxels; residue amalgamated by the veneer");
    println!("  present-day precip  : {precip:.3}");
    println!(
        "  soil depth  OLD → NEW : {} → {} voxels   ({:.2} m → {:.2} m)",
        old_soil_voxels(precip),
        h_m.map_or_else(|| old_soil_voxels(precip), new_voxels),
        f64::from(old_soil_voxels(precip)) * VOXEL_M,
        f64::from(h_m.map_or_else(|| old_soil_voxels(precip), new_voxels)) * VOXEL_M,
    );
    println!(
        "  veneer budget: retired {} → carry-H {} → distribution-first {:.2} voxels",
        old_veneer_voxels(precip),
        veneer,
        // journal/0055: the record now expresses ALL of H, so the residue the
        // veneer exists to amalgamate is zero. Nothing was deleted to make that
        // happen — the subtraction simply has nothing left to subtract.
        h_m.map_or(f64::from(old_veneer_voxels(precip)), |h| ((h - rec_units
            .iter()
            .sum::<f64>())
            / VOXEL_M)
            .max(0.0)),
    );

    // What is actually in the ground: the topmost contiguous clastic band (the
    // loose cover a player digs) and the record beneath it.
    //
    // Note the readout changed meaning with journal/0055: the retired rule
    // amalgamated everything unresolvable into ONE surficial band, so "the
    // topmost contiguous clastic run" was the diggable pile. The record is
    // interbedded, so that run is now often one thin bed. What a player digs is
    // the whole recorded column above basement, and its clastic share.
    // P11 slice 3: the record is per column — read the chunk-centre column's
    // SubCell (the station voxel is addressed by chunk here).
    let Some(sub) = col.centre_record() else {
        println!("  (no record realized at the centre column — fallback ground)");
        return;
    };
    let mut loose_m = 0.0f64;
    let mut column_m = 0.0f64;
    for e in sub.strata().events.iter().rev() {
        let c = &set.member(e.member).class;
        if c == CLASS_IGNEOUS_INTRUSIVE || c == CLASS_IGNEOUS_EXTRUSIVE {
            break;
        }
        column_m += f64::from(e.thickness_m);
        if c == CLASS_CLASTIC_FINE || c == CLASS_CLASTIC_COARSE {
            loose_m += f64::from(e.thickness_m);
        }
    }
    println!(
        "  GENERATED sediment    : {:.0} voxels ({column_m:.2} m) above basement, of which \
         {loose_m:.2} m loose clastic",
        (column_m / VOXEL_M).round()
    );
    // Distribution-first expression (journal/0055): how the record slices into
    // voxel spans, and how many of those spans straddle a contact.
    let fill = sub.fill();
    let mixed = (1..=fill.depth_count() as u32)
        .filter(|d| matches!(fill.plan(*d), Some(Plan::Mixed(_))))
        .count();
    println!(
        "  EXPRESSED             : {} events, {} voxel spans, {mixed} of them mixed",
        sub.strata().events.len(),
        fill.depth_count()
    );
    // What the world is skinned with here (journal/0055, journal/0074): the
    // surface voxel is the record's top span (`plan(1)`) expressed as a partial,
    // through the same fill machinery as every buried voxel.
    let i = 16 * 32 + 16;
    let top = fill.plan(1);
    println!(
        "  SURFACE               : {:?}, filled {} of 8 eighths{}",
        col.surface[i],
        if top.is_some() {
            col.surface_eighths[i]
        } else {
            0
        },
        match top {
            None => " (fallback — no record to skin it with)".to_string(),
            Some(Plan::Single(k)) =>
                format!(" of {}", set.member(sub.strata().events[*k].member).id),
            Some(Plan::Mixed(_)) => " (mixed top span)".to_string(),
        }
    );
    let tail: Vec<String> = sub
        .strata()
        .events
        .iter()
        .rev()
        .take(4)
        .map(|e| {
            let id = &set.member(e.member).id;
            format!(
                "{}@{:.2}m",
                id.rsplit('/').next().unwrap_or(id),
                e.thickness_m
            )
        })
        .collect();
    println!("  top-down record       : {}\n", tail.join(" / "));
}
