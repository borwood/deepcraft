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
use super::providers::WaveCell;
use super::recorder::{Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, Eolian};
use super::weather_behavior;

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

/// **The MFD partition** — the number of D8 directions, and the two geometric
/// constants a multi-receiver partition needs that a single-receiver one never
/// had to name.
///
/// `MFD_DIST` is the true flow-path length in cell widths (`1` cardinal, `√2`
/// diagonal): a steepest-*drop* rule can ignore it, a slope-weighted partition
/// cannot, or every diagonal is over-weighted by `√2` and the drainage net
/// acquires a systematic X-bias.
///
/// `MFD_CONTOUR` is Quinn (1991)'s **contour width** — the length of the cell
/// boundary the flow crosses, normalised to the cardinal case (`0.5Δ` cardinal,
/// `0.354Δ` diagonal ⇒ `1` and `1/√2`). It is the *width of the gate*, not the
/// steepness of the drop, and it is why a diagonal neighbour receives less than a
/// cardinal one at equal slope.
const MFD_DIRS: usize = 8;
const SQRT2: f64 = std::f64::consts::SQRT_2;
const MFD_DIST: [f64; MFD_DIRS] = [SQRT2, 1.0, SQRT2, 1.0, 1.0, SQRT2, 1.0, SQRT2];
const MFD_CONTOUR: [f64; MFD_DIRS] = [
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
    1.0,
    std::f64::consts::FRAC_1_SQRT_2,
];

/// **The representational floor on a partition share.** A neighbour allotted less
/// than this fraction of the cell's discharge is dropped and the survivors
/// renormalised.
///
/// It is not a physical parameter — it is what keeps the *record* honest and
/// affordable. Without it every land cell dumps an infinitesimal trickle into
/// every downslope neighbour, the flux record grows an entry for each, and the
/// archive pays megabytes to store noise that no consumer can distinguish from
/// zero. The steepest receiver always carries at least `1/8` of the discharge, so
/// a survivor always exists and no cell is ever turned into a sink by the floor.
///
/// **⚠ STUB #22 — a record-affordability constant that changes the physics.** The
/// floor is applied *here*, before renormalisation, so the surviving receivers are
/// handed the dropped share and the **solve** moves water it otherwise would not.
/// A requirement of the *record* has leaked into the *landscape*: by
/// ARCHITECTURE.md's test, if the record consumer vanished tomorrow this constant
/// would not exist in this shape. It is inert on the near-flat ground the slice is
/// for (shares near `1/6` there, far above the floor) and bites in the moderately
/// convergent regime, making the net slightly more channelised than the exponent
/// alone specifies — argued, **not measured**. Mass is unaffected: renormalisation
/// is exact and the residual rule makes the split exact.
/// **Heir:** whoever settles the record's size budget (flow.md § 9 item 7b, the
/// aggregation window; the marine-sink lever) — and they should decide whether the
/// solve may see this at all, the alternative being a floor applied only on the
/// way into the record. See `docs/design/stubs.md` § 22.
const MFD_MIN_WEIGHT: f64 = 0.01;

/// The number of **species** the suspended load is resolved into — one per
/// [`Litho`], which is the material granularity deep time can distinguish at all
/// (`lithology.rs`: *"not the material registry — the handful of classes the
/// deep-time record can distinguish"*). Members within a class are chosen at
/// collapse time, so resolving the load any finer than this would be inventing
/// identity the tier does not have.
const SPECIES: usize = Litho::COUNT;

/// **The competence ceiling, per unit of transport capacity** — the one
/// calibration constant material-aware transport adds, in
/// `settle_energy`-units per (metre/iteration) of stream capacity.
///
/// `material-behavior.md` § 13.5: *"sorting is the falling ceiling; we write the
/// ceiling, not the sort."* This is that ceiling. A flow of capacity `cap` can
/// hold in suspension every species whose settling velocity is at most
/// `COMPETENCE_SCALE · cap`; everything heavier rains out **wherever it is**,
/// regardless of whether the flow still has capacity to spare. Capacity is the
/// *total mass* limit and competence is the *size/density* limit, and § 13.5 is
/// explicit that both are needed: without competence a flow with spare capacity
/// carries boulders to the sea, and nothing ever fines downstream.
///
/// **Where the number comes from, and why it is not a tuning knob.** It is fixed
/// by an anchor that already ships: [`energy_band`] calls a capacity of `0.002`
/// the Low/Medium boundary, and `litho_of_tag` turns exactly that boundary into
/// the coarse/fine clastic split — so `0.002` is *already* the world's stated
/// "energy at which sand stops moving". `settle_energy` puts the coarse-clastic
/// reference sheet at `≈0.84`, and `0.84 / 0.002 = 420`. The ceiling therefore
/// crosses the coarse-clastic threshold at precisely the capacity the shipped
/// facies rule already crosses it at, and the rest of the roster arranges itself
/// around that: a trunk reach at `cap = 0.02` lifts everything the world has
/// (ceiling `8.4`, against basement's `2.85`), and a distal reach at
/// `cap = 2 × 10⁻⁴` cannot hold mud (ceiling `0.084`, against mudstone's `0.098`).
/// The dynamic range is the roster's, not a fit.
///
/// It is **linear** in capacity because capacity is already a stream-power proxy
/// (`k·A^m·S^n`) and competence in a real channel scales with a power of stream
/// power; linear is the simplest form that spans the roster, in the same
/// plausible-not-tuned register as S9's physics constants. Chosen and written
/// **before** the outcome probe was run, and not revisited after.
const COMPETENCE_SCALE: f64 = 420.0;

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

/// **The MFD partition of one cell's discharge** (flow.md § 2.6, § 2.4).
///
/// Writes the eight normalised out-weights of cell `i` into `w_out` (aligned to
/// [`NEIGH8`], and therefore to `FaceKey::lateral`), and returns the direction
/// carrying the largest share (`-1` when the cell is a sink and every weight is
/// zero).
///
/// **The field partitioned is the FREE-SURFACE POTENTIAL, and that is the point.**
/// `filled` is the priority-flood surface: bare ground where the land drains, and
/// the *spill-level water surface* inside every depression. For free-phase flow
/// that is `head = z_bed + depth` exactly — pressure head is zero at a free
/// surface, so a lake's potential is flat and its bed's elevation is irrelevant.
/// So the partition descends a potential rather than a topography, which is what
/// flow.md § 2.4 asks of the free regime. (`dc:field/head` is the **bound**
/// regime's potential — a water table that deliberately crosses surface divides.
/// Routing free surface water down it would make rivers cross divides too, which
/// § 2.4's own qualification forbids. Bound MFD is continuation (c)'s, not this
/// pass's.)
///
/// The weight is **Holmgren (1994)** with Quinn's contour width:
///
/// ```text
///   w_k  ∝  (Δh_k / d_k)^p · L_k          d_k = 1 or √2,  L_k = 1 or 1/√2
/// ```
///
/// `p` is the **convergence exponent**: `p = 1` is Quinn's maximally dispersive
/// form, `p → ∞` is single-receiver D8. Nothing here is novel; what it buys over
/// D8 is that a cell's discharge can leave through more than one face **within one
/// epoch**, which is the entire difference between temporal divergence (avulsion,
/// which the record already had) and simultaneous divergence (concurrent
/// distributaries, which it structurally could not hold).
#[inline]
#[allow(clippy::too_many_arguments)]
fn partition_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    filled: &[f64],
    sea_level: f64,
    p: f64,
    int_p: Option<i32>,
    w_out: &mut [f64],
) -> i32 {
    w_out.iter_mut().for_each(|v| *v = 0.0);
    if surf[i] <= sea_level || is_border(i, w) {
        return -1;
    }
    let (gx, gy) = coords_of(i, w);
    let fi = filled[i];
    let mut sum = 0.0;
    for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
        let Some(j) = in_grid(gx + dx, gy + dy, w) else {
            continue;
        };
        let drop = fi - filled[j];
        if drop <= 0.0 {
            continue;
        }
        let s = drop / MFD_DIST[d];
        // Integer exponents go through `powi` — the default `p = 4` is three
        // multiplies, where `powf` on 8 directions × 297k cells × 200 epochs is
        // half a billion transcendental calls and would dominate the deep run.
        let sp = match int_p {
            Some(k) => s.powi(k),
            None => s.powf(p),
        };
        let raw = sp * MFD_CONTOUR[d];
        w_out[d] = raw;
        sum += raw;
    }
    if sum <= 0.0 {
        return -1;
    }
    // Normalise, then apply the representational floor and renormalise over the
    // survivors. The largest share is at least `1/8` before the floor, so the
    // survivor set is never empty and `sum2 > 0` always.
    let mut sum2 = 0.0;
    for v in w_out.iter_mut() {
        let n = *v / sum;
        *v = if n >= MFD_MIN_WEIGHT { n } else { 0.0 };
        sum2 += *v;
    }
    let mut best = -1i32;
    let mut best_w = 0.0;
    for (d, v) in w_out.iter_mut().enumerate() {
        if *v <= 0.0 {
            continue;
        }
        *v /= sum2;
        if *v > best_w {
            best_w = *v;
            best = d as i32;
        }
    }
    best
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

// The per-cell weathering kernel now lives in the north-star behavior shape
// (`weather_behavior::weather_one_cell` / `WeatheringPass`, S16). The domain
// narrative that used to sit here — **why it is the rate-limiting phase on
// hillslopes** (journal/0029: diffusion is flux-limited by the regolith actually
// available, so the landscape's lowering rate collapses to the rate bedrock is
// *converted* to regolith, which is why weathering had to be coupled at all),
// and **why the factor is a sum/product over agents** (in-place weathering is not
// one process; today the mechanical/abrasion term plus the periglacial frost
// term, and the day the dissolution agent lands a limestone weathers *fast*
// through the chemical term while resisting the mechanical one — the karst story
// arriving without a rewrite) — carries over unchanged; the composition order is
// pinned in `BedrockWeather::weather_rate`.

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
fn record_cell(s: &mut DeepStrata, dh: f64, tag: DepTag, chapter: u8, species: Litho) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        s.deposit_as(tag, dh, chapter, species);
    } else {
        s.erode(-dh);
    }
}

/// **Which material the unit arriving at this cell is made of** (Movement 2b).
///
/// The cell's net gain has several sources and only one of them carried an
/// identity here: the transport pass knows, per species, exactly what it set
/// down (`dep`). Everything else in `dh` — bedrock weathered to regolith in
/// place, cover crept in from a neighbour — is material that never rode a load,
/// and it keeps the answer the record has always given, the tag's own lithology.
/// So the unit's species is the **argmax of the whole mixture**: each transported
/// species against the un-transported remainder, with ties going to the
/// incumbent (a strict `>` over fixed index order, so it is deterministic).
///
/// This is deliberately *not* a threshold on "was most of this transported" —
/// a threshold would be a second rule with a number in it. It is one comparison
/// over one mixture, and it degenerates exactly to the old behaviour when the
/// pass carried nothing here.
#[inline]
fn arriving_species(dh: f64, dep: &[f64], tag_species: Litho) -> Litho {
    let mut best = tag_species;
    let mut best_m = dh - dep.iter().sum::<f64>();
    for (k, &m) in dep.iter().enumerate() {
        if m > best_m {
            best_m = m;
            best = Litho::ALL[k];
        }
    }
    best.as_deposited()
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
    /// **Per-cell suspended load leaving toward the receiver this epoch** (the
    /// `qs_out` of [`Self::transport`]). Empty unless the flow record is on
    /// ([`Self::set_flux_record`]) — and then it is a pure side-write off the
    /// transport chain, so the erosion result is bit-for-bit unchanged either way.
    /// It is the **L** of the flow atom (flow.md § 1.3): the only place the load
    /// crossing a face is ever visible, because `qs` afterwards holds each cell's
    /// *in*-load summed over contributors and cannot be factored back apart.
    out_load: Vec<f64>,
    /// **The MFD partition** (flow.md § 2.6). `Some(p)` ⇒ multi-receiver routing
    /// at convergence exponent `p`; `None` ⇒ the single-receiver D8 path, which is
    /// the pre-MFD solve byte for byte. `mfd_int_p` caches `p` when it is a small
    /// integer so the hot inner loop uses `powi` instead of `powf`.
    mfd: Option<f64>,
    mfd_int_p: Option<i32>,
    /// `n × 8` normalised out-weights, aligned to [`NEIGH8`] (and therefore to
    /// `FaceKey::lateral`). Empty when MFD is off.
    mfd_w: Vec<f64>,
    /// **Per-face outgoing discharge and load** — `n × 8`, f32, gen-time scratch,
    /// empty unless the flow record is on ([`Self::set_flux_record`]).
    ///
    /// This is *what the solve actually moved*, written by the phase that moved
    /// it: `out_area` by [`Self::accumulate_area`], `out_face_load` by
    /// [`Self::transport`]. The record reads these rather than re-deriving shares
    /// from the weights — two derivations of one quantity is exactly the drift
    /// flow.md § 3 exists to prevent, and it would put the flux record and the
    /// mass budget on different arithmetic.
    out_area: Vec<f32>,
    out_face_load: Vec<f32>,
    /// **Material-aware transport** (Movement 2b). Off ⇒ every vector below is
    /// empty, no branch in [`Self::exchange_cell`] fires, and the pass is the
    /// scalar solve byte for byte.
    sorted: bool,
    /// The settling velocity of each species (`settling_table`), and the species
    /// indices sorted **descending** by it — the order deposition draws in, which
    /// is the only place "the load is kept sorted" is cashed out. Sorting a
    /// seven-element array once per run is cheaper and more honest than keeping a
    /// sorted structure per cell: the ordering is a property of the *materials*,
    /// not of any particular load.
    w_settle: [f64; SPECIES],
    ws_order: [usize; SPECIES],
    /// The composition of the material **below the record** — what incision
    /// detaches. Asked of the *same* near-surface-composition seam every other
    /// consumer uses, with an **empty section**: a window containing no recorded
    /// units is entirely whatever lies beneath the pile. So the pass names no
    /// lithology; it asks a question and the seam answers. (Refreshed each epoch
    /// in [`Self::expose`], because the seam's heir — structural deformation — may
    /// one day answer it differently per cell, at which point this becomes a plane
    /// rather than a constant.)
    bedrock_sp: [f64; SPECIES],
    /// The index of the last non-zero entry of [`Self::bedrock_sp`] — the species
    /// that takes the residual, so an incision split is exact by construction.
    last_bedrock_species: usize,
    /// **The load itself** — `n × SPECIES` metres of suspended material, the
    /// multiset of § 13.3. Cell `c`'s slice holds its *in*-load while upstream
    /// cells are still contributing, and is rewritten in place by
    /// [`Self::exchange_cell`] to hold its *out*-load; the chain is strictly
    /// downstream-ordered, so a cell's slice is never read after it is spent.
    ///
    /// **This vector is the mass authority when it is non-empty.** The scalar
    /// `qs` plane is not maintained on the sorted path at all: `qin` is summed
    /// from here and the per-face scalar written to the flux record is the sum of
    /// the per-species shares — one arithmetic, so the record and the budget
    /// cannot drift (flow.md § 3).
    qs_sp: Vec<f64>,
    /// `n × SPECIES` — the composition of the **surface loose** at each cell, from
    /// the record's own near-surface window (the `outcrop_shares` seam). This is
    /// the identity entrainment removes: a reach cutting a sandstone bench hands
    /// the flow sand, and a stripped column hands it basement debris.
    shares: Vec<f64>,
    /// `n × SPECIES` — what the transport pass **set down** at each cell this
    /// epoch, per species. Read by [`Self::record`] to name the arriving unit, and
    /// by the outcome probe to measure the downstream fining gradient.
    dep_sp: Vec<f64>,
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
            out_load: Vec::new(),
            mfd: None,
            mfd_int_p: None,
            mfd_w: Vec::new(),
            out_area: Vec::new(),
            out_face_load: Vec::new(),
            sorted: false,
            w_settle: [0.0; SPECIES],
            ws_order: [0; SPECIES],
            bedrock_sp: [0.0; SPECIES],
            last_bedrock_species: 0,
            qs_sp: Vec::new(),
            shares: Vec::new(),
            dep_sp: Vec::new(),
            heap: BinaryHeap::new(),
        }
    }

    /// Turn **material-aware transport** on (Movement 2b, `material-behavior.md`
    /// § 13.3–13.6). Off is the scalar-load solve, byte for byte, because every
    /// species vector stays empty and every branch that reads one is skipped.
    ///
    /// The settling order is computed once here: a stable descending sort of the
    /// species by [`lithology::settling_table`], ties broken by index so the draw
    /// order is deterministic across platforms.
    pub fn set_material_transport(&mut self, on: bool) {
        self.sorted = on;
        if !on {
            self.qs_sp = Vec::new();
            self.shares = Vec::new();
            self.dep_sp = Vec::new();
            return;
        }
        self.w_settle = lithology::settling_table();
        let mut order: [usize; SPECIES] = std::array::from_fn(|i| i);
        let w = self.w_settle;
        order.sort_by(|&a, &b| w[b].total_cmp(&w[a]).then(a.cmp(&b)));
        self.ws_order = order;
        if self.qs_sp.len() != self.n * SPECIES {
            self.qs_sp = vec![0.0; self.n * SPECIES];
            self.dep_sp = vec![0.0; self.n * SPECIES];
        }
    }

    /// Whether material-aware transport is on.
    #[inline]
    pub fn is_material_transport(&self) -> bool {
        self.sorted
    }

    /// The settling velocity of each species, indexed by [`Litho::index`] (all
    /// zero when material-aware transport is off).
    pub fn settling(&self) -> &[f64; SPECIES] {
        &self.w_settle
    }

    /// **What the transport pass set down at each cell in the last epoch, per
    /// species** (`n × SPECIES` metres; empty when material-aware transport is
    /// off). The measurement surface for the downstream-fining gradient — and the
    /// authority [`Self::record`] names the arriving unit from.
    pub fn deposited_species(&self) -> &[f64] {
        &self.dep_sp
    }

    /// Turn the **flow record's** per-epoch per-face capture on. Off (the
    /// default) the buffers stay empty and neither [`Self::accumulate_area`] nor
    /// [`Self::transport`] touches them, so every direct caller of `step` keeps
    /// the byte-identical old behaviour.
    pub fn set_flux_record(&mut self, on: bool) {
        if on && self.out_load.len() != self.n {
            self.out_load = vec![0.0; self.n];
            self.out_area = vec![0.0; self.n * MFD_DIRS];
            self.out_face_load = vec![0.0; self.n * MFD_DIRS];
        } else if !on {
            self.out_load = Vec::new();
            self.out_area = Vec::new();
            self.out_face_load = Vec::new();
        }
    }

    /// Turn **multiple-flow-direction routing** on at convergence exponent `p`
    /// (flow.md § 2.6). `None` is the single-receiver D8 path — the pre-MFD solve,
    /// byte for byte, because nothing else in the class reads `mfd_w`.
    pub fn set_mfd(&mut self, p: Option<f64>) {
        self.mfd = p;
        self.mfd_int_p = p.and_then(|p| {
            let r = p.round();
            // `powi` is exact for the integer case and cheap; the guard keeps the
            // fallback for the fractional exponents a probe may sweep.
            (p == r && (1.0..=16.0).contains(&r)).then_some(r as i32)
        });
        if p.is_some() {
            if self.mfd_w.len() != self.n * MFD_DIRS {
                self.mfd_w = vec![0.0; self.n * MFD_DIRS];
            }
        } else {
            self.mfd_w = Vec::new();
        }
    }

    /// Whether MFD routing is on.
    #[inline]
    pub fn is_mfd(&self) -> bool {
        self.mfd.is_some()
    }

    /// The suspended load each cell handed to its receivers in the last
    /// [`Self::transport`], **summed over faces** (empty unless
    /// [`Self::set_flux_record`] is on).
    pub fn out_load(&self) -> &[f64] {
        &self.out_load
    }

    /// **The discharge that left each cell through each of its eight lateral
    /// faces** this epoch (`n × 8`, aligned to `FaceKey::lateral`) — the flow
    /// record's `M`. Empty unless [`Self::set_flux_record`] is on. A cell whose
    /// eight entries are all zero is a **sink**: its discharge left through a
    /// boundary face, which is the record's call to make, not the solve's.
    pub fn out_face_area(&self) -> &[f32] {
        &self.out_area
    }

    /// **The suspended load that crossed each of the eight lateral faces** this
    /// epoch (`n × 8`) — the flow record's `L`, partitioned by exactly the same
    /// shares as the discharge. Empty unless [`Self::set_flux_record`] is on.
    pub fn out_face_load(&self) -> &[f32] {
        &self.out_face_load
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
            + self.frost.len()
            + self.mfd_w.len()
            + self.qs_sp.len()
            + self.shares.len()
            + self.dep_sp.len();
        f64s * 8
            + self.recv.len() * 4
            + self.order.capacity() * 4
            + self.done.len()
            + self.litho.len()
            + (self.out_area.len() + self.out_face_load.len()) * 4
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
    pub fn snapshot_bedrock(&mut self, grid: &DeepGrid) {
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
        // Movement 2b: the same near-surface window, read for a different
        // question — not "how fast does this cell erode" but "**what is it made
        // of**", which is the identity entrainment lifts into the load. One walk,
        // two consumers; the erodibility blend and the entrainment composition can
        // never disagree about what is lying at the surface.
        if self.sorted {
            if self.shares.len() != self.n * SPECIES {
                self.shares = vec![0.0; self.n * SPECIES];
            }
            // What lies below the record, asked of the seam with an empty section.
            self.bedrock_sp
                .copy_from_slice(cfg.providers.outcrop_shares(&[]).shares());
            self.last_bedrock_species = (0..SPECIES)
                .rev()
                .find(|&k| self.bedrock_sp[k] > 0.0)
                .unwrap_or(0);
            let strata = &grid.strata;
            let providers = cfg.providers;
            let compose = |i: usize, out: &mut [f64]| {
                let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
                out.copy_from_slice(providers.outcrop_shares(units).shares());
            };
            if self.par() {
                self.shares
                    .par_chunks_mut(SPECIES)
                    .enumerate()
                    .for_each(|(i, out)| compose(i, out));
            } else {
                for (i, out) in self.shares.chunks_mut(SPECIES).enumerate() {
                    compose(i, out);
                }
            }
        }
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
        // The outcrop-shares seam (providers.rs § `outcrop_shares`): identity = the
        // per-Litho shares of the near-surface window; heir = structural deformation
        // (dip/fold), a pinned pair with `outcrop_at`. Erosion blends the
        // susceptibility table by share rather than taking the window's argmax and
        // stepping the rate discontinuously at the plurality crossover — the S-4
        // flag from walk-0071, cured by construction (journal/0072). Argmax is the
        // degenerate case: a single-lithology window blends to that rock's rate bit
        // for bit. `litho[i]` (the debug outcrop, read by `exposed()`) stays the
        // dominant share.
        let providers = cfg.providers;
        let per_cell = |i: usize| -> (u8, f64, f64) {
            let units = strata.get(i).map_or(&[][..], |s| s.units.as_slice());
            let shares = providers.outcrop_shares(units);
            let k = lithology::dominant_litho(&shares).index() as u8;
            (
                k,
                lithology::blend_susceptibility(&shares, &flow_tab),
                lithology::blend_susceptibility(&shares, &creep_tab),
            )
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
        let providers = cfg.providers;
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
            let shares =
                providers.outcrop_shares(strata.get(i).map_or(&[][..], |s| s.units.as_slice()));
            1.0 + gain * band * lithology::blend_susceptibility(&shares, &frost_tab)
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

    /// Receivers on the filled free-surface potential. Sea and border cells are
    /// sinks (`recv = -1`). Per-cell independent → byte-identical parallel.
    ///
    /// **Two routings, one phase.** With MFD off this is the historical D8
    /// steepest-descent rule and `recv` *is* the routing. With MFD on the phase
    /// computes the full eight-way partition into `mfd_w` — that is what the
    /// accumulation and transport chains then read — and `recv` becomes the
    /// **argmax share**: a projection of the partition, kept because the exported
    /// drainage network and the biotic layer still consume a single receiver. It is
    /// no longer the routing, and that is the interesting fact about it (see the
    /// [`Self::recv`] doc).
    pub fn route(&mut self) {
        let (w, sea) = (self.w, self.sea_level);
        let surf = &self.surf;
        let filled = &self.filled;
        let Some(p) = self.mfd else {
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
            return;
        };
        let int_p = self.mfd_int_p;
        let step = |i: usize, r: &mut i32, ws: &mut [f64]| {
            let best = partition_cell(i, w, surf, filled, sea, p, int_p, ws);
            *r = if best < 0 {
                -1
            } else {
                let (dx, dy) = NEIGH8[best as usize];
                let (gx, gy) = coords_of(i, w);
                in_grid(gx + dx, gy + dy, w).expect("a weighted direction is in-grid") as i32
            };
        };
        if self.par() {
            self.recv
                .par_iter_mut()
                .zip(self.mfd_w.par_chunks_mut(MFD_DIRS))
                .enumerate()
                .for_each(|(i, (r, ws))| step(i, r, ws));
        } else {
            for (i, (r, ws)) in self
                .recv
                .iter_mut()
                .zip(self.mfd_w.chunks_mut(MFD_DIRS))
                .enumerate()
            {
                step(i, r, ws);
            }
        }
    }

    /// The neighbour cell index of direction `d` from cell `i`, valid only where
    /// `mfd_w[i * 8 + d] > 0` (a weighted direction is in-grid by construction).
    #[inline]
    fn mfd_neighbour(&self, i: usize, d: usize) -> usize {
        let (dx, dy) = NEIGH8[d];
        (i as isize + dy as isize * self.w as isize + dx as isize) as usize
    }

    // ---- phase 5: drainage-area accumulation (SCALAR — flux chain) --------

    /// Drainage area (in cell units) accumulated downstream in descending
    /// filled order. A receiver must see all upstream contributions before it is
    /// routed on — a serial dependency chain; a level-parallel gather would
    /// reorder the sums and break byte-identity.
    ///
    /// ## Why the priority-flood order is still a topological order under MFD
    ///
    /// This was the part MFD was expected to break, and it does not. `self.order`
    /// is the priority-flood pop order, which is **strictly ascending in
    /// `(filled, index)`**: each cell is pushed exactly once, with its final
    /// `filled` value, and popped in heap order. Reversed, it is strictly
    /// *descending*. Every routed edge — single-receiver or MFD — goes to a
    /// neighbour with **strictly smaller `filled`** (`partition_cell` weights only
    /// `drop > 0`). So every out-edge points to a cell that comes strictly later in
    /// the reversed scan, and a cell is processed only after every contributor.
    ///
    /// The tree gave a *convenient* traversal; what actually licensed it was the
    /// potential ordering, and that licenses the **DAG** identically. MFD needs no
    /// new topological sort — it needs the observation that the old one was never
    /// about the tree.
    pub fn accumulate_area(&mut self) {
        self.area.iter_mut().for_each(|a| *a = 1.0);
        if self.mfd.is_none() {
            let record = !self.out_area.is_empty();
            if record {
                self.out_area.iter_mut().for_each(|v| *v = 0.0);
            }
            for k in (0..self.order.len()).rev() {
                let i = self.order[k] as usize;
                let rc = self.recv[i];
                if rc >= 0 {
                    self.area[rc as usize] += self.area[i];
                    if record {
                        let (gx, gy) = coords_of(i, self.w);
                        let (jx, jy) = coords_of(rc as usize, self.w);
                        let d = NEIGH8
                            .iter()
                            .position(|&(dx, dy)| (gx + dx, gy + dy) == (jx, jy))
                            .expect("the receiver is a D8 neighbour");
                        self.out_area[i * MFD_DIRS + d] = self.area[i] as f32;
                    }
                }
            }
            return;
        }
        let record = !self.out_area.is_empty();
        if record {
            self.out_area.iter_mut().for_each(|v| *v = 0.0);
        }
        for k in (0..self.order.len()).rev() {
            let i = self.order[k] as usize;
            let base = i * MFD_DIRS;
            let a = self.area[i];
            // The last weighted direction takes the **residual**, so the shares sum
            // to `a` exactly rather than to `a · Σw` with Σw off by an ulp. Mass
            // conservation down a DAG is a chain of these; a per-hop rounding error
            // would compound over a thousand hops and leak silently.
            let Some(last) = (0..MFD_DIRS).rev().find(|&d| self.mfd_w[base + d] > 0.0) else {
                continue;
            };
            let mut given = 0.0;
            for d in 0..MFD_DIRS {
                let wt = self.mfd_w[base + d];
                if wt <= 0.0 {
                    continue;
                }
                let share = if d == last { a - given } else { wt * a };
                given += share;
                let j = self.mfd_neighbour(i, d);
                self.area[j] += share;
                if record {
                    self.out_area[base + d] = share as f32;
                }
            }
        }
    }

    // ---- phase 6: stream-power transport (SCALAR — flux chain) ------------

    /// The **per-cell load exchange** of the transport phase, factored out so the
    /// single-receiver and MFD chains share one arithmetic (they must: two copies
    /// of a mass budget is the drift flow.md § 3 exists to prevent).
    ///
    /// `s` is the energy slope the cell's flow descends and `floor` the elevation
    /// its bedrock may not be cut below. Returns the load handed onward. Writes
    /// `energy[c]` and `dh[c]` and mutates the cell's own `R`/`H` — never a
    /// neighbour's, which is what leaves the *routing* of the result to the caller.
    #[inline]
    fn exchange_cell(
        &mut self,
        grid: &mut DeepGrid,
        cfg: &DeepConfig,
        c: usize,
        qin: f64,
        s: f64,
        floor: f64,
    ) -> f64 {
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
        let base = c * SPECIES;
        // ---- COMPETENCE: the falling ceiling (Movement 2b, § 13.5) ----------
        //
        // Capacity says how much the flow can hold; competence says **what**. A
        // species whose settling velocity exceeds what this cell's energy can lift
        // rains out here, whether or not the flow has capacity to spare — that is
        // the difference between a mass budget and a facies, and it is the entire
        // mechanism by which a load fines downstream. Nothing is sorted: the
        // ceiling falls, and the load is what is left under it.
        let qin = if self.sorted {
            let ceiling = COMPETENCE_SCALE * cap;
            let mut rained = 0.0;
            for k in 0..SPECIES {
                if self.w_settle[k] > ceiling {
                    let m = self.qs_sp[base + k];
                    if m > 0.0 {
                        self.qs_sp[base + k] = 0.0;
                        self.dep_sp[base + k] += m;
                        rained += m;
                    }
                }
            }
            if rained > 0.0 {
                grid.h[c] += rained;
                self.dh[c] += rained;
            }
            // Re-summed rather than decremented: the species vector is the
            // authority, so the scalar every downstream decision reads is *derived
            // from it*, in fixed index order, and cannot drift away from it.
            self.qs_sp[base..base + SPECIES].iter().sum()
        } else {
            qin
        };
        if qin <= cap {
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
            // `H` can carry a sub-ULP negative from round-off; on the scalar path
            // that has always flowed straight through (and `x.max(0.0)` would not
            // be byte-identical), but a *negative entrainment* would mean handing
            // the load a negative quantity of a named material, which is not a
            // thing. Floored only on the sorted path, where `avail` is otherwise
            // the same expression bit for bit.
            let avail = if self.sorted {
                grid.h[c].max(0.0)
            } else {
                grid.h[c]
            };
            let ent = avail.min(room * sus);
            grid.h[c] -= ent;
            self.dh[c] -= ent;
            room -= ent;
            // **Entrainment is where identity enters the load** (§ 13.6: the
            // `loose→load` removal at the source). What comes off is what is lying
            // here — the record's own near-surface composition — so a reach cutting
            // a sandstone bench hands the flow sand and a stripped column hands it
            // basement debris. The last non-zero share takes the residual, so the
            // split is exact by construction rather than exact-to-an-ulp: the same
            // discipline the MFD face split needs, for the same reason.
            if self.sorted && ent > 0.0 {
                let sb = base;
                if let Some(last) = (0..SPECIES).rev().find(|&k| self.shares[sb + k] > 0.0) {
                    let mut given = 0.0;
                    for k in 0..SPECIES {
                        let sh = self.shares[sb + k];
                        if sh <= 0.0 {
                            continue;
                        }
                        let add = if k == last { ent - given } else { sh * ent };
                        given += add;
                        self.qs_sp[base + k] += add;
                    }
                }
            }
            let mut carried = qin + ent;
            // Then incise bedrock, shielded by remaining cover and scaled by
            // the same susceptibility. Where cover is thin enough for this
            // term to matter at all, the outcropping lithology is what the
            // flow is grinding — and on a stripped column that is basement.
            if room > 0.0 {
                let shield = (-grid.h[c] / cfg.h_star).exp();
                let inc_pot = cfg.k_bedrock * ae * sn * shield * sus;
                let max_inc = (grid.r[c] - floor).max(0.0);
                let inc = inc_pot.min(room).min(max_inc);
                grid.r[c] -= inc;
                carried += inc;
                // Incision detaches material from **below the record**, and the
                // composition seam answers that question too — an empty section is
                // whatever lies beneath the pile (`bedrock_sp`). Nothing is named
                // here; on today's world the answer comes back as the hardest,
                // coarsest thing there is, which is why a headwater reach cutting
                // rock rather than reworking cover puts gravel into the load.
                if self.sorted && inc > 0.0 {
                    let mut given = 0.0;
                    let last = self.last_bedrock_species;
                    for k in 0..SPECIES {
                        let sh = self.bedrock_sp[k];
                        if sh <= 0.0 {
                            continue;
                        }
                        let add = if k == last { inc - given } else { sh * inc };
                        given += add;
                        self.qs_sp[base + k] += add;
                    }
                }
            }
            if self.sorted {
                self.qs_sp[base..base + SPECIES].iter().sum()
            } else {
                carried
            }
        } else {
            // Over capacity: deposit the excess as alluvium.
            let dep = qin - cap;
            if self.sorted {
                // **Coarsest first.** The load is drawn down in descending settling
                // velocity, so the excess the flow cannot hold is paid for out of
                // the heaviest fraction it is carrying — a bar of gravel and sand,
                // not an average of everything in the water. The fines that survive
                // this cell are what reaches the next one, and *that* is the
                // downstream gradient: a distal cell deposits mud because there is
                // nothing else left, not because it is a low-energy place.
                let mut remaining = dep;
                let mut placed = 0.0;
                for &k in &self.ws_order {
                    if remaining <= 0.0 {
                        break;
                    }
                    let have = self.qs_sp[base + k];
                    if have <= 0.0 {
                        continue;
                    }
                    let take = if have <= remaining {
                        self.qs_sp[base + k] = 0.0;
                        have
                    } else {
                        self.qs_sp[base + k] = have - remaining;
                        remaining
                    };
                    self.dep_sp[base + k] += take;
                    placed += take;
                    remaining -= take;
                }
                grid.h[c] += placed;
                self.dh[c] += placed;
                self.qs_sp[base..base + SPECIES].iter().sum()
            } else {
                grid.h[c] += dep;
                self.dh[c] += dep;
                cap
            }
        }
    }

    /// Stream-power transport with cover shielding and explicit flux routing.
    /// Suspended load flows down the receiver chain (`qs[rc] += qs_out`), so a
    /// cell needs its full upstream load before it runs — a serial chain, kept
    /// scalar for byte-identity. Zeroes `dh` (the recorder's net-ΔH scratch).
    ///
    /// ## The two invariants MFD had to re-derive
    ///
    /// **Mass down a DAG.** The single-receiver chain conserved mass because each
    /// cell's `qs_out` had exactly one destination. Under MFD it has several, and
    /// the budget survives on one condition: **the shares sum to the whole, with no
    /// residue.** Normalised `f64` weights do not sum to `1` to the bit, so the
    /// last weighted direction takes `qs_out − Σ(earlier shares)` rather than
    /// `w · qs_out`. That makes the split *exact* by construction rather than
    /// exact-to-an-ulp per hop, and a per-hop ulp compounds down a thousand-cell
    /// chain. Everything above the split — entrainment, incision, deposition — is
    /// untouched, which is why one `exchange_cell` serves both paths.
    ///
    /// **The never-incise-below-the-receiver clamp** becomes *below the **lowest**
    /// receiver*. The clamp exists so no runaway knickpoint digs a hole its own
    /// outlet cannot drain — and with several outlets, the cell still drains as
    /// long as it stays above the lowest of them. Cutting to the *highest* would be
    /// arbitrarily stricter; cutting past the lowest makes the cell a pit. In the
    /// single-receiver limit the D8 receiver **is** the lowest neighbour, so the
    /// generalisation reduces to today's rule exactly rather than approximately.
    ///
    /// **The energy slope becomes the share-weighted mean** `Σ w_k · S_k`. Stream
    /// power is `Q·S`; split the discharge and the total power released is
    /// `Σ Q_k·S_k = Q·Σ w_k S_k`. So the weighted mean is not a smoothing choice —
    /// it is the slope that keeps the cell's energy budget equal to the sum of the
    /// budgets of the flows leaving it.
    pub fn transport(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig) {
        self.dh.iter_mut().for_each(|d| *d = 0.0);
        self.qs.iter_mut().for_each(|q| *q = 0.0);
        self.energy.iter_mut().for_each(|e| *e = 0.0);
        // Flow-record side-buffers (empty ⇒ inert). Written, never read, by the
        // transport chain — they cannot perturb an f64 anywhere.
        self.out_load.iter_mut().for_each(|q| *q = 0.0);
        self.out_face_load.iter_mut().for_each(|q| *q = 0.0);
        self.qs_sp.iter_mut().for_each(|q| *q = 0.0);
        self.dep_sp.iter_mut().for_each(|q| *q = 0.0);
        let record = !self.out_face_load.is_empty();
        let sorted = self.sorted;
        if self.mfd.is_none() {
            for k in (0..self.order.len()).rev() {
                let c = self.order[k] as usize;
                let rc = self.recv[c];
                let base = c * SPECIES;
                let qin = if sorted {
                    self.qs_sp[base..base + SPECIES].iter().sum()
                } else {
                    self.qs[c]
                };
                if rc < 0 {
                    // Sink: everything suspended settles here (marine / border).
                    grid.h[c] += qin;
                    self.dh[c] += qin;
                    self.energy[c] = 0.0;
                    if sorted {
                        for s in 0..SPECIES {
                            self.dep_sp[base + s] += self.qs_sp[base + s];
                            self.qs_sp[base + s] = 0.0;
                        }
                    }
                    continue;
                }
                let rc = rc as usize;
                let s = ((self.filled[c] - self.filled[rc]).max(0.0)) / self.cell_m;
                let floor = grid.surf_at(rc);
                let qs_out = self.exchange_cell(grid, cfg, c, qin, s, floor);
                if sorted {
                    // One receiver: the whole multiset moves, species by species.
                    let rb = rc * SPECIES;
                    for s in 0..SPECIES {
                        self.qs_sp[rb + s] += self.qs_sp[base + s];
                    }
                } else {
                    self.qs[rc] += qs_out;
                }
                // The load crossing cell→receiver this epoch — the flow atom's `L`.
                // Captured here because it is unrecoverable afterwards: `qs[rc]` is a
                // sum over every contributor, with no unique factorization.
                if record {
                    self.out_load[c] = qs_out;
                    let (gx, gy) = coords_of(c, self.w);
                    let (jx, jy) = coords_of(rc, self.w);
                    let d = NEIGH8
                        .iter()
                        .position(|&(dx, dy)| (gx + dx, gy + dy) == (jx, jy))
                        .expect("the receiver is a D8 neighbour");
                    self.out_face_load[c * MFD_DIRS + d] = qs_out as f32;
                }
            }
            return;
        }
        for k in (0..self.order.len()).rev() {
            let c = self.order[k] as usize;
            let base = c * MFD_DIRS;
            let sbase = c * SPECIES;
            let qin = if sorted {
                self.qs_sp[sbase..sbase + SPECIES].iter().sum()
            } else {
                self.qs[c]
            };
            // One sweep over the partition for the three things the exchange needs:
            // is there an outlet at all, what slope does the flow descend, and how
            // low may the bed be cut.
            let mut s_bar = 0.0;
            let mut floor = f64::INFINITY;
            let mut last = usize::MAX;
            for (d, &dist) in MFD_DIST.iter().enumerate() {
                let wt = self.mfd_w[base + d];
                if wt <= 0.0 {
                    continue;
                }
                let j = self.mfd_neighbour(c, d);
                let drop = (self.filled[c] - self.filled[j]).max(0.0);
                s_bar += wt * (drop / (self.cell_m * dist));
                floor = floor.min(grid.surf_at(j));
                last = d;
            }
            if last == usize::MAX {
                // Sink: everything suspended settles here (marine / border).
                grid.h[c] += qin;
                self.dh[c] += qin;
                self.energy[c] = 0.0;
                if sorted {
                    for s in 0..SPECIES {
                        self.dep_sp[sbase + s] += self.qs_sp[sbase + s];
                        self.qs_sp[sbase + s] = 0.0;
                    }
                }
                continue;
            }
            let qs_out = self.exchange_cell(grid, cfg, c, qin, s_bar, floor);
            if record {
                self.out_load[c] = qs_out;
            }
            if !sorted {
                let mut given = 0.0;
                for d in 0..MFD_DIRS {
                    let wt = self.mfd_w[base + d];
                    if wt <= 0.0 {
                        continue;
                    }
                    let share = if d == last {
                        qs_out - given
                    } else {
                        wt * qs_out
                    };
                    given += share;
                    let j = self.mfd_neighbour(c, d);
                    self.qs[j] += share;
                    if record {
                        self.out_face_load[base + d] = share as f32;
                    }
                }
                continue;
            }
            // **The residual trick, PER SPECIES** — re-derived, not ported.
            //
            // journal/0109's scalar rule ("the last weighted direction takes
            // `q − Σ(earlier)`") makes one split exact. The obvious generalisation
            // — split the *total* exactly and then apportion each species by its
            // fraction of the total — is **wrong**, and wrong in the silent
            // direction: it re-derives the per-species amount from a shared
            // quantity, so each species picks up its own rounding against a common
            // denominator and the sum of a species over all its faces no longer
            // equals what the cell held. The leak is per-species, invisible in the
            // total, and unattributable afterwards because `qs_sp[j][s]` is a sum
            // over contributors with no unique factorisation. So each species runs
            // its **own** budget: for species `s`, the last weighted direction
            // takes `q_s − Σ(earlier shares of s)`, and every species is exact on
            // its own terms.
            //
            // The face scalar the flux record stores is then the **sum of the
            // species shares on that face** — not a second split of the total. One
            // arithmetic, so the record and the budget cannot disagree (flow.md
            // § 3), exactly as 0109 required of the scalar version.
            let mut face_total = [0.0f64; MFD_DIRS];
            for s in 0..SPECIES {
                let q_s = self.qs_sp[sbase + s];
                if q_s <= 0.0 {
                    continue;
                }
                let mut given = 0.0;
                for d in 0..MFD_DIRS {
                    let wt = self.mfd_w[base + d];
                    if wt <= 0.0 {
                        continue;
                    }
                    let share = if d == last { q_s - given } else { wt * q_s };
                    given += share;
                    face_total[d] += share;
                    let j = self.mfd_neighbour(c, d);
                    self.qs_sp[j * SPECIES + s] += share;
                }
            }
            if record {
                for d in 0..MFD_DIRS {
                    if self.mfd_w[base + d] > 0.0 {
                        self.out_face_load[base + d] = face_total[d] as f32;
                    }
                }
            }
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
        // **S16: routed through the north-star weathering behavior shape.** Each
        // cell builds a `WeatherCtx` view over the height adapter and runs the
        // `WeatheringPass`; `weather_one_cell` is byte-identical to the old
        // `weather_cell` by construction — same operands, same f64 grouping
        // (`base × ((biotic × weatherability) × frost) × taper`), same
        // `R -= q; H += q; dH += q` transfer (docs/spikes/S16). The rate factors
        // are surfaced by name: `wmult` = biotic, the blended `sus` =
        // weatherability, `frost` = the periglacial agent multiplier.
        if parallel {
            grid.r
                .par_iter_mut()
                .zip(grid.h.par_iter_mut())
                .zip(dh.par_iter_mut())
                .enumerate()
                .for_each(|(i, ((r, h), d))| {
                    weather_behavior::weather_one_cell(
                        r,
                        h,
                        d,
                        sea,
                        weathering,
                        h_star,
                        wmult_at(bio, i),
                        sus_at(sus, i),
                        frost_at(frost, i),
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
                weather_behavior::weather_one_cell(
                    r,
                    h,
                    d,
                    sea,
                    weathering,
                    h_star,
                    wmult_at(bio, i),
                    sus_at(sus, i),
                    frost_at(frost, i),
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

    /// The per-cell **frost (periglacial) weathering multiplier** from the last
    /// step (`≥ 1.0` where the frost agent bit, exactly the identity `1.0`/empty
    /// when the frost agent is off). Read-only; the inventory-weathering pass reads
    /// it as the frost agent's driver (material-behavior.md §4). Mirrors [`area`].
    pub fn frost(&self) -> &[f64] {
        &self.frost
    }

    /// The single receiver of each cell as of the last routing (`-1` = sink) — the
    /// exported final drainage network (§ 7.3). This is the last iteration's
    /// routing exactly (no recompute).
    ///
    /// **Under MFD this is no longer the routing** (flow.md § 2.6). With MFD off it
    /// is the D8 steepest-descent receiver and the solve genuinely sends the cell's
    /// whole discharge there. With MFD on the solve sends the discharge to *several*
    /// neighbours, and this plane holds the **argmax share** — a projection of the
    /// partition, retained only because two consumers still want one arrow per cell
    /// (the biotic layer's valley test and the `DeepField` drainage export).
    ///
    /// That is a change of *kind*, and it is worth stating for FLOW continuation
    /// (e), the retirement slice: before MFD, `recv` was an authority the record
    /// shadowed; after MFD it is a **summary of an authority** in the exact sense
    /// ARCHITECTURE.md warns about — it would not exist in this shape if those two
    /// consumers vanished. It is not *meaningless* (the argmax of a partition is a
    /// well-defined thing and still answers "which way does most of the water go"),
    /// but it can no longer be read as "where the water went", and every new
    /// consumer should read the flux record instead.
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
        let dep = &self.dep_sp;
        let sorted = self.sorted;
        let species_at = |i: usize, tag: DepTag| {
            let t = lithology::litho_of_tag(tag);
            if sorted {
                arriving_species(dh[i], &dep[i * SPECIES..(i + 1) * SPECIES], t)
            } else {
                t
            }
        };
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(s, dh[i], tag, chapter, species_at(i, tag));
            });
        } else {
            for i in 0..self.n {
                let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea);
                record_cell(&mut grid.strata[i], dh[i], tag, chapter, species_at(i, tag));
            }
        }
    }

    // ---- phase 10: eolian transport (SCALAR — per-row wind march) ---------

    /// **Wind deflation and downwind loess/dune deposition** (the eolian agent,
    /// [`Agent::Eolian`], journal/0034). A 1D march along the **prevailing-wind**
    /// direction the climate already uses ([`climate::zonal_wind`] — never a
    /// second wind): dry, unvegetated, subaerial cells hand loose cover to an
    /// airborne load; vegetated or humid downwind cells trap it. In the arid
    /// source zone the trapped sand records as a **dune field** ([`Eolian::Dune`],
    /// coarse); on the damp margin the fine silt records as a **loess** sheet
    /// ([`Eolian::Loess`], fine). At 460 m the unit is the *region*, not the
    /// individual dune (earth-processes.md § 4).
    ///
    /// **Smooth zonal profile** (journal/0037): the march direction is the sign
    /// of `zonal_wind`, and the deflation rate is scaled by its **magnitude**
    /// `|zonal_wind| ∈ [0, 1]`. So in a calm belt (the horse latitudes at 30°,
    /// the polar front at 60°) the wind neither picks up nor carries — dune fields
    /// fade to nothing there rather than reversing direction at full strength
    /// across a single grid row (the analytic-boundary "scream" the old three-way
    /// `wind_dx` bit produced).
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
        let providers = cfg.providers;
        let (w, thr, sea) = (self.w, cfg.eolian_arid_precip, self.sea_level);
        let (defl, dep_frac) = (cfg.eolian_deflation, cfg.eolian_deposit_frac);
        for gy in 0..w {
            // Direction and strength both come from the shared smooth profile: the
            // sign steps the row, the magnitude (0..1) scales deflation so calm
            // belts do no eolian work (journal/0037).
            let wind = climate::zonal_wind(grid.lat_deg(gy));
            let dir = if wind > 0.0 { 1 } else { -1 };
            let wind_mag = wind.abs();
            let mut load = 0.0f64;
            let mut last_land: Option<usize> = None;
            for s in 0..w {
                let gx = if dir > 0 { s } else { w - 1 - s };
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
                let shares = providers
                    .outcrop_shares(grid.strata.get(i).map_or(&[][..], |s| s.units.as_slice()));
                let sus = lithology::blend_susceptibility(&shares, &sus_tab);
                let avail = grid.h[i].max(0.0);
                let pickup = (defl * sus * arid * (1.0 - veg) * wind_mag).clamp(0.0, avail);
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
        let (base_rate, band) = (cfg.wave_erosion, cfg.wave_band_m);
        // The configured rate is still the *off switch* — a `0.0` global rate
        // means "no littoral term at all", provider or no provider, which is the
        // byte-identity escape `tests/full_agents.rs` already leans on. The
        // provider is consulted only for cells that survive this gate.
        if base_rate <= 0.0 || band <= 0.0 {
            return;
        }
        // The wave-energy seam (providers.rs § `wave_energy`): identity = the
        // configured global rate; heir = fetch (S11 body graph) × zonal wind.
        let providers = cfg.providers;
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
            let shares = providers
                .outcrop_shares(grid.strata.get(i).map_or(&[][..], |s| s.units.as_slice()));
            let taper = (1.0 - free / band).clamp(0.0, 1.0);
            let rate = providers.wave_energy(WaveCell {
                index: i,
                gx: gx as usize,
                gy: gy as usize,
                base_rate,
            });
            let cut = (rate * lithology::blend_susceptibility(&shares, &sus_tab) * taper).min(free);
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

#[cfg(test)]
mod mfd_tests {
    use super::*;

    /// A 3×3 patch centred on cell 4, with the given drop (metres of filled
    /// potential) to each of the eight neighbours in [`NEIGH8`] order. Everything
    /// is far above the sea stand and the centre is interior.
    fn patch(drops: [f64; 8]) -> (usize, Vec<f64>, Vec<f64>) {
        let w = 3usize;
        let c = 4usize;
        let mut filled = vec![0.0; w * w];
        filled[c] = 100.0;
        for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
            let j = ((1 + dy) as usize) * w + ((1 + dx) as usize);
            filled[j] = 100.0 - drops[d];
        }
        let surf = filled.clone();
        (c, surf, filled)
    }

    /// The partition is a **probability**: one cell's shares sum to one. If they
    /// did not, discharge would be created or destroyed at every junction and the
    /// mass budget flow.md § 3 rests on would mean nothing.
    #[test]
    fn the_partition_weights_sum_to_one() {
        for p in [1.0f64, 1.1, 2.0, 4.0, 6.0] {
            let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.9, 0.25, 1.5, 4.0]);
            let mut w_out = [0.0f64; MFD_DIRS];
            let int_p = (p == p.round()).then_some(p as i32);
            let best = partition_cell(c, 3, &surf, &filled, -1000.0, p, int_p, &mut w_out);
            assert!(
                best >= 0,
                "p={p}: a cell with downslope neighbours is a sink"
            );
            let sum: f64 = w_out.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-12,
                "p={p}: weights sum to {sum}, not 1"
            );
            assert!(w_out.iter().all(|&v| v >= 0.0));
        }
    }

    /// **`p → ∞` is single-receiver D8.** That property is what makes the exponent
    /// a *convergence knob* rather than a different model: the partition contains
    /// the thing it replaces as a limit, so "how much does MFD change the world"
    /// has a continuous answer instead of a discrete one.
    ///
    /// Note what "large" has to mean, because it is the honest reading of the knob:
    /// `p` acts on the **ratio** of slopes, so two neighbours within a few percent
    /// of each other still share at `p = 16`. Collapse is a limit, not a threshold
    /// — which is precisely why `p = 4` leaves gorges convergent (their sidewalls
    /// are nowhere near the channel's slope) and delta tops divergent (theirs are).
    #[test]
    fn a_large_exponent_collapses_the_partition_onto_one_receiver() {
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.1, 0.25, 1.5, 2.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(c, 3, &surf, &filled, -1000.0, 16.0, Some(16), &mut w_out);
        assert!(best >= 0);
        assert_eq!(
            w_out.iter().filter(|&&v| v > 0.0).count(),
            1,
            "weights {w_out:?} did not collapse onto one receiver"
        );
        assert!((w_out[best as usize] - 1.0).abs() < 1e-12);
        assert_eq!(best, 3);
    }

    /// **The partition follows SLOPE, not drop.** Index 7 is a diagonal with a 4 m
    /// drop over `√2` cells (slope 2.83); index 3 a cardinal with 3 m over one cell
    /// (slope 3.0). The cardinal must take the larger share — a steepest-*drop*
    /// rule, which is what `route_cell` uses, picks the diagonal instead. Without
    /// the true flow-path length every diagonal is over-weighted by `√2` and the
    /// drainage net acquires a systematic X-bias.
    #[test]
    fn the_partition_follows_slope_not_drop() {
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 3.0, 0.1, 0.25, 1.5, 4.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(c, 3, &surf, &filled, -1000.0, 4.0, Some(4), &mut w_out);
        assert_eq!(best, 3, "the diagonal's √2 path length was not applied");
        assert!(w_out[3] > w_out[7]);
        // The steepest-DROP rule would have said otherwise — pinned, so the
        // difference between the two routings is a fact this file states rather
        // than a claim the prose makes.
        assert_eq!(route_cell(c, 3, &surf, &filled, -1000.0), 8);
    }

    /// **`p = 1` is maximally dispersive** (Quinn) and must genuinely spread:
    /// every downslope neighbour above the representational floor keeps a share.
    /// With equal drops all round, the cardinals win twice over — a shorter path
    /// (steeper slope) *and* a wider contour — so their share is exactly `2×` a
    /// diagonal's. That is the two `√2`s the partition carries, isolated.
    #[test]
    fn a_unit_exponent_spreads_across_every_downslope_neighbour() {
        let (c, surf, filled) = patch([1.0; 8]);
        let mut w_out = [0.0f64; MFD_DIRS];
        partition_cell(c, 3, &surf, &filled, -1000.0, 1.0, Some(1), &mut w_out);
        assert_eq!(w_out.iter().filter(|&&v| v > 0.0).count(), 8);
        let ratio = w_out[1] / w_out[0];
        assert!(
            (ratio - 2.0).abs() < 1e-9,
            "cardinal/diagonal share ratio is {ratio}, expected 2 (√2 slope × √2 contour)"
        );
    }

    /// A cell with no downslope neighbour is a **sink** and every weight is zero —
    /// the record then reads it as a boundary-face exit, the same convention the
    /// single-receiver path used.
    #[test]
    fn a_cell_with_no_lower_neighbour_is_a_sink() {
        let (c, surf, filled) = patch([-1.0; 8]);
        let mut w_out = [1.0f64; MFD_DIRS];
        let best = partition_cell(c, 3, &surf, &filled, -1000.0, 4.0, Some(4), &mut w_out);
        assert_eq!(best, -1);
        assert!(w_out.iter().all(|&v| v == 0.0));
    }

    /// The **representational floor** drops a share below [`MFD_MIN_WEIGHT`] and
    /// renormalises the survivors, so the sum is still exactly one. Without the
    /// renormalisation the floor would quietly delete discharge — a leak that would
    /// show up in the mass budget as a mystery rather than as a rule.
    #[test]
    fn the_weight_floor_renormalises_rather_than_deleting_discharge() {
        // One dominant cardinal and one very gentle one: at p = 4 the gentle
        // neighbour's share falls under a percent and is dropped.
        let (c, surf, filled) = patch([-1.0, -1.0, -1.0, 10.0, 0.3, -1.0, -1.0, -1.0]);
        let mut w_out = [0.0f64; MFD_DIRS];
        partition_cell(c, 3, &surf, &filled, -1000.0, 4.0, Some(4), &mut w_out);
        let sum: f64 = w_out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "sum {sum} after the floor");
        assert!(
            w_out.iter().all(|&v| v == 0.0 || v >= MFD_MIN_WEIGHT),
            "a sub-floor weight survived: {w_out:?}"
        );
    }
}
