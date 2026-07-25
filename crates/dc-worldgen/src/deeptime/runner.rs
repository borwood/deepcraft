//! The **deep-time pass-runner** — the epoch loop, re-housed as self-declaring
//! passes (Movement 1 of the deep-time-loop → declared-passes conversion,
//! journal/0090). It is the north-star pass-runner instantiated at the deep-time
//! tier: a pass **declares itself** (`{reads, writes}` over a deep-cell axis
//! vocabulary, plus its cadence), the runner **topo-sorts by the declared
//! reads/writes** (rejecting cycles / conflicting writers) and **drives the
//! epoch loop**, firing each pass at its cadence and handing it `dt` (the phase
//! length).
//!
//! It shares the graph math with the pregen [`crate::pipeline`] through the
//! [`crate::passgraph`] kernel — the same classification / Kahn's-algorithm /
//! ambiguity-rejection, one implementation, **not a second runner beside it**
//! (spines A-4). The difference is the execution model, which is genuinely
//! different and is what this file adds: `pipeline` runs a DAG **once** to build
//! world state from nothing; this runs a **loop** over state that persists
//! across epochs — so it calls the kernel in *loop mode* (`require_creator =
//! false`: every axis is already present from the pre-loop seed or the previous
//! turn), it has a **cadence/rate axis** (`pipeline` has none), and it carries
//! **loop-carried edges** (the biology↔erosion lag) that must not be read as
//! within-epoch cycles — declared as [`DeepPass::reads_prev`] and handed to the
//! kernel as reader-before-writer **anti-dependencies** (journal/0104).
//!
//! ## The two orthogonal axes (material-behavior.md §5 "order × rate")
//! - **ORDER** — the topo-sort of `{reads, writes}`. Reproduces exactly the
//!   hand-written phase order of the old loop, and would *reject* an illegal one.
//! - **RATE** — each pass's cadence [`DeepPass::period`] (epochs per firing) and
//!   the `dt` = phase length it is handed. This movement **pins** the rate: every
//!   pass keeps the effective cadence it had, and `dt` is threaded but inert (no
//!   transform scales by it yet — today's magnitudes are per-epoch constants), so
//!   the re-housing is byte-identical. `climate` is the one real coarse-rate pass
//!   already in the loop (it re-marches every `remarch_interval` epochs).
//!
//! ## Byte-identity
//! Each pass body is *literally the same call* the old loop made, in the same
//! order the topo-sort reproduces, with the same arguments — so the production
//! world hashes to the same goldens. This is a re-housing, not a rewrite.
//!
//! ## The crossing constraint (north-star)
//! A [`DeepPass`] is plain data + opaque ids (`&'static str`, `&[DeepAxis]`
//! slices) + a bare `fn` pointer body — **no closures cross the pass seam**. The
//! rich state lives on the runner's side in [`DeepStepCtx`]; the declaration a
//! future SDK / WASM backend would marshal is the plain-data part.

use super::biotic::BioticSim;
use super::erosion::Erosion;
use super::flux::FluxAccum;
use super::grid::{DeepConfig, DeepGrid, sea_level_at};
use super::inventory::FactLedger;
use super::{TectonicSchedule, climate};
use crate::passgraph::{self, Decl, GraphError};

/// The deep-cell resource axes a deep-time pass declares over. The **order axis**
/// of the runner: an edge exists wherever one pass writes what another reads.
///
/// The two-plane terrain (`R`+`H`) is a *pipeline* — nearly every erosion phase
/// reads the current terrain and the next transforms it, and several phases read
/// an **intermediate** terrain state (the recorder reads the post-isostasy
/// terrain, then the wind/wave agents transform it further). A single shared
/// "Terrain" axis cannot express that (the topo model orders a reader after *all*
/// writers), so each terrain-transforming stage exposes its output as a distinct
/// **revision** token the next stage consumes — `Forced → … → Weathered →
/// Diffused → Compensated → Windblown → Settled`. That is the honest declaration
/// of a fixed-order relaxation pipeline, and it is what forces the topo-sort to
/// reproduce the old loop's phase order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DeepAxis {
    /// Precip/temperature fields on the current topography. `climate` writes;
    /// the erosion pipeline (`forcing`, `deposition`, `eolian`) reads.
    Climate,
    /// The blended analytic tectonic thickening plane + chapter stamp.
    /// `tectonics` writes; `forcing` reads.
    Forcing,
    /// Terrain after the epoch's external forcing (uplift / crustal thickening).
    /// `forcing` writes; `drainage` + `frost` read.
    Forced,
    /// The per-cell outcropping-lithology susceptibility planes. `expose` writes;
    /// `transport` / `weather` / `diffuse` / `eolian` / `wave` read.
    Exposed,
    /// The periglacial frost-weathering multiplier plane. `frost` writes;
    /// `weather` + `biotic` read.
    Frosted,
    /// The drainage solve — filled surface, receivers, drainage area. `drainage`
    /// writes; `transport` reads (and `biotic` reads `area`).
    Routed,
    /// The stream transport-capacity (energy) plane. `transport` writes;
    /// `deposition` reads (for the facies band).
    Energy,
    /// The per-cell net thickness change this epoch. `transport` **creates** it
    /// (zeroed then filled); `weather` + `diffuse` add to it; `deposition` reads
    /// it. The creator→modifier→reader chain forces transport → weather/diffuse →
    /// deposition.
    DeltaH,
    /// Terrain after bedrock weathering. `weather` writes; `diffuse` reads.
    Weathered,
    /// Terrain after hillslope diffusion. `diffuse` writes; `isostasy` reads
    /// (tectonic) / `deposition` reads (legacy).
    Diffused,
    /// Crustal thickness `t_crust`. `forcing` (tectonic) writes; `isostasy` reads.
    CrustThick,
    /// Terrain after Airy isostasy. `isostasy` writes; `deposition` + `eolian`
    /// read (tectonic path).
    Compensated,
    /// This epoch's strata record. `deposition` **creates** it; the `eolian` +
    /// `wave` agents append to it (modifiers); `biotic` reads it — the edge that
    /// orders `biotic` after every erosion pass.
    Recorded,
    /// Terrain after the eolian agent. `eolian` writes; `wave` reads.
    Windblown,
    /// Terrain after the wave agent — the final erosion terrain. `wave` writes;
    /// `biotic` reads.
    Settled,
    /// The biotic modifier planes (`bio_weather` / `bio_resist`). `biotic`
    /// **creates** them; the erosion `weather` / `diffuse` / `eolian` passes read
    /// them **one epoch LATER** (`reads_prev`) — the loop-carried edge.
    BioMod,
    /// The biotic organic/charcoal units. `biotic` writes; nothing within the
    /// epoch reads them (a distinct axis so `biotic` need not be a modifier of
    /// `Recorded`, which would make it un-orderable against the agents).
    BioRecorded,
    /// The **`temperature` condition-field** (`dc:field/temperature`, §14) — the
    /// per-cell geothermal gradient the [`geotherm`](super::geotherm) field pass
    /// writes. Read by no *in-epoch* pass (coal rank samples it post-loop at
    /// finalize), so it is a pure write axis; a distinct token so the field pass
    /// declares an honest, orderable output on the runner.
    Geotherm,
    /// The per-cell **inventory-weathering fact-ledger sink** — the accumulating
    /// saprolite band the [`weather_inventory`](super::weather_inventory) pass writes
    /// (`Structure→Loose` on the bedrock seam, cause-carrying, every epoch). Like
    /// [`Geotherm`] it is a **pure write axis no in-epoch pass reads** (the collapse
    /// folds it post-loop); a distinct token so the material-transformation pass
    /// declares an honest, orderable output without perturbing the erosion pipeline.
    Saprolite,
    /// The **face-flux record sink** (FLOW slice 1, flow.md § 2) — the per-chapter
    /// flux-on-faces archive the [`flow_record`](super::flux) pass accumulates from
    /// the drainage solve's own outputs. Like [`Geotherm`] and [`Saprolite`] it is a
    /// **pure write axis no in-epoch pass reads** (the record is the seam to the
    /// *next* tier, not to this epoch), so declaring it cannot perturb erosion — and
    /// it is what makes the recording an ordered, self-declaring pass rather than a
    /// hook bolted onto the drainage solve.
    FlowFlux,
    /// The **`head` condition-field** (`dc:field/head`, §14; flow.md § 2.4) — the
    /// per-cell hydraulic **potential** the [`head`](super::head) field pass
    /// relaxes, plus its vertical-exchange companion. Unlike [`Geotherm`] this one
    /// **is** read in-epoch: `dc:deep/flow_record` consumes it to fill the
    /// vertical (slot↔slot) faces, which is the edge that orders the two. It is
    /// still a pure sidecar — nothing in the erosion pipeline reads it, so it
    /// cannot perturb the terrain.
    Head,
}

/// Owned working state the epoch loop threads through its passes. Constructed
/// once (from the seeded grid + the pre-loop climate march + biota init +
/// tectonic schedule), driven for `cfg.iterations` epochs, then destructured
/// into the [`super::DeepRun`]. It is the runner-side rich state the crossing
/// constraint keeps *off* the declaration seam.
pub struct DeepStepCtx<'a> {
    pub cfg: &'a DeepConfig,
    pub grid: DeepGrid,
    pub erosion: Erosion,
    pub biota: Option<BioticSim>,
    pub tec: TectonicSchedule,
    /// Scratch for the tectonic pass's per-iteration blended forcing plane
    /// (empty when tectonic history is off).
    pub blended: Vec<f64>,
    // --- per-epoch, set by the runner before each pass fires ---
    pub epoch: u32,
    pub sea_level: f64,
    /// The phase length handed to the firing pass (`= period`). Pinned/inert this
    /// movement — carried as the rate axis's time-base, scaled by nothing yet.
    pub dt: f64,
    // --- ledger accumulators (the old loop's running sums) ---
    pub uplift_total: f64,
    pub biotic_total: f64,
    pub thickening_total: f64,
    /// **Per-cell inventory-weathering accumulators** (journal/0094) — the saprolite
    /// band grown across epochs by the `dc:deep/weather_inventory` pass. Each is a
    /// **bedrock-only** [`FactLedger`] keyed at the stable sentinel slot 0
    /// (record-growth-invariant; see
    /// [`weather_bedrock_epoch`](super::weather_inventory::weather_bedrock_epoch)).
    /// **Empty** when `weather_inventory` is off (the pass is absent) ⇒ byte-identical.
    /// Re-keyed onto the final record post-loop
    /// ([`finalize_ledgers`](super::weather_inventory::finalize_ledgers)).
    ///
    /// **This is GEN-TIME scratch, and that is why it is still per-cell**
    /// (journal/0102). The pass appends into one cell's ledger every epoch, and an
    /// insert into a grid-wide array would memmove every fact after it — so the
    /// growable per-cell container is the right shape *here*, and the wrong shape
    /// for the record that ships. `finalize_ledgers` compacts these into one
    /// [`LedgerField`](super::inventory::LedgerField) and they are dropped.
    pub weather_ledgers: Vec<FactLedger>,
    /// **The face-flux accumulator** (FLOW slice 1) — the `dc:deep/flow_record`
    /// pass adds each epoch's routed discharge into it and it flushes to sparse
    /// per-chapter entries at every chapter boundary. Inactive (and the pass
    /// absent) when `flow_record` is off ⇒ byte-identical.
    pub flux: FluxAccum,
}

/// One self-declaring deep-time pass: identity + declared reads/writes (the
/// order axis) + cadence (the rate axis) + a bare-fn body. Plain data + opaque
/// ids — the crossing constraint.
#[derive(Debug)]
pub struct DeepPass {
    pub id: &'static str,
    /// Within-epoch reads — handed to the topo-sort as ordering inputs.
    pub reads: &'static [DeepAxis],
    pub writes: &'static [DeepAxis],
    /// **Lagged** (previous-epoch) reads — handed to the topo-sort as
    /// **anti-dependency** edges: this pass is ordered **BEFORE** every pass that
    /// writes these axes, so it provably observes the previous epoch's value
    /// before the in-place write clobbers it (journal/0104).
    ///
    /// That reversed direction is why a lagged read is a separate field rather
    /// than an entry in `reads`: moving it into `reads` would order this pass
    /// *after* the writer — the opposite of what it needs — and, for the
    /// biology↔erosion feedback, close a within-epoch cycle the runner would
    /// (correctly) reject. Declared here it is a loop-carried edge instead.
    ///
    /// **This was documentation until journal/0104.** `passgraph` never received
    /// it, so which epoch a lagged reader actually saw was an accident of the
    /// id-lexicographic tie-break — rename a pass and the physics changed
    /// silently. It is now enforced.
    pub reads_prev: &'static [DeepAxis],
    /// Cadence: the pass fires when `epoch % period == 0`. A `period > 1` pass is
    /// a coarse-rate pass — seeded before the loop and re-run every `period`
    /// epochs, so it never fires at epoch 0 (the seed).
    pub period: u32,
    pub body: for<'a> fn(&mut DeepStepCtx<'a>),
}

impl DeepPass {
    /// Whether this pass fires on `epoch`. Period 1 fires every epoch; a
    /// coarse-rate pass fires on its multiples but not epoch 0 (seeded).
    #[inline]
    fn fires(&self, epoch: u32) -> bool {
        epoch.is_multiple_of(self.period) && (self.period == 1 || epoch > 0)
    }
}

// -------------------------------------------------------------- the passes --
// Each body is exactly the call(s) the old `run_cells` loop / `Erosion::step`
// made, in the same order, unchanged — this is a re-housing, not a rewrite.
// `Erosion::step` itself is retained (direct callers — profiling harnesses and
// the erodibility/deeptime tests — still use it); these bodies drive the same
// public phase methods it composes.

/// Re-march orographic precipitation on the current topography (the coarse-rate
/// climate pass — [`DeepConfig::remarch_interval`]). Reads the terrain **before**
/// this epoch's forcing runs, so it is sequenced first by `forcing` reading
/// [`DeepAxis::Climate`].
fn climate_pass(ctx: &mut DeepStepCtx<'_>) {
    climate::march(&mut ctx.grid, ctx.sea_level);
}

/// Blend the two chapter forcing planes for this epoch, accumulate the
/// thickening ledger, and load the blended plane + chapter stamp into erosion.
fn tectonics_pass(ctx: &mut DeepStepCtx<'_>) {
    let chapter = ctx.tec.blend_into(ctx.cfg, ctx.epoch, &mut ctx.blended);
    ctx.thickening_total += ctx.blended.iter().sum::<f64>();
    ctx.erosion.set_tectonic(chapter, &ctx.blended);
}

/// External forcing, legacy path: add the constant uplift plane to bedrock. The
/// ledger value is the (constant) total uplift.
fn forcing_legacy_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.uplift_total += ctx.erosion.apply_uplift(&mut ctx.grid);
}

/// External forcing, tectonic path: add the blended analytic thickening rate to
/// the crustal columns (elevation is derived later by isostasy). No ledger here
/// — the tectonic path's ledger input is the isostatic injection.
fn forcing_tectonic_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.apply_thickening(&mut ctx.grid);
}

/// Expose the outcropping lithology and cache its agent susceptibility planes.
fn expose_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.expose(&ctx.grid, ctx.cfg);
}

/// The periglacial frost-weathering multiplier for this epoch (the frost agent).
fn frost_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.periglacial(&ctx.grid, ctx.cfg);
}

/// The drainage solve: snapshot the surface, priority-flood, D8-route, and
/// accumulate drainage area. A field pass — the routing solver.
fn drainage_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.build_surface(&ctx.grid);
    ctx.erosion.flood();
    ctx.erosion.route();
    ctx.erosion.accumulate_area();
}

/// Stream-power transport with cover shielding and bedrock incision (transport +
/// incision are one interleaved per-cell flux chain — they are not separable
/// byte-identically, so they stay one pass). On the tectonic path it first
/// snapshots bedrock (for the exhumation ledger), exactly where the old step did.
fn transport_pass(ctx: &mut DeepStepCtx<'_>) {
    if ctx.cfg.tectonic_history {
        ctx.erosion.snapshot_bedrock(&ctx.grid);
    }
    ctx.erosion.transport(&mut ctx.grid, ctx.cfg);
}

/// Subaerial bedrock → regolith weathering.
fn weather_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.weather(&mut ctx.grid, ctx.cfg);
}

/// Flux-limited hillslope diffusion of the regolith.
fn diffuse_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.diffuse(&mut ctx.grid, ctx.cfg);
}

/// Exhumation bookkeeping + Airy isostasy (the tectonic path). The ledger input
/// is the isostatic injection ΣΔR.
fn isostasy_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.track_exhumation(&mut ctx.grid);
    ctx.uplift_total += ctx.erosion.isostasy(&mut ctx.grid, ctx.cfg);
}

/// The strata recorder: stamp each cell's net thickness change under the tag
/// measured now.
fn deposition_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.record(&mut ctx.grid);
}

/// The eolian agent: wind deflation + downwind loess/dune deposition.
fn eolian_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.wind(&mut ctx.grid, ctx.cfg);
}

/// The littoral wave agent: wave-cut erosion at the current sea stand.
fn wave_pass(ctx: &mut DeepStepCtx<'_>) {
    ctx.erosion.wave(&mut ctx.grid, ctx.cfg);
}

/// **The geotherm — the first §5 field pass.** Recompute the per-cell
/// geothermal gradient (the `temperature` condition-field, `dc:field/temperature`)
/// from the evolved crustal columns (`t_crust`/`crust_kind`) and the current
/// chapter's analytic tectonic setting. It **plants a field and runs no edges** —
/// it never touches `R`/`H`/the record — so it perturbs the erosion result not at
/// all; only coal rank (post-loop) reads what it writes. A coarse-rate pass
/// ([`super::geotherm::GEOTHERM_PERIOD`]): heat flow evolves slowly.
fn geotherm_pass(ctx: &mut DeepStepCtx<'_>) {
    let extent_km = ctx.grid.w as f64 * ctx.grid.cell_m / 1000.0;
    let v_ref = super::tectonics::reference_velocity(ctx.cfg, extent_km);
    let chapter = ctx.erosion.current_chapter();
    // Disjoint field borrows: the plate table (immutable) and the grid (mutable).
    let plates = ctx.tec.plates_at(chapter);
    super::geotherm::march(&mut ctx.grid, plates, v_ref, ctx.cfg);
}

/// The biotic layer: reads this epoch's fresh post-erosion terrain + drainage,
/// deposits its organic record, and writes the modifiers the NEXT epoch's
/// erosion consumes.
fn biotic_pass(ctx: &mut DeepStepCtx<'_>) {
    if let Some(b) = ctx.biota.as_mut() {
        ctx.biotic_total += b.step(&mut ctx.grid, &ctx.erosion, ctx.epoch);
    }
}

/// **Inventory weathering as a per-epoch process** (journal/0094, Movement 3) —
/// the first *cellular* pass on the runner. Weathers each subaerial cell's bedrock
/// `Structure` seam into `Loose` saprolite on **this epoch's live inputs**
/// (contemporaneous regolith `H`, frost, biotic multiplier), **accumulating** the
/// cause-carrying facts across the whole loop into [`DeepStepCtx::weather_ledgers`].
///
/// **Two-authorities split** (material-behavior.md §11): it READS the terrain
/// (regolith cover shielding) but WRITES ONLY the ledger sidecar — it never touches
/// `R`/`H`, so the height-tier `dc:deep/weather` pass and the erosion result are
/// byte-unchanged (mirrors how the geotherm plants a field and runs no edges). Its
/// output axis [`DeepAxis::Saprolite`] is read by no in-epoch pass. `dt` is live —
/// the share scales by the pass's phase length.
fn weather_inventory_pass(ctx: &mut DeepStepCtx<'_>) {
    let chapter = ctx.erosion.current_chapter();
    super::weather_inventory::weather_epoch(
        &mut ctx.weather_ledgers,
        &ctx.grid,
        ctx.erosion.frost(),
        chapter,
        ctx.sea_level,
        ctx.dt,
        ctx.cfg,
    );
}

/// **The flow record — FLOW slice 1's recording half** (flow.md § 2). Add this
/// epoch's drainage solve to the per-chapter **face-flux** archive: each cell's
/// routed discharge is credited to the *face* it crossed (a D8 lateral face, or a
/// boundary face at a sink), together with the suspended load that crossed with it.
///
/// It replaces the **output representation** of the solve, never the solve: it
/// reads `recv`/`area`/`out_load`/the routed surface exactly as the sim computed
/// them, and writes only its own record ([`DeepAxis::FlowFlux`], which nothing in
/// the epoch reads) — so, like the geotherm and the inventory-weathering pass, it
/// cannot perturb the erosion result.
///
/// Reading [`DeepAxis::Energy`] is what sequences it **after** `dc:deep/transport`,
/// which is where the per-face load comes from; reading [`DeepAxis::Routed`] pins
/// it after the drainage solve whose output it is recording. Declare what you
/// read: it does not read the strata record, because the atom's stratum slot is
/// *derived* from the chapter stamp rather than stored (`flux::slot_for_chapter`).
fn flow_record_pass(ctx: &mut DeepStepCtx<'_>) {
    let chapter = ctx.erosion.current_chapter();
    let sea = ctx.sea_level;
    ctx.flux.add_epoch(
        chapter,
        ctx.erosion.recv(),
        ctx.erosion.area(),
        ctx.erosion.out_load(),
        ctx.erosion.routed_surface(),
        sea,
    );
    // FLOW continuation (a): the vertical (slot↔slot) faces slice 1 left honestly
    // zero, filled from the head field's exchange plane. A no-op — leaving them
    // zero exactly as before — when the head field is off and the plane is empty.
    ctx.flux
        .add_epoch_vertical(chapter, &ctx.grid.head_exchange);
}

/// **The head field — the §5 field pass that makes flow descend a POTENTIAL**
/// (flow.md § 2.4, continuation (a)). Relax the `head` condition-field
/// (`dc:field/head`) over the live topography and the live strata record: the
/// column's transmissivity, vertical conductivity and confinement are *derived*
/// from the units the recorder already stamped, and the potential is pinned at the
/// free water (sea, lakes, perennial streams) and left free — **uncapped** —
/// wherever a confining bed seals a permeable one. That uncapped degree of freedom
/// is artesian, and it is the thing the unconfined `H = y + sat` proxy cannot
/// express.
///
/// Like the geotherm it **plants a field and runs no edges** — it never touches
/// `R`/`H`/the record — so the erosion result is byte-unchanged. It reads the
/// drainage solve's filled surface (where free water stands) and drainage area
/// (which channels are perennial), and the **start-of-epoch** record for the
/// column's materials, exactly as `dc:deep/expose` does. A coarse-rate pass
/// ([`super::head::HEAD_PERIOD`]): groundwater equilibrates in millennia against a
/// 2.5 Myr epoch, so the cadence samples the topography rather than relaxing the
/// water.
fn head_pass(ctx: &mut DeepStepCtx<'_>) {
    let sea = ctx.sea_level;
    // Disjoint borrows: the erosion solve's planes (immutable) and the grid
    // (mutable). The ground surface is `R + H`, rebuilt here rather than cached —
    // flow.md § 10.2: nothing that stores a derived elevation may go stale when
    // compaction lands.
    let ground: Vec<f64> = (0..ctx.grid.w * ctx.grid.w)
        .map(|i| ctx.grid.surf_at(i))
        .collect();
    super::head::march(
        &mut ctx.grid,
        ctx.erosion.filled(),
        ctx.erosion.routed_surface(),
        &ground,
        ctx.erosion.area(),
        sea,
    );
}

// --- cfg-selected read slices for the terrain-revision pipeline -------------
// The token a downstream pass consumes depends on which upstream stages exist
// (isostasy only on the tectonic path, the agents only on `full_agents`), so a
// few `&'static` slices stand in for each combination.
use DeepAxis::*;
const DEP_READS_TEC: &[DeepAxis] = &[Energy, DeltaH, Climate, Compensated];
const DEP_READS_LEG: &[DeepAxis] = &[Energy, DeltaH, Climate, Diffused];
const EOL_READS_TEC: &[DeepAxis] = &[Exposed, Climate, Recorded, Compensated];
const EOL_READS_LEG: &[DeepAxis] = &[Exposed, Climate, Recorded, Diffused];
const BIO_READS_AGENTS: &[DeepAxis] = &[Routed, Frosted, Recorded, Settled];
const BIO_READS_TEC: &[DeepAxis] = &[Routed, Frosted, Recorded, Compensated];
const BIO_READS_LEG: &[DeepAxis] = &[Routed, Frosted, Recorded, Diffused];
// The inventory-weathering pass reads the **contemporaneous** settled terrain (for
// this-epoch regolith `H` cover-shielding), the frost multiplier, and **this epoch's
// BioMod** — sequencing it after the erosion pipeline so `H` is this epoch's. The
// terrain revision it reads is the last one the active roster produces (Settled with
// agents, Compensated on the bare tectonic path, Diffused legacy). Frosted exists
// only with the agent roster, so it appears only in the agents slice.
//
// **`BioMod` is a WITHIN-EPOCH read, not loop-carried** (spine-audit 2026-07-25).
// `weather_epoch` reads `grid.bio_weather`, which `dc:deep/biotic` overwrites *in
// place* each epoch — so a `reads_prev` declaration was a fiction: with no edge
// against `biotic`, which epoch's plane it saw was decided by the id-lexicographic
// tie-break (`dc:deep/biotic` < `dc:deep/weather_inventory` ⇒ **this** epoch's).
// Declaring it as a real `reads` makes the graph state what actually happens and
// PINS the order instead of inheriting it from a tie-break. It adds only
// `biotic → weather_inventory`, and nothing reads `Saprolite`, so there is no cycle.
//
// **`Exposed` is deliberately NOT declared**: susceptibility is a constant off
// `BEDROCK_SEAM_MATERIAL` (stub #16's one flat granite basement), so this pass does
// not read the outcropping lithology today. The genesis/emplacement heir that retires
// #16 re-adds it — declare what you read, not what you intend to read.
// The flow record reads the head field only when the head pass is in the roster.
// A read of an axis nobody writes is legal in loop mode, but declaring one the
// roster cannot satisfy would state a dependency that does not exist — declare
// what you read.
const FLOW_READS_HEAD: &[DeepAxis] = &[Routed, Energy, Head];
const FLOW_READS_BARE: &[DeepAxis] = &[Routed, Energy];
const WINV_READS_AGENTS: &[DeepAxis] = &[Settled, Frosted, BioMod];
const WINV_READS_TEC: &[DeepAxis] = &[Compensated, BioMod];
const WINV_READS_LEG: &[DeepAxis] = &[Diffused, BioMod];

/// Build the active pass roster for a config. A phase gated off (`tectonic_
/// history`, `full_agents`, `biotic`, `record`) is simply **absent** — the old
/// loop's `if` guards, expressed as pass presence — and the runner topo-sorts
/// whatever roster it is given. This roster reproduces `Erosion::step`'s exact
/// phase order for the production (all-flags-on) config; independent per-cell
/// passes (e.g. `expose` vs `frost`) may be interleaved differently by the
/// id-tie-break, which is byte-identical because their writes are disjoint.
///
/// **The biology↔erosion loop-carried edge**, made concrete: `biotic` *creates*
/// [`DeepAxis::BioMod`]; the erosion `weather`/`diffuse`/`eolian` passes read it —
/// but as a [`DeepPass::reads_prev`] (the previous epoch's value), NOT a `reads`.
/// So the within-epoch graph runs `erosion… → biotic` (via [`DeepAxis::Recorded`]
/// and [`DeepAxis::Settled`]) with no back-edge, hence no cycle. Move that
/// `BioMod` read into `reads` and the runner rejects the schedule as a
/// within-epoch cycle — the guarantee ecology.md wanted (a single-epoch pass
/// graph would refuse the biology↔erosion cycle), now enforced by the runner.
/// Proven in the tests.
///
/// And the lag itself is **enforced, not merely annotated** (journal/0104): each
/// `reads_prev` becomes a reader→writer anti-dependency, so `weather`/`diffuse`/
/// `eolian` are ordered before `biotic`, and the three passes that lag-read
/// [`DeepAxis::Recorded`] (`expose`, `frost`, `head`) are ordered before
/// `deposition`/`eolian`/`wave`. Those orderings held before this edge existed,
/// but only as a by-product of the id-lexicographic tie-break; they are now
/// rename-proof, which is what
/// `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename` proves.
pub fn deep_passes(cfg: &DeepConfig) -> Vec<DeepPass> {
    let mut passes = Vec::new();

    // Climate — the one coarse-rate pass already in the loop.
    passes.push(DeepPass {
        id: "dc:deep/climate",
        reads: &[],
        writes: &[Climate],
        // **Under-declared, and provably harmless** (audited by journal/0104,
        // filed as ROADMAP Owed): this pass really does lag-read the terrain (the
        // start-of-epoch topography = last epoch's final surface), and does not
        // say so. It is safe because the ordering it needs is pinned by a TRUE
        // forward edge, not by the tie-break: `forcing` reads `Climate`, and every
        // terrain writer in the roster is downstream of `forcing`, so climate is
        // provably ahead of all of them and always samples the pre-forcing surface
        // (byte-identical to the old loop's top-of-body `climate::march`).
        // Declaring it would be free — the anti-dependency edge is already implied
        // by the existing transitive closure, so it cannot move the schedule — but
        // it needs one cfg-selected slice per terrain-revision roster
        // (`Settled` / `Compensated` / `Diffused`).
        reads_prev: &[],
        period: cfg.remarch_interval.max(1),
        body: climate_pass,
    });

    // Tectonic forcing (blend) → erosion's forcing plane.
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/tectonics",
            reads: &[],
            writes: &[Forcing],
            reads_prev: &[],
            period: 1,
            body: tectonics_pass,
        });
    }

    // External forcing into the terrain / crustal columns. Reads Climate to
    // sequence the erosion pipeline after the epoch's marched climate (climate
    // samples the pre-forcing topography).
    passes.push(if cfg.tectonic_history {
        DeepPass {
            id: "dc:deep/forcing",
            reads: &[Climate, Forcing],
            writes: &[Forced, CrustThick],
            reads_prev: &[],
            period: 1,
            body: forcing_tectonic_pass,
        }
    } else {
        DeepPass {
            id: "dc:deep/forcing",
            reads: &[Climate],
            writes: &[Forced],
            reads_prev: &[],
            period: 1,
            body: forcing_legacy_pass,
        }
    });

    // Expose the outcropping lithology (reads the start-of-epoch record).
    passes.push(DeepPass {
        id: "dc:deep/expose",
        reads: &[],
        writes: &[Exposed],
        reads_prev: &[Recorded],
        period: 1,
        body: expose_pass,
    });

    // Periglacial frost agent.
    if cfg.full_agents {
        passes.push(DeepPass {
            id: "dc:deep/frost",
            reads: &[Forced],
            writes: &[Frosted],
            reads_prev: &[Recorded],
            period: 1,
            body: frost_pass,
        });
    }

    // Drainage solve (surface / flood / route / accumulate).
    passes.push(DeepPass {
        id: "dc:deep/drainage",
        reads: &[Forced],
        writes: &[Routed],
        reads_prev: &[],
        period: 1,
        body: drainage_pass,
    });

    // Transport + incision (one interleaved flux chain).
    passes.push(DeepPass {
        id: "dc:deep/transport",
        reads: &[Routed, Exposed],
        writes: &[Energy, DeltaH],
        reads_prev: &[],
        period: 1,
        body: transport_pass,
    });

    // The head field (FLOW continuation (a)): the `head` condition-field, relaxed
    // over the live topography and the start-of-epoch record. Gated behind
    // `head_field` (absent = off ⇒ the planes stay empty ⇒ the flow record's
    // vertical faces stay at slice 1's honest zero). Reads `Routed` (the filled
    // surface and drainage area — where free water stands and runs) and the
    // previous epoch's `Recorded` (what the column is made of), exactly as
    // `dc:deep/expose` does; writes only `Head`.
    if cfg.head_field {
        passes.push(DeepPass {
            id: "dc:deep/head",
            reads: &[Routed],
            writes: &[Head],
            reads_prev: &[Recorded],
            period: super::head::HEAD_PERIOD,
            body: head_pass,
        });
    }

    // The flow record (FLOW slice 1 + continuation (a)): flux on faces, per
    // chapter. Gated behind `flow_record` (absent = off), a pure sidecar over the
    // drainage solve's own outputs plus the head field — nothing in the epoch reads
    // FlowFlux, so it never perturbs erosion. Reading `Head` is what orders it after
    // the head field whose exchange fills its vertical faces; the read is declared
    // even when `head_field` is off, because what the pass reads is a property of
    // the pass, not of the roster (an absent writer is legal in loop mode — state
    // is seeded before the loop).
    if cfg.flow_record {
        passes.push(DeepPass {
            id: "dc:deep/flow_record",
            reads: if cfg.head_field {
                FLOW_READS_HEAD
            } else {
                FLOW_READS_BARE
            },
            writes: &[FlowFlux],
            reads_prev: &[],
            period: 1,
            body: flow_record_pass,
        });
    }

    // Bedrock weathering (reads last epoch's biotic weathering multiplier).
    passes.push(DeepPass {
        id: "dc:deep/weather",
        reads: &[DeltaH, Exposed, Frosted],
        writes: &[DeltaH, Weathered],
        reads_prev: &[BioMod],
        period: 1,
        body: weather_pass,
    });

    // Hillslope diffusion (reads last epoch's biotic root-cohesion resistance).
    passes.push(DeepPass {
        id: "dc:deep/diffuse",
        reads: &[Weathered, Exposed, DeltaH],
        writes: &[DeltaH, Diffused],
        reads_prev: &[BioMod],
        period: 1,
        body: diffuse_pass,
    });

    // Exhumation + isostasy (tectonic path).
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/isostasy",
            reads: &[Diffused, CrustThick],
            writes: &[Compensated],
            reads_prev: &[],
            period: 1,
            body: isostasy_pass,
        });
    }

    // The geotherm field pass (the `temperature` condition-field). Coarse-rate,
    // and only on the tectonic path — it solves over the crustal columns, which
    // exist only there. Reads Climate (surface temperature, the field's boundary
    // condition) and CrustThick (the crustal thickness the gradient reads); writes
    // the Geotherm axis, read by no in-epoch pass. Seeded pre-loop (mod.rs), like
    // the climate march.
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/geotherm",
            reads: &[Climate, CrustThick],
            writes: &[Geotherm],
            reads_prev: &[],
            period: super::geotherm::GEOTHERM_PERIOD,
            body: geotherm_pass,
        });
    }

    // Strata recorder.
    if cfg.record {
        passes.push(DeepPass {
            id: "dc:deep/deposition",
            reads: if cfg.tectonic_history {
                DEP_READS_TEC
            } else {
                DEP_READS_LEG
            },
            writes: &[Recorded],
            reads_prev: &[],
            period: 1,
            body: deposition_pass,
        });
    }

    // Eolian + wave agents (run after the recorder, self-record).
    if cfg.full_agents {
        passes.push(DeepPass {
            id: "dc:deep/eolian",
            reads: if cfg.tectonic_history {
                EOL_READS_TEC
            } else {
                EOL_READS_LEG
            },
            writes: &[Recorded, Windblown],
            reads_prev: &[BioMod],
            period: 1,
            body: eolian_pass,
        });
        passes.push(DeepPass {
            id: "dc:deep/wave",
            reads: &[Exposed, Recorded, Windblown],
            writes: &[Recorded, Settled],
            reads_prev: &[],
            period: 1,
            body: wave_pass,
        });
    }

    // Biotic layer (loop-carried: creates BioMod for next epoch's erosion).
    if cfg.biotic {
        let reads: &[DeepAxis] = if cfg.full_agents {
            BIO_READS_AGENTS
        } else if cfg.tectonic_history {
            BIO_READS_TEC
        } else {
            BIO_READS_LEG
        };
        passes.push(DeepPass {
            id: "dc:deep/biotic",
            reads,
            writes: &[BioMod, BioRecorded],
            reads_prev: &[],
            period: 1,
            body: biotic_pass,
        });
    }

    // Inventory weathering (journal/0094): the first cellular material pass, gated
    // behind `weather_inventory` (absent = off, the old `if` as pass presence — so
    // the production world is byte-identical off, the S-5 identity default). Reads
    // the contemporaneous terrain (for this-epoch regolith `H`), Frosted, and **this
    // epoch's BioMod as a real within-epoch read** (see WINV_READS_* — the former
    // `reads_prev` was a fiction the tie-break decided; spine-audit 2026-07-25).
    // Writes only the Saprolite ledger sink (no in-epoch reader), so it never
    // perturbs erosion.
    if cfg.weather_inventory {
        let reads: &[DeepAxis] = if cfg.full_agents {
            WINV_READS_AGENTS
        } else if cfg.tectonic_history {
            WINV_READS_TEC
        } else {
            WINV_READS_LEG
        };
        passes.push(DeepPass {
            id: "dc:deep/weather_inventory",
            reads,
            writes: &[Saprolite],
            reads_prev: &[],
            period: 1,
            body: weather_inventory_pass,
        });
    }

    passes
}

/// A validated, topo-sorted deep-time schedule.
#[derive(Debug)]
pub struct DeepSchedule {
    passes: Vec<DeepPass>,
    /// Indices into `passes`, in execution order.
    order: Vec<usize>,
}

impl DeepSchedule {
    /// Validate + topo-sort in **loop mode** (`require_creator = false`: state is
    /// seeded before the loop). **Both** edge kinds are handed to the kernel: the
    /// live `reads` (writer → reader) and the lagged `reads_prev` as
    /// reader → writer anti-dependencies (journal/0104). Handing the lag over is
    /// what makes it a mechanism instead of a comment — before that, which epoch a
    /// lagged reader saw was decided by the id-lexicographic tie-break.
    pub fn new(passes: Vec<DeepPass>) -> Result<Self, GraphError<DeepAxis>> {
        let decls: Vec<Decl<DeepAxis>> = passes
            .iter()
            .map(|p| Decl {
                id: p.id,
                reads: p.reads,
                writes: p.writes,
                reads_prev: p.reads_prev,
            })
            .collect();
        let scheduled = passgraph::schedule(&decls, false)?;
        Ok(Self {
            passes,
            order: scheduled.order,
        })
    }

    /// Pass ids in execution order (for tests and diagnostics).
    pub fn ordered_ids(&self) -> Vec<&'static str> {
        self.order.iter().map(|&i| self.passes[i].id).collect()
    }

    /// Drive the epoch loop: for each epoch, set the per-epoch state, then run
    /// every pass in topo order that fires this epoch, handing it its `dt`. This
    /// is the whole scheduler — order from the topo-sort, rate from the cadence.
    pub fn run(&self, ctx: &mut DeepStepCtx<'_>) {
        for it in 0..ctx.cfg.iterations {
            ctx.epoch = it;
            ctx.sea_level = sea_level_at(ctx.cfg, it);
            // The erosion phase methods read `self.sea_level` (the paleo-sea-level
            // stand); the old `Erosion::step` set it as its first line. Set it once
            // per epoch here so every erosion sub-pass sees the same stand — this
            // is load-bearing for byte-identity (the sinusoidal stand drives the
            // shoreline, marine deposition, and the wave agent).
            ctx.erosion.set_sea_level(ctx.sea_level);
            for &i in &self.order {
                let p = &self.passes[i];
                if p.fires(it) {
                    ctx.dt = f64::from(p.period);
                    (p.body)(ctx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_on() -> DeepConfig {
        // The production flag set (biotic + erodibility + tectonic_history +
        // full_agents + record all on) — the config the goldens run.
        DeepConfig {
            tectonic_history: true,
            biotic: true,
            full_agents: true,
            erodibility: true,
            ..DeepConfig::default()
        }
    }

    const PRODUCTION_ORDER: [&str; 17] = [
        "dc:deep/climate",
        "dc:deep/expose",
        "dc:deep/tectonics",
        "dc:deep/forcing",
        "dc:deep/drainage",
        "dc:deep/frost",
        // The geotherm field pass sorts in here: it becomes ready once `forcing`
        // has written CrustThick (and climate has written Climate), and among the
        // ready pool the id-tie-break places `dc:deep/geotherm` after `frost` and
        // before `transport`. It plants a field and has no in-epoch reader, so its
        // position never affects the erosion result.
        "dc:deep/geotherm",
        // The head field sorts in here: it becomes ready once `drainage` has
        // written Routed, and among the ready pool the id-tie-break places
        // `dc:deep/head` after `geotherm` and before `transport`. Like the geotherm
        // it plants a field and no *erosion* pass reads it, so its position never
        // affects the terrain — only `dc:deep/flow_record` reads it, and that pass
        // writes nothing the epoch consumes either.
        "dc:deep/head",
        "dc:deep/transport",
        // The flow record sorts in here: it becomes ready once `transport` has
        // written Energy (and `drainage` Routed), and among the ready pool the
        // id-tie-break places `dc:deep/flow_record` before `weather`. Like the
        // geotherm it writes an axis no in-epoch pass reads, so its position
        // never affects the erosion result.
        "dc:deep/flow_record",
        "dc:deep/weather",
        "dc:deep/diffuse",
        "dc:deep/isostasy",
        "dc:deep/deposition",
        "dc:deep/eolian",
        "dc:deep/wave",
        "dc:deep/biotic",
    ];

    #[test]
    fn order_reproduces_the_erosion_step_phase_order() {
        let sched = DeepSchedule::new(deep_passes(&all_on())).expect("valid schedule");
        // climate/expose/tectonics are mutually independent (disjoint writes) so
        // the id-tie-break interleaves them; the load-bearing chain — forcing →
        // drainage → transport → weather → diffuse → isostasy → deposition →
        // eolian → wave → biotic — is exactly `Erosion::step`'s phase order.
        assert_eq!(sched.ordered_ids(), PRODUCTION_ORDER.to_vec());
    }

    #[test]
    fn order_ignores_registration_order() {
        let mut passes = deep_passes(&all_on());
        passes.reverse();
        let sched = DeepSchedule::new(passes).expect("still valid");
        assert_eq!(sched.ordered_ids(), PRODUCTION_ORDER.to_vec());
    }

    #[test]
    fn the_biology_erosion_lag_must_be_loop_carried_or_the_runner_rejects_a_cycle() {
        // The lag, declared WRONG: an erosion pass reads BioMod as a within-epoch
        // read (not `reads_prev`). `erosion` creates Routed that `biotic` reads
        // (erosion → biotic); `biotic` creates BioMod that `erosion` reads
        // (biotic → erosion): a 2-cycle the runner rejects by name. This is the
        // guarantee ecology.md wanted — a single-epoch graph refuses the cycle —
        // now enforced, not just commented.
        use DeepAxis::*;
        let passes = vec![
            DeepPass {
                id: "dc:deep/erosion",
                reads: &[BioMod], // WRONG: should be reads_prev
                writes: &[Routed],
                reads_prev: &[],
                period: 1,
                body: transport_pass,
            },
            DeepPass {
                id: "dc:deep/biotic",
                reads: &[Routed],
                writes: &[BioMod],
                reads_prev: &[],
                period: 1,
                body: biotic_pass,
            },
        ];
        let err = DeepSchedule::new(passes).unwrap_err();
        assert!(matches!(err, GraphError::Cycle(_)), "{err:?}");

        // Declared RIGHT, the same pair is schedulable — and the anti-dependency
        // edge orders the lagged reader ahead of the writer, which is what makes
        // "it reads LAST epoch's BioMod" true rather than hopeful (journal/0104).
        let passes = vec![
            DeepPass {
                id: "dc:deep/erosion",
                reads: &[],
                writes: &[Routed],
                reads_prev: &[BioMod],
                period: 1,
                body: transport_pass,
            },
            DeepPass {
                id: "dc:deep/biotic",
                reads: &[Routed],
                writes: &[BioMod],
                reads_prev: &[],
                period: 1,
                body: biotic_pass,
            },
        ];
        assert_eq!(
            DeepSchedule::new(passes).unwrap().ordered_ids(),
            vec!["dc:deep/erosion", "dc:deep/biotic"],
        );
    }

    /// **The point of journal/0104.** `reads_prev` used to be consumed by nothing,
    /// so a lagged reader saw the previous epoch's plane only if the
    /// id-lexicographic tie-break happened to park it ahead of the writer. Rename
    /// the pass and the physics changed — silently, with every test still green.
    ///
    /// So: rename each lagged reader to an id that sorts **after** every writer of
    /// the axis it lags on, and assert the schedule still puts it first. Under the
    /// old kernel `dc:deep/zzz_head` would slide behind `dc:deep/deposition` and
    /// start reading THIS epoch's strata record; here the anti-dependency edge
    /// holds it in place and the tie-break never gets a say.
    #[test]
    fn a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename() {
        // Every (lagged reader, lagged axis) pair in the production roster, with a
        // rename chosen to lose the alphabet fight against that axis's writers.
        for (victim, hostile) in [
            ("dc:deep/head", "dc:deep/zzz_head"),
            ("dc:deep/expose", "dc:deep/zzz_expose"),
            ("dc:deep/frost", "dc:deep/zzz_frost"),
            ("dc:deep/weather", "dc:deep/zzz_weather"),
            ("dc:deep/diffuse", "dc:deep/zzz_diffuse"),
            ("dc:deep/eolian", "dc:deep/zzz_eolian"),
        ] {
            let mut passes = deep_passes(&all_on());
            let lagged: Vec<DeepAxis> = passes
                .iter()
                .find(|p| p.id == victim)
                .unwrap_or_else(|| panic!("{victim} is in the production roster"))
                .reads_prev
                .to_vec();
            assert!(!lagged.is_empty(), "{victim} declares a lagged read");
            // Every pass that writes an axis the victim lag-reads.
            let writers: Vec<&'static str> = passes
                .iter()
                .filter(|p| p.id != victim && p.writes.iter().any(|w| lagged.contains(w)))
                .map(|p| p.id)
                .collect();
            assert!(!writers.is_empty(), "{victim}'s lagged axis has writers");
            assert!(
                writers.iter().all(|w| *w < hostile),
                "the rename must lose the tie-break against {writers:?}, or this \
                 test proves nothing"
            );

            for p in &mut passes {
                if p.id == victim {
                    p.id = hostile;
                }
            }
            let order = DeepSchedule::new(passes)
                .expect("the rename must not make the roster unschedulable")
                .ordered_ids();
            let at = |id: &str| order.iter().position(|x| *x == id).expect("scheduled");
            for w in writers {
                assert!(
                    at(hostile) < at(w),
                    "{hostile} lag-reads {lagged:?} and must stay ahead of its \
                     writer {w}; got {order:?}"
                );
            }
        }
    }

    /// The invariant the anti-dependency edge buys, checked across the whole
    /// production roster rather than one pass: **no lagged reader is ever
    /// scheduled after a writer of the axis it lags on.** Without the edge this is
    /// merely true; with it, it is guaranteed.
    #[test]
    fn every_lagged_read_in_the_roster_is_ordered_before_its_writers() {
        for cfg in [
            all_on(),
            DeepConfig {
                weather_inventory: true,
                ..all_on()
            },
            DeepConfig {
                head_field: false,
                ..all_on()
            },
            DeepConfig {
                full_agents: false,
                ..all_on()
            },
            DeepConfig {
                tectonic_history: false,
                ..all_on()
            },
        ] {
            let passes = deep_passes(&cfg);
            let order = DeepSchedule::new(deep_passes(&cfg))
                .expect("valid roster")
                .ordered_ids();
            let at = |id: &str| order.iter().position(|x| *x == id).expect("scheduled");
            for reader in &passes {
                for axis in reader.reads_prev {
                    // A lagged read is never also a live read of the same axis —
                    // the two edge kinds point in opposite directions.
                    assert!(!reader.reads.contains(axis), "{} {axis:?}", reader.id);
                    for w in passes.iter().filter(|p| p.writes.contains(axis)) {
                        if w.id != reader.id {
                            assert!(
                                at(reader.id) < at(w.id),
                                "{} lag-reads {axis:?} but runs after its writer {}",
                                reader.id,
                                w.id
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_geotherm_is_a_declared_coarse_rate_field_pass() {
        let passes = deep_passes(&all_on());
        let geo = passes
            .iter()
            .find(|p| p.id == "dc:deep/geotherm")
            .expect("the geotherm pass is scheduled on the tectonic path");
        // It is a real field pass: declares its reads/writes over the deep-cell
        // axis vocabulary (surface temperature via Climate, crustal thickness via
        // CrustThick), and writes the temperature field.
        assert_eq!(geo.reads, &[DeepAxis::Climate, DeepAxis::CrustThick]);
        assert_eq!(geo.writes, &[DeepAxis::Geotherm]);
        // Coarse rate — heat flow evolves slowly, so it re-marches on a cadence
        // and, like climate, is seeded before the loop (never fires at epoch 0).
        assert!(geo.period > 1);
        assert!(!geo.fires(0));
        assert!(geo.fires(geo.period));
        // The schedule as a whole is valid with the field pass in it.
        let sched = DeepSchedule::new(deep_passes(&all_on())).expect("valid with the geotherm");
        assert!(sched.ordered_ids().contains(&"dc:deep/geotherm"));
    }

    /// **The head field is a declared coarse-rate FIELD pass** (flow.md § 2.4,
    /// material-behavior.md §5) — and, unlike the geotherm, it has a declared
    /// in-epoch *reader*: `dc:deep/flow_record` reads `Head` to fill the vertical
    /// faces, which is the edge that orders the two. That edge must be real, not a
    /// tie-break, or "the record's vertical faces come from the head field" is a
    /// comment rather than a guarantee (the spine-audit 2026-07-25 lesson).
    #[test]
    fn the_head_field_is_a_declared_field_pass_the_flow_record_reads() {
        let passes = deep_passes(&all_on());
        let head = passes
            .iter()
            .find(|p| p.id == "dc:deep/head")
            .expect("the head field pass is scheduled by default");
        assert_eq!(head.reads, &[DeepAxis::Routed]);
        assert_eq!(head.reads_prev, &[DeepAxis::Recorded]);
        assert_eq!(head.writes, &[DeepAxis::Head]);
        // Coarse rate — groundwater equilibrates in millennia against a 2.5 Myr
        // epoch, so this samples the topography rather than relaxing the water.
        assert!(head.period > 1);
        assert!(!head.fires(0), "seeded pre-loop, never fired at epoch 0");
        assert!(head.fires(head.period));

        let rec = passes
            .iter()
            .find(|p| p.id == "dc:deep/flow_record")
            .expect("the flow record is on by default");
        assert!(
            rec.reads.contains(&DeepAxis::Head),
            "the flow record must DECLARE the head field it consumes, so the order \
             is pinned by the graph and not by the id tie-break"
        );
        let order = DeepSchedule::new(deep_passes(&all_on()))
            .expect("valid with the head field")
            .ordered_ids();
        let at = |id: &str| order.iter().position(|x| *x == id).expect("scheduled");
        assert!(at("dc:deep/head") < at("dc:deep/flow_record"));
        assert!(at("dc:deep/drainage") < at("dc:deep/head"));
    }

    /// Off, the pass is **absent** — the old `if` as pass presence — so the head
    /// planes stay empty and the flow record's vertical faces stay at FLOW slice
    /// 1's honest zero. And the flow record must then stop declaring a read it no
    /// longer has: declare what you read, not what you intend to read.
    #[test]
    fn the_head_field_is_absent_when_the_flag_is_off() {
        let cfg = DeepConfig {
            head_field: false,
            ..all_on()
        };
        let passes = deep_passes(&cfg);
        assert!(passes.iter().all(|p| p.id != "dc:deep/head"));
        let rec = passes
            .iter()
            .find(|p| p.id == "dc:deep/flow_record")
            .expect("the flow record is still on");
        assert!(!rec.reads.contains(&DeepAxis::Head));
        DeepSchedule::new(deep_passes(&cfg)).expect("valid without the head field");
    }

    #[test]
    fn the_geotherm_is_absent_off_the_tectonic_path() {
        // No crustal columns without tectonic history, so the field pass that
        // solves over them is simply not scheduled (pass presence = the old `if`).
        let cfg = DeepConfig {
            tectonic_history: false,
            biotic: true,
            full_agents: true,
            erodibility: true,
            ..DeepConfig::default()
        };
        let passes = deep_passes(&cfg);
        assert!(!passes.iter().any(|p| p.id == "dc:deep/geotherm"));
    }

    #[test]
    fn weather_inventory_is_absent_off_and_a_declared_cellular_pass_on() {
        // Off (production default): the pass is not scheduled — the byte-identity
        // default (pass presence = the old `if`).
        assert!(
            !deep_passes(&all_on())
                .iter()
                .any(|p| p.id == "dc:deep/weather_inventory"),
            "weather_inventory is off by default ⇒ absent"
        );

        // On: a declared cellular pass, period 1 (fires every epoch), writing only
        // the Saprolite sink and reading the settled terrain + frost (contemporaneous
        // H) + **this epoch's BioMod as a real within-epoch read**.
        let cfg = DeepConfig {
            weather_inventory: true,
            ..all_on()
        };
        let passes = deep_passes(&cfg);
        let w = passes
            .iter()
            .find(|p| p.id == "dc:deep/weather_inventory")
            .expect("scheduled when flagged on");
        assert_eq!(w.writes, &[DeepAxis::Saprolite]);
        assert!(w.reads.contains(&DeepAxis::Settled) && w.reads.contains(&DeepAxis::Frosted));
        assert_eq!(w.period, 1);
        assert!(w.fires(0) && w.fires(1));

        // **The declaration must state what the pass DOES** (spine-audit 2026-07-25).
        // `weather_epoch` reads `grid.bio_weather`, which `biotic` overwrites in place
        // each epoch — so it observes THIS epoch's plane, and a `reads_prev` claim was
        // a fiction the id-tie-break happened to satisfy. BioMod is a real within-epoch
        // read; nothing is loop-carried here.
        assert!(
            w.reads.contains(&DeepAxis::BioMod),
            "BioMod is a within-epoch read: the pass sees this epoch's plane"
        );
        assert_eq!(w.reads_prev, &[], "nothing is loop-carried on this pass");
        // `Exposed` is NOT read: susceptibility is a constant off the stub-#16 bedrock
        // seam. The genesis heir that retires #16 re-adds it.
        assert!(!w.reads.contains(&DeepAxis::Exposed));

        // The schedule is valid with it, and it sorts AFTER the terrain settles and
        // biology runs (it reads Settled which wave writes, and it is the last thing
        // the graph must place — a pure sink perturbs nothing before it).
        let sched = DeepSchedule::new(deep_passes(&cfg)).expect("valid with weather_inventory");
        let order = sched.ordered_ids();
        let pos = |id: &str| order.iter().position(|&x| x == id).unwrap();
        assert!(pos("dc:deep/weather_inventory") > pos("dc:deep/wave"));
        assert!(pos("dc:deep/weather_inventory") > pos("dc:deep/frost"));
        // **PINNED, not inherited from a tie-break.** Declaring BioMod as a real read
        // adds the `biotic → weather_inventory` edge, so the order the pass actually
        // depends on is now enforced by the graph. Before the fix this held only
        // because `dc:deep/biotic` sorts lexicographically first among the ready pool
        // — rename either pass and the physics would have changed silently.
        assert!(pos("dc:deep/weather_inventory") > pos("dc:deep/biotic"));

        // Off-flag order is exactly the production order (the pass added nothing).
        assert_eq!(
            DeepSchedule::new(deep_passes(&all_on()))
                .unwrap()
                .ordered_ids(),
            PRODUCTION_ORDER.to_vec()
        );
    }

    #[test]
    fn climate_is_a_coarse_rate_pass_seeded_at_epoch_zero() {
        let passes = deep_passes(&all_on());
        let climate = passes.iter().find(|p| p.id == "dc:deep/climate").unwrap();
        assert!(climate.period > 1, "climate re-marches on a coarse cadence");
        // Seeded before the loop, so it does not fire at epoch 0 in-loop, then
        // fires on its multiples — exactly the old `it > 0 && it % remarch == 0`.
        assert!(!climate.fires(0));
        assert!(climate.fires(climate.period));
        assert!(!climate.fires(1));
        // the erosion sub-passes are rate-1: fire every epoch including 0.
        let transport = passes.iter().find(|p| p.id == "dc:deep/transport").unwrap();
        assert!(transport.fires(0) && transport.fires(1));
    }
}
