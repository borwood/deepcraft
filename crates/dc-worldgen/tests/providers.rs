//! **The provider seam** (journal/0060) — falsifiers for the three converted
//! seams and, above all, the *cross-commit* byte-identity proof.
//!
//! ## Why the goldens are constants and not a self-comparison
//!
//! The obvious test — "build a field twice, once through the old entry and once
//! through the new one, assert equal" — is what `deep_config_plumbing.rs` does
//! for `DeepOverrides`, and it is a good test, but it is **circular** as a proof
//! that the *provider slice* changed nothing: both sides run post-slice code. If
//! the pointer indirection had perturbed an expression, both halves would be
//! perturbed identically and the test would still pass.
//!
//! So the real proof is a **golden captured from pre-slice `main`**
//! (`2434f37`, "journal/0056: the walk that found three things"). This file was
//! written and run *first*, against untouched `main`, and the two hashes below
//! were transcribed from that run. The provider conversions then had to
//! reproduce them.
//!
//! **To re-run this proof independently:**
//!
//! ```text
//! git worktree add ../preslice 2434f37
//! cp crates/dc-worldgen/tests/providers.rs ../preslice/crates/dc-worldgen/tests/
//! # in ../preslice, delete everything below the "The seam itself" banner —
//! # those tests name `Providers`, which does not exist pre-slice.
//! cargo test -p dc-worldgen --release --test providers -- --nocapture
//! ```
//!
//! The printed `surface fingerprint` / `record fingerprint` must equal
//! [`GOLDEN_SURFACE`] / [`GOLDEN_RECORD`]. The fingerprint function deliberately
//! uses only `deeptime`'s pre-slice public API, so it compiles unchanged on
//! either side of the slice.
//!
//! The world hashed is the **production** config at `Extent::Small`
//! (`build_field`, which is `production_config`: biotic on, erodibility on,
//! tectonic history on, full agent roster on) — so all three converted seams are
//! live in the hashed run: `outcrop_at` through `expose`/`periglacial`/`eolian`/
//! `wave`, `wave_energy` through the littoral agent, `parent_p` through the
//! biotic layer's rock-phosphorus pool.

use dc_worldgen::deeptime::{
    self, Aridity, Biofacies, DeepField, DepEnv, EnergyBand, Eolian, build_field,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The seed the goldens were captured at.
const SEED: u64 = 0x0B0A_57EE_0059;

/// FNV-1a-64 over the surface planes of a production `DeepField`, captured from
/// pre-slice `main` (`2434f37`). See the module docs to re-derive.
const GOLDEN_SURFACE: u64 = 0x7B89_68FD_90E0_4062;
/// FNV-1a-64 over the strata record of the same field.
///
/// **Re-baselined 2026-07-22 by journal/0063 — authorized, and note what did
/// NOT move.** The pre-slice value was `0xA53B_D77F_769D_7FF4`
/// (`main` 2434f37). Coal promotion moved from the seam-thickness axis onto the
/// burial-depth axis, which retags units in the record and therefore changes
/// this hash by construction. [`GOLDEN_SURFACE`] is **unchanged**, which is the
/// informative half: promotion runs at finalize, after every erosion epoch, so
/// it can move which units are labelled coal and cannot move a metre of ground.
/// The provider slice's byte-identity proof is intact — it just now sits on top
/// of one authorized behaviour change, exactly as the goldens in
/// `contents_contract.rs` do.
const GOLDEN_RECORD: u64 = 0x7A7B_0528_2017_2C71;

// ---------------------------------------------------------------------------
// A deterministic fingerprint (FNV-1a 64), written by hand so it depends on
// nothing but the bytes: no `#[derive(Hash)]` discriminant encoding, no
// `DefaultHasher` (whose output is explicitly not stable across releases).

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.byte(b);
        }
    }
    fn f64(&mut self, v: f64) {
        // `to_bits` — a bit-for-bit read, so a 1-ULP drift cannot hide.
        self.u64(v.to_bits());
    }
    fn i32(&mut self, v: i32) {
        self.u64(v as u32 as u64);
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
}

// Explicit enum codes rather than derived `Hash`: the derived encoding is a
// rustc implementation detail, and a golden that silently changes with the
// toolchain is not a golden.
fn env_code(e: DepEnv) -> u8 {
    match e {
        DepEnv::Subaerial => 0,
        DepEnv::Subsea => 1,
    }
}
fn aridity_code(a: Aridity) -> u8 {
    match a {
        Aridity::Arid => 0,
        Aridity::Humid => 1,
    }
}
fn energy_code(e: EnergyBand) -> u8 {
    match e {
        EnergyBand::Low => 0,
        EnergyBand::Medium => 1,
        EnergyBand::High => 2,
    }
}
fn biota_code(b: Biofacies) -> u8 {
    match b {
        Biofacies::Mineral => 0,
        Biofacies::Soil => 1,
        Biofacies::Peat => 2,
        Biofacies::Coal => 3,
        Biofacies::Charcoal => 4,
        Biofacies::Retro => 5,
    }
}
fn eolian_code(e: Eolian) -> u8 {
    match e {
        Eolian::None => 0,
        Eolian::Loess => 1,
        Eolian::Dune => 2,
    }
}

/// Every plane the world *keeps* except the record: elevation, regolith, the
/// drainage export, exhumation/crust, and the shape metadata.
fn surface_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.w);
    h.usize(f.wp);
    h.f64(f.cell_m);
    h.usize(f.surf.len());
    for v in &f.surf {
        h.f64(*v);
    }
    h.usize(f.regolith.len());
    for v in &f.regolith {
        h.f64(*v);
    }
    h.usize(f.recv.len());
    for v in &f.recv {
        h.i32(*v);
    }
    for v in &f.area {
        h.f64(*v);
    }
    for v in &f.lake {
        h.byte(u8::from(*v));
    }
    h.usize(f.exhum.len());
    for v in &f.exhum {
        h.f64(*v);
    }
    for v in &f.t_crust {
        h.f64(*v);
    }
    h.usize(f.chapters.len());
    h.0
}

/// The tagged deposition log, unit by unit: every tag axis, thickness bits,
/// unconformity flag, chapter, and the per-cell strip count.
fn record_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.strata.len());
    for s in &f.strata {
        h.usize(s.units.len());
        h.u64(u64::from(s.strips));
        for u in &s.units {
            h.byte(env_code(u.tag.env));
            h.byte(aridity_code(u.tag.aridity));
            h.byte(energy_code(u.tag.energy));
            h.byte(biota_code(u.tag.biota));
            h.byte(eolian_code(u.tag.eolian));
            h.f64(u.thickness_m);
            h.byte(u8::from(u.unconformity));
            h.byte(u.chapter);
        }
    }
    h.0
}

fn production_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    build_field(&pregen.grid, SEED)
}

// ---------------------------------------------------------------------------
// The proof.

/// **The byte-identity acceptance test for the provider slice.** The production
/// world at [`SEED`] must hash to the values captured from pre-slice `main`.
///
/// This is the only test in the suite that can fail *because* a provider seam
/// was introduced: everything else the slice touches is either a compile-time
/// shape or a self-comparison.
#[test]
fn the_production_world_still_hashes_to_the_pre_slice_goldens() {
    let f = production_field();
    let surface = surface_fingerprint(&f);
    let record = record_fingerprint(&f);
    println!("surface fingerprint = {surface:#018X}");
    println!("record  fingerprint = {record:#018X}");
    assert_eq!(
        surface, GOLDEN_SURFACE,
        "the surface planes moved: {surface:#018X} != {GOLDEN_SURFACE:#018X} \
         (pre-slice main 2434f37)"
    );
    assert_eq!(
        record, GOLDEN_RECORD,
        "the strata record moved: {record:#018X} != {GOLDEN_RECORD:#018X} \
         (pre-slice main 2434f37)"
    );
}

/// The fingerprint is a fingerprint: two runs of the same world agree, so a
/// mismatch above means the world moved and not that the hash is unstable.
#[test]
fn the_fingerprint_is_reproducible_within_a_build() {
    let a = production_field();
    let b = production_field();
    assert_eq!(surface_fingerprint(&a), surface_fingerprint(&b));
    assert_eq!(record_fingerprint(&a), record_fingerprint(&b));
}

// ---------------------------------------------------------------------------
// The seam itself. (These reference `Providers`, so they do not compile against
// pre-slice main — delete them for the re-derivation run described above.)

/// `Providers::default()` is the identity set: each slot holds exactly the
/// function that was inlined at the call site before the slice.
#[test]
fn the_default_providers_are_the_identity_functions() {
    use deeptime::providers::Providers;
    let p = Providers::default();
    // "Every slot holds its identity function", stated by address.
    assert!(p.is_identity());
    // `wave_energy`'s identity returns the config rate unchanged, whatever it is.
    for rate in [0.0, 0.05, 1.0, 12.5] {
        let cell = deeptime::providers::WaveCell {
            index: 7,
            gx: 3,
            gy: 4,
            base_rate: rate,
        };
        assert_eq!((p.wave_energy)(cell).to_bits(), rate.to_bits());
    }
    // And `parent_p`'s identity is the uniform initial rock-P pool.
    for index in [0usize, 1, 999] {
        let cell = deeptime::providers::ParentCell {
            index,
            gx: index % 32,
            gy: index / 32,
        };
        assert_eq!((p.parent_p)(cell), 1.0);
    }
}

/// The identity `depth_to_water` leaves the plane empty — the empty-plane +
/// identity-accessor shape, stated through the slot rather than the free
/// function, so a mis-wired `default()` fails here.
#[test]
fn the_identity_water_table_slot_leaves_the_plane_empty() {
    use dc_worldgen::deeptime::providers::{Providers, WaterPass};
    let (precip, r, h) = (vec![0.4f32; 9], vec![25.0f64; 9], vec![2.0f64; 9]);
    let (area, recv, filled) = (vec![12.0f64; 9], vec![-1i32; 9], vec![27.5f64; 9]);
    let mut plane = vec![0.5f32; 9];
    (Providers::default().depth_to_water)(
        WaterPass {
            w: 3,
            epoch: 0,
            precip: &precip,
            r: &r,
            h: &h,
            area: &area,
            recv: &recv,
            filled: &filled,
        },
        &mut plane,
    );
    assert!(plane.is_empty());
}

/// The identity `outcrop_at` is the pre-slice `exposed_litho`: top of the
/// record, or `Basement` when the record is empty.
#[test]
fn the_identity_outcrop_is_the_top_of_the_record() {
    use dc_worldgen::deeptime::providers::identity_outcrop_at;
    use dc_worldgen::deeptime::{DepTag, DepUnit, Litho, litho_of_tag};
    assert_eq!(identity_outcrop_at(None), Litho::Basement);
    let tag = DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low);
    let unit = DepUnit {
        tag,
        thickness_m: 1.0,
        unconformity: false,
        chapter: 0,
    };
    assert_eq!(identity_outcrop_at(Some(&unit)), litho_of_tag(tag));
}

/// Ritual time, on demand: the production deep-time run at `Extent::Medium` —
/// the "generating world history…" ritual the user waits through. `#[ignore]`d
/// because it costs ~17 s and measures the machine, not the code; run it with
/// `cargo test -p dc-worldgen --release --test providers -- --ignored --nocapture`
/// to reproduce the before/after numbers in journal/0060.
#[test]
#[ignore = "17 s wall-clock measurement, not a correctness assertion"]
fn ritual_time_medium() {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t = std::time::Instant::now();
    let f = build_field(&pregen.grid, SEED);
    let dt = t.elapsed();
    println!(
        "RITUAL medium w={} cell_m={} secs={:.3}",
        f.w,
        f.cell_m,
        dt.as_secs_f64()
    );
}

/// The `parent_p` seam is **pass-level**: the plane is materialized once at
/// `BioticSim::new` and read by index, never called per cell inside the epoch
/// loop. The observable consequence is that the plane exists and is `n` long.
#[test]
fn the_parent_phosphorus_plane_is_materialized_once() {
    use dc_worldgen::deeptime::{BioticSim, DeepConfig, build_cells};
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let cfg = DeepConfig {
        seed: SEED,
        cell_m: 4000.0,
        iterations: 1,
        biotic: true,
        ..DeepConfig::default()
    };
    let mut grid = build_cells(&pregen.grid, &cfg);
    let n = grid.w * grid.w;
    let sim = BioticSim::new(&mut grid, &cfg, false);
    assert_eq!(sim.parent_p().len(), n, "the plane is one entry per cell");
    assert!(
        sim.parent_p().iter().all(|v| *v == 1.0),
        "the identity provider gives a uniform 1.0 parent-P field"
    );
}

/// **The `depth_to_water` agreement test** — the strongest statement this slice
/// can make, and the one ARCHITECTURE.md § *A summary is not an authority* asks
/// for: *a test asserting the summary AGREES with the authority*.
///
/// The seam has two paths: the **empty plane**, where `wet_at` computes the
/// three-term proxy inline (what the identity provider selects), and the
/// **materialized plane**, where a provider filled `n` entries at the pass
/// boundary and `wet_at` reads by index. Registering a provider that
/// materializes *exactly the proxy* runs the whole production world down the
/// second path and lands on the pre-slice goldens — so the two branches are the
/// same function, and the pass-level plumbing itself perturbs nothing.
///
/// Without this, the byte-identity proof only ever exercises the empty branch,
/// and the plane path would ship untested until an heir arrived to break it.
#[test]
fn the_materialized_proxy_plane_agrees_with_the_inline_proxy() {
    use dc_worldgen::deeptime::providers::{Providers, WaterPass, identity_wet_index};
    fn proxy_plane(pass: WaterPass<'_>, out: &mut Vec<f32>) {
        let n = pass.w * pass.w;
        out.clear();
        out.reserve(n);
        for i in 0..n {
            out.push(identity_wet_index(
                pass.precip[i],
                pass.r[i] + pass.h[i],
                pass.area[i],
            ));
        }
    }
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let mut cfg = deeptime::production_config(&pregen.grid, SEED);
    cfg.providers = Providers {
        depth_to_water: proxy_plane,
        ..Providers::default()
    };
    assert!(
        !cfg.providers.is_identity(),
        "this must be a registered non-default provider, not the identity slot"
    );
    let f = deeptime::build_field_cfg(&pregen.grid, &cfg);
    assert_eq!(
        surface_fingerprint(&f),
        GOLDEN_SURFACE,
        "the materialized-plane path disagrees with the inline proxy (surface)"
    );
    assert_eq!(
        record_fingerprint(&f),
        GOLDEN_RECORD,
        "the materialized-plane path disagrees with the inline proxy (record)"
    );
}

/// The `depth_to_water` seam is a seam: a world where every cell's water table
/// sits at the surface is not this world. Four consumers threshold `wet` — the
/// peat-former waterlog gate, the decomposition drain factor, the fire dryness
/// term, and the peat-site test — so saturating the plane has to move the
/// organic record.
#[test]
fn a_non_identity_water_table_reaches_the_run() {
    use dc_worldgen::deeptime::providers::{Providers, WaterPass};
    fn drowned(pass: WaterPass<'_>, out: &mut Vec<f32>) {
        out.clear();
        out.resize(pass.w * pass.w, 1.0);
    }
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let base = deeptime::production_config(&pregen.grid, SEED);
    let mut swapped = base;
    swapped.providers = Providers {
        depth_to_water: drowned,
        ..Providers::default()
    };
    let a = deeptime::build_field_cfg(&pregen.grid, &base);
    let b = deeptime::build_field_cfg(&pregen.grid, &swapped);
    assert_ne!(
        record_fingerprint(&a),
        record_fingerprint(&b),
        "a saturated water-table provider did not reach the biotic layer"
    );
}

/// The `depth_to_water` seam is **pass-level, once per epoch**: the provider is
/// called at the top of `BioticSim::step`, not per cell. Counting is not
/// possible through a `fn` pointer without state, so the observable is the
/// plane's *shape* — one entry per cell, refreshed each epoch — plus the fact
/// that the identity leaves it empty and therefore allocates nothing.
#[test]
fn the_identity_water_table_plane_is_empty() {
    use dc_worldgen::deeptime::{BioticSim, DeepConfig, build_cells};
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let cfg = DeepConfig {
        seed: SEED,
        cell_m: 4000.0,
        iterations: 1,
        biotic: true,
        ..DeepConfig::default()
    };
    let mut grid = build_cells(&pregen.grid, &cfg);
    let sim = BioticSim::new(&mut grid, &cfg, false);
    assert!(
        sim.wet().is_empty(),
        "the identity provider must leave the water-table plane unallocated"
    );
}

/// A **non-identity** provider actually reaches the run — the seam is a seam,
/// not decoration. Halving the littoral rate everywhere must move the surface.
///
/// (This is the `full_agents` override falsifier's shape, one layer down.)
#[test]
fn a_non_identity_wave_provider_reaches_the_run() {
    use dc_worldgen::deeptime::providers::{Providers, WaveCell};
    fn half(c: WaveCell) -> f64 {
        c.base_rate * 0.5
    }
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    let base = deeptime::production_config(&pregen.grid, SEED);
    let mut swapped = base;
    swapped.providers = Providers {
        wave_energy: half,
        ..Providers::default()
    };
    let a = deeptime::build_field_cfg(&pregen.grid, &base);
    let b = deeptime::build_field_cfg(&pregen.grid, &swapped);
    assert_ne!(
        surface_fingerprint(&a),
        surface_fingerprint(&b),
        "a halved wave-energy provider did not reach the littoral agent"
    );
}
