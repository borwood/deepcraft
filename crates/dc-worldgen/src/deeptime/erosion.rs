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
use super::recorder::{
    Aridity, DeepStrata, DepEnv, DepTag, EnergyBand, Eolian, MemberCtx, dep_tags,
};
use super::weather_behavior;
use dc_core::materials::MaterialId;

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
///
/// **2026-07-26 (hybrid `p`, journal/0113):** the floor became a *knob*
/// ([`MfdParams::min_weight`], [`DeepConfig::mfd_min_weight`]) rather than a
/// hard-coded constant, so its effect on the solve can be **measured end-to-end**
/// instead of argued. The default is unchanged and the stub is unchanged — a knob
/// is not an heir.
pub const MFD_MIN_WEIGHT: f64 = 0.01;

/// **The hybrid-`p` law — a spatially varying convergence exponent**
/// (FLOW continuation (b'), journal/0113, `docs/design/flow.md` § 2.6.2).
///
/// journal/0109 shipped **one** exponent for the whole world, and that is
/// physically wrong in a specific way: it applies **hillslope sheet-flow behaviour
/// inside channels.** Real water spreads where it is unchannelised and stays in
/// its banks once it is not, and a uniform `p` cannot say both. The measured cost
/// was the peak catchment collapsing **1,245 → 84 cells**: dispersing at *every*
/// cell compounds down the chain, so a trunk river never accumulates.
///
/// **The discriminator is channelisation, and the standard index for it is
/// Montgomery & Dietrich (1988, 1992)'s `χ = A · S²`** — the drainage-area × slope
/// product whose exceedance marks a channel head on a real landscape. `A` is the
/// cell's drainage area in cells (**lagged one epoch** — see [`Erosion::route`])
/// and `S` the steepest downslope gradient on the free-surface potential. The
/// exponent ramps log-linearly in `χ` from [`Self::p_hill`] to [`Self::p_chan`]
/// between [`Self::chi_lo`] and [`Self::chi_hi`].
///
/// **Why `A·S²` and not `A` alone, which is the obvious choice.** An area-only law
/// destroys what the slice before this one bought. Deltas, alluvial-fan tops and
/// braid plains are exactly the places with the *largest* `A`, so an area-only law
/// would make them the most convergent ground on the world and concurrent
/// distributaries would vanish. `A·S²` puts them back on the dispersive side for
/// the physically correct reason: **a delta is where a channel loses its
/// confinement.** Three regimes fall out of one law:
///
/// | regime | `A` | `S` | `χ` | `p` | behaviour |
/// |---|---|---|---|---|---|
/// | hillslope / interfluve | small | any | low | `p_hill` | sheet flow, spreads |
/// | trunk river, gorge, incised valley | large | moderate–high | high | `p_chan` | stays in its banks |
/// | delta top, fan, coastal plain | large | ≈0 | low | `p_hill` | splits — distributaries |
///
/// **Honest limit at this tier.** At 460 m a cell contains an entire
/// hillslope-and-channel system, so this is not "is this cell a channel" but a
/// **sub-grid parameterisation of how much of the cell's discharge is confined**.
/// The thresholds are therefore calibrated against *this world's* own `χ`
/// distribution (`examples/hybrid_p_probe.rs` prints it), never lifted from a
/// field study at 10 m.
///
/// **⚠ STUB #26 — [`Self::chi_lo`] / [`Self::chi_hi`] are fitted to ONE world.**
/// The index is cited and general; the two thresholds are a stand-in until either
/// the joint supply+transport calibration gives `χ` a physical scale (stub #24) or
/// the index is re-expressed dimensionlessly. See `docs/design/stubs.md` § 26.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MfdParams {
    /// The convergence exponent where flow is **unchannelised**. Quinn (1991) /
    /// Freeman (1991)'s dispersive limit is `1.0`.
    ///
    /// Holmgren's calibrated `4–6` band — journal/0109's uniform default — is the
    /// *compromise* a single-exponent scheme is forced into, because one number
    /// has to serve hillslope and channel alike. Once `p` varies, the endpoints
    /// should be the endpoints and not the compromise.
    pub p_hill: f64,
    /// The convergence exponent at the top of the ramp — the last value before
    /// the cell is treated as fully channelised. At `p = 16` a neighbour at 90 %
    /// of the maximum slope keeps 19 % of the steepest direction's weight and one
    /// at 70 % keeps 0.3 %, which the representational floor then drops.
    ///
    /// `p_chan == p_hill` is **uniform `p`** — journal/0109's solve, recovered
    /// exactly, with no ramp and no switch evaluated.
    pub p_chan: f64,
    /// `χ` at or below which the exponent is [`Self::p_hill`].
    pub chi_lo: f64,
    /// **`χ` at or above which the cell routes SINGLE-RECEIVER, exactly** — the
    /// hard switch, and it is a measured necessity rather than a stylistic choice.
    ///
    /// A smooth exponent cannot make a channel confined on terrain this smooth.
    /// The partition's weights go as `(Sₖ/S_max)^p`, so a neighbour at 90 % of the
    /// steepest slope still keeps 19 % at `p = 16` and 8 % at `p = 24`; suppressing
    /// it below the representational floor needs `p > 44`, and a neighbour at 95 %
    /// needs `p > 90`. **Measured on the shipped world: `p = 16` everywhere lifts
    /// the peak catchment only 84 → 145 cells against D8's 1,175** — the trunk
    /// still bleeds a fifth of its discharge at every hop, and a fifth per hop
    /// down a fifty-hop chain is everything.
    ///
    /// So above `chi_hi` the solve routes the whole discharge down the steepest
    /// slope: **once flow is channelised it is confined, and confined flow takes
    /// one path.** The ramp below it is what keeps the transition continuous, so
    /// the switch fires at `p = p_chan` rather than out of a dispersive state.
    pub chi_hi: f64,
    /// [`MFD_MIN_WEIGHT`] as a knob — the representational floor on a share.
    pub min_weight: f64,
}

impl MfdParams {
    /// journal/0109's solve: one exponent everywhere. A named constructor because
    /// it is the control every hybrid measurement is quoted against — and because
    /// it is a **pinned identity path**: `partition_cell` skips the `S/S_max`
    /// normalisation for a uniform law so the arithmetic is bit-for-bit 0109's,
    /// which is what keeps the scalar-load and anonymous-creep goldens reachable.
    ///
    /// The price of that pin is that a uniform law inherits 0109's exponent-range
    /// limit: raw `S^p` on the gentlest gradients this world carries underflows
    /// somewhere past `p ≈ 40`, and a partition that underflows to zero reports a
    /// draining cell as a sink. Uniform `p` is a *control*, not a shipping mode;
    /// the ramp is where large exponents live and it is normalised.
    #[must_use]
    pub fn uniform(p: f64) -> Self {
        Self {
            p_hill: p,
            p_chan: p,
            chi_lo: 1.0,
            chi_hi: 1.0,
            min_weight: MFD_MIN_WEIGHT,
        }
    }

    /// Is this the uniform-`p` solve (no ramp)?
    #[inline]
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        self.p_hill == self.p_chan
    }

    /// Is the flow at channelisation index `chi` confined — i.e. does this cell
    /// route **single-receiver**? Always false for a uniform law.
    #[inline]
    #[must_use]
    pub fn is_channel(&self, chi: f64) -> bool {
        !self.is_uniform() && chi >= self.chi_hi
    }

    /// **The exponent at channelisation index `chi`.** `p_hill` at or below
    /// `chi_lo`, `p_chan` at or above `chi_hi`, log-linear between — monotone
    /// non-decreasing in `chi` by construction, which is the invariant the gate
    /// pins (a law wired backwards would disperse channels and concentrate
    /// hillslopes, and no absolute count could see it). Above `chi_hi` the
    /// exponent stops being consulted at all: see [`Self::is_channel`].
    ///
    /// The ramp is **rounded to an integer**, and that is a cost decision stated
    /// rather than hidden: a fractional exponent forces `powf` on
    /// `8 × cells × epochs` ≈ half a billion directions per production run, where
    /// an integer goes through `powi` — a handful of multiplies. At a 460 m tier
    /// where the exponent is a coarse sub-grid dial, the difference between
    /// `p = 7` and `p = 7.3` is false precision; the difference in gen time is
    /// not. Uniform mode does **not** round, so a probe may still sweep `p = 1.5`.
    #[inline]
    #[must_use]
    pub fn exponent_at(&self, chi: f64) -> f64 {
        if self.is_uniform() || chi <= self.chi_lo {
            return self.p_hill;
        }
        if chi >= self.chi_hi {
            return self.p_chan;
        }
        let t = (chi / self.chi_lo).ln() / (self.chi_hi / self.chi_lo).ln();
        (self.p_hill + (self.p_chan - self.p_hill) * t).round()
    }
}

impl Default for MfdParams {
    /// The shipped hybrid law — see [`DeepConfig::mfd_exponent_channel`] and
    /// siblings for where each number comes from. `chi_lo`/`chi_hi` are calibrated
    /// against the shipped world's own `χ` distribution (journal/0113).
    fn default() -> Self {
        Self {
            p_hill: 1.0,
            p_chan: 16.0,
            chi_lo: 3.0e-2,
            chi_hi: 1.2e-1,
            min_weight: MFD_MIN_WEIGHT,
        }
    }
}

/// The number of **species** the suspended load is resolved into — one per
/// [`Litho`], which is the material granularity deep time can distinguish at all
/// (`lithology.rs`: *"not the material registry — the handful of classes the
/// deep-time record can distinguish"*). Members within a class are chosen at
/// collapse time, so resolving the load any finer than this would be inventing
/// identity the tier does not have.
const SPECIES: usize = Litho::COUNT;

/// **The competence ceiling, per unit of transport capacity** — the one
/// calibration constant material-aware transport adds, in
/// `settle_energy`-units per (metre/iteration) of stream capacity, **stated
/// relative to `k_transport`**.
///
/// `material-behavior.md` § 13.5: *"sorting is the falling ceiling; we write the
/// ceiling, not the sort."* This is that ceiling. A flow of capacity `cap` can
/// hold in suspension every species whose settling velocity is at most
/// `COMPETENCE_PER_KT · k_transport · cap`; everything heavier rains out
/// **wherever it is**, regardless of whether the flow still has capacity to spare.
/// Capacity is the *total mass* limit and competence is the *size/density* limit,
/// and § 13.5 is explicit that both are needed: without competence a flow with
/// spare capacity carries boulders to the sea, and nothing ever fines downstream.
///
/// **Where the number comes from — and see corrections #59 for what was wrong with
/// the way that used to be said.** The *ratio* is fixed by an anchor that already
/// ships; the old doc comment stated that anchor as an **absolute capacity**, which
/// silently held `k_transport = 0.0016` inside it and called itself "not a tuning
/// knob" on the strength of a derivation from a constant that was itself about to
/// be recalibrated. [`energy_band`] calls a capacity of
/// `ENERGY_LOW_MED · k_transport / REFERENCE_KT` the Low/Medium boundary, and
/// `litho_of_tag` turns exactly that boundary into the coarse/fine clastic split —
/// so that boundary is *already* the world's stated "energy at which sand stops
/// moving". `settle_energy` puts the coarse-clastic reference sheet at `≈0.84`, and
/// at the historical `k_transport = 0.0016` the boundary is `0.002`, giving
/// `0.84 / 0.002 = 420`. The ceiling therefore crosses the coarse-clastic threshold
/// at precisely the capacity the shipped facies rule already crosses it at, and the
/// rest of the roster arranges itself around that.
///
/// It is **linear** in capacity because capacity is already a stream-power proxy
/// (`k·A^m·S^n`) and competence in a real channel scales with a power of stream
/// power; linear is the simplest form that spans the roster, in the same
/// plausible-not-tuned register as S9's physics constants. Chosen and written
/// **before** the outcome probe was run, and not revisited after.
///
/// **RE-EXPRESSED relative to `k_transport` 2026-07-26 (journal/0114), and the
/// value it produces at the historical `k_transport` is unchanged to the bit.**
/// `420` was `0.84 / 0.002`, and `0.002` was a boundary written in absolute
/// capacity because `k_transport` had never moved. The joint calibration moves it,
/// and left as an absolute this constant would have made every calibrated flow
/// competent to carry basement — sorting would vanish, "does sand move" would read
/// yes for a reason that is an artifact, and the facies gradient would be destroyed
/// rather than measured. The relative form says what was always meant: the ceiling
/// is a statement about **where in a drainage network you are**, not about the
/// value of a rate constant.
const COMPETENCE_SCALE: f64 = 420.0;

/// **The `k_transport` every threshold in this section was written against.**
///
/// `COMPETENCE_SCALE`, [`ENERGY_LOW_MED`] and [`ENERGY_MED_HIGH`] are numbers in
/// units of `k_transport · A^m · S^n`, and they were chosen when `k_transport` was
/// `0.0016` and had never moved. Naming that value is what lets the calibration
/// scale the coefficient and carry the thresholds along instead of silently
/// re-labelling the world.
///
/// **Every ratio below is formed as `k / REFERENCE_KT`, deliberately**: `x / x` is
/// exactly `1.0` in IEEE-754, so at the historical coefficient each threshold is
/// bit-identical to the absolute constant it replaced, and the pre-calibration
/// world reproduces its goldens to the bit.
const REFERENCE_KT: f64 = 0.0016;

/// **The Low/Medium energy boundary** at [`REFERENCE_KT`] — the capacity at which
/// the shipped facies rule says sand stops moving. Scaled with `k_transport` by
/// [`energy_band`]; see [`REFERENCE_KT`].
const ENERGY_LOW_MED: f64 = 0.002;
/// **The Medium/High energy boundary** at [`REFERENCE_KT`] — the trunk threshold.
/// See [`ENERGY_LOW_MED`].
const ENERGY_MED_HIGH: f64 = 0.02;

/// **The competence ceiling of a flow with transport capacity `cap`**, in a world
/// whose stream-transport coefficient is `k_transport` — the largest settling
/// velocity ([`lithology::settling_table`]) the flow can hold in suspension. See
/// [`COMPETENCE_SCALE`] for where the constant comes from and [`REFERENCE_KT`] for
/// why the second argument exists.
///
/// Public because the invariant *"nothing leaves a cell that the cell could not
/// carry"* is checked against it from outside, and a test that re-derived the
/// ceiling would be checking its own arithmetic rather than the pass's.
#[inline]
pub fn competence_ceiling(cap: f64, k_transport: f64) -> f64 {
    COMPETENCE_SCALE * (REFERENCE_KT / k_transport) * cap
}

/// **Split a bulk quantity into species by a composition, exactly** — the one
/// place a metre of *something* becomes metres of *named things* in this pass.
///
/// `shares` is a per-[`Litho`] composition (an [`outcrop_shares`] read, or the
/// bedrock composition below the record). Normalised `f64` shares do not sum to
/// `1` to the bit, so a plain `share × total` per species leaves a residue and the
/// itemisation stops equalling its own total. **The last non-zero share takes
/// `total − Σ(earlier)`** — journal/0109's residual rule, on the species axis.
///
/// **This is the anti-leak rule journal/0110 had to state twice, and it is why
/// there is one function rather than three.** The tempting shortcut — split the
/// total exactly *somewhere else*, then apportion species by fraction here —
/// makes every species round against a shared denominator: the total stays
/// perfect and each species drifts, invisibly and unattributably. Entrainment,
/// incision and hillslope creep all route through this, so no caller can invent
/// its own budget.
///
/// An all-zero composition returns all zeros: the caller moved bulk it has no
/// identity for, and fabricating one would be worse than recording none.
#[inline]
fn split_by_shares(total: f64, shares: &[f64]) -> [f64; SPECIES] {
    let mut out = [0.0; SPECIES];
    let Some(last) = (0..SPECIES).rev().find(|&k| shares[k] > 0.0) else {
        return out;
    };
    let mut given = 0.0;
    for k in 0..SPECIES {
        let sh = shares[k];
        if sh <= 0.0 {
            continue;
        }
        let v = if k == last { total - given } else { sh * total };
        given += v;
        out[k] = v;
    }
    out
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
/// form, large `p` is single-receiver steepest-slope. Nothing here is novel; what
/// it buys over D8 is that a cell's discharge can leave through more than one face
/// **within one epoch**, which is the entire difference between temporal
/// divergence (avulsion, which the record already had) and simultaneous divergence
/// (concurrent distributaries, which it structurally could not hold).
///
/// **`p` is SPATIALLY VARYING** (journal/0113). `area_i` is the cell's drainage
/// area from the *previous* epoch; the exponent is [`MfdParams::exponent_at`] of
/// the channelisation index `χ = area_i · S_max²`, so unchannelised ground
/// disperses and channelised ground stays in its banks. `MfdParams::uniform`
/// recovers journal/0109's single exponent with no ramp evaluated.
///
/// **The slopes are normalised by `S_max` before exponentiation.** Algebraically
/// that is a no-op — the renormalisation at the end divides it straight back out —
/// but it makes every base lie in `(0, 1]`, so a large `p_chan` can never underflow
/// a whole partition to zero and turn a draining cell into a sink. Without it the
/// safe exponent range is bounded by the world's smallest slope, which is a
/// coupling nobody would remember.
#[inline]
#[allow(clippy::too_many_arguments)]
fn partition_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    filled: &[f64],
    sea_level: f64,
    area_i: f64,
    cell_m: f64,
    mp: &MfdParams,
    w_out: &mut [f64],
) -> i32 {
    w_out.iter_mut().for_each(|v| *v = 0.0);
    if surf[i] <= sea_level || is_border(i, w) {
        return -1;
    }
    let (gx, gy) = coords_of(i, w);
    let fi = filled[i];
    // Pass 1: the downslope gradients, and the steepest of them — which is both
    // the normaliser and the `S` of the channelisation index. `steepest` is the
    // direction that takes the whole discharge when the cell is channelised.
    let mut slope = [0.0f64; MFD_DIRS];
    let mut s_max = 0.0f64;
    let mut steepest = -1i32;
    for (d, (dx, dy)) in NEIGH8.into_iter().enumerate() {
        let Some(j) = in_grid(gx + dx, gy + dy, w) else {
            continue;
        };
        let drop = fi - filled[j];
        if drop <= 0.0 {
            continue;
        }
        let s = drop / MFD_DIST[d];
        slope[d] = s;
        if s > s_max {
            s_max = s;
            steepest = d as i32;
        }
    }
    if s_max <= 0.0 {
        return -1;
    }
    // The channelisation index. `s_max / cell_m` is the dimensionless gradient —
    // `slope` above is a rise per **cell width**, and leaving the cell size inside
    // `χ` would put a second, invisible resolution factor in a threshold that
    // already carries one through `A` (stub #26 owns what remains).
    let s_dim = s_max / cell_m;
    let chi = area_i * s_dim * s_dim;
    // **The channel switch.** Confined flow takes one path: the whole discharge
    // goes down the steepest slope, exactly. This is a *routing* statement, not a
    // large exponent — see `MfdParams::chi_hi` for why an exponent cannot do it.
    // The mass rules are trivially satisfied (one weighted direction, so it is
    // also the last, and it takes `q − 0`), and the traversal licence holds
    // because the chosen direction has `drop > 0` like every other weighted one.
    if mp.is_channel(chi) {
        w_out[steepest as usize] = 1.0;
        return steepest;
    }
    let p = mp.exponent_at(chi);
    // Integer exponents go through `powi` — the hybrid ramp is rounded so this is
    // the taken branch on every cell of a production run, where `powf` on
    // 8 directions × 297k cells × 200 epochs is half a billion transcendental
    // calls and would dominate the deep run. `powf` survives for the fractional
    // uniform exponents a probe may sweep.
    let pr = p.round();
    let int_p = (p == pr && (1.0..=64.0).contains(&pr)).then_some(pr as i32);
    // **Normalise by `S_max` — on the hybrid path only.** Algebraically the
    // division is a no-op (the renormalisation below divides it straight back
    // out), but it puts every base in `(0, 1]` so the ramp's large exponents
    // cannot underflow a whole partition to zero and report a draining cell as a
    // sink. It is *not* applied to a uniform law, and that is deliberate: `x/1.0`
    // is exact, so journal/0109's arithmetic survives **bit for bit** and the
    // uniform world stays a reachable cross-commit fixed point (the goldens in
    // `material_transport.rs` and `material_creep.rs` are pinned there). The cost
    // of that choice is that a *uniform* law keeps 0109's exponent-range limit —
    // see [`MfdParams::uniform`].
    let norm = if mp.is_uniform() { 1.0 } else { s_max };
    let mut sum = 0.0;
    for (d, &s) in slope.iter().enumerate() {
        if s <= 0.0 {
            continue;
        }
        let base = s / norm;
        let sp = match int_p {
            Some(k) => base.powi(k),
            None => base.powf(p),
        };
        let raw = sp * MFD_CONTOUR[d];
        w_out[d] = raw;
        sum += raw;
    }
    if sum <= 0.0 {
        return -1;
    }
    // Normalise, then apply the representational floor and renormalise over the
    // survivors. The steepest direction's base is exactly `1`, so its raw weight is
    // at least `1/√2` against a total of at most `8`; its share is therefore never
    // below `1/12`, the survivor set is never empty and `sum2 > 0` always.
    let mut sum2 = 0.0;
    for v in w_out.iter_mut() {
        let n = *v / sum;
        *v = if n >= mp.min_weight { n } else { 0.0 };
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

/// **The per-edge coefficient at which an explicit 4-neighbour Laplacian stops
/// oscillating** — the bound the hillslope-transport operator sub-cycles to
/// respect (journal/0122).
///
/// # Where 1/8 comes from — derived, never tuned
///
/// Write one epoch of hillslope diffusion on the frozen surface as
/// `h_i ← h_i + a·Σ_j (s_j − s_i)` over the four cardinal neighbours, with `a`
/// the per-edge coefficient ([`eff_diff`]). The von Neumann amplification factor
/// is `g(k) = 1 − 2a(2 − cos k_x − cos k_y)`, so:
///
/// - `a ≤ 1/4` ⇒ `g ≥ −1`: **stable**, but the grid-scale (Nyquist) mode is
///   reflected with its amplitude intact — `g(π,π) = −1` is a period-2
///   flip-flop that never decays.
/// - `a ≤ 1/8` ⇒ `g ≥ 0`: **monotone**. No mode may change sign, so a
///   checkerboard cannot survive a step, let alone be created by one.
///
/// **What the shipped world actually sits at, since the config rate is not the
/// whole story.** `diffusion = 0.12` is 4 % *inside* this bound — but [`eff_diff`]
/// folds in the lithology's creep susceptibility, and peat is the softest thing in
/// the world, so the shipped grid's **peak effective coefficient is 0.261**: 2.1×
/// past. Under the calibration it is **12.60**, or **100.8× past**. That gap is the
/// whole defect: a 2.1× excursion on a handful of soft cells carrying 4.6 m of cover
/// produces a wobble the surface absorbs (shipped `conc(h)` ACF −0.10, no
/// checkerboard), while a 100.8× excursion everywhere carrying 41 m produces
/// −0.82 and a 40 m grid-scale residual.
///
/// *The two facts journal/0116 could not reconcile — "saturation alone is not
/// sufficient" and "the limiter is deaf to the step" — are the same fact read from
/// either side of this constant.* The coefficient decides that the grid-scale mode
/// flips sign; the limiter decides how far, by capping the export at the cell's
/// inventory, which is what turns a divergence into a finite period-2 limit cycle.
///
/// **This is why journal/0116's 4× time-step refinement read as a null.** It took
/// the coefficient 5.4 → 1.35, which is still **11× past the bound**; the
/// experiment was sound and the refinement was an order of magnitude too small to
/// reach the register it was testing. See `corrections.md` #72.
pub const CREEP_MAX_EDGE_COEFF: f64 = 0.125;

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

/// **Material-aware hillslope creep** (Movement 2b continuation (b),
/// `material-behavior.md` § 13.2 — the **gravity / mass-wasting** member of the
/// transport family). The per-species net thickness change at cell `i`, gathered
/// from exactly the same four edges, with exactly the same fluxes, as
/// [`diffuse_net_cell`].
///
/// **Colluvium is not sorted, and that is the point.** Creep is diffusive and
/// gravity-driven: it has no competence ceiling, no settling draw, no
/// coarsest-first. Every edge moves the **donor's whole composition in
/// proportion** — the near-surface window the `outcrop_shares` seam already reads
/// each epoch, which for a stripped column is honestly the bedrock beneath. So a
/// colluvial apron is *locally derived and poorly sorted*, against a fluvial
/// deposit's *far-travelled and sorted*, and that contrast is a real facies
/// distinction rather than a second copy of the river's rule.
///
/// **Why this cannot leak.** The edge flux is antisymmetric to the bit — cell `i`
/// computes `eff_diff(i)·(sᵢ − sⱼ)·scale[i]` and cell `j` computes
/// `eff_diff(i)·−(sⱼ − sᵢ)·scale[i]`, and IEEE-754 subtraction is exactly
/// antisymmetric — and **both endpoints split it by the same donor composition
/// through the same [`split_by_shares`]**, so what leaves `i` of a species is bit
/// for bit what arrives at `j`. No species is created or destroyed anywhere on the
/// grid, which is the creep analogue of journal/0110's per-species junction test
/// and is asserted as one.
///
/// **This is an attribution, never a mass authority.** The terrain still moves by
/// the scalar [`diffuse_net_cell`], unchanged and byte-identical; this vector only
/// says *what* the metres were made of. That separation is deliberate: identity
/// riding a second arithmetic could not perturb `H` even if it were wrong.
///
/// **Returns the gross traffic** through the cell — the sum of every edge flux, in
/// or out. That is the audit's denominator, and it has to be: a cell that sheds as
/// much as it gains has a net near zero with real material moving through it, and
/// dividing a rounding error by *that* would report a leak where there is only
/// cancellation.
#[expect(
    clippy::too_many_arguments,
    reason = "the diffusion kernel's own arity"
)]
#[inline]
fn diffuse_species_cell(
    i: usize,
    w: usize,
    surf: &[f64],
    scale: &[f64],
    diffusion: f64,
    resist: &[f32],
    sus: &[f64],
    shares: &[f64],
    out: &mut [f64],
    accumulate: bool,
) -> f64 {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut gross = 0.0;
    if !accumulate {
        out.fill(0.0);
    }
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w) {
            let d = si - surf[j];
            if d > 0.0 {
                let f = eff_diff(diffusion, resist, sus, i) * d * scale[i];
                let s = split_by_shares(f, &shares[i * SPECIES..(i + 1) * SPECIES]);
                for k in 0..SPECIES {
                    out[k] -= s[k];
                }
                gross += f;
            } else if d < 0.0 {
                let f = eff_diff(diffusion, resist, sus, j) * (-d) * scale[j];
                let s = split_by_shares(f, &shares[j * SPECIES..(j + 1) * SPECIES]);
                for k in 0..SPECIES {
                    out[k] += s[k];
                }
                gross += f;
            }
        }
    }
    gross
}

/// The number of edges cell `i` **sends** creep across this epoch — the face count
/// a gravity-caused [`super::flux::FluxEntry`] would need if the flow record grew
/// the mass-wasting mover (stubs.md #18's `cause`). Counted rather than recorded,
/// because the count is the cost estimate that decides whether recording it is
/// affordable, and a probe that guessed it would be guessing the answer.
#[inline]
fn diffuse_outflux_faces(i: usize, w: usize, surf: &[f64], scale: &[f64]) -> usize {
    let (gx, gy) = coords_of(i, w);
    let si = surf[i];
    let mut n = 0;
    for (dx, dy) in NEIGH4 {
        if let Some(j) = in_grid(gx + dx, gy + dy, w)
            && si - surf[j] > 0.0
            && scale[i] > 0.0
        {
            n += 1;
        }
    }
    n
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

/// Map a stream transport capacity to a facies energy band, in a world whose
/// stream-transport coefficient is `k_transport`. Thresholds are in the capacity
/// units of the transport pass (metres/iteration); calibrated so headwater
/// hillslopes read Low, trunk rivers read High.
///
/// **The second argument arrived with the erosional calibration (journal/0114) and
/// it is not a convenience.** `cap = k_transport · A^m · S^n`, so a bare capacity
/// is `k_transport` times a position in the drainage network, and the boundaries
/// below describe the *position*. Left absolute, a calibration that raised
/// `k_transport` would have re-labelled essentially every depositional site on the
/// world **High energy** — `litho_of_tag` would then have recorded coarse clastic
/// everywhere, and the facies gradient the Movement 2b probes exist to measure
/// would have been erased by the same commit that was supposed to make the world
/// erode. Scaling with the reference keeps the classification invariant under a
/// pure change of rate, which is what a calibration is.
pub fn energy_band(cap: f64, k_transport: f64) -> EnergyBand {
    let s = k_transport / REFERENCE_KT;
    if cap < ENERGY_LOW_MED * s {
        EnergyBand::Low
    } else if cap < ENERGY_MED_HIGH * s {
        EnergyBand::Medium
    } else {
        EnergyBand::High
    }
}

/// The measured depositional tag for a cell given its final surface, precip, and
/// the transport capacity it saw this iteration.
#[inline]
fn tag_of(surf_i: f64, precip_i: f32, energy_i: f64, sea_level: f64, k_transport: f64) -> DepTag {
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
    DepTag::mineral(env, aridity, energy_band(energy_i, k_transport))
}

/// Apply one cell's net thickness change to its strata record under `tag`,
/// stamped with the current tectonic `chapter` (0 when tectonic history is off).
///
/// **`tag` and `species` are LAZY, and that is a cost decision** (P11 slice 1).
/// Deposition-time member fitness is an inverse-CDF over a class's registered
/// members, and it allocates; evaluating it eagerly as a call argument would run
/// it for **every cell every epoch** — ~297 k × 200 on a production world — when
/// only the depositing minority can use the answer. A cell that eroded or did
/// nothing this epoch never asks which rock arrived, because none did.
#[inline]
fn record_cell(
    s: &mut DeepStrata,
    dh: f64,
    chapter: u8,
    deposit: impl FnOnce() -> (DepTag, MaterialId),
) {
    if dh.abs() < 1e-9 {
        return;
    }
    if dh > 0.0 {
        let (tag, species) = deposit();
        s.deposit_as(tag, dh, chapter, species);
    } else {
        s.erode(-dh);
    }
}

/// **Which material the unit arriving at this cell is made of** (Movement 2b).
///
/// The cell's net gain has several sources, and **two of them now carry an
/// identity**: the fluvial transport pass knows, per species, exactly what it set
/// down (`dep`), and hillslope creep knows, per species, exactly what came down
/// the slope into this cell (`creep`, Movement 2b continuation (b)). What is left
/// in `dh` — bedrock weathered to regolith in place, wind and wave, the biotic
/// layer — never rode any mover, and keeps the answer the record has always
/// given: the tag's own lithology.
///
/// So the unit's **class** is the argmax of the whole mixture: each carried
/// species against the un-carried remainder, with ties going to the incumbent (a
/// strict `>` over fixed index order, so it is deterministic).
///
/// ⚠ **It answers the CLASS, not the rock** (P11 slice 1). The transport budgets
/// are still `Litho::COUNT`-wide, so this argmax is over classes; the *member* is
/// then chosen by fitness at deposition under the cell's own climate
/// ([`super::recorder::MemberCtx::surface`]). Slice 2 re-grades the budgets, at
/// which point the argmax is over materials and this function returns the rock
/// directly.
///
/// **The two movers are summed, not ranked.** A cell that receives half a metre of
/// fine clastic from upstream and half a metre of the same rock off the slope
/// above has a metre of that rock, and pretending the two halves compete would
/// make the answer depend on which agent we asked first.
///
/// **STUB #25 — which *mover* delivered it is a different axis, and the record does
/// not carry one.** Colluvium and alluvium are separable only by signature, not by
/// label; the byte that would fix it costs ~42 MiB at today's `DepUnit` layout, and
/// the free version is a packed `(species, mover)` byte. See `stubs.md` § 25 and
/// `examples/colluvium_probe.rs`, which measures the signature the label is missing.
///
/// Only *gains* are candidates: a species creep took **away** from this cell is
/// not something the cell can be made of, so the negative entries are clamped out
/// of both the mixture and the remainder.
///
/// This is deliberately *not* a threshold on "was most of this transported" —
/// a threshold would be a second rule with a number in it. It is one comparison
/// over one mixture, and it degenerates exactly to the old behaviour when nothing
/// was carried here.
#[inline]
fn arriving_species(dh: f64, dep: &[f64], creep: &[f64], tag_species: Litho) -> Litho {
    let mut mix = [0.0; SPECIES];
    let mut carried = 0.0;
    for k in 0..SPECIES {
        let m = dep[k] + creep.get(k).copied().unwrap_or(0.0).max(0.0);
        mix[k] = m;
        carried += m;
    }
    let mut best = tag_species;
    let mut best_m = dh - carried;
    let mut moved = false;
    for (k, &m) in mix.iter().enumerate() {
        if m > best_m {
            best_m = m;
            best = Litho::ALL[k];
            moved = true;
        }
    }
    // [`Litho::as_deposited`] answers *"what is this rock once a mover has set it
    // down"*, so it applies to the **carried** winner and not to the tag's own
    // default: the default was never carried anywhere and the record has always
    // been allowed to say what it says. (On today's world the distinction is
    // inert — the erosion recorder builds mineral tags only, and `litho_of_tag`
    // maps those to clastics, which `as_deposited` leaves alone — but the rule
    // should be right rather than accidentally right.)
    if moved { best.as_deposited() } else { best }
}

/// **Where the transport pass picked material up and where it put it down**,
/// summed over a whole run in metres (Movement 2b). All zero when material-aware
/// transport is off.
///
/// It exists because "the facies gradient did not express" is not one finding, it
/// is three, with three different heirs:
///
/// - **nothing was picked up** — the pass is not the thing shaping this landscape,
///   and the heir is the erosion budget, not the sorting rule;
/// - **it was picked up and set straight back down** (`by_competence` ≈
///   `entrained + incised`) — the flows cannot carry what the hillslopes supply,
///   and the heir is the competence calibration or the discharge;
/// - **it travelled and then fined** — the slice worked.
///
/// A single "did the gradient appear" number cannot tell those apart, and
/// guessing between them is how a slice gets tuned in the wrong place.
#[derive(Clone, Copy, Default, Debug)]
pub struct TransportLedger {
    /// Loose cover lifted into the load at its source (`loose→load`, § 13.6).
    pub entrained_m: f64,
    /// Bedrock detached into the load by incision.
    pub incised_m: f64,
    /// Set down because the flow ran out of **capacity** — the coarsest-first draw.
    pub deposited_by_capacity_m: f64,
    /// Set down because the flow ran out of **competence** — the falling ceiling.
    pub deposited_by_competence_m: f64,
    /// Set down at a sink (the sea, or the domain border), where everything
    /// suspended settles regardless.
    pub deposited_at_sink_m: f64,
    /// **The control the whole diagnosis turns on:** bedrock converted to regolith
    /// *in place* by the weathering phase, over the run. This material never enters
    /// a load and never travels, and if it dwarfs `entrained_m` then the archive is
    /// not a fluvial deposit at all and no amount of sorting can make it read like
    /// one.
    pub weathered_m: f64,
    /// The other control: regolith moved by **hillslope diffusion**, summed as the
    /// per-epoch gain side of the gather (so it is mass *moved*, not net change,
    /// which is zero by construction). Creep is the § 13.2 gravity/mass-wasting
    /// family — a transport agent this slice deliberately does not make
    /// material-aware — so this number is how much of the world's sediment routing
    /// the slice did **not** reach.
    pub diffused_m: f64,

    // ---- the DENUDATION itemisation (journal/0111) ------------------------
    //
    // Everything above measures material **moving inside** the landscape.
    // Denudation is a different question: how much leaves the *land system*
    // altogether. Weathering in place is not denudation; creeping one cell
    // downslope is not denudation. Crossing the shoreline is.
    //
    // These are the complete set of ways mass crosses from a subaerial cell to a
    // submerged one (or piles against the domain edge) in this engine, and they
    // are accumulated **only when [`Erosion::set_denudation_ledger`] is on** —
    // off, they are exactly zero, no branch fires, and the run is byte- *and
    // cost*-identical to production. They are read-only with respect to the
    // physics: every one is a `+=` on this struct.
    /// Fluvial load deposited at a sink that is **submerged** at the epoch's sea
    /// stand — sediment yield to the sea, the closest thing this engine has to
    /// what a gauging station or a cosmogenic-nuclide catchment average measures.
    pub sink_marine_m: f64,
    /// Fluvial load deposited at a sink that is a **subaerial domain-border**
    /// cell. Not denudation: it is still on land, piled against the edge of the
    /// simulated box. Broken out so it can never be quietly counted as export.
    /// `sink_marine_m + sink_border_m == deposited_at_sink_m` — the itemisation
    /// the gate asserts.
    pub sink_border_m: f64,
    /// Regolith **crept across the shoreline** by hillslope diffusion — the
    /// land→sea half of the gather's edge fluxes. Creep moves 918× the fluvial
    /// load on this world (corrections #55), so this is the term that decides the
    /// answer, and it is the one no prior instrument could see.
    pub creep_to_sea_m: f64,
    /// Shore material quarried by the **wave** agent and deposited offshore
    /// (loose cover first, then bedrock). By construction the sink is submerged,
    /// so all of it is export.
    pub wave_offshore_m: f64,
    /// The bedrock share of [`Self::wave_offshore_m`]. Wave attack lowers `R`
    /// *after* [`Erosion::track_exhumation`] has run, so this material is
    /// **missing from `grid.exhum`** and must be added back when bedrock erosion
    /// is totalled.
    pub wave_bedrock_m: f64,
    /// Airborne **dust settling on the sea** during the eolian march.
    pub eolian_to_sea_m: f64,
    /// **Cell-epochs in which the creep flux limiter bound** — the cell wanted to
    /// shed more regolith than it had, so [`diffuse_scale_cell`] clamped its export
    /// to its whole `H` (journal/0114).
    ///
    /// **This is the honesty check on the erosional calibration, and it measures a
    /// discretisation limit rather than a physics one.** Hillslope diffusion is
    /// explicit: a cell's potential outflow is `Σ_downhill diffusion · Δsurf`, and
    /// once that exceeds the regolith present, the pass stops being a diffusion and
    /// becomes *"move everything one cell downslope this epoch"*. At 460 m cells and
    /// 2.5 Myr per epoch that conveyor is a creep velocity of ~0.18 mm/yr, which is
    /// squarely inside the measured range for real soil creep — so the *rate* stays
    /// honest even where the *operator* has degenerated. What is lost is sub-cell
    /// structure, not magnitude.
    ///
    /// Reported so a calibration cannot quietly buy its denudation by pushing the
    /// whole world into that regime. **Counts only cells that actually held
    /// regolith** — see [`Self::creep_cell_epochs`]. Counted only when
    /// [`DeepConfig::denudation_ledger`](super::grid::DeepConfig::denudation_ledger)
    /// is on; zero, and not even summed, in production.
    pub creep_limited_cell_epochs: u64,
    /// Cell-epochs in which the diffusion pass ran **over a cell that had regolith
    /// to move** — the denominator for [`Self::creep_limited_cell_epochs`].
    ///
    /// The `h > 0` restriction is load-bearing, not tidiness. The limiter's predicate
    /// is `potential outflow > available cover`, which is *trivially* true at `h == 0`,
    /// so counting every cell would score the bare ocean floor and every stripped ridge
    /// as transport-limited and report a saturation that is really an absence. Same
    /// gating.
    ///
    /// **Since journal/0122 the unit is a cell-SUB-STEP**, because the pass sub-cycles
    /// (`DeepConfig::creep_substep`). Both this and
    /// [`Self::creep_limited_cell_epochs`] count every sub-step, so the *ratio* — which
    /// is the only thing anyone reads — is unchanged in meaning, and identical in value
    /// on any world that takes one sub-step. Named `_cell_epochs` still, because
    /// renaming a counter whose ratio is quoted in four journal entries buys nothing.
    pub creep_cell_epochs: u64,
}

impl TransportLedger {
    /// **Total export from the subaerial land system** over the run, in metres of
    /// cell-thickness — the catchment-averaged denudation numerator. Border
    /// accumulation is deliberately *excluded*: it never left the land.
    pub fn exported_m(&self) -> f64 {
        self.sink_marine_m + self.creep_to_sea_m + self.wave_offshore_m + self.eolian_to_sea_m
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
    /// **The stream-transport coefficient the last [`Self::transport`] ran with** —
    /// the reference the energy bands and the competence ceiling are expressed
    /// *relative to* (journal/0114).
    ///
    /// Transport capacity is `cap = k_transport · A^m · S^n`, so a capacity in
    /// isolation is not a geomorphic quantity: it is `k_transport` multiplied by
    /// one. The Low/Medium/High boundaries and [`COMPETENCE_PER_KT`] are statements
    /// about **where in a drainage network you are** — `A^m·S^n` — and they were
    /// written as absolute capacities only because `k_transport` had never moved.
    /// Carrying the reference here is what lets the erosional calibration scale the
    /// rate without re-labelling every depositional environment on the world as
    /// high-energy.
    k_transport: f64,
    scale: Vec<f64>,
    /// Diffusion gather scratch: per-cell net ΔH, applied after the gather.
    netdiff: Vec<f64>,
    /// **The epoch's TOTAL diffusion ΔH when the pass sub-cycles** (journal/0122).
    /// Empty — and never touched — when `creep_substeps == 1` or creep carries no
    /// identity, so the shipped configuration's scratch residency is unmoved. It
    /// exists solely so the species itemisation is audited against the sum of the
    /// sub-steps rather than against the last one.
    netdiff_acc: Vec<f64>,
    /// **How many sub-steps the last [`Erosion::diffuse`] took** to keep its
    /// per-edge coefficient inside [`CREEP_MAX_EDGE_COEFF`]. `1` on every world
    /// whose peak effective creep diffusivity already sits inside the bound.
    creep_substeps: u32,
    /// The peak effective creep diffusivity the last [`Erosion::diffuse`] actually
    /// reduced — the numerator of its sub-step count. Held because
    /// [`Erosion::max_eff_creep`] re-derived after the run answers about a
    /// **different epoch**: the biotic pass rewrites `bio_resist` *after* erosion
    /// (the lagged coupling), so the post-run planes are one epoch ahead of the ones
    /// the last diffusion saw. A gate that compares the count to a re-derivation is
    /// comparing across that lag; this is the epoch-matched value.
    creep_peak_coeff: f64,
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
    /// **The MFD partition** (flow.md § 2.6, § 2.6.2). `Some(params)` ⇒
    /// multi-receiver routing under the hybrid-`p` law ([`MfdParams`]); `None` ⇒
    /// the single-receiver D8 path, which is the pre-MFD solve byte for byte.
    mfd: Option<MfdParams>,
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
    /// **Material-aware hillslope creep** (Movement 2b continuation (b)) — the
    /// gravity/mass-wasting member of § 13.2's transport family. Off ⇒
    /// [`Self::creep_sp`] is empty, [`Self::diffuse`] never runs its third pass,
    /// and the record names the arriving unit exactly as the fluvial-only slice
    /// did. Requires [`Self::sorted`], because the composition it moves is the
    /// same `outcrop_shares` plane the load entrains from — one walk, now three
    /// consumers.
    creep_carries: bool,
    /// `n × SPECIES` — the **net** metres of each species creep delivered to (or
    /// took from) each cell this epoch. Signed: a hillslope cell loses species it
    /// sheds and gains what came down from above.
    ///
    /// It is an **attribution, not a mass authority** — `grid.h` still moves by
    /// the scalar `netdiff`, bit for bit as before. Read by [`Self::record`] to
    /// name the arriving unit and by the outcome probe.
    creep_sp: Vec<f64>,
    /// `n` — the **gross** creep traffic through each cell this epoch (every edge
    /// flux, in or out). The itemisation audit's denominator; see
    /// [`diffuse_species_cell`].
    creep_gross: Vec<f64>,
    /// **The creep itemisation audit** — the largest relative gap, over every
    /// (cell, epoch), between `Σ_species creep_sp[cell]` and the scalar `netdiff`
    /// the terrain actually moved. The standing probe-defect shape in this repo is
    /// *an itemisation that stops equalling its own total*, and this is that check
    /// on the identity path, taken continuously rather than once.
    creep_itemisation_residue: f64,
    /// **The creep conservation audit** — the largest relative gap, over every
    /// (species, epoch), between `Σ_cells creep_sp[·][s]` and **zero**. Creep only
    /// *moves* material: whatever any cell gained of a species, some other cell
    /// lost. This is the global half, and it is the one an antisymmetry mistake
    /// would show up in.
    creep_conservation_residue: f64,
    /// How many (cell → neighbour) creep faces carried flux in the **last** epoch
    /// — the cost input for stubs.md #18's gravity mover in the flow record.
    creep_faces: usize,
    /// **The per-species split audit.** The largest *relative* discrepancy, over
    /// every `(cell, species, epoch)` of the run, between what a cell held of a
    /// species and the sum of what its receivers were handed of it.
    ///
    /// It is a running maximum rather than a stored plane because that is all the
    /// claim needs: a leak anywhere is a leak. journal/0109 proved the scalar split
    /// with a per-cell residue test over a stored plane; the per-species plane
    /// would be `n × 8 × 7` and is not worth 133 MB to assert a scalar.
    split_residue: f64,
    /// **The transport ledger** (Movement 2b instruments) — running totals over
    /// the whole run, in metres, of what the pass picked up and where it put it
    /// down. Gen-time only, five `f64`s, and they are what turns "the facies
    /// gradient did not express" from a shrug into a diagnosis: a load that never
    /// leaves its source cell and a load that is never picked up at all are very
    /// different failures with very different heirs.
    ledger: TransportLedger,
    /// **The denudation ledger switch** (journal/0111). Off by default and off in
    /// production: the five export counters on [`TransportLedger`] stay exactly
    /// zero, the shoreline-creep sweep in [`Self::diffuse`] never runs, and the
    /// solve is byte- and cost-identical. On, the phases additionally tally what
    /// crosses out of the land system. Nothing it does writes to the grid.
    denude: bool,
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
            // Overwritten by every `transport` call before anything reads it; the
            // seed value is `DeepConfig::default()`'s so a harness that classifies
            // without ever transporting sees the historical thresholds.
            k_transport: 0.0016,
            scale: vec![0.0; n],
            netdiff: vec![0.0; n],
            netdiff_acc: Vec::new(),
            creep_substeps: 1,
            creep_peak_coeff: 0.0,
            litho: Vec::new(),
            sus_flow: Vec::new(),
            sus_creep: Vec::new(),
            frost: Vec::new(),
            cur_chapter: 0,
            forcing: Vec::new(),
            r_snap: Vec::new(),
            out_load: Vec::new(),
            mfd: None,
            mfd_w: Vec::new(),
            out_area: Vec::new(),
            out_face_load: Vec::new(),
            sorted: false,
            w_settle: [0.0; SPECIES],
            ws_order: [0; SPECIES],
            bedrock_sp: [0.0; SPECIES],
            qs_sp: Vec::new(),
            shares: Vec::new(),
            dep_sp: Vec::new(),
            creep_carries: false,
            creep_sp: Vec::new(),
            creep_gross: Vec::new(),
            creep_itemisation_residue: 0.0,
            creep_conservation_residue: 0.0,
            creep_faces: 0,
            split_residue: 0.0,
            ledger: TransportLedger::default(),
            denude: false,
            heap: BinaryHeap::new(),
        }
    }

    /// Turn the **denudation ledger** on (journal/0111) — the five export
    /// counters on [`TransportLedger`] that measure what leaves the *land system*
    /// rather than what moves inside it.
    ///
    /// It is a flag rather than always-on for one reason: the shoreline-creep
    /// term needs a per-epoch sweep over every cell's four edges, which is real
    /// gen-time work for a number no production consumer reads. Off is therefore
    /// the S-5 identity floor — zero counters, zero branches, zero cost — and the
    /// measurement probe is the only caller that turns it on. **The gate asserts
    /// the surface plane is bit-identical either way**, which is what makes a
    /// number taken with the flag on a number about the shipped world.
    pub fn set_denudation_ledger(&mut self, on: bool) {
        self.denude = on;
    }

    /// Whether the denudation ledger is on.
    #[inline]
    pub fn is_denudation_ledger(&self) -> bool {
        self.denude
    }

    /// Split a sink's arriving load into **export** and **edge pile-up**
    /// (journal/0111). A sink is either a submerged cell — in which case the
    /// sediment has left the land system and is the yield a real catchment study
    /// would weigh — or a subaerial domain-border cell, in which case it has not
    /// left anything and is an artifact of simulating a box. Conflating the two
    /// would inflate denudation by whatever the border happens to catch, so the
    /// two are counted apart and the gate asserts they re-sum to the total.
    ///
    /// Read-only: it inspects the grid and adds to the ledger. No-op when the
    /// denudation ledger is off.
    #[inline]
    fn tally_sink(&mut self, grid: &DeepGrid, c: usize, qin: f64) {
        if !self.denude {
            return;
        }
        if grid.surf_at(c) <= self.sea_level {
            self.ledger.sink_marine_m += qin;
        } else {
            self.ledger.sink_border_m += qin;
        }
    }

    /// **How much regolith creeps across the shoreline this epoch** — the
    /// land→sea half of the diffusion gather's edge fluxes (journal/0111).
    ///
    /// This is the term the answer turns on, because creep moves 918× what the
    /// rivers pick up on this world (corrections #55) and no instrument before
    /// this one could say how much of it actually *leaves*. It re-reads exactly
    /// the influx expression [`diffuse_net_cell`] uses for a submerged cell `j`
    /// from a higher subaerial neighbour `i` — the same donor scale, the same
    /// effective diffusivity — so the number is a partition of a flux the solve
    /// already computed, not a second model of it.
    ///
    /// Called after the gather's two passes and before they are applied, on the
    /// same frozen surface, so it sees the epoch the flux belongs to.
    fn tally_creep_to_sea(&mut self, grid: &DeepGrid, _cfg: &DeepConfig, diff: f64) {
        let (w, sea) = (self.w, self.sea_level);
        let mut sum = 0.0;
        for j in 0..self.n {
            if self.surf[j] > sea {
                continue; // the receiving cell must be under water
            }
            let (gx, gy) = coords_of(j, w);
            for (dx, dy) in NEIGH4 {
                let Some(i) = in_grid(gx + dx, gy + dy, w) else {
                    continue;
                };
                // Only a *subaerial* donor standing above the water is denudation:
                // a subsea→subsea edge is marine redistribution, and a downhill
                // edge out of `j` is not influx at all.
                if self.surf[i] <= sea {
                    continue;
                }
                let d = self.surf[i] - self.surf[j];
                if d > 0.0 {
                    sum += eff_diff(diff, &grid.bio_resist, &self.sus_creep, i) * d * self.scale[i];
                }
            }
        }
        self.ledger.creep_to_sea_m += sum;
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

    /// Turn **material-aware hillslope creep** on (Movement 2b continuation (b),
    /// `material-behavior.md` § 13.2 — the gravity/mass-wasting member of the
    /// transport family). Off is the anonymous-creep path, byte for byte, because
    /// the species plane stays empty and [`Self::record`] falls back to the
    /// fluvial-only mixture.
    ///
    /// **Gated on [`Self::set_material_transport`]**, and that is not a
    /// convenience: what creep moves is the near-surface composition the
    /// `outcrop_shares` seam publishes into [`Self::shares`], which only exists on
    /// the material-aware path. Asking for creep identity without it would have to
    /// take a *second* composition walk beside the one already running — the
    /// re-invention-next-door this project keeps catching itself at (spines A-4).
    pub fn set_material_creep(&mut self, on: bool) {
        self.creep_carries = on && self.sorted;
        if !self.creep_carries {
            self.creep_sp = Vec::new();
            self.creep_gross = Vec::new();
            return;
        }
        if self.creep_sp.len() != self.n * SPECIES {
            self.creep_sp = vec![0.0; self.n * SPECIES];
            self.creep_gross = vec![0.0; self.n];
        }
    }

    /// Whether material-aware hillslope creep is on.
    #[inline]
    pub fn is_material_creep(&self) -> bool {
        self.creep_carries
    }

    /// **What hillslope creep delivered to each cell in the last epoch, per
    /// species** (`n × SPECIES` metres, signed; empty when creep carries no
    /// identity). The colluvial half of the mixture [`Self::record`] names the
    /// arriving unit from.
    pub fn creep_species(&self) -> &[f64] {
        &self.creep_sp
    }

    /// **The largest relative gap between the creep itemisation and its own
    /// total**, over every (cell, epoch) of the run. `0.0` when creep carries no
    /// identity.
    pub fn max_creep_itemisation_residue(&self) -> f64 {
        self.creep_itemisation_residue
    }

    /// **The largest relative amount of any species creep created or destroyed**,
    /// over every (species, epoch) of the run. `0.0` when creep carries no
    /// identity. Creep only moves material, so the honest value is zero to
    /// round-off.
    pub fn max_creep_conservation_residue(&self) -> f64 {
        self.creep_conservation_residue
    }

    /// How many donor→receiver creep faces carried flux in the last epoch — the
    /// entry count a gravity-caused flow record would pay per chapter
    /// (stubs.md #18).
    pub fn creep_outflux_faces(&self) -> usize {
        self.creep_faces
    }

    /// The settling velocity of each species, indexed by [`Litho::index`] (all
    /// zero when material-aware transport is off).
    pub fn settling(&self) -> &[f64; SPECIES] {
        &self.w_settle
    }

    /// **The largest relative per-species split residue over the whole run**
    /// (`0.0` when material-aware transport is off, or when every split was exact).
    ///
    /// This is the *local* half of the per-species mass proof, the direct analogue
    /// of journal/0109's `the_partition_leaves_no_residue`: for every cell, every
    /// species and every epoch, the shares handed to the receivers summed to what
    /// the cell held. The *global* half — a share written to the record but never
    /// added to a neighbour — is the whole-world `Δ(ΣR + ΣH) == uplift + biotic`
    /// ledger, which this cannot see and which cannot see this.
    pub fn max_species_split_residue(&self) -> f64 {
        self.split_residue
    }

    /// **What the transport pass set down at each cell in the last epoch, per
    /// species** (`n × SPECIES` metres; empty when material-aware transport is
    /// off). The measurement surface for the downstream-fining gradient — and the
    /// authority [`Self::record`] names the arriving unit from.
    pub fn deposited_species(&self) -> &[f64] {
        &self.dep_sp
    }

    /// **The suspended load still in flight after the last transport**
    /// (`n × SPECIES` metres; empty when material-aware transport is off). Cell
    /// `c`'s slice is what it handed onward — the quantity the competence
    /// invariant is read against.
    pub fn load_species(&self) -> &[f64] {
        &self.qs_sp
    }

    /// The transport capacity seen at each cell in the last epoch (`cap`, the
    /// stream-power budget) — the flow's own energy, which is what sets its
    /// competence ceiling.
    pub fn energy(&self) -> &[f64] {
        &self.energy
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

    /// Turn **multiple-flow-direction routing** on under the hybrid-`p` law
    /// ([`MfdParams`], flow.md § 2.6/§ 2.6.2). `None` is the single-receiver D8
    /// path — the pre-MFD solve, byte for byte, because nothing else in the class
    /// reads `mfd_w`. [`MfdParams::uniform`] is journal/0109's one-exponent solve.
    pub fn set_mfd(&mut self, p: Option<MfdParams>) {
        self.mfd = p;
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

    /// **The transport ledger over the whole run** (all zero when material-aware
    /// transport is off). See [`TransportLedger`].
    pub fn transport_ledger(&self) -> TransportLedger {
        self.ledger
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
            + self.netdiff_acc.len()
            + self.sus_flow.len()
            + self.sus_creep.len()
            + self.frost.len()
            + self.mfd_w.len()
            + self.qs_sp.len()
            + self.shares.len()
            + self.dep_sp.len()
            + self.creep_sp.len()
            + self.creep_gross.len();
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

    /// **The solve's own processing order** — cells by ascending filled surface,
    /// the array [`Self::transport`] walks in reverse.
    ///
    /// Public so an instrument can traverse the transport graph **without
    /// re-deriving it** (S-3: the summary derived from the authority, never beside
    /// it). A probe that computed its own D8 receivers would be a second routing
    /// rule that can silently disagree with the one the world was built by.
    pub fn processing_order(&self) -> &[u32] {
        &self.order
    }

    /// **Every cell this cell hands load to, this epoch** — the single D8 receiver
    /// off the MFD path, or every MFD direction carrying positive weight. Yields
    /// nothing for a sink.
    ///
    /// The companion to [`Self::processing_order`]; together they are the transport
    /// graph exactly as [`Self::transport`] walks it.
    pub fn out_edges(&self, c: usize, f: &mut dyn FnMut(usize)) {
        if self.mfd.is_none() {
            let rc = self.recv[c];
            if rc >= 0 {
                f(rc as usize);
            }
            return;
        }
        let base = c * MFD_DIRS;
        for d in 0..MFD_DIRS {
            if self.mfd_w[base + d] > 0.0 {
                f(self.mfd_neighbour(c, d));
            }
        }
    }

    /// Set the paleo-sea-level stand the standalone phase methods read (the
    /// profiling harness drives phases individually; [`Self::step`] sets this).
    pub fn set_sea_level(&mut self, sea_level: f64) {
        self.sea_level = sea_level;
    }

    /// One deep-time iteration at the given `sea_level` stand. Returns the
    /// total uplift added this step (for the mass-conservation ledger).
    ///
    /// `mem` is the deposition-identity context (P11 slice 1) — the registered
    /// content plus this epoch's addressed member-fitness stream. It is consulted
    /// only by the phases that write an identity (`record`, `wind`, `wave`), so a
    /// caller running with `cfg.record` off never reaches it; it is still a
    /// parameter rather than a default because *which members exist* is a world
    /// input and must not be assumed by a solver.
    pub fn step(
        &mut self,
        grid: &mut DeepGrid,
        cfg: &DeepConfig,
        sea_level: f64,
        mem: MemberCtx<'_>,
    ) -> f64 {
        self.sea_level = sea_level;
        // Phase 1: the external forcing. Legacy path adds a constant uplift plane
        // to bedrock; tectonic-history path adds the analytic thickening rate to
        // the crustal columns (elevation is then *derived* by isostasy, phase 8b).
        // `step` is the one-epoch driver (profiling harnesses + the erodibility /
        // deeptime tests still call it), so its phase length is one epoch: `dt =
        // 1.0`. The runner drives the same phase methods at the cadence the world
        // authored — this is the constant the RATE axis replaced there.
        let legacy_uplift = if cfg.tectonic_history {
            self.apply_thickening(grid, 1.0);
            0.0
        } else {
            self.apply_uplift(grid, 1.0)
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
        self.diffuse(grid, cfg, 1.0);
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
            self.record(grid, mem);
        }
        // The wind and wave agents run **after** the recorder and self-record
        // (like the biotic layer), so their own facies reach the record rather
        // than being lumped under the epoch's fluvial tag. Both only redistribute
        // mass — wind moves loose `H`, wave moves `R`/`H` offshore — so the mass
        // ledger `Δ(ΣR+ΣH) == uplift + biotic` is untouched (journal/0034).
        if cfg.full_agents {
            self.wind(grid, cfg, mem);
            self.wave(grid, cfg, mem);
        }
        ledger
    }

    // ---- phase 1 (tectonic): crustal thickening + isostasy ----------------

    /// Add the blended analytic thickening rate into the crustal columns
    /// (§ 4.2/§ 5.2). Positive rates thicken (orogeny/arc), negative thin
    /// (rift/trench/ridge); `t_crust` is floored so a column cannot thin to
    /// nothing. Purely per-cell → byte-identical parallel. Elevation is untouched
    /// here — that is isostasy's job.
    ///
    /// **`dt` is the phase length** (RATE, journal/0123): the forcing plane is a
    /// thickening *rate* per epoch, so a turn covering `dt` epochs adds `f × dt`.
    /// `dt = 1.0` is bit-identical to the pre-RATE pass.
    pub fn apply_thickening(&mut self, grid: &mut DeepGrid, dt: f64) {
        let forcing = &self.forcing;
        if forcing.is_empty() {
            return;
        }
        if self.par() {
            grid.t_crust
                .par_iter_mut()
                .zip(forcing.par_iter())
                .for_each(|(t, f)| *t = (*t + *f * dt).max(1000.0));
        } else {
            for (t, f) in grid.t_crust.iter_mut().zip(forcing.iter()) {
                *t = (*t + *f * dt).max(1000.0);
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

    /// Add the per-cell uplift into bedrock. Returns the total added.
    ///
    /// **`dt` is the phase length** (RATE, journal/0123): `grid.uplift` is metres
    /// *per epoch*, so a turn covering `dt` epochs adds `u × dt` — and the ledger
    /// total it returns scales with it, or the mass-conservation check would be
    /// reading a different amount of time than the grid got. `dt = 1.0` is
    /// bit-identical to the pre-RATE pass.
    pub fn apply_uplift(&mut self, grid: &mut DeepGrid, dt: f64) -> f64 {
        if self.par() {
            grid.r
                .par_iter_mut()
                .zip(grid.uplift.par_iter())
                .for_each(|(r, u)| *r += *u * dt);
        } else {
            for i in 0..self.n {
                grid.r[i] += grid.uplift[i] * dt;
            }
        }
        self.uplift_sum * dt
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
    ///
    /// ## The one-epoch lag on drainage area, and why it is not a cheat
    ///
    /// The hybrid-`p` law ([`MfdParams`]) needs the cell's drainage area to decide
    /// how channelised it is — and drainage area is computed by
    /// [`Self::accumulate_area`], which runs *after* this phase and *from* the
    /// weights this phase writes. The dependency is genuinely circular, and there
    /// is no fixed point to iterate to: a second accumulation pass would use an
    /// area that its own weights then invalidate.
    ///
    /// So the exponent reads `self.area` as this phase finds it — **the previous
    /// epoch's accumulation**. This is the same shape as the S10 biotic coupling
    /// (`grid.bio_weather` is a lagged plane for exactly this reason) and it costs
    /// nothing: no extra storage, no extra pass, and `accumulate_area` re-seeds
    /// `area` to `1.0` at its own start so nothing stale survives into the sums.
    ///
    /// At epoch 0 the plane is all zeros, so `χ = 0`, the exponent is `p_hill`
    /// everywhere and the first epoch is maximally dispersive. That is the honest
    /// initial condition rather than a guess: nothing is channelised until water
    /// has run once, which is also the physical statement.
    pub fn route(&mut self) {
        let (w, sea, cell_m) = (self.w, self.sea_level, self.cell_m);
        let parallel = self.par();
        let Some(mp) = self.mfd else {
            let surf = &self.surf;
            let filled = &self.filled;
            if parallel {
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
        // Disjoint field borrows: `area` is read-only here (so the parallel path
        // stays byte-identical), `recv`/`mfd_w` are written per cell.
        let Erosion {
            surf,
            filled,
            area,
            recv,
            mfd_w,
            ..
        } = self;
        let (surf, filled, area) = (&*surf, &*filled, &*area);
        let step = |i: usize, r: &mut i32, ws: &mut [f64]| {
            let best = partition_cell(i, w, surf, filled, sea, area[i], cell_m, &mp, ws);
            *r = if best < 0 {
                -1
            } else {
                let (dx, dy) = NEIGH8[best as usize];
                let (gx, gy) = coords_of(i, w);
                in_grid(gx + dx, gy + dy, w).expect("a weighted direction is in-grid") as i32
            };
        };
        if parallel {
            recv.par_iter_mut()
                .zip(mfd_w.par_chunks_mut(MFD_DIRS))
                .enumerate()
                .for_each(|(i, (r, ws))| step(i, r, ws));
        } else {
            for (i, (r, ws)) in recv.iter_mut().zip(mfd_w.chunks_mut(MFD_DIRS)).enumerate() {
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
                self.ledger.entrained_m += ent;
                let add = split_by_shares(ent, &self.shares[base..base + SPECIES]);
                for (k, a) in add.iter().enumerate() {
                    self.qs_sp[base + k] += a;
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
                    self.ledger.incised_m += inc;
                    let add = split_by_shares(inc, &self.bedrock_sp);
                    for (k, a) in add.iter().enumerate() {
                        self.qs_sp[base + k] += a;
                    }
                }
            }
            if self.sorted {
                self.settle_above_competence(grid, c, cap, cfg.k_transport);
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
                self.ledger.deposited_by_capacity_m += placed;
                self.settle_above_competence(grid, c, cap, cfg.k_transport);
                self.qs_sp[base..base + SPECIES].iter().sum()
            } else {
                grid.h[c] += dep;
                self.dh[c] += dep;
                cap
            }
        }
    }

    /// **COMPETENCE — the falling ceiling** (Movement 2b, `material-behavior.md`
    /// § 13.5). Every species in cell `c`'s outgoing load whose settling velocity
    /// exceeds what a flow of capacity `cap` can hold is set down **here**, however
    /// much capacity the flow has to spare.
    ///
    /// Capacity says *how much* a flow can carry; competence says *what*. Only the
    /// second one produces a facies: without it a stream with spare capacity would
    /// carry boulders to the sea and nothing would ever fine downstream. § 13.5's
    /// instruction is exact — *"sorting is the falling ceiling; we write the
    /// ceiling, not the sort"* — and this is the whole of the sort: no list is
    /// reordered anywhere, the ceiling simply falls as energy falls and the load is
    /// whatever is still under it.
    ///
    /// It runs **last**, on the load actually leaving, so it covers the material
    /// this cell just entrained or incised as well as what arrived: a reach that
    /// prises loose a grain size it cannot lift drops it straight back, which is
    /// the honest outcome and keeps the invariant clean — **nothing leaves a cell
    /// that the cell could not carry**. (That is deliberately *not* armouring: the
    /// grain stays in the ordinary loose cover and is tried again next epoch, so no
    /// permanent lag or pavement forms. Selective *entrainment* — the fine-side,
    /// cohesion-driven half of Hjulström's curve, which is what actually armours a
    /// bed — is § 13.4 and is deferred.)
    #[inline]
    fn settle_above_competence(&mut self, grid: &mut DeepGrid, c: usize, cap: f64, k_t: f64) {
        let ceiling = competence_ceiling(cap, k_t);
        let base = c * SPECIES;
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
            self.ledger.deposited_by_competence_m += rained;
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
        // The coefficient this epoch's capacities are built from, kept so the
        // recorder can classify them against the same reference (journal/0114).
        self.k_transport = cfg.k_transport;
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
                        self.ledger.deposited_at_sink_m += qin;
                    }
                    self.tally_sink(grid, c, qin);
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
                    self.ledger.deposited_at_sink_m += qin;
                }
                self.tally_sink(grid, c, qin);
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
                for (d, ft) in face_total.iter_mut().enumerate() {
                    let wt = self.mfd_w[base + d];
                    if wt <= 0.0 {
                        continue;
                    }
                    let share = if d == last { q_s - given } else { wt * q_s };
                    given += share;
                    *ft += share;
                    let j = self.mfd_neighbour(c, d);
                    self.qs_sp[j * SPECIES + s] += share;
                }
                // The audit: what the receivers were handed, against what the cell
                // held. Relative, because loads span many orders of magnitude.
                let residue = ((given - q_s) / q_s).abs();
                if residue > self.split_residue {
                    self.split_residue = residue;
                }
            }
            if record {
                for (d, ft) in face_total.iter().enumerate() {
                    if self.mfd_w[base + d] > 0.0 {
                        self.out_face_load[base + d] = *ft as f32;
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
        // A Movement 2b **control**, not a term: how much regolith this world makes
        // in place, against how much its rivers ever pick up (`TransportLedger`).
        // Reading a plane cannot perturb it, and it is summed only when
        // material-aware transport is on, so the scalar path is untouched.
        let h_before = if self.sorted {
            grid.h.iter().sum::<f64>()
        } else {
            0.0
        };
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
        if self.sorted {
            self.ledger.weathered_m += grid.h.iter().sum::<f64>() - h_before;
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

    /// **Flux-limited hillslope diffusion of the regolith, sub-cycled to the
    /// monotonicity bound** (journal/0122).
    ///
    /// The inner step is unchanged and is the gather form: fluxes are computed
    /// from a frozen surface and a frozen per-cell limiter, then each cell sums
    /// its own in/out edges. It conserves `ΣH` exactly (each edge's flux is
    /// referenced identically from both endpoints) and is byte-identical
    /// scalar↔parallel.
    ///
    /// # What was wrong, and what the sub-cycle fixes
    ///
    /// The step above is an **explicit** Laplacian, and an explicit Laplacian has
    /// a period-2 grid-scale mode whenever its per-edge coefficient exceeds
    /// [`CREEP_MAX_EDGE_COEFF`]. At `diffusion = 5.4` (the shipped rate under
    /// `EROSION_CALIBRATION`) the coefficient is **43× past** that bound, and the
    /// flux limiter — which caps a cell's export at its *entire inventory* rather
    /// than at the amount that would level the pair — turns the divergence into a
    /// **saturated flip-flop**: cell A hands B all of its cover, B is now higher
    /// and hands it back, forever, at an amplitude set by the cover rather than by
    /// the rate. That is exactly why journal/0116 measured the limiter to be
    /// *deaf to the time step* (96.0 → 94.7 % binding under a 4× refinement) while
    /// the regolith checkerboard *sharpened* (ACF −0.82 → −0.93).
    ///
    /// So the fix is **not** a smaller `dt` asked of the caller, and it is not a
    /// cap on the rate either — capping would reinstate the one-cell-per-epoch
    /// conveyor that `stubs.md` #27 is about. The pass takes the epoch it is given
    /// and **splits it internally** into `n` steps each of which respects the
    /// bound, with `n` derived from the rate the config actually states:
    ///
    /// ```text
    /// n = ceil( max_cell eff_diff / CREEP_MAX_EDGE_COEFF )
    /// ```
    ///
    /// `n = 1` reproduces the previous operator **bit for bit** (`x / 1.0 == x`),
    /// so every world whose peak effective diffusivity already sat inside the
    /// bound is untouched.
    ///
    /// **The shipped configuration is NOT such a world** — and the sentence that
    /// used to end this paragraph said it was (anti-shape A-2, caught by the
    /// 2026-07-29 spine-audit). journal/0122 measured the shipped world at **2.1×
    /// past** the bound; it takes **`n = 2`**, `DeepConfig::creep_substep` defaults
    /// **on**, and **its goldens moved with the fix**. A reader of this file alone
    /// would have concluded that no golden could have moved, which is exactly
    /// backwards. The reachable `n = 1` fixed point is
    /// `creep_substep: false` — asserted by name in `tests/creep_operator.rs`
    /// against `GOLDEN_SURFACE_UNBOUNDED_CREEP` / `GOLDEN_RECORD_UNBOUNDED_CREEP` —
    /// and `the_shipped_world_has_cells_past_the_bound` in the same file is the
    /// standing check that the claim above stays true.
    ///
    /// **Stability is therefore independent of the rate the caller states**, which
    /// is the property the brief asked for: the operator does not require a small
    /// `dt` to behave, because it no longer takes the caller's `dt` as its
    /// integration step — it takes `dt` as the *amount of world time to integrate*
    /// (RATE, journal/0123) and picks its own step inside it. The cost is `n×` the pass, paid at gen time, where this
    /// project's doctrine is explicit that time is not the constraint and
    /// simulation is never cheapened to save it.
    ///
    /// **The max is over cells, not over the config**, because [`eff_diff`] folds
    /// in biotic root cohesion and the lithology's creep susceptibility — a
    /// peat-dominated cell can carry several times the config rate, and it is the
    /// worst cell that decides whether *any* cell may oscillate. `f64::max` is
    /// order-independent, so the reduction cannot make the run non-deterministic.
    pub fn diffuse(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, dt: f64) {
        // **`dt` from outside; the sub-cycle from inside** (RATE, journal/0123).
        // `cfg.diffusion` is a rate *per epoch*; the phase length the runner hands
        // this pass says how many epochs this turn covers, so the diffusivity the
        // turn actually applies is the product. Then — and only then — the
        // stability sub-cycle below divides that down to the von Neumann bound.
        //
        // The two divisions are NOT the same knob and must stay in this order:
        // `dt` is **authored** (how much world time passed, a modelling choice),
        // `n_sub` is **derived** (how finely this operator must integrate it to
        // stay monotone, a numerical fact only the operator knows). That is the
        // split the user ruled on 2026-07-29 when RATE was nearly widened to own
        // substepping: *"couldn't substepping be solved within the field instead,
        // where it takes `dt` from outside and calcs its own internal multiplier?"*
        // Yes — and this is what that looks like. `stubs.md` § 30's remaining half
        // is hoisting the derived multiplier into the S-10 field-solver kernel so
        // no future pass author has to write the analysis; the authored half is
        // discharged here.
        //
        // `dt = 1.0` (the shipped roster) is bit-identical to the pre-RATE
        // operator: `x * 1.0 == x` exactly for f64, so `rate == cfg.diffusion` and
        // every quantity below is the same bit pattern it was.
        let rate = cfg.diffusion * dt;
        if rate <= 0.0 {
            return;
        }
        let d_max = self.max_eff_creep(grid, rate);
        let n_sub = if !cfg.creep_substep {
            // The pre-journal/0122 operator, bit for bit: one raw step at whatever
            // coefficient the config states. Kept reachable so the goldens captured
            // under it stay fixed points (`DeepConfig::creep_substep`).
            1
        } else if d_max > CREEP_MAX_EDGE_COEFF {
            // `ceil` of a finite positive ratio; the `max(1)` is belt-and-braces
            // against a denormal reduction, not a live case.
            ((d_max / CREEP_MAX_EDGE_COEFF).ceil() as u32).max(1)
        } else {
            1
        };
        self.creep_substeps = n_sub;
        self.creep_peak_coeff = d_max;
        let diff_sub = rate / f64::from(n_sub);
        // The species itemisation is audited against the epoch's TOTAL ΔH, so a
        // sub-cycled epoch needs somewhere to accumulate it. Allocated only when
        // both sub-cycling and identity-carrying creep are live, so the shipped
        // (n = 1) configuration's scratch residency does not move.
        let accumulate = n_sub > 1 && self.creep_carries;
        if accumulate {
            if self.netdiff_acc.len() == self.n {
                self.netdiff_acc.fill(0.0);
            } else {
                self.netdiff_acc = vec![0.0; self.n];
            }
        }
        self.creep_faces = 0;
        for s in 0..n_sub {
            self.diffuse_step(grid, cfg, diff_sub, s > 0);
            if accumulate {
                for (a, nd) in self.netdiff_acc.iter_mut().zip(self.netdiff.iter()) {
                    *a += *nd;
                }
            }
        }
        if self.creep_carries {
            self.audit_creep_species(accumulate);
        }
    }

    /// **The largest effective creep diffusivity anywhere on the grid** — the
    /// quantity [`diffuse`](Self::diffuse) divides by [`CREEP_MAX_EDGE_COEFF`] to
    /// pick its sub-cycle count. Read by the operator probe so the report can say
    /// *which* cells forced the count.
    pub fn max_eff_creep(&self, grid: &DeepGrid, diffusion: f64) -> f64 {
        let (resist, sus) = (&grid.bio_resist, &self.sus_creep);
        (0..self.n)
            .map(|i| eff_diff(diffusion, resist, sus, i))
            .fold(0.0f64, f64::max)
    }

    /// **How many sub-steps the last [`diffuse`](Self::diffuse) took.** `1` means
    /// the epoch's stated rate already sat inside [`CREEP_MAX_EDGE_COEFF`] and the
    /// operator was the pre-journal/0122 one bit for bit.
    #[inline]
    pub fn creep_substeps(&self) -> u32 {
        self.creep_substeps
    }

    /// **The peak effective creep diffusivity the last [`Self::diffuse`] divided
    /// down**, on the planes that epoch actually held. Use this, never a post-run
    /// [`Self::max_eff_creep`], to check the sub-step count against its own input:
    /// the biotic layer rewrites `bio_resist` after erosion, so a re-derivation is
    /// one epoch out of phase and disagrees by a few per cent.
    #[inline]
    pub fn creep_peak_coeff(&self) -> f64 {
        self.creep_peak_coeff
    }

    /// One sub-step of the hillslope gather, at the sub-step diffusivity
    /// `diff`. `carry_on` is set for every sub-step after the first: the species
    /// itemisation then *accumulates* rather than overwrites, so the epoch's
    /// creep plane is the sum of its sub-steps and [`Self::record`] still reads
    /// one epoch's worth of arriving colluvium.
    fn diffuse_step(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, diff: f64, carry_on: bool) {
        let parallel = self.par();
        let w = self.w;
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
        // The other Movement 2b control: how much regolith **creep** moves, against
        // how much the rivers do. The gain side only — the net is zero by
        // construction, so summing it would report nothing.
        if self.sorted {
            self.ledger.diffused_m += self.netdiff.iter().map(|v| v.max(0.0)).sum::<f64>();
        }
        // journal/0111: of the creep above, how much crossed the shoreline and so
        // actually LEFT the land. Read-only, on the same frozen surface, before
        // the gather is applied. Off ⇒ not even called.
        if self.denude {
            self.tally_creep_to_sea(grid, cfg, diff);
            // journal/0114: and how often the limiter bound — the discretisation
            // honesty check on the calibration. `scale[i] < 1.0` is exactly "this
            // cell wanted to shed more than it had".
            //
            // **Both counters skip cells with no regolith, and that is the whole
            // point of the measurement.** `out > h` is trivially true at `h == 0`,
            // so a denominator of every cell would score the bare ocean floor and
            // every stripped ridge as "transport-limited" and report a saturation
            // that is really just an absence. The question is *of the cells that had
            // something to move, how many shipped all of it* — anything else is a
            // statistic about emptiness.
            for (i, sc) in self.scale.iter().enumerate() {
                if grid.h[i] > 0.0 {
                    self.ledger.creep_cell_epochs += 1;
                    if *sc < 1.0 {
                        self.ledger.creep_limited_cell_epochs += 1;
                    }
                }
            }
        }
        // Pass 3 (Movement 2b continuation (b)): **the same fluxes, carrying
        // identity.** Runs only when creep carries material; the terrain below is
        // applied from `netdiff` either way, so this pass cannot move a metre of
        // rock — it can only name the metres the pass above already moved.
        if self.creep_carries {
            let surf = &self.surf;
            let scale = &self.scale;
            let resist = &grid.bio_resist;
            let sus = &self.sus_creep;
            let shares = &self.shares;
            let creep = &mut self.creep_sp;
            let gross = &mut self.creep_gross;
            if parallel {
                creep
                    .par_chunks_mut(SPECIES)
                    .zip(gross.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (out, g))| {
                        let f = diffuse_species_cell(
                            i, w, surf, scale, diff, resist, sus, shares, out, carry_on,
                        );
                        if carry_on {
                            *g += f;
                        } else {
                            *g = f;
                        }
                    });
            } else {
                for (i, (out, g)) in creep.chunks_mut(SPECIES).zip(gross.iter_mut()).enumerate() {
                    let f = diffuse_species_cell(
                        i, w, surf, scale, diff, resist, sus, shares, out, carry_on,
                    );
                    if carry_on {
                        *g += f;
                    } else {
                        *g = f;
                    }
                }
            }
            // Summed across sub-steps on purpose: a gravity-caused flow record
            // would pay one entry per face **per sub-step**, so the cost estimate
            // this counter exists to be (stubs.md #18) has to count them all.
            self.creep_faces += (0..self.n)
                .map(|i| diffuse_outflux_faces(i, w, &self.surf, &self.scale))
                .sum::<usize>();
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

    /// **The two creep-identity audits, taken every epoch** (Movement 2b
    /// continuation (b)).
    ///
    /// 1. **The itemisation equals its own total.** `Σ_species creep_sp[cell]` must
    ///    be the scalar `netdiff[cell]` the terrain moved. This is the check that
    ///    catches the failure this repo keeps catching — *a missing row in an
    ///    itemisation* — and it is the one that would fire if a species were ever
    ///    dropped from a split. It cannot be bit-exact because the two sums visit
    ///    the same terms in different orders (four edges of seven species against
    ///    seven species of four edges), so it is relative, scaled by the traffic
    ///    through the cell rather than by the net — a cell that gains as much as it
    ///    sheds has a net near zero and real material moving through it.
    /// 2. **No species is created or destroyed.** `Σ_cells creep_sp[·][s]` must be
    ///    zero: creep only *moves*. This is the global half, and it is where an
    ///    antisymmetry mistake between the two endpoints of an edge would land —
    ///    the direct analogue of journal/0110's `no_species_leaks_at_its_own
    ///    _junction`, taken over the whole grid because a diffusion junction has no
    ///    downstream order to walk.
    ///
    /// Running maxima rather than stored planes, for journal/0110's reason: a leak
    /// anywhere is a leak, and the per-cell per-species plane is not worth its
    /// megabytes to assert a scalar.
    fn audit_creep_species(&mut self, sub_cycled: bool) {
        // The total the itemisation must equal. A sub-cycled epoch's creep plane
        // is the sum of its sub-steps, so it is audited against the summed ΔH —
        // never against the last sub-step's, which would report a leak of
        // everything the earlier sub-steps moved.
        let net_total: &[f64] = if sub_cycled {
            &self.netdiff_acc
        } else {
            &self.netdiff
        };
        let mut sum_s = [0.0; SPECIES];
        let mut abs_s = [0.0; SPECIES];
        let mut worst_item = self.creep_itemisation_residue;
        for (i, row) in self.creep_sp.chunks(SPECIES).enumerate() {
            let mut net = 0.0;
            for (k, &v) in row.iter().enumerate() {
                net += v;
                sum_s[k] += v;
                abs_s[k] += v.abs();
            }
            let gross = self.creep_gross[i];
            if gross > 0.0 {
                let rel = (net - net_total[i]).abs() / gross;
                if rel > worst_item {
                    worst_item = rel;
                }
            }
        }
        self.creep_itemisation_residue = worst_item;
        for k in 0..SPECIES {
            if abs_s[k] > 0.0 {
                let rel = sum_s[k].abs() / abs_s[k];
                if rel > self.creep_conservation_residue {
                    self.creep_conservation_residue = rel;
                }
            }
        }
    }

    // ---- phase 9: strata recorder (PARALLEL — per-cell independent) --------

    /// Record each cell's net thickness change this iteration under the tag
    /// measured now. Each cell's `DeepStrata` is independent → byte-identical
    /// parallel (per-cell record ops, disjoint records).
    pub fn record(&mut self, grid: &mut DeepGrid, mem: MemberCtx<'_>) {
        let parallel = self.par();
        let sea = self.sea_level;
        let chapter = self.cur_chapter;
        // **The formation context of this geological day** (P11 slice 1). Lifted
        // before the record is borrowed mutably: `lat_deg` takes `&self`, and the
        // per-cell temperature is the same air-temperature model the biotic gate
        // and the frost agent read (one climate, never a second).
        let w = grid.w;
        let lat: Vec<f64> = (0..w).map(|gy| grid.lat_deg(gy)).collect();
        let (r, h, precip, energy) = (&grid.r, &grid.h, &grid.precip, &self.energy);
        // The reference the energy bands are relative to — the coefficient the
        // capacities in `energy` were actually produced with, carried from
        // `transport` rather than re-read from a config this phase does not take.
        let k_t = self.k_transport;
        let dh = &self.dh;
        let dep = &self.dep_sp;
        let creep = &self.creep_sp;
        let sorted = self.sorted;
        // **The two halves of a unit's identity, in order** (P11 slice 1):
        //
        // 1. the CLASS is what the movers determine — the argmax of everything
        //    that arrived (`arriving_species`), or the tag's own lithology where
        //    nothing rode a mover. That is still `Litho`-grade because the
        //    transport budgets are (slice 2 re-grades them);
        // 2. the MEMBER is what the environment determines — fitness over the
        //    registered members of that class, under this cell's temperature and
        //    precipitation *this epoch*, through an addressed draw.
        //
        // What used to happen instead: (1) was recorded and (2) was invented at
        // expression, hundreds of millions of sim-years later, from the climate of
        // the chunk's centre. The class is a fact about the load; the member is a
        // fact about the day. Both are known here and neither was written down.
        let species_at = |i: usize, tag: DepTag| {
            let t = lithology::litho_of_tag(tag);
            let class = if sorted {
                let cr = if creep.is_empty() {
                    &[][..]
                } else {
                    &creep[i * SPECIES..(i + 1) * SPECIES]
                };
                arriving_species(dh[i], &dep[i * SPECIES..(i + 1) * SPECIES], cr, t)
            } else {
                t
            };
            let temp_c = f64::from(climate::air_temp_c(lat[i / w], r[i] + h[i]));
            mem.surface(
                i,
                temp_c,
                f64::from(precip[i]),
                class,
                dep_tags::TRANSPORT,
                0,
            )
        };
        let deposit_at = |i: usize| {
            let tag = tag_of(r[i] + h[i], precip[i], energy[i], sea, k_t);
            (tag, species_at(i, tag))
        };
        if parallel {
            grid.strata.par_iter_mut().enumerate().for_each(|(i, s)| {
                record_cell(s, dh[i], chapter, || deposit_at(i));
            });
        } else {
            for (i, s) in grid.strata.iter_mut().enumerate().take(self.n) {
                record_cell(s, dh[i], chapter, || deposit_at(i));
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
    pub fn wind(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, mem: MemberCtx<'_>) {
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
                        // journal/0111: dust that reaches the sea has left the land.
                        if self.denude {
                            self.ledger.eolian_to_sea_m += load;
                        }
                        grid.h[i] += load;
                        if record {
                            let tag = DepTag {
                                eolian: Eolian::Loess,
                                ..DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low)
                            };
                            // Dust settling on the sea: the wind carries no
                            // identity of its own (§ 13.2's wind member is
                            // deferred), so the class is the tag's — but the
                            // MEMBER is picked by fitness here, today.
                            let m = mem.surface(
                                i,
                                f64::from(climate::air_temp_c(grid.lat_deg(gy), surf)),
                                f64::from(grid.precip[i]),
                                lithology::litho_of_tag(tag),
                                dep_tags::EOLIAN,
                                0,
                            );
                            grid.strata[i].deposit_as(tag, load, chapter, m);
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
                        let m = mem.surface(
                            i,
                            f64::from(climate::air_temp_c(grid.lat_deg(gy), surf)),
                            precip,
                            lithology::litho_of_tag(tag),
                            dep_tags::EOLIAN,
                            1,
                        );
                        grid.strata[i].deposit_as(tag, drop, chapter, m);
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
                    let m = mem.surface(
                        i,
                        f64::from(climate::air_temp_c(grid.lat_deg(gy), grid.r[i] + grid.h[i])),
                        f64::from(grid.precip[i]),
                        lithology::litho_of_tag(tag),
                        dep_tags::EOLIAN,
                        2,
                    );
                    grid.strata[i].deposit_as(tag, load, chapter, m);
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
    pub fn wave(&mut self, grid: &mut DeepGrid, cfg: &DeepConfig, mem: MemberCtx<'_>) {
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
            // journal/0111: `j` is a subsea cell by construction, so every metre
            // of this is export from the land system — and the bedrock share is
            // exhumation `track_exhumation` cannot see, because wave runs after it.
            if self.denude {
                self.ledger.wave_offshore_m += cut;
                self.ledger.wave_bedrock_m += cut - removed_h;
            }
            if record {
                if removed_h > 0.0 {
                    grid.strata[i].erode(removed_h);
                }
                let tag = DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low);
                let m = mem.surface(
                    j,
                    // `j`'s own row: the bed is laid where the sediment lands.
                    f64::from(climate::air_temp_c(
                        grid.lat_deg(j / w),
                        grid.r[j] + grid.h[j],
                    )),
                    f64::from(grid.precip[j]),
                    lithology::litho_of_tag(tag),
                    dep_tags::WAVE,
                    0,
                );
                grid.strata[j].deposit_as(tag, cut, chapter, m);
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
            let best = partition_cell(
                c,
                3,
                &surf,
                &filled,
                -1000.0,
                0.0,
                1.0,
                &MfdParams::uniform(p),
                &mut w_out,
            );
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

    /// **`p → ∞` is a SINGLE-RECEIVER limit.** That property is what makes the
    /// exponent a *convergence knob* rather than a different model: the partition
    /// contains the thing it replaces as a limit, so "how much does MFD change the
    /// world" has a continuous answer instead of a discrete one.
    ///
    /// > **Not, however, `route_cell` — corrections #58.** journal/0109 and
    /// > flow.md § 2.6.1 both say *"`p → ∞` is single-receiver D8 **exactly**"*, and
    /// > it is not: the partition's limit is the steepest **slope** and `route_cell`
    /// > takes the steepest **drop**, which differ on diagonals by the very `√2`
    /// > path length that slice introduced. `the_partition_follows_slope_not_drop`
    /// > below pins a case where the two pick *different* receivers. The limit is
    /// > the physically correct one; it is the claim of identity that is wrong.
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
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(16.0),
            &mut w_out,
        );
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
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
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
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(1.0),
            &mut w_out,
        );
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
        let best = partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
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
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut w_out,
        );
        let sum: f64 = w_out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "sum {sum} after the floor");
        assert!(
            w_out.iter().all(|&v| v == 0.0 || v >= MFD_MIN_WEIGHT),
            "a sub-floor weight survived: {w_out:?}"
        );
    }

    // ---- hybrid `p` (journal/0113) ----------------------------------------

    /// **The ramp is monotone non-decreasing in the channelisation index**, and
    /// hits its endpoints. This is the invariant that would catch a law wired
    /// backwards — dispersing channels and concentrating hillslopes — which no
    /// absolute count of anything could see, because both directions produce
    /// plausible-looking numbers.
    #[test]
    fn the_exponent_ramps_monotonically_from_hillslope_to_channel() {
        let mp = MfdParams::default();
        assert_eq!(mp.exponent_at(0.0), mp.p_hill);
        assert_eq!(mp.exponent_at(mp.chi_lo), mp.p_hill);
        assert_eq!(mp.exponent_at(mp.chi_hi), mp.p_chan);
        assert_eq!(mp.exponent_at(1.0e9), mp.p_chan);
        let mut prev = mp.exponent_at(0.0);
        for k in 0..400 {
            // Sweep χ across six decades either side of the ramp.
            let chi = 10f64.powf(-7.0 + 6.0 * f64::from(k) / 400.0);
            let p = mp.exponent_at(chi);
            assert!(p >= prev, "exponent fell from {prev} to {p} at chi={chi}");
            assert!((mp.p_hill..=mp.p_chan).contains(&p));
            prev = p;
        }
    }

    /// **Uniform mode is the ramp's degenerate case and evaluates no ramp at all**
    /// — `p_chan == p_hill` must return that exponent for every `χ`, including
    /// fractional exponents a probe may sweep (which must NOT be rounded).
    #[test]
    fn a_uniform_law_ignores_the_channelisation_index() {
        for p in [1.0f64, 1.5, 4.0, 7.25] {
            let mp = MfdParams::uniform(p);
            for chi in [0.0, 1e-9, 1e-3, 1.0, 1e6] {
                assert_eq!(mp.exponent_at(chi), p, "uniform p={p} moved at chi={chi}");
            }
        }
    }

    /// **The same cell, routed two ways by its drainage area alone.** One patch,
    /// one slope field — a moderately convergent junction with a clear steepest
    /// line and two near-rivals. Given a hillslope's area it must spread; given a
    /// trunk river's area it must not. That *is* the slice, isolated from the
    /// world: the partition now depends on how channelised the flow is, and on
    /// nothing else that changed.
    #[test]
    fn the_same_slope_field_disperses_on_a_hillslope_and_concentrates_in_a_channel() {
        let mp = MfdParams::default();
        let (c, surf, filled) = patch([1.0, 2.0, 0.5, 2.6, 2.2, 0.25, 1.5, 1.0]);
        // `cell_m = 100` makes the steepest gradient `2.6/100 = 0.026`, a
        // production-like value; `χ = A · 0.026²`, so `A = 1` is a hillslope
        // (χ = 6.8e-4, below `chi_lo`) and `A = 4000` is a channel (χ = 2.7,
        // above `chi_hi`). One slope field, two drainage areas.
        let mut hill = [0.0f64; MFD_DIRS];
        let mut chan = [0.0f64; MFD_DIRS];
        partition_cell(c, 3, &surf, &filled, -1000.0, 1.0, 100.0, &mp, &mut hill);
        partition_cell(c, 3, &surf, &filled, -1000.0, 4000.0, 100.0, &mp, &mut chan);
        let n_hill = hill.iter().filter(|&&v| v > 0.0).count();
        let n_chan = chan.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(n_chan, 1, "a channelised cell must route single-receiver");
        assert!(
            n_hill > n_chan,
            "hillslope kept {n_hill} receivers, channel kept {n_chan} — the law did not vary"
        );
        let top_hill = hill.iter().cloned().fold(0.0f64, f64::max);
        let top_chan = chan.iter().cloned().fold(0.0f64, f64::max);
        assert!(
            top_chan > top_hill,
            "the channel's steepest share ({top_chan}) is no larger than the hillslope's \
             ({top_hill})"
        );
        // Both are still probabilities — the ramp may not leak discharge.
        for w in [&hill, &chan] {
            let sum: f64 = w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
        }
    }

    /// **A large channel exponent must never turn a draining cell into a sink.**
    /// This is what the `S/S_max` normalisation buys: without it the raw weights
    /// are `S^p`, and on the gentle gradients this world actually has
    /// (`S ~ 1e-5` per cell width in places) a `p` of 64 underflows *every*
    /// direction to zero, the partition sums to zero, and the cell is reported as
    /// a sink — silently disconnecting a drainage network. Scale-free: it is a
    /// statement about floating-point range, not about grid size.
    #[test]
    fn a_steep_exponent_on_a_gentle_slope_still_finds_a_receiver() {
        let (c, surf, filled) = patch([1e-5, 2e-5, 5e-6, 3e-5, 9e-6, 2.5e-6, 1.5e-5, 4e-5]);
        // A **hybrid** law: the normalisation is deliberately not applied to a
        // uniform one (see `MfdParams::uniform`), so this pins the guarantee where
        // it exists. `chi` is far below `chi_lo`, so the exponent is `p_hill`.
        let mp = MfdParams {
            p_hill: 63.0,
            p_chan: 64.0,
            ..MfdParams::default()
        };
        let mut w_out = [0.0f64; MFD_DIRS];
        let best = partition_cell(c, 3, &surf, &filled, -1000.0, 0.0, 1.0, &mp, &mut w_out);
        assert!(
            best >= 0,
            "a cell with eight downslope neighbours became a sink"
        );
        let sum: f64 = w_out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
    }

    /// **The floor is a knob now, and `0.0` disables it** — the measurement stub #22
    /// was owed. With the floor off, every downslope neighbour keeps its share no
    /// matter how small; with it on, the sub-floor ones are dropped and the rest
    /// renormalised. Both still sum to one.
    #[test]
    fn a_zero_floor_keeps_every_downslope_neighbour() {
        let (c, surf, filled) = patch([-1.0, -1.0, -1.0, 10.0, 0.3, -1.0, -1.0, -1.0]);
        let mut floored = [0.0f64; MFD_DIRS];
        let mut open = [0.0f64; MFD_DIRS];
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams::uniform(4.0),
            &mut floored,
        );
        partition_cell(
            c,
            3,
            &surf,
            &filled,
            -1000.0,
            0.0,
            1.0,
            &MfdParams {
                min_weight: 0.0,
                ..MfdParams::uniform(4.0)
            },
            &mut open,
        );
        assert_eq!(floored.iter().filter(|&&v| v > 0.0).count(), 1);
        assert_eq!(open.iter().filter(|&&v| v > 0.0).count(), 2);
        for w in [&floored, &open] {
            let sum: f64 = w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "weights sum to {sum}");
        }
    }
}

// ---------------------------------------------------------------------------
// Movement 2b continuation (b) — the creep split. Its own module so the two
// slices' unit tests do not share a namespace.

#[cfg(test)]
mod creep_tests {
    use super::*;

    /// **The split closes exactly.** `Σ_species` of a composition split is the
    /// quantity that went in, to the bit — not to an epsilon. That is the whole
    /// content of the residual rule, and it is what makes the itemisation audit's
    /// tolerance a statement about *summation order* rather than about a leak we
    /// decided to tolerate.
    #[test]
    fn a_composition_split_closes_to_the_bit() {
        // Shares that do not sum to 1 in binary — the normal case for a
        // normalised f64 composition, and the reason the rule exists.
        let mut shares = [0.0; SPECIES];
        for (k, v) in [0.17, 0.03, 0.31, 0.0, 0.29, 0.11, 0.09]
            .into_iter()
            .enumerate()
        {
            shares[k] = v;
        }
        for total in [1.0f64, 3.7e-5, 2.4e3, 9.81e-12, 0.0] {
            let out = split_by_shares(total, &shares);
            let sum: f64 = out.iter().sum();
            assert_eq!(
                sum, total,
                "the split of {total} summed to {sum}; the last non-zero share is \
                 not taking the residual"
            );
        }
    }

    /// **A species with a zero share receives nothing**, whatever the residual
    /// rule does. A composition that says "there is no basement here" must not
    /// have basement fall out of the arithmetic at the end.
    #[test]
    fn an_absent_species_stays_absent() {
        let mut shares = [0.0; SPECIES];
        shares[1] = 0.5;
        shares[4] = 0.5;
        let out = split_by_shares(7.0, &shares);
        for (k, v) in out.iter().enumerate() {
            if shares[k] == 0.0 {
                assert_eq!(*v, 0.0, "species {k} materialised out of a zero share");
            }
        }
    }

    /// **An all-zero composition splits into nothing** — the honest answer when a
    /// cell has no identity to give. The alternative (spreading the quantity
    /// evenly, or handing it to species 0) would be fabricating provenance, which
    /// is strictly worse than recording none.
    #[test]
    fn an_unknown_composition_fabricates_no_identity() {
        let out = split_by_shares(42.0, &[0.0; SPECIES]);
        assert!(out.iter().all(|v| *v == 0.0), "{out:?}");
    }

    /// **CREEP DOES NOT SORT — and this is the test that would catch it starting
    /// to.**
    ///
    /// Colluvium is poorly sorted because gravity is not selective: a diffusive
    /// flux moves the donor's whole composition in proportion, with no competence
    /// ceiling and no settling draw. So the *only* thing that may decide how a
    /// creep flux resolves into species is the composition — never the settling
    /// velocity, never the grain size, never the quantity.
    ///
    /// Asserted as **scale invariance**: doubling the flux must double every
    /// species' share, exactly. A competence ceiling or a coarsest-first draw is by
    /// construction non-linear in the quantity (it thresholds, or it drains one
    /// species before touching the next), so any sorting rule that crept into this
    /// path would break this equality. It is arithmetic and therefore scale-free.
    #[test]
    fn the_creep_split_is_linear_in_the_quantity_so_nothing_is_sorted() {
        let mut shares = [0.0; SPECIES];
        for (k, v) in [0.4, 0.0, 0.05, 0.25, 0.2, 0.0, 0.1]
            .into_iter()
            .enumerate()
        {
            shares[k] = v;
        }
        let one = split_by_shares(1.0, &shares);
        let many = split_by_shares(1024.0, &shares);
        for k in 0..SPECIES {
            assert_eq!(
                many[k],
                one[k] * 1024.0,
                "species {k} did not scale with the flux — something in the creep \
                 path is selecting by grain size"
            );
        }
    }

    /// **The two endpoints of a creep edge see the same flux, to the bit.** The
    /// whole per-species conservation argument rests on IEEE-754 subtraction being
    /// exactly antisymmetric, so the donor's `sᵢ − sⱼ` and the receiver's
    /// `−(sⱼ − sᵢ)` are the same number and split the same way. If that ever
    /// stopped holding, every species would leak at every junction.
    #[test]
    fn a_surface_difference_is_exactly_antisymmetric() {
        for (a, b) in [
            (1234.5678901234, 1234.5678901233),
            (1e-300, 3e-300),
            (0.1, 0.2),
            (1e17, 1.0),
            (-4321.9, 8765.1),
        ] {
            assert_eq!(a - b, -(b - a), "({a}, {b}) broke the antisymmetry");
        }
    }
}

#[cfg(test)]
mod hillslope_operator_tests {
    use super::*;

    /// A bare grid with flat bedrock and a **checkerboard regolith cover** — the
    /// smallest world on which the hypothesis journal/0116 refused to promote can
    /// be settled. Nothing here is stochastic and nothing is seeded: the whole
    /// experiment is arithmetic on one initial condition.
    fn checkerboard(w: usize, cover: f64) -> DeepGrid {
        let n = w * w;
        let mut grid = DeepGrid::from_parts(w, 460.0, vec![0.0; n], vec![0.0; n]);
        for i in 0..n {
            let (x, y) = (i % w, i / w);
            grid.h[i] = if (x + y) % 2 == 0 { cover } else { 0.0 };
        }
        grid
    }

    /// A config that carries **nothing but the diffusion coefficient** — every
    /// other phase is irrelevant because the test calls `diffuse` directly.
    fn creep_cfg(diffusion: f64) -> DeepConfig {
        DeepConfig {
            diffusion,
            erodibility: false,
            biotic: false,
            ..DeepConfig::default()
        }
    }

    /// The amplitude of the grid-scale mode over a window `margin` cells in from
    /// the border, signed by the parity that started high. Positive ⇒ still in
    /// phase with the initial condition; negative ⇒ **flipped**.
    ///
    /// **The margin is load-bearing, not tidiness.** A border cell has three
    /// neighbours (a corner two), so its amplification factor is not the interior
    /// one and its deviation contaminates any window it is in. That contamination
    /// spreads inward exactly one cell per step, so a window `margin` cells in is
    /// clean for `margin` steps — which is how many every test below takes. An
    /// earlier draft of this helper measured the whole interior and reported a
    /// *sign flip* on the shipped arm, which is the defect these tests exist to
    /// detect: the instrument was manufacturing the signature.
    const MARGIN: usize = 8;
    fn signed_amplitude(grid: &DeepGrid) -> f64 {
        let w = grid.w;
        let (lo, hi) = (MARGIN, w - MARGIN);
        // The window is even in both axes, so it holds equal counts of the two
        // parities and the window mean of an undisturbed checkerboard is exactly
        // `cover / 2` — no bias enters through the reference.
        debug_assert_eq!((hi - lo) % 2, 0);
        let mut sum = 0.0;
        let mut n = 0usize;
        for y in lo..hi {
            for x in lo..hi {
                sum += grid.h[y * w + x];
                n += 1;
            }
        }
        let mean = sum / n as f64;
        let mut acc = 0.0;
        for y in lo..hi {
            for x in lo..hi {
                let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
                acc += sign * (grid.h[y * w + x] - mean);
            }
        }
        acc / n as f64
    }

    /// **`dt` IS A TRUE MULTIPLIER OF THE RATE, TO THE BIT** (RATE, journal/0123).
    ///
    /// The claim the axis rests on is that a phase length is *time*: half the rate
    /// for twice as long is the same amount of creep. Asserted as **bit equality**
    /// rather than a tolerance, because it is not an approximation — `rate =
    /// cfg.diffusion × dt` is one multiply, and doubling is exact in binary
    /// floating point, so `0.12 × 2.0` and `0.24 × 1.0` are the same `f64`. A
    /// tolerance here would hide the interesting failure: the operator quietly
    /// dividing by `dt` somewhere else as well.
    ///
    /// Scale-free: a per-cell arithmetic identity, with no length in it.
    #[test]
    fn dt_scales_the_creep_rate_and_zero_stops_it() {
        let cover = 4.0;
        // Same product, stated two ways: an authored phase length of 2 epochs at
        // half the per-epoch rate, against one epoch at the whole rate.
        let mut slow = checkerboard(24, cover);
        let mut long_step = Erosion::new(&slow);
        long_step.diffuse(&mut slow, &creep_cfg(0.12), 2.0);

        let mut fast = checkerboard(24, cover);
        let mut one_epoch = Erosion::new(&fast);
        one_epoch.diffuse(&mut fast, &creep_cfg(0.24), 1.0);

        for (i, (a, b)) in slow.h.iter().zip(fast.h.iter()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "cell {i}: half the rate for twice the phase length must be the \
                 same creep ({a} != {b}) — `dt` is not scaling the rate"
            );
        }
        // And the sub-cycle count follows the *scaled* rate, not the config one:
        // a longer phase is a bigger step and needs finer integration, which is
        // the whole reason the two divisions have to compose.
        assert_eq!(long_step.creep_substeps(), one_epoch.creep_substeps());
        assert_eq!(
            long_step.creep_peak_coeff().to_bits(),
            one_epoch.creep_peak_coeff().to_bits()
        );

        // A zero-length phase moves nothing — the degenerate case that proves the
        // pass is reading `dt` at all rather than ignoring it in the common path.
        let mut still = checkerboard(24, cover);
        let before = still.h.clone();
        Erosion::new(&still).diffuse(&mut still, &creep_cfg(0.24), 0.0);
        assert_eq!(still.h, before, "a zero-length phase eroded something");
    }

    /// **The forcing passes are rate-shaped too**, and their `dt` scaling is exact
    /// for the same reason: one multiply against a per-epoch plane. The ledger
    /// total must scale with the grid, or the mass-conservation falsifier would be
    /// accounting for a different amount of time than the world got.
    #[test]
    fn dt_scales_the_uplift_plane_and_its_ledger_together() {
        // A grid with a non-uniform uplift plane and a flat bedrock datum.
        fn uplifting(w: usize) -> DeepGrid {
            let mut g = checkerboard(w, 0.0);
            g.uplift = (0..w * w).map(|i| 0.25 + i as f64 * 0.001).collect();
            g
        }
        let w = 16;
        let plane = uplifting(w).uplift;
        let expect: f64 = plane.iter().sum();

        let mut doubled = uplifting(w);
        let mut er = Erosion::new(&doubled);
        let ledger = er.apply_uplift(&mut doubled, 2.0);
        for (i, u) in plane.iter().enumerate() {
            assert_eq!(
                doubled.r[i].to_bits(),
                (*u * 2.0).to_bits(),
                "cell {i}: uplift did not scale by the phase length"
            );
        }
        assert_eq!(
            ledger.to_bits(),
            (expect * 2.0).to_bits(),
            "the ledger must account for the same phase length the grid got"
        );

        let mut zero = uplifting(w);
        let mut er = Erosion::new(&zero);
        assert_eq!(er.apply_uplift(&mut zero, 0.0), 0.0);
        assert!(
            zero.r.iter().all(|r| *r == 0.0),
            "a zero-length phase uplifted something"
        );
    }

    /// **THE HYPOTHESIS, ISOLATED.** journal/0116 wrote it and explicitly declined
    /// to promote it: *"a donor-cell scheme that moves everything downslope has a
    /// period-2 mode by construction — A gives all its cover to B, B is now higher
    /// and gives it back."* Every number in that entry was *consistent* with it and
    /// none of them isolated it, because every number came from a world in which
    /// eight other passes were also running.
    ///
    /// This is the isolation. Flat bedrock, a checkerboard of cover, the creep
    /// pass and nothing else, at the **calibrated** rate (`0.12 × 45`). The
    /// pre-journal/0122 operator's arithmetic is exact and can be read off by
    /// hand: a high cell's requested outflux is `4 × 5.4 × cover`, i.e. `21.6×` its
    /// own inventory, so the limiter binds and it ships **all** of it, `cover/4`
    /// per edge; each low cell has four high neighbours and therefore receives
    /// exactly `cover`. **The two populations swap, to the bit, forever.**
    ///
    /// Run under the sub-cycled operator the same initial condition decays
    /// monotonically and never changes sign — which is the other half of the
    /// claim: it is the *coefficient*, not the geometry, that made a checkerboard
    /// a fixed point of the pass.
    ///
    /// Scale-free: the argument is a two-population recurrence with no length in
    /// it, so a 12×12 grid tests it as completely as a 288×288 one.
    #[test]
    fn the_calibrated_rate_has_an_exact_period_2_mode_and_the_bound_removes_it() {
        let cover = 40.0;
        let calibrated = 0.12 * 45.0;
        let cfg = creep_cfg(calibrated);

        // --- the operator as it stood: one raw step at the stated rate ---------
        let mut grid = checkerboard(32, cover);
        let mut er = Erosion::new(&grid);
        let a0 = signed_amplitude(&grid);
        let mut legacy = Vec::new();
        for _ in 0..6 {
            er.diffuse_step(&mut grid, &cfg, calibrated, false);
            legacy.push(signed_amplitude(&grid));
        }
        // Period 2: strict sign alternation with the amplitude *conserved* rather
        // than decaying. Both halves matter — a decaying alternation would just be
        // a stable scheme resolving a rough initial condition.
        for (k, a) in legacy.iter().enumerate() {
            assert_eq!(
                a.is_sign_negative(),
                k % 2 == 0,
                "step {k}: the legacy operator did not alternate (amplitudes {legacy:?})"
            );
            assert!(
                (a.abs() - a0.abs()).abs() < 1e-9,
                "step {k}: amplitude {a} is not the initial {a0} — inside the clean \
                 window the flip-flop is exactly amplitude-preserving, which is what \
                 makes it a *mode* rather than a transient"
            );
        }

        // --- the sub-cycled operator, same rate, same initial condition --------
        let mut grid = checkerboard(32, cover);
        let mut er = Erosion::new(&grid);
        er.diffuse(&mut grid, &cfg, 1.0);
        assert_eq!(
            er.creep_substeps(),
            (calibrated / CREEP_MAX_EDGE_COEFF).ceil() as u32,
            "the sub-cycle count must follow from the rate, not from a table"
        );
        // 44 sub-steps at a per-edge coefficient of 5.4/44 = 0.1227 give the
        // grid-scale mode an amplification of `(1 − 8a)^44 = 0.0182^44`, i.e. it is
        // annihilated inside the first epoch rather than flipped. What the loop
        // then asserts is that it **stays** annihilated and never changes sign —
        // the residual is border relaxation diffusing inward, six orders of
        // magnitude below the mode it replaced.
        for step in 0..6 {
            let a = signed_amplitude(&grid);
            assert!(
                a > -a0 * 1e-6,
                "step {step}: the sub-cycled operator flipped the grid-scale mode \
                 ({a} against an initial {a0}) — the monotonicity bound is not holding"
            );
            assert!(
                a.abs() < a0 * 1e-3,
                "step {step}: the checkerboard is still here, {a} of {a0}"
            );
            er.diffuse(&mut grid, &cfg, 1.0);
        }
    }

    /// **And the shipped rate was never oscillating** — the fact that makes
    /// journal/0116's *"saturation alone is not sufficient"* qualification and this
    /// operator's bound the same statement. At `diffusion = 0.12` the limiter still
    /// binds (a cell with 4.6 m of cover on a steep edge wants to shed more than it
    /// has), but `0.12 < 1/8`, so the mode decays instead of flipping and the pass
    /// takes exactly one sub-step.
    #[test]
    fn the_shipped_rate_sits_inside_the_bound_and_decays() {
        let shipped = 0.12;
        assert!(shipped < CREEP_MAX_EDGE_COEFF);
        let cfg = creep_cfg(shipped);
        let mut grid = checkerboard(32, 4.6);
        let mut er = Erosion::new(&grid);
        let mut prev = signed_amplitude(&grid);
        for step in 0..8 {
            er.diffuse(&mut grid, &cfg, 1.0);
            assert_eq!(
                er.creep_substeps(),
                1,
                "step {step} sub-cycled at the shipped rate"
            );
            let a = signed_amplitude(&grid);
            assert!(
                a >= 0.0 && a < prev,
                "step {step}: {prev} → {a} is not a decay"
            );
            prev = a;
        }
    }

    /// **Mass is exact and nothing goes negative**, at any rate, sub-cycled or
    /// not. Non-negotiable, and scale-free: a per-cell predicate and a grid-wide
    /// sum, neither of which knows how big the world is.
    #[test]
    fn the_operator_conserves_mass_exactly_and_never_goes_negative() {
        for rate in [0.12, 1.0, 5.4, 40.0] {
            let cfg = creep_cfg(rate);
            // Rough bedrock the cover has to chase, so the limiter is genuinely
            // engaged rather than the flat case conserving mass trivially.
            let w = 16;
            let n = w * w;
            let r: Vec<f64> = (0..n)
                .map(|i| {
                    let (x, y) = ((i % w) as f64, (i / w) as f64);
                    100.0 - 3.0 * x + 7.0 * ((x * 0.7).sin() + (y * 1.3).cos())
                })
                .collect();
            let mut grid = DeepGrid::from_parts(w, 460.0, r, vec![0.0; n]);
            for i in 0..n {
                grid.h[i] = if i % 3 == 0 { 30.0 } else { 2.0 };
            }
            let before: f64 = grid.h.iter().sum();
            let mut er = Erosion::new(&grid);
            for _ in 0..20 {
                er.diffuse(&mut grid, &cfg, 1.0);
                assert!(
                    grid.h.iter().all(|v| *v >= -1e-9),
                    "rate {rate}: regolith went negative"
                );
            }
            let after: f64 = grid.h.iter().sum();
            assert!(
                (after - before).abs() / before < 1e-12,
                "rate {rate}: ΣH moved {before} → {after}"
            );
        }
    }

    /// **Order independence** — the property `water/sat.rs` gets "by construction"
    /// and this pass gets the same way: every flux is a pure function of a frozen
    /// surface and a frozen per-cell limiter, so the scalar and the data-parallel
    /// drivers must agree **bit for bit**, including across sub-steps, where a
    /// mistake in the accumulation would show up as a drift rather than as a
    /// wrong answer.
    #[test]
    fn the_sub_cycled_operator_is_bit_identical_scalar_and_parallel() {
        let cfg = creep_cfg(5.4);
        let mut a = checkerboard(64, 40.0);
        let mut b = checkerboard(64, 40.0);
        let mut ea = Erosion::new(&a);
        let mut eb = Erosion::new(&b);
        ea.set_parallel(false);
        eb.set_parallel(true);
        for _ in 0..4 {
            ea.diffuse(&mut a, &cfg, 1.0);
            eb.diffuse(&mut b, &cfg, 1.0);
        }
        assert_eq!(a.h, b.h, "scalar and parallel sub-cycling disagree");
    }

    /// **`n = 1` is the old operator, bit for bit** — `x / 1.0 == x`, asserted
    /// rather than argued. (This once read "the shipped world is untouched";
    /// that claim retired when `creep_substep` went default-ON with 2 sub-steps
    /// and the goldens moved. The predicate this test pins is unchanged —
    /// spine-audit residue fix, 2026-07-29.)
    #[test]
    fn inside_the_bound_the_driver_is_the_raw_step() {
        let cfg = creep_cfg(0.12);
        let mut a = checkerboard(32, 4.6);
        let mut b = checkerboard(32, 4.6);
        let mut ea = Erosion::new(&a);
        let mut eb = Erosion::new(&b);
        for _ in 0..5 {
            ea.diffuse(&mut a, &cfg, 1.0);
            eb.diffuse_step(&mut b, &cfg, 0.12, false);
        }
        assert_eq!(a.h, b.h);
    }
}
