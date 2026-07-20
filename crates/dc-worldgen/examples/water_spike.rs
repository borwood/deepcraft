//! S11 water spike measurement harness.
//!
//! Two questions (docs/design/water.md § "The hypothesis S11 tests"):
//!
//! - **Q1** is the bound-water relaxation genuinely LOCAL? Measures the halo of
//!   a drain perturbation against permeability contrast, the way S9 measured
//!   erosion's decay length.
//! - **Q2** does the free-water body graph stay sparse under adversarial
//!   digging? Runs the seven scenarios, the sparsity growth curve, the event
//!   storm, the determinism proof, and the persistence budget.
//!
//! Run: `cargo run --release -p dc-worldgen --example water_spike`
//! Args: `--q1` / `--q2` to run one half; `--quick` for smaller grids.
//!
//! Recorded numbers live in `docs/spikes/S11-results.md`.
//!
//! `Instant` wraps runs only — never inside the model (the crate rule).

use std::collections::HashMap;
use std::time::Instant;

use dc_worldgen::water::{
    BodyGraph, BodyId, BodyKind, ConnIndex, ROCK_VOID, RockProps, SatField, VoxWorld, WaterEvent,
    bind,
};

// ---------------------------------------------------------------- Q1: bound water

/// Rock ids for the Q1 column. 0 is reserved for open space.
const R_BEDROCK: u8 = 1;
const R_AQUITARD: u8 = 2;
const R_AQUIFER: u8 = 3;
const R_SOIL: u8 = 4;

struct Q1Case {
    k_aquifer: f32,
    k_aquitard: f32,
}

/// The Q1 column. The bottom layer is regional deep groundwater (a sink);
/// above it a leaky aquitard whose permeability is the experiment's variable;
/// above that the aquifer at a FIXED permeability, so convergence time is the
/// same in every case; soil on top takes the recharge.
fn build_q1(n: usize, dy: usize, case: &Q1Case) -> SatField {
    let props = vec![
        RockProps {
            porosity: 1.0,
            perm: 0.0,
        }, // ROCK_VOID
        RockProps {
            // Pore space per step is what a drained voxel can ACCEPT per step,
            // so this is the deep sink's throughput. It must sit well above the
            // recharge or the whole column balances on a knife edge and the
            // measurement reads the knife edge instead of the halo (the first
            // version used 0.02 against a recharge of 0.02, and the
            // low-contrast profiles came back flat).
            porosity: 0.20,
            perm: 1.0,
        }, // bedrock: the deep sink
        RockProps {
            porosity: 0.10,
            perm: case.k_aquitard,
        }, // aquitard: the variable
        RockProps {
            porosity: 0.30,
            perm: case.k_aquifer,
        }, // aquifer
        RockProps {
            porosity: 0.35,
            perm: case.k_aquifer,
        }, // soil
    ];
    let mut f = SatField::new(n, dy, n, props);
    for y in 0..dy {
        let r = match y {
            y if y < dy / 6 => R_BEDROCK,
            y if y < dy / 3 => R_AQUITARD,
            y if y < (dy * 5) / 6 => R_AQUIFER,
            _ => R_SOIL,
        };
        for z in 0..n {
            for x in 0..n {
                let i = f.idx(x, y, z);
                f.rock[i] = r;
                // Start wet and let the drains pull the table down: converges
                // in hundreds of steps instead of tens of thousands.
                if r == R_AQUIFER || r == R_AQUITARD {
                    f.sat[i] = 1.0;
                }
                if r == R_BEDROCK {
                    f.drain[i] = true;
                }
            }
        }
    }
    f.recharge = 0.02;
    // Explicit-diffusion stability: the per-step lateral transfer must stay
    // under phi/(2*ndim) or the field oscillates instead of relaxing, and the
    // oscillation floor swamps any halo measurement. phi = 0.30, 4 lateral
    // neighbours => k*lateral_c <= 0.0375. (First version of this harness used
    // 1.0 and measured pure noise.)
    f.lateral_c = 0.03;
    f.vertical_c = 1.0;
    f
}

/// Saturated-thickness per column: the continuous field the water table reads.
fn storage(f: &SatField) -> Vec<f32> {
    let mut out = vec![0.0f32; f.dx * f.dz];
    for y in 0..f.dy {
        for z in 0..f.dz {
            for x in 0..f.dx {
                let i = f.idx(x, y, z);
                if f.rock[i] == ROCK_VOID {
                    continue;
                }
                out[z * f.dx + x] += f.sat[i];
            }
        }
    }
    out
}

/// Discrete water table per column, in voxels (0 where dry).
fn table(f: &SatField) -> Vec<f32> {
    f.water_table()
        .into_iter()
        .map(|o| o.unwrap_or(0.0))
        .collect()
}

/// Largest ring at or beyond which the disturbance stays under `thresh`
/// (i.e. the halo: the deepest ring that still exceeds it, +1).
fn halo(rings: &[f32], thresh: f32) -> usize {
    let mut h = 0;
    for (d, &v) in rings.iter().enumerate() {
        if v >= thresh {
            h = d + 1;
        }
    }
    h
}

/// Place the drainage network: a lattice of seepage columns through the
/// aquifer, spacing `l`. This is the boundary condition a real water table
/// actually has — streams and coastlines every few hundred metres — and its
/// absence is what made the first version of this measurement degenerate.
fn add_drain_lattice(f: &mut SatField, l: usize, top: usize) {
    for z in 0..f.dz {
        for x in 0..f.dx {
            if x % l != 0 || z % l != 0 {
                continue;
            }
            for y in 0..top {
                let i = f.idx(x, y, z);
                f.drain[i] = true;
            }
        }
    }
}

/// Max |a-b| per Chebyshev ring around `(cx, cz)`.
fn ring_max_at(a: &[f32], b: &[f32], n: usize, cx: usize, cz: usize, rmax: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; rmax + 1];
    for z in 0..n {
        for x in 0..n {
            let d = (x as i64 - cx as i64)
                .abs()
                .max((z as i64 - cz as i64).abs()) as usize;
            if d > rmax {
                continue;
            }
            let e = (a[z * n + x] - b[z * n + x]).abs();
            if e > out[d] {
                out[d] = e;
            }
        }
    }
    out
}

fn clone_field(f: &SatField) -> SatField {
    let mut g = SatField::new(f.dx, f.dy, f.dz, f.props.clone());
    g.rock.copy_from_slice(&f.rock);
    g.sat.copy_from_slice(&f.sat);
    g.drain.copy_from_slice(&f.drain);
    g.recharge = f.recharge;
    g.lateral_c = f.lateral_c;
    g.vertical_c = f.vertical_c;
    g
}

fn q1(n: usize, dy: usize, steady: u32, budget: u32) {
    println!("\n== Q1 - is the bound-water relaxation LOCAL? ==");
    println!(
        "grid {n}x{dy}x{n} = {} cells; equilibrate {steady} steps, post-edit budget {budget} steps",
        n * dy * n
    );
    println!("boundary conditions: a drainage lattice of seepage columns at spacing L (streams,");
    println!("coast) plus a leaky aquitard over a deep sink. Aquifer k is FIXED at 1.0; the");
    println!("contrast variable is the aquitard beneath it (the loose-vs-packed soil contrast).");
    println!("perturbation: ONE new seepage column (a dug shaft) at the centre of a lattice cell.");
    println!("control and treated advance the SAME number of steps before differencing.");
    println!(
        "\n{:<6} {:>11} {:>10} {:>9} {:>9} {:>9} {:>6} {:>8} {:>9}",
        "L", "k_aquitard", "contrast", "halo_bud", "halo_ss", "vis_halo", "L/2", "drift", "eq_s"
    );

    let contrasts = [
        ("uniform", 1.0f32),
        ("leaky", 0.1),
        ("100", 0.01),
        ("1000", 0.001),
        ("10000", 0.0001),
    ];
    let mut profiles: Vec<(String, Vec<f32>)> = Vec::new();

    for l in [8usize, 16, 32] {
        for (label, k_at) in contrasts {
            let case = Q1Case {
                k_aquifer: 1.0,
                k_aquitard: k_at,
            };
            let aq_top = (dy * 5) / 6;
            let t0 = Instant::now();
            let mut f = build_q1(n, dy, &case);
            add_drain_lattice(&mut f, l, aq_top);
            f.relax(steady);
            let secs = t0.elapsed().as_secs_f64();
            let ref_store = storage(&f);
            let mut probe = clone_field(&f);
            probe.relax(20);
            let drift = storage(&probe)
                .iter()
                .zip(ref_store.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);

            let cx = (n / 2 / l) * l + l / 2;
            let cz = cx;
            let rmax = cx.min(n - 1 - cx);

            let mut control = clone_field(&f);
            let mut treated = clone_field(&f);
            for y in 0..aq_top {
                let i = treated.idx(cx, y, cz);
                treated.drain[i] = true;
            }

            control.relax(budget);
            treated.relax(budget);
            let h_bud = halo(
                &ring_max_at(&storage(&treated), &storage(&control), n, cx, cz, rmax),
                0.01,
            );
            control.relax(steady);
            treated.relax(steady);
            let ss_rings = ring_max_at(&storage(&treated), &storage(&control), n, cx, cz, rmax);
            let h_ss = halo(&ss_rings, 0.01);
            let h_vis = halo(
                &ring_max_at(&table(&treated), &table(&control), n, cx, cz, rmax),
                1.0,
            );

            println!(
                "{:<6} {:>11.4} {:>10.0} {:>9} {:>9} {:>9} {:>6} {:>8.4} {:>9.2}",
                l,
                k_at,
                1.0 / k_at,
                h_bud,
                h_ss,
                h_vis,
                l / 2,
                drift,
                secs
            );
            if l == 32 {
                profiles.push((format!("L=32 contrast {label}"), ss_rings));
            }
        }
    }
    for (name, rings) in &profiles {
        let prof: Vec<String> = rings
            .iter()
            .enumerate()
            .take(34)
            .filter(|(d, _)| d % 2 == 0)
            .map(|(d, v)| format!("d{d}={v:.4}"))
            .collect();
        println!("  storage-delta profile [{name}]: {}", prof.join(" "));
    }

    // Secondary perturbation: a saturation spike rather than a drain.
    let case = Q1Case {
        k_aquifer: 1.0,
        k_aquitard: 0.01,
    };
    let l = 32usize;
    let aq_top = (dy * 5) / 6;
    let mut f = build_q1(n, dy, &case);
    add_drain_lattice(&mut f, l, aq_top);
    f.relax(steady);
    let cx = (n / 2 / l) * l + l / 2;
    let rmax = cx.min(n - 1 - cx);
    let mut control = clone_field(&f);
    let mut treated = clone_field(&f);
    for y in 0..dy {
        let i = treated.idx(cx, y, cx);
        treated.sat[i] = 1.0;
    }
    control.relax(steady);
    treated.relax(steady);
    println!(
        "saturation-spike perturbation (L=32, contrast 100): halo {} cells @ 0.01",
        halo(
            &ring_max_at(&storage(&treated), &storage(&control), n, cx, cx, rmax),
            0.01
        )
    );

    // Conservation: with no recharge and no drains, bound water is exactly
    // conserved through the relaxation.
    let mut h = build_q1(48, dy, &case);
    h.recharge = 0.0;
    for d in h.drain.iter_mut() {
        *d = false;
    }
    for i in 0..h.sat.len() {
        h.sat[i] = if (i % 7) as f32 > 3.0 { 0.8 } else { 0.1 };
    }
    let before = h.total_water();
    h.relax(64);
    let after = h.total_water() + h.crossed_to_free;
    println!(
        "conservation (no recharge, no drains, 64 steps): before {before:.6}, after {after:.6}, drift {:.3e}",
        (after - before).abs() / before.max(1.0)
    );

    // Determinism of the relaxation itself.
    let mut a = build_q1(48, dy, &case);
    add_drain_lattice(&mut a, 16, aq_top);
    a.relax(60);
    let mut b = build_q1(48, dy, &case);
    add_drain_lattice(&mut b, 16, aq_top);
    b.relax(60);
    println!(
        "relaxation double-run byte-identical: {}",
        a.sat
            .iter()
            .map(|v| v.to_bits())
            .eq(b.sat.iter().map(|v| v.to_bits()))
    );
}

// ---------------------------------------------------------------- Q2 plumbing

struct World {
    vox: VoxWorld,
    conn: ConnIndex,
    graph: BodyGraph,
}

impl World {
    fn solid(dx: usize, dy: usize, dz: usize) -> Self {
        let vox = VoxWorld::solid(dx, dy, dz);
        let conn = ConnIndex::build(&vox);
        Self {
            vox,
            conn,
            graph: BodyGraph::new(),
        }
    }

    fn dig(&mut self, lo: [i64; 3], hi: [i64; 3]) -> (u64, u64) {
        self.vox.dig(lo, hi);
        self.conn.apply_edit(&self.vox, lo, hi)
    }

    fn fill(&mut self, lo: [i64; 3], hi: [i64; 3]) -> (u64, u64) {
        self.vox.fill(lo, hi);
        self.conn.apply_edit(&self.vox, lo, hi)
    }

    /// Rebind and resolve. `pinned_skip` excludes level-pinned components from
    /// the hypsometry scan — the measured cost of the pinned form.
    fn settle(&mut self, events: &[WaterEvent]) -> (dc_worldgen::water::ResolveStats, f64, f64) {
        let (comp_of, _body_of) = bind(&self.graph, &mut self.conn, &self.vox);
        // A level-pinned component never needs a capacity curve, so the scan
        // skips it. This is where the pinned form pays for itself.
        let skip: std::collections::BTreeSet<u32> = self
            .graph
            .bodies
            .iter()
            .filter(|b| b.merged_into.is_none() && b.kind == BodyKind::Pinned)
            .filter_map(|b| comp_of.get(&b.id).copied())
            .collect();
        let t0 = Instant::now();
        let hyps = self.conn.hypsometry_skipping(&self.vox, &skip);
        let hyp_s = t0.elapsed().as_secs_f64();
        let t1 = Instant::now();
        let stats = self.graph.resolve(events, &comp_of, &hyps);
        (stats, hyp_s, t1.elapsed().as_secs_f64())
    }

    fn wet(&mut self, x: usize, y: usize, z: usize) -> bool {
        let (_, body_of) = bind(&self.graph, &mut self.conn, &self.vox);
        self.graph.water_at(&mut self.conn, &body_of, x, y, z)
    }

    fn report(&mut self, tag: &str) {
        let bytes = self.graph.to_bytes().len();
        println!(
            "  [{tag}] bodies={} links={} coarse_nodes={} components={} persisted={}B",
            self.graph.live_nodes(),
            self.graph.live_links(),
            self.conn.coarse_nodes(),
            self.conn.component_count(),
            bytes
        );
    }
}

fn level_of(w: &World, id: BodyId) -> f32 {
    w.graph.get(w.graph.resolve_id(id)).level
}

fn volume_of(w: &World, id: BodyId) -> f64 {
    w.graph.get(w.graph.resolve_id(id)).volume
}

// ---------------------------------------------------------------- scenarios

fn scenario_1_lake_into_cave() {
    println!("\n== Scenario 1 — breach a lake bottom into a void ==");
    let mut w = World::solid(96, 64, 96);
    w.dig([30, 40, 30], [65, 47, 65]); // lake basin, 36x8x36
    w.dig([40, 10, 40], [55, 25, 55]); // cave, 16x16x16
    let lake_vol = 36.0 * 36.0 * 6.0;
    let lake = w.graph.add(BodyKind::Finite, [32, 41, 32], lake_vol, 46.0);
    w.settle(&[]);
    println!(
        "  before breach: lake level {:.2}, volume {:.0}",
        level_of(&w, lake),
        volume_of(&w, lake)
    );
    w.report("before");

    // Breach: a 1-voxel shaft from the lake floor into the cave roof.
    w.dig([47, 26, 47], [47, 39, 47]);
    let (st, hyp_s, res_s) = w.settle(&[WaterEvent::SpaceOpened { body: lake }]);
    println!(
        "  after breach: level {:.2}, volume {:.0}, merges {}, rounds {}, hyp {:.1} ms, resolve {:.3} ms",
        level_of(&w, lake),
        volume_of(&w, lake),
        st.merges,
        st.fixpoint_rounds,
        hyp_s * 1e3,
        res_s * 1e3
    );
    println!(
        "  cave floor (47,11,47) wet? {}   old lake surface (32,45,32) wet? {}",
        w.wet(47, 11, 47),
        w.wet(32, 45, 32)
    );
    w.report("after");
    println!(
        "  volume conserved: {:.0} == {lake_vol:.0}",
        volume_of(&w, lake)
    );

    // With an outlet: the cave transits to a lower sink instead of ponding.
    let mut w = World::solid(96, 64, 96);
    w.dig([30, 40, 30], [65, 47, 65]);
    w.dig([40, 10, 40], [55, 25, 55]);
    w.dig([2, 2, 2], [8, 8, 8]); // the sink, disconnected geometry
    let lake = w.graph.add(BodyKind::Finite, [32, 41, 32], lake_vol, 46.0);
    let cave = w.graph.add(BodyKind::Finite, [41, 11, 41], 0.0, 0.0);
    let sink = w.graph.add(BodyKind::Finite, [3, 3, 3], 0.0, 0.0);
    w.dig([47, 26, 47], [47, 39, 47]);
    let (st, _, _) = w.settle(&[
        WaterEvent::SpaceOpened { body: lake },
        WaterEvent::Breach {
            a: cave,
            b: sink,
            sill: 18.0,
        },
    ]);
    println!(
        "  with cave outlet at sill 18: merged level {:.2}, sink volume {:.0}, transits {}",
        level_of(&w, lake),
        volume_of(&w, sink),
        st.transits
    );
}

fn scenario_2_km_channel() {
    println!("\n== Scenario 2 — dig a km-long channel tangent from a river ==");
    let mut w = World::solid(1280, 48, 64);
    w.dig([0, 36, 0], [7, 40, 63]); // the river reach, along z at x<8
    let river = w.graph.add(BodyKind::Pinned, [2, 37, 32], 0.0, 40.0);
    w.settle(&[]);
    w.report("river only");

    let t0 = Instant::now();
    let (chunks, unions) = w.dig([8, 36, 30], [1279, 39, 33]);
    let dig_s = t0.elapsed().as_secs_f64();
    let (st, hyp_s, res_s) = w.settle(&[WaterEvent::SpaceOpened { body: river }]);
    println!(
        "  channel {} voxels long ({:.0} m at 0.9 m): {} chunks relabelled, {} coarse unions, index {:.1} ms, hyp {:.1} ms, resolve {:.3} ms, rounds {}",
        1272,
        1272.0 * 0.9,
        chunks,
        unions,
        dig_s * 1e3,
        hyp_s * 1e3,
        res_s * 1e3,
        st.fixpoint_rounds
    );
    let t0 = Instant::now();
    let far = w.wet(1279, 37, 31);
    let q_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  far end (1279,37,31) wet? {far}   (whole-query {q_us:.1} us incl. rebind)");
    let (_, body_of) = bind(&w.graph, &mut w.conn, &w.vox);
    let t0 = Instant::now();
    let mut hits = 0;
    for _ in 0..100_000 {
        if w.graph.water_at(&mut w.conn, &body_of, 1279, 37, 31) {
            hits += 1;
        }
    }
    println!(
        "  hot query: {:.1} ns/voxel over {hits} lookups (component → body → level)",
        t0.elapsed().as_secs_f64() * 1e9 / 100_000.0
    );
    w.report("after channel");

    // Cut the channel in the middle — the split case union-find cannot do.
    let t0 = Instant::now();
    let (chunks, unions) = w.fill([640, 36, 30], [641, 39, 33]);
    let split_s = t0.elapsed().as_secs_f64();
    w.settle(&[WaterEvent::InletCut { a: river, b: river }]);
    println!(
        "  SPLIT (2-voxel plug at x=640): {chunks} chunks relabelled, {unions} unions, {:.1} ms — components now {}",
        split_s * 1e3,
        w.conn.component_count()
    );
    println!(
        "  far end still wet? {}  (it is now a separate component with no body — derivation says dry)",
        w.wet(1279, 37, 31)
    );
}

fn scenario_3_km_chasm() {
    println!("\n== Scenario 3 — dig a km-deep chasm from a lake bottom ==");
    for pinned in [false, true] {
        let mut w = World::solid(64, 1216, 64);
        w.dig([16, 1200, 16], [47, 1209, 47]);
        let kind = if pinned {
            BodyKind::Pinned
        } else {
            BodyKind::Finite
        };
        let vol = 32.0 * 32.0 * 8.0;
        let lake = w.graph.add(kind, [17, 1201, 17], vol, 1208.0);
        w.settle(&[]);
        let t0 = Instant::now();
        let (chunks, _) = w.dig([30, 8, 30], [33, 1199, 33]);
        let dig_s = t0.elapsed().as_secs_f64();
        let (st, hyp_s, res_s) = w.settle(&[WaterEvent::SpaceOpened { body: lake }]);
        let t1 = Instant::now();
        let bottom_wet = w.wet(31, 9, 31);
        let q_us = t1.elapsed().as_secs_f64() * 1e6;
        println!(
            "  {} lake: chasm 1192 voxels ({:.0} m) — {chunks} chunks, index {:.1} ms, hyp {:.1} ms, resolve {:.3} ms, rounds {}",
            if pinned { "PINNED" } else { "FINITE" },
            1192.0 * 0.9,
            dig_s * 1e3,
            hyp_s * 1e3,
            res_s * 1e3,
            st.fixpoint_rounds
        );
        println!(
            "    level after {:.2} (was 1208); standing at the bottom (31,9,31): wet={bottom_wet}, answered in {q_us:.1} us",
            level_of(&w, lake)
        );
    }
}

fn scenario_4_ocean() {
    println!("\n== Scenario 4 — OCEAN SCALE: breaching a sea must not drain it ==");
    let (dx, dy, dz) = (512, 64, 512);
    for pinned in [false, true] {
        let mut w = World::solid(dx, dy, dz);
        w.dig([0, 32, 0], [511, 63, 511]); // the sea basin
        let sea_vol = 512.0 * 512.0 * 28.0;
        let kind = if pinned {
            BodyKind::Pinned
        } else {
            BodyKind::Finite
        };
        let sea = w.graph.add(kind, [10, 33, 10], sea_vol, 60.0);
        let (_, hyp0, _) = w.settle(&[]);
        println!(
            "  {} sea: {} voxel-volumes, hypsometry scan {:.1} ms",
            if pinned { "PINNED " } else { "FINITE " },
            sea_vol,
            hyp0 * 1e3
        );

        // Breach: a chasm under the sea floor with room for 3.8 M voxels.
        let t0 = Instant::now();
        w.dig([56, 4, 56], [455, 27, 455]);
        w.dig([200, 28, 200], [203, 31, 203]); // the breach shaft
        let dig_s = t0.elapsed().as_secs_f64();
        let (st, hyp_s, res_s) = w.settle(&[WaterEvent::SpaceOpened { body: sea }]);
        println!(
            "    breach into a 400x24x400 void ({} voxels): level {:.3} (was 60), Δ {:.3} voxels ({:.2} m), merges {}, rounds {}",
            400 * 24 * 400,
            level_of(&w, sea),
            60.0 - level_of(&w, sea),
            (60.0 - level_of(&w, sea)) as f64 * 0.9,
            st.merges,
            st.fixpoint_rounds
        );
        println!(
            "    index {:.1} ms, hypsometry {:.1} ms, resolve {:.3} ms; void floor (300,5,300) wet? {}",
            dig_s * 1e3,
            hyp_s * 1e3,
            res_s * 1e3,
            w.wet(300, 5, 300)
        );
        // What the pinned form actually costs: the capacity scan it can refuse.
        let t0 = Instant::now();
        let full = w.conn.hypsometry(&w.vox);
        let full_s = t0.elapsed().as_secs_f64();
        let sea_hyp_bytes: usize = full.values().map(|h| h.len() * 4).sum();
        println!(
            "    capacity scan if EVERY body is finite: {:.1} ms, {} component curves, {} B of curve",
            full_s * 1e3,
            full.len(),
            sea_hyp_bytes
        );
        w.report(if pinned { "pinned" } else { "finite" });
    }
    println!(
        "  reading: the FINITE sea loses level in proportion to the void it is asked to fill;"
    );
    println!("  the PINNED sea does not move and the void fills to sea level.");
}

fn scenario_5_adversarial(steps: usize) {
    println!("\n== Scenario 5 — adversarial sparsity: does the graph explode? ==");
    let (dx, dy, dz) = (512, 64, 512);
    let mut w = World::solid(dx, dy, dz);
    // One fed river along x.
    w.dig([0, 30, 4], [511, 34, 7]);
    let river = w.graph.add(BodyKind::Pinned, [4, 31, 5], 0.0, 34.0);
    w.settle(&[]);

    println!(
        "\n  {:>8} {:>10} {:>8} {:>8} {:>12} {:>13} {:>12}",
        "edits", "voxels", "bodies", "links", "coarse_nodes", "components", "persisted_B"
    );
    let mut edits = 0usize;
    let mut voxels = 0usize;
    let mut events: Vec<WaterEvent> = Vec::new();

    let checkpoint = |w: &mut World, edits: usize, voxels: usize| {
        let b = w.graph.to_bytes().len();
        println!(
            "  {:>8} {:>10} {:>8} {:>8} {:>12} {:>13} {:>12}",
            edits,
            voxels,
            w.graph.live_nodes(),
            w.graph.live_links(),
            w.conn.coarse_nodes(),
            w.conn.component_count(),
            b
        );
    };
    checkpoint(&mut w, 0, 0);

    // (a) A comb of parallel trenches off the river.
    for i in 0..64 {
        let z = 10 + i * 7;
        if z + 2 >= dz as i64 {
            break;
        }
        voxels += w.vox.dig([20 + i * 4, 30, z], [20 + i * 4 + 2, 34, z + 2]);
        w.conn
            .apply_edit(&w.vox, [20 + i * 4, 30, z], [20 + i * 4 + 2, 34, z + 2]);
        voxels += w.vox.dig([20 + i * 4, 30, 4], [20 + i * 4 + 2, 34, z]);
        w.conn
            .apply_edit(&w.vox, [20 + i * 4, 30, 4], [20 + i * 4 + 2, 34, z]);
        edits += 2;
    }
    events.push(WaterEvent::SpaceOpened { body: river });
    w.settle(&events);
    checkpoint(&mut w, edits, voxels);

    // (b) ~100 SEPARATE channels — deliberately not touching the river.
    for i in 0..100i64 {
        let z = 300 + (i % 50) * 4;
        let x = 20 + (i / 50) * 200;
        voxels += w.vox.dig([x, 20, z], [x + 120, 23, z + 1]);
        w.conn.apply_edit(&w.vox, [x, 20, z], [x + 120, 23, z + 1]);
        edits += 1;
    }
    w.settle(&events);
    checkpoint(&mut w, edits, voxels);

    // (c) A spiral shaft.
    let (cx, cz) = (256i64, 256i64);
    for s in 0..steps as i64 {
        let ang = s as f64 * 0.35;
        let r = 12.0 + (s as f64) * 0.02;
        let x = cx + (r * ang.cos()) as i64;
        let z = cz + (r * ang.sin()) as i64;
        let y = 60 - (s / 3).min(50);
        voxels += w.vox.dig([x, y, z], [x + 1, y + 1, z + 1]);
        w.conn.apply_edit(&w.vox, [x, y, z], [x + 1, y + 1, z + 1]);
        edits += 1;
    }
    w.settle(&events);
    checkpoint(&mut w, edits, voxels);

    // (d) A dug maze: a grid of corridors with random-ish plugs.
    let mut rng: u64 = 0xD15E_A5E5;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for i in 0..48i64 {
        let x = 60 + i * 8;
        voxels += w.vox.dig([x, 12, 60], [x + 1, 14, 440]);
        w.conn.apply_edit(&w.vox, [x, 12, 60], [x + 1, 14, 440]);
        edits += 1;
    }
    for i in 0..48i64 {
        let z = 60 + i * 8;
        voxels += w.vox.dig([60, 12, z], [440, 14, z + 1]);
        w.conn.apply_edit(&w.vox, [60, 12, z], [440, 14, z + 1]);
        edits += 1;
    }
    for _ in 0..400 {
        let x = 60 + (next() % 380) as i64;
        let z = 60 + (next() % 380) as i64;
        w.vox.fill([x, 12, z], [x + 1, 14, z + 1]);
        w.conn.apply_edit(&w.vox, [x, 12, z], [x + 1, 14, z + 1]);
        edits += 1;
    }
    w.settle(&events);
    checkpoint(&mut w, edits, voxels);

    // Per-edit cost of the index at this world size.
    let t0 = Instant::now();
    for i in 0..200i64 {
        let x = 100 + i;
        w.vox.dig([x, 40, 100], [x + 1, 41, 101]);
        w.conn.apply_edit(&w.vox, [x, 40, 100], [x + 1, 41, 101]);
    }
    println!(
        "  index maintenance: {:.3} ms per 1-voxel edit (relabel touched chunk + coarse rebuild) at {} coarse nodes",
        t0.elapsed().as_secs_f64() * 1e3 / 200.0,
        w.conn.coarse_nodes()
    );

    // Now bind a body to every component that has none — the worst case for
    // node growth: every disconnected hole is its own body.
    let comps = w.conn.component_count();
    let t0 = Instant::now();
    let mut seeded = 0;
    // One body per COMPONENT that has none — the true worst case for node
    // growth under the model: every disconnected hole a player dug is its own
    // persisted body.
    let mut claimed: std::collections::HashSet<u32> = std::collections::HashSet::new();
    'seed: for z in (0..dz).step_by(2) {
        for y in (0..dy).step_by(2) {
            for x in (0..dx).step_by(2) {
                if w.vox.is_solid(x, y, z) {
                    continue;
                }
                let Some(root) = w.conn.component(x, y, z) else {
                    continue;
                };
                if !claimed.insert(root) {
                    continue;
                }
                w.graph.add(
                    BodyKind::Finite,
                    [x as i32, y as i32, z as i32],
                    1.0,
                    y as f32,
                );
                seeded += 1;
                if seeded > 4000 {
                    break 'seed;
                }
            }
        }
    }
    let (st, hyp_s, res_s) = w.settle(&events);
    println!(
        "  WORST CASE — a body seeded per chunk-local hole ({seeded} added, {comps} components): after merge bodies={} links={} persisted={} B, hyp {:.0} ms, resolve {:.1} ms, rounds {}, merges {}, seed scan {:.0} ms",
        w.graph.live_nodes(),
        w.graph.live_links(),
        w.graph.to_bytes().len(),
        hyp_s * 1e3,
        res_s * 1e3,
        st.fixpoint_rounds,
        st.merges,
        t0.elapsed().as_secs_f64() * 1e3
    );
    println!(
        "  persisted bytes per live body: {:.1}",
        w.graph.to_bytes().len() as f64 / w.graph.live_nodes().max(1) as f64
    );
}

fn scenario_6_sealed_basin() {
    println!("\n== Scenario 6 — a bucket poured into a sealed stone basin ==");
    let mut w = World::solid(32, 32, 32);
    w.dig([12, 8, 12], [15, 11, 15]); // a 4x4x4 sealed void, no outlet, no pores
    w.settle(&[]);
    println!(
        "  derivation alone: (13,9,13) wet? {} — correct, nothing says water is here",
        w.wet(13, 9, 13)
    );
    let bucket = w.graph.add(BodyKind::Finite, [13, 9, 13], 8.0, 0.0);
    let (_, _, res_s) = w.settle(&[WaterEvent::RegimeCross {
        body: bucket,
        delta: 0.0,
    }]);
    println!(
        "  after the pour (8 voxel-volumes): level {:.2}, (13,8,13) wet? {}, (13,10,13) wet? {}",
        level_of(&w, bucket),
        w.wet(13, 8, 13),
        w.wet(13, 10, 13)
    );
    let bytes = w.graph.to_bytes();
    println!(
        "  the storage no derivation can replace: {} B for the whole graph ({} body), resolve {:.3} ms",
        bytes.len(),
        w.graph.live_nodes(),
        res_s * 1e3
    );
}

fn scenario_7_reload() {
    println!("\n== Scenario 7 — unload / reload identity ==");
    let (dx, dy, dz) = (192, 48, 192);
    let mut w = World::solid(dx, dy, dz);
    w.dig([0, 30, 4], [191, 34, 7]);
    let river = w.graph.add(BodyKind::Pinned, [4, 31, 5], 0.0, 34.0);
    for i in 0..24i64 {
        let z = 12 + i * 7;
        w.vox.dig([10 + i * 6, 30, z], [12 + i * 6, 34, z + 2]);
        w.conn
            .apply_edit(&w.vox, [10 + i * 6, 30, z], [12 + i * 6, 34, z + 2]);
        w.vox.dig([10 + i * 6, 30, 4], [12 + i * 6, 34, z]);
        w.conn
            .apply_edit(&w.vox, [10 + i * 6, 30, 4], [12 + i * 6, 34, z]);
    }
    w.dig([60, 6, 60], [100, 20, 100]);
    let cave = w.graph.add(BodyKind::Finite, [61, 7, 61], 9000.0, 0.0);
    w.settle(&[
        WaterEvent::SpaceOpened { body: river },
        WaterEvent::RegimeCross {
            body: cave,
            delta: 0.0,
        },
    ]);
    let (_, body_of) = bind(&w.graph, &mut w.conn, &w.vox);
    let t0 = Instant::now();
    let before = w.graph.derive_wet(&w.vox, &mut w.conn, &body_of);
    let derive_s = t0.elapsed().as_secs_f64();
    let wet_count: u32 = before.iter().map(|x| x.count_ones()).sum();

    // Unload: keep ONLY the persisted graph bytes and the geometry.
    let bytes = w.graph.to_bytes();
    let vox = w.vox.clone();
    drop(w);

    let t0 = Instant::now();
    let graph = BodyGraph::from_bytes(&bytes);
    let mut conn = ConnIndex::build(&vox);
    let (_, body_of) = bind(&graph, &mut conn, &vox);
    let rebuild_s = t0.elapsed().as_secs_f64();
    let after = graph.derive_wet(&vox, &mut conn, &body_of);

    println!(
        "  world {dx}x{dy}x{dz} ({} voxels), {} wet voxels derived in {:.0} ms",
        dx * dy * dz,
        wet_count,
        derive_s * 1e3
    );
    println!(
        "  persisted graph: {} B for {} bodies; full index rebuild + rebind {:.0} ms",
        bytes.len(),
        graph.live_nodes(),
        rebuild_s * 1e3
    );
    println!(
        "  derived water byte-identical after reload: {}",
        before == after
    );
}

fn event_storm() {
    println!("\n== Event storm — blocking the outlet of a chain of bodies ==");
    for n in [10usize, 100, 1000] {
        let mut g = BodyGraph::new();
        let mut ids = Vec::new();
        for i in 0..n {
            ids.push(g.add(BodyKind::Finite, [i as i32, 0, 0], 0.0, 0.0));
        }
        // Each body spills into the next over a lip at the same height, so
        // filling the head of the chain cascades all the way to the tail.
        // (A descending sill per body cannot be expressed once the chain is
        // longer than the container is tall, which is why the first version
        // of this measurement never fired for n >= 100.)
        for i in 0..n - 1 {
            g.links.push(dc_worldgen::water::Link {
                a: ids[i],
                b: ids[i + 1],
                sill: 32.0,
                open: true,
            });
        }
        let mut comp_of = HashMap::new();
        let mut hyps = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            comp_of.insert(*id, i as u32);
            hyps.insert(i as u32, vec![100u32; 64]);
        }
        // Load the head of the chain, then block the tail's outlet: everything
        // must re-resolve.
        // Enough to fill every body in the chain to its lip, and then some.
        let mut evs = vec![WaterEvent::RegimeCross {
            body: ids[0],
            delta: n as f64 * 3200.0 + 1000.0,
        }];
        let t0 = Instant::now();
        let st = g.resolve(&evs, &comp_of, &hyps);
        let fill_s = t0.elapsed().as_secs_f64();
        // Now the storm proper: block the TAIL's outlet and pour again. The
        // block is one event, but every body upstream of it has to re-resolve.
        evs.clear();
        evs.push(WaterEvent::OutletBlocked {
            a: ids[n - 2],
            b: ids[n - 1],
        });
        evs.push(WaterEvent::RegimeCross {
            body: ids[0],
            delta: n as f64 * 3200.0,
        });
        let t0 = Instant::now();
        let st2 = g.resolve(&evs, &comp_of, &hyps);
        println!(
            "  chain n={n}: fill = 1 event -> rounds {} transits {} ({:.2} ms); then OutletBlocked+pour -> rounds {} transits {} touched {} ({:.2} ms)",
            st.fixpoint_rounds,
            st.transits,
            fill_s * 1e3,
            st2.fixpoint_rounds,
            st2.transits,
            st2.bodies_touched,
            t0.elapsed().as_secs_f64() * 1e3
        );
    }
}

fn determinism() {
    println!("\n== Determinism — order-independence of event resolution ==");
    // A graph with every event kind touching overlapping bodies.
    let build = || {
        let mut g = BodyGraph::new();
        let ids: Vec<BodyId> = (0..24)
            .map(|i| g.add(BodyKind::Finite, [i, 0, 0], 50.0, 0.0))
            .collect();
        for i in 0..23usize {
            g.links.push(dc_worldgen::water::Link {
                a: ids[i],
                b: ids[i + 1],
                sill: 3.0 + (i % 5) as f32,
                open: true,
            });
        }
        let mut comp_of = HashMap::new();
        let mut hyps = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            // Two bodies deliberately share a component, to exercise merging.
            let c = (i / 2) as u32;
            comp_of.insert(*id, c);
            hyps.insert(c, vec![10u32; 32]);
        }
        (g, ids, comp_of, hyps)
    };
    let (_, ids, _, _) = build();
    let base_events: Vec<WaterEvent> = vec![
        WaterEvent::RegimeCross {
            body: ids[0],
            delta: 300.0,
        },
        WaterEvent::RegimeCross {
            body: ids[7],
            delta: 120.0,
        },
        WaterEvent::Breach {
            a: ids[3],
            b: ids[17],
            sill: 6.0,
        },
        WaterEvent::OutletBlocked {
            a: ids[4],
            b: ids[5],
        },
        WaterEvent::InletCut {
            a: ids[10],
            b: ids[11],
        },
        WaterEvent::SpaceOpened { body: ids[9] },
        WaterEvent::Saturated { body: ids[2] },
        WaterEvent::RegimeCross {
            body: ids[20],
            delta: -20.0,
        },
        WaterEvent::Breach {
            a: ids[21],
            b: ids[2],
            sill: 4.0,
        },
        WaterEvent::SpaceOpened { body: ids[15] },
        // The two adversarial shapes: a breach and a block naming the SAME
        // link in one batch, and several crossings into ONE body whose
        // magnitudes make float addition non-associative.
        WaterEvent::Breach {
            a: ids[12],
            b: ids[19],
            sill: 8.0,
        },
        WaterEvent::OutletBlocked {
            a: ids[19],
            b: ids[12],
        },
        WaterEvent::Breach {
            a: ids[19],
            b: ids[12],
            sill: 2.0,
        },
        WaterEvent::RegimeCross {
            body: ids[22],
            delta: 1e16,
        },
        WaterEvent::RegimeCross {
            body: ids[22],
            delta: 1.0,
        },
        WaterEvent::RegimeCross {
            body: ids[22],
            delta: -1e16,
        },
    ];

    let run = |order: &[usize]| {
        let (mut g, _, comp_of, hyps) = build();
        let evs: Vec<WaterEvent> = order.iter().map(|&i| base_events[i]).collect();
        g.resolve(&evs, &comp_of, &hyps);
        g.to_bytes()
    };

    let identity: Vec<usize> = (0..base_events.len()).collect();
    let reference = run(&identity);
    let mut rng: u64 = 0x5111_0BED;
    let mut ok = true;
    let trials = 256;
    for _ in 0..trials {
        let mut order = identity.clone();
        for i in (1..order.len()).rev() {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let j = (rng % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        if run(&order) != reference {
            ok = false;
            println!("  DIVERGED on order {order:?}");
            break;
        }
    }
    println!("  {trials} shuffled event orders → byte-identical graph: {ok}");
    println!(
        "  double-run byte-identical: {}",
        run(&identity) == reference
    );

    // The same proof one level up: a geometric edit sequence applied in two
    // different orders must land on the same persisted graph.
    let build_world = |order: &[usize]| {
        let mut w = World::solid(96, 48, 96);
        w.dig([0, 20, 0], [95, 24, 3]);
        let river = w.graph.add(BodyKind::Pinned, [2, 21, 1], 0.0, 24.0);
        let digs: Vec<([i64; 3], [i64; 3])> = (0..8)
            .map(|i| ([10 + i * 10, 20, 4], [12 + i * 10, 24, 40 + (i % 3) * 10]))
            .collect();
        for &i in order {
            let (lo, hi) = digs[i];
            w.dig(lo, hi);
        }
        w.settle(&[WaterEvent::SpaceOpened { body: river }]);
        w.graph.to_bytes()
    };
    let a = build_world(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let b = build_world(&[7, 3, 0, 5, 2, 6, 1, 4]);
    println!("  edit-order-independent world graph: {}", a == b);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let quick = has("--quick");
    let only_q1 = has("--q1");
    let only_q2 = has("--q2");

    println!("S11 — water locality + body graph (spike harness)");

    if !only_q2 {
        let num = |flag: &str, dflt: usize| -> usize {
            args.iter()
                .position(|a| a == flag)
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(dflt)
        };
        let (n, dy, steady, budget) = if quick {
            (65usize, 24usize, 400u32, 24u32)
        } else {
            (97, 32, 1600, 64)
        };
        let (n, dy) = (num("--n", n), num("--dy", dy));
        let (steady, budget) = (
            num("--steady", steady as usize) as u32,
            num("--budget", budget as usize) as u32,
        );
        q1(n, dy, steady, budget);
    }
    if !only_q1 {
        scenario_1_lake_into_cave();
        scenario_2_km_channel();
        scenario_3_km_chasm();
        scenario_4_ocean();
        scenario_5_adversarial(if quick { 200 } else { 900 });
        scenario_6_sealed_basin();
        scenario_7_reload();
        event_storm();
        determinism();
    }
}
