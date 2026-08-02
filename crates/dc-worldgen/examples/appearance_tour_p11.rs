//! **The tour map for the three stacked appearance changes nobody has stood in
//! front of** — octaves member stepping (journal/0128–0129), member diversity in
//! the deep record (P11 slice 1), and provenance over site climate (P11 slice 2b).
//!
//! Merged 2026-07-29 → 2026-08-02, never walked. This probe finds the stations on
//! the world the client actually boots — `BENCH_SEED` 1337, `Extent::Medium`,
//! default `DeepOverrides` — and prints ready-to-paste poses. **It tunes nothing.**
//!
//! Run: `cargo run --release -p dc-worldgen --example appearance_tour_p11`
//! Options: `-- --stride N` (deep-cell sampling stride for the expression scan,
//! default 6), `-- --top N` (how many ranked stations to print, default 3).
//!
//! # Score what the walk will SEE
//!
//! Every station is filtered by **the expression path that renders it**, not by
//! the record alone (tour-map skill, the far-frontier trap):
//!
//! - land is `WorldGenerator::surface_elev_m > 2 m` — the *near-field* surface the
//!   player stands on, after river carving, not the deep bilinear `surf`;
//! - the material claim is read through `ColumnFill::plan(d)` over the chunk's own
//!   `StrataRec`, which is exactly what `collapse.rs` interns into voxels: the
//!   **surface voxel is `plan(1)`** and the voxel `d` below it is `plan(d + 1)`.
//!
//! # The three signatures, and why they need different stations
//!
//! `StrataEvent::dither` splits the record in two, and the split decides which
//! change is visible where:
//!
//! - **year-zero veneer events carry `dither: true`** — the clastic veneer, the
//!   igneous bodies, the placer, the weathering front. Their member is re-picked
//!   per voxel column through `selection_field` (now `draws::Octaves`). **This is
//!   the only place the octaves slice can be seen**, and it needs a `Single` top
//!   span in a ≥2-member class (station set 1).
//! - **deep-history events carry `dither: false`** — the recorded `MaterialId` from
//!   `DepUnit::species`, laid by fitness at deposition. Expression no longer
//!   re-adjudicates them. **This is where P11's member diversity lives**, below the
//!   veneer, which is why station set 2 is a bench cut (station set 2).
//! - **provenance** is a property of those same recorded units: a bed whose name
//!   came from the arriving composition rather than from a draw under the site's
//!   climate (station set 3).

use std::collections::HashMap;
use std::time::Instant;

use dc_core::coarse::DitherSource;
use dc_core::materials::MaterialId;
use dc_core::materials::geology::{GeoMemberIdx, GeologySet, vanilla};
use dc_sim::statistical::rng::Draws;
use dc_worldgen::deeptime::climate;
use dc_worldgen::deeptime::lithology::Litho;
use dc_worldgen::deeptime::{
    DeepConfig, DeepStrata, MemberCtx, SEA_LEVEL_M, dep_tags, litho_of_tag, production_config,
    run_cells,
};
use dc_worldgen::draws::{Coherent, Octaves};
use dc_worldgen::geology::{StrataEvent, dithered_member_with};
use dc_worldgen::pregen::{CELL_VOXELS, Extent, Pregen, WorldParams};
use dc_worldgen::{ColumnFill, DeepOverrides, Plan, WorldGenerator};

/// The client's `BENCH_SEED`.
const SEED: u64 = 1337;
/// `GenOptions::default().extent` — the client's boot extent.
const EXTENT: Extent = Extent::Medium;
/// N=2 voxel edge, metres.
const VOXEL_M: f64 = 0.9;
/// Keep stations off the border wilds (no deep-time run out there).
const EDGE_MARGIN: i64 = 6;
/// Near-field surface elevation a station must clear to be walkable land.
const LAND_M: f64 = 2.0;

// ─────────────────────────────── coordinates ────────────────────────────────

/// The deep-cell ↔ world-voxel bridge — `DeepField::deep_coords`' own centring,
/// lifted verbatim from `walk_tour_0115` (which lifted it from `tour_map`).
struct Conv {
    w: usize,
    wp: usize,
}

impl Conv {
    fn idx_to_voxel(&self, idx: usize) -> (i64, i64) {
        let (w, wp) = (self.w as f64, self.wp as f64);
        let (gx, gy) = ((idx % self.w) as f64, (idx / self.w) as f64);
        let px = (gx + 0.5) / w * wp - 0.5;
        let py = (gy + 0.5) / w * wp - 0.5;
        let half = (self.wp / 2) as f64;
        (
            ((px - half + 0.5) * CELL_VOXELS as f64).round() as i64,
            ((py - half + 0.5) * CELL_VOXELS as f64).round() as i64,
        )
    }
    fn meters(&self, idx: usize) -> (f64, f64) {
        let (vx, vz) = self.idx_to_voxel(idx);
        (vx as f64 * VOXEL_M, vz as f64 * VOXEL_M)
    }
    fn interior(&self, idx: usize) -> bool {
        let (gx, gy) = ((idx % self.w) as i64, (idx / self.w) as i64);
        let w = self.w as i64;
        gx >= EDGE_MARGIN && gy >= EDGE_MARGIN && gx < w - EDGE_MARGIN && gy < w - EDGE_MARGIN
    }
}

fn km_between(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt() / 1000.0
}

// ─────────────────────────────── the expression read ─────────────────────────

/// What the top of a column expresses, read through the path that renders it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TopSpan {
    /// No record — a fallback column (ocean floor, wilds, basement-less).
    None,
    /// `Plan::Mixed` — several events share the surface voxel. `mixed_at` uses
    /// the **undithered** `event.member`, so the octaves dither cannot move it.
    Mixed,
    /// `Plan::Single` on a **veneer** event (`dither: true`) in a class with ≥2
    /// members — the octaves slice is live here, and only here.
    SingleMovable,
    /// `Plan::Single` on an event whose member cannot move: either a recorded
    /// deep-history identity (`dither: false`) or a one-member class.
    SingleFrozen,
}

fn class_members(set: &GeologySet, m: GeoMemberIdx) -> usize {
    set.class(set.member(m).class.as_str())
        .map_or(0, |c| c.members().len())
}

fn top_span(set: &GeologySet, fill: &ColumnFill, events: &[StrataEvent]) -> (TopSpan, Option<usize>) {
    match fill.plan(1) {
        None => (TopSpan::None, None),
        Some(Plan::Mixed(_)) => (TopSpan::Mixed, None),
        Some(Plan::Single(k)) => {
            let e = &events[*k];
            if e.dither && class_members(set, e.member) > 1 {
                (TopSpan::SingleMovable, Some(*k))
            } else {
                (TopSpan::SingleFrozen, Some(*k))
            }
        }
    }
}

/// The expressed member at depth index `d` (1 = the surface voxel), at one voxel
/// column — exactly `collapse.rs`'s resolution, `Mixed` reduced to its heaviest
/// share so a vertical succession reads as one member per voxel.
fn expressed_member(
    set: &GeologySet,
    fill: &ColumnFill,
    events: &[StrataEvent],
    d: u32,
    vx: i64,
    vz: i64,
) -> Option<GeoMemberIdx> {
    match fill.plan(d)? {
        Plan::Single(k) => {
            let e = events[*k];
            Some(dithered_member_with(
                set,
                &e,
                &dc_worldgen::geology::selection_field(SEED, e.sel_salt),
                vx,
                vz,
            ))
        }
        Plan::Mixed(w) => w
            .iter()
            .max_by_key(|(_, eighths)| *eighths)
            .map(|(k, _)| events[*k].member),
    }
}

// ─────────────────────────────── the member field ────────────────────────────

/// Half-edge of the member-field window, voxels. 64 → a 128-voxel (115 m) square:
/// four chunks across, which is what a standing player sees of the ground in
/// front of them.
const WIN_VOX: i64 = 64;

/// The distinct members a source draws over the window, and the minority share.
/// **This is the ranking quantity for station set 1**: a `Single` top span in a
/// two-member class shows the octaves patchwork only if the fitness envelope at
/// this site actually splits the two members. A minority share near 0 is a class
/// that is *registered* two-member and *expressed* one-member — nothing to see.
fn member_window(
    set: &GeologySet,
    e: &StrataEvent,
    src: &dyn DitherSource,
    cx: i64,
    cz: i64,
    half: i64,
) -> (usize, f64, Vec<(GeoMemberIdx, usize)>) {
    let mut counts: Vec<(GeoMemberIdx, usize)> = Vec::new();
    let n = (half * 2) as usize;
    for z in 0..n as i64 {
        for x in 0..n as i64 {
            let m = dithered_member_with(set, e, src, cx - half + x, cz - half + z);
            match counts.iter_mut().find(|(k, _)| *k == m) {
                Some((_, c)) => *c += 1,
                None => counts.push((m, 1)),
            }
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    let total = (n * n) as f64;
    let minority = if counts.len() > 1 {
        counts[1..].iter().map(|(_, c)| *c).sum::<usize>() as f64 / total
    } else {
        0.0
    };
    (counts.len(), minority, counts)
}

/// `P(m(p) == m(p + lag·x̂))` over a member field — the patch-size read. Returned
/// for the lag ladder so "visible spatial extent" is a measurement, not an
/// adjective.
fn agreement(field: &[u16], n: usize, lag: usize) -> f64 {
    let (mut same, mut total) = (0u64, 0u64);
    for z in 0..n {
        for x in 0..n.saturating_sub(lag) {
            total += 1;
            if field[z * n + x] == field[z * n + x + lag] {
                same += 1;
            }
        }
    }
    same as f64 / total.max(1) as f64
}

fn member_field(
    set: &GeologySet,
    e: &StrataEvent,
    src: &dyn DitherSource,
    cx: i64,
    cz: i64,
    half: i64,
) -> Vec<u16> {
    let n = (half * 2) as usize;
    let mut out = Vec::with_capacity(n * n);
    for z in 0..n as i64 {
        for x in 0..n as i64 {
            out.push(dithered_member_with(set, e, src, cx - half + x, cz - half + z).0 as u16);
        }
    }
    out
}

// ─────────────────────────────── record-side scans ───────────────────────────

/// A run of one member in a column, top-down or bottom-up as built.
#[derive(Clone, Copy)]
struct Band {
    mat: MaterialId,
    thickness_m: f64,
}

/// **Station set 2's ranking quantity**: alternations between two members of the
/// SAME class in the recorded succession, counting only bands thick enough to
/// express (≥ one voxel). Two adjacent bands of mudstone and siltstone are one
/// alternation; mudstone → sandstone is a class change and is *not* counted,
/// because the claim P11 slice 1 makes is that the record distinguishes rocks
/// **inside** a class it used to collapse.
fn member_alternations(strata: &DeepStrata) -> (usize, Vec<Band>) {
    // Coalesce adjacent same-material units, bottom-up (the record's own order).
    let mut bands: Vec<Band> = Vec::new();
    for u in &strata.units {
        if u.thickness_m <= 0.0 {
            continue;
        }
        match bands.last_mut() {
            Some(b) if b.mat == u.species => b.thickness_m += u.thickness_m,
            _ => bands.push(Band {
                mat: u.species,
                thickness_m: u.thickness_m,
            }),
        }
    }
    let thick: Vec<Band> = bands.into_iter().filter(|b| b.thickness_m >= VOXEL_M).collect();
    let mut alt = 0usize;
    for w in thick.windows(2) {
        if w[0].mat != w[1].mat
            && Litho::of_material(w[0].mat) == Litho::of_material(w[1].mat)
        {
            alt += 1;
        }
    }
    (alt, thick)
}

/// **Station set 3's instrument, localized.** The aggregate audit
/// (`DeepConfig::identity_audit`) says *how many metres* of the archive carry a
/// name the deposition site's own climate would not have drawn; it is a run-wide
/// tally with no per-cell plane exported, so it cannot say **where**.
///
/// This re-evaluates the same counterfactual per recorded unit, post-hoc, under
/// the **site's present climate** — which is what the walk can actually stand in:
/// `MemberCtx::surface(cell, air_temp_c(lat, r+h), precip, class, TRANSPORT, 0)`,
/// the exact call `erosion::record` makes, at the exact draw address (cell,
/// chapter, tag, k).
///
/// **The difference from the in-run audit is named, not assumed away:** the audit
/// asks the question under the *depositional epoch's* climate; this asks it under
/// the present one. So a unit can disagree here for two reasons — its identity
/// came from the arriving load (the ruling's outcome), or the site's climate has
/// drifted since it was laid. The aggregate audit is printed beside this as the
/// control on the first mechanism's magnitude, and the class-vs-tag column below
/// separates the classes the environment implies from the classes that arrived.
fn site_would_draw(
    mem_by_chapter: &mut HashMap<u8, MemberCtx<'_>>,
    geology: &'static GeologySet,
    cell: usize,
    temp_c: f64,
    precip: f64,
    mat: MaterialId,
    chapter: u8,
) -> MaterialId {
    let ctx = mem_by_chapter
        .entry(chapter)
        .or_insert_with(|| MemberCtx::new(geology, SEED, u64::from(chapter)));
    ctx.surface(
        cell,
        temp_c,
        precip,
        Litho::of_material(mat),
        dep_tags::TRANSPORT,
        0,
    )
}

// ─────────────────────────────── report helpers ──────────────────────────────

fn short(m: MaterialId) -> &'static str {
    // `qualified_name` is `dc:mudstone` etc.; the tail is what a briefing reads.
    let q = m.qualified_name();
    q.rsplit(':').next().unwrap_or(q)
}

fn pose(label: &str, x: f64, z: f64, surf: f64, standoff: f64, yaw: f64, pitch: f64, why: &str) {
    println!(
        "    POSE {label}: pose_set{{ x: {x:.0}, z: {z:.0}, surface: true }} then \
         pose_set{{ x: {x:.0}, y: {:.0}, z: {z:.0}, yaw: {yaw:.2}, pitch: {pitch:.2} }}",
        surf + standoff
    );
    println!("      ({why}; feet y is worldgen-frame {:.0} m + {standoff:.0} m standoff — teleport with surface:true FIRST and read the live ground Y, then re-set y)", surf);
}

// ─────────────────────────────── main ────────────────────────────────────────

fn arg_val(name: &str, default: usize) -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn main() {
    let stride = arg_val("--stride", 6).max(1);
    let top_n = arg_val("--top", 3).max(1);
    let t0 = Instant::now();

    println!("=== APPEARANCE TOUR: octaves stepping · member diversity · provenance ===");
    println!(
        "seed {SEED}, extent {}, default deep overrides, N=2 ({VOXEL_M} m voxels)",
        EXTENT.label()
    );

    let t = Instant::now();
    let pregen = Pregen::run_with(
        WorldParams {
            seed: SEED,
            extent: EXTENT,
        },
        &DeepOverrides::default(),
    );
    eprintln!("pregen + deep field: {:.1} s", t.elapsed().as_secs_f64());
    let field = &pregen.deep;
    let (w, wp) = (field.w, field.wp);
    let conv = Conv { w, wp };
    println!(
        "deep grid {w}x{w} @ {:.1} m/cell ({} cells); pregen {wp}x{wp}\n",
        field.cell_m,
        w * w
    );

    // The content set the client boots with. Leaked deliberately: `MemberCtx`
    // borrows it for the whole run and the probe is a one-shot binary.
    let set: &'static GeologySet = Box::leak(Box::new(vanilla()));

    // ── the land mask, from the NEAR-FIELD surface (what the walk stands on) ──
    let mut wgen = WorldGenerator::new(&pregen);

    // ═══════════════════════════════════════════════════════════════════════
    // PASS 1 — the expression scan. One `column_record` per sampled deep cell,
    // at the chunk holding that cell's centre. This is the only instrument that
    // can answer "what is the surface top span", because the veneer that owns
    // the surface is built at expression and is not in the deep record at all.
    // ═══════════════════════════════════════════════════════════════════════
    struct Sample {
        idx: usize,
        vx: i64,
        vz: i64,
        surf_m: f64,
        span: TopSpan,
        /// Ranking quantity for station 1: minority member share in the window.
        minority: f64,
        distinct: usize,
        /// The movable event, when there is one.
        event: Option<StrataEvent>,
        /// Expressed member alternations down the column (station 2's verified half).
        expr_alt: usize,
        expr_depth: usize,
    }

    let t = Instant::now();
    let mut samples: Vec<Sample> = Vec::new();
    let mut span_counts = [0usize; 4]; // None, Mixed, SingleMovable, SingleFrozen
    let mut land_sampled = 0usize;
    let mut sampled = 0usize;
    for gy in (0..w).step_by(stride) {
        for gx in (0..w).step_by(stride) {
            let idx = gy * w + gx;
            if !conv.interior(idx) {
                continue;
            }
            sampled += 1;
            // Cheap record-side reject first: the deep bilinear surface. The
            // near-field read below is the authority; this only saves work.
            if field.surf[idx] <= SEA_LEVEL_M {
                continue;
            }
            let (vx, vz) = conv.idx_to_voxel(idx);
            let surf_m = wgen.surface_elev_m(vx, vz);
            if surf_m <= LAND_M {
                continue;
            }
            land_sampled += 1;
            let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
            let col = wgen.column_record(cx, cz);
            let fill = ColumnFill::build(&col.strata, VOXEL_M);
            let (span, k) = top_span(set, &fill, &col.strata.events);
            span_counts[match span {
                TopSpan::None => 0,
                TopSpan::Mixed => 1,
                TopSpan::SingleMovable => 2,
                TopSpan::SingleFrozen => 3,
            }] += 1;

            // Station 1's ranking quantity — only meaningful on a movable span.
            let (mut minority, mut distinct, mut event) = (0.0, 0usize, None);
            if span == TopSpan::SingleMovable
                && let Some(k) = k
            {
                let e = col.strata.events[k];
                let src = Octaves::member(Draws::from_recorded_salt(SEED, e.sel_salt));
                // A 32-voxel half-window is the cheap screen; the top picks get
                // the full 128-voxel window below.
                let (d, m, _) = member_window(set, &e, &src, vx, vz, 32);
                minority = m;
                distinct = d;
                event = Some(e);
            }

            // Station 2's verified half: the expressed vertical succession at
            // this very voxel column, through `plan(d)`.
            let depth = fill.depth_count();
            let mut expr_alt = 0usize;
            let mut prev: Option<GeoMemberIdx> = None;
            for d in 1..=depth as u32 {
                let Some(m) = expressed_member(set, &fill, &col.strata.events, d, vx, vz) else {
                    continue;
                };
                if let Some(p) = prev
                    && p != m
                    && set.member(p).class == set.member(m).class
                {
                    expr_alt += 1;
                }
                prev = Some(m);
            }

            samples.push(Sample {
                idx,
                vx,
                vz,
                surf_m,
                span,
                minority,
                distinct,
                event,
                expr_alt,
                expr_depth: depth,
            });
        }
    }
    eprintln!(
        "expression scan: {:.1} s ({} cells sampled, stride {stride})",
        t.elapsed().as_secs_f64(),
        sampled
    );

    println!("--- THE EXPRESSION SCAN (stride {stride} over the deep grid) ---");
    println!(
        "  {sampled} interior deep cells sampled · {land_sampled} on land (near-field surface > {LAND_M:.0} m)"
    );
    println!(
        "  surface top span:  Single/MOVABLE {} ({:.1} %)   Single/frozen {}   Mixed {}   no record {}",
        span_counts[2],
        100.0 * span_counts[2] as f64 / land_sampled.max(1) as f64,
        span_counts[3],
        span_counts[1],
        span_counts[0]
    );
    println!(
        "  (MOVABLE = `Plan::Single` on a `dither:true` veneer event in a >=2-member class. \
         Only these can show the octaves slice: `Mixed` uses the UNDITHERED member by design, \
         and deep-history events carry `dither:false`.)\n"
    );

    // ═══════════════════════════════════════════════════════════════════════
    // STATION SET 1 — the octaves near-member stepping
    // ═══════════════════════════════════════════════════════════════════════
    println!("========== STATION SET 1 — OCTAVES NEAR-MEMBER STEPPING ==========");
    println!(
        "The standing U3-dominance question (ROADMAP § Observed). The old U3 pose has a\n\
         `Mixed` top span and is structurally blind; this looks for `Single` in a two-member\n\
         class, on land, where the octave structure has visible spatial extent.\n"
    );
    let mut movable: Vec<&Sample> = samples
        .iter()
        .filter(|s| s.span == TopSpan::SingleMovable && s.distinct > 1)
        .collect();
    movable.sort_by(|a, b| b.minority.total_cmp(&a.minority));

    if movable.is_empty() {
        println!(
            "  ⚠ NULL — not one sampled land cell expresses `Plan::Single` on a movable\n\
             (`dither:true`, >=2-member) event with more than one member drawn in its window.\n\
             The control is the census above: the scan DID find {} movable spans and {} Single\n\
             spans overall, so the instrument sees the category; what is absent is the\n\
             two-member expression. Do not launch for this station.",
            span_counts[2],
            span_counts[2] + span_counts[3]
        );
    } else {
        println!(
            "  DISTRIBUTION: {} of {land_sampled} sampled land cells ({:.1} %) carry a movable\n\
             `Single` top span whose window draws BOTH members. Minority share p50 {:.3}, \
             max {:.3}.",
            movable.len(),
            100.0 * movable.len() as f64 / land_sampled.max(1) as f64,
            movable[movable.len() / 2].minority,
            movable[0].minority
        );
        let mut picked: Vec<&&Sample> = Vec::new();
        for s in &movable {
            if picked.len() >= top_n {
                break;
            }
            let (mx, mz) = conv.meters(s.idx);
            if picked
                .iter()
                .all(|p| km_between(conv.meters(p.idx), (mx, mz)) > 3.0)
            {
                picked.push(s);
            }
        }
        for (rank, s) in picked.iter().enumerate() {
            let (mx, mz) = conv.meters(s.idx);
            let e = s.event.expect("a movable sample carries its event");
            let after = Octaves::member(Draws::from_recorded_salt(SEED, e.sel_salt));
            let before = Coherent::new(Draws::from_recorded_salt(SEED, e.sel_salt), 32);
            let (da, ma, counts) = member_window(set, &e, &after, s.vx, s.vz, WIN_VOX);
            let (db, mb, _) = member_window(set, &e, &before, s.vx, s.vz, WIN_VOX);
            let fa = member_field(set, &e, &after, s.vx, s.vz, WIN_VOX);
            let fb = member_field(set, &e, &before, s.vx, s.vz, WIN_VOX);
            let n = (WIN_VOX * 2) as usize;
            println!(
                "\n  S1 #{}  world ({mx:.0} m, {mz:.0} m)  deep cell ({},{})  voxel ({}, {})",
                rank + 1,
                s.idx % w,
                s.idx / w,
                s.vx,
                s.vz
            );
            println!("    near-field surface elevation {:.1} m", s.surf_m);
            println!(
                "    surface event: class {}  ({} registered members)  sel_salt {:#x} tag {}",
                set.member(e.member).class,
                class_members(set, e.member),
                e.sel_salt,
                e.sel_tag
            );
            print!("    RANKING QUANTITY — members drawn over a 115 m window (AFTER/octaves): ");
            for (m, c) in counts.iter() {
                print!(
                    "{} {:.1}%  ",
                    short(set.member(*m).material),
                    100.0 * *c as f64 / (n * n) as f64
                );
            }
            println!("\n      distinct {da}, minority share {ma:.3}");
            println!(
                "    A/B at this site — BEFORE (single octave, stride 32): distinct {db}, minority {mb:.3}"
            );
            println!("    patch size, P(m(p) == m(p+lag)) along x:");
            println!("      lag(vox)  metres   BEFORE   AFTER");
            for lag in [1usize, 2, 4, 8, 16, 32, 64] {
                println!(
                    "      {lag:>8}  {:>6.1}   {:.4}   {:.4}",
                    lag as f64 * VOXEL_M,
                    agreement(&fb, n, lag),
                    agreement(&fa, n, lag)
                );
            }
            pose(
                "eyes-down",
                mx,
                mz,
                s.surf_m,
                12.0,
                0.0,
                -0.9,
                "12 m up, pitched down: a 115 m patch of ground fills the frame. --fullbright, no --edges (material question)",
            );
            pose(
                "standing",
                mx,
                mz,
                s.surf_m,
                1.8,
                0.0,
                -0.35,
                "eye height, the ground the octaves slice draws under your feet",
            );
        }

        // The contrast: a Mixed top span nearby.
        let anchor = conv.meters(picked[0].idx);
        let mut mixed: Vec<&Sample> = samples.iter().filter(|s| s.span == TopSpan::Mixed).collect();
        mixed.sort_by(|a, b| {
            km_between(conv.meters(a.idx), anchor).total_cmp(&km_between(conv.meters(b.idx), anchor))
        });
        println!("\n  CONTRAST (the `Mixed` top span the octaves dither cannot touch):");
        match mixed.first() {
            None => println!(
                "    none sampled — every land sample was `Single`. The contrast pair does not exist at this stride."
            ),
            Some(c) => {
                let (cx_m, cz_m) = conv.meters(c.idx);
                println!(
                    "    world ({cx_m:.0} m, {cz_m:.0} m), surface {:.1} m — {:.1} km from S1 #1",
                    c.surf_m,
                    km_between((cx_m, cz_m), anchor)
                );
                pose(
                    "contrast",
                    cx_m,
                    cz_m,
                    c.surf_m,
                    12.0,
                    0.0,
                    -0.9,
                    "same framing as S1 #1 — the per-voxel eighth allocation varies this ground, not the member dither",
                );
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATION SET 2 — member diversity in the deep record (the bench cut)
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n\n========== STATION SET 2 — MEMBER DIVERSITY IN THE DEEP RECORD ==========");
    println!(
        "P11 slices 1 + 2b. Ranked by MEMBER ALTERNATIONS inside one class in the recorded\n\
         succession, counting only bands >= one voxel (0.9 m) thick. Shortlisted on the deep\n\
         record over ALL land cells, then VERIFIED through the expression path.\n"
    );
    let t = Instant::now();
    let mut alt_rank: Vec<(usize, usize, Vec<Band>)> = Vec::new();
    let mut cells_with_alt = 0usize;
    let mut land_cells = 0usize;
    for idx in 0..w * w {
        if !conv.interior(idx) || field.surf[idx] <= SEA_LEVEL_M {
            continue;
        }
        land_cells += 1;
        let (alt, bands) = member_alternations(&field.strata[idx]);
        if alt > 0 {
            cells_with_alt += 1;
            alt_rank.push((alt, idx, bands));
        }
    }
    alt_rank.sort_by(|a, b| b.0.cmp(&a.0));
    eprintln!("record scan: {:.1} s", t.elapsed().as_secs_f64());
    println!(
        "  DISTRIBUTION: {cells_with_alt} of {land_cells} interior LAND deep cells ({:.2} %) carry\n\
         at least one within-class member alternation in expressible bands. Max {} alternations.",
        100.0 * cells_with_alt as f64 / land_cells.max(1) as f64,
        alt_rank.first().map(|a| a.0).unwrap_or(0)
    );
    if alt_rank.is_empty() {
        println!(
            "  ⚠ NULL — no land cell alternates members inside a class in expressible bands.\n\
             Control: run `member_diversity_probe`, whose census reports the per-class distinct\n\
             material counts over the same archive; a non-trivial count there with a zero here\n\
             means the diversity exists but never stacks two members adjacently."
        );
    } else {
        let mut picked: Vec<&(usize, usize, Vec<Band>)> = Vec::new();
        for c in &alt_rank {
            if picked.len() >= top_n {
                break;
            }
            let m = conv.meters(c.1);
            // Must be walkable land in the NEAR field, verified, not just recorded.
            let (vx, vz) = conv.idx_to_voxel(c.1);
            if wgen.surface_elev_m(vx, vz) <= LAND_M {
                continue;
            }
            if picked
                .iter()
                .all(|p| km_between(conv.meters(p.1), m) > 5.0)
            {
                picked.push(c);
            }
        }
        for (rank, (alt, idx, bands)) in picked.iter().enumerate() {
            let (mx, mz) = conv.meters(*idx);
            let (vx, vz) = conv.idx_to_voxel(*idx);
            let surf_m = wgen.surface_elev_m(vx, vz);
            let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
            let col = wgen.column_record(cx, cz);
            let fill = ColumnFill::build(&col.strata, VOXEL_M);
            println!(
                "\n  S2 #{}  world ({mx:.0} m, {mz:.0} m)  deep cell ({},{})  voxel ({vx}, {vz})  \
                 chunk ({cx}, {cz})",
                rank + 1,
                idx % w,
                idx / w
            );
            println!("    near-field surface elevation {surf_m:.1} m");
            println!("    RANKING QUANTITY — {alt} within-class member alternations in the record");
            println!("    recorded succession (bottom-up, bands >= 0.9 m):");
            for b in bands.iter() {
                println!("      {:>10}  {:>8.2} m", short(b.mat), b.thickness_m);
            }
            // VERIFY through expression: walk the plans and print the member per voxel.
            println!(
                "    EXPRESSED column at voxel ({vx},{vz}) — `plan(d)`, d=1 is the SURFACE voxel:"
            );
            let depth = fill.depth_count();
            let mut runs: Vec<(String, u32, u32)> = Vec::new();
            let mut expr_alt = 0usize;
            let mut prev: Option<GeoMemberIdx> = None;
            for d in 1..=depth as u32 {
                let Some(m) = expressed_member(set, &fill, &col.strata.events, d, vx, vz) else {
                    continue;
                };
                let name = short(set.member(m).material).to_string();
                match runs.last_mut() {
                    Some((n, _, hi)) if *n == name => *hi = d,
                    _ => runs.push((name, d, d)),
                }
                if let Some(p) = prev
                    && p != m
                    && set.member(p).class == set.member(m).class
                {
                    expr_alt += 1;
                }
                prev = Some(m);
            }
            println!(
                "      {} recorded voxels deep; {expr_alt} within-class alternations SURVIVE expression",
                depth
            );
            for (name, lo, hi) in runs.iter().take(28) {
                println!(
                    "      d {lo:>3}..{hi:<3} ({:>7.1} m below surface)  {name}",
                    (*lo as f64 - 1.0) * VOXEL_M
                );
            }
            if runs.len() > 28 {
                println!("      ... {} more runs below", runs.len() - 28);
            }
            if expr_alt == 0 {
                println!(
                    "      ⚠ the record's alternations do NOT survive quantization at this column —\n\
                       a band thinner than the voxel it must fill. Prefer a lower-ranked station\n\
                       whose expressed count is non-zero."
                );
            }
            // ── the bench cut, in VOXELS (world_fill speaks voxels) ──
            let sy = (surf_m / VOXEL_M).floor() as i64;
            let deepest = runs.last().map(|(_, _, hi)| *hi).unwrap_or(1) as i64;
            let cut = deepest.min(24).max(8); // voxels of face to expose
            // 61 x 41 x 8 = 20,008 voxels of dc:air — the skill's ~20k bench.
            println!(
                "    BENCH CUT (voxels; `world_fill` speaks VOXELS, pose speaks METRES):\n\
                 \x20     dc:air  x [{}, {}]  y [{}, {}]  z [{}, {}]   = {} voxels",
                vx - 30,
                vx + 30,
                sy - cut + 1,
                sy,
                vz - 20,
                vz - 0,
                61 * 21 * cut
            );
            println!(
                "      cuts {cut} voxels ({:.1} m) down from the surface voxel y={sy}; \
                 the exposed NORTH face is at z = {} (voxel), i.e. {:.0} m",
                cut as f64 * VOXEL_M,
                vz - 21,
                (vz - 21) as f64 * VOXEL_M
            );
            pose(
                "road-cut face",
                mx,
                (vz - 40) as f64 * VOXEL_M,
                surf_m,
                3.0,
                0.0,
                -0.15,
                "stand ~18 m south of the cut looking NORTH at the exposed face (yaw 0 = +z); \
                 --fullbright for material, no --edges for the colour read",
            );
            pose(
                "face, elevated",
                mx,
                (vz - 55) as f64 * VOXEL_M,
                surf_m,
                18.0,
                0.0,
                -0.45,
                "back off and up so the whole cut face and the veneer above it are in one frame",
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATION SET 3 — provenance over site climate
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n\n========== STATION SET 3 — PROVENANCE OVER SITE CLIMATE ==========");
    println!(
        "P11 slice 2b retired the deposition draw for transported deposits. The audit's\n\
         headline: some share of the record would have been NAMED DIFFERENTLY by the\n\
         deposition site's own climate. This station set asks where that reaches the surface.\n"
    );

    // (a) THE INSTRUMENT, PROVED: the aggregate audit on the shipped world.
    let t = Instant::now();
    let cfg = DeepConfig {
        identity_audit: true,
        ..production_config(&pregen.grid, SEED)
    };
    let run = run_cells(&pregen.grid, &cfg, true);
    eprintln!("audited deep run: {:.1} s", t.elapsed().as_secs_f64());
    let m = run.erosion.identity_provenance_m();
    let total = m[1] + m[2];
    println!("  (a) THE AGGREGATE AUDIT (`DeepConfig::identity_audit`, off in production):");
    println!(
        "      from the ARRIVING COMPOSITION  {:>12.1} m  {:>6.2} %",
        m[1],
        100.0 * m[1] / total.max(1e-9)
    );
    println!(
        "      from the FITNESS DRAW          {:>12.1} m  {:>6.2} %",
        m[2],
        100.0 * m[2] / total.max(1e-9)
    );
    println!(
        "      ...of the transported metres, the ones the SITE would have named differently:\n\
         \x20     {:.1} m = {:.2} % of transported, {:.2} % of all record",
        m[3],
        100.0 * m[3] / m[1].max(1e-9),
        100.0 * m[3] / total.max(1e-9)
    );

    // Self-check: the audited run's record must be the field the walk stands on.
    let mut agree = 0usize;
    let mut checked = 0usize;
    for idx in (0..w * w).step_by(97) {
        if idx >= run.grid.strata.len() {
            break;
        }
        checked += 1;
        if run.grid.strata[idx].units.len() == field.strata[idx].units.len() {
            agree += 1;
        }
    }
    println!(
        "      SELF-CHECK: audited-run record vs the pregen field the walk renders — \
         {agree}/{checked} sampled cells agree on unit count (the audit is bit-inert; a \
         mismatch here means the two runs are not the same world and every S3 coordinate \
         below is void)."
    );

    // (b) LOCALIZED: the same counterfactual, per near-surface recorded unit,
    //     under the SITE'S PRESENT climate.
    let mut mem_cache: HashMap<u8, MemberCtx<'_>> = HashMap::new();
    struct Mismatch {
        idx: usize,
        /// Metres below the top of the deep record where the unit sits.
        depth_m: f64,
        recorded: MaterialId,
        would: MaterialId,
        thickness_m: f64,
        /// True when the class the environment tag implies differs from the
        /// class the record carries — a class-level provenance signal.
        travelled_class: bool,
    }
    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut agreements: Vec<(usize, MaterialId)> = Vec::new();
    let (mut units_checked, mut units_mismatched) = (0usize, 0usize);
    let t = Instant::now();
    for idx in 0..w * w {
        if !conv.interior(idx) || field.surf[idx] <= SEA_LEVEL_M {
            continue;
        }
        if idx >= run.grid.strata.len() {
            break;
        }
        let temp_c = f64::from(climate::air_temp_c(
            run.grid.lat_deg(idx / w),
            run.grid.r[idx] + run.grid.h[idx],
        ));
        let precip = f64::from(run.grid.precip[idx]);
        // Top-down through the record; only the near-surface metres can express.
        let mut depth_m = 0.0f64;
        for u in field.strata[idx].units.iter().rev() {
            if depth_m > 30.0 {
                break;
            }
            if u.thickness_m < VOXEL_M {
                depth_m += u.thickness_m;
                continue;
            }
            units_checked += 1;
            let would = site_would_draw(
                &mut mem_cache,
                set,
                idx,
                temp_c,
                precip,
                u.species,
                u.chapter,
            );
            if would != u.species {
                units_mismatched += 1;
                mismatches.push(Mismatch {
                    idx,
                    depth_m,
                    recorded: u.species,
                    would,
                    thickness_m: u.thickness_m,
                    travelled_class: Litho::of_material(u.species) != litho_of_tag(u.tag),
                });
            } else if u.thickness_m >= VOXEL_M {
                agreements.push((idx, u.species));
            }
            depth_m += u.thickness_m;
        }
    }
    eprintln!("provenance scan: {:.1} s", t.elapsed().as_secs_f64());
    println!(
        "\n  (b) LOCALIZED — the same counterfactual per near-surface unit, under the SITE'S\n\
         \x20     PRESENT climate (the audit asks it under the DEPOSITIONAL epoch's; the\n\
         \x20     difference is stated, not assumed away — see `site_would_draw`).\n\
         \x20     {units_mismatched} of {units_checked} expressible units in the top 30 m of the\n\
         \x20     record ({:.2} %) carry a name the site's present climate would not draw,\n\
         \x20     across {} distinct land cells.",
        100.0 * units_mismatched as f64 / units_checked.max(1) as f64,
        {
            let mut ids: Vec<usize> = mismatches.iter().map(|m| m.idx).collect();
            ids.sort_unstable();
            ids.dedup();
            ids.len()
        }
    );

    // Rank: shallowest first (must reach the expression), then thickest, and
    // prefer a pair that differs VISIBLY (grain size, i.e. a coarse-clastic pair)
    // over an albedo-adjacent one.
    let visible = |a: MaterialId, b: MaterialId| -> u8 {
        let l = Litho::of_material(a);
        if l != Litho::of_material(b) {
            return 3;
        }
        match l.code() {
            "clastic-coarse" => 2, // sandstone vs conglomerate — grain size, loud
            _ => 1,                // mudstone/siltstone, granite/diorite — adjacent
        }
    };
    mismatches.sort_by(|a, b| {
        visible(b.recorded, b.would)
            .cmp(&visible(a.recorded, a.would))
            .then(a.depth_m.total_cmp(&b.depth_m))
            .then(b.thickness_m.total_cmp(&a.thickness_m))
    });

    if mismatches.is_empty() {
        println!(
            "\n  ⚠ NULL — no near-surface land unit disagrees with its site's present-climate\n\
             counterfactual. CONTROL: the aggregate audit above reports {:.1} m ({:.2} % of the\n\
             archive) that DO disagree at deposition time, so the instrument is proven on the\n\
             record; what is absent is a disagreement that survives to the near surface on land.\n\
             The walk cannot see this change. Brief the null; do not launch for it.",
            m[3],
            100.0 * m[3] / total.max(1e-9)
        );
    } else {
        let mut picked: Vec<&Mismatch> = Vec::new();
        for c in &mismatches {
            if picked.len() >= top_n {
                break;
            }
            let (vx, vz) = conv.idx_to_voxel(c.idx);
            if wgen.surface_elev_m(vx, vz) <= LAND_M {
                continue;
            }
            let mm = conv.meters(c.idx);
            if picked.iter().all(|p| km_between(conv.meters(p.idx), mm) > 5.0) {
                picked.push(c);
            }
        }
        for (rank, c) in picked.iter().enumerate() {
            let (mx, mz) = conv.meters(c.idx);
            let (vx, vz) = conv.idx_to_voxel(c.idx);
            let surf_m = wgen.surface_elev_m(vx, vz);
            let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
            println!(
                "\n  S3 #{}  world ({mx:.0} m, {mz:.0} m)  deep cell ({},{})  voxel ({vx}, {vz})  \
                 chunk ({cx}, {cz})",
                rank + 1,
                c.idx % w,
                c.idx / w
            );
            println!("    near-field surface elevation {surf_m:.1} m");
            println!(
                "    RANKING QUANTITY — recorded {} vs the site's present-climate counterfactual {} \
                 ({} contrast); {:.2} m thick, {:.2} m below the top of the deep record",
                short(c.recorded),
                short(c.would),
                match visible(c.recorded, c.would) {
                    3 => "CROSS-CLASS",
                    2 => "grain-size (loud)",
                    _ => "albedo-adjacent (subtle)",
                },
                c.thickness_m,
                c.depth_m
            );
            println!(
                "    class the environment tag implies vs the class recorded: {}",
                if c.travelled_class {
                    "DIFFER — the load's class outvoted the site's environment (class-level provenance)"
                } else {
                    "same class; the disagreement is at MEMBER grade"
                }
            );
            // Does it reach the expression, and at what depth?
            let col = wgen.column_record(cx, cz);
            let fill = ColumnFill::build(&col.strata, VOXEL_M);
            let mut found: Option<u32> = None;
            for d in 1..=fill.depth_count() as u32 {
                if let Some(mem) = expressed_member(set, &fill, &col.strata.events, d, vx, vz)
                    && set.member(mem).material == c.recorded
                {
                    found = Some(d);
                    break;
                }
            }
            match found {
                Some(1) => println!(
                    "    EXPRESSION: the recorded rock is the SURFACE VOXEL (plan(1)) — visible with no cut."
                ),
                Some(d) => println!(
                    "    EXPRESSION: first expressed at plan({d}) = {:.1} m below the surface — needs a bench cut {} voxels deep.",
                    (d as f64 - 1.0) * VOXEL_M,
                    d
                ),
                None => println!(
                    "    ⚠ EXPRESSION: this recorded rock does NOT appear anywhere in the expressed\n\
                       column at this voxel — the veneer or the quantizer buried it. This station is\n\
                       record-only and the walk cannot see it."
                ),
            }
            let sy = (surf_m / VOXEL_M).floor() as i64;
            let cut = found.map(|d| (d as i64 + 4).min(24)).unwrap_or(16);
            println!(
                "    BENCH CUT (voxels): dc:air  x [{}, {}]  y [{}, {}]  z [{}, {}] = {} voxels",
                vx - 30,
                vx + 30,
                sy - cut + 1,
                sy,
                vz - 20,
                vz,
                61 * 21 * cut
            );
            pose(
                "provenance face",
                mx,
                (vz - 40) as f64 * VOXEL_M,
                surf_m,
                3.0,
                0.0,
                -0.15,
                "eye height ~18 m south of the cut, looking NORTH at the face; --fullbright",
            );
        }

        // The contrast: the nearest cell where recorded == counterfactual, same class.
        if let Some(anchor) = picked.first() {
            let a = conv.meters(anchor.idx);
            let want = Litho::of_material(anchor.recorded);
            let mut same: Vec<&(usize, MaterialId)> = agreements
                .iter()
                .filter(|(_, mat)| Litho::of_material(*mat) == want)
                .collect();
            same.sort_by(|x, y| {
                km_between(conv.meters(x.0), a).total_cmp(&km_between(conv.meters(y.0), a))
            });
            println!("\n  CONTRAST (recorded == the site's own counterfactual, same class):");
            match same.first() {
                None => println!(
                    "    none found in the same class — the contrast pair does not exist for {}.",
                    want.code()
                ),
                Some((idx, mat)) => {
                    let (bx, bz) = conv.meters(*idx);
                    let (vx, vz) = conv.idx_to_voxel(*idx);
                    let surf_m = wgen.surface_elev_m(vx, vz);
                    println!(
                        "    world ({bx:.0} m, {bz:.0} m), surface {surf_m:.1} m, recorded {} — \
                         {:.1} km from S3 #1",
                        short(*mat),
                        km_between((bx, bz), a)
                    );
                    pose(
                        "contrast",
                        bx,
                        (vz - 40) as f64 * VOXEL_M,
                        surf_m,
                        3.0,
                        0.0,
                        -0.15,
                        "same framing as S3 #1: here the rock IS what this place would make",
                    );
                }
            }
        }
    }

    // ─────────────────────── inter-station distances ────────────────────────
    println!("\n\n========== INTER-STATION DISTANCES (one launch plans them all) ==========");
    let s1 = movable.first().map(|s| conv.meters(s.idx));
    let s2 = alt_rank.first().map(|c| conv.meters(c.1));
    let s3 = mismatches.first().map(|c| conv.meters(c.idx));
    for (an, a) in [("S1", s1), ("S2", s2), ("S3", s3)] {
        for (bn, b) in [("S1", s1), ("S2", s2), ("S3", s3)] {
            if an < bn
                && let (Some(a), Some(b)) = (a, b)
            {
                println!("  {an} ↔ {bn}: {:.1} km", km_between(a, b));
            }
        }
    }
    println!("  (teleports are free; the distances are for briefing, not for travel.)");

    println!("\ntotal probe wall-clock {:.1} s", t0.elapsed().as_secs_f64());
}

/// **The gate's view of this instrument.**
///
/// A tour map's *stations* are recommendations judged by the eye and are
/// deliberately not gated (the precedent `palette_quant_tour` and
/// `walk_tour_0115` set). What IS gated is the part that can be silently wrong
/// while still printing a confident coordinate: the two **classifiers** that
/// decide which cell the walk is sent to.
#[cfg(test)]
mod gate {
    use super::*;
    use dc_worldgen::deeptime::{DepEnv, DepTag, DepUnit, EnergyBand};
    use dc_worldgen::deeptime::recorder::Aridity;

    fn unit(mat: MaterialId, t: f64) -> DepUnit {
        DepUnit {
            tag: DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::Medium),
            thickness_m: t,
            unconformity: false,
            chapter: 0,
            species: mat,
        }
    }

    /// **`member_alternations` must count WITHIN-class changes and only those.**
    ///
    /// It is station set 2's whole ranking quantity, and the claim P11 slice 1
    /// makes is specifically about members *inside* a class the record used to
    /// collapse. A mudstone→sandstone contact is a class change: real, visible,
    /// and **not evidence for this slice**. If this classifier drifts, the tour
    /// sends the walk to an ordinary lithological contact and the walk "confirms"
    /// a change that was always there.
    ///
    /// **Scale-free**: it is a per-column arithmetic over adjacent recorded
    /// bands. No world, no extent, no run length enters it.
    #[test]
    fn only_within_class_member_changes_are_counted_as_alternations() {
        let mut s = DeepStrata::default();
        // mudstone / siltstone / mudstone — two within-class alternations.
        s.units = vec![
            unit(MaterialId::MUDSTONE, 4.0),
            unit(MaterialId::SILTSTONE, 3.0),
            unit(MaterialId::MUDSTONE, 2.0),
        ];
        let (alt, bands) = member_alternations(&s);
        assert_eq!(bands.len(), 3, "the three bands must survive coalescing");
        assert_eq!(alt, 2, "mudstone/siltstone/mudstone is two alternations");

        // A class change must NOT count.
        s.units = vec![
            unit(MaterialId::MUDSTONE, 4.0),
            unit(MaterialId::SANDSTONE, 3.0),
        ];
        assert_eq!(
            member_alternations(&s).0,
            0,
            "a clastic-fine → clastic-coarse contact is a CLASS change and is not this slice's claim"
        );

        // A band thinner than a voxel cannot express and must not be counted.
        s.units = vec![
            unit(MaterialId::MUDSTONE, 4.0),
            unit(MaterialId::SILTSTONE, 0.2),
            unit(MaterialId::MUDSTONE, 4.0),
        ];
        assert_eq!(
            member_alternations(&s).0,
            0,
            "a 0.2 m band cannot fill a 0.9 m voxel — scoring it sends the walk to an invisible contact"
        );

        // Adjacent same-material units coalesce rather than counting as a contact.
        s.units = vec![
            unit(MaterialId::MUDSTONE, 2.0),
            unit(MaterialId::MUDSTONE, 2.0),
        ];
        let (alt, bands) = member_alternations(&s);
        assert_eq!((alt, bands.len()), (0, 1), "one rock is one band");
    }

    /// **The `TopSpan` classifier must separate MOVABLE from FROZEN.**
    ///
    /// This is the whole of station set 1's filter, and getting it wrong is the
    /// exact failure the old U3 pose already demonstrated: a `Mixed` top span is
    /// structurally blind to the member dither, and so is a `dither: false`
    /// deep-history event. A tour that scores those as stations sends the walk to
    /// ground where the shipped change cannot appear at any magnitude.
    ///
    /// **Scale-free**: a per-column classification of one `Plan` against one
    /// event's flags and its class's member count.
    #[test]
    fn the_top_span_classifier_only_calls_a_veneer_multi_member_single_movable() {
        let set = vanilla();
        let mud = set
            .member_index("dc:geo/mudstone")
            .expect("vanilla registers mudstone");
        let coal = set
            .member_index("dc:geo/coal")
            .expect("vanilla registers coal");
        assert!(
            class_members(&set, mud) > 1,
            "clastic-fine must stay a multi-member class for this test to mean anything"
        );
        assert_eq!(
            class_members(&set, coal),
            1,
            "organic-coal must stay a one-member class for this test to mean anything"
        );

        let ev = |member, dither| StrataEvent {
            member,
            thickness_m: 5.0,
            temp_c: 15.0,
            precip: 0.6,
            depth_m: 1.0,
            sel_salt: 0x5700_000E,
            sel_tag: 3,
            ore: None,
            accessory: None,
            dither,
        };

        // A veneer event in a two-member class: MOVABLE.
        let mut rec = dc_worldgen::StrataRec::default();
        rec.events = vec![ev(mud, true)];
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(
            top_span(&set, &fill, &rec.events).0,
            TopSpan::SingleMovable
        );

        // The same class, but a RECORDED identity (`dither: false`): frozen.
        rec.events = vec![ev(mud, false)];
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(
            top_span(&set, &fill, &rec.events).0,
            TopSpan::SingleFrozen,
            "a deep-history event's member is recorded and expression must not be scored as \
             able to move it"
        );

        // A veneer event in a ONE-member class: frozen, because the dither has
        // nothing to pick between.
        rec.events = vec![ev(coal, true)];
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(
            top_span(&set, &fill, &rec.events).0,
            TopSpan::SingleFrozen,
            "a one-member class is a constant field under any dither source"
        );

        // Two events sharing the surface voxel: MIXED, which `mixed_at` resolves
        // from the UNDITHERED member — blind to this slice by design.
        rec.events = vec![ev(mud, true), ev(coal, true)];
        rec.events[0].thickness_m = 0.5;
        rec.events[1].thickness_m = 0.5;
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(top_span(&set, &fill, &rec.events).0, TopSpan::Mixed);
    }
}
