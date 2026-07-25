//! **The FLOW continuation (a) acceptance probe** (`docs/design/flow.md` § 2.4,
//! journal/0098).
//!
//! Builds the **production world** — seed 1337, `Extent::Medium`, production flags
//! — and reports the numbers the slice exists to produce:
//!
//! 1. **The vertical faces are no longer zero.** Slice 1 recorded them structurally
//!    and left them honestly empty (no vertical term existed in the solve). Count,
//!    magnitude distribution (min / mean / p95 / max), and the fraction of columns
//!    that carry any. Zero here means the head field feeds nothing and the slice
//!    failed — the probe says so in those words.
//! 2. **Is an artesian case REPRESENTABLE, and does one OCCUR?** These are
//!    different questions and the probe answers both separately. Representability
//!    is proven by construction in `deeptime::head`'s tests; here the question is
//!    empirical — does *this* world grow a confining bed over a permeable one and
//!    stand water above the ground? Reported plainly either way: a natural absence,
//!    stated, is worth more than a manufactured presence.
//! 3. **Measured resident cost** of the head plane + the new vertical entries,
//!    against the flow record's own 40.66 MiB and the field's 149.21 MiB baseline
//!    (gen time is free; residency is not).
//!
//! Run: `cargo run --release -p dc-worldgen --example head_field_probe`

use std::time::Instant;

use dc_worldgen::deeptime::{
    DeepConfig, DeepField, FaceKey, build_field_cfg, column_hydro, production_config,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Count the record's vertical (slot↔slot) crossings, and the columns carrying
/// any. **The** number this slice exists to move off zero — shared by `main`
/// (which prints the production report) and the gate test below (journal/0103:
/// `cargo test` builds examples but never runs them, so a claim only reaches the
/// gate through a `#[test]` sharing the instrument's code).
fn vertical_census(f: &DeepField) -> Vertical {
    let cells = f.flux.census().cells;
    let mut v = Vertical::default();
    let mut carrying = vec![false; cells];
    for (i, c) in carrying.iter_mut().enumerate() {
        for e in f.flux.entries_for(i) {
            match e.face {
                FaceKey::Down => {
                    v.down.push(f64::from(e.magnitude));
                    *c = true;
                }
                FaceKey::Up => {
                    v.up.push(f64::from(e.magnitude));
                    *c = true;
                }
                _ => {}
            }
        }
    }
    v.columns = carrying.iter().filter(|b| **b).count();
    v
}

#[derive(Default)]
struct Vertical {
    down: Vec<f64>,
    up: Vec<f64>,
    columns: usize,
}

impl Vertical {
    fn n(&self) -> usize {
        self.down.len() + self.up.len()
    }
}

/// min / mean / p95 / max of a sample (sorted in place).
fn dist(v: &mut [f64]) -> (f64, f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let p95 = v[((v.len() as f64 * 0.95) as usize).min(v.len() - 1)];
    (v[0], mean, p95, v[v.len() - 1])
}

fn main() {
    let t0 = Instant::now();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t_pregen = t0.elapsed();

    let cfg = production_config(&pregen.grid, SEED);
    let t1 = Instant::now();
    let f: DeepField = build_field_cfg(&pregen.grid, &cfg);
    let t_deep = t1.elapsed();

    // The A/B for cost and gen time: the same world with the head field absent.
    let t2 = Instant::now();
    let bare = build_field_cfg(
        &pregen.grid,
        &DeepConfig {
            head_field: false,
            ..cfg
        },
    );
    let t_bare = t2.elapsed();

    let rec = &f.flux;
    let c = rec.census();
    let cells = c.cells;
    let chapters = c.chapters as usize;

    println!("=== FLOW continuation (a) — the head field, production world ===");
    println!(
        "seed {SEED} · Extent::Medium · deep grid {w}×{w} = {cells} cells · {chapters} chapters",
        w = rec.w
    );
    println!(
        "pregen {:.1} s · deep-time run WITH head {:.1} s · WITHOUT {:.1} s  (+{:.1} s)",
        t_pregen.as_secs_f64(),
        t_deep.as_secs_f64(),
        t_bare.as_secs_f64(),
        t_deep.as_secs_f64() - t_bare.as_secs_f64()
    );

    // ---- 1. the vertical faces ---------------------------------------------
    let v = vertical_census(&f);
    let (n_vertical, carrying) = (v.n(), v.columns);
    let (mut down_mags, mut up_mags) = (v.down, v.up);

    println!("\n--- ACCEPTANCE 1: the vertical (slot<->slot) faces ---");
    println!(
        "slice 1 recorded these structurally and left them HONESTLY ZERO \
         (its solve had no vertical term)."
    );
    println!("vertical entries now                          : {n_vertical:>12}");
    println!(
        "  DOWN  (infiltration / recharge)             : {:>12}",
        down_mags.len()
    );
    println!(
        "  UP    (artesian rise / a spring's last step): {:>12}",
        up_mags.len()
    );
    println!(
        "columns carrying any vertical flux            : {carrying:>12}   ({:.3} % of cells)",
        carrying as f64 * 100.0 / cells.max(1) as f64
    );
    let mut all: Vec<f64> = down_mags.iter().chain(up_mags.iter()).copied().collect();
    let (mn, mean, p95, mx) = dist(&mut all);
    println!(
        "magnitude (cell-epoch source units, chapter integral)  min {mn:.6}  \
         mean {mean:.6}  p95 {p95:.6}  max {mx:.6}"
    );
    let (dmn, dmean, dp95, dmx) = dist(&mut down_mags);
    println!("  DOWN  min {dmn:.6}  mean {dmean:.6}  p95 {dp95:.6}  max {dmx:.6}");
    let (umn, umean, up95, umx) = dist(&mut up_mags);
    println!("  UP    min {umn:.6}  mean {umean:.6}  p95 {up95:.6}  max {umx:.6}");
    if n_vertical == 0 {
        println!(
            "\n*** FAIL: the vertical faces are STILL zero. The head field computed \
             a potential that nothing consumed — machinery built beside the hole it \
             was meant to fill. ***"
        );
    } else {
        println!(
            "\nPASS: slice 1's honest zero is now {n_vertical} recorded crossings on \
             {carrying} columns."
        );
    }

    // ---- 2. artesian: representable, and does it occur? --------------------
    println!("\n--- ACCEPTANCE 2: the artesian case ---");
    // Artesian is a statement about LAND: under the sea the potential is pinned at
    // the stand, and standing above the seabed means only "there is water above".
    let sea = dc_worldgen::deeptime::sea_level_at(&cfg, cfg.iterations.saturating_sub(1));
    let mut confined = 0usize;
    let mut land = 0usize;
    let mut lakes = 0usize;
    let mut artesian: Vec<(usize, f64)> = Vec::new();
    let mut water_table_below = 0usize;
    for i in 0..cells {
        let subaerial = f.surf[i] > sea;
        if column_hydro(&f.strata[i]).confined && subaerial {
            confined += 1;
        }
        if !subaerial {
            continue;
        }
        land += 1;
        let (h, s) = (f.head[i], f.surf[i]);
        if h > s + 1e-6 {
            // A lake is pinned at its own water surface, which stands above the
            // ground by construction. That is the lake, not an aquifer.
            if f.lake.get(i).copied().unwrap_or(false) {
                lakes += 1;
            } else {
                artesian.push((i, h - s));
            }
        } else if h < s - 1e-6 {
            water_table_below += 1;
        }
    }
    println!("final sea stand {sea:.2} m · subaerial columns {land} of {cells}");
    println!(
        "subaerial columns CONFINED (>= {:.0} m low-k cap over a permeable bed): {confined:>10}   \
         ({:.3} % of land)",
        dc_worldgen::deeptime::CONFINING_CAP_M,
        confined as f64 * 100.0 / land.max(1) as f64
    );
    println!(
        "subaerial columns with the water table BELOW ground                  : {water_table_below:>10}"
    );
    println!("subaerial columns holding a LAKE (head above ground, and correct)    : {lakes:>10}");
    println!(
        "subaerial columns ARTESIAN (head ABOVE the local ground)             : {:>10}",
        artesian.len()
    );
    if artesian.is_empty() {
        println!(
            "\nNo artesian column occurs naturally on this world. Stated plainly \
             rather than manufactured: the representation ADMITS it — proven by \
             construction in `deeptime::head::tests::\
             head_can_exceed_the_local_surface_which_is_artesian`, where a confined \
             column's head stands metres above its own ground, which `H = y + sat` \
             cannot express at any resolution — but this world's record did not \
             grow the confining geometry that would produce one."
        );
    } else {
        artesian.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("finite"));
        let mut heads: Vec<f64> = artesian.iter().map(|(_, d)| *d).collect();
        let (amn, amean, ap95, amx) = dist(&mut heads);
        println!(
            "excess head above ground (m)  min {amn:.3}  mean {amean:.3}  \
             p95 {ap95:.3}  max {amx:.3}"
        );
        println!("\nthe strongest artesian columns:");
        for (i, excess) in artesian.iter().take(5) {
            let (gx, gy) = (i % rec.w, i / rec.w);
            let hydro = column_hydro(&f.strata[*i]);
            println!(
                "  cell ({gx:>4},{gy:>4}) ~({:>7.1} km,{:>7.1} km)  ground {:>8.1} m  \
                 head {:>8.1} m  EXCESS {excess:>7.2} m  cap {:>6.1} m  k_vert {:.4}",
                gx as f64 * f.cell_m / 1000.0,
                gy as f64 * f.cell_m / 1000.0,
                f.surf[*i],
                f.head[*i],
                hydro.cap_m,
                hydro.k_vertical
            );
        }
        println!(
            "\nEach of those is a column whose water stands ABOVE its own ground \
             because a confining bed holds it there. `H = y + sat` is explicitly \
             unconfined — head IS the elevation — so not one of them is expressible \
             under the proxy this field replaces."
        );
    }

    // ---- 3. the measured cost ----------------------------------------------
    let head_bytes = f.head.len() * std::mem::size_of::<f64>();
    let vertical_bytes = n_vertical * std::mem::size_of::<dc_worldgen::deeptime::FluxEntry>();
    let with = f.resident_bytes();
    let without = bare.resident_bytes();
    println!("\n--- ACCEPTANCE 3: MEASURED RESIDENT COST ---");
    println!(
        "head plane ({} cells x 8 B)   : {:>12} B  ({:.2} MiB)",
        f.head.len(),
        head_bytes,
        mib(head_bytes)
    );
    println!(
        "new vertical entries x 16 B  : {:>12} B  ({:.2} MiB)",
        vertical_bytes,
        mib(vertical_bytes)
    );
    println!(
        "DeepField without the head   : {:>12} B  ({:.2} MiB)",
        without,
        mib(without)
    );
    println!(
        "DeepField with    the head   : {:>12} B  ({:.2} MiB)   = {:.4}x   (+{:.2} MiB)",
        with,
        mib(with),
        with as f64 / without.max(1) as f64,
        mib(with - without)
    );
    println!(
        "flow record total            : {:>12} B  ({:.2} MiB), of which vertical {:.2} MiB",
        rec.resident_bytes(),
        mib(rec.resident_bytes()),
        mib(vertical_bytes)
    );
    println!(
        "FACE SPARSITY (entries / cells x chapters x {FACE}) : {:.4} %",
        c.face_sparsity() * 100.0,
        FACE = dc_worldgen::deeptime::FACE_SLOTS
    );

    // ---- a readable column, so the numbers are not abstract ----------------
    let deepest = (0..cells)
        .filter(|i| f.surf[*i] > sea && f.head[*i] < f.surf[*i] - 1e-6)
        .max_by(|a, b| {
            (f.surf[*a] - f.head[*a])
                .partial_cmp(&(f.surf[*b] - f.head[*b]))
                .expect("finite")
        });
    if let Some(i) = deepest {
        let (gx, gy) = (i % rec.w, i / rec.w);
        let hydro = column_hydro(&f.strata[i]);
        println!("\n--- the deepest water table in the world: cell ({gx}, {gy}) ---");
        println!(
            "ground {:.1} m · head {:.1} m · water table {:.1} m down · \
             {} units · cap {:.1} m · k_vert {:.4} · T {:.2} · confined {}",
            f.surf[i],
            f.head[i],
            f.surf[i] - f.head[i],
            f.strata[i].units.len(),
            hydro.cap_m,
            hydro.k_vertical,
            hydro.transmissivity,
            hydro.confined
        );
        for e in rec.entries_for(i) {
            if e.face.is_vertical() {
                println!(
                    "  chapter {:>2}  {:>4}  magnitude {:>12.6}",
                    e.chapter,
                    e.face.label(),
                    e.magnitude
                );
            }
        }
    }
}

/// **The gate's view of this instrument** (journal/0103).
///
/// journal/0098's whole claim is that the head field turned slice 1's honest
/// structural zero into real vertical flux. The probe printed `*** FAIL ***` for
/// the null and exited 0; a regression that unwired the head-field consumer would
/// have restored the zero and passed the gate.
///
/// Run at [`Extent::Small`]. "Does anything at all cross a slot boundary?" is a
/// question about whether the term is **wired to a consumer**, which is not a
/// function of grid width. The production counts (307,364 crossings, the MiB) are
/// the example's job and stay at [`Extent::Medium`].
#[cfg(test)]
mod gate {
    use super::*;

    use std::sync::OnceLock;

    /// **Built once for the whole binary** — the two tests share the pregen, so
    /// the gate pays for it a single time (sizing the gate is part of the
    /// conversion, CLAUDE.md § Gates).
    fn small_pregen() -> &'static Pregen {
        static PREGEN: OnceLock<Pregen> = OnceLock::new();
        PREGEN.get_or_init(|| {
            Pregen::run(WorldParams {
                seed: SEED,
                extent: Extent::Small,
            })
        })
    }

    fn small_field() -> &'static DeepField {
        static FIELD: OnceLock<DeepField> = OnceLock::new();
        FIELD.get_or_init(|| {
            let p = small_pregen();
            build_field_cfg(&p.grid, &production_config(&p.grid, SEED))
        })
    }

    #[test]
    fn the_head_field_fills_the_vertical_faces_slice_one_left_empty() {
        let f = small_field();
        let v = vertical_census(f);
        let (down, up, carrying) = (v.down.len(), v.up.len(), v.columns);
        assert!(
            down + up > 0,
            "the vertical faces are STILL zero on {} cells — the head field computed a \
             potential that nothing consumed, which is machinery built beside the hole it \
             was meant to fill",
            f.flux.census().cells,
        );
        assert!(
            carrying > 0,
            "vertical entries exist ({down} down, {up} up) but no column is marked as \
             carrying them — the index and the entries disagree"
        );
    }

    /// The A/B the cost report rests on: turning the head field off must
    /// actually remove it. If `head_field: false` still built a head plane, every
    /// "+X MiB for the head" number in the corpus would be measuring nothing.
    #[test]
    fn turning_the_head_field_off_removes_the_vertical_flux() {
        let pregen = small_pregen();
        let cfg = production_config(&pregen.grid, SEED);
        let bare = build_field_cfg(
            &pregen.grid,
            &DeepConfig {
                head_field: false,
                ..cfg
            },
        );
        let bv = vertical_census(&bare);
        let (down, up) = (bv.down.len(), bv.up.len());
        assert_eq!(
            down + up,
            0,
            "head_field: false still recorded {down} down / {up} up vertical entries — \
             the control is not a control, so every measured cost of the head field is wrong"
        );
        // Bound, not a baked figure: a sibling is moving ledger residency, and a
        // probe must not be the thing that fails when memory improves.
        assert!(
            bare.resident_bytes() < small_field().resident_bytes(),
            "the field without the head field ({} B) is not smaller than the field with it \
             ({} B)",
            bare.resident_bytes(),
            small_field().resident_bytes(),
        );
    }
}
