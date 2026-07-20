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
//! agent reads an abrasion axis, and the designed-but-unbuilt dissolution,
//! frost/ice and littoral agents each have their own; see the lithology module
//! docs for why a one-number erodibility would foreclose karst. Off by default,
//! and with it off every multiplier is the exact identity `1.0`, so the
//! uncoupled path is byte-identical.
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

use super::grid::{DeepConfig, DeepGrid, SEA_LEVEL_M};
use super::lithology::{self, Agent, Litho};
use super::recorder::{Aridity, DeepStrata, DepEnv, DepTag, EnergyBand};

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
/// multiplication is not associative): the two modifier layers combine with each
/// other first — `rate_mult = wmult × litho_sus`, biology on the left — and the
/// product then scales the base rate before the cover taper:
/// `weathering × rate_mult × taper`. With lithology off, `wmult × 1.0 == wmult`
/// exactly, so the S10 expression is reproduced bit for bit.
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

/// Apply one cell's net thickness change to its strata record under `tag`.
#[inline]
fn record_cell(s: &mut DeepStrata, dh: f64, tag: DepTag) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        s.deposit(tag, dh);
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
    heap: BinaryHeap<Reverse<Item>>,
}

impl Erosion {
    pub fn new(grid: &DeepGrid) -> Self {
        let n = grid.w * grid.w;
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
            heap: BinaryHeap::new(),
        }
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
            + self.sus_creep.len();
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
        let uplift_added = self.apply_uplift(grid);
        self.expose(grid, cfg);
        self.build_surface(grid);
        self.flood();
        self.route();
        self.accumulate_area();
        self.transport(grid, cfg);
        self.weather(grid, cfg);
        self.diffuse(grid, cfg);
        if cfg.record {
            self.record(grid);
        }
        uplift_added
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
    /// therefore uniform `1.0` when the biotic layer is off). Purely local per
    /// cell → byte-identical parallel.
    pub fn weather(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        let parallel = self.par();
        let (sea, weathering, h_star) = (self.sea_level, cfg.weathering, cfg.h_star);
        let dh = &mut self.dh;
        let bio = &grid.bio_weather;
        let sus = &self.sus_flow;
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
                        wmult_at(bio, i) * sus_at(sus, i),
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
                    wmult_at(bio, i) * sus_at(sus, i),
                );
            }
        }
    }

    /// The drainage area (in cell units) accumulated this step — read by the S10
    /// biotic disturbance phase to find flood-prone valley cells.
    pub fn area(&self) -> &[f64] {
        &self.area
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
        let (r, h, precip, energy) = (&grid.r, &grid.h, &grid.precip, &self.energy);
        let dh = &self.dh;
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(s, dh[i], tag);
            });
        } else {
            for i in 0..self.n {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(&mut grid.strata[i], dh[i], tag);
            }
        }
    }
}
