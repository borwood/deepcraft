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

fn top_span(
    set: &GeologySet,
    fill: &ColumnFill,
    events: &[StrataEvent],
) -> (TopSpan, Option<usize>) {
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

/// One expressed voxel: the member that fills it, and **whether the member got
/// there by the octaves dither or by the record**. The second half is what keeps
/// station set 2 honest — a member contact drawn by the veneer's per-column
/// dither is *not* evidence for P11's recorded diversity, and the two look
/// identical on a cut face.
#[derive(Clone, Copy)]
struct Expressed {
    member: GeoMemberIdx,
    /// `Plan::Single` on a `dither: true` event in a ≥2-member class — the only
    /// shape the octaves slice can move.
    movable: bool,
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
) -> Option<Expressed> {
    match fill.plan(d)? {
        Plan::Single(k) => {
            let e = events[*k];
            Some(Expressed {
                member: dithered_member_with(
                    set,
                    &e,
                    &dc_worldgen::geology::selection_field(SEED, e.sel_salt),
                    vx,
                    vz,
                ),
                movable: e.dither && class_members(set, e.member) > 1,
            })
        }
        Plan::Mixed(w) => w
            .iter()
            .max_by_key(|(_, eighths)| *eighths)
            .map(|(k, _)| Expressed {
                member: events[*k].member,
                movable: false,
            }),
    }
}

/// Walk a column's expressed voxels and split its within-class member contacts
/// into the two that must never be confused:
///
/// - **recorded** — both sides came from `dither: false` deep-history events, so
///   the contact is a fact deep time wrote down (P11 slices 1/2b);
/// - **dithered** — at least one side is a movable veneer span, so the contact is
///   the octaves selection field drawing a patch boundary (journal/0128–0129).
///
/// Returns `(recorded, dithered, movable spans, runs)`, where a run is
/// `(material name, first depth, last depth, movable)`.
type ColumnRuns = Vec<(String, u32, u32, bool)>;
fn expressed_column(
    set: &GeologySet,
    fill: &ColumnFill,
    events: &[StrataEvent],
    vx: i64,
    vz: i64,
) -> (usize, usize, usize, ColumnRuns) {
    let (mut recorded, mut dithered, mut movable_spans) = (0usize, 0usize, 0usize);
    let mut runs: ColumnRuns = Vec::new();
    let mut prev: Option<Expressed> = None;
    for d in 1..=fill.depth_count() as u32 {
        let Some(e) = expressed_member(set, fill, events, d, vx, vz) else {
            continue;
        };
        if e.movable {
            movable_spans += 1;
        }
        let name = short(set.member(e.member).material).to_string();
        match runs.last_mut() {
            Some((n, _, hi, mv)) if *n == name && *mv == e.movable => *hi = d,
            _ => runs.push((name, d, d, e.movable)),
        }
        if let Some(p) = prev
            && p.member != e.member
            && set.member(p.member).class == set.member(e.member).class
        {
            if p.movable || e.movable {
                dithered += 1;
            } else {
                recorded += 1;
            }
        }
        prev = Some(e);
    }
    (recorded, dithered, movable_spans, runs)
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
    counts.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
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
fn coalesced_bands(strata: &DeepStrata) -> Vec<Band> {
    let mut bands: Vec<Band> = Vec::new();
    for u in &strata.units {
        if u.thickness_m() <= 0.0 {
            continue;
        }
        match bands.last_mut() {
            Some(b) if b.mat == u.species() => b.thickness_m += u.thickness_m(),
            _ => bands.push(Band {
                mat: u.species(),
                thickness_m: u.thickness_m(),
            }),
        }
    }
    bands
}

fn member_alternations(strata: &DeepStrata) -> (usize, Vec<Band>) {
    // Coalesce adjacent same-material units, bottom-up (the record's own order).
    let thick: Vec<Band> = coalesced_bands(strata)
        .into_iter()
        .filter(|b| b.thickness_m >= VOXEL_M)
        .collect();
    let mut alt = 0usize;
    for w in thick.windows(2) {
        if w[0].mat != w[1].mat && Litho::of_material(w[0].mat) == Litho::of_material(w[1].mat) {
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

/// **How loud is the difference between two rocks?** `3` cross-class, `2` a
/// grain-size pair (sandstone vs conglomerate — the loudest thing the vanilla set
/// has inside a class), `1` albedo-adjacent (mudstone/siltstone,
/// granite/diorite, basalt/andesite: journal/0129's finding that a within-class
/// member dither's *ceiling* is a subtle contrast).
///
/// ⚠ `Litho::code()` is `"coarse"` / `"fine"`, **not** the namespaced class id.
/// Matching on the string `"clastic-coarse"` here scored every sandstone-vs-
/// conglomerate station as albedo-adjacent on this probe's first run — the exact
/// inverse of the truth. Match the enum.
fn pair_visibility(a: MaterialId, b: MaterialId) -> u8 {
    let l = Litho::of_material(a);
    if l != Litho::of_material(b) {
        return 3;
    }
    match l {
        Litho::ClasticCoarse => 2,
        _ => 1,
    }
}

fn visibility_label(v: u8) -> &'static str {
    match v {
        3 => "CROSS-CLASS",
        2 => "grain-size (loud)",
        _ => "albedo-adjacent (subtle)",
    }
}

/// **The square-edge signature** (journal/0129, `palette_quant_tour`'s metric,
/// re-derived here because an example is its own crate root and a sibling cannot
/// be called): how much of a selection field's curvature sits on the 28.8 m chunk
/// lattice. A bilinear field is linear inside its own cell, so a single octave at
/// chunk wavelength puts **all** its curvature on the 32-voxel grid; that
/// concentration IS the defect the octaves slice removed. Returns
/// `(mean |Δ²u| on the lattice, off it)` — "on" is phases 0 and 31, because
/// `x + 1` is in the next cell.
fn lattice_curvature(src: &dyn DitherSource, cx: i64, cz: i64, tag: u64, half: i64) -> (f64, f64) {
    let (mut on, mut on_n, mut off, mut off_n) = (0.0f64, 0u64, 0.0f64, 0u64);
    for z in (cz - half)..(cz + half) {
        for x in (cx - half)..(cx + half) {
            let d2 = (src.uniform(x - 1, z, tag) - 2.0 * src.uniform(x, z, tag)
                + src.uniform(x + 1, z, tag))
            .abs();
            if (x - 1).div_euclid(32) != (x + 1).div_euclid(32) {
                on += d2;
                on_n += 1;
            } else {
                off += d2;
                off_n += 1;
            }
        }
    }
    (on / on_n.max(1) as f64, off / off_n.max(1) as f64)
}

fn short(m: MaterialId) -> &'static str {
    // `qualified_name` is `dc:mudstone` etc.; the tail is what a briefing reads.
    let q = m.qualified_name();
    q.rsplit(':').next().unwrap_or(q)
}

/// **Yaw is Bevy's: view dir at yaw θ is `(−sin θ, ·, −cos θ)`**
/// (`dc-client::player.rs:57`). So **yaw 0 looks toward −Z** and **yaw π toward
/// +Z**. Getting this backwards points the camera at the intact hillside behind
/// the player instead of at the cut face — the pose-recording hazard of
/// corrections #48 with a sign instead of a distance.
const YAW_NORTH: f64 = std::f64::consts::PI;
const YAW_SOUTH: f64 = 0.0;

fn pose(label: &str, x: f64, y: f64, z: f64, yaw: f64, pitch: f64, why: &str) {
    println!(
        "    POSE {label}: pose_set{{ x: {x:.0}, z: {z:.0}, surface: true }}  → read the live \
         ground Y, then  pose_set{{ x: {x:.0}, y: {y:.0}, z: {z:.0}, yaw: {yaw:.3}, pitch: {pitch:.2} }}"
    );
    println!(
        "      ({why}; y {y:.0} m is the WORLDGEN frame — the live client frame can carry a \
         constant offset, so always teleport with surface:true first)"
    );
}

/// A road-cut bench, in **voxels** (`world_fill` speaks voxels; poses speak
/// metres). Chunk-aligned in x so the face is one chunk's record and cannot be
/// confounded by the per-chunk `StrataRec` stepping journal/0129 names as a third
/// mechanism at this site. Returns `(x0, x1, y0, y1, z0, z1, face_z_voxel)`.
#[allow(clippy::too_many_arguments)]
fn bench(vx: i64, vz: i64, surf_m: f64, cut: i64) -> (i64, i64, i64, i64, i64, i64, i64) {
    let cx = vx.div_euclid(32);
    let sy = (surf_m / VOXEL_M).floor() as i64;
    // 32 (one chunk) × 41 × cut voxels. cut = 16 → 20,992, the skill's ~20k.
    (cx * 32, cx * 32 + 31, sy - cut + 1, sy, vz - 40, vz, vz + 1)
}

fn print_bench(vx: i64, vz: i64, surf_m: f64, cut: i64, mx: f64) {
    // **The bench floor must stay above sea level.** A cut deeper than the ground
    // is high floods, and a flooded bench is a null frame with a confident pose
    // attached — the S2 #2 station asked for 24 voxels on 16.7 m of ground and
    // would have put the player at y = −4.9 m.
    let cut = cut.min(((surf_m - 2.0) / VOXEL_M).floor().max(3.0) as i64);
    let (x0, x1, y0, y1, z0, z1, face) = bench(vx, vz, surf_m, cut);
    let n = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1);
    println!(
        "    BENCH CUT — `world_fill` dc:air  x [{x0}, {x1}]  y [{y0}, {y1}]  z [{z0}, {z1}]  \
         = {n} voxels (VOXEL coords)"
    );
    println!(
        "      one chunk wide in x (chunk {}), 41 voxels of approach in z, {cut} voxels \
         ({:.1} m) of face. The EXPOSED FACE is the intact wall at voxel z = {face} \
         ({:.0} m); stand on the bench floor and look NORTH at it.",
        vx.div_euclid(32),
        cut as f64 * VOXEL_M,
        face as f64 * VOXEL_M
    );
    // Stand ~19 m back from the face, on the bench floor.
    let stand_z = (vz - 20) as f64 * VOXEL_M;
    let floor_m = y0 as f64 * VOXEL_M;
    pose(
        "at the face",
        mx,
        floor_m + 1.7,
        stand_z,
        YAW_NORTH,
        -0.05,
        "eye height on the bench floor, ~19 m back from the face — the whole cut fills the frame. \
         --fullbright (material question), NO --edges (never diff colour on an edges frame)",
    );
    pose(
        "the face from above",
        mx,
        surf_m + 14.0,
        (vz - 45) as f64 * VOXEL_M,
        YAW_NORTH,
        -0.5,
        "back off past the bench lip and up 14 m: the cut face and the intact veneer above it \
         in one frame",
    );
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
        /// The movable event, when there is one — at the SURFACE (`plan(1)`) or,
        /// for the buried arm, the shallowest movable span in the column.
        event: Option<StrataEvent>,
        /// Depth index of `event` (1 = surface voxel).
        event_depth: u32,
        /// Ranking quantity for station 1: minority member share in the window.
        minority: f64,
        distinct: usize,
        /// Within-class member contacts in the EXPRESSED column, split by cause.
        expr_recorded: usize,
        expr_dithered: usize,
        /// Voxel spans in this column the octaves dither can move.
        movable_spans: usize,
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
            let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
            let col = wgen.column_record(cx, cz);
            // P11 slice 3: the record is per column — read the voxel's OWN
            // column's SubCell, which is exactly what expression renders here.
            let Some(sub) = col.record_for(lx, lz) else {
                span_counts[0] += 1;
                continue;
            };
            let fill = sub.fill();
            let (span, k) = top_span(set, fill, &sub.strata().events);
            span_counts[match span {
                TopSpan::None => 0,
                TopSpan::Mixed => 1,
                TopSpan::SingleMovable => 2,
                TopSpan::SingleFrozen => 3,
            }] += 1;

            // Station 2's verified half AND station 1's buried arm, from one
            // walk of the expressed column.
            let (expr_recorded, expr_dithered, movable_spans, _) =
                expressed_column(set, fill, &sub.strata().events, vx, vz);

            // Station 1's event: the SURFACE one where the top span is movable,
            // else the SHALLOWEST movable span in the column — which is a station
            // a bench cut can reach even when the surface cannot show it.
            let mut event = k
                .filter(|_| span == TopSpan::SingleMovable)
                .map(|k| (sub.strata().events[k], 1u32));
            if event.is_none() && movable_spans > 0 {
                for d in 2..=fill.depth_count() as u32 {
                    if let Some(Plan::Single(k)) = fill.plan(d) {
                        let e = sub.strata().events[*k];
                        if e.dither && class_members(set, e.member) > 1 {
                            event = Some((e, d));
                            break;
                        }
                    }
                }
            }
            // Ranking quantity — only meaningful where there is a movable event.
            // A 32-voxel half-window is the cheap screen; the top picks get the
            // full 128-voxel window below.
            let (minority, distinct) = match event {
                Some((e, _)) => {
                    let src = Octaves::member(Draws::from_recorded_salt(SEED, e.sel_salt));
                    let (d, m, _) = member_window(set, &e, &src, vx, vz, 32);
                    (m, d)
                }
                None => (0.0, 0),
            };

            samples.push(Sample {
                idx,
                vx,
                vz,
                surf_m,
                span,
                event: event.map(|(e, _)| e),
                event_depth: event.map_or(0, |(_, d)| d),
                minority,
                distinct,
                expr_recorded,
                expr_dithered,
                movable_spans,
                expr_depth: fill.depth_count(),
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
         and deep-history events carry `dither:false`.)"
    );
    // **The buried arm, and the positive control for station set 1's null.**
    // A `Plan::Single` on a movable veneer event is the only shape the octaves
    // slice can move, and the surface is not the only place it occurs. If this
    // count is zero too, the classifier is suspect; if it is non-zero, a zero at
    // the surface is a fact about the surface (CLAUDE.md § tour-map: a zero from
    // an unproven census is not evidence).
    let buried_movable = samples.iter().filter(|s| s.movable_spans > 0).count();
    let movable_span_total: usize = samples.iter().map(|s| s.movable_spans).sum();
    println!(
        "  CONTROL — the same classifier BELOW the surface: {buried_movable} of {} land columns \
         ({:.1} %) hold at least one movable `Single` span, {movable_span_total} spans in all.",
        samples.len(),
        100.0 * buried_movable as f64 / samples.len().max(1) as f64
    );
    // The expression-verified rate for station set 2, split by CAUSE. A contact
    // drawn by the veneer's per-column dither is not evidence for P11's recorded
    // diversity, and on a cut face the two are indistinguishable — so they are
    // counted apart here rather than summed into one flattering number.
    let with_recorded = samples.iter().filter(|s| s.expr_recorded > 0).count();
    let with_dithered = samples.iter().filter(|s| s.expr_dithered > 0).count();
    let deepest = samples.iter().map(|s| s.expr_depth).max().unwrap_or(0);
    let mean_depth =
        samples.iter().map(|s| s.expr_depth).sum::<usize>() as f64 / samples.len().max(1) as f64;
    println!(
        "  expressed within-class member contacts, by cause:  RECORDED (both sides `dither:false`, \
         P11's claim) {with_recorded} columns ({:.1} %)  ·  DITHERED (a veneer span on one side) \
         {with_dithered} columns ({:.1} %)",
        100.0 * with_recorded as f64 / samples.len().max(1) as f64,
        100.0 * with_dithered as f64 / samples.len().max(1) as f64
    );
    println!("  recorded column depth: mean {mean_depth:.1} voxels, max {deepest}\n");

    // ═══════════════════════════════════════════════════════════════════════
    // STATION SET 1 — the octaves near-member stepping
    // ═══════════════════════════════════════════════════════════════════════
    println!("========== STATION SET 1 — OCTAVES NEAR-MEMBER STEPPING ==========");
    println!(
        "The standing U3-dominance question (ROADMAP § Observed). The old U3 pose has a\n\
         `Mixed` top span and is structurally blind; this looks for `Single` in a two-member\n\
         class, on land, where the octave structure has visible spatial extent.\n"
    );
    // The brief's station: a movable span AT THE SURFACE. The buried arm is the
    // fallback, and the two are never merged — one is walkable without a shovel
    // and the other is not.
    let surface_movable = samples
        .iter()
        .filter(|s| s.span == TopSpan::SingleMovable && s.distinct > 1)
        .count();
    if surface_movable == 0 {
        println!(
            "  ⚠ SURFACE NULL — 0 of {land_sampled} sampled land columns express `Plan::Single`\n\
             on a movable (`dither:true`, >=2-member) event at the SURFACE VOXEL. The surface\n\
             top span is `Mixed` on {} of them ({:.1} %), and `mixed_at` resolves a Mixed span\n\
             from the UNDITHERED `event.member` by design — so the octaves slice cannot move\n\
             the ground you stand on ANYWHERE on this world, not just at the old U3 pose.\n\
             The old pose was not an unlucky pick; it was the universal case.",
            span_counts[1],
            100.0 * span_counts[1] as f64 / land_sampled.max(1) as f64
        );
        println!(
            "  INSTRUMENT PROVED, not assumed: the identical classifier finds {buried_movable} \
             columns\n  with a movable span BELOW the surface ({movable_span_total} spans). A zero \
             from an unproven\n  census is not evidence — this one has its positive control on the \
             same sample."
        );
    }
    // **The loudness a movable span can reach is a property of its CLASS**, and
    // journal/0129's first finding is that the ceiling is low: the four two-member
    // classes are mudstone/siltstone, sandstone/conglomerate, granite/diorite and
    // basalt/andesite, and only the second is a grain-size contrast. Rank by that
    // first — a perfectly balanced basalt-vs-andesite patchwork is a station the
    // eye cannot adjudicate, and sending the walk there wastes the session.
    let span_class = |s: &Sample| -> u8 {
        s.event.map_or(0, |e| {
            let cls = set.class(set.member(e.member).class.as_str());
            match cls.map(|c| c.members()) {
                Some(ms) if ms.len() > 1 => {
                    pair_visibility(set.member(ms[0]).material, set.member(ms[1]).material)
                }
                _ => 0,
            }
        })
    };
    let mut class_tally: HashMap<&str, usize> = HashMap::new();
    for s in samples
        .iter()
        .filter(|s| s.event.is_some() && s.distinct > 1)
    {
        *class_tally
            .entry(set.member(s.event.expect("filtered").member).class.as_str())
            .or_insert(0) += 1;
    }
    let mut movable: Vec<&Sample> = samples
        .iter()
        .filter(|s| s.event.is_some() && s.distinct > 1)
        .collect();
    movable.sort_by(|a, b| {
        // Surface spans first (walkable without a cut), then by how LOUD the
        // class's member pair can be, then by how balanced the draw is over the
        // window, then by how shallow the span is.
        (a.event_depth == 1)
            .cmp(&(b.event_depth == 1))
            .reverse()
            .then(span_class(b).cmp(&span_class(a)))
            .then(b.minority.total_cmp(&a.minority))
            .then(a.event_depth.cmp(&b.event_depth))
    });
    if !class_tally.is_empty() {
        let mut rows: Vec<(&&str, &usize)> = class_tally.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        println!("\n  which CLASSES carry the movable spans (the dither's expressive ceiling):");
        for (cls, n) in rows {
            let pair = set
                .class(cls)
                .map(|c| c.members())
                .filter(|m| m.len() > 1)
                .map(|m| {
                    let (a, b) = (set.member(m[0]).material, set.member(m[1]).material);
                    format!(
                        "{} vs {} — {}",
                        short(a),
                        short(b),
                        visibility_label(pair_visibility(a, b))
                    )
                })
                .unwrap_or_else(|| "?".into());
            println!("    {n:>6} columns  {cls}  [{pair}]");
        }
    }

    if movable.is_empty() {
        println!(
            "\n  ⚠ FULL NULL — no sampled land column holds a movable span at ANY depth whose\n\
             window draws both members. Nothing to launch for; the walk cannot see this slice."
        );
    } else {
        println!(
            "\n  DISTRIBUTION (surface + buried): {} of {land_sampled} sampled land columns \
             ({:.1} %) hold\n  a movable span whose 115 m window draws BOTH members. Minority \
             share p50 {:.3}, max {:.3}.\n  Of those, {} are at the SURFACE voxel; the rest need \
             a cut.",
            movable.len(),
            100.0 * movable.len() as f64 / land_sampled.max(1) as f64,
            movable[movable.len() / 2].minority,
            movable[0].minority,
            movable.iter().filter(|s| s.event_depth == 1).count()
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
            if s.event_depth == 1 {
                println!("    the movable span is the SURFACE VOXEL — no cut needed.");
            } else {
                println!(
                    "    ⚠ BURIED: the movable span is `plan({})` = {:.1} m below the surface. \
                     The surface here is `{:?}` and CANNOT show the slice; this station needs a \
                     bench cut {} voxels deep.",
                    s.event_depth,
                    (s.event_depth as f64 - 1.0) * VOXEL_M,
                    s.span,
                    s.event_depth
                );
            }
            println!(
                "    event: class {}  ({} registered members)  sel_salt {:#x} tag {}",
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
            println!(
                "\n      distinct {da}, minority share {ma:.3}   [{}]",
                visibility_label(span_class(s))
            );
            println!(
                "    A/B at this site — BEFORE (single octave, stride 32): distinct {db}, minority {mb:.3}"
            );
            // The journal/0129 headline, re-measured here rather than remembered.
            let (b_on, b_off) = lattice_curvature(&before, s.vx, s.vz, e.sel_tag, WIN_VOX);
            let (a_on, a_off) = lattice_curvature(&after, s.vx, s.vz, e.sel_tag, WIN_VOX);
            println!(
                "    SQUARE-EDGE SIGNATURE |Δ²u| on the 28.8 m lattice vs off it:\n\
                 \x20     BEFORE on {b_on:.3e} off {b_off:.3e}  ratio {:.3e}\n\
                 \x20     AFTER  on {a_on:.3e} off {a_off:.3e}  ratio {:.3}   \
                 (≈1 means the chunk grid is no longer special)",
                b_on / b_off.max(f64::MIN_POSITIVE),
                a_on / a_off
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
            if s.event_depth == 1 {
                pose(
                    "eyes-down",
                    mx,
                    s.surf_m + 12.0,
                    mz,
                    YAW_SOUTH,
                    -0.9,
                    "12 m up, pitched steeply down: a ~115 m patch of ground fills the frame, \
                     which is the window the shares above were measured over. --fullbright \
                     (material question), NO --edges",
                );
                pose(
                    "standing",
                    mx,
                    s.surf_m + 1.7,
                    mz,
                    YAW_SOUTH,
                    -0.35,
                    "eye height — the ground the octaves selection field draws under your feet",
                );
            } else {
                print_bench(
                    s.vx,
                    s.vz,
                    s.surf_m,
                    (s.event_depth as i64 + 4).clamp(8, 24),
                    mx,
                );
            }
        }

        // **The contrast is FREE at every one of these stations, and that is the
        // finding.** The `Mixed` top span the dither cannot touch is not somewhere
        // else on the map — it is the ground directly above the cut, on 99.1 % of
        // land. Climb out of the bench and you are standing on it.
        let anchor = conv.meters(picked[0].idx);
        let mixed_pct = 100.0 * span_counts[1] as f64 / land_sampled.max(1) as f64;
        println!("\n  CONTRAST (the `Mixed` top span the octaves dither cannot touch):");
        println!(
            "    NO TRAVEL NEEDED. The surface above every station above is `Mixed` — that is \
             {mixed_pct:.1} % of\n    sampled land. Climb out of the bench, look at the ground, \
             and that IS the contrast:\n    a surface whose material varies only by the per-voxel \
             EIGHTH allocation, beside a cut\n    face whose material varies by the octaves \
             selection field."
        );
        pose(
            "contrast — the surface above S1 #1",
            anchor.0,
            picked[0].surf_m + 12.0,
            anchor.1,
            YAW_SOUTH,
            -0.9,
            "12 m up over the same coordinates, pitched down",
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATION SET 2 — member diversity in the deep record (the bench cut)
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n\n========== STATION SET 2 — MEMBER DIVERSITY IN THE DEEP RECORD ==========");
    println!(
        "P11 slices 1 + 2b. **Ranked on the EXPRESSION, not on the record** — the number that\n\
         matters is how many within-class member contacts survive the 0.9 m quantizer onto a\n\
         cut face, with both sides `dither:false` so the contact is a recorded fact and not\n\
         the veneer's own selection field drawing a patch edge.\n"
    );
    // The record-side census, kept as the CONSERVATIVE reading and reported
    // beside the expressed one because the two disagree by two orders of
    // magnitude — and the disagreement is a real mechanism, not an error. A band
    // thinner than a voxel still wins eighths inside a `Plan::Mixed` span and can
    // be the dominant share at some depth, so the expression shows contacts the
    // "band >= 0.9 m" record filter refuses to count. Both numbers are true of
    // different questions: this one is "thick, unambiguous banding in the
    // archive"; the expressed one is "what a face shows".
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
    alt_rank.sort_by_key(|(alt, _, _)| std::cmp::Reverse(*alt));
    eprintln!("record scan: {:.1} s", t.elapsed().as_secs_f64());

    let mut s2: Vec<&Sample> = samples.iter().filter(|s| s.expr_recorded > 0).collect();
    s2.sort_by(|a, b| {
        b.expr_recorded
            .cmp(&a.expr_recorded)
            .then(a.expr_dithered.cmp(&b.expr_dithered))
    });
    println!(
        "  DISTRIBUTION (expressed, the walk's own instrument): {} of {} sampled land columns \
         ({:.1} %)\n  show at least one RECORDED within-class member contact on a cut face. \
         Max {} contacts in one column.",
        s2.len(),
        samples.len(),
        100.0 * s2.len() as f64 / samples.len().max(1) as f64,
        s2.first().map_or(0, |s| s.expr_recorded)
    );
    println!(
        "  DISTRIBUTION (record-side, conservative): {cells_with_alt} of {land_cells} interior \
         LAND deep cells\n  ({:.3} %) stack two members of one class in bands each >= 0.9 m. \
         Max {} alternations.\n  The 2-orders-of-magnitude gap is the quantizer, not an error — \
         see the comment in the source.",
        100.0 * cells_with_alt as f64 / land_cells.max(1) as f64,
        alt_rank.first().map(|a| a.0).unwrap_or(0)
    );
    if s2.is_empty() {
        println!(
            "  ⚠ NULL — no sampled land column expresses a recorded within-class member contact.\n\
             CONTROL: the record-side census above found {cells_with_alt} cells that DO stack two \
             members\n  of one class in the archive, so the instrument reads the record; what is \
             absent is\n  survival through expression. Brief the null; the walk cannot see this."
        );
    } else {
        let mut picked: Vec<&&Sample> = Vec::new();
        for c in &s2 {
            if picked.len() >= top_n {
                break;
            }
            let m = conv.meters(c.idx);
            // 20 km apart: at 5 km the three picks all landed in one basin on a
            // single grid column, which is a distribution the walk cannot read.
            if picked
                .iter()
                .all(|p| km_between(conv.meters(p.idx), m) > 20.0)
            {
                picked.push(c);
            }
        }
        for (rank, s) in picked.iter().enumerate() {
            let idx = &s.idx;
            let (mx, _mz) = conv.meters(*idx);
            let (vx, vz) = (s.vx, s.vz);
            let (alt, bands) = member_alternations(&field.strata[*idx]);
            let surf_m = wgen.surface_elev_m(vx, vz);
            let (cx, cz) = (vx.div_euclid(32), vz.div_euclid(32));
            let (lx, lz) = (vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
            let col = wgen.column_record(cx, cz);
            println!(
                "\n  S2 #{}  world ({mx:.0} m, {:.0} m)  deep cell ({},{})  voxel ({vx}, {vz})  \
                 chunk ({cx}, {cz})",
                rank + 1,
                _mz,
                idx % w,
                idx / w
            );
            println!("    near-field surface elevation {surf_m:.1} m");
            // P11 slice 3: the voxel's own column's SubCell (what renders here).
            let Some(sub) = col.record_for(lx, lz) else {
                println!("    (no record realized at this column — station skipped)");
                continue;
            };
            let fill = sub.fill();
            let (recorded, dithered, movable_spans, runs) =
                expressed_column(set, fill, &sub.strata().events, vx, vz);
            println!(
                "    RANKING QUANTITY — {recorded} RECORDED within-class member contacts on the \
                 expressed face\n      (+ {dithered} contacts the veneer's dither drew, over \
                 {movable_spans} movable spans — read those as noise for THIS question)"
            );
            println!(
                "    the deep record here holds {alt} within-class alternation(s) in bands \
                 >= 0.9 m ({} such bands).",
                bands.len()
            );
            // The record is finely laminated — hundreds of centimetre bands — so
            // printing it raw buries the walk's quotable thicknesses in a wall of
            // `0.00 m` lines. Elide the laminae below 5 cm and SAY how many were
            // elided, rather than silently truncating.
            let all = coalesced_bands(&field.strata[*idx]);
            let (mut acc, mut shown, mut elided) = (0.0f64, 0usize, 0usize);
            println!(
                "    RECORDED succession, TOP-DOWN ({} coalesced bands in all; bands under 5 cm \
                 elided):",
                all.len()
            );
            for b in all.iter().rev() {
                if b.thickness_m >= 0.05 && shown < 16 {
                    shown += 1;
                    println!(
                        "      {:>7.2}–{:>7.2} m below record top   {:>22}  {:>7.2} m{}",
                        acc,
                        acc + b.thickness_m,
                        short(b.mat),
                        b.thickness_m,
                        if b.thickness_m < VOXEL_M {
                            "  (sub-voxel — expresses through `Plan::Mixed` eighths)"
                        } else {
                            ""
                        }
                    );
                } else {
                    elided += 1;
                }
                acc += b.thickness_m;
                if shown >= 16 {
                    break;
                }
            }
            println!(
                "      ({elided} laminae elided in the {acc:.1} m shown; {} bands lie below)",
                all.len().saturating_sub(shown + elided)
            );
            println!(
                "    EXPRESSED column at voxel ({vx},{vz}) — `plan(d)`, d=1 is the SURFACE voxel, \
                 {} voxels of record:",
                fill.depth_count()
            );
            for (name, lo, hi, mv) in runs.iter().take(28) {
                println!(
                    "      d {lo:>3}..{hi:<3} ({:>7.1} m below surface)  {name}{}",
                    (*lo as f64 - 1.0) * VOXEL_M,
                    if *mv {
                        "   [veneer, dither can move it]"
                    } else {
                        ""
                    }
                );
            }
            if runs.len() > 28 {
                println!("      ... {} more runs below", runs.len() - 28);
            }
            let deepest = runs.last().map(|(_, _, hi, _)| *hi).unwrap_or(1) as i64;
            print_bench(vx, vz, surf_m, deepest.clamp(8, 24), mx);
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
        // Top-down through the record, **coalescing adjacent same-rock units the
        // way `ColumnFill` will**: a bed the record kept as six thin units still
        // fills a voxel, and testing each unit against the 0.9 m floor on its own
        // is the `round Σ` vs `Σ round` error `fill.rs` exists to prevent. The
        // band's chapter and tag are the TOPMOST unit's — that is the one whose
        // draw address the counterfactual must reproduce.
        let mut bands: Vec<(MaterialId, f64, f64, u8, bool)> = Vec::new(); // mat, thick, depth, chapter, travelled
        let mut depth_m = 0.0f64;
        for u in field.strata[idx].units.iter().rev() {
            if depth_m > 30.0 {
                break;
            }
            let travelled = Litho::of_material(u.species()) != litho_of_tag(u.tag());
            match bands.last_mut() {
                Some(b) if b.0 == u.species() => b.1 += u.thickness_m(),
                _ => bands.push((
                    u.species(),
                    u.thickness_m(),
                    depth_m,
                    u.chapter(),
                    travelled,
                )),
            }
            depth_m += u.thickness_m();
        }
        for (mat, thickness_m, depth_m, chapter, travelled_class) in bands {
            if thickness_m < VOXEL_M {
                continue;
            }
            units_checked += 1;
            let would = site_would_draw(&mut mem_cache, set, idx, temp_c, precip, mat, chapter);
            if would != mat {
                units_mismatched += 1;
                mismatches.push(Mismatch {
                    idx,
                    depth_m,
                    recorded: mat,
                    would,
                    thickness_m,
                    travelled_class,
                });
            } else {
                agreements.push((idx, mat));
            }
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

    // Rank: loudest pair first (it must be SEEABLE), then shallowest (it must
    // reach the expression), then thickest.
    mismatches.sort_by(|a, b| {
        pair_visibility(b.recorded, b.would)
            .cmp(&pair_visibility(a.recorded, a.would))
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
            if picked
                .iter()
                .all(|p| km_between(conv.meters(p.idx), mm) > 5.0)
            {
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
                visibility_label(pair_visibility(c.recorded, c.would)),
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
            // Does it reach the expression, and at what depth? (P11 slice 3:
            // through the voxel's own column's SubCell — what renders here.)
            let col = wgen.column_record(cx, cz);
            let sub = col.record_for(vx.rem_euclid(32) as usize, vz.rem_euclid(32) as usize);
            let mut found: Option<u32> = None;
            if let Some(sub) = sub {
                let fill = sub.fill();
                for d in 1..=fill.depth_count() as u32 {
                    if let Some(e) = expressed_member(set, fill, &sub.strata().events, d, vx, vz)
                        && set.member(e.member).material == c.recorded
                    {
                        found = Some(d);
                        break;
                    }
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
            if found == Some(1) {
                pose(
                    "provenance, standing",
                    mx,
                    surf_m + 1.7,
                    mz,
                    YAW_SOUTH,
                    -0.5,
                    "the rock is the surface voxel — no cut. --fullbright, then `world_get_contents` \
                     at the feet voxel to read the mixture (NEVER `scan_region` for a material \
                     question)",
                );
            } else {
                print_bench(
                    vx,
                    vz,
                    surf_m,
                    found.map_or(16, |d| (d as i64 + 4).clamp(8, 24)),
                    mx,
                );
            }
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
                    print_bench(vx, vz, surf_m, 12, bx);
                    println!(
                        "      same framing as S3 #1 — but here the rock IS what this place \
                         would make."
                    );
                }
            }
        }
    }

    // ─────────────────────── inter-station distances ────────────────────────
    println!("\n\n========== INTER-STATION DISTANCES (one launch plans them all) ==========");
    let p1 = movable.first().map(|s| conv.meters(s.idx));
    let p2 = s2.first().map(|s| conv.meters(s.idx));
    let p3 = mismatches.first().map(|c| conv.meters(c.idx));
    for (an, a) in [("S1", p1), ("S2", p2), ("S3", p3)] {
        for (bn, b) in [("S1", p1), ("S2", p2), ("S3", p3)] {
            if an < bn
                && let (Some(a), Some(b)) = (a, b)
            {
                println!("  {an} ↔ {bn}: {:.1} km", km_between(a, b));
            }
        }
    }
    println!("  (teleports are free; the distances are for briefing, not for travel.)");

    println!(
        "\ntotal probe wall-clock {:.1} s",
        t0.elapsed().as_secs_f64()
    );
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
    use dc_worldgen::deeptime::recorder::Aridity;
    use dc_worldgen::deeptime::{DepEnv, DepTag, DepUnit, EnergyBand};

    fn unit(mat: MaterialId, t: f64) -> DepUnit {
        DepUnit::new(
            DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::Medium),
            t,
            false,
            0,
            mat,
        )
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

        let with_events = |events: Vec<StrataEvent>| dc_worldgen::StrataRec { events };

        // A veneer event in a two-member class: MOVABLE.
        let mut rec = with_events(vec![ev(mud, true)]);
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(top_span(&set, &fill, &rec.events).0, TopSpan::SingleMovable);

        // The same class, but a RECORDED identity (`dither: false`): frozen.
        rec = with_events(vec![ev(mud, false)]);
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(
            top_span(&set, &fill, &rec.events).0,
            TopSpan::SingleFrozen,
            "a deep-history event's member is recorded and expression must not be scored as \
             able to move it"
        );

        // A veneer event in a ONE-member class: frozen, because the dither has
        // nothing to pick between.
        rec = with_events(vec![ev(coal, true)]);
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(
            top_span(&set, &fill, &rec.events).0,
            TopSpan::SingleFrozen,
            "a one-member class is a constant field under any dither source"
        );

        // Two events sharing the surface voxel: MIXED, which `mixed_at` resolves
        // from the UNDITHERED member — blind to this slice by design.
        rec = with_events(vec![ev(mud, true), ev(coal, true)]);
        rec.events[0].thickness_m = 0.5;
        rec.events[1].thickness_m = 0.5;
        let fill = ColumnFill::build(&rec, VOXEL_M);
        assert_eq!(top_span(&set, &fill, &rec.events).0, TopSpan::Mixed);
    }

    /// The materials present in a record's top window (metres per material over
    /// the top `win_m` of the pile) — the presence quantity the frontier finder
    /// compares across adjacent cells.
    fn top_window_materials(rec: &DeepStrata, win_m: f64) -> Vec<(MaterialId, f64)> {
        let mut out: Vec<(MaterialId, f64)> = Vec::new();
        let mut acc = 0.0f64;
        for u in rec.units.iter().rev() {
            if acc >= win_m {
                break;
            }
            let take = u.thickness_m().min(win_m - acc);
            if take <= 0.0 {
                continue;
            }
            acc += take;
            match out.iter_mut().find(|(m, _)| *m == u.species()) {
                Some((_, t)) => *t += take,
                None => out.push((u.species(), take)),
            }
        }
        out
    }

    /// **P11 slice 3's ACCEPTANCE INSTRUMENT** (design audit § 4.2; the
    /// mudstone-mix border invariant, asserted rather than snapshotted):
    ///
    /// > At a deep-cell frontier where one cell's near-surface record contains
    /// > a member and its neighbour's does not, the presence border stops being
    /// > a straight chunk-quantized line: both parent records are realized by
    /// > voxel columns on BOTH sides of the geometric cell edge, and no chunk
    /// > straddling the frontier is single-parent.
    ///
    /// The asserts are **structural impossibilities under the retired
    /// chunk-centre NEAREST read**, never statistical bounds — and that is a
    /// deliberate correction to the design audit's § 4.2, which drafted
    /// per-chunk 4σ *binomial* floors. A binomial bound assumes per-column
    /// independent draws; the audit's own source pick (I-2: `Octaves`,
    /// coherent on purpose — a rock body is not per-voxel speckle) makes
    /// nearby columns share low-frequency draw values, so realized counts at
    /// chunk granularity are legitimately over-dispersed and a 4σ binomial
    /// would fire on healthy worlds (journal/0129: *"the source has to be
    /// white for that test to mean anything"* — the same lesson, at this
    /// joint). What survives coherence:
    ///
    /// 1. a frontier exists (found, not fabricated; a null is printed and is
    ///    a result — corrections #51);
    /// 2. **a realized-parent transition exists INSIDE a chunk, off the
    ///    chunk-pitch lattice** — impossible under the retired read (the
    ///    parent was constant per chunk, flipping only at x ≡ 0 mod 32; the
    ///    field report's straight border, inverted into a tripwire);
    /// 3. **deep interfingering**: a column MORE than 16 voxels west of the
    ///    geometric cell edge realizes the east parent, and mirrored —
    ///    impossible under the retired read (nearest-at-centre could hand a
    ///    column the far parent only inside a straddling chunk, ≤ 16 voxels
    ///    from the edge);
    /// 4. the MM-1 frequency-vs-weight comparison is PRINTED as a report, not
    ///    asserted — the law is asserted where it is measurable (dc-core,
    ///    white source); a fitted band here would be the tolerance
    ///    anti-pattern;
    /// 5. F2: the realized cell's record total equals its own regolith `H`
    ///    (the Law-3 leak tripwire, per column via the bundle).
    ///
    /// **Extent: Medium, and that is the smallest that exercises it** — Small's
    /// sampled chunks carry no strata record (journal/0129's golden note), so
    /// no presence frontier is realizable there; the extent requirement is
    /// about record presence, not production magnitude. Added gate wall-clock
    /// is reported in the slice's RETURN spec.
    #[test]
    fn the_presence_border_interfingers_across_the_deep_cell_frontier() {
        let pregen = Pregen::run(WorldParams {
            seed: SEED,
            extent: EXTENT,
        });
        let deep = &pregen.deep;
        let w = deep.w;
        let conv = Conv { w, wp: deep.wp };
        let win_m = 3.0 * VOXEL_M;

        // ---- 1. the frontier finder (the tour-map half) --------------------
        // Horizontally adjacent interior cell pairs; strongest = most present
        // metres of a material absent next door.
        let mut best: Option<(usize, usize, MaterialId, f64)> = None;
        for iy in 0..w {
            for ix in 0..w - 1 {
                let (ia, ib) = (iy * w + ix, iy * w + ix + 1);
                if !conv.interior(ia) || !conv.interior(ib) {
                    continue;
                }
                let (Some(ra), Some(rb)) = (deep.record_at_cell(ix as i64, iy as i64), {
                    deep.record_at_cell(ix as i64 + 1, iy as i64)
                }) else {
                    continue;
                };
                if ra.units.is_empty() || rb.units.is_empty() {
                    continue;
                }
                let (ta, tb) = (
                    top_window_materials(ra, win_m),
                    top_window_materials(rb, win_m),
                );
                for (m, t) in &ta {
                    if !tb.iter().any(|(mb, _)| mb == m) && best.as_ref().is_none_or(|b| *t > b.3) {
                        best = Some((ia, ib, *m, *t));
                    }
                }
                for (m, t) in &tb {
                    if !ta.iter().any(|(ma, _)| ma == m) && best.as_ref().is_none_or(|b| *t > b.3) {
                        best = Some((ib, ia, *m, *t));
                    }
                }
            }
        }
        let Some((present, absent, mat, strength)) = best else {
            // A null is a result: brief it, never fabricate a station.
            println!(
                "NULL: no adjacent deep-cell pair differs in top-window material \
                 presence on this world — nothing for the border invariant to bind on"
            );
            return;
        };
        let (pa, pb) = (conv.meters(present), conv.meters(absent));
        println!(
            "frontier: {mat:?} present {strength:.2} m in cell {present} \
             ({:.0} m, {:.0} m), absent in neighbour {absent} ({:.0} m, {:.0} m)",
            pa.0, pa.1, pb.0, pb.1
        );
        // Ready pose for the walk (station 1), on the edge midpoint.
        let (va, vb) = (conv.idx_to_voxel(present), conv.idx_to_voxel(absent));
        let edge_vx = (va.0 + vb.0) / 2;
        println!(
            "station pose (feet, world metres): x {:.1} z {:.1} — the cell edge runs N-S here",
            edge_vx as f64 * VOXEL_M,
            va.1 as f64 * VOXEL_M
        );

        // ---- 2..5. the frontier neighbourhood ------------------------------
        // A band of chunks either side of the edge (±2 chunks in x, a strip in
        // z), so the deep-interfingering assert (#3) can see columns well past
        // the 16-voxel reach the retired read had.
        let mut g = WorldGenerator::new(&pregen);
        let (west, east) = if va.0 < vb.0 {
            (present, absent)
        } else {
            (absent, present)
        };
        let edge_cx = edge_vx.div_euclid(32);
        let mut off_lattice_transitions = 0usize; // #2
        let mut deep_west_east = 0usize; // #3: >16 vox west, realizes east
        let mut deep_east_west = 0usize; // #3 mirrored
        let mut f2_checked = 0usize;
        let mut columns_seen = 0usize;
        // #4's report quantities: pooled realized-vs-weight for the two parents.
        let mut expect_w = 0.0f64;
        let mut obs_w = 0usize;
        for k in -6i64..=6 {
            let cz = (va.1 + k * 32).div_euclid(32);
            for dcx in -2i64..=2 {
                let ccx = edge_cx + dcx;
                let col = g.column_record(ccx, cz);
                for lz in 0..32usize {
                    // The realized parent along this west→east row of columns;
                    // a change at lx with vx % 32 != 0 is a transition the
                    // retired chunk-constant read could not produce. Rows are
                    // scanned per chunk, so a chunk-boundary flip never counts.
                    let mut prev: Option<u32> = None;
                    for lx in 0..32usize {
                        let (vx, vz) = (ccx * 32 + lx as i64, cz * 32 + lz as i64);
                        let Some(sub) = col.record_for(lx, lz) else {
                            prev = None;
                            continue;
                        };
                        let Some(gcell) = sub.cell_index() else {
                            prev = None;
                            continue;
                        };
                        columns_seen += 1;
                        if let Some(p) = prev
                            && p != gcell
                        {
                            // lx > 0 by construction: within-chunk transition.
                            off_lattice_transitions += 1;
                        }
                        prev = Some(gcell);
                        // ---- 3. deep interfingering past the 16-voxel reach.
                        if vx < edge_vx - 16 && gcell as usize == east {
                            deep_west_east += 1;
                        }
                        if vx > edge_vx + 16 && gcell as usize == west {
                            deep_east_west += 1;
                        }
                        // ---- 4's report: realized-vs-weight for the west
                        // parent (printed, never asserted — see the doc
                        // comment on why a σ bound is unsound here).
                        if let Some((gx, gy)) = deep.deep_coords(vx, vz) {
                            let (i0, j0) = (gx.floor(), gy.floor());
                            let (fx, fz) = (gx - i0, gy - j0);
                            for (di, dj, wt) in [
                                (0i64, 0i64, (1.0 - fx) * (1.0 - fz)),
                                (1, 0, fx * (1.0 - fz)),
                                (0, 1, (1.0 - fx) * fz),
                                (1, 1, fx * fz),
                            ] {
                                let ci = (i0 as i64 + di).clamp(0, w as i64 - 1) as u32;
                                let cj = (j0 as i64 + dj).clamp(0, w as i64 - 1) as u32;
                                if cj * w as u32 + ci == west as u32 {
                                    expect_w += wt;
                                }
                            }
                        }
                        if gcell as usize == west {
                            obs_w += 1;
                        }
                        // ---- 5. the F2 bundle: record total == the SAME
                        // cell's H. Sampled sparsely (one column in 64) — the
                        // predicate is per-column and identical everywhere.
                        if (lz * 32 + lx) % 64 == 0 {
                            let (ci, cj) = (gcell as i64 % w as i64, gcell as i64 / w as i64);
                            if let Some(bundle) = deep.cell_bundle(ci, cj)
                                && let Some(h) = bundle.regolith_m
                            {
                                // Bound: the recorder's f64 accumulation
                                // residual, plus (post-pack) half a thickness
                                // quantum — both under 1e-3 m. Derived, not
                                // fitted.
                                assert!(
                                    (bundle.record.total_m() - h).abs() < 1e-3,
                                    "cell {gcell}: record total {} != regolith H {h} — \
                                     the dithered column reads a record and an H that \
                                     disagree (the F2 Law-3 leak)",
                                    bundle.record.total_m()
                                );
                                f2_checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(
            columns_seen > 10_000,
            "only {columns_seen} recorded columns near the frontier — the sample \
             cannot carry the invariant"
        );
        // ---- 2. the border left the chunk lattice --------------------------
        assert!(
            off_lattice_transitions > 0,
            "no realized-parent transition occurs INSIDE any chunk near the \
             frontier — under the retired chunk-centre NEAREST read the parent was \
             constant per chunk (transitions only at 28.8 m chunk pitch), and that \
             is still what this world shows (the field report's straight border)"
        );
        // ---- 3. deep interfingering ----------------------------------------
        assert!(
            deep_west_east > 0 && deep_east_west > 0,
            "no column further than 16 voxels from the cell edge realizes the far \
             parent (west→east {deep_west_east}, east→west {deep_east_west}) — the \
             retired read could reach at most 16 voxels past the edge (a straddling \
             chunk's half), so a cell-wide membership blend must exceed it"
        );
        assert!(f2_checked > 0, "the F2 bundle check never ran");
        println!(
            "border invariant: {off_lattice_transitions} within-chunk parent \
             transitions; deep interfingering west→east {deep_west_east} / east→west \
             {deep_east_west} columns (past the 16-voxel legacy reach); MM-1 report — \
             west parent realized {obs_w} of {columns_seen} columns vs summed bilinear \
             weight {expect_w:.0} (printed, not asserted: coherent source); F2 bundle \
             checked at {f2_checked} columns"
        );
    }
}
