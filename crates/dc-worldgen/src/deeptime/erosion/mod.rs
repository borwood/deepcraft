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
//!
//! # The module map
//!
//! This module was a single 4,712-line file until 2026-08-02, when it was
//! re-housed — a **pure move**, no behaviour change — onto ordinary module
//! boundaries, one per concern. Nothing was renamed and no arithmetic moved;
//! every item is where it was, in the file that names its concern:
//!
//! - `mfd` / `mfd_law` — the flow-direction laws: the D8 receiver rule and the
//!   MFD weight partition, and the hybrid-`p` convergence law that drives it.
//! - `routing` — the routing phase: receiver/weight planes, drainage-area
//!   accumulation, processing order, out-edges.
//! - `flood` — priority-flood depression filling (and the S9b parallel probe).
//! - `transport` — the mass-routing transport chain: entrainment, capacity,
//!   competence, deposition, the species split.
//! - `creep_kernel` / `creep` — the hillslope-creep gather kernels and the
//!   explicit step over them, and the pass that sub-cycles it to its bound.
//! - `weathering` — bedrock-to-regolith conversion, outcrop exposure, frost.
//! - `record` — the strata recorder phase and the facies tagging it needs.
//! - `agents` — the mass-neutral surface agents (wind, wave), which self-record.
//! - `uplift` — the vertical drivers: uplift, thickening, isostasy, exhumation.
//! - `ledger` — the denudation ledger and the flux-record exports.
//!
//! **Which side of the engine/pack partition this sits on** (north star): all of
//! it is **pass/content logic — plugin side by destination**. The one
//! engine-shaped thing embedded here is the field-solver gather in
//! `creep_kernel` (spines § S-10); extracting it is the deferred E4 arc and this
//! move does not attempt it.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::grid::{DeepConfig, DeepGrid, SEA_LEVEL_M};
use super::recorder::MemberCtx;
use super::species::{SpeciesAxis, SpeciesLayout, SpeciesPlane};

mod agents;
mod creep;
mod creep_kernel;
mod flood;
mod ledger;
mod mfd;
mod mfd_law;
mod record;
mod routing;
mod transport;
mod uplift;
mod weathering;

use flood::Item;

pub use creep_kernel::CREEP_MAX_EDGE_COEFF;
pub use flood::{flood_fill_serial, flood_fill_tiled};
pub use ledger::TransportLedger;
pub use mfd::MFD_MIN_WEIGHT;
pub use mfd_law::MfdParams;
pub use record::energy_band;
pub use transport::{ANCHOR_MATERIAL, competence_ceiling};

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

// **`const SPECIES: usize = Litho::COUNT` LIVED HERE AND IS GONE** (P11 slice 2).
//
// It said the load could be resolved no finer than the seven classes, because
// *"members within a class are chosen at collapse time, so resolving the load any
// finer than this would be inventing identity the tier does not have."* Slice 1
// gave the tier that identity — `DepUnit::species` is a registry `MaterialId` —
// and the sentence expired with it. The four budget planes are now CSR-sparse over
// a content-derived [`SpeciesAxis`] (`super::super::species`), so the width is a
// property of the registered content and of each cell's own catchment, never a
// constant in this file.

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
    /// **The alphabet the load, the record and the erosion tables are resolved
    /// against** — the registered content's materials, ordered by descending
    /// settling energy (P11 slice 2, `super::super::species`).
    ///
    /// Empty when no content set was supplied ([`SpeciesAxis::empty`], the
    /// degenerate door), in which case every member-grade consumer falls through to
    /// the class-grade answer it gave before this slice.
    ///
    /// **The axis order is load-bearing, not cosmetic.** Two loops in the transport
    /// pass are order-sensitive — the capacity drawdown (*coarsest first*) and the
    /// competence ceiling — and under a sparse row a permutation of the whole axis
    /// walked with a membership test would put the registry's width straight back
    /// into the hot loop. A CSR row is stored ascending in axis order, which **is**
    /// coarsest-first, so both loops walk the row and stop early. It also gives the
    /// residual split's *"last non-zero share takes the remainder"* and the
    /// arriving-identity argmax's tie-break one total, deterministic,
    /// content-derived order instead of an enum's declaration order.
    axis: SpeciesAxis,
    /// The composition of the material **below the record** — what incision
    /// detaches — as the axis codes and values of a sparse row (today: one entry,
    /// the basement, at share `1.0`).
    ///
    /// It is what the near-surface window walk answers for an **empty section**: a
    /// window containing no recorded units is entirely whatever lies beneath the
    /// pile. So the pass names no lithology; it asks the same question every other
    /// consumer asks and takes the answer. (Refreshed each epoch in
    /// [`Self::expose`], because the window's heir — structural deformation — may
    /// one day answer it differently per cell, at which point this becomes a plane
    /// rather than a constant.)
    bedrock_axis: Vec<u8>,
    bedrock_sp: Vec<f64>,
    /// Reused buffer for the transport closure's seed masks (window | basement),
    /// so the per-epoch rebuild allocates nothing.
    bedrock_seed_scratch: Vec<u64>,
    /// **Per-cell presence masks of the near-surface window** — bit `k` = axis
    /// index `k`. The seed every layout below is built from, written by
    /// [`Self::expose`] in the same walk that computes the window shares.
    masks: Vec<u64>,
    /// **The window layout** — what lies *at* each cell. [`Self::shares`] rides it.
    window: SpeciesLayout,
    /// **The transport layout** — the window (plus the bedrock incision can reach)
    /// closed downstream over the solve's own routing, rebuilt each epoch in
    /// [`Self::transport`] because both of its inputs move each epoch. It is exact,
    /// not a heuristic: a species can never arrive at a cell whose row has no slot
    /// for it, because the propagation *is* the transport graph's reachability
    /// computed with `u64` ORs instead of `f64` adds.
    tlayout: SpeciesLayout,
    /// **The creep layout** — the window dilated by one 4-neighbourhood ring, since
    /// creep moves the **donor's** composition across an edge.
    clayout: SpeciesLayout,
    /// **The load itself** — metres of suspended material over [`Self::tlayout`],
    /// the multiset of § 13.3. Cell `c`'s row holds its *in*-load while upstream
    /// cells are still contributing, and is rewritten in place by
    /// [`Self::exchange_cell`] to hold its *out*-load; the chain is strictly
    /// downstream-ordered, so a cell's row is never read after it is spent.
    ///
    /// **This plane is the mass authority when it is non-empty.** The scalar
    /// `qs` plane is not maintained on the sorted path at all: `qin` is summed
    /// from here and the per-face scalar written to the flux record is the sum of
    /// the per-species shares — one arithmetic, so the record and the budget
    /// cannot drift (flow.md § 3).
    qs_sp: SpeciesPlane,
    /// The composition of the **surface loose** at each cell over
    /// [`Self::window`], from the record's own near-surface window. This is the
    /// identity entrainment removes: a reach cutting a sandstone bench hands the
    /// flow sand, and a stripped column hands it basement debris. Since slice 2 it
    /// is per *material*, so a siltstone bench and a mudstone bench are two
    /// different benches.
    shares: SpeciesPlane,
    /// What the transport pass **set down** at each cell this epoch, per species,
    /// over [`Self::tlayout`]. Read by [`Self::record`] to name the arriving unit,
    /// and by the outcome probe to measure the downstream fining gradient.
    dep_sp: SpeciesPlane,
    /// **Material-aware hillslope creep** (Movement 2b continuation (b)) — the
    /// gravity/mass-wasting member of § 13.2's transport family. Off ⇒
    /// [`Self::creep_sp`] is empty, [`Self::diffuse`] never runs its third pass,
    /// and the record names the arriving unit exactly as the fluvial-only slice
    /// did. Requires [`Self::sorted`], because the composition it moves is the
    /// same `outcrop_shares` plane the load entrains from — one walk, now three
    /// consumers.
    creep_carries: bool,
    /// The **net** metres of each species creep delivered to (or took from) each
    /// cell this epoch, over [`Self::clayout`]. Signed: a hillslope cell loses
    /// species it sheds and gains what came down from above.
    ///
    /// It is an **attribution, not a mass authority** — `grid.h` still moves by
    /// the scalar `netdiff`, bit for bit as before. Read by [`Self::record`] to
    /// name the arriving unit and by the outcome probe.
    creep_sp: SpeciesPlane,
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
            axis: SpeciesAxis::empty(),
            bedrock_axis: Vec::new(),
            bedrock_sp: Vec::new(),
            bedrock_seed_scratch: Vec::new(),
            masks: Vec::new(),
            window: SpeciesLayout::empty(),
            tlayout: SpeciesLayout::empty(),
            clayout: SpeciesLayout::empty(),
            qs_sp: SpeciesPlane::default(),
            shares: SpeciesPlane::default(),
            dep_sp: SpeciesPlane::default(),
            creep_carries: false,
            creep_sp: SpeciesPlane::default(),
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
            + self.creep_gross.len();
        f64s * 8
            + self.recv.len() * 4
            + self.order.capacity() * 4
            + self.done.len()
            + self.litho.len()
            + (self.out_area.len() + self.out_face_load.len()) * 4
            // The four budget planes and the three CSR row indices they ride
            // (P11 slice 2). Sparse, so this scales with each cell's own
            // catchment rather than with the registry.
            + self.species_bytes()
            + self.masks.len() * 8
    }

    /// **What the four CSR budget planes and their layouts cost right now**
    /// (bytes) — the number the residency probe reports against the dense
    /// member-grade comparand and the shipped class-grade one. Zero when
    /// material-aware transport is off.
    pub fn species_bytes(&self) -> usize {
        self.qs_sp.approx_bytes()
            + self.dep_sp.approx_bytes()
            + self.shares.approx_bytes()
            + self.creep_sp.approx_bytes()
            + self.window.approx_bytes()
            + self.tlayout.approx_bytes()
            + self.clayout.approx_bytes()
    }

    /// **The species axis this run resolves its load against** — empty until
    /// [`Self::set_species_axis`] supplies a content set.
    pub fn species_axis(&self) -> &SpeciesAxis {
        &self.axis
    }

    /// The three layouts, for the residency probe: `(window, transport, creep)`.
    pub fn species_layouts(&self) -> (&SpeciesLayout, &SpeciesLayout, &SpeciesLayout) {
        (&self.window, &self.tlayout, &self.clayout)
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
}
