//! The two-plane erosion engine: bedrock stock `R` and alluvium stock `H`
//! evolved by uplift, priority-flood drainage, mass-conserving stream-power
//! transport with alluvial-cover shielding, bedrock weathering, and hillslope
//! diffusion — the SPACE-family model, stripped to the mechanism (orogeny recon
//! § erosion, earth-processes.md § 4).
//!
//! **Mass is conserved explicitly down the receiver chain.** Every metre that
//! leaves a cell (entrained alluvium or incised bedrock) becomes suspended
//! flux; every metre the flow can no longer carry is deposited as alluvium
//! downstream; whatever reaches a sink (sea or domain border) is deposited
//! there. The only external input is uplift, so over `N` iterations
//! `Δ(ΣR + ΣH) = N · Σuplift` — the falsifier the mass-conservation test asserts.
//!
//! **Never incise below the receiver**: bedrock lowering at a cell is clamped
//! so its bedrock top cannot drop below the receiver's surface — no runaway
//! knickpoint digs a hole its own outlet can't drain.
//!
//! Determinism: a fixed scan order everywhere, no wall clock, no ambient
//! entropy (the only draws are the addressed initial-roughness jitter in
//! grid.rs). A timing harness may wrap `Instant` *around* a run, never inside.
//!
//! **S9b parallelism.** The per-cell-independent phases (uplift apply, surface
//! build, D8 routing, weathering, hillslope diffusion, and the strata recorder)
//! run data-parallel across cells via rayon when [`Erosion::set_parallel`] is
//! on. Each is expressed as a *pure per-cell function* driven by either a
//! sequential or a `par_iter` loop, so the parallel result is **byte-identical**
//! to the scalar one by construction (identical per-cell arithmetic, disjoint
//! writes, no cross-cell summation reorder). The three phases with a genuine
//! cross-cell dependency — the priority-flood fill (a global min-heap), the
//! drainage-area accumulation, and the mass-routing transport pass (both are
//! downstream-ordered flux chains) — stay **scalar**: parallelizing them means a
//! level-ordered gather whose summation order differs from the serial scan,
//! which would break byte-identity. They are the measured serial floor
//! (S9b-results).
//!
//! **Erodibility coupling (journal/0029).** With [`DeepConfig::erodibility`] on,
//! the fluvial terms and hillslope diffusion are modulated per cell per epoch by
//! the resistance of the lithology outcropping there ([`super::lithology`]).
//! Resistance is **agent-specific, never a single scalar** — the mechanical
//! agent reads an abrasion axis, and (journal/0034) the frost/ice, littoral and
//! eolian agents are now live behind [`DeepConfig::full_agents`], each reading
//! its own axis; only the dissolution (karst) agent remains designed-but-unbuilt.
//! See the lithology module docs for why a one-number erodibility would foreclose
//! karst. Both flags are off by default, and with them off every multiplier is
//! the exact identity `1.0` and every added phase is skipped, so the uncoupled
//! path is byte-identical.
//!
//! **The full agent roster (journal/0034), behind [`DeepConfig::full_agents`]:**
//! - **frost** — a temperature-gated weathering multiplier folded into the
//!   `weather` phase (freeze–thaw peaks near `0°C`, weighted by the frost/ice
//!   axis; see [`Erosion::periglacial`]);
//! - **wave** — littoral cutting at the current sea stand ([`Erosion::wave`]),
//!   mass-neutral (quarried rock goes offshore);
//! - **wind** — deflation + downwind loess/dune deposition along the climate's
//!   own prevailing wind ([`Erosion::wind`]), mass-neutral (pure redistribution).
//!
//! Wind and wave run **after** the recorder and self-record, so their distinct
//! facies reach the strata; frost rides the normal weathering record.
//!
//! Note (S9b): hillslope diffusion is reformulated from the original scatter
//! (`h[i] -= f; h[j] += f`) to an equivalent **gather** (each cell sums its own
//! in/out edge fluxes), which conserves mass identically but changes the
//! floating-point summation order. Scalar and parallel both use the gather, so
//! they agree to the bit; the gather differs from the pre-S9b scatter only in fp
//! round-off (well inside the mass-conservation slack).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use rayon::prelude::*;

use super::climate;
use super::grid::{DeepConfig, DeepGrid, SEA_LEVEL_M};
use super::lithology::{self, Agent, Litho};
use super::recorder::{Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, Eolian};

/// Strictly-descending fill increment (metres) — as in pregen hydrology.
const EPS: f64 = 0.001;

const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const NEIGH4: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// Below this cell count the rayon fork/join overhead outweighs the work, so the
/// per-cell phases fall back to the sequential loop even when parallel is on.
const PAR_MIN_CELLS: usize = 1 << 15;

/// A priority-flood heap item ordered by filled elevation (min-heap via
/// `Reverse`), ties broken by index for determinism.
#[derive(PartialEq)]
struct Item {
    filled: f64,
    idx: u32,
}
impl Eq for Item {}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.filled
            .total_cmp(&other.filled)
            .then(self.idx.cmp(&other.idx))
    }
}

// ---------------------------------------------------------------------------
// Pure per-cell kernels. Each is a function of read-only inputs and returns (or
// mutates only) the single cell's own state, so a sequential and a parallel
// driver over them produce byte-identical output. Grid geometry helpers are
// free functions taking `w` so the kernels don't borrow `&Erosion`.

#[inline]
fn in_grid(gx: i32, gy: i32, w: usize) -> Option<usize> {
    if gx >= 0 && gy >= 0 && (gx as usize) < w && (gy as usize) < w {
        Some(gy as usize * w + gx as usize)
    } else {
        None
    }
}

#[inline]
fn coords_of(i: usize, w: usize) -> (i32, i32) {
    ((i % w) as i32, (i / w) as i32)
}

#[inline]
fn is_border(i: usize, w: usize) -> bool {
    let (gx, gy) = coords_of(i, w);
    gx == 0 || gy == 0 || gx as usize == w - 1 || gy as usize == w - 1
}

/// D8 steepest-descent receiver of cell `i` on the filled surface (`-1` = sink).
#[inline]
fn route_cell(i: usize, w: usize, surf: &[f64], filled: &[f64], sea_level: f64) -> i32 {
    if surf[i] <= sea_level || is_border(i, w) {
        return -1;
    }
    let (gx, gy) = coords_of(i, w);
    let fi = filled[i];
    let mut best: Option<(f64, usize)> = None;
    for (dx, dy) in NEIGH8 {
        let Some(j) = in_grid(gx + dx, gy + dy, w) else {
            continue;
        };
        if filled[j] < fi && best.is_none_or(|(bf, _)| filled[j] < bf) {
            best = Some((filled[j], j));
        }
    }
    best.map_or(-1, |(_, j)| j as i32)
}

/// A per-cell erodibility multiplier, or the exact identity `1.0` when the
/// plane is empty (coupling off). `x * 1.0` is bit-exact for every finite `x`,
/// which is what makes the uncoupled path byte-identical.
#[inline]
fn sus_at(sus: &[f64], i: usize) -> f64 {
    if sus.is_empty() { 1.0 } else { sus[i] }
}

/// The effective per-cell hillslope diffusivity.
///
/// **Composition order is deliberate and fixed** (journal/0029): the config
/// diffusivity is reduced first by the cell's biotic root-cohesion resistance
/// (S10 `resist`), then by the lithology's abrasion susceptibility. Rock first
/// in *meaning* — what the slope is made of — biology second, applied to the
/// slope biology actually lives on; but in *arithmetic* the biotic factor is
/// applied first and the lithic factor multiplied onto the right, because f64
/// multiplication is not associative and the order has to be pinned for
/// byte-identity. Written the other way round, turning coupling off would not
/// reproduce the S10 result bit for bit.
///
/// With both layers off this is exactly `diffusion`; with only biology on it is
/// exactly the S10 expression.
#[inline]
fn eff_diff(diffusion: f64, resist: &[f32], sus: &[f64], i: usize) -> f64 {
    let biotic = if resist.is_empty() {
        diffusion
    } else {
        diffusion * (1.0 - f64::from(resist[i]))
    };
    if sus.is_empty() {
        biotic
    } else {
        biotic * sus[i]
    }
}

/// Net hillslope-diffusion thickness change at cell `i` (metres), gathered from
/// its four edges on the frozen surface with the frozen per-cell limiter
/// `scale`. Outflux edges (i higher) use `scale[i]` and the donor `i`'s effective
/// diffusivity; influx edges (neighbour higher) use the donor `j`'s `scale[j]`
/// and `j`'s effective diffusivity — exactly the flux the scatter form moved, so
/// the two conserve mass identically. Summation order is fixed (`NEIGH4`).
#[inline]
fn diffuse_net_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    scale: &[f64],
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut net = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                net -= eff_diff(diffusion, resist, sus, i) * d * scale[i];
            } else if d < 0.0 {
                net += eff_diff(diffusion, resist, sus, j) * (-d) * scale[j];
            }
        }
    }
    net
}

/// Per-cell diffusion outflux sum → limiter scale on the frozen surface (using
/// the donor cell's biotic-reduced effective diffusivity).
#[inline]
fn diffuse_scale_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    h: f64,
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut out = 0.0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                out += eff_diff(diffusion, resist, sus, i) * d;
            }
        }
    }
    if out > h && out > 0.0 { h / out } else { 1.0 }
}

/// Subaerial bedrock→regolith weathering for one cell (cover-tapered), scaled by
/// `rate_mult` — the **product of the two modifier layers**, the biotic
/// weathering multiplier (S10 `wmult`) and the lithologic susceptibility. When
/// both layers are off that product is exactly `1.0` and this is bit-identical
/// to the pre-S10 rate. Land plants accelerate chemical weathering several-fold
/// (ecology.md § 1).
///
/// **This is the rate-limiting phase on hillslopes, which is why it must be
/// coupled** (journal/0029). Hillslope diffusion is flux-limited by the regolith
/// actually available, so on any real slope it exports everything there is and
/// the landscape's lowering rate collapses to the rate bedrock is *converted*
/// into regolith. Coupling incision and entrainment alone left the world
/// statistically unchanged, because on the majority of land neither term is what
/// sets the pace.
///
/// **Composition order** (fixed, and load-bearing for byte-identity since f64
/// multiplication is not associative): the modifier layers combine with each
/// other first — `rate_mult = wmult × litho_sus × frost`, biology on the left,
/// the periglacial frost multiplier on the right (journal/0034) — and the product
/// then scales the base rate before the cover taper: `weathering × rate_mult ×
/// taper`. With lithology and frost off, `wmult × 1.0 × 1.0 == wmult` exactly, so
/// the S10 expression is reproduced bit for bit. Frost is the second in-place
/// weathering *agent*: earlier the sum over agents had one mechanical term, now
/// freeze–thaw adds a temperature-gated term to the same phase (the karst story
/// will add a third, chemical, term here without a rewrite).
///
/// **What the lithic factor will mean when there is more than one agent.**
/// In-place weathering is not one process; it is the sum of every agent's attack
/// on rock that has not moved yet. Today that sum has exactly one term, the
/// mechanical one, so the factor is the abrasion susceptibility. When the
/// dissolution agent lands this becomes a sum over agents — and a limestone will
/// weather *fast* through the chemical term while resisting the mechanical one,
/// which is the karst story arriving without anything here being rewritten.
#[inline]
fn weather_cell(
    r: &mut f64,
    h: &mut f64,
    dh: &mut f64,
    sea: f64,
    weathering: f64,
    h_star: f64,
    rate_mult: f64,
) {
    if *r + *h > sea {
        let wth = weathering * rate_mult * (-*h / h_star).exp();
        *r -= wth;
        *h += wth;
        *dh += wth;
    }
}

/// The biotic weathering multiplier at cell `i`: `1.0` when the biotic layer is
/// off (empty slice), so the abiotic weathering rate is byte-identical.
#[inline]
fn wmult_at(bio_weather: &[f32], i: usize) -> f64 {
    if bio_weather.is_empty() {
        1.0
    } else {
        f64::from(bio_weather[i])
    }
}

/// The frost weathering multiplier at cell `i`: `≥ 1.0` when the periglacial
/// agent is on, and the exact identity `1.0` when the plane is empty (the frost
/// agent is off). Multiplied onto the weathering rate, so `× 1.0` keeps the
/// frost-off path byte-identical (journal/0034).
#[inline]
fn frost_at(frost: &[f64], i: usize) -> f64 {
    if frost.is_empty() { 1.0 } else { frost[i] }
}

// ---------------------------------------------------------------------------
// S9b flood-parallelism probe (MEASUREMENT ONLY — not on the byte-identical
// path). The verdict hinges on whether the priority-flood, the step's dominant
// serial cost, can be parallelized. These two functions let the harness measure
// the *optimistic* parallel-flood ceiling and its correctness cost, without
// pretending the result is deterministic or exact.

/// Serial priority-flood, returning only the filled surface (the reference for
/// the tiled-flood divergence check). Same algorithm as [`Erosion::flood`].
pub fn flood_fill_serial(w: usize, surf: &[f64], sea: f64) -> Vec<f64> {
    let n = w * w;
    let mut filled = vec![f64::INFINITY; n];
    let mut done = vec![false; n];
    let mut heap: BinaryHeap<Reverse<Item>> = BinaryHeap::new();
    for i in 0..n {
        if surf[i] <= sea || is_border(i, w) {
            filled[i] = surf[i];
            heap.push(Reverse(Item {
                filled: surf[i],
                idx: i as u32,
            }));
        }
    }
    while let Some(Reverse(item)) = heap.pop() {
        let i = item.idx as usize;
        if done[i] {
            continue;
        }
        done[i] = true;
        let (gx, gy) = coords_of(i, w);
        for (dx, dy) in NEIGH8 {
            if let Some(j) = in_grid(gx + dx, gy + dy, w) {
                if done[j] || filled[j].is_finite() {
                    continue;
                }
                filled[j] = surf[j].max(filled[i] + EPS);
                heap.push(Reverse(Item {
                    filled: filled[j],
                    idx: j as u32,
                }));
            }
        }
    }
    filled
}

/// **Optimistic** tiled parallel priority-flood: split the grid into `strips`
/// row bands, fill each in parallel with its internal seams treated as *open*
/// outlets (a cell on a strip's top/bottom edge pours at its own surface). This
/// is the best case for a parallel flood — no reconciliation passes — so its
/// wall time upper-bounds any correct tiled flood's speedup, and its divergence
/// from [`flood_fill_serial`] is the correctness debt a real (Barnes-style)
/// parallel flood must pay back with border-relaxation sweeps. It is **not**
/// deterministic-equivalent to the serial fill and is never on the sim path.
pub fn flood_fill_tiled(w: usize, surf: &[f64], sea: f64, strips: usize) -> Vec<f64> {
    let n = w * w;
    let strips = strips.clamp(1, w);
    let bands: Vec<(usize, usize)> = (0..strips)
        .map(|t| (t * w / strips, (t + 1) * w / strips))
        .collect();
    let results: Vec<Vec<f64>> = bands
        .par_iter()
        .map(|&(y0, y1)| {
            let h = y1 - y0;
            let mut filled = vec![f64::INFINITY; h * w];
            let mut done = vec![false; h * w];
            let mut heap: BinaryHeap<Reverse<Item>> = BinaryHeap::new();
            for ly in 0..h {
                let gy = y0 + ly;
                for gx in 0..w {
                    let gi = gy * w + gx;
                    let li = ly * w + gx;
                    let seam = (ly == 0 && y0 > 0) || (ly == h - 1 && y1 < w);
                    if surf[gi] <= sea || is_border(gi, w) || seam {
                        filled[li] = surf[gi];
                        heap.push(Reverse(Item {
                            filled: surf[gi],
                            idx: li as u32,
                        }));
                    }
                }
            }
            while let Some(Reverse(item)) = heap.pop() {
                let li = item.idx as usize;
                if done[li] {
                    continue;
                }
                done[li] = true;
                let lx = (li % w) as i32;
                let lly = (li / w) as i32;
                for (dx, dy) in NEIGH8 {
                    let (nx, ny) = (lx + dx, lly + dy);
                    if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                        continue;
                    }
                    let lj = ny as usize * w + nx as usize;
                    if done[lj] || filled[lj].is_finite() {
                        continue;
                    }
                    let gj = (y0 + ny as usize) * w + nx as usize;
                    filled[lj] = surf[gj].max(filled[li] + EPS);
                    heap.push(Reverse(Item {
                        filled: filled[lj],
                        idx: lj as u32,
                    }));
                }
            }
            filled
        })
        .collect();
    let mut out = vec![f64::INFINITY; n];
    for (t, &(y0, y1)) in bands.iter().enumerate() {
        let h = y1 - y0;
        out[y0 * w..y1 * w].copy_from_slice(&results[t][..h * w]);
    }
    out
}

/// Map a stream transport capacity to a facies energy band. Thresholds are in
/// the capacity units of the transport pass (metres/iteration); calibrated so
/// headwater hillslopes read Low, trunk rivers read High.
pub fn energy_band(cap: f64) -> EnergyBand {
    if cap < 0.002 {
        EnergyBand::Low
    } else if cap < 0.02 {
        EnergyBand::Medium
    } else {
        EnergyBand::High
    }
}

/// The measured depositional tag for a cell given its final surface, precip, and
/// the transport capacity it saw this iteration.
#[inline]
fn tag_of(surf_i: f64, precip_i: f32, energy_i: f64, sea_level: f64) -> DepTag {
    let env = if surf_i <= sea_level {
        DepEnv::Subsea
    } else {
        DepEnv::Subaerial
    };
    let aridity = if f64::from(precip_i) < 0.32 {
        Aridity::Arid
    } else {
        Aridity::Humid
    };
    DepTag::mineral(env, aridity, energy_band(energy_i))
}

/// Apply one cell's net thickness change to its strata record under `tag`,
/// stamped with the current tectonic `chapter` (0 when tectonic history is off).
#[inline]
fn record_cell(s: &mut DeepStrata, dh: f64, tag: DepTag, chapter: u8) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        s.deposit(tag, dh, chapter);
    } else {
        s.erode(-dh);
    }
}

/// Reusable scratch for the erosion iteration (allocated once, reused every
/// step — the per-iteration working set the memory measurement counts).
pub struct Erosion {
    w: usize,
    n: usize,
    cell_m: f64,
    /// Data-parallel per-cell phases when set (byte-identical to scalar).
    parallel: bool,
    /// `Σ grid.uplift` precomputed once (constant across iterations): the ledger
    /// value each step returns. Summed sequentially so it equals the scalar
    /// in-loop fold to the bit.
    uplift_sum: f64,
    /// Sea level for the current step (set by [`Erosion::step`]); the dynamic
    /// paleo-sea-level stand drives shoreline transgression/regression, so a
    /// coastal column records alternating marine/subaerial bands (the classic
    /// layered cliff — earth-processes.md § 6). Mean is [`SEA_LEVEL_M`].
    sea_level: f64,
    surf: Vec<f64>,
    filled: Vec<f64>,
    done: Vec<bool>,
    recv: Vec<i32>,
    area: Vec<f64>,
    qs: Vec<f64>,
    order: Vec<u32>,
    dh: Vec<f64>,
    energy: Vec<f64>,
    scale: Vec<f64>,
    /// Diffusion gather scratch: per-cell net ΔH, applied after the gather.
    netdiff: Vec<f64>,
    /// **Erodibility coupling planes** (empty when `cfg.erodibility` is off, and
    /// then read as the exact identity `1.0`).
    ///
    /// `litho[i]` is the lithology outcropping at cell `i` this epoch;
    /// `sus_flow[i]` is its abrasion susceptibility for the fluvial terms
    /// (cover entrainment + bedrock incision) and `sus_creep[i]` the same for
    /// hillslope diffusion under its own, weaker contrast knob. Both are looked
    /// up from a six-entry table built once per epoch, so the `powf` is paid six
    /// times per epoch rather than once per cell.
    litho: Vec<u8>,
    sus_flow: Vec<f64>,
    sus_creep: Vec<f64>,
    /// **Periglacial frost weathering multiplier** plane (journal/0034), empty
    /// when the frost agent (`cfg.full_agents`) is off and then read as the exact
    /// identity `1.0`. `frost[i] ≥ 1.0` is the freeze–thaw enhancement of the
    /// weathering rate at cell `i` this epoch — computed fresh each epoch because
    /// it depends on the current surface (temperature lapses with elevation).
    frost: Vec<f64>,
    /// **Tectonic-history state** (empty/zero when `cfg.tectonic_history` is off).
    ///
    /// `cur_chapter` is the chapter index the recorder stamps this iteration (0
    /// off the flag). `forcing[i]` is the blended analytic thickening rate
    /// (m/iter) for the current iteration, written by [`Self::set_forcing`] from
    /// the driver's precomputed per-chapter planes. `r_snap[i]` snapshots bedrock
    /// before the erosion phases so [`Self::track_exhumation`] can attribute the
    /// R-lowering (incision + weathering) to `exhum`/`t_crust` — never the
    /// isostatic bedrock motion, which runs afterward.
    cur_chapter: u8,
    forcing: Vec<f64>,
    r_snap: Vec<f64>,
    heap: BinaryHeap<Reverse<Item>>,
}

impl Erosion {
    pub fn new(grid: &DeepGrid) -> Self {
        let n = grid.w * grid.w;
        // NB (tectonics.md § 14.3): `uplift_sum` caches the constant per-cell
        // uplift plane — valid on the legacy path where uplift never changes. On
        // the tectonic-history path the ledger is *not* this sum; it is the
        // isostatic bedrock injection returned by [`Self::isostasy`] each step
        // (`uplift(t)` invalidates the cached constant — flagged so the spike
        // does not discover it as a mysterious conservation failure).
        let uplift_sum = grid.uplift.iter().sum();
        Self {
            w: grid.w,
            n,
            cell_m: grid.cell_m,
            parallel: false,
            uplift_sum,
            sea_level: SEA_LEVEL_M,
            surf: vec![0.0; n],
            filled: vec![0.0; n],
            done: vec![false; n],
            recv: vec![-1; n],
            area: vec![0.0; n],
            qs: vec![0.0; n],
            order: Vec::with_capacity(n),
            dh: vec![0.0; n],
            energy: vec![0.0; n],
            scale: vec![0.0; n],
            netdiff: vec![0.0; n],
            litho: Vec::new(),
            sus_flow: Vec::new(),
            sus_creep: Vec::new(),
            frost: Vec::new(),
            cur_chapter: 0,
            forcing: Vec::new(),
            r_snap: Vec::new(),
            heap: BinaryHeap::new(),
        }
    }

    /// Set the tectonic chapter the recorder stamps and load this iteration's
    /// blended thickening forcing (§ 3.1). Called by the run driver before each
    /// [`Self::step`] on the tectonic-history path. `plane` is the analytic
    /// forcing already blended across the chapter ramp.
    pub fn set_tectonic(&mut self, chapter: u8, plane: &[f64]) {
        self.cur_chapter = chapter;
        if self.forcing.len() != self.n {
            self.forcing = vec![0.0; self.n];
            self.r_snap = vec![0.0; self.n];
        }
        self.forcing.copy_from_slice(plane);
    }

    /// The chapter the recorder is currently stamping (read by the biotic layer so
    /// its organic units carry the same chapter).
    #[inline]
    pub fn current_chapter(&self) -> u8 {
        self.cur_chapter
    }

    /// Turn data-parallel per-cell phases on/off. Off by default (the scalar
    /// reference path). Parallel output is byte-identical (see module docs).
    pub fn set_parallel(&mut self, on: bool) {
        self.parallel = on;
    }

    /// Whether cell-parallel phases actually fork (parallel on *and* the grid is
    /// big enough to amortise rayon's overhead).
    #[inline]
    fn par(&self) -> bool {
        self.parallel && self.n >= PAR_MIN_CELLS
    }

    /// Working-set footprint of the scratch arrays (bytes), for the memory
    /// budget — separate from the grid's own resident state.
    pub fn scratch_bytes(&self) -> usize {
        let f64s = self.surf.len()
            + self.filled.len()
            + self.area.len()
            + self.qs.len()
            + self.dh.len()
            + self.energy.len()
            + self.scale.len()
            + self.netdiff.len()
            + self.sus_flow.len()
            + self.sus_creep.len()
            + self.frost.len();
        f64s * 8
            + self.recv.len() * 4
            + self.order.capacity() * 4
            + self.done.len()
            + self.litho.len()
    }

    /// The lithology outcropping at each cell as of the last [`Self::expose`]
    /// (empty when the erodibility coupling is off) — read by the measurement
    /// probe to attribute landform statistics to rock type.
    pub fn exposed(&self) -> impl Iterator<Item = Litho> + '_ {
        self.litho.iter().map(|&b| Litho::ALL[b as usize])
    }

    /// Set the paleo-sea-level stand the standalone phase methods read (the
    /// profiling harness drives phases individually; [`Self::step`] sets this).
    pub fn set_sea_level(&mut self, sea_level: f64) {
        self.sea_level = sea_level;
    }

    /// One deep-time iteration at the given `sea_level` stand. Returns the
    /// total uplift added this step (for the mass-conservation ledger).
    pub fn step(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, sea_level: f64) -> f64 {
        self.sea_level = sea_level;
        // Phase 1: the external forcing. Legacy path adds a constant uplift plane
        // to bedrock; tectonic-history path adds the analytic thickening rate to
        // the crustal columns (elevation is then *derived* by isostasy, phase 8b).
        let legacy_uplift = if cfg.tectonic_history {
            self.apply_thickening(grid);
            0.0
        } else {
            self.apply_uplift(grid)
        };
        self.expose(grid, cfg);
        self.periglacial(grid, cfg);
        self.build_surface(grid);
        self.flood();
        self.route();
        self.accumulate_area();
        if cfg.tectonic_history {
            self.snapshot_bedrock(grid);
        }
        self.transport(grid, cfg);
        self.weather(grid, cfg);
        self.diffuse(grid, cfg);
        // Phase 8b: exhumation bookkeeping + isostasy. The R-lowering the erosion
        // phases just did decrements `t_crust` and grows `exhum`; then Airy
        // compensation of the smoothed load derives the new bedrock surface. The
        // ledger for the tectonic path is the isostatic injection ΣΔR (an external
        // input to `ΣR+ΣH`, declared exactly like `biotic_total`).
        let ledger = if cfg.tectonic_history {
            self.track_exhumation(grid);
            self.isostasy(grid, cfg)
        } else {
            legacy_uplift
        };
        if cfg.record {
            self.record(grid);
        }
        // The wind and wave agents run **after** the recorder and self-record
        // (like the biotic layer), so their own facies reach the record rather
        // than being lumped under the epoch's fluvial tag. Both only redistribute
        // mass — wind moves loose `H`, wave moves `R`/`H` offshore — so the mass
        // ledger `Δ(ΣR+ΣH) == uplift + biotic` is untouched (journal/0034).
        if cfg.full_agents {
            self.wind(grid, cfg);
            self.wave(grid, cfg);
        }
        ledger
    }

    // ---- phase 1 (tectonic): crustal thickening + isostasy ----------------

    /// Add the blended analytic thickening rate into the crustal columns
    /// (§ 4.2/§ 5.2). Positive rates thicken (orogeny/arc), negative thin
    /// (rift/trench/ridge); `t_crust` is floored so a column cannot thin to
    /// nothing. Purely per-cell → byte-identical parallel. Elevation is untouched
    /// here — that is isostasy's job.
    pub fn apply_thickening(&mut self, grid: &mut DeepGrid) {
        let forcing = &self.forcing;
        if forcing.is_empty() {
            return;
        }
        if self.par() {
            grid.t_crust
                .par_iter_mut()
                .zip(forcing.par_iter())
                .for_each(|(t, f)| *t = (*t + *f).max(1000.0));
        } else {
            for (t, f) in grid.t_crust.iter_mut().zip(forcing.iter()) {
                *t = (*t + *f).max(1000.0);
            }
        }
    }

    /// Snapshot bedrock before the erosion phases, so [`Self::track_exhumation`]
    /// can measure the R-lowering the phases produce (incision + weathering) and
    /// attribute it to the column — never the later isostatic motion.
    fn snapshot_bedrock(&mut self, grid: &DeepGrid) {
        self.r_snap.copy_from_slice(&grid.r);
    }

    /// Every metre of bedrock the erosion phases removed this step decrements the
    /// crustal thickness and grows cumulative exhumation (§ 5.2 — "every metre of
    /// bedrock converted or incised decrements `t_crust` and increments `exhum`").
    /// Deposition grows `H`, not `t_crust`, so it is not counted here. Per-cell →
    /// byte-identical parallel.
    pub fn track_exhumation(&mut self, grid: &mut DeepGrid) {
        let snap = &self.r_snap;
        if self.par() {
            grid.exhum
                .par_iter_mut()
                .zip(grid.t_crust.par_iter_mut())
                .zip(grid.r.par_iter())
                .zip(snap.par_iter())
                .for_each(|(((e, t), r), s)| {
                    let removed = (*s - *r).max(0.0);
                    *e += removed;
                    *t -= removed;
                });
        } else {
            for (((e, t), r), s) in grid
                .exhum
                .iter_mut()
                .zip(grid.t_crust.iter_mut())
                .zip(grid.r.iter())
                .zip(snap.iter())
            {
                let removed = (*s - *r).max(0.0);
                *e += removed;
                *t -= removed;
            }
        }
    }

    /// **Airy compensation of the smoothed load** (§ 6.1). Computes the flexural-
    /// wavelength-smoothed crust and sediment loads, derives the Airy equilibrium
    /// surface per cell, and relaxes bedrock toward it at `iso_rate`. Returns the
    /// summed injected ΔR (the mass-ledger external input). The smoothing is a
    /// fixed-order separable blur — scalar in both drivers, so isostasy is
    /// byte-identical scalar↔parallel and is the run's only new serial cost of
    /// note (§ SPIKE 3/8).
    pub fn isostasy(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) -> f64 {
        use super::isostasy;
        let radius = isostasy::flex_radius_cells(cfg.flex_wavelength_km, self.cell_m);
        let t_bar = isostasy::box_smooth(&grid.t_crust, self.w, radius);
        let h_bar = isostasy::box_smooth(&grid.h, self.w, radius);
        let rate = cfg.iso_rate;
        let mut injected = 0.0f64;
        for i in 0..self.n {
            let rho_c =
                isostasy::rho_crust(super::tectonics::CrustKind::from_index(grid.crust_kind[i]));
            let e_eq = isostasy::equilibrium(t_bar[i], h_bar[i], rho_c);
            let surf = grid.r[i] + grid.h[i];
            let d_r = rate * (e_eq - surf);
            grid.r[i] += d_r;
            injected += d_r;
        }
        injected
    }

    // ---- phase 1: uplift into bedrock -------------------------------------

    /// Add the per-cell uplift into bedrock. Returns the (constant) total.
    pub fn apply_uplift(&mut self, grid: &mut DeepGrid) -> f64 {
        if self.par() {
            grid.r
                .par_iter_mut()
                .zip(grid.uplift.par_iter())
                .for_each(|(r, u)| *r += *u);
        } else {
            for i in 0..self.n {
                grid.r[i] += grid.uplift[i];
            }
        }
        self.uplift_sum
    }

    // ---- phase 1b: expose lithology (PARALLEL — per-cell independent) ------

    /// Determine which lithology outcrops at each cell and cache its
    /// agent-specific rate multipliers for this epoch (the erodibility
    /// coupling, `deeptime::lithology`).
    ///
    /// Runs at the **top** of the step, so every phase in the epoch sees one
    /// consistent answer to "what rock is at the surface here" — the record is
    /// only rewritten at the end of the step, so this reads the true current
    /// state, not a lagged one (unlike the S10 biotic modifiers, which are
    /// deliberately one epoch behind to break the biology↔erosion cycle).
    ///
    /// **The exposed unit is the top of the record, and an empty record means
    /// basement.** That single fallback is where resistant shield and craton
    /// landscapes come from: strip a column past its whole sedimentary history
    /// and what the flow meets next is igneous basement, the hardest thing in
    /// the world. Nobody wrote a shield rule.
    ///
    /// A note on *which* plane the contrast between beds rides. The brief said
    /// "modulate bedrock incision", but in the two-plane model the strata record
    /// mirrors `H`, not `R` — `R` is basement everywhere. So bed-to-bed contrast
    /// (sandstone standing over mudstone) necessarily rides on **cover
    /// entrainment**, and the incision term carries the basement contrast. Both
    /// are coupled here, through the same per-cell susceptibility, because
    /// wherever cover is thin enough for incision to matter the outcropping
    /// lithology *is* what the flow is grinding.
    ///
    /// Purely per-cell → byte-identical parallel. When the coupling is off this
    /// leaves the planes empty and every consumer reads the exact identity.
    pub fn expose(&mut self, grid: &DeepGrid, cfg: &DeepConfig) {
        if !cfg.erodibility {
            self.litho.clear();
            self.sus_flow.clear();
            self.sus_creep.clear();
            return;
        }
        // Six-entry tables, rebuilt each epoch (cheap, and keeps the knobs live
        // if a harness mutates the config between steps).
        let flow_tab = lithology::susceptibility_table(
            Agent::Abrasion,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let creep_tab = lithology::susceptibility_table(
            Agent::Abrasion,
            cfg.erodibility_diffusion_contrast,
            cfg.erodibility_max,
        );
        if self.litho.len() != self.n {
            self.litho = vec![0u8; self.n];
            self.sus_flow = vec![1.0; self.n];
            self.sus_creep = vec![1.0; self.n];
        }
        let strata = &grid.strata;
        let per_cell = |i: usize| -> (u8, f64, f64) {
            let top = strata.get(i).and_then(|s| s.units.last());
            let l = lithology::exposed_litho(top);
            let k = l.index();
            (k as u8, flow_tab[k], creep_tab[k])
        };
        if self.par() {
            self.litho
                .par_iter_mut()
                .zip(self.sus_flow.par_iter_mut())
                .zip(self.sus_creep.par_iter_mut())
                .enumerate()
                .for_each(|(i, ((l, f), c))| {
                    let (li, fi, ci) = per_cell(i);
                    *l = li;
                    *f = fi;
                    *c = ci;
                });
        } else {
            for i in 0..self.n {
                let (li, fi, ci) = per_cell(i);
                self.litho[i] = li;
                self.sus_flow[i] = fi;
                self.sus_creep[i] = ci;
            }
        }
    }

    // ---- phase 1c: periglacial frost (PARALLEL — per-cell independent) -----

    /// Compute the **temperature-gated frost weathering multiplier** per cell for
    /// this epoch (the frost agent, journal/0034). Leaves the plane empty — read
    /// as the identity `1.0` — when `full_agents` is off, so the frost-off path is
    /// byte-identical.
    ///
    /// **Freeze–thaw is maximal in a band around `0°C`, not monotonic with cold.**
    /// Rock shatters where water repeatedly crosses the phase boundary inside its
    /// pores and joints; a permanently frozen summit barely weathers, and so does
    /// a warm lowland. So the multiplier peaks at `0°C` and tapers linearly to
    /// `1.0` at `±frost_band_width_c`. Temperature is the shared climate model
    /// ([`climate::air_temp_c`]) — latitude minus an elevation lapse — so the
    /// periglacial band rides *up* the mountains and *down* the latitudes exactly
    /// where the biotic layer already agrees it freezes (deep time carries no
    /// pregen `temp_c`, but latitude and the eroding surface are enough for an
    /// honest gate).
    ///
    /// The enhancement is weighted by the rock's **frost/ice** resistance axis
    /// ([`Agent::FrostIce`]): a permeable, poorly-cemented bed shatters where a
    /// tight granite endures — the axis 0029 built and left dormant, now read. It
    /// is independent of the erodibility flag (it reads the exposed lithology
    /// straight from the record top), so periglacial shattering works whether or
    /// not the mechanical coupling is on. Purely per-cell → byte-identical
    /// parallel.
    pub fn periglacial(&mut self, grid: &DeepGrid, cfg: &DeepConfig) {
        if !cfg.full_agents {
            self.frost.clear();
            return;
        }
        let frost_tab = lithology::susceptibility_table(
            Agent::FrostIce,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        if self.frost.len() != self.n {
            self.frost = vec![1.0; self.n];
        }
        let (w, gain, width) = (self.w, cfg.frost_weathering_gain, cfg.frost_band_width_c);
        let strata = &grid.strata;
        let (r, h) = (&grid.r, &grid.h);
        let per_cell = |i: usize| -> f64 {
            let gy = i / w;
            let surf = r[i] + h[i];
            let t = f64::from(climate::air_temp_c(grid.lat_deg(gy), surf));
            // Triangular freeze–thaw band centred on 0 °C.
            let band = if width > 0.0 {
                (1.0 - (t / width).abs()).max(0.0)
            } else {
                0.0
            };
            if band <= 0.0 {
                return 1.0;
            }
            let l = lithology::exposed_litho(strata.get(i).and_then(|s| s.units.last()));
            1.0 + gain * band * frost_tab[l.index()]
        };
        if self.par() {
            self.frost
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, f)| *f = per_cell(i));
        } else {
            for i in 0..self.n {
                self.frost[i] = per_cell(i);
            }
        }
    }

    // ---- phase 2: surface snapshot ----------------------------------------

    /// Snapshot the current surface `R + H` into scratch.
    pub fn build_surface(&mut self, grid: &DeepGrid) {
        if self.par() {
            self.surf
                .par_iter_mut()
                .zip(grid.r.par_iter())
                .zip(grid.h.par_iter())
                .for_each(|((s, r), h)| *s = *r + *h);
        } else {
            for i in 0..self.n {
                self.surf[i] = grid.r[i] + grid.h[i];
            }
        }
    }

    // ---- phase 3: priority-flood fill (SCALAR — serial min-heap) ----------

    /// Priority-flood depression fill (Barnes 2014) seeded from sea and border.
    /// Serial by nature (a global elevation-ordered frontier); the measured
    /// deep-time floor S9b could not break without abandoning byte-identity.
    pub fn flood(&mut self) {
        self.heap.clear();
        self.order.clear();
        for f in &mut self.filled {
            *f = f64::INFINITY;
        }
        self.done.iter_mut().for_each(|d| *d = false);
        for i in 0..self.n {
            if self.surf[i] <= self.sea_level || is_border(i, self.w) {
                self.filled[i] = self.surf[i];
                self.heap.push(Reverse(Item {
                    filled: self.filled[i],
                    idx: i as u32,
                }));
            }
        }
        while let Some(Reverse(item)) = self.heap.pop() {
            let i = item.idx as usize;
            if self.done[i] {
                continue;
            }
            self.done[i] = true;
            self.order.push(i as u32);
            let (gx, gy) = coords_of(i, self.w);
            for (dx, dy) in NEIGH8 {
                let Some(j) = in_grid(gx + dx, gy + dy, self.w) else {
                    continue;
                };
                if self.done[j] || self.filled[j].is_finite() {
                    continue;
                }
                self.filled[j] = self.surf[j].max(self.filled[i] + EPS);
                self.heap.push(Reverse(Item {
                    filled: self.filled[j],
                    idx: j as u32,
                }));
            }
        }
    }

    // ---- phase 4: D8 routing (PARALLEL — per-cell independent) -------------

    /// D8 steepest-descent receivers on filled elevation. Sea and border cells
    /// are sinks (`recv = -1`). Per-cell independent → byte-identical parallel.
    pub fn route(&mut self) {
        let (w, sea) = (self.w, self.sea_level);
        let surf = &self.surf;
        let filled = &self.filled;
        if self.par() {
            self.recv
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, r)| *r = route_cell(i, w, surf, filled, sea));
        } else {
            for i in 0..self.n {
                self.recv[i] = route_cell(i, w, surf, filled, sea);
            }
        }
    }

    // ---- phase 5: drainage-area accumulation (SCALAR — flux chain) --------

    /// Drainage area (in cell units) accumulated downstream in descending
    /// filled order. A receiver must see all upstream contributions before it is
    /// routed on — a serial dependency chain; a level-parallel gather would
    /// reorder the sums and break byte-identity.
    pub fn accumulate_area(&mut self) {
        self.area.iter_mut().for_each(|a| *a = 1.0);
        for k in (0..self.order.len()).rev() {
            let i = self.order[k] as usize;
            let rc = self.recv[i];
            if rc >= 0 {
                self.area[rc as usize] += self.area[i];
            }
        }
    }

    // ---- phase 6: stream-power transport (SCALAR — flux chain) ------------

    /// Stream-power transport with cover shielding and explicit flux routing.
    /// Suspended load flows down the receiver chain (`qs[rc] += qs_out`), so a
    /// cell needs its full upstream load before it runs — a serial chain, kept
    /// scalar for byte-identity. Zeroes `dh` (the recorder's net-ΔH scratch).
    pub fn transport(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        self.dh.iter_mut().for_each(|d| *d = 0.0);
        self.qs.iter_mut().for_each(|q| *q = 0.0);
        self.energy.iter_mut().for_each(|e| *e = 0.0);
        for k in (0..self.order.len()).rev() {
            let c = self.order[k] as usize;
            let rc = self.recv[c];
            let qin = self.qs[c];
            if rc < 0 {
                // Sink: everything suspended settles here (marine / border).
                grid.h[c] += qin;
                self.dh[c] += qin;
                self.energy[c] = 0.0;
                continue;
            }
            let rc = rc as usize;
            let s = ((self.filled[c] - self.filled[rc]).max(0.0)) / self.cell_m;
            let ae = if (cfg.m_exp - 0.5).abs() < 1e-9 {
                self.area[c].sqrt()
            } else {
                self.area[c].powf(cfg.m_exp)
            };
            let sn = if (cfg.n_exp - 1.0).abs() < 1e-9 {
                s
            } else {
                s.powf(cfg.n_exp)
            };
            let cap = cfg.k_transport * ae * sn;
            self.energy[c] = cap;
            // The erodibility coupling's fluvial multiplier for this cell: the
            // abrasion susceptibility of whatever lithology outcrops here.
            // Exactly `1.0` when the coupling is off.
            let sus = sus_at(&self.sus_flow, c);
            let qs_out = if qin <= cap {
                let mut room = cap - qin;
                // Entrain the exposed cover. Transport-limited, now scaled by
                // how detachable that rock is: a weak mudstone hands the flow
                // everything it can carry, a competent sandstone hands over less
                // than the flow has room for and the difference is what leaves a
                // resistant bed standing proud. This is where bed-to-bed
                // differential erosion lives (see `expose`).
                //
                // `sus > 1` (rock softer than the fine-clastic reference) can
                // push entrainment past this cell's remaining capacity; that is
                // physical and mass-safe — the excess is routed downstream as
                // suspended load and the receiver, seeing `qin > cap`, deposits
                // it. Over-entrainment also drives `room` negative, which skips
                // incision: a thick soft cover shields the bedrock beneath it,
                // which is correct.
                let ent = grid.h[c].min(room * sus);
                grid.h[c] -= ent;
                self.dh[c] -= ent;
                room -= ent;
                let mut carried = qin + ent;
                // Then incise bedrock, shielded by remaining cover and scaled by
                // the same susceptibility. Where cover is thin enough for this
                // term to matter at all, the outcropping lithology is what the
                // flow is grinding — and on a stripped column that is basement.
                if room > 0.0 {
                    let shield = (-grid.h[c] / cfg.h_star).exp();
                    let inc_pot = cfg.k_bedrock * ae * sn * shield * sus;
                    let floor = grid.surf_at(rc);
                    let max_inc = (grid.r[c] - floor).max(0.0);
                    let inc = inc_pot.min(room).min(max_inc);
                    grid.r[c] -= inc;
                    carried += inc;
                }
                carried
            } else {
                // Over capacity: deposit the excess as alluvium.
                let dep = qin - cap;
                grid.h[c] += dep;
                self.dh[c] += dep;
                cap
            };
            self.qs[rc] += qs_out;
        }
    }

    // ---- phase 7: bedrock weathering (PARALLEL — per-cell independent) -----

    /// Subaerial bedrock → regolith, cover-tapered, scaled by the per-cell biotic
    /// weathering multiplier (S10 lagged coupling — `grid.bio_weather`, empty and
    /// therefore uniform `1.0` when the biotic layer is off), the lithologic
    /// abrasion susceptibility, and the periglacial **frost** multiplier
    /// (journal/0034, empty and uniform `1.0` when the frost agent is off). Purely
    /// local per cell → byte-identical parallel.
    pub fn weather(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        let parallel = self.par();
        let (sea, weathering, h_star) = (self.sea_level, cfg.weathering, cfg.h_star);
        let dh = &mut self.dh;
        let bio = &grid.bio_weather;
        let sus = &self.sus_flow;
        let frost = &self.frost;
        if parallel {
            grid.r
                .par_iter_mut()
                .zip(grid.h.par_iter_mut())
                .zip(dh.par_iter_mut())
                .enumerate()
                .for_each(|(i, ((r, h), d))| {
                    weather_cell(
                        r,
                        h,
                        d,
                        sea,
                        weathering,
                        h_star,
                        wmult_at(bio, i) * sus_at(sus, i) * frost_at(frost, i),
                    );
                });
        } else {
            for (i, ((r, h), d)) in grid
                .r
                .iter_mut()
                .zip(grid.h.iter_mut())
                .zip(dh.iter_mut())
                .enumerate()
            {
                weather_cell(
                    r,
                    h,
                    d,
                    sea,
                    weathering,
                    h_star,
                    wmult_at(bio, i) * sus_at(sus, i) * frost_at(frost, i),
                );
            }
        }
    }

    /// The drainage area (in cell units) accumulated this step — read by the S10
    /// biotic disturbance phase to find flood-prone valley cells, and exported as
    /// the final-chapter discharge field (§ 7.3, drainage export).
    pub fn area(&self) -> &[f64] {
        &self.area
    }

    /// The D8 receiver of each cell as of the last routing (`-1` = sink) — the
    /// exported final drainage network (§ 7.3). This is the last iteration's
    /// routing exactly (no recompute), so the export matches what the sim used.
    pub fn recv(&self) -> &[i32] {
        &self.recv
    }

    /// The filled (depression-filled) surface from the last flood — used to build
    /// the exported lake mask (a cell whose fill sits above its own surface).
    pub fn filled(&self) -> &[f64] {
        &self.filled
    }

    /// The pre-erosion surface snapshot from the last routing.
    pub fn routed_surface(&self) -> &[f64] {
        &self.surf
    }

    // ---- phase 8: hillslope diffusion (PARALLEL — gather form) ------------

    /// Flux-limited hillslope diffusion of the regolith along the surface
    /// gradient, in **gather** form: fluxes are computed from a frozen surface
    /// and a frozen per-cell limiter, then each cell sums its own in/out edges.
    /// Conserves `ΣH` exactly (each edge's flux is referenced identically from
    /// both endpoints) and is byte-identical scalar↔parallel.
    pub fn diffuse(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        if cfg.diffusion <= 0.0 {
            return;
        }
        let parallel = self.par();
        let (w, diff) = (self.w, cfg.diffusion);
        // Freeze the surface.
        self.build_surface(grid);
        // Pass 1: per-cell limiter scale on the frozen surface.
        {
            let surf = &self.surf;
            let scale = &mut self.scale;
            let h = &grid.h;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            if parallel {
                scale.par_iter_mut().enumerate().for_each(|(i, sc)| {
                    *sc = diffuse_scale_cell(i, w, surf, h[i], diff, resist, sus)
                });
            } else {
                for i in 0..self.n {
                    scale[i] = diffuse_scale_cell(i, w, surf, h[i], diff, resist, sus);
                }
            }
        }
        // Pass 2: gather net ΔH per cell.
        {
            let surf = &self.surf;
            let scale = &self.scale;
            let netdiff = &mut self.netdiff;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            if parallel {
                netdiff.par_iter_mut().enumerate().for_each(|(i, nd)| {
                    *nd = diffuse_net_cell(i, w, surf, scale, diff, resist, sus)
                });
            } else {
                for (i, nd) in netdiff.iter_mut().enumerate() {
                    *nd = diffuse_net_cell(i, w, surf, scale, diff, resist, sus);
                }
            }
        }
        // Apply: h += net, dh += net (disjoint per-cell writes).
        if parallel {
            grid.h
                .par_iter_mut()
                .zip(self.dh.par_iter_mut())
                .zip(self.netdiff.par_iter())
                .for_each(|((h, d), nd)| {
                    *h += *nd;
                    *d += *nd;
                });
        } else {
            for i in 0..self.n {
                grid.h[i] += self.netdiff[i];
                self.dh[i] += self.netdiff[i];
            }
        }
    }

    // ---- phase 9: strata recorder (PARALLEL — per-cell independent) --------

    /// Record each cell's net thickness change this iteration under the tag
    /// measured now. Each cell's `DeepStrata` is independent → byte-identical
    /// parallel (per-cell record ops, disjoint records).
    pub fn record(&mut self, grid: &mut DeepGrid) {
        let parallel = self.par();
        let sea = self.sea_level;
        let chapter = self.cur_chapter;
        let (r, h, precip, energy) = (&grid.r, &grid.h, &grid.precip, &self.energy);
        let dh = &self.dh;
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(s, dh[i], tag, chapter);
            });
        } else {
            for i in 0..self.n {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(&mut grid.strata[i], dh[i], tag, chapter);
            }
        }
    }

    // ---- phase 10: eolian transport (SCALAR — per-row wind march) ---------

    /// **Wind deflation and downwind loess/dune deposition** (the eolian agent,
    /// [`Agent::Eolian`], journal/0034). A 1D march along the **prevailing-wind**
    /// direction the climate already uses ([`climate::wind_dx`] — never a second
    /// wind): dry, unvegetated, subaerial cells hand loose cover to an airborne
    /// load; vegetated or humid downwind cells trap it. In the arid source zone
    /// the trapped sand records as a **dune field** ([`Eolian::Dune`], coarse); on
    /// the damp margin the fine silt records as a **loess** sheet
    /// ([`Eolian::Loess`], fine). At 460 m the unit is the *region*, not the
    /// individual dune (earth-processes.md § 4).
    ///
    /// Wind only **redistributes** loose `H` (it never touches bedrock, and never
    /// adds external mass), so the mass ledger `Δ(ΣR+ΣH) == uplift + biotic` is
    /// untouched: each row conserves its own airborne load, and whatever is still
    /// aloft at the downwind edge settles there. The deflation susceptibility is
    /// the rock's **eolian** resistance axis (cohesion — loose sand blows, crusted
    /// clay resists), scaled by how arid and how bare the cell is.
    ///
    /// Runs **after** the recorder and self-records (like the biotic layer), so
    /// its own facies reach the record rather than being lumped under the epoch's
    /// fluvial tag. Scalar in both drivers (a row carries a serial load), so it is
    /// deterministic and scalar↔parallel byte-identical.
    pub fn wind(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        let record = !grid.strata.is_empty();
        let chapter = self.cur_chapter;
        let sus_tab = lithology::susceptibility_table(
            Agent::Eolian,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        let (w, thr, sea) = (self.w, cfg.eolian_arid_precip, self.sea_level);
        let (defl, dep_frac) = (cfg.eolian_deflation, cfg.eolian_deposit_frac);
        for gy in 0..w {
            let dx = climate::wind_dx(grid.lat_deg(gy));
            let mut load = 0.0f64;
            let mut last_land: Option<usize> = None;
            for s in 0..w {
                let gx = if dx > 0 { s } else { w - 1 - s };
                let i = gy * w + gx;
                let surf = grid.r[i] + grid.h[i];
                if surf <= sea {
                    // Over water: the airborne load settles out (dust on the sea).
                    if load > 0.0 {
                        grid.h[i] += load;
                        if record {
                            let tag = DepTag {
                                eolian: Eolian::Loess,
                                ..DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low)
                            };
                            grid.strata[i].deposit(tag, load, chapter);
                        }
                        load = 0.0;
                    }
                    continue;
                }
                last_land = Some(i);
                let precip = f64::from(grid.precip[i]);
                let arid = if thr > 0.0 {
                    ((thr - precip) / thr).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let veg = if grid.bio_resist.is_empty() {
                    0.0
                } else {
                    f64::from(grid.bio_resist[i])
                };
                // Deflation: dry, bare cells hand loose cover to the wind. Floor
                // available cover at zero first — `H` can carry a sub-ULP negative
                // from fp round-off, and `clamp(0.0, neg)` would panic.
                let l = lithology::exposed_litho(grid.strata.get(i).and_then(|s| s.units.last()));
                let avail = grid.h[i].max(0.0);
                let pickup = (defl * sus_tab[l.index()] * arid * (1.0 - veg)).clamp(0.0, avail);
                if pickup > 0.0 {
                    grid.h[i] -= pickup;
                    load += pickup;
                    if record {
                        grid.strata[i].erode(pickup);
                    }
                }
                // Deposition: vegetated or humid ground traps the load.
                let trap = (1.0 - arid).max(veg).clamp(0.0, 1.0);
                let drop = load * dep_frac * trap;
                if drop > 0.0 {
                    grid.h[i] += drop;
                    load -= drop;
                    if record {
                        // Dune field in the hyper-arid sand-source core, loess on
                        // the semi-arid downwind margin. (0.7 is an appearance-
                        // class split — a user-owned magnitude, journal/0034.)
                        let (facies, energy, ar) = if arid > 0.7 {
                            (Eolian::Dune, EnergyBand::High, Aridity::Arid)
                        } else if precip < thr {
                            (Eolian::Loess, EnergyBand::Low, Aridity::Arid)
                        } else {
                            (Eolian::Loess, EnergyBand::Low, Aridity::Humid)
                        };
                        let tag = DepTag {
                            eolian: facies,
                            ..DepTag::mineral(DepEnv::Subaerial, ar, energy)
                        };
                        grid.strata[i].deposit(tag, drop, chapter);
                    }
                }
            }
            // Conserve the row: whatever is still aloft settles at the downwind
            // land edge (so Σ eolian ΔH over the row is zero — pure redistribution).
            if load > 0.0
                && let Some(i) = last_land
            {
                grid.h[i] += load;
                if record {
                    let tag = DepTag {
                        eolian: Eolian::Loess,
                        ..DepTag::mineral(DepEnv::Subaerial, Aridity::Arid, EnergyBand::Low)
                    };
                    grid.strata[i].deposit(tag, load, chapter);
                }
            }
        }
    }

    // ---- phase 11: littoral wave attack (SCALAR — shore cells) ------------

    /// **Wave-cut littoral erosion** at the current sea-level stand (the wave
    /// agent, [`Agent::Wave`], journal/0034). A cell is attacked when it stands in
    /// the freeboard band `(sea, sea + wave_band_m]` **and** touches open water (a
    /// subsea neighbour) — the shoreline. Waves cut it down toward sea level,
    /// clamped so the cell never drops below the stand (a wave-cut platform forms
    /// *at* sea level, it does not dig a hole), scaled by the rock's **wave**
    /// resistance axis (cohesion/jointing — the axis 0029 built and left dormant)
    /// and a freeboard taper (strongest right at the waterline).
    ///
    /// The quarried volume goes **offshore** into the deepest adjacent subsea cell
    /// as marine sediment, so the term is mass-neutral (`ΣR+ΣH` unchanged): loose
    /// cover is entrained first, then bedrock, and the sum is deposited into the
    /// sink. Because the sea-level curve cycles (§ 6), the attacked band sweeps up
    /// and down the coast over the run — which is what records **raised and
    /// drowned wave-cut features** in a column over successive stands.
    ///
    /// Scalar (it writes a neighbour's cell), so deterministic and byte-identical
    /// scalar↔parallel. Only the thin shore band does work.
    pub fn wave(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        let (rate, band) = (cfg.wave_erosion, cfg.wave_band_m);
        if rate <= 0.0 || band <= 0.0 {
            return;
        }
        let record = !grid.strata.is_empty();
        let chapter = self.cur_chapter;
        let (w, sea) = (self.w, self.sea_level);
        let sus_tab = lithology::susceptibility_table(
            Agent::Wave,
            cfg.erodibility_contrast,
            cfg.erodibility_max,
        );
        for i in 0..self.n {
            let free = grid.r[i] + grid.h[i] - sea;
            if free <= 0.0 || free > band {
                continue; // below water, or too high up the shore to be reached
            }
            // Deepest adjacent subsea cell is the offshore sink (deterministic).
            let (gx, gy) = coords_of(i, w);
            let mut sink: Option<usize> = None;
            let mut sink_surf = f64::INFINITY;
            for (dx, dy) in NEIGH8 {
                if let Some(j) = in_grid(gx + dx, gy + dy, w) {
                    let sj = grid.r[j] + grid.h[j];
                    if sj <= sea && sj < sink_surf {
                        sink_surf = sj;
                        sink = Some(j);
                    }
                }
            }
            let Some(j) = sink else {
                continue; // not on the coast — no open water adjacent
            };
            let l = lithology::exposed_litho(grid.strata.get(i).and_then(|s| s.units.last()));
            let taper = (1.0 - free / band).clamp(0.0, 1.0);
            let cut = (rate * sus_tab[l.index()] * taper).min(free);
            if cut <= 0.0 {
                continue;
            }
            // Entrain loose cover first, then quarry bedrock; the sum goes offshore.
            let removed_h = grid.h[i].min(cut);
            grid.h[i] -= removed_h;
            grid.r[i] -= cut - removed_h;
            grid.h[j] += cut;
            if record {
                if removed_h > 0.0 {
                    grid.strata[i].erode(removed_h);
                }
                grid.strata[j].deposit(
                    DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low),
                    cut,
                    chapter,
                );
            }
        }
    }
}
